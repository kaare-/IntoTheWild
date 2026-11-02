//! Rain and unfinished work. Both run from the simulation-time queue,
//! including when nobody has sent a command.

use crate::model::{
    cell_id, cell_xy, ActorId, CellId, ConstructionId, EventId, FireId, ItemId, Material, Region,
    Role, Scope, Wake, Work, WorkId, WorkKind, World,
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
                Wake::Work(id) => self.complete_work(id),
                Wake::Fire(id) => self.burn_down(id),
            }
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

    fn apply_rain(&mut self) {
        let first = !self.raining;
        self.raining = true;
        self.schedule(self.time + 1, Wake::Rain);

        let pile_cells: Vec<CellId> = self.piles.iter().map(|pile| pile.cell).collect();
        let pile_gains: Vec<u32> = self
            .piles
            .iter()
            .map(|pile| {
                self.wet_add(
                    self.tuning.rain_gain(pile.material),
                    self.exposure_millis(pile.cell),
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
                (
                    *id,
                    self.wet_add(
                        self.tuning.rain_gain(self.item(*id).unwrap().material),
                        self.item_exposure(*id),
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
                let mut gain = self.wet_add(self.tuning.body_rain, self.exposure_millis(cell));
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
