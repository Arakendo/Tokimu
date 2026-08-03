//! Bounded extraction of selected MilkDrop custom-wave scalar properties.
//!
//! This module intentionally accepts only literal properties in real
//! `[wave_N]` sections. `wavecode_N` and any unfamiliar property remain
//! source-visible unsupported constructs; they are not silently evaluated.

use std::collections::BTreeSet;

use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::{section_index, MilkDropConstruct, MilkDropPresetDocument, MilkDropSourceLocation};

pub const MAX_CUSTOM_WAVE_SAMPLES: u16 = 1_024;
const MIN_CUSTOM_WAVE_SAMPLES: u16 = 16;

/// Provider-neutral scalar description of one selected custom MilkDrop wave.
///
/// It is presentation input only. It owns no sample source, renderer mesh,
/// shader, texture, or custom equation execution.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct MilkDropCustomWave {
    pub index: u8,
    pub enabled: bool,
    pub samples: u16,
    pub spectrum: bool,
    pub dots: bool,
    pub thick: bool,
    pub additive: bool,
    pub scaling: f32,
    pub color: [f32; 4],
    /// Normalized presentation center in `[0, 1]` source coordinates.
    pub center: [f32; 2],
}

/// Which explicit audio observation supplied a lowered custom wave.
///
/// This is deliberately a source-selection record, not an executable
/// MilkDrop expression. Per-point `wavecode` remains unsupported.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub enum MilkDropCustomWaveSampleSource {
    Waveform,
    Spectrum,
}

/// Renderer-neutral points for one selected literal custom-wave description.
///
/// Points are normalized to the presentation unit square. They have no GPU
/// resource, mesh, shader binding, or line-rasterization policy attached.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct MilkDropCustomWaveFrame {
    pub wave: MilkDropCustomWave,
    pub source: MilkDropCustomWaveSampleSource,
    pub points: Vec<[f32; 2]>,
}

impl MilkDropCustomWave {
    fn defaults(index: u8) -> Self {
        Self {
            index,
            enabled: true,
            samples: 512,
            spectrum: false,
            dots: false,
            thick: false,
            additive: false,
            scaling: 1.0,
            color: [1.0, 1.0, 1.0, 1.0],
            center: [0.5, 0.5],
        }
    }
}

/// Resolves bounded custom-wave properties in source order.
///
/// Only a section named exactly `wave_N` is considered. Repeated selected
/// properties are rejected rather than allowing accidental source-order
/// dependence. Sections with no selected properties are omitted.
pub fn resolve_selected_custom_waves(
    document: &MilkDropPresetDocument,
) -> Result<Vec<MilkDropCustomWave>, MilkDropCustomWaveError> {
    let mut waves = Vec::new();

    for section in &document.sections {
        let Some(index) = section_index(&section.name, "wave_") else {
            continue;
        };
        let mut wave = MilkDropCustomWave::defaults(index);
        let mut seen = BTreeSet::new();
        let mut selected = false;

        for entry in &section.entries {
            if entry.construct != MilkDropConstruct::SelectedCustomWaveParameter {
                continue;
            }
            selected = true;
            if !seen.insert(entry.key.as_str()) {
                return Err(MilkDropCustomWaveError::DuplicateProperty {
                    line: entry.location.line,
                    key: entry.key.clone(),
                });
            }
            apply_property(&mut wave, &entry.key, &entry.value, entry.location)?;
        }

        if selected {
            waves.push(wave);
        }
    }

    Ok(waves)
}

/// Lowers selected literal custom-wave descriptions against explicit audio
/// observations.
///
/// The admitted subset only resamples the requested source, then applies the
/// selected wave's static center and scale. It does not execute `wavecode`,
/// synthesize a mesh, or infer a renderer line style. Disabled waves are
/// omitted. A selected source with fewer than two samples is rejected instead
/// of producing an ambiguous degenerate line.
pub fn lower_selected_custom_waves(
    waves: &[MilkDropCustomWave],
    waveform: &[f32],
    spectrum: &[f32],
) -> Result<Vec<MilkDropCustomWaveFrame>, MilkDropCustomWaveError> {
    let mut frames = Vec::new();
    for wave in waves.iter().filter(|wave| wave.enabled) {
        let (source, values) = if wave.spectrum {
            (MilkDropCustomWaveSampleSource::Spectrum, spectrum)
        } else {
            (MilkDropCustomWaveSampleSource::Waveform, waveform)
        };
        if values.len() < 2 {
            return Err(MilkDropCustomWaveError::InsufficientSamples {
                wave: wave.index,
                sample_source: source,
                count: values.len(),
            });
        }
        if values.iter().any(|value| !value.is_finite()) {
            return Err(MilkDropCustomWaveError::NonFiniteSamples {
                wave: wave.index,
                sample_source: source,
            });
        }

        let point_count = usize::from(wave.samples);
        let points = (0..point_count)
            .map(|index| {
                let unit = index as f32 / (point_count - 1) as f32;
                let source_value = resample(values, unit);
                let normalized_value = match source {
                    MilkDropCustomWaveSampleSource::Waveform => source_value.clamp(-1.0, 1.0),
                    MilkDropCustomWaveSampleSource::Spectrum => {
                        source_value.clamp(0.0, 1.0) * 2.0 - 1.0
                    }
                };
                [
                    wave.center[0] + (unit - 0.5) * wave.scaling,
                    wave.center[1] - normalized_value * 0.5 * wave.scaling,
                ]
            })
            .collect();
        frames.push(MilkDropCustomWaveFrame {
            wave: wave.clone(),
            source,
            points,
        });
    }
    Ok(frames)
}

fn resample(values: &[f32], unit: f32) -> f32 {
    let source_position = unit * (values.len() - 1) as f32;
    let lower = source_position.floor() as usize;
    let upper = (lower + 1).min(values.len() - 1);
    let fraction = source_position - lower as f32;
    values[lower] + (values[upper] - values[lower]) * fraction
}

fn apply_property(
    wave: &mut MilkDropCustomWave,
    key: &str,
    value: &str,
    location: MilkDropSourceLocation,
) -> Result<(), MilkDropCustomWaveError> {
    match key {
        "enabled" => wave.enabled = parse_flag(key, value, location.line)?,
        "samples" => {
            let samples = parse_u16(key, value, location.line)?;
            if !(MIN_CUSTOM_WAVE_SAMPLES..=MAX_CUSTOM_WAVE_SAMPLES).contains(&samples) {
                return Err(MilkDropCustomWaveError::OutOfRange {
                    line: location.line,
                    key: key.to_owned(),
                    value: value.to_owned(),
                    minimum: f32::from(MIN_CUSTOM_WAVE_SAMPLES),
                    maximum: f32::from(MAX_CUSTOM_WAVE_SAMPLES),
                });
            }
            wave.samples = samples;
        }
        "bspectrum" => wave.spectrum = parse_flag(key, value, location.line)?,
        "busedots" => wave.dots = parse_flag(key, value, location.line)?,
        "bdrawthick" => wave.thick = parse_flag(key, value, location.line)?,
        "badditive" => wave.additive = parse_flag(key, value, location.line)?,
        "scaling" => wave.scaling = parse_range(key, value, location.line, 0.0, 4.0)?,
        "r" => wave.color[0] = parse_range(key, value, location.line, 0.0, 1.0)?,
        "g" => wave.color[1] = parse_range(key, value, location.line, 0.0, 1.0)?,
        "b" => wave.color[2] = parse_range(key, value, location.line, 0.0, 1.0)?,
        "a" => wave.color[3] = parse_range(key, value, location.line, 0.0, 1.0)?,
        "x" => wave.center[0] = parse_range(key, value, location.line, 0.0, 1.0)?,
        "y" => wave.center[1] = parse_range(key, value, location.line, 0.0, 1.0)?,
        _ => unreachable!("selected custom-wave classification must remain synchronized"),
    }
    Ok(())
}

fn parse_flag(key: &str, value: &str, line: usize) -> Result<bool, MilkDropCustomWaveError> {
    match parse_finite(key, value, line)? {
        0.0 => Ok(false),
        1.0 => Ok(true),
        _ => Err(MilkDropCustomWaveError::InvalidFlag {
            line,
            key: key.to_owned(),
            value: value.to_owned(),
        }),
    }
}

fn parse_u16(key: &str, value: &str, line: usize) -> Result<u16, MilkDropCustomWaveError> {
    let parsed = parse_finite(key, value, line)?;
    if parsed.fract() != 0.0 || !(0.0..=f32::from(u16::MAX)).contains(&parsed) {
        return Err(MilkDropCustomWaveError::InvalidInteger {
            line,
            key: key.to_owned(),
            value: value.to_owned(),
        });
    }
    Ok(parsed as u16)
}

fn parse_range(
    key: &str,
    value: &str,
    line: usize,
    minimum: f32,
    maximum: f32,
) -> Result<f32, MilkDropCustomWaveError> {
    let parsed = parse_finite(key, value, line)?;
    if !(minimum..=maximum).contains(&parsed) {
        return Err(MilkDropCustomWaveError::OutOfRange {
            line,
            key: key.to_owned(),
            value: value.to_owned(),
            minimum,
            maximum,
        });
    }
    Ok(parsed)
}

fn parse_finite(key: &str, value: &str, line: usize) -> Result<f32, MilkDropCustomWaveError> {
    let parsed =
        value
            .trim()
            .parse::<f32>()
            .map_err(|_| MilkDropCustomWaveError::InvalidValue {
                line,
                key: key.to_owned(),
                value: value.to_owned(),
            })?;
    if parsed.is_finite() {
        Ok(parsed)
    } else {
        Err(MilkDropCustomWaveError::InvalidValue {
            line,
            key: key.to_owned(),
            value: value.to_owned(),
        })
    }
}

#[derive(Clone, Debug, Error, PartialEq)]
pub enum MilkDropCustomWaveError {
    #[error("MilkDrop custom-wave property `{key}` is declared more than once; second declaration is at line {line}")]
    DuplicateProperty { line: usize, key: String },
    #[error("MilkDrop custom-wave property `{key}` at line {line} is not a finite numeric value `{value}`")]
    InvalidValue {
        line: usize,
        key: String,
        value: String,
    },
    #[error("MilkDrop custom-wave property `{key}` at line {line} must be an integer, received `{value}`")]
    InvalidInteger {
        line: usize,
        key: String,
        value: String,
    },
    #[error("MilkDrop custom-wave property `{key}` at line {line} must be zero or one, received `{value}`")]
    InvalidFlag {
        line: usize,
        key: String,
        value: String,
    },
    #[error("MilkDrop custom-wave property `{key}` at line {line} must be between {minimum} and {maximum}, received `{value}`")]
    OutOfRange {
        line: usize,
        key: String,
        value: String,
        minimum: f32,
        maximum: f32,
    },
    #[error(
        "MilkDrop custom wave {wave} requires at least two {sample_source:?} samples, received {count}"
    )]
    InsufficientSamples {
        wave: u8,
        sample_source: MilkDropCustomWaveSampleSource,
        count: usize,
    },
    #[error("MilkDrop custom wave {wave} received non-finite {sample_source:?} samples")]
    NonFiniteSamples {
        wave: u8,
        sample_source: MilkDropCustomWaveSampleSource,
    },
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::MilkDropPresetDocument;

    #[test]
    fn resolves_a_bounded_custom_wave_without_executing_code() {
        let document = MilkDropPresetDocument::parse(
            "[preset00]\nfDecay=0.98\n[wave_2]\nenabled=1\nsamples=64\nbSpectrum=1\nbUseDots=0\nbDrawThick=1\nbAdditive=1\nscaling=1.5\nr=0.2\ng=0.4\nb=0.6\na=0.8\nx=0.25\ny=0.75\nwavecode_2=sample=sample*0.5;",
        )
        .unwrap();
        let waves = resolve_selected_custom_waves(&document).unwrap();

        assert_eq!(waves.len(), 1);
        assert_eq!(waves[0].index, 2);
        assert_eq!(waves[0].samples, 64);
        assert!(waves[0].spectrum);
        assert!(waves[0].thick);
        assert!(waves[0].additive);
        assert_eq!(waves[0].color, [0.2, 0.4, 0.6, 0.8]);
        assert_eq!(waves[0].center, [0.25, 0.75]);
        assert_eq!(document.unsupported_entries, 1);
    }

    #[test]
    fn custom_wave_rejects_out_of_range_properties_at_the_source_line() {
        let document = MilkDropPresetDocument::parse("[wave_0]\nsamples=4").unwrap();
        assert!(matches!(
            resolve_selected_custom_waves(&document),
            Err(MilkDropCustomWaveError::OutOfRange { line: 2, .. })
        ));
    }

    #[test]
    fn prefixed_legacy_keys_remain_explicitly_unsupported() {
        let document = MilkDropPresetDocument::parse("[preset00]\nwave_0_enabled=1").unwrap();
        assert_eq!(
            document.sections[0].entries[0].construct,
            MilkDropConstruct::UnsupportedCustomWave
        );
    }

    #[test]
    fn selected_wave_lowering_resamples_explicit_audio_without_executing_code() {
        let document =
            MilkDropPresetDocument::parse("[wave_0]\nsamples=16\nscaling=1\nx=0.5\ny=0.5").unwrap();
        let waves = resolve_selected_custom_waves(&document).unwrap();
        let frames = lower_selected_custom_waves(&waves, &[-1.0, 1.0], &[0.0, 1.0]).unwrap();

        assert_eq!(frames.len(), 1);
        assert_eq!(frames[0].source, MilkDropCustomWaveSampleSource::Waveform);
        assert_eq!(frames[0].points.len(), 16);
        assert_eq!(frames[0].points.first(), Some(&[0.0, 1.0]));
        assert_eq!(frames[0].points.last(), Some(&[1.0, 0.0]));
    }

    #[test]
    fn selected_wave_lowering_requires_the_explicit_requested_source() {
        let document = MilkDropPresetDocument::parse("[wave_0]\nsamples=16\nbSpectrum=1").unwrap();
        let waves = resolve_selected_custom_waves(&document).unwrap();
        assert!(matches!(
            lower_selected_custom_waves(&waves, &[0.0, 1.0], &[]),
            Err(MilkDropCustomWaveError::InsufficientSamples {
                sample_source: MilkDropCustomWaveSampleSource::Spectrum,
                ..
            })
        ));
    }
}
