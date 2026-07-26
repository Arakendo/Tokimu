use std::panic::{catch_unwind, AssertUnwindSafe};

use crate::{
    parse_xml_bytes, parse_xml_events, XmlDiagnosticCategory, XmlDiagnosticCode, XmlEvent,
    XmlLimits, XmlOptions, XmlSourceId, XmlSpan,
};

#[test]
fn parses_well_formed_elements_with_stable_source_order() {
    let source = XmlSourceId::new(3);
    let input = include_str!("../../../../../tests/fixtures/xml/well-formed/basic-elements.xml");
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
    assert!(events.iter().any(
        |event| matches!(event, XmlEvent::EndElement { name, .. } if name.local_name == "document")
    ));
}

#[test]
fn expands_element_and_attribute_namespaces() {
    let input = include_str!("../../../../../tests/fixtures/xml/namespaces/prefixed-elements.xml");
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
        include_str!("../../../../../tests/fixtures/xml/references/character-references.xml");
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
    let input = include_str!("../../../../../tests/fixtures/xml/malformed/mismatched-close.xml");
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
fn diagnoses_node_attribute_name_value_and_text_limits() {
    let source = XmlSourceId::new(90);

    let node_error = parse_xml_events(
        source,
        "<root><child/></root>",
        XmlOptions {
            limits: XmlLimits {
                max_nodes: 3,
                ..XmlLimits::default()
            },
        },
    )
    .expect_err("event count must respect the configured node limit");
    assert_eq!(node_error.code, XmlDiagnosticCode::NodeLimitExceeded);

    let attribute_error = parse_xml_events(
        source,
        "<root first=\"one\" second=\"two\"/>",
        XmlOptions {
            limits: XmlLimits {
                max_attributes_per_element: 1,
                ..XmlLimits::default()
            },
        },
    )
    .expect_err("attribute count must respect the configured limit");
    assert_eq!(
        attribute_error.code,
        XmlDiagnosticCode::AttributeLimitExceeded
    );

    let name_error = parse_xml_events(
        source,
        "<long-name/>",
        XmlOptions {
            limits: XmlLimits {
                max_name_bytes: 4,
                ..XmlLimits::default()
            },
        },
    )
    .expect_err("element names must respect the configured limit");
    assert_eq!(name_error.code, XmlDiagnosticCode::NameLimitExceeded);

    let value_error = parse_xml_events(
        source,
        "<root label=\"too-long\"/>",
        XmlOptions {
            limits: XmlLimits {
                max_attribute_value_bytes: 3,
                ..XmlLimits::default()
            },
        },
    )
    .expect_err("attribute values must respect the configured limit");
    assert_eq!(
        value_error.code,
        XmlDiagnosticCode::AttributeValueLimitExceeded
    );

    let text_error = parse_xml_events(
        source,
        "<root>text</root>",
        XmlOptions {
            limits: XmlLimits {
                max_decoded_text_bytes: 3,
                ..XmlLimits::default()
            },
        },
    )
    .expect_err("decoded text must respect the configured limit");
    assert_eq!(text_error.code, XmlDiagnosticCode::DecodedTextLimitExceeded);
}

#[test]
fn diagnoses_unbound_prefixes_and_truncated_input_at_xml_boundary() {
    let source = XmlSourceId::new(91);
    let namespace_error = parse_xml_events(
        source,
        "<root><unknown:item/></root>",
        XmlOptions::default(),
    )
    .expect_err("unbound prefixes must not become lexical-only identities");
    assert_eq!(namespace_error.code, XmlDiagnosticCode::UnboundPrefix);
    assert_eq!(namespace_error.category, XmlDiagnosticCategory::Namespace);

    let syntax_error = parse_xml_events(source, "<root><child>", XmlOptions::default())
        .expect_err("truncated XML must stop at the XML boundary");
    assert_eq!(syntax_error.code, XmlDiagnosticCode::ParserSyntax);
    assert_eq!(syntax_error.category, XmlDiagnosticCategory::Syntax);
    assert_eq!(
        syntax_error.related_span,
        Some(XmlSpan::new(source, 6, 13)),
        "the opening child element remains available as diagnostic context"
    );

    let mismatch_error = parse_xml_events(source, "<root></other>", XmlOptions::default())
        .expect_err("mismatched end elements must stop at the XML boundary");
    assert_eq!(mismatch_error.code, XmlDiagnosticCode::ParserSyntax);
    assert_eq!(mismatch_error.category, XmlDiagnosticCategory::Syntax);
    assert_eq!(
        mismatch_error.related_span,
        Some(XmlSpan::new(source, 0, 6)),
        "the opening root element remains available as diagnostic context"
    );
}

#[test]
fn rejects_hostile_document_structure_and_disabled_features() {
    let source = XmlSourceId::new(92);
    let cases = [
        (
            "multiple document elements",
            "<first/><second/>",
            XmlDiagnosticCode::ParserSyntax,
        ),
        (
            "text after the document element",
            "<root/>unexpected",
            XmlDiagnosticCode::ParserSyntax,
        ),
        (
            "comment-only input",
            "<!-- no document element -->",
            XmlDiagnosticCode::ParserSyntax,
        ),
        (
            "duplicate attributes",
            "<root id=\"first\" id=\"second\"/>",
            XmlDiagnosticCode::ParserSyntax,
        ),
        (
            "unterminated comment",
            "<root><!-- unfinished</root>",
            XmlDiagnosticCode::ParserSyntax,
        ),
        (
            "unsupported entity reference",
            "<root>&untrusted;</root>",
            XmlDiagnosticCode::UnsupportedEntityReference,
        ),
        (
            "disabled DTD entity declaration",
            "<!DOCTYPE root [<!ENTITY untrusted \"payload\">]><root>&untrusted;</root>",
            XmlDiagnosticCode::UnsupportedDocumentType,
        ),
    ];

    for (name, input, expected_code) in cases {
        let error = parse_xml_events(source, input, XmlOptions::default())
            .expect_err(&format!("hostile case '{name}' must not parse"));
        assert_eq!(error.code, expected_code, "hostile case '{name}'");
        assert_eq!(error.source, Some(source), "hostile case '{name}'");
        assert!(error.span.is_some(), "hostile case '{name}'");
    }
}

#[test]
fn seeded_hostile_inputs_fail_without_panics_or_source_loss() {
    for seed in 1..=64u32 {
        let source = XmlSourceId::new(300 + seed);
        let input = seeded_hostile_input(seed);
        let result = catch_unwind(AssertUnwindSafe(|| {
            parse_xml_events(source, &input, XmlOptions::default())
        }));
        let parsed = result.unwrap_or_else(|_| {
            panic!("hostile seed {seed} must produce a diagnostic instead of panicking")
        });
        let error = parsed.expect_err(&format!(
            "hostile seed {seed} must not be accepted: {input:?}"
        ));
        assert_eq!(
            error.source,
            Some(source),
            "hostile seed {seed} must retain source identity"
        );
        assert!(
            error.span.is_some(),
            "hostile seed {seed} must identify the failing source range"
        );
        assert!(!error.can_continue, "hostile seed {seed} must stop parsing");
    }
}

fn seeded_hostile_input(seed: u32) -> String {
    match seed % 6 {
        0 => format!("<root{seed}><child></root{seed}>"),
        1 => format!("<root{seed}><child>"),
        2 => format!("<first{seed}/><second{seed}/>"),
        3 => format!("<!DOCTYPE root{seed} [<!ENTITY entity{seed} \"payload\">]><root{seed}/>"),
        4 => format!("<root{seed}>&entity{seed};</root{seed}>"),
        _ => format!("<root{seed}/>trailing{seed}"),
    }
}
