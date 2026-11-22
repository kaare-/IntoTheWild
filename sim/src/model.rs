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
    Net,
    Mat,
    Canoe,
    /// A handful that does not feed until it is cooked. Not the sweet
    /// berries under the rock, and not the dark berries.
    PaleBerry,
    /// Carved from a dry branch. Bare hands can still paddle; holding this names it.
    Paddle,
    /// Carved from a bone. With hide strips, it takes a fish from the brook.
    Hook,
    /// Pitch from the pine on the dry floor south of the trail.
    Resin,
    /// Withies from the willow on the dry bank south of the brook.
    /// A bundle is cordage. A bone fish hook spends it as the line
    /// when the arms hold no hide strips.
    Willow,
    /// Fibres from the nettles east of the trail.
    /// A bundle is cordage, spent where hide strips or willow cordage are spent
    /// when neither of those is at hand. Bare hands sting. Mud on the skin does not.
    Nettle,
    /// Roots from the cattails in the shallows of the still.
    /// They are not the reeds at the brook. Raw roots do not feed until cooked.
    Cattail,
    /// A sheet of bark folded into a cup. It holds a drink the way a clay pot does.
    /// It is not a pot, and it is not the bark basket.
    BarkCup,
    /// Cattail roots pounded with a stone on a stone. Raw flour does not feed.
    /// Cooked, it is a flatbread.
    Flour,
    /// A handful from the wet bank. Steeped in hot water over a fire, it is mint tea.
    Mint,
    /// A handful from the pine south of the trail. Steeped in hot water, it eases a chill.
    /// It is not mint, and it does not settle a stomach.
    Needle,
    /// A bow drill. Two dry branches and cordage. It lights a fire without tinder.
    Drill,
    /// Bark from the willow, not the withies. Steeped, it eases a sunburn a little.
    /// It is not cordage, and it does not settle a stomach or a chill.
    WillowBark,
    /// Seed-head down from the cattails. It is not the roots.
    /// A handful in the arms is tinder, and it catches when bark would not.
    Fluff,
    /// One worm from damp earth. On a bone fish hook it is spent,
    /// and that stretch of the brook is quiet for one tick.
    Worm,
    /// A clear stone on the rock face. It can spark with flint.
    /// It is not the loose stone, and it is not the flint nodule.
    Quartz,
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
            Material::Net => "net",
            Material::Mat => "reed mat",
            Material::Canoe => "dugout canoe",
            Material::PaleBerry => "pale berry",
            Material::Paddle => "paddle",
            Material::Hook => "fish hook",
            Material::Resin => "resin",
            Material::Willow => "willow",
            Material::Nettle => "nettle",
            Material::Cattail => "cattail",
            Material::BarkCup => "bark cup",
            Material::Flour => "flour",
            Material::Mint => "mint",
            Material::Needle => "pine needles",
            Material::Drill => "bow drill",
            Material::WillowBark => "willow bark",
            Material::Fluff => "cattail fluff",
            Material::Worm => "worm",
            Material::Quartz => "quartz",
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
            Material::BitterBerry => "handful of dark berries",
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
            Material::Net => "fishing net",
            Material::Mat => "reed mat",
            Material::Canoe => "dugout canoe",
            Material::PaleBerry => "handful of pale berries",
            Material::Paddle => "wooden paddle",
            Material::Hook => "bone fish hook",
            Material::Resin => "lump of pine resin",
            Material::Willow => "bundle of willow cordage",
            Material::Nettle => "bundle of nettle cordage",
            Material::Cattail => "cattail roots",
            Material::BarkCup => "bark cup",
            Material::Flour => "cattail flour",
            Material::Mint => "handful of mint",
            Material::Needle => "handful of pine needles",
            Material::Drill => "bow drill",
            Material::WillowBark => "strip of willow bark",
            Material::Fluff => "handful of cattail fluff",
            Material::Worm => "worm",
            Material::Quartz => "clear stone of quartz",
        }
    }

    pub fn cooked_phrase(self) -> &'static str {
        match self {
            Material::Mushroom => "a mushroom",
            Material::BitterBerry => "a handful of dark berries",
            Material::Berry => "a handful of berries",
            Material::PaleBerry => "a handful of pale berries",
            Material::Paddle => "a wooden paddle",
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
    Hare,
    Tracks,
    Rock,
    Slope,
    Person(String),
    Basket,
    Charcoal,
    Reeds,
    Snare,
    Weir,
    Bone,
    Salt,
    Honey,
    Eggs,
    Moss,
    Bow,
    Net,
    Mat,
    Canoe,
    PaleBerries,
    Paddle,
    Hook,
    Resin,
    Willow,
    Nettle,
    Cattail,
    BarkCup,
    Mint,
    Needle,
    Drill,
    WillowBark,
    Fluff,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Attempt {
    Look,
    /// The arms line from look, and nothing else.
    Arms,
    /// Bare `snare`. Names the verb. Does not set one.
    NameSnare,
    /// Bare `canoe`. Names the verbs. Does not board, carve, or paddle.
    NameCanoe,
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
    /// A dry branch pointed into a wooden spear. No flint and no stone.
    PointSpear,
    /// Hold that wooden point in a burning fire until it hardens.
    HardenSpear,
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
    /// Dry branches on a burning fire, covered with a handful of mud.
    Bank,
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
    /// A bone fish hook and hide strips. The strips are spent as the line.
    FishHook,
    TakeBrand,
    WeaveBasket,
    /// A sheet of bark folded into a cup that holds water. Not a clay pot.
    FoldCup,
    PutFish,
    Thatch,
    SetSnare,
    SetNet,
    /// Reeds and cordage in a brook cell. Spent when the weir is set.
    /// After a wait a raw fish lies there. The weir is gone. The brook is not fished out.
    SetWeir,
    CarveAwl,
    SaltMeat,
    SaltFish,
    SmokeFish,
    SmokeMeat,
    SmokeNest,
    DressSting,
    CraftBow,
    DryClothes,
    WeaveNet,
    WeaveMat,
    SetMat,
    HollowLog,
    CarveCanoe,
    Launch,
    Board,
    Paddle(Option<Dir>),
    Disembark,
    Beach,
    CarvePaddle,
    CarveHook,
    /// Spend a lump of pine resin on the dugout here, or the one afloat beside you.
    SealCanoe,
    /// A lump of pine resin spent on a bark cup. Not a dugout.
    SealCup,
    /// Cattail roots pounded with a stone on a stone. The stones stay.
    Grind,
    /// Steep a handful of mint in a pot or a bark cup of water over a fire.
    /// The tea is drunk in the same act. A sick stomach eases.
    Brew,
    /// Steep a handful of pine needles the same way. A chill eases slightly.
    /// A sick stomach does not.
    SteepPine,
    /// Two dry branches and cordage become a bow drill. The drill is not a hunting bow.
    CarveDrill,
    /// The bow drill is the tinder and the fuel. Bark, resin, and charcoal stay where they are.
    DrillFire,
    /// Steep a strip of willow bark in a pot or a bark cup of water over a fire.
    /// The tea is drunk in the same act. A sunburn eases slightly.
    /// A sick stomach and a chill do not.
    SteepWillow,
    /// A column of smoke from a burning fire. Green reeds or pine needles
    /// may be laid on it. They do not change what the column means.
    Signal(Option<Material>),
    /// Flint struck on quartz. Sparks jump. It does not light a fire.
    Strike,
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
                | Attempt::Brew
                | Attempt::SteepPine
                | Attempt::SteepWillow
                | Attempt::SetClay
                | Attempt::BindSpear
                | Attempt::Knap
                | Attempt::Haft
                | Attempt::PointSpear
                | Attempt::HardenSpear
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
                | Attempt::FishHook
                | Attempt::TakeBrand
                | Attempt::WeaveBasket
                | Attempt::FoldCup
                | Attempt::PutFish
                | Attempt::Thatch
                | Attempt::SetSnare
                | Attempt::SetNet
                | Attempt::SetWeir
                | Attempt::CarveAwl
                | Attempt::SaltMeat
                | Attempt::SaltFish
                | Attempt::SmokeFish
                | Attempt::SmokeMeat
                | Attempt::SmokeNest
                | Attempt::DressSting
                | Attempt::CraftBow
                | Attempt::DryClothes
                | Attempt::WeaveNet
                | Attempt::WeaveMat
                | Attempt::SetMat
                | Attempt::HollowLog
                | Attempt::CarveCanoe
                | Attempt::Launch
                | Attempt::Board
                | Attempt::Paddle(_)
                | Attempt::Disembark
                | Attempt::Beach
                | Attempt::CarvePaddle
                | Attempt::CarveHook
                | Attempt::SealCanoe
                | Attempt::SealCup
                | Attempt::Grind
                | Attempt::CarveDrill
                | Attempt::DrillFire
                | Attempt::Signal(_)
                | Attempt::Strike
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
    /// The sim time when stings leave the body. None means the skin is clear.
    /// Mud and ochre do not change it. Nettles and bees share the timer.
    /// `nettle_sting` says which one look names.
    #[serde(default)]
    pub stings_until: Option<u64>,
    /// The sting on `stings_until` came from nettles. Bee stings leave this false.
    #[serde(default)]
    pub nettle_sting: bool,
    /// When wet clothes at a fire become dry. None means they are not drying.
    /// Stepping off the fire clears it. The clothes stay as wet as they were.
    #[serde(default)]
    pub clothes_dry_at: Option<u64>,
    /// This person has eaten the bitter berries. Look may then say so.
    /// Someone who has not tasted them does not get that sentence.
    #[serde(default)]
    pub tasted_bitter: bool,
    /// This person has tried to eat a raw mushroom. Look may then say it
    /// needs heat. Someone who has not tried does not get that sentence.
    #[serde(default)]
    pub raw_mushroom_known: bool,
    /// This person has eaten pale berries raw. Look may then say they must
    /// be cooked. Someone who has not eaten them does not get that sentence.
    #[serde(default)]
    pub pale_berry_known: bool,
    /// This person has tried to eat bark. Look may then say it does not
    /// feed them. Someone who has not tried does not get that sentence.
    #[serde(default)]
    pub bark_known: bool,
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
    /// A dry branch pointed into a spear. No stone and no flint.
    #[serde(default)]
    pub wooden_tip: bool,
    /// Hide strips or willow cordage lashing a flint tip to the haft. A stone spear leaves this false.
    #[serde(default)]
    pub bound: bool,
    /// The lashing was willow cordage. Hide strips leave this false.
    #[serde(default)]
    pub bound_willow: bool,
    /// The lashing was nettle cordage. Hide strips and willow leave this false.
    #[serde(default)]
    pub bound_nettle: bool,
    /// Meat that has been held over a fire. Raw meat leaves this false.
    #[serde(default)]
    pub cooked: bool,
    /// Raw meat rubbed with salt. It can be eaten cold. Cooked meat leaves this false.
    #[serde(default)]
    pub salted: bool,
    /// A raw fish or a portion of raw meat hung in the smoke of a fire.
    /// It can be eaten cold. It does not spoil.
    #[serde(default)]
    pub smoked: bool,
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
    /// A carcass from the wolf, not the deer. Other materials leave this false.
    #[serde(default)]
    pub wolf: bool,
    /// A carcass from the otter, not the deer or the wolf. Other materials leave this false.
    #[serde(default)]
    pub otter: bool,
    /// A carcass from the fox, not the deer, the wolf, or the otter. Other materials leave this false.
    #[serde(default)]
    pub fox: bool,
    /// A carcass from the hare, not the deer, the wolf, the otter, or the fox.
    /// Other materials leave this false.
    #[serde(default)]
    pub hare: bool,
    /// A carcass from the beaver, not the deer or the other animals.
    /// Other materials leave this false.
    #[serde(default)]
    pub beaver: bool,
    /// Pine resin rubbed into a dugout, or into a bark cup.
    /// A pitched canoe does not keep soaking the person aboard it.
    /// A sealed bark cup keeps a drink. An unsealed one lets the drink soak through.
    #[serde(default)]
    pub pitched: bool,
    /// A bow drill that has already lit a fire. It stays in the arms, and it will not light another until it is carved anew.
    #[serde(default)]
    pub spent: bool,
}

impl Item {
    /// A sewn hide cloak on the body. It does not take a carry slot.
    /// Taking it off puts it back in the count. A hide that is not sewn still counts.
    pub(crate) fn worn_cloak(&self) -> bool {
        self.material == Material::Hide && self.worn && self.sewn
    }
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
    /// Set when someone feeds the fire an extra dry branch, or lights it
    /// with charcoal. A small fire smelts copper. Iron waits for this.
    #[serde(default)]
    pub hot: bool,
    /// Lit with a lump of charcoal. It is hot, and it burns for twice as long.
    #[serde(default)]
    pub charcoal: bool,
    /// A column of smoke rises from this fire until it goes out.
    /// Look names it. The person who sent it up does not announce it.
    #[serde(default)]
    pub column: bool,
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
    /// A drink in an unsealed bark cup soaks through on this tick.
    /// A sealed cup, or a drink held by a pot, keeps it.
    CupLeak(ItemId),
    /// A hide snare set on this cell. It springs on that tick.
    Snare(CellId),
    /// A fishing net set in this brook cell. It catches on that tick.
    NetCatch(CellId),
    /// A reed weir set in this brook cell. It catches on that tick, then is gone.
    Weir(CellId),
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
    /// A wet bank on this cell grows mint again.
    MintReturn(CellId),
    /// The pine on this cell has needles again.
    NeedleReturn(CellId),
    /// Pitch lumps lie under the pine on this cell again.
    ResinReturn(CellId),
    /// Willow cordage hangs on this bank again.
    WillowReturn(CellId),
    /// A strip of willow bark hangs on this willow again. Not the cordage.
    WillowBarkReturn(CellId),
    /// Nettles grow on this floor again.
    NettleReturn(CellId),
    /// Cattails grow in these shallows again.
    CattailReturn(CellId),
    /// Cattail fluff hangs on this stand again. Not the roots.
    FluffReturn(CellId),
    /// A flint nodule lies on this slope cell again.
    FlintReturn(CellId),
    /// A lump of ochre lies on this slope cell again.
    OchreReturn(CellId),
    /// Clay lies on this brook bank again.
    ClayReturn(CellId),
    /// A lump of copper ore lies on this brook cell again.
    CopperReturn(CellId),
    /// A lump of iron ore lies on this slope cell again.
    IronReturn(CellId),
    /// Reeds grow at this brook edge again.
    ReedReturn(CellId),
    /// Dry branches lie under this rock again.
    BranchReturn(CellId),
    /// Bark lies under this rock again.
    BarkReturn(CellId),
    /// A berry patch on this cell has fruit again.
    BerryReturn(CellId),
    /// A mushroom patch on this cell has mushrooms again.
    MushroomReturn(CellId),
    /// A deer stands on the trail again after the last one was brought down.
    DeerReturn,
    /// An otter is in the brook again after the last one was brought down.
    OtterReturn,
    /// A fox is on the dry floor again after the last one was brought down.
    FoxReturn,
    /// A hare is on the dry floor again after the last one was snared.
    HareReturn,
    /// A beaver is in the brook again after the last one was brought down.
    BeaverReturn,
    /// The otter swims one step along the brook when nobody is beside it.
    Otter,
    /// A beaver returns to its brook cell when that water is clear.
    Beaver,
    /// The fox trots one step on the dry floor when nobody is beside it.
    Fox,
    /// Bee stings on a body. Each tick, any sting whose time has come fades.
    Stings,
    /// Wet clothes on this person dry at the fire they are standing by.
    DryClothes(ActorId),
    /// Dry branches banked on the fire in this cell, under a cover of mud.
    Charcoal(CellId),
    /// Worms are in this damp earth again.
    WormReturn(CellId),
}

/// A seeded berry patch. Emptying it, by gathering or by the deer, arms the same wait.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct BerryPatch {
    pub material: Material,
    pub count: u32,
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

/// One otter. It keeps to the brook and the bank beside it.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Otter {
    pub cell: CellId,
    pub route_at: usize,
    /// It just returned. The swim on this tick waits.
    #[serde(default)]
    pub held: bool,
}

/// A short loop on the brook and the dry bank beside it. Orthogonal.
pub(crate) const OTTER_ROUTE: [(i32, i32); 6] = [(0, 1), (1, 1), (1, 2), (0, 2), (0, 3), (1, 3)];

/// One beaver. It keeps to the south brook, and the bank south of that water
/// when someone steps onto it. It does not share a cell with the otter.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Beaver {
    pub cell: CellId,
    /// It just slipped aside. The return on this tick waits.
    #[serde(default)]
    pub held: bool,
}

/// The brook cell a beaver keeps, and the dry bank it dives to.
pub(crate) const BEAVER_HOME: (i32, i32) = (0, 3);
pub(crate) const BEAVER_BANK: (i32, i32) = (0, 4);

/// One fox. It keeps to the dry valley floor, not the brook.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Fox {
    pub cell: CellId,
    pub route_at: usize,
    /// It just returned. The trot on this tick waits.
    #[serde(default)]
    pub held: bool,
}

/// A short run on the dry floor at the south-east. Orthogonal.
pub(crate) const FOX_ROUTE: [(i32, i32); 2] = [(5, 4), (6, 4)];

/// One hare. It keeps to one dry cell on the valley floor, south of the trail.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Hare {
    pub cell: CellId,
}

/// The dry floor where a new hare stands after the last one was snared.
pub(crate) const HARE_HOME: (i32, i32) = (2, 4);

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
    /// An otter on the brook, or on the bank beside it. Old valleys have none.
    #[serde(default)]
    pub(crate) otter: Option<Otter>,
    /// A fox on the dry valley floor. Old valleys have none.
    #[serde(default)]
    pub(crate) fox: Option<Fox>,
    /// A hare on one dry cell of the valley floor. Old valleys have none.
    #[serde(default)]
    pub(crate) hare: Option<Hare>,
    /// A beaver on the south brook, or the bank it dived to. Old valleys have none.
    #[serde(default)]
    pub(crate) beaver: Option<Beaver>,
    /// Cells the deer has left. The marks stay; decay is not in this valley.
    #[serde(default)]
    pub(crate) tracks: BTreeSet<CellId>,
    /// Cells where a berry pile reached zero. The patch puts the pile back
    /// after `berry_ticks`.
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
    /// Dry branches spent onto a fire under a cover of mud, waiting to char.
    /// The count is those branches. Absent once they char, or the fire dies first.
    #[serde(default)]
    pub(crate) banks: BTreeMap<CellId, u32>,
    /// A lean-to of branches. It stands in that cell and breaks the wind.
    #[serde(default)]
    pub(crate) lean_tos: BTreeSet<CellId>,
    /// A lean-to thatched with reeds. Rain and cold match the rock. A bare lean-to does not.
    #[serde(default)]
    pub(crate) thatched: BTreeSet<CellId>,
    /// A hide snare set on a cell, springing at this sim time. The strips are spent.
    #[serde(default)]
    pub(crate) snares: BTreeMap<CellId, u64>,
    /// Snares in `snares` that were set with willow cordage rather than hide strips.
    #[serde(default)]
    pub(crate) snare_willow: BTreeSet<CellId>,
    /// Snares in `snares` that were set with nettle cordage.
    #[serde(default)]
    pub(crate) snare_nettle: BTreeSet<CellId>,
    /// Brook cells where a fishing net is set. It stays until someone gathers it.
    #[serde(default)]
    pub(crate) set_nets: BTreeSet<CellId>,
    /// When that set net catches a fish. Absent once it has caught, or was gathered.
    #[serde(default)]
    pub(crate) net_catches_at: BTreeMap<CellId, u64>,
    /// A reed weir set in a brook cell, catching at this sim time.
    /// The reeds and the cordage are already spent. Absent once it has caught.
    #[serde(default)]
    pub(crate) weirs: BTreeMap<CellId, u64>,
    /// The heavy-log pile on this cell has been charred by a fire.
    #[serde(default)]
    pub(crate) charred_logs: BTreeSet<CellId>,
    /// A lifted heavy log that a fire has charred.
    #[serde(default)]
    pub(crate) charred_items: BTreeSet<ItemId>,
    /// Who is aboard which dugout canoe.
    #[serde(default)]
    pub(crate) aboard: BTreeMap<ActorId, ItemId>,
    /// A brook cell stays quiet until this sim time after a fish is taken there.
    /// Banks that touch that cell fish the same stretch.
    #[serde(default)]
    pub(crate) brook_quiet: BTreeMap<CellId, u64>,
    /// A cell a deer has just left, or where a bolt or a kill left tracks,
    /// until this sim time. The permanent hoofprints are separate.
    #[serde(default)]
    pub(crate) fresh_tracks: BTreeMap<CellId, u64>,
    /// The northwest corner, after a night howl, until this sim time.
    /// Look names the tracks for someone who can see the cell, then they
    /// fade like deer tracks. The wolf may already have left.
    #[serde(default)]
    pub(crate) wolf_tracks: BTreeMap<CellId, u64>,
    /// Where a wolf is standing after a night howl. Empty once it leaves.
    #[serde(default)]
    pub(crate) wolf_cell: Option<CellId>,
    /// The wolf is gone at this sim time. Tracks can outlast it. Day shows no wolf.
    #[serde(default)]
    pub(crate) wolf_until: u64,
    /// Slope cells where a salt crust forms again after it is taken.
    #[serde(default)]
    pub(crate) salt_licks: BTreeSet<CellId>,
    /// The dry margin of the still, where evaporated water leaves a salt crust.
    #[serde(default)]
    pub(crate) salt_pans: BTreeSet<CellId>,
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
    /// Damp earth that holds worms: the damp bank and the wet bank.
    /// Look names them until the handful is dug.
    #[serde(default)]
    pub(crate) worm_beds: BTreeSet<CellId>,
    /// When a dug bed has worms again. Look says the damp earth has none until then.
    #[serde(default)]
    pub(crate) worm_returns_at: BTreeMap<CellId, u64>,
    /// The rock face where a clear stone of quartz lies. It does not grow back.
    #[serde(default)]
    pub(crate) quartz_faces: BTreeSet<CellId>,
    /// Shade under the rock, where a handful of moss stays dry enough to light a fire.
    #[serde(default)]
    pub(crate) shade_moss: BTreeSet<CellId>,
    /// When a bare bank grows moss again. Look says the bank is bare until then.
    #[serde(default)]
    pub(crate) moss_returns_at: BTreeMap<CellId, u64>,
    /// The wet bank where wild mint grows. Bare once the handful is taken.
    #[serde(default)]
    pub(crate) mint_banks: BTreeSet<CellId>,
    /// When a bare wet bank grows mint again. Look says the bank has no mint until then.
    #[serde(default)]
    pub(crate) mint_returns_at: BTreeMap<CellId, u64>,
    /// The pine where needles lie. Bare once the handful is taken. Pitch is separate.
    #[serde(default)]
    pub(crate) needle_stands: BTreeSet<CellId>,
    /// When a bare pine has needles again. Look says the pine has no needles until then.
    #[serde(default)]
    pub(crate) needle_returns_at: BTreeMap<CellId, u64>,
    /// The pine on the dry floor. Bare once the last lump of pitch is taken.
    #[serde(default)]
    pub(crate) resin_stands: BTreeSet<CellId>,
    /// When a bare pine has pitch again. Look says the pine has no resin until then.
    #[serde(default)]
    pub(crate) resin_returns_at: BTreeMap<CellId, u64>,
    /// The willow on the dry bank south of the brook. Bare once the last bundle is taken.
    #[serde(default)]
    pub(crate) willow_stands: BTreeSet<CellId>,
    /// The same willow, where a strip of bark hangs. Not the cordage.
    /// Bare once that strip is taken, until it grows back.
    #[serde(default)]
    pub(crate) willow_bark_stands: BTreeSet<CellId>,
    /// When the bare willow has bark again. Look says it has none until then.
    #[serde(default)]
    pub(crate) willow_bark_returns_at: BTreeMap<CellId, u64>,
    /// When a bare willow has withies again. Look says the willow has none until then.
    #[serde(default)]
    pub(crate) willow_returns_at: BTreeMap<CellId, u64>,
    /// The nettles on the dry floor east of the trail. Bare once the last bundle is taken.
    #[serde(default)]
    pub(crate) nettle_stands: BTreeSet<CellId>,
    /// When bare nettles have grown again. Look says the nettles are bare until then.
    #[serde(default)]
    pub(crate) nettle_returns_at: BTreeMap<CellId, u64>,
    /// The cattails in the shallows of the still. Not the reeds at the brook.
    /// Bare once the last roots are dug.
    #[serde(default)]
    pub(crate) cattail_stands: BTreeSet<CellId>,
    /// When bare shallows have cattails again. Look says the shallows have none until then.
    #[serde(default)]
    pub(crate) cattail_returns_at: BTreeMap<CellId, u64>,
    /// When the cattails have fluff again. Look says they have none until then.
    /// Ice holds it, the same as the roots.
    #[serde(default)]
    pub(crate) fluff_returns_at: BTreeMap<CellId, u64>,
    /// Slope cells where a flint nodule lies again after it is taken.
    #[serde(default)]
    pub(crate) flint_beds: BTreeSet<CellId>,
    /// When a bare slope has a flint nodule again. Look says the slope has no flint until then.
    #[serde(default)]
    pub(crate) flint_returns_at: BTreeMap<CellId, u64>,
    /// Slope cells where a lump of ochre lies again after it is gathered.
    #[serde(default)]
    pub(crate) ochre_beds: BTreeSet<CellId>,
    /// When a bare slope has a lump of ochre again. Look says the slope has no ochre until then.
    #[serde(default)]
    pub(crate) ochre_returns_at: BTreeMap<CellId, u64>,
    /// Brook-bank cells whose seeded clay returns after the lumps are gathered.
    /// The count is the pile that lies there again.
    #[serde(default)]
    pub(crate) clay_beds: BTreeMap<CellId, u32>,
    /// When that bank has clay again. Look says there is none until then.
    #[serde(default)]
    pub(crate) clay_returns_at: BTreeMap<CellId, u64>,
    /// Brook cells where a lump of copper ore lies again after it is gathered.
    #[serde(default)]
    pub(crate) copper_beds: BTreeSet<CellId>,
    /// When that brook has copper ore again. Look says the brook has none until then.
    #[serde(default)]
    pub(crate) copper_returns_at: BTreeMap<CellId, u64>,
    /// Slope cells where a lump of iron ore lies again after it is gathered.
    #[serde(default)]
    pub(crate) iron_beds: BTreeSet<CellId>,
    /// When that slope has iron ore again. Look says the slope has none until then.
    #[serde(default)]
    pub(crate) iron_returns_at: BTreeMap<CellId, u64>,
    /// Brook-edge cells where reeds grow again after they are gathered.
    /// The count is the pile that grows there again.
    #[serde(default)]
    pub(crate) reed_beds: BTreeMap<CellId, u32>,
    /// When that edge has reeds again. Look says the edge has none until then.
    #[serde(default)]
    pub(crate) reed_returns_at: BTreeMap<CellId, u64>,
    /// The rock cells whose seeded dry branches return after they are taken.
    #[serde(default)]
    pub(crate) branch_beds: BTreeSet<CellId>,
    /// When that rock has a pile of dry branches again. Look says there are
    /// none under the rock until then.
    #[serde(default)]
    pub(crate) branch_returns_at: BTreeMap<CellId, u64>,
    /// The rock cells whose seeded bark returns after it is gathered or used.
    /// The count is the pile that lies there again. Slope bark is not in this.
    #[serde(default)]
    pub(crate) bark_beds: BTreeMap<CellId, u32>,
    /// When that rock has its bark again. Look says there is none until then.
    #[serde(default)]
    pub(crate) bark_returns_at: BTreeMap<CellId, u64>,
    /// Seeded berry patches. The count is what grows back after the pile is gone.
    #[serde(default)]
    pub(crate) berry_patches: BTreeMap<CellId, BerryPatch>,
    /// When an emptied patch has berries again. Look says they are gone until then.
    #[serde(default)]
    pub(crate) berry_returns_at: BTreeMap<CellId, u64>,
    /// Seeded mushroom patches. The count is what grows back after the pile is gone.
    #[serde(default)]
    pub(crate) mushroom_patches: BTreeMap<CellId, u32>,
    /// When an emptied patch has mushrooms again. Look says they are gone until then.
    #[serde(default)]
    pub(crate) mushroom_returns_at: BTreeMap<CellId, u64>,
    /// When a deer stands on the trail again. Look names no standing deer until then.
    #[serde(default)]
    pub(crate) deer_returns_at: Option<u64>,
    /// When an otter is in the brook again. Look names no live otter until then.
    #[serde(default)]
    pub(crate) otter_returns_at: Option<u64>,
    /// When a fox is on the dry floor again. Look names no live fox until then.
    #[serde(default)]
    pub(crate) fox_returns_at: Option<u64>,
    /// When a hare is on the dry floor again. Look names no live hare until then.
    #[serde(default)]
    pub(crate) hare_returns_at: Option<u64>,
    /// When a beaver is in the brook again. Look names no live beaver until then.
    #[serde(default)]
    pub(crate) beaver_returns_at: Option<u64>,
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
                // The seeded brook is (0,1), (1,1), (0,2), (1,2), plus one
                // cell farther south at (0,3). (1,3) stays the dry south bank.
                let region = if y == 0 {
                    Region::Slope
                } else if (x <= 1 && (y == 1 || y == 2)) || (x == 0 && y == 3) {
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
            otter: Some(Otter {
                cell: cell_id(1, 1).unwrap(),
                route_at: 1,
                held: false,
            }),
            fox: Some(Fox {
                cell: cell_id(5, 4).unwrap(),
                route_at: 0,
                held: false,
            }),
            hare: Some(Hare {
                cell: cell_id(HARE_HOME.0, HARE_HOME.1).unwrap(),
            }),
            beaver: Some(Beaver {
                cell: cell_id(BEAVER_HOME.0, BEAVER_HOME.1).unwrap(),
                held: false,
            }),
            tracks: BTreeSet::new(),
            spent_berries: BTreeSet::new(),
            deer_ate_berries: BTreeSet::new(),
            ochre_marks: BTreeSet::new(),
            charcoal_marks: BTreeSet::new(),
            banks: BTreeMap::new(),
            lean_tos: BTreeSet::new(),
            thatched: BTreeSet::new(),
            snares: BTreeMap::new(),
            snare_willow: BTreeSet::new(),
            snare_nettle: BTreeSet::new(),
            set_nets: BTreeSet::new(),
            net_catches_at: BTreeMap::new(),
            weirs: BTreeMap::new(),
            charred_logs: BTreeSet::new(),
            charred_items: BTreeSet::new(),
            aboard: BTreeMap::new(),
            brook_quiet: BTreeMap::new(),
            fresh_tracks: BTreeMap::new(),
            wolf_tracks: BTreeMap::new(),
            wolf_cell: None,
            wolf_until: 0,
            salt_licks: BTreeSet::new(),
            salt_pans: BTreeSet::new(),
            salt_returns_at: BTreeMap::new(),
            honey_nests: BTreeSet::new(),
            honey_returns_at: BTreeMap::new(),
            smoked_nests: BTreeSet::new(),
            bird_nests: BTreeSet::new(),
            egg_returns_at: BTreeMap::new(),
            moss_banks: BTreeSet::new(),
            worm_beds: BTreeSet::new(),
            worm_returns_at: BTreeMap::new(),
            quartz_faces: BTreeSet::new(),
            shade_moss: BTreeSet::new(),
            moss_returns_at: BTreeMap::new(),
            mint_banks: BTreeSet::new(),
            mint_returns_at: BTreeMap::new(),
            needle_stands: BTreeSet::new(),
            needle_returns_at: BTreeMap::new(),
            resin_stands: BTreeSet::new(),
            resin_returns_at: BTreeMap::new(),
            willow_stands: BTreeSet::new(),
            willow_bark_stands: BTreeSet::new(),
            willow_bark_returns_at: BTreeMap::new(),
            willow_returns_at: BTreeMap::new(),
            nettle_stands: BTreeSet::new(),
            nettle_returns_at: BTreeMap::new(),
            cattail_stands: BTreeSet::new(),
            cattail_returns_at: BTreeMap::new(),
            fluff_returns_at: BTreeMap::new(),
            flint_beds: BTreeSet::new(),
            flint_returns_at: BTreeMap::new(),
            ochre_beds: BTreeSet::new(),
            ochre_returns_at: BTreeMap::new(),
            clay_beds: BTreeMap::new(),
            clay_returns_at: BTreeMap::new(),
            copper_beds: BTreeSet::new(),
            copper_returns_at: BTreeMap::new(),
            iron_beds: BTreeSet::new(),
            iron_returns_at: BTreeMap::new(),
            reed_beds: BTreeMap::new(),
            reed_returns_at: BTreeMap::new(),
            branch_beds: BTreeSet::new(),
            branch_returns_at: BTreeMap::new(),
            bark_beds: BTreeMap::new(),
            bark_returns_at: BTreeMap::new(),
            berry_patches: BTreeMap::new(),
            berry_returns_at: BTreeMap::new(),
            mushroom_patches: BTreeMap::new(),
            mushroom_returns_at: BTreeMap::new(),
            deer_returns_at: None,
            otter_returns_at: None,
            fox_returns_at: None,
            hare_returns_at: None,
            beaver_returns_at: None,
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
        world.schedule(4, Wake::Otter);
        world.schedule(6, Wake::Beaver);
        world.schedule(4, Wake::Fox);
        let rock = cell_id(6, 1).unwrap();
        world.add_pile(rock, Material::Branch, 4, 0);
        world.branch_beds.insert(rock);
        world.add_pile(cell_id(6, 1).unwrap(), Material::Bark, 2, 0);
        world.bark_beds.insert(rock, 2);
        world.add_pile(cell_id(6, 1).unwrap(), Material::Berry, 4, 0);
        world.berry_patches.insert(
            cell_id(6, 1).unwrap(),
            BerryPatch {
                material: Material::Berry,
                count: 4,
            },
        );
        world.add_pile(cell_id(6, 1).unwrap(), Material::Pot, 1, 0);
        let pale = cell_id(1, 4).unwrap();
        world.add_pile(pale, Material::PaleBerry, 2, 0);
        world.berry_patches.insert(
            pale,
            BerryPatch {
                material: Material::PaleBerry,
                count: 2,
            },
        );
        world.add_pile(cell_id(4, 1).unwrap(), Material::BitterBerry, 2, 0);
        world.berry_patches.insert(
            cell_id(4, 1).unwrap(),
            BerryPatch {
                material: Material::BitterBerry,
                count: 2,
            },
        );
        let mushrooms = cell_id(3, 1).unwrap();
        world.add_pile(mushrooms, Material::Mushroom, 2, 0);
        world.mushroom_patches.insert(mushrooms, 2);
        world.add_pile(cell_id(5, 2).unwrap(), Material::Branch, 4, 0);
        world.add_pile(cell_id(1, 0).unwrap(), Material::Bark, 2, 0);
        world.add_pile(cell_id(2, 0).unwrap(), Material::Stone, 1, 0);
        let flint = cell_id(4, 0).unwrap();
        world.add_pile(flint, Material::Flint, 1, 0);
        world.flint_beds.insert(flint);
        let ochre = cell_id(3, 0).unwrap();
        world.add_pile(ochre, Material::Ochre, 1, 0);
        world.ochre_beds.insert(ochre);
        // A second lump, on the rock face. It does not grow back with the slope.
        // A clear stone of quartz lies on that same face. It does not grow back.
        let face = cell_id(6, 1).unwrap();
        world.add_pile(face, Material::Ochre, 1, 0);
        world.add_pile(face, Material::Quartz, 1, 0);
        world.quartz_faces.insert(face);
        let iron = cell_id(5, 0).unwrap();
        world.add_pile(iron, Material::IronOre, 1, 0);
        world.iron_beds.insert(iron);
        let copper = cell_id(1, 2).unwrap();
        world.add_pile(copper, Material::CopperOre, 1, 0);
        world.copper_beds.insert(copper);
        let clay = cell_id(2, 2).unwrap();
        world.add_pile(clay, Material::Clay, 2, 0);
        world.clay_beds.insert(clay, 2);
        // The same wet bank, beside the brook and the still. One handful of mint.
        world.add_pile(clay, Material::Mint, 1, 0);
        world.mint_banks.insert(clay);
        let reeds = cell_id(0, 3).unwrap();
        world.add_pile(reeds, Material::Reed, 4, 0);
        world.reed_beds.insert(reeds, 4);
        let cattails = cell_id(2, 1).unwrap();
        world.add_pile(cattails, Material::Cattail, 2, 0);
        world.add_pile(cattails, Material::Fluff, 1, 0);
        world.cattail_stands.insert(cattails);
        world.add_pile(cell_id(3, 2).unwrap(), Material::Log, 1, 0);
        world.add_pile(cell_id(0, 0).unwrap(), Material::Salt, 1, 0);
        world.salt_licks.insert(cell_id(0, 0).unwrap());
        let pan = cell_id(2, 0).unwrap();
        world.add_pile(pan, Material::Salt, 1, 0);
        world.salt_pans.insert(pan);
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
        world.add_pile(bank, Material::Worm, 1, 0);
        world.worm_beds.insert(bank);
        world.add_pile(clay, Material::Worm, 1, 0);
        world.worm_beds.insert(clay);
        // Shade under the rock. This handful stays dry, and it lights a fire the way bark does.
        let shade = cell_id(6, 1).unwrap();
        world.add_pile(shade, Material::Moss, 1, 0);
        world.shade_moss.insert(shade);
        // A pine on the dry floor south of the trail. Two lumps of pitch.
        // A handful of needles lies there too. Pitch and needles are not the same gather.
        let pine = cell_id(2, 4).unwrap();
        world.add_pile(pine, Material::Resin, 2, 0);
        world.resin_stands.insert(pine);
        world.add_pile(pine, Material::Needle, 1, 0);
        world.needle_stands.insert(pine);
        // A willow on the dry bank south of the brook. Two bundles of cordage.
        let willow = cell_id(0, 4).unwrap();
        world.add_pile(willow, Material::Willow, 2, 0);
        world.willow_stands.insert(willow);
        world.add_pile(willow, Material::WillowBark, 1, 0);
        world.willow_bark_stands.insert(willow);
        // Nettles on the dry floor east of the trail. Two bundles of cordage.
        let nettles = cell_id(6, 2).unwrap();
        world.add_pile(nettles, Material::Nettle, 2, 0);
        world.nettle_stands.insert(nettles);
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
                nettle_sting: false,
                clothes_dry_at: None,
                tasted_bitter: false,
                raw_mushroom_known: false,
                pale_berry_known: false,
                bark_known: false,
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

    pub fn otter_cell(&self) -> Option<(i32, i32)> {
        self.otter.as_ref().map(|otter| cell_xy(otter.cell))
    }

    pub fn beaver_cell(&self) -> Option<(i32, i32)> {
        self.beaver.as_ref().map(|beaver| cell_xy(beaver.cell))
    }

    pub fn fox_cell(&self) -> Option<(i32, i32)> {
        self.fox.as_ref().map(|fox| cell_xy(fox.cell))
    }

    pub fn hare_cell(&self) -> Option<(i32, i32)> {
        self.hare.as_ref().map(|hare| cell_xy(hare.cell))
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
        self.scare_wolf(None);
        self.arm_spears();
        self.tend_drying();
        self.tend_clothes();
        self.fire_due();
        self.tend_branch_beds();
        self.tend_bark_beds();
        self.normalize();
        self.trim_events();
        self.time += 1;
        self.howl_on_nightfall();
        self.crack_on_freeze();
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
            if matches!(
                material,
                Material::Berry | Material::BitterBerry | Material::PaleBerry
            ) {
                self.arm_berry_patch(emptied);
            }
            if material == Material::Mushroom {
                self.arm_mushroom_patch(emptied);
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
                wooden_tip: false,
                bound: false,
                bound_willow: false,
                bound_nettle: false,
                cooked: false,
                salted: false,
                smoked: false,
                spoiled: false,
                spoils_at: None,
                worn: false,
                pierced: false,
                sewn: false,
                small: false,
                shaped: false,
                in_basket: false,
                bone: false,
                wolf: false,
                otter: false,
                fox: false,
                hare: false,
                beaver: false,
                pitched: false,
                spent: false,
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
                && !food.smoked
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

    /// A drink gathered into an unsealed bark cup soaks through `cup_leak_ticks` later.
    /// Sealing the cup before that, or boiling the drink, leaves the wake with nothing to do.
    pub(crate) fn arm_cup_leak(&mut self, id: ItemId) {
        if !self
            .item(id)
            .is_some_and(|drink| drink.material == Material::Water)
        {
            return;
        }
        let at = self.time.saturating_add(self.tuning.cup_leak_ticks);
        self.schedule(at, Wake::CupLeak(id));
    }

    /// The lick is empty. Another crust forms `salt_ticks` after this step.
    /// A wake that finds salt already there does not add a second crust.
    pub(crate) fn arm_salt_lick(&mut self, cell: CellId) {
        if (!self.salt_licks.contains(&cell) && !self.salt_pans.contains(&cell))
            || self.salt_returns_at.contains_key(&cell)
        {
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

    /// The wet bank is bare. Mint returns on the same wait as moss.
    /// A wake that finds mint already there does not add a second handful.
    pub(crate) fn arm_mint_bank(&mut self, cell: CellId) {
        if !self.mint_banks.contains(&cell) || self.mint_returns_at.contains_key(&cell) {
            return;
        }
        if self.pile_count(cell, Material::Mint) > 0
            || !self.ground_ids(cell, Material::Mint).is_empty()
        {
            return;
        }
        let at = self.time.saturating_add(self.tuning.moss_ticks);
        self.mint_returns_at.insert(cell, at);
        self.schedule(at, Wake::MintReturn(cell));
    }

    /// The pine has no needles. They return on the same wait as moss.
    /// Pitch on that pine is a different pile and is not touched.
    pub(crate) fn arm_needle_stand(&mut self, cell: CellId) {
        if !self.needle_stands.contains(&cell) || self.needle_returns_at.contains_key(&cell) {
            return;
        }
        if self.pile_count(cell, Material::Needle) > 0
            || !self.ground_ids(cell, Material::Needle).is_empty()
        {
            return;
        }
        let at = self.time.saturating_add(self.tuning.moss_ticks);
        self.needle_returns_at.insert(cell, at);
        self.schedule(at, Wake::NeedleReturn(cell));
    }

    /// The pine is bare. Pitch returns `resin_ticks` after this step.
    /// A wake that finds resin already there does not add a second stand.
    pub(crate) fn arm_resin_stand(&mut self, cell: CellId) {
        if !self.resin_stands.contains(&cell) || self.resin_returns_at.contains_key(&cell) {
            return;
        }
        if self.pile_count(cell, Material::Resin) > 0
            || !self.ground_ids(cell, Material::Resin).is_empty()
        {
            return;
        }
        let at = self.time.saturating_add(self.tuning.resin_ticks);
        self.resin_returns_at.insert(cell, at);
        self.schedule(at, Wake::ResinReturn(cell));
    }

    /// The willow is bare. Withies return `willow_ticks` after this step.
    /// A wake that finds cordage already there does not add a second stand.
    pub(crate) fn arm_willow_stand(&mut self, cell: CellId) {
        if !self.willow_stands.contains(&cell) || self.willow_returns_at.contains_key(&cell) {
            return;
        }
        if self.pile_count(cell, Material::Willow) > 0
            || !self.ground_ids(cell, Material::Willow).is_empty()
        {
            return;
        }
        let at = self.time.saturating_add(self.tuning.willow_ticks);
        self.willow_returns_at.insert(cell, at);
        self.schedule(at, Wake::WillowReturn(cell));
    }

    /// The willow has no bark. One strip returns on the same wait as moss.
    /// Cordage on that willow is a different pile and is not touched.
    pub(crate) fn arm_willow_bark(&mut self, cell: CellId) {
        if !self.willow_bark_stands.contains(&cell)
            || self.willow_bark_returns_at.contains_key(&cell)
        {
            return;
        }
        if self.pile_count(cell, Material::WillowBark) > 0
            || !self.ground_ids(cell, Material::WillowBark).is_empty()
        {
            return;
        }
        let at = self.time.saturating_add(self.tuning.moss_ticks);
        self.willow_bark_returns_at.insert(cell, at);
        self.schedule(at, Wake::WillowBarkReturn(cell));
    }

    /// The nettles are bare. They grow again `nettle_ticks` after this step.
    /// A wake that finds cordage already there does not add a second stand.
    pub(crate) fn arm_nettle_stand(&mut self, cell: CellId) {
        if !self.nettle_stands.contains(&cell) || self.nettle_returns_at.contains_key(&cell) {
            return;
        }
        if self.pile_count(cell, Material::Nettle) > 0
            || !self.ground_ids(cell, Material::Nettle).is_empty()
        {
            return;
        }
        let at = self.time.saturating_add(self.tuning.nettle_ticks);
        self.nettle_returns_at.insert(cell, at);
        self.schedule(at, Wake::NettleReturn(cell));
    }

    /// The shallows are dug out. Cattails grow again `cattail_ticks` after this step.
    /// A wake that finds roots already there does not add a second stand.
    pub(crate) fn arm_cattail_stand(&mut self, cell: CellId) {
        if !self.cattail_stands.contains(&cell) || self.cattail_returns_at.contains_key(&cell) {
            return;
        }
        if self.pile_count(cell, Material::Cattail) > 0
            || !self.ground_ids(cell, Material::Cattail).is_empty()
        {
            return;
        }
        let at = self.time.saturating_add(self.tuning.cattail_ticks);
        self.cattail_returns_at.insert(cell, at);
        self.schedule(at, Wake::CattailReturn(cell));
    }

    /// The cattails have no fluff. It returns on the same wait as the roots.
    /// Ice, or a dug-out stand, holds that wake. A wake that finds fluff
    /// already there does not add a second handful.
    pub(crate) fn arm_fluff(&mut self, cell: CellId) {
        if !self.cattail_stands.contains(&cell) || self.fluff_returns_at.contains_key(&cell) {
            return;
        }
        if self.pile_count(cell, Material::Fluff) > 0
            || !self.ground_ids(cell, Material::Fluff).is_empty()
        {
            return;
        }
        let at = self.time.saturating_add(self.tuning.cattail_ticks);
        self.fluff_returns_at.insert(cell, at);
        self.schedule(at, Wake::FluffReturn(cell));
    }

    /// The damp earth has no worms. They return `moss_ticks` after this step.
    /// A wake that finds a worm already there does not add a second one.
    pub(crate) fn arm_worm_bed(&mut self, cell: CellId) {
        if !self.worm_beds.contains(&cell) || self.worm_returns_at.contains_key(&cell) {
            return;
        }
        if self.pile_count(cell, Material::Worm) > 0
            || !self.ground_ids(cell, Material::Worm).is_empty()
        {
            return;
        }
        let at = self.time.saturating_add(self.tuning.moss_ticks);
        self.worm_returns_at.insert(cell, at);
        self.schedule(at, Wake::WormReturn(cell));
    }

    /// An emptied berry patch grows the same pile back after `berry_ticks`.
    /// A wake that finds berries already there does not add a second pile.
    pub(crate) fn arm_berry_patch(&mut self, cell: CellId) {
        if !self.berry_patches.contains_key(&cell) || self.berry_returns_at.contains_key(&cell) {
            return;
        }
        let Some(patch) = self.berry_patches.get(&cell).copied() else {
            return;
        };
        if self.pile_count(cell, patch.material) > 0 {
            return;
        }
        let at = self.time.saturating_add(self.tuning.berry_ticks);
        self.berry_returns_at.insert(cell, at);
        self.schedule(at, Wake::BerryReturn(cell));
    }

    /// An emptied mushroom patch grows the same pile back after `mushroom_ticks`.
    /// Gathering the last one, or cooking it off the pile, leaves the same wait.
    /// A wake that finds mushrooms already there does not add a second pile.
    pub(crate) fn arm_mushroom_patch(&mut self, cell: CellId) {
        if !self.mushroom_patches.contains_key(&cell)
            || self.mushroom_returns_at.contains_key(&cell)
        {
            return;
        }
        if self.pile_count(cell, Material::Mushroom) > 0 {
            return;
        }
        let at = self.time.saturating_add(self.tuning.mushroom_ticks);
        self.mushroom_returns_at.insert(cell, at);
        self.schedule(at, Wake::MushroomReturn(cell));
    }

    /// The patch is waiting. Look says the mushrooms are gone.
    pub(crate) fn mushroom_patch_bare(&self, cell: CellId) -> bool {
        self.mushroom_returns_at.contains_key(&cell)
            && self.mushroom_patches.contains_key(&cell)
            && self.pile_count(cell, Material::Mushroom) == 0
    }

    /// The patch is waiting. Look says the berries are gone.
    pub(crate) fn berry_patch_bare(&self, cell: CellId, material: Material) -> bool {
        self.berry_returns_at.contains_key(&cell)
            && self
                .berry_patches
                .get(&cell)
                .is_some_and(|patch| patch.material == material)
            && self.pile_count(cell, material) == 0
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

    /// The slope has no lump of ochre. Another lies there `ochre_ticks` after
    /// this step. Gathering it leaves that wait. Smearing or marking with the
    /// lump that was carried does not start another. A wake that finds a lump
    /// already there does not add a second one.
    pub(crate) fn arm_ochre_bed(&mut self, cell: CellId) {
        if !self.ochre_beds.contains(&cell) || self.ochre_returns_at.contains_key(&cell) {
            return;
        }
        if self.pile_count(cell, Material::Ochre) > 0
            || !self.ground_ids(cell, Material::Ochre).is_empty()
        {
            return;
        }
        let at = self.time.saturating_add(self.tuning.ochre_ticks);
        self.ochre_returns_at.insert(cell, at);
        self.schedule(at, Wake::OchreReturn(cell));
    }

    /// An unshaped lump is still on this bank, in the pile or lying loose.
    /// A pot, or a lump already in the fire, does not count.
    pub(crate) fn clay_bed_holding(&self, cell: CellId) -> bool {
        if self.pile_count(cell, Material::Clay) > 0 {
            return true;
        }
        self.ground_ids(cell, Material::Clay).iter().any(|id| {
            self.item(*id)
                .is_some_and(|clay| !clay.shaped && clay.firing_ready_at.is_none())
        })
    }

    /// The brook bank has no clay lumps. The same pile lies there `clay_bank_ticks`
    /// after this step. Gathering the last lump, or shaping it off the pile,
    /// leaves that wait. A wake that finds clay already there does not add
    /// a second pile.
    pub(crate) fn arm_clay_bank(&mut self, cell: CellId) {
        if !self.clay_beds.contains_key(&cell) || self.clay_returns_at.contains_key(&cell) {
            return;
        }
        if self.clay_bed_holding(cell) {
            return;
        }
        let at = self.time.saturating_add(self.tuning.clay_bank_ticks);
        self.clay_returns_at.insert(cell, at);
        self.schedule(at, Wake::ClayReturn(cell));
    }

    /// A lump of copper ore is still lying on this brook, not set in a fire.
    pub(crate) fn copper_bed_holding(&self, cell: CellId) -> bool {
        if self.pile_count(cell, Material::CopperOre) > 0 {
            return true;
        }
        self.ground_ids(cell, Material::CopperOre).iter().any(|id| {
            self.item(*id)
                .is_some_and(|ore| ore.firing_ready_at.is_none())
        })
    }

    /// The brook has no copper ore. Another lump lies there `copper_ore_ticks`
    /// after this step. Gathering it leaves that wait. Smelting the lump that
    /// was carried does not start another. A wake that finds a lump already
    /// there does not add a second one.
    pub(crate) fn arm_copper_bed(&mut self, cell: CellId) {
        if !self.copper_beds.contains(&cell) || self.copper_returns_at.contains_key(&cell) {
            return;
        }
        if self.copper_bed_holding(cell) {
            return;
        }
        let at = self.time.saturating_add(self.tuning.copper_ore_ticks);
        self.copper_returns_at.insert(cell, at);
        self.schedule(at, Wake::CopperReturn(cell));
    }

    /// A lump of iron ore is still lying on this slope, not set in a fire.
    pub(crate) fn iron_bed_holding(&self, cell: CellId) -> bool {
        if self.pile_count(cell, Material::IronOre) > 0 {
            return true;
        }
        self.ground_ids(cell, Material::IronOre).iter().any(|id| {
            self.item(*id)
                .is_some_and(|ore| ore.firing_ready_at.is_none())
        })
    }

    /// The slope has no iron ore. Another lump lies there `iron_ore_ticks`
    /// after this step. Gathering it leaves that wait. Smelting the lump that
    /// was carried does not start another. A wake that finds a lump already
    /// there does not add a second one.
    pub(crate) fn arm_iron_bed(&mut self, cell: CellId) {
        if !self.iron_beds.contains(&cell) || self.iron_returns_at.contains_key(&cell) {
            return;
        }
        if self.iron_bed_holding(cell) {
            return;
        }
        let at = self.time.saturating_add(self.tuning.iron_ore_ticks);
        self.iron_returns_at.insert(cell, at);
        self.schedule(at, Wake::IronReturn(cell));
    }

    /// A bundle of reeds is still on this edge, in the pile or lying loose.
    pub(crate) fn reed_bed_holding(&self, cell: CellId) -> bool {
        self.pile_count(cell, Material::Reed) > 0
            || !self.ground_ids(cell, Material::Reed).is_empty()
    }

    /// The brook's edge has no reeds. The same pile grows there `reed_ticks`
    /// after this step. Gathering the last bundle leaves that wait. Thatching
    /// the last bundle off the edge leaves the same wait. A wake that finds
    /// reeds already there does not add a second pile.
    pub(crate) fn arm_reed_bed(&mut self, cell: CellId) {
        if !self.reed_beds.contains_key(&cell) || self.reed_returns_at.contains_key(&cell) {
            return;
        }
        if self.reed_bed_holding(cell) {
            return;
        }
        let at = self.time.saturating_add(self.tuning.reed_ticks);
        self.reed_returns_at.insert(cell, at);
        self.schedule(at, Wake::ReedReturn(cell));
    }

    /// The seeded branches under a rock are gone, gathered or snapped away.
    /// The same dry pile lies there again `branch_ticks` after this step.
    /// A wake that finds branches already there does not add a second pile.
    pub(crate) fn tend_branch_beds(&mut self) {
        let beds: Vec<CellId> = self.branch_beds.iter().copied().collect();
        for cell in beds {
            if self.branch_returns_at.contains_key(&cell) {
                continue;
            }
            if self.pile_count(cell, Material::Branch) > 0
                || !self.ground_ids(cell, Material::Branch).is_empty()
            {
                continue;
            }
            let at = self.time.saturating_add(self.tuning.branch_ticks);
            self.branch_returns_at.insert(cell, at);
            self.schedule(at, Wake::BranchReturn(cell));
        }
    }

    /// The seeded bark under a rock is gone, gathered or used.
    /// The same pile lies there again `bark_pile_ticks` after this step.
    /// A wake that finds bark already there does not add a second pile.
    /// Bark on the slope is not a bed, so it does not return.
    pub(crate) fn tend_bark_beds(&mut self) {
        let beds: Vec<(CellId, u32)> = self
            .bark_beds
            .iter()
            .map(|(cell, count)| (*cell, *count))
            .collect();
        for (cell, _count) in beds {
            if self.bark_returns_at.contains_key(&cell) {
                continue;
            }
            if self.pile_count(cell, Material::Bark) > 0
                || !self.ground_ids(cell, Material::Bark).is_empty()
            {
                continue;
            }
            let at = self.time.saturating_add(self.tuning.bark_pile_ticks);
            self.bark_returns_at.insert(cell, at);
            self.schedule(at, Wake::BarkReturn(cell));
        }
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

    /// The otter is down. Another is in the brook after the same wait as the
    /// deer. A carcass still lying where it fell does not move that tick.
    /// A wake that finds an otter already swimming does not add a second one.
    pub(crate) fn arm_otter_return(&mut self) {
        if self.otter.is_some() || self.otter_returns_at.is_some() {
            return;
        }
        let at = self.time.saturating_add(self.tuning.deer_ticks);
        self.otter_returns_at = Some(at);
        self.schedule(at, Wake::OtterReturn);
    }

    /// The fox is down. Another is on the dry floor after the same wait as the
    /// otter. A carcass still lying where it fell does not move that tick.
    /// A wake that finds a fox already standing does not add a second one.
    pub(crate) fn arm_fox_return(&mut self) {
        if self.fox.is_some() || self.fox_returns_at.is_some() {
            return;
        }
        let at = self.time.saturating_add(self.tuning.deer_ticks);
        self.fox_returns_at = Some(at);
        self.schedule(at, Wake::FoxReturn);
    }

    /// The hare is snared. Another stands on the dry floor after the same wait
    /// as the fox. A carcass still lying where it fell does not move that tick.
    /// A wake that finds a hare already there does not add a second one.
    pub(crate) fn arm_hare_return(&mut self) {
        if self.hare.is_some() || self.hare_returns_at.is_some() {
            return;
        }
        let at = self.time.saturating_add(self.tuning.deer_ticks);
        self.hare_returns_at = Some(at);
        self.schedule(at, Wake::HareReturn);
    }

    /// The beaver is down. Another is in the south brook after the same wait
    /// as the otter. A carcass still lying where it fell does not move that
    /// tick. A wake that finds a beaver already there does not add a second one.
    pub(crate) fn arm_beaver_return(&mut self) {
        if self.beaver.is_some() || self.beaver_returns_at.is_some() {
            return;
        }
        let at = self.time.saturating_add(self.tuning.deer_ticks);
        self.beaver_returns_at = Some(at);
        self.schedule(at, Wake::BeaverReturn);
    }

    pub(crate) fn carry_count(&self, actor: ActorId) -> u32 {
        self.items
            .values()
            .filter(|i| i.carrier == Some(actor) && !i.in_basket && !i.worn_cloak())
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

    /// A dry handful of moss in the arms. Moss still on the ground is not tinder yet.
    pub(crate) fn moss_tinder_in_arms(&self, actor: ActorId) -> bool {
        !self.dry_loose_ids(actor, Material::Moss).is_empty()
    }

    /// Cattail fluff in the arms that can still catch. Slightly damp is enough.
    /// Soaked fluff cannot. Fluff still on the cattails is not tinder yet.
    pub(crate) fn fluff_tinder_in_arms(&self, actor: ActorId) -> bool {
        self.loose_ids(actor, Material::Fluff)
            .into_iter()
            .any(|id| {
                self.item(id)
                    .is_some_and(|item| item.moisture < self.tuning.soaked_at)
            })
    }

    /// Dry branches lying on this cell, pile and loose. Carried wood stays in the arms.
    pub(crate) fn dry_wood_on_ground(&self, cell: CellId) -> u32 {
        let loose = self
            .ground_ids(cell, Material::Branch)
            .into_iter()
            .filter(|id| {
                self.item(*id)
                    .is_some_and(|item| self.is_dry(item.moisture))
            })
            .count() as u32;
        self.dry_pile_count(cell, Material::Branch) + loose
    }

    /// Spend that dry wood. The count is what left the ground.
    pub(crate) fn take_dry_wood_on_ground(&mut self, cell: CellId) -> u32 {
        let pile = self.dry_pile_count(cell, Material::Branch);
        if pile > 0 {
            self.take_from_pile(cell, Material::Branch, pile);
        }
        let loose: Vec<_> = self
            .ground_ids(cell, Material::Branch)
            .into_iter()
            .filter(|id| {
                self.item(*id)
                    .is_some_and(|item| self.is_dry(item.moisture))
            })
            .collect();
        let n = loose.len() as u32;
        for id in loose {
            self.items.remove(&id);
        }
        pile + n
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
        self.mint_burning(cell, lit_event, false)
    }

    /// A lump of charcoal instead of a branch. Hot from the start, and twice
    /// the life of a bark-and-branch fire.
    pub(crate) fn mint_charcoal_fire(&mut self, cell: CellId, lit_event: EventId) -> FireId {
        self.mint_burning(cell, lit_event, true)
    }

    /// The wood chars on the step that starts at `until`. A fire due then, or
    /// sooner, would burn down first and the bank would fail. Hold it one tick
    /// past that step. Rain, or anything else that puts the fire out, still wins.
    pub(crate) fn hold_banked_fire(&mut self, cell: CellId, until: u64) {
        let Some(id) = self.fire_at(cell).map(|fire| fire.id) else {
            return;
        };
        let goes = self
            .fires
            .get(&id)
            .map(|fire| fire.goes_out_at)
            .unwrap_or(0);
        if goes > until {
            return;
        }
        let hold = until.saturating_add(1);
        if let Some(fire) = self.fires.get_mut(&id) {
            fire.goes_out_at = hold;
        }
        self.schedule.retain(|entry| entry.wake != Wake::Fire(id));
        self.schedule(hold, Wake::Fire(id));
    }

    fn mint_burning(&mut self, cell: CellId, lit_event: EventId, charcoal: bool) -> FireId {
        let id = self.next_fire;
        self.next_fire += 1;
        let span = if charcoal {
            self.tuning.fire_ticks.saturating_mul(2)
        } else {
            self.tuning.fire_ticks
        };
        let goes_out_at = self.time.saturating_add(span);
        self.fires.insert(
            id,
            Fire {
                id,
                cell,
                lit_event,
                goes_out_at,
                hot: charcoal,
                charcoal,
                column: false,
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
        // A banked fire's wood is the charcoal. The leftover lump waits
        // until that wood chars, or comes back as branches if the fire dies.
        if !self.banks.contains_key(&fire.cell) {
            self.mint_item(Material::Charcoal, 0, fire.cell, None, Role::Loose);
        }
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

    /// The heavy log on this cell, pile or lifted piece, and whether a fire has charred it.
    pub(crate) fn log_charred_at(&self, cell: CellId) -> bool {
        if self.charred_logs.contains(&cell) && self.pile_count(cell, Material::Log) > 0 {
            return true;
        }
        self.items.values().any(|item| {
            item.cell == cell
                && item.carrier.is_none()
                && item.material == Material::Log
                && self.charred_items.contains(&item.id)
        })
    }

    /// A charred log is light enough for one person. A raw log still needs two.
    pub(crate) fn hands_on_the_log(&self, cell: CellId) -> u32 {
        if self.log_charred_at(cell) {
            1
        } else {
            self.tuning.drag_needed
        }
    }

    /// The dugout on this cell, or the one floating on the next cell.
    /// A beached canoe, one afloat underfoot, and one you are aboard all count.
    pub(crate) fn canoe_to_seal(&self, actor: ActorId) -> Option<ItemId> {
        let person = self.actor(actor)?;
        if let Some(id) = self.aboard.get(&actor).copied() {
            if self
                .item(id)
                .is_some_and(|canoe| canoe.material == Material::Canoe)
            {
                return Some(id);
            }
        }
        if let Some(id) = self.canoe_at(person.cell) {
            return Some(id);
        }
        self.floating_canoe_near(person.cell)
    }

    /// A bark cup in the arms, or one lying at the feet. The arms win.
    /// An unsealed cup is preferred so a second sealed one does not hide it.
    pub(crate) fn cup_to_seal(&self, actor_id: ActorId) -> Option<ItemId> {
        let mut ids = self.loose_ids(actor_id, Material::BarkCup);
        if let Some(actor) = self.actor(actor_id) {
            ids.extend(self.ground_ids(actor.cell, Material::BarkCup));
        }
        ids.iter()
            .copied()
            .find(|id| self.item(*id).is_some_and(|cup| !cup.pitched))
            .or_else(|| ids.first().copied())
    }

    pub(crate) fn canoe_pitched(&self, id: ItemId) -> bool {
        self.item(id)
            .is_some_and(|canoe| canoe.material == Material::Canoe && canoe.pitched)
    }

    pub(crate) fn canoe_at(&self, cell: CellId) -> Option<ItemId> {
        self.items.values().find_map(|item| {
            (item.cell == cell && item.carrier.is_none() && item.material == Material::Canoe)
                .then_some(item.id)
        })
    }

    /// A dugout in the flowing brook, not one still on the bank.
    pub(crate) fn canoe_floats(&self, id: ItemId) -> bool {
        self.item(id).is_some_and(|canoe| {
            canoe.material == Material::Canoe && self.cell(canoe.cell).region == Region::Brook
        })
    }

    /// The brook cell beside this bank, preferring west, then north, south, and east.
    pub(crate) fn brook_neighbor(&self, cell: CellId) -> Option<CellId> {
        let (x, y) = cell_xy(cell);
        for (dx, dy) in [(-1, 0), (0, -1), (0, 1), (1, 0)] {
            let Some(next) = cell_id(x + dx, y + dy) else {
                continue;
            };
            if self.cell(next).region == Region::Brook {
                return Some(next);
            }
        }
        None
    }

    /// The next brook cell in that direction, if the canoe can go there.
    pub(crate) fn brook_step(&self, cell: CellId, dir: Dir) -> Option<CellId> {
        let (x, y) = cell_xy(cell);
        let (dx, dy) = dir.delta();
        let next = cell_id(x + dx, y + dy)?;
        (self.cell(next).region == Region::Brook).then_some(next)
    }

    /// Which way a bare or named paddle can go. The bank and the map edge refuse.
    pub(crate) fn paddle_way(&self, cell: CellId, dir: Option<Dir>) -> Result<Dir, &'static str> {
        if let Some(dir) = dir {
            if self.brook_step(cell, dir).is_some() {
                return Ok(dir);
            }
            let (x, y) = cell_xy(cell);
            let (dx, dy) = dir.delta();
            return if cell_id(x + dx, y + dy).is_none() {
                Err("There is nowhere to paddle.")
            } else {
                Err("The canoe stays in the brook.")
            };
        }
        for way in [Dir::North, Dir::South, Dir::East, Dir::West] {
            if self.brook_step(cell, way).is_some() {
                return Ok(way);
            }
        }
        Err("The canoe has nowhere to go.")
    }

    /// A cell beside this brook that is not flowing water. Dry ground comes first.
    pub(crate) fn bank_beside(&self, cell: CellId) -> Option<CellId> {
        let (x, y) = cell_xy(cell);
        let mut dry = Vec::new();
        let mut other = Vec::new();
        for (dx, dy) in [(0, 1), (1, 0), (0, -1), (-1, 0)] {
            let Some(next) = cell_id(x + dx, y + dy) else {
                continue;
            };
            if self.cell(next).region == Region::Brook {
                continue;
            }
            if self.cell(next).still {
                other.push(next);
            } else {
                dry.push(next);
            }
        }
        dry.into_iter().next().or_else(|| other.into_iter().next())
    }

    /// Where a floating dugout can be beached. A person already on the bank
    /// hauls it onto that cell. Someone in the brook, aboard or not, takes
    /// the nearest dry bank.
    pub(crate) fn beach_bank(&self, actor: ActorId) -> Option<CellId> {
        let person = self.actor(actor)?;
        let canoe = self.floating_canoe_near(person.cell)?;
        let water = self.item(canoe)?.cell;
        if person.cell != water && self.cell(person.cell).region != Region::Brook {
            return Some(person.cell);
        }
        self.bank_beside(water)
    }

    /// Why `beach` cannot haul. A dugout already lying on this cell is not missing.
    pub(crate) fn beach_miss(&self, actor: ActorId) -> &'static str {
        let Some(person) = self.actor(actor) else {
            return "There is no dugout canoe here.";
        };
        if self
            .canoe_at(person.cell)
            .is_some_and(|id| !self.canoe_floats(id))
        {
            "The canoe is already on land."
        } else {
            "There is no dugout canoe here."
        }
    }

    /// A floating dugout on this cell or the next.
    pub(crate) fn floating_canoe_near(&self, cell: CellId) -> Option<ItemId> {
        self.items.values().find_map(|item| {
            (item.material == Material::Canoe
                && item.carrier.is_none()
                && self.cell(item.cell).region == Region::Brook
                && chebyshev(cell, item.cell) <= 1)
                .then_some(item.id)
        })
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

    /// Still water freezes once the evening has chilled, and again each night.
    /// The day after dawn thaws it. The brook keeps running. A fire on the
    /// pool keeps it open while the cold would otherwise freeze it.
    pub(crate) fn still_would_freeze(&self, cell: CellId) -> bool {
        self.time >= self.tuning.chill_at
            && self.cell(cell).still
            && self.cell(cell).region != Region::Brook
            && !self.morning()
    }

    pub(crate) fn still_frozen(&self, cell: CellId) -> bool {
        self.still_would_freeze(cell) && self.fire_at(cell).is_none()
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

    /// A heavy log, or a tree that still stands here. Brush is the loose branches.
    pub(crate) fn has_timber(&self, cell: CellId) -> bool {
        self.pile_count(cell, Material::Log) > 0
            || self.ground_ids(cell, Material::Log).is_empty() == false
            || self.resin_stands.contains(&cell)
            || self.willow_stands.contains(&cell)
            || self.honey_nests.contains(&cell)
            || self.bird_nests.contains(&cell)
    }

    /// The sound of one step onto brush or timber. Nothing on open ground.
    /// `snapped` keeps the sound when that step took the last loose branch.
    pub(crate) fn brush_crack(&self, cell: CellId, snapped: bool) -> Option<&'static str> {
        if self.has_brush(cell) || snapped {
            Some("Branches break in the brush.")
        } else if self.has_timber(cell) {
            Some("Wood cracks in the timber.")
        } else {
            None
        }
    }

    /// One crack for this step. People who can see the cell do not get it.
    /// Someone out of sight but within hearing gets the direction. Not every tick.
    pub(crate) fn note_brush(&mut self, cell: CellId, snapped: bool) {
        let Some(text) = self.brush_crack(cell, snapped) else {
            return;
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

    pub(crate) fn cookable_carried(&self, actor: ActorId) -> bool {
        !self.loose_ids(actor, Material::Carcass).is_empty()
            || self.raw_mushroom_id(actor).is_some()
            || self.raw_egg_id(actor).is_some()
            || !self.loose_ids(actor, Material::Berry).is_empty()
            || self.raw_pale_id(actor).is_some()
            || self.raw_cattail_id(actor).is_some()
            || self.raw_flour_id(actor).is_some()
    }

    /// Raw cattail roots in the arms. Cooked roots are already food.
    pub(crate) fn raw_cattail_id(&self, actor: ActorId) -> Option<ItemId> {
        self.loose_ids(actor, Material::Cattail)
            .into_iter()
            .find(|id| self.item(*id).is_some_and(|root| !root.cooked))
    }

    /// Cooked cattail roots, from the arms before the ground.
    pub(crate) fn cooked_cattail_id(&self, actor: ActorId) -> Option<ItemId> {
        let carried = self
            .loose_ids(actor, Material::Cattail)
            .into_iter()
            .find(|id| self.item(*id).is_some_and(|root| root.cooked));
        if carried.is_some() {
            return carried;
        }
        let cell = self.actor(actor)?.cell;
        self.ground_ids(cell, Material::Cattail)
            .into_iter()
            .find(|id| self.item(*id).is_some_and(|root| root.cooked))
    }

    pub(crate) fn raw_cattail_near(&self, actor: ActorId) -> bool {
        self.raw_cattail_id(actor).is_some()
            || self
                .actor(actor)
                .is_some_and(|person| self.pile_count(person.cell, Material::Cattail) > 0)
    }

    /// Roots dry enough to pound. Raw first, then cooked. A soaked handful is not flour.
    pub(crate) fn grind_root(&self, actor: ActorId) -> Result<ItemId, &'static str> {
        let ids = self.loose_ids(actor, Material::Cattail);
        let pick = |cooked: bool| {
            ids.iter().copied().find(|id| {
                self.item(*id).is_some_and(|root| {
                    root.cooked == cooked && root.moisture < self.tuning.soaked_at
                })
            })
        };
        if let Some(id) = pick(false) {
            return Ok(id);
        }
        if let Some(id) = pick(true) {
            return Ok(id);
        }
        if ids.iter().any(|id| {
            self.item(*id)
                .is_some_and(|root| root.moisture >= self.tuning.soaked_at)
        }) {
            return Err("The cattail roots are too wet to grind.");
        }
        Err("You have no cattail roots to grind.")
    }

    /// A stone under the handstone. One on the ground, one in the pile, or the rock itself.
    pub(crate) fn stone_under(&self, actor: ActorId) -> bool {
        let Some(person) = self.actor(actor) else {
            return false;
        };
        self.pile_count(person.cell, Material::Stone) > 0
            || !self.ground_ids(person.cell, Material::Stone).is_empty()
            || self.cell(person.cell).rock
    }

    /// Cattail flour that has not been cooked. The arms, then the ground.
    pub(crate) fn raw_flour_id(&self, actor: ActorId) -> Option<ItemId> {
        let carried = self
            .loose_ids(actor, Material::Flour)
            .into_iter()
            .find(|id| self.item(*id).is_some_and(|flour| !flour.cooked));
        if carried.is_some() {
            return carried;
        }
        let cell = self.actor(actor)?.cell;
        self.ground_ids(cell, Material::Flour)
            .into_iter()
            .find(|id| self.item(*id).is_some_and(|flour| !flour.cooked))
    }

    /// A cattail flatbread. The arms, then the ground.
    pub(crate) fn cooked_flour_id(&self, actor: ActorId) -> Option<ItemId> {
        let carried = self
            .loose_ids(actor, Material::Flour)
            .into_iter()
            .find(|id| self.item(*id).is_some_and(|flour| flour.cooked));
        if carried.is_some() {
            return carried;
        }
        let cell = self.actor(actor)?.cell;
        self.ground_ids(cell, Material::Flour)
            .into_iter()
            .find(|id| self.item(*id).is_some_and(|flour| flour.cooked))
    }

    /// Stock on an orthogonal neighbor, named as a step. Empty when it is already here,
    /// or when the next cell that has it is not a single step. This does not move anyone.
    pub(crate) fn gather_step_cue(&self, cell: CellId, material: Material) -> Option<String> {
        if self.pile_count(cell, material) > 0 || !self.ground_ids(cell, material).is_empty() {
            return None;
        }
        let (x, y) = cell_xy(cell);
        let mut ways = Vec::new();
        for dir in Dir::all() {
            let (dx, dy) = dir.delta();
            let Some(dest) = cell_id(x + dx, y + dy) else {
                continue;
            };
            if self.pile_count(dest, material) > 0 || !self.ground_ids(dest, material).is_empty() {
                ways.push(dir.word());
            }
        }
        match ways.len() {
            0 => self.ore_diagonal_step(cell, material),
            1 => Some(format!(" Step {}.", ways[0])),
            _ => {
                let last = ways.pop()?;
                Some(format!(" Step {} or {last}.", ways.join(", ")))
            }
        }
    }

    /// One diagonal lump of ore, and no orthogonal one. The words match look:
    /// `Step north and west.` Anything else stays quiet.
    fn ore_diagonal_step(&self, cell: CellId, material: Material) -> Option<String> {
        if !matches!(material, Material::IronOre | Material::CopperOre) {
            return None;
        }
        let (x, y) = cell_xy(cell);
        let mut ways = Vec::new();
        for (dx, dy) in [(-1, -1), (1, -1), (-1, 1), (1, 1)] {
            let Some(dest) = cell_id(x + dx, y + dy) else {
                continue;
            };
            if self.pile_count(dest, material) > 0 || !self.ground_ids(dest, material).is_empty() {
                ways.push(self.relative(cell, dest));
            }
        }
        if ways.len() == 1 {
            Some(format!(" Step {}.", ways[0]))
        } else {
            None
        }
    }

    /// Drinkable water one step off. The brook always counts. Still water counts
    /// until it freezes. Water already here is not a step. One diagonal, and no
    /// orthogonal source, is named the way look names it. This does not move anyone.
    pub(crate) fn water_step_cue(&self, cell: CellId) -> Option<String> {
        if self.drinkable_water(cell) {
            return None;
        }
        let (x, y) = cell_xy(cell);
        let mut ways = Vec::new();
        for dir in Dir::all() {
            let (dx, dy) = dir.delta();
            let Some(dest) = cell_id(x + dx, y + dy) else {
                continue;
            };
            if self.drinkable_water(dest) {
                ways.push(dir.word());
            }
        }
        match ways.len() {
            0 => {
                let mut diag = Vec::new();
                for (dx, dy) in [(-1, -1), (1, -1), (-1, 1), (1, 1)] {
                    let Some(dest) = cell_id(x + dx, y + dy) else {
                        continue;
                    };
                    if self.drinkable_water(dest) {
                        diag.push(self.relative(cell, dest));
                    }
                }
                if diag.len() == 1 {
                    Some(format!(" Step {}.", diag[0]))
                } else {
                    None
                }
            }
            1 => Some(format!(" Step {}.", ways[0])),
            _ => {
                let last = ways.pop()?;
                Some(format!(" Step {} or {last}.", ways.join(", ")))
            }
        }
    }

    fn drinkable_water(&self, cell: CellId) -> bool {
        self.water_here(cell) && !self.still_frozen(cell)
    }

    /// The pool underfoot or one step off is ice, so a water step names the brook instead.
    pub(crate) fn frozen_still_beside(&self, cell: CellId) -> bool {
        let frozen = |id: CellId| self.cell(id).still && self.still_frozen(id);
        if frozen(cell) {
            return true;
        }
        let (x, y) = cell_xy(cell);
        Dir::all().into_iter().any(|dir| {
            let (dx, dy) = dir.delta();
            cell_id(x + dx, y + dy).is_some_and(frozen)
        })
    }

    /// The ordinary miss, plus a step when the stock is one cell over.
    pub(crate) fn gather_miss(&self, cell: CellId, material: Material, bare: String) -> String {
        match self.gather_step_cue(cell, material) {
            Some(cue) => format!("{bare}{cue}"),
            None => bare,
        }
    }

    pub(crate) fn cattail_bare(&self, cell: CellId) -> bool {
        self.cattail_stands.contains(&cell)
            && self.pile_count(cell, Material::Cattail) == 0
            && self.ground_ids(cell, Material::Cattail).is_empty()
    }

    /// A raw handful of pale berries in the arms. A cooked one is already food.
    pub(crate) fn raw_pale_id(&self, actor: ActorId) -> Option<ItemId> {
        self.loose_ids(actor, Material::PaleBerry)
            .into_iter()
            .find(|id| self.item(*id).is_some_and(|berry| !berry.cooked))
    }

    /// One cooked handful of pale berries, from the arms before the ground.
    pub(crate) fn cooked_pale_id(&self, actor: ActorId) -> Option<ItemId> {
        let carried = self
            .loose_ids(actor, Material::PaleBerry)
            .into_iter()
            .find(|id| self.item(*id).is_some_and(|berry| berry.cooked));
        if carried.is_some() {
            return carried;
        }
        let cell = self.actor(actor)?.cell;
        self.ground_ids(cell, Material::PaleBerry)
            .into_iter()
            .find(|id| self.item(*id).is_some_and(|berry| berry.cooked))
    }

    pub(crate) fn raw_pale_near(&self, actor: ActorId) -> bool {
        if self.raw_pale_id(actor).is_some() {
            return true;
        }
        let Some(cell) = self.actor(actor).map(|person| person.cell) else {
            return false;
        };
        self.pile_count(cell, Material::PaleBerry) > 0
            || self
                .ground_ids(cell, Material::PaleBerry)
                .iter()
                .any(|id| self.item(*id).is_some_and(|berry| !berry.cooked))
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
        let piles: u32 = [
            Material::Mushroom,
            Material::Berry,
            Material::PaleBerry,
            Material::Egg,
            Material::Cattail,
        ]
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

    pub(crate) fn carrying_wolf_hide(&self, actor: ActorId) -> bool {
        self.loose_ids(actor, Material::Hide)
            .iter()
            .any(|id| self.item(*id).is_some_and(|hide| hide.wolf))
    }

    /// A sewn wolf hide worn on the body. It cuts more cold than a deer cloak,
    /// and the body line says a little wind instead of the open wind.
    pub(crate) fn wearing_wolf_cloak(&self, actor: ActorId) -> bool {
        self.loose_ids(actor, Material::Hide).iter().any(|id| {
            self.item(*id)
                .is_some_and(|hide| hide.worn && hide.sewn && hide.wolf)
        })
    }

    pub(crate) fn carrying_otter_hide(&self, actor: ActorId) -> bool {
        self.loose_ids(actor, Material::Hide)
            .iter()
            .any(|id| self.item(*id).is_some_and(|hide| hide.otter))
    }

    pub(crate) fn carrying_fox_hide(&self, actor: ActorId) -> bool {
        self.loose_ids(actor, Material::Hide)
            .iter()
            .any(|id| self.item(*id).is_some_and(|hide| hide.fox))
    }

    /// A sewn fox hide worn on the body. It takes the open wind down to a
    /// little, and the cold stays at a deer cloak's cut. Rain still soaks it.
    pub(crate) fn wearing_fox_cloak(&self, actor: ActorId) -> bool {
        self.loose_ids(actor, Material::Hide).iter().any(|id| {
            self.item(*id)
                .is_some_and(|hide| hide.worn && hide.sewn && hide.fox)
        })
    }

    /// A sewn hare hide worn on the body, and no heavier hide with it.
    /// A deer cloak worn as well still takes the deer cut.
    pub(crate) fn wearing_only_hare_cloak(&self, actor: ActorId) -> bool {
        self.wearing_hare_cloak(actor)
            && self
                .loose_ids(actor, Material::Hide)
                .iter()
                .all(|id| self.item(*id).is_some_and(|hide| !hide.worn || hide.hare))
    }

    /// A sewn hare hide worn on the body. It is a light cover: a quarter of
    /// the open cold, not half the way a deer cloak does, and the wind stays.
    /// Rain still soaks it.
    pub(crate) fn wearing_hare_cloak(&self, actor: ActorId) -> bool {
        self.loose_ids(actor, Material::Hide).iter().any(|id| {
            self.item(*id)
                .is_some_and(|hide| hide.worn && hide.sewn && hide.hare)
        })
    }

    pub(crate) fn carrying_hare_hide(&self, actor: ActorId) -> bool {
        self.loose_ids(actor, Material::Hide)
            .iter()
            .any(|id| self.item(*id).is_some_and(|hide| hide.hare))
    }

    pub(crate) fn carrying_beaver_hide(&self, actor: ActorId) -> bool {
        self.loose_ids(actor, Material::Hide)
            .iter()
            .any(|id| self.item(*id).is_some_and(|hide| hide.beaver))
    }

    /// A sewn otter hide worn on the body. Rain and the brook wet it less
    /// than a deer cloak, and that wet cold is cut again.
    pub(crate) fn wearing_otter_cloak(&self, actor: ActorId) -> bool {
        self.loose_ids(actor, Material::Hide).iter().any(|id| {
            self.item(*id)
                .is_some_and(|hide| hide.worn && hide.sewn && hide.otter)
        })
    }

    /// A sewn beaver hide worn on the body. Rain and the brook wet it the
    /// way an otter cloak does. The words are its own.
    pub(crate) fn wearing_beaver_cloak(&self, actor: ActorId) -> bool {
        self.loose_ids(actor, Material::Hide).iter().any(|id| {
            self.item(*id)
                .is_some_and(|hide| hide.worn && hide.sewn && hide.beaver)
        })
    }

    /// Otter or beaver. Both halve the brook step, the rain on the clothes,
    /// and the wet cold.
    pub(crate) fn wearing_water_cloak(&self, actor: ActorId) -> bool {
        self.wearing_otter_cloak(actor) || self.wearing_beaver_cloak(actor)
    }

    /// A step into the brook. A water cloak takes half of that wet.
    pub(crate) fn wade_wet(&self, actor: ActorId) -> u32 {
        let wet = self.tuning.wade;
        if self.wearing_water_cloak(actor) {
            wet / 2
        } else {
            wet
        }
    }

    /// The wind a body feels. A wolf cloak, a fox cloak, or a reed mat
    /// underfoot takes the open wind down to a little. Carrying the mat
    /// does not. A fox cloak does not cut the cold further than a deer cloak.
    pub(crate) fn wind_on(&self, actor: ActorId) -> &'static str {
        let cell = self.actor(actor).map(|person| person.cell).unwrap_or(0);
        if self.wind_at(cell) == "in the wind"
            && (self.wearing_wolf_cloak(actor)
                || self.wearing_fox_cloak(actor)
                || self.reed_mat_here(cell))
        {
            "in a little wind"
        } else {
            self.wind_at(cell)
        }
    }

    /// A reed mat lying on this cell. Carrying one does not put it underfoot.
    pub(crate) fn reed_mat_here(&self, cell: CellId) -> bool {
        !self.ground_ids(cell, Material::Mat).is_empty()
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
        self.meat_near(actor, |meat| {
            !meat.cooked && !meat.salted && !meat.smoked && !meat.spoiled
        })
    }

    /// Smoked meat in the arms, else on this cell. It can be eaten cold.
    pub(crate) fn smoked_meat_id(&self, actor: ActorId) -> Option<ItemId> {
        let carried = self
            .loose_ids(actor, Material::Meat)
            .into_iter()
            .find(|id| {
                self.item(*id)
                    .is_some_and(|meat| meat.smoked && !meat.cooked)
            });
        if carried.is_some() {
            return carried;
        }
        let cell = self.actor(actor)?.cell;
        self.ground_ids(cell, Material::Meat)
            .into_iter()
            .find(|id| {
                self.item(*id)
                    .is_some_and(|meat| meat.smoked && !meat.cooked)
            })
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
                self.item(*id).is_some_and(|fish| {
                    !fish.cooked && !fish.salted && !fish.smoked && !fish.spoiled
                })
            });
        if carried.is_some() {
            return carried;
        }
        let cell = self.actor(actor)?.cell;
        self.ground_ids(cell, Material::Fish)
            .into_iter()
            .find(|id| {
                self.item(*id).is_some_and(|fish| {
                    !fish.cooked && !fish.salted && !fish.smoked && !fish.spoiled
                })
            })
    }

    /// A smoked fish in the arms, else one on this cell. It can be eaten cold.
    pub(crate) fn smoked_fish_id(&self, actor: ActorId) -> Option<ItemId> {
        let carried = self
            .loose_ids(actor, Material::Fish)
            .into_iter()
            .find(|id| {
                self.item(*id)
                    .is_some_and(|fish| fish.smoked && !fish.cooked)
            });
        if carried.is_some() {
            return carried;
        }
        let cell = self.actor(actor)?.cell;
        self.ground_ids(cell, Material::Fish)
            .into_iter()
            .find(|id| {
                self.item(*id)
                    .is_some_and(|fish| fish.smoked && !fish.cooked)
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

    /// The fishing net on this brook is no longer set. A catch that was waiting does not land.
    pub(crate) fn clear_set_net(&mut self, cell: CellId) {
        self.set_nets.remove(&cell);
        self.net_catches_at.remove(&cell);
    }

    /// The damp bank, the wet bank, or a cell with mud on it.
    pub(crate) fn damp_earth(&self, cell: CellId) -> bool {
        self.worm_beds.contains(&cell)
            || self.pile_count(cell, Material::Mud) > 0
            || !self.ground_ids(cell, Material::Mud).is_empty()
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

    /// The howl put a wolf on this ground. It stands through `wolf_ticks`
    /// clocks, counting the nightfall tick, and only while it is night.
    /// Tracks are separate and can still be fresh after it leaves.
    pub(crate) fn wolf_standing(&self) -> Option<CellId> {
        let cell = self.wolf_cell?;
        if !self.night() || self.time >= self.wolf_until {
            None
        } else {
            Some(cell)
        }
    }

    /// A fire on or next to the wolf, a brand on its ground, or that smoke.
    pub(crate) fn wolf_threatened(&self) -> bool {
        let Some(cell) = self.wolf_standing() else {
            return false;
        };
        let smoke = self
            .fires
            .values()
            .any(|fire| chebyshev(fire.cell, cell) <= 1);
        let brand = self.items.values().any(|item| {
            item.material == Material::Brand
                && item.role == Role::Loose
                && match item.carrier {
                    Some(id) => self.actor(id).is_some_and(|person| person.cell == cell),
                    None => item.cell == cell,
                }
        });
        smoke || brand
    }

    /// Fire, a brand, or smoke sends the wolf off. The tracks stay.
    /// `actor` is set when a command caused it, so that person does not
    /// also read the sentence in the event list.
    pub(crate) fn scare_wolf(&mut self, actor: Option<ActorId>) -> bool {
        let Some(cell) = self.wolf_standing() else {
            return false;
        };
        if !self.wolf_threatened() {
            return false;
        }
        self.wolf_cell = None;
        self.emit(
            "The wolf flees the fire.",
            Scope::Cell(cell),
            actor.into_iter().collect(),
            vec![cell],
            vec![],
            vec![],
            None,
        );
        true
    }

    /// Someone stepped closer to the wolf. It is gone. The tracks stay.
    pub(crate) fn frighten_wolf(&mut self, origin: CellId, dest: CellId) {
        let Some(cell) = self.wolf_standing() else {
            return;
        };
        if chebyshev(dest, cell) < chebyshev(origin, cell) {
            self.wolf_cell = None;
        }
    }

    /// A successful cast empties that stretch until this time. The next
    /// cast on the step that begins then can take another fish.
    pub(crate) fn brook_is_quiet(&self, cell: CellId) -> bool {
        self.brook_quiet
            .get(&cell)
            .is_some_and(|until| self.time < *until)
    }

    /// One piece of raw meat lying on this cell. Cooked meat is not raw.
    /// Smoked meat is not raw. Spoiled meat is not raw either. It is not food.
    pub(crate) fn raw_meat_at(&self, cell: CellId) -> Option<ItemId> {
        self.ground_ids(cell, Material::Meat)
            .into_iter()
            .find(|id| {
                self.item(*id)
                    .is_some_and(|meat| !meat.cooked && !meat.smoked && !meat.spoiled)
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
        if let Some(canoe) = self.aboard.get(&actor).copied() {
            let still = self
                .item(canoe)
                .is_some_and(|boat| boat.cell == dest && boat.material == Material::Canoe);
            if !still {
                self.aboard.remove(&actor);
            }
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
        assert_eq!(
            world.pile_count(super::cell_id(2, 0).unwrap(), Material::Salt),
            1
        );
        assert_eq!(world.material_tally(Material::Salt), 1);
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
        let _ = raw;
        let still_salt = world.examine(near, ExamineTarget::Salt);
        assert!(
            still_salt.contains(
                "A salt crust lies on the dry margin of the still to the east, in the open."
            ),
            "{still_salt}"
        );
        assert!(!still_salt.contains("carried by you"), "{still_salt}");
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
            .all(|event| event.text != "You eat the salted fish."));
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
mod smoke_fish {
    use super::{Attempt, ExamineTarget, Intent, Material, Role, World};

    fn intent(id: &str, actor: u32, attempt: Attempt) -> Intent {
        Intent {
            command_id: id.into(),
            actor_id: actor,
            attempt,
        }
    }

    #[test]
    fn a_fire_smokes_a_raw_fish_and_the_raw_one_still_spoils() {
        let mut world = World::valley(crate::tuning::Tuning::evening(), 1);
        let near = world.spawn_at("Near", "n", 4, 3).unwrap();
        let seen = world.spawn_at("Seen", "s", 5, 3).unwrap();
        let quiet = world.spawn_at("Quiet", "q", 0, 4).unwrap();
        assert_eq!(
            crate::parse::parse_attempt("smoke fish").unwrap(),
            Attempt::SmokeFish
        );
        assert_eq!(
            crate::parse::parse_attempt("smoke the fish").unwrap(),
            Attempt::SmokeFish
        );
        let cell = world.actor(near).unwrap().cell;
        let carried = world.mint_item(Material::Fish, 0, cell, Some(near), Role::Loose);
        let no_fire = world.step(&[intent("cold", near, Attempt::SmokeFish)]);
        assert_eq!(no_fire.results[0].message, "There is no fire burning here.");
        for id in [near, seen, quiet] {
            assert!(world
                .observe(id)
                .unwrap()
                .events
                .iter()
                .all(|event| event.text != "There is no fire burning here."));
        }
        world.items.remove(&carried);
        world.mint_item(Material::Bark, 0, cell, Some(near), Role::Loose);
        world.mint_item(Material::Branch, 0, cell, Some(near), Role::Loose);
        let lit = world.step(&[intent("light", near, Attempt::Light)]);
        assert!(lit.results[0].success, "{}", lit.results[0].message);
        let none = world.step(&[intent("none", near, Attempt::SmokeFish)]);
        assert_eq!(
            none.results[0].message,
            "There is no raw fish here to smoke."
        );
        for id in [near, seen, quiet] {
            assert!(world
                .observe(id)
                .unwrap()
                .events
                .iter()
                .all(|event| event.text != "There is no raw fish here to smoke."));
        }
        let ground = world.mint_item(Material::Fish, 0, cell, None, Role::Loose);
        let from_ground = world.step(&[intent("ground", near, Attempt::SmokeFish)]);
        assert_eq!(
            from_ground.results[0].message,
            "You smoke a raw fish. It will keep, and it can be eaten cold."
        );
        assert!(world.item(ground).unwrap().smoked);
        assert!(world
            .observe(near)
            .unwrap()
            .text
            .contains("A smoked fish lies here. It can be eaten cold."));
        world.items.remove(&ground);
        let kept = world.mint_item(Material::Fish, 0, cell, Some(near), Role::Loose);
        let yours = "You smoke a raw fish. It will keep, and it can be eaten cold.";
        let smoked = world.step(&[intent("smoke", near, Attempt::SmokeFish)]);
        assert_eq!(smoked.results[0].message, yours);
        assert!(world.item(kept).unwrap().smoked);
        assert!(world.item(kept).unwrap().spoils_at.is_none());
        assert!(!world.item(kept).unwrap().salted);
        assert!(!world.item(kept).unwrap().cooked);
        let arms = world.observe(near).unwrap();
        assert!(
            arms.text.contains("You have a smoked fish."),
            "{}",
            arms.text
        );
        assert!(arms.events.iter().all(|event| {
            event.text != yours && !event.text.contains("Near smokes a raw fish")
        }));
        assert!(world.observe(seen).unwrap().events.iter().any(|event| {
            event.text == "Near smokes a raw fish. It will keep, and it can be eaten cold."
        }));
        assert!(world
            .observe(quiet)
            .unwrap()
            .events
            .iter()
            .all(|event| !event.text.contains("smokes a raw fish")));
        let again = world.step(&[intent("again", near, Attempt::SmokeFish)]);
        assert_eq!(again.results[0].message, "The fish is already smoked.");
        for id in [near, seen, quiet] {
            assert!(world
                .observe(id)
                .unwrap()
                .events
                .iter()
                .all(|event| event.text != "The fish is already smoked."));
        }
        let raw = world.mint_item(Material::Fish, 0, cell, None, Role::Loose);
        let ticks = world.tuning.meat_ticks;
        for _ in 0..ticks {
            world.step(&[]);
        }
        let going = world.observe(near).unwrap().text;
        assert!(going.contains("You have a smoked fish."), "{going}");
        assert!(
            going.contains("A raw fish lies here. It is going bad."),
            "{going}"
        );
        assert!(!going.contains("A salted fish"), "{going}");
        assert!(!going.contains("A cooked fish"), "{going}");
        let examined = world.examine(near, ExamineTarget::Fish);
        assert!(
            examined.contains("A smoked fish is carried by you"),
            "{examined}"
        );
        assert!(examined.contains("It can be eaten cold."), "{examined}");
        assert!(
            examined.contains("A raw fish lies in contact with the ground"),
            "{examined}"
        );
        assert!(!examined.contains("salted fish"), "{examined}");
        assert!(!examined.contains("cooked fish"), "{examined}");
        world.step(&[]);
        let here = world.observe(near).unwrap();
        assert!(here
            .events
            .iter()
            .any(|event| event.text == "A raw fish spoils. It is not food."));
        assert!(
            here.text.contains("You have a smoked fish."),
            "{}",
            here.text
        );
        assert!(
            here.text
                .contains("A spoiled fish lies here. It is not food."),
            "{}",
            here.text
        );
        assert!(world.item(kept).unwrap().smoked);
        assert!(!world.item(kept).unwrap().spoiled);
        assert!(world.item(raw).unwrap().spoiled);
        let eaten = world.step(&[intent("eat", near, Attempt::Eat(Material::Fish))]);
        assert!(
            eaten.results[0]
                .message
                .starts_with("You eat the smoked fish."),
            "{}",
            eaten.results[0].message
        );
        assert!(world
            .observe(seen)
            .unwrap()
            .events
            .iter()
            .any(|event| event.text == "Near eats the smoked fish."));
        assert!(world
            .observe(near)
            .unwrap()
            .events
            .iter()
            .all(|event| event.text != "You eat the smoked fish."));
        assert!(world.observe(near).unwrap().events.iter().all(|event| {
            !event.text.contains("Near eats the smoked") && !event.text.contains("Near smokes")
        }));
        assert!(world
            .observe(quiet)
            .unwrap()
            .events
            .iter()
            .all(|event| !event.text.contains("smoked fish")));
        let refused = world.step(&[intent("bad", near, Attempt::Eat(Material::Fish))]);
        assert_eq!(refused.results[0].message, "The fish has spoiled.");
        assert!(world
            .observe(seen)
            .unwrap()
            .events
            .iter()
            .all(|event| event.text != "The fish has spoiled."));
    }

    #[test]
    fn going_bad_stays_on_the_raw_fish_in_a_mixed_arms_line() {
        let mut world = World::valley(crate::tuning::Tuning::evening(), 1);
        let near = world.spawn_at("Near", "n", 4, 3).unwrap();
        let seen = world.spawn_at("Seen", "s", 5, 3).unwrap();
        let cell = world.actor(near).unwrap().cell;
        let smoked = world.mint_item(Material::Fish, 0, cell, Some(near), Role::Loose);
        if let Some(fish) = world.items.get_mut(&smoked) {
            fish.smoked = true;
            fish.spoils_at = None;
        }
        let raw = world.mint_item(Material::Fish, 0, cell, Some(near), Role::Loose);
        let before = world.observe(near).unwrap().text;
        assert!(
            before.contains("You have a smoked fish, a raw fish."),
            "{before}"
        );
        assert!(!before.contains("going bad"), "{before}");
        let ticks = world.tuning.meat_ticks;
        for _ in 0..ticks {
            world.step(&[]);
        }
        let mixed = world.observe(near).unwrap().text;
        assert!(
            mixed.contains("You have a smoked fish, a raw fish that is going bad."),
            "{mixed}"
        );
        assert!(!mixed.contains(", going bad"), "{mixed}");
        assert!(
            world
                .examine(seen, ExamineTarget::Fish)
                .contains("A smoked fish is carried by Near")
                && world
                    .examine(seen, ExamineTarget::Fish)
                    .contains("A raw fish is carried by Near")
                && world
                    .examine(seen, ExamineTarget::Fish)
                    .contains("It is going bad."),
            "{}",
            world.examine(seen, ExamineTarget::Fish)
        );
        world.items.remove(&smoked);
        let only = world.observe(near).unwrap().text;
        assert!(
            only.contains("You have a raw fish that is going bad."),
            "{only}"
        );
        assert!(!only.contains(", going bad"), "{only}");
        assert!(world.item(raw).unwrap().spoils_at.is_some());
        assert!(!world.item(raw).unwrap().smoked);
    }
}

#[cfg(test)]
mod smoke_meat {
    use super::{Attempt, ExamineTarget, Intent, Material, Role, World};

    fn intent(id: &str, actor: u32, attempt: Attempt) -> Intent {
        Intent {
            command_id: id.into(),
            actor_id: actor,
            attempt,
        }
    }

    #[test]
    fn a_fire_smokes_raw_meat_and_the_raw_portion_still_spoils() {
        let mut world = World::valley(crate::tuning::Tuning::evening(), 1);
        let near = world.spawn_at("Near", "n", 4, 3).unwrap();
        let seen = world.spawn_at("Seen", "s", 5, 3).unwrap();
        let quiet = world.spawn_at("Quiet", "q", 0, 4).unwrap();
        assert_eq!(
            crate::parse::parse_attempt("smoke meat").unwrap(),
            Attempt::SmokeMeat
        );
        assert_eq!(
            crate::parse::parse_attempt("smoke the meat").unwrap(),
            Attempt::SmokeMeat
        );
        let cell = world.actor(near).unwrap().cell;
        let carried = world.mint_item(Material::Meat, 0, cell, Some(near), Role::Loose);
        let no_fire = world.step(&[intent("cold", near, Attempt::SmokeMeat)]);
        assert_eq!(no_fire.results[0].message, "There is no fire burning here.");
        for id in [near, seen, quiet] {
            assert!(world
                .observe(id)
                .unwrap()
                .events
                .iter()
                .all(|event| event.text != "There is no fire burning here."));
        }
        world.items.remove(&carried);
        world.mint_item(Material::Bark, 0, cell, Some(near), Role::Loose);
        world.mint_item(Material::Branch, 0, cell, Some(near), Role::Loose);
        world.step(&[intent("light", near, Attempt::Light)]);
        let none = world.step(&[intent("none", near, Attempt::SmokeMeat)]);
        assert_eq!(
            none.results[0].message,
            "There is no raw meat here to smoke."
        );
        for id in [near, seen, quiet] {
            assert!(world
                .observe(id)
                .unwrap()
                .events
                .iter()
                .all(|event| event.text != "There is no raw meat here to smoke."));
        }
        let ground = world.mint_item(Material::Meat, 0, cell, None, Role::Loose);
        let from_ground = world.step(&[intent("ground", near, Attempt::SmokeMeat)]);
        assert_eq!(
            from_ground.results[0].message,
            "You smoke a portion of raw meat. It will keep, and it can be eaten cold."
        );
        assert!(world.item(ground).unwrap().smoked);
        assert!(world.item(ground).unwrap().spoils_at.is_none());
        assert!(world
            .observe(near)
            .unwrap()
            .text
            .contains("A portion of smoked meat lies here. It can be eaten cold."));
        world.items.remove(&ground);
        let yours = "You smoke a portion of raw meat. It will keep, and it can be eaten cold.";
        let kept = world.mint_item(Material::Meat, 0, cell, Some(near), Role::Loose);
        let smoked = world.step(&[intent("smoke", near, Attempt::SmokeMeat)]);
        assert_eq!(smoked.results[0].message, yours);
        assert!(world.item(kept).unwrap().smoked);
        assert!(world.item(kept).unwrap().spoils_at.is_none());
        let arms = world.observe(near).unwrap();
        assert!(
            arms.text.contains("You have a portion of smoked meat."),
            "{}",
            arms.text
        );
        assert!(arms.events.iter().all(|event| {
            event.text != yours && !event.text.contains("Near smokes a portion of raw meat")
        }));
        assert!(world.observe(seen).unwrap().events.iter().any(|event| {
            event.text
                == "Near smokes a portion of raw meat. It will keep, and it can be eaten cold."
        }));
        assert!(world
            .observe(quiet)
            .unwrap()
            .events
            .iter()
            .all(|event| !event.text.contains("smokes a portion of raw meat")));
        let again = world.step(&[intent("again", near, Attempt::SmokeMeat)]);
        assert_eq!(again.results[0].message, "The meat is already smoked.");
        for id in [near, seen, quiet] {
            assert!(world
                .observe(id)
                .unwrap()
                .events
                .iter()
                .all(|event| event.text != "The meat is already smoked."));
        }
        let raw = world.mint_item(Material::Meat, 0, cell, Some(near), Role::Loose);
        let before = world.observe(near).unwrap().text;
        assert!(
            before.contains("a portion of smoked meat") && before.contains("a portion of raw meat"),
            "{before}"
        );
        assert!(!before.contains("going bad"), "{before}");
        let ticks = world.tuning.meat_ticks;
        for _ in 0..ticks {
            world.step(&[]);
        }
        let mixed = world.observe(near).unwrap().text;
        assert!(
            mixed.contains("a portion of raw meat that is going bad, a portion of smoked meat"),
            "{mixed}"
        );
        assert!(!mixed.contains(", going bad"), "{mixed}");
        let examined = world.examine(seen, ExamineTarget::Meat);
        assert!(
            examined.contains("A portion of smoked meat is carried by Near"),
            "{examined}"
        );
        assert!(examined.contains("It can be eaten cold."), "{examined}");
        assert!(
            examined.contains("A portion of raw meat is carried by Near"),
            "{examined}"
        );
        assert!(examined.contains("It is going bad."), "{examined}");
        assert!(
            !examined.contains("salted meat") && !examined.contains("cooked meat"),
            "{examined}"
        );
        assert_eq!(
            world.examine(quiet, ExamineTarget::Meat),
            "You see no meat from here."
        );
        world.step(&[]);
        let spoiled = world.observe(near).unwrap();
        assert!(spoiled
            .events
            .iter()
            .any(|event| event.text == "A portion of raw meat spoils. It is not food."));
        assert!(world
            .observe(seen)
            .unwrap()
            .events
            .iter()
            .any(|event| { event.text == "A portion of raw meat spoils. It is not food." }));
        assert!(world
            .observe(quiet)
            .unwrap()
            .events
            .iter()
            .all(|event| event.text != "A portion of raw meat spoils. It is not food."));
        assert!(world.item(kept).unwrap().smoked);
        assert!(!world.item(kept).unwrap().spoiled);
        assert!(world.item(raw).unwrap().spoiled);
        let eaten = world.step(&[intent("eat", near, Attempt::Eat(Material::Meat))]);
        assert!(
            eaten.results[0]
                .message
                .starts_with("You eat the smoked meat."),
            "{}",
            eaten.results[0].message
        );
        assert!(world
            .observe(seen)
            .unwrap()
            .events
            .iter()
            .any(|event| event.text == "Near eats the smoked meat."));
        assert!(world
            .observe(near)
            .unwrap()
            .events
            .iter()
            .all(|event| event.text != "You eat the smoked meat."));
        assert!(world
            .observe(near)
            .unwrap()
            .events
            .iter()
            .all(|event| !event.text.contains("Near eats the smoked")
                && !event.text.contains("Near smokes")));
        let refused = world.step(&[intent("bad", near, Attempt::Eat(Material::Meat))]);
        assert_eq!(refused.results[0].message, "The meat has spoiled.");
        assert!(world
            .observe(seen)
            .unwrap()
            .events
            .iter()
            .all(|event| event.text != "The meat has spoiled."));
    }
}

#[cfg(test)]
mod salt_lick {
    use super::{Attempt, Intent, Material, Role, World};

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
        assert_eq!(
            world.pile_count(super::cell_id(2, 0).unwrap(), Material::Salt),
            1
        );
        assert_eq!(world.material_tally(Material::Salt), 3);
    }

    #[test]
    fn the_still_margin_holds_a_salt_crust_that_can_be_gathered() {
        let mut world = World::valley(crate::tuning::Tuning::evening(), 1);
        let near = world.spawn_at("Near", "n", 4, 0).unwrap();
        let seen = world.spawn_at("Seen", "s", 3, 0).unwrap();
        let quiet = world.spawn_at("Quiet", "q", 6, 4).unwrap();
        let missed = world.step(&[intent("none", near, Attempt::Gather(Material::Salt))]);
        assert_eq!(
            missed.results[0].message,
            "There is no salt here to gather."
        );
        assert!(world
            .observe(near)
            .unwrap()
            .events
            .iter()
            .any(|event| { event.text == "There is no salt here to gather." }));
        assert!(world
            .observe(seen)
            .unwrap()
            .events
            .iter()
            .all(|event| { event.text != "There is no salt here to gather." }));
        assert!(world
            .observe(quiet)
            .unwrap()
            .events
            .iter()
            .all(|event| { event.text != "There is no salt here to gather." }));

        let _ = world.step(&[intent("west", near, Attempt::Move(super::Dir::West))]);
        let _ = world.step(&[intent("west2", near, Attempt::Move(super::Dir::West))]);
        assert_eq!(super::cell_xy(world.actor(near).unwrap().cell), (2, 0));
        let here = world.observe(near).unwrap().text;
        assert!(
            here.contains("A salt crust lies on the dry margin of the still."),
            "{here}"
        );
        assert!(!here.contains("The still margin is dry."), "{here}");
        let from_seen = world.observe(seen).unwrap().text;
        assert!(
            from_seen.contains("A salt crust lies on the dry margin of the still to the west."),
            "{from_seen}"
        );
        assert!(!world.observe(quiet).unwrap().text.contains("still margin"));

        let gathered = world.step(&[intent("gather", near, Attempt::Gather(Material::Salt))]);
        assert_eq!(gathered.results[0].message, "You gather a salt crust.");
        assert!(world.observe(near).unwrap().events.iter().all(|event| {
            event.text != "You gather a salt crust." && event.text != "Near gathers a salt crust."
        }));
        assert!(world
            .observe(seen)
            .unwrap()
            .events
            .iter()
            .any(|event| { event.text == "Near gathers a salt crust." }));
        assert!(world
            .observe(quiet)
            .unwrap()
            .events
            .iter()
            .all(|event| { !event.text.contains("salt") }));
        let bare = world.observe(near).unwrap().text;
        assert!(bare.contains("The still margin is dry."), "{bare}");
        assert!(
            !bare.contains("A salt crust lies on the dry margin"),
            "{bare}"
        );
        assert!(bare.contains("a salt crust"), "{bare}");
        let held = world.examine(near, super::ExamineTarget::Salt);
        assert!(held.contains("It can salt raw meat."), "{held}");

        world.mint_item(
            Material::Meat,
            0,
            world.actor(near).unwrap().cell,
            None,
            Role::Loose,
        );
        let salted = world.step(&[intent("salt", near, Attempt::SaltMeat)]);
        assert_eq!(
            salted.results[0].message,
            "You salt a portion of raw meat. It will keep, and it can be eaten cold."
        );
        assert!(world.observe(near).unwrap().events.iter().all(|event| {
            !event.text.contains("You salt") && !event.text.contains("Near salts")
        }));
        assert!(world.observe(seen).unwrap().events.iter().any(|event| {
            event.text
                == "Near salts a portion of raw meat. It will keep, and it can be eaten cold."
        }));

        let back_at = *world.salt_returns_at.values().next().unwrap();
        assert_eq!(world.tuning.salt_ticks, 24);
        assert!(back_at > world.time);
        while world.time < back_at {
            world.step(&[]);
        }
        assert!(world
            .observe(near)
            .unwrap()
            .text
            .contains("The still margin is dry."));
        world.step(&[]);
        let formed = "A salt crust forms on the still margin.";
        let back = world.observe(near).unwrap();
        assert!(
            back.text
                .contains("A salt crust lies on the dry margin of the still."),
            "{}",
            back.text
        );
        assert!(back.events.iter().any(|event| event.text == formed));
        assert!(world
            .observe(seen)
            .unwrap()
            .events
            .iter()
            .any(|event| event.text == formed));
        assert!(world
            .observe(quiet)
            .unwrap()
            .events
            .iter()
            .all(|event| event.text != formed));
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
        assert!(here.contains("You hear the bees buzzing."), "{here}");
        let from_west = world.observe(seen).unwrap().text;
        assert!(
            from_west.contains("A bees' nest hangs in a tree on the slope to the east."),
            "{from_west}"
        );
        assert!(
            from_west.contains("You hear the bees buzzing to the east."),
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
            .observe(near)
            .unwrap()
            .events
            .iter()
            .any(|event| event.text == "You are not carrying honey."));
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
        assert!(!empty.contains("buzzing"), "{empty}");
        assert!(empty.contains("a honeycomb"), "{empty}");
        let west_empty = world.observe(seen).unwrap().text;
        assert!(
            west_empty.contains("The bees' nest in the tree on the slope to the east is empty."),
            "{west_empty}"
        );
        assert!(!west_empty.contains("buzzing"), "{west_empty}");
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
            .any(|event| event.text == "The hunger eases."));
        assert!(world.observe(near).unwrap().events.iter().all(|event| {
            event.text != "You eat the honeycomb." && !event.text.contains("Near eats")
        }));
        assert!(world
            .observe(seen)
            .unwrap()
            .events
            .iter()
            .any(|event| event.text == "Near eats the honeycomb."));
        assert!(world
            .observe(seen)
            .unwrap()
            .events
            .iter()
            .all(
                |event| event.text != "The hunger eases." && event.text != "You eat the honeycomb."
            ));
        assert!(world
            .observe(quiet)
            .unwrap()
            .events
            .iter()
            .all(|event| !event.text.contains("honeycomb")));
        assert_eq!(world.material_tally(Material::Honey), 0);
    }

    #[test]
    fn two_cells_off_the_nest_you_see_the_tree_and_do_not_hear_the_bees() {
        let mut world = World::valley(crate::tuning::Tuning::evening(), 1);
        let far = world.spawn_at("Far", "f", 4, 0).unwrap();
        let quiet = world.spawn_at("Quiet", "q", 5, 3).unwrap();
        let text = world.observe(far).unwrap().text;
        assert!(
            text.contains("A bees' nest hangs in a tree on the slope to the east."),
            "{text}"
        );
        assert!(!text.contains("buzzing"), "{text}");
        let missed = world.observe(quiet).unwrap().text;
        assert!(
            !missed.contains("bees") && !missed.contains("buzzing"),
            "{missed}"
        );
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
        let before = world.observe(near).unwrap().text;
        assert!(before.contains("You hear the bees buzzing."), "{before}");
        assert!(world
            .observe(seen)
            .unwrap()
            .text
            .contains("You hear the bees buzzing to the east."));
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
        assert!(!here.contains("buzzing"), "{here}");
        assert!(here.contains("You are well."), "{here}");
        let west = world.observe(seen).unwrap().text;
        assert!(west.contains("Smoke hangs in it."), "{west}");
        assert!(!west.contains("buzzing"), "{west}");
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
        assert!(!empty.contains("buzzing"), "{empty}");
        let west_empty = world.observe(seen).unwrap().text;
        assert!(
            west_empty.contains("The bees' nest in the tree on the slope to the east is empty."),
            "{west_empty}"
        );
        assert!(!west_empty.contains("buzzing"), "{west_empty}");
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
            back.text.contains("You hear the bees buzzing."),
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
        assert!(cooked_look.events.iter().all(|event| {
            event.text != "You cook the eggs over the fire."
                && !event.text.starts_with("Near cooks")
        }));
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
            .any(|event| event.text == "The hunger eases."));
        assert!(fed.events.iter().all(|event| {
            event.text != "You eat the cooked eggs." && !event.text.contains("Near eats")
        }));
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
        let quiet_look = world.observe(quiet).unwrap().text;
        assert!(
            quiet_look.contains("Dry moss grows in the shade"),
            "{quiet_look}"
        );
        assert!(!quiet_look.contains("damp bank"), "{quiet_look}");
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
        assert!(stung.contains("Your skin stings."), "{stung}");
        assert!(!stung.contains("bare skin"), "{stung}");
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
        let quiet_moss = world.examine(quiet, ExamineTarget::Moss);
        assert!(
            quiet_moss.contains("Dry moss grows in the shade"),
            "{quiet_moss}"
        );
        assert!(!quiet_moss.contains("damp bank"), "{quiet_moss}");
        assert!(!quiet_moss.contains("carried"), "{quiet_moss}");
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
        let quiet_after = world.examine(quiet, ExamineTarget::Moss);
        assert!(
            quiet_after.contains("Dry moss grows in the shade"),
            "{quiet_after}"
        );
        assert!(!quiet_after.contains("damp bank"), "{quiet_after}");
        assert!(!quiet_after.contains("carried"), "{quiet_after}");
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
        let quiet_moss = world.examine(quiet, super::ExamineTarget::Moss);
        assert!(
            quiet_moss.contains("Dry moss grows in the shade"),
            "{quiet_moss}"
        );
        assert!(!quiet_moss.contains("damp bank"), "{quiet_moss}");
        assert!(!quiet_moss.contains("carried"), "{quiet_moss}");
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

    #[test]
    fn dry_moss_in_the_shade_lights_a_fire_and_wet_moss_does_not() {
        let mut world = World::valley(crate::tuning::Tuning::evening(), 1);
        let rock = super::cell_id(6, 1).unwrap();
        let near = world.spawn_at("Near", "n", 6, 1).unwrap();
        let seen = world.spawn_at("Seen", "s", 3, 3).unwrap();
        let quiet = world.spawn_at("Quiet", "q", 5, 3).unwrap();
        let bark = world.pile_count(rock, Material::Bark);
        let here = world.observe(near).unwrap().text;
        assert!(here.contains("Dry moss grows in the shade here."), "{here}");
        assert!(!here.contains("damp bank"), "{here}");
        let named = world.examine(near, ExamineTarget::Moss);
        assert!(
            named.contains("Dry moss grows in the shade here, under the rock overhang.")
                && named.contains("It can light a fire."),
            "{named}"
        );
        assert!(!named.contains("damp bank"), "{named}");
        let distant = world.observe(quiet).unwrap().text;
        assert!(
            distant.contains("Dry moss grows in the shade to the"),
            "{distant}"
        );
        assert!(!distant.contains("damp bank"), "{distant}");
        let gathered = world.step(&[intent("gather", near, Attempt::Gather(Material::Moss))]);
        assert_eq!(gathered.results[0].message, "You gather a handful of moss.");
        let taken = world.observe(near).unwrap();
        assert!(
            taken.text.contains("The shade here has no moss."),
            "{}",
            taken.text
        );
        assert!(
            taken.text.contains("You have a handful of moss."),
            "{}",
            taken.text
        );
        assert!(taken
            .events
            .iter()
            .all(|event| !event.text.contains("gathers")));
        assert!(world
            .observe(quiet)
            .unwrap()
            .events
            .iter()
            .any(|event| { event.text == "Near gathers a handful of moss." }));
        assert!(world
            .observe(seen)
            .unwrap()
            .events
            .iter()
            .all(|event| !event.text.contains("gathers")));
        let yours =
            "You light a fire with the moss from the arms and a dry branch from the pile at the feet.";
        let theirs =
            "Near lights a fire with the moss from the arms and a dry branch from the pile at the feet.";
        let lit = world.step(&[intent("light", near, Attempt::Light)]);
        assert_eq!(lit.results[0].message, yours);
        assert_eq!(world.pile_count(rock, Material::Bark), bark);
        let after = world.observe(near).unwrap();
        assert!(
            after.text.contains("A small fire burns here."),
            "{}",
            after.text
        );
        assert!(after.text.contains("A pile of bark lies here.") || after.text.contains("bark"),);
        assert!(after.events.iter().all(|event| event.text != theirs));
        assert!(world
            .observe(quiet)
            .unwrap()
            .events
            .iter()
            .any(|event| event.text == theirs));
        assert!(world
            .observe(seen)
            .unwrap()
            .events
            .iter()
            .all(|event| event.text != theirs));

        let mut wet = World::valley(crate::tuning::Tuning::evening(), 1);
        let floor = super::cell_id(4, 3).unwrap();
        let near = wet.spawn_at("Near", "n", 4, 3).unwrap();
        let seen = wet.spawn_at("Seen", "s", 3, 3).unwrap();
        let quiet = wet.spawn_at("Quiet", "q", 5, 3).unwrap();
        wet.mint_item(Material::Moss, 1000, floor, Some(near), super::Role::Loose);
        wet.mint_item(Material::Branch, 0, floor, Some(near), super::Role::Loose);
        let refused = wet.step(&[intent("wet", near, Attempt::Light)]);
        assert_eq!(refused.results[0].message, "The moss is too damp to catch.");
        for person in [near, seen, quiet] {
            assert!(wet
                .observe(person)
                .unwrap()
                .events
                .iter()
                .all(|event| { event.text != "The moss is too damp to catch." }));
        }
        assert!(wet.fire_at(floor).is_none());
        wet.mint_item(Material::Bark, 0, floor, Some(near), super::Role::Loose);
        let bark_still = wet.step(&[intent("bark", near, Attempt::Light)]);
        assert_eq!(
            bark_still.results[0].message,
            "You light a fire with the bark and the branch from the arms."
        );
        assert_eq!(wet.loose_ids(near, Material::Moss).len(), 1);

        let mut rain = World::valley(crate::tuning::Tuning::evening(), 1);
        rain.raining = true;
        let open = super::cell_id(4, 4).unwrap();
        let near = rain.spawn_at("Near", "n", 4, 4).unwrap();
        let seen = rain.spawn_at("Seen", "s", 4, 2).unwrap();
        rain.mint_item(Material::Moss, 0, open, Some(near), super::Role::Loose);
        rain.mint_item(Material::Branch, 0, open, Some(near), super::Role::Loose);
        let soaked = rain.step(&[intent("rain", near, Attempt::Light)]);
        assert_eq!(
            soaked.results[0].message,
            "The rain puts the moss out before it catches."
        );
        assert!(rain
            .observe(near)
            .unwrap()
            .events
            .iter()
            .all(|event| { event.text != "The rain puts the moss out before it catches." }));
        assert!(rain
            .observe(seen)
            .unwrap()
            .events
            .iter()
            .all(|event| { event.text != "The rain puts the moss out before it catches." }));
        assert_eq!(rain.loose_ids(near, Material::Moss).len(), 1);
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
            "You need a dry branch and hide strips or willow cordage."
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

    #[test]
    fn willow_cordage_binds_a_bow_when_no_hide_strips_are_held() {
        let mut world = World::valley(crate::tuning::Tuning::evening(), 1);
        let near = world.spawn_at("Near", "n", 4, 3).unwrap();
        let seen = world.spawn_at("Seen", "s", 3, 3).unwrap();
        let quiet = world.spawn_at("Quiet", "q", 0, 0).unwrap();
        let cell = world.actor(near).unwrap().cell;
        world.mint_item(Material::Branch, 0, cell, Some(near), Role::Loose);
        let none = world.step(&[intent("none", near, Attempt::CraftBow)]);
        assert_eq!(
            none.results[0].message,
            "You need hide strips or willow cordage."
        );
        assert!(world
            .observe(near)
            .unwrap()
            .events
            .iter()
            .any(|event| event.text == "You need hide strips or willow cordage."));
        assert!(world
            .observe(seen)
            .unwrap()
            .events
            .iter()
            .all(|event| event.text != "You need hide strips or willow cordage."));
        assert!(world
            .observe(quiet)
            .unwrap()
            .events
            .iter()
            .all(|event| event.text != "You need hide strips or willow cordage."));

        world.mint_item(Material::Cord, 0, cell, Some(near), Role::Loose);
        world.mint_item(Material::Willow, 0, cell, Some(near), Role::Loose);
        let prefer = world.step(&[intent("prefer", near, Attempt::CraftBow)]);
        assert_eq!(
            prefer.results[0].message,
            "You craft a bow from a dry branch and hide strips."
        );
        assert!(world.loose_ids(near, Material::Cord).is_empty());
        assert_eq!(world.loose_ids(near, Material::Willow).len(), 1);
        let hide = world.examine(near, ExamineTarget::Bow);
        assert!(
            hide.contains("It is a dry branch bound with hide strips."),
            "{hide}"
        );
        let hide_bow = world.loose_ids(near, Material::Bow)[0];
        world.items.remove(&hide_bow);

        world.mint_item(Material::Branch, 0, cell, Some(near), Role::Loose);
        let crafted = world.step(&[intent("craft", near, Attempt::CraftBow)]);
        let yours = "You craft a bow from a dry branch and willow cordage.";
        let witnessed = "Near crafts a bow from a dry branch and willow cordage.";
        assert_eq!(crafted.results[0].message, yours);
        assert!(world.loose_ids(near, Material::Willow).is_empty());
        assert!(world
            .observe(near)
            .unwrap()
            .events
            .iter()
            .all(|event| { event.text != yours && !event.text.contains("crafts a bow") }));
        assert!(world
            .observe(seen)
            .unwrap()
            .events
            .iter()
            .any(|event| event.text == witnessed));
        assert!(world
            .observe(quiet)
            .unwrap()
            .events
            .iter()
            .all(|event| !event.text.contains("willow cordage")));
        let examined = world.examine(near, ExamineTarget::Bow);
        assert!(
            examined.contains("It is a dry branch bound with willow cordage."),
            "{examined}"
        );
        world.step(&[intent("drop", near, Attempt::Drop(Material::Bow))]);
        let here = world.observe(near).unwrap().text;
        assert!(
            here.contains("A bow lies here. It is a dry branch bound with willow cordage."),
            "{here}"
        );
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
    use super::{Attempt, Dir, ExamineTarget, Material, Role, World};

    fn intent(id: &str, actor: u32, attempt: Attempt) -> super::Intent {
        super::Intent {
            command_id: id.into(),
            actor_id: actor,
            attempt,
        }
    }

    #[test]
    fn a_step_through_brush_is_heard_out_of_sight_and_not_every_tick() {
        let mut world = World::valley(crate::tuning::Tuning::evening(), 1);
        let near = world.spawn_at("Near", "n", 2, 2).unwrap();
        let seen = world.spawn_at("Seen", "s", 3, 3).unwrap();
        let quiet = world.spawn_at("Quiet", "q", 0, 0).unwrap();
        for _ in 0..3 {
            world.step(&[]);
        }
        assert_ne!(world.deer_cell(), Some((5, 2)));
        assert!(world.observe(near).unwrap().events.iter().all(|event| {
            !event.text.contains("branches breaking") && !event.text.contains("wood cracking")
        }));
        world.step(&[]);
        assert_eq!(world.deer_cell(), Some((5, 2)));
        let heard = "You hear branches breaking to the east.";
        let near_hear = world.observe(near).unwrap();
        assert_eq!(
            near_hear
                .events
                .iter()
                .filter(|event| event.text == heard)
                .count(),
            1
        );
        assert!(near_hear.events.iter().all(|event| {
            !event.text.contains("A branch snapped") && !event.text.contains("Branches break")
        }));
        let seen_view = world.observe(seen).unwrap();
        assert!(seen_view
            .events
            .iter()
            .any(|event| { event.text.contains("A branch snapped under the deer") }));
        assert!(seen_view
            .events
            .iter()
            .all(|event| !event.text.contains("You hear branches breaking")));
        assert!(world.observe(quiet).unwrap().events.iter().all(|event| {
            !event.text.contains("branches") && !event.text.contains("deer steps")
        }));
        world.step(&[]);
        assert_eq!(world.deer_cell(), Some((5, 1)));
        assert_eq!(
            world
                .observe(near)
                .unwrap()
                .events
                .iter()
                .filter(|event| event.text == heard)
                .count(),
            1
        );

        let walked = world.step(&[intent("log", near, Attempt::Move(Dir::East))]);
        assert_eq!(walked.results[0].message, "You walk east.");
        let crack = "You hear wood cracking to the south and east.";
        assert!(world
            .observe(near)
            .unwrap()
            .events
            .iter()
            .all(|event| { event.text != crack && !event.text.contains("Wood cracks") }));
        assert!(world
            .observe(seen)
            .unwrap()
            .events
            .iter()
            .any(|event| { event.text == "Near walks east." }));
        assert!(world
            .observe(seen)
            .unwrap()
            .events
            .iter()
            .all(|event| event.text != crack));
        assert!(world
            .observe(quiet)
            .unwrap()
            .events
            .iter()
            .any(|event| event.text == crack));
        let open = world.step(&[intent("open", near, Attempt::Move(Dir::East))]);
        assert_eq!(open.results[0].message, "You walk east.");
        assert_eq!(
            world
                .observe(quiet)
                .unwrap()
                .events
                .iter()
                .filter(|event| event.text == crack)
                .count(),
            1
        );
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
mod ochre_return {
    use super::{Attempt, ExamineTarget, Material, World};

    fn intent(id: &str, actor: u32, attempt: Attempt) -> super::Intent {
        super::Intent {
            command_id: id.into(),
            actor_id: actor,
            attempt,
        }
    }

    #[test]
    fn the_slope_has_no_ochre_until_a_lump_lies_there_again() {
        let mut world = World::valley(crate::tuning::Tuning::evening(), 1);
        let near = world.spawn_at("Near", "n", 3, 0).unwrap();
        let seen = world.spawn_at("Seen", "s", 3, 1).unwrap();
        let quiet = world.spawn_at("Quiet", "q", 6, 4).unwrap();
        let here = world.observe(near).unwrap().text;
        assert!(here.contains("A lump of ochre lies here."), "{here}");
        let south = world.observe(seen).unwrap().text;
        assert!(south.contains("North: slope, a lump of ochre"), "{south}");
        assert!(
            !south.contains("The slope to the north has no ochre."),
            "{south}"
        );
        assert!(!world.observe(quiet).unwrap().text.contains("ochre"));
        let gathered = world.step(&[intent("gather", near, Attempt::Gather(Material::Ochre))]);
        assert_eq!(gathered.results[0].message, "You gather a lump of ochre.");
        let slope = world.actor(near).unwrap().cell;
        let due = world.ochre_returns_at.get(&slope).copied();
        let smeared = world.step(&[intent("smear", near, Attempt::SmearOchre)]);
        assert_eq!(
            smeared.results[0].message,
            "You smear red ochre on your skin."
        );
        assert_eq!(world.ochre_returns_at.get(&slope).copied(), due);
        let empty = world.observe(near).unwrap().text;
        assert!(empty.contains("The slope has no ochre."), "{empty}");
        assert!(!empty.contains("A lump of ochre lies here."), "{empty}");
        assert_eq!(
            world.examine(near, ExamineTarget::Ochre),
            "The slope here has no ochre."
        );
        let south = world.observe(seen).unwrap().text;
        assert!(
            south.contains("The slope to the north has no ochre."),
            "{south}"
        );
        assert!(!south.contains("a lump of ochre"), "{south}");
        assert_eq!(
            world.examine(seen, ExamineTarget::Ochre),
            "The slope to the north has no ochre."
        );
        assert!(!world.observe(quiet).unwrap().text.contains("ochre"));
        assert_eq!(
            world.examine(quiet, ExamineTarget::Ochre),
            "You see no ochre from here."
        );
        let ticks = world.tuning.ochre_ticks;
        for _ in 0..ticks - 2 {
            world.step(&[]);
        }
        assert!(world
            .observe(near)
            .unwrap()
            .text
            .contains("The slope has no ochre."));
        world.step(&[]);
        let back = "A lump of ochre lies on the slope again.";
        let again = world.observe(near).unwrap();
        assert!(
            again.text.contains("A lump of ochre lies here."),
            "{}",
            again.text
        );
        assert!(
            !again.text.contains("The slope has no ochre."),
            "{}",
            again.text
        );
        assert!(again.events.iter().any(|event| event.text == back));
        let seen_back = world.observe(seen).unwrap();
        assert!(
            seen_back.text.contains("North: slope, a lump of ochre"),
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
        assert!(!world.observe(quiet).unwrap().text.contains("ochre"));
        let second = world.step(&[intent("second", near, Attempt::Gather(Material::Ochre))]);
        assert_eq!(second.results[0].message, "You gather a lump of ochre.");
        let taken = world.observe(near).unwrap().text;
        assert!(taken.contains("The slope has no ochre."), "{taken}");
        assert!(!taken.contains("A lump of ochre lies here."), "{taken}");
        assert!(world.ochre_returns_at.contains_key(&slope));
    }
}

#[cfg(test)]
mod ochre_arms {
    use super::{ExamineTarget, Material, World};

    #[test]
    fn two_lumps_of_ochre_stay_lumps() {
        let mut world = World::valley(crate::tuning::Tuning::evening(), 1);
        let near = world.spawn_at("Near", "n", 3, 4).unwrap();
        let seen = world.spawn_at("Seen", "s", 4, 4).unwrap();
        let quiet = world.spawn_at("Quiet", "q", 6, 4).unwrap();
        let cell = world.actor(near).unwrap().cell;
        world.mint_item(Material::Ochre, 0, cell, Some(near), super::Role::Loose);
        let one = world.observe(near).unwrap().text;
        assert!(one.contains("You have a lump of ochre."), "{one}");
        assert!(!one.contains("2 ochre"), "{one}");
        assert!(!one.contains("carried by you (dry)"), "{one}");
        assert_eq!(
            world.examine(near, ExamineTarget::Ochre),
            "A lump of ochre is carried by you, in the open."
        );
        assert_eq!(
            world.examine(seen, ExamineTarget::Ochre),
            "A lump of ochre is carried by Near, in the open."
        );
        assert_eq!(
            world.examine(quiet, ExamineTarget::Ochre),
            "You see no ochre from here."
        );
        world.mint_item(Material::Ochre, 0, cell, Some(near), super::Role::Loose);
        let two = world.observe(near).unwrap().text;
        assert!(two.contains("You have two lumps of ochre."), "{two}");
        assert!(!two.contains("2 ochre"), "{two}");
        assert!(!two.contains("carried by you (dry)"), "{two}");
        assert_eq!(
            world.examine(near, ExamineTarget::Ochre),
            "Two lumps of ochre are carried by you, in the open."
        );
        assert_eq!(
            world.examine(seen, ExamineTarget::Ochre),
            "Two lumps of ochre are carried by Near, in the open."
        );
        assert_eq!(
            world.examine(quiet, ExamineTarget::Ochre),
            "You see no ochre from here."
        );
    }
}

#[cfg(test)]
mod dugout {
    use super::{cell_id, cell_xy, Attempt, Dir, ExamineTarget, Material, Role, World};

    fn intent(id: &str, actor: u32, attempt: Attempt) -> super::Intent {
        super::Intent {
            command_id: id.into(),
            actor_id: actor,
            attempt,
        }
    }

    #[test]
    fn one_person_drags_a_charred_log_and_a_raw_log_names_that() {
        let mut raw = World::valley(crate::tuning::Tuning::evening(), 1);
        let near = raw.spawn_at("Near", "n", 3, 2).unwrap();
        let seen = raw.spawn_at("Seen", "s", 2, 3).unwrap();
        raw.step(&[intent("grip", near, Attempt::Grab)]);
        let stay = raw.step(&[intent("pull", near, Attempt::Drag(Dir::East))]);
        assert_eq!(
            stay.results[0].message,
            "The log does not move for you alone. A charred log is light enough."
        );
        assert!(raw.observe(near).unwrap().events.iter().all(|event| {
            !event.text.contains("light enough") && !event.text.contains("pulls on the log alone")
        }));
        assert!(raw
            .observe(seen)
            .unwrap()
            .events
            .iter()
            .any(|event| { event.text == "Near pulls on the log alone. It does not move." }));
        assert!(raw
            .observe(seen)
            .unwrap()
            .events
            .iter()
            .all(|event| !event.text.contains("light enough")));
        raw.step(&[]);
        assert_eq!(raw.log_cell(), Some((3, 2)));

        let mut world = World::valley(crate::tuning::Tuning::evening(), 1);
        let near = world.spawn_at("Near", "n", 3, 2).unwrap();
        let seen = world.spawn_at("Seen", "s", 2, 3).unwrap();
        let quiet = world.spawn_at("Quiet", "q", 6, 4).unwrap();
        let cell = world.actor(near).unwrap().cell;
        world.charred_logs.insert(cell);
        world.step(&[intent("grip", near, Attempt::Grab)]);
        assert!(world.log_charred_at(cell));
        let set = world.step(&[intent("pull", near, Attempt::Drag(Dir::West))]);
        assert_eq!(
            set.results[0].message,
            "You set your weight to drag the log west."
        );
        assert!(world
            .observe(near)
            .unwrap()
            .events
            .iter()
            .all(|event| { event.text != "You set your weight to drag the log west." }));
        world.step(&[]);
        assert_eq!(world.log_cell(), Some((2, 2)));
        assert_eq!(world.actor_cell(near), Some((2, 2)));
        assert!(world.log_charred_at(world.actor(near).unwrap().cell));
        let shift = "The log shifts west. Near moves with it, still in contact with it.";
        assert!(world
            .observe(seen)
            .unwrap()
            .events
            .iter()
            .any(|event| event.text == shift));
        assert!(world
            .observe(quiet)
            .unwrap()
            .events
            .iter()
            .all(|event| !event.text.contains("shifts")));
    }

    #[test]
    fn the_heavy_log_at_the_floor_becomes_a_canoe_that_floats() {
        let mut world = World::valley(crate::tuning::Tuning::evening(), 1);
        let near = world.spawn_at("Near", "n", 3, 2).unwrap();
        let seen = world.spawn_at("Seen", "s", 2, 3).unwrap();
        let quiet = world.spawn_at("Quiet", "q", 6, 4).unwrap();
        let here = world.observe(near).unwrap();
        assert!(
            here.text.contains("The heavy log lies here."),
            "{}",
            here.text
        );
        assert_eq!(world.actor_cell(near), Some((3, 2)));
        let cold = world.step(&[intent("cold", near, Attempt::HollowLog)]);
        assert_eq!(cold.results[0].message, "There is no fire burning here.");
        assert!(world
            .observe(seen)
            .unwrap()
            .events
            .iter()
            .all(|event| !event.text.contains("fire burning")));
        let cell = world.actor(near).unwrap().cell;
        world.mint_item(Material::Bark, 0, cell, Some(near), Role::Loose);
        world.mint_item(Material::Branch, 0, cell, Some(near), Role::Loose);
        let lit = world.step(&[intent("light", near, Attempt::Light)]);
        assert!(lit.results[0].success, "{}", lit.results[0].message);
        let hollow = world.step(&[intent("hollow", near, Attempt::HollowLog)]);
        assert_eq!(
            hollow.results[0].message,
            "You hollow the heavy log with the fire."
        );
        let charred = world.observe(near).unwrap();
        assert!(charred.events.iter().all(|event| {
            event.text != "You hollow the heavy log with the fire."
                && event.text != "Near hollows the heavy log with the fire."
        }));
        assert!(
            charred
                .text
                .contains("The heavy log lies here. The fire has charred it."),
            "{}",
            charred.text
        );
        assert!(world
            .observe(seen)
            .unwrap()
            .events
            .iter()
            .any(|event| { event.text == "Near hollows the heavy log with the fire." }));
        assert!(
            world
                .observe(seen)
                .unwrap()
                .text
                .contains("The heavy log to the north and east is charred."),
            "{}",
            world.observe(seen).unwrap().text
        );
        let bare = world.step(&[intent("bare", near, Attempt::CarveCanoe)]);
        assert_eq!(
            bare.results[0].message,
            "You need a flint flake or an iron knife to carve the canoe."
        );
        assert!(world
            .observe(seen)
            .unwrap()
            .events
            .iter()
            .all(|event| !event.text.contains("flint flake")));
        world.mint_item(Material::Flake, 0, cell, Some(near), Role::Loose);
        let carved = world.step(&[intent("carve", near, Attempt::CarveCanoe)]);
        assert_eq!(
            carved.results[0].message,
            "You carve the charred log into a dugout canoe."
        );
        let dugout = world.observe(near).unwrap();
        assert!(dugout.events.iter().all(|event| {
            event.text != "You carve the charred log into a dugout canoe."
                && event.text != "Near carves the charred log into a dugout canoe."
        }));
        assert!(
            dugout.text.contains("A dugout canoe lies here."),
            "{}",
            dugout.text
        );
        assert!(!dugout.text.contains("The heavy log"), "{}", dugout.text);
        assert_eq!(
            world.examine(near, ExamineTarget::Canoe),
            "A dugout canoe lies here. It was hollowed from the heavy log."
        );
        let hauled = world.step(&[intent("haul", near, Attempt::Drag(Dir::West))]);
        assert_eq!(hauled.results[0].message, "You drag the dugout canoe west.");
        assert_eq!(world.actor_cell(near), Some((2, 2)));
        assert!(world.observe(near).unwrap().events.iter().all(|event| {
            !event.text.contains("You drag the dugout canoe")
                && !event.text.contains("Near drags the dugout canoe")
        }));
        assert!(world
            .observe(seen)
            .unwrap()
            .events
            .iter()
            .any(|event| { event.text == "Near drags the dugout canoe west." }));
        let water = world.step(&[intent("water", near, Attempt::Drag(Dir::West))]);
        assert_eq!(water.results[0].message, "Launch the canoe from the bank.");
        assert!(world
            .observe(seen)
            .unwrap()
            .events
            .iter()
            .all(|event| !event.text.contains("from the bank")));
        let launched = world.step(&[intent("launch", near, Attempt::Launch)]);
        assert_eq!(
            launched.results[0].message,
            "You launch the dugout canoe into the brook."
        );
        assert_eq!(world.actor_cell(near), Some((2, 2)));
        let bank = world.observe(near).unwrap();
        assert!(bank.events.iter().all(|event| {
            !event.text.contains("You launch") && !event.text.contains("Near launches")
        }));
        assert!(
            world
                .observe(seen)
                .unwrap()
                .text
                .contains("A dugout canoe floats in the brook to the north and west."),
            "{}",
            world.observe(seen).unwrap().text
        );
        let boarded = world.step(&[intent("board", near, Attempt::Board)]);
        assert_eq!(boarded.results[0].message, "You board the dugout canoe.");
        assert_eq!(world.actor_cell(near), Some((1, 2)));
        let afloat = world.observe(near).unwrap();
        assert!(
            afloat
                .text
                .contains("You are aboard the dugout canoe in the brook."),
            "{}",
            afloat.text
        );
        assert!(!afloat.text.contains("You are on the brook"));
        assert!(!afloat
            .text
            .contains("A dugout canoe floats here in the brook."));
        assert!(afloat.text.contains("The water here is raw."));
        assert!(
            world
                .observe(seen)
                .unwrap()
                .text
                .contains("A dugout canoe floats in the brook to the north and west."),
            "{}",
            world.observe(seen).unwrap().text
        );
        assert!(afloat.events.iter().all(|event| {
            !event.text.contains("You board") && !event.text.contains("Near boards")
        }));
        assert!(world
            .observe(seen)
            .unwrap()
            .events
            .iter()
            .any(|event| event.text == "Near boards the dugout canoe."));
        let far = world.observe(quiet).unwrap();
        assert!(far.events.iter().all(|event| {
            !event.text.contains("canoe")
                && !event.text.contains("hollow")
                && !event.text.contains("charred")
        }));
        assert!(!far.text.contains("canoe"));
        assert!(!far.text.contains("charred"));
        assert_eq!(world.material_tally(Material::Canoe), 1);
        assert_eq!(world.material_tally(Material::Log), 0);
    }

    #[test]
    fn paddling_stays_in_the_brook_and_disembark_steps_onto_the_bank() {
        let mut world = World::valley(crate::tuning::Tuning::evening(), 1);
        let near = world.spawn_at("Near", "n", 2, 2).unwrap();
        let seen = world.spawn_at("Seen", "s", 3, 3).unwrap();
        let quiet = world.spawn_at("Quiet", "q", 6, 4).unwrap();
        world.mint_item(
            Material::Canoe,
            0,
            cell_id(1, 2).unwrap(),
            None,
            Role::Loose,
        );
        assert_eq!(
            crate::parse::parse_attempt("paddle").unwrap(),
            Attempt::Paddle(None)
        );
        assert_eq!(
            crate::parse::parse_attempt("paddle north").unwrap(),
            Attempt::Paddle(Some(Dir::North))
        );
        assert_eq!(
            crate::parse::parse_attempt("leave canoe").unwrap(),
            Attempt::Disembark
        );
        let boarded = world.step(&[intent("board", near, Attempt::Board)]);
        assert_eq!(boarded.results[0].message, "You board the dugout canoe.");
        let walked = world.step(&[intent("walk", near, Attempt::Move(Dir::North))]);
        assert_eq!(
            walked.results[0].message,
            "You are aboard the canoe. Disembark first."
        );
        assert!(!walked.results[0].success);
        assert_eq!(world.actor_cell(near), Some((1, 2)));
        assert!(world
            .observe(near)
            .unwrap()
            .events
            .iter()
            .all(|event| { event.text != "You are aboard the canoe. Disembark first." }));
        assert!(world
            .observe(seen)
            .unwrap()
            .events
            .iter()
            .all(|event| { !event.text.contains("Disembark") }));
        let paddled = world.step(&[intent("paddle", near, Attempt::Paddle(None))]);
        assert_eq!(
            paddled.results[0].message,
            "You paddle the dugout canoe north."
        );
        assert_eq!(world.actor_cell(near), Some((1, 1)));
        let boat = world
            .items
            .values()
            .find(|item| item.material == Material::Canoe)
            .unwrap();
        assert_eq!(cell_xy(boat.cell), (1, 1));
        assert!(world.aboard.contains_key(&near));
        let look = world.observe(near).unwrap();
        assert!(look
            .text
            .contains("You are aboard the dugout canoe in the brook."));
        assert!(!look.text.contains("You are on the brook"));
        assert!(!look
            .text
            .contains("A dugout canoe floats here in the brook."));
        assert!(look.events.iter().all(|event| {
            event.text != "You paddle the dugout canoe north."
                && event.text != "Near paddles the dugout canoe north."
        }));
        let witness = world.observe(seen).unwrap();
        assert!(witness
            .events
            .iter()
            .any(|event| event.text == "Near paddles the dugout canoe north."));
        assert!(
            witness
                .text
                .contains("A dugout canoe floats in the brook to the north and west."),
            "{}",
            witness.text
        );
        let banked = world.step(&[intent("east", near, Attempt::Paddle(Some(Dir::East)))]);
        assert_eq!(banked.results[0].message, "The canoe stays in the brook.");
        assert_eq!(world.actor_cell(near), Some((1, 1)));
        assert!(world
            .observe(near)
            .unwrap()
            .events
            .iter()
            .all(|event| { event.text != "The canoe stays in the brook." }));
        assert!(world
            .observe(seen)
            .unwrap()
            .events
            .iter()
            .all(|event| { event.text != "The canoe stays in the brook." }));
        let left = world.step(&[intent("leave", near, Attempt::Disembark)]);
        assert_eq!(left.results[0].message, "You leave the dugout canoe.");
        assert_eq!(world.actor_cell(near), Some((1, 0)));
        assert!(!world.aboard.contains_key(&near));
        let boat = world
            .items
            .values()
            .find(|item| item.material == Material::Canoe)
            .unwrap();
        assert_eq!(cell_xy(boat.cell), (1, 1));
        let ashore = world.observe(near).unwrap();
        assert!(
            ashore
                .text
                .contains("A dugout canoe floats in the brook to the south."),
            "{}",
            ashore.text
        );
        assert!(!ashore.text.contains("You are aboard"));
        assert!(ashore.events.iter().all(|event| {
            event.text != "You leave the dugout canoe."
                && event.text != "Near leaves the dugout canoe."
        }));
        assert!(world
            .observe(seen)
            .unwrap()
            .events
            .iter()
            .any(|event| event.text == "Near leaves the dugout canoe."));
        let far = world.observe(quiet).unwrap();
        assert!(far.events.iter().all(|event| {
            !event.text.contains("paddle") && !event.text.contains("leaves the dugout")
        }));
    }

    #[test]
    fn a_spear_from_the_dugout_takes_a_fish_and_the_bank_still_does() {
        let mut world = World::valley(crate::tuning::Tuning::evening(), 1);
        let near = world.spawn_at("Near", "n", 2, 2).unwrap();
        let seen = world.spawn_at("Seen", "s", 3, 3).unwrap();
        let heard = world.spawn_at("Heard", "h", 4, 4).unwrap();
        let quiet = world.spawn_at("Quiet", "q", 6, 4).unwrap();
        let bank = world.spawn_at("Bank", "b", 2, 1).unwrap();
        world.mint_item(
            Material::Canoe,
            0,
            cell_id(1, 2).unwrap(),
            None,
            Role::Loose,
        );
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
        let boarded = world.step(&[intent("board", near, Attempt::Board)]);
        assert_eq!(boarded.results[0].message, "You board the dugout canoe.");
        assert!(world.aboard.contains_key(&near));
        world.step(&[intent("drop", near, Attempt::Drop(Material::Spear))]);
        let bare = world.step(&[intent("bare", near, Attempt::Fish)]);
        assert_eq!(bare.results[0].message, "You are not carrying a spear.");
        assert!(!bare.results[0].success);
        assert!(world
            .observe(seen)
            .unwrap()
            .events
            .iter()
            .all(|event| { !event.text.contains("not carrying a spear") }));
        world.step(&[intent("spear", near, Attempt::Gather(Material::Spear))]);
        let dry = world.spawn_at("Dry", "d", 4, 0).unwrap();
        let inland = world.step(&[intent("inland", dry, Attempt::Fish)]);
        assert_eq!(inland.results[0].message, "The brook is not here.");
        assert!(world
            .observe(dry)
            .unwrap()
            .events
            .iter()
            .any(|event| { event.text == "The brook is not here." }));
        assert!(world
            .observe(seen)
            .unwrap()
            .events
            .iter()
            .all(|event| { !event.text.contains("brook is not here") }));
        assert_eq!(
            crate::parse::parse_attempt("spear fish").unwrap(),
            Attempt::Fish
        );
        let cast = world.step(&[intent("fish", near, Attempt::Fish)]);
        assert_eq!(cast.results[0].message, "You spear a fish from the brook.");
        assert_eq!(world.actor_cell(near), Some((1, 2)));
        assert!(world.aboard.contains_key(&near));
        let actor = world.observe(near).unwrap();
        assert!(actor
            .events
            .iter()
            .any(|event| event.text == "You spear a fish from the brook."));
        assert!(actor
            .events
            .iter()
            .all(|event| event.text != "Near spears a fish from the brook."
                && !event.text.contains("splash")));
        assert!(world
            .observe(seen)
            .unwrap()
            .events
            .iter()
            .any(|event| event.text == "Near spears a fish from the brook."));
        assert!(world
            .observe(seen)
            .unwrap()
            .events
            .iter()
            .all(|event| { !event.text.contains("splash") }));
        assert!(world
            .observe(heard)
            .unwrap()
            .events
            .iter()
            .any(|event| event.text == "You hear a splash to the north and west."));
        assert!(world
            .observe(heard)
            .unwrap()
            .events
            .iter()
            .all(|event| { !event.text.contains("spears a fish") }));
        assert!(world
            .observe(quiet)
            .unwrap()
            .events
            .iter()
            .all(|event| { !event.text.contains("fish") && !event.text.contains("splash") }));
        let again = world.step(&[intent("again", near, Attempt::Fish)]);
        assert_eq!(
            again.results[0].message,
            "The fish are gone from this stretch of the brook."
        );
        assert!(world
            .observe(seen)
            .unwrap()
            .events
            .iter()
            .all(|event| { !event.text.contains("fish are gone") }));
        let other = world.mint_item(
            Material::Spear,
            0,
            world.actor(bank).unwrap().cell,
            Some(bank),
            Role::Loose,
        );
        {
            let point = world.items.get_mut(&other).unwrap();
            point.hardened = true;
            point.flint_tip = true;
        }
        assert!(!world.aboard.contains_key(&bank));
        let from_bank = world.step(&[intent("bank", bank, Attempt::Fish)]);
        assert_eq!(
            from_bank.results[0].message,
            "You spear a fish from the brook."
        );
        assert_eq!(world.actor_cell(bank), Some((2, 1)));
        assert_eq!(world.material_tally(Material::Fish), 2);
    }

    #[test]
    fn beaching_puts_the_dugout_on_the_bank_and_ashore_if_you_are_aboard() {
        let mut world = World::valley(crate::tuning::Tuning::evening(), 1);
        let near = world.spawn_at("Near", "n", 2, 2).unwrap();
        let seen = world.spawn_at("Seen", "s", 3, 3).unwrap();
        let quiet = world.spawn_at("Quiet", "q", 6, 4).unwrap();
        world.mint_item(
            Material::Canoe,
            0,
            cell_id(1, 2).unwrap(),
            None,
            Role::Loose,
        );
        assert_eq!(
            crate::parse::parse_attempt("beach").unwrap(),
            Attempt::Beach
        );
        assert_eq!(
            crate::parse::parse_attempt("drag canoe").unwrap(),
            Attempt::Beach
        );
        let hauled = world.step(&[intent("haul", near, Attempt::Beach)]);
        assert_eq!(hauled.results[0].message, "You beach the dugout canoe.");
        assert_eq!(world.actor_cell(near), Some((2, 2)));
        let boat = world
            .items
            .values()
            .find(|item| item.material == Material::Canoe)
            .unwrap();
        assert_eq!(cell_xy(boat.cell), (2, 2));
        let ashore = world.observe(near).unwrap();
        assert!(
            ashore.text.contains("A dugout canoe lies here."),
            "{}",
            ashore.text
        );
        assert!(!ashore.text.contains("floats"));
        assert!(ashore.events.iter().all(|event| {
            event.text != "You beach the dugout canoe."
                && event.text != "Near beaches the dugout canoe."
        }));
        let witness = world.observe(seen).unwrap();
        assert!(witness
            .events
            .iter()
            .any(|event| event.text == "Near beaches the dugout canoe."));
        assert!(
            witness
                .text
                .contains("A dugout canoe lies to the north and west."),
            "{}",
            witness.text
        );
        assert!(!witness.text.contains("floats in the brook"));
        let launched = world.step(&[intent("launch", near, Attempt::Launch)]);
        assert_eq!(
            launched.results[0].message,
            "You launch the dugout canoe into the brook."
        );
        assert!(world
            .observe(seen)
            .unwrap()
            .text
            .contains("A dugout canoe floats in the brook to the north and west."));
        let boarded = world.step(&[intent("board", near, Attempt::Board)]);
        assert_eq!(boarded.results[0].message, "You board the dugout canoe.");
        assert_eq!(world.actor_cell(near), Some((1, 2)));
        let beached = world.step(&[intent("beach", near, Attempt::Beach)]);
        assert_eq!(beached.results[0].message, "You beach the dugout canoe.");
        assert_eq!(world.actor_cell(near), Some((1, 3)));
        assert!(!world.aboard.contains_key(&near));
        let boat = world
            .items
            .values()
            .find(|item| item.material == Material::Canoe)
            .unwrap();
        assert_eq!(cell_xy(boat.cell), (1, 3));
        let land = world.observe(near).unwrap();
        assert!(
            land.text.contains("A dugout canoe lies here."),
            "{}",
            land.text
        );
        assert!(!land.text.contains("floats"));
        assert!(!land.text.contains("You are aboard"));
        assert!(land.events.iter().all(|event| {
            event.text != "You beach the dugout canoe."
                && event.text != "Near beaches the dugout canoe."
        }));
        let far_look = world.observe(seen).unwrap();
        assert!(
            far_look.text.contains("A dugout canoe lies to the west."),
            "{}",
            far_look.text
        );
        assert!(!far_look.text.contains("floats in the brook"));
        let again = world.step(&[intent("again", near, Attempt::Beach)]);
        assert_eq!(again.results[0].message, "The canoe is already on land.");
        let told = world.observe(near).unwrap();
        assert!(
            told.text.contains("A dugout canoe lies here."),
            "{}",
            told.text
        );
        assert!(told
            .events
            .iter()
            .all(|event| event.text != "The canoe is already on land."));
        assert!(told.events.iter().all(|event| {
            event.text != "You beach the dugout canoe."
                && event.text != "Near beaches the dugout canoe."
        }));
        assert!(world
            .observe(seen)
            .unwrap()
            .events
            .iter()
            .all(|event| { event.text != "The canoe is already on land." }));
        let away = world.step(&[intent("away", near, Attempt::Move(Dir::East))]);
        assert!(away.results[0].success, "{}", away.results[0].message);
        let missing = world.step(&[intent("none", near, Attempt::Beach)]);
        assert_eq!(missing.results[0].message, "There is no dugout canoe here.");
        assert!(world
            .observe(seen)
            .unwrap()
            .events
            .iter()
            .all(|event| { !event.text.contains("no dugout canoe") }));
        assert!(world
            .observe(quiet)
            .unwrap()
            .events
            .iter()
            .all(|event| { !event.text.contains("beach") && !event.text.contains("canoe") }));
    }

    #[test]
    fn pine_resin_pitches_the_dugout_and_the_brook_stays_off_the_rider() {
        let mut world = World::valley(crate::tuning::Tuning::evening(), 1);
        let near = world.spawn_at("Near", "n", 2, 2).unwrap();
        let seen = world.spawn_at("Seen", "s", 3, 3).unwrap();
        let quiet = world.spawn_at("Quiet", "q", 6, 4).unwrap();
        assert_eq!(
            crate::parse::parse_attempt("seal canoe").unwrap(),
            Attempt::SealCanoe
        );
        assert_eq!(
            crate::parse::parse_attempt("pitch the canoe").unwrap(),
            Attempt::SealCanoe
        );
        assert_eq!(
            crate::parse::parse_attempt("waterproof dugout").unwrap(),
            Attempt::SealCanoe
        );
        let missing = world.step(&[intent("none", near, Attempt::SealCanoe)]);
        assert_eq!(missing.results[0].message, "There is no dugout canoe here.");
        assert!(world
            .observe(near)
            .unwrap()
            .events
            .iter()
            .all(|event| { event.text != "There is no dugout canoe here." }));
        assert!(world
            .observe(seen)
            .unwrap()
            .events
            .iter()
            .all(|event| { event.text != "There is no dugout canoe here." }));
        world.mint_item(
            Material::Canoe,
            0,
            cell_id(2, 2).unwrap(),
            None,
            Role::Loose,
        );
        let bare = world.step(&[intent("bare", near, Attempt::SealCanoe)]);
        assert_eq!(bare.results[0].message, "You have no pine resin.");
        assert!(world
            .observe(near)
            .unwrap()
            .events
            .iter()
            .all(|event| event.text != "You have no pine resin."));
        assert!(world
            .observe(seen)
            .unwrap()
            .events
            .iter()
            .all(|event| event.text != "You have no pine resin."));
        let before = world.observe(near).unwrap();
        assert!(
            before.text.contains("A dugout canoe lies here."),
            "{}",
            before.text
        );
        assert!(!before.text.contains("pitched"));
        world.mint_item(
            Material::Resin,
            0,
            cell_id(2, 2).unwrap(),
            Some(near),
            Role::Loose,
        );
        world.mint_item(
            Material::Resin,
            0,
            cell_id(2, 2).unwrap(),
            Some(near),
            Role::Loose,
        );
        assert!(world
            .observe(near)
            .unwrap()
            .allowed_actions
            .iter()
            .any(|action| action == "seal canoe"));
        let sealed = world.step(&[intent("seal", near, Attempt::SealCanoe)]);
        assert_eq!(
            sealed.results[0].message,
            "You seal the dugout canoe with pine resin."
        );
        assert_eq!(world.loose_ids(near, Material::Resin).len(), 1);
        assert!(world
            .items
            .values()
            .any(|item| item.material == Material::Canoe && item.pitched));
        let after = world.observe(near).unwrap();
        assert!(
            after.text.contains("A pitched dugout canoe lies here."),
            "{}",
            after.text
        );
        assert!(after.events.iter().all(|event| {
            !event.text.contains("seals the dugout") && !event.text.contains("You seal")
        }));
        let examined = world.examine(near, ExamineTarget::Canoe);
        assert!(
            examined.contains("A pitched dugout canoe lies here. Pine resin seals it."),
            "{examined}"
        );
        let witness = world.observe(seen).unwrap();
        assert!(witness
            .events
            .iter()
            .any(|event| event.text == "Near seals the dugout canoe with pine resin."));
        assert!(
            witness
                .text
                .contains("A pitched dugout canoe lies to the north and west."),
            "{}",
            witness.text
        );
        assert!(world
            .observe(quiet)
            .unwrap()
            .events
            .iter()
            .all(|event| { !event.text.contains("seals") && !event.text.contains("pitched") }));
        let again = world.step(&[intent("again", near, Attempt::SealCanoe)]);
        assert_eq!(again.results[0].message, "The canoe is already pitched.");
        assert_eq!(world.loose_ids(near, Material::Resin).len(), 1);
        assert!(world
            .observe(near)
            .unwrap()
            .events
            .iter()
            .all(|event| { event.text != "The canoe is already pitched." }));
        assert!(world
            .observe(seen)
            .unwrap()
            .events
            .iter()
            .all(|event| { event.text != "The canoe is already pitched." }));
        let launched = world.step(&[intent("launch", near, Attempt::Launch)]);
        assert!(
            launched.results[0].success,
            "{}",
            launched.results[0].message
        );
        let boarded = world.step(&[intent("board", near, Attempt::Board)]);
        assert_eq!(boarded.results[0].message, "You board the dugout canoe.");
        let afloat = world.observe(near).unwrap();
        assert!(
            afloat
                .text
                .contains("You are aboard the pitched dugout canoe in the brook."),
            "{}",
            afloat.text
        );
        let ashore = world.step(&[intent("ashore", near, Attempt::Beach)]);
        assert!(ashore.results[0].success, "{}", ashore.results[0].message);
        if let Some(boat) = world
            .items
            .values_mut()
            .find(|item| item.material == Material::Canoe)
        {
            boat.pitched = false;
        }
        let relaunched = world.step(&[intent("relaunch", near, Attempt::Launch)]);
        assert!(
            relaunched.results[0].success,
            "{}",
            relaunched.results[0].message
        );
        let back = world.step(&[intent("back", near, Attempt::Board)]);
        assert_eq!(back.results[0].message, "You board the dugout canoe.");
        let open = world.observe(near).unwrap();
        assert!(
            open.text
                .contains("You are aboard the dugout canoe in the brook."),
            "{}",
            open.text
        );
        assert!(!open.text.contains("pitched"));
        let aboard = world.step(&[intent("aboard", near, Attempt::SealCanoe)]);
        assert_eq!(
            aboard.results[0].message,
            "You seal the dugout canoe with pine resin."
        );
        assert!(world.loose_ids(near, Material::Resin).is_empty());
        let sealed_aboard = world.observe(near).unwrap();
        assert!(
            sealed_aboard
                .text
                .contains("You are aboard the pitched dugout canoe in the brook."),
            "{}",
            sealed_aboard.text
        );
        assert!(sealed_aboard.events.iter().all(|event| {
            !event.text.contains("seals the dugout") && !event.text.contains("You seal")
        }));
        let wade = world.spawn_at("Wade", "w", 0, 2).unwrap();
        if let Some(actor) = world.actors.get_mut(&near) {
            actor.wetness = 0;
        }
        if let Some(actor) = world.actors.get_mut(&wade) {
            actor.wetness = 0;
        }
        while world.time < 16 {
            world.step(&[]);
        }
        if let Some(actor) = world.actors.get_mut(&near) {
            actor.wetness = 0;
        }
        if let Some(actor) = world.actors.get_mut(&wade) {
            actor.wetness = 0;
        }
        world.step(&[]);
        world.step(&[]);
        let rider = world.actor(near).unwrap().wetness;
        let soaked = world.actor(wade).unwrap().wetness;
        assert!(soaked > rider, "pitched rider {rider}, brook body {soaked}");
        assert_eq!(soaked - rider, world.tuning.brook_rain * 2);
        assert!(world.raining);
        let rider_look = world.observe(near).unwrap();
        assert!(
            rider_look
                .text
                .contains("You are aboard the pitched dugout canoe in the brook."),
            "{}",
            rider_look.text
        );
    }
}

#[cfg(test)]
mod reed_mat {
    use super::{Attempt, Dir, ExamineTarget, Material, World};

    fn intent(id: &str, actor: u32, attempt: Attempt) -> super::Intent {
        super::Intent {
            command_id: id.into(),
            actor_id: actor,
            attempt,
        }
    }

    #[test]
    fn two_bundles_weave_a_mat_and_rest_works_on_it_in_the_open() {
        let mut world = World::valley(crate::tuning::Tuning::evening(), 1);
        let near = world.spawn_at("Near", "n", 0, 3).unwrap();
        let seen = world.spawn_at("Seen", "s", 1, 3).unwrap();
        let quiet = world.spawn_at("Quiet", "q", 6, 4).unwrap();
        let open = world.observe(near).unwrap();
        assert!(open.text.contains("You are in the wind."), "{}", open.text);
        let refused = world.step(&[intent("open", near, Attempt::Rest)]);
        assert_eq!(
            refused.results[0].message,
            "The wind will not let you rest."
        );
        assert!(world
            .observe(seen)
            .unwrap()
            .events
            .iter()
            .all(|event| !event.text.contains("will not let you rest")));
        world.step(&[intent("one", near, Attempt::Gather(Material::Reed))]);
        world.step(&[intent("east", near, Attempt::Move(Dir::East))]);
        let short = world.step(&[intent("short", near, Attempt::WeaveMat)]);
        assert_eq!(
            short.results[0].message,
            "You need two bundles of reeds to weave a mat."
        );
        assert!(world
            .observe(seen)
            .unwrap()
            .events
            .iter()
            .all(|event| !event.text.contains("two bundles")));
        world.step(&[intent("back", near, Attempt::Move(Dir::West))]);
        world.step(&[intent("two", near, Attempt::Gather(Material::Reed))]);
        let woven = world.step(&[intent("weave", near, Attempt::WeaveMat)]);
        assert_eq!(woven.results[0].message, "You weave a reed mat.");
        let after = world.observe(near).unwrap();
        assert!(after
            .events
            .iter()
            .all(|event| event.text != "You weave a reed mat."));
        assert!(after.text.contains("You have a reed mat"), "{}", after.text);
        assert!(world
            .observe(seen)
            .unwrap()
            .events
            .iter()
            .any(|event| event.text == "Near weaves a reed mat."));
        assert_eq!(
            world.examine(near, ExamineTarget::Mat),
            "A reed mat is carried by you, in the open. It is woven from reeds."
        );
        let carried = world.step(&[intent("carry", near, Attempt::Rest)]);
        assert_eq!(
            carried.results[0].message,
            "The wind will not let you rest."
        );
        let set = world.step(&[intent("set", near, Attempt::SetMat)]);
        assert_eq!(set.results[0].message, "You set a reed mat.");
        let down = world.observe(near).unwrap();
        assert!(down
            .events
            .iter()
            .all(|event| event.text != "You set a reed mat."));
        assert!(down.text.contains("A reed mat lies here."), "{}", down.text);
        assert!(
            down.text.contains("You are in a little wind."),
            "{}",
            down.text
        );
        assert!(down.text.contains("in the open"), "{}", down.text);
        assert!(!down.text.contains("You have a reed mat"), "{}", down.text);
        assert!(down.allowed_actions.iter().any(|action| action == "rest"));
        let west = world.observe(seen).unwrap();
        assert!(west
            .events
            .iter()
            .any(|event| event.text == "Near sets a reed mat."));
        assert!(
            west.text.contains("A reed mat lies to the west."),
            "{}",
            west.text
        );
        assert_eq!(
            world.examine(near, ExamineTarget::Mat),
            "A reed mat lies here. It is woven from reeds."
        );
        world.actors.get_mut(&near).unwrap().energy = world.tuning.tiring_at;
        let rested = world.step(&[intent("rest", near, Attempt::Rest)]);
        assert_eq!(
            rested.results[0].message,
            "You rest on the reed mat. You are rested."
        );
        let lay = world.observe(near).unwrap();
        assert!(lay.events.iter().all(|event| {
            event.text != "You rest on the reed mat." && event.text != "Near rests on the reed mat."
        }));
        assert!(lay.text.contains("You are rested."), "{}", lay.text);
        assert!(world
            .observe(seen)
            .unwrap()
            .events
            .iter()
            .any(|event| event.text == "Near rests on the reed mat."));
        assert!(world
            .observe(quiet)
            .unwrap()
            .events
            .iter()
            .all(|event| { !event.text.contains("mat") && !event.text.contains("rest") }));
        assert!(!world.observe(quiet).unwrap().text.contains("mat"));
        let gathered = world.step(&[intent("gather", near, Attempt::Gather(Material::Mat))]);
        assert_eq!(gathered.results[0].message, "You gather a reed mat.");
        let up = world.observe(near).unwrap();
        assert!(up.text.contains("You are in the wind."), "{}", up.text);
        assert!(up
            .events
            .iter()
            .all(|event| event.text != "Near gathers a reed mat."));
        let dropped = world.step(&[intent("drop", near, Attempt::Drop(Material::Mat))]);
        assert_eq!(
            dropped.results[0].message,
            "You drop a reed mat. It is in contact with the ground."
        );
        let again = world.observe(near).unwrap();
        assert!(
            again.text.contains("A reed mat lies here."),
            "{}",
            again.text
        );
        assert!(
            again.text.contains("You are in a little wind."),
            "{}",
            again.text
        );
        assert!(again.events.iter().all(|event| {
            !event.text.contains("You drop a reed mat")
                && !event.text.contains("Near drops a reed mat")
        }));
        world.actors.get_mut(&near).unwrap().energy = world.tuning.tiring_at;
        let second = world.step(&[intent("rest-drop", near, Attempt::Rest)]);
        assert_eq!(
            second.results[0].message,
            "You rest on the reed mat. You are rested."
        );
        assert!(world
            .observe(seen)
            .unwrap()
            .events
            .iter()
            .any(|event| event.text == "Near drops a reed mat. It is in contact with the ground."));
        assert_eq!(world.material_tally(Material::Mat), 1);
    }
}

#[cfg(test)]
mod set_net {
    use super::{Attempt, Dir, ExamineTarget, Material, World};

    fn intent(id: &str, actor: u32, attempt: Attempt) -> super::Intent {
        super::Intent {
            command_id: id.into(),
            actor_id: actor,
            attempt,
        }
    }

    #[test]
    fn a_net_set_in_the_brook_catches_a_fish_and_can_be_gathered() {
        let mut world = World::valley(crate::tuning::Tuning::evening(), 1);
        let near = world.spawn_at("Near", "n", 0, 3).unwrap();
        let seen = world.spawn_at("Seen", "s", 0, 4).unwrap();
        let heard = world.spawn_at("Heard", "h", 3, 2).unwrap();
        let quiet = world.spawn_at("Quiet", "q", 6, 4).unwrap();
        for (id, _) in [("one", 1), ("two", 2), ("three", 3)] {
            world.step(&[intent(id, near, Attempt::Gather(Material::Reed))]);
        }
        let woven = world.step(&[intent("weave", near, Attempt::WeaveNet)]);
        assert_eq!(woven.results[0].message, "You weave a fishing net.");
        let ashore = world.step(&[intent("ashore", near, Attempt::Move(Dir::South))]);
        assert_eq!(ashore.results[0].message, "You walk south.");
        let bank = world.step(&[intent("bank", near, Attempt::SetNet)]);
        assert_eq!(
            bank.results[0].message,
            "You cannot set a net out of the brook."
        );
        assert!(world
            .observe(seen)
            .unwrap()
            .events
            .iter()
            .all(|event| !event.text.contains("out of the brook")));
        let wade = world.step(&[intent("wade", near, Attempt::Move(Dir::North))]);
        assert_eq!(wade.results[0].message, "You wade north.");
        assert_eq!(world.actor_cell(near), Some((0, 3)));
        let set = world.step(&[intent("set", near, Attempt::SetNet)]);
        assert_eq!(
            set.results[0].message,
            "You set a fishing net in the brook."
        );
        let yours = "You set a fishing net in the brook.";
        let witnessed = "Near sets a fishing net in the brook.";
        let after = world.observe(near).unwrap();
        assert!(after.events.iter().all(|event| event.text != yours));
        assert!(
            after.text.contains("A fishing net is set here."),
            "{}",
            after.text
        );
        assert!(
            !after.text.contains("You have a fishing net"),
            "{}",
            after.text
        );
        assert!(world
            .observe(seen)
            .unwrap()
            .events
            .iter()
            .any(|event| event.text == witnessed));
        let south = world.observe(seen).unwrap().text;
        assert!(
            south.contains("A fishing net is set in the brook to the north."),
            "{south}"
        );
        assert_eq!(
            world.examine(near, ExamineTarget::Net),
            "A fishing net is set here, in the brook. It is woven from reeds."
        );
        assert!(!world.observe(quiet).unwrap().text.contains("net"));
        assert!(world
            .observe(quiet)
            .unwrap()
            .events
            .iter()
            .all(|event| !event.text.contains("net")));
        let ticks = world.tuning.net_ticks;
        for _ in 0..ticks - 1 {
            world.step(&[]);
        }
        assert!(!world
            .observe(near)
            .unwrap()
            .text
            .contains("A raw fish lies here."));
        world.step(&[]);
        let caught = "The fishing net catches a fish.";
        let landed = world.observe(near).unwrap();
        assert!(landed.events.iter().any(|event| event.text == caught));
        assert!(
            landed.text.contains("A fishing net is set here."),
            "{}",
            landed.text
        );
        assert!(
            landed.text.contains("A raw fish lies here."),
            "{}",
            landed.text
        );
        assert!(landed
            .events
            .iter()
            .all(|event| !event.text.contains("splash")));
        let seen_landed = world.observe(seen).unwrap();
        assert!(seen_landed.events.iter().any(|event| event.text == caught));
        assert!(seen_landed
            .events
            .iter()
            .all(|event| !event.text.contains("splash")));
        assert!(
            seen_landed
                .text
                .contains("A fishing net is set in the brook to the north. A raw fish lies there."),
            "{}",
            seen_landed.text
        );
        let splash = "You hear a splash to the south and west.";
        let ears = world.observe(heard).unwrap();
        assert!(ears.events.iter().any(|event| event.text == splash));
        assert!(ears.events.iter().all(|event| event.text != caught));
        assert!(!ears.text.contains("fishing net"), "{}", ears.text);
        assert!(!ears.text.contains("raw fish"), "{}", ears.text);
        let far = world.observe(quiet).unwrap();
        assert!(far.events.iter().all(|event| {
            event.text != caught && !event.text.contains("splash") && !event.text.contains("fish")
        }));
        assert!(!far.text.contains("fish"));
        assert!(!far.text.contains("splash"));
        let gathered = world.step(&[intent("gather", near, Attempt::Gather(Material::Net))]);
        assert_eq!(gathered.results[0].message, "You gather a fishing net.");
        let back = world.observe(near).unwrap();
        assert!(back.events.iter().all(|event| {
            event.text != "You gather a fishing net." && event.text != "Near gathers a fishing net."
        }));
        assert!(
            back.text.contains("You have a fishing net"),
            "{}",
            back.text
        );
        assert!(
            !back.text.contains("A fishing net is set here."),
            "{}",
            back.text
        );
        assert!(back.text.contains("A raw fish lies here."), "{}", back.text);
        assert_eq!(world.material_tally(Material::Net), 1);
        assert_eq!(world.material_tally(Material::Fish), 1);
        assert!(world
            .observe(seen)
            .unwrap()
            .events
            .iter()
            .any(|event| event.text == "Near gathers a fishing net."));
    }
}

#[cfg(test)]
mod weir {
    use super::{Attempt, Dir, ExamineTarget, Material, World};

    fn intent(id: &str, actor: u32, attempt: Attempt) -> super::Intent {
        super::Intent {
            command_id: id.into(),
            actor_id: actor,
            attempt,
        }
    }

    #[test]
    fn a_reed_weir_in_the_brook_catches_a_fish_and_is_spent() {
        let mut world = World::valley(crate::tuning::Tuning::evening(), 1);
        let near = world.spawn_at("Near", "n", 1, 2).unwrap();
        let seen = world.spawn_at("Seen", "s", 1, 3).unwrap();
        let quiet = world.spawn_at("Quiet", "q", 5, 3).unwrap();
        assert!(matches!(
            crate::parse_attempt("set weir").unwrap(),
            Attempt::SetWeir
        ));
        assert!(matches!(
            crate::parse_attempt("weir").unwrap(),
            Attempt::SetWeir
        ));
        assert!(matches!(
            crate::parse_attempt("set trap").unwrap(),
            Attempt::SetSnare
        ));

        let ashore = world.step(&[intent("ashore", quiet, Attempt::SetWeir)]);
        assert_eq!(
            ashore.results[0].message,
            "You cannot set a weir out of the brook."
        );
        assert!(world
            .observe(quiet)
            .unwrap()
            .events
            .iter()
            .any(|event| { event.text == "You cannot set a weir out of the brook." }));
        assert!(world
            .observe(near)
            .unwrap()
            .events
            .iter()
            .all(|event| !event.text.contains("weir")));
        assert!(world
            .observe(seen)
            .unwrap()
            .events
            .iter()
            .all(|event| !event.text.contains("weir")));

        let bare = world.step(&[intent("bare", near, Attempt::SetWeir)]);
        assert_eq!(bare.results[0].message, "You have no reeds to set a weir.");
        assert!(world
            .observe(near)
            .unwrap()
            .events
            .iter()
            .any(|event| event.text == "You have no reeds to set a weir."));
        assert!(world
            .observe(seen)
            .unwrap()
            .events
            .iter()
            .all(|event| !event.text.contains("reeds to set")));
        assert!(world
            .observe(quiet)
            .unwrap()
            .events
            .iter()
            .all(|event| !event.text.contains("reeds to set")));

        world.step(&[intent("west", near, Attempt::Move(Dir::West))]);
        world.step(&[intent("south", near, Attempt::Move(Dir::South))]);
        assert_eq!(world.actor_cell(near), Some((0, 3)));
        let unbound = world.step(&[intent("unbound", near, Attempt::SetWeir)]);
        assert_eq!(
            unbound.results[0].message,
            "You have no cordage to bind a weir."
        );
        assert!(world
            .observe(near)
            .unwrap()
            .events
            .iter()
            .any(|event| event.text == "You have no cordage to bind a weir."));
        assert!(world
            .observe(seen)
            .unwrap()
            .events
            .iter()
            .all(|event| !event.text.contains("cordage")));
        assert!(world
            .observe(quiet)
            .unwrap()
            .events
            .iter()
            .all(|event| !event.text.contains("cordage")));

        let reeds = world.step(&[intent("reeds", near, Attempt::Gather(Material::Reed))]);
        assert!(reeds.results[0].success, "{}", reeds.results[0].message);
        world.step(&[intent("to-willow", near, Attempt::Move(Dir::South))]);
        assert_eq!(world.actor_cell(near), Some((0, 4)));
        let willow = world.step(&[intent("willow", near, Attempt::Gather(Material::Willow))]);
        assert!(willow.results[0].success, "{}", willow.results[0].message);
        world.step(&[intent("back", near, Attempt::Move(Dir::North))]);
        assert_eq!(world.actor_cell(near), Some((0, 3)));
        assert!(world
            .observe(near)
            .unwrap()
            .allowed_actions
            .iter()
            .any(|action| action == "set weir"));

        let set = world.step(&[intent("set", near, Attempt::SetWeir)]);
        assert_eq!(set.results[0].message, "You set a reed weir in the brook.");
        let yours = "You set a reed weir in the brook.";
        let witnessed = "Near sets a reed weir in the brook.";
        let after = world.observe(near).unwrap();
        assert!(after.events.iter().all(|event| event.text != yours));
        assert!(after
            .events
            .iter()
            .all(|event| !event.text.contains("Near sets")));
        assert!(
            after.text.contains("A reed weir is set in the brook here."),
            "{}",
            after.text
        );
        assert!(!after.text.contains("fishing net"), "{}", after.text);
        assert_eq!(
            world.examine(near, ExamineTarget::Weir),
            "A reed weir is set here, in the brook. It is reeds bound with cordage."
        );
        assert!(world
            .observe(seen)
            .unwrap()
            .events
            .iter()
            .any(|event| event.text == witnessed));
        let from_seen = world.observe(seen).unwrap().text;
        assert!(
            from_seen.contains("A reed weir is set in the brook to the west."),
            "{from_seen}"
        );
        assert!(world
            .observe(quiet)
            .unwrap()
            .events
            .iter()
            .all(|event| event.text != witnessed && !event.text.contains("sets a reed weir")));
        assert!(!world.observe(quiet).unwrap().text.contains("reed weir"));
        assert!(world.brook_quiet.is_empty());
        assert!(world.set_nets.is_empty());
        assert!(world.snares.is_empty());
        assert_eq!(world.material_tally(Material::Reed), 3);
        assert_eq!(world.material_tally(Material::Willow), 1);

        let ticks = world.tuning.weir_ticks;
        for _ in 0..ticks - 1 {
            world.step(&[]);
        }
        let brook = super::cell_id(0, 3).unwrap();
        assert!(world.weirs.contains_key(&brook));
        assert!(!world
            .observe(near)
            .unwrap()
            .text
            .contains("A raw fish lies here."));
        world.step(&[]);
        let caught = "The reed weir catches a fish.";
        let landed = world.observe(near).unwrap();
        assert!(landed.events.iter().any(|event| event.text == caught));
        assert!(
            !landed.text.contains("A reed weir is set"),
            "{}",
            landed.text
        );
        assert!(
            landed.text.contains("A raw fish lies here."),
            "{}",
            landed.text
        );
        assert!(world
            .observe(seen)
            .unwrap()
            .events
            .iter()
            .any(|event| event.text == caught));
        assert!(world
            .observe(quiet)
            .unwrap()
            .events
            .iter()
            .all(|event| event.text != caught));
        assert!(world.weirs.is_empty());
        assert!(world.brook_quiet.is_empty());
        assert_eq!(world.material_tally(Material::Fish), 1);
        assert_eq!(
            world.examine(near, ExamineTarget::Weir),
            "You see no weir from here."
        );
    }
}

#[cfg(test)]
mod fishing_net {
    use super::{Attempt, Dir, ExamineTarget, Material, World};

    fn intent(id: &str, actor: u32, attempt: Attempt) -> super::Intent {
        super::Intent {
            command_id: id.into(),
            actor_id: actor,
            attempt,
        }
    }

    #[test]
    fn three_bundles_of_reeds_weave_a_net_that_takes_a_fish() {
        let mut world = World::valley(crate::tuning::Tuning::evening(), 1);
        let near = world.spawn_at("Near", "n", 0, 3).unwrap();
        let seen = world.spawn_at("Seen", "s", 1, 3).unwrap();
        let quiet = world.spawn_at("Quiet", "q", 6, 4).unwrap();
        let bare = world.step(&[intent("bare", near, Attempt::Fish)]);
        assert_eq!(bare.results[0].message, "You are not carrying a spear.");
        assert!(world
            .observe(seen)
            .unwrap()
            .events
            .iter()
            .all(|event| !event.text.contains("spear")));
        world.step(&[intent("one", near, Attempt::Gather(Material::Reed))]);
        world.step(&[intent("two", near, Attempt::Gather(Material::Reed))]);
        world.step(&[intent("off", near, Attempt::Move(Dir::East))]);
        assert_eq!(world.actor_cell(near), Some((1, 3)));
        let short = world.step(&[intent("short", near, Attempt::WeaveNet)]);
        assert_eq!(
            short.results[0].message,
            "You need three bundles of reeds to weave a net."
        );
        assert!(world
            .observe(quiet)
            .unwrap()
            .events
            .iter()
            .all(|event| !event.text.contains("reeds")));
        world.step(&[intent("back", near, Attempt::Move(Dir::West))]);
        world.step(&[intent("three", near, Attempt::Gather(Material::Reed))]);
        let woven = world.step(&[intent("weave", near, Attempt::WeaveNet)]);
        assert_eq!(woven.results[0].message, "You weave a fishing net.");
        let yours = "You weave a fishing net.";
        let witnessed = "Near weaves a fishing net.";
        assert!(world
            .observe(near)
            .unwrap()
            .events
            .iter()
            .all(|event| event.text != yours));
        assert!(world
            .observe(seen)
            .unwrap()
            .events
            .iter()
            .any(|event| event.text == witnessed));
        assert!(world
            .observe(quiet)
            .unwrap()
            .events
            .iter()
            .all(|event| !event.text.contains("net")));
        let arms = world.observe(near).unwrap().text;
        assert!(arms.contains("You have a fishing net."), "{arms}");
        assert!(!arms.contains("2 net"), "{arms}");
        let carried = world.examine(near, ExamineTarget::Net);
        assert_eq!(
            carried,
            "A fishing net is carried by you, in the open. It is woven from reeds."
        );
        assert!(world
            .examine(seen, ExamineTarget::Net)
            .contains("A fishing net is carried by Near, in the open."));
        assert_eq!(
            world.examine(quiet, ExamineTarget::Net),
            "You see no fishing net from here."
        );
        let cast = world.step(&[intent("fish", near, Attempt::Fish)]);
        assert_eq!(
            cast.results[0].message,
            "You take a fish from the brook with the net."
        );
        let took = "You take a fish from the brook with the net.";
        let seen_took = "Near takes a fish from the brook with the net.";
        assert!(world
            .observe(near)
            .unwrap()
            .events
            .iter()
            .all(|event| event.text != took));
        assert!(world
            .observe(seen)
            .unwrap()
            .events
            .iter()
            .any(|event| event.text == seen_took));
        assert!(world
            .observe(quiet)
            .unwrap()
            .events
            .iter()
            .all(|event| !event.text.contains("fish") && !event.text.contains("splash")));
        assert_eq!(world.material_tally(Material::Net), 1);
        assert_eq!(world.material_tally(Material::Fish), 1);
        let after = world.observe(near).unwrap().text;
        assert!(after.contains("You have a fishing net"), "{after}");
        assert!(
            after.contains("a raw fish") || after.contains("a fish"),
            "{after}"
        );
        let gone = world.step(&[intent("again", near, Attempt::Fish)]);
        assert_eq!(
            gone.results[0].message,
            "The fish are gone from this stretch of the brook."
        );
        assert!(world
            .observe(seen)
            .unwrap()
            .events
            .iter()
            .all(|event| !event.text.contains("fish are gone")));
        world.step(&[intent("drop", near, Attempt::Drop(Material::Net))]);
        let ground = world.observe(near).unwrap().text;
        assert!(ground.contains("A fishing net lies here."), "{ground}");
        let west = world.observe(seen).unwrap().text;
        assert!(west.contains("A fishing net lies to the west."), "{west}");
        let exam = world.examine(near, ExamineTarget::Net);
        assert!(
            exam.starts_with("A fishing net lies here. It is woven from reeds."),
            "{exam}"
        );
        let picked = world.step(&[intent("pick", near, Attempt::Gather(Material::Net))]);
        assert_eq!(picked.results[0].message, "You gather a fishing net.");
        assert!(world
            .observe(near)
            .unwrap()
            .text
            .contains("You have a fishing net"));
    }
}

#[cfg(test)]
mod reed_return {
    use super::{Attempt, ExamineTarget, Material, World};

    fn intent(id: &str, actor: u32, attempt: Attempt) -> super::Intent {
        super::Intent {
            command_id: id.into(),
            actor_id: actor,
            attempt,
        }
    }

    #[test]
    fn the_brooks_edge_has_no_reeds_until_they_grow_again() {
        let mut world = World::valley(crate::tuning::Tuning::evening(), 1);
        let near = world.spawn_at("Near", "n", 0, 3).unwrap();
        let seen = world.spawn_at("Seen", "s", 1, 3).unwrap();
        let quiet = world.spawn_at("Quiet", "q", 6, 4).unwrap();
        let here = world.observe(near).unwrap().text;
        assert!(
            here.contains("Reeds grow here, at the brook's edge."),
            "{here}"
        );
        assert!(world
            .observe(seen)
            .unwrap()
            .text
            .contains("Reeds grow at the brook's edge to the west."));
        assert!(!world.observe(quiet).unwrap().text.contains("reed"));
        let one = world.step(&[intent("one", near, Attempt::Gather(Material::Reed))]);
        assert_eq!(one.results[0].message, "You gather a bundle of reeds.");
        let still = world.observe(near).unwrap().text;
        assert!(
            still.contains("Reeds grow here, at the brook's edge."),
            "{still}"
        );
        assert!(!still.contains("The brook's edge has no reeds."), "{still}");
        assert!(still.contains("You have a bundle of reeds."), "{still}");
        let two = world.step(&[intent("two", near, Attempt::Gather(Material::Reed))]);
        assert_eq!(two.results[0].message, "You gather a bundle of reeds.");
        let arms = world.observe(near).unwrap().text;
        assert!(arms.contains("You have two bundles of reeds."), "{arms}");
        assert!(!arms.contains("2 reed"), "{arms}");
        assert!(!arms.contains("2 reeds"), "{arms}");
        let near_exam = world.examine(near, ExamineTarget::Reeds);
        assert!(
            near_exam.contains("Two bundles of reeds are carried by you"),
            "{near_exam}"
        );
        assert!(!near_exam.contains("2 bundles"), "{near_exam}");
        assert!(!near_exam.contains("2 reeds"), "{near_exam}");
        let seen_exam = world.examine(seen, ExamineTarget::Reeds);
        assert!(
            seen_exam.contains("Two bundles of reeds are carried by Near"),
            "{seen_exam}"
        );
        world.step(&[intent("three", near, Attempt::Gather(Material::Reed))]);
        let last = world.step(&[intent("four", near, Attempt::Gather(Material::Reed))]);
        assert_eq!(last.results[0].message, "You gather a bundle of reeds.");
        let empty = world.observe(near).unwrap().text;
        assert!(empty.contains("The brook's edge has no reeds."), "{empty}");
        assert!(
            !empty.contains("Reeds grow here, at the brook's edge."),
            "{empty}"
        );
        assert!(empty.contains("You have four bundles of reeds."), "{empty}");
        let near_empty = world.examine(near, ExamineTarget::Reeds);
        assert!(
            near_empty.contains("The brook's edge here has no reeds."),
            "{near_empty}"
        );
        let west = world.observe(seen).unwrap().text;
        assert!(
            west.contains("The brook's edge to the west has no reeds."),
            "{west}"
        );
        assert!(!west.contains("Reeds grow"), "{west}");
        let seen_empty = world.examine(seen, ExamineTarget::Reeds);
        assert!(
            seen_empty.contains("The brook's edge to the west has no reeds."),
            "{seen_empty}"
        );
        assert!(
            seen_empty.contains("Four bundles of reeds are carried by Near"),
            "{seen_empty}"
        );
        assert!(!world.observe(quiet).unwrap().text.contains("reed"));
        assert_eq!(
            world.examine(quiet, ExamineTarget::Reeds),
            "You see no reeds from here."
        );
        let ticks = world.tuning.reed_ticks;
        for _ in 0..ticks - 1 {
            world.step(&[]);
        }
        assert!(world
            .observe(near)
            .unwrap()
            .text
            .contains("The brook's edge has no reeds."));
        world.step(&[]);
        let back = "Reeds grow at the brook's edge again.";
        let again = world.observe(near).unwrap();
        assert!(
            again.text.contains("Reeds grow here, at the brook's edge."),
            "{}",
            again.text
        );
        assert!(
            !again.text.contains("The brook's edge has no reeds."),
            "{}",
            again.text
        );
        assert!(again.events.iter().any(|event| event.text == back));
        let seen_back = world.observe(seen).unwrap();
        assert!(
            seen_back
                .text
                .contains("Reeds grow at the brook's edge to the west."),
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
        assert!(!world.observe(quiet).unwrap().text.contains("reed"));
        let dropped = world.step(&[intent("drop", near, Attempt::Drop(Material::Reed))]);
        assert_eq!(
            dropped.results[0].message,
            "You drop a bundle of reeds. It is in contact with the ground."
        );
        let second = world.step(&[intent("second", near, Attempt::Gather(Material::Reed))]);
        assert_eq!(second.results[0].message, "You gather a bundle of reeds.");
        let taken = world.observe(near).unwrap().text;
        assert!(
            taken.contains("Reeds grow here, at the brook's edge."),
            "{taken}"
        );
        assert!(!taken.contains("The brook's edge has no reeds."), "{taken}");
        assert!(taken.contains("You have four bundles of reeds."), "{taken}");
        assert!(!taken.contains("2 reed"), "{taken}");
        assert!(!taken.contains("4 reed"), "{taken}");
        assert!(!taken.contains("carried by you (dry)"), "{taken}");
    }
}

#[cfg(test)]
mod iron_return {
    use super::{Attempt, ExamineTarget, Material, World};

    fn intent(id: &str, actor: u32, attempt: Attempt) -> super::Intent {
        super::Intent {
            command_id: id.into(),
            actor_id: actor,
            attempt,
        }
    }

    #[test]
    fn the_slope_has_no_iron_ore_until_a_lump_lies_there_again() {
        let mut world = World::valley(crate::tuning::Tuning::evening(), 1);
        let near = world.spawn_at("Near", "n", 5, 0).unwrap();
        let seen = world.spawn_at("Seen", "s", 5, 1).unwrap();
        let quiet = world.spawn_at("Quiet", "q", 6, 4).unwrap();
        let here = world.observe(near).unwrap().text;
        assert!(here.contains("A lump of iron ore lies here."), "{here}");
        assert!(world
            .observe(seen)
            .unwrap()
            .text
            .contains("A lump of iron ore lies on the slope to the north."));
        assert!(!world.observe(quiet).unwrap().text.contains("iron"));
        let gathered = world.step(&[intent("gather", near, Attempt::Gather(Material::IronOre))]);
        assert_eq!(
            gathered.results[0].message,
            "You gather a lump of iron ore."
        );
        let empty = world.observe(near).unwrap().text;
        assert!(empty.contains("The slope has no iron ore."), "{empty}");
        assert!(!empty.contains("A lump of iron ore lies here."), "{empty}");
        assert!(empty.contains("You have a lump of iron ore."), "{empty}");
        assert!(!empty.contains("2 iron"), "{empty}");
        assert!(!empty.contains("2 ore"), "{empty}");
        let near_exam = world.examine(near, ExamineTarget::IronOre);
        assert!(
            near_exam.contains("The slope here has no iron ore."),
            "{near_exam}"
        );
        assert!(
            near_exam.contains("A lump of iron ore is carried by you, in the open."),
            "{near_exam}"
        );
        let south = world.observe(seen).unwrap().text;
        assert!(
            south.contains("The slope to the north has no iron ore."),
            "{south}"
        );
        assert!(
            !south.contains("A lump of iron ore lies on the slope"),
            "{south}"
        );
        let seen_exam = world.examine(seen, ExamineTarget::IronOre);
        assert!(
            seen_exam.contains("The slope to the north has no iron ore."),
            "{seen_exam}"
        );
        assert!(
            seen_exam.contains("A lump of iron ore is carried by Near, in the open."),
            "{seen_exam}"
        );
        assert!(!world.observe(quiet).unwrap().text.contains("iron"));
        assert_eq!(
            world.examine(quiet, ExamineTarget::IronOre),
            "You see no iron ore from here."
        );
        let ticks = world.tuning.iron_ore_ticks;
        for _ in 0..ticks - 1 {
            world.step(&[]);
        }
        assert!(world
            .observe(near)
            .unwrap()
            .text
            .contains("The slope has no iron ore."));
        world.step(&[]);
        let back = "A lump of iron ore lies on the slope again.";
        let again = world.observe(near).unwrap();
        assert!(
            again.text.contains("A lump of iron ore lies here."),
            "{}",
            again.text
        );
        assert!(
            !again.text.contains("The slope has no iron ore."),
            "{}",
            again.text
        );
        assert!(again.events.iter().any(|event| event.text == back));
        let seen_back = world.observe(seen).unwrap();
        assert!(
            seen_back
                .text
                .contains("A lump of iron ore lies on the slope to the north."),
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
        assert!(!world.observe(quiet).unwrap().text.contains("iron"));
        let second = world.step(&[intent("second", near, Attempt::Gather(Material::IronOre))]);
        assert_eq!(second.results[0].message, "You gather a lump of iron ore.");
        let taken = world.observe(near).unwrap().text;
        assert!(taken.contains("The slope has no iron ore."), "{taken}");
        assert!(!taken.contains("A lump of iron ore lies here."), "{taken}");
        assert!(taken.contains("You have two lumps of iron ore."), "{taken}");
        assert!(!taken.contains("2 iron"), "{taken}");
        assert!(!taken.contains("2 ore"), "{taken}");
        assert!(!taken.contains("carried by you (dry)"), "{taken}");
        let two = world.examine(seen, ExamineTarget::IronOre);
        assert!(
            two.contains("Two lumps of iron ore are carried by Near"),
            "{two}"
        );
    }
}

#[cfg(test)]
mod copper_return {
    use super::{Attempt, ExamineTarget, Material, World};

    fn intent(id: &str, actor: u32, attempt: Attempt) -> super::Intent {
        super::Intent {
            command_id: id.into(),
            actor_id: actor,
            attempt,
        }
    }

    #[test]
    fn the_brook_has_no_copper_ore_until_a_lump_lies_there_again() {
        let mut world = World::valley(crate::tuning::Tuning::evening(), 1);
        let near = world.spawn_at("Near", "n", 1, 2).unwrap();
        let seen = world.spawn_at("Seen", "s", 2, 2).unwrap();
        let quiet = world.spawn_at("Quiet", "q", 6, 4).unwrap();
        let here = world.observe(near).unwrap().text;
        assert!(here.contains("A lump of copper ore lies here."), "{here}");
        assert!(world
            .observe(seen)
            .unwrap()
            .text
            .contains("A lump of copper ore lies on the brook to the west."));
        assert!(!world.observe(quiet).unwrap().text.contains("copper"));
        let gathered = world.step(&[intent("gather", near, Attempt::Gather(Material::CopperOre))]);
        assert_eq!(
            gathered.results[0].message,
            "You gather a lump of copper ore."
        );
        let empty = world.observe(near).unwrap().text;
        assert!(empty.contains("The brook has no copper ore."), "{empty}");
        assert!(
            !empty.contains("A lump of copper ore lies here."),
            "{empty}"
        );
        assert!(empty.contains("You have a lump of copper ore."), "{empty}");
        assert!(!empty.contains("2 ore"), "{empty}");
        assert!(!empty.contains("2 copper"), "{empty}");
        let near_exam = world.examine(near, ExamineTarget::CopperOre);
        assert!(
            near_exam.contains("The brook here has no copper ore."),
            "{near_exam}"
        );
        assert!(
            near_exam.contains("A lump of copper ore is carried by you, in the open."),
            "{near_exam}"
        );
        let west = world.observe(seen).unwrap().text;
        assert!(
            west.contains("The brook to the west has no copper ore."),
            "{west}"
        );
        assert!(
            !west.contains("A lump of copper ore lies on the brook"),
            "{west}"
        );
        let seen_exam = world.examine(seen, ExamineTarget::CopperOre);
        assert!(
            seen_exam.contains("The brook to the west has no copper ore."),
            "{seen_exam}"
        );
        assert!(
            seen_exam.contains("A lump of copper ore is carried by Near, in the open."),
            "{seen_exam}"
        );
        assert!(!world.observe(quiet).unwrap().text.contains("copper"));
        assert_eq!(
            world.examine(quiet, ExamineTarget::CopperOre),
            "You see no copper ore from here."
        );
        let ticks = world.tuning.copper_ore_ticks;
        for _ in 0..ticks - 1 {
            world.step(&[]);
        }
        assert!(world
            .observe(near)
            .unwrap()
            .text
            .contains("The brook has no copper ore."));
        world.step(&[]);
        let back = "A lump of copper ore lies on the brook again.";
        let again = world.observe(near).unwrap();
        assert!(
            again.text.contains("A lump of copper ore lies here."),
            "{}",
            again.text
        );
        assert!(
            !again.text.contains("The brook has no copper ore."),
            "{}",
            again.text
        );
        assert!(again.events.iter().any(|event| event.text == back));
        let seen_back = world.observe(seen).unwrap();
        assert!(
            seen_back
                .text
                .contains("A lump of copper ore lies on the brook to the west."),
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
        assert!(!world.observe(quiet).unwrap().text.contains("copper"));
        let second = world.step(&[intent("second", near, Attempt::Gather(Material::CopperOre))]);
        assert_eq!(
            second.results[0].message,
            "You gather a lump of copper ore."
        );
        let taken = world.observe(near).unwrap().text;
        assert!(taken.contains("The brook has no copper ore."), "{taken}");
        assert!(
            !taken.contains("A lump of copper ore lies here."),
            "{taken}"
        );
        assert!(
            taken.contains("You have two lumps of copper ore."),
            "{taken}"
        );
        assert!(!taken.contains("2 ore"), "{taken}");
        assert!(!taken.contains("2 copper"), "{taken}");
        assert!(!taken.contains("carried by you (dry)"), "{taken}");
        let two = world.examine(seen, ExamineTarget::CopperOre);
        assert!(
            two.contains("Two lumps of copper ore are carried by Near"),
            "{two}"
        );
    }
}

#[cfg(test)]
mod clay_return {
    use super::{Attempt, ExamineTarget, Material, World};

    fn intent(id: &str, actor: u32, attempt: Attempt) -> super::Intent {
        super::Intent {
            command_id: id.into(),
            actor_id: actor,
            attempt,
        }
    }

    #[test]
    fn the_bank_has_no_clay_until_it_lies_there_again() {
        let mut world = World::valley(crate::tuning::Tuning::evening(), 1);
        let near = world.spawn_at("Near", "n", 2, 2).unwrap();
        let seen = world.spawn_at("Seen", "s", 3, 2).unwrap();
        let quiet = world.spawn_at("Quiet", "q", 6, 4).unwrap();
        let bank = world.actor(near).unwrap().cell;
        let here = world.observe(near).unwrap().text;
        assert!(here.contains("Clay lies on the bank here."), "{here}");
        assert!(world
            .observe(seen)
            .unwrap()
            .text
            .contains("Clay lies on the brook bank to the west."));
        assert!(!world.observe(quiet).unwrap().text.contains("clay"));
        let first = world.step(&[intent("first", near, Attempt::Gather(Material::Clay))]);
        assert_eq!(first.results[0].message, "You gather a lump of clay.");
        let one = world.observe(near).unwrap().text;
        assert!(
            one.contains("A lump of clay lies here. It is unfired."),
            "{one}"
        );
        assert!(one.contains("You have a lump of clay."), "{one}");
        assert!(!one.contains("There is no clay on the bank."), "{one}");
        assert!(!one.contains("2 clay"), "{one}");
        assert!(!world.clay_returns_at.contains_key(&bank));
        let second = world.step(&[intent("second", near, Attempt::Gather(Material::Clay))]);
        assert_eq!(second.results[0].message, "You gather a lump of clay.");
        let empty = world.observe(near).unwrap().text;
        assert!(empty.contains("There is no clay on the bank."), "{empty}");
        assert!(!empty.contains("Clay lies on the bank here."), "{empty}");
        assert!(!empty.contains("A lump of clay lies here."), "{empty}");
        assert!(empty.contains("You have two lumps of clay."), "{empty}");
        assert!(!empty.contains("2 clay"), "{empty}");
        assert!(!empty.contains("carried by you (dry)"), "{empty}");
        let near_exam = world.examine(near, ExamineTarget::Clay);
        assert!(
            near_exam.contains("There is no clay on the bank."),
            "{near_exam}"
        );
        assert!(
            near_exam.contains(
                "Two lumps of clay are carried by you, in the open. They are unfired. They cannot boil water."
            ),
            "{near_exam}"
        );
        let west = world.observe(seen).unwrap().text;
        assert!(
            west.contains("There is no clay on the brook bank to the west."),
            "{west}"
        );
        assert!(!west.contains("Clay lies on the brook bank"), "{west}");
        let seen_exam = world.examine(seen, ExamineTarget::Clay);
        assert!(
            seen_exam.contains("There is no clay on the brook bank to the west."),
            "{seen_exam}"
        );
        assert!(
            seen_exam.contains("Two lumps of clay are carried by Near, in the open."),
            "{seen_exam}"
        );
        assert!(!world.observe(quiet).unwrap().text.contains("clay"));
        assert_eq!(
            world.examine(quiet, ExamineTarget::Clay),
            "You see no clay from here."
        );
        let ticks = world.tuning.clay_bank_ticks;
        for _ in 0..ticks - 1 {
            world.step(&[]);
        }
        assert!(world
            .observe(near)
            .unwrap()
            .text
            .contains("There is no clay on the bank."));
        world.step(&[]);
        let back = "Clay lies on the bank again.";
        let again = world.observe(near).unwrap();
        assert!(
            again.text.contains("Clay lies on the bank here."),
            "{}",
            again.text
        );
        assert!(
            !again.text.contains("There is no clay on the bank."),
            "{}",
            again.text
        );
        assert!(again.events.iter().any(|event| event.text == back));
        let seen_back = world.observe(seen).unwrap();
        assert!(
            seen_back
                .text
                .contains("Clay lies on the brook bank to the west."),
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
        assert!(!world.observe(quiet).unwrap().text.contains("clay"));
        let third = world.step(&[intent("third", near, Attempt::Gather(Material::Clay))]);
        assert_eq!(third.results[0].message, "You gather a lump of clay.");
        let taken = world.observe(near).unwrap().text;
        assert!(
            taken.contains("A lump of clay lies here. It is unfired."),
            "{taken}"
        );
        assert!(taken.contains("You have three lumps of clay."), "{taken}");
        assert!(!taken.contains("2 clay"), "{taken}");
        assert!(!taken.contains("3 clay"), "{taken}");
        assert!(!taken.contains("There is no clay on the bank."), "{taken}");
    }
}

#[cfg(test)]
mod branch_return {
    use super::{Attempt, Material, World};

    fn intent(id: &str, actor: u32, attempt: Attempt) -> super::Intent {
        super::Intent {
            command_id: id.into(),
            actor_id: actor,
            attempt,
        }
    }

    #[test]
    fn dry_branches_under_the_rock_return_after_they_are_gone() {
        let mut world = World::valley(crate::tuning::Tuning::evening(), 1);
        let near = world.spawn_at("Near", "n", 6, 1).unwrap();
        let seen = world.spawn_at("Seen", "s", 5, 1).unwrap();
        let quiet = world.spawn_at("Quiet", "q", 0, 4).unwrap();
        let rock = world.actor(near).unwrap().cell;
        let open = super::cell_id(5, 2).unwrap();
        let outside = super::cell_id(5, 1).unwrap();
        let stocked = world.observe(near).unwrap().text;
        assert!(
            stocked.contains("A pile of branches lies here."),
            "{stocked}"
        );
        assert!(
            stocked.contains("Wood left here stays out of the weather."),
            "{stocked}"
        );
        world.tracks.insert(open);
        world.tracks.insert(outside);
        while world.pile_count(open, Material::Branch) > 0 {
            world.take_from_pile(open, Material::Branch, 1);
        }
        let here = world.observe(near).unwrap().text;
        assert!(here.contains("A pile of branches lies here."), "{here}");
        assert!(
            here.contains("The branches here stay out of the weather."),
            "{here}"
        );
        while world.pile_count(rock, Material::Branch) > 0 {
            world.take_from_pile(rock, Material::Branch, 1);
        }
        world.step(&[]);
        let empty = world.observe(near).unwrap().text;
        assert!(
            empty.contains("There are no branches under the rock."),
            "{empty}"
        );
        assert!(
            !empty.contains("The branches here stay out of the weather."),
            "{empty}"
        );
        assert!(
            !empty.contains("Wood left here stays out of the weather."),
            "{empty}"
        );
        assert!(!empty.contains("A pile of branches lies here."), "{empty}");
        let seen_empty = world.observe(seen).unwrap().text;
        assert!(
            seen_empty.contains("There are no branches under the rock to the east."),
            "{seen_empty}"
        );
        assert!(!world
            .observe(quiet)
            .unwrap()
            .text
            .contains("branches under the rock"));
        assert!(!world.branch_returns_at.contains_key(&open));
        let ticks = world.tuning.branch_ticks;
        let started = world.time;
        for _ in 0..ticks - 1 {
            world.step(&[]);
            let text = world.observe(near).unwrap().text;
            let bare = text.contains("There are no branches under the rock.");
            let ground = text.contains("A deer stands on the rock ground.");
            assert!(bare || ground, "clock {} {text}", world.time);
            assert!(!(bare && ground), "clock {} {text}", world.time);
            assert!(
                !text.contains("A deer stands on these branches."),
                "clock {} {text}",
                world.time
            );
        }
        world.step(&[]);
        assert_eq!(world.time, started + ticks);
        let back = "Branches lie under the rock again.";
        let again = world.observe(near).unwrap();
        assert!(
            again.text.contains("A pile of branches lies here."),
            "{}",
            again.text
        );
        assert!(
            again
                .text
                .contains("The branches here stay out of the weather."),
            "{}",
            again.text
        );
        assert!(
            !again.text.contains("There are no branches under the rock."),
            "{}",
            again.text
        );
        assert_eq!(world.pile_count(rock, Material::Branch), 4);
        assert!(world.is_dry(world.pile_moisture(rock, Material::Branch)));
        assert!(again.events.iter().any(|event| event.text == back));
        let seen_back = world.observe(seen).unwrap();
        assert!(
            seen_back.text.contains("a pile of branches"),
            "{}",
            seen_back.text
        );
        assert!(
            !seen_back.text.contains("no branches under the rock"),
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
        assert_eq!(world.pile_count(open, Material::Branch), 0);
        let second = world.step(&[intent("again", near, Attempt::Gather(Material::Branch))]);
        assert_eq!(second.results[0].message, "You gather a branch.");
        assert_eq!(world.pile_count(rock, Material::Branch), 3);
        assert!(world
            .observe(near)
            .unwrap()
            .text
            .contains("A pile of branches lies here."));
    }

    #[test]
    fn an_empty_rock_does_not_say_wood_stays_out_of_the_weather() {
        let mut world = World::valley(crate::tuning::Tuning::evening(), 1);
        let near = world.spawn_at("Near", "n", 6, 1).unwrap();
        let rock = world.actor(near).unwrap().cell;
        let stocked = world.observe(near).unwrap().text;
        assert!(
            stocked.contains("Wood left here stays out of the weather."),
            "{stocked}"
        );
        assert!(
            stocked.contains("A pile of branches lies here."),
            "{stocked}"
        );
        while world.pile_count(rock, Material::Branch) > 0 {
            world.take_from_pile(rock, Material::Branch, 1);
        }
        world.step(&[]);
        let empty = world.observe(near).unwrap().text;
        assert!(
            empty.contains("There are no branches under the rock."),
            "{empty}"
        );
        assert!(
            !empty.contains("Wood left here stays out of the weather."),
            "{empty}"
        );
        assert!(
            !empty.contains("The branches here stay out of the weather."),
            "{empty}"
        );
        assert!(!empty.contains("A pile of branches lies here."), "{empty}");
    }

    #[test]
    fn a_deer_on_the_empty_rock_is_not_standing_on_branches() {
        let mut world = World::valley(crate::tuning::Tuning::evening(), 1);
        let near = world.spawn_at("Near", "n", 6, 1).unwrap();
        let seen = world.spawn_at("Seen", "s", 5, 3).unwrap();
        let quiet = world.spawn_at("Quiet", "q", 0, 0).unwrap();
        let rock = world.actor(near).unwrap().cell;
        while world.pile_count(rock, Material::Branch) > 0 {
            world.take_from_pile(rock, Material::Branch, 1);
        }
        world.step(&[]);
        let away = world.observe(near).unwrap();
        assert!(
            away.text.contains("There are no branches under the rock."),
            "{}",
            away.text
        );
        assert!(!away.text.contains("A deer stands on the rock ground."));
        world.deer.as_mut().unwrap().cell = rock;
        let here = world.observe(near).unwrap();
        assert!(
            here.text.contains("A deer stands on the rock ground."),
            "{}",
            here.text
        );
        assert!(!here.text.contains("A deer stands on these branches."));
        assert!(!here.text.contains("There are no branches under the rock."));
        assert!(here
            .events
            .iter()
            .all(|event| event.text != "A deer stands on the rock ground."));
        let seen_text = world.observe(seen).unwrap().text;
        assert!(
            !seen_text.contains("A deer stands on these branches"),
            "{seen_text}"
        );
        assert!(
            !(seen_text.contains("these branches")
                && seen_text.contains("no branches under the rock")),
            "{seen_text}"
        );
        let quiet_text = world.observe(quiet).unwrap().text;
        assert!(!quiet_text.contains("rock ground"), "{quiet_text}");
        assert!(
            !quiet_text.contains("branches under the rock"),
            "{quiet_text}"
        );
        world.deer.as_mut().unwrap().cell = super::cell_id(2, 3).unwrap();
        let bare = world.observe(near).unwrap().text;
        assert!(
            bare.contains("There are no branches under the rock."),
            "{bare}"
        );
        assert!(!bare.contains("A deer stands on the rock ground."));
        assert!(!bare.contains("A deer stands on these branches."));
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

    #[test]
    fn soaked_clothes_dry_a_step_at_a_time_on_a_fire() {
        let mut world = World::valley(crate::tuning::Tuning::evening(), 1);
        let near = world.spawn_at("Near", "n", 6, 1).unwrap();
        let seen = world.spawn_at("Seen", "s", 5, 1).unwrap();
        let quiet = world.spawn_at("Quiet", "q", 0, 4).unwrap();
        let open = world.spawn_at("Open", "o", 4, 3).unwrap();
        world.actors.get_mut(&near).unwrap().wetness = 1000;
        world.actors.get_mut(&open).unwrap().wetness = 1000;
        let cell = world.actor(near).unwrap().cell;
        world.mint_fire(cell, 0);
        let mut damp = 0;
        let mut dry = 0;
        let mut noted = Vec::new();
        let mut words = Vec::new();
        for _ in 0..8 {
            world.step(&[]);
            let look = world.observe(near).unwrap();
            words.push(look.self_info.wetness.clone());
            for event in &look.events {
                if noted.contains(&event.id) {
                    continue;
                }
                noted.push(event.id);
                if event.text == "Your clothes are damp." {
                    damp += 1;
                }
                if event.text == "Your clothes are dry." {
                    dry += 1;
                }
            }
            assert!(look
                .events
                .iter()
                .all(|event| event.text != "Near's clothes are damp."
                    && event.text != "Near's clothes are dry."));
            assert!(!look.text.contains("drying by the fire"), "{}", look.text);
        }
        assert_eq!(damp, 1, "{words:?}");
        assert_eq!(dry, 1, "{words:?}");
        assert!(words.contains(&"soaked".to_string()), "{words:?}");
        assert!(words.contains(&"damp".to_string()), "{words:?}");
        assert_eq!(words.last().map(String::as_str), Some("dry"), "{words:?}");
        assert!(!words.contains(&"wet".to_string()), "{words:?}");
        let body = world.examine(seen, ExamineTarget::Person("Near".into()));
        assert!(body.contains("Their clothes are dry."), "{body}");
        assert!(world.observe(seen).unwrap().events.iter().all(|event| {
            event.text != "Your clothes are damp." && event.text != "Your clothes are dry."
        }));
        assert!(world.observe(quiet).unwrap().events.iter().all(|event| {
            event.text != "Your clothes are damp." && event.text != "Your clothes are dry."
        }));
        assert_eq!(world.observe(open).unwrap().self_info.wetness, "soaked");
        let before = world
            .observe(near)
            .unwrap()
            .events
            .iter()
            .filter(|event| event.text == "Your clothes are dry.")
            .count();
        world.step(&[]);
        world.step(&[]);
        let after = world
            .observe(near)
            .unwrap()
            .events
            .iter()
            .filter(|event| event.text == "Your clothes are dry.")
            .count();
        assert_eq!(after, before);
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
                && on_it.contains("Wolf tracks mark the ground here.")
                && on_it.contains("A wolf stands here."),
            "{on_it}"
        );
        let dark = world.observe(seen).unwrap().text;
        assert!(!dark.contains("Wolf tracks"), "{dark}");
        assert!(!dark.contains("A wolf stands"), "{dark}");
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
        assert!(lit.contains("A wolf stands to the north."), "{lit}");
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
        assert!(!dark_again.contains("A wolf stands"), "{dark_again}");
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

    fn intent(id: &str, actor: u32, attempt: super::Attempt) -> super::Intent {
        super::Intent {
            command_id: id.into(),
            actor_id: actor,
            attempt,
        }
    }

    #[test]
    fn a_wolf_stands_on_the_howl_then_leaves_and_the_day_has_none() {
        let mut world = World::valley(crate::tuning::Tuning::evening(), 1);
        let here = world.spawn_at("Near", "n", 0, 0).unwrap();
        let seen = world.spawn_at("Seen", "s", 0, 1).unwrap();
        let quiet = world.spawn_at("Quiet", "q", 6, 1).unwrap();
        let night = world.tuning.night_at;
        while world.time + 1 < night {
            world.step(&[]);
        }
        let day = world.observe(here).unwrap().text;
        assert!(!day.contains("A wolf stands"), "{day}");
        world.step(&[]);
        assert_eq!(world.time, night);
        let on_it = world.observe(here).unwrap().text;
        assert!(on_it.contains("A wolf stands here."), "{on_it}");
        assert!(
            on_it.contains("Wolf tracks mark the ground here."),
            "{on_it}"
        );
        assert!(!world.observe(seen).unwrap().text.contains("A wolf stands"));
        assert!(!world.observe(quiet).unwrap().text.contains("A wolf stands"));
        let fire = world.mint_fire(world.actor(seen).unwrap().cell, 0);
        let lit = world.observe(seen).unwrap().text;
        assert!(lit.contains("A wolf stands to the north."), "{lit}");
        world.extinguish(fire, "The fire goes out.", None);
        let ticks = world.tuning.wolf_ticks;
        for _ in 0..ticks - 1 {
            world.step(&[]);
            assert!(
                world
                    .observe(here)
                    .unwrap()
                    .text
                    .contains("A wolf stands here."),
                "clock {}",
                world.time
            );
        }
        world.step(&[]);
        assert_eq!(world.time, night + ticks);
        let left = world.observe(here).unwrap().text;
        assert!(!left.contains("A wolf stands"), "{left}");
        assert!(left.contains("Wolf tracks mark the ground here."), "{left}");
        let dawn = night + world.tuning.night_len;
        while world.time < dawn {
            world.step(&[]);
        }
        assert!(!world.night());
        let morning = world.observe(here).unwrap().text;
        assert!(!morning.contains("A wolf stands"), "{morning}");
        assert!(!morning.contains("wolf"), "{morning}");
    }

    #[test]
    fn walking_closer_leaves_the_wolf_gone_and_the_tracks_stay() {
        let mut world = World::valley(crate::tuning::Tuning::evening(), 1);
        let here = world.spawn_at("Near", "n", 0, 0).unwrap();
        let walker = world.spawn_at("Seen", "s", 2, 0).unwrap();
        let night = world.tuning.night_at;
        while world.time < night {
            world.step(&[]);
        }
        assert!(world
            .observe(here)
            .unwrap()
            .text
            .contains("A wolf stands here."));
        let stepped = world.step(&[intent(
            "closer",
            walker,
            super::Attempt::Move(super::Dir::West),
        )]);
        assert_eq!(stepped.results[0].message, "You walk west.");
        let after = world.observe(here).unwrap().text;
        assert!(!after.contains("A wolf stands"), "{after}");
        assert!(
            after.contains("Wolf tracks mark the ground here."),
            "{after}"
        );
        assert!(!world
            .observe(walker)
            .unwrap()
            .text
            .contains("A wolf stands"));
        assert!(!after.contains("The wolf flees the fire."));
    }

    #[test]
    fn a_fire_sends_the_wolf_off_and_the_lighter_does_not_read_it_twice() {
        let mut world = World::valley(crate::tuning::Tuning::evening(), 1);
        let near = world.spawn_at("Near", "n", 0, 0).unwrap();
        let seen = world.spawn_at("Seen", "s", 0, 1).unwrap();
        let quiet = world.spawn_at("Quiet", "q", 6, 1).unwrap();
        let night = world.tuning.night_at;
        while world.time < night {
            world.step(&[]);
        }
        let standing = world.observe(near).unwrap().text;
        assert!(standing.contains("A wolf stands here."), "{standing}");
        let refused = world.step(&[intent("rain", near, super::Attempt::Light)]);
        assert_eq!(
            refused.results[0].message,
            "The rain puts the bark out before it catches."
        );
        assert!(!refused.results[0].success);
        assert!(world
            .observe(near)
            .unwrap()
            .text
            .contains("A wolf stands here."));
        assert!(world.observe(seen).unwrap().events.iter().all(|event| {
            event.text != "The rain puts the bark out before it catches."
                && event.text != "The wolf flees the fire."
        }));
        let clear = night + world.tuning.night_len + world.tuning.day_len;
        while world.time < clear {
            world.step(&[]);
        }
        assert!(world.night());
        assert!(!world.is_raining());
        assert!(world
            .observe(near)
            .unwrap()
            .text
            .contains("A wolf stands here."));
        let cell = world.actor(near).unwrap().cell;
        world.mint_item(
            super::Material::Branch,
            0,
            cell,
            Some(near),
            super::Role::Loose,
        );
        world.mint_item(
            super::Material::Bark,
            0,
            cell,
            Some(near),
            super::Role::Loose,
        );
        let lit = world.step(&[intent("light", near, super::Attempt::Light)]);
        assert!(
            lit.results[0].message.contains("You light a fire"),
            "{}",
            lit.results[0].message
        );
        assert!(
            lit.results[0].message.contains("The wolf flees the fire."),
            "{}",
            lit.results[0].message
        );
        let near_look = world.observe(near).unwrap();
        assert!(
            !near_look.text.contains("A wolf stands"),
            "{}",
            near_look.text
        );
        assert!(near_look.text.contains("Wolf tracks mark the ground here."));
        assert!(near_look
            .events
            .iter()
            .all(|event| event.text != "The wolf flees the fire."));
        let seen_look = world.observe(seen).unwrap();
        assert!(seen_look
            .events
            .iter()
            .any(|event| event.text == "The wolf flees the fire."));
        assert!(!seen_look.text.contains("A wolf stands"));
        assert!(world
            .observe(quiet)
            .unwrap()
            .events
            .iter()
            .all(|event| event.text != "The wolf flees the fire."));
    }

    #[test]
    fn a_brand_on_the_wolfs_ground_sends_it_off() {
        let mut world = World::valley(crate::tuning::Tuning::evening(), 1);
        let near = world.spawn_at("Near", "n", 0, 0).unwrap();
        let night = world.tuning.night_at;
        while world.time < night {
            world.step(&[]);
        }
        assert!(world
            .observe(near)
            .unwrap()
            .text
            .contains("A wolf stands here."));
        let cell = world.actor(near).unwrap().cell;
        world.mint_item(
            super::Material::Brand,
            0,
            cell,
            Some(near),
            super::Role::Loose,
        );
        world.step(&[]);
        let look = world.observe(near).unwrap();
        assert!(!look.text.contains("A wolf stands"), "{}", look.text);
        assert!(look.text.contains("Wolf tracks mark the ground here."));
        assert!(look
            .events
            .iter()
            .any(|event| event.text == "The wolf flees the fire."));
    }

    #[test]
    fn a_hardened_spear_brings_the_wolf_down_on_its_own_cell() {
        let mut world = World::valley(crate::tuning::Tuning::evening(), 1);
        let near = world.spawn_at("Near", "n", 0, 0).unwrap();
        let seen = world.spawn_at("Seen", "s", 0, 1).unwrap();
        let quiet = world.spawn_at("Quiet", "q", 6, 4).unwrap();
        let night = world.tuning.night_at;
        while world.time < night {
            world.step(&[]);
        }
        world.wolf_until = night + world.tuning.night_len;
        assert!(world.deer_cell().is_some());
        let bare = world.step(&[intent("bare", near, super::Attempt::Hunt)]);
        assert_eq!(bare.results[0].message, "You are not carrying a spear.");
        assert!(!bare.results[0].success);
        assert!(world
            .observe(seen)
            .unwrap()
            .events
            .iter()
            .all(|event| event.text != "You are not carrying a spear."));
        assert!(world
            .observe(near)
            .unwrap()
            .text
            .contains("A wolf stands here."));
        let cell = world.actor(near).unwrap().cell;
        let spear = world.mint_item(
            super::Material::Spear,
            0,
            cell,
            Some(near),
            super::Role::Loose,
        );
        let soft = world.step(&[intent("soft", near, super::Attempt::Hunt)]);
        assert_eq!(
            soft.results[0].message,
            "The spear's point is not hardened."
        );
        assert!(world
            .observe(seen)
            .unwrap()
            .events
            .iter()
            .all(|event| event.text != "The spear's point is not hardened."));
        {
            let point = world.items.get_mut(&spear).unwrap();
            point.hardened = true;
            point.flint_tip = true;
        }
        let shot = world.step(&[intent("shot", near, super::Attempt::Hunt)]);
        assert_eq!(
            shot.results[0].message,
            "You bring the wolf down with the flint spear."
        );
        assert!(shot.results[0].success);
        assert!(world.wolf_standing().is_none());
        assert!(world.deer_cell().is_some());
        let look = world.observe(near).unwrap();
        assert!(
            look.text.contains("A wolf carcass lies here."),
            "{}",
            look.text
        );
        assert!(!look.text.contains("A deer carcass"), "{}", look.text);
        assert!(!look.text.contains("A wolf stands"), "{}", look.text);
        assert!(look.text.contains("Wolf tracks mark the ground here."));
        assert!(look
            .events
            .iter()
            .all(|event| !event.text.contains("brings the wolf")));
        let heard = "Near brings the wolf down with the flint spear.";
        assert!(world
            .observe(seen)
            .unwrap()
            .events
            .iter()
            .any(|event| event.text == heard));
        assert!(world
            .observe(quiet)
            .unwrap()
            .events
            .iter()
            .all(|event| event.text != heard));
        let body = world.examine(near, super::ExamineTarget::Carcass);
        assert!(body.contains("A wolf carcass"), "{body}");
        let cut = world.step(&[intent("cut", near, super::Attempt::Butcher)]);
        assert_eq!(
            cut.results[0].message,
            "You butcher the wolf carcass into three portions of raw meat, a hide, and a bone."
        );
        assert_eq!(world.ground_ids(cell, super::Material::Meat).len(), 3);
        assert_eq!(world.ground_ids(cell, super::Material::Hide).len(), 1);
        assert!(world.ground_ids(cell, super::Material::Carcass).is_empty());
    }

    #[test]
    fn a_bow_brings_the_wolf_down_at_sight() {
        let mut world = World::valley(crate::tuning::Tuning::evening(), 1);
        let near = world.spawn_at("Near", "n", 0, 2).unwrap();
        let seen = world.spawn_at("Seen", "s", 0, 1).unwrap();
        let quiet = world.spawn_at("Quiet", "q", 6, 4).unwrap();
        let night = world.tuning.night_at;
        while world.time < night {
            world.step(&[]);
        }
        if let Some(deer) = world.deer.as_mut() {
            deer.cell = super::cell_id(6, 4).unwrap();
        }
        let cell = world.actor(near).unwrap().cell;
        world.mint_item(
            super::Material::Bow,
            0,
            cell,
            Some(near),
            super::Role::Loose,
        );
        world.mint_item(
            super::Material::Flake,
            0,
            cell,
            Some(near),
            super::Role::Loose,
        );
        let shot = world.step(&[intent("shot", near, super::Attempt::Hunt)]);
        assert_eq!(
            shot.results[0].message,
            "You bring the wolf down with the bow. The flint flake is spent."
        );
        assert!(world.loose_ids(near, super::Material::Flake).is_empty());
        assert_eq!(world.loose_ids(near, super::Material::Bow).len(), 1);
        assert!(world.wolf_standing().is_none());
        assert!(world.items.values().any(|item| {
            item.material == super::Material::Carcass
                && item.wolf
                && item.cell == super::cell_id(0, 0).unwrap()
        }));
        assert!(world
            .observe(near)
            .unwrap()
            .events
            .iter()
            .all(|event| !event.text.contains("the wolf")));
        let heard = "Near brings the wolf down with the bow. The flint flake is spent.";
        assert!(world
            .observe(seen)
            .unwrap()
            .events
            .iter()
            .any(|event| event.text == heard));
        assert!(world
            .observe(quiet)
            .unwrap()
            .events
            .iter()
            .all(|event| event.text != heard));
    }

    #[test]
    fn a_spear_from_the_next_cell_can_miss_or_bolt() {
        let (miss_seed, miss_at) = wolf_cast(1);
        let (bolt_seed, bolt_at) = wolf_cast(2);
        expect_cast(miss_seed, miss_at, "You miss the wolf. It moves off.");
        expect_cast(
            bolt_seed,
            bolt_at,
            "The wolf bolts. Your spear does not land.",
        );
    }

    fn wolf_cast(residue: u64) -> (u64, u64) {
        for seed in 1..80 {
            let mut world = World::valley(crate::tuning::Tuning::evening(), seed);
            let near = world.spawn_at("Near", "n", 0, 1).unwrap();
            let night = world.tuning.night_at;
            while world.time < night {
                world.step(&[]);
            }
            let end = night + world.tuning.night_len;
            world.wolf_until = end;
            while world.time < end {
                let hash = super::contest_hash(seed, world.time, "hunt", "0,0:wolf", near);
                if hash % 3 == residue {
                    return (seed, world.time);
                }
                world.step(&[]);
            }
        }
        panic!("no wolf cast for residue {residue}");
    }

    fn expect_cast(seed: u64, at: u64, yours: &str) {
        let mut world = World::valley(crate::tuning::Tuning::evening(), seed);
        let near = world.spawn_at("Near", "n", 0, 1).unwrap();
        let seen = world.spawn_at("Seen", "s", 1, 1).unwrap();
        let quiet = world.spawn_at("Quiet", "q", 6, 4).unwrap();
        let night = world.tuning.night_at;
        while world.time < night {
            world.step(&[]);
        }
        world.wolf_until = night + world.tuning.night_len;
        while world.time < at {
            world.step(&[]);
        }
        let cell = world.actor(near).unwrap().cell;
        let spear = world.mint_item(
            super::Material::Spear,
            0,
            cell,
            Some(near),
            super::Role::Loose,
        );
        world.items.get_mut(&spear).unwrap().hardened = true;
        let shot = world.step(&[intent("shot", near, super::Attempt::Hunt)]);
        assert_eq!(shot.results[0].message, yours);
        assert!(!shot.results[0].success);
        assert!(world.wolf_standing().is_none());
        assert!(world
            .items
            .values()
            .all(|item| item.material != super::Material::Carcass));
        assert!(world
            .observe(near)
            .unwrap()
            .events
            .iter()
            .all(|event| event.text != yours && !event.text.contains("the wolf")));
        let heard = if yours.starts_with("You miss") {
            "Near misses the wolf. It moves off."
        } else {
            "The wolf bolts from Near's spear."
        };
        assert!(world
            .observe(seen)
            .unwrap()
            .events
            .iter()
            .any(|event| event.text == heard));
        assert!(world
            .observe(quiet)
            .unwrap()
            .events
            .iter()
            .all(|event| event.text != heard));
    }

    #[test]
    fn lighting_still_sends_the_wolf_off_when_a_spear_is_carried() {
        let mut world = World::valley(crate::tuning::Tuning::evening(), 1);
        let near = world.spawn_at("Near", "n", 0, 0).unwrap();
        let clear = world.tuning.night_at + world.tuning.night_len + world.tuning.day_len;
        while world.time < clear {
            world.step(&[]);
        }
        assert!(world
            .observe(near)
            .unwrap()
            .text
            .contains("A wolf stands here."));
        let cell = world.actor(near).unwrap().cell;
        let spear = world.mint_item(
            super::Material::Spear,
            0,
            cell,
            Some(near),
            super::Role::Loose,
        );
        world.items.get_mut(&spear).unwrap().hardened = true;
        world.mint_item(
            super::Material::Branch,
            0,
            cell,
            Some(near),
            super::Role::Loose,
        );
        world.mint_item(
            super::Material::Bark,
            0,
            cell,
            Some(near),
            super::Role::Loose,
        );
        let lit = world.step(&[intent("light", near, super::Attempt::Light)]);
        assert!(
            lit.results[0].message.contains("The wolf flees the fire."),
            "{}",
            lit.results[0].message
        );
        assert!(world.wolf_standing().is_none());
        assert!(world
            .items
            .values()
            .all(|item| item.material != super::Material::Carcass));
    }

    #[test]
    fn a_neighbor_who_can_see_reads_a_wolf_carcass() {
        let mut world = World::valley(crate::tuning::Tuning::evening(), 1);
        let near = world.spawn_at("Near", "n", 0, 0).unwrap();
        let seen = world.spawn_at("Seen", "s", 0, 1).unwrap();
        let wolf_cell = world.actor(near).unwrap().cell;
        let deer_cell = super::cell_id(1, 1).unwrap();
        let wolf = world.mint_item(
            super::Material::Carcass,
            0,
            wolf_cell,
            None,
            super::Role::Loose,
        );
        world.items.get_mut(&wolf).unwrap().wolf = true;
        world.mint_item(
            super::Material::Carcass,
            0,
            deer_cell,
            None,
            super::Role::Loose,
        );
        let day = world.observe(seen).unwrap().text;
        let north = day
            .lines()
            .find(|line| line.starts_with("North:"))
            .unwrap_or("");
        assert!(north.contains("a wolf carcass"), "{north}");
        assert!(!north.contains("a carcass"), "{north}");
        let east = day
            .lines()
            .find(|line| line.starts_with("East:"))
            .unwrap_or("");
        assert!(east.contains("a carcass"), "{east}");
        assert!(!east.contains("wolf"), "{east}");
        let here = world.observe(near).unwrap().text;
        assert!(here.contains("A wolf carcass lies here."), "{here}");
        let night = world.tuning.night_at;
        while world.time < night {
            world.step(&[]);
        }
        let dark = world.observe(seen).unwrap().text;
        assert!(
            dark.contains("North: you cannot make out the ground."),
            "{dark}"
        );
        assert!(!dark.contains("wolf carcass"), "{dark}");
        assert!(world
            .observe(near)
            .unwrap()
            .text
            .contains("A wolf carcass lies here."));
        world.mint_fire(wolf_cell, 0);
        let lit = world.observe(seen).unwrap().text;
        let lit_north = lit
            .lines()
            .find(|line| line.starts_with("North:"))
            .unwrap_or("");
        assert!(lit_north.contains("a wolf carcass"), "{lit_north}");
        assert!(!lit_north.contains("a carcass"), "{lit_north}");
    }

    #[test]
    fn a_wolf_hide_sews_into_a_warmer_cloak() {
        let mut world = World::valley(crate::tuning::Tuning::evening(), 1);
        let near = world.spawn_at("Near", "n", 5, 3).unwrap();
        let seen = world.spawn_at("Seen", "s", 5, 3).unwrap();
        let quiet = world.spawn_at("Quiet", "q", 0, 4).unwrap();
        let cell = world.actor(near).unwrap().cell;
        let carcass = world.mint_item(super::Material::Carcass, 0, cell, None, super::Role::Loose);
        world.items.get_mut(&carcass).unwrap().wolf = true;
        world.mint_item(
            super::Material::Knife,
            0,
            cell,
            Some(near),
            super::Role::Loose,
        );
        world.mint_item(
            super::Material::Awl,
            0,
            cell,
            Some(near),
            super::Role::Loose,
        );
        world.mint_item(
            super::Material::Hide,
            0,
            cell,
            Some(seen),
            super::Role::Loose,
        );
        world.mint_item(
            super::Material::Awl,
            0,
            cell,
            Some(seen),
            super::Role::Loose,
        );

        let butchered = world.step(&[intent("butcher", near, super::Attempt::Butcher)]);
        assert!(
            butchered.results[0].success,
            "{}",
            butchered.results[0].message
        );
        assert!(butchered.results[0].message.contains("wolf carcass"));
        let hides = world.ground_ids(cell, super::Material::Hide);
        assert_eq!(hides.len(), 1);
        assert!(world.item(hides[0]).unwrap().wolf);

        let pierced_deer = world.step(&[
            intent(
                "gather",
                near,
                super::Attempt::Gather(super::Material::Hide),
            ),
            intent("pierce-deer", seen, super::Attempt::Pierce),
        ]);
        assert!(
            pierced_deer.results.iter().all(|result| result.success),
            "{pierced_deer:?}"
        );
        let sewn = world.step(&[
            intent("pierce", near, super::Attempt::Pierce),
            intent("sew-deer", seen, super::Attempt::Sew),
        ]);
        let near_pierce = sewn
            .results
            .iter()
            .find(|result| result.actor_id == near)
            .unwrap();
        assert_eq!(
            near_pierce.message,
            "You pierce the wolf hide with the copper awl. It is ready to sew."
        );
        assert!(world
            .observe(near)
            .unwrap()
            .events
            .iter()
            .all(|event| { !event.text.contains("wolf hide") }));
        assert!(world.observe(seen).unwrap().events.iter().any(|event| {
            event.text == "Near pierces the wolf hide with the copper awl. It is ready to sew."
        }));
        assert!(world
            .observe(quiet)
            .unwrap()
            .events
            .iter()
            .all(|event| { !event.text.contains("wolf hide") }));

        let cloaks = world.step(&[
            intent("sew", near, super::Attempt::Sew),
            intent("wear-deer", seen, super::Attempt::Wear),
        ]);
        assert_eq!(
            cloaks.results[0].message,
            "You sew the pierced wolf hide into a cloak."
        );
        let carried = world.examine(near, super::ExamineTarget::Hide);
        assert!(
            carried.contains(
                "A sewn wolf-hide cloak is carried by you, in the open. It keeps off more of the cold."
            ) || carried.contains("It keeps off more of the cold."),
            "{carried}"
        );
        assert!(
            !carried.contains("deer-hide cloak is carried by you"),
            "{carried}"
        );

        let wore = world.step(&[intent("wear", near, super::Attempt::Wear)]);
        assert_eq!(
            wore.results[0].message,
            "You wear the sewn wolf-hide cloak."
        );
        let on_you = world.examine(near, super::ExamineTarget::Hide);
        assert!(
            on_you.contains("It keeps off more of the wind and the cold."),
            "{on_you}"
        );
        assert!(world
            .examine(seen, super::ExamineTarget::Hide)
            .contains("A sewn deer-hide cloak is on you"));
        let near_events = world.observe(near).unwrap();
        assert!(near_events.events.iter().all(|event| {
            !event.text.contains("sewn wolf-hide") && !event.text.contains("pierced wolf hide")
        }));
        assert!(world
            .observe(seen)
            .unwrap()
            .events
            .iter()
            .any(|event| { event.text == "Near sews the pierced wolf hide into a cloak." }));
        assert!(world
            .observe(seen)
            .unwrap()
            .events
            .iter()
            .any(|event| { event.text == "Near wears the sewn wolf-hide cloak." }));
        assert!(world
            .observe(quiet)
            .unwrap()
            .events
            .iter()
            .all(|event| { !event.text.contains("wolf") }));
        assert!(near_events
            .text
            .contains("The sewn wolf-hide cloak is on you."));
        assert!(
            near_events.text.contains("You are in the wind.")
                || near_events.text.contains("You are in a little wind.")
        );

        for id in [near, seen, quiet] {
            let person = world.actors.get_mut(&id).unwrap();
            person.cold = 0;
            person.warmth = 0;
            person.warm_from_fire = false;
            person.cell = cell;
        }
        for _ in 0..8 {
            world.step(&[]);
        }
        let wolf_body = world.observe(near).unwrap();
        let deer_body = world.observe(seen).unwrap();
        let bare_body = world.observe(quiet).unwrap();
        assert!(
            wolf_body.text.contains("You are warm."),
            "{}",
            wolf_body.text
        );
        assert!(
            wolf_body.text.contains("You are in a little wind."),
            "{}",
            wolf_body.text
        );
        assert!(
            deer_body.text.contains("You are chilly."),
            "{}",
            deer_body.text
        );
        assert!(
            deer_body.text.contains("You are in the wind."),
            "{}",
            deer_body.text
        );
        assert!(
            bare_body.text.contains("You are cold."),
            "{}",
            bare_body.text
        );
        assert!(
            bare_body.text.contains("You are in the wind."),
            "{}",
            bare_body.text
        );
        assert!(wolf_body.self_info.cold == "warm");
        assert!(deer_body.self_info.cold == "chilly");
        assert!(bare_body.self_info.cold == "cold");
        let wolf_cold = world.actor(near).unwrap().cold;
        let deer_cold = world.actor(seen).unwrap().cold;
        let bare_cold = world.actor(quiet).unwrap().cold;
        assert!(wolf_cold < deer_cold && deer_cold < bare_cold);
    }

    #[test]
    fn a_wolf_hide_that_is_not_ready_fails_in_private() {
        let mut world = World::valley(crate::tuning::Tuning::evening(), 1);
        let near = world.spawn_at("Near", "n", 5, 3).unwrap();
        let seen = world.spawn_at("Seen", "s", 5, 3).unwrap();
        let cell = world.actor(near).unwrap().cell;
        let hide = world.mint_item(
            super::Material::Hide,
            0,
            cell,
            Some(near),
            super::Role::Loose,
        );
        world.items.get_mut(&hide).unwrap().wolf = true;
        let missed = world.step(&[intent("pierce", near, super::Attempt::Pierce)]);
        assert!(!missed.results[0].success);
        assert_eq!(
            missed.results[0].message,
            "You are not carrying a copper awl."
        );
        assert!(world
            .observe(seen)
            .unwrap()
            .events
            .iter()
            .all(|event| { !event.text.contains("wolf") && !event.text.contains("awl") }));
        world.mint_item(
            super::Material::Awl,
            0,
            cell,
            Some(near),
            super::Role::Loose,
        );
        let early = world.step(&[intent("sew", near, super::Attempt::Sew)]);
        assert!(!early.results[0].success);
        assert_eq!(early.results[0].message, "The wolf hide is not pierced.");
        assert!(world.observe(near).unwrap().events.iter().all(|event| {
            event.text != "The wolf hide is not pierced."
                && !event.text.contains("sews the pierced wolf")
        }));
        assert!(world
            .observe(seen)
            .unwrap()
            .events
            .iter()
            .all(|event| { !event.text.contains("wolf") }));
    }

    #[test]
    fn gathering_a_wolf_hide_names_the_wolf() {
        let mut world = World::valley(crate::tuning::Tuning::evening(), 1);
        let near = world.spawn_at("Near", "n", 5, 3).unwrap();
        let seen = world.spawn_at("Seen", "s", 5, 3).unwrap();
        let quiet = world.spawn_at("Quiet", "q", 0, 4).unwrap();
        let cell = world.actor(near).unwrap().cell;
        let missed = world.step(&[intent(
            "none",
            quiet,
            super::Attempt::Gather(super::Material::Hide),
        )]);
        assert!(!missed.results[0].success);
        assert_eq!(
            missed.results[0].message,
            "There is no hide here to gather."
        );
        assert!(world
            .observe(near)
            .unwrap()
            .events
            .iter()
            .all(|event| { event.text != "There is no hide here to gather." }));
        assert!(world
            .observe(seen)
            .unwrap()
            .events
            .iter()
            .all(|event| { event.text != "There is no hide here to gather." }));
        let wolf = world.mint_item(super::Material::Hide, 0, cell, None, super::Role::Loose);
        world.items.get_mut(&wolf).unwrap().wolf = true;
        world.mint_item(super::Material::Hide, 0, cell, None, super::Role::Loose);
        let lifted = world.step(&[intent(
            "wolf",
            near,
            super::Attempt::Gather(super::Material::Hide),
        )]);
        assert_eq!(lifted.results[0].message, "You gather a wolf hide.");
        let arms = world.observe(near).unwrap().text;
        assert!(arms.contains("You have a wolf hide."), "{arms}");
        assert!(arms.contains("A deer hide lies here."), "{arms}");
        assert!(world
            .observe(near)
            .unwrap()
            .events
            .iter()
            .all(|event| { !event.text.contains("wolf hide") }));
        assert!(world
            .observe(seen)
            .unwrap()
            .events
            .iter()
            .any(|event| { event.text == "Near gathers a wolf hide." }));
        assert!(world
            .observe(quiet)
            .unwrap()
            .events
            .iter()
            .all(|event| { !event.text.contains("wolf hide") }));
        let deer = world.step(&[intent(
            "deer",
            near,
            super::Attempt::Gather(super::Material::Hide),
        )]);
        assert_eq!(deer.results[0].message, "You gather a deer hide.");
        assert!(world
            .observe(seen)
            .unwrap()
            .events
            .iter()
            .any(|event| { event.text == "Near gathers a deer hide." }));
        assert!(world
            .observe(near)
            .unwrap()
            .events
            .iter()
            .all(|event| { !event.text.contains("gathers a deer hide") }));
        let again = world.mint_item(super::Material::Hide, 0, cell, None, super::Role::Loose);
        world.items.get_mut(&again).unwrap().wolf = true;
        let race = world.step(&[
            intent(
                "near-race",
                near,
                super::Attempt::Gather(super::Material::Hide),
            ),
            intent(
                "seen-race",
                seen,
                super::Attempt::Gather(super::Material::Hide),
            ),
        ]);
        let win = race.results.iter().find(|result| result.success).unwrap();
        let lose = race.results.iter().find(|result| !result.success).unwrap();
        assert_eq!(win.message, "You gather a wolf hide.");
        assert_eq!(
            lose.message,
            "Your hand meets someone else's on the same wolf hide."
        );
        let heard = format!("{} gathers a wolf hide.", world.name_of(win.actor_id));
        assert!(world
            .observe(lose.actor_id)
            .unwrap()
            .events
            .iter()
            .any(|event| event.text == heard));
        assert!(world
            .observe(win.actor_id)
            .unwrap()
            .events
            .iter()
            .all(|event| event.text != heard && !event.text.contains("same wolf hide")));
        for id in [near, seen, quiet] {
            assert!(world.observe(id).unwrap().events.iter().all(|event| {
                !event.text.contains("reaches for a wolf hide")
                    && event.text != "Your hand meets someone else's on the same wolf hide."
            }));
        }
        assert!(world
            .observe(quiet)
            .unwrap()
            .events
            .iter()
            .all(|event| { event.text != heard }));
    }
}

#[cfg(test)]
mod berry_return {
    use super::{Attempt, Material, World};

    fn intent(id: &str, actor: u32, attempt: Attempt) -> super::Intent {
        super::Intent {
            command_id: id.into(),
            actor_id: actor,
            attempt,
        }
    }

    fn empty_patch(world: &mut World, actor: u32, material: Material) {
        let cell = world.actor(actor).unwrap().cell;
        let mut n = 0;
        while world.pile_count(cell, material) > 0 {
            n += 1;
            let took = world.step(&[intent(
                &format!("gather-{n}"),
                actor,
                Attempt::Gather(material),
            )]);
            assert!(took.results[0].success, "{}", took.results[0].message);
        }
    }

    #[test]
    fn poison_berries_return_after_they_are_gathered() {
        let mut world = World::valley(crate::tuning::Tuning::evening(), 1);
        let near = world.spawn_at("Near", "n", 4, 1).unwrap();
        let seen = world.spawn_at("Seen", "s", 4, 2).unwrap();
        let quiet = world.spawn_at("Quiet", "q", 0, 4).unwrap();
        let here = world.observe(near).unwrap().text;
        assert!(here.contains("Dark berries lie here"), "{here}");
        assert!(!here.contains("poison"), "{here}");
        assert!(world.observe(seen).unwrap().text.contains("dark berries"));
        assert!(!world.observe(seen).unwrap().text.contains("poison"));
        empty_patch(&mut world, near, Material::BitterBerry);
        let gone = world.observe(near).unwrap().text;
        assert!(
            gone.contains("The dark berries that were here are gone."),
            "{gone}"
        );
        assert!(!gone.contains("Dark berries lie here"), "{gone}");
        let south = world.observe(seen).unwrap().text;
        assert!(
            south.contains("The dark berries to the north are gone."),
            "{south}"
        );
        assert!(!world.observe(quiet).unwrap().text.contains("dark berries"));
        assert!(!world.observe(quiet).unwrap().text.contains("poison"));
        let ticks = world.tuning.berry_ticks;
        for _ in 0..ticks - 1 {
            world.step(&[]);
        }
        assert!(world
            .observe(near)
            .unwrap()
            .text
            .contains("The dark berries that were here are gone."));
        world.step(&[]);
        let back = "Dark berries lie here again.";
        let again = world.observe(near).unwrap();
        assert!(
            again.text.contains("Dark berries lie here"),
            "{}",
            again.text
        );
        assert!(!again.text.contains("poison"), "{}", again.text);
        assert!(!again.text.contains("are gone"), "{}", again.text);
        assert!(again.events.iter().any(|event| event.text == back));
        let seen_back = world.observe(seen).unwrap();
        assert!(
            seen_back.text.contains("dark berries"),
            "{}",
            seen_back.text
        );
        assert!(!seen_back.text.contains("are gone"), "{}", seen_back.text);
        assert!(seen_back.events.iter().any(|event| event.text == back));
        assert!(world
            .observe(quiet)
            .unwrap()
            .events
            .iter()
            .all(|event| event.text != back));
        assert!(!world.observe(quiet).unwrap().text.contains("dark berries"));
        assert!(!world.observe(quiet).unwrap().text.contains("poison"));
        let second = world.step(&[intent(
            "second",
            near,
            Attempt::Gather(Material::BitterBerry),
        )]);
        assert_eq!(
            second.results[0].message,
            "You gather a handful of dark berries."
        );
        assert_eq!(
            world.pile_count(world.actor(near).unwrap().cell, Material::BitterBerry),
            1
        );
    }

    #[test]
    fn berries_the_deer_ate_grow_back_under_the_rock() {
        let mut world = World::valley(crate::tuning::Tuning::evening(), 1);
        let near = world.spawn_at("Near", "n", 6, 1).unwrap();
        let seen = world.spawn_at("Seen", "s", 5, 1).unwrap();
        let quiet = world.spawn_at("Quiet", "q", 0, 4).unwrap();
        let rock = world.actor(near).unwrap().cell;
        let stocked = world.observe(near).unwrap().text;
        assert!(stocked.contains("Berries lie here"), "{stocked}");
        let deer = world.deer.as_mut().unwrap();
        deer.cell = rock;
        deer.route_at = 6;
        deer.held = false;
        for _ in 0..4 {
            world.step(&[]);
        }
        assert_eq!(world.pile_count(rock, Material::Berry), 0);
        assert!(world.deer_ate_berries.contains(&rock));
        assert!(world.berry_returns_at.contains_key(&rock));
        let gone = world.observe(near).unwrap().text;
        assert!(
            gone.contains(
                "A handful of berries lies here. It is good to eat raw. A deer ate the rest."
            ),
            "{gone}"
        );
        assert!(!gone.contains("are gone"), "{gone}");
        assert!(!gone.contains("after eating the berries"), "{gone}");
        assert!(!gone.contains("Berries lie here"), "{gone}");
        world.deer = None;
        let ticks = world.tuning.berry_ticks;
        for _ in 0..ticks {
            world.step(&[]);
        }
        let back = "Berries lie here again.";
        let again = world.observe(near).unwrap();
        assert!(again.text.contains("Berries lie here"), "{}", again.text);
        assert!(!again.text.contains("are gone"), "{}", again.text);
        assert!(again.events.iter().any(|event| event.text == back));
        assert!(world
            .observe(seen)
            .unwrap()
            .events
            .iter()
            .any(|event| event.text == back));
        assert!(world
            .observe(quiet)
            .unwrap()
            .events
            .iter()
            .all(|event| event.text != back));
        let second = world.step(&[intent("second", near, Attempt::Gather(Material::Berry))]);
        assert_eq!(
            second.results[0].message,
            "You gather a handful of berries."
        );
        assert_eq!(world.pile_count(rock, Material::Berry), 3);
    }
}

#[cfg(test)]
mod mushroom_return {
    use super::{Attempt, Material, World};

    fn intent(id: &str, actor: u32, attempt: Attempt) -> super::Intent {
        super::Intent {
            command_id: id.into(),
            actor_id: actor,
            attempt,
        }
    }

    #[test]
    fn mushrooms_return_after_they_are_gathered() {
        let mut world = World::valley(crate::tuning::Tuning::evening(), 1);
        let near = world.spawn_at("Near", "n", 3, 1).unwrap();
        let seen = world.spawn_at("Seen", "s", 3, 2).unwrap();
        let quiet = world.spawn_at("Quiet", "q", 0, 4).unwrap();
        let here = world.observe(near).unwrap().text;
        assert!(here.contains("Mushrooms lie here"), "{here}");
        assert!(world.observe(seen).unwrap().text.contains("mushrooms"));
        let mut n = 0;
        while world.pile_count(world.actor(near).unwrap().cell, Material::Mushroom) > 0 {
            n += 1;
            let took = world.step(&[intent(
                &format!("gather-{n}"),
                near,
                Attempt::Gather(Material::Mushroom),
            )]);
            assert_eq!(took.results[0].message, "You gather a mushroom.");
        }
        let gone = world.observe(near).unwrap().text;
        assert!(
            gone.contains("The mushrooms that were here are gone."),
            "{gone}"
        );
        assert!(!gone.contains("Mushrooms lie here"), "{gone}");
        let south = world.observe(seen).unwrap().text;
        assert!(
            south.contains("The mushrooms to the north are gone."),
            "{south}"
        );
        let examined = world.examine(seen, super::ExamineTarget::Mushrooms);
        assert!(
            examined.contains("The mushrooms that were to the north are gone."),
            "{examined}"
        );
        assert!(!world.observe(quiet).unwrap().text.contains("mushroom"));
        let ticks = world.tuning.mushroom_ticks;
        for _ in 0..ticks - 1 {
            world.step(&[]);
        }
        assert!(world
            .observe(near)
            .unwrap()
            .text
            .contains("The mushrooms that were here are gone."));
        world.step(&[]);
        let back = "Mushrooms lie here again.";
        let again = world.observe(near).unwrap();
        assert!(again.text.contains("Mushrooms lie here"), "{}", again.text);
        assert!(!again.text.contains("are gone"), "{}", again.text);
        assert!(again.events.iter().any(|event| event.text == back));
        let seen_back = world.observe(seen).unwrap();
        assert!(seen_back.text.contains("mushrooms"), "{}", seen_back.text);
        assert!(!seen_back.text.contains("are gone"), "{}", seen_back.text);
        assert!(seen_back.events.iter().any(|event| event.text == back));
        assert!(world
            .observe(quiet)
            .unwrap()
            .events
            .iter()
            .all(|event| event.text != back));
        assert!(!world.observe(quiet).unwrap().text.contains("mushroom"));
        let second = world.step(&[intent("second", near, Attempt::Gather(Material::Mushroom))]);
        assert_eq!(second.results[0].message, "You gather a mushroom.");
        assert_eq!(
            world.pile_count(world.actor(near).unwrap().cell, Material::Mushroom),
            1
        );
    }
}

#[cfg(test)]
mod bark_return {
    use super::{Attempt, Material, World};

    fn intent(id: &str, actor: u32, attempt: Attempt) -> super::Intent {
        super::Intent {
            command_id: id.into(),
            actor_id: actor,
            attempt,
        }
    }

    #[test]
    fn bark_under_the_rock_returns_after_it_is_gathered() {
        let mut world = World::valley(crate::tuning::Tuning::evening(), 1);
        let near = world.spawn_at("Near", "n", 6, 1).unwrap();
        let seen = world.spawn_at("Seen", "s", 5, 1).unwrap();
        let quiet = world.spawn_at("Quiet", "q", 0, 4).unwrap();
        let rock = world.actor(near).unwrap().cell;
        let slope = super::cell_id(1, 0).unwrap();
        let stocked = world.observe(near).unwrap().text;
        assert!(stocked.contains("A pile of bark lies here."), "{stocked}");
        while world.pile_count(slope, Material::Bark) > 0 {
            world.take_from_pile(slope, Material::Bark, 1);
        }
        let mut n = 0;
        while world.pile_count(rock, Material::Bark) > 0 {
            n += 1;
            let took = world.step(&[intent(
                &format!("gather-{n}"),
                near,
                Attempt::Gather(Material::Bark),
            )]);
            assert_eq!(took.results[0].message, "You gather a sheet of bark.");
        }
        assert!(!world.bark_returns_at.contains_key(&slope));
        assert_eq!(world.pile_count(slope, Material::Bark), 0);
        let gone = world.observe(near).unwrap().text;
        assert!(gone.contains("There is no bark under the rock."), "{gone}");
        assert!(!gone.contains("A pile of bark lies here."), "{gone}");
        assert!(!gone.contains("A sheet of bark lies here."), "{gone}");
        let west = world.observe(seen).unwrap().text;
        assert!(
            west.contains("There is no bark under the rock to the east."),
            "{west}"
        );
        let examined = world.examine(seen, super::ExamineTarget::Bark);
        assert!(
            examined.contains("There is no bark under the rock to the east."),
            "{examined}"
        );
        assert!(!world
            .observe(quiet)
            .unwrap()
            .text
            .contains("bark under the rock"));
        let ticks = world.tuning.bark_pile_ticks;
        for _ in 0..ticks - 1 {
            world.step(&[]);
        }
        assert!(world
            .observe(near)
            .unwrap()
            .text
            .contains("There is no bark under the rock."));
        world.step(&[]);
        let back = "Bark lies under the rock again.";
        let again = world.observe(near).unwrap();
        assert!(
            again.text.contains("A pile of bark lies here."),
            "{}",
            again.text
        );
        assert!(
            !again.text.contains("There is no bark under the rock."),
            "{}",
            again.text
        );
        assert_eq!(world.pile_count(rock, Material::Bark), 2);
        assert!(world.is_dry(world.pile_moisture(rock, Material::Bark)));
        assert!(again.events.iter().any(|event| event.text == back));
        let seen_back = world.observe(seen).unwrap();
        assert!(
            seen_back.text.contains("a pile of bark"),
            "{}",
            seen_back.text
        );
        assert!(
            !seen_back.text.contains("no bark under the rock"),
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
        assert_eq!(world.pile_count(slope, Material::Bark), 0);
        let second = world.step(&[intent("second", near, Attempt::Gather(Material::Bark))]);
        assert_eq!(second.results[0].message, "You gather a sheet of bark.");
        assert_eq!(world.pile_count(rock, Material::Bark), 1);
        assert!(world
            .observe(near)
            .unwrap()
            .text
            .contains("A sheet of bark lies here."));
    }

    #[test]
    fn using_the_last_sheet_under_the_rock_starts_the_same_wait() {
        let mut world = World::valley(crate::tuning::Tuning::evening(), 1);
        let near = world.spawn_at("Near", "n", 6, 1).unwrap();
        let rock = world.actor(near).unwrap().cell;
        let first = world.step(&[intent("weave-1", near, Attempt::WeaveBasket)]);
        assert!(first.results[0].success, "{}", first.results[0].message);
        assert_eq!(world.pile_count(rock, Material::Bark), 1);
        assert!(!world.bark_returns_at.contains_key(&rock));
        let light = world.step(&[intent("light", near, Attempt::Light)]);
        assert!(light.results[0].success, "{}", light.results[0].message);
        let gone = world.observe(near).unwrap().text;
        assert!(gone.contains("There is no bark under the rock."), "{gone}");
        assert!(world.bark_returns_at.contains_key(&rock));
        assert!(!world
            .bark_returns_at
            .contains_key(&super::cell_id(1, 0).unwrap()));
    }
}

#[cfg(test)]
mod bark_arms {
    use super::{ExamineTarget, Material, World};

    #[test]
    fn two_sheets_of_bark_stay_sheets() {
        let mut world = World::valley(crate::tuning::Tuning::evening(), 1);
        let near = world.spawn_at("Near", "n", 3, 4).unwrap();
        let seen = world.spawn_at("Seen", "s", 4, 4).unwrap();
        let quiet = world.spawn_at("Quiet", "q", 6, 4).unwrap();
        let cell = world.actor(near).unwrap().cell;
        world.mint_item(Material::Bark, 0, cell, Some(near), super::Role::Loose);
        let one = world.observe(near).unwrap().text;
        assert!(one.contains("You have a dry sheet of bark."), "{one}");
        assert!(!one.contains("2 bark"), "{one}");
        assert!(!one.contains("carried by you (dry)"), "{one}");
        assert_eq!(
            world.examine(near, ExamineTarget::Bark),
            "A dry sheet of bark is carried by you, in the open."
        );
        assert_eq!(
            world.examine(seen, ExamineTarget::Bark),
            "A dry sheet of bark is carried by Near, in the open."
        );
        assert_eq!(
            world.examine(quiet, ExamineTarget::Bark),
            "You see no bark from here."
        );
        world.mint_item(Material::Bark, 0, cell, Some(near), super::Role::Loose);
        let two = world.observe(near).unwrap().text;
        assert!(two.contains("You have two dry sheets of bark."), "{two}");
        assert!(!two.contains("2 bark"), "{two}");
        assert!(!two.contains("carried by you (dry)"), "{two}");
        assert_eq!(
            world.examine(near, ExamineTarget::Bark),
            "Two dry sheets of bark are carried by you, in the open."
        );
        assert_eq!(
            world.examine(seen, ExamineTarget::Bark),
            "Two dry sheets of bark are carried by Near, in the open."
        );
        assert_eq!(
            world.examine(quiet, ExamineTarget::Bark),
            "You see no bark from here."
        );
        world.mint_item(Material::Bark, 0, cell, Some(near), super::Role::Loose);
        let three = world.observe(near).unwrap().text;
        assert!(
            three.contains("You have three dry sheets of bark."),
            "{three}"
        );
        assert!(!three.contains("3 bark"), "{three}");
        assert!(!three.contains("carried by you (dry)"), "{three}");
        assert_eq!(
            world.examine(seen, ExamineTarget::Bark),
            "Three dry sheets of bark are carried by Near, in the open."
        );
    }
}

#[cfg(test)]
mod carve_paddle {
    use super::{Attempt, CellId, Dir, ExamineTarget, Intent, Material, World};

    fn intent(id: &str, actor: u32, attempt: Attempt) -> Intent {
        Intent {
            command_id: id.into(),
            actor_id: actor,
            attempt,
        }
    }

    fn at(x: i32, y: i32) -> CellId {
        super::cell_id(x, y).unwrap()
    }

    #[test]
    fn a_dry_branch_and_a_flake_or_knife_become_a_wooden_paddle() {
        let mut world = World::valley(crate::tuning::Tuning::evening(), 1);
        let near = world.spawn_at("Near", "n", 4, 3).unwrap();
        let seen = world.spawn_at("Seen", "s", 5, 3).unwrap();
        let quiet = world.spawn_at("Quiet", "q", 0, 4).unwrap();
        let cell = at(4, 3);
        assert_eq!(
            crate::parse::parse_attempt("carve paddle").unwrap(),
            Attempt::CarvePaddle
        );
        assert_eq!(
            crate::parse::parse_attempt("carve a paddle").unwrap(),
            Attempt::CarvePaddle
        );

        let bare = world.step(&[intent("bare", near, Attempt::CarvePaddle)]);
        assert_eq!(bare.results[0].message, "You have no dry branch to carve.");
        assert!(!bare.results[0].success);
        assert!(world
            .observe(near)
            .unwrap()
            .events
            .iter()
            .any(|event| event.text == "You have no dry branch to carve."));
        assert!(world
            .observe(seen)
            .unwrap()
            .events
            .iter()
            .all(|event| event.text != "You have no dry branch to carve."));
        assert!(world
            .observe(quiet)
            .unwrap()
            .events
            .iter()
            .all(|event| { !event.text.contains("carve") && !event.text.contains("paddle") }));

        world.mint_item(Material::Branch, 1000, cell, Some(near), super::Role::Loose);
        world.mint_item(Material::Flake, 0, cell, Some(near), super::Role::Loose);
        let wet = world.step(&[intent("wet", near, Attempt::CarvePaddle)]);
        assert_eq!(wet.results[0].message, "The branch is not dry.");
        assert_eq!(world.loose_ids(near, Material::Flake).len(), 1);
        assert_eq!(world.loose_ids(near, Material::Branch).len(), 1);

        world.mint_item(Material::Branch, 0, cell, Some(near), super::Role::Loose);
        let carved = world.step(&[intent("carve", near, Attempt::CarvePaddle)]);
        assert_eq!(
            carved.results[0].message,
            "You carve a dry branch into a wooden paddle with the flint flake."
        );
        assert!(carved.results[0].success);
        assert_eq!(world.loose_ids(near, Material::Flake).len(), 1);
        assert_eq!(world.loose_ids(near, Material::Paddle).len(), 1);
        assert_eq!(world.loose_ids(near, Material::Branch).len(), 1);
        let look = world.observe(near).unwrap();
        assert!(look.text.contains("a wooden paddle"), "{}", look.text);
        assert!(look.text.contains("a sharp flint flake"), "{}", look.text);
        assert!(look
            .events
            .iter()
            .all(|event| !event.text.contains("Near carves")));
        assert!(world.observe(seen).unwrap().events.iter().any(|event| {
            event.text == "Near carves a dry branch into a wooden paddle with the flint flake."
        }));
        assert!(world
            .observe(quiet)
            .unwrap()
            .events
            .iter()
            .all(|event| { !event.text.contains("paddle") && !event.text.contains("carves") }));
        let examined = world.examine(near, ExamineTarget::Paddle);
        assert!(
            examined.contains("A wooden paddle is carried by you"),
            "{examined}"
        );
        assert!(examined.contains("carved from a dry branch"), "{examined}");
        let seen_exam = world.examine(seen, ExamineTarget::Paddle);
        assert!(
            seen_exam.contains("A wooden paddle is carried by Near"),
            "{seen_exam}"
        );

        let dropped = world.step(&[intent("drop", near, Attempt::Drop(Material::Paddle))]);
        assert_eq!(
            dropped.results[0].message,
            "You drop a wooden paddle. It is in contact with the ground."
        );
        let lying = world.observe(near).unwrap();
        assert!(
            lying.text.contains("A wooden paddle lies here."),
            "{}",
            lying.text
        );
        assert!(
            !lying.text.contains("You have a wooden paddle"),
            "{}",
            lying.text
        );

        world.mint_item(Material::Knife, 0, cell, Some(near), super::Role::Loose);
        world.mint_item(Material::Branch, 0, cell, None, super::Role::Loose);
        let with_knife = world.step(&[intent("knife", near, Attempt::CarvePaddle)]);
        assert_eq!(
            with_knife.results[0].message,
            "You carve a dry branch into a wooden paddle with the iron knife."
        );
        assert_eq!(world.loose_ids(near, Material::Knife).len(), 1);
        assert_eq!(world.ground_ids(cell, Material::Branch).len(), 0);
        assert!(world.observe(seen).unwrap().events.iter().any(|event| {
            event.text == "Near carves a dry branch into a wooden paddle with the iron knife."
        }));

        let shade = world.spawn_at("Shade", "h", 6, 1).unwrap();
        let rock = at(6, 1);
        assert_eq!(world.pile_count(rock, Material::Branch), 4);
        world.mint_item(Material::Flake, 0, rock, Some(shade), super::Role::Loose);
        let from_pile = world.step(&[intent("pile", shade, Attempt::CarvePaddle)]);
        assert_eq!(
            from_pile.results[0].message,
            "You carve a dry branch into a wooden paddle with the flint flake."
        );
        assert!(world.pile_count(rock, Material::Branch) < 4);
        assert_eq!(world.loose_ids(shade, Material::Paddle).len(), 1);
        assert!(world
            .observe(shade)
            .unwrap()
            .text
            .contains("a wooden paddle"));
        assert!(world
            .observe(quiet)
            .unwrap()
            .events
            .iter()
            .all(|event| { !event.text.contains("Shade carves") }));
    }

    #[test]
    fn a_held_paddle_is_named_and_bare_hands_still_paddle() {
        let mut world = World::valley(crate::tuning::Tuning::evening(), 1);
        let near = world.spawn_at("Near", "n", 2, 2).unwrap();
        let seen = world.spawn_at("Seen", "s", 3, 3).unwrap();
        world.mint_item(Material::Canoe, 0, at(1, 2), None, super::Role::Loose);
        let boarded = world.step(&[intent("board", near, Attempt::Board)]);
        assert_eq!(boarded.results[0].message, "You board the dugout canoe.");
        let bare = world.step(&[intent("bare", near, Attempt::Paddle(Some(Dir::North)))]);
        assert_eq!(
            bare.results[0].message,
            "You paddle the dugout canoe north."
        );
        assert!(world
            .observe(near)
            .unwrap()
            .events
            .iter()
            .all(|event| !event.text.contains("Near paddles")));
        world.mint_item(
            Material::Paddle,
            0,
            at(1, 1),
            Some(near),
            super::Role::Loose,
        );
        let named = world.step(&[intent("named", near, Attempt::Paddle(Some(Dir::South)))]);
        assert_eq!(
            named.results[0].message,
            "You paddle the dugout canoe south with the wooden paddle."
        );
        assert!(world
            .observe(near)
            .unwrap()
            .events
            .iter()
            .all(|event| !event.text.contains("Near paddles")));
        assert!(world.observe(seen).unwrap().events.iter().any(|event| {
            event.text == "Near paddles the dugout canoe south with the wooden paddle."
        }));
        assert!(world
            .observe(near)
            .unwrap()
            .text
            .contains("a wooden paddle"));
    }
}

#[cfg(test)]
mod carve_hook {
    use super::{Attempt, CellId, Dir, ExamineTarget, Intent, Material, World};

    fn intent(id: &str, actor: u32, attempt: Attempt) -> Intent {
        Intent {
            command_id: id.into(),
            actor_id: actor,
            attempt,
        }
    }

    fn at(x: i32, y: i32) -> CellId {
        super::cell_id(x, y).unwrap()
    }

    fn say(world: &mut World, id: &str, actor: u32, attempt: Attempt) -> String {
        world.step(&[intent(id, actor, attempt)]).results[0]
            .message
            .clone()
    }

    #[test]
    fn a_bone_and_a_flake_or_knife_become_a_fish_hook() {
        let mut world = World::valley(crate::tuning::Tuning::evening(), 1);
        let near = world.spawn_at("Near", "n", 4, 3).unwrap();
        let seen = world.spawn_at("Seen", "s", 5, 3).unwrap();
        let quiet = world.spawn_at("Quiet", "q", 0, 4).unwrap();
        let cell = at(4, 3);
        assert_eq!(
            crate::parse::parse_attempt("carve hook").unwrap(),
            Attempt::CarveHook
        );
        assert_eq!(
            crate::parse::parse_attempt("carve a fish hook").unwrap(),
            Attempt::CarveHook
        );

        let bare = say(&mut world, "bare", near, Attempt::CarveHook);
        assert_eq!(bare, "You have no bone to carve.");
        assert!(world
            .observe(near)
            .unwrap()
            .events
            .iter()
            .any(|event| event.text == "You have no bone to carve."));
        assert!(world
            .observe(seen)
            .unwrap()
            .events
            .iter()
            .all(|event| { event.text != "You have no bone to carve." }));
        assert!(world.observe(quiet).unwrap().events.iter().all(|event| {
            !event.text.contains("hook") && !event.text.contains("bone to carve")
        }));

        world.mint_item(Material::Bone, 0, cell, None, super::Role::Loose);
        let no_tool = say(&mut world, "tool", near, Attempt::CarveHook);
        assert_eq!(
            no_tool,
            "You need a flint flake or an iron knife to carve a hook."
        );
        assert_eq!(world.ground_ids(cell, Material::Bone).len(), 1);
        assert!(world
            .observe(seen)
            .unwrap()
            .events
            .iter()
            .all(|event| { !event.text.contains("to carve a hook") }));

        world.mint_item(Material::Flake, 0, cell, Some(near), super::Role::Loose);
        let carved = say(&mut world, "carve", near, Attempt::CarveHook);
        assert_eq!(
            carved,
            "You carve a bone into a bone fish hook with the flint flake."
        );
        assert_eq!(world.ground_ids(cell, Material::Bone).len(), 0);
        assert_eq!(world.loose_ids(near, Material::Flake).len(), 1);
        assert_eq!(world.loose_ids(near, Material::Hook).len(), 1);
        let look = world.observe(near).unwrap();
        assert!(look.text.contains("a bone fish hook"), "{}", look.text);
        assert!(look.text.contains("a sharp flint flake"), "{}", look.text);
        assert!(look
            .events
            .iter()
            .all(|event| !event.text.contains("Near carves")));
        assert!(world.observe(seen).unwrap().events.iter().any(|event| {
            event.text == "Near carves a bone into a bone fish hook with the flint flake."
        }));
        assert!(world
            .observe(quiet)
            .unwrap()
            .events
            .iter()
            .all(|event| { !event.text.contains("hook") && !event.text.contains("carves") }));
        let examined = world.examine(near, ExamineTarget::Hook);
        assert!(
            examined.contains("A bone fish hook is carried by you"),
            "{examined}"
        );
        assert!(examined.contains("carved from a bone"), "{examined}");
        let seen_exam = world.examine(seen, ExamineTarget::Hook);
        assert!(
            seen_exam.contains("A bone fish hook is carried by Near"),
            "{seen_exam}"
        );

        let dropped = say(&mut world, "drop", near, Attempt::Drop(Material::Hook));
        assert_eq!(
            dropped,
            "You drop a bone fish hook. It is in contact with the ground."
        );
        let lying = world.observe(near).unwrap();
        assert!(
            lying.text.contains("A bone fish hook lies here."),
            "{}",
            lying.text
        );

        world.mint_item(Material::Knife, 0, cell, Some(near), super::Role::Loose);
        world.mint_item(Material::Bone, 0, cell, Some(near), super::Role::Loose);
        let with_knife = say(&mut world, "knife", near, Attempt::CarveHook);
        assert_eq!(
            with_knife,
            "You carve a bone into a bone fish hook with the iron knife."
        );
        assert_eq!(world.loose_ids(near, Material::Knife).len(), 1);
        assert_eq!(world.loose_ids(near, Material::Bone).len(), 0);
        assert!(world.observe(seen).unwrap().events.iter().any(|event| {
            event.text == "Near carves a bone into a bone fish hook with the iron knife."
        }));
    }

    #[test]
    fn a_butchered_bone_and_an_iron_knife_carve_a_hook_under_the_rock() {
        let mut world = World::valley(crate::tuning::Tuning::evening(), 1);
        let near = world.spawn_at("Near", "n", 4, 3).unwrap();
        let seen = world.spawn_at("Seen", "s", 5, 3).unwrap();
        let quiet = world.spawn_at("Quiet", "q", 3, 3).unwrap();
        assert_eq!(
            say(&mut world, "none", near, Attempt::CarveHook),
            "You have no bone to carve."
        );
        for n in 0..3 {
            let message = say(
                &mut world,
                &format!("n{n}"),
                near,
                Attempt::Move(Dir::North),
            );
            assert_eq!(message, "You walk north.");
        }
        assert_eq!(
            say(&mut world, "east", near, Attempt::Move(Dir::East)),
            "You walk east."
        );
        assert_eq!(world.actor_cell(near), Some((5, 0)));
        assert_eq!(
            say(&mut world, "ore", near, Attempt::Gather(Material::IronOre)),
            "You gather a lump of iron ore."
        );
        for i in 0..3 {
            let message = say(&mut world, &format!("w{i}"), near, Attempt::Move(Dir::West));
            assert_eq!(message, "You walk west.");
        }
        assert_eq!(world.actor_cell(near), Some((2, 0)));
        assert_eq!(
            say(&mut world, "stone", near, Attempt::Gather(Material::Stone)),
            "You gather a stone."
        );
        for i in 0..4 {
            let message = say(&mut world, &format!("e{i}"), near, Attempt::Move(Dir::East));
            assert_eq!(message, "You walk east.");
        }
        assert_eq!(
            say(&mut world, "south", near, Attempt::Move(Dir::South)),
            "You walk south, under the rock overhang, out of the wind."
        );
        assert_eq!(world.actor_cell(near), Some((6, 1)));
        assert_eq!(
            say(&mut world, "bark", near, Attempt::Gather(Material::Bark)),
            "You gather a sheet of bark."
        );
        assert_eq!(
            say(
                &mut world,
                "tinder",
                near,
                Attempt::Gather(Material::Branch)
            ),
            "You gather a branch."
        );
        assert_eq!(
            say(&mut world, "light", near, Attempt::Light),
            "You light a fire with the bark and the branch from the arms."
        );
        assert_eq!(
            say(&mut world, "haft", near, Attempt::Gather(Material::Branch)),
            "You gather a branch."
        );
        assert_eq!(
            say(&mut world, "bind", near, Attempt::BindSpear),
            "You bind a stone and a branch into a spear."
        );
        world.mint_item(
            Material::Charcoal,
            0,
            world.actor(near).unwrap().cell,
            Some(near),
            super::Role::Loose,
        );
        assert_eq!(
            say(&mut world, "feed", near, Attempt::Feed),
            "You feed a lump of charcoal into the fire. It burns hotter."
        );
        assert_eq!(
            say(&mut world, "smelt", near, Attempt::SmeltIron),
            "You set a lump of iron ore by the fire. It leaves your arms. It is not iron yet."
        );
        let mut bloom = 0;
        while !world
            .observe(near)
            .unwrap()
            .text
            .contains("A bloom of iron lies here.")
        {
            world.step(&[]);
            bloom += 1;
            assert!(bloom < 8, "the ore should become a bloom");
        }
        assert_eq!(
            say(&mut world, "bloom", near, Attempt::Gather(Material::Iron)),
            "You gather a bloom of iron."
        );
        assert_eq!(
            say(&mut world, "hammer", near, Attempt::HammerIron),
            "You hammer the bloom of iron into an iron knife at the fire."
        );
        let mut waits = 0;
        while !world
            .observe(near)
            .unwrap()
            .text
            .contains("a spear with a hardened point")
        {
            world.step(&[]);
            waits += 1;
            assert!(waits < 8, "the point should harden under the rock");
        }
        let mut found = 0;
        while world.deer_cell() != Some((6, 1)) {
            world.step(&[]);
            found += 1;
            assert!(
                found < 24,
                "deer at {:?} after the knife is in hand",
                world.deer_cell()
            );
        }
        assert_eq!(
            say(&mut world, "hunt", near, Attempt::Hunt),
            "You bring the deer down. A carcass lies where it stood."
        );
        assert_eq!(
            say(&mut world, "butcher", near, Attempt::Butcher),
            "You butcher the carcass with the iron knife into three portions of raw meat, a hide, and a bone."
        );
        assert!(world
            .observe(near)
            .unwrap()
            .text
            .contains("A bone lies here."));
        let carved = say(&mut world, "hook", near, Attempt::CarveHook);
        assert_eq!(
            carved,
            "You carve a bone into a bone fish hook with the iron knife."
        );
        let look = world.observe(near).unwrap();
        assert!(look.text.contains("a bone fish hook"), "{}", look.text);
        assert!(look.text.contains("an iron knife"), "{}", look.text);
        assert!(!look.text.contains("A bone lies here."), "{}", look.text);
        assert!(look
            .events
            .iter()
            .all(|event| !event.text.contains("Near carves")));
        assert!(world.observe(seen).unwrap().events.iter().any(|event| {
            event.text == "Near carves a bone into a bone fish hook with the iron knife."
        }));
        assert!(world.observe(quiet).unwrap().events.iter().all(|event| {
            !event.text.contains("fish hook") && !event.text.contains("Near carves")
        }));
        let dropped = say(&mut world, "drop", near, Attempt::Drop(Material::Hook));
        assert_eq!(
            dropped,
            "You drop a bone fish hook. It is in contact with the ground."
        );
        let seen_look = world.observe(seen).unwrap();
        assert!(
            seen_look
                .text
                .contains("A bone fish hook lies under the rock to the north and east."),
            "{}",
            seen_look.text
        );
        assert!(!world.observe(quiet).unwrap().text.contains("hook"));
    }
}

#[cfg(test)]
mod fish_hook {
    use super::{Attempt, CellId, Intent, Material, World};

    fn intent(id: &str, actor: u32, attempt: Attempt) -> Intent {
        Intent {
            command_id: id.into(),
            actor_id: actor,
            attempt,
        }
    }

    fn at(x: i32, y: i32) -> CellId {
        super::cell_id(x, y).unwrap()
    }

    fn say(world: &mut World, id: &str, actor: u32, attempt: Attempt) -> String {
        world.step(&[intent(id, actor, attempt)]).results[0]
            .message
            .clone()
    }

    #[test]
    fn a_bone_fish_hook_and_hide_strips_take_a_fish_from_the_brook() {
        let mut world = World::valley(crate::tuning::Tuning::evening(), 1);
        let near = world.spawn_at("Near", "n", 2, 2).unwrap();
        let seen = world.spawn_at("Seen", "s", 3, 3).unwrap();
        let heard = world.spawn_at("Heard", "h", 5, 3).unwrap();
        let quiet = world.spawn_at("Quiet", "q", 6, 4).unwrap();
        let dry = world.spawn_at("Dry", "d", 4, 0).unwrap();
        assert_eq!(
            crate::parse::parse_attempt("fish with hook").unwrap(),
            Attempt::FishHook
        );
        assert_eq!(crate::parse::parse_attempt("fish").unwrap(), Attempt::Fish);
        let bank = at(2, 2);
        world.mint_item(Material::Hook, 0, bank, Some(dry), super::Role::Loose);
        world.mint_item(Material::Cord, 0, bank, Some(dry), super::Role::Loose);
        assert_eq!(
            say(&mut world, "inland", dry, Attempt::FishHook),
            "The brook is not here."
        );
        assert_eq!(world.loose_ids(dry, Material::Cord).len(), 1);
        assert!(world
            .observe(dry)
            .unwrap()
            .events
            .iter()
            .all(|event| { event.text != "The brook is not here." }));
        assert!(world
            .observe(seen)
            .unwrap()
            .events
            .iter()
            .all(|event| { !event.text.contains("brook is not here") }));

        assert_eq!(
            say(&mut world, "nohook", near, Attempt::FishHook),
            "You are not carrying a bone fish hook."
        );
        assert!(world
            .observe(near)
            .unwrap()
            .events
            .iter()
            .all(|event| { event.text != "You are not carrying a bone fish hook." }));
        assert!(world
            .observe(seen)
            .unwrap()
            .events
            .iter()
            .all(|event| { !event.text.contains("bone fish hook") }));

        world.mint_item(Material::Hook, 0, bank, Some(near), super::Role::Loose);
        assert_eq!(
            say(&mut world, "noline", near, Attempt::FishHook),
            "You have no hide strips for a line."
        );
        assert_eq!(world.loose_ids(near, Material::Hook).len(), 1);
        assert!(world
            .observe(near)
            .unwrap()
            .events
            .iter()
            .all(|event| { event.text != "You have no hide strips for a line." }));
        assert!(world
            .observe(seen)
            .unwrap()
            .events
            .iter()
            .all(|event| { !event.text.contains("hide strips") }));

        world.mint_item(Material::Cord, 0, bank, Some(near), super::Role::Loose);
        world.mint_item(Material::Spear, 0, bank, Some(near), super::Role::Loose);
        assert_eq!(
            say(&mut world, "stone", near, Attempt::Fish),
            "You need a fire-hardened flint spear."
        );
        assert_eq!(world.loose_ids(near, Material::Cord).len(), 1);
        let caught = say(&mut world, "hook", near, Attempt::FishHook);
        assert_eq!(
            caught,
            "You catch a fish from the brook with the bone fish hook. The hide strips are spent."
        );
        assert_eq!(world.loose_ids(near, Material::Cord).len(), 0);
        assert_eq!(world.loose_ids(near, Material::Hook).len(), 1);
        assert_eq!(world.loose_ids(near, Material::Fish).len(), 1);
        assert_eq!(world.loose_ids(near, Material::Spear).len(), 1);
        let look = world.observe(near).unwrap();
        assert!(look.text.contains("a bone fish hook"), "{}", look.text);
        assert!(look.text.contains("raw fish"), "{}", look.text);
        assert!(look.events.iter().all(|event| {
            event.text
                != "You catch a fish from the brook with the bone fish hook. The hide strips are spent."
                && !event.text.contains("Near catches")
                && !event.text.contains("splash")
                && !event.text.contains("You catch")
        }));
        assert!(world.observe(seen).unwrap().events.iter().any(|event| {
            event.text
                == "Near catches a fish from the brook with the bone fish hook. The hide strips are spent."
        }));
        assert!(world
            .observe(seen)
            .unwrap()
            .events
            .iter()
            .all(|event| !event.text.contains("splash")));
        assert!(world
            .observe(heard)
            .unwrap()
            .events
            .iter()
            .any(|event| { event.text == "You hear a splash to the north and west." }));
        assert!(world
            .observe(heard)
            .unwrap()
            .events
            .iter()
            .all(|event| { !event.text.contains("catches a fish") }));
        assert!(world
            .observe(quiet)
            .unwrap()
            .events
            .iter()
            .all(|event| { !event.text.contains("fish") && !event.text.contains("splash") }));

        world.mint_item(Material::Cord, 0, bank, Some(near), super::Role::Loose);
        let gone = say(&mut world, "again", near, Attempt::FishHook);
        assert_eq!(gone, "The fish are gone from this stretch of the brook.");
        assert!(world
            .observe(near)
            .unwrap()
            .events
            .iter()
            .all(|event| { event.text != "The fish are gone from this stretch of the brook." }));
        assert_eq!(world.loose_ids(near, Material::Cord).len(), 1);
        assert_eq!(world.loose_ids(near, Material::Fish).len(), 1);
        assert!(world
            .observe(seen)
            .unwrap()
            .events
            .iter()
            .all(|event| { !event.text.contains("fish are gone") }));

        let other = world.spawn_at("Other", "o", 0, 3).unwrap();
        world.mint_item(Material::Hook, 0, at(0, 3), Some(other), super::Role::Loose);
        world.mint_item(Material::Cord, 0, at(0, 3), Some(other), super::Role::Loose);
        let bare = say(&mut world, "bare", other, Attempt::Fish);
        assert_eq!(
            bare,
            "You catch a fish from the brook with the bone fish hook. The hide strips are spent."
        );
        assert!(world.observe(other).unwrap().events.iter().all(|event| {
            !event.text.contains("You catch") && !event.text.contains("Other catches")
        }));
        assert!(world.aboard.is_empty());

        let aboard = world.spawn_at("Aboard", "a", 0, 1).unwrap();
        world.mint_item(Material::Canoe, 0, at(0, 1), None, super::Role::Loose);
        world.mint_item(
            Material::Hook,
            0,
            at(0, 1),
            Some(aboard),
            super::Role::Loose,
        );
        world.mint_item(
            Material::Cord,
            0,
            at(0, 1),
            Some(aboard),
            super::Role::Loose,
        );
        assert_eq!(
            say(&mut world, "board", aboard, Attempt::Board),
            "You board the dugout canoe."
        );
        let from_boat = say(&mut world, "boat", aboard, Attempt::FishHook);
        assert_eq!(
            from_boat,
            "You catch a fish from the brook with the bone fish hook. The hide strips are spent."
        );
        assert!(world.aboard.contains_key(&aboard));
        assert_eq!(world.actor_cell(aboard), Some((0, 1)));
        assert_eq!(world.loose_ids(aboard, Material::Hook).len(), 1);
        assert_eq!(world.loose_ids(aboard, Material::Fish).len(), 1);
    }

    #[test]
    fn from_the_rock_the_hook_reaches_the_brook_bank() {
        let mut world = World::valley(crate::tuning::Tuning::evening(), 1);
        let near = world.spawn_at("Near", "n", 6, 1).unwrap();
        let rock = at(6, 1);
        world.mint_item(Material::Spear, 0, rock, Some(near), super::Role::Loose);
        world.mint_item(Material::Knife, 0, rock, Some(near), super::Role::Loose);
        world.mint_item(Material::Hook, 0, rock, Some(near), super::Role::Loose);
        world.mint_item(Material::Hide, 0, rock, None, super::Role::Loose);
        for _ in 0..20 {
            world.step(&[]);
        }
        assert_eq!(
            say(&mut world, "dry", near, Attempt::FishHook),
            "The brook is not here."
        );
        assert!(world
            .observe(near)
            .unwrap()
            .events
            .iter()
            .all(|event| { event.text != "The brook is not here." }));
        assert_eq!(
            say(&mut world, "cut", near, Attempt::CutHide),
            "You cut the deer hide into strips with the iron knife."
        );
        assert_eq!(
            say(&mut world, "out", near, Attempt::Move(super::Dir::West)),
            "You walk west, out from under the rock overhang, into the wind and the rain."
        );
        for i in 0..2 {
            assert_eq!(
                say(
                    &mut world,
                    &format!("w{i}"),
                    near,
                    Attempt::Move(super::Dir::West)
                ),
                "You walk west."
            );
        }
        assert_eq!(
            say(&mut world, "ice", near, Attempt::Move(super::Dir::West)),
            "You walk west across the ice."
        );
        assert_eq!(
            say(&mut world, "south", near, Attempt::Move(super::Dir::South)),
            "You walk south."
        );
        assert_eq!(world.actor_cell(near), Some((2, 2)));
        assert_eq!(
            say(&mut world, "noline", near, Attempt::FishHook),
            "You have no hide strips for a line."
        );
        assert!(world
            .observe(near)
            .unwrap()
            .events
            .iter()
            .all(|event| { event.text != "You have no hide strips for a line." }));
        assert_eq!(
            say(&mut world, "drop", near, Attempt::Drop(Material::Hook)),
            "You drop a bone fish hook. It is in contact with the ground."
        );
        assert_eq!(
            say(&mut world, "nohook", near, Attempt::FishHook),
            "You are not carrying a bone fish hook."
        );
        assert!(world
            .observe(near)
            .unwrap()
            .events
            .iter()
            .all(|event| { event.text != "You are not carrying a bone fish hook." }));
        assert_eq!(
            say(&mut world, "pickup", near, Attempt::Gather(Material::Hook)),
            "You gather a bone fish hook."
        );
        assert_eq!(
            say(&mut world, "back", near, Attempt::Move(super::Dir::North)),
            "You walk north across the ice."
        );
        assert_eq!(
            say(&mut world, "off", near, Attempt::Move(super::Dir::East)),
            "You walk east."
        );
        for i in 0..2 {
            assert_eq!(
                say(
                    &mut world,
                    &format!("e{i}"),
                    near,
                    Attempt::Move(super::Dir::East)
                ),
                "You walk east."
            );
        }
        assert_eq!(
            say(&mut world, "in", near, Attempt::Move(super::Dir::East)),
            "You walk east, under the rock overhang, out of the wind and the rain."
        );
        assert_eq!(world.actor_cell(near), Some((6, 1)));
        assert_eq!(
            say(&mut world, "strips", near, Attempt::Gather(Material::Cord)),
            "You gather hide strips."
        );
        assert_eq!(
            say(&mut world, "out2", near, Attempt::Move(super::Dir::West)),
            "You walk west, out from under the rock overhang, into the wind and the rain."
        );
        for i in 0..2 {
            assert_eq!(
                say(
                    &mut world,
                    &format!("w2{i}"),
                    near,
                    Attempt::Move(super::Dir::West)
                ),
                "You walk west."
            );
        }
        assert_eq!(
            say(&mut world, "ice2", near, Attempt::Move(super::Dir::West)),
            "You walk west across the ice."
        );
        assert_eq!(
            say(&mut world, "south2", near, Attempt::Move(super::Dir::South)),
            "You walk south."
        );
        let caught = say(&mut world, "cast", near, Attempt::FishHook);
        assert_eq!(
            caught,
            "You catch a fish from the brook with the bone fish hook. The hide strips are spent."
        );
        let again = say(&mut world, "again", near, Attempt::FishHook);
        assert_eq!(again, "The fish are gone from this stretch of the brook.");
        assert!(world.observe(near).unwrap().events.iter().all(|event| {
            event.text != "The fish are gone from this stretch of the brook."
                && !event.text.contains("You catch")
        }));
        let look = world.observe(near).unwrap();
        assert!(look.text.contains("a bone fish hook"), "{}", look.text);
        assert!(look.text.contains("raw fish"), "{}", look.text);
        assert!(!look.text.contains("hide strips"), "{}", look.text);
    }
}

#[cfg(test)]
mod brook_south {
    use super::{Attempt, Material, Role, World};

    fn intent(id: &str, actor: u32, attempt: Attempt) -> super::Intent {
        super::Intent {
            command_id: id.into(),
            actor_id: actor,
            attempt,
        }
    }

    #[test]
    fn the_brook_runs_one_cell_farther_south() {
        let mut world = World::valley(crate::tuning::Tuning::evening(), 1);
        let water = super::cell_id(0, 3).unwrap();
        let bank = super::cell_id(1, 3).unwrap();
        assert_eq!(world.cell(water).region, super::Region::Brook);
        assert!(!world.cell(water).still);
        assert_ne!(world.cell(bank).region, super::Region::Brook);

        let near = world.spawn_at("Near", "n", 0, 2).unwrap();
        let seen = world.spawn_at("Seen", "s", 1, 3).unwrap();
        let quiet = world.spawn_at("Quiet", "q", 6, 4).unwrap();
        let south = world.spawn_at("South", "o", 0, 4).unwrap();

        let wade = world.step(&[intent("wade", near, Attempt::Move(super::Dir::South))]);
        assert_eq!(wade.results[0].message, "You wade south.");
        assert_eq!(world.actor_cell(near), Some((0, 3)));
        let here = world.observe(near).unwrap().text;
        assert!(here.contains("You are on the brook"), "{here}");
        assert!(here.contains("The water here is raw."), "{here}");
        assert!(
            here.contains("Reeds grow here, at the brook's edge."),
            "{here}"
        );
        let distant = world.observe(seen).unwrap().text;
        assert!(distant.contains("West: brook"), "{distant}");
        assert!(
            distant.contains("Reeds grow at the brook's edge to the west."),
            "{distant}"
        );
        let from_south = world.observe(south).unwrap().text;
        assert!(from_south.contains("North: brook"), "{from_south}");
        let named = world.examine(south, super::ExamineTarget::Water);
        assert!(
            named.contains("The brook to the north, in the open is running water."),
            "{named}"
        );
        let far = world.observe(quiet).unwrap();
        assert!(!far.text.contains("brook"), "{}", far.text);
        assert!(far
            .events
            .iter()
            .all(|event| !event.text.contains("wade") && !event.text.contains("brook")));

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
        let caught = world.step(&[intent("cast", near, Attempt::Fish)]);
        assert_eq!(
            caught.results[0].message,
            "You spear a fish from the brook."
        );
        let again = world.step(&[intent("again", near, Attempt::Fish)]);
        assert_eq!(
            again.results[0].message,
            "The fish are gone from this stretch of the brook."
        );
        let quiet_here = world.observe(near).unwrap().text;
        assert!(
            quiet_here.contains("The brook is quiet. The fish are gone."),
            "{quiet_here}"
        );
        let same_stretch = world.observe(seen).unwrap().text;
        assert!(
            same_stretch.contains("The brook is quiet. The fish are gone."),
            "{same_stretch}"
        );

        let back = world.step(&[intent("back", near, Attempt::Move(super::Dir::North))]);
        assert_eq!(back.results[0].message, "You wade north.");
        assert_eq!(world.actor_cell(near), Some((0, 2)));
        let old = world.step(&[intent("old", near, Attempt::Fish)]);
        assert_eq!(old.results[0].message, "You spear a fish from the brook.");

        world.mint_item(
            Material::Canoe,
            0,
            world.actor(near).unwrap().cell,
            None,
            Role::Loose,
        );
        let boarded = world.step(&[intent("board", near, Attempt::Board)]);
        assert_eq!(boarded.results[0].message, "You board the dugout canoe.");
        let paddled = world.step(&[intent(
            "paddle",
            near,
            Attempt::Paddle(Some(super::Dir::South)),
        )]);
        assert_eq!(
            paddled.results[0].message,
            "You paddle the dugout canoe south."
        );
        assert_eq!(world.actor_cell(near), Some((0, 3)));
        let aboard = world.observe(near).unwrap().text;
        assert!(aboard.contains("You are aboard the dugout canoe in the brook."));
        assert!(aboard.contains("The water here is raw."), "{aboard}");
        let stays = world.step(&[intent(
            "stay",
            near,
            Attempt::Paddle(Some(super::Dir::South)),
        )]);
        assert_eq!(stays.results[0].message, "The canoe stays in the brook.");
        let east = world.step(&[intent(
            "east",
            near,
            Attempt::Paddle(Some(super::Dir::East)),
        )]);
        assert_eq!(east.results[0].message, "The canoe stays in the brook.");
        let witness = world.observe(seen).unwrap().text;
        assert!(
            witness.contains("A dugout canoe floats in the brook to the west."),
            "{witness}"
        );
        let named = world.observe(south).unwrap().text;
        assert!(named.contains("North: brook"), "{named}");
        assert!(
            world.observe(quiet).unwrap().text.contains("brook") == false,
            "{}",
            world.observe(quiet).unwrap().text
        );
    }
}

#[cfg(test)]
mod pine_resin {
    use super::{Attempt, Material, World};

    fn intent(id: &str, actor: u32, attempt: Attempt) -> super::Intent {
        super::Intent {
            command_id: id.into(),
            actor_id: actor,
            attempt,
        }
    }

    #[test]
    fn the_pine_south_of_the_trail_gives_pitch_and_grows_it_again() {
        assert!(matches!(
            crate::parse::parse_attempt("gather resin").unwrap(),
            Attempt::Gather(Material::Resin)
        ));
        assert!(matches!(
            crate::parse::parse_attempt("gather pitch").unwrap(),
            Attempt::Gather(Material::Resin)
        ));
        let mut world = World::valley(crate::tuning::Tuning::evening(), 1);
        let near = world.spawn_at("Near", "n", 2, 4).unwrap();
        let seen = world.spawn_at("Seen", "s", 2, 3).unwrap();
        let quiet = world.spawn_at("Quiet", "q", 6, 4).unwrap();
        let here = world.observe(near).unwrap().text;
        assert!(
            here.contains("Lumps of pitch lie here. It is pine resin."),
            "{here}"
        );
        let south = world.observe(seen).unwrap().text;
        assert!(
            south.contains("Lumps of pitch lie to the south. It is pine resin."),
            "{south}"
        );
        assert!(south.contains("South: valley floor, pine resin"), "{south}");
        assert!(!world.observe(quiet).unwrap().text.contains("resin"));
        assert!(!world.observe(quiet).unwrap().text.contains("pitch"));
        let none = world.step(&[intent("none", quiet, Attempt::Gather(Material::Resin))]);
        assert_eq!(none.results[0].message, "There is no resin here to gather.");
        assert!(world
            .observe(quiet)
            .unwrap()
            .events
            .iter()
            .any(|event| event.text == "There is no resin here to gather."));
        assert!(world
            .observe(near)
            .unwrap()
            .events
            .iter()
            .all(|event| { event.text != "There is no resin here to gather." }));
        assert!(world
            .observe(seen)
            .unwrap()
            .events
            .iter()
            .all(|event| { event.text != "There is no resin here to gather." }));
        let one = world.step(&[intent("one", near, Attempt::Gather(Material::Resin))]);
        assert_eq!(one.results[0].message, "You gather a lump of pine resin.");
        let left = world.observe(near).unwrap();
        assert!(
            left.text
                .contains("A lump of pitch lies here. It is pine resin."),
            "{}",
            left.text
        );
        assert!(
            left.text.contains("You have a lump of pine resin."),
            "{}",
            left.text
        );
        assert!(left
            .events
            .iter()
            .all(|event| !event.text.contains("gathers")));
        assert!(world
            .observe(seen)
            .unwrap()
            .events
            .iter()
            .any(|event| event.text == "Near gathers a lump of pine resin."));
        assert!(
            world
                .observe(seen)
                .unwrap()
                .text
                .contains("A lump of pitch lies to the south. It is pine resin."),
            "{}",
            world.observe(seen).unwrap().text
        );
        assert!(world
            .observe(quiet)
            .unwrap()
            .events
            .iter()
            .all(|event| !event.text.contains("gathers")));
        let two = world.step(&[intent("two", near, Attempt::Gather(Material::Resin))]);
        assert_eq!(two.results[0].message, "You gather a lump of pine resin.");
        let bare = world.observe(near).unwrap();
        assert!(
            bare.text.contains("The pine here has no resin."),
            "{}",
            bare.text
        );
        assert!(
            bare.text.contains("You have two lumps of pine resin."),
            "{}",
            bare.text
        );
        assert!(!bare.text.contains("pitch lie"), "{}", bare.text);
        assert!(bare
            .events
            .iter()
            .all(|event| !event.text.contains("gathers")));
        let seen_bare = world.observe(seen).unwrap().text;
        assert!(
            seen_bare.contains("The pine to the south has no resin."),
            "{seen_bare}"
        );
        assert!(
            seen_bare.contains("South: valley floor, a bare pine"),
            "{seen_bare}"
        );
        let carried = world.examine(near, super::ExamineTarget::Resin);
        assert!(
            carried.contains("Two lumps of pine resin are carried by you"),
            "{carried}"
        );
        assert!(
            world
                .examine(seen, super::ExamineTarget::Resin)
                .contains("Two lumps of pine resin are carried by Near"),
            "{}",
            world.examine(seen, super::ExamineTarget::Resin)
        );
        assert_eq!(
            world.examine(quiet, super::ExamineTarget::Resin),
            "You see no resin from here."
        );
        let ticks = world.tuning.resin_ticks;
        for _ in 0..ticks - 1 {
            world.step(&[]);
        }
        assert!(world
            .observe(near)
            .unwrap()
            .text
            .contains("The pine here has no resin."));
        world.step(&[]);
        let grown = "Pine resin lies under the pine again.";
        let back = world.observe(near).unwrap();
        assert!(
            back.text
                .contains("Lumps of pitch lie here. It is pine resin."),
            "{}",
            back.text
        );
        assert!(back.events.iter().any(|event| event.text == grown));
        assert!(world
            .observe(seen)
            .unwrap()
            .events
            .iter()
            .any(|event| event.text == grown));
        assert!(world
            .observe(quiet)
            .unwrap()
            .events
            .iter()
            .all(|event| event.text != grown && !event.text.contains("pitch")));
        assert!(!world.observe(quiet).unwrap().text.contains("resin"));
    }

    #[test]
    fn pine_resin_in_the_arms_lights_a_fire_and_bark_still_does() {
        let mut world = World::valley(crate::tuning::Tuning::evening(), 1);
        let near = world.spawn_at("Near", "n", 4, 3).unwrap();
        let seen = world.spawn_at("Seen", "s", 3, 3).unwrap();
        let quiet = world.spawn_at("Quiet", "q", 0, 0).unwrap();
        let cell = world.actor(near).unwrap().cell;
        world.mint_item(Material::Resin, 0, cell, Some(near), super::Role::Loose);
        let bare = world.step(&[intent("bare", near, Attempt::Light)]);
        assert_eq!(
            bare.results[0].message,
            "You need a dry branch or a lump of charcoal for the fire."
        );
        assert_eq!(world.loose_ids(near, Material::Resin).len(), 1);
        assert!(world.fire_at(cell).is_none());
        assert!(world.observe(near).unwrap().events.iter().all(|event| {
            event.text != "You need a dry branch or a lump of charcoal for the fire."
        }));
        assert!(world.observe(seen).unwrap().events.iter().all(|event| {
            event.text != "You need a dry branch or a lump of charcoal for the fire."
        }));
        world.mint_item(Material::Branch, 0, cell, Some(near), super::Role::Loose);
        world.mint_item(Material::Bark, 0, cell, Some(near), super::Role::Loose);
        let lit = world.step(&[intent("light", near, Attempt::Light)]);
        assert_eq!(
            lit.results[0].message,
            "You light a fire with the pine resin and the branch from the arms."
        );
        assert!(world.fire_at(cell).is_some());
        assert!(world.loose_ids(near, Material::Resin).is_empty());
        assert!(world.loose_ids(near, Material::Branch).is_empty());
        assert_eq!(world.loose_ids(near, Material::Bark).len(), 1);
        let look = world.observe(near).unwrap();
        assert!(
            look.text.contains("A small fire burns here."),
            "{}",
            look.text
        );
        assert!(look.events.iter().all(|event| {
            !event.text.contains("lights a fire") && !event.text.contains("You light")
        }));
        assert!(world.observe(seen).unwrap().events.iter().any(|event| {
            event.text == "Near lights a fire with the pine resin and the branch from the arms."
        }));
        assert!(world.observe(quiet).unwrap().events.iter().all(|event| {
            !event.text.contains("lights a fire") && !event.text.contains("resin")
        }));
        let again = world.step(&[intent("again", near, Attempt::Light)]);
        assert_eq!(again.results[0].message, "A fire already burns here.");
        assert!(world
            .observe(near)
            .unwrap()
            .events
            .iter()
            .all(|event| { event.text != "A fire already burns here." }));
        world.raining = true;
        let open = world.spawn_at("Open", "o", 2, 4).unwrap();
        let pine = world.actor(open).unwrap().cell;
        world.mint_item(Material::Resin, 0, pine, Some(open), super::Role::Loose);
        world.mint_item(Material::Branch, 0, pine, Some(open), super::Role::Loose);
        let wet = world.step(&[intent("wet", open, Attempt::Light)]);
        assert_eq!(
            wet.results[0].message,
            "The rain puts the resin out before it catches."
        );
        assert_eq!(world.loose_ids(open, Material::Resin).len(), 1);
        assert!(world
            .observe(open)
            .unwrap()
            .events
            .iter()
            .all(|event| { event.text != "The rain puts the resin out before it catches." }));
        world.raining = false;
        let feet = world.spawn_at("Feet", "f", 5, 2).unwrap();
        let bank = world.actor(feet).unwrap().cell;
        world.mint_item(Material::Resin, 0, bank, Some(feet), super::Role::Loose);
        let from_pile = world.step(&[intent("pile", feet, Attempt::Light)]);
        assert_eq!(
            from_pile.results[0].message,
            "You light a fire with the pine resin from the arms and a dry branch from the pile at the feet."
        );
        assert!(world.loose_ids(feet, Material::Resin).is_empty());
        assert!(world.fire_at(bank).is_some());
        let elsewhere = world.spawn_at("Else", "e", 4, 4).unwrap();
        let floor = world.actor(elsewhere).unwrap().cell;
        world.mint_item(
            Material::Bark,
            0,
            floor,
            Some(elsewhere),
            super::Role::Loose,
        );
        world.mint_item(
            Material::Branch,
            0,
            floor,
            Some(elsewhere),
            super::Role::Loose,
        );
        let still = world.step(&[intent("still", elsewhere, Attempt::Light)]);
        assert_eq!(
            still.results[0].message,
            "You light a fire with the bark and the branch from the arms."
        );
        assert!(world.loose_ids(elsewhere, Material::Bark).is_empty());
        assert!(world
            .observe(elsewhere)
            .unwrap()
            .events
            .iter()
            .all(|event| {
                event.text != "You light a fire with the bark and the branch from the arms."
                    && event.text
                        != "Else lights a fire with the bark and the branch from the arms."
            }));
    }
}

#[cfg(test)]
mod willow {
    use super::{Attempt, Material, Role, World};

    fn intent(id: &str, actor: u32, attempt: Attempt) -> super::Intent {
        super::Intent {
            command_id: id.into(),
            actor_id: actor,
            attempt,
        }
    }

    #[test]
    fn the_willow_south_of_the_brook_gives_cordage_and_it_lines_a_hook() {
        assert!(matches!(
            crate::parse::parse_attempt("strip willow").unwrap(),
            Attempt::Gather(Material::Willow)
        ));
        assert!(matches!(
            crate::parse::parse_attempt("gather cordage").unwrap(),
            Attempt::Gather(Material::Willow)
        ));
        assert!(matches!(
            crate::parse::parse_attempt("gather hide strips").unwrap(),
            Attempt::Gather(Material::Cord)
        ));
        let mut world = World::valley(crate::tuning::Tuning::evening(), 1);
        let bank = super::cell_id(0, 4).unwrap();
        assert_eq!(world.pile_count(bank, Material::Willow), 2);
        let near = world.spawn_at("Near", "n", 0, 4).unwrap();
        let seen = world.spawn_at("Seen", "s", 1, 3).unwrap();
        let quiet = world.spawn_at("Quiet", "q", 6, 4).unwrap();
        let here = world.observe(near).unwrap().text;
        assert!(
            here.contains("Bundles of willow cordage hang here. They are withies from a willow."),
            "{here}"
        );
        assert!(
            here.contains("North: brook"),
            "the bank is the dry cell south of the brook: {here}"
        );
        let from_bank = world.observe(seen).unwrap().text;
        assert!(
            from_bank.contains(
                "Willow cordage hangs to the south and west. It is withies from a willow."
            ),
            "{from_bank}"
        );
        assert!(
            !world.observe(quiet).unwrap().text.contains("willow"),
            "{}",
            world.observe(quiet).unwrap().text
        );
        let none = world.step(&[intent("none", quiet, Attempt::Gather(Material::Willow))]);
        assert!(!none.results[0].success);
        assert_eq!(
            none.results[0].message,
            "There is no willow here to gather."
        );
        assert!(world
            .observe(quiet)
            .unwrap()
            .events
            .iter()
            .any(|event| { event.text == "There is no willow here to gather." }));
        assert!(world
            .observe(near)
            .unwrap()
            .events
            .iter()
            .all(|event| event.text != "There is no willow here to gather."));
        assert!(world
            .observe(seen)
            .unwrap()
            .events
            .iter()
            .all(|event| event.text != "There is no willow here to gather."));

        let one = world.step(&[intent("one", near, Attempt::Gather(Material::Willow))]);
        assert_eq!(
            one.results[0].message,
            "You gather a bundle of willow cordage."
        );
        assert!(world
            .observe(near)
            .unwrap()
            .events
            .iter()
            .all(|event| { !event.text.contains("gather") && !event.text.contains("cordage") }));
        assert!(world
            .observe(seen)
            .unwrap()
            .events
            .iter()
            .any(|event| { event.text == "Near gathers a bundle of willow cordage." }));
        assert!(world.observe(quiet).unwrap().events.iter().all(|event| {
            !event.text.contains("gathers") && !event.text.contains("bundle of willow")
        }));
        let arms = world.observe(near).unwrap().text;
        assert!(
            arms.contains("You have a bundle of willow cordage."),
            "{arms}"
        );
        assert!(
            arms.contains("A bundle of willow cordage hangs here. It is withies from a willow."),
            "{arms}"
        );

        let two = world.step(&[intent("two", near, Attempt::Gather(Material::Willow))]);
        assert_eq!(
            two.results[0].message,
            "You gather a bundle of willow cordage."
        );
        let bare = world.observe(near).unwrap().text;
        assert!(bare.contains("The willow here has no withies."), "{bare}");
        assert!(
            bare.contains("You have two bundles of willow cordage."),
            "{bare}"
        );
        assert!(world
            .observe(seen)
            .unwrap()
            .text
            .contains("The willow to the south and west has no withies."));

        let ticks = world.tuning.willow_ticks;
        for _ in 0..ticks {
            world.step(&[]);
        }
        let back = world.observe(near).unwrap().text;
        assert!(
            back.contains("Bundles of willow cordage hang here. They are withies from a willow."),
            "{back}"
        );
        assert!(world
            .observe(near)
            .unwrap()
            .events
            .iter()
            .any(|event| { event.text == "Willow cordage hangs on the willow again." }));
        assert!(world
            .observe(quiet)
            .unwrap()
            .events
            .iter()
            .all(|event| event.text != "Willow cordage hangs on the willow again."));

        world.mint_item(Material::Hook, 0, bank, Some(near), Role::Loose);
        let cast = world.step(&[intent("cast", near, Attempt::FishHook)]);
        assert_eq!(
            cast.results[0].message,
            "You catch a fish from the brook with the bone fish hook. The willow cordage is spent."
        );
        assert_eq!(world.loose_ids(near, Material::Willow).len(), 1);
        assert_eq!(world.loose_ids(near, Material::Hook).len(), 1);
        assert_eq!(world.loose_ids(near, Material::Fish).len(), 1);
        assert!(world.observe(near).unwrap().events.iter().all(|event| {
            !event.text.contains("You catch") && !event.text.contains("catches a fish")
        }));
        assert!(world.observe(seen).unwrap().events.iter().any(|event| {
            event.text
                == "Near catches a fish from the brook with the bone fish hook. The willow cordage is spent."
        }));
        assert!(world.observe(quiet).unwrap().events.iter().all(|event| {
            !event.text.contains("catches a fish")
                && !event.text.contains("willow cordage is spent")
        }));

        let again = world.step(&[intent("again", near, Attempt::FishHook)]);
        assert_eq!(
            again.results[0].message,
            "The fish are gone from this stretch of the brook."
        );
        let other = world.spawn_at("Other", "o", 1, 2).unwrap();
        let water = world.actor(other).unwrap().cell;
        world.mint_item(Material::Hook, 0, water, Some(other), Role::Loose);
        world.mint_item(Material::Cord, 0, water, Some(other), Role::Loose);
        world.mint_item(Material::Willow, 0, water, Some(other), Role::Loose);
        let prefer = world.step(&[intent("prefer", other, Attempt::FishHook)]);
        assert_eq!(
            prefer.results[0].message,
            "You catch a fish from the brook with the bone fish hook. The hide strips are spent."
        );
        assert!(world.loose_ids(other, Material::Cord).is_empty());
        assert_eq!(world.loose_ids(other, Material::Willow).len(), 1);
    }

    #[test]
    fn a_willow_bundle_sets_a_snare_and_hide_strips_are_spent_first() {
        let mut world = World::valley(crate::tuning::Tuning::evening(), 1);
        let floor = super::cell_id(6, 2).unwrap();
        let near = world.spawn_at("Near", "n", 6, 2).unwrap();
        let seen = world.spawn_at("Seen", "s", 6, 3).unwrap();
        let quiet = world.spawn_at("Quiet", "q", 0, 0).unwrap();
        world.mint_item(Material::Willow, 0, floor, Some(near), Role::Loose);
        world.mint_item(Material::Cord, 0, floor, Some(near), Role::Loose);
        let prefer = world.step(&[intent("prefer", near, Attempt::SetSnare)]);
        assert_eq!(prefer.results[0].message, "You set a snare of hide strips.");
        assert!(world.loose_ids(near, Material::Cord).is_empty());
        assert_eq!(world.loose_ids(near, Material::Willow).len(), 1);
        assert!(world
            .observe(near)
            .unwrap()
            .text
            .contains("A hide snare is set here."));
        for _ in 0..world.tuning.snare_ticks {
            world.step(&[]);
        }
        assert!(world
            .observe(near)
            .unwrap()
            .events
            .iter()
            .any(|event| { event.text == "The snare catches small game." }));
        assert_eq!(world.hare_cell(), Some((2, 4)));
        assert!(!world.snares.contains_key(&floor));

        world.mint_item(Material::Willow, 0, floor, Some(near), Role::Loose);
        let set = world.step(&[intent("set", near, Attempt::SetSnare)]);
        assert_eq!(set.results[0].message, "You set a snare of willow cordage.");
        assert_eq!(world.loose_ids(near, Material::Willow).len(), 1);
        assert!(world.observe(near).unwrap().events.iter().all(|event| {
            event.text != "You set a snare of willow cordage."
                && !event.text.contains("sets a snare")
        }));
        assert!(world
            .observe(seen)
            .unwrap()
            .events
            .iter()
            .any(|event| { event.text == "Near sets a snare of willow cordage." }));
        assert!(world
            .observe(quiet)
            .unwrap()
            .events
            .iter()
            .all(|event| { !event.text.contains("snare") }));
        let here = world.observe(near).unwrap().text;
        assert!(here.contains("A willow snare is set here."), "{here}");
        let from_seen = world.observe(seen).unwrap().text;
        assert!(
            from_seen.contains("A willow snare is set on the valley floor to the north."),
            "{from_seen}"
        );
        let examined = world.examine(near, super::ExamineTarget::Snare);
        assert!(
            examined.contains("A willow snare is set here")
                && examined.contains("It is made of willow cordage."),
            "{examined}"
        );
        let none = world.step(&[intent("none", quiet, Attempt::SetSnare)]);
        let refused = "You have no hide strips or willow cordage to set a snare.";
        assert_eq!(none.results[0].message, refused);
        assert!(world
            .observe(quiet)
            .unwrap()
            .events
            .iter()
            .all(|event| event.text != refused));
        assert!(world
            .observe(near)
            .unwrap()
            .events
            .iter()
            .all(|event| event.text != refused));
        for _ in 0..world.tuning.snare_ticks {
            world.step(&[]);
        }
        assert!(world
            .observe(near)
            .unwrap()
            .events
            .iter()
            .any(|event| { event.text == "The willow snare catches small game." }));
        assert!(world
            .observe(seen)
            .unwrap()
            .events
            .iter()
            .any(|event| { event.text == "The willow snare catches small game." }));
        assert_eq!(world.hare_cell(), Some((2, 4)));
        assert!(world
            .observe(quiet)
            .unwrap()
            .events
            .iter()
            .all(|event| { !event.text.contains("hare") }));
        let after = world.observe(near).unwrap().text;
        assert!(after.contains("raw meat lie"), "{after}");
        assert!(!after.contains("willow snare"), "{after}");
    }

    #[test]
    fn willow_cordage_binds_a_flint_spear_when_no_hide_strips_are_held() {
        let mut world = World::valley(crate::tuning::Tuning::evening(), 1);
        let floor = super::cell_id(6, 2).unwrap();
        let near = world.spawn_at("Near", "n", 6, 2).unwrap();
        let seen = world.spawn_at("Seen", "s", 6, 3).unwrap();
        let quiet = world.spawn_at("Quiet", "q", 0, 0).unwrap();
        world.mint_item(Material::Flake, 0, floor, Some(near), Role::Loose);
        world.mint_item(Material::Branch, 0, floor, Some(near), Role::Loose);
        world.mint_item(Material::Cord, 0, floor, Some(near), Role::Loose);
        world.mint_item(Material::Willow, 0, floor, Some(near), Role::Loose);
        let prefer = world.step(&[intent("prefer", near, Attempt::Haft)]);
        let hide =
            "You haft the flint flake and a dry branch into a spear. The tip is bound with hide.";
        assert_eq!(prefer.results[0].message, hide);
        assert!(world.loose_ids(near, Material::Cord).is_empty());
        assert_eq!(world.loose_ids(near, Material::Willow).len(), 1);
        assert!(world
            .observe(near)
            .unwrap()
            .text
            .contains("a flint tip bound with hide, not yet fire-hardened"));
        let hide_spear = world.loose_ids(near, Material::Spear)[0];
        world.items.remove(&hide_spear);

        world.mint_item(Material::Flake, 0, floor, Some(near), Role::Loose);
        world.mint_item(Material::Branch, 0, floor, Some(near), Role::Loose);
        let haft = world.step(&[intent("haft", near, Attempt::Haft)]);
        let yours =
            "You haft the flint flake and a dry branch into a spear. The tip is bound with willow cordage.";
        let witnessed =
            "Near hafts the flint flake and a dry branch into a spear. The tip is bound with willow cordage.";
        assert_eq!(haft.results[0].message, yours);
        assert!(world.loose_ids(near, Material::Willow).is_empty());
        assert!(world
            .observe(near)
            .unwrap()
            .events
            .iter()
            .all(|event| { event.text != yours && !event.text.contains("hafts the flint") }));
        assert!(world
            .observe(seen)
            .unwrap()
            .events
            .iter()
            .any(|event| event.text == witnessed));
        assert!(world
            .observe(quiet)
            .unwrap()
            .events
            .iter()
            .all(|event| !event.text.contains("willow cordage")));
        let here = world.observe(near).unwrap().text;
        assert!(
            here.contains(
                "a spear with a flint tip bound with willow cordage, not yet fire-hardened"
            ),
            "{here}"
        );
        let examined = world.examine(near, super::ExamineTarget::Spear);
        assert!(
            examined.contains(
                "A spear with a flint tip bound with willow cordage, not yet fire-hardened, is carried by you"
            ),
            "{examined}"
        );

        world.mint_item(Material::Flake, 0, floor, Some(near), Role::Loose);
        world.mint_item(Material::Branch, 0, floor, Some(near), Role::Loose);
        let bare = world.step(&[intent("bare", near, Attempt::Haft)]);
        assert_eq!(
            bare.results[0].message,
            "You haft the flint flake and a dry branch into a spear."
        );
        assert!(world
            .observe(near)
            .unwrap()
            .events
            .iter()
            .all(|event| { !event.text.contains("You haft the flint") }));
    }
}

#[cfg(test)]
mod otter {
    use super::{Attempt, Dir, World};

    fn intent(id: &str, actor: u32, attempt: Attempt) -> super::Intent {
        super::Intent {
            command_id: id.into(),
            actor_id: actor,
            attempt,
        }
    }

    #[test]
    fn an_otter_on_the_brook_is_named_in_sight_and_splashes_away() {
        let mut world = World::valley(crate::tuning::Tuning::evening(), 1);
        assert_eq!(world.otter_cell(), Some((1, 1)));
        let seen = world.spawn_at("Seen", "s", 3, 3).unwrap();
        let near = world.spawn_at("Near", "n", 4, 3).unwrap();
        let quiet = world.spawn_at("Quiet", "q", 6, 3).unwrap();
        let from_seen = world.observe(seen).unwrap().text;
        assert!(
            from_seen.contains("An otter is in the brook to the north and west."),
            "{from_seen}"
        );
        assert!(
            !world.observe(near).unwrap().text.contains("otter"),
            "{}",
            world.observe(near).unwrap().text
        );
        assert!(!world.observe(quiet).unwrap().text.contains("otter"));

        world.step(&[intent("west", seen, Attempt::Move(Dir::West))]);
        world.step(&[intent("west-2", seen, Attempt::Move(Dir::West))]);
        world.step(&[intent("north", seen, Attempt::Move(Dir::North))]);
        let step = world.step(&[intent("onto", seen, Attempt::Move(Dir::North))]);
        assert_eq!(step.results[0].message, "You wade north.");
        assert!(!step.results[0].message.contains("otter"));
        assert_eq!(world.otter_cell(), Some((1, 2)));
        let yours: Vec<_> = world
            .observe(seen)
            .unwrap()
            .events
            .iter()
            .filter(|event| event.text.contains("otter") || event.text.contains("splash"))
            .map(|event| event.text.clone())
            .collect();
        assert_eq!(
            yours,
            vec!["An otter splashes away to the south.".to_string()]
        );
        let heard: Vec<_> = world
            .observe(near)
            .unwrap()
            .events
            .iter()
            .filter(|event| event.text.contains("splash") || event.text.contains("otter"))
            .map(|event| event.text.clone())
            .collect();
        assert_eq!(
            heard,
            vec!["You hear a splash to the north and west.".to_string()]
        );
        assert!(world
            .observe(quiet)
            .unwrap()
            .events
            .iter()
            .all(|event| !event.text.contains("otter") && !event.text.contains("splash")));
        let after = world.observe(seen).unwrap().text;
        assert!(
            after.contains("An otter is in the brook to the south."),
            "{after}"
        );
        let beside = world.observe(seen).unwrap().text;
        assert!(
            beside.contains("an otter") || after.contains("An otter"),
            "{beside}"
        );

        for _ in 0..4 {
            world.step(&[]);
        }
        assert_eq!(world.otter_cell(), Some((1, 2)));

        let mut alone = World::valley(crate::tuning::Tuning::evening(), 1);
        for _ in 0..5 {
            alone.step(&[]);
        }
        assert_eq!(alone.otter_cell(), Some((1, 2)));
    }
}

#[cfg(test)]
mod beaver {
    use super::{Attempt, Dir, World};

    fn intent(id: &str, actor: u32, attempt: Attempt) -> super::Intent {
        super::Intent {
            command_id: id.into(),
            actor_id: actor,
            attempt,
        }
    }

    #[test]
    fn a_beaver_on_the_south_brook_is_named_and_splashes_to_the_bank() {
        let mut world = World::valley(crate::tuning::Tuning::evening(), 1);
        assert_eq!(world.beaver_cell(), Some((0, 3)));
        assert_eq!(world.otter_cell(), Some((1, 1)));
        let seen = world.spawn_at("Seen", "s", 2, 3).unwrap();
        let near = world.spawn_at("Near", "n", 3, 3).unwrap();
        let quiet = world.spawn_at("Quiet", "q", 5, 3).unwrap();
        let from_seen = world.observe(seen).unwrap();
        assert!(
            from_seen
                .text
                .contains("A beaver is in the brook to the west."),
            "{}",
            from_seen.text
        );
        assert!(from_seen.map.contains("~z"), "{}", from_seen.map);
        assert!(
            from_seen.legend.contains("z beaver"),
            "{}",
            from_seen.legend
        );
        assert!(!world.observe(near).unwrap().text.contains("beaver"));
        assert!(!world.observe(quiet).unwrap().text.contains("beaver"));

        let closer = world.step(&[intent("west", seen, Attempt::Move(Dir::West))]);
        assert_eq!(closer.results[0].message, "You walk west.");
        assert!(!closer.results[0].message.contains("beaver"));
        let beside = world.observe(seen).unwrap().text;
        assert!(
            beside.contains("A beaver is in the brook to the west."),
            "{beside}"
        );
        assert!(beside.contains("a beaver"), "{beside}");

        let step = world.step(&[intent("onto", seen, Attempt::Move(Dir::West))]);
        assert_eq!(step.results[0].message, "You wade west.");
        assert!(!step.results[0].message.contains("beaver"));
        assert_eq!(world.beaver_cell(), Some((0, 4)));
        let yours: Vec<_> = world
            .observe(seen)
            .unwrap()
            .events
            .iter()
            .filter(|event| event.text.contains("beaver") || event.text.contains("splash"))
            .map(|event| event.text.clone())
            .collect();
        assert_eq!(
            yours,
            vec!["A beaver splashes away to the south.".to_string()]
        );
        let heard: Vec<_> = world
            .observe(near)
            .unwrap()
            .events
            .iter()
            .filter(|event| event.text.contains("splash") || event.text.contains("beaver"))
            .map(|event| event.text.clone())
            .collect();
        assert_eq!(heard, vec!["You hear a splash to the west.".to_string()]);
        assert!(world
            .observe(quiet)
            .unwrap()
            .events
            .iter()
            .all(|event| { !event.text.contains("beaver") && !event.text.contains("splash") }));
        let after = world.observe(seen).unwrap().text;
        assert!(
            after.contains("A beaver is on the bank to the south."),
            "{after}"
        );
        assert!(
            world.observe(seen).unwrap().map.contains(".z")
                || world.observe(seen).unwrap().map.contains("~z"),
            "{}",
            world.observe(seen).unwrap().map
        );

        world.step(&[intent("off", seen, Attempt::Move(Dir::East))]);
        let mut back = 0;
        while world.beaver_cell() != Some((0, 3)) && back < 12 {
            world.step(&[]);
            back += 1;
        }
        assert_eq!(world.beaver_cell(), Some((0, 3)));
        assert!(world
            .observe(seen)
            .unwrap()
            .events
            .iter()
            .any(|event| { event.text == "A beaver swims north." }));
        assert!(world
            .observe(quiet)
            .unwrap()
            .events
            .iter()
            .all(|event| { !event.text.contains("beaver") && !event.text.contains("swims") }));
    }

    fn quiet_valley(seed: u64) -> World {
        let mut world = World::valley(crate::tuning::Tuning::evening(), seed);
        world.deer = None;
        world.otter = None;
        world.fox = None;
        world.hare = None;
        world
    }

    fn flint_spear(world: &mut World, actor: u32, cell: super::CellId) {
        let id = world.mint_item(
            super::Material::Spear,
            0,
            cell,
            Some(actor),
            super::Role::Loose,
        );
        let spear = world.items.get_mut(&id).unwrap();
        spear.hardened = true;
        spear.flint_tip = true;
    }

    #[test]
    fn a_spear_or_a_bow_brings_the_beaver_down() {
        let mut world = quiet_valley(1);
        let near = world.spawn_at("Near", "n", 2, 3).unwrap();
        let seen = world.spawn_at("Seen", "s", 2, 2).unwrap();
        let quiet = world.spawn_at("Quiet", "q", 5, 3).unwrap();
        let bare = world.step(&[intent("bare", near, Attempt::Hunt)]);
        assert_eq!(bare.results[0].message, "You are not carrying a spear.");
        assert!(world
            .observe(near)
            .unwrap()
            .events
            .iter()
            .any(|event| event.text == "You are not carrying a spear."));
        for person in [seen, quiet] {
            assert!(world
                .observe(person)
                .unwrap()
                .events
                .iter()
                .all(|event| event.text != "You are not carrying a spear."));
        }
        let cell = world.actor(near).unwrap().cell;
        flint_spear(&mut world, near, cell);
        let far = world.step(&[intent("far", near, Attempt::Hunt)]);
        assert_eq!(
            far.results[0].message,
            "The beaver is not close enough to hunt."
        );
        assert!(world
            .observe(near)
            .unwrap()
            .events
            .iter()
            .any(|event| { event.text == "The beaver is not close enough to hunt." }));
        for person in [seen, quiet] {
            assert!(world
                .observe(person)
                .unwrap()
                .events
                .iter()
                .all(|event| { !event.text.contains("beaver") && !event.text.contains("hunt") }));
        }

        let mut beside = quiet_valley(1);
        let hunter = beside.spawn_at("Near", "n", 0, 3).unwrap();
        let eyes = beside.spawn_at("Seen", "s", 2, 3).unwrap();
        let ears = beside.spawn_at("Quiet", "q", 5, 3).unwrap();
        let stood = beside.actor(hunter).unwrap().cell;
        flint_spear(&mut beside, hunter, stood);
        let hit = beside.step(&[intent("hunt", hunter, Attempt::Hunt)]);
        let yours = "You bring the beaver down with the flint spear.";
        let witnessed = "Near brings the beaver down with the flint spear.";
        assert_eq!(hit.results[0].message, yours);
        assert!(beside.beaver_cell().is_none());
        assert!(beside
            .observe(hunter)
            .unwrap()
            .events
            .iter()
            .all(|event| { event.text != yours && event.text != witnessed }));
        assert!(beside
            .observe(eyes)
            .unwrap()
            .events
            .iter()
            .any(|event| event.text == witnessed));
        assert!(beside
            .observe(ears)
            .unwrap()
            .events
            .iter()
            .all(|event| !event.text.contains("beaver")));
        let here = beside.observe(hunter).unwrap().text;
        assert!(here.contains("A beaver carcass lies here."), "{here}");
        let bare = beside.step(&[intent("empty", ears, Attempt::Butcher)]);
        assert_eq!(
            bare.results[0].message,
            "You need a flint spear or an iron knife."
        );
        assert!(beside
            .observe(ears)
            .unwrap()
            .events
            .iter()
            .any(|event| { event.text == "You need a flint spear or an iron knife." }));
        assert!(beside
            .observe(hunter)
            .unwrap()
            .events
            .iter()
            .all(|event| { event.text != "You need a flint spear or an iron knife." }));
        let cut = beside.step(&[intent("cut", hunter, Attempt::Butcher)]);
        let yours =
            "You butcher the beaver carcass into a portion of raw meat, a beaver hide, and a bone.";
        let witnessed =
            "Near butchers the beaver carcass into a portion of raw meat, a beaver hide, and a bone.";
        assert_eq!(cut.results[0].message, yours);
        assert!(cut.results[0].success);
        assert!(beside
            .observe(hunter)
            .unwrap()
            .events
            .iter()
            .all(|event| { event.text != yours && event.text != witnessed }));
        assert!(beside
            .observe(eyes)
            .unwrap()
            .events
            .iter()
            .any(|event| event.text == witnessed));
        assert!(beside.observe(ears).unwrap().events.iter().all(|event| {
            !event.text.contains("butcher") && !event.text.contains("beaver hide")
        }));
        let left = beside.observe(hunter).unwrap().text;
        assert!(left.contains("A portion of raw meat lies here."), "{left}");
        assert!(left.contains("A beaver hide lies here."), "{left}");
        assert!(left.contains("A bone lies here."), "{left}");
        assert!(!left.contains("A beaver carcass"), "{left}");
        assert!(!left.contains("A deer hide"), "{left}");
        let taken = beside.step(&[intent(
            "hide",
            hunter,
            Attempt::Gather(super::Material::Hide),
        )]);
        assert_eq!(taken.results[0].message, "You gather a beaver hide.");
        let wear = beside.step(&[intent("wear", hunter, Attempt::Wear)]);
        assert_eq!(wear.results[0].message, "You wear the beaver hide.");
        let arms = beside.observe(hunter).unwrap().text;
        assert!(arms.contains("a beaver hide"), "{arms}");
        assert!(!arms.contains("cloak"), "{arms}");

        let mut shot = quiet_valley(4);
        let archer = shot.spawn_at("Near", "n", 2, 3).unwrap();
        let watch = shot.spawn_at("Seen", "s", 2, 2).unwrap();
        let far_one = shot.spawn_at("Quiet", "q", 5, 3).unwrap();
        let place = shot.actor(archer).unwrap().cell;
        shot.mint_item(
            super::Material::Bow,
            0,
            place,
            Some(archer),
            super::Role::Loose,
        );
        shot.mint_item(
            super::Material::Flake,
            0,
            place,
            Some(archer),
            super::Role::Loose,
        );
        let loosed = shot.step(&[intent("bow", archer, Attempt::Hunt)]);
        let line = "You bring the beaver down with the bow. The flint flake is spent.";
        let named = "Near brings the beaver down with the bow. The flint flake is spent.";
        assert_eq!(loosed.results[0].message, line);
        assert!(shot.beaver_cell().is_none());
        assert_eq!(shot.loose_ids(archer, super::Material::Flake).len(), 0);
        assert!(shot
            .observe(archer)
            .unwrap()
            .events
            .iter()
            .all(|event| { event.text != line && event.text != named }));
        assert!(shot
            .observe(watch)
            .unwrap()
            .events
            .iter()
            .any(|event| event.text == named));
        assert!(shot
            .observe(far_one)
            .unwrap()
            .events
            .iter()
            .all(|event| !event.text.contains("beaver")));
        let bank = shot.spawn_at("Bank", "b", 0, 4).unwrap();
        let from_bank = shot.observe(bank).unwrap().text;
        assert!(from_bank.contains("a beaver carcass"), "{from_bank}");
    }

    #[test]
    fn an_iron_knife_butchers_the_beaver() {
        let mut world = quiet_valley(1);
        let near = world.spawn_at("Near", "n", 0, 3).unwrap();
        let seen = world.spawn_at("Seen", "s", 2, 3).unwrap();
        let quiet = world.spawn_at("Quiet", "q", 5, 3).unwrap();
        let cell = world.actor(near).unwrap().cell;
        world.beaver = None;
        let carcass = world.mint_item(super::Material::Carcass, 0, cell, None, super::Role::Loose);
        world.items.get_mut(&carcass).unwrap().beaver = true;
        world.mint_item(
            super::Material::Knife,
            0,
            cell,
            Some(near),
            super::Role::Loose,
        );
        let cut = world.step(&[intent("cut", near, Attempt::Butcher)]);
        let yours = "You butcher the beaver carcass with the iron knife into a portion of raw meat, a beaver hide, and a bone.";
        let witnessed = "Near butchers the beaver carcass with the iron knife into a portion of raw meat, a beaver hide, and a bone.";
        assert_eq!(cut.results[0].message, yours);
        assert!(world
            .observe(near)
            .unwrap()
            .events
            .iter()
            .all(|event| { event.text != yours && event.text != witnessed }));
        assert!(world
            .observe(seen)
            .unwrap()
            .events
            .iter()
            .any(|event| event.text == witnessed));
        assert!(world
            .observe(quiet)
            .unwrap()
            .events
            .iter()
            .all(|event| { !event.text.contains("beaver") }));
        let here = world.observe(near).unwrap().text;
        assert!(here.contains("A beaver hide lies here."), "{here}");
        assert!(here.contains("A portion of raw meat lies here."), "{here}");
        assert!(here.contains("A bone lies here."), "{here}");
        assert_eq!(world.loose_ids(near, super::Material::Knife).len(), 1);
    }

    #[test]
    fn a_beaver_hide_sews_into_a_cloak_that_sheds_the_wet() {
        let mut world = World::valley(crate::tuning::Tuning::evening(), 1);
        let near = world.spawn_at("Near", "n", 4, 3).unwrap();
        let seen = world.spawn_at("Seen", "s", 4, 3).unwrap();
        let quiet = world.spawn_at("Quiet", "q", 0, 4).unwrap();
        let cell = world.actor(near).unwrap().cell;
        let hide = world.mint_item(
            super::Material::Hide,
            0,
            cell,
            Some(near),
            super::Role::Loose,
        );
        world.items.get_mut(&hide).unwrap().beaver = true;
        let awl = world.mint_item(
            super::Material::Awl,
            0,
            cell,
            Some(near),
            super::Role::Loose,
        );
        world.items.get_mut(&awl).unwrap().bone = true;

        let early = world.step(&[intent("sew", near, Attempt::Sew)]);
        assert_eq!(early.results[0].message, "The beaver hide is not pierced.");
        assert!(world
            .observe(near)
            .unwrap()
            .events
            .iter()
            .all(|event| { event.text != "The beaver hide is not pierced." }));
        assert!(world
            .observe(seen)
            .unwrap()
            .events
            .iter()
            .all(|event| !event.text.contains("beaver")));

        let pierced = world.step(&[intent("pierce", near, Attempt::Pierce)]);
        assert_eq!(
            pierced.results[0].message,
            "You pierce the beaver hide with the bone awl. It is ready to sew."
        );
        assert!(world.observe(near).unwrap().events.iter().all(|event| {
            !event.text.contains("pierce") && !event.text.contains("beaver hide")
        }));
        assert!(world.observe(seen).unwrap().events.iter().any(|event| {
            event.text == "Near pierces the beaver hide with the bone awl. It is ready to sew."
        }));
        assert!(world
            .observe(quiet)
            .unwrap()
            .events
            .iter()
            .all(|event| !event.text.contains("beaver")));

        let sewn = world.step(&[intent("sew-cloak", near, Attempt::Sew)]);
        assert_eq!(
            sewn.results[0].message,
            "You sew the pierced beaver hide into a cloak."
        );
        let again = world.step(&[intent("again", near, Attempt::Sew)]);
        assert_eq!(
            again.results[0].message,
            "The beaver hide is already a cloak."
        );
        assert!(world
            .observe(seen)
            .unwrap()
            .events
            .iter()
            .all(|event| { event.text != "The beaver hide is already a cloak." }));
        let wore = world.step(&[intent("wear", near, Attempt::Wear)]);
        assert_eq!(
            wore.results[0].message,
            "You wear the sewn beaver-hide cloak."
        );
        let look = world.observe(near).unwrap().text;
        assert!(
            look.contains("The sewn beaver-hide cloak is on you. It sheds the rain and the brook."),
            "{look}"
        );
        assert!(look.contains("a sewn beaver-hide cloak"), "{look}");
        assert!(!look.contains("otter-hide"), "{look}");
        let held = world.examine(near, super::ExamineTarget::Hide);
        assert!(
            held.contains(
                "A sewn beaver-hide cloak is on you, in the open. It sheds the rain and the brook."
            ),
            "{held}"
        );
        assert!(world
            .observe(seen)
            .unwrap()
            .events
            .iter()
            .any(|event| { event.text == "Near sews the pierced beaver hide into a cloak." }));
        assert!(world
            .observe(seen)
            .unwrap()
            .events
            .iter()
            .any(|event| { event.text == "Near wears the sewn beaver-hide cloak." }));
        assert!(world.observe(near).unwrap().events.iter().all(|event| {
            !event.text.contains("sewn beaver-hide") && !event.text.contains("pierced beaver")
        }));

        let bare = world.spawn_at("Bare", "b", 1, 3).unwrap();
        let bank = world.actor(bare).unwrap().cell;
        world.actors.get_mut(&near).unwrap().cell = bank;
        world.actors.get_mut(&near).unwrap().wetness = 0;
        world.actors.get_mut(&near).unwrap().cold = 0;
        world.actors.get_mut(&bare).unwrap().wetness = 0;
        world.actors.get_mut(&bare).unwrap().cold = 0;
        let waded = world.step(&[
            intent("wade-beaver", near, Attempt::Move(Dir::North)),
            intent("wade-bare", bare, Attempt::Move(Dir::North)),
        ]);
        assert!(
            waded.results.iter().all(|result| result.success),
            "{waded:?}"
        );
        let cloaked = world.actor(near).unwrap().wetness;
        let open = world.actor(bare).unwrap().wetness;
        assert_eq!(open, world.tuning.wade);
        assert_eq!(cloaked, world.tuning.wade / 2);

        world.actors.get_mut(&near).unwrap().cell = cell;
        world.actors.get_mut(&bare).unwrap().cell = cell;
        world.actors.get_mut(&near).unwrap().wetness = 0;
        world.actors.get_mut(&bare).unwrap().wetness = 0;
        world.actors.get_mut(&near).unwrap().cold = 0;
        world.actors.get_mut(&bare).unwrap().cold = 0;
        world.time = world.tuning.rain_starts_at;
        world.step(&[]);
        world.step(&[]);
        assert!(world.actor(near).unwrap().cold < world.actor(bare).unwrap().cold);
        assert_eq!(
            world.actor(near).unwrap().wetness * 2,
            world.actor(bare).unwrap().wetness
        );
    }

    #[test]
    fn a_beaver_is_in_the_brook_again_after_the_same_wait_as_the_otter() {
        let mut world = quiet_valley(1);
        let near = world.spawn_at("Near", "n", 0, 3).unwrap();
        let seen = world.spawn_at("Seen", "s", 2, 3).unwrap();
        let heard = world.spawn_at("Heard", "h", 3, 3).unwrap();
        let quiet = world.spawn_at("Quiet", "q", 6, 4).unwrap();
        let stood = world.actor(near).unwrap().cell;
        flint_spear(&mut world, near, stood);
        let shot = world.step(&[intent("shot", near, Attempt::Hunt)]);
        assert_eq!(
            shot.results[0].message,
            "You bring the beaver down with the flint spear."
        );
        assert!(world.beaver_cell().is_none());
        assert_eq!(world.beaver_returns_at, Some(world.tuning.deer_ticks));
        assert_eq!(world.tuning.deer_ticks, 24);
        let ticks = world.tuning.deer_ticks;
        for _ in 0..ticks - 1 {
            world.step(&[]);
        }
        assert!(world.beaver_cell().is_none());
        let waiting = world.observe(near).unwrap();
        assert!(
            !waiting.text.contains("A beaver is in the brook"),
            "{}",
            waiting.text
        );
        assert!(
            waiting.text.contains("A beaver carcass lies here."),
            "{}",
            waiting.text
        );
        assert!(waiting
            .events
            .iter()
            .all(|event| event.text != "A beaver is in the brook again."));
        world.step(&[]);
        assert_eq!(world.beaver_cell(), Some((0, 3)));
        assert!(world.beaver_returns_at.is_none());
        let arrived = "A beaver is in the brook again.";
        let back = world.observe(near).unwrap();
        assert!(
            back.text.contains("A beaver is in the brook here."),
            "{}",
            back.text
        );
        assert!(
            back.text.contains("A beaver carcass lies here."),
            "{}",
            back.text
        );
        assert_eq!(
            back.events
                .iter()
                .filter(|event| event.text == arrived)
                .count(),
            1
        );
        let seen_back = world.observe(seen).unwrap();
        assert!(
            seen_back
                .text
                .contains("A beaver is in the brook to the west."),
            "{}",
            seen_back.text
        );
        assert_eq!(
            seen_back
                .events
                .iter()
                .filter(|event| event.text == arrived)
                .count(),
            1
        );
        assert!(seen_back
            .events
            .iter()
            .all(|event| !event.text.starts_with("You hear a beaver")));
        let heard_back = world.observe(heard).unwrap();
        assert!(!heard_back.text.contains("A beaver is in the brook"));
        assert_eq!(
            heard_back
                .events
                .iter()
                .filter(|event| event.text == "You hear a beaver to the west.")
                .count(),
            1
        );
        assert!(heard_back.events.iter().all(|event| event.text != arrived));
        assert!(world
            .observe(quiet)
            .unwrap()
            .events
            .iter()
            .all(|event| !event.text.contains("beaver")));
        assert!(!world
            .observe(quiet)
            .unwrap()
            .text
            .contains("A beaver is in the brook"));
        world.step(&[]);
        assert_eq!(world.beaver_cell(), Some((0, 3)));

        let off = world.step(&[intent("off", near, Attempt::Move(Dir::South))]);
        assert!(off.results[0].success, "{off:?}");
        assert!(!off.results[0].message.contains("beaver"));
        let onto = world.step(&[intent("onto", near, Attempt::Move(Dir::North))]);
        assert!(onto.results[0].success, "{onto:?}");
        assert!(!onto.results[0].message.contains("beaver"));
        assert_eq!(world.beaver_cell(), Some((0, 4)));
        assert!(world
            .observe(near)
            .unwrap()
            .events
            .iter()
            .any(|event| { event.text == "A beaver splashes away to the south." }));
        assert!(world
            .observe(quiet)
            .unwrap()
            .events
            .iter()
            .all(|event| { !event.text.contains("beaver") && !event.text.contains("splash") }));

        let mut brought = false;
        for n in 0..12 {
            if world.beaver_cell().is_none() {
                brought = true;
                break;
            }
            let (bx, by) = world.beaver_cell().unwrap();
            let (nx, ny) = super::cell_xy(world.actor(near).unwrap().cell);
            if (bx - nx).abs().max((by - ny).abs()) > 1 {
                world.step(&[]);
                continue;
            }
            let again = world.step(&[intent(&format!("again-{n}"), near, Attempt::Hunt)]);
            if again.results[0]
                .message
                .starts_with("You bring the beaver down")
            {
                brought = true;
                assert!(world.beaver_cell().is_none());
                let second = world.beaver_returns_at.unwrap();
                assert_eq!(second, world.time + world.tuning.deer_ticks - 1);
                break;
            }
        }
        assert!(brought, "the returned beaver can be hunted");
    }
}

#[cfg(test)]
mod fox {
    use super::{Attempt, Dir, World};

    fn intent(id: &str, actor: u32, attempt: Attempt) -> super::Intent {
        super::Intent {
            command_id: id.into(),
            actor_id: actor,
            attempt,
        }
    }

    #[test]
    fn a_fox_on_the_dry_floor_is_named_in_sight_and_slips_away() {
        let mut world = World::valley(crate::tuning::Tuning::evening(), 1);
        assert_eq!(world.fox_cell(), Some((5, 4)));
        let seen = world.spawn_at("Seen", "s", 4, 3).unwrap();
        let heard = world.spawn_at("Heard", "h", 2, 4).unwrap();
        let quiet = world.spawn_at("Quiet", "q", 1, 3).unwrap();
        let from_seen = world.observe(seen).unwrap().text;
        assert!(
            from_seen.contains("A fox is on the valley floor to the south and east."),
            "{from_seen}"
        );
        assert!(!world.observe(heard).unwrap().text.contains("fox"));
        assert!(!world.observe(quiet).unwrap().text.contains("fox"));

        let beside = world.step(&[intent("south", seen, Attempt::Move(Dir::South))]);
        assert_eq!(beside.results[0].message, "You walk south.");
        assert!(!beside.results[0].message.contains("fox"));
        assert_eq!(world.fox_cell(), Some((5, 4)));
        let named = world.observe(seen).unwrap().text;
        assert!(named.contains("East: valley floor, a fox."), "{named}");

        let step = world.step(&[intent("onto", seen, Attempt::Move(Dir::East))]);
        assert_eq!(step.results[0].message, "You walk east.");
        assert!(!step.results[0].message.contains("fox"));
        assert_eq!(world.fox_cell(), Some((6, 4)));
        let yours: Vec<_> = world
            .observe(seen)
            .unwrap()
            .events
            .iter()
            .filter(|event| event.text.contains("fox"))
            .map(|event| event.text.clone())
            .collect();
        assert_eq!(yours, vec!["A fox slips away to the east.".to_string()]);
        let ears: Vec<_> = world
            .observe(heard)
            .unwrap()
            .events
            .iter()
            .filter(|event| event.text.contains("fox"))
            .map(|event| event.text.clone())
            .collect();
        assert_eq!(ears, vec!["You hear a fox to the east.".to_string()]);
        assert!(world
            .observe(quiet)
            .unwrap()
            .events
            .iter()
            .all(|event| !event.text.contains("fox")));
        let after = world.observe(seen).unwrap().text;
        assert!(
            after.contains("A fox is on the valley floor to the east."),
            "{after}"
        );
        assert!(after.contains("East: valley floor, a fox."), "{after}");

        for _ in 0..4 {
            world.step(&[]);
        }
        assert_eq!(world.fox_cell(), Some((6, 4)));

        let mut alone = World::valley(crate::tuning::Tuning::evening(), 1);
        for _ in 0..5 {
            alone.step(&[]);
        }
        assert_eq!(alone.fox_cell(), Some((6, 4)));
        let walker = alone.spawn_at("Seen", "s", 5, 3).unwrap();
        let look = alone.observe(walker).unwrap().text;
        assert!(
            look.contains("A fox is on the valley floor to the south and east."),
            "{look}"
        );
    }
}

#[cfg(test)]
mod hare {
    use super::{Attempt, Dir, Material, Role, World};

    fn intent(id: &str, actor: u32, attempt: Attempt) -> super::Intent {
        super::Intent {
            command_id: id.into(),
            actor_id: actor,
            attempt,
        }
    }

    #[test]
    fn bare_inventory_names_the_arms_and_snare_and_canoe_point_at_the_verbs() {
        use crate::parse::parse_attempt;
        let mut world = World::valley(crate::tuning::Tuning::evening(), 1);
        let near = world.spawn_at("Near", "n", 4, 3).unwrap();
        let seen = world.spawn_at("Seen", "s", 3, 3).unwrap();
        assert_eq!(parse_attempt("inventory"), Ok(Attempt::Arms));
        assert_eq!(parse_attempt("inv"), Ok(Attempt::Arms));
        assert_eq!(parse_attempt("snare"), Ok(Attempt::NameSnare));
        assert_eq!(parse_attempt("trap"), Ok(Attempt::NameSnare));
        assert_eq!(parse_attempt("canoe"), Ok(Attempt::NameCanoe));
        assert_eq!(parse_attempt("dugout"), Ok(Attempt::NameCanoe));
        assert!(matches!(parse_attempt("set snare"), Ok(Attempt::SetSnare)));
        let empty = world.step(&[intent("arms", near, Attempt::Arms)]);
        assert_eq!(empty.results[0].message, "Your arms are empty.");
        assert!(world
            .observe(near)
            .unwrap()
            .events
            .iter()
            .all(|event| event.text != "Your arms are empty."));
        assert!(world
            .observe(seen)
            .unwrap()
            .events
            .iter()
            .all(|event| event.text != "Your arms are empty."));
        let snare = world.step(&[intent("snare", near, Attempt::NameSnare)]);
        assert!(snare.results[0].success);
        assert_eq!(snare.results[0].message, "Set a snare.");
        let canoe = world.step(&[intent("canoe", near, Attempt::NameCanoe)]);
        assert!(canoe.results[0].success);
        assert_eq!(canoe.results[0].message, "Board, carve canoe, or paddle.");
        assert!(world.observe(near).unwrap().events.iter().all(|event| {
            event.text != "Set a snare." && event.text != "Board, carve canoe, or paddle."
        }));
        assert!(world.observe(seen).unwrap().events.iter().all(|event| {
            event.text != "Set a snare." && event.text != "Board, carve canoe, or paddle."
        }));
        assert!(world.snares.is_empty());
        world.mint_item(
            Material::Branch,
            0,
            world.actor(near).unwrap().cell,
            Some(near),
            Role::Loose,
        );
        let held = world.arms_line(near);
        assert!(held.starts_with("You have "), "{held}");
        assert!(world.observe(near).unwrap().text.contains(&held), "{held}");
        let again = world.step(&[intent("again", near, Attempt::Arms)]);
        assert_eq!(again.results[0].message, held);
        assert!(world
            .observe(seen)
            .unwrap()
            .events
            .iter()
            .all(|event| event.text != held));
        assert_eq!(
            world.examine(near, super::ExamineTarget::Hare),
            "A hare is on the valley floor to the south and west."
        );
    }

    #[test]
    fn examine_hare_names_the_animal_underfoot() {
        let mut world = World::valley(crate::tuning::Tuning::evening(), 1);
        let foot = world.spawn_at("Foot", "f", 2, 4).unwrap();
        assert_eq!(world.hare_cell(), Some((2, 4)));
        assert_eq!(
            world.examine(foot, super::ExamineTarget::Hare),
            "A hare is on the valley floor here."
        );
        assert!(world
            .observe(foot)
            .unwrap()
            .events
            .iter()
            .all(|event| !event.text.contains("hare")));
    }

    #[test]
    fn a_hare_on_the_dry_floor_is_named_and_bolts_when_someone_comes_close() {
        let mut world = World::valley(crate::tuning::Tuning::evening(), 1);
        assert_eq!(world.hare_cell(), Some((2, 4)));
        let near = world.spawn_at("Near", "n", 4, 3).unwrap();
        let heard = world.spawn_at("Heard", "h", 5, 3).unwrap();
        let quiet = world.spawn_at("Quiet", "q", 6, 3).unwrap();
        let look = world.observe(near).unwrap();
        assert!(
            look.text
                .contains("A hare is on the valley floor to the south and west."),
            "{}",
            look.text
        );
        assert!(
            look.text
                .contains("Strip willow south of the brook, then set a snare."),
            "{}",
            look.text
        );
        assert!(
            look.text.contains("South and west: valley floor, a hare.") || look.map.contains('r'),
            "text {}\nmap {}",
            look.text,
            look.map
        );
        assert!(look.map.contains('r'), "{}", look.map);
        assert_eq!(
            world.examine(near, super::ExamineTarget::Hare),
            "A hare is on the valley floor to the south and west."
        );
        assert!(world
            .observe(near)
            .unwrap()
            .events
            .iter()
            .all(|event| !event.text.contains("hare")));
        assert_eq!(
            world.examine(heard, super::ExamineTarget::Hare),
            "You cannot see a hare from here."
        );
        assert!(world
            .observe(heard)
            .unwrap()
            .events
            .iter()
            .all(|event| !event.text.contains("hare")));
        assert!(!world.observe(heard).unwrap().text.contains("hare"));
        assert!(!world.observe(heard).unwrap().text.contains("strip willow"));
        assert!(!world.observe(quiet).unwrap().text.contains("hare"));
        assert!(!world.observe(quiet).unwrap().text.contains("strip willow"));

        let beside = world.spawn_at("Beside", "b", 2, 3).unwrap();
        let south = world.observe(beside).unwrap().text;
        assert!(
            south.contains("A hare is on the valley floor to the south.")
                && south.contains("a hare"),
            "{south}"
        );

        let step = world.step(&[intent("west", near, Attempt::Move(Dir::West))]);
        assert_eq!(step.results[0].message, "You walk west.");
        assert!(!step.results[0].message.contains("hare"));
        assert_eq!(world.hare_cell(), Some((1, 4)));
        let yours: Vec<_> = world
            .observe(near)
            .unwrap()
            .events
            .iter()
            .filter(|event| event.text.contains("hare"))
            .map(|event| event.text.clone())
            .collect();
        assert_eq!(
            yours,
            vec!["A hare startles and bolts to the west.".to_string()]
        );
        let ears: Vec<_> = world
            .observe(heard)
            .unwrap()
            .events
            .iter()
            .filter(|event| event.text.contains("hare"))
            .map(|event| event.text.clone())
            .collect();
        assert_eq!(
            ears,
            vec!["You hear a hare to the south and west.".to_string()]
        );
        assert!(world
            .observe(quiet)
            .unwrap()
            .events
            .iter()
            .all(|event| !event.text.contains("hare")));
        let after = world.observe(near).unwrap().text;
        assert!(
            after.contains("A hare is on the valley floor to the south and west."),
            "{after}"
        );
    }

    #[test]
    fn a_snare_on_the_next_cell_catches_the_hare_and_leaves_a_carcass() {
        for willow in [false, true] {
            let mut world = World::valley(crate::tuning::Tuning::evening(), 1);
            assert_eq!(world.hare_cell(), Some((2, 4)));
            let near = world.spawn_at("Near", "n", 2, 2).unwrap();
            let seen = world.spawn_at("Seen", "s", 4, 3).unwrap();
            let quiet = world.spawn_at("Quiet", "q", 6, 3).unwrap();
            let step = world.step(&[intent("south", near, Attempt::Move(Dir::South))]);
            assert_eq!(step.results[0].message, "You walk south.");
            assert_eq!(world.hare_cell(), Some((3, 4)));
            let floor = world.actor(near).unwrap().cell;
            if willow {
                world.mint_item(
                    super::Material::Willow,
                    0,
                    floor,
                    Some(near),
                    super::Role::Loose,
                );
            } else {
                world.mint_item(
                    super::Material::Cord,
                    0,
                    floor,
                    Some(near),
                    super::Role::Loose,
                );
            }
            let set = world.step(&[intent("set", near, Attempt::SetSnare)]);
            let reply = if willow {
                "You set a snare of willow cordage."
            } else {
                "You set a snare of hide strips."
            };
            assert_eq!(set.results[0].message, reply);
            assert!(world
                .observe(near)
                .unwrap()
                .events
                .iter()
                .all(|event| event.text != reply));
            for _ in 0..world.tuning.snare_ticks {
                world.step(&[]);
            }
            let caught = if willow {
                "The willow snare catches a hare."
            } else {
                "The snare catches a hare."
            };
            assert!(world.hare_cell().is_none());
            let here = world.observe(near).unwrap();
            assert!(here.events.iter().any(|event| event.text == caught));
            assert!(
                here.text.contains("A hare carcass lies here."),
                "{}",
                here.text
            );
            assert!(!here.text.contains("raw meat"), "{}", here.text);
            assert!(
                !here.text.contains("A hare is on the valley floor"),
                "{}",
                here.text
            );
            let body = world.examine(near, super::ExamineTarget::Carcass);
            assert!(
                body.contains("A hare carcass lies in contact with the ground"),
                "{body}"
            );
            assert!(world
                .observe(seen)
                .unwrap()
                .events
                .iter()
                .any(|event| event.text == caught));
            assert!(world
                .observe(quiet)
                .unwrap()
                .events
                .iter()
                .all(|event| { !event.text.contains("hare") && !event.text.contains("snare") }));
        }
    }

    #[test]
    fn a_spear_or_a_bow_brings_the_hare_down() {
        use super::contest_hash;
        assert!(matches!(
            crate::parse::parse_attempt("hunt hare").unwrap(),
            Attempt::Hunt
        ));
        let mut world = World::valley(crate::tuning::Tuning::evening(), 1);
        world.deer = None;
        world.fox = None;
        world.otter = None;
        let near = world.spawn_at("Near", "n", 2, 4).unwrap();
        let seen = world.spawn_at("Seen", "s", 4, 3).unwrap();
        let quiet = world.spawn_at("Quiet", "q", 0, 0).unwrap();
        let bare = world.step(&[intent("bare", near, Attempt::Hunt)]);
        assert_eq!(bare.results[0].message, "You are not carrying a spear.");
        assert!(world
            .observe(near)
            .unwrap()
            .events
            .iter()
            .any(|event| { event.text == "You are not carrying a spear." }));
        assert!(world.observe(seen).unwrap().events.iter().all(|event| {
            !event.text.contains("spear") && event.text != bare.results[0].message
        }));
        assert!(world
            .observe(quiet)
            .unwrap()
            .events
            .iter()
            .all(|event| event.text != bare.results[0].message));
        let cell = world.actor(near).unwrap().cell;
        let spear = world.mint_item(Material::Spear, 0, cell, Some(near), Role::Loose);
        world.items.get_mut(&spear).unwrap().flint_tip = true;
        let soft = world.step(&[intent("soft", near, Attempt::Hunt)]);
        assert_eq!(
            soft.results[0].message,
            "The flint tip is not fire-hardened."
        );
        assert!(world
            .observe(seen)
            .unwrap()
            .events
            .iter()
            .all(|event| event.text != soft.results[0].message));
        world.items.get_mut(&spear).unwrap().hardened = true;
        let shot = world.step(&[intent("shot", near, Attempt::Hunt)]);
        let yours = "You bring the hare down with the flint spear.";
        let witnessed = "Near brings the hare down with the flint spear.";
        assert_eq!(shot.results[0].message, yours);
        assert!(world.hare_cell().is_none());
        assert_eq!(
            world.hare_returns_at,
            Some(world.time + world.tuning.deer_ticks - 1)
        );
        let here = world.observe(near).unwrap();
        assert!(
            here.text.contains("A hare carcass lies here."),
            "{}",
            here.text
        );
        assert!(here
            .events
            .iter()
            .all(|event| event.text != yours && event.text != witnessed));
        assert!(world
            .observe(seen)
            .unwrap()
            .events
            .iter()
            .any(|event| event.text == witnessed));
        assert!(world
            .observe(quiet)
            .unwrap()
            .events
            .iter()
            .all(|event| !event.text.contains("hare")));
        let cut = world.step(&[intent("cut", near, Attempt::Butcher)]);
        assert_eq!(
            cut.results[0].message,
            "You butcher the hare carcass into a portion of raw meat, a hare hide, and a bone."
        );
        assert!(world
            .observe(near)
            .unwrap()
            .events
            .iter()
            .all(|event| event.text != cut.results[0].message));
        assert!(world.observe(seen).unwrap().events.iter().any(|event| {
            event.text
                == "Near butchers the hare carcass into a portion of raw meat, a hare hide, and a bone."
        }));

        let mut beside = World::valley(crate::tuning::Tuning::evening(), 1);
        beside.deer = None;
        beside.fox = None;
        beside.otter = None;
        let hunter = beside.spawn_at("Near", "n", 3, 4).unwrap();
        let witness = beside.spawn_at("Seen", "s", 4, 3).unwrap();
        let ears = beside.spawn_at("Heard", "h", 5, 4).unwrap();
        let far = beside.spawn_at("Quiet", "q", 0, 0).unwrap();
        let wood = beside.mint_item(
            Material::Spear,
            0,
            beside.actor(hunter).unwrap().cell,
            Some(hunter),
            Role::Loose,
        );
        {
            let point = beside.items.get_mut(&wood).unwrap();
            point.wooden_tip = true;
            point.hardened = true;
        }
        let short = beside.step(&[intent("short", hunter, Attempt::Hunt)]);
        assert_eq!(
            short.results[0].message,
            "The hare is not close enough to hunt."
        );
        assert!(beside.hare_cell() == Some((2, 4)));
        assert!(beside
            .observe(hunter)
            .unwrap()
            .events
            .iter()
            .any(|event| { event.text == short.results[0].message }));
        assert!(beside
            .observe(witness)
            .unwrap()
            .events
            .iter()
            .all(|event| { event.text != short.results[0].message }));
        beside.items.remove(&wood);
        let stone = beside.mint_item(
            Material::Spear,
            0,
            beside.actor(hunter).unwrap().cell,
            Some(hunter),
            Role::Loose,
        );
        beside.items.get_mut(&stone).unwrap().hardened = true;
        let subject = "2,4:hare";
        let mut miss_at = None;
        for t in 0..40 {
            if contest_hash(1, t, "hunt", subject, hunter) % 3 != 0 {
                miss_at = Some(t);
                break;
            }
        }
        let miss_at = miss_at.unwrap();
        while beside.time < miss_at {
            beside.step(&[]);
        }
        let miss = beside.step(&[intent("miss", hunter, Attempt::Hunt)]);
        assert!(
            miss.results[0].message.contains("bolts"),
            "{}",
            miss.results[0].message
        );
        assert_ne!(beside.hare_cell(), Some((2, 4)));
        assert!(beside.observe(hunter).unwrap().events.iter().all(|event| {
            event.text != miss.results[0].message && !event.text.contains("misses the hare")
        }));
        assert!(beside
            .observe(witness)
            .unwrap()
            .events
            .iter()
            .any(|event| { event.text.contains("hare") && event.text.contains("bolts") }));
        assert!(beside
            .observe(ears)
            .unwrap()
            .events
            .iter()
            .any(|event| { event.text == "You hear a hare to the west." }));
        assert!(beside
            .observe(far)
            .unwrap()
            .events
            .iter()
            .all(|event| !event.text.contains("hare")));

        let mut aimed = World::valley(crate::tuning::Tuning::evening(), 1);
        aimed.deer = None;
        aimed.fox = None;
        aimed.otter = None;
        let bow = aimed.spawn_at("Near", "n", 4, 4).unwrap();
        let saw = aimed.spawn_at("Seen", "s", 4, 3).unwrap();
        let quiet = aimed.spawn_at("Quiet", "q", 0, 0).unwrap();
        let stand = aimed.actor(bow).unwrap().cell;
        aimed.mint_item(Material::Bow, 0, stand, Some(bow), Role::Loose);
        aimed.mint_item(Material::Flake, 0, stand, Some(bow), Role::Loose);
        let loosed = aimed.step(&[intent("loosed", bow, Attempt::Hunt)]);
        assert_eq!(
            loosed.results[0].message,
            "You bring the hare down with the bow. The flint flake is spent."
        );
        assert!(aimed.loose_ids(bow, Material::Flake).is_empty());
        assert_eq!(aimed.loose_ids(bow, Material::Bow).len(), 1);
        assert!(aimed.hare_cell().is_none());
        assert!(aimed
            .observe(bow)
            .unwrap()
            .events
            .iter()
            .all(|event| { event.text != loosed.results[0].message }));
        assert!(aimed.observe(saw).unwrap().events.iter().any(|event| {
            event.text == "Near brings the hare down with the bow. The flint flake is spent."
        }));
        assert!(aimed
            .observe(quiet)
            .unwrap()
            .events
            .iter()
            .all(|event| !event.text.contains("hare")));
        assert!(aimed
            .items
            .values()
            .any(|item| item.material == Material::Carcass && item.hare));
    }

    #[test]
    fn a_hare_is_on_the_dry_floor_again_after_the_same_wait_as_the_fox() {
        let mut world = World::valley(crate::tuning::Tuning::evening(), 1);
        let near = world.spawn_at("Near", "n", 3, 4).unwrap();
        let seen = world.spawn_at("Seen", "s", 4, 3).unwrap();
        let heard = world.spawn_at("Heard", "h", 5, 4).unwrap();
        let quiet = world.spawn_at("Quiet", "q", 0, 0).unwrap();
        assert_eq!(world.hare_cell(), Some((2, 4)));
        world.mint_item(
            Material::Cord,
            0,
            world.actor(near).unwrap().cell,
            Some(near),
            Role::Loose,
        );
        let set = world.step(&[intent("set", near, Attempt::SetSnare)]);
        assert_eq!(set.results[0].message, "You set a snare of hide strips.");
        assert!(world.hare_returns_at.is_none());
        for _ in 0..world.tuning.snare_ticks {
            world.step(&[]);
        }
        assert!(world.hare_cell().is_none());
        let back_at = world.hare_returns_at.unwrap();
        assert_eq!(back_at, world.time + world.tuning.deer_ticks - 1);
        assert_eq!(world.tuning.deer_ticks, 24);
        let gone = world.examine(quiet, super::ExamineTarget::Hare);
        assert_eq!(gone, "You cannot see a hare from here.");
        assert!(world
            .observe(near)
            .unwrap()
            .events
            .iter()
            .all(|event| event.text != gone));
        while world.time < back_at {
            world.step(&[]);
        }
        assert!(world.hare_cell().is_none());
        let waiting = world.observe(near).unwrap();
        assert!(
            !waiting.text.contains("A hare is on the valley floor"),
            "{}",
            waiting.text
        );
        assert!(waiting
            .events
            .iter()
            .all(|event| event.text != "A hare is on the valley floor again."));
        world.step(&[]);
        assert_eq!(world.hare_cell(), Some((2, 4)));
        assert!(world.hare_returns_at.is_none());
        let arrived = "A hare is on the valley floor again.";
        let back = world.observe(near).unwrap();
        assert!(
            back.text
                .contains("A hare is on the valley floor to the west."),
            "{}",
            back.text
        );
        assert!(
            back.text
                .contains("Strip willow south of the brook, then set a snare."),
            "{}",
            back.text
        );
        assert_eq!(
            back.events
                .iter()
                .filter(|event| event.text == arrived)
                .count(),
            1
        );
        assert_eq!(
            world.examine(near, super::ExamineTarget::Hare),
            "A hare is on the valley floor to the west."
        );
        let seen_back = world.observe(seen).unwrap();
        assert!(
            seen_back
                .text
                .contains("A hare is on the valley floor to the south and west."),
            "{}",
            seen_back.text
        );
        assert_eq!(
            seen_back
                .events
                .iter()
                .filter(|event| event.text == arrived)
                .count(),
            1
        );
        assert!(seen_back
            .events
            .iter()
            .all(|event| !event.text.starts_with("You hear a hare")));
        let heard_back = world.observe(heard).unwrap();
        assert!(!heard_back.text.contains("A hare is on the valley floor"));
        assert_eq!(
            heard_back
                .events
                .iter()
                .filter(|event| event.text == "You hear a hare to the west.")
                .count(),
            1
        );
        assert!(heard_back.events.iter().all(|event| event.text != arrived));
        assert!(world
            .observe(quiet)
            .unwrap()
            .events
            .iter()
            .all(|event| !event.text.contains("hare")));
        assert!(!world
            .observe(quiet)
            .unwrap()
            .text
            .contains("A hare is on the valley floor"));
        world.step(&[]);
        assert_eq!(
            world
                .observe(near)
                .unwrap()
                .events
                .iter()
                .filter(|event| event.text == arrived)
                .count(),
            1
        );

        let onto = world.step(&[intent("onto", near, Attempt::Move(Dir::West))]);
        assert_eq!(onto.results[0].message, "You walk west.");
        assert!(!onto.results[0].message.contains("hare"));
        assert_eq!(world.hare_cell(), Some((1, 4)));
        let bolted = world.observe(near).unwrap();
        assert!(bolted
            .events
            .iter()
            .any(|event| event.text == "A hare startles and bolts to the west."));
        assert!(world
            .observe(heard)
            .unwrap()
            .events
            .iter()
            .any(|event| event.text == "You hear a hare to the west."));
        world.mint_item(
            Material::Cord,
            0,
            world.actor(near).unwrap().cell,
            Some(near),
            Role::Loose,
        );
        let again = world.step(&[intent("again", near, Attempt::SetSnare)]);
        assert_eq!(again.results[0].message, "You set a snare of hide strips.");
        for _ in 0..world.tuning.snare_ticks {
            world.step(&[]);
        }
        assert!(world.hare_cell().is_none());
        assert_eq!(
            world.hare_returns_at,
            Some(world.time + world.tuning.deer_ticks - 1)
        );
        assert!(world
            .observe(near)
            .unwrap()
            .events
            .iter()
            .any(|event| event.text == "The snare catches a hare."));
        assert!(world
            .observe(quiet)
            .unwrap()
            .events
            .iter()
            .all(|event| !event.text.contains("snare") && !event.text.contains("hare")));
    }

    #[test]
    fn a_flint_spear_or_an_iron_knife_butchers_the_hare_carcass() {
        let mut world = World::valley(crate::tuning::Tuning::evening(), 1);
        let near = world.spawn_at("Near", "n", 2, 3).unwrap();
        let seen = world.spawn_at("Seen", "s", 3, 3).unwrap();
        let quiet = world.spawn_at("Quiet", "q", 6, 3).unwrap();
        let cell = world.actor(near).unwrap().cell;
        let bare = world.step(&[intent("none", near, Attempt::Butcher)]);
        assert_eq!(
            bare.results[0].message,
            "You need a flint spear or an iron knife."
        );
        assert!(world
            .observe(seen)
            .unwrap()
            .events
            .iter()
            .all(|event| { event.text != "You need a flint spear or an iron knife." }));
        let spear = world.mint_item(
            super::Material::Spear,
            0,
            cell,
            Some(near),
            super::Role::Loose,
        );
        let point = world.items.get_mut(&spear).unwrap();
        point.hardened = true;
        point.flint_tip = true;
        let empty = world.step(&[intent("empty", near, Attempt::Butcher)]);
        assert_eq!(
            empty.results[0].message,
            "There is no carcass here to butcher."
        );
        assert!(world
            .observe(seen)
            .unwrap()
            .events
            .iter()
            .all(|event| { event.text != "There is no carcass here to butcher." }));
        let carcass = world.mint_item(super::Material::Carcass, 0, cell, None, super::Role::Loose);
        world.items.get_mut(&carcass).unwrap().hare = true;
        let cut = world.step(&[intent("cut", near, Attempt::Butcher)]);
        let yours =
            "You butcher the hare carcass into a portion of raw meat, a hare hide, and a bone.";
        let heard =
            "Near butchers the hare carcass into a portion of raw meat, a hare hide, and a bone.";
        assert_eq!(cut.results[0].message, yours);
        let here = world.observe(near).unwrap();
        assert!(here
            .events
            .iter()
            .all(|event| event.text != yours && event.text != heard));
        assert!(
            here.text.contains("A portion of raw meat lies here."),
            "{}",
            here.text
        );
        assert!(
            here.text.contains("A hare hide lies here."),
            "{}",
            here.text
        );
        assert!(here.text.contains("A bone lies here."), "{}", here.text);
        assert!(!here.text.contains("A hare carcass"), "{}", here.text);
        assert!(!here.text.contains("deer hide"), "{}", here.text);
        assert_eq!(world.ground_ids(cell, super::Material::Meat).len(), 1);
        assert!(world
            .ground_ids(cell, super::Material::Hide)
            .iter()
            .any(|id| {
                world
                    .item(*id)
                    .is_some_and(|hide| hide.hare && !hide.fox && !hide.otter && !hide.wolf)
            }));
        assert!(world
            .observe(seen)
            .unwrap()
            .events
            .iter()
            .any(|event| event.text == heard));
        assert!(world
            .observe(quiet)
            .unwrap()
            .events
            .iter()
            .all(|event| { !event.text.contains("butcher") && !event.text.contains("hare") }));
        let taken = world.step(&[intent("hide", near, Attempt::Gather(super::Material::Hide))]);
        assert_eq!(taken.results[0].message, "You gather a hare hide.");
        let arms = world.observe(near).unwrap();
        assert!(arms.text.contains("a hare hide"), "{}", arms.text);
        assert!(arms.events.iter().all(|event| {
            event.text != "You gather a hare hide." && event.text != "Near gathers a hare hide."
        }));
        assert!(world
            .observe(seen)
            .unwrap()
            .events
            .iter()
            .any(|event| { event.text == "Near gathers a hare hide." }));

        let mut world = World::valley(crate::tuning::Tuning::evening(), 1);
        let near = world.spawn_at("Near", "n", 2, 3).unwrap();
        let seen = world.spawn_at("Seen", "s", 3, 3).unwrap();
        let cell = world.actor(near).unwrap().cell;
        let spear = world.mint_item(
            super::Material::Spear,
            0,
            cell,
            Some(near),
            super::Role::Loose,
        );
        world.items.get_mut(&spear).unwrap().flint_tip = true;
        world.mint_item(
            super::Material::Knife,
            0,
            cell,
            Some(near),
            super::Role::Loose,
        );
        let carcass = world.mint_item(super::Material::Carcass, 0, cell, None, super::Role::Loose);
        world.items.get_mut(&carcass).unwrap().hare = true;
        let both = world.step(&[intent("both", near, Attempt::Butcher)]);
        assert_eq!(
            both.results[0].message,
            "You butcher the hare carcass with the iron knife into a portion of raw meat, a hare hide, and a bone."
        );
        assert_eq!(world.loose_ids(near, super::Material::Knife).len(), 1);
        assert_eq!(world.loose_ids(near, super::Material::Spear).len(), 1);
        assert!(world
            .observe(near)
            .unwrap()
            .events
            .iter()
            .all(|event| { !event.text.contains("butcher") }));
        assert!(world.observe(seen).unwrap().events.iter().any(|event| {
            event.text == "Near butchers the hare carcass with the iron knife into a portion of raw meat, a hare hide, and a bone."
        }));
    }

    #[test]
    fn a_hare_hide_sews_into_a_light_cloak() {
        let mut world = World::valley(crate::tuning::Tuning::evening(), 1);
        let near = world.spawn_at("Near", "n", 4, 3).unwrap();
        let seen = world.spawn_at("Seen", "s", 5, 3).unwrap();
        let quiet = world.spawn_at("Quiet", "q", 0, 4).unwrap();
        let bare = world.spawn_at("Bare", "b", 6, 3).unwrap();
        let deer = world.spawn_at("Deer", "d", 3, 3).unwrap();
        let cell = world.actor(near).unwrap().cell;
        let hide = world.mint_item(Material::Hide, 0, cell, Some(near), Role::Loose);
        world.items.get_mut(&hide).unwrap().hare = true;
        let awl = world.mint_item(Material::Awl, 0, cell, Some(near), Role::Loose);
        world.items.get_mut(&awl).unwrap().bone = true;
        let deer_cell = world.actor(deer).unwrap().cell;
        let deer_hide = world.mint_item(Material::Hide, 0, deer_cell, Some(deer), Role::Loose);
        world.items.get_mut(&deer_hide).unwrap().sewn = true;
        world.items.get_mut(&deer_hide).unwrap().worn = true;

        let early = world.step(&[intent("sew", near, Attempt::Sew)]);
        assert_eq!(early.results[0].message, "The hare hide is not pierced.");
        assert!(!early.results[0].success);
        assert!(world
            .observe(near)
            .unwrap()
            .events
            .iter()
            .all(|event| event.text != "The hare hide is not pierced."));
        assert!(world.observe(seen).unwrap().events.iter().all(|event| {
            !event.text.contains("hare") && event.text != "The hare hide is not pierced."
        }));

        let pierced = world.step(&[intent("pierce", near, Attempt::Pierce)]);
        assert_eq!(
            pierced.results[0].message,
            "You pierce the hare hide with the bone awl. It is ready to sew."
        );
        assert!(world
            .observe(near)
            .unwrap()
            .events
            .iter()
            .all(|event| { !event.text.contains("pierce") && !event.text.contains("hare hide") }));
        assert!(world.observe(seen).unwrap().events.iter().any(|event| {
            event.text == "Near pierces the hare hide with the bone awl. It is ready to sew."
        }));
        assert!(world
            .observe(quiet)
            .unwrap()
            .events
            .iter()
            .all(|event| !event.text.contains("hare") && !event.text.contains("pierce")));

        let sewn = world.step(&[intent("sew-cloak", near, Attempt::Sew)]);
        assert_eq!(
            sewn.results[0].message,
            "You sew the pierced hare hide into a cloak."
        );
        let again = world.step(&[intent("again", near, Attempt::Sew)]);
        assert_eq!(
            again.results[0].message,
            "The hare hide is already a cloak."
        );
        assert!(world
            .observe(seen)
            .unwrap()
            .events
            .iter()
            .all(|event| event.text != "The hare hide is already a cloak."));
        let carried = world.examine(near, super::ExamineTarget::Hide);
        assert!(
            carried.contains(
                "A sewn hare-hide cloak is carried by you, in the open. It is a light cover."
            ),
            "{carried}"
        );
        let wore = world.step(&[intent("wear", near, Attempt::Wear)]);
        assert_eq!(
            wore.results[0].message,
            "You wear the sewn hare-hide cloak."
        );
        let look = world.observe(near).unwrap().text;
        assert!(
            look.contains("The sewn hare-hide cloak is on you. It is a light cover."),
            "{look}"
        );
        assert!(look.contains("a sewn hare-hide cloak"), "{look}");
        assert!(look.contains("You are in the wind."), "{look}");
        assert!(!look.contains("little wind"), "{look}");
        let held = world.examine(near, super::ExamineTarget::Hide);
        assert!(
            held.contains("A sewn hare-hide cloak is on you, in the open. It is a light cover."),
            "{held}"
        );
        assert!(world
            .observe(seen)
            .unwrap()
            .events
            .iter()
            .any(|event| { event.text == "Near sews the pierced hare hide into a cloak." }));
        assert!(world
            .observe(seen)
            .unwrap()
            .events
            .iter()
            .any(|event| { event.text == "Near wears the sewn hare-hide cloak." }));
        assert!(world.observe(near).unwrap().events.iter().all(|event| {
            !event.text.contains("sewn hare-hide") && !event.text.contains("pierced hare")
        }));
        assert!(world
            .observe(quiet)
            .unwrap()
            .events
            .iter()
            .all(|event| !event.text.contains("hare") && !event.text.contains("cloak")));

        world.step(&[]);
        let hare_cold = world.actor(near).unwrap().cold;
        let deer_cold = world.actor(deer).unwrap().cold;
        let bare_cold = world.actor(bare).unwrap().cold;
        assert!(
            hare_cold < bare_cold && hare_cold > deer_cold,
            "hare {hare_cold} deer {deer_cold} bare {bare_cold}"
        );
        assert!(!world.wearing_otter_cloak(near));
        assert!(world.wearing_only_hare_cloak(near));
    }
}

#[cfg(test)]
mod fox_hunt {
    use super::{Attempt, Material, Role, World};

    fn intent(id: &str, actor: u32, attempt: Attempt) -> super::Intent {
        super::Intent {
            command_id: id.into(),
            actor_id: actor,
            attempt,
        }
    }

    fn flint_spear(world: &mut World, actor: u32) {
        let cell = world.actor(actor).unwrap().cell;
        let spear = world.mint_item(Material::Spear, 0, cell, Some(actor), Role::Loose);
        let point = world.items.get_mut(&spear).unwrap();
        point.hardened = true;
        point.flint_tip = true;
    }

    #[test]
    fn a_flint_spear_or_a_bow_brings_the_fox_down() {
        assert!(matches!(
            crate::parse::parse_attempt("hunt fox").unwrap(),
            Attempt::Hunt
        ));
        let mut world = World::valley(crate::tuning::Tuning::evening(), 1);
        let near = world.spawn_at("Near", "n", 5, 4).unwrap();
        let seen = world.spawn_at("Seen", "s", 4, 4).unwrap();
        let quiet = world.spawn_at("Quiet", "q", 1, 3).unwrap();
        assert!(world.observe(near).unwrap().legend.contains("x fox"));
        assert!(
            world.observe(seen).unwrap().map.contains('x'),
            "{}",
            world.observe(seen).unwrap().map
        );
        let bare = world.step(&[intent("bare", near, Attempt::Hunt)]);
        assert_eq!(bare.results[0].message, "You are not carrying a spear.");
        assert!(world
            .observe(seen)
            .unwrap()
            .events
            .iter()
            .all(|event| event.text != "You are not carrying a spear."));
        flint_spear(&mut world, near);
        let shot = world.step(&[intent("shot", near, Attempt::Hunt)]);
        assert_eq!(
            shot.results[0].message,
            "You bring the fox down with the flint spear."
        );
        assert!(shot.results[0].success);
        assert!(world.fox_cell().is_none());
        let here = world.observe(near).unwrap().text;
        assert!(here.contains("A fox carcass lies here."), "{here}");
        assert!(!here.contains("A deer carcass"), "{here}");
        assert!(!here.contains("A fox is on the valley floor"), "{here}");
        let east = world.observe(seen).unwrap().text;
        assert!(east.contains("East: valley floor, a fox carcass"), "{east}");
        let body = world.examine(near, super::ExamineTarget::Carcass);
        assert!(
            body.contains("A fox carcass lies in contact with the ground"),
            "{body}"
        );
        assert!(world
            .observe(near)
            .unwrap()
            .events
            .iter()
            .all(|event| !event.text.contains("fox")));
        let heard = "Near brings the fox down with the flint spear.";
        assert!(world
            .observe(seen)
            .unwrap()
            .events
            .iter()
            .any(|event| event.text == heard));
        assert!(world
            .observe(quiet)
            .unwrap()
            .events
            .iter()
            .all(|event| !event.text.contains("fox")));
        let left = world.step(&[intent("butcher", near, Attempt::Butcher)]);
        assert_eq!(
            left.results[0].message,
            "You butcher the fox carcass into a portion of raw meat, a fox hide, and a bone."
        );
        let cut = world.observe(near).unwrap();
        assert!(
            cut.text.contains("A portion of raw meat lies here."),
            "{}",
            cut.text
        );
        assert!(cut.text.contains("A fox hide lies here."), "{}", cut.text);
        assert!(cut.text.contains("A bone lies here."), "{}", cut.text);
        assert!(!cut.text.contains("A fox carcass"), "{}", cut.text);
        assert!(!cut.text.contains("deer hide"), "{}", cut.text);
        assert!(cut
            .events
            .iter()
            .all(|event| !event.text.contains("butcher")));
        let butchered =
            "Near butchers the fox carcass into a portion of raw meat, a fox hide, and a bone.";
        assert!(world
            .observe(seen)
            .unwrap()
            .events
            .iter()
            .any(|event| event.text == butchered));
        assert!(world
            .observe(quiet)
            .unwrap()
            .events
            .iter()
            .all(|event| !event.text.contains("butcher") && !event.text.contains("fox hide")));
        assert_eq!(
            world
                .ground_ids(world.actor(near).unwrap().cell, Material::Meat)
                .len(),
            1
        );
        assert!(world
            .ground_ids(world.actor(near).unwrap().cell, Material::Hide)
            .iter()
            .any(|id| world
                .item(*id)
                .is_some_and(|hide| hide.fox && !hide.otter && !hide.wolf)));

        let mut world = World::valley(crate::tuning::Tuning::evening(), 1);
        let near = world.spawn_at("Near", "n", 4, 4).unwrap();
        let seen = world.spawn_at("Seen", "s", 6, 3).unwrap();
        let quiet = world.spawn_at("Quiet", "q", 0, 4).unwrap();
        let cell = world.actor(near).unwrap().cell;
        world.mint_item(Material::Bow, 0, cell, Some(near), Role::Loose);
        world.mint_item(Material::Flake, 0, cell, Some(near), Role::Loose);
        let shot = world.step(&[intent("bow", near, Attempt::Hunt)]);
        assert_eq!(
            shot.results[0].message,
            "You bring the fox down with the bow. The flint flake is spent."
        );
        assert!(world.fox_cell().is_none());
        assert!(world.loose_ids(near, Material::Flake).is_empty());
        assert_eq!(world.loose_ids(near, Material::Bow).len(), 1);
        assert!(world
            .observe(near)
            .unwrap()
            .events
            .iter()
            .all(|event| !event.text.contains("fox")));
        assert!(world.observe(seen).unwrap().events.iter().any(|event| {
            event.text == "Near brings the fox down with the bow. The flint flake is spent."
        }));
        assert!(world
            .observe(quiet)
            .unwrap()
            .events
            .iter()
            .all(|event| !event.text.contains("fox")));
        let floor = world.observe(near).unwrap().text;
        assert!(
            floor.contains("East: valley floor, a fox carcass"),
            "{floor}"
        );
    }

    #[test]
    fn a_knife_is_named_when_both_tools_butcher_the_fox() {
        let mut world = World::valley(crate::tuning::Tuning::evening(), 1);
        let near = world.spawn_at("Near", "n", 4, 4).unwrap();
        let seen = world.spawn_at("Seen", "s", 5, 4).unwrap();
        let quiet = world.spawn_at("Quiet", "q", 0, 4).unwrap();
        let cell = world.actor(near).unwrap().cell;
        flint_spear(&mut world, near);
        let bare = world.step(&[intent("none", near, Attempt::Butcher)]);
        assert_eq!(
            bare.results[0].message,
            "There is no carcass here to butcher."
        );
        assert!(world
            .observe(seen)
            .unwrap()
            .events
            .iter()
            .all(|event| event.text != "There is no carcass here to butcher."));
        assert!(world
            .observe(quiet)
            .unwrap()
            .events
            .iter()
            .all(|event| event.text != "There is no carcass here to butcher."));
        let carcass = world.mint_item(Material::Carcass, 0, cell, None, Role::Loose);
        world.items.get_mut(&carcass).unwrap().fox = true;
        world.mint_item(Material::Knife, 0, cell, Some(near), Role::Loose);
        let both = world.step(&[intent("both", near, Attempt::Butcher)]);
        assert_eq!(
            both.results[0].message,
            "You butcher the fox carcass with the iron knife into a portion of raw meat, a fox hide, and a bone."
        );
        assert_eq!(world.loose_ids(near, Material::Knife).len(), 1);
        assert_eq!(world.loose_ids(near, Material::Spear).len(), 1);
        assert!(world
            .observe(near)
            .unwrap()
            .events
            .iter()
            .all(|event| {
                event.text
                    != "You butcher the fox carcass with the iron knife into a portion of raw meat, a fox hide, and a bone."
                    && event.text
                        != "Near butchers the fox carcass with the iron knife into a portion of raw meat, a fox hide, and a bone."
            }));
        assert!(world.observe(seen).unwrap().events.iter().any(|event| {
            event.text == "Near butchers the fox carcass with the iron knife into a portion of raw meat, a fox hide, and a bone."
        }));
        let here = world.observe(near).unwrap().text;
        assert!(here.contains("A fox hide lies here."), "{here}");
        assert!(
            world.observe(seen).unwrap().text.contains("a fox hide"),
            "{}",
            world.observe(seen).unwrap().text
        );
        let taken = world.step(&[intent("hide", near, Attempt::Gather(Material::Hide))]);
        assert_eq!(taken.results[0].message, "You gather a fox hide.");
        let arms = world.observe(near).unwrap().text;
        assert!(arms.contains("a fox hide"), "{arms}");
        assert!(!arms.contains("deer hide"), "{arms}");
        assert!(world
            .observe(near)
            .unwrap()
            .events
            .iter()
            .all(|event| !event.text.contains("You gather a fox hide.")
                && !event.text.contains("gathers a fox hide")));
        assert!(world
            .observe(seen)
            .unwrap()
            .events
            .iter()
            .any(|event| { event.text == "Near gathers a fox hide." }));
        let body = world.examine(near, super::ExamineTarget::Hide);
        assert!(body.contains("A fox hide is carried by you"), "{body}");
    }

    #[test]
    fn a_fox_hide_sews_into_a_cloak_that_breaks_the_wind() {
        let mut world = World::valley(crate::tuning::Tuning::evening(), 1);
        let near = world.spawn_at("Near", "n", 4, 3).unwrap();
        let seen = world.spawn_at("Seen", "s", 4, 3).unwrap();
        let quiet = world.spawn_at("Quiet", "q", 0, 4).unwrap();
        let wolf = world.spawn_at("Wolf", "w", 4, 3).unwrap();
        world.fox = None;
        let cell = world.actor(near).unwrap().cell;
        let hide = world.mint_item(Material::Hide, 0, cell, Some(near), Role::Loose);
        world.items.get_mut(&hide).unwrap().fox = true;
        let awl = world.mint_item(Material::Awl, 0, cell, Some(near), Role::Loose);
        world.items.get_mut(&awl).unwrap().bone = true;
        let deer_hide = world.mint_item(Material::Hide, 0, cell, Some(seen), Role::Loose);
        world.items.get_mut(&deer_hide).unwrap().sewn = true;
        world.items.get_mut(&deer_hide).unwrap().worn = true;
        let wolf_hide = world.mint_item(Material::Hide, 0, cell, Some(wolf), Role::Loose);
        world.items.get_mut(&wolf_hide).unwrap().wolf = true;
        world.items.get_mut(&wolf_hide).unwrap().sewn = true;
        world.items.get_mut(&wolf_hide).unwrap().worn = true;

        let early = world.step(&[intent("sew", near, Attempt::Sew)]);
        assert_eq!(early.results[0].message, "The fox hide is not pierced.");
        assert!(!early.results[0].success);
        assert!(world
            .observe(near)
            .unwrap()
            .events
            .iter()
            .all(|event| event.text != "The fox hide is not pierced."));
        assert!(world
            .observe(seen)
            .unwrap()
            .events
            .iter()
            .all(|event| !event.text.contains("fox")));
        assert!(world
            .observe(quiet)
            .unwrap()
            .events
            .iter()
            .all(|event| !event.text.contains("fox")));

        let pierced = world.step(&[intent("pierce", near, Attempt::Pierce)]);
        assert_eq!(
            pierced.results[0].message,
            "You pierce the fox hide with the bone awl. It is ready to sew."
        );
        assert!(world
            .observe(near)
            .unwrap()
            .events
            .iter()
            .all(|event| { !event.text.contains("pierce") && !event.text.contains("fox hide") }));
        assert!(world.observe(seen).unwrap().events.iter().any(|event| {
            event.text == "Near pierces the fox hide with the bone awl. It is ready to sew."
        }));
        assert!(world
            .observe(quiet)
            .unwrap()
            .events
            .iter()
            .all(|event| !event.text.contains("fox") && !event.text.contains("pierce")));

        let sewn = world.step(&[intent("sew-cloak", near, Attempt::Sew)]);
        assert_eq!(
            sewn.results[0].message,
            "You sew the pierced fox hide into a cloak."
        );
        let again = world.step(&[intent("again", near, Attempt::Sew)]);
        assert_eq!(again.results[0].message, "The fox hide is already a cloak.");
        assert!(world
            .observe(seen)
            .unwrap()
            .events
            .iter()
            .all(|event| event.text != "The fox hide is already a cloak."));
        let carried = world.examine(near, super::ExamineTarget::Hide);
        assert!(
            carried.contains(
                "A sewn fox-hide cloak is carried by you, in the open. It breaks the wind."
            ),
            "{carried}"
        );
        let wore = world.step(&[intent("wear", near, Attempt::Wear)]);
        assert_eq!(wore.results[0].message, "You wear the sewn fox-hide cloak.");
        let look = world.observe(near).unwrap().text;
        assert!(
            look.contains("The sewn fox-hide cloak is on you. It breaks the wind."),
            "{look}"
        );
        assert!(look.contains("a sewn fox-hide cloak"), "{look}");
        assert!(look.contains("You are in a little wind."), "{look}");
        let held = world.examine(near, super::ExamineTarget::Hide);
        assert!(
            held.contains("A sewn fox-hide cloak is on you, in the open. It breaks the wind."),
            "{held}"
        );
        let seen_hide = world.examine(seen, super::ExamineTarget::Hide);
        assert!(
            seen_hide
                .contains("A sewn fox-hide cloak is on Near, in the open. It breaks the wind."),
            "{seen_hide}"
        );
        assert!(world
            .observe(seen)
            .unwrap()
            .events
            .iter()
            .any(|event| { event.text == "Near sews the pierced fox hide into a cloak." }));
        assert!(world
            .observe(seen)
            .unwrap()
            .events
            .iter()
            .any(|event| { event.text == "Near wears the sewn fox-hide cloak." }));
        assert!(world.observe(near).unwrap().events.iter().all(|event| {
            !event.text.contains("sewn fox-hide") && !event.text.contains("pierced fox")
        }));
        let quiet_events: Vec<_> = world
            .observe(quiet)
            .unwrap()
            .events
            .iter()
            .map(|event| event.text.clone())
            .collect();
        assert!(
            quiet_events.iter().all(|text| !text.contains("fox")),
            "{quiet_events:?}"
        );

        for id in [near, seen, quiet, wolf] {
            let person = world.actors.get_mut(&id).unwrap();
            person.cold = 0;
            person.warmth = 0;
            person.warm_from_fire = false;
            person.wetness = 0;
            person.cell = cell;
        }
        for _ in 0..8 {
            world.step(&[]);
        }
        let fox_body = world.observe(near).unwrap();
        let deer_body = world.observe(seen).unwrap();
        let bare_body = world.observe(quiet).unwrap();
        let wolf_body = world.observe(wolf).unwrap();
        assert!(
            fox_body.text.contains("You are in a little wind."),
            "{}",
            fox_body.text
        );
        assert!(
            deer_body.text.contains("You are in the wind."),
            "{}",
            deer_body.text
        );
        assert!(
            wolf_body.text.contains("You are in a little wind."),
            "{}",
            wolf_body.text
        );
        assert!(
            bare_body.text.contains("You are in the wind."),
            "{}",
            bare_body.text
        );
        let fox_cold = world.actor(near).unwrap().cold;
        let deer_cold = world.actor(seen).unwrap().cold;
        let wolf_cold = world.actor(wolf).unwrap().cold;
        let bare_cold = world.actor(quiet).unwrap().cold;
        assert_eq!(fox_cold, deer_cold, "fox {fox_cold} deer {deer_cold}");
        assert!(wolf_cold < fox_cold && fox_cold < bare_cold);

        world.actors.get_mut(&near).unwrap().wetness = 0;
        world.actors.get_mut(&seen).unwrap().wetness = 0;
        world.time = world.tuning.rain_starts_at;
        world.step(&[]);
        world.step(&[]);
        let fox_rain = world.actor(near).unwrap().wetness;
        let deer_rain = world.actor(seen).unwrap().wetness;
        assert!(fox_rain > 0, "{fox_rain}");
        assert_eq!(fox_rain, deer_rain, "fox {fox_rain} deer {deer_rain}");

        world.actors.get_mut(&quiet).unwrap().cell = super::cell_id(0, 4).unwrap();
        let off = world.step(&[intent("remove", near, Attempt::Remove)]);
        assert_eq!(
            off.results[0].message,
            "You remove the sewn fox-hide cloak. The wind reaches you again."
        );
        let bare_wind = world.observe(near).unwrap();
        assert!(
            bare_wind.text.contains("You are in the wind."),
            "{}",
            bare_wind.text
        );
        assert!(bare_wind.events.iter().all(|event| {
            !event.text.contains("removes the sewn fox-hide")
                && !event.text.contains("The wind reaches you again.")
        }));
        assert!(world
            .observe(seen)
            .unwrap()
            .events
            .iter()
            .any(|event| { event.text == "Near removes the sewn fox-hide cloak." }));
        assert!(world
            .observe(quiet)
            .unwrap()
            .events
            .iter()
            .all(|event| !event.text.contains("fox")));
    }

    #[test]
    fn a_fox_is_on_the_dry_floor_again_after_the_same_wait_as_the_otter() {
        let mut world = World::valley(crate::tuning::Tuning::evening(), 1);
        let deer = world.deer.as_mut().unwrap();
        deer.cell = super::cell_id(2, 0).unwrap();
        let near = world.spawn_at("Near", "n", 5, 4).unwrap();
        let seen = world.spawn_at("Seen", "s", 4, 3).unwrap();
        let heard = world.spawn_at("Heard", "h", 5, 1).unwrap();
        let quiet = world.spawn_at("Quiet", "q", 0, 0).unwrap();
        flint_spear(&mut world, near);
        let shot = world.step(&[intent("shot", near, Attempt::Hunt)]);
        assert_eq!(
            shot.results[0].message,
            "You bring the fox down with the flint spear."
        );
        assert!(world.fox_cell().is_none());
        assert_eq!(world.fox_returns_at, Some(world.tuning.deer_ticks));
        assert_eq!(world.tuning.deer_ticks, 24);
        let ticks = world.tuning.deer_ticks;
        for _ in 0..ticks - 1 {
            world.step(&[]);
        }
        assert!(world.fox_cell().is_none());
        let waiting = world.observe(near).unwrap().text;
        assert!(
            !waiting.contains("A fox is on the valley floor"),
            "{waiting}"
        );
        assert!(waiting.contains("A fox carcass lies here."), "{waiting}");
        assert!(world
            .observe(near)
            .unwrap()
            .events
            .iter()
            .all(|event| { event.text != "A fox is on the valley floor again." }));
        world.step(&[]);
        assert_eq!(world.fox_cell(), Some((5, 4)));
        assert!(world.fox_returns_at.is_none());
        let back = world.observe(near).unwrap();
        assert!(
            back.text.contains("A fox is on the valley floor here."),
            "{}",
            back.text
        );
        assert!(
            back.text.contains("A fox carcass lies here."),
            "{}",
            back.text
        );
        let arrived = "A fox is on the valley floor again.";
        let near_again: Vec<_> = back
            .events
            .iter()
            .filter(|event| event.text == arrived)
            .collect();
        assert_eq!(near_again.len(), 1);
        let seen_back = world.observe(seen).unwrap();
        assert!(
            seen_back
                .text
                .contains("A fox is on the valley floor to the south and east."),
            "{}",
            seen_back.text
        );
        assert_eq!(
            seen_back
                .events
                .iter()
                .filter(|event| event.text == arrived)
                .count(),
            1
        );
        assert!(seen_back
            .events
            .iter()
            .all(|event| !event.text.starts_with("You hear a fox")));
        let heard_back = world.observe(heard).unwrap();
        assert!(!heard_back.text.contains("A fox is on the valley floor"));
        assert_eq!(
            heard_back
                .events
                .iter()
                .filter(|event| event.text == "You hear a fox to the south.")
                .count(),
            1
        );
        assert!(heard_back.events.iter().all(|event| event.text != arrived));
        assert!(world
            .observe(quiet)
            .unwrap()
            .events
            .iter()
            .all(|event| !event.text.contains("fox")));
        assert!(!world
            .observe(quiet)
            .unwrap()
            .text
            .contains("A fox is on the valley floor"));
        world.step(&[]);
        assert_eq!(world.fox_cell(), Some((5, 4)));
        assert_eq!(
            world
                .observe(near)
                .unwrap()
                .events
                .iter()
                .filter(|event| event.text == arrived)
                .count(),
            1
        );

        let again = world.step(&[intent("again", near, Attempt::Hunt)]);
        assert_eq!(
            again.results[0].message,
            "You bring the fox down with the flint spear."
        );
        assert!(world.fox_cell().is_none());
        let second = world.fox_returns_at.unwrap();
        assert_eq!(second, world.time + world.tuning.deer_ticks - 1);
        while world.time < second {
            world.step(&[]);
        }
        assert!(world.fox_cell().is_none());
        world.step(&[]);
        assert_eq!(world.fox_cell(), Some((5, 4)));
        let off = world.step(&[intent("off", near, Attempt::Move(super::Dir::West))]);
        assert_eq!(off.results[0].message, "You walk west.");
        assert_eq!(world.fox_cell(), Some((5, 4)));
        let onto = world.step(&[intent("onto", near, Attempt::Move(super::Dir::East))]);
        assert_eq!(onto.results[0].message, "You walk east.");
        assert!(!onto.results[0].message.contains("fox"));
        assert_eq!(world.fox_cell(), Some((6, 4)));
        let slipped = world.observe(near).unwrap();
        assert!(slipped
            .events
            .iter()
            .any(|event| event.text == "A fox slips away to the east."));
        assert!(
            slipped
                .text
                .contains("A fox is on the valley floor to the east."),
            "{}",
            slipped.text
        );
        assert!(world
            .observe(quiet)
            .unwrap()
            .events
            .iter()
            .all(|event| event.text != "A fox slips away to the east."));
    }

    #[test]
    fn a_spear_from_the_next_cell_can_miss_the_fox() {
        let mut found = None;
        for seed in 1..40 {
            let hash = super::contest_hash(seed, 0, "hunt", "5,4:fox", 1);
            if hash % 3 == 1 {
                found = Some(seed);
                break;
            }
        }
        let seed = found.expect("a miss seed");
        let mut world = World::valley(crate::tuning::Tuning::evening(), seed);
        let near = world.spawn_at("Near", "n", 5, 3).unwrap();
        let seen = world.spawn_at("Seen", "s", 4, 4).unwrap();
        let heard = world.spawn_at("Heard", "h", 2, 4).unwrap();
        let quiet = world.spawn_at("Quiet", "q", 0, 4).unwrap();
        flint_spear(&mut world, near);
        let shot = world.step(&[intent("miss", near, Attempt::Hunt)]);
        assert_eq!(
            shot.results[0].message,
            "You miss the fox with the flint spear. It slips away to the east."
        );
        assert!(!shot.results[0].success);
        assert_eq!(world.fox_cell(), Some((6, 4)));
        assert!(world
            .items
            .values()
            .all(|item| item.material != Material::Carcass));
        assert!(world
            .observe(near)
            .unwrap()
            .events
            .iter()
            .all(|event| !event.text.contains("fox")));
        assert!(world.observe(seen).unwrap().events.iter().any(|event| {
            event.text == "Near misses the fox with the flint spear. It slips away to the east."
        }));
        let ears: Vec<_> = world
            .observe(heard)
            .unwrap()
            .events
            .iter()
            .filter(|event| event.text.contains("fox"))
            .map(|event| event.text.clone())
            .collect();
        assert_eq!(ears, vec!["You hear a fox to the east.".to_string()]);
        assert!(world
            .observe(quiet)
            .unwrap()
            .events
            .iter()
            .all(|event| !event.text.contains("fox")));
    }

    #[test]
    fn a_fox_out_of_reach_stays_with_the_hunter() {
        let mut world = World::valley(crate::tuning::Tuning::evening(), 1);
        let near = world.spawn_at("Near", "n", 6, 2).unwrap();
        let seen = world.spawn_at("Seen", "s", 4, 2).unwrap();
        let quiet = world.spawn_at("Quiet", "q", 0, 4).unwrap();
        flint_spear(&mut world, near);
        let far = world.step(&[intent("far", near, Attempt::Hunt)]);
        assert_eq!(
            far.results[0].message,
            "The fox is not close enough to hunt."
        );
        assert_eq!(world.fox_cell(), Some((5, 4)));
        assert!(world
            .observe(near)
            .unwrap()
            .events
            .iter()
            .any(|event| { event.text == "The fox is not close enough to hunt." }));
        assert!(world
            .observe(seen)
            .unwrap()
            .events
            .iter()
            .all(|event| event.text != "The fox is not close enough to hunt."));
        assert!(world
            .observe(quiet)
            .unwrap()
            .events
            .iter()
            .all(|event| event.text != "The fox is not close enough to hunt."));
    }
}

#[cfg(test)]
mod otter_hunt {
    use super::{Attempt, Material, Role, World};

    fn intent(id: &str, actor: u32, attempt: Attempt) -> super::Intent {
        super::Intent {
            command_id: id.into(),
            actor_id: actor,
            attempt,
        }
    }

    fn park_the_deer(world: &mut World) {
        let deer = world.deer.as_mut().unwrap();
        deer.cell = super::cell_id(6, 4).unwrap();
    }

    fn flint_spear(world: &mut World, actor: u32) {
        let cell = world.actor(actor).unwrap().cell;
        let spear = world.mint_item(Material::Spear, 0, cell, Some(actor), Role::Loose);
        let point = world.items.get_mut(&spear).unwrap();
        point.hardened = true;
        point.flint_tip = true;
    }

    #[test]
    fn a_flint_spear_or_a_bow_brings_the_otter_down() {
        assert!(matches!(
            crate::parse::parse_attempt("hunt otter").unwrap(),
            Attempt::Hunt
        ));
        let mut world = World::valley(crate::tuning::Tuning::evening(), 1);
        park_the_deer(&mut world);
        let near = world.spawn_at("Near", "n", 1, 1).unwrap();
        let seen = world.spawn_at("Seen", "s", 2, 1).unwrap();
        let quiet = world.spawn_at("Quiet", "q", 6, 4).unwrap();
        let bare = world.step(&[intent("bare", near, Attempt::Hunt)]);
        assert_eq!(bare.results[0].message, "You are not carrying a spear.");
        assert!(world
            .observe(seen)
            .unwrap()
            .events
            .iter()
            .all(|event| event.text != "You are not carrying a spear."));
        flint_spear(&mut world, near);
        let shot = world.step(&[intent("shot", near, Attempt::Hunt)]);
        assert_eq!(
            shot.results[0].message,
            "You bring the otter down with the flint spear."
        );
        assert!(shot.results[0].success);
        assert!(world.otter_cell().is_none());
        let here = world.observe(near).unwrap().text;
        assert!(here.contains("An otter carcass lies here."), "{here}");
        assert!(!here.contains("A deer carcass"), "{here}");
        assert!(!here.contains("An otter is in the brook"), "{here}");
        let west = world.observe(seen).unwrap().text;
        assert!(west.contains("West: brook, an otter carcass"), "{west}");
        let body = world.examine(near, super::ExamineTarget::Carcass);
        assert!(
            body.contains("An otter carcass lies in contact with the ground"),
            "{body}"
        );
        assert!(world
            .observe(near)
            .unwrap()
            .events
            .iter()
            .all(|event| !event.text.contains("otter")));
        let heard = "Near brings the otter down with the flint spear.";
        assert!(world
            .observe(seen)
            .unwrap()
            .events
            .iter()
            .any(|event| event.text == heard));
        assert!(world
            .observe(quiet)
            .unwrap()
            .events
            .iter()
            .all(|event| !event.text.contains("otter")));
        let left = world.step(&[intent("butcher", near, Attempt::Butcher)]);
        assert_eq!(
            left.results[0].message,
            "You butcher the otter carcass into a portion of raw meat, an otter hide, and a bone."
        );
        assert!(left.results[0].success);
        let cut = world.observe(near).unwrap();
        assert!(
            cut.text.contains("A portion of raw meat lies here."),
            "{}",
            cut.text
        );
        assert!(
            cut.text.contains("An otter hide lies here."),
            "{}",
            cut.text
        );
        assert!(cut.text.contains("A bone lies here."), "{}", cut.text);
        assert!(!cut.text.contains("An otter carcass"), "{}", cut.text);
        assert!(!cut.text.contains("deer hide"), "{}", cut.text);
        assert!(cut
            .events
            .iter()
            .all(|event| !event.text.contains("butcher")));
        let butchered =
            "Near butchers the otter carcass into a portion of raw meat, an otter hide, and a bone.";
        assert!(world
            .observe(seen)
            .unwrap()
            .events
            .iter()
            .any(|event| event.text == butchered));
        assert!(world
            .observe(quiet)
            .unwrap()
            .events
            .iter()
            .all(|event| !event.text.contains("butcher") && !event.text.contains("otter hide")));
        let floor = world.actor(near).unwrap().cell;
        assert_eq!(world.ground_ids(floor, Material::Meat).len(), 1);
        assert!(world
            .ground_ids(floor, Material::Hide)
            .iter()
            .any(|id| world.item(*id).is_some_and(|hide| hide.otter && !hide.wolf)));

        let mut world = World::valley(crate::tuning::Tuning::evening(), 1);
        park_the_deer(&mut world);
        let near = world.spawn_at("Near", "n", 3, 1).unwrap();
        let seen = world.spawn_at("Seen", "s", 2, 1).unwrap();
        let quiet = world.spawn_at("Quiet", "q", 6, 4).unwrap();
        let cell = world.actor(near).unwrap().cell;
        world.mint_item(Material::Bow, 0, cell, Some(near), Role::Loose);
        world.mint_item(Material::Flake, 0, cell, Some(near), Role::Loose);
        let shot = world.step(&[intent("bow", near, Attempt::Hunt)]);
        assert_eq!(
            shot.results[0].message,
            "You bring the otter down with the bow. The flint flake is spent."
        );
        assert!(world.otter_cell().is_none());
        assert!(world.loose_ids(near, Material::Flake).is_empty());
        assert_eq!(world.loose_ids(near, Material::Bow).len(), 1);
        assert!(world
            .observe(near)
            .unwrap()
            .events
            .iter()
            .all(|event| !event.text.contains("otter")));
        assert!(world.observe(seen).unwrap().events.iter().any(|event| {
            event.text == "Near brings the otter down with the bow. The flint flake is spent."
        }));
        assert!(world
            .observe(quiet)
            .unwrap()
            .events
            .iter()
            .all(|event| !event.text.contains("otter")));
        let water = world.observe(seen).unwrap().text;
        assert!(
            water.contains("an otter carcass") || water.contains("An otter carcass"),
            "{water}"
        );
    }

    #[test]
    fn a_spear_from_the_next_cell_can_miss_the_otter() {
        let mut found = None;
        for seed in 1..40 {
            let hash = super::contest_hash(seed, 0, "hunt", "1,1:otter", 1);
            if hash % 3 == 1 {
                found = Some(seed);
                break;
            }
        }
        let seed = found.expect("a miss seed");
        let mut world = World::valley(crate::tuning::Tuning::evening(), seed);
        park_the_deer(&mut world);
        let near = world.spawn_at("Near", "n", 0, 2).unwrap();
        let seen = world.spawn_at("Seen", "s", 0, 3).unwrap();
        let quiet = world.spawn_at("Quiet", "q", 6, 4).unwrap();
        let heard = world.spawn_at("Far", "f", 4, 1).unwrap();
        flint_spear(&mut world, near);
        let shot = world.step(&[intent("miss", near, Attempt::Hunt)]);
        assert_eq!(
            shot.results[0].message,
            "You miss the otter with the flint spear. It splashes away to the south."
        );
        assert!(!shot.results[0].success);
        assert_eq!(world.otter_cell(), Some((1, 2)));
        assert!(world.otter_returns_at.is_none());
        assert!(world
            .items
            .values()
            .all(|item| item.material != Material::Carcass));
        assert!(world
            .observe(near)
            .unwrap()
            .events
            .iter()
            .all(|event| !event.text.contains("otter") && !event.text.contains("splash")));
        assert!(world.observe(seen).unwrap().events.iter().any(|event| {
            event.text
                == "Near misses the otter with the flint spear. It splashes away to the south."
        }));
        let splash: Vec<_> = world
            .observe(heard)
            .unwrap()
            .events
            .iter()
            .filter(|event| event.text.contains("splash"))
            .map(|event| event.text.clone())
            .collect();
        assert_eq!(splash, vec!["You hear a splash to the west.".to_string()]);
        assert!(world
            .observe(quiet)
            .unwrap()
            .events
            .iter()
            .all(|event| !event.text.contains("otter") && !event.text.contains("splash")));
    }

    #[test]
    fn a_knife_is_named_when_both_tools_butcher_the_otter() {
        let mut world = World::valley(crate::tuning::Tuning::evening(), 1);
        let near = world.spawn_at("Near", "n", 2, 2).unwrap();
        let seen = world.spawn_at("Seen", "s", 3, 2).unwrap();
        let cell = world.actor(near).unwrap().cell;
        let carcass = world.mint_item(Material::Carcass, 0, cell, None, Role::Loose);
        world.items.get_mut(&carcass).unwrap().otter = true;
        flint_spear(&mut world, near);
        world.mint_item(Material::Knife, 0, cell, Some(near), Role::Loose);
        let both = world.step(&[intent("both", near, Attempt::Butcher)]);
        assert_eq!(
            both.results[0].message,
            "You butcher the otter carcass with the iron knife into a portion of raw meat, an otter hide, and a bone."
        );
        assert_eq!(world.loose_ids(near, Material::Knife).len(), 1);
        assert_eq!(world.loose_ids(near, Material::Spear).len(), 1);
        assert!(world
            .observe(near)
            .unwrap()
            .events
            .iter()
            .all(|event| !event.text.contains("butcher")));
        assert!(world.observe(seen).unwrap().events.iter().any(|event| {
            event.text == "Near butchers the otter carcass with the iron knife into a portion of raw meat, an otter hide, and a bone."
        }));
        let bare = world.step(&[intent("again", near, Attempt::Butcher)]);
        assert_eq!(
            bare.results[0].message,
            "There is no carcass here to butcher."
        );
        assert!(world
            .observe(seen)
            .unwrap()
            .events
            .iter()
            .all(|event| event.text != "There is no carcass here to butcher."));
        let taken = world.step(&[intent("hide", near, Attempt::Gather(Material::Hide))]);
        assert_eq!(taken.results[0].message, "You gather an otter hide.");
        let arms = world.observe(near).unwrap().text;
        assert!(arms.contains("an otter hide"), "{arms}");
        assert!(!arms.contains("deer hide"), "{arms}");
        assert!(world
            .observe(near)
            .unwrap()
            .events
            .iter()
            .all(|event| !event.text.contains("You gather an otter hide.")));
    }

    #[test]
    fn an_otter_hide_sews_into_a_cloak_that_keeps_off_the_wet() {
        let mut world = World::valley(crate::tuning::Tuning::evening(), 1);
        let near = world.spawn_at("Near", "n", 4, 3).unwrap();
        let seen = world.spawn_at("Seen", "s", 4, 3).unwrap();
        let quiet = world.spawn_at("Quiet", "q", 0, 4).unwrap();
        let cell = world.actor(near).unwrap().cell;
        let hide = world.mint_item(Material::Hide, 0, cell, Some(near), Role::Loose);
        world.items.get_mut(&hide).unwrap().otter = true;
        let awl = world.mint_item(Material::Awl, 0, cell, Some(near), Role::Loose);
        world.items.get_mut(&awl).unwrap().bone = true;

        let early = world.step(&[intent("sew", near, Attempt::Sew)]);
        assert_eq!(early.results[0].message, "The otter hide is not pierced.");
        assert!(!early.results[0].success);
        assert!(world
            .observe(near)
            .unwrap()
            .events
            .iter()
            .all(|event| { event.text != "The otter hide is not pierced." }));
        assert!(world
            .observe(seen)
            .unwrap()
            .events
            .iter()
            .all(|event| !event.text.contains("otter")));

        let pierced = world.step(&[intent("pierce", near, Attempt::Pierce)]);
        assert_eq!(
            pierced.results[0].message,
            "You pierce the otter hide with the bone awl. It is ready to sew."
        );
        assert!(world
            .observe(near)
            .unwrap()
            .events
            .iter()
            .all(|event| !event.text.contains("pierce") && !event.text.contains("otter hide")));
        assert!(world.observe(seen).unwrap().events.iter().any(|event| {
            event.text == "Near pierces the otter hide with the bone awl. It is ready to sew."
        }));
        assert!(world
            .observe(quiet)
            .unwrap()
            .events
            .iter()
            .all(|event| !event.text.contains("otter") && !event.text.contains("pierce")));

        let sewn = world.step(&[intent("sew-cloak", near, Attempt::Sew)]);
        assert_eq!(
            sewn.results[0].message,
            "You sew the pierced otter hide into a cloak."
        );
        let again = world.step(&[intent("again", near, Attempt::Sew)]);
        assert_eq!(
            again.results[0].message,
            "The otter hide is already a cloak."
        );
        assert!(world
            .observe(seen)
            .unwrap()
            .events
            .iter()
            .all(|event| event.text != "The otter hide is already a cloak."));
        let wore = world.step(&[intent("wear", near, Attempt::Wear)]);
        assert_eq!(
            wore.results[0].message,
            "You wear the sewn otter-hide cloak."
        );
        let look = world.observe(near).unwrap().text;
        assert!(
            look.contains("The sewn otter-hide cloak is on you."),
            "{look}"
        );
        assert!(look.contains("a sewn otter-hide cloak"), "{look}");
        assert!(!look.contains("deer-hide"), "{look}");
        let held = world.examine(near, super::ExamineTarget::Hide);
        assert!(
            held.contains(
                "A sewn otter-hide cloak is on you, in the open. It keeps the rain and the brook off."
            ),
            "{held}"
        );
        assert!(world
            .observe(seen)
            .unwrap()
            .events
            .iter()
            .any(|event| { event.text == "Near sews the pierced otter hide into a cloak." }));
        assert!(world
            .observe(seen)
            .unwrap()
            .events
            .iter()
            .any(|event| { event.text == "Near wears the sewn otter-hide cloak." }));
        assert!(world.observe(near).unwrap().events.iter().all(|event| {
            !event.text.contains("sewn otter-hide") && !event.text.contains("pierced otter")
        }));

        let deer = world.spawn_at("Deer", "d", 1, 3).unwrap();
        let bank = world.actor(deer).unwrap().cell;
        let deer_hide = world.mint_item(Material::Hide, 0, bank, Some(deer), Role::Loose);
        world.items.get_mut(&deer_hide).unwrap().sewn = true;
        world.items.get_mut(&deer_hide).unwrap().worn = true;
        world.actors.get_mut(&near).unwrap().cell = bank;
        world.actors.get_mut(&near).unwrap().wetness = 0;
        world.actors.get_mut(&near).unwrap().cold = 0;
        world.actors.get_mut(&deer).unwrap().wetness = 0;
        world.actors.get_mut(&deer).unwrap().cold = 0;
        let waded = world.step(&[
            intent("wade-otter", near, Attempt::Move(super::Dir::North)),
            intent("wade-deer", deer, Attempt::Move(super::Dir::North)),
        ]);
        assert!(
            waded.results.iter().all(|result| result.success),
            "{waded:?}"
        );
        let otter_wet = world.actor(near).unwrap().wetness;
        let deer_wet = world.actor(deer).unwrap().wetness;
        assert_eq!(deer_wet, world.tuning.wade);
        assert_eq!(otter_wet, world.tuning.wade / 2);
        assert!(otter_wet < deer_wet);

        world.actors.get_mut(&near).unwrap().cell = cell;
        world.actors.get_mut(&deer).unwrap().cell = cell;
        world.actors.get_mut(&near).unwrap().wetness = 0;
        world.actors.get_mut(&deer).unwrap().wetness = 0;
        world.actors.get_mut(&near).unwrap().cold = 0;
        world.actors.get_mut(&deer).unwrap().cold = 0;
        world.time = world.tuning.rain_starts_at;
        world.step(&[]);
        world.step(&[]);
        let otter_cold = world.actor(near).unwrap().cold;
        let deer_cold = world.actor(deer).unwrap().cold;
        let otter_rain = world.actor(near).unwrap().wetness;
        let deer_rain = world.actor(deer).unwrap().wetness;
        assert!(
            otter_cold < deer_cold,
            "otter {otter_cold} deer {deer_cold}"
        );
        assert!(
            otter_rain < deer_rain,
            "otter {otter_rain} deer {deer_rain}"
        );
        assert_eq!(otter_rain * 2, deer_rain);
    }

    #[test]
    fn an_otter_is_in_the_brook_again_after_the_same_wait_as_the_deer() {
        let mut world = World::valley(crate::tuning::Tuning::evening(), 1);
        park_the_deer(&mut world);
        let near = world.spawn_at("Near", "n", 1, 1).unwrap();
        let seen = world.spawn_at("Seen", "s", 2, 1).unwrap();
        let quiet = world.spawn_at("Quiet", "q", 6, 4).unwrap();
        flint_spear(&mut world, near);
        let shot = world.step(&[intent("shot", near, Attempt::Hunt)]);
        assert_eq!(
            shot.results[0].message,
            "You bring the otter down with the flint spear."
        );
        assert!(world.otter_cell().is_none());
        assert_eq!(world.otter_returns_at, Some(world.tuning.deer_ticks));
        let ticks = world.tuning.deer_ticks;
        for _ in 0..ticks - 1 {
            world.step(&[]);
        }
        assert!(world.otter_cell().is_none());
        let waiting = world.observe(near).unwrap().text;
        assert!(!waiting.contains("An otter is in the brook"), "{waiting}");
        assert!(waiting.contains("An otter carcass lies here."), "{waiting}");
        world.step(&[]);
        assert_eq!(world.otter_cell(), Some((0, 1)));
        assert!(world.otter_returns_at.is_none());
        let back = world.observe(near).unwrap();
        assert!(
            back.text.contains("An otter is in the brook to the west."),
            "{}",
            back.text
        );
        assert!(
            back.text.contains("An otter carcass lies here."),
            "{}",
            back.text
        );
        let arrived = "An otter is in the brook again.";
        assert!(back.events.iter().any(|event| event.text == arrived));
        let seen_back = world.observe(seen).unwrap();
        assert!(
            seen_back
                .text
                .contains("An otter is in the brook to the west."),
            "{}",
            seen_back.text
        );
        assert!(seen_back.events.iter().any(|event| event.text == arrived));
        assert!(world
            .observe(quiet)
            .unwrap()
            .events
            .iter()
            .all(|event| event.text != arrived));
        assert!(!world
            .observe(quiet)
            .unwrap()
            .text
            .contains("An otter is in the brook"));
        world.step(&[]);
        assert_eq!(world.otter_cell(), Some((0, 1)));
    }

    #[test]
    fn a_bone_on_the_ground_is_gathered() {
        let mut world = World::valley(crate::tuning::Tuning::evening(), 1);
        let near = world.spawn_at("Near", "n", 4, 3).unwrap();
        let seen = world.spawn_at("Seen", "s", 4, 3).unwrap();
        let quiet = world.spawn_at("Quiet", "q", 0, 4).unwrap();
        let cell = world.actor(near).unwrap().cell;
        let none = world.step(&[intent("none", near, Attempt::Gather(Material::Bone))]);
        assert_eq!(none.results[0].message, "There is no bone here to gather.");
        assert!(!none.results[0].success);
        assert!(world
            .observe(seen)
            .unwrap()
            .events
            .iter()
            .all(|event| event.text != "There is no bone here to gather."));
        assert!(world
            .observe(quiet)
            .unwrap()
            .events
            .iter()
            .all(|event| event.text != "There is no bone here to gather."));
        world.mint_item(Material::Bone, 0, cell, None, Role::Loose);
        let before = world.observe(near).unwrap().text;
        assert!(before.contains("A bone lies here."), "{before}");
        assert!(!before.contains("You have a bone"), "{before}");
        let taken = world.step(&[intent("bone", near, Attempt::Gather(Material::Bone))]);
        assert_eq!(taken.results[0].message, "You gather a bone.");
        assert!(world.ground_ids(cell, Material::Bone).is_empty());
        assert_eq!(world.loose_ids(near, Material::Bone).len(), 1);
        let after = world.observe(near).unwrap().text;
        assert!(!after.contains("A bone lies here."), "{after}");
        assert!(after.contains("a bone"), "{after}");
        assert!(world.observe(near).unwrap().events.iter().all(|event| {
            event.text != "You gather a bone." && event.text != "Near gathers a bone."
        }));
        assert!(world
            .observe(seen)
            .unwrap()
            .events
            .iter()
            .any(|event| { event.text == "Near gathers a bone." }));
        assert!(world
            .observe(quiet)
            .unwrap()
            .events
            .iter()
            .all(|event| !event.text.contains("bone")));
    }
}

#[cfg(test)]
mod wooden_spear {
    use super::*;

    fn intent(id: &str, actor: u32, attempt: Attempt) -> Intent {
        Intent {
            command_id: id.into(),
            actor_id: actor,
            attempt,
        }
    }

    #[test]
    fn a_dry_branch_points_into_a_wooden_spear_and_the_fire_hardens_it() {
        let mut world = World::valley(crate::tuning::Tuning::evening(), 1);
        let near = world.spawn_at("Near", "n", 6, 1).unwrap();
        let seen = world.spawn_at("Seen", "s", 5, 1).unwrap();
        let quiet = world.spawn_at("Quiet", "q", 0, 4).unwrap();
        let none = world.step(&[intent("none", near, Attempt::PointSpear)]);
        assert_eq!(none.results[0].message, "You have no branch to point.");
        assert!(world
            .observe(seen)
            .unwrap()
            .events
            .iter()
            .all(|event| event.text != "You have no branch to point."));
        world.step(&[intent("branch", near, Attempt::Gather(Material::Branch))]);
        let pointed = world.step(&[intent("point", near, Attempt::PointSpear)]);
        assert_eq!(
            pointed.results[0].message,
            "You point a dry branch into a wooden spear."
        );
        let arms = world.observe(near).unwrap().text;
        assert!(arms.contains("a wooden spear with a green point"), "{arms}");
        assert!(world.observe(near).unwrap().events.iter().all(|event| {
            event.text != "You point a dry branch into a wooden spear."
                && event.text != "Near points a dry branch into a wooden spear."
        }));
        assert!(world
            .observe(seen)
            .unwrap()
            .events
            .iter()
            .any(|event| { event.text == "Near points a dry branch into a wooden spear." }));
        assert!(world
            .observe(quiet)
            .unwrap()
            .events
            .iter()
            .all(|event| !event.text.contains("wooden spear")));
        let green = world.step(&[intent("green", near, Attempt::Hunt)]);
        assert_eq!(
            green.results[0].message,
            "The wooden point is not hardened."
        );
        let cold = world.step(&[intent("cold", near, Attempt::HardenSpear)]);
        assert_eq!(cold.results[0].message, "There is no fire burning here.");
        assert!(world
            .observe(seen)
            .unwrap()
            .events
            .iter()
            .all(|event| { event.text != "There is no fire burning here." }));
        world.step(&[intent("light", near, Attempt::Light)]);
        let held = world.step(&[intent("hold", near, Attempt::HardenSpear)]);
        assert_eq!(
            held.results[0].message,
            "You hold the wooden point in the fire. It is not hard yet."
        );
        assert!(world
            .observe(near)
            .unwrap()
            .events
            .iter()
            .all(|event| { !event.text.contains("hold the wooden point") }));
        assert!(world
            .observe(seen)
            .unwrap()
            .events
            .iter()
            .any(|event| { event.text == "Near holds the wooden point in the fire." }));
        assert!(world
            .observe(quiet)
            .unwrap()
            .events
            .iter()
            .all(|event| !event.text.contains("wooden point")));
        let waiting = world.observe(near).unwrap().text;
        assert!(
            waiting.contains("The wooden point of your spear is hardening in the fire."),
            "{waiting}"
        );
        let again = world.step(&[intent("again", near, Attempt::HardenSpear)]);
        assert_eq!(
            again.results[0].message,
            "The wooden point is already hardening."
        );
        for _ in 0..4 {
            world.step(&[]);
        }
        let hard = world.observe(near).unwrap();
        assert!(
            hard.text
                .contains("a wooden spear with a fire-hardened point"),
            "{}",
            hard.text
        );
        assert!(hard
            .events
            .iter()
            .any(|event| event.text == "The wooden point of your spear hardens."));
        assert!(world
            .observe(seen)
            .unwrap()
            .events
            .iter()
            .any(|event| { event.text == "The wooden point of Near's spear hardens." }));
        assert!(world
            .observe(quiet)
            .unwrap()
            .events
            .iter()
            .all(|event| !event.text.contains("wooden point")));
        let examined = world.examine(near, ExamineTarget::Spear);
        assert!(
            examined.contains("A wooden spear with a fire-hardened point is carried by you"),
            "{examined}"
        );
        let done = world.step(&[intent("done", near, Attempt::HardenSpear)]);
        assert_eq!(
            done.results[0].message,
            "The wooden point is already hardened."
        );
        world.fox = None;
        world.otter = None;
        world.wolf_cell = None;
        let here = world.actor(near).unwrap().cell;
        let (x, y) = cell_xy(here);
        let next = cell_id(x, y + 1).unwrap();
        world.deer.as_mut().unwrap().cell = next;
        let aside = world.step(&[intent("aside", near, Attempt::Hunt)]);
        assert_eq!(
            aside.results[0].message,
            "The deer is not close enough to hunt."
        );
        assert!(world
            .observe(near)
            .unwrap()
            .events
            .iter()
            .any(|event| { event.text == "The deer is not close enough to hunt." }));
        assert!(world
            .observe(seen)
            .unwrap()
            .events
            .iter()
            .all(|event| { event.text != "The deer is not close enough to hunt." }));
        assert!(world
            .observe(quiet)
            .unwrap()
            .events
            .iter()
            .all(|event| { event.text != "The deer is not close enough to hunt." }));
        world.deer.as_mut().unwrap().cell = here;
        let down = world.step(&[intent("down", near, Attempt::Hunt)]);
        assert_eq!(
            down.results[0].message,
            "You bring the deer down. A carcass lies where it stood."
        );
        assert!(world.observe(near).unwrap().events.iter().all(|event| {
            event.text != "Near brings the deer down."
                && event.text != "You bring the deer down. A carcass lies where it stood."
        }));
        assert!(world
            .observe(seen)
            .unwrap()
            .events
            .iter()
            .any(|event| { event.text == "Near brings the deer down." }));
        assert!(world
            .observe(quiet)
            .unwrap()
            .events
            .iter()
            .all(|event| { event.text != "Near brings the deer down." }));
    }
}

#[cfg(test)]
mod charcoal_bank {
    use super::{cell_id, Attempt, ExamineTarget, Intent, Material, Role, Wake, World};

    fn intent(id: &str, actor: u32, attempt: Attempt) -> Intent {
        Intent {
            command_id: id.into(),
            actor_id: actor,
            attempt,
        }
    }

    #[test]
    fn mud_covers_dry_branches_on_a_fire_and_they_char() {
        assert!(matches!(
            crate::parse::parse_attempt("bank").unwrap(),
            Attempt::Bank
        ));
        assert!(matches!(
            crate::parse::parse_attempt("smother").unwrap(),
            Attempt::Bank
        ));
        assert!(matches!(
            crate::parse::parse_attempt("cover fire").unwrap(),
            Attempt::Bank
        ));
        let mut world = World::valley(crate::tuning::Tuning::evening(), 1);
        let cell = cell_id(4, 4).unwrap();
        let near = world.spawn_at("Near", "n", 4, 4).unwrap();
        let seen = world.spawn_at("Seen", "s", 4, 2).unwrap();
        let quiet = world.spawn_at("Quiet", "q", 0, 0).unwrap();
        let bare = world.step(&[intent("bare", near, Attempt::Bank)]);
        assert_eq!(bare.results[0].message, "There is no fire burning here.");
        assert!(world
            .observe(near)
            .unwrap()
            .events
            .iter()
            .any(|event| { event.text == "There is no fire burning here." }));
        assert!(world
            .observe(seen)
            .unwrap()
            .events
            .iter()
            .all(|event| { event.text != "There is no fire burning here." }));
        assert!(world
            .observe(quiet)
            .unwrap()
            .events
            .iter()
            .all(|event| { event.text != "There is no fire burning here." }));
        world.mint_fire(cell, 0);
        world.add_pile(cell, Material::Branch, 3, 0);
        let soft = world.step(&[intent("soft", near, Attempt::Bank)]);
        assert_eq!(
            soft.results[0].message,
            "You have no mud to cover the fire."
        );
        assert!(world
            .observe(seen)
            .unwrap()
            .events
            .iter()
            .all(|event| { event.text != soft.results[0].message }));
        world.mint_item(Material::Mud, 0, cell, Some(near), Role::Loose);
        let banked = world.step(&[intent("banked", near, Attempt::Bank)]);
        let yours = "You bank the branches on the fire and cover them with mud.";
        let witnessed = "Near banks the branches on the fire and cover them with mud.";
        assert_eq!(banked.results[0].message, yours);
        assert_eq!(world.dry_wood_on_ground(cell), 0);
        assert!(world.loose_ids(near, Material::Mud).is_empty());
        assert!(world.fire_at(cell).is_some());
        let here = world.observe(near).unwrap();
        assert!(
            here.text
                .contains("Branches are banked on the fire under a cover of mud."),
            "{}",
            here.text
        );
        assert!(here
            .events
            .iter()
            .all(|event| { event.text != yours && event.text != witnessed }));
        assert!(world
            .observe(seen)
            .unwrap()
            .events
            .iter()
            .any(|event| { event.text == witnessed }));
        assert!(world
            .observe(quiet)
            .unwrap()
            .events
            .iter()
            .all(|event| { !event.text.contains("bank") }));
        let again = world.step(&[intent("again", near, Attempt::Bank)]);
        assert_eq!(again.results[0].message, "The fire is already banked.");
        assert!(world
            .observe(seen)
            .unwrap()
            .events
            .iter()
            .all(|event| { event.text != again.results[0].message }));
        let mut charred = None;
        for _ in 0..6 {
            world.step(&[]);
            if world.fire_at(cell).is_none() {
                charred = Some(world.time);
                break;
            }
        }
        assert!(charred.is_some());
        let line = "The banked wood chars into charcoal. The fire is out.";
        assert!(world
            .observe(near)
            .unwrap()
            .events
            .iter()
            .any(|event| { event.text == line }));
        assert!(world
            .observe(seen)
            .unwrap()
            .events
            .iter()
            .any(|event| { event.text == line }));
        assert!(world
            .observe(quiet)
            .unwrap()
            .events
            .iter()
            .all(|event| { event.text != line }));
        let ground = world.observe(near).unwrap();
        assert!(
            ground.text.contains("Lumps of charcoal lie here."),
            "{}",
            ground.text
        );
        assert!(
            !ground.text.contains("A small fire burns"),
            "{}",
            ground.text
        );
        assert_eq!(world.ground_ids(cell, Material::Charcoal).len(), 3);
        let taken = world.step(&[intent("taken", near, Attempt::Gather(Material::Charcoal))]);
        assert_eq!(taken.results[0].message, "You gather a lump of charcoal.");
        assert!(world
            .observe(near)
            .unwrap()
            .events
            .iter()
            .all(|event| { event.text != taken.results[0].message }));
        assert!(world
            .observe(seen)
            .unwrap()
            .events
            .iter()
            .any(|event| { event.text == "Near gathers a lump of charcoal." }));
    }

    #[test]
    fn a_fire_that_dies_gives_the_banked_wood_back() {
        let mut world = World::valley(crate::tuning::Tuning::evening(), 1);
        let cell = cell_id(4, 4).unwrap();
        let near = world.spawn_at("Near", "n", 4, 4).unwrap();
        let seen = world.spawn_at("Seen", "s", 4, 2).unwrap();
        world.mint_fire(cell, 0);
        world.add_pile(cell, Material::Branch, 2, 1000);
        world.mint_item(Material::Mud, 0, cell, Some(near), Role::Loose);
        let wet = world.step(&[intent("wet", near, Attempt::Bank)]);
        assert_eq!(wet.results[0].message, "The wood is too wet to bank.");
        assert!(world
            .observe(seen)
            .unwrap()
            .events
            .iter()
            .all(|event| { event.text != wet.results[0].message }));
        assert_eq!(world.pile_count(cell, Material::Branch), 2);
        world.take_from_pile(cell, Material::Branch, 2);
        world.add_pile(cell, Material::Branch, 2, 0);
        let banked = world.step(&[intent("banked", near, Attempt::Bank)]);
        assert_eq!(
            banked.results[0].message,
            "You bank the branches on the fire and cover them with mud."
        );
        let id = world.fire_at(cell).unwrap().id;
        world.extinguish(id, "Rain puts out the fire.", None);
        assert_eq!(world.ground_ids(cell, Material::Charcoal).len(), 0);
        let mut restored = false;
        for _ in 0..6 {
            world.step(&[]);
            if world
                .observe(near)
                .unwrap()
                .events
                .iter()
                .any(|event| event.text == "The banked wood did not char. The fire is out.")
            {
                restored = true;
                break;
            }
        }
        assert!(restored);
        assert_eq!(world.dry_wood_on_ground(cell), 2);
        assert_eq!(world.ground_ids(cell, Material::Charcoal).len(), 0);
        assert!(world
            .observe(seen)
            .unwrap()
            .events
            .iter()
            .any(|event| { event.text == "The banked wood did not char. The fire is out." }));
    }

    #[test]
    fn a_dying_fire_is_held_until_the_banked_wood_chars() {
        let mut world = World::valley(crate::tuning::Tuning::evening(), 1);
        let cell = cell_id(4, 4).unwrap();
        let near = world.spawn_at("Near", "n", 4, 4).unwrap();
        let seen = world.spawn_at("Seen", "s", 4, 2).unwrap();
        let quiet = world.spawn_at("Quiet", "q", 0, 0).unwrap();
        world.mint_fire(cell, 0);
        let id = world.fire_at(cell).unwrap().id;
        world.fires.get_mut(&id).unwrap().goes_out_at = world.time;
        world.schedule.retain(|entry| entry.wake != Wake::Fire(id));
        world.schedule(world.time, Wake::Fire(id));
        world.add_pile(cell, Material::Branch, 2, 0);
        world.mint_item(Material::Mud, 0, cell, Some(near), Role::Loose);
        let banked = world.step(&[intent("banked", near, Attempt::Bank)]);
        let yours = "You bank the branches on the fire and cover them with mud.";
        assert_eq!(banked.results[0].message, yours);
        assert!(world.fire_at(cell).is_some());
        assert!(world.observe(near).unwrap().events.iter().all(|event| {
            event.text != yours
                && event.text != "Near banks the branches on the fire and cover them with mud."
        }));
        let mut charred = false;
        for _ in 0..8 {
            world.step(&[]);
            if world
                .observe(near)
                .unwrap()
                .events
                .iter()
                .any(|event| event.text == "The banked wood chars into charcoal. The fire is out.")
            {
                charred = true;
                break;
            }
        }
        assert!(charred);
        assert!(world
            .observe(near)
            .unwrap()
            .events
            .iter()
            .all(|event| { event.text != "The banked wood did not char. The fire is out." }));
        assert_eq!(world.ground_ids(cell, Material::Charcoal).len(), 2);
        assert!(world.observe(seen).unwrap().events.iter().any(|event| {
            event.text == "The banked wood chars into charcoal. The fire is out."
        }));
        assert!(world.observe(quiet).unwrap().events.iter().all(|event| {
            event.text != "The banked wood chars into charcoal. The fire is out."
        }));
    }

    #[test]
    fn charcoal_on_the_ground_lights_hot_even_when_a_branch_is_here() {
        let mut world = World::valley(crate::tuning::Tuning::evening(), 1);
        let cell = cell_id(4, 4).unwrap();
        let near = world.spawn_at("Near", "n", 4, 4).unwrap();
        let seen = world.spawn_at("Seen", "s", 4, 2).unwrap();
        let quiet = world.spawn_at("Quiet", "q", 0, 0).unwrap();
        world.add_pile(cell, Material::Bark, 1, 0);
        world.add_pile(cell, Material::Branch, 2, 0);
        world.mint_item(Material::Charcoal, 0, cell, None, Role::Loose);
        let lit = world.step(&[intent("lit", near, Attempt::Light)]);
        let yours = "You light a fire with bark from the pile at the feet and a lump of charcoal from the ground.";
        let witnessed = "Near lights a fire with bark from the pile at the feet and a lump of charcoal from the ground.";
        assert_eq!(lit.results[0].message, yours);
        let fire = world.fire_at(cell).unwrap();
        assert!(fire.hot && fire.charcoal);
        assert_eq!(world.ground_ids(cell, Material::Charcoal).len(), 0);
        assert_eq!(world.pile_count(cell, Material::Branch), 2);
        assert!(world
            .observe(near)
            .unwrap()
            .events
            .iter()
            .all(|event| { event.text != yours && event.text != witnessed }));
        assert!(world
            .observe(seen)
            .unwrap()
            .events
            .iter()
            .any(|event| event.text == witnessed));
        assert!(world
            .observe(quiet)
            .unwrap()
            .events
            .iter()
            .all(|event| event.text != witnessed && event.text != yours));
    }

    #[test]
    fn a_lump_of_charcoal_lights_a_hot_fire() {
        let mut world = World::valley(crate::tuning::Tuning::evening(), 1);
        let cell = cell_id(4, 4).unwrap();
        let near = world.spawn_at("Near", "n", 4, 4).unwrap();
        let seen = world.spawn_at("Seen", "s", 4, 2).unwrap();
        let quiet = world.spawn_at("Quiet", "q", 0, 0).unwrap();
        world.mint_item(Material::Charcoal, 0, cell, Some(near), Role::Loose);
        let bare = world.step(&[intent("bare", near, Attempt::Light)]);
        assert_eq!(
            bare.results[0].message,
            "You need dry loose bark for tinder."
        );
        for person in [near, seen, quiet] {
            assert!(world
                .observe(person)
                .unwrap()
                .events
                .iter()
                .all(|event| { event.text != bare.results[0].message }));
        }
        assert_eq!(world.loose_ids(near, Material::Charcoal).len(), 1);
        world.mint_item(Material::Bark, 0, cell, Some(near), Role::Loose);
        world.mint_item(Material::Branch, 0, cell, Some(near), Role::Loose);
        let lit = world.step(&[intent("lit", near, Attempt::Light)]);
        let yours = "You light a fire with the bark and a lump of charcoal from the arms.";
        let witnessed = "Near lights a fire with the bark and a lump of charcoal from the arms.";
        assert_eq!(lit.results[0].message, yours);
        assert!(world.loose_ids(near, Material::Charcoal).is_empty());
        assert!(world.loose_ids(near, Material::Bark).is_empty());
        assert_eq!(world.loose_ids(near, Material::Branch).len(), 1);
        let fire = world.fire_at(cell).unwrap();
        assert!(fire.hot && fire.charcoal);
        assert_eq!(
            fire.goes_out_at,
            world.time + world.tuning.fire_ticks * 2 - 1
        );
        let here = world.observe(near).unwrap();
        assert!(
            here.text.contains("A hot charcoal fire burns here."),
            "{}",
            here.text
        );
        assert!(!here.text.contains("A small fire burns"), "{}", here.text);
        assert!(here
            .events
            .iter()
            .all(|event| event.text != yours && event.text != witnessed));
        assert!(world
            .observe(seen)
            .unwrap()
            .events
            .iter()
            .any(|event| event.text == witnessed));
        assert!(world
            .observe(quiet)
            .unwrap()
            .events
            .iter()
            .all(|event| { !event.text.contains("lights") }));
        let named = world.examine(near, ExamineTarget::Fire);
        assert!(named.contains("A hot charcoal fire burns"), "{named}");
        let plain = cell_id(3, 4).unwrap();
        let other = world.spawn_at("Ash", "a", 3, 4).unwrap();
        world.mint_item(Material::Bark, 0, plain, Some(other), Role::Loose);
        world.mint_item(Material::Branch, 0, plain, Some(other), Role::Loose);
        let small = world.step(&[intent("small", other, Attempt::Light)]);
        assert!(
            small.results[0].message.contains("branch"),
            "{}",
            small.results[0].message
        );
        let ordinary = world.fire_at(plain).unwrap();
        assert!(!ordinary.charcoal && !ordinary.hot);
        assert_eq!(
            ordinary.goes_out_at,
            world.time + world.tuning.fire_ticks - 1
        );
        assert!(world
            .observe(other)
            .unwrap()
            .text
            .contains("A small fire burns here."));
    }

    #[test]
    fn a_hot_charcoal_fire_smelts_copper_in_half_the_wait() {
        let mut world = World::valley(crate::tuning::Tuning::evening(), 2);
        let cell = cell_id(4, 4).unwrap();
        let near = world.spawn_at("Near", "n", 4, 4).unwrap();
        let seen = world.spawn_at("Seen", "s", 4, 2).unwrap();
        let quiet = world.spawn_at("Quiet", "q", 0, 0).unwrap();
        let refused = world.step(&[intent("none", near, Attempt::Smelt)]);
        assert_eq!(
            refused.results[0].message,
            "You are not carrying copper ore."
        );
        assert!(world
            .observe(near)
            .unwrap()
            .events
            .iter()
            .any(|event| event.text == "You are not carrying copper ore."));
        for person in [seen, quiet] {
            assert!(world
                .observe(person)
                .unwrap()
                .events
                .iter()
                .all(|event| { event.text != "You are not carrying copper ore." }));
        }
        world.mint_item(Material::Bark, 0, cell, Some(near), Role::Loose);
        world.mint_item(Material::Branch, 0, cell, Some(near), Role::Loose);
        world.mint_item(Material::Charcoal, 0, cell, Some(near), Role::Loose);
        world.step(&[intent("lit", near, Attempt::Light)]);
        assert!(world.fire_at(cell).unwrap().charcoal);
        world.mint_item(Material::CopperOre, 0, cell, Some(near), Role::Loose);
        let set = world.step(&[intent("smelt", near, Attempt::Smelt)]);
        let yours =
            "You set a lump of copper ore by the fire. It leaves your arms. It is not metal yet.";
        let witnessed = "Near sets a lump of copper ore by the fire. It leaves their arms.";
        assert_eq!(set.results[0].message, yours);
        assert!(world.observe(near).unwrap().events.iter().all(|event| {
            event.text != yours && event.text != witnessed && !event.text.contains("not metal yet")
        }));
        assert!(world
            .observe(seen)
            .unwrap()
            .events
            .iter()
            .any(|event| event.text == witnessed));
        assert!(world
            .observe(quiet)
            .unwrap()
            .events
            .iter()
            .all(|event| !event.text.contains("copper")));
        let here = world.observe(near).unwrap().text;
        assert!(
            here.contains("A lump of copper ore is smelting in the hot charcoal fire."),
            "{here}"
        );
        assert!(!here.contains("is smelting in the fire."), "{here}");
        let named = world.examine(near, ExamineTarget::CopperOre);
        assert!(
            named.contains("A lump of copper ore is smelting in the hot charcoal fire."),
            "{named}"
        );
        let set_at = world.sim_time();
        world.step(&[]);
        assert_eq!(world.material_tally(Material::Copper), 0);
        assert!(world
            .observe(near)
            .unwrap()
            .text
            .contains("A lump of copper ore is smelting in the hot charcoal fire."));
        world.step(&[]);
        assert_eq!(world.sim_time(), set_at + 2);
        assert_eq!(world.material_tally(Material::Copper), 1);
        assert!(!world.observe(near).unwrap().text.contains("is smelting"));
        let bead = "The copper ore smelts into a bead.";
        assert!(world
            .observe(near)
            .unwrap()
            .events
            .iter()
            .any(|event| event.text == bead));
        assert!(world
            .observe(seen)
            .unwrap()
            .events
            .iter()
            .any(|event| event.text == bead));
        assert!(world
            .observe(quiet)
            .unwrap()
            .events
            .iter()
            .all(|event| event.text != bead));
        world.step(&[intent("bead", near, Attempt::Gather(Material::Copper))]);
        let hammered = world.step(&[intent("awl", near, Attempt::Hammer)]);
        let hammer_yours = "You hammer the copper bead into a copper awl at the fire.";
        let hammer_seen = "Near hammers the copper bead into a copper awl at the fire.";
        assert_eq!(hammered.results[0].message, hammer_yours);
        assert!(world
            .observe(near)
            .unwrap()
            .events
            .iter()
            .all(|event| { event.text != hammer_yours && event.text != hammer_seen }));
        assert!(world
            .observe(seen)
            .unwrap()
            .events
            .iter()
            .any(|event| event.text == hammer_seen));
        assert!(world.observe(near).unwrap().text.contains("a copper awl"));
    }

    #[test]
    fn smelt_names_the_step_when_the_ore_is_one_cell_off() {
        let mut world = World::valley(crate::tuning::Tuning::evening(), 1);
        let near = world.spawn_at("Near", "n", 6, 0).unwrap();
        let seen = world.spawn_at("Seen", "s", 6, 1).unwrap();
        let quiet = world.spawn_at("Quiet", "q", 0, 4).unwrap();
        let stayed = world.actor(near).unwrap().cell;
        let iron = world.step(&[intent("iron", near, Attempt::SmeltIron)]);
        assert_eq!(
            iron.results[0].message,
            "You are not carrying iron ore. Step west."
        );
        assert_eq!(world.actor(near).unwrap().cell, stayed);
        let iron_line = "You are not carrying iron ore. Step west.";
        assert!(world
            .observe(near)
            .unwrap()
            .events
            .iter()
            .any(|event| event.text == iron_line));
        for person in [seen, quiet] {
            assert!(world
                .observe(person)
                .unwrap()
                .events
                .iter()
                .all(|event| event.text != iron_line));
        }
        let rock = world.spawn_at("Rock", "r", 6, 1).unwrap();
        let under = world.step(&[intent("under", rock, Attempt::SmeltIron)]);
        assert_eq!(
            under.results[0].message,
            "You are not carrying iron ore. Step north and west."
        );
        let bank = world.spawn_at("Bank", "b", 2, 2).unwrap();
        let copper = world.step(&[intent("copper", bank, Attempt::Smelt)]);
        assert_eq!(
            copper.results[0].message,
            "You are not carrying copper ore. Step west."
        );
        assert_eq!(world.actor(bank).unwrap().cell, cell_id(2, 2).unwrap());
        let copper_line = "You are not carrying copper ore. Step west.";
        assert!(world
            .observe(bank)
            .unwrap()
            .events
            .iter()
            .any(|event| event.text == copper_line));
        assert!(world
            .observe(near)
            .unwrap()
            .events
            .iter()
            .all(|event| event.text != copper_line));
    }

    #[test]
    fn a_steep_names_what_is_missing() {
        let mut world = World::valley(crate::tuning::Tuning::evening(), 1);
        let rock = cell_id(6, 1).unwrap();
        let near = world.spawn_at("Near", "n", 6, 1).unwrap();
        let seen = world.spawn_at("Seen", "s", 4, 1).unwrap();
        let quiet = world.spawn_at("Quiet", "q", 0, 0).unwrap();
        let says = |world: &mut World, id: &str, attempt: Attempt, line: &str| {
            let step = world.step(&[intent(id, near, attempt)]);
            assert_eq!(step.results[0].message, line);
            assert!(world
                .observe(near)
                .unwrap()
                .events
                .iter()
                .any(|event| event.text == line));
            for person in [seen, quiet] {
                assert!(world
                    .observe(person)
                    .unwrap()
                    .events
                    .iter()
                    .all(|event| event.text != line));
            }
            assert_eq!(world.actor(near).unwrap().cell, rock);
        };
        says(
            &mut world,
            "ore",
            Attempt::Gather(Material::IronOre),
            "There is no iron ore here to gather. Step north and west.",
        );
        says(
            &mut world,
            "fire",
            Attempt::Brew,
            "There is no fire burning here.",
        );
        world.mint_fire(rock, 0);
        says(
            &mut world,
            "mint",
            Attempt::Brew,
            "You have no mint to steep.",
        );
        says(
            &mut world,
            "pine",
            Attempt::SteepPine,
            "You have no pine needles to steep.",
        );
        says(
            &mut world,
            "willow",
            Attempt::SteepWillow,
            "You have no willow bark to steep.",
        );
        world.mint_item(Material::Mint, 0, rock, Some(near), Role::Loose);
        says(
            &mut world,
            "vessel",
            Attempt::Brew,
            "You need a pot or a bark cup to steep the mint.",
        );
        world.mint_item(Material::Pot, 0, rock, Some(near), Role::Loose);
        says(&mut world, "pot", Attempt::Brew, "The pot has no water.");
        let pot = world.loose_ids(near, Material::Pot)[0];
        world.items.remove(&pot);
        world.mint_item(Material::BarkCup, 0, rock, Some(near), Role::Loose);
        says(
            &mut world,
            "cup",
            Attempt::Brew,
            "The bark cup has no water.",
        );
        world.mint_item(Material::Needle, 0, rock, Some(near), Role::Loose);
        says(
            &mut world,
            "needles",
            Attempt::SteepPine,
            "The bark cup has no water.",
        );
        world.mint_item(Material::WillowBark, 0, rock, Some(near), Role::Loose);
        says(
            &mut world,
            "bark",
            Attempt::SteepWillow,
            "The bark cup has no water.",
        );
        says(
            &mut world,
            "resin",
            Attempt::SealCup,
            "You have no pine resin to seal the cup.",
        );
        let cup = world.loose_ids(near, Material::BarkCup)[0];
        world.items.remove(&cup);
        says(
            &mut world,
            "gone",
            Attempt::SealCup,
            "There is no bark cup here.",
        );
    }

    #[test]
    fn an_empty_pot_names_the_step_to_drinkable_water() {
        let mut world = World::valley(crate::tuning::Tuning::evening(), 1);
        let bank = cell_id(2, 2).unwrap();
        let near = world.spawn_at("Near", "n", 2, 2).unwrap();
        let seen = world.spawn_at("Seen", "s", 3, 2).unwrap();
        let quiet = world.spawn_at("Quiet", "q", 6, 3).unwrap();
        world.mint_fire(bank, 0);
        world.mint_item(Material::Mint, 0, bank, Some(near), Role::Loose);
        world.mint_item(Material::Pot, 0, bank, Some(near), Role::Loose);
        let both = "The pot has no water. Step north or west.";
        let step = world.step(&[intent("both", near, Attempt::Brew)]);
        assert_eq!(step.results[0].message, both);
        assert!(world
            .observe(near)
            .unwrap()
            .events
            .iter()
            .any(|event| event.text == both));
        for person in [seen, quiet] {
            assert!(world
                .observe(person)
                .unwrap()
                .events
                .iter()
                .all(|event| event.text != both));
        }
        assert_eq!(world.actor(near).unwrap().cell, bank);
        assert!(world.loose_ids(near, Material::Water).is_empty());
        let pot = world.loose_ids(near, Material::Pot)[0];
        world.items.remove(&pot);
        world.mint_item(Material::BarkCup, 0, bank, Some(near), Role::Loose);
        let cup_line = "The bark cup has no water. Step north or west.";
        let in_cup = world.step(&[intent("cup", near, Attempt::Brew)]);
        assert_eq!(in_cup.results[0].message, cup_line);
        assert!(world
            .observe(seen)
            .unwrap()
            .events
            .iter()
            .all(|event| event.text != cup_line));
        let brook = cell_id(1, 2).unwrap();
        world.actors.get_mut(&near).unwrap().cell = brook;
        world.mint_fire(brook, 0);
        let here = world.step(&[intent("here", near, Attempt::Brew)]);
        assert_eq!(here.results[0].message, "The bark cup has no water.");
        assert!(world.loose_ids(near, Material::Water).is_empty());
        assert_eq!(world.actor(near).unwrap().cell, brook);

        let slope = cell_id(3, 0).unwrap();
        let west = world.spawn_at("West", "w", 3, 0).unwrap();
        world.mint_fire(slope, 0);
        world.mint_item(Material::Pot, 0, slope, Some(west), Role::Loose);
        world.mint_item(Material::Needle, 0, slope, Some(west), Role::Loose);
        let diagonal = world.step(&[intent("diag", west, Attempt::SteepPine)]);
        assert_eq!(
            diagonal.results[0].message,
            "The pot has no water. Step south and west."
        );
        assert_eq!(world.actor(west).unwrap().cell, slope);
        world.time = world.tuning.chill_at;
        let frozen = world.step(&[intent("ice", west, Attempt::SteepPine)]);
        assert_eq!(frozen.results[0].message, "The pot has no water.");
        assert_eq!(world.actor(west).unwrap().cell, slope);
        assert!(world.loose_ids(west, Material::Water).is_empty());

        world.actors.get_mut(&near).unwrap().cell = bank;
        world.mint_item(Material::Pot, 0, bank, Some(near), Role::Loose);
        let iced = "The pot has no water. The still is frozen. Step west.";
        let west_step = world.step(&[intent("west", near, Attempt::Brew)]);
        assert_eq!(west_step.results[0].message, iced);
        assert!(world
            .observe(near)
            .unwrap()
            .events
            .iter()
            .any(|event| event.text == iced));
        for person in [seen, quiet] {
            assert!(world
                .observe(person)
                .unwrap()
                .events
                .iter()
                .all(|event| event.text != iced));
        }
        assert_eq!(world.actor(near).unwrap().cell, bank);
        assert!(world.loose_ids(near, Material::Water).is_empty());
    }

    #[test]
    fn iron_needs_a_hot_charcoal_fire() {
        let mut world = World::valley(crate::tuning::Tuning::evening(), 3);
        let cell = cell_id(6, 1).unwrap();
        let near = world.spawn_at("Near", "n", 6, 1).unwrap();
        let seen = world.spawn_at("Seen", "s", 4, 1).unwrap();
        let quiet = world.spawn_at("Quiet", "q", 0, 0).unwrap();
        let bare = world.step(&[intent("bare", near, Attempt::SmeltIron)]);
        assert_eq!(
            bare.results[0].message,
            "You are not carrying iron ore. Step north and west."
        );
        assert!(world
            .observe(near)
            .unwrap()
            .events
            .iter()
            .any(|event| event.text == "You are not carrying iron ore. Step north and west."));
        for person in [seen, quiet] {
            assert!(world
                .observe(person)
                .unwrap()
                .events
                .iter()
                .all(|event| event.text != "You are not carrying iron ore. Step north and west."));
        }
        world.mint_item(Material::IronOre, 0, cell, Some(near), Role::Loose);
        world.mint_item(Material::Bark, 0, cell, Some(near), Role::Loose);
        world.mint_item(Material::Branch, 0, cell, Some(near), Role::Loose);
        world.step(&[intent("lit", near, Attempt::Light)]);
        let refused = world.step(&[intent("plain", near, Attempt::SmeltIron)]);
        assert_eq!(
            refused.results[0].message,
            "Iron needs a hot charcoal fire."
        );
        assert!(world
            .observe(near)
            .unwrap()
            .events
            .iter()
            .any(|event| event.text == "Iron needs a hot charcoal fire."));
        for person in [seen, quiet] {
            assert!(world
                .observe(person)
                .unwrap()
                .events
                .iter()
                .all(|event| { !event.text.contains("charcoal fire") }));
        }
        world.mint_item(Material::Branch, 0, cell, Some(near), Role::Loose);
        world.step(&[intent("branch", near, Attempt::Feed)]);
        let fed = world.step(&[intent("fed", near, Attempt::SmeltIron)]);
        assert_eq!(fed.results[0].message, "Iron needs a hot charcoal fire.");
        assert!(!world.fire_at(cell).unwrap().charcoal);
        let mut waits = 0;
        while !world
            .observe(near)
            .unwrap()
            .text
            .contains("A lump of charcoal lies here")
        {
            world.step(&[]);
            waits += 1;
            assert!(waits < 20);
        }
        world.step(&[intent("lump", near, Attempt::Gather(Material::Charcoal))]);
        world.mint_item(Material::Bark, 0, cell, Some(near), Role::Loose);
        let relit = world.step(&[intent("again", near, Attempt::Light)]);
        assert!(
            relit.results[0].message.contains("charcoal"),
            "{}",
            relit.results[0].message
        );
        assert!(world.fire_at(cell).unwrap().charcoal);
        let set = world.step(&[intent("smelt", near, Attempt::SmeltIron)]);
        let yours =
            "You set a lump of iron ore by the fire. It leaves your arms. It is not iron yet.";
        let witnessed = "Near sets a lump of iron ore by the fire. It leaves their arms.";
        assert_eq!(set.results[0].message, yours);
        assert!(world.observe(near).unwrap().events.iter().all(|event| {
            event.text != yours && event.text != witnessed && !event.text.contains("not iron yet")
        }));
        assert!(world
            .observe(seen)
            .unwrap()
            .events
            .iter()
            .any(|event| event.text == witnessed));
        assert!(world
            .observe(quiet)
            .unwrap()
            .events
            .iter()
            .all(|event| !event.text.contains("iron ore")));
        assert!(world
            .observe(near)
            .unwrap()
            .text
            .contains("A lump of iron ore is smelting in the hot charcoal fire."));
        for _ in 0..4 {
            world.step(&[]);
        }
        assert_eq!(world.material_tally(Material::Iron), 1);
        assert!(world
            .observe(near)
            .unwrap()
            .events
            .iter()
            .any(|event| event.text == "The iron ore smelts into a bloom."));
        assert!(world
            .observe(seen)
            .unwrap()
            .events
            .iter()
            .any(|event| event.text == "The iron ore smelts into a bloom."));
        assert!(world
            .observe(quiet)
            .unwrap()
            .events
            .iter()
            .all(|event| event.text != "The iron ore smelts into a bloom."));
    }
}

#[cfg(test)]
mod nettle {
    use super::{Attempt, Dir, Material, Role, World};

    fn intent(id: &str, actor: u32, attempt: Attempt) -> super::Intent {
        super::Intent {
            command_id: id.into(),
            actor_id: actor,
            attempt,
        }
    }

    fn events(world: &World, actor: u32) -> Vec<String> {
        world
            .observe(actor)
            .unwrap()
            .events
            .iter()
            .map(|event| event.text.clone())
            .collect()
    }

    #[test]
    fn nettles_east_of_the_trail_strip_into_cordage() {
        assert!(matches!(
            crate::parse::parse_attempt("strip nettle").unwrap(),
            Attempt::Gather(Material::Nettle)
        ));
        assert!(matches!(
            crate::parse::parse_attempt("gather nettle").unwrap(),
            Attempt::Gather(Material::Nettle)
        ));
        assert!(matches!(
            crate::parse::parse_attempt("strip willow").unwrap(),
            Attempt::Gather(Material::Willow)
        ));
        assert!(matches!(
            crate::parse::parse_attempt("gather cordage").unwrap(),
            Attempt::Gather(Material::Willow)
        ));
        let mut world = World::valley(crate::tuning::Tuning::evening(), 1);
        let stand = super::cell_id(6, 2).unwrap();
        assert_eq!(world.pile_count(stand, Material::Nettle), 2);
        let near = world.spawn_at("Near", "n", 4, 3).unwrap();
        let seen = world.spawn_at("Seen", "s", 3, 3).unwrap();
        let quiet = world.spawn_at("Quiet", "q", 5, 3).unwrap();
        let from_here = world.observe(near).unwrap().text;
        assert!(from_here.contains("Nettles grow to the"), "{from_here}");
        assert!(world
            .observe(quiet)
            .unwrap()
            .text
            .contains("Nettles grow to the"));
        assert!(!world
            .observe(seen)
            .unwrap()
            .text
            .to_lowercase()
            .contains("nettle"));

        let miss = world.step(&[intent("miss", near, Attempt::Gather(Material::Nettle))]);
        assert_eq!(
            miss.results[0].message,
            "There is no nettle here to gather."
        );
        assert!(events(&world, near).contains(&"There is no nettle here to gather.".to_string()));
        assert!(events(&world, seen)
            .iter()
            .all(|text| !text.contains("nettle")));
        assert!(events(&world, quiet)
            .iter()
            .all(|text| !text.contains("nettle")));

        let other = world.spawn_at("Other", "o", 0, 0).unwrap();
        world.mint_item(
            Material::Cord,
            0,
            world.actor(other).unwrap().cell,
            Some(other),
            Role::Loose,
        );
        world.mint_item(
            Material::Nettle,
            0,
            world.actor(other).unwrap().cell,
            Some(other),
            Role::Loose,
        );
        let prefer = world.step(&[intent("prefer", other, Attempt::SetSnare)]);
        assert_eq!(prefer.results[0].message, "You set a snare of hide strips.");
        assert!(world.loose_ids(other, Material::Cord).is_empty());
        assert_eq!(world.loose_ids(other, Material::Nettle).len(), 1);
        assert!(events(&world, other).iter().all(|text| {
            text != "You set a snare of hide strips."
                && text != "Other sets a snare of hide strips."
        }));

        for (id, dir) in [
            ("north", Dir::North),
            ("east", Dir::East),
            ("east2", Dir::East),
        ] {
            world.step(&[intent(id, near, Attempt::Move(dir))]);
        }
        assert_eq!(world.actor(near).unwrap().cell, stand);
        assert_eq!(world.hare_cell(), Some((2, 4)));
        let on_stand = world.observe(near).unwrap().text;
        assert!(
            on_stand.contains("Nettles grow here. They can be stripped for cordage."),
            "{on_stand}"
        );

        let strip = world.step(&[intent("strip", near, Attempt::Gather(Material::Nettle))]);
        let yours = "You gather a bundle of nettle cordage. The nettles sting.";
        let theirs = "Near gathers a bundle of nettle cordage. The nettles sting.";
        assert_eq!(strip.results[0].message, yours);
        assert!(events(&world, near)
            .iter()
            .all(|text| text != yours && text != theirs));
        assert!(events(&world, quiet).contains(&theirs.to_string()));
        assert!(events(&world, seen)
            .iter()
            .all(|text| !text.contains("nettle")));
        let stung = world.observe(near).unwrap();
        assert!(
            stung.text.contains("You are stung by nettles."),
            "{}",
            stung.text
        );
        assert!(stung.text.contains("Your skin stings."), "{}", stung.text);
        assert!(!stung.text.contains("bare skin"), "{}", stung.text);
        assert_eq!(stung.self_info.illness, "stung by nettles");
        let quiet_look = world.examine(quiet, super::ExamineTarget::Person("Near".into()));
        assert!(
            quiet_look.contains("They are stung by nettles."),
            "{quiet_look}"
        );
        assert!(quiet_look.contains("Their skin stings."), "{quiet_look}");
        assert!(!quiet_look.contains("bare skin"), "{quiet_look}");
        assert_eq!(
            world.examine(seen, super::ExamineTarget::Person("Near".into())),
            "You cannot see Near from here."
        );
        assert!(world
            .observe(near)
            .unwrap()
            .text
            .contains("a bundle of nettle cordage"));
        assert_eq!(world.loose_ids(near, Material::Nettle).len(), 1);

        {
            let person = world.actors.get_mut(&near).unwrap();
            person.stings_until = None;
            person.nettle_sting = false;
            person.mud = world.tuning.smear_at;
        }
        let again = world.step(&[intent("again", near, Attempt::Gather(Material::Nettle))]);
        let calm = "You gather a bundle of nettle cordage.";
        let calm_seen = "Near gathers a bundle of nettle cordage.";
        assert_eq!(again.results[0].message, calm);
        assert!(events(&world, near)
            .iter()
            .all(|text| text != calm && text != calm_seen));
        assert!(events(&world, quiet).iter().any(|text| text == calm_seen));
        assert!(events(&world, seen)
            .iter()
            .all(|text| !text.contains("gathers")));
        assert!(world.actor(near).unwrap().stings_until.is_none());
        assert!(world
            .observe(near)
            .unwrap()
            .text
            .contains("The nettles here are bare."));
        assert!(world
            .observe(near)
            .unwrap()
            .text
            .contains("two bundles of nettle cordage"));
        assert_eq!(world.loose_ids(near, Material::Nettle).len(), 2);

        world.mint_item(Material::Flake, 0, stand, Some(near), Role::Loose);
        world.mint_item(Material::Branch, 0, stand, Some(near), Role::Loose);
        let haft = world.step(&[intent("haft", near, Attempt::Haft)]);
        let bound = "You haft the flint flake and a dry branch into a spear. The tip is bound with nettle cordage.";
        let bound_seen = "Near hafts the flint flake and a dry branch into a spear. The tip is bound with nettle cordage.";
        assert_eq!(haft.results[0].message, bound);
        assert!(events(&world, near)
            .iter()
            .all(|text| text != bound && text != bound_seen));
        assert!(events(&world, quiet).contains(&bound_seen.to_string()));
        assert!(events(&world, seen)
            .iter()
            .all(|text| !text.contains("haft")));
        assert!(world
            .observe(near)
            .unwrap()
            .text
            .contains("bound with nettle cordage"));
        assert_eq!(world.loose_ids(near, Material::Nettle).len(), 1);

        let snare = world.step(&[intent("snare", near, Attempt::SetSnare)]);
        let set = "You set a snare of nettle cordage.";
        let set_seen = "Near sets a snare of nettle cordage.";
        assert_eq!(snare.results[0].message, set);
        assert!(events(&world, near)
            .iter()
            .all(|text| text != set && text != set_seen));
        assert!(events(&world, quiet).contains(&set_seen.to_string()));
        assert!(events(&world, seen)
            .iter()
            .all(|text| !text.contains("nettle")));
        assert!(world.loose_ids(near, Material::Nettle).is_empty());
        assert!(world
            .observe(near)
            .unwrap()
            .text
            .contains("A nettle snare is set here."));
        assert!(!world
            .observe(seen)
            .unwrap()
            .text
            .to_lowercase()
            .contains("nettle"));
    }
}

#[cfg(test)]
mod owl {
    use super::World;

    #[test]
    fn an_owl_hoots_on_some_night_ticks_and_not_as_an_event() {
        let mut world = World::valley(crate::tuning::Tuning::evening(), 1);
        let near = world.spawn_at("Near", "n", 4, 3).unwrap();
        let seen = world.spawn_at("Seen", "s", 3, 3).unwrap();
        let quiet = world.spawn_at("Quiet", "q", 5, 3).unwrap();
        let rock = world.spawn_at("Rock", "r", 6, 1).unwrap();
        let thatch = world.spawn_at("Thatch", "t", 5, 2).unwrap();
        world.thatched.insert(world.actor(thatch).unwrap().cell);
        let far = world.spawn_at("Far", "f", 0, 4).unwrap();
        let slope = world.spawn_at("Slope", "p", 4, 1).unwrap();
        let roost = world.spawn_at("Roost", "o", 4, 0).unwrap();
        assert!(!world.observe(near).unwrap().text.contains("owl"));
        while world.sim_time() < world.tuning.night_at {
            world.step(&[]);
        }
        assert!(world.night());
        assert_eq!(world.sim_time() % 3, 0);
        let open = world.observe(near).unwrap();
        assert!(
            open.text.contains("You hear an owl hoot to the north."),
            "{}",
            open.text
        );
        assert!(world
            .observe(seen)
            .unwrap()
            .text
            .contains("You hear an owl hoot to the north and east."));
        assert!(world
            .observe(quiet)
            .unwrap()
            .text
            .contains("You hear an owl hoot to the north and west."));
        let sheltered = world.observe(rock).unwrap().text;
        assert!(
            sheltered.contains("You hear an owl, muffled."),
            "{sheltered}"
        );
        assert!(!sheltered.contains("hoot to the"), "{sheltered}");
        let under = world.observe(thatch).unwrap().text;
        assert!(under.contains("You hear an owl, muffled."), "{under}");
        assert!(!world.observe(far).unwrap().text.contains("owl"));
        assert!(world.observe(slope).unwrap().text.contains("An owl hoots."));
        assert!(!world.observe(slope).unwrap().text.contains("hoot to the"));
        assert!(world
            .observe(roost)
            .unwrap()
            .text
            .contains("An owl hoots here."));
        for person in [near, seen, quiet, rock, thatch, far, slope, roost] {
            assert!(world
                .observe(person)
                .unwrap()
                .events
                .iter()
                .all(|event| !event.text.contains("hoot") && !event.text.contains("an owl")));
        }
        world.step(&[]);
        assert!(world.night());
        assert!(!world.observe(near).unwrap().text.contains("owl"));
        assert!(!world.observe(rock).unwrap().text.contains("owl"));
    }
}

#[cfg(test)]
mod frogs {
    use super::World;

    #[test]
    fn frogs_croak_at_dusk_near_water_and_not_as_an_event() {
        let mut world = World::valley(crate::tuning::Tuning::evening(), 1);
        let near = world.spawn_at("Near", "n", 4, 3).unwrap();
        let seen = world.spawn_at("Seen", "s", 3, 3).unwrap();
        let quiet = world.spawn_at("Quiet", "q", 5, 3).unwrap();
        let still = world.spawn_at("Still", "t", 2, 1).unwrap();
        let brook = world.spawn_at("Brook", "b", 0, 2).unwrap();
        let far = world.spawn_at("Far", "f", 6, 4).unwrap();
        assert!(!world.night());
        assert!(!world.morning());
        for person in [near, seen, quiet, still, brook, far] {
            let look = world.observe(person).unwrap();
            assert!(!look.text.to_lowercase().contains("frog"), "{}", look.text);
            assert!(!look.text.contains("croak"), "{}", look.text);
        }
        world.step(&[]);
        assert_eq!(world.sim_time(), 1);
        let open = world.observe(near).unwrap();
        assert!(open.text.contains("Frogs croak."), "{}", open.text);
        assert!(!open.text.contains("croak to the"), "{}", open.text);
        assert!(world.observe(seen).unwrap().text.contains("Frogs croak."));
        assert!(!world.observe(seen).unwrap().text.contains("croak to the"));
        let heard = world.observe(quiet).unwrap().text;
        assert!(
            heard.contains("You hear frogs croak to the north and west."),
            "{heard}"
        );
        assert!(world
            .observe(still)
            .unwrap()
            .text
            .contains("Frogs croak here."));
        assert!(world
            .observe(brook)
            .unwrap()
            .text
            .contains("Frogs croak here."));
        assert!(!world
            .observe(far)
            .unwrap()
            .text
            .to_lowercase()
            .contains("frog"));
        for person in [near, seen, quiet, still, brook, far] {
            assert!(world.observe(person).unwrap().events.iter().all(|event| {
                !event.text.to_lowercase().contains("frog") && !event.text.contains("croak")
            }));
        }
        world.step(&[]);
        assert_eq!(world.sim_time(), 2);
        assert!(!world
            .observe(near)
            .unwrap()
            .text
            .to_lowercase()
            .contains("frog"));
        assert!(!world.observe(quiet).unwrap().text.contains("croak"));
        while world.sim_time() < 73 {
            world.step(&[]);
        }
        assert!(world.night());
        assert_eq!(world.sim_time() % 3, 1);
        assert!(!world
            .observe(still)
            .unwrap()
            .text
            .to_lowercase()
            .contains("frog"));
        assert!(!world.observe(quiet).unwrap().text.contains("croak"));
        while !world.morning() {
            world.step(&[]);
        }
        while world.sim_time() % 3 != 1 {
            world.step(&[]);
        }
        assert!(world.morning());
        assert!(!world
            .observe(still)
            .unwrap()
            .text
            .to_lowercase()
            .contains("frog"));
        assert!(!world.observe(near).unwrap().text.contains("croak"));
    }
}

#[cfg(test)]
mod birdsong {
    use super::World;

    #[test]
    fn birds_sing_on_some_morning_ticks_and_not_as_an_event() {
        let mut world = World::valley(crate::tuning::Tuning::evening(), 1);
        let near = world.spawn_at("Near", "n", 4, 3).unwrap();
        let seen = world.spawn_at("Seen", "s", 3, 3).unwrap();
        let quiet = world.spawn_at("Quiet", "q", 6, 1).unwrap();
        let thatch = world.spawn_at("Thatch", "t", 5, 2).unwrap();
        world.thatched.insert(world.actor(thatch).unwrap().cell);
        let bare = world.spawn_at("Bare", "b", 4, 2).unwrap();
        world.lean_tos.insert(world.actor(bare).unwrap().cell);
        let far = world.spawn_at("Far", "f", 0, 4).unwrap();
        for person in [near, seen, quiet, thatch, bare, far] {
            let look = world.observe(person).unwrap();
            assert!(!look.text.contains("birdsong"), "{}", look.text);
            assert!(look
                .events
                .iter()
                .all(|event| !event.text.contains("birdsong")));
        }
        world.step(&[]);
        assert_eq!(world.sim_time(), 1);
        assert!(!world.observe(near).unwrap().text.contains("birdsong"));
        while world.sim_time() < world.tuning.night_at {
            world.step(&[]);
        }
        assert!(world.night());
        assert_eq!(world.sim_time() % 3, 0);
        let night = world.observe(near).unwrap().text;
        assert!(
            night.contains("You hear an owl hoot to the north."),
            "{night}"
        );
        assert!(!night.contains("birdsong"), "{night}");
        assert!(!world.observe(quiet).unwrap().text.contains("birdsong"));
        while !world.morning() {
            world.step(&[]);
        }
        assert!(world.morning());
        assert!(world.raining);
        let wet = world.observe(near).unwrap().text;
        assert!(wet.contains("Rain is falling"), "{wet}");
        assert!(!wet.contains("birdsong"), "{wet}");
        assert!(!world.observe(quiet).unwrap().text.contains("birdsong"));
        assert!(!world.observe(thatch).unwrap().text.contains("birdsong"));
        world.step(&[]);
        assert!(world.morning());
        assert!(!world.raining);
        let open = world.observe(near).unwrap().text;
        assert!(open.contains("The sky is clear."), "{open}");
        assert!(!open.contains("Rain is falling"), "{open}");
        assert!(open.contains("You hear birdsong."), "{open}");
        assert!(!open.contains("faint"), "{open}");
        assert!(!open.contains("owl"), "{open}");
        assert!(!open.contains("croak"), "{open}");
        assert!(!open.contains("wind rush"), "{open}");
        assert!(world
            .observe(seen)
            .unwrap()
            .text
            .contains("You hear birdsong."));
        assert!(world
            .observe(far)
            .unwrap()
            .text
            .contains("You hear birdsong."));
        assert!(world
            .observe(bare)
            .unwrap()
            .text
            .contains("You hear birdsong."));
        assert!(!world.observe(bare).unwrap().text.contains("faint"));
        let under = world.observe(quiet).unwrap().text;
        assert!(under.contains("You hear birdsong, faint."), "{under}");
        assert!(!under.contains("You hear birdsong."), "{under}");
        let roof = world.observe(thatch).unwrap().text;
        assert!(roof.contains("You hear birdsong, faint."), "{roof}");
        assert!(!roof.contains("muffled"), "{roof}");
        for person in [near, seen, quiet, thatch, bare, far] {
            assert!(world
                .observe(person)
                .unwrap()
                .events
                .iter()
                .all(|event| !event.text.contains("birdsong")));
        }
        world.step(&[]);
        assert!(world.morning());
        assert!(!world.raining);
        let quiet_morning = world.observe(near).unwrap().text;
        assert!(
            quiet_morning.contains("The sky is clear."),
            "{quiet_morning}"
        );
        assert!(!quiet_morning.contains("birdsong"), "{quiet_morning}");
        assert!(!world.observe(quiet).unwrap().text.contains("birdsong"));
        assert!(!world.observe(thatch).unwrap().text.contains("birdsong"));
    }
}

#[cfg(test)]
mod willow_bark {
    use super::{Attempt, ExamineTarget, Material, Role, World};

    fn intent(id: &str, actor: u32, attempt: Attempt) -> super::Intent {
        super::Intent {
            command_id: id.into(),
            actor_id: actor,
            attempt,
        }
    }

    #[test]
    fn willow_bark_is_not_cordage_and_the_tea_eases_a_sunburn() {
        assert!(matches!(
            crate::parse::parse_attempt("strip willow bark").unwrap(),
            Attempt::Gather(Material::WillowBark)
        ));
        assert!(matches!(
            crate::parse::parse_attempt("gather willow bark").unwrap(),
            Attempt::Gather(Material::WillowBark)
        ));
        assert!(matches!(
            crate::parse::parse_attempt("strip willow").unwrap(),
            Attempt::Gather(Material::Willow)
        ));
        assert!(matches!(
            crate::parse::parse_attempt("steep willow").unwrap(),
            Attempt::SteepWillow
        ));
        assert!(matches!(
            crate::parse::parse_attempt("brew willow bark tea").unwrap(),
            Attempt::SteepWillow
        ));

        let mut world = World::valley(crate::tuning::Tuning::evening(), 1);
        let stand = super::cell_id(0, 4).unwrap();
        let near = world.spawn_at("Near", "n", 0, 4).unwrap();
        let seen = world.spawn_at("Seen", "s", 1, 4).unwrap();
        let quiet = world.spawn_at("Quiet", "q", 5, 3).unwrap();
        let cordage = world.pile_count(stand, Material::Willow);
        assert!(cordage >= 2);
        let here = world.observe(near).unwrap().text;
        assert!(
            here.contains(
                "A strip of willow bark hangs on the willow here. It is not the cordage."
            ),
            "{here}"
        );
        assert!(
            here.contains("Bundles of willow cordage hang here. They are withies from a willow."),
            "{here}"
        );
        let named = world.examine(near, ExamineTarget::WillowBark);
        assert!(named.contains("It is not the cordage."), "{named}");
        assert!(!named.contains("withies"), "{named}");
        let cord = world.examine(near, ExamineTarget::Willow);
        assert!(cord.contains("withies from a willow"), "{cord}");
        assert!(!cord.contains("willow bark"), "{cord}");
        let distant = world.observe(seen).unwrap().text;
        assert!(
            distant.contains(
                "A strip of willow bark hangs on the willow to the west. It is not the cordage."
            ),
            "{distant}"
        );
        assert!(!world.observe(quiet).unwrap().text.contains("willow bark"));

        let gathered = world.step(&[intent("strip", near, Attempt::Gather(Material::WillowBark))]);
        assert_eq!(
            gathered.results[0].message,
            "You gather a strip of willow bark."
        );
        assert_eq!(world.pile_count(stand, Material::Willow), cordage);
        let taken = world.observe(near).unwrap();
        assert!(
            taken.text.contains("The willow here has no bark."),
            "{}",
            taken.text
        );
        assert!(taken.text.contains("willow cordage"), "{}", taken.text);
        assert!(
            taken.text.contains("a strip of willow bark"),
            "{}",
            taken.text
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
            .any(|event| { event.text == "Near gathers a strip of willow bark." }));
        assert!(world
            .observe(quiet)
            .unwrap()
            .events
            .iter()
            .all(|event| !event.text.contains("gathers") && !event.text.contains("willow bark")));

        let open = super::cell_id(4, 4).unwrap();
        world.actors.get_mut(&near).unwrap().cell = open;
        world.raining = true;
        world.mint_fire(open, 0);
        world.mint_item(Material::Pot, 0, open, Some(near), Role::Loose);
        world.mint_item(Material::Water, 0, open, Some(near), Role::Loose);
        let illness = world.tuning.ill_at;
        let cold = world.tuning.chilly_at;
        let sun = world.tuning.burned_at;
        {
            let person = world.actors.get_mut(&near).unwrap();
            person.illness = illness;
            person.cold = cold;
            person.sunburn = sun;
        }
        let steeped = world.step(&[intent("tea", near, Attempt::SteepWillow)]);
        assert_eq!(
            steeped.results[0].message,
            "You steep a strip of willow bark in the pot over the fire and drink the willow bark tea. The sunburn eases."
        );
        assert!(!steeped.results[0].message.contains("stomach"));
        assert!(!steeped.results[0].message.contains("chill"));
        assert_eq!(
            world.actor(near).unwrap().illness,
            illness - world.tuning.illness_recovery
        );
        assert_eq!(
            world.actor(near).unwrap().sunburn,
            sun - world.tuning.sun_per_tick * 2
        );
        assert!(world.loose_ids(near, Material::WillowBark).is_empty());
        let heard =
            "Near steeps a strip of willow bark in the pot over the fire and drinks the willow bark tea.";
        assert!(world
            .observe(near)
            .unwrap()
            .events
            .iter()
            .all(|event| event.text != heard && !event.text.contains("sunburn")));
        let witness = world.spawn_at("Ash", "a", 4, 2).unwrap();
        assert!(world
            .observe(witness)
            .unwrap()
            .events
            .iter()
            .any(|event| event.text == heard));
        assert!(world
            .observe(witness)
            .unwrap()
            .events
            .iter()
            .all(|event| { !event.text.contains("sunburn") && !event.text.contains("stomach") }));

        world.actors.get_mut(&near).unwrap().sunburn = 0;
        world.mint_item(Material::WillowBark, 0, open, Some(near), Role::Loose);
        world.mint_item(Material::Water, 0, open, Some(near), Role::Loose);
        let plain = world.step(&[intent("plain", near, Attempt::SteepWillow)]);
        assert_eq!(
            plain.results[0].message,
            "You steep a strip of willow bark in the pot over the fire and drink the willow bark tea."
        );
        assert!(!plain.results[0].message.contains("sunburn"));

        let missing = world.step(&[intent("none", near, Attempt::SteepWillow)]);
        assert_eq!(
            missing.results[0].message,
            "You have no willow bark to steep."
        );
        assert!(world
            .observe(near)
            .unwrap()
            .events
            .iter()
            .any(|event| { event.text == "You have no willow bark to steep." }));
        assert!(world
            .observe(witness)
            .unwrap()
            .events
            .iter()
            .all(|event| { event.text != "You have no willow bark to steep." }));
    }

    #[test]
    fn a_stripped_willow_grows_bark_again_and_the_cordage_stays() {
        let mut world = World::valley(crate::tuning::Tuning::evening(), 1);
        let stand = super::cell_id(0, 4).unwrap();
        let near = world.spawn_at("Near", "n", 0, 4).unwrap();
        let seen = world.spawn_at("Seen", "s", 1, 4).unwrap();
        let quiet = world.spawn_at("Quiet", "q", 5, 3).unwrap();
        let cordage = world.pile_count(stand, Material::Willow);
        let none = world.step(&[intent("none", quiet, Attempt::Gather(Material::WillowBark))]);
        assert_eq!(
            none.results[0].message,
            "There is no willow bark here to gather."
        );
        assert!(world
            .observe(quiet)
            .unwrap()
            .events
            .iter()
            .any(|event| { event.text == "There is no willow bark here to gather." }));
        assert!(world
            .observe(near)
            .unwrap()
            .events
            .iter()
            .all(|event| { event.text != "There is no willow bark here to gather." }));
        let gathered = world.step(&[intent("strip", near, Attempt::Gather(Material::WillowBark))]);
        assert_eq!(
            gathered.results[0].message,
            "You gather a strip of willow bark."
        );
        assert!(world
            .observe(near)
            .unwrap()
            .events
            .iter()
            .all(|event| !event.text.contains("gathers")));
        assert_eq!(world.pile_count(stand, Material::Willow), cordage);
        let bare = world.observe(near).unwrap().text;
        assert!(bare.contains("The willow here has no bark."), "{bare}");
        assert!(
            bare.contains("Bundles of willow cordage hang here."),
            "{bare}"
        );
        let ticks = world.tuning.moss_ticks;
        for _ in 0..ticks - 1 {
            world.step(&[]);
        }
        assert!(world
            .observe(near)
            .unwrap()
            .text
            .contains("The willow here has no bark."));
        assert!(!world
            .observe(near)
            .unwrap()
            .events
            .iter()
            .any(|event| event.text.contains("grows on the willow again")));
        world.step(&[]);
        let grown = "Willow bark grows on the willow again.";
        let back = world.observe(near).unwrap();
        assert!(
            back.text.contains(
                "A strip of willow bark hangs on the willow here. It is not the cordage."
            ),
            "{}",
            back.text
        );
        assert!(!back.text.contains("The willow here has no bark."));
        assert!(back.text.contains("Bundles of willow cordage hang here."));
        assert!(back.events.iter().any(|event| event.text == grown));
        assert!(world
            .observe(seen)
            .unwrap()
            .events
            .iter()
            .any(|event| event.text == grown));
        assert!(world
            .observe(seen)
            .unwrap()
            .text
            .contains("A strip of willow bark hangs on the willow to the west."));
        assert!(world
            .observe(quiet)
            .unwrap()
            .events
            .iter()
            .all(|event| event.text != grown));
        assert!(!world.observe(quiet).unwrap().text.contains("willow bark"));
        assert_eq!(world.pile_count(stand, Material::Willow), cordage);
        assert!(!world
            .observe(near)
            .unwrap()
            .events
            .iter()
            .any(|event| event.text.contains("cordage hangs on the willow again")));
        let again = world.step(&[intent("again", near, Attempt::Gather(Material::WillowBark))]);
        assert_eq!(
            again.results[0].message,
            "You gather a strip of willow bark."
        );
        assert!(world
            .observe(near)
            .unwrap()
            .text
            .contains("The willow here has no bark."));
    }
}

#[cfg(test)]
mod cattail {
    use super::{Attempt, Dir, Material, World};

    fn intent(id: &str, actor: u32, attempt: Attempt) -> super::Intent {
        super::Intent {
            command_id: id.into(),
            actor_id: actor,
            attempt,
        }
    }

    fn go(world: &mut World, id: &str, actor: u32, dir: Dir) {
        let moved = world.step(&[intent(id, actor, Attempt::Move(dir))]);
        assert!(
            moved.results[0].message.contains("You walk"),
            "{}",
            moved.results[0].message
        );
    }

    #[test]
    fn cattail_roots_are_not_reeds_and_need_cooking() {
        let mut world = World::valley(crate::tuning::Tuning::evening(), 1);
        let near = world.spawn_at("Near", "n", 4, 3).unwrap();
        let seen = world.spawn_at("Seen", "s", 3, 3).unwrap();
        let quiet = world.spawn_at("Quiet", "q", 5, 3).unwrap();
        let reeds = world.spawn_at("Reed", "r", 0, 3).unwrap();
        assert_eq!(
            crate::parse::parse_attempt("gather cattail").unwrap(),
            Attempt::Gather(Material::Cattail)
        );
        assert_eq!(
            crate::parse::parse_attempt("dig roots").unwrap(),
            Attempt::Gather(Material::Cattail)
        );
        assert_eq!(
            crate::parse::parse_attempt("gather reeds").unwrap(),
            Attempt::Gather(Material::Reed)
        );
        let reed_look = world.observe(reeds).unwrap().text;
        assert!(reed_look.contains("Reeds grow here"), "{reed_look}");
        assert!(!reed_look.contains("Cattails grow here"), "{reed_look}");
        let before = world.material_tally(Material::Reed);
        let missed = world.step(&[intent(
            "dig-reeds",
            reeds,
            Attempt::Gather(Material::Cattail),
        )]);
        assert_eq!(
            missed.results[0].message,
            "There is no cattail here to gather."
        );
        assert_eq!(world.material_tally(Material::Reed), before);
        assert!(world
            .observe(reeds)
            .unwrap()
            .events
            .iter()
            .any(|event| { event.text == "There is no cattail here to gather." }));
        assert!(world
            .observe(near)
            .unwrap()
            .events
            .iter()
            .all(|event| event.text != "There is no cattail here to gather."));

        let trail = world.observe(near).unwrap().text;
        assert!(!trail.contains("Cattails grow here"), "{trail}");
        let miss = world.step(&[intent("miss", near, Attempt::Gather(Material::Cattail))]);
        assert_eq!(
            miss.results[0].message,
            "There is no cattail here to gather."
        );
        assert!(world
            .observe(near)
            .unwrap()
            .events
            .iter()
            .any(|event| { event.text == "There is no cattail here to gather." }));
        for person in [seen, quiet] {
            assert!(world
                .observe(person)
                .unwrap()
                .events
                .iter()
                .all(|event| { event.text != "There is no cattail here to gather." }));
        }

        go(&mut world, "n1", near, Dir::North);
        go(&mut world, "w1", near, Dir::West);
        go(&mut world, "w2", near, Dir::West);
        go(&mut world, "n2", near, Dir::North);
        let here = world.observe(near).unwrap().text;
        assert!(
            here.contains(
                "Cattails grow here, in the shallows. The roots can be dug. They are not the reeds."
            ),
            "{here}"
        );
        assert!(!here.contains("Reeds grow here"), "{here}");
        assert!(world.observe(seen).unwrap().text.contains(
            "Cattails grow in the shallows to the north and west. They are not the reeds."
        ));
        assert!(!world.observe(quiet).unwrap().text.contains("cattail"));

        let dug = world.step(&[intent("dig", near, Attempt::Gather(Material::Cattail))]);
        assert_eq!(dug.results[0].message, "You gather cattail roots.");
        let near_events = world.observe(near).unwrap();
        assert!(near_events
            .events
            .iter()
            .all(|event| !event.text.contains("gathers cattail roots")));
        assert!(world
            .observe(seen)
            .unwrap()
            .events
            .iter()
            .any(|event| event.text == "Near gathers cattail roots."));
        assert!(world
            .observe(quiet)
            .unwrap()
            .events
            .iter()
            .all(|event| !event.text.contains("cattail")));
        assert!(world.observe(near).unwrap().text.contains("cattail roots"));

        let raw = world.step(&[intent("raw", near, Attempt::Eat(Material::Cattail))]);
        assert_eq!(
            raw.results[0].message,
            "Raw cattail roots do not feed you. They must be cooked."
        );
        assert!(world.observe(near).unwrap().events.iter().any(|event| {
            event.text == "Raw cattail roots do not feed you. They must be cooked."
        }));
        for person in [seen, quiet] {
            assert!(world.observe(person).unwrap().events.iter().all(|event| {
                event.text != "Raw cattail roots do not feed you. They must be cooked."
            }));
        }

        for (id, dir) in [
            ("e1", Dir::East),
            ("e2", Dir::East),
            ("e3", Dir::East),
            ("e4", Dir::East),
        ] {
            go(&mut world, id, near, dir);
        }
        let lit = world.step(&[intent("light", near, Attempt::Light)]);
        assert!(
            lit.results[0].message.contains("You light a fire"),
            "{}",
            lit.results[0].message
        );
        let cooked = world.step(&[intent("cook", near, Attempt::Cook)]);
        assert_eq!(
            cooked.results[0].message,
            "You cook the cattail roots over the fire."
        );
        assert!(world
            .observe(near)
            .unwrap()
            .events
            .iter()
            .all(|event| !event.text.contains("cooks the cattail roots")));
        assert!(world
            .observe(quiet)
            .unwrap()
            .events
            .iter()
            .any(|event| { event.text == "Near cooks the cattail roots over the fire." }));
        assert!(world
            .observe(seen)
            .unwrap()
            .events
            .iter()
            .all(|event| !event.text.contains("cooks the cattail roots")));
        assert!(world
            .observe(near)
            .unwrap()
            .text
            .contains("cooked cattail roots"));

        world.actors.get_mut(&near).unwrap().hunger = world.tuning.hungry_at;
        let eaten = world.step(&[intent("eat", near, Attempt::Eat(Material::Cattail))]);
        assert_eq!(
            eaten.results[0].message,
            "You eat the cooked cattail roots. The hunger eases."
        );
        assert!(world
            .observe(near)
            .unwrap()
            .events
            .iter()
            .all(|event| !event.text.contains("eats the cooked cattail roots")));
        assert!(world
            .observe(near)
            .unwrap()
            .events
            .iter()
            .any(|event| event.text == "The hunger eases."));
        assert!(world
            .observe(quiet)
            .unwrap()
            .events
            .iter()
            .any(|event| event.text == "Near eats the cooked cattail roots."));
        assert!(world
            .observe(quiet)
            .unwrap()
            .events
            .iter()
            .all(|event| event.text != "The hunger eases."));
        assert!(world
            .observe(seen)
            .unwrap()
            .events
            .iter()
            .all(|event| !event.text.contains("eats the cooked cattail roots")));
    }

    #[test]
    fn dug_out_cattails_grow_again_after_twenty_four_ticks() {
        let mut world = World::valley(crate::tuning::Tuning::evening(), 1);
        let near = world.spawn_at("Near", "n", 2, 1).unwrap();
        let seen = world.spawn_at("Seen", "s", 3, 3).unwrap();
        let quiet = world.spawn_at("Quiet", "q", 5, 3).unwrap();
        assert_eq!(world.tuning.cattail_ticks, 24);
        let one = world.step(&[intent("one", near, Attempt::Gather(Material::Cattail))]);
        assert_eq!(one.results[0].message, "You gather cattail roots.");
        assert!(world.cattail_returns_at.is_empty());
        assert!(world.observe(near).unwrap().text.contains(
            "Cattails grow here, in the shallows. The roots can be dug. They are not the reeds."
        ));
        let two = world.step(&[intent("two", near, Attempt::Gather(Material::Cattail))]);
        assert_eq!(two.results[0].message, "You gather cattail roots.");
        assert!(world
            .observe(near)
            .unwrap()
            .events
            .iter()
            .all(|event| { !event.text.contains("gathers cattail roots") }));
        assert!(world
            .observe(seen)
            .unwrap()
            .events
            .iter()
            .any(|event| event.text == "Near gathers cattail roots."));
        let bare = world.observe(near).unwrap().text;
        assert!(
            bare.contains("The shallows here have no cattails."),
            "{bare}"
        );
        assert!(!bare.contains("Cattails grow here"), "{bare}");
        assert!(world
            .observe(seen)
            .unwrap()
            .text
            .contains("The shallows to the north and west have no cattails."));
        assert!(!world.observe(quiet).unwrap().text.contains("cattail"));
        let missed = world.step(&[intent("none", near, Attempt::Gather(Material::Cattail))]);
        assert_eq!(
            missed.results[0].message,
            "There is no cattail here to gather."
        );
        assert!(world
            .observe(near)
            .unwrap()
            .events
            .iter()
            .any(|event| { event.text == "There is no cattail here to gather." }));
        for person in [seen, quiet] {
            assert!(world
                .observe(person)
                .unwrap()
                .events
                .iter()
                .all(|event| { event.text != "There is no cattail here to gather." }));
        }
        let ticks = world.tuning.cattail_ticks;
        // The bare gather above was one of the twenty-four ticks.
        for _ in 0..ticks - 2 {
            world.step(&[]);
        }
        assert!(world
            .observe(near)
            .unwrap()
            .text
            .contains("The shallows here have no cattails."));
        assert!(world
            .observe(near)
            .unwrap()
            .events
            .iter()
            .all(|event| { event.text != "Cattails grow again in the shallows." }));
        world.step(&[]);
        let grown = "Cattails grow again in the shallows.";
        let held = world.observe(near).unwrap();
        assert!(
            held.text
                .contains("The still water is frozen. Ice covers it."),
            "{}",
            held.text
        );
        assert!(
            held.text.contains("The shallows here have no cattails."),
            "{}",
            held.text
        );
        assert!(!held.text.contains("Cattails grow here"), "{}", held.text);
        assert!(held.events.iter().all(|event| event.text != grown));
        assert!(world
            .observe(seen)
            .unwrap()
            .events
            .iter()
            .all(|event| event.text != grown));
        let thaw = world.tuning.night_at + world.tuning.night_len;
        while world.time < thaw {
            world.step(&[]);
            assert!(world
                .observe(near)
                .unwrap()
                .events
                .iter()
                .all(|event| event.text != grown));
            assert!(world
                .observe(near)
                .unwrap()
                .text
                .contains("The shallows here have no cattails."));
        }
        world.step(&[]);
        let back = world.observe(near).unwrap();
        assert!(
            back.text.contains("The day has thawed the still water."),
            "{}",
            back.text
        );
        assert!(!back.text.contains("Ice covers it."), "{}", back.text);
        assert!(
            back.text.contains(
                "Cattails grow here, in the shallows. The roots can be dug. They are not the reeds."
            ),
            "{}",
            back.text
        );
        assert!(!back.text.contains("The shallows here have no cattails."));
        assert!(back.events.iter().any(|event| event.text == grown));
        let west = world.observe(seen).unwrap();
        assert!(west.text.contains(
            "Cattails grow in the shallows to the north and west. They are not the reeds."
        ));
        assert!(west.events.iter().any(|event| event.text == grown));
        assert!(world
            .observe(quiet)
            .unwrap()
            .events
            .iter()
            .all(|event| event.text != grown));
        assert!(!world.observe(quiet).unwrap().text.contains("cattail"));
        let again = world.step(&[intent("again", near, Attempt::Gather(Material::Cattail))]);
        assert_eq!(again.results[0].message, "You gather cattail roots.");
    }

    #[test]
    fn ice_covers_cattail_roots_until_the_still_thaws() {
        let mut world = World::valley(crate::tuning::Tuning::evening(), 1);
        let near = world.spawn_at("Near", "n", 2, 1).unwrap();
        let seen = world.spawn_at("Seen", "s", 3, 3).unwrap();
        let quiet = world.spawn_at("Quiet", "q", 5, 3).unwrap();
        let open = world.observe(near).unwrap().text;
        assert!(open.contains(
            "Cattails grow here, in the shallows. The roots can be dug. They are not the reeds."
        ));
        assert!(!open.contains("Ice covers"));
        while world.time < world.tuning.chill_at {
            world.step(&[]);
        }
        let iced = world.observe(near).unwrap().text;
        assert!(
            iced.contains("The still water is frozen. Ice covers it."),
            "{iced}"
        );
        assert!(
            iced.contains(
                "Cattails stand under the ice. The roots cannot be dug. They are not the reeds."
            ),
            "{iced}"
        );
        assert!(!iced.contains("The roots can be dug."), "{iced}");
        assert!(world.observe(seen).unwrap().text.contains(
            "Cattails stand under the ice to the north and west. They are not the reeds."
        ));
        assert!(!world.observe(quiet).unwrap().text.contains("cattail"));
        let dug = world.step(&[intent("ice", near, Attempt::Gather(Material::Cattail))]);
        assert_eq!(dug.results[0].message, "Ice covers the cattail roots.");
        assert!(world
            .observe(near)
            .unwrap()
            .events
            .iter()
            .any(|event| event.text == "Ice covers the cattail roots."));
        for person in [seen, quiet] {
            assert!(world
                .observe(person)
                .unwrap()
                .events
                .iter()
                .all(|event| { event.text != "Ice covers the cattail roots." }));
        }
        assert!(world.pile_count(world.actor(near).unwrap().cell, Material::Cattail) > 0);
    }

    #[test]
    fn cattail_fluff_is_not_the_roots_and_it_catches_in_the_rain() {
        assert_eq!(
            crate::parse::parse_attempt("gather fluff").unwrap(),
            Attempt::Gather(Material::Fluff)
        );
        assert_eq!(
            crate::parse::parse_attempt("gather cattail fluff").unwrap(),
            Attempt::Gather(Material::Fluff)
        );
        assert_eq!(
            crate::parse::parse_attempt("gather cattail").unwrap(),
            Attempt::Gather(Material::Cattail)
        );
        assert_eq!(
            crate::parse::parse_attempt("dig roots").unwrap(),
            Attempt::Gather(Material::Cattail)
        );
        let mut world = World::valley(crate::tuning::Tuning::evening(), 1);
        let stand = super::cell_id(2, 1).unwrap();
        let near = world.spawn_at("Near", "n", 2, 1).unwrap();
        let seen = world.spawn_at("Seen", "s", 2, 2).unwrap();
        let quiet = world.spawn_at("Quiet", "q", 5, 3).unwrap();
        let roots = world.pile_count(stand, Material::Cattail);
        let here = world.observe(near).unwrap().text;
        assert!(
            here.contains(
                "Cattails grow here, in the shallows. The roots can be dug. They are not the reeds."
            ),
            "{here}"
        );
        assert!(
            here.contains("Cattail fluff hangs on them. It is not the roots."),
            "{here}"
        );
        assert!(world
            .observe(seen)
            .unwrap()
            .text
            .contains("Cattail fluff hangs on the cattails to the north. It is not the roots."));
        assert!(!world
            .observe(quiet)
            .unwrap()
            .text
            .to_lowercase()
            .contains("fluff"));
        let none = world.step(&[intent("none", quiet, Attempt::Gather(Material::Fluff))]);
        assert_eq!(
            none.results[0].message,
            "There is no cattail fluff here to gather."
        );
        assert!(world
            .observe(quiet)
            .unwrap()
            .events
            .iter()
            .any(|event| { event.text == "There is no cattail fluff here to gather." }));
        assert!(world
            .observe(near)
            .unwrap()
            .events
            .iter()
            .all(|event| { event.text != "There is no cattail fluff here to gather." }));
        let gathered = world.step(&[intent("fluff", near, Attempt::Gather(Material::Fluff))]);
        assert_eq!(
            gathered.results[0].message,
            "You gather a handful of cattail fluff."
        );
        assert_eq!(world.pile_count(stand, Material::Cattail), roots);
        assert!(world
            .observe(near)
            .unwrap()
            .events
            .iter()
            .all(|event| !event.text.contains("gathers")));
        assert!(world
            .observe(seen)
            .unwrap()
            .events
            .iter()
            .any(|event| { event.text == "Near gathers a handful of cattail fluff." }));
        assert!(world
            .observe(quiet)
            .unwrap()
            .events
            .iter()
            .all(|event| { event.text != "Near gathers a handful of cattail fluff." }));
        let bare = world.observe(near).unwrap().text;
        assert!(bare.contains("The cattails here have no fluff."), "{bare}");
        assert!(bare.contains("The roots can be dug."), "{bare}");
        assert!(world
            .observe(near)
            .unwrap()
            .text
            .contains("You have a dry handful of cattail fluff."));
        let roots_taken = world.step(&[intent("roots", near, Attempt::Gather(Material::Cattail))]);
        assert_eq!(roots_taken.results[0].message, "You gather cattail roots.");
        assert!(world.pile_count(stand, Material::Cattail) > 0);

        world.time = world.tuning.chill_at;
        let iced = world.observe(near).unwrap().text;
        assert!(
            iced.contains("Cattails stand under the ice. The roots cannot be dug."),
            "{iced}"
        );
        assert!(!iced.contains("fluff hangs"), "{iced}");
        let frozen = world.step(&[intent("ice", near, Attempt::Gather(Material::Fluff))]);
        assert_eq!(
            frozen.results[0].message,
            "The cattails stand under the ice."
        );
        assert!(world
            .observe(near)
            .unwrap()
            .events
            .iter()
            .any(|event| { event.text == "The cattails stand under the ice." }));
        assert!(world
            .observe(seen)
            .unwrap()
            .events
            .iter()
            .all(|event| { event.text != "The cattails stand under the ice." }));

        let mut rain = World::valley(crate::tuning::Tuning::evening(), 1);
        rain.raining = true;
        let open = super::cell_id(4, 4).unwrap();
        let near = rain.spawn_at("Near", "n", 4, 4).unwrap();
        let seen = rain.spawn_at("Seen", "s", 4, 3).unwrap();
        let quiet = rain.spawn_at("Quiet", "q", 0, 4).unwrap();
        rain.mint_item(Material::Bark, 0, open, Some(near), super::Role::Loose);
        rain.mint_item(Material::Branch, 0, open, Some(near), super::Role::Loose);
        let bark = rain.step(&[intent("bark", near, Attempt::Light)]);
        assert_eq!(
            bark.results[0].message,
            "The rain puts the bark out before it catches."
        );
        rain.mint_item(
            Material::Fluff,
            rain.tuning.damp_at,
            open,
            Some(near),
            super::Role::Loose,
        );
        let open_rain = rain.step(&[intent("open", near, Attempt::Light)]);
        assert_eq!(
            open_rain.results[0].message,
            "The rain puts the cattail fluff out before it catches."
        );
        assert!(rain.fire_at(open).is_none());
        for person in [seen, quiet] {
            assert!(rain.observe(person).unwrap().events.iter().all(|event| {
                event.text != "The rain puts the cattail fluff out before it catches."
            }));
        }
        assert_eq!(rain.loose_ids(near, Material::Fluff).len(), 1);

        let mut damp = World::valley(crate::tuning::Tuning::evening(), 1);
        let floor = super::cell_id(4, 3).unwrap();
        let near = damp.spawn_at("Near", "n", 4, 3).unwrap();
        let seen = damp.spawn_at("Seen", "s", 3, 3).unwrap();
        damp.mint_item(
            Material::Moss,
            damp.tuning.damp_at,
            floor,
            Some(near),
            super::Role::Loose,
        );
        damp.mint_item(Material::Branch, 0, floor, Some(near), super::Role::Loose);
        let moss = damp.step(&[intent("moss", near, Attempt::Light)]);
        assert_eq!(moss.results[0].message, "The moss is too damp to catch.");
        assert!(damp
            .observe(seen)
            .unwrap()
            .events
            .iter()
            .all(|event| event.text != "The moss is too damp to catch."));
        damp.mint_item(
            Material::Fluff,
            damp.tuning.damp_at,
            floor,
            Some(near),
            super::Role::Loose,
        );
        let damp_look = damp.observe(near).unwrap();
        assert!(
            damp_look.text.contains("a damp handful of cattail fluff"),
            "{}",
            damp_look.text
        );
        assert!(!damp_look.text.contains("a dry handful of cattail fluff"));
        assert!(damp_look
            .events
            .iter()
            .all(|event| !event.text.contains("handful of cattail fluff")));
        let named = damp.examine(near, super::ExamineTarget::Fluff);
        assert!(
            named.contains(
                "A damp handful of cattail fluff is carried by you. It is not the roots."
            ),
            "{named}"
        );
        let caught = damp.step(&[intent("damp", near, Attempt::Light)]);
        let yours = "You light a fire with the cattail fluff and the branch from the arms.";
        let theirs = "Near lights a fire with the cattail fluff and the branch from the arms.";
        assert_eq!(caught.results[0].message, yours);
        assert!(damp.fire_at(floor).is_some());
        assert!(damp
            .observe(near)
            .unwrap()
            .events
            .iter()
            .all(|event| event.text != theirs));
        assert!(damp
            .observe(seen)
            .unwrap()
            .events
            .iter()
            .any(|event| event.text == theirs));

        let mut soaked = World::valley(crate::tuning::Tuning::evening(), 1);
        let floor = super::cell_id(4, 3).unwrap();
        let near = soaked.spawn_at("Near", "n", 4, 3).unwrap();
        let seen = soaked.spawn_at("Seen", "s", 3, 3).unwrap();
        soaked.mint_item(
            Material::Fluff,
            soaked.tuning.soaked_at,
            floor,
            Some(near),
            super::Role::Loose,
        );
        soaked.mint_item(Material::Branch, 0, floor, Some(near), super::Role::Loose);
        assert!(soaked
            .observe(near)
            .unwrap()
            .text
            .contains("a soaked handful of cattail fluff"));
        let refused = soaked.step(&[intent("soaked", near, Attempt::Light)]);
        assert_eq!(
            refused.results[0].message,
            "The cattail fluff is too damp to catch."
        );
        assert!(soaked
            .observe(seen)
            .unwrap()
            .events
            .iter()
            .all(|event| event.text != "The cattail fluff is too damp to catch."));
        assert!(soaked.fire_at(floor).is_none());
    }

    #[test]
    fn gathered_fluff_hangs_again_after_the_roots_wait_and_ice_holds_it() {
        let mut world = World::valley(crate::tuning::Tuning::evening(), 1);
        let stand = super::cell_id(2, 1).unwrap();
        let near = world.spawn_at("Near", "n", 2, 1).unwrap();
        let seen = world.spawn_at("Seen", "s", 3, 3).unwrap();
        let quiet = world.spawn_at("Quiet", "q", 5, 3).unwrap();
        let roots = world.pile_count(stand, Material::Cattail);
        let gathered = world.step(&[intent("fluff", near, Attempt::Gather(Material::Fluff))]);
        assert_eq!(
            gathered.results[0].message,
            "You gather a handful of cattail fluff."
        );
        assert_eq!(world.pile_count(stand, Material::Cattail), roots);
        assert!(world
            .observe(near)
            .unwrap()
            .text
            .contains("The cattails here have no fluff."));
        let missed = world.step(&[intent("none", near, Attempt::Gather(Material::Fluff))]);
        assert_eq!(
            missed.results[0].message,
            "There is no cattail fluff here to gather."
        );
        assert!(world
            .observe(near)
            .unwrap()
            .events
            .iter()
            .any(|event| { event.text == "There is no cattail fluff here to gather." }));
        assert!(world
            .observe(seen)
            .unwrap()
            .events
            .iter()
            .all(|event| { event.text != "There is no cattail fluff here to gather." }));
        let ticks = world.tuning.cattail_ticks;
        // The miss above was one of the wait's ticks. The due tick is still ice.
        for _ in 0..ticks - 2 {
            world.step(&[]);
        }
        assert!(world
            .observe(near)
            .unwrap()
            .text
            .contains("The cattails here have no fluff."));
        assert!(world
            .observe(near)
            .unwrap()
            .events
            .iter()
            .all(|event| { event.text != "Cattail fluff hangs on the cattails again." }));
        world.step(&[]);
        let held = world.observe(near).unwrap();
        assert!(held.text.contains("Ice covers it."), "{}", held.text);
        assert!(held.text.contains("The cattails here have no fluff."));
        assert!(held
            .events
            .iter()
            .all(|event| { event.text != "Cattail fluff hangs on the cattails again." }));
        let thaw = world.tuning.night_at + world.tuning.night_len;
        while world.time < thaw {
            world.step(&[]);
        }
        world.step(&[]);
        let grown = "Cattail fluff hangs on the cattails again.";
        let back = world.observe(near).unwrap();
        assert!(
            back.text
                .contains("Cattail fluff hangs on them. It is not the roots."),
            "{}",
            back.text
        );
        assert!(!back.text.contains("The cattails here have no fluff."));
        assert!(back.text.contains("The roots can be dug."));
        assert!(back.events.iter().any(|event| event.text == grown));
        assert!(back
            .events
            .iter()
            .all(|event| event.text != "Cattails grow again in the shallows."));
        assert!(world
            .observe(seen)
            .unwrap()
            .events
            .iter()
            .any(|event| event.text == grown));
        assert!(world
            .observe(quiet)
            .unwrap()
            .events
            .iter()
            .all(|event| event.text != grown));
        assert_eq!(world.pile_count(stand, Material::Cattail), roots);
        let again = world.step(&[intent("again", near, Attempt::Gather(Material::Fluff))]);
        assert_eq!(
            again.results[0].message,
            "You gather a handful of cattail fluff."
        );
    }
}

#[cfg(test)]
mod bark_cup {
    use super::*;

    fn intent(id: &str, actor: ActorId, attempt: Attempt) -> Intent {
        Intent {
            command_id: id.into(),
            actor_id: actor,
            attempt,
        }
    }

    #[test]
    fn a_bark_cup_holds_water_beside_a_clay_pot() {
        assert!(matches!(
            crate::parse::parse_attempt("fold bark"),
            Ok(Attempt::FoldCup)
        ));
        assert!(matches!(
            crate::parse::parse_attempt("make bark cup"),
            Ok(Attempt::FoldCup)
        ));
        assert!(matches!(
            crate::parse::parse_attempt("sew bark"),
            Ok(Attempt::FoldCup)
        ));
        assert!(matches!(
            crate::parse::parse_attempt("sew"),
            Ok(Attempt::Sew)
        ));
        assert!(matches!(
            crate::parse::parse_attempt("gather birch bark"),
            Ok(Attempt::Gather(Material::Bark))
        ));
        assert!(matches!(
            crate::parse::parse_attempt("weave basket"),
            Ok(Attempt::WeaveBasket)
        ));

        let mut world = World::valley(crate::tuning::Tuning::evening(), 1);
        let near = world.spawn_at("Near", "n", 4, 3).unwrap();
        let seen = world.spawn_at("Seen", "s", 3, 3).unwrap();
        let quiet = world.spawn_at("Quiet", "q", 5, 3).unwrap();
        let miss = world.step(&[intent("miss", near, Attempt::FoldCup)]);
        assert_eq!(
            miss.results[0].message,
            "You need a sheet of bark to fold a cup."
        );
        assert!(world
            .observe(near)
            .unwrap()
            .events
            .iter()
            .any(|event| { event.text == "You need a sheet of bark to fold a cup." }));
        for person in [seen, quiet] {
            assert!(world
                .observe(person)
                .unwrap()
                .events
                .iter()
                .all(|event| { event.text != "You need a sheet of bark to fold a cup." }));
        }

        for (i, dir) in [Dir::North, Dir::East, Dir::North, Dir::East]
            .into_iter()
            .enumerate()
        {
            world.step(&[intent(&format!("to-rock-{i}"), near, Attempt::Move(dir))]);
        }
        assert_eq!(world.actor(near).unwrap().cell, cell_id(6, 1).unwrap());
        let gathered = world.step(&[intent("bark", near, Attempt::Gather(Material::Bark))]);
        assert_eq!(gathered.results[0].message, "You gather a sheet of bark.");
        let folded = world.step(&[intent("fold", near, Attempt::FoldCup)]);
        assert_eq!(
            folded.results[0].message,
            "You fold a sheet of bark into a bark cup."
        );
        let near_fold = world.observe(near).unwrap();
        assert!(near_fold.text.contains("a bark cup"), "{}", near_fold.text);
        assert!(near_fold
            .events
            .iter()
            .all(|event| !event.text.contains("bark cup")));
        assert!(world
            .observe(quiet)
            .unwrap()
            .events
            .iter()
            .any(|event| { event.text == "Near folds a sheet of bark into a bark cup." }));
        assert!(world
            .observe(seen)
            .unwrap()
            .events
            .iter()
            .all(|event| { !event.text.contains("bark cup") }));

        for i in 0..4 {
            world.step(&[intent(
                &format!("to-still-{i}"),
                near,
                Attempt::Move(Dir::West),
            )]);
        }
        assert_eq!(world.actor(near).unwrap().cell, cell_id(2, 1).unwrap());
        assert!(world.time < world.tuning.chill_at);
        let water = world.step(&[intent("water", near, Attempt::Gather(Material::Water))]);
        assert_eq!(
            water.results[0].message,
            "You gather a drink of water from the still water into the bark cup."
        );
        let held = world.observe(near).unwrap();
        assert!(
            held.text.contains("a bark cup with a drink of water in it"),
            "{}",
            held.text
        );
        assert!(!held.text.contains("a pot with"));
        assert!(held
            .events
            .iter()
            .all(|event| { !event.text.starts_with("Near gathers a drink of water") }));
        assert!(world.observe(seen).unwrap().events.iter().any(|event| {
            event.text == "Near gathers a drink of water from the still water into the bark cup."
        }));
        assert!(world
            .observe(quiet)
            .unwrap()
            .events
            .iter()
            .all(|event| { !event.text.contains("into the bark cup") }));

        for i in 0..4 {
            world.step(&[intent(&format!("back-{i}"), near, Attempt::Move(Dir::East))]);
        }
        assert_eq!(world.actor(near).unwrap().cell, cell_id(6, 1).unwrap());
        let lit = world.step(&[intent("light", near, Attempt::Light)]);
        assert!(lit.results[0].success, "{}", lit.results[0].message);
        let boiled = world.step(&[intent("boil", near, Attempt::Boil)]);
        assert!(
            boiled.results[0]
                .message
                .starts_with("You boil the drink in the bark cup over the fire and drink it."),
            "{}",
            boiled.results[0].message
        );
        assert!(!boiled.results[0].message.contains("pot"));
        assert!(world
            .observe(near)
            .unwrap()
            .events
            .iter()
            .all(|event| { !event.text.contains("boils the drink in the bark cup") }));
        assert!(world.observe(quiet).unwrap().events.iter().any(|event| {
            event
                .text
                .starts_with("Near boils the drink in the bark cup over the fire and drinks it.")
        }));
        assert!(world
            .observe(seen)
            .unwrap()
            .events
            .iter()
            .all(|event| { !event.text.contains("boils the drink") }));

        let mut pots = World::valley(crate::tuning::Tuning::evening(), 1);
        let potter = pots.spawn_at("Potter", "p", 1, 1).unwrap();
        let cell = pots.actor(potter).unwrap().cell;
        pots.mint_item(Material::Bark, 0, cell, Some(potter), Role::Loose);
        let cup = pots.step(&[intent("fold-pot", potter, Attempt::FoldCup)]);
        assert_eq!(
            cup.results[0].message,
            "You fold a sheet of bark into a bark cup."
        );
        pots.mint_item(Material::Pot, 0, cell, Some(potter), Role::Loose);
        pots.mint_fire(cell, 0);
        let into = pots.step(&[intent(
            "pot-water",
            potter,
            Attempt::Gather(Material::Water),
        )]);
        assert_eq!(
            into.results[0].message,
            "You gather a drink of water from the brook into the pot."
        );
        let boiled_pot = pots.step(&[intent("pot-boil", potter, Attempt::Boil)]);
        assert!(
            boiled_pot.results[0]
                .message
                .starts_with("You boil the drink in the pot over the fire and drink it."),
            "{}",
            boiled_pot.results[0].message
        );
        assert!(!boiled_pot.results[0].message.contains("bark cup"));
    }

    #[test]
    fn a_drop_stays_with_the_reply_and_a_soaked_cup_is_named() {
        let mut world = World::valley(crate::tuning::Tuning::evening(), 1);
        let near = world.spawn_at("Near", "n", 1, 1).unwrap();
        let seen = world.spawn_at("Seen", "s", 3, 3).unwrap();
        let quiet = world.spawn_at("Quiet", "q", 5, 3).unwrap();
        let cell = world.actor(near).unwrap().cell;
        let miss = world.step(&[intent("miss", near, Attempt::Drop(Material::Pot))]);
        assert_eq!(miss.results[0].message, "You are not carrying a loose pot.");
        assert!(world
            .observe(near)
            .unwrap()
            .events
            .iter()
            .any(|event| { event.text == "You are not carrying a loose pot." }));
        for person in [seen, quiet] {
            assert!(world
                .observe(person)
                .unwrap()
                .events
                .iter()
                .all(|event| { event.text != "You are not carrying a loose pot." }));
        }
        world.mint_item(Material::Bark, 0, cell, Some(near), Role::Loose);
        world.step(&[intent("fold", near, Attempt::FoldCup)]);
        let dry = world.step(&[intent("dry", near, Attempt::Drop(Material::BarkCup))]);
        assert_eq!(
            dry.results[0].message,
            "You drop a bark cup. It is in contact with the ground."
        );
        assert!(world
            .observe(near)
            .unwrap()
            .events
            .iter()
            .all(|event| { !event.text.contains("drops") }));
        assert!(world.observe(seen).unwrap().events.iter().any(|event| {
            event.text == "Near drops a bark cup. It is in contact with the ground."
        }));
        assert!(world
            .observe(quiet)
            .unwrap()
            .events
            .iter()
            .all(|event| { !event.text.contains("drops") }));
        let lifted = world.step(&[intent("lift", near, Attempt::Gather(Material::BarkCup))]);
        assert_eq!(lifted.results[0].message, "You gather a bark cup.");
        world.step(&[intent("water", near, Attempt::Gather(Material::Water))]);
        let ticks = world.tuning.cup_leak_ticks;
        for _ in 0..ticks {
            world.step(&[]);
        }
        let wet = world.step(&[intent("wet", near, Attempt::Drop(Material::BarkCup))]);
        assert_eq!(
            wet.results[0].message,
            "You drop a soaked bark cup. It is in contact with the ground."
        );
        assert!(world
            .observe(near)
            .unwrap()
            .events
            .iter()
            .all(|event| { !event.text.contains("drops") }));
        assert!(world.observe(seen).unwrap().events.iter().any(|event| {
            event.text == "Near drops a soaked bark cup. It is in contact with the ground."
        }));
        assert!(world
            .observe(quiet)
            .unwrap()
            .events
            .iter()
            .all(|event| { !event.text.contains("drops") }));
        assert!(world
            .observe(near)
            .unwrap()
            .text
            .contains("A soaked bark cup lies here."));
        let lifted = world.step(&[intent("lift-wet", near, Attempt::Gather(Material::BarkCup))]);
        assert_eq!(lifted.results[0].message, "You gather a soaked bark cup.");
        assert!(world
            .observe(near)
            .unwrap()
            .events
            .iter()
            .all(|event| { !event.text.contains("gathers a soaked bark cup") }));
        assert!(world
            .observe(seen)
            .unwrap()
            .events
            .iter()
            .any(|event| { event.text == "Near gathers a soaked bark cup." }));
        assert!(world
            .observe(quiet)
            .unwrap()
            .events
            .iter()
            .all(|event| { !event.text.contains("gathers a soaked bark cup") }));
        assert!(world
            .observe(near)
            .unwrap()
            .text
            .contains("a soaked bark cup"));
    }

    #[test]
    fn a_sealed_bark_cup_is_named_when_it_is_dropped() {
        let mut world = World::valley(crate::tuning::Tuning::evening(), 1);
        let near = world.spawn_at("Near", "n", 1, 1).unwrap();
        let seen = world.spawn_at("Seen", "s", 3, 3).unwrap();
        let quiet = world.spawn_at("Quiet", "q", 5, 3).unwrap();
        let cell = world.actor(near).unwrap().cell;
        world.mint_item(Material::Bark, 0, cell, Some(near), Role::Loose);
        world.mint_item(Material::Resin, 0, cell, Some(near), Role::Loose);
        world.step(&[intent("fold", near, Attempt::FoldCup)]);
        if let Some(cup) = world
            .items
            .values_mut()
            .find(|item| item.material == Material::BarkCup)
        {
            cup.moisture = world.tuning.soaked_at;
        }
        let sealed = world.step(&[intent("seal", near, Attempt::SealCup)]);
        assert_eq!(
            sealed.results[0].message,
            "You seal the bark cup with pine resin."
        );
        let dropped = world.step(&[intent("drop", near, Attempt::Drop(Material::BarkCup))]);
        assert_eq!(
            dropped.results[0].message,
            "You drop a sealed bark cup. It is in contact with the ground."
        );
        assert!(world
            .observe(near)
            .unwrap()
            .events
            .iter()
            .all(|event| { !event.text.contains("drops") }));
        assert!(world.observe(seen).unwrap().events.iter().any(|event| {
            event.text == "Near drops a sealed bark cup. It is in contact with the ground."
        }));
        assert!(world
            .observe(quiet)
            .unwrap()
            .events
            .iter()
            .all(|event| { !event.text.contains("drops") }));
        let ground = world.observe(near).unwrap().text;
        assert!(ground.contains("A sealed bark cup lies here."), "{ground}");
        assert!(!ground.contains("A soaked bark cup"));
        let lifted = world.step(&[intent(
            "lift-sealed",
            near,
            Attempt::Gather(Material::BarkCup),
        )]);
        assert_eq!(lifted.results[0].message, "You gather a sealed bark cup.");
        assert!(world
            .observe(near)
            .unwrap()
            .events
            .iter()
            .all(|event| { !event.text.contains("gathers") }));
        assert!(world
            .observe(seen)
            .unwrap()
            .events
            .iter()
            .any(|event| { event.text == "Near gathers a sealed bark cup." }));
        assert!(world
            .observe(quiet)
            .unwrap()
            .events
            .iter()
            .all(|event| { !event.text.contains("gathers") }));
        let arms = world.observe(near).unwrap().text;
        assert!(arms.contains("a sealed bark cup"), "{arms}");
        assert!(!arms.contains("a soaked bark cup"));
    }

    #[test]
    fn pine_resin_seals_a_bark_cup_and_an_unsealed_one_leaks() {
        assert!(matches!(
            crate::parse::parse_attempt("seal bark cup"),
            Ok(Attempt::SealCup)
        ));
        assert!(matches!(
            crate::parse::parse_attempt("pitch the bark cup"),
            Ok(Attempt::SealCup)
        ));
        assert!(matches!(
            crate::parse::parse_attempt("seal canoe"),
            Ok(Attempt::SealCanoe)
        ));

        let mut world = World::valley(crate::tuning::Tuning::evening(), 1);
        let near = world.spawn_at("Near", "n", 1, 1).unwrap();
        let seen = world.spawn_at("Seen", "s", 3, 3).unwrap();
        let quiet = world.spawn_at("Quiet", "q", 5, 3).unwrap();
        let cell = world.actor(near).unwrap().cell;
        world.mint_item(Material::Bark, 0, cell, Some(near), Role::Loose);
        world.step(&[intent("fold", near, Attempt::FoldCup)]);
        let bare = world.step(&[intent("bare", near, Attempt::SealCup)]);
        assert_eq!(
            bare.results[0].message,
            "You have no pine resin to seal the cup."
        );
        assert!(world
            .observe(near)
            .unwrap()
            .events
            .iter()
            .any(|event| { event.text == "You have no pine resin to seal the cup." }));
        for person in [seen, quiet] {
            assert!(world
                .observe(person)
                .unwrap()
                .events
                .iter()
                .all(|event| { event.text != "You have no pine resin to seal the cup." }));
        }
        assert!(world
            .observe(near)
            .unwrap()
            .text
            .contains("You have a bark cup."));
        assert!(!world.observe(near).unwrap().text.contains("sealed"));

        let open = world.step(&[intent("water", near, Attempt::Gather(Material::Water))]);
        assert_eq!(
            open.results[0].message,
            "You gather a drink of water from the brook into the bark cup."
        );
        let ticks = world.tuning.cup_leak_ticks;
        for _ in 0..ticks {
            world.step(&[]);
        }
        let soaked = world.observe(near).unwrap();
        assert!(soaked
            .events
            .iter()
            .any(|event| event.text == "The drink soaks through the bark cup."));
        assert!(soaked.text.contains("a soaked bark cup"), "{}", soaked.text);
        assert!(!soaked.text.contains("You have a bark cup."));
        assert!(!soaked.text.contains("drink of"));
        assert!(soaked
            .events
            .iter()
            .all(|event| event.text != "The bark is soaked."));
        let examined = world.examine(near, ExamineTarget::BarkCup);
        assert!(
            examined.contains("a soaked bark cup") && examined.contains("The bark is soaked."),
            "{examined}"
        );
        assert!(world
            .examine(seen, ExamineTarget::BarkCup)
            .contains("soaked bark cup"));
        assert!(!world
            .examine(quiet, ExamineTarget::BarkCup)
            .contains("soaked"));
        assert!(world
            .observe(seen)
            .unwrap()
            .events
            .iter()
            .any(|event| { event.text == "The drink soaks through the bark cup." }));
        assert!(world
            .observe(quiet)
            .unwrap()
            .events
            .iter()
            .all(|event| { event.text != "The drink soaks through the bark cup." }));

        world.mint_item(Material::Resin, 0, cell, Some(near), Role::Loose);
        world.mint_item(Material::Resin, 0, cell, Some(near), Role::Loose);
        let sealed = world.step(&[intent("seal", near, Attempt::SealCup)]);
        assert_eq!(
            sealed.results[0].message,
            "You seal the bark cup with pine resin."
        );
        assert_eq!(world.loose_ids(near, Material::Resin).len(), 1);
        let named = world.observe(near).unwrap();
        assert!(named.text.contains("a sealed bark cup"), "{}", named.text);
        assert!(named.events.iter().all(|event| {
            !event.text.contains("seals the bark cup") && !event.text.contains("You seal")
        }));
        assert!(world
            .observe(seen)
            .unwrap()
            .events
            .iter()
            .any(|event| { event.text == "Near seals the bark cup with pine resin." }));
        assert!(world
            .observe(quiet)
            .unwrap()
            .events
            .iter()
            .all(|event| { !event.text.contains("seals the bark cup") }));
        let again = world.step(&[intent("again", near, Attempt::SealCup)]);
        assert_eq!(again.results[0].message, "The bark cup is already sealed.");
        assert_eq!(world.loose_ids(near, Material::Resin).len(), 1);
        assert!(world
            .observe(near)
            .unwrap()
            .events
            .iter()
            .any(|event| { event.text == "The bark cup is already sealed." }));
        assert!(world
            .observe(seen)
            .unwrap()
            .events
            .iter()
            .all(|event| { event.text != "The bark cup is already sealed." }));

        let held = world.step(&[intent("held", near, Attempt::Gather(Material::Water))]);
        assert_eq!(
            held.results[0].message,
            "You gather a drink of water from the brook into the sealed bark cup."
        );
        assert!(world
            .observe(near)
            .unwrap()
            .text
            .contains("a sealed bark cup with a drink of raw water in it"));
        let soaked_before = world
            .observe(near)
            .unwrap()
            .events
            .iter()
            .filter(|event| event.text == "The drink soaks through the bark cup.")
            .count();
        for _ in 0..ticks {
            world.step(&[]);
        }
        let kept = world.observe(near).unwrap();
        let soaked_after = kept
            .events
            .iter()
            .filter(|event| event.text == "The drink soaks through the bark cup.")
            .count();
        assert_eq!(soaked_before, soaked_after);
        assert!(
            kept.text
                .contains("a sealed bark cup with a drink of raw water in it"),
            "{}",
            kept.text
        );
    }
}

#[cfg(test)]
mod flour {
    use super::*;

    fn intent(id: &str, actor: ActorId, attempt: Attempt) -> Intent {
        Intent {
            command_id: id.into(),
            actor_id: actor,
            attempt,
        }
    }

    #[test]
    fn cattail_roots_grind_to_flour_and_cook_to_flatbread() {
        assert!(matches!(
            crate::parse::parse_attempt("grind"),
            Ok(Attempt::Grind)
        ));
        assert!(matches!(
            crate::parse::parse_attempt("grind cattail roots"),
            Ok(Attempt::Grind)
        ));
        assert!(matches!(
            crate::parse::parse_attempt("eat flatbread"),
            Ok(Attempt::Eat(Material::Flour))
        ));

        let mut world = World::valley(crate::tuning::Tuning::evening(), 1);
        let near = world.spawn_at("Near", "n", 4, 3).unwrap();
        let seen = world.spawn_at("Seen", "s", 3, 3).unwrap();
        let quiet = world.spawn_at("Quiet", "q", 5, 3).unwrap();
        let rock = cell_id(6, 1).unwrap();

        let miss = world.step(&[intent("miss", near, Attempt::Grind)]);
        assert_eq!(
            miss.results[0].message,
            "You have no cattail roots to grind."
        );
        assert!(world
            .observe(near)
            .unwrap()
            .events
            .iter()
            .any(|event| { event.text == "You have no cattail roots to grind." }));
        for person in [seen, quiet] {
            assert!(world
                .observe(person)
                .unwrap()
                .events
                .iter()
                .all(|event| { event.text != "You have no cattail roots to grind." }));
        }

        let cell = world.actor(near).unwrap().cell;
        let wet = world.mint_item(
            Material::Cattail,
            world.tuning.soaked_at,
            cell,
            Some(near),
            Role::Loose,
        );
        let too_wet = world.step(&[intent("wet", near, Attempt::Grind)]);
        assert_eq!(
            too_wet.results[0].message,
            "The cattail roots are too wet to grind."
        );
        world.items.remove(&wet);

        world.mint_item(Material::Cattail, 0, cell, Some(near), Role::Loose);
        let no_stone = world.step(&[intent("nostone", near, Attempt::Grind)]);
        assert_eq!(no_stone.results[0].message, "You need a stone to grind on.");
        world.mint_item(Material::Stone, 0, cell, Some(near), Role::Loose);
        let no_under = world.step(&[intent("nounder", near, Attempt::Grind)]);
        assert_eq!(
            no_under.results[0].message,
            "You need a stone on stone to grind."
        );
        for person in [seen, quiet] {
            assert!(world.observe(person).unwrap().events.iter().all(|event| {
                event.text != "You need a stone to grind on."
                    && event.text != "You need a stone on stone to grind."
                    && event.text != "The cattail roots are too wet to grind."
            }));
        }

        world.actors.get_mut(&near).unwrap().cell = rock;
        let ground = world.step(&[intent("grind", near, Attempt::Grind)]);
        assert_eq!(
            ground.results[0].message,
            "You grind the cattail roots into flour on the stone."
        );
        let arms = world.observe(near).unwrap().text;
        assert!(arms.contains("cattail flour"), "{arms}");
        assert!(arms.contains("a dry stone"), "{arms}");
        assert!(world.loose_ids(near, Material::Cattail).is_empty());
        assert_eq!(world.loose_ids(near, Material::Stone).len(), 1);
        assert!(world.observe(near).unwrap().events.iter().all(|event| {
            event.text != "Near grinds the cattail roots into flour on the stone."
        }));
        assert!(world.observe(quiet).unwrap().events.iter().any(|event| {
            event.text == "Near grinds the cattail roots into flour on the stone."
        }));
        assert!(world.observe(seen).unwrap().events.iter().all(|event| {
            event.text != "Near grinds the cattail roots into flour on the stone."
        }));

        let raw = world.step(&[intent("rawflour", near, Attempt::Eat(Material::Flour))]);
        assert_eq!(
            raw.results[0].message,
            "Raw cattail flour does not feed you. It must be cooked."
        );
        assert!(world.observe(seen).unwrap().events.iter().all(|event| {
            event.text != "Raw cattail flour does not feed you. It must be cooked."
        }));

        let cold = world.step(&[intent("cold", near, Attempt::Cook)]);
        assert_eq!(cold.results[0].message, "There is no fire burning here.");

        world.mint_fire(rock, 0);
        let cooked = world.step(&[intent("cook", near, Attempt::Cook)]);
        assert_eq!(
            cooked.results[0].message,
            "You cook the flour into a flatbread over the fire."
        );
        let baked = world.observe(near).unwrap().text;
        assert!(baked.contains("a cattail flatbread"), "{baked}");
        assert!(world
            .observe(near)
            .unwrap()
            .events
            .iter()
            .all(|event| { event.text != "Near cooks the flour into a flatbread over the fire." }));
        assert!(world
            .observe(quiet)
            .unwrap()
            .events
            .iter()
            .any(|event| { event.text == "Near cooks the flour into a flatbread over the fire." }));
        assert!(world
            .observe(seen)
            .unwrap()
            .events
            .iter()
            .all(|event| { event.text != "Near cooks the flour into a flatbread over the fire." }));

        let eaten = world.step(&[intent("eat", near, Attempt::Eat(Material::Flour))]);
        assert!(
            eaten.results[0]
                .message
                .starts_with("You eat the cattail flatbread."),
            "{}",
            eaten.results[0].message
        );
        assert!(world.loose_ids(near, Material::Flour).is_empty());
        assert!(world
            .observe(near)
            .unwrap()
            .events
            .iter()
            .all(|event| { event.text != "Near eats the cattail flatbread." }));
        assert!(world
            .observe(quiet)
            .unwrap()
            .events
            .iter()
            .any(|event| { event.text == "Near eats the cattail flatbread." }));

        let cooked_root = world.mint_item(Material::Cattail, 0, rock, Some(near), Role::Loose);
        world.items.get_mut(&cooked_root).unwrap().cooked = true;
        let again = world.step(&[intent("cooked-root", near, Attempt::Grind)]);
        assert_eq!(
            again.results[0].message,
            "You grind the cattail roots into flour on the stone."
        );
        assert!(world.loose_ids(near, Material::Flour).len() == 1);
        assert!(world.loose_ids(near, Material::Cattail).is_empty());
    }
}

#[cfg(test)]
mod gather_step {
    use super::*;

    fn intent(id: &str, actor: ActorId, attempt: Attempt) -> Intent {
        Intent {
            command_id: id.into(),
            actor_id: actor,
            attempt,
        }
    }

    #[test]
    fn a_gather_one_cell_off_names_the_step_and_stays_private() {
        let mut world = World::valley(crate::tuning::Tuning::evening(), 1);
        let near = world.spawn_at("Near", "n", 1, 4).unwrap();
        let seen = world.spawn_at("Seen", "s", 1, 3).unwrap();
        let quiet = world.spawn_at("Quiet", "q", 6, 4).unwrap();

        let far = world.step(&[intent("far", quiet, Attempt::Gather(Material::Willow))]);
        assert_eq!(far.results[0].message, "There is no willow here to gather.");
        assert!(world
            .observe(quiet)
            .unwrap()
            .events
            .iter()
            .any(|event| { event.text == "There is no willow here to gather." }));
        assert!(world
            .observe(near)
            .unwrap()
            .events
            .iter()
            .all(|event| { event.text != "There is no willow here to gather." }));

        let beside = world.step(&[intent("beside", near, Attempt::Gather(Material::Willow))]);
        assert_eq!(
            beside.results[0].message,
            "There is no willow here to gather. Step west."
        );
        assert!(world
            .observe(near)
            .unwrap()
            .events
            .iter()
            .any(|event| { event.text == "There is no willow here to gather. Step west." }));
        for person in [seen, quiet] {
            assert!(world
                .observe(person)
                .unwrap()
                .events
                .iter()
                .all(|event| { !event.text.contains("Step west") }));
        }
        assert_eq!(cell_xy(world.actor(near).unwrap().cell), (1, 4));

        let walked = world.step(&[intent("walk", near, Attempt::Move(Dir::West))]);
        assert!(walked.results[0].message.contains("You walk"));
        let gathered = world.step(&[intent("take", near, Attempt::Gather(Material::Willow))]);
        assert_eq!(
            gathered.results[0].message,
            "You gather a bundle of willow cordage."
        );
        assert!(world
            .observe(near)
            .unwrap()
            .events
            .iter()
            .all(|event| { event.text != "Near gathers a bundle of willow cordage." }));
        assert!(world
            .observe(seen)
            .unwrap()
            .events
            .iter()
            .any(|event| { event.text == "Near gathers a bundle of willow cordage." }));

        world.actors.get_mut(&near).unwrap().cell = cell_id(2, 1).unwrap();
        let salt = world.step(&[intent("salt", near, Attempt::Gather(Material::Salt))]);
        assert_eq!(
            salt.results[0].message,
            "There is no salt here to gather. Step north."
        );
        world.actors.get_mut(&near).unwrap().cell = cell_id(1, 0).unwrap();
        let both = world.step(&[intent("both", near, Attempt::Gather(Material::Salt))]);
        assert_eq!(
            both.results[0].message,
            "There is no salt here to gather. Step east or west."
        );

        world.actors.get_mut(&near).unwrap().cell = cell_id(1, 3).unwrap();
        let reeds = world.step(&[intent("reeds", near, Attempt::Gather(Material::Reed))]);
        assert_eq!(
            reeds.results[0].message,
            "There is no reed here to gather. Step west."
        );

        world.actors.get_mut(&near).unwrap().cell = cell_id(5, 1).unwrap();
        let ochre = world.step(&[intent("ochre", near, Attempt::Gather(Material::Ochre))]);
        assert_eq!(
            ochre.results[0].message,
            "There is no ochre here to gather. Step east."
        );

        let rock = cell_id(6, 1).unwrap();
        world.mint_item(Material::Charcoal, 0, rock, None, Role::Loose);
        let charcoal = world.step(&[intent("char", near, Attempt::Gather(Material::Charcoal))]);
        assert_eq!(
            charcoal.results[0].message,
            "There is no charcoal here to gather. Step east."
        );
        assert_eq!(cell_xy(world.actor(near).unwrap().cell), (5, 1));
    }
}

#[cfg(test)]
mod arms {
    use super::{cell_id, Attempt, Intent, Material, Role, World};

    fn intent(id: &str, actor: u32, attempt: Attempt) -> Intent {
        Intent {
            command_id: id.into(),
            actor_id: actor,
            attempt,
        }
    }

    #[test]
    fn five_slots_hold_charcoal_ore_tinder_and_a_tool() {
        let mut world = World::valley(crate::tuning::Tuning::evening(), 1);
        assert_eq!(world.tuning.carry_slots, 5);
        let cell = cell_id(4, 4).unwrap();
        let near = world.spawn_at("Near", "n", 4, 4).unwrap();
        let seen = world.spawn_at("Seen", "s", 4, 2).unwrap();
        let quiet = world.spawn_at("Quiet", "q", 0, 0).unwrap();
        for material in [
            Material::Charcoal,
            Material::IronOre,
            Material::Bark,
            Material::Stone,
        ] {
            world.mint_item(material, 0, cell, Some(near), Role::Loose);
        }
        assert_eq!(world.carry_count(near), 4);
        world.mint_item(Material::Branch, 0, cell, None, Role::Loose);
        let fifth = world.step(&[intent("fifth", near, Attempt::Gather(Material::Branch))]);
        assert_eq!(fifth.results[0].message, "You gather a branch.");
        assert_eq!(world.carry_count(near), 5);
        let arms = world.observe(near).unwrap().text;
        assert!(arms.contains("charcoal"), "{arms}");
        assert!(arms.contains("iron ore"), "{arms}");
        assert!(arms.contains("bark"), "{arms}");
        assert!(arms.contains("stone"), "{arms}");
        world.mint_item(Material::Berry, 0, cell, None, Role::Loose);
        let full = world.step(&[intent("full", near, Attempt::Gather(Material::Berry))]);
        let sentence = "Your arms are full. Drop something first.";
        assert_eq!(full.results[0].message, sentence);
        assert!(world
            .observe(near)
            .unwrap()
            .events
            .iter()
            .any(|event| event.text == sentence));
        for person in [seen, quiet] {
            assert!(world
                .observe(person)
                .unwrap()
                .events
                .iter()
                .all(|event| event.text != sentence));
        }
        assert_eq!(world.carry_count(near), 5);
    }

    fn cloak(
        world: &mut World,
        cell: super::CellId,
        actor: u32,
        wolf: bool,
        otter: bool,
        fox: bool,
        hare: bool,
        beaver: bool,
        sewn: bool,
        worn: bool,
    ) -> u32 {
        let id = world.mint_item(Material::Hide, 0, cell, Some(actor), Role::Loose);
        let hide = world.items.get_mut(&id).unwrap();
        hide.wolf = wolf;
        hide.otter = otter;
        hide.fox = fox;
        hide.hare = hare;
        hide.beaver = beaver;
        hide.sewn = sewn;
        hide.worn = worn;
        id
    }

    #[test]
    fn a_worn_sewn_cloak_does_not_take_a_slot() {
        let mut world = World::valley(crate::tuning::Tuning::evening(), 1);
        let cell = cell_id(4, 4).unwrap();
        let near = world.spawn_at("Near", "n", 4, 4).unwrap();
        let seen = world.spawn_at("Seen", "s", 4, 2).unwrap();
        let quiet = world.spawn_at("Quiet", "q", 0, 0).unwrap();
        for material in [
            Material::Charcoal,
            Material::IronOre,
            Material::Bark,
            Material::Stone,
        ] {
            world.mint_item(material, 0, cell, Some(near), Role::Loose);
        }
        cloak(
            &mut world, cell, near, false, false, false, false, false, true, false,
        );
        assert_eq!(world.carry_count(near), 5);
        world.mint_item(Material::Branch, 0, cell, None, Role::Loose);
        let held = world.step(&[intent("held", near, Attempt::Gather(Material::Branch))]);
        let sentence = "Your arms are full. Drop something first.";
        assert_eq!(held.results[0].message, sentence);

        let wore = world.step(&[intent("wear", near, Attempt::Wear)]);
        assert_eq!(
            wore.results[0].message,
            "You wear the sewn deer-hide cloak."
        );
        assert_eq!(world.carry_count(near), 4);
        let fifth = world.step(&[intent("fifth", near, Attempt::Gather(Material::Branch))]);
        assert_eq!(fifth.results[0].message, "You gather a branch.");
        assert_eq!(world.carry_count(near), 5);
        world.mint_item(Material::Berry, 0, cell, None, Role::Loose);
        let full = world.step(&[intent("full", near, Attempt::Gather(Material::Berry))]);
        assert_eq!(full.results[0].message, sentence);
        assert!(world
            .observe(near)
            .unwrap()
            .events
            .iter()
            .any(|event| event.text == sentence));
        for person in [seen, quiet] {
            assert!(world
                .observe(person)
                .unwrap()
                .events
                .iter()
                .all(|event| event.text != sentence));
        }

        let off = world.step(&[intent("off", near, Attempt::Remove)]);
        assert_eq!(
            off.results[0].message,
            "You remove the sewn deer-hide cloak. The wind reaches you again."
        );
        assert_eq!(world.carry_count(near), 6);
        let dropped = world.step(&[intent("drop", near, Attempt::Drop(Material::Branch))]);
        assert!(dropped.results[0].success, "{}", dropped.results[0].message);
        assert_eq!(world.carry_count(near), 5);
        let still = world.step(&[intent("still", near, Attempt::Gather(Material::Berry))]);
        assert_eq!(still.results[0].message, sentence);
        assert!(world
            .observe(seen)
            .unwrap()
            .events
            .iter()
            .all(|event| event.text != sentence));

        let again = world.step(&[intent("again", near, Attempt::Wear)]);
        assert_eq!(
            again.results[0].message,
            "You wear the sewn deer-hide cloak."
        );
        assert_eq!(world.carry_count(near), 4);
        let fits = world.step(&[intent("fits", near, Attempt::Gather(Material::Berry))]);
        assert_eq!(fits.results[0].message, "You gather a handful of berries.");
        assert_eq!(world.carry_count(near), 5);

        let mut plain = World::valley(crate::tuning::Tuning::evening(), 1);
        let plain_cell = cell_id(4, 4).unwrap();
        let bare = plain.spawn_at("Near", "n", 4, 4).unwrap();
        for material in [
            Material::Charcoal,
            Material::IronOre,
            Material::Bark,
            Material::Stone,
        ] {
            plain.mint_item(material, 0, plain_cell, Some(bare), Role::Loose);
        }
        cloak(
            &mut plain, plain_cell, bare, false, false, false, false, false, false, false,
        );
        assert_eq!(plain.carry_count(bare), 5);
        let wore_hide = plain.step(&[intent("wear-hide", bare, Attempt::Wear)]);
        assert_eq!(wore_hide.results[0].message, "You wear the deer hide.");
        assert_eq!(plain.carry_count(bare), 5);
        plain.mint_item(Material::Branch, 0, plain_cell, None, Role::Loose);
        let blocked = plain.step(&[intent("blocked", bare, Attempt::Gather(Material::Branch))]);
        assert_eq!(blocked.results[0].message, sentence);

        for (wolf, otter, fox, hare, beaver) in [
            (true, false, false, false, false),
            (false, true, false, false, false),
            (false, false, true, false, false),
            (false, false, false, true, false),
            (false, false, false, false, true),
        ] {
            let mut world = World::valley(crate::tuning::Tuning::evening(), 1);
            let cell = cell_id(4, 4).unwrap();
            let actor = world.spawn_at("Near", "n", 4, 4).unwrap();
            for _ in 0..5 {
                world.mint_item(Material::Stone, 0, cell, Some(actor), Role::Loose);
            }
            cloak(
                &mut world, cell, actor, wolf, otter, fox, hare, beaver, true, true,
            );
            assert_eq!(world.carry_count(actor), 5);
            let id = world
                .loose_ids(actor, Material::Hide)
                .into_iter()
                .next()
                .unwrap();
            world.items.get_mut(&id).unwrap().worn = false;
            assert_eq!(world.carry_count(actor), 6);
        }
    }

    #[test]
    fn a_worn_cloak_is_named_after_what_the_arms_hold() {
        let mut world = World::valley(crate::tuning::Tuning::evening(), 1);
        let cell = cell_id(4, 4).unwrap();
        let near = world.spawn_at("Near", "n", 4, 4).unwrap();
        for material in [
            Material::Charcoal,
            Material::IronOre,
            Material::Bark,
            Material::Stone,
            Material::Pot,
        ] {
            world.mint_item(material, 0, cell, Some(near), Role::Loose);
        }
        cloak(
            &mut world, cell, near, false, false, false, false, false, true, true,
        );
        let line = world.arms_line(near);
        let (have, wear) = line.split_once(" You wear ").unwrap();
        assert!(have.starts_with("You have "), "{line}");
        assert!(!have.contains("cloak"), "{line}");
        assert!(have.contains("charcoal"), "{line}");
        assert!(have.contains("iron ore"), "{line}");
        assert!(have.contains("bark"), "{line}");
        assert!(have.contains("stone"), "{line}");
        assert!(have.contains("pot"), "{line}");
        assert_eq!(wear, "a sewn deer-hide cloak.");
        assert_eq!(world.observe(near).unwrap().text.contains(&line), true);
        for (wolf, otter, fox, hare, beaver, phrase) in [
            (true, false, false, false, false, "a sewn wolf-hide cloak."),
            (false, true, false, false, false, "a sewn otter-hide cloak."),
            (false, false, true, false, false, "a sewn fox-hide cloak."),
            (false, false, false, true, false, "a sewn hare-hide cloak."),
            (
                false,
                false,
                false,
                false,
                true,
                "a sewn beaver-hide cloak.",
            ),
        ] {
            let mut world = World::valley(crate::tuning::Tuning::evening(), 1);
            let cell = cell_id(4, 4).unwrap();
            let actor = world.spawn_at("Near", "n", 4, 4).unwrap();
            world.mint_item(Material::Stone, 0, cell, Some(actor), Role::Loose);
            cloak(
                &mut world, cell, actor, wolf, otter, fox, hare, beaver, true, true,
            );
            let line = world.arms_line(actor);
            assert!(
                line.starts_with("You have ") && line.ends_with(&format!(" You wear {phrase}")),
                "{line}"
            );
            let have = line.split(" You wear ").next().unwrap();
            assert!(!have.contains("cloak"), "{line}");
        }
        let mut carried = World::valley(crate::tuning::Tuning::evening(), 1);
        let cell = cell_id(4, 4).unwrap();
        let actor = carried.spawn_at("Near", "n", 4, 4).unwrap();
        cloak(
            &mut carried,
            cell,
            actor,
            false,
            false,
            false,
            false,
            false,
            true,
            false,
        );
        let line = carried.arms_line(actor);
        assert_eq!(line, "You have a sewn deer-hide cloak.");
        let mut plain = World::valley(crate::tuning::Tuning::evening(), 1);
        let cell = cell_id(4, 4).unwrap();
        let actor = plain.spawn_at("Near", "n", 4, 4).unwrap();
        cloak(
            &mut plain, cell, actor, false, false, false, false, false, false, true,
        );
        let line = plain.arms_line(actor);
        assert_eq!(line, "You have a deer hide on you.");
        let mut only = World::valley(crate::tuning::Tuning::evening(), 1);
        let cell = cell_id(4, 4).unwrap();
        let actor = only.spawn_at("Near", "n", 4, 4).unwrap();
        cloak(
            &mut only, cell, actor, false, false, false, false, false, true, true,
        );
        assert_eq!(
            only.arms_line(actor),
            "Your arms are empty. You wear a sewn deer-hide cloak."
        );
    }
}

#[cfg(test)]
mod mint_tea {
    use super::{cell_id, Attempt, Dir, ExamineTarget, Intent, Material, Role, World};

    fn intent(id: &str, actor: u32, attempt: Attempt) -> Intent {
        Intent {
            command_id: id.into(),
            actor_id: actor,
            attempt,
        }
    }

    #[test]
    fn wild_mint_on_the_wet_bank_steeps_into_tea() {
        assert!(matches!(
            crate::parse::parse_attempt("steep mint").unwrap(),
            Attempt::Brew
        ));
        assert!(matches!(
            crate::parse::parse_attempt("brew tea").unwrap(),
            Attempt::Brew
        ));
        assert!(matches!(
            crate::parse::parse_attempt("gather mint").unwrap(),
            Attempt::Gather(Material::Mint)
        ));

        let mut world = World::valley(crate::tuning::Tuning::evening(), 1);
        let bank = cell_id(2, 2).unwrap();
        assert_eq!(world.pile_count(bank, Material::Mint), 1);
        let near = world.spawn_at("Near", "n", 3, 2).unwrap();
        let seen = world.spawn_at("Seen", "s", 3, 2).unwrap();
        let quiet = world.spawn_at("Quiet", "q", 5, 3).unwrap();

        let from_beside = world.observe(near).unwrap().text;
        assert!(
            from_beside.contains("Wild mint grows on the wet bank to the west."),
            "{from_beside}"
        );
        assert!(from_beside.contains("wild mint"), "{from_beside}");
        let miss = world.step(&[intent("miss", near, Attempt::Gather(Material::Mint))]);
        assert_eq!(
            miss.results[0].message,
            "There is no mint here to gather. Step west."
        );
        assert!(world
            .observe(near)
            .unwrap()
            .events
            .iter()
            .any(|event| event.text == "There is no mint here to gather. Step west."));
        assert!(world
            .observe(seen)
            .unwrap()
            .events
            .iter()
            .all(|event| !event.text.contains("mint")));
        assert!(world
            .observe(quiet)
            .unwrap()
            .events
            .iter()
            .all(|event| !event.text.contains("mint")));

        world.step(&[intent("west", near, Attempt::Move(Dir::West))]);
        assert_eq!(world.actor(near).unwrap().cell, bank);
        let here = world.observe(near).unwrap().text;
        assert!(
            here.contains("Wild mint grows on the wet bank here."),
            "{here}"
        );
        let examined = world.examine(near, ExamineTarget::Mint);
        assert!(
            examined.contains("It can be steeped in hot water."),
            "{examined}"
        );
        let gathered = world.step(&[intent("gather", near, Attempt::Gather(Material::Mint))]);
        assert_eq!(gathered.results[0].message, "You gather a handful of mint.");
        let held = world.observe(near).unwrap();
        assert!(
            held.text.contains("The wet bank here has no mint."),
            "{}",
            held.text
        );
        assert!(
            held.text.contains("You have a handful of mint."),
            "{}",
            held.text
        );
        assert!(held
            .events
            .iter()
            .all(|event| !event.text.contains("gathers a handful of mint")));
        assert!(world
            .observe(seen)
            .unwrap()
            .events
            .iter()
            .any(|event| { event.text == "Near gathers a handful of mint." }));
        assert!(world
            .observe(quiet)
            .unwrap()
            .events
            .iter()
            .all(|event| !event.text.contains("mint")));
        let bare = world.observe(seen).unwrap().text;
        assert!(
            bare.contains("The wet bank to the west has no mint."),
            "{bare}"
        );
        assert!(bare.contains("no mint"), "{bare}");

        let no_fire = world.step(&[intent("steep", near, Attempt::Brew)]);
        assert_eq!(no_fire.results[0].message, "There is no fire burning here.");
        assert!(world
            .observe(near)
            .unwrap()
            .events
            .iter()
            .any(|event| { event.text == "There is no fire burning here." }));
        assert!(world
            .observe(seen)
            .unwrap()
            .events
            .iter()
            .all(|event| event.text != "There is no fire burning here."));

        world.mint_fire(bank, 0);
        let no_water = world.step(&[intent("water", near, Attempt::Brew)]);
        assert_eq!(
            no_water.results[0].message,
            "You need a pot or a bark cup to steep the mint."
        );
        assert!(world
            .observe(seen)
            .unwrap()
            .events
            .iter()
            .all(|event| !event.text.contains("steep")));

        world.mint_item(Material::Water, 0, bank, Some(near), Role::Loose);
        world.mint_item(Material::Bowl, 0, bank, Some(near), Role::Loose);
        let bowl = world.step(&[intent("bowl", near, Attempt::Brew)]);
        assert_eq!(
            bowl.results[0].message,
            "You need a pot or a bark cup to steep the mint."
        );
        let bowl_id = world.loose_ids(near, Material::Bowl)[0];
        world.items.remove(&bowl_id);
        world.mint_item(Material::Clay, 0, bank, Some(near), Role::Loose);
        let clay = world.step(&[intent("clay", near, Attempt::Brew)]);
        assert_eq!(
            clay.results[0].message,
            "You need a pot or a bark cup to steep the mint."
        );
        let clay_id = world.loose_ids(near, Material::Clay)[0];
        world.items.remove(&clay_id);

        world.mint_item(Material::Pot, 0, bank, Some(near), Role::Loose);
        let well = world.step(&[intent("well", near, Attempt::Brew)]);
        assert_eq!(
            well.results[0].message,
            "You steep a handful of mint in the pot over the fire and drink the mint tea."
        );
        assert!(!well.results[0].message.contains("stomach"));
        let after_well = world.observe(near).unwrap();
        assert!(after_well
            .events
            .iter()
            .all(|event| !event.text.contains("steeps a handful of mint")));
        assert!(!after_well.text.contains("a handful of mint"));
        assert!(world.observe(seen).unwrap().events.iter().any(|event| {
            event.text
                == "Near steeps a handful of mint in the pot over the fire and drinks the mint tea."
        }));
        assert!(world
            .observe(quiet)
            .unwrap()
            .events
            .iter()
            .all(|event| !event.text.contains("steep")));

        world.mint_item(Material::Mint, 0, bank, Some(near), Role::Loose);
        world.mint_item(Material::Water, 0, bank, Some(near), Role::Loose);
        world.mint_item(Material::BitterBerry, 0, bank, Some(near), Role::Loose);
        let eaten = world.step(&[intent("eat", near, Attempt::Eat(Material::BitterBerry))]);
        assert!(eaten.results[0].message.contains("Your stomach turns."));
        assert!(world.actor(near).unwrap().illness >= world.tuning.ill_at);
        let steeped = world.step(&[intent("tea", near, Attempt::Brew)]);
        assert_eq!(
            steeped.results[0].message,
            "You steep a handful of mint in the pot over the fire and drink the mint tea. Your stomach eases."
        );
        assert!(world.actor(near).unwrap().illness < world.tuning.ill_at);
        let eased = world.observe(near).unwrap();
        assert!(eased
            .events
            .iter()
            .any(|event| event.text == "Your stomach settles."));
        assert!(eased
            .events
            .iter()
            .all(|event| !event.text.contains("steeps a handful of mint")));
        let seen_tea = world.observe(seen).unwrap();
        assert!(seen_tea.events.iter().any(|event| {
            event.text
                == "Near steeps a handful of mint in the pot over the fire and drinks the mint tea."
        }));
        assert!(seen_tea
            .events
            .iter()
            .all(|event| !event.text.contains("stomach")));
        assert!(world
            .observe(quiet)
            .unwrap()
            .events
            .iter()
            .all(|event| !event.text.contains("stomach") && !event.text.contains("steep")));

        let gone = world.step(&[intent("again", near, Attempt::Brew)]);
        assert_eq!(gone.results[0].message, "You have no mint to steep.");
        assert!(world
            .observe(seen)
            .unwrap()
            .events
            .iter()
            .all(|event| event.text != "You have no mint to steep."));

        let pot_id = world.loose_ids(near, Material::Pot)[0];
        world.items.remove(&pot_id);
        let cup = world.mint_item(Material::BarkCup, 0, bank, Some(near), Role::Loose);
        world.mint_item(Material::Mint, 0, bank, Some(near), Role::Loose);
        world.mint_item(Material::Water, 0, bank, Some(near), Role::Loose);
        let in_cup = world.step(&[intent("cup", near, Attempt::Brew)]);
        assert_eq!(
            in_cup.results[0].message,
            "You steep a handful of mint in the bark cup over the fire and drink the mint tea."
        );
        assert!(world.observe(seen).unwrap().events.iter().any(|event| {
            event.text
                == "Near steeps a handful of mint in the bark cup over the fire and drinks the mint tea."
        }));
        world.items.get_mut(&cup).unwrap().pitched = true;
        world.mint_item(Material::Mint, 0, bank, Some(near), Role::Loose);
        world.mint_item(Material::Water, 0, bank, Some(near), Role::Loose);
        let sealed = world.step(&[intent("sealed", near, Attempt::Brew)]);
        assert_eq!(
            sealed.results[0].message,
            "You steep a handful of mint in the sealed bark cup over the fire and drink the mint tea."
        );

        let at = world.mint_returns_at[&bank];
        while world.time < at {
            world.step(&[]);
        }
        assert!(world
            .observe(near)
            .unwrap()
            .text
            .contains("The wet bank here has no mint."));
        world.step(&[]);
        let back = world.observe(near).unwrap();
        assert!(
            back.text.contains("Wild mint grows on the wet bank here."),
            "{}",
            back.text
        );
        assert!(back
            .events
            .iter()
            .any(|event| event.text == "Wild mint grows on the wet bank again."));
        assert!(world
            .observe(seen)
            .unwrap()
            .text
            .contains("Wild mint grows on the wet bank to the west."));
    }
}

#[cfg(test)]
mod pine_tea {
    use super::{cell_id, Attempt, Dir, ExamineTarget, Intent, Material, Role, World};

    fn intent(id: &str, actor: u32, attempt: Attempt) -> Intent {
        Intent {
            command_id: id.into(),
            actor_id: actor,
            attempt,
        }
    }

    #[test]
    fn pine_needles_steep_into_a_tea_that_eases_a_chill() {
        assert!(matches!(
            crate::parse::parse_attempt("steep pine").unwrap(),
            Attempt::SteepPine
        ));
        assert!(matches!(
            crate::parse::parse_attempt("brew pine needles").unwrap(),
            Attempt::SteepPine
        ));
        assert!(matches!(
            crate::parse::parse_attempt("brew pine needle tea").unwrap(),
            Attempt::SteepPine
        ));
        assert!(matches!(
            crate::parse::parse_attempt("steep mint").unwrap(),
            Attempt::Brew
        ));
        assert!(matches!(
            crate::parse::parse_attempt("gather pine needles").unwrap(),
            Attempt::Gather(Material::Needle)
        ));
        assert!(matches!(
            crate::parse::parse_attempt("gather resin").unwrap(),
            Attempt::Gather(Material::Resin)
        ));

        let mut world = World::valley(crate::tuning::Tuning::evening(), 1);
        let pine = cell_id(2, 4).unwrap();
        assert_eq!(world.pile_count(pine, Material::Needle), 1);
        assert_eq!(world.pile_count(pine, Material::Resin), 2);
        let near = world.spawn_at("Near", "n", 1, 4).unwrap();
        let seen = world.spawn_at("Seen", "s", 3, 4).unwrap();
        let quiet = world.spawn_at("Quiet", "q", 6, 0).unwrap();

        let beside = world.observe(near).unwrap().text;
        assert!(
            beside.contains("Pine needles lie under the pine to the east."),
            "{beside}"
        );
        assert!(beside.contains("pine needles"), "{beside}");
        assert!(
            beside.contains("pine resin") || beside.contains("pitch"),
            "{beside}"
        );
        let miss = world.step(&[intent("miss", near, Attempt::Gather(Material::Needle))]);
        assert_eq!(
            miss.results[0].message,
            "There are no pine needles here to gather. Step east."
        );
        assert!(world
            .observe(seen)
            .unwrap()
            .events
            .iter()
            .all(|event| { event.text != "There are no pine needles here to gather. Step east." }));
        assert!(world
            .observe(quiet)
            .unwrap()
            .events
            .iter()
            .all(|event| { !event.text.contains("pine needles") }));

        world.step(&[intent("east", near, Attempt::Move(Dir::East))]);
        assert_eq!(world.actor(near).unwrap().cell, pine);
        let here = world.observe(near).unwrap().text;
        assert!(
            here.contains("Pine needles lie under the pine here."),
            "{here}"
        );
        assert!(
            here.contains("It is pine resin.") || here.contains("pitch"),
            "{here}"
        );
        let examined = world.examine(near, ExamineTarget::Needle);
        assert!(
            examined.contains("They can be steeped in hot water."),
            "{examined}"
        );
        assert!(!examined.contains("mint"), "{examined}");
        let gathered = world.step(&[intent("gather", near, Attempt::Gather(Material::Needle))]);
        assert_eq!(
            gathered.results[0].message,
            "You gather a handful of pine needles."
        );
        let held = world.observe(near).unwrap();
        assert!(
            held.text.contains("The pine here has no needles."),
            "{}",
            held.text
        );
        assert!(
            held.text.contains("You have a handful of pine needles."),
            "{}",
            held.text
        );
        assert!(
            held.text.contains("pine resin") || held.text.contains("pitch"),
            "{}",
            held.text
        );
        assert!(held
            .events
            .iter()
            .all(|event| { !event.text.contains("gathers a handful of pine needles") }));
        assert!(world
            .observe(seen)
            .unwrap()
            .events
            .iter()
            .any(|event| { event.text == "Near gathers a handful of pine needles." }));
        assert!(world
            .observe(quiet)
            .unwrap()
            .events
            .iter()
            .all(|event| { !event.text.contains("pine needles") }));
        let bare = world.observe(seen).unwrap().text;
        assert!(
            bare.contains("The pine to the west has no needles."),
            "{bare}"
        );
        assert!(bare.contains("no needles"), "{bare}");

        let no_fire = world.step(&[intent("steep", near, Attempt::SteepPine)]);
        assert_eq!(no_fire.results[0].message, "There is no fire burning here.");
        assert!(world
            .observe(seen)
            .unwrap()
            .events
            .iter()
            .all(|event| { event.text != "There is no fire burning here." }));

        world.mint_fire(pine, 0);
        let no_water = world.step(&[intent("water", near, Attempt::SteepPine)]);
        assert_eq!(
            no_water.results[0].message,
            "You need a pot or a bark cup to steep the pine needles."
        );
        world.mint_item(Material::Water, 0, pine, Some(near), Role::Loose);
        world.mint_item(Material::Bowl, 0, pine, Some(near), Role::Loose);
        let bowl = world.step(&[intent("bowl", near, Attempt::SteepPine)]);
        assert_eq!(
            bowl.results[0].message,
            "You need a pot or a bark cup to steep the pine needles."
        );
        let bowl_id = world.loose_ids(near, Material::Bowl)[0];
        world.items.remove(&bowl_id);
        world.mint_item(Material::Pot, 0, pine, Some(near), Role::Loose);
        let not_mint = world.step(&[intent("mint", near, Attempt::Brew)]);
        assert_eq!(not_mint.results[0].message, "You have no mint to steep.");
        assert_eq!(world.loose_ids(near, Material::Needle).len(), 1);
        world.actors.get_mut(&near).unwrap().cold = 0;
        world.actors.get_mut(&near).unwrap().illness = 800;
        let well = world.step(&[intent("warm", near, Attempt::SteepPine)]);
        assert_eq!(
            well.results[0].message,
            "You steep a handful of pine needles in the pot over the fire and drink the pine needle tea."
        );
        assert!(!well.results[0].message.contains("chill"));
        assert!(!well.results[0].message.contains("stomach"));
        assert!(world.actor(near).unwrap().illness >= world.tuning.ill_at);

        world.mint_item(Material::Needle, 0, pine, Some(near), Role::Loose);
        world.mint_item(Material::Water, 0, pine, Some(near), Role::Loose);
        let ease = world.tuning.cold_per_tick.saturating_mul(2);
        let chilly = world
            .tuning
            .chilly_at
            .saturating_add(ease)
            .saturating_add(world.tuning.fire_warmth)
            .saturating_add(40);
        world.actors.get_mut(&near).unwrap().cold = chilly;
        world.actors.get_mut(&near).unwrap().illness = 800;
        let steeped = world.step(&[intent("chill", near, Attempt::SteepPine)]);
        assert_eq!(
            steeped.results[0].message,
            "You steep a handful of pine needles in the pot over the fire and drink the pine needle tea. The chill eases."
        );
        assert!(!steeped.results[0].message.contains("stomach"));
        assert!(!steeped.results[0].message.contains("mint tea"));
        let after = world.actor(near).unwrap();
        assert_eq!(after.cold, chilly - ease - world.tuning.fire_warmth);
        assert!(after.illness >= world.tuning.ill_at);
        let eased = world.observe(near).unwrap();
        assert!(eased.events.iter().all(|event| {
            !event.text.contains("steeps a handful of pine needles")
                && !event.text.contains("The chill eases.")
        }));
        let seen_tea = world.observe(seen).unwrap();
        assert!(seen_tea.events.iter().any(|event| {
            event.text == "Near steeps a handful of pine needles in the pot over the fire and drinks the pine needle tea."
        }));
        assert!(seen_tea.events.iter().all(|event| {
            !event.text.contains("The chill eases.") && !event.text.contains("stomach")
        }));
        assert!(world.observe(quiet).unwrap().events.iter().all(|event| {
            !event.text.contains("pine needle") && !event.text.contains("The chill eases.")
        }));

        let gone = world.step(&[intent("again", near, Attempt::SteepPine)]);
        assert_eq!(
            gone.results[0].message,
            "You have no pine needles to steep."
        );
        assert!(world
            .observe(seen)
            .unwrap()
            .events
            .iter()
            .all(|event| { event.text != "You have no pine needles to steep." }));

        let pot_id = world.loose_ids(near, Material::Pot)[0];
        world.items.remove(&pot_id);
        let cup = world.mint_item(Material::BarkCup, 0, pine, Some(near), Role::Loose);
        world.mint_item(Material::Needle, 0, pine, Some(near), Role::Loose);
        world.mint_item(Material::Water, 0, pine, Some(near), Role::Loose);
        world.actors.get_mut(&near).unwrap().cold = 0;
        let in_cup = world.step(&[intent("cup", near, Attempt::SteepPine)]);
        assert_eq!(
            in_cup.results[0].message,
            "You steep a handful of pine needles in the bark cup over the fire and drink the pine needle tea."
        );
        world.items.get_mut(&cup).unwrap().pitched = true;
        world.mint_item(Material::Needle, 0, pine, Some(near), Role::Loose);
        world.mint_item(Material::Water, 0, pine, Some(near), Role::Loose);
        let sealed = world.step(&[intent("sealed", near, Attempt::SteepPine)]);
        assert_eq!(
            sealed.results[0].message,
            "You steep a handful of pine needles in the sealed bark cup over the fire and drink the pine needle tea."
        );

        let at = world.needle_returns_at[&pine];
        while world.time < at {
            world.step(&[]);
        }
        assert!(world
            .observe(near)
            .unwrap()
            .text
            .contains("The pine here has no needles."));
        world.step(&[]);
        let back = world.observe(near).unwrap();
        assert!(
            back.text.contains("Pine needles lie under the pine here."),
            "{}",
            back.text
        );
        assert!(
            back.text.contains("pine resin") || back.text.contains("pitch"),
            "{}",
            back.text
        );
        assert!(back
            .events
            .iter()
            .any(|event| { event.text == "Pine needles lie under the pine again." }));
    }
}

#[cfg(test)]
mod bow_drill {
    use super::{cell_id, Attempt, Intent, Material, Role, World};

    fn intent(id: &str, actor: u32, attempt: Attempt) -> Intent {
        Intent {
            command_id: id.into(),
            actor_id: actor,
            attempt,
        }
    }

    fn says(world: &World, actor: u32, line: &str) -> bool {
        world
            .observe(actor)
            .unwrap()
            .events
            .iter()
            .any(|event| event.text == line)
    }

    #[test]
    fn a_bow_drill_lights_a_fire_without_tinder() {
        assert!(matches!(
            crate::parse::parse_attempt("carve drill").unwrap(),
            Attempt::CarveDrill
        ));
        assert!(matches!(
            crate::parse::parse_attempt("make bow drill").unwrap(),
            Attempt::CarveDrill
        ));
        assert!(matches!(
            crate::parse::parse_attempt("light fire with drill").unwrap(),
            Attempt::DrillFire
        ));
        assert!(matches!(
            crate::parse::parse_attempt("drill fire").unwrap(),
            Attempt::DrillFire
        ));
        assert!(matches!(
            crate::parse::parse_attempt("light").unwrap(),
            Attempt::Light
        ));
        assert!(matches!(
            crate::parse::parse_attempt("light fire").unwrap(),
            Attempt::Light
        ));
        assert!(matches!(
            crate::parse::parse_attempt("craft bow").unwrap(),
            Attempt::CraftBow
        ));

        let mut world = World::valley(crate::tuning::Tuning::evening(), 1);
        let open = cell_id(4, 3).unwrap();
        let rock = cell_id(6, 1).unwrap();
        let bark_before = world.pile_count(rock, Material::Bark);
        let branches_before = world.pile_count(rock, Material::Branch);
        assert!(bark_before >= 2);
        assert!(branches_before >= 2);
        let near = world.spawn_at("Near", "n", 4, 3).unwrap();
        let seen = world.spawn_at("Seen", "s", 3, 3).unwrap();
        let quiet = world.spawn_at("Quiet", "q", 5, 3).unwrap();

        let missing = world.step(&[intent("missing", near, Attempt::CarveDrill)]);
        assert_eq!(
            missing.results[0].message,
            "You need two dry branches and hide strips or willow cordage."
        );
        assert!(says(
            &world,
            near,
            "You need two dry branches and hide strips or willow cordage."
        ));
        assert!(!says(
            &world,
            seen,
            "You need two dry branches and hide strips or willow cordage."
        ));
        assert!(!says(
            &world,
            quiet,
            "You need two dry branches and hide strips or willow cordage."
        ));

        let no_drill = world.step(&[intent("nodrill", near, Attempt::DrillFire)]);
        assert_eq!(no_drill.results[0].message, "You have no bow drill.");
        assert!(says(&world, near, "You have no bow drill."));
        assert!(!says(&world, quiet, "You have no bow drill."));
        assert!(!says(&world, seen, "You have no bow drill."));

        world.mint_item(Material::Branch, 0, open, Some(near), Role::Loose);
        world.mint_item(Material::Willow, 0, open, Some(near), Role::Loose);
        let one = world.step(&[intent("one", near, Attempt::CarveDrill)]);
        assert_eq!(
            one.results[0].message,
            "You need two dry branches, one for the spindle and one for the fireboard."
        );
        assert!(says(
            &world,
            near,
            "You need two dry branches, one for the spindle and one for the fireboard."
        ));
        assert!(!says(
            &world,
            quiet,
            "You need two dry branches, one for the spindle and one for the fireboard."
        ));

        world.mint_item(Material::Branch, 0, open, Some(near), Role::Loose);
        let carved = world.step(&[intent("carve", near, Attempt::CarveDrill)]);
        let yours = "You carve a bow drill from two dry branches and willow cordage.";
        let theirs = "Near carves a bow drill from two dry branches and willow cordage.";
        assert_eq!(carved.results[0].message, yours);
        assert!(!says(&world, near, theirs));
        assert!(says(&world, seen, theirs));
        assert!(says(&world, quiet, theirs));
        assert_eq!(world.loose_ids(near, Material::Drill).len(), 1);
        let arms = world.observe(near).unwrap().text;
        assert!(arms.contains("a bow drill"), "{arms}");

        let still_bark = world.step(&[intent("bark", near, Attempt::Light)]);
        assert_eq!(
            still_bark.results[0].message,
            "You need dry loose bark for tinder."
        );
        assert!(!says(&world, seen, "You need dry loose bark for tinder."));
        assert_eq!(world.loose_ids(near, Material::Drill).len(), 1);
        assert!(world.fire_at(open).is_none());

        let drilled = world.step(&[intent("drill", near, Attempt::DrillFire)]);
        let lit = "You light a fire with the bow drill. The bow drill is spent.";
        let heard = "Near lights a fire with the bow drill. The bow drill is spent.";
        assert_eq!(drilled.results[0].message, lit);
        assert!(!says(&world, near, heard));
        assert!(says(&world, seen, heard));
        assert!(says(&world, quiet, heard));
        assert!(world.fire_at(open).is_some());
        assert_eq!(world.loose_ids(near, Material::Drill).len(), 1);
        assert!(world
            .item(world.loose_ids(near, Material::Drill)[0])
            .is_some_and(|drill| drill.spent));
        let spent_arms = world.observe(near).unwrap().text;
        assert!(spent_arms.contains("a spent bow drill"), "{spent_arms}");
        assert!(world.loose_ids(near, Material::Charcoal).is_empty());
        assert_eq!(world.pile_count(rock, Material::Bark), bark_before);

        let need_branch = world.step(&[intent("anew", near, Attempt::CarveDrill)]);
        assert_eq!(
            need_branch.results[0].message,
            "You need a dry branch to carve the bow drill anew."
        );
        assert!(says(
            &world,
            near,
            "You need a dry branch to carve the bow drill anew."
        ));
        assert!(!says(
            &world,
            seen,
            "You need a dry branch to carve the bow drill anew."
        ));

        world.mint_item(Material::Branch, 0, open, Some(near), Role::Loose);
        let anew = world.step(&[intent("restore", near, Attempt::CarveDrill)]);
        let restored = "You carve the spent bow drill anew with one dry branch.";
        let restored_heard = "Near carves the spent bow drill anew with one dry branch.";
        assert_eq!(anew.results[0].message, restored);
        assert!(!says(&world, near, restored_heard));
        assert!(says(&world, seen, restored_heard));
        assert!(says(&world, quiet, restored_heard));
        assert!(world
            .item(world.loose_ids(near, Material::Drill)[0])
            .is_some_and(|drill| !drill.spent));
        assert!(world.loose_ids(near, Material::Branch).is_empty());

        let busy_fire = world.step(&[intent("busy", near, Attempt::DrillFire)]);
        assert_eq!(busy_fire.results[0].message, "A fire already burns here.");
        assert!(says(&world, near, "A fire already burns here."));
        assert!(!says(&world, quiet, "A fire already burns here."));

        let hearth = world.spawn_at("Hearth", "h", 6, 1).unwrap();
        world.mint_item(Material::Cord, 0, rock, Some(hearth), Role::Loose);
        let rock_branches = world.pile_count(rock, Material::Branch);
        assert!(rock_branches >= 2, "{rock_branches}");
        let at_rock = world.step(&[intent("rock", hearth, Attempt::CarveDrill)]);
        assert_eq!(
            at_rock.results[0].message,
            "You carve a bow drill from two dry branches and hide strips."
        );
        assert_eq!(world.pile_count(rock, Material::Branch), rock_branches - 2);
        assert_eq!(world.pile_count(rock, Material::Bark), bark_before);
        assert!(says(
            &world,
            quiet,
            "Hearth carves a bow drill from two dry branches and hide strips."
        ));
        assert!(!says(
            &world,
            seen,
            "Hearth carves a bow drill from two dry branches and hide strips."
        ));
        assert!(!says(
            &world,
            hearth,
            "Hearth carves a bow drill from two dry branches and hide strips."
        ));

        let hearth_lit = world.step(&[intent("hearth", hearth, Attempt::DrillFire)]);
        assert_eq!(
            hearth_lit.results[0].message,
            "You light a fire with the bow drill. The bow drill is spent."
        );
        assert!(says(
            &world,
            quiet,
            "Hearth lights a fire with the bow drill. The bow drill is spent."
        ));
        assert!(!says(
            &world,
            hearth,
            "Hearth lights a fire with the bow drill. The bow drill is spent."
        ));
        assert!(!says(
            &world,
            seen,
            "Hearth lights a fire with the bow drill. The bow drill is spent."
        ));
        for _ in 0..20 {
            if world.fire_at(rock).is_none() {
                break;
            }
            world.step(&[]);
        }
        assert!(world.fire_at(rock).is_none());
        let spent_fire = world.step(&[intent("spent", hearth, Attempt::DrillFire)]);
        assert_eq!(spent_fire.results[0].message, "The bow drill is spent.");
        assert!(says(&world, hearth, "The bow drill is spent."));
        assert!(!says(&world, quiet, "The bow drill is spent."));
        assert!(!says(&world, seen, "The bow drill is spent."));

        let open = cell_id(4, 3).unwrap();
        let wet = world.spawn_at("Wet", "w", 4, 3).unwrap();
        world.mint_item(Material::Drill, 0, open, Some(wet), Role::Loose);
        world.raining = true;
        let rain = world.step(&[intent("rain", wet, Attempt::DrillFire)]);
        assert_eq!(
            rain.results[0].message,
            "The rain puts the drill out before it catches."
        );
        assert!(says(
            &world,
            wet,
            "The rain puts the drill out before it catches."
        ));
        assert!(!says(
            &world,
            near,
            "The rain puts the drill out before it catches."
        ));
        assert_eq!(world.loose_ids(wet, Material::Drill).len(), 1);
    }
}

#[cfg(test)]
mod signal {
    use super::*;

    fn intent(id: &str, actor: ActorId, attempt: Attempt) -> Intent {
        Intent {
            command_id: id.into(),
            actor_id: actor,
            attempt,
        }
    }

    fn says(world: &World, actor: ActorId, text: &str) -> bool {
        world
            .observe(actor)
            .unwrap()
            .events
            .iter()
            .any(|event| event.text == text)
    }

    #[test]
    fn a_smoke_signal_is_a_column_past_the_ordinary_smoke() {
        assert!(matches!(
            crate::parse::parse_attempt("signal").unwrap(),
            Attempt::Signal(None)
        ));
        assert!(matches!(
            crate::parse::parse_attempt("smoke signal").unwrap(),
            Attempt::Signal(None)
        ));
        assert!(matches!(
            crate::parse::parse_attempt("signal with reeds").unwrap(),
            Attempt::Signal(Some(Material::Reed))
        ));
        assert!(matches!(
            crate::parse::parse_attempt("smoke signal with green leaves").unwrap(),
            Attempt::Signal(Some(Material::Needle))
        ));

        let mut world = World::valley(crate::tuning::Tuning::evening(), 1);
        let rock = cell_id(6, 1).unwrap();
        let near = world.spawn_at("Near", "n", 6, 1).unwrap();
        let west = world.spawn_at("West", "w", 4, 1).unwrap();
        let seen = world.spawn_at("Seen", "s", 3, 3).unwrap();
        let quiet = world.spawn_at("Quiet", "q", 2, 1).unwrap();
        let far = world.spawn_at("Far", "f", 1, 1).unwrap();
        let past = world.spawn_at("Past", "p", 0, 1).unwrap();
        let missed = world.step(&[intent("none", near, Attempt::Signal(None))]);
        assert_eq!(missed.results[0].message, "There is no fire burning here.");
        assert!(says(&world, near, "There is no fire burning here."));
        for person in [west, seen, quiet, far, past] {
            assert!(!says(&world, person, "There is no fire burning here."));
        }
        let lit = world.step(&[intent("light", near, Attempt::Light)]);
        assert!(lit.results[0].success, "{}", lit.results[0].message);
        let before = world.observe(near).unwrap().text;
        assert!(before.contains("A small fire burns here. It warms you."));
        assert!(!before.contains("A column of smoke rises from the fire."));
        assert!(!world.observe(quiet).unwrap().text.contains("You see smoke"));
        assert!(world
            .observe(seen)
            .unwrap()
            .text
            .contains("Smoke rises to the north and east."));
        let bare = world.step(&[intent("signal", near, Attempt::Signal(None))]);
        assert_eq!(bare.results[0].message, "You send up a column of smoke.");
        let reply = "You send up a column of smoke.";
        for person in [near, west, seen, quiet, far, past] {
            assert!(!says(&world, person, reply));
            assert!(world
                .observe(person)
                .unwrap()
                .events
                .iter()
                .all(|event| !event.text.contains("sends up")));
        }
        let here = world.observe(near).unwrap().text;
        assert!(
            here.contains("A column of smoke rises from the fire."),
            "{here}"
        );
        assert!(here.contains("A small fire burns here. It warms you."));
        let watching = world.observe(west).unwrap().text;
        assert!(
            watching.contains("A column of smoke rises from the fire."),
            "{watching}"
        );
        assert!(!watching.contains("You see smoke"), "{watching}");
        let plume = world.observe(seen).unwrap().text;
        assert!(
            plume.contains("Smoke rises to the north and east."),
            "{plume}"
        );
        assert!(!plume.contains("You see smoke"), "{plume}");
        assert!(!plume.contains("You smell smoke"), "{plume}");
        let see = world.observe(quiet).unwrap().text;
        assert!(see.contains("You see smoke to the east."), "{see}");
        assert!(!see.contains("You smell smoke"), "{see}");
        let smell = world.observe(far).unwrap().text;
        assert!(smell.contains("You smell smoke to the east."), "{smell}");
        assert!(!smell.contains("You see smoke"), "{smell}");
        let none = world.observe(past).unwrap().text;
        assert!(!none.contains("You see smoke"), "{none}");
        assert!(!none.contains("You smell smoke"), "{none}");
        for person in [quiet, far, past, seen, west] {
            assert!(world.observe(person).unwrap().events.iter().all(|event| {
                event.text != "You see smoke to the east."
                    && event.text != "You smell smoke to the east."
            }));
        }
        let again = world.step(&[intent("again", near, Attempt::Signal(None))]);
        assert_eq!(
            again.results[0].message,
            "A column of smoke is already rising."
        );
        assert!(says(&world, near, "A column of smoke is already rising."));
        assert!(!says(&world, seen, "A column of smoke is already rising."));
        while world.fire_at(rock).is_some() {
            world.step(&[]);
        }
        assert!(!world.observe(quiet).unwrap().text.contains("You see smoke"));
        assert!(!world
            .observe(near)
            .unwrap()
            .text
            .contains("A column of smoke rises from the fire."));

        let mut green = World::valley(crate::tuning::Tuning::evening(), 1);
        let near = green.spawn_at("Near", "n", 6, 1).unwrap();
        let quiet = green.spawn_at("Quiet", "q", 2, 1).unwrap();
        green.step(&[intent("light", near, Attempt::Light)]);
        let no_reeds = green.step(&[intent("reeds", near, Attempt::Signal(Some(Material::Reed)))]);
        assert_eq!(
            no_reeds.results[0].message,
            "You have no green reeds to lay on the fire."
        );
        assert!(says(
            &green,
            near,
            "You have no green reeds to lay on the fire."
        ));
        assert!(!says(
            &green,
            quiet,
            "You have no green reeds to lay on the fire."
        ));
        assert!(!green.observe(quiet).unwrap().text.contains("You see smoke"));
        green.add_pile(cell_id(6, 1).unwrap(), Material::Reed, 1, 0);
        let laid = green.step(&[intent("lay", near, Attempt::Signal(Some(Material::Reed)))]);
        assert_eq!(
            laid.results[0].message,
            "You lay green reeds on the fire. A column of smoke rises."
        );
        assert_eq!(green.pile_count(cell_id(6, 1).unwrap(), Material::Reed), 0);
        assert!(!says(
            &green,
            near,
            "You lay green reeds on the fire. A column of smoke rises."
        ));
        assert!(green
            .observe(quiet)
            .unwrap()
            .text
            .contains("You see smoke to the east."));

        let mut leaves = World::valley(crate::tuning::Tuning::evening(), 1);
        let near = leaves.spawn_at("Near", "n", 6, 1).unwrap();
        let far = leaves.spawn_at("Far", "f", 1, 1).unwrap();
        leaves.step(&[intent("light", near, Attempt::Light)]);
        leaves.add_pile(cell_id(6, 1).unwrap(), Material::Needle, 1, 0);
        let laid = leaves.step(&[intent(
            "leaves",
            near,
            Attempt::Signal(Some(Material::Needle)),
        )]);
        assert_eq!(
            laid.results[0].message,
            "You lay green pine needles on the fire. A column of smoke rises."
        );
        assert!(leaves
            .observe(far)
            .unwrap()
            .text
            .contains("You smell smoke to the east."));
        assert!(!says(
            &leaves,
            far,
            "You lay green pine needles on the fire. A column of smoke rises."
        ));
    }
}

#[cfg(test)]
mod worms {
    use super::*;

    fn intent(id: &str, actor: ActorId, attempt: Attempt) -> Intent {
        Intent {
            command_id: id.into(),
            actor_id: actor,
            attempt,
        }
    }

    fn says(world: &World, actor: ActorId, text: &str) -> bool {
        world
            .observe(actor)
            .unwrap()
            .events
            .iter()
            .any(|event| event.text == text)
    }

    #[test]
    fn worms_from_damp_earth_let_the_hook_fish_again_sooner() {
        assert!(matches!(
            crate::parse::parse_attempt("dig worms").unwrap(),
            Attempt::Gather(Material::Worm)
        ));
        assert!(matches!(
            crate::parse::parse_attempt("gather worms").unwrap(),
            Attempt::Gather(Material::Worm)
        ));

        let mut world = World::valley(crate::tuning::Tuning::evening(), 1);
        let bank = cell_id(2, 2).unwrap();
        let near = world.spawn_at("Near", "n", 4, 3).unwrap();
        let seen = world.spawn_at("Seen", "s", 3, 3).unwrap();
        let quiet = world.spawn_at("Quiet", "q", 5, 3).unwrap();
        let missed = world.step(&[intent("none", near, Attempt::Gather(Material::Worm))]);
        assert_eq!(missed.results[0].message, "There are no worms here to dig.");
        assert!(says(&world, near, "There are no worms here to dig."));
        assert!(!says(&world, seen, "There are no worms here to dig."));
        assert!(!says(&world, quiet, "There are no worms here to dig."));
        let far = world.observe(near).unwrap().text;
        assert!(
            far.contains("Worms are in the damp earth to the north and west."),
            "{far}"
        );
        assert!(world
            .observe(near)
            .unwrap()
            .events
            .iter()
            .all(|event| !event.text.contains("damp earth")));

        world.step(&[intent("n", near, Attempt::Move(Dir::North))]);
        world.step(&[intent("w", near, Attempt::Move(Dir::West))]);
        world.step(&[intent("w2", near, Attempt::Move(Dir::West))]);
        assert_eq!(world.actor_cell(near), Some((2, 2)));
        let here = world.observe(near).unwrap().text;
        assert!(here.contains("Worms are in the damp earth here."), "{here}");
        let dug = world.step(&[intent("dig", near, Attempt::Gather(Material::Worm))]);
        assert_eq!(dug.results[0].message, "You gather a worm.");
        assert!(world
            .observe(near)
            .unwrap()
            .text
            .contains("You have a worm."));
        assert!(world
            .observe(near)
            .unwrap()
            .events
            .iter()
            .all(|event| event.text != "Near gathers a worm."));
        assert!(world
            .observe(seen)
            .unwrap()
            .events
            .iter()
            .any(|event| event.text == "Near gathers a worm."));
        assert!(world
            .observe(quiet)
            .unwrap()
            .events
            .iter()
            .all(|event| event.text != "Near gathers a worm."));
        let bare = world.observe(near).unwrap().text;
        assert!(bare.contains("The damp earth here has no worms."), "{bare}");
        let again = world.step(&[intent("again", near, Attempt::Gather(Material::Worm))]);
        assert_eq!(
            again.results[0].message,
            "The damp earth here has no worms."
        );
        assert!(says(&world, near, "The damp earth here has no worms."));
        assert!(!says(&world, seen, "The damp earth here has no worms."));

        world.mint_item(Material::Hook, 0, bank, Some(near), Role::Loose);
        world.mint_item(Material::Cord, 0, bank, Some(near), Role::Loose);
        world.mint_item(Material::Cord, 0, bank, Some(near), Role::Loose);
        world.step(&[intent("drop", near, Attempt::Drop(Material::Worm))]);
        let caught = world.step(&[intent("bare", near, Attempt::FishHook)]);
        assert_eq!(
            caught.results[0].message,
            "You catch a fish from the brook with the bone fish hook. The hide strips are spent."
        );
        assert_eq!(world.loose_ids(near, Material::Worm).len(), 0);
        let gone = world.step(&[intent("soon", near, Attempt::FishHook)]);
        assert_eq!(
            gone.results[0].message,
            "The fish are gone from this stretch of the brook."
        );
        assert!(!says(
            &world,
            near,
            "The fish are gone from this stretch of the brook."
        ));

        let picked = world.step(&[intent("pick", near, Attempt::Gather(Material::Worm))]);
        assert_eq!(picked.results[0].message, "You gather a worm.");
        world.step(&[intent("west", near, Attempt::Move(Dir::West))]);
        world.step(&[intent("brook", near, Attempt::Move(Dir::West))]);
        assert_eq!(world.actor_cell(near), Some((0, 2)));
        let wormed = world.step(&[intent("worm", near, Attempt::FishHook)]);
        assert_eq!(
            wormed.results[0].message,
            "You catch a fish from the brook with the bone fish hook and a worm. The hide strips are spent."
        );
        assert_eq!(world.loose_ids(near, Material::Worm).len(), 0);
        assert!(world.observe(near).unwrap().events.iter().all(|event| {
            !event.text.contains("You catch") && !event.text.contains("Near catches")
        }));
        assert!(world
            .observe(seen)
            .unwrap()
            .events
            .iter()
            .any(|event| { event.text == "You hear a splash to the north and west." }));
        assert!(world
            .observe(seen)
            .unwrap()
            .events
            .iter()
            .all(|event| { !event.text.contains("and a worm") }));
        assert!(world.observe(quiet).unwrap().events.iter().all(|event| {
            !event.text.contains("and a worm") && event.text != "Near gathers a worm."
        }));
        world.mint_item(
            Material::Cord,
            0,
            cell_id(0, 2).unwrap(),
            Some(near),
            Role::Loose,
        );
        let next = world.step(&[intent("next", near, Attempt::FishHook)]);
        assert_eq!(
            next.results[0].message,
            "You catch a fish from the brook with the bone fish hook. The hide strips are spent."
        );

        let mud = cell_id(4, 4).unwrap();
        world.spawn_at("Mud", "m", 4, 4).unwrap();
        let mud_id = world
            .actors
            .values()
            .find(|actor| actor.name == "Mud")
            .unwrap()
            .id;
        world.add_pile(mud, Material::Mud, 1, 0);
        let empty = world.step(&[intent("mud", mud_id, Attempt::Gather(Material::Worm))]);
        assert_eq!(
            empty.results[0].message,
            "The damp earth here has no worms."
        );
        assert!(world
            .observe(mud_id)
            .unwrap()
            .text
            .contains("The damp earth here has no worms."));
    }

    #[test]
    fn dug_worms_return_after_twenty_four_ticks() {
        let mut world = World::valley(crate::tuning::Tuning::evening(), 1);
        let bank = cell_id(2, 2).unwrap();
        let near = world.spawn_at("Near", "n", 2, 2).unwrap();
        let seen = world.spawn_at("Seen", "s", 3, 3).unwrap();
        let quiet = world.spawn_at("Quiet", "q", 5, 3).unwrap();
        assert_eq!(world.tuning.moss_ticks, 24);
        let dug = world.step(&[intent("dig", near, Attempt::Gather(Material::Worm))]);
        assert_eq!(dug.results[0].message, "You gather a worm.");
        assert!(world
            .observe(near)
            .unwrap()
            .text
            .contains("The damp earth here has no worms."));
        let missed = world.step(&[intent("none", near, Attempt::Gather(Material::Worm))]);
        assert_eq!(
            missed.results[0].message,
            "The damp earth here has no worms."
        );
        assert!(says(&world, near, "The damp earth here has no worms."));
        assert!(!says(&world, seen, "The damp earth here has no worms."));
        let ticks = world.tuning.moss_ticks;
        for _ in 0..ticks - 2 {
            world.step(&[]);
        }
        assert!(world
            .observe(near)
            .unwrap()
            .text
            .contains("The damp earth here has no worms."));
        assert!(world
            .observe(near)
            .unwrap()
            .events
            .iter()
            .all(|event| { event.text != "Worms are in the damp earth again." }));
        world.step(&[]);
        let grown = "Worms are in the damp earth again.";
        let back = world.observe(near).unwrap();
        assert!(
            back.text.contains("Worms are in the damp earth here."),
            "{}",
            back.text
        );
        assert!(!back.text.contains("The damp earth here has no worms."));
        assert!(back.events.iter().any(|event| event.text == grown));
        assert!(world
            .observe(seen)
            .unwrap()
            .events
            .iter()
            .any(|event| event.text == grown));
        assert!(world
            .observe(seen)
            .unwrap()
            .text
            .contains("Worms are in the damp earth to the north and west."));
        assert!(world
            .observe(quiet)
            .unwrap()
            .events
            .iter()
            .all(|event| event.text != grown));
        assert!(!world.observe(quiet).unwrap().text.contains("damp earth"));
        assert_eq!(world.pile_count(bank, Material::Worm), 1);
        let again = world.step(&[intent("again", near, Attempt::Gather(Material::Worm))]);
        assert_eq!(again.results[0].message, "You gather a worm.");
    }

    #[test]
    fn quartz_on_the_rock_face_is_a_clear_stone() {
        let mut world = World::valley(crate::tuning::Tuning::evening(), 1);
        let near = world.spawn_at("Near", "n", 4, 3).unwrap();
        let seen = world.spawn_at("Seen", "s", 5, 3).unwrap();
        let quiet = world.spawn_at("Quiet", "q", 3, 3).unwrap();
        let face = cell_id(6, 1).unwrap();
        let distant = "A clear stone of quartz lies on the rock face to the north and east.";
        assert!(world.observe(near).unwrap().text.contains(distant));
        assert!(world
            .observe(seen)
            .unwrap()
            .text
            .contains("A clear stone of quartz lies on the rock face to the north and east."));
        assert!(!world
            .observe(quiet)
            .unwrap()
            .text
            .contains("clear stone of quartz"));
        let missed = world.step(&[intent("none", near, Attempt::Gather(Material::Quartz))]);
        assert_eq!(
            missed.results[0].message,
            "There is no quartz here to gather."
        );
        assert!(says(&world, near, "There is no quartz here to gather."));
        assert!(!says(&world, seen, "There is no quartz here to gather."));
        assert!(!says(&world, quiet, "There is no quartz here to gather."));
        for (n, step) in [Dir::North, Dir::East, Dir::North, Dir::East]
            .into_iter()
            .enumerate()
        {
            let moved = world.step(&[intent(&format!("walk{n}"), near, Attempt::Move(step))]);
            assert!(moved.results[0].success, "{}", moved.results[0].message);
        }
        assert_eq!(world.actor(near).unwrap().cell, face);
        let here = world.observe(near).unwrap().text;
        assert!(
            here.contains("A clear stone of quartz lies on the rock face. It is a clear stone.")
        );
        assert!(here.contains("A lump of ochre lies on the rock face."));
        let gathered = world.step(&[intent("gather", near, Attempt::Gather(Material::Quartz))]);
        assert_eq!(
            gathered.results[0].message,
            "You gather a clear stone of quartz."
        );
        let third = "Near gathers a clear stone of quartz.";
        assert!(says(&world, seen, third));
        assert!(!says(&world, quiet, third));
        assert!(!world
            .observe(near)
            .unwrap()
            .events
            .iter()
            .any(|event| event.text == third));
        let after = world.observe(near).unwrap().text;
        assert!(after.contains("The rock face has no quartz."));
        assert!(after.contains("You have a clear stone of quartz."));
        assert!(after.contains("A lump of ochre lies on the rock face."));
        let second = world.step(&[intent("again", near, Attempt::Gather(Material::Quartz))]);
        assert_eq!(second.results[0].message, "The rock face has no quartz.");
        assert!(says(&world, near, "The rock face has no quartz."));
        assert!(!says(&world, seen, "The rock face has no quartz."));
        let bare = world.step(&[intent("strike", near, Attempt::Strike)]);
        assert_eq!(bare.results[0].message, "You have no flint to strike.");
        assert!(says(&world, near, "You have no flint to strike."));
        assert!(!says(&world, seen, "You have no flint to strike."));
        for (n, step) in [Dir::West, Dir::West, Dir::North].into_iter().enumerate() {
            let moved = world.step(&[intent(&format!("slope{n}"), near, Attempt::Move(step))]);
            assert!(moved.results[0].success, "{}", moved.results[0].message);
        }
        let nodule = world.step(&[intent("flint", near, Attempt::Gather(Material::Flint))]);
        assert_eq!(nodule.results[0].message, "You gather a flint nodule.");
        let struck = world.step(&[intent("sparks", near, Attempt::Strike)]);
        assert_eq!(
            struck.results[0].message,
            "You strike the flint on the quartz. Sparks jump."
        );
        let spark = "You strike the flint on the quartz. Sparks jump.";
        assert!(!says(&world, near, spark));
        assert!(!says(&world, seen, spark));
        assert!(!says(&world, quiet, spark));
        assert!(world.fire_at(world.actor(near).unwrap().cell).is_none());
        assert!(world
            .observe(near)
            .unwrap()
            .text
            .contains("a clear stone of quartz"));
        assert!(world
            .observe(seen)
            .unwrap()
            .text
            .contains("The rock face to the north and east has no quartz."));
    }

    #[test]
    fn sparks_catch_in_dry_tinder_and_a_dry_branch() {
        let mut world = World::valley(crate::tuning::Tuning::evening(), 1);
        let face = cell_id(6, 1).unwrap();
        let near = world.spawn_at("Near", "n", 6, 1).unwrap();
        let seen = world.spawn_at("Seen", "s", 5, 3).unwrap();
        let quiet = world.spawn_at("Quiet", "q", 3, 3).unwrap();
        world.add_pile(face, Material::Flint, 1, 0);
        let nodule = world.step(&[intent("flint", near, Attempt::Gather(Material::Flint))]);
        assert_eq!(nodule.results[0].message, "You gather a flint nodule.");
        let stone = world.step(&[intent("stone", near, Attempt::Gather(Material::Quartz))]);
        assert_eq!(
            stone.results[0].message,
            "You gather a clear stone of quartz."
        );
        let bare = world.step(&[intent("bare", near, Attempt::Strike)]);
        assert_eq!(
            bare.results[0].message,
            "You strike the flint on the quartz. Sparks jump."
        );
        assert!(world.fire_at(face).is_none());
        let bark = world.step(&[intent("bark", near, Attempt::Gather(Material::Bark))]);
        assert!(bark.results[0].success, "{}", bark.results[0].message);
        let branch = world.step(&[intent("branch", near, Attempt::Gather(Material::Branch))]);
        assert!(branch.results[0].success, "{}", branch.results[0].message);
        let caught = world.step(&[intent("catch", near, Attempt::Strike)]);
        assert_eq!(
            caught.results[0].message,
            "You strike the flint on the quartz. The sparks catch."
        );
        let third = "Near strikes the flint on the quartz. The sparks catch.";
        assert!(says(&world, seen, third));
        assert!(!says(&world, quiet, third));
        assert!(!world
            .observe(near)
            .unwrap()
            .events
            .iter()
            .any(|event| event.text == third));
        let here = world.observe(near).unwrap().text;
        assert!(
            here.contains("A small fire burns here. It warms you."),
            "{here}"
        );
        assert!(here.contains("a clear stone of quartz"), "{here}");
        assert!(world.loose_ids(near, Material::Bark).is_empty());
        assert!(world.loose_ids(near, Material::Branch).is_empty());
        let again_bark = world.step(&[intent("bark2", near, Attempt::Gather(Material::Bark))]);
        assert!(
            again_bark.results[0].success,
            "{}",
            again_bark.results[0].message
        );
        let again_branch =
            world.step(&[intent("branch2", near, Attempt::Gather(Material::Branch))]);
        assert!(
            again_branch.results[0].success,
            "{}",
            again_branch.results[0].message
        );
        let blocked = world.step(&[intent("blocked", near, Attempt::Strike)]);
        assert_eq!(blocked.results[0].message, "A fire already burns here.");
        assert!(says(&world, near, "A fire already burns here."));
        assert!(!says(&world, seen, "A fire already burns here."));
        assert!(!says(&world, quiet, "A fire already burns here."));

        let mut moss = World::valley(crate::tuning::Tuning::evening(), 1);
        let near = moss.spawn_at("Near", "n", 6, 1).unwrap();
        moss.add_pile(face, Material::Flint, 1, 0);
        moss.step(&[intent("flint", near, Attempt::Gather(Material::Flint))]);
        moss.step(&[intent("stone", near, Attempt::Gather(Material::Quartz))]);
        moss.step(&[intent("moss", near, Attempt::Gather(Material::Moss))]);
        moss.step(&[intent("branch", near, Attempt::Gather(Material::Branch))]);
        let caught = moss.step(&[intent("catch", near, Attempt::Strike)]);
        assert_eq!(
            caught.results[0].message,
            "You strike the flint on the quartz. The sparks catch."
        );
        assert!(moss.fire_at(face).is_some());
        assert!(moss.loose_ids(near, Material::Moss).is_empty());

        let mut fluff = World::valley(crate::tuning::Tuning::evening(), 1);
        let near = fluff.spawn_at("Near", "n", 6, 1).unwrap();
        fluff.add_pile(face, Material::Flint, 1, 0);
        fluff.mint_item(Material::Fluff, 0, face, Some(near), Role::Loose);
        fluff.step(&[intent("flint", near, Attempt::Gather(Material::Flint))]);
        fluff.step(&[intent("stone", near, Attempt::Gather(Material::Quartz))]);
        fluff.step(&[intent("branch", near, Attempt::Gather(Material::Branch))]);
        let caught = fluff.step(&[intent("catch", near, Attempt::Strike)]);
        assert_eq!(
            caught.results[0].message,
            "You strike the flint on the quartz. The sparks catch."
        );
        assert!(fluff.loose_ids(near, Material::Fluff).is_empty());

        let mut damp = World::valley(crate::tuning::Tuning::evening(), 1);
        let near = damp.spawn_at("Near", "n", 6, 1).unwrap();
        damp.add_pile(face, Material::Flint, 1, 0);
        damp.mint_item(
            Material::Fluff,
            damp.tuning.damp_at,
            face,
            Some(near),
            Role::Loose,
        );
        damp.step(&[intent("flint", near, Attempt::Gather(Material::Flint))]);
        damp.step(&[intent("stone", near, Attempt::Gather(Material::Quartz))]);
        damp.step(&[intent("branch", near, Attempt::Gather(Material::Branch))]);
        let jumped = damp.step(&[intent("jump", near, Attempt::Strike)]);
        assert_eq!(
            jumped.results[0].message,
            "You strike the flint on the quartz. Sparks jump."
        );
        assert!(damp.fire_at(face).is_none());
        assert!(!damp.loose_ids(near, Material::Fluff).is_empty());
    }
}
