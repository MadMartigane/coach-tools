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

/// One whistle motif: two short bursts with rapid interruptions simulated by
/// sequential tones.
const BELL_MOTIF: [Tone; 6] = [
    // First burst
    Tone {
        frequency_hz: 2900.0,
        offset_seconds: 0.0,
        duration_seconds: 0.045,
        peak_gain: 0.38,
        wave: OscillatorType::Sine,
    },
    Tone {
        frequency_hz: 3050.0,
        offset_seconds: 0.04,
        duration_seconds: 0.04,
        peak_gain: 0.36,
        wave: OscillatorType::Sine,
    },
    Tone {
        frequency_hz: 2950.0,
        offset_seconds: 0.075,
        duration_seconds: 0.035,
        peak_gain: 0.32,
        wave: OscillatorType::Sine,
    },
    // Short silence
    // Second burst
    Tone {
        frequency_hz: 3200.0,
        offset_seconds: 0.165,
        duration_seconds: 0.04,
        peak_gain: 0.40,
        wave: OscillatorType::Sine,
    },
    Tone {
        frequency_hz: 3350.0,
        offset_seconds: 0.20,
        duration_seconds: 0.035,
        peak_gain: 0.35,
        wave: OscillatorType::Sine,
    },
    Tone {
        frequency_hz: 3100.0,
        offset_seconds: 0.23,
        duration_seconds: 0.03,
        peak_gain: 0.28,
        wave: OscillatorType::Sine,
    },
];

/// The six-tone double whistle burst every bell cue is built from, ending a work
/// cycle.
const BELL_TONES: &[Tone] = &BELL_MOTIF;

/// Length of one whistle motif, in seconds: the patch ends at 0.23 + 0.03.
const BELL_MOTIF_SECONDS: f64 = 0.26;

/// Silence between the end of one motif and the start of the next, in seconds.
const BELL_MOTIF_GAP_SECONDS: f64 = 0.20;

/// Builds the whistle motif with every tone offset by `offset_seconds`.
const fn bell_motif_at(offset_seconds: f64) -> [Tone; 6] {
    let mut tones = BELL_MOTIF;
    let mut index = 0;
    while index < tones.len() {
        tones[index].offset_seconds += offset_seconds;
        index += 1;
    }
    tones
}

/// Two motifs: the DoubleBell cue, ending the cool-down.
const DOUBLE_BELL_TONES: &[Tone] = &{
    let first = bell_motif_at(0.0);
    let second = bell_motif_at(BELL_MOTIF_SECONDS + BELL_MOTIF_GAP_SECONDS);
    [
        first[0], first[1], first[2], first[3], first[4], first[5], second[0], second[1],
        second[2], second[3], second[4], second[5],
    ]
};

/// Three motifs: the TripleBell cue, ending the last work cycle of the session.
const TRIPLE_BELL_TONES: &[Tone] = &{
    let first = bell_motif_at(0.0);
    let second = bell_motif_at(BELL_MOTIF_SECONDS + BELL_MOTIF_GAP_SECONDS);
    let third = bell_motif_at(2.0 * (BELL_MOTIF_SECONDS + BELL_MOTIF_GAP_SECONDS));
    [
        first[0], first[1], first[2], first[3], first[4], first[5], second[0], second[1],
        second[2], second[3], second[4], second[5], third[0], third[1], third[2], third[3],
        third[4], third[5],
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
        Signal::Bell => BELL_TONES,
        Signal::DoubleBell => DOUBLE_BELL_TONES,
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

#[cfg(test)]
mod tests {
    use super::*;

    /// Tolerance for cue end times, which add f64 offsets and durations.
    const END_TOLERANCE_SECONDS: f64 = 1e-9;

    /// Seconds between the starts of consecutive motifs: one motif plus its gap.
    const BELL_MOTIF_STRIDE_SECONDS: f64 = BELL_MOTIF_SECONDS + BELL_MOTIF_GAP_SECONDS;

    /// The Bell, DoubleBell and TripleBell cue tables.
    fn bell_cues() -> [&'static [Tone]; 3] {
        [BELL_TONES, DOUBLE_BELL_TONES, TRIPLE_BELL_TONES]
    }

    /// Seconds after the cue start when its last tone stops sounding.
    fn cue_end_seconds(tones: &[Tone]) -> f64 {
        tones
            .iter()
            .map(|tone| tone.offset_seconds + tone.duration_seconds)
            .fold(0.0, f64::max)
    }

    #[test]
    fn double_and_triple_bell_tables_hold_two_and_three_motifs() {
        assert_eq!(DOUBLE_BELL_TONES.len(), 12);
        assert_eq!(TRIPLE_BELL_TONES.len(), 18);
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
    fn repeated_bell_motifs_only_shift_the_motif_offsets() {
        let motif_length = BELL_MOTIF.len();
        for tones in [DOUBLE_BELL_TONES, TRIPLE_BELL_TONES] {
            for (index, tone) in tones.iter().enumerate() {
                let motif_tone = BELL_MOTIF[index % motif_length];
                assert_eq!(tone.frequency_hz, motif_tone.frequency_hz);
                assert_eq!(tone.duration_seconds, motif_tone.duration_seconds);
                assert_eq!(tone.peak_gain, motif_tone.peak_gain);
                assert_eq!(tone.wave, motif_tone.wave);
                let expected_offset_seconds = (index / motif_length) as f64
                    * BELL_MOTIF_STRIDE_SECONDS
                    + motif_tone.offset_seconds;
                assert!(
                    (tone.offset_seconds - expected_offset_seconds).abs() < END_TOLERANCE_SECONDS,
                    "tone {index}: expected offset {expected_offset_seconds} s, got {} s",
                    tone.offset_seconds
                );
            }
        }
    }

    #[test]
    fn consecutive_bell_motifs_are_separated_by_silence() {
        for (tones, expected_end_seconds) in [(DOUBLE_BELL_TONES, 0.72), (TRIPLE_BELL_TONES, 1.18)]
        {
            let motifs: Vec<&[Tone]> = tones.chunks(BELL_MOTIF.len()).collect();
            for (index, pair) in motifs.windows(2).enumerate() {
                let previous_last_tone = pair[0][BELL_MOTIF.len() - 1];
                let previous_end_seconds =
                    previous_last_tone.offset_seconds + previous_last_tone.duration_seconds;
                let gap_seconds = pair[1][0].offset_seconds - previous_end_seconds;
                assert!(
                    (gap_seconds - BELL_MOTIF_GAP_SECONDS).abs() < END_TOLERANCE_SECONDS,
                    "motif {}: expected a {BELL_MOTIF_GAP_SECONDS} s gap, got {gap_seconds} s",
                    index + 1
                );
            }
            let actual_end_seconds = cue_end_seconds(tones);
            assert!(
                (actual_end_seconds - expected_end_seconds).abs() < END_TOLERANCE_SECONDS,
                "expected the cue to end at {expected_end_seconds} s, got {actual_end_seconds} s"
            );
        }
    }
}
