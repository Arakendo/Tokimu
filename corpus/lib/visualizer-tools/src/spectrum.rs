//! Deterministic spectrum-bar lowering for visualizer corpus evidence.
//!
//! This module turns normalized spectrum observations into bounded rectangle
//! geometry. It owns neither renderer meshes nor MilkDrop custom-shape syntax.

use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::VisualizerFrameInput;

pub const MAX_SPECTRUM_BARS: usize = 128;

/// Provider-neutral rectangle geometry for a spectrum-bar visualizer proof.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct VisualizerSpectrumBars {
    /// Normalized rectangles ordered from the lowest to highest source band.
    pub bars: Vec<VisualizerSpectrumBar>,
    /// Shared normalized baseline for every bar.
    pub baseline: f32,
}

/// One normalized spectrum-bar rectangle.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct VisualizerSpectrumBar {
    /// Inclusive lower-left normalized position.
    pub minimum: [f32; 2],
    /// Inclusive upper-right normalized position.
    pub maximum: [f32; 2],
    /// The normalized aggregate source energy before geometry lowering.
    pub energy: f32,
}

impl VisualizerSpectrumBars {
    /// Lowers one valid spectrum into a bounded sequence of normalized bars.
    ///
    /// Source bins are grouped in stable source order. The result deliberately
    /// stays renderer-neutral so another consumer may turn it into quads,
    /// instanced primitives, SVG rectangles, or diagnostic JSON.
    pub fn from_frame(
        frame: &VisualizerFrameInput,
        requested_bars: usize,
        gain: f32,
    ) -> Result<Self, VisualizerSpectrumBarsError> {
        frame.validate()?;
        if !(1..=MAX_SPECTRUM_BARS).contains(&requested_bars) {
            return Err(VisualizerSpectrumBarsError::InvalidBarCount { requested_bars });
        }
        if !gain.is_finite() || !(0.0..=4.0).contains(&gain) {
            return Err(VisualizerSpectrumBarsError::InvalidGain { gain });
        }
        let count = requested_bars.min(frame.spectrum.len());
        let cell_width = 2.0 / count as f32;
        let inset = (cell_width * 0.12).min(cell_width * 0.4);
        let bars = (0..count)
            .map(|index| {
                let start = index * frame.spectrum.len() / count;
                let end = ((index + 1) * frame.spectrum.len() / count).max(start + 1);
                let energy = frame.spectrum[start..end]
                    .iter()
                    .copied()
                    .fold(0.0_f32, f32::max);
                let height = (energy * gain).clamp(0.0, 1.0) * 2.0;
                let left = -1.0 + index as f32 * cell_width + inset;
                let right = -1.0 + (index + 1) as f32 * cell_width - inset;
                VisualizerSpectrumBar {
                    minimum: [left, -1.0],
                    maximum: [right, -1.0 + height],
                    energy,
                }
            })
            .collect();

        Ok(Self {
            bars,
            baseline: -1.0,
        })
    }

    pub fn to_structural_json(&self) -> Result<String, VisualizerSpectrumBarsError> {
        serde_json::to_string_pretty(self)
            .map_err(|error| VisualizerSpectrumBarsError::Serialization(error.to_string()))
    }
}

#[derive(Clone, Debug, Error, PartialEq)]
pub enum VisualizerSpectrumBarsError {
    #[error(transparent)]
    Input(#[from] crate::VisualizerInputError),
    #[error("spectrum-bar lowering requires between one and {MAX_SPECTRUM_BARS} bars, received {requested_bars}")]
    InvalidBarCount { requested_bars: usize },
    #[error(
        "spectrum-bar lowering gain must be finite and between zero and four, received {gain}"
    )]
    InvalidGain { gain: f32 },
    #[error("could not serialize spectrum-bar evidence: {0}")]
    Serialization(String),
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        SyntheticAudioFixture, SyntheticVisualizerConfig, SyntheticVisualizerInput,
        VisualizerViewport,
    };

    fn fixture_frame() -> VisualizerFrameInput {
        SyntheticVisualizerInput::new(
            SyntheticAudioFixture::FrequencySweep,
            SyntheticVisualizerConfig::default(),
        )
        .unwrap()
        .frame(24, VisualizerViewport::new(640, 360).unwrap())
        .unwrap()
    }

    #[test]
    fn spectrum_bar_lowering_is_deterministic_and_bounded() {
        let frame = fixture_frame();
        let first = VisualizerSpectrumBars::from_frame(&frame, 16, 1.0).unwrap();
        let second = VisualizerSpectrumBars::from_frame(&frame, 16, 1.0).unwrap();

        assert_eq!(first, second);
        assert_eq!(first.bars.len(), 16);
        assert!(first.bars.iter().all(|bar| {
            bar.minimum[0].is_finite()
                && bar.maximum[0].is_finite()
                && (-1.0..=1.0).contains(&bar.minimum[0])
                && (-1.0..=1.0).contains(&bar.maximum[0])
                && bar.minimum[0] < bar.maximum[0]
                && bar.minimum[1] == -1.0
                && (-1.0..=1.0).contains(&bar.maximum[1])
                && (0.0..=1.0).contains(&bar.energy)
        }));
        assert!(first
            .bars
            .windows(2)
            .all(|pair| pair[0].maximum[0] < pair[1].minimum[0]));
    }

    #[test]
    fn spectrum_bar_lowering_rejects_invalid_parameters() {
        let frame = fixture_frame();
        assert!(matches!(
            VisualizerSpectrumBars::from_frame(&frame, 0, 1.0),
            Err(VisualizerSpectrumBarsError::InvalidBarCount { .. })
        ));
        assert!(matches!(
            VisualizerSpectrumBars::from_frame(&frame, 8, 4.1),
            Err(VisualizerSpectrumBarsError::InvalidGain { .. })
        ));
    }

    #[test]
    fn spectrum_bar_lowering_preserves_the_input_empty_spectrum_diagnostic() {
        let mut frame = fixture_frame();
        frame.spectrum.clear();

        assert!(matches!(
            VisualizerSpectrumBars::from_frame(&frame, 8, 1.0),
            Err(VisualizerSpectrumBarsError::Input(
                crate::VisualizerInputError::EmptyBuffer { buffer: "spectrum" }
            ))
        ));
    }
}
