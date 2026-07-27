use super::super::*;

use xml_tools::{parse_xml_events, XmlDiagnosticCode, XmlLimits, XmlOptions, XmlSourceId};

#[test]
fn vector_document_adapter_preserves_closed_contours() {
    let paths = parse_svg_document_vector_paths(
        r#"<svg><path d="M0 0 L24 0 L24 24 Z"/></svg>"#,
        8,
        [0.0, 0.0, 24.0, 24.0],
    )
    .unwrap();

    assert_eq!(paths.len(), 1);
    assert_eq!(paths[0].contours.len(), 1);
    assert!(paths[0].contours[0].closed);
}

#[test]
fn vector_document_adapter_ignores_document_metadata() {
    let paths = parse_svg_document_vector_paths(
        r#"<svg id="svg-root"><path id="path-01" d="M0 0 L24 0 L24 24 Z" /></svg>"#,
        8,
        [0.0, 0.0, 24.0, 24.0],
    )
    .expect("metadata must not be parsed as path data");

    assert_eq!(paths.len(), 1);
    assert!(paths[0].contours[0].closed);
}

#[test]
fn vector_document_adapter_ignores_geometry_inside_comments() {
    let paths = parse_svg_document_vector_paths(
        r#"<svg>
                <!-- <path d="M0 0 L24 0 L24 24 Z"/><rect x="0" y="0" width="24" height="24"/> -->
                <line x1="0" y1="0" x2="24" y2="24"/>
            </svg>"#,
        8,
        [0.0, 0.0, 24.0, 24.0],
    )
    .expect("comments should not be treated as geometry");

    assert_eq!(paths.len(), 1);
    assert!(!paths[0].contours[0].closed);
}

#[test]
fn vector_document_adapter_consumes_decoded_attributes_and_ignores_processing_instructions() {
    let records = parse_svg_document_vector_records_with_xml_options(
            r#"<?xml version="1.0"?><svg><?corpus keep?><path d="M0&#x20;0&#x20;L24&#x20;0"/><?corpus keep?><line x1="0" y1="24" x2="24" y2="24"/></svg>"#,
            8,
            [0.0, 0.0, 24.0, 24.0],
            XmlOptions::default(),
        )
        .expect("XML-decoded SVG attributes must reach SVG lowering unchanged");

    assert_eq!(records.len(), 2);
    assert_eq!(
        records[0].path.contours[0].points,
        vec![[-0.5, 0.5], [0.5, 0.5]]
    );
    assert_eq!(
        records[1].path.contours[0].points,
        vec![[-0.5, -0.5], [0.5, -0.5]]
    );
    assert!(records[0].source_span.start < records[1].source_span.start);
}

#[test]
fn vector_document_adapter_does_not_match_element_name_prefixes() {
    let paths = parse_svg_document_vector_paths(
        r#"<svg>
                <pathology d="M0 0 L24 0 L24 24 Z"/>
                <rectangle x="0" y="0" width="24" height="24"/>
            </svg>"#,
        8,
        [0.0, 0.0, 24.0, 24.0],
    )
    .expect("unrelated element names should be ignored");

    assert!(paths.is_empty());
}

#[test]
fn vector_document_adapter_handles_gt_inside_quoted_attributes() {
    let paths = parse_svg_document_vector_paths(
        r#"<svg><path data-label="a > b" d="M0 0 L24 0 L24 24 Z" /></svg>"#,
        8,
        [0.0, 0.0, 24.0, 24.0],
    )
    .expect("quoted attribute text must not terminate the tag early");

    assert_eq!(paths.len(), 1);
    assert!(paths[0].contours[0].closed);
}

#[test]
fn vector_document_adapter_accepts_single_quoted_attributes() {
    let paths = parse_svg_document_vector_paths(
        "<svg><path d='M0 0 L24 0 L24 24 Z' /></svg>",
        8,
        [0.0, 0.0, 24.0, 24.0],
    )
    .expect("single-quoted SVG attributes should parse");

    assert_eq!(paths.len(), 1);
    assert!(paths[0].contours[0].closed);
}

#[test]
fn vector_document_adapter_accepts_single_quoted_path_data() {
    let paths = parse_svg_document_vector_paths(
        "<svg><path d='M0 0 L24 0 L24 24 Z' /></svg>",
        8,
        [0.0, 0.0, 24.0, 24.0],
    )
    .expect("vector adapter should accept single-quoted path data");

    assert_eq!(paths.len(), 1);
    assert_eq!(paths[0].contours.len(), 1);
    assert_eq!(
        paths[0].contours[0].points.first().copied(),
        Some([-0.5, 0.5])
    );
    assert!(paths[0].contours[0].closed);
}

#[test]
fn document_adapters_reject_unterminated_path_elements() {
    let svg = r#"<svg><path d="M0 0 L24 0"#;

    let vector_error = parse_svg_document_vector_paths(svg, 8, [0.0, 0.0, 24.0, 24.0])
        .expect_err("vector adapter must reject truncated path markup");
    assert!(vector_error.contains("SVG XML syntax error"));
}

#[test]
fn structured_svg_diagnostics_preserve_xml_and_svg_boundaries() {
    let xml_error = parse_svg_document_vector_records_with_xml_options(
        r#"<svg><path d="M0 0""#,
        8,
        [0.0, 0.0, 24.0, 24.0],
        XmlOptions::default(),
    )
    .expect_err("truncated XML must preserve its XML diagnostic");
    assert_eq!(xml_error.stage, SvgImportStage::Xml);
    assert_eq!(
        xml_error.xml.as_ref().map(|diagnostic| diagnostic.code),
        Some(XmlDiagnosticCode::ParserSyntax)
    );
    assert!(xml_error.span.is_some());

    let limit_error = parse_svg_document_vector_records_with_xml_options(
        r#"<svg><path d="M0 0 L24 24"/></svg>"#,
        8,
        [0.0, 0.0, 24.0, 24.0],
        XmlOptions {
            limits: XmlLimits {
                max_input_bytes: 8,
                ..XmlLimits::default()
            },
        },
    )
    .expect_err("XML resource limits must remain distinguishable");
    assert_eq!(limit_error.stage, SvgImportStage::Xml);
    assert_eq!(
        limit_error.xml.as_ref().map(|diagnostic| diagnostic.code),
        Some(XmlDiagnosticCode::InputTooLarge)
    );

    let namespace_error = parse_svg_document_vector_records_with_xml_options(
        r#"<svg><foreign:path/></svg>"#,
        8,
        [0.0, 0.0, 24.0, 24.0],
        XmlOptions::default(),
    )
    .expect_err("unbound XML prefixes must stop before SVG interpretation");
    assert_eq!(namespace_error.stage, SvgImportStage::Xml);
    assert_eq!(
        namespace_error
            .xml
            .as_ref()
            .map(|diagnostic| diagnostic.code),
        Some(XmlDiagnosticCode::UnboundPrefix)
    );

    let svg_error = parse_svg_document_vector_records_with_xml_options(
        r#"<svg><rect x="0" y="0" width="-1" height="1"/></svg>"#,
        8,
        [0.0, 0.0, 24.0, 24.0],
        XmlOptions::default(),
    )
    .expect_err("valid XML with unsupported SVG geometry must be an SVG diagnostic");
    assert_eq!(svg_error.stage, SvgImportStage::Svg);
    assert!(svg_error.xml.is_none());
    assert!(svg_error.span.is_some());
}

#[test]
fn semantic_pass_has_explicit_root_and_namespace_policy() {
    let prefixed = parse_svg_document_vector_records_with_xml_options(
            r#"<svg:svg xmlns:svg="http://www.w3.org/2000/svg"><svg:path d="M0 0 L24 0 L24 24 Z"/></svg:svg>"#,
            8,
            [0.0, 0.0, 24.0, 24.0],
            XmlOptions::default(),
        )
        .expect("SVG-prefixed elements must be admitted through expanded names");
    assert_eq!(prefixed.len(), 1);
    assert_eq!(prefixed[0].source_span.source.value(), 0);

    let default_namespaced = parse_svg_document_vector_records_with_xml_options(
        r#"<svg xmlns="http://www.w3.org/2000/svg"><path d="M0 0 L24 0 L24 24 Z"/></svg>"#,
        8,
        [0.0, 0.0, 24.0, 24.0],
        XmlOptions::default(),
    )
    .expect("default SVG namespaces must be admitted through expanded names");
    assert_eq!(default_namespaced.len(), 1);

    let foreign = parse_svg_document_vector_records_with_xml_options(
            r#"<svg xmlns="http://www.w3.org/2000/svg" xmlns:other="urn:other"><other:path d="M0 0 L24 0 L24 24 Z"/></svg>"#,
            8,
            [0.0, 0.0, 24.0, 24.0],
            XmlOptions::default(),
        )
        .expect("foreign geometry names must not become SVG paths by local-name collision");
    assert!(foreign.is_empty());

    let invalid_root = parse_svg_document_vector_records_with_xml_options(
        r#"<document><path d="M0 0 L24 0 L24 24 Z"/></document>"#,
        8,
        [0.0, 0.0, 24.0, 24.0],
        XmlOptions::default(),
    )
    .expect_err("a valid XML non-SVG document must be an SVG-stage diagnostic");
    assert_eq!(invalid_root.stage, SvgImportStage::Svg);
    assert!(invalid_root.xml.is_none());
}

#[test]
fn semantic_profile_diagnoses_unadmitted_svg_features() {
    for element in [
        r#"<text x="1" y="1">not admitted</text>"#,
        r#"<mask id="mask"><rect width="24" height="24"/></mask>"#,
    ] {
        let diagnostic = parse_svg_document_vector_records_with_xml_options(
            &format!("<svg>{element}</svg>"),
            8,
            [0.0, 0.0, 24.0, 24.0],
            XmlOptions::default(),
        )
        .expect_err("unadmitted SVG features must not be silently accepted");
        assert_eq!(diagnostic.stage, SvgImportStage::Svg);
        assert!(diagnostic.span.is_some());
        assert!(diagnostic
            .message
            .contains("outside the admitted importer profile"));
    }
}

#[test]
fn vector_document_adapter_rejects_unterminated_primitive_elements() {
    let error = parse_svg_document_vector_paths(
        r#"<svg><rect x="0" y="0" width="24""#,
        8,
        [0.0, 0.0, 24.0, 24.0],
    )
    .expect_err("vector adapter must reject truncated primitive markup");

    assert!(error.contains("SVG XML syntax error"));
}

#[test]
fn vector_document_adapter_stops_at_the_xml_profile_boundary() {
    let error = parse_svg_document_vector_paths(
        r#"<!DOCTYPE svg SYSTEM "external.dtd"><svg><path d="M0 0 L24 0"/></svg>"#,
        8,
        [0.0, 0.0, 24.0, 24.0],
    )
    .expect_err("disabled XML DTD processing must stop before SVG interpretation");

    assert!(error.contains("UnsupportedDocumentType"));
    assert!(error.contains("DOCTYPE declarations"));
}

#[test]
fn vector_document_adapter_accepts_whitespace_around_attribute_equals() {
    let records = parse_svg_document_vector_records(
        r#"<svg><path d = "M0 0 L24 0 L24 24 Z" fill = "none" stroke = "black"/></svg>"#,
        8,
        [0.0, 0.0, 24.0, 24.0],
    )
    .expect("whitespace around attribute equals should be accepted");

    assert_eq!(records.len(), 1);
    assert!(!records[0].fill);
    assert!(records[0].stroke);
    assert!(records[0].path.contours[0].closed);
}

#[test]
fn vector_document_adapter_preserves_open_and_multiple_contours() {
    let paths = parse_svg_document_vector_paths(
        r#"<svg><path d="M0 0 L24 0 M0 24 L24 24"/></svg>"#,
        8,
        [0.0, 0.0, 24.0, 24.0],
    )
    .unwrap();

    assert_eq!(paths.len(), 1);
    assert_eq!(paths[0].contours.len(), 2);
    assert!(paths[0].contours.iter().all(|contour| !contour.closed));
}

#[test]
fn vector_document_adapter_handles_primitive_elements() {
    let paths = parse_svg_document_vector_paths(
            r#"<svg><circle cx="12" cy="12" r="4"/><ellipse cx="12" cy="12" rx="6" ry="4"/><rect x="2" y="3" width="5" height="6"/><line x1="0" y1="0" x2="24" y2="24"/></svg>"#,
            8,
            [0.0, 0.0, 24.0, 24.0],
        )
        .unwrap();

    assert_eq!(paths.len(), 4);
    assert!(paths[0].contours[0].closed);
    assert!(paths[1].contours[0].closed);
    assert!(paths[2].contours[0].closed);
    assert!(!paths[3].contours[0].closed);
}

#[test]
fn primitive_elements_reject_negative_dimensions() {
    let circle_error = parse_svg_document_vector_paths(
        r#"<svg><circle cx="12" cy="12" r="-4"/></svg>"#,
        8,
        [0.0, 0.0, 24.0, 24.0],
    )
    .expect_err("negative circle radii must be rejected");
    assert!(circle_error.contains("circle radius"));

    let ellipse_error = parse_svg_document_vector_paths(
        r#"<svg><ellipse cx="12" cy="12" rx="6" ry="-4"/></svg>"#,
        8,
        [0.0, 0.0, 24.0, 24.0],
    )
    .expect_err("negative ellipse radii must be rejected");
    assert!(ellipse_error.contains("ellipse radii"));

    let rect_error = parse_svg_document_vector_paths(
        r#"<svg><rect x="0" y="0" width="-4" height="8"/></svg>"#,
        8,
        [0.0, 0.0, 24.0, 24.0],
    )
    .expect_err("negative rectangle dimensions must be rejected");
    assert!(rect_error.contains("width and height"));

    let radius_error = parse_svg_document_vector_records(
        r#"<svg><rect x="0" y="0" width="8" height="8" rx="-1"/></svg>"#,
        8,
        [0.0, 0.0, 24.0, 24.0],
    )
    .expect_err("negative rectangle radii must be rejected");
    assert!(radius_error.contains("corner radii"));
}

#[test]
fn rounded_rect_couples_an_omitted_radius() {
    let records = parse_svg_document_vector_records(
            r#"<svg><rect x="0" y="0" width="12" height="8" ry="2"/><rect x="20" y="0" width="12" height="8" rx="2"/></svg>"#,
            8,
            [0.0, 0.0, 32.0, 8.0],
        )
        .expect("rounded rectangles with one radius should parse");

    assert_eq!(records.len(), 2);
    assert!(records[0].path.contours[0].points.len() > 5);
    assert_eq!(
        records[0].path.contours[0].points.len(),
        records[1].path.contours[0].points.len()
    );
}

#[test]
fn vector_document_adapter_preserves_mixed_element_order() {
    let records = parse_svg_document_vector_records(
        r#"<svg>
                <rect x="0" y="0" width="4" height="4"/>
                <path d="M8 0 L12 0 L12 4 Z"/>
                <line x1="16" y1="0" x2="20" y2="4"/>
            </svg>"#,
        8,
        [0.0, 0.0, 20.0, 4.0],
    )
    .expect("mixed SVG elements should preserve source order");

    assert_eq!(records.len(), 3);
    assert_eq!(records[0].path.contours[0].points[0], [-0.5, 0.5]);
    let path_start = records[1].path.contours[0].points[0];
    let line_start = records[2].path.contours[0].points[0];
    assert!((path_start[0] + 0.1).abs() < 1.0e-6 && (path_start[1] - 0.5).abs() < 1.0e-6);
    assert!((line_start[0] - 0.3).abs() < 1.0e-6 && (line_start[1] - 0.5).abs() < 1.0e-6);
}

#[test]
fn vector_document_adapter_rejects_unmatched_trailing_point_coordinate() {
    let error = parse_svg_document_vector_paths(
        r#"<svg><polyline points="0,0 12,12 24"/></svg>"#,
        8,
        [0.0, 0.0, 24.0, 24.0],
    )
    .expect_err("an unmatched trailing coordinate must not be discarded");

    assert!(error.contains("requires an even number of coordinates"));
}

#[test]
fn vector_document_adapter_rejects_invalid_polyline_numbers() {
    let error = parse_svg_document_vector_paths(
        r#"<svg><polyline points="0,0 nope,12"/></svg>"#,
        8,
        [0.0, 0.0, 24.0, 24.0],
    )
    .expect_err("an invalid primitive coordinate must not be discarded");

    assert!(error.contains("invalid number 'nope'"));
}

#[test]
fn vector_document_adapter_rejects_non_finite_polyline_numbers() {
    let error = parse_svg_document_vector_paths(
        r#"<svg><polyline points="0,0 NaN,12"/></svg>"#,
        8,
        [0.0, 0.0, 24.0, 24.0],
    )
    .expect_err("non-finite primitive coordinates must not enter geometry");

    assert!(error.contains("non-finite number 'NaN'"));
}

#[test]
fn vector_document_adapter_rejects_malformed_numeric_attributes() {
    for (element, attribute) in [
        ("circle", "r"),
        ("ellipse", "rx"),
        ("line", "x1"),
        ("rect", "width"),
    ] {
        let source = match element {
            "circle" => format!(r#"<svg><circle r="nope"/></svg>"#),
            "ellipse" => format!(r#"<svg><ellipse rx="nope" ry="2"/></svg>"#),
            "line" => format!(r#"<svg><line x1="nope" y1="0" x2="1" y2="1"/></svg>"#),
            "rect" => format!(r#"<svg><rect width="nope" height="1"/></svg>"#),
            _ => unreachable!(),
        };
        let error = parse_svg_document_vector_paths(&source, 8, [0.0, 0.0, 24.0, 24.0])
            .expect_err("a malformed numeric attribute must not become a default");
        assert!(
            error.contains(&format!("attribute '{attribute}'")),
            "{error}"
        );
        assert!(error.contains("invalid number 'nope'"), "{error}");
    }
}

#[test]
fn vector_document_adapter_rejects_invalid_view_box_dimensions() {
    let error = parse_svg_document_vector_paths(
        r#"<svg><path d="M0 0 L1 1"/></svg>"#,
        8,
        [0.0, 0.0, 0.0, 24.0],
    )
    .expect_err("a zero-width viewBox cannot be normalized");

    assert!(error.contains("positive dimensions"));
}

#[test]
fn vector_document_adapter_normalizes_negative_primitive_coordinates() {
    let records = parse_svg_document_vector_records(
        r#"<svg><line x1="-10" y1="-5" x2="10" y2="5"/></svg>"#,
        8,
        [-10.0, -5.0, 20.0, 10.0],
    )
    .expect("negative source coordinates should normalize");

    assert_eq!(records.len(), 1);
    assert_eq!(records[0].path.contours[0].points[0], [-0.5, 0.5]);
    assert_eq!(records[0].path.contours[0].points[1], [0.5, -0.5]);
}

#[test]
fn vector_records_preserve_fill_and_stroke_intent() {
    let records = parse_svg_document_vector_records(
            r#"<svg>
                <path d="M0 0 L24 0 L24 24 Z" fill="none" stroke="black"/>
                <path d="M1 1 L23 1 L23 23 Z" style="fill: none; stroke: black; fill-rule: evenodd"/>
                <rect x="2" y="2" width="4" height="4"/>
            </svg>"#,
            8,
            [0.0, 0.0, 24.0, 24.0],
        )
        .expect("SVG paint metadata should parse");

    assert_eq!(records.len(), 3);
    assert!(!records[0].fill && records[0].stroke);
    assert!(!records[1].fill && records[1].stroke);
    assert_eq!(records[1].fill_rule, SvgFillRule::EvenOdd);
    assert!(records[2].fill && !records[2].stroke);
    assert_eq!(records[2].fill_rule, SvgFillRule::NonZero);
}

#[test]
fn vector_records_preserve_bounded_solid_paint_and_current_color() {
    let records = parse_svg_document_vector_records(
        r##"<svg color="#123456" fill="#abc" stroke="currentColor">
                <path d="M0 0 L2 0 L2 2 Z"/>
                <path fill="none" stroke="red" d="M3 0 L5 0 L5 2 Z"/>
            </svg>"##,
        8,
        [0.0, 0.0, 10.0, 10.0],
    )
    .expect("bounded solid paint should lower");

    assert_eq!(records.len(), 2);
    assert_eq!(
        records[0].fill_color,
        Some(SvgColor::Rgba([
            0xaa as f32 / 255.0,
            0xbb as f32 / 255.0,
            0xcc as f32 / 255.0,
            1.0
        ]))
    );
    assert_eq!(
        records[0].stroke_color,
        Some(SvgColor::Rgba([
            0x12 as f32 / 255.0,
            0x34 as f32 / 255.0,
            0x56 as f32 / 255.0,
            1.0
        ]))
    );
    assert_eq!(records[1].fill_color, None);
    assert_eq!(
        records[1].stroke_color,
        Some(SvgColor::Rgba([1.0, 0.0, 0.0, 1.0]))
    );
}

#[test]
fn unsupported_solid_paint_is_diagnosed() {
    let error = parse_svg_document_vector_records(
        r#"<svg><rect width="2" height="2" fill="url(#gradient)"/></svg>"#,
        8,
        [0.0, 0.0, 10.0, 10.0],
    )
    .expect_err("gradient paint must remain outside the solid paint slice");
    assert!(error.contains("solid color"), "{error}");
}

#[test]
fn vector_records_preserve_inherited_opacity_intent() {
    let records = parse_svg_document_vector_records(
        r#"<svg opacity="0.8" fill-opacity="0.7" stroke-opacity="0.6">
                <g>
                    <path d="M0 0 L2 0 L2 2 Z"/>
                    <path opacity="0.4" fill-opacity="0.3" stroke-opacity="0.2"
                        d="M3 0 L5 0 L5 2 Z"/>
                </g>
            </svg>"#,
        8,
        [0.0, 0.0, 10.0, 10.0],
    )
    .expect("opacity intent should lower through the SVG profile");

    assert_eq!(records.len(), 2);
    assert_eq!(records[0].opacity, 0.8);
    assert_eq!(records[0].fill_opacity, 0.7);
    assert_eq!(records[0].stroke_opacity, 0.6);
    assert_eq!(records[1].opacity, 0.4);
    assert_eq!(records[1].fill_opacity, 0.3);
    assert_eq!(records[1].stroke_opacity, 0.2);
}

#[test]
fn invalid_opacity_intent_is_diagnosed() {
    for attribute in [r#"opacity="1.1""#, r#"fill-opacity="nope""#] {
        let source = format!(r#"<svg {attribute}><rect width="2" height="2"/></svg>"#);
        let error = parse_svg_document_vector_records(&source, 8, [0.0, 0.0, 10.0, 10.0])
            .expect_err("invalid opacity must remain visible as a diagnostic");
        assert!(error.contains("opacity"), "{error}");
    }
}

#[test]
fn vector_records_preserve_bounded_stroke_intent_and_inheritance() {
    let records = parse_svg_document_vector_records(
        r#"<svg stroke="black" stroke-width="3" stroke-linecap="round" stroke-linejoin="bevel" stroke-miterlimit="6">
                <g>
                    <path d="M0 0 L12 0"/>
                    <path stroke-width="2" stroke-linecap="square" d="M0 1 L12 1"/>
                </g>
            </svg>"#,
        8,
        [0.0, 0.0, 24.0, 24.0],
    )
    .expect("bounded stroke intent should lower through the SVG profile");

    assert_eq!(records.len(), 2);
    assert_eq!(records[0].stroke_width, 3.0);
    assert_eq!(records[0].stroke_linecap, SvgStrokeLinecap::Round);
    assert_eq!(records[0].stroke_linejoin, SvgStrokeLinejoin::Bevel);
    assert_eq!(records[0].stroke_miterlimit, 6.0);
    assert_eq!(records[1].stroke_width, 2.0);
    assert_eq!(records[1].stroke_linecap, SvgStrokeLinecap::Square);
    assert_eq!(records[1].stroke_linejoin, SvgStrokeLinejoin::Bevel);
    assert_eq!(records[1].stroke_miterlimit, 6.0);
}

#[test]
fn vector_records_preserve_dash_pattern_and_phase() {
    let records = parse_svg_document_vector_records(
        r#"<svg stroke="black" stroke-dasharray="6 3 2" stroke-dashoffset="-4">
                <path d="M0 0 L12 0"/>
                <path style="stroke-dasharray: none; stroke-dashoffset: 2" d="M0 1 L12 1"/>
            </svg>"#,
        8,
        [0.0, 0.0, 24.0, 24.0],
    )
    .expect("dash presentation should parse");

    assert_eq!(
        records[0].stroke_dasharray,
        Some(vec![6.0, 3.0, 2.0, 6.0, 3.0, 2.0])
    );
    assert_eq!(records[0].stroke_dashoffset, -4.0);
    assert_eq!(records[1].stroke_dasharray, None);
    assert_eq!(records[1].stroke_dashoffset, 2.0);
}

#[test]
fn vector_records_reject_invalid_stroke_parameters() {
    for attribute in [
        r#"stroke-width="-1""#,
        r#"stroke-linecap="triangle""#,
        r#"stroke-linejoin="spike""#,
        r#"stroke-miterlimit="0""#,
        r#"stroke-dasharray="0 0""#,
        r#"stroke-dasharray="4 -1""#,
        r#"stroke-dashoffset="nan""#,
    ] {
        let source = format!(r#"<svg stroke="black" {attribute}><path d="M0 0 L1 1"/></svg>"#);
        let error = parse_svg_document_vector_records(&source, 8, [0.0, 0.0, 24.0, 24.0])
            .expect_err("invalid stroke parameters must remain visible diagnostics");
        assert!(error.contains("stroke-"), "{error}");
    }
}

#[test]
fn nested_svg_presentation_state_inherits_and_restores_for_siblings() {
    let records = parse_svg_document_vector_records(
        r#"<svg fill="none" stroke="black" fill-rule="evenodd">
                <g fill="white" style="fill: none; stroke: none">
                    <path d="M0 0 L24 0 L24 24 Z"/>
                </g>
                <path d="M1 1 L23 1 L23 23 Z"/>
                <g fill="none" stroke="none" fill-rule="nonzero">
                    <path d="M2 2 L22 2 L22 22 Z" fill="inherit"/>
                </g>
                <path d="M3 3 L21 3 L21 21 Z"/>
            </svg>"#,
        8,
        [0.0, 0.0, 24.0, 24.0],
    )
    .expect("nested SVG presentation state should lower deterministically");

    assert_eq!(records.len(), 4);

    // Supported inline style declarations override presentation attributes.
    assert!(!records[0].fill);
    assert!(!records[0].stroke);
    assert_eq!(records[0].fill_rule, SvgFillRule::EvenOdd);

    assert!(!records[1].fill && records[1].stroke);
    assert_eq!(records[1].fill_rule, SvgFillRule::EvenOdd);

    assert!(!records[2].fill && !records[2].stroke);
    assert_eq!(records[2].fill_rule, SvgFillRule::NonZero);

    assert!(!records[3].fill && records[3].stroke);
    assert_eq!(records[3].fill_rule, SvgFillRule::EvenOdd);
}

#[test]
fn supported_inline_style_overrides_attributes_and_uses_last_declaration() {
    let records = parse_svg_document_vector_records(
        r#"<svg stroke="black" stroke-width="2">
                <line x1="0" y1="0" x2="10" y2="0"
                    stroke="none"
                    style="stroke: black; stroke-width: 3; stroke-width: 4"/>
            </svg>"#,
        8,
        [0.0, 0.0, 10.0, 10.0],
    )
    .expect("supported inline style should lower");

    assert_eq!(records.len(), 1);
    assert!(records[0].stroke);
    assert_eq!(records[0].stroke_width, 4.0);
}

#[test]
fn supported_style_numeric_values_report_malformed_overrides() {
    let error = parse_svg_document_vector_records(
        r#"<svg stroke-width="2">
                <line x1="0" y1="0" x2="10" y2="0" style="stroke-width: nope"/>
            </svg>"#,
        8,
        [0.0, 0.0, 10.0, 10.0],
    )
    .expect_err("malformed supported style values must remain visible");
    assert!(error.contains("stroke-width"), "{error}");
}

#[test]
fn defs_geometry_is_non_rendering_storage_until_referenced() {
    let records = parse_svg_document_vector_records(
        r#"<svg>
                <defs>
                    <rect x="1" y="1" width="8" height="8"/>
                </defs>
                <rect x="2" y="2" width="4" height="4"/>
            </svg>"#,
        8,
        [0.0, 0.0, 10.0, 10.0],
    )
    .expect("defs should be accepted as non-rendering storage");

    assert_eq!(records.len(), 1);
    let bounds = records[0].path.bounds().expect("rendered bounds");
    assert!((bounds.0[0] + 0.3).abs() < 1.0e-6);
    assert!((bounds.0[1] + 0.1).abs() < 1.0e-6);
    assert!((bounds.1[0] - 0.1).abs() < 1.0e-6);
    assert!((bounds.1[1] - 0.3).abs() < 1.0e-6);
}

#[test]
fn local_geometric_clip_path_is_preserved_on_the_target_record() {
    let records = parse_svg_document_vector_records(
        r##"<svg>
                <defs>
                    <clipPath id="clip"><circle cx="5" cy="5" r="2"/></clipPath>
                </defs>
                <rect x="0" y="0" width="10" height="10" clip-path="url(#clip)"/>
            </svg>"##,
        16,
        [0.0, 0.0, 10.0, 10.0],
    )
    .expect("local geometric clip path should lower");

    assert_eq!(records.len(), 1);
    let clip_path = records[0].clip_path.as_ref().expect("resolved clip path");
    assert!(clip_path.bounds().is_some());
}

#[test]
fn transformed_local_clip_path_uses_the_shared_transform_stack() {
    let records = parse_svg_document_vector_records(
        r##"<svg>
                <defs>
                    <clipPath id="clip" transform="translate(2 1) scale(2)">
                        <rect x="0" y="0" width="2" height="3"/>
                    </clipPath>
                </defs>
                <rect transform="translate(1 2)" x="0" y="0" width="8" height="8"
                      clip-path="url(#clip)"/>
            </svg>"##,
        8,
        [0.0, 0.0, 20.0, 20.0],
    )
    .expect("transformed local clip path should lower");

    let clip_bounds = records[0]
        .clip_path
        .as_ref()
        .and_then(|path| path.bounds())
        .expect("clip bounds should be preserved");
    assert!((clip_bounds.1[0] - clip_bounds.0[0] - 0.2).abs() < 1.0e-6);
    assert!((clip_bounds.1[1] - clip_bounds.0[1] - 0.3).abs() < 1.0e-6);
    assert!(clip_bounds.0[0].is_finite() && clip_bounds.0[1].is_finite());
}

#[test]
fn clip_path_geometry_uses_the_shared_lowering_for_core_shape_families() {
    let records = parse_svg_document_vector_records(
        r##"<svg>
                <defs>
                    <clipPath id="rect"><rect x="0" y="0" width="4" height="4"/></clipPath>
                    <clipPath id="circle"><circle cx="2" cy="2" r="2"/></clipPath>
                    <clipPath id="polygon"><polygon points="0,0 4,0 2,4"/></clipPath>
                    <clipPath id="path"><path d="M0 0 C1 4 3 4 4 0 Z"/></clipPath>
                </defs>
                <rect x="0" y="0" width="4" height="4" clip-path="url(#rect)"/>
                <rect x="5" y="0" width="4" height="4" clip-path="url(#circle)"/>
                <rect x="0" y="5" width="4" height="4" clip-path="url(#polygon)"/>
                <rect x="5" y="5" width="4" height="4" clip-path="url(#path)"/>
            </svg>"##,
        16,
        [0.0, 0.0, 10.0, 10.0],
    )
    .expect("core clip shape families should share SVG lowering");

    assert_eq!(records.len(), 4);
    for record in records {
        assert!(record.clip_path.is_some(), "clip path should be attached");
        assert!(
            record
                .clip_path
                .as_ref()
                .and_then(|path| path.bounds())
                .is_some(),
            "clip path should retain finite geometry"
        );
    }
}

#[test]
fn clip_path_references_report_missing_and_multiple_geometry() {
    let missing = parse_svg_document_vector_records(
        r##"<svg><rect x="0" y="0" width="10" height="10" clip-path="url(#missing)"/></svg>"##,
        8,
        [0.0, 0.0, 10.0, 10.0],
    )
    .expect_err("missing clip paths must be diagnosed");
    assert!(missing.contains("clip-path target '#missing'"), "{missing}");

    let multiple = parse_svg_document_vector_records(
        r##"<svg><defs><clipPath id="clip">
                <rect x="0" y="0" width="4" height="4"/><circle cx="5" cy="5" r="2"/>
            </clipPath></defs></svg>"##,
        8,
        [0.0, 0.0, 10.0, 10.0],
    )
    .expect_err("multiple clip geometry must remain outside the first slice");
    assert!(
        multiple.contains("multiple geometric children"),
        "{multiple}"
    );

    let nested = parse_svg_document_vector_records(
        r##"<svg><defs><clipPath id="outer"><clipPath id="inner">
                <rect x="0" y="0" width="4" height="4"/>
            </clipPath></clipPath></defs></svg>"##,
        8,
        [0.0, 0.0, 10.0, 10.0],
    )
    .expect_err("nested clip paths must remain outside the first slice");
    assert!(nested.contains("nested SVG clipPath"), "{nested}");
}

#[test]
fn local_use_reuses_a_previously_defined_geometry_record() {
    let records = parse_svg_document_vector_records(
        r##"<svg viewBox="0 0 10 10">
                <defs><rect id="box" x="1" y="2" width="4" height="3"/></defs>
                <use href="#box"/>
                <rect x="1" y="2" width="4" height="3"/>
            </svg>"##,
        8,
        [0.0, 0.0, 10.0, 10.0],
    )
    .expect("local geometric use should lower");

    assert_eq!(records.len(), 2);
    assert_eq!(records[0].path, records[1].path);
}

#[test]
fn local_xlink_use_reuses_a_geometric_definition() {
    let records = parse_svg_document_vector_records(
        r##"<svg xmlns:xlink="http://www.w3.org/1999/xlink">
                <defs><circle id="dot" cx="5" cy="5" r="2"/></defs>
                <use xlink:href="#dot"/>
            </svg>"##,
        16,
        [0.0, 0.0, 10.0, 10.0],
    )
    .expect("xlink local geometric use should lower");

    assert_eq!(records.len(), 1);
    assert!(records[0].path.bounds().is_some());
}

#[test]
fn use_references_report_missing_external_and_cyclic_targets() {
    let missing = parse_svg_document_vector_records(
        r##"<svg><use href="#missing"/></svg>"##,
        8,
        [0.0, 0.0, 10.0, 10.0],
    )
    .expect_err("missing local targets must be diagnosed");
    assert!(
        missing.contains("target '#missing' is missing"),
        "{missing}"
    );

    let external = parse_svg_document_vector_records(
        r##"<svg><use href="other.svg#box"/></svg>"##,
        8,
        [0.0, 0.0, 10.0, 10.0],
    )
    .expect_err("external targets must remain outside the profile");
    assert!(external.contains("external"), "{external}");

    let cyclic = parse_svg_document_vector_records(
        r##"<svg>
                <defs><use id="loop" href="#loop"/></defs>
                <use href="#loop"/>
            </svg>"##,
        8,
        [0.0, 0.0, 10.0, 10.0],
    )
    .expect_err("cyclic or non-geometric targets must be diagnosed");
    assert!(cyclic.contains("cyclic or non-geometric"), "{cyclic}");
}

#[test]
fn use_references_reject_duplicate_ids_and_use_site_overrides() {
    let duplicate = parse_svg_document_vector_records(
        r##"<svg><defs>
                <rect id="box" x="0" y="0" width="2" height="2"/>
                <circle id="box" cx="5" cy="5" r="1"/>
            </defs></svg>"##,
        8,
        [0.0, 0.0, 10.0, 10.0],
    )
    .expect_err("duplicate definition ids must be diagnosed");
    assert!(
        duplicate.contains("duplicate SVG definition id 'box'"),
        "{duplicate}"
    );

    let override_error = parse_svg_document_vector_records(
        r##"<svg><defs><rect id="box" x="0" y="0" width="2" height="2"/></defs>
                <use href="#box" transform="translate(1 1)"/>
            </svg>"##,
        8,
        [0.0, 0.0, 10.0, 10.0],
    )
    .expect_err("use-site transforms must remain an explicit profile gap");
    assert!(
        override_error.contains("overrides and transforms"),
        "{override_error}"
    );
}

#[test]
fn svg_transforms_compose_through_nested_state_before_normalization() {
    let assert_point = |actual: [f32; 2], expected: [f32; 2]| {
        assert!(
            (actual[0] - expected[0]).abs() < 1.0e-5 && (actual[1] - expected[1]).abs() < 1.0e-5,
            "expected {expected:?}, received {actual:?}"
        );
    };
    let records = parse_svg_document_vector_records(
        r#"<svg>
                <g transform="translate(10 20)">
                    <path transform="scale(2)" d="M0 0 L10 0"/>
                    <path transform="rotate(90 10 10)" d="M20 10 L20 20"/>
                </g>
            </svg>"#,
        8,
        [0.0, 0.0, 100.0, 100.0],
    )
    .expect("supported transforms should lower through the SVG state stack");

    assert_eq!(records.len(), 2);
    assert_point(records[0].path.contours[0].points[0], [-0.4, 0.3]);
    assert_point(records[0].path.contours[0].points[1], [-0.2, 0.3]);
    assert_point(records[1].path.contours[0].points[0], [-0.3, 0.1]);
    assert_point(records[1].path.contours[0].points[1], [-0.4, 0.1]);

    let listed = parse_svg_document_vector_records(
        r#"<svg><path transform="translate(10) scale(2)" d="M10 0 L20 0"/></svg>"#,
        8,
        [0.0, 0.0, 100.0, 100.0],
    )
    .expect("transform lists should compose in SVG order");
    assert_point(listed[0].path.contours[0].points[0], [-0.2, 0.5]);
    assert_point(listed[0].path.contours[0].points[1], [0.0, 0.5]);

    let compact = parse_svg_document_vector_records(
        r#"<svg><path transform="translate(10-20) scale(2,.5)" d="M10 20 L20 20"/></svg>"#,
        8,
        [0.0, 0.0, 100.0, 100.0],
    )
    .expect("SVG numbers may use sign and comma separators without whitespace");
    assert_point(compact[0].path.contours[0].points[0], [-0.2, 0.6]);
    assert_point(compact[0].path.contours[0].points[1], [0.0, 0.6]);
}

#[test]
fn unsupported_or_malformed_svg_transforms_are_svg_diagnostics() {
    let skewed = parse_svg_document_vector_records_with_xml_options(
        r#"<svg><path transform="skewX(45) skewY(45)" d="M0 0 L10 0 L0 10 Z"/></svg>"#,
        8,
        [0.0, 0.0, 100.0, 100.0],
        XmlOptions::default(),
    )
    .expect("skew transforms should lower through the SVG state stack");
    assert_eq!(skewed.len(), 1);
    assert_eq!(skewed[0].path.contours[0].points[1], [-0.3, 0.4]);

    for transform in ["unsupported(10)", "translate(nope)", "translate(10"] {
        let diagnostic = parse_svg_document_vector_records_with_xml_options(
            &format!(r#"<svg><path transform="{transform}" d="M0 0 L24 0"/></svg>"#),
            8,
            [0.0, 0.0, 24.0, 24.0],
            XmlOptions::default(),
        )
        .expect_err("unsupported SVG transform syntax must remain visible");
        assert_eq!(diagnostic.stage, SvgImportStage::Svg);
        assert!(diagnostic.xml.is_none());
        assert!(diagnostic.span.is_some());
    }
}

#[test]
fn svg_viewport_policy_distinguishes_caller_bounds_from_root_view_box() {
    let source = r#"<svg viewBox="10 20 20 10" width="200" height="100">
            <line x1="10" y1="20" x2="30" y2="30"/>
        </svg>"#;
    let document = parse_svg_document_vector_records_with_viewport(
        source,
        8,
        SvgViewportSource::DocumentViewBox,
        XmlOptions::default(),
    )
    .expect("the document viewBox should provide coordinate normalization");
    assert_eq!(document[0].path.contours[0].points[0], [-0.5, 0.25]);
    assert_eq!(document[0].path.contours[0].points[1], [0.5, -0.25]);

    let caller = parse_svg_document_vector_records_with_viewport(
        source,
        8,
        SvgViewportSource::Caller([0.0, 0.0, 40.0, 40.0]),
        XmlOptions::default(),
    )
    .expect("the caller viewport must remain an explicit alternate path");
    assert_eq!(caller[0].path.contours[0].points[0], [-0.25, 0.0]);
    assert_eq!(caller[0].path.contours[0].points[1], [0.25, -0.25]);

    let missing = parse_svg_document_vector_records_with_viewport(
        r#"<svg><line x1="0" y1="0" x2="1" y2="1"/></svg>"#,
        8,
        SvgViewportSource::DocumentViewBox,
        XmlOptions::default(),
    )
    .expect_err("document viewBox mode must not invent a coordinate model");
    assert_eq!(missing.stage, SvgImportStage::Svg);
    assert!(missing.span.is_some());
}

#[test]
fn document_view_box_meet_alignment_is_explicit_and_slice_is_rejected() {
    let source = r#"<svg viewBox="0 0 20 10" preserveAspectRatio="xMinYMin">
            <line x1="0" y1="0" x2="20" y2="10"/>
        </svg>"#;
    let aligned = parse_svg_document_vector_records_with_viewport(
        source,
        8,
        SvgViewportSource::DocumentViewBox,
        XmlOptions::default(),
    )
    .expect("meet alignment should lower");
    assert_eq!(aligned[0].path.contours[0].points[0], [-0.5, 0.5]);
    assert_eq!(aligned[0].path.contours[0].points[1], [0.5, 0.0]);

    let none = parse_svg_document_vector_records_with_viewport(
        r#"<svg viewBox="0 0 20 10" preserveAspectRatio="none">
                <line x1="0" y1="0" x2="20" y2="10"/>
            </svg>"#,
        8,
        SvgViewportSource::DocumentViewBox,
        XmlOptions::default(),
    )
    .expect("none should preserve non-uniform mapping");
    assert_eq!(none[0].path.contours[0].points[0], [-0.5, 0.5]);
    assert_eq!(none[0].path.contours[0].points[1], [0.5, -0.5]);

    let error = parse_svg_document_vector_records_with_viewport(
        r#"<svg viewBox="0 0 20 10" preserveAspectRatio="xMidYMid slice">
                <line x1="0" y1="0" x2="20" y2="10"/>
            </svg>"#,
        8,
        SvgViewportSource::DocumentViewBox,
        XmlOptions::default(),
    )
    .expect_err("slice mode must remain an explicit profile gap");
    assert!(error.message.contains("slice"));
}

#[test]
fn svg_records_preserve_source_and_transformed_bounds_before_normalization() {
    let source = r#"<svg viewBox="0 0 20 10">
            <g transform="translate(2 3)">
                <rect x="1" y="2" width="4" height="2"/>
            </g>
        </svg>"#;
    let records = parse_svg_document_vector_records_with_viewport(
        source,
        8,
        SvgViewportSource::DocumentViewBox,
        XmlOptions::default(),
    )
    .expect("transformed rectangle should lower");
    assert_eq!(records.len(), 1);
    assert_eq!(records[0].source_bounds, Some(([1.0, 2.0], [5.0, 4.0])));
    assert_eq!(
        records[0].transformed_bounds,
        Some(([3.0, 5.0], [7.0, 7.0]))
    );
    let normalized_bounds = records[0].path.bounds().expect("normalized bounds");
    assert!((normalized_bounds.0[0] + 0.35).abs() < 1.0e-6);
    assert!((normalized_bounds.0[1] + 0.1).abs() < 1.0e-6);
    assert!((normalized_bounds.1[0] + 0.15).abs() < 1.0e-6);
    assert!(normalized_bounds.1[1].abs() < 1.0e-6);
}

#[test]
fn svg_lowering_reuses_an_existing_parser_neutral_event_stream() {
    let source = r#"<svg><path d="M0 0 L24 0 L24 24 Z"/></svg>"#;
    let events = parse_xml_events(XmlSourceId::new(77), source, XmlOptions::default())
        .expect("fixture XML should parse once before SVG lowering");
    let from_events = parse_svg_document_vector_records_from_xml_events(
        &events,
        8,
        SvgViewportSource::Caller([0.0, 0.0, 24.0, 24.0]),
    )
    .expect("SVG lowering should consume existing parser-neutral events");
    let from_source = parse_svg_document_vector_records(source, 8, [0.0, 0.0, 24.0, 24.0])
        .expect("source convenience API should use the same semantic lowering");

    assert_eq!(from_events.len(), from_source.len());
    assert_eq!(from_events[0].path, from_source[0].path);
    assert_eq!(from_events[0].fill, from_source[0].fill);
    assert_eq!(from_events[0].stroke, from_source[0].stroke);
    assert_eq!(from_events[0].fill_rule, from_source[0].fill_rule);
    assert_eq!(from_events[0].source_span.source.value(), 77);
    assert_eq!(from_source[0].source_span.source.value(), 0);
}

#[test]
fn convex_fill_adapter_routes_supported_svg_geometry() {
    let svg = r#"<svg><rect x="0" y="0" width="12" height="12" /></svg>"#;
    let meshes = parse_svg_document_convex_fill_meshes(svg, 8, [0.0, 0.0, 12.0, 12.0])
        .expect("rectangle should use the shared convex fill tessellator");

    assert_eq!(meshes.len(), 1);
    assert_eq!(meshes[0].len(), 6);
}

#[test]
fn convex_fill_adapter_reports_unsupported_svg_topology() {
    let svg = r#"<svg><path d="M 0 0 L 12 0 L 6 3 L 0 12 Z" /></svg>"#;
    let error = parse_svg_document_convex_fill_meshes(svg, 8, [0.0, 0.0, 12.0, 12.0])
        .expect_err("concave fill should be diagnosed");

    assert!(error.contains("SVG fill path 0 is unsupported"));
}
