//! Serializable diagnostic-artifact schemas.
//!
//! Producers populate these provider-neutral records; artifact writing and
//! reviewed-golden policy remain separate concerns.

use serde::Serialize;

use cgm_corpus::{
    CgmClipIndicator, CgmPictureControlState, CgmPresentationState, CgmPrimitiveKind, CgmVdcExtent,
};

#[derive(Clone, Debug, Serialize)]
pub struct ArtifactEnvelope {
    pub schema: u32,
    pub artifact: String,
    pub producer: String,
    pub case_id: String,
    pub input_hash: String,
    pub source: String,
    pub algorithms: ArtifactAlgorithms,
}

#[derive(Clone, Debug, Serialize)]
pub struct ArtifactAlgorithms {
    pub flatten: String,
    pub tessellator: String,
    pub fill_rule: String,
}

#[derive(Clone, Debug, Serialize)]
pub struct OutlineArtifact {
    pub metadata: ArtifactEnvelope,
    pub character: char,
    pub units_per_em: f32,
    pub contours: Vec<OutlineContourArtifact>,
}

/// Parser-neutral evidence recorded before SVG semantics lower XML elements
/// into vector paths. The corpus deliberately stores counts and document
/// structure, not quick-xml implementation types.
#[derive(Clone, Debug, Serialize)]
pub struct XmlArtifact {
    pub metadata: ArtifactEnvelope,
    pub event_count: usize,
    pub start_elements: usize,
    pub end_elements: usize,
    pub text_nodes: usize,
    pub comments: usize,
    pub processing_instructions: usize,
    pub document_roots: usize,
    pub has_document_element: bool,
}

/// Parser and source-state evidence for a pinned CGM fixture.
///
/// This remains separate from `VectorArtifact`: CGM element lifecycle and
/// presentation state are source-format evidence, not vector contracts.
#[derive(Clone, Debug, Serialize)]
pub struct CgmArtifact {
    pub metadata: ArtifactEnvelope,
    pub metafile_name: String,
    pub picture_name: String,
    pub source_bytes: usize,
    pub element_count: usize,
    pub primitive_count: usize,
    pub attribute_count: usize,
    /// Source-format snapshots for every primitive in the picture. These make
    /// CGM state inheritance and control boundaries inspectable without
    /// leaking source concepts into `VectorArtifact`.
    pub primitives: Vec<CgmPrimitiveSourceArtifact>,
    /// Source-format controls are recorded as evidence only. They do not imply
    /// that the provider-neutral vector artifact has applied a clip.
    pub clip_rectangle: Option<CgmVdcExtent>,
    pub clip_indicator: Option<CgmClipIndicator>,
    pub diagnostic_count: usize,
}

/// Source-only evidence active at one CGM primitive boundary.
#[derive(Clone, Debug, Serialize)]
pub struct CgmPrimitiveSourceArtifact {
    pub source_element: usize,
    pub source_offset: usize,
    pub attribute_count: usize,
    pub kind: CgmPrimitiveKind,
    pub state: CgmPresentationState,
    pub controls: CgmPictureControlState,
}

/// Structural evidence for an SVG fixture that is valid XML but intentionally
/// outside the currently admitted SVG semantic profile.
#[derive(Clone, Debug, Serialize)]
pub struct SvgProfileExclusionArtifact {
    pub metadata: ArtifactEnvelope,
    pub expectation: String,
    pub diagnostic: String,
}

#[derive(Clone, Debug, Serialize)]
pub struct OutlineContourArtifact {
    pub start: [f32; 2],
    pub closed: bool,
    pub segments: Vec<OutlineSegmentArtifact>,
}

#[derive(Clone, Debug, Serialize)]
#[serde(tag = "kind")]
pub enum OutlineSegmentArtifact {
    Line {
        end: [f32; 2],
    },
    Quadratic {
        control: [f32; 2],
        end: [f32; 2],
    },
    Cubic {
        control1: [f32; 2],
        control2: [f32; 2],
        end: [f32; 2],
    },
}

#[derive(Clone, Debug, Serialize)]
pub struct VectorArtifact {
    pub metadata: ArtifactEnvelope,
    /// Optional producer-space bounds. SVG records populate these before
    /// viewBox normalization; other producers may leave them absent.
    pub source_bounds: Option<([f32; 2], [f32; 2])>,
    pub transformed_bounds: Option<([f32; 2], [f32; 2])>,
    pub bounds: Option<([f32; 2], [f32; 2])>,
    pub contours: Vec<VectorContourArtifact>,
    /// Producer-level paint intent is recorded beside geometry rather than
    /// being folded into a renderer or tessellation artifact.
    pub paint_records: Vec<PaintArtifact>,
    pub intersections: Vec<SegmentIntersectionArtifact>,
    pub clips: Vec<ClipPathArtifact>,
}

/// Order-preserving structural identity for provider-neutral vector paths.
///
/// Unlike mesh fingerprints, vector contour order is significant because it
/// records the producer's source order and contour topology before any
/// tessellator is involved.
#[derive(Clone, Debug, Serialize)]
pub struct VectorFingerprint {
    pub metadata: ArtifactEnvelope,
    pub path_count: usize,
    pub contour_count: usize,
    pub point_count: usize,
    pub canonical_path_hash: String,
}

/// Bounded paint semantics preserved by an importer for corpus review.
///
/// These values describe source intent only. They do not claim that the mesh
/// artifact has performed SVG compositing or renderer blending.
#[derive(Clone, Debug, Serialize)]
pub struct PaintArtifact {
    pub record_index: usize,
    pub fill: bool,
    pub stroke: bool,
    pub fill_color: Option<[f32; 4]>,
    pub stroke_color: Option<[f32; 4]>,
    pub fill_opacity: f32,
    pub stroke_opacity: f32,
    pub opacity: f32,
    pub stroke_width: f32,
}

#[derive(Clone, Debug, Serialize)]
pub struct ClipPathArtifact {
    pub target_record: usize,
    pub bounds: Option<([f32; 2], [f32; 2])>,
    pub contour_count: usize,
    pub point_count: usize,
}

#[derive(Clone, Debug, Serialize)]
pub struct VectorContourArtifact {
    pub index: usize,
    pub closed: bool,
    pub points: Vec<[f32; 2]>,
    pub signed_area: f32,
}

#[derive(Clone, Debug, Serialize)]
pub struct SegmentIntersectionArtifact {
    pub first_contour: usize,
    pub first_segment: usize,
    pub second_contour: usize,
    pub second_segment: usize,
    pub point: [f32; 2],
}

#[derive(Clone, Debug, Serialize)]
pub struct MeshArtifact {
    pub metadata: ArtifactEnvelope,
    pub bounds: Option<([f32; 2], [f32; 2])>,
    pub triangles: Vec<[f32; 2]>,
    pub validation: MeshValidation,
}

#[derive(Clone, Debug, Serialize)]
pub struct MeshValidation {
    pub finite: bool,
    pub complete_triangles: bool,
    pub triangle_count: usize,
    pub degenerate_triangles: usize,
    pub total_area: f32,
}

#[derive(Clone, Debug, Serialize)]
pub struct MeshFingerprint {
    pub metadata: ArtifactEnvelope,
    pub bounds: Option<([f32; 2], [f32; 2])>,
    pub triangle_count: usize,
    pub degenerate_triangles: usize,
    pub total_area: f32,
    pub canonical_triangle_hash: String,
}

#[derive(Clone, Debug, Serialize)]
pub struct ImageFingerprint {
    pub metadata: ArtifactEnvelope,
    pub width: u32,
    pub height: u32,
    pub format: String,
    pub source_buffer: String,
    pub pixel_hash: String,
}

#[derive(Clone, Debug, Serialize)]
pub struct GraphArtifact {
    pub metadata: ArtifactEnvelope,
    pub nodes: Vec<GraphNode>,
    pub edges: Vec<GraphEdge>,
}

#[derive(Clone, Debug, Serialize)]
pub struct GraphNode {
    pub id: String,
    pub stage: String,
    pub status: String,
    pub artifact: String,
}

#[derive(Clone, Debug, Serialize)]
pub struct GraphEdge {
    pub from: String,
    pub to: String,
}
