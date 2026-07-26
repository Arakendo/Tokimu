use crate::{
    validate_xml_input, XmlDiagnosticCategory, XmlDiagnosticCode, XmlLimits, XmlOptions,
    XmlSourceId, XmlSpan,
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
