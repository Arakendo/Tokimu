use serde::{Deserialize, Serialize};

use crate::{ParticleError, ParticleVec2, ScalarRange};

/// Stable identity for one active particle.
#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize)]
pub struct ParticleId(pub u64);

/// Application-defined presentation role carried without prescribing pixels.
#[derive(Clone, Copy, Debug, Default, Deserialize, Eq, Hash, PartialEq, Serialize)]
pub struct ParticlePresentationRole(pub u16);

/// Validated bounds for one particle system.
#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Serialize)]
pub struct ParticleSystemConfig {
    pub capacity: usize,
    pub maximum_burst: usize,
    pub maximum_lifetime: f32,
    pub maximum_step_seconds: f32,
}

impl ParticleSystemConfig {
    pub fn validate(self) -> Result<Self, ParticleError> {
        if self.capacity == 0 {
            return Err(ParticleError::ZeroCapacity);
        }
        if self.maximum_burst == 0 {
            return Err(ParticleError::ZeroMaximumBurst);
        }
        if !self.maximum_lifetime.is_finite() || self.maximum_lifetime <= 0.0 {
            return Err(ParticleError::InvalidMaximumLifetime);
        }
        if !self.maximum_step_seconds.is_finite() || self.maximum_step_seconds <= 0.0 {
            return Err(ParticleError::InvalidMaximumStep);
        }
        Ok(self)
    }
}

impl Default for ParticleSystemConfig {
    fn default() -> Self {
        Self {
            capacity: 512,
            maximum_burst: 128,
            maximum_lifetime: 10.0,
            maximum_step_seconds: 1.0 / 30.0,
        }
    }
}

/// One bounded request to create particles.
#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Serialize)]
pub struct ParticleSpawn2d {
    pub count: usize,
    pub origin: ParticleVec2,
    pub inherited_velocity: ParticleVec2,
    pub direction_radians: ScalarRange,
    pub speed: ScalarRange,
    pub lifetime: ScalarRange,
    pub initial_size: ScalarRange,
    pub final_size: ScalarRange,
    pub initial_rotation: ScalarRange,
    pub angular_velocity: ScalarRange,
    pub acceleration: ParticleVec2,
    pub drag: f32,
    pub presentation_role: ParticlePresentationRole,
}

impl ParticleSpawn2d {
    pub fn burst(count: usize, origin: ParticleVec2) -> Self {
        Self {
            count,
            origin,
            inherited_velocity: ParticleVec2::ZERO,
            direction_radians: ScalarRange::constant(0.0),
            speed: ScalarRange::constant(0.0),
            lifetime: ScalarRange::constant(1.0),
            initial_size: ScalarRange::constant(1.0),
            final_size: ScalarRange::constant(0.0),
            initial_rotation: ScalarRange::constant(0.0),
            angular_velocity: ScalarRange::constant(0.0),
            acceleration: ParticleVec2::ZERO,
            drag: 0.0,
            presentation_role: ParticlePresentationRole::default(),
        }
    }

    pub(crate) fn validate(self, config: ParticleSystemConfig) -> Result<(), ParticleError> {
        if self.count > config.maximum_burst {
            return Err(ParticleError::BurstTooLarge {
                requested: self.count,
                maximum: config.maximum_burst,
            });
        }
        self.origin.validate("origin")?;
        self.inherited_velocity.validate("inherited_velocity")?;
        self.acceleration.validate("acceleration")?;
        self.direction_radians.validate("direction_radians")?;
        self.speed.validate("speed")?;
        self.lifetime.validate("lifetime")?;
        self.initial_size.validate("initial_size")?;
        self.final_size.validate("final_size")?;
        self.initial_rotation.validate("initial_rotation")?;
        self.angular_velocity.validate("angular_velocity")?;

        if self.speed.minimum < 0.0 {
            return Err(ParticleError::Negative { field: "speed" });
        }
        if self.lifetime.minimum <= 0.0 || self.lifetime.maximum > config.maximum_lifetime {
            return Err(ParticleError::NonPositiveLifetime);
        }
        if self.initial_size.minimum < 0.0 {
            return Err(ParticleError::Negative {
                field: "initial_size",
            });
        }
        if self.final_size.minimum < 0.0 {
            return Err(ParticleError::Negative {
                field: "final_size",
            });
        }
        if !self.drag.is_finite() {
            return Err(ParticleError::NonFinite { field: "drag" });
        }
        if self.drag < 0.0 {
            return Err(ParticleError::Negative { field: "drag" });
        }
        Ok(())
    }
}

/// Provider-neutral state retained for one active particle.
#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Serialize)]
pub struct ParticleState2d {
    pub id: ParticleId,
    pub position: ParticleVec2,
    pub velocity: ParticleVec2,
    pub acceleration: ParticleVec2,
    pub age: f32,
    pub lifetime: f32,
    pub initial_size: f32,
    pub final_size: f32,
    pub rotation: f32,
    pub angular_velocity: f32,
    pub drag: f32,
    pub presentation_role: ParticlePresentationRole,
}

impl ParticleState2d {
    pub fn normalized_age(self) -> f32 {
        (self.age / self.lifetime).clamp(0.0, 1.0)
    }

    pub fn size(self) -> f32 {
        self.initial_size + (self.final_size - self.initial_size) * self.normalized_age()
    }
}

/// Result of one bounded spawn request.
#[derive(Clone, Copy, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
pub struct ParticleSpawnReport {
    pub requested: usize,
    pub spawned: usize,
    pub dropped: usize,
    pub active: usize,
}

/// Result of one deterministic simulation step.
#[derive(Clone, Copy, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
pub struct ParticleStepReport {
    pub active_before: usize,
    pub active_after: usize,
    pub expired: usize,
}

/// Read-only structural evidence for one particle-system observation.
#[derive(Clone, Copy, Debug, Serialize)]
pub struct ParticleSystemSnapshot<'a> {
    pub schema: u16,
    pub seed: u32,
    pub capacity: usize,
    pub next_id: u64,
    pub dropped_total: u64,
    pub particles: &'a [ParticleState2d],
}
