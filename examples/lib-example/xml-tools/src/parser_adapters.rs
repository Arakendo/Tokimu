use std::str;

use crate::{
    parse_xml_events, validate_xml_input, XmlDiagnostic, XmlDiagnosticCategory, XmlDiagnosticCode,
    XmlDocument, XmlDocumentId, XmlOptions, XmlSourceId, XmlSpan,
};

/// Parses a source buffer through the initial UTF-8-only XML profile.
///
/// This form lets corpus runners retain original bytes and report an encoding
/// boundary before adapting the source to Rust text. It deliberately does not
/// add transcoding: non-UTF-8 input remains an explicit unsupported feature.
pub fn parse_xml_bytes(
    source: XmlSourceId,
    input: &[u8],
    options: XmlOptions,
) -> Result<Vec<crate::XmlEvent>, XmlDiagnostic> {
    validate_xml_input(source, input, options)?;
    let text = str::from_utf8(input).map_err(|error| {
        let start = error.valid_up_to();
        let end = start
            .saturating_add(error.error_len().unwrap_or(1))
            .min(input.len());
        XmlDiagnostic::at(
            XmlDiagnosticCategory::Encoding,
            XmlDiagnosticCode::UnsupportedEncoding,
            source,
            XmlSpan::new(source, start, end),
            "XML source is not valid UTF-8; the initial XML profile supports UTF-8 only",
        )
    })?;
    parse_xml_events(source, text, options)
}

/// Parses UTF-8 XML and retains the resulting immutable document.
pub fn parse_xml_document(
    id: XmlDocumentId,
    source: XmlSourceId,
    input: &str,
    options: XmlOptions,
) -> Result<XmlDocument, XmlDiagnostic> {
    let events = parse_xml_events(source, input, options)?;
    XmlDocument::from_events(id, source, &events)
}

/// Parses UTF-8 XML source bytes and retains the resulting immutable document.
pub fn parse_xml_document_bytes(
    id: XmlDocumentId,
    source: XmlSourceId,
    input: &[u8],
    options: XmlOptions,
) -> Result<XmlDocument, XmlDiagnostic> {
    let events = parse_xml_bytes(source, input, options)?;
    XmlDocument::from_events(id, source, &events)
}
