//! Player lines become attempts. The core still decides whether they happen.

use crate::model::{Attempt, Dir, ExamineTarget, Material};

pub fn help_text() -> &'static str {
    "\
look                         where you are, and the local map
examine wood|bark|berries|poison berries|mushrooms|pot|mud|water|stone|clay|bowl|spear|carcass|fire|deer|tracks|log|frame|rock|slope|self|here|<name>
say <words>                  people nearby hear you
n s e w                      walk
gather branch|bark|berries|poison berries|mushrooms|pot|mud|water|stone|clay|bowl|spear|carcass
drop branch|bark|berries|poison berries|mushrooms|pot|mud|water|stone|clay|bowl|spear|carcass
set clay                     a lump at a burning fire; it hardens into a bowl
bind spear                   a carried stone and a carried branch; hold it at a fire
grab                         take hold of the heavy log
drag n|s|e|w                 pull; one person cannot move it
release                      let go of the log
raise frame                  three branches, then three phases of work
lay bark                     bark over a frame supported at both ends
drink                        raw water where you stand; still water freezes
boil                         a pot or a fired bowl, and a fire; unfired clay cannot
light fire                   dry bark and a dry branch, from the arms if you carry them
cook                         mushrooms, sweet berries, or a carcass; poison berries are not food
hunt                         a hardened spear; the deer on your cell or the next
eat berries                  sweet berries only; poison berries sicken you
eat carcass                  raw meat is refused; it must be cooked
smear mud                    mud on skin against the sun
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
        "gather" | "get" | "take" => parse_material(rest).map(Attempt::Gather),
        "drop" => parse_material(rest).map(Attempt::Drop),
        "grab" | "grip" => Ok(Attempt::Grab),
        "drag" | "pull" => parse_dir(rest).map(Attempt::Drag),
        "release" | "ungrab" => Ok(Attempt::Release),
        "let" if rest == "go" => Ok(Attempt::Release),
        "interrupt" | "stop" => Ok(Attempt::Interrupt),
        "raise" if rest == "frame" => Ok(Attempt::RaiseFrame),
        "build" if rest == "frame" => Ok(Attempt::RaiseFrame),
        "lay" if rest == "bark" => Ok(Attempt::LayBark),
        "roof" => Ok(Attempt::LayBark),
        "drink" => Ok(Attempt::Drink),
        "boil" => Ok(Attempt::Boil),
        "set" if rest == "clay" => Ok(Attempt::SetClay),
        "bind" if rest == "spear" => Ok(Attempt::BindSpear),
        "hunt" if rest.is_empty() || rest == "deer" => Ok(Attempt::Hunt),
        "light" | "kindle" if rest.is_empty() || rest == "fire" => Ok(Attempt::Light),
        "cook" => Ok(Attempt::Cook),
        "eat" => parse_eat(rest),
        "smear" if rest.is_empty() || rest == "mud" => Ok(Attempt::Smear),
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
        "clay" => Ok(Material::Clay),
        "bowl" => Ok(Material::Bowl),
        "spear" | "spears" => Ok(Material::Spear),
        "carcass" | "deer carcass" => Ok(Material::Carcass),
        _ => Err(
            "Name a material: branch, bark, berries, poison berries, mushrooms, pot, mud, water, stone, clay, bowl, spear, carcass, or log."
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
        "bark" | "wood" | "branch" | "branches" | "log" => {
            Err("Bark is tinder, not food. Light a fire and cook berries.".into())
        }
        "mud" => Err("Mud is for your skin, not your stomach.".into()),
        "carcass" | "deer carcass" | "meat" | "raw meat" => Ok(Attempt::Eat(Material::Carcass)),
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
        "clay" => ExamineTarget::Clay,
        "bowl" => ExamineTarget::Bowl,
        "spear" | "spears" => ExamineTarget::Spear,
        "carcass" | "deer carcass" => ExamineTarget::Carcass,
        "fire" | "flame" => ExamineTarget::Fire,
        "water" | "brook" => ExamineTarget::Water,
        "deer" | "animal" => ExamineTarget::Deer,
        "tracks" | "hoofprints" | "prints" => ExamineTarget::Tracks,
        "log" => ExamineTarget::Log,
        "frame" | "shelter" | "roof" => ExamineTarget::Frame,
        "rock" | "overhang" | "rock overhang" => ExamineTarget::Rock,
        "slope" | "the slope" => ExamineTarget::Slope,
        other => ExamineTarget::Person(other.to_string()),
    }
}
