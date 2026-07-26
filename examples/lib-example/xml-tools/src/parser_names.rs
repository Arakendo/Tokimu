use quick_xml::{name::ResolveResult, reader::NsReader};

use crate::parser_support::{decode_utf8, limit_error, parser_error};
use crate::{
    ExpandedName, XmlAttribute, XmlDiagnostic, XmlDiagnosticCategory, XmlDiagnosticCode, XmlLimits,
    XmlSourceId, XmlSpan,
};

pub(crate) const XMLNS_NAMESPACE: &str = "http://www.w3.org/2000/xmlns/";

pub(crate) fn parse_attributes(
    source: XmlSourceId,
    span: XmlSpan,
    reader: &NsReader<&[u8]>,
    element: &quick_xml::events::BytesStart<'_>,
    limits: XmlLimits,
) -> Result<Vec<XmlAttribute>, XmlDiagnostic> {
    let mut attributes = Vec::with_capacity(limits.max_attributes_per_element.min(8));
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

pub(crate) fn expanded_name(
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

pub(crate) fn validate_name(
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
