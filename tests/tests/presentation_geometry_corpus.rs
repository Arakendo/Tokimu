use presentation_geometry_corpus::{find_case, run_case, run_generated_case, CorpusCase};

#[test]
fn workspace_consumer_can_run_synthetic_geometry_cases() {
    for id in [
        "synthetic/convex-rectangle",
        "synthetic/concave-notch",
        "synthetic/multi-contour-hole",
        "synthetic/near-degenerate",
        "synthetic/self-intersection-bowtie",
    ] {
        let case = find_case(id).expect("registered corpus case");
        let report = run_case(case);
        assert!(report.passed(), "{id} failed: {:?}", report.diagnostics);
    }
}

#[test]
fn workspace_consumer_can_distinguish_producer_cases() {
    assert!(matches!(
        find_case("synthetic/convex-rectangle"),
        Some(CorpusCase::Synthetic(_))
    ));
    assert!(matches!(
        find_case("ui/panel-surface"),
        Some(CorpusCase::Ui(_))
    ));
    assert!(matches!(
        find_case("svg/w3c/painting-fill-03-t"),
        Some(CorpusCase::W3cSvg(_))
    ));
    assert!(matches!(
        find_case("svg/w3c/paths-data-16-t"),
        Some(CorpusCase::W3cSvg(_))
    ));
}

#[test]
fn workspace_consumer_can_run_admitted_w3c_geometry() {
    for id in ["svg/w3c/painting-fill-03-t", "svg/w3c/paths-data-16-t"] {
        let case = find_case(id).expect("registered W3C corpus case");
        let report = run_case(case);
        assert!(report.passed(), "W3C case failed: {:?}", report.diagnostics);
    }
}

#[test]
fn seeded_generation_has_no_unpromoted_failures_for_review_seed() {
    for seed in [42, 1, 7, 2026] {
        for index in 0..250 {
            let report = run_generated_case(seed, index);
            assert!(
                report.passed(),
                "generated/{seed}/{index} failed: {:?}",
                report.diagnostics
            );
        }
    }
}
