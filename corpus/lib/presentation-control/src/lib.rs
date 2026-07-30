//! Incubating, provider-neutral presentation-control semantics.
//!
//! Applications communicate transient presentation intent through this crate.
//! Importers retain source truth, while renderer adapters remain responsible
//! for turning resolved presentation into materials, bindings, and pixels.

mod color;
mod control;
mod error;
mod override_value;
mod target;

pub use color::{PresentationColor, SourcePresentation};
pub use control::{PresentationControl, PresentationTargetState, ResolvedPresentation};
pub use error::PresentationControlError;
pub use override_value::{
    PresentationEmphasis, PresentationLayer, PresentationOverride, PresentationTint, TintMode,
};
pub use target::{PresentationTargetDescriptor, PresentationTargetId, PresentationTargetKind};

#[cfg(test)]
mod tests;
