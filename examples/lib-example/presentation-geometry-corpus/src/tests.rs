use super::*;
use crate::cases::{CgmExpectation, CGM_SOURCE_STAGES, CGM_STAGES};

#[test]
fn initial_cases_are_deterministic_and_stage_order_is_stable() {
    assert_eq!(glyph_cases()[0], GlyphCase::new("glyph/inter/K", 'K'));
    assert_eq!(
        CorpusStage::ALL.map(CorpusStage::name),
        ["source", "xml", "outline", "vector", "mesh"]
    );
}

#[test]
fn mesh_fingerprint_ignores_triangle_and_vertex_order() {
    let first = [
        [0.0, 0.0],
        [1.0, 0.0],
        [0.0, 1.0],
        [1.0, 0.0],
        [1.0, 1.0],
        [0.0, 1.0],
    ];
    let reordered = [
        [0.0, 1.0],
        [1.0, 1.0],
        [1.0, 0.0],
        [0.0, 1.0],
        [0.0, 0.0],
        [1.0, 0.0],
    ];
    assert_eq!(
        canonical_triangle_hash(&first),
        canonical_triangle_hash(&reordered)
    );
}

#[test]
fn generated_cases_replay_from_seed_and_index() {
    let first = run_generated_case(42, 7);
    let replay = run_generated_case(42, 7);
    assert_eq!(first.id, "generated/42/7");
    assert_eq!(first.producer, replay.producer);
    assert_eq!(first.stages, replay.stages);
    assert_eq!(first.diagnostics, replay.diagnostics);
    assert!(first.passed());
}

#[test]
fn case_lookup_does_not_accept_unknown_cases() {
    assert!(find_glyph_case("glyph/inter/k").is_some());
    assert!(find_glyph_case("glyph/inter/unknown").is_none());
}

#[test]
fn synthetic_cases_have_stable_ids_and_valid_input_paths() {
    assert_eq!(synthetic_cases().len(), 5);
    for case in synthetic_cases() {
        assert!(synthetic_path(*case).is_finite());
    }
}

#[test]
fn self_intersection_is_classified_at_the_vector_boundary() {
    let report = run_synthetic_case(synthetic_cases()[4]);
    assert!(report.passed());
    assert_eq!(report.stages[1].stage, CorpusStage::Vector);
    assert_eq!(report.stages[1].status, StageStatus::ExpectedFailure);
    assert_eq!(report.stages[2].stage, CorpusStage::Mesh);
    assert_eq!(report.stages[2].status, StageStatus::ExpectedFailure);
}

#[test]
fn golden_diff_reports_the_first_changed_line() {
    let diff = golden_diff("one\ntwo\n", "one\nchanged\n");
    assert!(diff.contains("line 2"));
    assert!(diff.contains("expected: two"));
    assert!(diff.contains("actual:   changed"));
}

#[test]
fn svg_cases_have_stable_ids() {
    assert_eq!(svg_cases().len(), 1);
    assert_eq!(svg_cases()[0].id, "svg/lucide/archive");
    assert_eq!(
        find_case("svg/lucide/archive"),
        Some(CorpusCase::Svg(svg_cases()[0]))
    );
}

#[test]
fn synthetic_svg_namespace_case_ignores_foreign_local_name_collisions() {
    let case = synthetic_svg_cases()[0];
    let report = run_synthetic_svg_case(case);
    assert!(report.passed(), "{report:#?}");
    assert_eq!(report.producer, "svg/synthetic");
    assert_eq!(report.stages[2].stage, CorpusStage::Vector);
    assert_eq!(report.stages[2].status, StageStatus::Ready);
    assert!(all_cases().contains(&CorpusCase::SyntheticSvg(case)));
}

#[test]
fn w3c_cases_are_registered_for_golden_comparison() {
    assert_eq!(w3c_svg_cases().len(), 50);
    for case in w3c_svg_cases() {
        let corpus_case = CorpusCase::W3cSvg(*case);
        assert!(all_cases().contains(&corpus_case));
        assert_eq!(find_case(case.id), Some(corpus_case));
    }
}

#[test]
fn w3c_profile_exclusions_are_explicitly_classified() {
    let upstream = w3c_svg_cases()
        .iter()
        .filter(|case| case.source == W3cSvgSource::UpstreamSvg)
        .collect::<Vec<_>>();
    assert_eq!(upstream.len(), 4);
    assert!(upstream
        .iter()
        .all(|case| case.expectation == W3cSvgExpectation::UnsupportedProfile));

    let derived = w3c_svg_cases()
        .iter()
        .filter(|case| case.source == W3cSvgSource::DerivedProfileFixture)
        .collect::<Vec<_>>();
    assert_eq!(derived.len(), 46);
    assert_eq!(
        derived
            .iter()
            .filter(|case| case.expectation == W3cSvgExpectation::StructuralPass)
            .count(),
        45
    );
    assert_eq!(
        derived
            .iter()
            .filter(|case| case.expectation == W3cSvgExpectation::ExpectedInvalidInput)
            .count(),
        1
    );
}

#[test]
fn ui_cases_have_stable_ids() {
    assert_eq!(ui_cases().len(), 1);
    assert_eq!(
        find_case("ui/panel-surface"),
        Some(CorpusCase::Ui(ui_cases()[0]))
    );
}

#[test]
fn cgm_cases_are_registered_at_their_honest_stage_boundary() {
    assert_eq!(cgm_cases().len(), 15);
    for case in cgm_cases() {
        let corpus_case = CorpusCase::Cgm(*case);
        assert!(all_cases().contains(&corpus_case));
        assert_eq!(find_case(case.id), Some(corpus_case));

        let report = run_cgm_case(*case);
        assert!(report.passed(), "{report:#?}");
        assert_eq!(report.producer, "cgm/webcgm");
        assert!(report.stages[0].summary.contains("elements="));
        assert!(report.stages[0].summary.contains("stateful-primitives="));
        if case.expectation == CgmExpectation::SourceOnly {
            assert_eq!(report.selected_stages, vec![CorpusStage::Source]);
            assert_eq!(report.stages.len(), 1);
        } else {
            assert_eq!(report.stages[1].stage, CorpusStage::Vector);
            assert!(matches!(
                report.stages[1].status,
                StageStatus::Ready | StageStatus::ExpectedFailure
            ));
        }
    }
}

#[test]
fn cgm_source_only_cases_preserve_inventory_and_vdc_evidence() {
    let inventory = cgm_cases()
        .iter()
        .copied()
        .find(|case| case.id == "cgm/webcgm/element-inventory")
        .expect("element inventory case should be registered");
    let inventory_report = run_cgm_case(inventory);
    assert!(inventory_report.passed(), "{inventory_report:#?}");
    assert_eq!(inventory_report.stages.len(), 1);
    assert!(inventory_report.stages[0].summary.contains("elements="));

    let vdc_extent = cgm_cases()
        .iter()
        .copied()
        .find(|case| case.id == "cgm/webcgm/vdc-extent")
        .expect("VDC extent case should be registered");
    let extent_report = run_cgm_case(vdc_extent);
    assert!(extent_report.passed(), "{extent_report:#?}");
    assert_eq!(extent_report.stages.len(), 1);
    assert!(extent_report.stages[0]
        .summary
        .contains("vdc-extent-pictures=1"));
    assert!(extent_report.stages[0]
        .summary
        .contains("metric-scaling-pictures=1"));
}

#[test]
fn cgm_non_vector_cases_write_source_artifacts_without_fabricating_vectors() {
    let source_only = cgm_cases()[0];
    let source_root = crate::write_cgm_artifacts(source_only).expect("write source-only artifacts");
    assert!(source_root.join("source.cgm").is_file());
    assert!(source_root.join("cgm.json").is_file());
    assert!(source_root.join("graph.json").is_file());
    assert!(!source_root.join("vector.json").exists());
    assert!(!source_root.join("vector-fingerprint.json").exists());

    let polygon_set = cgm_cases()
        .iter()
        .copied()
        .find(|case| case.id == "cgm/webcgm/polygon-set")
        .expect("polygon-set case should be registered");
    let polygon_root =
        crate::write_cgm_artifacts(polygon_set).expect("write expected-boundary source artifacts");
    assert!(!polygon_root.join("vector.json").exists());
    assert!(!polygon_root.join("vector-fingerprint.json").exists());
    let graph = std::fs::read_to_string(polygon_root.join("graph.json"))
        .expect("read expected-boundary graph artifact");
    assert!(graph.contains("expected-failure"));
    assert!(graph.contains("not-produced"));
}

#[test]
fn cgm_vector_artifacts_emit_repeatable_structural_fingerprints() {
    let case = cgm_cases()
        .iter()
        .copied()
        .find(|case| case.id == "cgm/webcgm/circle")
        .expect("circle case should be registered");

    let root = crate::write_cgm_artifacts(case).expect("write first CGM vector artifacts");
    let first = std::fs::read_to_string(root.join("vector-fingerprint.json"))
        .expect("read first CGM vector fingerprint");
    crate::write_cgm_artifacts(case).expect("write second CGM vector artifacts");
    let second = std::fs::read_to_string(root.join("vector-fingerprint.json"))
        .expect("read second CGM vector fingerprint");

    assert_eq!(
        first, second,
        "CGM vector artifact fingerprint must be repeatable"
    );
    assert!(first.contains("canonical_path_hash"));
}

#[test]
fn cgm_source_only_artifact_writing_clears_stale_vector_evidence() {
    let source_only = cgm_cases()[0];
    let root = std::path::PathBuf::from("target/presentation-geometry-corpus").join(source_only.id);
    std::fs::create_dir_all(&root).expect("create source-only artifact directory");
    for file_name in ["vector.json", "vector-fingerprint.json"] {
        std::fs::write(root.join(file_name), "stale")
            .unwrap_or_else(|error| panic!("seed stale {file_name}: {error}"));
    }

    let root = crate::write_cgm_artifacts(source_only).expect("write source-only CGM artifacts");
    assert!(!root.join("vector.json").exists());
    assert!(!root.join("vector-fingerprint.json").exists());
}

#[test]
fn cgm_polygon_set_is_an_explicit_expected_vector_boundary() {
    let polygon_set = cgm_cases()
        .iter()
        .copied()
        .find(|case| case.id == "cgm/webcgm/polygon-set")
        .expect("polygon-set case should be registered");
    let report = run_cgm_case(polygon_set);

    assert!(report.passed(), "{report:#?}");
    assert_eq!(report.stages[1].status, StageStatus::ExpectedFailure);
    assert!(report.stages[1]
        .summary
        .contains("polygon-set point/flag topology"));
}

#[test]
fn cgm_runner_source_stage_reports_state_and_clip_observations() {
    let direct_color = cgm_cases()
        .iter()
        .copied()
        .find(|case| case.id == "cgm/webcgm/direct-colors")
        .expect("direct-color case should be registered");
    let direct_report = run_cgm_case(direct_color);
    assert!(direct_report.stages[0]
        .summary
        .contains("stateful-primitives=1"));

    let clipping = cgm_cases()
        .iter()
        .copied()
        .find(|case| case.id == "cgm/webcgm/clip-controls")
        .expect("clip-control case should be registered");
    let clip_report = run_cgm_case(clipping);
    assert!(clip_report.stages[0]
        .summary
        .contains("clip-rectangle-primitives=9"));
}

#[test]
fn cgm_artifacts_preserve_source_and_vector_evidence() {
    let root = crate::write_cgm_artifacts(cgm_cases()[4]).expect("write CGM artifacts");

    for artifact in [
        "source.cgm",
        "cgm.json",
        "vector.json",
        "vector-fingerprint.json",
        "graph.json",
    ] {
        assert!(root.join(artifact).is_file(), "missing {artifact}");
    }

    let graph = std::fs::read_to_string(root.join("graph.json")).expect("read graph artifact");
    assert!(graph.contains("expected-failure"));
    assert!(graph.contains("vector.json"));

    let cgm = std::fs::read_to_string(root.join("cgm.json")).expect("read CGM artifact");
    let cgm: serde_json::Value = serde_json::from_str(&cgm).expect("parse CGM artifact");
    let primitives = cgm["primitives"]
        .as_array()
        .expect("CGM artifact should retain primitive source snapshots");
    assert_eq!(
        primitives.len(),
        cgm["primitive_count"]
            .as_u64()
            .expect("CGM artifact should declare a primitive count") as usize
    );
    let attribute_count = cgm["attribute_count"]
        .as_u64()
        .expect("CGM artifact should declare an attribute count");
    assert!(primitives.iter().all(|primitive| {
        primitive["attribute_count"]
            .as_u64()
            .is_some_and(|count| count <= attribute_count)
            && primitive.get("state").is_some()
            && primitive.get("controls").is_some()
    }));
}

#[test]
fn cgm_source_artifacts_preserve_state_oriented_fixture_evidence() {
    let direct_color = cgm_cases()
        .iter()
        .copied()
        .find(|case| case.id == "cgm/webcgm/direct-colors")
        .expect("direct-color case should be registered");
    let direct_root =
        crate::write_cgm_artifacts(direct_color).expect("write direct-color artifacts");
    let direct: serde_json::Value = serde_json::from_str(
        &std::fs::read_to_string(direct_root.join("cgm.json")).expect("read direct-color artifact"),
    )
    .expect("parse direct-color artifact");
    assert!(direct["primitives"].as_array().is_some_and(|primitives| {
        primitives.iter().any(|primitive| {
            primitive["state"]["line_color"]["kind"] == "direct"
                || primitive["state"]["fill_color"]["kind"] == "direct"
        })
    }));

    let clipping = cgm_cases()
        .iter()
        .copied()
        .find(|case| case.id == "cgm/webcgm/clip-controls")
        .expect("clip-control case should be registered");
    let clip_root = crate::write_cgm_artifacts(clipping).expect("write clip-control artifacts");
    let clip: serde_json::Value = serde_json::from_str(
        &std::fs::read_to_string(clip_root.join("cgm.json")).expect("read clip-control artifact"),
    )
    .expect("parse clip-control artifact");
    assert!(clip["primitives"].as_array().is_some_and(|primitives| {
        primitives.iter().all(|primitive| {
            primitive["controls"]["clip_rectangle"].is_object()
                && primitive["controls"]["clip_indicator"] == "off"
        })
    }));
}

#[test]
fn producer_stage_selection_is_explicit() {
    assert_eq!(
        CorpusCase::Glyph(glyph_cases()[0]).selected_stages(),
        &GLYPH_STAGES
    );
    assert_eq!(
        CorpusCase::Svg(svg_cases()[0]).selected_stages(),
        &SVG_STAGES
    );
    assert_eq!(
        CorpusCase::SyntheticSvg(synthetic_svg_cases()[0]).selected_stages(),
        &SVG_STAGES
    );
    assert_eq!(
        CorpusCase::W3cSvg(w3c_svg_cases()[0]).selected_stages(),
        &SVG_STAGES
    );
    assert_eq!(
        CorpusCase::Cgm(cgm_cases()[0]).selected_stages(),
        &CGM_SOURCE_STAGES
    );
    assert_eq!(
        CorpusCase::Cgm(cgm_cases()[2]).selected_stages(),
        &CGM_STAGES
    );
}

#[test]
fn xml_stage_retains_parser_neutral_document_evidence() {
    let inspection = inspect_xml_stage(
        "<?xml version=\"1.0\"?><svg><!-- note --><path d=\"M 0 0\"/></svg>",
        XmlSourceId::new(77),
    )
    .expect("valid XML should produce stage evidence");
    assert_eq!(inspection.evidence.start_elements, 2);
    assert_eq!(inspection.evidence.end_elements, 2);
    assert_eq!(inspection.evidence.comments, 1);
    // XML declarations are handled by the parser profile, rather than
    // retained as ordinary processing-instruction document nodes.
    assert_eq!(inspection.evidence.processing_instructions, 0);
    assert_eq!(inspection.evidence.document_roots, 1);
    assert!(inspection.evidence.has_document_element);
    assert_eq!(inspection.events.len(), inspection.evidence.event_count);
}
