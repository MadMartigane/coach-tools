//! Keeps the browser chrome color in step with the session state.
//!
//! Colors live only in the `:root` custom properties of `app/styles/main.css`;
//! this module passes around token names and resolves them at runtime.

use leptos::prelude::{document, window};
use timer_core::{PhaseKind, SessionState};

/// Background color of the app shell, also used when no session runs.
pub const THEME_BG: &str = "--color-bg";

const FINISHED_COLOR: &str = "--phase-done-bg";
const PAUSED_COLOR: &str = "--phase-paused-bg";
const PREPARATION_COLOR: &str = "--phase-prep-bg";
const WORK_COLOR: &str = "--phase-work-bg";
const REST_COLOR: &str = "--phase-rest-bg";
const REST_BETWEEN_SETS_COLOR: &str = "--phase-rest-sets-bg";
const COOL_DOWN_COLOR: &str = "--phase-cooldown-bg";

const THEME_COLOR_SELECTOR: &str = "meta[name=\"theme-color\"]";
const THEME_COLOR_ATTRIBUTE: &str = "content";
const CSS_CUSTOM_PROPERTY_PREFIX: &str = "--";

/// Sets the `<meta name="theme-color">` content to `color`.
///
/// `color` is either a CSS custom property name (e.g. `--color-bg`), resolved
/// from the root element's computed style, or a raw CSS color value.
pub fn set_theme_color(color: &str) {
    let Some(resolved_color) = resolve_color(color) else {
        return;
    };
    let Ok(Some(meta)) = document().query_selector(THEME_COLOR_SELECTOR) else {
        return;
    };
    let _ = meta.set_attribute(THEME_COLOR_ATTRIBUTE, &resolved_color);
}

fn resolve_color(color: &str) -> Option<String> {
    let raw_value = if color.starts_with(CSS_CUSTOM_PROPERTY_PREFIX) {
        computed_custom_property(color)?
    } else {
        color.to_owned()
    };
    let trimmed_value = raw_value.trim();
    (!trimmed_value.is_empty()).then(|| trimmed_value.to_owned())
}

/// The computed value of the custom property `name` on the root element, or
/// `None` when the element or the property is unavailable.
fn computed_custom_property(name: &str) -> Option<String> {
    let root = document().document_element()?;
    let style = window().get_computed_style(&root).ok()??;
    style.get_property_value(name).ok()
}

/// The CSS custom property holding the background color of `kind`.
pub fn phase_color_token(kind: PhaseKind) -> &'static str {
    match kind {
        PhaseKind::Preparation => PREPARATION_COLOR,
        PhaseKind::Work => WORK_COLOR,
        PhaseKind::Rest => REST_COLOR,
        PhaseKind::RestBetweenSets => REST_BETWEEN_SETS_COLOR,
        PhaseKind::CoolDown => COOL_DOWN_COLOR,
    }
}

/// The theme color token for a session in `state`, showing `phase` while running.
pub fn theme_color_for_session(state: SessionState, phase: Option<PhaseKind>) -> &'static str {
    match (state, phase) {
        (SessionState::Finished, _) => FINISHED_COLOR,
        (SessionState::Paused, _) => PAUSED_COLOR,
        (SessionState::Running, Some(kind)) => phase_color_token(kind),
        (SessionState::Running, None) => THEME_BG,
    }
}
