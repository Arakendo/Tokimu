use std::path::PathBuf;

use cgm_corpus::{
    inspect_binary_cgm_file, lower_picture_primitives, CgmAttributeValue, CgmClipIndicator,
    CgmColor, CgmColorSelectionMode, CgmPolygonSetEdgeFlag, CgmPrimitiveKind, CgmPrimitiveTopology,
    CgmScalingMode, CgmVdcExtent, DecodeLimits, DelimiterElement, ElementSupport,
};

const SELECTED_CASES: &[&str] = &[
    "ALLELM01.cgm",
    "VDCEXT01.cgm",
    "POLYLN01.cgm",
    "POLYGN01.cgm",
    "RCTNGL01.cgm",
    "CIRCLE01.cgm",
    "ELLIPS01.cgm",
    "CIRARC01.cgm",
    "ELLARC01.cgm",
    "PLGSET01.cgm",
    "INTSTL01.cgm",
    "LINCAP01.cgm",
    "LNJOIN01.cgm",
    "CLIPNG01.cgm",
    "COLRMD01.cgm",
];

fn fixture(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../../third-party/fixtures/webcgm-test-suite/upstream/static10")
        .join(name)
}

#[test]
fn selected_polyline_fixture_has_stable_binary_lifecycle() {
    let inspection = inspect_binary_cgm_file(fixture("POLYLN01.cgm"), DecodeLimits::default())
        .expect("selected POLYLN01 fixture should inspect");

    assert_eq!(inspection.metafile_name, "POLYLN01");
    assert_eq!(inspection.pictures.len(), 1);
    assert_eq!(inspection.pictures[0].name, "picture 1");
    assert!(inspection.elements.len() > 10);
    assert_eq!(inspection.trailing_padding_bytes, 2);
    assert_eq!(
        inspection
            .elements
            .first()
            .and_then(|element| element.delimiter),
        Some(DelimiterElement::BeginMetafile)
    );
    assert_eq!(
        inspection
            .elements
            .last()
            .and_then(|element| element.delimiter),
        Some(DelimiterElement::EndMetafile)
    );
    assert!(
        !inspection.diagnostics.is_empty(),
        "non-lifecycle elements should remain visible as unsupported diagnostics"
    );

    for pair in inspection.elements.windows(2) {
        assert_eq!(
            pair[0].source_offset + pair[0].encoded_length,
            pair[1].source_offset,
            "element offsets should account for headers, partitions, and padding"
        );
    }
}

#[test]
fn every_selected_binary_fixture_reaches_a_complete_lifecycle() {
    for name in SELECTED_CASES {
        let inspection = inspect_binary_cgm_file(fixture(name), DecodeLimits::default())
            .unwrap_or_else(|error| panic!("{name} should inspect: {error}"));

        assert!(
            !inspection.metafile_name.is_empty(),
            "{name} should name its metafile"
        );
        assert!(
            !inspection.pictures.is_empty(),
            "{name} should contain at least one picture"
        );
        assert_eq!(
            inspection
                .elements
                .first()
                .and_then(|element| element.delimiter),
            Some(DelimiterElement::BeginMetafile),
            "{name} should start with BEGIN METAFILE"
        );
        assert_eq!(
            inspection
                .elements
                .last()
                .and_then(|element| element.delimiter),
            Some(DelimiterElement::EndMetafile),
            "{name} should end with END METAFILE"
        );
        assert!(
            inspection.trailing_padding_bytes <= 2,
            "{name} should contain at most two bytes of record padding"
        );
    }
}

#[test]
fn broad_inventory_fixture_decodes_without_losing_offsets() {
    let inspection = inspect_binary_cgm_file(fixture("ALLELM01.cgm"), DecodeLimits::default())
        .expect("selected ALLELM01 fixture should inspect");

    assert_eq!(inspection.metafile_name, "ALLELM01");
    assert_eq!(inspection.pictures.len(), 1);
    let identities = inspection
        .elements
        .iter()
        .map(|element| (element.class, element.id))
        .collect::<std::collections::BTreeSet<_>>();
    assert!(
        identities.len() >= 40,
        "ALLELM01 should expose a broad element inventory, found {} identities",
        identities.len()
    );
    assert!(inspection
        .elements
        .iter()
        .all(|element| element.encoded_length >= 2));
}

#[test]
fn selected_extent_fixture_exposes_picture_local_coordinate_state() {
    let inspection = inspect_binary_cgm_file(fixture("VDCEXT01.cgm"), DecodeLimits::default())
        .expect("selected VDC extent fixture should inspect");
    let descriptor = &inspection.pictures[0].descriptor;

    assert_eq!(descriptor.scaling_mode, CgmScalingMode::Metric);
    assert_eq!(
        descriptor.color_selection_mode,
        CgmColorSelectionMode::Indexed
    );
    assert_eq!(
        descriptor.vdc_extent,
        Some(CgmVdcExtent {
            first: [0, 1000],
            second: [1000, 0],
        })
    );
}

#[test]
fn selected_attribute_fixtures_preserve_presentation_mutations() {
    let polyline = inspect_binary_cgm_file(fixture("POLYLN01.cgm"), DecodeLimits::default())
        .expect("selected polyline fixture should inspect");
    let attributes = &polyline.pictures[0].attributes;
    assert!(attributes
        .iter()
        .any(|attribute| matches!(attribute.value, CgmAttributeValue::LineWidth { .. })));
    assert!(polyline
        .elements
        .iter()
        .any(|element| element.support == ElementSupport::Attribute));

    let polygon = inspect_binary_cgm_file(fixture("INTSTL01.cgm"), DecodeLimits::default())
        .expect("selected interior-style fixture should inspect");
    let attributes = &polygon.pictures[0].attributes;
    assert!(attributes
        .iter()
        .any(|attribute| matches!(attribute.value, CgmAttributeValue::InteriorStyle { .. })));
    assert!(attributes
        .iter()
        .any(|attribute| matches!(attribute.value, CgmAttributeValue::FillColor { .. })));
    assert!(attributes
        .iter()
        .any(|attribute| matches!(attribute.value, CgmAttributeValue::EdgeVisibility { .. })));

    let color_modes = inspect_binary_cgm_file(fixture("COLRMD01.cgm"), DecodeLimits::default())
        .expect("selected colour-mode fixture should inspect");
    assert!(color_modes
        .pictures
        .iter()
        .flat_map(|picture| &picture.attributes)
        .any(|attribute| matches!(
            attribute.value,
            CgmAttributeValue::LineColor {
                color: CgmColor::Direct(_)
            } | CgmAttributeValue::FillColor {
                color: CgmColor::Direct(_)
            }
        )));
}

#[test]
fn selected_primitives_capture_their_explicit_cgm_presentation_state() {
    let line_caps = inspect_binary_cgm_file(fixture("LINCAP01.cgm"), DecodeLimits::default())
        .expect("selected line-cap fixture should inspect");
    let capped_primitives = line_caps
        .pictures
        .iter()
        .flat_map(|picture| &picture.primitives)
        .filter(|primitive| primitive.state.line_cap.is_some())
        .count();
    assert!(
        capped_primitives > 0,
        "LINCAP01 should attach its explicit cap state to later primitives"
    );

    let interior = inspect_binary_cgm_file(fixture("INTSTL01.cgm"), DecodeLimits::default())
        .expect("selected interior-style fixture should inspect");
    assert!(interior
        .pictures
        .iter()
        .flat_map(|picture| &picture.primitives)
        .any(|primitive| {
            primitive.state.interior_style.is_some()
                || primitive.state.fill_color.is_some()
                || primitive.state.edge_visible.is_some()
        }));
}

#[test]
fn polygon_set_records_and_clipping_controls_remain_explicit_source_state() {
    let polygon_set = inspect_binary_cgm_file(fixture("PLGSET01.cgm"), DecodeLimits::default())
        .expect("selected polygon-set fixture should inspect");
    let records = polygon_set.pictures[0]
        .primitives
        .iter()
        .find_map(|primitive| match &primitive.kind {
            CgmPrimitiveKind::PolygonSet { records } => Some(records),
            _ => None,
        })
        .expect("PLGSET01 should preserve point/flag records");
    assert_eq!(records.len(), 6);
    assert_eq!(records[0].point, [200, 400]);
    assert_eq!(records[2].point, [800, 400]);
    assert_eq!(
        records.iter().map(|record| record.edge).collect::<Vec<_>>(),
        vec![
            CgmPolygonSetEdgeFlag::Visible,
            CgmPolygonSetEdgeFlag::Invisible,
            CgmPolygonSetEdgeFlag::CloseVisible,
            CgmPolygonSetEdgeFlag::Visible,
            CgmPolygonSetEdgeFlag::Visible,
            CgmPolygonSetEdgeFlag::CloseInvisible,
        ],
        "PLGSET01 preserves the outer boundary and triangular cut-out edge semantics"
    );
    let error = lower_picture_primitives(&polygon_set.pictures[0])
        .expect_err("polygon-set topology must not lower as an ordinary polygon");
    assert!(matches!(
        error,
        cgm_corpus::CgmError::UnsupportedPrimitiveLowering {
            kind: "polygon-set point/flag topology",
            ..
        }
    ));

    let clipping = inspect_binary_cgm_file(fixture("CLIPNG01.cgm"), DecodeLimits::default())
        .expect("selected clipping fixture should inspect");
    assert!(clipping.elements.iter().any(|element| {
        element.class == 3 && element.id == 5 && element.support == ElementSupport::Control
    }));
    assert!(clipping.elements.iter().any(|element| {
        element.class == 3 && element.id == 6 && element.support == ElementSupport::Control
    }));
    let picture = clipping
        .pictures
        .first()
        .expect("CLIPNG01 should contain a picture");
    assert_eq!(
        picture.controls.clip_rectangle,
        Some(CgmVdcExtent {
            first: [0, 0],
            second: [1000, 1000],
        })
    );
    assert_eq!(picture.controls.clip_indicator, Some(CgmClipIndicator::Off));
    assert!(picture.primitives.iter().all(|primitive| {
        primitive.controls.clip_rectangle == picture.controls.clip_rectangle
            && primitive.controls.clip_indicator == picture.controls.clip_indicator
    }));
}

#[test]
fn selected_polyline_and_polygon_preserve_source_points_before_vector_lowering() {
    let polyline = inspect_binary_cgm_file(fixture("POLYLN01.cgm"), DecodeLimits::default())
        .expect("selected polyline fixture should inspect");
    let lines = polyline.pictures[0]
        .primitives
        .iter()
        .filter_map(|primitive| match &primitive.kind {
            CgmPrimitiveKind::Polyline { points } => Some((primitive, points)),
            _ => None,
        })
        .collect::<Vec<_>>();
    assert!(!lines.is_empty(), "POLYLN01 should preserve polylines");
    assert!(lines.iter().all(|(_, points)| !points.is_empty()));
    assert!(polyline
        .elements
        .iter()
        .any(|element| element.support == ElementSupport::Primitive));

    let polygon = inspect_binary_cgm_file(fixture("POLYGN01.cgm"), DecodeLimits::default())
        .expect("selected polygon fixture should inspect");
    let polygons = polygon.pictures[0]
        .primitives
        .iter()
        .filter_map(|primitive| match &primitive.kind {
            CgmPrimitiveKind::Polygon { points } => Some(points),
            _ => None,
        })
        .collect::<Vec<_>>();
    assert!(!polygons.is_empty(), "POLYGN01 should preserve polygons");
    assert!(polygons.iter().all(|points| points.len() >= 3));
}

#[test]
fn selected_primitives_lower_through_the_shared_vector_contract() {
    let polyline = inspect_binary_cgm_file(fixture("POLYLN01.cgm"), DecodeLimits::default())
        .expect("selected polyline fixture should inspect");
    let paths = lower_picture_primitives(&polyline.pictures[0])
        .expect("POLYLN01 straight primitives should lower");
    assert!(!paths.is_empty());
    assert!(paths.iter().all(|primitive| primitive.path.is_finite()));
    assert!(paths
        .iter()
        .any(|primitive| primitive.topology == CgmPrimitiveTopology::Open));

    let polygon = inspect_binary_cgm_file(fixture("POLYGN01.cgm"), DecodeLimits::default())
        .expect("selected polygon fixture should inspect");
    let paths =
        lower_picture_primitives(&polygon.pictures[0]).expect("POLYGN01 primitives should lower");
    assert!(paths.iter().any(|primitive| {
        primitive.topology == CgmPrimitiveTopology::Closed
            && primitive.path.contours[0].points.len() >= 3
            && primitive.path.is_finite()
    }));

    let rectangle = inspect_binary_cgm_file(fixture("RCTNGL01.cgm"), DecodeLimits::default())
        .expect("selected rectangle fixture should inspect");
    let paths = lower_picture_primitives(&rectangle.pictures[0])
        .expect("RCTNGL01 straight primitives should lower");
    assert!(paths
        .iter()
        .any(|primitive| primitive.topology == CgmPrimitiveTopology::Closed));

    let circle = inspect_binary_cgm_file(fixture("CIRCLE01.cgm"), DecodeLimits::default())
        .expect("selected circle fixture should inspect");
    let paths = lower_picture_primitives(&circle.pictures[0])
        .expect("CIRCLE01 should lower through deterministic flattening");
    assert!(paths.iter().any(|primitive| {
        primitive.topology == CgmPrimitiveTopology::Closed
            && primitive.path.contours[0].points.len() == 32
    }));

    let ellipse = inspect_binary_cgm_file(fixture("ELLIPS01.cgm"), DecodeLimits::default())
        .expect("selected ellipse fixture should inspect");
    let paths = lower_picture_primitives(&ellipse.pictures[0])
        .expect("ELLIPS01 should lower through conjugate-diameter flattening");
    assert!(paths.iter().any(|primitive| {
        primitive.topology == CgmPrimitiveTopology::Closed
            && primitive.path.contours[0].points.len() == 32
            && primitive.path.is_finite()
    }));

    for name in ["CIRARC01.cgm", "ELLARC01.cgm"] {
        let inspection = inspect_binary_cgm_file(fixture(name), DecodeLimits::default())
            .unwrap_or_else(|error| panic!("{name} should inspect: {error}"));
        let paths = lower_picture_primitives(&inspection.pictures[0])
            .unwrap_or_else(|error| panic!("{name} should lower: {error}"));
        assert!(paths.iter().any(|primitive| {
            primitive.topology == CgmPrimitiveTopology::Open
                && primitive.path.contours[0].points.len() == 33
                && primitive.path.is_finite()
        }));
    }
}

#[test]
fn selected_state_fixtures_lower_without_claiming_paint_or_clip_execution() {
    for name in [
        "INTSTL01.cgm",
        "LINCAP01.cgm",
        "LNJOIN01.cgm",
        "CLIPNG01.cgm",
        "COLRMD01.cgm",
    ] {
        let inspection = inspect_binary_cgm_file(fixture(name), DecodeLimits::default())
            .unwrap_or_else(|error| panic!("{name} should inspect: {error}"));
        let picture = inspection
            .pictures
            .first()
            .unwrap_or_else(|| panic!("{name} should contain a picture"));
        let primitives = lower_picture_primitives(picture)
            .unwrap_or_else(|error| panic!("{name} should lower its admitted primitives: {error}"));
        assert!(
            !primitives.is_empty(),
            "{name} should lower at least one source primitive"
        );
        assert!(
            primitives
                .iter()
                .all(|primitive| primitive.path.is_finite()),
            "{name} should not produce non-finite vector geometry"
        );
    }
}

#[test]
fn selected_vector_lowering_is_repeatable_and_has_finite_bounds() {
    for name in [
        "POLYLN01.cgm",
        "POLYGN01.cgm",
        "RCTNGL01.cgm",
        "CIRCLE01.cgm",
        "ELLIPS01.cgm",
        "CIRARC01.cgm",
        "ELLARC01.cgm",
    ] {
        let inspection = inspect_binary_cgm_file(fixture(name), DecodeLimits::default())
            .unwrap_or_else(|error| panic!("{name} should inspect: {error}"));
        let first = lower_picture_primitives(&inspection.pictures[0])
            .unwrap_or_else(|error| panic!("{name} should lower: {error}"));
        let second = lower_picture_primitives(&inspection.pictures[0])
            .unwrap_or_else(|error| panic!("{name} should lower repeatedly: {error}"));

        assert_eq!(first, second, "{name} lowering should be deterministic");
        assert!(
            first.iter().all(|primitive| {
                primitive.path.is_finite() && primitive.path.bounds().is_some()
            }),
            "{name} paths should retain finite structural bounds"
        );
    }
}

#[test]
fn selected_closed_primitives_preserve_source_parameters_before_vector_lowering() {
    let rectangle = inspect_binary_cgm_file(fixture("RCTNGL01.cgm"), DecodeLimits::default())
        .expect("selected rectangle fixture should inspect");
    assert!(rectangle.pictures[0]
        .primitives
        .iter()
        .any(|primitive| matches!(primitive.kind, CgmPrimitiveKind::Rectangle { .. })));

    let circle = inspect_binary_cgm_file(fixture("CIRCLE01.cgm"), DecodeLimits::default())
        .expect("selected circle fixture should inspect");
    assert!(circle.pictures[0]
        .primitives
        .iter()
        .any(|primitive| matches!(
            primitive.kind,
            CgmPrimitiveKind::Circle { radius, .. } if radius > 0
        )));

    let ellipse = inspect_binary_cgm_file(fixture("ELLIPS01.cgm"), DecodeLimits::default())
        .expect("selected ellipse fixture should inspect");
    assert!(ellipse.pictures[0]
        .primitives
        .iter()
        .any(|primitive| matches!(primitive.kind, CgmPrimitiveKind::Ellipse { .. })));

    let circular_arc = inspect_binary_cgm_file(fixture("CIRARC01.cgm"), DecodeLimits::default())
        .expect("selected circular arc fixture should inspect");
    assert!(circular_arc.pictures[0]
        .primitives
        .iter()
        .any(|primitive| matches!(
            primitive.kind,
            CgmPrimitiveKind::CircularArc { radius, .. } if radius > 0
        )));

    let elliptical_arc = inspect_binary_cgm_file(fixture("ELLARC01.cgm"), DecodeLimits::default())
        .expect("selected elliptical arc fixture should inspect");
    assert!(elliptical_arc.pictures[0]
        .primitives
        .iter()
        .any(|primitive| matches!(primitive.kind, CgmPrimitiveKind::EllipticalArc { .. })));
}

#[test]
fn circular_arc_preserves_counter_clockwise_source_endpoints() {
    let inspection = inspect_binary_cgm_file(fixture("CIRARC01.cgm"), DecodeLimits::default())
        .expect("selected circular arc fixture should inspect");
    let picture = &inspection.pictures[0];
    let source = picture
        .primitives
        .iter()
        .find(|primitive| matches!(primitive.kind, CgmPrimitiveKind::CircularArc { .. }))
        .expect("fixture should contain a circular arc");
    let lowered = cgm_corpus::lower_primitive(
        source,
        picture.descriptor.vdc_extent.expect("fixture VDC extent"),
    )
    .expect("circular arc should lower");
    let points = &lowered.path.contours[0].points;

    assert_eq!(lowered.topology, CgmPrimitiveTopology::Open);
    assert_eq!(points.len(), 33);
    // The first CIRARC01 arc has center=(125,800), start=(100,0),
    // end=(0,100), and VDC extent=(0,0)..(1000,1000). The adapted path
    // therefore starts at (225,800) and ends at (125,900), preserving the
    // source counter-clockwise sweep in the fixture's VDC coordinate order.
    assert_close(points[0], [0.225, 0.8]);
    assert_close(points[16], [0.195_710_69, 0.870_710_7]);
    assert_close(*points.last().expect("arc endpoint"), [0.125, 0.9]);
}

#[test]
fn elliptical_arc_preserves_source_derived_endpoints_and_bounds() {
    let inspection = inspect_binary_cgm_file(fixture("ELLARC01.cgm"), DecodeLimits::default())
        .expect("selected elliptical arc fixture should inspect");
    let picture = &inspection.pictures[0];
    let extent = picture.descriptor.vdc_extent.expect("fixture VDC extent");
    let source = picture
        .primitives
        .iter()
        .find(|primitive| matches!(primitive.kind, CgmPrimitiveKind::EllipticalArc { .. }))
        .expect("fixture should contain an elliptical arc");
    let (center, start_vector, end_vector) = match source.kind {
        CgmPrimitiveKind::EllipticalArc {
            center,
            start_vector,
            end_vector,
            ..
        } => (center, start_vector, end_vector),
        _ => unreachable!("selected source is an elliptical arc"),
    };
    let lowered = cgm_corpus::lower_primitive(source, extent).expect("elliptical arc should lower");
    let points = &lowered.path.contours[0].points;
    let expected_start = extent
        .normalize([center[0] + start_vector[0], center[1] + start_vector[1]])
        .expect("source start point should normalize");
    let expected_end = extent
        .normalize([center[0] + end_vector[0], center[1] + end_vector[1]])
        .expect("source end point should normalize");

    assert_eq!(lowered.topology, CgmPrimitiveTopology::Open);
    assert_eq!(points.len(), 33);
    assert_close(points[0], expected_start);
    assert_close(*points.last().expect("arc endpoint"), expected_end);
    let (minimum, maximum) = lowered.path.bounds().expect("arc should have bounds");
    for endpoint in [expected_start, expected_end] {
        assert!(
            endpoint[0] >= minimum[0]
                && endpoint[0] <= maximum[0]
                && endpoint[1] >= minimum[1]
                && endpoint[1] <= maximum[1],
            "endpoint {endpoint:?} must remain inside the lowered arc bounds"
        );
    }
}

fn assert_close(actual: [f32; 2], expected: [f32; 2]) {
    for index in 0..2 {
        assert!(
            (actual[index] - expected[index]).abs() <= 0.000_001,
            "coordinate {index}: expected {}, got {}",
            expected[index],
            actual[index]
        );
    }
}
