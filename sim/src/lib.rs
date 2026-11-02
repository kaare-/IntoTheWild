//! Villmarks simulation. No sockets, no clock, no database, no model.
//! Call [`World::step`] with the intents that belong to one window.

mod model;
mod observe;
mod parse;
mod process;
mod relations;
mod resolve;
mod serde_map;
mod tuning;

pub use model::{
    contest_hash, ActorId, Attempt, CommandResult, ConstructionId, Dir, Event, EventId,
    ExamineTarget, Intent, ItemId, Material, StepReport, WorkId, World, HEIGHT, WIDTH,
};
pub use observe::Observation;
pub use parse::{help_text, parse_attempt};
pub use tuning::Tuning;
