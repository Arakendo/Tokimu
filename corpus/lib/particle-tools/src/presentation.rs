use serde::{Deserialize, Serialize};

use crate::{ParticleError, ParticleId, ParticlePresentationRole, ParticleState2d, ParticleVec2};

/// Validated provider-neutral visibility bounds.
#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Serialize)]
pub struct ParticleView2d {
    pub minimum: ParticleVec2,
    pub maximum: ParticleVec2,
}

impl ParticleView2d {
    pub fn new(minimum: ParticleVec2, maximum: ParticleVec2) -> Result<Self, ParticleError> {
        minimum.validate("view.minimum")?;
        maximum.validate("view.maximum")?;
        if minimum.x > maximum.x || minimum.y > maximum.y {
            return Err(ParticleError::InvertedBounds {
                field: "particle_view",
            });
        }
        Ok(Self { minimum, maximum })
    }

    fn intersects(self, particle: ParticleState2d) -> bool {
        let radius = particle.size().max(0.0);
        particle.position.x + radius >= self.minimum.x
            && particle.position.x - radius <= self.maximum.x
            && particle.position.y + radius >= self.minimum.y
            && particle.position.y - radius <= self.maximum.y
    }
}

/// Renderer-independent visible state for one particle.
#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Serialize)]
pub struct ParticleInstance2d {
    pub id: ParticleId,
    pub position: ParticleVec2,
    pub size: f32,
    pub rotation: f32,
    pub normalized_age: f32,
    pub presentation_role: ParticlePresentationRole,
}

/// Structural counts produced while lowering one visible batch.
#[derive(Clone, Copy, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
pub struct ParticleInstanceReport {
    pub considered: usize,
    pub visible: usize,
    pub outside_view: usize,
    pub omitted_by_limit: usize,
}

/// Bounded visible particle observations plus their lowering report.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct ParticleInstanceBatch2d {
    pub schema: u16,
    pub instances: Vec<ParticleInstance2d>,
    pub report: ParticleInstanceReport,
}

/// Lower active particles without assigning pixels, materials, or renderer objects.
pub fn lower_particle_instances_2d(
    particles: &[ParticleState2d],
    view: ParticleView2d,
    maximum_instances: usize,
) -> ParticleInstanceBatch2d {
    let mut instances = Vec::with_capacity(particles.len().min(maximum_instances));
    let mut outside_view = 0;
    let mut omitted_by_limit = 0;

    for particle in particles {
        if !view.intersects(*particle) {
            outside_view += 1;
            continue;
        }
        if instances.len() == maximum_instances {
            omitted_by_limit += 1;
            continue;
        }
        instances.push(ParticleInstance2d {
            id: particle.id,
            position: particle.position,
            size: particle.size(),
            rotation: particle.rotation,
            normalized_age: particle.normalized_age(),
            presentation_role: particle.presentation_role,
        });
    }

    ParticleInstanceBatch2d {
        schema: 1,
        report: ParticleInstanceReport {
            considered: particles.len(),
            visible: instances.len(),
            outside_view,
            omitted_by_limit,
        },
        instances,
    }
}
