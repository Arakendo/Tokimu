use crate::{
    parse_xml_bytes, parse_xml_document, parse_xml_events, ExpandedName, XmlDiagnosticCategory,
    XmlDiagnosticCode, XmlDocument, XmlDocumentId, XmlEvent, XmlNodeKind, XmlOptions, XmlSourceId,
    XmlSpan,
};

#[test]
fn immutable_document_preserves_parent_child_order_and_spans() {
    let source = XmlSourceId::new(10);
    let input = include_str!("../../../../../tests/fixtures/xml/well-formed/basic-elements.xml");
    let document = parse_xml_document(XmlDocumentId::new(1), source, input, XmlOptions::default())
        .expect("well-formed fixture must retain an immutable document");

    assert_eq!(document.id(), XmlDocumentId::new(1));
    assert_eq!(document.source(), source);
    let root = document
        .document_element()
        .expect("fixture supplies a document element");
    let XmlNodeKind::Element {
        name, attributes, ..
    } = document
        .node_kind(root)
        .expect("root handle belongs to document")
    else {
        panic!("document root must be an element");
    };
    assert_eq!(name.local_name, "document");
    assert_eq!(attributes[0].name.local_name, "id");

    let children = document
        .children(root)
        .expect("root children are available");
    assert_eq!(children.len(), 2);
    let entry = children[0];
    let empty = children[1];
    assert_eq!(document.parent(entry), Some(root));
    assert_eq!(document.parent(empty), Some(root));
    assert!(matches!(
        document.node_kind(entry),
        Some(XmlNodeKind::Element { name, .. }) if name.local_name == "entry"
    ));
    assert!(matches!(
        document.node_kind(empty),
        Some(XmlNodeKind::Element { name, .. }) if name.local_name == "empty"
    ));

    let text = document
        .children(entry)
        .expect("entry children are available")[0];
    assert!(matches!(
        document.node_kind(text),
        Some(XmlNodeKind::Text { text }) if text == "ready"
    ));
    let span = document.node_span(root).expect("root span is available");
    assert_eq!(span.source, source);
    assert!(input[span.start..span.end].starts_with("<document"));
    assert!(input[span.start..span.end].ends_with("</document>"));
}

#[test]
fn immutable_document_preserves_expanded_names() {
    let source = XmlSourceId::new(11);
    let input = include_str!("../../../../../tests/fixtures/xml/namespaces/prefixed-elements.xml");
    let document = parse_xml_document(XmlDocumentId::new(2), source, input, XmlOptions::default())
        .expect("namespaced fixture must retain an immutable document");
    let root = document
        .document_element()
        .expect("fixture supplies a document element");
    let item = document.children(root).expect("root has one child")[0];
    let XmlNodeKind::Element {
        name, attributes, ..
    } = document
        .node_kind(item)
        .expect("item handle belongs to document")
    else {
        panic!("fixture child must be an element");
    };
    assert_eq!(
        name.namespace_uri.as_deref(),
        Some("urn:tokimu:xml-fixture")
    );
    assert_eq!(name.local_name, "item");
    assert_eq!(attributes[0].name.namespace_uri, name.namespace_uri);
}

#[test]
fn document_handles_are_rejected_by_other_documents() {
    let source = XmlSourceId::new(12);
    let first = parse_xml_document(
        XmlDocumentId::new(3),
        source,
        "<first/>",
        XmlOptions::default(),
    )
    .expect("first document must parse");
    let second = parse_xml_document(
        XmlDocumentId::new(4),
        source,
        "<second/>",
        XmlOptions::default(),
    )
    .expect("second document must parse");
    let first_root = first
        .document_element()
        .expect("first document supplies an element");
    assert!(second.node_kind(first_root).is_none());
    assert!(second.children(first_root).is_none());
}

#[test]
fn document_builder_diagnoses_invalid_synthetic_event_order() {
    let source = XmlSourceId::new(13);
    let events = vec![XmlEvent::EndElement {
        name: ExpandedName {
            namespace_uri: None,
            local_name: "root".to_owned(),
        },
        lexical_prefix: None,
        span: XmlSpan::new(source, 0, 7),
    }];
    let error = XmlDocument::from_events(XmlDocumentId::new(5), source, &events)
        .expect_err("document builder must reject unmatched end elements");
    assert_eq!(error.code, XmlDiagnosticCode::DocumentStructure);
    assert_eq!(error.category, XmlDiagnosticCategory::WellFormedness);
}

#[test]
fn w3c_smoke_selection_records_accepted_rejected_and_unsupported_cases() {
    let source = XmlSourceId::new(71);
    let accepted = include_str!(
            "../../../../../third-party/fixtures/w3c-xml-20130923/upstream/xmlconf/eduni/errata-2e/E57.xml"
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
            "../../../../../third-party/fixtures/w3c-xml-20130923/upstream/xmlconf/xmltest/not-wf/sa/039.xml"
        );
    let error = parse_xml_events(source, malformed, XmlOptions::default())
        .expect_err("selected W3C malformed case must be rejected");
    assert_eq!(error.code, XmlDiagnosticCode::ParserSyntax);

    let non_utf8 = include_bytes!(
            "../../../../../third-party/fixtures/w3c-xml-20130923/upstream/xmlconf/eduni/errata-2e/E61.xml"
        );
    let error = parse_xml_bytes(source, non_utf8, XmlOptions::default())
        .expect_err("selected W3C UTF-16 case must remain explicitly unsupported");
    assert_eq!(error.code, XmlDiagnosticCode::UnsupportedEncoding);
}

#[test]
fn seeded_well_formed_documents_preserve_deterministic_events_and_roots() {
    for seed in 1..=32u32 {
        let input = seeded_document(seed);
        let source = XmlSourceId::new(200 + seed);
        let options = XmlOptions::default();
        let first = parse_xml_events(source, &input, options)
            .unwrap_or_else(|error| panic!("seed {seed} must remain well formed: {error}"));
        let second = parse_xml_events(source, &input, options)
            .unwrap_or_else(|error| panic!("seed {seed} must remain repeatable: {error}"));
        assert_eq!(first, second, "seed {seed} must preserve event order");

        let document = parse_xml_document(XmlDocumentId::new(200 + seed), source, &input, options)
            .unwrap_or_else(|error| panic!("seed {seed} must retain a document: {error}"));
        assert!(
            document.document_element().is_some(),
            "seed {seed} must retain one document element"
        );
    }
}

fn seeded_document(seed: u32) -> String {
    let mut state = seed;
    let mut document = String::from("<?xml version=\"1.0\" encoding=\"UTF-8\"?>");
    append_seeded_element(&mut document, &mut state, 0);
    document
}

fn append_seeded_element(output: &mut String, state: &mut u32, depth: usize) {
    let id = next_seed(state) % 97;
    output.push_str(&format!("<node{id} value=\"v{}\">", next_seed(state) % 53));
    output.push_str(&format!("text{}", next_seed(state) % 101));

    let child_count = if depth < 4 { next_seed(state) % 3 } else { 0 };
    for _ in 0..child_count {
        append_seeded_element(output, state, depth + 1);
    }
    output.push_str(&format!("</node{id}>"));
}

fn next_seed(state: &mut u32) -> u32 {
    *state = state.wrapping_mul(1_664_525).wrapping_add(1_013_904_223);
    *state
}
