use std::{error::Error, fmt};

use xml_tools::{XmlDiagnostic, XmlSpan};

/// The SVG pipeline boundary that produced an import diagnostic.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SvgImportStage {
    Xml,
    Svg,
}

/// Selects the coordinate bounds used to normalize SVG user-space geometry.
///
/// This importer intentionally resolves only `viewBox` coordinates. Physical
/// viewport sizing (`width`, `height`, and `preserveAspectRatio`) remains an
/// embedding/rendering decision outside this initial profile.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum SvgViewportSource {
    /// Keep the established embedding path: the caller owns normalization
    /// bounds and the document's root `viewBox` is not interpreted.
    Caller([f32; 4]),
    /// Read and validate the root SVG element's `viewBox` attribute.
    DocumentViewBox,
}

/// Root-document aspect policy supported by the coordinate-only SVG profile.
/// Slice modes remain intentionally outside this first viewport slice.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SvgPreserveAspectRatio {
    None,
    XMinYMinMeet,
    XMidYMinMeet,
    XMaxYMinMeet,
    XMinYMidMeet,
    XMidYMidMeet,
    XMaxYMidMeet,
    XMinYMaxMeet,
    XMidYMaxMeet,
    XMaxYMaxMeet,
}

/// Structured SVG import failure that preserves XML diagnostics instead of
/// reducing them to formatted strings at the importer boundary.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SvgImportDiagnostic {
    pub stage: SvgImportStage,
    pub message: String,
    pub span: Option<XmlSpan>,
    pub related_span: Option<XmlSpan>,
    pub can_continue: bool,
    /// XML diagnostics retain their full structured data without making every
    /// SVG import result carry the parser diagnostic inline.
    pub xml: Option<Box<XmlDiagnostic>>,
}

impl SvgImportDiagnostic {
    pub(super) fn svg(span: Option<XmlSpan>, message: impl Into<String>) -> Self {
        Self {
            stage: SvgImportStage::Svg,
            message: message.into(),
            span,
            related_span: None,
            can_continue: false,
            xml: None,
        }
    }
}

impl From<XmlDiagnostic> for SvgImportDiagnostic {
    fn from(diagnostic: XmlDiagnostic) -> Self {
        Self {
            stage: SvgImportStage::Xml,
            message: diagnostic.message.clone(),
            span: diagnostic.span,
            related_span: diagnostic.related_span,
            can_continue: diagnostic.can_continue,
            xml: Some(Box::new(diagnostic)),
        }
    }
}

impl fmt::Display for SvgImportDiagnostic {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.xml {
            Some(diagnostic) => write!(
                formatter,
                "SVG XML syntax error {:?}/{:?} at {:?}: {}",
                diagnostic.category, diagnostic.code, self.span, self.message
            ),
            None => write!(
                formatter,
                "SVG semantic error at {:?}: {}",
                self.span, self.message
            ),
        }
    }
}

impl Error for SvgImportDiagnostic {}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SvgFillRule {
    NonZero,
    EvenOdd,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SvgStrokeLinecap {
    Butt,
    Round,
    Square,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SvgStrokeLinejoin {
    Miter,
    Round,
    Bevel,
}

/// A bounded solid SVG color representation. Color parsing stays in the SVG
/// adapter; renderer-specific color objects do not cross this boundary.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum SvgColor {
    Rgba([f32; 4]),
}

/// SVG element geometry plus the small amount of paint intent needed by an
/// importer. This stays above `VectorPath`: fill/stroke are SVG semantics, not
/// responsibilities of the provider-neutral geometry contract.
#[derive(Clone, Debug, PartialEq)]
pub struct SvgVectorRecord {
    pub path: crate::VectorPath,
    /// A resolved local geometric clip path, when the source element used one.
    /// Structural intersection is a later lowering step; SVG parsing preserves
    /// the provider-neutral relationship here.
    pub clip_path: Option<crate::VectorPath>,
    /// Bounds in the SVG element's original user coordinate space.
    pub source_bounds: Option<([f32; 2], [f32; 2])>,
    /// Bounds after inherited element transforms, before viewBox normalization.
    pub transformed_bounds: Option<([f32; 2], [f32; 2])>,
    pub fill: bool,
    pub stroke: bool,
    pub fill_color: Option<SvgColor>,
    pub stroke_color: Option<SvgColor>,
    pub fill_opacity: f32,
    pub stroke_opacity: f32,
    pub opacity: f32,
    pub fill_rule: SvgFillRule,
    pub stroke_width: f32,
    pub stroke_linecap: SvgStrokeLinecap,
    pub stroke_linejoin: SvgStrokeLinejoin,
    pub stroke_miterlimit: f32,
    pub stroke_dasharray: Option<Vec<f32>>,
    pub stroke_dashoffset: f32,
    pub source_span: XmlSpan,
}
