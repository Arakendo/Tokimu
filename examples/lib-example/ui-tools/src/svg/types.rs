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

/// Structured SVG import failure that preserves XML diagnostics instead of
/// reducing them to formatted strings at the importer boundary.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SvgImportDiagnostic {
    pub stage: SvgImportStage,
    pub message: String,
    pub span: Option<XmlSpan>,
    pub related_span: Option<XmlSpan>,
    pub can_continue: bool,
    pub xml: Option<XmlDiagnostic>,
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
            xml: Some(diagnostic),
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

/// SVG element geometry plus the small amount of paint intent needed by an
/// importer. This stays above `VectorPath`: fill/stroke are SVG semantics, not
/// responsibilities of the provider-neutral geometry contract.
#[derive(Clone, Debug, PartialEq)]
pub struct SvgVectorRecord {
    pub path: crate::VectorPath,
    pub fill: bool,
    pub stroke: bool,
    pub fill_rule: SvgFillRule,
    pub source_span: XmlSpan,
}
