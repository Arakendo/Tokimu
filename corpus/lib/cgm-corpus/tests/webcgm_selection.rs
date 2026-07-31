use std::path::PathBuf;

use cgm_corpus::{
    cgm_element_name, inspect_binary_cgm_file, lower_picture_primitives, summarize_diagnostics,
    CgmAttributeValue, CgmClipIndicator, CgmColor, CgmColorSelectionMode, CgmColorValueExtent,
    CgmEdgeIntent, CgmFillIntent, CgmInteriorStyle, CgmPolygonSetEdgeFlag, CgmPrimitiveKind,
    CgmPrimitiveTopology, CgmScalingMode, CgmStrokeIntent, CgmTextRecordKind, CgmVdcExtent,
    DecodeLimits, DelimiterElement, ElementSupport,
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
    "CLIPNG02.cgm",
    "APNTXT01.cgm",
    "CHRHGT01.cgm",
    "CHRORI01.cgm",
    "TXTALN01.cgm",
    "CHRSPA01.cgm",
    "TXTPTH01.cgm",
    "CELARY01.cgm",
    "COLRMD01.cgm",
    "COLVAL01.cgm",
    "POLYBZ01.cgm",
    "POLYBZ04.cgm",
];

#[test]
fn deferred_cgm_element_names_remain_provider_owned_and_conservative() {
    assert_eq!(cgm_element_name(4, 5), "CGM text primitive");
    assert_eq!(cgm_element_name(4, 6), "CGM append text primitive");
    assert_eq!(cgm_element_name(4, 9), "CGM cell array raster primitive");
    assert_eq!(cgm_element_name(4, 26), "CGM polybezier primitive");
    assert_eq!(cgm_element_name(9, 99), "CGM class 9 element 99");
}

#[test]
fn selected_polybezier_fixture_preserves_cgm_continuity_and_control_points() {
    let inspection = inspect_binary_cgm_file(fixture("POLYBZ01.cgm"), DecodeLimits::default())
        .expect("selected POLYBZ01 fixture should inspect");
    let picture = inspection
        .pictures
        .first()
        .expect("POLYBZ01 should contain one picture");
    let records = picture
        .primitives
        .iter()
        .filter_map(|primitive| match &primitive.kind {
            CgmPrimitiveKind::PolyBezier { continuity, points } => Some((*continuity, points)),
            _ => None,
        })
        .collect::<Vec<_>>();

    assert_eq!(
        records.len(),
        8,
        "POLYBZ01 contains eight polybezier records"
    );
    assert!(records.iter().all(|(_, points)| !points.is_empty()));
    assert!(records.iter().all(|(_, points)| points.len() >= 2));
    assert!(inspection
        .elements
        .iter()
        .filter(|element| element.class == 4 && element.id == 26)
        .all(|element| element.support == ElementSupport::Primitive));
}

#[test]
fn selected_polybezier_comparison_fixture_preserves_multiple_source_records() {
    let inspection = inspect_binary_cgm_file(fixture("POLYBZ04.cgm"), DecodeLimits::default())
        .expect("selected POLYBZ04 fixture should inspect");
    let picture = inspection
        .pictures
        .first()
        .expect("POLYBZ04 should contain one picture");
    let records = picture
        .primitives
        .iter()
        .filter_map(|primitive| match &primitive.kind {
            CgmPrimitiveKind::PolyBezier { continuity, points } => Some((*continuity, points)),
            _ => None,
        })
        .collect::<Vec<_>>();

    assert!(
        records.len() > 1,
        "POLYBZ04 should independently exercise multiple polybezier records"
    );
    assert!(records.iter().all(|(_, points)| points.len() >= 2));
    assert!(inspection
        .elements
        .iter()
        .filter(|element| element.class == 4 && element.id == 26)
        .all(|element| element.support == ElementSupport::Primitive));
}

#[test]
fn deferred_feature_summary_excludes_preserved_cell_array_source_records() {
    let inspection = inspect_binary_cgm_file(fixture("CELARY01.cgm"), DecodeLimits::default())
        .expect("selected CELARY01 fixture should inspect");
    let summaries = summarize_diagnostics(&inspection.diagnostics);

    assert_eq!(
        summaries.iter().map(|feature| feature.count).sum::<usize>(),
        inspection.diagnostics.len()
    );
    assert!(!summaries
        .iter()
        .any(|feature| feature.class == 4 && feature.id == 9));
}

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
    assert!(attributes.iter().any(|attribute| {
        matches!(
            attribute.value,
            CgmAttributeValue::InteriorStyle {
                style: CgmInteriorStyle::Solid
            }
        )
    }));
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
fn selected_color_value_extent_preserves_direct_component_range() {
    let inspection = inspect_binary_cgm_file(fixture("COLVAL01.cgm"), DecodeLimits::default())
        .expect("selected color-value-extent fixture should inspect");

    assert_eq!(
        inspection.metafile.color_value_extent,
        Some(CgmColorValueExtent {
            minimum: [0, 0, 0],
            maximum: [100, 100, 100],
        }),
        "COLVAL01 must preserve its source component range rather than imply 0..255"
    );
}

#[test]
fn selected_interior_style_fixture_resolves_only_its_explicit_palette_entries() {
    let inspection = inspect_binary_cgm_file(fixture("INTSTL01.cgm"), DecodeLimits::default())
        .expect("selected interior-style fixture should inspect");
    let primitive = inspection.pictures[0]
        .primitives
        .iter()
        .find(|primitive| primitive.state.fill_color == Some(CgmColor::Indexed(vec![1])))
        .expect("INTSTL01 should carry an indexed fill using its explicit palette");

    assert_eq!(primitive.state.color_table.get(&1), Some(&[0, 0, 0]));
    assert_eq!(
        primitive.state.normalize_explicit_color(
            &inspection.metafile,
            primitive
                .state
                .fill_color
                .as_ref()
                .expect("selected primitive must carry its fill color"),
        ),
        Some([0.0, 0.0, 0.0]),
        "indexed color resolution must use the picture-local explicit palette"
    );
    assert!(
        primitive
            .state
            .normalize_explicit_color(&inspection.metafile, &CgmColor::Indexed(vec![200]))
            .is_none(),
        "missing palette entries must remain unresolved rather than fall back"
    );
}

#[test]
fn selected_append_text_fixture_preserves_text_source_records_without_rendering_them() {
    let inspection = inspect_binary_cgm_file(fixture("APNTXT01.cgm"), DecodeLimits::default())
        .expect("selected append-text fixture should inspect");
    assert!(inspection.elements.iter().any(|element| {
        element.class == 4 && element.id == 6 && element.support == ElementSupport::Text
    }));
    assert!(!inspection
        .diagnostics
        .iter()
        .any(|diagnostic| diagnostic.class == 4 && matches!(diagnostic.id, 5 | 6)));

    let records = &inspection.pictures[0].text_records;
    assert_eq!(
        records.len(),
        12,
        "fixture should retain all text source records"
    );
    assert!(records
        .iter()
        .all(|record| record.source_offset < inspection.source_bytes));
    assert!(records
        .iter()
        .all(|record| record.attribute_count <= inspection.pictures[0].attributes.len()));
    assert!(matches!(
        records.first().map(|record| &record.kind),
        Some(CgmTextRecordKind::Restricted { text, .. }) if text == "Restricted text"
    ));
    assert!(matches!(
        records.iter().find(|record| matches!(record.kind, CgmTextRecordKind::Append { .. })).map(|record| &record.kind),
        Some(CgmTextRecordKind::Append { text, .. }) if text == " gjhi"
    ));
    assert!(
        inspection.pictures[0]
            .primitives
            .iter()
            .all(|primitive| primitive.source_offset < inspection.source_bytes),
        "text preservation must not corrupt surrounding geometry source records"
    );
}

#[test]
fn selected_character_height_fixture_preserves_text_state_without_layout() {
    let inspection = inspect_binary_cgm_file(fixture("CHRHGT01.cgm"), DecodeLimits::default())
        .expect("selected character-height fixture should inspect");
    let picture = &inspection.pictures[0];

    let heights = picture
        .attributes
        .iter()
        .filter_map(|attribute| match &attribute.value {
            CgmAttributeValue::CharacterHeight { value } => Some(*value),
            _ => None,
        })
        .collect::<Vec<_>>();
    assert_eq!(heights, vec![100, 50, 25, 10, 19, 10]);
    assert!(picture.attributes.iter().any(|attribute| matches!(
        &attribute.value,
        CgmAttributeValue::CharacterOrientation {
            up: [0, 1],
            base: [1, 0],
        }
    )));
    assert!(picture.attributes.iter().any(|attribute| matches!(
        &attribute.value,
        CgmAttributeValue::TextAlignment {
            horizontal: 3,
            vertical: 0,
            continuous_horizontal: [0, 0, 0, 0],
            continuous_vertical: [0, 0, 0, 0],
        }
    )));
    assert!(picture.text_records.iter().any(|record| {
        record.state.character_orientation.is_some() && record.state.text_alignment.is_some()
    }));
    assert!(picture.text_records.iter().all(|record| {
        record.state.character_height.is_some()
            || record.state.character_orientation.is_none() && record.state.text_alignment.is_none()
    }));
    assert!(!inspection.elements.iter().any(|element| {
        element.class == 5
            && matches!(element.id, 15 | 16 | 18)
            && element.support == ElementSupport::Unsupported
    }));
}

#[test]
fn selected_character_orientation_fixture_preserves_source_state_without_layout() {
    let inspection = inspect_binary_cgm_file(fixture("CHRORI01.cgm"), DecodeLimits::default())
        .expect("selected character-orientation fixture should inspect");
    let picture = &inspection.pictures[0];

    assert!(picture.attributes.iter().any(|attribute| matches!(
        &attribute.value,
        CgmAttributeValue::CharacterOrientation { .. }
    )));
    assert!(picture
        .text_records
        .iter()
        .any(|record| record.state.character_orientation.is_some()));
    assert!(inspection.elements.iter().any(|element| {
        element.class == 5 && element.id == 16 && element.support == ElementSupport::Attribute
    }));
}

#[test]
fn selected_text_alignment_fixture_preserves_source_state_without_layout() {
    let inspection = inspect_binary_cgm_file(fixture("TXTALN01.cgm"), DecodeLimits::default())
        .expect("selected text-alignment fixture should inspect");
    let picture = &inspection.pictures[0];

    assert!(picture
        .attributes
        .iter()
        .any(|attribute| matches!(&attribute.value, CgmAttributeValue::TextAlignment { .. })));
    assert!(picture
        .text_records
        .iter()
        .any(|record| record.state.text_alignment.is_some()));
    assert!(inspection.elements.iter().any(|element| {
        element.class == 5 && element.id == 18 && element.support == ElementSupport::Attribute
    }));
}

#[test]
fn selected_character_spacing_fixture_preserves_encoded_source_state_without_layout() {
    let inspection = inspect_binary_cgm_file(fixture("CHRSPA01.cgm"), DecodeLimits::default())
        .expect("selected character-spacing fixture should inspect");
    let picture = &inspection.pictures[0];

    assert!(picture.attributes.iter().any(|attribute| matches!(
        &attribute.value,
        CgmAttributeValue::CharacterSpacing { bytes } if !bytes.is_empty()
    )));
    assert!(picture
        .text_records
        .iter()
        .any(|record| record.state.character_spacing.is_some()));
    assert!(inspection.elements.iter().any(|element| {
        element.class == 5 && element.id == 13 && element.support == ElementSupport::Attribute
    }));
}

#[test]
fn selected_text_path_fixture_preserves_direction_source_state_without_layout() {
    let inspection = inspect_binary_cgm_file(fixture("TXTPTH01.cgm"), DecodeLimits::default())
        .expect("selected text-path fixture should inspect");
    let picture = &inspection.pictures[0];

    assert!(picture.attributes.iter().any(|attribute| matches!(
        &attribute.value,
        CgmAttributeValue::TextPath { value } if *value <= 3
    )));
    assert!(picture
        .text_records
        .iter()
        .any(|record| record.state.text_path.is_some()));
    assert!(inspection.elements.iter().any(|element| {
        element.class == 5 && element.id == 17 && element.support == ElementSupport::Attribute
    }));
}

#[test]
fn selected_cell_array_fixture_preserves_its_raster_header_without_decoding_pixels() {
    let inspection = inspect_binary_cgm_file(fixture("CELARY01.cgm"), DecodeLimits::default())
        .expect("selected cell-array fixture should inspect");
    assert!(inspection.elements.iter().any(|element| {
        element.class == 4 && element.id == 9 && element.support == ElementSupport::Raster
    }));
    assert!(!inspection
        .diagnostics
        .iter()
        .any(|diagnostic| diagnostic.class == 4 && diagnostic.id == 9));
    let record = inspection.pictures[0]
        .cell_arrays
        .first()
        .expect("fixture should preserve one cell array header");
    assert_eq!(record.first, [400, 700]);
    assert_eq!(record.second, [600, 500]);
    assert_eq!(record.third, [600, 700]);
    assert_eq!(record.dimensions, [2, 2]);
    assert_eq!(record.local_color_precision, 8);
    assert_eq!(record.representation, 0);
    assert_eq!(record.payload_bytes, 12);
    assert!(
        inspection.pictures.iter().all(|picture| {
            picture
                .primitives
                .iter()
                .all(|primitive| primitive.source_offset < inspection.source_bytes)
        }),
        "deferred cell-array records must not corrupt the surrounding picture lifecycle"
    );
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

    let additional = inspect_binary_cgm_file(fixture("CLIPNG02.cgm"), DecodeLimits::default())
        .expect("additional selected clipping fixture should inspect");
    let additional_picture = additional
        .pictures
        .first()
        .expect("CLIPNG02 should contain a picture");
    assert!(additional_picture.controls.clip_rectangle.is_some());
    assert!(additional_picture.primitives.iter().all(|primitive| {
        primitive.controls.clip_rectangle.is_some()
            && primitive.controls.clip_indicator != Some(CgmClipIndicator::On)
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
fn selected_lowering_keeps_fill_edge_and_stroke_intent_distinct() {
    let line = inspect_binary_cgm_file(fixture("POLYLN01.cgm"), DecodeLimits::default())
        .expect("selected polyline fixture should inspect");
    let line_primitives = lower_picture_primitives(&line.pictures[0])
        .expect("selected polyline fixture should lower");
    assert!(line_primitives.iter().any(|primitive| {
        primitive.topology == CgmPrimitiveTopology::Open
            && primitive.presentation.stroke == CgmStrokeIntent::SourceDefined
            && primitive.presentation.fill == CgmFillIntent::NotApplicable
            && primitive.presentation.edge == CgmEdgeIntent::NotApplicable
    }));

    let polygon = inspect_binary_cgm_file(fixture("INTSTL01.cgm"), DecodeLimits::default())
        .expect("selected interior-style fixture should inspect");
    let polygon_primitives = lower_picture_primitives(&polygon.pictures[0])
        .expect("selected interior-style fixture should lower");
    assert!(polygon_primitives.iter().any(|primitive| {
        primitive.topology == CgmPrimitiveTopology::Closed
            && primitive.presentation.fill == CgmFillIntent::SourceSolid
            && matches!(
                primitive.presentation.edge,
                CgmEdgeIntent::SourceVisible | CgmEdgeIntent::SourceHidden
            )
            && primitive.presentation.stroke == CgmStrokeIntent::NotApplicable
    }));
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
