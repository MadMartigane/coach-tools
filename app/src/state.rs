//! Application state shared by every screen: the saved timers, the current
//! screen and the browser services a session needs.

use leptos::prelude::*;
use timer_core::{InvalidTimer, Timer};

use crate::platform::audio::SoundPlayer;
use crate::platform::storage::{load_timers, save_timers};
use crate::platform::wake_lock::WakeLockGuard;

/// The screen currently shown, with the data that screen needs.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Screen {
    /// The list of saved timers.
    TimerList,
    /// The editor for a new timer (`index` is `None`) or a saved one.
    Editor { index: Option<usize>, draft: Timer },
    /// A running session of `timer`.
    Session { timer: Timer },
    /// The completed session of `timer`, which lasted `total_duration` seconds.
    Done { timer: Timer, total_duration: u64 },
}

/// Shared application state, provided as context to every screen.
#[derive(Clone)]
pub struct AppContext {
    pub timers: RwSignal<Vec<Timer>>,
    pub screen: RwSignal<Screen>,
    pub sound: SoundPlayer,
    pub wake_lock: WakeLockGuard,
}

impl AppContext {
    /// Loads the saved timers and starts on the timer list.
    pub fn new() -> Self {
        Self {
            timers: RwSignal::new(load_timers()),
            screen: RwSignal::new(Screen::TimerList),
            sound: SoundPlayer::new(),
            wake_lock: WakeLockGuard::default(),
        }
    }

    /// Opens the editor on a new timer with the default program.
    pub fn open_new_editor(&self) {
        self.screen.set(Screen::Editor {
            index: None,
            draft: Timer::default_new(),
        });
    }

    /// Opens the editor on the saved timer at `index`.
    pub fn open_edit_editor(&self, index: usize) {
        let draft = self.timers.get_untracked()[index].clone();
        self.screen.set(Screen::Editor {
            index: Some(index),
            draft,
        });
    }

    /// Stores `timer` at `index` (or appends it when `index` is `None`),
    /// persists the list and returns to the timer list. Fails without
    /// changing anything when the timer is invalid.
    pub fn save_editor(&self, index: Option<usize>, mut timer: Timer) -> Result<(), InvalidTimer> {
        timer.name = timer.name.trim().to_string();
        timer.validate()?;
        self.timers.update(|timers| match index {
            Some(position) => timers[position] = timer,
            None => timers.push(timer),
        });
        save_timers(&self.timers.get_untracked());
        self.screen.set(Screen::TimerList);
        Ok(())
    }

    /// Removes the saved timer at `index`, persists the list and returns to
    /// the timer list.
    pub fn delete_timer(&self, index: usize) {
        self.timers.update(|timers| {
            timers.remove(index);
        });
        save_timers(&self.timers.get_untracked());
        self.screen.set(Screen::TimerList);
    }

    /// Unlocks sound, keeps the screen awake and starts a session of `timer`.
    pub fn start_session(&self, timer: Timer) {
        self.sound.unlock();
        self.wake_lock.acquire();
        self.screen.set(Screen::Session { timer });
    }

    /// Ends a session and shows the done screen.
    pub fn finish_session(&self, timer: Timer, total_duration: u64) {
        self.wake_lock.release();
        self.screen.set(Screen::Done {
            timer,
            total_duration,
        });
    }

    /// Stops any session and returns to the timer list.
    pub fn exit_to_list(&self) {
        self.wake_lock.release();
        self.screen.set(Screen::TimerList);
    }
}
