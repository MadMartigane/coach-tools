//! The ordered phases and signal events of a session.

use crate::timer::{InvalidTimer, Timer};

/// Which part of the session a phase belongs to.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PhaseKind {
    /// Warm-up before the first work cycle.
    Preparation,
    /// A work cycle.
    Work,
    /// Rest between two work cycles of the same set.
    Rest,
    /// Rest between two sets.
    RestBetweenSets,
    /// Cool-down after the last set.
    CoolDown,
}

/// A contiguous part of the session, in whole seconds from session start.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Phase {
    /// Which part of the session this phase is.
    pub kind: PhaseKind,
    /// Whole seconds from session start where the phase begins.
    pub start: u64,
    /// Length of the phase in whole seconds.
    pub duration: u64,
    /// 1-based set this phase belongs to.
    pub set: u32,
    /// 1-based cycle within the set for [`PhaseKind::Work`] and
    /// [`PhaseKind::Rest`], `None` otherwise.
    pub cycle: Option<u32>,
}

/// A cue the user perceives during the session.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Signal {
    /// Start of a work cycle.
    Whistle,
    /// End of a work cycle that is not the last one of its set.
    Bell,
    /// End of the last work cycle of a set.
    TripleBell,
    /// One countdown second before a phase ends.
    CountdownBeep,
}

/// A [`Signal`] scheduled at a given second of the session.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SignalEvent {
    /// The cue to play.
    pub signal: Signal,
    /// Whole seconds from session start when the cue fires.
    pub at: u64,
}

/// The phases and cues of a session, derived from a [`Timer`].
pub struct Timeline {
    phases: Vec<Phase>,
    signals: Vec<SignalEvent>,
    total_duration: u64,
}

/// The countdown fires one, two and three seconds before a phase ends.
const COUNTDOWN_OFFSETS: [u64; 3] = [1, 2, 3];

/// The preparation phase opens the first set.
const FIRST_SET: u32 = 1;

impl Timeline {
    /// Builds the timeline of `timer`, after checking the program is valid.
    ///
    /// Zero-duration phases are left out together with their cues.
    pub fn build(timer: &Timer) -> Result<Timeline, InvalidTimer> {
        timer.validate()?;

        let mut timeline = TimelineBuilder::default();
        if timer.preparation > 0 {
            timeline.push_countdown_phase(
                PhaseKind::Preparation,
                timer.preparation.into(),
                FIRST_SET,
                None,
            );
        }

        for set in 1..=timer.sets {
            for cycle in 1..=timer.cycles_per_set {
                if timer.work > 0 {
                    let work =
                        timeline.push_phase(PhaseKind::Work, timer.work.into(), set, Some(cycle));
                    timeline.push_signal(Signal::Whistle, work.start);
                    let end_signal = if cycle == timer.cycles_per_set {
                        Signal::TripleBell
                    } else {
                        Signal::Bell
                    };
                    timeline.push_signal(end_signal, work.end());
                }

                let rests_before_next_cycle = cycle < timer.cycles_per_set;
                if rests_before_next_cycle && timer.rest > 0 {
                    timeline.push_countdown_phase(
                        PhaseKind::Rest,
                        timer.rest.into(),
                        set,
                        Some(cycle),
                    );
                }
            }

            if set < timer.sets && timer.rest_between_sets > 0 {
                timeline.push_phase(
                    PhaseKind::RestBetweenSets,
                    timer.rest_between_sets.into(),
                    set,
                    None,
                );
            }

            let is_last_set = set == timer.sets;
            if is_last_set && timer.cool_down > 0 {
                timeline.push_phase(PhaseKind::CoolDown, timer.cool_down.into(), set, None);
            }
        }

        Ok(timeline.finish())
    }

    /// The session phases, in chronological order.
    pub fn phases(&self) -> &[Phase] {
        &self.phases
    }

    /// The scheduled cues, sorted by `at` and keeping construction order on
    /// ties.
    pub fn signals(&self) -> &[SignalEvent] {
        &self.signals
    }

    /// Whole seconds from session start to the end of the last phase.
    pub fn total_duration(&self) -> u64 {
        self.total_duration
    }
}

impl Phase {
    fn end(&self) -> u64 {
        self.start + self.duration
    }
}

/// Accumulates phases and cues while [`Timeline::build`] walks the program.
#[derive(Default)]
struct TimelineBuilder {
    phases: Vec<Phase>,
    signals: Vec<SignalEvent>,
    cursor: u64,
}

impl TimelineBuilder {
    /// Appends a phase at the cursor and returns it.
    fn push_phase(
        &mut self,
        kind: PhaseKind,
        duration: u64,
        set: u32,
        cycle: Option<u32>,
    ) -> Phase {
        let phase = Phase {
            kind,
            start: self.cursor,
            duration,
            set,
            cycle,
        };
        self.cursor += duration;
        self.phases.push(phase);
        phase
    }

    /// Appends a phase together with its countdown cues.
    ///
    /// Cues only survive when their second is inside the phase, so a phase
    /// shorter than three seconds beeps fewer times.
    fn push_countdown_phase(
        &mut self,
        kind: PhaseKind,
        duration: u64,
        set: u32,
        cycle: Option<u32>,
    ) {
        let start = self.cursor;
        let end = start + duration;

        for offset in COUNTDOWN_OFFSETS {
            let inside_phase = end.checked_sub(offset).filter(|at| *at >= start);
            if let Some(at) = inside_phase {
                self.push_signal(Signal::CountdownBeep, at);
            }
        }

        self.push_phase(kind, duration, set, cycle);
    }

    fn push_signal(&mut self, signal: Signal, at: u64) {
        self.signals.push(SignalEvent { signal, at });
    }

    fn finish(mut self) -> Timeline {
        self.signals.sort_by_key(|event| event.at);

        Timeline {
            phases: self.phases,
            signals: self.signals,
            total_duration: self.cursor,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{Phase, PhaseKind, Signal, SignalEvent, Timeline};
    use crate::timer::Timer;

    fn full_program() -> Timer {
        Timer {
            name: String::from("Full"),
            preparation: 10,
            work: 20,
            rest: 10,
            cycles_per_set: 2,
            sets: 2,
            rest_between_sets: 30,
            cool_down: 15,
        }
    }

    fn reference_phases() -> Vec<(PhaseKind, u64, u64, u32, Option<u32>)> {
        vec![
            (PhaseKind::Preparation, 0, 10, 1, None),
            (PhaseKind::Work, 10, 20, 1, Some(1)),
            (PhaseKind::Rest, 30, 10, 1, Some(1)),
            (PhaseKind::Work, 40, 20, 1, Some(2)),
            (PhaseKind::RestBetweenSets, 60, 30, 1, None),
            (PhaseKind::Work, 90, 20, 2, Some(1)),
            (PhaseKind::Rest, 110, 10, 2, Some(1)),
            (PhaseKind::Work, 120, 20, 2, Some(2)),
            (PhaseKind::CoolDown, 140, 15, 2, None),
        ]
    }

    fn occurrences(signals: &[SignalEvent], signal: Signal) -> Vec<u64> {
        signals
            .iter()
            .filter(|event| event.signal == signal)
            .map(|event| event.at)
            .collect()
    }

    fn count_kind(phases: &[Phase], kind: PhaseKind) -> usize {
        phases.iter().filter(|phase| phase.kind == kind).count()
    }

    #[test]
    fn full_program_phase_order() {
        let timeline = Timeline::build(&full_program()).expect("valid timer");

        let table: Vec<_> = timeline
            .phases()
            .iter()
            .map(|phase| {
                (
                    phase.kind,
                    phase.start,
                    phase.duration,
                    phase.set,
                    phase.cycle,
                )
            })
            .collect();

        assert_eq!(table, reference_phases());
        assert_eq!(timeline.total_duration(), 155);
    }

    #[test]
    fn cycle_and_set_counts() {
        let timer = Timer {
            name: String::from("Eight"),
            preparation: 5,
            work: 20,
            rest: 10,
            cycles_per_set: 8,
            sets: 3,
            rest_between_sets: 5,
            cool_down: 0,
        };

        let timeline = Timeline::build(&timer).expect("valid timer");

        assert_eq!(count_kind(timeline.phases(), PhaseKind::Work), 24);
        assert_eq!(count_kind(timeline.phases(), PhaseKind::Rest), 21);
        assert_eq!(count_kind(timeline.phases(), PhaseKind::RestBetweenSets), 2);
        assert_eq!(count_kind(timeline.phases(), PhaseKind::CoolDown), 0);
    }

    #[test]
    fn bell_and_whistle_rules() {
        let timer = Timer {
            name: String::from("Three"),
            preparation: 5,
            work: 20,
            rest: 10,
            cycles_per_set: 3,
            sets: 2,
            rest_between_sets: 30,
            cool_down: 0,
        };

        let timeline = Timeline::build(&timer).expect("valid timer");
        let signals = timeline.signals();

        let work_starts: Vec<u64> = timeline
            .phases()
            .iter()
            .filter(|phase| phase.kind == PhaseKind::Work)
            .map(|phase| phase.start)
            .collect();
        let first_and_second_work_ends: Vec<u64> = timeline
            .phases()
            .iter()
            .filter(|phase| phase.kind == PhaseKind::Work)
            .filter(|phase| phase.cycle != Some(timer.cycles_per_set))
            .map(|phase| phase.start + phase.duration)
            .collect();
        let last_work_ends: Vec<u64> = timeline
            .phases()
            .iter()
            .filter(|phase| phase.kind == PhaseKind::Work)
            .filter(|phase| phase.cycle == Some(timer.cycles_per_set))
            .map(|phase| phase.start + phase.duration)
            .collect();

        assert_eq!(occurrences(signals, Signal::Whistle), work_starts);
        assert_eq!(occurrences(signals, Signal::Whistle).len(), 6);
        assert_eq!(
            occurrences(signals, Signal::Bell),
            first_and_second_work_ends
        );
        assert_eq!(occurrences(signals, Signal::Bell).len(), 4);
        assert_eq!(occurrences(signals, Signal::TripleBell), last_work_ends);
        assert_eq!(occurrences(signals, Signal::TripleBell).len(), 2);
    }

    #[test]
    fn countdown_beeps_limited_to_preparation_and_rest() {
        let timeline = Timeline::build(&full_program()).expect("valid timer");

        assert_eq!(
            occurrences(timeline.signals(), Signal::CountdownBeep),
            vec![7, 8, 9, 37, 38, 39, 117, 118, 119]
        );

        for beep in occurrences(timeline.signals(), Signal::CountdownBeep) {
            let inside_beeping_phase = timeline.phases().iter().any(|phase| {
                let beeps = matches!(phase.kind, PhaseKind::Preparation | PhaseKind::Rest);
                beeps && phase.start <= beep && beep < phase.start + phase.duration
            });

            assert!(
                inside_beeping_phase,
                "countdown beep at {beep} is outside preparation and rest"
            );
        }
    }

    #[test]
    fn short_phase_beeps_only_for_existing_seconds() {
        let timer = Timer {
            name: String::from("Short"),
            preparation: 2,
            work: 20,
            rest: 0,
            cycles_per_set: 1,
            sets: 1,
            rest_between_sets: 0,
            cool_down: 0,
        };

        let timeline = Timeline::build(&timer).expect("valid timer");

        assert_eq!(
            occurrences(timeline.signals(), Signal::CountdownBeep),
            vec![0, 1]
        );
    }

    #[test]
    fn zero_duration_phases_are_skipped() {
        let timer = Timer {
            name: String::from("Back to back"),
            preparation: 0,
            work: 20,
            rest: 0,
            cycles_per_set: 2,
            sets: 1,
            rest_between_sets: 0,
            cool_down: 0,
        };

        let timeline = Timeline::build(&timer).expect("valid timer");
        let phases = timeline.phases();

        assert_eq!(
            phases.first().map(|phase| phase.kind),
            Some(PhaseKind::Work)
        );
        assert_eq!(phases.first().map(|phase| phase.start), Some(0));
        assert_eq!(phases.len(), 2);
        assert_eq!(phases[1].start, phases[0].start + phases[0].duration);

        let signals = timeline.signals();
        assert!(signals.contains(&SignalEvent {
            signal: Signal::Bell,
            at: 20
        }));
        assert!(signals.contains(&SignalEvent {
            signal: Signal::Whistle,
            at: 20
        }));
    }

    #[test]
    fn all_zero_timer_yields_empty_timeline() {
        let timer = Timer {
            name: String::from("Empty"),
            preparation: 0,
            work: 0,
            rest: 0,
            cycles_per_set: 1,
            sets: 1,
            rest_between_sets: 0,
            cool_down: 0,
        };

        let timeline = Timeline::build(&timer).expect("valid timer");

        assert!(timeline.phases().is_empty());
        assert!(timeline.signals().is_empty());
        assert_eq!(timeline.total_duration(), 0);
    }
}
