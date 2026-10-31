//! The screens of the app and the icons they share.

use std::time::Duration;

use leptos::prelude::*;

pub mod done;
pub mod editor;
pub mod icons;
pub mod session;
pub mod timer_list;

/// How long a first tap stays armed before a confirming tap is required again.
const CONFIRM_WINDOW_MILLIS: u64 = 3000;

/// Arms a two-tap control: the next tap within the confirmation window
/// performs the action, and the control disarms itself when the window closes.
pub fn arm_confirmation(armed: RwSignal<bool>, pending_reset: StoredValue<Option<TimeoutHandle>>) {
    armed.set(true);
    let reset = move || armed.set(false);
    if let Ok(handle) = set_timeout_with_handle(reset, Duration::from_millis(CONFIRM_WINDOW_MILLIS))
    {
        pending_reset.update_value(|pending| *pending = Some(handle));
    }
}
