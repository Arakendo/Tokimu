use std::fmt;

use crate::{XmlSourceId, XmlSpan};

/// The processing boundary that produced an XML diagnostic.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum XmlDiagnosticCategory {
    Syntax,
    WellFormedness,
    Namespace,
    UnsupportedFeature,
    ResourceLimit,
    Encoding,
    InternalAdapter,
}

/// Stable diagnostic identities for the diagnostic-core slice.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum XmlDiagnosticCode {
    InvalidOptions,
    InputTooLarge,
    ParserSyntax,
    UnboundPrefix,
    UnsupportedDocumentType,
    UnsupportedEncoding,
    UnsupportedEntityReference,
    NestingDepthExceeded,
    NodeLimitExceeded,
    AttributeLimitExceeded,
    NameLimitExceeded,
    AttributeValueLimitExceeded,
    DecodedTextLimitExceeded,
    DocumentStructure,
}

/// Severity carried independently from category and code for future recovery.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum XmlDiagnosticSeverity {
    Error,
    Warning,
}

/// Parser-neutral XML failure information for importer-facing diagnostics.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct XmlDiagnostic {
    pub category: XmlDiagnosticCategory,
    pub code: XmlDiagnosticCode,
    pub severity: XmlDiagnosticSeverity,
    pub source: Option<XmlSourceId>,
    pub span: Option<XmlSpan>,
    pub related_span: Option<XmlSpan>,
    pub message: String,
    pub can_continue: bool,
}

impl XmlDiagnostic {
    pub fn invalid_options(message: impl Into<String>) -> Self {
        Self {
            category: XmlDiagnosticCategory::InternalAdapter,
            code: XmlDiagnosticCode::InvalidOptions,
            severity: XmlDiagnosticSeverity::Error,
            source: None,
            span: None,
            related_span: None,
            message: message.into(),
            can_continue: false,
        }
    }

    pub fn input_too_large(source: XmlSourceId, input_bytes: usize, limit: usize) -> Self {
        Self {
            category: XmlDiagnosticCategory::ResourceLimit,
            code: XmlDiagnosticCode::InputTooLarge,
            severity: XmlDiagnosticSeverity::Error,
            source: Some(source),
            span: None,
            related_span: None,
            message: format!(
                "XML input contains {input_bytes} bytes, exceeding the configured {limit}-byte limit"
            ),
            can_continue: false,
        }
    }

    pub(crate) fn at(
        category: XmlDiagnosticCategory,
        code: XmlDiagnosticCode,
        source: XmlSourceId,
        span: XmlSpan,
        message: impl Into<String>,
    ) -> Self {
        Self {
            category,
            code,
            severity: XmlDiagnosticSeverity::Error,
            source: Some(source),
            span: Some(span),
            related_span: None,
            message: message.into(),
            can_continue: false,
        }
    }
}

impl fmt::Display for XmlDiagnostic {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{:?}: {}", self.code, self.message)
    }
}

impl std::error::Error for XmlDiagnostic {}
