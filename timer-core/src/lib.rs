//! Pure session logic for the TABATA TIMER. This crate has no web dependencies.

pub mod clock;
pub mod format;
pub mod session;
pub mod timeline;
pub mod timer;

pub use clock::{Clock, SimulatedClock};
pub use format::{format_duration, format_mm_ss, format_timer_summary, phase_label};
pub use session::{PhaseView, Session, SessionState};
pub use timeline::{Phase, PhaseKind, Signal, SignalEvent, Timeline};
pub use timer::{InvalidTimer, Timer};
