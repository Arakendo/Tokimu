use std::{fmt, str};

use crate::{
    XmlDiagnostic, XmlDiagnosticCategory, XmlDiagnosticCode, XmlEvent, XmlSourceId, XmlSpan,
};

pub(crate) fn append_text(
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

pub(crate) fn flush_pending_text(
    events: &mut Vec<XmlEvent>,
    pending: &mut Option<(String, XmlSpan)>,
    max_nodes: usize,
) -> Result<(), XmlDiagnostic> {
    if let Some((text, span)) = pending.take() {
        push_event(events, XmlEvent::Text { text, span }, max_nodes)?;
    }
    Ok(())
}

pub(crate) fn push_event(
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

pub(crate) fn event_span(event: &XmlEvent) -> XmlSpan {
    match event {
        XmlEvent::StartElement { span, .. }
        | XmlEvent::EndElement { span, .. }
        | XmlEvent::Text { span, .. }
        | XmlEvent::Comment { span, .. }
        | XmlEvent::ProcessingInstruction { span, .. } => *span,
    }
}

pub(crate) fn decode_reference(reference: &[u8]) -> Option<String> {
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

pub(crate) fn decode_utf8(
    source: XmlSourceId,
    span: XmlSpan,
    bytes: &[u8],
) -> Result<String, XmlDiagnostic> {
    str::from_utf8(bytes)
        .map(str::to_owned)
        .map_err(|error| encoding_error(source, span, error))
}

pub(crate) fn parser_error(
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

pub(crate) fn structure_error(
    source: XmlSourceId,
    span: XmlSpan,
    related_span: Option<XmlSpan>,
    message: impl Into<String>,
) -> XmlDiagnostic {
    let mut diagnostic = XmlDiagnostic::at(
        XmlDiagnosticCategory::Syntax,
        XmlDiagnosticCode::ParserSyntax,
        source,
        span,
        message,
    );
    diagnostic.related_span = related_span;
    diagnostic
}

pub(crate) fn encoding_error(
    source: XmlSourceId,
    span: XmlSpan,
    error: impl fmt::Display,
) -> XmlDiagnostic {
    XmlDiagnostic::at(
        XmlDiagnosticCategory::Encoding,
        XmlDiagnosticCode::UnsupportedEncoding,
        source,
        span,
        format!("XML text could not be decoded as UTF-8: {error}"),
    )
}

pub(crate) fn limit_error(
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
