//! One observation function. Text, map, and JSON are views of the same
//! facts. Moisture numbers, tokens, and the cause log stay here.

use crate::model::{
    cell_id, cell_xy, chebyshev, ActorId, CellId, Dir, ExamineTarget, ItemId, Material, Role,
    Scope, World, DEER_ROUTE,
};
use serde::Serialize;

pub const LEGEND: &str = "\
@ you          p other
L log          w dry wood
n flint nodule
W wet wood     b bark
B wet bark     * fire
y berries      Y wet berries
m mushrooms    q dark berries
h pale berries
k pot          u water
s stone         c clay
o ochre        C copper ore   e copper bead
g iron ore     I bloom of iron
K iron knife   i copper awl
v bowl         j spear
a carcass      + more than the letter shows
d deer    x fox    r hare    z beaver    t otter
\" tracks
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
    pub ochre: String,
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
            legend: self.legend_for(actor_id),
            self_info: SelfInfo {
                id: actor.id,
                name: actor.name.clone(),
                x,
                y,
                region: self.cell(actor.cell).region.word().into(),
                under_roof: self.exposure_millis(actor.cell) < 1000,
                wetness: if actor.clothes_dry_at.is_some() {
                    "drying".into()
                } else {
                    self.clothes_word(actor.wetness).into()
                },
                thirst: self.thirst_word(actor.thirst).into(),
                hunger: self.hunger_word(actor.hunger).into(),
                illness: self.body_word(actor).into(),
                energy: self.energy_word(actor.energy).into(),
                cold: self.cold_word(actor.cold).into(),
                wind: self.wind_on(actor.id).into(),
                sunburn: self.sun_word(actor.sunburn).into(),
                mud: self.mud_word(actor.mud).into(),
                ochre: self.ochre_word(actor.ochre).into(),
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
            ExamineTarget::Pot => self.examine_pot(actor_id),
            ExamineTarget::Mud => self.examine_material(actor_id, Material::Mud),
            ExamineTarget::Stone => self.examine_material(actor_id, Material::Stone),
            ExamineTarget::Flint => self.examine_flint(actor_id),
            ExamineTarget::Ochre => self.examine_ochre(actor_id),
            ExamineTarget::CopperOre => self.examine_copper_ore(actor_id),
            ExamineTarget::IronOre => self.examine_iron_ore(actor_id),
            ExamineTarget::Iron => self.examine_iron(actor_id),
            ExamineTarget::Knife => self.examine_knife(actor_id),
            ExamineTarget::Copper => self.examine_copper(actor_id),
            ExamineTarget::Awl => self.examine_awl(actor_id),
            ExamineTarget::Flake => self.examine_flake(actor_id),
            ExamineTarget::Clay => self.examine_clay(actor_id),
            ExamineTarget::Bowl => self.examine_bowl(actor_id),
            ExamineTarget::Spear => self.examine_spear(actor_id),
            ExamineTarget::Carcass => self.examine_carcass(actor_id),
            ExamineTarget::Meat => self.examine_meat(actor_id),
            ExamineTarget::Fish => self.examine_fish(actor_id),
            ExamineTarget::Brand => self.examine_brand(actor_id),
            ExamineTarget::Basket => self.examine_basket(actor_id),
            ExamineTarget::Charcoal => self.examine_charcoal(actor_id),
            ExamineTarget::Reeds => self.examine_reeds(actor_id),
            ExamineTarget::Snare => self.examine_snare(actor_id),
            ExamineTarget::Weir => self.examine_weir(actor_id),
            ExamineTarget::Bone => self.examine_bone(actor_id),
            ExamineTarget::Salt => self.examine_salt(actor_id),
            ExamineTarget::Honey => self.examine_honey(actor_id),
            ExamineTarget::Eggs => self.examine_eggs(actor_id),
            ExamineTarget::Moss => self.examine_moss(actor_id),
            ExamineTarget::Mint => self.examine_mint(actor_id),
            ExamineTarget::Needle => self.examine_needles(actor_id),
            ExamineTarget::Drill => self.examine_drill(actor_id),
            ExamineTarget::Resin => self.examine_resin(actor_id),
            ExamineTarget::Willow => self.examine_willow(actor_id),
            ExamineTarget::WillowBark => self.examine_willow_bark(actor_id),
            ExamineTarget::Fluff => self.examine_fluff(actor_id),
            ExamineTarget::Nettle => self.examine_nettle(actor_id),
            ExamineTarget::Cattail => self.examine_cattail(actor_id),
            ExamineTarget::BarkCup => self.examine_bark_cup(actor_id),
            ExamineTarget::Bow => self.examine_bow(actor_id),
            ExamineTarget::Paddle => self.examine_paddle(actor_id),
            ExamineTarget::Hook => self.examine_hook(actor_id),
            ExamineTarget::Net => self.examine_net(actor_id),
            ExamineTarget::Mat => self.examine_mat(actor_id),
            ExamineTarget::Canoe => self.examine_canoe(actor_id),
            ExamineTarget::PaleBerries => self.examine_material(actor_id, Material::PaleBerry),
            ExamineTarget::Hide => self.examine_hide(actor_id),
            ExamineTarget::Cord => self.examine_cord(actor_id),
            ExamineTarget::Fire => self.examine_fire(actor_id),
            ExamineTarget::Water => self.examine_water(actor_id),
            ExamineTarget::Deer => self.examine_deer(actor_id),
            ExamineTarget::Hare => self.examine_hare(actor_id),
            ExamineTarget::Tracks => self.examine_tracks(actor_id),
            ExamineTarget::Rock => self.examine_rock(actor_id),
            ExamineTarget::Slope => self.examine_slope(actor_id),
            ExamineTarget::Person(name) => {
                let origin = actor.cell;
                if actor.name.eq_ignore_ascii_case(&name) {
                    return self.self_text(actor_id);
                }
                let named = self
                    .actors
                    .values()
                    .find(|other| other.name.eq_ignore_ascii_case(&name))
                    .map(|other| (other.id, other.cell, other.name.clone()));
                match named {
                    Some((id, cell, _)) if chebyshev(origin, cell) <= self.tuning.sight => {
                        self.person_text(id)
                    }
                    Some((_, _, display)) => {
                        format!("You cannot see {display} from here.")
                    }
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
        if let Some(soaked) = self.soaked_open_berries(actor_id) {
            lines.push(soaked);
        }
        if let Some(rot) = self.rot_line(actor_id) {
            lines.push(rot);
        }
        lines.push(self.here_text(actor_id));
        lines.push(self.self_text(actor_id));
        if let Some(work) = self.work_view(actor_id) {
            lines.push(work.detail);
        }
        if self.dark_for(actor_id) {
            if self
                .seen_deer_tracks(actor_id)
                .is_some_and(|tracks| tracks.contains("Deer tracks mark the ground here."))
            {
                lines.push("Deer tracks mark the ground here.".into());
            }
            if self.wolf_standing()
                == Some(self.actor(actor_id).map(|actor| actor.cell).unwrap_or(0))
            {
                lines.push("A wolf stands here.".into());
            }
            if let Some(otter) = self.seen_otter(actor_id) {
                lines.push(otter);
            }
            if let Some(beaver) = self.seen_beaver(actor_id) {
                lines.push(beaver);
            }
            if let Some(fox) = self.seen_fox(actor_id) {
                lines.push(fox);
            }
            if let Some(hare) = self.seen_hare(actor_id) {
                lines.push(hare);
                lines.push("Strip willow south of the brook, then set a snare.".into());
            }
            if self
                .seen_wolf_tracks(actor_id)
                .is_some_and(|tracks| tracks.contains("Wolf tracks mark the ground here."))
            {
                lines.push("Wolf tracks mark the ground here.".into());
            }
            if let Some(marks) = self.night_far_landmarks(actor_id) {
                lines.push(marks);
            }
            if let Some(smoke) = self.seen_smoke(actor_id) {
                lines.push(smoke);
            }
            if let Some(column) = self.seen_signal(actor_id) {
                lines.push(column);
            }
            lines.push("The dark hides what lies farther off.".into());
        } else {
            if let Some(deer) = self.deer_look_line(actor_id) {
                lines.push(deer);
            }
            if let Some(tracks) = self.seen_deer_tracks(actor_id) {
                lines.push(tracks);
            }
            if let Some(wolf) = self.seen_wolf(actor_id) {
                lines.push(wolf);
            }
            if let Some(otter) = self.seen_otter(actor_id) {
                lines.push(otter);
            }
            if let Some(beaver) = self.seen_beaver(actor_id) {
                lines.push(beaver);
            }
            if let Some(fox) = self.seen_fox(actor_id) {
                lines.push(fox);
            }
            if let Some(hare) = self.seen_hare(actor_id) {
                lines.push(hare);
                lines.push("Strip willow south of the brook, then set a snare.".into());
            }
            if let Some(tracks) = self.seen_wolf_tracks(actor_id) {
                lines.push(tracks);
            }
            if let Some(person) = self.person_on_the_trail_west_of_the_south_floor(actor_id) {
                lines.push(person);
            }
            if let Some(person) = self.person_on_the_berries_north_of_the_west_floor(actor_id) {
                lines.push(person);
            }
            if let Some(seen) = self.seen_under_the_rock(actor_id) {
                lines.push(seen);
            }
            if let Some(flint) = self.seen_flint(actor_id) {
                lines.push(flint);
            }
            if let Some(berries) = self.seen_berry_patches(actor_id) {
                lines.push(berries);
            }
            if let Some(mushrooms) = self.seen_mushroom_patches(actor_id) {
                lines.push(mushrooms);
            }
            if let Some(branches) = self.seen_rock_branches(actor_id) {
                lines.push(branches);
            }
            if let Some(bark) = self.seen_rock_bark(actor_id) {
                lines.push(bark);
            }
            if let Some(ochre) = self.seen_ochre(actor_id) {
                lines.push(ochre);
            }
            if let Some(quartz) = self.seen_quartz(actor_id) {
                lines.push(quartz);
            }
            if let Some(ore) = self.seen_copper_ore(actor_id) {
                lines.push(ore);
            }
            if let Some(iron) = self.seen_iron_ore(actor_id) {
                lines.push(iron);
            }
            if let Some(clay) = self.seen_clay(actor_id) {
                lines.push(clay);
            }
            if let Some(reeds) = self.seen_reeds(actor_id) {
                lines.push(reeds);
            }
            if let Some(net) = self.seen_net(actor_id) {
                lines.push(net);
            }
            if let Some(hung) = self.seen_hanging_fish(actor_id) {
                lines.push(hung);
            }
            if let Some(mat) = self.seen_mat(actor_id) {
                lines.push(mat);
            }
            if let Some(canoe) = self.seen_canoe(actor_id) {
                lines.push(canoe);
            }
            if let Some(lean_to) = self.seen_lean_to(actor_id) {
                lines.push(lean_to);
            }
            if let Some(quiet) = self.seen_quiet_brook(actor_id) {
                lines.push(quiet);
            }
            if let Some(mark) = self.seen_ochre_mark(actor_id) {
                lines.push(mark);
            }
            if let Some(mark) = self.seen_charcoal_mark(actor_id) {
                lines.push(mark);
            }
            if let Some(snare) = self.seen_snare(actor_id) {
                lines.push(snare);
            }
            if let Some(weir) = self.seen_weir(actor_id) {
                lines.push(weir);
            }
            if let Some(meat) = self.seen_meat(actor_id) {
                lines.push(meat);
            }
            if let Some(bone) = self.seen_bone(actor_id) {
                lines.push(bone);
            }
            if let Some(hook) = self.seen_hook(actor_id) {
                lines.push(hook);
            }
            if let Some(salt) = self.seen_salt(actor_id) {
                lines.push(salt);
            }
            if let Some(honey) = self.seen_honey(actor_id) {
                lines.push(honey);
            }
            if let Some(eggs) = self.seen_eggs(actor_id) {
                lines.push(eggs);
            }
            if let Some(moss) = self.seen_moss(actor_id) {
                lines.push(moss);
            }
            if let Some(worms) = self.seen_worms(actor_id) {
                lines.push(worms);
            }
            if let Some(mint) = self.seen_mint(actor_id) {
                lines.push(mint);
            }
            if let Some(resin) = self.seen_resin(actor_id) {
                lines.push(resin);
            }
            if let Some(needles) = self.seen_needles(actor_id) {
                lines.push(needles);
            }
            if let Some(willow) = self.seen_willow(actor_id) {
                lines.push(willow);
            }
            if let Some(nettles) = self.seen_nettle(actor_id) {
                lines.push(nettles);
            }
            if let Some(cattails) = self.seen_cattail(actor_id) {
                lines.push(cattails);
            }
            if let Some(ice) = self.seen_frozen_pool(actor_id) {
                lines.push(ice);
            }
            if let Some(open) = self.seen_open_pool(actor_id) {
                lines.push(open);
            }
            if let Some(thawed) = self.seen_thawed_pool(actor_id) {
                lines.push(thawed);
            }
            if let Some(smoke) = self.seen_smoke(actor_id) {
                lines.push(smoke);
            }
            if let Some(column) = self.seen_signal(actor_id) {
                lines.push(column);
            }
            if let Some(whistle) = self.seen_whistle(actor_id) {
                lines.push(whistle);
            }
            if let Some(flute) = self.seen_flute(actor_id) {
                lines.push(flute);
            }
            if let Some(drum) = self.seen_drum(actor_id) {
                lines.push(drum);
            }
        }
        lines.push("Around you:".into());
        for dir in Dir::all() {
            lines.push(self.adjacent_line(actor_id, actor.cell, dir));
        }
        if !self.leftover_handful(actor.cell) {
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

    /// Open berries cross into soaked on the wet-season rain. Shelter keeps
    /// a pile dry, so this sentence is not about the rock.
    fn soaked_open_berries(&self, viewer: ActorId) -> Option<String> {
        if !self.wet_season || !self.raining {
            return None;
        }
        let origin = self.actor(viewer)?.cell;
        let mut lines = Vec::new();
        for cell in self.visible_cell_ids(viewer) {
            if cell == origin || self.exposure_millis(cell) < 1000 {
                continue;
            }
            let way = self.relative(origin, cell);
            if self.pile_count(cell, Material::BitterBerry) > 0
                && self.pile_moisture(cell, Material::BitterBerry) >= self.tuning.soaked_at
            {
                lines.push(format!(
                    "The {} to the {way} are soaked.",
                    self.bitter_kind(viewer)
                ));
            }
            if self.pile_count(cell, Material::Berry) > 0
                && self.pile_moisture(cell, Material::Berry) >= self.tuning.soaked_at
            {
                lines.push(format!("The berries to the {way} are soaked."));
            }
        }
        if lines.is_empty() {
            None
        } else {
            Some(lines.join(" "))
        }
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
                            "The last branch is rotting to the {}.",
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
        if self.night() && self.raining {
            "Night has fallen. Rain is falling on the valley."
        } else if self.night() {
            "Night has fallen. The stars are out over the valley."
        } else if self.morning() && self.raining {
            "Morning is on the valley. Rain is falling on the valley."
        } else if self.morning() {
            "Morning is on the valley. The sky is clear."
        } else if self.raining {
            "Rain is falling on the valley."
        } else if self.time + 2 >= self.tuning.rain_starts_at {
            "The light is going. The air smells like rain."
        } else {
            "The valley is still dry. The sky is clear. Evening is coming."
        }
    }

    /// The first night starts at `night_at` and lasts `night_len` ticks.
    /// Then `day_len` ticks of morning, and the pair repeats.
    pub(crate) fn night(&self) -> bool {
        if self.time < self.tuning.night_at {
            return false;
        }
        let cycle = self.tuning.night_len.saturating_add(self.tuning.day_len);
        if cycle == 0 {
            return false;
        }
        (self.time - self.tuning.night_at) % cycle < self.tuning.night_len
    }

    pub(crate) fn morning(&self) -> bool {
        self.time >= self.tuning.night_at && !self.night()
    }

    /// Night wind, from the same shelter the cold uses. The open rushes.
    /// A little wind is only a whisper. Rock and a thatched lean-to say nothing.
    /// The line is look, not a tick event.
    fn night_wind_line(&self, actor_id: ActorId) -> Option<String> {
        if !self.night() {
            return None;
        }
        match self.wind_on(actor_id) {
            "in the wind" => Some("You hear the wind rush.".into()),
            "in a little wind" => Some("The wind is a whisper here.".into()),
            _ => None,
        }
    }

    /// An owl on the north slope, some night ticks and not others.
    /// The line is look, not a tick event. Out of sight, the direction
    /// is from the listener. Under the rock or a thatched lean-to it is muffled.
    fn owl_hoots(&self) -> bool {
        self.night() && self.time % 3 == 0
    }

    fn owl_roost(&self) -> CellId {
        crate::model::cell_id(4, 0).unwrap()
    }

    fn owl_sheltered(&self, cell: CellId) -> bool {
        self.thatched.contains(&cell) || self.exposure_millis(cell) == 0
    }

    fn owl_line(&self, actor_id: ActorId) -> Option<String> {
        if !self.owl_hoots() {
            return None;
        }
        let origin = self.actor(actor_id)?.cell;
        let roost = self.owl_roost();
        if chebyshev(origin, roost) > self.tuning.hearing {
            return None;
        }
        if self.owl_sheltered(origin) {
            return Some("You hear an owl, muffled.".into());
        }
        let distance = chebyshev(origin, roost);
        if distance == 0 {
            return Some("An owl hoots here.".into());
        }
        if distance <= self.tuning.sight {
            return Some("An owl hoots.".into());
        }
        let way = self.relative(origin, roost);
        Some(format!("You hear an owl hoot to the {way}."))
    }

    /// Dusk and evening, some ticks and not others. Night and morning are quiet.
    /// The nearest brook or still water inside hearing is the water they sit on.
    /// The line is look, not a tick event.
    fn frogs_call(&self) -> bool {
        !self.night() && !self.morning() && self.time % 3 == 1
    }

    fn nearest_frog_water(&self, origin: CellId) -> Option<CellId> {
        let mut best: Option<(i32, CellId)> = None;
        for cell in &self.cells {
            if cell.region != crate::model::Region::Brook && !cell.still {
                continue;
            }
            let dist = chebyshev(origin, cell.id);
            if dist > self.tuning.hearing {
                continue;
            }
            let take = match best {
                None => true,
                Some((near, id)) => dist < near || (dist == near && cell.id < id),
            };
            if take {
                best = Some((dist, cell.id));
            }
        }
        best.map(|(_, id)| id)
    }

    fn frog_line(&self, actor_id: ActorId) -> Option<String> {
        if !self.frogs_call() {
            return None;
        }
        let origin = self.actor(actor_id)?.cell;
        let water = self.nearest_frog_water(origin)?;
        let distance = chebyshev(origin, water);
        if distance == 0 {
            return Some("Frogs croak here.".into());
        }
        if distance <= self.tuning.sight {
            return Some("Frogs croak.".into());
        }
        let way = self.relative(origin, water);
        Some(format!("You hear frogs croak to the {way}."))
    }

    /// Clear morning, some ticks and not others. The first look after the dawn
    /// lets the rain up is one of them. A morning look that still says the rain
    /// is falling stays quiet, and so do night and evening.
    /// Under the rock or a thatched lean-to it is faint. The line is look, not a tick event.
    fn birds_sing(&self) -> bool {
        if !self.morning() || self.raining {
            return false;
        }
        let first_clear = self
            .tuning
            .night_at
            .saturating_add(self.tuning.night_len)
            .saturating_add(1);
        self.time == first_clear || self.time % 3 == 0
    }

    fn bird_line(&self, actor_id: ActorId) -> Option<String> {
        if !self.birds_sing() {
            return None;
        }
        let origin = self.actor(actor_id)?.cell;
        if self.thatched.contains(&origin) || self.exposure_millis(origin) == 0 {
            return Some("You hear birdsong, faint.".into());
        }
        Some("You hear birdsong.".into())
    }

    /// A fire underfoot or on the next cell. Farther light does not help.
    fn fire_nearby(&self, cell: CellId) -> bool {
        self.fires
            .values()
            .any(|fire| chebyshev(cell, fire.cell) <= 1)
    }

    fn dark_from(&self, cell: CellId) -> bool {
        self.night() && !self.fire_nearby(cell)
    }

    /// The next cell is the rock, and no fire is close enough to light the ground.
    /// The rest of that cell stays dark.
    fn night_compass_mark(&self, origin: CellId, dir: Dir) -> Option<String> {
        let (x, y) = cell_xy(origin);
        let (dx, dy) = dir.delta();
        let dest = cell_id(x + dx, y + dy)?;
        if !self.cell(dest).rock {
            return None;
        }
        Some(format!("{}: the rock overhang.", capitalize(dir.word())))
    }

    /// A rock or a burning fire inside sight, farther than the next cell.
    /// The compass already names a rock on the next cell. The ground stays dark.
    fn night_far_landmarks(&self, viewer: ActorId) -> Option<String> {
        let origin = self.actor(viewer)?.cell;
        let mut lines = Vec::new();
        for cell in self.visible_cell_ids(viewer) {
            if cell == origin || orthogonal_neighbor(origin, cell) {
                continue;
            }
            let rock = self.cell(cell).rock;
            let fire = self.fire_at(cell).cloned();
            if !rock && fire.is_none() {
                continue;
            }
            let way = self.relative(origin, cell);
            let mut line = match (&fire, rock) {
                (Some(fire), true) if fire.charcoal => {
                    format!("A hot charcoal fire burns under the rock overhang to the {way}.")
                }
                (Some(fire), true) if fire.hot => {
                    format!("A fire burns hotter under the rock overhang to the {way}.")
                }
                (Some(_), true) => {
                    format!("A small fire burns under the rock overhang to the {way}.")
                }
                (None, true) => format!("The rock overhang is to the {way}."),
                (Some(fire), false) if fire.charcoal => {
                    format!("A hot charcoal fire burns to the {way}.")
                }
                (Some(fire), false) if fire.hot => format!("A fire burns hotter to the {way}."),
                (Some(_), false) => format!("A small fire burns to the {way}."),
                (None, false) => continue,
            };
            if fire.is_some_and(|fire| fire.column) {
                line.push_str(" A column of smoke rises from the fire.");
            }
            if !lines.contains(&line) {
                lines.push(line);
            }
        }
        if lines.is_empty() {
            None
        } else {
            Some(lines.join(" "))
        }
    }

    fn dark_for(&self, viewer: ActorId) -> bool {
        self.actor(viewer)
            .is_some_and(|actor| self.dark_from(actor.cell))
    }

    fn here_text(&self, actor_id: ActorId) -> String {
        let actor = self.actor(actor_id).unwrap();
        let cell = self.cell(actor.cell);
        let outside = self.outside_floor(actor.cell);
        let aboard = self
            .aboard
            .get(&actor.id)
            .copied()
            .filter(|id| self.canoe_floats(*id));
        let mut parts = Vec::new();
        if let Some(id) = aboard {
            let hull = if self.canoe_pitched(id) {
                "pitched dugout canoe"
            } else {
                "dugout canoe"
            };
            parts.push(format!("You are aboard the {hull} in the brook."));
            let riders: Vec<_> = self
                .aboard
                .iter()
                .filter(|(who, canoe)| **canoe == id && **who != actor.id)
                .map(|(who, _)| self.name_of(*who))
                .collect();
            if riders.len() == 1 {
                parts.push(format!("{} is aboard it.", riders[0]));
            } else if riders.len() > 1 {
                parts.push(format!("{} are aboard it.", riders.join(" and ")));
            }
            parts.push(format!("You are {}.", self.cover_phrase(actor.cell)));
        } else if let Some(floor) = outside {
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
        if self.lean_tos.contains(&actor.cell) {
            parts.push(self.lean_to_here_line(actor.cell));
        }
        if cell.region == crate::model::Region::Brook {
            parts.push("The water here is raw.".into());
        }
        if self
            .brook_stretch(actor.cell)
            .is_some_and(|stretch| self.brook_is_quiet(stretch))
        {
            parts.push("The brook is quiet. The fish are gone.".into());
        }
        if cell.still && cell.region != crate::model::Region::Brook {
            if self.still_frozen(actor.cell) && self.ice_hole(actor.cell) {
                parts.push("The still water is frozen. A hole is open in the ice.".into());
            } else if self.still_frozen(actor.cell) {
                parts.push("The still water is frozen. Ice covers it.".into());
            } else if self.still_would_freeze(actor.cell) {
                parts.push("The fire is keeping the still water from freezing.".into());
            } else if self.time >= self.tuning.chill_at {
                parts.push("The day has thawed the still water.".into());
            } else {
                parts.push("Still water lies here. It is not the brook.".into());
            }
        }
        if let Some(fire) = self.fire_at(actor.cell) {
            let kept = if self.raining && self.lean_tos.contains(&actor.cell) {
                " The lean-to keeps the fire."
            } else {
                ""
            };
            if fire.charcoal {
                parts.push(format!(
                    "A hot charcoal fire burns here. It warms you.{kept}"
                ));
            } else if fire.hot {
                parts.push(format!("A fire burns hotter here. It warms you.{kept}"));
            } else {
                parts.push(format!("A small fire burns here. It warms you.{kept}"));
            }
            if fire.column {
                parts.push("A column of smoke rises from the fire.".into());
            }
            if self.banks.contains_key(&actor.cell) {
                parts.push("Branches are banked on the fire under a cover of mud.".into());
            }
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
                if self.branches_lying(actor.cell) {
                    parts.push("A deer stands on these branches.".into());
                } else if !self.deer_on_empty_rock_ground(actor.cell) {
                    parts.push("A deer stands here.".into());
                }
            }
            _ if berries_here => {
                parts.push(format!(
                    "Berries lie here, {}. They are good to eat raw.",
                    self.condition_word(self.pile_moisture(actor.cell, Material::Berry))
                ));
            }
            _ => {}
        }
        let leftover = self.leftover_handful(actor.cell);
        if !berries_here && self.spent_berries.contains(&actor.cell) && !leftover {
            if self.deer_ate_berries.contains(&actor.cell) {
                if deer_here && !self.deer_on_empty_rock_ground(actor.cell) {
                    parts
                        .push("A deer stands here after eating the berries that were here.".into());
                } else if !deer_here {
                    if let Some(deer_cell) = self.deer.as_ref().and_then(|deer| {
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
                                let way = self.relative(actor.cell, deer_cell);
                                parts.push(format!(
                                "The berries that were here are gone. The deer {place} ate them and stands to the {way}."
                            ));
                            } else if self.deer_stands_past_the_wood(actor.cell) {
                                parts.push(
                                    "The berries that were here are gone. The deer ate them."
                                        .into(),
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
                }
            } else {
                parts.push("The berries that were here are gone.".into());
            }
        }
        if leftover {
            let rest = if self.deer_ate_berries.contains(&actor.cell) {
                " A deer ate the rest."
            } else {
                ""
            };
            parts.push(format!(
                "A handful of berries lies here. It is good to eat raw.{rest}"
            ));
        }
        let still_eating = self.deer_meal_this_tick(actor.cell) == Some(false);
        let deer_named_standing_here = deer_here
            && !berries_here
            && self.spent_berries.contains(&actor.cell)
            && self.deer_ate_berries.contains(&actor.cell);
        if self.deer_on_empty_rock_ground(actor.cell) {
            parts.push("A deer stands on the rock ground.".into());
        }
        if deer_here
            && !berries_here
            && !self.open_loose_branches(actor.cell)
            && !still_eating
            && !deer_named_standing_here
            && !self.deer_on_empty_rock_ground(actor.cell)
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
            let known = if self.bitter_known(actor_id) {
                " They taste bitter. They are poisonous."
            } else {
                ""
            };
            parts.push(format!(
                "{} lie here, {}.{}",
                self.bitter_kind_cap(actor_id),
                self.condition_word(self.pile_moisture(actor.cell, Material::BitterBerry)),
                known
            ));
        } else if self.berry_patch_bare(actor.cell, Material::BitterBerry) {
            parts.push(format!(
                "The {} that were here are gone.",
                self.bitter_kind(actor_id)
            ));
        }
        if self.pile_count(actor.cell, Material::PaleBerry) > 0 {
            let known = if self.pale_berry_known(actor_id) {
                " They must be cooked."
            } else {
                ""
            };
            parts.push(format!(
                "Pale berries lie here, {}.{known}",
                self.condition_word(self.pile_moisture(actor.cell, Material::PaleBerry))
            ));
        } else if self.berry_patch_bare(actor.cell, Material::PaleBerry) {
            parts.push("The pale berries that were here are gone.".into());
        }
        let cooked_pale = self
            .ground_ids(actor.cell, Material::PaleBerry)
            .into_iter()
            .filter(|id| self.item(*id).is_some_and(|berry| berry.cooked))
            .count();
        if cooked_pale == 1 {
            parts.push("A cooked handful of pale berries lies here. It is good to eat.".into());
        } else if cooked_pale > 1 {
            parts.push(format!(
                "{} cooked handfuls of pale berries lie here. They are good to eat.",
                count_word(cooked_pale as u32)
            ));
        }
        if self.pile_count(actor.cell, Material::Mushroom) > 0 {
            let known = if self.raw_mushroom_known(actor_id) {
                " They must be cooked. Raw mushrooms do not feed you."
            } else {
                ""
            };
            parts.push(format!(
                "Mushrooms lie here, {}.{known}",
                self.condition_word(self.pile_moisture(actor.cell, Material::Mushroom))
            ));
        } else if self.mushroom_patch_bare(actor.cell) {
            parts.push("The mushrooms that were here are gone.".into());
        }
        let cooked_mushrooms = self
            .ground_ids(actor.cell, Material::Mushroom)
            .into_iter()
            .filter(|id| self.item(*id).is_some_and(|mushroom| mushroom.cooked))
            .count();
        if cooked_mushrooms == 1 {
            parts.push("A cooked mushroom lies here.".into());
        } else if cooked_mushrooms > 1 {
            parts.push(format!(
                "{} cooked mushrooms lie here.",
                count_word(cooked_mushrooms as u32)
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
            let worst = self.branch_moisture_here(actor.cell);
            if !self.is_dry(worst) {
                parts.push(format!("The branches are {}.", self.condition_word(worst)));
            }
        } else if !deer_on_this_wood && branches_here == 1 {
            let drying = self
                .ground_ids(actor.cell, Material::Branch)
                .iter()
                .any(|id| {
                    self.item(*id)
                        .is_some_and(|item| item.firing_ready_at.is_some())
                });
            let word = self.condition_word(self.branch_moisture_here(actor.cell));
            if drying {
                parts.push(format!(
                    "A {word} branch lies here, drying by the fire. It is not dry yet."
                ));
            } else {
                parts.push(format!("A {word} branch lies here."));
            }
        } else if self.branch_bare(actor.cell) && !self.deer_on_empty_rock_ground(actor.cell) {
            parts.push("There are no branches under the rock.".into());
        }
        if self.loose_ids(actor_id, Material::Branch).iter().any(|id| {
            self.item(*id)
                .is_some_and(|item| item.firing_ready_at.is_some())
        }) {
            let moisture = self
                .loose_ids(actor_id, Material::Branch)
                .iter()
                .filter_map(|id| self.item(*id))
                .filter(|item| item.firing_ready_at.is_some())
                .map(|item| item.moisture)
                .max()
                .unwrap_or(0);
            parts.push(format!(
                "The {} branch in your arms is drying by the fire. It is not dry yet.",
                self.condition_word(moisture)
            ));
        }
        for id in self.ground_ids(actor.cell, Material::Brand) {
            parts.push("A burning brand lies here.".into());
        }
        for id in self.ground_ids(actor.cell, Material::Fish) {
            parts.push(self.fish_ground_line(id));
        }
        for _id in self.ground_ids(actor.cell, Material::Basket) {
            parts.push("A bark basket lies here.".into());
        }
        for id in self.ground_ids(actor.cell, Material::BarkCup) {
            parts.push(format!("{} lies here.", self.bark_cup_name(id, true)));
        }
        let charcoal_here = self.ground_ids(actor.cell, Material::Charcoal).len();
        if charcoal_here == 1 {
            parts.push("A lump of charcoal lies here.".into());
        } else if charcoal_here > 1 {
            parts.push("Lumps of charcoal lie here.".into());
        }
        let reeds_here = self.pile_count(actor.cell, Material::Reed)
            + self.ground_ids(actor.cell, Material::Reed).len() as u32;
        if reeds_here > 0 && self.beside_the_brook(actor.cell) {
            parts.push("Reeds grow here, at the brook's edge.".into());
        } else if reeds_here == 1 {
            parts.push("A bundle of reeds lies here.".into());
        } else if reeds_here > 1 {
            parts.push("Bundles of reeds lie here.".into());
        } else if self.reed_bare(actor.cell) {
            parts.push("The brook's edge has no reeds.".into());
        }
        let cattails_here = self.pile_count(actor.cell, Material::Cattail)
            + self.ground_ids(actor.cell, Material::Cattail).len() as u32;
        if cattails_here > 0 && self.cattail_stands.contains(&actor.cell) {
            if self.still_frozen(actor.cell) {
                parts.push(
                    "Cattails stand under the ice. The roots cannot be dug. They are not the reeds."
                        .into(),
                );
                if self.pile_count(actor.cell, Material::Fluff) == 0
                    && self.ground_ids(actor.cell, Material::Fluff).is_empty()
                {
                    parts.push("The cattails here have no fluff.".into());
                }
            } else {
                parts.push(
                    "Cattails grow here, in the shallows. The roots can be dug. They are not the reeds."
                        .into(),
                );
                if self.pile_count(actor.cell, Material::Fluff) > 0
                    || !self.ground_ids(actor.cell, Material::Fluff).is_empty()
                {
                    parts.push("Cattail fluff hangs on them. It is not the roots.".into());
                } else {
                    parts.push("The cattails here have no fluff.".into());
                }
            }
        } else if cattails_here > 0 && self.still_frozen(actor.cell) {
            parts.push("Cattail roots lie under the ice.".into());
        } else if cattails_here == 1 {
            parts.push("Cattail roots lie here.".into());
        } else if cattails_here > 1 {
            parts.push("Cattail roots lie here.".into());
        } else if self.cattail_bare(actor.cell) {
            parts.push("The shallows here have no cattails.".into());
        }
        let flour_ids = self.ground_ids(actor.cell, Material::Flour);
        let flatbreads = flour_ids
            .iter()
            .filter(|id| self.item(**id).is_some_and(|flour| flour.cooked))
            .count();
        let raw_flour = flour_ids.len() - flatbreads;
        if raw_flour > 0 {
            parts.push("Cattail flour lies here.".into());
        }
        if flatbreads == 1 {
            parts.push("A cattail flatbread lies here.".into());
        } else if flatbreads > 1 {
            parts.push("Cattail flatbreads lie here.".into());
        }
        let mats_here = self.ground_ids(actor.cell, Material::Mat).len();
        if mats_here == 1 {
            parts.push("A reed mat lies here.".into());
        } else if mats_here > 1 {
            parts.push("Reed mats lie here.".into());
        }
        if self.set_nets.contains(&actor.cell) {
            parts.push("A fishing net is set here.".into());
        } else {
            let nets_here = self.ground_ids(actor.cell, Material::Net).len();
            if nets_here == 1 {
                parts.push("A fishing net lies here.".into());
            } else if nets_here > 1 {
                parts.push("Fishing nets lie here.".into());
            }
        }
        let bark_here = self.pile_count(actor.cell, Material::Bark)
            + self.ground_ids(actor.cell, Material::Bark).len() as u32;
        let bark_lesson = if self.bark_known(actor.id) {
            " Bark does not feed you."
        } else {
            ""
        };
        if bark_here > 1 {
            parts.push(format!("A pile of bark lies here.{bark_lesson}"));
        } else if bark_here == 1 {
            parts.push(format!("A sheet of bark lies here.{bark_lesson}"));
        } else if self.bark_bare(actor.cell) {
            parts.push("There is no bark under the rock.".into());
        }
        if self.pile_count(actor.cell, Material::Log) > 0
            || self.items.values().any(|item| {
                item.material == Material::Log && item.cell == actor.cell && item.carrier.is_none()
            })
        {
            if self.log_charred_at(actor.cell) {
                parts.push("The heavy log lies here. The fire has charred it.".into());
            } else {
                parts.push("The heavy log lies here.".into());
            }
        }
        if let Some(id) = self.canoe_at(actor.cell) {
            if aboard != Some(id) {
                let hull = if self.canoe_pitched(id) {
                    "pitched dugout canoe"
                } else {
                    "dugout canoe"
                };
                if self.canoe_floats(id) {
                    parts.push(format!("A {hull} floats here in the brook."));
                } else {
                    parts.push(format!("A {hull} lies here."));
                }
                let riders: Vec<_> = self
                    .aboard
                    .iter()
                    .filter(|(who, canoe)| **canoe == id && **who != actor.id)
                    .map(|(who, _)| self.name_of(*who))
                    .collect();
                if riders.len() == 1 {
                    parts.push(format!("{} is aboard it.", riders[0]));
                } else if riders.len() > 1 {
                    parts.push(format!("{} are aboard it.", riders.join(" and ")));
                }
            }
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
        if self.pile_count(actor.cell, Material::Flint) > 0
            || !self.ground_ids(actor.cell, Material::Flint).is_empty()
        {
            parts.push("A flint nodule lies here.".into());
        } else if self.flint_bare(actor.cell) {
            parts.push("The slope has no flint.".into());
        }
        if self.pile_count(actor.cell, Material::Ochre) > 0
            || !self.ground_ids(actor.cell, Material::Ochre).is_empty()
        {
            if self.cell(actor.cell).rock {
                parts.push("A lump of ochre lies on the rock face.".into());
            } else {
                parts.push("A lump of ochre lies here.".into());
            }
        } else if self.ochre_bare(actor.cell) {
            parts.push("The slope has no ochre.".into());
        }
        if self.pile_count(actor.cell, Material::Quartz) > 0
            || !self.ground_ids(actor.cell, Material::Quartz).is_empty()
        {
            if self.quartz_faces.contains(&actor.cell) || self.cell(actor.cell).rock {
                parts.push(
                    "A clear stone of quartz lies on the rock face. It is a clear stone. It can spark."
                        .into(),
                );
            } else {
                parts.push("A clear stone of quartz lies here. It can spark.".into());
            }
        } else if self.quartz_faces.contains(&actor.cell) {
            parts.push("The rock face has no quartz.".into());
        }
        if self.pile_count(actor.cell, Material::Salt) > 0
            || !self.ground_ids(actor.cell, Material::Salt).is_empty()
        {
            if self.salt_pans.contains(&actor.cell) {
                parts.push("A salt crust lies on the dry margin of the still.".into());
            } else {
                parts.push("A salt crust lies here.".into());
            }
        } else if self.salt_pans.contains(&actor.cell)
            && self.salt_returns_at.contains_key(&actor.cell)
        {
            parts.push("The still margin is dry.".into());
        } else if self.salt_lick_bare(actor.cell) {
            parts.push("The slope is bare.".into());
        }
        if self.pile_count(actor.cell, Material::Honey) > 0
            || !self.ground_ids(actor.cell, Material::Honey).is_empty()
        {
            parts.push("A bees' nest hangs in the tree here.".into());
            if self.smoked_nests.contains(&actor.cell) {
                parts.push("Smoke hangs in the bees' nest.".into());
            } else if let Some(buzz) = self.bees_buzz_line(actor.cell, actor.cell) {
                parts.push(buzz);
            }
        } else if self.honey_nests.contains(&actor.cell) {
            parts.push("The bees' nest is empty.".into());
        }
        if self.pile_count(actor.cell, Material::Egg) > 0
            || !self.ground_ids(actor.cell, Material::Egg).is_empty()
        {
            parts.push("A bird's nest sits in the tree here. It holds eggs.".into());
        } else if self.bird_nests.contains(&actor.cell) {
            parts.push("The bird's nest in the tree here is empty.".into());
        }
        if self.pile_count(actor.cell, Material::Moss) > 0
            || !self.ground_ids(actor.cell, Material::Moss).is_empty()
        {
            if self.shade_moss.contains(&actor.cell) {
                parts.push("Dry moss grows in the shade here.".into());
            } else {
                parts.push("Moss grows on the damp bank here.".into());
            }
        } else if self.shade_bare(actor.cell) {
            parts.push("The shade here has no moss.".into());
        } else if self.moss_bare(actor.cell) {
            parts.push("The damp bank here is bare.".into());
        }
        if self.pile_count(actor.cell, Material::Worm) > 0
            || !self.ground_ids(actor.cell, Material::Worm).is_empty()
        {
            parts.push("Worms are in the damp earth here.".into());
        } else if self.worm_beds.contains(&actor.cell) || self.damp_earth(actor.cell) {
            parts.push("The damp earth here has no worms.".into());
        }
        if self.pile_count(actor.cell, Material::Mint) > 0
            || !self.ground_ids(actor.cell, Material::Mint).is_empty()
        {
            parts.push("Wild mint grows on the wet bank here.".into());
        } else if self.mint_bare(actor.cell) {
            parts.push("The wet bank here has no mint.".into());
        }
        let resin_here = self.pile_count(actor.cell, Material::Resin)
            + self.ground_ids(actor.cell, Material::Resin).len() as u32;
        if resin_here > 1 {
            parts.push("Lumps of pitch lie here. It is pine resin.".into());
        } else if resin_here == 1 {
            parts.push("A lump of pitch lies here. It is pine resin.".into());
        } else if self.resin_bare(actor.cell) {
            parts.push("The pine here has no resin.".into());
        }
        if self.pile_count(actor.cell, Material::Needle) > 0
            || !self.ground_ids(actor.cell, Material::Needle).is_empty()
        {
            parts.push("Pine needles lie under the pine here.".into());
        } else if self.needle_bare(actor.cell) {
            parts.push("The pine here has no needles.".into());
        }
        let willow_here = self.pile_count(actor.cell, Material::Willow)
            + self.ground_ids(actor.cell, Material::Willow).len() as u32;
        if willow_here > 1 {
            parts.push(
                "Bundles of willow cordage hang here. They are withies from a willow.".into(),
            );
        } else if willow_here == 1 {
            parts
                .push("A bundle of willow cordage hangs here. It is withies from a willow.".into());
        } else if self.willow_bare(actor.cell) {
            parts.push("The willow here has no withies.".into());
        }
        let bark_here = self.pile_count(actor.cell, Material::WillowBark)
            + self.ground_ids(actor.cell, Material::WillowBark).len() as u32;
        if bark_here > 1 {
            parts.push(
                "Strips of willow bark hang on the willow here. They are not the cordage.".into(),
            );
        } else if bark_here == 1 {
            parts.push(
                "A strip of willow bark hangs on the willow here. It is not the cordage.".into(),
            );
        } else if self.willow_bark_bare(actor.cell) {
            parts.push("The willow here has no bark.".into());
        }
        let nettle_here = self.pile_count(actor.cell, Material::Nettle)
            + self.ground_ids(actor.cell, Material::Nettle).len() as u32;
        if nettle_here > 1 {
            parts.push("Nettles grow here. They can be stripped for cordage.".into());
        } else if nettle_here == 1 {
            parts.push("A stand of nettles grows here. It can be stripped for cordage.".into());
        } else if self.nettle_bare(actor.cell) {
            parts.push("The nettles here are bare.".into());
        }
        let iron_smelting = self.iron_smelting_count(actor.cell);
        let iron_lying = self.pile_count(actor.cell, Material::IronOre)
            + self.ground_ids(actor.cell, Material::IronOre).len() as u32
            - iron_smelting as u32;
        if iron_lying > 0 {
            parts.push("A lump of iron ore lies here.".into());
        } else if self.iron_bare(actor.cell) {
            parts.push("The slope has no iron ore.".into());
        }
        if iron_smelting == 1 {
            parts.push(format!(
                "A lump of iron ore is smelting in {}.",
                self.iron_smelt_fire(actor.cell)
            ));
        } else if iron_smelting > 1 {
            parts.push(format!(
                "Lumps of iron ore are smelting in {}.",
                self.iron_smelt_fire(actor.cell)
            ));
        }
        if !self.ground_ids(actor.cell, Material::Iron).is_empty() {
            parts.push("A bloom of iron lies here. It is metal from a hotter fire.".into());
        }
        if !self.ground_ids(actor.cell, Material::Knife).is_empty() {
            parts.push("An iron knife lies here. It can cut.".into());
        }
        if self.pile_count(actor.cell, Material::CopperOre) > 0
            || self.ground_ids(actor.cell, Material::CopperOre).len()
                > self.copper_smelting_count(actor.cell)
        {
            parts.push("A lump of copper ore lies here.".into());
        } else if self.copper_bare(actor.cell) {
            parts.push("The brook has no copper ore.".into());
        }
        let smelting = self.copper_smelting_count(actor.cell);
        if smelting == 1 {
            parts.push(format!(
                "A lump of copper ore is smelting in {}.",
                self.copper_smelt_fire(actor.cell)
            ));
        } else if smelting > 1 {
            parts.push(format!(
                "Lumps of copper ore are smelting in {}.",
                self.copper_smelt_fire(actor.cell)
            ));
        }
        if !self.ground_ids(actor.cell, Material::Copper).is_empty() {
            parts.push("A copper bead lies here. It is metal from the fire.".into());
        }
        if self
            .ground_ids(actor.cell, Material::Awl)
            .iter()
            .any(|id| self.item(*id).is_some_and(|awl| !awl.bone))
        {
            parts.push("A copper awl lies here.".into());
        }
        if self
            .ground_ids(actor.cell, Material::Awl)
            .iter()
            .any(|id| self.item(*id).is_some_and(|awl| awl.bone))
        {
            parts.push("A bone awl lies here. It can pierce hide.".into());
        }
        if !self.ground_ids(actor.cell, Material::Bone).is_empty() {
            parts.push("A bone lies here.".into());
        } else if self.bone_cache_bare(actor.cell) {
            parts.push("The rock has no bone.".into());
        }
        if !self.ground_ids(actor.cell, Material::Hook).is_empty() {
            parts.push("A bone fish hook lies here.".into());
        }
        if self.ochre_marks.contains(&actor.cell) {
            let surface = if self.cell(actor.cell).rock {
                "rock"
            } else {
                "ground"
            };
            parts.push(format!("A red ochre mark is on the {surface} here."));
        }
        if self.charcoal_marks.contains(&actor.cell) {
            let surface = if self.cell(actor.cell).rock {
                "rock"
            } else {
                "ground"
            };
            parts.push(format!("A black charcoal mark is on the {surface} here."));
        }
        let firing = self.clay_firing_count(actor.cell);
        let loose_clay = self
            .ground_ids(actor.cell, Material::Clay)
            .iter()
            .filter(|id| self.item(**id).is_some_and(|clay| !clay.shaped))
            .count() as u32;
        let clay_pots = self
            .ground_ids(actor.cell, Material::Clay)
            .iter()
            .filter(|id| {
                self.item(**id)
                    .is_some_and(|clay| clay.shaped && clay.firing_ready_at.is_none())
            })
            .count();
        let pot_firing = self.pot_firing_count(actor.cell);
        let clay_pile = self.pile_count(actor.cell, Material::Clay);
        if firing == 1 {
            parts.push("A lump of clay is hardening in the fire.".into());
        } else if firing > 1 {
            parts.push("Lumps of clay are hardening in the fire.".into());
        }
        if pot_firing == 1 {
            parts.push("An unfired clay pot is hardening in the fire. It is not done yet.".into());
        } else if pot_firing > 1 {
            parts
                .push("Unfired clay pots are hardening in the fire. They are not done yet.".into());
        }
        if clay_pile > 1 {
            parts.push("Clay lies on the bank here.".into());
        } else if clay_pile == 1 || (loose_clay > firing as u32) {
            parts.push("A lump of clay lies here. It is unfired.".into());
        } else if self.clay_bare(actor.cell) {
            parts.push("There is no clay on the bank.".into());
        }
        if clay_pots == 1 {
            parts.push("An unfired clay pot lies here. It is not hard yet.".into());
        } else if clay_pots > 1 {
            parts.push("Unfired clay pots lie here. They are not hard yet.".into());
        }
        if !self.ground_ids(actor.cell, Material::Bowl).is_empty() {
            parts.push("A fired bowl sits here. It can boil water over a fire.".into());
        }
        let carcasses = self.ground_ids(actor.cell, Material::Carcass);
        if carcasses
            .iter()
            .any(|id| self.item(*id).is_some_and(|carcass| carcass.otter))
        {
            parts.push("An otter carcass lies here.".into());
        }
        if carcasses
            .iter()
            .any(|id| self.item(*id).is_some_and(|carcass| carcass.wolf))
        {
            parts.push("A wolf carcass lies here.".into());
        }
        if carcasses
            .iter()
            .any(|id| self.item(*id).is_some_and(|carcass| carcass.fox))
        {
            parts.push("A fox carcass lies here.".into());
        }
        if carcasses
            .iter()
            .any(|id| self.item(*id).is_some_and(|carcass| carcass.hare))
        {
            parts.push("A hare carcass lies here.".into());
        }
        if carcasses
            .iter()
            .any(|id| self.item(*id).is_some_and(|carcass| carcass.beaver))
        {
            parts.push("A beaver carcass lies here.".into());
        }
        if carcasses.iter().any(|id| {
            self.item(*id).is_some_and(|carcass| {
                !carcass.wolf && !carcass.otter && !carcass.fox && !carcass.hare && !carcass.beaver
            })
        }) {
            parts.push("A deer carcass lies here.".into());
        }
        if let Some(line) = self.meat_here_line(actor.cell) {
            parts.push(line);
        }
        if !self.ground_ids(actor.cell, Material::Cord).is_empty() {
            parts.push("Hide strips lie here. They are cordage.".into());
        }
        let bows_here = self.ground_ids(actor.cell, Material::Bow);
        if !bows_here.is_empty() {
            let willow = bows_here
                .iter()
                .any(|id| self.item(*id).is_some_and(|bow| bow.bound_willow));
            let nettle = bows_here
                .iter()
                .any(|id| self.item(*id).is_some_and(|bow| bow.bound_nettle));
            let hide = bows_here.iter().any(|id| {
                self.item(*id)
                    .is_some_and(|bow| !bow.bound_willow && !bow.bound_nettle)
            });
            if hide {
                parts.push("A bow lies here. It is a dry branch bound with hide strips.".into());
            }
            if willow {
                parts.push("A bow lies here. It is a dry branch bound with willow cordage.".into());
            }
            if nettle {
                parts.push("A bow lies here. It is a dry branch bound with nettle cordage.".into());
            }
        }
        if !self.ground_ids(actor.cell, Material::Paddle).is_empty() {
            parts.push("A wooden paddle lies here.".into());
        }
        let drills_here = self.ground_ids(actor.cell, Material::Drill);
        let spent_here = drills_here
            .iter()
            .filter(|id| self.item(**id).is_some_and(|drill| drill.spent))
            .count();
        if drills_here.len() > spent_here {
            parts.push("A bow drill lies here.".into());
        }
        if spent_here > 0 {
            parts.push("A spent bow drill lies here.".into());
        }
        if self.snares.contains_key(&actor.cell) {
            if self.snare_nettle.contains(&actor.cell) {
                parts.push("A nettle snare is set here.".into());
            } else if self.snare_willow.contains(&actor.cell) {
                parts.push("A willow snare is set here.".into());
            } else {
                parts.push("A hide snare is set here.".into());
            }
        }
        if self.weirs.contains_key(&actor.cell) {
            parts.push("A reed weir is set in the brook here.".into());
        }
        parts.extend(self.hide_here_lines(&self.ground_ids(actor.cell, Material::Hide)));
        for id in self.ground_ids(actor.cell, Material::Spear) {
            let spear = self.item(id);
            let hardened = spear.is_some_and(|item| item.hardened);
            if let Some(item) = spear.filter(|item| item.flint_tip) {
                parts.push(format!(
                    "A spear with {} lies here.",
                    self.flint_tip_words(item)
                ));
            } else if spear.is_some_and(|item| item.wooden_tip) {
                if hardened {
                    parts.push("A wooden spear lies here. The point is fire-hardened.".into());
                } else {
                    parts.push("A wooden spear lies here. The point is still green.".into());
                }
            } else if hardened {
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

    /// Frozen still water in sight. The person standing on it already has the place line.
    fn seen_frozen_pool(&self, viewer: ActorId) -> Option<String> {
        let origin = self.actor(viewer)?.cell;
        let mut lines = Vec::new();
        for cell in self.visible_cell_ids(viewer) {
            if cell == origin || !self.still_frozen(cell) {
                continue;
            }
            let way = self.relative(origin, cell);
            if self.ice_hole(cell) {
                lines.push(format!(
                    "The still water to the {way} is frozen. A hole is open in the ice."
                ));
            } else {
                lines.push(format!(
                    "The still water to the {way} is frozen. Ice covers it."
                ));
            }
        }
        if lines.is_empty() {
            None
        } else {
            Some(lines.join(" "))
        }
    }

    /// Deer tracks on a cell this person can see. The deer need not still be there.
    /// Past sight, nothing. The sentence fades when `track_ticks` have passed.
    fn seen_deer_tracks(&self, viewer: ActorId) -> Option<String> {
        let origin = self.actor(viewer)?.cell;
        let mut lines = Vec::new();
        for cell in self.visible_cell_ids(viewer) {
            if !self.deer_tracks_fresh(cell) {
                continue;
            }
            let line = if cell == origin {
                "Deer tracks mark the ground here.".to_string()
            } else {
                format!(
                    "Deer tracks mark the ground to the {}.",
                    self.relative(origin, cell)
                )
            };
            if !lines.contains(&line) {
                lines.push(line);
            }
        }
        if lines.is_empty() {
            None
        } else {
            Some(lines.join(" "))
        }
    }

    /// An otter on the brook, or on the bank beside it. Past sight, nothing.
    /// At night, with no fire nearby, only an otter underfoot is named.
    fn seen_otter(&self, viewer: ActorId) -> Option<String> {
        let origin = self.actor(viewer)?.cell;
        let otter = self.otter.as_ref()?;
        if self.dark_for(viewer) && otter.cell != origin {
            return None;
        }
        if !self.visible_cell_ids(viewer).contains(&otter.cell) {
            return None;
        }
        let brook = self.cell(otter.cell).region == crate::model::Region::Brook;
        if otter.cell == origin {
            if brook {
                Some("An otter is in the brook here.".into())
            } else {
                Some("An otter is on the bank here.".into())
            }
        } else {
            let way = self.relative(origin, otter.cell);
            if brook {
                Some(format!("An otter is in the brook to the {way}."))
            } else {
                Some(format!("An otter is on the bank to the {way}."))
            }
        }
    }

    /// A beaver on the south brook, or on the bank it dived to. Past sight, nothing.
    /// At night, with no fire nearby, only a beaver underfoot is named.
    fn seen_beaver(&self, viewer: ActorId) -> Option<String> {
        let origin = self.actor(viewer)?.cell;
        let beaver = self.beaver.as_ref()?;
        if self.dark_for(viewer) && beaver.cell != origin {
            return None;
        }
        if !self.visible_cell_ids(viewer).contains(&beaver.cell) {
            return None;
        }
        let brook = self.cell(beaver.cell).region == crate::model::Region::Brook;
        if beaver.cell == origin {
            if brook {
                Some("A beaver is in the brook here.".into())
            } else {
                Some("A beaver is on the bank here.".into())
            }
        } else {
            let way = self.relative(origin, beaver.cell);
            if brook {
                Some(format!("A beaver is in the brook to the {way}."))
            } else {
                Some(format!("A beaver is on the bank to the {way}."))
            }
        }
    }

    /// A fox on the dry valley floor. Past sight, nothing.
    /// At night, with no fire nearby, only a fox underfoot is named.
    fn seen_fox(&self, viewer: ActorId) -> Option<String> {
        let origin = self.actor(viewer)?.cell;
        let fox = self.fox.as_ref()?;
        if self.dark_for(viewer) && fox.cell != origin {
            return None;
        }
        if !self.visible_cell_ids(viewer).contains(&fox.cell) {
            return None;
        }
        if fox.cell == origin {
            Some("A fox is on the valley floor here.".into())
        } else {
            let way = self.relative(origin, fox.cell);
            Some(format!("A fox is on the valley floor to the {way}."))
        }
    }

    /// A hare on one dry cell of the valley floor. Past sight, nothing.
    /// At night, with no fire nearby, only a hare underfoot is named.
    fn seen_hare(&self, viewer: ActorId) -> Option<String> {
        let origin = self.actor(viewer)?.cell;
        let hare = self.hare.as_ref()?;
        if self.dark_for(viewer) && hare.cell != origin {
            return None;
        }
        if !self.visible_cell_ids(viewer).contains(&hare.cell) {
            return None;
        }
        if hare.cell == origin {
            Some("A hare is on the valley floor here.".into())
        } else {
            let way = self.relative(origin, hare.cell);
            Some(format!("A hare is on the valley floor to the {way}."))
        }
    }

    /// A wolf standing on ground this person can see. Past sight, nothing.
    /// At night, with no fire nearby, only a wolf underfoot is named. A fire
    /// nearby names the neighbor, the way day does. The wolf leaves after
    /// `wolf_ticks`, and the day has none.
    fn seen_wolf(&self, viewer: ActorId) -> Option<String> {
        let origin = self.actor(viewer)?.cell;
        let cell = self.wolf_standing()?;
        if !self.visible_cell_ids(viewer).contains(&cell) {
            return None;
        }
        if cell == origin {
            Some("A wolf stands here.".into())
        } else {
            Some(format!(
                "A wolf stands to the {}.",
                self.relative(origin, cell)
            ))
        }
    }

    /// Wolf tracks on ground this person can see. The wolf may already have left.
    /// Past sight, nothing. At night, with no fire nearby, only the tracks
    /// underfoot are named. A fire nearby names the neighbor tracks again,
    /// the way day does. They fade when `track_ticks` have passed.
    fn seen_wolf_tracks(&self, viewer: ActorId) -> Option<String> {
        let origin = self.actor(viewer)?.cell;
        let mut lines = Vec::new();
        for cell in self.visible_cell_ids(viewer) {
            if !self.wolf_tracks_fresh(cell) {
                continue;
            }
            let line = if cell == origin {
                "Wolf tracks mark the ground here.".to_string()
            } else {
                format!(
                    "Wolf tracks mark the ground to the {}.",
                    self.relative(origin, cell)
                )
            };
            if !lines.contains(&line) {
                lines.push(line);
            }
        }
        if lines.is_empty() {
            None
        } else {
            Some(lines.join(" "))
        }
    }

    /// A signal column past the ordinary smoke. One cell past that smoke
    /// is still a thing you see. One cell farther, you only smell it.
    /// Someone who can see the fire already has the fire sentence.
    /// The ordinary plume, one cell past sight, stays "Smoke rises".
    fn seen_signal(&self, viewer: ActorId) -> Option<String> {
        let origin = self.actor(viewer)?.cell;
        let see_at = self.tuning.smoke_range().saturating_add(1);
        let smell_at = see_at.saturating_add(1);
        let mut lines = Vec::new();
        for fire in self.fires.values() {
            if !fire.column {
                continue;
            }
            let distance = chebyshev(origin, fire.cell);
            let way = self.relative(origin, fire.cell);
            let line = if distance == see_at {
                format!("You see smoke to the {way}. Someone is signaling with smoke.")
            } else if distance == smell_at {
                format!("You smell smoke to the {way}. Someone is signaling with smoke.")
            } else {
                continue;
            };
            if !lines.contains(&line) {
                lines.push(line);
            }
        }
        if lines.is_empty() {
            None
        } else {
            Some(lines.join(" "))
        }
    }

    /// A bone whistle just blown, past sight. One cell past sight, and one
    /// cell farther, a look says which way. Someone who can see the blower
    /// already has the blow. It is not smoke, and it does not last.
    fn seen_whistle(&self, viewer: ActorId) -> Option<String> {
        let sound = self.whistle?;
        if self.time > sound.until {
            return None;
        }
        let origin = self.actor(viewer)?.cell;
        let distance = chebyshev(origin, sound.cell);
        let near = self.tuning.sight.saturating_add(1);
        let far = near.saturating_add(1);
        if distance != near && distance != far {
            return None;
        }
        let way = self.relative(origin, sound.cell);
        Some(format!("You hear a whistle to the {way}."))
    }

    /// A reed flute just blown, past sight. The same two cells as the whistle.
    /// Someone who can see the blower already has the blow. The hare stays.
    fn seen_flute(&self, viewer: ActorId) -> Option<String> {
        let sound = self.flute?;
        if self.time > sound.until {
            return None;
        }
        let origin = self.actor(viewer)?.cell;
        let distance = chebyshev(origin, sound.cell);
        let near = self.tuning.sight.saturating_add(1);
        let far = near.saturating_add(1);
        if distance != near && distance != far {
            return None;
        }
        let way = self.relative(origin, sound.cell);
        Some(format!("You hear a flute to the {way}."))
    }

    /// A drum just beaten, past sight. The whistle's two cells, and one cell
    /// farther. Someone who can see the beater already has the beat.
    /// The hare does not bolt.
    fn seen_drum(&self, viewer: ActorId) -> Option<String> {
        let sound = self.drum?;
        if self.time > sound.until {
            return None;
        }
        let origin = self.actor(viewer)?.cell;
        let distance = chebyshev(origin, sound.cell);
        let near = self.tuning.sight.saturating_add(1);
        let far = near.saturating_add(1);
        let farther = far.saturating_add(1);
        if distance != near && distance != far && distance != farther {
            return None;
        }
        let way = self.relative(origin, sound.cell);
        Some(format!(
            "You hear a drum to the {way}. Someone is beating a drum as a signal."
        ))
    }

    /// Smoke from a fire past sight, still inside the smoke range.
    /// Someone who can see the cell already has the fire sentence.
    fn seen_smoke(&self, viewer: ActorId) -> Option<String> {
        let origin = self.actor(viewer)?.cell;
        let mut lines = Vec::new();
        for fire in self.fires.values() {
            let distance = chebyshev(origin, fire.cell);
            if distance <= self.tuning.sight || distance > self.tuning.smoke_range() {
                continue;
            }
            let line = format!("Smoke rises to the {}.", self.relative(origin, fire.cell));
            if !lines.contains(&line) {
                lines.push(line);
            }
        }
        if lines.is_empty() {
            None
        } else {
            Some(lines.join(" "))
        }
    }

    /// Still water the day has thawed. The person standing on it already
    /// has the place line. A fire that is keeping the pool open is not this.
    fn seen_thawed_pool(&self, viewer: ActorId) -> Option<String> {
        if self.time < self.tuning.chill_at {
            return None;
        }
        let origin = self.actor(viewer)?.cell;
        let mut lines = Vec::new();
        for cell in self.visible_cell_ids(viewer) {
            if cell == origin || self.still_frozen(cell) || self.still_would_freeze(cell) {
                continue;
            }
            let place = self.cell(cell);
            if !place.still || place.region == crate::model::Region::Brook {
                continue;
            }
            lines.push(format!(
                "The still water to the {} has thawed.",
                self.relative(origin, cell)
            ));
        }
        if lines.is_empty() {
            None
        } else {
            Some(lines.join(" "))
        }
    }

    /// A fire on still water in sight, while the cold would freeze it. The
    /// person standing on it already has the place line. Before the fire,
    /// and once the day has thawed the pool, this stays quiet.
    fn seen_open_pool(&self, viewer: ActorId) -> Option<String> {
        if self.time < self.tuning.chill_at {
            return None;
        }
        let origin = self.actor(viewer)?.cell;
        let mut lines = Vec::new();
        for cell in self.visible_cell_ids(viewer) {
            if cell == origin || !self.still_would_freeze(cell) || self.fire_at(cell).is_none() {
                continue;
            }
            let place = self.cell(cell);
            if !place.still || place.region == crate::model::Region::Brook {
                continue;
            }
            lines.push(format!(
                "The fire is keeping the still water to the {} from freezing.",
                self.relative(origin, cell)
            ));
        }
        if lines.is_empty() {
            None
        } else {
            Some(lines.join(" "))
        }
    }

    /// The pile is gone and one loose handful is still on this ground.
    fn leftover_handful(&self, cell: CellId) -> bool {
        self.pile_count(cell, Material::Berry) == 0
            && !self.ground_ids(cell, Material::Berry).is_empty()
    }

    /// A berry patch in sight that has not grown back. The compass drops the
    /// fruit while it is gone. Someone on the patch already has the place line.
    fn seen_berry_patches(&self, viewer: ActorId) -> Option<String> {
        let origin = self.actor(viewer)?.cell;
        let mut lines = Vec::new();
        for cell in self.visible_cell_ids(viewer) {
            if cell == origin {
                continue;
            }
            let way = self.relative(origin, cell);
            if self.berry_patch_bare(cell, Material::BitterBerry) {
                lines.push(format!(
                    "The {} to the {way} are gone.",
                    self.bitter_kind(viewer)
                ));
            } else if self.berry_patch_bare(cell, Material::Berry)
                && !self.deer_ate_berries.contains(&cell)
            {
                lines.push(format!("The berries to the {way} are gone."));
            }
        }
        if lines.is_empty() {
            None
        } else {
            Some(lines.join(" "))
        }
    }

    /// A mushroom patch in sight that has not grown back. The compass drops
    /// the mushrooms while they are gone. Someone on the patch already has the place line.
    fn seen_mushroom_patches(&self, viewer: ActorId) -> Option<String> {
        let origin = self.actor(viewer)?.cell;
        let mut lines = Vec::new();
        for cell in self.visible_cell_ids(viewer) {
            if cell == origin {
                continue;
            }
            if self.mushroom_patch_bare(cell) {
                lines.push(format!(
                    "The mushrooms to the {} are gone.",
                    self.relative(origin, cell)
                ));
            }
        }
        if lines.is_empty() {
            None
        } else {
            Some(lines.join(" "))
        }
    }

    /// A flint nodule in sight, other than the one under the looker's feet.
    /// The compass may still list it. The sentence names the slope, as ochre and iron do.
    fn seen_flint(&self, viewer: ActorId) -> Option<String> {
        let origin = self.actor(viewer)?.cell;
        let mut lines = Vec::new();
        for cell in self.visible_cell_ids(viewer) {
            if cell == origin {
                continue;
            }
            let count = self.pile_count(cell, Material::Flint)
                + self.ground_ids(cell, Material::Flint).len() as u32;
            if count > 0 {
                let region = self.cell(cell).region.word();
                lines.push(format!(
                    "A flint nodule lies on the {region} to the {}.",
                    self.relative(origin, cell)
                ));
            } else if self.flint_bare(cell) {
                lines.push(format!(
                    "The slope to the {} has no flint.",
                    self.relative(origin, cell)
                ));
            }
        }
        if lines.is_empty() {
            None
        } else {
            Some(lines.join(" "))
        }
    }

    /// A lump of ochre in sight that the place line and the compass do not already name.
    fn seen_ochre(&self, viewer: ActorId) -> Option<String> {
        let origin = self.actor(viewer)?.cell;
        let mut lines = Vec::new();
        for cell in self.visible_cell_ids(viewer) {
            if cell == origin {
                continue;
            }
            let count = self.pile_count(cell, Material::Ochre)
                + self.ground_ids(cell, Material::Ochre).len() as u32;
            if count > 0 {
                if orthogonal_neighbor(origin, cell) {
                    continue;
                }
                let way = self.relative(origin, cell);
                if self.cell(cell).rock {
                    lines.push(format!(
                        "A lump of ochre lies on the rock face to the {way}."
                    ));
                } else {
                    let region = self.cell(cell).region.word();
                    lines.push(format!(
                        "A lump of ochre lies on the {region} to the {way}."
                    ));
                }
            } else if self.ochre_bare(cell) {
                lines.push(format!(
                    "The slope to the {} has no ochre.",
                    self.relative(origin, cell)
                ));
            }
        }
        if lines.is_empty() {
            None
        } else {
            Some(lines.join(" "))
        }
    }

    /// A clear stone of quartz on a rock face in sight, not underfoot.
    fn seen_quartz(&self, viewer: ActorId) -> Option<String> {
        let origin = self.actor(viewer)?.cell;
        let mut lines = Vec::new();
        for cell in self.visible_cell_ids(viewer) {
            if cell == origin || !self.quartz_faces.contains(&cell) {
                continue;
            }
            let way = self.relative(origin, cell);
            if self.pile_count(cell, Material::Quartz) > 0
                || !self.ground_ids(cell, Material::Quartz).is_empty()
            {
                lines.push(format!(
                    "A clear stone of quartz lies on the rock face to the {way}. It can spark."
                ));
            } else {
                lines.push(format!("The rock face to the {way} has no quartz."));
            }
        }
        if lines.is_empty() {
            None
        } else {
            Some(lines.join(" "))
        }
    }

    /// A lump of copper ore in sight, other than the one under the looker's feet.
    /// The compass may still list it. A neighbor hears the sentence too, as a mark does.
    fn seen_copper_ore(&self, viewer: ActorId) -> Option<String> {
        let origin = self.actor(viewer)?.cell;
        let mut lines = Vec::new();
        for cell in self.visible_cell_ids(viewer) {
            if cell == origin {
                continue;
            }
            let count = self.pile_count(cell, Material::CopperOre)
                + self
                    .ground_ids(cell, Material::CopperOre)
                    .into_iter()
                    .filter(|id| {
                        self.item(*id)
                            .and_then(|item| item.firing_ready_at)
                            .is_none()
                    })
                    .count() as u32;
            if count > 0 {
                let region = self.cell(cell).region.word();
                lines.push(format!(
                    "A lump of copper ore lies on the {region} to the {}.",
                    self.relative(origin, cell)
                ));
            } else if self.copper_bare(cell) {
                lines.push(format!(
                    "The brook to the {} has no copper ore.",
                    self.relative(origin, cell)
                ));
            }
        }
        if lines.is_empty() {
            None
        } else {
            Some(lines.join(" "))
        }
    }

    /// Standing under the branches. In the rain, the line says the smear stays.
    fn lean_to_here_line(&self, cell: CellId) -> String {
        let name = if self.thatched.contains(&cell) {
            "thatched lean-to"
        } else {
            "lean-to of branches"
        };
        if self.raining && self.thatched.contains(&cell) {
            "A thatched lean-to stands here. It breaks the wind. The rain does not reach you under it, and it does not wash you.".into()
        } else if self.raining {
            format!("A {name} stands here. It breaks the wind. Only a little rain reaches you under it, and it does not wash you.")
        } else {
            format!("A {name} stands here. It breaks the wind.")
        }
    }

    /// A lean-to in sight, other than the one under the looker's feet.
    fn seen_lean_to(&self, viewer: ActorId) -> Option<String> {
        let origin = self.actor(viewer)?.cell;
        let mut lines = Vec::new();
        for cell in self.visible_cell_ids(viewer) {
            if cell == origin || !self.lean_tos.contains(&cell) {
                continue;
            }
            let place = if self.cell(cell).rock {
                "rock overhang"
            } else if self.beside_the_brook(cell) {
                "brook bank"
            } else {
                self.cell(cell).region.word()
            };
            let dir = self.relative(origin, cell);
            let rain = if self.raining {
                let kept = if self.fire_at(cell).is_some() {
                    " The lean-to keeps the fire."
                } else {
                    ""
                };
                if self.thatched.contains(&cell) {
                    format!(" The rain does not reach anyone under it.{kept}")
                } else {
                    format!(" The rain does not wash anyone under it.{kept}")
                }
            } else {
                String::new()
            };
            let name = if self.thatched.contains(&cell) {
                "thatched lean-to"
            } else {
                "lean-to of branches"
            };
            lines.push(format!(
                "A {name} stands on the {place} to the {dir}.{rain}"
            ));
        }
        if lines.is_empty() {
            None
        } else {
            Some(lines.join(" "))
        }
    }

    /// A brook cell a cast has just emptied, seen from off that stretch.
    fn seen_quiet_brook(&self, viewer: ActorId) -> Option<String> {
        let origin = self.actor(viewer)?.cell;
        let own = self.brook_stretch(origin);
        let mut lines = Vec::new();
        for cell in self.visible_cell_ids(viewer) {
            if self.cell(cell).region != crate::model::Region::Brook || !self.brook_is_quiet(cell) {
                continue;
            }
            if own == Some(cell) {
                continue;
            }
            let line = format!("The brook to the {} is quiet.", self.relative(origin, cell));
            if !lines.contains(&line) {
                lines.push(line);
            }
        }
        if lines.is_empty() {
            None
        } else {
            Some(lines.join(" "))
        }
    }

    /// Reeds at a brook edge in sight, other than the ones under the looker's feet.
    fn seen_reeds(&self, viewer: ActorId) -> Option<String> {
        let origin = self.actor(viewer)?.cell;
        let mut lines = Vec::new();
        for cell in self.visible_cell_ids(viewer) {
            if cell == origin {
                continue;
            }
            let count = self.pile_count(cell, Material::Reed)
                + self.ground_ids(cell, Material::Reed).len() as u32;
            if count > 0 && self.beside_the_brook(cell) {
                lines.push(format!(
                    "Reeds grow at the brook's edge to the {}.",
                    self.relative(origin, cell)
                ));
            } else if self.reed_bare(cell) {
                lines.push(format!(
                    "The brook's edge to the {} has no reeds.",
                    self.relative(origin, cell)
                ));
            }
        }
        if lines.is_empty() {
            None
        } else {
            Some(lines.join(" "))
        }
    }

    /// A raw fish hung by a fire, or set on the next cell, in sight and not underfoot.
    /// Past sight it is not on the look.
    fn seen_hanging_fish(&self, viewer: ActorId) -> Option<String> {
        let origin = self.actor(viewer)?.cell;
        let mut lines = Vec::new();
        for cell in self.visible_cell_ids(viewer) {
            if cell == origin {
                continue;
            }
            for id in self.ground_ids(cell, Material::Fish) {
                let Some(fish) = self.item(id) else {
                    continue;
                };
                if fish.smoked || fish.cooked || fish.spoiled || fish.firing_ready_at.is_none() {
                    continue;
                }
                let way = self.relative(origin, cell);
                if self.fire_at(cell).is_some() {
                    lines.push(format!("A raw fish hangs by the fire to the {way}."));
                } else {
                    lines.push(format!("A raw fish is set near the fire to the {way}."));
                }
            }
        }
        if lines.is_empty() {
            None
        } else {
            Some(lines.join(" "))
        }
    }

    /// A fishing net in sight, other than the one under the looker's feet.
    fn seen_net(&self, viewer: ActorId) -> Option<String> {
        let origin = self.actor(viewer)?.cell;
        let mut lines = Vec::new();
        for cell in self.visible_cell_ids(viewer) {
            if cell == origin {
                continue;
            }
            if self.set_nets.contains(&cell) {
                let mut line = format!(
                    "A fishing net is set in the brook to the {}.",
                    self.relative(origin, cell)
                );
                if !self.ground_ids(cell, Material::Fish).is_empty() {
                    line.push_str(" A raw fish lies there.");
                }
                lines.push(line);
                continue;
            }
            let count = self.ground_ids(cell, Material::Net).len() as u32;
            if count == 1 {
                lines.push(format!(
                    "A fishing net lies to the {}.",
                    self.relative(origin, cell)
                ));
            } else if count > 1 {
                lines.push(format!(
                    "Fishing nets lie to the {}.",
                    self.relative(origin, cell)
                ));
            }
        }
        if lines.is_empty() {
            None
        } else {
            Some(lines.join(" "))
        }
    }

    /// A dugout in sight, other than the one under the looker's feet.
    fn seen_canoe(&self, viewer: ActorId) -> Option<String> {
        let origin = self.actor(viewer)?.cell;
        let mut lines = Vec::new();
        for cell in self.visible_cell_ids(viewer) {
            if cell == origin {
                continue;
            }
            let Some(id) = self.canoe_at(cell) else {
                continue;
            };
            let place = self.relative(origin, cell);
            let hull = if self.canoe_pitched(id) {
                "pitched dugout canoe"
            } else {
                "dugout canoe"
            };
            let line = if self.canoe_floats(id) {
                format!("A {hull} floats in the brook to the {place}.")
            } else {
                format!("A {hull} lies to the {place}.")
            };
            lines.push(line);
        }
        for cell in self.visible_cell_ids(viewer) {
            if cell == origin || !self.log_charred_at(cell) {
                continue;
            }
            lines.push(format!(
                "The heavy log to the {} is charred.",
                self.relative(origin, cell)
            ));
        }
        if lines.is_empty() {
            None
        } else {
            Some(lines.join(" "))
        }
    }

    /// A reed mat in sight, other than the one under the looker's feet.
    fn seen_mat(&self, viewer: ActorId) -> Option<String> {
        let origin = self.actor(viewer)?.cell;
        let mut lines = Vec::new();
        for cell in self.visible_cell_ids(viewer) {
            if cell == origin {
                continue;
            }
            let count = self.ground_ids(cell, Material::Mat).len();
            if count == 1 {
                lines.push(format!(
                    "A reed mat lies to the {}.",
                    self.relative(origin, cell)
                ));
            } else if count > 1 {
                lines.push(format!(
                    "Reed mats lie to the {}.",
                    self.relative(origin, cell)
                ));
            }
        }
        if lines.is_empty() {
            None
        } else {
            Some(lines.join(" "))
        }
    }

    /// Unfired clay in sight, other than the pile under the looker's feet.
    /// The seeded pile is on the bank beside the brook, so the sentence says so.
    fn seen_clay(&self, viewer: ActorId) -> Option<String> {
        let origin = self.actor(viewer)?.cell;
        let mut lines = Vec::new();
        for cell in self.visible_cell_ids(viewer) {
            if cell == origin {
                continue;
            }
            let count = self.unfired_clay_count(cell);
            let pots = self.clay_pot_count(cell);
            let firing = self.pot_firing_count(cell);
            let place = if self.beside_the_brook(cell) {
                "brook bank"
            } else {
                self.cell(cell).region.word()
            };
            let dir = self.relative(origin, cell);
            if count == 0 && pots == 0 && firing == 0 {
                if self.clay_bare(cell) {
                    lines.push(format!("There is no clay on the brook bank to the {dir}."));
                }
                continue;
            }
            if count == 1 {
                lines.push(format!("A lump of clay lies on the {place} to the {dir}."));
            } else if count > 1 {
                lines.push(format!("Clay lies on the {place} to the {dir}."));
            }
            if pots == 1 {
                lines.push(format!(
                    "An unfired clay pot lies on the {place} to the {dir}."
                ));
            } else if pots > 1 {
                lines.push(format!(
                    "Unfired clay pots lie on the {place} to the {dir}."
                ));
            }
            if firing == 1 {
                let spot = if self.cell(cell).rock {
                    format!("under the rock overhang to the {dir}")
                } else {
                    format!("on the {place} to the {dir}")
                };
                lines.push(format!(
                    "An unfired clay pot is hardening in the fire {spot}. It is not done yet."
                ));
            } else if firing > 1 {
                let spot = if self.cell(cell).rock {
                    format!("under the rock overhang to the {dir}")
                } else {
                    format!("on the {place} to the {dir}")
                };
                lines.push(format!(
                    "Unfired clay pots are hardening in the fire {spot}. They are not done yet."
                ));
            }
        }
        if lines.is_empty() {
            None
        } else {
            Some(lines.join(" "))
        }
    }

    fn unfired_clay_count(&self, cell: CellId) -> u32 {
        let loose = self
            .ground_ids(cell, Material::Clay)
            .iter()
            .filter(|id| {
                self.item(**id)
                    .is_some_and(|clay| !clay.shaped && clay.firing_ready_at.is_none())
            })
            .count() as u32;
        self.pile_count(cell, Material::Clay) + loose
    }

    fn unfired_pot_ready(&self, actor_id: ActorId) -> bool {
        let cell = self.actor(actor_id).map(|actor| actor.cell).unwrap_or(0);
        self.loose_ids(actor_id, Material::Clay)
            .iter()
            .chain(self.ground_ids(cell, Material::Clay).iter())
            .any(|id| {
                self.item(*id)
                    .is_some_and(|clay| clay.shaped && clay.firing_ready_at.is_none())
            })
    }

    fn clay_ready_to_shape(&self, actor_id: ActorId) -> bool {
        let cell = self.actor(actor_id).map(|actor| actor.cell).unwrap_or(0);
        self.loose_ids(actor_id, Material::Clay)
            .iter()
            .any(|id| self.item(*id).is_some_and(|clay| !clay.shaped))
            || self.ground_ids(cell, Material::Clay).iter().any(|id| {
                self.item(*id)
                    .is_some_and(|clay| !clay.shaped && clay.firing_ready_at.is_none())
            })
            || self.pile_count(cell, Material::Clay) > 0
    }

    fn clay_pot_count(&self, cell: CellId) -> usize {
        self.ground_ids(cell, Material::Clay)
            .iter()
            .filter(|id| {
                self.item(**id)
                    .is_some_and(|clay| clay.shaped && clay.firing_ready_at.is_none())
            })
            .count()
    }

    fn beside_the_brook(&self, cell: CellId) -> bool {
        // The reed bed stays the edge even where the brook now runs through it.
        if self.reed_beds.contains_key(&cell) {
            return true;
        }
        if self.cell(cell).region == crate::model::Region::Brook {
            return false;
        }
        let (x, y) = cell_xy(cell);
        [(1, 0), (-1, 0), (0, 1), (0, -1)].iter().any(|(dx, dy)| {
            cell_id(x + dx, y + dy)
                .is_some_and(|next| self.cell(next).region == crate::model::Region::Brook)
        })
    }

    /// A lump of iron ore in sight, other than the one under the looker's feet.
    fn seen_iron_ore(&self, viewer: ActorId) -> Option<String> {
        let origin = self.actor(viewer)?.cell;
        let mut lines = Vec::new();
        for cell in self.visible_cell_ids(viewer) {
            if cell == origin {
                continue;
            }
            let count = self.pile_count(cell, Material::IronOre)
                + self.ground_ids(cell, Material::IronOre).len() as u32;
            if count > 0 {
                let region = self.cell(cell).region.word();
                lines.push(format!(
                    "A lump of iron ore lies on the {region} to the {}.",
                    self.relative(origin, cell)
                ));
            } else if self.iron_bare(cell) {
                lines.push(format!(
                    "The slope to the {} has no iron ore.",
                    self.relative(origin, cell)
                ));
            }
        }
        if lines.is_empty() {
            None
        } else {
            Some(lines.join(" "))
        }
    }

    /// A red ochre mark in sight, other than the one under the looker's feet.
    /// The compass may still list it. This is the sentence, as a lump is named from off the cell.
    fn seen_ochre_mark(&self, viewer: ActorId) -> Option<String> {
        let origin = self.actor(viewer)?.cell;
        let mut lines = Vec::new();
        for cell in self.visible_cell_ids(viewer) {
            if cell == origin || !self.ochre_marks.contains(&cell) {
                continue;
            }
            let place = if self.cell(cell).rock {
                format!("on the rock to the {}", self.relative(origin, cell))
            } else {
                let region = self.cell(cell).region.word();
                format!(
                    "on the ground on the {region} to the {}",
                    self.relative(origin, cell)
                )
            };
            lines.push(format!("A red ochre mark is {place}."));
        }
        if lines.is_empty() {
            None
        } else {
            Some(lines.join(" "))
        }
    }

    /// Meat lying on a cell this person can see, other than under their feet.
    /// The compass still lists it on the next cell. A diagonal in sight gets
    /// the same sentence, because the compass does not name that square.
    fn seen_meat(&self, viewer: ActorId) -> Option<String> {
        let origin = self.actor(viewer)?.cell;
        let mut lines = Vec::new();
        for cell in self.visible_cell_ids(viewer) {
            if cell == origin {
                continue;
            }
            let ids = self.ground_ids(cell, Material::Meat);
            if ids.is_empty() {
                continue;
            }
            let place = if self.cell(cell).rock {
                format!("under the rock to the {}", self.relative(origin, cell))
            } else {
                let region = self.cell(cell).region.word();
                format!("on the {region} to the {}", self.relative(origin, cell))
            };
            lines.extend(self.meat_place_lines(&ids, &place));
        }
        if lines.is_empty() {
            None
        } else {
            Some(lines.join(" "))
        }
    }

    /// A salt crust on a cell this person can see but is not standing on.
    fn seen_salt(&self, viewer: ActorId) -> Option<String> {
        let origin = self.actor(viewer)?.cell;
        let mut lines = Vec::new();
        for cell in self.visible_cell_ids(viewer) {
            if cell == origin {
                continue;
            }
            let region = self.cell(cell).region.word();
            let way = self.relative(origin, cell);
            if self.pile_count(cell, Material::Salt) > 0
                || !self.ground_ids(cell, Material::Salt).is_empty()
            {
                if self.salt_pans.contains(&cell) {
                    lines.push(format!(
                        "A salt crust lies on the dry margin of the still to the {way}."
                    ));
                } else {
                    lines.push(format!("A salt crust lies on the {region} to the {way}."));
                }
            } else if self.salt_pans.contains(&cell) && self.salt_returns_at.contains_key(&cell) {
                lines.push(format!("The still margin to the {way} is dry."));
            } else if self.salt_lick_bare(cell) {
                lines.push(format!("The slope to the {way} is bare."));
            }
        }
        if lines.is_empty() {
            None
        } else {
            Some(lines.join(" "))
        }
    }

    fn examine_salt(&self, viewer: ActorId) -> String {
        let origin = self.actor(viewer).unwrap().cell;
        let mut lines = Vec::new();
        for cell in self.visible_cell_ids(viewer) {
            if self.pile_count(cell, Material::Salt) == 0
                && self.ground_ids(cell, Material::Salt).is_empty()
            {
                continue;
            }
            let place = if self.salt_pans.contains(&cell) && cell == origin {
                format!(
                    "on the dry margin of the still, {}",
                    self.cover_phrase(cell)
                )
            } else if self.salt_pans.contains(&cell) {
                format!(
                    "on the dry margin of the still to the {}, {}",
                    self.relative(origin, cell),
                    self.cover_phrase(cell)
                )
            } else {
                self.place_phrase(origin, cell)
            };
            lines.push(format!("A salt crust lies {place}. It can salt raw meat."));
        }
        for actor in self.actors.values() {
            if chebyshev(origin, actor.cell) > self.tuning.sight {
                continue;
            }
            if self.loose_ids(actor.id, Material::Salt).is_empty() {
                continue;
            }
            let who = if actor.id == viewer {
                "you"
            } else {
                actor.name.as_str()
            };
            lines.push(format!(
                "A salt crust is carried by {who}, {}. It can salt raw meat.",
                self.cover_phrase(actor.cell)
            ));
        }
        if lines.is_empty() {
            "You see no salt from here.".into()
        } else {
            lines.join(" ")
        }
    }

    /// Bees in a full nest, heard on that cell and the next. Smoke and an
    /// empty nest are quiet. Two cells off, the tree can still be seen.
    fn bees_buzz_line(&self, origin: CellId, cell: CellId) -> Option<String> {
        if self.smoked_nests.contains(&cell) || chebyshev(origin, cell) > 1 {
            return None;
        }
        let full = self.pile_count(cell, Material::Honey) > 0
            || !self.ground_ids(cell, Material::Honey).is_empty();
        if !full {
            return None;
        }
        if origin == cell {
            Some("You hear the bees buzzing.".into())
        } else {
            let way = self.relative(origin, cell);
            Some(format!("You hear the bees buzzing to the {way}."))
        }
    }

    /// A bees' nest in a tree this person can see but is not standing under.
    fn seen_honey(&self, viewer: ActorId) -> Option<String> {
        let origin = self.actor(viewer)?.cell;
        let mut lines = Vec::new();
        for cell in self.visible_cell_ids(viewer) {
            if cell == origin {
                continue;
            }
            let region = self.cell(cell).region.word();
            let way = self.relative(origin, cell);
            if self.pile_count(cell, Material::Honey) > 0
                || !self.ground_ids(cell, Material::Honey).is_empty()
            {
                if self.smoked_nests.contains(&cell) {
                    lines.push(format!(
                        "A bees' nest hangs in a tree on the {region} to the {way}. Smoke hangs in it."
                    ));
                } else {
                    lines.push(format!(
                        "A bees' nest hangs in a tree on the {region} to the {way}."
                    ));
                    if let Some(buzz) = self.bees_buzz_line(origin, cell) {
                        lines.push(buzz);
                    }
                }
            } else if self.honey_nests.contains(&cell) {
                lines.push(format!(
                    "The bees' nest in the tree on the {region} to the {way} is empty."
                ));
            }
        }
        if lines.is_empty() {
            None
        } else {
            Some(lines.join(" "))
        }
    }

    fn examine_honey(&self, viewer: ActorId) -> String {
        let origin = self.actor(viewer).unwrap().cell;
        let mut lines = Vec::new();
        for cell in self.visible_cell_ids(viewer) {
            let place = self.place_phrase(origin, cell);
            if self.pile_count(cell, Material::Honey) > 0
                || !self.ground_ids(cell, Material::Honey).is_empty()
            {
                if self.smoked_nests.contains(&cell) {
                    lines.push(format!(
                        "A bees' nest hangs in a tree {place}. Smoke hangs in it. The bees will not sting."
                    ));
                } else {
                    lines.push(format!(
                        "A bees' nest hangs in a tree {place}. The bees sting if you gather the honeycomb."
                    ));
                }
            } else if self.honey_nests.contains(&cell) {
                let cover = self.cover_phrase(cell);
                if cell == origin {
                    lines.push(format!("The bees' nest {cover} here is empty."));
                } else {
                    let way = self.relative(origin, cell);
                    lines.push(format!("The bees' nest {cover} to the {way} is empty."));
                }
            }
        }
        for actor in self.actors.values() {
            if chebyshev(origin, actor.cell) > self.tuning.sight {
                continue;
            }
            let count = self.loose_ids(actor.id, Material::Honey).len() as u32;
            if count == 0 {
                continue;
            }
            let who = if actor.id == viewer {
                "you"
            } else {
                actor.name.as_str()
            };
            let cover = self.cover_phrase(actor.cell);
            if count == 1 {
                lines.push(format!("A honeycomb is carried by {who} {cover}."));
            } else {
                lines.push(format!(
                    "{} honeycombs are carried by {who} {cover}.",
                    capitalize(&count_word(count))
                ));
            }
        }
        if lines.is_empty() {
            "You see no honey from here.".into()
        } else {
            lines.join(" ")
        }
    }

    /// A bird's nest in a tree this person can see but is not standing under.
    fn seen_eggs(&self, viewer: ActorId) -> Option<String> {
        let origin = self.actor(viewer)?.cell;
        let mut lines = Vec::new();
        for cell in self.visible_cell_ids(viewer) {
            if cell == origin {
                continue;
            }
            let region = self.cell(cell).region.word();
            let way = self.relative(origin, cell);
            if self.pile_count(cell, Material::Egg) > 0
                || !self.ground_ids(cell, Material::Egg).is_empty()
            {
                lines.push(format!(
                    "A bird's nest sits in a tree on the {region} to the {way}."
                ));
            } else if self.bird_nests.contains(&cell) {
                lines.push(format!(
                    "The bird's nest in the tree on the {region} to the {way} is empty."
                ));
            }
        }
        if lines.is_empty() {
            None
        } else {
            Some(lines.join(" "))
        }
    }

    fn examine_eggs(&self, viewer: ActorId) -> String {
        let origin = self.actor(viewer).unwrap().cell;
        let mut lines = Vec::new();
        for cell in self.visible_cell_ids(viewer) {
            let cover = self.cover_phrase(cell);
            let holds = self.pile_count(cell, Material::Egg) > 0
                || !self.ground_ids(cell, Material::Egg).is_empty();
            if !holds && !self.bird_nests.contains(&cell) {
                continue;
            }
            if cell == origin {
                if holds {
                    lines.push(format!(
                        "A bird's nest sits in a tree here, {cover}. It holds eggs."
                    ));
                } else {
                    lines.push(format!("The bird's nest {cover} here is empty."));
                }
            } else {
                let way = self.relative(origin, cell);
                if holds {
                    lines.push(format!(
                        "A bird's nest sits in a tree to the {way}, {cover}. It holds eggs."
                    ));
                } else {
                    lines.push(format!("The bird's nest {cover} to the {way} is empty."));
                }
            }
        }
        for actor in self.actors.values() {
            if chebyshev(origin, actor.cell) > self.tuning.sight {
                continue;
            }
            let ids = self.loose_ids(actor.id, Material::Egg);
            if ids.is_empty() {
                continue;
            }
            let who = if actor.id == viewer {
                "you"
            } else {
                actor.name.as_str()
            };
            let cover = self.cover_phrase(actor.cell);
            let cooked = ids
                .iter()
                .any(|id| self.item(*id).is_some_and(|eggs| eggs.cooked));
            if cooked {
                lines.push(format!(
                    "Cooked eggs are carried by {who} {cover}. They are good to eat."
                ));
            } else {
                lines.push(format!(
                    "Eggs are carried by {who} {cover}. They must be cooked before they feed you."
                ));
            }
        }
        if lines.is_empty() {
            "You see no eggs from here.".into()
        } else {
            lines.join(" ")
        }
    }

    /// Moss on a damp bank this person can see but is not standing on.
    fn seen_moss(&self, viewer: ActorId) -> Option<String> {
        let origin = self.actor(viewer)?.cell;
        let mut lines = Vec::new();
        for cell in self.visible_cell_ids(viewer) {
            if cell == origin {
                continue;
            }
            let way = self.relative(origin, cell);
            if self.pile_count(cell, Material::Moss) > 0
                || !self.ground_ids(cell, Material::Moss).is_empty()
            {
                if self.shade_moss.contains(&cell) {
                    lines.push(format!("Dry moss grows in the shade to the {way}."));
                } else {
                    lines.push(format!("Moss grows on the damp bank to the {way}."));
                }
            } else if self.shade_bare(cell) {
                lines.push(format!("The shade to the {way} has no moss."));
            } else if self.moss_bare(cell) {
                lines.push(format!("The damp bank to the {way} is bare."));
            }
        }
        if lines.is_empty() {
            None
        } else {
            Some(lines.join(" "))
        }
    }

    /// Worms in damp earth this person can see but is not standing on.
    fn seen_worms(&self, viewer: ActorId) -> Option<String> {
        let origin = self.actor(viewer)?.cell;
        let mut lines = Vec::new();
        for cell in self.visible_cell_ids(viewer) {
            if cell == origin || !self.worm_beds.contains(&cell) {
                continue;
            }
            let way = self.relative(origin, cell);
            if self.pile_count(cell, Material::Worm) > 0
                || !self.ground_ids(cell, Material::Worm).is_empty()
            {
                lines.push(format!("Worms are in the damp earth to the {way}."));
            } else {
                lines.push(format!("The damp earth to the {way} has no worms."));
            }
        }
        if lines.is_empty() {
            None
        } else {
            Some(lines.join(" "))
        }
    }

    /// Wild mint on a wet bank this person can see but is not standing on.
    fn seen_mint(&self, viewer: ActorId) -> Option<String> {
        let origin = self.actor(viewer)?.cell;
        let mut lines = Vec::new();
        for cell in self.visible_cell_ids(viewer) {
            if cell == origin {
                continue;
            }
            let way = self.relative(origin, cell);
            if self.pile_count(cell, Material::Mint) > 0
                || !self.ground_ids(cell, Material::Mint).is_empty()
            {
                lines.push(format!("Wild mint grows on the wet bank to the {way}."));
            } else if self.mint_bare(cell) {
                lines.push(format!("The wet bank to the {way} has no mint."));
            }
        }
        if lines.is_empty() {
            None
        } else {
            Some(lines.join(" "))
        }
    }

    fn examine_mint(&self, viewer: ActorId) -> String {
        let origin = self.actor(viewer).unwrap().cell;
        let mut lines = Vec::new();
        for cell in self.visible_cell_ids(viewer) {
            let holds = self.pile_count(cell, Material::Mint) > 0
                || !self.ground_ids(cell, Material::Mint).is_empty();
            if !holds && !self.mint_bare(cell) {
                continue;
            }
            if holds {
                lines.push(format!(
                    "Wild mint grows on the wet bank {}. It can be steeped in hot water.",
                    self.place_phrase(origin, cell)
                ));
            } else if cell == origin {
                lines.push("The wet bank here has no mint.".into());
            } else {
                let way = self.relative(origin, cell);
                lines.push(format!("The wet bank to the {way} has no mint."));
            }
        }
        for actor in self.actors.values() {
            if chebyshev(origin, actor.cell) > self.tuning.sight {
                continue;
            }
            let count = self.loose_ids(actor.id, Material::Mint).len() as u32;
            if count == 0 {
                continue;
            }
            let who = if actor.id == viewer {
                "you"
            } else {
                actor.name.as_str()
            };
            let cover = self.cover_phrase(actor.cell);
            if count == 1 {
                lines.push(format!(
                    "A handful of mint is carried by {who} {cover}. It can be steeped in hot water."
                ));
            } else {
                lines.push(format!(
                    "{} handfuls of mint are carried by {who} {cover}. They can be steeped in hot water.",
                    capitalize(&count_word(count))
                ));
            }
        }
        if lines.is_empty() {
            "You see no mint from here.".into()
        } else {
            lines.join(" ")
        }
    }

    fn examine_moss(&self, viewer: ActorId) -> String {
        let origin = self.actor(viewer).unwrap().cell;
        let mut lines = Vec::new();
        for cell in self.visible_cell_ids(viewer) {
            let holds = self.pile_count(cell, Material::Moss) > 0
                || !self.ground_ids(cell, Material::Moss).is_empty();
            if !holds && !self.moss_bare(cell) && !self.shade_bare(cell) {
                continue;
            }
            if holds && self.shade_moss.contains(&cell) {
                lines.push(format!(
                    "Dry moss grows in the shade {}. It can light a fire.",
                    self.place_phrase(origin, cell)
                ));
            } else if holds {
                lines.push(format!(
                    "Moss grows on the damp bank {}. It can be pressed on a sting.",
                    self.place_phrase(origin, cell)
                ));
            } else if self.shade_bare(cell) && cell == origin {
                lines.push("The shade here has no moss.".into());
            } else if self.shade_bare(cell) {
                let way = self.relative(origin, cell);
                lines.push(format!("The shade to the {way} has no moss."));
            } else if cell == origin {
                lines.push("The damp bank here is bare.".into());
            } else {
                let way = self.relative(origin, cell);
                lines.push(format!("The damp bank to the {way} is bare."));
            }
        }
        for actor in self.actors.values() {
            if chebyshev(origin, actor.cell) > self.tuning.sight {
                continue;
            }
            let count = self.loose_ids(actor.id, Material::Moss).len() as u32;
            if count == 0 {
                continue;
            }
            let who = if actor.id == viewer {
                "you"
            } else {
                actor.name.as_str()
            };
            let cover = self.cover_phrase(actor.cell);
            if count == 1 {
                lines.push(format!(
                    "A handful of moss is carried by {who} {cover}. It can be pressed on a sting."
                ));
            } else {
                lines.push(format!(
                    "{} handfuls of moss are carried by {who} {cover}. They can be pressed on a sting.",
                    capitalize(&count_word(count))
                ));
            }
        }
        if lines.is_empty() {
            "You see no moss from here.".into()
        } else {
            lines.join(" ")
        }
    }

    /// Pine needles under a pine this person can see but is not standing on.
    /// Pitch on that pine is a different sentence.
    fn seen_needles(&self, viewer: ActorId) -> Option<String> {
        let origin = self.actor(viewer)?.cell;
        let mut lines = Vec::new();
        for cell in self.visible_cell_ids(viewer) {
            if cell == origin {
                continue;
            }
            let way = self.relative(origin, cell);
            if self.pile_count(cell, Material::Needle) > 0
                || !self.ground_ids(cell, Material::Needle).is_empty()
            {
                lines.push(format!("Pine needles lie under the pine to the {way}."));
            } else if self.needle_bare(cell) {
                lines.push(format!("The pine to the {way} has no needles."));
            }
        }
        if lines.is_empty() {
            None
        } else {
            Some(lines.join(" "))
        }
    }

    fn examine_needles(&self, viewer: ActorId) -> String {
        let origin = self.actor(viewer).unwrap().cell;
        let mut lines = Vec::new();
        for cell in self.visible_cell_ids(viewer) {
            let holds = self.pile_count(cell, Material::Needle) > 0
                || !self.ground_ids(cell, Material::Needle).is_empty();
            if !holds && !self.needle_bare(cell) {
                continue;
            }
            if holds {
                lines.push(format!(
                    "Pine needles lie under the pine {}. They can be steeped in hot water.",
                    self.place_phrase(origin, cell)
                ));
            } else if cell == origin {
                lines.push("The pine here has no needles.".into());
            } else {
                let way = self.relative(origin, cell);
                lines.push(format!("The pine to the {way} has no needles."));
            }
        }
        for actor in self.actors.values() {
            if chebyshev(origin, actor.cell) > self.tuning.sight {
                continue;
            }
            let count = self.loose_ids(actor.id, Material::Needle).len() as u32;
            if count == 0 {
                continue;
            }
            let who = if actor.id == viewer {
                "you"
            } else {
                actor.name.as_str()
            };
            let cover = self.cover_phrase(actor.cell);
            if count == 1 {
                lines.push(format!(
                    "A handful of pine needles is carried by {who} {cover}. They can be steeped in hot water."
                ));
            } else {
                lines.push(format!(
                    "{} handfuls of pine needles are carried by {who} {cover}. They can be steeped in hot water.",
                    capitalize(&count_word(count))
                ));
            }
        }
        if lines.is_empty() {
            "You see no pine needles from here.".into()
        } else {
            lines.join(" ")
        }
    }

    /// Pitch under a pine this person can see but is not standing on.
    fn seen_resin(&self, viewer: ActorId) -> Option<String> {
        let origin = self.actor(viewer)?.cell;
        let mut lines = Vec::new();
        for cell in self.visible_cell_ids(viewer) {
            if cell == origin {
                continue;
            }
            let way = self.relative(origin, cell);
            let count = self.pile_count(cell, Material::Resin)
                + self.ground_ids(cell, Material::Resin).len() as u32;
            if count > 1 {
                lines.push(format!(
                    "Lumps of pitch lie to the {way}. It is pine resin."
                ));
            } else if count == 1 {
                lines.push(format!(
                    "A lump of pitch lies to the {way}. It is pine resin."
                ));
            } else if self.resin_bare(cell) {
                lines.push(format!("The pine to the {way} has no resin."));
            }
        }
        if lines.is_empty() {
            None
        } else {
            Some(lines.join(" "))
        }
    }

    fn examine_resin(&self, viewer: ActorId) -> String {
        let origin = self.actor(viewer).unwrap().cell;
        let mut lines = Vec::new();
        for cell in self.visible_cell_ids(viewer) {
            let count = self.pile_count(cell, Material::Resin)
                + self.ground_ids(cell, Material::Resin).len() as u32;
            if count == 0 && !self.resin_bare(cell) {
                continue;
            }
            let place = self.place_phrase(origin, cell);
            if count > 1 {
                lines.push(format!(
                    "Lumps of pine resin lie {place}. They are pitch from a pine."
                ));
            } else if count == 1 {
                lines.push(format!(
                    "A lump of pine resin lies {place}. It is pitch from a pine."
                ));
            } else if cell == origin {
                lines.push("The pine here has no resin.".into());
            } else {
                let way = self.relative(origin, cell);
                lines.push(format!("The pine to the {way} has no resin."));
            }
        }
        for actor in self.actors.values() {
            if chebyshev(origin, actor.cell) > self.tuning.sight {
                continue;
            }
            let count = self.loose_ids(actor.id, Material::Resin).len() as u32;
            if count == 0 {
                continue;
            }
            let who = if actor.id == viewer {
                "you"
            } else {
                actor.name.as_str()
            };
            let cover = self.cover_phrase(actor.cell);
            if count == 1 {
                lines.push(format!(
                    "A lump of pine resin is carried by {who} {cover}. It is pitch from a pine."
                ));
            } else {
                lines.push(format!(
                    "{} lumps of pine resin are carried by {who} {cover}. They are pitch from a pine.",
                    capitalize(&count_word(count))
                ));
            }
        }
        if lines.is_empty() {
            "You see no resin from here.".into()
        } else {
            lines.join(" ")
        }
    }

    /// Cordage on a willow this person can see but is not standing on.
    fn seen_willow(&self, viewer: ActorId) -> Option<String> {
        let origin = self.actor(viewer)?.cell;
        let mut lines = Vec::new();
        for cell in self.visible_cell_ids(viewer) {
            if cell == origin {
                continue;
            }
            let way = self.relative(origin, cell);
            let count = self.pile_count(cell, Material::Willow)
                + self.ground_ids(cell, Material::Willow).len() as u32;
            if count > 1 {
                lines.push(format!(
                    "Willow cordage hangs to the {way}. It is withies from a willow."
                ));
            } else if count == 1 {
                lines.push(format!(
                    "A bundle of willow cordage hangs to the {way}. It is withies from a willow."
                ));
            } else if self.willow_bare(cell) {
                lines.push(format!("The willow to the {way} has no withies."));
            }
            let bark = self.pile_count(cell, Material::WillowBark)
                + self.ground_ids(cell, Material::WillowBark).len() as u32;
            if bark > 1 {
                lines.push(format!(
                    "Strips of willow bark hang on the willow to the {way}. They are not the cordage."
                ));
            } else if bark == 1 {
                lines.push(format!(
                    "A strip of willow bark hangs on the willow to the {way}. It is not the cordage."
                ));
            } else if self.willow_bark_bare(cell) {
                lines.push(format!("The willow to the {way} has no bark."));
            }
        }
        if lines.is_empty() {
            None
        } else {
            Some(lines.join(" "))
        }
    }

    fn examine_willow(&self, viewer: ActorId) -> String {
        let origin = self.actor(viewer).unwrap().cell;
        let mut lines = Vec::new();
        for cell in self.visible_cell_ids(viewer) {
            let count = self.pile_count(cell, Material::Willow)
                + self.ground_ids(cell, Material::Willow).len() as u32;
            if count == 0 && !self.willow_bare(cell) {
                continue;
            }
            let place = self.place_phrase(origin, cell);
            if count > 1 {
                lines.push(format!(
                    "Bundles of willow cordage hang {place}. They are withies from a willow."
                ));
            } else if count == 1 {
                lines.push(format!(
                    "A bundle of willow cordage hangs {place}. It is withies from a willow."
                ));
            } else if cell == origin {
                lines.push("The willow here has no withies.".into());
            } else {
                let way = self.relative(origin, cell);
                lines.push(format!("The willow to the {way} has no withies."));
            }
        }
        for actor in self.actors.values() {
            if chebyshev(origin, actor.cell) > self.tuning.sight {
                continue;
            }
            let count = self.loose_ids(actor.id, Material::Willow).len() as u32;
            if count == 0 {
                continue;
            }
            let who = if actor.id == viewer {
                "you"
            } else {
                actor.name.as_str()
            };
            let cover = self.cover_phrase(actor.cell);
            if count == 1 {
                lines.push(format!(
                    "A bundle of willow cordage is carried by {who} {cover}. It is withies from a willow."
                ));
            } else {
                lines.push(format!(
                    "{} bundles of willow cordage are carried by {who} {cover}. They are withies from a willow.",
                    capitalize(&count_word(count))
                ));
            }
        }
        if lines.is_empty() {
            "You see no willow from here.".into()
        } else {
            lines.join(" ")
        }
    }

    /// Willow bark on the same stand. The words are not the cordage.
    fn examine_willow_bark(&self, viewer: ActorId) -> String {
        let origin = self.actor(viewer).unwrap().cell;
        let mut lines = Vec::new();
        for cell in self.visible_cell_ids(viewer) {
            let count = self.pile_count(cell, Material::WillowBark)
                + self.ground_ids(cell, Material::WillowBark).len() as u32;
            if count == 0 && !self.willow_bark_bare(cell) {
                continue;
            }
            let place = self.place_phrase(origin, cell);
            if count > 1 {
                lines.push(format!(
                    "Strips of willow bark hang on the willow {place}. They are not the cordage."
                ));
            } else if count == 1 {
                lines.push(format!(
                    "A strip of willow bark hangs on the willow {place}. It is not the cordage."
                ));
            } else if cell == origin {
                lines.push("The willow here has no bark.".into());
            } else {
                let way = self.relative(origin, cell);
                lines.push(format!("The willow to the {way} has no bark."));
            }
        }
        for actor in self.actors.values() {
            if chebyshev(origin, actor.cell) > self.tuning.sight {
                continue;
            }
            let count = self.loose_ids(actor.id, Material::WillowBark).len() as u32;
            if count == 0 {
                continue;
            }
            let who = if actor.id == viewer {
                "you"
            } else {
                actor.name.as_str()
            };
            let cover = self.cover_phrase(actor.cell);
            if count == 1 {
                lines.push(format!(
                    "A strip of willow bark is carried by {who} {cover}. It is not cordage. It can be steeped in hot water."
                ));
            } else {
                lines.push(format!(
                    "{} strips of willow bark are carried by {who} {cover}. They are not cordage. They can be steeped in hot water.",
                    capitalize(&count_word(count))
                ));
            }
        }
        if lines.is_empty() {
            "You see no willow bark from here.".into()
        } else {
            lines.join(" ")
        }
    }

    /// Nettles this person can see but is not standing on.
    fn seen_nettle(&self, viewer: ActorId) -> Option<String> {
        let origin = self.actor(viewer)?.cell;
        let mut lines = Vec::new();
        for cell in self.visible_cell_ids(viewer) {
            if cell == origin {
                continue;
            }
            let way = self.relative(origin, cell);
            let count = self.pile_count(cell, Material::Nettle)
                + self.ground_ids(cell, Material::Nettle).len() as u32;
            if count > 1 {
                lines.push(format!("Nettles grow to the {way}."));
            } else if count == 1 {
                lines.push(format!("A stand of nettles grows to the {way}."));
            } else if self.nettle_bare(cell) {
                lines.push(format!("The nettles to the {way} are bare."));
            }
        }
        if lines.is_empty() {
            None
        } else {
            Some(lines.join(" "))
        }
    }

    fn seen_cattail(&self, viewer: ActorId) -> Option<String> {
        let origin = self.actor(viewer)?.cell;
        let mut lines = Vec::new();
        for cell in self.visible_cell_ids(viewer) {
            if cell == origin {
                continue;
            }
            let way = self.relative(origin, cell);
            let count = self.pile_count(cell, Material::Cattail);
            if count > 0 && self.still_frozen(cell) {
                lines.push(format!(
                    "Cattails stand under the ice to the {way}. They are not the reeds."
                ));
            } else if count > 0 {
                lines.push(format!(
                    "Cattails grow in the shallows to the {way}. They are not the reeds."
                ));
                if self.pile_count(cell, Material::Fluff) > 0
                    || !self.ground_ids(cell, Material::Fluff).is_empty()
                {
                    lines.push(format!(
                        "Cattail fluff hangs on the cattails to the {way}. It is not the roots."
                    ));
                } else {
                    lines.push(format!("The cattails to the {way} have no fluff."));
                }
            } else if self.cattail_bare(cell) {
                lines.push(format!("The shallows to the {way} have no cattails."));
            }
        }
        if lines.is_empty() {
            None
        } else {
            Some(lines.join(" "))
        }
    }

    fn examine_cattail(&self, viewer: ActorId) -> String {
        let origin = self.actor(viewer).unwrap().cell;
        let mut lines = Vec::new();
        for cell in self.visible_cell_ids(viewer) {
            let count = self.pile_count(cell, Material::Cattail)
                + self.ground_ids(cell, Material::Cattail).len() as u32;
            if count == 0 && !self.cattail_bare(cell) {
                continue;
            }
            let place = self.place_phrase(origin, cell);
            if self.cattail_stands.contains(&cell) && self.pile_count(cell, Material::Cattail) > 0 {
                if self.still_frozen(cell) {
                    lines.push(format!(
                        "Cattails stand {place}, under the ice. The roots cannot be dug. They are not the reeds."
                    ));
                } else {
                    lines.push(format!(
                        "Cattails grow {place}, in the shallows. The roots can be dug. They are not the reeds."
                    ));
                }
            } else if count > 0 && self.still_frozen(cell) {
                lines.push(format!("Cattail roots lie under the ice {place}."));
            } else if count > 0 {
                lines.push(format!("Cattail roots are {place}."));
            } else if cell == origin {
                lines.push("The shallows here have no cattails.".into());
            } else {
                let way = self.relative(origin, cell);
                lines.push(format!("The shallows to the {way} have no cattails."));
            }
        }
        if lines.is_empty() {
            "There are no cattails in sight.".into()
        } else {
            lines.join(" ")
        }
    }

    fn examine_fluff(&self, viewer: ActorId) -> String {
        let actor = self.actor(viewer).unwrap();
        let mut lines = Vec::new();
        for cell in self.visible_cell_ids(viewer) {
            if self.pile_count(cell, Material::Cattail) == 0 || self.still_frozen(cell) {
                continue;
            }
            let place = self.place_phrase(actor.cell, cell);
            if self.pile_count(cell, Material::Fluff) > 0
                || !self.ground_ids(cell, Material::Fluff).is_empty()
            {
                lines.push(format!(
                    "Cattail fluff hangs on the cattails {place}. It is not the roots. It can light a fire."
                ));
            } else if cell == actor.cell {
                lines.push("The cattails here have no fluff. The roots can still be dug.".into());
            } else {
                let way = self.relative(actor.cell, cell);
                lines.push(format!(
                    "The cattails to the {way} have no fluff. The roots can still be dug."
                ));
            }
        }
        let carried = self.loose_ids(viewer, Material::Fluff);
        let word = carried
            .iter()
            .filter_map(|id| self.item(*id).map(|item| item.moisture))
            .max()
            .map(|moisture| self.condition_word(moisture))
            .unwrap_or("dry");
        if carried.len() == 1 {
            lines.push(format!(
                "A {word} handful of cattail fluff is carried by you. It is not the roots. It can light a fire."
            ));
        } else if carried.len() > 1 {
            lines.push(format!(
                "{} {word} handfuls of cattail fluff are carried by you. They are not the roots.",
                count_word(carried.len() as u32)
            ));
        }
        if lines.is_empty() {
            "You see no cattail fluff from here.".into()
        } else {
            lines.join(" ")
        }
    }

    fn examine_nettle(&self, viewer: ActorId) -> String {
        let origin = self.actor(viewer).unwrap().cell;
        let mut lines = Vec::new();
        for cell in self.visible_cell_ids(viewer) {
            let count = self.pile_count(cell, Material::Nettle)
                + self.ground_ids(cell, Material::Nettle).len() as u32;
            if count == 0 && !self.nettle_bare(cell) {
                continue;
            }
            let place = self.place_phrase(origin, cell);
            if count > 1 {
                lines.push(format!(
                    "Nettles grow {place}. They can be stripped for cordage."
                ));
            } else if count == 1 {
                lines.push(format!(
                    "A stand of nettles grows {place}. It can be stripped for cordage."
                ));
            } else if cell == origin {
                lines.push("The nettles here are bare.".into());
            } else {
                let way = self.relative(origin, cell);
                lines.push(format!("The nettles to the {way} are bare."));
            }
        }
        for actor in self.actors.values() {
            if chebyshev(origin, actor.cell) > self.tuning.sight {
                continue;
            }
            let count = self.loose_ids(actor.id, Material::Nettle).len() as u32;
            if count == 0 {
                continue;
            }
            let who = if actor.id == viewer {
                "you"
            } else {
                actor.name.as_str()
            };
            let cover = self.cover_phrase(actor.cell);
            if count == 1 {
                lines.push(format!(
                    "A bundle of nettle cordage is carried by {who} {cover}. It is stripped from nettles."
                ));
            } else {
                lines.push(format!(
                    "{} bundles of nettle cordage are carried by {who} {cover}. They are stripped from nettles.",
                    capitalize(&count_word(count))
                ));
            }
        }
        if lines.is_empty() {
            "You see no nettles from here.".into()
        } else {
            lines.join(" ")
        }
    }

    fn bow_counts(&self, ids: &[crate::model::ItemId]) -> (u32, u32, u32) {
        let willow = ids
            .iter()
            .filter(|id| self.item(**id).is_some_and(|bow| bow.bound_willow))
            .count() as u32;
        let nettle = ids
            .iter()
            .filter(|id| self.item(**id).is_some_and(|bow| bow.bound_nettle))
            .count() as u32;
        (ids.len() as u32 - willow - nettle, willow, nettle)
    }

    fn bow_kind_lines(
        &self,
        hide: u32,
        willow: u32,
        nettle: u32,
        lying: bool,
        where_clause: &str,
    ) -> Vec<String> {
        let mut lines = Vec::new();
        for (count, binding) in [
            (hide, "hide strips"),
            (willow, "willow cordage"),
            (nettle, "nettle cordage"),
        ] {
            if count == 0 {
                continue;
            }
            let made = format!("a dry branch bound with {binding}");
            if lying {
                if count == 1 {
                    lines.push(format!("A bow lies {where_clause}. It is {made}."));
                } else {
                    lines.push(format!(
                        "{} bows lie {where_clause}. Each is {made}.",
                        capitalize(&count_word(count))
                    ));
                }
            } else if count == 1 {
                lines.push(format!("A bow is carried by {where_clause}. It is {made}."));
            } else {
                lines.push(format!(
                    "{} bows are carried by {where_clause}. Each is {made}.",
                    capitalize(&count_word(count))
                ));
            }
        }
        lines
    }

    fn examine_bow(&self, viewer: ActorId) -> String {
        let origin = self.actor(viewer).unwrap().cell;
        let mut lines = Vec::new();
        for cell in self.visible_cell_ids(viewer) {
            let ids = self.ground_ids(cell, Material::Bow);
            let (hide, willow, nettle) = self.bow_counts(&ids);
            if hide + willow + nettle == 0 {
                continue;
            }
            let place = if origin == cell {
                "here".to_string()
            } else {
                format!("to the {}", self.relative(origin, cell))
            };
            lines.extend(self.bow_kind_lines(hide, willow, nettle, true, &place));
        }
        for actor in self.actors.values() {
            if chebyshev(origin, actor.cell) > self.tuning.sight {
                continue;
            }
            let ids = self.loose_ids(actor.id, Material::Bow);
            let (hide, willow, nettle) = self.bow_counts(&ids);
            if hide + willow + nettle == 0 {
                continue;
            }
            let who = if actor.id == viewer {
                "you"
            } else {
                actor.name.as_str()
            };
            let cover = self.cover_phrase(actor.cell);
            lines.extend(self.bow_kind_lines(
                hide,
                willow,
                nettle,
                false,
                &format!("{who} {cover}"),
            ));
        }
        if lines.is_empty() {
            "You see no bow from here.".into()
        } else {
            lines.join(" ")
        }
    }

    fn examine_drill(&self, viewer: ActorId) -> String {
        let origin = self.actor(viewer).unwrap().cell;
        let mut lines = Vec::new();
        for cell in self.visible_cell_ids(viewer) {
            if self.ground_ids(cell, Material::Drill).is_empty() {
                continue;
            }
            let place = if origin == cell {
                "here".to_string()
            } else {
                format!("to the {}", self.relative(origin, cell))
            };
            let ids = self.ground_ids(cell, Material::Drill);
            let spent = ids
                .iter()
                .filter(|id| self.item(**id).is_some_and(|drill| drill.spent))
                .count();
            if ids.len() > spent {
                lines.push(format!(
                    "A bow drill lies {place}. It can light a fire without tinder."
                ));
            }
            if spent > 0 {
                lines.push(format!("A spent bow drill lies {place}."));
            }
        }
        for actor in self.actors.values() {
            if chebyshev(origin, actor.cell) > self.tuning.sight {
                continue;
            }
            let ids = self.loose_ids(actor.id, Material::Drill);
            if ids.is_empty() {
                continue;
            }
            let spent = ids
                .iter()
                .filter(|id| self.item(**id).is_some_and(|drill| drill.spent))
                .count();
            let fresh = ids.len() - spent;
            if actor.id == viewer {
                if fresh > 0 {
                    lines.push(
                        "A bow drill is in your arms. It can light a fire without tinder.".into(),
                    );
                }
                if spent > 0 {
                    lines.push("A spent bow drill is in your arms.".into());
                }
            } else {
                if fresh > 0 {
                    lines.push(format!(
                        "A bow drill is in {}'s arms. It can light a fire without tinder.",
                        actor.name
                    ));
                }
                if spent > 0 {
                    lines.push(format!("A spent bow drill is in {}'s arms.", actor.name));
                }
            }
        }
        if lines.is_empty() {
            "You see no bow drill from here.".into()
        } else {
            lines.join(" ")
        }
    }

    fn examine_paddle(&self, viewer: ActorId) -> String {
        let origin = self.actor(viewer).unwrap().cell;
        let mut lines = Vec::new();
        for cell in self.visible_cell_ids(viewer) {
            let count = self.ground_ids(cell, Material::Paddle).len() as u32;
            if count == 0 {
                continue;
            }
            let place = if origin == cell {
                "here".to_string()
            } else {
                format!("to the {}", self.relative(origin, cell))
            };
            if count == 1 {
                lines.push(format!(
                    "A wooden paddle lies {place}. It is carved from a dry branch."
                ));
            } else {
                lines.push(format!(
                    "{} wooden paddles lie {place}. Each is carved from a dry branch.",
                    capitalize(&count_word(count))
                ));
            }
        }
        for actor in self.actors.values() {
            if chebyshev(origin, actor.cell) > self.tuning.sight {
                continue;
            }
            let count = self.loose_ids(actor.id, Material::Paddle).len() as u32;
            if count == 0 {
                continue;
            }
            let who = if actor.id == viewer {
                "you"
            } else {
                actor.name.as_str()
            };
            let cover = self.cover_phrase(actor.cell);
            if count == 1 {
                lines.push(format!(
                    "A wooden paddle is carried by {who} {cover}. It is carved from a dry branch."
                ));
            } else {
                lines.push(format!(
                    "{} wooden paddles are carried by {who} {cover}. Each is carved from a dry branch.",
                    capitalize(&count_word(count))
                ));
            }
        }
        if lines.is_empty() {
            "You see no wooden paddle from here.".into()
        } else {
            lines.join(" ")
        }
    }

    fn examine_hook(&self, viewer: ActorId) -> String {
        let origin = self.actor(viewer).unwrap().cell;
        let mut lines = Vec::new();
        for cell in self.visible_cell_ids(viewer) {
            let count = self.ground_ids(cell, Material::Hook).len() as u32;
            if count == 0 {
                continue;
            }
            let place = if origin == cell {
                "here".to_string()
            } else {
                format!("to the {}", self.relative(origin, cell))
            };
            if count == 1 {
                lines.push(format!(
                    "A bone fish hook lies {place}. It is carved from a bone."
                ));
            } else {
                lines.push(format!(
                    "{} bone fish hooks lie {place}. Each is carved from a bone.",
                    capitalize(&count_word(count))
                ));
            }
        }
        for actor in self.actors.values() {
            if chebyshev(origin, actor.cell) > self.tuning.sight {
                continue;
            }
            let count = self.loose_ids(actor.id, Material::Hook).len() as u32;
            if count == 0 {
                continue;
            }
            let who = if actor.id == viewer {
                "you"
            } else {
                actor.name.as_str()
            };
            let cover = self.cover_phrase(actor.cell);
            if count == 1 {
                lines.push(format!(
                    "A bone fish hook is carried by {who} {cover}. It is carved from a bone."
                ));
            } else {
                lines.push(format!(
                    "{} bone fish hooks are carried by {who} {cover}. Each is carved from a bone.",
                    capitalize(&count_word(count))
                ));
            }
        }
        if lines.is_empty() {
            "You see no bone fish hook from here.".into()
        } else {
            lines.join(" ")
        }
    }

    /// A branch in the pile, or one lying loose. The rock's shelter does not count.
    fn branches_lying(&self, cell: crate::model::CellId) -> bool {
        self.pile_count(cell, Material::Branch) > 0
            || !self.ground_ids(cell, Material::Branch).is_empty()
    }

    /// The deer is standing on this rock and no branch is under it.
    /// Look names that ground and does not also say the pile is gone.
    fn deer_on_empty_rock_ground(&self, cell: crate::model::CellId) -> bool {
        self.cell(cell).rock
            && !self.branches_lying(cell)
            && self.deer.as_ref().is_some_and(|deer| deer.cell == cell)
    }

    /// The rock's dry branches were taken and have not lain there again.
    fn branch_bare(&self, cell: crate::model::CellId) -> bool {
        self.branch_returns_at.contains_key(&cell)
            && self.pile_count(cell, Material::Branch) == 0
            && self.ground_ids(cell, Material::Branch).is_empty()
    }

    /// The rock's branches, for someone who can see that ground and is not standing on it.
    /// Past sight, nothing. At night, with no fire nearby, this line is not read.
    fn seen_rock_branches(&self, viewer: ActorId) -> Option<String> {
        let origin = self.actor(viewer)?.cell;
        let mut lines = Vec::new();
        for cell in self.visible_cell_ids(viewer) {
            if cell == origin || !self.cell(cell).rock {
                continue;
            }
            if self.branch_bare(cell) {
                lines.push(format!(
                    "There are no branches under the rock to the {}.",
                    self.relative(origin, cell)
                ));
            }
        }
        if lines.is_empty() {
            None
        } else {
            Some(lines.join(" "))
        }
    }

    /// The rock's bark was taken and has not lain there again.
    fn bark_bare(&self, cell: crate::model::CellId) -> bool {
        self.bark_returns_at.contains_key(&cell)
            && self.pile_count(cell, Material::Bark) == 0
            && self.ground_ids(cell, Material::Bark).is_empty()
    }

    /// The rock's bark, for someone who can see that ground and is not standing on it.
    /// Past sight, nothing. At night, with no fire nearby, this line is not read.
    fn seen_rock_bark(&self, viewer: ActorId) -> Option<String> {
        let origin = self.actor(viewer)?.cell;
        let mut lines = Vec::new();
        for cell in self.visible_cell_ids(viewer) {
            if cell == origin || !self.cell(cell).rock {
                continue;
            }
            if self.bark_bare(cell) {
                lines.push(format!(
                    "There is no bark under the rock to the {}.",
                    self.relative(origin, cell)
                ));
            }
        }
        if lines.is_empty() {
            None
        } else {
            Some(lines.join(" "))
        }
    }

    /// The brook edge's reeds were gathered and have not grown there again.
    fn reed_bare(&self, cell: crate::model::CellId) -> bool {
        self.reed_returns_at.contains_key(&cell) && !self.reed_bed_holding(cell)
    }

    /// The slope's iron ore was gathered and a lump has not lain there again.
    fn iron_bare(&self, cell: crate::model::CellId) -> bool {
        self.iron_returns_at.contains_key(&cell) && !self.iron_bed_holding(cell)
    }

    /// The brook's copper ore was gathered and a lump has not lain there again.
    fn copper_bare(&self, cell: crate::model::CellId) -> bool {
        self.copper_returns_at.contains_key(&cell) && !self.copper_bed_holding(cell)
    }

    /// The brook bank's clay was gathered and has not lain there again.
    fn clay_bare(&self, cell: crate::model::CellId) -> bool {
        self.clay_returns_at.contains_key(&cell) && !self.clay_bed_holding(cell)
    }

    /// The slope's ochre was gathered and a lump has not lain there again.
    fn ochre_bare(&self, cell: crate::model::CellId) -> bool {
        self.ochre_returns_at.contains_key(&cell)
            && self.pile_count(cell, Material::Ochre) == 0
            && self.ground_ids(cell, Material::Ochre).is_empty()
    }

    /// The slope's flint was gathered and a nodule has not lain there again.
    fn flint_bare(&self, cell: crate::model::CellId) -> bool {
        self.flint_returns_at.contains_key(&cell)
            && self.pile_count(cell, Material::Flint) == 0
            && self.ground_ids(cell, Material::Flint).is_empty()
    }

    /// The damp bank was gathered and moss has not grown there again.
    /// The wet bank was gathered bare and mint has not grown there again.
    fn mint_bare(&self, cell: crate::model::CellId) -> bool {
        self.mint_returns_at.contains_key(&cell)
            && self.pile_count(cell, Material::Mint) == 0
            && self.ground_ids(cell, Material::Mint).is_empty()
    }

    fn moss_bare(&self, cell: crate::model::CellId) -> bool {
        self.moss_returns_at.contains_key(&cell)
            && self.pile_count(cell, Material::Moss) == 0
            && self.ground_ids(cell, Material::Moss).is_empty()
    }

    /// The shade under the rock held moss, and that handful is gone.
    fn shade_bare(&self, cell: crate::model::CellId) -> bool {
        self.shade_moss.contains(&cell)
            && self.pile_count(cell, Material::Moss) == 0
            && self.ground_ids(cell, Material::Moss).is_empty()
    }

    /// The pine was gathered bare of needles, and they have not lain there again.
    /// Pitch uses a different wait.
    fn needle_bare(&self, cell: crate::model::CellId) -> bool {
        self.needle_returns_at.contains_key(&cell)
            && self.pile_count(cell, Material::Needle) == 0
            && self.ground_ids(cell, Material::Needle).is_empty()
    }

    /// The pine was gathered bare and pitch has not lain there again.
    fn resin_bare(&self, cell: crate::model::CellId) -> bool {
        self.resin_returns_at.contains_key(&cell)
            && self.pile_count(cell, Material::Resin) == 0
            && self.ground_ids(cell, Material::Resin).is_empty()
    }

    /// The strip of willow bark was taken. It does not grow back.
    fn willow_bark_bare(&self, cell: crate::model::CellId) -> bool {
        self.willow_bark_stands.contains(&cell)
            && self.pile_count(cell, Material::WillowBark) == 0
            && self.ground_ids(cell, Material::WillowBark).is_empty()
    }

    /// The willow was stripped bare and cordage has not hung there again.
    fn willow_bare(&self, cell: crate::model::CellId) -> bool {
        self.willow_returns_at.contains_key(&cell)
            && self.pile_count(cell, Material::Willow) == 0
            && self.ground_ids(cell, Material::Willow).is_empty()
    }

    /// The nettles were stripped bare and have not grown there again.
    fn nettle_bare(&self, cell: crate::model::CellId) -> bool {
        self.nettle_returns_at.contains_key(&cell)
            && self.pile_count(cell, Material::Nettle) == 0
            && self.ground_ids(cell, Material::Nettle).is_empty()
    }

    /// The scavenged bone under the rock was taken and has not lain there again.
    fn bone_cache_bare(&self, cell: crate::model::CellId) -> bool {
        self.bone_returns_at.contains_key(&cell)
            && self.pile_count(cell, Material::Bone) == 0
            && self.ground_ids(cell, Material::Bone).is_empty()
    }

    /// The salt lick on this cell was gathered and has not formed again.
    fn salt_lick_bare(&self, cell: crate::model::CellId) -> bool {
        self.salt_returns_at.contains_key(&cell)
            && self.pile_count(cell, Material::Salt) == 0
            && self.ground_ids(cell, Material::Salt).is_empty()
    }

    /// A bone, or a bone awl, on a cell this person can see but is not standing on.
    fn seen_bone(&self, viewer: ActorId) -> Option<String> {
        let origin = self.actor(viewer)?.cell;
        let mut lines = Vec::new();
        for cell in self.visible_cell_ids(viewer) {
            if cell == origin {
                continue;
            }
            let place = if self.cell(cell).rock {
                format!("under the rock to the {}", self.relative(origin, cell))
            } else {
                let region = self.cell(cell).region.word();
                format!("on the {region} to the {}", self.relative(origin, cell))
            };
            if !self.ground_ids(cell, Material::Bone).is_empty() {
                lines.push(format!("A bone lies {place}."));
            } else if self.bone_cache_bare(cell) {
                lines.push(format!(
                    "The rock to the {} has no bone.",
                    self.relative(origin, cell)
                ));
            }
            if self
                .ground_ids(cell, Material::Awl)
                .iter()
                .any(|id| self.item(*id).is_some_and(|awl| awl.bone))
            {
                lines.push(format!("A bone awl lies {place}. It can pierce hide."));
            }
        }
        if lines.is_empty() {
            None
        } else {
            Some(lines.join(" "))
        }
    }

    /// A bone fish hook on a cell this person can see but is not standing on.
    fn seen_hook(&self, viewer: ActorId) -> Option<String> {
        let origin = self.actor(viewer)?.cell;
        let mut lines = Vec::new();
        for cell in self.visible_cell_ids(viewer) {
            if cell == origin || self.ground_ids(cell, Material::Hook).is_empty() {
                continue;
            }
            let place = if self.cell(cell).rock {
                format!("under the rock to the {}", self.relative(origin, cell))
            } else {
                let region = self.cell(cell).region.word();
                format!("on the {region} to the {}", self.relative(origin, cell))
            };
            let count = self.ground_ids(cell, Material::Hook).len() as u32;
            if count == 1 {
                lines.push(format!("A bone fish hook lies {place}."));
            } else {
                lines.push(format!(
                    "{} bone fish hooks lie {place}.",
                    capitalize(&count_word(count))
                ));
            }
        }
        if lines.is_empty() {
            None
        } else {
            Some(lines.join(" "))
        }
    }

    fn examine_bone(&self, viewer: ActorId) -> String {
        let origin = self.actor(viewer).unwrap().cell;
        let mut lines = Vec::new();
        for cell in self.visible_cell_ids(viewer) {
            if self.ground_ids(cell, Material::Bone).is_empty() {
                continue;
            }
            lines.push(format!(
                "A bone lies {}. It can be carved into an awl.",
                self.place_phrase(origin, cell)
            ));
        }
        for actor in self.actors.values() {
            if chebyshev(origin, actor.cell) > self.tuning.sight {
                continue;
            }
            if self.loose_ids(actor.id, Material::Bone).is_empty() {
                continue;
            }
            let who = if actor.id == viewer {
                "you"
            } else {
                actor.name.as_str()
            };
            lines.push(format!(
                "A bone is carried by {who}, {}. It can be carved into an awl.",
                self.cover_phrase(actor.cell)
            ));
        }
        if lines.is_empty() {
            "You see no bone from here.".into()
        } else {
            lines.join(" ")
        }
    }

    /// A reed weir in a brook this person can see but is not standing in.
    fn seen_weir(&self, viewer: ActorId) -> Option<String> {
        let origin = self.actor(viewer)?.cell;
        let mut lines = Vec::new();
        for cell in self.visible_cell_ids(viewer) {
            if cell == origin || !self.weirs.contains_key(&cell) {
                continue;
            }
            lines.push(format!(
                "A reed weir is set in the brook to the {}.",
                self.relative(origin, cell)
            ));
        }
        if lines.is_empty() {
            None
        } else {
            Some(lines.join(" "))
        }
    }

    fn examine_weir(&self, viewer: ActorId) -> String {
        let origin = self.actor(viewer).unwrap().cell;
        let mut lines = Vec::new();
        for cell in self.visible_cell_ids(viewer) {
            if !self.weirs.contains_key(&cell) {
                continue;
            }
            let place = if origin == cell {
                "here, in the brook".to_string()
            } else {
                format!("in the brook to the {}", self.relative(origin, cell))
            };
            lines.push(format!(
                "A reed weir is set {place}. It is reeds bound with cordage."
            ));
        }
        if lines.is_empty() {
            "You see no weir from here.".into()
        } else {
            lines.join(" ")
        }
    }

    /// A hide snare in sight, other than the one under the looker's feet.
    fn seen_snare(&self, viewer: ActorId) -> Option<String> {
        let origin = self.actor(viewer)?.cell;
        let mut lines = Vec::new();
        for cell in self.visible_cell_ids(viewer) {
            if cell == origin || !self.snares.contains_key(&cell) {
                continue;
            }
            let place = if self.cell(cell).rock {
                format!("under the rock to the {}", self.relative(origin, cell))
            } else {
                let region = self.cell(cell).region.word();
                format!("on the {region} to the {}", self.relative(origin, cell))
            };
            let kind = if self.snare_nettle.contains(&cell) {
                "nettle"
            } else if self.snare_willow.contains(&cell) {
                "willow"
            } else {
                "hide"
            };
            lines.push(format!("A {kind} snare is set {place}."));
        }
        if lines.is_empty() {
            None
        } else {
            Some(lines.join(" "))
        }
    }

    fn examine_snare(&self, viewer: ActorId) -> String {
        let origin = self.actor(viewer).unwrap().cell;
        let mut lines = Vec::new();
        for cell in self.visible_cell_ids(viewer) {
            if !self.snares.contains_key(&cell) {
                continue;
            }
            let place = self.place_phrase(origin, cell);
            if self.snare_nettle.contains(&cell) {
                lines.push(format!(
                    "A nettle snare is set {place}. It is made of nettle cordage."
                ));
            } else if self.snare_willow.contains(&cell) {
                lines.push(format!(
                    "A willow snare is set {place}. It is made of willow cordage."
                ));
            } else {
                lines.push(format!(
                    "A hide snare is set {place}. It is made of hide strips."
                ));
            }
        }
        if lines.is_empty() {
            "You see no snare from here.".into()
        } else {
            lines.join(" ")
        }
    }

    /// A black charcoal mark in sight, other than the one under the looker's feet.
    fn seen_charcoal_mark(&self, viewer: ActorId) -> Option<String> {
        let origin = self.actor(viewer)?.cell;
        let mut lines = Vec::new();
        for cell in self.visible_cell_ids(viewer) {
            if cell == origin || !self.charcoal_marks.contains(&cell) {
                continue;
            }
            let place = if self.cell(cell).rock {
                format!("on the rock to the {}", self.relative(origin, cell))
            } else {
                let region = self.cell(cell).region.word();
                format!(
                    "on the ground on the {region} to the {}",
                    self.relative(origin, cell)
                )
            };
            lines.push(format!("A black charcoal mark is {place}."));
        }
        if lines.is_empty() {
            None
        } else {
            Some(lines.join(" "))
        }
    }

    /// A rock in sight that the compass lines do not already stand on.
    /// The pot is on that ground, or in the hands of the person standing there.
    fn seen_under_the_rock(&self, viewer: ActorId) -> Option<String> {
        let origin = self.actor(viewer)?.cell;
        let mut lines = Vec::new();
        for cell in self.visible_cell_ids(viewer) {
            if !self.cell(cell).rock || cell == origin || orthogonal_neighbor(origin, cell) {
                continue;
            }
            let person_already = self.rock_person_already_in_the_compass(origin, cell);
            let mut names: Vec<String> = self
                .actors
                .values()
                .filter(|actor| actor.cell == cell && actor.id != viewer)
                .map(|actor| actor.name.clone())
                .collect();
            names.sort();
            let pot_on_the_ground = self.pile_count(cell, Material::Pot) > 0
                || !self.ground_ids(cell, Material::Pot).is_empty();
            let mut parts = Vec::new();
            if !names.is_empty() && !person_already {
                let ground = self.cell(cell).region.word();
                if names.len() == 1 {
                    let holder = self
                        .actors
                        .values()
                        .find(|actor| actor.cell == cell && actor.name == names[0]);
                    let carried = holder.and_then(|actor| self.carried_pot_phrase(actor.id));
                    if let Some(pot) = carried {
                        parts.push(format!(
                            "{} stands on the {ground} under the rock overhang, carrying {pot}.",
                            names[0]
                        ));
                    } else {
                        parts.push(format!(
                            "{} stands on the {ground} under the rock overhang.",
                            names[0]
                        ));
                    }
                } else {
                    let verb = "stand";
                    parts.push(format!(
                        "{} {verb} on the {ground} under the rock overhang.",
                        self.join_names(&names)
                    ));
                    for name in &names {
                        let holder = self
                            .actors
                            .values()
                            .find(|actor| actor.cell == cell && actor.name == *name);
                        if let Some(pot) =
                            holder.and_then(|actor| self.carried_pot_phrase(actor.id))
                        {
                            parts.push(format!("{name} is carrying {pot}."));
                        }
                    }
                }
            }
            if pot_on_the_ground {
                if parts.is_empty() {
                    parts.push("A pot sits on the ground under the rock overhang.".into());
                } else {
                    parts.push("A pot sits on the ground there.".into());
                }
            }
            if let Some(fire) = self.fire_at(cell) {
                let burn = if parts.is_empty() {
                    if fire.charcoal {
                        "A hot charcoal fire burns under the rock overhang."
                    } else if fire.hot {
                        "A fire burns hotter under the rock overhang."
                    } else {
                        "A small fire burns under the rock overhang."
                    }
                } else if fire.charcoal {
                    "A hot charcoal fire burns there."
                } else if fire.hot {
                    "A fire burns hotter there."
                } else {
                    "A small fire burns there."
                };
                let mut warmed: Vec<String> = self
                    .actors
                    .values()
                    .filter(|actor| actor.cell == cell && actor.id != viewer)
                    .map(|actor| actor.name.clone())
                    .collect();
                warmed.sort();
                if warmed.is_empty() {
                    parts.push(format!("{burn}"));
                } else {
                    parts.push(format!("{burn} It warms {}.", self.join_names(&warmed)));
                }
                if fire.column {
                    parts.push("A column of smoke rises from the fire.".into());
                }
                if self.banks.contains_key(&cell) {
                    parts.push("Branches are banked on it under a cover of mud.".into());
                }
            }
            let charcoal = self.ground_ids(cell, Material::Charcoal).len();
            if charcoal == 1 {
                parts.push("A lump of charcoal lies there.".into());
            } else if charcoal > 1 {
                parts.push("Lumps of charcoal lie there.".into());
            }
            if !parts.is_empty() {
                lines.push(parts.join(" "));
            }
        }
        if lines.is_empty() {
            None
        } else {
            Some(lines.join(" "))
        }
    }

    fn carried_pot_phrase(&self, actor_id: ActorId) -> Option<String> {
        let ids = self.loose_ids(actor_id, Material::Pot);
        if ids.len() == 1 {
            let moisture = self.item(ids[0]).map(|item| item.moisture).unwrap_or(0);
            Some(format!("a {} pot", self.condition_word(moisture)))
        } else if ids.len() > 1 {
            Some(format!("{} pots", count_word(ids.len() as u32)))
        } else {
            None
        }
    }

    /// The compass already names the person under the rock, one step past the outside floor.
    fn rock_person_already_in_the_compass(&self, origin: CellId, rock: CellId) -> bool {
        let (x, y) = cell_xy(origin);
        for dir in Dir::all() {
            let (dx, dy) = dir.delta();
            let Some(dest) = cell_id(x + dx, y + dy) else {
                continue;
            };
            let Some(floor) = self.outside_floor(dest) else {
                continue;
            };
            if dir.word() != floor.rock_side {
                continue;
            }
            let Some(rock_dir) = Dir::all()
                .into_iter()
                .find(|way| way.word() == floor.rock_side)
            else {
                continue;
            };
            let (rx, ry) = rock_dir.delta();
            let (fx, fy) = cell_xy(dest);
            if cell_id(fx + rx, fy + ry) == Some(rock)
                && self
                    .name_under_that_rock(origin, dest, floor.rock_side)
                    .is_some()
            {
                return true;
            }
        }
        false
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
        let stocked = self.actor(actor_id).is_some_and(|actor| {
            self.pile_count(actor.cell, Material::Branch) > 0
                || !self.ground_ids(actor.cell, Material::Branch).is_empty()
        });
        if stocked && self.open_wood_off_the_rock_is_gone() {
            line.push_str(" The branches here stay out of the weather.");
        } else if stocked {
            line.push_str(" Wood left here stays out of the weather.");
        }
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

    fn clothes_for_them(&self, moisture: u32) -> String {
        let word = self.clothes_word(moisture);
        if word == "wet" && !self.raining {
            "wet from the brook".into()
        } else {
            word.into()
        }
    }

    fn clothes_line(&self, actor_id: ActorId, moisture: u32) -> String {
        if self
            .actor(actor_id)
            .is_some_and(|actor| actor.clothes_dry_at.is_some())
        {
            return "Your clothes are drying by the fire.".into();
        }
        let word = self.clothes_word(moisture);
        let under_lean_to = self.actor(actor_id).is_some_and(|actor| {
            self.lean_tos.contains(&actor.cell)
                && self.exposure_millis(actor.cell) == self.tuning.lean_to
        });
        if under_lean_to && self.raining && word == "damp" {
            return "A little rain dampens your clothes.".into();
        }
        if word == "wet" && self.raining {
            "The rain wets your clothes.".into()
        } else if word == "wet" {
            "The brook wets your clothes.".into()
        } else if word == "dry"
            && self.raining
            && self
                .actor(actor_id)
                .is_some_and(|actor| self.rain_exposure(actor.cell) == 0)
        {
            "The rain does not wet your clothes.".into()
        } else {
            format!("Your clothes are {word}.")
        }
    }

    fn arm_bit(&self, actor_id: ActorId, item: &CarryView) -> String {
        if item.material == "stone" && item.count == 1 {
            format!("a {} stone", item.condition)
        } else if item.material == "flint" && item.count == 1 {
            format!("a {} flint nodule", item.condition)
        } else if item.material == "flake" && item.count == 1 {
            "a sharp flint flake".into()
        } else if item.material == "flake" {
            format!("{} sharp flint flakes", count_word(item.count))
        } else if item.material == "branch" && item.count == 1 {
            format!("a {} branch", item.condition)
        } else if item.material == "bark" && item.count == 1 {
            format!("a {} sheet of bark", item.condition)
        } else if item.material == "bark" {
            format!(
                "{} {} sheets of bark",
                count_word(item.count),
                item.condition
            )
        } else if item.material == "basket" && item.count == 1 {
            self.basket_arm_phrase(actor_id)
        } else if item.material == "bark cup" && item.count == 1 {
            self.bark_cup_arm_phrase(actor_id)
        } else if item.material == "pot" && item.count == 1 {
            format!("a {} pot", item.condition)
        } else if item.material == "clay" {
            self.clay_carry_phrase(actor_id)
        } else if item.material == "bowl" && item.count == 1 {
            "a fired bowl".into()
        } else if item.material == "carcass" && item.count == 1 {
            if self
                .loose_ids(actor_id, Material::Carcass)
                .iter()
                .any(|id| self.item(*id).is_some_and(|carcass| carcass.otter))
            {
                "an otter carcass".into()
            } else if self
                .loose_ids(actor_id, Material::Carcass)
                .iter()
                .any(|id| self.item(*id).is_some_and(|carcass| carcass.wolf))
            {
                "a wolf carcass".into()
            } else if self
                .loose_ids(actor_id, Material::Carcass)
                .iter()
                .any(|id| self.item(*id).is_some_and(|carcass| carcass.fox))
            {
                "a fox carcass".into()
            } else if self
                .loose_ids(actor_id, Material::Carcass)
                .iter()
                .any(|id| self.item(*id).is_some_and(|carcass| carcass.hare))
            {
                "a hare carcass".into()
            } else if self
                .loose_ids(actor_id, Material::Carcass)
                .iter()
                .any(|id| self.item(*id).is_some_and(|carcass| carcass.beaver))
            {
                "a beaver carcass".into()
            } else {
                "a deer carcass".into()
            }
        } else if item.material == "strips" && item.count == 1 {
            "hide strips".into()
        } else if item.material == "hide" && item.count == 1 {
            let hide = self
                .loose_ids(actor_id, Material::Hide)
                .into_iter()
                .find(|id| self.item(*id).is_some_and(|hide| !hide.worn_cloak()))
                .and_then(|id| self.item(id));
            let otter = hide.is_some_and(|hide| hide.otter);
            let fox = hide.is_some_and(|hide| hide.fox);
            let hare = hide.is_some_and(|hide| hide.hare);
            let beaver = hide.is_some_and(|hide| hide.beaver);
            let wolf = hide.is_some_and(|hide| hide.wolf);
            let scraped = hide.is_some_and(|hide| hide.scraped);
            let kind = if otter {
                "otter"
            } else if fox {
                "fox"
            } else if hare {
                "hare"
            } else if beaver {
                "beaver"
            } else if wolf {
                "wolf"
            } else {
                "deer"
            };
            let cloak = if otter {
                "otter-hide"
            } else if fox {
                "fox-hide"
            } else if hare {
                "hare-hide"
            } else if beaver {
                "beaver-hide"
            } else if wolf {
                "wolf-hide"
            } else {
                "deer-hide"
            };
            let article = if otter && !scraped { "an" } else { "a" };
            let raw = if scraped {
                format!("scraped {kind}")
            } else {
                kind.to_string()
            };
            match (
                hide.is_some_and(|hide| hide.worn),
                hide.is_some_and(|hide| hide.pierced),
                hide.is_some_and(|hide| hide.sewn),
            ) {
                (_, _, true) if hide.is_some_and(|hide| hide.worn) => {
                    format!("a sewn {cloak} cloak on you")
                }
                (_, _, true) => format!("a sewn {cloak} cloak"),
                (true, true, _) => format!("{article} pierced {kind} hide on you (ready to sew)"),
                (false, true, _) => format!("{article} pierced {kind} hide (ready to sew)"),
                (true, false, _) => format!("{article} {raw} hide on you"),
                (false, false, _) => format!("{article} {raw} hide"),
            }
        } else if item.material == "ochre" && item.count == 1 {
            "a lump of ochre".into()
        } else if item.material == "ochre" {
            format!("{} lumps of ochre", count_word(item.count))
        } else if item.material == "salt" && item.count == 1 {
            "a salt crust".into()
        } else if item.material == "honey" && item.count == 1 {
            "a honeycomb".into()
        } else if item.material == "honey" {
            format!("{} honeycombs", count_word(item.count))
        } else if item.material == "moss" && item.count == 1 {
            "a handful of moss".into()
        } else if item.material == "moss" {
            format!("{} handfuls of moss", count_word(item.count))
        } else if item.material == "mint" && item.count == 1 {
            "a handful of mint".into()
        } else if item.material == "mint" {
            format!("{} handfuls of mint", count_word(item.count))
        } else if item.material == "pine needles" && item.count == 1 {
            "a handful of pine needles".into()
        } else if item.material == "pine needles" {
            format!("{} handfuls of pine needles", count_word(item.count))
        } else if item.material == "resin" && item.count == 1 {
            "a lump of pine resin".into()
        } else if item.material == "resin" {
            format!("{} lumps of pine resin", count_word(item.count))
        } else if item.material == "willow bark" && item.count == 1 {
            "a strip of willow bark".into()
        } else if item.material == "willow bark" {
            format!("{} strips of willow bark", count_word(item.count))
        } else if item.material == "cattail fluff" && item.count == 1 {
            format!("a {} handful of cattail fluff", item.condition)
        } else if item.material == "cattail fluff" {
            format!(
                "{} {} handfuls of cattail fluff",
                count_word(item.count),
                item.condition
            )
        } else if item.material == "willow" && item.count == 1 {
            "a bundle of willow cordage".into()
        } else if item.material == "willow" {
            format!("{} bundles of willow cordage", count_word(item.count))
        } else if item.material == "nettle" && item.count == 1 {
            "a bundle of nettle cordage".into()
        } else if item.material == "nettle" {
            format!("{} bundles of nettle cordage", count_word(item.count))
        } else if item.material == "flour" {
            let ids = self.loose_ids(actor_id, Material::Flour);
            let cooked = ids
                .iter()
                .filter(|id| self.item(**id).is_some_and(|flour| flour.cooked))
                .count();
            if cooked > 0 && cooked == ids.len() {
                if cooked == 1 {
                    "a cattail flatbread".into()
                } else {
                    "cattail flatbreads".into()
                }
            } else {
                "cattail flour".into()
            }
        } else if item.material == "cattail" {
            let ids = self.loose_ids(actor_id, Material::Cattail);
            let cooked = ids
                .iter()
                .filter(|id| self.item(**id).is_some_and(|root| root.cooked))
                .count();
            if cooked > 0 && cooked == ids.len() {
                "cooked cattail roots".into()
            } else {
                "cattail roots".into()
            }
        } else if item.material == "net" && item.count == 1 {
            "a fishing net".into()
        } else if item.material == "net" {
            format!("{} fishing nets", count_word(item.count))
        } else if item.material == "reed mat" && item.count == 1 {
            "a reed mat".into()
        } else if item.material == "reed mat" {
            format!("{} reed mats", count_word(item.count))
        } else if item.material == "bow" && item.count == 1 {
            "a bow".into()
        } else if item.material == "bow" {
            format!("{} bows", count_word(item.count))
        } else if item.material == "bow drill" {
            let ids = self.loose_ids(actor_id, Material::Drill);
            let spent = ids
                .iter()
                .filter(|id| self.item(**id).is_some_and(|drill| drill.spent))
                .count();
            let fresh = ids.len().saturating_sub(spent);
            if fresh == 0 && spent == 1 {
                "a spent bow drill".into()
            } else if fresh == 1 && spent == 0 {
                "a bow drill".into()
            } else if fresh == 0 && spent > 1 {
                format!("{} spent bow drills", count_word(spent as u32))
            } else if spent == 0 {
                format!("{} bow drills", count_word(fresh as u32))
            } else {
                "a bow drill and a spent bow drill".into()
            }
        } else if item.material == "paddle" && item.count == 1 {
            "a wooden paddle".into()
        } else if item.material == "paddle" {
            format!("{} wooden paddles", count_word(item.count))
        } else if item.material == "egg" {
            if self
                .loose_ids(actor_id, Material::Egg)
                .iter()
                .any(|id| self.item(*id).is_some_and(|eggs| eggs.cooked))
            {
                "cooked eggs".into()
            } else {
                "eggs".into()
            }
        } else if item.material == "iron ore" && item.count == 1 {
            "a lump of iron ore".into()
        } else if item.material == "iron ore" {
            format!("{} lumps of iron ore", count_word(item.count))
        } else if item.material == "iron" && item.count == 1 {
            "a bloom of iron, metal from a hotter fire".into()
        } else if item.material == "iron knife" && item.count == 1 {
            "an iron knife".into()
        } else if item.material == "copper ore" && item.count == 1 {
            "a lump of copper ore".into()
        } else if item.material == "copper ore" {
            format!("{} lumps of copper ore", count_word(item.count))
        } else if item.material == "copper" && item.count == 1 {
            "a copper bead, metal from the fire".into()
        } else if item.material == "awl" && item.count == 1 {
            let bone = self
                .loose_ids(actor_id, Material::Awl)
                .first()
                .and_then(|id| self.item(*id))
                .is_some_and(|awl| awl.bone);
            if bone {
                "a bone awl".into()
            } else {
                "a copper awl".into()
            }
        } else if item.material == "bone" && item.count == 1 {
            "a bone".into()
        } else if item.material == "worm" && item.count == 1 {
            "a worm".into()
        } else if item.material == "worm" {
            format!("{} worms", count_word(item.count))
        } else if item.material == "quartz" && item.count == 1 {
            "a clear stone of quartz".into()
        } else if item.material == "quartz" {
            format!("{} clear stones of quartz", count_word(item.count))
        } else if item.material == "fish hook" && item.count == 1 {
            "a bone fish hook".into()
        } else if item.material == "fish hook" {
            format!("{} bone fish hooks", count_word(item.count))
        } else if item.material == "whistle" && item.count == 1 {
            "a bone whistle".into()
        } else if item.material == "whistle" {
            format!("{} bone whistles", count_word(item.count))
        } else if item.material == "flute" && item.count == 1 {
            "a reed flute".into()
        } else if item.material == "flute" {
            format!("{} reed flutes", count_word(item.count))
        } else if item.material == "drum" && item.count == 1 {
            "a drum".into()
        } else if item.material == "drum" {
            format!("{} drums", count_word(item.count))
        } else if item.material == "brand" && item.count == 1 {
            "a burning brand".into()
        } else if item.material == "fish" {
            self.fish_carry_phrase(actor_id)
        } else if item.material == "spear" {
            let bits: Vec<String> = self
                .loose_ids(actor_id, Material::Spear)
                .iter()
                .map(|id| {
                    let spear = self.item(*id);
                    if let Some(item) = spear.filter(|spear| spear.flint_tip) {
                        format!("a spear with {}", self.flint_tip_words(item))
                    } else if spear.is_some_and(|spear| spear.wooden_tip) {
                        if spear.is_some_and(|spear| spear.hardened) {
                            "a wooden spear with a fire-hardened point".into()
                        } else {
                            "a wooden spear with a green point".into()
                        }
                    } else if spear.is_some_and(|spear| spear.hardened) {
                        "a spear with a hardened point".into()
                    } else {
                        "a spear with an unhardened point".into()
                    }
                })
                .collect();
            bits.join(", ")
        } else if item.material == "mushroom" {
            let ids = self.loose_ids(actor_id, Material::Mushroom);
            let cooked = ids
                .iter()
                .filter(|id| self.item(**id).is_some_and(|mushroom| mushroom.cooked))
                .count() as u32;
            let raw = ids.len() as u32 - cooked;
            if cooked > 0 && raw == 0 {
                if cooked == 1 {
                    "a cooked mushroom".into()
                } else {
                    format!("{} cooked mushrooms", count_word(cooked))
                }
            } else if item.count == 1 {
                format!("a {} mushroom", item.condition)
            } else {
                format!("{} {} mushrooms", count_word(item.count), item.condition)
            }
        } else if item.material == "pale berry" {
            let ids = self.loose_ids(actor_id, Material::PaleBerry);
            let cooked = ids
                .iter()
                .filter(|id| self.item(**id).is_some_and(|berry| berry.cooked))
                .count() as u32;
            let raw = ids.len() as u32 - cooked;
            if cooked > 0 && raw == 0 {
                if cooked == 1 {
                    "a cooked handful of pale berries".into()
                } else {
                    format!("{} cooked handfuls of pale berries", count_word(cooked))
                }
            } else if item.count == 1 {
                format!("a {} handful of pale berries", item.condition)
            } else {
                format!(
                    "{} {} handfuls of pale berries",
                    count_word(item.count),
                    item.condition
                )
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
            let kind = self.bitter_kind(actor_id);
            if item.count == 1 {
                format!("a {} handful of {kind}", item.condition)
            } else {
                format!(
                    "{} {} handfuls of {kind}",
                    count_word(item.count),
                    item.condition
                )
            }
        } else if item.material == "reed" {
            if item.count == 1 {
                "a bundle of reeds".into()
            } else {
                format!("{} bundles of reeds", count_word(item.count))
            }
        } else if item.material == "charcoal" {
            if item.count == 1 {
                "a lump of charcoal".into()
            } else {
                format!("{} lumps of charcoal", count_word(item.count))
            }
        } else if item.material == "meat" {
            self.meat_carry_phrase(actor_id)
        } else {
            format!(
                "{} {} carried by you ({})",
                item.count, item.material, item.condition
            )
        }
    }

    fn clay_carry_phrase(&self, actor_id: ActorId) -> String {
        let ids = self.loose_ids(actor_id, Material::Clay);
        let pots = ids
            .iter()
            .filter(|id| self.item(**id).is_some_and(|clay| clay.shaped))
            .count();
        let lumps = ids.len() - pots;
        let mut bits = Vec::new();
        if lumps == 1 {
            bits.push("a lump of clay".to_string());
        } else if lumps > 1 {
            bits.push(format!("{} lumps of clay", count_word(lumps as u32)));
        }
        if pots == 1 {
            bits.push("an unfired clay pot".to_string());
        } else if pots > 1 {
            bits.push(format!("{} unfired clay pots", count_word(pots as u32)));
        }
        if bits.is_empty() {
            "a lump of clay".into()
        } else {
            bits.join(", ")
        }
    }

    fn meat_carry_phrase(&self, actor_id: ActorId) -> String {
        let ids = self.loose_ids(actor_id, Material::Meat);
        let mut bits = Vec::new();
        for face in ["raw", "going", "spoiled", "salted", "smoked", "cooked"] {
            let full = self.meat_tally(&ids, face, false);
            let small = self.meat_tally(&ids, face, true);
            if full > 0 {
                bits.push(meat_bit(face, full, false));
            }
            if small > 0 {
                bits.push(meat_bit(face, small, true));
            }
        }
        if bits.is_empty() {
            "meat".into()
        } else {
            bits.join(", ")
        }
    }

    /// Bee stings take the well word. Nettle stings name the plant.
    /// An illness still says ill.
    fn body_word(&self, actor: &crate::model::Actor) -> &'static str {
        if actor.stings_until.is_some() && self.illness_word(actor.illness) == "well" {
            if actor.nettle_sting {
                "stung by nettles"
            } else {
                "stung"
            }
        } else {
            self.illness_word(actor.illness)
        }
    }

    fn self_text(&self, actor_id: ActorId) -> String {
        let actor = self.actor(actor_id).unwrap();
        let mut parts = vec![
            self.clothes_line(actor_id, actor.wetness),
            format!("You are {}.", self.thirst_sentence(actor.thirst)),
            format!("You are {}.", self.hunger_word(actor.hunger)),
            format!("You are {}.", self.body_word(actor)),
            format!("You are {}.", self.energy_word(actor.energy)),
            format!("You are {}.", self.cold_word(actor.cold)),
            format!("You are {}.", self.wind_on(actor_id)),
            format!("Your skin is {}.", self.sun_word(actor.sunburn)),
            self.skin_sentence(actor, true),
        ];
        if let Some(held) = self.sun_held_off(actor.ochre) {
            parts.push(held.into());
        }
        if let Some(wind) = self.night_wind_line(actor_id) {
            parts.push(wind);
        }
        if let Some(owl) = self.owl_line(actor_id) {
            parts.push(owl);
        }
        if let Some(frogs) = self.frog_line(actor_id) {
            parts.push(frogs);
        }
        if let Some(birds) = self.bird_line(actor_id) {
            parts.push(birds);
        }
        if self.illness_word(actor.illness) == "ill" {
            parts.push("Your stomach has not settled.".into());
        }
        if actor.sunburn >= self.tuning.burned_at {
            parts.push("The sun burns. It hurts.".into());
        } else if actor.sunburn >= self.tuning.pink_at {
            parts.push("The sun is hot on your skin.".into());
        }
        if self.wearing_hide(actor_id) {
            let worn = |pred: fn(&crate::model::Item) -> bool| {
                self.loose_ids(actor_id, Material::Hide)
                    .iter()
                    .any(|id| self.item(*id).is_some_and(|hide| hide.worn && pred(hide)))
            };
            let sewn =
                worn(|hide| hide.sewn && !hide.otter && !hide.fox && !hide.hare && !hide.beaver);
            let pierced = worn(|hide| {
                hide.pierced && !hide.sewn && !hide.otter && !hide.fox && !hide.hare && !hide.beaver
            });
            let wolf = worn(|hide| hide.wolf && !hide.fox && !hide.hare && !hide.beaver);
            let otter_sewn = worn(|hide| hide.otter && hide.sewn);
            let otter_pierced = worn(|hide| hide.otter && hide.pierced && !hide.sewn);
            let otter_plain = worn(|hide| hide.otter && !hide.sewn && !hide.pierced);
            let fox_sewn = worn(|hide| hide.fox && hide.sewn);
            let fox_pierced = worn(|hide| hide.fox && hide.pierced && !hide.sewn);
            let fox_plain = worn(|hide| hide.fox && !hide.sewn && !hide.pierced);
            let hare_sewn = worn(|hide| hide.hare && hide.sewn);
            let hare_pierced = worn(|hide| hide.hare && hide.pierced && !hide.sewn);
            let hare_plain = worn(|hide| hide.hare && !hide.sewn && !hide.pierced);
            let beaver_sewn = worn(|hide| hide.beaver && hide.sewn);
            let beaver_pierced = worn(|hide| hide.beaver && hide.pierced && !hide.sewn);
            let beaver_plain = worn(|hide| hide.beaver && !hide.sewn && !hide.pierced);
            if otter_sewn {
                parts.push("The sewn otter-hide cloak is on you.".into());
            } else if fox_sewn {
                parts.push("The sewn fox-hide cloak is on you. It breaks the wind.".into());
            } else if hare_sewn {
                parts.push("The sewn hare-hide cloak is on you. It is a light cover.".into());
            } else if beaver_sewn {
                parts.push(
                    "The sewn beaver-hide cloak is on you. It sheds the rain and the brook.".into(),
                );
            } else if sewn {
                parts.push(if wolf {
                    "The sewn wolf-hide cloak is on you.".into()
                } else {
                    "The sewn deer-hide cloak is on you.".into()
                });
            } else if otter_pierced {
                parts.push("The pierced otter hide is on you. It is ready to sew.".into());
            } else if fox_pierced {
                parts.push("The pierced fox hide is on you. It is ready to sew.".into());
            } else if hare_pierced {
                parts.push("The pierced hare hide is on you. It is ready to sew.".into());
            } else if beaver_pierced {
                parts.push("The pierced beaver hide is on you. It is ready to sew.".into());
            } else if pierced {
                parts.push(if wolf {
                    "The pierced wolf hide is on you. It is ready to sew.".into()
                } else {
                    "The pierced deer hide is on you. It is ready to sew.".into()
                });
            } else if otter_plain {
                parts.push(if worn(|hide| hide.otter && hide.scraped) {
                    "The scraped otter hide is on you.".into()
                } else {
                    "The otter hide is on you.".into()
                });
            } else if fox_plain {
                parts.push(if worn(|hide| hide.fox && hide.scraped) {
                    "The scraped fox hide is on you.".into()
                } else {
                    "The fox hide is on you.".into()
                });
            } else if hare_plain {
                parts.push(if worn(|hide| hide.hare && hide.scraped) {
                    "The scraped hare hide is on you.".into()
                } else {
                    "The hare hide is on you.".into()
                });
            } else if beaver_plain {
                parts.push(if worn(|hide| hide.beaver && hide.scraped) {
                    "The scraped beaver hide is on you.".into()
                } else {
                    "The beaver hide is on you.".into()
                });
            } else {
                let scraped = worn(|hide| {
                    hide.scraped && !hide.otter && !hide.fox && !hide.hare && !hide.beaver
                });
                parts.push(if wolf {
                    if scraped {
                        "The scraped wolf hide is on you.".into()
                    } else {
                        "The wolf hide is on you.".into()
                    }
                } else if scraped {
                    "The scraped deer hide is on you.".into()
                } else {
                    "The deer hide is on you.".into()
                });
            }
        }
        if actor.warm_from_fire && self.fire_at(actor.cell).is_none() {
            parts.push("The warmth of the fire is still on you.".into());
        }
        parts.push(self.arms_line(actor_id));
        let flakes = self.loose_ids(actor_id, Material::Flake).len();
        if flakes == 1 {
            parts.push("It is sharp enough to cut or haft later.".into());
        } else if flakes > 1 {
            parts.push("They are sharp enough to cut or haft later.".into());
        }
        if self.flint_tip_is_hardening(actor_id) {
            parts.push("The flint tip of your spear is hardening in the fire.".into());
        } else if self.wooden_point_is_hardening(actor_id) {
            parts.push("The wooden point of your spear is hardening in the fire.".into());
        } else if self.spear_is_hardening(actor_id) {
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

    /// A sewn cloak on the body, named apart from what the arms hold.
    fn worn_cloak_phrase(&self, actor_id: ActorId) -> Option<String> {
        let (otter, fox, hare, beaver, wolf) = self
            .loose_ids(actor_id, Material::Hide)
            .into_iter()
            .find_map(|id| {
                let hide = self.item(id)?;
                hide.worn_cloak().then_some((
                    hide.otter,
                    hide.fox,
                    hide.hare,
                    hide.beaver,
                    hide.wolf,
                ))
            })?;
        let cloak = if otter {
            "otter-hide"
        } else if fox {
            "fox-hide"
        } else if hare {
            "hare-hide"
        } else if beaver {
            "beaver-hide"
        } else if wolf {
            "wolf-hide"
        } else {
            "deer-hide"
        };
        Some(format!("a sewn {cloak} cloak"))
    }

    /// The carrying sentence look already uses. Empty arms say so.
    /// A worn sewn cloak is the next sentence, not another thing in the list.
    pub fn arms_line(&self, actor_id: ActorId) -> String {
        let carrying = self.carry_views(actor_id);
        if carrying.is_empty() {
            return "Your arms are empty.".into();
        }
        let drinks = carrying
            .iter()
            .find(|item| item.material == "water")
            .map(|item| item.count)
            .unwrap_or(0);
        let raw_drinks = self
            .items
            .values()
            .filter(|item| {
                item.carrier == Some(actor_id)
                    && item.role == Role::Loose
                    && item.material == Material::Water
                    && item.raw
            })
            .count() as u32;
        let pot_holds_the_drink = drinks > 0
            && carrying
                .iter()
                .any(|item| item.material == "pot" && item.count == 1);
        let cup_sealed = self
            .loose_ids(actor_id, Material::BarkCup)
            .iter()
            .any(|id| self.item(*id).is_some_and(|cup| cup.pitched));
        let cup_holds_the_drink = !pot_holds_the_drink
            && drinks > 0
            && carrying
                .iter()
                .any(|item| item.material == "bark cup" && item.count == 1);
        let bits: Vec<String> = carrying
            .iter()
            .filter_map(|item| {
                if (pot_holds_the_drink || cup_holds_the_drink) && item.material == "water" {
                    return None;
                }
                if pot_holds_the_drink && item.material == "pot" {
                    return Some(pot_drink_phrase(drinks, raw_drinks));
                }
                if cup_holds_the_drink && item.material == "bark cup" {
                    return Some(cup_drink_phrase(drinks, raw_drinks, cup_sealed));
                }
                if item.material == "hide" {
                    let counting = self
                        .loose_ids(actor_id, Material::Hide)
                        .into_iter()
                        .filter(|id| self.item(*id).is_some_and(|hide| !hide.worn_cloak()))
                        .count() as u32;
                    if counting == 0 {
                        return None;
                    }
                    if counting != item.count {
                        let mut trimmed = item.clone();
                        trimmed.count = counting;
                        return Some(self.arm_bit(actor_id, &trimmed));
                    }
                }
                Some(self.arm_bit(actor_id, item))
            })
            .collect();
        let have = if bits.is_empty() {
            "Your arms are empty.".to_string()
        } else {
            format!("You have {}.", bits.join(", "))
        };
        match self.worn_cloak_phrase(actor_id) {
            Some(cloak) => format!("{have} You wear {cloak}."),
            None => have,
        }
    }

    fn person_text(&self, actor_id: ActorId) -> String {
        let actor = self.actor(actor_id).unwrap();
        let clothes = if actor.clothes_dry_at.is_some() {
            "drying by the fire".to_string()
        } else {
            self.clothes_for_them(actor.wetness)
        };
        format!(
            "{} is on the {}, {}. Their clothes are {}. They are {}. They are {}. They are {}. They are {}.",
            actor.name,
            self.cell(actor.cell).region.word(),
            self.cover_phrase(actor.cell),
            clothes,
            self.thirst_sentence(actor.thirst),
            self.hunger_word(actor.hunger),
            self.body_word(actor),
            self.energy_word(actor.energy)
        ) + &format!(
            " They are {}. They are {}. {}",
            self.cold_word(actor.cold),
            self.wind_on(actor_id),
            self.skin_sentence(actor, false)
        ) + &self
            .sun_held_off(actor.ochre)
            .map(|line| format!(" {line}"))
            .unwrap_or_default()
            + &self.spear_clause(actor_id)
            + &self.carcass_clause(actor_id)
    }

    /// A full red-ochre smear, the same cut of the sun mud already has.
    /// Thin ochre does not say it. Rain in the open takes the smear off.
    fn sun_held_off(&self, ochre: u32) -> Option<&'static str> {
        if ochre >= self.tuning.smear_at {
            Some("The red ochre holds the sun off.")
        } else {
            None
        }
    }

    /// Bare skin that is stinging does not also say it is bare.
    /// Mud and ochre still name the cover. The sting timer is unchanged.
    fn skin_sentence(&self, actor: &crate::model::Actor, yours: bool) -> String {
        let cover = self.skin_cover(actor.mud, actor.ochre);
        if actor.stings_until.is_some() && cover == "bare skin" {
            return if yours {
                "Your skin stings.".into()
            } else {
                "Their skin stings.".into()
            };
        }
        if yours {
            format!("Your skin is {cover}.")
        } else {
            format!("Their skin is {cover}.")
        }
    }

    fn skin_cover(&self, mud: u32, ochre: u32) -> String {
        if mud == 0 && ochre == 0 {
            return "bare skin".into();
        }
        if ochre == 0 {
            return self.mud_word(mud).into();
        }
        if mud == 0 {
            return self.ochre_word(ochre).into();
        }
        format!("{} and {}", self.mud_word(mud), self.ochre_word(ochre))
    }

    fn carcass_clause(&self, actor_id: ActorId) -> String {
        let carried = self.loose_ids(actor_id, Material::Carcass);
        if carried.is_empty() {
            String::new()
        } else if carried
            .iter()
            .any(|id| self.item(*id).is_some_and(|carcass| carcass.otter))
        {
            " They carry an otter carcass.".into()
        } else if carried
            .iter()
            .any(|id| self.item(*id).is_some_and(|carcass| carcass.wolf))
        {
            " They carry a wolf carcass.".into()
        } else if carried
            .iter()
            .any(|id| self.item(*id).is_some_and(|carcass| carcass.fox))
        {
            " They carry a fox carcass.".into()
        } else if carried
            .iter()
            .any(|id| self.item(*id).is_some_and(|carcass| carcass.hare))
        {
            " They carry a hare carcass.".into()
        } else if carried
            .iter()
            .any(|id| self.item(*id).is_some_and(|carcass| carcass.beaver))
        {
            " They carry a beaver carcass.".into()
        } else {
            " They carry a deer carcass.".into()
        }
    }

    fn flint_tip_words(&self, spear: &crate::model::Item) -> &'static str {
        if spear.bound_nettle {
            return if spear.hardened {
                "a fire-hardened flint tip bound with nettle cordage"
            } else {
                "a flint tip bound with nettle cordage, not yet fire-hardened"
            };
        }
        match (spear.hardened, spear.bound, spear.bound_willow) {
            (true, true, true) => "a fire-hardened flint tip bound with willow cordage",
            (true, true, false) => "a fire-hardened flint tip bound with hide",
            (true, false, _) => "a fire-hardened flint tip",
            (false, true, true) => "a flint tip bound with willow cordage, not yet fire-hardened",
            (false, true, false) => "a flint tip bound with hide, not yet fire-hardened",
            (false, false, _) => "a flint tip, not yet fire-hardened",
        }
    }

    fn spear_clause(&self, actor_id: ActorId) -> String {
        let Some(id) = self.loose_ids(actor_id, Material::Spear).first().copied() else {
            return String::new();
        };
        let spear = self.item(id);
        let hardened = spear.is_some_and(|item| item.hardened);
        if let Some(item) = spear.filter(|item| item.flint_tip) {
            format!(" They carry a spear with {}.", self.flint_tip_words(item))
        } else if spear.is_some_and(|item| item.wooden_tip) {
            if hardened {
                " They carry a wooden spear with a fire-hardened point.".into()
            } else if self.wooden_point_is_hardening(actor_id) {
                " They carry a wooden spear with a green point. The wooden point is hardening in the fire.".into()
            } else {
                " They carry a wooden spear with a green point.".into()
            }
        } else if hardened {
            " They carry a spear with a hardened point.".into()
        } else if self.spear_is_hardening(actor_id) {
            " They carry a spear with an unhardened point. The point is hardening in the fire."
                .into()
        } else {
            " They carry a spear with an unhardened point.".into()
        }
    }

    fn flint_tip_is_hardening(&self, actor_id: ActorId) -> bool {
        let Some(actor) = self.actor(actor_id) else {
            return false;
        };
        if self.fire_at(actor.cell).is_none() {
            return false;
        }
        self.loose_ids(actor_id, Material::Spear).iter().any(|id| {
            self.item(*id).is_some_and(|spear| {
                spear.flint_tip && !spear.hardened && spear.firing_ready_at.is_some()
            })
        })
    }

    fn spear_is_hardening(&self, actor_id: ActorId) -> bool {
        let Some(actor) = self.actor(actor_id) else {
            return false;
        };
        if self.fire_at(actor.cell).is_none() {
            return false;
        }
        self.loose_ids(actor_id, Material::Spear).iter().any(|id| {
            self.item(*id).is_some_and(|spear| {
                !spear.flint_tip
                    && !spear.wooden_tip
                    && !spear.hardened
                    && spear.firing_ready_at.is_some()
            })
        })
    }

    fn wooden_point_is_hardening(&self, actor_id: ActorId) -> bool {
        let Some(actor) = self.actor(actor_id) else {
            return false;
        };
        if self.fire_at(actor.cell).is_none() {
            return false;
        }
        self.loose_ids(actor_id, Material::Spear).iter().any(|id| {
            self.item(*id).is_some_and(|spear| {
                spear.wooden_tip && !spear.hardened && spear.firing_ready_at.is_some()
            })
        })
    }

    fn clay_firing_count(&self, cell: CellId) -> usize {
        self.ground_ids(cell, Material::Clay)
            .into_iter()
            .filter(|id| {
                self.item(*id)
                    .is_some_and(|item| !item.shaped && item.firing_ready_at.is_some())
            })
            .count()
    }

    fn pot_firing_count(&self, cell: CellId) -> usize {
        self.ground_ids(cell, Material::Clay)
            .into_iter()
            .filter(|id| {
                self.item(*id)
                    .is_some_and(|item| item.shaped && item.firing_ready_at.is_some())
            })
            .count()
    }

    /// Iron smelts only in a hot charcoal fire, and look names that heat.
    fn iron_smelt_fire(&self, cell: CellId) -> &'static str {
        if self.fire_at(cell).is_some_and(|fire| fire.charcoal) {
            "the hot charcoal fire"
        } else {
            "the fire"
        }
    }

    /// A hot charcoal fire is named. A plain fire stays "the fire".
    fn copper_smelt_fire(&self, cell: CellId) -> &'static str {
        if self.fire_at(cell).is_some_and(|fire| fire.charcoal) {
            "the hot charcoal fire"
        } else {
            "the fire"
        }
    }

    fn copper_smelting_count(&self, cell: CellId) -> usize {
        self.ground_ids(cell, Material::CopperOre)
            .into_iter()
            .filter(|id| {
                self.item(*id)
                    .and_then(|item| item.firing_ready_at)
                    .is_some()
            })
            .count()
    }

    fn iron_smelting_count(&self, cell: CellId) -> usize {
        self.ground_ids(cell, Material::IronOre)
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
                    return Some(format!(
                        "A deer stands {}, after eating the berries there.",
                        self.mark_place(origin, deer.cell)
                    ));
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

    fn adjacent_line(&self, viewer: ActorId, origin: CellId, dir: Dir) -> String {
        let line = self.adjacent_detail(viewer, origin, dir);
        if !self.dark_from(origin) || line.ends_with("the valley ends.") {
            return line;
        }
        if let Some(mark) = self.night_compass_mark(origin, dir) {
            return mark;
        }
        format!(
            "{}: you cannot make out the ground.",
            capitalize(dir.word())
        )
    }

    fn adjacent_detail(&self, viewer: ActorId, origin: CellId, dir: Dir) -> String {
        let (x, y) = cell_xy(origin);
        let (dx, dy) = dir.delta();
        let Some(dest) = cell_id(x + dx, y + dy) else {
            return format!("{}: the valley ends.", capitalize(dir.word()));
        };
        if let Some(floor) = self.outside_floor(origin) {
            if dest == floor.wood {
                if self.last_branch_is_still_there(dest) {
                    return format!(
                        "{}: the last branch. The deer left hoofprints on the ground there.",
                        capitalize(dir.word())
                    );
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
            bits.push(self.bitter_kind(viewer).into());
        }
        if self.pile_count(dest, Material::PaleBerry) > 0 {
            bits.push("pale berries".into());
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
        if self.pile_count(dest, Material::Reed) > 0
            || !self.ground_ids(dest, Material::Reed).is_empty()
        {
            bits.push("reeds".into());
        }
        if !self.ground_ids(dest, Material::Net).is_empty() {
            bits.push("a fishing net".into());
        }
        if !self.ground_ids(dest, Material::Mat).is_empty() {
            bits.push("a reed mat".into());
        }
        if let Some(id) = self.canoe_at(dest) {
            if self.canoe_pitched(id) {
                bits.push("a pitched dugout canoe".into());
            } else {
                bits.push("a dugout canoe".into());
            }
        }
        if self.pile_count(dest, Material::Stone) > 0
            || !self.ground_ids(dest, Material::Stone).is_empty()
        {
            bits.push("a stone".into());
        }
        if self.pile_count(dest, Material::Flint) > 0
            || !self.ground_ids(dest, Material::Flint).is_empty()
        {
            bits.push("a flint nodule".into());
        }
        if self.pile_count(dest, Material::Ochre) > 0
            || !self.ground_ids(dest, Material::Ochre).is_empty()
        {
            if self.cell(dest).rock {
                bits.push("a lump of ochre on the rock face".into());
            } else {
                bits.push("a lump of ochre".into());
            }
        }
        if self.pile_count(dest, Material::Quartz) > 0
            || !self.ground_ids(dest, Material::Quartz).is_empty()
        {
            bits.push("a clear stone of quartz on the rock face".into());
        } else if self.quartz_faces.contains(&dest) {
            bits.push("no quartz on the rock face".into());
        }
        if self.pile_count(dest, Material::Salt) > 0
            || !self.ground_ids(dest, Material::Salt).is_empty()
        {
            bits.push("a salt crust".into());
        }
        if self.pile_count(dest, Material::Honey) > 0
            || !self.ground_ids(dest, Material::Honey).is_empty()
        {
            bits.push("a bees' nest".into());
        } else if self.honey_nests.contains(&dest) {
            bits.push("an empty bees' nest".into());
        }
        if self.pile_count(dest, Material::Egg) > 0
            || !self.ground_ids(dest, Material::Egg).is_empty()
        {
            bits.push("a bird's nest".into());
        } else if self.bird_nests.contains(&dest) {
            bits.push("an empty bird's nest".into());
        }
        if self.pile_count(dest, Material::Moss) > 0
            || !self.ground_ids(dest, Material::Moss).is_empty()
        {
            if self.shade_moss.contains(&dest) {
                bits.push("dry moss".into());
            } else {
                bits.push("moss".into());
            }
        } else if self.shade_bare(dest) {
            bits.push("no moss".into());
        } else if self.moss_bare(dest) {
            bits.push("a bare bank".into());
        }
        if self.pile_count(dest, Material::Mint) > 0
            || !self.ground_ids(dest, Material::Mint).is_empty()
        {
            bits.push("wild mint".into());
        } else if self.mint_bare(dest) {
            bits.push("no mint".into());
        }
        if self.pile_count(dest, Material::Resin) > 0
            || !self.ground_ids(dest, Material::Resin).is_empty()
        {
            bits.push("pine resin".into());
        } else if self.resin_bare(dest) {
            bits.push("a bare pine".into());
        }
        if self.pile_count(dest, Material::Needle) > 0
            || !self.ground_ids(dest, Material::Needle).is_empty()
        {
            bits.push("pine needles".into());
        } else if self.needle_bare(dest) {
            bits.push("no needles".into());
        }
        if self.pile_count(dest, Material::Willow) > 0
            || !self.ground_ids(dest, Material::Willow).is_empty()
        {
            bits.push("willow cordage".into());
        } else if self.willow_bare(dest) {
            bits.push("a bare willow".into());
        }
        if self.pile_count(dest, Material::WillowBark) > 0
            || !self.ground_ids(dest, Material::WillowBark).is_empty()
        {
            bits.push("willow bark".into());
        } else if self.willow_bark_bare(dest) {
            bits.push("no willow bark".into());
        }
        if self.pile_count(dest, Material::Nettle) > 0
            || !self.ground_ids(dest, Material::Nettle).is_empty()
        {
            bits.push("nettles".into());
        } else if self.nettle_bare(dest) {
            bits.push("bare nettles".into());
        }
        if self.pile_count(dest, Material::IronOre) > 0
            || self.ground_ids(dest, Material::IronOre).len() > self.iron_smelting_count(dest)
        {
            bits.push("a lump of iron ore".into());
        }
        if self.iron_smelting_count(dest) > 0 {
            bits.push(format!(
                "iron ore smelting in {}",
                self.iron_smelt_fire(dest)
            ));
        }
        if !self.ground_ids(dest, Material::Iron).is_empty() {
            bits.push("a bloom of iron, metal from a hotter fire".into());
        }
        if !self.ground_ids(dest, Material::Knife).is_empty() {
            bits.push("an iron knife".into());
        }
        if self.pile_count(dest, Material::CopperOre) > 0
            || !self.ground_ids(dest, Material::CopperOre).is_empty()
        {
            bits.push("a lump of copper ore".into());
        }
        if !self.ground_ids(dest, Material::Copper).is_empty() {
            bits.push("a copper bead, metal from the fire".into());
        }
        if self
            .ground_ids(dest, Material::Awl)
            .iter()
            .any(|id| self.item(*id).is_some_and(|awl| !awl.bone))
        {
            bits.push("a copper awl".into());
        }
        if self
            .ground_ids(dest, Material::Awl)
            .iter()
            .any(|id| self.item(*id).is_some_and(|awl| awl.bone))
        {
            bits.push("a bone awl".into());
        }
        if !self.ground_ids(dest, Material::Bone).is_empty() {
            bits.push("a bone".into());
        }
        if !self.ground_ids(dest, Material::Hook).is_empty() {
            bits.push("a bone fish hook".into());
        }
        if self.ochre_marks.contains(&dest) {
            bits.push("a red ochre mark".into());
        }
        if self.charcoal_marks.contains(&dest) {
            bits.push("a black charcoal mark".into());
        }
        let clay_firing = self.clay_firing_count(dest);
        let unshaped_clay = self.ground_ids(dest, Material::Clay).iter().any(|id| {
            self.item(*id)
                .is_some_and(|clay| !clay.shaped && clay.firing_ready_at.is_none())
        });
        if clay_firing > 0 && self.pile_count(dest, Material::Clay) == 0 {
            bits.push("clay hardening in the fire".into());
        } else if self.pile_count(dest, Material::Clay) > 0 || unshaped_clay {
            bits.push("clay".into());
        }
        if self.pot_firing_count(dest) > 0 {
            bits.push("a pot hardening in the fire".into());
        } else if self.ground_ids(dest, Material::Clay).iter().any(|id| {
            self.item(*id)
                .is_some_and(|clay| clay.shaped && clay.firing_ready_at.is_none())
        }) {
            bits.push("an unfired clay pot".into());
        }
        if !self.ground_ids(dest, Material::Bowl).is_empty() {
            bits.push("a bowl".into());
        }
        let carcasses = self.ground_ids(dest, Material::Carcass);
        if carcasses
            .iter()
            .any(|id| self.item(*id).is_some_and(|carcass| carcass.otter))
        {
            bits.push("an otter carcass".into());
        }
        if carcasses
            .iter()
            .any(|id| self.item(*id).is_some_and(|carcass| carcass.wolf))
        {
            bits.push("a wolf carcass".into());
        }
        if carcasses
            .iter()
            .any(|id| self.item(*id).is_some_and(|carcass| carcass.fox))
        {
            bits.push("a fox carcass".into());
        }
        if carcasses
            .iter()
            .any(|id| self.item(*id).is_some_and(|carcass| carcass.hare))
        {
            bits.push("a hare carcass".into());
        }
        if carcasses
            .iter()
            .any(|id| self.item(*id).is_some_and(|carcass| carcass.beaver))
        {
            bits.push("a beaver carcass".into());
        }
        if carcasses.iter().any(|id| {
            self.item(*id).is_some_and(|carcass| {
                !carcass.wolf && !carcass.otter && !carcass.fox && !carcass.hare && !carcass.beaver
            })
        }) {
            bits.push("a carcass".into());
        }
        if let Some(bit) = self.meat_compass_bit(dest) {
            bits.push(bit);
        }
        if !self.ground_ids(dest, Material::Cord).is_empty() {
            bits.push("hide strips".into());
        }
        if !self.ground_ids(dest, Material::Bow).is_empty() {
            bits.push("a bow".into());
        }
        let drills_there = self.ground_ids(dest, Material::Drill);
        let spent_there = drills_there
            .iter()
            .filter(|id| self.item(**id).is_some_and(|drill| drill.spent))
            .count();
        if drills_there.len() > spent_there {
            bits.push("a bow drill".into());
        }
        if spent_there > 0 {
            bits.push("a spent bow drill".into());
        }
        if !self.ground_ids(dest, Material::Paddle).is_empty() {
            bits.push("a wooden paddle".into());
        }
        bits.extend(self.hide_compass_bits(&self.ground_ids(dest, Material::Hide)));
        if self
            .actors
            .values()
            .any(|person| person.cell == dest && self.spear_is_hardening(person.id))
        {
            bits.push("a spear hardening in the fire".into());
        }
        if self.ground_ids(dest, Material::Spear).len() == 1 {
            let spear = self
                .ground_ids(dest, Material::Spear)
                .first()
                .and_then(|id| self.item(*id));
            if let Some(item) = spear.filter(|spear| spear.flint_tip) {
                bits.push(format!("a spear with {}", self.flint_tip_words(item)));
            } else if spear.is_some_and(|spear| spear.wooden_tip) {
                if spear.is_some_and(|spear| spear.hardened) {
                    bits.push("a wooden spear with a fire-hardened point".into());
                } else {
                    bits.push("a wooden spear with a green point".into());
                }
            } else if spear.is_some_and(|spear| spear.hardened) {
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
        if self.pile_count(dest, Material::Cattail) > 0 && self.still_frozen(dest) {
            bits.push("cattails under the ice".into());
        } else if self.pile_count(dest, Material::Cattail) > 0 || self.cattail_bare(dest) {
            bits.push("cattails".into());
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
        if self.otter.as_ref().is_some_and(|otter| otter.cell == dest) {
            bits.push("an otter".into());
        }
        if self
            .beaver
            .as_ref()
            .is_some_and(|beaver| beaver.cell == dest)
        {
            bits.push("a beaver".into());
        }
        if self.fox.as_ref().is_some_and(|fox| fox.cell == dest) {
            bits.push("a fox".into());
        }
        if self.hare.as_ref().is_some_and(|hare| hare.cell == dest) {
            bits.push("a hare".into());
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
            if let Some(past) = self.person_on_the_berries_off_this_floor(viewer, origin, dest) {
                line.push_str(&past);
            }
            return line;
        }
        if let Some(line) = self.outside_floor_after_the_wood_is_gone(viewer, origin, dir, dest) {
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
        viewer: ActorId,
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
        if let Some(past) = self.person_on_the_berries_off_this_floor(viewer, origin, dest) {
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
        viewer: ActorId,
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
                " {} of that floor, {name} stands on the {}.",
                capitalize(way),
                self.bitter_kind(viewer)
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
            "{name} stands {cover}, on the {} north of the west floor.",
            self.bitter_kind(viewer)
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

    /// A fired pot, and an unfired one when that is what `examine pot` is asking about.
    /// The wait is named only while the unfired pot is still in the fire.
    fn examine_pot(&self, viewer: ActorId) -> String {
        let fired = self.examine_material(viewer, Material::Pot);
        let waiting = self.unfired_pot_lines(viewer);
        let none = fired == "You see no pot from here.";
        match (none, waiting) {
            (true, None) => fired,
            (true, Some(waiting)) => waiting,
            (false, None) => fired,
            (false, Some(waiting)) => format!("{fired}\n{waiting}"),
        }
    }

    fn unfired_pot_lines(&self, viewer: ActorId) -> Option<String> {
        let origin = self.actor(viewer)?.cell;
        let mut lines = Vec::new();
        for cell in self.visible_cell_ids(viewer) {
            for id in self.ground_ids(cell, Material::Clay) {
                if !self.item(id).is_some_and(|clay| clay.shaped) {
                    continue;
                }
                let firing = self
                    .item(id)
                    .and_then(|clay| clay.firing_ready_at)
                    .is_some();
                if firing {
                    if origin == cell {
                        lines.push(
                            "An unfired clay pot is hardening in the fire. It is not done yet."
                                .to_string(),
                        );
                    } else {
                        lines.push(format!(
                            "An unfired clay pot is hardening in the fire {}. It is not done yet.",
                            self.place_phrase(origin, cell)
                        ));
                    }
                } else {
                    lines.push(format!(
                        "An unfired clay pot lies {}. It is not hard yet.",
                        self.place_phrase(origin, cell)
                    ));
                }
            }
        }
        for other in self.visible_actor_ids(viewer) {
            let ids: Vec<_> = self
                .loose_ids(other, Material::Clay)
                .into_iter()
                .filter(|id| self.item(*id).is_some_and(|clay| clay.shaped))
                .collect();
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
                    "An unfired clay pot is carried by {who}, {}. It is not hard yet.",
                    self.cover_phrase(cell)
                ));
            } else {
                lines.push(format!(
                    "{} unfired clay pots are carried by {who}, {}. They are not hard yet.",
                    ids.len(),
                    self.cover_phrase(cell)
                ));
            }
        }
        if lines.is_empty() {
            None
        } else {
            Some(lines.join("\n"))
        }
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
            } else if self.clay_bare(cell) {
                if origin == cell {
                    lines.push("There is no clay on the bank.".into());
                } else {
                    lines.push(format!(
                        "There is no clay on the brook bank to the {}.",
                        self.relative(origin, cell)
                    ));
                }
            }
            for id in self.ground_ids(cell, Material::Clay) {
                if self.item(id).is_some_and(|clay| clay.shaped) {
                    let firing = self
                        .item(id)
                        .and_then(|clay| clay.firing_ready_at)
                        .is_some();
                    if firing {
                        if origin == cell {
                            lines.push(
                                "An unfired clay pot is hardening in the fire. It is not done yet."
                                    .into(),
                            );
                        } else {
                            lines.push(format!(
                                "An unfired clay pot is hardening in the fire {}. It is not done yet.",
                                self.place_phrase(origin, cell)
                            ));
                        }
                    } else {
                        lines.push(format!(
                            "An unfired clay pot lies {}. It is not hard yet.",
                            self.place_phrase(origin, cell)
                        ));
                    }
                    continue;
                }
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
            let pots = ids
                .iter()
                .filter(|id| self.item(**id).is_some_and(|clay| clay.shaped))
                .count();
            let lumps = ids.len() - pots;
            if pots == 1 {
                lines.push(format!(
                    "An unfired clay pot is carried by {who}, {}. It is not hard yet.",
                    self.cover_phrase(cell)
                ));
            } else if pots > 1 {
                lines.push(format!(
                    "{} unfired clay pots are carried by {who}, {}. They are not hard yet.",
                    pots,
                    self.cover_phrase(cell)
                ));
            }
            if lumps == 0 {
                continue;
            }
            if lumps == 1 {
                lines.push(format!(
                    "A lump of clay is carried by {who}, {}. It is unfired. It cannot boil water.",
                    self.cover_phrase(cell)
                ));
            } else {
                lines.push(format!(
                    "{} lumps of clay are carried by {who}, {}. They are unfired. They cannot boil water.",
                    capitalize(&count_word(lumps as u32)),
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
                let item = self.item(id);
                let moisture = item.map(|item| item.moisture).unwrap_or(0);
                let kind = if item.is_some_and(|carcass| carcass.otter) {
                    "otter"
                } else if item.is_some_and(|carcass| carcass.wolf) {
                    "wolf"
                } else if item.is_some_and(|carcass| carcass.fox) {
                    "fox"
                } else if item.is_some_and(|carcass| carcass.hare) {
                    "hare"
                } else if item.is_some_and(|carcass| carcass.beaver) {
                    "beaver"
                } else {
                    "deer"
                };
                let article = if kind == "otter" { "An" } else { "A" };
                lines.push(format!(
                    "{article} {kind} carcass lies in contact with the ground {}. It is {}. {} It must be cooked before it feeds you.",
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
            let carried = self.loose_ids(other, Material::Carcass);
            let kind = if carried
                .iter()
                .any(|id| self.item(*id).is_some_and(|carcass| carcass.otter))
            {
                "otter"
            } else if carried
                .iter()
                .any(|id| self.item(*id).is_some_and(|carcass| carcass.wolf))
            {
                "wolf"
            } else if carried
                .iter()
                .any(|id| self.item(*id).is_some_and(|carcass| carcass.fox))
            {
                "fox"
            } else if carried
                .iter()
                .any(|id| self.item(*id).is_some_and(|carcass| carcass.hare))
            {
                "hare"
            } else if carried
                .iter()
                .any(|id| self.item(*id).is_some_and(|carcass| carcass.beaver))
            {
                "beaver"
            } else {
                "deer"
            };
            let article = if kind == "otter" { "An" } else { "A" };
            lines.push(format!(
                "{article} {kind} carcass is carried by {who}, {}. It must be cooked before it feeds you.",
                self.cover_phrase(cell)
            ));
        }
        if lines.is_empty() {
            "You see no carcass from here.".into()
        } else {
            lines.join("\n")
        }
    }

    fn examine_brand(&self, viewer: ActorId) -> String {
        let origin = self.actor(viewer).unwrap().cell;
        let mut lines = Vec::new();
        for cell in self.visible_cell_ids(viewer) {
            for _id in self.ground_ids(cell, Material::Brand) {
                lines.push(format!(
                    "A burning brand lies in contact with the ground {}. It will not burn long.",
                    self.place_phrase(origin, cell)
                ));
            }
        }
        for other in self.visible_actor_ids(viewer) {
            if self.loose_ids(other, Material::Brand).is_empty() {
                continue;
            }
            let who = if other == viewer {
                "you".to_string()
            } else {
                self.name_of(other)
            };
            let cell = self.actor(other).unwrap().cell;
            lines.push(format!(
                "A burning brand is carried by {who}, {}. It will not burn long.",
                self.cover_phrase(cell)
            ));
        }
        if lines.is_empty() {
            "You see no brand from here.".into()
        } else {
            lines.join("\n")
        }
    }

    fn examine_charcoal(&self, viewer: ActorId) -> String {
        let origin = self.actor(viewer).unwrap().cell;
        let mut lines = Vec::new();
        for cell in self.visible_cell_ids(viewer) {
            for _id in self.ground_ids(cell, Material::Charcoal) {
                lines.push(format!(
                    "A lump of charcoal lies in contact with the ground {}. The fire left it. It is cold.",
                    self.place_phrase(origin, cell)
                ));
            }
        }
        for other in self.visible_actor_ids(viewer) {
            if self.loose_ids(other, Material::Charcoal).is_empty() {
                continue;
            }
            let who = if other == viewer {
                "you".to_string()
            } else {
                self.name_of(other)
            };
            let cell = self.actor(other).unwrap().cell;
            lines.push(format!(
                "A lump of charcoal is carried by {who}, {}. The fire left it. It is cold.",
                self.cover_phrase(cell)
            ));
        }
        for cell in self.visible_cell_ids(viewer) {
            if !self.charcoal_marks.contains(&cell) {
                continue;
            }
            let place = if self.cell(cell).rock {
                if origin == cell {
                    "on the rock here".to_string()
                } else {
                    format!("on the rock to the {}", self.relative(origin, cell))
                }
            } else {
                let region = self.cell(cell).region.word();
                if origin == cell {
                    format!("on the ground here, on the {region}")
                } else {
                    format!(
                        "on the ground on the {region} to the {}",
                        self.relative(origin, cell)
                    )
                }
            };
            lines.push(format!("A black charcoal mark is {place}."));
        }
        if lines.is_empty() {
            "You see no charcoal from here.".into()
        } else {
            lines.join("\n")
        }
    }

    fn examine_reeds(&self, viewer: ActorId) -> String {
        let origin = self.actor(viewer).unwrap().cell;
        let mut lines = Vec::new();
        for cell in self.visible_cell_ids(viewer) {
            let count = self.pile_count(cell, Material::Reed)
                + self.ground_ids(cell, Material::Reed).len() as u32;
            if count == 0 {
                if self.reed_bare(cell) {
                    if origin == cell {
                        lines.push("The brook's edge here has no reeds.".into());
                    } else {
                        lines.push(format!(
                            "The brook's edge to the {} has no reeds.",
                            self.relative(origin, cell)
                        ));
                    }
                }
                continue;
            }
            let place = if origin == cell {
                if self.beside_the_brook(cell) {
                    "here, at the brook's edge".to_string()
                } else {
                    "here".to_string()
                }
            } else {
                format!("to the {}", self.relative(origin, cell))
            };
            lines.push(format!("Reeds grow {place}."));
        }
        for other in self.visible_actor_ids(viewer) {
            let count = self.loose_ids(other, Material::Reed).len() as u32;
            if count == 0 {
                continue;
            }
            let who = if other == viewer {
                "you".to_string()
            } else {
                self.name_of(other)
            };
            let bundle = if count == 1 {
                "A bundle of reeds is".to_string()
            } else {
                format!("{} bundles of reeds are", capitalize(&count_word(count)))
            };
            lines.push(format!(
                "{bundle} carried by {who}, {}.",
                self.cover_phrase(self.actor(other).unwrap().cell)
            ));
        }
        if lines.is_empty() {
            "You see no reeds from here.".into()
        } else {
            lines.join("\n")
        }
    }

    fn examine_fish(&self, viewer: ActorId) -> String {
        let origin = self.actor(viewer).unwrap().cell;
        let mut lines = Vec::new();
        for cell in self.visible_cell_ids(viewer) {
            for id in self.ground_ids(cell, Material::Fish) {
                let place = self.place_phrase(origin, cell);
                lines.push(match self.fish_state(id) {
                    "cooked" => format!(
                        "A cooked fish lies in contact with the ground {place}. It is good to eat."
                    ),
                    "smoked" => format!(
                        "A smoked fish lies in contact with the ground {place}. It can be eaten cold."
                    ),
                    "salted" => format!(
                        "A salted fish lies in contact with the ground {place}. It can be eaten cold."
                    ),
                    "spoiled" => format!(
                        "A spoiled fish lies in contact with the ground {place}. It is not food."
                    ),
                    "going" => format!(
                        "A raw fish lies in contact with the ground {place}. It is going bad."
                    ),
                    "hanging" => format!(
                        "A raw fish hangs by the fire {place}. It is not smoked yet."
                    ),
                    "set" => format!(
                        "A raw fish is set near the fire {place}. It is not smoked yet."
                    ),
                    _ => format!(
                        "A raw fish lies in contact with the ground {place}. It must be cooked before it feeds you."
                    ),
                });
            }
        }
        for other in self.visible_actor_ids(viewer) {
            let who = if other == viewer {
                "you".to_string()
            } else {
                self.name_of(other)
            };
            let cell = self.actor(other).unwrap().cell;
            let cover = self.cover_phrase(cell);
            for id in self.loose_ids(other, Material::Fish) {
                let in_basket = self.item(id).is_some_and(|fish| fish.in_basket);
                lines.push(match (in_basket, self.fish_state(id)) {
                    (true, "cooked") => format!(
                        "A cooked fish is in the bark basket carried by {who}, {cover}. It is good to eat."
                    ),
                    (true, "smoked") => format!(
                        "A smoked fish is in the bark basket carried by {who}, {cover}. It can be eaten cold."
                    ),
                    (true, "salted") => format!(
                        "A salted fish is in the bark basket carried by {who}, {cover}. It can be eaten cold."
                    ),
                    (true, "spoiled") => format!(
                        "A spoiled fish is in the bark basket carried by {who}, {cover}. It is not food."
                    ),
                    (true, "going") => format!(
                        "A raw fish is in the bark basket carried by {who}, {cover}. It is going bad."
                    ),
                    (true, _) => format!(
                        "A raw fish is in the bark basket carried by {who}, {cover}. It must be cooked before it feeds you."
                    ),
                    (false, "cooked") => {
                        format!("A cooked fish is carried by {who}, {cover}. It is good to eat.")
                    }
                    (false, "smoked") => format!(
                        "A smoked fish is carried by {who}, {cover}. It can be eaten cold."
                    ),
                    (false, "salted") => format!(
                        "A salted fish is carried by {who}, {cover}. It can be eaten cold."
                    ),
                    (false, "spoiled") => {
                        format!("A spoiled fish is carried by {who}, {cover}. It is not food.")
                    }
                    (false, "going") => {
                        format!("A raw fish is carried by {who}, {cover}. It is going bad.")
                    }
                    (false, _) => format!(
                        "A raw fish is carried by {who}, {cover}. It must be cooked before it feeds you."
                    ),
                });
            }
        }
        if lines.is_empty() {
            "You see no fish from here.".into()
        } else {
            lines.join("\n")
        }
    }

    fn examine_meat(&self, viewer: ActorId) -> String {
        let origin = self.actor(viewer).unwrap().cell;
        let mut lines = Vec::new();
        for cell in self.visible_cell_ids(viewer) {
            let ids = self.ground_ids(cell, Material::Meat);
            if ids.is_empty() {
                continue;
            }
            let place = self.place_phrase(origin, cell);
            lines.extend(self.meat_place_lines(&ids, &place));
        }
        for other in self.visible_actor_ids(viewer) {
            let ids = self.loose_ids(other, Material::Meat);
            if ids.is_empty() {
                continue;
            }
            let who = if other == viewer {
                "you".to_string()
            } else {
                self.name_of(other)
            };
            let cover = self.cover_phrase(self.actor(other).unwrap().cell);
            lines.extend(self.meat_held_lines(&ids, &who, &cover));
        }
        if lines.is_empty() {
            "You see no meat from here.".into()
        } else {
            lines.join("\n")
        }
    }

    fn meat_held_lines(&self, ids: &[crate::model::ItemId], who: &str, cover: &str) -> Vec<String> {
        let mut lines = Vec::new();
        for face in ["raw", "going", "spoiled", "salted", "smoked", "cooked"] {
            let full = self.meat_tally(ids, face, false);
            let small = self.meat_tally(ids, face, true);
            if full > 0 {
                lines.push(meat_held(face, full, false, who, cover));
            }
            if small > 0 {
                lines.push(meat_held(face, small, true, who, cover));
            }
        }
        lines
    }

    fn plain_scraped(
        &self,
        ids: &[crate::model::ItemId],
        pred: impl Fn(&crate::model::Item) -> bool,
    ) -> bool {
        ids.iter().any(|id| {
            self.item(*id)
                .is_some_and(|hide| pred(hide) && hide.scraped && !hide.pierced && !hide.sewn)
        })
    }

    fn hide_stage(&self, ids: &[crate::model::ItemId], wolf: bool) -> Option<(bool, bool)> {
        let mine: Vec<_> = ids
            .iter()
            .copied()
            .filter(|id| {
                self.item(*id).is_some_and(|hide| {
                    hide.wolf == wolf && !hide.otter && !hide.fox && !hide.hare && !hide.beaver
                })
            })
            .collect();
        if mine.is_empty() {
            return None;
        }
        let sewn = mine
            .iter()
            .any(|id| self.item(*id).is_some_and(|hide| hide.sewn));
        let pierced = mine
            .iter()
            .any(|id| self.item(*id).is_some_and(|hide| hide.pierced));
        Some((sewn, pierced))
    }

    fn hide_here_lines(&self, ids: &[crate::model::ItemId]) -> Vec<String> {
        let mut lines = Vec::new();
        for wolf in [true, false] {
            let Some((sewn, pierced)) = self.hide_stage(ids, wolf) else {
                continue;
            };
            let kind = if wolf { "wolf" } else { "deer" };
            let cloak = if wolf { "wolf-hide" } else { "deer-hide" };
            if sewn {
                lines.push(format!("A sewn {cloak} cloak lies here."));
            } else if pierced {
                lines.push(format!(
                    "A pierced {kind} hide lies here. It is ready to sew."
                ));
            } else if self.plain_scraped(ids, |hide| {
                hide.wolf == wolf && !hide.otter && !hide.fox && !hide.hare && !hide.beaver
            }) {
                lines.push(format!("A scraped {kind} hide lies here."));
            } else {
                lines.push(format!("A {kind} hide lies here."));
            }
        }
        lines.extend(self.otter_hide_here(&ids));
        lines.extend(self.fox_hide_here(&ids));
        lines.extend(self.hare_hide_here(&ids));
        lines.extend(self.beaver_hide_here(&ids));
        lines
    }

    fn fox_hide_stage(&self, ids: &[ItemId]) -> Option<(bool, bool)> {
        let fox: Vec<_> = ids
            .iter()
            .copied()
            .filter(|id| self.item(*id).is_some_and(|hide| hide.fox))
            .collect();
        if fox.is_empty() {
            return None;
        }
        let sewn = fox
            .iter()
            .any(|id| self.item(*id).is_some_and(|hide| hide.sewn));
        let pierced = fox
            .iter()
            .any(|id| self.item(*id).is_some_and(|hide| hide.pierced));
        Some((sewn, pierced))
    }

    fn fox_hide_here(&self, ids: &[ItemId]) -> Vec<String> {
        let Some((sewn, pierced)) = self.fox_hide_stage(ids) else {
            return Vec::new();
        };
        if sewn {
            vec!["A sewn fox-hide cloak lies here. It breaks the wind.".into()]
        } else if pierced {
            vec!["A pierced fox hide lies here. It is ready to sew.".into()]
        } else if self.plain_scraped(ids, |hide| hide.fox) {
            vec!["A scraped fox hide lies here.".into()]
        } else {
            vec!["A fox hide lies here.".into()]
        }
    }

    fn fox_hide_compass(&self, ids: &[ItemId]) -> Vec<String> {
        let Some((sewn, pierced)) = self.fox_hide_stage(ids) else {
            return Vec::new();
        };
        if sewn {
            vec!["a sewn fox-hide cloak".into()]
        } else if pierced {
            vec!["a pierced fox hide".into()]
        } else if self.plain_scraped(ids, |hide| hide.fox) {
            vec!["a scraped fox hide".into()]
        } else {
            vec!["a fox hide".into()]
        }
    }

    fn fox_hide_ground_line(&self, ids: &[ItemId], place: &str) -> Option<String> {
        let Some((sewn, pierced)) = self.fox_hide_stage(ids) else {
            return None;
        };
        if sewn {
            Some(format!(
                "A sewn fox-hide cloak lies {place}. It breaks the wind."
            ))
        } else if pierced {
            Some(format!(
                "A pierced fox hide lies {place}. It is ready to sew."
            ))
        } else if self.plain_scraped(ids, |hide| hide.fox) {
            Some(format!("A scraped fox hide lies {place}."))
        } else {
            Some(format!("A fox hide lies {place}."))
        }
    }

    fn fox_hide_held_line(&self, hides: &[ItemId], who: &str, cover: &str) -> Option<String> {
        let Some((sewn, pierced)) = self.fox_hide_stage(hides) else {
            return None;
        };
        let worn = hides
            .iter()
            .any(|id| self.item(*id).is_some_and(|hide| hide.fox && hide.worn));
        if sewn && worn {
            Some(format!(
                "A sewn fox-hide cloak is on {who}, {cover}. It breaks the wind."
            ))
        } else if sewn {
            Some(format!(
                "A sewn fox-hide cloak is carried by {who}, {cover}. It breaks the wind."
            ))
        } else if pierced && worn {
            Some(format!(
                "A pierced fox hide is on {who}, {cover}. It is ready to sew."
            ))
        } else if pierced {
            Some(format!(
                "A pierced fox hide is carried by {who}, {cover}. It is ready to sew."
            ))
        } else if worn && self.plain_scraped(hides, |hide| hide.fox) {
            Some(format!("A scraped fox hide is on {who}, {cover}."))
        } else if worn {
            Some(format!("A fox hide is on {who}, {cover}."))
        } else if self.plain_scraped(hides, |hide| hide.fox) {
            Some(format!("A scraped fox hide is carried by {who}, {cover}."))
        } else {
            Some(format!("A fox hide is carried by {who}, {cover}."))
        }
    }

    fn hare_hide_ids(&self, ids: &[ItemId]) -> Vec<ItemId> {
        ids.iter()
            .copied()
            .filter(|id| self.item(*id).is_some_and(|hide| hide.hare))
            .collect()
    }

    fn hare_hide_stage(&self, ids: &[ItemId]) -> Option<(bool, bool)> {
        let hare = self.hare_hide_ids(ids);
        if hare.is_empty() {
            return None;
        }
        let sewn = hare
            .iter()
            .any(|id| self.item(*id).is_some_and(|hide| hide.sewn));
        let pierced = hare
            .iter()
            .any(|id| self.item(*id).is_some_and(|hide| hide.pierced));
        Some((sewn, pierced))
    }

    fn hare_hide_here(&self, ids: &[ItemId]) -> Vec<String> {
        let Some((sewn, pierced)) = self.hare_hide_stage(ids) else {
            return Vec::new();
        };
        if sewn {
            vec!["A sewn hare-hide cloak lies here. It is a light cover.".into()]
        } else if pierced {
            vec!["A pierced hare hide lies here. It is ready to sew.".into()]
        } else if self.plain_scraped(ids, |hide| hide.hare) {
            vec!["A scraped hare hide lies here.".into()]
        } else {
            vec!["A hare hide lies here.".into()]
        }
    }

    fn hare_hide_compass(&self, ids: &[ItemId]) -> Vec<String> {
        let Some((sewn, pierced)) = self.hare_hide_stage(ids) else {
            return Vec::new();
        };
        if sewn {
            vec!["a sewn hare-hide cloak".into()]
        } else if pierced {
            vec!["a pierced hare hide".into()]
        } else if self.plain_scraped(ids, |hide| hide.hare) {
            vec!["a scraped hare hide".into()]
        } else {
            vec!["a hare hide".into()]
        }
    }

    fn hare_hide_ground_line(&self, ids: &[ItemId], place: &str) -> Option<String> {
        let Some((sewn, pierced)) = self.hare_hide_stage(ids) else {
            return None;
        };
        if sewn {
            Some(format!(
                "A sewn hare-hide cloak lies {place}. It is a light cover."
            ))
        } else if pierced {
            Some(format!(
                "A pierced hare hide lies {place}. It is ready to sew."
            ))
        } else if self.plain_scraped(ids, |hide| hide.hare) {
            Some(format!("A scraped hare hide lies {place}."))
        } else {
            Some(format!("A hare hide lies {place}."))
        }
    }

    fn hare_hide_held_line(&self, hides: &[ItemId], who: &str, cover: &str) -> Option<String> {
        let Some((sewn, pierced)) = self.hare_hide_stage(hides) else {
            return None;
        };
        let worn = hides
            .iter()
            .any(|id| self.item(*id).is_some_and(|hide| hide.hare && hide.worn));
        if sewn && worn {
            Some(format!(
                "A sewn hare-hide cloak is on {who}, {cover}. It is a light cover."
            ))
        } else if sewn {
            Some(format!(
                "A sewn hare-hide cloak is carried by {who}, {cover}. It is a light cover."
            ))
        } else if pierced && worn {
            Some(format!(
                "A pierced hare hide is on {who}, {cover}. It is ready to sew."
            ))
        } else if pierced {
            Some(format!(
                "A pierced hare hide is carried by {who}, {cover}. It is ready to sew."
            ))
        } else if worn && self.plain_scraped(hides, |hide| hide.hare) {
            Some(format!("A scraped hare hide is on {who}, {cover}."))
        } else if worn {
            Some(format!("A hare hide is on {who}, {cover}."))
        } else if self.plain_scraped(hides, |hide| hide.hare) {
            Some(format!("A scraped hare hide is carried by {who}, {cover}."))
        } else {
            Some(format!("A hare hide is carried by {who}, {cover}."))
        }
    }

    fn beaver_hide_ids(&self, ids: &[ItemId]) -> Vec<ItemId> {
        ids.iter()
            .copied()
            .filter(|id| self.item(*id).is_some_and(|hide| hide.beaver))
            .collect()
    }

    fn beaver_hide_stage(&self, ids: &[ItemId]) -> Option<(bool, bool)> {
        let beaver = self.beaver_hide_ids(ids);
        if beaver.is_empty() {
            return None;
        }
        let sewn = beaver
            .iter()
            .any(|id| self.item(*id).is_some_and(|hide| hide.sewn));
        let pierced = beaver
            .iter()
            .any(|id| self.item(*id).is_some_and(|hide| hide.pierced));
        Some((sewn, pierced))
    }

    fn beaver_hide_here(&self, ids: &[ItemId]) -> Vec<String> {
        let Some((sewn, pierced)) = self.beaver_hide_stage(ids) else {
            return Vec::new();
        };
        if sewn {
            vec!["A sewn beaver-hide cloak lies here. It sheds the rain and the brook.".into()]
        } else if pierced {
            vec!["A pierced beaver hide lies here. It is ready to sew.".into()]
        } else if self.plain_scraped(ids, |hide| hide.beaver) {
            vec!["A scraped beaver hide lies here.".into()]
        } else {
            vec!["A beaver hide lies here.".into()]
        }
    }

    fn beaver_hide_compass(&self, ids: &[ItemId]) -> Vec<String> {
        let Some((sewn, pierced)) = self.beaver_hide_stage(ids) else {
            return Vec::new();
        };
        if sewn {
            vec!["a sewn beaver-hide cloak".into()]
        } else if pierced {
            vec!["a pierced beaver hide".into()]
        } else if self.plain_scraped(ids, |hide| hide.beaver) {
            vec!["a scraped beaver hide".into()]
        } else {
            vec!["a beaver hide".into()]
        }
    }

    fn beaver_hide_ground_line(&self, ids: &[ItemId], place: &str) -> Option<String> {
        let Some((sewn, pierced)) = self.beaver_hide_stage(ids) else {
            return None;
        };
        if sewn {
            Some(format!(
                "A sewn beaver-hide cloak lies {place}. It sheds the rain and the brook."
            ))
        } else if pierced {
            Some(format!(
                "A pierced beaver hide lies {place}. It is ready to sew."
            ))
        } else if self.plain_scraped(ids, |hide| hide.beaver) {
            Some(format!("A scraped beaver hide lies {place}."))
        } else {
            Some(format!("A beaver hide lies {place}."))
        }
    }

    fn beaver_hide_held_line(&self, hides: &[ItemId], who: &str, cover: &str) -> Option<String> {
        let Some((sewn, pierced)) = self.beaver_hide_stage(hides) else {
            return None;
        };
        let worn = hides
            .iter()
            .any(|id| self.item(*id).is_some_and(|hide| hide.beaver && hide.worn));
        if sewn && worn {
            Some(format!(
                "A sewn beaver-hide cloak is on {who}, {cover}. It sheds the rain and the brook."
            ))
        } else if sewn {
            Some(format!(
                "A sewn beaver-hide cloak is carried by {who}, {cover}. It sheds the rain and the brook."
            ))
        } else if pierced && worn {
            Some(format!(
                "A pierced beaver hide is on {who}, {cover}. It is ready to sew."
            ))
        } else if pierced {
            Some(format!(
                "A pierced beaver hide is carried by {who}, {cover}. It is ready to sew."
            ))
        } else if worn && self.plain_scraped(hides, |hide| hide.beaver) {
            Some(format!("A scraped beaver hide is on {who}, {cover}."))
        } else if worn {
            Some(format!("A beaver hide is on {who}, {cover}."))
        } else if self.plain_scraped(hides, |hide| hide.beaver) {
            Some(format!(
                "A scraped beaver hide is carried by {who}, {cover}."
            ))
        } else {
            Some(format!("A beaver hide is carried by {who}, {cover}."))
        }
    }

    fn hide_compass_bits(&self, ids: &[crate::model::ItemId]) -> Vec<String> {
        let mut bits = Vec::new();
        for wolf in [true, false] {
            let Some((sewn, pierced)) = self.hide_stage(ids, wolf) else {
                continue;
            };
            let kind = if wolf { "wolf" } else { "deer" };
            let cloak = if wolf { "wolf-hide" } else { "deer-hide" };
            if sewn {
                bits.push(format!("a sewn {cloak} cloak"));
            } else if pierced {
                bits.push(format!("a pierced {kind} hide (ready to sew)"));
            } else if self.plain_scraped(ids, |hide| {
                hide.wolf == wolf && !hide.otter && !hide.fox && !hide.hare && !hide.beaver
            }) {
                bits.push(format!("a scraped {kind} hide"));
            } else {
                bits.push(format!("a {kind} hide"));
            }
        }
        bits.extend(self.otter_hide_compass(&ids));
        bits.extend(self.fox_hide_compass(&ids));
        bits.extend(self.hare_hide_compass(&ids));
        bits.extend(self.beaver_hide_compass(&ids));
        bits
    }

    fn otter_hide_here(&self, ids: &[ItemId]) -> Vec<String> {
        let otter: Vec<_> = ids
            .iter()
            .copied()
            .filter(|id| self.item(*id).is_some_and(|hide| hide.otter))
            .collect();
        if otter.is_empty() {
            return Vec::new();
        }
        let sewn = otter
            .iter()
            .any(|id| self.item(*id).is_some_and(|hide| hide.sewn));
        let pierced = otter
            .iter()
            .any(|id| self.item(*id).is_some_and(|hide| hide.pierced));
        if sewn {
            vec!["A sewn otter-hide cloak lies here.".into()]
        } else if pierced {
            vec!["A pierced otter hide lies here. It is ready to sew.".into()]
        } else if self.plain_scraped(ids, |hide| hide.otter) {
            vec!["A scraped otter hide lies here.".into()]
        } else {
            vec!["An otter hide lies here.".into()]
        }
    }

    fn otter_hide_compass(&self, ids: &[ItemId]) -> Vec<String> {
        let otter: Vec<_> = ids
            .iter()
            .copied()
            .filter(|id| self.item(*id).is_some_and(|hide| hide.otter))
            .collect();
        if otter.is_empty() {
            return Vec::new();
        }
        let sewn = otter
            .iter()
            .any(|id| self.item(*id).is_some_and(|hide| hide.sewn));
        let pierced = otter
            .iter()
            .any(|id| self.item(*id).is_some_and(|hide| hide.pierced));
        if sewn {
            vec!["a sewn otter-hide cloak".into()]
        } else if pierced {
            vec!["a pierced otter hide".into()]
        } else if self.plain_scraped(&otter, |hide| hide.otter) {
            vec!["a scraped otter hide".into()]
        } else {
            vec!["an otter hide".into()]
        }
    }

    fn otter_hide_ground_line(&self, ids: &[ItemId], place: &str) -> Option<String> {
        let otter: Vec<_> = ids
            .iter()
            .copied()
            .filter(|id| self.item(*id).is_some_and(|hide| hide.otter))
            .collect();
        if otter.is_empty() {
            return None;
        }
        let sewn = otter
            .iter()
            .any(|id| self.item(*id).is_some_and(|hide| hide.sewn));
        let pierced = otter
            .iter()
            .any(|id| self.item(*id).is_some_and(|hide| hide.pierced));
        if sewn {
            Some(format!(
                "A sewn otter-hide cloak lies {place}. It keeps the rain and the brook off."
            ))
        } else if pierced {
            Some(format!(
                "A pierced otter hide lies {place}. It is ready to sew."
            ))
        } else if self.plain_scraped(&otter, |hide| hide.otter) {
            Some(format!("A scraped otter hide lies {place}."))
        } else {
            Some(format!("An otter hide lies {place}."))
        }
    }

    fn otter_hide_held_line(&self, hides: &[ItemId], who: &str, cover: &str) -> Option<String> {
        let otter: Vec<_> = hides
            .iter()
            .copied()
            .filter(|id| self.item(*id).is_some_and(|hide| hide.otter))
            .collect();
        if otter.is_empty() {
            return None;
        }
        let sewn = otter
            .iter()
            .any(|id| self.item(*id).is_some_and(|hide| hide.sewn));
        let pierced = otter
            .iter()
            .any(|id| self.item(*id).is_some_and(|hide| hide.pierced));
        let worn = otter
            .iter()
            .any(|id| self.item(*id).is_some_and(|hide| hide.worn));
        if sewn && worn {
            Some(format!(
                "A sewn otter-hide cloak is on {who}, {cover}. It keeps the rain and the brook off."
            ))
        } else if sewn {
            Some(format!(
                "A sewn otter-hide cloak is carried by {who}, {cover}. It keeps the rain and the brook off."
            ))
        } else if pierced && worn {
            Some(format!(
                "A pierced otter hide is on {who}, {cover}. It is ready to sew."
            ))
        } else if pierced {
            Some(format!(
                "A pierced otter hide is carried by {who}, {cover}. It is ready to sew."
            ))
        } else if worn && self.plain_scraped(&otter, |hide| hide.otter) {
            Some(format!("A scraped otter hide is on {who}, {cover}."))
        } else if worn {
            Some(format!("An otter hide is on {who}, {cover}."))
        } else if self.plain_scraped(&otter, |hide| hide.otter) {
            Some(format!(
                "A scraped otter hide is carried by {who}, {cover}."
            ))
        } else {
            Some(format!("An otter hide is carried by {who}, {cover}."))
        }
    }

    fn examine_hide(&self, viewer: ActorId) -> String {
        let origin = self.actor(viewer).unwrap().cell;
        let mut lines = Vec::new();
        for cell in self.visible_cell_ids(viewer) {
            let ids = self.ground_ids(cell, Material::Hide);
            if ids.is_empty() {
                continue;
            }
            let place = self.place_phrase(origin, cell);
            for wolf in [true, false] {
                let Some((sewn, pierced)) = self.hide_stage(&ids, wolf) else {
                    continue;
                };
                if wolf {
                    if sewn {
                        lines.push(format!(
                            "A sewn wolf-hide cloak lies {place}. It keeps off more of the cold."
                        ));
                    } else if pierced {
                        lines.push(format!(
                            "A pierced wolf hide lies {place}. It is ready to sew."
                        ));
                    } else if self.plain_scraped(&ids, |hide| {
                        hide.wolf && !hide.otter && !hide.fox && !hide.hare && !hide.beaver
                    }) {
                        lines.push(format!("A scraped wolf hide lies {place}."));
                    } else {
                        lines.push(format!("A wolf hide lies {place}."));
                    }
                } else if sewn {
                    lines.push(format!(
                        "A sewn deer-hide cloak lies {place}. It can be worn against the cold."
                    ));
                } else if pierced {
                    lines.push(format!(
                        "A pierced deer hide lies {place}. It is ready to sew."
                    ));
                } else if self.plain_scraped(&ids, |hide| {
                    !hide.wolf && !hide.otter && !hide.fox && !hide.hare && !hide.beaver
                }) {
                    lines.push(format!("A scraped deer hide lies {place}."));
                } else {
                    lines.push(format!("A deer hide lies {place}."));
                }
            }
            if let Some(line) = self.otter_hide_ground_line(&ids, &place) {
                lines.push(line);
            }
            if let Some(line) = self.fox_hide_ground_line(&ids, &place) {
                lines.push(line);
            }
            if let Some(line) = self.hare_hide_ground_line(&ids, &place) {
                lines.push(line);
            }
            if let Some(line) = self.beaver_hide_ground_line(&ids, &place) {
                lines.push(line);
            }
        }
        for other in self.visible_actor_ids(viewer) {
            let hides = self.loose_ids(other, Material::Hide);
            if hides.is_empty() {
                continue;
            }
            let who = if other == viewer {
                "you".to_string()
            } else {
                self.name_of(other)
            };
            let cell = self.actor(other).unwrap().cell;
            let cover = self.cover_phrase(cell);
            for wolf in [true, false] {
                let mine: Vec<_> = hides
                    .iter()
                    .copied()
                    .filter(|id| {
                        self.item(*id).is_some_and(|hide| {
                            hide.wolf == wolf
                                && !hide.otter
                                && !hide.fox
                                && !hide.hare
                                && !hide.beaver
                        })
                    })
                    .collect();
                if mine.is_empty() {
                    continue;
                }
                let sewn = mine
                    .iter()
                    .any(|id| self.item(*id).is_some_and(|hide| hide.sewn));
                let pierced = mine
                    .iter()
                    .any(|id| self.item(*id).is_some_and(|hide| hide.pierced));
                let worn = mine
                    .iter()
                    .any(|id| self.item(*id).is_some_and(|hide| hide.worn));
                if wolf {
                    if sewn && worn {
                        lines.push(format!(
                            "A sewn wolf-hide cloak is on {who}, {cover}. It keeps off more of the wind and the cold."
                        ));
                    } else if sewn {
                        lines.push(format!(
                            "A sewn wolf-hide cloak is carried by {who}, {cover}. It keeps off more of the cold."
                        ));
                    } else if pierced && worn {
                        lines.push(format!(
                            "A pierced wolf hide is on {who}, {cover}. It is ready to sew."
                        ));
                    } else if pierced {
                        lines.push(format!(
                            "A pierced wolf hide is carried by {who}, {cover}. It is ready to sew."
                        ));
                    } else if worn && self.plain_scraped(&mine, |hide| hide.wolf) {
                        lines.push(format!("A scraped wolf hide is on {who}, {cover}."));
                    } else if worn {
                        lines.push(format!("A wolf hide is on {who}, {cover}."));
                    } else if self.plain_scraped(&mine, |hide| hide.wolf) {
                        lines.push(format!("A scraped wolf hide is carried by {who}, {cover}."));
                    } else {
                        lines.push(format!("A wolf hide is carried by {who}, {cover}."));
                    }
                } else if sewn && worn {
                    lines.push(format!(
                        "A sewn deer-hide cloak is on {who}, {cover}. It keeps off the cold."
                    ));
                } else if sewn {
                    lines.push(format!(
                        "A sewn deer-hide cloak is carried by {who}, {cover}. It can be worn against the cold."
                    ));
                } else if pierced && worn {
                    lines.push(format!(
                        "A pierced deer hide is on {who}, {cover}. It is ready to sew."
                    ));
                } else if pierced {
                    lines.push(format!(
                        "A pierced deer hide is carried by {who}, {cover}. It is ready to sew."
                    ));
                } else if self.plain_scraped(&mine, |hide| {
                    !hide.wolf && !hide.otter && !hide.fox && !hide.hare && !hide.beaver
                }) {
                    lines.push(format!("A scraped deer hide is carried by {who}, {cover}."));
                } else {
                    lines.push(format!("A deer hide is carried by {who}, {cover}."));
                }
            }
            if let Some(line) = self.otter_hide_held_line(&hides, &who, &cover) {
                lines.push(line);
            }
            if let Some(line) = self.fox_hide_held_line(&hides, &who, &cover) {
                lines.push(line);
            }
            if let Some(line) = self.hare_hide_held_line(&hides, &who, &cover) {
                lines.push(line);
            }
            if let Some(line) = self.beaver_hide_held_line(&hides, &who, &cover) {
                lines.push(line);
            }
        }
        if lines.is_empty() {
            "You see no hide from here.".into()
        } else {
            lines.join("\n")
        }
    }

    fn examine_cord(&self, viewer: ActorId) -> String {
        let origin = self.actor(viewer).unwrap().cell;
        let mut lines = Vec::new();
        for cell in self.visible_cell_ids(viewer) {
            if self.ground_ids(cell, Material::Cord).is_empty() {
                continue;
            }
            let place = if origin == cell {
                "here".to_string()
            } else {
                format!("on the ground to the {}", self.relative(origin, cell))
            };
            lines.push(format!("Hide strips lie {place}. They are cordage."));
        }
        for actor in self.actors.values() {
            if chebyshev(origin, actor.cell) > self.tuning.sight {
                continue;
            }
            if self.loose_ids(actor.id, Material::Cord).is_empty() {
                continue;
            }
            let who = if actor.id == viewer {
                "you"
            } else {
                actor.name.as_str()
            };
            lines.push(format!(
                "Hide strips are carried by {who}, {}. They are cordage.",
                self.cover_phrase(actor.cell)
            ));
        }
        if lines.is_empty() {
            "You see no hide strips from here.".into()
        } else {
            lines.join("\n")
        }
    }

    fn examine_spear(&self, viewer: ActorId) -> String {
        let origin = self.actor(viewer).unwrap().cell;
        let mut lines = Vec::new();
        for cell in self.visible_cell_ids(viewer) {
            for id in self.ground_ids(cell, Material::Spear) {
                let spear = self.item(id);
                if let Some(item) = spear.filter(|item| item.flint_tip) {
                    let comma = if item.hardened { "" } else { "," };
                    lines.push(format!(
                        "A spear with {}{comma} lies in contact with the ground {}.",
                        self.flint_tip_words(item),
                        self.place_phrase(origin, cell)
                    ));
                    continue;
                }
                if spear.is_some_and(|item| item.wooden_tip) {
                    let point = if spear.is_some_and(|item| item.hardened) {
                        "The point is fire-hardened."
                    } else {
                        "The point is still green."
                    };
                    lines.push(format!(
                        "A wooden spear lies in contact with the ground {}. {point}",
                        self.place_phrase(origin, cell)
                    ));
                    continue;
                }
                let hardened = spear.is_some_and(|item| item.hardened);
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
                let spear = self.item(id);
                if let Some(item) = spear.filter(|item| item.flint_tip) {
                    let waiting = !item.hardened && item.firing_ready_at.is_some();
                    let comma = if item.hardened { "" } else { "," };
                    let mut line = format!(
                        "A spear with {}{comma} is carried by {who}, {}.",
                        self.flint_tip_words(item),
                        self.cover_phrase(cell)
                    );
                    if waiting {
                        line.push_str(" The flint tip is hardening in the fire.");
                    }
                    lines.push(line);
                    continue;
                }
                if spear.is_some_and(|item| item.wooden_tip) {
                    let hardened = spear.is_some_and(|item| item.hardened);
                    let point = if hardened {
                        "a fire-hardened point"
                    } else {
                        "a green point"
                    };
                    let mut line = format!(
                        "A wooden spear with {point} is carried by {who}, {}.",
                        self.cover_phrase(cell)
                    );
                    if !hardened && self.wooden_point_is_hardening(other) {
                        line.push_str(" The wooden point is hardening in the fire.");
                    }
                    lines.push(line);
                    continue;
                }
                let hardened = spear.is_some_and(|item| item.hardened);
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

    fn examine_flint(&self, viewer: ActorId) -> String {
        let origin = self.actor(viewer).unwrap().cell;
        let mut lines = Vec::new();
        for cell in self.visible_cell_ids(viewer) {
            let on_ground = self.pile_count(cell, Material::Flint)
                + self.ground_ids(cell, Material::Flint).len() as u32;
            if on_ground > 0 {
                let region = self.cell(cell).region.word();
                let place = if region == "slope" {
                    if origin == cell {
                        "here, on the slope".to_string()
                    } else {
                        format!("on the slope to the {}", self.relative(origin, cell))
                    }
                } else {
                    self.place_phrase(origin, cell)
                };
                lines.push(format!(
                    "A flint nodule lies {place}. It is a sharp stone. It can be knapped."
                ));
            }
        }
        for actor in self.actors.values() {
            if chebyshev(origin, actor.cell) > self.tuning.sight {
                continue;
            }
            if self.loose_ids(actor.id, Material::Flint).is_empty() {
                continue;
            }
            let who = if actor.id == viewer {
                "you"
            } else {
                actor.name.as_str()
            };
            lines.push(format!(
                "A flint nodule is carried by {who}, {}. It is a sharp stone. It can be knapped.",
                self.cover_phrase(actor.cell)
            ));
        }
        if lines.is_empty() {
            for cell in self.visible_cell_ids(viewer) {
                if !self.flint_bare(cell) {
                    continue;
                }
                if origin == cell {
                    lines.push("The slope here has no flint.".into());
                } else {
                    lines.push(format!(
                        "The slope to the {} has no flint.",
                        self.relative(origin, cell)
                    ));
                }
            }
        }
        lines.extend(self.flake_lines(viewer));
        if lines.is_empty() {
            "You see no flint from here.".into()
        } else {
            lines.join("\n")
        }
    }

    fn examine_ochre(&self, viewer: ActorId) -> String {
        let origin = self.actor(viewer).unwrap().cell;
        let mut lines = Vec::new();
        for cell in self.visible_cell_ids(viewer) {
            let on_ground = self.pile_count(cell, Material::Ochre)
                + self.ground_ids(cell, Material::Ochre).len() as u32;
            if on_ground > 0 {
                if self.cell(cell).rock {
                    if origin == cell {
                        lines.push("A lump of ochre lies here, on the rock face.".into());
                    } else {
                        lines.push(format!(
                            "A lump of ochre lies on the rock face to the {}.",
                            self.relative(origin, cell)
                        ));
                    }
                } else {
                    let region = self.cell(cell).region.word();
                    let place = if origin == cell {
                        format!("here, on the {region}")
                    } else {
                        format!("on the {region} to the {}", self.relative(origin, cell))
                    };
                    lines.push(format!("A lump of ochre lies {place}."));
                }
            } else if self.ochre_bare(cell) {
                if origin == cell {
                    lines.push("The slope here has no ochre.".into());
                } else {
                    lines.push(format!(
                        "The slope to the {} has no ochre.",
                        self.relative(origin, cell)
                    ));
                }
            }
        }
        for actor in self.actors.values() {
            if chebyshev(origin, actor.cell) > self.tuning.sight {
                continue;
            }
            let count = self.loose_ids(actor.id, Material::Ochre).len() as u32;
            if count == 0 {
                continue;
            }
            let who = if actor.id == viewer {
                "you"
            } else {
                actor.name.as_str()
            };
            let cover = self.cover_phrase(actor.cell);
            if count == 1 {
                lines.push(format!("A lump of ochre is carried by {who}, {cover}."));
            } else {
                lines.push(format!(
                    "{} lumps of ochre are carried by {who}, {cover}.",
                    capitalize(&count_word(count))
                ));
            }
        }
        if lines.is_empty() {
            "You see no ochre from here.".into()
        } else {
            lines.join("\n")
        }
    }

    fn examine_iron_ore(&self, viewer: ActorId) -> String {
        let origin = self.actor(viewer).unwrap().cell;
        let mut lines = Vec::new();
        for cell in self.visible_cell_ids(viewer) {
            let smelting = self.iron_smelting_count(cell);
            let lying = self.pile_count(cell, Material::IronOre)
                + self.ground_ids(cell, Material::IronOre).len() as u32
                - smelting as u32;
            if lying > 0 {
                let region = self.cell(cell).region.word();
                let place = if origin == cell {
                    format!("here, on the {region}")
                } else {
                    format!("on the {region} to the {}", self.relative(origin, cell))
                };
                lines.push(format!("A lump of iron ore lies {place}."));
            } else if self.iron_bare(cell) {
                if origin == cell {
                    lines.push("The slope here has no iron ore.".into());
                } else {
                    lines.push(format!(
                        "The slope to the {} has no iron ore.",
                        self.relative(origin, cell)
                    ));
                }
            }
            if smelting > 0 {
                let fire = self.iron_smelt_fire(cell);
                if origin == cell {
                    lines.push(format!("A lump of iron ore is smelting in {fire}."));
                } else {
                    lines.push(format!(
                        "A lump of iron ore is smelting in {fire} {}.",
                        self.place_phrase(origin, cell)
                    ));
                }
            }
        }
        for actor in self.actors.values() {
            if chebyshev(origin, actor.cell) > self.tuning.sight {
                continue;
            }
            let count = self.loose_ids(actor.id, Material::IronOre).len() as u32;
            if count == 0 {
                continue;
            }
            let who = if actor.id == viewer {
                "you"
            } else {
                actor.name.as_str()
            };
            let cover = self.cover_phrase(actor.cell);
            if count == 1 {
                lines.push(format!("A lump of iron ore is carried by {who}, {cover}."));
            } else {
                lines.push(format!(
                    "{} lumps of iron ore are carried by {who}, {cover}.",
                    capitalize(&count_word(count))
                ));
            }
        }
        if lines.is_empty() {
            "You see no iron ore from here.".into()
        } else {
            lines.join("\n")
        }
    }

    fn examine_iron(&self, viewer: ActorId) -> String {
        let origin = self.actor(viewer).unwrap().cell;
        let mut lines = Vec::new();
        for cell in self.visible_cell_ids(viewer) {
            if self.ground_ids(cell, Material::Iron).is_empty() {
                continue;
            }
            let place = if origin == cell {
                "here".to_string()
            } else {
                format!("on the ground to the {}", self.relative(origin, cell))
            };
            lines.push(format!(
                "A bloom of iron lies {place}. It is metal from a hotter fire."
            ));
        }
        for actor in self.actors.values() {
            if chebyshev(origin, actor.cell) > self.tuning.sight {
                continue;
            }
            if self.loose_ids(actor.id, Material::Iron).is_empty() {
                continue;
            }
            let who = if actor.id == viewer {
                "you"
            } else {
                actor.name.as_str()
            };
            lines.push(format!(
                "A bloom of iron is carried by {who}, {}. It is metal from a hotter fire.",
                self.cover_phrase(actor.cell)
            ));
        }
        if lines.is_empty() {
            "You see no bloom of iron from here.".into()
        } else {
            lines.join("\n")
        }
    }

    fn examine_knife(&self, viewer: ActorId) -> String {
        let origin = self.actor(viewer).unwrap().cell;
        let mut lines = Vec::new();
        for cell in self.visible_cell_ids(viewer) {
            if self.ground_ids(cell, Material::Knife).is_empty() {
                continue;
            }
            let place = if origin == cell {
                "here".to_string()
            } else {
                format!("on the ground to the {}", self.relative(origin, cell))
            };
            lines.push(format!("An iron knife lies {place}. It can cut."));
        }
        for actor in self.actors.values() {
            if chebyshev(origin, actor.cell) > self.tuning.sight {
                continue;
            }
            if self.loose_ids(actor.id, Material::Knife).is_empty() {
                continue;
            }
            let who = if actor.id == viewer {
                "you"
            } else {
                actor.name.as_str()
            };
            lines.push(format!(
                "An iron knife is carried by {who}, {}. It can cut.",
                self.cover_phrase(actor.cell)
            ));
        }
        if lines.is_empty() {
            "You see no iron knife from here.".into()
        } else {
            lines.join("\n")
        }
    }

    fn examine_copper_ore(&self, viewer: ActorId) -> String {
        let origin = self.actor(viewer).unwrap().cell;
        let mut lines = Vec::new();
        for cell in self.visible_cell_ids(viewer) {
            let smelting = self.copper_smelting_count(cell);
            let lying = self.pile_count(cell, Material::CopperOre)
                + self.ground_ids(cell, Material::CopperOre).len() as u32
                - smelting as u32;
            if lying > 0 {
                let region = self.cell(cell).region.word();
                let place = if origin == cell {
                    format!("here, on the {region}")
                } else {
                    format!("on the {region} to the {}", self.relative(origin, cell))
                };
                lines.push(format!("A lump of copper ore lies {place}."));
            } else if self.copper_bare(cell) {
                if origin == cell {
                    lines.push("The brook here has no copper ore.".into());
                } else {
                    lines.push(format!(
                        "The brook to the {} has no copper ore.",
                        self.relative(origin, cell)
                    ));
                }
            }
            if smelting > 0 {
                let fire = self.copper_smelt_fire(cell);
                if origin == cell {
                    lines.push(format!("A lump of copper ore is smelting in {fire}."));
                } else {
                    lines.push(format!(
                        "A lump of copper ore is smelting in {fire} {}.",
                        self.place_phrase(origin, cell)
                    ));
                }
            }
        }
        for actor in self.actors.values() {
            if chebyshev(origin, actor.cell) > self.tuning.sight {
                continue;
            }
            let count = self.loose_ids(actor.id, Material::CopperOre).len() as u32;
            if count == 0 {
                continue;
            }
            let who = if actor.id == viewer {
                "you"
            } else {
                actor.name.as_str()
            };
            let cover = self.cover_phrase(actor.cell);
            if count == 1 {
                lines.push(format!(
                    "A lump of copper ore is carried by {who}, {cover}."
                ));
            } else {
                lines.push(format!(
                    "{} lumps of copper ore are carried by {who}, {cover}.",
                    capitalize(&count_word(count))
                ));
            }
        }
        if lines.is_empty() {
            "You see no copper ore from here.".into()
        } else {
            lines.join("\n")
        }
    }

    fn examine_copper(&self, viewer: ActorId) -> String {
        let origin = self.actor(viewer).unwrap().cell;
        let mut lines = Vec::new();
        for cell in self.visible_cell_ids(viewer) {
            if self.ground_ids(cell, Material::Copper).is_empty() {
                continue;
            }
            let place = if origin == cell {
                "here".to_string()
            } else {
                format!("on the ground to the {}", self.relative(origin, cell))
            };
            lines.push(format!(
                "A copper bead lies {place}. It is metal from the fire."
            ));
        }
        for actor in self.actors.values() {
            if chebyshev(origin, actor.cell) > self.tuning.sight {
                continue;
            }
            if self.loose_ids(actor.id, Material::Copper).is_empty() {
                continue;
            }
            let who = if actor.id == viewer {
                "you"
            } else {
                actor.name.as_str()
            };
            lines.push(format!(
                "A copper bead is carried by {who}, {}. It is metal from the fire.",
                self.cover_phrase(actor.cell)
            ));
        }
        if lines.is_empty() {
            "You see no copper bead from here.".into()
        } else {
            lines.join("\n")
        }
    }

    fn examine_awl(&self, viewer: ActorId) -> String {
        let origin = self.actor(viewer).unwrap().cell;
        let mut lines = Vec::new();
        for cell in self.visible_cell_ids(viewer) {
            if self.ground_ids(cell, Material::Awl).is_empty() {
                continue;
            }
            let place = if origin == cell {
                "here".to_string()
            } else {
                format!("on the ground to the {}", self.relative(origin, cell))
            };
            for id in self.ground_ids(cell, Material::Awl) {
                let kind = if self.item(id).is_some_and(|awl| awl.bone) {
                    "bone"
                } else {
                    "copper"
                };
                lines.push(format!("A {kind} awl lies {place}. It can pierce hide."));
            }
        }
        for actor in self.actors.values() {
            if chebyshev(origin, actor.cell) > self.tuning.sight {
                continue;
            }
            for id in self.loose_ids(actor.id, Material::Awl) {
                let kind = if self.item(id).is_some_and(|awl| awl.bone) {
                    "bone"
                } else {
                    "copper"
                };
                let who = if actor.id == viewer {
                    "you"
                } else {
                    actor.name.as_str()
                };
                lines.push(format!(
                    "A {kind} awl is carried by {who}, {}. It can pierce hide.",
                    self.cover_phrase(actor.cell)
                ));
            }
        }
        if lines.is_empty() {
            "You see no copper awl from here.".into()
        } else {
            lines.join("\n")
        }
    }

    fn examine_flake(&self, viewer: ActorId) -> String {
        let lines = self.flake_lines(viewer);
        if lines.is_empty() {
            "You see no flint flake from here.".into()
        } else {
            lines.join("\n")
        }
    }

    fn flake_lines(&self, viewer: ActorId) -> Vec<String> {
        let origin = self.actor(viewer).unwrap().cell;
        let mut lines = Vec::new();
        for actor in self.actors.values() {
            if chebyshev(origin, actor.cell) > self.tuning.sight {
                continue;
            }
            let count = self.loose_ids(actor.id, Material::Flake).len() as u32;
            if count == 0 {
                continue;
            }
            let who = if actor.id == viewer {
                "you"
            } else {
                actor.name.as_str()
            };
            let cover = self.cover_phrase(actor.cell);
            if count == 1 {
                lines.push(format!(
                    "A sharp flint flake is carried by {who}, {cover}. It is sharp enough to cut or haft later."
                ));
            } else {
                lines.push(format!(
                    "{} sharp flint flakes are carried by {who}, {cover}. They are sharp enough to cut or haft later.",
                    capitalize(&count_word(count))
                ));
            }
        }
        lines
    }

    pub(crate) fn bitter_known(&self, viewer: ActorId) -> bool {
        self.actor(viewer).is_some_and(|actor| actor.tasted_bitter)
    }

    /// The patch's name for this person. Taste is what turns it to poison.
    pub(crate) fn bitter_kind(&self, viewer: ActorId) -> &'static str {
        if self.bitter_known(viewer) {
            "poison berries"
        } else {
            "dark berries"
        }
    }

    fn bitter_kind_cap(&self, viewer: ActorId) -> &'static str {
        if self.bitter_known(viewer) {
            "Poison berries"
        } else {
            "Dark berries"
        }
    }

    fn food_kind_for(&self, viewer: ActorId, material: Material) -> (&'static str, &'static str) {
        if material == Material::BitterBerry {
            (self.bitter_kind(viewer), "are")
        } else {
            food_kind(material)
        }
    }

    fn pile_noun_for(&self, viewer: ActorId, material: Material, count: u32) -> String {
        if material == Material::BitterBerry {
            let kind = self.bitter_kind(viewer);
            if count == 1 {
                format!("handful of {kind}")
            } else {
                format!("handfuls of {kind}")
            }
        } else {
            pile_noun(material, count).into()
        }
    }

    fn carried_berry_phrase(
        &self,
        viewer: ActorId,
        material: Material,
        count: u32,
        condition: &str,
    ) -> String {
        if material == Material::BitterBerry {
            let kind = self.bitter_kind(viewer);
            if count == 1 {
                format!("A {condition} handful of {kind}")
            } else {
                format!(
                    "{} {condition} handfuls of {kind}",
                    capitalize(&count_word(count))
                )
            }
        } else {
            carried_food_phrase(material, count, condition)
        }
    }

    pub(crate) fn raw_mushroom_known(&self, viewer: ActorId) -> bool {
        self.actor(viewer)
            .is_some_and(|actor| actor.raw_mushroom_known)
    }

    pub(crate) fn pale_berry_known(&self, viewer: ActorId) -> bool {
        self.actor(viewer)
            .is_some_and(|actor| actor.pale_berry_known)
    }

    pub(crate) fn bark_known(&self, viewer: ActorId) -> bool {
        self.actor(viewer).is_some_and(|actor| actor.bark_known)
    }

    /// Bitter berries keep their answer until this person has tasted them.
    /// A raw mushroom keeps its warning until this person has tried to eat one.
    fn food_clause_for(&self, viewer: ActorId, material: Material) -> &'static str {
        if material == Material::BitterBerry && self.bitter_known(viewer) {
            " They taste bitter. They are poisonous."
        } else if material == Material::Mushroom && self.raw_mushroom_known(viewer) {
            " They must be cooked. Raw mushrooms do not feed you."
        } else if material == Material::PaleBerry && self.pale_berry_known(viewer) {
            " They must be cooked."
        } else {
            food_clause(material)
        }
    }

    fn examine_material(&self, viewer: ActorId, material: Material) -> String {
        let origin = self.actor(viewer).unwrap().cell;
        let mut lines = Vec::new();
        for cell in self.visible_cell_ids(viewer) {
            let count = self.pile_count(cell, material);
            if count == 0 && self.berry_patch_bare(cell, material) {
                let way = if cell == origin {
                    "here".to_string()
                } else {
                    format!("to the {}", self.relative(origin, cell))
                };
                if material == Material::BitterBerry {
                    lines.push(format!(
                        "The {} that were {way} are gone.",
                        self.bitter_kind(viewer)
                    ));
                } else if !self.deer_ate_berries.contains(&cell) {
                    lines.push(format!("The berries that were {way} are gone."));
                }
            } else if count == 0 && material == Material::Mushroom && self.mushroom_patch_bare(cell)
            {
                let way = if cell == origin {
                    "here".to_string()
                } else {
                    format!("to the {}", self.relative(origin, cell))
                };
                lines.push(format!("The mushrooms that were {way} are gone."));
            } else if count == 0 && material == Material::Bark && self.bark_bare(cell) {
                if cell == origin {
                    lines.push("There is no bark under the rock.".into());
                } else {
                    lines.push(format!(
                        "There is no bark under the rock to the {}.",
                        self.relative(origin, cell)
                    ));
                }
            } else if count > 0 {
                if material == Material::Stone && count == 1 {
                    lines.push(format!(
                        "A stone lies in contact with the ground {}. The stone is {}. {}{}",
                        self.place_phrase(origin, cell),
                        self.condition_word(self.pile_moisture(cell, material)),
                        self.rain_reaches_phrase(cell),
                        self.food_clause_for(viewer, material)
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
                    let (kind, verb) = self.food_kind_for(viewer, material);
                    let head = if count == 1 {
                        format!(
                            "A handful of berries is in contact with the ground {}",
                            self.place_phrase(origin, cell)
                        )
                    } else {
                        format!(
                            "A pile of {count} {} is in contact with the ground {}",
                            self.pile_noun_for(viewer, material, count),
                            self.place_phrase(origin, cell)
                        )
                    };
                    lines.push(format!(
                        "{head}.{eating} The {kind} {verb} {}. {}{}{}{}",
                        self.condition_word(self.pile_moisture(cell, material)),
                        self.rain_reaches_phrase(cell),
                        self.heavier_rain_clause(cell, material),
                        self.rotting_clause(cell, material),
                        self.food_clause_for(viewer, material)
                    ));
                } else if material == Material::Branch {
                    let noun = self.pile_noun_for(viewer, material, count);
                    let (kind, verb) = self.food_kind_for(viewer, material);
                    lines.push(format!(
                        "A pile of {count} {noun} is in contact with the ground {}{}. The {kind} {verb} {}. {}{}{}{}{}",
                        self.mark_place(origin, cell),
                        self.hoofprints_on_wood(origin, cell),
                        self.condition_word(self.pile_moisture(cell, material)),
                        self.rain_reaches_phrase(cell),
                        self.heavier_rain_clause(cell, material),
                        self.rotting_clause(cell, material),
                        self.food_clause_for(viewer, material),
                        self.deer_on_this_wood(origin, cell)
                    ));
                } else {
                    let noun = self.pile_noun_for(viewer, material, count);
                    let (kind, verb) = self.food_kind_for(viewer, material);
                    lines.push(format!(
                        "A pile of {count} {noun} is in contact with the ground {}. The {kind} {verb} {}. {}{}{}{}",
                        self.place_phrase(origin, cell),
                        self.condition_word(self.pile_moisture(cell, material)),
                        self.rain_reaches_phrase(cell),
                        self.heavier_rain_clause(cell, material),
                        self.rotting_clause(cell, material),
                        self.food_clause_for(viewer, material)
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
                        self.food_clause_for(viewer, material)
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
                } else if material == Material::Berry && loose.len() == 1 {
                    let ate = if self.deer_ate_berries.contains(&cell) {
                        " A deer ate the rest."
                    } else {
                        ""
                    };
                    lines.push(format!(
                        "A handful of berries lies in contact with the ground {}. It is {}. {}{}{ate}",
                        self.place_phrase(origin, cell),
                        self.condition_word(worst),
                        self.rain_reaches_phrase(cell),
                        self.food_clause_for(viewer, material)
                    ));
                } else if material == Material::Branch {
                    let drying = self.ground_ids(cell, Material::Branch).iter().any(|id| {
                        self.item(*id)
                            .is_some_and(|item| item.firing_ready_at.is_some())
                    });
                    let wait = if drying {
                        " It is drying by the fire. It is not dry yet."
                    } else {
                        ""
                    };
                    lines.push(format!(
                        "{} loose {} {} in contact with the ground {}{}. {}{}{}{}{}{wait}",
                        loose.len(),
                        self.pile_noun_for(viewer, material, loose.len() as u32),
                        if loose.len() == 1 { "is" } else { "are" },
                        self.mark_place(origin, cell),
                        self.hoofprints_on_wood(origin, cell),
                        self.condition_sentence(worst, cell),
                        self.heavier_rain_clause(cell, material),
                        self.rotting_clause(cell, material),
                        self.food_clause_for(viewer, material),
                        self.deer_on_this_wood(origin, cell)
                    ));
                } else {
                    lines.push(format!(
                        "{} loose {} {} in contact with the ground {}. {}{}{}{}",
                        loose.len(),
                        self.pile_noun_for(viewer, material, loose.len() as u32),
                        if loose.len() == 1 { "is" } else { "are" },
                        self.place_phrase(origin, cell),
                        self.condition_sentence(worst, cell),
                        self.heavier_rain_clause(cell, material),
                        self.rotting_clause(cell, material),
                        self.food_clause_for(viewer, material)
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
            } else if material == Material::Bark {
                let word = self.condition_word(moisture);
                if ids.len() == 1 {
                    lines.push(format!(
                        "A {word} sheet of bark is carried by {who}, {}.",
                        self.cover_phrase(cell)
                    ));
                } else {
                    lines.push(format!(
                        "{} {word} sheets of bark are carried by {who}, {}.",
                        capitalize(&count_word(ids.len() as u32)),
                        self.cover_phrase(cell)
                    ));
                }
            } else if material == Material::Pot && ids.len() == 1 {
                lines.push(format!(
                    "A {} pot is carried by {who}, {}, and it can boil water over a fire.",
                    self.condition_word(moisture),
                    self.cover_phrase(cell)
                ));
            } else if material == Material::Mushroom {
                let cooked = ids
                    .iter()
                    .filter(|id| self.item(**id).is_some_and(|mushroom| mushroom.cooked))
                    .count();
                if cooked == ids.len() {
                    let phrase = if cooked == 1 {
                        "A cooked mushroom is".to_string()
                    } else {
                        format!("{} cooked mushrooms are", count_word(cooked as u32))
                    };
                    lines.push(format!(
                        "{phrase} carried by {who}, {}. {}",
                        self.cover_phrase(cell),
                        if cooked == 1 {
                            "It is good to eat."
                        } else {
                            "They are good to eat."
                        }
                    ));
                } else {
                    let phrase = carried_food_phrase(
                        material,
                        ids.len() as u32,
                        self.condition_word(moisture),
                    );
                    lines.push(format!(
                        "{phrase} {} carried by {who}, {}.{}",
                        if ids.len() == 1 { "is" } else { "are" },
                        self.cover_phrase(cell),
                        self.food_clause_for(viewer, material)
                    ));
                }
            } else if material == Material::Branch {
                let word = self.condition_word(moisture);
                let drying = ids.iter().any(|id| {
                    self.item(*id)
                        .is_some_and(|item| item.firing_ready_at.is_some())
                });
                let wait = if drying {
                    " It is drying by the fire. It is not dry yet."
                } else {
                    ""
                };
                if ids.len() == 1 {
                    lines.push(format!(
                        "A {word} branch is carried by {who}, {}.{wait}",
                        self.cover_phrase(cell)
                    ));
                } else {
                    lines.push(format!(
                        "{} {word} branches are carried by {who}, {}.{wait}",
                        count_word(ids.len() as u32),
                        self.cover_phrase(cell)
                    ));
                }
            } else if matches!(material, Material::Berry | Material::BitterBerry) {
                let phrase = self.carried_berry_phrase(
                    viewer,
                    material,
                    ids.len() as u32,
                    self.condition_word(moisture),
                );
                lines.push(format!(
                    "{phrase} {} carried by {who}, {}.{}",
                    if ids.len() == 1 { "is" } else { "are" },
                    self.cover_phrase(cell),
                    self.food_clause_for(viewer, material)
                ));
            } else {
                lines.push(format!(
                    "{} {} {} carried by {who}, {}. {}{}",
                    ids.len(),
                    self.pile_noun_for(viewer, material, ids.len() as u32),
                    if ids.len() == 1 { "is" } else { "are" },
                    self.cover_phrase(cell),
                    self.condition_word(moisture),
                    self.food_clause_for(viewer, material)
                ));
            }
        }
        if material == Material::Bark && self.bark_known(viewer) && !lines.is_empty() {
            lines.push("Bark does not feed you.".into());
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
                Material::BitterBerry => self.bitter_kind(viewer),
                Material::Mud => "mud",
                Material::Mushroom => "mushrooms",
                Material::Pot => "pot",
                Material::Stone => "stone",
                Material::Flint => "flint",
                Material::Flake => "flint flake",
                Material::Meat => "meat",
                Material::Hide => "hide",
                Material::Cord => "hide strips",
                Material::Fish => "fish",
                Material::Brand => "brand",
                Material::Ochre => "ochre",
                Material::CopperOre => "copper ore",
                Material::IronOre => "iron ore",
                Material::Iron => "bloom of iron",
                Material::Knife => "iron knife",
                Material::Copper => "copper bead",
                Material::Awl => "awl",
                Material::Bone => "bone",
                Material::Salt => "salt",
                Material::Honey => "honey",
                Material::Egg => "eggs",
                Material::Moss => "moss",
                Material::Mint => "mint",
                Material::Needle => "pine needles",
                Material::Drill => "bow drill",
                Material::Resin => "resin",
                Material::Willow => "willow",
                Material::Nettle => "nettles",
                Material::Cattail => "cattail roots",
                Material::BarkCup => "bark cup",
                Material::Flour => "cattail flour",
                Material::Bow => "bow",
                Material::Clay => "clay",
                Material::Bowl => "bowl",
                Material::Spear => "spear",
                Material::Carcass => "carcass",
                Material::Water => "water",
                Material::Basket => "bark basket",
                Material::Charcoal => "charcoal",
                Material::Reed => "reeds",
                Material::Net => "fishing net",
                Material::Mat => "reed mat",
                Material::Canoe => "dugout canoe",
                Material::PaleBerry => "pale berries",
                Material::Paddle => "wooden paddle",
                Material::Hook => "bone fish hook",
                Material::WillowBark => "willow bark",
                Material::Fluff => "cattail fluff",
                Material::Worm => "worm",
                Material::Quartz => "quartz",
                Material::Whistle => "bone whistle",
                Material::Flute => "reed flute",
                Material::Drum => "drum",
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
            let charred = if self.charred_items.contains(&item.id) {
                " The fire has charred it. It can be carved into a canoe."
            } else {
                ""
            };
            return format!(
                "The heavy log is in contact with the ground {}. It is {}. {held} {}{charred}",
                self.place_phrase(origin, item.cell),
                self.condition_word(item.moisture),
                self.rain_reaches_phrase(item.cell)
            );
        }
        for cell in self.visible_cell_ids(viewer) {
            if self.pile_count(cell, Material::Log) > 0 {
                let charred = if self.charred_logs.contains(&cell) {
                    " The fire has charred it. It can be carved into a canoe."
                } else {
                    ""
                };
                return format!(
                    "A heavy log lies in contact with the ground {}. It has not been lifted. It is {}. {}{charred}",
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
            let reach = if chebyshev(origin, fire.cell) == 1 {
                " The heat does not reach you from here."
            } else {
                ""
            };
            let kind = if fire.charcoal {
                "A hot charcoal fire burns"
            } else if fire.hot {
                "A fire burns hotter"
            } else {
                "A small fire burns"
            };
            let cover = if self.banks.contains_key(&fire.cell) {
                " Branches are banked on it under a cover of mud."
            } else {
                ""
            };
            lines.push(format!(
                "{kind} {}. It was lit on this ground. It warms anyone standing at it.{reach}{cover}",
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
            } else if place.still && self.still_frozen(cell) && self.ice_hole(cell) {
                lines.push(format!(
                    "Still water {} is frozen. A hole is open in the ice.",
                    self.place_phrase(origin, cell)
                ));
            } else if place.still && self.still_frozen(cell) {
                lines.push(format!(
                    "Still water {} is frozen. Ice covers it. A fire on it would keep it open.",
                    self.place_phrase(origin, cell)
                ));
            } else if place.still && self.still_would_freeze(cell) {
                lines.push(format!(
                    "Still water {} is open because a fire is on it. Drinking it turns your stomach almost every time.",
                    self.place_phrase(origin, cell)
                ));
            } else if place.still && self.time >= self.tuning.chill_at {
                lines.push(format!(
                    "Still water {} has thawed. The day warmed it. Drinking it turns your stomach almost every time.",
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

    fn examine_hare(&self, viewer: ActorId) -> String {
        self.seen_hare(viewer)
            .unwrap_or_else(|| "You cannot see a hare from here.".into())
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
                        || event.text == "A deer eats the last handful of berries."
                        || event.text
                            == "A deer eats from the pile and leaves a handful of berries.")
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
            if event.text == "A deer eats the last handful of berries."
                || event.text == "A deer eats from the pile and leaves a handful of berries."
            {
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
        if self.leftover_handful(cell) {
            return if origin == cell {
                " A deer ate from the pile. A handful of berries is left."
            } else {
                " A deer ate from the pile there. A handful of berries is left."
            };
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

    fn branch_moisture_here(&self, cell: CellId) -> u32 {
        let pile = if self.pile_count(cell, Material::Branch) > 0 {
            self.pile_moisture(cell, Material::Branch)
        } else {
            0
        };
        let loose = self
            .ground_ids(cell, Material::Branch)
            .iter()
            .filter_map(|id| self.item(*id).map(|item| item.moisture))
            .max()
            .unwrap_or(0);
        pile.max(loose)
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

    /// `* fire` stays in the key only when a fire cell is inside sight,
    /// so the star is on the map. Smoke past sight does not add it.
    fn legend_for(&self, viewer: ActorId) -> String {
        if self.sees_a_fire_cell(viewer) {
            return LEGEND.to_string();
        }
        LEGEND
            .lines()
            .map(|line| match line.trim_end().strip_suffix("* fire") {
                Some(kept) => kept.trim_end().to_string(),
                None => line.to_string(),
            })
            .collect::<Vec<_>>()
            .join("\n")
    }

    /// A fire on a cell this person can see. The map draws the star there.
    fn sees_a_fire_cell(&self, viewer: ActorId) -> bool {
        let Some(origin) = self.actor(viewer).map(|actor| actor.cell) else {
            return false;
        };
        self.fires
            .values()
            .any(|fire| chebyshev(origin, fire.cell) <= self.tuning.sight)
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
        if self.fox.as_ref().is_some_and(|fox| fox.cell == cell) {
            marks.push('x');
        }
        if self.hare.as_ref().is_some_and(|hare| hare.cell == cell) {
            marks.push('r');
        }
        if self
            .beaver
            .as_ref()
            .is_some_and(|beaver| beaver.cell == cell)
        {
            marks.push('z');
        }
        if self.otter.as_ref().is_some_and(|otter| otter.cell == cell) {
            marks.push('t');
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
        if self.pile_count(cell, Material::PaleBerry) > 0
            || self.worst_loose(cell, Material::PaleBerry).is_some()
        {
            marks.push('h');
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
        if self.pile_count(cell, Material::Flint) > 0
            || self.worst_loose(cell, Material::Flint).is_some()
        {
            marks.push('n');
        }
        if self.pile_count(cell, Material::Ochre) > 0
            || self.worst_loose(cell, Material::Ochre).is_some()
        {
            marks.push('o');
        }
        if self.pile_count(cell, Material::IronOre) > 0
            || self.worst_loose(cell, Material::IronOre).is_some()
        {
            marks.push('g');
        }
        if self.pile_count(cell, Material::Iron) > 0
            || self.worst_loose(cell, Material::Iron).is_some()
        {
            marks.push('I');
        }
        if self.pile_count(cell, Material::Knife) > 0
            || self.worst_loose(cell, Material::Knife).is_some()
        {
            marks.push('K');
        }
        if self.pile_count(cell, Material::CopperOre) > 0
            || self.worst_loose(cell, Material::CopperOre).is_some()
        {
            marks.push('C');
        }
        if self.pile_count(cell, Material::Copper) > 0
            || self.worst_loose(cell, Material::Copper).is_some()
        {
            marks.push('e');
        }
        if self.pile_count(cell, Material::Awl) > 0
            || self.worst_loose(cell, Material::Awl).is_some()
        {
            marks.push('i');
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
        for item in self.items.values().filter(|item| {
            item.carrier == Some(actor_id) && item.role == Role::Loose && !item.in_basket
        }) {
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
            .filter(|event| {
                self.event_visible(viewer, origin, event)
                    && self.shape_for_these_eyes(viewer, event).is_none()
                    && self.gather_for_these_eyes(viewer, event).is_none()
                    && self.drop_for_these_eyes(viewer, event).is_none()
                    && self.fold_for_these_eyes(viewer, event).is_none()
                    && self.grind_for_these_eyes(viewer, event).is_none()
                    && self.seal_cup_for_these_eyes(viewer, event).is_none()
                    && self.smoke_for_these_eyes(viewer, event).is_none()
                    && self.walk_for_these_eyes(viewer, event).is_none()
                    && self.smear_for_these_eyes(viewer, event).is_none()
                    && self.mark_for_these_eyes(viewer, event).is_none()
                    && self.cut_for_these_eyes(viewer, event).is_none()
                    && self.snare_for_these_eyes(viewer, event).is_none()
                    && self.set_net_for_these_eyes(viewer, event).is_none()
                    && self.weir_for_these_eyes(viewer, event).is_none()
                    && self.carve_for_these_eyes(viewer, event).is_none()
                    && self.blow_for_these_eyes(viewer, event).is_none()
                    && self.drum_for_these_eyes(viewer, event).is_none()
                    && self.bone_pierce_for_these_eyes(viewer, event).is_none()
                    && self.scrape_for_these_eyes(viewer, event).is_none()
                    && self.salt_for_these_eyes(viewer, event).is_none()
                    && self.press_for_these_eyes(viewer, event).is_none()
                    && self.craft_for_these_eyes(viewer, event).is_none()
                    && self.clothes_for_these_eyes(viewer, event).is_none()
                    && self.bow_hunt_for_these_eyes(viewer, event).is_none()
                    && self.scare_for_these_eyes(viewer, event).is_none()
                    && self.wolf_hunt_for_these_eyes(viewer, event).is_none()
                    && self.otter_hunt_for_these_eyes(viewer, event).is_none()
                    && self.beaver_hunt_for_these_eyes(viewer, event).is_none()
                    && self.fox_hunt_for_these_eyes(viewer, event).is_none()
                    && self.hare_hunt_for_these_eyes(viewer, event).is_none()
                    && self.otter_butcher_for_these_eyes(viewer, event).is_none()
                    && self.fox_butcher_for_these_eyes(viewer, event).is_none()
                    && self.hare_butcher_for_these_eyes(viewer, event).is_none()
                    && self.beaver_butcher_for_these_eyes(viewer, event).is_none()
                    && self.otter_cloak_for_these_eyes(viewer, event).is_none()
                    && self.fox_cloak_for_these_eyes(viewer, event).is_none()
                    && self.hare_cloak_for_these_eyes(viewer, event).is_none()
                    && self.beaver_cloak_for_these_eyes(viewer, event).is_none()
                    && self.wolf_cloak_for_these_eyes(viewer, event).is_none()
                    && self.weave_net_for_these_eyes(viewer, event).is_none()
                    && self.mat_for_these_eyes(viewer, event).is_none()
                    && self.canoe_for_these_eyes(viewer, event).is_none()
                    && self.light_for_these_eyes(viewer, event).is_none()
                    && self.drink_for_these_eyes(viewer, event).is_none()
                    && self.strike_for_these_eyes(viewer, event).is_none()
                    && self.fire_pot_for_these_eyes(viewer, event).is_none()
                    && self.boil_reply_for_these_eyes(viewer, event).is_none()
                    && self.steep_reply_for_these_eyes(viewer, event).is_none()
                    && self.point_for_these_eyes(viewer, event).is_none()
                    && self.hold_point_for_these_eyes(viewer, event).is_none()
                    && self.wooden_deer_for_these_eyes(viewer, event).is_none()
                    && self.copper_set_for_these_eyes(viewer, event).is_none()
                    && self.copper_awl_for_these_eyes(viewer, event).is_none()
                    && self.smelt_set_for_these_eyes(viewer, event).is_none()
                    && self.canoe_refusal_for_these_eyes(viewer, event).is_none()
                    && self.alone_pull_for_these_eyes(viewer, event).is_none()
                    && self.net_fish_for_these_eyes(viewer, event).is_none()
                    && self.hook_fish_for_these_eyes(viewer, event).is_none()
                    && self.ice_break_for_these_eyes(viewer, event).is_none()
                    && self.haft_for_these_eyes(viewer, event).is_none()
                    && self.bank_for_these_eyes(viewer, event).is_none()
                    && self.thatch_for_these_eyes(viewer, event).is_none()
                    && self.knap_for_these_eyes(viewer, event).is_none()
                    && self.eat_for_these_eyes(viewer, event).is_none()
                    && self.cook_reply_for_these_eyes(viewer, event).is_none()
            })
            .map(|event| EventView {
                id: event.id,
                time: event.time,
                text: self.heard_text(viewer, origin, event),
            })
            .collect();
        if views.len() > 40 {
            views = views.split_off(views.len() - 40);
        }
        views
    }

    /// A fire still burning where the sky does not reach it. Open fires are
    /// already out by the time this rain is read.
    fn fire_the_rain_misses(&self, origin: CellId) -> bool {
        self.fires.values().any(|fire| {
            chebyshev(origin, fire.cell) <= self.tuning.sight
                && self.exposure_millis(fire.cell) < 1000
        })
    }

    /// A deer hide put on, filled for the person who wears it.
    fn wear_for_these_eyes(&self, viewer: ActorId, event: &crate::model::Event) -> Option<String> {
        if event.affected_actors.len() != 1 || event.affected_actors[0] != viewer {
            return None;
        }
        let name = self.name_of(viewer);
        let rest = event.text.strip_prefix(&format!("{name} wears "))?;
        if rest != "the deer hide." && rest != "the sewn deer-hide cloak." {
            return None;
        }
        Some(crate::model::spoken_act(None, "wear", rest))
    }

    /// A deer hide taken off, filled for the person who wore it.
    /// Anyone else keeps the stored sentence.
    fn remove_for_these_eyes(
        &self,
        viewer: ActorId,
        event: &crate::model::Event,
    ) -> Option<String> {
        if event.affected_actors.len() != 1 || event.affected_actors[0] != viewer {
            return None;
        }
        let name = self.name_of(viewer);
        let rest = event.text.strip_prefix(&format!("{name} removes "))?;
        if rest != "the deer hide." && rest != "the sewn deer-hide cloak." {
            return None;
        }
        let mut line = crate::model::spoken_act(None, "remove", rest);
        let open = event
            .affected_cells
            .first()
            .is_some_and(|cell| self.exposure_millis(*cell) >= 1000);
        if open {
            line.push_str(" The wind reaches you again.");
        }
        Some(line)
    }

    /// Moss pressed on a sting. The person already has this as the reply,
    /// so the event list does not repeat it. Anyone else keeps the stored sentence.
    fn press_for_these_eyes(&self, viewer: ActorId, event: &crate::model::Event) -> Option<String> {
        if event.affected_actors.len() != 1 || event.affected_actors[0] != viewer {
            return None;
        }
        let name = self.name_of(viewer);
        if event.text == format!("{name} presses moss on the stings.") {
            Some("You press moss on the stings.".into())
        } else {
            None
        }
    }

    /// Mud or ochre put on the skin. The smearer already has this as the
    /// reply, so the event list does not repeat it. Anyone else keeps the
    /// stored sentence.
    fn smear_for_these_eyes(&self, viewer: ActorId, event: &crate::model::Event) -> Option<String> {
        if event.affected_actors.len() != 1 || event.affected_actors[0] != viewer {
            return None;
        }
        let name = self.name_of(viewer);
        let rest = event.text.strip_prefix(&format!("{name} smears "))?;
        let stuff = rest.strip_suffix(" on their skin.")?;
        if stuff != "mud" && stuff != "red ochre" {
            return None;
        }
        Some(crate::model::spoken_smear(None, stuff))
    }

    /// A mark on the ground or the rock. The marker already has this as the
    /// reply, so the event list does not repeat it. Anyone else keeps the
    /// stored sentence.
    fn mark_for_these_eyes(&self, viewer: ActorId, event: &crate::model::Event) -> Option<String> {
        if event.affected_actors.len() != 1 || event.affected_actors[0] != viewer {
            return None;
        }
        let name = self.name_of(viewer);
        let rest = event.text.strip_prefix(&format!("{name} marks "))?;
        if !matches!(
            rest,
            "the ground with red ochre."
                | "the rock with red ochre."
                | "the ground with charcoal."
                | "the rock with charcoal."
        ) {
            return None;
        }
        Some(crate::model::spoken_act(None, "mark", rest))
    }

    /// Smoking the nest, hidden from the person who smoked it.
    /// Someone who can see the tree still reads the name.
    fn smoke_for_these_eyes(&self, viewer: ActorId, event: &crate::model::Event) -> Option<String> {
        if event.affected_actors.len() != 1 || event.affected_actors[0] != viewer {
            return None;
        }
        let name = self.name_of(viewer);
        let nest = format!("{name} smokes the bees' nest. The bees quiet.");
        let hung = format!("{name} hangs a raw fish by the fire.");
        let set = format!("{name} sets a raw fish near the fire.");
        let fish = format!("{name} smokes a raw fish. It will keep, and it can be eaten cold.");
        let hang_meat = format!("{name} hangs a portion of raw meat by the fire.");
        let hang_piece = format!("{name} hangs a smaller piece of raw meat by the fire.");
        let set_meat = format!("{name} sets a portion of raw meat near the fire.");
        let set_piece = format!("{name} sets a smaller piece of raw meat near the fire.");
        let portion =
            format!("{name} smokes a portion of raw meat. It will keep, and it can be eaten cold.");
        let piece = format!(
            "{name} smokes a smaller piece of raw meat. It will keep, and it can be eaten cold."
        );
        if event.text == nest
            || event.text == hung
            || event.text == set
            || event.text == fish
            || event.text == hang_meat
            || event.text == hang_piece
            || event.text == set_meat
            || event.text == set_piece
            || event.text == portion
            || event.text == piece
        {
            Some(event.text.clone())
        } else {
            None
        }
    }

    /// Bee stings leaving, filled for the person they were on.
    fn stings_for_these_eyes(
        &self,
        viewer: ActorId,
        event: &crate::model::Event,
    ) -> Option<String> {
        if event.affected_actors.len() != 1 || event.affected_actors[0] != viewer {
            return None;
        }
        let name = self.name_of(viewer);
        if event.text == format!("The stings on {name} fade.") {
            Some("The stings fade.".into())
        } else if event.text == format!("The nettle stings on {name} fade.") {
            Some("The nettle stings fade.".into())
        } else {
            None
        }
    }

    /// Rain taking the mud, filled for the person it washed.
    fn wash_for_these_eyes(&self, viewer: ActorId, event: &crate::model::Event) -> Option<String> {
        if event.affected_actors.len() != 1 || event.affected_actors[0] != viewer {
            return None;
        }
        let name = self.name_of(viewer);
        if event.text == format!("The rain washes the mud off {name}.") {
            return Some("The rain washes the mud off you.".into());
        }
        if event.text == format!("The rain washes the red ochre off {name}.") {
            return Some("The rain washes the red ochre off you.".into());
        }
        None
    }

    /// Cooked meat eaten, filled for the person who ate it.
    fn eat_meat_for_these_eyes(
        &self,
        viewer: ActorId,
        event: &crate::model::Event,
    ) -> Option<String> {
        if event.affected_actors.len() != 1 || event.affected_actors[0] != viewer {
            return None;
        }
        let name = self.name_of(viewer);
        let rest = event.text.strip_prefix(&format!("{name} eats "))?;
        if rest != "the cooked meat."
            && rest != "the salted meat."
            && rest != "the cooked fish."
            && rest != "the salted fish."
            && rest != "the smoked fish."
            && rest != "the smoked meat."
        {
            return None;
        }
        Some(crate::model::spoken_act(None, "eat", rest))
    }

    /// A cooked mushroom eaten, filled for the person who ate it.
    fn eat_mushroom_for_these_eyes(
        &self,
        viewer: ActorId,
        event: &crate::model::Event,
    ) -> Option<String> {
        if event.affected_actors.len() != 1 || event.affected_actors[0] != viewer {
            return None;
        }
        let name = self.name_of(viewer);
        let rest = event.text.strip_prefix(&format!("{name} eats "))?;
        if rest != "the cooked mushroom."
            && rest != "the cooked eggs."
            && rest != "the cooked pale berries."
            && rest != "a handful of pale berries."
        {
            return None;
        }
        Some(crate::model::spoken_act(None, "eat", rest))
    }

    /// Food cooked over a fire. The cook already has the reply, so the event
    /// list does not repeat it. Someone who can see keeps the third person.
    fn cook_reply_for_these_eyes(
        &self,
        viewer: ActorId,
        event: &crate::model::Event,
    ) -> Option<String> {
        if event.affected_actors.len() != 1 || event.affected_actors[0] != viewer {
            return None;
        }
        let name = self.name_of(viewer);
        let rest = event.text.strip_prefix(&format!("{name} cooks "))?;
        if rest.ends_with(" over the fire.") {
            Some(event.text.clone())
        } else {
            None
        }
    }

    /// Raw meat held over the fire, filled for the person who cooked it.
    fn cook_meat_for_these_eyes(
        &self,
        viewer: ActorId,
        event: &crate::model::Event,
    ) -> Option<String> {
        if event.affected_actors.len() != 1 || event.affected_actors[0] != viewer {
            return None;
        }
        let name = self.name_of(viewer);
        let rest = event.text.strip_prefix(&format!("{name} cooks "))?;
        if rest != "the raw meat over the fire." && rest != "the fish over the fire." {
            return None;
        }
        Some(crate::model::spoken_act(None, "cook", rest))
    }

    /// A mushroom held over the fire, filled for the person who cooked it.
    fn cook_mushroom_for_these_eyes(
        &self,
        viewer: ActorId,
        event: &crate::model::Event,
    ) -> Option<String> {
        if event.affected_actors.len() != 1 || event.affected_actors[0] != viewer {
            return None;
        }
        let name = self.name_of(viewer);
        let rest = event.text.strip_prefix(&format!("{name} cooks "))?;
        if rest != "the mushroom over the fire."
            && rest != "the eggs over the fire."
            && rest != "the pale berries over the fire."
            && rest != "the cattail roots over the fire."
            && rest != "the flour into a flatbread over the fire."
        {
            return None;
        }
        Some(crate::model::spoken_act(None, "cook", rest))
    }

    fn meat_here_line(&self, cell: crate::model::CellId) -> Option<String> {
        let ids = self.ground_ids(cell, Material::Meat);
        if ids.is_empty() {
            return None;
        }
        Some(self.meat_place_lines(&ids, "here").join(" "))
    }

    fn meat_place_lines(&self, ids: &[crate::model::ItemId], place: &str) -> Vec<String> {
        let mut lines = Vec::new();
        for face in [
            "hanging", "set", "raw", "going", "spoiled", "salted", "smoked", "cooked",
        ] {
            let full = self.meat_tally(ids, face, false);
            let small = self.meat_tally(ids, face, true);
            if full > 0 {
                lines.push(meat_lie(face, full, place, false));
            }
            if small > 0 {
                lines.push(meat_lie(face, small, place, true));
            }
        }
        lines
    }

    /// Raw, going bad, spoiled, salted, or cooked. Look is rebuilt from the item.
    fn meat_face(&self, meat: &crate::model::Item) -> &'static str {
        if meat.cooked {
            "cooked"
        } else if meat.spoiled {
            "spoiled"
        } else if meat.smoked {
            "smoked"
        } else if meat.firing_ready_at.is_some() {
            if self.fire_at(meat.cell).is_some() {
                "hanging"
            } else {
                "set"
            }
        } else if meat.salted {
            "salted"
        } else if meat.spoils_at == Some(self.time) {
            "going"
        } else {
            "raw"
        }
    }

    fn meat_tally(&self, ids: &[crate::model::ItemId], face: &str, small: bool) -> u32 {
        ids.iter()
            .filter(|id| {
                self.item(**id)
                    .is_some_and(|meat| meat.small == small && self.meat_face(meat) == face)
            })
            .count() as u32
    }

    fn meat_compass_bit(&self, cell: crate::model::CellId) -> Option<String> {
        let ids = self.ground_ids(cell, Material::Meat);
        if ids.is_empty() {
            return None;
        }
        let mut parts = Vec::new();
        for face in [
            "hanging", "set", "raw", "going", "spoiled", "salted", "smoked", "cooked",
        ] {
            let full = self.meat_tally(&ids, face, false);
            let small = self.meat_tally(&ids, face, true);
            if full > 0 {
                parts.push(meat_bit(face, full, false));
            }
            if small > 0 {
                parts.push(meat_bit(face, small, true));
            }
        }
        Some(parts.join(" and "))
    }

    /// A carcass cut with the flint spear, filled for the person who cut it.
    fn butcher_for_these_eyes(
        &self,
        viewer: ActorId,
        event: &crate::model::Event,
    ) -> Option<String> {
        if event.affected_actors.len() != 1 || event.affected_actors[0] != viewer {
            return None;
        }
        let name = self.name_of(viewer);
        let rest = event.text.strip_prefix(&format!("{name} butchers "))?;
        if rest != "the carcass into three portions of raw meat, a hide, and a bone."
            && rest
                != "the carcass with the iron knife into three portions of raw meat, a hide, and a bone."
            && rest != "the wolf carcass into three portions of raw meat, a hide, and a bone."
            && rest
                != "the wolf carcass with the iron knife into three portions of raw meat, a hide, and a bone."
        {
            return None;
        }
        Some(crate::model::spoken_act(None, "butcher", rest))
    }

    /// A reed weir set in the brook. The person who set it already has this
    /// as the reply, so the event list does not repeat it. A witness keeps
    /// the stored sentence. The later catch names no one.
    fn weir_for_these_eyes(&self, viewer: ActorId, event: &crate::model::Event) -> Option<String> {
        if event.affected_actors.len() != 1 || event.affected_actors[0] != viewer {
            return None;
        }
        let name = self.name_of(viewer);
        let rest = event.text.strip_prefix(&format!("{name} sets "))?;
        if rest != "a reed weir in the brook." {
            return None;
        }
        Some(crate::model::spoken_act(None, "set", rest))
    }

    /// A fishing net set in the brook. The person who set it already has this
    /// as the reply, so the event list does not repeat it.
    fn set_net_for_these_eyes(
        &self,
        viewer: ActorId,
        event: &crate::model::Event,
    ) -> Option<String> {
        if event.affected_actors.len() != 1 || event.affected_actors[0] != viewer {
            return None;
        }
        let name = self.name_of(viewer);
        let rest = event.text.strip_prefix(&format!("{name} sets "))?;
        if rest != "a fishing net in the brook." {
            return None;
        }
        Some(crate::model::spoken_act(None, "set", rest))
    }

    /// A snare of hide strips. The person who set it already has this as the
    /// reply, so the event list does not repeat it. A witness keeps the
    /// stored sentence.
    fn snare_for_these_eyes(&self, viewer: ActorId, event: &crate::model::Event) -> Option<String> {
        if event.affected_actors.len() != 1 || event.affected_actors[0] != viewer {
            return None;
        }
        let name = self.name_of(viewer);
        let rest = event.text.strip_prefix(&format!("{name} sets "))?;
        if rest != "a snare of hide strips."
            && rest != "a snare of willow cordage."
            && rest != "a snare of nettle cordage."
        {
            return None;
        }
        Some(crate::model::spoken_act(None, "set", rest))
    }

    /// A portion split with the iron knife. The cutter already has this as
    /// the reply, so the event list does not repeat it. A witness keeps the
    /// stored sentence. That covers hide strips and a cut of raw or cooked meat.
    fn cut_for_these_eyes(&self, viewer: ActorId, event: &crate::model::Event) -> Option<String> {
        if event.affected_actors.len() != 1 || event.affected_actors[0] != viewer {
            return None;
        }
        let name = self.name_of(viewer);
        let rest = event.text.strip_prefix(&format!("{name} cuts "))?;
        if rest != "the raw meat into two smaller pieces with the iron knife."
            && rest != "the cooked meat into two smaller pieces with the iron knife."
            && rest != "the salted meat into two smaller pieces with the iron knife."
            && rest != "the smoked meat into two smaller pieces with the iron knife."
            && rest != "the deer hide into strips with the iron knife."
        {
            return None;
        }
        Some(crate::model::spoken_act(None, "cut", rest))
    }

    fn basket_arm_phrase(&self, actor_id: ActorId) -> String {
        match self.basket_contents(actor_id) {
            Some(Material::Berry) => "a bark basket holding a handful of berries".into(),
            Some(Material::Fish) => match self.basket_fish_state(actor_id) {
                "cooked" => "a bark basket holding a cooked fish".into(),
                "smoked" => "a bark basket holding a smoked fish".into(),
                "salted" => "a bark basket holding a salted fish".into(),
                "spoiled" => "a bark basket holding a spoiled fish".into(),
                "going" => "a bark basket holding a raw fish that is going bad".into(),
                _ => "a bark basket holding a raw fish".into(),
            },
            _ => "a bark basket".into(),
        }
    }

    fn basket_contents(&self, actor_id: ActorId) -> Option<Material> {
        self.items.values().find_map(|item| {
            (item.carrier == Some(actor_id) && item.in_basket).then_some(item.material)
        })
    }

    fn basket_held_sentence(&self, actor_id: ActorId) -> String {
        match self.basket_contents(actor_id) {
            Some(Material::Berry) => "It holds a handful of berries.".into(),
            Some(Material::Fish) => match self.basket_fish_state(actor_id) {
                "cooked" => "It holds a cooked fish.".into(),
                "smoked" => "It holds a smoked fish. It can be eaten cold.".into(),
                "salted" => "It holds a salted fish. It can be eaten cold.".into(),
                "spoiled" => "It holds a spoiled fish. It is not food.".into(),
                "going" => "It holds a raw fish. It is going bad.".into(),
                _ => "It holds a raw fish.".into(),
            },
            _ => "It is empty.".into(),
        }
    }

    /// Cooked fish stays good. A raw fish is going bad on the tick before it spoils.
    fn fish_state(&self, id: crate::model::ItemId) -> &'static str {
        let Some(fish) = self.item(id) else {
            return "raw";
        };
        if fish.cooked {
            "cooked"
        } else if fish.spoiled {
            "spoiled"
        } else if fish.smoked {
            "smoked"
        } else if fish.salted {
            "salted"
        } else if fish.firing_ready_at.is_some() {
            if self.fire_at(fish.cell).is_some() {
                "hanging"
            } else {
                "set"
            }
        } else if fish.spoils_at == Some(self.time) {
            "going"
        } else {
            "raw"
        }
    }

    fn fish_ground_line(&self, id: crate::model::ItemId) -> String {
        match self.fish_state(id) {
            "cooked" => "A cooked fish lies here.".into(),
            "smoked" => "A smoked fish lies here. It can be eaten cold.".into(),
            "salted" => "A salted fish lies here. It can be eaten cold.".into(),
            "spoiled" => "A spoiled fish lies here. It is not food.".into(),
            "going" => "A raw fish lies here. It is going bad.".into(),
            "hanging" => "A raw fish hangs by the fire. It is not smoked yet.".into(),
            "set" => "A raw fish is set near the fire. It is not smoked yet.".into(),
            _ => "A raw fish lies here.".into(),
        }
    }

    fn fish_carry_phrase(&self, actor_id: ActorId) -> String {
        let mut bits = Vec::new();
        for id in self.loose_ids(actor_id, Material::Fish) {
            if self.item(id).is_some_and(|fish| fish.in_basket) {
                continue;
            }
            bits.push(match self.fish_state(id) {
                "cooked" => "a cooked fish",
                "smoked" => "a smoked fish",
                "salted" => "a salted fish",
                "spoiled" => "a spoiled fish",
                "going" => "a raw fish that is going bad",
                _ => "a raw fish",
            });
        }
        if bits.is_empty() {
            "a raw fish".into()
        } else {
            bits.join(", ")
        }
    }

    fn basket_fish_state(&self, actor_id: ActorId) -> &'static str {
        let id = self
            .items
            .values()
            .find(|item| {
                item.carrier == Some(actor_id) && item.in_basket && item.material == Material::Fish
            })
            .map(|fish| fish.id);
        match id {
            Some(id) => self.fish_state(id),
            None => "raw",
        }
    }

    fn examine_basket(&self, viewer: ActorId) -> String {
        let origin = self.actor(viewer).unwrap().cell;
        let mut lines = Vec::new();
        for cell in self.visible_cell_ids(viewer) {
            if !self.ground_ids(cell, Material::Basket).is_empty() {
                lines.push(format!(
                    "A bark basket lies {}. It is woven from a sheet of bark. It is empty. {}",
                    self.place_phrase(origin, cell),
                    self.rain_reaches_phrase(cell)
                ));
            }
        }
        for other in self.visible_actor_ids(viewer) {
            if self.loose_ids(other, Material::Basket).is_empty() {
                continue;
            }
            let who = if other == viewer {
                "You carry".to_string()
            } else {
                format!("{} carries", self.name_of(other))
            };
            let held = self.basket_held_sentence(other);
            let cell = self.actor(other).unwrap().cell;
            lines.push(format!(
                "{who} a bark basket, {}. It is woven from a sheet of bark. {held}",
                self.cover_phrase(cell)
            ));
        }
        if lines.is_empty() {
            "You see no bark basket from here.".into()
        } else {
            lines.join("\n")
        }
    }

    /// A bark cup is a folded sheet, not a clay pot and not a bark basket.
    fn examine_bark_cup(&self, viewer: ActorId) -> String {
        let origin = self.actor(viewer).unwrap().cell;
        let mut lines = Vec::new();
        for cell in self.visible_cell_ids(viewer) {
            for id in self.ground_ids(cell, Material::BarkCup) {
                let (name, hold) = self.bark_cup_examine(id, true);
                lines.push(format!(
                    "{name} lies {}. It is folded from a sheet of birch bark. {hold} {}",
                    self.place_phrase(origin, cell),
                    self.rain_reaches_phrase(cell)
                ));
            }
        }
        for other in self.visible_actor_ids(viewer) {
            let Some(id) = self.loose_ids(other, Material::BarkCup).first().copied() else {
                continue;
            };
            let (name, hold) = self.bark_cup_examine(id, false);
            let who = if other == viewer {
                "You carry".to_string()
            } else {
                format!("{} carries", self.name_of(other))
            };
            let cell = self.actor(other).unwrap().cell;
            lines.push(format!(
                "{who} {name}, {}. It is folded from a sheet of birch bark. {hold}",
                self.cover_phrase(cell)
            ));
        }
        if lines.is_empty() {
            "You see no bark cup from here.".into()
        } else {
            lines.join("\n")
        }
    }

    /// A raw fish set into the bark basket, filled for the person who put it there.
    fn put_fish_for_these_eyes(
        &self,
        viewer: ActorId,
        event: &crate::model::Event,
    ) -> Option<String> {
        if event.affected_actors.len() != 1 || event.affected_actors[0] != viewer {
            return None;
        }
        let name = self.name_of(viewer);
        let rest = event.text.strip_prefix(&format!("{name} puts "))?;
        if rest != "the raw fish in the bark basket." {
            return None;
        }
        Some(crate::model::spoken_act(None, "put", rest))
    }

    /// A bow crafted from a dry branch and hide strips. The crafter already
    /// has this as the reply, so the event list does not repeat it.
    fn craft_for_these_eyes(&self, viewer: ActorId, event: &crate::model::Event) -> Option<String> {
        if event.affected_actors.len() != 1 || event.affected_actors[0] != viewer {
            return None;
        }
        let name = self.name_of(viewer);
        let rest = event.text.strip_prefix(&format!("{name} crafts "))?;
        if rest != "a bow from a dry branch and hide strips."
            && rest != "a bow from a dry branch and willow cordage."
            && rest != "a bow from a dry branch and nettle cordage."
        {
            return None;
        }
        Some(crate::model::spoken_act(None, "craft", rest))
    }

    /// A deer brought down with a carried bow. The hunter already has this
    /// as the reply, so the event list does not repeat it. A witness who
    /// can see the deer's cell keeps the stored line.
    fn bow_hunt_for_these_eyes(
        &self,
        viewer: ActorId,
        event: &crate::model::Event,
    ) -> Option<String> {
        if event.affected_actors.len() != 1 || event.affected_actors[0] != viewer {
            return None;
        }
        let name = self.name_of(viewer);
        let rest = event.text.strip_prefix(&format!("{name} brings "))?;
        if rest != "the deer down with the bow. The flint flake is spent." {
            return None;
        }
        Some(crate::model::spoken_act(None, "bring", rest))
    }

    fn examine_net(&self, viewer: ActorId) -> String {
        let origin = self.actor(viewer).unwrap().cell;
        let mut lines = Vec::new();
        for cell in self.visible_cell_ids(viewer) {
            if self.set_nets.contains(&cell) {
                let place = if origin == cell {
                    "here, in the brook".to_string()
                } else {
                    format!("in the brook to the {}", self.relative(origin, cell))
                };
                let mut line = format!("A fishing net is set {place}. It is woven from reeds.");
                if !self.ground_ids(cell, Material::Fish).is_empty() {
                    line.push_str(" A raw fish lies there.");
                }
                lines.push(line);
                continue;
            }
            let count = self.ground_ids(cell, Material::Net).len() as u32;
            if count == 0 {
                continue;
            }
            let place = if origin == cell {
                "here".to_string()
            } else {
                format!("to the {}", self.relative(origin, cell))
            };
            if count == 1 {
                lines.push(format!(
                    "A fishing net lies {place}. It is woven from reeds."
                ));
            } else {
                lines.push(format!(
                    "Fishing nets lie {place}. Each is woven from reeds."
                ));
            }
        }
        for actor in self.actors.values() {
            if chebyshev(origin, actor.cell) > self.tuning.sight {
                continue;
            }
            let count = self.loose_ids(actor.id, Material::Net).len() as u32;
            if count == 0 {
                continue;
            }
            let who = if actor.id == viewer {
                "you"
            } else {
                actor.name.as_str()
            };
            let cover = self.cover_phrase(actor.cell);
            if count == 1 {
                lines.push(format!(
                    "A fishing net is carried by {who}, {cover}. It is woven from reeds."
                ));
            } else {
                lines.push(format!(
                    "{} fishing nets are carried by {who}, {cover}. Each is woven from reeds.",
                    capitalize(&count_word(count))
                ));
            }
        }
        if lines.is_empty() {
            "You see no fishing net from here.".into()
        } else {
            lines.join("\n")
        }
    }

    fn examine_mat(&self, viewer: ActorId) -> String {
        let origin = self.actor(viewer).unwrap().cell;
        let mut lines = Vec::new();
        for cell in self.visible_cell_ids(viewer) {
            let count = self.ground_ids(cell, Material::Mat).len() as u32;
            if count == 0 {
                continue;
            }
            let place = if origin == cell {
                "here".to_string()
            } else {
                format!("to the {}", self.relative(origin, cell))
            };
            if count == 1 {
                lines.push(format!("A reed mat lies {place}. It is woven from reeds."));
            } else {
                lines.push(format!("Reed mats lie {place}. Each is woven from reeds."));
            }
        }
        for actor in self.actors.values() {
            if chebyshev(origin, actor.cell) > self.tuning.sight {
                continue;
            }
            let count = self.loose_ids(actor.id, Material::Mat).len() as u32;
            if count == 0 {
                continue;
            }
            let who = if actor.id == viewer {
                "you"
            } else {
                actor.name.as_str()
            };
            let cover = self.cover_phrase(actor.cell);
            if count == 1 {
                lines.push(format!(
                    "A reed mat is carried by {who}, {cover}. It is woven from reeds."
                ));
            } else {
                lines.push(format!(
                    "{} reed mats are carried by {who}, {cover}. Each is woven from reeds.",
                    capitalize(&count_word(count))
                ));
            }
        }
        if lines.is_empty() {
            "You see no reed mat from here.".into()
        } else {
            lines.join("\n")
        }
    }

    fn examine_canoe(&self, viewer: ActorId) -> String {
        let origin = self.actor(viewer).unwrap().cell;
        let mut lines = Vec::new();
        for cell in self.visible_cell_ids(viewer) {
            let Some(id) = self.canoe_at(cell) else {
                continue;
            };
            let place = if origin == cell {
                "here".to_string()
            } else {
                format!("to the {}", self.relative(origin, cell))
            };
            let hull = if self.canoe_pitched(id) {
                "pitched dugout canoe"
            } else {
                "dugout canoe"
            };
            let seal = if self.canoe_pitched(id) {
                " Pine resin seals it."
            } else {
                ""
            };
            if self.canoe_floats(id) {
                lines.push(format!(
                    "A {hull} floats in the brook {place}.{seal} It was hollowed from the heavy log."
                ));
            } else {
                lines.push(format!(
                    "A {hull} lies {place}.{seal} It was hollowed from the heavy log."
                ));
            }
        }
        if lines.is_empty() {
            "You see no dugout canoe from here.".into()
        } else {
            lines.join(" ")
        }
    }

    /// Hollowing, carving, hauling, launching, or boarding the dugout.
    /// The person who did it already has the reply.
    fn canoe_for_these_eyes(&self, viewer: ActorId, event: &crate::model::Event) -> Option<String> {
        if event.affected_actors.len() != 1 || event.affected_actors[0] != viewer {
            return None;
        }
        let name = self.name_of(viewer);
        let text = event.text.as_str();
        if text == format!("{name} hollows the heavy log with the fire.")
            || text == format!("{name} carves the charred log into a dugout canoe.")
            || text == format!("{name} launches the dugout canoe into the brook.")
            || text == format!("{name} boards the dugout canoe.")
            || text == format!("{name} drags the dugout canoe north.")
            || text == format!("{name} drags the dugout canoe south.")
            || text == format!("{name} drags the dugout canoe east.")
            || text == format!("{name} drags the dugout canoe west.")
            || text == format!("{name} paddles the dugout canoe north.")
            || text == format!("{name} paddles the dugout canoe south.")
            || text == format!("{name} paddles the dugout canoe east.")
            || text == format!("{name} paddles the dugout canoe west.")
            || text == format!("{name} paddles the dugout canoe north with the wooden paddle.")
            || text == format!("{name} paddles the dugout canoe south with the wooden paddle.")
            || text == format!("{name} paddles the dugout canoe east with the wooden paddle.")
            || text == format!("{name} paddles the dugout canoe west with the wooden paddle.")
            || text == format!("{name} leaves the dugout canoe.")
            || text == format!("{name} beaches the dugout canoe.")
            || text == format!("{name} seals the dugout canoe with pine resin.")
        {
            return Some(text.to_string());
        }
        None
    }

    /// A canoe refusal the actor already has as the command reply.
    /// It does not sit in their event list. Nobody else is in the audience.
    /// The solo pull is the reply only. Someone who can see still hears the pull.
    fn alone_pull_for_these_eyes(
        &self,
        viewer: ActorId,
        event: &crate::model::Event,
    ) -> Option<String> {
        if event.affected_actors.len() != 1 || event.affected_actors[0] != viewer {
            return None;
        }
        let name = self.name_of(viewer);
        let text = event.text.as_str();
        if text == format!("{name} pulls on the log alone. It does not move.") {
            return Some(text.to_string());
        }
        None
    }

    fn canoe_refusal_for_these_eyes(
        &self,
        viewer: ActorId,
        event: &crate::model::Event,
    ) -> Option<String> {
        if event.affected_actors.len() != 1 || event.affected_actors[0] != viewer {
            return None;
        }
        const LINES: &[&str] = &[
            "You are aboard the canoe. Disembark first.",
            "The canoe stays in the brook.",
            "There is nowhere to paddle.",
            "The canoe has nowhere to go.",
            "The canoe is already on land.",
            "There is no dugout canoe here.",
            "There is no bank here.",
            "The canoe is already in the brook.",
            "There is nowhere to drag the canoe.",
            "Launch the canoe from the bank.",
            "The canoe is already carved.",
            "You need a flint flake or an iron knife to carve the canoe.",
            "You are already aboard the canoe.",
            "There is no canoe here to board.",
            "You are not aboard a canoe.",
            "The brook is not beside you.",
            "The log is already a canoe.",
            "You have no pine resin.",
            "The canoe is already pitched.",
        ];
        LINES
            .contains(&event.text.as_str())
            .then(|| event.text.clone())
    }

    /// A reed mat woven, set, dropped, or rested on. The person who did it
    /// already has the reply, so the event list does not repeat it.
    fn mat_for_these_eyes(&self, viewer: ActorId, event: &crate::model::Event) -> Option<String> {
        if event.affected_actors.len() != 1 || event.affected_actors[0] != viewer {
            return None;
        }
        let name = self.name_of(viewer);
        let text = event.text.as_str();
        if text == format!("{name} weaves a reed mat.")
            || text == format!("{name} sets a reed mat.")
            || text == format!("{name} rests on the reed mat.")
            || text == format!("{name} drops a reed mat. It is in contact with the ground.")
        {
            return Some(text.to_string());
        }
        None
    }

    /// A fishing net woven from three bundles of reeds. The weaver already
    /// has this as the reply, so the event list does not repeat it.
    fn weave_net_for_these_eyes(
        &self,
        viewer: ActorId,
        event: &crate::model::Event,
    ) -> Option<String> {
        if event.affected_actors.len() != 1 || event.affected_actors[0] != viewer {
            return None;
        }
        let name = self.name_of(viewer);
        let rest = event.text.strip_prefix(&format!("{name} weaves "))?;
        if rest != "a fishing net." {
            return None;
        }
        Some(crate::model::spoken_act(None, "weave", rest))
    }

    /// A fish taken with a net. The fisher already has this as the reply.
    fn net_fish_for_these_eyes(
        &self,
        viewer: ActorId,
        event: &crate::model::Event,
    ) -> Option<String> {
        if event.affected_actors.len() != 1 || event.affected_actors[0] != viewer {
            return None;
        }
        let name = self.name_of(viewer);
        let rest = event.text.strip_prefix(&format!("{name} takes "))?;
        if rest != "a fish from the brook with the net." {
            return None;
        }
        Some(crate::model::spoken_act(None, "take", rest))
    }

    /// A fish taken with a bone hook. The fisher already has this as the reply.
    fn hook_fish_for_these_eyes(
        &self,
        viewer: ActorId,
        event: &crate::model::Event,
    ) -> Option<String> {
        if event.affected_actors.len() != 1 || event.affected_actors[0] != viewer {
            return None;
        }
        let name = self.name_of(viewer);
        let rest = event.text.strip_prefix(&format!("{name} catches "))?;
        if rest != "a fish from the brook with the bone fish hook. The hide strips are spent."
            && rest != "a fish from the brook with the bone fish hook. The willow cordage is spent."
            && rest != "a fish from the brook with the bone fish hook. The nettle cordage is spent."
            && rest
                != "a fish from the brook with the bone fish hook and a worm. The hide strips are spent."
            && rest
                != "a fish from the brook with the bone fish hook and a worm. The willow cordage is spent."
            && rest
                != "a fish from the brook with the bone fish hook and a worm. The nettle cordage is spent."
            && rest != "a fish from the brook with the bone fish hook and a worm."
            && rest != "a fish from the shallows with the bone fish hook and a worm."
            && rest != "a fish from the hole in the ice with the bone fish hook and a worm."
        {
            return None;
        }
        Some(crate::model::spoken_act(None, "catch", rest))
    }

    /// A bark basket woven from a sheet of bark, filled for the weaver.
    fn weave_for_these_eyes(&self, viewer: ActorId, event: &crate::model::Event) -> Option<String> {
        if event.affected_actors.len() != 1 || event.affected_actors[0] != viewer {
            return None;
        }
        let name = self.name_of(viewer);
        let rest = event.text.strip_prefix(&format!("{name} weaves "))?;
        if rest != "a bark basket." {
            return None;
        }
        Some(crate::model::spoken_act(None, "weave", rest))
    }

    /// A bark cup folded from a sheet of bark. The folder already read the sentence.
    fn fold_for_these_eyes(&self, viewer: ActorId, event: &crate::model::Event) -> Option<String> {
        if event.affected_actors.len() != 1 || event.affected_actors[0] != viewer {
            return None;
        }
        let name = self.name_of(viewer);
        let rest = event.text.strip_prefix(&format!("{name} folds "))?;
        if rest != "a sheet of bark into a bark cup." {
            return None;
        }
        Some(crate::model::spoken_act(None, "fold", rest))
    }

    /// Roots pounded into flour. The person who ground them already read the sentence.
    fn grind_for_these_eyes(&self, viewer: ActorId, event: &crate::model::Event) -> Option<String> {
        if event.affected_actors.len() != 1 || event.affected_actors[0] != viewer {
            return None;
        }
        let name = self.name_of(viewer);
        let rest = event.text.strip_prefix(&format!("{name} grinds "))?;
        if rest != "the cattail roots into flour on the stone." {
            return None;
        }
        Some(crate::model::spoken_act(None, "grind", rest))
    }

    /// A bark cup sealed with pine resin. The person who sealed it already read the sentence.
    fn seal_cup_for_these_eyes(
        &self,
        viewer: ActorId,
        event: &crate::model::Event,
    ) -> Option<String> {
        if event.affected_actors.len() != 1 || event.affected_actors[0] != viewer {
            return None;
        }
        let name = self.name_of(viewer);
        let rest = event.text.strip_prefix(&format!("{name} seals "))?;
        if rest != "the bark cup with pine resin." {
            return None;
        }
        Some(crate::model::spoken_act(None, "seal", rest))
    }

    /// A dry branch pointed into a wooden spear. The person who pointed it
    /// already has this as the reply, so the event list does not repeat it.
    fn point_for_these_eyes(&self, viewer: ActorId, event: &crate::model::Event) -> Option<String> {
        if event.affected_actors.len() != 1 || event.affected_actors[0] != viewer {
            return None;
        }
        let name = self.name_of(viewer);
        let rest = event.text.strip_prefix(&format!("{name} points "))?;
        if rest != "a dry branch into a wooden spear." {
            return None;
        }
        Some(crate::model::spoken_act(None, "point", rest))
    }

    /// A wooden point held in the fire. The person who held it already has
    /// the reply, so the event list does not repeat it.
    fn hold_point_for_these_eyes(
        &self,
        viewer: ActorId,
        event: &crate::model::Event,
    ) -> Option<String> {
        if event.affected_actors.len() != 1 || event.affected_actors[0] != viewer {
            return None;
        }
        let name = self.name_of(viewer);
        let rest = event.text.strip_prefix(&format!("{name} holds "))?;
        if rest != "the wooden point in the fire." {
            return None;
        }
        Some("You hold the wooden point in the fire. It is not hard yet.".into())
    }

    /// A lump shaped into a pot. The shaper already has this as the reply,
    /// so the event list does not repeat it. A witness keeps the stored line.
    fn shape_for_these_eyes(&self, viewer: ActorId, event: &crate::model::Event) -> Option<String> {
        if event.affected_actors.len() != 1 || event.affected_actors[0] != viewer {
            return None;
        }
        let name = self.name_of(viewer);
        let rest = event.text.strip_prefix(&format!("{name} shapes "))?;
        if rest != "the clay into an unfired pot." {
            return None;
        }
        Some(crate::model::spoken_act(None, "shape", rest))
    }

    /// An unfired pot set by a fire. The person who set it already has this
    /// as the reply, so the event list does not repeat it. A witness keeps
    /// the stored line.
    fn fire_pot_for_these_eyes(
        &self,
        viewer: ActorId,
        event: &crate::model::Event,
    ) -> Option<String> {
        if event.affected_actors.len() != 1 || event.affected_actors[0] != viewer {
            return None;
        }
        let name = self.name_of(viewer);
        let rest = event.text.strip_prefix(&format!("{name} sets "))?;
        let yours = if rest == "the unfired pot by the fire. It leaves their arms." {
            "You set the unfired pot by the fire. It leaves your arms."
        } else if rest == "the unfired pot by the fire." {
            "You set the unfired pot by the fire."
        } else {
            return None;
        };
        Some(yours.to_string())
    }

    /// A lean-to raised from branches, filled for the person who built it.
    fn lean_to_for_these_eyes(
        &self,
        viewer: ActorId,
        event: &crate::model::Event,
    ) -> Option<String> {
        if event.affected_actors.len() != 1 || event.affected_actors[0] != viewer {
            return None;
        }
        let name = self.name_of(viewer);
        let rest = event.text.strip_prefix(&format!("{name} builds "))?;
        if rest != "a lean-to of three branches." {
            return None;
        }
        Some(crate::model::spoken_act(None, "build", rest))
    }

    /// Reeds laid on a lean-to, filled for the person who thatched it.
    fn thatch_for_these_eyes(
        &self,
        viewer: ActorId,
        event: &crate::model::Event,
    ) -> Option<String> {
        if event.affected_actors.len() != 1 || event.affected_actors[0] != viewer {
            return None;
        }
        let name = self.name_of(viewer);
        let rest = event.text.strip_prefix(&format!("{name} thatches "))?;
        if rest != "the lean-to with reeds." {
            return None;
        }
        Some(crate::model::spoken_act(None, "thatch", rest))
    }

    /// A wooden spear bringing the deer down. The hunter already has the reply,
    /// so that event does not repeat. Someone who can see the cell keeps
    /// the stored line. A stone spear is not marked, and keeps its own line.
    fn wooden_deer_for_these_eyes(
        &self,
        viewer: ActorId,
        event: &crate::model::Event,
    ) -> Option<String> {
        if event.affected_actors.len() != 1 || event.affected_actors[0] != viewer {
            return None;
        }
        let name = self.name_of(viewer);
        if event.text != format!("{name} brings the deer down.") {
            return None;
        }
        let wooden = event.affected_items.iter().any(|id| {
            self.item(*id)
                .is_some_and(|item| item.material == Material::Spear && item.wooden_tip)
        });
        if !wooden {
            return None;
        }
        Some("You bring the deer down. A carcass lies where it stood.".into())
    }

    /// A flint spear bringing the deer down, filled for the person who struck.
    /// Anyone else keeps the stored sentence. A stone spear keeps its own line.
    fn bring_for_these_eyes(&self, viewer: ActorId, event: &crate::model::Event) -> Option<String> {
        if event.affected_actors.len() != 1 || event.affected_actors[0] != viewer {
            return None;
        }
        let name = self.name_of(viewer);
        let rest = event.text.strip_prefix(&format!("{name} brings "))?;
        if rest != "the deer down with the flint spear." {
            return None;
        }
        Some(crate::model::spoken_act(None, "bring", rest))
    }

    /// A miss from the next cell, filled for the person who threw.
    /// Anyone else who can see the deer's cell keeps the stored sentence.
    fn miss_for_these_eyes(&self, viewer: ActorId, event: &crate::model::Event) -> Option<String> {
        if event.affected_actors.len() != 1 || event.affected_actors[0] != viewer {
            return None;
        }
        let name = self.name_of(viewer);
        let rest = event.text.strip_prefix(&format!("{name} misses "))?;
        if rest != "the deer. It moves off."
            && rest != "the deer with the flint spear. It moves off."
        {
            return None;
        }
        Some(crate::model::spoken_act(None, "miss", rest))
    }

    /// A bolt from the next cell, filled for the person who threw the flint spear.
    /// Anyone else who can see the deer's cell keeps the stored sentence.
    /// A stone spear keeps its own line.
    fn bolt_for_these_eyes(&self, viewer: ActorId, event: &crate::model::Event) -> Option<String> {
        if event.affected_actors.len() != 1 || event.affected_actors[0] != viewer {
            return None;
        }
        let name = self.name_of(viewer);
        if event.text != format!("The deer bolts. {name}'s flint spear does not land.") {
            return None;
        }
        Some("The deer bolts. Your flint spear does not land.".to_string())
    }

    /// A flint tip finishing in the fire, filled for the person who held it.
    /// Anyone else keeps the stored sentence. A stone point keeps its own line.
    fn harden_for_these_eyes(
        &self,
        viewer: ActorId,
        event: &crate::model::Event,
    ) -> Option<String> {
        if event.affected_actors.len() != 1 || event.affected_actors[0] != viewer {
            return None;
        }
        let name = self.name_of(viewer);
        let rest = event.text.strip_prefix(&format!("{name} hardens "))?;
        if rest != "the flint tip of the spear in the fire." {
            return None;
        }
        Some(crate::model::spoken_act(None, "harden", rest))
    }

    /// The wooden point finishing in the fire. The holder hears "your".
    /// A witness keeps the stored line. This is the result, not the command.
    fn wooden_point_for_these_eyes(
        &self,
        viewer: ActorId,
        event: &crate::model::Event,
    ) -> Option<String> {
        if event.affected_actors.len() != 1 || event.affected_actors[0] != viewer {
            return None;
        }
        let name = self.name_of(viewer);
        if event.text != format!("The wooden point of {name}'s spear hardens.") {
            return None;
        }
        Some("The wooden point of your spear hardens.".into())
    }

    /// A brand taken, or going out, filled for the person who carries it.
    fn brand_for_these_eyes(&self, viewer: ActorId, event: &crate::model::Event) -> Option<String> {
        if event.affected_actors.len() != 1 || event.affected_actors[0] != viewer {
            return None;
        }
        let name = self.name_of(viewer);
        if let Some(rest) = event.text.strip_prefix(&format!("{name} takes ")) {
            if rest.starts_with("a burning brand from the fire.") {
                return Some(crate::model::spoken_act(None, "take", rest));
            }
        }
        if event.text == format!("{name}'s brand burns out.") {
            return Some("Your brand burns out.".into());
        }
        if event.text == format!("The rain puts {name}'s brand out.") {
            return Some("The rain puts your brand out.".into());
        }
        None
    }

    /// A fish taken from the brook, filled for the person who speared it.
    fn fish_for_these_eyes(&self, viewer: ActorId, event: &crate::model::Event) -> Option<String> {
        if event.affected_actors.len() != 1 || event.affected_actors[0] != viewer {
            return None;
        }
        let name = self.name_of(viewer);
        let rest = event.text.strip_prefix(&format!("{name} spears "))?;
        if rest != "a fish from the brook." {
            return None;
        }
        Some(crate::model::spoken_act(None, "spear", rest))
    }

    /// The walk. The walker already has this as the reply, so the event
    /// list does not repeat it. Anyone else keeps the stored sentence, or
    /// the footsteps line when they cannot see the arrival.
    fn walk_for_these_eyes(&self, viewer: ActorId, event: &crate::model::Event) -> Option<String> {
        if event.affected_actors.len() != 1 || event.affected_actors[0] != viewer {
            return None;
        }
        let name = self.name_of(viewer);
        if let Some(rest) = event.text.strip_prefix(&format!("{name} walks ")) {
            return Some(crate::model::spoken_act(None, "walk", rest));
        }
        if let Some(rest) = event.text.strip_prefix(&format!("{name} wades ")) {
            return Some(crate::model::spoken_act(None, "wade", rest));
        }
        None
    }

    /// A wolf howl. The stored sentence has no direction. Within sight of
    /// the corner it stays that way. Farther off, the direction is from
    /// the person who hears it. Look does not name a wolf.
    fn howl_for_these_ears(&self, origin: CellId, event: &crate::model::Event) -> Option<String> {
        if event.text != "A wolf howls." {
            return None;
        }
        let cell = event.affected_cells.first().copied()?;
        if chebyshev(origin, cell) <= self.tuning.sight {
            return Some("You hear a wolf howl.".into());
        }
        let way = self.relative(origin, cell);
        Some(format!("You hear a wolf howl to the {way}."))
    }

    /// A deer or a person stepping into brush or timber. Someone who can see
    /// that cell already has the step, so this line is not delivered there.
    /// Out of sight, within hearing, the direction is from the listener.
    /// Past hearing, nothing.
    fn brush_for_these_ears(&self, origin: CellId, event: &crate::model::Event) -> Option<String> {
        let cell = self.brush_crack_cell(event)?;
        if !self.hears_brush(origin, event) {
            return None;
        }
        let way = self.relative(origin, cell);
        let sound = if event.text == "Wood cracks in the timber." {
            "wood cracking"
        } else {
            "branches breaking"
        };
        Some(format!("You hear {sound} to the {way}."))
    }

    fn hears_brush(&self, origin: CellId, event: &crate::model::Event) -> bool {
        let Some(cell) = self.brush_crack_cell(event) else {
            return false;
        };
        let distance = chebyshev(origin, cell);
        distance > self.tuning.sight && distance <= self.tuning.hearing
    }

    fn brush_crack_marker(&self, event: &crate::model::Event) -> bool {
        event.text == "Branches break in the brush." || event.text == "Wood cracks in the timber."
    }

    fn brush_crack_cell(&self, event: &crate::model::Event) -> Option<CellId> {
        if !self.brush_crack_marker(event) {
            return None;
        }
        event.affected_cells.first().copied()
    }

    /// Ice taking the still pool, heard from out of sight. Someone who can
    /// see the pool already has the ice in look, so the crack is not delivered
    /// there. Past hearing, nothing.
    fn ice_for_these_ears(&self, origin: CellId, event: &crate::model::Event) -> Option<String> {
        let cell = self.ice_cell(event)?;
        if !self.hears_ice_at(origin, cell) {
            return None;
        }
        let way = self.relative(origin, cell);
        Some(format!("You hear ice forming to the {way}."))
    }

    fn hears_ice(&self, origin: CellId, event: &crate::model::Event) -> bool {
        self.ice_cell(event)
            .is_some_and(|cell| self.hears_ice_at(origin, cell))
    }

    fn hears_ice_at(&self, origin: CellId, cell: CellId) -> bool {
        let distance = chebyshev(origin, cell);
        distance > self.tuning.sight && distance <= self.tuning.hearing
    }

    fn ice_forming(&self, event: &crate::model::Event) -> bool {
        event.text == "Ice forms on the still water."
    }

    fn ice_cell(&self, event: &crate::model::Event) -> Option<CellId> {
        if !self.ice_forming(event) {
            return None;
        }
        event.affected_cells.first().copied()
    }

    /// The person who broke the ice already has the reply.
    fn ice_break_for_these_eyes(
        &self,
        viewer: ActorId,
        event: &crate::model::Event,
    ) -> Option<String> {
        if event.affected_actors.len() != 1 || event.affected_actors[0] != viewer {
            return None;
        }
        let name = self.name_of(viewer);
        let rest = event.text.strip_prefix(&format!("{name} breaks "))?;
        if !rest.starts_with("a hole in the ice with ") {
            return None;
        }
        Some(crate::model::spoken_act(None, "break", rest))
    }

    fn ice_break_marker(&self, event: &crate::model::Event) -> bool {
        event.text.contains(" breaks a hole in the ice with ")
    }

    fn ice_break_cell(&self, event: &crate::model::Event) -> Option<CellId> {
        if !self.ice_break_marker(event) {
            return None;
        }
        event.affected_cells.first().copied()
    }

    /// A hole chopped in the ice, heard from out of sight. Someone who can
    /// see the still keeps the break sentence. Past hearing, nothing.
    fn ice_break_for_these_ears(
        &self,
        origin: CellId,
        event: &crate::model::Event,
    ) -> Option<String> {
        let cell = self.ice_break_cell(event)?;
        if !self.hears_an_ice_break(origin, event) {
            return None;
        }
        let way = self.relative(origin, cell);
        Some(format!("You hear ice break to the {way}."))
    }

    fn hears_an_ice_break(&self, origin: CellId, event: &crate::model::Event) -> bool {
        let Some(cell) = self.ice_break_cell(event) else {
            return false;
        };
        let distance = chebyshev(origin, cell);
        distance > self.tuning.sight && distance <= self.tuning.hearing
    }

    /// A successful walk, heard from out of sight. Someone who can see the
    /// arrival keeps the walk sentence. The walker reads it in the second
    /// person instead. Past hearing, nothing is delivered.
    fn footsteps_for_these_ears(
        &self,
        origin: CellId,
        event: &crate::model::Event,
    ) -> Option<String> {
        let cell = self.walk_arrival(event)?;
        if chebyshev(origin, cell) <= self.tuning.sight {
            return None;
        }
        if chebyshev(origin, cell) > self.tuning.hearing {
            return None;
        }
        let way = self.relative(origin, cell);
        Some(format!("You hear footsteps to the {way}."))
    }

    fn hears_footsteps(&self, origin: CellId, event: &crate::model::Event) -> bool {
        let Some(cell) = self.walk_arrival(event) else {
            return false;
        };
        chebyshev(origin, cell) > self.tuning.sight
            && chebyshev(origin, cell) <= self.tuning.hearing
    }

    /// The cell someone arrived on. Only a walk or a wade that succeeded.
    fn walk_arrival(&self, event: &crate::model::Event) -> Option<CellId> {
        let crate::model::Scope::Cell(cell) = event.scope else {
            return None;
        };
        if event.affected_actors.len() != 1 {
            return None;
        }
        let name = self.name_of(event.affected_actors[0]);
        let text = event.text.as_str();
        if text.starts_with(&format!("{name} walks "))
            || text.starts_with(&format!("{name} wades "))
        {
            Some(cell)
        } else {
            None
        }
    }

    /// A fish taken out of sight, by a spear or by a set net. The catch
    /// sentence stays with someone who can see it. Past hearing, the event
    /// is not delivered at all.
    fn splash_for_these_ears(&self, origin: CellId, event: &crate::model::Event) -> Option<String> {
        let cell = self.splash_cell(event)?;
        if chebyshev(origin, cell) <= self.tuning.sight {
            return None;
        }
        if chebyshev(origin, cell) > self.tuning.hearing {
            return None;
        }
        let way = self.relative(origin, cell);
        Some(format!("You hear a splash to the {way}."))
    }

    /// A beaver returning to the brook, heard from out of sight. Someone who
    /// can see that water keeps the sentence. Past hearing, nothing is delivered.
    fn beaver_for_these_ears(&self, origin: CellId, event: &crate::model::Event) -> Option<String> {
        let cell = self.beaver_return_cell(event)?;
        if chebyshev(origin, cell) <= self.tuning.sight {
            return None;
        }
        if chebyshev(origin, cell) > self.tuning.hearing {
            return None;
        }
        let way = self.relative(origin, cell);
        Some(format!("You hear a beaver to the {way}."))
    }

    fn hears_a_beaver(&self, origin: CellId, event: &crate::model::Event) -> bool {
        let Some(cell) = self.beaver_return_cell(event) else {
            return false;
        };
        chebyshev(origin, cell) > self.tuning.sight
            && chebyshev(origin, cell) <= self.tuning.hearing
    }

    fn beaver_return_cell(&self, event: &crate::model::Event) -> Option<CellId> {
        if event.text != "A beaver is in the brook again." {
            return None;
        }
        event.affected_cells.first().copied()
    }

    /// A fox slipping away, or one returning to the floor, heard from out of
    /// sight. Someone who can see the cell keeps that sentence. Past hearing,
    /// nothing is delivered.
    fn fox_for_these_ears(&self, origin: CellId, event: &crate::model::Event) -> Option<String> {
        let cell = self.fox_slip_cell(event)?;
        if chebyshev(origin, cell) <= self.tuning.sight {
            return None;
        }
        if chebyshev(origin, cell) > self.tuning.hearing {
            return None;
        }
        let way = self.relative(origin, cell);
        Some(format!("You hear a fox to the {way}."))
    }

    /// A hare bolting, or one returning to the floor, heard from out of sight.
    /// Someone who can see the cell keeps that sentence. Past hearing, nothing
    /// is delivered.
    fn hare_for_these_ears(&self, origin: CellId, event: &crate::model::Event) -> Option<String> {
        let cell = self.hare_bolt_cell(event)?;
        if chebyshev(origin, cell) <= self.tuning.sight {
            return None;
        }
        if chebyshev(origin, cell) > self.tuning.hearing {
            return None;
        }
        let way = self.relative(origin, cell);
        Some(format!("You hear a hare to the {way}."))
    }

    fn hears_a_hare(&self, origin: CellId, event: &crate::model::Event) -> bool {
        let Some(cell) = self.hare_bolt_cell(event) else {
            return false;
        };
        chebyshev(origin, cell) > self.tuning.sight
            && chebyshev(origin, cell) <= self.tuning.hearing
    }

    fn hare_bolt_cell(&self, event: &crate::model::Event) -> Option<CellId> {
        let bolted = event.text.contains("startles and bolts")
            || event.text.starts_with("The hare bolts")
            || (event.text.contains("misses the hare") && event.text.contains("bolts"));
        if !bolted && event.text != "A hare is on the valley floor again." {
            return None;
        }
        event.affected_cells.first().copied()
    }

    fn hears_a_fox(&self, origin: CellId, event: &crate::model::Event) -> bool {
        let Some(cell) = self.fox_slip_cell(event) else {
            return false;
        };
        chebyshev(origin, cell) > self.tuning.sight
            && chebyshev(origin, cell) <= self.tuning.hearing
    }

    fn fox_slip_cell(&self, event: &crate::model::Event) -> Option<CellId> {
        if !event.text.contains("slips away") && event.text != "A fox is on the valley floor again."
        {
            return None;
        }
        event.affected_cells.first().copied()
    }

    fn hears_a_splash(&self, origin: CellId, event: &crate::model::Event) -> bool {
        let Some(cell) = self.splash_cell(event) else {
            return false;
        };
        chebyshev(origin, cell) > self.tuning.sight
            && chebyshev(origin, cell) <= self.tuning.hearing
    }

    fn splash_cell(&self, event: &crate::model::Event) -> Option<CellId> {
        let splash = event.text.contains(" spears a fish from the brook.")
            || event
                .text
                .contains(" catches a fish from the brook with the bone fish hook")
            || event
                .text
                .contains(" catches a fish from the shallows with the bone fish hook")
            || event
                .text
                .contains(" catches a fish from the hole in the ice with the bone fish hook")
            || event.text == "The fishing net catches a fish."
            || event.text.contains("splashes away");
        if !splash {
            return None;
        }
        event.affected_cells.first().copied()
    }

    /// A branch held or set to dry, and the moment it is dry, for the person who did it.
    fn dry_for_these_eyes(&self, viewer: ActorId, event: &crate::model::Event) -> Option<String> {
        if event.affected_actors.len() != 1 || event.affected_actors[0] != viewer {
            return None;
        }
        let name = self.name_of(viewer);
        if let Some(rest) = event.text.strip_prefix(&format!("{name} holds ")) {
            if rest.starts_with("the soaked branch by the fire.")
                || rest.starts_with("the damp branch by the fire.")
            {
                return Some(crate::model::spoken_act(None, "hold", rest));
            }
        }
        if let Some(rest) = event.text.strip_prefix(&format!("{name} sets ")) {
            if rest.starts_with("a soaked branch by the fire.")
                || rest.starts_with("a damp branch by the fire.")
            {
                return Some(crate::model::spoken_act(None, "set", rest));
            }
        }
        if event.text == format!("{name} dries the branch at the fire.") {
            return Some("You dry the branch at the fire.".into());
        }
        None
    }

    /// Starting to dry clothes. The person who said it already has the reply,
    /// so their event list does not repeat it. Anyone else keeps the sentence.
    fn clothes_for_these_eyes(
        &self,
        viewer: ActorId,
        event: &crate::model::Event,
    ) -> Option<String> {
        if event.affected_actors.len() != 1 || event.affected_actors[0] != viewer {
            return None;
        }
        let name = self.name_of(viewer);
        let rest = event.text.strip_prefix(&format!("{name} holds their "))?;
        if rest == "wet clothes by the fire. They are not dry yet."
            || rest == "damp clothes by the fire. They are not dry yet."
            || rest == "soaked clothes by the fire. They are not dry yet."
        {
            return Some(format!("You hold your {rest}"));
        }
        None
    }

    /// Clothes that have finished drying, in the first person for the wearer.
    fn clothes_dry_for_these_eyes(
        &self,
        viewer: ActorId,
        event: &crate::model::Event,
    ) -> Option<String> {
        if event.affected_actors.len() != 1 || event.affected_actors[0] != viewer {
            return None;
        }
        let name = self.name_of(viewer);
        if event.text == format!("{name}'s clothes dry at the fire.") {
            return Some("Your clothes dry at the fire.".into());
        }
        None
    }

    /// A successful haft, filled for the person who bound it.
    /// Anyone else keeps the stored sentence.
    fn haft_for_these_eyes(&self, viewer: ActorId, event: &crate::model::Event) -> Option<String> {
        if event.affected_actors.len() != 1 || event.affected_actors[0] != viewer {
            return None;
        }
        let name = self.name_of(viewer);
        let rest = event.text.strip_prefix(&format!("{name} hafts "))?;
        if rest != "the flint flake and a dry branch into a spear."
            && rest != "the flint flake and a dry branch into a spear. The tip is bound with hide."
            && rest
                != "the flint flake and a dry branch into a spear. The tip is bound with willow cordage."
            && rest
                != "the flint flake and a dry branch into a spear. The tip is bound with nettle cordage."
        {
            return None;
        }
        Some(crate::model::spoken_act(None, "haft", rest))
    }

    /// Food eaten. The eater already has the reply, so the event list does
    /// not repeat it. Someone who can see keeps the third person. A hunger
    /// line is a different event and stays with the eater.
    fn eat_for_these_eyes(&self, viewer: ActorId, event: &crate::model::Event) -> Option<String> {
        if event.affected_actors.len() != 1 || event.affected_actors[0] != viewer {
            return None;
        }
        let name = self.name_of(viewer);
        let rest = event.text.strip_prefix(&format!("{name} eats "))?;
        const RESTS: &[&str] = &[
            "the honeycomb.",
            "the cooked meat.",
            "the salted meat.",
            "the smoked meat.",
            "the cooked fish.",
            "the salted fish.",
            "the smoked fish.",
            "the cooked mushroom.",
            "the cooked cattail roots.",
            "the cattail flatbread.",
            "the cooked eggs.",
            "the cooked pale berries.",
            "a handful of pale berries.",
            "a handful of berries.",
            "a handful of dark berries.",
        ];
        if RESTS.contains(&rest) {
            Some(event.text.clone())
        } else {
            None
        }
    }

    /// A successful knap. The knapper already has the reply, so the event
    /// list does not repeat it. Someone who can see keeps the third person.
    fn knap_for_these_eyes(&self, viewer: ActorId, event: &crate::model::Event) -> Option<String> {
        if event.affected_actors.len() != 1 || event.affected_actors[0] != viewer {
            return None;
        }
        let name = self.name_of(viewer);
        let rest = event.text.strip_prefix(&format!("{name} knaps "))?;
        if rest != "the flint nodule into a sharp flake." {
            return None;
        }
        Some(crate::model::spoken_act(None, "knap", rest))
    }

    /// A copper bead hammered into an awl, or a bloom hammered into a knife.
    /// The smith already has the reply, so the event list does not repeat it.
    /// Someone who can see keeps the stored third person.
    fn copper_awl_for_these_eyes(
        &self,
        viewer: ActorId,
        event: &crate::model::Event,
    ) -> Option<String> {
        if event.affected_actors.len() != 1 || event.affected_actors[0] != viewer {
            return None;
        }
        let name = self.name_of(viewer);
        let rest = event.text.strip_prefix(&format!("{name} hammers "))?;
        if rest != "the copper bead into a copper awl at the fire."
            && rest != "the copper bead into a copper awl with a stone."
            && rest != "the bloom of iron into an iron knife at the fire."
            && rest != "the bloom of iron into an iron knife with a stone."
        {
            return None;
        }
        Some(crate::model::spoken_act(None, "hammer", rest))
    }

    /// A hide pierced with the awl, filled for the person who pierced it.
    /// Anyone else keeps the stored sentence.
    fn pierce_for_these_eyes(
        &self,
        viewer: ActorId,
        event: &crate::model::Event,
    ) -> Option<String> {
        if event.affected_actors.len() != 1 || event.affected_actors[0] != viewer {
            return None;
        }
        let name = self.name_of(viewer);
        let rest = event.text.strip_prefix(&format!("{name} pierces "))?;
        if rest != "the deer hide with the copper awl. It is ready to sew."
            && rest != "the deer hide with the bone awl. It is ready to sew."
        {
            return None;
        }
        Some(crate::model::spoken_act(None, "pierce", rest))
    }

    /// Raw meat rubbed with salt. The person who salted it already has the reply.
    fn salt_for_these_eyes(&self, viewer: ActorId, event: &crate::model::Event) -> Option<String> {
        if event.affected_actors.len() != 1 || event.affected_actors[0] != viewer {
            return None;
        }
        let name = self.name_of(viewer);
        let rest = event.text.strip_prefix(&format!("{name} salts "))?;
        if rest != "a portion of raw meat. It will keep, and it can be eaten cold."
            && rest != "a smaller piece of raw meat. It will keep, and it can be eaten cold."
            && rest != "a raw fish. It will keep, and it can be eaten cold."
        {
            return None;
        }
        Some(crate::model::spoken_act(None, "salt", rest))
    }

    /// A bone carved into an awl. The carver already has this as the reply.
    fn carve_for_these_eyes(&self, viewer: ActorId, event: &crate::model::Event) -> Option<String> {
        if event.affected_actors.len() != 1 || event.affected_actors[0] != viewer {
            return None;
        }
        let name = self.name_of(viewer);
        let rest = event.text.strip_prefix(&format!("{name} carves "))?;
        if rest != "a bone into a bone awl with the iron knife."
            && rest != "a bone into a bone awl with the flint flake."
            && rest != "a dry branch into a wooden paddle with the iron knife."
            && rest != "a dry branch into a wooden paddle with the flint flake."
            && rest != "a bone into a bone fish hook with the iron knife."
            && rest != "a bone into a bone fish hook with the flint flake."
            && rest != "a bow drill from two dry branches and hide strips."
            && rest != "a bow drill from two dry branches and willow cordage."
            && rest != "a bow drill from two dry branches and nettle cordage."
            && rest != "the spent bow drill anew with one dry branch."
            && rest != "a bone into a bone whistle with the iron knife."
            && rest != "a bone into a bone whistle with the flint flake."
            && rest != "a bundle of reeds into a reed flute with the iron knife."
            && rest != "a bundle of reeds into a reed flute with the flint flake."
        {
            return None;
        }
        Some(crate::model::spoken_act(None, "carve", rest))
    }

    /// The person who blew the whistle already has the reply.
    fn blow_for_these_eyes(&self, viewer: ActorId, event: &crate::model::Event) -> Option<String> {
        if event.affected_actors.len() != 1 || event.affected_actors[0] != viewer {
            return None;
        }
        let name = self.name_of(viewer);
        if event.text == format!("{name} blows the bone whistle.") {
            Some("You blow the bone whistle.".into())
        } else if event.text == format!("{name} blows the reed flute.") {
            Some("You blow the reed flute.".into())
        } else {
            None
        }
    }

    /// The person who sewed or beat the drum already has the reply.
    fn drum_for_these_eyes(&self, viewer: ActorId, event: &crate::model::Event) -> Option<String> {
        if event.affected_actors.len() != 1 || event.affected_actors[0] != viewer {
            return None;
        }
        let name = self.name_of(viewer);
        if event.text == format!("{name} beats the drum.") {
            return Some("You beat the drum.".into());
        }
        let rest = event.text.strip_prefix(&format!("{name} sews "))?;
        if rest.contains("over a hoop of dry branch") {
            Some(format!("You sew {rest}"))
        } else {
            None
        }
    }

    /// A hide scraped with a flint flake or a stone. The scraper already has the reply.
    fn scrape_for_these_eyes(
        &self,
        viewer: ActorId,
        event: &crate::model::Event,
    ) -> Option<String> {
        if event.affected_actors.len() != 1 || event.affected_actors[0] != viewer {
            return None;
        }
        let name = self.name_of(viewer);
        event
            .text
            .starts_with(&format!("{name} scrapes "))
            .then(|| event.text.clone())
    }

    /// A hide pierced with a bone awl. The copper awl still reaches the piercer.
    fn bone_pierce_for_these_eyes(
        &self,
        viewer: ActorId,
        event: &crate::model::Event,
    ) -> Option<String> {
        let line = self.pierce_for_these_eyes(viewer, event)?;
        line.contains("bone awl").then_some(line)
    }

    /// A pierced hide sewn into a cloak, filled for the person who sewed it.
    /// Anyone else keeps the stored sentence.
    fn sew_for_these_eyes(&self, viewer: ActorId, event: &crate::model::Event) -> Option<String> {
        if event.affected_actors.len() != 1 || event.affected_actors[0] != viewer {
            return None;
        }
        let name = self.name_of(viewer);
        let rest = event.text.strip_prefix(&format!("{name} sews "))?;
        if rest != "the pierced deer hide into a cloak." {
            return None;
        }
        Some(crate::model::spoken_act(None, "sew", rest))
    }

    /// Rest under shelter, filled for the person who lay down.
    /// Anyone else keeps the stored sentence. The open wind never starts one.
    fn rest_for_these_eyes(&self, viewer: ActorId, event: &crate::model::Event) -> Option<String> {
        if event.affected_actors.len() != 1 || event.affected_actors[0] != viewer {
            return None;
        }
        let name = self.name_of(viewer);
        let rest = event.text.strip_prefix(&format!("{name} rests"))?;
        if rest != "." && rest != " under the rock." && rest != " under the lean-to." {
            return None;
        }
        Some(format!("You rest{rest}"))
    }

    /// A lump of copper ore set by the fire. The person who set it already
    /// has the reply, so the event list does not repeat the shorter line.
    /// Someone who can see it keeps the stored sentence.
    fn copper_set_for_these_eyes(
        &self,
        viewer: ActorId,
        event: &crate::model::Event,
    ) -> Option<String> {
        if event.affected_actors.len() != 1 || event.affected_actors[0] != viewer {
            return None;
        }
        let name = self.name_of(viewer);
        if event.text
            != format!("{name} sets a lump of copper ore by the fire. It leaves their arms.")
        {
            return None;
        }
        Some(
            "You set a lump of copper ore by the fire. It leaves your arms. It is not metal yet."
                .into(),
        )
    }

    /// A lump of iron ore set by the fire, filled for the person who set it.
    /// Anyone else keeps the stored sentence. Copper is the reply only.
    fn smelt_set_for_these_eyes(
        &self,
        viewer: ActorId,
        event: &crate::model::Event,
    ) -> Option<String> {
        if event.affected_actors.len() != 1 || event.affected_actors[0] != viewer {
            return None;
        }
        let name = self.name_of(viewer);
        let rest = event.text.strip_prefix(&format!("{name} sets "))?;
        if rest != "a lump of iron ore by the fire. It leaves their arms." {
            return None;
        }
        Some("You set a lump of iron ore by the fire. It leaves your arms.".into())
    }

    /// Branches banked on a fire. The banker already has the reply.
    fn bank_for_these_eyes(&self, viewer: ActorId, event: &crate::model::Event) -> Option<String> {
        if event.affected_actors.len() != 1 || event.affected_actors[0] != viewer {
            return None;
        }
        let name = self.name_of(viewer);
        let rest = event.text.strip_prefix(&format!("{name} banks "))?;
        if rest != "the branches on the fire and cover them with mud." {
            return None;
        }
        Some(crate::model::spoken_act(None, "bank", rest))
    }

    /// A dry branch fed into a fire, filled for the person who fed it.
    fn feed_for_these_eyes(&self, viewer: ActorId, event: &crate::model::Event) -> Option<String> {
        if event.affected_actors.len() != 1 || event.affected_actors[0] != viewer {
            return None;
        }
        let name = self.name_of(viewer);
        let rest = event.text.strip_prefix(&format!("{name} feeds "))?;
        if rest != "a dry branch into the fire. It burns hotter."
            && rest != "a lump of charcoal into the fire. It burns hotter."
        {
            return None;
        }
        Some(crate::model::spoken_act(None, "feed", rest))
    }

    /// A drop. The person who dropped it already has the reply.
    /// The event list does not repeat the third person. A witness keeps it.
    fn drop_for_these_eyes(&self, viewer: ActorId, event: &crate::model::Event) -> Option<String> {
        if event.affected_actors.len() != 1 || event.affected_actors[0] != viewer {
            return None;
        }
        let name = self.name_of(viewer);
        let rest = event.text.strip_prefix(&format!("{name} drops "))?;
        if !rest.contains("in contact with the ground.") {
            return None;
        }
        Some(event.text.clone())
    }

    /// A successful gather. The gatherer already has this as the reply,
    /// so the event list does not repeat it. Anyone else keeps the stored sentence.
    fn gather_for_these_eyes(
        &self,
        viewer: ActorId,
        event: &crate::model::Event,
    ) -> Option<String> {
        if event.affected_actors.len() != 1 || event.affected_actors[0] != viewer {
            return None;
        }
        let name = self.name_of(viewer);
        let rest = event.text.strip_prefix(&format!("{name} gathers "))?;
        let rest = rest.strip_suffix(" The bees sting.").unwrap_or(rest);
        let rest = rest.strip_suffix(" The nettles sting.").unwrap_or(rest);
        let bare = rest
            .strip_suffix(". You hear branches breaking.")
            .unwrap_or(rest)
            .strip_suffix('.')?;
        let (article, noun) = if let Some(noun) = bare.strip_prefix("an ") {
            ("an", noun)
        } else if let Some(noun) = bare.strip_prefix("a ") {
            ("a", noun)
        } else {
            ("", bare)
        };
        const NOUNS: &[&str] = &[
            "branch",
            "sheet of bark",
            "log",
            "handful of berries",
            "mushroom",
            "handful of poison berries",
            "handful of dark berries",
            "handful of pale berries",
            "pot",
            "stone",
            "lump of clay",
            "unfired clay pot",
            "bowl",
            "spear",
            "carcass",
            "deer hide",
            "wolf hide",
            "otter hide",
            "fox hide",
            "hare hide",
            "beaver hide",
            "bone",
            "drink of water",
            "flint nodule",
            "lump of ochre",
            "lump of copper ore",
            "lump of iron ore",
            "bloom of iron",
            "iron knife",
            "copper bead",
            "copper awl",
            "lump of charcoal",
            "bundle of reeds",
            "handful of mud",
            "hide strips",
            "honeycomb",
            "the eggs",
            "handful of moss",
            "handful of mint",
            "handful of pine needles",
            "salt crust",
            "lump of pine resin",
            "bundle of willow cordage",
            "strip of willow bark",
            "bundle of nettle cordage",
            "cattail roots",
            "handful of cattail fluff",
            "worm",
            "clear stone of quartz",
            "reed flute",
            "cattail flour",
            "cattail flatbread",
            "bark cup",
            "sealed bark cup",
            "soaked bark cup",
            "fishing net",
            "reed mat",
        ];
        let known = NOUNS
            .iter()
            .copied()
            .any(|word| noun == word || noun.starts_with(&format!("{word} from ")));
        if !known {
            return None;
        }
        let filled = if article.is_empty() {
            format!("{noun}.")
        } else {
            format!("{article} {noun}.")
        };
        Some(crate::model::spoken_act(None, "gather", &filled))
    }

    /// A dry branch gathered out of sight. The gather sentence stays with
    /// someone who can see it. Past hearing, the event is not delivered at all.
    fn branch_break_for_these_ears(
        &self,
        origin: CellId,
        event: &crate::model::Event,
    ) -> Option<String> {
        let cell = self.branch_break_cell(event)?;
        if chebyshev(origin, cell) <= self.tuning.sight {
            return None;
        }
        if chebyshev(origin, cell) > self.tuning.hearing {
            return None;
        }
        let way = self.relative(origin, cell);
        Some(format!("You hear branches breaking to the {way}."))
    }

    fn hears_a_branch_break(&self, origin: CellId, event: &crate::model::Event) -> bool {
        let Some(cell) = self.branch_break_cell(event) else {
            return false;
        };
        chebyshev(origin, cell) > self.tuning.sight
            && chebyshev(origin, cell) <= self.tuning.hearing
    }

    /// The cell of a gathered branch, the act that already says the snap.
    fn branch_break_cell(&self, event: &crate::model::Event) -> Option<CellId> {
        if !event.text.contains(" gathers a branch.") {
            return None;
        }
        event.affected_cells.first().copied()
    }

    /// A successful steep. The stored sentence names the actor and has no
    /// stomach words. The person who drank already has the reply, so
    /// `recent_events` drops it. A witness keeps the act.
    fn steep_reply_for_these_eyes(
        &self,
        viewer: ActorId,
        event: &crate::model::Event,
    ) -> Option<String> {
        if event.affected_actors.len() != 1 || event.affected_actors[0] != viewer {
            return None;
        }
        let name = self.name_of(viewer);
        let rest = event.text.strip_prefix(&format!("{name} steeps "))?;
        let tea = rest
            .strip_prefix("a handful of mint ")
            .or_else(|| rest.strip_prefix("a handful of pine needles "))
            .or_else(|| rest.strip_prefix("a strip of willow bark "))?;
        if tea.starts_with("in the pot ")
            || tea.starts_with("in the bark cup ")
            || tea.starts_with("in the sealed bark cup ")
        {
            Some(event.text.clone())
        } else {
            None
        }
    }

    /// A successful boil, filled for the person who drank it.
    /// The stored sentence names them and keeps the thirst words on the end.
    /// Anyone else hears the act without that private ending.
    /// The person who boiled already has the reply, so `recent_events`
    /// drops it. A witness keeps the act.
    fn boil_reply_for_these_eyes(
        &self,
        viewer: ActorId,
        event: &crate::model::Event,
    ) -> Option<String> {
        if event.affected_actors.first().copied() != Some(viewer) {
            return None;
        }
        self.boil_for_these_eyes(viewer, event)
    }

    /// A successful boil, filled for the person who drank it.
    /// The stored sentence names them and keeps the thirst words on the end.
    /// Anyone else hears the act without that private ending.
    fn boil_for_these_eyes(&self, viewer: ActorId, event: &crate::model::Event) -> Option<String> {
        if event.affected_actors.len() != 1 {
            return None;
        }
        let (public, coda) = Self::split_boil_coda(&event.text);
        let actor_id = event.affected_actors[0];
        let name = self.name_of(actor_id);
        let rest = public.strip_prefix(&format!("{name} boils "))?;
        let you_rest = match rest {
            "the drink in the pot over the fire and drinks it." => {
                "the drink in the pot over the fire and drink it."
            }
            "the drink in the bark cup over the fire and drinks it." => {
                "the drink in the bark cup over the fire and drink it."
            }
            "the drink in the sealed bark cup over the fire and drinks it." => {
                "the drink in the sealed bark cup over the fire and drink it."
            }
            "a drink of water over the fire and drinks it." => {
                "a drink of water over the fire and drink it."
            }
            _ => return None,
        };
        if viewer == actor_id {
            Some(format!(
                "{}{coda}",
                crate::model::spoken_act(None, "boil", you_rest)
            ))
        } else {
            Some(public.to_string())
        }
    }

    /// An otter hide pierced, sewn, worn, or taken off. The person who did it
    /// already read the sentence as the command result. A refusal stays with them.
    fn otter_cloak_for_these_eyes(
        &self,
        viewer: ActorId,
        event: &crate::model::Event,
    ) -> Option<String> {
        if event.affected_actors.len() != 1 || event.affected_actors[0] != viewer {
            return None;
        }
        let failures = [
            "The otter hide is not pierced.",
            "The otter hide is not scraped.",
            "The otter hide is already scraped.",
            "The otter hide is already pierced.",
            "The otter hide is already a cloak.",
            "You are not wearing an otter hide.",
            "You are not carrying an otter hide.",
            "You need a copper awl and an otter hide.",
        ];
        if failures.iter().any(|line| line == &event.text) {
            return Some(event.text.clone());
        }
        let name = self.name_of(viewer);
        let lines = [
            format!("{name} pierces the otter hide with the copper awl. It is ready to sew."),
            format!("{name} pierces the otter hide with the bone awl. It is ready to sew."),
            format!("{name} sews the pierced otter hide into a cloak."),
            format!("{name} wears the sewn otter-hide cloak."),
            format!("{name} wears the otter hide."),
            format!("{name} removes the sewn otter-hide cloak."),
            format!("{name} removes the otter hide."),
        ];
        if lines.iter().any(|line| line == &event.text) {
            Some(event.text.clone())
        } else {
            None
        }
    }

    /// A beaver hide pierced, sewn, worn, or taken off. The person who did it
    /// already read the sentence as the command result. A refusal stays with them.
    fn beaver_cloak_for_these_eyes(
        &self,
        viewer: ActorId,
        event: &crate::model::Event,
    ) -> Option<String> {
        if event.affected_actors.len() != 1 || event.affected_actors[0] != viewer {
            return None;
        }
        let failures = [
            "The beaver hide is not pierced.",
            "The beaver hide is not scraped.",
            "The beaver hide is already scraped.",
            "The beaver hide is already pierced.",
            "The beaver hide is already a cloak.",
            "You are not wearing a beaver hide.",
            "You are not carrying a beaver hide.",
            "You need a copper awl and a beaver hide.",
        ];
        if failures.iter().any(|line| line == &event.text) {
            return Some(event.text.clone());
        }
        let name = self.name_of(viewer);
        let lines = [
            format!("{name} pierces the beaver hide with the copper awl. It is ready to sew."),
            format!("{name} pierces the beaver hide with the bone awl. It is ready to sew."),
            format!("{name} sews the pierced beaver hide into a cloak."),
            format!("{name} wears the sewn beaver-hide cloak."),
            format!("{name} wears the beaver hide."),
            format!("{name} removes the sewn beaver-hide cloak."),
            format!("{name} removes the beaver hide."),
        ];
        if lines.iter().any(|line| line == &event.text) {
            Some(event.text.clone())
        } else {
            None
        }
    }

    /// A hare hide pierced, sewn, worn, or taken off. The person who did it
    /// already read the sentence as the command result. A refusal stays with them.
    fn hare_cloak_for_these_eyes(
        &self,
        viewer: ActorId,
        event: &crate::model::Event,
    ) -> Option<String> {
        if event.affected_actors.len() != 1 || event.affected_actors[0] != viewer {
            return None;
        }
        let failures = [
            "The hare hide is not pierced.",
            "The hare hide is not scraped.",
            "The hare hide is already scraped.",
            "The hare hide is already pierced.",
            "The hare hide is already a cloak.",
            "You are not wearing a hare hide.",
            "You are not carrying a hare hide.",
            "You need a copper awl and a hare hide.",
        ];
        if failures.iter().any(|line| line == &event.text) {
            return Some(event.text.clone());
        }
        let name = self.name_of(viewer);
        let lines = [
            format!("{name} pierces the hare hide with the copper awl. It is ready to sew."),
            format!("{name} pierces the hare hide with the bone awl. It is ready to sew."),
            format!("{name} sews the pierced hare hide into a cloak."),
            format!("{name} wears the sewn hare-hide cloak."),
            format!("{name} wears the hare hide."),
            format!("{name} removes the sewn hare-hide cloak."),
            format!("{name} removes the hare hide."),
        ];
        if lines.iter().any(|line| line == &event.text) {
            Some(event.text.clone())
        } else {
            None
        }
    }

    /// A fox hide pierced, sewn, worn, or taken off. The person who did it
    /// already read the sentence as the command result. A refusal stays with them.
    fn fox_cloak_for_these_eyes(
        &self,
        viewer: ActorId,
        event: &crate::model::Event,
    ) -> Option<String> {
        if event.affected_actors.len() != 1 || event.affected_actors[0] != viewer {
            return None;
        }
        let failures = [
            "The fox hide is not pierced.",
            "The fox hide is not scraped.",
            "The fox hide is already scraped.",
            "The fox hide is already pierced.",
            "The fox hide is already a cloak.",
            "You are not wearing a fox hide.",
            "You are not carrying a fox hide.",
            "You need a copper awl and a fox hide.",
        ];
        if failures.iter().any(|line| line == &event.text) {
            return Some(event.text.clone());
        }
        let name = self.name_of(viewer);
        let lines = [
            format!("{name} pierces the fox hide with the copper awl. It is ready to sew."),
            format!("{name} pierces the fox hide with the bone awl. It is ready to sew."),
            format!("{name} sews the pierced fox hide into a cloak."),
            format!("{name} wears the sewn fox-hide cloak."),
            format!("{name} wears the fox hide."),
            format!("{name} removes the sewn fox-hide cloak."),
            format!("{name} removes the fox hide."),
        ];
        if lines.iter().any(|line| line == &event.text) {
            Some(event.text.clone())
        } else {
            None
        }
    }

    /// A wolf hide pierced, sewn, worn, or taken off. The person who did it
    /// already read the sentence as the command result.
    fn wolf_cloak_for_these_eyes(
        &self,
        viewer: ActorId,
        event: &crate::model::Event,
    ) -> Option<String> {
        if event.affected_actors.len() != 1 || event.affected_actors[0] != viewer {
            return None;
        }
        let failures = [
            "The wolf hide is not pierced.",
            "The wolf hide is not scraped.",
            "The wolf hide is already scraped.",
            "The wolf hide is already pierced.",
            "The wolf hide is already a cloak.",
            "You are not wearing a wolf hide.",
            "You need a copper awl and a wolf hide.",
        ];
        if failures.iter().any(|line| line == &event.text) {
            return Some(event.text.clone());
        }
        let name = self.name_of(viewer);
        let lines = [
            format!("{name} pierces the wolf hide with the copper awl. It is ready to sew."),
            format!("{name} pierces the wolf hide with the bone awl. It is ready to sew."),
            format!("{name} sews the pierced wolf hide into a cloak."),
            format!("{name} wears the sewn wolf-hide cloak."),
            format!("{name} wears the wolf hide."),
            format!("{name} removes the sewn wolf-hide cloak."),
            format!("{name} removes the wolf hide."),
        ];
        if lines.iter().any(|line| line == &event.text) {
            Some(event.text.clone())
        } else {
            None
        }
    }

    /// A wolf brought down, or missed, filled for the hunter.
    /// That person already read the sentence as the command result.
    fn wolf_hunt_for_these_eyes(
        &self,
        viewer: ActorId,
        event: &crate::model::Event,
    ) -> Option<String> {
        if event.affected_actors.len() != 1 || event.affected_actors[0] != viewer {
            return None;
        }
        let name = self.name_of(viewer);
        let lines = [
            format!("{name} brings the wolf down. A wolf carcass lies where it stood."),
            format!("{name} brings the wolf down with the flint spear."),
            format!("{name} brings the wolf down with the bow. The flint flake is spent."),
            format!("{name} misses the wolf. It moves off."),
            format!("{name} misses the wolf with the flint spear. It moves off."),
            format!("The wolf bolts from {name}'s spear."),
            format!("The wolf bolts. {name}'s flint spear does not land."),
        ];
        if lines.iter().any(|line| line == &event.text) {
            Some(event.text.clone())
        } else {
            None
        }
    }

    /// A beaver brought down, or missed. The hunter already has the reply,
    /// so the event list does not repeat it. Someone who can see keeps
    /// the stored third person.
    fn beaver_hunt_for_these_eyes(
        &self,
        viewer: ActorId,
        event: &crate::model::Event,
    ) -> Option<String> {
        if event.affected_actors.len() != 1 || event.affected_actors[0] != viewer {
            return None;
        }
        let name = self.name_of(viewer);
        let text = event.text.as_str();
        let brings = text == format!("{name} brings the beaver down with the flint spear.")
            || text
                == format!("{name} brings the beaver down. A beaver carcass lies where it stood.")
            || text
                == format!("{name} brings the beaver down with the bow. The flint flake is spent.");
        let misses = text.starts_with(&format!("{name} misses the beaver"))
            && text.contains("splashes away");
        let bolts = text.starts_with("The beaver splashes away to the ")
            && (text.ends_with(&format!("{name}'s flint spear does not land."))
                || text.ends_with(&format!("{name}'s spear does not land.")));
        let stays = text == format!("{name} misses the beaver with the flint spear. It stays.")
            || text == format!("{name} misses the beaver. It stays.")
            || text == format!("The beaver stays. {name}'s flint spear does not land.")
            || text == format!("The beaver stays. {name}'s spear does not land.");
        if brings || misses || bolts || stays {
            Some(event.text.clone())
        } else {
            None
        }
    }

    /// An otter brought down, or missed, filled for the hunter.
    /// That person already read the sentence as the command result.
    fn otter_hunt_for_these_eyes(
        &self,
        viewer: ActorId,
        event: &crate::model::Event,
    ) -> Option<String> {
        if event.affected_actors.len() != 1 || event.affected_actors[0] != viewer {
            return None;
        }
        let name = self.name_of(viewer);
        let text = event.text.as_str();
        let brings = text == format!("{name} brings the otter down with the flint spear.")
            || text
                == format!("{name} brings the otter down. An otter carcass lies where it stood.")
            || text
                == format!("{name} brings the otter down with the bow. The flint flake is spent.");
        let misses =
            text.starts_with(&format!("{name} misses the otter")) && text.contains("splashes away");
        let bolts = text.starts_with("The otter splashes away to the ")
            && (text.ends_with(&format!("{name}'s flint spear does not land."))
                || text.ends_with(&format!("{name}'s spear does not land.")));
        if brings || misses || bolts {
            Some(event.text.clone())
        } else {
            None
        }
    }

    /// A fox brought down, or missed, filled for the hunter.
    /// That person already read the sentence as the command result.
    fn fox_hunt_for_these_eyes(
        &self,
        viewer: ActorId,
        event: &crate::model::Event,
    ) -> Option<String> {
        if event.affected_actors.len() != 1 || event.affected_actors[0] != viewer {
            return None;
        }
        let name = self.name_of(viewer);
        let text = event.text.as_str();
        let brings = text == format!("{name} brings the fox down with the flint spear.")
            || text == format!("{name} brings the fox down. A fox carcass lies where it stood.")
            || text
                == format!("{name} brings the fox down with the bow. The flint flake is spent.");
        let misses =
            text.starts_with(&format!("{name} misses the fox")) && text.contains("slips away");
        let bolts = text.starts_with("The fox slips away to the ")
            && (text.ends_with(&format!("{name}'s flint spear does not land."))
                || text.ends_with(&format!("{name}'s spear does not land.")));
        let stays = text == format!("{name} misses the fox with the flint spear. It stays.")
            || text == format!("{name} misses the fox. It stays.")
            || text == format!("The fox stays. {name}'s flint spear does not land.")
            || text == format!("The fox stays. {name}'s spear does not land.");
        if brings || misses || bolts || stays {
            Some(event.text.clone())
        } else {
            None
        }
    }

    /// A hare brought down, or missed, filled for the hunter.
    /// That person already read the sentence as the command result.
    fn hare_hunt_for_these_eyes(
        &self,
        viewer: ActorId,
        event: &crate::model::Event,
    ) -> Option<String> {
        if event.affected_actors.len() != 1 || event.affected_actors[0] != viewer {
            return None;
        }
        let name = self.name_of(viewer);
        let text = event.text.as_str();
        let brings = text == format!("{name} brings the hare down with the flint spear.")
            || text == format!("{name} brings the hare down. A hare carcass lies where it stood.")
            || text
                == format!("{name} brings the hare down with the bow. The flint flake is spent.");
        let misses = text.starts_with(&format!("{name} misses the hare")) && text.contains("bolts");
        let bolts = text.starts_with("The hare bolts to the ")
            && (text.ends_with(&format!("{name}'s flint spear does not land."))
                || text.ends_with(&format!("{name}'s spear does not land.")));
        let stays = text == format!("{name} misses the hare with the flint spear. It stays.")
            || text == format!("{name} misses the hare. It stays.")
            || text == format!("The hare stays. {name}'s flint spear does not land.")
            || text == format!("The hare stays. {name}'s spear does not land.");
        if brings || misses || bolts || stays {
            Some(event.text.clone())
        } else {
            None
        }
    }

    /// An otter carcass cut up, filled for the person who cut it.
    /// That person already read the sentence as the command result.
    fn otter_butcher_for_these_eyes(
        &self,
        viewer: ActorId,
        event: &crate::model::Event,
    ) -> Option<String> {
        if event.affected_actors.len() != 1 || event.affected_actors[0] != viewer {
            return None;
        }
        let name = self.name_of(viewer);
        let text = event.text.as_str();
        if text
            == format!("{name} butchers the otter carcass with the iron knife into a portion of raw meat, an otter hide, and a bone.")
            || text
                == format!("{name} butchers the otter carcass into a portion of raw meat, an otter hide, and a bone.")
        {
            Some(event.text.clone())
        } else {
            None
        }
    }

    /// A fox carcass cut up, filled for the person who cut it.
    /// That person already read the sentence as the command result.
    fn fox_butcher_for_these_eyes(
        &self,
        viewer: ActorId,
        event: &crate::model::Event,
    ) -> Option<String> {
        if event.affected_actors.len() != 1 || event.affected_actors[0] != viewer {
            return None;
        }
        let name = self.name_of(viewer);
        let text = event.text.as_str();
        if text
            == format!("{name} butchers the fox carcass with the iron knife into a portion of raw meat, a fox hide, and a bone.")
            || text
                == format!("{name} butchers the fox carcass into a portion of raw meat, a fox hide, and a bone.")
        {
            Some(event.text.clone())
        } else {
            None
        }
    }

    /// A hare carcass cut up, filled for the person who cut it.
    /// That person already read the sentence as the command result.
    fn hare_butcher_for_these_eyes(
        &self,
        viewer: ActorId,
        event: &crate::model::Event,
    ) -> Option<String> {
        if event.affected_actors.len() != 1 || event.affected_actors[0] != viewer {
            return None;
        }
        let name = self.name_of(viewer);
        let text = event.text.as_str();
        if text
            == format!("{name} butchers the hare carcass with the iron knife into a portion of raw meat, a hare hide, and a bone.")
            || text
                == format!(
                    "{name} butchers the hare carcass into a portion of raw meat, a hare hide, and a bone."
                )
        {
            Some(event.text.clone())
        } else {
            None
        }
    }

    /// A beaver carcass cut up, filled for the person who cut it.
    /// That person already read the sentence as the command result.
    fn beaver_butcher_for_these_eyes(
        &self,
        viewer: ActorId,
        event: &crate::model::Event,
    ) -> Option<String> {
        if event.affected_actors.len() != 1 || event.affected_actors[0] != viewer {
            return None;
        }
        let name = self.name_of(viewer);
        let text = event.text.as_str();
        if text
            == format!("{name} butchers the beaver carcass with the iron knife into a portion of raw meat, a beaver hide, and a bone.")
            || text
                == format!(
                    "{name} butchers the beaver carcass into a portion of raw meat, a beaver hide, and a bone."
                )
        {
            Some(event.text.clone())
        } else {
            None
        }
    }

    /// The fire sent the wolf off, filled for the person whose command did it.
    /// That person already read the sentence as the command result.
    fn scare_for_these_eyes(&self, viewer: ActorId, event: &crate::model::Event) -> Option<String> {
        if event.text != "The wolf flees the fire." {
            return None;
        }
        if event.affected_actors.len() == 1 && event.affected_actors[0] == viewer {
            Some("The wolf flees the fire.".into())
        } else {
            None
        }
    }

    /// A successful light, bark or pine resin, with or without a brand.
    /// The lighter already has the reply, so it does not sit in their events.
    /// Anyone else keeps the stored third-person sentence.
    /// The drinker already has the reply. The third person stays with
    /// someone who can see the cell.
    fn drink_for_these_eyes(&self, viewer: ActorId, event: &crate::model::Event) -> Option<String> {
        if event.affected_actors.len() != 1 || event.affected_actors[0] != viewer {
            return None;
        }
        let name = self.name_of(viewer);
        if event.text == format!("{name} drinks from the brook.")
            || event.text == format!("{name} drinks the still water.")
        {
            Some(event.text.clone())
        } else {
            None
        }
    }

    fn light_for_these_eyes(&self, viewer: ActorId, event: &crate::model::Event) -> Option<String> {
        if event.affected_actors.len() != 1 || event.affected_actors[0] != viewer {
            return None;
        }
        let name = self.name_of(viewer);
        let rest = event.text.strip_prefix(&format!("{name} lights "))?;
        if !rest.starts_with("a fire with ") || !rest.ends_with('.') {
            return None;
        }
        Some(event.text.clone())
    }

    /// The striker already has the reply. The third person stays with
    /// someone who can see the cell.
    fn strike_for_these_eyes(
        &self,
        viewer: ActorId,
        event: &crate::model::Event,
    ) -> Option<String> {
        if event.affected_actors.len() != 1 || event.affected_actors[0] != viewer {
            return None;
        }
        let name = self.name_of(viewer);
        let line = format!("{name} strikes the flint on the quartz. The sparks catch.");
        if event.text == line {
            Some(event.text.clone())
        } else {
            None
        }
    }

    /// An event keeps the weather of its own tick. The wood is named, not counted.
    fn heard_text(&self, viewer: ActorId, origin: CellId, event: &crate::model::Event) -> String {
        if let Some(line) = self.howl_for_these_ears(origin, event) {
            return line;
        }
        if let Some(line) = self.ice_for_these_ears(origin, event) {
            return line;
        }
        if let Some(line) = self.ice_break_for_these_ears(origin, event) {
            return line;
        }
        if let Some(line) = self.brush_for_these_ears(origin, event) {
            return line;
        }
        if let Some(line) = self.gather_for_these_eyes(viewer, event) {
            return line;
        }
        if let Some(line) = self.branch_break_for_these_ears(origin, event) {
            return line;
        }
        if let Some(line) = self.smelt_set_for_these_eyes(viewer, event) {
            return line;
        }
        if let Some(line) = self.feed_for_these_eyes(viewer, event) {
            return line;
        }
        if let Some(line) = self.knap_for_these_eyes(viewer, event) {
            return line;
        }
        if let Some(line) = self.pierce_for_these_eyes(viewer, event) {
            return line;
        }
        if let Some(line) = self.sew_for_these_eyes(viewer, event) {
            return line;
        }
        if let Some(line) = self.rest_for_these_eyes(viewer, event) {
            return line;
        }
        if let Some(line) = self.dry_for_these_eyes(viewer, event) {
            return line;
        }
        if let Some(line) = self.clothes_dry_for_these_eyes(viewer, event) {
            return line;
        }
        if let Some(line) = self.fish_for_these_eyes(viewer, event) {
            return line;
        }
        if let Some(line) = self.splash_for_these_ears(origin, event) {
            return line;
        }
        if let Some(line) = self.fox_for_these_ears(origin, event) {
            return line;
        }
        if let Some(line) = self.hare_for_these_ears(origin, event) {
            return line;
        }
        if let Some(line) = self.beaver_for_these_ears(origin, event) {
            return line;
        }
        if let Some(line) = self.walk_for_these_eyes(viewer, event) {
            return line;
        }
        if let Some(line) = self.footsteps_for_these_ears(origin, event) {
            return line;
        }
        if let Some(line) = self.brand_for_these_eyes(viewer, event) {
            return line;
        }
        if let Some(line) = self.harden_for_these_eyes(viewer, event) {
            return line;
        }
        if let Some(line) = self.wooden_point_for_these_eyes(viewer, event) {
            return line;
        }
        if let Some(line) = self.bring_for_these_eyes(viewer, event) {
            return line;
        }
        if let Some(line) = self.miss_for_these_eyes(viewer, event) {
            return line;
        }
        if let Some(line) = self.bolt_for_these_eyes(viewer, event) {
            return line;
        }
        if let Some(line) = self.butcher_for_these_eyes(viewer, event) {
            return line;
        }
        if let Some(line) = self.cut_for_these_eyes(viewer, event) {
            return line;
        }
        if let Some(line) = self.shape_for_these_eyes(viewer, event) {
            return line;
        }
        if let Some(line) = self.weave_for_these_eyes(viewer, event) {
            return line;
        }
        if let Some(line) = self.craft_for_these_eyes(viewer, event) {
            return line;
        }
        if let Some(line) = self.bow_hunt_for_these_eyes(viewer, event) {
            return line;
        }
        if let Some(line) = self.put_fish_for_these_eyes(viewer, event) {
            return line;
        }
        if let Some(line) = self.fire_pot_for_these_eyes(viewer, event) {
            return line;
        }
        if let Some(line) = self.lean_to_for_these_eyes(viewer, event) {
            return line;
        }
        if let Some(line) = self.thatch_for_these_eyes(viewer, event) {
            return line;
        }
        if let Some(line) = self.cook_meat_for_these_eyes(viewer, event) {
            return line;
        }
        if let Some(line) = self.cook_mushroom_for_these_eyes(viewer, event) {
            return line;
        }
        if let Some(line) = self.eat_meat_for_these_eyes(viewer, event) {
            return line;
        }
        if let Some(line) = self.stings_for_these_eyes(viewer, event) {
            return line;
        }
        if let Some(line) = self.eat_mushroom_for_these_eyes(viewer, event) {
            return line;
        }
        if let Some(line) = self.wear_for_these_eyes(viewer, event) {
            return line;
        }
        if let Some(line) = self.remove_for_these_eyes(viewer, event) {
            return line;
        }
        if let Some(line) = self.smear_for_these_eyes(viewer, event) {
            return line;
        }
        if let Some(line) = self.mark_for_these_eyes(viewer, event) {
            return line;
        }
        if let Some(line) = self.wash_for_these_eyes(viewer, event) {
            return line;
        }
        if let Some(line) = self.boil_for_these_eyes(viewer, event) {
            return line;
        }
        if event.text.starts_with("Rain reaches the valley") {
            let rain = if self.exposure_millis(origin) >= 1000 {
                "The rain wets your clothes.".to_string()
            } else {
                event.text.clone()
            };
            if self.fire_the_rain_misses(origin) {
                return format!("{rain} The rain does not reach the fire.");
            }
            return rain;
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
        self.bitter_heard(viewer, event.text.clone())
    }

    /// A stored line says "dark berries". Someone who has tasted them hears the name.
    fn bitter_heard(&self, viewer: ActorId, text: String) -> String {
        if self.bitter_known(viewer) {
            text.replace("dark berries", "poison berries")
        } else {
            text
        }
    }

    fn split_boil_coda(text: &str) -> (&str, &str) {
        for coda in [
            " The thirst eases.",
            " You were not thirsty.",
            " You are still thirsty.",
        ] {
            if let Some(public) = text.strip_suffix(coda) {
                return (public, coda);
            }
        }
        (text, "")
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
                if self.ice_forming(event) {
                    return self.hears_ice(origin, event);
                }
                if self.ice_break_marker(event) && self.hears_an_ice_break(origin, event) {
                    return true;
                }
                if self.brush_crack_marker(event) {
                    return self.hears_brush(origin, event);
                }
                chebyshev(origin, cell) <= self.tuning.sight
                    || event.affected_actors.contains(&viewer)
                    || self.hears_a_branch_break(origin, event)
                    || self.hears_a_splash(origin, event)
                    || self.hears_a_fox(origin, event)
                    || self.hears_a_hare(origin, event)
                    || self.hears_a_beaver(origin, event)
                    || self.hears_footsteps(origin, event)
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
            format!("examine {}", self.bitter_kind(actor_id)),
            "examine pale berries".into(),
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
            && (self.carry_count(actor_id) < self.tuning.carry_slots
                || self.basket_can_hold_berries(actor_id))
        {
            actions.push("gather berries".into());
        }
        if self
            .ground_ids(actor.cell, Material::Fish)
            .first()
            .and_then(|id| self.item(*id))
            .is_some_and(|fish| !fish.cooked)
            && (self.carry_count(actor_id) < self.tuning.carry_slots
                || self.basket_has_room(actor_id))
        {
            actions.push("gather fish".into());
        }
        if self.raw_fish_in_arms(actor_id).is_some() && self.basket_has_room(actor_id) {
            actions.push("put fish".into());
        }
        if self.loose_ids(actor_id, Material::Basket).is_empty()
            && (self.pile_count(actor.cell, Material::Bark) > 0
                || !self.loose_ids(actor_id, Material::Bark).is_empty())
            && (!self.loose_ids(actor_id, Material::Bark).is_empty()
                || self.carry_count(actor_id) < self.tuning.carry_slots)
        {
            actions.push("weave basket".into());
        }
        if self.has_material_at_hand(actor_id, Material::Berry) {
            actions.push("eat berries".into());
        }
        if self.raw_pale_near(actor_id) || self.cooked_pale_id(actor_id).is_some() {
            actions.push("eat pale berries".into());
        }
        if !self.loose_ids(actor_id, Material::Carcass).is_empty()
            || !self.ground_ids(actor.cell, Material::Carcass).is_empty()
        {
            actions.push("eat carcass".into());
        }
        if self.cooked_meat_id(actor_id).is_some() {
            actions.push("eat meat".into());
        }
        if self.cooked_mushroom_id(actor_id).is_some() {
            actions.push("eat mushrooms".into());
        }
        if self.pile_count(actor.cell, Material::BitterBerry) > 0
            && self.carry_count(actor_id) < self.tuning.carry_slots
        {
            actions.push(format!("gather {}", self.bitter_kind(actor_id)));
        }
        if self.pile_count(actor.cell, Material::PaleBerry) > 0
            && self.carry_count(actor_id) < self.tuning.carry_slots
        {
            actions.push("gather pale berries".into());
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
        if (self.pile_count(actor.cell, Material::Flint) > 0
            || !self.ground_ids(actor.cell, Material::Flint).is_empty())
            && self.carry_count(actor_id) < self.tuning.carry_slots
        {
            actions.push("gather flint".into());
        }
        if (self.pile_count(actor.cell, Material::Ochre) > 0
            || !self.ground_ids(actor.cell, Material::Ochre).is_empty())
            && self.carry_count(actor_id) < self.tuning.carry_slots
        {
            actions.push("gather ochre".into());
        }
        if (self.pile_count(actor.cell, Material::IronOre) > 0
            || !self.ground_ids(actor.cell, Material::IronOre).is_empty())
            && self.carry_count(actor_id) < self.tuning.carry_slots
        {
            actions.push("gather iron ore".into());
        }
        if (self.pile_count(actor.cell, Material::CopperOre) > 0
            || !self.ground_ids(actor.cell, Material::CopperOre).is_empty())
            && self.carry_count(actor_id) < self.tuning.carry_slots
        {
            actions.push("gather copper ore".into());
        }
        if (self.pile_count(actor.cell, Material::Copper) > 0
            || !self.ground_ids(actor.cell, Material::Copper).is_empty())
            && self.carry_count(actor_id) < self.tuning.carry_slots
        {
            actions.push("gather copper bead".into());
        }
        if (self.pile_count(actor.cell, Material::Awl) > 0
            || !self.ground_ids(actor.cell, Material::Awl).is_empty())
            && self.carry_count(actor_id) < self.tuning.carry_slots
        {
            actions.push("gather copper awl".into());
        }
        if actor.busy.is_none()
            && self.gripping(actor_id).is_none()
            && !self.loose_ids(actor_id, Material::Copper).is_empty()
            && (self.fire_at(actor.cell).is_some()
                || !self.loose_ids(actor_id, Material::Stone).is_empty())
        {
            actions.push("hammer".into());
        }
        if (self.pile_count(actor.cell, Material::Iron) > 0
            || !self.ground_ids(actor.cell, Material::Iron).is_empty())
            && self.carry_count(actor_id) < self.tuning.carry_slots
        {
            actions.push("gather bloom of iron".into());
        }
        if (self.pile_count(actor.cell, Material::Knife) > 0
            || !self.ground_ids(actor.cell, Material::Knife).is_empty())
            && self.carry_count(actor_id) < self.tuning.carry_slots
        {
            actions.push("gather iron knife".into());
        }
        if actor.busy.is_none()
            && self.gripping(actor_id).is_none()
            && !self.loose_ids(actor_id, Material::Iron).is_empty()
            && (self.fire_at(actor.cell).is_some_and(|fire| fire.hot)
                || !self.loose_ids(actor_id, Material::Stone).is_empty())
        {
            actions.push("hammer bloom".into());
        }
        if actor.busy.is_none()
            && self.gripping(actor_id).is_none()
            && self.fire_at(actor.cell).is_some_and(|fire| !fire.hot)
            && (!self.loose_ids(actor_id, Material::Charcoal).is_empty()
                || self.has_dry(actor_id, Material::Branch))
        {
            actions.push("feed".into());
        }
        if actor.busy.is_none()
            && self.gripping(actor_id).is_none()
            && self.fire_at(actor.cell).is_some()
            && !self.banks.contains_key(&actor.cell)
            && (!self.loose_ids(actor_id, Material::Mud).is_empty()
                || self.pile_count(actor.cell, Material::Mud) > 0
                || !self.ground_ids(actor.cell, Material::Mud).is_empty())
            && self.dry_wood_on_ground(actor.cell) > 0
        {
            actions.push("bank".into());
        }
        if actor.busy.is_none()
            && self.gripping(actor_id).is_none()
            && self.fire_at(actor.cell).is_some()
            && !self.loose_ids(actor_id, Material::CopperOre).is_empty()
        {
            actions.push("smelt".into());
        }
        if actor.busy.is_none()
            && self.gripping(actor_id).is_none()
            && self.fire_at(actor.cell).is_some_and(|fire| fire.charcoal)
            && !self.loose_ids(actor_id, Material::IronOre).is_empty()
        {
            actions.push("smelt iron".into());
        }
        if !self.loose_ids(actor_id, Material::Flint).is_empty() {
            actions.push("knap".into());
        }
        if !self.loose_ids(actor_id, Material::Flake).is_empty()
            && !self.loose_ids(actor_id, Material::Branch).is_empty()
        {
            actions.push("haft".into());
        }
        if !self.dry_loose_ids(actor_id, Material::Branch).is_empty() {
            actions.push("point branch".into());
        }
        if self.fire_at(actor.cell).is_some()
            && self.loose_ids(actor_id, Material::Spear).iter().any(|id| {
                self.item(*id)
                    .is_some_and(|spear| spear.wooden_tip && !spear.hardened)
            })
        {
            actions.push("harden spear".into());
        }
        if actor.busy.is_none()
            && self.gripping(actor_id).is_none()
            && self.bow_refusal(actor_id).is_none()
        {
            actions.push("craft bow".into());
        }
        if self.actors.values().any(|other| {
            chebyshev(actor.cell, other.cell) <= self.tuning.sight
                && !self.loose_ids(other.id, Material::Flake).is_empty()
        }) {
            actions.push("examine flake".into());
        }
        if (self.pile_count(actor.cell, Material::Clay) > 0
            || !self.ground_ids(actor.cell, Material::Clay).is_empty())
            && self.carry_count(actor_id) < self.tuning.carry_slots
        {
            actions.push("gather clay".into());
        }
        if self.clay_ready_to_shape(actor_id) {
            actions.push("shape clay".into());
        }
        if self.fire_at(actor.cell).is_some() && self.unfired_pot_ready(actor_id) {
            actions.push("fire pot".into());
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
        if !self.ground_ids(actor.cell, Material::Meat).is_empty()
            && self.carry_count(actor_id) < self.tuning.carry_slots
        {
            actions.push("gather meat".into());
        }
        if !self.ground_ids(actor.cell, Material::Hide).is_empty()
            && self.carry_count(actor_id) < self.tuning.carry_slots
        {
            actions.push("gather hide".into());
        }
        if (self
            .loose_ids(actor_id, Material::Spear)
            .iter()
            .any(|id| self.item(*id).is_some_and(|spear| spear.flint_tip))
            || !self.loose_ids(actor_id, Material::Knife).is_empty())
            && !self.ground_ids(actor.cell, Material::Carcass).is_empty()
        {
            actions.push("butcher".into());
        }
        if !self.loose_ids(actor_id, Material::Knife).is_empty()
            && (self
                .loose_ids(actor_id, Material::Meat)
                .iter()
                .any(|id| self.item(*id).is_some_and(|meat| !meat.small))
                || self
                    .ground_ids(actor.cell, Material::Meat)
                    .iter()
                    .any(|id| self.item(*id).is_some_and(|meat| !meat.small)))
        {
            actions.push("cut meat".into());
        }
        if !self.loose_ids(actor_id, Material::Knife).is_empty() && self.whole_hide_near(actor_id) {
            actions.push("cut hide".into());
        }
        if self.has_bone(actor_id) && self.has_carve_tool(actor_id) {
            actions.push("carve awl".into());
        }
        if self.paddle_branch_refusal(actor_id).is_none()
            && self.paddle_tool_rest(actor_id).is_some()
        {
            actions.push("carve paddle".into());
        }
        if self.has_bone(actor_id) && self.hook_tool_rest(actor_id).is_some() {
            actions.push("carve hook".into());
        }
        if actor.busy.is_none()
            && self.gripping(actor_id).is_none()
            && self.fishing_water(actor.cell)
            && !self.loose_ids(actor_id, Material::Hook).is_empty()
            && !self.loose_ids(actor_id, Material::Cord).is_empty()
            && !self
                .brook_stretch(actor.cell)
                .is_some_and(|stretch| self.brook_is_quiet(stretch))
        {
            actions.push("fish with hook".into());
        }
        if self.has_salt(actor_id) && self.raw_portion_to_salt(actor_id).is_some() {
            actions.push("salt meat".into());
        }
        if self.fire_at(actor.cell).is_some() && self.raw_portion_to_salt(actor_id).is_some() {
            actions.push("smoke meat".into());
        }
        if self.has_salt(actor_id) && self.raw_fish_id(actor_id).is_some() {
            actions.push("salt fish".into());
        }
        if (self.pile_count(actor.cell, Material::Salt) > 0
            || !self.ground_ids(actor.cell, Material::Salt).is_empty())
            && self.carry_count(actor_id) < self.tuning.carry_slots
        {
            actions.push("gather salt".into());
        }
        if (self.pile_count(actor.cell, Material::Honey) > 0
            || !self.ground_ids(actor.cell, Material::Honey).is_empty())
            && self.carry_count(actor_id) < self.tuning.carry_slots
        {
            actions.push("gather honey".into());
            if self.honey_nests.contains(&actor.cell)
                && !self.smoked_nests.contains(&actor.cell)
                && self.smoke_reaches(actor_id, actor.cell)
            {
                actions.push("smoke nest".into());
            }
        }
        if !self.loose_ids(actor_id, Material::Honey).is_empty() {
            actions.push("eat honey".into());
        }
        if (self.pile_count(actor.cell, Material::Egg) > 0
            || !self.ground_ids(actor.cell, Material::Egg).is_empty())
            && self.carry_count(actor_id) < self.tuning.carry_slots
        {
            actions.push("gather eggs".into());
        }
        if self.cooked_egg_id(actor_id).is_some() {
            actions.push("eat eggs".into());
        }
        if (self.pile_count(actor.cell, Material::Moss) > 0
            || !self.ground_ids(actor.cell, Material::Moss).is_empty())
            && self.carry_count(actor_id) < self.tuning.carry_slots
        {
            actions.push("gather moss".into());
        }
        if (self.pile_count(actor.cell, Material::Resin) > 0
            || !self.ground_ids(actor.cell, Material::Resin).is_empty())
            && self.carry_count(actor_id) < self.tuning.carry_slots
        {
            actions.push("gather resin".into());
        }
        if (self.pile_count(actor.cell, Material::Willow) > 0
            || !self.ground_ids(actor.cell, Material::Willow).is_empty())
            && self.carry_count(actor_id) < self.tuning.carry_slots
        {
            actions.push("strip willow".into());
        }
        if (self.pile_count(actor.cell, Material::Nettle) > 0
            || !self.ground_ids(actor.cell, Material::Nettle).is_empty())
            && self.carry_count(actor_id) < self.tuning.carry_slots
        {
            actions.push("strip nettle".into());
        }
        if !self.loose_ids(actor_id, Material::Resin).is_empty()
            && self
                .canoe_to_seal(actor_id)
                .is_some_and(|id| !self.canoe_pitched(id))
        {
            actions.push("seal canoe".into());
        }
        if !self.loose_ids(actor_id, Material::Resin).is_empty()
            && self
                .cup_to_seal(actor_id)
                .is_some_and(|id| self.item(id).is_some_and(|cup| !cup.pitched))
        {
            actions.push("seal bark cup".into());
        }
        if !self.loose_ids(actor_id, Material::Moss).is_empty() && actor.stings_until.is_some() {
            actions.push("press moss".into());
        }
        if !self.ground_ids(actor.cell, Material::Bone).is_empty()
            && self.carry_count(actor_id) < self.tuning.carry_slots
        {
            actions.push("gather bone".into());
        }
        if !self.snares.contains_key(&actor.cell) && self.has_snare_line(actor_id) {
            actions.push("set snare".into());
        }
        if self.cell(actor.cell).region == crate::model::Region::Brook
            && !self.weirs.contains_key(&actor.cell)
            && self.has_reeds(actor_id)
            && self.has_snare_line(actor_id)
        {
            actions.push("set weir".into());
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
        if !self.loose_ids(actor_id, Material::Bow).is_empty() {
            actions.push("drop bow".into());
        }
        if !self.ground_ids(actor.cell, Material::Bow).is_empty()
            && self.carry_count(actor_id) < self.tuning.carry_slots
        {
            actions.push("gather bow".into());
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
            actions.push(format!("drop {}", self.bitter_kind(actor_id)));
        }
        if !self.loose_ids(actor_id, Material::PaleBerry).is_empty() {
            actions.push("drop pale berries".into());
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
        if !self.loose_ids(actor_id, Material::Ochre).is_empty() {
            actions.push("drop ochre".into());
        }
        if !self.loose_ids(actor_id, Material::IronOre).is_empty() {
            actions.push("drop iron ore".into());
        }
        if !self.loose_ids(actor_id, Material::Iron).is_empty() {
            actions.push("drop bloom of iron".into());
        }
        if !self.loose_ids(actor_id, Material::Knife).is_empty() {
            actions.push("drop iron knife".into());
        }
        if !self.loose_ids(actor_id, Material::CopperOre).is_empty() {
            actions.push("drop copper ore".into());
        }
        if !self.loose_ids(actor_id, Material::Copper).is_empty() {
            actions.push("drop copper bead".into());
        }
        if !self.loose_ids(actor_id, Material::Awl).is_empty() {
            actions.push("drop copper awl".into());
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
        if !self.loose_ids(actor_id, Material::Hide).is_empty() && !self.wearing_hide(actor_id) {
            actions.push("wear hide".into());
        }
        if self.wearing_hide(actor_id) {
            actions.push("remove hide".into());
        }
        if !self.loose_ids(actor_id, Material::Awl).is_empty()
            && self
                .loose_ids(actor_id, Material::Hide)
                .iter()
                .any(|id| self.item(*id).is_some_and(|hide| !hide.pierced))
        {
            actions.push("pierce hide".into());
        }
        if self
            .loose_ids(actor_id, Material::Hide)
            .iter()
            .any(|id| self.item(*id).is_some_and(|hide| hide.pierced))
        {
            actions.push("sew hide".into());
        }
        if actor.busy.is_none()
            && self.gripping(actor_id).is_none()
            && (self.exposure_millis(actor.cell) < 1000 || self.reed_mat_here(actor.cell))
        {
            actions.push("rest".into());
        }
        if actor.busy.is_none()
            && self.gripping(actor_id).is_none()
            && self.fire_at(actor.cell).is_some()
            && actor.wetness > 0
            && actor.clothes_dry_at.is_none()
        {
            actions.push("dry clothes".into());
        }
        if !self.loose_ids(actor_id, Material::Hide).is_empty() {
            actions.push("drop hide".into());
        }
        let deer_dist = self
            .deer
            .as_ref()
            .map(|deer| chebyshev(actor.cell, deer.cell));
        let hardened_spear = self
            .loose_ids(actor_id, Material::Spear)
            .iter()
            .any(|id| self.item(*id).is_some_and(|spear| spear.hardened));
        let any_spear = !self.loose_ids(actor_id, Material::Spear).is_empty();
        let bow_ready = !self.loose_ids(actor_id, Material::Bow).is_empty()
            && !self.loose_ids(actor_id, Material::Flake).is_empty()
            && deer_dist.is_some_and(|dist| dist <= self.tuning.sight);
        let close_spear = hardened_spear && deer_dist.is_some_and(|dist| dist <= 1);
        let bow_past_the_spear =
            hardened_spear && bow_ready && deer_dist.is_some_and(|dist| dist > 1);
        let wolf_dist = self.wolf_standing().map(|cell| chebyshev(actor.cell, cell));
        let wolf_spear = hardened_spear && wolf_dist.is_some_and(|dist| dist <= 1);
        let wolf_bow = !self.loose_ids(actor_id, Material::Bow).is_empty()
            && !self.loose_ids(actor_id, Material::Flake).is_empty()
            && wolf_dist.is_some_and(|dist| dist <= self.tuning.sight);
        let wolf_bow_past = hardened_spear && wolf_bow && wolf_dist.is_some_and(|dist| dist > 1);
        let otter_dist = self
            .otter
            .as_ref()
            .map(|otter| chebyshev(actor.cell, otter.cell));
        let fox_dist = self.fox.as_ref().map(|fox| chebyshev(actor.cell, fox.cell));
        let fox_nearer = fox_dist.is_some_and(|dist| {
            deer_dist.is_none_or(|far| dist < far)
                && otter_dist.is_none_or(|far| dist < far)
                && wolf_dist.is_none_or(|far| dist < far)
        });
        let fox_spear = hardened_spear && fox_dist.is_some_and(|dist| dist <= 1) && fox_nearer;
        let fox_bow = !self.loose_ids(actor_id, Material::Bow).is_empty()
            && !self.loose_ids(actor_id, Material::Flake).is_empty()
            && fox_dist.is_some_and(|dist| dist <= self.tuning.sight)
            && fox_nearer;
        let fox_bow_past = hardened_spear && fox_bow && fox_dist.is_some_and(|dist| dist > 1);
        if close_spear
            || wolf_spear
            || fox_spear
            || (!any_spear && (bow_ready || wolf_bow || fox_bow))
            || bow_past_the_spear
            || wolf_bow_past
            || fox_bow_past
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
        if !self.loose_ids(actor_id, Material::Ochre).is_empty() {
            actions.push("smear ochre".into());
        }
        if (!self.loose_ids(actor_id, Material::Ochre).is_empty()
            || !self.loose_ids(actor_id, Material::Charcoal).is_empty())
            && !self.ochre_marks.contains(&actor.cell)
            && !self.charcoal_marks.contains(&actor.cell)
        {
            actions.push("mark".into());
            if self.cell(actor.cell).rock {
                actions.push("mark rock".into());
            }
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

    /// Look and the arms read the cup. A leak leaves it soaked. Sealing names the resin.
    fn bark_cup_arm_phrase(&self, actor_id: ActorId) -> String {
        let Some(id) = self.loose_ids(actor_id, Material::BarkCup).first().copied() else {
            return "a bark cup".into();
        };
        self.bark_cup_name(id, false)
    }

    fn bark_cup_name(&self, id: ItemId, capital: bool) -> String {
        let (name, _) = self.bark_cup_examine(id, capital);
        name
    }

    fn bark_cup_examine(&self, id: ItemId, capital: bool) -> (String, &'static str) {
        let cup = self.item(id);
        let (stem, hold) = if cup.is_some_and(|cup| cup.pitched) {
            ("sealed bark cup", "Pine resin seals it.")
        } else if cup.is_some_and(|cup| cup.moisture >= self.tuning.soaked_at) {
            ("soaked bark cup", "The bark is soaked.")
        } else if cup.is_some_and(|cup| cup.moisture >= self.tuning.damp_at) {
            ("damp bark cup", "The bark is damp.")
        } else {
            ("bark cup", "A drink soaks through it.")
        };
        let name = if capital {
            format!("A {stem}")
        } else {
            format!("a {stem}")
        };
        (name, hold)
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

fn orthogonal_neighbor(from: CellId, to: CellId) -> bool {
    let (x1, y1) = cell_xy(from);
    let (x2, y2) = cell_xy(to);
    let dx = (x1 - x2).abs();
    let dy = (y1 - y2).abs();
    (dx == 1 && dy == 0) || (dx == 0 && dy == 1)
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

/// Brook water in the pot is raw. Any other drink in that pot is not.
fn cup_drink_phrase(drinks: u32, raw_drinks: u32, sealed: bool) -> String {
    let cup = if sealed {
        "a sealed bark cup"
    } else {
        "a bark cup"
    };
    let boiled = drinks.saturating_sub(raw_drinks);
    match (boiled > 0, raw_drinks > 0, drinks == 1) {
        (true, false, true) => format!("{cup} with a drink of water in it"),
        (false, true, true) => format!("{cup} with a drink of raw water in it"),
        (true, false, false) => format!("{cup} with {drinks} drinks of water in it"),
        (false, true, false) => format!("{cup} with {drinks} drinks of raw water in it"),
        (true, true, _) => format!(
            "{cup} with {boiled} drinks of water and {raw_drinks} drinks of raw water in it"
        ),
        (false, false, _) => cup.into(),
    }
}

fn pot_drink_phrase(drinks: u32, raw_drinks: u32) -> String {
    let raw = raw_drinks.min(drinks);
    let plain = drinks.saturating_sub(raw);
    match (raw, plain) {
        (1, 0) => "a pot with a drink of raw water in it".into(),
        (raw, 0) => format!("a pot with {} drinks of raw water in it", count_word(raw)),
        (0, 1) => "a pot with a drink of water in it".into(),
        (0, plain) => format!("a pot with {} drinks of water in it", count_word(plain)),
        (raw, plain) => format!(
            "a pot with {} and {} in it",
            drink_count_phrase(raw, true),
            drink_count_phrase(plain, false)
        ),
    }
}

fn drink_count_phrase(count: u32, raw: bool) -> String {
    let kind = if raw { "raw water" } else { "water" };
    if count == 1 {
        format!("a drink of {kind}")
    } else {
        format!("{} drinks of {kind}", count_word(count))
    }
}

fn meat_kind_name(face: &str) -> &'static str {
    match face {
        "salted" => "salted meat",
        "smoked" => "smoked meat",
        "cooked" => "cooked meat",
        "spoiled" => "spoiled meat",
        _ => "raw meat",
    }
}

fn meat_lie(face: &str, count: u32, place: &str, small: bool) -> String {
    if face == "hanging" || face == "set" {
        let noun = if small {
            if count == 1 {
                "A smaller piece of raw meat".to_string()
            } else {
                format!(
                    "{} smaller pieces of raw meat",
                    capitalize(&count_word(count))
                )
            }
        } else if count == 1 {
            "A portion of raw meat".to_string()
        } else {
            format!("{} portions of raw meat", capitalize(&count_word(count)))
        };
        let (verb, spot) = if face == "hanging" {
            (if count == 1 { "hangs" } else { "hang" }, "by the fire")
        } else {
            (
                if count == 1 { "is set" } else { "are set" },
                "near the fire",
            )
        };
        let yet = if count == 1 {
            "It is not smoked yet."
        } else {
            "They are not smoked yet."
        };
        return if place == "here" {
            format!("{noun} {verb} {spot}. {yet}")
        } else {
            format!("{noun} {verb} {spot} {place}. {yet}")
        };
    }
    let kind = meat_kind_name(face);
    let line = if small {
        if count == 1 {
            format!("A smaller piece of {kind} lies {place}.")
        } else {
            format!(
                "{} smaller pieces of {kind} lie {place}.",
                capitalize(&count_word(count))
            )
        }
    } else if count == 1 {
        format!("A portion of {kind} lies {place}.")
    } else {
        format!(
            "{} portions of {kind} lie {place}.",
            capitalize(&count_word(count))
        )
    };
    let tail = match face {
        "salted" | "smoked" => " It can be eaten cold.",
        "going" => {
            if count == 1 {
                " It is going bad."
            } else {
                " They are going bad."
            }
        }
        "spoiled" => {
            if count == 1 {
                " It is not food."
            } else {
                " They are not food."
            }
        }
        _ => "",
    };
    format!("{line}{tail}")
}

fn meat_bit(face: &str, count: u32, small: bool) -> String {
    let kind = meat_kind_name(face);
    let bit = if small {
        smaller_bit(kind, count)
    } else {
        portion_bit(kind, count)
    };
    if face == "going" {
        let verb = if count == 1 { "is" } else { "are" };
        format!("{bit} that {verb} going bad")
    } else {
        bit
    }
}

fn meat_held(face: &str, count: u32, small: bool, who: &str, cover: &str) -> String {
    let kind = meat_kind_name(face);
    let (noun, verb) = if small {
        if count == 1 {
            (format!("A smaller piece of {kind}"), "is")
        } else {
            (
                format!(
                    "{} smaller pieces of {kind}",
                    capitalize(&count_word(count))
                ),
                "are",
            )
        }
    } else if count == 1 {
        (format!("A portion of {kind}"), "is")
    } else {
        (
            format!("{} portions of {kind}", capitalize(&count_word(count))),
            "are",
        )
    };
    let tail = match face {
        "salted" | "smoked" => " It can be eaten cold.",
        "going" => {
            if count == 1 {
                " It is going bad."
            } else {
                " They are going bad."
            }
        }
        "spoiled" => {
            if count == 1 {
                " It is not food."
            } else {
                " They are not food."
            }
        }
        _ => "",
    };
    format!("{noun} {verb} carried by {who}, {cover}.{tail}")
}

fn smaller_bit(kind: &str, count: u32) -> String {
    if count == 1 {
        format!("a smaller piece of {kind}")
    } else {
        format!("{} smaller pieces of {kind}", count_word(count))
    }
}

fn portion_bit(kind: &str, count: u32) -> String {
    if count == 1 {
        format!("a portion of {kind}")
    } else {
        format!("{} portions of {kind}", count_word(count))
    }
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
        (Material::PaleBerry, true) => "handful of pale berries",
        (Material::PaleBerry, false) => "handfuls of pale berries",
        (Material::Mushroom, true) => "mushroom",
        (Material::Mushroom, false) => "mushrooms",
        (Material::Pot, _) => "pot",
        (Material::Stone, _) => "stone",
        (Material::Flint, _) => "flint nodule",
        (Material::Flake, _) => "flint flake",
        (Material::Clay, true) => "lump of clay",
        (Material::Clay, false) => "lumps of clay",
        (Material::Ochre, true) => "lump of ochre",
        (Material::Ochre, false) => "lumps of ochre",
        (Material::IronOre, true) => "lump of iron ore",
        (Material::IronOre, false) => "lumps of iron ore",
        (Material::Iron, true) => "bloom of iron",
        (Material::Iron, false) => "blooms of iron",
        (Material::Knife, true) => "iron knife",
        (Material::Knife, false) => "iron knives",
        (Material::CopperOre, true) => "lump of copper ore",
        (Material::CopperOre, false) => "lumps of copper ore",
        (Material::Copper, true) => "copper bead",
        (Material::Copper, false) => "copper beads",
        (Material::Awl, true) => "copper awl",
        (Material::Awl, false) => "copper awls",
        (Material::Bone, true) => "bone",
        (Material::Bone, false) => "bones",
        (Material::Salt, _) => "salt crust",
        (Material::Honey, _) => "honeycomb",
        (Material::Egg, _) => "eggs",
        (Material::Moss, _) => "handful of moss",
        (Material::Mint, true) => "handful of mint",
        (Material::Mint, false) => "handfuls of mint",
        (Material::Drill, true) => "bow drill",
        (Material::Drill, false) => "bow drills",
        (Material::Needle, true) => "handful of pine needles",
        (Material::Needle, false) => "handfuls of pine needles",
        (Material::Resin, true) => "lump of pine resin",
        (Material::Resin, false) => "lumps of pine resin",
        (Material::Willow, true) => "bundle of willow cordage",
        (Material::Willow, false) => "bundles of willow cordage",
        (Material::WillowBark, true) => "strip of willow bark",
        (Material::WillowBark, false) => "strips of willow bark",
        (Material::Fluff, true) => "handful of cattail fluff",
        (Material::Fluff, false) => "handfuls of cattail fluff",
        (Material::Worm, true) => "worm",
        (Material::Worm, false) => "worms",
        (Material::Quartz, true) => "clear stone of quartz",
        (Material::Quartz, false) => "clear stones of quartz",
        (Material::Whistle, true) => "bone whistle",
        (Material::Whistle, false) => "bone whistles",
        (Material::Flute, true) => "reed flute",
        (Material::Flute, false) => "reed flutes",
        (Material::Drum, true) => "drum",
        (Material::Drum, false) => "drums",
        (Material::Nettle, true) => "bundle of nettle cordage",
        (Material::Nettle, false) => "bundles of nettle cordage",
        (Material::Cattail, _) => "cattail roots",
        (Material::Flour, true) => "cattail flour",
        (Material::Flour, false) => "cattail flour",
        (Material::BarkCup, true) => "bark cup",
        (Material::BarkCup, false) => "bark cups",
        (Material::Bow, true) => "bow",
        (Material::Bow, false) => "bows",
        (Material::Net, true) => "fishing net",
        (Material::Net, false) => "fishing nets",
        (Material::Mat, true) => "reed mat",
        (Material::Mat, false) => "reed mats",
        (Material::Canoe, _) => "dugout canoe",
        (Material::Paddle, true) => "wooden paddle",
        (Material::Paddle, false) => "wooden paddles",
        (Material::Hook, true) => "bone fish hook",
        (Material::Hook, false) => "bone fish hooks",
        (Material::Bowl, _) => "bowl",
        (Material::Spear, _) => "spear",
        (Material::Carcass, _) => "carcass",
        (Material::Meat, _) => "raw meat",
        (Material::Hide, _) => "deer hide",
        (Material::Cord, _) => "hide strips",
        (Material::Fish, _) => "fish",
        (Material::Brand, _) => "brand",
        (Material::Basket, _) => "bark basket",
        (Material::Charcoal, true) => "lump of charcoal",
        (Material::Charcoal, false) => "lumps of charcoal",
        (Material::Reed, true) => "bundle of reeds",
        (Material::Reed, false) => "bundles of reeds",
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
        Material::PaleBerry => ("pale berries", "are"),
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
        Material::BitterBerry => "",
        Material::Mud => " It can be smeared on skin against the sun.",
        Material::Mushroom => "",
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
