use hello_doom_visibility_conformance::{
    classify_relational_depth, observe_candidate_support, resolve_ordered_authority,
    AuthorityOccurrence, CandidateFacingObservation, CandidateSourceSupport, ComparisonConvention,
    ComparisonSample, ComparisonSampleValue, DepthObservation, FiniteInterval,
    OrderedAuthorityResolution, RelationalTolerance,
};

fn interval(start: f64, end: f64) -> FiniteInterval {
    FiniteInterval::new(start, end).expect("static finite interval")
}

fn main() {
    let candidate = CandidateSourceSupport::WallSeg {
        source_seg: 559,
        source_parameter: interval(0.0, 1.0),
        view_horizontal: interval(0.30, 0.70),
        vertical: interval(0.25, 0.80),
    };
    let authority = AuthorityOccurrence {
        identity: "synthetic-finite-sky-boundary".to_owned(),
        order: 7,
        source_parameter: interval(0.10, 0.90),
        view_horizontal: interval(0.25, 0.75),
        vertical: interval(0.20, 0.90),
    };
    let support = observe_candidate_support(&candidate, &authority);
    let sample = |horizontal, candidate_ray_t| ComparisonSample {
        horizontal,
        vertical: 0.5,
        authority_source_parameter: 0.5,
        convention: ComparisonConvention::ExplicitRay,
        value: ComparisonSampleValue::Comparable {
            candidate_ray_t,
            authority_ray_t: 8.0,
        },
    };
    let tolerance = RelationalTolerance::new(0.001).expect("static tolerance");
    let facing = CandidateFacingObservation {
        normal_dot_view: -1.0,
    };
    let nearer = classify_relational_depth(
        support.clone(),
        &authority,
        &[sample(0.4, 4.0)],
        tolerance,
        facing,
    );
    let beyond = classify_relational_depth(
        support.clone(),
        &authority,
        &[sample(0.4, 9.0)],
        tolerance,
        facing,
    );
    let straddling = classify_relational_depth(
        support,
        &authority,
        &[sample(0.4, 4.0), sample(0.6, 9.0)],
        tolerance,
        facing,
    );
    let ledger = resolve_ordered_authority(&candidate, &[authority]);

    assert_eq!(nearer.depth, Some(DepthObservation::Nearer));
    assert_eq!(beyond.depth, Some(DepthObservation::Beyond));
    assert_eq!(straddling.depth, Some(DepthObservation::Straddling));
    assert!(matches!(
        ledger,
        OrderedAuthorityResolution::Resolved { .. }
    ));
    println!(
        "status=validated; fixture=relational-classifier; support={:?}; ledger={:?}; nearer={:?}; beyond={:?}; straddling={:?}; comparison-domain={}; epsilon={:.6}; renderer-policy=none; stable-contract=none",
        nearer.support,
        ledger,
        nearer.depth,
        beyond.depth,
        straddling.depth,
        nearer.comparison_domain,
        tolerance.ray_t_epsilon,
    );
}
