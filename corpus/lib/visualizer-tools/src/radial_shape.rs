//! Deterministic radial-shape lowering for visualizer corpus evidence.
//!
//! This is an original Tokimu presentation primitive driven by normalized
//! visualizer observations. It is not a projectM object or a claim of MilkDrop
//! custom-shape compatibility.

use std::f32::consts::TAU;

use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::VisualizerFrameInput;

pub const MIN_RADIAL_SHAPE_SIDES: usize = 3;
pub const MAX_RADIAL_SHAPE_SIDES: usize = 64;

/// Provider-neutral polygon geometry for one audio-reactive radial shape.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct VisualizerRadialShape {
    pub center: [f32; 2],
    /// Counter-clockwise perimeter vertices in normalized presentation space.
    pub perimeter: Vec<VisualizerRadialShapeVertex>,
    pub fill: [f32; 4],
}

/// One radial vertex and the normalized spectrum observation that influenced it.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct VisualizerRadialShapeVertex {
    pub position: [f32; 2],
    pub source_energy: f32,
}

impl VisualizerRadialShape {
    /// Lowers one valid frame into a bounded, deterministic polygon.
    pub fn from_frame(
        frame: &VisualizerFrameInput,
        sides: usize,
        base_radius: f32,
        audio_gain: f32,
    ) -> Result<Self, VisualizerRadialShapeError> {
        frame.validate()?;
        if !(MIN_RADIAL_SHAPE_SIDES..=MAX_RADIAL_SHAPE_SIDES).contains(&sides) {
            return Err(VisualizerRadialShapeError::InvalidSideCount { sides });
        }
        if !base_radius.is_finite() || !(0.05..=0.9).contains(&base_radius) {
            return Err(VisualizerRadialShapeError::InvalidBaseRadius { base_radius });
        }
        if !audio_gain.is_finite() || !(0.0..=1.0).contains(&audio_gain) {
            return Err(VisualizerRadialShapeError::InvalidAudioGain { audio_gain });
        }

        let phase = frame.time_seconds.fract() * TAU;
        let beat_radius = frame.beat.pulse * audio_gain * 0.15;
        let perimeter = (0..sides)
            .map(|index| {
                let spectrum_index = index * frame.spectrum.len() / sides;
                let source_energy = frame.spectrum[spectrum_index];
                let radius =
                    (base_radius + source_energy * audio_gain * 0.35 + beat_radius).min(1.0);
                let angle = phase + index as f32 / sides as f32 * TAU;
                VisualizerRadialShapeVertex {
                    position: [angle.cos() * radius, angle.sin() * radius],
                    source_energy,
                }
            })
            .collect();

        Ok(Self {
            center: [0.0, 0.0],
            perimeter,
            fill: [0.35, 0.95, 0.82, 0.72],
        })
    }

    pub fn to_structural_json(&self) -> Result<String, VisualizerRadialShapeError> {
        serde_json::to_string_pretty(self)
            .map_err(|error| VisualizerRadialShapeError::Serialization(error.to_string()))
    }
}

#[derive(Clone, Debug, Error, PartialEq)]
pub enum VisualizerRadialShapeError {
    #[error(transparent)]
    Input(#[from] crate::VisualizerInputError),
    #[error(
        "radial shape requires between {MIN_RADIAL_SHAPE_SIDES} and {MAX_RADIAL_SHAPE_SIDES} sides, received {sides}"
    )]
    InvalidSideCount { sides: usize },
    #[error(
        "radial shape base radius must be finite and between 0.05 and 0.9, received {base_radius}"
    )]
    InvalidBaseRadius { base_radius: f32 },
    #[error(
        "radial shape audio gain must be finite and between zero and one, received {audio_gain}"
    )]
    InvalidAudioGain { audio_gain: f32 },
    #[error("could not serialize radial-shape evidence: {0}")]
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
        .frame(90, VisualizerViewport::new(640, 360).unwrap())
        .unwrap()
    }

    #[test]
    fn radial_shape_lowering_is_deterministic_and_bounded() {
        let frame = fixture_frame();
        let first = VisualizerRadialShape::from_frame(&frame, 24, 0.35, 0.4).unwrap();
        let second = VisualizerRadialShape::from_frame(&frame, 24, 0.35, 0.4).unwrap();

        assert_eq!(first, second);
        assert_eq!(first.perimeter.len(), 24);
        assert!(first.perimeter.iter().all(|vertex| {
            vertex.position[0].is_finite()
                && vertex.position[1].is_finite()
                && vertex.position[0].hypot(vertex.position[1]) <= 1.0
                && (0.0..=1.0).contains(&vertex.source_energy)
        }));
    }

    #[test]
    fn radial_shape_lowering_rejects_unbounded_parameters() {
        let frame = fixture_frame();
        assert!(matches!(
            VisualizerRadialShape::from_frame(&frame, 2, 0.35, 0.4),
            Err(VisualizerRadialShapeError::InvalidSideCount { .. })
        ));
        assert!(matches!(
            VisualizerRadialShape::from_frame(&frame, 24, 0.0, 0.4),
            Err(VisualizerRadialShapeError::InvalidBaseRadius { .. })
        ));
        assert!(matches!(
            VisualizerRadialShape::from_frame(&frame, 24, 0.35, 1.1),
            Err(VisualizerRadialShapeError::InvalidAudioGain { .. })
        ));
    }
}
