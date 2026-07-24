use xml_tools::{
    parse_xml_bytes, parse_xml_document_bytes, XmlDiagnosticCode, XmlDocumentId, XmlOptions,
    XmlSourceId,
};

#[derive(Clone, Copy)]
enum ExpectedResult {
    Accepted,
    Rejected(XmlDiagnosticCode),
    UnsupportedByProfile(XmlDiagnosticCode),
    Deferred,
}

struct SelectedCase {
    id: &'static str,
    bytes: &'static [u8],
    expected: ExpectedResult,
    manifest_record: &'static str,
}

const SELECTION_MANIFEST: &str =
    include_str!("../../../../third-party/fixtures/w3c-xml-20130923/selected/selection-v1.toml");

const SELECTED_CASES: &[SelectedCase] = &[
    SelectedCase {
        id: "eduni-errata-2e-e57",
        bytes: include_bytes!(
            "../../../../third-party/fixtures/w3c-xml-20130923/upstream/xmlconf/eduni/errata-2e/E57.xml"
        ),
        expected: ExpectedResult::Accepted,
        manifest_record: r#"id = "eduni-errata-2e-e57"
path = "eduni/errata-2e/E57.xml"
classification = "Accepted"
capabilities = ["utf-8", "empty-elements", "xml-prefix", "namespace-expanded-attributes"]
reason = "Exercises a no-DTD empty element with the predefined xml prefix and a namespace-resolved attribute."
expected = "parse""#,
    },
    SelectedCase {
        id: "xmltest-not-wf-sa-039",
        bytes: include_bytes!(
            "../../../../third-party/fixtures/w3c-xml-20130923/upstream/xmlconf/xmltest/not-wf/sa/039.xml"
        ),
        expected: ExpectedResult::Rejected(XmlDiagnosticCode::ParserSyntax),
        manifest_record: r#"id = "xmltest-not-wf-sa-039"
path = "xmltest/not-wf/sa/039.xml"
classification = "Rejected"
capabilities = ["well-formedness", "matching-end-tags"]
reason = "Exercises an explicitly malformed end tag without requiring DTD processing."
expected_diagnostic_code = "ParserSyntax""#,
    },
    SelectedCase {
        id: "eduni-errata-2e-e61",
        bytes: include_bytes!(
            "../../../../third-party/fixtures/w3c-xml-20130923/upstream/xmlconf/eduni/errata-2e/E61.xml"
        ),
        expected: ExpectedResult::UnsupportedByProfile(XmlDiagnosticCode::UnsupportedEncoding),
        manifest_record: r#"id = "eduni-errata-2e-e61"
path = "eduni/errata-2e/E61.xml"
classification = "UnsupportedByProfile"
capabilities = ["encoding-declaration", "source-bytes"]
reason = "The fixture is UTF-16. The initial XML profile accepts UTF-8 source bytes only and must diagnose the boundary before parser adaptation."
expected_diagnostic_code = "UnsupportedEncoding""#,
    },
    SelectedCase {
        id: "eduni-namespaces-1-0-013",
        bytes: include_bytes!(
            "../../../../third-party/fixtures/w3c-xml-20130923/upstream/xmlconf/eduni/namespaces/1.0/013.xml"
        ),
        expected: ExpectedResult::Deferred,
        manifest_record: r#"id = "eduni-namespaces-1-0-013"
path = "eduni/namespaces/1.0/013.xml"
classification = "Rejected"
capabilities = ["namespace-qname-validation"]
reason = "Exercises a malformed QName with multiple colons. This is admitted as a future namespace-diagnostic case until the parser adapter maps its failure consistently."
expected = "deferred-smoke""#,
    },
];

#[test]
fn selected_runner_matches_the_reviewed_v1_manifest() {
    for case in SELECTED_CASES {
        assert!(
            SELECTION_MANIFEST.contains(case.manifest_record),
            "selected runner case '{}' must match the reviewed v1 manifest",
            case.id
        );
    }
}

/// Runs the complete reviewed v1 manifest, not the full upstream W3C suite.
///
/// Keep this explicit until the selection grows enough to deserve a dedicated
/// corpus command or CI tier. The test reports deferred cases separately so
/// they cannot be mistaken for either a passing profile claim or a regression.
#[test]
#[ignore = "runs the reviewed W3C XML v1 selection explicitly"]
fn runs_selected_w3c_xml_v1_manifest() {
    let mut accepted = 0usize;
    let mut rejected = 0usize;
    let mut unsupported = 0usize;
    let mut deferred = 0usize;

    for (index, case) in SELECTED_CASES.iter().enumerate() {
        let result = parse_xml_bytes(
            XmlSourceId::new((index + 1) as u32),
            case.bytes,
            XmlOptions::default(),
        );
        match case.expected {
            ExpectedResult::Accepted => {
                result.unwrap_or_else(|error| {
                    panic!("selected accepted case '{}' failed: {error}", case.id)
                });
                let document = parse_xml_document_bytes(
                    XmlDocumentId::new((index + 1) as u32),
                    XmlSourceId::new((index + 1) as u32),
                    case.bytes,
                    XmlOptions::default(),
                )
                .unwrap_or_else(|error| {
                    panic!(
                        "selected accepted case '{}' could not retain a document: {error}",
                        case.id
                    )
                });
                assert!(
                    document.document_element().is_some(),
                    "selected accepted case '{}' must retain a document element",
                    case.id
                );
                accepted += 1;
            }
            ExpectedResult::Rejected(code) => {
                let error = result.expect_err("selected rejected case must not parse");
                assert_eq!(error.code, code, "selected rejected case '{}'", case.id);
                rejected += 1;
            }
            ExpectedResult::UnsupportedByProfile(code) => {
                let error = result.expect_err("unsupported profile case must not parse");
                assert_eq!(error.code, code, "selected unsupported case '{}'", case.id);
                unsupported += 1;
            }
            ExpectedResult::Deferred => {
                result.expect_err("deferred case must still be rejected by the initial profile");
                deferred += 1;
            }
        }
    }

    eprintln!(
        "W3C XML selection v1: accepted={accepted}, rejected={rejected}, unsupported={unsupported}, deferred={deferred}"
    );
}
