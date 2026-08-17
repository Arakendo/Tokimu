use hello_doom_visibility_conformance::{
    compose_ordered_partitions, AuthorityKind, AuthorityOccurrence, ContributionDisposition,
    ContributionDomain, ContributionProvenance, DepthProfileSegment, FiniteInterval,
    OrderedPartitionAuthority, RelationalTolerance,
};

fn interval(start: f64, end: f64) -> FiniteInterval {
    FiniteInterval::new(start, end).expect("static finite interval")
}

fn authority(
    identity: &str,
    order: u32,
    start: f64,
    end: f64,
    delta: f64,
) -> OrderedPartitionAuthority {
    OrderedPartitionAuthority {
        authority: AuthorityOccurrence {
            identity: identity.to_owned(),
            order,
            source_parameter: interval(start, end),
            view_horizontal: interval(0.0, 1.0),
            vertical: interval(0.0, 1.0),
        },
        kind: AuthorityKind::SolidCoverage,
        depth_profiles: vec![DepthProfileSegment {
            source_parameter: interval(start, end),
            candidate_minus_authority_start: delta,
            candidate_minus_authority_end: delta,
        }],
    }
}

fn main() {
    let result = compose_ordered_partitions(
        ContributionProvenance {
            source_identity: "synthetic-two-authority-contribution".to_owned(),
            sidedef_role: "ceiling-occurrence".to_owned(),
            material_identity: "CEIL3_5".to_owned(),
        },
        ContributionDomain {
            source_parameter: interval(0.0, 1.0),
            horizontal: interval(0.0, 1.0),
            vertical: interval(0.0, 1.0),
        },
        &[
            authority("later-beyond", 2, 0.5, 1.0, 1.0),
            authority("earlier-nearer", 1, 0.0, 0.5, -1.0),
        ],
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
    assert_eq!(count(ContributionDisposition::UnresolvedFailOpen), 0);

    println!(
        "status=validated; fixture=ordered-partition-composition; authorities={}; steps={:?}; fragments={}; retained={}; rejected={}; unresolved={}; conserved={}; renderer-policy=none; screen-columns=none; stable-contract=none",
        2,
        result.steps,
        result.fragments.len(),
        count(ContributionDisposition::RetainedNearer),
        count(ContributionDisposition::RejectedBeyond),
        count(ContributionDisposition::UnresolvedFailOpen),
        result.is_conserved(1.0e-9),
    );
}
