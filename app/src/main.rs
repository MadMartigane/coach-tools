mod platform;
mod state;
mod ui;

use leptos::prelude::*;

use state::{AppContext, Screen};
use ui::done::DoneScreen;
use ui::editor::EditorScreen;
use ui::session::SessionScreen;
use ui::timer_list::TimerListScreen;

fn main() {
    leptos::mount::mount_to_body(App);
}

/// Provides the shared state and shows whichever screen the state selects.
#[component]
fn App() -> impl IntoView {
    let ctx = AppContext::new();
    provide_context(ctx.clone());

    move || match ctx.screen.get() {
        Screen::TimerList => view! { <TimerListScreen /> }.into_any(),
        Screen::Editor { index, draft } => {
            view! { <EditorScreen index=index initial=draft /> }.into_any()
        }
        Screen::Session { timer } => view! { <SessionScreen timer=timer /> }.into_any(),
        Screen::Done {
            timer,
            total_duration,
        } => view! { <DoneScreen timer=timer total_duration=total_duration /> }.into_any(),
    }
}
