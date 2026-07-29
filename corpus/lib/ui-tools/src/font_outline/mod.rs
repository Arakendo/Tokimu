mod lowering;
mod provider;
mod types;

pub use types::{
    UiGlyphContour, UiGlyphFillTopology, UiGlyphOutline, UiGlyphOutlineDiagnostic,
    UiGlyphOutlineDiagnosticKind, UiGlyphOutlineSegment, UiGlyphVectorDiagnostic,
    UiGlyphVectorDiagnosticKind, UiGlyphVectorOptions,
};

const POINT_EPSILON: f32 = 1.0e-4;

#[cfg(test)]
mod tests;
