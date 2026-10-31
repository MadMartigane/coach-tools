//! Synthesizes the session cues with Web Audio, so the app ships no audio files.

use std::cell::RefCell;
use std::rc::Rc;

use leptos::__reexports::send_wrapper::SendWrapper;
use leptos::prelude::window;
use timer_core::Signal;
use wasm_bindgen::JsValue;
use web_sys::{AudioContext, OscillatorType};

/// Seconds the gain envelope takes to rise from silence to the peak gain.
const ATTACK_SECONDS: f64 = 0.01;

/// Silence level for the gain envelope. Exponential ramps cannot reach zero.
const SILENCE_GAIN: f32 = 0.0001;

/// One synthesized tone inside a cue, timed relative to the cue start.
#[derive(Debug, Clone, Copy)]
struct Tone {
    frequency_hz: f32,
    offset_seconds: f64,
    duration_seconds: f64,
    peak_gain: f32,
    wave: OscillatorType,
}

const COUNTDOWN_BEEP_TONES: &[Tone] = &[Tone {
    frequency_hz: 880.0,
    offset_seconds: 0.0,
    duration_seconds: 0.10,
    peak_gain: 0.25,
    wave: OscillatorType::Sine,
}];

const WHISTLE_TONES: &[Tone] = &[
    Tone {
        frequency_hz: 1760.0,
        offset_seconds: 0.0,
        duration_seconds: 0.12,
        peak_gain: 0.35,
        wave: OscillatorType::Sine,
    },
    Tone {
        frequency_hz: 1975.5,
        offset_seconds: 0.12,
        duration_seconds: 0.30,
        peak_gain: 0.35,
        wave: OscillatorType::Sine,
    },
];

const BELL_TONES: &[Tone] = &[Tone {
    frequency_hz: 659.25,
    offset_seconds: 0.0,
    duration_seconds: 0.38,
    peak_gain: 0.35,
    wave: OscillatorType::Triangle,
}];

const TRIPLE_BELL_TONES: &[Tone] = &[
    Tone {
        frequency_hz: 659.25,
        offset_seconds: 0.0,
        duration_seconds: 0.18,
        peak_gain: 0.35,
        wave: OscillatorType::Triangle,
    },
    Tone {
        frequency_hz: 659.25,
        offset_seconds: 0.22,
        duration_seconds: 0.18,
        peak_gain: 0.35,
        wave: OscillatorType::Triangle,
    },
    Tone {
        frequency_hz: 659.25,
        offset_seconds: 0.44,
        duration_seconds: 0.18,
        peak_gain: 0.35,
        wave: OscillatorType::Triangle,
    },
];

/// Plays session cues. The audio context must be created inside a user
/// gesture, so [`SoundPlayer::unlock`] has to run in a Start, Resume or
/// Replay tap handler before [`SoundPlayer::play`] can make sound.
#[derive(Clone)]
pub struct SoundPlayer {
    context: SendWrapper<Rc<RefCell<Option<AudioContext>>>>,
}

impl Default for SoundPlayer {
    fn default() -> Self {
        Self {
            context: SendWrapper::new(Rc::default()),
        }
    }
}

impl SoundPlayer {
    /// Creates a player with no audio context yet.
    pub fn new() -> Self {
        Self::default()
    }

    /// Creates the audio context on first use and resumes it, and asks iOS
    /// to route audio through the playback session so cues play with the
    /// silent switch on.
    pub fn unlock(&self) {
        request_playback_audio_session();
        let mut context_slot = self.context.borrow_mut();
        if context_slot.is_none() {
            *context_slot = AudioContext::new().ok();
        }
        if let Some(context) = context_slot.as_ref() {
            let _ = context.resume();
        }
    }

    /// Schedules the tones of `signal` to play now. Does nothing before
    /// [`SoundPlayer::unlock`] has created an audio context.
    pub fn play(&self, signal: Signal) {
        let context_slot = self.context.borrow();
        let Some(context) = context_slot.as_ref() else {
            return;
        };
        for tone in tones_for(signal) {
            let _ = schedule_tone(context, tone);
        }
    }
}

fn tones_for(signal: Signal) -> &'static [Tone] {
    match signal {
        Signal::CountdownBeep => COUNTDOWN_BEEP_TONES,
        Signal::Whistle => WHISTLE_TONES,
        Signal::Bell => BELL_TONES,
        Signal::TripleBell => TRIPLE_BELL_TONES,
    }
}

/// Sets `navigator.audioSession.type` to `"playback"` when the browser
/// exposes the Audio Session API, and does nothing otherwise.
fn request_playback_audio_session() {
    let navigator = window().navigator();
    let Ok(audio_session) = js_sys::Reflect::get(&navigator, &JsValue::from_str("audioSession"))
    else {
        return;
    };
    if audio_session.is_object() {
        let _ = js_sys::Reflect::set(
            &audio_session,
            &JsValue::from_str("type"),
            &JsValue::from_str("playback"),
        );
    }
}

/// Connects one oscillator through a gain envelope to the output and
/// schedules both to start and stop at the tone's times.
fn schedule_tone(context: &AudioContext, tone: &Tone) -> Result<(), JsValue> {
    let oscillator = context.create_oscillator()?;
    let gain = context.create_gain()?;
    oscillator.set_type(tone.wave);
    oscillator.frequency().set_value(tone.frequency_hz);
    oscillator.connect_with_audio_node(&gain)?;
    gain.connect_with_audio_node(&context.destination())?;

    let start = context.current_time() + tone.offset_seconds;
    let end = start + tone.duration_seconds;
    let envelope = gain.gain();
    envelope.set_value_at_time(SILENCE_GAIN, start)?;
    envelope.linear_ramp_to_value_at_time(tone.peak_gain, start + ATTACK_SECONDS)?;
    envelope.exponential_ramp_to_value_at_time(SILENCE_GAIN, end)?;

    oscillator.start_with_when(start)?;
    oscillator.stop_with_when(end)?;
    Ok(())
}
