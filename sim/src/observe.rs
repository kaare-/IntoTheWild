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
B wet bark     * fire
y berries      Y wet berries
m mushrooms    q poison berries
k pot          u water
s stone         c clay
v bowl
d deer         \" tracks
R rock roof    H bark roof
f unfinished frame
~ brook    = still water
^ slope    . ground
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
    pub hunger: String,
    pub illness: String,
    pub energy: String,
    pub cold: String,
    pub wind: String,
    pub sunburn: String,
    pub mud: String,
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
    pub berry_pile: u32,
    pub berry_pile_condition: Option<String>,
    pub loose_berries: u32,
    pub poison_berry_pile: u32,
    pub mushroom_pile: u32,
    pub pot_pile: u32,
    pub stone_pile: u32,
    pub loose_stone: u32,
    pub clay_pile: u32,
    pub loose_clay: u32,
    pub bowl: u32,
    pub still: bool,
    pub frozen: bool,
    pub fire: bool,
    pub log: bool,
    pub log_condition: Option<String>,
    pub log_gripped_by: Vec<String>,
    pub frame: String,
    pub actors: Vec<String>,
    pub deer: bool,
    pub tracks: bool,
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
                hunger: self.hunger_word(actor.hunger).into(),
                illness: self.illness_word(actor.illness).into(),
                energy: self.energy_word(actor.energy).into(),
                cold: self.cold_word(actor.cold).into(),
                wind: self.wind_word(self.exposure_millis(actor.cell)).into(),
                sunburn: self.sun_word(actor.sunburn).into(),
                mud: self.mud_word(actor.mud).into(),
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
            ExamineTarget::Berries => self.examine_material(actor_id, Material::Berry),
            ExamineTarget::BitterBerries => self.examine_material(actor_id, Material::BitterBerry),
            ExamineTarget::Mushrooms => self.examine_material(actor_id, Material::Mushroom),
            ExamineTarget::Pot => self.examine_material(actor_id, Material::Pot),
            ExamineTarget::Mud => self.examine_material(actor_id, Material::Mud),
            ExamineTarget::Stone => self.examine_material(actor_id, Material::Stone),
            ExamineTarget::Clay => self.examine_clay(actor_id),
            ExamineTarget::Bowl => self.examine_bowl(actor_id),
            ExamineTarget::Fire => self.examine_fire(actor_id),
            ExamineTarget::Water => self.examine_water(actor_id),
            ExamineTarget::Deer => self.examine_deer(actor_id),
            ExamineTarget::Tracks => self.examine_tracks(actor_id),
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
        if let Some(season) = self.season_line(actor_id) {
            lines.push(season);
        }
        if let Some(rot) = self.rot_line(actor_id) {
            lines.push(rot);
        }
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

    fn season_line(&self, viewer: ActorId) -> Option<String> {
        if !self.wet_season || !self.raining {
            return None;
        }
        let sees_open_wood = self.visible_cell_ids(viewer).iter().any(|cell| {
            self.exposure_millis(*cell) >= 1000
                && (self.pile_count(*cell, Material::Branch) > 0
                    || self.pile_count(*cell, Material::Bark) > 0
                    || self.pile_count(*cell, Material::Log) > 0
                    || self.items.values().any(|item| {
                        item.cell == *cell
                            && item.carrier.is_none()
                            && item.role == Role::Loose
                            && matches!(
                                item.material,
                                Material::Branch | Material::Bark | Material::Log
                            )
                    }))
        });
        sees_open_wood.then(|| "The rain is heavier on the wood this season.".to_string())
    }

    fn rot_line(&self, viewer: ActorId) -> Option<String> {
        let sees = self.visible_cell_ids(viewer).iter().any(|cell| {
            self.exposure_millis(*cell) >= 1000
                && (self.pile_count(*cell, Material::Branch) > 0
                    || self.items.values().any(|item| {
                        item.cell == *cell
                            && item.carrier.is_none()
                            && item.role == Role::Loose
                            && item.material == Material::Branch
                    }))
        });
        sees.then(|| "Wood left in the open is rotting.".to_string())
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
        if cell.region == crate::model::Region::Brook {
            parts.push("The water here is raw.".into());
        }
        if cell.still && cell.region != crate::model::Region::Brook {
            if self.still_frozen(actor.cell) {
                parts.push("The still water is frozen.".into());
            } else if self.time >= self.tuning.chill_at && self.fire_at(actor.cell).is_some() {
                parts.push("The fire is keeping the still water from freezing.".into());
            } else {
                parts.push("Still water lies here. It is not the brook.".into());
            }
        }
        if self.fire_at(actor.cell).is_some() {
            parts.push("A small fire burns here. It warms anyone standing at it.".into());
        } else if self
            .fires
            .values()
            .any(|fire| chebyshev(actor.cell, fire.cell) == 1)
        {
            parts.push("The heat of the fire does not reach you.".into());
        }
        if self.pile_count(actor.cell, Material::Berry) > 0 {
            parts.push(format!(
                "Berries lie here, {}. They are good to eat raw.",
                self.condition_word(self.pile_moisture(actor.cell, Material::Berry))
            ));
        }
        if self.pile_count(actor.cell, Material::BitterBerry) > 0 {
            parts.push(format!(
                "Poison berries lie here, {}. They are poisonous.",
                self.condition_word(self.pile_moisture(actor.cell, Material::BitterBerry))
            ));
        }
        if self.pile_count(actor.cell, Material::Mushroom) > 0 {
            parts.push(format!(
                "Mushrooms lie here, {}. They must be cooked. Raw mushrooms do not feed you.",
                self.condition_word(self.pile_moisture(actor.cell, Material::Mushroom))
            ));
        }
        if self.pile_count(actor.cell, Material::Pot) > 0 {
            parts.push("A pot sits here.".into());
        }
        if self.pile_count(actor.cell, Material::Stone) > 0
            || !self.ground_ids(actor.cell, Material::Stone).is_empty()
        {
            parts.push("A loose stone lies here.".into());
        }
        let firing = self
            .ground_ids(actor.cell, Material::Clay)
            .into_iter()
            .any(|id| {
                self.item(id)
                    .and_then(|item| item.firing_ready_at)
                    .is_some()
            });
        let loose_clay = self.ground_ids(actor.cell, Material::Clay).len() as u32;
        let clay_pile = self.pile_count(actor.cell, Material::Clay);
        if firing {
            parts.push("A lump of clay is set by the fire. It is not hard yet.".into());
        }
        if clay_pile > 1 {
            parts.push("Clay lies on the bank here.".into());
        } else if clay_pile == 1 || (loose_clay > 0 && !firing) {
            parts.push("A lump of clay lies here. It is unfired.".into());
        }
        if !self.ground_ids(actor.cell, Material::Bowl).is_empty() {
            parts.push("A fired bowl sits here. It can boil water over a fire.".into());
        }
        if self
            .deer
            .as_ref()
            .is_some_and(|deer| deer.cell == actor.cell)
        {
            parts.push("A deer stands here.".into());
        }
        if self.tracks.contains(&actor.cell) {
            parts.push("Hoofprints mark the ground.".into());
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
            format!("You are {}.", self.hunger_word(actor.hunger)),
            format!("You are {}.", self.illness_word(actor.illness)),
            format!("You are {}.", self.energy_word(actor.energy)),
            format!("You are {}.", self.cold_word(actor.cold)),
            format!(
                "You are {}.",
                self.wind_word(self.exposure_millis(actor.cell))
            ),
            format!("Your skin is {}.", self.sun_word(actor.sunburn)),
            format!("Your skin is {}.", self.mud_word(actor.mud)),
        ];
        if actor.warm_from_fire && self.fire_at(actor.cell).is_none() {
            parts.push("The warmth of the fire is still on you.".into());
        }
        let carrying = self.carry_views(actor_id);
        if carrying.is_empty() {
            parts.push("Your arms are empty.".into());
        } else {
            let bits: Vec<String> = carrying
                .iter()
                .map(|item| {
                    if item.material == "stone" && item.count == 1 {
                        format!("a {} stone", item.condition)
                    } else if item.material == "bark" && item.count == 1 {
                        format!("a {} sheet of bark", item.condition)
                    } else if item.material == "pot" && item.count == 1 {
                        format!("a {} pot", item.condition)
                    } else if item.material == "clay" && item.count == 1 {
                        "a lump of clay".into()
                    } else if item.material == "bowl" && item.count == 1 {
                        "a fired bowl".into()
                    } else {
                        format!(
                            "{} {} carried by you ({})",
                            item.count, item.material, item.condition
                        )
                    }
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
            "{} is on the {}, {}. Their clothes are {}. They are {}. They are {}. They are {}. They are {}.",
            actor.name,
            self.cell(actor.cell).region.word(),
            self.cover_phrase(actor.cell),
            self.condition_word(actor.wetness),
            self.thirst_sentence(actor.thirst),
            self.hunger_word(actor.hunger),
            self.illness_word(actor.illness),
            self.energy_word(actor.energy)
        ) + &format!(
            " They are {}. They are {}. Their skin is {}. Their skin is {}. ({x},{y})",
            self.cold_word(actor.cold),
            self.wind_word(self.exposure_millis(actor.cell)),
            self.sun_word(actor.sunburn),
            self.mud_word(actor.mud)
        )
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
        if self.pile_count(dest, Material::Berry) > 0 {
            bits.push("berries".into());
        }
        if self.pile_count(dest, Material::BitterBerry) > 0 {
            bits.push("poison berries".into());
        }
        if self.pile_count(dest, Material::Mushroom) > 0 {
            bits.push("mushrooms".into());
        }
        if self.pile_count(dest, Material::Pot) > 0 {
            bits.push("a pot".into());
        }
        if self.pile_count(dest, Material::Stone) > 0
            || !self.ground_ids(dest, Material::Stone).is_empty()
        {
            bits.push("a stone".into());
        }
        if self.pile_count(dest, Material::Clay) > 0
            || !self.ground_ids(dest, Material::Clay).is_empty()
        {
            bits.push("clay".into());
        }
        if !self.ground_ids(dest, Material::Bowl).is_empty() {
            bits.push("a bowl".into());
        }
        if cell.still && cell.region != crate::model::Region::Brook {
            bits.push("still water".into());
        }
        if self.deer.as_ref().is_some_and(|deer| deer.cell == dest) {
            bits.push("a deer".into());
        }
        if self.tracks.contains(&dest) {
            bits.push("hoofprints".into());
        }
        if self.fire_at(dest).is_some() {
            bits.push("a fire".into());
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

    fn examine_clay(&self, viewer: ActorId) -> String {
        let origin = self.actor(viewer).unwrap().cell;
        let mut lines = Vec::new();
        for cell in self.visible_cell_ids(viewer) {
            let count = self.pile_count(cell, Material::Clay);
            if count > 1 {
                lines.push(format!(
                    "A pile of {count} lumps of clay is in contact with the ground {}. The clay is {}. {} It is unfired. It cannot boil water.",
                    self.place_phrase(origin, cell),
                    self.condition_word(self.pile_moisture(cell, Material::Clay)),
                    self.rain_reaches_phrase(cell)
                ));
            } else if count == 1 {
                lines.push(format!(
                    "A lump of clay lies in contact with the ground {}. It is unfired. It cannot boil water.",
                    self.place_phrase(origin, cell)
                ));
            }
            for id in self.ground_ids(cell, Material::Clay) {
                let firing = self
                    .item(id)
                    .and_then(|item| item.firing_ready_at)
                    .is_some();
                if firing {
                    lines.push(format!(
                        "A lump of clay is set by the fire {}. It is not hard yet.",
                        self.place_phrase(origin, cell)
                    ));
                } else {
                    lines.push(format!(
                        "A lump of clay lies in contact with the ground {}. It is unfired. It cannot boil water.",
                        self.place_phrase(origin, cell)
                    ));
                }
            }
        }
        for other in self.visible_actor_ids(viewer) {
            let ids = self.loose_ids(other, Material::Clay);
            if ids.is_empty() {
                continue;
            }
            let who = if other == viewer {
                "you".to_string()
            } else {
                self.name_of(other)
            };
            let cell = self.actor(other).unwrap().cell;
            if ids.len() == 1 {
                lines.push(format!(
                    "A lump of clay is carried by {who}, {}. It is unfired. It cannot boil water.",
                    self.cover_phrase(cell)
                ));
            } else {
                lines.push(format!(
                    "{} lumps of clay are carried by {who}, {}. They are unfired. They cannot boil water.",
                    ids.len(),
                    self.cover_phrase(cell)
                ));
            }
        }
        if lines.is_empty() {
            "You see no clay from here.".into()
        } else {
            lines.join("\n")
        }
    }

    fn examine_bowl(&self, viewer: ActorId) -> String {
        let origin = self.actor(viewer).unwrap().cell;
        let mut lines = Vec::new();
        for cell in self.visible_cell_ids(viewer) {
            for id in self.ground_ids(cell, Material::Bowl) {
                let moisture = self.item(id).map(|item| item.moisture).unwrap_or(0);
                lines.push(format!(
                    "A fired bowl lies in contact with the ground {}. It is {}. {} It can boil water over a fire.",
                    self.place_phrase(origin, cell),
                    self.condition_word(moisture),
                    self.rain_reaches_phrase(cell)
                ));
            }
        }
        for other in self.visible_actor_ids(viewer) {
            let ids = self.loose_ids(other, Material::Bowl);
            if ids.is_empty() {
                continue;
            }
            let who = if other == viewer {
                "you".to_string()
            } else {
                self.name_of(other)
            };
            let cell = self.actor(other).unwrap().cell;
            for _ in &ids {
                lines.push(format!(
                    "A fired bowl is carried by {who}, {}. It can boil water over a fire.",
                    self.cover_phrase(cell)
                ));
            }
        }
        if lines.is_empty() {
            "You see no bowl from here.".into()
        } else {
            lines.join("\n")
        }
    }

    fn examine_material(&self, viewer: ActorId, material: Material) -> String {
        let origin = self.actor(viewer).unwrap().cell;
        let mut lines = Vec::new();
        for cell in self.visible_cell_ids(viewer) {
            let count = self.pile_count(cell, material);
            if count > 0 {
                if material == Material::Stone && count == 1 {
                    lines.push(format!(
                        "A stone lies in contact with the ground {}. The stone is {}. {}{}",
                        self.place_phrase(origin, cell),
                        self.condition_word(self.pile_moisture(cell, material)),
                        self.rain_reaches_phrase(cell),
                        food_clause(material)
                    ));
                } else if material == Material::Pot && count == 1 {
                    lines.push(format!(
                        "A pot lies in contact with the ground {}. The pot is {}. {} It can boil water over a fire.",
                        self.place_phrase(origin, cell),
                        self.condition_word(self.pile_moisture(cell, material)),
                        self.rain_reaches_phrase(cell)
                    ));
                } else {
                    let noun = pile_noun(material, count);
                    let (kind, verb) = food_kind(material);
                    lines.push(format!(
                        "A pile of {count} {noun} is in contact with the ground {}. The {kind} {verb} {}. {}{}{}{}",
                        self.place_phrase(origin, cell),
                        self.condition_word(self.pile_moisture(cell, material)),
                        self.rain_reaches_phrase(cell),
                        self.heavier_rain_clause(cell, material),
                        self.rotting_clause(cell, material),
                        food_clause(material)
                    ));
                }
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
                if material == Material::Stone && loose.len() == 1 {
                    lines.push(format!(
                        "A stone lies in contact with the ground {}. The stone is {}. {}{}",
                        self.place_phrase(origin, cell),
                        self.condition_word(worst),
                        self.rain_reaches_phrase(cell),
                        food_clause(material)
                    ));
                } else if material == Material::Pot && loose.len() == 1 {
                    lines.push(format!(
                        "A pot lies in contact with the ground {}. The pot is {}. {} It can boil water over a fire.",
                        self.place_phrase(origin, cell),
                        self.condition_word(worst),
                        self.rain_reaches_phrase(cell)
                    ));
                } else {
                    lines.push(format!(
                        "{} loose {} {} in contact with the ground {}. {}{}{}{}",
                        loose.len(),
                        pile_noun(material, loose.len() as u32),
                        if loose.len() == 1 { "is" } else { "are" },
                        self.place_phrase(origin, cell),
                        self.condition_sentence(worst, cell),
                        self.heavier_rain_clause(cell, material),
                        self.rotting_clause(cell, material),
                        food_clause(material)
                    ));
                }
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
            if material == Material::Stone && ids.len() == 1 {
                lines.push(format!(
                    "A {} stone is carried by {who}, {}, and it is a found stone.",
                    self.condition_word(moisture),
                    self.cover_phrase(cell)
                ));
            } else if material == Material::Bark && ids.len() == 1 {
                lines.push(format!(
                    "A {} sheet of bark is carried by {who}, {}.",
                    self.condition_word(moisture),
                    self.cover_phrase(cell)
                ));
            } else if material == Material::Pot && ids.len() == 1 {
                lines.push(format!(
                    "A {} pot is carried by {who}, {}, and it can boil water over a fire.",
                    self.condition_word(moisture),
                    self.cover_phrase(cell)
                ));
            } else {
                lines.push(format!(
                    "{} {} {} carried by {who}, {}. {}{}",
                    ids.len(),
                    pile_noun(material, ids.len() as u32),
                    if ids.len() == 1 { "is" } else { "are" },
                    self.cover_phrase(cell),
                    self.condition_word(moisture),
                    food_clause(material)
                ));
            }
        }
        if lines.is_empty() {
            let name = match material {
                Material::Branch => "wood",
                Material::Bark => "bark",
                Material::Log => "log",
                Material::Berry => "berries",
                Material::BitterBerry => "poison berries",
                Material::Mud => "mud",
                Material::Mushroom => "mushrooms",
                Material::Pot => "pot",
                Material::Stone => "stone",
                Material::Clay => "clay",
                Material::Bowl => "bowl",
                Material::Water => "water",
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

    fn examine_fire(&self, viewer: ActorId) -> String {
        let origin = self.actor(viewer).unwrap().cell;
        let mut lines = Vec::new();
        for fire in self.fires.values() {
            if chebyshev(origin, fire.cell) > self.tuning.sight {
                continue;
            }
            let (x, y) = cell_xy(fire.cell);
            let reach = if chebyshev(origin, fire.cell) == 1 {
                " The heat does not reach you from here."
            } else {
                ""
            };
            lines.push(format!(
                "A small fire burns {}. It was lit on this ground. It warms anyone standing at it.{reach} ({x},{y})",
                self.place_phrase(origin, fire.cell)
            ));
        }
        if lines.is_empty() {
            "You see no fire from here.".into()
        } else {
            lines.join("\n")
        }
    }

    fn examine_water(&self, viewer: ActorId) -> String {
        let origin = self.actor(viewer).unwrap().cell;
        let mut lines = Vec::new();
        for cell in self.visible_cell_ids(viewer) {
            let place = self.cell(cell);
            if place.region == crate::model::Region::Brook {
                lines.push(format!(
                    "The brook {} is running water. It does not freeze. Drinking it turns your stomach only occasionally. Boiled in a pot over a fire, it does not.",
                    self.place_phrase(origin, cell)
                ));
            } else if place.still && self.still_frozen(cell) {
                lines.push(format!(
                    "Still water {} is frozen. A fire on it would keep it open.",
                    self.place_phrase(origin, cell)
                ));
            } else if place.still && self.time >= self.tuning.chill_at {
                lines.push(format!(
                    "Still water {} is open because a fire is on it. Drinking it turns your stomach almost every time.",
                    self.place_phrase(origin, cell)
                ));
            } else if place.still {
                lines.push(format!(
                    "Still water {} does not run. Drinking it turns your stomach almost every time. Boiled in a pot over a fire, it does not.",
                    self.place_phrase(origin, cell)
                ));
            }
        }
        let carried = self.examine_material(viewer, Material::Water);
        if !carried.starts_with("You see no") {
            lines.push(carried);
        }
        if lines.is_empty() {
            "You see no water from here.".into()
        } else {
            lines.join("\n")
        }
    }

    fn examine_deer(&self, viewer: ActorId) -> String {
        let origin = self.actor(viewer).unwrap().cell;
        let Some(deer) = self.deer.as_ref() else {
            return "You cannot see a deer from here.".into();
        };
        if chebyshev(origin, deer.cell) > self.tuning.sight {
            return "You cannot see a deer from here.".into();
        }
        let eating = if self.pile_count(deer.cell, Material::Berry) > 0 {
            " It is eating the berries."
        } else {
            ""
        };
        format!(
            "A deer stands {}.{}",
            self.place_phrase(origin, deer.cell),
            eating
        )
    }

    fn examine_tracks(&self, viewer: ActorId) -> String {
        let origin = self.actor(viewer).unwrap().cell;
        let mut lines = Vec::new();
        for cell in self.visible_cell_ids(viewer) {
            if self.tracks.contains(&cell) {
                lines.push(format!(
                    "Hoofprints mark the ground {}.",
                    self.place_phrase(origin, cell)
                ));
            }
        }
        if lines.is_empty() {
            "You see no hoofprints from here.".into()
        } else {
            lines.join("\n")
        }
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

    fn heavier_rain_clause(&self, cell: CellId, material: Material) -> &'static str {
        if self.wet_season
            && self.raining
            && self.exposure_millis(cell) >= 1000
            && matches!(material, Material::Branch | Material::Bark | Material::Log)
        {
            " The rain is heavier on it this season."
        } else {
            ""
        }
    }

    fn rotting_clause(&self, cell: CellId, material: Material) -> &'static str {
        if material == Material::Branch && self.exposure_millis(cell) >= 1000 {
            " It is rotting in the open."
        } else {
            ""
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
        } else if place.still {
            '='
        } else {
            match place.region {
                crate::model::Region::Brook => '~',
                crate::model::Region::Slope => '^',
                crate::model::Region::Floor => '.',
            }
        }
    }

    fn content_char(&self, cell: CellId) -> char {
        if self.fire_at(cell).is_some() {
            return '*';
        }
        if self.deer.as_ref().is_some_and(|deer| deer.cell == cell) {
            return 'd';
        }
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
        let berry_moisture = self.worst_loose(cell, Material::Berry);
        if self.pile_count(cell, Material::Berry) > 0 || berry_moisture.is_some() {
            let moisture = berry_moisture
                .unwrap_or(0)
                .max(self.pile_moisture(cell, Material::Berry));
            return if moisture >= self.tuning.damp_at {
                'Y'
            } else {
                'y'
            };
        }
        if self.pile_count(cell, Material::Mushroom) > 0
            || self.worst_loose(cell, Material::Mushroom).is_some()
        {
            return 'm';
        }
        if self.pile_count(cell, Material::BitterBerry) > 0
            || self.worst_loose(cell, Material::BitterBerry).is_some()
        {
            return 'q';
        }
        if self.pile_count(cell, Material::Pot) > 0
            || self.worst_loose(cell, Material::Pot).is_some()
        {
            return 'k';
        }
        if self.pile_count(cell, Material::Water) > 0
            || self.items.values().any(|item| {
                item.material == Material::Water
                    && item.role == Role::Loose
                    && item.carrier.is_none()
                    && item.cell == cell
            })
        {
            return 'u';
        }
        if self.pile_count(cell, Material::Stone) > 0
            || self.worst_loose(cell, Material::Stone).is_some()
        {
            return 's';
        }
        if self.pile_count(cell, Material::Clay) > 0
            || self.worst_loose(cell, Material::Clay).is_some()
        {
            return 'c';
        }
        if self.worst_loose(cell, Material::Bowl).is_some() {
            return 'v';
        }
        if self.tracks.contains(&cell) {
            return '"';
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
        let berry_pile = self.pile_count(cell, Material::Berry);
        let loose_berries = self
            .items
            .values()
            .filter(|item| {
                item.cell == cell
                    && item.material == Material::Berry
                    && item.role == Role::Loose
                    && item.carrier.is_none()
            })
            .count() as u32;
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
            berry_pile,
            berry_pile_condition: (berry_pile > 0).then(|| {
                self.condition_word(self.pile_moisture(cell, Material::Berry))
                    .into()
            }),
            loose_berries,
            poison_berry_pile: self.pile_count(cell, Material::BitterBerry),
            mushroom_pile: self.pile_count(cell, Material::Mushroom),
            pot_pile: self.pile_count(cell, Material::Pot),
            stone_pile: self.pile_count(cell, Material::Stone),
            loose_stone: self.ground_ids(cell, Material::Stone).len() as u32,
            clay_pile: self.pile_count(cell, Material::Clay),
            loose_clay: self.ground_ids(cell, Material::Clay).len() as u32,
            bowl: self.ground_ids(cell, Material::Bowl).len() as u32,
            still: place.still,
            frozen: self.still_frozen(cell),
            fire: self.fire_at(cell).is_some(),
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
            deer: self.deer.as_ref().is_some_and(|deer| deer.cell == cell),
            tracks: self.tracks.contains(&cell),
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
            "examine berries".into(),
            "examine poison berries".into(),
            "examine mud".into(),
            "examine mushrooms".into(),
            "examine pot".into(),
            "examine fire".into(),
            "examine water".into(),
            "say".into(),
            "help".into(),
        ];
        if self
            .deer
            .as_ref()
            .is_some_and(|deer| chebyshev(actor.cell, deer.cell) <= self.tuning.sight)
        {
            actions.push("examine deer".into());
        }
        if self
            .visible_cell_ids(actor_id)
            .iter()
            .any(|cell| self.tracks.contains(cell))
        {
            actions.push("examine tracks".into());
        }
        for other in self.visible_actor_ids(actor_id) {
            if other != actor_id {
                actions.push(format!("examine {}", self.name_of(other)));
            }
        }
        if self.water_here(actor.cell) && !self.still_frozen(actor.cell) && actor.busy.is_none() {
            actions.push("drink".into());
        }
        if actor.busy.is_none()
            && self.gripping(actor_id).is_none()
            && self.fire_at(actor.cell).is_some()
            && !self.loose_ids(actor_id, Material::Water).is_empty()
            && (!self.loose_ids(actor_id, Material::Pot).is_empty()
                || !self.loose_ids(actor_id, Material::Bowl).is_empty())
        {
            actions.push("boil".into());
        }
        if actor.busy.is_none()
            && self.gripping(actor_id).is_none()
            && self.fire_at(actor.cell).is_some()
            && !self.loose_ids(actor_id, Material::Clay).is_empty()
        {
            actions.push("set clay".into());
        }
        if actor.busy.is_none() && self.light_refusal(actor_id).is_none() {
            actions.push("light fire".into());
        }
        if actor.busy.is_none() && self.cook_refusal(actor_id).is_none() {
            actions.push("cook".into());
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
        if self.pile_count(actor.cell, Material::Berry) > 0
            && self.carry_count(actor_id) < self.tuning.carry_slots
        {
            actions.push("gather berries".into());
        }
        if self.has_material_at_hand(actor_id, Material::Berry) {
            actions.push("eat berries".into());
        }
        if self.pile_count(actor.cell, Material::BitterBerry) > 0
            && self.carry_count(actor_id) < self.tuning.carry_slots
        {
            actions.push("gather poison berries".into());
        }
        if self.pile_count(actor.cell, Material::Mushroom) > 0
            && self.carry_count(actor_id) < self.tuning.carry_slots
        {
            actions.push("gather mushrooms".into());
        }
        if self.pile_count(actor.cell, Material::Pot) > 0
            && self.carry_count(actor_id) < self.tuning.carry_slots
        {
            actions.push("gather pot".into());
        }
        if (self.pile_count(actor.cell, Material::Stone) > 0
            || !self.ground_ids(actor.cell, Material::Stone).is_empty())
            && self.carry_count(actor_id) < self.tuning.carry_slots
        {
            actions.push("gather stone".into());
        }
        if (self.pile_count(actor.cell, Material::Clay) > 0
            || !self.ground_ids(actor.cell, Material::Clay).is_empty())
            && self.carry_count(actor_id) < self.tuning.carry_slots
        {
            actions.push("gather clay".into());
        }
        if !self.ground_ids(actor.cell, Material::Bowl).is_empty()
            && self.carry_count(actor_id) < self.tuning.carry_slots
        {
            actions.push("gather bowl".into());
        }
        if self.water_here(actor.cell)
            && !self.still_frozen(actor.cell)
            && self.carry_count(actor_id) < self.tuning.carry_slots
        {
            actions.push("gather water".into());
        }
        if self.water_here(actor.cell) && self.carry_count(actor_id) < self.tuning.carry_slots {
            actions.push("gather mud".into());
        }
        if !self.loose_ids(actor_id, Material::Branch).is_empty() {
            actions.push("drop branch".into());
        }
        if !self.loose_ids(actor_id, Material::Bark).is_empty() {
            actions.push("drop bark".into());
        }
        if !self.loose_ids(actor_id, Material::Berry).is_empty() {
            actions.push("drop berries".into());
        }
        if !self.loose_ids(actor_id, Material::Water).is_empty() {
            actions.push("drop water".into());
        }
        if !self.loose_ids(actor_id, Material::BitterBerry).is_empty() {
            actions.push("drop poison berries".into());
        }
        if !self.loose_ids(actor_id, Material::Mushroom).is_empty() {
            actions.push("drop mushrooms".into());
        }
        if !self.loose_ids(actor_id, Material::Pot).is_empty() {
            actions.push("drop pot".into());
        }
        if !self.loose_ids(actor_id, Material::Stone).is_empty() {
            actions.push("drop stone".into());
        }
        if !self.loose_ids(actor_id, Material::Clay).is_empty() {
            actions.push("drop clay".into());
        }
        if !self.loose_ids(actor_id, Material::Bowl).is_empty() {
            actions.push("drop bowl".into());
        }
        if !self.loose_ids(actor_id, Material::Mud).is_empty() {
            actions.push("drop mud".into());
            actions.push("smear mud".into());
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
        (Material::Berry, true) => "handful of berries",
        (Material::Berry, false) => "handfuls of berries",
        (Material::BitterBerry, true) => "handful of poison berries",
        (Material::BitterBerry, false) => "handfuls of poison berries",
        (Material::Mushroom, true) => "mushroom",
        (Material::Mushroom, false) => "mushrooms",
        (Material::Pot, _) => "pot",
        (Material::Stone, _) => "stone",
        (Material::Clay, true) => "lump of clay",
        (Material::Clay, false) => "lumps of clay",
        (Material::Bowl, _) => "bowl",
        (Material::Mud, true) => "handful of mud",
        (Material::Mud, false) => "handfuls of mud",
        (Material::Water, true) => "drink of water",
        (Material::Water, false) => "drinks of water",
    }
}

fn food_kind(material: Material) -> (&'static str, &'static str) {
    match material {
        Material::Bark => ("bark", "is"),
        Material::Berry => ("sweet berries", "are"),
        Material::BitterBerry => ("poison berries", "are"),
        Material::Mud => ("mud", "is"),
        Material::Mushroom => ("mushrooms", "are"),
        Material::Pot => ("pot", "is"),
        Material::Stone => ("stone", "is"),
        Material::Clay => ("clay", "is"),
        Material::Bowl => ("bowl", "is"),
        Material::Water => ("water", "is"),
        _ => ("wood", "is"),
    }
}

fn food_clause(material: Material) -> &'static str {
    match material {
        Material::Berry => " They are good to eat raw.",
        Material::BitterBerry => " They are poisonous. Cooking does not make them food.",
        Material::Mud => " It can be smeared on skin against the sun.",
        Material::Mushroom => " They must be cooked. Raw mushrooms do not feed you.",
        Material::Pot => " It can boil water over a fire.",
        Material::Stone => " It is a found stone.",
        _ => "",
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
