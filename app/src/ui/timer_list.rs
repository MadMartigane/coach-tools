//! The home screen: the saved timers, each one tap away from a session.

use leptos::prelude::*;
use timer_core::{Timer, format_timer_summary};

use crate::state::AppContext;
use crate::ui::icons::{IconPencil, IconPlus, LogoMark};

/// Lists the saved timers, or invites the user to create the first one.
#[component]
pub fn TimerListScreen() -> impl IntoView {
    let timers = expect_context::<AppContext>().timers;

    view! {
        <main class="screen-neutral">
            <header class="screen-header">
                <h1 class="brand-title">
                    <LogoMark />
                    <span>"TABATA TIMER"</span>
                </h1>
            </header>
            {move || {
                if timers.with(Vec::is_empty) {
                    empty_state().into_any()
                } else {
                    timer_list(timers).into_any()
                }
            }}
        </main>
    }
}

fn empty_state() -> impl IntoView {
    view! {
        <div class="empty-state">
            <p class="empty-state-text">"Aucun timer enregistré."</p>
            {new_timer_button()}
        </div>
    }
}

fn timer_list(timers: RwSignal<Vec<Timer>>) -> impl IntoView {
    let rows = move || {
        timers
            .get()
            .into_iter()
            .enumerate()
            .map(|(index, timer)| timer_row(index, timer))
            .collect_view()
    };
    view! {
        <ul class="timer-list">{rows}</ul>
        {new_timer_button()}
    }
}

fn timer_row(index: usize, timer: Timer) -> impl IntoView {
    let ctx = expect_context::<AppContext>();
    let summary = format_timer_summary(&timer);
    let session_timer = timer.clone();
    let edit_ctx = ctx.clone();
    view! {
        <li class="timer-row">
            <button
                type="button"
                class="timer-row-main"
                on:click=move |_| {
                    if session_timer.total_duration() > 0 {
                        ctx.start_session(session_timer.clone());
                    }
                }
            >
                <span class="timer-row-name">{timer.name}</span>
                <span class="timer-row-summary">{summary}</span>
            </button>
            <button
                type="button"
                class="timer-row-edit"
                on:click=move |_| edit_ctx.open_edit_editor(index)
            >
                <IconPencil />
                <span>"Modifier"</span>
            </button>
        </li>
    }
}

fn new_timer_button() -> impl IntoView {
    let ctx = expect_context::<AppContext>();
    view! {
        <button
            type="button"
            class="btn btn-hero"
            on:click=move |_| ctx.open_new_editor()
        >
            <IconPlus />
            <span>"Nouveau timer"</span>
        </button>
    }
}
