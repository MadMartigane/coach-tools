//! The active session screen: the countdown of the current phase, the hero
//! Pause/Resume and two-tap Stop controls, and the progress strip.

use std::cell::RefCell;
use std::rc::Rc;
use std::time::Duration;

use leptos::prelude::*;
use timer_core::{
    Phase, PhaseKind, PhaseView, Session, SessionState, Timer, format_duration, format_mm_ss,
    phase_label,
};
use web_sys::VisibilityState;

use crate::platform::audio::SoundPlayer;
use crate::platform::clock::BrowserClock;
use crate::platform::theme::{
    THEME_BG, phase_color_token, set_theme_color, theme_color_for_session,
};
use crate::state::AppContext;
use crate::ui::arm_confirmation;
use crate::ui::icons::{IconPause, IconPlay, IconSquare, LogoMark};

/// How often the session is sampled, so cues and the countdown stay current.
const TICK_INTERVAL_MILLIS: u64 = 250;

/// Phase title shown while the session is paused.
const PAUSED_PHASE_TITLE: &str = "PAUSE";

/// The cycle number shown during the preparation, which precedes cycle 1.
const FIRST_CYCLE: u32 = 1;

/// Reactive copy of the session state, refreshed on every tick.
#[derive(Clone, Copy)]
struct SessionSignals {
    state: RwSignal<SessionState>,
    phase_view: RwSignal<Option<PhaseView>>,
    elapsed: RwSignal<u64>,
    remaining_total: RwSignal<u64>,
}

/// Runs `timer` from the first second, with the countdown of each phase.
#[component]
pub fn SessionScreen(timer: Timer) -> impl IntoView {
    let ctx = expect_context::<AppContext>();
    let session = Rc::new(RefCell::new(
        Session::new(&timer, BrowserClock::new()).expect("saved timers are validated on load"),
    ));
    let phases: Vec<Phase> = session.borrow().timeline().phases().to_vec();
    let total_duration = timer.total_duration();
    let cycles_per_set = timer.cycles_per_set;
    let sets = timer.sets;
    let title = timer.name.clone();

    let signals = SessionSignals {
        state: RwSignal::new(SessionState::Running),
        phase_view: RwSignal::new(None),
        elapsed: RwSignal::new(0),
        remaining_total: RwSignal::new(total_duration),
    };
    let interval = StoredValue::new(None::<IntervalHandle>);
    let visibility_listener = StoredValue::new(None::<WindowListenerHandle>);
    let pending_stop_reset = StoredValue::new(None::<TimeoutHandle>);
    let confirm_stop = RwSignal::new(false);

    let tick = {
        let session = Rc::clone(&session);
        let ctx = ctx.clone();
        let timer = timer.clone();
        Rc::new(move || {
            if advance(&session, &ctx.sound, signals) == SessionState::Finished {
                interval.update_value(|handle| {
                    if let Some(handle) = handle.take() {
                        handle.clear();
                    }
                });
                ctx.finish_session(timer.clone(), total_duration);
            }
        })
    };

    if advance(&session, &ctx.sound, signals) == SessionState::Finished {
        ctx.finish_session(timer.clone(), 0);
    } else {
        let tick_on_interval = Rc::clone(&tick);
        let handle = set_interval_with_handle(
            move || tick_on_interval(),
            Duration::from_millis(TICK_INTERVAL_MILLIS),
        )
        .ok();
        interval.set_value(handle);

        let tick_on_visible = Rc::clone(&tick);
        let wake_lock_ctx = ctx.clone();
        let listener = window_event_listener_untyped("visibilitychange", move |_| {
            let visible = document().visibility_state() == VisibilityState::Visible;
            if visible && signals.state.get_untracked() != SessionState::Finished {
                wake_lock_ctx.wake_lock.acquire();
                tick_on_visible();
            }
        });
        visibility_listener.set_value(Some(listener));
    }

    let toggle_pause = {
        let session = Rc::clone(&session);
        let ctx = ctx.clone();
        let tick = Rc::clone(&tick);
        move |_| {
            let is_paused = session.borrow().state() == SessionState::Paused;
            if is_paused {
                ctx.sound.unlock();
                session.borrow_mut().resume();
            } else {
                session.borrow_mut().pause();
            }
            tick();
        }
    };

    let stop = {
        let ctx = ctx.clone();
        move |_| {
            if confirm_stop.get_untracked() {
                ctx.exit_to_list();
            } else {
                arm_confirmation(confirm_stop, pending_stop_reset);
            }
        }
    };

    on_cleanup(move || {
        interval.update_value(|handle| {
            if let Some(handle) = handle.take() {
                handle.clear();
            }
        });
        visibility_listener.update_value(|listener| {
            if let Some(listener) = listener.take() {
                listener.remove();
            }
        });
        pending_stop_reset.update_value(|pending| {
            if let Some(handle) = pending.take() {
                handle.clear();
            }
        });
        set_theme_color(THEME_BG);
    });

    let phase_title_text = move || phase_title(signals.state.get(), signals.phase_view.get());
    let phase_remaining = move || signals.phase_view.get().map_or(0, |view| view.remaining);
    let set_number = move || signals.phase_view.get().map_or(sets, |view| view.set);
    let cycle_number = move || current_cycle(signals.phase_view.get(), cycles_per_set);

    view! {
        <main class=move || screen_class(signals.state.get(), signals.phase_view.get())>
            <header class="session-header">
                <LogoMark />
                <span class="session-title">{title}</span>
                <span class="session-chip">{move || format!("Set {}/{}", set_number(), sets)}</span>
            </header>
            <div class="session-body">
                <div class="countdown-zone">
                    <p class="phase-title">{phase_title_text}</p>
                    <p class="countdown-digits">{move || format_mm_ss(phase_remaining())}</p>
                </div>
                <div class="session-controls">
                    <button type="button" class="btn-phase-hero" on:click=toggle_pause>
                        {move || {
                            if signals.state.get() == SessionState::Paused {
                                view! { <IconPlay /><span>"Reprendre"</span> }.into_any()
                            } else {
                                view! { <IconPause /><span>"Pause"</span> }.into_any()
                            }
                        }}
                    </button>
                    <button type="button" class="btn-phase-secondary" on:click=stop>
                        {move || {
                            if confirm_stop.get() {
                                view! { <IconSquare /><span>"Confirmer l'arrêt"</span> }.into_any()
                            } else {
                                view! { <IconSquare /><span>"Arrêter"</span> }.into_any()
                            }
                        }}
                    </button>
                </div>
            </div>
            <footer class="progress-strip">
                <div class="progress-row-top">
                    <span>{phase_title_text}</span>
                    <span>{move || format!("Cycle {}/{}", cycle_number(), cycles_per_set)}</span>
                    <span>{move || format!("Set {}/{}", set_number(), sets)}</span>
                </div>
                <div class="progress-track">
                    {phases
                        .into_iter()
                        .map(|phase| progress_segment(phase, signals.elapsed))
                        .collect_view()}
                </div>
                <div class="progress-row-bottom">
                    <div>
                        <p class="progress-metric-value">{move || format_duration(signals.elapsed.get())}</p>
                        <p class="progress-metric-label">"Écoulé"</p>
                    </div>
                    <div>
                        <p class="progress-metric-value">{format_duration(total_duration)}</p>
                        <p class="progress-metric-label">"Total"</p>
                    </div>
                    <div>
                        <p class="progress-metric-value">{move || format_duration(signals.remaining_total.get())}</p>
                        <p class="progress-metric-label">"Restant"</p>
                    </div>
                </div>
            </footer>
        </main>
    }
}

/// Delivers the cues that became due, then publishes the session state to
/// `signals` and to the browser theme color. Returns the state reached.
fn advance(
    session: &RefCell<Session<BrowserClock>>,
    sound: &SoundPlayer,
    signals: SessionSignals,
) -> SessionState {
    let mut session = session.borrow_mut();
    for event in session.poll_signals() {
        sound.play(event.signal);
    }
    let state = session.state();
    let phase = session.current_phase();
    signals.state.set(state);
    signals.phase_view.set(phase);
    signals.elapsed.set(session.elapsed());
    signals.remaining_total.set(session.remaining());
    set_theme_color(theme_color_for_session(state, phase.map(|view| view.kind)));
    state
}

/// The root class of the session screen, which sets the phase colours.
fn screen_class(state: SessionState, phase: Option<PhaseView>) -> &'static str {
    match (state, phase.map(|view| view.kind)) {
        (SessionState::Paused, _) => "session-screen phase-paused",
        (SessionState::Finished, _) => "session-screen phase-done",
        (SessionState::Running, Some(PhaseKind::Preparation)) => "session-screen phase-prep",
        (SessionState::Running, Some(PhaseKind::Work)) => "session-screen phase-work",
        (SessionState::Running, Some(PhaseKind::Rest)) => "session-screen phase-rest",
        (SessionState::Running, Some(PhaseKind::RestBetweenSets)) => {
            "session-screen phase-rest-sets"
        }
        (SessionState::Running, Some(PhaseKind::CoolDown)) => "session-screen phase-cooldown",
        (SessionState::Running, None) => "session-screen",
    }
}

/// The title above the countdown: the phase name, or `PAUSE` while paused.
fn phase_title(state: SessionState, phase: Option<PhaseView>) -> &'static str {
    match state {
        SessionState::Paused => PAUSED_PHASE_TITLE,
        _ => phase.map_or("", |view| phase_label(view.kind)),
    }
}

/// The cycle shown in the progress row. Phases outside the work and rest
/// cycles show the first cycle during the preparation and the last cycle of
/// the set otherwise.
fn current_cycle(phase: Option<PhaseView>, cycles_per_set: u32) -> u32 {
    match phase {
        Some(PhaseView {
            cycle: Some(cycle), ..
        }) => cycle,
        Some(PhaseView {
            kind: PhaseKind::Preparation,
            ..
        }) => FIRST_CYCLE,
        _ => cycles_per_set,
    }
}

/// One flexible track segment, sized by the phase duration and dimmed once
/// the session has run past the end of the phase.
fn progress_segment(phase: Phase, elapsed: RwSignal<u64>) -> impl IntoView {
    let phase_end = phase.start + phase.duration;
    let class = move || {
        if elapsed.get() >= phase_end {
            "progress-segment progress-segment-past"
        } else {
            "progress-segment"
        }
    };
    let style = format!(
        "flex-grow: {}; background: var({});",
        phase.duration,
        phase_color_token(phase.kind)
    );
    view! { <div class=class style=style></div> }
}
