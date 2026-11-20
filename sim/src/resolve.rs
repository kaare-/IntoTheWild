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
        let mut ochre_smears = Vec::new();
        let mut marks = Vec::new();
        let mut sets = Vec::new();
        let mut smelts = Vec::new();
        let mut feeds = Vec::new();
        let mut hammers = Vec::new();
        let mut pierces = Vec::new();
        let mut sews = Vec::new();
        let mut binds = Vec::new();
        let mut knaps = Vec::new();
        let mut hafts = Vec::new();
        let mut hunts = Vec::new();
        let mut butchers = Vec::new();
        let mut cuts = Vec::new();
        let mut shapes = Vec::new();
        let mut firings = Vec::new();
        let mut wears = Vec::new();
        let mut removes = Vec::new();
        let mut rests = Vec::new();

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
                        Attempt::SmearOchre => ochre_smears.push(intent),
                        Attempt::Mark | Attempt::MarkRock => marks.push(intent),
                        Attempt::SetClay => sets.push(intent),
                        Attempt::Smelt | Attempt::SmeltIron => smelts.push(intent),
                        Attempt::Feed => feeds.push(intent),
                        Attempt::Hammer | Attempt::HammerIron => hammers.push(intent),
                        Attempt::Pierce => pierces.push(intent),
                        Attempt::Sew => sews.push(intent),
                        Attempt::BindSpear => binds.push(intent),
                        Attempt::Knap => knaps.push(intent),
                        Attempt::Haft => hafts.push(intent),
                        Attempt::Hunt => hunts.push(intent),
                        Attempt::Butcher => butchers.push(intent),
                        Attempt::Cut => cuts.push(intent),
                        Attempt::Shape => shapes.push(intent),
                        Attempt::FirePot => firings.push(intent),
                        Attempt::Wear => wears.push(intent),
                        Attempt::Remove => removes.push(intent),
                        Attempt::Rest => rests.push(intent),
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
        self.apply_feeds(feeds, &mut results);
        self.apply_smelts(smelts, &mut results);
        self.apply_hammers(hammers, &mut results);
        self.apply_pierces(pierces, &mut results);
        self.apply_sews(sews, &mut results);
        self.apply_binds(binds, &mut results);
        self.apply_knaps(knaps, &mut results);
        self.apply_hafts(hafts, &mut results);
        self.apply_hunts(hunts, &mut results);
        self.apply_butchers(butchers, &mut results);
        self.apply_cuts(cuts, &mut results);
        self.apply_shapes(shapes, &mut results);
        self.apply_pot_firings(firings, &mut results);
        self.apply_wears(wears, &mut results);
        self.apply_removes(removes, &mut results);
        self.apply_rests(rests, &mut results);
        self.apply_lights(lights, &start_cells, &mut results);
        self.apply_cooks(cooks, &mut results);
        self.apply_eats(eats, &mut results);
        self.apply_smears(smears, &mut results);
        self.apply_ochre_smears(ochre_smears, &mut results);
        self.apply_marks(marks, &mut results);
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
            Attempt::Knap => {
                if busy {
                    Err("You are in the middle of something.".into())
                } else if body_taken {
                    Err("You already committed to something else this window.".into())
                } else if gripping {
                    Err("Your hands are on the log.".into())
                } else if self.loose_ids(actor.id, Material::Flint).is_empty() {
                    Err("You are not carrying a flint nodule.".into())
                } else {
                    Ok(true)
                }
            }
            Attempt::Haft => {
                if busy {
                    Err("You are in the middle of something.".into())
                } else if body_taken {
                    Err("You already committed to something else this window.".into())
                } else if gripping {
                    Err("Your hands are on the log.".into())
                } else if self.loose_ids(actor.id, Material::Flake).is_empty() {
                    Err("You are not carrying a flint flake.".into())
                } else if self.loose_ids(actor.id, Material::Branch).is_empty() {
                    Err("You are not carrying a branch.".into())
                } else if self.dry_loose_ids(actor.id, Material::Branch).is_empty() {
                    Err("The branch is too damp to haft.".into())
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
            Attempt::Wear => {
                if busy {
                    Err("You are in the middle of something.".into())
                } else if body_taken {
                    Err("You already committed to something else this window.".into())
                } else if gripping {
                    Err("Your hands are on the log.".into())
                } else if self
                    .loose_ids(actor.id, Material::Hide)
                    .iter()
                    .any(|id| self.item(*id).is_some_and(|hide| hide.worn))
                {
                    Err("The hide is already on you.".into())
                } else if self.loose_ids(actor.id, Material::Hide).is_empty() {
                    Err("You are not carrying a deer hide.".into())
                } else {
                    Ok(true)
                }
            }
            Attempt::Remove => {
                if busy {
                    Err("You are in the middle of something.".into())
                } else if body_taken {
                    Err("You already committed to something else this window.".into())
                } else if gripping {
                    Err("Your hands are on the log.".into())
                } else if !self
                    .loose_ids(actor.id, Material::Hide)
                    .iter()
                    .any(|id| self.item(*id).is_some_and(|hide| hide.worn))
                {
                    Err("You are not wearing a deer hide.".into())
                } else {
                    Ok(true)
                }
            }
            Attempt::FirePot => {
                if busy {
                    Err("You are in the middle of something.".into())
                } else if body_taken {
                    Err("You already committed to something else this window.".into())
                } else if gripping {
                    Err("Your hands are on the log.".into())
                } else if self.unfired_pot(actor.id).is_none() {
                    if self.pot_already_firing(actor.id) {
                        Err("The pot is already in the fire.".into())
                    } else {
                        Err("You have no unfired pot to fire.".into())
                    }
                } else if self.fire_at(actor.cell).is_none() {
                    Err("There is no fire burning here.".into())
                } else {
                    Ok(true)
                }
            }
            Attempt::Shape => {
                if busy {
                    Err("You are in the middle of something.".into())
                } else if body_taken {
                    Err("You already committed to something else this window.".into())
                } else if gripping {
                    Err("Your hands are on the log.".into())
                } else if self.clay_to_shape(actor.id).is_none() {
                    if self.shaped_pot_near(actor.id) {
                        Err("The clay is already a pot.".into())
                    } else {
                        Err("You have no clay to shape.".into())
                    }
                } else {
                    Ok(true)
                }
            }
            Attempt::Cut => {
                if busy {
                    Err("You are in the middle of something.".into())
                } else if body_taken {
                    Err("You already committed to something else this window.".into())
                } else if gripping {
                    Err("Your hands are on the log.".into())
                } else if self.loose_ids(actor.id, Material::Knife).is_empty() {
                    Err("You need an iron knife to cut the meat.".into())
                } else {
                    match self.meat_to_cut(actor.id) {
                        Ok(_) => Ok(true),
                        Err(message) => Err(message.into()),
                    }
                }
            }
            Attempt::Butcher => {
                if busy {
                    Err("You are in the middle of something.".into())
                } else if body_taken {
                    Err("You already committed to something else this window.".into())
                } else if gripping {
                    Err("Your hands are on the log.".into())
                } else if !self.has_butcher_tool(actor.id) {
                    Err("You need a flint spear or an iron knife.".into())
                } else if self.ground_ids(actor.cell, Material::Carcass).is_empty() {
                    Err("There is no carcass here to butcher.".into())
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
                    } else if self
                        .loose_ids(actor.id, Material::Spear)
                        .iter()
                        .any(|id| self.item(*id).is_some_and(|spear| spear.flint_tip))
                    {
                        Err("The flint tip is not fire-hardened.".into())
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
                } else if !self
                    .loose_ids(actor.id, Material::Clay)
                    .iter()
                    .any(|id| self.item(*id).is_some_and(|clay| !clay.shaped))
                {
                    Err("You are not carrying clay.".into())
                } else if self.fire_at(actor.cell).is_none() {
                    Err("There is no fire burning here.".into())
                } else {
                    Ok(true)
                }
            }
            Attempt::Smelt => {
                if busy {
                    Err("You are in the middle of something.".into())
                } else if body_taken {
                    Err("You already committed to something else this window.".into())
                } else if gripping {
                    Err("Your hands are on the log.".into())
                } else if !self.loose_ids(actor.id, Material::CopperOre).is_empty() {
                    if self.fire_at(actor.cell).is_none() {
                        Err("There is no fire burning here.".into())
                    } else {
                        Ok(true)
                    }
                } else if !self.loose_ids(actor.id, Material::IronOre).is_empty() {
                    self.iron_smelt_ready(actor)
                } else {
                    Err("You are not carrying copper ore.".into())
                }
            }
            Attempt::SmeltIron => {
                if busy {
                    Err("You are in the middle of something.".into())
                } else if body_taken {
                    Err("You already committed to something else this window.".into())
                } else if gripping {
                    Err("Your hands are on the log.".into())
                } else {
                    self.iron_smelt_ready(actor)
                }
            }
            Attempt::Feed => {
                if busy {
                    Err("You are in the middle of something.".into())
                } else if body_taken {
                    Err("You already committed to something else this window.".into())
                } else if gripping {
                    Err("Your hands are on the log.".into())
                } else if self.fire_at(actor.cell).is_none() {
                    Err("There is no fire burning here.".into())
                } else if self.fire_at(actor.cell).is_some_and(|fire| fire.hot) {
                    Err("The fire is already hot enough for iron.".into())
                } else if !self.has_dry(actor.id, Material::Branch) {
                    Err("You have no dry branch to feed the fire.".into())
                } else {
                    Ok(true)
                }
            }
            Attempt::Hammer => {
                if busy {
                    Err("You are in the middle of something.".into())
                } else if body_taken {
                    Err("You already committed to something else this window.".into())
                } else if gripping {
                    Err("Your hands are on the log.".into())
                } else if !self.loose_ids(actor.id, Material::Copper).is_empty() {
                    if self.fire_at(actor.cell).is_none()
                        && self.loose_ids(actor.id, Material::Stone).is_empty()
                    {
                        Err("You need a fire or a stone to hammer the bead.".into())
                    } else {
                        Ok(true)
                    }
                } else if !self.loose_ids(actor.id, Material::Iron).is_empty() {
                    self.iron_hammer_ready(actor)
                } else {
                    Err("You are not carrying a copper bead.".into())
                }
            }
            Attempt::HammerIron => {
                if busy {
                    Err("You are in the middle of something.".into())
                } else if body_taken {
                    Err("You already committed to something else this window.".into())
                } else if gripping {
                    Err("Your hands are on the log.".into())
                } else {
                    self.iron_hammer_ready(actor)
                }
            }
            Attempt::Pierce => {
                if busy {
                    Err("You are in the middle of something.".into())
                } else if body_taken {
                    Err("You already committed to something else this window.".into())
                } else if gripping {
                    Err("Your hands are on the log.".into())
                } else if self.loose_ids(actor.id, Material::Awl).is_empty() {
                    Err("You are not carrying a copper awl.".into())
                } else if self.loose_ids(actor.id, Material::Hide).is_empty() {
                    Err("You are not carrying a deer hide.".into())
                } else if self
                    .loose_ids(actor.id, Material::Hide)
                    .iter()
                    .all(|id| self.item(*id).is_some_and(|hide| hide.pierced))
                {
                    Err("The deer hide is already pierced.".into())
                } else {
                    Ok(true)
                }
            }
            Attempt::Rest => {
                if busy {
                    Err("You are in the middle of something.".into())
                } else if body_taken {
                    Err("You already committed to something else this window.".into())
                } else if gripping {
                    Err("Your hands are on the log.".into())
                } else if self.exposure_millis(actor.cell) >= 1000 {
                    Err("The wind will not let you rest.".into())
                } else {
                    Ok(true)
                }
            }
            Attempt::Sew => {
                if busy {
                    Err("You are in the middle of something.".into())
                } else if body_taken {
                    Err("You already committed to something else this window.".into())
                } else if gripping {
                    Err("Your hands are on the log.".into())
                } else if self.loose_ids(actor.id, Material::Hide).is_empty() {
                    Err("You are not carrying a deer hide.".into())
                } else if self
                    .loose_ids(actor.id, Material::Hide)
                    .iter()
                    .any(|id| self.item(*id).is_some_and(|hide| hide.pierced))
                {
                    Ok(true)
                } else if self
                    .loose_ids(actor.id, Material::Hide)
                    .iter()
                    .any(|id| self.item(*id).is_some_and(|hide| hide.sewn))
                {
                    Err("The deer hide is already a cloak.".into())
                } else {
                    Err("The deer hide is not pierced.".into())
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
                        Material::Mushroom => {
                            if self.cooked_mushroom_id(actor.id).is_some() {
                                Ok(true)
                            } else if self.raw_mushroom_near(actor.id) {
                                Err("Raw mushrooms do not feed you.".into())
                            } else {
                                Err("There is no cooked mushroom here to eat.".into())
                            }
                        }
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
                        Material::Meat => {
                            if self.cooked_meat_id(actor.id).is_some() {
                                Ok(true)
                            } else if self.raw_meat_near(actor.id) {
                                Err("Raw meat is not food.".into())
                            } else {
                                Err("There is no cooked meat here to eat.".into())
                            }
                        }
                        Material::Clay => Err("Unfired clay is not food.".into()),
                        Material::Bowl => Err("A bowl is not food.".into()),
                        Material::Bark => Err("Bark is tinder, not food.".into()),
                        Material::Ochre => Err("Ochre is not food.".into()),
                        Material::CopperOre => Err("Copper ore is not food.".into()),
                        Material::IronOre => Err("Iron ore is not food.".into()),
                        Material::Iron => Err("Iron is not food.".into()),
                        Material::Knife => Err("An iron knife is not food.".into()),
                        Material::Copper => Err("Copper is not food.".into()),
                        Material::Awl => Err("A copper awl is not food.".into()),
                        _ => Err("Bark is tinder, not food.".into()),
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
            Attempt::SmearOchre => {
                if busy {
                    Err("You are in the middle of something.".into())
                } else if body_taken {
                    Err("You already committed to something else this window.".into())
                } else if gripping {
                    Err("Your hands are on the log.".into())
                } else if self.loose_ids(actor.id, Material::Ochre).is_empty() {
                    Err("You have no ochre to smear.".into())
                } else {
                    Ok(true)
                }
            }
            Attempt::Mark | Attempt::MarkRock => {
                if busy {
                    Err("You are in the middle of something.".into())
                } else if body_taken {
                    Err("You already committed to something else this window.".into())
                } else if gripping {
                    Err("Your hands are on the log.".into())
                } else if matches!(attempt, Attempt::MarkRock) && !self.cell(actor.cell).rock {
                    Err("There is no rock here to mark.".into())
                } else if self.ochre_marks.contains(&actor.cell) {
                    Err("A red mark is already here.".into())
                } else if self.loose_ids(actor.id, Material::Ochre).is_empty() {
                    Err("You have no ochre to mark with.".into())
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
                            | Material::Pot
                            | Material::Spear
                            | Material::Carcass
                            | Material::Meat
                            | Material::Hide
                            | Material::Branch
                            | Material::CopperOre
                            | Material::IronOre
                            | Material::Iron
                            | Material::Knife
                            | Material::Copper
                            | Material::Awl
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
                    | Material::Pot
                    | Material::Spear
                    | Material::Carcass
                    | Material::Meat
                    | Material::Hide
                    | Material::Branch
                    | Material::Flint
                    | Material::CopperOre
                    | Material::IronOre
                    | Material::Iron
                    | Material::Knife
                    | Material::Copper
                    | Material::Awl
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
                    let from = if material == Material::Pot {
                        format!(" from the ground {}", self.cover_phrase(cell))
                    } else {
                        String::new()
                    };
                    let shaped = material == Material::Clay
                        && self.item(item).is_some_and(|clay| clay.shaped);
                    let rest = if shaped {
                        "an unfired clay pot.".to_string()
                    } else {
                        format!("{} {noun}{from}.{breaking}", material.article())
                    };
                    let text = crate::model::spoken_act(Some(name.as_str()), "gather", &rest);
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
                        crate::model::spoken_act(None, "gather", &rest),
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
                item.worn = false;
            }
            let noun = material.gather_noun();
            self.emit(
                format!(
                    "{name} drops {} {noun}. It is in contact with the ground.",
                    material.article()
                ),
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
                format!(
                    "You drop {} {noun}. It is in contact with the ground.",
                    material.article()
                ),
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
            let left_overhang = self.cell(origin).rock
                && !self.cell(dest).rock
                && self.exposure_millis(dest) >= 1000;
            let rain_too = self.raining || self.time >= self.tuning.rain_starts_at;
            let shelter = if rain_too {
                "out of the wind and the rain"
            } else {
                "out of the wind"
            };
            let exposure = if rain_too {
                "out from under the rock overhang, into the wind and the rain"
            } else {
                "out from under the rock overhang, into the wind"
            };
            let text = if brook {
                format!("{name} wades {word}.", word = dir.word())
            } else if entered_overhang {
                format!(
                    "{name} walks {word}, under the rock overhang, {shelter}.",
                    word = dir.word()
                )
            } else if left_overhang {
                format!("{name} walks {word}, {exposure}.", word = dir.word())
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
                    "You walk {}, under the rock overhang, {shelter}.",
                    dir.word()
                )
            } else if left_overhang {
                format!("You walk {}, {exposure}.", dir.word())
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
        if !self.cookable_carried(actor)
            && self.cookable_pile(person.cell) == 0
            && self.raw_meat_at(person.cell).is_none()
        {
            if self.has_material_at_hand(actor, Material::BitterBerry) {
                return Some("Cooking does not make poison berries food.".into());
            }
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
            let cell = self
                .actor(intent.actor_id)
                .map(|actor| actor.cell)
                .unwrap_or(0);
            if self.raw_meat_at(cell).is_some() {
                self.cook_meat(intent, results);
                continue;
            }
            if self.raw_mushroom_id(intent.actor_id).is_some()
                || self.pile_count(cell, Material::Mushroom) > 0
            {
                self.cook_mushroom(intent, results);
                continue;
            }
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

    /// Heat leaves the mushroom in the arms, or on the ground if it was a pile.
    /// It does not feed anyone until it is eaten.
    fn cook_mushroom(&mut self, intent: Intent, results: &mut Vec<CommandResult>) {
        if let Some(message) = self.cook_refusal(intent.actor_id) {
            self.finish(intent.command_id, intent.actor_id, false, message, results);
            return;
        }
        let cell = self
            .actor(intent.actor_id)
            .map(|actor| actor.cell)
            .unwrap_or(0);
        let id = if let Some(id) = self.raw_mushroom_id(intent.actor_id) {
            if let Some(mushroom) = self.items.get_mut(&id) {
                mushroom.cooked = true;
            }
            id
        } else if self.pile_count(cell, Material::Mushroom) > 0 {
            let moisture = self
                .take_from_pile(cell, Material::Mushroom, 1)
                .unwrap_or(0);
            let id = self.mint_item(Material::Mushroom, moisture, cell, None, Role::Loose);
            if let Some(mushroom) = self.items.get_mut(&id) {
                mushroom.cooked = true;
            }
            id
        } else {
            self.finish(
                intent.command_id,
                intent.actor_id,
                false,
                "There is nothing here to cook.",
                results,
            );
            return;
        };
        let rest = "the mushroom over the fire.";
        let name = self.name_of(intent.actor_id);
        let fire_event = self
            .fire_at(cell)
            .map(|fire| fire.lit_event)
            .into_iter()
            .collect();
        self.emit(
            crate::model::spoken_act(Some(name.as_str()), "cook", rest),
            Scope::Cell(cell),
            vec![intent.actor_id],
            vec![cell],
            vec![id],
            fire_event,
            None,
        );
        self.finish(
            intent.command_id,
            intent.actor_id,
            true,
            crate::model::spoken_act(None, "cook", rest),
            results,
        );
    }

    /// Raw meat on the fire's own cell. It stays on the ground and is not eaten.
    fn cook_meat(&mut self, intent: Intent, results: &mut Vec<CommandResult>) {
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
        let Some(id) = self.raw_meat_at(cell) else {
            self.finish(
                intent.command_id,
                intent.actor_id,
                false,
                "There is nothing here to cook.",
                results,
            );
            return;
        };
        if let Some(meat) = self.items.get_mut(&id) {
            meat.cooked = true;
        }
        let rest = "the raw meat over the fire.";
        let name = self.name_of(intent.actor_id);
        let fire_event = self
            .fire_at(cell)
            .map(|fire| fire.lit_event)
            .into_iter()
            .collect();
        self.emit(
            crate::model::spoken_act(Some(name.as_str()), "cook", rest),
            Scope::Cell(cell),
            vec![intent.actor_id],
            vec![cell],
            vec![id],
            fire_event,
            None,
        );
        self.finish(
            intent.command_id,
            intent.actor_id,
            true,
            crate::model::spoken_act(None, "cook", rest),
            results,
        );
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
        let from_brook = self.cell(cell).region == Region::Brook;
        if from_brook {
            if let Some(drink) = self.items.get_mut(&item) {
                drink.raw = true;
            }
        }
        let name = self.name_of(intent.actor_id);
        let source = if from_brook {
            "the brook"
        } else {
            "the still water"
        };
        let into_pot = !self.loose_ids(intent.actor_id, Material::Pot).is_empty();
        let pot = if into_pot { " into the pot" } else { "" };
        let rest = format!("a drink of water from {source}{pot}.");
        self.emit(
            crate::model::spoken_act(Some(name.as_str()), "gather", &rest),
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
            crate::model::spoken_act(None, "gather", &rest),
            results,
        );
    }

    fn apply_knaps(&mut self, knaps: Vec<Intent>, results: &mut Vec<CommandResult>) {
        let rest = "the flint nodule into a sharp flake.";
        for intent in knaps {
            let nodule = self
                .loose_ids(intent.actor_id, Material::Flint)
                .first()
                .copied();
            let Some(nodule) = nodule else {
                self.finish(
                    intent.command_id,
                    intent.actor_id,
                    false,
                    "You are not carrying a flint nodule.",
                    results,
                );
                continue;
            };
            let cell = self
                .actor(intent.actor_id)
                .map(|actor| actor.cell)
                .unwrap_or(0);
            self.items.remove(&nodule);
            let id = self.mint_item(Material::Flake, 0, cell, Some(intent.actor_id), Role::Loose);
            let name = self.name_of(intent.actor_id);
            self.emit(
                crate::model::spoken_act(Some(name.as_str()), "knap", rest),
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
                crate::model::spoken_act(None, "knap", rest),
                results,
            );
        }
    }

    fn apply_hafts(&mut self, hafts: Vec<Intent>, results: &mut Vec<CommandResult>) {
        let rest = "the flint flake and a dry branch into a spear.";
        for intent in hafts {
            let flake = self
                .loose_ids(intent.actor_id, Material::Flake)
                .first()
                .copied();
            let branch = self
                .dry_loose_ids(intent.actor_id, Material::Branch)
                .first()
                .copied();
            let (Some(flake), Some(branch)) = (flake, branch) else {
                self.finish(
                    intent.command_id,
                    intent.actor_id,
                    false,
                    "You need a carried flint flake and a carried dry branch.",
                    results,
                );
                continue;
            };
            let cell = self
                .actor(intent.actor_id)
                .map(|actor| actor.cell)
                .unwrap_or(0);
            self.items.remove(&flake);
            self.items.remove(&branch);
            let id = self.mint_item(Material::Spear, 0, cell, Some(intent.actor_id), Role::Loose);
            if let Some(spear) = self.items.get_mut(&id) {
                spear.flint_tip = true;
            }
            let name = self.name_of(intent.actor_id);
            self.emit(
                crate::model::spoken_act(Some(name.as_str()), "haft", rest),
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
                crate::model::spoken_act(None, "haft", rest),
                results,
            );
        }
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
                } else if self
                    .loose_ids(actor.id, Material::Spear)
                    .iter()
                    .any(|id| self.item(*id).is_some_and(|spear| spear.flint_tip))
                {
                    "The flint tip is not fire-hardened."
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
                    let flint = self
                        .loose_ids(actor.id, Material::Spear)
                        .iter()
                        .any(|id| self.item(*id).is_some_and(|spear| spear.flint_tip));
                    let (heard, yours) = if flint {
                        (
                            format!("{name} misses the deer with the flint spear. It moves off."),
                            "You miss the deer with the flint spear. It moves off.",
                        )
                    } else {
                        (
                            format!("{name} misses the deer. It moves off."),
                            "You miss the deer. It moves off.",
                        )
                    };
                    self.emit(
                        heard,
                        Scope::Cell(deer.cell),
                        vec![actor.id],
                        vec![deer.cell],
                        vec![],
                        vec![],
                        draw,
                    );
                    self.finish(intent.command_id, actor.id, false, yours, results);
                }
                _ => {
                    self.flee_deer(actor.cell);
                    let name = self.name_of(actor.id);
                    let flint = self
                        .loose_ids(actor.id, Material::Spear)
                        .iter()
                        .any(|id| self.item(*id).is_some_and(|spear| spear.flint_tip));
                    let (heard, yours) = if flint {
                        (
                            format!("The deer bolts. {name}'s flint spear does not land."),
                            "The deer bolts. Your flint spear does not land.",
                        )
                    } else {
                        (
                            format!("The deer bolts from {name}'s spear."),
                            "The deer bolts. Your spear does not land.",
                        )
                    };
                    self.emit(
                        heard,
                        Scope::Cell(deer.cell),
                        vec![actor.id],
                        vec![deer.cell],
                        vec![],
                        vec![],
                        draw,
                    );
                    self.finish(intent.command_id, actor.id, false, yours, results);
                }
            }
        }
    }

    fn apply_wears(&mut self, wears: Vec<Intent>, results: &mut Vec<CommandResult>) {
        for intent in wears {
            let Some(id) = self
                .loose_ids(intent.actor_id, Material::Hide)
                .into_iter()
                .find(|id| self.item(*id).is_some_and(|hide| !hide.worn))
            else {
                self.finish(
                    intent.command_id,
                    intent.actor_id,
                    false,
                    "You are not carrying a deer hide.",
                    results,
                );
                continue;
            };
            let sewn = self.item(id).is_some_and(|hide| hide.sewn);
            let rest = if sewn {
                "the sewn deer-hide cloak."
            } else {
                "the deer hide."
            };
            if let Some(hide) = self.items.get_mut(&id) {
                hide.worn = true;
            }
            let cell = self
                .actor(intent.actor_id)
                .map(|actor| actor.cell)
                .unwrap_or(0);
            let name = self.name_of(intent.actor_id);
            self.emit(
                crate::model::spoken_act(Some(name.as_str()), "wear", rest),
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
                crate::model::spoken_act(None, "wear", rest),
                results,
            );
        }
    }

    fn apply_removes(&mut self, removes: Vec<Intent>, results: &mut Vec<CommandResult>) {
        for intent in removes {
            let Some(id) = self
                .loose_ids(intent.actor_id, Material::Hide)
                .into_iter()
                .find(|id| self.item(*id).is_some_and(|hide| hide.worn))
            else {
                self.finish(
                    intent.command_id,
                    intent.actor_id,
                    false,
                    "You are not wearing a deer hide.",
                    results,
                );
                continue;
            };
            let sewn = self.item(id).is_some_and(|hide| hide.sewn);
            let rest = if sewn {
                "the sewn deer-hide cloak."
            } else {
                "the deer hide."
            };
            if let Some(hide) = self.items.get_mut(&id) {
                hide.worn = false;
            }
            let cell = self
                .actor(intent.actor_id)
                .map(|actor| actor.cell)
                .unwrap_or(0);
            let name = self.name_of(intent.actor_id);
            self.emit(
                crate::model::spoken_act(Some(name.as_str()), "remove", rest),
                Scope::Cell(cell),
                vec![intent.actor_id],
                vec![cell],
                vec![id],
                vec![],
                None,
            );
            let mut yours = crate::model::spoken_act(None, "remove", rest);
            if self.exposure_millis(cell) >= 1000 {
                yours.push_str(" The wind reaches you again.");
            }
            self.finish(intent.command_id, intent.actor_id, true, yours, results);
        }
    }

    fn apply_rests(&mut self, rests: Vec<Intent>, results: &mut Vec<CommandResult>) {
        for intent in rests {
            let Some(actor) = self.actor(intent.actor_id).cloned() else {
                continue;
            };
            let before = actor.energy;
            let after = before.saturating_sub(self.tuning.eat_relief);
            let was = self.energy_word(before);
            let now = self.energy_word(after);
            if let Some(person) = self.actors.get_mut(&intent.actor_id) {
                person.energy = after;
            }
            let under_rock = self.cell(actor.cell).rock;
            let place = if under_rock { " under the rock" } else { "" };
            let name = self.name_of(intent.actor_id);
            self.emit(
                format!("{name} rests{place}."),
                Scope::Cell(actor.cell),
                vec![intent.actor_id],
                vec![actor.cell],
                vec![],
                vec![],
                None,
            );
            let mut message = format!("You rest{place}.");
            if was != now {
                message.push_str(&format!(" You are {now}."));
            }
            self.finish(intent.command_id, intent.actor_id, true, message, results);
        }
    }

    fn has_flint_spear(&self, actor: crate::model::ActorId) -> bool {
        self.loose_ids(actor, Material::Spear)
            .iter()
            .any(|id| self.item(*id).is_some_and(|spear| spear.flint_tip))
    }

    fn has_butcher_tool(&self, actor: crate::model::ActorId) -> bool {
        self.has_flint_spear(actor) || !self.loose_ids(actor, Material::Knife).is_empty()
    }

    fn apply_butchers(&mut self, butchers: Vec<Intent>, results: &mut Vec<CommandResult>) {
        for intent in butchers {
            let Some(actor) = self.actor(intent.actor_id).cloned() else {
                continue;
            };
            let with_knife = !self.loose_ids(actor.id, Material::Knife).is_empty();
            if !self.has_butcher_tool(actor.id) {
                self.finish(
                    intent.command_id,
                    actor.id,
                    false,
                    "You need a flint spear or an iron knife.",
                    results,
                );
                continue;
            }
            let Some(carcass) = self
                .ground_ids(actor.cell, Material::Carcass)
                .first()
                .copied()
            else {
                self.finish(
                    intent.command_id,
                    actor.id,
                    false,
                    "There is no carcass here to butcher.",
                    results,
                );
                continue;
            };
            self.items.remove(&carcass);
            let mut ids = Vec::new();
            for _ in 0..3 {
                ids.push(self.mint_item(Material::Meat, 0, actor.cell, None, Role::Loose));
            }
            ids.push(self.mint_item(Material::Hide, 0, actor.cell, None, Role::Loose));
            let name = self.name_of(actor.id);
            let rest = if with_knife {
                "the carcass with the iron knife into three portions of raw meat and a hide."
            } else {
                "the carcass into three portions of raw meat and a hide."
            };
            self.emit(
                crate::model::spoken_act(Some(name.as_str()), "butcher", rest),
                Scope::Cell(actor.cell),
                vec![actor.id],
                vec![actor.cell],
                ids,
                vec![],
                None,
            );
            self.finish(
                intent.command_id,
                actor.id,
                true,
                crate::model::spoken_act(None, "butcher", rest),
                results,
            );
        }
    }

    /// A whole portion in the arms, or else one on this cell.
    /// Cutting a carried portion needs a free arm. If the arms are full and a
    /// whole portion is also on the ground, that one is cut instead.
    fn meat_to_cut(
        &self,
        actor: crate::model::ActorId,
    ) -> Result<crate::model::ItemId, &'static str> {
        let carried = self
            .loose_ids(actor, Material::Meat)
            .into_iter()
            .find(|id| self.item(*id).is_some_and(|meat| !meat.small));
        let cell = self.actor(actor).map(|person| person.cell).unwrap_or(0);
        let ground = self
            .ground_ids(cell, Material::Meat)
            .into_iter()
            .find(|id| self.item(*id).is_some_and(|meat| !meat.small));
        if let Some(id) = carried {
            if self.carry_count(actor) < self.tuning.carry_slots {
                return Ok(id);
            }
            if ground.is_some() {
                return Ok(ground.unwrap());
            }
            return Err("Your arms are full.");
        }
        if let Some(id) = ground {
            return Ok(id);
        }
        let any = !self.loose_ids(actor, Material::Meat).is_empty()
            || !self.ground_ids(cell, Material::Meat).is_empty();
        if any {
            Err("The piece of meat is already small.")
        } else {
            Err("There is no meat here to cut.")
        }
    }

    fn unshaped_carried(&self, actor: crate::model::ActorId) -> Option<crate::model::ItemId> {
        self.loose_ids(actor, Material::Clay)
            .into_iter()
            .find(|id| self.item(*id).is_some_and(|clay| !clay.shaped))
    }

    fn unshaped_ground(&self, cell: crate::model::CellId) -> Option<crate::model::ItemId> {
        self.ground_ids(cell, Material::Clay)
            .into_iter()
            .find(|id| {
                self.item(*id)
                    .is_some_and(|clay| !clay.shaped && clay.firing_ready_at.is_none())
            })
    }

    /// A lump in the arms, else a lump on this cell, else one from the pile.
    fn clay_to_shape(&self, actor: crate::model::ActorId) -> Option<Option<crate::model::ItemId>> {
        if self.unshaped_carried(actor).is_some() {
            return Some(self.unshaped_carried(actor));
        }
        let cell = self.actor(actor).map(|person| person.cell).unwrap_or(0);
        if let Some(id) = self.unshaped_ground(cell) {
            return Some(Some(id));
        }
        if self.pile_count(cell, Material::Clay) > 0 {
            return Some(None);
        }
        None
    }

    fn unfired_pot(&self, actor: crate::model::ActorId) -> Option<crate::model::ItemId> {
        let ready = |id: crate::model::ItemId| {
            self.item(id)
                .is_some_and(|clay| clay.shaped && clay.firing_ready_at.is_none())
        };
        if let Some(id) = self
            .loose_ids(actor, Material::Clay)
            .into_iter()
            .find(|id| ready(*id))
        {
            return Some(id);
        }
        let cell = self.actor(actor).map(|person| person.cell).unwrap_or(0);
        self.ground_ids(cell, Material::Clay)
            .into_iter()
            .find(|id| ready(*id))
    }

    fn pot_already_firing(&self, actor: crate::model::ActorId) -> bool {
        let cell = self.actor(actor).map(|person| person.cell).unwrap_or(0);
        self.loose_ids(actor, Material::Clay)
            .iter()
            .chain(self.ground_ids(cell, Material::Clay).iter())
            .any(|id| {
                self.item(*id)
                    .is_some_and(|clay| clay.shaped && clay.firing_ready_at.is_some())
            })
    }

    fn apply_pot_firings(&mut self, firings: Vec<Intent>, results: &mut Vec<CommandResult>) {
        for intent in firings {
            let Some(actor) = self.actor(intent.actor_id).cloned() else {
                continue;
            };
            let Some(id) = self.unfired_pot(actor.id) else {
                let message = if self.pot_already_firing(actor.id) {
                    "The pot is already in the fire."
                } else {
                    "You have no unfired pot to fire."
                };
                self.finish(intent.command_id, actor.id, false, message, results);
                continue;
            };
            if self.fire_at(actor.cell).is_none() {
                self.finish(
                    intent.command_id,
                    actor.id,
                    false,
                    "There is no fire burning here.",
                    results,
                );
                continue;
            }
            let carried = self.item(id).and_then(|pot| pot.carrier).is_some();
            let ready = self.time.saturating_add(self.tuning.clay_ticks);
            if let Some(pot) = self.items.get_mut(&id) {
                pot.carrier = None;
                pot.cell = actor.cell;
                pot.firing_ready_at = Some(ready);
            }
            self.schedule(ready, Wake::Harden(id));
            let name = self.name_of(actor.id);
            let rest = if carried {
                "the unfired pot by the fire. It leaves their arms."
            } else {
                "the unfired pot by the fire."
            };
            let yours = if carried {
                "You set the unfired pot by the fire. It leaves your arms. It is not hard yet."
            } else {
                "You set the unfired pot by the fire. It is not hard yet."
            };
            self.emit(
                crate::model::spoken_act(Some(name.as_str()), "set", rest),
                Scope::Cell(actor.cell),
                vec![actor.id],
                vec![actor.cell],
                vec![id],
                vec![],
                None,
            );
            self.finish(intent.command_id, actor.id, true, yours, results);
        }
    }

    fn shaped_pot_near(&self, actor: crate::model::ActorId) -> bool {
        let cell = self.actor(actor).map(|person| person.cell).unwrap_or(0);
        self.loose_ids(actor, Material::Clay)
            .iter()
            .any(|id| self.item(*id).is_some_and(|clay| clay.shaped))
            || self
                .ground_ids(cell, Material::Clay)
                .iter()
                .any(|id| self.item(*id).is_some_and(|clay| clay.shaped))
    }

    fn apply_shapes(&mut self, shapes: Vec<Intent>, results: &mut Vec<CommandResult>) {
        for intent in shapes {
            let Some(actor) = self.actor(intent.actor_id).cloned() else {
                continue;
            };
            let Some(source) = self.clay_to_shape(actor.id) else {
                let message = if self.shaped_pot_near(actor.id) {
                    "The clay is already a pot."
                } else {
                    "You have no clay to shape."
                };
                self.finish(intent.command_id, actor.id, false, message, results);
                continue;
            };
            let id = if let Some(id) = source {
                if let Some(clay) = self.items.get_mut(&id) {
                    clay.shaped = true;
                }
                id
            } else {
                let moisture = self
                    .take_from_pile(actor.cell, Material::Clay, 1)
                    .unwrap_or(0);
                let id = self.mint_item(Material::Clay, moisture, actor.cell, None, Role::Loose);
                if let Some(clay) = self.items.get_mut(&id) {
                    clay.shaped = true;
                }
                id
            };
            let name = self.name_of(actor.id);
            let rest = "the clay into an unfired pot.";
            self.emit(
                crate::model::spoken_act(Some(name.as_str()), "shape", rest),
                Scope::Cell(actor.cell),
                vec![actor.id],
                vec![actor.cell],
                vec![id],
                vec![],
                None,
            );
            self.finish(
                intent.command_id,
                actor.id,
                true,
                crate::model::spoken_act(None, "shape", rest),
                results,
            );
        }
    }

    fn apply_cuts(&mut self, cuts: Vec<Intent>, results: &mut Vec<CommandResult>) {
        for intent in cuts {
            let Some(actor) = self.actor(intent.actor_id).cloned() else {
                continue;
            };
            if self.loose_ids(actor.id, Material::Knife).is_empty() {
                self.finish(
                    intent.command_id,
                    actor.id,
                    false,
                    "You need an iron knife to cut the meat.",
                    results,
                );
                continue;
            }
            let id = match self.meat_to_cut(actor.id) {
                Ok(id) => id,
                Err(message) => {
                    self.finish(intent.command_id, actor.id, false, message, results);
                    continue;
                }
            };
            let Some(meat) = self.item(id).cloned() else {
                self.finish(
                    intent.command_id,
                    actor.id,
                    false,
                    "There is no meat here to cut.",
                    results,
                );
                continue;
            };
            self.items.remove(&id);
            let mut ids = Vec::new();
            for _ in 0..2 {
                let piece = self.mint_item(Material::Meat, 0, meat.cell, meat.carrier, Role::Loose);
                if let Some(item) = self.items.get_mut(&piece) {
                    item.cooked = meat.cooked;
                    item.small = true;
                }
                ids.push(piece);
            }
            let name = self.name_of(actor.id);
            let rest = if meat.cooked {
                "the cooked meat into two smaller pieces with the iron knife."
            } else {
                "the raw meat into two smaller pieces with the iron knife."
            };
            self.emit(
                crate::model::spoken_act(Some(name.as_str()), "cut", rest),
                Scope::Cell(actor.cell),
                vec![actor.id],
                vec![actor.cell],
                ids,
                vec![],
                None,
            );
            self.finish(
                intent.command_id,
                actor.id,
                true,
                crate::model::spoken_act(None, "cut", rest),
                results,
            );
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
        let flint = self
            .loose_ids(intent.actor_id, Material::Spear)
            .iter()
            .any(|id| self.item(*id).is_some_and(|spear| spear.flint_tip));
        self.tracks.insert(deer_cell);
        self.deer = None;
        let id = self.mint_item(Material::Carcass, 0, deer_cell, None, Role::Loose);
        let (heard, yours) = if flint {
            let rest = "the deer down with the flint spear.";
            (
                crate::model::spoken_act(Some(name.as_str()), "bring", rest),
                crate::model::spoken_act(None, "bring", rest),
            )
        } else {
            (
                format!("{name} brings the deer down."),
                "You bring the deer down. A carcass lies where it stood.".to_string(),
            )
        };
        self.emit(
            heard,
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
            yours,
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
                .into_iter()
                .find(|id| self.item(*id).is_some_and(|clay| !clay.shaped))
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

    fn iron_smelt_ready(&self, actor: &crate::model::Actor) -> Result<bool, String> {
        if self.loose_ids(actor.id, Material::IronOre).is_empty() {
            Err("You are not carrying iron ore.".into())
        } else if self.fire_at(actor.cell).is_none() {
            Err("There is no fire burning here.".into())
        } else if self.fire_at(actor.cell).is_some_and(|fire| !fire.hot) {
            Err("The fire is not hot enough for iron. Feed it more fuel.".into())
        } else {
            Ok(true)
        }
    }

    fn iron_hammer_ready(&self, actor: &crate::model::Actor) -> Result<bool, String> {
        let hot = self.fire_at(actor.cell).is_some_and(|fire| fire.hot);
        if self.loose_ids(actor.id, Material::Iron).is_empty() {
            Err("You are not carrying a bloom of iron.".into())
        } else if !hot && self.loose_ids(actor.id, Material::Stone).is_empty() {
            Err("You need a hotter fire or a stone to hammer the bloom.".into())
        } else {
            Ok(true)
        }
    }

    fn apply_feeds(&mut self, feeds: Vec<Intent>, results: &mut Vec<CommandResult>) {
        for intent in feeds {
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
            if self.fire_at(cell).is_some_and(|fire| fire.hot) {
                self.finish(
                    intent.command_id,
                    intent.actor_id,
                    false,
                    "The fire is already hot enough for iron.",
                    results,
                );
                continue;
            }
            if self.consume_dry(intent.actor_id, Material::Branch).is_err() {
                self.finish(
                    intent.command_id,
                    intent.actor_id,
                    false,
                    "You have no dry branch to feed the fire.",
                    results,
                );
                continue;
            }
            if let Some(fire) = self.fires.values_mut().find(|fire| fire.cell == cell) {
                fire.hot = true;
            }
            let name = self.name_of(intent.actor_id);
            let rest = "a dry branch into the fire. It burns hotter.";
            self.emit(
                crate::model::spoken_act(Some(name.as_str()), "feed", rest),
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
                crate::model::spoken_act(None, "feed", rest),
                results,
            );
        }
    }

    fn apply_smelts(&mut self, smelts: Vec<Intent>, results: &mut Vec<CommandResult>) {
        for intent in smelts {
            let iron = matches!(intent.attempt, Attempt::SmeltIron)
                || (matches!(intent.attempt, Attempt::Smelt)
                    && self
                        .loose_ids(intent.actor_id, Material::CopperOre)
                        .is_empty()
                    && !self
                        .loose_ids(intent.actor_id, Material::IronOre)
                        .is_empty());
            if iron {
                self.apply_iron_smelt(intent, results);
                continue;
            }
            let Some(id) = self
                .loose_ids(intent.actor_id, Material::CopperOre)
                .first()
                .copied()
            else {
                self.finish(
                    intent.command_id,
                    intent.actor_id,
                    false,
                    "You are not carrying copper ore.",
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
            self.schedule(ready, Wake::Smelt(id));
            let name = self.name_of(intent.actor_id);
            self.emit(
                format!("{name} sets a lump of copper ore by the fire. It leaves their arms."),
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
                "You set a lump of copper ore by the fire. It leaves your arms. It is not metal yet.",
                results,
            );
        }
    }

    fn apply_iron_smelt(&mut self, intent: Intent, results: &mut Vec<CommandResult>) {
        let Some(id) = self
            .loose_ids(intent.actor_id, Material::IronOre)
            .first()
            .copied()
        else {
            self.finish(
                intent.command_id,
                intent.actor_id,
                false,
                "You are not carrying iron ore.",
                results,
            );
            return;
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
            return;
        }
        if self.fire_at(cell).is_some_and(|fire| !fire.hot) {
            self.finish(
                intent.command_id,
                intent.actor_id,
                false,
                "The fire is not hot enough for iron. Feed it more fuel.",
                results,
            );
            return;
        }
        let ready = self.time.saturating_add(self.tuning.clay_ticks);
        if let Some(item) = self.items.get_mut(&id) {
            item.carrier = None;
            item.cell = cell;
            item.firing_ready_at = Some(ready);
        }
        self.schedule(ready, Wake::Smelt(id));
        let name = self.name_of(intent.actor_id);
        self.emit(
            format!("{name} sets a lump of iron ore by the fire. It leaves their arms."),
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
            "You set a lump of iron ore by the fire. It leaves your arms. It is not iron yet.",
            results,
        );
    }

    fn apply_hammers(&mut self, hammers: Vec<Intent>, results: &mut Vec<CommandResult>) {
        for intent in hammers {
            let iron = matches!(intent.attempt, Attempt::HammerIron)
                || (matches!(intent.attempt, Attempt::Hammer)
                    && self.loose_ids(intent.actor_id, Material::Copper).is_empty()
                    && !self.loose_ids(intent.actor_id, Material::Iron).is_empty());
            if iron {
                self.apply_iron_hammer(intent, results);
                continue;
            }
            let bead = self
                .loose_ids(intent.actor_id, Material::Copper)
                .first()
                .copied();
            let Some(bead) = bead else {
                self.finish(
                    intent.command_id,
                    intent.actor_id,
                    false,
                    "You are not carrying a copper bead.",
                    results,
                );
                continue;
            };
            let cell = self
                .actor(intent.actor_id)
                .map(|actor| actor.cell)
                .unwrap_or(0);
            let at_fire = self.fire_at(cell).is_some();
            if !at_fire && self.loose_ids(intent.actor_id, Material::Stone).is_empty() {
                self.finish(
                    intent.command_id,
                    intent.actor_id,
                    false,
                    "You need a fire or a stone to hammer the bead.",
                    results,
                );
                continue;
            }
            let rest = if at_fire {
                "the copper bead into a copper awl at the fire."
            } else {
                "the copper bead into a copper awl with a stone."
            };
            self.items.remove(&bead);
            let id = self.mint_item(Material::Awl, 0, cell, Some(intent.actor_id), Role::Loose);
            let name = self.name_of(intent.actor_id);
            self.emit(
                crate::model::spoken_act(Some(name.as_str()), "hammer", rest),
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
                crate::model::spoken_act(None, "hammer", rest),
                results,
            );
        }
    }

    fn apply_iron_hammer(&mut self, intent: Intent, results: &mut Vec<CommandResult>) {
        let Some(bloom) = self
            .loose_ids(intent.actor_id, Material::Iron)
            .first()
            .copied()
        else {
            self.finish(
                intent.command_id,
                intent.actor_id,
                false,
                "You are not carrying a bloom of iron.",
                results,
            );
            return;
        };
        let cell = self
            .actor(intent.actor_id)
            .map(|actor| actor.cell)
            .unwrap_or(0);
        let at_fire = self.fire_at(cell).is_some_and(|fire| fire.hot);
        if !at_fire && self.loose_ids(intent.actor_id, Material::Stone).is_empty() {
            self.finish(
                intent.command_id,
                intent.actor_id,
                false,
                "You need a hotter fire or a stone to hammer the bloom.",
                results,
            );
            return;
        }
        let rest = if at_fire {
            "the bloom of iron into an iron knife at the fire."
        } else {
            "the bloom of iron into an iron knife with a stone."
        };
        self.items.remove(&bloom);
        let id = self.mint_item(Material::Knife, 0, cell, Some(intent.actor_id), Role::Loose);
        let name = self.name_of(intent.actor_id);
        self.emit(
            crate::model::spoken_act(Some(name.as_str()), "hammer", rest),
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
            crate::model::spoken_act(None, "hammer", rest),
            results,
        );
    }

    fn apply_pierces(&mut self, pierces: Vec<Intent>, results: &mut Vec<CommandResult>) {
        let rest = "the deer hide with the copper awl. It is ready to sew.";
        for intent in pierces {
            let awl = self
                .loose_ids(intent.actor_id, Material::Awl)
                .first()
                .copied();
            let hide = self
                .loose_ids(intent.actor_id, Material::Hide)
                .into_iter()
                .find(|id| self.item(*id).is_some_and(|hide| !hide.pierced));
            let (Some(_awl), Some(hide)) = (awl, hide) else {
                self.finish(
                    intent.command_id,
                    intent.actor_id,
                    false,
                    "You need a copper awl and a deer hide.",
                    results,
                );
                continue;
            };
            if let Some(hide) = self.items.get_mut(&hide) {
                hide.pierced = true;
            }
            let cell = self
                .actor(intent.actor_id)
                .map(|actor| actor.cell)
                .unwrap_or(0);
            let name = self.name_of(intent.actor_id);
            self.emit(
                crate::model::spoken_act(Some(name.as_str()), "pierce", rest),
                Scope::Cell(cell),
                vec![intent.actor_id],
                vec![cell],
                vec![hide],
                vec![],
                None,
            );
            self.finish(
                intent.command_id,
                intent.actor_id,
                true,
                crate::model::spoken_act(None, "pierce", rest),
                results,
            );
        }
    }

    fn apply_sews(&mut self, sews: Vec<Intent>, results: &mut Vec<CommandResult>) {
        let rest = "the pierced deer hide into a cloak.";
        for intent in sews {
            let hide = self
                .loose_ids(intent.actor_id, Material::Hide)
                .into_iter()
                .find(|id| self.item(*id).is_some_and(|hide| hide.pierced));
            let Some(hide) = hide else {
                self.finish(
                    intent.command_id,
                    intent.actor_id,
                    false,
                    "The deer hide is not pierced.",
                    results,
                );
                continue;
            };
            if let Some(item) = self.items.get_mut(&hide) {
                item.pierced = false;
                item.sewn = true;
            }
            let cell = self
                .actor(intent.actor_id)
                .map(|actor| actor.cell)
                .unwrap_or(0);
            let name = self.name_of(intent.actor_id);
            self.emit(
                crate::model::spoken_act(Some(name.as_str()), "sew", rest),
                Scope::Cell(cell),
                vec![intent.actor_id],
                vec![cell],
                vec![hide],
                vec![],
                None,
            );
            self.finish(
                intent.command_id,
                intent.actor_id,
                true,
                crate::model::spoken_act(None, "sew", rest),
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
            let in_pot = !self.loose_ids(actor.id, Material::Pot).is_empty();
            let coda = if self.thirst_word(before) == "not thirsty" {
                ""
            } else if self.thirst_word(after) == "not thirsty" {
                " The thirst eases."
            } else {
                " You are still thirsty."
            };
            let heard = format!(
                "{}{coda}",
                crate::model::spoken_act(
                    Some(actor.name.as_str()),
                    "boil",
                    boil_witness_rest(in_pot)
                )
            );
            self.emit(
                heard,
                Scope::Cell(actor.cell),
                vec![actor.id],
                vec![actor.cell],
                vec![item_id],
                fire_event,
                None,
            );
            let message = format!(
                "{}{coda}",
                crate::model::spoken_act(None, "boil", boil_actor_rest(in_pot))
            );
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
            if material == Material::Meat {
                self.eat_meat(intent, results);
                continue;
            }
            if material == Material::Mushroom {
                self.eat_mushroom(intent, results);
                continue;
            }
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

    fn eat_mushroom(&mut self, intent: Intent, results: &mut Vec<CommandResult>) {
        let Some(id) = self.cooked_mushroom_id(intent.actor_id) else {
            let message = if self.raw_mushroom_near(intent.actor_id) {
                "Raw mushrooms do not feed you."
            } else {
                "There is no cooked mushroom here to eat."
            };
            self.finish(intent.command_id, intent.actor_id, false, message, results);
            return;
        };
        let before = self
            .actor(intent.actor_id)
            .map(|actor| actor.hunger)
            .unwrap_or(0);
        let after = before.saturating_sub(self.tuning.cook_relief);
        if let Some(person) = self.actors.get_mut(&intent.actor_id) {
            person.hunger = after;
        }
        self.items.remove(&id);
        let cell = self
            .actor(intent.actor_id)
            .map(|actor| actor.cell)
            .unwrap_or(0);
        let name = self.name_of(intent.actor_id);
        let rest = "the cooked mushroom.";
        self.emit(
            crate::model::spoken_act(Some(name.as_str()), "eat", rest),
            Scope::Cell(cell),
            vec![intent.actor_id],
            vec![cell],
            vec![id],
            vec![],
            None,
        );
        let eased = self.hunger_word(before) != "not hungry"
            && self.hunger_word(after) != self.hunger_word(before);
        let mut message = crate::model::spoken_act(None, "eat", rest);
        if eased {
            message.push_str(" The hunger eases.");
            self.emit(
                "The hunger eases.",
                Scope::Personal(intent.actor_id),
                vec![intent.actor_id],
                vec![cell],
                vec![],
                vec![],
                None,
            );
        } else {
            message.push_str(&format!(" You are {}.", self.hunger_word(after)));
        }
        self.finish(intent.command_id, intent.actor_id, true, message, results);
    }

    fn eat_meat(&mut self, intent: Intent, results: &mut Vec<CommandResult>) {
        let Some(id) = self.cooked_meat_id(intent.actor_id) else {
            let message = if self.raw_meat_near(intent.actor_id) {
                "Raw meat is not food."
            } else {
                "There is no cooked meat here to eat."
            };
            self.finish(intent.command_id, intent.actor_id, false, message, results);
            return;
        };
        let before = self
            .actor(intent.actor_id)
            .map(|actor| actor.hunger)
            .unwrap_or(0);
        let relief = if self.item(id).is_some_and(|meat| meat.small) {
            self.tuning.cook_relief / 2
        } else {
            self.tuning.cook_relief
        };
        let after = before.saturating_sub(relief);
        if let Some(person) = self.actors.get_mut(&intent.actor_id) {
            person.hunger = after;
        }
        self.items.remove(&id);
        let cell = self
            .actor(intent.actor_id)
            .map(|actor| actor.cell)
            .unwrap_or(0);
        let name = self.name_of(intent.actor_id);
        let rest = "the cooked meat.";
        self.emit(
            crate::model::spoken_act(Some(name.as_str()), "eat", rest),
            Scope::Cell(cell),
            vec![intent.actor_id],
            vec![cell],
            vec![id],
            vec![],
            None,
        );
        let eased = self.hunger_word(before) != "not hungry"
            && self.hunger_word(after) != self.hunger_word(before);
        let mut message = crate::model::spoken_act(None, "eat", rest);
        if eased {
            message.push_str(" The hunger eases.");
            self.emit(
                "The hunger eases.",
                Scope::Personal(intent.actor_id),
                vec![intent.actor_id],
                vec![cell],
                vec![],
                vec![],
                None,
            );
        } else {
            message.push_str(&format!(" You are {}.", self.hunger_word(after)));
        }
        self.finish(intent.command_id, intent.actor_id, true, message, results);
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

    fn apply_ochre_smears(&mut self, smears: Vec<Intent>, results: &mut Vec<CommandResult>) {
        for intent in smears {
            let Some(item_id) = self
                .loose_ids(intent.actor_id, Material::Ochre)
                .first()
                .copied()
            else {
                self.finish(
                    intent.command_id,
                    intent.actor_id,
                    false,
                    "You have no ochre to smear.",
                    results,
                );
                continue;
            };
            self.items.remove(&item_id);
            if let Some(person) = self.actors.get_mut(&intent.actor_id) {
                person.ochre = 1000;
            }
            let cell = self
                .actor(intent.actor_id)
                .map(|actor| actor.cell)
                .unwrap_or(0);
            let name = self.name_of(intent.actor_id);
            self.emit(
                format!("{name} smears red ochre on their skin."),
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
                "You smear red ochre on your skin.",
                results,
            );
        }
    }

    fn apply_marks(&mut self, marks: Vec<Intent>, results: &mut Vec<CommandResult>) {
        for intent in marks {
            let cell = self
                .actor(intent.actor_id)
                .map(|actor| actor.cell)
                .unwrap_or(0);
            if matches!(intent.attempt, Attempt::MarkRock) && !self.cell(cell).rock {
                self.finish(
                    intent.command_id,
                    intent.actor_id,
                    false,
                    "There is no rock here to mark.",
                    results,
                );
                continue;
            }
            if self.ochre_marks.contains(&cell) {
                self.finish(
                    intent.command_id,
                    intent.actor_id,
                    false,
                    "A red mark is already here.",
                    results,
                );
                continue;
            }
            let Some(item_id) = self
                .loose_ids(intent.actor_id, Material::Ochre)
                .first()
                .copied()
            else {
                self.finish(
                    intent.command_id,
                    intent.actor_id,
                    false,
                    "You have no ochre to mark with.",
                    results,
                );
                continue;
            };
            self.items.remove(&item_id);
            self.ochre_marks.insert(cell);
            let rock = self.cell(cell).rock;
            let surface = if rock { "rock" } else { "ground" };
            let name = self.name_of(intent.actor_id);
            let rest = format!("the {surface} with red ochre.");
            self.emit(
                crate::model::spoken_act(Some(name.as_str()), "mark", &rest),
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
                crate::model::spoken_act(None, "mark", &rest),
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

/// The witness rest of one boil. The actor's rest names the same fire and
/// says "drink".
fn boil_witness_rest(in_pot: bool) -> &'static str {
    if in_pot {
        "the drink in the pot over the fire and drinks it."
    } else {
        "a drink of water over the fire and drinks it."
    }
}

fn boil_actor_rest(in_pot: bool) -> &'static str {
    if in_pot {
        "the drink in the pot over the fire and drink it."
    } else {
        "a drink of water over the fire and drink it."
    }
}

/// One light, filled for the person who set it and for anyone else.
fn light_line(who: Option<&str>, bark_arms: bool, branch_arms: bool) -> String {
    let fuel = match (bark_arms, branch_arms) {
        (true, true) => "the bark and the branch from the arms",
        (false, false) => "bark and a dry branch from the pile at the feet",
        (true, false) => "the bark from the arms and a dry branch from the pile at the feet",
        (false, true) => "bark from the pile at the feet and the branch from the arms",
    };
    crate::model::spoken_act(who, "light", &format!("a fire with {fuel}."))
}
