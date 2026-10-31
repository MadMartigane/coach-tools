//! Keeps the screen awake during a session through the Screen Wake Lock API.

use std::cell::RefCell;
use std::rc::Rc;

use js_sys::{Array, Function, Object, Promise, Reflect};
use leptos::__reexports::send_wrapper::SendWrapper;
use leptos::prelude::window;
use wasm_bindgen::{JsCast, JsValue};
use wasm_bindgen_futures::{JsFuture, spawn_local};

const WAKE_LOCK_PROPERTY: &str = "wakeLock";
const WAKE_LOCK_REQUEST_METHOD: &str = "request";
const WAKE_LOCK_SCREEN_TYPE: &str = "screen";
const SENTINEL_RELEASE_METHOD: &str = "release";

/// Holds the screen wake lock sentinel while a session runs.
#[derive(Clone)]
pub struct WakeLockGuard {
    sentinel: SendWrapper<Rc<RefCell<Option<Object>>>>,
}

impl Default for WakeLockGuard {
    fn default() -> Self {
        Self {
            sentinel: SendWrapper::new(Rc::default()),
        }
    }
}

impl WakeLockGuard {
    /// Requests a screen wake lock. Unsupported browsers and rejected requests
    /// are ignored, so the session still runs without one.
    pub fn acquire(&self) {
        let Some(wake_lock) = wake_lock_object() else {
            return;
        };
        let Ok(request) = Reflect::get(&wake_lock, &JsValue::from_str(WAKE_LOCK_REQUEST_METHOD))
            .and_then(|value| value.dyn_into::<Function>())
        else {
            return;
        };
        let arguments = Array::of1(&JsValue::from_str(WAKE_LOCK_SCREEN_TYPE));
        let Ok(pending) = Reflect::apply(&request, &wake_lock, &arguments) else {
            return;
        };
        let Ok(promise) = pending.dyn_into::<Promise>() else {
            return;
        };

        let sentinel_slot = self.sentinel.clone();
        spawn_local(async move {
            if let Ok(sentinel) = JsFuture::from(promise).await
                && let Ok(sentinel) = sentinel.dyn_into::<Object>()
            {
                *sentinel_slot.borrow_mut() = Some(sentinel);
            }
        });
    }

    /// Releases the held wake lock, if any.
    pub fn release(&self) {
        let Some(sentinel) = self.sentinel.borrow_mut().take() else {
            return;
        };
        let release = Reflect::get(&sentinel, &JsValue::from_str(SENTINEL_RELEASE_METHOD))
            .and_then(|value| value.dyn_into::<Function>());
        if let Ok(release) = release {
            let _ = Reflect::apply(&release, &sentinel, &Array::new());
        }
    }
}

/// The `navigator.wakeLock` object, or `None` when the browser lacks the API.
fn wake_lock_object() -> Option<JsValue> {
    let navigator = window().navigator();
    let wake_lock = Reflect::get(&navigator, &JsValue::from_str(WAKE_LOCK_PROPERTY)).ok()?;
    wake_lock.is_object().then_some(wake_lock)
}
