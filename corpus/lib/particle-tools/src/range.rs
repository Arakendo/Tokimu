use serde::{Deserialize, Serialize};

use crate::ParticleError;

/// Inclusive scalar range sampled deterministically by an emitter.
#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Serialize)]
pub struct ScalarRange {
    pub minimum: f32,
    pub maximum: f32,
}

impl ScalarRange {
    pub const fn constant(value: f32) -> Self {
        Self {
            minimum: value,
            maximum: value,
        }
    }

    pub fn new(minimum: f32, maximum: f32, field: &'static str) -> Result<Self, ParticleError> {
        let range = Self { minimum, maximum };
        range.validate(field)?;
        Ok(range)
    }

    pub(crate) fn validate(self, field: &'static str) -> Result<(), ParticleError> {
        if !self.minimum.is_finite() || !self.maximum.is_finite() {
            return Err(ParticleError::NonFinite { field });
        }
        if self.minimum > self.maximum {
            return Err(ParticleError::InvertedRange { field });
        }
        Ok(())
    }
}

impl Default for ScalarRange {
    fn default() -> Self {
        Self::constant(0.0)
    }
}
