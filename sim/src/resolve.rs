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
        let mut drinks = Vec::new();
        let mut boils = Vec::new();
        let mut lights = Vec::new();
        let mut cooks = Vec::new();
        let mut eats = Vec::new();
        let mut smears = Vec::new();
        let mut sets = Vec::new();
        let mut binds = Vec::new();
        let mut hunts = Vec::new();

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
                        Attempt::Drink => drinks.push(intent),
                        Attempt::Boil => boils.push(intent),
                        Attempt::Light => lights.push(intent),
                        Attempt::Cook => cooks.push(intent),
                        Attempt::Eat(_) => eats.push(intent),
                        Attempt::Smear => smears.push(intent),
                        Attempt::SetClay => sets.push(intent),
                        Attempt::BindSpear => binds.push(intent),
                        Attempt::Hunt => hunts.push(intent),
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
        self.apply_drinks(drinks, &mut results);
        self.apply_boils(boils, &mut results);
        self.apply_sets(sets, &mut results);
        self.apply_binds(binds, &mut results);
        self.apply_hunts(hunts, &mut results);
        self.apply_lights(lights, &start_cells, &mut results);
        self.apply_cooks(cooks, &mut results);
        self.apply_eats(eats, &mut results);
        self.apply_smears(smears, &mut results);
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
            Attempt::Drink => {
                if busy {
                    Err("You are in the middle of something.".into())
                } else if body_taken {
                    Err("You already committed to something else this window.".into())
                } else if !self.water_here(actor.cell) {
                    Err("The brook is not here.".into())
                } else if self.still_frozen(actor.cell) {
                    Err("The still water is frozen.".into())
                } else {
                    Ok(true)
                }
            }
            Attempt::Boil => {
                if busy {
                    Err("You are in the middle of something.".into())
                } else if body_taken {
                    Err("You already committed to something else this window.".into())
                } else if gripping {
                    Err("Your hands are on the log.".into())
                } else if self.fire_at(actor.cell).is_none() {
                    Err("There is no fire burning here.".into())
                } else if self.loose_ids(actor.id, Material::Water).is_empty() {
                    Err("You have no water to boil.".into())
                } else if self.loose_ids(actor.id, Material::Pot).is_empty()
                    && self.loose_ids(actor.id, Material::Bowl).is_empty()
                {
                    if self.loose_ids(actor.id, Material::Clay).is_empty() {
                        Err("You have no pot to boil it in.".into())
                    } else {
                        Err("Unfired clay cannot boil water.".into())
                    }
                } else {
                    Ok(true)
                }
            }
            Attempt::BindSpear => {
                if busy {
                    Err("You are in the middle of something.".into())
                } else if body_taken {
                    Err("You already committed to something else this window.".into())
                } else if gripping {
                    Err("Your hands are on the log.".into())
                } else if self.loose_ids(actor.id, Material::Stone).is_empty() {
                    Err("You are not carrying a stone.".into())
                } else if self.loose_ids(actor.id, Material::Branch).is_empty() {
                    Err("You are not carrying a branch.".into())
                } else {
                    Ok(true)
                }
            }
            Attempt::Hunt => {
                if busy {
                    Err("You are in the middle of something.".into())
                } else if body_taken {
                    Err("You already committed to something else this window.".into())
                } else if gripping {
                    Err("Your hands are on the log.".into())
                } else if !self
                    .loose_ids(actor.id, Material::Spear)
                    .iter()
                    .any(|id| self.item(*id).is_some_and(|spear| spear.hardened))
                {
                    if self.loose_ids(actor.id, Material::Spear).is_empty() {
                        Err("You are not carrying a spear.".into())
                    } else {
                        Err("The spear's point is not hardened.".into())
                    }
                } else if self.deer.as_ref().is_none() {
                    Err("There is no deer to hunt.".into())
                } else if self
                    .deer
                    .as_ref()
                    .is_some_and(|deer| crate::model::chebyshev(actor.cell, deer.cell) > 1)
                {
                    Err("The deer is not close enough to hunt.".into())
                } else {
                    Ok(true)
                }
            }
            Attempt::SetClay => {
                if busy {
                    Err("You are in the middle of something.".into())
                } else if body_taken {
                    Err("You already committed to something else this window.".into())
                } else if gripping {
                    Err("Your hands are on the log.".into())
                } else if self.loose_ids(actor.id, Material::Clay).is_empty() {
                    Err("You are not carrying clay.".into())
                } else if self.fire_at(actor.cell).is_none() {
                    Err("There is no fire burning here.".into())
                } else {
                    Ok(true)
                }
            }
            Attempt::Eat(material) => {
                if busy {
                    Err("You are in the middle of something.".into())
                } else if body_taken {
                    Err("You already committed to something else this window.".into())
                } else if gripping {
                    Err("Your hands are on the log.".into())
                } else {
                    match material {
                        Material::Mushroom => Err("Raw mushrooms do not feed you.".into()),
                        Material::BitterBerry => {
                            if self.has_material_at_hand(actor.id, Material::BitterBerry) {
                                Ok(true)
                            } else {
                                Err("There are no poison berries here to eat.".into())
                            }
                        }
                        Material::Berry => {
                            if self.has_material_at_hand(actor.id, Material::Berry) {
                                Ok(true)
                            } else {
                                Err("There are no sweet berries here to eat.".into())
                            }
                        }
                        Material::Stone => Err("A stone is not food.".into()),
                        Material::Spear => Err("A spear is not food.".into()),
                        Material::Carcass => {
                            if self.loose_ids(actor.id, Material::Carcass).is_empty()
                                && self.ground_ids(actor.cell, Material::Carcass).is_empty()
                            {
                                Err("You are not carrying a carcass.".into())
                            } else {
                                Err("The carcass must be cooked.".into())
                            }
                        }
                        Material::Clay => Err("Unfired clay is not food.".into()),
                        Material::Bowl => Err("A bowl is not food.".into()),
                        _ => Err("Bark is tinder, not food. Light a fire and cook berries.".into()),
                    }
                }
            }
            Attempt::Smear => {
                if busy {
                    Err("You are in the middle of something.".into())
                } else if body_taken {
                    Err("You already committed to something else this window.".into())
                } else if gripping {
                    Err("Your hands are on the log.".into())
                } else if self.loose_ids(actor.id, Material::Mud).is_empty() {
                    Err("You have no mud to smear.".into())
                } else {
                    Ok(true)
                }
            }
            Attempt::Light => {
                if busy {
                    Err("You are in the middle of something.".into())
                } else if body_taken {
                    Err("You already committed to something else this window.".into())
                } else if let Some(message) = self.light_refusal(actor.id) {
                    Err(message)
                } else {
                    Ok(true)
                }
            }
            Attempt::Cook => {
                if busy {
                    Err("You are in the middle of something.".into())
                } else if body_taken {
                    Err("You already committed to something else this window.".into())
                } else if let Some(message) = self.cook_refusal(actor.id) {
                    Err(message)
                } else {
                    Ok(true)
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
                if *material == Material::Water {
                    if !self.water_here(actor.cell) {
                        return Err("The brook is not here.".into());
                    }
                    if self.still_frozen(actor.cell) {
                        return Err("The still water is frozen.".into());
                    }
                } else if *material == Material::Mud {
                    if !self.water_here(actor.cell) {
                        return Err("There is no mud here to gather.".into());
                    }
                } else if self.pile_count(actor.cell, *material) == 0
                    && (!matches!(
                        *material,
                        Material::Stone
                            | Material::Clay
                            | Material::Bowl
                            | Material::Spear
                            | Material::Carcass
                    ) || self.ground_ids(actor.cell, *material).is_empty())
                {
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
        let mut water = Vec::new();
        let mut mud = Vec::new();
        for intent in gathers {
            let Attempt::Gather(material) = intent.attempt else {
                continue;
            };
            if material == Material::Water {
                water.push(intent);
                continue;
            }
            if material == Material::Mud {
                mud.push(intent);
                continue;
            }
            let cell = start_cells[&intent.actor_id];
            groups.entry((cell, material)).or_default().push(intent);
        }
        for intent in water {
            self.gather_brook_water(intent, start_cells, results);
        }
        for intent in mud {
            self.gather_mud(intent, start_cells, results);
        }
        for ((cell, material), mut claims) in groups {
            let ground = if matches!(
                material,
                Material::Stone
                    | Material::Clay
                    | Material::Bowl
                    | Material::Spear
                    | Material::Carcass
            ) {
                self.ground_ids(cell, material)
            } else {
                Vec::new()
            };
            let available = self.pile_count(cell, material) + ground.len() as u32;
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
            let noun = material.gather_noun();
            for intent in claims {
                let hash = contest_hash(self.seed, self.time, "gather", &subject, intent.actor_id);
                let draw = Some(Draw {
                    tag: "gather".into(),
                    subject: subject.clone(),
                    value: hash,
                });
                let name = self.name_of(intent.actor_id);
                if winners.contains(&intent.actor_id) {
                    let item = if self.pile_count(cell, material) > 0 {
                        let moisture = self.take_from_pile(cell, material, 1).unwrap_or(0);
                        self.mint_item(material, moisture, cell, Some(intent.actor_id), Role::Loose)
                    } else if let Some(id) = self.ground_ids(cell, material).first().copied() {
                        if let Some(lying) = self.items.get_mut(&id) {
                            lying.carrier = Some(intent.actor_id);
                            lying.firing_ready_at = None;
                        }
                        id
                    } else {
                        self.finish(
                            intent.command_id,
                            intent.actor_id,
                            false,
                            format!("There is no {} here to gather.", material.word()),
                            results,
                        );
                        continue;
                    };
                    let breaking = if material == Material::Branch {
                        " You hear branches breaking."
                    } else {
                        ""
                    };
                    let text = format!("{name} gathers a {noun}.{breaking}");
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
            let noun = material.gather_noun();
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
            let entered_overhang = self.cell(dest).rock && !self.cell(origin).rock;
            let text = if brook {
                format!("{name} wades {word}.", word = dir.word())
            } else if entered_overhang {
                format!(
                    "{name} walks {word}, under the rock overhang, out of the wind.",
                    word = dir.word()
                )
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
            } else if entered_overhang {
                format!(
                    "You walk {}, under the rock overhang, out of the wind.",
                    dir.word()
                )
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

    pub(crate) fn light_refusal(&self, actor: crate::model::ActorId) -> Option<String> {
        let person = self.actor(actor)?;
        if self.raining && self.exposure_millis(person.cell) >= 1000 {
            return Some("The rain puts the bark out before it catches.".into());
        }
        if self.fire_at(person.cell).is_some() {
            return Some("A fire already burns here.".into());
        }
        if !self.has_dry(actor, Material::Bark) {
            return Some(if self.has_material_at_hand(actor, Material::Bark) {
                "The bark is too damp to catch.".into()
            } else {
                "You need dry loose bark for tinder.".into()
            });
        }
        if !self.has_dry(actor, Material::Branch) {
            return Some(if self.has_material_at_hand(actor, Material::Branch) {
                "The branch is too damp to burn.".into()
            } else {
                "You need a dry branch for the fire.".into()
            });
        }
        None
    }

    pub(crate) fn cook_refusal(&self, actor: crate::model::ActorId) -> Option<String> {
        let person = self.actor(actor)?;
        if self.fire_at(person.cell).is_none() {
            return Some("There is no fire burning here.".into());
        }
        if !self.cookable_carried(actor) && self.cookable_pile(person.cell) == 0 {
            return Some("There is nothing here to cook.".into());
        }
        None
    }

    fn apply_lights(
        &mut self,
        lights: Vec<Intent>,
        start_cells: &BTreeMap<crate::model::ActorId, crate::model::CellId>,
        results: &mut Vec<CommandResult>,
    ) {
        let mut groups: BTreeMap<crate::model::CellId, Vec<Intent>> = BTreeMap::new();
        for intent in lights {
            let cell = start_cells.get(&intent.actor_id).copied().unwrap_or(0);
            groups.entry(cell).or_default().push(intent);
        }
        for (cell, mut claims) in groups {
            let subject = format!("{cell}:fire");
            claims.sort_by(|a, b| {
                let left = contest_hash(self.seed, self.time, "light", &subject, a.actor_id);
                let right = contest_hash(self.seed, self.time, "light", &subject, b.actor_id);
                left.cmp(&right).then(a.actor_id.cmp(&b.actor_id))
            });
            let mut lit = self.fire_at(cell).is_some();
            for intent in claims {
                let hash = contest_hash(self.seed, self.time, "light", &subject, intent.actor_id);
                let draw = Some(Draw {
                    tag: "light".into(),
                    subject: subject.clone(),
                    value: hash,
                });
                if lit || self.light_refusal(intent.actor_id).is_some() {
                    let message = self
                        .light_refusal(intent.actor_id)
                        .unwrap_or_else(|| "A fire already burns here.".into());
                    self.finish(intent.command_id, intent.actor_id, false, message, results);
                    continue;
                }
                let bark = self.consume_dry(intent.actor_id, Material::Bark);
                let branch = self.consume_dry(intent.actor_id, Material::Branch);
                let (Ok((bark_item, bark_arms)), Ok((branch_item, branch_arms))) = (bark, branch)
                else {
                    self.finish(
                        intent.command_id,
                        intent.actor_id,
                        false,
                        "You need dry loose bark and a dry branch.",
                        results,
                    );
                    continue;
                };
                let name = self.name_of(intent.actor_id);
                let event = self.emit(
                    light_line(Some(&name), bark_arms, branch_arms),
                    Scope::Cell(cell),
                    vec![intent.actor_id],
                    vec![cell],
                    bark_item.into_iter().chain(branch_item).collect(),
                    vec![],
                    draw,
                );
                self.mint_fire(cell, event);
                lit = true;
                self.finish(
                    intent.command_id,
                    intent.actor_id,
                    true,
                    light_line(None, bark_arms, branch_arms),
                    results,
                );
            }
        }
    }

    fn apply_cooks(&mut self, cooks: Vec<Intent>, results: &mut Vec<CommandResult>) {
        let mut carried = Vec::new();
        let mut piles: BTreeMap<crate::model::CellId, Vec<Intent>> = BTreeMap::new();
        for intent in cooks {
            if self.cookable_carried(intent.actor_id) {
                carried.push(intent);
            } else {
                let cell = self
                    .actor(intent.actor_id)
                    .map(|actor| actor.cell)
                    .unwrap_or(0);
                piles.entry(cell).or_default().push(intent);
            }
        }
        for intent in carried {
            self.cook_for(intent, None, "", results);
        }
        for (cell, mut claims) in piles {
            let available = self.cookable_pile(cell);
            let subject = format!("{cell}:food");
            claims.sort_by(|a, b| {
                let left = contest_hash(self.seed, self.time, "cook", &subject, a.actor_id);
                let right = contest_hash(self.seed, self.time, "cook", &subject, b.actor_id);
                left.cmp(&right).then(a.actor_id.cmp(&b.actor_id))
            });
            let winners: BTreeSet<_> = claims
                .iter()
                .take(available as usize)
                .map(|intent| intent.actor_id)
                .collect();
            for intent in claims {
                let hash = contest_hash(self.seed, self.time, "cook", &subject, intent.actor_id);
                if winners.contains(&intent.actor_id) {
                    self.cook_for(intent, Some(hash), &subject, results);
                } else {
                    let name = self.name_of(intent.actor_id);
                    self.emit(
                        format!(
                            "{name} reaches for the berries, but another hand is already there."
                        ),
                        Scope::Cell(cell),
                        vec![intent.actor_id],
                        vec![cell],
                        vec![],
                        vec![],
                        Some(Draw {
                            tag: "cook".into(),
                            subject: subject.clone(),
                            value: hash,
                        }),
                    );
                    self.finish(
                        intent.command_id,
                        intent.actor_id,
                        false,
                        "There are no berries left to cook.",
                        results,
                    );
                }
            }
        }
    }

    fn cook_for(
        &mut self,
        intent: Intent,
        draw: Option<u64>,
        subject: &str,
        results: &mut Vec<CommandResult>,
    ) {
        if self.cook_refusal(intent.actor_id).is_some() {
            let message = self
                .cook_refusal(intent.actor_id)
                .unwrap_or_else(|| "There is no fire burning here.".into());
            self.finish(intent.command_id, intent.actor_id, false, message, results);
            return;
        }
        let cell = self
            .actor(intent.actor_id)
            .map(|actor| actor.cell)
            .unwrap_or(0);
        let Ok((item, material)) = self.consume_food(intent.actor_id) else {
            self.finish(
                intent.command_id,
                intent.actor_id,
                false,
                "There is nothing here to cook.",
                results,
            );
            return;
        };
        let phrase = material.cooked_phrase();
        let before = self
            .actor(intent.actor_id)
            .map(|actor| actor.hunger)
            .unwrap_or(0);
        let after = before.saturating_sub(self.tuning.cook_relief);
        let energy_before = self
            .actor(intent.actor_id)
            .map(|actor| actor.energy)
            .unwrap_or(0);
        let energy_after = energy_before.saturating_sub(self.tuning.eat_relief);
        if let Some(person) = self.actors.get_mut(&intent.actor_id) {
            person.hunger = after;
            person.energy = energy_after;
        }
        let name = self.name_of(intent.actor_id);
        let fire_event = self
            .fire_at(cell)
            .map(|fire| fire.lit_event)
            .into_iter()
            .collect();
        self.emit(
            format!("{name} cooks {phrase} over the fire."),
            Scope::Cell(cell),
            vec![intent.actor_id],
            vec![cell],
            item.into_iter().collect(),
            fire_event,
            draw.map(|value| Draw {
                tag: "cook".into(),
                subject: subject.to_string(),
                value,
            }),
        );
        let hunger_now = self.hunger_word(after);
        let energy_was = self.energy_word(energy_before);
        let energy_now = self.energy_word(energy_after);
        let mut message = format!("You cook {phrase}. You are {hunger_now}.");
        if energy_was != energy_now {
            message.push_str(&format!(" You are {energy_now}."));
        }
        self.finish(intent.command_id, intent.actor_id, true, message, results);
    }

    fn gather_brook_water(
        &mut self,
        intent: Intent,
        start_cells: &BTreeMap<crate::model::ActorId, crate::model::CellId>,
        results: &mut Vec<CommandResult>,
    ) {
        let cell = start_cells.get(&intent.actor_id).copied().unwrap_or(0);
        if !self.water_here(cell) {
            self.finish(
                intent.command_id,
                intent.actor_id,
                false,
                "The brook is not here.",
                results,
            );
            return;
        }
        if self.still_frozen(cell) {
            self.finish(
                intent.command_id,
                intent.actor_id,
                false,
                "The still water is frozen.",
                results,
            );
            return;
        }
        if self.carry_count(intent.actor_id) >= self.tuning.carry_slots {
            self.finish(
                intent.command_id,
                intent.actor_id,
                false,
                "Your arms are full.",
                results,
            );
            return;
        }
        let item = self.mint_item(Material::Water, 0, cell, Some(intent.actor_id), Role::Loose);
        let name = self.name_of(intent.actor_id);
        let source = if self.cell(cell).region == Region::Brook {
            "the brook"
        } else {
            "the still water"
        };
        self.emit(
            format!("{name} gathers a drink of water from {source}."),
            Scope::Cell(cell),
            vec![intent.actor_id],
            vec![cell],
            vec![item],
            vec![],
            None,
        );
        self.finish(
            intent.command_id,
            intent.actor_id,
            true,
            "You gather a drink of water.",
            results,
        );
    }

    fn apply_binds(&mut self, binds: Vec<Intent>, results: &mut Vec<CommandResult>) {
        for intent in binds {
            let stone = self
                .loose_ids(intent.actor_id, Material::Stone)
                .first()
                .copied();
            let branch = self
                .loose_ids(intent.actor_id, Material::Branch)
                .first()
                .copied();
            let (Some(stone), Some(branch)) = (stone, branch) else {
                self.finish(
                    intent.command_id,
                    intent.actor_id,
                    false,
                    "You need a carried stone and a carried branch.",
                    results,
                );
                continue;
            };
            let cell = self
                .actor(intent.actor_id)
                .map(|actor| actor.cell)
                .unwrap_or(0);
            self.items.remove(&stone);
            self.items.remove(&branch);
            let id = self.mint_item(Material::Spear, 0, cell, Some(intent.actor_id), Role::Loose);
            let name = self.name_of(intent.actor_id);
            self.emit(
                format!("{name} binds a stone and a branch into a spear."),
                Scope::Cell(cell),
                vec![intent.actor_id],
                vec![cell],
                vec![id],
                vec![],
                None,
            );
            self.finish(
                intent.command_id,
                intent.actor_id,
                true,
                "You bind a stone and a branch into a spear.",
                results,
            );
        }
    }

    fn apply_hunts(&mut self, mut hunts: Vec<Intent>, results: &mut Vec<CommandResult>) {
        hunts.sort_by_key(|intent| intent.actor_id);
        for intent in hunts {
            let Some(actor) = self.actor(intent.actor_id).cloned() else {
                continue;
            };
            if !self
                .loose_ids(actor.id, Material::Spear)
                .iter()
                .any(|id| self.item(*id).is_some_and(|spear| spear.hardened))
            {
                let message = if self.loose_ids(actor.id, Material::Spear).is_empty() {
                    "You are not carrying a spear."
                } else {
                    "The spear's point is not hardened."
                };
                self.finish(intent.command_id, actor.id, false, message, results);
                continue;
            }
            let Some(deer) = self.deer.clone() else {
                self.finish(
                    intent.command_id,
                    actor.id,
                    false,
                    "There is no deer to hunt.",
                    results,
                );
                continue;
            };
            let dist = crate::model::chebyshev(actor.cell, deer.cell);
            if dist > 1 {
                self.finish(
                    intent.command_id,
                    actor.id,
                    false,
                    "The deer is not close enough to hunt.",
                    results,
                );
                continue;
            }
            if dist == 0 {
                self.bring_down(&intent, deer.cell, None, results);
                continue;
            }
            let (x, y) = crate::model::cell_xy(deer.cell);
            let subject = format!("{x},{y}:deer");
            let hash = contest_hash(self.seed, self.time, "hunt", &subject, actor.id);
            let draw = Some(Draw {
                tag: "hunt".into(),
                subject,
                value: hash,
            });
            match hash % 3 {
                0 => self.bring_down(&intent, deer.cell, draw, results),
                1 => {
                    self.flee_deer(actor.cell);
                    let name = self.name_of(actor.id);
                    self.emit(
                        format!("{name} misses the deer. It moves off."),
                        Scope::Cell(deer.cell),
                        vec![actor.id],
                        vec![deer.cell],
                        vec![],
                        vec![],
                        draw,
                    );
                    self.finish(
                        intent.command_id,
                        actor.id,
                        false,
                        "You miss the deer. It moves off.",
                        results,
                    );
                }
                _ => {
                    self.flee_deer(actor.cell);
                    let name = self.name_of(actor.id);
                    self.emit(
                        format!("The deer bolts from {name}'s spear."),
                        Scope::Cell(deer.cell),
                        vec![actor.id],
                        vec![deer.cell],
                        vec![],
                        vec![],
                        draw,
                    );
                    self.finish(
                        intent.command_id,
                        actor.id,
                        false,
                        "The deer bolts. Your spear does not land.",
                        results,
                    );
                }
            }
        }
    }

    /// A hardened spear on the deer's own cell. No draw: the point is already there.
    /// The next cell is the even case, and that success carries the draw.
    fn bring_down(
        &mut self,
        intent: &Intent,
        deer_cell: crate::model::CellId,
        draw: Option<Draw>,
        results: &mut Vec<CommandResult>,
    ) {
        let name = self.name_of(intent.actor_id);
        self.tracks.insert(deer_cell);
        self.deer = None;
        let id = self.mint_item(Material::Carcass, 0, deer_cell, None, Role::Loose);
        self.emit(
            format!("{name} brings the deer down."),
            Scope::Cell(deer_cell),
            vec![intent.actor_id],
            vec![deer_cell],
            vec![id],
            vec![],
            draw,
        );
        self.finish(
            intent.command_id.clone(),
            intent.actor_id,
            true,
            "You bring the deer down. A carcass lies where it stood.",
            results,
        );
    }

    /// One step along the trail, away from the hunter when that choice is clear.
    /// The route step for this tick does not also fire.
    fn flee_deer(&mut self, hunter: crate::model::CellId) {
        let Some(deer) = self.deer.clone() else {
            return;
        };
        let n = crate::model::DEER_ROUTE.len();
        let forward_at = (deer.route_at + 1) % n;
        let back_at = (deer.route_at + n - 1) % n;
        let forward = crate::model::cell_id(
            crate::model::DEER_ROUTE[forward_at].0,
            crate::model::DEER_ROUTE[forward_at].1,
        );
        let back = crate::model::cell_id(
            crate::model::DEER_ROUTE[back_at].0,
            crate::model::DEER_ROUTE[back_at].1,
        );
        let (Some(forward), Some(back)) = (forward, back) else {
            return;
        };
        let mut dest_at = if crate::model::chebyshev(forward, hunter)
            > crate::model::chebyshev(back, hunter)
        {
            forward_at
        } else if crate::model::chebyshev(back, hunter) > crate::model::chebyshev(forward, hunter) {
            back_at
        } else {
            forward_at
        };
        let mut dest = crate::model::cell_id(
            crate::model::DEER_ROUTE[dest_at].0,
            crate::model::DEER_ROUTE[dest_at].1,
        )
        .unwrap_or(deer.cell);
        if dest == hunter {
            dest_at = if dest_at == forward_at {
                back_at
            } else {
                forward_at
            };
            dest = crate::model::cell_id(
                crate::model::DEER_ROUTE[dest_at].0,
                crate::model::DEER_ROUTE[dest_at].1,
            )
            .unwrap_or(deer.cell);
        }
        self.tracks.insert(deer.cell);
        if let Some(deer) = self.deer.as_mut() {
            deer.cell = dest;
            deer.route_at = dest_at;
            deer.held = true;
        }
    }

    fn apply_sets(&mut self, sets: Vec<Intent>, results: &mut Vec<CommandResult>) {
        for intent in sets {
            let Some(id) = self
                .loose_ids(intent.actor_id, Material::Clay)
                .first()
                .copied()
            else {
                self.finish(
                    intent.command_id,
                    intent.actor_id,
                    false,
                    "You are not carrying clay.",
                    results,
                );
                continue;
            };
            let cell = self
                .actor(intent.actor_id)
                .map(|actor| actor.cell)
                .unwrap_or(0);
            if self.fire_at(cell).is_none() {
                self.finish(
                    intent.command_id,
                    intent.actor_id,
                    false,
                    "There is no fire burning here.",
                    results,
                );
                continue;
            }
            let ready = self.time.saturating_add(self.tuning.clay_ticks);
            if let Some(item) = self.items.get_mut(&id) {
                item.carrier = None;
                item.cell = cell;
                item.firing_ready_at = Some(ready);
            }
            self.schedule(ready, Wake::Harden(id));
            let name = self.name_of(intent.actor_id);
            self.emit(
                format!("{name} sets a lump of clay by the fire."),
                Scope::Cell(cell),
                vec![intent.actor_id],
                vec![cell],
                vec![id],
                vec![],
                None,
            );
            self.finish(
                intent.command_id,
                intent.actor_id,
                true,
                "You set a lump of clay by the fire. It is not hard yet.",
                results,
            );
        }
    }

    fn apply_boils(&mut self, boils: Vec<Intent>, results: &mut Vec<CommandResult>) {
        for intent in boils {
            let Some(actor) = self.actor(intent.actor_id).cloned() else {
                continue;
            };
            if self.fire_at(actor.cell).is_none() {
                self.finish(
                    intent.command_id,
                    intent.actor_id,
                    false,
                    "There is no fire burning here.",
                    results,
                );
                continue;
            }
            let Some(item_id) = self.loose_ids(actor.id, Material::Water).first().copied() else {
                self.finish(
                    intent.command_id,
                    intent.actor_id,
                    false,
                    "You have no water to boil.",
                    results,
                );
                continue;
            };
            if self.loose_ids(actor.id, Material::Pot).is_empty()
                && self.loose_ids(actor.id, Material::Bowl).is_empty()
            {
                let message = if self.loose_ids(actor.id, Material::Clay).is_empty() {
                    "You have no pot to boil it in."
                } else {
                    "Unfired clay cannot boil water."
                };
                self.finish(intent.command_id, intent.actor_id, false, message, results);
                continue;
            }
            self.items.remove(&item_id);
            let before = actor.thirst;
            let after = before.saturating_sub(self.tuning.drink_relief);
            if let Some(person) = self.actors.get_mut(&actor.id) {
                person.thirst = after;
            }
            let fire_event = self
                .fire_at(actor.cell)
                .map(|fire| fire.lit_event)
                .into_iter()
                .collect();
            self.emit(
                format!(
                    "{} boils a drink of water over the fire and drinks it.",
                    actor.name
                ),
                Scope::Cell(actor.cell),
                vec![actor.id],
                vec![actor.cell],
                vec![item_id],
                fire_event,
                None,
            );
            let message = if before == 0 {
                "You boil a drink of water and drink it. You were not thirsty."
            } else if after < self.tuning.thirsty_at {
                "You boil a drink of water and drink it. The thirst eases."
            } else {
                "You boil a drink of water and drink it. You are still thirsty."
            };
            self.finish(intent.command_id, intent.actor_id, true, message, results);
        }
    }

    fn apply_drinks(&mut self, drinks: Vec<Intent>, results: &mut Vec<CommandResult>) {
        for intent in drinks {
            let Some(actor) = self.actor(intent.actor_id).cloned() else {
                continue;
            };
            if !self.water_here(actor.cell) {
                self.finish(
                    intent.command_id,
                    intent.actor_id,
                    false,
                    "The brook is not here.",
                    results,
                );
                continue;
            }
            if self.still_frozen(actor.cell) {
                self.finish(
                    intent.command_id,
                    intent.actor_id,
                    false,
                    "The still water is frozen.",
                    results,
                );
                continue;
            }
            let still =
                self.cell(actor.cell).still && self.cell(actor.cell).region != Region::Brook;
            let chance = if still {
                self.tuning.still_chance
            } else {
                self.tuning.illness_chance
            };
            let subject = format!("{}:{}", actor.cell, if still { "still" } else { "brook" });
            let hash = contest_hash(self.seed, self.time, "drink", &subject, actor.id);
            let sick = (hash % 1000) < u64::from(chance);
            let thirst_before = actor.thirst;
            let thirst_after = thirst_before.saturating_sub(self.tuning.drink_relief);
            let illness_before = actor.illness;
            let illness_after = if sick {
                (illness_before + self.tuning.illness_hit).min(1000)
            } else {
                illness_before
            };
            if let Some(person) = self.actors.get_mut(&actor.id) {
                person.thirst = thirst_after;
                person.illness = illness_after;
            }
            let drink_line = if still {
                format!("{} drinks the still water.", actor.name)
            } else {
                format!("{} drinks from the brook.", actor.name)
            };
            let drink_event = self.emit(
                drink_line,
                Scope::Cell(actor.cell),
                vec![actor.id],
                vec![actor.cell],
                vec![],
                vec![],
                Some(Draw {
                    tag: "drink".into(),
                    subject,
                    value: hash,
                }),
            );
            let turned = illness_before < self.tuning.ill_at && illness_after >= self.tuning.ill_at;
            if turned {
                self.emit(
                    format!("{}'s stomach turns.", actor.name),
                    Scope::Personal(actor.id),
                    vec![actor.id],
                    vec![actor.cell],
                    vec![],
                    vec![drink_event],
                    None,
                );
            }
            let lead = if still {
                "You drink the still water."
            } else {
                "You drink from the brook."
            };
            let mut message = if thirst_before == 0 {
                format!("{lead} You were not thirsty.")
            } else if thirst_after < self.tuning.thirsty_at {
                format!("{lead} The thirst eases.")
            } else {
                format!("{lead} You are still thirsty.")
            };
            if turned {
                message.push_str(" Your stomach turns.");
            }
            self.finish(intent.command_id, intent.actor_id, true, message, results);
        }
    }

    fn apply_eats(&mut self, eats: Vec<Intent>, results: &mut Vec<CommandResult>) {
        for intent in eats {
            let Attempt::Eat(material) = intent.attempt else {
                continue;
            };
            if material == Material::BitterBerry {
                self.eat_poison(intent, results);
                continue;
            }
            if material != Material::Berry || !self.has_material_at_hand(intent.actor_id, material)
            {
                self.finish(
                    intent.command_id,
                    intent.actor_id,
                    false,
                    "There are no sweet berries here to eat.",
                    results,
                );
                continue;
            }
            let Ok(item) = self.take_hand(intent.actor_id, Material::Berry) else {
                self.finish(
                    intent.command_id,
                    intent.actor_id,
                    false,
                    "There are no sweet berries here to eat.",
                    results,
                );
                continue;
            };
            let before = self
                .actor(intent.actor_id)
                .map(|actor| actor.energy)
                .unwrap_or(0);
            let after = before.saturating_sub(self.tuning.eat_relief);
            if let Some(person) = self.actors.get_mut(&intent.actor_id) {
                person.energy = after;
            }
            let cell = self
                .actor(intent.actor_id)
                .map(|actor| actor.cell)
                .unwrap_or(0);
            let name = self.name_of(intent.actor_id);
            self.emit(
                format!("{name} eats a handful of berries."),
                Scope::Cell(cell),
                vec![intent.actor_id],
                vec![cell],
                item.into_iter().collect(),
                vec![],
                None,
            );
            let message = if before == 0 {
                "You eat a handful of berries. You were rested."
            } else if after < self.tuning.tiring_at {
                "You eat a handful of berries. The tiredness eases."
            } else {
                "You eat a handful of berries. You are still tiring."
            };
            self.finish(intent.command_id, intent.actor_id, true, message, results);
        }
    }

    fn eat_poison(&mut self, intent: Intent, results: &mut Vec<CommandResult>) {
        if !self.has_material_at_hand(intent.actor_id, Material::BitterBerry) {
            self.finish(
                intent.command_id,
                intent.actor_id,
                false,
                "There are no poison berries here to eat.",
                results,
            );
            return;
        }
        let Ok(item) = self.take_hand(intent.actor_id, Material::BitterBerry) else {
            self.finish(
                intent.command_id,
                intent.actor_id,
                false,
                "There are no poison berries here to eat.",
                results,
            );
            return;
        };
        let illness_before = self
            .actor(intent.actor_id)
            .map(|actor| actor.illness)
            .unwrap_or(0);
        let illness_after = (illness_before + self.tuning.illness_hit).min(1000);
        if let Some(person) = self.actors.get_mut(&intent.actor_id) {
            person.illness = illness_after;
        }
        let cell = self
            .actor(intent.actor_id)
            .map(|actor| actor.cell)
            .unwrap_or(0);
        let name = self.name_of(intent.actor_id);
        let eaten = self.emit(
            format!("{name} eats a handful of poison berries."),
            Scope::Cell(cell),
            vec![intent.actor_id],
            vec![cell],
            item.into_iter().collect(),
            vec![],
            None,
        );
        let turned = illness_before < self.tuning.ill_at && illness_after >= self.tuning.ill_at;
        if turned {
            self.emit(
                format!("{name}'s stomach turns."),
                Scope::Personal(intent.actor_id),
                vec![intent.actor_id],
                vec![cell],
                vec![],
                vec![eaten],
                None,
            );
        }
        let message = if turned {
            "You eat a handful of poison berries. Your stomach turns."
        } else {
            "You eat a handful of poison berries. You are still ill."
        };
        self.finish(intent.command_id, intent.actor_id, true, message, results);
    }

    fn apply_smears(&mut self, smears: Vec<Intent>, results: &mut Vec<CommandResult>) {
        for intent in smears {
            let Some(item_id) = self
                .loose_ids(intent.actor_id, Material::Mud)
                .first()
                .copied()
            else {
                self.finish(
                    intent.command_id,
                    intent.actor_id,
                    false,
                    "You have no mud to smear.",
                    results,
                );
                continue;
            };
            self.items.remove(&item_id);
            if let Some(person) = self.actors.get_mut(&intent.actor_id) {
                person.mud = 1000;
            }
            let cell = self
                .actor(intent.actor_id)
                .map(|actor| actor.cell)
                .unwrap_or(0);
            let name = self.name_of(intent.actor_id);
            self.emit(
                format!("{name} smears mud on their skin."),
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
                "You smear mud on your skin.",
                results,
            );
        }
    }

    fn gather_mud(
        &mut self,
        intent: Intent,
        start_cells: &BTreeMap<crate::model::ActorId, crate::model::CellId>,
        results: &mut Vec<CommandResult>,
    ) {
        let cell = start_cells.get(&intent.actor_id).copied().unwrap_or(0);
        if !self.water_here(cell) {
            self.finish(
                intent.command_id,
                intent.actor_id,
                false,
                "There is no mud here to gather.",
                results,
            );
            return;
        }
        if self.carry_count(intent.actor_id) >= self.tuning.carry_slots {
            self.finish(
                intent.command_id,
                intent.actor_id,
                false,
                "Your arms are full.",
                results,
            );
            return;
        }
        let item = self.mint_item(Material::Mud, 0, cell, Some(intent.actor_id), Role::Loose);
        let name = self.name_of(intent.actor_id);
        self.emit(
            format!("{name} gathers a handful of mud."),
            Scope::Cell(cell),
            vec![intent.actor_id],
            vec![cell],
            vec![item],
            vec![],
            None,
        );
        self.finish(
            intent.command_id,
            intent.actor_id,
            true,
            "You gather a handful of mud.",
            results,
        );
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

/// `who` is the public line. None is the line the lighter hears.
fn light_line(who: Option<&str>, bark_arms: bool, branch_arms: bool) -> String {
    let fuel = match (bark_arms, branch_arms) {
        (true, true) => "the bark and the branch from the arms",
        (false, false) => "bark and a dry branch from the pile at the feet",
        (true, false) => "the bark from the arms and a dry branch from the pile at the feet",
        (false, true) => "bark from the pile at the feet and the branch from the arms",
    };
    match who {
        Some(name) => format!("{name} lights a fire with {fuel}."),
        None => format!("You light a fire with {fuel}."),
    }
}
