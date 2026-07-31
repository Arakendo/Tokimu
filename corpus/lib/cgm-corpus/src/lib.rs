//! Bounded structural inspection for the selected WebCGM corpus.
//!
//! The crate intentionally stops after CGM descriptor and presentation-state
//! inspection and provider-neutral primitive lowering. It makes source
//! framing, lifecycle, and source-to-vector failures observable without
//! admitting CGM concepts into an engine capability.

mod binary;
mod error;
mod lowering;
mod model;

pub use binary::{inspect_binary_cgm, inspect_binary_cgm_file, parameter_bytes, DecodeLimits};
pub use error::{CgmError, CgmResult};
pub use lowering::{
    lower_picture_primitives, lower_primitive, CgmEdgeIntent, CgmFillIntent,
    CgmPrimitivePresentation, CgmPrimitiveTopology, CgmStrokeIntent, CgmVectorPrimitive,
};
pub use model::{
    cgm_element_name, summarize_diagnostics, CgmAttribute, CgmAttributeValue, CgmCellArrayRecord,
    CgmClipIndicator, CgmColor, CgmColorSelectionMode, CgmColorValueExtent, CgmDeferredFeature,
    CgmDiagnostic, CgmDiagnosticCode, CgmElement, CgmEncoding, CgmInspection, CgmInteriorStyle,
    CgmMetafileDescriptor, CgmPartition, CgmPicture, CgmPictureControlState, CgmPictureDescriptor,
    CgmPolygonSetEdgeFlag, CgmPolygonSetRecord, CgmPresentationState, CgmPrimitive,
    CgmPrimitiveKind, CgmScalingMode, CgmTextAlignment, CgmTextOrientation, CgmTextRecord,
    CgmTextRecordKind, CgmVdcExtent, CgmVdcType, DelimiterElement, ElementSupport,
};
