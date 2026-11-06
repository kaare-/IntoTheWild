//! One observation function. Text, map, and JSON are views of the same
//! facts. Moisture numbers, tokens, and the cause log stay here.

use crate::model::{
    cell_id, cell_xy, chebyshev, ActorId, CellId, Dir, ExamineTarget, Material, Role, Scope, World,
    DEER_ROUTE,
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
v bowl         j spear
a carcass      + more than the letter shows
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
    pub carcass: u32,
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
                wetness: self.clothes_word(actor.wetness).into(),
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
            ExamineTarget::Spear => self.examine_spear(actor_id),
            ExamineTarget::Carcass => self.examine_carcass(actor_id),
            ExamineTarget::Fire => self.examine_fire(actor_id),
            ExamineTarget::Water => self.examine_water(actor_id),
            ExamineTarget::Deer => self.examine_deer(actor_id),
            ExamineTarget::Tracks => self.examine_tracks(actor_id),
            ExamineTarget::Rock => self.examine_rock(actor_id),
            ExamineTarget::Slope => self.examine_slope(actor_id),
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
        if let Some(deer) = self.deer_look_line(actor_id) {
            lines.push(deer);
        }
        if let Some(person) = self.person_on_the_trail_west_of_the_south_floor(actor_id) {
            lines.push(person);
        }
        if let Some(person) = self.person_on_the_berries_north_of_the_west_floor(actor_id) {
            lines.push(person);
        }
        lines.push("Around you:".into());
        for dir in Dir::all() {
            lines.push(self.adjacent_line(actor.cell, dir));
        }
        if let Some(ate) = self.deer_on_the_named_floor_ate(actor.cell) {
            lines.push(ate);
        } else if self.meal_follows_the_prints(actor.cell) {
            lines.push("The deer ate the berries that were here.".into());
        }
        if self.deer_named_on_corner(actor.cell) {
            lines.push("The deer on that wood ate the berries.".into());
        }
        if self.deer_named_next_door(actor.cell) {
            lines.push("That deer ate the berries that were here.".into());
        }
        if self.prints_stay_on_the_wood(actor.cell) && self.deer_past_the_wood(actor.cell).is_none()
        {
            lines.push(self.prints_left_behind(actor.cell));
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
        let origin = self.actor(viewer)?.cell;
        let deer_on_the_empty_rock = self.open_wood_keeps_its_way(origin);
        let mut lines = Vec::new();
        for cell in self.visible_cell_ids(viewer) {
            let deer_on_it = self.deer.as_ref().is_some_and(|deer| deer.cell == cell);
            if self.open_loose_branches(cell) && !deer_on_it {
                let line = if self
                    .outside_floor(origin)
                    .is_some_and(|floor| floor.wood == cell)
                {
                    if self.last_branch_is_still_there(cell) {
                        format!(
                            "The last branch is rotting to the {}, and the deer left hoofprints on that ground.",
                            self.relative(origin, cell)
                        )
                    } else {
                        format!(
                            "Wood left in the open is rotting to the {}, and the deer left hoofprints on the rotting wood.",
                            self.relative(origin, cell)
                        )
                    }
                } else if let Some(side) = self.wood_south_of_the_outside_floor(origin, cell) {
                    format!(
                        "Wood left in the open is rotting {side} of the outside floor, and the deer left hoofprints on the rotting wood."
                    )
                } else if let Some(side) = self.wood_east_of_the_west_floor(origin, cell) {
                    format!(
                        "Wood left in the open is rotting {side} of the west floor, and the deer left hoofprints on the rotting wood."
                    )
                } else if deer_on_the_empty_rock {
                    format!(
                        "Wood left in the open is rotting to the {}.",
                        self.relative(origin, cell)
                    )
                } else {
                    let prints =
                        if !self.cell(origin).rock && origin != cell && self.tracks.contains(&cell)
                        {
                            ", and the deer left hoofprints on the rotting wood"
                        } else {
                            self.hoofprints_on_wood(origin, cell)
                        };
                    format!(
                        "Wood left in the open is rotting {}{}.",
                        self.cell_offset(origin, cell),
                        prints
                    )
                };
                if !lines.contains(&line) {
                    lines.push(line);
                }
            }
        }
        if lines.is_empty() {
            None
        } else {
            Some(lines.join("\n"))
        }
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
        let outside = self.outside_floor(actor.cell);
        let mut parts = Vec::new();
        if let Some(floor) = outside {
            parts.push("You stand on the outside floor.".into());
            parts.push("Hoofprints mark the ground.".into());
            parts.push(format!(
                "The rock overhang is the next square {}.",
                floor.rock_side
            ));
            let wood = if self.last_branch_is_still_there(floor.wood) {
                "The last branch"
            } else {
                "The rotting wood"
            };
            parts.push(format!("{wood} is the next square {}.", floor.wood_side));
        } else {
            parts.push(format!(
                "You are on the {}, {}.",
                cell.region.word(),
                self.cover_phrase(actor.cell)
            ));
            if let Some(side) = self.outside_floor_next_door(actor.cell) {
                parts.push(format!("The outside floor is the next square {side}."));
            }
        }
        if cell.trail {
            parts.push("A deer trail passes here.".into());
        }
        if cell.rock {
            parts.push(self.rock_overhead(actor_id));
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
        let deer_here = self
            .deer
            .as_ref()
            .is_some_and(|deer| deer.cell == actor.cell);
        let berries_here = self.pile_count(actor.cell, Material::Berry) > 0;
        match self.deer_meal_this_tick(actor.cell) {
            Some(true) if deer_here => {
                let names_the_empty_pile = !berries_here
                    && self.spent_berries.contains(&actor.cell)
                    && self.deer_ate_berries.contains(&actor.cell);
                if !names_the_empty_pile {
                    parts.push("The last handful is gone.".into());
                }
            }
            Some(false) if deer_here => parts.push(self.eating_handful_line(actor.cell, true)),
            _ if deer_here && berries_here => {
                parts.push("A deer stands on these branches.".into());
            }
            _ if berries_here => {
                parts.push(format!(
                    "Berries lie here, {}. They are good to eat raw.",
                    self.condition_word(self.pile_moisture(actor.cell, Material::Berry))
                ));
            }
            _ => {}
        }
        if !berries_here && self.spent_berries.contains(&actor.cell) {
            if self.deer_ate_berries.contains(&actor.cell) {
                if deer_here {
                    parts.push("The deer standing here ate the berries that were here.".into());
                } else if let Some(deer_cell) = self.deer.as_ref().and_then(|deer| {
                    (chebyshev(actor.cell, deer.cell) <= self.tuning.sight).then_some(deer.cell)
                }) {
                    if self.deer_on_the_named_floor_ate(actor.cell).is_none() {
                        if self.direction_names_deer(actor.cell, deer_cell)
                            || self.neighbor_line_names_deer(actor.cell, deer_cell)
                            || self.south_line_stands_the_deer(actor.cell)
                        {
                            parts.push("The berries that were here are gone.".into());
                        } else if self.open_loose_branches(deer_cell) {
                            let place = self.cell_offset(actor.cell, deer_cell);
                            parts.push(format!(
                                "The berries that were here are gone. The deer {place} ate them."
                            ));
                            parts.push(format!(
                                "The deer stands to the {}.",
                                self.relative(actor.cell, deer_cell)
                            ));
                        } else if self.deer_stands_past_the_wood(actor.cell) {
                            parts.push(
                                "The berries that were here are gone. The deer ate them.".into(),
                            );
                        } else {
                            let place = self.cell_offset(actor.cell, deer_cell);
                            let cover = self.cover_phrase(deer_cell);
                            parts.push(format!(
                                "The berries that were here are gone. The deer {place} ate them and stands there, {cover}."
                            ));
                        }
                    }
                } else if !self.meal_follows_the_prints(actor.cell) {
                    parts.push("A deer ate the berries that were here.".into());
                }
            } else {
                parts.push("The berries that were here are gone.".into());
            }
        }
        let still_eating = self.deer_meal_this_tick(actor.cell) == Some(false);
        let deer_named_standing_here = deer_here
            && !berries_here
            && self.spent_berries.contains(&actor.cell)
            && self.deer_ate_berries.contains(&actor.cell);
        if deer_here
            && !berries_here
            && !self.open_loose_branches(actor.cell)
            && !still_eating
            && !deer_named_standing_here
        {
            // A hoofprint is stored on the cell the deer leaves. While the
            // deer stands on a trail, the marks underfoot, if any, were stored
            // on an earlier pass. They are not a new step. The trail was
            // already named in this line.
            if cell.trail {
                parts.push("A deer stands on that trail.".into());
            } else {
                parts.push("A deer stands here.".into());
            }
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
        if self.pile_count(actor.cell, Material::Pot) > 0
            || !self.ground_ids(actor.cell, Material::Pot).is_empty()
        {
            parts.push("A pot sits here.".into());
        }
        let branches_here = self.pile_count(actor.cell, Material::Branch)
            + self.ground_ids(actor.cell, Material::Branch).len() as u32;
        let deer_on_this_wood = deer_here && self.open_loose_branches(actor.cell);
        if !deer_on_this_wood && branches_here > 1 {
            parts.push("A pile of branches lies here.".into());
        } else if !deer_on_this_wood && branches_here == 1 {
            parts.push("A branch lies here.".into());
        }
        let bark_here = self.pile_count(actor.cell, Material::Bark)
            + self.ground_ids(actor.cell, Material::Bark).len() as u32;
        if bark_here > 1 {
            parts.push("A pile of bark lies here.".into());
        } else if bark_here == 1 {
            parts.push("A sheet of bark lies here.".into());
        }
        if self.pile_count(actor.cell, Material::Log) > 0
            || self.items.values().any(|item| {
                item.material == Material::Log && item.cell == actor.cell && item.carrier.is_none()
            })
        {
            parts.push("The heavy log lies here.".into());
        }
        let water_here = self.pile_count(actor.cell, Material::Water)
            + self.ground_ids(actor.cell, Material::Water).len() as u32;
        if water_here == 1 {
            parts.push("A drink of water lies here.".into());
        } else if water_here > 1 {
            parts.push("Water lies here.".into());
        }
        let mud_here = self.pile_count(actor.cell, Material::Mud)
            + self.ground_ids(actor.cell, Material::Mud).len() as u32;
        if mud_here == 1 {
            parts.push("A handful of mud lies here.".into());
        } else if mud_here > 1 {
            parts.push("Mud lies here.".into());
        }
        if self.pile_count(actor.cell, Material::Stone) > 0
            || !self.ground_ids(actor.cell, Material::Stone).is_empty()
        {
            parts.push("A loose stone lies here.".into());
        }
        let firing = self.clay_firing_count(actor.cell);
        let loose_clay = self.ground_ids(actor.cell, Material::Clay).len() as u32;
        let clay_pile = self.pile_count(actor.cell, Material::Clay);
        if firing == 1 {
            parts.push("A lump of clay is hardening in the fire.".into());
        } else if firing > 1 {
            parts.push("Lumps of clay are hardening in the fire.".into());
        }
        if clay_pile > 1 {
            parts.push("Clay lies on the bank here.".into());
        } else if clay_pile == 1 || (loose_clay > firing as u32) {
            parts.push("A lump of clay lies here. It is unfired.".into());
        }
        if !self.ground_ids(actor.cell, Material::Bowl).is_empty() {
            parts.push("A fired bowl sits here. It can boil water over a fire.".into());
        }
        if !self.ground_ids(actor.cell, Material::Carcass).is_empty() {
            parts.push("A deer carcass lies here.".into());
        }
        for id in self.ground_ids(actor.cell, Material::Spear) {
            let hardened = self.item(id).is_some_and(|item| item.hardened);
            if hardened {
                parts.push("A spear lies here. The point is hardened.".into());
            } else {
                parts.push("A spear lies here. The point is not hardened.".into());
            }
        }
        if self.tracks.contains(&actor.cell) && self.prints_stay_on_the_wood(actor.cell) {
            parts.push("The deer left hoofprints on the ground here.".into());
        } else if outside.is_none() && self.tracks.contains(&actor.cell) {
            parts.push("Hoofprints mark the ground.".into());
        }
        if !cell.rock {
            if let Some(company) = self.company_here(actor_id) {
                parts.push(company);
            }
        }
        parts.join(" ")
    }

    /// Under the rock the other person is named with the overhang, before the wood.
    fn rock_overhead(&self, actor_id: ActorId) -> String {
        let mut line = if self.raining {
            "Rock leans overhead, and the rain does not reach this ground.".to_string()
        } else {
            "Rock leans overhead.".to_string()
        };
        if let Some(company) = self.company_here(actor_id) {
            line.push(' ');
            line.push_str(&company);
        }
        line.push_str(if self.open_wood_off_the_rock_is_gone() {
            " The branches here stay out of the weather."
        } else {
            " Wood left here stays out of the weather."
        });
        line
    }

    /// The open pile south of the outside floor has rotted away. Tracks still mark it.
    fn open_wood_off_the_rock_is_gone(&self) -> bool {
        self.tracked_wood_floors()
            .is_some_and(|floors| !self.open_loose_branches(floors.wood))
    }

    fn company_here(&self, actor_id: ActorId) -> Option<String> {
        let others = self.others_here(actor_id);
        if others.is_empty() {
            None
        } else {
            Some(format!(
                "{} {} here.",
                self.names(&others),
                if others.len() == 1 { "is" } else { "are" }
            ))
        }
    }

    fn clothes_line(&self, actor_id: ActorId, moisture: u32) -> String {
        let word = self.clothes_word(moisture);
        if word == "wet" {
            "The rain wets your clothes.".into()
        } else if word == "dry"
            && self.raining
            && self
                .actor(actor_id)
                .is_some_and(|actor| self.exposure_millis(actor.cell) == 0)
        {
            "The rain does not wet your clothes.".into()
        } else {
            format!("Your clothes are {word}.")
        }
    }

    fn self_text(&self, actor_id: ActorId) -> String {
        let actor = self.actor(actor_id).unwrap();
        let mut parts = vec![
            self.clothes_line(actor_id, actor.wetness),
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
                    } else if item.material == "carcass" && item.count == 1 {
                        "a deer carcass".into()
                    } else if item.material == "spear" && item.count == 1 {
                        let hardened = self
                            .loose_ids(actor_id, Material::Spear)
                            .first()
                            .and_then(|id| self.item(*id))
                            .is_some_and(|spear| spear.hardened);
                        if hardened {
                            "a spear with a hardened point".into()
                        } else {
                            "a spear with an unhardened point".into()
                        }
                    } else if item.material == "mushroom" {
                        if item.count == 1 {
                            format!("a {} mushroom", item.condition)
                        } else {
                            format!("{} {} mushrooms", count_word(item.count), item.condition)
                        }
                    } else if item.material == "berry" {
                        if item.count == 1 {
                            format!("a {} handful of berries", item.condition)
                        } else {
                            format!(
                                "{} {} handfuls of berries",
                                count_word(item.count),
                                item.condition
                            )
                        }
                    } else if item.material == "poison berry" {
                        if item.count == 1 {
                            format!("a {} handful of poison berries", item.condition)
                        } else {
                            format!(
                                "{} {} handfuls of poison berries",
                                count_word(item.count),
                                item.condition
                            )
                        }
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
        if self.spear_is_hardening(actor_id) {
            parts.push("The point of your spear is hardening in the fire.".into());
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
            self.clothes_word(actor.wetness),
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
        ) + &self.spear_clause(actor_id)
            + &self.carcass_clause(actor_id)
    }

    fn carcass_clause(&self, actor_id: ActorId) -> String {
        if self.loose_ids(actor_id, Material::Carcass).is_empty() {
            String::new()
        } else {
            " They carry a deer carcass.".into()
        }
    }

    fn spear_clause(&self, actor_id: ActorId) -> String {
        let Some(id) = self.loose_ids(actor_id, Material::Spear).first().copied() else {
            return String::new();
        };
        let hardened = self.item(id).is_some_and(|item| item.hardened);
        if hardened {
            " They carry a spear with a hardened point.".into()
        } else if self.spear_is_hardening(actor_id) {
            " They carry a spear with an unhardened point. The point is hardening in the fire."
                .into()
        } else {
            " They carry a spear with an unhardened point.".into()
        }
    }

    fn spear_is_hardening(&self, actor_id: ActorId) -> bool {
        let Some(actor) = self.actor(actor_id) else {
            return false;
        };
        if self.fire_at(actor.cell).is_none() {
            return false;
        }
        self.loose_ids(actor_id, Material::Spear).iter().any(|id| {
            self.item(*id)
                .is_some_and(|spear| !spear.hardened && spear.firing_ready_at.is_some())
        })
    }

    fn clay_firing_count(&self, cell: CellId) -> usize {
        self.ground_ids(cell, Material::Clay)
            .into_iter()
            .filter(|id| {
                self.item(*id)
                    .and_then(|item| item.firing_ready_at)
                    .is_some()
            })
            .count()
    }

    /// A deer on a cell in sight that the four neighbor lines do not name.
    /// Due north, south, east, or west of the next cell is already "a deer"
    /// on that line. A diagonal, or a deer farther off, is this sentence.
    fn open_loose_branches(&self, cell: CellId) -> bool {
        self.exposure_millis(cell) >= 1000
            && (self.pile_count(cell, Material::Branch) > 0
                || self.items.values().any(|item| {
                    item.cell == cell
                        && item.carrier.is_none()
                        && item.role == Role::Loose
                        && item.material == Material::Branch
                }))
    }

    fn deer_look_line(&self, viewer: ActorId) -> Option<String> {
        let origin = self.actor(viewer)?.cell;
        let deer = self.deer.as_ref()?;
        if let Some(floor) = self.outside_floor(origin) {
            if let Some(way) = orthogonal_step(floor.wood, deer.cell) {
                if self.first_cell_off_this_wood(floor.wood) == Some(deer.cell) {
                    return Some(format!("A deer stands on the {way} floor."));
                }
            }
        }
        if self.deer_stands_on_the_south_floor_from_here(origin) {
            return None;
        }
        if let Some(line) = self.deer_on_the_trail_past_the_south_floor(viewer) {
            return Some(line);
        }
        if chebyshev(origin, deer.cell) > self.tuning.sight {
            return None;
        }
        if self.south_line_stands_the_deer(origin) {
            return None;
        }
        if origin != deer.cell {
            if let Some(last) = self.deer_meal_this_tick(deer.cell) {
                if last {
                    return Some("The last handful is gone.".into());
                }
                return Some(self.eating_handful_line(deer.cell, false));
            }
        }
        if origin != deer.cell
            && self.pile_count(deer.cell, Material::Branch) > 0
            && self.pile_count(deer.cell, Material::Berry) > 0
            && self.deer_meal_this_tick(deer.cell).is_none()
        {
            return Some(format!(
                "A deer stands on these branches {}.",
                self.mark_place(origin, deer.cell)
            ));
        }
        if self.open_loose_branches(deer.cell) {
            if self.look_already_turns_toward(origin, deer.cell) {
                return None;
            }
            return Some(format!(
                "A deer stands {}{}.",
                self.deer_place(origin, deer.cell),
                self.off_rock_wood_hoofprints(viewer, origin, deer.cell)
            ));
        }
        if origin == deer.cell {
            return None;
        }
        let (x1, y1) = cell_xy(origin);
        let (x2, y2) = cell_xy(deer.cell);
        let dx = x2 - x1;
        let dy = y2 - y1;
        let next_door = (dx == 0 && dy.abs() == 1) || (dy == 0 && dx.abs() == 1);
        if next_door {
            return None;
        }
        if self.spent_berries.contains(&origin)
            && self.deer_ate_berries.contains(&origin)
            && self.pile_count(origin, Material::Berry) == 0
            && self.deer_on_the_named_floor_ate(origin).is_none()
            && !self.direction_names_deer(origin, deer.cell)
            && !self.neighbor_line_names_deer(origin, deer.cell)
            && !self.south_line_stands_the_deer(origin)
        {
            return None;
        }
        if let Some(line) = self.deer_on_named_ground_from_far(origin) {
            return Some(line);
        }
        Some(format!(
            "A deer stands {}.",
            self.mark_place(origin, deer.cell)
        ))
    }

    fn adjacent_line(&self, origin: CellId, dir: Dir) -> String {
        let (x, y) = cell_xy(origin);
        let (dx, dy) = dir.delta();
        let Some(dest) = cell_id(x + dx, y + dy) else {
            return format!("{}: the valley ends.", capitalize(dir.word()));
        };
        if let Some(floor) = self.outside_floor(origin) {
            if dest == floor.wood {
                if self.last_branch_is_still_there(dest) {
                    return format!("{}: the last branch.", capitalize(dir.word()));
                }
                return format!(
                    "{}: the rotting wood. Hoofprints mark that wood.",
                    capitalize(dir.word())
                );
            }
            if dir.word() == floor.rock_side && self.cell(dest).rock {
                if let Some(name) = self.name_under_that_rock(origin, origin, floor.rock_side) {
                    return self.rock_from_the_outside_floor(dir, dest, &name);
                }
            }
        }
        if let Some(floor) = self.outside_floor(dest) {
            if dir.word() == floor.rock_side {
                let mut line = format!(
                    "{}: the outside floor. Hoofprints mark that floor. The rock overhang is the next square {} of that floor.",
                    capitalize(dir.word()),
                    floor.rock_side
                );
                if let Some(name) = self.name_under_that_rock(origin, dest, floor.rock_side) {
                    line.push_str(&format!(" {name}"));
                }
                return line;
            }
        }
        if self.west_floor_left_to_the_south(origin, dest) {
            let mut line = format!(
                "{}: the west floor. Hoofprints mark that floor.",
                capitalize(dir.word())
            );
            if let Some(past) = self.floor_past_the_west_floor(origin, dest) {
                line.push_str(&past);
            }
            return line;
        }
        if let Some(line) = self.south_floor_to_the_east(origin, dest, dir) {
            return line;
        }
        let cell = self.cell(dest);
        let mut bits = vec![cell.region.word().to_string()];
        if cell.rock {
            bits.push("rock overhang".into());
        }
        if self.pile_count(dest, Material::Log) > 0 || self.log_item().map(|i| i.cell) == Some(dest)
        {
            bits.push("the heavy log".into());
        }
        let deer_on_this_wood = self
            .deer
            .as_ref()
            .is_some_and(|deer| deer.cell == dest && self.open_loose_branches(dest));
        let deer_just_arrived = self.deer.as_ref().is_some_and(|deer| {
            deer.cell == dest
                && self.pile_count(dest, Material::Branch) > 0
                && self.pile_count(dest, Material::Berry) > 0
                && self.deer_meal_this_tick(dest).is_none()
        });
        let deer_eating = self
            .deer
            .as_ref()
            .is_some_and(|deer| deer.cell == dest && self.deer_meal_this_tick(dest).is_some());
        if self.pile_count(dest, Material::Branch) > 0 && !deer_on_this_wood {
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
        if self.pile_count(dest, Material::Pot) > 0
            || !self.ground_ids(dest, Material::Pot).is_empty()
        {
            bits.push("a pot".into());
        }
        let bark_there = self.pile_count(dest, Material::Bark)
            + self.ground_ids(dest, Material::Bark).len() as u32;
        if bark_there == 1 {
            bits.push("a sheet of bark".into());
        } else if bark_there > 1 {
            bits.push("a pile of bark".into());
        }
        if self.pile_count(dest, Material::Stone) > 0
            || !self.ground_ids(dest, Material::Stone).is_empty()
        {
            bits.push("a stone".into());
        }
        let clay_firing = self.clay_firing_count(dest);
        if clay_firing > 0 && self.pile_count(dest, Material::Clay) == 0 {
            bits.push("clay hardening in the fire".into());
        } else if self.pile_count(dest, Material::Clay) > 0
            || !self.ground_ids(dest, Material::Clay).is_empty()
        {
            bits.push("clay".into());
        }
        if !self.ground_ids(dest, Material::Bowl).is_empty() {
            bits.push("a bowl".into());
        }
        if !self.ground_ids(dest, Material::Carcass).is_empty() {
            bits.push("a carcass".into());
        }
        if self
            .actors
            .values()
            .any(|person| person.cell == dest && self.spear_is_hardening(person.id))
        {
            bits.push("a spear hardening in the fire".into());
        }
        if self.ground_ids(dest, Material::Spear).len() == 1 {
            let hardened = self
                .ground_ids(dest, Material::Spear)
                .first()
                .and_then(|id| self.item(*id))
                .is_some_and(|spear| spear.hardened);
            if hardened {
                bits.push("a spear with a hardened point".into());
            } else {
                bits.push("a spear with an unhardened point".into());
            }
        } else if self.ground_ids(dest, Material::Spear).len() > 1 {
            bits.push("spears".into());
        }
        if cell.still && cell.region != crate::model::Region::Brook {
            bits.push("still water".into());
        }
        if self.deer.as_ref().is_some_and(|deer| deer.cell == dest)
            && !deer_on_this_wood
            && !deer_just_arrived
            && !deer_eating
        {
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
        if let Some(line) = self.deer_leads_this_way(origin, dir) {
            return line;
        }
        if let Some(side) = self.wood_south_of_these_prints(origin, dir, dest) {
            let wood = self
                .corner_wood(origin, dir)
                .and_then(|(cell, _)| self.last_branch_is_still_there(cell).then_some(cell));
            let name = if wood.is_some() {
                "The last branch"
            } else {
                "The rotting wood"
            };
            let mut line = format!(
                "{}: hoofprints mark the outside floor. {name} is the next square {side} of that floor.",
                capitalize(dir.word())
            );
            if let Some(past) = self.person_on_the_berries_off_this_floor(origin, dest) {
                line.push_str(&past);
            }
            return line;
        }
        if let Some(line) = self.outside_floor_after_the_wood_is_gone(origin, dir, dest) {
            return line;
        }
        let line = format!("{}: {}.", capitalize(dir.word()), bits.join(", "));
        match self.corner_beyond(origin, dir) {
            Some(corner) => format!("{line} {corner}"),
            None => line,
        }
    }

    /// The look at sim time 16, while one branch is still on the open wood.
    fn last_branch_is_still_there(&self, cell: CellId) -> bool {
        self.time == 16 && self.pile_count(cell, Material::Branch) == 1
    }

    /// The viewer is on the rock after the deer ate the berries there.
    fn open_wood_keeps_its_way(&self, origin: CellId) -> bool {
        self.cell(origin).rock
            && self.pile_count(origin, Material::Berry) == 0
            && self.spent_berries.contains(&origin)
            && self.deer_ate_berries.contains(&origin)
            && self.deer.is_some()
    }

    /// The deer has left the rock and the corner, and the prints stay on the wood.
    fn prints_stay_on_the_wood(&self, origin: CellId) -> bool {
        if !self.open_wood_keeps_its_way(origin) {
            return false;
        }
        let Some(deer) = &self.deer else {
            return false;
        };
        if deer.cell == origin {
            return false;
        }
        let (x, y) = cell_xy(origin);
        if cell_id(x - 1, y).is_some_and(|west| deer.cell == west) {
            return false;
        }
        Dir::all().into_iter().any(|dir| {
            self.corner_wood(origin, dir)
                .is_some_and(|(cell, _)| self.tracks.contains(&cell) && deer.cell != cell)
        })
    }

    /// At sim time 15 the prints name the walk after the meal. The meal
    /// comes first, once the walk continues past the south floor.
    fn meal_follows_the_prints(&self, origin: CellId) -> bool {
        if !self.open_wood_keeps_its_way(origin) {
            return false;
        }
        let Some(steps) = self.steps_off_the_wood(origin) else {
            return false;
        };
        if steps.len() < 3 || chebyshev(origin, steps[1]) > self.tuning.sight {
            return false;
        }
        let Some(wood) = Dir::all().into_iter().find_map(|dir| {
            self.corner_wood(origin, dir)
                .and_then(|(cell, _)| self.tracks.contains(&cell).then_some(cell))
        }) else {
            return false;
        };
        self.floor_west_of_this_wood(origin, wood) == Some(steps[0])
    }

    fn prints_left_behind(&self, origin: CellId) -> String {
        let placed = "The deer left hoofprints on that wood";
        let Some(steps) = self.steps_off_the_wood(origin) else {
            return format!("{placed}.");
        };
        if steps.len() < 2 {
            return format!("{placed}.");
        }
        let mut line = String::new();
        let mut prev = steps[0];
        let mut seen_floor: Option<&str> = None;
        for cell in steps.iter().copied().skip(1) {
            let Some(dir) = orthogonal_step(prev, cell) else {
                return format!("{placed}.");
            };
            let left_sight = chebyshev(origin, cell) > self.tuning.sight;
            let deer_here = self.deer.as_ref().is_some_and(|deer| deer.cell == cell);
            if let Some(from) = seen_floor {
                if left_sight {
                    let ground = self.cell(cell).region.word();
                    let same_walk = line.ends_with("still in sight.");
                    if same_walk {
                        let onto = format!(", onto the {from} floor, still in sight.");
                        if line.ends_with(&onto) {
                            line.truncate(line.len() - onto.len());
                            let marked = "mark the deer's walk";
                            if let Some(at) = line.find(marked) {
                                line.truncate(at + marked.len());
                                line.push_str(&format!(
                                    ", and on the {from} floor they turn {dir} out of sight."
                                ));
                            }
                        }
                        break;
                    }
                    line.push_str(&format!(
                        " The deer went on from the {from} floor onto the {ground} on the {from} floor's {dir} side"
                    ));
                    if self.exposure_millis(cell) >= 1000 {
                        line.push_str(", in the open");
                    }
                    let beside = Dir::all()
                        .into_iter()
                        .find(|way| way.word() == dir)
                        .and_then(|way| {
                            let (ox, oy) = cell_xy(origin);
                            let (dx, dy) = way.delta();
                            cell_id(ox + dx, oy + dy)
                        });
                    if beside.is_some_and(|beside| {
                        beside != cell && self.cell(beside).region.word() == ground
                    }) {
                        line.push_str(&format!(", not the {ground} one step to your {dir}"));
                    }
                    line.push('.');
                    if !same_walk {
                        let trail = self.cell(cell).trail;
                        let prints = self.tracks.contains(&cell);
                        if trail && prints && deer_here {
                            line.push_str(&format!(
                                " A deer trail passes on that {ground}, hoofprints mark that {ground}, and the deer stands on that {ground}, out of sight."
                            ));
                        } else if deer_here {
                            line.push_str(&format!(
                                " The deer stands on that {ground}, out of sight."
                            ));
                        } else {
                            line.push_str(" Out of sight.");
                        }
                    }
                    break;
                }
                line.push_str(&format!(
                    " The deer went on from the {from} floor onto the floor {dir} of the {from} floor."
                ));
            } else {
                let Some(wood_side) = Dir::all().into_iter().find_map(|way| {
                    let (wood, _) = self.corner_wood(origin, way)?;
                    if !self.tracks.contains(&wood) {
                        return None;
                    }
                    orthogonal_step(wood, prev)
                }) else {
                    return format!("{placed}.");
                };
                line = format!(
                    "Hoofprints on that wood mark the deer's walk {dir} off the {wood_side} floor, onto the {dir} floor"
                );
                if left_sight {
                    if deer_here {
                        line.push_str(", and the deer is on that floor, out of sight.");
                    } else {
                        line.push_str(", out of sight.");
                    }
                    break;
                }
                line.push_str(", still in sight.");
            }
            seen_floor = Some(dir);
            prev = cell;
        }
        if line.is_empty() {
            format!("{placed}.")
        } else {
            line
        }
    }

    /// Cells the deer stepped after the rotting wood, through the step that
    /// leaves sight. The first is the valley floor South already described.
    fn steps_off_the_wood(&self, origin: CellId) -> Option<Vec<CellId>> {
        let deer = self.deer.as_ref()?;
        let wood = Dir::all().into_iter().find_map(|dir| {
            self.corner_wood(origin, dir).and_then(|(cell, _)| {
                (self.tracks.contains(&cell) && deer.cell != cell).then_some(cell)
            })
        })?;
        let n = DEER_ROUTE.len();
        let (dx, dy) = cell_xy(deer.cell);
        let mut idx = if DEER_ROUTE.get(deer.route_at) == Some(&(dx, dy)) {
            deer.route_at
        } else {
            DEER_ROUTE
                .iter()
                .position(|&cell| cell == (dx, dy))
                .unwrap_or(deer.route_at)
        };
        let mut back = Vec::new();
        let mut found = false;
        for _ in 0..n {
            let (x, y) = DEER_ROUTE[idx];
            let cell = cell_id(x, y)?;
            if cell == wood {
                found = true;
                break;
            }
            back.push(cell);
            idx = (idx + n - 1) % n;
        }
        if !found || back.is_empty() {
            return None;
        }
        back.reverse();
        let mut steps = Vec::new();
        for cell in back {
            steps.push(cell);
            if chebyshev(origin, cell) > self.tuning.sight {
                break;
            }
        }
        Some(steps)
    }

    /// The deer is standing on the open floor just past this wood, still in sight.
    /// South names that animal. The place line does not count the cells.
    fn deer_stands_past_this_wood(
        &self,
        origin: CellId,
        wood: CellId,
    ) -> Option<(CellId, &'static str)> {
        if !self.open_wood_keeps_its_way(origin) {
            return None;
        }
        let deer = self.deer.as_ref()?;
        if deer.cell == wood || chebyshev(origin, deer.cell) > self.tuning.sight {
            return None;
        }
        let (wx, wy) = cell_xy(wood);
        let stood = cell_id(wx - 1, wy)?;
        if deer.cell != stood || self.first_cell_off_this_wood(wood) != Some(stood) {
            return None;
        }
        let way = orthogonal_step(wood, stood)?;
        Some((stood, way))
    }

    /// South will say where this deer stands, past the wood.
    fn deer_stands_past_the_wood(&self, origin: CellId) -> bool {
        self.deer_past_the_wood(origin).is_some()
    }

    /// The floor under the deer standing just past the wood, the name sim time 14 already uses.
    fn deer_past_the_wood(&self, origin: CellId) -> Option<String> {
        Dir::all().into_iter().find_map(|dir| {
            let (wood, _) = self.corner_wood(origin, dir)?;
            let named_earlier = Dir::all()
                .into_iter()
                .take_while(|earlier| *earlier != dir)
                .any(|earlier| {
                    self.corner_wood(origin, earlier)
                        .is_some_and(|(cell, _)| cell == wood)
                });
            if named_earlier {
                return None;
            }
            let (_, way) = self.deer_stands_past_this_wood(origin, wood)?;
            Some(format!("the {way} floor"))
        })
    }

    /// The prints on the square this way, then the deer past the wood.
    /// What is said of that square is still true if you are standing on it.
    fn deer_leads_this_way(&self, origin: CellId, dir: Dir) -> Option<String> {
        if !self.open_wood_keeps_its_way(origin) {
            return None;
        }
        let (corner, _) = self.corner_wood(origin, dir)?;
        let named_earlier = Dir::all()
            .into_iter()
            .take_while(|earlier| *earlier != dir)
            .any(|earlier| {
                self.corner_wood(origin, earlier)
                    .is_some_and(|(cell, _)| cell == corner)
            });
        if !named_earlier {
            return None;
        }
        let (_, way) = self.deer_stands_past_this_wood(origin, corner)?;
        let (x, y) = cell_xy(origin);
        let (dx, dy) = dir.delta();
        let dest = cell_id(x + dx, y + dy)?;
        if !self.tracks.contains(&dest) {
            return None;
        }
        let side = orthogonal_step(dest, corner)?;
        let mut line = format!(
            "{}: hoofprints mark the outside floor.",
            capitalize(dir.word())
        );
        if let Some(name) = self.name_standing_on(origin, dest) {
            line.push_str(&format!(" {name} stands on that floor."));
        }
        line.push_str(&format!(
            " The rotting wood is the next square {side} of that floor. Past that wood, the deer stands on the {way} floor, and hoofprints mark that floor."
        ));
        Some(line)
    }

    /// The outside floor stays in West after the pile is gone. The look says the wood is gone.
    /// Step is not named on the berries unless Step is standing there.
    fn outside_floor_after_the_wood_is_gone(
        &self,
        origin: CellId,
        dir: Dir,
        dest: CellId,
    ) -> Option<String> {
        if !self.open_wood_keeps_its_way(origin)
            || !self.tracks.contains(&dest)
            || self.cell(dest).rock
        {
            return None;
        }
        let (x, y) = cell_xy(dest);
        let (ox, oy) = cell_xy(origin);
        let (dx, dy) = dir.delta();
        if ox + dx != x || oy + dy != y {
            return None;
        }
        let mut woods = Vec::new();
        for (px, py) in [(0, -1), (0, 1), (-1, 0), (1, 0)] {
            let Some(wood) = cell_id(x + px, y + py) else {
                continue;
            };
            if wood == origin || self.cell(wood).rock || chebyshev(origin, wood) != 1 {
                continue;
            }
            if !self.tracks.contains(&wood) || self.open_loose_branches(wood) {
                continue;
            }
            woods.push(wood);
        }
        if woods.len() != 1 {
            return None;
        }
        let side = orthogonal_step(dest, woods[0])?;
        let mut line = format!(
            "{}: hoofprints mark the outside floor. The last branch is gone from the next square {side}. Hoofprints mark that ground.",
            capitalize(dir.word())
        );
        if let Some(name) = self.name_standing_on(origin, dest) {
            line.push_str(&format!(" {name} stands on that floor."));
        }
        if let Some(past) = self.person_on_the_berries_off_this_floor(origin, dest) {
            line.push_str(&past);
        }
        Some(line)
    }

    /// The same square, once the deer has left the west floor, at sim time 14 and 15.
    fn wood_south_of_these_prints(
        &self,
        origin: CellId,
        dir: Dir,
        dest: CellId,
    ) -> Option<&'static str> {
        if !self.tracks.contains(&dest) || !self.open_wood_keeps_its_way(origin) {
            return None;
        }
        let (corner, _) = self.corner_wood(origin, dir)?;
        let named_earlier = Dir::all()
            .into_iter()
            .take_while(|earlier| *earlier != dir)
            .any(|earlier| {
                self.corner_wood(origin, earlier)
                    .is_some_and(|(cell, _)| cell == corner)
            });
        if !named_earlier {
            return None;
        }
        let side = orthogonal_step(dest, corner)?;
        let entered = self.floor_west_of_this_wood(origin, corner)?;
        if self.deer_stands_on_next_floor(origin, entered).is_none()
            && self.next_floor_still_in_sight(origin, entered).is_none()
        {
            return None;
        }
        Some(side)
    }

    /// The one person standing on a cell in sight. The name is theirs.
    fn name_standing_on(&self, origin: CellId, cell: CellId) -> Option<String> {
        if chebyshev(origin, cell) > self.tuning.sight {
            return None;
        }
        let mut names: Vec<_> = self
            .actors
            .values()
            .filter(|actor| actor.cell == cell)
            .map(|actor| actor.name.clone())
            .collect();
        if names.len() != 1 {
            return None;
        }
        names.pop()
    }

    /// A person on the poison berries one step off the outside floor.
    /// "That floor" stays the outside floor. The deer is not named here.
    fn person_on_the_berries_off_this_floor(
        &self,
        origin: CellId,
        floor: CellId,
    ) -> Option<String> {
        let (x, y) = cell_xy(floor);
        let mut found = Vec::new();
        for dir in Dir::all() {
            let (dx, dy) = dir.delta();
            let Some(cell) = cell_id(x + dx, y + dy) else {
                continue;
            };
            if cell == origin || self.pile_count(cell, Material::BitterBerry) == 0 {
                continue;
            }
            let Some(name) = self.name_standing_on(origin, cell) else {
                continue;
            };
            let way = orthogonal_step(floor, cell)?;
            found.push(format!(
                " {} of that floor, {name} stands on the poison berries.",
                capitalize(way)
            ));
        }
        if found.len() != 1 {
            return None;
        }
        found.pop()
    }

    /// The valley floor just outside the rock, once the deer has passed the wood.
    fn outside_floor(&self, here: CellId) -> Option<OutsideFloor> {
        if !self.tracks.contains(&here) {
            return None;
        }
        if self.deer.as_ref().is_some_and(|deer| deer.cell == here) {
            return None;
        }
        let (x, y) = cell_xy(here);
        for dir in Dir::all() {
            let (dx, dy) = dir.delta();
            let Some(rock) = cell_id(x - dx, y - dy) else {
                continue;
            };
            if !self.open_wood_keeps_its_way(rock) {
                continue;
            }
            let Some((corner, _)) = self.corner_wood(rock, dir) else {
                continue;
            };
            let Some(wood_side) = orthogonal_step(here, corner) else {
                continue;
            };
            let deer_there = self.deer_stands_past_this_wood(rock, corner).is_some();
            let left = self
                .floor_west_of_this_wood(rock, corner)
                .is_some_and(|entered| {
                    self.deer_stands_on_next_floor(rock, entered).is_some()
                        || self.next_floor_still_in_sight(rock, entered).is_some()
                });
            if deer_there || left {
                return Some(OutsideFloor {
                    wood_side,
                    rock_side: facing_back(dir),
                    wood: corner,
                });
            }
        }
        None
    }

    /// The outside floor is the next square, on the side away from the rock.
    fn outside_floor_next_door(&self, here: CellId) -> Option<&'static str> {
        let (x, y) = cell_xy(here);
        for dir in Dir::all() {
            let (dx, dy) = dir.delta();
            let Some(dest) = cell_id(x + dx, y + dy) else {
                continue;
            };
            let Some(floor) = self.outside_floor(dest) else {
                continue;
            };
            if dir.word() == floor.rock_side {
                return Some(dir.word());
            }
        }
        None
    }

    /// The person on the rock, named, standing on the ground that is already there.
    /// The rock square keeps the valley floor. It does not get a new floor name.
    fn name_under_that_rock(
        &self,
        origin: CellId,
        floor_cell: CellId,
        rock_side: &str,
    ) -> Option<String> {
        let dir = Dir::all().into_iter().find(|dir| dir.word() == rock_side)?;
        let (x, y) = cell_xy(floor_cell);
        let (dx, dy) = dir.delta();
        let rock = cell_id(x + dx, y + dy)?;
        if !self.cell(rock).rock || chebyshev(origin, rock) > self.tuning.sight {
            return None;
        }
        let mut names: Vec<_> = self
            .actors
            .values()
            .filter(|actor| actor.cell == rock)
            .map(|actor| actor.name.clone())
            .collect();
        if names.len() != 1 {
            return None;
        }
        let name = names.pop()?;
        let ground = self.cell(rock).region.word();
        Some(format!("{name} stands on the {ground} under that rock."))
    }

    /// East from the outside floor names the rock and the person on the valley floor under it.
    /// The prints that follow mark that floor. The person is not a count.
    fn rock_from_the_outside_floor(&self, dir: Dir, rock: CellId, name: &str) -> String {
        let mut parts = vec![format!(
            "{}: the rock overhang. {name}",
            capitalize(dir.word())
        )];
        if self.pile_count(rock, Material::Branch) > 0 {
            parts.push("A pile of branches lies there.".into());
        }
        if self.pile_count(rock, Material::Pot) > 0
            || !self.ground_ids(rock, Material::Pot).is_empty()
        {
            parts.push("A pot sits there.".into());
        }
        let bark = self.pile_count(rock, Material::Bark)
            + self.ground_ids(rock, Material::Bark).len() as u32;
        if bark > 1 {
            parts.push("A pile of bark lies there.".into());
        } else if bark == 1 {
            parts.push("A sheet of bark lies there.".into());
        }
        if self.tracks.contains(&rock) {
            parts.push("Hoofprints mark that floor.".into());
        }
        parts.join(" ")
    }

    /// The west floor is the next square south, after the deer has left it.
    fn west_floor_left_to_the_south(&self, origin: CellId, dest: CellId) -> bool {
        if self.deer.as_ref().is_some_and(|deer| deer.cell == dest) || !self.tracks.contains(&dest)
        {
            return false;
        }
        let (x, y) = cell_xy(origin);
        for dir in Dir::all() {
            let (dx, dy) = dir.delta();
            let Some(next) = cell_id(x + dx, y + dy) else {
                continue;
            };
            let Some(floor) = self.outside_floor(next) else {
                continue;
            };
            if dir.word() != floor.rock_side {
                continue;
            }
            return self.first_cell_off_this_wood(floor.wood) == Some(dest);
        }
        false
    }

    /// The rotting wood is the next square south of the outside floor.
    fn wood_south_of_the_outside_floor(
        &self,
        origin: CellId,
        cell: CellId,
    ) -> Option<&'static str> {
        let (x, y) = cell_xy(origin);
        for dir in Dir::all() {
            let (dx, dy) = dir.delta();
            let Some(next) = cell_id(x + dx, y + dy) else {
                continue;
            };
            let Some(floor) = self.outside_floor(next) else {
                continue;
            };
            if dir.word() == floor.rock_side && floor.wood == cell {
                return Some(floor.wood_side);
            }
        }
        None
    }

    /// The open wood, the floor west of it, the south floor, and the trail west of that.
    fn floors_off_the_rotting_wood(&self) -> Option<FloorsOffWood> {
        for y in 0..crate::model::HEIGHT {
            for x in 0..crate::model::WIDTH {
                let Some(wood) = cell_id(x, y) else {
                    continue;
                };
                if !self.open_loose_branches(wood) {
                    continue;
                }
                let Some(west) = cell_id(x - 1, y) else {
                    continue;
                };
                let Some(south) = cell_id(x - 1, y + 1) else {
                    continue;
                };
                let Some(far) = cell_id(x - 2, y + 1) else {
                    continue;
                };
                if !self.cell(far).trail {
                    continue;
                }
                let Some(side) = orthogonal_step(west, wood) else {
                    continue;
                };
                return Some(FloorsOffWood {
                    wood,
                    west,
                    south,
                    far,
                    side,
                });
            }
        }
        None
    }

    /// Far's square sees this rotting wood as east of the west floor.
    fn wood_east_of_the_west_floor(&self, origin: CellId, cell: CellId) -> Option<&'static str> {
        if self.time < 10 {
            return None;
        }
        let floors = self.floors_off_the_rotting_wood()?;
        (origin == floors.far && cell == floors.wood).then_some(floors.side)
    }

    /// The outside floor is the open square between the rock and the rotting wood.
    fn outside_floor_cell(&self, floors: &FloorsOffWood) -> Option<CellId> {
        let (x, y) = cell_xy(floors.wood);
        let outside = cell_id(x, y - 1)?;
        let (ox, oy) = cell_xy(outside);
        let rock = cell_id(ox + 1, oy)?;
        self.cell(rock).rock.then_some(outside)
    }

    /// From Far's trail, the deer stands on the outside floor or the west floor.
    fn deer_on_named_ground_from_far(&self, origin: CellId) -> Option<String> {
        if self.time < 10 {
            return None;
        }
        let floors = self.floors_off_the_rotting_wood()?;
        if origin != floors.far {
            return None;
        }
        let deer = self.deer.as_ref()?;
        if chebyshev(origin, deer.cell) > self.tuning.sight {
            return None;
        }
        let cover = self.cover_phrase(deer.cell);
        if self.outside_floor_cell(&floors) == Some(deer.cell) {
            return Some(format!("A deer stands on the outside floor, {cover}."));
        }
        if deer.cell == floors.west {
            return Some(format!("A deer stands on the west floor, {cover}."));
        }
        None
    }

    /// The deer has walked off the west floor onto the south floor, or past it.
    fn deer_has_reached_the_south_floor(&self, floors: &FloorsOffWood) -> bool {
        let Some(deer) = self.deer.as_ref() else {
            return false;
        };
        let (sx, sy) = cell_xy(floors.south);
        let reached = DEER_ROUTE
            .iter()
            .enumerate()
            .rev()
            .find(|(_, step)| **step == (sx, sy))
            .map(|(index, _)| index);
        reached.is_some_and(|index| deer.route_at >= index)
    }

    /// East from the trail names the south floor once the deer has reached it,
    /// then the west floor, the rotting wood, and the outside floor. The deer
    /// stays on the south-floor sentence.
    fn south_floor_to_the_east(&self, origin: CellId, dest: CellId, dir: Dir) -> Option<String> {
        let floors = self.floors_off_the_rotting_wood()?;
        if origin != floors.far
            || dest != floors.south
            || !self.deer_has_reached_the_south_floor(&floors)
        {
            return None;
        }
        let way = capitalize(dir.word());
        let mut line = if self.deer.as_ref().is_some_and(|deer| deer.cell == dest) {
            format!(
                "{way}: the south floor. The deer stands on that floor. Hoofprints mark that floor."
            )
        } else if self.tracks.contains(&dest) {
            format!("{way}: the south floor. Hoofprints mark that floor.")
        } else {
            return None;
        };
        if let Some(past) = self.west_floor_north_of_the_south_floor(&floors) {
            line.push_str(&past);
        }
        Some(line)
    }

    /// The west floor is the square directly north of the south floor.
    /// The rotting wood is east of that floor, and the outside floor is north of the wood.
    fn west_floor_north_of_the_south_floor(&self, floors: &FloorsOffWood) -> Option<String> {
        if !self.tracks.contains(&floors.west) {
            return None;
        }
        let way = orthogonal_step(floors.south, floors.west)?;
        let named = capitalize(way);
        let mut line =
            format!(" {named} of that floor is the west floor. Hoofprints mark that floor.");
        if let Some(past) = self.wood_and_outside_floor_from_the_west_floor(floors) {
            line.push_str(&past);
        }
        Some(line)
    }

    /// From the west floor, the rotting wood and the outside floor north of it.
    /// The last "that floor" is the outside floor. The deer is not named here.
    fn wood_and_outside_floor_from_the_west_floor(&self, floors: &FloorsOffWood) -> Option<String> {
        if chebyshev(floors.far, floors.wood) > self.tuning.sight {
            return None;
        }
        let outside = self.outside_floor_cell(floors)?;
        if chebyshev(floors.far, outside) > self.tuning.sight || !self.tracks.contains(&outside) {
            return None;
        }
        let to_wood = orthogonal_step(floors.west, floors.wood)?;
        let to_outside = orthogonal_step(floors.wood, outside)?;
        Some(format!(
            " {} of that floor is the rotting wood. {} of that wood is the outside floor. Hoofprints mark that floor.",
            capitalize(to_wood),
            capitalize(to_outside)
        ))
    }

    /// The floor the deer stepped onto after the west floor, still in sight.
    fn floor_past_the_west_floor(&self, origin: CellId, west: CellId) -> Option<String> {
        let steps = self.steps_off_the_wood(origin)?;
        if steps.first().copied() != Some(west) {
            return None;
        }
        let next = *steps.get(1)?;
        if chebyshev(origin, next) > self.tuning.sight {
            return None;
        }
        let way = orthogonal_step(west, next)?;
        let named = capitalize(way);
        if self.deer.as_ref().is_some_and(|deer| deer.cell == next) {
            return Some(format!(
                " {named} of that floor is the {way} floor, still in sight. The deer stands on the {way} floor."
            ));
        }
        if self.tracks.contains(&next) {
            return Some(format!(
                " {named} of that floor is the {way} floor, still in sight. Hoofprints mark the {way} floor."
            ));
        }
        None
    }

    fn west_floor_from_here(&self, origin: CellId) -> Option<CellId> {
        let (x, y) = cell_xy(origin);
        for dir in Dir::all() {
            let (dx, dy) = dir.delta();
            let Some(next) = cell_id(x + dx, y + dy) else {
                continue;
            };
            let Some(floor) = self.outside_floor(next) else {
                continue;
            };
            if dir.word() != floor.rock_side {
                continue;
            }
            return self.first_cell_off_this_wood(floor.wood);
        }
        None
    }

    /// South already says this deer stands on the south floor.
    fn deer_stands_on_the_south_floor_from_here(&self, origin: CellId) -> bool {
        let Some(west) = self.west_floor_from_here(origin) else {
            return false;
        };
        let Some(steps) = self.steps_off_the_wood(origin) else {
            return false;
        };
        if steps.first().copied() != Some(west) {
            return false;
        }
        let Some(next) = steps.get(1).copied() else {
            return false;
        };
        self.deer.as_ref().is_some_and(|deer| deer.cell == next)
            && chebyshev(origin, next) <= self.tuning.sight
    }

    /// The deer has left the south floor onto the trail still in sight.
    /// A person standing on that trail is named from the person, not hardcoded.
    fn deer_on_the_trail_past_the_south_floor(&self, viewer: ActorId) -> Option<String> {
        let origin = self.actor(viewer)?.cell;
        let west = self.west_floor_from_here(origin)?;
        let steps = self.steps_off_the_wood(origin)?;
        if steps.first().copied() != Some(west) {
            return None;
        }
        let south_floor = *steps.get(1)?;
        let trail = *steps.get(2)?;
        let deer = self.deer.as_ref()?;
        if deer.cell != trail || chebyshev(origin, trail) > self.tuning.sight {
            return None;
        }
        if deer.cell == south_floor || deer.cell == west {
            return None;
        }
        if !self.cell(trail).trail {
            return None;
        }
        let way = orthogonal_step(south_floor, trail)?;
        let mut line =
            format!("A deer stands in the open, on the deer trail {way} of the south floor.");
        if let Some(name) = self.other_name_on(viewer, trail) {
            line.push_str(&format!(" {name} stands on that trail."));
        }
        Some(line)
    }

    /// At the south floor, before the deer arrives, the person on the trail is named.
    fn person_on_the_trail_west_of_the_south_floor(&self, viewer: ActorId) -> Option<String> {
        let origin = self.actor(viewer)?.cell;
        let floors = self.floors_off_the_rotting_wood()?;
        if !self.deer_has_reached_the_south_floor(&floors) {
            return None;
        }
        let west = self.west_floor_from_here(origin)?;
        if west != floors.west {
            return None;
        }
        if self
            .deer
            .as_ref()
            .is_some_and(|deer| deer.cell == floors.far)
        {
            return None;
        }
        let name = self.other_name_on(viewer, floors.far)?;
        let way = orthogonal_step(floors.south, floors.far)?;
        Some(format!(
            "{name} stands on the deer trail {way} of the south floor."
        ))
    }

    /// From the trail, the person on the poison berries north of the west floor.
    /// The sentence stays out of East.
    fn person_on_the_berries_north_of_the_west_floor(&self, viewer: ActorId) -> Option<String> {
        let origin = self.actor(viewer)?.cell;
        let floors = self.floors_off_the_rotting_wood()?;
        if origin != floors.far || !self.deer_has_reached_the_south_floor(&floors) {
            return None;
        }
        let (x, y) = cell_xy(floors.west);
        let berries = cell_id(x, y - 1)?;
        if orthogonal_step(floors.west, berries) != Some("north")
            || self.pile_count(berries, Material::BitterBerry) == 0
            || chebyshev(origin, berries) > self.tuning.sight
        {
            return None;
        }
        let name = self.other_name_on(viewer, berries)?;
        let cover = self.cover_phrase(berries);
        Some(format!(
            "{name} stands {cover}, on the poison berries north of the west floor."
        ))
    }

    /// The other person standing on a cell in sight. One name, taken from that person.
    fn other_name_on(&self, viewer: ActorId, cell: CellId) -> Option<String> {
        let origin = self.actor(viewer)?.cell;
        if chebyshev(origin, cell) > self.tuning.sight {
            return None;
        }
        let mut names: Vec<_> = self
            .actors
            .values()
            .filter(|actor| actor.id != viewer && actor.cell == cell)
            .map(|actor| actor.name.clone())
            .collect();
        if names.len() != 1 {
            return None;
        }
        names.pop()
    }

    fn corner_beyond(&self, origin: CellId, dir: Dir) -> Option<String> {
        if let Some((corner, side)) = self.corner_deer(origin, dir) {
            let named_earlier = Dir::all()
                .into_iter()
                .take_while(|earlier| *earlier != dir)
                .any(|earlier| {
                    self.corner_deer(origin, earlier)
                        .is_some_and(|(cell, _)| cell == corner)
                });
            if named_earlier {
                return Some(format!("That way, to the {side}, reaches the same wood."));
            }
            if self.look_already_turns_toward(origin, corner) && self.tracks.contains(&corner) {
                return Some(format!(
                    "The corner beyond, to the {side}, has the rotting wood and the deer, {}, and hoofprints mark the wood under the deer.",
                    self.cover_phrase(corner)
                ));
            }
            return Some(format!(
                "The corner beyond, to the {side}, has the rotting wood and the deer."
            ));
        }
        if !self.open_wood_keeps_its_way(origin) {
            return None;
        }
        let (corner, side) = self.corner_wood(origin, dir)?;
        let named_earlier = Dir::all()
            .into_iter()
            .take_while(|earlier| *earlier != dir)
            .any(|earlier| {
                self.corner_wood(origin, earlier)
                    .is_some_and(|(cell, _)| cell == corner)
            });
        if named_earlier {
            Some(format!("That way, to the {side}, reaches the same wood."))
        } else {
            let mut line = format!("The corner beyond, to the {side}, has that wood.");
            if let Some((_, way)) = self.deer_stands_past_this_wood(origin, corner) {
                line.push_str(&format!(
                    " Past that wood is the {way} floor. Hoofprints mark that floor. The deer stands on the {way} floor."
                ));
                return Some(line);
            }
            if let Some(entered) = self.floor_west_of_this_wood(origin, corner) {
                if let Some(wood_side) = orthogonal_step(corner, entered) {
                    let standing = self.deer_stands_on_next_floor(origin, entered);
                    let left_sight = self.deer_entered_west_of_this_wood(origin, corner);
                    if standing.is_some() || left_sight {
                        line.push_str(&format!(
                            " Past that wood is the {wood_side} floor. The deer has left it. Hoofprints mark that ground."
                        ));
                        if let Some(step) = standing {
                            line.push_str(&format!(
                                " {} of the {wood_side} floor is the {step} floor, still in sight. The deer stands on the {step} floor.",
                                capitalize(step)
                            ));
                        } else if let Some(step) = self.next_floor_still_in_sight(origin, entered) {
                            line.push_str(&format!(
                                " {} of the {wood_side} floor is the {step} floor, still in sight. Hoofprints mark the {step} floor.",
                                capitalize(step)
                            ));
                        }
                    }
                }
            }
            Some(line)
        }
    }

    /// The next floor the deer stepped onto after leaving the wood, while that
    /// floor is still in sight and the deer has left it.
    fn next_floor_still_in_sight(&self, origin: CellId, from: CellId) -> Option<&'static str> {
        let steps = self.steps_off_the_wood(origin)?;
        if steps.first().copied() != Some(from) {
            return None;
        }
        let next = *steps.get(1)?;
        let step = orthogonal_step(from, next)?;
        if chebyshev(origin, next) > self.tuning.sight || !self.tracks.contains(&next) {
            return None;
        }
        if self.deer.as_ref().is_some_and(|deer| deer.cell == next) {
            return None;
        }
        Some(step)
    }

    /// After West has followed South. The deer South named ate the berries
    /// that were here. The passage after this sentence does not repeat it.
    fn deer_on_the_named_floor_ate(&self, origin: CellId) -> Option<String> {
        if self.pile_count(origin, Material::Berry) > 0
            || !self.spent_berries.contains(&origin)
            || !self.deer_ate_berries.contains(&origin)
        {
            return None;
        }
        Dir::all().into_iter().find_map(|dir| {
            let (wood, _) = self.corner_wood(origin, dir)?;
            let entered = self.floor_west_of_this_wood(origin, wood)?;
            self.deer_stands_on_next_floor(origin, entered)
        })?;
        Some("The deer ate the berries that were here.".into())
    }

    /// South already says this deer stands on the south floor, so the look
    /// does not count the cells for another standing sentence.
    fn south_line_stands_the_deer(&self, origin: CellId) -> bool {
        Dir::all().into_iter().any(|dir| {
            self.corner_wood(origin, dir).is_some_and(|(wood, _)| {
                self.floor_west_of_this_wood(origin, wood)
                    .is_some_and(|entered| {
                        self.deer_stands_on_next_floor(origin, entered).is_some()
                    })
            })
        })
    }

    /// The deer is standing on the floor it stepped onto after the floor west
    /// of this wood, and that floor is still in sight.
    fn deer_stands_on_next_floor(&self, origin: CellId, from: CellId) -> Option<&'static str> {
        let deer = self.deer.as_ref()?;
        let steps = self.steps_off_the_wood(origin)?;
        if steps.first().copied() != Some(from) {
            return None;
        }
        let next = *steps.get(1)?;
        if deer.cell != next {
            return None;
        }
        let step = orthogonal_step(from, next)?;
        if chebyshev(origin, next) > self.tuning.sight {
            return None;
        }
        Some(step)
    }

    /// The cell immediately west of this wood, once the deer has stepped onto
    /// it and left hoofprints, and it is not the open cell west of the floor
    /// West names.
    fn floor_west_of_this_wood(&self, origin: CellId, wood: CellId) -> Option<CellId> {
        let entered = self.first_cell_off_this_wood(wood)?;
        let (wx, wy) = cell_xy(wood);
        let west_of_wood = cell_id(wx - 1, wy)?;
        if entered != west_of_wood || !self.tracks.contains(&entered) {
            return None;
        }
        let (ox, oy) = cell_xy(origin);
        if cell_id(ox - 2, oy).is_some_and(|open_west_of_floor| entered == open_west_of_floor) {
            return None;
        }
        Some(entered)
    }

    /// The deer has left sight after stepping from this wood onto the cell
    /// immediately west of it, which is not the open cell west of the floor
    /// West names.
    fn deer_entered_west_of_this_wood(&self, origin: CellId, wood: CellId) -> bool {
        let Some(deer) = &self.deer else {
            return false;
        };
        if chebyshev(origin, deer.cell) <= self.tuning.sight {
            return false;
        }
        self.floor_west_of_this_wood(origin, wood).is_some()
    }

    fn first_cell_off_this_wood(&self, wood: CellId) -> Option<CellId> {
        let deer = self.deer.as_ref()?;
        let n = DEER_ROUTE.len();
        let (dx, dy) = cell_xy(deer.cell);
        let mut idx = if DEER_ROUTE.get(deer.route_at) == Some(&(dx, dy)) {
            deer.route_at
        } else {
            DEER_ROUTE
                .iter()
                .position(|&cell| cell == (dx, dy))
                .unwrap_or(deer.route_at)
        };
        let mut back = Vec::new();
        let mut found = false;
        for _ in 0..n {
            let (x, y) = DEER_ROUTE[idx];
            let cell = cell_id(x, y)?;
            if cell == wood {
                found = true;
                break;
            }
            back.push(cell);
            idx = (idx + n - 1) % n;
        }
        if !found || back.is_empty() {
            return None;
        }
        back.reverse();
        Some(back[0])
    }

    /// South has already named this deer on the corner wood.
    fn deer_named_on_corner(&self, origin: CellId) -> bool {
        let Some(deer) = &self.deer else {
            return false;
        };
        deer.cell != origin
            && chebyshev(origin, deer.cell) <= self.tuning.sight
            && self.pile_count(origin, Material::Berry) == 0
            && self.spent_berries.contains(&origin)
            && self.deer_ate_berries.contains(&origin)
            && self.direction_names_deer(origin, deer.cell)
    }

    fn deer_named_next_door(&self, origin: CellId) -> bool {
        let Some(deer) = &self.deer else {
            return false;
        };
        deer.cell != origin
            && self.pile_count(origin, Material::Berry) == 0
            && self.spent_berries.contains(&origin)
            && self.deer_ate_berries.contains(&origin)
            && self.neighbor_line_names_deer(origin, deer.cell)
    }

    /// The neighbor line already says "a deer" for this cell.
    fn neighbor_line_names_deer(&self, origin: CellId, deer_cell: CellId) -> bool {
        let (x1, y1) = cell_xy(origin);
        let (x2, y2) = cell_xy(deer_cell);
        let dx = x2 - x1;
        let dy = y2 - y1;
        let next_door = (dx == 0 && dy.abs() == 1) || (dy == 0 && dx.abs() == 1);
        if !next_door {
            return false;
        }
        let just_arrived = self.pile_count(deer_cell, Material::Branch) > 0
            && self.pile_count(deer_cell, Material::Berry) > 0
            && self.deer_meal_this_tick(deer_cell).is_none();
        !self.open_loose_branches(deer_cell)
            && !just_arrived
            && self.deer_meal_this_tick(deer_cell).is_none()
    }

    fn direction_names_deer(&self, origin: CellId, deer_cell: CellId) -> bool {
        Dir::all().into_iter().any(|dir| {
            self.corner_deer(origin, dir)
                .is_some_and(|(cell, _)| cell == deer_cell)
        })
    }

    fn corner_deer(&self, origin: CellId, dir: Dir) -> Option<(CellId, &'static str)> {
        self.corner_on(origin, dir, true)
    }

    fn corner_wood(&self, origin: CellId, dir: Dir) -> Option<(CellId, &'static str)> {
        self.corner_on(origin, dir, false)
    }

    fn corner_on(
        &self,
        origin: CellId,
        dir: Dir,
        deer_required: bool,
    ) -> Option<(CellId, &'static str)> {
        let (x, y) = cell_xy(origin);
        let (dx, dy) = dir.delta();
        let sides: [(i32, i32, &str); 2] = if dx != 0 {
            [(0, -1, "north"), (0, 1, "south")]
        } else {
            [(-1, 0, "west"), (1, 0, "east")]
        };
        for (px, py, side) in sides {
            let Some(corner) = cell_id(x + dx + px, y + dy + py) else {
                continue;
            };
            if !self.open_loose_branches(corner) || chebyshev(origin, corner) > self.tuning.sight {
                continue;
            }
            let deer_there = self.deer.as_ref().is_some_and(|deer| deer.cell == corner);
            if deer_required != deer_there {
                continue;
            }
            return Some((corner, side));
        }
        None
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
                    if origin == cell {
                        lines.push("A lump of clay is hardening in the fire.".into());
                    } else {
                        lines.push(format!(
                            "A lump of clay is hardening in the fire {}.",
                            self.place_phrase(origin, cell)
                        ));
                    }
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

    fn examine_carcass(&self, viewer: ActorId) -> String {
        let origin = self.actor(viewer).unwrap().cell;
        let mut lines = Vec::new();
        for cell in self.visible_cell_ids(viewer) {
            for id in self.ground_ids(cell, Material::Carcass) {
                let moisture = self.item(id).map(|item| item.moisture).unwrap_or(0);
                lines.push(format!(
                    "A deer carcass lies in contact with the ground {}. It is {}. {} It must be cooked before it feeds you.",
                    self.place_phrase(origin, cell),
                    self.condition_word(moisture),
                    self.rain_reaches_phrase(cell)
                ));
            }
        }
        for other in self.visible_actor_ids(viewer) {
            if self.loose_ids(other, Material::Carcass).is_empty() {
                continue;
            }
            let who = if other == viewer {
                "you".to_string()
            } else {
                self.name_of(other)
            };
            let cell = self.actor(other).unwrap().cell;
            lines.push(format!(
                "A deer carcass is carried by {who}, {}. It must be cooked before it feeds you.",
                self.cover_phrase(cell)
            ));
        }
        if lines.is_empty() {
            "You see no carcass from here.".into()
        } else {
            lines.join("\n")
        }
    }

    fn examine_spear(&self, viewer: ActorId) -> String {
        let origin = self.actor(viewer).unwrap().cell;
        let mut lines = Vec::new();
        for cell in self.visible_cell_ids(viewer) {
            for id in self.ground_ids(cell, Material::Spear) {
                let hardened = self.item(id).is_some_and(|item| item.hardened);
                let point = if hardened {
                    "The point is hardened."
                } else {
                    "The point is not hardened."
                };
                lines.push(format!(
                    "A spear lies in contact with the ground {}. {point}",
                    self.place_phrase(origin, cell)
                ));
            }
        }
        for other in self.visible_actor_ids(viewer) {
            let who = if other == viewer {
                "you".to_string()
            } else {
                self.name_of(other)
            };
            let cell = self.actor(other).unwrap().cell;
            for id in self.loose_ids(other, Material::Spear) {
                let hardened = self.item(id).is_some_and(|item| item.hardened);
                let point = if hardened {
                    "a hardened point"
                } else {
                    "an unhardened point"
                };
                let waiting = !hardened && self.spear_is_hardening(other);
                if waiting {
                    lines.push(format!(
                        "A spear with {point} is carried by {who}, {}. The point is hardening in the fire.",
                        self.cover_phrase(cell)
                    ));
                } else {
                    lines.push(format!(
                        "A spear with {point} is carried by {who}, {}.",
                        self.cover_phrase(cell)
                    ));
                }
            }
        }
        if lines.is_empty() {
            "You see no spear from here.".into()
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
                } else if material == Material::Bark && count == 1 {
                    lines.push(format!(
                        "A sheet of bark lies in contact with the ground {}. The bark is {}. {}",
                        self.place_phrase(origin, cell),
                        self.condition_word(self.pile_moisture(cell, material)),
                        self.rain_reaches_phrase(cell)
                    ));
                } else if material == Material::Pot && count == 1 {
                    lines.push(format!(
                        "A pot lies in contact with the ground {}. The pot is {}. {} It can boil water over a fire.",
                        self.place_phrase(origin, cell),
                        self.condition_word(self.pile_moisture(cell, material)),
                        self.rain_reaches_phrase(cell)
                    ));
                } else if material == Material::Berry {
                    let eating = if self.deer.as_ref().is_some_and(|deer| deer.cell == cell) {
                        " A deer is eating them."
                    } else {
                        ""
                    };
                    let (kind, verb) = food_kind(material);
                    let head = if count == 1 {
                        format!(
                            "A handful of berries is in contact with the ground {}",
                            self.place_phrase(origin, cell)
                        )
                    } else {
                        format!(
                            "A pile of {count} {} is in contact with the ground {}",
                            pile_noun(material, count),
                            self.place_phrase(origin, cell)
                        )
                    };
                    lines.push(format!(
                        "{head}.{eating} The {kind} {verb} {}. {}{}{}{}",
                        self.condition_word(self.pile_moisture(cell, material)),
                        self.rain_reaches_phrase(cell),
                        self.heavier_rain_clause(cell, material),
                        self.rotting_clause(cell, material),
                        food_clause(material)
                    ));
                } else if material == Material::Branch {
                    let noun = pile_noun(material, count);
                    let (kind, verb) = food_kind(material);
                    lines.push(format!(
                        "A pile of {count} {noun} is in contact with the ground {}{}. The {kind} {verb} {}. {}{}{}{}{}",
                        self.mark_place(origin, cell),
                        self.hoofprints_on_wood(origin, cell),
                        self.condition_word(self.pile_moisture(cell, material)),
                        self.rain_reaches_phrase(cell),
                        self.heavier_rain_clause(cell, material),
                        self.rotting_clause(cell, material),
                        food_clause(material),
                        self.deer_on_this_wood(origin, cell)
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
                } else if material == Material::Bark && loose.len() == 1 {
                    lines.push(format!(
                        "A sheet of bark lies in contact with the ground {}. The bark is {}. {}",
                        self.place_phrase(origin, cell),
                        self.condition_word(worst),
                        self.rain_reaches_phrase(cell)
                    ));
                } else if material == Material::Pot && loose.len() == 1 {
                    lines.push(format!(
                        "A pot lies in contact with the ground {}. The pot is {}. {} It can boil water over a fire.",
                        self.place_phrase(origin, cell),
                        self.condition_word(worst),
                        self.rain_reaches_phrase(cell)
                    ));
                } else if material == Material::Branch {
                    lines.push(format!(
                        "{} loose {} {} in contact with the ground {}{}. {}{}{}{}{}",
                        loose.len(),
                        pile_noun(material, loose.len() as u32),
                        if loose.len() == 1 { "is" } else { "are" },
                        self.mark_place(origin, cell),
                        self.hoofprints_on_wood(origin, cell),
                        self.condition_sentence(worst, cell),
                        self.heavier_rain_clause(cell, material),
                        self.rotting_clause(cell, material),
                        food_clause(material),
                        self.deer_on_this_wood(origin, cell)
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
            } else if matches!(
                material,
                Material::Mushroom | Material::Berry | Material::BitterBerry
            ) {
                let phrase =
                    carried_food_phrase(material, ids.len() as u32, self.condition_word(moisture));
                lines.push(format!(
                    "{phrase} {} carried by {who}, {}.{}",
                    if ids.len() == 1 { "is" } else { "are" },
                    self.cover_phrase(cell),
                    food_clause(material)
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
            if material == Material::Berry && self.spent_berries.contains(&origin) {
                if self.deer_ate_berries.contains(&origin) {
                    let still_here = self.deer.as_ref().is_some_and(|deer| deer.cell == origin);
                    if still_here {
                        return "The berries that were here are gone. The deer here ate them."
                            .into();
                    }
                    return "The berries that were here are gone. A deer ate them.".into();
                }
                return "The berries that were here are gone.".into();
            }
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
                Material::Spear => "spear",
                Material::Carcass => "carcass",
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

    fn examine_rock(&self, viewer: ActorId) -> String {
        let origin = self.actor(viewer).unwrap().cell;
        let mut lines = Vec::new();
        for cell in self.visible_cell_ids(viewer) {
            if !self.cell(cell).rock {
                continue;
            }
            let exposure = self.exposure_millis(cell);
            let wind = self.wind_word(exposure);
            if cell == origin {
                let rain = if exposure == 0 {
                    "Rain does not reach you."
                } else if exposure < 1000 {
                    "Only a little rain reaches you."
                } else if self.raining {
                    "Rain reaches you."
                } else {
                    "The sky is open above you."
                };
                lines.push(format!("Rock leans overhead. You are {wind}. {rain}"));
            } else {
                lines.push(format!(
                    "Rock leans overhead to the {}. It is {wind}. {}",
                    self.relative(origin, cell),
                    self.rain_reaches_phrase(cell)
                ));
            }
        }
        if lines.is_empty() {
            "You cannot see a rock from here.".into()
        } else {
            lines.join("\n")
        }
    }

    fn examine_slope(&self, viewer: ActorId) -> String {
        let origin = self.actor(viewer).unwrap().cell;
        let visible = self.visible_cell_ids(viewer);
        if !visible
            .iter()
            .any(|cell| self.cell(*cell).region == crate::model::Region::Slope)
        {
            return "You cannot see a slope from here.".into();
        }
        let (ox, oy) = cell_xy(origin);
        let mut ways = Vec::new();
        for dir in Dir::all() {
            let (dx, dy) = dir.delta();
            let Some(dest) = cell_id(ox + dx, oy + dy) else {
                continue;
            };
            if visible.contains(&dest) && self.cell(dest).region == crate::model::Region::Slope {
                ways.push(dir.word());
            }
        }
        if ways.is_empty() {
            let mut north = false;
            let mut east = false;
            let mut south = false;
            let mut west = false;
            for cell in &visible {
                if self.cell(*cell).region != crate::model::Region::Slope {
                    continue;
                }
                let (x, y) = cell_xy(*cell);
                north |= y < oy;
                east |= x > ox;
                south |= y > oy;
                west |= x < ox;
            }
            for (on, word) in [
                (north, "north"),
                (east, "east"),
                (south, "south"),
                (west, "west"),
            ] {
                if on {
                    ways.push(word);
                }
            }
        }
        let way = match ways.len() {
            0 => "here".to_string(),
            1 => ways[0].to_string(),
            2 => format!("{} and {}", ways[0], ways[1]),
            _ => format!(
                "{}, and {}",
                ways[..ways.len() - 1].join(", "),
                ways[ways.len() - 1]
            ),
        };
        format!("The slope is open ground. It runs {way} from where you stand.")
    }

    fn examine_deer(&self, viewer: ActorId) -> String {
        let origin = self.actor(viewer).unwrap().cell;
        let Some(deer) = self.deer.as_ref() else {
            return "You cannot see a deer from here.".into();
        };
        if chebyshev(origin, deer.cell) > self.tuning.sight {
            return "You cannot see a deer from here.".into();
        }
        if let Some(floor) = self.deer_past_the_wood(origin) {
            return format!("A deer stands on {floor}. Hoofprints mark that floor.");
        }
        let eating = if self.pile_count(deer.cell, Material::Berry) > 0 {
            " It is eating the berries."
        } else if self.deer_ate_berries.contains(&deer.cell) {
            if origin == deer.cell {
                " It ate the berries here."
            } else {
                " It ate the berries there."
            }
        } else {
            ""
        };
        format!(
            "A deer stands {}.{}",
            self.deer_place(origin, deer.cell),
            eating
        )
    }

    fn eating_handful_line(&self, cell: CellId, here: bool) -> String {
        let place = if here { "here" } else { "there" };
        if self.pile_count(cell, Material::Berry) == 1 {
            format!("A deer is eating the berries {place}. This handful is the last.")
        } else if self.earlier_deer_handfuls(cell) > 0 {
            format!("A deer is eating the berries {place}. Another handful is gone. More are left.")
        } else {
            format!("A deer is eating the berries {place}. A handful is gone. More are left.")
        }
    }

    fn earlier_deer_handfuls(&self, cell: CellId) -> usize {
        let tick = self.time.saturating_sub(1);
        self.events
            .iter()
            .filter(|event| {
                event.time < tick
                    && event.affected_cells.contains(&cell)
                    && (event.text == "A deer eats a handful of berries."
                        || event.text == "A deer eats the last handful of berries.")
            })
            .count()
    }

    /// `Some(true)` when the handful that went on this tick was the last.
    fn deer_meal_this_tick(&self, cell: CellId) -> Option<bool> {
        if self.time == 0 {
            return None;
        }
        let tick = self.time - 1;
        let mut ate = false;
        let mut last = false;
        for event in &self.events {
            if event.time != tick || !event.affected_cells.contains(&cell) {
                continue;
            }
            if event.text == "A deer eats the last handful of berries." {
                ate = true;
                last = true;
            } else if event.text == "A deer eats a handful of berries." {
                ate = true;
            }
        }
        ate.then_some(last)
    }

    fn deer_on_this_wood(&self, origin: CellId, cell: CellId) -> &'static str {
        if !self.deer.as_ref().is_some_and(|deer| deer.cell == cell) {
            return "";
        }
        match self.deer_meal_this_tick(cell) {
            Some(true) if origin == cell => {
                " A deer is eating the berries here. The last handful is gone."
            }
            Some(true) => " A deer is eating the berries there. The last handful is gone.",
            Some(false) if origin == cell => {
                " A deer is eating the berries here. A handful is gone."
            }
            Some(false) => " A deer is eating the berries there. A handful is gone.",
            None if self.exposure_millis(cell) >= 1000 => " A deer stands on the rotting branches.",
            None => " A deer stands on these branches.",
        }
    }

    /// Someone who cannot see the rock still has the deer on the open wood.
    fn off_rock_wood_hoofprints(
        &self,
        viewer: ActorId,
        origin: CellId,
        cell: CellId,
    ) -> &'static str {
        if self.tracks.contains(&cell) && !self.sees_rock(viewer) {
            ", and hoofprints mark the rotting wood under the deer"
        } else {
            self.hoofprints_on_wood(origin, cell)
        }
    }

    fn sees_rock(&self, viewer: ActorId) -> bool {
        self.visible_cell_ids(viewer)
            .into_iter()
            .any(|cell| self.cell(cell).rock)
    }

    fn hoofprints_on_wood(&self, origin: CellId, cell: CellId) -> &'static str {
        if !self.tracks.contains(&cell) {
            ""
        } else if origin == cell {
            ", and hoofprints mark the ground"
        } else {
            ", and hoofprints mark the ground there"
        }
    }

    /// The place line already said which way to turn toward this deer.
    fn look_already_turns_toward(&self, origin: CellId, deer_cell: CellId) -> bool {
        origin != deer_cell
            && self.pile_count(origin, Material::Berry) == 0
            && self.spent_berries.contains(&origin)
            && self.deer_ate_berries.contains(&origin)
            && self.open_loose_branches(deer_cell)
            && chebyshev(origin, deer_cell) <= self.tuning.sight
    }

    fn deer_place(&self, origin: CellId, deer_cell: CellId) -> String {
        if self.open_loose_branches(deer_cell) {
            if let Some(side) = self.wood_east_of_the_west_floor(origin, deer_cell) {
                return format!(
                    "on the rotting wood {side} of the west floor, {}",
                    self.cover_phrase(deer_cell)
                );
            }
            format!("on the rotting wood {}", self.mark_place(origin, deer_cell))
        } else {
            self.mark_place(origin, deer_cell)
        }
    }

    fn examine_tracks(&self, viewer: ActorId) -> String {
        let origin = self.actor(viewer).unwrap().cell;
        let mut lines = Vec::new();
        for cell in self.visible_cell_ids(viewer) {
            if self.tracks.contains(&cell) {
                lines.push(format!(
                    "Hoofprints mark the ground {}.",
                    self.mark_place(origin, cell)
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

    fn cell_offset(&self, from: CellId, to: CellId) -> String {
        if from == to {
            return "here".into();
        }
        let (x1, y1) = cell_xy(from);
        let (x2, y2) = cell_xy(to);
        let dx = x2 - x1;
        let dy = y2 - y1;
        let mut parts = Vec::new();
        if dy != 0 {
            let n = dy.unsigned_abs();
            let way = if dy < 0 { "north" } else { "south" };
            parts.push(format!("{} {way}", cell_span(n)));
        }
        if dx != 0 {
            let n = dx.unsigned_abs();
            let way = if dx > 0 { "east" } else { "west" };
            parts.push(format!("{} {way}", cell_span(n)));
        }
        parts.join(" and ")
    }

    fn mark_place(&self, from: CellId, to: CellId) -> String {
        if from == to {
            return self.place_phrase(from, to);
        }
        format!("{}, {}", self.cell_offset(from, to), self.cover_phrase(to))
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
                let id = cell_id(x, y).unwrap();
                let [left, right] = self.glyph(viewer, id);
                line.push(left);
                line.push(right);
                if left != '?' && self.content_marks(id).len() > 1 {
                    line.push('+');
                } else {
                    line.push(' ');
                }
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
        let content = self.content_marks(cell).first().copied().unwrap_or(' ');
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

    /// One letter for the cell, in the same order as before. Later marks are
    /// present but not drawn. The map adds `+` when this list is longer than one.
    fn content_marks(&self, cell: CellId) -> Vec<char> {
        let mut marks = Vec::new();
        if self.fire_at(cell).is_some() {
            marks.push('*');
        }
        if self.deer.as_ref().is_some_and(|deer| deer.cell == cell) {
            marks.push('d');
        }
        let log_here = self.pile_count(cell, Material::Log) > 0
            || self.items.values().any(|item| {
                item.material == Material::Log && item.cell == cell && item.carrier.is_none()
            });
        if log_here {
            marks.push('L');
        }
        let branch_moisture = self.worst_loose(cell, Material::Branch);
        if self.pile_count(cell, Material::Branch) > 0 || branch_moisture.is_some() {
            let moisture = branch_moisture
                .unwrap_or(0)
                .max(self.pile_moisture(cell, Material::Branch));
            marks.push(if moisture >= self.tuning.damp_at {
                'W'
            } else {
                'w'
            });
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
            marks.push(if moisture >= self.tuning.damp_at {
                'B'
            } else {
                'b'
            });
        }
        let berry_moisture = self.worst_loose(cell, Material::Berry);
        if self.pile_count(cell, Material::Berry) > 0 || berry_moisture.is_some() {
            let moisture = berry_moisture
                .unwrap_or(0)
                .max(self.pile_moisture(cell, Material::Berry));
            marks.push(if moisture >= self.tuning.damp_at {
                'Y'
            } else {
                'y'
            });
        }
        if self.pile_count(cell, Material::Mushroom) > 0
            || self.worst_loose(cell, Material::Mushroom).is_some()
        {
            marks.push('m');
        }
        if self.pile_count(cell, Material::BitterBerry) > 0
            || self.worst_loose(cell, Material::BitterBerry).is_some()
        {
            marks.push('q');
        }
        if self.pile_count(cell, Material::Pot) > 0
            || self.worst_loose(cell, Material::Pot).is_some()
        {
            marks.push('k');
        }
        if self.pile_count(cell, Material::Water) > 0
            || self.items.values().any(|item| {
                item.material == Material::Water
                    && item.role == Role::Loose
                    && item.carrier.is_none()
                    && item.cell == cell
            })
        {
            marks.push('u');
        }
        if self.pile_count(cell, Material::Stone) > 0
            || self.worst_loose(cell, Material::Stone).is_some()
        {
            marks.push('s');
        }
        if self.pile_count(cell, Material::Clay) > 0
            || self.worst_loose(cell, Material::Clay).is_some()
        {
            marks.push('c');
        }
        if self.worst_loose(cell, Material::Bowl).is_some() {
            marks.push('v');
        }
        if self.ground_ids(cell, Material::Carcass).len() > 0
            || self.worst_loose(cell, Material::Carcass).is_some()
        {
            marks.push('a');
        }
        if self.pile_count(cell, Material::Spear) > 0
            || self.worst_loose(cell, Material::Spear).is_some()
        {
            marks.push('j');
        }
        if self.tracks.contains(&cell) {
            marks.push('"');
        }
        marks
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
            carcass: self.ground_ids(cell, Material::Carcass).len() as u32,
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
                text: self.heard_text(origin, event),
            })
            .collect();
        if views.len() > 40 {
            views = views.split_off(views.len() - 40);
        }
        views
    }

    /// An event keeps the weather of its own tick. The wood is named, not counted.
    fn heard_text(&self, origin: CellId, event: &crate::model::Event) -> String {
        if event.text.starts_with("Rain reaches the valley") && self.exposure_millis(origin) >= 1000
        {
            return "The rain wets your clothes.".into();
        }
        if event.text == "A branch rots away in the open. You hear branches breaking."
            || event.text == "The last branch rots away in the open. You hear a branch break."
        {
            if let Some(&cell) = event.affected_cells.first() {
                let (gone, sound) = if event.text.starts_with("The last branch") {
                    ("The last branch rots away", "You hear a branch break.")
                } else {
                    ("A branch rots away", "You hear branches breaking.")
                };
                return format!(
                    "{gone} {}. {sound}",
                    self.event_place(origin, cell, event.time)
                );
            }
        }
        if let Some(rest) = event.text.strip_prefix("A deer steps ") {
            if let Some(dir) = rest.strip_suffix(". A branch snapped under the deer.") {
                if let Scope::Cell(cell) = event.scope {
                    return format!(
                        "A deer steps {dir}. A branch snapped under the deer {}.",
                        self.event_place(origin, cell, event.time)
                    );
                }
            }
        }
        event.text.clone()
    }

    /// The open wood south of the outside floor, named from whoever hears it.
    fn event_place(&self, origin: CellId, cell: CellId, time: u64) -> String {
        let weather = self.weather_at(cell, time);
        if origin == cell {
            return format!("here, {weather}");
        }
        if let Some(floors) = self.named_wood_floors() {
            if cell == floors.wood {
                if self.cell(origin).rock {
                    let (x, y) = cell_xy(floors.wood);
                    let side = cell_id(x, y - 1)
                        .and_then(|outside| orthogonal_step(outside, floors.wood))
                        .unwrap_or("south");
                    if self.open_loose_branches(cell) {
                        return format!(
                            "on the rotting wood, the next square {side} of the outside floor, {weather}"
                        );
                    }
                    return format!("on the next square {side} of the outside floor, {weather}");
                }
                return format!("east of the west floor, {weather}");
            }
        }
        format!("{}, {weather}", self.cell_offset(origin, cell))
    }

    fn weather_at(&self, cell: CellId, time: u64) -> &'static str {
        if self.cell(cell).rock {
            "under the rock overhang"
        } else if time >= self.tuning.rain_starts_at {
            "in the rain"
        } else {
            "in the open"
        }
    }

    fn named_wood_floors(&self) -> Option<FloorsOffWood> {
        self.floors_off_the_rotting_wood()
            .or_else(|| self.tracked_wood_floors())
    }

    /// The same square after the pile is gone. The tracks still mark it.
    fn tracked_wood_floors(&self) -> Option<FloorsOffWood> {
        for y in 0..crate::model::HEIGHT {
            for x in 0..crate::model::WIDTH {
                let Some(wood) = cell_id(x, y) else {
                    continue;
                };
                let Some(outside) = cell_id(x, y - 1) else {
                    continue;
                };
                let Some(rock) = cell_id(x + 1, y - 1) else {
                    continue;
                };
                if !self.cell(rock).rock
                    || !self.tracks.contains(&wood)
                    || !self.tracks.contains(&outside)
                {
                    continue;
                }
                let Some(west) = cell_id(x - 1, y) else {
                    continue;
                };
                let Some(south) = cell_id(x - 1, y + 1) else {
                    continue;
                };
                let Some(far) = cell_id(x - 2, y + 1) else {
                    continue;
                };
                if !self.cell(far).trail {
                    continue;
                }
                let Some(side) = orthogonal_step(west, wood) else {
                    continue;
                };
                return Some(FloorsOffWood {
                    wood,
                    west,
                    south,
                    far,
                    side,
                });
            }
        }
        None
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
            .any(|cell| self.cell(*cell).rock)
        {
            actions.push("examine rock".into());
        }
        if self
            .visible_cell_ids(actor_id)
            .iter()
            .any(|cell| self.cell(*cell).region == crate::model::Region::Slope)
        {
            actions.push("examine slope".into());
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
        if !self.loose_ids(actor_id, Material::Carcass).is_empty()
            || !self.ground_ids(actor.cell, Material::Carcass).is_empty()
        {
            actions.push("eat carcass".into());
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
        if !self.ground_ids(actor.cell, Material::Spear).is_empty()
            && self.carry_count(actor_id) < self.tuning.carry_slots
        {
            actions.push("gather spear".into());
        }
        if !self.ground_ids(actor.cell, Material::Carcass).is_empty()
            && self.carry_count(actor_id) < self.tuning.carry_slots
        {
            actions.push("gather carcass".into());
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
        if !self.loose_ids(actor_id, Material::Spear).is_empty() {
            actions.push("drop spear".into());
        }
        if !self.loose_ids(actor_id, Material::Carcass).is_empty() {
            actions.push("drop carcass".into());
        }
        if self
            .loose_ids(actor_id, Material::Spear)
            .iter()
            .any(|id| self.item(*id).is_some_and(|spear| spear.hardened))
            && self
                .deer
                .as_ref()
                .is_some_and(|deer| chebyshev(actor.cell, deer.cell) <= 1)
        {
            actions.push("hunt".into());
        }
        if !self.loose_ids(actor_id, Material::Stone).is_empty()
            && !self.loose_ids(actor_id, Material::Branch).is_empty()
        {
            actions.push("bind spear".into());
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

fn carried_food_phrase(material: Material, count: u32, condition: &str) -> String {
    let (one, many) = match material {
        Material::Mushroom => ("mushroom", "mushrooms"),
        Material::Berry => ("handful of berries", "handfuls of berries"),
        Material::BitterBerry => ("handful of poison berries", "handfuls of poison berries"),
        _ => return format!("{count} {}", pile_noun(material, count)),
    };
    if count == 1 {
        format!("A {condition} {one}")
    } else {
        format!("{} {condition} {many}", capitalize(&count_word(count)))
    }
}

fn cell_span(count: u32) -> String {
    let word = if count == 1 {
        "one".to_string()
    } else {
        count_word(count)
    };
    let noun = if count == 1 { "cell" } else { "cells" };
    format!("{word} {noun}")
}

fn count_word(count: u32) -> String {
    match count {
        2 => "two".into(),
        3 => "three".into(),
        4 => "four".into(),
        5 => "five".into(),
        6 => "six".into(),
        7 => "seven".into(),
        8 => "eight".into(),
        9 => "nine".into(),
        10 => "ten".into(),
        11 => "eleven".into(),
        12 => "twelve".into(),
        other => other.to_string(),
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
        (Material::Spear, _) => "spear",
        (Material::Carcass, _) => "carcass",
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
        Material::Spear => ("spear", "is"),
        Material::Carcass => ("carcass", "is"),
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

#[derive(Clone, Copy)]
struct OutsideFloor {
    wood_side: &'static str,
    rock_side: &'static str,
    wood: CellId,
}

#[derive(Clone, Copy)]
struct FloorsOffWood {
    wood: CellId,
    west: CellId,
    south: CellId,
    far: CellId,
    side: &'static str,
}

fn facing_back(dir: Dir) -> &'static str {
    match dir {
        Dir::North => "south",
        Dir::South => "north",
        Dir::East => "west",
        Dir::West => "east",
    }
}

fn orthogonal_step(from: CellId, to: CellId) -> Option<&'static str> {
    let (x1, y1) = cell_xy(from);
    let (x2, y2) = cell_xy(to);
    match (x2 - x1, y2 - y1) {
        (-1, 0) => Some("west"),
        (1, 0) => Some("east"),
        (0, -1) => Some("north"),
        (0, 1) => Some("south"),
        _ => None,
    }
}
