use serde::{Deserialize, Serialize};

use crate::{ParticleError, ParticleSpawn2d, ParticleSpawnReport, ParticleSystem2d};

/// Fixed-rate emitter that converts explicit elapsed time into bounded spawns.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct ParticleEmitter2d {
    template: ParticleSpawn2d,
    particles_per_second: f32,
    fractional_particles: f64,
    enabled: bool,
}

impl ParticleEmitter2d {
    pub fn new(
        mut template: ParticleSpawn2d,
        particles_per_second: f32,
    ) -> Result<Self, ParticleError> {
        if !particles_per_second.is_finite() {
            return Err(ParticleError::NonFinite {
                field: "particles_per_second",
            });
        }
        if particles_per_second < 0.0 {
            return Err(ParticleError::Negative {
                field: "particles_per_second",
            });
        }
        template.count = 0;
        Ok(Self {
            template,
            particles_per_second,
            fractional_particles: 0.0,
            enabled: true,
        })
    }

    pub fn template(&self) -> ParticleSpawn2d {
        self.template
    }

    pub fn set_template(&mut self, mut template: ParticleSpawn2d) {
        template.count = 0;
        self.template = template;
    }

    pub fn particles_per_second(&self) -> f32 {
        self.particles_per_second
    }

    pub fn set_particles_per_second(&mut self, value: f32) -> Result<(), ParticleError> {
        if !value.is_finite() {
            return Err(ParticleError::NonFinite {
                field: "particles_per_second",
            });
        }
        if value < 0.0 {
            return Err(ParticleError::Negative {
                field: "particles_per_second",
            });
        }
        self.particles_per_second = value;
        Ok(())
    }

    pub fn is_enabled(&self) -> bool {
        self.enabled
    }

    pub fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
    }

    pub fn emit(
        &mut self,
        system: &mut ParticleSystem2d,
        delta_seconds: f32,
    ) -> Result<ParticleSpawnReport, ParticleError> {
        validate_delta(delta_seconds, system)?;
        if !self.enabled || delta_seconds == 0.0 || self.particles_per_second == 0.0 {
            return Ok(ParticleSpawnReport {
                active: system.active_count(),
                ..ParticleSpawnReport::default()
            });
        }

        let accumulated = self.fractional_particles
            + f64::from(self.particles_per_second) * f64::from(delta_seconds);
        let count = accumulated.floor() as usize;
        if count == 0 {
            self.fractional_particles = accumulated;
            return Ok(ParticleSpawnReport {
                active: system.active_count(),
                ..ParticleSpawnReport::default()
            });
        }

        let mut request = self.template;
        request.count = count;
        let report = system.spawn(request)?;
        self.fractional_particles = accumulated - count as f64;
        Ok(report)
    }

    pub fn reset(&mut self) {
        self.fractional_particles = 0.0;
    }
}

fn validate_delta(delta_seconds: f32, system: &ParticleSystem2d) -> Result<(), ParticleError> {
    if !delta_seconds.is_finite() {
        return Err(ParticleError::NonFinite {
            field: "delta_seconds",
        });
    }
    if delta_seconds < 0.0 {
        return Err(ParticleError::Negative {
            field: "delta_seconds",
        });
    }
    if delta_seconds > system.config().maximum_step_seconds {
        return Err(ParticleError::StepTooLarge {
            requested: delta_seconds,
            maximum: system.config().maximum_step_seconds,
        });
    }
    Ok(())
}
