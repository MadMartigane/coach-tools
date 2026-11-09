//! The session engine: current phase, remaining time and signal delivery.

use crate::clock::Clock;
use crate::timeline::{PhaseKind, SignalEvent, Timeline};
use crate::timer::{InvalidTimer, Timer};

/// Where a session currently stands.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SessionState {
    /// Time advances and cues become due.
    Running,
    /// Time is frozen where it was when the session paused.
    Paused,
    /// The whole timeline has elapsed.
    Finished,
}

/// What the user sees of the current phase.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PhaseView {
    /// Which part of the session the phase is.
    pub kind: PhaseKind,
    /// Whole seconds left in the phase.
    pub remaining: u64,
    /// 1-based set the phase belongs to.
    pub set: u32,
    /// 1-based cycle within the set, when the phase has one.
    pub cycle: Option<u32>,
}

/// Runs a [`Timeline`] against a [`Clock`], with pause and resume.
pub struct Session<C: Clock> {
    timeline: Timeline,
    clock: C,
    pause_started_at: Option<u64>,
    paused_seconds: u64,
    next_signal: usize,
}

impl<C: Clock> Session<C> {
    /// Builds the session of `timer`, starting the clock at session time zero
    /// as if the user had just tapped Start.
    pub fn new(timer: &Timer, clock: C) -> Result<Self, InvalidTimer> {
        Ok(Self {
            timeline: Timeline::build(timer)?,
            clock,
            pause_started_at: None,
            paused_seconds: 0,
            next_signal: 0,
        })
    }

    /// Where the session currently stands.
    pub fn state(&self) -> SessionState {
        if self.session_time() >= self.timeline.total_duration() {
            SessionState::Finished
        } else if self.pause_started_at.is_some() {
            SessionState::Paused
        } else {
            SessionState::Running
        }
    }

    /// Freezes the session at its current time; any other state ignores it.
    pub fn pause(&mut self) {
        if self.state() == SessionState::Running {
            self.pause_started_at = Some(self.clock.now());
        }
    }

    /// Continues a paused session from the second it was paused; any other
    /// state ignores it.
    pub fn resume(&mut self) {
        if let Some(pause_started_at) = self.pause_started_at {
            self.paused_seconds += self.clock.now().saturating_sub(pause_started_at);
            self.pause_started_at = None;
        }
    }

    /// The phase in progress, or `None` once the session is finished.
    pub fn current_phase(&self) -> Option<PhaseView> {
        if self.state() == SessionState::Finished {
            return None;
        }

        let now = self.session_time();
        let phase = self
            .timeline
            .phases()
            .iter()
            .find(|phase| phase.start <= now && now < phase.start + phase.duration)?;

        Some(PhaseView {
            kind: phase.kind,
            remaining: phase.start + phase.duration - now,
            set: phase.set,
            cycle: phase.cycle,
        })
    }

    /// Whole seconds the session has run, never past the timeline total.
    pub fn elapsed(&self) -> u64 {
        self.session_time().min(self.timeline.total_duration())
    }

    /// Whole seconds left of the whole session.
    pub fn remaining(&self) -> u64 {
        self.timeline.total_duration() - self.elapsed()
    }

    /// Every cue that became due and was not delivered yet, in time order.
    pub fn poll_signals(&mut self) -> Vec<SignalEvent> {
        let now = self.session_time();
        let start = self.next_signal;
        let due = self
            .timeline
            .signals()
            .iter()
            .skip(start)
            .take_while(|event| event.at <= now)
            .count();
        let end = start + due;

        self.next_signal = end;
        self.timeline.signals()[start..end].to_vec()
    }

    /// The phases and cues this session runs through.
    pub fn timeline(&self) -> &Timeline {
        &self.timeline
    }

    /// Session time: clock time minus every second spent paused, frozen while
    /// the session is paused.
    fn session_time(&self) -> u64 {
        let instant = self.pause_started_at.unwrap_or_else(|| self.clock.now());
        instant.saturating_sub(self.paused_seconds)
    }
}

#[cfg(test)]
mod tests {
    use super::{PhaseView, Session, SessionState};
    use crate::clock::SimulatedClock;
    use crate::timeline::{PhaseKind, Signal, SignalEvent};
    use crate::timer::Timer;
    use proptest::prelude::*;

    fn walk_timer() -> Timer {
        Timer {
            name: String::from("Walk"),
            preparation: 10,
            work: 20,
            rest: 10,
            cycles_per_set: 1,
            sets: 1,
            rest_between_sets: 0,
            cool_down: 15,
        }
    }

    fn empty_timer() -> Timer {
        Timer {
            name: String::from("Empty"),
            preparation: 0,
            work: 0,
            rest: 0,
            cycles_per_set: 1,
            sets: 1,
            rest_between_sets: 0,
            cool_down: 0,
        }
    }

    fn countdown_beep(at: u64) -> SignalEvent {
        SignalEvent {
            signal: Signal::CountdownBeep,
            at,
        }
    }

    fn whistle(at: u64) -> SignalEvent {
        SignalEvent {
            signal: Signal::Whistle,
            at,
        }
    }

    fn double_bell(at: u64) -> SignalEvent {
        SignalEvent {
            signal: Signal::DoubleBell,
            at,
        }
    }

    fn triple_bell(at: u64) -> SignalEvent {
        SignalEvent {
            signal: Signal::TripleBell,
            at,
        }
    }

    fn preparation_view(remaining: u64) -> PhaseView {
        PhaseView {
            kind: PhaseKind::Preparation,
            remaining,
            set: 1,
            cycle: None,
        }
    }

    fn work_view(remaining: u64) -> PhaseView {
        PhaseView {
            kind: PhaseKind::Work,
            remaining,
            set: 1,
            cycle: Some(1),
        }
    }

    #[test]
    fn walks_full_program_with_clock() {
        let clock = SimulatedClock::new();
        let mut session = Session::new(&walk_timer(), clock.clone()).expect("valid walk timer");

        assert_eq!(session.state(), SessionState::Running);
        assert_eq!(session.current_phase(), Some(preparation_view(10)));

        clock.advance(7);
        assert_eq!(session.poll_signals(), vec![countdown_beep(7)]);

        clock.advance(2);
        assert_eq!(
            session.poll_signals(),
            vec![countdown_beep(8), countdown_beep(9)]
        );

        clock.advance(1);
        assert_eq!(session.current_phase(), Some(work_view(20)));
        assert_eq!(session.poll_signals(), vec![whistle(10)]);

        clock.advance(20);
        assert_eq!(
            session.current_phase(),
            Some(PhaseView {
                kind: PhaseKind::CoolDown,
                remaining: 15,
                set: 1,
                cycle: None,
            })
        );
        assert_eq!(
            session.poll_signals(),
            vec![countdown_beep(29), triple_bell(30)]
        );

        clock.advance(15);
        assert_eq!(session.poll_signals(), vec![double_bell(45)]);
        assert_eq!(session.state(), SessionState::Finished);
        assert_eq!(session.current_phase(), None);
        assert_eq!(session.elapsed(), 45);
        assert_eq!(session.remaining(), 0);
    }

    #[test]
    fn pause_freezes_remaining_and_signals() {
        let clock = SimulatedClock::new();
        let mut session = Session::new(&walk_timer(), clock.clone()).expect("valid walk timer");

        clock.advance(15);
        // The beeps at 7, 8, 9 and the whistle at 10 fall before the pause.
        assert_eq!(session.poll_signals().len(), 4);

        session.pause();
        clock.advance(100);

        assert_eq!(session.state(), SessionState::Paused);
        assert_eq!(session.current_phase(), Some(work_view(15)));
        assert_eq!(session.elapsed(), 15);
        assert!(session.poll_signals().is_empty());

        session.resume();
        assert_eq!(session.state(), SessionState::Running);

        clock.advance(15);
        assert_eq!(
            session.current_phase().map(|phase| phase.kind),
            Some(PhaseKind::CoolDown)
        );
    }

    #[test]
    fn finished_session_reports_totals_and_no_phase() {
        let empty_clock = SimulatedClock::new();
        let mut empty = Session::new(&empty_timer(), empty_clock).expect("valid empty timer");

        assert_eq!(empty.state(), SessionState::Finished);
        assert_eq!(empty.current_phase(), None);
        assert_eq!(empty.elapsed(), 0);
        assert!(empty.poll_signals().is_empty());

        let clock = SimulatedClock::new();
        let session = Session::new(&walk_timer(), clock.clone()).expect("valid walk timer");
        clock.advance(60);

        assert_eq!(session.elapsed(), 45);
        assert_eq!(session.remaining(), 0);
        assert_eq!(session.current_phase(), None);
        assert_eq!(session.state(), SessionState::Finished);
    }

    fn valid_timer() -> impl Strategy<Value = Timer> {
        (
            0u32..=600,
            0u32..=600,
            0u32..=600,
            1u32..=20,
            1u32..=10,
            0u32..=600,
            0u32..=600,
        )
            .prop_map(
                |(preparation, work, rest, cycles_per_set, sets, rest_between_sets, cool_down)| {
                    Timer {
                        name: String::from("Property"),
                        preparation,
                        work,
                        rest,
                        cycles_per_set,
                        sets,
                        rest_between_sets,
                        cool_down,
                    }
                },
            )
    }

    fn assert_within_total(
        session: &Session<SimulatedClock>,
        total: u64,
    ) -> Result<(), TestCaseError> {
        prop_assert!(session.remaining() <= total);
        prop_assert_eq!(session.elapsed() + session.remaining(), total);
        if let Some(view) = session.current_phase() {
            prop_assert!(1 <= view.remaining);
            prop_assert!(view.remaining <= session.remaining());
        }
        Ok(())
    }

    fn assert_finished(session: &Session<SimulatedClock>, total: u64) -> Result<(), TestCaseError> {
        prop_assert_eq!(session.state(), SessionState::Finished);
        prop_assert_eq!(session.current_phase(), None);
        prop_assert_eq!(session.remaining(), 0);
        prop_assert_eq!(session.elapsed(), total);
        Ok(())
    }

    proptest! {
        #[test]
        fn remaining_stays_within_total_duration(
            timer in valid_timer(),
            advances in prop::collection::vec(0u64..=900, 0..40),
        ) {
            let clock = SimulatedClock::new();
            let session = Session::new(&timer, clock.clone()).expect("valid generated timer");
            let total = timer.total_duration();

            assert_within_total(&session, total)?;
            for step in advances {
                clock.advance(step);
                assert_within_total(&session, total)?;
            }
        }

        #[test]
        fn remaining_never_increases_as_clock_advances(
            timer in valid_timer(),
            steps in prop::collection::vec((0u64..=900, any::<bool>()), 1..40),
        ) {
            let clock = SimulatedClock::new();
            let mut session = Session::new(&timer, clock.clone()).expect("valid generated timer");

            for (seconds, toggle) in steps {
                if toggle {
                    let before_toggle = session.remaining();
                    if session.state() == SessionState::Paused {
                        session.resume();
                    } else {
                        session.pause();
                    }
                    prop_assert_eq!(session.remaining(), before_toggle);
                }

                let before = session.remaining();
                let was_paused = session.state() == SessionState::Paused;
                clock.advance(seconds);
                prop_assert!(session.remaining() <= before);
                if was_paused {
                    prop_assert_eq!(session.remaining(), before);
                }
            }
        }

        #[test]
        fn finished_session_stays_finished(
            timer in valid_timer(),
            overshoot in prop_oneof![Just(0u64), 1u64..=900],
            later in prop::collection::vec((0u64..=900, any::<bool>()), 0..20),
        ) {
            let clock = SimulatedClock::new();
            let mut session = Session::new(&timer, clock.clone()).expect("valid generated timer");
            let total = timer.total_duration();

            clock.advance(total + overshoot);
            session.poll_signals();
            assert_finished(&session, total)?;

            for (seconds, toggle) in later {
                if toggle {
                    session.pause();
                } else {
                    session.resume();
                }
                clock.advance(seconds);
                assert_finished(&session, total)?;
                prop_assert!(session.poll_signals().is_empty());
            }
        }

        #[test]
        fn timeline_phases_add_up_to_total_duration(timer in valid_timer()) {
            let clock = SimulatedClock::new();
            let session = Session::new(&timer, clock.clone()).expect("valid generated timer");
            let total = timer.total_duration();
            let timeline = session.timeline();

            let mut expected_start = 0;
            for phase in timeline.phases() {
                prop_assert_eq!(phase.start, expected_start);
                expected_start = phase.start + phase.duration;
            }
            let summed_duration: u64 = timeline.phases().iter().map(|phase| phase.duration).sum();
            prop_assert_eq!(summed_duration, timeline.total_duration());
            prop_assert_eq!(timeline.total_duration(), total);
            prop_assert_eq!(session.remaining(), total);
        }
    }
}
