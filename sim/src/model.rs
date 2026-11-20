//! Valley state. Quantities stay piles until a person lifts them or builds
//! them into a frame; after that they are items with stable ids.

use crate::serde_map;
use crate::tuning::Tuning;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

pub const WIDTH: i32 = 7;
pub const HEIGHT: i32 = 5;
const EVENT_CAP: usize = 20_000;

/// One successful act, filled while looking at one recipient.
/// `rest` already closes the sentence, as in `a pot.`
pub(crate) fn spoken_act(name: Option<&str>, stem: &str, rest: &str) -> String {
    match name {
        None => format!("You {stem} {rest}"),
        Some(name) => format!("{name} {} {rest}", third_person(stem)),
    }
}

/// A smear on the skin. One pattern: `your` for the actor, `their` for a witness.
pub(crate) fn spoken_smear(name: Option<&str>, stuff: &str) -> String {
    let skin = if name.is_none() { "your" } else { "their" };
    spoken_act(name, "smear", &format!("{stuff} on {skin} skin."))
}

/// Third person of a present stem. `thatch` becomes `thatches`.
fn third_person(stem: &str) -> String {
    if stem.ends_with("ch")
        || stem.ends_with("sh")
        || stem.ends_with('s')
        || stem.ends_with('x')
        || stem.ends_with('z')
    {
        format!("{stem}es")
    } else {
        format!("{stem}s")
    }
}

pub type ActorId = u32;
pub type ItemId = u32;
pub type ConstructionId = u32;
pub type WorkId = u32;
pub type FireId = u32;
pub type DeerId = u32;
pub type EventId = u64;
pub type CellId = u16;

pub fn cell_id(x: i32, y: i32) -> Option<CellId> {
    if x < 0 || y < 0 || x >= WIDTH || y >= HEIGHT {
        None
    } else {
        Some((y * WIDTH + x) as u16)
    }
}

pub fn cell_xy(id: CellId) -> (i32, i32) {
    let n = id as i32;
    (n % WIDTH, n / WIDTH)
}

pub fn chebyshev(a: CellId, b: CellId) -> i32 {
    let (x1, y1) = cell_xy(a);
    let (x2, y2) = cell_xy(b);
    (x1 - x2).abs().max((y1 - y2).abs())
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[repr(u8)]
pub enum Material {
    Branch,
    Bark,
    Log,
    Berry,
    Water,
    Mushroom,
    BitterBerry,
    Pot,
    Mud,
    Stone,
    Clay,
    Bowl,
    Spear,
    Carcass,
    Flint,
    Flake,
    Meat,
    Hide,
    Ochre,
    CopperOre,
    IronOre,
    Iron,
    Knife,
    Copper,
    Awl,
    Cord,
    Fish,
    Brand,
    Basket,
    Charcoal,
    Reed,
    Bone,
    Salt,
    Honey,
    Egg,
    Moss,
    Bow,
}

impl Material {
    pub fn word(self) -> &'static str {
        match self {
            Material::Branch => "branch",
            Material::Bark => "bark",
            Material::Log => "log",
            Material::Berry => "berry",
            Material::Water => "water",
            Material::Mushroom => "mushroom",
            Material::BitterBerry => "poison berry",
            Material::Pot => "pot",
            Material::Mud => "mud",
            Material::Stone => "stone",
            Material::Clay => "clay",
            Material::Bowl => "bowl",
            Material::Spear => "spear",
            Material::Carcass => "carcass",
            Material::Flint => "flint",
            Material::Flake => "flake",
            Material::Meat => "meat",
            Material::Hide => "hide",
            Material::Ochre => "ochre",
            Material::CopperOre => "copper ore",
            Material::IronOre => "iron ore",
            Material::Iron => "iron",
            Material::Knife => "iron knife",
            Material::Copper => "copper",
            Material::Awl => "awl",
            Material::Cord => "strips",
            Material::Fish => "fish",
            Material::Brand => "brand",
            Material::Basket => "basket",
            Material::Charcoal => "charcoal",
            Material::Reed => "reed",
            Material::Bone => "bone",
            Material::Salt => "salt",
            Material::Honey => "honey",
            Material::Egg => "egg",
            Material::Moss => "moss",
            Material::Bow => "bow",
        }
    }

    pub fn gather_noun(self) -> &'static str {
        match self {
            Material::Branch => "branch",
            Material::Bark => "sheet of bark",
            Material::Log => "log",
            Material::Berry => "handful of berries",
            Material::Water => "drink of water",
            Material::Mushroom => "mushroom",
            Material::BitterBerry => "handful of poison berries",
            Material::Pot => "pot",
            Material::Mud => "handful of mud",
            Material::Stone => "stone",
            Material::Clay => "lump of clay",
            Material::Bowl => "bowl",
            Material::Spear => "spear",
            Material::Carcass => "carcass",
            Material::Flint => "flint nodule",
            Material::Flake => "flint flake",
            Material::Meat => "raw meat",
            Material::Hide => "deer hide",
            Material::Ochre => "lump of ochre",
            Material::CopperOre => "lump of copper ore",
            Material::IronOre => "lump of iron ore",
            Material::Iron => "bloom of iron",
            Material::Knife => "iron knife",
            Material::Copper => "copper bead",
            Material::Awl => "copper awl",
            Material::Cord => "hide strips",
            Material::Fish => "fish",
            Material::Brand => "burning brand",
            Material::Basket => "bark basket",
            Material::Charcoal => "lump of charcoal",
            Material::Reed => "bundle of reeds",
            Material::Bone => "bone",
            Material::Salt => "salt crust",
            Material::Honey => "honeycomb",
            Material::Egg => "eggs",
            Material::Moss => "handful of moss",
            Material::Bow => "bow",
        }
    }

    pub fn cooked_phrase(self) -> &'static str {
        match self {
            Material::Mushroom => "a mushroom",
            Material::BitterBerry => "a handful of poison berries",
            Material::Berry => "a handful of berries",
            Material::Carcass => "the carcass",
            _ => "some food",
        }
    }

    pub fn article(self) -> &'static str {
        match self {
            Material::Knife => "an",
            _ => "a",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Role {
    Loose,
    Support,
    Beam,
    BarkCover,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Region {
    Slope,
    Brook,
    Floor,
}

impl Region {
    pub fn word(self) -> &'static str {
        match self {
            Region::Slope => "slope",
            Region::Brook => "brook",
            Region::Floor => "valley floor",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum Dir {
    North,
    East,
    South,
    West,
}

impl Dir {
    pub fn delta(self) -> (i32, i32) {
        match self {
            Dir::North => (0, -1),
            Dir::East => (1, 0),
            Dir::South => (0, 1),
            Dir::West => (-1, 0),
        }
    }

    pub fn word(self) -> &'static str {
        match self {
            Dir::North => "north",
            Dir::East => "east",
            Dir::South => "south",
            Dir::West => "west",
        }
    }

    pub fn all() -> [Dir; 4] {
        [Dir::North, Dir::East, Dir::South, Dir::West]
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum ExamineTarget {
    Here,
    SelfBody,
    Wood,
    Bark,
    Log,
    Frame,
    Berries,
    BitterBerries,
    Mushrooms,
    Pot,
    Mud,
    Stone,
    Flint,
    Ochre,
    CopperOre,
    IronOre,
    Iron,
    Knife,
    Copper,
    Awl,
    Flake,
    Clay,
    Bowl,
    Spear,
    Carcass,
    Meat,
    Hide,
    Cord,
    Fish,
    Brand,
    Fire,
    Water,
    Deer,
    Tracks,
    Rock,
    Slope,
    Person(String),
    Basket,
    Charcoal,
    Reeds,
    Snare,
    Bone,
    Salt,
    Honey,
    Eggs,
    Moss,
    Bow,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Attempt {
    Look,
    Examine(ExamineTarget),
    Say(String),
    Move(Dir),
    Gather(Material),
    Drop(Material),
    Grab,
    Drag(Dir),
    Release,
    Interrupt,
    RaiseFrame,
    LayBark,
    Drink,
    Light,
    Cook,
    Boil,
    SetClay,
    BindSpear,
    Knap,
    Haft,
    Hunt,
    Butcher,
    Wear,
    Remove,
    Eat(Material),
    Smear,
    SmearOchre,
    Mark,
    MarkRock,
    Smelt,
    SmeltIron,
    Feed,
    Hammer,
    HammerIron,
    Pierce,
    Sew,
    Rest,
    Cut,
    Shape,
    FirePot,
    BuildLeanTo,
    CutHide,
    DryBranch,
    Fish,
    TakeBrand,
    WeaveBasket,
    PutFish,
    Thatch,
    SetSnare,
    CarveAwl,
    SaltMeat,
    SaltFish,
    SmokeNest,
    DressSting,
    CraftBow,
    DryClothes,
}

impl Attempt {
    pub fn uses_body(&self) -> bool {
        matches!(
            self,
            Attempt::Move(_)
                | Attempt::Gather(_)
                | Attempt::Drop(_)
                | Attempt::Grab
                | Attempt::Drag(_)
                | Attempt::RaiseFrame
                | Attempt::LayBark
                | Attempt::Drink
                | Attempt::Light
                | Attempt::Cook
                | Attempt::Boil
                | Attempt::SetClay
                | Attempt::BindSpear
                | Attempt::Knap
                | Attempt::Haft
                | Attempt::Hunt
                | Attempt::Butcher
                | Attempt::Wear
                | Attempt::Remove
                | Attempt::Eat(_)
                | Attempt::Smear
                | Attempt::SmearOchre
                | Attempt::Mark
                | Attempt::MarkRock
                | Attempt::Smelt
                | Attempt::SmeltIron
                | Attempt::Feed
                | Attempt::Hammer
                | Attempt::HammerIron
                | Attempt::Pierce
                | Attempt::Sew
                | Attempt::Rest
                | Attempt::Cut
                | Attempt::Shape
                | Attempt::FirePot
                | Attempt::BuildLeanTo
                | Attempt::CutHide
                | Attempt::DryBranch
                | Attempt::Fish
                | Attempt::TakeBrand
                | Attempt::WeaveBasket
                | Attempt::PutFish
                | Attempt::Thatch
                | Attempt::SetSnare
                | Attempt::CarveAwl
                | Attempt::SaltMeat
                | Attempt::SaltFish
                | Attempt::SmokeNest
                | Attempt::DressSting
                | Attempt::CraftBow
                | Attempt::DryClothes
        )
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Intent {
    pub command_id: String,
    pub actor_id: ActorId,
    pub attempt: Attempt,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct CommandResult {
    pub command_id: String,
    pub actor_id: ActorId,
    pub success: bool,
    pub message: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct StepReport {
    pub time: u64,
    pub results: Vec<CommandResult>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Cell {
    pub id: CellId,
    pub x: i32,
    pub y: i32,
    pub region: Region,
    pub rock: bool,
    pub trail: bool,
    /// A pool that does not run. Drinking it is not the same as the brook.
    #[serde(default)]
    pub still: bool,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Pile {
    pub cell: CellId,
    pub material: Material,
    pub count: u32,
    pub moisture: u32,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Actor {
    pub id: ActorId,
    pub name: String,
    pub token: String,
    pub cell: CellId,
    pub wetness: u32,
    #[serde(default)]
    pub thirst: u32,
    #[serde(default)]
    pub hunger: u32,
    #[serde(default)]
    pub illness: u32,
    /// Rises like hunger. Food brings it back down. The word stays on the observation.
    #[serde(default)]
    pub energy: u32,
    /// Open sky and wind. Shelter brings the gain down with exposure.
    #[serde(default)]
    pub cold: u32,
    /// Ticks of leftover fire heat. Charged while standing on a fire.
    /// Each tick off the fire spends one, and the sky's cold is halved.
    #[serde(default)]
    pub warmth: u32,
    /// The warm word is still the fire's. It lasts through the leftover
    /// ticks and the open-wind ticks after them, until the word leaves warm
    /// or shelter is what is holding it.
    #[serde(default)]
    pub warm_from_fire: bool,
    /// Sun on bare skin. Mud on the skin brings the gain down. Rain hides the sun.
    #[serde(default)]
    pub sunburn: u32,
    /// A smear of mud. It flakes off, and rain in the open washes it off.
    #[serde(default)]
    pub mud: u32,
    /// A smear of red ochre. It flakes off, and rain in the open washes it off.
    #[serde(default)]
    pub ochre: u32,
    /// The sim time when bee stings leave the body. None means the skin is clear.
    /// Mud and ochre do not change it.
    #[serde(default)]
    pub stings_until: Option<u64>,
    /// When wet clothes at a fire become dry. None means they are not drying.
    /// Stepping off the fire clears it. The clothes stay as wet as they were.
    #[serde(default)]
    pub clothes_dry_at: Option<u64>,
    pub busy: Option<WorkId>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Item {
    pub id: ItemId,
    pub material: Material,
    pub moisture: u32,
    pub cell: CellId,
    pub carrier: Option<ActorId>,
    pub role: Role,
    pub construction: Option<ConstructionId>,
    /// When set, this lump of clay is waiting on a fire until this tick,
    /// or a carried spear is waiting for its point to harden.
    #[serde(default)]
    pub firing_ready_at: Option<u64>,
    /// A spear's point. Other materials leave this false.
    #[serde(default)]
    pub hardened: bool,
    /// Brook water that has not been boiled. Boiled water is not raw.
    #[serde(default)]
    pub raw: bool,
    /// A spear hafted from a flint flake. It is not the stone-bound spear.
    #[serde(default)]
    pub flint_tip: bool,
    /// Hide strips lashing a flint tip to the haft. A stone spear leaves this false.
    #[serde(default)]
    pub bound: bool,
    /// Meat that has been held over a fire. Raw meat leaves this false.
    #[serde(default)]
    pub cooked: bool,
    /// Raw meat rubbed with salt. It can be eaten cold. Cooked meat leaves this false.
    #[serde(default)]
    pub salted: bool,
    /// Raw meat that has waited out its spoil time. It is not food.
    /// Salted meat and cooked meat leave this false.
    #[serde(default)]
    pub spoiled: bool,
    /// The tick when raw unsalted meat spoils, counted from the step it was cut.
    /// The step before that, look says it is going bad. Salt clears this.
    /// Cooked meat clears this and stays good.
    #[serde(default)]
    pub spoils_at: Option<u64>,
    /// A hide worn against the cold. Other materials leave this false.
    #[serde(default)]
    pub worn: bool,
    /// A hide the copper awl has pierced. Sewing consumes this. Other materials leave this false.
    #[serde(default)]
    pub pierced: bool,
    /// A pierced hide sewn into a cloak. Other materials leave this false.
    #[serde(default)]
    pub sewn: bool,
    /// A portion the iron knife has already split. A whole portion leaves this false.
    #[serde(default)]
    pub small: bool,
    /// A lump of clay shaped into a pot. It has not been fired.
    #[serde(default)]
    pub shaped: bool,
    /// A handful of berries sitting in a bark basket. It does not take an arm slot.
    #[serde(default)]
    pub in_basket: bool,
    /// An awl carved from a bone. A copper awl leaves this false.
    #[serde(default)]
    pub bone: bool,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Construction {
    pub id: ConstructionId,
    pub cell: CellId,
    pub support_a: Option<ItemId>,
    pub support_b: Option<ItemId>,
    pub beam: Option<ItemId>,
    pub bark: Option<ItemId>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum WorkKind {
    Drag,
    RaiseFrame,
    LayBark,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Work {
    pub id: WorkId,
    pub kind: WorkKind,
    pub actors: Vec<ActorId>,
    pub completes_at: u64,
    pub cause_events: Vec<EventId>,
    pub item: Option<ItemId>,
    pub dir: Option<Dir>,
    pub phase_done: u8,
    pub phases: u8,
    pub construction: Option<ConstructionId>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Grip {
    pub actor: ActorId,
    pub item: ItemId,
    pub since_event: EventId,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Fire {
    pub id: FireId,
    pub cell: CellId,
    pub lit_event: EventId,
    pub goes_out_at: u64,
    /// Set when someone feeds the fire an extra dry branch. A small fire
    /// smelts copper. Iron waits for this.
    #[serde(default)]
    pub hot: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Wake {
    Rain,
    Thirst,
    Hunger,
    Work(WorkId),
    Fire(FireId),
    Illness,
    Deer,
    Season,
    Rot,
    Energy,
    Weather,
    Harden(ItemId),
    HardenSpear(ItemId),
    Smelt(ItemId),
    DryBranch(ItemId),
    BrandOut(ItemId),
    /// A hide snare set on this cell. It springs on that tick.
    Snare(CellId),
    /// Raw unsalted meat spoils on this tick. Salted meat and cooked meat do not.
    Spoil(ItemId),
    /// The salt lick on this cell forms another crust.
    SaltReturn(CellId),
    /// The bees' nest on this cell has honey again.
    HoneyReturn(CellId),
    /// The bird's nest on this cell holds eggs again.
    EggReturn(CellId),
    /// The damp bank on this cell grows moss again.
    MossReturn(CellId),
    /// A flint nodule lies on this slope cell again.
    FlintReturn(CellId),
    /// A deer stands on the trail again after the last one was brought down.
    DeerReturn,
    /// Bee stings on a body. Each tick, any sting whose time has come fades.
    Stings,
    /// Wet clothes on this person dry at the fire they are standing by.
    DryClothes(ActorId),
}

/// One deer. It is not a sector and not a pile. `route_at` is the cell it
/// is standing on in [`DEER_ROUTE`].
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Deer {
    pub id: DeerId,
    pub cell: CellId,
    pub route_at: usize,
    pub last_event: Option<EventId>,
    /// A miss or a bolt already moved the deer this tick, so the route step waits.
    #[serde(default)]
    pub held: bool,
}

/// Trail the deer already uses, then a detour to the berries under the rock
/// and back. Orthogonal, so each step is one direction.
pub(crate) const DEER_ROUTE: [(i32, i32); 12] = [
    (2, 3),
    (3, 3),
    (4, 3),
    (4, 2),
    (5, 2),
    (5, 1),
    (6, 1),
    (5, 1),
    (5, 2),
    (4, 2),
    (4, 3),
    (3, 3),
];

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Scheduled {
    pub at: u64,
    pub seq: u32,
    pub wake: Wake,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Scope {
    Personal(ActorId),
    Cell(CellId),
    Valley,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Draw {
    pub tag: String,
    pub subject: String,
    pub value: u64,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Event {
    pub id: EventId,
    pub time: u64,
    pub text: String,
    pub scope: Scope,
    pub affected_actors: Vec<ActorId>,
    pub affected_cells: Vec<CellId>,
    pub affected_items: Vec<ItemId>,
    pub causes: Vec<EventId>,
    pub draw: Option<Draw>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct World {
    pub tuning: Tuning,
    pub seed: u64,
    pub time: u64,
    pub raining: bool,
    /// Open rain soaks wood twice as fast once this is set. Shelter does not change.
    #[serde(default)]
    pub wet_season: bool,
    pub first_rain: Option<EventId>,
    pub next_actor: ActorId,
    pub next_item: ItemId,
    pub next_construction: ConstructionId,
    pub next_work: WorkId,
    #[serde(default = "default_next_id")]
    pub next_fire: FireId,
    pub next_event: EventId,
    pub next_seq: u32,
    pub cells: Vec<Cell>,
    #[serde(with = "serde_map")]
    pub(crate) actors: BTreeMap<ActorId, Actor>,
    #[serde(with = "serde_map")]
    pub(crate) items: BTreeMap<ItemId, Item>,
    #[serde(with = "serde_map")]
    pub(crate) constructions: BTreeMap<ConstructionId, Construction>,
    #[serde(with = "serde_map")]
    pub(crate) works: BTreeMap<WorkId, Work>,
    pub(crate) piles: Vec<Pile>,
    #[serde(default)]
    pub(crate) fires: BTreeMap<FireId, Fire>,
    #[serde(default)]
    pub(crate) deer: Option<Deer>,
    /// Cells the deer has left. The marks stay; decay is not in this valley.
    #[serde(default)]
    pub(crate) tracks: BTreeSet<CellId>,
    /// Cells where a berry pile reached zero. The pile is not put back.
    #[serde(default)]
    pub(crate) spent_berries: BTreeSet<CellId>,
    /// Cells where the deer ate the last handful. A person taking the last
    /// handful leaves this unset.
    #[serde(default)]
    pub(crate) deer_ate_berries: BTreeSet<CellId>,
    /// A red ochre mark left on a cell. Open rain washes it. The rock keeps it.
    #[serde(default)]
    pub(crate) ochre_marks: BTreeSet<CellId>,
    /// A black charcoal mark left on a cell. Open rain washes it. The rock keeps it.
    #[serde(default)]
    pub(crate) charcoal_marks: BTreeSet<CellId>,
    /// A lean-to of branches. It stands in that cell and breaks the wind.
    #[serde(default)]
    pub(crate) lean_tos: BTreeSet<CellId>,
    /// A lean-to thatched with reeds. Rain and cold match the rock. A bare lean-to does not.
    #[serde(default)]
    pub(crate) thatched: BTreeSet<CellId>,
    /// A hide snare set on a cell, springing at this sim time. The strips are spent.
    #[serde(default)]
    pub(crate) snares: BTreeMap<CellId, u64>,
    /// A brook cell stays quiet until this sim time after a fish is taken there.
    /// Banks that touch that cell fish the same stretch.
    #[serde(default)]
    pub(crate) brook_quiet: BTreeMap<CellId, u64>,
    /// A cell a deer has just left, or where a bolt or a kill left tracks,
    /// until this sim time. The permanent hoofprints are separate.
    #[serde(default)]
    pub(crate) fresh_tracks: BTreeMap<CellId, u64>,
    /// The northwest corner, after a night howl, until this sim time.
    /// There is no wolf on that ground. Look names the tracks for someone
    /// who can see the cell, then they fade like deer tracks.
    #[serde(default)]
    pub(crate) wolf_tracks: BTreeMap<CellId, u64>,
    /// Slope cells where a salt crust forms again after it is taken.
    #[serde(default)]
    pub(crate) salt_licks: BTreeSet<CellId>,
    /// When the bare lick forms another crust. Look says the slope is bare until then.
    #[serde(default)]
    pub(crate) salt_returns_at: BTreeMap<CellId, u64>,
    /// A tree that holds a bees' nest. Empty once the honeycomb is taken.
    #[serde(default)]
    pub(crate) honey_nests: BTreeSet<CellId>,
    /// When an emptied nest fills again. Look says the nest is empty until then.
    #[serde(default)]
    pub(crate) honey_returns_at: BTreeMap<CellId, u64>,
    /// A nest the bees have been smoked out of. The next gather there does not sting.
    #[serde(default)]
    pub(crate) smoked_nests: BTreeSet<CellId>,
    /// A tree on the west slope that holds a bird's nest. Empty once the eggs are taken.
    #[serde(default)]
    pub(crate) bird_nests: BTreeSet<CellId>,
    /// When an emptied bird's nest holds eggs again. Look says the nest is empty until then.
    #[serde(default)]
    pub(crate) egg_returns_at: BTreeMap<CellId, u64>,
    /// Damp banks where moss grows. Bare once the handful is taken.
    #[serde(default)]
    pub(crate) moss_banks: BTreeSet<CellId>,
    /// When a bare bank grows moss again. Look says the bank is bare until then.
    #[serde(default)]
    pub(crate) moss_returns_at: BTreeMap<CellId, u64>,
    /// Slope cells where a flint nodule lies again after it is taken.
    #[serde(default)]
    pub(crate) flint_beds: BTreeSet<CellId>,
    /// When a bare slope has a flint nodule again. Look says the slope has no flint until then.
    #[serde(default)]
    pub(crate) flint_returns_at: BTreeMap<CellId, u64>,
    /// When a deer stands on the trail again. Look names no standing deer until then.
    #[serde(default)]
    pub(crate) deer_returns_at: Option<u64>,
    /// The next deer's id. The first deer is 1.
    #[serde(default = "default_next_deer_id")]
    pub(crate) next_deer_id: u32,
    pub(crate) grips: Vec<Grip>,
    pub(crate) schedule: Vec<Scheduled>,
    pub(crate) events: Vec<Event>,
    pub(crate) applied: BTreeMap<String, CommandResult>,
}

const SPAWNS: [(i32, i32); 7] = [(4, 3), (3, 3), (5, 3), (4, 4), (2, 3), (6, 3), (3, 4)];

fn default_next_deer_id() -> u32 {
    2
}

impl World {
    pub fn valley(tuning: Tuning, seed: u64) -> Self {
        let mut cells = Vec::with_capacity((WIDTH * HEIGHT) as usize);
        for y in 0..HEIGHT {
            for x in 0..WIDTH {
                let rock = x == 6 && y == 1;
                let region = if y == 0 {
                    Region::Slope
                } else if x <= 1 && (y == 1 || y == 2) {
                    Region::Brook
                } else {
                    Region::Floor
                };
                let trail = matches!((x, y), (2, 3) | (3, 3) | (4, 3) | (4, 2) | (5, 2));
                cells.push(Cell {
                    id: cell_id(x, y).unwrap(),
                    x,
                    y,
                    region,
                    rock,
                    trail,
                    still: x == 2 && y == 1,
                });
            }
        }
        let mut world = World {
            tuning,
            seed,
            time: 0,
            raining: false,
            wet_season: false,
            first_rain: None,
            next_actor: 1,
            next_item: 1,
            next_construction: 1,
            next_work: 1,
            next_fire: 1,
            next_event: 1,
            next_seq: 1,
            cells,
            actors: BTreeMap::new(),
            items: BTreeMap::new(),
            constructions: BTreeMap::new(),
            works: BTreeMap::new(),
            piles: Vec::new(),
            fires: BTreeMap::new(),
            deer: Some(Deer {
                id: 1,
                cell: cell_id(2, 3).unwrap(),
                route_at: 0,
                last_event: None,
                held: false,
            }),
            tracks: BTreeSet::new(),
            spent_berries: BTreeSet::new(),
            deer_ate_berries: BTreeSet::new(),
            ochre_marks: BTreeSet::new(),
            charcoal_marks: BTreeSet::new(),
            lean_tos: BTreeSet::new(),
            thatched: BTreeSet::new(),
            snares: BTreeMap::new(),
            brook_quiet: BTreeMap::new(),
            fresh_tracks: BTreeMap::new(),
            wolf_tracks: BTreeMap::new(),
            salt_licks: BTreeSet::new(),
            salt_returns_at: BTreeMap::new(),
            honey_nests: BTreeSet::new(),
            honey_returns_at: BTreeMap::new(),
            smoked_nests: BTreeSet::new(),
            bird_nests: BTreeSet::new(),
            egg_returns_at: BTreeMap::new(),
            moss_banks: BTreeSet::new(),
            moss_returns_at: BTreeMap::new(),
            flint_beds: BTreeSet::new(),
            flint_returns_at: BTreeMap::new(),
            deer_returns_at: None,
            next_deer_id: default_next_deer_id(),
            grips: Vec::new(),
            schedule: Vec::new(),
            events: Vec::new(),
            applied: BTreeMap::new(),
        };
        let rain_at = world.tuning.rain_starts_at;
        world.schedule(rain_at, Wake::Rain);
        world.schedule(world.tuning.season_at, Wake::Season);
        world.schedule(world.tuning.rot_ticks, Wake::Rot);
        world.schedule(0, Wake::Thirst);
        world.schedule(0, Wake::Hunger);
        world.schedule(0, Wake::Illness);
        world.schedule(0, Wake::Stings);
        world.schedule(0, Wake::Energy);
        world.schedule(0, Wake::Weather);
        world.schedule(0, Wake::Deer);
        world.add_pile(cell_id(6, 1).unwrap(), Material::Branch, 4, 0);
        world.add_pile(cell_id(6, 1).unwrap(), Material::Bark, 2, 0);
        world.add_pile(cell_id(6, 1).unwrap(), Material::Berry, 4, 0);
        world.add_pile(cell_id(6, 1).unwrap(), Material::Pot, 1, 0);
        world.add_pile(cell_id(4, 1).unwrap(), Material::BitterBerry, 2, 0);
        world.add_pile(cell_id(3, 1).unwrap(), Material::Mushroom, 2, 0);
        world.add_pile(cell_id(5, 2).unwrap(), Material::Branch, 4, 0);
        world.add_pile(cell_id(1, 0).unwrap(), Material::Bark, 2, 0);
        world.add_pile(cell_id(2, 0).unwrap(), Material::Stone, 1, 0);
        let flint = cell_id(4, 0).unwrap();
        world.add_pile(flint, Material::Flint, 1, 0);
        world.flint_beds.insert(flint);
        world.add_pile(cell_id(3, 0).unwrap(), Material::Ochre, 1, 0);
        world.add_pile(cell_id(5, 0).unwrap(), Material::IronOre, 1, 0);
        world.add_pile(cell_id(1, 2).unwrap(), Material::CopperOre, 1, 0);
        world.add_pile(cell_id(2, 2).unwrap(), Material::Clay, 2, 0);
        world.add_pile(cell_id(0, 3).unwrap(), Material::Reed, 4, 0);
        world.add_pile(cell_id(3, 2).unwrap(), Material::Log, 1, 0);
        world.add_pile(cell_id(0, 0).unwrap(), Material::Salt, 1, 0);
        world.salt_licks.insert(cell_id(0, 0).unwrap());
        let tree = cell_id(6, 0).unwrap();
        world.add_pile(tree, Material::Honey, 1, 0);
        world.honey_nests.insert(tree);
        let bird = cell_id(1, 0).unwrap();
        world.add_pile(bird, Material::Egg, 1, 0);
        world.bird_nests.insert(bird);
        // The south bank of the brook. Damp stones, and moss on them.
        let bank = cell_id(1, 3).unwrap();
        world.add_pile(bank, Material::Moss, 1, 0);
        world.moss_banks.insert(bank);
        world.normalize();
        world
    }

    pub fn join(&mut self, name: &str, token: &str) -> Result<ActorId, String> {
        let name = clean_name(name)?;
        if token.is_empty() || token.len() > 80 {
            return Err("missing token".into());
        }
        if self.actors.values().any(|a| a.token == token) {
            return Err("that token is already in the valley".into());
        }
        if self
            .actors
            .values()
            .any(|a| a.name.eq_ignore_ascii_case(&name))
        {
            return Err(format!("{name} is already a name in the valley"));
        }
        let n = self.actors.len();
        let (x, y) = if n < SPAWNS.len() {
            SPAWNS[n]
        } else {
            let i = n as i32;
            (i % WIDTH, (i / WIDTH) % HEIGHT)
        };
        self.spawn_at(&name, token, x, y)
    }

    pub fn spawn_at(&mut self, name: &str, token: &str, x: i32, y: i32) -> Result<ActorId, String> {
        let name = clean_name(name)?;
        let cell = cell_id(x, y).ok_or("that place is outside the valley")?;
        if self.actors.values().any(|a| a.token == token) {
            return Err("that token is already in the valley".into());
        }
        if self
            .actors
            .values()
            .any(|a| a.name.eq_ignore_ascii_case(&name))
        {
            return Err(format!("{name} is already a name in the valley"));
        }
        let id = self.next_actor;
        self.next_actor += 1;
        self.actors.insert(
            id,
            Actor {
                id,
                name,
                token: token.to_string(),
                cell,
                wetness: 0,
                thirst: 0,
                hunger: 0,
                illness: 0,
                energy: 0,
                cold: 0,
                warmth: 0,
                warm_from_fire: false,
                sunburn: 0,
                mud: 0,
                ochre: 0,
                stings_until: None,
                clothes_dry_at: None,
                busy: None,
            },
        );
        Ok(id)
    }

    pub fn actor_by_token(&self, token: &str) -> Option<ActorId> {
        self.actors
            .values()
            .find(|a| a.token == token)
            .map(|a| a.id)
    }

    pub fn actor_cell(&self, id: ActorId) -> Option<(i32, i32)> {
        self.actor(id).map(|actor| cell_xy(actor.cell))
    }

    pub fn stored_result(&self, command_id: &str) -> Option<CommandResult> {
        self.applied.get(command_id).cloned()
    }

    pub fn sim_time(&self) -> u64 {
        self.time
    }

    pub fn is_raining(&self) -> bool {
        self.raining
    }

    pub fn event_log(&self) -> &[Event] {
        &self.events
    }

    pub fn material_tally(&self, material: Material) -> u32 {
        let piles: u32 = self
            .piles
            .iter()
            .filter(|p| p.material == material)
            .map(|p| p.count)
            .sum();
        let items = self
            .items
            .values()
            .filter(|i| i.material == material)
            .count() as u32;
        piles + items
    }

    /// One row per item of this material: id, carrier, x, y. The id stays
    /// with the item when it is carried and when it is dropped.
    pub fn material_items(&self, material: Material) -> Vec<(ItemId, Option<ActorId>, i32, i32)> {
        let mut rows: Vec<_> = self
            .items
            .values()
            .filter(|item| item.material == material)
            .map(|item| {
                let (x, y) = cell_xy(item.cell);
                (item.id, item.carrier, x, y)
            })
            .collect();
        rows.sort();
        rows
    }

    /// Each spear's id, who carries it, and whether the point has hardened.
    /// Binding spends the stone and the branch. The id does not change when
    /// the point hardens.
    pub fn spears(&self) -> Vec<(ItemId, Option<ActorId>, bool)> {
        let mut rows: Vec<_> = self
            .items
            .values()
            .filter(|item| item.material == Material::Spear)
            .map(|item| (item.id, item.carrier, item.hardened))
            .collect();
        rows.sort();
        rows
    }

    pub fn log_cell(&self) -> Option<(i32, i32)> {
        if let Some(item) = self.items.values().find(|i| i.material == Material::Log) {
            return Some(cell_xy(item.cell));
        }
        self.piles
            .iter()
            .find(|p| p.material == Material::Log && p.count > 0)
            .map(|p| cell_xy(p.cell))
    }

    pub fn log_gripped_by(&self) -> Vec<String> {
        let Some(item) = self.items.values().find(|i| i.material == Material::Log) else {
            return Vec::new();
        };
        let mut names: Vec<String> = self
            .grips
            .iter()
            .filter(|g| g.item == item.id)
            .filter_map(|g| self.actors.get(&g.actor).map(|a| a.name.clone()))
            .collect();
        names.sort();
        names
    }

    pub fn deer_cell(&self) -> Option<(i32, i32)> {
        self.deer.as_ref().map(|deer| cell_xy(deer.cell))
    }

    pub fn hoofprints_at(&self, x: i32, y: i32) -> bool {
        cell_id(x, y).is_some_and(|id| self.tracks.contains(&id))
    }

    pub fn exposure_at(&self, x: i32, y: i32) -> Option<u32> {
        cell_id(x, y).map(|id| self.exposure_millis(id))
    }

    pub fn step(&mut self, intents: &[Intent]) -> StepReport {
        let time = self.time;
        let results = self.resolve(intents);
        self.arm_spears();
        self.tend_drying();
        self.tend_clothes();
        self.fire_due();
        self.normalize();
        self.trim_events();
        self.time += 1;
        self.howl_on_nightfall();
        StepReport { time, results }
    }

    pub(crate) fn schedule(&mut self, at: u64, wake: Wake) {
        if self.schedule.iter().any(|s| s.at == at && s.wake == wake) {
            return;
        }
        let seq = self.next_seq;
        self.next_seq += 1;
        self.schedule.push(Scheduled { at, seq, wake });
    }

    pub(crate) fn actor(&self, id: ActorId) -> Option<&Actor> {
        self.actors.get(&id)
    }

    pub(crate) fn cell(&self, id: CellId) -> &Cell {
        &self.cells[id as usize]
    }

    pub(crate) fn item(&self, id: ItemId) -> Option<&Item> {
        self.items.get(&id)
    }

    pub(crate) fn pile_count(&self, cell: CellId, material: Material) -> u32 {
        self.piles
            .iter()
            .find(|p| p.cell == cell && p.material == material)
            .map(|p| p.count)
            .unwrap_or(0)
    }

    pub(crate) fn pile_moisture(&self, cell: CellId, material: Material) -> u32 {
        self.piles
            .iter()
            .find(|p| p.cell == cell && p.material == material)
            .map(|p| p.moisture)
            .unwrap_or(0)
    }

    pub(crate) fn add_pile(&mut self, cell: CellId, material: Material, count: u32, moisture: u32) {
        if count == 0 {
            return;
        }
        if let Some(pile) = self
            .piles
            .iter_mut()
            .find(|p| p.cell == cell && p.material == material)
        {
            let total = pile.count + count;
            pile.moisture = (pile.moisture * pile.count + moisture * count) / total.max(1);
            pile.count = total;
        } else {
            self.piles.push(Pile {
                cell,
                material,
                count,
                moisture,
            });
        }
    }

    pub(crate) fn take_from_pile(
        &mut self,
        cell: CellId,
        material: Material,
        n: u32,
    ) -> Option<u32> {
        let idx = self
            .piles
            .iter()
            .position(|p| p.cell == cell && p.material == material)?;
        if self.piles[idx].count < n {
            return None;
        }
        let moisture = self.piles[idx].moisture;
        self.piles[idx].count -= n;
        if self.piles[idx].count == 0 {
            let emptied = self.piles[idx].cell;
            self.piles.remove(idx);
            if material == Material::Berry {
                self.spent_berries.insert(emptied);
                self.deer_ate_berries.remove(&emptied);
            }
        }
        Some(moisture)
    }

    pub(crate) fn mint_item(
        &mut self,
        material: Material,
        moisture: u32,
        cell: CellId,
        carrier: Option<ActorId>,
        role: Role,
    ) -> ItemId {
        let id = self.next_item;
        self.next_item += 1;
        self.items.insert(
            id,
            Item {
                id,
                material,
                moisture,
                cell,
                carrier,
                role,
                construction: None,
                firing_ready_at: None,
                hardened: false,
                raw: false,
                flint_tip: false,
                bound: false,
                cooked: false,
                salted: false,
                spoiled: false,
                spoils_at: None,
                worn: false,
                pierced: false,
                sewn: false,
                small: false,
                shaped: false,
                in_basket: false,
                bone: false,
            },
        );
        if material == Material::Meat || material == Material::Fish {
            self.arm_raw_spoil(id);
        }
        id
    }

    /// Raw meat spoils `meat_ticks` after the step that cut it.
    /// A raw fish spoils the same number of ticks after the step it was speared.
    /// Salt and cooking clear the wait. A wake that finds either does nothing.
    pub(crate) fn arm_raw_spoil(&mut self, id: ItemId) {
        let at = self.time.saturating_add(self.tuning.meat_ticks);
        let arm = self.item(id).is_some_and(|food| {
            (food.material == Material::Meat || food.material == Material::Fish)
                && !food.cooked
                && !food.salted
                && !food.spoiled
        });
        if !arm {
            return;
        }
        if let Some(meat) = self.items.get_mut(&id) {
            meat.spoils_at = Some(at);
        }
        self.schedule(at, Wake::Spoil(id));
    }

    /// The lick is empty. Another crust forms `salt_ticks` after this step.
    /// A wake that finds salt already there does not add a second crust.
    pub(crate) fn arm_salt_lick(&mut self, cell: CellId) {
        if !self.salt_licks.contains(&cell) || self.salt_returns_at.contains_key(&cell) {
            return;
        }
        if self.pile_count(cell, Material::Salt) > 0
            || !self.ground_ids(cell, Material::Salt).is_empty()
        {
            return;
        }
        let at = self.time.saturating_add(self.tuning.salt_ticks);
        self.salt_returns_at.insert(cell, at);
        self.schedule(at, Wake::SaltReturn(cell));
    }

    /// The nest is empty. Honey returns `honey_ticks` after this step.
    /// A wake that finds honey already there does not add a second comb.
    pub(crate) fn arm_honey_nest(&mut self, cell: CellId) {
        if !self.honey_nests.contains(&cell) || self.honey_returns_at.contains_key(&cell) {
            return;
        }
        if self.pile_count(cell, Material::Honey) > 0
            || !self.ground_ids(cell, Material::Honey).is_empty()
        {
            return;
        }
        let at = self.time.saturating_add(self.tuning.honey_ticks);
        self.honey_returns_at.insert(cell, at);
        self.schedule(at, Wake::HoneyReturn(cell));
    }

    /// The nest is empty. Eggs return `egg_ticks` after this step.
    /// A wake that finds eggs already there does not add a second clutch.
    pub(crate) fn arm_bird_nest(&mut self, cell: CellId) {
        if !self.bird_nests.contains(&cell) || self.egg_returns_at.contains_key(&cell) {
            return;
        }
        if self.pile_count(cell, Material::Egg) > 0
            || !self.ground_ids(cell, Material::Egg).is_empty()
        {
            return;
        }
        let at = self.time.saturating_add(self.tuning.egg_ticks);
        self.egg_returns_at.insert(cell, at);
        self.schedule(at, Wake::EggReturn(cell));
    }

    /// The bank is bare. Moss returns `moss_ticks` after this step.
    /// A wake that finds moss already there does not add a second handful.
    pub(crate) fn arm_moss_bank(&mut self, cell: CellId) {
        if !self.moss_banks.contains(&cell) || self.moss_returns_at.contains_key(&cell) {
            return;
        }
        if self.pile_count(cell, Material::Moss) > 0
            || !self.ground_ids(cell, Material::Moss).is_empty()
        {
            return;
        }
        let at = self.time.saturating_add(self.tuning.moss_ticks);
        self.moss_returns_at.insert(cell, at);
        self.schedule(at, Wake::MossReturn(cell));
    }

    /// The slope has no flint nodule. Another lies there `flint_ticks` after
    /// this step. Gathering it, or knapping the one that was carried, leaves
    /// the same wait. A wake that finds a nodule already there does not add
    /// a second one.
    pub(crate) fn arm_flint_bed(&mut self, cell: CellId) {
        if !self.flint_beds.contains(&cell) || self.flint_returns_at.contains_key(&cell) {
            return;
        }
        if self.pile_count(cell, Material::Flint) > 0
            || !self.ground_ids(cell, Material::Flint).is_empty()
        {
            return;
        }
        let at = self.time.saturating_add(self.tuning.flint_ticks);
        self.flint_returns_at.insert(cell, at);
        self.schedule(at, Wake::FlintReturn(cell));
    }

    /// The deer is down. Another stands on the trail `deer_ticks` after this
    /// step. A carcass that is butchered, or still lying where it fell, does
    /// not move that tick. A wake that finds a deer already standing does not
    /// add a second one.
    pub(crate) fn arm_deer_return(&mut self) {
        if self.deer.is_some() || self.deer_returns_at.is_some() {
            return;
        }
        let at = self.time.saturating_add(self.tuning.deer_ticks);
        self.deer_returns_at = Some(at);
        self.schedule(at, Wake::DeerReturn);
    }

    pub(crate) fn carry_count(&self, actor: ActorId) -> u32 {
        self.items
            .values()
            .filter(|i| i.carrier == Some(actor) && !i.in_basket)
            .count() as u32
    }

    pub(crate) fn ground_ids(&self, cell: CellId, material: Material) -> Vec<ItemId> {
        let mut ids: Vec<ItemId> = self
            .items
            .values()
            .filter(|item| {
                item.cell == cell
                    && item.material == material
                    && item.role == Role::Loose
                    && item.carrier.is_none()
            })
            .map(|item| item.id)
            .collect();
        ids.sort();
        ids
    }

    pub(crate) fn loose_ids(&self, actor: ActorId, material: Material) -> Vec<ItemId> {
        let mut ids: Vec<ItemId> = self
            .items
            .values()
            .filter(|i| i.carrier == Some(actor) && i.material == material && i.role == Role::Loose)
            .map(|i| i.id)
            .collect();
        ids.sort();
        ids
    }

    pub(crate) fn is_dry(&self, moisture: u32) -> bool {
        moisture < self.tuning.damp_at
    }

    pub(crate) fn fire_at(&self, cell: CellId) -> Option<&Fire> {
        self.fires.values().find(|fire| fire.cell == cell)
    }

    pub(crate) fn dry_loose_ids(&self, actor: ActorId, material: Material) -> Vec<ItemId> {
        self.loose_ids(actor, material)
            .into_iter()
            .filter(|id| {
                self.item(*id)
                    .is_some_and(|item| self.is_dry(item.moisture))
            })
            .collect()
    }

    pub(crate) fn dry_pile_count(&self, cell: CellId, material: Material) -> u32 {
        let count = self.pile_count(cell, material);
        if count > 0 && self.is_dry(self.pile_moisture(cell, material)) {
            count
        } else {
            0
        }
    }

    pub(crate) fn has_dry(&self, actor: ActorId, material: Material) -> bool {
        let Some(person) = self.actor(actor) else {
            return false;
        };
        !self.dry_loose_ids(actor, material).is_empty()
            || self.dry_pile_count(person.cell, material) > 0
    }

    pub(crate) fn has_material_at_hand(&self, actor: ActorId, material: Material) -> bool {
        let Some(person) = self.actor(actor) else {
            return false;
        };
        !self.loose_ids(actor, material).is_empty() || self.pile_count(person.cell, material) > 0
    }

    /// Dry fuel from the arms when it is already carried. The pile at the
    /// feet is spent only when the arms do not hold a dry piece. The bool
    /// is true when the piece came from the arms.
    pub(crate) fn consume_dry(
        &mut self,
        actor: ActorId,
        material: Material,
    ) -> Result<(Option<ItemId>, bool), ()> {
        if let Some(item_id) = self.dry_loose_ids(actor, material).first().copied() {
            self.items.remove(&item_id);
            return Ok((Some(item_id), true));
        }
        let Some(person_cell) = self.actor(actor).map(|person| person.cell) else {
            return Err(());
        };
        if self.dry_pile_count(person_cell, material) > 0
            && self.take_from_pile(person_cell, material, 1).is_some()
        {
            return Ok((None, false));
        }
        Err(())
    }

    /// One loose handful, carried before the pile on this cell.
    pub(crate) fn take_hand(
        &mut self,
        actor: ActorId,
        material: Material,
    ) -> Result<Option<ItemId>, ()> {
        if let Some(item_id) = self.loose_ids(actor, material).first().copied() {
            self.items.remove(&item_id);
            return Ok(Some(item_id));
        }
        let Some(person_cell) = self.actor(actor).map(|person| person.cell) else {
            return Err(());
        };
        if self.take_from_pile(person_cell, material, 1).is_some() {
            Ok(None)
        } else {
            Err(())
        }
    }

    /// Sweet berries and a carcass are eaten by the cook. A mushroom is not:
    /// heat leaves it in the arms or on the ground until someone eats it.
    /// Poison berries are not food.
    pub(crate) fn consume_food(
        &mut self,
        actor: ActorId,
    ) -> Result<(Option<ItemId>, Material), ()> {
        if let Some(id) = self.loose_ids(actor, Material::Carcass).first().copied() {
            self.items.remove(&id);
            return Ok((Some(id), Material::Carcass));
        }
        if let Ok(item) = self.take_hand(actor, Material::Berry) {
            return Ok((item, Material::Berry));
        }
        let Some(cell) = self.actor(actor).map(|person| person.cell) else {
            return Err(());
        };
        if let Some(id) = self.ground_ids(cell, Material::Carcass).first().copied() {
            self.items.remove(&id);
            return Ok((Some(id), Material::Carcass));
        }
        Err(())
    }

    pub(crate) fn mint_fire(&mut self, cell: CellId, lit_event: EventId) -> FireId {
        let id = self.next_fire;
        self.next_fire += 1;
        let goes_out_at = self.time.saturating_add(self.tuning.fire_ticks);
        self.fires.insert(
            id,
            Fire {
                id,
                cell,
                lit_event,
                goes_out_at,
                hot: false,
            },
        );
        self.schedule(goes_out_at, Wake::Fire(id));
        id
    }

    pub(crate) fn extinguish(&mut self, id: FireId, text: &str, extra_cause: Option<EventId>) {
        let Some(fire) = self.fires.remove(&id) else {
            return;
        };
        self.schedule.retain(|entry| entry.wake != Wake::Fire(id));
        let mut causes = vec![fire.lit_event];
        if let Some(cause) = extra_cause {
            causes.push(cause);
        }
        self.mint_item(Material::Charcoal, 0, fire.cell, None, Role::Loose);
        self.emit(
            text,
            Scope::Cell(fire.cell),
            vec![],
            vec![fire.cell],
            vec![],
            causes,
            None,
        );
    }

    pub(crate) fn gripping(&self, actor: ActorId) -> Option<ItemId> {
        self.grips.iter().find(|g| g.actor == actor).map(|g| g.item)
    }

    pub(crate) fn log_at(&self, cell: CellId) -> bool {
        self.pile_count(cell, Material::Log) > 0
            || self
                .items
                .values()
                .any(|i| i.material == Material::Log && i.cell == cell && i.carrier.is_none())
    }

    pub(crate) fn log_item(&self) -> Option<&Item> {
        self.items.values().find(|i| i.material == Material::Log)
    }

    pub(crate) fn construction_at(&self, cell: CellId) -> Option<&Construction> {
        self.constructions.values().find(|c| c.cell == cell)
    }

    pub(crate) fn condition_word(&self, moisture: u32) -> &'static str {
        if moisture >= self.tuning.soaked_at {
            "soaked"
        } else if moisture >= self.tuning.damp_at {
            "damp"
        } else {
            "dry"
        }
    }

    /// Clothes take the rain as soon as it reaches the body. A dry word
    /// means the rain has not touched them. Shade does not wipe a burn.
    pub(crate) fn clothes_word(&self, moisture: u32) -> &'static str {
        if moisture == 0 {
            "dry"
        } else if moisture >= self.tuning.soaked_at {
            "soaked"
        } else if moisture >= self.tuning.damp_at {
            "damp"
        } else {
            "wet"
        }
    }

    pub(crate) fn thirst_word(&self, thirst: u32) -> &'static str {
        if thirst >= self.tuning.parched_at {
            "parched"
        } else if thirst >= self.tuning.thirsty_at {
            "thirsty"
        } else {
            "not thirsty"
        }
    }

    pub(crate) fn thirst_sentence(&self, thirst: u32) -> &'static str {
        match self.thirst_word(thirst) {
            "parched" => "parched",
            "thirsty" => "thirsty",
            _ => "not thirsty",
        }
    }

    pub(crate) fn hunger_word(&self, hunger: u32) -> &'static str {
        if hunger >= self.tuning.starving_at {
            "starving"
        } else if hunger >= self.tuning.hungry_at {
            "hungry"
        } else {
            "not hungry"
        }
    }

    pub(crate) fn illness_word(&self, illness: u32) -> &'static str {
        if illness >= self.tuning.ill_at {
            "ill"
        } else {
            "well"
        }
    }

    pub(crate) fn energy_word(&self, energy: u32) -> &'static str {
        if energy >= self.tuning.spent_at {
            "spent"
        } else if energy >= self.tuning.tiring_at {
            "tiring"
        } else {
            "rested"
        }
    }

    pub(crate) fn cold_word(&self, cold: u32) -> &'static str {
        if cold >= self.tuning.cold_at {
            "cold"
        } else if cold >= self.tuning.chilly_at {
            "chilly"
        } else {
            "warm"
        }
    }

    pub(crate) fn sun_word(&self, sunburn: u32) -> &'static str {
        if sunburn >= self.tuning.burned_at {
            "burned"
        } else if sunburn >= self.tuning.pink_at {
            "pink"
        } else {
            "unburned"
        }
    }

    pub(crate) fn mud_word(&self, mud: u32) -> &'static str {
        if mud == 0 {
            "bare skin"
        } else if mud < self.tuning.smear_at {
            "thin mud"
        } else {
            "mud-smeared"
        }
    }

    pub(crate) fn ochre_word(&self, ochre: u32) -> &'static str {
        if ochre == 0 {
            "bare"
        } else if ochre < self.tuning.smear_at {
            "thin red ochre"
        } else {
            "red-ochre-smeared"
        }
    }

    pub(crate) fn wind_word(&self, exposure: u32) -> &'static str {
        if exposure == 0 {
            "out of the wind"
        } else if exposure < 1000 {
            "in a little wind"
        } else {
            "in the wind"
        }
    }

    /// The wind on a body. A thatched lean-to is out of the wind. The rock
    /// is too, and a bare lean-to is only a little wind.
    pub(crate) fn wind_at(&self, cell: CellId) -> &'static str {
        if self.thatched.contains(&cell) {
            "out of the wind"
        } else {
            self.wind_word(self.exposure_millis(cell))
        }
    }

    pub(crate) fn water_here(&self, cell: CellId) -> bool {
        self.cell(cell).region == Region::Brook || self.cell(cell).still
    }

    /// Still water freezes once the evening has chilled. The brook keeps
    /// running. A fire on the pool keeps it open.
    pub(crate) fn still_frozen(&self, cell: CellId) -> bool {
        self.time >= self.tuning.chill_at
            && self.cell(cell).still
            && self.cell(cell).region != Region::Brook
            && self.fire_at(cell).is_none()
    }

    pub(crate) fn has_brush(&self, cell: CellId) -> bool {
        self.pile_count(cell, Material::Branch) > 0
            || self.items.values().any(|item| {
                item.cell == cell
                    && item.material == Material::Branch
                    && item.role == Role::Loose
                    && item.carrier.is_none()
            })
    }

    pub(crate) fn cookable_carried(&self, actor: ActorId) -> bool {
        !self.loose_ids(actor, Material::Carcass).is_empty()
            || self.raw_mushroom_id(actor).is_some()
            || self.raw_egg_id(actor).is_some()
            || !self.loose_ids(actor, Material::Berry).is_empty()
    }

    /// Raw eggs in the arms. Cooked eggs are already food.
    pub(crate) fn raw_egg_id(&self, actor: ActorId) -> Option<ItemId> {
        self.loose_ids(actor, Material::Egg)
            .into_iter()
            .find(|id| self.item(*id).is_some_and(|eggs| !eggs.cooked))
    }

    /// Cooked eggs, from the arms before the ground.
    pub(crate) fn cooked_egg_id(&self, actor: ActorId) -> Option<ItemId> {
        let carried = self
            .loose_ids(actor, Material::Egg)
            .into_iter()
            .find(|id| self.item(*id).is_some_and(|eggs| eggs.cooked));
        if carried.is_some() {
            return carried;
        }
        let cell = self.actor(actor)?.cell;
        self.ground_ids(cell, Material::Egg)
            .into_iter()
            .find(|id| self.item(*id).is_some_and(|eggs| eggs.cooked))
    }

    pub(crate) fn raw_egg_near(&self, actor: ActorId) -> bool {
        if self.raw_egg_id(actor).is_some() {
            return true;
        }
        let Some(cell) = self.actor(actor).map(|person| person.cell) else {
            return false;
        };
        self.pile_count(cell, Material::Egg) > 0
            || self
                .ground_ids(cell, Material::Egg)
                .iter()
                .any(|id| self.item(*id).is_some_and(|eggs| !eggs.cooked))
    }

    /// A raw mushroom in the arms. A cooked one is already food.
    pub(crate) fn raw_mushroom_id(&self, actor: ActorId) -> Option<ItemId> {
        self.loose_ids(actor, Material::Mushroom)
            .into_iter()
            .find(|id| self.item(*id).is_some_and(|mushroom| !mushroom.cooked))
    }

    /// One cooked mushroom, from the arms before the ground.
    pub(crate) fn cooked_mushroom_id(&self, actor: ActorId) -> Option<ItemId> {
        let carried = self
            .loose_ids(actor, Material::Mushroom)
            .into_iter()
            .find(|id| self.item(*id).is_some_and(|mushroom| mushroom.cooked));
        if carried.is_some() {
            return carried;
        }
        let cell = self.actor(actor)?.cell;
        self.ground_ids(cell, Material::Mushroom)
            .into_iter()
            .find(|id| self.item(*id).is_some_and(|mushroom| mushroom.cooked))
    }

    pub(crate) fn raw_mushroom_near(&self, actor: ActorId) -> bool {
        if self.raw_mushroom_id(actor).is_some() {
            return true;
        }
        let Some(cell) = self.actor(actor).map(|person| person.cell) else {
            return false;
        };
        self.pile_count(cell, Material::Mushroom) > 0
            || self
                .ground_ids(cell, Material::Mushroom)
                .iter()
                .any(|id| self.item(*id).is_some_and(|mushroom| !mushroom.cooked))
    }

    pub(crate) fn cookable_pile(&self, cell: CellId) -> u32 {
        let piles: u32 = [Material::Mushroom, Material::Berry, Material::Egg]
            .into_iter()
            .map(|material| self.pile_count(cell, material))
            .sum();
        piles + self.ground_ids(cell, Material::Carcass).len() as u32
    }

    pub(crate) fn wearing_hide(&self, actor: ActorId) -> bool {
        self.loose_ids(actor, Material::Hide)
            .iter()
            .any(|id| self.item(*id).is_some_and(|hide| hide.worn))
    }

    /// One piece of cooked meat, from the arms before the ground.
    pub(crate) fn cooked_meat_id(&self, actor: ActorId) -> Option<ItemId> {
        let carried = self
            .loose_ids(actor, Material::Meat)
            .into_iter()
            .find(|id| self.item(*id).is_some_and(|meat| meat.cooked));
        if carried.is_some() {
            return carried;
        }
        let cell = self.actor(actor)?.cell;
        self.ground_ids(cell, Material::Meat)
            .into_iter()
            .find(|id| self.item(*id).is_some_and(|meat| meat.cooked))
    }

    /// Salted raw meat, from the arms before the ground. It can be eaten cold.
    pub(crate) fn salted_meat_id(&self, actor: ActorId) -> Option<ItemId> {
        let carried = self
            .loose_ids(actor, Material::Meat)
            .into_iter()
            .find(|id| {
                self.item(*id)
                    .is_some_and(|meat| meat.salted && !meat.cooked)
            });
        if carried.is_some() {
            return carried;
        }
        let cell = self.actor(actor)?.cell;
        self.ground_ids(cell, Material::Meat)
            .into_iter()
            .find(|id| {
                self.item(*id)
                    .is_some_and(|meat| meat.salted && !meat.cooked)
            })
    }

    pub(crate) fn raw_meat_near(&self, actor: ActorId) -> bool {
        self.meat_near(actor, |meat| !meat.cooked && !meat.salted && !meat.spoiled)
    }

    pub(crate) fn spoiled_meat_near(&self, actor: ActorId) -> bool {
        self.meat_near(actor, |meat| meat.spoiled && !meat.cooked)
    }

    fn meat_near(&self, actor: ActorId, keep: impl Fn(&Item) -> bool) -> bool {
        for id in self.loose_ids(actor, Material::Meat) {
            if self.item(id).is_some_and(|meat| keep(meat)) {
                return true;
            }
        }
        let Some(cell) = self.actor(actor).map(|person| person.cell) else {
            return false;
        };
        for id in self.ground_ids(cell, Material::Meat) {
            if self.item(id).is_some_and(|meat| keep(meat)) {
                return true;
            }
        }
        false
    }

    /// A raw fish in the arms, else one on this cell. A spoiled fish is not raw.
    pub(crate) fn raw_fish_id(&self, actor: ActorId) -> Option<ItemId> {
        let carried = self
            .loose_ids(actor, Material::Fish)
            .into_iter()
            .find(|id| {
                self.item(*id)
                    .is_some_and(|fish| !fish.cooked && !fish.salted && !fish.spoiled)
            });
        if carried.is_some() {
            return carried;
        }
        let cell = self.actor(actor)?.cell;
        self.ground_ids(cell, Material::Fish)
            .into_iter()
            .find(|id| {
                self.item(*id)
                    .is_some_and(|fish| !fish.cooked && !fish.salted && !fish.spoiled)
            })
    }

    /// A salted fish in the arms, else one on this cell. It can be eaten cold.
    pub(crate) fn salted_fish_id(&self, actor: ActorId) -> Option<ItemId> {
        let carried = self
            .loose_ids(actor, Material::Fish)
            .into_iter()
            .find(|id| {
                self.item(*id)
                    .is_some_and(|fish| fish.salted && !fish.cooked)
            });
        if carried.is_some() {
            return carried;
        }
        let cell = self.actor(actor)?.cell;
        self.ground_ids(cell, Material::Fish)
            .into_iter()
            .find(|id| {
                self.item(*id)
                    .is_some_and(|fish| fish.salted && !fish.cooked)
            })
    }

    pub(crate) fn spoiled_fish_near(&self, actor: ActorId) -> bool {
        let carried = self.loose_ids(actor, Material::Fish).iter().any(|id| {
            self.item(*id)
                .is_some_and(|fish| fish.spoiled && !fish.cooked)
        });
        if carried {
            return true;
        }
        let Some(cell) = self.actor(actor).map(|person| person.cell) else {
            return false;
        };
        self.ground_ids(cell, Material::Fish).iter().any(|id| {
            self.item(*id)
                .is_some_and(|fish| fish.spoiled && !fish.cooked)
        })
    }

    pub(crate) fn cooked_fish_id(&self, actor: ActorId) -> Option<ItemId> {
        let carried = self
            .loose_ids(actor, Material::Fish)
            .into_iter()
            .find(|id| self.item(*id).is_some_and(|fish| fish.cooked));
        if carried.is_some() {
            return carried;
        }
        let cell = self.actor(actor)?.cell;
        self.ground_ids(cell, Material::Fish)
            .into_iter()
            .find(|id| self.item(*id).is_some_and(|fish| fish.cooked))
    }

    pub(crate) fn raw_fish_near(&self, actor: ActorId) -> bool {
        self.raw_fish_id(actor).is_some()
    }

    /// In the flowing brook, or on a cell that touches it. Still water is not the brook.
    pub(crate) fn fishing_water(&self, cell: CellId) -> bool {
        self.brook_stretch(cell).is_some()
    }

    /// The brook cell a cast from here takes a fish from. Standing in the
    /// brook, that cell. Standing on the bank, the brook cell beside it.
    pub(crate) fn brook_stretch(&self, cell: CellId) -> Option<CellId> {
        if self.cell(cell).region == Region::Brook {
            return Some(cell);
        }
        let (x, y) = cell_xy(cell);
        [(1, 0), (-1, 0), (0, 1), (0, -1)]
            .iter()
            .filter_map(|(dx, dy)| cell_id(x + dx, y + dy))
            .find(|next| self.cell(*next).region == Region::Brook)
    }

    /// The deer left this cell, or a bolt or a kill left tracks here.
    /// Look names them through the next `track_ticks` steps.
    pub(crate) fn leave_fresh_tracks(&mut self, cell: CellId) {
        let until = self
            .time
            .saturating_add(1)
            .saturating_add(self.tuning.track_ticks);
        let entry = self.fresh_tracks.entry(cell).or_insert(0);
        if *entry < until {
            *entry = until;
        }
    }

    pub(crate) fn deer_tracks_fresh(&self, cell: CellId) -> bool {
        self.fresh_tracks
            .get(&cell)
            .is_some_and(|until| self.time < *until)
    }

    /// The howl has just been heard. The tracks are on that corner for
    /// `track_ticks` clocks, counting this one. A later howl starts the wait again.
    pub(crate) fn mark_wolf_tracks(&mut self, cell: CellId) {
        let until = self.time.saturating_add(self.tuning.track_ticks);
        let entry = self.wolf_tracks.entry(cell).or_insert(0);
        if *entry < until {
            *entry = until;
        }
    }

    pub(crate) fn wolf_tracks_fresh(&self, cell: CellId) -> bool {
        self.wolf_tracks
            .get(&cell)
            .is_some_and(|until| self.time < *until)
    }

    /// A successful cast empties that stretch until this time. The next
    /// cast on the step that begins then can take another fish.
    pub(crate) fn brook_is_quiet(&self, cell: CellId) -> bool {
        self.brook_quiet
            .get(&cell)
            .is_some_and(|until| self.time < *until)
    }

    /// One piece of raw meat lying on this cell. Cooked meat is not raw.
    /// Spoiled meat is not raw either. It is not food.
    pub(crate) fn raw_meat_at(&self, cell: CellId) -> Option<ItemId> {
        self.ground_ids(cell, Material::Meat)
            .into_iter()
            .find(|id| {
                self.item(*id)
                    .is_some_and(|meat| !meat.cooked && !meat.spoiled)
            })
    }

    pub(crate) fn name_of(&self, id: ActorId) -> String {
        self.actors
            .get(&id)
            .map(|a| a.name.clone())
            .unwrap_or_else(|| "someone".into())
    }

    pub(crate) fn names(&self, ids: &[ActorId]) -> String {
        let mut names: Vec<String> = ids.iter().map(|id| self.name_of(*id)).collect();
        names.sort();
        match names.len() {
            0 => "nobody".into(),
            1 => names[0].clone(),
            2 => format!("{} and {}", names[0], names[1]),
            _ => {
                let last = names[names.len() - 1].clone();
                format!("{}, and {last}", names[..names.len() - 1].join(", "))
            }
        }
    }

    pub(crate) fn move_actor(&mut self, actor: ActorId, dest: CellId) {
        if let Some(person) = self.actors.get_mut(&actor) {
            person.cell = dest;
        }
        let carried: Vec<ItemId> = self
            .items
            .values()
            .filter(|i| i.carrier == Some(actor))
            .map(|i| i.id)
            .collect();
        for id in carried {
            if let Some(item) = self.items.get_mut(&id) {
                item.cell = dest;
            }
        }
    }

    pub(crate) fn emit(
        &mut self,
        text: impl Into<String>,
        scope: Scope,
        mut actors: Vec<ActorId>,
        mut cells: Vec<CellId>,
        mut items: Vec<ItemId>,
        mut causes: Vec<EventId>,
        draw: Option<Draw>,
    ) -> EventId {
        actors.sort_unstable();
        actors.dedup();
        cells.sort_unstable();
        cells.dedup();
        items.sort_unstable();
        items.dedup();
        causes.sort_unstable();
        causes.dedup();
        let id = self.next_event;
        self.next_event += 1;
        self.events.push(Event {
            id,
            time: self.time,
            text: text.into(),
            scope,
            affected_actors: actors,
            affected_cells: cells,
            affected_items: items,
            causes,
            draw,
        });
        id
    }

    pub(crate) fn finish(
        &mut self,
        command_id: String,
        actor_id: ActorId,
        success: bool,
        message: impl Into<String>,
        results: &mut Vec<CommandResult>,
    ) {
        let result = CommandResult {
            command_id,
            actor_id,
            success,
            message: message.into(),
        };
        self.applied
            .insert(result.command_id.clone(), result.clone());
        results.push(result);
    }

    pub(crate) fn relative(&self, from: CellId, to: CellId) -> String {
        if from == to {
            return "here".into();
        }
        let (x1, y1) = cell_xy(from);
        let (x2, y2) = cell_xy(to);
        let dx = x2 - x1;
        let dy = y2 - y1;
        let ns = if dy < 0 {
            "north"
        } else if dy > 0 {
            "south"
        } else {
            ""
        };
        let ew = if dx > 0 {
            "east"
        } else if dx < 0 {
            "west"
        } else {
            ""
        };
        match (ns, ew) {
            ("", "") => "here".into(),
            (ns, "") => ns.into(),
            ("", ew) => ew.into(),
            (ns, ew) => format!("{ns} and {ew}"),
        }
    }

    fn normalize(&mut self) {
        self.piles.sort_by_key(|p| (p.cell, p.material as u8));
        self.grips.sort_by_key(|g| (g.item, g.actor));
        self.schedule.sort_by_key(|s| (s.at, s.seq));
    }

    fn trim_events(&mut self) {
        if self.events.len() > EVENT_CAP {
            let extra = self.events.len() - EVENT_CAP;
            self.events.drain(0..extra);
        }
    }

    pub(crate) fn wet_add(&self, gain: u32, exposure: u32) -> u32 {
        ((gain as u64) * (exposure as u64) / 1000) as u32
    }
}

fn default_next_id() -> u32 {
    1
}

fn clean_name(name: &str) -> Result<String, String> {
    let name = name.trim();
    let chars: Vec<char> = name.chars().collect();
    if chars.is_empty() || chars.len() > 24 {
        return Err("use a name of 1 to 24 characters".into());
    }
    if !chars[0].is_ascii_alphabetic() {
        return Err("a name starts with a letter".into());
    }
    if !chars
        .iter()
        .all(|c| c.is_ascii_alphanumeric() || *c == ' ' || *c == '-' || *c == '\'')
    {
        return Err("use letters, numbers, spaces, hyphens, or apostrophes".into());
    }
    Ok(name.to_string())
}

pub fn contest_hash(seed: u64, time: u64, tag: &str, subject: &str, actor: u32) -> u64 {
    let mut hash = 0xcbf29ce484222325u64;
    feed(&mut hash, &seed.to_le_bytes());
    feed(&mut hash, &time.to_le_bytes());
    feed(&mut hash, tag.as_bytes());
    feed(&mut hash, subject.as_bytes());
    feed(&mut hash, &actor.to_le_bytes());
    hash
}

fn feed(hash: &mut u64, bytes: &[u8]) {
    for byte in bytes {
        *hash ^= *byte as u64;
        *hash = hash.wrapping_mul(0x100000001b3);
    }
    *hash ^= 0xff;
    *hash = hash.wrapping_mul(0x100000001b3);
}

#[cfg(test)]
mod salt_meat {
    use super::{Attempt, ExamineTarget, Intent, Material, Role, World};

    fn intent(id: &str, actor: u32, attempt: Attempt) -> Intent {
        Intent {
            command_id: id.into(),
            actor_id: actor,
            attempt,
        }
    }

    #[test]
    fn a_salt_crust_on_the_slope_makes_raw_meat_edible_cold() {
        let mut world = World::valley(crate::tuning::Tuning::evening(), 1);
        let near = world.spawn_at("Near", "n", 0, 0).unwrap();
        let seen = world.spawn_at("West", "w", 1, 0).unwrap();
        let far = world.spawn_at("Far", "f", 6, 3).unwrap();
        let here = world.observe(near).unwrap().text;
        assert!(here.contains("A salt crust lies here."), "{here}");
        let from_west = world.observe(seen).unwrap().text;
        assert!(
            from_west.contains("A salt crust lies on the slope to the west."),
            "{from_west}"
        );
        assert!(!world.observe(far).unwrap().text.contains("salt"));
        let bare = world.step(&[intent("bare", near, Attempt::SaltMeat)]);
        assert_eq!(
            bare.results[0].message,
            "There is no raw meat here to salt."
        );
        assert!(world
            .observe(seen)
            .unwrap()
            .events
            .iter()
            .all(|event| !event.text.contains("salt")));
        let gathered = world.step(&[intent("gather", near, Attempt::Gather(Material::Salt))]);
        assert_eq!(gathered.results[0].message, "You gather a salt crust.");
        assert!(world.observe(near).unwrap().text.contains("a salt crust"));
        let meat = world.mint_item(
            Material::Meat,
            0,
            world.actor(near).unwrap().cell,
            None,
            Role::Loose,
        );
        let yours = "You salt a portion of raw meat. It will keep, and it can be eaten cold.";
        let witnessed = "West salts a portion of raw meat. It will keep, and it can be eaten cold.";
        let _ = witnessed;
        let salted = world.step(&[intent("salt", near, Attempt::SaltMeat)]);
        assert_eq!(salted.results[0].message, yours);
        assert!(world
            .observe(near)
            .unwrap()
            .events
            .iter()
            .all(|event| event.text != yours && !event.text.contains("Near salts")));
        assert!(world
            .observe(seen)
            .unwrap()
            .events
            .iter()
            .any(|event| event.text
                == "Near salts a portion of raw meat. It will keep, and it can be eaten cold."));
        assert!(world
            .observe(far)
            .unwrap()
            .events
            .iter()
            .all(|event| !event.text.contains("salt")));
        let ground = world.observe(near).unwrap().text;
        assert!(
            ground.contains("A portion of salted meat lies here. It can be eaten cold."),
            "{ground}"
        );
        assert_eq!(world.material_tally(Material::Salt), 0);
        assert!(world.item(meat).unwrap().salted);
        let eaten = world.step(&[intent("eat", near, Attempt::Eat(Material::Meat))]);
        assert!(
            eaten.results[0]
                .message
                .starts_with("You eat the salted meat."),
            "{}",
            eaten.results[0].message
        );
        assert_eq!(world.material_tally(Material::Meat), 0);
        let raw = world.mint_item(
            Material::Meat,
            0,
            world.actor(near).unwrap().cell,
            None,
            Role::Loose,
        );
        let refused = world.step(&[intent("raw", near, Attempt::Eat(Material::Meat))]);
        assert_eq!(refused.results[0].message, "Raw meat is not food.");
        assert!(world
            .observe(seen)
            .unwrap()
            .events
            .iter()
            .all(|event| !event.text.contains("not food")));
        let _ = (raw, ExamineTarget::Salt);
        assert!(world
            .examine(near, ExamineTarget::Salt)
            .contains("You see no salt from here."));
    }
}

#[cfg(test)]
mod spoil_meat {
    use super::{Attempt, ExamineTarget, Intent, Material, Role, World};

    fn intent(id: &str, actor: u32, attempt: Attempt) -> Intent {
        Intent {
            command_id: id.into(),
            actor_id: actor,
            attempt,
        }
    }

    #[test]
    fn raw_meat_spoils_and_salted_meat_keeps() {
        let mut world = World::valley(crate::tuning::Tuning::evening(), 1);
        let near = world.spawn_at("Near", "n", 4, 3).unwrap();
        let seen = world.spawn_at("West", "w", 5, 3).unwrap();
        let quiet = world.spawn_at("Quiet", "q", 0, 4).unwrap();
        let cell = world.actor(near).unwrap().cell;
        let raw = world.mint_item(Material::Meat, 0, cell, None, Role::Loose);
        let salted = world.mint_item(Material::Meat, 0, cell, None, Role::Loose);
        if let Some(meat) = world.items.get_mut(&salted) {
            meat.salted = true;
            meat.spoils_at = None;
        }
        let cooked = world.mint_item(Material::Meat, 0, cell, None, Role::Loose);
        if let Some(meat) = world.items.get_mut(&cooked) {
            meat.cooked = true;
            meat.spoils_at = None;
        }
        let ticks = world.tuning.meat_ticks;
        assert_eq!(ticks, 24);
        assert_eq!(world.item(raw).unwrap().spoils_at, Some(ticks));
        for _ in 0..ticks {
            world.step(&[]);
        }
        assert_eq!(world.sim_time(), ticks);
        let going = world.observe(near).unwrap().text;
        assert!(
            going.contains("A portion of raw meat lies here. It is going bad."),
            "{going}"
        );
        assert!(
            going.contains("A portion of salted meat lies here. It can be eaten cold."),
            "{going}"
        );
        assert!(
            going.contains("A portion of cooked meat lies here."),
            "{going}"
        );
        assert!(!going.contains("spoiled"), "{going}");
        let examined = world.examine(near, ExamineTarget::Meat);
        assert!(
            examined.contains("A portion of raw meat lies here, in the rain. It is going bad."),
            "{examined}"
        );
        assert!(
            examined
                .contains("A portion of salted meat lies here, in the rain. It can be eaten cold."),
            "{examined}"
        );
        assert!(
            examined.contains("A portion of cooked meat lies here, in the rain."),
            "{examined}"
        );
        world.step(&[]);
        let spoiled = "A portion of raw meat spoils. It is not food.";
        let here = world.observe(near).unwrap();
        assert!(here.events.iter().any(|event| event.text == spoiled));
        assert!(
            here.text
                .contains("A portion of spoiled meat lies here. It is not food."),
            "{}",
            here.text
        );
        assert!(
            here.text
                .contains("A portion of salted meat lies here. It can be eaten cold."),
            "{}",
            here.text
        );
        assert!(
            here.text.contains("A portion of cooked meat lies here."),
            "{}",
            here.text
        );
        assert!(world
            .observe(seen)
            .unwrap()
            .events
            .iter()
            .any(|event| event.text == spoiled));
        assert!(world
            .observe(quiet)
            .unwrap()
            .events
            .iter()
            .all(|event| { event.text != spoiled && !event.text.contains("spoiled") }));
        assert!(!world.observe(quiet).unwrap().text.contains("meat"));
        let cooked_bite = world.step(&[intent("cooked", near, Attempt::Eat(Material::Meat))]);
        assert!(
            cooked_bite.results[0]
                .message
                .starts_with("You eat the cooked meat."),
            "{}",
            cooked_bite.results[0].message
        );
        let kept = world.step(&[intent("keep", near, Attempt::Eat(Material::Meat))]);
        assert!(
            kept.results[0]
                .message
                .starts_with("You eat the salted meat."),
            "{}",
            kept.results[0].message
        );
        let refused = world.step(&[intent("bad", near, Attempt::Eat(Material::Meat))]);
        assert!(!refused.results[0].success);
        assert_eq!(refused.results[0].message, "The meat has spoiled.");
        assert!(world
            .observe(seen)
            .unwrap()
            .events
            .iter()
            .all(|event| event.text != "The meat has spoiled."));
        assert!(world.item(raw).unwrap().spoiled);
        assert_eq!(world.material_tally(Material::Meat), 1);
    }
}

#[cfg(test)]
mod spoil_fish {
    use super::{Attempt, ExamineTarget, Intent, Material, Role, World};

    fn intent(id: &str, actor: u32, attempt: Attempt) -> Intent {
        Intent {
            command_id: id.into(),
            actor_id: actor,
            attempt,
        }
    }

    #[test]
    fn raw_fish_spoils_and_cooked_fish_stays_good() {
        let mut world = World::valley(crate::tuning::Tuning::evening(), 1);
        let near = world.spawn_at("Near", "n", 4, 3).unwrap();
        let seen = world.spawn_at("West", "w", 5, 3).unwrap();
        let quiet = world.spawn_at("Quiet", "q", 0, 4).unwrap();
        let cell = world.actor(near).unwrap().cell;
        let raw = world.mint_item(Material::Fish, 0, cell, None, Role::Loose);
        let cooked = world.mint_item(Material::Fish, 0, cell, None, Role::Loose);
        if let Some(fish) = world.items.get_mut(&cooked) {
            fish.cooked = true;
            fish.spoils_at = None;
        }
        let ticks = world.tuning.meat_ticks;
        assert_eq!(world.item(raw).unwrap().spoils_at, Some(ticks));
        for _ in 0..ticks {
            world.step(&[]);
        }
        let going = world.observe(near).unwrap().text;
        assert!(
            going.contains("A raw fish lies here. It is going bad."),
            "{going}"
        );
        assert!(going.contains("A cooked fish lies here."), "{going}");
        assert!(!going.contains("spoiled"), "{going}");
        let examined = world.examine(near, ExamineTarget::Fish);
        assert!(
            examined.contains(
                "A raw fish lies in contact with the ground here, in the rain. It is going bad."
            ),
            "{examined}"
        );
        assert!(
            examined.contains(
                "A cooked fish lies in contact with the ground here, in the rain. It is good to eat."
            ),
            "{examined}"
        );
        world.step(&[]);
        let spoiled = "A raw fish spoils. It is not food.";
        let here = world.observe(near).unwrap();
        assert!(here.events.iter().any(|event| event.text == spoiled));
        assert!(
            here.text
                .contains("A spoiled fish lies here. It is not food."),
            "{}",
            here.text
        );
        assert!(
            here.text.contains("A cooked fish lies here."),
            "{}",
            here.text
        );
        assert!(world
            .observe(seen)
            .unwrap()
            .events
            .iter()
            .any(|event| event.text == spoiled));
        assert!(world
            .observe(quiet)
            .unwrap()
            .events
            .iter()
            .all(|event| event.text != spoiled && !event.text.contains("fish spoils")));
        assert!(!world.observe(quiet).unwrap().text.contains("spoiled fish"));
        let cooked_bite = world.step(&[intent("cooked", near, Attempt::Eat(Material::Fish))]);
        assert!(
            cooked_bite.results[0]
                .message
                .starts_with("You eat the cooked fish."),
            "{}",
            cooked_bite.results[0].message
        );
        let refused = world.step(&[intent("bad", near, Attempt::Eat(Material::Fish))]);
        assert!(!refused.results[0].success);
        assert_eq!(refused.results[0].message, "The fish has spoiled.");
        assert!(world
            .observe(seen)
            .unwrap()
            .events
            .iter()
            .all(|event| event.text != "The fish has spoiled."));
        assert!(world.item(raw).unwrap().spoiled);
        assert!(world.item(cooked).is_none());
    }
}

#[cfg(test)]
mod salt_fish {
    use super::{Attempt, ExamineTarget, Intent, Material, Role, World};

    fn intent(id: &str, actor: u32, attempt: Attempt) -> Intent {
        Intent {
            command_id: id.into(),
            actor_id: actor,
            attempt,
        }
    }

    #[test]
    fn a_salt_crust_keeps_a_raw_fish_and_it_can_be_eaten_cold() {
        let mut world = World::valley(crate::tuning::Tuning::evening(), 1);
        let near = world.spawn_at("Near", "n", 0, 0).unwrap();
        let seen = world.spawn_at("West", "w", 1, 0).unwrap();
        let quiet = world.spawn_at("Quiet", "q", 6, 4).unwrap();
        let bare = world.step(&[intent("bare", near, Attempt::SaltFish)]);
        assert_eq!(
            bare.results[0].message,
            "There is no raw fish here to salt."
        );
        assert!(world
            .observe(seen)
            .unwrap()
            .events
            .iter()
            .all(|event| event.text != "There is no raw fish here to salt."));
        let cell = world.actor(near).unwrap().cell;
        let kept = world.mint_item(Material::Fish, 0, cell, None, Role::Loose);
        let yours = "You salt a raw fish. It will keep, and it can be eaten cold.";
        let salted = world.step(&[intent("salt", near, Attempt::SaltFish)]);
        assert_eq!(salted.results[0].message, yours);
        assert!(world.item(kept).unwrap().salted);
        assert!(world.item(kept).unwrap().spoils_at.is_none());
        assert!(world
            .observe(near)
            .unwrap()
            .events
            .iter()
            .all(|event| event.text != yours && !event.text.contains("Near salts")));
        assert!(world.observe(seen).unwrap().events.iter().any(|event| {
            event.text == "Near salts a raw fish. It will keep, and it can be eaten cold."
        }));
        assert!(world
            .observe(quiet)
            .unwrap()
            .events
            .iter()
            .all(|event| !event.text.contains("salts a raw fish")));
        world.mint_item(Material::Salt, 0, cell, Some(near), Role::Loose);
        let again = world.step(&[intent("again", near, Attempt::SaltFish)]);
        assert_eq!(again.results[0].message, "The fish is already salted.");
        assert!(world
            .observe(seen)
            .unwrap()
            .events
            .iter()
            .all(|event| event.text != "The fish is already salted."));
        let raw = world.mint_item(Material::Fish, 0, cell, None, Role::Loose);
        let ticks = world.tuning.meat_ticks;
        for _ in 0..ticks {
            world.step(&[]);
        }
        let going = world.observe(near).unwrap().text;
        assert!(
            going.contains("A salted fish lies here. It can be eaten cold."),
            "{going}"
        );
        assert!(
            going.contains("A raw fish lies here. It is going bad."),
            "{going}"
        );
        world.step(&[]);
        let here = world.observe(near).unwrap();
        assert!(here
            .events
            .iter()
            .any(|event| event.text == "A raw fish spoils. It is not food."));
        assert!(
            here.text
                .contains("A salted fish lies here. It can be eaten cold."),
            "{}",
            here.text
        );
        assert!(
            here.text
                .contains("A spoiled fish lies here. It is not food."),
            "{}",
            here.text
        );
        assert!(world
            .observe(quiet)
            .unwrap()
            .events
            .iter()
            .all(|event| event.text != "A raw fish spoils. It is not food."));
        let examined = world.examine(near, ExamineTarget::Fish);
        assert!(examined.contains("It can be eaten cold."), "{examined}");
        let eaten = world.step(&[intent("eat", near, Attempt::Eat(Material::Fish))]);
        assert!(
            eaten.results[0]
                .message
                .starts_with("You eat the salted fish."),
            "{}",
            eaten.results[0].message
        );
        assert!(world
            .observe(seen)
            .unwrap()
            .events
            .iter()
            .any(|event| event.text == "Near eats the salted fish."));
        assert!(world
            .observe(near)
            .unwrap()
            .events
            .iter()
            .any(|event| event.text == "You eat the salted fish."));
        assert!(world
            .observe(near)
            .unwrap()
            .events
            .iter()
            .all(|event| !event.text.contains("Near eats the salted")));
        assert!(world
            .observe(quiet)
            .unwrap()
            .events
            .iter()
            .all(|event| !event.text.contains("eats the salted fish")));
        world.mint_item(Material::Salt, 0, cell, Some(near), Role::Loose);
        let spoiled = world.step(&[intent("bad", near, Attempt::SaltFish)]);
        assert_eq!(spoiled.results[0].message, "The fish has spoiled.");
        assert!(world
            .observe(seen)
            .unwrap()
            .events
            .iter()
            .all(|event| event.text != "The fish has spoiled."));
        assert!(world.item(raw).unwrap().spoiled);
        assert!(!world.item(raw).unwrap().salted);
    }
}

#[cfg(test)]
mod salt_lick {
    use super::{Attempt, Intent, Material, World};

    fn intent(id: &str, actor: u32, attempt: Attempt) -> Intent {
        Intent {
            command_id: id.into(),
            actor_id: actor,
            attempt,
        }
    }

    #[test]
    fn the_slope_is_bare_until_the_salt_crust_forms_again() {
        let mut world = World::valley(crate::tuning::Tuning::evening(), 1);
        let near = world.spawn_at("Near", "n", 0, 0).unwrap();
        let seen = world.spawn_at("West", "w", 1, 0).unwrap();
        let quiet = world.spawn_at("Quiet", "q", 6, 4).unwrap();
        let here = world.observe(near).unwrap().text;
        assert!(here.contains("A salt crust lies here."), "{here}");
        assert!(!here.contains("The slope is bare."), "{here}");
        let from_west = world.observe(seen).unwrap().text;
        assert!(
            from_west.contains("A salt crust lies on the slope to the west."),
            "{from_west}"
        );
        assert!(!world.observe(quiet).unwrap().text.contains("salt crust"));
        let gathered = world.step(&[intent("gather", near, Attempt::Gather(Material::Salt))]);
        assert_eq!(gathered.results[0].message, "You gather a salt crust.");
        let bare = world.observe(near).unwrap().text;
        assert!(bare.contains("The slope is bare."), "{bare}");
        assert!(!bare.contains("A salt crust lies here."), "{bare}");
        let west_bare = world.observe(seen).unwrap().text;
        assert!(
            west_bare.contains("The slope to the west is bare."),
            "{west_bare}"
        );
        assert!(
            !west_bare.contains("A salt crust lies on the slope"),
            "{west_bare}"
        );
        let missed = world.observe(quiet).unwrap().text;
        assert!(!missed.contains("The slope is bare."), "{missed}");
        assert!(!missed.contains("is bare."), "{missed}");
        assert!(!missed.contains("salt crust"), "{missed}");
        let ticks = world.tuning.salt_ticks;
        for _ in 0..ticks - 1 {
            world.step(&[]);
        }
        assert!(world
            .observe(near)
            .unwrap()
            .text
            .contains("The slope is bare."),);
        world.step(&[]);
        let formed = "A salt crust forms on the slope.";
        let back = world.observe(near).unwrap();
        assert!(
            back.text.contains("A salt crust lies here."),
            "{}",
            back.text
        );
        assert!(!back.text.contains("The slope is bare."), "{}", back.text);
        assert!(back.events.iter().any(|event| event.text == formed));
        let west = world.observe(seen).unwrap();
        assert!(
            west.text
                .contains("A salt crust lies on the slope to the west."),
            "{}",
            west.text
        );
        assert!(west.events.iter().any(|event| event.text == formed));
        assert!(world
            .observe(quiet)
            .unwrap()
            .events
            .iter()
            .all(|event| event.text != formed));
        assert!(!world.observe(quiet).unwrap().text.contains("salt crust"));
        let again = world.step(&[intent("again", near, Attempt::Gather(Material::Salt))]);
        assert_eq!(again.results[0].message, "You gather a salt crust.");
        assert!(world
            .observe(near)
            .unwrap()
            .text
            .contains("The slope is bare."));
        assert_eq!(world.material_tally(Material::Salt), 2);
    }
}

#[cfg(test)]
mod honey {
    use super::{Attempt, Intent, Material, World};

    fn intent(id: &str, actor: u32, attempt: Attempt) -> Intent {
        Intent {
            command_id: id.into(),
            actor_id: actor,
            attempt,
        }
    }

    #[test]
    fn a_bees_nest_in_the_east_tree_feeds_and_stings() {
        let mut world = World::valley(crate::tuning::Tuning::evening(), 1);
        let near = world.spawn_at("Near", "n", 6, 0).unwrap();
        let seen = world.spawn_at("West", "w", 5, 0).unwrap();
        let quiet = world.spawn_at("Quiet", "q", 2, 3).unwrap();
        let here = world.observe(near).unwrap().text;
        assert!(
            here.contains("A bees' nest hangs in the tree here."),
            "{here}"
        );
        let from_west = world.observe(seen).unwrap().text;
        assert!(
            from_west.contains("A bees' nest hangs in a tree on the slope to the east."),
            "{from_west}"
        );
        let missed = world.observe(quiet).unwrap().text;
        assert!(!missed.contains("bees"), "{missed}");
        let none = world.step(&[intent("none", quiet, Attempt::Gather(Material::Honey))]);
        assert_eq!(none.results[0].message, "There is no honey here to gather.");
        assert!(world
            .observe(near)
            .unwrap()
            .events
            .iter()
            .all(|event| event.text != "There is no honey here to gather."));
        let bare = world.step(&[intent("bare", near, Attempt::Eat(Material::Honey))]);
        assert_eq!(bare.results[0].message, "You are not carrying honey.");
        assert!(world
            .observe(seen)
            .unwrap()
            .events
            .iter()
            .all(|event| event.text != "You are not carrying honey."));
        let yours = "You gather a honeycomb. The bees sting.";
        let gathered = world.step(&[intent("gather", near, Attempt::Gather(Material::Honey))]);
        assert_eq!(gathered.results[0].message, yours);
        assert!(world
            .observe(near)
            .unwrap()
            .events
            .iter()
            .all(|event| { event.text != yours && !event.text.contains("Near gathers") }));
        assert!(world
            .observe(seen)
            .unwrap()
            .events
            .iter()
            .any(|event| event.text == "Near gathers a honeycomb. The bees sting."));
        assert!(world
            .observe(quiet)
            .unwrap()
            .events
            .iter()
            .all(|event| !event.text.contains("honeycomb") && !event.text.contains("sting")));
        let empty = world.observe(near).unwrap().text;
        assert!(empty.contains("The bees' nest is empty."), "{empty}");
        assert!(!empty.contains("A bees' nest hangs"), "{empty}");
        assert!(empty.contains("a honeycomb"), "{empty}");
        let west_empty = world.observe(seen).unwrap().text;
        assert!(
            west_empty.contains("The bees' nest in the tree on the slope to the east is empty."),
            "{west_empty}"
        );
        if let Some(person) = world.actors.get_mut(&near) {
            person.hunger = world.tuning.hungry_at;
        }
        let eaten = world.step(&[intent("eat", near, Attempt::Eat(Material::Honey))]);
        assert_eq!(
            eaten.results[0].message,
            "You eat the honeycomb. The hunger eases."
        );
        assert!(world
            .observe(near)
            .unwrap()
            .events
            .iter()
            .any(|event| event.text == "You eat the honeycomb."));
        assert!(world
            .observe(near)
            .unwrap()
            .events
            .iter()
            .all(|event| !event.text.contains("Near eats")));
        assert!(world
            .observe(seen)
            .unwrap()
            .events
            .iter()
            .any(|event| event.text == "Near eats the honeycomb."));
        assert!(world
            .observe(quiet)
            .unwrap()
            .events
            .iter()
            .all(|event| !event.text.contains("honeycomb")));
        assert_eq!(world.material_tally(Material::Honey), 0);
    }

    #[test]
    fn bee_stings_show_on_the_body_and_then_fade() {
        let mut world = World::valley(crate::tuning::Tuning::evening(), 1);
        let near = world.spawn_at("Near", "n", 6, 0).unwrap();
        let seen = world.spawn_at("West", "w", 5, 0).unwrap();
        let quiet = world.spawn_at("Quiet", "q", 2, 3).unwrap();
        if let Some(person) = world.actors.get_mut(&near) {
            person.mud = 400;
            person.ochre = 400;
        }
        let gathered = world.step(&[intent("gather", near, Attempt::Gather(Material::Honey))]);
        assert_eq!(
            gathered.results[0].message,
            "You gather a honeycomb. The bees sting."
        );
        let body = world.observe(near).unwrap();
        assert!(body.text.contains("You are stung."), "{}", body.text);
        assert!(!body.text.contains("You are well."), "{}", body.text);
        assert_eq!(body.self_info.illness, "stung");
        let exam = world.examine(near, super::ExamineTarget::SelfBody);
        assert!(exam.contains("You are stung."), "{exam}");
        assert!(!exam.contains("You are well."), "{exam}");
        let west = world.examine(seen, super::ExamineTarget::Person("Near".into()));
        assert!(west.contains("They are stung."), "{west}");
        let missed = world.observe(quiet).unwrap().text;
        assert!(!missed.contains("stung"), "{missed}");
        assert_eq!(
            world.examine(quiet, super::ExamineTarget::Person("Near".into())),
            "You cannot see Near from here."
        );
        assert!(body.text.contains("thin mud"), "{}", body.text);
        assert!(world.actor(near).unwrap().mud > 0);
        assert!(world.actor(near).unwrap().ochre > 0);
        let mut faded = false;
        for _ in 0..8 {
            world.step(&[]);
            let look = world.observe(near).unwrap();
            if look.text.contains("You are well.") {
                faded = true;
                assert!(!look.text.contains("You are stung."), "{}", look.text);
                assert_eq!(look.self_info.illness, "well");
                assert!(look
                    .events
                    .iter()
                    .any(|event| event.text == "The stings fade."));
                assert!(look
                    .events
                    .iter()
                    .all(|event| event.text != "The stings on Near fade."));
                let west = world.observe(seen).unwrap();
                assert!(west
                    .events
                    .iter()
                    .any(|event| event.text == "The stings on Near fade."));
                assert!(world
                    .observe(quiet)
                    .unwrap()
                    .events
                    .iter()
                    .all(|event| !event.text.contains("sting")));
                break;
            }
            assert!(look.text.contains("You are stung."), "{}", look.text);
        }
        assert!(faded);
        assert!(world.actor(near).unwrap().mud > 0);
        assert!(world.actor(near).unwrap().ochre > 0);
        assert!(world.actor(near).unwrap().stings_until.is_none());
    }

    #[test]
    fn a_brand_smokes_the_nest_and_the_next_gather_does_not_sting() {
        let mut world = World::valley(crate::tuning::Tuning::evening(), 1);
        let near = world.spawn_at("Near", "n", 6, 0).unwrap();
        let seen = world.spawn_at("West", "w", 5, 0).unwrap();
        let quiet = world.spawn_at("Quiet", "q", 2, 3).unwrap();
        let none = world.step(&[intent("none", near, Attempt::SmokeNest)]);
        assert_eq!(
            none.results[0].message,
            "You have no smoke to quiet the bees."
        );
        assert!(world
            .observe(seen)
            .unwrap()
            .events
            .iter()
            .all(|event| event.text != "You have no smoke to quiet the bees."));
        let cell = world.actor(near).unwrap().cell;
        world.mint_item(Material::Brand, 0, cell, Some(near), super::Role::Loose);
        let smoked = world.step(&[intent("smoke", near, Attempt::SmokeNest)]);
        assert_eq!(
            smoked.results[0].message,
            "You smoke the bees' nest. The bees quiet."
        );
        let yours = "You smoke the bees' nest. The bees quiet.";
        let named = "Near smokes the bees' nest. The bees quiet.";
        assert!(world
            .observe(near)
            .unwrap()
            .events
            .iter()
            .all(|event| { event.text != yours && event.text != named }));
        assert!(world
            .observe(seen)
            .unwrap()
            .events
            .iter()
            .any(|event| event.text == named));
        assert!(world
            .observe(quiet)
            .unwrap()
            .events
            .iter()
            .all(|event| !event.text.contains("smoke")));
        let here = world.observe(near).unwrap().text;
        assert!(here.contains("Smoke hangs in the bees' nest."), "{here}");
        assert!(here.contains("You are well."), "{here}");
        let west = world.observe(seen).unwrap().text;
        assert!(west.contains("Smoke hangs in it."), "{west}");
        assert!(!world.observe(quiet).unwrap().text.contains("Smoke"));
        let gathered = world.step(&[intent("gather", near, Attempt::Gather(Material::Honey))]);
        assert_eq!(gathered.results[0].message, "You gather a honeycomb.");
        let body = world.observe(near).unwrap();
        assert!(body.text.contains("You are well."), "{}", body.text);
        assert!(!body.text.contains("You are stung."), "{}", body.text);
        assert!(body.self_info.illness != "stung");
        assert!(world.actor(near).unwrap().stings_until.is_none());
        assert!(body
            .events
            .iter()
            .all(|event| !event.text.contains("Near gathers") && !event.text.contains("sting")));
        assert!(world
            .observe(seen)
            .unwrap()
            .events
            .iter()
            .any(|event| event.text == "Near gathers a honeycomb."));
        assert!(world
            .observe(seen)
            .unwrap()
            .events
            .iter()
            .all(|event| { !event.text.contains("sting") }));
        assert!(world
            .observe(quiet)
            .unwrap()
            .events
            .iter()
            .all(|event| !event.text.contains("honeycomb") && !event.text.contains("sting")));
    }

    #[test]
    fn a_fire_next_to_the_nest_smokes_it_without_a_brand() {
        let mut world = World::valley(crate::tuning::Tuning::evening(), 1);
        let near = world.spawn_at("Near", "n", 6, 0).unwrap();
        world.mint_fire(super::cell_id(6, 1).unwrap(), 0);
        let smoked = world.step(&[intent("smoke", near, Attempt::SmokeNest)]);
        assert_eq!(
            smoked.results[0].message,
            "You smoke the bees' nest. The bees quiet."
        );
        let gathered = world.step(&[intent("gather", near, Attempt::Gather(Material::Honey))]);
        assert_eq!(gathered.results[0].message, "You gather a honeycomb.");
        assert!(world.actor(near).unwrap().stings_until.is_none());
        assert!(!world.observe(near).unwrap().text.contains("You are stung."));
    }

    #[test]
    fn the_bees_nest_is_empty_until_it_fills_again() {
        let mut world = World::valley(crate::tuning::Tuning::evening(), 1);
        let near = world.spawn_at("Near", "n", 6, 0).unwrap();
        let seen = world.spawn_at("West", "w", 5, 0).unwrap();
        let quiet = world.spawn_at("Quiet", "q", 2, 3).unwrap();
        let gathered = world.step(&[intent("gather", near, Attempt::Gather(Material::Honey))]);
        assert_eq!(
            gathered.results[0].message,
            "You gather a honeycomb. The bees sting."
        );
        let empty = world.observe(near).unwrap().text;
        assert!(empty.contains("The bees' nest is empty."), "{empty}");
        assert!(!empty.contains("A bees' nest hangs"), "{empty}");
        let west_empty = world.observe(seen).unwrap().text;
        assert!(
            west_empty.contains("The bees' nest in the tree on the slope to the east is empty."),
            "{west_empty}"
        );
        assert!(!world.observe(quiet).unwrap().text.contains("bees"));
        let ticks = world.tuning.honey_ticks;
        for _ in 0..ticks - 1 {
            world.step(&[]);
        }
        assert!(world
            .observe(near)
            .unwrap()
            .text
            .contains("The bees' nest is empty."));
        world.step(&[]);
        let filled = "The bees' nest fills again.";
        let back = world.observe(near).unwrap();
        assert!(
            back.text.contains("A bees' nest hangs in the tree here."),
            "{}",
            back.text
        );
        assert!(
            !back.text.contains("The bees' nest is empty."),
            "{}",
            back.text
        );
        assert!(back.events.iter().any(|event| event.text == filled));
        let west = world.observe(seen).unwrap();
        assert!(
            west.text
                .contains("A bees' nest hangs in a tree on the slope to the east."),
            "{}",
            west.text
        );
        assert!(west.events.iter().any(|event| event.text == filled));
        assert!(world
            .observe(quiet)
            .unwrap()
            .events
            .iter()
            .all(|event| event.text != filled));
        assert!(!world.observe(quiet).unwrap().text.contains("bees"));
        let again = world.step(&[intent("again", near, Attempt::Gather(Material::Honey))]);
        assert_eq!(
            again.results[0].message,
            "You gather a honeycomb. The bees sting."
        );
        assert!(world
            .observe(near)
            .unwrap()
            .text
            .contains("The bees' nest is empty."));
    }

    #[test]
    fn two_honeycombs_in_the_arms_stay_honeycombs() {
        let mut world = World::valley(crate::tuning::Tuning::evening(), 1);
        let near = world.spawn_at("Near", "n", 6, 0).unwrap();
        let seen = world.spawn_at("West", "w", 5, 0).unwrap();
        let quiet = world.spawn_at("Quiet", "q", 2, 3).unwrap();
        world.step(&[intent("gather", near, Attempt::Gather(Material::Honey))]);
        let one = world.observe(near).unwrap().text;
        assert!(one.contains("You have a honeycomb."), "{one}");
        assert!(!one.contains("2 honey"), "{one}");
        let exam = world.examine(near, super::ExamineTarget::Honey);
        assert!(
            exam.contains("The bees' nest in the open here is empty."),
            "{exam}"
        );
        assert!(
            exam.contains("A honeycomb is carried by you in the open."),
            "{exam}"
        );
        assert!(!exam.contains("here, in the open is empty"), "{exam}");
        assert!(!exam.contains("carried by you,"), "{exam}");
        let west = world.examine(seen, super::ExamineTarget::Honey);
        assert!(
            west.contains("The bees' nest in the open to the east is empty."),
            "{west}"
        );
        assert!(
            west.contains("A honeycomb is carried by Near in the open."),
            "{west}"
        );
        assert_eq!(
            world.examine(quiet, super::ExamineTarget::Honey),
            "You see no honey from here."
        );
        let cell = world.actor(near).unwrap().cell;
        world.mint_item(Material::Honey, 0, cell, Some(near), super::Role::Loose);
        let two = world.observe(near).unwrap().text;
        assert!(two.contains("You have two honeycombs."), "{two}");
        assert!(!two.contains("2 honey"), "{two}");
        assert!(!two.contains("carried by you (dry)"), "{two}");
        let exam = world.examine(near, super::ExamineTarget::Honey);
        assert!(
            exam.contains("Two honeycombs are carried by you in the open."),
            "{exam}"
        );
        assert!(!exam.contains("carried by you,"), "{exam}");
        let west = world.examine(seen, super::ExamineTarget::Honey);
        assert!(
            west.contains("Two honeycombs are carried by Near in the open."),
            "{west}"
        );
        assert!(!west.contains("2 honey"), "{west}");
        world.raining = true;
        let rain = world.examine(near, super::ExamineTarget::Honey);
        assert!(
            rain.contains("The bees' nest in the rain here is empty."),
            "{rain}"
        );
        assert!(
            rain.contains("Two honeycombs are carried by you in the rain."),
            "{rain}"
        );
        assert!(!rain.contains(", in the rain"), "{rain}");
        let west_rain = world.examine(seen, super::ExamineTarget::Honey);
        assert!(
            west_rain.contains("The bees' nest in the rain to the east is empty."),
            "{west_rain}"
        );
        assert!(
            west_rain.contains("Two honeycombs are carried by Near in the rain."),
            "{west_rain}"
        );
        assert_eq!(
            world.examine(quiet, super::ExamineTarget::Honey),
            "You see no honey from here."
        );
    }
}

#[cfg(test)]
mod names {
    use super::{Attempt, World};

    #[test]
    fn examine_out_of_sight_uses_the_display_name() {
        let mut world = World::valley(crate::tuning::Tuning::evening(), 1);
        let near = world.spawn_at("Near", "n", 6, 0).unwrap();
        let seen = world.spawn_at("Seen", "w", 5, 0).unwrap();
        let quiet = world.spawn_at("Quiet", "q", 2, 3).unwrap();
        let _mckay = world.spawn_at("McKay", "m", 0, 0).unwrap();
        let Attempt::Examine(lower) = crate::parse::parse_attempt("examine near").unwrap() else {
            panic!("examine near");
        };
        assert_eq!(
            world.examine(quiet, lower),
            "You cannot see Near from here."
        );
        let Attempt::Examine(typed) = crate::parse::parse_attempt("examine Near").unwrap() else {
            panic!("examine Near");
        };
        assert_eq!(
            world.examine(quiet, typed),
            "You cannot see Near from here."
        );
        let Attempt::Examine(seen_name) = crate::parse::parse_attempt("examine seen").unwrap()
        else {
            panic!("examine seen");
        };
        assert_eq!(
            world.examine(quiet, seen_name),
            "You cannot see Seen from here."
        );
        let Attempt::Examine(mixed) = crate::parse::parse_attempt("examine mckay").unwrap() else {
            panic!("examine mckay");
        };
        assert_eq!(
            world.examine(quiet, mixed.clone()),
            "You cannot see McKay from here."
        );
        assert_eq!(
            world.examine(near, mixed),
            "You cannot see McKay from here."
        );
        let west = world.examine(seen, super::ExamineTarget::Person("near".into()));
        assert!(
            west.starts_with("Near is on the slope, in the open."),
            "{west}"
        );
        assert!(!west.contains("cannot see"), "{west}");
        assert!(!west.contains('('), "{west}");
        world.actors.get_mut(&near).unwrap().cell = super::cell_id(6, 1).unwrap();
        let under = world.examine(seen, super::ExamineTarget::Person("near".into()));
        assert!(
            under.contains("on the valley floor, under the rock overhang."),
            "{under}"
        );
        assert!(!under.contains('('), "{under}");
        world.mint_fire(super::cell_id(6, 1).unwrap(), 0);
        let fire = world.examine(seen, super::ExamineTarget::Fire);
        assert!(fire.contains("under the rock overhang"), "{fire}");
        assert!(!fire.contains('('), "{fire}");
        let from_quiet = world.examine(quiet, super::ExamineTarget::Fire);
        assert_eq!(from_quiet, "You see no fire from here.");
        let self_look = world.examine(quiet, super::ExamineTarget::Person("quiet".into()));
        assert!(!self_look.contains("cannot see"), "{self_look}");
        assert!(self_look.contains("You are"), "{self_look}");
        assert!(!self_look.contains('('), "{self_look}");
    }
}

#[cfg(test)]
mod eggs {
    use super::{Attempt, Intent, Material, World};

    fn intent(id: &str, actor: u32, attempt: Attempt) -> Intent {
        Intent {
            command_id: id.into(),
            actor_id: actor,
            attempt,
        }
    }

    #[test]
    fn a_birds_nest_on_the_west_slope_feeds_once_cooked() {
        let mut world = World::valley(crate::tuning::Tuning::evening(), 1);
        let near = world.spawn_at("Near", "n", 1, 0).unwrap();
        let seen = world.spawn_at("West", "w", 2, 0).unwrap();
        let quiet = world.spawn_at("Quiet", "q", 5, 3).unwrap();
        let here = world.observe(near).unwrap().text;
        assert!(
            here.contains("A bird's nest sits in the tree here. It holds eggs."),
            "{here}"
        );
        let west = world.observe(seen).unwrap().text;
        assert!(
            west.contains("A bird's nest sits in a tree on the slope to the west."),
            "{west}"
        );
        assert!(!world.observe(quiet).unwrap().text.contains("bird"));
        let none = world.step(&[intent("none", quiet, Attempt::Gather(Material::Egg))]);
        assert_eq!(none.results[0].message, "There are no eggs here to gather.");
        assert!(world
            .observe(near)
            .unwrap()
            .events
            .iter()
            .all(|event| event.text != "There are no eggs here to gather."));
        let gathered = world.step(&[intent("gather", near, Attempt::Gather(Material::Egg))]);
        assert_eq!(gathered.results[0].message, "You gather the eggs.");
        let after = world.observe(near).unwrap();
        assert!(
            after
                .text
                .contains("The bird's nest in the tree here is empty."),
            "{}",
            after.text
        );
        assert!(after.text.contains("You have eggs."), "{}", after.text);
        assert!(after
            .events
            .iter()
            .all(|event| !event.text.contains("gathers the eggs")));
        let seen_after = world.observe(seen).unwrap();
        assert!(seen_after
            .events
            .iter()
            .any(|event| event.text == "Near gathers the eggs."));
        assert!(
            seen_after
                .text
                .contains("The bird's nest in the tree on the slope to the west is empty."),
            "{}",
            seen_after.text
        );
        let quiet_events: Vec<_> = world
            .observe(quiet)
            .unwrap()
            .events
            .iter()
            .map(|event| event.text.clone())
            .collect();
        assert!(quiet_events
            .iter()
            .any(|event| event == "There are no eggs here to gather."));
        assert!(quiet_events
            .iter()
            .all(|event| !event.contains("gathers the eggs")));
        let raw = world.step(&[intent("raw", near, Attempt::Eat(Material::Egg))]);
        assert_eq!(raw.results[0].message, "The eggs are raw.");
        assert!(world
            .observe(seen)
            .unwrap()
            .events
            .iter()
            .all(|event| event.text != "The eggs are raw."));
        let nofire = world.step(&[intent("cold", near, Attempt::Cook)]);
        assert_eq!(nofire.results[0].message, "There is no fire burning here.");
        assert!(world
            .observe(seen)
            .unwrap()
            .events
            .iter()
            .all(|event| event.text != "There is no fire burning here."));
        world.mint_fire(world.actor(near).unwrap().cell, 0);
        let cooked = world.step(&[intent("cook", near, Attempt::Cook)]);
        assert_eq!(
            cooked.results[0].message,
            "You cook the eggs over the fire."
        );
        let cooked_look = world.observe(near).unwrap();
        assert!(cooked_look
            .events
            .iter()
            .any(|event| event.text == "You cook the eggs over the fire."));
        assert!(cooked_look
            .events
            .iter()
            .all(|event| !event.text.starts_with("Near cooks")));
        assert!(
            cooked_look.text.contains("cooked eggs"),
            "{}",
            cooked_look.text
        );
        assert!(world
            .observe(seen)
            .unwrap()
            .events
            .iter()
            .any(|event| event.text == "Near cooks the eggs over the fire."));
        assert!(world
            .observe(quiet)
            .unwrap()
            .events
            .iter()
            .all(|event| !event.text.contains("cooks the eggs")));
        if let Some(person) = world.actors.get_mut(&near) {
            person.hunger = world.tuning.hungry_at;
        }
        let eaten = world.step(&[intent("eat", near, Attempt::Eat(Material::Egg))]);
        assert_eq!(
            eaten.results[0].message,
            "You eat the cooked eggs. The hunger eases."
        );
        let fed = world.observe(near).unwrap();
        assert!(fed
            .events
            .iter()
            .any(|event| event.text == "You eat the cooked eggs."));
        assert!(world
            .observe(seen)
            .unwrap()
            .events
            .iter()
            .any(|event| event.text == "Near eats the cooked eggs."));
        assert!(world
            .observe(seen)
            .unwrap()
            .events
            .iter()
            .all(|event| event.text != "The hunger eases."));
        assert!(world
            .observe(quiet)
            .unwrap()
            .events
            .iter()
            .all(|event| !event.text.contains("eats the cooked eggs")
                && event.text != "The hunger eases."));
    }

    #[test]
    fn the_birds_nest_is_empty_until_it_holds_eggs_again() {
        let mut world = World::valley(crate::tuning::Tuning::evening(), 1);
        let near = world.spawn_at("Near", "n", 1, 0).unwrap();
        let seen = world.spawn_at("Seen", "s", 2, 0).unwrap();
        let quiet = world.spawn_at("Quiet", "q", 5, 3).unwrap();
        let gathered = world.step(&[intent("gather", near, Attempt::Gather(Material::Egg))]);
        assert_eq!(gathered.results[0].message, "You gather the eggs.");
        let empty = world.observe(near).unwrap().text;
        assert!(
            empty.contains("The bird's nest in the tree here is empty."),
            "{empty}"
        );
        assert!(!empty.contains("It holds eggs."), "{empty}");
        let west_empty = world.observe(seen).unwrap().text;
        assert!(
            west_empty.contains("The bird's nest in the tree on the slope to the west is empty."),
            "{west_empty}"
        );
        assert!(!world.observe(quiet).unwrap().text.contains("bird"));
        let ticks = world.tuning.egg_ticks;
        for _ in 0..ticks - 1 {
            world.step(&[]);
        }
        assert!(world
            .observe(near)
            .unwrap()
            .text
            .contains("The bird's nest in the tree here is empty."));
        world.step(&[]);
        let filled = "The bird's nest fills again.";
        let back = world.observe(near).unwrap();
        assert!(
            back.text
                .contains("A bird's nest sits in the tree here. It holds eggs."),
            "{}",
            back.text
        );
        assert!(
            !back
                .text
                .contains("The bird's nest in the tree here is empty."),
            "{}",
            back.text
        );
        assert!(back.events.iter().any(|event| event.text == filled));
        let west = world.observe(seen).unwrap();
        assert!(
            west.text
                .contains("A bird's nest sits in a tree on the slope to the west."),
            "{}",
            west.text
        );
        assert!(west.events.iter().any(|event| event.text == filled));
        assert!(world
            .observe(quiet)
            .unwrap()
            .events
            .iter()
            .all(|event| event.text != filled));
        assert!(!world.observe(quiet).unwrap().text.contains("bird"));
        let again = world.step(&[intent("again", near, Attempt::Gather(Material::Egg))]);
        assert_eq!(again.results[0].message, "You gather the eggs.");
        let taken = world.observe(near).unwrap();
        assert!(
            taken
                .text
                .contains("The bird's nest in the tree here is empty."),
            "{}",
            taken.text
        );
        assert!(taken
            .events
            .iter()
            .all(|event| !event.text.contains("gathers the eggs")));
        assert!(world
            .observe(seen)
            .unwrap()
            .events
            .iter()
            .any(|event| event.text == "Near gathers the eggs."));
        assert!(world
            .observe(quiet)
            .unwrap()
            .events
            .iter()
            .all(|event| !event.text.contains("eggs")));
    }
}

#[cfg(test)]
mod brook_clothes {
    use super::{Attempt, Dir, ExamineTarget, Intent, World};

    fn intent(id: &str, actor: u32, attempt: Attempt) -> Intent {
        Intent {
            command_id: id.into(),
            actor_id: actor,
            attempt,
        }
    }

    #[test]
    fn a_dry_sky_wade_names_the_brook() {
        let mut world = World::valley(crate::tuning::Tuning::evening(), 1);
        let near = world.spawn_at("Near", "n", 2, 1).unwrap();
        let seen = world.spawn_at("Seen", "s", 2, 1).unwrap();
        let quiet = world.spawn_at("Quiet", "q", 5, 3).unwrap();
        let wade = world.step(&[intent("west", near, Attempt::Move(Dir::West))]);
        assert_eq!(wade.results[0].message, "You wade west.");
        assert!(!world.raining);
        let look = world.observe(near).unwrap().text;
        assert!(look.contains("The brook wets your clothes."), "{look}");
        assert!(!look.contains("The rain wets your clothes."), "{look}");
        let seen_body = world.examine(seen, ExamineTarget::Person("Near".into()));
        assert!(
            seen_body.contains("Their clothes are wet from the brook."),
            "{seen_body}"
        );
        assert!(!seen_body.contains("rain"), "{seen_body}");
        assert_eq!(
            world.examine(quiet, ExamineTarget::Person("Near".into())),
            "You cannot see Near from here."
        );
        let quiet_look = world.observe(quiet).unwrap().text;
        assert!(quiet_look.contains("Your clothes are dry."), "{quiet_look}");
        assert!(!quiet_look.contains("brook"), "{quiet_look}");
        world.raining = true;
        let raining = world.observe(near).unwrap().text;
        assert!(raining.contains("The rain wets your clothes."), "{raining}");
        assert!(
            !raining.contains("The brook wets your clothes."),
            "{raining}"
        );
    }
}

#[cfg(test)]
mod moss {
    use super::{Attempt, ExamineTarget, Intent, Material, World};

    fn intent(id: &str, actor: u32, attempt: Attempt) -> Intent {
        Intent {
            command_id: id.into(),
            actor_id: actor,
            attempt,
        }
    }

    #[test]
    fn moss_on_the_damp_bank_clears_a_sting() {
        assert!(matches!(
            crate::parse::parse_attempt("press moss").unwrap(),
            Attempt::DressSting
        ));
        assert!(matches!(
            crate::parse::parse_attempt("dress sting").unwrap(),
            Attempt::DressSting
        ));
        assert!(matches!(
            crate::parse::parse_attempt("use moss").unwrap(),
            Attempt::DressSting
        ));
        let mut world = World::valley(crate::tuning::Tuning::evening(), 1);
        let near = world.spawn_at("Near", "n", 1, 3).unwrap();
        let seen = world.spawn_at("Seen", "s", 3, 3).unwrap();
        let quiet = world.spawn_at("Quiet", "q", 5, 3).unwrap();
        let here = world.observe(near).unwrap().text;
        assert!(here.contains("Moss grows on the damp bank here."), "{here}");
        let west = world.observe(seen).unwrap().text;
        assert!(
            west.contains("Moss grows on the damp bank to the west."),
            "{west}"
        );
        assert!(!world.observe(quiet).unwrap().text.contains("moss"));
        let none = world.step(&[intent("none", quiet, Attempt::Gather(Material::Moss))]);
        assert_eq!(none.results[0].message, "There is no moss here to gather.");
        assert!(world
            .observe(near)
            .unwrap()
            .events
            .iter()
            .all(|event| event.text != "There is no moss here to gather."));
        let bare = world.step(&[intent("bare", near, Attempt::DressSting)]);
        assert_eq!(bare.results[0].message, "You have no moss.");
        assert!(world
            .observe(seen)
            .unwrap()
            .events
            .iter()
            .all(|event| event.text != "You have no moss."));
        let gathered = world.step(&[intent("gather", near, Attempt::Gather(Material::Moss))]);
        assert_eq!(gathered.results[0].message, "You gather a handful of moss.");
        let taken = world.observe(near).unwrap();
        assert!(
            taken.text.contains("You have a handful of moss."),
            "{}",
            taken.text
        );
        assert!(
            taken.text.contains("The damp bank here is bare."),
            "{}",
            taken.text
        );
        assert!(!taken.text.contains("Moss grows"), "{}", taken.text);
        assert!(taken
            .events
            .iter()
            .all(|event| !event.text.contains("gathers")));
        assert!(world
            .observe(seen)
            .unwrap()
            .events
            .iter()
            .any(|event| event.text == "Near gathers a handful of moss."));
        let seen_bare = world.observe(seen).unwrap().text;
        assert!(
            seen_bare.contains("The damp bank to the west is bare."),
            "{seen_bare}"
        );
        assert!(!seen_bare.contains("Moss grows"));
        assert!(!world.observe(quiet).unwrap().text.contains("damp bank"));
        assert!(world
            .observe(quiet)
            .unwrap()
            .events
            .iter()
            .any(|event| event.text == "There is no moss here to gather."));
        assert!(world
            .observe(quiet)
            .unwrap()
            .events
            .iter()
            .all(|event| !event.text.contains("gathers")));
        let clear = world.step(&[intent("clear", near, Attempt::DressSting)]);
        assert_eq!(clear.results[0].message, "You are not stung.");
        assert!(world
            .observe(seen)
            .unwrap()
            .events
            .iter()
            .all(|event| event.text != "You are not stung."));
        if let Some(person) = world.actors.get_mut(&near) {
            person.stings_until = Some(world.time.saturating_add(4));
        }
        let stung = world.observe(near).unwrap().text;
        assert!(stung.contains("You are stung."), "{stung}");
        let seen_body = world.examine(seen, ExamineTarget::Person("Near".into()));
        assert!(seen_body.contains("They are stung."), "{seen_body}");
        let dressed = world.step(&[intent("dress", near, Attempt::DressSting)]);
        assert_eq!(dressed.results[0].message, "You press moss on the stings.");
        let eased = world.observe(near).unwrap();
        assert!(eased.text.contains("You are well."), "{}", eased.text);
        assert!(!eased.text.contains("You are stung."), "{}", eased.text);
        assert!(!eased.text.contains("handful of moss"), "{}", eased.text);
        assert!(eased
            .events
            .iter()
            .all(|event| !event.text.contains("press")));
        assert!(world.actor(near).unwrap().stings_until.is_none());
        assert!(world
            .observe(seen)
            .unwrap()
            .events
            .iter()
            .any(|event| event.text == "Near presses moss on the stings."));
        let after = world.examine(seen, ExamineTarget::Person("Near".into()));
        assert!(after.contains("They are well."), "{after}");
        assert!(!after.contains("stung"), "{after}");
        assert!(world
            .observe(quiet)
            .unwrap()
            .events
            .iter()
            .all(|event| !event.text.contains("press") && !event.text.contains("sting")));
        let spent = world.step(&[intent("spent", near, Attempt::DressSting)]);
        assert_eq!(spent.results[0].message, "You have no moss.");
    }

    #[test]
    fn a_press_spends_one_handful_and_leaves_the_other() {
        let mut world = World::valley(crate::tuning::Tuning::evening(), 1);
        let near = world.spawn_at("Near", "n", 1, 3).unwrap();
        let seen = world.spawn_at("Seen", "s", 3, 3).unwrap();
        let quiet = world.spawn_at("Quiet", "q", 5, 3).unwrap();
        let bank = world.actor(near).unwrap().cell;
        let first = world.step(&[intent("first", near, Attempt::Gather(Material::Moss))]);
        assert_eq!(first.results[0].message, "You gather a handful of moss.");
        world.moss_returns_at.remove(&bank);
        world.add_pile(bank, Material::Moss, 1, 0);
        let second = world.step(&[intent("second", near, Attempt::Gather(Material::Moss))]);
        assert_eq!(second.results[0].message, "You gather a handful of moss.");
        assert_eq!(world.loose_ids(near, Material::Moss).len(), 2);
        let before = world.observe(near).unwrap().text;
        assert!(
            before.contains("You have two handfuls of moss."),
            "{before}"
        );
        let seen_before = world.examine(seen, ExamineTarget::Moss);
        assert!(
            seen_before.contains("Two handfuls of moss are carried by Near"),
            "{seen_before}"
        );
        let near_before = world.examine(near, ExamineTarget::Moss);
        assert!(
            near_before.contains("Two handfuls of moss are carried by you"),
            "{near_before}"
        );
        assert_eq!(
            world.examine(quiet, ExamineTarget::Moss),
            "You see no moss from here."
        );
        if let Some(person) = world.actors.get_mut(&near) {
            person.stings_until = Some(world.time.saturating_add(4));
        }
        let pressed = world.step(&[intent("press", near, Attempt::DressSting)]);
        assert_eq!(pressed.results[0].message, "You press moss on the stings.");
        assert_eq!(world.loose_ids(near, Material::Moss).len(), 1);
        assert!(world.actor(near).unwrap().stings_until.is_none());
        let after = world.observe(near).unwrap();
        assert!(
            after.text.contains("You have a handful of moss."),
            "{}",
            after.text
        );
        assert!(after.text.contains("You are well."), "{}", after.text);
        assert!(!after.text.contains("two handfuls"), "{}", after.text);
        assert!(!after.text.contains("2 moss"), "{}", after.text);
        assert!(after
            .events
            .iter()
            .all(|event| !event.text.contains("press")));
        let near_after = world.examine(near, ExamineTarget::Moss);
        assert!(
            near_after.contains("A handful of moss is carried by you"),
            "{near_after}"
        );
        assert!(!near_after.contains("Two handfuls"), "{near_after}");
        let seen_after = world.examine(seen, ExamineTarget::Moss);
        assert!(
            seen_after.contains("A handful of moss is carried by Near"),
            "{seen_after}"
        );
        assert!(!seen_after.contains("Two handfuls"), "{seen_after}");
        assert!(world
            .observe(seen)
            .unwrap()
            .events
            .iter()
            .any(|event| event.text == "Near presses moss on the stings."));
        let seen_body = world.examine(seen, ExamineTarget::Person("Near".into()));
        assert!(seen_body.contains("They are well."), "{seen_body}");
        assert_eq!(
            world.examine(quiet, ExamineTarget::Moss),
            "You see no moss from here."
        );
        assert!(world
            .observe(quiet)
            .unwrap()
            .events
            .iter()
            .all(|event| !event.text.contains("press")));
        let held = world.step(&[intent("held", near, Attempt::DressSting)]);
        assert_eq!(held.results[0].message, "You are not stung.");
        assert_eq!(world.loose_ids(near, Material::Moss).len(), 1);
        assert!(world
            .observe(near)
            .unwrap()
            .text
            .contains("You have a handful of moss."));
    }

    #[test]
    fn the_damp_bank_is_bare_until_moss_grows_again() {
        let mut world = World::valley(crate::tuning::Tuning::evening(), 1);
        let near = world.spawn_at("Near", "n", 1, 3).unwrap();
        let seen = world.spawn_at("Seen", "s", 3, 3).unwrap();
        let quiet = world.spawn_at("Quiet", "q", 5, 3).unwrap();
        let gathered = world.step(&[intent("gather", near, Attempt::Gather(Material::Moss))]);
        assert_eq!(gathered.results[0].message, "You gather a handful of moss.");
        let empty = world.observe(near).unwrap().text;
        assert!(empty.contains("The damp bank here is bare."), "{empty}");
        assert!(empty.contains("You have a handful of moss."), "{empty}");
        assert!(!empty.contains("2 moss"), "{empty}");
        assert!(!empty.contains("Moss grows"), "{empty}");
        let one = world.examine(near, super::ExamineTarget::Moss);
        assert!(one.contains("A handful of moss is carried by you"), "{one}");
        let seen_one = world.examine(seen, super::ExamineTarget::Moss);
        assert!(
            seen_one.contains("A handful of moss is carried by Near"),
            "{seen_one}"
        );
        let west = world.observe(seen).unwrap().text;
        assert!(
            west.contains("The damp bank to the west is bare."),
            "{west}"
        );
        assert!(!world.observe(quiet).unwrap().text.contains("damp bank"));
        let ticks = world.tuning.moss_ticks;
        for _ in 0..ticks - 1 {
            world.step(&[]);
        }
        assert!(world
            .observe(near)
            .unwrap()
            .text
            .contains("The damp bank here is bare."));
        world.step(&[]);
        let grown = "Moss grows on the damp bank again.";
        let back = world.observe(near).unwrap();
        assert!(
            back.text.contains("Moss grows on the damp bank here."),
            "{}",
            back.text
        );
        assert!(
            !back.text.contains("The damp bank here is bare."),
            "{}",
            back.text
        );
        assert!(back.events.iter().any(|event| event.text == grown));
        let seen_back = world.observe(seen).unwrap();
        assert!(
            seen_back
                .text
                .contains("Moss grows on the damp bank to the west."),
            "{}",
            seen_back.text
        );
        assert!(seen_back.events.iter().any(|event| event.text == grown));
        assert!(world
            .observe(quiet)
            .unwrap()
            .events
            .iter()
            .all(|event| event.text != grown));
        assert!(!world.observe(quiet).unwrap().text.contains("damp bank"));
        let again = world.step(&[intent("again", near, Attempt::Gather(Material::Moss))]);
        assert_eq!(again.results[0].message, "You gather a handful of moss.");
        let taken = world.observe(near).unwrap();
        assert!(
            taken.text.contains("The damp bank here is bare."),
            "{}",
            taken.text
        );
        assert!(
            taken.text.contains("You have two handfuls of moss."),
            "{}",
            taken.text
        );
        assert!(!taken.text.contains("2 moss"), "{}", taken.text);
        assert!(!taken.text.contains("carried by you"), "{}", taken.text);
        let two = world.examine(near, super::ExamineTarget::Moss);
        assert!(
            two.contains("Two handfuls of moss are carried by you"),
            "{two}"
        );
        let seen_two = world.examine(seen, super::ExamineTarget::Moss);
        assert!(
            seen_two.contains("Two handfuls of moss are carried by Near"),
            "{seen_two}"
        );
        assert_eq!(
            world.examine(quiet, super::ExamineTarget::Moss),
            "You see no moss from here."
        );
        assert!(taken
            .events
            .iter()
            .all(|event| !event.text.contains("gathers")));
        assert!(world
            .observe(seen)
            .unwrap()
            .events
            .iter()
            .any(|event| event.text == "Near gathers a handful of moss."));
        assert!(world
            .observe(quiet)
            .unwrap()
            .events
            .iter()
            .all(|event| !event.text.contains("gathers")));
    }
}

#[cfg(test)]
mod bow {
    use super::{Attempt, ExamineTarget, Material, Role, World};

    fn intent(id: &str, actor: u32, attempt: Attempt) -> super::Intent {
        super::Intent {
            command_id: id.into(),
            actor_id: actor,
            attempt,
        }
    }

    #[test]
    fn a_dry_branch_and_hide_strips_craft_into_a_bow() {
        assert!(matches!(
            crate::parse::parse_attempt("craft bow").unwrap(),
            Attempt::CraftBow
        ));
        assert!(matches!(
            crate::parse::parse_attempt("make bow").unwrap(),
            Attempt::CraftBow
        ));
        assert!(matches!(
            crate::parse::parse_attempt("examine bow").unwrap(),
            Attempt::Examine(ExamineTarget::Bow)
        ));
        let mut world = World::valley(crate::tuning::Tuning::evening(), 1);
        let near = world.spawn_at("Near", "n", 4, 3).unwrap();
        let seen = world.spawn_at("Seen", "s", 3, 3).unwrap();
        let quiet = world.spawn_at("Quiet", "q", 0, 4).unwrap();
        let cell = world.actor(near).unwrap().cell;
        let missing = world.step(&[intent("missing", near, Attempt::CraftBow)]);
        assert_eq!(
            missing.results[0].message,
            "You need a dry branch and hide strips."
        );
        assert!(world
            .observe(seen)
            .unwrap()
            .events
            .iter()
            .all(|event| !event.text.contains("bow")));
        assert!(world
            .observe(quiet)
            .unwrap()
            .events
            .iter()
            .all(|event| !event.text.contains("bow")));
        world.mint_item(Material::Branch, 1000, cell, Some(near), Role::Loose);
        world.mint_item(Material::Cord, 0, cell, Some(near), Role::Loose);
        let damp = world.step(&[intent("damp", near, Attempt::CraftBow)]);
        assert_eq!(damp.results[0].message, "The branch is not dry.");
        assert_eq!(world.loose_ids(near, Material::Cord).len(), 1);
        assert!(world.loose_ids(near, Material::Bow).is_empty());
        assert!(world
            .observe(seen)
            .unwrap()
            .events
            .iter()
            .all(|event| event.text != "The branch is not dry."));
        world.mint_item(Material::Branch, 0, cell, Some(near), Role::Loose);
        let crafted = world.step(&[intent("craft", near, Attempt::CraftBow)]);
        assert_eq!(
            crafted.results[0].message,
            "You craft a bow from a dry branch and hide strips."
        );
        assert_eq!(world.loose_ids(near, Material::Bow).len(), 1);
        assert!(world.loose_ids(near, Material::Cord).is_empty());
        assert_eq!(world.dry_loose_ids(near, Material::Branch).len(), 0);
        assert_eq!(world.loose_ids(near, Material::Branch).len(), 1);
        let arms = world.observe(near).unwrap();
        assert!(arms.text.contains("a bow."), "{}", arms.text);
        assert!(arms
            .events
            .iter()
            .all(|event| !event.text.contains("craft")));
        let yours = world.examine(near, ExamineTarget::Bow);
        assert!(yours.contains("A bow is carried by you"), "{yours}");
        assert!(
            yours.contains("It is a dry branch bound with hide strips."),
            "{yours}"
        );
        let theirs = world.examine(seen, ExamineTarget::Bow);
        assert!(theirs.contains("A bow is carried by Near"), "{theirs}");
        assert_eq!(
            world.examine(quiet, ExamineTarget::Bow),
            "You see no bow from here."
        );
        assert!(world
            .observe(seen)
            .unwrap()
            .events
            .iter()
            .any(|event| event.text == "Near crafts a bow from a dry branch and hide strips."));
        assert!(world
            .observe(quiet)
            .unwrap()
            .events
            .iter()
            .all(|event| !event.text.contains("craft") && !event.text.contains("bow")));
        let dropped = world.step(&[intent("drop", near, Attempt::Drop(Material::Bow))]);
        assert_eq!(
            dropped.results[0].message,
            "You drop a bow. It is in contact with the ground."
        );
        let here = world.observe(near).unwrap().text;
        assert!(
            here.contains("A bow lies here. It is a dry branch bound with hide strips."),
            "{here}"
        );
        assert!(!here.contains("You have a bow."), "{here}");
        let east = world.examine(seen, ExamineTarget::Bow);
        assert!(east.contains("A bow lies to the east."), "{east}");
        assert_eq!(
            world.examine(quiet, ExamineTarget::Bow),
            "You see no bow from here."
        );
    }

    #[test]
    fn a_branch_and_strips_lying_here_craft_into_a_bow() {
        let mut world = World::valley(crate::tuning::Tuning::evening(), 1);
        let near = world.spawn_at("Near", "n", 4, 3).unwrap();
        let seen = world.spawn_at("Seen", "s", 3, 3).unwrap();
        let quiet = world.spawn_at("Quiet", "q", 0, 4).unwrap();
        let cell = world.actor(near).unwrap().cell;
        world.mint_item(Material::Branch, 1000, cell, None, Role::Loose);
        world.mint_item(Material::Cord, 0, cell, None, Role::Loose);
        let damp = world.step(&[intent("damp", near, Attempt::CraftBow)]);
        assert_eq!(damp.results[0].message, "The branch is not dry.");
        assert_eq!(world.ground_ids(cell, Material::Branch).len(), 1);
        assert_eq!(world.ground_ids(cell, Material::Cord).len(), 1);
        assert!(world.loose_ids(near, Material::Bow).is_empty());
        assert!(world
            .observe(seen)
            .unwrap()
            .events
            .iter()
            .all(|event| event.text != "The branch is not dry."));
        let lying = world.mint_item(Material::Branch, 0, cell, None, Role::Loose);
        let from_ground = world.step(&[intent("ground", near, Attempt::CraftBow)]);
        assert_eq!(
            from_ground.results[0].message,
            "You craft a bow from a dry branch and hide strips."
        );
        assert!(world.items.get(&lying).is_none());
        assert_eq!(world.ground_ids(cell, Material::Cord).len(), 0);
        assert_eq!(world.ground_ids(cell, Material::Branch).len(), 1);
        assert_eq!(world.loose_ids(near, Material::Bow).len(), 1);
        assert!(world
            .observe(near)
            .unwrap()
            .events
            .iter()
            .all(|event| !event.text.contains("craft")));
        assert!(world
            .observe(seen)
            .unwrap()
            .events
            .iter()
            .any(|event| event.text == "Near crafts a bow from a dry branch and hide strips."));
        assert!(world
            .observe(quiet)
            .unwrap()
            .events
            .iter()
            .all(|event| !event.text.contains("craft") && !event.text.contains("bow")));
        let carried_branch = world.mint_item(Material::Branch, 0, cell, Some(near), Role::Loose);
        let carried_strips = world.mint_item(Material::Cord, 0, cell, Some(near), Role::Loose);
        let ground_branch = world.mint_item(Material::Branch, 0, cell, None, Role::Loose);
        let ground_strips = world.mint_item(Material::Cord, 0, cell, None, Role::Loose);
        let from_arms = world.step(&[intent("arms", near, Attempt::CraftBow)]);
        assert_eq!(
            from_arms.results[0].message,
            "You craft a bow from a dry branch and hide strips."
        );
        assert!(world.items.get(&carried_branch).is_none());
        assert!(world.items.get(&carried_strips).is_none());
        assert!(world.items.get(&ground_branch).is_some());
        assert!(world.items.get(&ground_strips).is_some());
        assert_eq!(world.loose_ids(near, Material::Bow).len(), 2);
        let again = world.step(&[intent("again", near, Attempt::CraftBow)]);
        assert_eq!(
            again.results[0].message,
            "You craft a bow from a dry branch and hide strips."
        );
        assert!(world.items.get(&ground_branch).is_none());
        assert!(world.items.get(&ground_strips).is_none());
        assert_eq!(world.ground_ids(cell, Material::Branch).len(), 1);
        assert_eq!(world.loose_ids(near, Material::Bow).len(), 3);
    }
}

#[cfg(test)]
mod bow_hunt {
    use super::{Attempt, Material, Role, World};

    fn intent(id: &str, actor: u32, attempt: Attempt) -> super::Intent {
        super::Intent {
            command_id: id.into(),
            actor_id: actor,
            attempt,
        }
    }

    fn place_deer(world: &mut World, x: i32, y: i32) {
        let cell = super::cell_id(x, y).unwrap();
        let deer = world.deer.as_mut().unwrap();
        deer.cell = cell;
    }

    fn arm(world: &mut World, actor: u32, material: Material) -> super::ItemId {
        let cell = world.actor(actor).unwrap().cell;
        world.mint_item(material, 0, cell, Some(actor), Role::Loose)
    }

    #[test]
    fn a_bow_and_a_flint_flake_bring_the_deer_down_at_sight() {
        assert!(matches!(
            crate::parse::parse_attempt("hunt").unwrap(),
            Attempt::Hunt
        ));
        let mut world = World::valley(crate::tuning::Tuning::evening(), 1);
        let near = world.spawn_at("Near", "n", 4, 1).unwrap();
        let seen = world.spawn_at("Seen", "s", 5, 1).unwrap();
        let quiet = world.spawn_at("Quiet", "q", 0, 4).unwrap();
        let bare = world.step(&[intent("bare", near, Attempt::Hunt)]);
        assert_eq!(bare.results[0].message, "You are not carrying a spear.");
        assert!(world
            .observe(seen)
            .unwrap()
            .events
            .iter()
            .all(|event| { event.text != "You are not carrying a spear." }));
        assert!(world
            .observe(quiet)
            .unwrap()
            .events
            .iter()
            .all(|event| { event.text != "You are not carrying a spear." }));
        arm(&mut world, near, Material::Bow);
        let flake_less = world.step(&[intent("flake", near, Attempt::Hunt)]);
        assert_eq!(
            flake_less.results[0].message,
            "You need a sharp flint flake."
        );
        assert!(world
            .observe(seen)
            .unwrap()
            .events
            .iter()
            .all(|event| { event.text != "You need a sharp flint flake." }));
        arm(&mut world, near, Material::Flake);
        place_deer(&mut world, 0, 0);
        let far = world.step(&[intent("far", near, Attempt::Hunt)]);
        assert_eq!(
            far.results[0].message,
            "The deer is not close enough for the bow."
        );
        assert_eq!(world.loose_ids(near, Material::Flake).len(), 1);
        assert_eq!(world.loose_ids(near, Material::Bow).len(), 1);
        assert!(world
            .observe(seen)
            .unwrap()
            .events
            .iter()
            .all(|event| { event.text != "The deer is not close enough for the bow." }));
        place_deer(&mut world, 6, 1);
        assert!(world
            .observe(near)
            .unwrap()
            .allowed_actions
            .iter()
            .any(|action| action == "hunt"));
        let before = world.observe(near).unwrap().text;
        assert!(before.contains("a sharp flint flake"), "{before}");
        let shot = world.step(&[intent("shot", near, Attempt::Hunt)]);
        assert_eq!(
            shot.results[0].message,
            "You bring the deer down with the bow. The flint flake is spent."
        );
        assert!(world.deer_cell().is_none());
        let deer_cell = super::cell_id(6, 1).unwrap();
        assert!(world.items.values().any(|item| {
            item.material == Material::Carcass && item.cell == deer_cell && item.carrier.is_none()
        }));
        assert!(world.loose_ids(near, Material::Flake).is_empty());
        assert_eq!(world.loose_ids(near, Material::Bow).len(), 1);
        let near_view = world.observe(near).unwrap();
        assert!(near_view.text.contains("a bow"), "{}", near_view.text);
        assert!(
            !near_view.text.contains("flint flake"),
            "{}",
            near_view.text
        );
        assert!(near_view
            .events
            .iter()
            .all(|event| !event.text.contains("with the bow")));
        assert!(world.observe(seen).unwrap().events.iter().any(|event| {
            event.text == "Near brings the deer down with the bow. The flint flake is spent."
        }));
        assert!(world.observe(quiet).unwrap().events.iter().all(|event| {
            !event.text.contains("with the bow") && !event.text.contains("brings the deer")
        }));
    }

    #[test]
    fn a_hardened_spear_still_hunts_when_a_bow_is_also_carried() {
        let mut world = World::valley(crate::tuning::Tuning::evening(), 1);
        let near = world.spawn_at("Near", "n", 4, 1).unwrap();
        place_deer(&mut world, 6, 1);
        let spear = arm(&mut world, near, Material::Spear);
        {
            let point = world.items.get_mut(&spear).unwrap();
            point.hardened = true;
            point.flint_tip = true;
        }
        let short = world.step(&[intent("short", near, Attempt::Hunt)]);
        assert_eq!(
            short.results[0].message,
            "The deer is not close enough to hunt."
        );
        arm(&mut world, near, Material::Bow);
        arm(&mut world, near, Material::Flake);
        place_deer(&mut world, 6, 1);
        let past = world.step(&[intent("past", near, Attempt::Hunt)]);
        assert_eq!(
            past.results[0].message,
            "You bring the deer down with the bow. The flint flake is spent."
        );
        assert_eq!(world.loose_ids(near, Material::Spear).len(), 1);
        assert!(world.loose_ids(near, Material::Flake).is_empty());
        assert_eq!(world.loose_ids(near, Material::Bow).len(), 1);
        assert!(world
            .observe(near)
            .unwrap()
            .events
            .iter()
            .all(|event| !event.text.contains("with the bow")));

        let mut world = World::valley(crate::tuning::Tuning::evening(), 1);
        let near = world.spawn_at("Near", "n", 4, 1).unwrap();
        place_deer(&mut world, 4, 1);
        let spear = arm(&mut world, near, Material::Spear);
        {
            let point = world.items.get_mut(&spear).unwrap();
            point.hardened = true;
            point.flint_tip = true;
        }
        arm(&mut world, near, Material::Bow);
        arm(&mut world, near, Material::Flake);
        let close = world.step(&[intent("close", near, Attempt::Hunt)]);
        assert_eq!(
            close.results[0].message,
            "You bring the deer down with the flint spear."
        );
        assert!(world
            .observe(near)
            .unwrap()
            .events
            .iter()
            .any(|event| { event.text == "You bring the deer down with the flint spear." }));
        assert_eq!(world.loose_ids(near, Material::Bow).len(), 1);
        assert_eq!(world.loose_ids(near, Material::Flake).len(), 1);
        assert!(world.deer_cell().is_none());
    }
}

#[cfg(test)]
mod deer_return {
    use super::{Attempt, ExamineTarget, Material, Role, World};

    fn intent(id: &str, actor: u32, attempt: Attempt) -> super::Intent {
        super::Intent {
            command_id: id.into(),
            actor_id: actor,
            attempt,
        }
    }

    #[test]
    fn a_deer_stands_on_the_trail_again_and_can_be_hunted() {
        let mut world = World::valley(crate::tuning::Tuning::evening(), 1);
        let near = world.spawn_at("Near", "n", 2, 3).unwrap();
        let seen = world.spawn_at("Seen", "s", 3, 3).unwrap();
        let quiet = world.spawn_at("Quiet", "q", 5, 4).unwrap();
        let spear = world.mint_item(
            Material::Spear,
            0,
            world.actor(near).unwrap().cell,
            Some(near),
            Role::Loose,
        );
        {
            let point = world.items.get_mut(&spear).unwrap();
            point.hardened = true;
            point.flint_tip = true;
        }
        let first = world.step(&[intent("first", near, Attempt::Hunt)]);
        assert_eq!(
            first.results[0].message,
            "You bring the deer down with the flint spear."
        );
        assert!(world.deer_cell().is_none());
        assert!(world.deer_returns_at.is_some());
        let down = world.observe(near).unwrap().text;
        assert!(down.contains("A deer carcass lies here."), "{down}");
        assert!(!down.contains("A deer stands"), "{down}");
        assert_eq!(
            world.examine(near, ExamineTarget::Deer),
            "You cannot see a deer from here."
        );
        let ticks = world.tuning.deer_ticks;
        for _ in 0..ticks - 1 {
            world.step(&[]);
        }
        assert!(world.deer_cell().is_none());
        assert!(!world.observe(near).unwrap().text.contains("A deer stands"));
        world.step(&[]);
        assert_eq!(world.deer_cell(), Some((2, 3)));
        let back = world.observe(near).unwrap();
        assert!(
            back.text.contains("A deer stands on that trail."),
            "{}",
            back.text
        );
        let arrived = "A deer stands in the valley again.";
        assert!(back.events.iter().any(|event| event.text == arrived));
        let seen_back = world.observe(seen).unwrap();
        assert!(seen_back.events.iter().any(|event| event.text == arrived));
        let seen_deer = world.examine(seen, ExamineTarget::Deer);
        assert!(seen_deer.contains("A deer stands"), "{seen_deer}");
        assert!(world
            .observe(quiet)
            .unwrap()
            .events
            .iter()
            .all(|event| event.text != arrived));
        assert_eq!(
            world.examine(quiet, ExamineTarget::Deer),
            "You cannot see a deer from here."
        );
        assert!(world.deer_returns_at.is_none());
        let second = world.step(&[intent("second", near, Attempt::Hunt)]);
        assert_eq!(
            second.results[0].message,
            "You bring the deer down with the flint spear."
        );
        assert!(world.deer_cell().is_none());
        assert!(world.deer_returns_at.is_some());
        assert!(world
            .observe(near)
            .unwrap()
            .events
            .iter()
            .any(|event| { event.text == "You bring the deer down with the flint spear." }));
        assert!(world
            .observe(seen)
            .unwrap()
            .events
            .iter()
            .any(|event| { event.text == "Near brings the deer down with the flint spear." }));
        assert!(world
            .observe(quiet)
            .unwrap()
            .events
            .iter()
            .all(|event| { !event.text.contains("brings the deer") && event.text != arrived }));
    }
}

#[cfg(test)]
mod flint_return {
    use super::{Attempt, ExamineTarget, Material, World};

    fn intent(id: &str, actor: u32, attempt: Attempt) -> super::Intent {
        super::Intent {
            command_id: id.into(),
            actor_id: actor,
            attempt,
        }
    }

    #[test]
    fn the_slope_has_no_flint_until_a_nodule_lies_there_again() {
        let mut world = World::valley(crate::tuning::Tuning::evening(), 1);
        let near = world.spawn_at("Near", "n", 4, 0).unwrap();
        let seen = world.spawn_at("Seen", "s", 4, 1).unwrap();
        let quiet = world.spawn_at("Quiet", "q", 6, 3).unwrap();
        let here = world.observe(near).unwrap().text;
        assert!(here.contains("A flint nodule lies here."), "{here}");
        assert!(world
            .observe(seen)
            .unwrap()
            .text
            .contains("A flint nodule lies on the slope to the north."));
        assert!(!world.observe(quiet).unwrap().text.contains("flint"));
        let gathered = world.step(&[intent("gather", near, Attempt::Gather(Material::Flint))]);
        assert_eq!(gathered.results[0].message, "You gather a flint nodule.");
        let knapped = world.step(&[intent("knap", near, Attempt::Knap)]);
        assert_eq!(
            knapped.results[0].message,
            "You knap the flint nodule into a sharp flake."
        );
        let empty = world.observe(near).unwrap().text;
        assert!(empty.contains("The slope has no flint."), "{empty}");
        assert!(!empty.contains("A flint nodule lies here."), "{empty}");
        assert_eq!(
            world.examine(near, ExamineTarget::Flint),
            "The slope here has no flint.\nA sharp flint flake is carried by you, in the open. It is sharp enough to cut or haft later."
        );
        let south = world.observe(seen).unwrap().text;
        assert!(
            south.contains("The slope to the north has no flint."),
            "{south}"
        );
        assert!(!south.contains("A flint nodule lies"), "{south}");
        assert!(!world.observe(quiet).unwrap().text.contains("flint"));
        let ticks = world.tuning.flint_ticks;
        for _ in 0..ticks - 2 {
            world.step(&[]);
        }
        assert!(world
            .observe(near)
            .unwrap()
            .text
            .contains("The slope has no flint."));
        world.step(&[]);
        let back = "A flint nodule lies on the slope again.";
        let again = world.observe(near).unwrap();
        assert!(
            again.text.contains("A flint nodule lies here."),
            "{}",
            again.text
        );
        assert!(
            !again.text.contains("The slope has no flint."),
            "{}",
            again.text
        );
        assert!(again.events.iter().any(|event| event.text == back));
        let seen_back = world.observe(seen).unwrap();
        assert!(
            seen_back
                .text
                .contains("A flint nodule lies on the slope to the north."),
            "{}",
            seen_back.text
        );
        assert!(seen_back.events.iter().any(|event| event.text == back));
        assert!(world
            .observe(quiet)
            .unwrap()
            .events
            .iter()
            .all(|event| event.text != back));
        assert!(!world.observe(quiet).unwrap().text.contains("flint"));
        let second = world.step(&[intent("second", near, Attempt::Gather(Material::Flint))]);
        assert_eq!(second.results[0].message, "You gather a flint nodule.");
        let taken = world.observe(near).unwrap().text;
        assert!(taken.contains("The slope has no flint."), "{taken}");
        assert!(!taken.contains("A flint nodule lies here."), "{taken}");
        assert!(world
            .flint_returns_at
            .contains_key(&world.actor(near).unwrap().cell));
    }
}

#[cfg(test)]
mod flake_arms {
    use super::{ExamineTarget, Material, World};

    #[test]
    fn two_sharp_flint_flakes_stay_flakes() {
        let mut world = World::valley(crate::tuning::Tuning::evening(), 1);
        let near = world.spawn_at("Near", "n", 4, 3).unwrap();
        let seen = world.spawn_at("Seen", "s", 5, 3).unwrap();
        let quiet = world.spawn_at("Quiet", "q", 0, 3).unwrap();
        let cell = world.actor(near).unwrap().cell;
        world.mint_item(Material::Flake, 0, cell, Some(near), super::Role::Loose);
        let one = world.observe(near).unwrap().text;
        assert!(one.contains("You have a sharp flint flake."), "{one}");
        assert!(
            one.contains("It is sharp enough to cut or haft later."),
            "{one}"
        );
        assert!(!one.contains("2 flake"), "{one}");
        assert!(!one.contains("two sharp"), "{one}");
        assert_eq!(
            world.examine(near, ExamineTarget::Flake),
            "A sharp flint flake is carried by you, in the open. It is sharp enough to cut or haft later."
        );
        assert_eq!(
            world.examine(seen, ExamineTarget::Flake),
            "A sharp flint flake is carried by Near, in the open. It is sharp enough to cut or haft later."
        );
        assert_eq!(
            world.examine(quiet, ExamineTarget::Flake),
            "You see no flint flake from here."
        );
        world.mint_item(Material::Flake, 0, cell, Some(near), super::Role::Loose);
        let two = world.observe(near).unwrap().text;
        assert!(two.contains("You have two sharp flint flakes."), "{two}");
        assert!(
            two.contains("They are sharp enough to cut or haft later."),
            "{two}"
        );
        assert!(!two.contains("2 flake"), "{two}");
        assert!(!two.contains("carried by you (dry)"), "{two}");
        assert!(!two.contains("It is sharp enough"), "{two}");
        assert_eq!(
            world.examine(near, ExamineTarget::Flake),
            "Two sharp flint flakes are carried by you, in the open. They are sharp enough to cut or haft later."
        );
        assert_eq!(
            world.examine(seen, ExamineTarget::Flake),
            "Two sharp flint flakes are carried by Near, in the open. They are sharp enough to cut or haft later."
        );
        assert_eq!(
            world.examine(quiet, ExamineTarget::Flake),
            "You see no flint flake from here."
        );
        world.raining = true;
        let rain = world.observe(near).unwrap().text;
        assert!(rain.contains("You have two sharp flint flakes."), "{rain}");
        assert!(!rain.contains("(dry)"), "{rain}");
        assert_eq!(
            world.examine(near, ExamineTarget::Flake),
            "Two sharp flint flakes are carried by you, in the rain. They are sharp enough to cut or haft later."
        );
        assert_eq!(
            world.examine(seen, ExamineTarget::Flake),
            "Two sharp flint flakes are carried by Near, in the rain. They are sharp enough to cut or haft later."
        );
    }
}

#[cfg(test)]
mod dry_clothes {
    use super::{Attempt, Dir, ExamineTarget, World};

    fn intent(id: &str, actor: u32, attempt: Attempt) -> super::Intent {
        super::Intent {
            command_id: id.into(),
            actor_id: actor,
            attempt,
        }
    }

    #[test]
    fn wet_clothes_dry_beside_the_fire() {
        assert!(matches!(
            crate::parse::parse_attempt("dry clothes").unwrap(),
            Attempt::DryClothes
        ));
        assert!(matches!(
            crate::parse::parse_attempt("wait by the fire").unwrap(),
            Attempt::DryClothes
        ));
        let mut world = World::valley(crate::tuning::Tuning::evening(), 1);
        let near = world.spawn_at("Near", "n", 6, 1).unwrap();
        let seen = world.spawn_at("Seen", "s", 5, 1).unwrap();
        let quiet = world.spawn_at("Quiet", "q", 5, 4).unwrap();
        world.actors.get_mut(&near).unwrap().wetness = 600;
        let none = world.step(&[intent("none", near, Attempt::DryClothes)]);
        assert_eq!(none.results[0].message, "There is no fire burning here.");
        assert!(world
            .observe(seen)
            .unwrap()
            .events
            .iter()
            .all(|event| event.text != "There is no fire burning here."));
        assert!(world
            .observe(quiet)
            .unwrap()
            .events
            .iter()
            .all(|event| event.text != "There is no fire burning here."));
        let cell = world.actor(near).unwrap().cell;
        world.mint_fire(cell, 0);
        let start = world.step(&[intent("start", near, Attempt::DryClothes)]);
        assert_eq!(
            start.results[0].message,
            "You hold your soaked clothes by the fire. They are not dry yet."
        );
        let look = world.observe(near).unwrap();
        assert!(
            look.text.contains("Your clothes are drying by the fire."),
            "{}",
            look.text
        );
        assert!(look
            .events
            .iter()
            .all(|event| !event.text.contains("clothes")));
        let seen_body = world.examine(seen, ExamineTarget::Person("Near".into()));
        assert!(
            seen_body.contains("Their clothes are drying by the fire."),
            "{seen_body}"
        );
        assert!(world.observe(seen).unwrap().events.iter().any(|event| {
            event.text == "Near holds their soaked clothes by the fire. They are not dry yet."
        }));
        assert_eq!(
            world.examine(quiet, ExamineTarget::Person("Near".into())),
            "You cannot see Near from here."
        );
        assert!(world
            .observe(quiet)
            .unwrap()
            .events
            .iter()
            .all(|event| !event.text.contains("clothes")));
        let again = world.step(&[intent("again", near, Attempt::DryClothes)]);
        assert_eq!(
            again.results[0].message,
            "Your clothes are already drying by the fire."
        );
        assert!(world
            .observe(seen)
            .unwrap()
            .events
            .iter()
            .all(|event| { event.text != "Your clothes are already drying by the fire." }));
        let ticks = world.tuning.clothes_ticks;
        for _ in 0..ticks - 2 {
            world.step(&[]);
        }
        assert!(world
            .observe(near)
            .unwrap()
            .text
            .contains("Your clothes are drying by the fire."));
        assert!(world.actor(near).unwrap().wetness > 0);
        world.step(&[]);
        assert_eq!(world.actor(near).unwrap().wetness, 0);
        assert!(world.actor(near).unwrap().clothes_dry_at.is_none());
        let done = world.observe(near).unwrap();
        assert!(!done.text.contains("drying by the fire"), "{}", done.text);
        assert!(done.text.contains("Your clothes are dry."), "{}", done.text);
        assert!(done
            .events
            .iter()
            .any(|event| event.text == "Your clothes dry at the fire."));
        let seen_done = world.examine(seen, ExamineTarget::Person("Near".into()));
        assert!(seen_done.contains("Their clothes are dry."), "{seen_done}");
        assert!(world
            .observe(seen)
            .unwrap()
            .events
            .iter()
            .any(|event| event.text == "Near's clothes dry at the fire."));
        assert!(world
            .observe(quiet)
            .unwrap()
            .events
            .iter()
            .all(|event| event.text != "Near's clothes dry at the fire."));
        let repeat = world.step(&[intent("repeat", near, Attempt::DryClothes)]);
        assert_eq!(repeat.results[0].message, "Your clothes are already dry.");
        assert!(world
            .observe(seen)
            .unwrap()
            .events
            .iter()
            .all(|event| event.text != "Your clothes are already dry."));
        world.actors.get_mut(&near).unwrap().wetness = 250;
        let damp = world.step(&[intent("damp", near, Attempt::DryClothes)]);
        assert_eq!(
            damp.results[0].message,
            "You hold your damp clothes by the fire. They are not dry yet."
        );
        world.step(&[intent("leave", near, Attempt::Move(Dir::North))]);
        let left = world.observe(near).unwrap().text;
        assert!(!left.contains("drying by the fire"), "{left}");
        assert!(left.contains("Your clothes are damp."), "{left}");
        let already = world
            .observe(near)
            .unwrap()
            .events
            .iter()
            .filter(|event| event.text == "Your clothes dry at the fire.")
            .count();
        let ticks = world.tuning.clothes_ticks;
        for _ in 0..ticks {
            world.step(&[]);
        }
        assert_eq!(world.actor(near).unwrap().wetness, 250);
        assert!(world.actor(near).unwrap().clothes_dry_at.is_none());
        let still = world
            .observe(near)
            .unwrap()
            .events
            .iter()
            .filter(|event| event.text == "Your clothes dry at the fire.")
            .count();
        assert_eq!(still, already);
    }
}

#[cfg(test)]
mod wolf {
    use super::World;

    fn howl_times(world: &World) -> Vec<u64> {
        world
            .events
            .iter()
            .filter(|event| event.text == "A wolf howls.")
            .map(|event| event.time)
            .collect()
    }

    #[test]
    fn a_wolf_howls_once_when_night_falls() {
        let mut world = World::valley(crate::tuning::Tuning::evening(), 1);
        let near = world.spawn_at("Near", "n", 1, 1).unwrap();
        let seen = world.spawn_at("Seen", "s", 6, 1).unwrap();
        let quiet = world.spawn_at("Quiet", "q", 0, 4).unwrap();
        let night = world.tuning.night_at;
        while world.time + 1 < night {
            world.step(&[]);
        }
        assert_eq!(world.time, night - 1);
        assert!(!world.night());
        assert!(howl_times(&world).is_empty());
        let day = world.observe(near).unwrap().text;
        assert!(!day.contains("Night has fallen."), "{day}");
        world.step(&[]);
        assert_eq!(world.time, night);
        assert!(world.night());
        assert_eq!(howl_times(&world), vec![night]);
        let fallen = world.observe(near).unwrap().text;
        assert!(fallen.contains("Night has fallen."), "{fallen}");
        let near_line = world
            .observe(near)
            .unwrap()
            .events
            .iter()
            .find(|event| event.text.contains("howl"))
            .map(|event| event.text.clone())
            .unwrap();
        let seen_line = world
            .observe(seen)
            .unwrap()
            .events
            .iter()
            .find(|event| event.text.contains("howl"))
            .map(|event| event.text.clone())
            .unwrap();
        let quiet_line = world
            .observe(quiet)
            .unwrap()
            .events
            .iter()
            .find(|event| event.text.contains("howl"))
            .map(|event| event.text.clone())
            .unwrap();
        assert_eq!(near_line, "You hear a wolf howl.");
        assert_eq!(seen_line, "You hear a wolf howl to the north and west.");
        assert_eq!(quiet_line, "You hear a wolf howl to the north.");
        assert!(!world.observe(near).unwrap().text.contains("wolf"));
        let dawn = night + world.tuning.night_len;
        while world.time < dawn {
            world.step(&[]);
        }
        assert_eq!(howl_times(&world), vec![night]);
        let next = dawn + world.tuning.day_len;
        while world.time + 1 < next {
            world.step(&[]);
        }
        assert_eq!(world.time, next - 1);
        assert!(!world.night());
        assert_eq!(howl_times(&world), vec![night]);
        let morning = world.observe(near).unwrap().text;
        assert!(!morning.contains("Night has fallen."), "{morning}");
        world.step(&[]);
        assert_eq!(world.time, next);
        assert!(world.night());
        assert_eq!(howl_times(&world), vec![night, next]);
        let again = world.observe(near).unwrap().text;
        assert!(again.contains("Night has fallen."), "{again}");
    }

    #[test]
    fn wolf_tracks_show_for_who_can_see_the_ground_and_then_fade() {
        let mut world = World::valley(crate::tuning::Tuning::evening(), 1);
        let here = world.spawn_at("Near", "n", 0, 0).unwrap();
        let seen = world.spawn_at("Seen", "s", 0, 1).unwrap();
        let quiet = world.spawn_at("Quiet", "q", 6, 1).unwrap();
        let night = world.tuning.night_at;
        while world.time + 1 < night {
            world.step(&[]);
        }
        for id in [here, seen, quiet] {
            assert!(
                !world.observe(id).unwrap().text.contains("Wolf tracks"),
                "{}",
                world.observe(id).unwrap().text
            );
        }
        world.step(&[]);
        assert_eq!(world.time, night);
        let on_it = world.observe(here).unwrap().text;
        assert!(
            on_it.contains("Night has fallen.")
                && on_it.contains("Wolf tracks mark the ground here."),
            "{on_it}"
        );
        assert!(!on_it.contains("A wolf stands"), "{on_it}");
        let dark = world.observe(seen).unwrap().text;
        assert!(!dark.contains("Wolf tracks"), "{dark}");
        assert!(
            dark.contains("The dark hides what lies farther off."),
            "{dark}"
        );
        let out = world.observe(quiet).unwrap().text;
        assert!(!out.contains("Wolf tracks"), "{out}");
        let deer_cell = super::cell_id(1, 1).unwrap();
        world.leave_fresh_tracks(deer_cell);
        let dark_deer = world.observe(seen).unwrap().text;
        assert!(
            !dark_deer.contains("Deer tracks mark the ground to the east."),
            "{dark_deer}"
        );
        let seen_cell = world.actor(seen).unwrap().cell;
        let fire = world.mint_fire(seen_cell, 0);
        let lit = world.observe(seen).unwrap().text;
        assert!(
            lit.contains("Wolf tracks mark the ground to the north."),
            "{lit}"
        );
        assert!(
            lit.contains("Deer tracks mark the ground to the east."),
            "{lit}"
        );
        assert!(
            !lit.contains("The dark hides what lies farther off."),
            "{lit}"
        );
        world.extinguish(fire, "The fire goes out.", None);
        let dark_again = world.observe(seen).unwrap().text;
        assert!(!dark_again.contains("Wolf tracks"), "{dark_again}");
        assert!(
            !dark_again.contains("Deer tracks mark the ground to the east."),
            "{dark_again}"
        );
        let ticks = world.tuning.track_ticks;
        for _ in 0..ticks - 1 {
            world.step(&[]);
            assert!(world
                .observe(here)
                .unwrap()
                .text
                .contains("Wolf tracks mark the ground here."));
            assert!(!world.observe(seen).unwrap().text.contains("Wolf tracks"));
        }
        world.step(&[]);
        assert_eq!(world.time, night + ticks);
        for id in [here, seen, quiet] {
            assert!(
                !world.observe(id).unwrap().text.contains("Wolf tracks"),
                "{}",
                world.observe(id).unwrap().text
            );
        }
    }
}
