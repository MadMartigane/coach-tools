//! Pure session logic for the TABATA TIMER. This crate has no web dependencies.

pub mod clock;
pub mod session;
pub mod timeline;
pub mod timer;

pub use clock::{Clock, SimulatedClock};
pub use session::{PhaseView, Session, SessionState};
pub use timeline::{Phase, PhaseKind, Signal, SignalEvent, Timeline};
pub use timer::{InvalidTimer, Timer};
