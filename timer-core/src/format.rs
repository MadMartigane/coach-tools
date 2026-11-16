//! Pure display formatting for the TABATA TIMER, per `docs/design-system.md`.

/// Seconds in one minute.
const SECONDS_PER_MINUTE: u64 = 60;

/// Seconds in one hour; durations from this value on use the `h:mm:ss` form.
const SECONDS_PER_HOUR: u64 = 3600;

/// Formats whole seconds as zero-padded `mm:ss`. Minutes are not capped at 59.
pub fn format_mm_ss(seconds: u64) -> String {
    let minutes = seconds / SECONDS_PER_MINUTE;
    let remaining_seconds = seconds % SECONDS_PER_MINUTE;
    format!("{minutes:02}:{remaining_seconds:02}")
}

/// Formats whole seconds as `mm:ss` below one hour, and `h:mm:ss` from one
/// hour on.
pub fn format_duration(seconds: u64) -> String {
    if seconds < SECONDS_PER_HOUR {
        return format_mm_ss(seconds);
    }

    let hours = seconds / SECONDS_PER_HOUR;
    let remaining_minutes_and_seconds = seconds % SECONDS_PER_HOUR;
    let minutes = remaining_minutes_and_seconds / SECONDS_PER_MINUTE;
    let remaining_seconds = remaining_minutes_and_seconds % SECONDS_PER_MINUTE;
    format!("{hours}:{minutes:02}:{remaining_seconds:02}")
}

#[cfg(test)]
mod tests {
    use super::{format_duration, format_mm_ss};

    #[test]
    fn mm_ss_is_zero_padded() {
        assert_eq!(format_mm_ss(0), "00:00");
        assert_eq!(format_mm_ss(487), "08:07");
    }

    #[test]
    fn mm_ss_does_not_cap_minutes() {
        assert_eq!(format_mm_ss(3600), "60:00");
    }

    #[test]
    fn duration_below_one_hour_uses_mm_ss() {
        assert_eq!(format_duration(0), "00:00");
        assert_eq!(format_duration(3599), "59:59");
    }

    #[test]
    fn duration_from_one_hour_uses_h_mm_ss() {
        assert_eq!(format_duration(3600), "1:00:00");
        assert_eq!(format_duration(3661), "1:01:01");
    }
}
