use quick_xml::{events::Event, reader::NsReader};

use crate::parser_names::{expanded_name, parse_attributes, validate_name};
use crate::parser_state::ParserState;
use crate::parser_support::{decode_reference, decode_utf8, encoding_error, parser_error};
use crate::{
    validate_xml_input, XmlDiagnostic, XmlDiagnosticCategory, XmlDiagnosticCode, XmlEvent,
    XmlOptions, XmlSourceId, XmlSpan,
};

/// Parses UTF-8 XML into a bounded, parser-neutral event sequence.
///
/// This adapter translates private `quick-xml` events. Structural policy such
/// as nesting, document-element rules, text buffering, and EOF diagnostics is
/// owned by [`ParserState`] so it remains independent of parser event types.
pub fn parse_xml_events(
    source: XmlSourceId,
    input: &str,
    options: XmlOptions,
) -> Result<Vec<XmlEvent>, XmlDiagnostic> {
    validate_xml_input(source, input.as_bytes(), options)?;

    let mut reader = NsReader::from_str(input);
    // `xml-tools` owns matching-end diagnostics so callers receive the opening
    // element span as structured context instead of a parser-specific error.
    reader.config_mut().check_end_names = false;
    let mut state = ParserState::new(source, input.len(), options);

    loop {
        let start = reader.buffer_position() as usize;
        let event = reader.read_event().map_err(|error| {
            parser_error(
                source,
                input.len(),
                start,
                reader.error_position() as usize,
                error,
            )
        })?;
        let end = reader.buffer_position() as usize;
        let span = XmlSpan::new(source, start, end);

        match event {
            Event::Start(element) => {
                let resolution = reader.resolver().resolve_element(element.name()).0;
                let (name, lexical_prefix) =
                    expanded_name(source, span, resolution, element.name().as_ref())?;
                validate_name(
                    source,
                    span,
                    &name,
                    lexical_prefix.as_deref(),
                    options.limits.max_name_bytes,
                )?;
                let attributes = parse_attributes(source, span, &reader, &element, options.limits)?;
                state.start_element(name, lexical_prefix, attributes, span)?;
            }
            Event::Empty(element) => {
                let resolution = reader.resolver().resolve_element(element.name()).0;
                let (name, lexical_prefix) =
                    expanded_name(source, span, resolution, element.name().as_ref())?;
                validate_name(
                    source,
                    span,
                    &name,
                    lexical_prefix.as_deref(),
                    options.limits.max_name_bytes,
                )?;
                let attributes = parse_attributes(source, span, &reader, &element, options.limits)?;
                state.empty_element(name, lexical_prefix, attributes, span)?;
            }
            Event::End(element) => {
                let resolution = reader.resolver().resolve_element(element.name()).0;
                let (name, lexical_prefix) =
                    expanded_name(source, span, resolution, element.name().as_ref())?;
                validate_name(
                    source,
                    span,
                    &name,
                    lexical_prefix.as_deref(),
                    options.limits.max_name_bytes,
                )?;
                state.end_element(name, lexical_prefix, span)?;
            }
            Event::Text(text) => {
                let text = text
                    .xml_content()
                    .map_err(|error| encoding_error(source, span, error))?
                    .into_owned();
                state.append_text(text, span)?;
            }
            Event::CData(text) => {
                let text = text
                    .xml_content()
                    .map_err(|error| encoding_error(source, span, error))?
                    .into_owned();
                state.append_text(text, span)?;
            }
            Event::GeneralRef(reference) => {
                let text = decode_reference(reference.as_ref()).ok_or_else(|| {
                    XmlDiagnostic::at(
                        XmlDiagnosticCategory::UnsupportedFeature,
                        XmlDiagnosticCode::UnsupportedEntityReference,
                        source,
                        span,
                        "XML entity references other than predefined and numeric references are unsupported",
                    )
                })?;
                state.append_text(text, span)?;
            }
            Event::Comment(comment) => {
                let text = comment
                    .xml_content()
                    .map_err(|error| encoding_error(source, span, error))?
                    .into_owned();
                state.comment(text, span)?;
            }
            Event::PI(instruction) => {
                let target = decode_utf8(source, span, instruction.target())?;
                let data = decode_utf8(source, span, instruction.content())?;
                state.processing_instruction(
                    target,
                    (!data.trim().is_empty()).then_some(data.trim().to_owned()),
                    span,
                )?;
            }
            Event::Decl(declaration) => {
                if let Some(encoding) = declaration.encoding() {
                    let encoding = encoding
                        .map_err(|error| parser_error(source, input.len(), start, end, error))?;
                    let encoding = decode_utf8(source, span, encoding.as_ref())?;
                    if !encoding.eq_ignore_ascii_case("utf-8")
                        && !encoding.eq_ignore_ascii_case("utf8")
                    {
                        return Err(XmlDiagnostic::at(
                            XmlDiagnosticCategory::Encoding,
                            XmlDiagnosticCode::UnsupportedEncoding,
                            source,
                            span,
                            format!(
                                "XML declaration requests unsupported encoding '{encoding}'; only UTF-8 input is supported"
                            ),
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
            Event::Eof => return state.finish(),
        }
    }
}
