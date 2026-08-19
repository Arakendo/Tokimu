//! A tiny bounded oscillator provider for corpus music evidence.

use audio_tools::{
    AudioValueError, NoteSequence, PcmClip, PcmClipLimits, SequenceControl, SequenceEventKind,
};
use thiserror::Error;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SimpleSynthConfig {
    pub sample_rate_hz: u32,
    pub render_time_units: u64,
    pub maximum_frames: usize,
    pub maximum_voices: usize,
    pub master_gain: f32,
}

#[derive(Clone, Debug, PartialEq)]
pub struct SimpleSynthesis {
    pub clip: PcmClip,
    pub rendered_time_units: u64,
    pub dispatched_events: usize,
    pub maximum_active_voices: usize,
    pub voice_steals: usize,
    pub substituted_instruments: usize,
    pub ignored_controls: usize,
    pub peak: f32,
    pub clipped_samples: usize,
    pub sample_fingerprint: u64,
}

#[derive(Clone, Debug, Error, PartialEq)]
pub enum SimpleSynthError {
    #[error("simple synth sample rate must be nonzero")]
    ZeroSampleRate,
    #[error("simple synth maximum voice count must be nonzero")]
    ZeroVoiceLimit,
    #[error("simple synth master gain must be finite and in [0, 1]")]
    InvalidMasterGain,
    #[error("simple synth frame count {frames} exceeds limit {limit}")]
    FrameLimitExceeded { frames: usize, limit: usize },
    #[error("simple synth PCM validation failed: {0}")]
    InvalidPcm(#[from] AudioValueError),
    #[error("PCM16 WAVE artifact exceeds the 32-bit RIFF size limit")]
    WaveArtifactTooLarge,
}

pub fn encode_pcm16_wave(clip: &PcmClip) -> Result<Vec<u8>, SimpleSynthError> {
    let sample_bytes = clip
        .interleaved_samples()
        .len()
        .checked_mul(2)
        .ok_or(SimpleSynthError::WaveArtifactTooLarge)?;
    let sample_bytes_u32 =
        u32::try_from(sample_bytes).map_err(|_| SimpleSynthError::WaveArtifactTooLarge)?;
    let riff_size = 36_u32
        .checked_add(sample_bytes_u32)
        .ok_or(SimpleSynthError::WaveArtifactTooLarge)?;
    let channels = u16::from(clip.channels());
    let block_align = channels
        .checked_mul(2)
        .ok_or(SimpleSynthError::WaveArtifactTooLarge)?;
    let byte_rate = clip
        .sample_rate_hz()
        .checked_mul(u32::from(block_align))
        .ok_or(SimpleSynthError::WaveArtifactTooLarge)?;
    let capacity = usize::try_from(riff_size)
        .ok()
        .and_then(|size| size.checked_add(8))
        .ok_or(SimpleSynthError::WaveArtifactTooLarge)?;
    let mut bytes = Vec::with_capacity(capacity);
    bytes.extend_from_slice(b"RIFF");
    bytes.extend_from_slice(&riff_size.to_le_bytes());
    bytes.extend_from_slice(b"WAVEfmt ");
    bytes.extend_from_slice(&16_u32.to_le_bytes());
    bytes.extend_from_slice(&1_u16.to_le_bytes());
    bytes.extend_from_slice(&channels.to_le_bytes());
    bytes.extend_from_slice(&clip.sample_rate_hz().to_le_bytes());
    bytes.extend_from_slice(&byte_rate.to_le_bytes());
    bytes.extend_from_slice(&block_align.to_le_bytes());
    bytes.extend_from_slice(&16_u16.to_le_bytes());
    bytes.extend_from_slice(b"data");
    bytes.extend_from_slice(&sample_bytes_u32.to_le_bytes());
    for sample in clip.interleaved_samples() {
        let pcm = if *sample <= -1.0 {
            i16::MIN
        } else {
            (sample.clamp(-1.0, 1.0) * f32::from(i16::MAX)).round() as i16
        };
        bytes.extend_from_slice(&pcm.to_le_bytes());
    }
    Ok(bytes)
}

#[derive(Clone, Copy, Debug)]
struct ChannelState {
    volume: f64,
    expression: f64,
    pan: f64,
    bend: f64,
}

impl Default for ChannelState {
    fn default() -> Self {
        Self {
            volume: 1.0,
            expression: 1.0,
            pan: 0.5,
            bend: 0.0,
        }
    }
}

#[derive(Clone, Copy, Debug)]
struct Voice {
    channel: u8,
    note: u8,
    velocity: f64,
    phase: f64,
    age: u64,
}

pub fn synthesize_sequence(
    sequence: &NoteSequence,
    config: SimpleSynthConfig,
) -> Result<SimpleSynthesis, SimpleSynthError> {
    if config.sample_rate_hz == 0 {
        return Err(SimpleSynthError::ZeroSampleRate);
    }
    if config.maximum_voices == 0 {
        return Err(SimpleSynthError::ZeroVoiceLimit);
    }
    if !config.master_gain.is_finite() || !(0.0..=1.0).contains(&config.master_gain) {
        return Err(SimpleSynthError::InvalidMasterGain);
    }
    let rendered_time_units = config.render_time_units.min(sequence.duration_units());
    let units_per_second = u64::from(sequence.timebase().units_per_second());
    let frames_u128 = u128::from(rendered_time_units)
        .saturating_mul(u128::from(config.sample_rate_hz))
        .div_ceil(u128::from(units_per_second));
    let frames = usize::try_from(frames_u128).unwrap_or(usize::MAX);
    if frames > config.maximum_frames {
        return Err(SimpleSynthError::FrameLimitExceeded {
            frames,
            limit: config.maximum_frames,
        });
    }

    let mut channels = [ChannelState::default(); 16];
    let mut voices = Vec::<Voice>::new();
    let mut samples = Vec::with_capacity(frames.saturating_mul(2));
    let mut event_index = 0;
    let mut voice_age = 0_u64;
    let mut maximum_active_voices = 0;
    let mut voice_steals = 0;
    let mut substituted_instruments = 0;
    let mut ignored_controls = 0;
    let mut peak = 0.0_f32;
    let mut clipped_samples = 0;

    for frame in 0..frames {
        while event_index < sequence.events().len()
            && event_frame(
                sequence.events()[event_index].time_units,
                config.sample_rate_hz,
                units_per_second,
            ) <= frame
        {
            let event = &sequence.events()[event_index];
            let channel = &mut channels[usize::from(event.channel)];
            match &event.kind {
                SequenceEventKind::NoteOn { note, velocity } => {
                    voices.retain(|voice| !(voice.channel == event.channel && voice.note == *note));
                    if *velocity != 0 {
                        if voices.len() == config.maximum_voices {
                            let oldest = voices
                                .iter()
                                .enumerate()
                                .min_by_key(|(_, voice)| voice.age)
                                .map(|(index, _)| index)
                                .expect("nonzero voice limit has an oldest voice");
                            voices.remove(oldest);
                            voice_steals += 1;
                        }
                        voices.push(Voice {
                            channel: event.channel,
                            note: *note,
                            velocity: f64::from(*velocity) / 127.0,
                            phase: 0.0,
                            age: voice_age,
                        });
                        voice_age = voice_age.saturating_add(1);
                        maximum_active_voices = maximum_active_voices.max(voices.len());
                    }
                }
                SequenceEventKind::NoteOff { note } => {
                    voices.retain(|voice| !(voice.channel == event.channel && voice.note == *note))
                }
                SequenceEventKind::Instrument { .. } => substituted_instruments += 1,
                SequenceEventKind::PitchBend { bend } => {
                    channel.bend = f64::from(*bend) / 8192.0;
                }
                SequenceEventKind::Control { control, value } => match control {
                    SequenceControl::Volume => channel.volume = f64::from(*value) / 127.0,
                    SequenceControl::Expression => {
                        channel.expression = f64::from(*value) / 127.0;
                    }
                    SequenceControl::Pan => channel.pan = f64::from(*value) / 127.0,
                    SequenceControl::AllSoundsOff | SequenceControl::AllNotesOff => {
                        voices.retain(|voice| voice.channel != event.channel);
                    }
                    SequenceControl::ResetControllers => *channel = ChannelState::default(),
                    _ => ignored_controls += 1,
                },
            }
            event_index += 1;
        }

        let mut left = 0.0_f64;
        let mut right = 0.0_f64;
        for voice in &mut voices {
            let channel = channels[usize::from(voice.channel)];
            let semitones = f64::from(voice.note) - 69.0 + channel.bend * 2.0;
            let frequency = 440.0 * 2.0_f64.powf(semitones / 12.0);
            voice.phase = (voice.phase + frequency / f64::from(config.sample_rate_hz)).fract();
            let oscillator = 1.0 - 4.0 * (voice.phase - 0.5).abs();
            let amplitude = oscillator
                * voice.velocity
                * channel.volume
                * channel.expression
                * f64::from(config.master_gain);
            left += amplitude * (1.0 - channel.pan);
            right += amplitude * channel.pan;
        }
        for sample in [left, right] {
            let unclamped = sample as f32;
            if unclamped.abs() > 1.0 {
                clipped_samples += 1;
            }
            let sample = unclamped.clamp(-1.0, 1.0);
            peak = peak.max(sample.abs());
            samples.push(sample);
        }
    }

    let sample_fingerprint = samples.iter().fold(0xcbf2_9ce4_8422_2325, |hash, sample| {
        sample
            .to_bits()
            .to_le_bytes()
            .iter()
            .fold(hash, |hash, byte| {
                (hash ^ u64::from(*byte)).wrapping_mul(0x0000_0100_0000_01b3)
            })
    });
    let clip = PcmClip::new(
        config.sample_rate_hz,
        2,
        samples,
        PcmClipLimits {
            maximum_frames: config.maximum_frames,
            maximum_channels: 2,
            maximum_sample_rate_hz: config.sample_rate_hz,
        },
    )?;
    Ok(SimpleSynthesis {
        clip,
        rendered_time_units,
        dispatched_events: event_index,
        maximum_active_voices,
        voice_steals,
        substituted_instruments,
        ignored_controls,
        peak,
        clipped_samples,
        sample_fingerprint,
    })
}

fn event_frame(time_units: u64, sample_rate_hz: u32, units_per_second: u64) -> usize {
    let frame = u128::from(time_units).saturating_mul(u128::from(sample_rate_hz))
        / u128::from(units_per_second);
    usize::try_from(frame).unwrap_or(usize::MAX)
}

#[cfg(test)]
mod tests {
    use audio_tools::{NoteSequenceLimits, SequenceEvent, SequenceTimebase};

    use super::*;

    fn fixture() -> NoteSequence {
        NoteSequence::new(
            SequenceTimebase::new(10, 10).unwrap(),
            1,
            10,
            vec![
                SequenceEvent {
                    time_units: 0,
                    order: 0,
                    channel: 0,
                    kind: SequenceEventKind::NoteOn {
                        note: 69,
                        velocity: 100,
                    },
                },
                SequenceEvent {
                    time_units: 10,
                    order: 1,
                    channel: 0,
                    kind: SequenceEventKind::NoteOff { note: 69 },
                },
            ],
            NoteSequenceLimits {
                maximum_events: 4,
                maximum_channels: 1,
                maximum_time_units: 10,
                maximum_units_per_second: 10,
            },
        )
        .unwrap()
    }

    #[test]
    fn fixed_sequence_produces_bounded_non_silent_pcm() {
        let synthesis = synthesize_sequence(
            &fixture(),
            SimpleSynthConfig {
                sample_rate_hz: 8_000,
                render_time_units: 10,
                maximum_frames: 8_000,
                maximum_voices: 4,
                master_gain: 0.2,
            },
        )
        .expect("synthesis");
        assert_eq!(synthesis.clip.frames(), 8_000);
        assert!(synthesis.peak > 0.0);
        assert_eq!(synthesis.clipped_samples, 0);
        assert_eq!(synthesis.maximum_active_voices, 1);
        assert_eq!(synthesis.voice_steals, 0);
    }

    #[test]
    fn output_is_repeatable_for_equal_inputs() {
        let config = SimpleSynthConfig {
            sample_rate_hz: 8_000,
            render_time_units: 10,
            maximum_frames: 8_000,
            maximum_voices: 4,
            master_gain: 0.2,
        };
        let first = synthesize_sequence(&fixture(), config).unwrap();
        let second = synthesize_sequence(&fixture(), config).unwrap();
        assert_eq!(first.sample_fingerprint, second.sample_fingerprint);
        assert_eq!(first.clip, second.clip);
    }

    #[test]
    fn synthesized_clip_encodes_as_canonical_pcm16_wave() {
        let synthesis = synthesize_sequence(
            &fixture(),
            SimpleSynthConfig {
                sample_rate_hz: 8_000,
                render_time_units: 1,
                maximum_frames: 800,
                maximum_voices: 4,
                master_gain: 0.2,
            },
        )
        .unwrap();
        let wave = encode_pcm16_wave(&synthesis.clip).unwrap();
        assert_eq!(&wave[..4], b"RIFF");
        assert_eq!(&wave[8..12], b"WAVE");
        assert_eq!(&wave[12..16], b"fmt ");
        assert_eq!(&wave[36..40], b"data");
        assert_eq!(wave.len(), 44 + synthesis.clip.frames() * 4);
    }
}
