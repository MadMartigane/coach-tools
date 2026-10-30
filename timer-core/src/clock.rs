//! The time source the session engine reads.

use std::cell::Cell;
use std::rc::Rc;

/// A source of whole seconds elapsed since the session started.
///
/// A clock must never return a smaller value on a later call, because the
/// session engine computes remaining time from timestamps instead of counting
/// ticks.
pub trait Clock {
    /// Whole seconds elapsed since the session started.
    fn now(&self) -> u64;
}

/// A clock moved by hand, shared between clones so a test can keep one and
/// hand the other to a session.
#[derive(Clone, Default)]
pub struct SimulatedClock(Rc<Cell<u64>>);

impl SimulatedClock {
    /// Creates a clock reading zero.
    pub fn new() -> Self {
        Self::default()
    }

    /// Moves the clock forward by `seconds`, through every clone.
    pub fn advance(&self, seconds: u64) {
        self.0.set(self.0.get() + seconds);
    }
}

impl Clock for SimulatedClock {
    fn now(&self) -> u64 {
        self.0.get()
    }
}

#[cfg(test)]
mod tests {
    use super::{Clock, SimulatedClock};

    #[test]
    fn starts_at_zero_and_advances() {
        let clock = SimulatedClock::new();
        assert_eq!(clock.now(), 0);

        clock.advance(5);
        assert_eq!(clock.now(), 5);

        clock.advance(2);
        assert_eq!(clock.now(), 7);
    }

    #[test]
    fn clones_share_time() {
        let clock = SimulatedClock::new();
        let clone = clock.clone();

        clone.advance(5);

        assert_eq!(clock.now(), 5);
    }
}
