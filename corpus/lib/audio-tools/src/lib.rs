//! Incubating Tokimu-shaped audio values for corpus consumers.
//!
//! These types separate decoded PCM and logical sound requests from source
//! formats and playback mechanisms. This corpus library is not an admitted
//! `tokimu-audio` capability and owns no device, mixer, clock, or platform API.

use thiserror::Error;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct PcmClipLimits {
    pub maximum_frames: usize,
    pub maximum_channels: u8,
    pub maximum_sample_rate_hz: u32,
}

#[derive(Clone, Debug, PartialEq)]
pub struct PcmClip {
    sample_rate_hz: u32,
    channels: u8,
    interleaved_samples: Vec<f32>,
}

impl PcmClip {
    pub fn new(
        sample_rate_hz: u32,
        channels: u8,
        interleaved_samples: Vec<f32>,
        limits: PcmClipLimits,
    ) -> Result<Self, AudioValueError> {
        if sample_rate_hz == 0 || sample_rate_hz > limits.maximum_sample_rate_hz {
            return Err(AudioValueError::InvalidSampleRate {
                sample_rate_hz,
                maximum_sample_rate_hz: limits.maximum_sample_rate_hz,
            });
        }
        if channels == 0 || channels > limits.maximum_channels {
            return Err(AudioValueError::InvalidChannelCount {
                channels,
                maximum_channels: limits.maximum_channels,
            });
        }
        let channels_usize = usize::from(channels);
        if !interleaved_samples.len().is_multiple_of(channels_usize) {
            return Err(AudioValueError::UnalignedSamples {
                samples: interleaved_samples.len(),
                channels,
            });
        }
        let frames = interleaved_samples.len() / channels_usize;
        if frames > limits.maximum_frames {
            return Err(AudioValueError::FrameLimitExceeded {
                frames,
                maximum_frames: limits.maximum_frames,
            });
        }
        if interleaved_samples
            .iter()
            .any(|sample| !sample.is_finite() || !(-1.0..=1.0).contains(sample))
        {
            return Err(AudioValueError::InvalidNormalizedSample);
        }
        Ok(Self {
            sample_rate_hz,
            channels,
            interleaved_samples,
        })
    }

    pub const fn sample_rate_hz(&self) -> u32 {
        self.sample_rate_hz
    }

    pub const fn channels(&self) -> u8 {
        self.channels
    }

    pub fn frames(&self) -> usize {
        self.interleaved_samples.len() / usize::from(self.channels)
    }

    pub fn interleaved_samples(&self) -> &[f32] {
        &self.interleaved_samples
    }
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct SoundClipKey(String);

impl SoundClipKey {
    pub fn new(value: impl Into<String>) -> Result<Self, AudioValueError> {
        let value = value.into();
        if value.is_empty()
            || value.len() > 128
            || !value.bytes().all(|byte| {
                byte.is_ascii_lowercase() || byte.is_ascii_digit() || b".-_".contains(&byte)
            })
        {
            return Err(AudioValueError::InvalidClipKey { value });
        }
        Ok(Self(value))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum SoundEmission {
    ListenerRelative,
    Spatial { position: [f32; 3] },
}

#[derive(Clone, Debug, PartialEq)]
pub struct SoundRequest {
    pub clip: SoundClipKey,
    pub emission: SoundEmission,
}

impl SoundRequest {
    pub fn new(clip: SoundClipKey, emission: SoundEmission) -> Result<Self, AudioValueError> {
        if let SoundEmission::Spatial { position } = emission {
            if position.iter().any(|component| !component.is_finite()) {
                return Err(AudioValueError::InvalidSpatialPosition);
            }
        }
        Ok(Self { clip, emission })
    }
}

#[derive(Clone, Debug, Error, Eq, PartialEq)]
pub enum AudioValueError {
    #[error("PCM sample rate {sample_rate_hz} is zero or exceeds {maximum_sample_rate_hz} Hz")]
    InvalidSampleRate {
        sample_rate_hz: u32,
        maximum_sample_rate_hz: u32,
    },
    #[error("PCM channel count {channels} is zero or exceeds {maximum_channels}")]
    InvalidChannelCount { channels: u8, maximum_channels: u8 },
    #[error("PCM sample count {samples} is not aligned to {channels} channels")]
    UnalignedSamples { samples: usize, channels: u8 },
    #[error("PCM frame count {frames} exceeds {maximum_frames}")]
    FrameLimitExceeded {
        frames: usize,
        maximum_frames: usize,
    },
    #[error("PCM contains a non-finite sample or a value outside normalized [-1, 1]")]
    InvalidNormalizedSample,
    #[error("audio clip key {value:?} is empty, too long, or not portable lowercase ASCII")]
    InvalidClipKey { value: String },
    #[error("spatial sound position contains a non-finite component")]
    InvalidSpatialPosition,
}

#[cfg(test)]
mod tests {
    use super::*;

    const LIMITS: PcmClipLimits = PcmClipLimits {
        maximum_frames: 4,
        maximum_channels: 2,
        maximum_sample_rate_hz: 48_000,
    };

    #[test]
    fn bounded_pcm_clip_retains_provider_neutral_samples() {
        let clip =
            PcmClip::new(11_025, 1, vec![-1.0, 0.0, 0.5], LIMITS).expect("bounded mono clip");
        assert_eq!(clip.sample_rate_hz(), 11_025);
        assert_eq!(clip.channels(), 1);
        assert_eq!(clip.frames(), 3);
        assert_eq!(clip.interleaved_samples(), [-1.0, 0.0, 0.5]);
    }

    #[test]
    fn logical_request_has_no_source_format_or_backend_handle() {
        let request = SoundRequest::new(
            SoundClipKey::new("monster.alert.zombieman").expect("portable key"),
            SoundEmission::Spatial {
                position: [1.0, 2.0, 3.0],
            },
        )
        .expect("finite spatial request");
        assert_eq!(request.clip.as_str(), "monster.alert.zombieman");
        assert_eq!(
            request.emission,
            SoundEmission::Spatial {
                position: [1.0, 2.0, 3.0]
            }
        );
    }

    #[test]
    fn invalid_pcm_and_spatial_values_fail_explicitly() {
        assert_eq!(
            PcmClip::new(11_025, 1, vec![f32::NAN], LIMITS),
            Err(AudioValueError::InvalidNormalizedSample)
        );
        assert!(matches!(
            SoundRequest::new(
                SoundClipKey::new("weapon.pistol").expect("portable key"),
                SoundEmission::Spatial {
                    position: [0.0, f32::INFINITY, 0.0]
                }
            ),
            Err(AudioValueError::InvalidSpatialPosition)
        ));
    }
}
