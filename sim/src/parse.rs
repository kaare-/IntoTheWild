//! Player lines become attempts. The core still decides whether they happen.

use crate::model::{Attempt, Dir, ExamineTarget, Material};

pub fn help_text() -> &'static str {
    "\
look                         where you are, and the local map
examine wood|bark|log|frame|self|here|<name>
say <words>                  people nearby hear you
n s e w                      walk
gather branch|bark           take one piece into your arms
drop branch|bark
grab                         take hold of the heavy log
drag n|s|e|w                 pull; one person cannot move it
release                      let go of the log
raise frame                  three branches, then three phases of work
lay bark                     bark over a frame supported at both ends
drink                        drink from the brook, if you are standing in it
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
        _ => Err("Name a material: branch, bark, or log.".into()),
    }
}

fn parse_target(text: &str) -> ExamineTarget {
    match text.trim() {
        "" | "here" | "ground" | "around" => ExamineTarget::Here,
        "self" | "me" | "myself" => ExamineTarget::SelfBody,
        "wood" | "branch" | "branches" | "firewood" => ExamineTarget::Wood,
        "bark" => ExamineTarget::Bark,
        "log" => ExamineTarget::Log,
        "frame" | "shelter" | "roof" => ExamineTarget::Frame,
        other => ExamineTarget::Person(other.to_string()),
    }
}
