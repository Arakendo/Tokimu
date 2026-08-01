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
mod revision;
mod scroll;
mod svg;
mod text;
mod text_input;
mod theme;
mod tree;
mod vector;

/// Intended entry point for ordinary UI consumers.
///
/// This tier exposes semantic regions, layout, controls, text intent, and
/// themes. It deliberately excludes importer, font-parser, rasterizer, and
/// tessellation APIs. Root re-exports remain during the staged migration.
pub mod consumer {
    pub use crate::{
        UiActionId, UiActivationKey, UiButton, UiButtonId, UiButtonSpec, UiCard, UiCardRole,
        UiCardSpec, UiConstraints, UiControlRole, UiCrossAxisAlignment, UiElevation, UiFitStatus,
        UiFocusDirection, UiFocusState, UiFrameLayout, UiHorizontalSplitLayout, UiHorizontalStack,
        UiInsets, UiInteractionState, UiLabel, UiLabelAnchor, UiLabelSpec, UiLayoutFit,
        UiLayoutResult, UiMainAxisAllocation, UiMeasureContext, UiMissingGlyphPolicy,
        UiModalDismissReason, UiModalDismissal, UiNodeConstraints, UiNodeContent, UiNodeId,
        UiNodeInteraction, UiNodeKind, UiNodeLayout, UiNodeSpec, UiNodeStacking, UiOverflowPolicy,
        UiPanel, UiPointerEvent, UiPointerPhase, UiPointerResolution, UiPointerRouter,
        UiPresentationInputs, UiPresentationInvalidation, UiPresentationRebuildEvidence,
        UiPresentationRevisionTracker, UiPresentationWorkEvidence, UiRadius, UiRect, UiRegion,
        UiRegionKind, UiResolvedFocus, UiResolvedLayout, UiResolvedNode, UiResolvedSemantics,
        UiResolvedTree, UiSemanticRole, UiSidebar, UiSizePolicy, UiSpacing, UiStateChip,
        UiStatusBar, UiSurfaceRole, UiTabStrip, UiTextAlign, UiTextAlignmentBasis, UiTextDirection,
        UiTextFit, UiTextInputEvent, UiTextInputOperation, UiTextInputResolution,
        UiTextInputRouter, UiTextInputState, UiTextLayout, UiTextLayoutReport, UiTextLineLayout,
        UiTextMeasure, UiTextOverflow, UiTextRole, UiTextSpec, UiTheme, UiThemeDiagnostic,
        UiThemeProfile, UiToolbar, UiToolbarLayout, UiTree, UiTreeDiagnostic, UiTreeDiagnosticKind,
        UiTreeError, UiUniformGridLayout, UiVerticalScroll, UiVerticalStack, UiWorkspace,
        UiWorkspaceLayout,
    };
}

/// Provider-facing contracts and external-format adapters.
///
/// Consumers should use these only when they intentionally select a concrete
/// font, icon, raster, or SVG implementation.
pub mod provider {
    pub use crate::{
        alpha_to_rgba8, parse_path, parse_svg_document_convex_fill_meshes,
        parse_svg_document_vector_paths, parse_svg_document_vector_records,
        parse_svg_document_vector_records_from_xml_events,
        parse_svg_document_vector_records_with_viewport,
        parse_svg_document_vector_records_with_xml_options, tokenize_path, SvgColor, SvgFillRule,
        SvgImportDiagnostic, SvgImportStage, SvgPathCommand, SvgStrokeLinecap, SvgStrokeLinejoin,
        SvgToken, SvgVectorRecord, SvgViewportSource, UiFontFormat, UiFontHandle, UiFontIdentity,
        UiFontProviderId, UiFontRasterizer, UiFontSource, UiGlyphContour, UiGlyphFillTopology,
        UiGlyphOutline, UiGlyphOutlineDiagnostic, UiGlyphOutlineDiagnosticKind,
        UiGlyphOutlineSegment, UiGlyphVectorDiagnostic, UiGlyphVectorDiagnosticKind,
        UiGlyphVectorOptions, UiIconDiagnostic, UiIconDiagnosticKind, UiIconHandle, UiIconId,
        UiIconMetrics, UiIconProviderId, UiIconResolution, UiIconSpec, UiIconTint,
        UiIconVectorAsset, UiIconVectorProvider, UiRasterGlyph, UiRasterText, UiRasterTextBitmap,
        UiRasterTextBlock, UiRasterTextGlyph, UiRasterTextMetricsProvider, UiTextMetrics,
        UiTextMetricsProvider,
    };
}

/// Renderer-neutral lowering and vector geometry APIs.
pub mod lowering {
    pub use crate::{
        clip_path_to_axis_aligned_rect, clip_path_to_convex_polygon, is_convex_polygon_clip,
        lower_resolved_tree_to_draw_list, lower_surface_to_vector, tessellate_convex_fill,
        tessellate_general_fill, tessellate_general_fill_with_rule, tessellate_path_strokes,
        tessellate_stroke, tessellate_stroke_with_style, validate_convex_fill, PathBuilder,
        UiDrawCacheKey, UiDrawCommand, UiDrawEntry, UiDrawList, UiDrawListBuilder,
        UiDrawListDiagnostic, UiDrawListDiagnosticKind, UiDrawListError, UiDrawListStatistics,
        UiDrawer, UiSurfaceCommand, UiSurfaceVectorLayer, UiSurfaceVectorLayerKind, UiTextCommand,
        VectorContour, VectorFillRule, VectorPath, VectorStrokeCap, VectorStrokeJoin,
        VectorStrokeStyle,
    };
}

/// Structured UI diagnostics exposed independently from provider mechanisms.
pub mod diagnostics {
    pub use crate::{
        UiDiagnostic, UiDiagnosticKind, UiDiagnosticSeverity, UiEvent, UiTextDiagnostic,
        UiTextDiagnosticKind,
    };
}

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
    lower_resolved_tree_to_draw_list, lower_surface_to_vector, UiDrawCacheKey, UiDrawCommand,
    UiDrawEntry, UiDrawList, UiDrawListBuilder, UiDrawListDiagnostic, UiDrawListDiagnosticKind,
    UiDrawListError, UiDrawListStatistics, UiDrawer, UiSurfaceCommand, UiSurfaceVectorLayer,
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
    UiConstraints, UiCrossAxisAlignment, UiFrameLayout, UiHorizontalSplitLayout, UiHorizontalStack,
    UiLayoutFit, UiLayoutResult, UiMainAxisAllocation, UiMeasurable, UiMeasureContext,
    UiOverflowPolicy, UiResolvedLayout, UiSizePolicy, UiUniformGridLayout, UiVerticalStack,
};
pub use presets::{UiToolbarLayout, UiWorkspaceLayout};
pub use raster::{
    alpha_to_rgba8, UiFontRasterizer, UiRasterGlyph, UiRasterText, UiRasterTextBitmap,
    UiRasterTextBlock, UiRasterTextGlyph, UiRasterTextMetricsProvider, UiTextMetrics,
};
pub use region::UiCardRole;
pub use region::{
    UiCard, UiInspector, UiPanel, UiRadius, UiRegion, UiRegionKind, UiSidebar, UiSpacing,
    UiStatusBar, UiSurfaceRole, UiTabStrip, UiToolbar, UiWorkspace,
};
pub use revision::{
    UiPresentationInputs, UiPresentationInvalidation, UiPresentationRebuildEvidence,
    UiPresentationRevisionTracker, UiPresentationWorkEvidence,
};
pub use scroll::{UiScrollVisibility, UiVerticalScroll};
pub use svg::{
    parse_path, parse_svg_document_convex_fill_meshes, parse_svg_document_vector_paths,
    parse_svg_document_vector_records, parse_svg_document_vector_records_from_xml_events,
    parse_svg_document_vector_records_with_viewport,
    parse_svg_document_vector_records_with_xml_options, tokenize_path, SvgColor, SvgFillRule,
    SvgImportDiagnostic, SvgImportStage, SvgPathCommand, SvgStrokeLinecap, SvgStrokeLinejoin,
    SvgToken, SvgVectorRecord, SvgViewportSource,
};
pub use text::{
    bitmap_glyph_height, layout_bitmap_text, measure_bitmap_text_width,
    UiBitmapTextMetricsProvider, UiGlyphQuad, UiMissingGlyphPolicy, UiTextAlign,
    UiTextAlignmentBasis, UiTextDiagnostic, UiTextDiagnosticKind, UiTextDirection, UiTextFit,
    UiTextLayout, UiTextLayoutReport, UiTextLineLayout, UiTextMeasure, UiTextMetricsProvider,
    UiTextOverflow, UiTextRole, UiTextSpec,
};
pub use text_input::{UiTextInputOperation, UiTextInputState};
pub use theme::{
    UiBorderScale, UiControlRole, UiElevation, UiRadiusScale, UiSpacingScale, UiSurfaceStyle,
    UiTextStyle, UiTheme, UiThemeDiagnostic, UiThemeProfile,
};
pub use tree::{
    UiFitStatus, UiModalDismissReason, UiModalDismissal, UiNodeConstraints, UiNodeContent,
    UiNodeId, UiNodeInteraction, UiNodeKind, UiNodeLayout, UiNodeSpec, UiNodeStacking,
    UiPointerEvent, UiPointerPhase, UiPointerResolution, UiPointerRouter, UiResolvedFocus,
    UiResolvedNode, UiResolvedSemantics, UiResolvedTree, UiSemanticRole, UiTextInputEvent,
    UiTextInputResolution, UiTextInputRouter, UiTree, UiTreeDiagnostic, UiTreeDiagnosticKind,
    UiTreeError,
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
