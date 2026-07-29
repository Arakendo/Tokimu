mod controls;
mod corpus;
mod draw;
mod font;
mod font_outline;
mod geometry;
mod icon;
mod layout;
mod presets;
mod raster;
mod region;
mod scroll;
mod svg;
mod text;
mod text_input;
mod theme;
mod vector;

pub use controls::{
    UiActionId, UiActivationKey, UiButton, UiButtonId, UiButtonSpec, UiCardSpec, UiDiagnostic,
    UiDiagnosticKind, UiDiagnosticSeverity, UiEvent, UiFocusDirection, UiFocusState,
    UiInteractionState, UiLabel, UiLabelAnchor, UiLabelSpec, UiStateChip,
};
pub use corpus::{
    samples as text_corpus_samples, UiTextCorpusGroup, UiTextCorpusSample, TEXT_CORPUS,
    TEXT_CORPUS_VERSION,
};
pub use draw::{
    lower_surface_to_vector, UiDrawer, UiSurfaceCommand, UiSurfaceVectorLayer,
    UiSurfaceVectorLayerKind, UiTextCommand,
};
pub use font::{UiFontFormat, UiFontHandle, UiFontIdentity, UiFontProviderId, UiFontSource};
pub use font_outline::{
    UiGlyphContour, UiGlyphFillTopology, UiGlyphOutline, UiGlyphOutlineDiagnostic,
    UiGlyphOutlineDiagnosticKind, UiGlyphOutlineSegment, UiGlyphVectorDiagnostic,
    UiGlyphVectorDiagnosticKind, UiGlyphVectorOptions,
};
pub use geometry::{window_to_world, UiHitRegion, UiInsets, UiPixelRect, UiRect};
pub use icon::{
    UiIconDiagnostic, UiIconDiagnosticKind, UiIconHandle, UiIconId, UiIconMetrics,
    UiIconProviderId, UiIconResolution, UiIconSpec, UiIconTint, UiIconVectorAsset,
    UiIconVectorProvider,
};
pub use layout::{
    UiConstraints, UiCrossAxisAlignment, UiHorizontalStack, UiLayoutResult, UiMainAxisAllocation,
    UiMeasurable, UiMeasureContext, UiSizePolicy, UiVerticalStack,
};
pub use presets::{UiToolbarLayout, UiWorkspaceLayout};
pub use raster::{
    alpha_to_rgba8, UiFontRasterizer, UiRasterGlyph, UiRasterText, UiRasterTextBitmap,
    UiRasterTextBlock, UiRasterTextGlyph, UiTextMetrics,
};
pub use region::UiCardRole;
pub use region::{
    UiCard, UiInspector, UiPanel, UiRadius, UiRegion, UiRegionKind, UiSidebar, UiSpacing,
    UiStatusBar, UiSurfaceRole, UiTabStrip, UiToolbar, UiWorkspace,
};
pub use scroll::UiVerticalScroll;
pub use svg::{
    parse_path, parse_svg_document_convex_fill_meshes, parse_svg_document_vector_paths,
    parse_svg_document_vector_records, parse_svg_document_vector_records_from_xml_events,
    parse_svg_document_vector_records_with_viewport,
    parse_svg_document_vector_records_with_xml_options, tokenize_path, SvgColor, SvgFillRule,
    SvgImportDiagnostic, SvgImportStage, SvgPathCommand, SvgStrokeLinecap, SvgStrokeLinejoin,
    SvgToken, SvgVectorRecord, SvgViewportSource,
};
pub use text::{
    bitmap_glyph_height, layout_bitmap_text, measure_bitmap_text_width, UiGlyphQuad,
    UiMissingGlyphPolicy, UiTextAlign, UiTextAlignmentBasis, UiTextDiagnostic,
    UiTextDiagnosticKind, UiTextDirection, UiTextLayout, UiTextLayoutReport, UiTextLineLayout,
    UiTextMeasure, UiTextMetricsProvider, UiTextOverflow, UiTextRole, UiTextSpec,
};
pub use text_input::{UiTextInputOperation, UiTextInputState};
pub use theme::{
    UiBorderScale, UiControlRole, UiElevation, UiRadiusScale, UiSpacingScale, UiSurfaceStyle,
    UiTextStyle, UiTheme,
};
pub(crate) use vector::tessellate_font_fill_with_rule;
pub use vector::{
    clip_path_to_axis_aligned_rect, clip_path_to_convex_polygon, is_convex_polygon_clip,
    tessellate_convex_fill, tessellate_general_fill, tessellate_general_fill_with_rule,
    tessellate_path_strokes, tessellate_stroke, tessellate_stroke_with_style, validate_convex_fill,
    PathBuilder, VectorContour, VectorFillRule, VectorPath, VectorStrokeCap, VectorStrokeJoin,
    VectorStrokeStyle,
};

#[cfg(test)]
mod tests;
