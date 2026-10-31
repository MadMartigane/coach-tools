//! A `timer_core::Clock` backed by the browser's monotonic performance clock.

use leptos::prelude::window;
use timer_core::Clock;

const MILLISECONDS_PER_SECOND: f64 = 1000.0;

/// Counts whole seconds elapsed since it was created.
#[derive(Clone, Copy)]
pub struct BrowserClock {
    start_ms: f64,
}

impl BrowserClock {
    /// Creates a clock whose zero point is the current instant.
    pub fn new() -> Self {
        Self {
            start_ms: Self::now_ms(),
        }
    }

    /// Milliseconds from an arbitrary browser origin, monotonic when
    /// `performance` is available and wall-clock time otherwise.
    fn now_ms() -> f64 {
        window()
            .performance()
            .map(|performance| performance.now())
            .unwrap_or_else(js_sys::Date::now)
    }
}

impl Default for BrowserClock {
    fn default() -> Self {
        Self::new()
    }
}

impl Clock for BrowserClock {
    fn now(&self) -> u64 {
        let elapsed_ms = (Self::now_ms() - self.start_ms).max(0.0);
        (elapsed_ms / MILLISECONDS_PER_SECOND).floor() as u64
    }
}
