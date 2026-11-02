//! One observation function. Text, map, and JSON are views of the same
//! facts. Moisture numbers, tokens, and the cause log stay here.

use crate::model::{
    cell_id, cell_xy, chebyshev, ActorId, CellId, Dir, ExamineTarget, Material, Role, Scope, World,
};
use serde::Serialize;

pub const LEGEND: &str = "\
@ you          p other
L log          w dry wood
W wet wood     b bark
B wet bark
R rock roof    H bark roof
f unfinished frame
~ brook    ^ slope    . ground
?? outside your sight";

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct Observation {
    pub sim_time: u64,
    pub raining: bool,
    pub text: String,
    pub map: String,
    pub legend: String,
    pub self_info: SelfInfo,
    pub visible_cells: Vec<CellView>,
    pub events: Vec<EventView>,
    pub allowed_actions: Vec<String>,
    pub work: Option<WorkView>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct SelfInfo {
    pub id: ActorId,
    pub name: String,
    pub x: i32,
    pub y: i32,
    pub region: String,
    pub under_roof: bool,
    pub wetness: String,
    pub thirst: String,
    pub gripping: bool,
    pub carrying: Vec<CarryView>,
    pub busy: bool,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct CarryView {
    pub material: String,
    pub condition: String,
    pub count: u32,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct CellView {
    pub x: i32,
    pub y: i32,
    pub region: String,
    pub under_roof: bool,
    pub roof: String,
    pub trail: bool,
    pub branch_pile: u32,
    pub branch_pile_condition: Option<String>,
    pub loose_branches: u32,
    pub loose_branch_condition: Option<String>,
    pub bark_pile: u32,
    pub bark_pile_condition: Option<String>,
    pub loose_bark: u32,
    pub log: bool,
    pub log_condition: Option<String>,
    pub log_gripped_by: Vec<String>,
    pub frame: String,
    pub actors: Vec<String>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct EventView {
    pub id: u64,
    pub time: u64,
    pub text: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct WorkView {
    pub kind: String,
    pub detail: String,
    pub finishes_at: u64,
}

impl World {
    pub fn observe(&self, actor_id: ActorId) -> Option<Observation> {
        let actor = self.actor(actor_id)?;
        let (x, y) = cell_xy(actor.cell);
        Some(Observation {
            sim_time: self.time,
            raining: self.raining,
            text: self.prose(actor_id),
            map: self.map_string(actor_id),
            legend: LEGEND.to_string(),
            self_info: SelfInfo {
                id: actor.id,
                name: actor.name.clone(),
                x,
                y,
                region: self.cell(actor.cell).region.word().into(),
                under_roof: self.exposure_millis(actor.cell) < 1000,
                wetness: self.condition_word(actor.wetness).into(),
                thirst: self.thirst_word(actor.thirst).into(),
                gripping: self.gripping(actor.id).is_some(),
                carrying: self.carry_views(actor.id),
                busy: actor.busy.is_some(),
            },
            visible_cells: self
                .visible_cell_ids(actor_id)
                .into_iter()
                .map(|cell| self.cell_view(cell))
                .collect(),
            events: self.recent_events(actor_id),
            allowed_actions: self.allowed_actions(actor_id),
            work: self.work_view(actor_id),
        })
    }

    pub fn examine(&self, actor_id: ActorId, target: ExamineTarget) -> String {
        let Some(actor) = self.actor(actor_id) else {
            return "No such person is in the valley.".into();
        };
        match target {
            ExamineTarget::Here => self.here_text(actor_id),
            ExamineTarget::SelfBody => self.self_text(actor_id),
            ExamineTarget::Wood => self.examine_material(actor_id, Material::Branch),
            ExamineTarget::Bark => self.examine_material(actor_id, Material::Bark),
            ExamineTarget::Log => self.examine_log(actor_id),
            ExamineTarget::Frame => self.examine_frames(actor_id),
            ExamineTarget::Person(name) => {
                if actor.name.eq_ignore_ascii_case(&name) {
                    return self.self_text(actor_id);
                }
                let found = self.actors.values().find(|other| {
                    other.name.eq_ignore_ascii_case(&name)
                        && chebyshev(actor.cell, other.cell) <= self.tuning.sight
                });
                match found {
                    Some(other) => self.person_text(other.id),
                    None => format!("You cannot see {name} from here."),
                }
            }
        }
    }

    fn prose(&self, actor_id: ActorId) -> String {
        let actor = self.actor(actor_id).unwrap();
        let mut lines = Vec::new();
        lines.push(self.sky_line().to_string());
        lines.push(self.here_text(actor_id));
        lines.push(self.self_text(actor_id));
        if let Some(work) = self.work_view(actor_id) {
            lines.push(work.detail);
        }
        lines.push("Around you:".into());
        for dir in Dir::all() {
            lines.push(self.adjacent_line(actor.cell, dir));
        }
        lines.join("\n")
    }

    fn sky_line(&self) -> &'static str {
        if self.raining {
            "Rain is falling on the valley."
        } else if self.time + 2 >= self.tuning.rain_starts_at {
            "The light is going. The air smells like rain."
        } else {
            "The valley is still dry. Evening is coming."
        }
    }

    fn here_text(&self, actor_id: ActorId) -> String {
        let actor = self.actor(actor_id).unwrap();
        let cell = self.cell(actor.cell);
        let mut parts = vec![format!(
            "You are on the {}, {}.",
            cell.region.word(),
            self.cover_phrase(actor.cell)
        )];
        if cell.trail {
            parts.push("A deer trail passes here.".into());
        }
        if cell.rock {
            parts.push("Rock leans overhead. Wood left here stays out of the weather.".into());
        }
        let others = self.others_here(actor_id);
        if !others.is_empty() {
            parts.push(format!(
                "{} {} here.",
                self.names(&others),
                if others.len() == 1 { "is" } else { "are" }
            ));
        }
        parts.join(" ")
    }

    fn self_text(&self, actor_id: ActorId) -> String {
        let actor = self.actor(actor_id).unwrap();
        let mut parts = vec![
            format!("Your clothes are {}.", self.condition_word(actor.wetness)),
            format!("You are {}.", self.thirst_sentence(actor.thirst)),
        ];
        let carrying = self.carry_views(actor_id);
        if carrying.is_empty() {
            parts.push("Your arms are empty.".into());
        } else {
            let bits: Vec<String> = carrying
                .iter()
                .map(|item| {
                    format!(
                        "{} {} carried by you ({})",
                        item.count, item.material, item.condition
                    )
                })
                .collect();
            parts.push(format!("You have {}.", bits.join(", ")));
        }
        if self.gripping(actor_id).is_some() {
            let mut holders = self.log_gripped_by();
            holders.retain(|name| name != &actor.name);
            if holders.is_empty() {
                parts.push("You are gripping the heavy log.".into());
            } else {
                parts.push(format!(
                    "You are gripping the heavy log, with {}.",
                    holders.join(", ")
                ));
            }
        }
        if actor.busy.is_some() {
            parts.push("Your body is occupied.".into());
        }
        parts.join(" ")
    }

    fn person_text(&self, actor_id: ActorId) -> String {
        let actor = self.actor(actor_id).unwrap();
        let (x, y) = cell_xy(actor.cell);
        format!(
            "{} is on the {}, {}. Their clothes are {}. They are {}.",
            actor.name,
            self.cell(actor.cell).region.word(),
            self.cover_phrase(actor.cell),
            self.condition_word(actor.wetness),
            self.thirst_sentence(actor.thirst)
        ) + &format!(" ({x},{y})")
    }

    fn adjacent_line(&self, origin: CellId, dir: Dir) -> String {
        let (x, y) = cell_xy(origin);
        let (dx, dy) = dir.delta();
        let Some(dest) = cell_id(x + dx, y + dy) else {
            return format!("{}: the valley ends.", capitalize(dir.word()));
        };
        let cell = self.cell(dest);
        let mut bits = vec![cell.region.word().to_string()];
        if cell.rock {
            bits.push("rock overhang".into());
        }
        if self.pile_count(dest, Material::Log) > 0 || self.log_item().map(|i| i.cell) == Some(dest)
        {
            bits.push("the heavy log".into());
        }
        if self.pile_count(dest, Material::Branch) > 0 {
            bits.push("a pile of branches".into());
        }
        if self.construction_at(dest).is_some() {
            bits.push("a frame".into());
        }
        let people = self
            .actors
            .values()
            .filter(|actor| actor.cell == dest)
            .count();
        if people > 0 {
            bits.push(format!(
                "{people} {}",
                if people == 1 { "person" } else { "people" }
            ));
        }
        format!("{}: {}.", capitalize(dir.word()), bits.join(", "))
    }

    fn examine_material(&self, viewer: ActorId, material: Material) -> String {
        let origin = self.actor(viewer).unwrap().cell;
        let mut lines = Vec::new();
        for cell in self.visible_cell_ids(viewer) {
            let count = self.pile_count(cell, material);
            if count > 0 {
                let noun = pile_noun(material, count);
                lines.push(format!(
                    "A pile of {count} {noun} is in contact with the ground {}. The {} is {}. {}",
                    self.place_phrase(origin, cell),
                    if material == Material::Bark {
                        "bark"
                    } else {
                        "wood"
                    },
                    self.condition_word(self.pile_moisture(cell, material)),
                    self.rain_reaches_phrase(cell)
                ));
            }
            let loose: Vec<_> = self
                .items
                .values()
                .filter(|item| {
                    item.material == material
                        && item.role == Role::Loose
                        && item.carrier.is_none()
                        && item.cell == cell
                })
                .map(|item| item.moisture)
                .collect();
            if !loose.is_empty() {
                let worst = loose.iter().copied().max().unwrap_or(0);
                lines.push(format!(
                    "{} loose {} {} in contact with the ground {}. {}",
                    loose.len(),
                    pile_noun(material, loose.len() as u32),
                    if loose.len() == 1 { "is" } else { "are" },
                    self.place_phrase(origin, cell),
                    self.condition_sentence(worst, cell)
                ));
            }
        }
        for other in self.visible_actor_ids(viewer) {
            let ids = self.loose_ids(other, material);
            if ids.is_empty() {
                continue;
            }
            let moisture = ids
                .iter()
                .filter_map(|id| self.item(*id).map(|item| item.moisture))
                .max()
                .unwrap_or(0);
            let who = if other == viewer {
                "you".to_string()
            } else {
                self.name_of(other)
            };
            let cell = self.actor(other).unwrap().cell;
            lines.push(format!(
                "{} {} {} carried by {who}, {}. {}",
                ids.len(),
                pile_noun(material, ids.len() as u32),
                if ids.len() == 1 { "is" } else { "are" },
                self.cover_phrase(cell),
                self.condition_word(moisture)
            ));
        }
        if lines.is_empty() {
            let name = match material {
                Material::Branch => "wood",
                Material::Bark => "bark",
                Material::Log => "log",
            };
            format!("You see no {name} from here.")
        } else {
            lines.join("\n")
        }
    }

    fn examine_log(&self, viewer: ActorId) -> String {
        let origin = self.actor(viewer).unwrap().cell;
        if let Some(item) = self.log_item() {
            if chebyshev(origin, item.cell) > self.tuning.sight {
                return "You see no log from here.".into();
            }
            let holders = self.log_gripped_by();
            let held = if holders.is_empty() {
                "Nobody is gripping it.".to_string()
            } else {
                format!(
                    "{} {} gripping it.",
                    self.join_names(&holders),
                    if holders.len() == 1 { "is" } else { "are" }
                )
            };
            return format!(
                "The heavy log is in contact with the ground {}. It is {}. {held} {}",
                self.place_phrase(origin, item.cell),
                self.condition_word(item.moisture),
                self.rain_reaches_phrase(item.cell)
            );
        }
        for cell in self.visible_cell_ids(viewer) {
            if self.pile_count(cell, Material::Log) > 0 {
                return format!(
                    "A heavy log lies in contact with the ground {}. It has not been lifted. It is {}. {}",
                    self.place_phrase(origin, cell),
                    self.condition_word(self.pile_moisture(cell, Material::Log)),
                    self.rain_reaches_phrase(cell)
                );
            }
        }
        "You see no log from here.".into()
    }

    fn examine_frames(&self, viewer: ActorId) -> String {
        let origin = self.actor(viewer).unwrap().cell;
        let mut lines = Vec::new();
        for frame in self.constructions.values() {
            if chebyshev(origin, frame.cell) > self.tuning.sight {
                continue;
            }
            let supports = frame.support_a.is_some() as u8 + frame.support_b.is_some() as u8;
            let beam = if frame.beam.is_some() {
                "a beam"
            } else {
                "no beam"
            };
            let supported = self.supported_at_both_ends(frame.id);
            let relation = if supported {
                "It is supported at both ends."
            } else {
                "It is not supported at both ends."
            };
            let wood_moisture = [frame.support_a, frame.support_b, frame.beam]
                .into_iter()
                .flatten()
                .filter_map(|id| self.item(id).map(|item| item.moisture))
                .max();
            let bark = match frame.bark.and_then(|id| self.item(id)) {
                Some(sheet) => format!(
                    "Bark lies over it and is {}.",
                    self.condition_word(sheet.moisture)
                ),
                None => "No bark lies over it.".into(),
            };
            let beneath = match wood_moisture {
                Some(moisture) => format!(
                    "The wood in the frame is {}.",
                    self.condition_word(moisture)
                ),
                None => "There is no wood in the frame yet.".into(),
            };
            lines.push(format!(
                "A frame stands {}. Supports: {supports}. It has {beam}. {relation} {bark} {beneath} {}",
                self.place_phrase(origin, frame.cell),
                self.rain_reaches_phrase(frame.cell)
            ));
        }
        if lines.is_empty() {
            "You see no frame from here.".into()
        } else {
            lines.join("\n")
        }
    }

    fn condition_sentence(&self, moisture: u32, cell: CellId) -> String {
        format!(
            "It is {}. {}",
            self.condition_word(moisture),
            self.rain_reaches_phrase(cell)
        )
    }

    fn place_phrase(&self, from: CellId, to: CellId) -> String {
        if from == to {
            format!("here, {}", self.cover_phrase(to))
        } else {
            format!(
                "to the {}, {}",
                self.relative(from, to),
                self.cover_phrase(to)
            )
        }
    }

    fn map_string(&self, viewer: ActorId) -> String {
        let mut header = String::from("  ");
        for x in 0..crate::model::WIDTH {
            header.push_str(&format!("{x:<3}"));
        }
        let mut lines = vec![header];
        for y in 0..crate::model::HEIGHT {
            let mut line = format!("{y} ");
            for x in 0..crate::model::WIDTH {
                let [left, right] = self.glyph(viewer, cell_id(x, y).unwrap());
                line.push(left);
                line.push(right);
                line.push(' ');
            }
            lines.push(line);
        }
        lines.join("\n")
    }

    fn glyph(&self, viewer: ActorId, cell: CellId) -> [char; 2] {
        let origin = match self.actor(viewer) {
            Some(actor) => actor.cell,
            None => return ['?', '?'],
        };
        if chebyshev(origin, cell) > self.tuning.sight {
            return ['?', '?'];
        }
        let terrain = self.terrain_char(cell);
        let content = self.content_char(cell);
        let viewer_here = self.actor(viewer).map(|a| a.cell) == Some(cell);
        let others = self
            .actors
            .values()
            .any(|actor| actor.id != viewer && actor.cell == cell);
        if viewer_here {
            ['@', if content != ' ' { content } else { terrain }]
        } else if others {
            ['p', if content != ' ' { content } else { terrain }]
        } else {
            [terrain, content]
        }
    }

    fn terrain_char(&self, cell: CellId) -> char {
        if let Some(frame) = self.construction_at(cell) {
            if frame.bark.is_some() && self.supported_at_both_ends(frame.id) {
                return 'H';
            }
            return 'f';
        }
        let place = self.cell(cell);
        if place.rock {
            'R'
        } else {
            match place.region {
                crate::model::Region::Brook => '~',
                crate::model::Region::Slope => '^',
                crate::model::Region::Floor => '.',
            }
        }
    }

    fn content_char(&self, cell: CellId) -> char {
        let log_here = self.pile_count(cell, Material::Log) > 0
            || self.items.values().any(|item| {
                item.material == Material::Log && item.cell == cell && item.carrier.is_none()
            });
        if log_here {
            return 'L';
        }
        let branch_moisture = self.worst_loose(cell, Material::Branch);
        if self.pile_count(cell, Material::Branch) > 0 || branch_moisture.is_some() {
            let moisture = branch_moisture
                .unwrap_or(0)
                .max(self.pile_moisture(cell, Material::Branch));
            return if moisture >= self.tuning.damp_at {
                'W'
            } else {
                'w'
            };
        }
        if self.pile_count(cell, Material::Bark) > 0
            || self.items.values().any(|item| {
                item.material == Material::Bark
                    && item.role == Role::Loose
                    && item.carrier.is_none()
                    && item.cell == cell
            })
        {
            let moisture = self.pile_moisture(cell, Material::Bark);
            return if moisture >= self.tuning.damp_at {
                'B'
            } else {
                'b'
            };
        }
        ' '
    }

    fn worst_loose(&self, cell: CellId, material: Material) -> Option<u32> {
        self.items
            .values()
            .filter(|item| {
                item.cell == cell
                    && item.material == material
                    && item.role == Role::Loose
                    && item.carrier.is_none()
            })
            .map(|item| item.moisture)
            .max()
    }

    fn cell_view(&self, cell: CellId) -> CellView {
        let place = self.cell(cell);
        let (x, y) = cell_xy(cell);
        let loose_branches: Vec<u32> = self
            .items
            .values()
            .filter(|item| {
                item.cell == cell
                    && item.material == Material::Branch
                    && item.role == Role::Loose
                    && item.carrier.is_none()
            })
            .map(|item| item.moisture)
            .collect();
        let loose_bark = self
            .items
            .values()
            .filter(|item| {
                item.cell == cell
                    && item.material == Material::Bark
                    && item.role == Role::Loose
                    && item.carrier.is_none()
            })
            .count() as u32;
        let log = self
            .log_item()
            .filter(|item| item.cell == cell && item.carrier.is_none());
        let pile_log = self.pile_count(cell, Material::Log) > 0;
        let branch_pile = self.pile_count(cell, Material::Branch);
        let bark_pile = self.pile_count(cell, Material::Bark);
        let mut actors: Vec<String> = self
            .actors
            .values()
            .filter(|actor| actor.cell == cell)
            .map(|actor| actor.name.clone())
            .collect();
        actors.sort();
        let frame = match self.construction_at(cell) {
            None => "none",
            Some(frame) if frame.bark.is_some() && self.supported_at_both_ends(frame.id) => {
                "roofed"
            }
            Some(frame) if self.supported_at_both_ends(frame.id) => "supported",
            Some(_) => "partial",
        };
        let roof = if place.rock {
            "rock"
        } else if frame == "roofed" {
            "bark"
        } else {
            "none"
        };
        CellView {
            x,
            y,
            region: place.region.word().into(),
            under_roof: self.exposure_millis(cell) < 1000,
            roof: roof.into(),
            trail: place.trail,
            branch_pile,
            branch_pile_condition: (branch_pile > 0).then(|| {
                self.condition_word(self.pile_moisture(cell, Material::Branch))
                    .into()
            }),
            loose_branches: loose_branches.len() as u32,
            loose_branch_condition: loose_branches
                .iter()
                .max()
                .map(|moisture| self.condition_word(*moisture).into()),
            bark_pile,
            bark_pile_condition: (bark_pile > 0).then(|| {
                self.condition_word(self.pile_moisture(cell, Material::Bark))
                    .into()
            }),
            loose_bark,
            log: log.is_some() || pile_log,
            log_condition: log
                .map(|item| self.condition_word(item.moisture).into())
                .or_else(|| {
                    pile_log.then(|| {
                        self.condition_word(self.pile_moisture(cell, Material::Log))
                            .into()
                    })
                }),
            log_gripped_by: if log.is_some() {
                self.log_gripped_by()
            } else {
                Vec::new()
            },
            frame: frame.into(),
            actors,
        }
    }

    fn carry_views(&self, actor_id: ActorId) -> Vec<CarryView> {
        let mut groups: Vec<(Material, u32, u32)> = Vec::new();
        for item in self
            .items
            .values()
            .filter(|item| item.carrier == Some(actor_id) && item.role == Role::Loose)
        {
            if let Some(group) = groups
                .iter_mut()
                .find(|(material, _, _)| *material == item.material)
            {
                group.1 += 1;
                group.2 = group.2.max(item.moisture);
            } else {
                groups.push((item.material, 1, item.moisture));
            }
        }
        groups
            .into_iter()
            .map(|(material, count, moisture)| CarryView {
                material: material.word().into(),
                condition: self.condition_word(moisture).into(),
                count,
            })
            .collect()
    }

    fn work_view(&self, actor_id: ActorId) -> Option<WorkView> {
        let work_id = self.actor(actor_id)?.busy?;
        let work = self.works.get(&work_id)?;
        let (kind, detail) = match work.kind {
            crate::model::WorkKind::Drag => (
                "drag",
                format!(
                    "You are dragging the log {}. It finishes at tick {}.",
                    work.dir.map(|dir| dir.word()).unwrap_or("on"),
                    work.completes_at
                ),
            ),
            crate::model::WorkKind::RaiseFrame => (
                "raise-frame",
                format!(
                    "You are raising a frame (phase {} of {}). It continues at tick {}.",
                    work.phase_done + 1,
                    work.phases,
                    work.completes_at
                ),
            ),
            crate::model::WorkKind::LayBark => (
                "lay-bark",
                format!(
                    "You are laying bark. It finishes at tick {}.",
                    work.completes_at
                ),
            ),
        };
        Some(WorkView {
            kind: kind.into(),
            detail,
            finishes_at: work.completes_at,
        })
    }

    fn recent_events(&self, viewer: ActorId) -> Vec<EventView> {
        let origin = self.actor(viewer).map(|a| a.cell).unwrap_or(0);
        let mut views: Vec<EventView> = self
            .events
            .iter()
            .filter(|event| self.event_visible(viewer, origin, event))
            .map(|event| EventView {
                id: event.id,
                time: event.time,
                text: event.text.clone(),
            })
            .collect();
        if views.len() > 40 {
            views = views.split_off(views.len() - 40);
        }
        views
    }

    fn event_visible(&self, viewer: ActorId, origin: CellId, event: &crate::model::Event) -> bool {
        match event.scope {
            Scope::Valley => true,
            Scope::Personal(actor) => actor == viewer,
            Scope::Cell(cell) => {
                chebyshev(origin, cell) <= self.tuning.sight
                    || event.affected_actors.contains(&viewer)
            }
        }
    }

    fn allowed_actions(&self, actor_id: ActorId) -> Vec<String> {
        let Some(actor) = self.actor(actor_id) else {
            return Vec::new();
        };
        let mut actions = vec![
            "look".into(),
            "examine here".into(),
            "examine self".into(),
            "examine wood".into(),
            "examine bark".into(),
            "examine log".into(),
            "examine frame".into(),
            "say".into(),
            "help".into(),
        ];
        for other in self.visible_actor_ids(actor_id) {
            if other != actor_id {
                actions.push(format!("examine {}", self.name_of(other)));
            }
        }
        if self.cell(actor.cell).region == crate::model::Region::Brook && actor.busy.is_none() {
            actions.push("drink".into());
        }
        if actor.busy.is_some() {
            actions.push("interrupt".into());
            if self.gripping(actor_id).is_some() {
                actions.push("release".into());
            }
            return actions;
        }
        if self.gripping(actor_id).is_some() {
            actions.push("release".into());
            for dir in Dir::all() {
                let (x, y) = cell_xy(actor.cell);
                let (dx, dy) = dir.delta();
                if let Some(dest) = cell_id(x + dx, y + dy) {
                    if self.cell(dest).region != crate::model::Region::Brook {
                        actions.push(format!("drag {}", short_dir(dir)));
                    }
                }
            }
            return actions;
        }
        for dir in Dir::all() {
            let (x, y) = cell_xy(actor.cell);
            let (dx, dy) = dir.delta();
            if cell_id(x + dx, y + dy).is_some() {
                actions.push(short_dir(dir).into());
            }
        }
        if self.pile_count(actor.cell, Material::Branch) > 0
            && self.carry_count(actor_id) < self.tuning.carry_slots
        {
            actions.push("gather branch".into());
        }
        if self.pile_count(actor.cell, Material::Bark) > 0
            && self.carry_count(actor_id) < self.tuning.carry_slots
        {
            actions.push("gather bark".into());
        }
        if !self.loose_ids(actor_id, Material::Branch).is_empty() {
            actions.push("drop branch".into());
        }
        if !self.loose_ids(actor_id, Material::Bark).is_empty() {
            actions.push("drop bark".into());
        }
        if self.log_at(actor.cell) {
            actions.push("grab".into());
        }
        if self.loose_ids(actor_id, Material::Branch).len() >= 3
            && self.cell(actor.cell).region != crate::model::Region::Brook
            && !self.cell(actor.cell).rock
            && self.construction_at(actor.cell).is_none()
        {
            actions.push("raise frame".into());
        }
        if let Some(frame) = self.construction_at(actor.cell) {
            if self.supported_at_both_ends(frame.id)
                && frame.bark.is_none()
                && !self.loose_ids(actor_id, Material::Bark).is_empty()
            {
                actions.push("lay bark".into());
            }
        }
        actions
    }

    fn visible_cell_ids(&self, viewer: ActorId) -> Vec<CellId> {
        let Some(actor) = self.actor(viewer) else {
            return Vec::new();
        };
        self.cells
            .iter()
            .filter(|cell| chebyshev(actor.cell, cell.id) <= self.tuning.sight)
            .map(|cell| cell.id)
            .collect()
    }

    fn visible_actor_ids(&self, viewer: ActorId) -> Vec<ActorId> {
        let Some(actor) = self.actor(viewer) else {
            return Vec::new();
        };
        let mut ids: Vec<_> = self
            .actors
            .values()
            .filter(|other| chebyshev(actor.cell, other.cell) <= self.tuning.sight)
            .map(|other| other.id)
            .collect();
        ids.sort();
        ids
    }

    fn others_here(&self, viewer: ActorId) -> Vec<ActorId> {
        let Some(actor) = self.actor(viewer) else {
            return Vec::new();
        };
        let mut ids: Vec<_> = self
            .actors
            .values()
            .filter(|other| other.id != viewer && other.cell == actor.cell)
            .map(|other| other.id)
            .collect();
        ids.sort();
        ids
    }

    fn join_names(&self, names: &[String]) -> String {
        match names.len() {
            0 => "nobody".into(),
            1 => names[0].clone(),
            2 => format!("{} and {}", names[0], names[1]),
            _ => format!(
                "{}, and {}",
                names[..names.len() - 1].join(", "),
                names[names.len() - 1]
            ),
        }
    }
}

fn pile_noun(material: Material, count: u32) -> &'static str {
    match (material, count == 1) {
        (Material::Branch, true) => "branch",
        (Material::Branch, false) => "branches",
        (Material::Bark, true) => "sheet of bark",
        (Material::Bark, false) => "sheets of bark",
        (Material::Log, _) => "log",
    }
}

fn short_dir(dir: Dir) -> &'static str {
    match dir {
        Dir::North => "n",
        Dir::East => "e",
        Dir::South => "s",
        Dir::West => "w",
    }
}

fn capitalize(word: &str) -> String {
    let mut chars = word.chars();
    match chars.next() {
        None => String::new(),
        Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
    }
}
