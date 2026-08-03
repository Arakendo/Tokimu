//! Incubating, provider-neutral input observations for visualizer corpus work.
//!
//! This crate does not capture or decode audio and does not own renderer
//! resources. It turns explicit time plus deterministic fixtures into bounded
//! waveform, spectrum, band, and beat observations.

mod audio_analysis;
mod native_visualizers;
mod radial_shape;
mod render_targets;
mod spectrum;
mod wav_fixture;
mod waveform;

pub use audio_analysis::{
    observe_pcm_analysis_timing, observe_pcm_analysis_working_set, PcmAnalysisBacklog,
    PcmAnalysisConfig, PcmAnalysisError, PcmAnalysisFrame, PcmAnalysisState,
    PcmAnalysisTimingObservation, PcmAnalysisWorkingSetObservation, PcmAnalyzer, PcmAudioWindow,
    PcmBacklogOverflowPolicy, PcmBacklogPush, PcmBacklogSnapshot, PcmFixture, MAX_PCM_CHANNELS,
    MAX_PCM_FRAMES, MAX_PCM_MEASUREMENT_ITERATIONS, MAX_PCM_PENDING_WINDOWS,
};

pub use native_visualizers::{
    NativeVisualizerDefinition, NativeVisualizerDefinitionError, NativeVisualizerKind,
    NativeVisualizerParameter,
};

pub use radial_shape::{
    VisualizerRadialShape, VisualizerRadialShapeError, VisualizerRadialShapeVertex,
    MAX_RADIAL_SHAPE_SIDES, MIN_RADIAL_SHAPE_SIDES,
};

pub use render_targets::{
    FeedbackInitialization, FeedbackTargetPairRequirement, RenderTargetColorInterpretation,
    RenderTargetGraphError, RenderTargetLoadBehavior, RenderTargetRequirement,
    RenderTargetSampling, VisualizerPassGraph, VisualizerPassGraphSummary,
    VisualizerPassRequirement, VisualizerResource, MAX_RENDER_PASSES, MAX_RENDER_TARGETS,
    MAX_RENDER_TARGET_DIMENSION,
};
pub use spectrum::{
    VisualizerSpectrumBar, VisualizerSpectrumBars, VisualizerSpectrumBarsError, MAX_SPECTRUM_BARS,
};

pub use wav_fixture::{decode_pcm16_wav, encode_pcm16_wav_fixture, WavFixtureError};
pub use waveform::{VisualizerWaveform, VisualizerWaveformError};

use std::{
    f32::consts::{PI, TAU},
    io,
    path::Path,
};

use screenshot::{write_bmp, write_manifest, Rgba8Image};
use serde::{Deserialize, Serialize};
use thiserror::Error;

pub const MAX_WAVEFORM_SAMPLES: usize = 2_048;
pub const MAX_SPECTRUM_BINS: usize = 1_024;
pub const MAX_VISUALIZER_UPDATE_HZ: f32 = 1_000.0;

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum SyntheticAudioFixture {
    Silence,
    Impulse,
    SteadyTone,
    FrequencySweep,
    SeededBandPulses,
}

impl SyntheticAudioFixture {
    pub const ALL: [Self; 5] = [
        Self::Silence,
        Self::Impulse,
        Self::SteadyTone,
        Self::FrequencySweep,
        Self::SeededBandPulses,
    ];

    pub const fn label(self) -> &'static str {
        match self {
            Self::Silence => "silence",
            Self::Impulse => "impulse",
            Self::SteadyTone => "steady-tone",
            Self::FrequencySweep => "frequency-sweep",
            Self::SeededBandPulses => "seeded-band-pulses",
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct VisualizerViewport {
    pub width: u32,
    pub height: u32,
}

impl VisualizerViewport {
    pub fn new(width: u32, height: u32) -> Result<Self, VisualizerInputError> {
        if width == 0 || height == 0 {
            return Err(VisualizerInputError::EmptyViewport { width, height });
        }
        Ok(Self { width, height })
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct AudioBands {
    pub sub_bass: f32,
    pub bass: f32,
    pub low_mid: f32,
    pub mid: f32,
    pub high_mid: f32,
    pub treble: f32,
}

impl AudioBands {
    pub fn shader_signal(self, phase: f32) -> [f32; 4] {
        [
            phase,
            ((self.sub_bass + self.bass) * 0.5).clamp(0.0, 1.0),
            ((self.low_mid + self.mid) * 0.5).clamp(0.0, 1.0),
            ((self.high_mid + self.treble) * 0.5).clamp(0.0, 1.0),
        ]
    }

    fn validate(self) -> Result<(), VisualizerInputError> {
        for (name, value) in [
            ("sub_bass", self.sub_bass),
            ("bass", self.bass),
            ("low_mid", self.low_mid),
            ("mid", self.mid),
            ("high_mid", self.high_mid),
            ("treble", self.treble),
        ] {
            validate_unit_value(name, value)?;
        }
        Ok(())
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct BeatObservation {
    pub energy: f32,
    pub pulse: f32,
    pub onset: bool,
}

impl BeatObservation {
    fn validate(self) -> Result<(), VisualizerInputError> {
        validate_unit_value("beat.energy", self.energy)?;
        validate_unit_value("beat.pulse", self.pulse)
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct VisualizerFrameInput {
    pub fixture: SyntheticAudioFixture,
    pub frame_index: u64,
    pub time_seconds: f32,
    pub delta_seconds: f32,
    pub viewport: VisualizerViewport,
    pub waveform: Vec<f32>,
    pub spectrum: Vec<f32>,
    pub bands: AudioBands,
    pub beat: BeatObservation,
}

impl VisualizerFrameInput {
    pub fn validate(&self) -> Result<(), VisualizerInputError> {
        validate_time("time_seconds", self.time_seconds, true)?;
        validate_time("delta_seconds", self.delta_seconds, false)?;
        if self.delta_seconds < 1.0 / MAX_VISUALIZER_UPDATE_HZ {
            return Err(VisualizerInputError::UpdateFrequencyTooHigh {
                delta_seconds: self.delta_seconds,
                maximum_hz: MAX_VISUALIZER_UPDATE_HZ,
            });
        }
        VisualizerViewport::new(self.viewport.width, self.viewport.height)?;
        validate_buffer("waveform", &self.waveform, MAX_WAVEFORM_SAMPLES, -1.0, 1.0)?;
        validate_buffer("spectrum", &self.spectrum, MAX_SPECTRUM_BINS, 0.0, 1.0)?;
        self.bands.validate()?;
        self.beat.validate()
    }

    pub fn to_structural_json(&self) -> Result<String, VisualizerInputError> {
        self.validate()?;
        serde_json::to_string_pretty(self)
            .map_err(|error| VisualizerInputError::Serialization(error.to_string()))
    }

    pub fn shader_signal(&self) -> [f32; 4] {
        self.bands
            .shader_signal((self.time_seconds / TAU).fract().rem_euclid(1.0))
    }
}

/// Writes deterministic CPU image evidence for one visualizer observation.
///
/// This is deliberately a source-side signal preview, not a GPU framebuffer
/// capture or a claim of backend pixel equivalence. The paired manifest makes
/// that boundary explicit for corpus review.
pub fn write_cpu_preview(
    image_path: impl AsRef<Path>,
    manifest_path: impl AsRef<Path>,
    input: &VisualizerFrameInput,
) -> io::Result<()> {
    input.validate().map_err(io::Error::other)?;
    let width = input.viewport.width;
    let height = input.viewport.height;
    let mut pixels = vec![0_u8; width as usize * height as usize * 4];
    let [phase, low, mid, high] = input.shader_signal();

    for y in 0..height {
        let vertical = (y as f32 / (height.saturating_sub(1).max(1)) as f32) * 2.0 - 1.0;
        for x in 0..width {
            let horizontal = (x as f32 / (width.saturating_sub(1).max(1)) as f32) * 2.0 - 1.0;
            let point_x = horizontal * 1.55;
            let radius = (point_x * point_x + vertical * vertical).sqrt();
            let angle = vertical.atan2(point_x);
            let rings = 0.5 + 0.5 * (radius * (18.0 + low * 26.0) - phase * TAU * 2.0).sin();
            let spokes = 0.5 + 0.5 * (angle * (4.0 + (mid * 8.0).floor()) + phase * TAU).cos();
            let sweep = 0.5 + 0.5 * ((point_x + vertical) * 10.0 - phase * TAU * 3.0).sin();
            let energy =
                (rings * (0.45 + low) + spokes * mid * 0.7 + sweep * high * 0.55).clamp(0.0, 1.0);
            let vignette = 1.0 - smoothstep(0.15, 1.35, radius);
            let color = preview_palette(energy, low, mid, high);
            let intensity = 0.25 + vignette * 0.85;
            let offset = ((y * width + x) * 4) as usize;
            pixels[offset] = float_to_u8(color[0] * intensity);
            pixels[offset + 1] = float_to_u8(color[1] * intensity);
            pixels[offset + 2] = float_to_u8(color[2] * intensity);
            pixels[offset + 3] = u8::MAX;
        }
    }

    write_bmp(
        image_path.as_ref(),
        Rgba8Image {
            width,
            height,
            pixels: &pixels,
        },
    )
    .map_err(io::Error::other)?;
    let width_text = width.to_string();
    let height_text = height.to_string();
    let frame_text = input.frame_index.to_string();
    let signal_text = format!("{phase:.6},{low:.6},{mid:.6},{high:.6}");
    write_manifest(
        manifest_path.as_ref(),
        &[
            ("schema", "tokimu-visualizer-preview-v1"),
            ("producer", "visualizer-tools"),
            ("fixture", input.fixture.label()),
            ("frame_index", &frame_text),
            ("width", &width_text),
            ("height", &height_text),
            ("signal", &signal_text),
            ("capture_kind", "deterministic-cpu-signal-preview"),
            ("gpu_framebuffer_capture", "false"),
            ("backend_pixel_equivalence", "not-claimed"),
        ],
    )
    .map_err(io::Error::other)
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SyntheticVisualizerConfig {
    pub delta_seconds: f32,
    pub waveform_samples: usize,
    pub spectrum_bins: usize,
    pub seed: u64,
}

impl Default for SyntheticVisualizerConfig {
    fn default() -> Self {
        Self {
            delta_seconds: 1.0 / 60.0,
            waveform_samples: 128,
            spectrum_bins: 64,
            seed: 0x544f_4b49_4d55,
        }
    }
}

impl SyntheticVisualizerConfig {
    pub fn validate(self) -> Result<(), VisualizerInputError> {
        validate_time("delta_seconds", self.delta_seconds, false)?;
        if self.delta_seconds < 1.0 / MAX_VISUALIZER_UPDATE_HZ {
            return Err(VisualizerInputError::UpdateFrequencyTooHigh {
                delta_seconds: self.delta_seconds,
                maximum_hz: MAX_VISUALIZER_UPDATE_HZ,
            });
        }
        validate_count("waveform", self.waveform_samples, MAX_WAVEFORM_SAMPLES)?;
        validate_count("spectrum", self.spectrum_bins, MAX_SPECTRUM_BINS)
    }
}

#[derive(Clone, Copy, Debug)]
pub struct SyntheticVisualizerInput {
    fixture: SyntheticAudioFixture,
    config: SyntheticVisualizerConfig,
}

impl SyntheticVisualizerInput {
    pub fn new(
        fixture: SyntheticAudioFixture,
        config: SyntheticVisualizerConfig,
    ) -> Result<Self, VisualizerInputError> {
        config.validate()?;
        Ok(Self { fixture, config })
    }

    pub const fn fixture(&self) -> SyntheticAudioFixture {
        self.fixture
    }

    pub fn set_fixture(&mut self, fixture: SyntheticAudioFixture) {
        self.fixture = fixture;
    }

    pub fn frame(
        &self,
        frame_index: u64,
        viewport: VisualizerViewport,
    ) -> Result<VisualizerFrameInput, VisualizerInputError> {
        let time_seconds = frame_index as f32 * self.config.delta_seconds;
        let waveform = (0..self.config.waveform_samples)
            .map(|sample| self.waveform_value(frame_index, sample, time_seconds))
            .collect::<Vec<_>>();
        let spectrum = (0..self.config.spectrum_bins)
            .map(|bin| self.spectrum_value(frame_index, bin, time_seconds))
            .collect::<Vec<_>>();
        let bands = bands_from_spectrum(&spectrum);
        let beat = beat_for_fixture(self.fixture, frame_index, time_seconds, bands);
        let input = VisualizerFrameInput {
            fixture: self.fixture,
            frame_index,
            time_seconds,
            delta_seconds: self.config.delta_seconds,
            viewport,
            waveform,
            spectrum,
            bands,
            beat,
        };
        input.validate()?;
        Ok(input)
    }

    fn waveform_value(&self, frame_index: u64, sample: usize, time_seconds: f32) -> f32 {
        let x = sample as f32 / self.config.waveform_samples as f32;
        match self.fixture {
            SyntheticAudioFixture::Silence => 0.0,
            SyntheticAudioFixture::Impulse => {
                let envelope = (-time_seconds * 7.0).exp();
                (TAU * (x * 6.0 + time_seconds * 10.0)).sin() * envelope
            }
            SyntheticAudioFixture::SteadyTone => (TAU * (x * 4.0 + time_seconds * 2.0)).sin() * 0.7,
            SyntheticAudioFixture::FrequencySweep => {
                let cycles = 1.0 + (time_seconds * 0.35).sin().abs() * 14.0;
                (TAU * (x * cycles + time_seconds)).sin() * 0.75
            }
            SyntheticAudioFixture::SeededBandPulses => {
                let pulse = seeded_unit(self.config.seed, frame_index / 12, sample as u64);
                let carrier = (TAU * (x * 3.0 + time_seconds * 1.5)).sin();
                carrier * (0.2 + pulse * 0.75)
            }
        }
        .clamp(-1.0, 1.0)
    }

    fn spectrum_value(&self, frame_index: u64, bin: usize, time_seconds: f32) -> f32 {
        let x = if self.config.spectrum_bins <= 1 {
            0.0
        } else {
            bin as f32 / (self.config.spectrum_bins - 1) as f32
        };
        match self.fixture {
            SyntheticAudioFixture::Silence => 0.0,
            SyntheticAudioFixture::Impulse => (-time_seconds * 6.0).exp() * (1.0 - x * 0.35),
            SyntheticAudioFixture::SteadyTone => gaussian(x, 0.22, 0.045),
            SyntheticAudioFixture::FrequencySweep => {
                let center = 0.08 + (time_seconds * 0.12).fract() * 0.84;
                gaussian(x, center, 0.055)
            }
            SyntheticAudioFixture::SeededBandPulses => {
                let block = frame_index / 12;
                let center = 0.08 + seeded_unit(self.config.seed, block, 0) * 0.84;
                let gain = 0.25 + seeded_unit(self.config.seed, block, 1) * 0.75;
                gaussian(x, center, 0.07) * gain
            }
        }
        .clamp(0.0, 1.0)
    }
}

#[derive(Clone, Debug, Error, PartialEq)]
pub enum VisualizerInputError {
    #[error("visualizer viewport must be non-empty, received {width}x{height}")]
    EmptyViewport { width: u32, height: u32 },
    #[error("{field} must be finite and {requirement}, received {value}")]
    InvalidTime {
        field: &'static str,
        value: f32,
        requirement: &'static str,
    },
    #[error(
        "visualizer update delta {delta_seconds} exceeds the bounded maximum of {maximum_hz} Hz"
    )]
    UpdateFrequencyTooHigh { delta_seconds: f32, maximum_hz: f32 },
    #[error("{buffer} must contain at least one value")]
    EmptyBuffer { buffer: &'static str },
    #[error("{buffer} contains {count} values; maximum is {maximum}")]
    BufferTooLarge {
        buffer: &'static str,
        count: usize,
        maximum: usize,
    },
    #[error(
        "{buffer}[{index}] must be finite and within [{minimum}, {maximum}], received {value}"
    )]
    InvalidBufferValue {
        buffer: &'static str,
        index: usize,
        value: f32,
        minimum: f32,
        maximum: f32,
    },
    #[error("{field} must be finite and within [0, 1], received {value}")]
    InvalidUnitValue { field: &'static str, value: f32 },
    #[error("could not serialize visualizer observation: {0}")]
    Serialization(String),
}

fn validate_time(
    field: &'static str,
    value: f32,
    allow_zero: bool,
) -> Result<(), VisualizerInputError> {
    let valid = value.is_finite()
        && if allow_zero {
            value >= 0.0
        } else {
            value > 0.0
        };
    if valid {
        Ok(())
    } else {
        Err(VisualizerInputError::InvalidTime {
            field,
            value,
            requirement: if allow_zero {
                "non-negative"
            } else {
                "positive"
            },
        })
    }
}

fn validate_count(
    buffer: &'static str,
    count: usize,
    maximum: usize,
) -> Result<(), VisualizerInputError> {
    if count == 0 {
        Err(VisualizerInputError::EmptyBuffer { buffer })
    } else if count > maximum {
        Err(VisualizerInputError::BufferTooLarge {
            buffer,
            count,
            maximum,
        })
    } else {
        Ok(())
    }
}

fn validate_buffer(
    buffer: &'static str,
    values: &[f32],
    maximum_count: usize,
    minimum: f32,
    maximum: f32,
) -> Result<(), VisualizerInputError> {
    validate_count(buffer, values.len(), maximum_count)?;
    for (index, value) in values.iter().copied().enumerate() {
        if !value.is_finite() || value < minimum || value > maximum {
            return Err(VisualizerInputError::InvalidBufferValue {
                buffer,
                index,
                value,
                minimum,
                maximum,
            });
        }
    }
    Ok(())
}

fn validate_unit_value(field: &'static str, value: f32) -> Result<(), VisualizerInputError> {
    if value.is_finite() && (0.0..=1.0).contains(&value) {
        Ok(())
    } else {
        Err(VisualizerInputError::InvalidUnitValue { field, value })
    }
}

fn gaussian(value: f32, center: f32, width: f32) -> f32 {
    let distance = (value - center) / width.max(f32::EPSILON);
    (-0.5 * distance * distance).exp()
}

fn smoothstep(edge_start: f32, edge_end: f32, value: f32) -> f32 {
    let normalized = ((value - edge_start) / (edge_end - edge_start)).clamp(0.0, 1.0);
    normalized * normalized * (3.0 - 2.0 * normalized)
}

fn preview_palette(value: f32, low: f32, mid: f32, high: f32) -> [f32; 3] {
    let cyan = [
        0.18 * (0.35 + low * 0.9),
        0.95 * (0.35 + low * 0.9),
        0.86 * (0.35 + low * 0.9),
    ];
    let amber = [
        1.0 * (0.15 + mid * 0.85),
        0.48 * (0.15 + mid * 0.85),
        0.12 * (0.15 + mid * 0.85),
    ];
    let ice = [
        0.50 * (0.20 + high * 0.8),
        0.68 * (0.20 + high * 0.8),
        1.0 * (0.20 + high * 0.8),
    ];
    let cyan_weight = smoothstep(0.0, 0.55, value);
    let amber_weight = smoothstep(0.42, 0.82, value);
    let ice_weight = smoothstep(0.75, 1.0, value);
    [
        0.03 + cyan[0] * cyan_weight + amber[0] * amber_weight + ice[0] * ice_weight,
        0.06 + cyan[1] * cyan_weight + amber[1] * amber_weight + ice[1] * ice_weight,
        0.11 + cyan[2] * cyan_weight + amber[2] * amber_weight + ice[2] * ice_weight,
    ]
}

fn float_to_u8(value: f32) -> u8 {
    (value.clamp(0.0, 1.0) * u8::MAX as f32).round() as u8
}

fn seeded_unit(seed: u64, frame_block: u64, channel: u64) -> f32 {
    let mut value = seed
        ^ frame_block.wrapping_mul(0x9e37_79b9_7f4a_7c15)
        ^ channel.wrapping_mul(0xbf58_476d_1ce4_e5b9);
    value ^= value >> 30;
    value = value.wrapping_mul(0xbf58_476d_1ce4_e5b9);
    value ^= value >> 27;
    value = value.wrapping_mul(0x94d0_49bb_1331_11eb);
    value ^= value >> 31;
    (value as u32) as f32 / u32::MAX as f32
}

pub(crate) fn bands_from_spectrum(spectrum: &[f32]) -> AudioBands {
    let band = |start: f32, end: f32| {
        let start = (start * spectrum.len() as f32).floor() as usize;
        let end = ((end * spectrum.len() as f32).ceil() as usize)
            .max(start + 1)
            .min(spectrum.len());
        spectrum[start.min(spectrum.len() - 1)..end]
            .iter()
            .sum::<f32>()
            / (end - start.min(spectrum.len() - 1)) as f32
    };
    AudioBands {
        sub_bass: band(0.0, 0.08),
        bass: band(0.08, 0.20),
        low_mid: band(0.20, 0.36),
        mid: band(0.36, 0.56),
        high_mid: band(0.56, 0.76),
        treble: band(0.76, 1.0),
    }
}

fn beat_for_fixture(
    fixture: SyntheticAudioFixture,
    frame_index: u64,
    time_seconds: f32,
    bands: AudioBands,
) -> BeatObservation {
    let energy = ((bands.sub_bass + bands.bass + bands.low_mid + bands.mid) * 0.25).clamp(0.0, 1.0);
    match fixture {
        SyntheticAudioFixture::Silence => BeatObservation::default(),
        SyntheticAudioFixture::Impulse => BeatObservation {
            energy,
            pulse: (-time_seconds * 8.0).exp().clamp(0.0, 1.0),
            onset: frame_index == 0,
        },
        SyntheticAudioFixture::SeededBandPulses => BeatObservation {
            energy,
            pulse: if frame_index.is_multiple_of(12) {
                1.0
            } else {
                0.25
            },
            onset: frame_index.is_multiple_of(12),
        },
        SyntheticAudioFixture::SteadyTone | SyntheticAudioFixture::FrequencySweep => {
            BeatObservation {
                energy,
                pulse: (0.5 + 0.5 * (time_seconds * PI * 2.0).sin()).clamp(0.0, 1.0),
                onset: false,
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn viewport() -> VisualizerViewport {
        VisualizerViewport::new(1280, 720).unwrap()
    }

    #[test]
    fn synthetic_frames_are_deterministic() {
        let source = SyntheticVisualizerInput::new(
            SyntheticAudioFixture::SeededBandPulses,
            SyntheticVisualizerConfig::default(),
        )
        .unwrap();
        let first = source.frame(42, viewport()).unwrap();
        let second = source.frame(42, viewport()).unwrap();
        assert_eq!(first, second);
        assert_eq!(
            first.to_structural_json().unwrap(),
            second.to_structural_json().unwrap()
        );
    }

    #[test]
    fn every_fixture_produces_a_valid_headless_observation() {
        for fixture in SyntheticAudioFixture::ALL {
            let source =
                SyntheticVisualizerInput::new(fixture, SyntheticVisualizerConfig::default())
                    .unwrap();
            source.frame(17, viewport()).unwrap().validate().unwrap();
        }
    }

    #[test]
    fn excessive_and_non_finite_input_fail_stably() {
        let oversized = SyntheticVisualizerConfig {
            waveform_samples: MAX_WAVEFORM_SAMPLES + 1,
            ..SyntheticVisualizerConfig::default()
        };
        assert!(matches!(
            oversized.validate(),
            Err(VisualizerInputError::BufferTooLarge {
                buffer: "waveform",
                ..
            })
        ));

        let mut frame = SyntheticVisualizerInput::new(
            SyntheticAudioFixture::Silence,
            SyntheticVisualizerConfig::default(),
        )
        .unwrap()
        .frame(0, viewport())
        .unwrap();
        frame.spectrum[3] = f32::NAN;
        assert!(matches!(
            frame.validate(),
            Err(VisualizerInputError::InvalidBufferValue {
                buffer: "spectrum",
                index: 3,
                ..
            })
        ));
    }

    #[test]
    fn empty_viewports_and_excessive_update_rates_are_rejected() {
        assert_eq!(
            VisualizerViewport::new(0, 720),
            Err(VisualizerInputError::EmptyViewport {
                width: 0,
                height: 720
            })
        );
        let too_fast = SyntheticVisualizerConfig {
            delta_seconds: 0.000_5,
            ..SyntheticVisualizerConfig::default()
        };
        assert!(matches!(
            too_fast.validate(),
            Err(VisualizerInputError::UpdateFrequencyTooHigh { .. })
        ));
    }

    #[test]
    fn cpu_preview_is_explicitly_labeled_as_non_gpu_evidence() {
        let output =
            std::env::temp_dir().join(format!("tokimu-visualizer-preview-{}", std::process::id()));
        std::fs::create_dir_all(&output).unwrap();
        let frame = SyntheticVisualizerInput::new(
            SyntheticAudioFixture::SteadyTone,
            SyntheticVisualizerConfig::default(),
        )
        .unwrap()
        .frame(12, viewport())
        .unwrap();
        let image = output.join("steady-tone.bmp");
        let manifest = output.join("steady-tone.preview.txt");
        write_cpu_preview(&image, &manifest, &frame).unwrap();
        assert!(image.is_file());
        let manifest = std::fs::read_to_string(manifest).unwrap();
        assert!(manifest.contains("capture_kind=deterministic-cpu-signal-preview"));
        assert!(manifest.contains("gpu_framebuffer_capture=false"));
        let _ = std::fs::remove_dir_all(output);
    }
}
