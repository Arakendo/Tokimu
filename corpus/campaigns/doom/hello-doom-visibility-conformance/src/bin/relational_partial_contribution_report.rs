use hello_doom_visibility_conformance::{
    split_contribution, ContributionDisposition, ContributionDomain, ContributionProvenance,
    DepthProfileSegment, FiniteInterval, RelationalTolerance, SupportObservation,
};

fn interval(start: f64, end: f64) -> FiniteInterval {
    FiniteInterval::new(start, end).expect("static finite interval")
}

fn main() {
    let original = ContributionDomain {
        source_parameter: interval(0.0, 1.0),
        horizontal: interval(0.0, 1.0),
        vertical: interval(0.0, 1.0),
    };
    let support = SupportObservation::Supported {
        candidate_source_parameter: interval(0.0, 1.0),
        source_parameter_overlap: interval(0.2, 0.8),
        horizontal_overlap: interval(0.1, 0.9),
        vertical_overlap: interval(0.25, 0.75),
        outside_source_parameter: [Some(interval(0.0, 0.2)), Some(interval(0.8, 1.0))],
        outside_horizontal: [Some(interval(0.0, 0.1)), Some(interval(0.9, 1.0))],
        outside_vertical: [Some(interval(0.0, 0.25)), Some(interval(0.75, 1.0))],
    };
    let result = split_contribution(
        ContributionProvenance {
            source_identity: "synthetic-subsector-104-ceiling-occurrence-41".to_owned(),
            sidedef_role: "ceiling".to_owned(),
            material_identity: "CEIL3_5".to_owned(),
        },
        original,
        &support,
        &[DepthProfileSegment {
            source_parameter: interval(0.2, 0.8),
            candidate_minus_authority_start: -2.0,
            candidate_minus_authority_end: 2.0,
        }],
        RelationalTolerance::new(0.001).expect("static tolerance"),
    );

    let count = |disposition| {
        result
            .fragments
            .iter()
            .filter(|fragment| fragment.disposition == disposition)
            .count()
    };
    assert!(result.is_conserved(1.0e-9));
    assert_eq!(count(ContributionDisposition::RetainedNearer), 1);
    assert_eq!(count(ContributionDisposition::RejectedBeyond), 1);
    assert_eq!(count(ContributionDisposition::OutsideSourceSupport), 6);
    assert_eq!(count(ContributionDisposition::UnresolvedFailOpen), 0);

    println!(
        "status=validated; fixture=relational-partial-contribution; fragments={}; retained={}; rejected={}; outside-support={}; unresolved={}; conserved={}; source-identity={}; sidedef-role={}; material={}; uv-source-ranges={:?}; renderer-policy=none; stable-contract=none",
        result.fragments.len(),
        count(ContributionDisposition::RetainedNearer),
        count(ContributionDisposition::RejectedBeyond),
        count(ContributionDisposition::OutsideSourceSupport),
        count(ContributionDisposition::UnresolvedFailOpen),
        result.is_conserved(1.0e-9),
        result.fragments[0].provenance.source_identity,
        result.fragments[0].provenance.sidedef_role,
        result.fragments[0].provenance.material_identity,
        result
            .fragments
            .iter()
            .map(|fragment| fragment.uv_source_parameter)
            .collect::<Vec<_>>(),
    );
}
