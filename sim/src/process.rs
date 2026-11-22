//! Rain and unfinished work. Both run from the simulation-time queue,
//! including when nobody has sent a command.

use crate::model::{
    cell_id, cell_xy, chebyshev, ActorId, CellId, ConstructionId, Dir, EventId, FireId, ItemId,
    Material, Region, Role, Scope, Wake, Work, WorkId, WorkKind, World, BEAVER_BANK, BEAVER_HOME,
    DEER_ROUTE, FOX_ROUTE, HARE_HOME, OTTER_ROUTE,
};

impl World {
    pub(crate) fn fire_due(&mut self) {
        let now = self.time;
        let mut due: Vec<_> = self
            .schedule
            .iter()
            .filter(|entry| entry.at <= now)
            .cloned()
            .collect();
        due.sort_by_key(|entry| (entry.at, entry.seq));
        self.schedule.retain(|entry| entry.at > now);
        for entry in due {
            match entry.wake {
                Wake::Rain => self.apply_rain(),
                Wake::Thirst => self.apply_thirst(),
                Wake::Hunger => self.apply_hunger(),
                Wake::Energy => self.apply_energy(),
                Wake::Weather => self.apply_weather(),
                Wake::Illness => self.apply_illness(),
                Wake::Work(id) => self.complete_work(id),
                Wake::Fire(id) => self.burn_down(id),
                Wake::Deer => self.apply_deer(),
                Wake::Otter => self.apply_otter(),
                Wake::Beaver => self.apply_beaver(),
                Wake::Fox => self.apply_fox(),
                Wake::Season => self.apply_season(),
                Wake::Rot => self.apply_rot(),
                Wake::Harden(id) => self.harden_clay(id),
                Wake::HardenSpear(id) => self.harden_spear(id),
                Wake::Smelt(id) => self.finish_smelt(id),
                Wake::DryBranch(id) => self.dry_branch(id),
                Wake::BrandOut(id) => self.brand_out(id),
                Wake::Snare(cell) => self.spring_snare(cell),
                Wake::NetCatch(cell) => self.catch_in_net(cell),
                Wake::Weir(cell) => self.catch_in_weir(cell),
                Wake::Spoil(id) => self.spoil_food(id),
                Wake::CupLeak(id) => self.leak_cup(id),
                Wake::SaltReturn(cell) => self.return_salt(cell),
                Wake::HoneyReturn(cell) => self.return_honey(cell),
                Wake::EggReturn(cell) => self.return_eggs(cell),
                Wake::MossReturn(cell) => self.return_moss(cell),
                Wake::MintReturn(cell) => self.return_mint(cell),
                Wake::NeedleReturn(cell) => self.return_needles(cell),
                Wake::ResinReturn(cell) => self.return_resin(cell),
                Wake::WillowReturn(cell) => self.return_willow(cell),
                Wake::WillowBarkReturn(cell) => self.return_willow_bark(cell),
                Wake::NettleReturn(cell) => self.return_nettle(cell),
                Wake::CattailReturn(cell) => self.return_cattail(cell),
                Wake::FluffReturn(cell) => self.return_fluff(cell),
                Wake::WormReturn(cell) => self.return_worms(cell),
                Wake::BoneReturn(cell) => self.return_bone(cell),
                Wake::FlintReturn(cell) => self.return_flint(cell),
                Wake::OchreReturn(cell) => self.return_ochre(cell),
                Wake::ClayReturn(cell) => self.return_clay(cell),
                Wake::CopperReturn(cell) => self.return_copper(cell),
                Wake::IronReturn(cell) => self.return_iron(cell),
                Wake::ReedReturn(cell) => self.return_reeds(cell),
                Wake::BranchReturn(cell) => self.return_branches(cell),
                Wake::BarkReturn(cell) => self.return_bark(cell),
                Wake::BerryReturn(cell) => self.return_berries(cell),
                Wake::MushroomReturn(cell) => self.return_mushrooms(cell),
                Wake::DeerReturn => self.return_deer(),
                Wake::OtterReturn => self.return_otter(),
                Wake::FoxReturn => self.return_fox(),
                Wake::HareReturn => self.return_hare(),
                Wake::BeaverReturn => self.return_beaver(),
                Wake::Stings => self.apply_stings(),
                Wake::DryClothes(actor) => self.dry_clothes(actor),
                Wake::Charcoal(cell) => self.char_bank(cell),
            }
        }
    }

    /// Raw meat or a raw fish has waited out its ticks. It is not food.
    /// Salted meat, cooked meat, and cooked fish keep.
    /// Someone who cannot see the cell hears nothing.
    fn spoil_food(&mut self, id: ItemId) {
        let Some(item) = self.item(id).cloned() else {
            return;
        };
        if item.cooked || item.salted || item.smoked || item.spoiled {
            return;
        }
        if item.material != Material::Meat && item.material != Material::Fish {
            return;
        }
        if item.spoils_at != Some(self.time) {
            return;
        }
        if let Some(food) = self.items.get_mut(&id) {
            food.spoiled = true;
            food.spoils_at = None;
        }
        let text = if item.material == Material::Fish {
            "A raw fish spoils. It is not food."
        } else if item.small {
            "A smaller piece of raw meat spoils. It is not food."
        } else {
            "A portion of raw meat spoils. It is not food."
        };
        self.emit(
            text,
            Scope::Cell(item.cell),
            vec![],
            vec![item.cell],
            vec![id],
            vec![],
            None,
        );
    }

    /// A drink has waited out its ticks in an unsealed bark cup.
    /// The drink is gone. A sealed cup, a pot, or no cup at all keeps it.
    /// Someone who cannot see the cell hears nothing.
    fn leak_cup(&mut self, id: ItemId) {
        let Some(water) = self.item(id).cloned() else {
            return;
        };
        if water.material != Material::Water {
            return;
        }
        let Some(actor) = water.carrier else {
            return;
        };
        if !self.loose_ids(actor, Material::Pot).is_empty() {
            return;
        }
        let Some(cup) = self.loose_ids(actor, Material::BarkCup).first().copied() else {
            return;
        };
        if self.item(cup).is_some_and(|cup| cup.pitched) {
            return;
        }
        if let Some(bark) = self.items.get_mut(&cup) {
            bark.moisture = bark.moisture.max(self.tuning.soaked_at);
        }
        self.items.remove(&id);
        let cell = self
            .actor(actor)
            .map(|person| person.cell)
            .unwrap_or(water.cell);
        self.emit(
            "The drink soaks through the bark cup.",
            Scope::Cell(cell),
            vec![actor],
            vec![cell],
            vec![cup],
            vec![],
            None,
        );
    }

    /// The rock has waited out the salt ticks. One bone lies there again.
    /// Someone who cannot see the cell hears nothing.
    fn return_bone(&mut self, cell: CellId) {
        let Some(at) = self.bone_returns_at.get(&cell).copied() else {
            return;
        };
        if at != self.time {
            return;
        }
        self.bone_returns_at.remove(&cell);
        if self.pile_count(cell, Material::Bone) > 0
            || !self.ground_ids(cell, Material::Bone).is_empty()
        {
            return;
        }
        self.mint_item(Material::Bone, 0, cell, None, Role::Loose);
        self.emit(
            "A bone lies under the rock again.",
            Scope::Cell(cell),
            vec![],
            vec![cell],
            vec![],
            vec![],
            None,
        );
    }

    /// The salt lick has waited out its ticks. A crust lies on the slope again.
    /// Someone who cannot see the cell hears nothing.
    fn return_salt(&mut self, cell: CellId) {
        let Some(at) = self.salt_returns_at.get(&cell).copied() else {
            return;
        };
        if at != self.time {
            return;
        }
        self.salt_returns_at.remove(&cell);
        if self.pile_count(cell, Material::Salt) > 0
            || !self.ground_ids(cell, Material::Salt).is_empty()
        {
            return;
        }
        self.add_pile(cell, Material::Salt, 1, 0);
        let text = if self.salt_pans.contains(&cell) {
            "A salt crust forms on the still margin."
        } else {
            "A salt crust forms on the slope."
        };
        self.emit(
            text,
            Scope::Cell(cell),
            vec![],
            vec![cell],
            vec![],
            vec![],
            None,
        );
    }

    /// The bees' nest has waited out its ticks. Honey hangs in the tree again.
    /// Someone who cannot see the cell hears nothing.
    fn return_honey(&mut self, cell: CellId) {
        let Some(at) = self.honey_returns_at.get(&cell).copied() else {
            return;
        };
        if at != self.time {
            return;
        }
        self.honey_returns_at.remove(&cell);
        if self.pile_count(cell, Material::Honey) > 0
            || !self.ground_ids(cell, Material::Honey).is_empty()
        {
            return;
        }
        self.add_pile(cell, Material::Honey, 1, 0);
        self.emit(
            "The bees' nest fills again.",
            Scope::Cell(cell),
            vec![],
            vec![cell],
            vec![],
            vec![],
            None,
        );
    }

    /// The bird's nest has waited out its ticks. Eggs sit in the tree again.
    /// Someone who cannot see the cell hears nothing.
    fn return_eggs(&mut self, cell: CellId) {
        let Some(at) = self.egg_returns_at.get(&cell).copied() else {
            return;
        };
        if at != self.time {
            return;
        }
        self.egg_returns_at.remove(&cell);
        if self.pile_count(cell, Material::Egg) > 0
            || !self.ground_ids(cell, Material::Egg).is_empty()
        {
            return;
        }
        self.add_pile(cell, Material::Egg, 1, 0);
        self.emit(
            "The bird's nest fills again.",
            Scope::Cell(cell),
            vec![],
            vec![cell],
            vec![],
            vec![],
            None,
        );
    }

    /// The damp earth has waited out the same ticks as moss. A worm is there again.
    /// Someone who cannot see the cell hears nothing.
    fn return_worms(&mut self, cell: CellId) {
        let Some(at) = self.worm_returns_at.get(&cell).copied() else {
            return;
        };
        if at != self.time {
            return;
        }
        self.worm_returns_at.remove(&cell);
        if self.pile_count(cell, Material::Worm) > 0
            || !self.ground_ids(cell, Material::Worm).is_empty()
        {
            return;
        }
        self.add_pile(cell, Material::Worm, 1, 0);
        self.emit(
            "Worms are in the damp earth again.",
            Scope::Cell(cell),
            vec![],
            vec![cell],
            vec![],
            vec![],
            None,
        );
    }

    /// The damp bank has waited out its ticks. Moss grows there again.
    /// Someone who cannot see the cell hears nothing.
    fn return_moss(&mut self, cell: CellId) {
        let Some(at) = self.moss_returns_at.get(&cell).copied() else {
            return;
        };
        if at != self.time {
            return;
        }
        self.moss_returns_at.remove(&cell);
        if self.pile_count(cell, Material::Moss) > 0
            || !self.ground_ids(cell, Material::Moss).is_empty()
        {
            return;
        }
        self.add_pile(cell, Material::Moss, 1, 0);
        self.emit(
            "Moss grows on the damp bank again.",
            Scope::Cell(cell),
            vec![],
            vec![cell],
            vec![],
            vec![],
            None,
        );
    }

    /// The wet bank has waited out the same ticks as moss. One handful lies there again.
    fn return_mint(&mut self, cell: CellId) {
        let Some(at) = self.mint_returns_at.get(&cell).copied() else {
            return;
        };
        if at != self.time {
            return;
        }
        self.mint_returns_at.remove(&cell);
        if self.pile_count(cell, Material::Mint) > 0
            || !self.ground_ids(cell, Material::Mint).is_empty()
        {
            return;
        }
        self.add_pile(cell, Material::Mint, 1, 0);
        self.emit(
            "Wild mint grows on the wet bank again.",
            Scope::Cell(cell),
            vec![],
            vec![cell],
            vec![],
            vec![],
            None,
        );
    }

    /// The pine has waited out the same ticks as moss. One handful of needles lies there again.
    /// Pitch is a different pile.
    fn return_needles(&mut self, cell: CellId) {
        let Some(at) = self.needle_returns_at.get(&cell).copied() else {
            return;
        };
        if at != self.time {
            return;
        }
        self.needle_returns_at.remove(&cell);
        if self.pile_count(cell, Material::Needle) > 0
            || !self.ground_ids(cell, Material::Needle).is_empty()
        {
            return;
        }
        self.add_pile(cell, Material::Needle, 1, 0);
        self.emit(
            "Pine needles lie under the pine again.",
            Scope::Cell(cell),
            vec![],
            vec![cell],
            vec![],
            vec![],
            None,
        );
    }

    /// The pine has waited out its ticks. Two lumps of pitch lie there again.
    /// Someone who cannot see the pine hears nothing.
    fn return_resin(&mut self, cell: CellId) {
        let Some(at) = self.resin_returns_at.get(&cell).copied() else {
            return;
        };
        if at != self.time {
            return;
        }
        self.resin_returns_at.remove(&cell);
        if self.pile_count(cell, Material::Resin) > 0
            || !self.ground_ids(cell, Material::Resin).is_empty()
        {
            return;
        }
        self.add_pile(cell, Material::Resin, 2, 0);
        self.emit(
            "Pine resin lies under the pine again.",
            Scope::Cell(cell),
            vec![],
            vec![cell],
            vec![],
            vec![],
            None,
        );
    }

    /// The willow has waited out its ticks. Two bundles of cordage hang there again.
    /// Someone who cannot see the willow hears nothing.
    fn return_willow(&mut self, cell: CellId) {
        let Some(at) = self.willow_returns_at.get(&cell).copied() else {
            return;
        };
        if at != self.time {
            return;
        }
        self.willow_returns_at.remove(&cell);
        if self.pile_count(cell, Material::Willow) > 0
            || !self.ground_ids(cell, Material::Willow).is_empty()
        {
            return;
        }
        self.add_pile(cell, Material::Willow, 2, 0);
        self.emit(
            "Willow cordage hangs on the willow again.",
            Scope::Cell(cell),
            vec![],
            vec![cell],
            vec![],
            vec![],
            None,
        );
    }

    /// The willow has waited out the same ticks as moss. One strip of bark hangs there again.
    /// Cordage is a different pile. Someone who cannot see the willow hears nothing.
    fn return_willow_bark(&mut self, cell: CellId) {
        let Some(at) = self.willow_bark_returns_at.get(&cell).copied() else {
            return;
        };
        if at != self.time {
            return;
        }
        self.willow_bark_returns_at.remove(&cell);
        if self.pile_count(cell, Material::WillowBark) > 0
            || !self.ground_ids(cell, Material::WillowBark).is_empty()
        {
            return;
        }
        self.add_pile(cell, Material::WillowBark, 1, 0);
        self.emit(
            "Willow bark grows on the willow again.",
            Scope::Cell(cell),
            vec![],
            vec![cell],
            vec![],
            vec![],
            None,
        );
    }

    /// The nettles have waited out their ticks. Two bundles of cordage grow there again.
    /// Someone who cannot see the stand hears nothing.
    fn return_nettle(&mut self, cell: CellId) {
        let Some(at) = self.nettle_returns_at.get(&cell).copied() else {
            return;
        };
        if at != self.time {
            return;
        }
        self.nettle_returns_at.remove(&cell);
        if self.pile_count(cell, Material::Nettle) > 0
            || !self.ground_ids(cell, Material::Nettle).is_empty()
        {
            return;
        }
        self.add_pile(cell, Material::Nettle, 2, 0);
        self.emit(
            "Nettles grow here again.",
            Scope::Cell(cell),
            vec![],
            vec![cell],
            vec![],
            vec![],
            None,
        );
    }

    /// The shallows have waited out their ticks. Cattails grow there again.
    /// Someone who cannot see the cell hears nothing.
    fn return_cattail(&mut self, cell: CellId) {
        let Some(at) = self.cattail_returns_at.get(&cell).copied() else {
            return;
        };
        if at != self.time {
            return;
        }
        self.cattail_returns_at.remove(&cell);
        if self.pile_count(cell, Material::Cattail) > 0
            || !self.ground_ids(cell, Material::Cattail).is_empty()
        {
            return;
        }
        if self.still_frozen(cell) {
            let next = self.time.saturating_add(1);
            self.cattail_returns_at.insert(cell, next);
            self.schedule(next, Wake::CattailReturn(cell));
            return;
        }
        self.add_pile(cell, Material::Cattail, 2, 0);
        let fluff_back = self.pile_count(cell, Material::Fluff) == 0
            && self.ground_ids(cell, Material::Fluff).is_empty()
            && !self.fluff_returns_at.contains_key(&cell);
        if fluff_back {
            self.add_pile(cell, Material::Fluff, 1, 0);
        }
        self.emit(
            "Cattails grow again in the shallows.",
            Scope::Cell(cell),
            vec![],
            vec![cell],
            vec![],
            vec![],
            None,
        );
        if fluff_back {
            self.emit(
                "Cattail fluff hangs on the cattails again.",
                Scope::Cell(cell),
                vec![],
                vec![cell],
                vec![],
                vec![],
                None,
            );
        }
    }

    /// The cattails have waited out the same ticks as the roots. Fluff hangs there again.
    /// Ice holds it. A bare stand holds it until the cattails are back.
    /// Someone who cannot see the shallows hears nothing.
    fn return_fluff(&mut self, cell: CellId) {
        let Some(at) = self.fluff_returns_at.get(&cell).copied() else {
            return;
        };
        if at != self.time {
            return;
        }
        self.fluff_returns_at.remove(&cell);
        let plants = self.pile_count(cell, Material::Cattail) > 0
            || !self.ground_ids(cell, Material::Cattail).is_empty();
        if self.still_frozen(cell) || !plants {
            let next = self.time.saturating_add(1);
            self.fluff_returns_at.insert(cell, next);
            self.schedule(next, Wake::FluffReturn(cell));
            return;
        }
        if self.pile_count(cell, Material::Fluff) > 0
            || !self.ground_ids(cell, Material::Fluff).is_empty()
        {
            return;
        }
        self.add_pile(cell, Material::Fluff, 1, 0);
        self.emit(
            "Cattail fluff hangs on the cattails again.",
            Scope::Cell(cell),
            vec![],
            vec![cell],
            vec![],
            vec![],
            None,
        );
    }

    /// The rock has waited out its ticks. A pile of dry branches lies there again.
    /// Someone who cannot see the rock hears nothing.
    fn return_branches(&mut self, cell: CellId) {
        let Some(at) = self.branch_returns_at.get(&cell).copied() else {
            return;
        };
        if at != self.time {
            return;
        }
        self.branch_returns_at.remove(&cell);
        if self.pile_count(cell, Material::Branch) > 0
            || !self.ground_ids(cell, Material::Branch).is_empty()
        {
            return;
        }
        self.add_pile(cell, Material::Branch, 4, 0);
        self.emit(
            "Branches lie under the rock again.",
            Scope::Cell(cell),
            vec![],
            vec![cell],
            vec![],
            vec![],
            None,
        );
    }

    /// The rock has waited out its ticks. The same pile of bark lies there again.
    /// Someone who cannot see the rock hears nothing.
    fn return_bark(&mut self, cell: CellId) {
        let Some(at) = self.bark_returns_at.get(&cell).copied() else {
            return;
        };
        if at != self.time {
            return;
        }
        self.bark_returns_at.remove(&cell);
        let Some(count) = self.bark_beds.get(&cell).copied() else {
            return;
        };
        if self.pile_count(cell, Material::Bark) > 0
            || !self.ground_ids(cell, Material::Bark).is_empty()
        {
            return;
        }
        self.add_pile(cell, Material::Bark, count, 0);
        self.emit(
            "Bark lies under the rock again.",
            Scope::Cell(cell),
            vec![],
            vec![cell],
            vec![],
            vec![],
            None,
        );
    }

    /// The patch has waited out its ticks. The same pile lies there again.
    /// Someone who cannot see the cell hears nothing.
    fn return_berries(&mut self, cell: CellId) {
        let Some(at) = self.berry_returns_at.get(&cell).copied() else {
            return;
        };
        if at != self.time {
            return;
        }
        self.berry_returns_at.remove(&cell);
        let Some(patch) = self.berry_patches.get(&cell).copied() else {
            return;
        };
        if self.pile_count(cell, patch.material) > 0 {
            return;
        }
        self.add_pile(cell, patch.material, patch.count, 0);
        self.spent_berries.remove(&cell);
        self.deer_ate_berries.remove(&cell);
        let text = if patch.material == Material::BitterBerry {
            "Dark berries lie here again."
        } else if patch.material == Material::PaleBerry {
            "Pale berries lie here again."
        } else {
            "Berries lie here again."
        };
        self.emit(
            text,
            Scope::Cell(cell),
            vec![],
            vec![cell],
            vec![],
            vec![],
            None,
        );
    }

    /// The patch has waited out its ticks. The same mushrooms lie there again.
    /// Someone who cannot see the cell hears nothing.
    fn return_mushrooms(&mut self, cell: CellId) {
        let Some(at) = self.mushroom_returns_at.get(&cell).copied() else {
            return;
        };
        if at != self.time {
            return;
        }
        self.mushroom_returns_at.remove(&cell);
        let Some(count) = self.mushroom_patches.get(&cell).copied() else {
            return;
        };
        if self.pile_count(cell, Material::Mushroom) > 0 {
            return;
        }
        self.add_pile(cell, Material::Mushroom, count, 0);
        self.emit(
            "Mushrooms lie here again.",
            Scope::Cell(cell),
            vec![],
            vec![cell],
            vec![],
            vec![],
            None,
        );
    }

    /// The slope has waited out its ticks. A flint nodule lies there again.
    /// Someone who cannot see the cell hears nothing.
    fn return_flint(&mut self, cell: CellId) {
        let Some(at) = self.flint_returns_at.get(&cell).copied() else {
            return;
        };
        if at != self.time {
            return;
        }
        self.flint_returns_at.remove(&cell);
        if self.pile_count(cell, Material::Flint) > 0
            || !self.ground_ids(cell, Material::Flint).is_empty()
        {
            return;
        }
        self.add_pile(cell, Material::Flint, 1, 0);
        self.emit(
            "A flint nodule lies on the slope again.",
            Scope::Cell(cell),
            vec![],
            vec![cell],
            vec![],
            vec![],
            None,
        );
    }

    /// The slope has waited out its ticks. A lump of ochre lies there again.
    /// Someone who cannot see the cell hears nothing.
    fn return_ochre(&mut self, cell: CellId) {
        let Some(at) = self.ochre_returns_at.get(&cell).copied() else {
            return;
        };
        if at != self.time {
            return;
        }
        self.ochre_returns_at.remove(&cell);
        if self.pile_count(cell, Material::Ochre) > 0
            || !self.ground_ids(cell, Material::Ochre).is_empty()
        {
            return;
        }
        self.add_pile(cell, Material::Ochre, 1, 0);
        self.emit(
            "A lump of ochre lies on the slope again.",
            Scope::Cell(cell),
            vec![],
            vec![cell],
            vec![],
            vec![],
            None,
        );
    }

    /// The bank has waited out its ticks. Clay lies there again.
    /// Someone who cannot see the cell hears nothing.
    fn return_clay(&mut self, cell: CellId) {
        let Some(at) = self.clay_returns_at.get(&cell).copied() else {
            return;
        };
        if at != self.time {
            return;
        }
        self.clay_returns_at.remove(&cell);
        let Some(count) = self.clay_beds.get(&cell).copied() else {
            return;
        };
        if self.clay_bed_holding(cell) {
            return;
        }
        self.add_pile(cell, Material::Clay, count, 0);
        self.emit(
            "Clay lies on the bank again.",
            Scope::Cell(cell),
            vec![],
            vec![cell],
            vec![],
            vec![],
            None,
        );
    }

    /// The brook has waited out its ticks. A lump of copper ore lies there again.
    /// Someone who cannot see the cell hears nothing.
    fn return_copper(&mut self, cell: CellId) {
        let Some(at) = self.copper_returns_at.get(&cell).copied() else {
            return;
        };
        if at != self.time {
            return;
        }
        self.copper_returns_at.remove(&cell);
        if self.copper_bed_holding(cell) {
            return;
        }
        self.add_pile(cell, Material::CopperOre, 1, 0);
        self.emit(
            "A lump of copper ore lies on the brook again.",
            Scope::Cell(cell),
            vec![],
            vec![cell],
            vec![],
            vec![],
            None,
        );
    }

    /// The slope has waited out its ticks. A lump of iron ore lies there again.
    /// Someone who cannot see the cell hears nothing.
    fn return_iron(&mut self, cell: CellId) {
        let Some(at) = self.iron_returns_at.get(&cell).copied() else {
            return;
        };
        if at != self.time {
            return;
        }
        self.iron_returns_at.remove(&cell);
        if self.iron_bed_holding(cell) {
            return;
        }
        self.add_pile(cell, Material::IronOre, 1, 0);
        self.emit(
            "A lump of iron ore lies on the slope again.",
            Scope::Cell(cell),
            vec![],
            vec![cell],
            vec![],
            vec![],
            None,
        );
    }

    /// The brook's edge has waited out its ticks. Reeds grow there again.
    /// Someone who cannot see the cell hears nothing.
    fn return_reeds(&mut self, cell: CellId) {
        let Some(at) = self.reed_returns_at.get(&cell).copied() else {
            return;
        };
        if at != self.time {
            return;
        }
        self.reed_returns_at.remove(&cell);
        let Some(count) = self.reed_beds.get(&cell).copied() else {
            return;
        };
        if self.reed_bed_holding(cell) {
            return;
        }
        self.add_pile(cell, Material::Reed, count, 0);
        self.emit(
            "Reeds grow at the brook's edge again.",
            Scope::Cell(cell),
            vec![],
            vec![cell],
            vec![],
            vec![],
            None,
        );
    }

    /// The valley has waited out its ticks. A deer stands on the trail again.
    /// The route step for this tick waits. Someone who cannot see that cell
    /// hears nothing.
    fn return_deer(&mut self) {
        let Some(at) = self.deer_returns_at else {
            return;
        };
        if at != self.time {
            return;
        }
        self.deer_returns_at = None;
        if self.deer.is_some() {
            return;
        }
        let (x, y) = DEER_ROUTE[0];
        let Some(cell) = cell_id(x, y) else {
            return;
        };
        let id = self.next_deer_id;
        self.next_deer_id = self.next_deer_id.saturating_add(1);
        self.deer = Some(crate::model::Deer {
            id,
            cell,
            route_at: 0,
            last_event: None,
            held: true,
        });
        self.emit(
            "A deer stands in the valley again.",
            Scope::Cell(cell),
            vec![],
            vec![cell],
            vec![],
            vec![],
            None,
        );
    }

    /// The same wait as the deer. An otter is in the brook again, at the
    /// start of its route, and it does not swim on this tick. Someone who
    /// cannot see that water hears nothing.
    fn return_otter(&mut self) {
        let Some(at) = self.otter_returns_at else {
            return;
        };
        if at != self.time {
            return;
        }
        self.otter_returns_at = None;
        if self.otter.is_some() {
            return;
        }
        let (x, y) = OTTER_ROUTE[0];
        let Some(cell) = cell_id(x, y) else {
            return;
        };
        self.otter = Some(crate::model::Otter {
            cell,
            route_at: 0,
            held: true,
        });
        self.emit(
            "An otter is in the brook again.",
            Scope::Cell(cell),
            vec![],
            vec![cell],
            vec![],
            vec![],
            None,
        );
    }

    /// The same wait as the otter. A fox is on the dry floor again, at the
    /// start of its route, and it does not trot on this tick. Someone who can
    /// see that cell reads the sentence. Someone out of sight but within
    /// hearing hears a fox that way. Past hearing, nothing.
    fn return_fox(&mut self) {
        let Some(at) = self.fox_returns_at else {
            return;
        };
        if at != self.time {
            return;
        }
        self.fox_returns_at = None;
        if self.fox.is_some() {
            return;
        }
        let (x, y) = FOX_ROUTE[0];
        let Some(cell) = cell_id(x, y) else {
            return;
        };
        self.fox = Some(crate::model::Fox {
            cell,
            route_at: 0,
            held: true,
        });
        self.emit(
            "A fox is on the valley floor again.",
            Scope::Cell(cell),
            vec![],
            vec![cell],
            vec![],
            vec![],
            None,
        );
    }

    /// The same wait as the fox. A hare is on the dry floor again, on its home
    /// cell. Someone who can see that cell reads the sentence. Someone out of
    /// sight but within hearing hears a hare that way. Past hearing, nothing.
    fn return_hare(&mut self) {
        let Some(at) = self.hare_returns_at else {
            return;
        };
        if at != self.time {
            return;
        }
        self.hare_returns_at = None;
        if self.hare.is_some() {
            return;
        }
        let (x, y) = HARE_HOME;
        let Some(cell) = cell_id(x, y) else {
            return;
        };
        self.hare = Some(crate::model::Hare { cell });
        self.emit(
            "A hare is on the valley floor again.",
            Scope::Cell(cell),
            vec![],
            vec![cell],
            vec![],
            vec![],
            None,
        );
    }

    /// The same wait as the hare. A beaver is in the south brook again, on its
    /// home cell, and it does not swim on this tick. Someone who can see that
    /// water reads the sentence. Someone out of sight but within hearing hears
    /// a beaver that way. Past hearing, nothing.
    fn return_beaver(&mut self) {
        let Some(at) = self.beaver_returns_at else {
            return;
        };
        if at != self.time {
            return;
        }
        self.beaver_returns_at = None;
        if self.beaver.is_some() {
            return;
        }
        let (x, y) = BEAVER_HOME;
        let Some(cell) = cell_id(x, y) else {
            return;
        };
        self.beaver = Some(crate::model::Beaver { cell, held: true });
        self.emit(
            "A beaver is in the brook again.",
            Scope::Cell(cell),
            vec![],
            vec![cell],
            vec![],
            vec![],
            None,
        );
    }

    /// The hide snare on this cell has waited out its ticks. It is gone.
    /// If the valley hare is on this cell or the next, that hare is caught
    /// and a hare carcass lies where the snare was. Otherwise one portion of
    /// raw meat lies there, and the line names small game. The live hare stays.
    fn spring_snare(&mut self, cell: CellId) {
        let Some(ready) = self.snares.get(&cell).copied() else {
            return;
        };
        if ready > self.time {
            return;
        }
        self.snares.remove(&cell);
        let nettle = self.snare_nettle.remove(&cell);
        let willow = self.snare_willow.remove(&cell);
        let valley = self
            .hare
            .as_ref()
            .is_some_and(|hare| chebyshev(hare.cell, cell) <= 1);
        let caught_id = if valley {
            self.hare = None;
            self.arm_hare_return();
            let carcass = self.mint_item(Material::Carcass, 0, cell, None, Role::Loose);
            if let Some(item) = self.items.get_mut(&carcass) {
                item.hare = true;
            }
            carcass
        } else {
            self.mint_item(Material::Meat, 0, cell, None, Role::Loose)
        };
        let caught = if valley && nettle {
            "The nettle snare catches a hare."
        } else if valley && willow {
            "The willow snare catches a hare."
        } else if valley {
            "The snare catches a hare."
        } else if nettle {
            "The nettle snare catches small game."
        } else if willow {
            "The willow snare catches small game."
        } else {
            "The snare catches small game."
        };
        self.emit(
            caught,
            Scope::Cell(cell),
            vec![],
            vec![cell],
            vec![caught_id],
            vec![],
            None,
        );
    }

    /// The fishing net in this brook has waited out its ticks. A raw fish lies
    /// there. The net stays set until someone gathers it. Someone who cannot
    /// see the cell hears nothing.
    fn catch_in_net(&mut self, cell: CellId) {
        let Some(ready) = self.net_catches_at.get(&cell).copied() else {
            return;
        };
        if ready > self.time {
            return;
        }
        self.net_catches_at.remove(&cell);
        if !self.set_nets.contains(&cell) || self.ground_ids(cell, Material::Net).is_empty() {
            self.set_nets.remove(&cell);
            return;
        }
        let fish = self.mint_item(Material::Fish, 0, cell, None, Role::Loose);
        self.emit(
            "The fishing net catches a fish.",
            Scope::Cell(cell),
            vec![],
            vec![cell],
            vec![fish],
            vec![],
            None,
        );
    }

    /// The reed weir in this brook has waited out its ticks. A raw fish lies
    /// there. The weir is gone. Someone who cannot see the cell hears nothing.
    /// The brook is not marked quiet.
    fn catch_in_weir(&mut self, cell: CellId) {
        let Some(ready) = self.weirs.get(&cell).copied() else {
            return;
        };
        if ready > self.time {
            return;
        }
        self.weirs.remove(&cell);
        let fish = self.mint_item(Material::Fish, 0, cell, None, Role::Loose);
        self.emit(
            "The reed weir catches a fish.",
            Scope::Cell(cell),
            vec![],
            vec![cell],
            vec![fish],
            vec![],
            None,
        );
    }

    /// One change of season. Open rain soaks wood twice as fast from now on.
    /// A roof and the rock overhang keep the gains they already had.
    /// A lump set by a fire becomes a bowl if that fire is still burning.
    /// Unfired clay stays clay. The bowl is not a pot, and it can boil water.
    fn harden_clay(&mut self, id: ItemId) {
        let Some(item) = self.item(id).cloned() else {
            return;
        };
        if item.material != Material::Clay || item.firing_ready_at != Some(self.time) {
            return;
        }
        if item.shaped {
            self.harden_pot(id, &item);
            return;
        }
        if item.carrier.is_some() || self.fire_at(item.cell).is_none() {
            if let Some(lying) = self.items.get_mut(&id) {
                lying.firing_ready_at = None;
            }
            if item.carrier.is_none() {
                self.emit(
                    "The clay is still soft. The fire is out.",
                    Scope::Cell(item.cell),
                    vec![],
                    vec![item.cell],
                    vec![id],
                    vec![],
                    None,
                );
            }
            return;
        }
        if let Some(lying) = self.items.get_mut(&id) {
            lying.material = Material::Bowl;
            lying.firing_ready_at = None;
        }
        self.emit(
            "The clay hardens into a bowl.",
            Scope::Cell(item.cell),
            vec![],
            vec![item.cell],
            vec![id],
            vec![],
            None,
        );
    }

    /// An unfired pot set by a fire becomes a pot if that fire is still burning.
    /// It can hold water, the same as a pot found in the valley. The fire stays lit.
    fn harden_pot(&mut self, id: ItemId, item: &crate::model::Item) {
        if item.carrier.is_some() || self.fire_at(item.cell).is_none() {
            if let Some(lying) = self.items.get_mut(&id) {
                lying.firing_ready_at = None;
            }
            if item.carrier.is_none() {
                self.emit(
                    "The clay pot is still soft. The fire is out.",
                    Scope::Cell(item.cell),
                    vec![],
                    vec![item.cell],
                    vec![id],
                    vec![],
                    None,
                );
            }
            return;
        }
        if let Some(lying) = self.items.get_mut(&id) {
            lying.material = Material::Pot;
            lying.shaped = false;
            lying.firing_ready_at = None;
        }
        self.emit(
            "The clay pot hardens. It can hold water.",
            Scope::Cell(item.cell),
            vec![],
            vec![item.cell],
            vec![id],
            vec![],
            None,
        );
    }

    /// A lump set by a fire becomes metal if that fire is still burning.
    /// Copper becomes a bead in a small fire, and in half that wait when the
    /// fire is a hot charcoal fire. Iron becomes a bloom only in a hot
    /// charcoal fire. The plain wait is clay's.
    fn finish_smelt(&mut self, id: ItemId) {
        let Some(item) = self.item(id).cloned() else {
            return;
        };
        if item.firing_ready_at != Some(self.time) {
            return;
        }
        let (from, into, cold) = match item.material {
            Material::CopperOre => (
                Material::Copper,
                "The copper ore smelts into a bead.",
                "The copper ore is still ore. The fire is out.",
            ),
            Material::IronOre => (
                Material::Iron,
                "The iron ore smelts into a bloom.",
                "The iron ore is still ore. The fire is out.",
            ),
            _ => return,
        };
        let hot_enough = item.material != Material::IronOre
            || self.fire_at(item.cell).is_some_and(|fire| fire.charcoal);
        if item.carrier.is_some() || self.fire_at(item.cell).is_none() || !hot_enough {
            if let Some(lying) = self.items.get_mut(&id) {
                lying.firing_ready_at = None;
            }
            if item.carrier.is_none() {
                let text = if self.fire_at(item.cell).is_some() {
                    "The iron ore is still ore. It needs a hot charcoal fire."
                } else {
                    cold
                };
                self.emit(
                    text,
                    Scope::Cell(item.cell),
                    vec![],
                    vec![item.cell],
                    vec![id],
                    vec![],
                    None,
                );
            }
            return;
        }
        if let Some(lying) = self.items.get_mut(&id) {
            lying.material = from;
            lying.firing_ready_at = None;
        }
        self.emit(
            into,
            Scope::Cell(item.cell),
            vec![],
            vec![item.cell],
            vec![id],
            vec![],
            None,
        );
    }

    /// A branch set to dry stays wet until the fire has had its wait.
    /// Walking off the fire, or the fire going out, clears that wait.
    pub(crate) fn tend_drying(&mut self) {
        let ids: Vec<ItemId> = self
            .items
            .values()
            .filter(|item| item.material == Material::Branch && item.firing_ready_at.is_some())
            .map(|item| item.id)
            .collect();
        for id in ids {
            let Some(item) = self.item(id).cloned() else {
                continue;
            };
            let cell = if let Some(carrier) = item.carrier {
                self.actor(carrier)
                    .map(|person| person.cell)
                    .unwrap_or(item.cell)
            } else {
                item.cell
            };
            if self.fire_at(cell).is_none() {
                if let Some(branch) = self.items.get_mut(&id) {
                    branch.firing_ready_at = None;
                }
            }
        }
    }

    fn dry_branch(&mut self, id: ItemId) {
        let Some(item) = self.item(id).cloned() else {
            return;
        };
        if item.material != Material::Branch || item.firing_ready_at != Some(self.time) {
            return;
        }
        let cell = if let Some(carrier) = item.carrier {
            self.actor(carrier)
                .map(|person| person.cell)
                .unwrap_or(item.cell)
        } else {
            item.cell
        };
        if self.fire_at(cell).is_none() {
            if let Some(branch) = self.items.get_mut(&id) {
                branch.firing_ready_at = None;
            }
            return;
        }
        if let Some(branch) = self.items.get_mut(&id) {
            branch.moisture = 0;
            branch.firing_ready_at = None;
        }
        let (text, affected) = if let Some(carrier) = item.carrier {
            let name = self.name_of(carrier);
            (
                format!("{name} dries the branch at the fire."),
                vec![carrier],
            )
        } else {
            ("The branch dries at the fire.".into(), vec![])
        };
        self.emit(
            text,
            Scope::Cell(cell),
            affected,
            vec![cell],
            vec![id],
            vec![],
            None,
        );
    }

    /// The wait at the fire is over. The clothes are dry. Someone who cannot
    /// see that person hears nothing. Leaving the fire earlier cleared the wait.
    fn dry_clothes(&mut self, actor: ActorId) {
        let Some(at) = self.actor(actor).and_then(|person| person.clothes_dry_at) else {
            return;
        };
        if at != self.time {
            return;
        }
        let cell = self.actor(actor).map(|person| person.cell).unwrap_or(0);
        if self.fire_at(cell).is_none() {
            if let Some(person) = self.actors.get_mut(&actor) {
                person.clothes_dry_at = None;
            }
            return;
        }
        if let Some(person) = self.actors.get_mut(&actor) {
            person.wetness = 0;
            person.clothes_dry_at = None;
        }
        let name = self.name_of(actor);
        self.emit(
            format!("{name}'s clothes dry at the fire."),
            Scope::Cell(cell),
            vec![actor],
            vec![cell],
            vec![],
            vec![],
            None,
        );
    }

    /// The clock has just become the first tick of a night, the same tick
    /// whose sky says night has fallen. The sound rises at the northwest
    /// corner, and a wolf stands on that ground for a short wait. Someone
    /// within sight hears it without a direction. Someone farther off hears
    /// the direction from where they stand. Any other tick, including the
    /// rest of the night and the whole day, stays quiet.
    pub(crate) fn howl_on_nightfall(&mut self) {
        let cycle = self.tuning.night_len.saturating_add(self.tuning.day_len);
        if cycle == 0 || self.time < self.tuning.night_at {
            return;
        }
        if (self.time - self.tuning.night_at) % cycle != 0 {
            return;
        }
        let cell = cell_id(0, 0).unwrap_or(0);
        self.mark_wolf_tracks(cell);
        self.wolf_cell = Some(cell);
        self.wolf_until = self.time.saturating_add(self.tuning.wolf_ticks);
        self.scare_wolf(None);
        self.emit(
            "A wolf howls.",
            Scope::Valley,
            Vec::new(),
            vec![cell],
            Vec::new(),
            Vec::new(),
            None,
        );
    }

    /// The still pool has just frozen. Someone who can see it already has
    /// the ice in look. Someone out of sight but within hearing gets the
    /// crack. The day thaw, and every tick the ice simply holds, is silent.
    pub(crate) fn crack_on_freeze(&mut self) {
        if self.time == 0 {
            return;
        }
        let previous = self.time - 1;
        let cells: Vec<CellId> = self
            .cells
            .iter()
            .filter(|cell| self.still_frozen(cell.id) && !self.still_cold_at(cell.id, previous))
            .map(|cell| cell.id)
            .collect();
        for cell in cells {
            self.emit(
                "Ice forms on the still water.",
                Scope::Cell(cell),
                Vec::new(),
                vec![cell],
                Vec::new(),
                Vec::new(),
                None,
            );
        }
    }

    /// Still water would be frozen at `time` if no fire were holding it open.
    fn still_cold_at(&self, cell: CellId, time: u64) -> bool {
        time >= self.tuning.chill_at
            && self.cell(cell).still
            && self.cell(cell).region != Region::Brook
            && !self.morning_at(time)
    }

    fn morning_at(&self, time: u64) -> bool {
        if time < self.tuning.night_at {
            return false;
        }
        let cycle = self.tuning.night_len.saturating_add(self.tuning.day_len);
        if cycle == 0 {
            return false;
        }
        (time - self.tuning.night_at) % cycle >= self.tuning.night_len
    }

    /// Standing on a burning fire dries clothes the way it warms the body.
    /// One step of `fire_warmth` each tick. A tick that stays in the same
    /// word is silent. Crossing into damp, and then into dry, each say it
    /// once, and only to that person. The last of the damp does not stop
    /// on the brook word. `dry clothes` still owns its own wait.
    pub(crate) fn ease_clothes_at_fire(&mut self) {
        let step = self.tuning.fire_warmth;
        if step == 0 {
            return;
        }
        let damp_at = self.tuning.damp_at;
        let ids: Vec<ActorId> = self.actors.keys().copied().collect();
        for id in ids {
            let Some(actor) = self.actor(id) else {
                continue;
            };
            if actor.clothes_dry_at.is_some() || actor.wetness == 0 {
                continue;
            }
            let cell = actor.cell;
            if self.fire_at(cell).is_none() {
                continue;
            }
            let before = actor.wetness;
            let mut after = before.saturating_sub(step);
            if before >= damp_at && after < damp_at {
                after = 0;
            }
            if after == before {
                continue;
            }
            let word_before = self.clothes_word(before);
            let word_after = self.clothes_word(after);
            if let Some(person) = self.actors.get_mut(&id) {
                person.wetness = after;
            }
            if word_before == word_after {
                continue;
            }
            let text = if after == 0 {
                "Your clothes are dry.".to_string()
            } else {
                format!("Your clothes are {word_after}.")
            };
            self.emit(
                text,
                Scope::Personal(id),
                vec![id],
                vec![cell],
                vec![],
                vec![],
                None,
            );
        }
    }

    /// A drying wait ends when the person steps off the fire. The clothes
    /// stay as wet as they were. Standing still at the fire lets the wait finish.
    pub(crate) fn tend_clothes(&mut self) {
        let actors: Vec<ActorId> = self.actors.keys().copied().collect();
        for actor in actors {
            let Some(person) = self.actor(actor) else {
                continue;
            };
            if person.clothes_dry_at.is_some() && self.fire_at(person.cell).is_none() {
                if let Some(person) = self.actors.get_mut(&actor) {
                    person.clothes_dry_at = None;
                }
            }
        }
    }

    /// A carried spear at a burning fire starts its wait. Stepping off, or
    /// putting the spear down, clears that wait. The deer is not involved.
    pub(crate) fn arm_spears(&mut self) {
        let actors: Vec<_> = self.actors.keys().copied().collect();
        for actor in actors {
            let Some(cell) = self.actor(actor).map(|person| person.cell) else {
                continue;
            };
            let at_fire = self.fire_at(cell).is_some();
            for id in self.loose_ids(actor, Material::Spear) {
                let Some(item) = self.item(id).cloned() else {
                    continue;
                };
                if item.hardened {
                    continue;
                }
                if item.wooden_tip {
                    if !at_fire && item.firing_ready_at.is_some() {
                        if let Some(spear) = self.items.get_mut(&id) {
                            spear.firing_ready_at = None;
                        }
                    }
                    continue;
                }
                if at_fire {
                    if item.firing_ready_at.is_none() {
                        let ready = self.time.saturating_add(self.tuning.spear_ticks);
                        if let Some(spear) = self.items.get_mut(&id) {
                            spear.firing_ready_at = Some(ready);
                        }
                        self.schedule(ready, Wake::HardenSpear(id));
                    }
                } else if item.firing_ready_at.is_some() {
                    if let Some(spear) = self.items.get_mut(&id) {
                        spear.firing_ready_at = None;
                    }
                }
            }
        }
    }

    fn harden_spear(&mut self, id: ItemId) {
        let Some(item) = self.item(id).cloned() else {
            return;
        };
        if item.material != Material::Spear
            || item.hardened
            || item.firing_ready_at != Some(self.time)
        {
            return;
        }
        let Some(carrier) = item.carrier else {
            if let Some(spear) = self.items.get_mut(&id) {
                spear.firing_ready_at = None;
            }
            return;
        };
        let cell = self
            .actor(carrier)
            .map(|person| person.cell)
            .unwrap_or(item.cell);
        if self.fire_at(cell).is_none() {
            if let Some(spear) = self.items.get_mut(&id) {
                spear.firing_ready_at = None;
            }
            return;
        }
        if let Some(spear) = self.items.get_mut(&id) {
            spear.hardened = true;
            spear.firing_ready_at = None;
        }
        let name = self.name_of(carrier);
        let text = if item.flint_tip {
            crate::model::spoken_act(
                Some(name.as_str()),
                "harden",
                "the flint tip of the spear in the fire.",
            )
        } else if item.wooden_tip {
            format!("The wooden point of {name}'s spear hardens.")
        } else {
            format!("The point of {name}'s spear hardens.")
        };
        self.emit(
            text,
            Scope::Cell(cell),
            vec![carrier],
            vec![cell],
            vec![id],
            vec![],
            None,
        );
    }

    fn apply_season(&mut self) {
        if self.wet_season {
            return;
        }
        self.wet_season = true;
        if !self.raining {
            return;
        }
        for cell in self.open_wood_cells() {
            self.emit(
                "The rain is heavier this season.",
                Scope::Cell(cell),
                vec![],
                vec![cell],
                vec![],
                vec![],
                None,
            );
        }
    }

    /// One loose branch on open ground is gone. Shelter (exposure under
    /// open sky) leaves a pile alone. A branch in a frame is not loose.
    fn apply_rot(&mut self) {
        let interval = self.tuning.rot_ticks.max(1);
        self.schedule(self.time + interval, Wake::Rot);
        let mut piles: Vec<CellId> = self
            .piles
            .iter()
            .filter(|pile| {
                pile.material == Material::Branch
                    && pile.count > 0
                    && self.exposure_millis(pile.cell) >= 1000
            })
            .map(|pile| pile.cell)
            .collect();
        piles.sort_unstable();
        piles.dedup();
        for cell in piles {
            let last = self.pile_count(cell, Material::Branch) == 1;
            if self.take_from_pile(cell, Material::Branch, 1).is_some() {
                let text = if last {
                    "The last branch rots away in the open. You hear a branch break."
                } else {
                    "A branch rots away in the open. You hear branches breaking."
                };
                self.emit(
                    text,
                    Scope::Cell(cell),
                    vec![],
                    vec![cell],
                    vec![],
                    vec![],
                    None,
                );
            }
        }
        let mut loose: Vec<(CellId, ItemId)> = self
            .items
            .values()
            .filter(|item| {
                item.material == Material::Branch
                    && item.role == Role::Loose
                    && item.carrier.is_none()
                    && self.exposure_millis(item.cell) >= 1000
            })
            .map(|item| (item.cell, item.id))
            .collect();
        loose.sort_unstable();
        let mut seen = Vec::new();
        for (cell, id) in loose {
            if seen.contains(&cell) {
                continue;
            }
            seen.push(cell);
            self.items.remove(&id);
            self.emit(
                "A branch rots away in the open. You hear branches breaking.",
                Scope::Cell(cell),
                vec![],
                vec![cell],
                vec![id],
                vec![],
                None,
            );
        }
    }

    fn open_wood_cells(&self) -> Vec<CellId> {
        let mut cells = Vec::new();
        for pile in &self.piles {
            if pile.count == 0 || self.exposure_millis(pile.cell) < 1000 {
                continue;
            }
            if matches!(
                pile.material,
                Material::Branch | Material::Bark | Material::Log
            ) {
                cells.push(pile.cell);
            }
        }
        for item in self.items.values() {
            if item.role != Role::Loose || item.carrier.is_some() {
                continue;
            }
            if self.exposure_millis(item.cell) < 1000 {
                continue;
            }
            if matches!(
                item.material,
                Material::Branch | Material::Bark | Material::Log
            ) {
                cells.push(item.cell);
            }
        }
        cells.sort_unstable();
        cells.dedup();
        cells
    }

    fn season_gain(&self, gain: u32, open_sky: bool) -> u32 {
        if self.wet_season && open_sky {
            gain.saturating_mul(2).min(1000)
        } else {
            gain
        }
    }

    /// One step along the trail, unless berries are under its feet. Eating
    /// takes one handful from the pile and does not mint an item. The cell
    /// it leaves keeps hoofprints.
    fn apply_deer(&mut self) {
        self.schedule(self.time + 1, Wake::Deer);
        let Some(deer) = self.deer.clone() else {
            return;
        };
        if deer.held {
            if let Some(deer) = self.deer.as_mut() {
                deer.held = false;
            }
            return;
        }
        if self.pile_count(deer.cell, Material::Berry) > 0 {
            let cell = deer.cell;
            let _ = self.take_from_pile(cell, Material::Berry, 1);
            let last = self.pile_count(cell, Material::Berry) == 0;
            if last {
                self.deer_ate_berries.insert(cell);
                if self.ground_ids(cell, Material::Berry).is_empty() {
                    self.mint_item(Material::Berry, 0, cell, None, Role::Loose);
                }
            }
            let leaves = last && !self.ground_ids(cell, Material::Berry).is_empty();
            let text = if leaves {
                "A deer eats from the pile and leaves a handful of berries."
            } else if last {
                "A deer eats the last handful of berries."
            } else {
                "A deer eats a handful of berries."
            };
            let id = self.emit(
                text,
                Scope::Cell(deer.cell),
                vec![],
                vec![deer.cell],
                vec![],
                deer.last_event.into_iter().collect(),
                None,
            );
            if let Some(deer) = self.deer.as_mut() {
                deer.last_event = Some(id);
            }
            return;
        }
        let next_at = (deer.route_at + 1) % DEER_ROUTE.len();
        let (nx, ny) = DEER_ROUTE[next_at];
        let Some(dest) = cell_id(nx, ny) else {
            return;
        };
        let dir = step_dir(deer.cell, dest);
        let origin = deer.cell;
        self.tracks.insert(origin);
        self.leave_fresh_tracks(origin);
        // The same hoofprints, written when the deer steps onto the floor
        // west of the wood, so they are there while the deer stands. A cell
        // that already has them, including the old trail, is not marked again.
        if cell_xy(origin).0 - 1 == cell_xy(dest).0
            && cell_xy(origin).1 == cell_xy(dest).1
            && self.pile_count(origin, Material::Branch) > 0
            && !self.cell(origin).rock
        {
            self.tracks.insert(dest);
        }
        let snapped = self.pile_count(dest, Material::Branch) > 0
            && self.take_from_pile(dest, Material::Branch, 1).is_some();
        let text = if snapped {
            format!(
                "A deer steps {}. A branch snapped under the deer.",
                dir.word()
            )
        } else {
            format!("A deer steps {}.", dir.word())
        };
        let id = self.emit(
            text,
            Scope::Cell(dest),
            vec![],
            vec![origin, dest],
            vec![],
            deer.last_event.into_iter().collect(),
            None,
        );
        if let Some(deer) = self.deer.as_mut() {
            deer.cell = dest;
            deer.route_at = next_at;
            deer.last_event = Some(id);
        }
        self.note_brush(dest, snapped);
    }

    /// One step along the brook when nobody is beside the otter.
    /// A person who has just stepped close already sent it off.
    fn apply_otter(&mut self) {
        self.schedule(self.time + 4, Wake::Otter);
        let Some(otter) = self.otter.clone() else {
            return;
        };
        if otter.held {
            if let Some(otter) = self.otter.as_mut() {
                otter.held = false;
            }
            return;
        }
        if self
            .actors
            .values()
            .any(|person| chebyshev(person.cell, otter.cell) <= 1)
        {
            return;
        }
        let next_at = (otter.route_at + 1) % OTTER_ROUTE.len();
        let (nx, ny) = OTTER_ROUTE[next_at];
        let Some(dest) = cell_id(nx, ny) else {
            return;
        };
        if self.actors.values().any(|person| person.cell == dest) {
            return;
        }
        if self
            .beaver
            .as_ref()
            .is_some_and(|beaver| beaver.cell == dest)
        {
            self.yield_beaver(dest);
        }
        if self
            .beaver
            .as_ref()
            .is_some_and(|beaver| beaver.cell == dest)
        {
            return;
        }
        let dir = step_dir(otter.cell, dest);
        let origin = otter.cell;
        self.emit(
            format!("An otter swims {}.", dir.word()),
            Scope::Cell(dest),
            vec![],
            vec![origin, dest],
            vec![],
            vec![],
            None,
        );
        if let Some(otter) = self.otter.as_mut() {
            otter.cell = dest;
            otter.route_at = next_at;
        }
    }

    /// Someone stepped closer. The otter leaves along the brook, away from them.
    /// The step they took already has its own reply, so this splash is only the event.
    pub(crate) fn frighten_otter(&mut self, _origin: CellId, dest: CellId) {
        let Some(otter) = self.otter.clone() else {
            return;
        };
        if dest != otter.cell {
            return;
        }
        let Some((next_at, next)) = self.otter_away(otter.route_at, dest) else {
            return;
        };
        if next == otter.cell {
            return;
        }
        let dir = step_dir(otter.cell, next);
        let from = otter.cell;
        self.emit(
            format!("An otter splashes away to the {}.", dir.word()),
            Scope::Cell(from),
            vec![],
            vec![from],
            vec![],
            vec![],
            None,
        );
        if let Some(otter) = self.otter.as_mut() {
            otter.cell = next;
            otter.route_at = next_at;
        }
    }

    /// The route neighbor farther from the person. Their own cell is not a landing.
    pub(crate) fn otter_away(&self, route_at: usize, person: CellId) -> Option<(usize, CellId)> {
        let len = OTTER_ROUTE.len();
        let mut best: Option<(usize, CellId, i32)> = None;
        for step in [1isize, -1] {
            let index = (route_at as isize + step).rem_euclid(len as isize) as usize;
            let (x, y) = OTTER_ROUTE[index];
            let cell = cell_id(x, y)?;
            if cell == person {
                continue;
            }
            let away = chebyshev(person, cell);
            let replace = best.is_none_or(|(_, _, far)| away > far)
                || (best.is_some_and(|(_, _, far)| away == far) && step > 0);
            if replace {
                best = Some((index, cell, away));
            }
        }
        best.map(|(index, cell, _)| (index, cell))
    }

    /// Back to the south brook when that water is empty. A dive already moved
    /// it this tick, so the return waits. It does not step onto the otter.
    fn apply_beaver(&mut self) {
        self.schedule(self.time + 4, Wake::Beaver);
        let Some(beaver) = self.beaver.clone() else {
            return;
        };
        if beaver.held {
            if let Some(beaver) = self.beaver.as_mut() {
                beaver.held = false;
            }
            return;
        }
        let Some(home) = cell_id(BEAVER_HOME.0, BEAVER_HOME.1) else {
            return;
        };
        if beaver.cell == home {
            return;
        }
        if self.actors.values().any(|person| person.cell == home) {
            return;
        }
        if self.otter.as_ref().is_some_and(|otter| otter.cell == home) {
            return;
        }
        let dir = step_dir(beaver.cell, home);
        let origin = beaver.cell;
        self.emit(
            format!("A beaver swims {}.", dir.word()),
            Scope::Cell(home),
            vec![],
            vec![origin, home],
            vec![],
            vec![],
            None,
        );
        if let Some(beaver) = self.beaver.as_mut() {
            beaver.cell = home;
        }
    }

    /// Someone stepped onto the beaver. It dives to the bank south of its brook.
    /// The step they took already has its own reply, so this splash is only the event.
    pub(crate) fn frighten_beaver(&mut self, _origin: CellId, dest: CellId) {
        let Some(beaver) = self.beaver.clone() else {
            return;
        };
        if dest != beaver.cell {
            return;
        }
        self.yield_beaver(dest);
    }

    /// Off the cell something else is taking, onto the other of home and bank.
    /// No splash when that landing is taken by a person or the otter.
    fn yield_beaver(&mut self, taken: CellId) {
        let Some(beaver) = self.beaver.clone() else {
            return;
        };
        if beaver.cell != taken {
            return;
        }
        let Some(next) = self.beaver_aside(beaver.cell) else {
            return;
        };
        if next == beaver.cell {
            return;
        }
        if self.actors.values().any(|person| person.cell == next) {
            return;
        }
        if self.otter.as_ref().is_some_and(|otter| otter.cell == next) {
            return;
        }
        let dir = step_dir(beaver.cell, next);
        let from = beaver.cell;
        self.emit(
            format!("A beaver splashes away to the {}.", dir.word()),
            Scope::Cell(from),
            vec![],
            vec![from],
            vec![],
            vec![],
            None,
        );
        if let Some(beaver) = self.beaver.as_mut() {
            beaver.cell = next;
            beaver.held = true;
        }
    }

    /// The other of the brook cell and the bank south of it.
    pub(crate) fn beaver_aside(&self, from: CellId) -> Option<CellId> {
        let home = cell_id(BEAVER_HOME.0, BEAVER_HOME.1)?;
        let bank = cell_id(BEAVER_BANK.0, BEAVER_BANK.1)?;
        if from == home {
            Some(bank)
        } else {
            Some(home)
        }
    }

    /// One step on the dry floor when nobody is beside the fox.
    /// A person who has just stepped close already sent it off.
    fn apply_fox(&mut self) {
        self.schedule(self.time + 4, Wake::Fox);
        let Some(fox) = self.fox.clone() else {
            return;
        };
        if fox.held {
            if let Some(fox) = self.fox.as_mut() {
                fox.held = false;
            }
            return;
        }
        if self
            .actors
            .values()
            .any(|person| chebyshev(person.cell, fox.cell) <= 1)
        {
            return;
        }
        let next_at = (fox.route_at + 1) % FOX_ROUTE.len();
        let (nx, ny) = FOX_ROUTE[next_at];
        let Some(dest) = cell_id(nx, ny) else {
            return;
        };
        if self.actors.values().any(|person| person.cell == dest) {
            return;
        }
        let dir = step_dir(fox.cell, dest);
        let origin = fox.cell;
        self.emit(
            format!("A fox trots {}.", dir.word()),
            Scope::Cell(dest),
            vec![],
            vec![origin, dest],
            vec![],
            vec![],
            None,
        );
        if let Some(fox) = self.fox.as_mut() {
            fox.cell = dest;
            fox.route_at = next_at;
        }
    }

    /// Someone stepped onto the fox. It slips along the floor, away from them.
    /// The step they took already has its own reply, so this slip is only the event.
    pub(crate) fn frighten_fox(&mut self, _origin: CellId, dest: CellId) {
        let Some(fox) = self.fox.clone() else {
            return;
        };
        if dest != fox.cell {
            return;
        }
        let Some((next_at, next)) = self.fox_away(fox.route_at, dest) else {
            return;
        };
        if next == fox.cell {
            return;
        }
        let dir = step_dir(fox.cell, next);
        let from = fox.cell;
        self.emit(
            format!("A fox slips away to the {}.", dir.word()),
            Scope::Cell(from),
            vec![],
            vec![from],
            vec![],
            vec![],
            None,
        );
        if let Some(fox) = self.fox.as_mut() {
            fox.cell = next;
            fox.route_at = next_at;
        }
    }

    /// The route neighbor farther from the person. Their own cell is not a landing.
    pub(crate) fn fox_away(&self, route_at: usize, person: CellId) -> Option<(usize, CellId)> {
        let len = FOX_ROUTE.len();
        let mut best: Option<(usize, CellId, i32)> = None;
        for step in [1isize, -1] {
            let index = (route_at as isize + step).rem_euclid(len as isize) as usize;
            let (x, y) = FOX_ROUTE[index];
            let cell = cell_id(x, y)?;
            if cell == person {
                continue;
            }
            let away = chebyshev(person, cell);
            let replace = best.is_none_or(|(_, _, far)| away > far)
                || (best.is_some_and(|(_, _, far)| away == far) && step > 0);
            if replace {
                best = Some((index, cell, away));
            }
        }
        best.map(|(index, cell, _)| (index, cell))
    }

    /// Someone stepped close to the hare, or onto it. It bolts to a dry
    /// neighbor farther from them. The step already has its own reply, so
    /// this bolt is only the event.
    pub(crate) fn frighten_hare(&mut self, origin: CellId, dest: CellId) {
        let Some(hare) = self.hare.clone() else {
            return;
        };
        if chebyshev(dest, hare.cell) > 1 {
            return;
        }
        if chebyshev(origin, hare.cell) <= 1 && dest != hare.cell {
            return;
        }
        let away_from = if dest == hare.cell { origin } else { dest };
        let Some(next) = self.hare_away(hare.cell, dest, away_from) else {
            return;
        };
        if next == hare.cell {
            return;
        }
        let dir = step_dir(hare.cell, next);
        let from = hare.cell;
        self.emit(
            format!("A hare startles and bolts to the {}.", dir.word()),
            Scope::Cell(from),
            vec![],
            vec![from],
            vec![],
            vec![],
            None,
        );
        if let Some(hare) = self.hare.as_mut() {
            hare.cell = next;
        }
    }

    /// A whistle within ordinary hearing. The hare bolts the same way it does
    /// when someone steps close. The blow's reply does not carry this line.
    pub(crate) fn whistle_frighten_hare(&mut self, blower: CellId) {
        let Some(hare) = self.hare.clone() else {
            return;
        };
        if chebyshev(blower, hare.cell) > self.tuning.hearing {
            return;
        }
        let Some(next) = self.hare_away(hare.cell, blower, blower) else {
            return;
        };
        if next == hare.cell {
            return;
        }
        let dir = step_dir(hare.cell, next);
        let from = hare.cell;
        self.emit(
            format!("A hare startles and bolts to the {}.", dir.word()),
            Scope::Cell(from),
            vec![],
            vec![from],
            vec![],
            vec![],
            None,
        );
        if let Some(hare) = self.hare.as_mut() {
            hare.cell = next;
        }
    }

    /// A missed throw. The hare bolts to a dry neighbor farther from the hunter.
    /// None when it has nowhere to go.
    pub(crate) fn bolt_hunted_hare(&mut self, hunter: CellId) -> Option<&'static str> {
        let hare = self.hare.clone()?;
        let next = self.hare_away(hare.cell, hunter, hunter)?;
        if next == hare.cell {
            return None;
        }
        let dir = step_dir(hare.cell, next);
        if let Some(hare) = self.hare.as_mut() {
            hare.cell = next;
        }
        Some(dir.word())
    }

    /// A dry floor neighbor, not the person and not the cell they stepped to
    /// when that cell is not the hare's own. Farther from `away_from` wins.
    fn hare_away(&self, from: CellId, person: CellId, away_from: CellId) -> Option<CellId> {
        let (x, y) = cell_xy(from);
        let mut best: Option<(CellId, i32)> = None;
        for (dx, dy) in [(0, 1), (0, -1), (1, 0), (-1, 0)] {
            let Some(cell) = cell_id(x + dx, y + dy) else {
                continue;
            };
            if cell == person {
                continue;
            }
            let place = self.cell(cell);
            if place.region != Region::Floor || place.rock {
                continue;
            }
            if self.actors.values().any(|actor| actor.cell == cell) {
                continue;
            }
            let away = chebyshev(away_from, cell);
            if best.is_none_or(|(_, far)| away > far) {
                best = Some((cell, away));
            }
        }
        best.map(|(cell, _)| cell)
    }

    fn apply_thirst(&mut self) {
        self.schedule(self.time + 1, Wake::Thirst);
        let gain = self.tuning.thirst_per_tick;
        if gain == 0 {
            return;
        }
        let thirsty_at = self.tuning.thirsty_at;
        let parched_at = self.tuning.parched_at;
        let ids: Vec<ActorId> = self.actors.keys().copied().collect();
        for id in ids {
            let Some(actor) = self.actors.get_mut(&id) else {
                continue;
            };
            let before = actor.thirst;
            actor.thirst = (actor.thirst + gain).min(1000);
            let after = actor.thirst;
            let name = actor.name.clone();
            let cell = actor.cell;
            let crossed = if before < parched_at && after >= parched_at {
                Some(format!("{name}'s mouth is dry."))
            } else if before < thirsty_at && after >= thirsty_at {
                Some(format!("Thirst is in {name}'s mouth."))
            } else {
                None
            };
            if let Some(text) = crossed {
                self.emit(
                    text,
                    Scope::Personal(id),
                    vec![id],
                    vec![cell],
                    vec![],
                    vec![],
                    None,
                );
            }
        }
    }

    fn apply_hunger(&mut self) {
        self.schedule(self.time + 1, Wake::Hunger);
        let gain = self.tuning.hunger_per_tick;
        if gain == 0 {
            return;
        }
        let hungry_at = self.tuning.hungry_at;
        let starving_at = self.tuning.starving_at;
        let ids: Vec<ActorId> = self.actors.keys().copied().collect();
        for id in ids {
            let Some(actor) = self.actors.get_mut(&id) else {
                continue;
            };
            let before = actor.hunger;
            actor.hunger = (actor.hunger + gain).min(1000);
            let after = actor.hunger;
            let name = actor.name.clone();
            let cell = actor.cell;
            let crossed = if before < starving_at && after >= starving_at {
                Some(format!("{name}'s stomach is empty."))
            } else if before < hungry_at && after >= hungry_at {
                Some(format!("Hunger is in {name}'s stomach."))
            } else {
                None
            };
            if let Some(text) = crossed {
                self.emit(
                    text,
                    Scope::Personal(id),
                    vec![id],
                    vec![cell],
                    vec![],
                    vec![],
                    None,
                );
            }
        }
    }

    fn apply_energy(&mut self) {
        self.schedule(self.time + 1, Wake::Energy);
        let gain = self.tuning.energy_per_tick;
        if gain == 0 {
            return;
        }
        let tiring_at = self.tuning.tiring_at;
        let spent_at = self.tuning.spent_at;
        let ids: Vec<ActorId> = self.actors.keys().copied().collect();
        for id in ids {
            let Some(actor) = self.actors.get_mut(&id) else {
                continue;
            };
            let before = actor.energy;
            actor.energy = (actor.energy + gain).min(1000);
            let after = actor.energy;
            let name = actor.name.clone();
            let cell = actor.cell;
            let crossed = if before < spent_at && after >= spent_at {
                Some(format!("{name} is spent."))
            } else if before < tiring_at && after >= tiring_at {
                Some(format!("{name} is tiring."))
            } else {
                None
            };
            if let Some(text) = crossed {
                self.emit(
                    text,
                    Scope::Personal(id),
                    vec![id],
                    vec![cell],
                    vec![],
                    vec![],
                    None,
                );
            }
        }
    }

    /// Cold, wind, and sun on a body. Exposure is the shelter the valley
    /// already has. Wind doubles the cold only under open sky. Full shelter,
    /// the rock, feels neither, and the cold already on the body leaves.
    /// A body standing on a burning fire loses cold that tick instead, and
    /// keeps a few ticks of leftover heat after it steps off or the fire
    /// goes out.
    /// During those ticks the sky's cold is halved. Mud cuts the sun. Rain
    /// hides the sun and washes mud off exposed skin. Rain still puts an
    /// open fire out.
    fn apply_weather(&mut self) {
        self.schedule(self.time + 1, Wake::Weather);
        let raining = self.raining;
        let cold_base = self.tuning.cold_per_tick;
        let sun_base = self.tuning.sun_per_tick;
        let chilly_at = self.tuning.chilly_at;
        let cold_at = self.tuning.cold_at;
        let pink_at = self.tuning.pink_at;
        let burned_at = self.tuning.burned_at;
        let smear_at = self.tuning.smear_at;
        let mud_sun = self.tuning.mud_sun;
        let mud_fade = self.tuning.mud_fade;
        let ids: Vec<ActorId> = self.actors.keys().copied().collect();
        for id in ids {
            let wearing = self.wearing_hide(id);
            let Some(actor) = self.actors.get(&id) else {
                continue;
            };
            let exposure = self.exposure_millis(actor.cell);
            let name = actor.name.clone();
            let cell = actor.cell;
            let cold_before = actor.cold;
            let sun_before = actor.sunburn;
            let mud_before = actor.mud;
            let ochre_before = actor.ochre;
            let on_fire = self.fire_at(cell).is_some();
            let carried = actor.warmth;
            let mut from_fire = actor.warm_from_fire;
            let hold = u32::try_from(self.tuning.warmth_hold).unwrap_or(u32::MAX);
            let mut sky_cold = ((u64::from(cold_base) * u64::from(exposure)) / 1000) as u32;
            if exposure >= 1000 {
                sky_cold = sky_cold.saturating_mul(2);
            }
            // Night doubles that open-sky cold again. A rock, a lean-to, and
            // a fire in the cell keep the daytime shelter.
            if self.night() && exposure >= 1000 && !on_fire {
                sky_cold = sky_cold.saturating_mul(2);
            }
            let mut cold_gain = sky_cold;
            if !on_fire && carried > 0 {
                cold_gain /= 2;
            }
            if self.wearing_wolf_cloak(id) {
                cold_gain /= 4;
            } else if wearing && !self.wearing_only_hare_cloak(id) {
                cold_gain /= 2;
            } else if self.wearing_only_hare_cloak(id) {
                cold_gain = cold_gain.saturating_mul(3) / 4;
            }
            // An otter cloak is a hide, so the line above already halved the
            // sky. Rain and the brook cut that wet cold once more.
            if self.wearing_water_cloak(id)
                && (self.raining || self.cell(cell).region == Region::Brook)
            {
                cold_gain /= 2;
            }
            // Rain and night hide the sun. Open day sky is what burns.
            let mut sun_gain = if raining || self.night() {
                0
            } else {
                ((u64::from(sun_base) * u64::from(exposure)) / 1000) as u32
            };
            if (mud_before >= smear_at || ochre_before >= smear_at) && sun_gain > 0 {
                sun_gain = ((u64::from(sun_gain) * u64::from(mud_sun)) / 1000) as u32;
            }
            let rain = self.rain_exposure(cell);
            let washed = raining && rain >= 1000 && mud_before > 0;
            let mud_loss = if washed { mud_before } else { mud_fade };
            let ochre_washed = raining && rain >= 1000 && ochre_before > 0;
            let ochre_loss = if ochre_washed { ochre_before } else { mud_fade };
            // A fire in the cell, lean-to included, spends warmth instead of
            // taking the sky. Across a night that holds the body warm. A fire
            // on the next cell does not.
            let cold_after = if on_fire {
                cold_before.saturating_sub(self.tuning.fire_warmth)
            } else if exposure == 0 {
                0
            } else {
                (cold_before + cold_gain).min(1000)
            };
            // A clear day in the open does not ease a burn. Shade and the
            // night do, one sun-tick at a time. A rest spends more of those.
            let sheltered = exposure == 0 || self.lean_tos.contains(&cell);
            let ease = if self.night() || sheltered {
                sun_base
            } else {
                0
            };
            let sun_after = sun_before
                .saturating_add(sun_gain)
                .saturating_sub(ease)
                .min(1000);
            let warmth_after = if on_fire {
                hold
            } else {
                carried.saturating_sub(1)
            };
            if on_fire {
                from_fire = cold_after < chilly_at;
            } else if cold_after >= chilly_at {
                from_fire = false;
            } else if warmth_after == 0 && sky_cold == 0 {
                from_fire = false;
            } else if carried > 0 || from_fire {
                from_fire = true;
            }
            if let Some(actor) = self.actors.get_mut(&id) {
                actor.cold = cold_after;
                actor.warmth = warmth_after;
                actor.warm_from_fire = from_fire;
                actor.sunburn = sun_after;
                actor.mud = mud_before.saturating_sub(mud_loss);
                actor.ochre = ochre_before.saturating_sub(ochre_loss);
            }
            if !on_fire && carried == hold && hold > 0 {
                self.emit(
                    format!("{name} still has the warmth of the fire."),
                    Scope::Personal(id),
                    vec![id],
                    vec![cell],
                    vec![],
                    vec![],
                    None,
                );
            }
            let cold_line = if on_fire && cold_before >= chilly_at && cold_after < chilly_at {
                Some(format!("{name} is warm at the fire."))
            } else if on_fire && cold_before >= cold_at && cold_after < cold_at {
                Some(format!("{name} is warming at the fire."))
            } else if !on_fire && cold_before < cold_at && cold_after >= cold_at {
                Some(if exposure >= 1000 {
                    format!("{name} is cold. The wind is on them.")
                } else {
                    format!("{name} is cold.")
                })
            } else if !on_fire && cold_before < chilly_at && cold_after >= chilly_at {
                Some(if exposure >= 1000 {
                    format!("{name} is chilly. The wind is on them.")
                } else {
                    format!("{name} is chilly.")
                })
            } else {
                None
            };
            if let Some(text) = cold_line {
                self.emit(
                    text,
                    Scope::Personal(id),
                    vec![id],
                    vec![cell],
                    vec![],
                    vec![],
                    None,
                );
            }
            // The crossing is said once, to that body, without their name.
            // Look keeps the same words from the sunburn itself.
            let sun_line = if sun_before < burned_at && sun_after >= burned_at {
                Some("Your skin is burned. The sun burns. It hurts.".to_string())
            } else if sun_before < pink_at && sun_after >= pink_at {
                Some("Your skin is pink. The sun is hot on your skin.".to_string())
            } else {
                None
            };
            if let Some(text) = sun_line {
                self.emit(
                    text,
                    Scope::Personal(id),
                    vec![id],
                    vec![cell],
                    vec![],
                    vec![],
                    None,
                );
            }
            // Crossing back down is private too, and it does not say the name.
            let fade = if sun_before >= burned_at && sun_after < burned_at {
                Some("The burn settles.")
            } else if sun_before >= pink_at && sun_after < pink_at {
                Some("The pink fades.")
            } else {
                None
            };
            if let Some(text) = fade {
                self.emit(
                    text.to_string(),
                    Scope::Personal(id),
                    vec![id],
                    vec![cell],
                    vec![],
                    vec![],
                    None,
                );
            }
            if washed {
                self.emit(
                    format!("The rain washes the mud off {name}."),
                    Scope::Cell(cell),
                    vec![id],
                    vec![cell],
                    vec![],
                    vec![],
                    None,
                );
            }
            if ochre_washed {
                self.emit(
                    format!("The rain washes the red ochre off {name}."),
                    Scope::Cell(cell),
                    vec![id],
                    vec![cell],
                    vec![],
                    vec![],
                    None,
                );
            }
        }
        self.ease_clothes_at_fire();
        self.wash_open_ochre_marks();
        self.wash_open_charcoal_marks();
    }

    /// Open rain takes a red ochre mark off the ground. The rock overhang keeps its mark.
    fn wash_open_ochre_marks(&mut self) {
        if !self.raining {
            return;
        }
        let open: Vec<CellId> = self
            .ochre_marks
            .iter()
            .copied()
            .filter(|cell| self.exposure_millis(*cell) >= 1000)
            .collect();
        for cell in open {
            self.ochre_marks.remove(&cell);
            let region = self.cell(cell).region.word();
            self.emit(
                format!("The rain washes the red ochre mark off the ground on the {region}."),
                Scope::Cell(cell),
                vec![],
                vec![cell],
                vec![],
                vec![],
                None,
            );
        }
    }

    /// Open rain takes a black charcoal mark off the ground. The rock overhang keeps its mark.
    fn wash_open_charcoal_marks(&mut self) {
        if !self.raining {
            return;
        }
        let open: Vec<CellId> = self
            .charcoal_marks
            .iter()
            .copied()
            .filter(|cell| self.exposure_millis(*cell) >= 1000)
            .collect();
        for cell in open {
            self.charcoal_marks.remove(&cell);
            let region = self.cell(cell).region.word();
            self.emit(
                format!("The rain washes the black charcoal mark off the ground on the {region}."),
                Scope::Cell(cell),
                vec![],
                vec![cell],
                vec![],
                vec![],
                None,
            );
        }
    }

    /// Bee stings leave on their tick. Someone who can see that person hears it.
    /// The person hears "The stings fade." Mud and ochre do not clear them.
    fn apply_stings(&mut self) {
        self.schedule(self.time + 1, Wake::Stings);
        let now = self.time;
        let due: Vec<ActorId> = self
            .actors
            .iter()
            .filter(|(_, actor)| actor.stings_until == Some(now))
            .map(|(id, _)| *id)
            .collect();
        for id in due {
            let Some(actor) = self.actors.get_mut(&id) else {
                continue;
            };
            let nettle = actor.nettle_sting;
            actor.stings_until = None;
            actor.nettle_sting = false;
            let cell = actor.cell;
            let name = actor.name.clone();
            let line = if nettle {
                format!("The nettle stings on {name} fade.")
            } else {
                format!("The stings on {name} fade.")
            };
            self.emit(
                line,
                Scope::Cell(cell),
                vec![id],
                vec![cell],
                vec![],
                vec![],
                None,
            );
        }
    }

    fn apply_illness(&mut self) {
        self.schedule(self.time + 1, Wake::Illness);
        let loss = self.tuning.illness_recovery;
        if loss == 0 {
            return;
        }
        let ids: Vec<ActorId> = self.actors.keys().copied().collect();
        for id in ids {
            self.ease_illness(id, loss);
        }
    }

    /// Take sunburn off one body. Crossing out of burned, then out of pink,
    /// is private. Nobody else is on that scope.
    pub(crate) fn ease_sunburn(&mut self, id: ActorId, loss: u32) {
        if loss == 0 {
            return;
        }
        let pink_at = self.tuning.pink_at;
        let burned_at = self.tuning.burned_at;
        let settled = {
            let Some(actor) = self.actors.get_mut(&id) else {
                return;
            };
            let before = actor.sunburn;
            if before == 0 {
                return;
            }
            let after = before.saturating_sub(loss);
            actor.sunburn = after;
            let mut lines = Vec::new();
            if before >= burned_at && after < burned_at {
                lines.push("The burn settles.");
            }
            if before >= pink_at && after < pink_at {
                lines.push("The pink fades.");
            }
            (actor.cell, lines)
        };
        let (cell, lines) = settled;
        for text in lines {
            self.emit(
                text.to_string(),
                Scope::Personal(id),
                vec![id],
                vec![cell],
                vec![],
                vec![],
                None,
            );
        }
    }

    /// Take illness off one body. Crossing back under ill is private:
    /// "Your stomach settles." Nobody else is on that scope. A rest the
    /// wind allows spends several of these ticks at once.
    pub(crate) fn ease_illness(&mut self, id: ActorId, loss: u32) {
        if loss == 0 {
            return;
        }
        let ill_at = self.tuning.ill_at;
        let settled = {
            let Some(actor) = self.actors.get_mut(&id) else {
                return;
            };
            let before = actor.illness;
            if before == 0 {
                return;
            }
            actor.illness = before.saturating_sub(loss);
            let after = actor.illness;
            (before >= ill_at && after < ill_at).then_some(actor.cell)
        };
        if let Some(cell) = settled {
            self.emit(
                "Your stomach settles.",
                Scope::Personal(id),
                vec![id],
                vec![cell],
                vec![],
                vec![],
                None,
            );
        }
    }

    fn apply_rain(&mut self) {
        let first_dawn = self.tuning.night_at.saturating_add(self.tuning.night_len);
        // The evening rain holds through the first night. At the first dawn it
        // lets up, and it stays up through the next night so that night is clear.
        // It starts again at the dawn after that.
        if self.raining && self.time == first_dawn {
            self.raining = false;
            let resume = first_dawn
                .saturating_add(self.tuning.day_len)
                .saturating_add(self.tuning.night_len);
            self.schedule(resume, Wake::Rain);
            self.emit(
                "The rain lets up.",
                Scope::Valley,
                Vec::new(),
                Vec::new(),
                Vec::new(),
                Vec::new(),
                None,
            );
            return;
        }
        let first = !self.raining;
        self.raining = true;
        self.schedule(self.time + 1, Wake::Rain);

        let pile_cells: Vec<CellId> = self.piles.iter().map(|pile| pile.cell).collect();
        let pile_gains: Vec<u32> = self
            .piles
            .iter()
            .map(|pile| {
                let exposure = self.rain_exposure(pile.cell);
                self.wet_add(
                    self.season_gain(self.tuning.rain_gain(pile.material), exposure >= 1000),
                    exposure,
                )
            })
            .collect();
        for (pile, gain) in self.piles.iter_mut().zip(pile_gains) {
            pile.moisture = (pile.moisture + gain).min(1000);
        }

        let item_ids: Vec<ItemId> = self.items.keys().copied().collect();
        let item_gains: Vec<(ItemId, u32)> = item_ids
            .iter()
            .map(|id| {
                let item = self.item(*id).unwrap();
                let exposure = self.item_exposure(*id);
                let open_sky = exposure >= 1000 && item.role != Role::BarkCover;
                (
                    *id,
                    self.wet_add(
                        self.season_gain(self.tuning.rain_gain(item.material), open_sky),
                        exposure,
                    ),
                )
            })
            .collect();
        for (id, gain) in item_gains {
            if let Some(item) = self.items.get_mut(&id) {
                item.moisture = (item.moisture + gain).min(1000);
            }
        }

        let actor_ids: Vec<ActorId> = self.actors.keys().copied().collect();
        let body_gains: Vec<(ActorId, u32)> = actor_ids
            .iter()
            .map(|id| {
                let cell = self.actor(*id).unwrap().cell;
                let exposure = self.exposure_millis(cell);
                let rain = self.rain_exposure(cell);
                let wetness = self.actor(*id).unwrap().wetness;
                let mut gain =
                    self.wet_add(self.season_gain(self.tuning.body_rain, rain >= 1000), rain);
                // A bare lean-to is one step short of the rock. A little rain
                // dampens the clothes, and it stops there. Thatch keeps the
                // rain off, the way the rock does. Open sky still soaks.
                if self.lean_tos.contains(&cell)
                    && !self.thatched.contains(&cell)
                    && exposure == self.tuning.lean_to
                {
                    gain = self.tuning.damp_at.saturating_sub(wetness);
                }
                // The brook soaks anyone standing in it, and anyone aboard an
                // unpitched dugout. Pine resin on the hull keeps that soak off.
                // Rain from the sky still falls on the person aboard.
                let pitched = self
                    .aboard
                    .get(id)
                    .is_some_and(|canoe| self.canoe_pitched(*canoe));
                if self.cell(cell).region == Region::Brook && !pitched {
                    gain += self.tuning.brook_rain;
                }
                if self.wearing_water_cloak(*id) {
                    gain /= 2;
                }
                (*id, gain)
            })
            .collect();
        for (id, gain) in body_gains {
            if let Some(actor) = self.actors.get_mut(&id) {
                actor.wetness = (actor.wetness + gain).min(1000);
            }
        }

        let mut affected_items: Vec<ItemId> = self
            .items
            .values()
            .filter(|item| {
                matches!(
                    item.material,
                    Material::Branch | Material::Bark | Material::Log | Material::Berry
                )
            })
            .map(|item| item.id)
            .collect();
        affected_items.sort();
        let mut affected_cells: Vec<CellId> = pile_cells;
        affected_cells.extend(self.cells.iter().map(|cell| cell.id));
        let cause = self.first_rain.into_iter().collect();
        let text = if first {
            "Rain reaches the valley and finds whatever was left in the open."
        } else {
            "Rain falls."
        };
        let id = self.emit(
            text,
            Scope::Valley,
            Vec::new(),
            affected_cells,
            affected_items,
            cause,
            None,
        );
        if first {
            self.first_rain = Some(id);
        }
        self.douse_open_fires(id);
        self.douse_open_brands();
    }

    fn douse_open_brands(&mut self) {
        let open: Vec<ItemId> = self
            .items
            .values()
            .filter(|item| item.material == Material::Brand)
            .filter(|item| self.item_exposure(item.id) >= 1000)
            .map(|item| item.id)
            .collect();
        for id in open {
            self.quench_brand(id, true);
        }
    }

    fn brand_out(&mut self, id: ItemId) {
        let Some(item) = self.item(id) else {
            return;
        };
        if item.material != Material::Brand || item.firing_ready_at != Some(self.time) {
            return;
        }
        self.quench_brand(id, false);
    }

    fn quench_brand(&mut self, id: ItemId, rain: bool) {
        let Some(item) = self.item(id).cloned() else {
            return;
        };
        if item.material != Material::Brand {
            return;
        }
        let (cell, affected, text) = if let Some(carrier) = item.carrier {
            let name = self.name_of(carrier);
            let cell = self
                .actor(carrier)
                .map(|person| person.cell)
                .unwrap_or(item.cell);
            let text = if rain {
                format!("The rain puts {name}'s brand out.")
            } else {
                format!("{name}'s brand burns out.")
            };
            (cell, vec![carrier], text)
        } else if rain {
            (
                item.cell,
                vec![],
                "The rain puts the brand out.".to_string(),
            )
        } else {
            (item.cell, vec![], "The brand burns out.".to_string())
        };
        self.items.remove(&id);
        self.emit(
            text,
            Scope::Cell(cell),
            affected,
            vec![cell],
            vec![],
            vec![],
            None,
        );
    }

    fn douse_open_fires(&mut self, rain_event: EventId) {
        let open: Vec<FireId> = self
            .fires
            .values()
            .filter(|fire| self.exposure_millis(fire.cell) >= 1000)
            .map(|fire| fire.id)
            .collect();
        for id in open {
            self.extinguish(id, "Rain puts out the fire.", Some(rain_event));
        }
    }

    fn burn_down(&mut self, id: FireId) {
        if self.fires.contains_key(&id) {
            self.extinguish(id, "The fire burns down.", None);
        }
    }

    /// The cover has waited. The wood chars if the fire is still burning.
    /// A fire that died first gives the branches back, and no charcoal from the bank.
    fn char_bank(&mut self, cell: CellId) {
        let Some(count) = self.banks.remove(&cell) else {
            return;
        };
        if count == 0 || self.fire_at(cell).is_none() {
            if count > 0 {
                self.add_pile(cell, Material::Branch, count, 0);
            }
            self.emit(
                "The banked wood did not char. The fire is out.",
                Scope::Cell(cell),
                vec![],
                vec![cell],
                vec![],
                vec![],
                None,
            );
            return;
        }
        let ids: Vec<_> = self
            .fires
            .values()
            .filter(|fire| fire.cell == cell)
            .map(|fire| fire.id)
            .collect();
        for id in ids {
            self.fires.remove(&id);
            self.schedule.retain(|entry| entry.wake != Wake::Fire(id));
        }
        let mut lumps = Vec::new();
        for _ in 0..count {
            lumps.push(self.mint_item(Material::Charcoal, 0, cell, None, Role::Loose));
        }
        let text = if count == 1 {
            "The banked wood chars into a lump of charcoal. The fire is out."
        } else {
            "The banked wood chars into charcoal. The fire is out."
        };
        self.emit(
            text,
            Scope::Cell(cell),
            vec![],
            vec![cell],
            lumps,
            vec![],
            None,
        );
    }

    fn complete_work(&mut self, id: WorkId) {
        let Some(work) = self.works.remove(&id) else {
            return;
        };
        match work.kind {
            WorkKind::Drag => self.finish_drag(work),
            WorkKind::RaiseFrame => self.finish_frame_phase(work),
            WorkKind::LayBark => self.finish_bark(work),
        }
    }

    fn finish_drag(&mut self, work: Work) {
        let Some(item_id) = work.item else {
            self.free_actors(&work.actors, work.id);
            return;
        };
        let Some(dir) = work.dir else {
            self.free_actors(&work.actors, work.id);
            return;
        };
        let Some(item) = self.item(item_id).cloned() else {
            self.free_actors(&work.actors, work.id);
            return;
        };
        let still: Vec<ActorId> = work
            .actors
            .iter()
            .copied()
            .filter(|actor| {
                self.gripping(*actor) == Some(item_id)
                    && self
                        .actor(*actor)
                        .map(|person| person.cell == item.cell && person.busy == Some(work.id))
                        .unwrap_or(false)
            })
            .collect();
        if (still.len() as u32) < self.hands_on_the_log(item.cell) {
            self.abort_drag(
                &work,
                item_id,
                item.cell,
                "There are not enough hands still on the log. It settles, in contact with the ground.",
            );
            return;
        }
        let (x, y) = cell_xy(item.cell);
        let (dx, dy) = dir.delta();
        let Some(dest) = cell_id(x + dx, y + dy) else {
            self.abort_drag(
                &work,
                item_id,
                item.cell,
                "The log has nowhere to go. It stays in contact with the ground.",
            );
            return;
        };
        if self.cell(dest).region == Region::Brook {
            self.abort_drag(
                &work,
                item_id,
                item.cell,
                "The brook is in the way. The log stays in contact with the ground.",
            );
            return;
        }
        if let Some(moving) = self.items.get_mut(&item_id) {
            moving.cell = dest;
        }
        for actor in &still {
            self.move_actor(*actor, dest);
        }
        let names = self.names(&still);
        let verb = if still.len() == 1 { "moves" } else { "move" };
        self.free_actors(&work.actors, work.id);
        self.emit(
            format!(
                "The log shifts {dir}. {names} {verb} with it, still in contact with it.",
                dir = dir.word()
            ),
            Scope::Cell(dest),
            still,
            vec![item.cell, dest],
            vec![item_id],
            work.cause_events,
            None,
        );
    }

    fn abort_drag(&mut self, work: &Work, item_id: ItemId, cell: CellId, text: &str) {
        self.free_actors(&work.actors, work.id);
        self.emit(
            text,
            Scope::Cell(cell),
            work.actors.clone(),
            vec![cell],
            vec![item_id],
            work.cause_events.clone(),
            None,
        );
    }

    fn finish_frame_phase(&mut self, mut work: Work) {
        let Some(actor_id) = work.actors.first().copied() else {
            return;
        };
        let cell = self.actor(actor_id).map(|a| a.cell).unwrap_or(0);
        let name = self.name_of(actor_id);
        let branches = self.loose_ids(actor_id, Material::Branch);
        let Some(branch_id) = branches.first().copied() else {
            self.free_actors(&[actor_id], work.id);
            self.emit(
                format!("{name} has no branch left in hand. The frame stays as it is."),
                Scope::Cell(cell),
                vec![actor_id],
                vec![cell],
                vec![],
                work.cause_events.clone(),
                None,
            );
            return;
        };
        let frame_id = if let Some(id) = work.construction {
            id
        } else if let Some(existing) = self.construction_at(cell) {
            existing.id
        } else {
            self.mint_construction(cell)
        };
        work.construction = Some(frame_id);
        let phase = work.phase_done;
        let role = if phase >= 2 {
            Role::Beam
        } else {
            Role::Support
        };
        if let Some(item) = self.items.get_mut(&branch_id) {
            item.carrier = None;
            item.cell = cell;
            item.role = role;
            item.construction = Some(frame_id);
        }
        if let Some(frame) = self.constructions.get_mut(&frame_id) {
            match phase {
                0 => frame.support_a = Some(branch_id),
                1 => frame.support_b = Some(branch_id),
                _ => frame.beam = Some(branch_id),
            }
        }
        work.phase_done += 1;
        let supported = self.supported_at_both_ends(frame_id);
        let text = match phase {
            0 => format!("{name} sets a support. The frame is not supported at both ends yet."),
            1 => format!("{name} sets the second support."),
            _ if supported => format!(
                "{name} lays a beam in contact with both supports. The frame is supported at both ends."
            ),
            _ => format!("{name} lays a beam, but it is not supported at both ends."),
        };
        self.emit(
            text,
            Scope::Cell(cell),
            vec![actor_id],
            vec![cell],
            vec![branch_id],
            work.cause_events.clone(),
            None,
        );
        if work.phase_done < work.phases {
            work.completes_at = self.time + self.tuning.frame_phase_ticks;
            let id = work.id;
            self.schedule(work.completes_at, Wake::Work(id));
            self.works.insert(id, work);
        } else {
            self.free_actors(&[actor_id], work.id);
        }
    }

    fn finish_bark(&mut self, work: Work) {
        let Some(actor_id) = work.actors.first().copied() else {
            return;
        };
        let cell = self.actor(actor_id).map(|a| a.cell).unwrap_or(0);
        let name = self.name_of(actor_id);
        let frame_id = work
            .construction
            .or_else(|| self.construction_at(cell).map(|c| c.id));
        let bark_id = self.loose_ids(actor_id, Material::Bark).first().copied();
        let ready = frame_id
            .map(|id| self.supported_at_both_ends(id))
            .unwrap_or(false);
        if !ready || bark_id.is_none() {
            self.free_actors(&[actor_id], work.id);
            self.emit(
                format!("{name} stops. The bark does not lie on a frame supported at both ends."),
                Scope::Cell(cell),
                vec![actor_id],
                vec![cell],
                vec![],
                work.cause_events,
                None,
            );
            return;
        }
        let frame_id = frame_id.unwrap();
        let bark_id = bark_id.unwrap();
        if let Some(item) = self.items.get_mut(&bark_id) {
            item.carrier = None;
            item.cell = cell;
            item.role = Role::BarkCover;
            item.construction = Some(frame_id);
        }
        if let Some(frame) = self.constructions.get_mut(&frame_id) {
            frame.bark = Some(bark_id);
        }
        let supported = self.supported_at_both_ends(frame_id);
        let text = if supported {
            format!(
                "{name} lays bark over the beam. It is supported at both ends, and the wood beneath is under a roof."
            )
        } else {
            format!("{name} lays bark, but the frame is not supported at both ends.")
        };
        self.free_actors(&[actor_id], work.id);
        self.emit(
            text,
            Scope::Cell(cell),
            vec![actor_id],
            vec![cell],
            vec![bark_id],
            work.cause_events,
            None,
        );
    }

    fn mint_construction(&mut self, cell: CellId) -> ConstructionId {
        let id = self.next_construction;
        self.next_construction += 1;
        self.constructions.insert(
            id,
            crate::model::Construction {
                id,
                cell,
                support_a: None,
                support_b: None,
                beam: None,
                bark: None,
            },
        );
        id
    }

    pub(crate) fn free_actors(&mut self, actors: &[ActorId], work_id: WorkId) {
        for actor in actors {
            if let Some(person) = self.actors.get_mut(actor) {
                if person.busy == Some(work_id) {
                    person.busy = None;
                }
            }
        }
    }

    pub(crate) fn interrupt_actor(&mut self, actor_id: ActorId) -> Result<(), String> {
        let person = self
            .actor(actor_id)
            .ok_or("No such person is in the valley.")?;
        let work_id = person
            .busy
            .ok_or("You are not in the middle of anything.")?;
        let cell = person.cell;
        let name = person.name.clone();
        let stop = self.emit(
            format!("{name} stops."),
            Scope::Cell(cell),
            vec![actor_id],
            vec![cell],
            vec![],
            vec![],
            None,
        );
        self.leave_work(actor_id, work_id, stop);
        Ok(())
    }

    pub(crate) fn release_actor(&mut self, actor_id: ActorId) -> Result<(), String> {
        let item = self
            .gripping(actor_id)
            .ok_or("You are not holding the log.")?;
        let cell = self.actor(actor_id).map(|a| a.cell).unwrap_or(0);
        let name = self.name_of(actor_id);
        self.grips.retain(|grip| grip.actor != actor_id);
        let event = self.emit(
            format!("{name} lets go of the log."),
            Scope::Cell(cell),
            vec![actor_id],
            vec![cell],
            vec![item],
            vec![],
            None,
        );
        if let Some(work_id) = self.actor(actor_id).and_then(|a| a.busy) {
            self.leave_work(actor_id, work_id, event);
        }
        Ok(())
    }

    pub(crate) fn leave_work(&mut self, actor_id: ActorId, work_id: WorkId, cause: EventId) {
        let Some(work) = self.works.get(&work_id).cloned() else {
            self.free_actors(&[actor_id], work_id);
            return;
        };
        match work.kind {
            WorkKind::Drag => {
                if let Some(person) = self.actors.get_mut(&actor_id) {
                    if person.busy == Some(work_id) {
                        person.busy = None;
                    }
                }
                let remaining: Vec<ActorId> = work
                    .actors
                    .iter()
                    .copied()
                    .filter(|id| *id != actor_id)
                    .collect();
                let cell = work
                    .item
                    .and_then(|id| self.item(id))
                    .map(|item| item.cell)
                    .unwrap_or(0);
                if (remaining.len() as u32) < self.hands_on_the_log(cell) {
                    self.works.remove(&work_id);
                    self.schedule
                        .retain(|entry| entry.wake != Wake::Work(work_id));
                    self.free_actors(&remaining, work_id);
                    let mut affected = remaining;
                    affected.push(actor_id);
                    self.emit(
                        "There are not enough hands. The log settles, still in contact with the ground.",
                        Scope::Cell(cell),
                        affected,
                        vec![cell],
                        work.item.into_iter().collect(),
                        vec![cause],
                        None,
                    );
                } else if let Some(live) = self.works.get_mut(&work_id) {
                    live.actors = remaining;
                    live.cause_events.push(cause);
                }
            }
            WorkKind::RaiseFrame | WorkKind::LayBark => {
                let cell = self.actor(actor_id).map(|a| a.cell).unwrap_or(0);
                self.works.remove(&work_id);
                self.schedule
                    .retain(|entry| entry.wake != Wake::Work(work_id));
                self.free_actors(&[actor_id], work_id);
                self.emit(
                    "The work stops. What was already raised stays as it is.",
                    Scope::Cell(cell),
                    vec![actor_id],
                    vec![cell],
                    vec![],
                    vec![cause],
                    None,
                );
            }
        }
    }
}

fn step_dir(from: CellId, to: CellId) -> Dir {
    let (x1, y1) = cell_xy(from);
    let (x2, y2) = cell_xy(to);
    if x2 > x1 {
        Dir::East
    } else if x2 < x1 {
        Dir::West
    } else if y2 > y1 {
        Dir::South
    } else {
        Dir::North
    }
}
