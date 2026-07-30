//! Incubating deterministic particle simulation for corpus evidence.
//!
//! Applications own why effects occur and how presentation roles are
//! interpreted. This crate owns only bounded spawning, particle state,
//! integration, expiration, and structural observations.

mod emitter;
mod error;
mod math;
mod model;
mod presentation;
mod range;
mod system;

pub use error::ParticleError;
pub use math::ParticleVec2;
pub use model::{
    ParticleId, ParticlePresentationRole, ParticleSpawn2d, ParticleSpawnReport, ParticleState2d,
    ParticleStepReport, ParticleSystemConfig, ParticleSystemSnapshot,
};
pub use presentation::{
    lower_particle_instances_2d, ParticleInstance2d, ParticleInstanceBatch2d,
    ParticleInstanceReport, ParticleView2d,
};
pub use range::ScalarRange;
pub use system::ParticleSystem2d;

#[cfg(test)]
mod tests;
pub use emitter::ParticleEmitter2d;
