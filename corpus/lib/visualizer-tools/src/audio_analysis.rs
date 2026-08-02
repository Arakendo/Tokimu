//! Deterministic PCM analysis used only as corpus evidence.
//!
//! This module consumes explicit PCM windows. It deliberately contains no
//! audio-device, browser, file-decoder, renderer, or wall-clock mechanism.
//! The spectrum is a small direct DFT reference implementation: it favors
//! inspectability and stable fixtures over production FFT throughput.

use std::{
    collections::VecDeque,
    f32::consts::TAU,
    hint::black_box,
    time::{Duration, Instant},
};

use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::{bands_from_spectrum, AudioBands, BeatObservation, MAX_SPECTRUM_BINS};

pub const MAX_PCM_CHANNELS: u8 = 2;
pub const MAX_PCM_FRAMES: usize = 2_048;
pub const MAX_PCM_PENDING_WINDOWS: usize = 8;
pub const MAX_PCM_MEASUREMENT_ITERATIONS: usize = 512;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct PcmAudioWindow {
    /// Samples are interleaved by frame: mono is `[L, L, ...]`, stereo is
    /// `[L, R, L, R, ...]`. Every sample is normalized PCM in `[-1, 1]`.
    pub interleaved_samples: Vec<f32>,
    pub sample_rate_hz: u32,
    pub channels: u8,
}

impl PcmAudioWindow {
    pub fn new(
        interleaved_samples: Vec<f32>,
        sample_rate_hz: u32,
        channels: u8,
    ) -> Result<Self, PcmAnalysisError> {
        let window = Self {
            interleaved_samples,
            sample_rate_hz,
            channels,
        };
        window.validate()?;
        Ok(window)
    }

    pub fn frame_count(&self) -> usize {
        self.interleaved_samples.len() / usize::from(self.channels)
    }

    pub fn validate(&self) -> Result<(), PcmAnalysisError> {
        if self.sample_rate_hz == 0 {
            return Err(PcmAnalysisError::EmptySampleRate);
        }
        if self.channels == 0 || self.channels > MAX_PCM_CHANNELS {
            return Err(PcmAnalysisError::UnsupportedChannelCount {
                channels: self.channels,
                maximum: MAX_PCM_CHANNELS,
            });
        }
        if self.interleaved_samples.is_empty() {
            return Err(PcmAnalysisError::EmptyWindow);
        }
        if !self
            .interleaved_samples
            .len()
            .is_multiple_of(usize::from(self.channels))
        {
            return Err(PcmAnalysisError::IncompleteInterleavedFrame {
                samples: self.interleaved_samples.len(),
                channels: self.channels,
            });
        }
        let frames = self.frame_count();
        if frames > MAX_PCM_FRAMES {
            return Err(PcmAnalysisError::TooManyFrames {
                frames,
                maximum: MAX_PCM_FRAMES,
            });
        }
        for (index, sample) in self.interleaved_samples.iter().copied().enumerate() {
            if !sample.is_finite() || !(-1.0..=1.0).contains(&sample) {
                return Err(PcmAnalysisError::InvalidSample { index, sample });
            }
        }
        Ok(())
    }

    fn mono_samples(&self) -> Vec<f32> {
        self.interleaved_samples
            .chunks_exact(usize::from(self.channels))
            .map(|frame| frame.iter().sum::<f32>() / frame.len() as f32)
            .collect()
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct PcmAnalysisConfig {
    pub spectrum_bins: usize,
    pub onset_energy_threshold: f32,
    /// `0` takes the current band value; `1` preserves the previous value.
    pub band_smoothing: f32,
    /// A rising-energy onset must exceed this change after the first window.
    pub onset_energy_delta: f32,
}

impl Default for PcmAnalysisConfig {
    fn default() -> Self {
        Self {
            spectrum_bins: 64,
            onset_energy_threshold: 0.20,
            band_smoothing: 0.65,
            onset_energy_delta: 0.08,
        }
    }
}

impl PcmAnalysisConfig {
    pub fn validate(self) -> Result<(), PcmAnalysisError> {
        if self.spectrum_bins == 0 || self.spectrum_bins > MAX_SPECTRUM_BINS {
            return Err(PcmAnalysisError::InvalidSpectrumBinCount {
                bins: self.spectrum_bins,
                maximum: MAX_SPECTRUM_BINS,
            });
        }
        if !self.onset_energy_threshold.is_finite()
            || !(0.0..=1.0).contains(&self.onset_energy_threshold)
        {
            return Err(PcmAnalysisError::InvalidOnsetThreshold {
                threshold: self.onset_energy_threshold,
            });
        }
        for (field, value) in [
            ("band smoothing", self.band_smoothing),
            ("onset energy delta", self.onset_energy_delta),
        ] {
            if !value.is_finite() || !(0.0..=1.0).contains(&value) {
                return Err(PcmAnalysisError::InvalidUnitConfiguration { field, value });
            }
        }
        Ok(())
    }
}

#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct PcmAnalysisFrame {
    pub sample_rate_hz: u32,
    pub channels: u8,
    pub waveform: Vec<f32>,
    pub spectrum: Vec<f32>,
    pub bands: AudioBands,
    pub smoothed_bands: AudioBands,
    pub beat: BeatObservation,
    /// The algorithm is named to prevent a direct DFT reference from being
    /// confused with an optimized runtime FFT implementation.
    pub spectrum_algorithm: &'static str,
    pub window_function: &'static str,
}

/// Caller-owned history for temporal analysis. Resetting or retaining it is an
/// explicit application/provider lifecycle decision.
#[derive(Clone, Copy, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct PcmAnalysisState {
    pub smoothed_bands: AudioBands,
    pub previous_energy: Option<f32>,
}

impl PcmAnalysisState {
    pub const fn reset(&mut self) {
        self.smoothed_bands = AudioBands {
            sub_bass: 0.0,
            bass: 0.0,
            low_mid: 0.0,
            mid: 0.0,
            high_mid: 0.0,
            treble: 0.0,
        };
        self.previous_energy = None;
    }
}

/// Explicit overload behavior for corpus-side PCM delivery.
///
/// Capture providers remain responsible for deciding whether this policy is
/// appropriate for their mechanism. The analysis contract only makes the
/// bounded queue and any loss visible to callers.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum PcmBacklogOverflowPolicy {
    DropOldest,
    DropNewest,
}

/// Result of attempting to queue one already-normalized PCM window.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct PcmBacklogPush {
    pub accepted: bool,
    pub pending_windows: usize,
    pub dropped_windows: u64,
}

/// Serializable state of a bounded PCM backlog without exposing its samples.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct PcmBacklogSnapshot {
    pub capacity: usize,
    pub overflow_policy: PcmBacklogOverflowPolicy,
    pub pending_windows: usize,
    pub dropped_windows: u64,
}

impl PcmBacklogSnapshot {
    pub fn to_structural_json(&self) -> Result<String, PcmAnalysisError> {
        serde_json::to_string_pretty(self)
            .map_err(|error| PcmAnalysisError::Serialization(error.to_string()))
    }
}

/// Native-host timing observation for a fixed, already-validated PCM window.
///
/// The observation describes one bounded workload. Its elapsed values are
/// deliberately not golden data: processor, build profile, and host load all
/// influence them. It exists to make analysis cost visible before an audio
/// device provider is introduced.
#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct PcmAnalysisTimingObservation {
    pub scope: &'static str,
    pub spectrum_algorithm: &'static str,
    pub window_function: &'static str,
    pub frame_count: usize,
    pub channels: u8,
    pub spectrum_bins: usize,
    pub iterations: usize,
    pub total_microseconds: u64,
    pub mean_microseconds: f64,
    pub maximum_microseconds: u64,
}

impl PcmAnalysisTimingObservation {
    pub fn to_observation_json(&self) -> Result<String, PcmAnalysisError> {
        serde_json::to_string_pretty(self)
            .map_err(|error| PcmAnalysisError::Serialization(error.to_string()))
    }
}

/// Portable structural working-set observation for one reference PCM analysis.
///
/// This is deliberately not an allocator profiler. It records the exact `f32`
/// slots implied by the current algorithm: the retained mono waveform and
/// spectrum plus the temporary Hann-window and direct-DFT spectrum buffers.
/// It excludes the caller-owned interleaved input, `Vec` capacity and allocator
/// overhead, analysis history, and all platform or audio-provider state.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
pub struct PcmAnalysisWorkingSetObservation {
    pub scope: &'static str,
    pub allocation_model: &'static str,
    pub frame_count: usize,
    pub channels: u8,
    pub spectrum_bins: usize,
    pub retained_waveform_f32_slots: usize,
    pub retained_spectrum_f32_slots: usize,
    pub transient_window_f32_slots: usize,
    pub transient_spectrum_f32_slots: usize,
    pub analyzer_owned_f32_slots: usize,
    pub analyzer_owned_bytes: usize,
}

impl PcmAnalysisWorkingSetObservation {
    pub fn to_observation_json(&self) -> Result<String, PcmAnalysisError> {
        serde_json::to_string_pretty(self)
            .map_err(|error| PcmAnalysisError::Serialization(error.to_string()))
    }
}

/// Observes the source-structural working set of the reference PCM analyzer.
///
/// The result is deterministic for a validated window and configuration. It
/// does not report actual allocation calls or claim a portable memory budget.
pub fn observe_pcm_analysis_working_set(
    window: &PcmAudioWindow,
    config: PcmAnalysisConfig,
) -> Result<PcmAnalysisWorkingSetObservation, PcmAnalysisError> {
    window.validate()?;
    config.validate()?;

    let frame_count = window.frame_count();
    let maximum_bins = frame_count / 2 + 1;
    if config.spectrum_bins > maximum_bins {
        return Err(PcmAnalysisError::SpectrumExceedsNyquist {
            bins: config.spectrum_bins,
            maximum: maximum_bins,
        });
    }

    let analyzer_owned_f32_slots = frame_count
        .saturating_mul(2)
        .saturating_add(config.spectrum_bins.saturating_mul(2));
    Ok(PcmAnalysisWorkingSetObservation {
        scope: "source-structural-working-set-not-allocation-profiler",
        allocation_model: "reference-pcm-analysis-f32-slots-v1",
        frame_count,
        channels: window.channels,
        spectrum_bins: config.spectrum_bins,
        retained_waveform_f32_slots: frame_count,
        retained_spectrum_f32_slots: config.spectrum_bins,
        transient_window_f32_slots: frame_count,
        transient_spectrum_f32_slots: config.spectrum_bins,
        analyzer_owned_f32_slots,
        analyzer_owned_bytes: analyzer_owned_f32_slots.saturating_mul(std::mem::size_of::<f32>()),
    })
}

/// Measures the current reference analysis implementation over a fixed window.
///
/// This is intentionally an observation helper, not a benchmark harness or a
/// runtime scheduler. It neither captures PCM nor sets an audio latency budget.
pub fn observe_pcm_analysis_timing(
    window: &PcmAudioWindow,
    config: PcmAnalysisConfig,
    iterations: usize,
) -> Result<PcmAnalysisTimingObservation, PcmAnalysisError> {
    if iterations == 0 || iterations > MAX_PCM_MEASUREMENT_ITERATIONS {
        return Err(PcmAnalysisError::InvalidMeasurementIterations {
            iterations,
            maximum: MAX_PCM_MEASUREMENT_ITERATIONS,
        });
    }
    window.validate()?;
    config.validate()?;

    let started = Instant::now();
    let mut maximum = Duration::ZERO;
    let mut energy_sum = 0.0_f32;
    for _ in 0..iterations {
        let iteration_started = Instant::now();
        let frame = PcmAnalyzer::analyze(window, config)?;
        maximum = maximum.max(iteration_started.elapsed());
        energy_sum += frame.beat.energy;
    }
    black_box(energy_sum);
    let elapsed = started.elapsed();
    let total_microseconds = elapsed.as_micros().min(u128::from(u64::MAX)) as u64;
    Ok(PcmAnalysisTimingObservation {
        scope: "native-host-observation-not-performance-contract",
        spectrum_algorithm: "direct-dft-magnitude-v1",
        window_function: "hann-v1",
        frame_count: window.frame_count(),
        channels: window.channels,
        spectrum_bins: config.spectrum_bins,
        iterations,
        total_microseconds,
        mean_microseconds: elapsed.as_secs_f64() * 1_000_000.0 / iterations as f64,
        maximum_microseconds: maximum.as_micros().min(u128::from(u64::MAX)) as u64,
    })
}

/// Bounded, caller-owned PCM backlog for deterministic corpus evidence.
///
/// This is intentionally not an audio-device ring buffer. It owns neither
/// callbacks nor clocks; it only records which explicit analysis windows were
/// retained or dropped before the caller drains them.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PcmAnalysisBacklog {
    capacity: usize,
    overflow_policy: PcmBacklogOverflowPolicy,
    pending: VecDeque<PcmAudioWindow>,
    dropped_windows: u64,
}

impl PcmAnalysisBacklog {
    pub fn new(
        capacity: usize,
        overflow_policy: PcmBacklogOverflowPolicy,
    ) -> Result<Self, PcmAnalysisError> {
        if capacity == 0 || capacity > MAX_PCM_PENDING_WINDOWS {
            return Err(PcmAnalysisError::InvalidBacklogCapacity {
                capacity,
                maximum: MAX_PCM_PENDING_WINDOWS,
            });
        }
        Ok(Self {
            capacity,
            overflow_policy,
            pending: VecDeque::with_capacity(capacity),
            dropped_windows: 0,
        })
    }

    pub const fn capacity(&self) -> usize {
        self.capacity
    }

    pub const fn overflow_policy(&self) -> PcmBacklogOverflowPolicy {
        self.overflow_policy
    }

    pub fn pending_windows(&self) -> usize {
        self.pending.len()
    }

    pub const fn dropped_windows(&self) -> u64 {
        self.dropped_windows
    }

    pub fn snapshot(&self) -> PcmBacklogSnapshot {
        PcmBacklogSnapshot {
            capacity: self.capacity,
            overflow_policy: self.overflow_policy,
            pending_windows: self.pending.len(),
            dropped_windows: self.dropped_windows,
        }
    }

    pub fn push(&mut self, window: PcmAudioWindow) -> Result<PcmBacklogPush, PcmAnalysisError> {
        window.validate()?;
        let mut accepted = true;
        if self.pending.len() == self.capacity {
            self.dropped_windows = self.dropped_windows.saturating_add(1);
            match self.overflow_policy {
                PcmBacklogOverflowPolicy::DropOldest => {
                    self.pending.pop_front();
                }
                PcmBacklogOverflowPolicy::DropNewest => accepted = false,
            }
        }
        if accepted {
            self.pending.push_back(window);
        }
        Ok(PcmBacklogPush {
            accepted,
            pending_windows: self.pending.len(),
            dropped_windows: self.dropped_windows,
        })
    }

    pub fn analyze_next(
        &mut self,
        config: PcmAnalysisConfig,
        state: &mut PcmAnalysisState,
    ) -> Result<Option<PcmAnalysisFrame>, PcmAnalysisError> {
        config.validate()?;
        let Some(window) = self.pending.pop_front() else {
            return Ok(None);
        };
        PcmAnalyzer::analyze_with_state(&window, config, state).map(Some)
    }
}

impl PcmAnalysisFrame {
    pub fn to_structural_json(&self) -> Result<String, PcmAnalysisError> {
        serde_json::to_string_pretty(self)
            .map_err(|error| PcmAnalysisError::Serialization(error.to_string()))
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PcmFixture {
    Silence,
    Impulse,
    ToneAtBin8,
    StereoToneAtBin8,
}

impl PcmFixture {
    pub const ALL: [Self; 4] = [
        Self::Silence,
        Self::Impulse,
        Self::ToneAtBin8,
        Self::StereoToneAtBin8,
    ];

    pub const fn label(self) -> &'static str {
        match self {
            Self::Silence => "pcm-silence",
            Self::Impulse => "pcm-impulse",
            Self::ToneAtBin8 => "pcm-tone-bin-8",
            Self::StereoToneAtBin8 => "pcm-stereo-tone-bin-8",
        }
    }

    pub fn window(self) -> PcmAudioWindow {
        const FRAMES: usize = 256;
        const SAMPLE_RATE_HZ: u32 = 48_000;
        match self {
            Self::Silence => PcmAudioWindow::new(vec![0.0; FRAMES], SAMPLE_RATE_HZ, 1),
            Self::Impulse => {
                let mut samples = vec![0.0; FRAMES];
                // The Hann window has zero weight at each endpoint. Center
                // the impulse so this fixture exercises the spectrum path.
                samples[FRAMES / 2] = 1.0;
                PcmAudioWindow::new(samples, SAMPLE_RATE_HZ, 1)
            }
            Self::ToneAtBin8 => {
                PcmAudioWindow::new(tone_samples(FRAMES, 8, 0.8), SAMPLE_RATE_HZ, 1)
            }
            Self::StereoToneAtBin8 => {
                let mono = tone_samples(FRAMES, 8, 0.8);
                let interleaved = mono.iter().flat_map(|sample| [*sample, *sample]).collect();
                PcmAudioWindow::new(interleaved, SAMPLE_RATE_HZ, 2)
            }
        }
        .expect("fixed PCM corpus fixture is valid")
    }
}

#[derive(Clone, Copy, Debug, Default)]
pub struct PcmAnalyzer;

impl PcmAnalyzer {
    pub fn analyze(
        window: &PcmAudioWindow,
        config: PcmAnalysisConfig,
    ) -> Result<PcmAnalysisFrame, PcmAnalysisError> {
        let mut state = PcmAnalysisState::default();
        Self::analyze_with_state(window, config, &mut state)
    }

    pub fn analyze_with_state(
        window: &PcmAudioWindow,
        config: PcmAnalysisConfig,
        state: &mut PcmAnalysisState,
    ) -> Result<PcmAnalysisFrame, PcmAnalysisError> {
        window.validate()?;
        config.validate()?;
        let waveform = window.mono_samples();
        let maximum_bins = waveform.len() / 2 + 1;
        if config.spectrum_bins > maximum_bins {
            return Err(PcmAnalysisError::SpectrumExceedsNyquist {
                bins: config.spectrum_bins,
                maximum: maximum_bins,
            });
        }
        let windowed = hann_window(&waveform);
        let spectrum = direct_dft_magnitude(&windowed, config.spectrum_bins);
        let energy = (waveform.iter().map(|sample| sample * sample).sum::<f32>()
            / waveform.len() as f32)
            .sqrt()
            .clamp(0.0, 1.0);
        let bands = bands_from_spectrum(&spectrum);
        let smoothed_bands = blend_bands(bands, state.smoothed_bands, config.band_smoothing);
        let onset = energy >= config.onset_energy_threshold
            && state
                .previous_energy
                .is_none_or(|previous| energy - previous >= config.onset_energy_delta);
        let beat = BeatObservation {
            energy,
            pulse: energy,
            onset,
        };
        state.smoothed_bands = smoothed_bands;
        state.previous_energy = Some(energy);
        Ok(PcmAnalysisFrame {
            sample_rate_hz: window.sample_rate_hz,
            channels: window.channels,
            waveform,
            spectrum: spectrum.clone(),
            bands,
            smoothed_bands,
            beat,
            spectrum_algorithm: "direct-dft-magnitude-v1",
            window_function: "hann-v1",
        })
    }
}

#[derive(Clone, Debug, Error, PartialEq)]
pub enum PcmAnalysisError {
    #[error("PCM sample rate must be non-zero")]
    EmptySampleRate,
    #[error("PCM channel count {channels} is unsupported; maximum is {maximum}")]
    UnsupportedChannelCount { channels: u8, maximum: u8 },
    #[error("PCM window must contain at least one complete frame")]
    EmptyWindow,
    #[error("PCM sample count {samples} does not contain complete {channels}-channel frames")]
    IncompleteInterleavedFrame { samples: usize, channels: u8 },
    #[error("PCM window has {frames} frames; maximum is {maximum}")]
    TooManyFrames { frames: usize, maximum: usize },
    #[error("PCM sample {index} must be finite and normalized within [-1, 1], received {sample}")]
    InvalidSample { index: usize, sample: f32 },
    #[error("spectrum bin count {bins} is invalid; maximum is {maximum}")]
    InvalidSpectrumBinCount { bins: usize, maximum: usize },
    #[error("onset energy threshold must be finite and within [0, 1], received {threshold}")]
    InvalidOnsetThreshold { threshold: f32 },
    #[error("{field} must be finite and within [0, 1], received {value}")]
    InvalidUnitConfiguration { field: &'static str, value: f32 },
    #[error("spectrum requests {bins} bins but this PCM window supports at most {maximum}")]
    SpectrumExceedsNyquist { bins: usize, maximum: usize },
    #[error("PCM backlog capacity {capacity} is invalid; maximum is {maximum}")]
    InvalidBacklogCapacity { capacity: usize, maximum: usize },
    #[error("PCM measurement iteration count {iterations} is invalid; maximum is {maximum}")]
    InvalidMeasurementIterations { iterations: usize, maximum: usize },
    #[error("could not serialize PCM analysis: {0}")]
    Serialization(String),
}

fn tone_samples(frame_count: usize, bin: usize, amplitude: f32) -> Vec<f32> {
    (0..frame_count)
        .map(|index| (TAU * bin as f32 * index as f32 / frame_count as f32).sin() * amplitude)
        .collect()
}

fn hann_window(samples: &[f32]) -> Vec<f32> {
    if samples.len() == 1 {
        return samples.to_vec();
    }
    samples
        .iter()
        .enumerate()
        .map(|(index, sample)| {
            let weight =
                0.5 - 0.5 * (TAU * index as f32 / (samples.len().saturating_sub(1)) as f32).cos();
            sample * weight
        })
        .collect()
}

fn direct_dft_magnitude(samples: &[f32], bin_count: usize) -> Vec<f32> {
    let sample_count = samples.len() as f32;
    (0..bin_count)
        .map(|bin| {
            let (real, imaginary) = samples.iter().enumerate().fold(
                (0.0_f32, 0.0_f32),
                |(real, imaginary), (index, sample)| {
                    let phase = TAU * bin as f32 * index as f32 / sample_count;
                    (
                        real + sample * phase.cos(),
                        imaginary - sample * phase.sin(),
                    )
                },
            );
            ((real * real + imaginary * imaginary).sqrt() * 2.0 / sample_count).clamp(0.0, 1.0)
        })
        .collect()
}

fn blend_bands(current: AudioBands, previous: AudioBands, retained: f32) -> AudioBands {
    let blend = |current: f32, previous: f32| current * (1.0 - retained) + previous * retained;
    AudioBands {
        sub_bass: blend(current.sub_bass, previous.sub_bass),
        bass: blend(current.bass, previous.bass),
        low_mid: blend(current.low_mid, previous.low_mid),
        mid: blend(current.mid, previous.mid),
        high_mid: blend(current.high_mid, previous.high_mid),
        treble: blend(current.treble, previous.treble),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fixed_pcm_analysis_is_deterministic() {
        let window = PcmFixture::ToneAtBin8.window();
        let first = PcmAnalyzer::analyze(&window, PcmAnalysisConfig::default()).unwrap();
        let second = PcmAnalyzer::analyze(&window, PcmAnalysisConfig::default()).unwrap();
        assert_eq!(first, second);
        assert_eq!(
            first.to_structural_json().unwrap(),
            second.to_structural_json().unwrap()
        );
    }

    #[test]
    fn silence_is_zero_energy_without_onset() {
        let frame =
            PcmAnalyzer::analyze(&PcmFixture::Silence.window(), PcmAnalysisConfig::default())
                .unwrap();
        assert_eq!(frame.beat, BeatObservation::default());
        assert!(frame.spectrum.iter().all(|value| *value == 0.0));
    }

    #[test]
    fn tone_fixture_peaks_at_its_exact_dft_bin() {
        let frame = PcmAnalyzer::analyze(
            &PcmFixture::ToneAtBin8.window(),
            PcmAnalysisConfig::default(),
        )
        .unwrap();
        let peak = frame
            .spectrum
            .iter()
            .enumerate()
            .max_by(|left, right| left.1.total_cmp(right.1))
            .unwrap();
        assert_eq!(peak.0, 8);
        assert!(frame.beat.onset);
    }

    #[test]
    fn centered_impulse_survives_the_declared_hann_window() {
        let frame =
            PcmAnalyzer::analyze(&PcmFixture::Impulse.window(), PcmAnalysisConfig::default())
                .unwrap();
        assert!(frame.spectrum.iter().any(|value| *value > 0.0));
    }

    #[test]
    fn stereo_mix_matches_an_equivalent_mono_fixture() {
        let config = PcmAnalysisConfig::default();
        let mono = PcmAnalyzer::analyze(&PcmFixture::ToneAtBin8.window(), config).unwrap();
        let stereo = PcmAnalyzer::analyze(&PcmFixture::StereoToneAtBin8.window(), config).unwrap();
        assert_eq!(mono.waveform, stereo.waveform);
        assert_eq!(mono.spectrum, stereo.spectrum);
    }

    #[test]
    fn malformed_pcm_fails_before_analysis() {
        assert!(matches!(
            PcmAudioWindow::new(vec![1.2], 48_000, 1),
            Err(PcmAnalysisError::InvalidSample { .. })
        ));
        assert!(matches!(
            PcmAudioWindow::new(vec![0.0, 0.0, 0.0], 48_000, 2),
            Err(PcmAnalysisError::IncompleteInterleavedFrame { .. })
        ));
    }

    #[test]
    fn smoothing_and_onset_use_only_explicit_caller_owned_history() {
        let config = PcmAnalysisConfig {
            band_smoothing: 0.5,
            onset_energy_delta: 0.1,
            ..PcmAnalysisConfig::default()
        };
        let mut state = PcmAnalysisState::default();
        let tone = PcmFixture::ToneAtBin8.window();
        let first = PcmAnalyzer::analyze_with_state(&tone, config, &mut state).unwrap();
        let second = PcmAnalyzer::analyze_with_state(&tone, config, &mut state).unwrap();
        assert!(first.beat.onset);
        assert!(!second.beat.onset);
        assert!(second.smoothed_bands.bass > first.smoothed_bands.bass);

        state.reset();
        assert!(
            PcmAnalyzer::analyze_with_state(&tone, config, &mut state)
                .unwrap()
                .beat
                .onset
        );
    }

    #[test]
    fn backlog_drop_oldest_retains_the_newest_windows_with_explicit_loss() {
        let mut backlog = PcmAnalysisBacklog::new(2, PcmBacklogOverflowPolicy::DropOldest).unwrap();
        for sample in [0.1, 0.2, 0.3] {
            backlog
                .push(PcmAudioWindow::new(vec![sample], 48_000, 1).unwrap())
                .unwrap();
        }
        assert_eq!(backlog.pending_windows(), 2);
        assert_eq!(backlog.dropped_windows(), 1);
        let config = PcmAnalysisConfig {
            spectrum_bins: 1,
            ..PcmAnalysisConfig::default()
        };
        let mut state = PcmAnalysisState::default();
        assert_eq!(
            backlog
                .analyze_next(config, &mut state)
                .unwrap()
                .unwrap()
                .waveform,
            vec![0.2]
        );
        assert_eq!(
            backlog
                .analyze_next(config, &mut state)
                .unwrap()
                .unwrap()
                .waveform,
            vec![0.3]
        );
        assert!(backlog.analyze_next(config, &mut state).unwrap().is_none());
    }

    #[test]
    fn backlog_drop_newest_preserves_existing_order_and_reports_rejection() {
        let mut backlog = PcmAnalysisBacklog::new(2, PcmBacklogOverflowPolicy::DropNewest).unwrap();
        for sample in [0.1, 0.2] {
            assert!(
                backlog
                    .push(PcmAudioWindow::new(vec![sample], 48_000, 1).unwrap())
                    .unwrap()
                    .accepted
            );
        }
        let rejected = backlog
            .push(PcmAudioWindow::new(vec![0.3], 48_000, 1).unwrap())
            .unwrap();
        assert!(!rejected.accepted);
        assert_eq!(rejected.dropped_windows, 1);
        let config = PcmAnalysisConfig {
            spectrum_bins: 1,
            ..PcmAnalysisConfig::default()
        };
        let mut state = PcmAnalysisState::default();
        assert_eq!(
            backlog
                .analyze_next(config, &mut state)
                .unwrap()
                .unwrap()
                .waveform,
            vec![0.1]
        );
    }

    #[test]
    fn backlog_capacity_is_bounded_before_any_window_is_queued() {
        assert!(matches!(
            PcmAnalysisBacklog::new(0, PcmBacklogOverflowPolicy::DropOldest),
            Err(PcmAnalysisError::InvalidBacklogCapacity { capacity: 0, .. })
        ));
        assert!(matches!(
            PcmAnalysisBacklog::new(
                MAX_PCM_PENDING_WINDOWS + 1,
                PcmBacklogOverflowPolicy::DropNewest
            ),
            Err(PcmAnalysisError::InvalidBacklogCapacity { .. })
        ));
    }

    #[test]
    fn timing_observation_records_a_bounded_reference_workload() {
        let observation = observe_pcm_analysis_timing(
            &PcmFixture::ToneAtBin8.window(),
            PcmAnalysisConfig::default(),
            2,
        )
        .unwrap();
        assert_eq!(
            observation.scope,
            "native-host-observation-not-performance-contract"
        );
        assert_eq!(observation.spectrum_algorithm, "direct-dft-magnitude-v1");
        assert_eq!(observation.frame_count, 256);
        assert_eq!(observation.iterations, 2);
        assert!(observation
            .to_observation_json()
            .unwrap()
            .contains("iterations"));
    }

    #[test]
    fn timing_observation_rejects_unbounded_iteration_counts() {
        let window = PcmFixture::Silence.window();
        assert!(matches!(
            observe_pcm_analysis_timing(&window, PcmAnalysisConfig::default(), 0),
            Err(PcmAnalysisError::InvalidMeasurementIterations { iterations: 0, .. })
        ));
        assert!(matches!(
            observe_pcm_analysis_timing(
                &window,
                PcmAnalysisConfig::default(),
                MAX_PCM_MEASUREMENT_ITERATIONS + 1
            ),
            Err(PcmAnalysisError::InvalidMeasurementIterations { .. })
        ));
    }

    #[test]
    fn working_set_observation_records_reference_buffer_slots_without_allocator_claims() {
        let window = PcmAudioWindow::new(vec![0.0; 8], 48_000, 1).unwrap();
        let config = PcmAnalysisConfig {
            spectrum_bins: 4,
            ..PcmAnalysisConfig::default()
        };

        let observation = observe_pcm_analysis_working_set(&window, config).unwrap();

        assert_eq!(
            observation.scope,
            "source-structural-working-set-not-allocation-profiler"
        );
        assert_eq!(observation.frame_count, 8);
        assert_eq!(observation.retained_waveform_f32_slots, 8);
        assert_eq!(observation.retained_spectrum_f32_slots, 4);
        assert_eq!(observation.transient_window_f32_slots, 8);
        assert_eq!(observation.transient_spectrum_f32_slots, 4);
        assert_eq!(observation.analyzer_owned_f32_slots, 24);
        assert_eq!(observation.analyzer_owned_bytes, 96);
    }

    #[test]
    fn working_set_observation_matches_analysis_nyquist_validation() {
        let window = PcmAudioWindow::new(vec![0.0; 8], 48_000, 1).unwrap();
        let config = PcmAnalysisConfig {
            spectrum_bins: 6,
            ..PcmAnalysisConfig::default()
        };

        assert!(matches!(
            observe_pcm_analysis_working_set(&window, config),
            Err(PcmAnalysisError::SpectrumExceedsNyquist {
                bins: 6,
                maximum: 5
            })
        ));
    }

    #[test]
    fn backlog_snapshot_is_compact_and_deterministic() {
        let mut backlog = PcmAnalysisBacklog::new(1, PcmBacklogOverflowPolicy::DropNewest).unwrap();
        backlog.push(PcmFixture::Silence.window()).unwrap();
        backlog.push(PcmFixture::Impulse.window()).unwrap();
        assert_eq!(
            backlog.snapshot(),
            PcmBacklogSnapshot {
                capacity: 1,
                overflow_policy: PcmBacklogOverflowPolicy::DropNewest,
                pending_windows: 1,
                dropped_windows: 1,
            }
        );
        assert!(backlog
            .snapshot()
            .to_structural_json()
            .unwrap()
            .contains("drop-newest"));
    }
}
