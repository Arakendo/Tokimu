use serde::{Deserialize, Serialize};

use crate::ParticleError;

/// Provider-neutral two-dimensional particle coordinate.
#[derive(Clone, Copy, Debug, Default, Deserialize, PartialEq, Serialize)]
pub struct ParticleVec2 {
    pub x: f32,
    pub y: f32,
}

impl ParticleVec2 {
    pub const ZERO: Self = Self::new(0.0, 0.0);

    pub const fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }

    pub fn validate(self, field: &'static str) -> Result<Self, ParticleError> {
        if !self.x.is_finite() || !self.y.is_finite() {
            return Err(ParticleError::NonFinite { field });
        }
        Ok(self)
    }

    pub fn add_vec(self, other: Self) -> Self {
        Self::new(self.x + other.x, self.y + other.y)
    }

    pub fn scale(self, scalar: f32) -> Self {
        Self::new(self.x * scalar, self.y * scalar)
    }
}
