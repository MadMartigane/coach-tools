//! The completion screen: the finished program, with Replay and Home.

use leptos::prelude::*;
use timer_core::{SessionState, Timer, format_mm_ss};

use crate::platform::theme::{THEME_BG, set_theme_color, theme_color_for_session};
use crate::state::AppContext;
use crate::ui::icons::{IconCheck, IconHome, IconRotateCcw, LogoMark};

/// Shows that `timer` finished after `total_duration` seconds, and offers to
/// replay it or return to the timer list.
#[component]
pub fn DoneScreen(timer: Timer, total_duration: u64) -> impl IntoView {
    let ctx = expect_context::<AppContext>();
    set_theme_color(theme_color_for_session(SessionState::Finished, None));
    on_cleanup(|| set_theme_color(THEME_BG));

    let replay_ctx = ctx.clone();
    let replay_timer = timer.clone();
    let set_chip = format!("Set {}/{}", timer.sets, timer.sets);

    view! {
        <main class="session-screen phase-done">
            <header class="session-header">
                <LogoMark />
                <span class="session-title">{timer.name.clone()}</span>
                <span class="session-chip">{set_chip}</span>
            </header>
            <div class="session-body">
                <div class="countdown-zone">
                    <span class="done-check">
                        <IconCheck />
                    </span>
                    <p class="phase-title">"Terminé"</p>
                    <p class="countdown-digits">{format_mm_ss(total_duration)}</p>
                </div>
                <div class="session-controls">
                    <button
                        type="button"
                        class="btn-phase-hero"
                        on:click=move |_| replay_ctx.start_session(replay_timer.clone())
                    >
                        <IconRotateCcw />
                        <span>"Rejouer"</span>
                    </button>
                    <button type="button" class="btn-phase-secondary" on:click=move |_| ctx.exit_to_list()>
                        <IconHome />
                        <span>"Accueil"</span>
                    </button>
                </div>
            </div>
        </main>
    }
}
