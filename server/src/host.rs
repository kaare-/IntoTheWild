//! The one authoritative process. Intents are queued until the window
//! closes; the core then reads one starting state and commits once.

use crate::persist::{Record, Store};
use serde::Serialize;
use std::collections::BTreeMap;
use std::path::Path;
use std::time::Instant;
use villmarks_sim::{
    help_text, parse_attempt, Attempt, CommandResult, Intent, Observation, Tuning, World,
};

#[derive(Clone, Debug, Serialize)]
pub struct Ack {
    pub ok: bool,
    pub command_id: String,
    pub duplicate: bool,
    pub late: bool,
    pub immediate: bool,
    pub window: u64,
    pub message: String,
}

#[derive(Clone, Debug, Serialize)]
pub struct Joined {
    pub actor_id: u32,
    pub name: String,
    pub token: String,
}

#[derive(Clone, Debug, Serialize)]
pub struct Clock {
    pub sim_time: u64,
    pub raining: bool,
    pub manual: bool,
    pub window_secs: f64,
    pub closes_in_ms: Option<u64>,
    pub queued: usize,
}

struct Pending {
    window: u64,
    late: bool,
}

/// A gather or a light the person reads when the window closes, including a
/// gather the arms cannot hold. It is not copied into their events.
#[derive(Clone, Debug, Serialize)]
pub struct SpokenReply {
    pub command_id: String,
    pub actor_id: u32,
    pub message: String,
}

#[derive(Clone, Debug, Serialize)]
pub struct AdvanceReport {
    pub sim_time: u64,
    pub replies: Vec<SpokenReply>,
}

pub struct Host {
    world: World,
    store: Store,
    queued: Vec<Intent>,
    pending: BTreeMap<String, Pending>,
    settled: BTreeMap<String, Ack>,
    latest_replies: Vec<SpokenReply>,
    pub manual: bool,
    window_secs: f64,
    deadline: Instant,
}

impl Host {
    pub fn open(
        data: &Path,
        config: &Path,
        seed: u64,
        snapshot_every: u32,
        manual: bool,
        window_secs: f64,
    ) -> Result<Self, String> {
        let text = std::fs::read_to_string(config).map_err(|err| err.to_string())?;
        let tuning = Tuning::from_toml(&text)?;
        let seed_world = World::valley(tuning, seed);
        let mut store = Store::open(data, snapshot_every)?;
        let world = store.load_or_seed(seed_world)?;
        Ok(Host {
            world,
            store,
            queued: Vec::new(),
            pending: BTreeMap::new(),
            settled: BTreeMap::new(),
            latest_replies: Vec::new(),
            manual,
            window_secs,
            deadline: Instant::now() + duration_from_secs(window_secs),
        })
    }

    pub fn join(&mut self, name: &str) -> Result<Joined, String> {
        let token = fresh_token();
        let actor_id = self.world.join(name, &token)?;
        let joined_name = self
            .world
            .observe(actor_id)
            .map(|obs| obs.self_info.name)
            .unwrap_or_else(|| name.to_string());
        self.store.commit(
            &self.world,
            &Record::Join {
                name: joined_name.clone(),
                token: token.clone(),
            },
        )?;
        Ok(Joined {
            actor_id,
            name: joined_name,
            token,
        })
    }

    pub fn submit(
        &mut self,
        token: &str,
        command_id: &str,
        line: &str,
        window_hint: Option<u64>,
    ) -> Ack {
        let Some(actor_id) = self.world.actor_by_token(token) else {
            return bad(
                command_id,
                "That token does not match anyone in the valley.",
            );
        };
        if command_id.is_empty() || command_id.len() > 80 {
            return bad(command_id, "command_id must be 1 to 80 characters.");
        }
        if let Some(previous) = self.world.stored_result(command_id) {
            return self.duplicate_resolved(previous);
        }
        if let Some(mut settled) = self.settled.get(command_id).cloned() {
            settled.duplicate = true;
            return settled;
        }
        if let Some(pending) = self.pending.get(command_id) {
            return Ack {
                ok: true,
                command_id: command_id.to_string(),
                duplicate: true,
                late: pending.late,
                immediate: false,
                window: pending.window,
                message: "Already queued for this window.".into(),
            };
        }
        let trimmed = line.trim();
        if trimmed.eq_ignore_ascii_case("help") || trimmed == "?" {
            return self.settle(command_id, false, false, true, help_text().into());
        }
        let attempt = match parse_attempt(trimmed) {
            Ok(attempt) => attempt,
            Err(message) => {
                let ack = bad(command_id, &message);
                self.settled.insert(command_id.to_string(), ack.clone());
                return ack;
            }
        };
        match attempt {
            Attempt::Arms => {
                let text = self.world.arms_line(actor_id);
                self.settle(command_id, false, false, true, text)
            }
            Attempt::NameSnare => {
                self.settle(command_id, false, false, true, "Set a snare.".into())
            }
            Attempt::NameCanoe => self.settle(
                command_id,
                false,
                false,
                true,
                "Board, carve canoe, or paddle.".into(),
            ),
            Attempt::NameSinew => self.settle(
                command_id,
                false,
                false,
                true,
                "Pull sinew from a carcass.".into(),
            ),
            Attempt::NameTorch => self.settle(
                command_id,
                false,
                false,
                true,
                "Make a pitch torch, or light one.".into(),
            ),
            Attempt::NameNeedle => self.settle(
                command_id,
                false,
                false,
                true,
                "Carve a bone needle.".into(),
            ),
            Attempt::NameAntler => self.settle(
                command_id,
                false,
                false,
                true,
                "Carve an antler pick.".into(),
            ),
            Attempt::NamePick => self.settle(
                command_id,
                false,
                false,
                true,
                "Dig worms from dry ground.".into(),
            ),
            Attempt::NameMushroom => self.settle(
                command_id,
                false,
                false,
                true,
                "Gather mushrooms, or cook them.".into(),
            ),
            Attempt::NameBasket => {
                let text = self.world.basket_name_line(actor_id);
                self.settle(command_id, false, false, true, text)
            }
            Attempt::Look => {
                let text = self
                    .world
                    .observe(actor_id)
                    .map(|obs| format!("{}\n\n{}\n{}", obs.text, obs.map, obs.legend))
                    .unwrap_or_else(|| "You cannot see.".into());
                self.settle(command_id, false, false, true, text)
            }
            Attempt::Examine(target) => {
                let text = self.world.examine(actor_id, target);
                self.settle(command_id, false, false, true, text)
            }
            other => {
                let late = window_hint
                    .map(|hint| hint < self.world.sim_time())
                    .unwrap_or(false);
                let window = self.world.sim_time();
                self.queued.push(Intent {
                    command_id: command_id.to_string(),
                    actor_id,
                    attempt: other,
                });
                self.pending
                    .insert(command_id.to_string(), Pending { window, late });
                Ack {
                    ok: true,
                    command_id: command_id.to_string(),
                    duplicate: false,
                    late,
                    immediate: false,
                    window,
                    message: format!(
                        "Queued for the window at tick {window}.{}",
                        if late {
                            " It arrived after that earlier window had closed, so it waits for this one."
                        } else {
                            ""
                        }
                    ),
                }
            }
        }
    }

    pub fn observe(&self, token: &str) -> Option<Observation> {
        let actor = self.world.actor_by_token(token)?;
        self.world.observe(actor)
    }

    pub fn clock(&self) -> Clock {
        let closes_in_ms = if self.manual {
            None
        } else {
            Some(
                self.deadline
                    .saturating_duration_since(Instant::now())
                    .as_millis() as u64,
            )
        };
        Clock {
            sim_time: self.world.sim_time(),
            raining: self.world.is_raining(),
            manual: self.manual,
            window_secs: self.window_secs,
            closes_in_ms,
            queued: self.queued.len(),
        }
    }

    pub fn seal(&mut self) -> Result<u64, String> {
        let replies = self.seal_once()?;
        self.latest_replies = replies;
        self.deadline = Instant::now() + duration_from_secs(self.window_secs);
        Ok(self.world.sim_time())
    }

    pub fn advance(&mut self, steps: u32) -> Result<AdvanceReport, String> {
        if !self.manual {
            return Err(
                "The valley advances on its own clock. Omit --manual to play that way.".into(),
            );
        }
        let steps = steps.clamp(1, 500);
        let mut replies = Vec::new();
        let mut sim_time = self.world.sim_time();
        for _ in 0..steps {
            replies.extend(self.seal_once()?);
            sim_time = self.world.sim_time();
            self.deadline = Instant::now() + duration_from_secs(self.window_secs);
        }
        self.latest_replies = replies.clone();
        Ok(AdvanceReport { sim_time, replies })
    }

    /// Replies from the latest seal, for the person this token names.
    pub fn replies_for(&self, token: &str) -> Vec<SpokenReply> {
        let Some(actor) = self.world.actor_by_token(token) else {
            return Vec::new();
        };
        self.latest_replies
            .iter()
            .filter(|reply| reply.actor_id == actor)
            .cloned()
            .collect()
    }

    fn seal_once(&mut self) -> Result<Vec<SpokenReply>, String> {
        let intents = std::mem::take(&mut self.queued);
        self.pending.clear();
        let report = self.world.step(&intents);
        let replies = spoken_replies(&intents, &report.results);
        self.store.commit(&self.world, &Record::Step { intents })?;
        Ok(replies)
    }

    pub fn snapshot(&mut self) -> Result<(), String> {
        self.store.snapshot_now(&self.world)
    }

    pub fn world(&self) -> &World {
        &self.world
    }

    fn settle(
        &mut self,
        command_id: &str,
        late: bool,
        duplicate: bool,
        immediate: bool,
        message: String,
    ) -> Ack {
        let ack = Ack {
            ok: true,
            command_id: command_id.to_string(),
            duplicate,
            late,
            immediate,
            window: self.world.sim_time(),
            message,
        };
        self.settled.insert(command_id.to_string(), ack.clone());
        ack
    }

    fn duplicate_resolved(&self, previous: CommandResult) -> Ack {
        Ack {
            ok: true,
            command_id: previous.command_id,
            duplicate: true,
            late: false,
            immediate: true,
            window: self.world.sim_time(),
            message: previous.message,
        }
    }
}

fn spoken_replies(intents: &[Intent], results: &[CommandResult]) -> Vec<SpokenReply> {
    let mut attempts = BTreeMap::new();
    for intent in intents {
        attempts.insert(intent.command_id.as_str(), &intent.attempt);
    }
    results
        .iter()
        .filter_map(|result| {
            let attempt = attempts.get(result.command_id.as_str())?;
            let spoken = match attempt {
                Attempt::Gather(_) | Attempt::Light if result.success => true,
                Attempt::Gather(_)
                    if !result.success
                        && result.message == "Your arms are full. Drop something first." =>
                {
                    true
                }
                _ => false,
            };
            if !spoken {
                return None;
            }
            Some(SpokenReply {
                command_id: result.command_id.clone(),
                actor_id: result.actor_id,
                message: result.message.clone(),
            })
        })
        .collect()
}

fn bad(command_id: &str, message: &str) -> Ack {
    Ack {
        ok: false,
        command_id: command_id.to_string(),
        duplicate: false,
        late: false,
        immediate: true,
        window: 0,
        message: message.to_string(),
    }
}

fn fresh_token() -> String {
    let mut bytes = [0u8; 16];
    getrandom::getrandom(&mut bytes).expect("token entropy");
    bytes.iter().map(|byte| format!("{byte:02x}")).collect()
}

fn duration_from_secs(secs: f64) -> std::time::Duration {
    if secs <= 0.0 {
        std::time::Duration::from_millis(1)
    } else {
        std::time::Duration::from_secs_f64(secs)
    }
}

#[cfg(test)]
mod tests {
    use super::Host;

    #[test]
    fn a_queued_gather_comes_back_as_the_reply_when_the_window_closes() {
        let data = std::env::temp_dir().join(format!(
            "villmarks-gather-reply-{}-{}.sqlite",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        let config =
            std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../config/evening.toml");
        let mut host = Host::open(&data, &config, 1, 1000, true, 20.0).unwrap();
        let near = host.join("Near").unwrap();
        let seen = host.join("Seen").unwrap();
        let hint = host.submit(&near.token, "sinew", "sinew", None);
        assert!(hint.immediate, "{}", hint.message);
        assert_eq!(hint.message, "Pull sinew from a carcass.");
        let needle = host.submit(&near.token, "needle", "needle", None);
        assert!(needle.immediate, "{}", needle.message);
        assert_eq!(needle.message, "Carve a bone needle.");
        let basket = host.submit(&near.token, "basket", "basket", None);
        assert!(basket.immediate, "{}", basket.message);
        assert_eq!(
            basket.message,
            "Weave a bark basket from a sheet of bark. It holds two more things."
        );
        for (id, line) in [("m1", "n"), ("m2", "n"), ("m3", "n")] {
            let queued = host.submit(&near.token, id, line, None);
            assert!(
                queued.message.starts_with("Queued for the window"),
                "{}",
                queued.message
            );
            host.advance(1).unwrap();
        }
        for (id, line) in [("s1", "w"), ("s2", "n"), ("s3", "n")] {
            host.submit(&seen.token, id, line, None);
            host.advance(1).unwrap();
        }
        let queued = host.submit(&near.token, "flint", "gather flint", None);
        assert!(
            queued.message.starts_with("Queued for the window"),
            "{}",
            queued.message
        );
        assert!(!queued.immediate);
        let closed = host.advance(1).unwrap();
        assert_eq!(closed.replies.len(), 1);
        assert_eq!(closed.replies[0].message, "You gather a flint nodule.");
        assert_eq!(closed.replies[0].actor_id, near.actor_id);
        let again = host.submit(&near.token, "flint", "gather flint", None);
        assert!(again.duplicate);
        assert_eq!(again.message, "You gather a flint nodule.");
        let yours = host.observe(&near.token).unwrap();
        assert!(yours
            .events
            .iter()
            .all(|event| event.text != "You gather a flint nodule."));
        let witness = host.observe(&seen.token).unwrap();
        assert!(witness
            .events
            .iter()
            .any(|event| { event.text == "Near gathers a flint nodule." }));
        let _ = std::fs::remove_file(&data);
        let _ = std::fs::remove_file(data.with_extension("sqlite-wal"));
        let _ = std::fs::remove_file(data.with_extension("sqlite-shm"));
    }

    #[test]
    fn a_queued_gather_the_arms_cannot_hold_comes_back_when_the_window_closes() {
        let data = std::env::temp_dir().join(format!(
            "villmarks-arms-full-reply-{}-{}.sqlite",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        let config =
            std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../config/evening.toml");
        let mut host = Host::open(&data, &config, 1, 1000, true, 20.0).unwrap();
        let near = host.join("Near").unwrap();
        let seen = host.join("Seen").unwrap();
        let quiet = host.join("Quiet").unwrap();
        for (id, line) in [("e1", "e"), ("e2", "e"), ("n1", "n"), ("n2", "n")] {
            let queued = host.submit(&near.token, id, line, None);
            assert!(
                queued.message.starts_with("Queued for the window"),
                "{}",
                queued.message
            );
            host.advance(1).unwrap();
        }
        let full = "Your arms are full. Drop something first.";
        let mut filled = false;
        for (n, line) in [
            "gather branch",
            "gather branch",
            "gather branch",
            "gather branch",
            "gather bark",
            "gather berries",
            "gather pot",
            "gather bone",
            "gather moss",
            "gather quartz",
        ]
        .into_iter()
        .enumerate()
        {
            let id = format!("fill-{n}");
            let queued = host.submit(&near.token, &id, line, None);
            assert!(!queued.immediate, "{}", queued.message);
            let closed = host.advance(1).unwrap();
            if closed.replies.iter().any(|reply| reply.message == full) {
                filled = true;
                assert!(closed
                    .replies
                    .iter()
                    .all(|reply| reply.actor_id == near.actor_id));
                break;
            }
        }
        assert!(filled, "a filler gather never found the arms full");
        let queued = host.submit(&near.token, "ochre", "gather ochre", None);
        assert!(!queued.immediate, "{}", queued.message);
        assert!(queued.message.starts_with("Queued for the window"));
        let closed = host.advance(1).unwrap();
        assert_eq!(closed.replies.len(), 1, "{:?}", closed.replies);
        assert_eq!(closed.replies[0].message, full);
        assert_eq!(closed.replies[0].actor_id, near.actor_id);
        assert_eq!(closed.replies[0].command_id, "ochre");
        let again = host.submit(&near.token, "ochre", "gather ochre", None);
        assert!(again.duplicate);
        assert_eq!(again.message, full);
        for token in [&near.token, &seen.token, &quiet.token] {
            let look = host.observe(token).unwrap();
            assert!(
                look.events.iter().all(|event| event.text != full),
                "{:?}",
                look.events
            );
        }
        assert!(host.replies_for(&seen.token).is_empty());
        assert_eq!(host.replies_for(&near.token).len(), 1);
        assert!(host.clock().sim_time < 17);
        assert!(!host.clock().raining);
        let _ = std::fs::remove_file(&data);
        let _ = std::fs::remove_file(data.with_extension("sqlite-wal"));
        let _ = std::fs::remove_file(data.with_extension("sqlite-shm"));
    }
}
