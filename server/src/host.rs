//! The one authoritative process. Intents are queued until the window
//! closes; the core then reads one starting state and commits once.

use crate::persist::{Record, Store};
use serde::Serialize;
use std::collections::BTreeMap;
use std::path::Path;
use std::time::Instant;
use villmarks_sim::{
    help_text, parse_attempt, Attempt, CommandResult, Intent, Material, Observation, Tuning, World,
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
            Attempt::NameOnion => self.settle(
                command_id,
                false,
                false,
                true,
                "Gather a wild onion, eat it raw, or cook it over a fire.".into(),
            ),
            Attempt::NameBasket => {
                let text = self.world.basket_name_line(actor_id);
                self.settle(command_id, false, false, true, text)
            }
            Attempt::NameMat => {
                let text = self.world.mat_name_line(actor_id);
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
                Attempt::Gather(_) if result.success => true,
                Attempt::Light => true,
                Attempt::Gather(_)
                    if !result.success
                        && result.message == "Your arms are full. Drop something first." =>
                {
                    true
                }
                Attempt::Gather(_)
                    if !result.success
                        && (result.message == "There is no basket here to gather."
                            || result.message == "You are already carrying a bark basket.") =>
                {
                    true
                }
                Attempt::PutInBasket(_) | Attempt::TakeFromBasket(_) => true,
                Attempt::Drop(_) => true,
                Attempt::Gather(_)
                    if !result.success
                        && result.message.contains("here to gather.")
                        && !result.message.contains("Step ") =>
                {
                    true
                }
                Attempt::Gather(_)
                    if !result.success && result.message.contains(" Step ") =>
                {
                    true
                }
                Attempt::Eat(_)
                    | Attempt::Roast
                    | Attempt::RoastCattail
                    | Attempt::RoastOnion
                    | Attempt::RoastMushroom
                    | Attempt::RoastMeat
                    | Attempt::RoastFish
                    | Attempt::Cook
                    | Attempt::CookCattail
                    | Attempt::CookOnion
                    | Attempt::CookMushroom
                    | Attempt::CookMeat
                    | Attempt::CookFish
                    | Attempt::Drink
                    | Attempt::BreakIce => true,
                Attempt::WeaveBasket | Attempt::WeaveMat | Attempt::SetMat | Attempt::Rest => true,
                Attempt::Gather(_)
                    if !result.success
                        && (result.message
                            == "Worms are in the dry ground here. Carve an antler into an antler pick to dig them."
                            || result.message == "The dry ground here has no worms."
                            || result.message == "Ice covers the cattail roots.") =>
                {
                    true
                }
                Attempt::Gather(Material::Onion) if !result.success => true,
                Attempt::Move(_) if result.success => true,
                Attempt::Move(_)
                    if !result.success
                        && result.message == "The valley does not go that way." =>
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

    #[test]
    fn a_missed_basket_gather_comes_back_when_the_window_closes() {
        let data = std::env::temp_dir().join(format!(
            "villmarks-basket-miss-{}-{}.sqlite",
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
        let none = "There is no basket here to gather.";
        let queued = host.submit(&near.token, "none", "gather basket", None);
        assert!(!queued.immediate, "{}", queued.message);
        let closed = host.advance(1).unwrap();
        assert_eq!(closed.replies.len(), 1, "{:?}", closed.replies);
        assert_eq!(closed.replies[0].message, none);
        assert_eq!(closed.replies[0].actor_id, near.actor_id);
        let wove = host.submit(&near.token, "weave", "weave basket", None);
        assert!(!wove.immediate, "{}", wove.message);
        host.advance(1).unwrap();
        let carrying = "You are already carrying a bark basket.";
        let queued = host.submit(&near.token, "again", "gather basket", None);
        assert!(!queued.immediate, "{}", queued.message);
        let closed = host.advance(1).unwrap();
        assert_eq!(closed.replies.len(), 1, "{:?}", closed.replies);
        assert_eq!(closed.replies[0].message, carrying);
        assert_eq!(closed.replies[0].actor_id, near.actor_id);
        assert_eq!(closed.replies[0].command_id, "again");
        let again = host.submit(&near.token, "again", "gather basket", None);
        assert!(again.duplicate);
        assert_eq!(again.message, carrying);
        for token in [&near.token, &seen.token, &quiet.token] {
            let look = host.observe(token).unwrap();
            assert!(
                look.events
                    .iter()
                    .all(|event| event.text != none && event.text != carrying),
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

    #[test]
    fn a_put_into_the_basket_comes_back_when_the_window_closes() {
        let data = std::env::temp_dir().join(format!(
            "villmarks-put-basket-{}-{}.sqlite",
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
        let none = "You are not carrying a bark basket.";
        let queued = host.submit(&near.token, "none", "put berries in basket", None);
        assert!(!queued.immediate, "{}", queued.message);
        let closed = host.advance(1).unwrap();
        assert_eq!(closed.replies.len(), 1, "{:?}", closed.replies);
        assert_eq!(closed.replies[0].message, none);
        assert_eq!(closed.replies[0].actor_id, near.actor_id);
        let wove = host.submit(&near.token, "weave", "weave basket", None);
        assert!(!wove.immediate, "{}", wove.message);
        host.advance(1).unwrap();
        let missing = "There is no reed flute here to put in the basket.";
        let queued = host.submit(&near.token, "flute", "put flute in basket", None);
        assert!(!queued.immediate, "{}", queued.message);
        let closed = host.advance(1).unwrap();
        assert_eq!(closed.replies[0].message, missing);
        let put = "You put a handful of berries in the bark basket.";
        let heard = "Near puts a handful of berries in the bark basket.";
        let queued = host.submit(&near.token, "berries", "put berries in basket", None);
        assert!(!queued.immediate, "{}", queued.message);
        let closed = host.advance(1).unwrap();
        assert_eq!(closed.replies.len(), 1, "{:?}", closed.replies);
        assert_eq!(closed.replies[0].message, put);
        assert_eq!(closed.replies[0].actor_id, near.actor_id);
        let full = "You put a pot in the bark basket. The basket is full.";
        let queued = host.submit(&near.token, "pot", "put pot in basket", None);
        let closed = host.advance(1).unwrap();
        assert_eq!(closed.replies[0].message, full, "{:?}", queued.message);
        let queued = host.submit(&near.token, "bone", "put bone in basket", None);
        assert!(!queued.immediate, "{}", queued.message);
        let closed = host.advance(1).unwrap();
        let refused = "The bark basket is full. A bone will not fit.";
        assert_eq!(closed.replies[0].message, refused);
        let duplicate = host.submit(&near.token, "bone", "put bone in basket", None);
        assert!(duplicate.duplicate);
        assert_eq!(duplicate.message, refused);
        let near_look = host.observe(&near.token).unwrap();
        assert!(near_look.events.iter().all(|event| {
            event.text != none
                && event.text != missing
                && event.text != put
                && event.text != heard
                && event.text != full
                && event.text != "The bark basket is full. A bone will not fit."
                && event.text != "Near puts a pot in the bark basket. The basket is full."
        }));
        let seen_look = host.observe(&seen.token).unwrap();
        assert!(seen_look.events.iter().all(|event| event.text != heard));
        let quiet_look = host.observe(&quiet.token).unwrap();
        assert!(quiet_look.events.iter().any(|event| event.text == heard));
        assert!(host.replies_for(&seen.token).is_empty());
        assert_eq!(host.replies_for(&near.token).len(), 1);
        assert!(host.clock().sim_time < 17);
        assert!(!host.clock().raining);
        let _ = std::fs::remove_file(&data);
        let _ = std::fs::remove_file(data.with_extension("sqlite-wal"));
        let _ = std::fs::remove_file(data.with_extension("sqlite-shm"));
    }

    #[test]
    fn eat_roast_drink_and_a_dry_dig_come_back_when_the_window_closes() {
        let data = std::env::temp_dir().join(format!(
            "villmarks-eat-reply-{}-{}.sqlite",
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
        let carve =
            "Worms are in the dry ground here. Carve an antler into an antler pick to dig them.";
        let queued = host.submit(&near.token, "dig", "dig worms", None);
        assert!(!queued.immediate, "{}", queued.message);
        let closed = host.advance(1).unwrap();
        assert_eq!(closed.replies.len(), 1, "{:?}", closed.replies);
        assert_eq!(closed.replies[0].message, carve);
        assert_eq!(closed.replies[0].actor_id, near.actor_id);
        for token in [&near.token, &seen.token, &quiet.token] {
            let look = host.observe(token).unwrap();
            assert!(
                look.events.iter().all(|event| event.text != carve),
                "{:?}",
                look.events
            );
        }
        for (id, line) in [("w1", "w"), ("w2", "w"), ("n1", "n")] {
            let queued = host.submit(&near.token, id, line, None);
            assert!(
                queued.message.starts_with("Queued for the window"),
                "{}",
                queued.message
            );
            host.advance(1).unwrap();
        }
        let queued = host.submit(&near.token, "sip", "drink", None);
        assert!(!queued.immediate, "{}", queued.message);
        let closed = host.advance(1).unwrap();
        assert_eq!(closed.replies.len(), 1, "{:?}", closed.replies);
        assert_eq!(
            closed.replies[0].message,
            "You drink from the brook. You are not thirsty."
        );
        assert!(!closed.replies[0].message.contains("eases"));
        assert_eq!(closed.replies[0].actor_id, near.actor_id);
        let drank = "Near drinks from the brook.";
        let near_look = host.observe(&near.token).unwrap();
        assert!(near_look.events.iter().all(|event| event.text != drank));
        assert!(host
            .observe(&seen.token)
            .unwrap()
            .events
            .iter()
            .any(|event| event.text == drank));
        assert!(host
            .observe(&quiet.token)
            .unwrap()
            .events
            .iter()
            .all(|event| event.text != drank));
        for (id, line) in [
            ("e1", "e"),
            ("n2", "n"),
            ("shroom", "gather mushroom"),
            ("e2", "e"),
            ("e3", "e"),
            ("e4", "e"),
        ] {
            let queued = host.submit(&near.token, id, line, None);
            assert!(
                queued.message.starts_with("Queued for the window"),
                "{} {}",
                line,
                queued.message
            );
            host.advance(1).unwrap();
        }
        let cold = "There is no fire burning here.";
        let queued = host.submit(&near.token, "cold", "roast", None);
        assert!(!queued.immediate, "{}", queued.message);
        let closed = host.advance(1).unwrap();
        assert_eq!(closed.replies[0].message, cold);
        for token in [&near.token, &seen.token, &quiet.token] {
            assert!(host
                .observe(token)
                .unwrap()
                .events
                .iter()
                .all(|event| event.text != cold));
        }
        let queued = host.submit(&near.token, "light", "light fire", None);
        assert!(!queued.immediate, "{}", queued.message);
        host.advance(1).unwrap();
        let roast = "You roast the mushroom over the fire.";
        let roast_heard = "Near roasts the mushroom over the fire.";
        let queued = host.submit(&near.token, "roast", "roast", None);
        assert!(!queued.immediate, "{}", queued.message);
        let closed = host.advance(1).unwrap();
        assert_eq!(closed.replies[0].message, roast);
        let eat_heard = "Near eats the cooked mushroom.";
        let queued = host.submit(&near.token, "eat", "eat mushroom", None);
        assert!(!queued.immediate, "{}", queued.message);
        let closed = host.advance(1).unwrap();
        assert!(
            closed.replies[0]
                .message
                .starts_with("You eat the cooked mushroom."),
            "{}",
            closed.replies[0].message
        );
        let away = "The brook is not here.";
        let queued = host.submit(&near.token, "dry", "drink", None);
        assert!(!queued.immediate, "{}", queued.message);
        let closed = host.advance(1).unwrap();
        assert_eq!(closed.replies[0].message, away);
        let again = host.submit(&near.token, "dry", "drink", None);
        assert!(again.duplicate);
        assert_eq!(again.message, away);
        let near_look = host.observe(&near.token).unwrap();
        assert!(near_look.events.iter().all(|event| {
            event.text != roast
                && event.text != roast_heard
                && event.text != "You eat the cooked mushroom."
                && !event.text.starts_with("You eat the cooked mushroom.")
                && event.text != eat_heard
                && event.text != away
        }));
        let quiet_look = host.observe(&quiet.token).unwrap();
        assert!(quiet_look
            .events
            .iter()
            .any(|event| event.text == roast_heard));
        assert!(quiet_look
            .events
            .iter()
            .any(|event| event.text == eat_heard));
        assert!(quiet_look.events.iter().all(|event| event.text != away));
        let seen_look = host.observe(&seen.token).unwrap();
        assert!(seen_look
            .events
            .iter()
            .all(|event| event.text != roast_heard && event.text != eat_heard));
        assert!(host.replies_for(&seen.token).is_empty());
        assert_eq!(host.replies_for(&near.token).len(), 1);
        assert!(host.clock().sim_time < 17);
        assert!(!host.clock().raining);
        let _ = std::fs::remove_file(&data);
        let _ = std::fs::remove_file(data.with_extension("sqlite-wal"));
        let _ = std::fs::remove_file(data.with_extension("sqlite-shm"));
    }

    #[test]
    fn a_bark_basket_drop_comes_back_when_the_window_closes() {
        let data = std::env::temp_dir().join(format!(
            "villmarks-basket-drop-{}-{}.sqlite",
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
        let missing = "You are not carrying a loose basket.";
        let queued = host.submit(&near.token, "none", "drop basket", None);
        assert!(!queued.immediate, "{}", queued.message);
        let closed = host.advance(1).unwrap();
        assert_eq!(closed.replies.len(), 1, "{:?}", closed.replies);
        assert_eq!(closed.replies[0].message, missing);
        assert_eq!(closed.replies[0].actor_id, near.actor_id);
        for token in [&near.token, &seen.token, &quiet.token] {
            assert!(host
                .observe(token)
                .unwrap()
                .events
                .iter()
                .all(|event| event.text != missing));
        }
        for (step, near_line, seen_line) in [
            ("1", "e", "e"),
            ("2", "e", "e"),
            ("3", "n", "e"),
            ("4", "n", "n"),
        ] {
            let queued = host.submit(&near.token, &format!("near-{step}"), near_line, None);
            assert!(
                queued.message.starts_with("Queued for the window"),
                "{}",
                queued.message
            );
            let seen_move = host.submit(&seen.token, &format!("seen-{step}"), seen_line, None);
            assert!(seen_move.message.starts_with("Queued for the window"));
            let quiet_move = host.submit(&quiet.token, &format!("quiet-{step}"), "w", None);
            assert!(quiet_move.message.starts_with("Queued for the window"));
            host.advance(1).unwrap();
        }
        let queued = host.submit(&near.token, "weave", "weave basket", None);
        assert!(!queued.immediate, "{}", queued.message);
        host.advance(1).unwrap();
        let reply = "You drop a bark basket. It is in contact with the ground.";
        let heard = "Near drops a bark basket. It is in contact with the ground.";
        let queued = host.submit(&near.token, "drop", "drop basket", None);
        assert!(!queued.immediate, "{}", queued.message);
        let closed = host.advance(1).unwrap();
        assert_eq!(closed.replies[0].message, reply);
        assert_eq!(closed.replies[0].actor_id, near.actor_id);
        let again = host.submit(&near.token, "drop", "drop basket", None);
        assert!(again.duplicate);
        assert_eq!(again.message, reply);
        let near_look = host.observe(&near.token).unwrap();
        assert!(near_look
            .events
            .iter()
            .all(|event| event.text != reply && event.text != heard));
        assert!(host
            .observe(&seen.token)
            .unwrap()
            .events
            .iter()
            .any(|event| event.text == heard));
        assert!(host
            .observe(&quiet.token)
            .unwrap()
            .events
            .iter()
            .all(|event| event.text != heard && event.text != reply));
        assert!(host.replies_for(&seen.token).is_empty());
        assert_eq!(host.replies_for(&near.token).len(), 1);
        assert!(host.clock().sim_time < 17);
        assert!(!host.clock().raining);
        let _ = std::fs::remove_file(&data);
        let _ = std::fs::remove_file(data.with_extension("sqlite-wal"));
        let _ = std::fs::remove_file(data.with_extension("sqlite-shm"));
    }

    #[test]
    fn a_stone_drop_comes_back_when_the_window_closes() {
        let data = std::env::temp_dir().join(format!(
            "villmarks-stone-drop-{}-{}.sqlite",
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
        let gone = "There is no berry here to gather.";
        let queued = host.submit(&near.token, "berries", "gather berries", None);
        assert!(!queued.immediate, "{}", queued.message);
        let closed = host.advance(1).unwrap();
        assert_eq!(closed.replies.len(), 1, "{:?}", closed.replies);
        assert_eq!(closed.replies[0].message, gone);
        for token in [&near.token, &seen.token, &quiet.token] {
            assert!(host
                .observe(token)
                .unwrap()
                .events
                .iter()
                .all(|event| event.text != gone));
        }
        let missing = "You are not carrying a loose stone.";
        let queued = host.submit(&near.token, "none", "drop stone", None);
        assert!(!queued.immediate, "{}", queued.message);
        let closed = host.advance(1).unwrap();
        assert_eq!(closed.replies[0].message, missing);
        for token in [&near.token, &seen.token, &quiet.token] {
            assert!(host
                .observe(token)
                .unwrap()
                .events
                .iter()
                .all(|event| event.text != missing));
        }
        let steps = [
            ("1", "w", "w", "w"),
            ("2", "w", "n", "w"),
            ("3", "n", "n", "w"),
            ("4", "n", "look", "w"),
            ("5", "n", "look", "look"),
        ];
        for (step, near_line, seen_line, quiet_line) in steps {
            host.submit(&near.token, &format!("near-{step}"), near_line, None);
            if seen_line != "look" {
                host.submit(&seen.token, &format!("seen-{step}"), seen_line, None);
            }
            if quiet_line != "look" {
                host.submit(&quiet.token, &format!("quiet-{step}"), quiet_line, None);
            }
            host.advance(1).unwrap();
        }
        let queued = host.submit(&near.token, "stone", "gather stone", None);
        assert!(!queued.immediate, "{}", queued.message);
        let closed = host.advance(1).unwrap();
        assert_eq!(closed.replies[0].message, "You gather a stone.");
        let reply = "You drop a stone. It is in contact with the ground.";
        let heard = "Near drops a stone. It is in contact with the ground.";
        let queued = host.submit(&near.token, "drop", "drop stone", None);
        assert!(!queued.immediate, "{}", queued.message);
        let closed = host.advance(1).unwrap();
        assert_eq!(closed.replies[0].message, reply);
        assert_eq!(closed.replies[0].actor_id, near.actor_id);
        let near_look = host.observe(&near.token).unwrap();
        assert!(near_look
            .events
            .iter()
            .all(|event| event.text != reply && event.text != heard));
        assert!(host
            .observe(&seen.token)
            .unwrap()
            .events
            .iter()
            .any(|event| event.text == heard));
        assert!(host
            .observe(&quiet.token)
            .unwrap()
            .events
            .iter()
            .all(|event| event.text != heard && event.text != reply));
        assert!(host.clock().sim_time < 17);
        assert!(!host.clock().raining);
        let _ = std::fs::remove_file(&data);
        let _ = std::fs::remove_file(data.with_extension("sqlite-wal"));
        let _ = std::fs::remove_file(data.with_extension("sqlite-shm"));
    }

    #[test]
    fn a_bark_basket_weave_comes_back_when_the_window_closes() {
        let data = std::env::temp_dir().join(format!(
            "villmarks-weave-basket-{}-{}.sqlite",
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
        let missing = "You need a sheet of bark to weave a basket.";
        let queued = host.submit(&near.token, "none", "weave basket", None);
        assert!(!queued.immediate, "{}", queued.message);
        let closed = host.advance(1).unwrap();
        assert_eq!(closed.replies.len(), 1, "{:?}", closed.replies);
        assert_eq!(closed.replies[0].message, missing);
        for token in [&near.token, &seen.token, &quiet.token] {
            assert!(host
                .observe(token)
                .unwrap()
                .events
                .iter()
                .all(|event| event.text != missing));
        }
        let steps = [
            ("1", "e", "e", "w"),
            ("2", "e", "e", "w"),
            ("3", "n", "e", "w"),
            ("4", "n", "n", "w"),
        ];
        for (step, near_line, seen_line, quiet_line) in steps {
            host.submit(&near.token, &format!("near-{step}"), near_line, None);
            host.submit(&seen.token, &format!("seen-{step}"), seen_line, None);
            host.submit(&quiet.token, &format!("quiet-{step}"), quiet_line, None);
            host.advance(1).unwrap();
        }
        let reply = "You weave a bark basket.";
        let heard = "Near weaves a bark basket.";
        let queued = host.submit(&near.token, "weave", "weave basket", None);
        assert!(!queued.immediate, "{}", queued.message);
        let closed = host.advance(1).unwrap();
        assert_eq!(closed.replies.len(), 1, "{:?}", closed.replies);
        assert_eq!(closed.replies[0].message, reply);
        assert_eq!(closed.replies[0].actor_id, near.actor_id);
        let again = host.submit(&near.token, "weave", "weave basket", None);
        assert!(again.duplicate);
        assert_eq!(again.message, reply);
        assert!(host.observe(&near.token).unwrap().events.iter().all(|event| {
            event.text != reply && event.text != heard
        }));
        assert!(host
            .observe(&seen.token)
            .unwrap()
            .events
            .iter()
            .any(|event| event.text == heard));
        assert!(host
            .observe(&quiet.token)
            .unwrap()
            .events
            .iter()
            .all(|event| event.text != heard && event.text != reply));
        let carrying = "You are already carrying a bark basket.";
        let queued = host.submit(&near.token, "second", "weave basket", None);
        assert!(!queued.immediate, "{}", queued.message);
        let closed = host.advance(1).unwrap();
        assert_eq!(closed.replies[0].message, carrying);
        for token in [&near.token, &seen.token, &quiet.token] {
            assert!(host
                .observe(token)
                .unwrap()
                .events
                .iter()
                .all(|event| event.text != carrying));
        }
        assert!(host.clock().sim_time < 17);
        assert!(!host.clock().raining);
        let _ = std::fs::remove_file(&data);
        let _ = std::fs::remove_file(data.with_extension("sqlite-wal"));
        let _ = std::fs::remove_file(data.with_extension("sqlite-shm"));
    }

    #[test]
    fn a_hole_in_the_ice_comes_back_when_the_window_closes() {
        let data = std::env::temp_dir().join(format!(
            "villmarks-ice-reply-{}-{}.sqlite",
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
        let none = "There is no ice here.";
        host.submit(&near.token, "early", "break ice", None);
        let closed = host.advance(1).unwrap();
        assert_eq!(closed.replies.len(), 1, "{:?}", closed.replies);
        assert_eq!(closed.replies[0].message, none);
        assert_eq!(closed.replies[0].actor_id, near.actor_id);
        for token in [&near.token, &seen.token, &quiet.token] {
            let look = host.observe(token).unwrap();
            assert!(look.events.iter().all(|event| event.text != none));
        }
        for (id, step) in [("n1", "n"), ("n2", "n"), ("w1", "w"), ("w2", "w")] {
            host.submit(&near.token, id, step, None);
            host.advance(1).unwrap();
        }
        host.submit(&near.token, "stone-step", "n", None);
        host.advance(1).unwrap();
        host.submit(&near.token, "stone", "gather stone", None);
        let gathered = host.advance(1).unwrap();
        assert_eq!(gathered.replies[0].message, "You gather a stone.");
        host.submit(&near.token, "back", "s", None);
        host.advance(1).unwrap();
        while host.clock().sim_time < 12 {
            host.advance(1).unwrap();
        }
        let covered = "Ice covers the cattail roots.";
        host.submit(&near.token, "roots-ice", "dig roots", None);
        let missed = host.advance(1).unwrap();
        assert_eq!(missed.replies.len(), 1, "{:?}", missed.replies);
        assert_eq!(missed.replies[0].message, covered);
        assert_eq!(missed.replies[0].actor_id, near.actor_id);
        for token in [&near.token, &seen.token, &quiet.token] {
            assert!(host
                .observe(token)
                .unwrap()
                .events
                .iter()
                .all(|event| event.text != covered));
        }
        let reply = "You break a hole in the ice with the stone.";
        let heard = "Near breaks a hole in the ice with the stone.";
        host.submit(&near.token, "open", "break ice", None);
        let closed = host.advance(1).unwrap();
        assert_eq!(closed.replies.len(), 1, "{:?}", closed.replies);
        assert_eq!(closed.replies[0].message, reply);
        assert_eq!(closed.replies[0].actor_id, near.actor_id);
        assert_eq!(closed.replies[0].command_id, "open");
        assert!(host
            .observe(&near.token)
            .unwrap()
            .events
            .iter()
            .all(|event| { event.text != reply && event.text != heard }));
        assert!(host
            .observe(&seen.token)
            .unwrap()
            .events
            .iter()
            .any(|event| event.text == heard));
        let quiet_look = host.observe(&quiet.token).unwrap();
        assert!(quiet_look.events.iter().all(|event| event.text != heard));
        assert!(quiet_look
            .events
            .iter()
            .any(|event| event.text.contains("You hear ice break")));
        let dug = "You gather cattail roots.";
        let named = "Near gathers cattail roots.";
        host.submit(&near.token, "roots", "dig roots", None);
        let gathered = host.advance(1).unwrap();
        assert_eq!(gathered.replies.len(), 1, "{:?}", gathered.replies);
        assert_eq!(gathered.replies[0].message, dug);
        assert_eq!(gathered.replies[0].actor_id, near.actor_id);
        let near_look = host.observe(&near.token).unwrap();
        assert!(near_look.text.contains(
            "Cattails stand in the hole in the ice. The roots can be dug. They are not the reeds."
        ));
        assert!(near_look.map.contains("@=l"), "{}", near_look.map);
        assert!(
            near_look.legend.contains("= frozen still"),
            "{}",
            near_look.legend
        );
        assert!(
            !near_look.legend.contains("= still water"),
            "{}",
            near_look.legend
        );
        assert!(near_look.legend.contains("=l frozen still with a hole"));
        assert!(host
            .observe(&seen.token)
            .unwrap()
            .legend
            .contains("= frozen still"));
        assert!(host.observe(&seen.token).unwrap().map.contains("p=l"));
        assert!(!host
            .observe(&quiet.token)
            .unwrap()
            .legend
            .contains("frozen still with a hole"));
        assert!(near_look
            .events
            .iter()
            .all(|event| event.text != dug && event.text != named));
        assert!(host
            .observe(&seen.token)
            .unwrap()
            .events
            .iter()
            .any(|event| event.text == named));
        assert!(host
            .observe(&quiet.token)
            .unwrap()
            .events
            .iter()
            .all(|event| !event.text.contains("cattail")));
        assert!(host.clock().sim_time < 17);
        assert!(!host.clock().raining);
        assert!(host.replies_for(&seen.token).is_empty());
        let _ = std::fs::remove_file(&data);
        let _ = std::fs::remove_file(data.with_extension("sqlite-wal"));
        let _ = std::fs::remove_file(data.with_extension("sqlite-shm"));
    }

    #[test]
    fn a_reed_mat_comes_back_when_the_window_closes() {
        let data = std::env::temp_dir().join(format!(
            "villmarks-mat-reply-{}-{}.sqlite",
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
        let named = host.submit(&near.token, "mat", "mat", None);
        assert!(named.immediate, "{}", named.message);
        assert_eq!(
            named.message,
            "Weave a reed mat from two bundles of reeds. Lay it under the rock or in a lean-to, then rest."
        );
        let miss = "You need two bundles of reeds to weave a mat.";
        let queued = host.submit(&near.token, "short", "weave mat", None);
        assert!(!queued.immediate, "{}", queued.message);
        let closed = host.advance(1).unwrap();
        assert_eq!(closed.replies.len(), 1, "{:?}", closed.replies);
        assert_eq!(closed.replies[0].message, miss);
        assert_eq!(closed.replies[0].actor_id, near.actor_id);
        for token in [&near.token, &seen.token, &quiet.token] {
            let look = host.observe(token).unwrap();
            assert!(look.events.iter().all(|event| event.text != miss));
        }
        let wind = "The wind will not let you rest.";
        host.submit(&near.token, "wind", "rest", None);
        let closed = host.advance(1).unwrap();
        assert_eq!(closed.replies[0].message, wind);
        for token in [&near.token, &seen.token, &quiet.token] {
            let look = host.observe(token).unwrap();
            assert!(look.events.iter().all(|event| event.text != wind));
        }
        for n in 1..=3 {
            host.submit(&near.token, &format!("near-w{n}"), "w", None);
            host.submit(&seen.token, &format!("seen-w{n}"), "w", None);
            host.advance(1).unwrap();
        }
        host.submit(&near.token, "w4", "w", None);
        host.advance(1).unwrap();
        let woven = "You weave a reed mat.";
        let heard = "Near weaves a reed mat.";
        let queued = host.submit(&near.token, "weave", "weave mat", None);
        assert!(!queued.immediate, "{}", queued.message);
        let closed = host.advance(1).unwrap();
        assert_eq!(closed.replies[0].message, woven);
        assert_eq!(closed.replies[0].actor_id, near.actor_id);
        assert!(host
            .observe(&near.token)
            .unwrap()
            .events
            .iter()
            .all(|event| event.text != woven && event.text != heard));
        assert!(host
            .observe(&seen.token)
            .unwrap()
            .events
            .iter()
            .any(|event| event.text == heard));
        assert!(host
            .observe(&quiet.token)
            .unwrap()
            .events
            .iter()
            .all(|event| event.text != heard && event.text != woven));
        let set = "You set a reed mat.";
        let set_heard = "Near sets a reed mat.";
        host.submit(&near.token, "set", "set mat", None);
        let closed = host.advance(1).unwrap();
        assert_eq!(closed.replies[0].message, set);
        assert!(host
            .observe(&near.token)
            .unwrap()
            .events
            .iter()
            .all(|event| event.text != set && event.text != set_heard));
        assert!(host
            .observe(&seen.token)
            .unwrap()
            .events
            .iter()
            .any(|event| event.text == set_heard));
        let rest = "You rest on the reed mat.";
        let rest_heard = "Near rests on the reed mat.";
        host.submit(&near.token, "rest", "sleep", None);
        let closed = host.advance(1).unwrap();
        assert_eq!(closed.replies[0].message, rest);
        assert!(host
            .observe(&near.token)
            .unwrap()
            .events
            .iter()
            .all(|event| event.text != rest && event.text != rest_heard));
        assert!(host
            .observe(&seen.token)
            .unwrap()
            .events
            .iter()
            .any(|event| event.text == rest_heard));
        assert!(host
            .observe(&quiet.token)
            .unwrap()
            .events
            .iter()
            .all(|event| !event.text.contains("mat") && !event.text.contains("rest")));
        assert!(host.replies_for(&seen.token).is_empty());
        assert!(host.clock().sim_time < 17);
        assert!(!host.clock().raining);
        let _ = std::fs::remove_file(&data);
        let _ = std::fs::remove_file(data.with_extension("sqlite-wal"));
        let _ = std::fs::remove_file(data.with_extension("sqlite-shm"));
    }

    #[test]
    fn a_walk_comes_back_when_the_window_closes() {
        let data = std::env::temp_dir().join(format!(
            "villmarks-walk-reply-{}-{}.sqlite",
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
        let queued = host.submit(&near.token, "south", "south", None);
        assert!(
            queued.message.starts_with("Queued for the window"),
            "{}",
            queued.message
        );
        let closed = host.advance(1).unwrap();
        assert_eq!(closed.replies.len(), 1, "{:?}", closed.replies);
        assert_eq!(closed.replies[0].message, "You walk south.");
        assert_eq!(closed.replies[0].actor_id, near.actor_id);
        assert_eq!(closed.replies[0].command_id, "south");
        let yours = host.observe(&near.token).unwrap();
        assert_eq!((yours.self_info.x, yours.self_info.y), (4, 4));
        assert!(yours.events.iter().all(|event| {
            event.text != "You walk south." && event.text != "Near walks south."
        }));
        assert!(host
            .observe(&seen.token)
            .unwrap()
            .events
            .iter()
            .any(|event| event.text == "Near walks south."));
        assert!(host
            .observe(&quiet.token)
            .unwrap()
            .events
            .iter()
            .any(|event| event.text == "Near walks south."));
        for (n, id) in (0..5).zip(["w1", "w2", "w3", "w4", "w5"]) {
            let queued = host.submit(&quiet.token, id, "west", None);
            assert!(
                queued.message.starts_with("Queued for the window"),
                "{}",
                queued.message
            );
            let closed = host.advance(1).unwrap();
            let expected = if n == 4 {
                "You wade west."
            } else {
                "You walk west."
            };
            assert_eq!(closed.replies[0].message, expected, "{:?}", closed.replies);
            assert!(host.observe(&quiet.token).unwrap().events.iter().all(
                |event| {
                    event.text != expected
                        && event.text != "Quiet walks west."
                        && event.text != "Quiet wades west."
                }
            ));
        }
        let quiet_at = host.observe(&quiet.token).unwrap();
        assert_eq!((quiet_at.self_info.x, quiet_at.self_info.y), (0, 3));
        host.submit(&near.token, "west", "west", None);
        let closed = host.advance(1).unwrap();
        assert_eq!(closed.replies[0].message, "You walk west.");
        assert!(host.observe(&near.token).unwrap().events.iter().all(
            |event| event.text != "You walk west." && event.text != "Near walks west."
        ));
        assert!(host
            .observe(&seen.token)
            .unwrap()
            .events
            .iter()
            .any(|event| event.text == "Near walks west."));
        let heard = host.observe(&quiet.token).unwrap();
        assert!(heard
            .events
            .iter()
            .any(|event| event.text == "You hear footsteps to the south and east."));
        assert!(heard
            .events
            .iter()
            .all(|event| event.text != "Near walks west."));
        assert!(!host.clock().raining);
        let _ = std::fs::remove_file(&data);
        let _ = std::fs::remove_file(data.with_extension("sqlite-wal"));
        let _ = std::fs::remove_file(data.with_extension("sqlite-shm"));
    }

    #[test]
    fn an_edge_walk_comes_back_when_the_window_closes() {
        let data = std::env::temp_dir().join(format!(
            "villmarks-edge-walk-{}-{}.sqlite",
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
        let edge = "The valley does not go that way.";
        for id in ["e1", "e2"] {
            host.submit(&near.token, id, "east", None);
            let closed = host.advance(1).unwrap();
            assert_eq!(closed.replies.len(), 1, "{:?}", closed.replies);
            assert!(closed.replies[0].message.starts_with("You walk east"));
        }
        let at_edge = host.observe(&near.token).unwrap();
        assert_eq!((at_edge.self_info.x, at_edge.self_info.y), (6, 3));
        host.submit(&near.token, "off", "east", None);
        let closed = host.advance(1).unwrap();
        assert_eq!(closed.replies.len(), 1, "{:?}", closed.replies);
        assert_eq!(closed.replies[0].message, edge);
        assert_eq!(closed.replies[0].actor_id, near.actor_id);
        assert_eq!(closed.replies[0].command_id, "off");
        let stayed = host.observe(&near.token).unwrap();
        assert_eq!((stayed.self_info.x, stayed.self_info.y), (6, 3));
        assert!(stayed.events.iter().all(|event| event.text != edge));
        assert!(host
            .observe(&seen.token)
            .unwrap()
            .events
            .iter()
            .all(|event| event.text != edge));
        assert!(host
            .observe(&quiet.token)
            .unwrap()
            .events
            .iter()
            .all(|event| event.text != edge));
        assert!(!host.clock().raining);
        let _ = std::fs::remove_file(&data);
        let _ = std::fs::remove_file(data.with_extension("sqlite-wal"));
        let _ = std::fs::remove_file(data.with_extension("sqlite-shm"));
    }

    #[test]
    fn a_bark_gather_one_cell_off_comes_back_when_the_window_closes() {
        let data = std::env::temp_dir().join(format!(
            "villmarks-bark-step-{}-{}.sqlite",
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
        for (id, line) in [("n1", "north"), ("n2", "north"), ("e1", "east")] {
            host.submit(&near.token, id, line, None);
            let closed = host.advance(1).unwrap();
            assert_eq!(closed.replies.len(), 1, "{:?}", closed.replies);
            assert!(
                closed.replies[0].message.starts_with("You walk"),
                "{:?}",
                closed.replies
            );
        }
        let beside = host.observe(&near.token).unwrap();
        assert_eq!((beside.self_info.x, beside.self_info.y), (5, 1));
        let step = "There is no bark here to gather. Step east.";
        host.submit(&near.token, "bark", "gather bark", None);
        let closed = host.advance(1).unwrap();
        assert_eq!(closed.replies.len(), 1, "{:?}", closed.replies);
        assert_eq!(closed.replies[0].message, step);
        assert_eq!(closed.replies[0].actor_id, near.actor_id);
        assert_eq!(closed.replies[0].command_id, "bark");
        let stayed = host.observe(&near.token).unwrap();
        assert_eq!((stayed.self_info.x, stayed.self_info.y), (5, 1));
        for token in [&near.token, &seen.token, &quiet.token] {
            assert!(host
                .observe(token)
                .unwrap()
                .events
                .iter()
                .all(|event| event.text != step));
        }
        assert!(!host.clock().raining);
        let _ = std::fs::remove_file(&data);
        let _ = std::fs::remove_file(data.with_extension("sqlite-wal"));
        let _ = std::fs::remove_file(data.with_extension("sqlite-shm"));
    }

    #[test]
    fn a_pale_berry_gather_one_cell_off_comes_back_when_the_window_closes() {
        let data = std::env::temp_dir().join(format!(
            "villmarks-pale-step-{}-{}.sqlite",
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
        for (id, line) in [
            ("s", "south"),
            ("w1", "west"),
            ("w2", "west"),
            ("w3", "west"),
        ] {
            host.submit(&near.token, id, line, None);
            let closed = host.advance(1).unwrap();
            assert_eq!(closed.replies.len(), 1, "{:?}", closed.replies);
            assert!(
                closed.replies[0].message.starts_with("You walk"),
                "{:?}",
                closed.replies
            );
        }
        let beside = host.observe(&near.token).unwrap();
        assert_eq!((beside.self_info.x, beside.self_info.y), (1, 4));
        let willow = "There is no willow here to gather. Step west.";
        host.submit(&near.token, "willow", "gather willow", None);
        let closed = host.advance(1).unwrap();
        assert_eq!(closed.replies.len(), 1, "{:?}", closed.replies);
        assert_eq!(closed.replies[0].message, willow);
        assert_eq!(closed.replies[0].actor_id, near.actor_id);
        host.submit(&near.token, "w4", "west", None);
        let _ = host.advance(1).unwrap();
        let pale = "There is no pale berry here to gather. Step east.";
        host.submit(&near.token, "pale", "gather pale berries", None);
        let closed = host.advance(1).unwrap();
        assert_eq!(closed.replies.len(), 1, "{:?}", closed.replies);
        assert_eq!(closed.replies[0].message, pale);
        assert_eq!(closed.replies[0].command_id, "pale");
        let stayed = host.observe(&near.token).unwrap();
        assert_eq!((stayed.self_info.x, stayed.self_info.y), (0, 4));
        for token in [&near.token, &seen.token, &quiet.token] {
            let events = &host.observe(token).unwrap().events;
            assert!(events.iter().all(|event| event.text != pale));
            assert!(events.iter().all(|event| event.text != willow));
        }
        assert!(!host.clock().raining);
        let _ = std::fs::remove_file(&data);
        let _ = std::fs::remove_file(data.with_extension("sqlite-wal"));
        let _ = std::fs::remove_file(data.with_extension("sqlite-shm"));
    }

    #[test]
    fn a_light_that_cannot_catch_comes_back_when_the_window_closes() {
        let data = std::env::temp_dir().join(format!(
            "villmarks-light-reply-{}-{}.sqlite",
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
        let miss = "You need dry loose bark for tinder.";
        host.submit(&near.token, "bare", "light fire", None);
        let closed = host.advance(1).unwrap();
        assert_eq!(closed.replies.len(), 1, "{:?}", closed.replies);
        assert_eq!(closed.replies[0].message, miss);
        assert_eq!(closed.replies[0].actor_id, near.actor_id);
        for token in [&near.token, &seen.token, &quiet.token] {
            assert!(host
                .observe(token)
                .unwrap()
                .events
                .iter()
                .all(|event| event.text != miss));
        }
        let _ = std::fs::remove_file(&data);
        let _ = std::fs::remove_file(data.with_extension("sqlite-wal"));
        let _ = std::fs::remove_file(data.with_extension("sqlite-shm"));
    }
}
