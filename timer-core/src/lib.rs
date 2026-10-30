//! Pure session logic for the TABATA TIMER. This crate has no web dependencies.

pub mod timeline;
pub mod timer;

pub use timeline::{Phase, PhaseKind, Signal, SignalEvent, Timeline};
pub use timer::{InvalidTimer, Timer};
