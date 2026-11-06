//! Local server. The simulation stays in one task; HTTP only carries intents
//! and observations.

mod host;
mod http;
mod persist;

pub use host::Host;
pub use http::{router, run_host, App, Msg};
pub use persist::{Record, Store};
