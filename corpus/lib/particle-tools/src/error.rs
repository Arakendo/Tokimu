use thiserror::Error;

/// Deterministic validation failures for particle definitions and updates.
#[derive(Clone, Debug, Error, PartialEq)]
pub enum ParticleError {
    #[error("particle capacity must be greater than zero")]
    ZeroCapacity,
    #[error("particle maximum burst must be greater than zero")]
    ZeroMaximumBurst,
    #[error("particle maximum lifetime must be finite and greater than zero")]
    InvalidMaximumLifetime,
    #[error("particle maximum step must be finite and greater than zero")]
    InvalidMaximumStep,
    #[error("particle field `{field}` must be finite")]
    NonFinite { field: &'static str },
    #[error("particle field `{field}` must be non-negative")]
    Negative { field: &'static str },
    #[error("particle range `{field}` has minimum greater than maximum")]
    InvertedRange { field: &'static str },
    #[error("particle bounds `{field}` have a minimum greater than maximum")]
    InvertedBounds { field: &'static str },
    #[error("particle lifetime range must be greater than zero")]
    NonPositiveLifetime,
    #[error("particle burst requested {requested} particles, above the maximum of {maximum}")]
    BurstTooLarge { requested: usize, maximum: usize },
    #[error("particle step {requested} exceeds the configured maximum of {maximum}")]
    StepTooLarge { requested: f32, maximum: f32 },
}
