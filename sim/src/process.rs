//! Rain and unfinished work. Both run from the simulation-time queue,
//! including when nobody has sent a command.

use crate::model::{
    cell_id, cell_xy, ActorId, CellId, ConstructionId, Dir, EventId, FireId, ItemId, Material,
    Region, Role, Scope, Wake, Work, WorkId, WorkKind, World, DEER_ROUTE,
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
                Wake::Season => self.apply_season(),
                Wake::Rot => self.apply_rot(),
            }
        }
    }

    /// One change of season. Open rain soaks wood twice as fast from now on.
    /// A roof and the rock overhang keep the gains they already had.
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
            if self.take_from_pile(cell, Material::Branch, 1).is_some() {
                self.emit(
                    "A branch rots away in the open. You hear branches breaking.",
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
        if self.pile_count(deer.cell, Material::Berry) > 0 {
            let _ = self.take_from_pile(deer.cell, Material::Berry, 1);
            let id = self.emit(
                "A deer eats a handful of berries.",
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
        let through_brush = self.has_brush(origin) || self.has_brush(dest);
        self.tracks.insert(origin);
        let mut text = format!("A deer steps {}.", dir.word());
        if through_brush {
            text.push_str(" You hear branches breaking.");
        }
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
    /// already has. Wind doubles the cold only under open sky. Mud cuts
    /// the sun. Rain hides the sun and washes mud off exposed skin.
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
        let mud_wash = self.tuning.mud_wash;
        let ids: Vec<ActorId> = self.actors.keys().copied().collect();
        for id in ids {
            let Some(actor) = self.actors.get(&id) else {
                continue;
            };
            let exposure = self.exposure_millis(actor.cell);
            let name = actor.name.clone();
            let cell = actor.cell;
            let cold_before = actor.cold;
            let sun_before = actor.sunburn;
            let mud_before = actor.mud;
            let mut cold_gain = ((u64::from(cold_base) * u64::from(exposure)) / 1000) as u32;
            if exposure >= 1000 {
                cold_gain = cold_gain.saturating_mul(2);
            }
            let mut sun_gain = if raining {
                0
            } else {
                ((u64::from(sun_base) * u64::from(exposure)) / 1000) as u32
            };
            if mud_before >= smear_at && sun_gain > 0 {
                sun_gain = ((u64::from(sun_gain) * u64::from(mud_sun)) / 1000) as u32;
            }
            let mud_loss = if mud_before == 0 {
                0
            } else if raining && exposure >= 1000 {
                mud_wash
            } else {
                mud_fade
            };
            if let Some(actor) = self.actors.get_mut(&id) {
                actor.cold = (cold_before + cold_gain).min(1000);
                actor.sunburn = (sun_before + sun_gain).min(1000);
                actor.mud = mud_before.saturating_sub(mud_loss);
            }
            let cold_after = cold_before + cold_gain;
            let sun_after = sun_before + sun_gain;
            let cold_line = if cold_before < cold_at && cold_after >= cold_at {
                Some(if exposure >= 1000 {
                    format!("{name} is cold. The wind is on them.")
                } else {
                    format!("{name} is cold.")
                })
            } else if cold_before < chilly_at && cold_after >= chilly_at {
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
            let sun_line = if sun_before < burned_at && sun_after >= burned_at {
                Some(format!("{name}'s skin is burned."))
            } else if sun_before < pink_at && sun_after >= pink_at {
                Some(format!("{name}'s skin is pink."))
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
        }
    }

    fn apply_illness(&mut self) {
        self.schedule(self.time + 1, Wake::Illness);
        let loss = self.tuning.illness_recovery;
        if loss == 0 {
            return;
        }
        let ill_at = self.tuning.ill_at;
        let ids: Vec<ActorId> = self.actors.keys().copied().collect();
        for id in ids {
            let Some(actor) = self.actors.get_mut(&id) else {
                continue;
            };
            let before = actor.illness;
            if before == 0 {
                continue;
            }
            actor.illness = before.saturating_sub(loss);
            let after = actor.illness;
            if before >= ill_at && after < ill_at {
                let name = actor.name.clone();
                let cell = actor.cell;
                self.emit(
                    format!("{name}'s stomach settles."),
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

    fn apply_rain(&mut self) {
        let first = !self.raining;
        self.raining = true;
        self.schedule(self.time + 1, Wake::Rain);

        let pile_cells: Vec<CellId> = self.piles.iter().map(|pile| pile.cell).collect();
        let pile_gains: Vec<u32> = self
            .piles
            .iter()
            .map(|pile| {
                let exposure = self.exposure_millis(pile.cell);
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
                let mut gain = self.wet_add(
                    self.season_gain(self.tuning.body_rain, exposure >= 1000),
                    exposure,
                );
                if self.cell(cell).region == Region::Brook {
                    gain += self.tuning.brook_rain;
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
        if (still.len() as u32) < self.tuning.drag_needed {
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
        self.free_actors(&work.actors, work.id);
        self.emit(
            format!(
                "The log shifts {dir}. {names} move with it, still in contact with it.",
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
                if (remaining.len() as u32) < self.tuning.drag_needed {
                    let cell = work
                        .item
                        .and_then(|id| self.item(id))
                        .map(|item| item.cell)
                        .unwrap_or(0);
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
