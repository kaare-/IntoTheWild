//! Player lines become attempts. The core still decides whether they happen.

use crate::model::{Attempt, Dir, ExamineTarget, Material};

pub fn help_text() -> &'static str {
    "\
look                         where you are, and the local map
examine wood|bark|berries|poison berries|mushrooms|pot|mud|water|stone|flint|ochre|iron ore|bloom of iron|iron knife|copper ore|copper bead|awl|flake|clay|bowl|spear|carcass|hide|brand|fire|deer|tracks|log|frame|rock|slope|self|here|<name>
say <words>                  people nearby hear you
n s e w                      walk
gather branch|bark|berries|poison berries|mushrooms|pot|mud|water|stone|flint|ochre|iron ore|bloom of iron|iron knife|copper ore|copper bead|awl|clay|bowl|spear|carcass|meat|hide
drop branch|bark|berries|poison berries|mushrooms|pot|mud|water|stone|flint|ochre|iron ore|bloom of iron|iron knife|copper ore|copper bead|awl|clay|bowl|spear|carcass|meat|hide
set clay                     a lump at a burning fire; it hardens into a bowl
bind spear                   a carried stone and a carried branch; hold it at a fire
knap                         a carried flint nodule into a sharp flake
haft                         a carried flint flake and a dry branch into a spear; hide strips, if carried, bind the tip. Hold it at a fire
dry branch                   a damp or soaked branch at a burning fire; after the wait it is dry and can haft. The fire is not used up.
take brand                   a burning brand from a fire already burning here; the fire stays lit. It goes out in the rain, or after a short wait.
grab                         take hold of the heavy log
drag n|s|e|w                 pull; one person cannot move it
release                      let go of the log
raise frame                  three branches, then three phases of work
lay bark                     bark over a frame supported at both ends
drink                        raw water where you stand; still water freezes
boil                         a pot or a fired bowl, and a fire; unfired clay cannot
light fire                   dry bark and a dry branch, from the arms if you carry them
cook                         heat a mushroom, or cook berries, a carcass, or raw meat; poison berries stay poisonous
hunt                         a hardened spear; the deer on your cell or the next
fish                         a fire-hardened flint spear, in the brook or beside it; the spear is not used up. That stretch is quiet for a short wait
spear fish                   the same cast
butcher                      a carcass at your feet into three portions; a flint spear or an iron knife in your arms. With both, the knife cuts. Neither is used up.
cut meat                     one portion of raw or cooked meat into two smaller pieces; an iron knife in your arms. The knife is not used up.
cut hide                     a whole deer hide into hide strips, with an iron knife. The knife is not used up.
set snare                    hide strips, from the arms or the ground, into a snare in this cell. After a short wait it catches a hare. The strips are spent.
set trap                     the same snare
shape clay                   a lump in your arms or at your feet into an unfired clay pot. It is not fired.
fire pot                     an unfired pot at a burning fire; it hardens and can hold water. The fire is not used up.
build lean-to                three dry branches into a lean-to in this cell. It breaks the wind.
gather reeds                 a bundle at the brook's edge
thatch lean-to               a carried bundle of reeds on a lean-to already standing here. You are out of the wind, and the rain does not reach you
weave basket                 a sheet of bark, from the arms or the pile at your feet, into a bark basket. It holds one handful of berries, or one raw fish, that would not fit in the arms.
gather charcoal              a lump left when a small fire goes out, from rain or from the fuel ending
make basket                  the same weave
put fish                     a raw fish from the arms into that basket. Berries and a fish do not share it.
wear hide                    a carried deer hide, against the cold
remove hide                  a worn deer hide; it returns to the arms
eat berries                  sweet berries only; poison berries sicken you
eat meat                     cooked meat; raw meat is not food
eat fish                     a cooked fish; raw fish is not food
eat mushrooms                a cooked mushroom; raw mushrooms do not feed you
eat carcass                  a carcass must be cooked
eat bark                     bark is tinder, not food
smear mud                    mud on skin against the sun
smear ochre                  red ochre on the skin, a mark unlike mud
mark                         a carried lump of ochre leaves a red mark on this ground. A lump of charcoal leaves a black mark instead, and is used up. With neither, you have nothing to mark with
mark rock                    the same mark on the rock, when you stand under it. Rain washes an open mark and leaves one under the rock
smelt                        a carried lump of copper ore at a burning fire; after the same wait as clay, a copper bead
smelt iron                   a carried lump of iron ore; a small fire refuses it, and a fire fed an extra dry branch yields a bloom of iron
feed                         an extra dry branch, or a carried lump of charcoal, into a burning fire; the fire burns hot enough for iron. With neither, you have nothing to feed the fire
hammer|forge                 a carried copper bead at a fire, or with a stone, into a copper awl
hammer bloom                 a carried bloom of iron at a hotter fire, or with a stone, into an iron knife
pierce hide                  a carried or worn deer hide, with a copper awl; the hide is ready to sew
sew hide                     a pierced deer hide into a cloak, worn against the cold
rest|sleep                   under the rock or a lean-to, tiredness eases; the open wind refuses
interrupt                    stop the work your body is in
help"
}

pub fn parse_attempt(line: &str) -> Result<Attempt, String> {
    let line = line.trim();
    if line.is_empty() {
        return Err("Say what you try. Type help.".into());
    }
    let lower = line.to_ascii_lowercase();
    let mut parts = lower.split_whitespace();
    let head = parts.next().unwrap();
    let rest = lower[head.len()..].trim();
    match head {
        "help" | "?" => Err(help_text().into()),
        "look" | "l" | "orient" => {
            if rest.is_empty() {
                Ok(Attempt::Look)
            } else if let Some(target) = rest.strip_prefix("at ") {
                Ok(Attempt::Examine(parse_target(target.trim())))
            } else {
                Ok(Attempt::Examine(parse_target(rest)))
            }
        }
        "examine" | "exam" | "ex" | "x" => Ok(Attempt::Examine(parse_target(rest))),
        "say" | "'" => {
            let spoken = line[head.len()..].trim();
            if spoken.is_empty() {
                Err("Say something.".into())
            } else {
                Ok(Attempt::Say(spoken.to_string()))
            }
        }
        "n" | "north" => Ok(Attempt::Move(Dir::North)),
        "s" | "south" => Ok(Attempt::Move(Dir::South)),
        "e" | "east" => Ok(Attempt::Move(Dir::East)),
        "w" | "west" => Ok(Attempt::Move(Dir::West)),
        "move" | "go" | "walk" => parse_dir(rest).map(Attempt::Move),
        "take"
            if rest == "brand"
                || rest == "ember"
                || rest == "a brand"
                || rest == "an ember"
                || rest == "a burning brand"
                || rest == "burning brand"
                || rest == "the brand"
                || rest == "the ember" =>
        {
            Ok(Attempt::TakeBrand)
        }
        "gather" | "get" | "take" => parse_material(rest).map(Attempt::Gather),
        "drop" => parse_material(rest).map(Attempt::Drop),
        "grab" | "grip" => Ok(Attempt::Grab),
        "drag" | "pull" => parse_dir(rest).map(Attempt::Drag),
        "release" | "ungrab" => Ok(Attempt::Release),
        "let" if rest == "go" => Ok(Attempt::Release),
        "interrupt" | "stop" => Ok(Attempt::Interrupt),
        "raise" if rest == "frame" => Ok(Attempt::RaiseFrame),
        "build" if rest == "frame" => Ok(Attempt::RaiseFrame),
        "build"
            if rest == "lean-to"
                || rest == "leanto"
                || rest == "shelter"
                || rest == "a lean-to" =>
        {
            Ok(Attempt::BuildLeanTo)
        }
        "thatch"
            if rest.is_empty()
                || rest == "lean-to"
                || rest == "leanto"
                || rest == "the lean-to"
                || rest == "shelter" =>
        {
            Ok(Attempt::Thatch)
        }
        "weave" | "make" if rest == "basket" || rest == "a basket" || rest == "bark basket" => {
            Ok(Attempt::WeaveBasket)
        }
        "put"
            if matches!(
                rest,
                "fish"
                    | "a fish"
                    | "the fish"
                    | "raw fish"
                    | "a raw fish"
                    | "the raw fish"
                    | "fish in basket"
                    | "fish in the basket"
                    | "the fish in the basket"
                    | "raw fish in basket"
                    | "raw fish in the basket"
                    | "the raw fish in the basket"
                    | "fish into basket"
                    | "fish into the basket"
                    | "the fish into the basket"
            ) =>
        {
            Ok(Attempt::PutFish)
        }
        "lay" if rest == "bark" => Ok(Attempt::LayBark),
        "roof" => Ok(Attempt::LayBark),
        "drink" => Ok(Attempt::Drink),
        "boil" => Ok(Attempt::Boil),
        "set" if rest == "clay" => Ok(Attempt::SetClay),
        "set"
            if rest == "snare"
                || rest == "a snare"
                || rest == "the snare"
                || rest == "trap"
                || rest == "a trap"
                || rest == "the trap" =>
        {
            Ok(Attempt::SetSnare)
        }
        "bind" if rest == "spear" => Ok(Attempt::BindSpear),
        "knap"
            if rest.is_empty() || rest == "flint" || rest == "nodule" || rest == "flint nodule" =>
        {
            Ok(Attempt::Knap)
        }
        "haft" if rest.is_empty() || rest == "spear" || rest == "flake" || rest == "flint" => {
            Ok(Attempt::Haft)
        }
        "fish" if rest.is_empty() || rest == "spear" => Ok(Attempt::Fish),
        "spear" if rest == "fish" => Ok(Attempt::Fish),
        "dry" if rest.is_empty() || rest == "branch" || rest == "wood" || rest == "the branch" => {
            Ok(Attempt::DryBranch)
        }
        "hunt" if rest.is_empty() || rest == "deer" => Ok(Attempt::Hunt),
        "butcher" if rest.is_empty() || rest == "carcass" || rest == "deer" => Ok(Attempt::Butcher),
        "cut" if rest == "meat" || rest == "raw meat" || rest == "cooked meat" => Ok(Attempt::Cut),
        "cut"
            if rest == "hide"
                || rest == "deer hide"
                || rest == "the hide"
                || rest == "strips"
                || rest == "hide strips" =>
        {
            Ok(Attempt::CutHide)
        }
        "shape" if rest.is_empty() || rest == "clay" || rest == "pot" || rest == "clay pot" => {
            Ok(Attempt::Shape)
        }
        "bake" if rest.is_empty() || rest == "pot" || rest == "clay pot" || rest == "the pot" => {
            Ok(Attempt::FirePot)
        }
        "fire" if rest == "pot" || rest == "clay pot" || rest == "the pot" => Ok(Attempt::FirePot),
        "wear" | "wrap" if rest.is_empty() || rest == "hide" || rest == "deer hide" => {
            Ok(Attempt::Wear)
        }
        "remove" if rest.is_empty() || rest == "hide" || rest == "deer hide" => Ok(Attempt::Remove),
        "light" | "kindle" if rest.is_empty() || rest == "fire" => Ok(Attempt::Light),
        "cook" => Ok(Attempt::Cook),
        "eat" => parse_eat(rest),
        "smear" if rest.is_empty() || rest == "mud" => Ok(Attempt::Smear),
        "smear" if rest == "ochre" => Ok(Attempt::SmearOchre),
        "mark" if rest.is_empty() || rest == "ground" => Ok(Attempt::Mark),
        "mark" if rest == "rock" => Ok(Attempt::MarkRock),
        "smelt" if rest.is_empty() || rest == "ore" || rest == "copper" || rest == "copper ore" => {
            Ok(Attempt::Smelt)
        }
        "smelt" if rest == "iron" || rest == "iron ore" => Ok(Attempt::SmeltIron),
        "feed"
            if rest.is_empty()
                || rest == "fire"
                || rest == "branch"
                || rest == "the fire"
                || rest == "charcoal"
                || rest == "lump of charcoal" =>
        {
            Ok(Attempt::Feed)
        }
        "hammer" | "forge"
            if rest.is_empty() || rest == "bead" || rest == "copper" || rest == "copper bead" =>
        {
            Ok(Attempt::Hammer)
        }
        "hammer" | "forge"
            if rest == "bloom"
                || rest == "iron"
                || rest == "bloom of iron"
                || rest == "knife"
                || rest == "iron knife" =>
        {
            Ok(Attempt::HammerIron)
        }
        "pierce" if rest.is_empty() || rest == "hide" || rest == "deer hide" => Ok(Attempt::Pierce),
        "sew" if rest.is_empty() || rest == "hide" || rest == "deer hide" || rest == "cloak" => {
            Ok(Attempt::Sew)
        }
        "rest" | "sleep" if rest.is_empty() => Ok(Attempt::Rest),
        _ => Err("Unknown attempt. Type help.".into()),
    }
}

fn parse_dir(text: &str) -> Result<Dir, String> {
    match text.trim() {
        "n" | "north" => Ok(Dir::North),
        "s" | "south" => Ok(Dir::South),
        "e" | "east" => Ok(Dir::East),
        "w" | "west" => Ok(Dir::West),
        _ => Err("Name a direction: n, s, e, or w.".into()),
    }
}

fn parse_material(text: &str) -> Result<Material, String> {
    match text.trim() {
        "branch" | "branches" | "wood" | "firewood" => Ok(Material::Branch),
        "bark" => Ok(Material::Bark),
        "log" => Ok(Material::Log),
        "berry" | "berries" | "sweet berries" => Ok(Material::Berry),
        "bitter" | "bitter berry" | "bitter berries" | "poison" | "poison berry"
        | "poison berries" => Ok(Material::BitterBerry),
        "mushroom" | "mushrooms" => Ok(Material::Mushroom),
        "pot" => Ok(Material::Pot),
        "mud" => Ok(Material::Mud),
        "water" => Ok(Material::Water),
        "stone" | "stones" => Ok(Material::Stone),
        "flint" | "flint nodule" | "nodule" => Ok(Material::Flint),
        "ochre" => Ok(Material::Ochre),
        "iron" | "iron ore" => Ok(Material::IronOre),
        "bloom" | "iron bloom" | "bloom of iron" | "lump of iron" => Ok(Material::Iron),
        "knife" | "iron knife" => Ok(Material::Knife),
        "copper ore" | "ore" => Ok(Material::CopperOre),
        "copper" | "copper bead" | "bead" => Ok(Material::Copper),
        "awl" | "copper awl" => Ok(Material::Awl),
        "clay" => Ok(Material::Clay),
        "bowl" => Ok(Material::Bowl),
        "spear" | "spears" => Ok(Material::Spear),
        "carcass" | "deer carcass" => Ok(Material::Carcass),
        "meat" | "raw meat" | "cooked meat" => Ok(Material::Meat),
        "fish" | "raw fish" | "cooked fish" => Ok(Material::Fish),
        "brand" | "ember" | "burning brand" => Ok(Material::Brand),
        "hide" | "deer hide" => Ok(Material::Hide),
        "basket" | "bark basket" => Ok(Material::Basket),
        "charcoal" | "lump of charcoal" => Ok(Material::Charcoal),
        "reed" | "reeds" | "bundle of reeds" => Ok(Material::Reed),
        "strips" | "hide strips" | "cord" | "cordage" => Ok(Material::Cord),
        _ => Err(
            "Name a material: branch, bark, berries, poison berries, mushrooms, pot, mud, water, stone, flint, ochre, iron ore, bloom of iron, iron knife, copper ore, copper bead, awl, clay, bowl, spear, carcass, meat, hide, strips, or log."
                .into(),
        ),
    }
}

fn parse_eat(text: &str) -> Result<Attempt, String> {
    match text.trim() {
        "" | "berry" | "berries" | "sweet" | "sweet berries" => Ok(Attempt::Eat(Material::Berry)),
        "mushroom" | "mushrooms" => Ok(Attempt::Eat(Material::Mushroom)),
        "bitter" | "bitter berry" | "bitter berries" | "poison" | "poison berry"
        | "poison berries" => Ok(Attempt::Eat(Material::BitterBerry)),
        "bark" => Ok(Attempt::Eat(Material::Bark)),
        "wood" | "branch" | "branches" | "log" => Err("Bark is tinder, not food.".into()),
        "mud" => Err("Mud is for your skin, not your stomach.".into()),
        "carcass" | "deer carcass" => Ok(Attempt::Eat(Material::Carcass)),
        "meat" | "cooked meat" | "raw meat" => Ok(Attempt::Eat(Material::Meat)),
        "fish" | "cooked fish" | "raw fish" => Ok(Attempt::Eat(Material::Fish)),
        _ => Err("Name a food: berries, poison berries, mushrooms, or carcass.".into()),
    }
}

fn parse_target(text: &str) -> ExamineTarget {
    match text.trim() {
        "" | "here" | "ground" | "around" => ExamineTarget::Here,
        "self" | "me" | "myself" => ExamineTarget::SelfBody,
        "wood" | "branch" | "branches" | "firewood" => ExamineTarget::Wood,
        "bark" => ExamineTarget::Bark,
        "berry" | "berries" | "fruit" | "sweet berries" => ExamineTarget::Berries,
        "bitter" | "bitter berries" | "bitter berry" | "poison" | "poison berries"
        | "poison berry" => ExamineTarget::BitterBerries,
        "mud" => ExamineTarget::Mud,
        "mushroom" | "mushrooms" => ExamineTarget::Mushrooms,
        "pot" => ExamineTarget::Pot,
        "stone" | "stones" => ExamineTarget::Stone,
        "flint" | "flint nodule" | "nodule" => ExamineTarget::Flint,
        "ochre" => ExamineTarget::Ochre,
        "iron" | "iron ore" => ExamineTarget::IronOre,
        "bloom" | "iron bloom" | "bloom of iron" | "lump of iron" => ExamineTarget::Iron,
        "knife" | "iron knife" => ExamineTarget::Knife,
        "copper" | "copper ore" | "ore" => ExamineTarget::CopperOre,
        "copper bead" | "bead" => ExamineTarget::Copper,
        "awl" | "copper awl" => ExamineTarget::Awl,
        "flake" | "flint flake" | "sharp flake" => ExamineTarget::Flake,
        "clay" => ExamineTarget::Clay,
        "bowl" => ExamineTarget::Bowl,
        "spear" | "spears" => ExamineTarget::Spear,
        "carcass" | "deer carcass" => ExamineTarget::Carcass,
        "hide" | "deer hide" => ExamineTarget::Hide,
        "strips" | "hide strips" | "cord" | "cordage" => ExamineTarget::Cord,
        "meat" | "raw meat" => ExamineTarget::Meat,
        "fish" | "raw fish" | "cooked fish" => ExamineTarget::Fish,
        "brand" | "ember" | "burning brand" => ExamineTarget::Brand,
        "fire" | "flame" => ExamineTarget::Fire,
        "water" | "brook" => ExamineTarget::Water,
        "deer" | "animal" => ExamineTarget::Deer,
        "tracks" | "hoofprints" | "prints" => ExamineTarget::Tracks,
        "log" => ExamineTarget::Log,
        "frame" | "shelter" | "roof" => ExamineTarget::Frame,
        "rock" | "overhang" | "rock overhang" => ExamineTarget::Rock,
        "basket" | "bark basket" => ExamineTarget::Basket,
        "charcoal" | "lump of charcoal" => ExamineTarget::Charcoal,
        "reed" | "reeds" | "bundle of reeds" => ExamineTarget::Reeds,
        "snare" | "trap" | "hide snare" => ExamineTarget::Snare,
        "slope" | "the slope" => ExamineTarget::Slope,
        other => ExamineTarget::Person(other.to_string()),
    }
}
