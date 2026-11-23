//! Attempts in one window read the same starting state. Packet order is
//! discarded: ties break by actor id, then command id, and equal claims
//! break by a recorded hash.

use crate::model::{
    cell_id, cell_xy, contest_hash, ActorId, Attempt, CommandResult, Dir, Draw, Intent, ItemId,
    Material, Region, Role, Scope, Wake, WorkKind, World,
};
use std::collections::{BTreeMap, BTreeSet};

enum WetBranch {
    Ready,
    Already,
    Dry,
    None,
}

#[derive(Clone, Copy)]
enum FuelSpot {
    Charcoal,
    BranchPile,
    BranchGround,
}

/// The raw mushroom refusal. Reply only. The same two sentences as the look
/// after this person has tried them.
const RAW_MUSHROOM_REFUSAL: &str = "Raw mushrooms do not feed you. They must be cooked.";
const BARK_REFUSAL: &str = "Bark does not feed you.";
/// A gather or a craft that needs a free arm. The second sentence names `drop`.
const ARMS_FULL: &str = "Your arms are full. Drop something first.";

/// Which cordage a snare, a weir, a bow, or a spear haft spends.
#[derive(Clone, Copy, PartialEq, Eq)]
enum Line {
    Hide,
    Willow,
    Nettle,
    Sinew,
}

enum SinewSource {
    Ready(crate::model::ItemId),
    StrippedCarcass,
    StrippedMeat,
    None,
}

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
        let mut arms = Vec::new();
        let mut hints = Vec::new();
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
        let mut brews = Vec::new();
        let mut pine_teas = Vec::new();
        let mut willow_teas = Vec::new();
        let mut lights = Vec::new();
        let mut drill_fires = Vec::new();
        let mut drill_carves = Vec::new();
        let mut cooks = Vec::new();
        let mut eats = Vec::new();
        let mut smears = Vec::new();
        let mut ochre_smears = Vec::new();
        let mut marks = Vec::new();
        let mut sets = Vec::new();
        let mut smelts = Vec::new();
        let mut feeds = Vec::new();
        let mut banks = Vec::new();
        let mut hammers = Vec::new();
        let mut pierces = Vec::new();
        let mut scrapes = Vec::new();
        let mut sews = Vec::new();
        let mut binds = Vec::new();
        let mut knaps = Vec::new();
        let mut hafts = Vec::new();
        let mut pitches = Vec::new();
        let mut points = Vec::new();
        let mut hardens = Vec::new();
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
        let mut cups = Vec::new();
        let mut nets = Vec::new();
        let mut mats = Vec::new();
        let mut puts = Vec::new();
        let mut basket_puts = Vec::new();
        let mut snares = Vec::new();
        let mut set_nets = Vec::new();
        let mut weirs = Vec::new();
        let mut set_mats = Vec::new();
        let mut hollows = Vec::new();
        let mut carves_canoe = Vec::new();
        let mut launches = Vec::new();
        let mut seals = Vec::new();
        let mut cup_seals = Vec::new();
        let mut grinds = Vec::new();
        let mut boards = Vec::new();
        let mut paddles = Vec::new();
        let mut disembarks = Vec::new();
        let mut beaches = Vec::new();
        let mut carves = Vec::new();
        let mut needle_carves = Vec::new();
        let mut pick_carves = Vec::new();
        let mut needle_sews = Vec::new();
        let mut paddle_carves = Vec::new();
        let mut hook_carves = Vec::new();
        let mut whistle_carves = Vec::new();
        let mut whistle_blows = Vec::new();
        let mut flute_carves = Vec::new();
        let mut flute_blows = Vec::new();
        let mut drum_stretches = Vec::new();
        let mut drum_beats = Vec::new();
        let mut stick_carves = Vec::new();
        let mut stick_beats = Vec::new();
        let mut torch_makes = Vec::new();
        let mut torch_lights = Vec::new();
        let mut salts = Vec::new();
        let mut salt_fish = Vec::new();
        let mut smoke_fish = Vec::new();
        let mut smoke_meat = Vec::new();
        let mut smokes = Vec::new();
        let mut dresses = Vec::new();
        let mut bows = Vec::new();
        let mut clothes = Vec::new();
        let mut signals = Vec::new();
        let mut strikes = Vec::new();
        let mut ice_breaks = Vec::new();
        let mut pulls = Vec::new();

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
                    let hook_line = matches!(
                        message.as_str(),
                        "The brook is not here."
                            | "You are not carrying a bone fish hook."
                            | "You have no hide strips for a line."
                            | "You have no worm to bait the hook."
                            | "Ice covers the shallows."
                            | "The fish are gone from this stretch of the brook."
                            | "The fish are gone from the shallows."
                    );
                    let fish_is_the_hook = matches!(intent.attempt, Attempt::Fish)
                        && self.loose_ids(actor.id, Material::Spear).is_empty()
                        && self.loose_ids(actor.id, Material::Net).is_empty()
                        && !self.loose_ids(actor.id, Material::Hook).is_empty();
                    let hook_refusal = hook_line
                        && (matches!(intent.attempt, Attempt::FishHook) || fish_is_the_hook);
                    // A light refusal is the reply only. Bark, resin, rain,
                    // and a fire already burning stay with the person who tried.
                    let light_refusal = matches!(intent.attempt, Attempt::Light);
                    // No line for a snare is the reply only. A snare already
                    // set in the cell still sits in the actor's event list.
                    let snare_line_refusal = matches!(intent.attempt, Attempt::SetSnare)
                        && message == "You have no hide strips or willow cordage to set a snare.";
                    let ice_refusal = matches!(intent.attempt, Attempt::BreakIce);
                    let sinew_refusal = matches!(intent.attempt, Attempt::PullSinew);
                    let sticks_refusal =
                        matches!(intent.attempt, Attempt::CarveSticks | Attempt::BeatSticks);
                    // A pitch torch miss is the reply only.
                    let torch_refusal =
                        matches!(intent.attempt, Attempt::MakeTorch | Attempt::LightTorch);
                    // A mushroom bite that does not feed is the reply only.
                    let mushroom_eat_refusal =
                        matches!(intent.attempt, Attempt::Eat(Material::Mushroom));
                    // No sweet berries on this ground is the reply only.
                    let berry_eat_refusal = matches!(intent.attempt, Attempt::Eat(Material::Berry));
                    // No brook here, or ice on the still, is the reply only.
                    let drink_refusal = matches!(intent.attempt, Attempt::Drink);
                    // A roast miss is the reply only. A cook with no fire here is
                    // the same, including the step when a fire burns next door.
                    let roast_refusal = matches!(intent.attempt, Attempt::Roast);
                    let cook_fire_refusal = matches!(intent.attempt, Attempt::Cook)
                        && message.starts_with("There is no fire burning here");
                    // A bead or an awl that is not at hand is the reply only,
                    // and so is the step when the bead is one cell off.
                    let hammer_refusal =
                        matches!(intent.attempt, Attempt::Hammer | Attempt::HammerIron);
                    let pierce_refusal = matches!(intent.attempt, Attempt::Pierce);
                    // A scrape with no flake, flint, stone, or clear stone is
                    // the reply only, and so is any other scrape miss.
                    let scrape_refusal = matches!(intent.attempt, Attempt::Scrape);
                    // A bone needle miss is the reply only.
                    let needle_refusal =
                        matches!(intent.attempt, Attempt::CarveNeedle | Attempt::SewNeedle);
                    // An antler pick miss is the reply only.
                    let pick_refusal = matches!(intent.attempt, Attempt::CarvePick);
                    // A dry-ground dig that needs the pick, or finds the ground empty, is the reply only.
                    let dry_dig_refusal = matches!(intent.attempt, Attempt::Gather(Material::Worm))
                        && (message == "The dry ground here has no worms."
                            || message == crate::model::DRY_WORM_CARVE);
                    // No hafted spear, no pine resin, or a tip already set
                    // is the reply only.
                    let pitch_refusal = matches!(intent.attempt, Attempt::PitchSpear);
                    // A gather that finds the arms full is the reply only.
                    let full_arms_gather =
                        matches!(intent.attempt, Attempt::Gather(_)) && message == ARMS_FULL;
                    // No bark basket here, or one already in the arms, is the reply only.
                    let basket_gather_miss = matches!(intent.attempt, Attempt::Gather(_))
                        && message == "You are already carrying a bark basket.";
                    // No basket, nothing to put, or a full basket is the reply only.
                    let put_in_basket_miss = matches!(intent.attempt, Attempt::PutInBasket(_));
                    // No bark basket in the arms is the reply only.
                    let basket_drop_miss =
                        matches!(intent.attempt, Attempt::Drop(Material::Basket));
                    // An empty pile, with no step named, is the reply only.
                    // A weave that finds the arms full is the same.
                    let empty_pile = matches!(intent.attempt, Attempt::Gather(_))
                        && message.contains("here to gather.")
                        && !message.contains("Step ");
                    let full_arms_weave = matches!(
                        intent.attempt,
                        Attempt::WeaveBasket | Attempt::WeaveNet | Attempt::WeaveMat
                    ) && message == ARMS_FULL;
                    let personal =
                        !matches!(intent.attempt, Attempt::SmokeFish | Attempt::SmokeMeat)
                            && message != RAW_MUSHROOM_REFUSAL
                            && message != BARK_REFUSAL
                            && !hook_refusal
                            && !light_refusal
                            && !snare_line_refusal
                            && !ice_refusal
                            && !sinew_refusal
                            && !sticks_refusal
                            && !torch_refusal
                            && !mushroom_eat_refusal
                            && !berry_eat_refusal
                            && !drink_refusal
                            && !roast_refusal
                            && !cook_fire_refusal
                            && !hammer_refusal
                            && !pierce_refusal
                            && !scrape_refusal
                            && !needle_refusal
                            && !pick_refusal
                            && !dry_dig_refusal
                            && !pitch_refusal
                            && !full_arms_gather
                            && !basket_gather_miss
                            && !put_in_basket_miss
                            && !basket_drop_miss
                            && !empty_pile
                            && !full_arms_weave;
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
                        Attempt::Arms => arms.push(intent),
                        Attempt::NameSnare
                        | Attempt::NameCanoe
                        | Attempt::NameSinew
                        | Attempt::NameTorch
                        | Attempt::NameMushroom
                        | Attempt::NameNeedle
                        | Attempt::NameAntler
                        | Attempt::NamePick
                        | Attempt::NameBasket => hints.push(intent),
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
                        Attempt::Brew => brews.push(intent),
                        Attempt::SteepPine => pine_teas.push(intent),
                        Attempt::SteepWillow => willow_teas.push(intent),
                        Attempt::Light => lights.push(intent),
                        Attempt::DrillFire => drill_fires.push(intent),
                        Attempt::CarveDrill => drill_carves.push(intent),
                        Attempt::Cook | Attempt::Roast => cooks.push(intent),
                        Attempt::Eat(_) => eats.push(intent),
                        Attempt::Smear => smears.push(intent),
                        Attempt::SmearOchre => ochre_smears.push(intent),
                        Attempt::Mark | Attempt::MarkRock => marks.push(intent),
                        Attempt::SetClay => sets.push(intent),
                        Attempt::Smelt | Attempt::SmeltIron => smelts.push(intent),
                        Attempt::Feed => feeds.push(intent),
                        Attempt::Bank => banks.push(intent),
                        Attempt::Hammer | Attempt::HammerIron => hammers.push(intent),
                        Attempt::Pierce => pierces.push(intent),
                        Attempt::Scrape => scrapes.push(intent),
                        Attempt::Sew => sews.push(intent),
                        Attempt::BindSpear => binds.push(intent),
                        Attempt::Knap => knaps.push(intent),
                        Attempt::Haft => hafts.push(intent),
                        Attempt::PitchSpear => pitches.push(intent),
                        Attempt::PointSpear => points.push(intent),
                        Attempt::HardenSpear => hardens.push(intent),
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
                        Attempt::FoldCup => cups.push(intent),
                        Attempt::WeaveNet => nets.push(intent),
                        Attempt::WeaveMat => mats.push(intent),
                        Attempt::SetMat => set_mats.push(intent),
                        Attempt::HollowLog => hollows.push(intent),
                        Attempt::CarveCanoe => carves_canoe.push(intent),
                        Attempt::Launch => launches.push(intent),
                        Attempt::SealCanoe => seals.push(intent),
                        Attempt::SealCup => cup_seals.push(intent),
                        Attempt::Grind => grinds.push(intent),
                        Attempt::Board => boards.push(intent),
                        Attempt::Paddle(_) => paddles.push(intent),
                        Attempt::Disembark => disembarks.push(intent),
                        Attempt::Beach => beaches.push(intent),
                        Attempt::PutFish => puts.push(intent),
                        Attempt::PutInBasket(_) => basket_puts.push(intent),
                        Attempt::SetSnare => snares.push(intent),
                        Attempt::SetNet => set_nets.push(intent),
                        Attempt::SetWeir => weirs.push(intent),
                        Attempt::CarveAwl => carves.push(intent),
                        Attempt::CarveNeedle => needle_carves.push(intent),
                        Attempt::CarvePick => pick_carves.push(intent),
                        Attempt::SewNeedle => needle_sews.push(intent),
                        Attempt::CarvePaddle => paddle_carves.push(intent),
                        Attempt::CarveHook => hook_carves.push(intent),
                        Attempt::CarveWhistle => whistle_carves.push(intent),
                        Attempt::BlowWhistle => whistle_blows.push(intent),
                        Attempt::CarveFlute => flute_carves.push(intent),
                        Attempt::BlowFlute => flute_blows.push(intent),
                        Attempt::StretchDrum => drum_stretches.push(intent),
                        Attempt::BeatDrum => drum_beats.push(intent),
                        Attempt::CarveSticks => stick_carves.push(intent),
                        Attempt::BeatSticks => stick_beats.push(intent),
                        Attempt::MakeTorch => torch_makes.push(intent),
                        Attempt::LightTorch => torch_lights.push(intent),
                        Attempt::SaltMeat => salts.push(intent),
                        Attempt::SaltFish => salt_fish.push(intent),
                        Attempt::SmokeFish => smoke_fish.push(intent),
                        Attempt::SmokeMeat => smoke_meat.push(intent),
                        Attempt::SmokeNest => smokes.push(intent),
                        Attempt::DressSting => dresses.push(intent),
                        Attempt::CraftBow => bows.push(intent),
                        Attempt::DryClothes => clothes.push(intent),
                        Attempt::Signal(_) => signals.push(intent),
                        Attempt::Strike => strikes.push(intent),
                        Attempt::BreakIce => ice_breaks.push(intent),
                        Attempt::PullSinew => pulls.push(intent),
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
        for intent in arms {
            let line = self.arms_line(intent.actor_id);
            self.finish(intent.command_id, intent.actor_id, true, line, &mut results);
        }
        for intent in hints {
            let line = match intent.attempt {
                Attempt::NameSnare => "Set a snare.",
                Attempt::NameSinew => "Pull sinew from a carcass.",
                Attempt::NameTorch => "Make a pitch torch, or light one.",
                Attempt::NameMushroom => "Gather mushrooms, or cook them.",
                Attempt::NameNeedle => "Carve a bone needle.",
                Attempt::NameAntler => "Carve an antler pick.",
                Attempt::NamePick => "Dig worms from dry ground.",
                Attempt::NameBasket => "",
                _ => "Board, carve canoe, or paddle.",
            };
            let line = if matches!(intent.attempt, Attempt::NameBasket) {
                self.basket_name_line(intent.actor_id)
            } else {
                line.to_string()
            };
            self.finish(intent.command_id, intent.actor_id, true, line, &mut results);
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
        self.apply_brews(brews, &mut results);
        self.apply_pine_teas(pine_teas, &mut results);
        self.apply_willow_teas(willow_teas, &mut results);
        self.apply_sets(sets, &mut results);
        self.apply_feeds(feeds, &mut results);
        self.apply_banks(banks, &mut results);
        self.apply_smelts(smelts, &mut results);
        self.apply_hammers(hammers, &mut results);
        self.apply_pierces(pierces, &mut results);
        self.apply_scrapes(scrapes, &mut results);
        self.apply_sews(sews, &mut results);
        self.apply_binds(binds, &mut results);
        self.apply_knaps(knaps, &mut results);
        self.apply_hafts(hafts, &mut results);
        self.apply_pitches(pitches, &mut results);
        self.apply_points(points, &mut results);
        self.apply_hardens(hardens, &mut results);
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
        self.apply_ice_breaks(ice_breaks, &mut results);
        self.apply_brands(brands, &mut results);
        self.apply_baskets(baskets, &mut results);
        self.apply_cups(cups, &mut results);
        self.apply_nets(nets, &mut results);
        self.apply_mats(mats, &mut results);
        self.apply_set_mats(set_mats, &mut results);
        self.apply_hollows(hollows, &mut results);
        self.apply_canoe_carves(carves_canoe, &mut results);
        self.apply_launches(launches, &mut results);
        self.apply_seals(seals, &mut results);
        self.apply_cup_seals(cup_seals, &mut results);
        self.apply_grinds(grinds, &mut results);
        self.apply_boards(boards, &mut results);
        self.apply_paddles(paddles, &mut results);
        self.apply_disembarks(disembarks, &mut results);
        self.apply_beaches(beaches, &mut results);
        self.apply_puts(puts, &mut results);
        self.apply_basket_puts(basket_puts, &mut results);
        self.apply_snares(snares, &mut results);
        self.apply_set_nets(set_nets, &mut results);
        self.apply_weirs(weirs, &mut results);
        self.apply_carves(carves, &mut results);
        self.apply_needle_carves(needle_carves, &mut results);
        self.apply_pick_carves(pick_carves, &mut results);
        self.apply_needle_sews(needle_sews, &mut results);
        self.apply_paddle_carves(paddle_carves, &mut results);
        self.apply_hook_carves(hook_carves, &mut results);
        self.apply_whistle_carves(whistle_carves, &mut results);
        self.apply_whistle_blows(whistle_blows, &mut results);
        self.apply_flute_carves(flute_carves, &mut results);
        self.apply_flute_blows(flute_blows, &mut results);
        self.apply_drum_stretches(drum_stretches, &mut results);
        self.apply_drum_beats(drum_beats, &mut results);
        self.apply_stick_carves(stick_carves, &mut results);
        self.apply_stick_beats(stick_beats, &mut results);
        self.apply_torch_makes(torch_makes, &mut results);
        self.apply_torch_lights(torch_lights, &mut results);
        self.apply_salts(salts, &mut results);
        self.apply_salt_fish(salt_fish, &mut results);
        self.apply_smoke_fish(smoke_fish, &mut results);
        self.apply_smoke_meat(smoke_meat, &mut results);
        self.apply_smokes(smokes, &mut results);
        self.apply_dresses(dresses, &mut results);
        self.apply_bows(bows, &mut results);
        self.apply_dry_clothes(clothes, &mut results);
        self.apply_signals(signals, &mut results);
        self.apply_strikes(strikes, &mut results);
        self.apply_pulls(pulls, &mut results);
        self.apply_lights(lights, &start_cells, &mut results);
        self.apply_drill_fires(drill_fires, &start_cells, &mut results);
        self.apply_drill_carves(drill_carves, &mut results);
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
            Attempt::Look
            | Attempt::Examine(_)
            | Attempt::Arms
            | Attempt::NameSnare
            | Attempt::NameCanoe
            | Attempt::NameSinew
            | Attempt::NameTorch
            | Attempt::NameMushroom
            | Attempt::NameNeedle
            | Attempt::NameAntler
            | Attempt::NamePick
            | Attempt::NameBasket => Ok(false),
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
                } else if self.drink_source(actor.cell).is_none() {
                    Err("The brook is not here.".into())
                } else if self.drink_source(actor.cell) == Some(true)
                    && self.still_frozen(actor.cell)
                {
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
                    && self.loose_ids(actor.id, Material::BarkCup).is_empty()
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
            Attempt::Brew => {
                if busy {
                    Err("You are in the middle of something.".into())
                } else if body_taken {
                    Err("You already committed to something else this window.".into())
                } else if gripping {
                    Err("Your hands are on the log.".into())
                } else if let Some(message) = self.steep_refusal(actor, Material::Mint) {
                    Err(message.into())
                } else {
                    Ok(true)
                }
            }
            Attempt::SteepWillow => {
                if busy {
                    Err("You are in the middle of something.".into())
                } else if body_taken {
                    Err("You already committed to something else this window.".into())
                } else if gripping {
                    Err("Your hands are on the log.".into())
                } else if let Some(message) = self.steep_refusal(actor, Material::WillowBark) {
                    Err(message.into())
                } else {
                    Ok(true)
                }
            }
            Attempt::SteepPine => {
                if busy {
                    Err("You are in the middle of something.".into())
                } else if body_taken {
                    Err("You already committed to something else this window.".into())
                } else if gripping {
                    Err("Your hands are on the log.".into())
                } else if let Some(message) = self.steep_refusal(actor, Material::Needle) {
                    Err(message.into())
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
                    Err(ARMS_FULL.into())
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
                } else if self.shallows_here(actor.cell)
                    && !self.loose_ids(actor.id, Material::Hook).is_empty()
                    && !self.loose_ids(actor.id, Material::Worm).is_empty()
                {
                    Ok(true)
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
                } else if !self.fishing_water(actor.cell) && !self.shallows_here(actor.cell) {
                    Err("The brook is not here.".into())
                } else if self.loose_ids(actor.id, Material::Hook).is_empty() {
                    Err("You are not carrying a bone fish hook.".into())
                } else if self.shallows_here(actor.cell) {
                    if self.still_frozen(actor.cell) && !self.ice_hole(actor.cell) {
                        Err("Ice covers the shallows.".into())
                    } else if self.loose_ids(actor.id, Material::Worm).is_empty() {
                        Err("You have no worm to bait the hook.".into())
                    } else if self.shallows_are_quiet(actor.cell) {
                        Err("The fish are gone from the shallows.".into())
                    } else {
                        Ok(true)
                    }
                } else if self
                    .brook_stretch(actor.cell)
                    .is_some_and(|stretch| self.brook_is_quiet(stretch))
                {
                    Err("The fish are gone from this stretch of the brook.".into())
                } else if self.fishing_line(actor.id).is_none()
                    && self.loose_ids(actor.id, Material::Worm).is_empty()
                {
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
            Attempt::PointSpear => {
                if busy {
                    Err("You are in the middle of something.".into())
                } else if body_taken {
                    Err("You already committed to something else this window.".into())
                } else if gripping {
                    Err("Your hands are on the log.".into())
                } else if self.loose_ids(actor.id, Material::Branch).is_empty() {
                    Err("You have no branch to point.".into())
                } else if self.dry_loose_ids(actor.id, Material::Branch).is_empty() {
                    Err("The branch is too damp to point.".into())
                } else {
                    Ok(true)
                }
            }
            Attempt::HardenSpear => {
                if busy {
                    Err("You are in the middle of something.".into())
                } else if body_taken {
                    Err("You already committed to something else this window.".into())
                } else if gripping {
                    Err("Your hands are on the log.".into())
                } else if self
                    .loose_ids(actor.id, Material::Spear)
                    .iter()
                    .any(|id| self.item(*id).is_some_and(|spear| spear.flint_tip))
                {
                    Err("The flint tip is already better than a wooden point.".into())
                } else if self.wooden_spear(actor.id).is_none() {
                    Err("You have no wooden spear to harden.".into())
                } else if self
                    .item(self.wooden_spear(actor.id).unwrap())
                    .is_some_and(|spear| spear.hardened)
                {
                    Err("The wooden point is already hardened.".into())
                } else if self.fire_at(actor.cell).is_none() {
                    Err("There is no fire burning here.".into())
                } else if self
                    .item(self.wooden_spear(actor.id).unwrap())
                    .is_some_and(|spear| spear.firing_ready_at.is_some())
                {
                    Err("The wooden point is already hardening.".into())
                } else {
                    Ok(true)
                }
            }
            Attempt::PitchSpear => {
                if busy {
                    Err("You are in the middle of something.".into())
                } else if body_taken {
                    Err("You already committed to something else this window.".into())
                } else if gripping {
                    Err("Your hands are on the log.".into())
                } else if self.hafted_spear(actor.id).is_none() {
                    Err("You have no hafted spear to pitch.".into())
                } else if self
                    .item(self.hafted_spear(actor.id).unwrap())
                    .is_some_and(|spear| spear.pitched || spear.hardened)
                {
                    Err("The tip is already set.".into())
                } else if self.loose_ids(actor.id, Material::Resin).is_empty() {
                    Err("You have no pine resin to pitch the spear.".into())
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
                    Err(self
                        .hide_missing(actor.id, "You are not carrying a deer hide.")
                        .into())
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
                    Err(self
                        .hide_line(
                            actor.id,
                            "You are not wearing a deer hide.",
                            "You are not wearing a wolf hide.",
                            "You are not wearing an otter hide.",
                            "You are not wearing a fox hide.",
                        )
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
                } else if !self.fire_near(actor.cell) {
                    Err("There is no fire burning here.".into())
                } else if self.fish_to_hang(actor.id).is_none() {
                    if self.fish_already_hanging(actor.id) {
                        Err("A raw fish is already hanging by the fire.".into())
                    } else if self.smoked_fish_id(actor.id).is_some() {
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
                } else if !self.fire_near(actor.cell) {
                    Err("There is no fire burning here.".into())
                } else if self.meat_to_hang(actor.id).is_none() {
                    if self.meat_already_hanging(actor.id) {
                        Err("A portion of raw meat is already hanging by the fire.".into())
                    } else if self.smoked_meat_id(actor.id).is_some() {
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
            Attempt::CarveNeedle => {
                if busy {
                    Err("You are in the middle of something.".into())
                } else if body_taken {
                    Err("You already committed to something else this window.".into())
                } else if gripping {
                    Err("Your hands are on the log.".into())
                } else if !self.has_bone(actor.id) {
                    Err("You have no bone to carve a needle.".into())
                } else if !self.has_carve_tool(actor.id) {
                    Err("You need a flint flake or an iron knife to carve a needle.".into())
                } else {
                    Ok(true)
                }
            }
            Attempt::CarvePick => {
                if busy {
                    Err("You are in the middle of something.".into())
                } else if body_taken {
                    Err("You already committed to something else this window.".into())
                } else if gripping {
                    Err("Your hands are on the log.".into())
                } else if !self.has_antler(actor.id) {
                    Err("You have no antler to carve.".into())
                } else if !self.has_carve_tool(actor.id) {
                    Err("You need a flint flake or an iron knife to carve an antler pick.".into())
                } else {
                    Ok(true)
                }
            }
            Attempt::SewNeedle => {
                if busy {
                    Err("You are in the middle of something.".into())
                } else if body_taken {
                    Err("You already committed to something else this window.".into())
                } else if gripping {
                    Err("Your hands are on the log.".into())
                } else if let Some(message) = self.needle_sew_refusal(actor.id) {
                    Err(message.into())
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
            Attempt::CarveWhistle => {
                if busy {
                    Err("You are in the middle of something.".into())
                } else if body_taken {
                    Err("You already committed to something else this window.".into())
                } else if gripping {
                    Err("Your hands are on the log.".into())
                } else if !self.has_bone(actor.id) {
                    Err("You have no bone to carve a whistle.".into())
                } else if self.whistle_tool_rest(actor.id).is_none() {
                    Err("You need a flint flake or an iron knife to carve a whistle.".into())
                } else {
                    Ok(true)
                }
            }
            Attempt::BlowWhistle => {
                if busy {
                    Err("You are in the middle of something.".into())
                } else if body_taken {
                    Err("You already committed to something else this window.".into())
                } else if gripping {
                    Err("Your hands are on the log.".into())
                } else if self.loose_ids(actor.id, Material::Whistle).is_empty() {
                    Err("You are not carrying a bone whistle.".into())
                } else {
                    Ok(true)
                }
            }
            Attempt::CarveFlute => {
                if busy {
                    Err("You are in the middle of something.".into())
                } else if body_taken {
                    Err("You already committed to something else this window.".into())
                } else if gripping {
                    Err("Your hands are on the log.".into())
                } else if !self.has_reeds(actor.id) {
                    Err("You have no reeds to carve a flute.".into())
                } else if self.flute_tool_rest(actor.id).is_none() {
                    Err("You need a flint flake or an iron knife to carve a flute.".into())
                } else {
                    Ok(true)
                }
            }
            Attempt::BlowFlute => {
                if busy {
                    Err("You are in the middle of something.".into())
                } else if body_taken {
                    Err("You already committed to something else this window.".into())
                } else if gripping {
                    Err("Your hands are on the log.".into())
                } else if self.loose_ids(actor.id, Material::Flute).is_empty() {
                    Err("You are not carrying a reed flute.".into())
                } else {
                    Ok(true)
                }
            }
            Attempt::StretchDrum => {
                if busy {
                    Err("You are in the middle of something.".into())
                } else if body_taken {
                    Err("You already committed to something else this window.".into())
                } else if gripping {
                    Err("Your hands are on the log.".into())
                } else if let Some(message) = self.drum_refusal(actor.id) {
                    Err(message.into())
                } else {
                    Ok(true)
                }
            }
            Attempt::BreakIce => {
                if busy {
                    Err("You are in the middle of something.".into())
                } else if body_taken {
                    Err("You already committed to something else this window.".into())
                } else if gripping {
                    Err("Your hands are on the log.".into())
                } else if !self.shallows_here(actor.cell) || !self.still_frozen(actor.cell) {
                    Err("There is no ice here.".into())
                } else if self.ice_hole(actor.cell) {
                    Err("A hole is already open in the ice.".into())
                } else if self.ice_tool(actor.id).is_none() {
                    Err("You have nothing to break the ice.".into())
                } else {
                    Ok(true)
                }
            }
            Attempt::BeatDrum => {
                if busy {
                    Err("You are in the middle of something.".into())
                } else if body_taken {
                    Err("You already committed to something else this window.".into())
                } else if gripping {
                    Err("Your hands are on the log.".into())
                } else if self.loose_ids(actor.id, Material::Drum).is_empty() {
                    Err("You are not carrying a drum.".into())
                } else {
                    Ok(true)
                }
            }
            Attempt::CarveSticks => {
                if busy {
                    Err("You are in the middle of something.".into())
                } else if body_taken {
                    Err("You already committed to something else this window.".into())
                } else if gripping {
                    Err("Your hands are on the log.".into())
                } else if self.sticks_tool_rest(actor.id).is_none() {
                    Err("You need a flint flake or an iron knife to carve rhythm sticks.".into())
                } else if let Some(miss) = self.sticks_branch_miss(actor.id) {
                    Err(miss.into())
                } else if self.sticks_arms_full(actor.id) {
                    Err(ARMS_FULL.into())
                } else {
                    Ok(true)
                }
            }
            Attempt::BeatSticks => {
                if busy {
                    Err("You are in the middle of something.".into())
                } else if body_taken {
                    Err("You already committed to something else this window.".into())
                } else if gripping {
                    Err("Your hands are on the log.".into())
                } else if self.loose_ids(actor.id, Material::Sticks).is_empty() {
                    Err("You are not carrying rhythm sticks.".into())
                } else {
                    Ok(true)
                }
            }
            Attempt::MakeTorch => {
                if busy {
                    Err("You are in the middle of something.".into())
                } else if body_taken {
                    Err("You already committed to something else this window.".into())
                } else if gripping {
                    Err("Your hands are on the log.".into())
                } else if let Some(miss) = self.torch_make_miss(actor.id) {
                    Err(miss.into())
                } else {
                    Ok(true)
                }
            }
            Attempt::LightTorch => {
                if busy {
                    Err("You are in the middle of something.".into())
                } else if body_taken {
                    Err("You already committed to something else this window.".into())
                } else if gripping {
                    Err("Your hands are on the log.".into())
                } else if let Some(miss) = self.torch_light_miss(actor.id) {
                    Err(miss.into())
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
            Attempt::PullSinew => {
                if busy {
                    Err("You are in the middle of something.".into())
                } else if body_taken {
                    Err("You already committed to something else this window.".into())
                } else if gripping {
                    Err("Your hands are on the log.".into())
                } else if self.carry_count(actor.id) >= self.tuning.carry_slots {
                    Err(ARMS_FULL.into())
                } else {
                    match self.sinew_source(actor.id) {
                        SinewSource::Ready(_) => Ok(true),
                        SinewSource::StrippedCarcass => {
                            Err("The carcass is already stripped.".into())
                        }
                        SinewSource::StrippedMeat => Err("The meat is already stripped.".into()),
                        SinewSource::None => {
                            Err("You have no carcass or raw meat to pull sinew from.".into())
                        }
                    }
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
                    Err("You have no hide strips or willow cordage to set a snare.".into())
                } else {
                    Ok(true)
                }
            }
            Attempt::SetWeir => {
                if busy {
                    Err("You are in the middle of something.".into())
                } else if body_taken {
                    Err("You already committed to something else this window.".into())
                } else if gripping {
                    Err("Your hands are on the log.".into())
                } else if self.cell(actor.cell).region != Region::Brook {
                    Err("You cannot set a weir out of the brook.".into())
                } else if self.weirs.contains_key(&actor.cell) {
                    Err("A reed weir is already set here.".into())
                } else if !self.has_reeds(actor.id) {
                    Err("You have no reeds to set a weir.".into())
                } else if !self.has_snare_line(actor.id) {
                    Err("You have no cordage to bind a weir.".into())
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
                    Err(self.no_carcass_here(actor.cell).into())
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
                    let otter = self
                        .otter
                        .as_ref()
                        .map(|otter| crate::model::chebyshev(actor.cell, otter.cell));
                    let fox = self
                        .fox
                        .as_ref()
                        .map(|fox| crate::model::chebyshev(actor.cell, fox.cell));
                    let hare = self
                        .hare
                        .as_ref()
                        .map(|hare| crate::model::chebyshev(actor.cell, hare.cell));
                    let beaver = self
                        .beaver
                        .as_ref()
                        .map(|beaver| crate::model::chebyshev(actor.cell, beaver.cell));
                    let nearer = |dist: i32| {
                        deer.is_none_or(|far| dist < far)
                            && wolf.is_none_or(|far| dist < far)
                            && fox.is_none_or(|far| dist < far)
                    };
                    let fox_nearer = |dist: i32| {
                        deer.is_none_or(|far| dist < far)
                            && wolf.is_none_or(|far| dist < far)
                            && otter.is_none_or(|far| dist < far)
                    };
                    let wolf_melee = wolf.is_some_and(|dist| dist <= 1);
                    let wolf_shot = self.bow_in_hand(actor.id)
                        && wolf.is_some_and(|dist| dist <= self.tuning.sight);
                    let otter_melee = otter.is_some_and(|dist| dist <= 1 && nearer(dist));
                    let otter_shot = self.bow_in_hand(actor.id)
                        && otter.is_some_and(|dist| dist <= self.tuning.sight && nearer(dist));
                    let fox_melee = fox.is_some_and(|dist| dist <= 1 && fox_nearer(dist));
                    let fox_shot = self.bow_in_hand(actor.id)
                        && fox.is_some_and(|dist| dist <= self.tuning.sight && fox_nearer(dist));
                    let hare_nearer = |dist: i32| {
                        deer.is_none_or(|far| dist < far)
                            && wolf.is_none_or(|far| dist < far)
                            && fox.is_none_or(|far| dist < far)
                            && otter.is_none_or(|far| dist < far)
                    };
                    let hare_melee = hare.is_some_and(|dist| {
                        hare_nearer(dist)
                            && (dist == 0 || (dist == 1 && self.spear_reaches_next(actor.id)))
                    });
                    let hare_shot = self.bow_in_hand(actor.id)
                        && hare.is_some_and(|dist| dist <= self.tuning.sight && hare_nearer(dist));
                    let beaver_nearer = |dist: i32| {
                        deer.is_none_or(|far| dist < far)
                            && wolf.is_none_or(|far| dist < far)
                            && fox.is_none_or(|far| dist < far)
                            && otter.is_none_or(|far| dist < far)
                            && hare.is_none_or(|far| dist < far)
                    };
                    let beaver_melee = beaver.is_some_and(|dist| dist <= 1 && beaver_nearer(dist));
                    let beaver_shot = self.bow_in_hand(actor.id)
                        && beaver
                            .is_some_and(|dist| dist <= self.tuning.sight && beaver_nearer(dist));
                    if wolf_melee
                        || deer.is_some_and(|dist| dist <= 1)
                        || wolf_shot
                        || self.bow_can_reach(actor.id)
                        || otter_melee
                        || otter_shot
                        || fox_melee
                        || fox_shot
                        || hare_melee
                        || hare_shot
                        || beaver_melee
                        || beaver_shot
                    {
                        Ok(true)
                    } else if beaver.is_some_and(|dist| beaver_nearer(dist)) {
                        Err("The beaver is not close enough to hunt.".into())
                    } else if hare.is_some_and(|dist| hare_nearer(dist)) {
                        Err("The hare is not close enough to hunt.".into())
                    } else if fox.is_some_and(|dist| fox_nearer(dist)) {
                        Err("The fox is not close enough to hunt.".into())
                    } else if otter.is_some_and(|dist| nearer(dist)) {
                        Err("The otter is not close enough to hunt.".into())
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
                    Err(self.unhardened_spear_line(actor.id).into())
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
                    let otter = self
                        .otter
                        .as_ref()
                        .map(|otter| crate::model::chebyshev(actor.cell, otter.cell));
                    let fox = self
                        .fox
                        .as_ref()
                        .map(|fox| crate::model::chebyshev(actor.cell, fox.cell));
                    let wolf_shot = wolf.is_some_and(|dist| dist <= self.tuning.sight)
                        && wolf.is_some_and(|dist| deer.is_none_or(|far| dist < far));
                    let deer_shot = deer.is_some_and(|dist| dist <= self.tuning.sight);
                    let otter_shot = otter.is_some_and(|dist| {
                        dist <= self.tuning.sight
                            && deer.is_none_or(|far| dist < far)
                            && wolf.is_none_or(|far| dist < far)
                            && fox.is_none_or(|far| dist < far)
                    });
                    let fox_shot = fox.is_some_and(|dist| {
                        dist <= self.tuning.sight
                            && deer.is_none_or(|far| dist < far)
                            && wolf.is_none_or(|far| dist < far)
                            && otter.is_none_or(|far| dist < far)
                    });
                    let hare = self
                        .hare
                        .as_ref()
                        .map(|hare| crate::model::chebyshev(actor.cell, hare.cell));
                    let hare_shot = hare.is_some_and(|dist| {
                        dist <= self.tuning.sight
                            && deer.is_none_or(|far| dist < far)
                            && wolf.is_none_or(|far| dist < far)
                            && fox.is_none_or(|far| dist < far)
                            && otter.is_none_or(|far| dist < far)
                    });
                    let beaver = self
                        .beaver
                        .as_ref()
                        .map(|beaver| crate::model::chebyshev(actor.cell, beaver.cell));
                    let beaver_shot = beaver.is_some_and(|dist| {
                        dist <= self.tuning.sight
                            && deer.is_none_or(|far| dist < far)
                            && wolf.is_none_or(|far| dist < far)
                            && fox.is_none_or(|far| dist < far)
                            && otter.is_none_or(|far| dist < far)
                            && hare.is_none_or(|far| dist < far)
                    });
                    if wolf_shot || deer_shot || otter_shot || fox_shot || hare_shot || beaver_shot
                    {
                        Ok(true)
                    } else if beaver.is_some_and(|dist| {
                        deer.is_none_or(|far| dist < far)
                            && wolf.is_none_or(|far| dist < far)
                            && fox.is_none_or(|far| dist < far)
                            && otter.is_none_or(|far| dist < far)
                            && hare.is_none_or(|far| dist < far)
                    }) {
                        Err("The beaver is not close enough for the bow.".into())
                    } else if hare.is_some_and(|dist| {
                        deer.is_none_or(|far| dist < far)
                            && wolf.is_none_or(|far| dist < far)
                            && fox.is_none_or(|far| dist < far)
                            && otter.is_none_or(|far| dist < far)
                    }) {
                        Err("The hare is not close enough for the bow.".into())
                    } else if self.deer.as_ref().is_none()
                        && fox.is_some_and(|dist| otter.is_none_or(|far| dist < far))
                    {
                        Err("The fox is not close enough for the bow.".into())
                    } else if self.deer.as_ref().is_none() && self.otter.is_some() {
                        Err("The otter is not close enough for the bow.".into())
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
                    Err(self.gather_miss(
                        actor.cell,
                        Material::CopperOre,
                        "You are not carrying copper ore.".into(),
                    ))
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
            Attempt::Bank => {
                if busy {
                    Err("You are in the middle of something.".into())
                } else if body_taken {
                    Err("You already committed to something else this window.".into())
                } else if gripping {
                    Err("Your hands are on the log.".into())
                } else if self.fire_at(actor.cell).is_none() {
                    Err("There is no fire burning here.".into())
                } else if self.banks.contains_key(&actor.cell) {
                    Err("The fire is already banked.".into())
                } else if self.loose_ids(actor.id, Material::Mud).is_empty()
                    && self.pile_count(actor.cell, Material::Mud) == 0
                    && self.ground_ids(actor.cell, Material::Mud).is_empty()
                {
                    Err("You have no mud to cover the fire.".into())
                } else if self.dry_wood_on_ground(actor.cell) == 0 {
                    let wet = self.pile_count(actor.cell, Material::Branch) > 0
                        || !self.ground_ids(actor.cell, Material::Branch).is_empty();
                    if wet {
                        Err("The wood is too wet to bank.".into())
                    } else {
                        Err("There is no dry wood here to bank.".into())
                    }
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
                } else if self.copper_bead_id(actor.id).is_some() {
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
                    Err(self.gather_miss(
                        actor.cell,
                        Material::Copper,
                        "You are not carrying a copper bead.".into(),
                    ))
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
                    Err(self
                        .hide_missing(actor.id, "You are not carrying a deer hide.")
                        .into())
                } else if self
                    .loose_ids(actor.id, Material::Hide)
                    .iter()
                    .all(|id| self.item(*id).is_some_and(|hide| hide.pierced))
                {
                    Err(self
                        .hide_line(
                            actor.id,
                            "The deer hide is already pierced.",
                            "The wolf hide is already pierced.",
                            "The otter hide is already pierced.",
                            "The fox hide is already pierced.",
                        )
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
            Attempt::Scrape => {
                if busy {
                    Err("You are in the middle of something.".into())
                } else if body_taken {
                    Err("You already committed to something else this window.".into())
                } else if gripping {
                    Err("Your hands are on the log.".into())
                } else if self.scrape_tool(actor.id).is_none() {
                    Err("You need a flint flake or a stone to scrape the hide.".into())
                } else if self.scrape_hide(actor.id).is_some() {
                    Ok(true)
                } else if self
                    .scrape_hide_any(actor.id)
                    .is_some_and(|id| self.item(id).is_some_and(|hide| hide.sewn))
                {
                    Err(self
                        .hide_line(
                            actor.id,
                            "The deer hide is already a cloak.",
                            "The wolf hide is already a cloak.",
                            "The otter hide is already a cloak.",
                            "The fox hide is already a cloak.",
                        )
                        .into())
                } else if self
                    .scrape_hide_any(actor.id)
                    .is_some_and(|id| self.item(id).is_some_and(|hide| hide.scraped))
                {
                    Err(self
                        .hide_line(
                            actor.id,
                            "The deer hide is already scraped.",
                            "The wolf hide is already scraped.",
                            "The otter hide is already scraped.",
                            "The fox hide is already scraped.",
                        )
                        .into())
                } else {
                    Err(self
                        .hide_missing(actor.id, "You are not carrying a deer hide.")
                        .into())
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
                    Err(self
                        .hide_missing(actor.id, "You are not carrying a deer hide.")
                        .into())
                } else if self.loose_ids(actor.id, Material::Hide).iter().any(|id| {
                    self.item(*id)
                        .is_some_and(|hide| hide.pierced && hide.scraped)
                }) {
                    Ok(true)
                } else if self.loose_ids(actor.id, Material::Hide).iter().any(|id| {
                    self.item(*id)
                        .is_some_and(|hide| !hide.scraped && !hide.sewn)
                }) {
                    Err(self
                        .hide_line(
                            actor.id,
                            "The deer hide is not scraped.",
                            "The wolf hide is not scraped.",
                            "The otter hide is not scraped.",
                            "The fox hide is not scraped.",
                        )
                        .into())
                } else if self
                    .loose_ids(actor.id, Material::Hide)
                    .iter()
                    .any(|id| self.item(*id).is_some_and(|hide| hide.sewn))
                {
                    Err(self
                        .hide_line(
                            actor.id,
                            "The deer hide is already a cloak.",
                            "The wolf hide is already a cloak.",
                            "The otter hide is already a cloak.",
                            "The fox hide is already a cloak.",
                        )
                        .into())
                } else {
                    Err(self.unpierced_sew_line(actor.id))
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
                        Material::Cattail => {
                            if self.cooked_cattail_id(actor.id).is_some() {
                                Ok(true)
                            } else if self.raw_cattail_near(actor.id) {
                                Err("Raw cattail roots do not feed you. They must be cooked."
                                    .into())
                            } else {
                                Err("There are no cattail roots here to eat.".into())
                            }
                        }
                        Material::Flour => {
                            if self.cooked_flour_id(actor.id).is_some() {
                                Ok(true)
                            } else if self.raw_flour_id(actor.id).is_some() {
                                Err("Raw cattail flour does not feed you. It must be cooked."
                                    .into())
                            } else {
                                Err("There is no cattail flour here to eat.".into())
                            }
                        }
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
                            if self.sweet_berries_underfoot(actor.id) {
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
            Attempt::CarveDrill => {
                if busy {
                    Err("You are in the middle of something.".into())
                } else if body_taken {
                    Err("You already committed to something else this window.".into())
                } else if gripping {
                    Err("Your hands are on the log.".into())
                } else if let Some(message) = self.drill_refusal(actor.id) {
                    Err(message.into())
                } else {
                    Ok(true)
                }
            }
            Attempt::DrillFire => {
                if busy {
                    Err("You are in the middle of something.".into())
                } else if body_taken {
                    Err("You already committed to something else this window.".into())
                } else if gripping {
                    Err("Your hands are on the log.".into())
                } else if let Some(message) = self.drill_fire_refusal(actor.id) {
                    Err(message.into())
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
            Attempt::Strike => {
                if busy {
                    Err("You are in the middle of something.".into())
                } else if body_taken {
                    Err("You already committed to something else this window.".into())
                } else if gripping {
                    Err("Your hands are on the log.".into())
                } else if self.loose_ids(actor.id, Material::Flint).is_empty()
                    && self.loose_ids(actor.id, Material::Flake).is_empty()
                {
                    Err("You have no flint to strike.".into())
                } else if self.loose_ids(actor.id, Material::Quartz).is_empty()
                    && self.pile_count(actor.cell, Material::Quartz) == 0
                    && self.ground_ids(actor.cell, Material::Quartz).is_empty()
                {
                    if self.quartz_faces.contains(&actor.cell) {
                        Err("The rock face has no quartz.".into())
                    } else {
                        Err("There is no quartz here to strike.".into())
                    }
                } else if let Some(message) = self.spark_catch_block(actor.id) {
                    Err(message)
                } else {
                    Ok(true)
                }
            }
            Attempt::Signal(green) => {
                if busy {
                    Err("You are in the middle of something.".into())
                } else if body_taken {
                    Err("You already committed to something else this window.".into())
                } else if gripping {
                    Err("Your hands are on the log.".into())
                } else if self.fire_at(actor.cell).is_none() {
                    Err("There is no fire burning here.".into())
                } else if self.fire_at(actor.cell).is_some_and(|fire| fire.column) {
                    Err("A column of smoke is already rising.".into())
                } else if let Some(material) = green {
                    if self.has_material_at_hand(actor.id, *material) {
                        Ok(true)
                    } else if *material == Material::Reed {
                        Err("You have no green reeds to lay on the fire.".into())
                    } else {
                        Err("You have no green pine needles to lay on the fire.".into())
                    }
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
            Attempt::Roast => {
                if busy {
                    Err("You are in the middle of something.".into())
                } else if body_taken {
                    Err("You already committed to something else this window.".into())
                } else if self.fire_at(actor.cell).is_none() {
                    Err(self.no_fire_here(actor.cell))
                } else if self.raw_meat_id(actor.id).is_some()
                    || self.raw_fish_id(actor.id).is_some()
                    || self.raw_mushroom_id(actor.id).is_some()
                    || self.pile_count(actor.cell, Material::Mushroom) > 0
                {
                    Ok(true)
                } else if self.spoiled_meat_near(actor.id) {
                    Err("The meat has spoiled.".into())
                } else if self.spoiled_fish_near(actor.id) {
                    Err("The fish has spoiled.".into())
                } else {
                    Err("There is nothing here to roast.".into())
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
                } else if *material == Material::Cattail
                    && self.still_frozen(actor.cell)
                    && (self.pile_count(actor.cell, Material::Cattail) > 0
                        || !self.ground_ids(actor.cell, Material::Cattail).is_empty())
                {
                    return Err("Ice covers the cattail roots.".into());
                } else if *material == Material::Quartz {
                    let here = self.pile_count(actor.cell, Material::Quartz) > 0
                        || !self.ground_ids(actor.cell, Material::Quartz).is_empty();
                    if !here {
                        let bare = if self.quartz_faces.contains(&actor.cell) {
                            "The rock face has no quartz.".to_string()
                        } else {
                            "There is no quartz here to gather.".to_string()
                        };
                        return Err(self.gather_miss(actor.cell, *material, bare));
                    }
                } else if *material == Material::Worm {
                    let here = self.pile_count(actor.cell, Material::Worm) > 0
                        || !self.ground_ids(actor.cell, Material::Worm).is_empty();
                    if !here {
                        if self.pick_can_dig_dry(actor.id, actor.cell) {
                            if self.carry_count(actor.id) >= self.tuning.carry_slots
                                && !self.gather_overflows_into_basket(actor.id, Material::Worm)
                            {
                                return Err(ARMS_FULL.into());
                            }
                            return Ok(true);
                        }
                        let bare = if self.damp_earth(actor.cell) {
                            "The damp earth here has no worms.".to_string()
                        } else if self.dry_worm_returns_at.contains_key(&actor.cell)
                            && self.pick_for_dry_dig(actor.id, actor.cell)
                        {
                            "The dry ground here has no worms.".to_string()
                        } else if self.dry_worms_open(actor.cell)
                            && !self.pick_for_dry_dig(actor.id, actor.cell)
                            && self.gather_step_cue(actor.cell, Material::Worm).is_none()
                        {
                            crate::model::DRY_WORM_CARVE.to_string()
                        } else {
                            "There are no worms here to dig.".to_string()
                        };
                        return Err(self.gather_miss(actor.cell, *material, bare));
                    }
                } else if *material == Material::Fluff {
                    let plants = self.pile_count(actor.cell, Material::Cattail) > 0;
                    if plants && self.still_frozen(actor.cell) {
                        return Err("The cattails stand under the ice.".into());
                    }
                    if !plants || self.pile_count(actor.cell, Material::Fluff) == 0 {
                        return Err(self.gather_miss(
                            actor.cell,
                            *material,
                            "There is no cattail fluff here to gather.".to_string(),
                        ));
                    }
                } else if *material == Material::Egg
                    && self.pile_count(actor.cell, Material::Egg) == 0
                    && self.ground_ids(actor.cell, Material::Egg).is_empty()
                {
                    return Err(self.gather_miss(
                        actor.cell,
                        *material,
                        "There are no eggs here to gather.".to_string(),
                    ));
                } else if *material == Material::Basket
                    && self.pile_count(actor.cell, Material::Basket) == 0
                    && self.ground_ids(actor.cell, Material::Basket).is_empty()
                    && !self.loose_ids(actor.id, Material::Basket).is_empty()
                    && self.gather_step_cue(actor.cell, Material::Basket).is_none()
                {
                    return Err("You are already carrying a bark basket.".into());
                } else if self.pile_count(actor.cell, *material) == 0
                    && (!matches!(
                        *material,
                        Material::Stone
                            | Material::Clay
                            | Material::Bowl
                            | Material::Pot
                            | Material::Basket
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
                            | Material::Nettle
                            | Material::Cattail
                            | Material::BarkCup
                            | Material::Bone
                            | Material::BoneNeedle
                            | Material::Antler
                            | Material::Pick
                            | Material::Berry
                            | Material::Flute
                            | Material::Drum
                    ) || self.ground_ids(actor.cell, *material).is_empty())
                {
                    let bare = if *material == Material::BitterBerry {
                        format!(
                            "There are no {} here to gather.",
                            self.bitter_kind(actor.id)
                        )
                    } else if *material == Material::Needle {
                        "There are no pine needles here to gather.".to_string()
                    } else {
                        format!("There is no {} here to gather.", material.word())
                    };
                    return Err(self.gather_miss(actor.cell, *material, bare));
                }
                if self.carry_count(actor.id) >= self.tuning.carry_slots
                    && !self.gather_overflows_into_basket(actor.id, *material)
                {
                    return Err(ARMS_FULL.into());
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
                    return Err(ARMS_FULL.into());
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
                    return Err(ARMS_FULL.into());
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
                    return Err(ARMS_FULL.into());
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
            Attempt::SealCup => {
                if busy {
                    Err("You are in the middle of something.".into())
                } else if body_taken {
                    Err("You already committed to something else this window.".into())
                } else if gripping {
                    Err("Your hands are on the log.".into())
                } else if self.cup_to_seal(actor.id).is_none() {
                    Err("There is no bark cup here.".into())
                } else if self
                    .cup_to_seal(actor.id)
                    .is_some_and(|id| self.item(id).is_some_and(|cup| cup.pitched))
                {
                    Err("The bark cup is already sealed.".into())
                } else if self.loose_ids(actor.id, Material::Resin).is_empty() {
                    Err("You have no pine resin to seal the cup.".into())
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
            Attempt::Grind => {
                if gripping {
                    return Err("Your hands are on the log.".into());
                }
                if let Err(message) = self.grind_root(actor.id) {
                    return Err(message.into());
                }
                if self.loose_ids(actor.id, Material::Stone).is_empty() {
                    return Err("You need a stone to grind on.".into());
                }
                if !self.stone_under(actor.id) {
                    return Err("You need a stone on stone to grind.".into());
                }
                Ok(true)
            }
            Attempt::FoldCup => {
                if gripping {
                    return Err("Your hands are on the log.".into());
                }
                if !self.loose_ids(actor.id, Material::BarkCup).is_empty() {
                    return Err("You are already carrying a bark cup.".into());
                }
                let carried_bark = !self.loose_ids(actor.id, Material::Bark).is_empty();
                let piled_bark = self.pile_count(actor.cell, Material::Bark) > 0;
                if !carried_bark && !piled_bark {
                    return Err("You need a sheet of bark to fold a cup.".into());
                }
                if !carried_bark && self.carry_count(actor.id) >= self.tuning.carry_slots {
                    return Err(ARMS_FULL.into());
                }
                Ok(true)
            }
            Attempt::PutFish => {
                if gripping {
                    return Err("Your hands are on the log.".into());
                }
                if self.loose_ids(actor.id, Material::Basket).is_empty() {
                    return Err("You are not carrying a bark basket.".into());
                }
                if !self.basket_has_room(actor.id) {
                    return Err("The bark basket is full.".into());
                }
                if self.raw_fish_in_arms(actor.id).is_none() {
                    return Err("You are not carrying a raw fish.".into());
                }
                Ok(true)
            }
            Attempt::PutInBasket(material) => {
                if gripping {
                    return Err("Your hands are on the log.".into());
                }
                if self.loose_ids(actor.id, Material::Basket).is_empty() {
                    return Err("You are not carrying a bark basket.".into());
                }
                if *material == Material::Basket {
                    return Err("A bark basket does not go into a bark basket.".into());
                }
                if !self.basket_put_ready(actor.id, actor.cell, *material) {
                    return Err(format!(
                        "There is no {} here to put in the basket.",
                        material.gather_noun()
                    ));
                }
                if !self.basket_has_room(actor.id) {
                    return Err("The bark basket is full.".into());
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
                    | Material::Basket
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
                    | Material::BoneNeedle
                    | Material::Antler
                    | Material::Pick
                    | Material::Salt
                    | Material::Honey
                    | Material::Egg
                    | Material::Net
                    | Material::Mat
                    | Material::Paddle
                    | Material::Hook
                    | Material::Worm
                    | Material::Quartz
                    | Material::Whistle
                    | Material::Flute
                    | Material::Drum
                    | Material::Willow
                    | Material::Nettle
                    | Material::BarkCup
                    | Material::Berry
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
            let first_hide = self
                .ground_ids(cell, Material::Hide)
                .first()
                .and_then(|id| self.item(*id).cloned());
            let noun =
                if material == Material::Hide && first_hide.as_ref().is_some_and(|hide| hide.fox) {
                    "fox hide"
                } else if material == Material::Hide
                    && first_hide.as_ref().is_some_and(|hide| hide.hare)
                {
                    "hare hide"
                } else if material == Material::Hide
                    && first_hide.as_ref().is_some_and(|hide| hide.beaver)
                {
                    "beaver hide"
                } else if material == Material::Hide
                    && first_hide.as_ref().is_some_and(|hide| hide.otter)
                {
                    "otter hide"
                } else if material == Material::Hide
                    && first_hide.as_ref().is_some_and(|hide| hide.wolf)
                {
                    "wolf hide"
                } else {
                    material.gather_noun()
                };
            let dry_winner = if material == Material::Worm && available == 0 {
                claims
                    .iter()
                    .filter(|intent| self.pick_can_dig_dry(intent.actor_id, cell))
                    .min_by_key(|intent| {
                        contest_hash(self.seed, self.time, "gather", &subject, intent.actor_id)
                    })
                    .map(|intent| intent.actor_id)
            } else {
                None
            };
            for intent in claims {
                let hash = contest_hash(self.seed, self.time, "gather", &subject, intent.actor_id);
                let draw = Some(Draw {
                    tag: "gather".into(),
                    subject: subject.clone(),
                    value: hash,
                });
                let name = self.name_of(intent.actor_id);
                if dry_winner == Some(intent.actor_id) {
                    let item =
                        self.mint_item(Material::Worm, 0, cell, Some(intent.actor_id), Role::Loose);
                    self.tuck_overflow_into_basket(intent.actor_id, item);
                    self.arm_dry_worms(cell);
                    let rest = self.basket_gather_rest(
                        intent.actor_id,
                        item,
                        "a worm from the dry ground with the antler pick.".into(),
                    );
                    self.emit(
                        crate::model::spoken_act(Some(name.as_str()), "gather", &rest),
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
                    continue;
                }
                if winners.contains(&intent.actor_id) {
                    let item = if self.pile_count(cell, material) > 0 {
                        let moisture = self.take_from_pile(cell, material, 1).unwrap_or(0);
                        self.mint_item(material, moisture, cell, Some(intent.actor_id), Role::Loose)
                    } else if let Some(id) = self.ground_ids(cell, material).first().copied() {
                        if let Some(lying) = self.items.get_mut(&id) {
                            lying.carrier = Some(intent.actor_id);
                            lying.firing_ready_at = None;
                            if material == Material::Basket {
                                lying.in_basket = false;
                            }
                        }
                        if material == Material::Basket {
                            self.lift_basket_contents(intent.actor_id, cell, id);
                        }
                        id
                    } else {
                        let bare = if material == Material::BitterBerry {
                            format!(
                                "There are no {} here to gather.",
                                self.bitter_kind(intent.actor_id)
                            )
                        } else if material == Material::Needle {
                            "There are no pine needles here to gather.".to_string()
                        } else {
                            format!("There is no {} here to gather.", material.word())
                        };
                        self.finish(
                            intent.command_id,
                            intent.actor_id,
                            false,
                            self.gather_miss(cell, material, bare),
                            results,
                        );
                        continue;
                    };
                    if material != Material::Basket {
                        self.tuck_overflow_into_basket(intent.actor_id, item);
                    }
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
                    } else if material == Material::Nettle
                        && self
                            .actor(intent.actor_id)
                            .is_some_and(|person| person.mud < self.tuning.smear_at)
                    {
                        "a bundle of nettle cordage. The nettles sting.".to_string()
                    } else if material == Material::Nettle {
                        "a bundle of nettle cordage.".to_string()
                    } else if material == Material::Cattail {
                        "cattail roots.".to_string()
                    } else if material == Material::Fluff {
                        "a handful of cattail fluff.".to_string()
                    } else if material == Material::Worm {
                        "a worm.".to_string()
                    } else if material == Material::Quartz {
                        "a clear stone of quartz.".to_string()
                    } else if material == Material::Flour {
                        if self.item(item).is_some_and(|flour| flour.cooked) {
                            "a cattail flatbread.".to_string()
                        } else {
                            "cattail flour.".to_string()
                        }
                    } else if noun == "otter hide" {
                        format!("an otter hide{from}.")
                    } else if noun == "fox hide" {
                        format!("a fox hide{from}.")
                    } else if noun == "hare hide" {
                        format!("a hare hide{from}.")
                    } else if noun == "beaver hide" {
                        format!("a beaver hide{from}.")
                    } else if material == Material::BarkCup {
                        let kind = if self.item(item).is_some_and(|cup| cup.pitched) {
                            "sealed bark cup"
                        } else if self
                            .item(item)
                            .is_some_and(|cup| cup.moisture >= self.tuning.soaked_at)
                        {
                            "soaked bark cup"
                        } else {
                            "bark cup"
                        };
                        format!("a {kind}{from}.")
                    } else if material == Material::Meat {
                        let phrase = self
                            .item(item)
                            .map(|meat| self.meat_lift_noun(meat))
                            .unwrap_or_else(|| "portion of raw meat".into());
                        format!("a {phrase}{from}.")
                    } else {
                        format!("{} {noun}{from}.", material.article())
                    };
                    if material == Material::Salt {
                        self.arm_salt_lick(cell);
                    }
                    if material == Material::Bone {
                        self.arm_bone_cache(cell);
                    }
                    if material == Material::Honey && self.smoked_nests.contains(&cell) {
                        self.smoked_nests.remove(&cell);
                    } else if material == Material::Honey {
                        let until = self.time.saturating_add(self.tuning.sting_ticks);
                        if let Some(person) = self.actors.get_mut(&intent.actor_id) {
                            person.stings_until = Some(until);
                            person.nettle_sting = false;
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
                    if material == Material::Mint {
                        self.arm_mint_bank(cell);
                    }
                    if material == Material::Needle {
                        self.arm_needle_stand(cell);
                    }
                    if material == Material::Resin {
                        self.arm_resin_stand(cell);
                    }
                    if material == Material::Willow {
                        self.arm_willow_stand(cell);
                    }
                    if material == Material::WillowBark {
                        self.arm_willow_bark(cell);
                    }
                    if material == Material::Nettle
                        && self
                            .actor(intent.actor_id)
                            .is_some_and(|person| person.mud < self.tuning.smear_at)
                    {
                        let until = self.time.saturating_add(self.tuning.sting_ticks);
                        if let Some(person) = self.actors.get_mut(&intent.actor_id) {
                            person.stings_until = Some(until);
                            person.nettle_sting = true;
                        }
                    }
                    if material == Material::Nettle {
                        self.arm_nettle_stand(cell);
                    }
                    if material == Material::Cattail {
                        self.arm_cattail_stand(cell);
                    }
                    if material == Material::Fluff {
                        self.arm_fluff(cell);
                    }
                    if material == Material::Worm {
                        self.arm_worm_bed(cell);
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
                    let rest = self.basket_gather_rest(intent.actor_id, item, rest);
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
            let sealed_cup =
                material == Material::BarkCup && self.item(item_id).is_some_and(|cup| cup.pitched);
            let soaked_cup = material == Material::BarkCup
                && !sealed_cup
                && self
                    .item(item_id)
                    .is_some_and(|cup| cup.moisture >= self.tuning.soaked_at);
            let flatbread =
                material == Material::Flour && self.item(item_id).is_some_and(|flour| flour.cooked);
            let flour = material == Material::Flour && !flatbread;
            let meat_noun = if material == Material::Meat {
                self.item(item_id).map(|meat| self.meat_lift_noun(meat))
            } else {
                None
            };
            let noun = if bone_awl {
                "bone awl"
            } else if sealed_cup {
                "sealed bark cup"
            } else if soaked_cup {
                "soaked bark cup"
            } else if flatbread {
                "cattail flatbread"
            } else if flour {
                "cattail flour"
            } else {
                material.gather_noun()
            };
            let noun = meat_noun.as_deref().unwrap_or(noun);
            let dropped = if material == Material::Cord {
                format!("{name} drops hide strips. They are in contact with the ground.")
            } else if material == Material::Egg {
                format!("{name} drops the eggs. They are in contact with the ground.")
            } else if flour {
                format!("{name} drops the cattail flour. It is in contact with the ground.")
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
            } else if flour {
                "You drop the cattail flour. It is in contact with the ground.".to_string()
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
            self.frighten_otter(origin, dest);
            self.frighten_beaver(origin, dest);
            self.frighten_fox(origin, dest);
            self.frighten_hare(origin, dest);
            let brook = self.cell(dest).region == Region::Brook;
            let ice = self.still_frozen(dest);
            if brook {
                let wet = self.wade_wet(intent.actor_id);
                if let Some(actor) = self.actors.get_mut(&intent.actor_id) {
                    actor.wetness = (actor.wetness + wet).min(1000);
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
            self.note_brush(dest, false);
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

    /// One lump of pine resin, spent on a bark cup in the arms or at the feet.
    /// The clay pot is left alone. An unsealed cup still lets a drink soak through.
    fn apply_cup_seals(&mut self, seals: Vec<Intent>, results: &mut Vec<CommandResult>) {
        for intent in seals {
            let Some(actor) = self.actor(intent.actor_id).cloned() else {
                continue;
            };
            let Some(id) = self.cup_to_seal(actor.id) else {
                self.finish(
                    intent.command_id,
                    actor.id,
                    false,
                    "There is no bark cup here.",
                    results,
                );
                continue;
            };
            if self.item(id).is_some_and(|cup| cup.pitched) {
                self.finish(
                    intent.command_id,
                    actor.id,
                    false,
                    "The bark cup is already sealed.",
                    results,
                );
                continue;
            }
            let Some(lump) = self.loose_ids(actor.id, Material::Resin).into_iter().next() else {
                self.finish(
                    intent.command_id,
                    actor.id,
                    false,
                    "You have no pine resin to seal the cup.",
                    results,
                );
                continue;
            };
            self.items.remove(&lump);
            if let Some(cup) = self.items.get_mut(&id) {
                cup.pitched = true;
            }
            let name = self.name_of(actor.id);
            let rest = "the bark cup with pine resin.";
            self.emit(
                crate::model::spoken_act(Some(name.as_str()), "seal", rest),
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
                crate::model::spoken_act(None, "seal", rest),
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
                let wet = self.wade_wet(actor.id);
                if let Some(person) = self.actors.get_mut(&actor.id) {
                    person.wetness = (person.wetness + wet).min(1000);
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
        let here = self
            .actor(people[0].actor_id)
            .map(|actor| actor.cell)
            .unwrap_or(0);
        if (people.len() as u32) < self.hands_on_the_log(here) {
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
                    "The log does not move for you alone. A charred log is light enough.",
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
        if self.fire_at(person.cell).is_some() {
            return Some("A fire already burns here.".into());
        }
        let open = self.raining && self.exposure_millis(person.cell) >= 1000;
        // A lump of pine resin in the arms is tinder. It is spent before
        // fluff, moss, and bark when it is held. Cattail fluff in the arms
        // still catches when it is only slightly damp. Soaked fluff does not.
        // The open rain puts it out, the same as bark. Dry moss is tinder the
        // way bark is, and it is spent before bark. Wet moss is not.
        // Resin on the ground is not tinder. Dry moss in the shade pile, or a
        // dry handful lying on this ground, is tinder when no dry bark is at
        // hand. A soaked sheet in the arms is not the tinder when a dry sheet
        // is on the ground or in the pile, at the feet or one step off out of
        // the rain.
        let tinder = self.tinder_at_hand(actor);
        let fuel = self.fuel_at_hand(actor);
        if tinder && fuel {
            if open {
                return Some(self.open_rain_tinder_line(actor));
            }
            return None;
        }
        if self.neighbor_dry_bark_cell(actor).is_some() {
            return None;
        }
        if let Some(there) = self.neighbor_fire_instead(actor) {
            return Some(self.fire_already_off(person.cell, there));
        }
        if open {
            return Some(self.open_rain_tinder_line(actor));
        }
        if !tinder {
            let carried_fluff = !self.loose_ids(actor, Material::Fluff).is_empty();
            let carried_moss = !self.loose_ids(actor, Material::Moss).is_empty();
            return Some(if carried_fluff {
                "The cattail fluff is too damp to catch.".into()
            } else if carried_moss || self.undry_moss_lying(actor) {
                "The moss is too damp to catch.".into()
            } else if self.has_material_at_hand(actor, Material::Bark) {
                "The bark is too damp to catch.".into()
            } else {
                "You need dry loose bark for tinder.".into()
            });
        }
        return Some(if self.has_material_at_hand(actor, Material::Branch) {
            "The branch is too damp to burn.".into()
        } else {
            "You need a dry branch or a lump of charcoal for the fire.".into()
        });
    }

    /// Dry tinder in the arms or on this cell. A soaked sheet is not dry.
    fn tinder_at_hand(&self, actor: crate::model::ActorId) -> bool {
        !self.loose_ids(actor, Material::Resin).is_empty()
            || self.fluff_tinder_in_arms(actor)
            || self.moss_tinder_in_arms(actor)
            || self.has_dry(actor, Material::Bark)
            || self.unheld_dry_moss(actor)
    }

    /// Dry moss still in the shade pile, or a dry handful lying on this ground.
    /// The damp bank is not this moss. Bark is spent first when a dry sheet is
    /// at hand.
    fn unheld_dry_moss(&self, actor: crate::model::ActorId) -> bool {
        let Some(person) = self.actor(actor) else {
            return false;
        };
        let cell = person.cell;
        let shade =
            self.shade_moss.contains(&cell) && self.dry_pile_count(cell, Material::Moss) > 0;
        shade || self.dry_ground_id(cell, Material::Moss).is_some()
    }

    /// Moss in the shade or on this ground that is too wet to catch.
    fn undry_moss_lying(&self, actor: crate::model::ActorId) -> bool {
        let Some(person) = self.actor(actor) else {
            return false;
        };
        let cell = person.cell;
        let shade = self.shade_moss.contains(&cell)
            && self.pile_count(cell, Material::Moss) > 0
            && self.dry_pile_count(cell, Material::Moss) == 0;
        let loose = self.ground_ids(cell, Material::Moss).into_iter().any(|id| {
            self.item(id)
                .is_some_and(|item| !self.is_dry(item.moisture))
        });
        shade || loose
    }

    /// The shade pile first, then a dry handful on the ground. Arms are spent
    /// by the caller before this.
    fn take_unheld_dry_moss(
        &mut self,
        actor: crate::model::ActorId,
    ) -> Option<(Option<crate::model::ItemId>, MossSpot)> {
        let cell = self.actor(actor)?.cell;
        if self.shade_moss.contains(&cell) && self.dry_pile_count(cell, Material::Moss) > 0 {
            self.take_from_pile(cell, Material::Moss, 1)?;
            return Some((None, MossSpot::Pile));
        }
        let id = self.dry_ground_id(cell, Material::Moss)?;
        self.items.remove(&id);
        Some((Some(id), MossSpot::Ground))
    }

    /// Dry fuel in the arms or on this cell.
    fn fuel_at_hand(&self, actor: crate::model::ActorId) -> bool {
        self.charcoal_in_arms(actor)
            || self.has_dry(actor, Material::Branch)
            || self.charcoal_on_ground(actor)
    }

    fn open_rain_tinder_line(&self, actor: crate::model::ActorId) -> String {
        let resin = !self.loose_ids(actor, Material::Resin).is_empty();
        let fluff = self.fluff_tinder_in_arms(actor);
        let carried_fluff = !self.loose_ids(actor, Material::Fluff).is_empty();
        let dry_moss = self.moss_tinder_in_arms(actor) || self.unheld_dry_moss(actor);
        let carried_moss = !self.loose_ids(actor, Material::Moss).is_empty();
        let dry_bark = self.has_dry(actor, Material::Bark);
        if resin {
            "The rain puts the resin out before it catches.".into()
        } else if fluff {
            "The rain puts the cattail fluff out before it catches.".into()
        } else if dry_moss || (carried_moss && !dry_bark) {
            "The rain puts the moss out before it catches.".into()
        } else if carried_fluff {
            "The cattail fluff is too damp to catch.".into()
        } else {
            "The rain puts the bark out before it catches.".into()
        }
    }

    /// Dry bark and fuel on the next cell, out of the rain, with no fire yet.
    /// North, then east, south, and west. The soaked sheet in the arms stays.
    fn neighbor_dry_bark_cell(&self, actor: crate::model::ActorId) -> Option<crate::model::CellId> {
        let person = self.actor(actor)?;
        let (x, y) = cell_xy(person.cell);
        for dir in Dir::all() {
            let (dx, dy) = dir.delta();
            let Some(next) = cell_id(x + dx, y + dy) else {
                continue;
            };
            if self.fire_at(next).is_some() {
                continue;
            }
            if self.raining && self.exposure_millis(next) >= 1000 {
                continue;
            }
            if self.dry_bark_spot(next).is_some() && self.dry_fuel_spot(next).is_some() {
                return Some(next);
            }
        }
        None
    }

    /// A fire already burns on the next sheltered cell. The open rain is not
    /// the reason the light failed.
    fn neighbor_fire_instead(&self, actor: crate::model::ActorId) -> Option<crate::model::CellId> {
        let person = self.actor(actor)?;
        let (x, y) = cell_xy(person.cell);
        for dir in Dir::all() {
            let (dx, dy) = dir.delta();
            let Some(next) = cell_id(x + dx, y + dy) else {
                continue;
            };
            if self.fire_at(next).is_none() {
                continue;
            }
            if self.raining && self.exposure_millis(next) >= 1000 {
                continue;
            }
            return Some(next);
        }
        None
    }

    fn fire_already_off(&self, from: crate::model::CellId, there: crate::model::CellId) -> String {
        if self.cell(there).rock {
            "A fire already burns under the rock.".into()
        } else {
            format!(
                "A fire already burns to the {}.",
                self.step_word(from, there)
            )
        }
    }

    fn step_word(&self, from: crate::model::CellId, there: crate::model::CellId) -> &'static str {
        let (x1, y1) = cell_xy(from);
        let (x2, y2) = cell_xy(there);
        match (x2 - x1, y2 - y1) {
            (0, -1) => "north",
            (1, 0) => "east",
            (0, 1) => "south",
            _ => "west",
        }
    }

    fn tinder_place(&self, from: crate::model::CellId, there: crate::model::CellId) -> String {
        if self.cell(there).rock {
            "under the rock".into()
        } else {
            format!("to the {}", self.step_word(from, there))
        }
    }

    fn dry_ground_id(
        &self,
        cell: crate::model::CellId,
        material: Material,
    ) -> Option<crate::model::ItemId> {
        self.ground_ids(cell, material).into_iter().find(|id| {
            self.item(*id)
                .is_some_and(|item| self.is_dry(item.moisture))
        })
    }

    fn dry_bark_spot(&self, cell: crate::model::CellId) -> Option<bool> {
        if self.dry_pile_count(cell, Material::Bark) > 0 {
            Some(true)
        } else if self.dry_ground_id(cell, Material::Bark).is_some() {
            Some(false)
        } else {
            None
        }
    }

    fn dry_fuel_spot(&self, cell: crate::model::CellId) -> Option<FuelSpot> {
        if !self.ground_ids(cell, Material::Charcoal).is_empty() {
            Some(FuelSpot::Charcoal)
        } else if self.dry_pile_count(cell, Material::Branch) > 0 {
            Some(FuelSpot::BranchPile)
        } else if self.dry_ground_id(cell, Material::Branch).is_some() {
            Some(FuelSpot::BranchGround)
        } else {
            None
        }
    }

    /// Spend the dry sheet and the fuel on `there`. The soaked sheet in the
    /// arms stays. The fire burns on that cell, not in the rain at the feet.
    fn light_dry_bark_off(
        &mut self,
        intent: Intent,
        there: crate::model::CellId,
        draw: &Option<Draw>,
        results: &mut Vec<CommandResult>,
    ) {
        let Some(person) = self.actor(intent.actor_id) else {
            return;
        };
        let from = person.cell;
        let Some(bark_pile) = self.dry_bark_spot(there) else {
            self.finish(
                intent.command_id,
                intent.actor_id,
                false,
                "You need dry loose bark for tinder.",
                results,
            );
            return;
        };
        let Some(fuel) = self.dry_fuel_spot(there) else {
            self.finish(
                intent.command_id,
                intent.actor_id,
                false,
                "You need a dry branch or a lump of charcoal for the fire.",
                results,
            );
            return;
        };
        let mut spent = Vec::new();
        if bark_pile {
            self.take_from_pile(there, Material::Bark, 1);
        } else if let Some(id) = self.dry_ground_id(there, Material::Bark) {
            self.items.remove(&id);
            spent.push(id);
        }
        let from_charcoal = matches!(fuel, FuelSpot::Charcoal);
        match fuel {
            FuelSpot::Charcoal => {
                if let Some(id) = self.ground_ids(there, Material::Charcoal).first().copied() {
                    self.items.remove(&id);
                    spent.push(id);
                }
            }
            FuelSpot::BranchPile => {
                self.take_from_pile(there, Material::Branch, 1);
            }
            FuelSpot::BranchGround => {
                if let Some(id) = self.dry_ground_id(there, Material::Branch) {
                    self.items.remove(&id);
                    spent.push(id);
                }
            }
        }
        let place = self.tinder_place(from, there);
        let name = self.name_of(intent.actor_id);
        let line = off_bark_line(Some(&name), &place, bark_pile, fuel);
        let yours = off_bark_line(None, &place, bark_pile, fuel);
        let copied = draw.as_ref().map(|draw| Draw {
            tag: draw.tag.clone(),
            subject: draw.subject.clone(),
            value: draw.value,
        });
        let event = self.emit(
            line,
            Scope::Cell(there),
            vec![intent.actor_id],
            vec![there],
            spent,
            vec![],
            copied,
        );
        if from_charcoal {
            self.mint_charcoal_fire(there, event);
        } else {
            self.mint_fire(there, event);
        }
        let mut yours = yours;
        if self.scare_wolf(Some(intent.actor_id)) {
            yours.push_str(" The wolf flees the fire.");
        }
        self.finish(intent.command_id, intent.actor_id, true, yours, results);
    }

    /// A lump you are holding, or one on this ground. Either is the fuel,
    /// even when a dry branch lies here.
    fn charcoal_in_arms(&self, actor: crate::model::ActorId) -> bool {
        !self.loose_ids(actor, Material::Charcoal).is_empty()
    }

    fn charcoal_on_ground(&self, actor: crate::model::ActorId) -> bool {
        let Some(person) = self.actor(actor) else {
            return false;
        };
        !self.ground_ids(person.cell, Material::Charcoal).is_empty()
    }

    /// Arms first, then one lump on the ground. The bool is true when it came from the arms.
    fn take_charcoal(
        &mut self,
        actor: crate::model::ActorId,
    ) -> Result<(Option<crate::model::ItemId>, bool), ()> {
        if let Some(id) = self.loose_ids(actor, Material::Charcoal).first().copied() {
            self.items.remove(&id);
            return Ok((Some(id), true));
        }
        let Some(person) = self.actor(actor) else {
            return Err(());
        };
        let Some(id) = self
            .ground_ids(person.cell, Material::Charcoal)
            .first()
            .copied()
        else {
            return Err(());
        };
        self.items.remove(&id);
        Ok((Some(id), false))
    }

    pub(crate) fn cook_refusal(&self, actor: crate::model::ActorId) -> Option<String> {
        let person = self.actor(actor)?;
        if self.fire_at(person.cell).is_none() {
            return Some(self.no_fire_here(person.cell));
        }
        if !self.cookable_carried(actor)
            && self.cookable_pile(person.cell) == 0
            && self.raw_meat_id(actor).is_none()
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

    /// Dry cattail fluff, dry moss, or dry bark, already in the arms.
    /// Resin is not this tinder. A damp handful is not.
    fn dry_tinder_in_arms(&self, actor: crate::model::ActorId) -> Option<Material> {
        let dry_fluff = self
            .loose_ids(actor, Material::Fluff)
            .into_iter()
            .any(|id| self.item(id).is_some_and(|item| self.is_dry(item.moisture)));
        if dry_fluff {
            Some(Material::Fluff)
        } else if self.moss_tinder_in_arms(actor) {
            Some(Material::Moss)
        } else if !self.dry_loose_ids(actor, Material::Bark).is_empty() {
            Some(Material::Bark)
        } else {
            None
        }
    }

    /// Rain or a fire already burning, and only when the sparks would catch.
    /// Without dry tinder and a dry branch this stays empty, and the strike
    /// is sparks only.
    fn spark_catch_block(&self, actor: crate::model::ActorId) -> Option<String> {
        let tinder = self.dry_tinder_in_arms(actor)?;
        if !self.has_dry(actor, Material::Branch) {
            return None;
        }
        let person = self.actor(actor)?;
        if self.raining && self.exposure_millis(person.cell) >= 1000 {
            return Some(match tinder {
                Material::Fluff => "The rain puts the cattail fluff out before it catches.".into(),
                Material::Moss => "The rain puts the moss out before it catches.".into(),
                _ => "The rain puts the bark out before it catches.".into(),
            });
        }
        if self.fire_at(person.cell).is_some() {
            return Some("A fire already burns here.".into());
        }
        None
    }

    fn take_dry_from_arms(
        &mut self,
        actor: crate::model::ActorId,
        material: Material,
    ) -> Option<crate::model::ItemId> {
        let id = self.dry_loose_ids(actor, material).first().copied()?;
        self.items.remove(&id);
        Some(id)
    }

    /// Flint on quartz. Sparks jump. With dry tinder in the arms and a dry
    /// branch, the sparks catch and a fire burns. The stone and the flint stay.
    /// The catch is heard by someone who can see the cell. The striker does
    /// not read it again. Without the tinder, only the striker hears the sparks.
    fn apply_strikes(&mut self, strikes: Vec<Intent>, results: &mut Vec<CommandResult>) {
        for intent in strikes {
            let Some(actor) = self.actor(intent.actor_id).cloned() else {
                continue;
            };
            let catch = self.dry_tinder_in_arms(actor.id).is_some()
                && self.has_dry(actor.id, Material::Branch)
                && self.fire_at(actor.cell).is_none()
                && !(self.raining && self.exposure_millis(actor.cell) >= 1000);
            if !catch {
                self.finish(
                    intent.command_id,
                    intent.actor_id,
                    true,
                    "You strike the flint on the quartz. Sparks jump.",
                    results,
                );
                continue;
            }
            let Some(tinder) = self.dry_tinder_in_arms(actor.id) else {
                continue;
            };
            let Some(tinder_id) = self.take_dry_from_arms(actor.id, tinder) else {
                self.finish(
                    intent.command_id,
                    intent.actor_id,
                    true,
                    "You strike the flint on the quartz. Sparks jump.",
                    results,
                );
                continue;
            };
            let (branch_id, _) = match self.consume_dry(actor.id, Material::Branch) {
                Ok(spent) => spent,
                Err(()) => {
                    self.finish(
                        intent.command_id,
                        intent.actor_id,
                        false,
                        "You need a dry branch for the fire.",
                        results,
                    );
                    continue;
                }
            };
            let name = self.name_of(actor.id);
            let rest = "the flint on the quartz. The sparks catch.";
            let line = crate::model::spoken_act(Some(&name), "strike", rest);
            let mut yours = crate::model::spoken_act(None, "strike", rest);
            let spent: Vec<_> = Some(tinder_id).into_iter().chain(branch_id).collect();
            let event = self.emit(
                line,
                Scope::Cell(actor.cell),
                vec![actor.id],
                vec![actor.cell],
                spent,
                vec![],
                None,
            );
            self.mint_fire(actor.cell, event);
            if self.scare_wolf(Some(actor.id)) {
                yours.push_str(" The wolf flees the fire.");
            }
            self.finish(intent.command_id, actor.id, true, yours, results);
        }
    }

    /// A column of smoke from the fire here. The reply stays with the person
    /// who sent it. Look, not an event, places it for everyone else.
    fn apply_signals(&mut self, signals: Vec<Intent>, results: &mut Vec<CommandResult>) {
        for intent in signals {
            let Some(actor) = self.actor(intent.actor_id).cloned() else {
                continue;
            };
            let Attempt::Signal(green) = intent.attempt else {
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
            if self.fire_at(actor.cell).is_some_and(|fire| fire.column) {
                self.finish(
                    intent.command_id,
                    actor.id,
                    false,
                    "A column of smoke is already rising.",
                    results,
                );
                continue;
            }
            let message = if let Some(material) = green {
                if self.take_hand(actor.id, material).is_err() {
                    let missing = if material == Material::Reed {
                        "You have no green reeds to lay on the fire."
                    } else {
                        "You have no green pine needles to lay on the fire."
                    };
                    self.finish(intent.command_id, actor.id, false, missing, results);
                    continue;
                }
                if material == Material::Reed {
                    "You lay green reeds on the fire. A column of smoke rises."
                } else {
                    "You lay green pine needles on the fire. A column of smoke rises."
                }
            } else {
                "You send up a column of smoke."
            };
            if let Some(fire) = self.fires.values_mut().find(|fire| fire.cell == actor.cell) {
                fire.column = true;
            }
            self.finish(intent.command_id, actor.id, true, message, results);
        }
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
                if !self.tinder_at_hand(intent.actor_id) {
                    if let Some(there) = self.neighbor_dry_bark_cell(intent.actor_id) {
                        self.light_dry_bark_off(intent, there, &draw, results);
                        continue;
                    }
                }
                let brand = self
                    .loose_ids(intent.actor_id, Material::Brand)
                    .first()
                    .copied();
                let resin_id = self
                    .loose_ids(intent.actor_id, Material::Resin)
                    .first()
                    .copied();
                let name = self.name_of(intent.actor_id);
                let use_charcoal = self.charcoal_in_arms(intent.actor_id)
                    || self.charcoal_on_ground(intent.actor_id);
                let fluff_id = self
                    .loose_ids(intent.actor_id, Material::Fluff)
                    .into_iter()
                    .find(|id| {
                        self.item(*id)
                            .is_some_and(|item| item.moisture < self.tuning.soaked_at)
                    });
                let (tinder_item, resin, moss_arms, bark_arms, fluff_arms) = if let Some(resin_id) =
                    resin_id
                {
                    self.items.remove(&resin_id);
                    (Some(resin_id), true, None, true, None)
                } else if let Some(fluff_id) = fluff_id {
                    self.items.remove(&fluff_id);
                    (Some(fluff_id), false, None, false, Some(true))
                } else if self.moss_tinder_in_arms(intent.actor_id) {
                    match self.consume_dry(intent.actor_id, Material::Moss) {
                        Ok((moss_item, moss_from_arms)) => (
                            moss_item,
                            false,
                            Some(if moss_from_arms {
                                MossSpot::Arms
                            } else {
                                MossSpot::Pile
                            }),
                            false,
                            None,
                        ),
                        Err(()) => {
                            self.finish(
                                intent.command_id,
                                intent.actor_id,
                                false,
                                "The moss is too damp to catch.",
                                results,
                            );
                            continue;
                        }
                    }
                } else if self.has_dry(intent.actor_id, Material::Bark) {
                    match self.consume_dry(intent.actor_id, Material::Bark) {
                        Ok((bark_item, bark_from_arms)) => {
                            (bark_item, false, None, bark_from_arms, None)
                        }
                        Err(()) => {
                            self.finish(
                                intent.command_id,
                                intent.actor_id,
                                false,
                                "You need dry loose bark and a dry branch.",
                                results,
                            );
                            continue;
                        }
                    }
                } else if let Some((moss_item, spot)) = self.take_unheld_dry_moss(intent.actor_id) {
                    (moss_item, false, Some(spot), false, None)
                } else {
                    self.finish(
                        intent.command_id,
                        intent.actor_id,
                        false,
                        "You need dry loose bark and a dry branch.",
                        results,
                    );
                    continue;
                };
                let (fuel_item, from_charcoal, charcoal_arms, branch_arms) = if use_charcoal {
                    match self.take_charcoal(intent.actor_id) {
                        Ok((lump, charcoal_arms)) => (lump, true, charcoal_arms, false),
                        Err(()) => {
                            self.finish(
                                intent.command_id,
                                intent.actor_id,
                                false,
                                "You need a dry branch or a lump of charcoal for the fire.",
                                results,
                            );
                            continue;
                        }
                    }
                } else {
                    match self.consume_dry(intent.actor_id, Material::Branch) {
                        Ok((branch_item, branch_arms)) => (branch_item, false, false, branch_arms),
                        Err(()) => {
                            self.finish(
                                intent.command_id,
                                intent.actor_id,
                                false,
                                if resin || moss_arms.is_some() || fluff_arms.is_some() {
                                    "You need a dry branch or a lump of charcoal for the fire."
                                } else {
                                    "You need dry loose bark and a dry branch."
                                },
                                results,
                            );
                            continue;
                        }
                    }
                };
                if let Some(brand_id) = brand {
                    self.items.remove(&brand_id);
                }
                let (line, yours) = if from_charcoal {
                    (
                        charcoal_light_line(
                            Some(&name),
                            resin,
                            moss_arms,
                            bark_arms,
                            fluff_arms.is_some(),
                            brand.is_some(),
                            charcoal_arms,
                        ),
                        charcoal_light_line(
                            None,
                            resin,
                            moss_arms,
                            bark_arms,
                            fluff_arms.is_some(),
                            brand.is_some(),
                            charcoal_arms,
                        ),
                    )
                } else if fluff_arms.is_some() {
                    (
                        fluff_light_line(Some(&name), branch_arms, brand.is_some()),
                        fluff_light_line(None, branch_arms, brand.is_some()),
                    )
                } else if let Some(spot) = moss_arms {
                    if brand.is_some() {
                        (
                            moss_brand_light_line(Some(&name), spot, branch_arms),
                            moss_brand_light_line(None, spot, branch_arms),
                        )
                    } else {
                        (
                            moss_light_line(Some(&name), spot, branch_arms),
                            moss_light_line(None, spot, branch_arms),
                        )
                    }
                } else if resin {
                    if brand.is_some() {
                        (
                            resin_brand_light_line(Some(&name), branch_arms),
                            resin_brand_light_line(None, branch_arms),
                        )
                    } else {
                        (
                            resin_light_line(Some(&name), branch_arms),
                            resin_light_line(None, branch_arms),
                        )
                    }
                } else if brand.is_some() {
                    (
                        brand_light_line(Some(&name), bark_arms, branch_arms),
                        brand_light_line(None, bark_arms, branch_arms),
                    )
                } else {
                    (
                        light_line(Some(&name), bark_arms, branch_arms),
                        light_line(None, bark_arms, branch_arms),
                    )
                };
                let spent: Vec<_> = tinder_item.into_iter().chain(fuel_item).collect();
                let event = self.emit(
                    line,
                    Scope::Cell(cell),
                    vec![intent.actor_id],
                    vec![cell],
                    spent,
                    vec![],
                    draw,
                );
                if from_charcoal {
                    self.mint_charcoal_fire(cell, event);
                } else {
                    self.mint_fire(cell, event);
                }
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
            if matches!(intent.attempt, Attempt::Roast) {
                self.roast_food(intent, results);
                continue;
            }
            let cell = self
                .actor(intent.actor_id)
                .map(|actor| actor.cell)
                .unwrap_or(0);
            if self.raw_meat_id(intent.actor_id).is_some() {
                self.cook_meat(intent, results, "cook");
                continue;
            }
            if self.raw_fish_id(intent.actor_id).is_some() {
                self.cook_fish(intent, results, "cook");
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
                self.heat_mushroom(intent, results, "cook");
                continue;
            }
            if self.raw_cattail_id(intent.actor_id).is_some()
                || self.pile_count(cell, Material::Cattail) > 0
            {
                self.cook_cattail(intent, results);
                continue;
            }
            if self.raw_flour_id(intent.actor_id).is_some() {
                self.cook_flour(intent, results);
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

    /// Raw meat from the arms, else the ground, then a raw fish, then a
    /// mushroom. The same food `cook` would take, and the line says roast.
    fn roast_food(&mut self, intent: Intent, results: &mut Vec<CommandResult>) {
        if self.raw_meat_id(intent.actor_id).is_some() {
            self.cook_meat(intent, results, "roast");
            return;
        }
        if self.raw_fish_id(intent.actor_id).is_some() {
            self.cook_fish(intent, results, "roast");
            return;
        }
        self.heat_mushroom(intent, results, "roast");
    }

    /// Heat leaves the mushroom in the arms, or on the ground if it was a pile.
    /// It does not feed anyone until it is eaten. `roast` uses that stem.
    /// `cook` stays cook.
    fn heat_mushroom(&mut self, intent: Intent, results: &mut Vec<CommandResult>, stem: &str) {
        if let Some(message) = self.cook_refusal(intent.actor_id) {
            let message = if stem == "roast" && message == "There is nothing here to cook." {
                "There is nothing here to roast.".to_string()
            } else {
                message
            };
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
            let message = if stem == "roast" {
                "There is nothing here to roast."
            } else {
                "There is nothing here to cook."
            };
            self.finish(intent.command_id, intent.actor_id, false, message, results);
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
            crate::model::spoken_act(Some(name.as_str()), stem, rest),
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
            crate::model::spoken_act(None, stem, rest),
            results,
        );
    }

    /// Heat leaves the roots in the arms, or on the ground if they were a pile.
    /// They do not feed anyone until they are eaten.
    fn cook_cattail(&mut self, intent: Intent, results: &mut Vec<CommandResult>) {
        if let Some(message) = self.cook_refusal(intent.actor_id) {
            self.finish(intent.command_id, intent.actor_id, false, message, results);
            return;
        }
        let cell = self
            .actor(intent.actor_id)
            .map(|actor| actor.cell)
            .unwrap_or(0);
        let id = if let Some(id) = self.raw_cattail_id(intent.actor_id) {
            if let Some(root) = self.items.get_mut(&id) {
                root.cooked = true;
            }
            id
        } else if self.pile_count(cell, Material::Cattail) > 0 {
            let moisture = self.take_from_pile(cell, Material::Cattail, 1).unwrap_or(0);
            let id = self.mint_item(Material::Cattail, moisture, cell, None, Role::Loose);
            if let Some(root) = self.items.get_mut(&id) {
                root.cooked = true;
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
        let rest = "the cattail roots over the fire.";
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

    /// Heat turns cattail flour into a flatbread. It does not feed anyone until it is eaten.
    fn cook_flour(&mut self, intent: Intent, results: &mut Vec<CommandResult>) {
        if let Some(message) = self.cook_refusal(intent.actor_id) {
            self.finish(intent.command_id, intent.actor_id, false, message, results);
            return;
        }
        let Some(id) = self.raw_flour_id(intent.actor_id) else {
            self.finish(
                intent.command_id,
                intent.actor_id,
                false,
                "There is nothing here to cook.",
                results,
            );
            return;
        };
        if let Some(flour) = self.items.get_mut(&id) {
            flour.cooked = true;
        }
        let cell = self
            .actor(intent.actor_id)
            .map(|actor| actor.cell)
            .unwrap_or(0);
        let rest = "the flour into a flatbread over the fire.";
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

    /// Raw meat from the arms, else on this cell. It stays where it was and is not eaten.
    fn cook_meat(&mut self, intent: Intent, results: &mut Vec<CommandResult>, stem: &str) {
        if self.cook_refusal(intent.actor_id).is_some() {
            let message = self
                .cook_refusal(intent.actor_id)
                .unwrap_or_else(|| "There is no fire burning here.".into());
            let message = self.roast_word(stem, message);
            self.finish(intent.command_id, intent.actor_id, false, message, results);
            return;
        }
        let cell = self
            .actor(intent.actor_id)
            .map(|actor| actor.cell)
            .unwrap_or(0);
        let Some(id) = self.raw_meat_id(intent.actor_id) else {
            self.finish(
                intent.command_id,
                intent.actor_id,
                false,
                self.roast_word(stem, "There is nothing here to cook.".into()),
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
            crate::model::spoken_act(Some(name.as_str()), stem, rest),
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
            crate::model::spoken_act(None, stem, rest),
            results,
        );
    }

    fn roast_word(&self, stem: &str, message: String) -> String {
        if stem == "roast" && message == "There is nothing here to cook." {
            "There is nothing here to roast.".into()
        } else {
            message
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
                ARMS_FULL,
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
        let into_cup = !into_pot
            && !self
                .loose_ids(intent.actor_id, Material::BarkCup)
                .is_empty();
        let sealed_cup = into_cup
            && self
                .loose_ids(intent.actor_id, Material::BarkCup)
                .iter()
                .any(|id| self.item(*id).is_some_and(|cup| cup.pitched));
        let pot = if into_pot {
            " into the pot"
        } else if sealed_cup {
            " into the sealed bark cup"
        } else if into_cup {
            " into the bark cup"
        } else {
            ""
        };
        if into_cup && !sealed_cup {
            self.arm_cup_leak(item);
        }
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
                .is_some_and(|spear| spear.flint_tip && self.spear_is_set(spear))
        });
        if flint {
            return None;
        }
        if ids
            .iter()
            .any(|id| self.item(*id).is_some_and(|spear| spear.flint_tip))
        {
            Some("The flint tip is not set. Pitch it with pine resin, or hold it at a fire.")
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
                    && self
                        .actor(actor)
                        .is_some_and(|person| self.shallows_here(person.cell)) =>
            {
                let cell = self.actor(actor).map(|person| person.cell).unwrap_or(0);
                if self.still_frozen(cell) && !self.ice_hole(cell) {
                    Some("Ice covers the shallows.")
                } else if self.loose_ids(actor, Material::Worm).is_empty() {
                    Some("You have no worm to bait the hook.")
                } else if self.shallows_are_quiet(cell) {
                    Some("The fish are gone from the shallows.")
                } else {
                    None
                }
            }
            Some("You are not carrying a spear.")
                if !self.loose_ids(actor, Material::Hook).is_empty()
                    && self.fishing_line(actor).is_some() =>
            {
                None
            }
            Some("You are not carrying a spear.")
                if !self.loose_ids(actor, Material::Hook).is_empty()
                    && !self.loose_ids(actor, Material::Worm).is_empty() =>
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
            && (self.fishing_line(intent.actor_id).is_some()
                || !self.loose_ids(intent.actor_id, Material::Worm).is_empty())
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
        if let Some(id) = self.loose_ids(actor, Material::Nettle).first().copied() {
            return Some((id, "The nettle cordage is spent."));
        }
        None
    }

    /// The still shallows. The worm is the bait. The hook stays. The line is not spent.
    fn apply_shallows_hook(
        &mut self,
        intent: Intent,
        cell: crate::model::CellId,
        results: &mut Vec<CommandResult>,
    ) {
        if self.still_frozen(cell) && !self.ice_hole(cell) {
            self.finish(
                intent.command_id,
                intent.actor_id,
                false,
                "Ice covers the shallows.",
                results,
            );
            return;
        }
        if self.loose_ids(intent.actor_id, Material::Worm).is_empty() {
            self.finish(
                intent.command_id,
                intent.actor_id,
                false,
                "You have no worm to bait the hook.",
                results,
            );
            return;
        }
        if self.shallows_are_quiet(cell) {
            self.finish(
                intent.command_id,
                intent.actor_id,
                false,
                "The fish are gone from the shallows.",
                results,
            );
            return;
        }
        let until = self.time.saturating_add(self.tuning.clay_ticks);
        self.shallows_quiet.insert(cell, until);
        let water = if self.ice_hole(cell) {
            "the hole in the ice"
        } else {
            "the shallows"
        };
        self.finish_worm_hook(intent, cell, water, results);
    }

    /// A stone, a bone, or an iron knife. The first one in the arms is the one named.
    fn ice_tool(&self, actor: crate::model::ActorId) -> Option<&'static str> {
        if !self.loose_ids(actor, Material::Stone).is_empty() {
            Some("the stone")
        } else if !self.loose_ids(actor, Material::Bone).is_empty() {
            Some("the bone")
        } else if !self.loose_ids(actor, Material::Knife).is_empty() {
            Some("the iron knife")
        } else {
            None
        }
    }

    /// Open a hole in the ice. The tool stays. Someone who cannot see the cell
    /// hears the break, and past hearing hears nothing.
    fn apply_ice_breaks(&mut self, breaks: Vec<Intent>, results: &mut Vec<CommandResult>) {
        for intent in breaks {
            let Some(actor) = self.actor(intent.actor_id).cloned() else {
                continue;
            };
            if !self.shallows_here(actor.cell) || !self.still_frozen(actor.cell) {
                self.finish(
                    intent.command_id,
                    actor.id,
                    false,
                    "There is no ice here.",
                    results,
                );
                continue;
            }
            if self.ice_hole(actor.cell) {
                self.finish(
                    intent.command_id,
                    actor.id,
                    false,
                    "A hole is already open in the ice.",
                    results,
                );
                continue;
            }
            let Some(tool) = self.ice_tool(actor.id) else {
                self.finish(
                    intent.command_id,
                    actor.id,
                    false,
                    "You have nothing to break the ice.",
                    results,
                );
                continue;
            };
            self.ice_holes.insert(actor.cell);
            let name = self.name_of(actor.id);
            let rest = format!("a hole in the ice with {tool}.");
            self.emit(
                crate::model::spoken_act(Some(name.as_str()), "break", &rest),
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
                crate::model::spoken_act(None, "break", &rest),
                results,
            );
        }
    }

    /// A bone hook and a worm, and no line. The worm is spent. The hook stays.
    fn finish_worm_hook(
        &mut self,
        intent: Intent,
        cell: crate::model::CellId,
        water: &str,
        results: &mut Vec<CommandResult>,
    ) {
        let Some(worm) = self
            .loose_ids(intent.actor_id, Material::Worm)
            .first()
            .copied()
        else {
            self.finish(
                intent.command_id,
                intent.actor_id,
                false,
                "You have no worm to bait the hook.",
                results,
            );
            return;
        };
        self.items.remove(&worm);
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
        let rest = format!("a fish from {water} with the bone fish hook and a worm.");
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

    /// The line is spent. The hook stays.
    fn apply_hook_fish(&mut self, intent: Intent, results: &mut Vec<CommandResult>) {
        let cell = self
            .actor(intent.actor_id)
            .map(|actor| actor.cell)
            .unwrap_or(0);
        if !self.fishing_water(cell) && !self.shallows_here(cell) {
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
        if self.shallows_here(cell) {
            self.apply_shallows_hook(intent, cell, results);
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
            if self.loose_ids(intent.actor_id, Material::Worm).is_empty() {
                self.finish(
                    intent.command_id,
                    intent.actor_id,
                    false,
                    "You have no hide strips for a line.",
                    results,
                );
            } else {
                if let Some(stretch) = self.brook_stretch(cell) {
                    let until = self.time.saturating_add(1);
                    self.brook_quiet.insert(stretch, until);
                }
                self.finish_worm_hook(intent, cell, "the brook", results);
            }
            return;
        };
        self.items.remove(&line);
        let worm_id = self
            .loose_ids(intent.actor_id, Material::Worm)
            .first()
            .copied();
        if let Some(id) = worm_id {
            self.items.remove(&id);
        }
        let wormed = worm_id.is_some();
        if let Some(stretch) = self.brook_stretch(cell) {
            let ticks = if wormed { 1 } else { self.tuning.clay_ticks };
            let until = self.time.saturating_add(ticks);
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
        let with = if wormed {
            "the bone fish hook and a worm"
        } else {
            "the bone fish hook"
        };
        let rest = format!("a fish from the brook with {with}. {spent}");
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

    fn cook_fish(&mut self, intent: Intent, results: &mut Vec<CommandResult>, stem: &str) {
        if self.cook_refusal(intent.actor_id).is_some() {
            let message = self
                .cook_refusal(intent.actor_id)
                .unwrap_or_else(|| "There is no fire burning here.".into());
            let message = self.roast_word(stem, message);
            self.finish(intent.command_id, intent.actor_id, false, message, results);
            return;
        }
        let Some(id) = self.raw_fish_id(intent.actor_id) else {
            self.finish(
                intent.command_id,
                intent.actor_id,
                false,
                self.roast_word(stem, "There is nothing here to cook.".into()),
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
            crate::model::spoken_act(Some(name.as_str()), stem, rest),
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
            crate::model::spoken_act(None, stem, rest),
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
            let nettle = if strips.is_none() && willow.is_none() {
                self.loose_ids(intent.actor_id, Material::Nettle)
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
            if let Some(nettle) = nettle {
                self.items.remove(&nettle);
            }
            let id = self.mint_item(Material::Spear, 0, cell, Some(intent.actor_id), Role::Loose);
            if let Some(spear) = self.items.get_mut(&id) {
                spear.flint_tip = true;
                spear.bound = strips.is_some() || willow.is_some() || nettle.is_some();
                spear.bound_willow = willow.is_some();
                spear.bound_nettle = nettle.is_some();
            }
            let rest = if strips.is_some() {
                "the flint flake and a dry branch into a spear. The tip is bound with hide."
            } else if willow.is_some() {
                "the flint flake and a dry branch into a spear. The tip is bound with willow cordage."
            } else if nettle.is_some() {
                "the flint flake and a dry branch into a spear. The tip is bound with nettle cordage."
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

    /// The hafted flint spear that still needs pitch, or the one already set.
    fn hafted_spear(&self, actor: crate::model::ActorId) -> Option<crate::model::ItemId> {
        let ids = self.loose_ids(actor, Material::Spear);
        ids.iter()
            .copied()
            .find(|id| {
                self.item(*id)
                    .is_some_and(|spear| spear.flint_tip && !spear.pitched && !spear.hardened)
            })
            .or_else(|| {
                ids.into_iter()
                    .find(|id| self.item(*id).is_some_and(|spear| spear.flint_tip))
            })
    }

    fn spear_is_set(&self, spear: &crate::model::Item) -> bool {
        spear.hardened || (spear.flint_tip && spear.pitched)
    }

    /// One lump of pine resin on a hafted flint spear. The tip is set.
    /// The fire is not required. The spear stays.
    fn apply_pitches(&mut self, pitches: Vec<Intent>, results: &mut Vec<CommandResult>) {
        for intent in pitches {
            let Some(id) = self.hafted_spear(intent.actor_id) else {
                self.finish(
                    intent.command_id,
                    intent.actor_id,
                    false,
                    "You have no hafted spear to pitch.",
                    results,
                );
                continue;
            };
            if self
                .item(id)
                .is_some_and(|spear| spear.pitched || spear.hardened)
            {
                self.finish(
                    intent.command_id,
                    intent.actor_id,
                    false,
                    "The tip is already set.",
                    results,
                );
                continue;
            }
            let Some(lump) = self
                .loose_ids(intent.actor_id, Material::Resin)
                .into_iter()
                .next()
            else {
                self.finish(
                    intent.command_id,
                    intent.actor_id,
                    false,
                    "You have no pine resin to pitch the spear.",
                    results,
                );
                continue;
            };
            self.items.remove(&lump);
            if let Some(spear) = self.items.get_mut(&id) {
                spear.pitched = true;
            }
            let cell = self
                .actor(intent.actor_id)
                .map(|actor| actor.cell)
                .unwrap_or(0);
            let name = self.name_of(intent.actor_id);
            let rest = "the join of the spear with pine resin.";
            self.emit(
                crate::model::spoken_act(Some(name.as_str()), "pitch", rest),
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
                crate::model::spoken_act(None, "pitch", rest),
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

    fn wooden_spear(&self, actor: crate::model::ActorId) -> Option<crate::model::ItemId> {
        self.loose_ids(actor, Material::Spear)
            .into_iter()
            .find(|id| self.item(*id).is_some_and(|spear| spear.wooden_tip))
    }

    fn unhardened_spear_line(&self, actor: crate::model::ActorId) -> String {
        if self.flint_tip_is_hardening(actor) {
            self.hardening_tip_line(actor)
        } else if self
            .loose_ids(actor, Material::Spear)
            .iter()
            .any(|id| self.item(*id).is_some_and(|spear| spear.flint_tip))
        {
            "The flint tip is not set. Pitch it with pine resin, or hold it at a fire.".into()
        } else if self.wooden_spear(actor).is_some() {
            "The wooden point is not hardened.".into()
        } else {
            "The spear's point is not hardened.".into()
        }
    }

    /// The wait still running on a flint tip. More than one tick is a little
    /// longer. The last tick, and the tick it finishes, is almost ready.
    fn hardening_tip_line(&self, actor: crate::model::ActorId) -> String {
        let left = self
            .loose_ids(actor, Material::Spear)
            .iter()
            .filter_map(|id| {
                self.item(*id).and_then(|spear| {
                    (spear.flint_tip && !spear.hardened)
                        .then_some(spear.firing_ready_at)
                        .flatten()
                })
            })
            .min()
            .map(|ready| ready.saturating_sub(self.time))
            .unwrap_or(0);
        if left <= 1 {
            "The flint tip is still hardening in the fire. It is almost ready.".into()
        } else {
            "The flint tip is still hardening in the fire. A little longer.".into()
        }
    }

    fn apply_points(&mut self, points: Vec<Intent>, results: &mut Vec<CommandResult>) {
        for intent in points {
            let Some(branch) = self
                .dry_loose_ids(intent.actor_id, Material::Branch)
                .first()
                .copied()
            else {
                let message = if self.loose_ids(intent.actor_id, Material::Branch).is_empty() {
                    "You have no branch to point."
                } else {
                    "The branch is too damp to point."
                };
                self.finish(intent.command_id, intent.actor_id, false, message, results);
                continue;
            };
            let cell = self
                .actor(intent.actor_id)
                .map(|actor| actor.cell)
                .unwrap_or(0);
            self.items.remove(&branch);
            let id = self.mint_item(Material::Spear, 0, cell, Some(intent.actor_id), Role::Loose);
            if let Some(spear) = self.items.get_mut(&id) {
                spear.wooden_tip = true;
            }
            let name = self.name_of(intent.actor_id);
            let rest = "a dry branch into a wooden spear.";
            self.emit(
                crate::model::spoken_act(Some(name.as_str()), "point", rest),
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
                crate::model::spoken_act(None, "point", rest),
                results,
            );
        }
    }

    fn apply_hardens(&mut self, hardens: Vec<Intent>, results: &mut Vec<CommandResult>) {
        for intent in hardens {
            let Some(actor) = self.actor(intent.actor_id).cloned() else {
                continue;
            };
            if self
                .loose_ids(actor.id, Material::Spear)
                .iter()
                .any(|id| self.item(*id).is_some_and(|spear| spear.flint_tip))
            {
                self.finish(
                    intent.command_id,
                    actor.id,
                    false,
                    "The flint tip is already better than a wooden point.",
                    results,
                );
                continue;
            }
            let Some(id) = self.wooden_spear(actor.id) else {
                self.finish(
                    intent.command_id,
                    actor.id,
                    false,
                    "You have no wooden spear to harden.",
                    results,
                );
                continue;
            };
            if self.item(id).is_some_and(|spear| spear.hardened) {
                self.finish(
                    intent.command_id,
                    actor.id,
                    false,
                    "The wooden point is already hardened.",
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
            if self
                .item(id)
                .is_some_and(|spear| spear.firing_ready_at.is_some())
            {
                self.finish(
                    intent.command_id,
                    actor.id,
                    false,
                    "The wooden point is already hardening.",
                    results,
                );
                continue;
            }
            let ready = self.time.saturating_add(self.tuning.spear_ticks);
            if let Some(spear) = self.items.get_mut(&id) {
                spear.firing_ready_at = Some(ready);
            }
            self.schedule(ready, crate::model::Wake::HardenSpear(id));
            let name = self.name_of(actor.id);
            let rest = "the wooden point in the fire.";
            self.emit(
                crate::model::spoken_act(Some(name.as_str()), "hold", rest),
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
                "You hold the wooden point in the fire. It is not hard yet.",
                results,
            );
        }
    }

    fn spear_can_hunt(&self, actor: crate::model::ActorId) -> bool {
        self.loose_ids(actor, Material::Spear)
            .iter()
            .any(|id| self.item(*id).is_some_and(|spear| self.spear_is_set(spear)))
    }

    /// The hardened spear that brought the deer down, when it is only a wooden point.
    /// A flint or stone point on the same person keeps the older line.
    fn wooden_kill(&self, actor: crate::model::ActorId) -> Option<crate::model::ItemId> {
        let spears = self.loose_ids(actor, Material::Spear);
        let other = spears.iter().any(|id| {
            self.item(*id)
                .is_some_and(|spear| self.spear_is_set(spear) && !spear.wooden_tip)
        });
        if other {
            return None;
        }
        spears.into_iter().find(|id| {
            self.item(*id)
                .is_some_and(|spear| spear.wooden_tip && spear.hardened)
        })
    }

    /// A flint or stone point reaches the next cell. A wooden point does not.
    fn spear_reaches_next(&self, actor: crate::model::ActorId) -> bool {
        self.loose_ids(actor, Material::Spear).iter().any(|id| {
            self.item(*id)
                .is_some_and(|spear| self.spear_is_set(spear) && !spear.wooden_tip)
        })
    }

    /// Any hardened spear reaches the animal's own cell. Only a flint or
    /// stone point reaches the next cell.
    fn spear_in_reach(&self, actor: crate::model::ActorId, dist: i32) -> bool {
        if dist == 0 {
            self.spear_can_hunt(actor)
        } else if dist == 1 {
            self.spear_reaches_next(actor)
        } else {
            false
        }
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

    /// The otter is nearer than the deer and nearer than a standing wolf.
    fn otter_is_nearest(&self, actor: crate::model::ActorId) -> bool {
        let Some(person) = self.actor(actor) else {
            return false;
        };
        let Some(otter) = self.otter.as_ref() else {
            return false;
        };
        let dist = crate::model::chebyshev(person.cell, otter.cell);
        if self
            .deer
            .as_ref()
            .is_some_and(|deer| dist >= crate::model::chebyshev(person.cell, deer.cell))
        {
            return false;
        }
        if self
            .fox
            .as_ref()
            .is_some_and(|fox| dist > crate::model::chebyshev(person.cell, fox.cell))
        {
            return false;
        }
        if self
            .wolf_standing()
            .is_some_and(|wolf| dist >= crate::model::chebyshev(person.cell, wolf))
        {
            return false;
        }
        if self
            .hare
            .as_ref()
            .is_some_and(|hare| crate::model::chebyshev(person.cell, hare.cell) < dist)
        {
            return false;
        }
        if self
            .beaver
            .as_ref()
            .is_some_and(|beaver| crate::model::chebyshev(person.cell, beaver.cell) < dist)
        {
            return false;
        }
        // A wolf this weapon can already reach stays the quarry. The otter
        // does not step in front of it.
        let wolf_dist = self
            .wolf_standing()
            .map(|wolf| crate::model::chebyshev(person.cell, wolf));
        let wolf_in_reach = wolf_dist.is_some_and(|far| {
            self.spear_in_reach(actor, far) || (self.bow_in_hand(actor) && far <= self.tuning.sight)
        });
        if wolf_in_reach {
            return false;
        }
        true
    }

    /// The beaver is nearer than the deer, the otter, the fox, and the hare.
    /// A wolf this weapon can already reach stays the quarry.
    fn beaver_is_nearest(&self, actor: crate::model::ActorId) -> bool {
        let Some(person) = self.actor(actor) else {
            return false;
        };
        let Some(beaver) = self.beaver.as_ref() else {
            return false;
        };
        let dist = crate::model::chebyshev(person.cell, beaver.cell);
        if self
            .deer
            .as_ref()
            .is_some_and(|deer| dist >= crate::model::chebyshev(person.cell, deer.cell))
        {
            return false;
        }
        if self
            .otter
            .as_ref()
            .is_some_and(|otter| dist >= crate::model::chebyshev(person.cell, otter.cell))
        {
            return false;
        }
        if self
            .fox
            .as_ref()
            .is_some_and(|fox| dist >= crate::model::chebyshev(person.cell, fox.cell))
        {
            return false;
        }
        if self
            .hare
            .as_ref()
            .is_some_and(|hare| crate::model::chebyshev(person.cell, hare.cell) < dist)
        {
            return false;
        }
        if self
            .wolf_standing()
            .is_some_and(|wolf| dist >= crate::model::chebyshev(person.cell, wolf))
        {
            return false;
        }
        let wolf_dist = self
            .wolf_standing()
            .map(|wolf| crate::model::chebyshev(person.cell, wolf));
        let wolf_in_reach = wolf_dist.is_some_and(|far| {
            self.spear_in_reach(actor, far) || (self.bow_in_hand(actor) && far <= self.tuning.sight)
        });
        if wolf_in_reach {
            return false;
        }
        true
    }

    /// A spear on the beaver's cell or the next, or a bow out to sight,
    /// and only when the beaver is the nearer animal.
    fn strike_the_beaver(
        &mut self,
        intent: &Intent,
        actor: &crate::model::Actor,
        results: &mut Vec<CommandResult>,
    ) -> bool {
        if !self.beaver_is_nearest(actor.id) {
            return false;
        }
        let Some(beaver) = self.beaver.clone() else {
            return false;
        };
        let dist = crate::model::chebyshev(actor.cell, beaver.cell);
        if self.spear_can_hunt(actor.id) && dist == 0 {
            self.bring_down_beaver(intent, beaver.cell, None, results);
            return true;
        }
        if self.spear_reaches_next(actor.id) && dist == 1 {
            self.contest_the_beaver(intent, actor, &beaver, results);
            return true;
        }
        if self.bow_in_hand(actor.id) && dist <= self.tuning.sight {
            self.loose_the_bow_at_the_beaver(intent, actor, beaver.cell, results);
            return true;
        }
        false
    }

    fn bring_down_beaver(
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
        self.beaver = None;
        self.arm_beaver_return();
        let id = self.mint_item(Material::Carcass, 0, cell, None, Role::Loose);
        if let Some(carcass) = self.items.get_mut(&id) {
            carcass.beaver = true;
        }
        let (heard, yours) = if flint {
            let rest = "the beaver down with the flint spear.";
            (
                crate::model::spoken_act(Some(name.as_str()), "bring", rest),
                crate::model::spoken_act(None, "bring", rest),
            )
        } else {
            (
                format!("{name} brings the beaver down. A beaver carcass lies where it stood."),
                "You bring the beaver down. A beaver carcass lies where it stood.".to_string(),
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

    fn loose_the_bow_at_the_beaver(
        &mut self,
        intent: &Intent,
        actor: &crate::model::Actor,
        cell: crate::model::CellId,
        results: &mut Vec<CommandResult>,
    ) {
        if let Some(flake) = self.loose_ids(actor.id, Material::Flake).first().copied() {
            self.items.remove(&flake);
        }
        self.beaver = None;
        self.arm_beaver_return();
        let id = self.mint_item(Material::Carcass, 0, cell, None, Role::Loose);
        if let Some(carcass) = self.items.get_mut(&id) {
            carcass.beaver = true;
        }
        let name = self.name_of(actor.id);
        let rest = "the beaver down with the bow. The flint flake is spent.";
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

    /// The next cell. One of three, from the same draw the otter uses.
    /// A miss leaves no carcass. The beaver splashes to the other of its
    /// brook cell and the bank, unless that landing is taken.
    fn contest_the_beaver(
        &mut self,
        intent: &Intent,
        actor: &crate::model::Actor,
        beaver: &crate::model::Beaver,
        results: &mut Vec<CommandResult>,
    ) {
        let (x, y) = crate::model::cell_xy(beaver.cell);
        let subject = format!("{x},{y}:beaver");
        let hash = contest_hash(self.seed, self.time, "hunt", &subject, actor.id);
        let draw = Some(Draw {
            tag: "hunt".into(),
            subject,
            value: hash,
        });
        if hash % 3 == 0 {
            self.bring_down_beaver(intent, beaver.cell, draw, results);
            return;
        }
        let cell = beaver.cell;
        let way = self.splash_the_beaver(actor.cell);
        let name = self.name_of(actor.id);
        let flint = self
            .loose_ids(actor.id, Material::Spear)
            .iter()
            .any(|id| self.item(*id).is_some_and(|spear| spear.flint_tip));
        let (heard, yours) = if hash % 3 == 1 {
            match (flint, way) {
                (true, Some(way)) => (
                    format!(
                        "{name} misses the beaver with the flint spear. It splashes away to the {way}."
                    ),
                    format!(
                        "You miss the beaver with the flint spear. It splashes away to the {way}."
                    ),
                ),
                (false, Some(way)) => (
                    format!("{name} misses the beaver. It splashes away to the {way}."),
                    format!("You miss the beaver. It splashes away to the {way}."),
                ),
                (true, None) => (
                    format!("{name} misses the beaver with the flint spear. It stays."),
                    "You miss the beaver with the flint spear. It stays.".to_string(),
                ),
                (false, None) => (
                    format!("{name} misses the beaver. It stays."),
                    "You miss the beaver. It stays.".to_string(),
                ),
            }
        } else {
            match (flint, way) {
                (true, Some(way)) => (
                    format!(
                        "The beaver splashes away to the {way}. {name}'s flint spear does not land."
                    ),
                    format!(
                        "The beaver splashes away to the {way}. Your flint spear does not land."
                    ),
                ),
                (false, Some(way)) => (
                    format!("The beaver splashes away to the {way}. {name}'s spear does not land."),
                    format!("The beaver splashes away to the {way}. Your spear does not land."),
                ),
                (true, None) => (
                    format!("The beaver stays. {name}'s flint spear does not land."),
                    "The beaver stays. Your flint spear does not land.".to_string(),
                ),
                (false, None) => (
                    format!("The beaver stays. {name}'s spear does not land."),
                    "The beaver stays. Your spear does not land.".to_string(),
                ),
            }
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

    /// Onto the other of the brook and the bank, away from the hunter.
    /// None when that landing is the hunter, a person, or the otter.
    fn splash_the_beaver(&mut self, hunter: crate::model::CellId) -> Option<&'static str> {
        let beaver = self.beaver.clone()?;
        let next = self.beaver_aside(beaver.cell)?;
        if next == hunter || next == beaver.cell {
            return None;
        }
        if self.actors.values().any(|person| person.cell == next) {
            return None;
        }
        if self.otter.as_ref().is_some_and(|otter| otter.cell == next) {
            return None;
        }
        let (x1, y1) = crate::model::cell_xy(beaver.cell);
        let (x2, y2) = crate::model::cell_xy(next);
        let word = if x2 > x1 {
            "east"
        } else if x2 < x1 {
            "west"
        } else if y2 > y1 {
            "south"
        } else {
            "north"
        };
        if let Some(beaver) = self.beaver.as_mut() {
            beaver.cell = next;
            beaver.held = true;
        }
        Some(word)
    }

    /// A spear on the otter's cell or the next, or a bow out to sight,
    /// and only when the otter is the nearer animal.
    fn strike_the_otter(
        &mut self,
        intent: &Intent,
        actor: &crate::model::Actor,
        results: &mut Vec<CommandResult>,
    ) -> bool {
        if !self.otter_is_nearest(actor.id) {
            return false;
        }
        let Some(otter) = self.otter.clone() else {
            return false;
        };
        let dist = crate::model::chebyshev(actor.cell, otter.cell);
        if self.spear_can_hunt(actor.id) && dist == 0 {
            self.bring_down_otter(intent, otter.cell, None, results);
            return true;
        }
        if self.spear_reaches_next(actor.id) && dist == 1 {
            self.contest_the_otter(intent, actor, &otter, results);
            return true;
        }
        if self.bow_in_hand(actor.id) && dist <= self.tuning.sight {
            self.loose_the_bow_at_the_otter(intent, actor, otter.cell, results);
            return true;
        }
        false
    }

    /// The fox is nearer than the deer, the otter, and a standing wolf.
    /// A wolf this weapon can already reach stays the quarry.
    fn fox_is_nearest(&self, actor: crate::model::ActorId) -> bool {
        let Some(person) = self.actor(actor) else {
            return false;
        };
        let Some(fox) = self.fox.as_ref() else {
            return false;
        };
        let dist = crate::model::chebyshev(person.cell, fox.cell);
        if self
            .deer
            .as_ref()
            .is_some_and(|deer| dist >= crate::model::chebyshev(person.cell, deer.cell))
        {
            return false;
        }
        if self
            .otter
            .as_ref()
            .is_some_and(|otter| dist >= crate::model::chebyshev(person.cell, otter.cell))
        {
            return false;
        }
        if self
            .wolf_standing()
            .is_some_and(|wolf| dist >= crate::model::chebyshev(person.cell, wolf))
        {
            return false;
        }
        if self
            .hare
            .as_ref()
            .is_some_and(|hare| crate::model::chebyshev(person.cell, hare.cell) < dist)
        {
            return false;
        }
        if self
            .beaver
            .as_ref()
            .is_some_and(|beaver| crate::model::chebyshev(person.cell, beaver.cell) < dist)
        {
            return false;
        }
        let wolf_dist = self
            .wolf_standing()
            .map(|wolf| crate::model::chebyshev(person.cell, wolf));
        let wolf_in_reach = wolf_dist.is_some_and(|far| {
            self.spear_in_reach(actor, far) || (self.bow_in_hand(actor) && far <= self.tuning.sight)
        });
        if wolf_in_reach {
            return false;
        }
        true
    }

    /// A spear on the fox's cell or the next, or a bow out to sight,
    /// and only when the fox is the nearer animal.
    fn strike_the_fox(
        &mut self,
        intent: &Intent,
        actor: &crate::model::Actor,
        results: &mut Vec<CommandResult>,
    ) -> bool {
        if !self.fox_is_nearest(actor.id) {
            return false;
        }
        let Some(fox) = self.fox.clone() else {
            return false;
        };
        let dist = crate::model::chebyshev(actor.cell, fox.cell);
        if self.spear_can_hunt(actor.id) && dist == 0 {
            self.bring_down_fox(intent, fox.cell, None, results);
            return true;
        }
        if self.spear_reaches_next(actor.id) && dist == 1 {
            self.contest_the_fox(intent, actor, &fox, results);
            return true;
        }
        if self.bow_in_hand(actor.id) && dist <= self.tuning.sight {
            self.loose_the_bow_at_the_fox(intent, actor, fox.cell, results);
            return true;
        }
        false
    }

    fn bring_down_fox(
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
        self.fox = None;
        self.arm_fox_return();
        let id = self.mint_item(Material::Carcass, 0, cell, None, Role::Loose);
        if let Some(carcass) = self.items.get_mut(&id) {
            carcass.fox = true;
        }
        let (heard, yours) = if flint {
            let rest = "the fox down with the flint spear.";
            (
                crate::model::spoken_act(Some(name.as_str()), "bring", rest),
                crate::model::spoken_act(None, "bring", rest),
            )
        } else {
            (
                format!("{name} brings the fox down. A fox carcass lies where it stood."),
                "You bring the fox down. A fox carcass lies where it stood.".to_string(),
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

    fn loose_the_bow_at_the_fox(
        &mut self,
        intent: &Intent,
        actor: &crate::model::Actor,
        cell: crate::model::CellId,
        results: &mut Vec<CommandResult>,
    ) {
        if let Some(flake) = self.loose_ids(actor.id, Material::Flake).first().copied() {
            self.items.remove(&flake);
        }
        self.fox = None;
        self.arm_fox_return();
        let id = self.mint_item(Material::Carcass, 0, cell, None, Role::Loose);
        if let Some(carcass) = self.items.get_mut(&id) {
            carcass.fox = true;
        }
        let name = self.name_of(actor.id);
        let rest = "the fox down with the bow. The flint flake is spent.";
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
    /// A miss or a bolt leaves no carcass. The fox slips away along the floor
    /// when the route has a cell that is not the hunter.
    fn contest_the_fox(
        &mut self,
        intent: &Intent,
        actor: &crate::model::Actor,
        fox: &crate::model::Fox,
        results: &mut Vec<CommandResult>,
    ) {
        let (x, y) = crate::model::cell_xy(fox.cell);
        let subject = format!("{x},{y}:fox");
        let hash = contest_hash(self.seed, self.time, "hunt", &subject, actor.id);
        let draw = Some(Draw {
            tag: "hunt".into(),
            subject,
            value: hash,
        });
        if hash % 3 == 0 {
            self.bring_down_fox(intent, fox.cell, draw, results);
            return;
        }
        let cell = fox.cell;
        let way = self.slip_the_fox(actor.cell);
        let name = self.name_of(actor.id);
        let flint = self
            .loose_ids(actor.id, Material::Spear)
            .iter()
            .any(|id| self.item(*id).is_some_and(|spear| spear.flint_tip));
        let (heard, yours) = if hash % 3 == 1 {
            match (flint, way) {
                (true, Some(way)) => (
                    format!(
                        "{name} misses the fox with the flint spear. It slips away to the {way}."
                    ),
                    format!("You miss the fox with the flint spear. It slips away to the {way}."),
                ),
                (false, Some(way)) => (
                    format!("{name} misses the fox. It slips away to the {way}."),
                    format!("You miss the fox. It slips away to the {way}."),
                ),
                (true, None) => (
                    format!("{name} misses the fox with the flint spear. It stays."),
                    "You miss the fox with the flint spear. It stays.".to_string(),
                ),
                (false, None) => (
                    format!("{name} misses the fox. It stays."),
                    "You miss the fox. It stays.".to_string(),
                ),
            }
        } else {
            match (flint, way) {
                (true, Some(way)) => (
                    format!("The fox slips away to the {way}. {name}'s flint spear does not land."),
                    format!("The fox slips away to the {way}. Your flint spear does not land."),
                ),
                (false, Some(way)) => (
                    format!("The fox slips away to the {way}. {name}'s spear does not land."),
                    format!("The fox slips away to the {way}. Your spear does not land."),
                ),
                (true, None) => (
                    format!("The fox stays. {name}'s flint spear does not land."),
                    "The fox stays. Your flint spear does not land.".to_string(),
                ),
                (false, None) => (
                    format!("The fox stays. {name}'s spear does not land."),
                    "The fox stays. Your spear does not land.".to_string(),
                ),
            }
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

    /// One step along the floor, away from the hunter. None when that cell is the hunter.
    fn slip_the_fox(&mut self, hunter: crate::model::CellId) -> Option<&'static str> {
        let fox = self.fox.clone()?;
        let (next_at, next) = self.fox_away(fox.route_at, hunter)?;
        let (x1, y1) = crate::model::cell_xy(fox.cell);
        let (x2, y2) = crate::model::cell_xy(next);
        let word = if x2 > x1 {
            "east"
        } else if x2 < x1 {
            "west"
        } else if y2 > y1 {
            "south"
        } else {
            "north"
        };
        if let Some(fox) = self.fox.as_mut() {
            fox.cell = next;
            fox.route_at = next_at;
        }
        Some(word)
    }

    fn bring_down_otter(
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
        self.otter = None;
        self.arm_otter_return();
        let id = self.mint_item(Material::Carcass, 0, cell, None, Role::Loose);
        if let Some(carcass) = self.items.get_mut(&id) {
            carcass.otter = true;
        }
        let (heard, yours) = if flint {
            let rest = "the otter down with the flint spear.";
            (
                crate::model::spoken_act(Some(name.as_str()), "bring", rest),
                crate::model::spoken_act(None, "bring", rest),
            )
        } else {
            (
                format!("{name} brings the otter down. An otter carcass lies where it stood."),
                "You bring the otter down. An otter carcass lies where it stood.".to_string(),
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

    fn loose_the_bow_at_the_otter(
        &mut self,
        intent: &Intent,
        actor: &crate::model::Actor,
        cell: crate::model::CellId,
        results: &mut Vec<CommandResult>,
    ) {
        if let Some(flake) = self.loose_ids(actor.id, Material::Flake).first().copied() {
            self.items.remove(&flake);
        }
        self.otter = None;
        self.arm_otter_return();
        let id = self.mint_item(Material::Carcass, 0, cell, None, Role::Loose);
        if let Some(carcass) = self.items.get_mut(&id) {
            carcass.otter = true;
        }
        let name = self.name_of(actor.id);
        let rest = "the otter down with the bow. The flint flake is spent.";
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
    /// A miss or a bolt leaves no carcass. The otter splashes away along the brook.
    fn contest_the_otter(
        &mut self,
        intent: &Intent,
        actor: &crate::model::Actor,
        otter: &crate::model::Otter,
        results: &mut Vec<CommandResult>,
    ) {
        let (x, y) = crate::model::cell_xy(otter.cell);
        let subject = format!("{x},{y}:otter");
        let hash = contest_hash(self.seed, self.time, "hunt", &subject, actor.id);
        let draw = Some(Draw {
            tag: "hunt".into(),
            subject,
            value: hash,
        });
        if hash % 3 == 0 {
            self.bring_down_otter(intent, otter.cell, draw, results);
            return;
        }
        let cell = otter.cell;
        let way = self.splash_the_otter(actor.cell).unwrap_or("away");
        let name = self.name_of(actor.id);
        let flint = self
            .loose_ids(actor.id, Material::Spear)
            .iter()
            .any(|id| self.item(*id).is_some_and(|spear| spear.flint_tip));
        let (heard, yours) = if hash % 3 == 1 {
            if flint {
                (
                    format!(
                        "{name} misses the otter with the flint spear. It splashes away to the {way}."
                    ),
                    format!("You miss the otter with the flint spear. It splashes away to the {way}."),
                )
            } else {
                (
                    format!("{name} misses the otter. It splashes away to the {way}."),
                    format!("You miss the otter. It splashes away to the {way}."),
                )
            }
        } else if flint {
            (
                format!(
                    "The otter splashes away to the {way}. {name}'s flint spear does not land."
                ),
                format!("The otter splashes away to the {way}. Your flint spear does not land."),
            )
        } else {
            (
                format!("The otter splashes away to the {way}. {name}'s spear does not land."),
                format!("The otter splashes away to the {way}. Your spear does not land."),
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

    /// One step along the brook, away from the hunter. The word is the direction it went.
    fn splash_the_otter(&mut self, hunter: crate::model::CellId) -> Option<&'static str> {
        let otter = self.otter.clone()?;
        let (next_at, next) = self.otter_away(otter.route_at, hunter)?;
        let (x1, y1) = crate::model::cell_xy(otter.cell);
        let (x2, y2) = crate::model::cell_xy(next);
        let word = if x2 > x1 {
            "east"
        } else if x2 < x1 {
            "west"
        } else if y2 > y1 {
            "south"
        } else {
            "north"
        };
        if let Some(otter) = self.otter.as_mut() {
            otter.cell = next;
            otter.route_at = next_at;
        }
        Some(word)
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
        if self.spear_reaches_next(actor.id) && dist == 1 {
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

    /// The hare is nearer than the deer, the fox, the otter, and a standing wolf.
    /// A wolf this weapon can already reach stays the quarry.
    fn hare_is_nearest(&self, actor: crate::model::ActorId) -> bool {
        let Some(person) = self.actor(actor) else {
            return false;
        };
        let Some(hare) = self.hare.as_ref() else {
            return false;
        };
        let dist = crate::model::chebyshev(person.cell, hare.cell);
        if self
            .deer
            .as_ref()
            .is_some_and(|deer| dist >= crate::model::chebyshev(person.cell, deer.cell))
        {
            return false;
        }
        if self
            .fox
            .as_ref()
            .is_some_and(|fox| dist >= crate::model::chebyshev(person.cell, fox.cell))
        {
            return false;
        }
        if self
            .otter
            .as_ref()
            .is_some_and(|otter| dist >= crate::model::chebyshev(person.cell, otter.cell))
        {
            return false;
        }
        if self
            .beaver
            .as_ref()
            .is_some_and(|beaver| crate::model::chebyshev(person.cell, beaver.cell) < dist)
        {
            return false;
        }
        if self
            .wolf_standing()
            .is_some_and(|wolf| dist >= crate::model::chebyshev(person.cell, wolf))
        {
            return false;
        }
        let wolf_dist = self
            .wolf_standing()
            .map(|wolf| crate::model::chebyshev(person.cell, wolf));
        let wolf_in_reach = wolf_dist.is_some_and(|far| {
            self.spear_in_reach(actor, far) || (self.bow_in_hand(actor) && far <= self.tuning.sight)
        });
        if wolf_in_reach {
            return false;
        }
        true
    }

    /// A spear on the hare's cell or the next, or a bow out to sight,
    /// and only when the hare is the nearer animal.
    fn strike_the_hare(
        &mut self,
        intent: &Intent,
        actor: &crate::model::Actor,
        results: &mut Vec<CommandResult>,
    ) -> bool {
        if !self.hare_is_nearest(actor.id) {
            return false;
        }
        let Some(hare) = self.hare.clone() else {
            return false;
        };
        let dist = crate::model::chebyshev(actor.cell, hare.cell);
        if self.spear_can_hunt(actor.id) && dist == 0 {
            self.bring_down_hare(intent, hare.cell, None, results);
            return true;
        }
        if self.spear_reaches_next(actor.id) && dist == 1 {
            self.contest_the_hare(intent, actor, hare.cell, results);
            return true;
        }
        if self.bow_in_hand(actor.id) && dist <= self.tuning.sight {
            self.loose_the_bow_at_the_hare(intent, actor, hare.cell, results);
            return true;
        }
        false
    }

    fn bring_down_hare(
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
        self.hare = None;
        self.arm_hare_return();
        let id = self.mint_item(Material::Carcass, 0, cell, None, Role::Loose);
        if let Some(carcass) = self.items.get_mut(&id) {
            carcass.hare = true;
        }
        let (heard, yours) = if flint {
            let rest = "the hare down with the flint spear.";
            (
                crate::model::spoken_act(Some(name.as_str()), "bring", rest),
                crate::model::spoken_act(None, "bring", rest),
            )
        } else {
            (
                format!("{name} brings the hare down. A hare carcass lies where it stood."),
                "You bring the hare down. A hare carcass lies where it stood.".to_string(),
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

    fn loose_the_bow_at_the_hare(
        &mut self,
        intent: &Intent,
        actor: &crate::model::Actor,
        cell: crate::model::CellId,
        results: &mut Vec<CommandResult>,
    ) {
        if let Some(flake) = self.loose_ids(actor.id, Material::Flake).first().copied() {
            self.items.remove(&flake);
        }
        self.hare = None;
        self.arm_hare_return();
        let id = self.mint_item(Material::Carcass, 0, cell, None, Role::Loose);
        if let Some(carcass) = self.items.get_mut(&id) {
            carcass.hare = true;
        }
        let name = self.name_of(actor.id);
        let rest = "the hare down with the bow. The flint flake is spent.";
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

    /// The next cell. One of three, from the same draw the fox uses.
    /// A miss or a bolt leaves no carcass. The hare bolts to a dry neighbor.
    fn contest_the_hare(
        &mut self,
        intent: &Intent,
        actor: &crate::model::Actor,
        cell: crate::model::CellId,
        results: &mut Vec<CommandResult>,
    ) {
        let (x, y) = crate::model::cell_xy(cell);
        let subject = format!("{x},{y}:hare");
        let hash = contest_hash(self.seed, self.time, "hunt", &subject, actor.id);
        let draw = Some(Draw {
            tag: "hunt".into(),
            subject,
            value: hash,
        });
        if hash % 3 == 0 {
            self.bring_down_hare(intent, cell, draw, results);
            return;
        }
        let way = self.bolt_hunted_hare(actor.cell);
        let name = self.name_of(actor.id);
        let flint = self
            .loose_ids(actor.id, Material::Spear)
            .iter()
            .any(|id| self.item(*id).is_some_and(|spear| spear.flint_tip));
        let (heard, yours) = if hash % 3 == 1 {
            match (flint, way) {
                (true, Some(way)) => (
                    format!("{name} misses the hare with the flint spear. It bolts to the {way}."),
                    format!("You miss the hare with the flint spear. It bolts to the {way}."),
                ),
                (false, Some(way)) => (
                    format!("{name} misses the hare. It bolts to the {way}."),
                    format!("You miss the hare. It bolts to the {way}."),
                ),
                (true, None) => (
                    format!("{name} misses the hare with the flint spear. It stays."),
                    "You miss the hare with the flint spear. It stays.".to_string(),
                ),
                (false, None) => (
                    format!("{name} misses the hare. It stays."),
                    "You miss the hare. It stays.".to_string(),
                ),
            }
        } else {
            match (flint, way) {
                (true, Some(way)) => (
                    format!("The hare bolts to the {way}. {name}'s flint spear does not land."),
                    format!("The hare bolts to the {way}. Your flint spear does not land."),
                ),
                (false, Some(way)) => (
                    format!("The hare bolts to the {way}. {name}'s spear does not land."),
                    format!("The hare bolts to the {way}. Your spear does not land."),
                ),
                (true, None) => (
                    format!("The hare stays. {name}'s flint spear does not land."),
                    "The hare stays. Your flint spear does not land.".to_string(),
                ),
                (false, None) => (
                    format!("The hare stays. {name}'s spear does not land."),
                    "The hare stays. Your spear does not land.".to_string(),
                ),
            }
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
                    "You are not carrying a spear.".to_string()
                } else {
                    self.unhardened_spear_line(actor.id)
                };
                self.finish(intent.command_id, actor.id, false, message, results);
                continue;
            }
            if self.fox_is_nearest(actor.id) {
                if self.strike_the_fox(&intent, &actor, results) {
                    continue;
                }
                self.finish(
                    intent.command_id,
                    actor.id,
                    false,
                    "The fox is not close enough to hunt.",
                    results,
                );
                continue;
            }
            if self.otter_is_nearest(actor.id) {
                if self.strike_the_otter(&intent, &actor, results) {
                    continue;
                }
                self.finish(
                    intent.command_id,
                    actor.id,
                    false,
                    "The otter is not close enough to hunt.",
                    results,
                );
                continue;
            }
            if self.beaver_is_nearest(actor.id) {
                if self.strike_the_beaver(&intent, &actor, results) {
                    continue;
                }
                self.finish(
                    intent.command_id,
                    actor.id,
                    false,
                    "The beaver is not close enough to hunt.",
                    results,
                );
                continue;
            }
            if self.hare_is_nearest(actor.id) {
                if self.strike_the_hare(&intent, &actor, results) {
                    continue;
                }
                self.finish(
                    intent.command_id,
                    actor.id,
                    false,
                    "The hare is not close enough to hunt.",
                    results,
                );
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
            if !self.spear_reaches_next(actor.id) {
                let message = "The deer is not close enough to hunt.";
                self.emit(
                    message,
                    Scope::Personal(actor.id),
                    vec![actor.id],
                    vec![actor.cell],
                    vec![],
                    vec![],
                    None,
                );
                self.finish(intent.command_id, actor.id, false, message, results);
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
            let otter = self.item(id).is_some_and(|hide| hide.otter);
            let fox = self.item(id).is_some_and(|hide| hide.fox);
            let hare = self.item(id).is_some_and(|hide| hide.hare);
            let beaver = self.item(id).is_some_and(|hide| hide.beaver);
            let rest = worn_phrase(wolf, otter, fox, hare, beaver, sewn);
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
            let otter = self.item(id).is_some_and(|hide| hide.otter);
            let fox = self.item(id).is_some_and(|hide| hide.fox);
            let hare = self.item(id).is_some_and(|hide| hide.hare);
            let beaver = self.item(id).is_some_and(|hide| hide.beaver);
            let rest = worn_phrase(wolf, otter, fox, hare, beaver, sewn);
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
            let carcasses = self.ground_ids(actor.cell, Material::Carcass);
            let otter = carcasses
                .iter()
                .copied()
                .find(|id| self.item(*id).is_some_and(|carcass| carcass.otter));
            let fox = carcasses
                .iter()
                .copied()
                .find(|id| self.item(*id).is_some_and(|carcass| carcass.fox));
            let hare = carcasses
                .iter()
                .copied()
                .find(|id| self.item(*id).is_some_and(|carcass| carcass.hare));
            let beaver = carcasses
                .iter()
                .copied()
                .find(|id| self.item(*id).is_some_and(|carcass| carcass.beaver));
            let other = carcasses.iter().copied().find(|id| {
                self.item(*id).is_some_and(|carcass| {
                    !carcass.otter && !carcass.fox && !carcass.hare && !carcass.beaver
                })
            });
            let Some(carcass) = other.or(fox).or(otter).or(hare).or(beaver) else {
                self.finish(
                    intent.command_id,
                    actor.id,
                    false,
                    "There is no carcass here to butcher.",
                    results,
                );
                continue;
            };
            let otter = self.item(carcass).is_some_and(|carcass| carcass.otter);
            let fox = self.item(carcass).is_some_and(|carcass| carcass.fox);
            let hare = self.item(carcass).is_some_and(|carcass| carcass.hare);
            let beaver = self.item(carcass).is_some_and(|carcass| carcass.beaver);
            let wolf = self.item(carcass).is_some_and(|carcass| carcass.wolf);
            self.items.remove(&carcass);
            let mut ids = Vec::new();
            let portions = if otter || fox || hare || beaver { 1 } else { 3 };
            for _ in 0..portions {
                ids.push(self.mint_item(Material::Meat, 0, actor.cell, None, Role::Loose));
            }
            let hide = self.mint_item(Material::Hide, 0, actor.cell, None, Role::Loose);
            if let Some(item) = self.items.get_mut(&hide) {
                item.wolf = wolf && !otter && !fox && !hare && !beaver;
                item.otter = otter;
                item.fox = fox;
                item.hare = hare;
                item.beaver = beaver;
            }
            ids.push(hide);
            ids.push(self.mint_item(Material::Bone, 0, actor.cell, None, Role::Loose));
            let name = self.name_of(actor.id);
            let rest = if otter && with_knife {
                "the otter carcass with the iron knife into a portion of raw meat, an otter hide, and a bone."
            } else if otter {
                "the otter carcass into a portion of raw meat, an otter hide, and a bone."
            } else if fox && with_knife {
                "the fox carcass with the iron knife into a portion of raw meat, a fox hide, and a bone."
            } else if fox {
                "the fox carcass into a portion of raw meat, a fox hide, and a bone."
            } else if hare && with_knife {
                "the hare carcass with the iron knife into a portion of raw meat, a hare hide, and a bone."
            } else if hare {
                "the hare carcass into a portion of raw meat, a hare hide, and a bone."
            } else if beaver && with_knife {
                "the beaver carcass with the iron knife into a portion of raw meat, a beaver hide, and a bone."
            } else if beaver {
                "the beaver carcass into a portion of raw meat, a beaver hide, and a bone."
            } else {
                if !wolf {
                    ids.push(self.mint_item(Material::Antler, 0, actor.cell, None, Role::Loose));
                }
                match (wolf, with_knife) {
                    (true, true) => {
                        "the wolf carcass with the iron knife into three portions of raw meat, a hide, and a bone."
                    }
                    (true, false) => {
                        "the wolf carcass into three portions of raw meat, a hide, and a bone."
                    }
                    (false, true) => {
                        "the carcass with the iron knife into three portions of raw meat, a hide, a bone, and an antler."
                    }
                    (false, false) => {
                        "the carcass into three portions of raw meat, a hide, a bone, and an antler."
                    }
                }
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
            return Err(ARMS_FULL);
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

    /// A raw fish that can be hung or set. One already waiting by the fire is not it.
    fn fish_to_hang(&self, actor: crate::model::ActorId) -> Option<crate::model::ItemId> {
        let ready = |id: crate::model::ItemId| {
            self.item(id).is_some_and(|fish| {
                !fish.cooked
                    && !fish.salted
                    && !fish.smoked
                    && !fish.spoiled
                    && fish.firing_ready_at.is_none()
            })
        };
        if let Some(id) = self
            .loose_ids(actor, Material::Fish)
            .into_iter()
            .find(|id| ready(*id))
        {
            return Some(id);
        }
        let cell = self.actor(actor)?.cell;
        self.ground_ids(cell, Material::Fish)
            .into_iter()
            .find(|id| ready(*id))
    }

    /// A raw fish already waiting on this cell. A second smoke does not start another wait.
    fn fish_already_hanging(&self, actor: crate::model::ActorId) -> bool {
        let Some(cell) = self.actor(actor).map(|person| person.cell) else {
            return false;
        };
        self.ground_ids(cell, Material::Fish).into_iter().any(|id| {
            self.item(id).is_some_and(|fish| {
                !fish.smoked && !fish.cooked && !fish.spoiled && fish.firing_ready_at.is_some()
            })
        })
    }

    /// Hang a raw fish in the smoke, or set it on the next cell. It is not smoked yet.
    /// The fire is not used up. When the wait ends, the fish keeps and can be eaten cold.
    fn apply_smoke_fish(&mut self, smokes: Vec<Intent>, results: &mut Vec<CommandResult>) {
        for intent in smokes {
            let Some(actor) = self.actor(intent.actor_id).cloned() else {
                continue;
            };
            if !self.fire_near(actor.cell) {
                self.finish(
                    intent.command_id,
                    actor.id,
                    false,
                    "There is no fire burning here.",
                    results,
                );
                continue;
            }
            let Some(fish_id) = self.fish_to_hang(actor.id) else {
                let message = if self.fish_already_hanging(actor.id) {
                    "A raw fish is already hanging by the fire."
                } else if self.smoked_fish_id(actor.id).is_some() {
                    "The fish is already smoked."
                } else if self.spoiled_fish_near(actor.id) {
                    "The fish has spoiled."
                } else {
                    "There is no raw fish here to smoke."
                };
                self.finish(intent.command_id, actor.id, false, message, results);
                continue;
            };
            let on_the_fire = self.fire_at(actor.cell).is_some();
            let ready = self.time.saturating_add(self.tuning.clay_ticks);
            if let Some(fish) = self.items.get_mut(&fish_id) {
                fish.carrier = None;
                fish.cell = actor.cell;
                fish.in_basket = false;
                fish.firing_ready_at = Some(ready);
            }
            self.schedule(ready, crate::model::Wake::SmokeFish(fish_id));
            let name = self.name_of(actor.id);
            let (verb, rest, reply) = if on_the_fire {
                (
                    "hang",
                    "a raw fish by the fire.",
                    "You hang a raw fish by the fire. It is not smoked yet.",
                )
            } else {
                (
                    "set",
                    "a raw fish near the fire.",
                    "You set a raw fish near the fire. It is not smoked yet.",
                )
            };
            self.emit(
                crate::model::spoken_act(Some(name.as_str()), verb, rest),
                Scope::Cell(actor.cell),
                vec![actor.id],
                vec![actor.cell],
                vec![fish_id],
                vec![],
                None,
            );
            self.finish(intent.command_id, actor.id, true, reply, results);
        }
    }

    /// A raw portion that can be hung or set. One already waiting by the fire is not it.
    fn meat_to_hang(&self, actor: crate::model::ActorId) -> Option<crate::model::ItemId> {
        let ready = |id: crate::model::ItemId| {
            self.item(id).is_some_and(|meat| {
                !meat.cooked
                    && !meat.salted
                    && !meat.smoked
                    && !meat.spoiled
                    && meat.firing_ready_at.is_none()
            })
        };
        if let Some(id) = self
            .loose_ids(actor, Material::Meat)
            .into_iter()
            .find(|id| ready(*id))
        {
            return Some(id);
        }
        let cell = self.actor(actor)?.cell;
        self.ground_ids(cell, Material::Meat)
            .into_iter()
            .find(|id| ready(*id))
    }

    /// A raw portion already waiting on this cell. A second smoke does not start another wait.
    fn meat_already_hanging(&self, actor: crate::model::ActorId) -> bool {
        let Some(cell) = self.actor(actor).map(|person| person.cell) else {
            return false;
        };
        self.ground_ids(cell, Material::Meat).into_iter().any(|id| {
            self.item(id).is_some_and(|meat| {
                !meat.smoked && !meat.cooked && !meat.spoiled && meat.firing_ready_at.is_some()
            })
        })
    }

    /// Hang a portion of raw meat in the smoke, or set it on the next cell. It is not smoked yet.
    /// The fire is not used up. When the wait ends, the meat keeps and can be eaten cold.
    fn apply_smoke_meat(&mut self, smokes: Vec<Intent>, results: &mut Vec<CommandResult>) {
        for intent in smokes {
            let Some(actor) = self.actor(intent.actor_id).cloned() else {
                continue;
            };
            if !self.fire_near(actor.cell) {
                self.finish(
                    intent.command_id,
                    actor.id,
                    false,
                    "There is no fire burning here.",
                    results,
                );
                continue;
            }
            let Some(meat_id) = self.meat_to_hang(actor.id) else {
                let message = if self.meat_already_hanging(actor.id) {
                    "A portion of raw meat is already hanging by the fire."
                } else if self.smoked_meat_id(actor.id).is_some() {
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
            let on_the_fire = self.fire_at(actor.cell).is_some();
            let ready = self.time.saturating_add(self.tuning.clay_ticks);
            if let Some(meat) = self.items.get_mut(&meat_id) {
                meat.carrier = None;
                meat.cell = actor.cell;
                meat.in_basket = false;
                meat.firing_ready_at = Some(ready);
            }
            self.schedule(ready, crate::model::Wake::SmokeMeat(meat_id));
            let name = self.name_of(actor.id);
            let (verb, rest, reply) = if on_the_fire && small {
                (
                    "hang",
                    "a smaller piece of raw meat by the fire.",
                    "You hang a smaller piece of raw meat by the fire. It is not smoked yet.",
                )
            } else if on_the_fire {
                (
                    "hang",
                    "a portion of raw meat by the fire.",
                    "You hang a portion of raw meat by the fire. It is not smoked yet.",
                )
            } else if small {
                (
                    "set",
                    "a smaller piece of raw meat near the fire.",
                    "You set a smaller piece of raw meat near the fire. It is not smoked yet.",
                )
            } else {
                (
                    "set",
                    "a portion of raw meat near the fire.",
                    "You set a portion of raw meat near the fire. It is not smoked yet.",
                )
            };
            self.emit(
                crate::model::spoken_act(Some(name.as_str()), verb, rest),
                Scope::Cell(actor.cell),
                vec![actor.id],
                vec![actor.cell],
                vec![meat_id],
                vec![],
                None,
            );
            self.finish(intent.command_id, actor.id, true, reply, results);
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

    fn apply_needle_carves(&mut self, carves: Vec<Intent>, results: &mut Vec<CommandResult>) {
        for intent in carves {
            let Some(actor) = self.actor(intent.actor_id).cloned() else {
                continue;
            };
            if !self.has_bone(actor.id) {
                self.finish(
                    intent.command_id,
                    actor.id,
                    false,
                    "You have no bone to carve a needle.",
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
                    "You need a flint flake or an iron knife to carve a needle.",
                    results,
                );
                continue;
            }
            if !self.take_one_bone(actor.id, actor.cell) {
                self.finish(
                    intent.command_id,
                    actor.id,
                    false,
                    "You have no bone to carve a needle.",
                    results,
                );
                continue;
            }
            let id = self.mint_item(
                Material::BoneNeedle,
                0,
                actor.cell,
                Some(actor.id),
                Role::Loose,
            );
            let name = self.name_of(actor.id);
            let rest = if with_knife {
                "a bone into a bone needle with the iron knife."
            } else {
                "a bone into a bone needle with the flint flake."
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

    /// An antler into an antler pick. The flake or the knife stays. The antler is spent.
    fn apply_pick_carves(&mut self, carves: Vec<Intent>, results: &mut Vec<CommandResult>) {
        for intent in carves {
            let Some(actor) = self.actor(intent.actor_id).cloned() else {
                continue;
            };
            if !self.has_antler(actor.id) {
                self.finish(
                    intent.command_id,
                    actor.id,
                    false,
                    "You have no antler to carve.",
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
                    "You need a flint flake or an iron knife to carve an antler pick.",
                    results,
                );
                continue;
            }
            if !self.take_one_antler(actor.id, actor.cell) {
                self.finish(
                    intent.command_id,
                    actor.id,
                    false,
                    "You have no antler to carve.",
                    results,
                );
                continue;
            }
            let id = self.mint_item(Material::Pick, 0, actor.cell, Some(actor.id), Role::Loose);
            let name = self.name_of(actor.id);
            let rest = if with_knife {
                "an antler into an antler pick with the iron knife."
            } else {
                "an antler into an antler pick with the flint flake."
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

    /// The stitch, not the pierce. A pierced scraped hide, a bone needle, and
    /// cordage or sinew. The needle stays. The thread is spent.
    fn needle_sew_refusal(&self, actor: crate::model::ActorId) -> Option<String> {
        if self.loose_ids(actor, Material::Hide).is_empty() {
            return Some(
                self.hide_missing(actor, "You are not carrying a deer hide.")
                    .to_string(),
            );
        }
        let hides = self.loose_ids(actor, Material::Hide);
        let ready = hides.iter().any(|id| {
            self.item(*id)
                .is_some_and(|hide| hide.pierced && hide.scraped && !hide.sewn)
        });
        if ready {
            if !self.has_bone_needle(actor) {
                return Some("You are not carrying a bone needle.".into());
            }
            if !self.has_snare_line(actor) {
                return Some("You have no cordage or sinew to sew with.".into());
            }
            return None;
        }
        if hides.iter().any(|id| {
            self.item(*id)
                .is_some_and(|hide| !hide.scraped && !hide.sewn)
        }) {
            return Some(
                self.hide_line(
                    actor,
                    "The deer hide is not scraped.",
                    "The wolf hide is not scraped.",
                    "The otter hide is not scraped.",
                    "The fox hide is not scraped.",
                )
                .to_string(),
            );
        }
        if hides
            .iter()
            .any(|id| self.item(*id).is_some_and(|hide| hide.sewn))
        {
            return Some(
                self.hide_line(
                    actor,
                    "The deer hide is already a cloak.",
                    "The wolf hide is already a cloak.",
                    "The otter hide is already a cloak.",
                    "The fox hide is already a cloak.",
                )
                .to_string(),
            );
        }
        Some(self.unpierced_sew_line(actor))
    }

    fn has_bone_needle(&self, actor: crate::model::ActorId) -> bool {
        let cell = self.actor(actor).map(|person| person.cell).unwrap_or(0);
        !self.loose_ids(actor, Material::BoneNeedle).is_empty()
            || !self.ground_ids(cell, Material::BoneNeedle).is_empty()
    }

    fn apply_needle_sews(&mut self, sews: Vec<Intent>, results: &mut Vec<CommandResult>) {
        for intent in sews {
            let Some(actor) = self.actor(intent.actor_id).cloned() else {
                continue;
            };
            if let Some(message) = self.needle_sew_refusal(actor.id) {
                self.finish(intent.command_id, actor.id, false, message, results);
                continue;
            }
            let Some(line) = self.take_snare_line(actor.id, actor.cell) else {
                self.finish(
                    intent.command_id,
                    actor.id,
                    false,
                    "You have no cordage or sinew to sew with.",
                    results,
                );
                continue;
            };
            let hide = self
                .loose_ids(actor.id, Material::Hide)
                .into_iter()
                .find(|id| {
                    self.item(*id)
                        .is_some_and(|hide| hide.pierced && hide.scraped && !hide.sewn)
                });
            let Some(hide) = hide else {
                self.finish(
                    intent.command_id,
                    actor.id,
                    false,
                    "You are not carrying a deer hide.",
                    results,
                );
                continue;
            };
            let kind = if self.item(hide).is_some_and(|hide| hide.beaver) {
                "beaver"
            } else if self.item(hide).is_some_and(|hide| hide.hare) {
                "hare"
            } else if self.item(hide).is_some_and(|hide| hide.otter) {
                "otter"
            } else if self.item(hide).is_some_and(|hide| hide.fox) {
                "fox"
            } else if self.item(hide).is_some_and(|hide| hide.wolf) {
                "wolf"
            } else {
                "deer"
            };
            let thread = match line {
                Line::Hide => "hide strips",
                Line::Willow => "willow cordage",
                Line::Nettle => "nettle cordage",
                Line::Sinew => "sinew",
            };
            if let Some(item) = self.items.get_mut(&hide) {
                item.pierced = false;
                item.sewn = true;
            }
            let rest =
                format!("the pierced {kind} hide into a cloak with the bone needle and {thread}.");
            let name = self.name_of(actor.id);
            self.emit(
                crate::model::spoken_act(Some(name.as_str()), "sew", &rest),
                Scope::Cell(actor.cell),
                vec![actor.id],
                vec![actor.cell],
                vec![hide],
                vec![],
                None,
            );
            self.finish(
                intent.command_id,
                actor.id,
                true,
                crate::model::spoken_act(None, "sew", &rest),
                results,
            );
        }
    }

    pub(crate) fn has_bone(&self, actor: crate::model::ActorId) -> bool {
        let cell = self.actor(actor).map(|person| person.cell).unwrap_or(0);
        !self.loose_ids(actor, Material::Bone).is_empty()
            || !self.ground_ids(cell, Material::Bone).is_empty()
    }

    /// An antler in the arms, else one lying here. It is not a bone.
    pub(crate) fn has_antler(&self, actor: crate::model::ActorId) -> bool {
        let cell = self.actor(actor).map(|person| person.cell).unwrap_or(0);
        !self.loose_ids(actor, Material::Antler).is_empty()
            || !self.ground_ids(cell, Material::Antler).is_empty()
    }

    /// An antler pick in the arms. One on the ground is not in the hands.
    pub(crate) fn has_antler_pick(&self, actor: crate::model::ActorId) -> bool {
        self.loose_ids(actor, Material::Pick)
            .iter()
            .any(|id| self.item(*id).is_some_and(|pick| !pick.in_basket))
    }

    /// A pick in the hands, or one lying on this cell. A pick in a basket is neither.
    /// The hands win only in the sense that either one can dig. The pick is not lifted.
    fn pick_for_dry_dig(&self, actor: crate::model::ActorId, cell: crate::model::CellId) -> bool {
        let standing = self.actor(actor).is_some_and(|person| person.cell == cell);
        standing
            && (self.has_antler_pick(actor) || !self.ground_ids(cell, Material::Pick).is_empty())
    }

    /// A pick can dig here when the ground is not waiting out a previous dig.
    /// A pick at the feet digs without a gather, the same way a bead at the feet hammers.
    fn pick_can_dig_dry(&self, actor: crate::model::ActorId, cell: crate::model::CellId) -> bool {
        self.pick_for_dry_dig(actor, cell) && self.dry_worms_open(cell)
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

    /// Knife before flake. Either may be in the arms or lying on this cell. It is not spent.
    pub(crate) fn whistle_tool_rest(&self, actor: crate::model::ActorId) -> Option<&'static str> {
        let cell = self.actor(actor).map(|person| person.cell).unwrap_or(0);
        let knife = !self.loose_ids(actor, Material::Knife).is_empty()
            || !self.ground_ids(cell, Material::Knife).is_empty();
        let flake = !self.loose_ids(actor, Material::Flake).is_empty()
            || !self.ground_ids(cell, Material::Flake).is_empty();
        if knife {
            Some("a bone into a bone whistle with the iron knife.")
        } else if flake {
            Some("a bone into a bone whistle with the flint flake.")
        } else {
            None
        }
    }

    fn apply_whistle_carves(&mut self, carves: Vec<Intent>, results: &mut Vec<CommandResult>) {
        for intent in carves {
            let Some(actor) = self.actor(intent.actor_id).cloned() else {
                continue;
            };
            let Some(rest) = self.whistle_tool_rest(actor.id) else {
                self.finish(
                    intent.command_id,
                    actor.id,
                    false,
                    "You need a flint flake or an iron knife to carve a whistle.",
                    results,
                );
                continue;
            };
            if !self.take_one_bone(actor.id, actor.cell) {
                self.finish(
                    intent.command_id,
                    actor.id,
                    false,
                    "You have no bone to carve a whistle.",
                    results,
                );
                continue;
            }
            let id = self.mint_item(
                Material::Whistle,
                0,
                actor.cell,
                Some(actor.id),
                Role::Loose,
            );
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

    fn apply_whistle_blows(&mut self, blows: Vec<Intent>, results: &mut Vec<CommandResult>) {
        for intent in blows {
            let Some(actor) = self.actor(intent.actor_id).cloned() else {
                continue;
            };
            if self.loose_ids(actor.id, Material::Whistle).is_empty() {
                self.finish(
                    intent.command_id,
                    actor.id,
                    false,
                    "You are not carrying a bone whistle.",
                    results,
                );
                continue;
            }
            self.whistle = Some(crate::model::WhistleSound {
                cell: actor.cell,
                until: self.time.saturating_add(1),
            });
            self.whistle_frighten_hare(actor.cell);
            self.whistle_frighten_fox(actor.cell);
            self.whistle_frighten_otter(actor.cell);
            self.whistle_frighten_beaver(actor.cell);
            let name = self.name_of(actor.id);
            let rest = "the bone whistle.";
            self.emit(
                crate::model::spoken_act(Some(name.as_str()), "blow", rest),
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
                crate::model::spoken_act(None, "blow", rest),
                results,
            );
        }
    }

    /// Knife before flake. Either may be in the arms or lying on this cell. It is not spent.
    pub(crate) fn flute_tool_rest(&self, actor: crate::model::ActorId) -> Option<&'static str> {
        let cell = self.actor(actor).map(|person| person.cell).unwrap_or(0);
        let knife = !self.loose_ids(actor, Material::Knife).is_empty()
            || !self.ground_ids(cell, Material::Knife).is_empty();
        let flake = !self.loose_ids(actor, Material::Flake).is_empty()
            || !self.ground_ids(cell, Material::Flake).is_empty();
        if knife {
            Some("a bundle of reeds into a reed flute with the iron knife.")
        } else if flake {
            Some("a bundle of reeds into a reed flute with the flint flake.")
        } else {
            None
        }
    }

    fn apply_flute_carves(&mut self, carves: Vec<Intent>, results: &mut Vec<CommandResult>) {
        for intent in carves {
            let Some(actor) = self.actor(intent.actor_id).cloned() else {
                continue;
            };
            let Some(rest) = self.flute_tool_rest(actor.id) else {
                self.finish(
                    intent.command_id,
                    actor.id,
                    false,
                    "You need a flint flake or an iron knife to carve a flute.",
                    results,
                );
                continue;
            };
            if !self.take_one_reed(actor.id, actor.cell) {
                self.finish(
                    intent.command_id,
                    actor.id,
                    false,
                    "You have no reeds to carve a flute.",
                    results,
                );
                continue;
            }
            let id = self.mint_item(Material::Flute, 0, actor.cell, Some(actor.id), Role::Loose);
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

    fn apply_flute_blows(&mut self, blows: Vec<Intent>, results: &mut Vec<CommandResult>) {
        for intent in blows {
            let Some(actor) = self.actor(intent.actor_id).cloned() else {
                continue;
            };
            if self.loose_ids(actor.id, Material::Flute).is_empty() {
                self.finish(
                    intent.command_id,
                    actor.id,
                    false,
                    "You are not carrying a reed flute.",
                    results,
                );
                continue;
            }
            self.flute = Some(crate::model::WhistleSound {
                cell: actor.cell,
                until: self.time.saturating_add(1),
            });
            let name = self.name_of(actor.id);
            let rest = "the reed flute.";
            self.emit(
                crate::model::spoken_act(Some(name.as_str()), "blow", rest),
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
                crate::model::spoken_act(None, "blow", rest),
                results,
            );
        }
    }

    /// A hide that is not already a cloak. Arms, then one lying in this cell.
    fn drum_hide_id(&self, actor: crate::model::ActorId) -> Option<crate::model::ItemId> {
        let open =
            |id: crate::model::ItemId| self.item(id).is_some_and(|hide| !hide.worn && !hide.sewn);
        if let Some(id) = self
            .loose_ids(actor, Material::Hide)
            .into_iter()
            .find(|id| open(*id))
        {
            return Some(id);
        }
        self.ground_ids(self.bow_cell(actor), Material::Hide)
            .into_iter()
            .find(|id| open(*id))
    }

    fn drum_hide_word(&self, id: crate::model::ItemId) -> &'static str {
        let Some(hide) = self.item(id) else {
            return "deer hide";
        };
        if hide.hare {
            "hare hide"
        } else if hide.fox {
            "fox hide"
        } else if hide.otter {
            "otter hide"
        } else if hide.beaver {
            "beaver hide"
        } else if hide.wolf {
            "wolf hide"
        } else {
            "deer hide"
        }
    }

    /// Hide, then a dry branch for the hoop, then cordage. The first gap is the refusal.
    pub(crate) fn drum_refusal(&self, actor: crate::model::ActorId) -> Option<&'static str> {
        if self.drum_hide_id(actor).is_none() {
            return Some("You have no hide to sew over a frame.");
        }
        let cell = self.bow_cell(actor);
        let dry = self.bow_branch_id(actor).is_some();
        let any_branch = !self.loose_ids(actor, Material::Branch).is_empty()
            || !self.ground_ids(cell, Material::Branch).is_empty();
        if !dry && any_branch {
            return Some("The branch is not dry.");
        }
        if !dry {
            return Some("You have no dry branch for a hoop.");
        }
        if self.drum_cord(actor).is_none() {
            Some("You have no cordage to sew a drum.")
        } else {
            None
        }
    }

    /// Hide strips, then willow, then nettle, then sinew.
    fn drum_cord(
        &self,
        actor: crate::model::ActorId,
    ) -> Option<(crate::model::ItemId, &'static str)> {
        if let Some((id, kind)) = self.bow_line(actor) {
            let word = match kind {
                Line::Hide => "hide strips",
                Line::Willow => "willow cordage",
                Line::Nettle => "nettle cordage",
                Line::Sinew => "sinew",
            };
            return Some((id, word));
        }
        let cell = self.bow_cell(actor);
        if let Some(id) = self.loose_ids(actor, Material::Sinew).first().copied() {
            return Some((id, "sinew"));
        }
        self.ground_ids(cell, Material::Sinew)
            .first()
            .copied()
            .map(|id| (id, "sinew"))
    }

    fn apply_drum_stretches(&mut self, stretches: Vec<Intent>, results: &mut Vec<CommandResult>) {
        for intent in stretches {
            let Some(actor) = self.actor(intent.actor_id).cloned() else {
                continue;
            };
            if let Some(message) = self.drum_refusal(actor.id) {
                self.finish(intent.command_id, actor.id, false, message, results);
                continue;
            }
            let Some(hide) = self.drum_hide_id(actor.id) else {
                continue;
            };
            let Some(branch) = self.bow_branch_id(actor.id) else {
                continue;
            };
            let Some((line, cord)) = self.drum_cord(actor.id) else {
                continue;
            };
            let hide_word = self.drum_hide_word(hide);
            self.items.remove(&hide);
            self.items.remove(&branch);
            self.items.remove(&line);
            let id = self.mint_item(Material::Drum, 0, actor.cell, Some(actor.id), Role::Loose);
            let name = self.name_of(actor.id);
            let rest = format!("a {hide_word} over a hoop of dry branch with {cord}.");
            self.emit(
                crate::model::spoken_act(Some(name.as_str()), "sew", &rest),
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
                crate::model::spoken_act(None, "sew", &rest),
                results,
            );
        }
    }

    /// A flint flake or an iron knife. The tool stays. The rest names it.
    fn sticks_tool_rest(&self, actor: ActorId) -> Option<&'static str> {
        let cell = self.bow_cell(actor);
        let knife = !self.loose_ids(actor, Material::Knife).is_empty()
            || !self.ground_ids(cell, Material::Knife).is_empty();
        let flake = !self.loose_ids(actor, Material::Flake).is_empty()
            || !self.ground_ids(cell, Material::Flake).is_empty();
        if knife {
            Some("two dry branches into rhythm sticks with the iron knife.")
        } else if flake {
            Some("two dry branches into rhythm sticks with the flint flake.")
        } else {
            None
        }
    }

    /// Two dry branches, from the arms and then the ground.
    fn rhythm_branch_ids(&self, actor: ActorId) -> Vec<ItemId> {
        let mut ids = self.dry_loose_ids(actor, Material::Branch);
        if let Some(person) = self.actor(actor) {
            for id in self.ground_ids(person.cell, Material::Branch) {
                if ids.len() >= 2 {
                    break;
                }
                if self.item(id).is_some_and(|item| self.is_dry(item.moisture))
                    && !ids.contains(&id)
                {
                    ids.push(id);
                }
            }
        }
        ids.truncate(2);
        ids
    }

    /// A damp branch is named before a short count. None at all asks for two.
    fn sticks_branch_miss(&self, actor: ActorId) -> Option<&'static str> {
        if self.rhythm_branch_ids(actor).len() >= 2 {
            return None;
        }
        let damp = self
            .loose_ids(actor, Material::Branch)
            .into_iter()
            .any(|id| {
                self.item(id)
                    .is_some_and(|item| !self.is_dry(item.moisture))
            })
            || self.actor(actor).is_some_and(|person| {
                self.ground_ids(person.cell, Material::Branch)
                    .into_iter()
                    .any(|id| {
                        self.item(id)
                            .is_some_and(|item| !self.is_dry(item.moisture))
                    })
            });
        if damp {
            Some("The branch is not dry.")
        } else {
            Some("You need two dry branches to carve rhythm sticks.")
        }
    }

    /// Both branches on the ground need a free hand. A carried branch frees one.
    fn sticks_arms_full(&self, actor: ActorId) -> bool {
        let ids = self.rhythm_branch_ids(actor);
        if ids.len() < 2 {
            return false;
        }
        let carried = ids.iter().any(|id| {
            self.item(*id)
                .is_some_and(|item| item.carrier == Some(actor))
        });
        !carried && self.carry_count(actor) >= self.tuning.carry_slots
    }

    fn torch_held_or_here(
        &self,
        actor: crate::model::ActorId,
        material: Material,
    ) -> Option<crate::model::ItemId> {
        if let Some(id) = self.loose_ids(actor, material).first().copied() {
            return Some(id);
        }
        let cell = self.actor(actor).map(|person| person.cell)?;
        self.ground_ids(cell, material).first().copied()
    }

    fn torch_dry_held_or_here(
        &self,
        actor: crate::model::ActorId,
        material: Material,
    ) -> Option<crate::model::ItemId> {
        if let Some(id) = self.dry_loose_ids(actor, material).first().copied() {
            return Some(id);
        }
        let cell = self.actor(actor)?.cell;
        self.ground_ids(cell, material).into_iter().find(|id| {
            self.item(*id)
                .is_some_and(|item| self.is_dry(item.moisture))
        })
    }

    /// A carried or dropped piece, or the pile on this cell.
    fn torch_present(&self, actor: crate::model::ActorId, material: Material) -> bool {
        self.torch_held_or_here(actor, material).is_some()
            || self.has_material_at_hand(actor, material)
    }

    /// A dry carried or dropped piece, or a dry pile on this cell.
    fn torch_dry_present(&self, actor: crate::model::ActorId, material: Material) -> bool {
        self.torch_dry_held_or_here(actor, material).is_some() || self.has_dry(actor, material)
    }

    /// No resin, no dry wood, or both pieces off the arms with the arms full.
    fn torch_make_miss(&self, actor: crate::model::ActorId) -> Option<&'static str> {
        if !self.torch_present(actor, Material::Resin) {
            return Some("You have no pine resin.");
        }
        let branch = self.torch_dry_present(actor, Material::Branch);
        let bark = self.torch_dry_present(actor, Material::Bark);
        if !branch && !bark {
            let damp_branch = self.torch_present(actor, Material::Branch);
            let damp_bark = self.torch_present(actor, Material::Bark);
            return Some(if damp_branch {
                "The branch is too damp to bind."
            } else if damp_bark {
                "The bark is too damp to wrap."
            } else {
                "You need a dry branch or a sheet of bark."
            });
        }
        let carried = !self.loose_ids(actor, Material::Resin).is_empty()
            || !self.dry_loose_ids(actor, Material::Branch).is_empty()
            || !self.dry_loose_ids(actor, Material::Bark).is_empty();
        if !carried && self.carry_count(actor) >= self.tuning.carry_slots {
            return Some(ARMS_FULL);
        }
        None
    }

    fn torch_light_miss(&self, actor: crate::model::ActorId) -> Option<&'static str> {
        let torches = self.loose_ids(actor, Material::Torch);
        if torches.is_empty() {
            return Some("You are not carrying a pitch torch.");
        }
        let unlit = torches.iter().any(|id| {
            self.item(*id)
                .is_some_and(|torch| torch.firing_ready_at.is_none())
        });
        if !unlit {
            return Some("The pitch torch is already lit.");
        }
        let cell = self.actor(actor).map(|person| person.cell).unwrap_or(0);
        if self.fire_at(cell).is_some() || self.torch_tinder_ready(actor) {
            return None;
        }
        let damp =
            self.torch_present(actor, Material::Bark) || self.torch_present(actor, Material::Moss);
        Some(if damp {
            "The tinder is too damp."
        } else {
            "You need a fire here, or tinder in the arms."
        })
    }

    /// Dry bark, then dry moss, then cattail fluff. A pile at the feet counts.
    /// Fluff still catches when damp.
    fn torch_tinder_ready(&self, actor: crate::model::ActorId) -> bool {
        self.torch_dry_present(actor, Material::Bark)
            || self.torch_dry_present(actor, Material::Moss)
            || self.torch_present(actor, Material::Fluff)
    }

    fn spend_torch_resin(&mut self, actor: crate::model::ActorId) -> bool {
        if let Some(id) = self.torch_held_or_here(actor, Material::Resin) {
            self.items.remove(&id);
            return true;
        }
        self.take_hand(actor, Material::Resin).is_ok()
    }

    /// A dry piece from the arms or the ground, then one from the dry pile.
    fn spend_torch_dry(&mut self, actor: crate::model::ActorId, material: Material) -> bool {
        if let Some(id) = self.torch_dry_held_or_here(actor, material) {
            self.items.remove(&id);
            return true;
        }
        self.consume_dry(actor, material).is_ok()
    }

    /// Dry bark, then dry moss, then cattail fluff. The words name which, and
    /// whether it came from the arms, the ground, or the pile at the feet.
    fn spend_torch_tinder(&mut self, actor: crate::model::ActorId) -> Option<&'static str> {
        if let Some(rest) = self.spend_named_dry(
            actor,
            Material::Bark,
            "the pitch torch with the bark.",
            "the pitch torch with a dry sheet of bark from the ground.",
            "the pitch torch with bark from the pile at the feet.",
        ) {
            return Some(rest);
        }
        if let Some(rest) = self.spend_named_dry(
            actor,
            Material::Moss,
            "the pitch torch with the moss.",
            "the pitch torch with dry moss from the ground.",
            "the pitch torch with moss from the pile at the feet.",
        ) {
            return Some(rest);
        }
        self.spend_named_fluff(actor)
    }

    fn spend_named_dry(
        &mut self,
        actor: crate::model::ActorId,
        material: Material,
        arms: &'static str,
        ground: &'static str,
        pile: &'static str,
    ) -> Option<&'static str> {
        if let Some(id) = self.dry_loose_ids(actor, material).first().copied() {
            self.items.remove(&id);
            return Some(arms);
        }
        let cell = self.actor(actor)?.cell;
        if let Some(id) = self.ground_ids(cell, material).into_iter().find(|id| {
            self.item(*id)
                .is_some_and(|item| self.is_dry(item.moisture))
        }) {
            self.items.remove(&id);
            return Some(ground);
        }
        if self.dry_pile_count(cell, material) > 0
            && self.take_from_pile(cell, material, 1).is_some()
        {
            return Some(pile);
        }
        None
    }

    /// Fluff still catches when it is damp.
    fn spend_named_fluff(&mut self, actor: crate::model::ActorId) -> Option<&'static str> {
        if let Some(id) = self.loose_ids(actor, Material::Fluff).first().copied() {
            self.items.remove(&id);
            return Some("the pitch torch with the cattail fluff.");
        }
        let cell = self.actor(actor)?.cell;
        if let Some(id) = self.ground_ids(cell, Material::Fluff).first().copied() {
            self.items.remove(&id);
            return Some("the pitch torch with cattail fluff from the ground.");
        }
        if self.take_from_pile(cell, Material::Fluff, 1).is_some() {
            return Some("the pitch torch with cattail fluff from the pile at the feet.");
        }
        None
    }

    fn apply_torch_makes(&mut self, makes: Vec<Intent>, results: &mut Vec<CommandResult>) {
        for intent in makes {
            let Some(actor) = self.actor(intent.actor_id).cloned() else {
                continue;
            };
            if let Some(miss) = self.torch_make_miss(actor.id) {
                self.finish(intent.command_id, actor.id, false, miss, results);
                continue;
            }
            if !self.spend_torch_resin(actor.id) {
                continue;
            }
            let wrapped = if self.torch_dry_present(actor.id, Material::Branch) {
                if !self.spend_torch_dry(actor.id, Material::Branch) {
                    continue;
                }
                false
            } else if self.spend_torch_dry(actor.id, Material::Bark) {
                true
            } else {
                continue;
            };
            let id = self.mint_item(Material::Torch, 0, actor.cell, Some(actor.id), Role::Loose);
            if let Some(torch) = self.items.get_mut(&id) {
                torch.bound = wrapped;
            }
            let rest = if wrapped {
                "a lump of pine resin and a sheet of bark into a pitch torch."
            } else {
                "a lump of pine resin and a dry branch into a pitch torch."
            };
            let name = self.name_of(actor.id);
            self.emit(
                crate::model::spoken_act(Some(name.as_str()), "bind", rest),
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
                crate::model::spoken_act(None, "bind", rest),
                results,
            );
        }
    }

    fn apply_torch_lights(&mut self, lights: Vec<Intent>, results: &mut Vec<CommandResult>) {
        for intent in lights {
            let Some(actor) = self.actor(intent.actor_id).cloned() else {
                continue;
            };
            if let Some(miss) = self.torch_light_miss(actor.id) {
                self.finish(intent.command_id, actor.id, false, miss, results);
                continue;
            }
            let Some(id) = self
                .loose_ids(actor.id, Material::Torch)
                .into_iter()
                .find(|id| {
                    self.item(*id)
                        .is_some_and(|torch| torch.firing_ready_at.is_none())
                })
            else {
                continue;
            };
            let from_fire = self.fire_at(actor.cell).is_some();
            let spent = if from_fire {
                None
            } else {
                self.spend_torch_tinder(actor.id)
            };
            if !from_fire && spent.is_none() {
                self.finish(
                    intent.command_id,
                    actor.id,
                    false,
                    "You need a fire here, or tinder in the arms.",
                    results,
                );
                continue;
            }
            let wrapped = self.item(id).is_some_and(|torch| torch.bound);
            let wait = if wrapped {
                self.tuning.clay_ticks.saturating_mul(2)
            } else {
                self.tuning.clay_ticks
            };
            let ready = self.time.saturating_add(wait);
            if let Some(torch) = self.items.get_mut(&id) {
                torch.firing_ready_at = Some(ready);
            }
            self.schedule(ready, Wake::TorchOut(id));
            let Some(rest) = spent.or(from_fire.then_some("the pitch torch from the fire.")) else {
                continue;
            };
            let name = self.name_of(actor.id);
            self.emit(
                crate::model::spoken_act(Some(name.as_str()), "light", rest),
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
                crate::model::spoken_act(None, "light", rest),
                results,
            );
        }
    }

    fn apply_stick_carves(&mut self, carves: Vec<Intent>, results: &mut Vec<CommandResult>) {
        for intent in carves {
            let Some(actor) = self.actor(intent.actor_id).cloned() else {
                continue;
            };
            let Some(rest) = self.sticks_tool_rest(actor.id) else {
                self.finish(
                    intent.command_id,
                    actor.id,
                    false,
                    "You need a flint flake or an iron knife to carve rhythm sticks.",
                    results,
                );
                continue;
            };
            if let Some(miss) = self.sticks_branch_miss(actor.id) {
                self.finish(intent.command_id, actor.id, false, miss, results);
                continue;
            }
            if self.sticks_arms_full(actor.id) {
                self.finish(intent.command_id, actor.id, false, ARMS_FULL, results);
                continue;
            }
            let branches = self.rhythm_branch_ids(actor.id);
            for id in &branches {
                self.items.remove(id);
            }
            let id = self.mint_item(Material::Sticks, 0, actor.cell, Some(actor.id), Role::Loose);
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

    fn apply_stick_beats(&mut self, beats: Vec<Intent>, results: &mut Vec<CommandResult>) {
        for intent in beats {
            let Some(actor) = self.actor(intent.actor_id).cloned() else {
                continue;
            };
            if self.loose_ids(actor.id, Material::Sticks).is_empty() {
                self.finish(
                    intent.command_id,
                    actor.id,
                    false,
                    "You are not carrying rhythm sticks.",
                    results,
                );
                continue;
            }
            self.sticks = Some(crate::model::WhistleSound {
                cell: actor.cell,
                until: self.time.saturating_add(1),
            });
            let name = self.name_of(actor.id);
            let rest = "the rhythm sticks.";
            self.emit(
                crate::model::spoken_act(Some(name.as_str()), "click", rest),
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
                crate::model::spoken_act(None, "click", rest),
                results,
            );
        }
    }

    fn apply_drum_beats(&mut self, beats: Vec<Intent>, results: &mut Vec<CommandResult>) {
        for intent in beats {
            let Some(actor) = self.actor(intent.actor_id).cloned() else {
                continue;
            };
            if self.loose_ids(actor.id, Material::Drum).is_empty() {
                self.finish(
                    intent.command_id,
                    actor.id,
                    false,
                    "You are not carrying a drum.",
                    results,
                );
                continue;
            }
            self.drum = Some(crate::model::WhistleSound {
                cell: actor.cell,
                until: self.time.saturating_add(1),
            });
            self.drum_startle_deer(actor.cell);
            let name = self.name_of(actor.id);
            let rest = "the drum.";
            self.emit(
                crate::model::spoken_act(Some(name.as_str()), "beat", rest),
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
                crate::model::spoken_act(None, "beat", rest),
                results,
            );
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

    fn take_one_antler(
        &mut self,
        actor: crate::model::ActorId,
        cell: crate::model::CellId,
    ) -> bool {
        if let Some(id) = self.loose_ids(actor, Material::Antler).first().copied() {
            self.items.remove(&id);
            return true;
        }
        if let Some(id) = self.ground_ids(cell, Material::Antler).first().copied() {
            self.items.remove(&id);
            return true;
        }
        false
    }

    fn take_one_bone(&mut self, actor: crate::model::ActorId, cell: crate::model::CellId) -> bool {
        if let Some(id) = self.loose_ids(actor, Material::Bone).first().copied() {
            self.items.remove(&id);
            return true;
        }
        if let Some(id) = self.ground_ids(cell, Material::Bone).first().copied() {
            self.items.remove(&id);
            self.arm_bone_cache(cell);
            return true;
        }
        false
    }

    fn apply_pulls(&mut self, pulls: Vec<Intent>, results: &mut Vec<CommandResult>) {
        for intent in pulls {
            let Some(actor) = self.actor(intent.actor_id).cloned() else {
                continue;
            };
            let Some(id) = self.sinew_ready(actor.id) else {
                let message = match self.sinew_source(actor.id) {
                    SinewSource::StrippedCarcass => "The carcass is already stripped.",
                    SinewSource::StrippedMeat => "The meat is already stripped.",
                    _ => "You have no carcass or raw meat to pull sinew from.",
                };
                self.finish(intent.command_id, actor.id, false, message, results);
                continue;
            };
            if let Some(item) = self.items.get_mut(&id) {
                item.stripped = true;
            }
            let from_meat = self
                .item(id)
                .is_some_and(|item| item.material == Material::Meat);
            let rest = if from_meat {
                "sinew from the raw meat."
            } else {
                "sinew from the carcass."
            };
            let sinew = self.mint_item(Material::Sinew, 0, actor.cell, Some(actor.id), Role::Loose);
            let name = self.name_of(actor.id);
            self.emit(
                crate::model::spoken_act(Some(name.as_str()), "pull", rest),
                Scope::Cell(actor.cell),
                vec![actor.id],
                vec![actor.cell],
                vec![sinew],
                vec![],
                None,
            );
            self.finish(
                intent.command_id,
                actor.id,
                true,
                crate::model::spoken_act(None, "pull", rest),
                results,
            );
        }
    }

    /// An unstripped carcass, then unstripped raw meat. Arms before the ground.
    fn sinew_ready(&self, actor: crate::model::ActorId) -> Option<crate::model::ItemId> {
        match self.sinew_source(actor) {
            SinewSource::Ready(id) => Some(id),
            _ => None,
        }
    }

    fn sinew_source(&self, actor: crate::model::ActorId) -> SinewSource {
        let cell = self.actor(actor).map(|person| person.cell).unwrap_or(0);
        let fresh = |id: crate::model::ItemId| self.item(id).is_some_and(|item| !item.stripped);
        if let Some(id) = self
            .loose_ids(actor, Material::Carcass)
            .into_iter()
            .find(|id| fresh(*id))
        {
            return SinewSource::Ready(id);
        }
        if let Some(id) = self
            .ground_ids(cell, Material::Carcass)
            .into_iter()
            .find(|id| fresh(*id))
        {
            return SinewSource::Ready(id);
        }
        let raw = |id: crate::model::ItemId| {
            self.item(id)
                .is_some_and(|meat| !meat.stripped && !meat.cooked && !meat.smoked && !meat.spoiled)
        };
        if let Some(id) = self
            .loose_ids(actor, Material::Meat)
            .into_iter()
            .find(|id| raw(*id))
        {
            return SinewSource::Ready(id);
        }
        if let Some(id) = self
            .ground_ids(cell, Material::Meat)
            .into_iter()
            .find(|id| raw(*id))
        {
            return SinewSource::Ready(id);
        }
        let stripped_carcass = self
            .loose_ids(actor, Material::Carcass)
            .into_iter()
            .chain(self.ground_ids(cell, Material::Carcass))
            .any(|id| self.item(id).is_some_and(|item| item.stripped));
        if stripped_carcass {
            return SinewSource::StrippedCarcass;
        }
        let stripped_meat = self
            .loose_ids(actor, Material::Meat)
            .into_iter()
            .chain(self.ground_ids(cell, Material::Meat))
            .any(|id| {
                self.item(id).is_some_and(|meat| {
                    meat.stripped && !meat.cooked && !meat.smoked && !meat.spoiled
                })
            });
        if stripped_meat {
            SinewSource::StrippedMeat
        } else {
            SinewSource::None
        }
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
            let Some(line) = self.take_snare_line(actor.id, actor.cell) else {
                self.finish(
                    intent.command_id,
                    actor.id,
                    false,
                    "You have no hide strips or willow cordage to set a snare.",
                    results,
                );
                continue;
            };
            let ready = self.time.saturating_add(self.tuning.snare_ticks);
            self.snares.insert(actor.cell, ready);
            match line {
                Line::Willow => {
                    self.snare_willow.insert(actor.cell);
                }
                Line::Nettle => {
                    self.snare_nettle.insert(actor.cell);
                }
                Line::Sinew => {
                    self.snare_sinew.insert(actor.cell);
                }
                Line::Hide => {}
            }
            self.schedule(ready, crate::model::Wake::Snare(actor.cell));
            let name = self.name_of(actor.id);
            let rest = match line {
                Line::Hide => "a snare of hide strips.",
                Line::Willow => "a snare of willow cordage.",
                Line::Nettle => "a snare of nettle cordage.",
                Line::Sinew => "a snare of sinew.",
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

    /// One bundle of reeds and one length of cordage, spent into this brook.
    /// After the wait a raw fish lies there and the weir is gone.
    fn apply_weirs(&mut self, weirs: Vec<Intent>, results: &mut Vec<CommandResult>) {
        for intent in weirs {
            let Some(actor) = self.actor(intent.actor_id).cloned() else {
                continue;
            };
            if self.cell(actor.cell).region != Region::Brook {
                self.finish(
                    intent.command_id,
                    actor.id,
                    false,
                    "You cannot set a weir out of the brook.",
                    results,
                );
                continue;
            }
            if self.weirs.contains_key(&actor.cell) {
                self.finish(
                    intent.command_id,
                    actor.id,
                    false,
                    "A reed weir is already set here.",
                    results,
                );
                continue;
            }
            if !self.has_reeds(actor.id) {
                self.finish(
                    intent.command_id,
                    actor.id,
                    false,
                    "You have no reeds to set a weir.",
                    results,
                );
                continue;
            }
            if !self.has_snare_line(actor.id) {
                self.finish(
                    intent.command_id,
                    actor.id,
                    false,
                    "You have no cordage to bind a weir.",
                    results,
                );
                continue;
            }
            if !self.take_one_reed(actor.id, actor.cell) {
                self.finish(
                    intent.command_id,
                    actor.id,
                    false,
                    "You have no reeds to set a weir.",
                    results,
                );
                continue;
            }
            if self.take_snare_line(actor.id, actor.cell).is_none() {
                self.finish(
                    intent.command_id,
                    actor.id,
                    false,
                    "You have no cordage to bind a weir.",
                    results,
                );
                continue;
            }
            let ready = self.time.saturating_add(self.tuning.weir_ticks);
            self.weirs.insert(actor.cell, ready);
            self.schedule(ready, crate::model::Wake::Weir(actor.cell));
            let name = self.name_of(actor.id);
            let rest = "a reed weir in the brook.";
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

    /// Hide strips if any are here, otherwise willow, otherwise nettle, otherwise sinew.
    pub(crate) fn has_snare_line(&self, actor: crate::model::ActorId) -> bool {
        self.has_cord(actor)
            || self.has_willow_cord(actor)
            || self.has_nettle_cord(actor)
            || self.has_sinew(actor)
    }

    fn has_sinew(&self, actor: crate::model::ActorId) -> bool {
        let cell = self.actor(actor).map(|person| person.cell).unwrap_or(0);
        !self.loose_ids(actor, Material::Sinew).is_empty()
            || !self.ground_ids(cell, Material::Sinew).is_empty()
    }

    fn has_willow_cord(&self, actor: crate::model::ActorId) -> bool {
        let cell = self.actor(actor).map(|person| person.cell).unwrap_or(0);
        !self.loose_ids(actor, Material::Willow).is_empty()
            || !self.ground_ids(cell, Material::Willow).is_empty()
    }

    fn has_nettle_cord(&self, actor: crate::model::ActorId) -> bool {
        let cell = self.actor(actor).map(|person| person.cell).unwrap_or(0);
        !self.loose_ids(actor, Material::Nettle).is_empty()
            || !self.ground_ids(cell, Material::Nettle).is_empty()
    }

    /// Hide strips, then willow cordage, then nettle cordage.
    /// Hide strips win when more than one is held, in the arms or on the ground.
    fn take_snare_line(
        &mut self,
        actor: crate::model::ActorId,
        cell: crate::model::CellId,
    ) -> Option<Line> {
        if self.take_one_cord(actor, cell) {
            return Some(Line::Hide);
        }
        if let Some(id) = self.loose_ids(actor, Material::Willow).first().copied() {
            self.items.remove(&id);
            return Some(Line::Willow);
        }
        if let Some(id) = self.ground_ids(cell, Material::Willow).first().copied() {
            self.items.remove(&id);
            return Some(Line::Willow);
        }
        if let Some(id) = self.loose_ids(actor, Material::Nettle).first().copied() {
            self.items.remove(&id);
            return Some(Line::Nettle);
        }
        if let Some(id) = self.ground_ids(cell, Material::Nettle).first().copied() {
            self.items.remove(&id);
            return Some(Line::Nettle);
        }
        if let Some(id) = self.loose_ids(actor, Material::Sinew).first().copied() {
            self.items.remove(&id);
            return Some(Line::Sinew);
        }
        if let Some(id) = self.ground_ids(cell, Material::Sinew).first().copied() {
            self.items.remove(&id);
            return Some(Line::Sinew);
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

    pub(crate) fn has_reeds(&self, actor: crate::model::ActorId) -> bool {
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

    /// Two more things fit in a carried bark basket. They do not take an arm slot.
    pub(crate) const BASKET_ROOM: u32 = 2;

    pub(crate) fn basket_load(&self, actor: crate::model::ActorId) -> u32 {
        self.items
            .values()
            .filter(|item| item.carrier == Some(actor) && item.in_basket)
            .count() as u32
    }

    pub(crate) fn basket_has_room(&self, actor: crate::model::ActorId) -> bool {
        !self.loose_ids(actor, Material::Basket).is_empty()
            && self.basket_load(actor) < Self::BASKET_ROOM
    }

    /// Bare `basket`. The reply stays with this person.
    pub fn basket_name_line(&self, actor: crate::model::ActorId) -> String {
        if self.loose_ids(actor, Material::Basket).is_empty() {
            "Weave a bark basket from a sheet of bark. It holds two more things.".into()
        } else {
            format!(
                "You carry a bark basket. {}",
                self.basket_held_sentence(actor)
            )
        }
    }

    pub(crate) fn basket_can_hold_berries(&self, actor: crate::model::ActorId) -> bool {
        self.basket_has_room(actor)
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
        material != Material::Basket && self.basket_has_room(actor)
    }

    /// Things still marked in a bark basket on this ground come up with it.
    fn lift_basket_contents(
        &mut self,
        actor: crate::model::ActorId,
        cell: crate::model::CellId,
        basket: crate::model::ItemId,
    ) {
        let held: Vec<_> = self
            .items
            .values()
            .filter(|item| {
                item.id != basket
                    && item.cell == cell
                    && item.carrier.is_none()
                    && item.in_basket
                    && item.role == Role::Loose
            })
            .map(|item| item.id)
            .collect();
        for id in held {
            if let Some(item) = self.items.get_mut(&id) {
                item.carrier = Some(actor);
            }
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
        if let Some(item) = self.items.get_mut(&item_id) {
            item.in_basket = true;
        }
    }

    /// A gather that would not fit in the arms names the basket it went into.
    fn basket_gather_rest(
        &self,
        actor: crate::model::ActorId,
        item_id: crate::model::ItemId,
        rest: String,
    ) -> String {
        if !self.item(item_id).is_some_and(|item| item.in_basket) {
            return rest;
        }
        if self.basket_load(actor) >= Self::BASKET_ROOM {
            format!("{rest} It goes into the bark basket. The basket is full.")
        } else {
            format!("{rest} It goes into the bark basket.")
        }
    }

    /// A carried loose thing, or one on this ground, that can go into the basket.
    fn basket_put_id(
        &self,
        actor: crate::model::ActorId,
        cell: crate::model::CellId,
        material: Material,
    ) -> Option<crate::model::ItemId> {
        if material == Material::Basket || material == Material::Water || material == Material::Mud
        {
            return None;
        }
        let carried = self.loose_ids(actor, material).into_iter().find(|id| {
            self.item(*id)
                .is_some_and(|item| !item.in_basket && !item.worn_cloak())
        });
        if carried.is_some() {
            return carried;
        }
        self.ground_ids(cell, material).into_iter().next()
    }

    fn basket_put_ready(
        &self,
        actor: crate::model::ActorId,
        cell: crate::model::CellId,
        material: Material,
    ) -> bool {
        self.basket_put_id(actor, cell, material).is_some()
            || (material != Material::Basket
                && material != Material::Water
                && material != Material::Mud
                && self.pile_count(cell, material) > 0)
    }

    fn apply_basket_puts(&mut self, puts: Vec<Intent>, results: &mut Vec<CommandResult>) {
        for intent in puts {
            let Attempt::PutInBasket(material) = intent.attempt else {
                continue;
            };
            let Some(actor) = self.actor(intent.actor_id).cloned() else {
                continue;
            };
            if self.loose_ids(actor.id, Material::Basket).is_empty() {
                self.finish(
                    intent.command_id,
                    actor.id,
                    false,
                    "You are not carrying a bark basket.",
                    results,
                );
                continue;
            }
            if material == Material::Basket {
                self.finish(
                    intent.command_id,
                    actor.id,
                    false,
                    "A bark basket does not go into a bark basket.",
                    results,
                );
                continue;
            }
            if !self.basket_has_room(actor.id) {
                self.finish(
                    intent.command_id,
                    actor.id,
                    false,
                    "The bark basket is full.",
                    results,
                );
                continue;
            }
            let id = if let Some(id) = self.basket_put_id(actor.id, actor.cell, material) {
                if let Some(item) = self.items.get_mut(&id) {
                    item.carrier = Some(actor.id);
                    item.cell = actor.cell;
                    item.in_basket = true;
                    item.worn = false;
                }
                id
            } else if let Some(moisture) = self.take_from_pile(actor.cell, material, 1) {
                let id =
                    self.mint_item(material, moisture, actor.cell, Some(actor.id), Role::Loose);
                if let Some(item) = self.items.get_mut(&id) {
                    item.in_basket = true;
                }
                id
            } else {
                self.finish(
                    intent.command_id,
                    actor.id,
                    false,
                    format!(
                        "There is no {} here to put in the basket.",
                        material.gather_noun()
                    ),
                    results,
                );
                continue;
            };
            let mut rest = format!(
                "{} {} in the bark basket.",
                material.article(),
                material.gather_noun()
            );
            if self.basket_load(actor.id) >= Self::BASKET_ROOM {
                rest.push_str(" The basket is full.");
            }
            let name = self.name_of(actor.id);
            self.emit(
                crate::model::spoken_act(Some(name.as_str()), "put", &rest),
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
                crate::model::spoken_act(None, "put", &rest),
                results,
            );
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

    /// A stone in the hands, on a stone or on the rock. The roots become flour. The stones stay.
    fn apply_grinds(&mut self, grinds: Vec<Intent>, results: &mut Vec<CommandResult>) {
        for intent in grinds {
            let Some(actor) = self.actor(intent.actor_id).cloned() else {
                continue;
            };
            let Ok(root) = self.grind_root(actor.id) else {
                self.finish(
                    intent.command_id,
                    intent.actor_id,
                    false,
                    "You have no cattail roots to grind.",
                    results,
                );
                continue;
            };
            if self.loose_ids(actor.id, Material::Stone).is_empty() || !self.stone_under(actor.id) {
                self.finish(
                    intent.command_id,
                    intent.actor_id,
                    false,
                    "You need a stone on stone to grind.",
                    results,
                );
                continue;
            }
            self.items.remove(&root);
            let flour = self.mint_item(Material::Flour, 0, actor.cell, Some(actor.id), Role::Loose);
            let rest = "the cattail roots into flour on the stone.";
            let name = self.name_of(actor.id);
            self.emit(
                crate::model::spoken_act(Some(name.as_str()), "grind", rest),
                Scope::Cell(actor.cell),
                vec![actor.id],
                vec![actor.cell],
                vec![flour],
                vec![],
                None,
            );
            self.finish(
                intent.command_id,
                actor.id,
                true,
                crate::model::spoken_act(None, "grind", rest),
                results,
            );
        }
    }

    /// One sheet of bark, from the arms first and then the pile, becomes a bark cup.
    /// The cup stays in the arms. A clay pot is left where it is.
    fn apply_cups(&mut self, folds: Vec<Intent>, results: &mut Vec<CommandResult>) {
        for intent in folds {
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
                    "You need a sheet of bark to fold a cup.",
                    results,
                );
                continue;
            };
            let id = self.mint_item(
                Material::BarkCup,
                0,
                actor.cell,
                Some(actor.id),
                Role::Loose,
            );
            let name = self.name_of(actor.id);
            let rest = "a sheet of bark into a bark cup.";
            let mut affected = bark;
            affected.push(id);
            self.emit(
                crate::model::spoken_act(Some(name.as_str()), "fold", rest),
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
                crate::model::spoken_act(None, "fold", rest),
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
                .is_some_and(|hide| !hide.pierced && !hide.sewn && !hide.worn && !hide.beaver)
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
                    self.schedule_spoil_wakes(piece);
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
        if self.fox_is_nearest(actor.id) {
            if let Some(fox) = self.fox.clone() {
                if crate::model::chebyshev(actor.cell, fox.cell) <= self.tuning.sight {
                    self.loose_the_bow_at_the_fox(intent, actor, fox.cell, results);
                    return;
                }
            }
            self.finish(
                intent.command_id.clone(),
                actor.id,
                false,
                "The fox is not close enough for the bow.",
                results,
            );
            return;
        }
        if self.otter_is_nearest(actor.id) {
            if let Some(otter) = self.otter.clone() {
                if crate::model::chebyshev(actor.cell, otter.cell) <= self.tuning.sight {
                    self.loose_the_bow_at_the_otter(intent, actor, otter.cell, results);
                    return;
                }
            }
        }
        if self.beaver_is_nearest(actor.id) {
            if let Some(beaver) = self.beaver.clone() {
                if crate::model::chebyshev(actor.cell, beaver.cell) <= self.tuning.sight {
                    self.loose_the_bow_at_the_beaver(intent, actor, beaver.cell, results);
                    return;
                }
            }
            self.finish(
                intent.command_id.clone(),
                actor.id,
                false,
                "The beaver is not close enough for the bow.",
                results,
            );
            return;
        }
        if self.hare_is_nearest(actor.id) {
            if let Some(hare) = self.hare.clone() {
                if crate::model::chebyshev(actor.cell, hare.cell) <= self.tuning.sight {
                    self.loose_the_bow_at_the_hare(intent, actor, hare.cell, results);
                    return;
                }
            }
            self.finish(
                intent.command_id.clone(),
                actor.id,
                false,
                "The hare is not close enough for the bow.",
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
        let mut yours = crate::model::spoken_act(None, "bring", rest);
        if let Some(place) = self.carcass_lies_off(actor.id, deer.cell) {
            yours.push_str(&place);
        }
        self.emit(
            crate::model::spoken_act(Some(name.as_str()), "bring", rest),
            Scope::Cell(deer.cell),
            vec![actor.id],
            vec![deer.cell],
            vec![id],
            vec![],
            None,
        );
        self.finish(intent.command_id.clone(), actor.id, true, yours, results);
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
        let place = self.carcass_lies_off(intent.actor_id, deer_cell);
        let (heard, yours) = if flint {
            let rest = "the deer down with the flint spear.";
            let mut yours = crate::model::spoken_act(None, "bring", rest);
            if let Some(place) = place.as_deref() {
                yours.push_str(place);
            }
            (
                crate::model::spoken_act(Some(name.as_str()), "bring", rest),
                yours,
            )
        } else if let Some(place) = place.as_deref() {
            (
                format!("{name} brings the deer down."),
                format!("You bring the deer down.{place}"),
            )
        } else {
            (
                format!("{name} brings the deer down."),
                "You bring the deer down. A carcass lies where it stood.".to_string(),
            )
        };
        let wooden = self.wooden_kill(intent.actor_id);
        let mut items = vec![id];
        if let Some(spear) = wooden {
            items.push(spear);
        }
        self.emit(
            heard,
            Scope::Cell(deer_cell),
            vec![intent.actor_id],
            vec![deer_cell],
            items,
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

    fn apply_banks(&mut self, mut banks: Vec<Intent>, results: &mut Vec<CommandResult>) {
        banks.sort_by_key(|intent| intent.actor_id);
        for intent in banks {
            let Some(actor) = self.actor(intent.actor_id).cloned() else {
                continue;
            };
            let cell = actor.cell;
            if self.fire_at(cell).is_none() {
                self.finish(
                    intent.command_id,
                    actor.id,
                    false,
                    "There is no fire burning here.",
                    results,
                );
                continue;
            }
            if self.banks.contains_key(&cell) {
                self.finish(
                    intent.command_id,
                    actor.id,
                    false,
                    "The fire is already banked.",
                    results,
                );
                continue;
            }
            if self.loose_ids(actor.id, Material::Mud).is_empty()
                && self.pile_count(cell, Material::Mud) == 0
                && self.ground_ids(cell, Material::Mud).is_empty()
            {
                self.finish(
                    intent.command_id,
                    actor.id,
                    false,
                    "You have no mud to cover the fire.",
                    results,
                );
                continue;
            }
            if self.dry_wood_on_ground(cell) == 0 {
                let wet = self.pile_count(cell, Material::Branch) > 0
                    || !self.ground_ids(cell, Material::Branch).is_empty();
                let message = if wet {
                    "The wood is too wet to bank."
                } else {
                    "There is no dry wood here to bank."
                };
                self.finish(intent.command_id, actor.id, false, message, results);
                continue;
            }
            let _ = self.take_hand(actor.id, Material::Mud);
            let spent = self.take_dry_wood_on_ground(cell);
            self.banks.insert(cell, spent);
            let ready = self.time.saturating_add(self.tuning.clay_ticks);
            self.hold_banked_fire(cell, ready);
            self.schedule(ready, Wake::Charcoal(cell));
            let name = self.name_of(actor.id);
            let rest = "the branches on the fire and cover them with mud.";
            self.emit(
                crate::model::spoken_act(Some(name.as_str()), "bank", rest),
                Scope::Cell(cell),
                vec![actor.id],
                vec![cell],
                vec![],
                vec![],
                None,
            );
            self.finish(
                intent.command_id,
                actor.id,
                true,
                crate::model::spoken_act(None, "bank", rest),
                results,
            );
        }
    }

    /// A plain fire will not smelt iron. The reply names the verb that fits
    /// what is already here: feed a carried lump, gather one and feed it,
    /// bank dry branches with mud, or, when no charcoal is here yet, this
    /// fire will leave a lump when it burns down, then light with that.
    fn iron_charcoal_hint(&self, actor_id: crate::model::ActorId) -> String {
        let cell = self.actor(actor_id).map(|actor| actor.cell).unwrap_or(0);
        let hot = self.fire_at(cell).is_some_and(|fire| fire.hot);
        let in_arms = !self.loose_ids(actor_id, Material::Charcoal).is_empty();
        let on_ground = !self.ground_ids(cell, Material::Charcoal).is_empty();
        let mud = !self.loose_ids(actor_id, Material::Mud).is_empty()
            || self.pile_count(cell, Material::Mud) > 0
            || !self.ground_ids(cell, Material::Mud).is_empty();
        let wood = self.dry_wood_on_ground(cell) > 0;
        let hint = if in_arms && !hot {
            "Feed a lump of charcoal."
        } else if on_ground && !hot {
            "Gather the charcoal and feed it."
        } else if in_arms || on_ground {
            "Wait for this fire to go out, then light with the charcoal."
        } else if wood && mud {
            "Bank the branches with mud."
        } else {
            "This fire will leave a lump of charcoal when it burns down, then light with that."
        };
        format!("Iron needs a hot charcoal fire. {hint}")
    }

    fn iron_smelt_ready(&self, actor: &crate::model::Actor) -> Result<bool, String> {
        if self.loose_ids(actor.id, Material::IronOre).is_empty() {
            Err(self.gather_miss(
                actor.cell,
                Material::IronOre,
                "You are not carrying iron ore.".into(),
            ))
        } else if self.fire_at(actor.cell).is_none() {
            Err("There is no fire burning here.".into())
        } else if self.fire_at(actor.cell).is_some_and(|fire| !fire.charcoal) {
            Err(self.iron_charcoal_hint(actor.id))
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
            let (rest, from_charcoal) = if let Some(item_id) = self
                .loose_ids(intent.actor_id, Material::Charcoal)
                .first()
                .copied()
            {
                self.items.remove(&item_id);
                ("a lump of charcoal into the fire. It burns hotter.", true)
            } else if self.consume_dry(intent.actor_id, Material::Branch).is_ok() {
                ("a dry branch into the fire. It burns hotter.", false)
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
                if from_charcoal {
                    fire.charcoal = true;
                }
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
                let cell = self
                    .actor(intent.actor_id)
                    .map(|actor| actor.cell)
                    .unwrap_or(0);
                self.finish(
                    intent.command_id,
                    intent.actor_id,
                    false,
                    self.gather_miss(
                        cell,
                        Material::CopperOre,
                        "You are not carrying copper ore.".into(),
                    ),
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
            let ready = self.time.saturating_add(self.copper_smelt_wait(cell));
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

    /// A hot charcoal fire smelts copper in half the clay wait, and at least one tick.
    /// A plain fire still takes the full wait. The bead is the same either way.
    fn copper_smelt_wait(&self, cell: crate::model::CellId) -> u64 {
        let ticks = self.tuning.clay_ticks;
        if self.fire_at(cell).is_some_and(|fire| fire.charcoal) {
            (ticks / 2).max(1)
        } else {
            ticks
        }
    }

    fn apply_iron_smelt(&mut self, intent: Intent, results: &mut Vec<CommandResult>) {
        let Some(id) = self
            .loose_ids(intent.actor_id, Material::IronOre)
            .first()
            .copied()
        else {
            let cell = self
                .actor(intent.actor_id)
                .map(|actor| actor.cell)
                .unwrap_or(0);
            self.finish(
                intent.command_id,
                intent.actor_id,
                false,
                self.gather_miss(
                    cell,
                    Material::IronOre,
                    "You are not carrying iron ore.".into(),
                ),
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
        if self.fire_at(cell).is_some_and(|fire| !fire.charcoal) {
            let message = self.iron_charcoal_hint(intent.actor_id);
            self.finish(intent.command_id, intent.actor_id, false, message, results);
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
            let bead = self.copper_bead_id(intent.actor_id);
            let iron = matches!(intent.attempt, Attempt::HammerIron)
                || (matches!(intent.attempt, Attempt::Hammer)
                    && bead.is_none()
                    && !self.loose_ids(intent.actor_id, Material::Iron).is_empty());
            if iron {
                self.apply_iron_hammer(intent, results);
                continue;
            }
            let Some(bead) = bead else {
                let cell = self
                    .actor(intent.actor_id)
                    .map(|actor| actor.cell)
                    .unwrap_or(0);
                self.finish(
                    intent.command_id,
                    intent.actor_id,
                    false,
                    self.gather_miss(
                        cell,
                        Material::Copper,
                        "You are not carrying a copper bead.".into(),
                    ),
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

    /// A copper bead in the arms, or the one lying on this ground.
    fn copper_bead_id(&self, actor: crate::model::ActorId) -> Option<crate::model::ItemId> {
        if let Some(id) = self.loose_ids(actor, Material::Copper).first().copied() {
            return Some(id);
        }
        let cell = self.actor(actor)?.cell;
        self.ground_ids(cell, Material::Copper).first().copied()
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
                let message = self.hide_line(
                    intent.actor_id,
                    "You need a copper awl and a deer hide.",
                    "You need a copper awl and a wolf hide.",
                    "You need a copper awl and an otter hide.",
                    "You need a copper awl and a fox hide.",
                );
                self.finish(intent.command_id, intent.actor_id, false, message, results);
                continue;
            };
            let wolf = self.item(hide).is_some_and(|hide| hide.wolf);
            let otter = self.item(hide).is_some_and(|hide| hide.otter);
            let fox = self.item(hide).is_some_and(|hide| hide.fox);
            let hare = self.item(hide).is_some_and(|hide| hide.hare);
            let beaver = self.item(hide).is_some_and(|hide| hide.beaver);
            let rest = if beaver && bone_awl {
                "the beaver hide with the bone awl. It is ready to sew."
            } else if beaver {
                "the beaver hide with the copper awl. It is ready to sew."
            } else if hare && bone_awl {
                "the hare hide with the bone awl. It is ready to sew."
            } else if hare {
                "the hare hide with the copper awl. It is ready to sew."
            } else {
                match (otter, fox, wolf, bone_awl) {
                    (true, _, _, true) => "the otter hide with the bone awl. It is ready to sew.",
                    (true, _, _, false) => {
                        "the otter hide with the copper awl. It is ready to sew."
                    }
                    (_, true, _, true) => "the fox hide with the bone awl. It is ready to sew.",
                    (_, true, _, false) => "the fox hide with the copper awl. It is ready to sew.",
                    (_, _, true, true) => "the wolf hide with the bone awl. It is ready to sew.",
                    (_, _, true, false) => "the wolf hide with the copper awl. It is ready to sew.",
                    (_, _, _, true) => "the deer hide with the bone awl. It is ready to sew.",
                    (_, _, _, false) => "the deer hide with the copper awl. It is ready to sew.",
                }
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

    /// A flint flake, a flint nodule, a loose stone, or a clear stone of quartz.
    /// The flake is named first. Quartz in the arms or underfoot is the stone.
    /// It is not gathered and it is not used up.
    fn scrape_tool(&self, actor: ActorId) -> Option<&'static str> {
        if !self.loose_ids(actor, Material::Flake).is_empty() {
            return Some("the flint flake");
        }
        if !self.loose_ids(actor, Material::Flint).is_empty() {
            return Some("the flint");
        }
        if !self.loose_ids(actor, Material::Stone).is_empty() {
            return Some("the stone");
        }
        if !self.loose_ids(actor, Material::Quartz).is_empty() {
            return Some("the clear stone");
        }
        let cell = self.actor(actor)?.cell;
        if self.pile_count(cell, Material::Quartz) > 0
            || !self.ground_ids(cell, Material::Quartz).is_empty()
        {
            return Some("the clear stone");
        }
        None
    }

    /// A hide that still needs scraping, carried first, then one on this ground.
    fn scrape_hide(&self, actor: ActorId) -> Option<ItemId> {
        let ready = |id: &ItemId| {
            self.item(*id)
                .is_some_and(|hide| !hide.scraped && !hide.sewn)
        };
        self.loose_ids(actor, Material::Hide)
            .into_iter()
            .find(ready)
            .or_else(|| {
                let cell = self.actor(actor)?.cell;
                self.ground_ids(cell, Material::Hide)
                    .into_iter()
                    .find(ready)
            })
    }

    /// Any hide in the arms or on this ground, for a refusal that names it.
    fn scrape_hide_any(&self, actor: ActorId) -> Option<ItemId> {
        self.loose_ids(actor, Material::Hide)
            .into_iter()
            .next()
            .or_else(|| {
                let cell = self.actor(actor)?.cell;
                self.ground_ids(cell, Material::Hide).into_iter().next()
            })
    }

    fn apply_scrapes(&mut self, scrapes: Vec<Intent>, results: &mut Vec<CommandResult>) {
        for intent in scrapes {
            let Some(tool) = self.scrape_tool(intent.actor_id) else {
                self.finish(
                    intent.command_id,
                    intent.actor_id,
                    false,
                    "You need a flint flake or a stone to scrape the hide.",
                    results,
                );
                continue;
            };
            let Some(hide) = self.scrape_hide(intent.actor_id) else {
                self.finish(
                    intent.command_id,
                    intent.actor_id,
                    false,
                    "You are not carrying a deer hide.",
                    results,
                );
                continue;
            };
            let wolf = self.item(hide).is_some_and(|hide| hide.wolf);
            let otter = self.item(hide).is_some_and(|hide| hide.otter);
            let fox = self.item(hide).is_some_and(|hide| hide.fox);
            let hare = self.item(hide).is_some_and(|hide| hide.hare);
            let beaver = self.item(hide).is_some_and(|hide| hide.beaver);
            let kind = if beaver {
                "beaver"
            } else if hare {
                "hare"
            } else if otter {
                "otter"
            } else if fox {
                "fox"
            } else if wolf {
                "wolf"
            } else {
                "deer"
            };
            let rest = format!("the {kind} hide with {tool}.");
            if let Some(item) = self.items.get_mut(&hide) {
                item.scraped = true;
            }
            let cell = self
                .actor(intent.actor_id)
                .map(|actor| actor.cell)
                .unwrap_or(0);
            let name = self.name_of(intent.actor_id);
            self.emit(
                crate::model::spoken_act(Some(name.as_str()), "scrape", &rest),
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
                crate::model::spoken_act(None, "scrape", &rest),
                results,
            );
        }
    }

    fn apply_sews(&mut self, sews: Vec<Intent>, results: &mut Vec<CommandResult>) {
        for intent in sews {
            let hide = self
                .loose_ids(intent.actor_id, Material::Hide)
                .into_iter()
                .find(|id| {
                    self.item(*id)
                        .is_some_and(|hide| hide.pierced && hide.scraped)
                });
            let Some(hide) = hide else {
                let raw = self
                    .loose_ids(intent.actor_id, Material::Hide)
                    .iter()
                    .any(|id| {
                        self.item(*id)
                            .is_some_and(|hide| !hide.scraped && !hide.sewn)
                    });
                let message = if raw {
                    self.hide_line(
                        intent.actor_id,
                        "The deer hide is not scraped.",
                        "The wolf hide is not scraped.",
                        "The otter hide is not scraped.",
                        "The fox hide is not scraped.",
                    )
                    .to_string()
                } else {
                    self.unpierced_sew_line(intent.actor_id)
                };
                self.finish(intent.command_id, intent.actor_id, false, message, results);
                continue;
            };
            let wolf = self.item(hide).is_some_and(|hide| hide.wolf);
            let otter = self.item(hide).is_some_and(|hide| hide.otter);
            let fox = self.item(hide).is_some_and(|hide| hide.fox);
            let hare = self.item(hide).is_some_and(|hide| hide.hare);
            let beaver = self.item(hide).is_some_and(|hide| hide.beaver);
            let rest = if beaver {
                "the pierced beaver hide into a cloak."
            } else if hare {
                "the pierced hare hide into a cloak."
            } else if otter {
                "the pierced otter hide into a cloak."
            } else if fox {
                "the pierced fox hide into a cloak."
            } else if wolf {
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

    /// Fire, then the herb, then a pot or a bark cup, then water in that vessel.
    /// Each miss names the one thing that is missing.
    fn steep_refusal(&self, actor: &crate::model::Actor, herb: Material) -> Option<String> {
        if self.fire_at(actor.cell).is_none() {
            return Some("There is no fire burning here.".into());
        }
        if self.loose_ids(actor.id, herb).is_empty() {
            return Some(
                match herb {
                    Material::Needle => "You have no pine needles to steep.",
                    Material::WillowBark => "You have no willow bark to steep.",
                    _ => "You have no mint to steep.",
                }
                .into(),
            );
        }
        let pot = !self.loose_ids(actor.id, Material::Pot).is_empty();
        let cup = !self.loose_ids(actor.id, Material::BarkCup).is_empty();
        if !pot && !cup {
            return Some(
                match herb {
                    Material::Needle => "You need a pot or a bark cup to steep the pine needles.",
                    Material::WillowBark => {
                        "You need a pot or a bark cup to steep the willow bark."
                    }
                    _ => "You need a pot or a bark cup to steep the mint.",
                }
                .into(),
            );
        }
        if self.loose_ids(actor.id, Material::Water).is_empty() {
            let bare = if pot {
                "The pot has no water."
            } else {
                "The bark cup has no water."
            };
            return Some(match self.water_step_cue(actor.cell) {
                Some(cue) => {
                    let ice = if self.frozen_still_beside(actor.cell) {
                        " The still is frozen."
                    } else {
                        ""
                    };
                    format!("{bare}{ice}{cue}")
                }
                None => bare.into(),
            });
        }
        None
    }

    /// A strip of willow bark and a drink of water, in a pot or a bark cup, over a fire.
    /// Both are spent. The tea is drunk in the same act. A sunburn eases by two open-sky ticks.
    /// A sick stomach and a chill are left alone. The sunburn line stays in the reply.
    fn apply_willow_teas(&mut self, teas: Vec<Intent>, results: &mut Vec<CommandResult>) {
        for intent in teas {
            let Some(actor) = self.actor(intent.actor_id).cloned() else {
                continue;
            };
            let cell = actor.cell;
            let refuse = self.steep_refusal(&actor, Material::WillowBark);
            if let Some(message) = refuse {
                self.emit(
                    message.to_string(),
                    crate::model::Scope::Personal(actor.id),
                    vec![actor.id],
                    vec![cell],
                    vec![],
                    vec![],
                    None,
                );
                self.finish(
                    intent.command_id,
                    intent.actor_id,
                    false,
                    message.to_string(),
                    results,
                );
                continue;
            }
            let bark_id = self.loose_ids(actor.id, Material::WillowBark)[0];
            let water_id = self.loose_ids(actor.id, Material::Water)[0];
            let in_pot = !self.loose_ids(actor.id, Material::Pot).is_empty();
            let vessel = if in_pot {
                "the pot"
            } else if self
                .loose_ids(actor.id, Material::BarkCup)
                .iter()
                .any(|id| self.item(*id).is_some_and(|cup| cup.pitched))
            {
                "the sealed bark cup"
            } else {
                "the bark cup"
            };
            self.items.remove(&bark_id);
            self.items.remove(&water_id);
            let stung = actor.sunburn >= self.tuning.pink_at;
            if stung {
                let ease = self.tuning.sun_per_tick.saturating_mul(2);
                if let Some(person) = self.actors.get_mut(&actor.id) {
                    person.sunburn = person.sunburn.saturating_sub(ease);
                }
            }
            let rest_you = format!(
                "a strip of willow bark in {vessel} over the fire and drink the willow bark tea."
            );
            let rest_them = format!(
                "a strip of willow bark in {vessel} over the fire and drinks the willow bark tea."
            );
            let name = self.name_of(intent.actor_id);
            self.emit(
                crate::model::spoken_act(Some(name.as_str()), "steep", &rest_them),
                crate::model::Scope::Cell(cell),
                vec![intent.actor_id],
                vec![cell],
                vec![],
                vec![],
                None,
            );
            let mut reply = crate::model::spoken_act(None, "steep", &rest_you);
            if stung {
                reply.push_str(" The sunburn eases.");
            }
            self.finish(intent.command_id, intent.actor_id, true, reply, results);
        }
    }

    /// A handful of pine needles and a drink of water, in a pot or a bark cup, over a fire.
    /// Both are spent. The tea is drunk in the same act. A chill eases by one open-wind tick.
    /// A sick stomach is left alone. The chill line stays in the reply.
    fn apply_pine_teas(&mut self, teas: Vec<Intent>, results: &mut Vec<CommandResult>) {
        for intent in teas {
            let Some(actor) = self.actor(intent.actor_id).cloned() else {
                continue;
            };
            let cell = actor.cell;
            let refuse = self.steep_refusal(&actor, Material::Needle);
            if let Some(message) = refuse {
                self.emit(
                    message.to_string(),
                    crate::model::Scope::Personal(actor.id),
                    vec![actor.id],
                    vec![cell],
                    vec![],
                    vec![],
                    None,
                );
                self.finish(
                    intent.command_id,
                    intent.actor_id,
                    false,
                    message.to_string(),
                    results,
                );
                continue;
            }
            let needle_id = self.loose_ids(actor.id, Material::Needle)[0];
            let water_id = self.loose_ids(actor.id, Material::Water)[0];
            let in_pot = !self.loose_ids(actor.id, Material::Pot).is_empty();
            let vessel = if in_pot {
                "the pot"
            } else if self
                .loose_ids(actor.id, Material::BarkCup)
                .iter()
                .any(|id| self.item(*id).is_some_and(|cup| cup.pitched))
            {
                "the sealed bark cup"
            } else {
                "the bark cup"
            };
            self.items.remove(&needle_id);
            self.items.remove(&water_id);
            let was_chilly = actor.cold >= self.tuning.chilly_at;
            if was_chilly {
                let ease = self.tuning.cold_per_tick.saturating_mul(2);
                if let Some(person) = self.actors.get_mut(&actor.id) {
                    person.cold = person.cold.saturating_sub(ease);
                }
            }
            let rest_you = format!(
                "a handful of pine needles in {vessel} over the fire and drink the pine needle tea."
            );
            let rest_them = format!(
                "a handful of pine needles in {vessel} over the fire and drinks the pine needle tea."
            );
            let name = self.name_of(intent.actor_id);
            self.emit(
                crate::model::spoken_act(Some(name.as_str()), "steep", &rest_them),
                crate::model::Scope::Cell(cell),
                vec![intent.actor_id],
                vec![cell],
                vec![],
                vec![],
                None,
            );
            let mut reply = crate::model::spoken_act(None, "steep", &rest_you);
            if was_chilly {
                reply.push_str(" The chill eases.");
            }
            self.finish(intent.command_id, intent.actor_id, true, reply, results);
        }
    }

    /// A handful of mint and a drink of water, in a pot or a bark cup, over a fire.
    /// Both are spent. The tea is drunk in the same act. A bowl is not enough.
    /// The stomach line stays in the reply. The stored sentence does not carry it.
    fn apply_brews(&mut self, brews: Vec<Intent>, results: &mut Vec<CommandResult>) {
        for intent in brews {
            let Some(actor) = self.actor(intent.actor_id).cloned() else {
                continue;
            };
            let cell = actor.cell;
            let refuse = self.steep_refusal(&actor, Material::Mint);
            if let Some(message) = refuse {
                self.emit(
                    message.to_string(),
                    crate::model::Scope::Personal(actor.id),
                    vec![actor.id],
                    vec![cell],
                    vec![],
                    vec![],
                    None,
                );
                self.finish(
                    intent.command_id,
                    intent.actor_id,
                    false,
                    message.to_string(),
                    results,
                );
                continue;
            }
            let mint_id = self.loose_ids(actor.id, Material::Mint)[0];
            let water_id = self.loose_ids(actor.id, Material::Water)[0];
            let in_pot = !self.loose_ids(actor.id, Material::Pot).is_empty();
            let vessel = if in_pot {
                "the pot"
            } else if self
                .loose_ids(actor.id, Material::BarkCup)
                .iter()
                .any(|id| self.item(*id).is_some_and(|cup| cup.pitched))
            {
                "the sealed bark cup"
            } else {
                "the bark cup"
            };
            self.items.remove(&mint_id);
            self.items.remove(&water_id);
            let was_ill = actor.illness >= self.tuning.ill_at;
            self.ease_illness(actor.id, self.tuning.illness_hit);
            let rest_you =
                format!("a handful of mint in {vessel} over the fire and drink the mint tea.");
            let rest_them =
                format!("a handful of mint in {vessel} over the fire and drinks the mint tea.");
            let name = self.name_of(intent.actor_id);
            self.emit(
                crate::model::spoken_act(Some(name.as_str()), "steep", &rest_them),
                crate::model::Scope::Cell(cell),
                vec![intent.actor_id],
                vec![cell],
                vec![],
                vec![],
                None,
            );
            let mut reply = crate::model::spoken_act(None, "steep", &rest_you);
            if was_ill {
                reply.push_str(" Your stomach eases.");
            }
            self.finish(intent.command_id, intent.actor_id, true, reply, results);
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
                && self.loose_ids(actor.id, Material::BarkCup).is_empty()
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
            let in_cup = !in_pot && !self.loose_ids(actor.id, Material::BarkCup).is_empty();
            let sealed_cup = in_cup
                && self
                    .loose_ids(actor.id, Material::BarkCup)
                    .iter()
                    .any(|id| self.item(*id).is_some_and(|cup| cup.pitched));
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
                    boil_witness_rest(in_pot, in_cup, sealed_cup)
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
                crate::model::spoken_act(None, "boil", boil_actor_rest(in_pot, in_cup, sealed_cup))
            );
            self.finish(intent.command_id, intent.actor_id, true, message, results);
        }
    }

    /// Still water on this cell, the brook underfoot, or the brook beside this
    /// bank. The bool is true when the drink is still water. A bank where a
    /// baited hook can take a fish is the brook, not a step into the water.
    fn drink_source(&self, cell: crate::model::CellId) -> Option<bool> {
        if self.water_here(cell) {
            let still = self.cell(cell).still && self.cell(cell).region != Region::Brook;
            return Some(still);
        }
        self.brook_stretch(cell).map(|_| false)
    }

    fn apply_drinks(&mut self, drinks: Vec<Intent>, results: &mut Vec<CommandResult>) {
        for intent in drinks {
            let Some(actor) = self.actor(intent.actor_id).cloned() else {
                continue;
            };
            let Some(still) = self.drink_source(actor.cell) else {
                self.finish(
                    intent.command_id,
                    intent.actor_id,
                    false,
                    "The brook is not here.",
                    results,
                );
                continue;
            };
            if still && self.still_frozen(actor.cell) {
                self.finish(
                    intent.command_id,
                    intent.actor_id,
                    false,
                    "The still water is frozen.",
                    results,
                );
                continue;
            }
            // The bank beside the brook is that same running water, and it
            // does not roll the sickness of the still or of a wade in the brook.
            let on_bank = !self.water_here(actor.cell);
            let chance = if still {
                self.tuning.still_chance
            } else if on_bank {
                0
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
            // Already not thirsty: the reply says so. It does not claim the thirst eases.
            let mut message = if self.thirst_word(thirst_before) == "not thirsty" {
                format!("{lead} You are not thirsty.")
            } else if self.thirst_word(thirst_after) == "not thirsty" {
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
            if material == Material::Cattail {
                self.eat_cattail(intent, results);
                continue;
            }
            if material == Material::Flour {
                self.eat_flour(intent, results);
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
            if material != Material::Berry || !self.sweet_berries_underfoot(intent.actor_id) {
                self.finish(
                    intent.command_id,
                    intent.actor_id,
                    false,
                    "There are no sweet berries here to eat.",
                    results,
                );
                continue;
            }
            let Ok(item) = self.take_sweet_berry(intent.actor_id) else {
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

    fn eat_cattail(&mut self, intent: Intent, results: &mut Vec<CommandResult>) {
        let Some(id) = self.cooked_cattail_id(intent.actor_id) else {
            let message = if self.raw_cattail_near(intent.actor_id) {
                "Raw cattail roots do not feed you. They must be cooked."
            } else {
                "There are no cattail roots here to eat."
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
        let rest = "the cooked cattail roots.";
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

    fn eat_flour(&mut self, intent: Intent, results: &mut Vec<CommandResult>) {
        let Some(id) = self.cooked_flour_id(intent.actor_id) else {
            let message = if self.raw_flour_id(intent.actor_id).is_some() {
                "Raw cattail flour does not feed you. It must be cooked."
            } else {
                "There is no cattail flour here to eat."
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
        let rest = "the cattail flatbread.";
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

    /// Cordage for the bow. Hide strips, then willow, then nettle.
    /// A bundle in the arms is spent before one lying in the cell.
    fn bow_line(&self, actor: crate::model::ActorId) -> Option<(crate::model::ItemId, Line)> {
        let cell = self.bow_cell(actor);
        if let Some(id) = self.loose_ids(actor, Material::Cord).first().copied() {
            return Some((id, Line::Hide));
        }
        if let Some(id) = self.ground_ids(cell, Material::Cord).first().copied() {
            return Some((id, Line::Hide));
        }
        if let Some(id) = self.loose_ids(actor, Material::Willow).first().copied() {
            return Some((id, Line::Willow));
        }
        if let Some(id) = self.ground_ids(cell, Material::Willow).first().copied() {
            return Some((id, Line::Willow));
        }
        if let Some(id) = self.loose_ids(actor, Material::Nettle).first().copied() {
            return Some((id, Line::Nettle));
        }
        self.ground_ids(cell, Material::Nettle)
            .first()
            .copied()
            .map(|id| (id, Line::Nettle))
    }

    pub(crate) fn bow_refusal(&self, actor: crate::model::ActorId) -> Option<&'static str> {
        let cell = self.bow_cell(actor);
        let dry = self.bow_branch_id(actor).is_some();
        let any_branch = !self.loose_ids(actor, Material::Branch).is_empty()
            || !self.ground_ids(cell, Material::Branch).is_empty();
        let line = self.bow_line(actor).is_some();
        if dry && line {
            None
        } else if any_branch && line {
            Some("The branch is not dry.")
        } else if !dry && !line {
            Some("You need a dry branch and hide strips or willow cordage.")
        } else if !dry {
            Some("You need a dry branch.")
        } else {
            Some("You need hide strips or willow cordage.")
        }
    }

    /// Two dry branches for a bow drill. Arms, then loose wood on this ground, then the pile.
    fn drill_wood(&self, actor: crate::model::ActorId) -> (Vec<crate::model::ItemId>, u32) {
        let mut ids = self.dry_loose_ids(actor, Material::Branch);
        let cell = self.bow_cell(actor);
        for id in self.ground_ids(cell, Material::Branch) {
            if ids.len() >= 2 {
                break;
            }
            if self.item(id).is_some_and(|item| self.is_dry(item.moisture)) && !ids.contains(&id) {
                ids.push(id);
            }
        }
        ids.truncate(2);
        let need = 2u32.saturating_sub(ids.len() as u32);
        let from_pile = need.min(self.dry_pile_count(cell, Material::Branch));
        (ids, from_pile)
    }

    fn dry_branches_at_hand(&self, actor: crate::model::ActorId) -> u32 {
        let cell = self.bow_cell(actor);
        let loose = self.dry_loose_ids(actor, Material::Branch).len() as u32;
        let ground = self
            .ground_ids(cell, Material::Branch)
            .into_iter()
            .filter(|id| {
                self.item(*id)
                    .is_some_and(|item| self.is_dry(item.moisture))
            })
            .count() as u32;
        loose + ground + self.dry_pile_count(cell, Material::Branch)
    }

    fn branches_at_hand(&self, actor: crate::model::ActorId) -> u32 {
        let cell = self.bow_cell(actor);
        let loose = self.loose_ids(actor, Material::Branch).len() as u32;
        let ground = self.ground_ids(cell, Material::Branch).len() as u32;
        loose + ground + self.pile_count(cell, Material::Branch)
    }

    /// The drill takes a slot. Branches and cordage already in the arms free theirs.
    fn drill_arms_full(&self, actor: crate::model::ActorId) -> bool {
        let (ids, _) = self.drill_wood(actor);
        let Some((cord, _)) = self.bow_line(actor) else {
            return false;
        };
        let mut freed = 0u32;
        for id in ids.into_iter().chain(std::iter::once(cord)) {
            if self.item(id).is_some_and(|item| {
                item.carrier == Some(actor) && !item.in_basket && !item.worn_cloak()
            }) {
                freed += 1;
            }
        }
        self.carry_count(actor) + 1 > self.tuning.carry_slots + freed
    }

    fn spent_drill_in_arms(&self, actor: crate::model::ActorId) -> Option<crate::model::ItemId> {
        self.loose_ids(actor, Material::Drill)
            .into_iter()
            .find(|id| self.item(*id).is_some_and(|drill| drill.spent))
    }

    pub(crate) fn drill_refusal(&self, actor: crate::model::ActorId) -> Option<&'static str> {
        if self.spent_drill_in_arms(actor).is_some() {
            let dry = self.dry_branches_at_hand(actor);
            let any = self.branches_at_hand(actor);
            if dry >= 1 {
                return None;
            }
            if any > 0 {
                return Some("The branch is not dry.");
            }
            return Some("You need a dry branch to carve the bow drill anew.");
        }
        let dry = self.dry_branches_at_hand(actor);
        let any = self.branches_at_hand(actor);
        let line = self.bow_line(actor).is_some();
        if dry >= 2 && line {
            if self.drill_arms_full(actor) {
                Some(ARMS_FULL)
            } else {
                None
            }
        } else if line && any > 0 && dry == 0 {
            Some("The branch is not dry.")
        } else if dry < 2 && !line {
            Some("You need two dry branches and hide strips or willow cordage.")
        } else if dry < 2 {
            Some("You need two dry branches, one for the spindle and one for the fireboard.")
        } else {
            Some("You need hide strips or willow cordage.")
        }
    }

    fn drill_ready_id(&self, actor: crate::model::ActorId) -> Option<crate::model::ItemId> {
        let ready = |id: crate::model::ItemId| {
            self.item(id)
                .is_some_and(|item| !item.spent && self.is_dry(item.moisture))
        };
        if let Some(id) = self
            .loose_ids(actor, Material::Drill)
            .into_iter()
            .find(|id| ready(*id))
        {
            return Some(id);
        }
        self.ground_ids(self.bow_cell(actor), Material::Drill)
            .into_iter()
            .find(|id| ready(*id))
    }

    fn spent_drill_at_hand(&self, actor: crate::model::ActorId) -> bool {
        self.loose_ids(actor, Material::Drill)
            .into_iter()
            .chain(self.ground_ids(self.bow_cell(actor), Material::Drill))
            .any(|id| self.item(id).is_some_and(|drill| drill.spent))
    }

    fn drill_anywhere(&self, actor: crate::model::ActorId) -> bool {
        !self.loose_ids(actor, Material::Drill).is_empty()
            || !self
                .ground_ids(self.bow_cell(actor), Material::Drill)
                .is_empty()
    }

    pub(crate) fn drill_fire_refusal(&self, actor: crate::model::ActorId) -> Option<&'static str> {
        let person = self.actor(actor)?;
        if self.raining && self.exposure_millis(person.cell) >= 1000 {
            return Some("The rain puts the drill out before it catches.");
        }
        if self.fire_at(person.cell).is_some() {
            return Some("A fire already burns here.");
        }
        if self.drill_ready_id(actor).is_some() {
            return None;
        }
        if self.spent_drill_at_hand(actor) {
            return Some("The bow drill is spent.");
        }
        if self.drill_anywhere(actor) {
            Some("The bow drill is too damp to catch.")
        } else {
            Some("You have no bow drill.")
        }
    }

    fn apply_drill_carves(&mut self, carves: Vec<Intent>, results: &mut Vec<CommandResult>) {
        for intent in carves {
            if let Some(message) = self.drill_refusal(intent.actor_id) {
                self.finish(intent.command_id, intent.actor_id, false, message, results);
                continue;
            }
            if let Some(drill) = self.spent_drill_in_arms(intent.actor_id) {
                let (branches, from_pile) = self.drill_wood(intent.actor_id);
                let cell = self.bow_cell(intent.actor_id);
                if let Some(branch) = branches.first().copied() {
                    self.items.remove(&branch);
                } else if from_pile > 0 {
                    self.take_from_pile(cell, Material::Branch, 1);
                } else {
                    self.finish(
                        intent.command_id,
                        intent.actor_id,
                        false,
                        "You need a dry branch to carve the bow drill anew.",
                        results,
                    );
                    continue;
                }
                if let Some(item) = self.items.get_mut(&drill) {
                    item.spent = false;
                }
                let name = self.name_of(intent.actor_id);
                let rest = "the spent bow drill anew with one dry branch.";
                self.emit(
                    crate::model::spoken_act(Some(name.as_str()), "carve", rest),
                    Scope::Cell(cell),
                    vec![intent.actor_id],
                    vec![cell],
                    vec![drill],
                    vec![],
                    None,
                );
                self.finish(
                    intent.command_id,
                    intent.actor_id,
                    true,
                    crate::model::spoken_act(None, "carve", rest),
                    results,
                );
                continue;
            }
            let (branches, from_pile) = self.drill_wood(intent.actor_id);
            let Some((cord, kind)) = self.bow_line(intent.actor_id) else {
                self.finish(
                    intent.command_id,
                    intent.actor_id,
                    false,
                    "You need hide strips or willow cordage.",
                    results,
                );
                continue;
            };
            if branches.len() as u32 + from_pile < 2 {
                self.finish(
                    intent.command_id,
                    intent.actor_id,
                    false,
                    "You need two dry branches, one for the spindle and one for the fireboard.",
                    results,
                );
                continue;
            }
            let cell = self.bow_cell(intent.actor_id);
            for id in &branches {
                self.items.remove(id);
            }
            self.items.remove(&cord);
            if from_pile > 0 {
                self.take_from_pile(cell, Material::Branch, from_pile);
            }
            let id = self.mint_item(Material::Drill, 0, cell, Some(intent.actor_id), Role::Loose);
            let name = self.name_of(intent.actor_id);
            let rest = match kind {
                Line::Hide => "a bow drill from two dry branches and hide strips.",
                Line::Willow => "a bow drill from two dry branches and willow cordage.",
                Line::Nettle => "a bow drill from two dry branches and nettle cordage.",
                Line::Sinew => "a bow drill from two dry branches and sinew.",
            };
            self.emit(
                crate::model::spoken_act(Some(name.as_str()), "carve", rest),
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
                crate::model::spoken_act(None, "carve", rest),
                results,
            );
        }
    }

    fn apply_drill_fires(
        &mut self,
        fires: Vec<Intent>,
        start_cells: &BTreeMap<crate::model::ActorId, crate::model::CellId>,
        results: &mut Vec<CommandResult>,
    ) {
        let mut groups: BTreeMap<crate::model::CellId, Vec<Intent>> = BTreeMap::new();
        for intent in fires {
            let cell = start_cells.get(&intent.actor_id).copied().unwrap_or(0);
            groups.entry(cell).or_default().push(intent);
        }
        for (cell, mut claims) in groups {
            let subject = format!("{cell}:drill");
            claims.sort_by(|a, b| {
                let left = contest_hash(self.seed, self.time, "drill", &subject, a.actor_id);
                let right = contest_hash(self.seed, self.time, "drill", &subject, b.actor_id);
                left.cmp(&right).then(a.actor_id.cmp(&b.actor_id))
            });
            let mut lit = self.fire_at(cell).is_some();
            for intent in claims {
                if lit || self.drill_fire_refusal(intent.actor_id).is_some() {
                    let message = self
                        .drill_fire_refusal(intent.actor_id)
                        .unwrap_or("A fire already burns here.");
                    self.finish(intent.command_id, intent.actor_id, false, message, results);
                    continue;
                }
                let Some(drill) = self.drill_ready_id(intent.actor_id) else {
                    self.finish(
                        intent.command_id,
                        intent.actor_id,
                        false,
                        "You have no bow drill.",
                        results,
                    );
                    continue;
                };
                if let Some(item) = self.items.get_mut(&drill) {
                    item.spent = true;
                }
                let name = self.name_of(intent.actor_id);
                let rest = "a fire with the bow drill. The bow drill is spent.";
                let hash = contest_hash(self.seed, self.time, "drill", &subject, intent.actor_id);
                let event = self.emit(
                    crate::model::spoken_act(Some(name.as_str()), "light", rest),
                    Scope::Cell(cell),
                    vec![intent.actor_id],
                    vec![cell],
                    vec![drill],
                    vec![],
                    Some(Draw {
                        tag: "drill".into(),
                        subject: subject.clone(),
                        value: hash,
                    }),
                );
                self.mint_fire(cell, event);
                lit = true;
                let mut yours = crate::model::spoken_act(None, "light", rest);
                if self.scare_wolf(Some(intent.actor_id)) {
                    yours.push_str(" The wolf flees the fire.");
                }
                self.finish(intent.command_id, intent.actor_id, true, yours, results);
            }
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
            let Some((line, kind)) = self.bow_line(intent.actor_id) else {
                self.finish(
                    intent.command_id,
                    intent.actor_id,
                    false,
                    "You need hide strips or willow cordage.",
                    results,
                );
                continue;
            };
            let cell = self
                .actor(intent.actor_id)
                .map(|actor| actor.cell)
                .unwrap_or(0);
            self.items.remove(&branch);
            self.items.remove(&line);
            let id = self.mint_item(Material::Bow, 0, cell, Some(intent.actor_id), Role::Loose);
            if let Some(bow) = self.items.get_mut(&id) {
                bow.bound_willow = kind == Line::Willow;
                bow.bound_nettle = kind == Line::Nettle;
            }
            let name = self.name_of(intent.actor_id);
            let rest = match kind {
                Line::Hide => "a bow from a dry branch and hide strips.",
                Line::Willow => "a bow from a dry branch and willow cordage.",
                Line::Nettle => "a bow from a dry branch and nettle cordage.",
                Line::Sinew => "a bow from a dry branch and sinew.",
            };
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
                person.nettle_sting = false;
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
                ARMS_FULL,
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
fn boil_witness_rest(in_pot: bool, in_cup: bool, sealed_cup: bool) -> &'static str {
    if in_pot {
        "the drink in the pot over the fire and drinks it."
    } else if sealed_cup {
        "the drink in the sealed bark cup over the fire and drinks it."
    } else if in_cup {
        "the drink in the bark cup over the fire and drinks it."
    } else {
        "a drink of water over the fire and drinks it."
    }
}

fn boil_actor_rest(in_pot: bool, in_cup: bool, sealed_cup: bool) -> &'static str {
    if in_pot {
        "the drink in the pot over the fire and drink it."
    } else if sealed_cup {
        "the drink in the sealed bark cup over the fire and drink it."
    } else if in_cup {
        "the drink in the bark cup over the fire and drink it."
    } else {
        "a drink of water over the fire and drink it."
    }
}

fn worn_phrase(
    wolf: bool,
    otter: bool,
    fox: bool,
    hare: bool,
    beaver: bool,
    sewn: bool,
) -> &'static str {
    match (otter, fox, hare, beaver, wolf, sewn) {
        (true, _, _, _, _, true) => "the sewn otter-hide cloak.",
        (true, _, _, _, _, false) => "the otter hide.",
        (_, true, _, _, _, true) => "the sewn fox-hide cloak.",
        (_, true, _, _, _, false) => "the fox hide.",
        (_, _, true, _, _, true) => "the sewn hare-hide cloak.",
        (_, _, true, _, _, false) => "the hare hide.",
        (_, _, _, true, _, true) => "the sewn beaver-hide cloak.",
        (_, _, _, true, _, false) => "the beaver hide.",
        (_, _, _, _, true, true) => "the sewn wolf-hide cloak.",
        (_, _, _, _, true, false) => "the wolf hide.",
        (_, _, _, _, _, true) => "the sewn deer-hide cloak.",
        (_, _, _, _, _, false) => "the deer hide.",
    }
}

impl World {
    /// Name the hide the person is holding. An otter hide wins over a fox
    /// hide, a fox hide over a wolf hide, and a wolf hide over a deer hide.
    fn hide_line<'a>(
        &self,
        actor: ActorId,
        deer: &'a str,
        wolf: &'a str,
        otter: &'a str,
        fox: &'a str,
    ) -> &'a str {
        if self.carrying_otter_hide(actor) {
            otter
        } else if self.carrying_fox_hide(actor) {
            fox
        } else if self.carrying_wolf_hide(actor) {
            wolf
        } else if self.carrying_hare_hide(actor) {
            hare_instead(deer)
        } else if self.carrying_beaver_hide(actor) {
            beaver_instead(deer)
        } else {
            deer
        }
    }

    /// The awl a sew miss should name. Arms win. The ground at the feet is next.
    /// One kind is named. Both kinds name the first. None names both tools.
    fn awl_tool_phrase(&self, ids: &[crate::model::ItemId]) -> Option<&'static str> {
        if ids.is_empty() {
            return None;
        }
        let bone = ids
            .iter()
            .any(|id| self.item(*id).is_some_and(|awl| awl.bone));
        let copper = ids
            .iter()
            .any(|id| self.item(*id).is_some_and(|awl| !awl.bone));
        Some(if bone && !copper {
            "the bone awl"
        } else if copper && !bone {
            "the copper awl"
        } else if self.item(ids[0]).is_some_and(|awl| awl.bone) {
            "the bone awl"
        } else {
            "the copper awl"
        })
    }

    /// An awl one cell off, orthogonal first, then a single diagonal.
    /// The words match a gather miss. Two cells off is no step. This does not move anyone.
    fn awl_neighbor(&self, actor: ActorId) -> Option<(&'static str, String)> {
        let cell = self.actor(actor)?.cell;
        let (x, y) = cell_xy(cell);
        let mut ortho = Vec::new();
        for dir in Dir::all() {
            let (dx, dy) = dir.delta();
            let Some(dest) = cell_id(x + dx, y + dy) else {
                continue;
            };
            let ids = self.ground_ids(dest, Material::Awl);
            if !ids.is_empty() {
                ortho.push((dir.word(), ids));
            }
        }
        let (ways, ids) = if !ortho.is_empty() {
            let ids = ortho
                .iter()
                .flat_map(|(_, ids)| ids.iter().copied())
                .collect::<Vec<_>>();
            let ways = ortho
                .iter()
                .map(|(way, _)| (*way).to_string())
                .collect::<Vec<_>>();
            (ways, ids)
        } else {
            let mut diag = Vec::new();
            for (dx, dy) in [(-1, -1), (1, -1), (-1, 1), (1, 1)] {
                let Some(dest) = cell_id(x + dx, y + dy) else {
                    continue;
                };
                let ids = self.ground_ids(dest, Material::Awl);
                if !ids.is_empty() {
                    diag.push((self.relative(cell, dest), ids));
                }
            }
            if diag.len() != 1 {
                return None;
            }
            let (way, ids) = diag.remove(0);
            (vec![way], ids)
        };
        let cue = match ways.len() {
            1 => format!(" Step {}.", ways[0]),
            _ => {
                let last = ways.last()?.to_string();
                let head = ways[..ways.len() - 1].join(", ");
                format!(" Step {head} or {last}.")
            }
        };
        Some((self.awl_tool_phrase(&ids)?, cue))
    }

    /// A scraped hide that is not pierced. Arms, then the feet, then one cell off.
    /// None of those names both tools.
    fn unpierced_sew_line(&self, actor: ActorId) -> String {
        let held = self.loose_ids(actor, Material::Awl);
        let (tool, cue) = if let Some(phrase) = self.awl_tool_phrase(&held) {
            (phrase, String::new())
        } else {
            let cell = self.actor(actor).map(|person| person.cell).unwrap_or(0);
            let ground = self.ground_ids(cell, Material::Awl);
            if let Some(phrase) = self.awl_tool_phrase(&ground) {
                (phrase, String::new())
            } else if let Some((phrase, cue)) = self.awl_neighbor(actor) {
                (phrase, cue)
            } else {
                ("a copper awl or a bone awl", String::new())
            }
        };
        let kind = if self.carrying_otter_hide(actor) {
            "otter"
        } else if self.carrying_fox_hide(actor) {
            "fox"
        } else if self.carrying_wolf_hide(actor) {
            "wolf"
        } else if self.carrying_hare_hide(actor) {
            "hare"
        } else if self.carrying_beaver_hide(actor) {
            "beaver"
        } else {
            "deer"
        };
        format!("The {kind} hide is not pierced. Pierce it with {tool}.{cue}")
    }

    fn hide_missing<'a>(&self, actor: ActorId, deer: &'a str) -> &'a str {
        self.hide_line(
            actor,
            deer,
            "You are not carrying a wolf hide.",
            "You are not carrying an otter hide.",
            "You are not carrying a fox hide.",
        )
    }
}

fn beaver_instead(deer: &str) -> &'static str {
    match deer {
        "You need a copper awl and a deer hide." => "You need a copper awl and a beaver hide.",
        "The deer hide is already pierced." => "The beaver hide is already pierced.",
        "The deer hide is already a cloak." => "The beaver hide is already a cloak.",
        "The deer hide is not pierced." => "The beaver hide is not pierced.",
        "The deer hide is not pierced. Pierce it with a copper awl or a bone awl." => {
            "The beaver hide is not pierced. Pierce it with a copper awl or a bone awl."
        }
        "The deer hide is not scraped." => "The beaver hide is not scraped.",
        "The deer hide is already scraped." => "The beaver hide is already scraped.",
        "You are not carrying a deer hide." => "You are not carrying a beaver hide.",
        "You are not wearing a deer hide." => "You are not wearing a beaver hide.",
        _ => "You are not carrying a beaver hide.",
    }
}

fn hare_instead(deer: &str) -> &'static str {
    match deer {
        "You need a copper awl and a deer hide." => "You need a copper awl and a hare hide.",
        "The deer hide is already pierced." => "The hare hide is already pierced.",
        "The deer hide is already a cloak." => "The hare hide is already a cloak.",
        "The deer hide is not pierced." => "The hare hide is not pierced.",
        "The deer hide is not pierced. Pierce it with a copper awl or a bone awl." => {
            "The hare hide is not pierced. Pierce it with a copper awl or a bone awl."
        }
        "The deer hide is not scraped." => "The hare hide is not scraped.",
        "The deer hide is already scraped." => "The hare hide is already scraped.",
        "You are not carrying a deer hide." => "You are not carrying a hare hide.",
        "You are not wearing a deer hide." => "You are not wearing a hare hide.",
        _ => "You are not carrying a hare hide.",
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

#[derive(Clone, Copy)]
enum MossSpot {
    Arms,
    Pile,
    Ground,
}

fn moss_fuel(spot: MossSpot, branch_arms: bool) -> &'static str {
    match (spot, branch_arms) {
        (MossSpot::Arms, true) => "the moss and the branch from the arms",
        (MossSpot::Arms, false) => {
            "the moss from the arms and a dry branch from the pile at the feet"
        }
        (MossSpot::Pile, true) => "moss from the pile at the feet and the branch from the arms",
        (MossSpot::Pile, false) => "moss and a dry branch from the pile at the feet",
        (MossSpot::Ground, true) => "moss from the ground and the branch from the arms",
        (MossSpot::Ground, false) => {
            "moss from the ground and a dry branch from the pile at the feet"
        }
    }
}

fn fluff_light_line(who: Option<&str>, branch_arms: bool, brand: bool) -> String {
    let fuel = if branch_arms {
        "the cattail fluff and the branch from the arms"
    } else {
        "the cattail fluff from the arms and a dry branch from the pile at the feet"
    };
    let rest = if brand {
        format!("a fire with the burning brand and {fuel}.")
    } else {
        format!("a fire with {fuel}.")
    };
    crate::model::spoken_act(who, "light", &rest)
}

fn moss_light_line(who: Option<&str>, spot: MossSpot, branch_arms: bool) -> String {
    crate::model::spoken_act(
        who,
        "light",
        &format!("a fire with {}.", moss_fuel(spot, branch_arms)),
    )
}

fn moss_brand_light_line(who: Option<&str>, spot: MossSpot, branch_arms: bool) -> String {
    crate::model::spoken_act(
        who,
        "light",
        &format!(
            "a fire with the burning brand and {}.",
            moss_fuel(spot, branch_arms)
        ),
    )
}

/// Charcoal stands in for the branch. Resin, dry moss, or bark is still the tinder.
fn charcoal_light_line(
    who: Option<&str>,
    resin: bool,
    moss_arms: Option<MossSpot>,
    bark_arms: bool,
    fluff: bool,
    brand: bool,
    charcoal_arms: bool,
) -> String {
    let lump = if charcoal_arms {
        "a lump of charcoal from the arms"
    } else {
        "a lump of charcoal from the ground"
    };
    let tinder = if resin {
        "the pine resin"
    } else if fluff {
        "the cattail fluff"
    } else if let Some(spot) = moss_arms {
        match spot {
            MossSpot::Arms => "the moss",
            MossSpot::Pile => "moss from the pile at the feet",
            MossSpot::Ground => "moss from the ground",
        }
    } else if bark_arms {
        "the bark"
    } else {
        "bark from the pile at the feet"
    };
    let fuel = if brand {
        format!("the burning brand and {tinder} and {lump}")
    } else {
        format!("{tinder} and {lump}")
    };
    crate::model::spoken_act(who, "light", &format!("a fire with {fuel}."))
}

/// Dry bark one step off. The soaked sheet in the arms is not named.
fn off_bark_line(who: Option<&str>, place: &str, bark_pile: bool, fuel: FuelSpot) -> String {
    let rest = match (bark_pile, fuel) {
        (true, FuelSpot::Charcoal) => format!(
            "a fire with bark from the pile {place} and a lump of charcoal from the ground."
        ),
        (false, FuelSpot::Charcoal) => format!(
            "a fire with a dry sheet of bark from the ground {place} and a lump of charcoal from the ground."
        ),
        (true, FuelSpot::BranchPile) => {
            format!("a fire with bark and a dry branch from the pile {place}.")
        }
        (true, FuelSpot::BranchGround) => format!(
            "a fire with bark from the pile {place} and a dry branch from the ground."
        ),
        (false, FuelSpot::BranchPile) => format!(
            "a fire with a dry sheet of bark from the ground {place} and a dry branch from the pile {place}."
        ),
        (false, FuelSpot::BranchGround) => format!(
            "a fire with a dry sheet of bark and a dry branch from the ground {place}."
        ),
    };
    crate::model::spoken_act(who, "light", &rest)
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
