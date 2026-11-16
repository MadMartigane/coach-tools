//! The home screen: the coach-tools mark, the language selector, one tile per tool and the app version.

use leptos::prelude::*;

use crate::locale::{ENGLISH_CODE, FRENCH_CODE, locale_code, locale_from_code};
use crate::state::AppContext;
use crate::ui::icons::{AppMark, LogoMark};
use crate::version::version_label;

/// Shows the coach-tools header with the language selector, one tile per tool and
/// the app version line. Today the only tool is the Tabata timer, whose tile opens
/// its list.
#[component]
pub fn HomeScreen() -> impl IntoView {
    let ctx = expect_context::<AppContext>();
    let locale = ctx.locale;
    let language_ctx = ctx.clone();

    view! {
        <main class="screen-neutral">
            <header class="screen-header">
                <h1 class="brand-title">
                    <AppMark />
                    <span>"coach-tools"</span>
                </h1>
                <select
                    class="lang-select"
                    aria-label=move || locale.get().catalog().language
                    on:change=move |event| {
                        if let Some(choice) = locale_from_code(&event_target_value(&event)) {
                            language_ctx.set_locale(choice);
                        }
                    }
                >
                    <option
                        value=FRENCH_CODE
                        lang=FRENCH_CODE
                        prop:selected=move || locale_code(locale.get()) == FRENCH_CODE
                    >
                        "Français"
                    </option>
                    <option
                        value=ENGLISH_CODE
                        lang=ENGLISH_CODE
                        prop:selected=move || locale_code(locale.get()) == ENGLISH_CODE
                    >
                        "English"
                    </option>
                </select>
            </header>
            <ul class="tool-grid">
                <li>
                    <button type="button" class="tool-tile" on:click=move |_| ctx.open_timer_list()>
                        <LogoMark />
                        <span class="tool-tile-title">"TABATA TIMER"</span>
                    </button>
                </li>
            </ul>
            <p class="app-version">{version_label()}</p>
        </main>
    }
}
