//! Bounded decoding for Doom sound-effect lumps.
//!
//! This concrete provider owns source bytes and PCM conversion only. It does
//! not own gameplay events, clip selection, playback, mixing, devices, or a
//! stable Tokimu audio contract.

use audio_tools::{AudioValueError, PcmClip, PcmClipLimits};
use doom_wad_provider::WadManifest;
use thiserror::Error;

const SOUND_HEADER_BYTES: usize = 8;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct DoomSoundDecodeLimits {
    pub maximum_samples: u32,
    pub maximum_sample_rate_hz: u16,
}

#[derive(Clone, Debug, PartialEq)]
pub struct DoomSoundEffect {
    pub source_lump_index: u32,
    pub source_name: String,
    pub format: u16,
    pub sample_rate_hz: u16,
    pub samples: Vec<u8>,
}

impl DoomSoundEffect {
    pub fn duration_seconds(&self) -> f64 {
        self.samples.len() as f64 / f64::from(self.sample_rate_hz)
    }

    /// Converts unsigned eight-bit mono source samples into finite normalized
    /// PCM without choosing resampling, mixing, or playback policy.
    pub fn normalized_mono_pcm(&self) -> Vec<f32> {
        self.samples
            .iter()
            .map(|sample| (f32::from(*sample) - 128.0) / 128.0)
            .collect()
    }

    pub fn to_pcm_clip(&self, limits: PcmClipLimits) -> Result<PcmClip, AudioValueError> {
        PcmClip::new(
            u32::from(self.sample_rate_hz),
            1,
            self.normalized_mono_pcm(),
            limits,
        )
    }

    pub fn sample_fingerprint(&self) -> u64 {
        self.samples
            .iter()
            .fold(0xcbf2_9ce4_8422_2325, |hash, byte| {
                (hash ^ u64::from(*byte)).wrapping_mul(0x0000_0100_0000_01b3)
            })
    }
}

#[derive(Clone, Debug, Error, Eq, PartialEq)]
pub enum DoomSoundDecodeError {
    #[error("Doom sound lump {name} is unavailable")]
    MissingLump { name: String },
    #[error("Doom sound lump {name} range is outside the retained WAD bytes")]
    LumpOutOfBounds { name: String },
    #[error("Doom sound lump {name} is truncated: expected at least 8 bytes, found {bytes}")]
    Truncated { name: String, bytes: usize },
    #[error("Doom sound lump {name} has unsupported format {format}, expected 3")]
    UnsupportedFormat { name: String, format: u16 },
    #[error("Doom sound lump {name} declares zero sample rate")]
    ZeroSampleRate { name: String },
    #[error("Doom sound lump {name} sample rate {sample_rate_hz} exceeds limit {limit_hz}")]
    SampleRateLimitExceeded {
        name: String,
        sample_rate_hz: u16,
        limit_hz: u16,
    },
    #[error(
        "Doom sound lump {name} declares {declared} samples but retains {actual} payload bytes"
    )]
    SampleCountMismatch {
        name: String,
        declared: u32,
        actual: usize,
    },
    #[error("Doom sound lump {name} has {samples} samples, exceeding limit {limit}")]
    SampleLimitExceeded {
        name: String,
        samples: u32,
        limit: u32,
    },
}

/// Resolves the last matching WAD lump, matching Doom's replacement-friendly
/// namespace lookup, then decodes format-3 unsigned eight-bit mono samples.
pub fn decode_doom_sound_effect(
    wad_bytes: &[u8],
    manifest: &WadManifest,
    name: &str,
    limits: DoomSoundDecodeLimits,
) -> Result<DoomSoundEffect, DoomSoundDecodeError> {
    let lump = manifest
        .lumps
        .iter()
        .rev()
        .find(|lump| lump.name.eq_ignore_ascii_case(name))
        .ok_or_else(|| DoomSoundDecodeError::MissingLump {
            name: name.to_owned(),
        })?;
    let start = usize::try_from(lump.offset).expect("u32 offset fits usize");
    let size = usize::try_from(lump.size).expect("u32 size fits usize");
    let bytes = start
        .checked_add(size)
        .filter(|end| *end <= wad_bytes.len())
        .map(|end| &wad_bytes[start..end])
        .ok_or_else(|| DoomSoundDecodeError::LumpOutOfBounds {
            name: lump.name.clone(),
        })?;
    if bytes.len() < SOUND_HEADER_BYTES {
        return Err(DoomSoundDecodeError::Truncated {
            name: lump.name.clone(),
            bytes: bytes.len(),
        });
    }
    let format = u16::from_le_bytes([bytes[0], bytes[1]]);
    if format != 3 {
        return Err(DoomSoundDecodeError::UnsupportedFormat {
            name: lump.name.clone(),
            format,
        });
    }
    let sample_rate_hz = u16::from_le_bytes([bytes[2], bytes[3]]);
    if sample_rate_hz == 0 {
        return Err(DoomSoundDecodeError::ZeroSampleRate {
            name: lump.name.clone(),
        });
    }
    if sample_rate_hz > limits.maximum_sample_rate_hz {
        return Err(DoomSoundDecodeError::SampleRateLimitExceeded {
            name: lump.name.clone(),
            sample_rate_hz,
            limit_hz: limits.maximum_sample_rate_hz,
        });
    }
    let declared = u32::from_le_bytes([bytes[4], bytes[5], bytes[6], bytes[7]]);
    let samples = &bytes[SOUND_HEADER_BYTES..];
    if usize::try_from(declared).expect("u32 count fits usize") != samples.len() {
        return Err(DoomSoundDecodeError::SampleCountMismatch {
            name: lump.name.clone(),
            declared,
            actual: samples.len(),
        });
    }
    if declared > limits.maximum_samples {
        return Err(DoomSoundDecodeError::SampleLimitExceeded {
            name: lump.name.clone(),
            samples: declared,
            limit: limits.maximum_samples,
        });
    }
    Ok(DoomSoundEffect {
        source_lump_index: lump.index,
        source_name: lump.name.clone(),
        format,
        sample_rate_hz,
        samples: samples.to_vec(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use doom_wad_provider::{WadKind, WadLumpObservation, WadSourceIdentity};

    fn manifest(name: &str, bytes: usize) -> WadManifest {
        WadManifest {
            source: WadSourceIdentity {
                label: "fixture".to_owned(),
                byte_len: bytes,
                blake3: "fixture".to_owned(),
            },
            kind: WadKind::Iwad,
            directory_offset: 0,
            directory_bytes: 0,
            total_lump_bytes: bytes as u64,
            lumps: vec![WadLumpObservation {
                index: 4,
                offset: 0,
                size: bytes as u32,
                name: name.to_owned(),
            }],
            namespaces: Vec::new(),
        }
    }

    #[test]
    fn format_three_sound_decodes_unsigned_mono_pcm() {
        let bytes = [3, 0, 0x11, 0x2b, 3, 0, 0, 0, 0, 128, 255];
        let sound = decode_doom_sound_effect(
            &bytes,
            &manifest("DSPISTOL", bytes.len()),
            "dspistol",
            DoomSoundDecodeLimits {
                maximum_samples: 8,
                maximum_sample_rate_hz: 48_000,
            },
        )
        .expect("bounded fixture decodes");
        assert_eq!(sound.sample_rate_hz, 11_025);
        assert_eq!(sound.samples, [0, 128, 255]);
        assert_eq!(sound.normalized_mono_pcm(), [-1.0, 0.0, 127.0 / 128.0]);
    }

    #[test]
    fn declared_sample_count_must_match_payload() {
        let bytes = [3, 0, 0x11, 0x2b, 4, 0, 0, 0, 128, 128, 128];
        assert!(matches!(
            decode_doom_sound_effect(
                &bytes,
                &manifest("DSPISTOL", bytes.len()),
                "DSPISTOL",
                DoomSoundDecodeLimits {
                    maximum_samples: 8,
                    maximum_sample_rate_hz: 48_000,
                },
            ),
            Err(DoomSoundDecodeError::SampleCountMismatch {
                declared: 4,
                actual: 3,
                ..
            })
        ));
    }

    #[test]
    fn unsupported_format_and_decode_limits_fail_explicitly() {
        let unsupported = [2, 0, 0x11, 0x2b, 1, 0, 0, 0, 128];
        assert!(matches!(
            decode_doom_sound_effect(
                &unsupported,
                &manifest("DSBAD", unsupported.len()),
                "DSBAD",
                DoomSoundDecodeLimits {
                    maximum_samples: 8,
                    maximum_sample_rate_hz: 48_000,
                },
            ),
            Err(DoomSoundDecodeError::UnsupportedFormat { format: 2, .. })
        ));

        let too_many_samples = [3, 0, 0x11, 0x2b, 3, 0, 0, 0, 127, 128, 129];
        assert!(matches!(
            decode_doom_sound_effect(
                &too_many_samples,
                &manifest("DSLARGE", too_many_samples.len()),
                "DSLARGE",
                DoomSoundDecodeLimits {
                    maximum_samples: 2,
                    maximum_sample_rate_hz: 48_000,
                },
            ),
            Err(DoomSoundDecodeError::SampleLimitExceeded {
                samples: 3,
                limit: 2,
                ..
            })
        ));
    }
}
