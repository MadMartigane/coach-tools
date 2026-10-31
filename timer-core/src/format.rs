//! Pure display formatting for the TABATA TIMER, per `docs/design-system.md`.

use crate::timeline::PhaseKind;
use crate::timer::Timer;

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

/// Summarises a program, e.g. `Prép 10 · 20/10 ×8 · 4 sets · 31:00`.
pub fn format_timer_summary(timer: &Timer) -> String {
    let set_label = if timer.sets == 1 { "set" } else { "sets" };
    let total = format_duration(timer.total_duration());
    format!(
        "Prép {} · {}/{} ×{} · {} {} · {}",
        timer.preparation,
        timer.work,
        timer.rest,
        timer.cycles_per_set,
        timer.sets,
        set_label,
        total
    )
}

/// The French label shown for a phase kind.
pub fn phase_label(kind: PhaseKind) -> &'static str {
    match kind {
        PhaseKind::Preparation => "Préparation",
        PhaseKind::Work => "Travail",
        PhaseKind::Rest => "Repos",
        PhaseKind::RestBetweenSets => "Repos entre sets",
        PhaseKind::CoolDown => "Retour au calme",
    }
}

#[cfg(test)]
mod tests {
    use super::{format_duration, format_mm_ss, format_timer_summary, phase_label};
    use crate::timeline::PhaseKind;
    use crate::timer::Timer;

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

    #[test]
    fn summary_uses_singular_set_label_for_one_set() {
        // 10 + 1 * (8 * 20 + 7 * 10) + 0 + 0 = 240 seconds.
        assert_eq!(
            format_timer_summary(&tabata()),
            "Prép 10 · 20/10 ×8 · 1 set · 04:00"
        );
    }

    #[test]
    fn summary_uses_plural_set_label_for_several_sets() {
        let timer = Timer {
            sets: 4,
            rest_between_sets: 60,
            cool_down: 0,
            ..tabata()
        };

        // 10 + 4 * 230 + 3 * 60 + 0 = 1110 seconds.
        assert_eq!(
            format_timer_summary(&timer),
            "Prép 10 · 20/10 ×8 · 4 sets · 18:30"
        );
    }

    #[test]
    fn phase_labels_are_french() {
        assert_eq!(phase_label(PhaseKind::Preparation), "Préparation");
        assert_eq!(phase_label(PhaseKind::Work), "Travail");
        assert_eq!(phase_label(PhaseKind::Rest), "Repos");
        assert_eq!(phase_label(PhaseKind::RestBetweenSets), "Repos entre sets");
        assert_eq!(phase_label(PhaseKind::CoolDown), "Retour au calme");
    }
}
