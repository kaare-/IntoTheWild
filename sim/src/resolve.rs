//! Attempts in one window read the same starting state. Packet order is
//! discarded: ties break by actor id, then command id, and equal claims
//! break by a recorded hash.

use crate::model::{
    cell_id, cell_xy, contest_hash, Attempt, CommandResult, Dir, Draw, Intent, Material, Region,
    Role, Scope, Wake, WorkKind, World,
};
use std::collections::{BTreeMap, BTreeSet};

enum WetBranch {
    Ready,
    Already,
    Dry,
    None,
}

/// The raw mushroom refusal. Reply only. The same two sentences as the look
/// after this person has tried them.
const RAW_MUSHROOM_REFUSAL: &str = "Raw mushrooms do not feed you. They must be cooked.";
const BARK_REFUSAL: &str = "Bark does not feed you.";

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
        let mut hide_cuts = Vec::new();
        let mut shapes = Vec::new();
        let mut firings = Vec::new();
        let mut lean_tos = Vec::new();
        let mut thatches = Vec::new();
        let mut wears = Vec::new();
        let mut removes = Vec::new();
        let mut rests = Vec::new();
        let mut dries = Vec::new();
        let mut fishes = Vec::new();
        let mut brands = Vec::new();
        let mut baskets = Vec::new();
        let mut nets = Vec::new();
        let mut mats = Vec::new();
        let mut puts = Vec::new();
        let mut snares = Vec::new();
        let mut set_nets = Vec::new();
        let mut set_mats = Vec::new();
        let mut hollows = Vec::new();
        let mut carves_canoe = Vec::new();
        let mut launches = Vec::new();
        let mut seals = Vec::new();
        let mut boards = Vec::new();
        let mut paddles = Vec::new();
        let mut disembarks = Vec::new();
        let mut beaches = Vec::new();
        let mut carves = Vec::new();
        let mut paddle_carves = Vec::new();
        let mut hook_carves = Vec::new();
        let mut salts = Vec::new();
        let mut salt_fish = Vec::new();
        let mut smoke_fish = Vec::new();
        let mut smoke_meat = Vec::new();
        let mut smokes = Vec::new();
        let mut dresses = Vec::new();
        let mut bows = Vec::new();
        let mut clothes = Vec::new();

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
                Err(message) => {
                    // A smoke refusal is the reply only, and so is a raw
                    // mushroom, and so is a fish-with-hook refusal. "There
                    // is no fire burning here" is also a feed, cook, and
                    // dry refusal, and those still sit in the actor's
                    // event list. A spear or a net that says the brook is
                    // not here, or that the fish are gone, still does too.
                    let hook_refusal = matches!(intent.attempt, Attempt::FishHook)
                        && matches!(
                            message.as_str(),
                            "The brook is not here."
                                | "You are not carrying a bone fish hook."
                                | "You have no hide strips for a line."
                                | "The fish are gone from this stretch of the brook."
                        );
                    // A light refusal is the reply only. Bark, resin, rain,
                    // and a fire already burning stay with the person who tried.
                    let light_refusal = matches!(intent.attempt, Attempt::Light);
                    let personal =
                        !matches!(intent.attempt, Attempt::SmokeFish | Attempt::SmokeMeat)
                            && message != RAW_MUSHROOM_REFUSAL
                            && message != BARK_REFUSAL
                            && !hook_refusal
                            && !light_refusal;
                    failures.push((intent, message, personal));
                }
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
                        Attempt::CutHide => hide_cuts.push(intent),
                        Attempt::Shape => shapes.push(intent),
                        Attempt::FirePot => firings.push(intent),
                        Attempt::BuildLeanTo => lean_tos.push(intent),
                        Attempt::Thatch => thatches.push(intent),
                        Attempt::Wear => wears.push(intent),
                        Attempt::Remove => removes.push(intent),
                        Attempt::Rest => rests.push(intent),
                        Attempt::DryBranch => dries.push(intent),
                        Attempt::Fish | Attempt::FishHook => fishes.push(intent),
                        Attempt::TakeBrand => brands.push(intent),
                        Attempt::WeaveBasket => baskets.push(intent),
                        Attempt::WeaveNet => nets.push(intent),
                        Attempt::WeaveMat => mats.push(intent),
                        Attempt::SetMat => set_mats.push(intent),
                        Attempt::HollowLog => hollows.push(intent),
                        Attempt::CarveCanoe => carves_canoe.push(intent),
                        Attempt::Launch => launches.push(intent),
                        Attempt::SealCanoe => seals.push(intent),
                        Attempt::Board => boards.push(intent),
                        Attempt::Paddle(_) => paddles.push(intent),
                        Attempt::Disembark => disembarks.push(intent),
                        Attempt::Beach => beaches.push(intent),
                        Attempt::PutFish => puts.push(intent),
                        Attempt::SetSnare => snares.push(intent),
                        Attempt::SetNet => set_nets.push(intent),
                        Attempt::CarveAwl => carves.push(intent),
                        Attempt::CarvePaddle => paddle_carves.push(intent),
                        Attempt::CarveHook => hook_carves.push(intent),
                        Attempt::SaltMeat => salts.push(intent),
                        Attempt::SaltFish => salt_fish.push(intent),
                        Attempt::SmokeFish => smoke_fish.push(intent),
                        Attempt::SmokeMeat => smoke_meat.push(intent),
                        Attempt::SmokeNest => smokes.push(intent),
                        Attempt::DressSting => dresses.push(intent),
                        Attempt::CraftBow => bows.push(intent),
                        Attempt::DryClothes => clothes.push(intent),
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
            if message == RAW_MUSHROOM_REFUSAL {
                if let Some(person) = self.actors.get_mut(&intent.actor_id) {
                    person.raw_mushroom_known = true;
                }
            }
            if message == BARK_REFUSAL {
                if let Some(person) = self.actors.get_mut(&intent.actor_id) {
                    person.bark_known = true;
                }
            }
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
        self.apply_hide_cuts(hide_cuts, &mut results);
        self.apply_shapes(shapes, &mut results);
        self.apply_pot_firings(firings, &mut results);
        self.apply_lean_tos(lean_tos, &mut results);
        self.apply_thatches(thatches, &mut results);
        self.apply_wears(wears, &mut results);
        self.apply_removes(removes, &mut results);
        self.apply_rests(rests, &mut results);
        self.apply_dries(dries, &mut results);
        self.apply_fishes(fishes, &mut results);
        self.apply_brands(brands, &mut results);
        self.apply_baskets(baskets, &mut results);
        self.apply_nets(nets, &mut results);
        self.apply_mats(mats, &mut results);
        self.apply_set_mats(set_mats, &mut results);
        self.apply_hollows(hollows, &mut results);
        self.apply_canoe_carves(carves_canoe, &mut results);
        self.apply_launches(launches, &mut results);
        self.apply_seals(seals, &mut results);
        self.apply_boards(boards, &mut results);
        self.apply_paddles(paddles, &mut results);
        self.apply_disembarks(disembarks, &mut results);
        self.apply_beaches(beaches, &mut results);
        self.apply_puts(puts, &mut results);
        self.apply_snares(snares, &mut results);
        self.apply_set_nets(set_nets, &mut results);
        self.apply_carves(carves, &mut results);
        self.apply_paddle_carves(paddle_carves, &mut results);
        self.apply_hook_carves(hook_carves, &mut results);
        self.apply_salts(salts, &mut results);
        self.apply_salt_fish(salt_fish, &mut results);
        self.apply_smoke_fish(smoke_fish, &mut results);
        self.apply_smoke_meat(smoke_meat, &mut results);
        self.apply_smokes(smokes, &mut results);
        self.apply_dresses(dresses, &mut results);
        self.apply_bows(bows, &mut results);
        self.apply_dry_clothes(clothes, &mut results);
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
            Attempt::TakeBrand => {
                if busy {
                    Err("You are in the middle of something.".into())
                } else if body_taken {
                    Err("You already committed to something else this window.".into())
                } else if gripping {
                    Err("Your hands are on the log.".into())
                } else if self.fire_at(actor.cell).is_none() {
                    Err("There is no fire burning here.".into())
                } else if !self.loose_ids(actor.id, Material::Brand).is_empty() {
                    Err("You are already carrying a burning brand.".into())
                } else if self.carry_count(actor.id) >= self.tuning.carry_slots {
                    Err("Your arms are full.".into())
                } else {
                    Ok(true)
                }
            }
            Attempt::SmokeNest => {
                if busy {
                    Err("You are in the middle of something.".into())
                } else if body_taken {
                    Err("You already committed to something else this window.".into())
                } else if gripping {
                    Err("Your hands are on the log.".into())
                } else if !self.honey_nests.contains(&actor.cell) {
                    Err("There is no bees' nest here to smoke.".into())
                } else if self.pile_count(actor.cell, Material::Honey) == 0
                    && self.ground_ids(actor.cell, Material::Honey).is_empty()
                {
                    Err("The bees' nest is empty.".into())
                } else if self.smoked_nests.contains(&actor.cell) {
                    Err("Smoke already hangs in the bees' nest.".into())
                } else if !self.smoke_reaches(actor.id, actor.cell) {
                    Err("You have no smoke to quiet the bees.".into())
                } else {
                    Ok(true)
                }
            }
            Attempt::Fish => {
                if busy {
                    Err("You are in the middle of something.".into())
                } else if body_taken {
                    Err("You already committed to something else this window.".into())
                } else if gripping {
                    Err("Your hands are on the log.".into())
                } else if !self.fishing_water(actor.cell) {
                    Err("The brook is not here.".into())
                } else if let Some(message) = self.fish_tool_refusal(actor.id) {
                    Err(message.into())
                } else if self
                    .brook_stretch(actor.cell)
                    .is_some_and(|stretch| self.brook_is_quiet(stretch))
                {
                    Err("The fish are gone from this stretch of the brook.".into())
                } else {
                    Ok(true)
                }
            }
            Attempt::FishHook => {
                if busy {
                    Err("You are in the middle of something.".into())
                } else if body_taken {
                    Err("You already committed to something else this window.".into())
                } else if gripping {
                    Err("Your hands are on the log.".into())
                } else if !self.fishing_water(actor.cell) {
                    Err("The brook is not here.".into())
                } else if self.loose_ids(actor.id, Material::Hook).is_empty() {
                    Err("You are not carrying a bone fish hook.".into())
                } else if self
                    .brook_stretch(actor.cell)
                    .is_some_and(|stretch| self.brook_is_quiet(stretch))
                {
                    Err("The fish are gone from this stretch of the brook.".into())
                } else if self.fishing_line(actor.id).is_none() {
                    Err("You have no hide strips for a line.".into())
                } else {
                    Ok(true)
                }
            }
            Attempt::DryBranch => {
                if busy {
                    Err("You are in the middle of something.".into())
                } else if body_taken {
                    Err("You already committed to something else this window.".into())
                } else if gripping {
                    Err("Your hands are on the log.".into())
                } else if self.fire_at(actor.cell).is_none() {
                    Err("There is no fire burning here.".into())
                } else {
                    match self.wet_branch_choice(actor.id) {
                        WetBranch::Ready => Ok(true),
                        WetBranch::Already => {
                            Err("The branch is already drying by the fire.".into())
                        }
                        WetBranch::Dry => Err("The branch is already dry.".into()),
                        WetBranch::None => Err("You have no branch to dry.".into()),
                    }
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
                    Err(if self.carrying_wolf_hide(actor.id) {
                        "You are not wearing a wolf hide."
                    } else {
                        "You are not wearing a deer hide."
                    }
                    .into())
                } else {
                    Ok(true)
                }
            }
            Attempt::BuildLeanTo => {
                if busy {
                    Err("You are in the middle of something.".into())
                } else if body_taken {
                    Err("You already committed to something else this window.".into())
                } else if gripping {
                    Err("Your hands are on the log.".into())
                } else if self.lean_tos.contains(&actor.cell) {
                    Err("A lean-to already stands here.".into())
                } else if self.dry_branches_for_lean_to(actor.id) < 3 {
                    Err("You need three dry branches to build a lean-to.".into())
                } else {
                    Ok(true)
                }
            }
            Attempt::SaltMeat => {
                if busy {
                    Err("You are in the middle of something.".into())
                } else if body_taken {
                    Err("You already committed to something else this window.".into())
                } else if gripping {
                    Err("Your hands are on the log.".into())
                } else if !self.has_salt(actor.id) {
                    Err("You have no salt.".into())
                } else if self.raw_portion_to_salt(actor.id).is_none() {
                    if self
                        .loose_ids(actor.id, Material::Meat)
                        .iter()
                        .chain(self.ground_ids(actor.cell, Material::Meat).iter())
                        .any(|id| {
                            self.item(*id)
                                .is_some_and(|meat| meat.salted && !meat.spoiled)
                        })
                    {
                        Err("The meat is already salted.".into())
                    } else if self.spoiled_meat_near(actor.id) {
                        Err("The meat has spoiled.".into())
                    } else {
                        Err("There is no raw meat here to salt.".into())
                    }
                } else {
                    Ok(true)
                }
            }
            Attempt::SaltFish => {
                if busy {
                    Err("You are in the middle of something.".into())
                } else if body_taken {
                    Err("You already committed to something else this window.".into())
                } else if gripping {
                    Err("Your hands are on the log.".into())
                } else if !self.has_salt(actor.id) {
                    Err("You have no salt.".into())
                } else if self.raw_fish_id(actor.id).is_none() {
                    if self.salted_fish_id(actor.id).is_some() {
                        Err("The fish is already salted.".into())
                    } else if self.spoiled_fish_near(actor.id) {
                        Err("The fish has spoiled.".into())
                    } else {
                        Err("There is no raw fish here to salt.".into())
                    }
                } else {
                    Ok(true)
                }
            }
            Attempt::SmokeFish => {
                if busy {
                    Err("You are in the middle of something.".into())
                } else if body_taken {
                    Err("You already committed to something else this window.".into())
                } else if gripping {
                    Err("Your hands are on the log.".into())
                } else if self.fire_at(actor.cell).is_none() {
                    Err("There is no fire burning here.".into())
                } else if self.raw_fish_id(actor.id).is_none() {
                    if self.smoked_fish_id(actor.id).is_some() {
                        Err("The fish is already smoked.".into())
                    } else if self.spoiled_fish_near(actor.id) {
                        Err("The fish has spoiled.".into())
                    } else {
                        Err("There is no raw fish here to smoke.".into())
                    }
                } else {
                    Ok(true)
                }
            }
            Attempt::SmokeMeat => {
                if busy {
                    Err("You are in the middle of something.".into())
                } else if body_taken {
                    Err("You already committed to something else this window.".into())
                } else if gripping {
                    Err("Your hands are on the log.".into())
                } else if self.fire_at(actor.cell).is_none() {
                    Err("There is no fire burning here.".into())
                } else if self.raw_portion_to_salt(actor.id).is_none() {
                    if self.smoked_meat_id(actor.id).is_some() {
                        Err("The meat is already smoked.".into())
                    } else if self.spoiled_meat_near(actor.id) {
                        Err("The meat has spoiled.".into())
                    } else {
                        Err("There is no raw meat here to smoke.".into())
                    }
                } else {
                    Ok(true)
                }
            }
            Attempt::CarveAwl => {
                if busy {
                    Err("You are in the middle of something.".into())
                } else if body_taken {
                    Err("You already committed to something else this window.".into())
                } else if gripping {
                    Err("Your hands are on the log.".into())
                } else if !self.has_bone(actor.id) {
                    Err("You have no bone to carve.".into())
                } else if !self.has_carve_tool(actor.id) {
                    Err("You need a flint flake or an iron knife to carve the bone.".into())
                } else {
                    Ok(true)
                }
            }
            Attempt::CarvePaddle => {
                if busy {
                    Err("You are in the middle of something.".into())
                } else if body_taken {
                    Err("You already committed to something else this window.".into())
                } else if gripping {
                    Err("Your hands are on the log.".into())
                } else if let Some(message) = self.paddle_branch_refusal(actor.id) {
                    Err(message.into())
                } else if self.paddle_tool_rest(actor.id).is_none() {
                    Err("You need a flint flake or an iron knife to carve a paddle.".into())
                } else {
                    Ok(true)
                }
            }
            Attempt::CarveHook => {
                if busy {
                    Err("You are in the middle of something.".into())
                } else if body_taken {
                    Err("You already committed to something else this window.".into())
                } else if gripping {
                    Err("Your hands are on the log.".into())
                } else if !self.has_bone(actor.id) {
                    Err("You have no bone to carve.".into())
                } else if self.hook_tool_rest(actor.id).is_none() {
                    Err("You need a flint flake or an iron knife to carve a hook.".into())
                } else {
                    Ok(true)
                }
            }
            Attempt::SetNet => {
                if busy {
                    Err("You are in the middle of something.".into())
                } else if body_taken {
                    Err("You already committed to something else this window.".into())
                } else if gripping {
                    Err("Your hands are on the log.".into())
                } else if self.loose_ids(actor.id, Material::Net).is_empty() {
                    Err("You are not carrying a fishing net.".into())
                } else if self.cell(actor.cell).region != Region::Brook {
                    Err("You cannot set a net out of the brook.".into())
                } else if self.set_nets.contains(&actor.cell) {
                    Err("A fishing net is already set here.".into())
                } else {
                    Ok(true)
                }
            }
            Attempt::SetSnare => {
                if busy {
                    Err("You are in the middle of something.".into())
                } else if body_taken {
                    Err("You already committed to something else this window.".into())
                } else if gripping {
                    Err("Your hands are on the log.".into())
                } else if self.snares.contains_key(&actor.cell) {
                    Err("A snare is already set here.".into())
                } else if !self.has_snare_line(actor.id) {
                    Err("You have no hide strips to set a snare.".into())
                } else {
                    Ok(true)
                }
            }
            Attempt::Thatch => {
                if busy {
                    Err("You are in the middle of something.".into())
                } else if body_taken {
                    Err("You already committed to something else this window.".into())
                } else if gripping {
                    Err("Your hands are on the log.".into())
                } else if !self.lean_tos.contains(&actor.cell) {
                    Err("There is no lean-to here to thatch.".into())
                } else if self.thatched.contains(&actor.cell) {
                    Err("The lean-to is already thatched.".into())
                } else if !self.has_reeds(actor.id) {
                    Err("You have no reeds to thatch with.".into())
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
            Attempt::CutHide => {
                if busy {
                    Err("You are in the middle of something.".into())
                } else if body_taken {
                    Err("You already committed to something else this window.".into())
                } else if gripping {
                    Err("Your hands are on the log.".into())
                } else if self.loose_ids(actor.id, Material::Knife).is_empty() {
                    Err("You need an iron knife to cut the hide.".into())
                } else {
                    match self.whole_hide_id(actor.id) {
                        Some(_) => Ok(true),
                        None if self.hide_near(actor.id) => {
                            Err("You have no whole deer hide to cut.".into())
                        }
                        None => Err("You have no deer hide to cut.".into()),
                    }
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
                } else if self.spear_can_hunt(actor.id) {
                    let wolf = self
                        .wolf_standing()
                        .map(|cell| crate::model::chebyshev(actor.cell, cell));
                    let deer = self
                        .deer
                        .as_ref()
                        .map(|deer| crate::model::chebyshev(actor.cell, deer.cell));
                    let wolf_melee = wolf.is_some_and(|dist| dist <= 1);
                    let wolf_shot = self.bow_in_hand(actor.id)
                        && wolf.is_some_and(|dist| dist <= self.tuning.sight);
                    if wolf_melee
                        || deer.is_some_and(|dist| dist <= 1)
                        || wolf_shot
                        || self.bow_can_reach(actor.id)
                    {
                        Ok(true)
                    } else if wolf.is_some_and(|dist| dist <= self.tuning.sight)
                        && wolf.is_some_and(|dist| deer.is_none_or(|far| dist < far))
                    {
                        Err("The wolf is not close enough to hunt.".into())
                    } else if self.deer.as_ref().is_none() {
                        Err("There is no deer to hunt.".into())
                    } else {
                        Err("The deer is not close enough to hunt.".into())
                    }
                } else if !self.loose_ids(actor.id, Material::Spear).is_empty() {
                    if self
                        .loose_ids(actor.id, Material::Spear)
                        .iter()
                        .any(|id| self.item(*id).is_some_and(|spear| spear.flint_tip))
                    {
                        Err("The flint tip is not fire-hardened.".into())
                    } else {
                        Err("The spear's point is not hardened.".into())
                    }
                } else if self.loose_ids(actor.id, Material::Bow).is_empty() {
                    Err("You are not carrying a spear.".into())
                } else if self.loose_ids(actor.id, Material::Flake).is_empty() {
                    Err("You need a sharp flint flake.".into())
                } else {
                    let wolf = self
                        .wolf_standing()
                        .map(|cell| crate::model::chebyshev(actor.cell, cell));
                    let deer = self
                        .deer
                        .as_ref()
                        .map(|deer| crate::model::chebyshev(actor.cell, deer.cell));
                    let wolf_shot = wolf.is_some_and(|dist| dist <= self.tuning.sight)
                        && wolf.is_some_and(|dist| deer.is_none_or(|far| dist < far));
                    let deer_shot = deer.is_some_and(|dist| dist <= self.tuning.sight);
                    if wolf_shot || deer_shot {
                        Ok(true)
                    } else if self.deer.as_ref().is_none() {
                        Err("There is no deer to hunt.".into())
                    } else {
                        Err("The deer is not close enough for the bow.".into())
                    }
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
                } else if !self.loose_ids(actor.id, Material::Charcoal).is_empty()
                    || self.has_dry(actor.id, Material::Branch)
                {
                    Ok(true)
                } else {
                    Err("You have nothing to feed the fire.".into())
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
                    Err(if self.carrying_wolf_hide(actor.id) {
                        "The wolf hide is already pierced."
                    } else {
                        "The deer hide is already pierced."
                    }
                    .into())
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
                } else if self.exposure_millis(actor.cell) >= 1000
                    && !self.reed_mat_here(actor.cell)
                {
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
                    Err(if self.carrying_wolf_hide(actor.id) {
                        "The wolf hide is already a cloak."
                    } else {
                        "The deer hide is already a cloak."
                    }
                    .into())
                } else {
                    Err(if self.carrying_wolf_hide(actor.id) {
                        "The wolf hide is not pierced."
                    } else {
                        "The deer hide is not pierced."
                    }
                    .into())
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
                                Err(RAW_MUSHROOM_REFUSAL.into())
                            } else {
                                Err("There is no cooked mushroom here to eat.".into())
                            }
                        }
                        Material::BitterBerry => {
                            if self.has_material_at_hand(actor.id, Material::BitterBerry) {
                                Ok(true)
                            } else {
                                Err(format!(
                                    "There are no {} here to eat.",
                                    self.bitter_kind(actor.id)
                                ))
                            }
                        }
                        Material::PaleBerry => {
                            if self.cooked_pale_id(actor.id).is_some()
                                || self.raw_pale_near(actor.id)
                            {
                                Ok(true)
                            } else {
                                Err("There are no pale berries here to eat.".into())
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
                        Material::Fish => {
                            if self.cooked_fish_id(actor.id).is_some()
                                || self.salted_fish_id(actor.id).is_some()
                                || self.smoked_fish_id(actor.id).is_some()
                            {
                                Ok(true)
                            } else if self.raw_fish_near(actor.id) {
                                Err("Raw fish is not food.".into())
                            } else if self.spoiled_fish_near(actor.id) {
                                Err("The fish has spoiled.".into())
                            } else {
                                Err("There is no cooked fish here to eat.".into())
                            }
                        }
                        Material::Meat => {
                            if self.cooked_meat_id(actor.id).is_some()
                                || self.salted_meat_id(actor.id).is_some()
                                || self.smoked_meat_id(actor.id).is_some()
                            {
                                Ok(true)
                            } else if self.raw_meat_near(actor.id) {
                                Err("Raw meat is not food.".into())
                            } else if self.spoiled_meat_near(actor.id) {
                                Err("The meat has spoiled.".into())
                            } else {
                                Err("There is no cooked meat here to eat.".into())
                            }
                        }
                        Material::Clay => Err("Unfired clay is not food.".into()),
                        Material::Bowl => Err("A bowl is not food.".into()),
                        Material::Bark => {
                            let near_bark = self.has_material_at_hand(actor.id, Material::Bark)
                                || !self.ground_ids(actor.cell, Material::Bark).is_empty();
                            if near_bark {
                                Err(BARK_REFUSAL.into())
                            } else {
                                Err("There is no bark here to eat.".into())
                            }
                        }
                        Material::Ochre => Err("Ochre is not food.".into()),
                        Material::CopperOre => Err("Copper ore is not food.".into()),
                        Material::IronOre => Err("Iron ore is not food.".into()),
                        Material::Iron => Err("Iron is not food.".into()),
                        Material::Knife => Err("An iron knife is not food.".into()),
                        Material::Copper => Err("Copper is not food.".into()),
                        Material::Awl => Err("A copper awl is not food.".into()),
                        Material::Bone => Err("A bone is not food.".into()),
                        Material::Salt => Err("Salt is not food.".into()),
                        Material::Honey => {
                            if self.loose_ids(actor.id, Material::Honey).is_empty() {
                                Err("You are not carrying honey.".into())
                            } else {
                                Ok(true)
                            }
                        }
                        Material::Egg => {
                            if self.cooked_egg_id(actor.id).is_some() {
                                Ok(true)
                            } else if self.raw_egg_near(actor.id) {
                                Err("The eggs are raw.".into())
                            } else {
                                Err("There are no cooked eggs here to eat.".into())
                            }
                        }
                        Material::Moss => Err("Moss is for a sting, not food.".into()),
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
            Attempt::CraftBow => {
                if busy {
                    Err("You are in the middle of something.".into())
                } else if body_taken {
                    Err("You already committed to something else this window.".into())
                } else if gripping {
                    Err("Your hands are on the log.".into())
                } else if let Some(message) = self.bow_refusal(actor.id) {
                    Err(message.into())
                } else {
                    Ok(true)
                }
            }
            Attempt::DryClothes => {
                if busy {
                    Err("You are in the middle of something.".into())
                } else if body_taken {
                    Err("You already committed to something else this window.".into())
                } else if gripping {
                    Err("Your hands are on the log.".into())
                } else if self.fire_at(actor.cell).is_none() {
                    Err("There is no fire burning here.".into())
                } else if actor.wetness == 0 {
                    Err("Your clothes are already dry.".into())
                } else if actor.clothes_dry_at.is_some() {
                    Err("Your clothes are already drying by the fire.".into())
                } else {
                    Ok(true)
                }
            }
            Attempt::DressSting => {
                if busy {
                    Err("You are in the middle of something.".into())
                } else if body_taken {
                    Err("You already committed to something else this window.".into())
                } else if gripping {
                    Err("Your hands are on the log.".into())
                } else if self.loose_ids(actor.id, Material::Moss).is_empty() {
                    Err("You have no moss.".into())
                } else if actor.stings_until.is_none() {
                    Err("You are not stung.".into())
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
                } else if self.charcoal_marks.contains(&actor.cell) {
                    Err("A black mark is already here.".into())
                } else if !self.loose_ids(actor.id, Material::Charcoal).is_empty()
                    || !self.loose_ids(actor.id, Material::Ochre).is_empty()
                {
                    Ok(true)
                } else {
                    Err("You have nothing to mark with.".into())
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
                if self
                    .aboard
                    .get(&actor.id)
                    .is_some_and(|id| self.canoe_floats(*id))
                {
                    return Err("You are aboard the canoe. Disembark first.".into());
                }
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
                } else if *material == Material::Egg
                    && self.pile_count(actor.cell, Material::Egg) == 0
                    && self.ground_ids(actor.cell, Material::Egg).is_empty()
                {
                    return Err("There are no eggs here to gather.".into());
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
                            | Material::Cord
                            | Material::Fish
                            | Material::Brand
                            | Material::Charcoal
                            | Material::Honey
                            | Material::Egg
                            | Material::Moss
                            | Material::Bow
                            | Material::Net
                            | Material::Mat
                            | Material::Paddle
                            | Material::Hook
                            | Material::Resin
                            | Material::Willow
                    ) || self.ground_ids(actor.cell, *material).is_empty())
                {
                    return Err(if *material == Material::BitterBerry {
                        format!(
                            "There are no {} here to gather.",
                            self.bitter_kind(actor.id)
                        )
                    } else {
                        format!("There is no {} here to gather.", material.word())
                    });
                }
                if self.carry_count(actor.id) >= self.tuning.carry_slots
                    && !self.gather_overflows_into_basket(actor.id, *material)
                {
                    return Err("Your arms are full.".into());
                }
                Ok(true)
            }
            Attempt::Drop(material) => {
                if self.loose_ids(actor.id, *material).is_empty() {
                    Err(if *material == Material::BitterBerry {
                        format!("You are not carrying loose {}.", self.bitter_kind(actor.id))
                    } else {
                        format!("You are not carrying a loose {}.", material.word())
                    })
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
                if self.gripping(actor.id).is_none() && self.canoe_at(actor.cell).is_some() {
                    let Some(dir) = attempt_dir(attempt) else {
                        return Err("Name a direction.".into());
                    };
                    if self
                        .canoe_at(actor.cell)
                        .is_some_and(|id| self.canoe_floats(id))
                    {
                        return Err("The canoe is already in the brook.".into());
                    }
                    let (x, y) = cell_xy(actor.cell);
                    let (dx, dy) = dir.delta();
                    let Some(dest) = cell_id(x + dx, y + dy) else {
                        return Err("There is nowhere to drag the canoe.".into());
                    };
                    if self.cell(dest).region == Region::Brook {
                        return Err("Launch the canoe from the bank.".into());
                    }
                    return Ok(true);
                }
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
            Attempt::WeaveBasket => {
                if gripping {
                    return Err("Your hands are on the log.".into());
                }
                if !self.loose_ids(actor.id, Material::Basket).is_empty() {
                    return Err("You are already carrying a bark basket.".into());
                }
                let carried_bark = !self.loose_ids(actor.id, Material::Bark).is_empty();
                let piled_bark = self.pile_count(actor.cell, Material::Bark) > 0;
                if !carried_bark && !piled_bark {
                    return Err("You need a sheet of bark to weave a basket.".into());
                }
                if !carried_bark && self.carry_count(actor.id) >= self.tuning.carry_slots {
                    return Err("Your arms are full.".into());
                }
                Ok(true)
            }
            Attempt::WeaveNet => {
                if gripping {
                    return Err("Your hands are on the log.".into());
                }
                if !self.loose_ids(actor.id, Material::Net).is_empty() {
                    return Err("You are already carrying a fishing net.".into());
                }
                let carried = self.loose_ids(actor.id, Material::Reed).len() as u32;
                let piled = self.pile_count(actor.cell, Material::Reed);
                if carried + piled < 3 {
                    return Err("You need three bundles of reeds to weave a net.".into());
                }
                let spent = carried.min(3);
                if self.carry_count(actor.id) - spent + 1 > self.tuning.carry_slots {
                    return Err("Your arms are full.".into());
                }
                Ok(true)
            }
            Attempt::WeaveMat => {
                if gripping {
                    return Err("Your hands are on the log.".into());
                }
                if !self.loose_ids(actor.id, Material::Mat).is_empty() {
                    return Err("You are already carrying a reed mat.".into());
                }
                let carried = self.loose_ids(actor.id, Material::Reed).len() as u32;
                let piled = self.pile_count(actor.cell, Material::Reed);
                if carried + piled < 2 {
                    return Err("You need two bundles of reeds to weave a mat.".into());
                }
                let spent = carried.min(2);
                if self.carry_count(actor.id) - spent + 1 > self.tuning.carry_slots {
                    return Err("Your arms are full.".into());
                }
                Ok(true)
            }
            Attempt::SetMat => {
                if busy {
                    return Err("You are in the middle of something.".into());
                }
                if body_taken {
                    return Err("You already committed to something else this window.".into());
                }
                if gripping {
                    return Err("Your hands are on the log.".into());
                }
                if self.loose_ids(actor.id, Material::Mat).is_empty() {
                    return Err("You are not carrying a reed mat.".into());
                }
                if self.reed_mat_here(actor.cell) {
                    return Err("A reed mat is already set here.".into());
                }
                Ok(true)
            }
            Attempt::HollowLog => {
                if busy {
                    Err("You are in the middle of something.".into())
                } else if body_taken {
                    Err("You already committed to something else this window.".into())
                } else if gripping {
                    Err("Your hands are on the log.".into())
                } else if self.canoe_at(actor.cell).is_some() {
                    Err("The log is already a canoe.".into())
                } else if !self.log_at(actor.cell) {
                    Err("There is no heavy log here.".into())
                } else if self.log_charred_at(actor.cell) {
                    Err("The heavy log is already charred.".into())
                } else if self.fire_at(actor.cell).is_none() {
                    Err("There is no fire burning here.".into())
                } else {
                    Ok(true)
                }
            }
            Attempt::CarveCanoe => {
                if busy {
                    Err("You are in the middle of something.".into())
                } else if body_taken {
                    Err("You already committed to something else this window.".into())
                } else if gripping {
                    Err("Let go of the log first.".into())
                } else if self.canoe_at(actor.cell).is_some() {
                    Err("The canoe is already carved.".into())
                } else if !self.log_charred_at(actor.cell) {
                    Err("The heavy log is not charred.".into())
                } else if !self.has_carve_tool(actor.id) {
                    Err("You need a flint flake or an iron knife to carve the canoe.".into())
                } else {
                    Ok(true)
                }
            }
            Attempt::SealCanoe => {
                if busy {
                    Err("You are in the middle of something.".into())
                } else if body_taken {
                    Err("You already committed to something else this window.".into())
                } else if gripping {
                    Err("Your hands are on the log.".into())
                } else if self.canoe_to_seal(actor.id).is_none() {
                    Err("There is no dugout canoe here.".into())
                } else if self
                    .canoe_to_seal(actor.id)
                    .is_some_and(|id| self.canoe_pitched(id))
                {
                    Err("The canoe is already pitched.".into())
                } else if self.loose_ids(actor.id, Material::Resin).is_empty() {
                    Err("You have no pine resin.".into())
                } else {
                    Ok(true)
                }
            }
            Attempt::Launch => {
                if busy {
                    Err("You are in the middle of something.".into())
                } else if body_taken {
                    Err("You already committed to something else this window.".into())
                } else if gripping {
                    Err("Your hands are on the log.".into())
                } else if self.canoe_at(actor.cell).is_none() {
                    Err("There is no dugout canoe here.".into())
                } else if self
                    .canoe_at(actor.cell)
                    .is_some_and(|id| self.canoe_floats(id))
                {
                    Err("The canoe is already in the brook.".into())
                } else if self.brook_neighbor(actor.cell).is_none() {
                    Err("The brook is not beside you.".into())
                } else {
                    Ok(true)
                }
            }
            Attempt::Board => {
                if busy {
                    Err("You are in the middle of something.".into())
                } else if body_taken {
                    Err("You already committed to something else this window.".into())
                } else if gripping {
                    Err("Your hands are on the log.".into())
                } else if self
                    .aboard
                    .get(&actor.id)
                    .is_some_and(|id| self.canoe_floats(*id))
                {
                    Err("You are already aboard the canoe.".into())
                } else if self.floating_canoe_near(actor.cell).is_none() {
                    Err("There is no canoe here to board.".into())
                } else {
                    Ok(true)
                }
            }
            Attempt::Paddle(dir) => {
                if busy {
                    Err("You are in the middle of something.".into())
                } else if body_taken {
                    Err("You already committed to something else this window.".into())
                } else if self
                    .aboard
                    .get(&actor.id)
                    .is_none_or(|id| !self.canoe_floats(*id))
                {
                    Err("You are not aboard a canoe.".into())
                } else if let Err(message) = self.paddle_way(actor.cell, *dir) {
                    Err(message.into())
                } else {
                    Ok(true)
                }
            }
            Attempt::Disembark => {
                if busy {
                    Err("You are in the middle of something.".into())
                } else if body_taken {
                    Err("You already committed to something else this window.".into())
                } else if self
                    .aboard
                    .get(&actor.id)
                    .is_none_or(|id| !self.canoe_floats(*id))
                {
                    Err("You are not aboard a canoe.".into())
                } else if self.bank_beside(actor.cell).is_none() {
                    Err("There is no bank here.".into())
                } else {
                    Ok(true)
                }
            }
            Attempt::Beach => {
                if busy {
                    Err("You are in the middle of something.".into())
                } else if body_taken {
                    Err("You already committed to something else this window.".into())
                } else if gripping {
                    Err("Your hands are on the log.".into())
                } else if self.floating_canoe_near(actor.cell).is_some() {
                    if self.beach_bank(actor.id).is_none() {
                        Err("There is no bank here.".into())
                    } else {
                        Ok(true)
                    }
                } else {
                    Err(self.beach_miss(actor.id).into())
                }
            }
            Attempt::PutFish => {
                if gripping {
                    return Err("Your hands are on the log.".into());
                }
                if self.loose_ids(actor.id, Material::Basket).is_empty() {
                    return Err("You are not carrying a bark basket.".into());
                }
                if self.basket_holds_berries(actor.id) {
                    return Err("The bark basket already holds a handful of berries.".into());
                }
                if self.basket_holds_fish(actor.id) {
                    return Err("The bark basket already holds a raw fish.".into());
                }
                if self.raw_fish_in_arms(actor.id).is_none() {
                    return Err("You are not carrying a raw fish.".into());
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
                    | Material::Cord
                    | Material::Fish
                    | Material::Brand
                    | Material::Charcoal
                    | Material::Bone
                    | Material::Salt
                    | Material::Honey
                    | Material::Egg
                    | Material::Net
                    | Material::Mat
                    | Material::Paddle
                    | Material::Hook
                    | Material::Willow
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
            let noun = if material == Material::Hide
                && self
                    .ground_ids(cell, Material::Hide)
                    .first()
                    .is_some_and(|id| self.item(*id).is_some_and(|hide| hide.wolf))
            {
                "wolf hide"
            } else {
                material.gather_noun()
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
                            if material == Material::BitterBerry {
                                format!(
                                    "There are no {} here to gather.",
                                    self.bitter_kind(intent.actor_id)
                                )
                            } else {
                                format!("There is no {} here to gather.", material.word())
                            },
                            results,
                        );
                        continue;
                    };
                    self.tuck_overflow_into_basket(intent.actor_id, item);
                    let from = if material == Material::Pot {
                        format!(" from the ground {}", self.cover_phrase(cell))
                    } else {
                        String::new()
                    };
                    let shaped = material == Material::Clay
                        && self.item(item).is_some_and(|clay| clay.shaped);
                    let awl_kind =
                        material == Material::Awl && self.item(item).is_some_and(|awl| awl.bone);
                    let rest = if shaped {
                        "an unfired clay pot.".to_string()
                    } else if material == Material::Cord {
                        "hide strips.".to_string()
                    } else if material == Material::Awl {
                        let kind = if awl_kind { "bone awl" } else { "copper awl" };
                        format!("a {kind}{from}.")
                    } else if material == Material::Egg {
                        "the eggs.".to_string()
                    } else if material == Material::Honey && self.smoked_nests.contains(&cell) {
                        "a honeycomb.".to_string()
                    } else if material == Material::Honey {
                        "a honeycomb. The bees sting.".to_string()
                    } else {
                        format!("{} {noun}{from}.", material.article())
                    };
                    if material == Material::Salt {
                        self.arm_salt_lick(cell);
                    }
                    if material == Material::Honey && self.smoked_nests.contains(&cell) {
                        self.smoked_nests.remove(&cell);
                    } else if material == Material::Honey {
                        let until = self.time.saturating_add(self.tuning.sting_ticks);
                        if let Some(person) = self.actors.get_mut(&intent.actor_id) {
                            person.stings_until = Some(until);
                        }
                    }
                    if material == Material::Honey {
                        self.arm_honey_nest(cell);
                    }
                    if material == Material::Egg {
                        self.arm_bird_nest(cell);
                    }
                    if material == Material::Moss {
                        self.arm_moss_bank(cell);
                    }
                    if material == Material::Resin {
                        self.arm_resin_stand(cell);
                    }
                    if material == Material::Willow {
                        self.arm_willow_stand(cell);
                    }
                    if material == Material::Flint {
                        self.arm_flint_bed(cell);
                    }
                    if material == Material::Ochre {
                        self.arm_ochre_bed(cell);
                    }
                    if material == Material::Clay {
                        self.arm_clay_bank(cell);
                    }
                    if material == Material::CopperOre {
                        self.arm_copper_bed(cell);
                    }
                    if material == Material::IronOre {
                        self.arm_iron_bed(cell);
                    }
                    if material == Material::Reed {
                        self.arm_reed_bed(cell);
                    }
                    if material == Material::Net {
                        self.clear_set_net(cell);
                    }
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
                    let mut reply = crate::model::spoken_act(None, "gather", &rest);
                    if material == Material::BitterBerry && self.bitter_known(intent.actor_id) {
                        reply = reply.replace("dark berries", "poison berries");
                    }
                    self.finish(intent.command_id, intent.actor_id, true, reply, results);
                } else if noun == "wolf hide" {
                    self.finish(
                        intent.command_id,
                        intent.actor_id,
                        false,
                        format!("Your hand meets someone else's on the same {noun}."),
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
                    if material == Material::BitterBerry {
                        format!(
                            "You are not carrying loose {}.",
                            self.bitter_kind(intent.actor_id)
                        )
                    } else {
                        format!("You are not carrying a loose {}.", material.word())
                    },
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
                item.in_basket = false;
            }
            if material == Material::Basket {
                let held: Vec<_> = self
                    .items
                    .values()
                    .filter(|item| item.carrier == Some(intent.actor_id) && item.in_basket)
                    .map(|item| item.id)
                    .collect();
                for id in held {
                    if let Some(held) = self.items.get_mut(&id) {
                        held.carrier = None;
                        held.cell = cell;
                        held.in_basket = false;
                    }
                }
            }
            let bone_awl =
                material == Material::Awl && self.item(item_id).is_some_and(|awl| awl.bone);
            let noun = if bone_awl {
                "bone awl"
            } else {
                material.gather_noun()
            };
            let dropped = if material == Material::Cord {
                format!("{name} drops hide strips. They are in contact with the ground.")
            } else if material == Material::Egg {
                format!("{name} drops the eggs. They are in contact with the ground.")
            } else {
                format!(
                    "{name} drops {} {noun}. It is in contact with the ground.",
                    material.article()
                )
            };
            self.emit(
                dropped,
                Scope::Cell(cell),
                vec![intent.actor_id],
                vec![cell],
                vec![item_id],
                vec![],
                None,
            );
            let mut reply = if material == Material::Cord {
                "You drop hide strips. They are in contact with the ground.".to_string()
            } else if material == Material::Egg {
                "You drop the eggs. They are in contact with the ground.".to_string()
            } else {
                format!(
                    "You drop {} {noun}. It is in contact with the ground.",
                    material.article()
                )
            };
            if material == Material::BitterBerry && self.bitter_known(intent.actor_id) {
                reply = reply.replace("dark berries", "poison berries");
            }
            self.finish(intent.command_id, intent.actor_id, true, reply, results);
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
            let id = self.mint_item(Material::Log, moisture, cell, None, Role::Loose);
            if self.charred_logs.remove(&cell) {
                self.charred_items.insert(id);
            }
            id
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
            let fled = self.scare_wolf(Some(intent.actor_id));
            if !fled {
                self.frighten_wolf(origin, dest);
            }
            let brook = self.cell(dest).region == Region::Brook;
            let ice = self.still_frozen(dest);
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
            let text = if ice {
                format!("{name} walks {word} across the ice.", word = dir.word())
            } else if brook {
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
            let mut message = if ice {
                format!("You walk {} across the ice.", dir.word())
            } else if brook {
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
            if fled {
                message.push_str(" The wolf flees the fire.");
            }
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

    /// The fire chars the heavy log. The fire is not used up. Look names the char.
    fn apply_hollows(&mut self, hollows: Vec<Intent>, results: &mut Vec<CommandResult>) {
        for intent in hollows {
            let Some(actor) = self.actor(intent.actor_id).cloned() else {
                continue;
            };
            if self.canoe_at(actor.cell).is_some() {
                self.finish(
                    intent.command_id,
                    actor.id,
                    false,
                    "The log is already a canoe.",
                    results,
                );
                continue;
            }
            if !self.log_at(actor.cell) {
                self.finish(
                    intent.command_id,
                    actor.id,
                    false,
                    "There is no heavy log here.",
                    results,
                );
                continue;
            }
            if self.log_charred_at(actor.cell) {
                self.finish(
                    intent.command_id,
                    actor.id,
                    false,
                    "The heavy log is already charred.",
                    results,
                );
                continue;
            }
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
            if let Some(id) = self
                .log_item()
                .filter(|log| log.cell == actor.cell)
                .map(|log| log.id)
            {
                self.charred_items.insert(id);
            } else {
                self.charred_logs.insert(actor.cell);
            }
            let name = self.name_of(actor.id);
            self.emit(
                format!("{name} hollows the heavy log with the fire."),
                Scope::Cell(actor.cell),
                vec![actor.id],
                vec![actor.cell],
                vec![],
                vec![],
                None,
            );
            self.finish(
                intent.command_id,
                actor.id,
                true,
                "You hollow the heavy log with the fire.",
                results,
            );
        }
    }

    /// A flint flake or an iron knife cuts the charred log into a dugout. The tool stays.
    fn apply_canoe_carves(&mut self, carves: Vec<Intent>, results: &mut Vec<CommandResult>) {
        for intent in carves {
            let Some(actor) = self.actor(intent.actor_id).cloned() else {
                continue;
            };
            if self.canoe_at(actor.cell).is_some() {
                self.finish(
                    intent.command_id,
                    actor.id,
                    false,
                    "The canoe is already carved.",
                    results,
                );
                continue;
            }
            if !self.log_charred_at(actor.cell) {
                self.finish(
                    intent.command_id,
                    actor.id,
                    false,
                    "The heavy log is not charred.",
                    results,
                );
                continue;
            }
            if !self.has_carve_tool(actor.id) {
                self.finish(
                    intent.command_id,
                    actor.id,
                    false,
                    "You need a flint flake or an iron knife to carve the canoe.",
                    results,
                );
                continue;
            }
            if self.gripping(actor.id).is_some() {
                self.finish(
                    intent.command_id,
                    actor.id,
                    false,
                    "Let go of the log first.",
                    results,
                );
                continue;
            }
            if let Some(id) = self
                .log_item()
                .filter(|log| log.cell == actor.cell && log.carrier.is_none())
                .map(|log| log.id)
            {
                self.items.remove(&id);
                self.charred_items.remove(&id);
            } else if self.take_from_pile(actor.cell, Material::Log, 1).is_none() {
                self.finish(
                    intent.command_id,
                    actor.id,
                    false,
                    "There is no heavy log here.",
                    results,
                );
                continue;
            }
            self.charred_logs.remove(&actor.cell);
            let id = self.mint_item(Material::Canoe, 0, actor.cell, None, Role::Loose);
            let name = self.name_of(actor.id);
            self.emit(
                format!("{name} carves the charred log into a dugout canoe."),
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
                "You carve the charred log into a dugout canoe.",
                results,
            );
        }
    }

    /// The dugout on this bank slides into the brook beside it. The person stays on the bank.
    fn apply_launches(&mut self, launches: Vec<Intent>, results: &mut Vec<CommandResult>) {
        for intent in launches {
            let Some(actor) = self.actor(intent.actor_id).cloned() else {
                continue;
            };
            let Some(id) = self.canoe_at(actor.cell) else {
                self.finish(
                    intent.command_id,
                    actor.id,
                    false,
                    "There is no dugout canoe here.",
                    results,
                );
                continue;
            };
            if self.canoe_floats(id) {
                self.finish(
                    intent.command_id,
                    actor.id,
                    false,
                    "The canoe is already in the brook.",
                    results,
                );
                continue;
            }
            let Some(brook) = self.brook_neighbor(actor.cell) else {
                self.finish(
                    intent.command_id,
                    actor.id,
                    false,
                    "The brook is not beside you.",
                    results,
                );
                continue;
            };
            if let Some(canoe) = self.items.get_mut(&id) {
                canoe.cell = brook;
            }
            let name = self.name_of(actor.id);
            self.emit(
                format!("{name} launches the dugout canoe into the brook."),
                Scope::Cell(actor.cell),
                vec![actor.id],
                vec![actor.cell, brook],
                vec![id],
                vec![],
                None,
            );
            self.finish(
                intent.command_id,
                actor.id,
                true,
                "You launch the dugout canoe into the brook.",
                results,
            );
        }
    }

    /// One lump of pine resin, spent on the dugout. Beached, afloat, or aboard.
    fn apply_seals(&mut self, seals: Vec<Intent>, results: &mut Vec<CommandResult>) {
        for intent in seals {
            let Some(actor) = self.actor(intent.actor_id).cloned() else {
                continue;
            };
            let Some(id) = self.canoe_to_seal(actor.id) else {
                self.finish(
                    intent.command_id,
                    actor.id,
                    false,
                    "There is no dugout canoe here.",
                    results,
                );
                continue;
            };
            if self.canoe_pitched(id) {
                self.finish(
                    intent.command_id,
                    actor.id,
                    false,
                    "The canoe is already pitched.",
                    results,
                );
                continue;
            }
            let Some(lump) = self.loose_ids(actor.id, Material::Resin).into_iter().next() else {
                self.finish(
                    intent.command_id,
                    actor.id,
                    false,
                    "You have no pine resin.",
                    results,
                );
                continue;
            };
            self.items.remove(&lump);
            if let Some(canoe) = self.items.get_mut(&id) {
                canoe.pitched = true;
            }
            let name = self.name_of(actor.id);
            let cell = self.item(id).map(|canoe| canoe.cell).unwrap_or(actor.cell);
            self.emit(
                format!("{name} seals the dugout canoe with pine resin."),
                Scope::Cell(cell),
                vec![actor.id],
                vec![cell],
                vec![id],
                vec![],
                None,
            );
            self.finish(
                intent.command_id,
                actor.id,
                true,
                "You seal the dugout canoe with pine resin.",
                results,
            );
        }
    }

    /// Step into a floating dugout on this cell or the next. The brook still wets you.
    fn apply_boards(&mut self, boards: Vec<Intent>, results: &mut Vec<CommandResult>) {
        for intent in boards {
            let Some(actor) = self.actor(intent.actor_id).cloned() else {
                continue;
            };
            let Some(id) = self.floating_canoe_near(actor.cell) else {
                self.finish(
                    intent.command_id,
                    actor.id,
                    false,
                    "There is no canoe here to board.",
                    results,
                );
                continue;
            };
            let dest = self.item(id).map(|canoe| canoe.cell).unwrap_or(actor.cell);
            let from_bank = actor.cell != dest;
            self.move_actor(actor.id, dest);
            if from_bank && self.cell(dest).region == Region::Brook {
                if let Some(person) = self.actors.get_mut(&actor.id) {
                    person.wetness = (person.wetness + self.tuning.wade).min(1000);
                }
            }
            self.aboard.insert(actor.id, id);
            let name = self.name_of(actor.id);
            self.emit(
                format!("{name} boards the dugout canoe."),
                Scope::Cell(dest),
                vec![actor.id],
                vec![dest],
                vec![id],
                vec![],
                None,
            );
            self.finish(
                intent.command_id,
                actor.id,
                true,
                "You board the dugout canoe.",
                results,
            );
        }
    }

    /// The dugout and everyone aboard it move one brook cell.
    fn apply_paddles(&mut self, paddles: Vec<Intent>, results: &mut Vec<CommandResult>) {
        for intent in paddles {
            let Attempt::Paddle(asked) = intent.attempt else {
                continue;
            };
            let Some(actor) = self.actor(intent.actor_id).cloned() else {
                continue;
            };
            let Some(canoe) = self
                .aboard
                .get(&actor.id)
                .copied()
                .filter(|id| self.canoe_floats(*id))
            else {
                self.finish(
                    intent.command_id,
                    actor.id,
                    false,
                    "You are not aboard a canoe.",
                    results,
                );
                continue;
            };
            let Ok(dir) = self.paddle_way(actor.cell, asked) else {
                let message = self.paddle_way(actor.cell, asked).unwrap_err();
                self.finish(intent.command_id, actor.id, false, message, results);
                continue;
            };
            let Some(dest) = self.brook_step(actor.cell, dir) else {
                continue;
            };
            let origin = actor.cell;
            if let Some(boat) = self.items.get_mut(&canoe) {
                boat.cell = dest;
            }
            let riders: Vec<_> = self
                .aboard
                .iter()
                .filter(|(_, id)| **id == canoe)
                .map(|(who, _)| *who)
                .collect();
            for rider in &riders {
                self.move_actor(*rider, dest);
                self.aboard.insert(*rider, canoe);
            }
            let name = self.name_of(actor.id);
            let rest = if self.loose_ids(actor.id, Material::Paddle).is_empty() {
                format!("the dugout canoe {}.", dir.word())
            } else {
                format!("the dugout canoe {} with the wooden paddle.", dir.word())
            };
            self.emit(
                crate::model::spoken_act(Some(name.as_str()), "paddle", &rest),
                Scope::Cell(dest),
                vec![actor.id],
                vec![origin, dest],
                vec![canoe],
                vec![],
                None,
            );
            self.finish(
                intent.command_id,
                actor.id,
                true,
                crate::model::spoken_act(None, "paddle", &rest),
                results,
            );
        }
    }

    /// Step out of the dugout onto the bank. The canoe stays in the brook.
    fn apply_disembarks(&mut self, disembarks: Vec<Intent>, results: &mut Vec<CommandResult>) {
        for intent in disembarks {
            let Some(actor) = self.actor(intent.actor_id).cloned() else {
                continue;
            };
            let Some(canoe) = self
                .aboard
                .get(&actor.id)
                .copied()
                .filter(|id| self.canoe_floats(*id))
            else {
                self.finish(
                    intent.command_id,
                    actor.id,
                    false,
                    "You are not aboard a canoe.",
                    results,
                );
                continue;
            };
            let Some(bank) = self.bank_beside(actor.cell) else {
                self.finish(
                    intent.command_id,
                    actor.id,
                    false,
                    "There is no bank here.",
                    results,
                );
                continue;
            };
            let origin = actor.cell;
            self.move_actor(actor.id, bank);
            self.aboard.remove(&actor.id);
            let name = self.name_of(actor.id);
            self.emit(
                format!("{name} leaves the dugout canoe."),
                Scope::Cell(origin),
                vec![actor.id],
                vec![origin, bank],
                vec![canoe],
                vec![],
                None,
            );
            self.finish(
                intent.command_id,
                actor.id,
                true,
                "You leave the dugout canoe.",
                results,
            );
        }
    }

    /// Haul a floating dugout onto the bank. Anyone aboard comes ashore with it.
    fn apply_beaches(&mut self, beaches: Vec<Intent>, results: &mut Vec<CommandResult>) {
        for intent in beaches {
            let Some(actor) = self.actor(intent.actor_id).cloned() else {
                continue;
            };
            let Some(id) = self.floating_canoe_near(actor.cell) else {
                self.finish(
                    intent.command_id,
                    actor.id,
                    false,
                    self.beach_miss(actor.id),
                    results,
                );
                continue;
            };
            let Some(bank) = self.beach_bank(actor.id) else {
                self.finish(
                    intent.command_id,
                    actor.id,
                    false,
                    "There is no bank here.",
                    results,
                );
                continue;
            };
            let origin = self.item(id).map(|canoe| canoe.cell).unwrap_or(actor.cell);
            let riders: Vec<_> = self
                .aboard
                .iter()
                .filter(|(_, canoe)| **canoe == id)
                .map(|(who, _)| *who)
                .collect();
            if let Some(canoe) = self.items.get_mut(&id) {
                canoe.cell = bank;
            }
            for rider in &riders {
                self.move_actor(*rider, bank);
                self.aboard.remove(rider);
            }
            let name = self.name_of(actor.id);
            self.emit(
                format!("{name} beaches the dugout canoe."),
                Scope::Cell(bank),
                vec![actor.id],
                vec![origin, bank],
                vec![id],
                vec![],
                None,
            );
            self.finish(
                intent.command_id,
                actor.id,
                true,
                "You beach the dugout canoe.",
                results,
            );
        }
    }

    /// One person hauls a dugout one cell. It does not enter the brook; launch does that.
    fn apply_canoe_drags(&mut self, drags: Vec<Intent>, results: &mut Vec<CommandResult>) {
        for intent in drags {
            let Attempt::Drag(dir) = intent.attempt else {
                continue;
            };
            let Some(actor) = self.actor(intent.actor_id).cloned() else {
                continue;
            };
            let Some(id) = self.canoe_at(actor.cell) else {
                self.finish(
                    intent.command_id,
                    actor.id,
                    false,
                    "There is no dugout canoe here.",
                    results,
                );
                continue;
            };
            if self.canoe_floats(id) {
                self.finish(
                    intent.command_id,
                    actor.id,
                    false,
                    "The canoe is already in the brook.",
                    results,
                );
                continue;
            }
            let (x, y) = cell_xy(actor.cell);
            let (dx, dy) = dir.delta();
            let Some(dest) = cell_id(x + dx, y + dy) else {
                self.finish(
                    intent.command_id,
                    actor.id,
                    false,
                    "There is nowhere to drag the canoe.",
                    results,
                );
                continue;
            };
            if self.cell(dest).region == Region::Brook {
                self.finish(
                    intent.command_id,
                    actor.id,
                    false,
                    "Launch the canoe from the bank.",
                    results,
                );
                continue;
            }
            if let Some(canoe) = self.items.get_mut(&id) {
                canoe.cell = dest;
            }
            self.move_actor(actor.id, dest);
            let name = self.name_of(actor.id);
            self.emit(
                format!("{name} drags the dugout canoe {}.", dir.word()),
                Scope::Cell(dest),
                vec![actor.id],
                vec![actor.cell, dest],
                vec![id],
                vec![],
                None,
            );
            self.finish(
                intent.command_id,
                actor.id,
                true,
                format!("You drag the dugout canoe {}.", dir.word()),
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
        let mut canoe_drags = Vec::new();
        let mut log_drags = Vec::new();
        for intent in drags {
            let on_canoe = self
                .actor(intent.actor_id)
                .and_then(|person| self.canoe_at(person.cell))
                .is_some();
            if on_canoe && self.gripping(intent.actor_id).is_none() {
                canoe_drags.push(intent);
            } else {
                log_drags.push(intent);
            }
        }
        self.apply_canoe_drags(canoe_drags, results);
        for intent in log_drags {
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
        let resin = !self.loose_ids(actor, Material::Resin).is_empty();
        if self.raining && self.exposure_millis(person.cell) >= 1000 {
            return Some(if resin {
                "The rain puts the resin out before it catches.".into()
            } else {
                "The rain puts the bark out before it catches.".into()
            });
        }
        if self.fire_at(person.cell).is_some() {
            return Some("A fire already burns here.".into());
        }
        // A lump of pine resin in the arms is tinder. It is spent before
        // bark when both are held, and it stands in when bark is absent.
        // Resin on the ground is not tinder.
        if !resin && !self.has_dry(actor, Material::Bark) {
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
            && !self.raw_fish_near(actor)
        {
            if self.spoiled_meat_near(actor) {
                return Some("The meat has spoiled.".into());
            }
            if self.spoiled_fish_near(actor) {
                return Some("The fish has spoiled.".into());
            }
            if self.has_material_at_hand(actor, Material::BitterBerry) {
                let kind = self.bitter_kind(actor);
                return Some(format!("Cooking does not make {kind} food."));
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
                let brand = self
                    .loose_ids(intent.actor_id, Material::Brand)
                    .first()
                    .copied();
                let resin = self
                    .loose_ids(intent.actor_id, Material::Resin)
                    .first()
                    .copied();
                let name = self.name_of(intent.actor_id);
                let (line, yours, spent) = if let Some(resin_id) = resin {
                    let Ok((branch_item, branch_arms)) =
                        self.consume_dry(intent.actor_id, Material::Branch)
                    else {
                        self.finish(
                            intent.command_id,
                            intent.actor_id,
                            false,
                            "You need a dry branch for the fire.",
                            results,
                        );
                        continue;
                    };
                    self.items.remove(&resin_id);
                    if let Some(brand) = brand {
                        self.items.remove(&brand);
                    }
                    let line = if brand.is_some() {
                        resin_brand_light_line(Some(&name), branch_arms)
                    } else {
                        resin_light_line(Some(&name), branch_arms)
                    };
                    let yours = if brand.is_some() {
                        resin_brand_light_line(None, branch_arms)
                    } else {
                        resin_light_line(None, branch_arms)
                    };
                    (
                        line,
                        yours,
                        Some(resin_id).into_iter().chain(branch_item).collect(),
                    )
                } else {
                    let bark = self.consume_dry(intent.actor_id, Material::Bark);
                    let branch = self.consume_dry(intent.actor_id, Material::Branch);
                    let (Ok((bark_item, bark_arms)), Ok((branch_item, branch_arms))) =
                        (bark, branch)
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
                    if let Some(brand) = brand {
                        self.items.remove(&brand);
                    }
                    let line = if brand.is_some() {
                        brand_light_line(Some(&name), bark_arms, branch_arms)
                    } else {
                        light_line(Some(&name), bark_arms, branch_arms)
                    };
                    let yours = if brand.is_some() {
                        brand_light_line(None, bark_arms, branch_arms)
                    } else {
                        light_line(None, bark_arms, branch_arms)
                    };
                    (
                        line,
                        yours,
                        bark_item.into_iter().chain(branch_item).collect(),
                    )
                };
                let event = self.emit(
                    line,
                    Scope::Cell(cell),
                    vec![intent.actor_id],
                    vec![cell],
                    spent,
                    vec![],
                    draw,
                );
                self.mint_fire(cell, event);
                lit = true;
                let mut yours = yours;
                if self.scare_wolf(Some(intent.actor_id)) {
                    yours.push_str(" The wolf flees the fire.");
                }
                self.finish(intent.command_id, intent.actor_id, true, yours, results);
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
            if self.raw_fish_id(intent.actor_id).is_some() {
                self.cook_fish(intent, results);
                continue;
            }
            if self.raw_egg_id(intent.actor_id).is_some()
                || self.pile_count(cell, Material::Egg) > 0
            {
                self.cook_eggs(intent, results);
                continue;
            }
            if self.raw_mushroom_id(intent.actor_id).is_some()
                || self.pile_count(cell, Material::Mushroom) > 0
            {
                self.cook_mushroom(intent, results);
                continue;
            }
            if self.raw_pale_id(intent.actor_id).is_some()
                || self.pile_count(cell, Material::PaleBerry) > 0
            {
                self.cook_pale(intent, results);
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

    /// Heat leaves the eggs in the arms, or on the ground if they were still in the nest.
    fn cook_eggs(&mut self, intent: Intent, results: &mut Vec<CommandResult>) {
        if let Some(message) = self.cook_refusal(intent.actor_id) {
            self.finish(intent.command_id, intent.actor_id, false, message, results);
            return;
        }
        let cell = self
            .actor(intent.actor_id)
            .map(|actor| actor.cell)
            .unwrap_or(0);
        let id = if let Some(id) = self.raw_egg_id(intent.actor_id) {
            if let Some(eggs) = self.items.get_mut(&id) {
                eggs.cooked = true;
            }
            id
        } else if self.pile_count(cell, Material::Egg) > 0 {
            let moisture = self.take_from_pile(cell, Material::Egg, 1).unwrap_or(0);
            let id = self.mint_item(Material::Egg, moisture, cell, None, Role::Loose);
            if let Some(eggs) = self.items.get_mut(&id) {
                eggs.cooked = true;
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
        let rest = "the eggs over the fire.";
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

    /// Heat leaves the pale berries in the arms, or on the ground if they were a pile.
    /// They do not feed anyone until they are eaten.
    fn cook_pale(&mut self, intent: Intent, results: &mut Vec<CommandResult>) {
        if let Some(message) = self.cook_refusal(intent.actor_id) {
            self.finish(intent.command_id, intent.actor_id, false, message, results);
            return;
        }
        let cell = self
            .actor(intent.actor_id)
            .map(|actor| actor.cell)
            .unwrap_or(0);
        let id = if let Some(id) = self.raw_pale_id(intent.actor_id) {
            if let Some(berry) = self.items.get_mut(&id) {
                berry.cooked = true;
            }
            id
        } else if self.pile_count(cell, Material::PaleBerry) > 0 {
            let moisture = self
                .take_from_pile(cell, Material::PaleBerry, 1)
                .unwrap_or(0);
            let id = self.mint_item(Material::PaleBerry, moisture, cell, None, Role::Loose);
            if let Some(berry) = self.items.get_mut(&id) {
                berry.cooked = true;
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
        if let Some(person) = self.actors.get_mut(&intent.actor_id) {
            person.pale_berry_known = true;
        }
        let rest = "the pale berries over the fire.";
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
            meat.spoiled = false;
            meat.spoils_at = None;
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
            let beds: Vec<_> = self.flint_beds.iter().copied().collect();
            for bed in beds {
                self.arm_flint_bed(bed);
            }
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

    fn wet_branch_choice(&self, actor: crate::model::ActorId) -> WetBranch {
        let Some(person) = self.actor(actor) else {
            return WetBranch::None;
        };
        let mut ids = self.loose_ids(actor, Material::Branch);
        ids.extend(self.ground_ids(person.cell, Material::Branch));
        let wet = ids.iter().any(|id| {
            self.item(*id)
                .is_some_and(|item| !self.is_dry(item.moisture) && item.firing_ready_at.is_none())
        });
        if wet {
            return WetBranch::Ready;
        }
        if self.pile_count(person.cell, Material::Branch) > 0
            && !self.is_dry(self.pile_moisture(person.cell, Material::Branch))
        {
            return WetBranch::Ready;
        }
        let drying = ids.iter().any(|id| {
            self.item(*id)
                .is_some_and(|item| item.firing_ready_at.is_some())
        });
        if drying {
            WetBranch::Already
        } else if !ids.is_empty() || self.pile_count(person.cell, Material::Branch) > 0 {
            WetBranch::Dry
        } else {
            WetBranch::None
        }
    }

    /// A carried wet branch, else one lying here, else one taken from a wet pile.
    fn take_wet_branch(&mut self, actor: crate::model::ActorId) -> Option<crate::model::ItemId> {
        let cell = self.actor(actor)?.cell;
        let mut ids = self.loose_ids(actor, Material::Branch);
        ids.extend(self.ground_ids(cell, Material::Branch));
        let ready = ids
            .into_iter()
            .filter(|id| {
                self.item(*id).is_some_and(|item| {
                    !self.is_dry(item.moisture) && item.firing_ready_at.is_none()
                })
            })
            .max_by_key(|id| self.item(*id).map(|item| item.moisture).unwrap_or(0));
        if ready.is_some() {
            return ready;
        }
        if self.pile_count(cell, Material::Branch) == 0
            || self.is_dry(self.pile_moisture(cell, Material::Branch))
        {
            return None;
        }
        let moisture = self.take_from_pile(cell, Material::Branch, 1)?;
        Some(self.mint_item(Material::Branch, moisture, cell, None, Role::Loose))
    }

    fn apply_dries(&mut self, dries: Vec<Intent>, results: &mut Vec<CommandResult>) {
        for intent in dries {
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
            let Some(id) = self.take_wet_branch(intent.actor_id) else {
                let message = match self.wet_branch_choice(intent.actor_id) {
                    WetBranch::Already => "The branch is already drying by the fire.",
                    WetBranch::Dry => "The branch is already dry.",
                    _ => "You have no branch to dry.",
                };
                self.finish(intent.command_id, intent.actor_id, false, message, results);
                continue;
            };
            let word = self
                .item(id)
                .map(|item| self.condition_word(item.moisture))
                .unwrap_or("damp");
            let carried = self
                .item(id)
                .is_some_and(|item| item.carrier == Some(intent.actor_id));
            let ready = self.time.saturating_add(self.tuning.clay_ticks);
            if let Some(branch) = self.items.get_mut(&id) {
                branch.firing_ready_at = Some(ready);
            }
            self.schedule(ready, Wake::DryBranch(id));
            let (verb, rest) = if carried {
                (
                    "hold",
                    format!("the {word} branch by the fire. It is not dry yet."),
                )
            } else {
                (
                    "set",
                    format!("a {word} branch by the fire. It is not dry yet."),
                )
            };
            let name = self.name_of(intent.actor_id);
            self.emit(
                crate::model::spoken_act(Some(name.as_str()), verb, &rest),
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
                crate::model::spoken_act(None, verb, &rest),
                results,
            );
        }
    }

    /// A carried brand, or a fire in this cell or the next.
    pub(crate) fn smoke_reaches(
        &self,
        actor: crate::model::ActorId,
        cell: crate::model::CellId,
    ) -> bool {
        !self.loose_ids(actor, Material::Brand).is_empty()
            || self
                .fires
                .values()
                .any(|fire| crate::model::chebyshev(fire.cell, cell) <= 1)
    }

    fn apply_smokes(&mut self, smokes: Vec<Intent>, results: &mut Vec<CommandResult>) {
        let rest = "the bees' nest. The bees quiet.";
        for intent in smokes {
            let cell = self
                .actor(intent.actor_id)
                .map(|actor| actor.cell)
                .unwrap_or(0);
            self.smoked_nests.insert(cell);
            let name = self.name_of(intent.actor_id);
            self.emit(
                crate::model::spoken_act(Some(name.as_str()), "smoke", rest),
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
                crate::model::spoken_act(None, "smoke", rest),
                results,
            );
        }
    }

    fn apply_brands(&mut self, brands: Vec<Intent>, results: &mut Vec<CommandResult>) {
        let rest = "a burning brand from the fire. It stays lit.";
        for intent in brands {
            let cell = self
                .actor(intent.actor_id)
                .map(|actor| actor.cell)
                .unwrap_or(0);
            let id = self.mint_item(Material::Brand, 0, cell, Some(intent.actor_id), Role::Loose);
            let ready = self.time.saturating_add(self.tuning.clay_ticks);
            if let Some(brand) = self.items.get_mut(&id) {
                brand.firing_ready_at = Some(ready);
            }
            self.schedule(ready, Wake::BrandOut(id));
            let name = self.name_of(intent.actor_id);
            self.emit(
                crate::model::spoken_act(Some(name.as_str()), "take", rest),
                Scope::Cell(cell),
                vec![intent.actor_id],
                vec![cell],
                vec![id],
                vec![],
                None,
            );
            let mut yours = crate::model::spoken_act(None, "take", rest);
            if self.scare_wolf(Some(intent.actor_id)) {
                yours.push_str(" The wolf flees the fire.");
            }
            self.finish(intent.command_id, intent.actor_id, true, yours, results);
        }
    }

    fn fish_spear_refusal(&self, actor: crate::model::ActorId) -> Option<&'static str> {
        let ids = self.loose_ids(actor, Material::Spear);
        if ids.is_empty() {
            return Some("You are not carrying a spear.");
        }
        let flint = ids.iter().any(|id| {
            self.item(*id)
                .is_some_and(|spear| spear.flint_tip && spear.hardened)
        });
        if flint {
            return None;
        }
        if ids
            .iter()
            .any(|id| self.item(*id).is_some_and(|spear| spear.flint_tip))
        {
            Some("The flint tip is not fire-hardened.")
        } else {
            Some("You need a fire-hardened flint spear.")
        }
    }

    /// A spear that can take a fish, or a fishing net when no spear is carried,
    /// or a bone fish hook and hide strips when neither of those is carried.
    /// A spear that is not ready still refuses. The net and the hook do not override it.
    fn fish_tool_refusal(&self, actor: crate::model::ActorId) -> Option<&'static str> {
        match self.fish_spear_refusal(actor) {
            None => None,
            Some("You are not carrying a spear.")
                if !self.loose_ids(actor, Material::Net).is_empty() =>
            {
                None
            }
            Some("You are not carrying a spear.")
                if !self.loose_ids(actor, Material::Hook).is_empty()
                    && self.fishing_line(actor).is_some() =>
            {
                None
            }
            Some("You are not carrying a spear.")
                if !self.loose_ids(actor, Material::Hook).is_empty() =>
            {
                let cell = self.actor(actor).map(|person| person.cell).unwrap_or(0);
                if self
                    .brook_stretch(cell)
                    .is_some_and(|stretch| self.brook_is_quiet(stretch))
                {
                    Some("The fish are gone from this stretch of the brook.")
                } else {
                    Some("You have no hide strips for a line.")
                }
            }
            Some(message) => Some(message),
        }
    }

    fn hook_fish_cast(&self, intent: &Intent) -> bool {
        if matches!(intent.attempt, Attempt::FishHook) {
            return true;
        }
        self.fish_spear_refusal(intent.actor_id) == Some("You are not carrying a spear.")
            && self.loose_ids(intent.actor_id, Material::Net).is_empty()
            && !self.loose_ids(intent.actor_id, Material::Hook).is_empty()
            && self.fishing_line(intent.actor_id).is_some()
    }

    /// Hide strips if the arms hold them, otherwise a bundle of willow cordage.
    /// The hook stays. One bundle of line is spent.
    fn fishing_line(
        &self,
        actor: crate::model::ActorId,
    ) -> Option<(crate::model::ItemId, &'static str)> {
        if let Some(id) = self.loose_ids(actor, Material::Cord).first().copied() {
            return Some((id, "The hide strips are spent."));
        }
        if let Some(id) = self.loose_ids(actor, Material::Willow).first().copied() {
            return Some((id, "The willow cordage is spent."));
        }
        None
    }

    /// The line is spent. The hook stays.
    fn apply_hook_fish(&mut self, intent: Intent, results: &mut Vec<CommandResult>) {
        let cell = self
            .actor(intent.actor_id)
            .map(|actor| actor.cell)
            .unwrap_or(0);
        if !self.fishing_water(cell) {
            self.finish(
                intent.command_id,
                intent.actor_id,
                false,
                "The brook is not here.",
                results,
            );
            return;
        }
        if self.loose_ids(intent.actor_id, Material::Hook).is_empty() {
            self.finish(
                intent.command_id,
                intent.actor_id,
                false,
                "You are not carrying a bone fish hook.",
                results,
            );
            return;
        }
        if self
            .brook_stretch(cell)
            .is_some_and(|stretch| self.brook_is_quiet(stretch))
        {
            self.finish(
                intent.command_id,
                intent.actor_id,
                false,
                "The fish are gone from this stretch of the brook.",
                results,
            );
            return;
        }
        let Some((line, spent)) = self.fishing_line(intent.actor_id) else {
            self.finish(
                intent.command_id,
                intent.actor_id,
                false,
                "You have no hide strips for a line.",
                results,
            );
            return;
        };
        self.items.remove(&line);
        if let Some(stretch) = self.brook_stretch(cell) {
            let until = self.time.saturating_add(self.tuning.clay_ticks);
            self.brook_quiet.insert(stretch, until);
        }
        let into_basket = self.carry_count(intent.actor_id) >= self.tuning.carry_slots
            && self.basket_has_room(intent.actor_id);
        let carrier = if self.carry_count(intent.actor_id) < self.tuning.carry_slots || into_basket
        {
            Some(intent.actor_id)
        } else {
            None
        };
        let id = self.mint_item(Material::Fish, 0, cell, carrier, Role::Loose);
        if into_basket {
            if let Some(fish) = self.items.get_mut(&id) {
                fish.in_basket = true;
            }
        }
        let name = self.name_of(intent.actor_id);
        let rest = format!("a fish from the brook with the bone fish hook. {spent}");
        self.emit(
            crate::model::spoken_act(Some(name.as_str()), "catch", &rest),
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
            crate::model::spoken_act(None, "catch", &rest),
            results,
        );
    }

    fn apply_fishes(&mut self, fishes: Vec<Intent>, results: &mut Vec<CommandResult>) {
        for intent in fishes {
            if self.hook_fish_cast(&intent) {
                self.apply_hook_fish(intent, results);
                continue;
            }
            let cell = self
                .actor(intent.actor_id)
                .map(|actor| actor.cell)
                .unwrap_or(0);
            if self
                .brook_stretch(cell)
                .is_some_and(|stretch| self.brook_is_quiet(stretch))
            {
                self.finish(
                    intent.command_id,
                    intent.actor_id,
                    false,
                    "The fish are gone from this stretch of the brook.",
                    results,
                );
                continue;
            }
            if let Some(stretch) = self.brook_stretch(cell) {
                let until = self.time.saturating_add(self.tuning.clay_ticks);
                self.brook_quiet.insert(stretch, until);
            }
            let into_basket = self.carry_count(intent.actor_id) >= self.tuning.carry_slots
                && self.basket_has_room(intent.actor_id);
            let carrier =
                if self.carry_count(intent.actor_id) < self.tuning.carry_slots || into_basket {
                    Some(intent.actor_id)
                } else {
                    None
                };
            let id = self.mint_item(Material::Fish, 0, cell, carrier, Role::Loose);
            if into_basket {
                if let Some(fish) = self.items.get_mut(&id) {
                    fish.in_basket = true;
                }
            }
            let name = self.name_of(intent.actor_id);
            let with_net = self.fish_spear_refusal(intent.actor_id).is_some();
            let (stem, rest) = if with_net {
                ("take", "a fish from the brook with the net.")
            } else {
                ("spear", "a fish from the brook.")
            };
            self.emit(
                crate::model::spoken_act(Some(name.as_str()), stem, rest),
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
                crate::model::spoken_act(None, stem, rest),
                results,
            );
        }
    }

    fn cook_fish(&mut self, intent: Intent, results: &mut Vec<CommandResult>) {
        if self.cook_refusal(intent.actor_id).is_some() {
            let message = self
                .cook_refusal(intent.actor_id)
                .unwrap_or_else(|| "There is no fire burning here.".into());
            self.finish(intent.command_id, intent.actor_id, false, message, results);
            return;
        }
        let Some(id) = self.raw_fish_id(intent.actor_id) else {
            self.finish(
                intent.command_id,
                intent.actor_id,
                false,
                "There is nothing here to cook.",
                results,
            );
            return;
        };
        if let Some(fish) = self.items.get_mut(&id) {
            fish.cooked = true;
            fish.spoiled = false;
            fish.spoils_at = None;
        }
        let cell = self
            .actor(intent.actor_id)
            .map(|actor| actor.cell)
            .unwrap_or(0);
        let rest = "the fish over the fire.";
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

    fn eat_fish(&mut self, intent: Intent, results: &mut Vec<CommandResult>) {
        let salted = self.salted_fish_id(intent.actor_id);
        let smoked = self.smoked_fish_id(intent.actor_id);
        let Some(id) = self.cooked_fish_id(intent.actor_id).or(salted).or(smoked) else {
            let message = if self.raw_fish_near(intent.actor_id) {
                "Raw fish is not food."
            } else if self.spoiled_fish_near(intent.actor_id) {
                "The fish has spoiled."
            } else {
                "There is no cooked fish here to eat."
            };
            self.finish(intent.command_id, intent.actor_id, false, message, results);
            return;
        };
        let (salted_bite, smoked_bite) = self.item(id).map_or((false, false), |fish| {
            (
                fish.salted && !fish.cooked && !fish.smoked,
                fish.smoked && !fish.cooked,
            )
        });
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
        let rest = if salted_bite {
            "the salted fish."
        } else if smoked_bite {
            "the smoked fish."
        } else {
            "the cooked fish."
        };
        self.emit(
            crate::model::spoken_act(Some(name.as_str()), "eat", rest),
            Scope::Cell(cell),
            vec![intent.actor_id],
            vec![cell],
            vec![],
            vec![],
            None,
        );
        let eased = self.hunger_word(before) != "not hungry"
            && self.hunger_word(after) != self.hunger_word(before);
        let mut message = crate::model::spoken_act(None, "eat", rest);
        if eased {
            message.push_str(" The hunger eases.");
        } else {
            message.push_str(&format!(" You are {}.", self.hunger_word(after)));
        }
        self.finish(intent.command_id, intent.actor_id, true, message, results);
    }

    fn apply_hafts(&mut self, hafts: Vec<Intent>, results: &mut Vec<CommandResult>) {
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
            let strips = self
                .loose_ids(intent.actor_id, Material::Cord)
                .first()
                .copied();
            let willow = if strips.is_none() {
                self.loose_ids(intent.actor_id, Material::Willow)
                    .first()
                    .copied()
            } else {
                None
            };
            self.items.remove(&flake);
            self.items.remove(&branch);
            if let Some(strips) = strips {
                self.items.remove(&strips);
            }
            if let Some(willow) = willow {
                self.items.remove(&willow);
            }
            let id = self.mint_item(Material::Spear, 0, cell, Some(intent.actor_id), Role::Loose);
            if let Some(spear) = self.items.get_mut(&id) {
                spear.flint_tip = true;
                spear.bound = strips.is_some() || willow.is_some();
                spear.bound_willow = willow.is_some();
            }
            let rest = if strips.is_some() {
                "the flint flake and a dry branch into a spear. The tip is bound with hide."
            } else if willow.is_some() {
                "the flint flake and a dry branch into a spear. The tip is bound with willow cordage."
            } else {
                "the flint flake and a dry branch into a spear."
            };
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

    fn spear_can_hunt(&self, actor: crate::model::ActorId) -> bool {
        self.loose_ids(actor, Material::Spear)
            .iter()
            .any(|id| self.item(*id).is_some_and(|spear| spear.hardened))
    }

    /// A bow and a sharp flint flake, and the deer is still inside sight.
    /// The flake is not spent. A hardened spear on the deer or the next cell
    /// does not ask this.
    fn bow_can_reach(&self, actor: crate::model::ActorId) -> bool {
        let Some(person) = self.actor(actor) else {
            return false;
        };
        if self.loose_ids(actor, Material::Bow).is_empty()
            || self.loose_ids(actor, Material::Flake).is_empty()
        {
            return false;
        }
        self.deer.as_ref().is_some_and(|deer| {
            crate::model::chebyshev(person.cell, deer.cell) <= self.tuning.sight
        })
    }

    fn bow_in_hand(&self, actor: crate::model::ActorId) -> bool {
        !self.loose_ids(actor, Material::Bow).is_empty()
            && !self.loose_ids(actor, Material::Flake).is_empty()
    }

    /// The wolf is the nearer hunt, so the deer is left for another cast.
    fn wolf_before_the_deer(&self, actor: crate::model::ActorId) -> bool {
        let Some(person) = self.actor(actor) else {
            return false;
        };
        let Some(wolf) = self.wolf_standing() else {
            return false;
        };
        let wolf_dist = crate::model::chebyshev(person.cell, wolf);
        match self.deer.as_ref() {
            Some(deer) => wolf_dist < crate::model::chebyshev(person.cell, deer.cell),
            None => true,
        }
    }

    /// A spear on the wolf's cell or the next, or a bow out to sight.
    /// False when this hunter cannot reach it, so the deer path can speak.
    fn strike_the_wolf(
        &mut self,
        intent: &Intent,
        actor: &crate::model::Actor,
        results: &mut Vec<CommandResult>,
    ) -> bool {
        let Some(cell) = self.wolf_standing() else {
            return false;
        };
        let dist = crate::model::chebyshev(actor.cell, cell);
        if self.spear_can_hunt(actor.id) && dist == 0 {
            self.bring_down_wolf(intent, cell, None, results);
            return true;
        }
        if self.spear_can_hunt(actor.id) && dist == 1 {
            self.contest_the_wolf(intent, actor, cell, results);
            return true;
        }
        if self.bow_in_hand(actor.id) && dist <= self.tuning.sight {
            self.loose_the_bow_at_the_wolf(intent, actor, cell, results);
            return true;
        }
        false
    }

    fn bring_down_wolf(
        &mut self,
        intent: &Intent,
        cell: crate::model::CellId,
        draw: Option<Draw>,
        results: &mut Vec<CommandResult>,
    ) {
        let name = self.name_of(intent.actor_id);
        let flint = self
            .loose_ids(intent.actor_id, Material::Spear)
            .iter()
            .any(|id| self.item(*id).is_some_and(|spear| spear.flint_tip));
        self.wolf_cell = None;
        let id = self.mint_item(Material::Carcass, 0, cell, None, Role::Loose);
        if let Some(carcass) = self.items.get_mut(&id) {
            carcass.wolf = true;
        }
        let (heard, yours) = if flint {
            let rest = "the wolf down with the flint spear.";
            (
                crate::model::spoken_act(Some(name.as_str()), "bring", rest),
                crate::model::spoken_act(None, "bring", rest),
            )
        } else {
            (
                format!("{name} brings the wolf down. A wolf carcass lies where it stood."),
                "You bring the wolf down. A wolf carcass lies where it stood.".to_string(),
            )
        };
        self.emit(
            heard,
            Scope::Cell(cell),
            vec![intent.actor_id],
            vec![cell],
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

    fn loose_the_bow_at_the_wolf(
        &mut self,
        intent: &Intent,
        actor: &crate::model::Actor,
        cell: crate::model::CellId,
        results: &mut Vec<CommandResult>,
    ) {
        if let Some(flake) = self.loose_ids(actor.id, Material::Flake).first().copied() {
            self.items.remove(&flake);
        }
        self.wolf_cell = None;
        let id = self.mint_item(Material::Carcass, 0, cell, None, Role::Loose);
        if let Some(carcass) = self.items.get_mut(&id) {
            carcass.wolf = true;
        }
        let name = self.name_of(actor.id);
        let rest = "the wolf down with the bow. The flint flake is spent.";
        self.emit(
            crate::model::spoken_act(Some(name.as_str()), "bring", rest),
            Scope::Cell(cell),
            vec![actor.id],
            vec![cell],
            vec![id],
            vec![],
            None,
        );
        self.finish(
            intent.command_id.clone(),
            actor.id,
            true,
            crate::model::spoken_act(None, "bring", rest),
            results,
        );
    }

    /// The next cell. One of three, from the same draw the deer uses.
    /// A miss or a bolt leaves no carcass. The tracks stay.
    fn contest_the_wolf(
        &mut self,
        intent: &Intent,
        actor: &crate::model::Actor,
        cell: crate::model::CellId,
        results: &mut Vec<CommandResult>,
    ) {
        let (x, y) = crate::model::cell_xy(cell);
        let subject = format!("{x},{y}:wolf");
        let hash = contest_hash(self.seed, self.time, "hunt", &subject, actor.id);
        let draw = Some(Draw {
            tag: "hunt".into(),
            subject,
            value: hash,
        });
        if hash % 3 == 0 {
            self.bring_down_wolf(intent, cell, draw, results);
            return;
        }
        self.wolf_cell = None;
        let name = self.name_of(actor.id);
        let flint = self
            .loose_ids(actor.id, Material::Spear)
            .iter()
            .any(|id| self.item(*id).is_some_and(|spear| spear.flint_tip));
        let (heard, yours) = if hash % 3 == 1 {
            if flint {
                (
                    format!("{name} misses the wolf with the flint spear. It moves off."),
                    "You miss the wolf with the flint spear. It moves off.",
                )
            } else {
                (
                    format!("{name} misses the wolf. It moves off."),
                    "You miss the wolf. It moves off.",
                )
            }
        } else if flint {
            (
                format!("The wolf bolts. {name}'s flint spear does not land."),
                "The wolf bolts. Your flint spear does not land.",
            )
        } else {
            (
                format!("The wolf bolts from {name}'s spear."),
                "The wolf bolts. Your spear does not land.",
            )
        };
        self.emit(
            heard,
            Scope::Cell(cell),
            vec![actor.id],
            vec![cell],
            vec![],
            vec![],
            draw,
        );
        self.finish(intent.command_id.clone(), actor.id, false, yours, results);
    }

    fn apply_hunts(&mut self, mut hunts: Vec<Intent>, results: &mut Vec<CommandResult>) {
        hunts.sort_by_key(|intent| intent.actor_id);
        for intent in hunts {
            let Some(actor) = self.actor(intent.actor_id).cloned() else {
                continue;
            };
            if !self.spear_can_hunt(actor.id) {
                if self.loose_ids(actor.id, Material::Spear).is_empty()
                    && !self.loose_ids(actor.id, Material::Bow).is_empty()
                {
                    self.loose_the_bow(&intent, &actor, results);
                    continue;
                }
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
            if self.wolf_before_the_deer(actor.id) && self.strike_the_wolf(&intent, &actor, results)
            {
                continue;
            }
            let Some(deer) = self.deer.clone() else {
                let wolf_in_sight = self.wolf_standing().is_some_and(|cell| {
                    crate::model::chebyshev(actor.cell, cell) <= self.tuning.sight
                });
                self.finish(
                    intent.command_id,
                    actor.id,
                    false,
                    if wolf_in_sight {
                        "The wolf is not close enough to hunt."
                    } else {
                        "There is no deer to hunt."
                    },
                    results,
                );
                continue;
            };
            let dist = crate::model::chebyshev(actor.cell, deer.cell);
            if dist > 1 {
                if self.bow_can_reach(actor.id) {
                    self.loose_the_bow(&intent, &actor, results);
                } else {
                    self.finish(
                        intent.command_id,
                        actor.id,
                        false,
                        "The deer is not close enough to hunt.",
                        results,
                    );
                }
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
            let wolf = self.item(id).is_some_and(|hide| hide.wolf);
            let rest = worn_phrase(wolf, sewn);
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
            let wolf = self.item(id).is_some_and(|hide| hide.wolf);
            let rest = worn_phrase(wolf, sewn);
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
            // The open wind already refused this rest. What remains is the
            // rock, a lean-to, or a reed mat, and that rest hurries a stomach.
            let hurried = self.tuning.illness_recovery.saturating_mul(5);
            self.ease_illness(intent.actor_id, hurried);
            let shade = self.tuning.sun_per_tick.saturating_mul(5);
            self.ease_sunburn(intent.actor_id, shade);
            let under_rock = self.cell(actor.cell).rock;
            let place = if under_rock {
                " under the rock"
            } else if self.lean_tos.contains(&actor.cell) {
                " under the lean-to"
            } else if self.reed_mat_here(actor.cell) {
                " on the reed mat"
            } else {
                ""
            };
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
            let wolf = self.item(carcass).is_some_and(|carcass| carcass.wolf);
            self.items.remove(&carcass);
            let mut ids = Vec::new();
            for _ in 0..3 {
                ids.push(self.mint_item(Material::Meat, 0, actor.cell, None, Role::Loose));
            }
            let hide = self.mint_item(Material::Hide, 0, actor.cell, None, Role::Loose);
            if wolf {
                if let Some(item) = self.items.get_mut(&hide) {
                    item.wolf = true;
                }
            }
            ids.push(hide);
            ids.push(self.mint_item(Material::Bone, 0, actor.cell, None, Role::Loose));
            let name = self.name_of(actor.id);
            let rest = match (wolf, with_knife) {
                (true, true) => {
                    "the wolf carcass with the iron knife into three portions of raw meat, a hide, and a bone."
                }
                (true, false) => {
                    "the wolf carcass into three portions of raw meat, a hide, and a bone."
                }
                (false, true) => {
                    "the carcass with the iron knife into three portions of raw meat, a hide, and a bone."
                }
                (false, false) => "the carcass into three portions of raw meat, a hide, and a bone.",
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

    fn branch_is_dry(&self, id: crate::model::ItemId) -> bool {
        self.item(id)
            .is_some_and(|branch| branch.moisture < self.tuning.damp_at)
    }

    fn dry_branches_for_lean_to(&self, actor: crate::model::ActorId) -> u32 {
        let cell = self.actor(actor).map(|person| person.cell).unwrap_or(0);
        let carried = self
            .loose_ids(actor, Material::Branch)
            .into_iter()
            .filter(|id| self.branch_is_dry(*id))
            .count() as u32;
        let piled = self
            .piles
            .iter()
            .filter(|pile| {
                pile.cell == cell
                    && pile.material == Material::Branch
                    && pile.moisture < self.tuning.damp_at
            })
            .map(|pile| pile.count)
            .sum::<u32>();
        let lying = self
            .ground_ids(cell, Material::Branch)
            .into_iter()
            .filter(|id| self.branch_is_dry(*id))
            .count() as u32;
        carried + piled + lying
    }

    fn spend_dry_branches(
        &mut self,
        actor: crate::model::ActorId,
        cell: crate::model::CellId,
        need: u32,
    ) {
        let mut left = need;
        let carried: Vec<_> = self
            .loose_ids(actor, Material::Branch)
            .into_iter()
            .filter(|id| self.branch_is_dry(*id))
            .collect();
        for id in carried {
            if left == 0 {
                break;
            }
            self.items.remove(&id);
            left -= 1;
        }
        if left > 0 {
            let piled = self
                .piles
                .iter()
                .find(|pile| {
                    pile.cell == cell
                        && pile.material == Material::Branch
                        && pile.moisture < self.tuning.damp_at
                })
                .map(|pile| pile.count)
                .unwrap_or(0);
            let take = left.min(piled);
            if take > 0 {
                self.take_from_pile(cell, Material::Branch, take);
                left -= take;
            }
        }
        if left > 0 {
            let lying: Vec<_> = self
                .ground_ids(cell, Material::Branch)
                .into_iter()
                .filter(|id| self.branch_is_dry(*id))
                .collect();
            for id in lying {
                if left == 0 {
                    break;
                }
                self.items.remove(&id);
                left -= 1;
            }
        }
    }

    fn apply_lean_tos(&mut self, builds: Vec<Intent>, results: &mut Vec<CommandResult>) {
        for intent in builds {
            let Some(actor) = self.actor(intent.actor_id).cloned() else {
                continue;
            };
            if self.lean_tos.contains(&actor.cell) {
                self.finish(
                    intent.command_id,
                    actor.id,
                    false,
                    "A lean-to already stands here.",
                    results,
                );
                continue;
            }
            if self.dry_branches_for_lean_to(actor.id) < 3 {
                self.finish(
                    intent.command_id,
                    actor.id,
                    false,
                    "You need three dry branches to build a lean-to.",
                    results,
                );
                continue;
            }
            self.spend_dry_branches(actor.id, actor.cell, 3);
            self.lean_tos.insert(actor.cell);
            let name = self.name_of(actor.id);
            let rest = "a lean-to of three branches.";
            self.emit(
                crate::model::spoken_act(Some(name.as_str()), "build", rest),
                Scope::Cell(actor.cell),
                vec![actor.id],
                vec![actor.cell],
                vec![],
                vec![],
                None,
            );
            self.finish(
                intent.command_id,
                actor.id,
                true,
                crate::model::spoken_act(None, "build", rest),
                results,
            );
        }
    }

    fn apply_salts(&mut self, salts: Vec<Intent>, results: &mut Vec<CommandResult>) {
        for intent in salts {
            let Some(actor) = self.actor(intent.actor_id).cloned() else {
                continue;
            };
            if !self.has_salt(actor.id) {
                self.finish(
                    intent.command_id,
                    actor.id,
                    false,
                    "You have no salt.",
                    results,
                );
                continue;
            }
            let Some(meat_id) = self.raw_portion_to_salt(actor.id) else {
                let message = if self.spoiled_meat_near(actor.id) {
                    "The meat has spoiled."
                } else {
                    "There is no raw meat here to salt."
                };
                self.finish(intent.command_id, actor.id, false, message, results);
                continue;
            };
            if !self.take_one_salt(actor.id, actor.cell) {
                self.finish(
                    intent.command_id,
                    actor.id,
                    false,
                    "You have no salt.",
                    results,
                );
                continue;
            }
            let small = self.item(meat_id).is_some_and(|meat| meat.small);
            if let Some(meat) = self.items.get_mut(&meat_id) {
                meat.salted = true;
                meat.spoiled = false;
                meat.spoils_at = None;
            }
            let name = self.name_of(actor.id);
            let rest = if small {
                "a smaller piece of raw meat. It will keep, and it can be eaten cold."
            } else {
                "a portion of raw meat. It will keep, and it can be eaten cold."
            };
            self.emit(
                crate::model::spoken_act(Some(name.as_str()), "salt", rest),
                Scope::Cell(actor.cell),
                vec![actor.id],
                vec![actor.cell],
                vec![meat_id],
                vec![],
                None,
            );
            self.finish(
                intent.command_id,
                actor.id,
                true,
                crate::model::spoken_act(None, "salt", rest),
                results,
            );
        }
    }

    fn apply_salt_fish(&mut self, salts: Vec<Intent>, results: &mut Vec<CommandResult>) {
        for intent in salts {
            let Some(actor) = self.actor(intent.actor_id).cloned() else {
                continue;
            };
            if !self.has_salt(actor.id) {
                self.finish(
                    intent.command_id,
                    actor.id,
                    false,
                    "You have no salt.",
                    results,
                );
                continue;
            }
            let Some(fish_id) = self.raw_fish_id(actor.id) else {
                let message = if self.salted_fish_id(actor.id).is_some() {
                    "The fish is already salted."
                } else if self.spoiled_fish_near(actor.id) {
                    "The fish has spoiled."
                } else {
                    "There is no raw fish here to salt."
                };
                self.finish(intent.command_id, actor.id, false, message, results);
                continue;
            };
            if !self.take_one_salt(actor.id, actor.cell) {
                self.finish(
                    intent.command_id,
                    actor.id,
                    false,
                    "You have no salt.",
                    results,
                );
                continue;
            }
            if let Some(fish) = self.items.get_mut(&fish_id) {
                fish.salted = true;
                fish.spoiled = false;
                fish.spoils_at = None;
            }
            let name = self.name_of(actor.id);
            let rest = "a raw fish. It will keep, and it can be eaten cold.";
            self.emit(
                crate::model::spoken_act(Some(name.as_str()), "salt", rest),
                Scope::Cell(actor.cell),
                vec![actor.id],
                vec![actor.cell],
                vec![fish_id],
                vec![],
                None,
            );
            self.finish(
                intent.command_id,
                actor.id,
                true,
                crate::model::spoken_act(None, "salt", rest),
                results,
            );
        }
    }

    /// Hang a raw fish in the smoke. The fire is not used up. The fish keeps, and it can be eaten cold.
    fn apply_smoke_fish(&mut self, smokes: Vec<Intent>, results: &mut Vec<CommandResult>) {
        for intent in smokes {
            let Some(actor) = self.actor(intent.actor_id).cloned() else {
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
            let Some(fish_id) = self.raw_fish_id(actor.id) else {
                let message = if self.smoked_fish_id(actor.id).is_some() {
                    "The fish is already smoked."
                } else if self.spoiled_fish_near(actor.id) {
                    "The fish has spoiled."
                } else {
                    "There is no raw fish here to smoke."
                };
                self.finish(intent.command_id, actor.id, false, message, results);
                continue;
            };
            if let Some(fish) = self.items.get_mut(&fish_id) {
                fish.smoked = true;
                fish.spoiled = false;
                fish.spoils_at = None;
            }
            let name = self.name_of(actor.id);
            let rest = "a raw fish. It will keep, and it can be eaten cold.";
            self.emit(
                crate::model::spoken_act(Some(name.as_str()), "smoke", rest),
                Scope::Cell(actor.cell),
                vec![actor.id],
                vec![actor.cell],
                vec![fish_id],
                vec![],
                None,
            );
            self.finish(
                intent.command_id,
                actor.id,
                true,
                crate::model::spoken_act(None, "smoke", rest),
                results,
            );
        }
    }

    /// Hang a portion of raw meat in the smoke. The fire is not used up.
    /// The meat keeps, and it can be eaten cold.
    fn apply_smoke_meat(&mut self, smokes: Vec<Intent>, results: &mut Vec<CommandResult>) {
        for intent in smokes {
            let Some(actor) = self.actor(intent.actor_id).cloned() else {
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
            let Some(meat_id) = self.raw_portion_to_salt(actor.id) else {
                let message = if self.smoked_meat_id(actor.id).is_some() {
                    "The meat is already smoked."
                } else if self.spoiled_meat_near(actor.id) {
                    "The meat has spoiled."
                } else {
                    "There is no raw meat here to smoke."
                };
                self.finish(intent.command_id, actor.id, false, message, results);
                continue;
            };
            let small = self.item(meat_id).is_some_and(|meat| meat.small);
            if let Some(meat) = self.items.get_mut(&meat_id) {
                meat.smoked = true;
                meat.spoiled = false;
                meat.spoils_at = None;
            }
            let name = self.name_of(actor.id);
            let rest = if small {
                "a smaller piece of raw meat. It will keep, and it can be eaten cold."
            } else {
                "a portion of raw meat. It will keep, and it can be eaten cold."
            };
            self.emit(
                crate::model::spoken_act(Some(name.as_str()), "smoke", rest),
                Scope::Cell(actor.cell),
                vec![actor.id],
                vec![actor.cell],
                vec![meat_id],
                vec![],
                None,
            );
            self.finish(
                intent.command_id,
                actor.id,
                true,
                crate::model::spoken_act(None, "smoke", rest),
                results,
            );
        }
    }

    pub(crate) fn has_salt(&self, actor: crate::model::ActorId) -> bool {
        let cell = self.actor(actor).map(|person| person.cell).unwrap_or(0);
        !self.loose_ids(actor, Material::Salt).is_empty()
            || !self.ground_ids(cell, Material::Salt).is_empty()
            || self.pile_count(cell, Material::Salt) > 0
    }

    fn take_one_salt(&mut self, actor: crate::model::ActorId, cell: crate::model::CellId) -> bool {
        if let Some(id) = self.loose_ids(actor, Material::Salt).first().copied() {
            self.items.remove(&id);
            return true;
        }
        if let Some(id) = self.ground_ids(cell, Material::Salt).first().copied() {
            self.items.remove(&id);
            self.arm_salt_lick(cell);
            return true;
        }
        if self.pile_count(cell, Material::Salt) > 0 {
            self.take_from_pile(cell, Material::Salt, 1);
            self.arm_salt_lick(cell);
            return true;
        }
        false
    }

    /// A raw unsalted, unsmoked portion in the arms, else one on this cell.
    pub(crate) fn raw_portion_to_salt(
        &self,
        actor: crate::model::ActorId,
    ) -> Option<crate::model::ItemId> {
        let carried = self
            .loose_ids(actor, Material::Meat)
            .into_iter()
            .find(|id| {
                self.item(*id).is_some_and(|meat| {
                    !meat.cooked && !meat.salted && !meat.smoked && !meat.spoiled
                })
            });
        if carried.is_some() {
            return carried;
        }
        let cell = self.actor(actor).map(|person| person.cell).unwrap_or(0);
        self.ground_ids(cell, Material::Meat)
            .into_iter()
            .find(|id| {
                self.item(*id).is_some_and(|meat| {
                    !meat.cooked && !meat.salted && !meat.smoked && !meat.spoiled
                })
            })
    }

    fn apply_carves(&mut self, carves: Vec<Intent>, results: &mut Vec<CommandResult>) {
        for intent in carves {
            let Some(actor) = self.actor(intent.actor_id).cloned() else {
                continue;
            };
            if !self.has_bone(actor.id) {
                self.finish(
                    intent.command_id,
                    actor.id,
                    false,
                    "You have no bone to carve.",
                    results,
                );
                continue;
            }
            let with_knife = !self.loose_ids(actor.id, Material::Knife).is_empty();
            if !with_knife && self.loose_ids(actor.id, Material::Flake).is_empty() {
                self.finish(
                    intent.command_id,
                    actor.id,
                    false,
                    "You need a flint flake or an iron knife to carve the bone.",
                    results,
                );
                continue;
            }
            if !self.take_one_bone(actor.id, actor.cell) {
                self.finish(
                    intent.command_id,
                    actor.id,
                    false,
                    "You have no bone to carve.",
                    results,
                );
                continue;
            }
            let id = self.mint_item(Material::Awl, 0, actor.cell, Some(actor.id), Role::Loose);
            if let Some(awl) = self.items.get_mut(&id) {
                awl.bone = true;
            }
            let name = self.name_of(actor.id);
            let rest = if with_knife {
                "a bone into a bone awl with the iron knife."
            } else {
                "a bone into a bone awl with the flint flake."
            };
            self.emit(
                crate::model::spoken_act(Some(name.as_str()), "carve", rest),
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
                crate::model::spoken_act(None, "carve", rest),
                results,
            );
        }
    }

    pub(crate) fn has_bone(&self, actor: crate::model::ActorId) -> bool {
        let cell = self.actor(actor).map(|person| person.cell).unwrap_or(0);
        !self.loose_ids(actor, Material::Bone).is_empty()
            || !self.ground_ids(cell, Material::Bone).is_empty()
    }

    pub(crate) fn has_carve_tool(&self, actor: crate::model::ActorId) -> bool {
        !self.loose_ids(actor, Material::Knife).is_empty()
            || !self.loose_ids(actor, Material::Flake).is_empty()
    }

    /// A dry branch in the arms, else one lying here, else one from a dry pile.
    pub(crate) fn paddle_branch_refusal(
        &self,
        actor: crate::model::ActorId,
    ) -> Option<&'static str> {
        if self.dry_branch_for_paddle(actor) {
            None
        } else if self.any_branch_for_paddle(actor) {
            Some("The branch is not dry.")
        } else {
            Some("You have no dry branch to carve.")
        }
    }

    fn dry_branch_for_paddle(&self, actor: crate::model::ActorId) -> bool {
        if !self.dry_loose_ids(actor, Material::Branch).is_empty() {
            return true;
        }
        let cell = self.actor(actor).map(|person| person.cell).unwrap_or(0);
        self.ground_ids(cell, Material::Branch).iter().any(|id| {
            self.item(*id)
                .is_some_and(|item| self.is_dry(item.moisture))
        }) || self.dry_pile_count(cell, Material::Branch) > 0
    }

    fn any_branch_for_paddle(&self, actor: crate::model::ActorId) -> bool {
        let cell = self.actor(actor).map(|person| person.cell).unwrap_or(0);
        !self.loose_ids(actor, Material::Branch).is_empty()
            || !self.ground_ids(cell, Material::Branch).is_empty()
            || self.pile_count(cell, Material::Branch) > 0
    }

    /// Knife before flake. Either may be in the arms or lying on this cell. It is not spent.
    pub(crate) fn paddle_tool_rest(&self, actor: crate::model::ActorId) -> Option<&'static str> {
        let cell = self.actor(actor).map(|person| person.cell).unwrap_or(0);
        let knife = !self.loose_ids(actor, Material::Knife).is_empty()
            || !self.ground_ids(cell, Material::Knife).is_empty();
        let flake = !self.loose_ids(actor, Material::Flake).is_empty()
            || !self.ground_ids(cell, Material::Flake).is_empty();
        if knife {
            Some("a dry branch into a wooden paddle with the iron knife.")
        } else if flake {
            Some("a dry branch into a wooden paddle with the flint flake.")
        } else {
            None
        }
    }

    fn take_paddle_branch(&mut self, actor: crate::model::ActorId) -> bool {
        if self.consume_dry(actor, Material::Branch).is_ok() {
            return true;
        }
        let cell = self.actor(actor).map(|person| person.cell).unwrap_or(0);
        let ground = self
            .ground_ids(cell, Material::Branch)
            .into_iter()
            .find(|id| {
                self.item(*id)
                    .is_some_and(|item| self.is_dry(item.moisture))
            });
        if let Some(id) = ground {
            self.items.remove(&id);
            return true;
        }
        false
    }

    fn apply_paddle_carves(&mut self, carves: Vec<Intent>, results: &mut Vec<CommandResult>) {
        for intent in carves {
            let Some(actor) = self.actor(intent.actor_id).cloned() else {
                continue;
            };
            if let Some(message) = self.paddle_branch_refusal(actor.id) {
                self.finish(intent.command_id, actor.id, false, message, results);
                continue;
            }
            let Some(rest) = self.paddle_tool_rest(actor.id) else {
                self.finish(
                    intent.command_id,
                    actor.id,
                    false,
                    "You need a flint flake or an iron knife to carve a paddle.",
                    results,
                );
                continue;
            };
            if !self.take_paddle_branch(actor.id) {
                self.finish(
                    intent.command_id,
                    actor.id,
                    false,
                    "You have no dry branch to carve.",
                    results,
                );
                continue;
            }
            let id = self.mint_item(Material::Paddle, 0, actor.cell, Some(actor.id), Role::Loose);
            let name = self.name_of(actor.id);
            self.emit(
                crate::model::spoken_act(Some(name.as_str()), "carve", rest),
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
                crate::model::spoken_act(None, "carve", rest),
                results,
            );
        }
    }

    /// Knife before flake. Either may be in the arms or lying on this cell. It is not spent.
    pub(crate) fn hook_tool_rest(&self, actor: crate::model::ActorId) -> Option<&'static str> {
        let cell = self.actor(actor).map(|person| person.cell).unwrap_or(0);
        let knife = !self.loose_ids(actor, Material::Knife).is_empty()
            || !self.ground_ids(cell, Material::Knife).is_empty();
        let flake = !self.loose_ids(actor, Material::Flake).is_empty()
            || !self.ground_ids(cell, Material::Flake).is_empty();
        if knife {
            Some("a bone into a bone fish hook with the iron knife.")
        } else if flake {
            Some("a bone into a bone fish hook with the flint flake.")
        } else {
            None
        }
    }

    fn apply_hook_carves(&mut self, carves: Vec<Intent>, results: &mut Vec<CommandResult>) {
        for intent in carves {
            let Some(actor) = self.actor(intent.actor_id).cloned() else {
                continue;
            };
            if !self.has_bone(actor.id) {
                self.finish(
                    intent.command_id,
                    actor.id,
                    false,
                    "You have no bone to carve.",
                    results,
                );
                continue;
            }
            let Some(rest) = self.hook_tool_rest(actor.id) else {
                self.finish(
                    intent.command_id,
                    actor.id,
                    false,
                    "You need a flint flake or an iron knife to carve a hook.",
                    results,
                );
                continue;
            };
            if !self.take_one_bone(actor.id, actor.cell) {
                self.finish(
                    intent.command_id,
                    actor.id,
                    false,
                    "You have no bone to carve.",
                    results,
                );
                continue;
            }
            let id = self.mint_item(Material::Hook, 0, actor.cell, Some(actor.id), Role::Loose);
            let name = self.name_of(actor.id);
            self.emit(
                crate::model::spoken_act(Some(name.as_str()), "carve", rest),
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
                crate::model::spoken_act(None, "carve", rest),
                results,
            );
        }
    }

    fn take_one_bone(&mut self, actor: crate::model::ActorId, cell: crate::model::CellId) -> bool {
        if let Some(id) = self.loose_ids(actor, Material::Bone).first().copied() {
            self.items.remove(&id);
            return true;
        }
        if let Some(id) = self.ground_ids(cell, Material::Bone).first().copied() {
            self.items.remove(&id);
            return true;
        }
        false
    }

    fn apply_snares(&mut self, snares: Vec<Intent>, results: &mut Vec<CommandResult>) {
        for intent in snares {
            let Some(actor) = self.actor(intent.actor_id).cloned() else {
                continue;
            };
            if self.snares.contains_key(&actor.cell) {
                self.finish(
                    intent.command_id,
                    actor.id,
                    false,
                    "A snare is already set here.",
                    results,
                );
                continue;
            }
            let Some(willow) = self.take_snare_line(actor.id, actor.cell) else {
                self.finish(
                    intent.command_id,
                    actor.id,
                    false,
                    "You have no hide strips to set a snare.",
                    results,
                );
                continue;
            };
            let ready = self.time.saturating_add(self.tuning.snare_ticks);
            self.snares.insert(actor.cell, ready);
            if willow {
                self.snare_willow.insert(actor.cell);
            }
            self.schedule(ready, crate::model::Wake::Snare(actor.cell));
            let name = self.name_of(actor.id);
            let rest = if willow {
                "a snare of willow cordage."
            } else {
                "a snare of hide strips."
            };
            self.emit(
                crate::model::spoken_act(Some(name.as_str()), "set", rest),
                Scope::Cell(actor.cell),
                vec![actor.id],
                vec![actor.cell],
                vec![],
                vec![],
                None,
            );
            self.finish(
                intent.command_id,
                actor.id,
                true,
                crate::model::spoken_act(None, "set", rest),
                results,
            );
        }
    }

    /// A carried fishing net leaves the arms and is set in this brook cell.
    fn apply_set_nets(&mut self, sets: Vec<Intent>, results: &mut Vec<CommandResult>) {
        for intent in sets {
            let Some(actor) = self.actor(intent.actor_id).cloned() else {
                continue;
            };
            if self.cell(actor.cell).region != Region::Brook {
                self.finish(
                    intent.command_id,
                    actor.id,
                    false,
                    "You cannot set a net out of the brook.",
                    results,
                );
                continue;
            }
            if self.set_nets.contains(&actor.cell) {
                self.finish(
                    intent.command_id,
                    actor.id,
                    false,
                    "A fishing net is already set here.",
                    results,
                );
                continue;
            }
            let Some(id) = self.loose_ids(actor.id, Material::Net).first().copied() else {
                self.finish(
                    intent.command_id,
                    actor.id,
                    false,
                    "You are not carrying a fishing net.",
                    results,
                );
                continue;
            };
            if let Some(net) = self.items.get_mut(&id) {
                net.carrier = None;
                net.cell = actor.cell;
            }
            let ready = self.time.saturating_add(self.tuning.net_ticks);
            self.set_nets.insert(actor.cell);
            self.net_catches_at.insert(actor.cell, ready);
            self.schedule(ready, crate::model::Wake::NetCatch(actor.cell));
            let name = self.name_of(actor.id);
            let rest = "a fishing net in the brook.";
            self.emit(
                crate::model::spoken_act(Some(name.as_str()), "set", rest),
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
                crate::model::spoken_act(None, "set", rest),
                results,
            );
        }
    }

    pub(crate) fn has_cord(&self, actor: crate::model::ActorId) -> bool {
        let cell = self.actor(actor).map(|person| person.cell).unwrap_or(0);
        !self.loose_ids(actor, Material::Cord).is_empty()
            || !self.ground_ids(cell, Material::Cord).is_empty()
    }

    /// Hide strips if any are here, otherwise a bundle of willow cordage.
    pub(crate) fn has_snare_line(&self, actor: crate::model::ActorId) -> bool {
        self.has_cord(actor) || self.has_willow_cord(actor)
    }

    fn has_willow_cord(&self, actor: crate::model::ActorId) -> bool {
        let cell = self.actor(actor).map(|person| person.cell).unwrap_or(0);
        !self.loose_ids(actor, Material::Willow).is_empty()
            || !self.ground_ids(cell, Material::Willow).is_empty()
    }

    /// `Some(false)` spends hide strips. `Some(true)` spends willow cordage.
    /// Hide strips win when both are held, in the arms or on the ground.
    fn take_snare_line(
        &mut self,
        actor: crate::model::ActorId,
        cell: crate::model::CellId,
    ) -> Option<bool> {
        if self.take_one_cord(actor, cell) {
            return Some(false);
        }
        if let Some(id) = self.loose_ids(actor, Material::Willow).first().copied() {
            self.items.remove(&id);
            return Some(true);
        }
        if let Some(id) = self.ground_ids(cell, Material::Willow).first().copied() {
            self.items.remove(&id);
            return Some(true);
        }
        None
    }

    fn take_one_cord(&mut self, actor: crate::model::ActorId, cell: crate::model::CellId) -> bool {
        if let Some(id) = self.loose_ids(actor, Material::Cord).first().copied() {
            self.items.remove(&id);
            return true;
        }
        if let Some(id) = self.ground_ids(cell, Material::Cord).first().copied() {
            self.items.remove(&id);
            return true;
        }
        false
    }

    fn has_reeds(&self, actor: crate::model::ActorId) -> bool {
        let cell = self.actor(actor).map(|person| person.cell).unwrap_or(0);
        !self.loose_ids(actor, Material::Reed).is_empty()
            || self.pile_count(cell, Material::Reed) > 0
            || !self.ground_ids(cell, Material::Reed).is_empty()
    }

    fn take_one_reed(&mut self, actor: crate::model::ActorId, cell: crate::model::CellId) -> bool {
        if let Some(id) = self.loose_ids(actor, Material::Reed).first().copied() {
            self.items.remove(&id);
            return true;
        }
        if self.pile_count(cell, Material::Reed) > 0
            && self.take_from_pile(cell, Material::Reed, 1).is_some()
        {
            self.arm_reed_bed(cell);
            return true;
        }
        if let Some(id) = self.ground_ids(cell, Material::Reed).first().copied() {
            self.items.remove(&id);
            self.arm_reed_bed(cell);
            return true;
        }
        false
    }

    fn apply_thatches(&mut self, thatches: Vec<Intent>, results: &mut Vec<CommandResult>) {
        for intent in thatches {
            let Some(actor) = self.actor(intent.actor_id).cloned() else {
                continue;
            };
            if !self.lean_tos.contains(&actor.cell) {
                self.finish(
                    intent.command_id,
                    actor.id,
                    false,
                    "There is no lean-to here to thatch.",
                    results,
                );
                continue;
            }
            if self.thatched.contains(&actor.cell) {
                self.finish(
                    intent.command_id,
                    actor.id,
                    false,
                    "The lean-to is already thatched.",
                    results,
                );
                continue;
            }
            if !self.take_one_reed(actor.id, actor.cell) {
                self.finish(
                    intent.command_id,
                    actor.id,
                    false,
                    "You have no reeds to thatch with.",
                    results,
                );
                continue;
            }
            self.thatched.insert(actor.cell);
            let name = self.name_of(actor.id);
            let rest = "the lean-to with reeds.";
            self.emit(
                crate::model::spoken_act(Some(name.as_str()), "thatch", rest),
                Scope::Cell(actor.cell),
                vec![actor.id],
                vec![actor.cell],
                vec![],
                vec![],
                None,
            );
            self.finish(
                intent.command_id,
                actor.id,
                true,
                crate::model::spoken_act(None, "thatch", rest),
                results,
            );
        }
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

    pub(crate) fn basket_has_room(&self, actor: crate::model::ActorId) -> bool {
        !self.loose_ids(actor, Material::Basket).is_empty()
            && !self
                .items
                .values()
                .any(|item| item.carrier == Some(actor) && item.in_basket)
    }

    pub(crate) fn basket_can_hold_berries(&self, actor: crate::model::ActorId) -> bool {
        self.basket_has_room(actor)
    }

    fn basket_holds_berries(&self, actor: crate::model::ActorId) -> bool {
        self.items.values().any(|item| {
            item.carrier == Some(actor) && item.material == Material::Berry && item.in_basket
        })
    }

    fn basket_holds_fish(&self, actor: crate::model::ActorId) -> bool {
        self.items.values().any(|item| {
            item.carrier == Some(actor) && item.material == Material::Fish && item.in_basket
        })
    }

    /// A raw fish in the arms, not the one already sitting in the basket.
    pub(crate) fn raw_fish_in_arms(
        &self,
        actor: crate::model::ActorId,
    ) -> Option<crate::model::ItemId> {
        self.loose_ids(actor, Material::Fish)
            .into_iter()
            .find(|id| {
                self.item(*id)
                    .is_some_and(|fish| !fish.cooked && !fish.in_basket)
            })
    }

    fn gather_overflows_into_basket(
        &self,
        actor: crate::model::ActorId,
        material: Material,
    ) -> bool {
        if !self.basket_has_room(actor) {
            return false;
        }
        match material {
            Material::Berry => true,
            Material::Fish => self
                .actor(actor)
                .map(|person| person.cell)
                .is_some_and(|cell| {
                    self.ground_ids(cell, Material::Fish)
                        .first()
                        .and_then(|id| self.item(*id))
                        .is_some_and(|fish| !fish.cooked)
                }),
            _ => false,
        }
    }

    fn tuck_overflow_into_basket(
        &mut self,
        actor: crate::model::ActorId,
        item_id: crate::model::ItemId,
    ) {
        if self.carry_count(actor) <= self.tuning.carry_slots {
            return;
        }
        if !self.basket_has_room(actor) {
            return;
        }
        let fits = self.item(item_id).is_some_and(|item| {
            item.material == Material::Berry || (item.material == Material::Fish && !item.cooked)
        });
        if !fits {
            return;
        }
        if let Some(item) = self.items.get_mut(&item_id) {
            item.in_basket = true;
        }
    }

    fn apply_puts(&mut self, puts: Vec<Intent>, results: &mut Vec<CommandResult>) {
        let rest = "the raw fish in the bark basket.";
        for intent in puts {
            let Some(id) = self.raw_fish_in_arms(intent.actor_id) else {
                self.finish(
                    intent.command_id,
                    intent.actor_id,
                    false,
                    "You are not carrying a raw fish.",
                    results,
                );
                continue;
            };
            if let Some(fish) = self.items.get_mut(&id) {
                fish.in_basket = true;
            }
            let cell = self
                .actor(intent.actor_id)
                .map(|actor| actor.cell)
                .unwrap_or(0);
            let name = self.name_of(intent.actor_id);
            self.emit(
                crate::model::spoken_act(Some(name.as_str()), "put", rest),
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
                crate::model::spoken_act(None, "put", rest),
                results,
            );
        }
    }

    fn apply_baskets(&mut self, weaves: Vec<Intent>, results: &mut Vec<CommandResult>) {
        for intent in weaves {
            let Some(actor) = self.actor(intent.actor_id).cloned() else {
                continue;
            };
            let bark = if let Some(id) = self.loose_ids(actor.id, Material::Bark).first().copied() {
                self.items.remove(&id);
                vec![id]
            } else if self.take_from_pile(actor.cell, Material::Bark, 1).is_some() {
                Vec::new()
            } else {
                self.finish(
                    intent.command_id,
                    actor.id,
                    false,
                    "You need a sheet of bark to weave a basket.",
                    results,
                );
                continue;
            };
            let id = self.mint_item(Material::Basket, 0, actor.cell, Some(actor.id), Role::Loose);
            let name = self.name_of(actor.id);
            let rest = "a bark basket.";
            let mut affected = bark;
            affected.push(id);
            self.emit(
                crate::model::spoken_act(Some(name.as_str()), "weave", rest),
                Scope::Cell(actor.cell),
                vec![actor.id],
                vec![actor.cell],
                affected,
                vec![],
                None,
            );
            self.finish(
                intent.command_id,
                actor.id,
                true,
                crate::model::spoken_act(None, "weave", rest),
                results,
            );
        }
    }

    /// Three bundles of reeds, from the arms first and then the pile at the
    /// feet, become one fishing net. The net stays in the arms.
    fn apply_nets(&mut self, weaves: Vec<Intent>, results: &mut Vec<CommandResult>) {
        for intent in weaves {
            let Some(actor) = self.actor(intent.actor_id).cloned() else {
                continue;
            };
            let mut spent = Vec::new();
            let mut left = 3u32;
            for id in self.loose_ids(actor.id, Material::Reed) {
                if left == 0 {
                    break;
                }
                self.items.remove(&id);
                spent.push(id);
                left -= 1;
            }
            if left > 0 {
                if self
                    .take_from_pile(actor.cell, Material::Reed, left)
                    .is_none()
                {
                    self.finish(
                        intent.command_id,
                        actor.id,
                        false,
                        "You need three bundles of reeds to weave a net.",
                        results,
                    );
                    continue;
                }
                self.arm_reed_bed(actor.cell);
            }
            let id = self.mint_item(Material::Net, 0, actor.cell, Some(actor.id), Role::Loose);
            spent.push(id);
            let name = self.name_of(actor.id);
            let rest = "a fishing net.";
            self.emit(
                crate::model::spoken_act(Some(name.as_str()), "weave", rest),
                Scope::Cell(actor.cell),
                vec![actor.id],
                vec![actor.cell],
                spent,
                vec![],
                None,
            );
            self.finish(
                intent.command_id,
                actor.id,
                true,
                crate::model::spoken_act(None, "weave", rest),
                results,
            );
        }
    }

    /// Two bundles of reeds, from the arms first and then the pile at the
    /// feet, become one reed mat. The mat stays in the arms until it is set
    /// or dropped.
    fn apply_mats(&mut self, weaves: Vec<Intent>, results: &mut Vec<CommandResult>) {
        for intent in weaves {
            let Some(actor) = self.actor(intent.actor_id).cloned() else {
                continue;
            };
            let mut spent = Vec::new();
            let mut left = 2u32;
            for id in self.loose_ids(actor.id, Material::Reed) {
                if left == 0 {
                    break;
                }
                self.items.remove(&id);
                spent.push(id);
                left -= 1;
            }
            if left > 0 {
                if self
                    .take_from_pile(actor.cell, Material::Reed, left)
                    .is_none()
                {
                    self.finish(
                        intent.command_id,
                        actor.id,
                        false,
                        "You need two bundles of reeds to weave a mat.",
                        results,
                    );
                    continue;
                }
                self.arm_reed_bed(actor.cell);
            }
            let id = self.mint_item(Material::Mat, 0, actor.cell, Some(actor.id), Role::Loose);
            spent.push(id);
            let name = self.name_of(actor.id);
            let rest = "a reed mat.";
            self.emit(
                crate::model::spoken_act(Some(name.as_str()), "weave", rest),
                Scope::Cell(actor.cell),
                vec![actor.id],
                vec![actor.cell],
                spent,
                vec![],
                None,
            );
            self.finish(
                intent.command_id,
                actor.id,
                true,
                crate::model::spoken_act(None, "weave", rest),
                results,
            );
        }
    }

    /// A carried reed mat leaves the arms and lies on this ground.
    fn apply_set_mats(&mut self, sets: Vec<Intent>, results: &mut Vec<CommandResult>) {
        for intent in sets {
            let Some(actor) = self.actor(intent.actor_id).cloned() else {
                continue;
            };
            if self.reed_mat_here(actor.cell) {
                self.finish(
                    intent.command_id,
                    actor.id,
                    false,
                    "A reed mat is already set here.",
                    results,
                );
                continue;
            }
            let Some(id) = self.loose_ids(actor.id, Material::Mat).first().copied() else {
                self.finish(
                    intent.command_id,
                    actor.id,
                    false,
                    "You are not carrying a reed mat.",
                    results,
                );
                continue;
            };
            if let Some(mat) = self.items.get_mut(&id) {
                mat.carrier = None;
                mat.cell = actor.cell;
            }
            let name = self.name_of(actor.id);
            let rest = "a reed mat.";
            self.emit(
                crate::model::spoken_act(Some(name.as_str()), "set", rest),
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
                crate::model::spoken_act(None, "set", rest),
                results,
            );
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
                self.arm_clay_bank(actor.cell);
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

    pub(crate) fn whole_hide_near(&self, actor: crate::model::ActorId) -> bool {
        self.whole_hide_id(actor).is_some()
    }

    fn hide_near(&self, actor: crate::model::ActorId) -> bool {
        let cell = self.actor(actor).map(|person| person.cell).unwrap_or(0);
        !self.loose_ids(actor, Material::Hide).is_empty()
            || !self.ground_ids(cell, Material::Hide).is_empty()
    }

    fn whole_hide_id(&self, actor: crate::model::ActorId) -> Option<crate::model::ItemId> {
        let whole = |id: &crate::model::ItemId| {
            self.item(*id)
                .is_some_and(|hide| !hide.pierced && !hide.sewn && !hide.worn)
        };
        if let Some(id) = self
            .loose_ids(actor, Material::Hide)
            .into_iter()
            .find(|id| whole(id))
        {
            return Some(id);
        }
        let cell = self.actor(actor).map(|person| person.cell).unwrap_or(0);
        self.ground_ids(cell, Material::Hide)
            .into_iter()
            .find(|id| whole(id))
    }

    fn apply_hide_cuts(&mut self, cuts: Vec<Intent>, results: &mut Vec<CommandResult>) {
        for intent in cuts {
            let Some(actor) = self.actor(intent.actor_id).cloned() else {
                continue;
            };
            if self.loose_ids(actor.id, Material::Knife).is_empty() {
                self.finish(
                    intent.command_id,
                    actor.id,
                    false,
                    "You need an iron knife to cut the hide.",
                    results,
                );
                continue;
            }
            let Some(id) = self.whole_hide_id(actor.id) else {
                let message = if self.hide_near(actor.id) {
                    "You have no whole deer hide to cut."
                } else {
                    "You have no deer hide to cut."
                };
                self.finish(intent.command_id, actor.id, false, message, results);
                continue;
            };
            let Some(hide) = self.item(id).cloned() else {
                continue;
            };
            self.items.remove(&id);
            let strips = self.mint_item(
                Material::Cord,
                hide.moisture,
                hide.cell,
                hide.carrier,
                Role::Loose,
            );
            let name = self.name_of(actor.id);
            let rest = "the deer hide into strips with the iron knife.";
            self.emit(
                crate::model::spoken_act(Some(name.as_str()), "cut", rest),
                Scope::Cell(actor.cell),
                vec![actor.id],
                vec![actor.cell],
                vec![strips],
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
                    item.salted = meat.salted;
                    item.smoked = meat.smoked;
                    item.small = true;
                    item.spoiled = meat.spoiled;
                    if meat.cooked || meat.salted || meat.smoked || meat.spoiled {
                        item.spoils_at = None;
                    } else {
                        item.spoils_at = meat.spoils_at;
                    }
                }
                if !meat.cooked && !meat.salted && !meat.smoked && !meat.spoiled {
                    if let Some(at) = meat.spoils_at {
                        self.schedule(at, Wake::Spoil(piece));
                    }
                }
                ids.push(piece);
            }
            let name = self.name_of(actor.id);
            let rest = if meat.cooked {
                "the cooked meat into two smaller pieces with the iron knife."
            } else if meat.salted {
                "the salted meat into two smaller pieces with the iron knife."
            } else if meat.smoked {
                "the smoked meat into two smaller pieces with the iron knife."
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

    /// A carried bow and a sharp flint flake. The deer can stand as far as
    /// sight. A kill spends one flake. A refusal leaves it. The bow stays.
    /// The spear's sentences are not used.
    fn loose_the_bow(
        &mut self,
        intent: &Intent,
        actor: &crate::model::Actor,
        results: &mut Vec<CommandResult>,
    ) {
        if self.loose_ids(actor.id, Material::Flake).is_empty() {
            self.finish(
                intent.command_id.clone(),
                actor.id,
                false,
                "You need a sharp flint flake.",
                results,
            );
            return;
        }
        if self.wolf_before_the_deer(actor.id) {
            if let Some(cell) = self.wolf_standing() {
                if crate::model::chebyshev(actor.cell, cell) <= self.tuning.sight {
                    self.loose_the_bow_at_the_wolf(intent, actor, cell, results);
                    return;
                }
            }
        }
        let Some(deer) = self.deer.clone() else {
            self.finish(
                intent.command_id.clone(),
                actor.id,
                false,
                "There is no deer to hunt.",
                results,
            );
            return;
        };
        if crate::model::chebyshev(actor.cell, deer.cell) > self.tuning.sight {
            self.finish(
                intent.command_id.clone(),
                actor.id,
                false,
                "The deer is not close enough for the bow.",
                results,
            );
            return;
        }
        let name = self.name_of(actor.id);
        if let Some(flake) = self.loose_ids(actor.id, Material::Flake).first().copied() {
            self.items.remove(&flake);
        }
        self.tracks.insert(deer.cell);
        self.leave_fresh_tracks(deer.cell);
        self.deer = None;
        self.arm_deer_return();
        let id = self.mint_item(Material::Carcass, 0, deer.cell, None, Role::Loose);
        let rest = "the deer down with the bow. The flint flake is spent.";
        self.emit(
            crate::model::spoken_act(Some(name.as_str()), "bring", rest),
            Scope::Cell(deer.cell),
            vec![actor.id],
            vec![deer.cell],
            vec![id],
            vec![],
            None,
        );
        self.finish(
            intent.command_id.clone(),
            actor.id,
            true,
            crate::model::spoken_act(None, "bring", rest),
            results,
        );
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
        self.leave_fresh_tracks(deer_cell);
        self.deer = None;
        self.arm_deer_return();
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
        self.leave_fresh_tracks(deer.cell);
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
            let rest = if let Some(item_id) = self
                .loose_ids(intent.actor_id, Material::Charcoal)
                .first()
                .copied()
            {
                self.items.remove(&item_id);
                "a lump of charcoal into the fire. It burns hotter."
            } else if self.consume_dry(intent.actor_id, Material::Branch).is_ok() {
                "a dry branch into the fire. It burns hotter."
            } else {
                self.finish(
                    intent.command_id,
                    intent.actor_id,
                    false,
                    "You have nothing to feed the fire.",
                    results,
                );
                continue;
            };
            if let Some(fire) = self.fires.values_mut().find(|fire| fire.cell == cell) {
                fire.hot = true;
            }
            let name = self.name_of(intent.actor_id);
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
        for intent in pierces {
            let awl = self
                .loose_ids(intent.actor_id, Material::Awl)
                .first()
                .copied();
            let bone_awl = awl.is_some_and(|id| self.item(id).is_some_and(|awl| awl.bone));
            let hide = self
                .loose_ids(intent.actor_id, Material::Hide)
                .into_iter()
                .find(|id| self.item(*id).is_some_and(|hide| !hide.pierced));
            let (Some(_awl), Some(hide)) = (awl, hide) else {
                let message = if self.carrying_wolf_hide(intent.actor_id) {
                    "You need a copper awl and a wolf hide."
                } else {
                    "You need a copper awl and a deer hide."
                };
                self.finish(intent.command_id, intent.actor_id, false, message, results);
                continue;
            };
            let wolf = self.item(hide).is_some_and(|hide| hide.wolf);
            let rest = match (wolf, bone_awl) {
                (true, true) => "the wolf hide with the bone awl. It is ready to sew.",
                (true, false) => "the wolf hide with the copper awl. It is ready to sew.",
                (false, true) => "the deer hide with the bone awl. It is ready to sew.",
                (false, false) => "the deer hide with the copper awl. It is ready to sew.",
            };
            if let Some(item) = self.items.get_mut(&hide) {
                item.pierced = true;
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
        for intent in sews {
            let hide = self
                .loose_ids(intent.actor_id, Material::Hide)
                .into_iter()
                .find(|id| self.item(*id).is_some_and(|hide| hide.pierced));
            let Some(hide) = hide else {
                let message = if self.carrying_wolf_hide(intent.actor_id) {
                    "The wolf hide is not pierced."
                } else {
                    "The deer hide is not pierced."
                };
                self.finish(intent.command_id, intent.actor_id, false, message, results);
                continue;
            };
            let wolf = self.item(hide).is_some_and(|hide| hide.wolf);
            let rest = if wolf {
                "the pierced wolf hide into a cloak."
            } else {
                "the pierced deer hide into a cloak."
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
            if material == Material::Fish {
                self.eat_fish(intent, results);
                continue;
            }
            if material == Material::Mushroom {
                self.eat_mushroom(intent, results);
                continue;
            }
            if material == Material::Honey {
                self.eat_honey(intent, results);
                continue;
            }
            if material == Material::Egg {
                self.eat_eggs(intent, results);
                continue;
            }
            if material == Material::BitterBerry {
                self.eat_poison(intent, results);
                continue;
            }
            if material == Material::PaleBerry {
                self.eat_pale(intent, results);
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

    fn eat_pale(&mut self, intent: Intent, results: &mut Vec<CommandResult>) {
        if let Some(id) = self.cooked_pale_id(intent.actor_id) {
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
            let rest = "the cooked pale berries.";
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
            return;
        }
        if !self.raw_pale_near(intent.actor_id) {
            self.finish(
                intent.command_id,
                intent.actor_id,
                false,
                "There are no pale berries here to eat.",
                results,
            );
            return;
        }
        let Ok(item) = self.take_hand(intent.actor_id, Material::PaleBerry) else {
            self.finish(
                intent.command_id,
                intent.actor_id,
                false,
                "There are no pale berries here to eat.",
                results,
            );
            return;
        };
        if let Some(person) = self.actors.get_mut(&intent.actor_id) {
            person.pale_berry_known = true;
        }
        let cell = self
            .actor(intent.actor_id)
            .map(|actor| actor.cell)
            .unwrap_or(0);
        let name = self.name_of(intent.actor_id);
        let rest = "a handful of pale berries.";
        self.emit(
            crate::model::spoken_act(Some(name.as_str()), "eat", rest),
            Scope::Cell(cell),
            vec![intent.actor_id],
            vec![cell],
            item.into_iter().collect(),
            vec![],
            None,
        );
        self.finish(
            intent.command_id,
            intent.actor_id,
            true,
            "You eat a handful of pale berries. They do not feed you. They must be cooked.",
            results,
        );
    }

    fn eat_honey(&mut self, intent: Intent, results: &mut Vec<CommandResult>) {
        let Some(id) = self
            .loose_ids(intent.actor_id, Material::Honey)
            .first()
            .copied()
        else {
            self.finish(
                intent.command_id,
                intent.actor_id,
                false,
                "You are not carrying honey.",
                results,
            );
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
        let rest = "the honeycomb.";
        self.emit(
            crate::model::spoken_act(Some(name.as_str()), "eat", rest),
            Scope::Cell(cell),
            vec![intent.actor_id],
            vec![cell],
            vec![],
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

    fn eat_eggs(&mut self, intent: Intent, results: &mut Vec<CommandResult>) {
        let Some(id) = self.cooked_egg_id(intent.actor_id) else {
            let message = if self.raw_egg_near(intent.actor_id) {
                "The eggs are raw."
            } else {
                "There are no cooked eggs here to eat."
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
        let rest = "the cooked eggs.";
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

    fn eat_mushroom(&mut self, intent: Intent, results: &mut Vec<CommandResult>) {
        let Some(id) = self.cooked_mushroom_id(intent.actor_id) else {
            let raw = self.raw_mushroom_near(intent.actor_id);
            if raw {
                if let Some(person) = self.actors.get_mut(&intent.actor_id) {
                    person.raw_mushroom_known = true;
                }
            }
            let message = if raw {
                RAW_MUSHROOM_REFUSAL
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
        let salted = self.salted_meat_id(intent.actor_id);
        let smoked = self.smoked_meat_id(intent.actor_id);
        let Some(id) = self.cooked_meat_id(intent.actor_id).or(salted).or(smoked) else {
            let message = if self.raw_meat_near(intent.actor_id) {
                "Raw meat is not food."
            } else if self.spoiled_meat_near(intent.actor_id) {
                "The meat has spoiled."
            } else {
                "There is no cooked meat here to eat."
            };
            self.finish(intent.command_id, intent.actor_id, false, message, results);
            return;
        };
        let (salted_bite, smoked_bite) = self.item(id).map_or((false, false), |meat| {
            (
                meat.salted && !meat.cooked && !meat.smoked,
                meat.smoked && !meat.cooked,
            )
        });
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
        let rest = if salted_bite {
            "the salted meat."
        } else if smoked_bite {
            "the smoked meat."
        } else {
            "the cooked meat."
        };
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
                format!(
                    "There are no {} here to eat.",
                    self.bitter_kind(intent.actor_id)
                ),
                results,
            );
            return;
        }
        let Ok(item) = self.take_hand(intent.actor_id, Material::BitterBerry) else {
            self.finish(
                intent.command_id,
                intent.actor_id,
                false,
                format!(
                    "There are no {} here to eat.",
                    self.bitter_kind(intent.actor_id)
                ),
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
            person.tasted_bitter = true;
        }
        let cell = self
            .actor(intent.actor_id)
            .map(|actor| actor.cell)
            .unwrap_or(0);
        let name = self.name_of(intent.actor_id);
        let eaten = self.emit(
            format!("{name} eats a handful of dark berries."),
            Scope::Cell(cell),
            vec![intent.actor_id],
            vec![cell],
            item.into_iter().collect(),
            vec![],
            None,
        );
        let turned = illness_before < self.tuning.ill_at && illness_after >= self.tuning.ill_at;
        if turned {
            // The sickness is this body's. The name is not said back,
            // and nobody else is on the personal scope.
            self.emit(
                "Your stomach turns.".to_string(),
                Scope::Personal(intent.actor_id),
                vec![intent.actor_id],
                vec![cell],
                vec![],
                vec![eaten],
                None,
            );
        }
        let message = if turned {
            "You eat a handful of poison berries. They taste bitter. Your stomach turns."
        } else {
            "You eat a handful of poison berries. They taste bitter. You are still ill."
        };
        self.finish(intent.command_id, intent.actor_id, true, message, results);
    }

    fn bow_cell(&self, actor: crate::model::ActorId) -> crate::model::CellId {
        self.actor(actor).map(|person| person.cell).unwrap_or(0)
    }

    /// A dry branch for the bow. One in the arms is spent before one lying in the cell.
    fn bow_branch_id(&self, actor: crate::model::ActorId) -> Option<crate::model::ItemId> {
        if let Some(id) = self.dry_loose_ids(actor, Material::Branch).first().copied() {
            return Some(id);
        }
        self.ground_ids(self.bow_cell(actor), Material::Branch)
            .into_iter()
            .find(|id| {
                self.item(*id)
                    .is_some_and(|item| self.is_dry(item.moisture))
            })
    }

    /// Hide strips for the bow. A bundle in the arms is spent before one lying in the cell.
    fn bow_strips_id(&self, actor: crate::model::ActorId) -> Option<crate::model::ItemId> {
        if let Some(id) = self.loose_ids(actor, Material::Cord).first().copied() {
            return Some(id);
        }
        self.ground_ids(self.bow_cell(actor), Material::Cord)
            .first()
            .copied()
    }

    pub(crate) fn bow_refusal(&self, actor: crate::model::ActorId) -> Option<&'static str> {
        let cell = self.bow_cell(actor);
        let dry = self.bow_branch_id(actor).is_some();
        let any_branch = !self.loose_ids(actor, Material::Branch).is_empty()
            || !self.ground_ids(cell, Material::Branch).is_empty();
        let strips = self.bow_strips_id(actor).is_some();
        if dry && strips {
            None
        } else if any_branch && strips {
            Some("The branch is not dry.")
        } else if !dry && !strips {
            Some("You need a dry branch and hide strips.")
        } else if !dry {
            Some("You need a dry branch.")
        } else {
            Some("You need hide strips.")
        }
    }

    fn apply_bows(&mut self, bows: Vec<Intent>, results: &mut Vec<CommandResult>) {
        for intent in bows {
            if let Some(message) = self.bow_refusal(intent.actor_id) {
                self.finish(intent.command_id, intent.actor_id, false, message, results);
                continue;
            }
            let Some(branch) = self.bow_branch_id(intent.actor_id) else {
                self.finish(
                    intent.command_id,
                    intent.actor_id,
                    false,
                    "You need a dry branch.",
                    results,
                );
                continue;
            };
            let Some(strips) = self.bow_strips_id(intent.actor_id) else {
                self.finish(
                    intent.command_id,
                    intent.actor_id,
                    false,
                    "You need hide strips.",
                    results,
                );
                continue;
            };
            let cell = self
                .actor(intent.actor_id)
                .map(|actor| actor.cell)
                .unwrap_or(0);
            self.items.remove(&branch);
            self.items.remove(&strips);
            let id = self.mint_item(Material::Bow, 0, cell, Some(intent.actor_id), Role::Loose);
            let name = self.name_of(intent.actor_id);
            let rest = "a bow from a dry branch and hide strips.";
            self.emit(
                crate::model::spoken_act(Some(name.as_str()), "craft", rest),
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
                crate::model::spoken_act(None, "craft", rest),
                results,
            );
        }
    }

    fn apply_dry_clothes(&mut self, clothes: Vec<Intent>, results: &mut Vec<CommandResult>) {
        for intent in clothes {
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
            let wetness = self
                .actor(intent.actor_id)
                .map(|actor| actor.wetness)
                .unwrap_or(0);
            if wetness == 0 {
                self.finish(
                    intent.command_id,
                    intent.actor_id,
                    false,
                    "Your clothes are already dry.",
                    results,
                );
                continue;
            }
            if self
                .actor(intent.actor_id)
                .is_some_and(|actor| actor.clothes_dry_at.is_some())
            {
                self.finish(
                    intent.command_id,
                    intent.actor_id,
                    false,
                    "Your clothes are already drying by the fire.",
                    results,
                );
                continue;
            }
            let word = self.clothes_word(wetness);
            let at = self.time.saturating_add(self.tuning.clothes_ticks);
            if let Some(person) = self.actors.get_mut(&intent.actor_id) {
                person.clothes_dry_at = Some(at);
            }
            self.schedule(at, crate::model::Wake::DryClothes(intent.actor_id));
            let name = self.name_of(intent.actor_id);
            let rest = format!("their {word} clothes by the fire. They are not dry yet.");
            self.emit(
                format!("{name} holds {rest}"),
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
                format!("You hold your {word} clothes by the fire. They are not dry yet."),
                results,
            );
        }
    }

    fn apply_dresses(&mut self, dresses: Vec<Intent>, results: &mut Vec<CommandResult>) {
        for intent in dresses {
            let Some(item_id) = self
                .loose_ids(intent.actor_id, Material::Moss)
                .first()
                .copied()
            else {
                self.finish(
                    intent.command_id,
                    intent.actor_id,
                    false,
                    "You have no moss.",
                    results,
                );
                continue;
            };
            if self
                .actor(intent.actor_id)
                .is_some_and(|actor| actor.stings_until.is_none())
            {
                self.finish(
                    intent.command_id,
                    intent.actor_id,
                    false,
                    "You are not stung.",
                    results,
                );
                continue;
            }
            self.items.remove(&item_id);
            if let Some(person) = self.actors.get_mut(&intent.actor_id) {
                person.stings_until = None;
            }
            let cell = self
                .actor(intent.actor_id)
                .map(|actor| actor.cell)
                .unwrap_or(0);
            let name = self.name_of(intent.actor_id);
            let rest = "moss on the stings.";
            self.emit(
                crate::model::spoken_act(Some(name.as_str()), "press", rest),
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
                crate::model::spoken_act(None, "press", rest),
                results,
            );
        }
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
                crate::model::spoken_smear(Some(name.as_str()), "mud"),
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
                crate::model::spoken_smear(None, "mud"),
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
                crate::model::spoken_smear(Some(name.as_str()), "red ochre"),
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
                crate::model::spoken_smear(None, "red ochre"),
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
            if self.charcoal_marks.contains(&cell) {
                self.finish(
                    intent.command_id,
                    intent.actor_id,
                    false,
                    "A black mark is already here.",
                    results,
                );
                continue;
            }
            let rock = self.cell(cell).rock;
            let surface = if rock { "rock" } else { "ground" };
            let name = self.name_of(intent.actor_id);
            if let Some(item_id) = self
                .loose_ids(intent.actor_id, Material::Charcoal)
                .first()
                .copied()
            {
                self.items.remove(&item_id);
                self.charcoal_marks.insert(cell);
                let rest = format!("the {surface} with charcoal.");
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
                    "You have nothing to mark with.",
                    results,
                );
                continue;
            };
            self.items.remove(&item_id);
            self.ochre_marks.insert(cell);
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
        let rest = "a handful of mud.";
        self.emit(
            crate::model::spoken_act(Some(name.as_str()), "gather", rest),
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
            crate::model::spoken_act(None, "gather", rest),
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

fn worn_phrase(wolf: bool, sewn: bool) -> &'static str {
    match (wolf, sewn) {
        (true, true) => "the sewn wolf-hide cloak.",
        (true, false) => "the wolf hide.",
        (false, true) => "the sewn deer-hide cloak.",
        (false, false) => "the deer hide.",
    }
}

fn resin_light_line(who: Option<&str>, branch_arms: bool) -> String {
    let fuel = if branch_arms {
        "the pine resin and the branch from the arms"
    } else {
        "the pine resin from the arms and a dry branch from the pile at the feet"
    };
    crate::model::spoken_act(who, "light", &format!("a fire with {fuel}."))
}

fn resin_brand_light_line(who: Option<&str>, branch_arms: bool) -> String {
    let fuel = if branch_arms {
        "the pine resin and the branch from the arms"
    } else {
        "the pine resin from the arms and a dry branch from the pile at the feet"
    };
    crate::model::spoken_act(
        who,
        "light",
        &format!("a fire with the burning brand and {fuel}."),
    )
}

fn brand_light_line(who: Option<&str>, bark_arms: bool, branch_arms: bool) -> String {
    let fuel = match (bark_arms, branch_arms) {
        (true, true) => "the bark and the branch from the arms",
        (false, false) => "bark and a dry branch from the pile at the feet",
        (true, false) => "the bark from the arms and a dry branch from the pile at the feet",
        (false, true) => "bark from the pile at the feet and the branch from the arms",
    };
    crate::model::spoken_act(
        who,
        "light",
        &format!("a fire with the burning brand and {fuel}."),
    )
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
