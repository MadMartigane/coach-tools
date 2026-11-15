//! The home screen: the coach-tools mark and one tile per tool.

use leptos::prelude::*;

use crate::state::AppContext;
use crate::ui::icons::{AppMark, LogoMark};

/// Shows the coach-tools header and one tile per tool. Today the only tool is
/// the Tabata timer, whose tile opens its list.
#[component]
pub fn HomeScreen() -> impl IntoView {
    let ctx = expect_context::<AppContext>();

    view! {
        <main class="screen-neutral">
            <header class="screen-header">
                <h1 class="brand-title">
                    <AppMark />
                    <span>"coach-tools"</span>
                </h1>
            </header>
            <ul class="tool-grid">
                <li>
                    <button type="button" class="tool-tile" on:click=move |_| ctx.open_timer_list()>
                        <LogoMark />
                        <span class="tool-tile-title">"TABATA TIMER"</span>
                    </button>
                </li>
            </ul>
        </main>
    }
}
