//! Parser-neutral, bounded XML ingestion contracts for incubating importers.
//!
//! This crate deliberately exposes no parser implementation types. Parser
//! adapters will translate their private errors and events into these stable
//! source, limit, and diagnostic contracts.

use std::{fmt, str};

use quick_xml::{events::Event, name::ResolveResult, reader::NsReader};

/// Opaque identity for a source supplied to an XML parser adapter.
#[derive(Clone, Copy, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct XmlSourceId(u32);

impl XmlSourceId {
    pub const fn new(value: u32) -> Self {
        Self(value)
    }

    pub const fn value(self) -> u32 {
        self.0
    }
}

/// A half-open byte span into the original UTF-8 source.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct XmlSpan {
    pub source: XmlSourceId,
    pub start: usize,
    pub end: usize,
}

impl XmlSpan {
    pub const fn new(source: XmlSourceId, start: usize, end: usize) -> Self {
        Self { source, start, end }
    }

    pub const fn is_valid(self) -> bool {
        self.start <= self.end
    }
}

/// Resource bounds applied before and during XML ingestion.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct XmlLimits {
    pub max_input_bytes: usize,
    pub max_nesting_depth: usize,
    pub max_nodes: usize,
    pub max_attributes_per_element: usize,
    pub max_name_bytes: usize,
    pub max_attribute_value_bytes: usize,
    pub max_decoded_text_bytes: usize,
    pub max_diagnostics: usize,
}

impl XmlLimits {
    /// Conservative defaults for untrusted UTF-8 XML supplied to an importer.
    pub const fn safe_defaults() -> Self {
        Self {
            max_input_bytes: 16 * 1024 * 1024,
            max_nesting_depth: 128,
            max_nodes: 100_000,
            max_attributes_per_element: 256,
            max_name_bytes: 1024,
            max_attribute_value_bytes: 64 * 1024,
            max_decoded_text_bytes: 4 * 1024 * 1024,
            max_diagnostics: 128,
        }
    }

    /// Rejects options that would make the declared profile impossible to
    /// enforce or silently disable a required bound.
    pub fn validate(self) -> Result<(), XmlDiagnostic> {
        for (name, value) in [
            ("max_input_bytes", self.max_input_bytes),
            ("max_nesting_depth", self.max_nesting_depth),
            ("max_nodes", self.max_nodes),
            (
                "max_attributes_per_element",
                self.max_attributes_per_element,
            ),
            ("max_name_bytes", self.max_name_bytes),
            ("max_attribute_value_bytes", self.max_attribute_value_bytes),
            ("max_decoded_text_bytes", self.max_decoded_text_bytes),
            ("max_diagnostics", self.max_diagnostics),
        ] {
            if value == 0 {
                return Err(XmlDiagnostic::invalid_options(format!(
                    "XML limit '{name}' must be greater than zero"
                )));
            }
        }
        Ok(())
    }
}

impl Default for XmlLimits {
    fn default() -> Self {
        Self::safe_defaults()
    }
}

/// Options owned by the parser-neutral XML boundary.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct XmlOptions {
    pub limits: XmlLimits,
}

impl XmlOptions {
    pub fn validate(self) -> Result<(), XmlDiagnostic> {
        self.limits.validate()
    }
}

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

    fn at(
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

/// Validates options and source-buffer size before parser-specific ingestion.
pub fn validate_xml_input(
    source: XmlSourceId,
    input: &[u8],
    options: XmlOptions,
) -> Result<(), XmlDiagnostic> {
    options.validate()?;
    if input.len() > options.limits.max_input_bytes {
        return Err(XmlDiagnostic::input_too_large(
            source,
            input.len(),
            options.limits.max_input_bytes,
        ));
    }
    Ok(())
}

/// A namespace-resolved XML name. The lexical prefix remains presentation data;
/// XML consumers should make semantic decisions from this expanded identity.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct ExpandedName {
    pub namespace_uri: Option<String>,
    pub local_name: String,
}

/// An owned XML attribute in source order.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct XmlAttribute {
    pub name: ExpandedName,
    pub lexical_prefix: Option<String>,
    pub value: String,
    pub span: XmlSpan,
}

/// Parser-neutral XML events for the first bounded importer profile.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum XmlEvent {
    StartElement {
        name: ExpandedName,
        lexical_prefix: Option<String>,
        attributes: Vec<XmlAttribute>,
        span: XmlSpan,
    },
    EndElement {
        name: ExpandedName,
        lexical_prefix: Option<String>,
        span: XmlSpan,
    },
    Text {
        text: String,
        span: XmlSpan,
    },
    Comment {
        text: String,
        span: XmlSpan,
    },
    ProcessingInstruction {
        target: String,
        data: Option<String>,
        span: XmlSpan,
    },
}

const XMLNS_NAMESPACE: &str = "http://www.w3.org/2000/xmlns/";

/// Parses UTF-8 XML into a bounded, parser-neutral event sequence.
///
/// This is intentionally not a DOM. It establishes the source ordering,
/// namespace, limit, and diagnostic behavior that later document adapters can
/// build upon without exposing the selected parser implementation.
pub fn parse_xml_events(
    source: XmlSourceId,
    input: &str,
    options: XmlOptions,
) -> Result<Vec<XmlEvent>, XmlDiagnostic> {
    validate_xml_input(source, input.as_bytes(), options)?;

    let mut reader = NsReader::from_str(input);
    let mut events = Vec::new();
    let mut depth = 0usize;
    let mut decoded_text_bytes = 0usize;
    let mut pending_text: Option<(String, XmlSpan)> = None;

    loop {
        let start = reader.buffer_position() as usize;
        let event = match reader.read_event() {
            Ok(event) => event,
            Err(error) => {
                return Err(parser_error(
                    source,
                    input.len(),
                    start,
                    reader.error_position() as usize,
                    error,
                ));
            }
        };
        let end = reader.buffer_position() as usize;
        let span = XmlSpan::new(source, start, end);

        match event {
            Event::Start(element) => {
                flush_pending_text(&mut events, &mut pending_text, options.limits.max_nodes)?;
                depth = depth.checked_add(1).expect("XML depth cannot overflow usize");
                if depth > options.limits.max_nesting_depth {
                    return Err(limit_error(
                        source,
                        span,
                        XmlDiagnosticCode::NestingDepthExceeded,
                        format!(
                            "XML nesting depth {depth} exceeds the configured {}-level limit",
                            options.limits.max_nesting_depth
                        ),
                    ));
                }
                let resolution = reader.resolver().resolve_element(element.name()).0;
                let (name, lexical_prefix) = expanded_name(source, span, resolution, element.name().as_ref())?;
                let attributes = parse_attributes(source, span, &reader, &element, options.limits)?;
                push_event(
                    &mut events,
                    XmlEvent::StartElement {
                        name,
                        lexical_prefix,
                        attributes,
                        span,
                    },
                    options.limits.max_nodes,
                )?;
            }
            Event::Empty(element) => {
                flush_pending_text(&mut events, &mut pending_text, options.limits.max_nodes)?;
                let resolution = reader.resolver().resolve_element(element.name()).0;
                let (name, lexical_prefix) = expanded_name(source, span, resolution, element.name().as_ref())?;
                let attributes = parse_attributes(source, span, &reader, &element, options.limits)?;
                push_event(
                    &mut events,
                    XmlEvent::StartElement {
                        name: name.clone(),
                        lexical_prefix: lexical_prefix.clone(),
                        attributes,
                        span,
                    },
                    options.limits.max_nodes,
                )?;
                push_event(
                    &mut events,
                    XmlEvent::EndElement {
                        name,
                        lexical_prefix,
                        span,
                    },
                    options.limits.max_nodes,
                )?;
            }
            Event::End(element) => {
                flush_pending_text(&mut events, &mut pending_text, options.limits.max_nodes)?;
                let resolution = reader.resolver().resolve_element(element.name()).0;
                let (name, lexical_prefix) = expanded_name(source, span, resolution, element.name().as_ref())?;
                push_event(
                    &mut events,
                    XmlEvent::EndElement {
                        name,
                        lexical_prefix,
                        span,
                    },
                    options.limits.max_nodes,
                )?;
                depth = depth.saturating_sub(1);
            }
            Event::Text(text) => append_text(
                &mut pending_text,
                text.xml_content().map_err(|error| encoding_error(source, span, error))?.into_owned(),
                span,
                &mut decoded_text_bytes,
                options.limits.max_decoded_text_bytes,
            )?,
            Event::CData(text) => append_text(
                &mut pending_text,
                text.xml_content().map_err(|error| encoding_error(source, span, error))?.into_owned(),
                span,
                &mut decoded_text_bytes,
                options.limits.max_decoded_text_bytes,
            )?,
            Event::GeneralRef(reference) => append_text(
                &mut pending_text,
                decode_reference(reference.as_ref()).ok_or_else(|| {
                    XmlDiagnostic::at(
                        XmlDiagnosticCategory::UnsupportedFeature,
                        XmlDiagnosticCode::UnsupportedEntityReference,
                        source,
                        span,
                        "XML entity references other than predefined and numeric references are unsupported",
                    )
                })?,
                span,
                &mut decoded_text_bytes,
                options.limits.max_decoded_text_bytes,
            )?,
            Event::Comment(comment) => {
                flush_pending_text(&mut events, &mut pending_text, options.limits.max_nodes)?;
                push_event(
                    &mut events,
                    XmlEvent::Comment {
                        text: comment.xml_content().map_err(|error| encoding_error(source, span, error))?.into_owned(),
                        span,
                    },
                    options.limits.max_nodes,
                )?;
            }
            Event::PI(instruction) => {
                flush_pending_text(&mut events, &mut pending_text, options.limits.max_nodes)?;
                let target = decode_utf8(source, span, instruction.target())?;
                let data = decode_utf8(source, span, instruction.content())?;
                push_event(
                    &mut events,
                    XmlEvent::ProcessingInstruction {
                        target,
                        data: (!data.trim().is_empty()).then_some(data.trim().to_owned()),
                        span,
                    },
                    options.limits.max_nodes,
                )?;
            }
            Event::Decl(declaration) => {
                if let Some(encoding) = declaration.encoding() {
                    let encoding = encoding.map_err(|error| parser_error(source, input.len(), start, end, error))?;
                    let encoding = decode_utf8(source, span, encoding.as_ref())?;
                    if !encoding.eq_ignore_ascii_case("utf-8") && !encoding.eq_ignore_ascii_case("utf8") {
                        return Err(XmlDiagnostic::at(
                            XmlDiagnosticCategory::Encoding,
                            XmlDiagnosticCode::UnsupportedEncoding,
                            source,
                            span,
                            format!("XML declaration requests unsupported encoding '{encoding}'; only UTF-8 input is supported"),
                        ));
                    }
                }
            }
            Event::DocType(_) => {
                return Err(XmlDiagnostic::at(
                    XmlDiagnosticCategory::UnsupportedFeature,
                    XmlDiagnosticCode::UnsupportedDocumentType,
                    source,
                    span,
                    "DOCTYPE declarations and DTD processing are disabled for this XML profile",
                ));
            }
            Event::Eof => {
                flush_pending_text(&mut events, &mut pending_text, options.limits.max_nodes)?;
                return Ok(events);
            }
        }
    }
}

/// Parses a source buffer through the initial UTF-8-only XML profile.
///
/// This form lets corpus runners retain the original bytes and report an
/// encoding boundary before adapting the source to Rust text. It deliberately
/// does not add transcoding: non-UTF-8 input remains an explicit unsupported
/// feature of the initial profile.
pub fn parse_xml_bytes(
    source: XmlSourceId,
    input: &[u8],
    options: XmlOptions,
) -> Result<Vec<XmlEvent>, XmlDiagnostic> {
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

fn parse_attributes(
    source: XmlSourceId,
    span: XmlSpan,
    reader: &NsReader<&[u8]>,
    element: &quick_xml::events::BytesStart<'_>,
    limits: XmlLimits,
) -> Result<Vec<XmlAttribute>, XmlDiagnostic> {
    let mut attributes = Vec::new();
    for attribute in element.attributes().with_checks(true) {
        let attribute = attribute
            .map_err(|error| parser_error(source, span.end, span.start, span.end, error))?;
        if attributes.len() >= limits.max_attributes_per_element {
            return Err(limit_error(
                source,
                span,
                XmlDiagnosticCode::AttributeLimitExceeded,
                format!(
                    "XML element exceeds the configured {}-attribute limit",
                    limits.max_attributes_per_element
                ),
            ));
        }
        let raw_name = attribute.key.as_ref();
        let (name, lexical_prefix) = if raw_name == b"xmlns" {
            (
                ExpandedName {
                    namespace_uri: Some(XMLNS_NAMESPACE.to_owned()),
                    local_name: "xmlns".to_owned(),
                },
                None,
            )
        } else if let Some(local) = raw_name.strip_prefix(b"xmlns:") {
            (
                ExpandedName {
                    namespace_uri: Some(XMLNS_NAMESPACE.to_owned()),
                    local_name: decode_utf8(source, span, local)?,
                },
                Some("xmlns".to_owned()),
            )
        } else {
            expanded_name(
                source,
                span,
                reader.resolver().resolve_attribute(attribute.key).0,
                raw_name,
            )?
        };
        validate_name(
            source,
            span,
            &name,
            lexical_prefix.as_deref(),
            limits.max_name_bytes,
        )?;
        let value = attribute
            .decode_and_unescape_value(reader.decoder())
            .map_err(|error| parser_error(source, span.end, span.start, span.end, error))?
            .into_owned();
        if value.len() > limits.max_attribute_value_bytes {
            return Err(limit_error(
                source,
                span,
                XmlDiagnosticCode::AttributeValueLimitExceeded,
                format!(
                    "XML attribute value contains {} bytes, exceeding the configured {}-byte limit",
                    value.len(),
                    limits.max_attribute_value_bytes
                ),
            ));
        }
        attributes.push(XmlAttribute {
            name,
            lexical_prefix,
            value,
            span,
        });
    }
    Ok(attributes)
}

fn expanded_name(
    source: XmlSourceId,
    span: XmlSpan,
    resolution: ResolveResult<'_>,
    raw_name: &[u8],
) -> Result<(ExpandedName, Option<String>), XmlDiagnostic> {
    let raw_name = decode_utf8(source, span, raw_name)?;
    let (lexical_prefix, local_name) = split_name(&raw_name);
    let namespace_uri = match resolution {
        ResolveResult::Unbound => None,
        ResolveResult::Bound(namespace) => Some(decode_utf8(source, span, namespace.as_ref())?),
        ResolveResult::Unknown(prefix) => {
            return Err(XmlDiagnostic::at(
                XmlDiagnosticCategory::Namespace,
                XmlDiagnosticCode::UnboundPrefix,
                source,
                span,
                format!(
                    "XML name '{raw_name}' uses an unbound prefix '{}'",
                    decode_utf8(source, span, &prefix)?
                ),
            ));
        }
    };
    Ok((
        ExpandedName {
            namespace_uri,
            local_name,
        },
        lexical_prefix,
    ))
}

fn split_name(name: &str) -> (Option<String>, String) {
    match name.split_once(':') {
        Some((prefix, local)) => (Some(prefix.to_owned()), local.to_owned()),
        None => (None, name.to_owned()),
    }
}

fn validate_name(
    source: XmlSourceId,
    span: XmlSpan,
    name: &ExpandedName,
    prefix: Option<&str>,
    limit: usize,
) -> Result<(), XmlDiagnostic> {
    let bytes = name.local_name.len() + prefix.map_or(0, str::len);
    if bytes > limit {
        return Err(limit_error(
            source,
            span,
            XmlDiagnosticCode::NameLimitExceeded,
            format!("XML name contains {bytes} bytes, exceeding the configured {limit}-byte limit"),
        ));
    }
    Ok(())
}

fn append_text(
    pending: &mut Option<(String, XmlSpan)>,
    text: String,
    span: XmlSpan,
    decoded_text_bytes: &mut usize,
    limit: usize,
) -> Result<(), XmlDiagnostic> {
    *decoded_text_bytes = decoded_text_bytes.saturating_add(text.len());
    if *decoded_text_bytes > limit {
        return Err(limit_error(
            span.source,
            span,
            XmlDiagnosticCode::DecodedTextLimitExceeded,
            format!(
                "decoded XML text contains {} bytes, exceeding the configured {limit}-byte limit",
                *decoded_text_bytes
            ),
        ));
    }
    match pending {
        Some((existing, existing_span)) => {
            existing.push_str(&text);
            existing_span.end = span.end;
        }
        None => *pending = Some((text, span)),
    }
    Ok(())
}

fn flush_pending_text(
    events: &mut Vec<XmlEvent>,
    pending: &mut Option<(String, XmlSpan)>,
    max_nodes: usize,
) -> Result<(), XmlDiagnostic> {
    if let Some((text, span)) = pending.take() {
        push_event(events, XmlEvent::Text { text, span }, max_nodes)?;
    }
    Ok(())
}

fn push_event(
    events: &mut Vec<XmlEvent>,
    event: XmlEvent,
    max_nodes: usize,
) -> Result<(), XmlDiagnostic> {
    if events.len() >= max_nodes {
        let span = event_span(&event);
        return Err(limit_error(
            span.source,
            span,
            XmlDiagnosticCode::NodeLimitExceeded,
            format!("XML event count exceeds the configured {max_nodes}-node limit"),
        ));
    }
    events.push(event);
    Ok(())
}

fn event_span(event: &XmlEvent) -> XmlSpan {
    match event {
        XmlEvent::StartElement { span, .. }
        | XmlEvent::EndElement { span, .. }
        | XmlEvent::Text { span, .. }
        | XmlEvent::Comment { span, .. }
        | XmlEvent::ProcessingInstruction { span, .. } => *span,
    }
}

fn decode_reference(reference: &[u8]) -> Option<String> {
    let reference = str::from_utf8(reference).ok()?;
    let character = match reference {
        "amp" => '&',
        "apos" => '\'',
        "gt" => '>',
        "lt" => '<',
        "quot" => '"',
        decimal if decimal.starts_with('#') => {
            let value = decimal
                .strip_prefix("#x")
                .or_else(|| decimal.strip_prefix("#X"))
                .and_then(|digits| u32::from_str_radix(digits, 16).ok())
                .or_else(|| {
                    decimal
                        .strip_prefix('#')
                        .and_then(|digits| digits.parse().ok())
                })?;
            char::from_u32(value)?
        }
        _ => return None,
    };
    Some(character.to_string())
}

fn decode_utf8(source: XmlSourceId, span: XmlSpan, bytes: &[u8]) -> Result<String, XmlDiagnostic> {
    str::from_utf8(bytes)
        .map(str::to_owned)
        .map_err(|error| encoding_error(source, span, error))
}

fn parser_error(
    source: XmlSourceId,
    input_len: usize,
    fallback_start: usize,
    position: usize,
    error: impl fmt::Display,
) -> XmlDiagnostic {
    let start = position.min(input_len).min(fallback_start);
    let end = position.max(fallback_start).min(input_len);
    XmlDiagnostic::at(
        XmlDiagnosticCategory::Syntax,
        XmlDiagnosticCode::ParserSyntax,
        source,
        XmlSpan::new(source, start, end),
        format!("XML parser rejected the source: {error}"),
    )
}

fn encoding_error(source: XmlSourceId, span: XmlSpan, error: impl fmt::Display) -> XmlDiagnostic {
    XmlDiagnostic::at(
        XmlDiagnosticCategory::Encoding,
        XmlDiagnosticCode::UnsupportedEncoding,
        source,
        span,
        format!("XML text could not be decoded as UTF-8: {error}"),
    )
}

fn limit_error(
    source: XmlSourceId,
    span: XmlSpan,
    code: XmlDiagnosticCode,
    message: impl Into<String>,
) -> XmlDiagnostic {
    XmlDiagnostic::at(
        XmlDiagnosticCategory::ResourceLimit,
        code,
        source,
        span,
        message,
    )
}

#[cfg(test)]
mod tests {
    use super::{
        parse_xml_bytes, parse_xml_events, validate_xml_input, XmlDiagnosticCategory,
        XmlDiagnosticCode, XmlEvent, XmlLimits, XmlOptions, XmlSourceId, XmlSpan,
    };

    #[test]
    fn default_limits_are_valid() {
        assert_eq!(XmlOptions::default().validate(), Ok(()));
    }

    #[test]
    fn zero_limits_are_diagnosed() {
        let options = XmlOptions {
            limits: XmlLimits {
                max_nodes: 0,
                ..XmlLimits::default()
            },
        };

        let error = options.validate().expect_err("zero limits are invalid");
        assert_eq!(error.code, XmlDiagnosticCode::InvalidOptions);
        assert_eq!(error.category, XmlDiagnosticCategory::InternalAdapter);
        assert!(error.message.contains("max_nodes"));
    }

    #[test]
    fn input_limit_is_diagnosed_with_source_identity() {
        let source = XmlSourceId::new(7);
        let options = XmlOptions {
            limits: XmlLimits {
                max_input_bytes: 3,
                ..XmlLimits::default()
            },
        };

        let error = validate_xml_input(source, b"<a/>", options)
            .expect_err("oversized XML input must be diagnosed");
        assert_eq!(error.code, XmlDiagnosticCode::InputTooLarge);
        assert_eq!(error.category, XmlDiagnosticCategory::ResourceLimit);
        assert_eq!(error.source, Some(source));
        assert!(!error.can_continue);
    }

    #[test]
    fn spans_are_half_open_and_source_scoped() {
        let span = XmlSpan::new(XmlSourceId::new(2), 4, 9);
        assert!(span.is_valid());
        assert_eq!(span.source.value(), 2);
        assert_eq!(span.end - span.start, 5);
    }

    #[test]
    fn parses_well_formed_elements_with_stable_source_order() {
        let source = XmlSourceId::new(3);
        let input = include_str!("../../../../tests/fixtures/xml/well-formed/basic-elements.xml");
        let events = parse_xml_events(source, input, XmlOptions::default())
            .expect("well-formed baseline fixture must parse");

        let document = events
            .iter()
            .find(|event| matches!(event, XmlEvent::StartElement { name, .. } if name.local_name == "document"));
        assert!(document.is_some());
        let entry = events.iter().find(|event| {
            matches!(
                event,
                XmlEvent::StartElement { name, attributes, .. }
                    if name.local_name == "entry" && attributes.len() == 1
            )
        });
        assert!(entry.is_some());
        assert!(events
            .iter()
            .any(|event| matches!(event, XmlEvent::Text { text, .. } if text == "ready")));
        assert!(events
            .iter()
            .any(|event| matches!(event, XmlEvent::EndElement { name, .. } if name.local_name == "document")));
    }

    #[test]
    fn expands_element_and_attribute_namespaces() {
        let input = include_str!("../../../../tests/fixtures/xml/namespaces/prefixed-elements.xml");
        let events = parse_xml_events(XmlSourceId::new(4), input, XmlOptions::default())
            .expect("namespace fixture must parse");
        let item = events
            .iter()
            .find_map(|event| match event {
                XmlEvent::StartElement {
                    name, attributes, ..
                } if name.local_name == "item" => Some((name, attributes)),
                _ => None,
            })
            .expect("fixture supplies a prefixed item element");

        assert_eq!(
            item.0.namespace_uri.as_deref(),
            Some("urn:tokimu:xml-fixture")
        );
        assert_eq!(item.0.local_name, "item");
        assert_eq!(
            item.1[0].name.namespace_uri.as_deref(),
            Some("urn:tokimu:xml-fixture")
        );
        assert_eq!(item.1[0].name.local_name, "kind");
    }

    #[test]
    fn decodes_predefined_and_numeric_references() {
        let input =
            include_str!("../../../../tests/fixtures/xml/references/character-references.xml");
        let events = parse_xml_events(XmlSourceId::new(5), input, XmlOptions::default())
            .expect("reference fixture must parse");
        let text = events
            .iter()
            .find_map(|event| match event {
                XmlEvent::Text { text, .. } if !text.trim().is_empty() => Some(text),
                _ => None,
            })
            .expect("fixture contains text");
        assert_eq!(text, "Tom & Ada & 77");
    }

    #[test]
    fn diagnoses_malformed_nesting() {
        let input = include_str!("../../../../tests/fixtures/xml/malformed/mismatched-close.xml");
        let error = parse_xml_events(XmlSourceId::new(6), input, XmlOptions::default())
            .expect_err("mismatched nesting must fail");
        assert_eq!(error.code, XmlDiagnosticCode::ParserSyntax);
        assert_eq!(error.category, XmlDiagnosticCategory::Syntax);
        assert!(error.span.is_some());
    }

    #[test]
    fn diagnoses_declared_non_utf8_encodings() {
        let error = parse_xml_events(
            XmlSourceId::new(7),
            "<?xml version=\"1.0\" encoding=\"ISO-8859-1\"?><root/>",
            XmlOptions::default(),
        )
        .expect_err("declared non-UTF-8 encoding is outside the initial profile");
        assert_eq!(error.code, XmlDiagnosticCode::UnsupportedEncoding);
        assert_eq!(error.category, XmlDiagnosticCategory::Encoding);
    }

    #[test]
    fn diagnoses_non_utf8_source_bytes_before_parser_adaptation() {
        let error = parse_xml_bytes(
            XmlSourceId::new(70),
            b"\xff\xfe<\0r\0o\0o\0t\0/\0>\0",
            XmlOptions::default(),
        )
        .expect_err("UTF-16 source bytes are outside the initial profile");
        assert_eq!(error.code, XmlDiagnosticCode::UnsupportedEncoding);
        assert_eq!(error.category, XmlDiagnosticCategory::Encoding);
        assert!(error.span.is_some());
    }

    #[test]
    fn diagnoses_disabled_doctype_processing() {
        let error = parse_xml_events(
            XmlSourceId::new(8),
            "<!DOCTYPE root SYSTEM \"external.dtd\"><root/>",
            XmlOptions::default(),
        )
        .expect_err("DOCTYPE is intentionally outside the first XML profile");
        assert_eq!(error.code, XmlDiagnosticCode::UnsupportedDocumentType);
        assert_eq!(error.category, XmlDiagnosticCategory::UnsupportedFeature);
    }

    #[test]
    fn diagnoses_depth_limits() {
        let error = parse_xml_events(
            XmlSourceId::new(9),
            "<a><b></b></a>",
            XmlOptions {
                limits: XmlLimits {
                    max_nesting_depth: 1,
                    ..XmlLimits::default()
                },
            },
        )
        .expect_err("nested element must exceed an explicit one-level limit");
        assert_eq!(error.code, XmlDiagnosticCode::NestingDepthExceeded);
        assert_eq!(error.category, XmlDiagnosticCategory::ResourceLimit);
    }

    #[test]
    fn w3c_smoke_selection_records_accepted_rejected_and_unsupported_cases() {
        let source = XmlSourceId::new(71);
        let accepted = include_str!(
            "../../../../third-party/fixtures/w3c-xml-20130923/upstream/xmlconf/eduni/errata-2e/E57.xml"
        );
        let events = parse_xml_events(source, accepted, XmlOptions::default())
            .expect("selected W3C accepted case must parse in the initial profile");
        assert!(events.iter().any(|event| {
            matches!(
                event,
                XmlEvent::StartElement { name, attributes, .. }
                    if name.local_name == "foo"
                        && attributes.iter().any(|attribute| {
                            attribute.name.local_name == "space"
                                && attribute.name.namespace_uri.as_deref()
                                    == Some("http://www.w3.org/XML/1998/namespace")
                        })
            )
        }));

        let malformed = include_str!(
            "../../../../third-party/fixtures/w3c-xml-20130923/upstream/xmlconf/xmltest/not-wf/sa/039.xml"
        );
        let error = parse_xml_events(source, malformed, XmlOptions::default())
            .expect_err("selected W3C malformed case must be rejected");
        assert_eq!(error.code, XmlDiagnosticCode::ParserSyntax);

        let non_utf8 = include_bytes!(
            "../../../../third-party/fixtures/w3c-xml-20130923/upstream/xmlconf/eduni/errata-2e/E61.xml"
        );
        let error = parse_xml_bytes(source, non_utf8, XmlOptions::default())
            .expect_err("selected W3C UTF-16 case must remain explicitly unsupported");
        assert_eq!(error.code, XmlDiagnosticCode::UnsupportedEncoding);
    }
}
