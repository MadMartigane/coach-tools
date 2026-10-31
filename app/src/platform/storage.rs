//! Persists the timer list in `localStorage` as versioned JSON.

use leptos::prelude::window;
use serde::{Deserialize, Serialize};
use timer_core::Timer;

/// The `localStorage` key holding the serialized timer list.
pub const STORAGE_KEY: &str = "coach-tools.tabata.timers.v1";

/// The schema version written by this build; other versions are discarded.
pub const CURRENT_VERSION: u32 = 1;

/// The on-disk shape of the stored timer list.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct StoredTimers {
    /// Schema version of `timers`, compared against [`CURRENT_VERSION`].
    pub version: u32,
    /// The saved timer programs.
    pub timers: Vec<Timer>,
}

/// Parses a stored timer list, keeping only timers that pass validation.
///
/// Returns an empty list when the input is absent, malformed, or written with
/// a different schema version.
pub fn parse_timers_json(raw: &str) -> Vec<Timer> {
    let Ok(stored) = serde_json::from_str::<StoredTimers>(raw) else {
        return Vec::new();
    };
    if stored.version != CURRENT_VERSION {
        return Vec::new();
    }
    stored
        .timers
        .into_iter()
        .filter(|timer| timer.validate().is_ok())
        .collect()
}

/// Serializes timers into the versioned JSON document stored in `localStorage`.
pub fn serialize_timers_json(timers: &[Timer]) -> Result<String, serde_json::Error> {
    serde_json::to_string(&StoredTimers {
        version: CURRENT_VERSION,
        timers: timers.to_vec(),
    })
}

/// Loads the saved timer list, or an empty list when nothing is stored or
/// `localStorage` is unavailable.
pub fn load_timers() -> Vec<Timer> {
    let raw = window()
        .local_storage()
        .ok()
        .flatten()
        .and_then(|storage| storage.get_item(STORAGE_KEY).ok().flatten())
        .unwrap_or_default();
    parse_timers_json(&raw)
}

/// Saves the timer list to `localStorage`, silently skipping the write when
/// storage is unavailable or full.
pub fn save_timers(timers: &[Timer]) {
    let Ok(json) = serialize_timers_json(timers) else {
        return;
    };
    if let Some(storage) = window().local_storage().ok().flatten() {
        let _ = storage.set_item(STORAGE_KEY, &json);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn invalid_timer() -> Timer {
        Timer {
            cycles_per_set: 0,
            ..Timer::default_new()
        }
    }

    #[test]
    fn empty_input_yields_no_timers() {
        assert!(parse_timers_json("").is_empty());
    }

    #[test]
    fn valid_list_round_trips() {
        let timers = vec![Timer::default_new()];

        let json = serialize_timers_json(&timers).expect("serialize");

        assert_eq!(parse_timers_json(&json), timers);
    }

    #[test]
    fn wrong_version_is_discarded() {
        let json = r#"{"version":2,"timers":[]}"#;

        assert!(parse_timers_json(json).is_empty());
    }

    #[test]
    fn invalid_timers_are_filtered_out() {
        let valid = Timer::default_new();
        let json = serde_json::to_string(&StoredTimers {
            version: CURRENT_VERSION,
            timers: vec![invalid_timer(), valid.clone()],
        })
        .expect("serialize");

        assert_eq!(parse_timers_json(&json), vec![valid]);
    }
}
