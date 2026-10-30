//! The timer configuration a user edits before a session starts.

/// A TABATA-style interval program. Every duration is a whole number of
/// seconds, which makes negative durations structurally impossible.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Timer {
    /// Display name of the program.
    pub name: String,
    /// Seconds before the first work cycle.
    pub preparation: u32,
    /// Seconds of work in one cycle.
    pub work: u32,
    /// Seconds of rest between two cycles of the same set.
    pub rest: u32,
    /// Number of work cycles in one set.
    pub cycles_per_set: u32,
    /// Number of sets in the session.
    pub sets: u32,
    /// Seconds of rest between two sets.
    pub rest_between_sets: u32,
    /// Seconds of cool-down after the last set.
    pub cool_down: u32,
}

/// Why a [`Timer`] cannot be used to build a session timeline.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InvalidTimer {
    /// The name is empty or contains only whitespace.
    EmptyName,
    /// A set has no work cycle.
    CyclesTooLow,
    /// The session has no set.
    SetsTooLow,
}

impl Timer {
    /// Checks the program is usable, returning the first problem found.
    pub fn validate(&self) -> Result<(), InvalidTimer> {
        if self.name.trim().is_empty() {
            return Err(InvalidTimer::EmptyName);
        }
        if self.cycles_per_set < 1 {
            return Err(InvalidTimer::CyclesTooLow);
        }
        if self.sets < 1 {
            return Err(InvalidTimer::SetsTooLow);
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::{InvalidTimer, Timer};

    fn tabata() -> Timer {
        Timer {
            name: String::from("Tabata"),
            preparation: 10,
            work: 20,
            rest: 10,
            cycles_per_set: 8,
            sets: 1,
            rest_between_sets: 0,
            cool_down: 0,
        }
    }

    #[test]
    fn valid_timer_passes() {
        let timer = Timer {
            preparation: 15,
            work: 30,
            rest: 20,
            cycles_per_set: 8,
            sets: 1,
            rest_between_sets: 45,
            cool_down: 30,
            ..tabata()
        };

        assert_eq!(timer.validate(), Ok(()));
    }

    #[test]
    fn blank_name_is_rejected() {
        for name in ["", "   "] {
            let timer = Timer {
                name: String::from(name),
                ..tabata()
            };

            assert_eq!(timer.validate(), Err(InvalidTimer::EmptyName));
        }
    }

    #[test]
    fn zero_cycles_is_rejected() {
        let timer = Timer {
            cycles_per_set: 0,
            ..tabata()
        };

        assert_eq!(timer.validate(), Err(InvalidTimer::CyclesTooLow));
    }

    #[test]
    fn zero_sets_is_rejected() {
        let timer = Timer {
            sets: 0,
            ..tabata()
        };

        assert_eq!(timer.validate(), Err(InvalidTimer::SetsTooLow));
    }

    #[test]
    fn first_error_wins() {
        let timer = Timer {
            name: String::new(),
            cycles_per_set: 0,
            ..tabata()
        };

        assert_eq!(timer.validate(), Err(InvalidTimer::EmptyName));
    }
}
