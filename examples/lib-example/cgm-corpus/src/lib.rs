//! Bounded structural inspection for the selected WebCGM corpus.
//!
//! The crate intentionally stops after CGM descriptor and presentation-state
//! inspection, before primitive lowering or vector geometry. It makes source
//! framing and lifecycle failures observable without admitting CGM concepts
//! into an engine capability.

mod binary;
mod error;
mod lowering;
mod model;

pub use binary::{inspect_binary_cgm, inspect_binary_cgm_file, parameter_bytes, DecodeLimits};
pub use error::{CgmError, CgmResult};
pub use lowering::{
    lower_picture_primitives, lower_primitive, CgmPrimitiveTopology, CgmVectorPrimitive,
};
pub use model::{
    CgmAttribute, CgmAttributeValue, CgmClipIndicator, CgmColor, CgmColorSelectionMode,
    CgmDiagnostic, CgmDiagnosticCode, CgmElement, CgmEncoding, CgmInspection,
    CgmMetafileDescriptor, CgmPartition, CgmPicture, CgmPictureControlState, CgmPictureDescriptor,
    CgmPolygonSetEdgeFlag, CgmPolygonSetRecord, CgmPresentationState, CgmPrimitive,
    CgmPrimitiveKind, CgmScalingMode, CgmVdcExtent, CgmVdcType, DelimiterElement, ElementSupport,
};
