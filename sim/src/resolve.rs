//! Attempts in one window read the same starting state. Packet order is
//! discarded: ties break by actor id, then command id, and equal claims
//! break by a recorded hash.

use crate::model::{
    cell_id, cell_xy, contest_hash, Attempt, CommandResult, Dir, Draw, Intent, Material, Region,
    Role, Scope, Wake, WorkKind, World,
};
use std::collections::{BTreeMap, BTreeSet};

impl World {
    pub(crate) fn resolve(&mut self, intents: &[Intent]) -> Vec<CommandResult> {
        let mut results = Vec::new();
        let mut pending = Vec::new();
        for intent in intents {
            if let Some(previous) = self.applied.get(&intent.command_id) {
                results.push(previous.clone());
            } else {
                pending.push(intent.clone());
            }
        }
        pending.sort_by(|a, b| {
            a.actor_id
                .cmp(&b.actor_id)
                .then(a.command_id.cmp(&b.command_id))
        });
        let mut seen = BTreeSet::new();
        pending.retain(|intent| seen.insert(intent.command_id.clone()));

        let mut body_taken = BTreeSet::new();
        let mut failures = Vec::new();
        let mut interrupts = Vec::new();
        let mut releases = Vec::new();
        let mut quiet = Vec::new();
        let mut gathers = Vec::new();
        let mut drops = Vec::new();
        let mut grabs = Vec::new();
        let mut moves = Vec::new();
        let mut says = Vec::new();
        let mut drags = Vec::new();
        let mut raises = Vec::new();
        let mut barks = Vec::new();

        for intent in pending {
            let Some(actor) = self.actor(intent.actor_id).cloned() else {
                failures.push((
                    intent,
                    "No such person is in the valley.".to_string(),
                    false,
                ));
                continue;
            };
            let taken = body_taken.contains(&actor.id);
            match self.precheck(&actor, &intent.attempt, taken) {
                Err(message) => failures.push((intent, message, true)),
                Ok(uses_body) => {
                    if uses_body {
                        body_taken.insert(actor.id);
                    }
                    match intent.attempt.clone() {
                        Attempt::Interrupt => interrupts.push(intent),
                        Attempt::Release => releases.push(intent),
                        Attempt::Look | Attempt::Examine(_) => quiet.push(intent),
                        Attempt::Gather(_) => gathers.push(intent),
                        Attempt::Drop(_) => drops.push(intent),
                        Attempt::Grab => grabs.push(intent),
                        Attempt::Move(_) => moves.push(intent),
                        Attempt::Say(_) => says.push(intent),
                        Attempt::Drag(_) => drags.push(intent),
                        Attempt::RaiseFrame => raises.push(intent),
                        Attempt::LayBark => barks.push(intent),
                    }
                }
            }
        }

        let start_cells: BTreeMap<_, _> = self.actors.iter().map(|(id, a)| (*id, a.cell)).collect();
        let start_grips: BTreeMap<_, _> = self
            .grips
            .iter()
            .map(|grip| (grip.actor, (grip.item, grip.since_event)))
            .collect();

        for (intent, message, personal) in failures {
            if personal {
                let cell = start_cells.get(&intent.actor_id).copied();
                self.emit(
                    message.clone(),
                    Scope::Personal(intent.actor_id),
                    vec![intent.actor_id],
                    cell.into_iter().collect(),
                    vec![],
                    vec![],
                    None,
                );
            }
            self.finish(
                intent.command_id,
                intent.actor_id,
                false,
                message,
                &mut results,
            );
        }
        for intent in interrupts {
            let ok = self.interrupt_actor(intent.actor_id).is_ok();
            self.finish(
                intent.command_id,
                intent.actor_id,
                ok,
                "You stop.",
                &mut results,
            );
        }
        for intent in releases {
            let ok = self.release_actor(intent.actor_id).is_ok();
            self.finish(
                intent.command_id,
                intent.actor_id,
                ok,
                "You let go of the log.",
                &mut results,
            );
        }
        for intent in quiet {
            self.finish(
                intent.command_id,
                intent.actor_id,
                true,
                "You take your bearings.",
                &mut results,
            );
        }

        let released: BTreeSet<_> = results
            .iter()
            .filter(|result| result.success && result.message == "You let go of the log.")
            .map(|result| result.actor_id)
            .collect();

        self.apply_gathers(gathers, &start_cells, &mut results);
        self.apply_drops(drops, &mut results);
        self.apply_grabs(grabs, &start_cells, &mut results);
        self.apply_moves(moves, &start_cells, &mut results);
        self.apply_says(says, &start_cells, &mut results);
        self.apply_drags(drags, &start_grips, &released, &mut results);
        self.apply_raises(raises, &mut results);
        self.apply_barks(barks, &mut results);
        results
    }

    fn precheck(
        &self,
        actor: &crate::model::Actor,
        attempt: &Attempt,
        body_taken: bool,
    ) -> Result<bool, String> {
        let busy = actor.busy.is_some();
        let gripping = self.gripping(actor.id).is_some();
        match attempt {
            Attempt::Look | Attempt::Examine(_) => Ok(false),
            Attempt::Say(text) => {
                let text = text.trim();
                if text.is_empty() {
                    Err("Say something.".into())
                } else if text.chars().count() > 200 {
                    Err("That is too long to call across the valley.".into())
                } else {
                    Ok(false)
                }
            }
            Attempt::Interrupt => {
                if busy {
                    Ok(false)
                } else {
                    Err("You are not in the middle of anything.".into())
                }
            }
            Attempt::Release => {
                if gripping {
                    Ok(false)
                } else {
                    Err("You are not holding the log.".into())
                }
            }
            _ if busy => Err("You are in the middle of something.".into()),
            _ if body_taken => Err("You already committed to something else this window.".into()),
            Attempt::Move(dir) => {
                if gripping {
                    return Err("Your hands are on the log.".into());
                }
                let (x, y) = cell_xy(actor.cell);
                let (dx, dy) = dir.delta();
                if cell_id(x + dx, y + dy).is_none() {
                    Err("The valley does not go that way.".into())
                } else {
                    Ok(true)
                }
            }
            Attempt::Gather(material) => {
                if gripping {
                    return Err("Your hands are on the log.".into());
                }
                if *material == Material::Log {
                    return Err("The log is too heavy to gather. Grab it instead.".into());
                }
                if self.pile_count(actor.cell, *material) == 0 {
                    return Err(format!("There is no {} here to gather.", material.word()));
                }
                if self.carry_count(actor.id) >= self.tuning.carry_slots {
                    return Err("Your arms are full.".into());
                }
                Ok(true)
            }
            Attempt::Drop(material) => {
                if self.loose_ids(actor.id, *material).is_empty() {
                    Err(format!("You are not carrying a loose {}.", material.word()))
                } else {
                    Ok(true)
                }
            }
            Attempt::Grab => {
                if gripping {
                    Err("You already have a grip on the log.".into())
                } else if !self.log_at(actor.cell) {
                    Err("There is no log here to grab.".into())
                } else {
                    Ok(true)
                }
            }
            Attempt::Drag(_) => {
                let Some(item_id) = self.gripping(actor.id) else {
                    return Err("You are not holding the log.".into());
                };
                let Some(item) = self.item(item_id) else {
                    return Err("You are not holding the log.".into());
                };
                if item.cell != actor.cell {
                    return Err("You lost contact with the log.".into());
                }
                let (x, y) = cell_xy(actor.cell);
                let (dx, dy) = attempt_dir(attempt).unwrap().delta();
                let Some(dest) = cell_id(x + dx, y + dy) else {
                    return Err("There is nowhere to drag the log.".into());
                };
                if self.cell(dest).region == Region::Brook {
                    return Err("The brook would take the log.".into());
                }
                Ok(true)
            }
            Attempt::RaiseFrame => {
                if gripping {
                    return Err("Your hands are on the log.".into());
                }
                if self.cell(actor.cell).region == Region::Brook {
                    return Err("You cannot set a frame in the brook.".into());
                }
                if self.cell(actor.cell).rock {
                    return Err("The rock overhang is already a roof.".into());
                }
                if self.construction_at(actor.cell).is_some() {
                    return Err("A frame is already started here.".into());
                }
                if self.loose_ids(actor.id, Material::Branch).len() < 3 {
                    return Err("You need three branches in your arms to raise a frame.".into());
                }
                Ok(true)
            }
            Attempt::LayBark => {
                if gripping {
                    return Err("Your hands are on the log.".into());
                }
                let Some(frame) = self.construction_at(actor.cell) else {
                    return Err("There is no frame here.".into());
                };
                if !self.supported_at_both_ends(frame.id) {
                    return Err("The frame is not supported at both ends.".into());
                }
                if frame.bark.is_some() {
                    return Err("Bark already lies on this frame.".into());
                }
                if self.loose_ids(actor.id, Material::Bark).is_empty() {
                    return Err("You are not carrying bark.".into());
                }
                Ok(true)
            }
        }
    }

    fn apply_gathers(
        &mut self,
        gathers: Vec<Intent>,
        start_cells: &BTreeMap<crate::model::ActorId, crate::model::CellId>,
        results: &mut Vec<CommandResult>,
    ) {
        let mut groups: BTreeMap<(crate::model::CellId, Material), Vec<Intent>> = BTreeMap::new();
        for intent in gathers {
            let Attempt::Gather(material) = intent.attempt else {
                continue;
            };
            let cell = start_cells[&intent.actor_id];
            groups.entry((cell, material)).or_default().push(intent);
        }
        for ((cell, material), mut claims) in groups {
            let available = self.pile_count(cell, material);
            let subject = format!("{cell}:{material:?}");
            claims.sort_by(|a, b| {
                let left = contest_hash(self.seed, self.time, "gather", &subject, a.actor_id);
                let right = contest_hash(self.seed, self.time, "gather", &subject, b.actor_id);
                left.cmp(&right).then(a.actor_id.cmp(&b.actor_id))
            });
            let winners: BTreeSet<_> = claims
                .iter()
                .take(available as usize)
                .map(|intent| intent.actor_id)
                .collect();
            claims.sort_by_key(|intent| intent.actor_id);
            let noun = match material {
                Material::Branch => "branch",
                Material::Bark => "sheet of bark",
                Material::Log => "log",
            };
            for intent in claims {
                let hash = contest_hash(self.seed, self.time, "gather", &subject, intent.actor_id);
                let draw = Some(Draw {
                    tag: "gather".into(),
                    subject: subject.clone(),
                    value: hash,
                });
                let name = self.name_of(intent.actor_id);
                if winners.contains(&intent.actor_id) {
                    let moisture = self.take_from_pile(cell, material, 1).unwrap_or(0);
                    let item = self.mint_item(
                        material,
                        moisture,
                        cell,
                        Some(intent.actor_id),
                        Role::Loose,
                    );
                    let text = format!("{name} gathers a {noun}.");
                    self.emit(
                        text,
                        Scope::Cell(cell),
                        vec![intent.actor_id],
                        vec![cell],
                        vec![item],
                        vec![],
                        draw,
                    );
                    self.finish(
                        intent.command_id,
                        intent.actor_id,
                        true,
                        format!("You gather a {noun}."),
                        results,
                    );
                } else {
                    let text =
                        format!("{name} reaches for a {noun}, but another hand is already there.");
                    self.emit(
                        text,
                        Scope::Cell(cell),
                        vec![intent.actor_id],
                        vec![cell],
                        vec![],
                        vec![],
                        draw,
                    );
                    self.finish(
                        intent.command_id,
                        intent.actor_id,
                        false,
                        format!("Your hand meets someone else's on the same {noun}."),
                        results,
                    );
                }
            }
        }
    }

    fn apply_drops(&mut self, drops: Vec<Intent>, results: &mut Vec<CommandResult>) {
        for intent in drops {
            let Attempt::Drop(material) = intent.attempt else {
                continue;
            };
            let Some(item_id) = self.loose_ids(intent.actor_id, material).first().copied() else {
                self.finish(
                    intent.command_id,
                    intent.actor_id,
                    false,
                    format!("You are not carrying a loose {}.", material.word()),
                    results,
                );
                continue;
            };
            let cell = self.actor(intent.actor_id).map(|a| a.cell).unwrap_or(0);
            let name = self.name_of(intent.actor_id);
            if let Some(item) = self.items.get_mut(&item_id) {
                item.carrier = None;
                item.cell = cell;
            }
            let noun = match material {
                Material::Branch => "branch",
                Material::Bark => "sheet of bark",
                Material::Log => "log",
            };
            self.emit(
                format!("{name} drops a {noun}. It is in contact with the ground."),
                Scope::Cell(cell),
                vec![intent.actor_id],
                vec![cell],
                vec![item_id],
                vec![],
                None,
            );
            self.finish(
                intent.command_id,
                intent.actor_id,
                true,
                format!("You drop a {noun}. It is in contact with the ground."),
                results,
            );
        }
    }

    fn apply_grabs(
        &mut self,
        grabs: Vec<Intent>,
        start_cells: &BTreeMap<crate::model::ActorId, crate::model::CellId>,
        results: &mut Vec<CommandResult>,
    ) {
        if grabs.is_empty() {
            return;
        }
        let cell = start_cells[&grabs[0].actor_id];
        let existed = self.log_item().map(|item| item.id);
        let item_id = if let Some(id) = existed {
            id
        } else if let Some(moisture) = self.take_from_pile(cell, Material::Log, 1) {
            self.mint_item(Material::Log, moisture, cell, None, Role::Loose)
        } else {
            for intent in grabs {
                self.finish(
                    intent.command_id,
                    intent.actor_id,
                    false,
                    "There is no log here to grab.",
                    results,
                );
            }
            return;
        };
        let mut grip_events = Vec::new();
        let mut hands = Vec::new();
        for intent in grabs {
            let name = self.name_of(intent.actor_id);
            let event = self.emit(
                format!("{name} grips the log."),
                Scope::Cell(cell),
                vec![intent.actor_id],
                vec![cell],
                vec![item_id],
                vec![],
                None,
            );
            self.grips.push(crate::model::Grip {
                actor: intent.actor_id,
                item: item_id,
                since_event: event,
            });
            grip_events.push(event);
            hands.push(intent.actor_id);
            self.finish(
                intent.command_id,
                intent.actor_id,
                true,
                "You grip the log.",
                results,
            );
        }
        if existed.is_none() {
            self.emit(
                "The log is one piece under their hands, in contact with the ground.",
                Scope::Cell(cell),
                hands,
                vec![cell],
                vec![item_id],
                grip_events,
                None,
            );
        }
    }

    fn apply_moves(
        &mut self,
        moves: Vec<Intent>,
        start_cells: &BTreeMap<crate::model::ActorId, crate::model::CellId>,
        results: &mut Vec<CommandResult>,
    ) {
        for intent in moves {
            let Attempt::Move(dir) = intent.attempt else {
                continue;
            };
            let origin = start_cells[&intent.actor_id];
            let (x, y) = cell_xy(origin);
            let (dx, dy) = dir.delta();
            let Some(dest) = cell_id(x + dx, y + dy) else {
                continue;
            };
            let name = self.name_of(intent.actor_id);
            self.move_actor(intent.actor_id, dest);
            let brook = self.cell(dest).region == Region::Brook;
            if brook {
                if let Some(actor) = self.actors.get_mut(&intent.actor_id) {
                    actor.wetness = (actor.wetness + self.tuning.wade).min(1000);
                }
            }
            let text = if brook {
                format!("{name} wades {word}.", word = dir.word())
            } else {
                format!("{name} walks {word}.", word = dir.word())
            };
            self.emit(
                text,
                Scope::Cell(dest),
                vec![intent.actor_id],
                vec![origin, dest],
                vec![],
                vec![],
                None,
            );
            let message = if brook {
                format!("You wade {}.", dir.word())
            } else {
                format!("You walk {}.", dir.word())
            };
            self.finish(intent.command_id, intent.actor_id, true, message, results);
        }
    }

    fn apply_says(
        &mut self,
        says: Vec<Intent>,
        start_cells: &BTreeMap<crate::model::ActorId, crate::model::CellId>,
        results: &mut Vec<CommandResult>,
    ) {
        for intent in says {
            let Attempt::Say(text) = intent.attempt else {
                continue;
            };
            let text = text.trim().to_string();
            let cell = start_cells[&intent.actor_id];
            let name = self.name_of(intent.actor_id);
            self.emit(
                format!("{name} says, \"{text}\""),
                Scope::Cell(cell),
                vec![intent.actor_id],
                vec![cell],
                vec![],
                vec![],
                None,
            );
            self.finish(
                intent.command_id,
                intent.actor_id,
                true,
                format!("You say, \"{text}\""),
                results,
            );
        }
    }

    fn apply_drags(
        &mut self,
        drags: Vec<Intent>,
        start_grips: &BTreeMap<
            crate::model::ActorId,
            (crate::model::ItemId, crate::model::EventId),
        >,
        released: &BTreeSet<crate::model::ActorId>,
        results: &mut Vec<CommandResult>,
    ) {
        let mut by_dir: BTreeMap<Dir, Vec<Intent>> = BTreeMap::new();
        for intent in drags {
            let Attempt::Drag(dir) = intent.attempt else {
                continue;
            };
            if released.contains(&intent.actor_id) {
                self.finish(
                    intent.command_id,
                    intent.actor_id,
                    false,
                    "You let go instead of pulling.",
                    results,
                );
                continue;
            }
            by_dir.entry(dir).or_default().push(intent);
        }
        let groups: Vec<(Dir, Vec<Intent>)> = by_dir.into_iter().collect();
        if groups.len() > 1 {
            for (_dir, intents) in groups {
                for intent in intents {
                    let name = self.name_of(intent.actor_id);
                    let cell = self.actor(intent.actor_id).map(|a| a.cell).unwrap_or(0);
                    self.emit(
                        format!("{name} pulls against the others. The log stays."),
                        Scope::Cell(cell),
                        vec![intent.actor_id],
                        vec![cell],
                        vec![],
                        vec![],
                        None,
                    );
                    self.finish(
                        intent.command_id,
                        intent.actor_id,
                        false,
                        "The pulls do not add up. The log stays.",
                        results,
                    );
                }
            }
            return;
        }
        let Some((dir, people)) = groups.into_iter().next() else {
            return;
        };
        if (people.len() as u32) < self.tuning.drag_needed {
            for intent in people {
                let name = self.name_of(intent.actor_id);
                let cell = self.actor(intent.actor_id).map(|a| a.cell).unwrap_or(0);
                self.emit(
                    format!("{name} pulls on the log alone. It does not move."),
                    Scope::Cell(cell),
                    vec![intent.actor_id],
                    vec![cell],
                    vec![],
                    vec![],
                    None,
                );
                self.finish(
                    intent.command_id,
                    intent.actor_id,
                    false,
                    "The log does not move for you alone.",
                    results,
                );
            }
            return;
        }
        let Some(item_id) = self.log_item().map(|item| item.id) else {
            return;
        };
        let mut actors = Vec::new();
        let mut causes = Vec::new();
        for intent in &people {
            if let Some((grip_item, since)) = start_grips.get(&intent.actor_id) {
                if *grip_item == item_id {
                    causes.push(*since);
                }
            }
            actors.push(intent.actor_id);
        }
        let cell = self.item(item_id).map(|item| item.cell).unwrap_or(0);
        for intent in &people {
            let name = self.name_of(intent.actor_id);
            let event = self.emit(
                format!("{name} sets their weight to drag the log {}.", dir.word()),
                Scope::Cell(cell),
                vec![intent.actor_id],
                vec![cell],
                vec![item_id],
                vec![],
                None,
            );
            causes.push(event);
        }
        let when = self.time + self.tuning.drag_ticks;
        self.begin_work(
            WorkKind::Drag,
            actors,
            when,
            causes,
            Some(item_id),
            Some(dir),
            1,
            None,
        );
        for intent in people {
            self.finish(
                intent.command_id,
                intent.actor_id,
                true,
                format!("You set your weight to drag the log {}.", dir.word()),
                results,
            );
        }
    }

    fn apply_raises(&mut self, raises: Vec<Intent>, results: &mut Vec<CommandResult>) {
        for intent in raises {
            let cell = self.actor(intent.actor_id).map(|a| a.cell).unwrap_or(0);
            if self.construction_at(cell).is_some() {
                self.finish(
                    intent.command_id,
                    intent.actor_id,
                    false,
                    "A frame is already started here.",
                    results,
                );
                continue;
            }
            let name = self.name_of(intent.actor_id);
            let event = self.emit(
                format!("{name} starts raising a frame."),
                Scope::Cell(cell),
                vec![intent.actor_id],
                vec![cell],
                vec![],
                vec![],
                None,
            );
            let when = self.time + self.tuning.frame_phase_ticks;
            self.begin_work(
                WorkKind::RaiseFrame,
                vec![intent.actor_id],
                when,
                vec![event],
                None,
                None,
                self.tuning.frame_phases,
                None,
            );
            self.finish(
                intent.command_id,
                intent.actor_id,
                true,
                "You start raising a frame.",
                results,
            );
        }
    }

    fn apply_barks(&mut self, barks: Vec<Intent>, results: &mut Vec<CommandResult>) {
        for intent in barks {
            let cell = self.actor(intent.actor_id).map(|a| a.cell).unwrap_or(0);
            let Some(frame_id) = self.construction_at(cell).map(|frame| frame.id) else {
                self.finish(
                    intent.command_id,
                    intent.actor_id,
                    false,
                    "There is no frame here.",
                    results,
                );
                continue;
            };
            if !self.supported_at_both_ends(frame_id) {
                self.finish(
                    intent.command_id,
                    intent.actor_id,
                    false,
                    "The frame is not supported at both ends.",
                    results,
                );
                continue;
            }
            let name = self.name_of(intent.actor_id);
            let event = self.emit(
                format!("{name} starts laying bark over the frame."),
                Scope::Cell(cell),
                vec![intent.actor_id],
                vec![cell],
                vec![],
                vec![],
                None,
            );
            let when = self.time + self.tuning.bark_ticks;
            self.begin_work(
                WorkKind::LayBark,
                vec![intent.actor_id],
                when,
                vec![event],
                None,
                None,
                1,
                Some(frame_id),
            );
            self.finish(
                intent.command_id,
                intent.actor_id,
                true,
                "You start laying bark over the frame.",
                results,
            );
        }
    }

    fn begin_work(
        &mut self,
        kind: WorkKind,
        actors: Vec<crate::model::ActorId>,
        completes_at: u64,
        cause_events: Vec<crate::model::EventId>,
        item: Option<crate::model::ItemId>,
        dir: Option<Dir>,
        phases: u8,
        construction: Option<crate::model::ConstructionId>,
    ) {
        let id = self.next_work;
        self.next_work += 1;
        for actor in &actors {
            if let Some(person) = self.actors.get_mut(actor) {
                person.busy = Some(id);
            }
        }
        self.schedule(completes_at, Wake::Work(id));
        self.works.insert(
            id,
            crate::model::Work {
                id,
                kind,
                actors,
                completes_at,
                cause_events,
                item,
                dir,
                phase_done: 0,
                phases,
                construction,
            },
        );
    }
}

fn attempt_dir(attempt: &Attempt) -> Option<Dir> {
    match attempt {
        Attempt::Drag(dir) | Attempt::Move(dir) => Some(*dir),
        _ => None,
    }
}
