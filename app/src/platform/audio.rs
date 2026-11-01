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

/// Frequency of the prime partial of every bell strike, in hertz.
const BELL_PRIME_HZ: f32 = 659.25;

/// Number of inharmonic partials in one bell strike.
const BELL_PARTIAL_COUNT: usize = 5;

/// How one partial of a bell strike scales against the strike it belongs to.
#[derive(Debug, Clone, Copy)]
struct BellPartial {
    frequency_multiplier: f32,
    duration_multiplier: f64,
    peak_multiplier: f32,
}

/// The five inharmonic partials of a bell, in order: hum, prime, tierce, quint and
/// nominal. Upper partials are shorter and quieter, so the low hum rings longest.
const BELL_PARTIALS: [BellPartial; BELL_PARTIAL_COUNT] = [
    BellPartial {
        frequency_multiplier: 0.5,
        duration_multiplier: 1.2,
        peak_multiplier: 0.5,
    },
    BellPartial {
        frequency_multiplier: 1.0,
        duration_multiplier: 1.0,
        peak_multiplier: 1.0,
    },
    BellPartial {
        frequency_multiplier: 1.2,
        duration_multiplier: 0.8,
        peak_multiplier: 0.4,
    },
    BellPartial {
        frequency_multiplier: 1.5,
        duration_multiplier: 0.6,
        peak_multiplier: 0.3,
    },
    BellPartial {
        frequency_multiplier: 2.0,
        duration_multiplier: 0.5,
        peak_multiplier: 0.25,
    },
];

/// Builds one bell strike: the five partials of [`BELL_PARTIALS`], all starting at
/// `offset_seconds`, each scaled from the strike's `duration_seconds` and `peak_gain`.
const fn bell_strike(
    offset_seconds: f64,
    duration_seconds: f64,
    peak_gain: f32,
) -> [Tone; BELL_PARTIAL_COUNT] {
    let strike = Tone {
        frequency_hz: BELL_PRIME_HZ,
        offset_seconds,
        duration_seconds,
        peak_gain,
        wave: OscillatorType::Sine,
    };
    let mut partials = [strike; BELL_PARTIAL_COUNT];
    let mut index = 0;
    while index < BELL_PARTIAL_COUNT {
        let scaling = BELL_PARTIALS[index];
        partials[index].frequency_hz = BELL_PRIME_HZ * scaling.frequency_multiplier;
        partials[index].duration_seconds = duration_seconds * scaling.duration_multiplier;
        partials[index].peak_gain = peak_gain * scaling.peak_multiplier;
        index += 1;
    }
    partials
}

/// Length of a Bell strike and of each DoubleBell strike, in seconds.
const SHORT_STRIKE_SECONDS: f64 = 0.35;

/// Length of each TripleBell strike, in seconds.
const LONG_STRIKE_SECONDS: f64 = 0.55;

/// Peak gain of a strike's prime partial, shared by every bell strike.
const STRIKE_PEAK_GAIN: f32 = 0.18;

/// Gap between the two DoubleBell strikes, in seconds.
const DOUBLE_BELL_SPACING_SECONDS: f64 = 0.30;

/// Gap between the three TripleBell strikes, in seconds.
const TRIPLE_BELL_SPACING_SECONDS: f64 = 0.45;

/// One short strike: the Bell cue.
const BELL_TONES: [Tone; BELL_PARTIAL_COUNT] =
    bell_strike(0.0, SHORT_STRIKE_SECONDS, STRIKE_PEAK_GAIN);

/// Two short strikes: the DoubleBell cue, marking a set the session survives.
const DOUBLE_BELL_TONES: [Tone; 2 * BELL_PARTIAL_COUNT] = {
    let first = bell_strike(0.0, SHORT_STRIKE_SECONDS, STRIKE_PEAK_GAIN);
    let second = bell_strike(
        DOUBLE_BELL_SPACING_SECONDS,
        SHORT_STRIKE_SECONDS,
        STRIKE_PEAK_GAIN,
    );
    [
        first[0], first[1], first[2], first[3], first[4], second[0], second[1], second[2],
        second[3], second[4],
    ]
};

/// Three long strikes: the TripleBell cue, marking the end of the session.
const TRIPLE_BELL_TONES: [Tone; 3 * BELL_PARTIAL_COUNT] = {
    let first = bell_strike(0.0, LONG_STRIKE_SECONDS, STRIKE_PEAK_GAIN);
    let second = bell_strike(
        TRIPLE_BELL_SPACING_SECONDS,
        LONG_STRIKE_SECONDS,
        STRIKE_PEAK_GAIN,
    );
    let third = bell_strike(
        2.0 * TRIPLE_BELL_SPACING_SECONDS,
        LONG_STRIKE_SECONDS,
        STRIKE_PEAK_GAIN,
    );
    [
        first[0], first[1], first[2], first[3], first[4], second[0], second[1], second[2],
        second[3], second[4], third[0], third[1], third[2], third[3], third[4],
    ]
};

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
        Signal::Bell => &BELL_TONES,
        Signal::DoubleBell => &DOUBLE_BELL_TONES,
        Signal::TripleBell => &TRIPLE_BELL_TONES,
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

#[cfg(test)]
mod tests {
    use super::*;

    /// Partial frequencies of one bell strike in hum, prime, tierce, quint, nominal order.
    const EXPECTED_STRIKE_HZ: [f32; BELL_PARTIAL_COUNT] = [329.625, 659.25, 791.1, 988.875, 1318.5];

    /// Tolerance for partial frequencies, which are computed in f32.
    const FREQUENCY_TOLERANCE_HZ: f32 = 0.001;

    /// Tolerance for cue end times, which add f64 offsets and durations.
    const END_TOLERANCE_SECONDS: f64 = 1e-9;

    /// Peak gain one strike may use, leaving headroom for the other cues.
    const PEAK_HEADROOM: f32 = 0.5;

    /// Shortest acceptable session-end cue, in seconds.
    const SESSION_END_MIN_SECONDS: f64 = 1.5;

    /// Every bell-family cue table, so each rule is checked for all three.
    fn bell_cues() -> [&'static [Tone]; 3] {
        [&BELL_TONES, &DOUBLE_BELL_TONES, &TRIPLE_BELL_TONES]
    }

    #[test]
    fn bell_tables_hold_one_two_and_three_strikes() {
        assert_eq!(BELL_TONES.len(), 5);
        assert_eq!(DOUBLE_BELL_TONES.len(), 10);
        assert_eq!(TRIPLE_BELL_TONES.len(), 15);
    }

    #[test]
    fn bell_tables_are_sorted_by_offset() {
        for tones in bell_cues() {
            assert!(
                tones
                    .windows(2)
                    .all(|pair| pair[0].offset_seconds <= pair[1].offset_seconds)
            );
        }
    }

    #[test]
    fn first_triple_strike_has_exactly_the_bell_partials() {
        let frequencies: Vec<f32> = TRIPLE_BELL_TONES[..BELL_PARTIAL_COUNT]
            .iter()
            .map(|tone| tone.frequency_hz)
            .collect();
        assert_eq!(frequencies.len(), EXPECTED_STRIKE_HZ.len());
        for (actual, expected) in frequencies.into_iter().zip(EXPECTED_STRIKE_HZ) {
            assert!(
                (actual - expected).abs() < FREQUENCY_TOLERANCE_HZ,
                "expected {expected} Hz, got {actual} Hz"
            );
        }
    }

    #[test]
    fn every_strike_stays_under_peak_gain_headroom() {
        for tones in bell_cues() {
            for strike in tones.chunks(BELL_PARTIAL_COUNT) {
                let peak_sum: f32 = strike.iter().map(|tone| tone.peak_gain).sum();
                assert!(peak_sum <= PEAK_HEADROOM, "strike peaks sum to {peak_sum}");
            }
        }
    }

    #[test]
    fn triple_bell_tail_outlasts_the_strike_spacing() {
        let cue_end_seconds = TRIPLE_BELL_TONES
            .iter()
            .map(|tone| tone.offset_seconds + tone.duration_seconds)
            .fold(0.0, f64::max);
        let last_strike_offset_seconds = 2.0 * TRIPLE_BELL_SPACING_SECONDS;
        let hum_tail_seconds = LONG_STRIKE_SECONDS * BELL_PARTIALS[0].duration_multiplier;
        assert!(cue_end_seconds >= SESSION_END_MIN_SECONDS);
        assert!(
            (cue_end_seconds - (last_strike_offset_seconds + hum_tail_seconds)).abs()
                < END_TOLERANCE_SECONDS
        );
    }
}
