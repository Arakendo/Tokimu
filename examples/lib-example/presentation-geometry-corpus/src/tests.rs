use super::*;

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
    assert_eq!(w3c_svg_cases().len(), 40);
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
    assert_eq!(derived.len(), 36);
    assert_eq!(
        derived
            .iter()
            .filter(|case| case.expectation == W3cSvgExpectation::StructuralPass)
            .count(),
        35
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
