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
    /// End of a work cycle that is not the last one of the session.
    Bell,
    /// End of the cool-down.
    DoubleBell,
    /// End of the last work cycle of the session.
    TripleBell,
    /// One second before a work cycle starts or ends.
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

/// The countdown fires three, two and one second before a work cycle starts.
const COUNTDOWN_OFFSETS: [u64; 3] = [3, 2, 1];

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
            timeline.push_phase(
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
                    let is_last_work_cycle = set == timer.sets && cycle == timer.cycles_per_set;
                    timeline.push_work_cycle_cues(work.start, work.end(), is_last_work_cycle);
                }

                let rests_before_next_cycle = cycle < timer.cycles_per_set;
                if rests_before_next_cycle && timer.rest > 0 {
                    timeline.push_phase(PhaseKind::Rest, timer.rest.into(), set, Some(cycle));
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
                let cool_down =
                    timeline.push_phase(PhaseKind::CoolDown, timer.cool_down.into(), set, None);
                timeline.push_signal(Signal::DoubleBell, cool_down.end());
            }
        }

        let total_duration = timer.total_duration();
        if timer.work == 0 && timer.cool_down == 0 && total_duration > 0 {
            timeline.push_signal(Signal::TripleBell, total_duration);
        }

        Ok(timeline.finish())
    }

    /// The session phases, in chronological order.
    pub fn phases(&self) -> &[Phase] {
        &self.phases
    }

    /// The scheduled cues, sorted by `at`; the same cue on the same second
    /// appears once.
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

    /// Appends the countdown beeps, the whistle and the end bell of one work
    /// cycle.
    ///
    /// Pre-beeps only survive when their second is not before the session, so
    /// a work cycle starting in the first seconds beeps fewer times.
    fn push_work_cycle_cues(&mut self, start: u64, end: u64, is_last_work_cycle: bool) {
        for offset in COUNTDOWN_OFFSETS {
            if let Some(at) = start.checked_sub(offset) {
                self.push_signal(Signal::CountdownBeep, at);
            }
        }

        self.push_signal(Signal::Whistle, start);
        self.push_signal(Signal::CountdownBeep, end - 1);

        let end_bell = if is_last_work_cycle {
            Signal::TripleBell
        } else {
            Signal::Bell
        };
        self.push_signal(end_bell, end);
    }

    fn push_signal(&mut self, signal: Signal, at: u64) {
        self.signals.push(SignalEvent { signal, at });
    }

    fn finish(mut self) -> Timeline {
        self.signals.sort_by_key(|event| event.at);
        self.signals
            .dedup_by(|kept, duplicate| kept.at == duplicate.at && kept.signal == duplicate.signal);

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

        assert_eq!(
            occurrences(signals, Signal::Whistle),
            vec![5, 35, 65, 115, 145, 175]
        );
        // Every work end but the last one of the session.
        assert_eq!(
            occurrences(signals, Signal::Bell),
            vec![25, 55, 85, 135, 165]
        );
        assert_eq!(occurrences(signals, Signal::Bell).len(), 5);
        assert!(occurrences(signals, Signal::DoubleBell).is_empty());
        assert_eq!(occurrences(signals, Signal::TripleBell), vec![195]);
        assert_eq!(timeline.total_duration(), 195);
    }

    #[test]
    fn countdown_beeps_around_every_work_cycle() {
        let timeline = Timeline::build(&full_program()).expect("valid timer");

        // Work starts at 10, 40, 90 and 120; work ends at 30, 60, 110 and 140.
        assert_eq!(
            occurrences(timeline.signals(), Signal::CountdownBeep),
            vec![
                7, 8, 9, 29, 37, 38, 39, 59, 87, 88, 89, 109, 117, 118, 119, 139
            ]
        );
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
        let signals = timeline.signals();

        assert_eq!(occurrences(signals, Signal::CountdownBeep), vec![0, 1, 21]);
        assert!(occurrences(signals, Signal::Bell).is_empty());
        assert_eq!(occurrences(signals, Signal::TripleBell), vec![22]);
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

        assert!(occurrences(signals, Signal::DoubleBell).is_empty());
        assert_eq!(
            occurrences(signals, Signal::CountdownBeep),
            vec![17, 18, 19, 39]
        );
        assert_eq!(occurrences(signals, Signal::TripleBell), vec![40]);
    }

    #[test]
    fn one_second_rest_still_gets_three_pre_beeps() {
        let timer = Timer {
            name: String::from("One second rest"),
            preparation: 0,
            work: 5,
            rest: 1,
            cycles_per_set: 2,
            sets: 1,
            rest_between_sets: 0,
            cool_down: 0,
        };

        let timeline = Timeline::build(&timer).expect("valid timer");
        let signals = timeline.signals();

        assert_eq!(
            occurrences(signals, Signal::CountdownBeep),
            vec![3, 4, 5, 10]
        );
        assert_eq!(occurrences(signals, Signal::Whistle), vec![0, 6]);
        assert_eq!(occurrences(signals, Signal::Bell), vec![5]);
        assert_eq!(occurrences(signals, Signal::TripleBell), vec![11]);
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

    #[test]
    fn session_end_fires_once_at_real_end_after_cooldown() {
        let timeline = Timeline::build(&full_program()).expect("valid timer");
        let signals = timeline.signals();

        // The last work cycle ends at 140 and the cool-down ends at 155.
        assert_eq!(occurrences(signals, Signal::DoubleBell), vec![155]);
        assert_eq!(occurrences(signals, Signal::TripleBell), vec![140]);
        assert_eq!(timeline.total_duration(), 155);
    }

    #[test]
    fn single_set_without_cool_down_skips_set_end() {
        let timeline = Timeline::build(&Timer::default_new()).expect("valid timer");
        let signals = timeline.signals();

        assert!(occurrences(signals, Signal::DoubleBell).is_empty());
        assert_eq!(occurrences(signals, Signal::Bell).len(), 7);
        assert_eq!(occurrences(signals, Signal::TripleBell), vec![240]);
    }
}
