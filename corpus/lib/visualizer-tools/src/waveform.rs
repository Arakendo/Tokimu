//! Deterministic waveform lowering for visualizer corpus evidence.
//!
//! This module turns an already-normalized audio observation into simple line
//! geometry. It owns neither a renderer mesh nor a MilkDrop waveform object.

use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::VisualizerFrameInput;

/// Provider-neutral line-strip geometry for one built-in waveform proof.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct VisualizerWaveform {
    /// Normalized `[-1, 1]` positions in source sample order.
    pub points: Vec<[f32; 2]>,
    /// A bounded presentation hint for a later renderer-facing consumer.
    pub thickness: f32,
}

impl VisualizerWaveform {
    /// Lowers the explicit waveform samples from one valid visualizer frame.
    pub fn from_frame(
        frame: &VisualizerFrameInput,
        gain: f32,
    ) -> Result<Self, VisualizerWaveformError> {
        frame.validate()?;
        if !gain.is_finite() || !(0.0..=4.0).contains(&gain) {
            return Err(VisualizerWaveformError::InvalidGain { gain });
        }
        if frame.waveform.len() < 2 {
            return Err(VisualizerWaveformError::TooFewSamples {
                count: frame.waveform.len(),
            });
        }
        let last = (frame.waveform.len() - 1) as f32;
        let points = frame
            .waveform
            .iter()
            .enumerate()
            .map(|(index, sample)| {
                let horizontal = index as f32 / last * 2.0 - 1.0;
                let vertical = (*sample * gain).clamp(-1.0, 1.0);
                [horizontal, vertical]
            })
            .collect();
        Ok(Self {
            points,
            thickness: 1.0,
        })
    }

    pub fn to_structural_json(&self) -> Result<String, VisualizerWaveformError> {
        serde_json::to_string_pretty(self)
            .map_err(|error| VisualizerWaveformError::Serialization(error.to_string()))
    }
}

#[derive(Clone, Debug, Error, PartialEq)]
pub enum VisualizerWaveformError {
    #[error(transparent)]
    Input(#[from] crate::VisualizerInputError),
    #[error("waveform lowering gain must be finite and between zero and four, received {gain}")]
    InvalidGain { gain: f32 },
    #[error("waveform lowering requires at least two samples, received {count}")]
    TooFewSamples { count: usize },
    #[error("could not serialize waveform evidence: {0}")]
    Serialization(String),
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        SyntheticAudioFixture, SyntheticVisualizerConfig, SyntheticVisualizerInput,
        VisualizerViewport,
    };

    #[test]
    fn waveform_lowering_is_deterministic_and_normalized() {
        let source = SyntheticVisualizerInput::new(
            SyntheticAudioFixture::FrequencySweep,
            SyntheticVisualizerConfig::default(),
        )
        .unwrap();
        let frame = source
            .frame(24, VisualizerViewport::new(640, 360).unwrap())
            .unwrap();
        let first = VisualizerWaveform::from_frame(&frame, 1.0).unwrap();
        let second = VisualizerWaveform::from_frame(&frame, 1.0).unwrap();

        assert_eq!(first, second);
        assert_eq!(first.points.len(), frame.waveform.len());
        assert_eq!(first.points.first().unwrap()[0], -1.0);
        assert_eq!(first.points.last().unwrap()[0], 1.0);
        assert!(first
            .points
            .iter()
            .all(|point| point[0].is_finite() && (-1.0..=1.0).contains(&point[1])));
    }

    #[test]
    fn waveform_lowering_rejects_invalid_gain() {
        let source = SyntheticVisualizerInput::new(
            SyntheticAudioFixture::Silence,
            SyntheticVisualizerConfig::default(),
        )
        .unwrap();
        let frame = source
            .frame(0, VisualizerViewport::new(640, 360).unwrap())
            .unwrap();
        assert!(matches!(
            VisualizerWaveform::from_frame(&frame, 4.1),
            Err(VisualizerWaveformError::InvalidGain { .. })
        ));
    }
}
