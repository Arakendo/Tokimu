//! Corpus-private relational contribution classification for the AR-0030
//! Doom study.
//!
//! This model intentionally is not renderer vocabulary. It separates a
//! contribution's own finite source support, an ordered Doom authority
//! occurrence, and comparable view-ray depth observations. A source boundary
//! never gains authority through its infinite supporting plane.

use std::cmp::Ordering;

/// A finite half-open interval in a declared comparison domain.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct FiniteInterval {
    pub start: f64,
    pub end: f64,
}

impl FiniteInterval {
    pub fn new(start: f64, end: f64) -> Option<Self> {
        (start.is_finite() && end.is_finite() && start < end).then_some(Self { start, end })
    }

    pub fn intersection(self, other: Self) -> Option<Self> {
        Self::new(self.start.max(other.start), self.end.min(other.end))
    }

    pub fn contains(self, value: f64) -> bool {
        value >= self.start && value < self.end
    }

    pub fn outside(self, retained: Self) -> [Option<Self>; 2] {
        [
            Self::new(self.start, retained.start),
            Self::new(retained.end, self.end),
        ]
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PlaneKind {
    Floor,
    Ceiling,
}

/// The source-owned domain in which one candidate contribution is eligible.
/// Wall identity is SEG-granular. Plane identity is occurrence/subsector-local,
/// never sector-global.
#[derive(Clone, Debug, PartialEq)]
pub enum CandidateSourceSupport {
    WallSeg {
        source_seg: u16,
        source_parameter: FiniteInterval,
        view_horizontal: FiniteInterval,
        vertical: FiniteInterval,
    },
    PlaneOccurrence {
        source_subsector: u16,
        plane: PlaneKind,
        occurrence: u64,
        source_parameter: FiniteInterval,
        view_horizontal: FiniteInterval,
        vertical: FiniteInterval,
    },
    Unresolved {
        reason: String,
    },
}

impl CandidateSourceSupport {
    fn finite_domains(&self) -> Option<(FiniteInterval, FiniteInterval, FiniteInterval)> {
        match self {
            Self::WallSeg {
                source_parameter,
                view_horizontal,
                vertical,
                ..
            }
            | Self::PlaneOccurrence {
                source_parameter,
                view_horizontal,
                vertical,
                ..
            } => Some((*source_parameter, *view_horizontal, *vertical)),
            Self::Unresolved { .. } => None,
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum SupportObservation {
    Supported {
        candidate_source_parameter: FiniteInterval,
        source_parameter_overlap: FiniteInterval,
        horizontal_overlap: FiniteInterval,
        vertical_overlap: FiniteInterval,
        outside_source_parameter: [Option<FiniteInterval>; 2],
        outside_horizontal: [Option<FiniteInterval>; 2],
        outside_vertical: [Option<FiniteInterval>; 2],
    },
    OutsideSourceSupport {
        candidate_source_parameter: FiniteInterval,
        reason: &'static str,
    },
    UnresolvedSupport {
        reason: String,
    },
}

/// One source-authorized occurrence in Doom's retained near-to-far order.
#[derive(Clone, Debug, PartialEq)]
pub struct AuthorityOccurrence {
    pub identity: String,
    pub order: u32,
    pub source_parameter: FiniteInterval,
    pub view_horizontal: FiniteInterval,
    pub vertical: FiniteInterval,
}

#[derive(Clone, Debug, PartialEq)]
pub enum OrderedAuthorityResolution {
    Resolved {
        authority: AuthorityOccurrence,
        support: SupportObservation,
    },
    OutsideAllAuthority {
        candidate_source_parameter: FiniteInterval,
    },
    Unresolved {
        reason: String,
    },
}

/// Candidate-facing evidence is retained for diagnosis but cannot authorize a
/// visibility decision.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct CandidateFacingObservation {
    pub normal_dot_view: f64,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ComparisonConvention {
    ColumnCenter,
    ColumnEdge,
    ExplicitRay,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum ComparisonSampleValue {
    Comparable {
        candidate_ray_t: f64,
        authority_ray_t: f64,
    },
    Parallel,
    BehindView,
    NearPlane,
    NonFinite,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ComparisonSample {
    pub horizontal: f64,
    pub vertical: f64,
    pub authority_source_parameter: f64,
    pub convention: ComparisonConvention,
    pub value: ComparisonSampleValue,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct RelationalTolerance {
    pub ray_t_epsilon: f64,
}

impl RelationalTolerance {
    pub fn new(ray_t_epsilon: f64) -> Option<Self> {
        (ray_t_epsilon.is_finite() && ray_t_epsilon >= 0.0).then_some(Self { ray_t_epsilon })
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum DepthObservation {
    Nearer,
    Beyond,
    Straddling,
    Unresolved { reason: String },
}

#[derive(Clone, Debug, PartialEq)]
pub struct RelationalClassification {
    pub support: SupportObservation,
    pub depth: Option<DepthObservation>,
    pub facing: CandidateFacingObservation,
    pub comparison_domain: &'static str,
    pub tolerance: RelationalTolerance,
}

/// Three finite domains carried by one source contribution. This is
/// corpus-private bookkeeping, not a renderer primitive.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ContributionDomain {
    pub source_parameter: FiniteInterval,
    pub horizontal: FiniteInterval,
    pub vertical: FiniteInterval,
}

impl ContributionDomain {
    fn volume(self) -> f64 {
        (self.source_parameter.end - self.source_parameter.start)
            * (self.horizontal.end - self.horizontal.start)
            * (self.vertical.end - self.vertical.start)
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ContributionProvenance {
    pub source_identity: String,
    pub sidedef_role: String,
    pub material_identity: String,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ContributionDisposition {
    RetainedNearer,
    RejectedBeyond,
    OutsideSourceSupport,
    UnresolvedFailOpen,
}

#[derive(Clone, Debug, PartialEq)]
pub struct ContributionFragment {
    pub provenance: ContributionProvenance,
    pub domain: ContributionDomain,
    /// UV progress remains tied to the original source parameterization.
    pub uv_source_parameter: FiniteInterval,
    pub disposition: ContributionDisposition,
    pub reason: &'static str,
}

/// A piecewise-linear depth relation over one source-parameter interval.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct DepthProfileSegment {
    pub source_parameter: FiniteInterval,
    pub candidate_minus_authority_start: f64,
    pub candidate_minus_authority_end: f64,
}

#[derive(Clone, Debug, PartialEq)]
pub struct ContributionConservation {
    pub original: ContributionDomain,
    pub fragments: Vec<ContributionFragment>,
}

impl ContributionConservation {
    pub fn is_conserved(&self, epsilon: f64) -> bool {
        let accounted = self
            .fragments
            .iter()
            .map(|fragment| fragment.domain.volume())
            .sum::<f64>();
        (accounted - self.original.volume()).abs() <= epsilon
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AuthorityKind {
    SolidCoverage,
    CutoutNonSolid,
}

/// One ordered, Doom-owned authority pass and the depth relation it can
/// establish inside its finite source support. This remains corpus-private;
/// it is not a renderer command or a general composition primitive.
#[derive(Clone, Debug, PartialEq)]
pub struct OrderedPartitionAuthority {
    pub authority: AuthorityOccurrence,
    pub kind: AuthorityKind,
    pub depth_profiles: Vec<DepthProfileSegment>,
}

/// Per-pass conservation evidence. `remaining` means eligible for a later
/// authority, not retained presentation.
#[derive(Clone, Debug, PartialEq)]
pub struct OrderedPartitionStep {
    pub authority_identity: String,
    pub authority_order: u32,
    pub remaining_before: f64,
    pub classified_now: f64,
    pub remaining_after: f64,
    pub conserved: bool,
}

#[derive(Clone, Debug, PartialEq)]
pub struct OrderedPartitionComposition {
    pub original: ContributionDomain,
    pub fragments: Vec<ContributionFragment>,
    pub steps: Vec<OrderedPartitionStep>,
}

impl OrderedPartitionComposition {
    pub fn is_conserved(&self, epsilon: f64) -> bool {
        let accounted = self
            .fragments
            .iter()
            .map(|fragment| fragment.domain.volume())
            .sum::<f64>();
        (accounted - self.original.volume()).abs() <= epsilon
            && self.steps.iter().all(|step| step.conserved)
    }
}

/// Monotonically composes finite authority occurrences over the contribution's
/// still-undecided domain. A pass can finalize nearer, beyond, or unresolved
/// fragments. Regions outside that pass remain eligible for later passes.
/// Nothing finalized by an earlier pass is reopened.
pub fn compose_ordered_partitions(
    provenance: ContributionProvenance,
    original: ContributionDomain,
    authorities: &[OrderedPartitionAuthority],
    tolerance: RelationalTolerance,
) -> OrderedPartitionComposition {
    let mut ordered = authorities.to_vec();
    ordered.sort_by(|left, right| {
        left.authority
            .order
            .cmp(&right.authority.order)
            .then_with(|| left.authority.identity.cmp(&right.authority.identity))
    });

    let mut remaining = vec![original];
    let mut fragments = Vec::new();
    let mut steps = Vec::new();

    for (index, pass) in ordered.iter().enumerate() {
        let before = normalized_volume(domains_volume(&remaining));
        if pass.kind == AuthorityKind::CutoutNonSolid {
            steps.push(OrderedPartitionStep {
                authority_identity: pass.authority.identity.clone(),
                authority_order: pass.authority.order,
                remaining_before: before,
                classified_now: 0.0,
                remaining_after: before,
                conserved: true,
            });
            continue;
        }

        if ordered[index + 1..]
            .iter()
            .take_while(|next| next.authority.order == pass.authority.order)
            .any(|next| {
                next.kind == AuthorityKind::SolidCoverage
                    && authority_domains_overlap(&pass.authority, &next.authority)
            })
        {
            for domain in remaining.drain(..) {
                fragments.push(fragment(
                    &provenance,
                    domain,
                    ContributionDisposition::UnresolvedFailOpen,
                    "ambiguous-overlapping-authority-order",
                ));
            }
            steps.push(OrderedPartitionStep {
                authority_identity: pass.authority.identity.clone(),
                authority_order: pass.authority.order,
                remaining_before: before,
                classified_now: before,
                remaining_after: 0.0,
                conserved: true,
            });
            break;
        }

        let mut next_remaining = Vec::new();
        let terminal_before = normalized_volume(domains_volume_from_fragments(&fragments));
        for domain in remaining.drain(..) {
            let Some(overlap) = authority_domain_overlap(domain, &pass.authority) else {
                next_remaining.push(domain);
                continue;
            };
            let support = SupportObservation::Supported {
                candidate_source_parameter: domain.source_parameter,
                source_parameter_overlap: overlap.source_parameter,
                horizontal_overlap: overlap.horizontal,
                vertical_overlap: overlap.vertical,
                outside_source_parameter: domain.source_parameter.outside(overlap.source_parameter),
                outside_horizontal: domain.horizontal.outside(overlap.horizontal),
                outside_vertical: domain.vertical.outside(overlap.vertical),
            };
            let profiles = crop_depth_profiles(&pass.depth_profiles, overlap.source_parameter);
            let partition =
                split_contribution(provenance.clone(), domain, &support, &profiles, tolerance);
            for classified in partition.fragments {
                if classified.disposition == ContributionDisposition::OutsideSourceSupport {
                    next_remaining.push(classified.domain);
                } else {
                    fragments.push(classified);
                }
            }
        }
        let after = normalized_volume(domains_volume(&next_remaining));
        let classified =
            normalized_volume(domains_volume_from_fragments(&fragments) - terminal_before);
        steps.push(OrderedPartitionStep {
            authority_identity: pass.authority.identity.clone(),
            authority_order: pass.authority.order,
            remaining_before: before,
            classified_now: classified,
            remaining_after: after,
            conserved: (before - classified - after).abs() <= tolerance.ray_t_epsilon,
        });
        remaining = next_remaining;
    }

    for domain in remaining {
        fragments.push(fragment(
            &provenance,
            domain,
            ContributionDisposition::UnresolvedFailOpen,
            "outside-all-ordered-authority",
        ));
    }
    OrderedPartitionComposition {
        original,
        fragments,
        steps,
    }
}

fn authority_domain_overlap(
    domain: ContributionDomain,
    authority: &AuthorityOccurrence,
) -> Option<ContributionDomain> {
    Some(ContributionDomain {
        source_parameter: domain
            .source_parameter
            .intersection(authority.source_parameter)?,
        horizontal: domain.horizontal.intersection(authority.view_horizontal)?,
        vertical: domain.vertical.intersection(authority.vertical)?,
    })
}

fn authority_domains_overlap(left: &AuthorityOccurrence, right: &AuthorityOccurrence) -> bool {
    left.source_parameter
        .intersection(right.source_parameter)
        .is_some()
        && left
            .view_horizontal
            .intersection(right.view_horizontal)
            .is_some()
        && left.vertical.intersection(right.vertical).is_some()
}

fn domains_volume(domains: &[ContributionDomain]) -> f64 {
    domains.iter().map(|domain| domain.volume()).sum()
}

fn domains_volume_from_fragments(fragments: &[ContributionFragment]) -> f64 {
    fragments
        .iter()
        .map(|fragment| fragment.domain.volume())
        .sum()
}

fn normalized_volume(value: f64) -> f64 {
    if value.abs() <= f64::EPSILON {
        0.0
    } else {
        value
    }
}

fn crop_depth_profiles(
    profiles: &[DepthProfileSegment],
    target: FiniteInterval,
) -> Vec<DepthProfileSegment> {
    profiles
        .iter()
        .filter_map(|profile| {
            let overlap = profile.source_parameter.intersection(target)?;
            let extent = profile.source_parameter.end - profile.source_parameter.start;
            let sample = |source_parameter: f64| {
                let progress = (source_parameter - profile.source_parameter.start) / extent;
                profile.candidate_minus_authority_start
                    + progress
                        * (profile.candidate_minus_authority_end
                            - profile.candidate_minus_authority_start)
            };
            Some(DepthProfileSegment {
                source_parameter: overlap,
                candidate_minus_authority_start: sample(overlap.start),
                candidate_minus_authority_end: sample(overlap.end),
            })
        })
        .collect()
}

/// Resolves only source occurrences allowed to close coverage. Masked/cutout
/// work remains drawable but cannot become relational authority.
pub fn resolve_ordered_solid_authority(
    candidate: &CandidateSourceSupport,
    authorities: &[(AuthorityOccurrence, AuthorityKind)],
) -> OrderedAuthorityResolution {
    let solid = authorities
        .iter()
        .filter(|(_, kind)| *kind == AuthorityKind::SolidCoverage)
        .map(|(authority, _)| authority.clone())
        .collect::<Vec<_>>();
    resolve_ordered_authority(candidate, &solid)
}

/// Partitions one complete contribution into explicit outside-support slabs
/// and depth-classified fragments within the finite authority overlap.
///
/// Depth profiles must cover the supported source-parameter interval without
/// gaps. Missing, overlapping, or non-finite profiles retain the whole
/// supported region as unresolved/fail-open evidence.
pub fn split_contribution(
    provenance: ContributionProvenance,
    original: ContributionDomain,
    support: &SupportObservation,
    profiles: &[DepthProfileSegment],
    tolerance: RelationalTolerance,
) -> ContributionConservation {
    let SupportObservation::Supported {
        source_parameter_overlap,
        horizontal_overlap,
        vertical_overlap,
        ..
    } = support
    else {
        return ContributionConservation {
            original,
            fragments: vec![fragment(
                &provenance,
                original,
                ContributionDisposition::UnresolvedFailOpen,
                "support-not-resolved-for-splitting",
            )],
        };
    };

    let overlap = ContributionDomain {
        source_parameter: *source_parameter_overlap,
        horizontal: *horizontal_overlap,
        vertical: *vertical_overlap,
    };
    let mut fragments = outside_support_slabs(&provenance, original, overlap);
    let mut cursor = overlap.source_parameter.start;
    let mut ordered = profiles.to_vec();
    ordered.sort_by(|left, right| {
        left.source_parameter
            .start
            .partial_cmp(&right.source_parameter.start)
            .unwrap_or(Ordering::Equal)
    });

    let valid = !ordered.is_empty()
        && ordered.iter().all(|profile| {
            profile.candidate_minus_authority_start.is_finite()
                && profile.candidate_minus_authority_end.is_finite()
                && profile.source_parameter.start >= overlap.source_parameter.start
                && profile.source_parameter.end <= overlap.source_parameter.end
        })
        && ordered.iter().all(|profile| {
            let contiguous =
                (profile.source_parameter.start - cursor).abs() <= tolerance.ray_t_epsilon;
            cursor = profile.source_parameter.end;
            contiguous
        })
        && (cursor - overlap.source_parameter.end).abs() <= tolerance.ray_t_epsilon;

    if !valid {
        fragments.push(fragment(
            &provenance,
            overlap,
            ContributionDisposition::UnresolvedFailOpen,
            "depth-profile-does-not-conserve-supported-range",
        ));
        return ContributionConservation {
            original,
            fragments,
        };
    }

    for profile in ordered {
        split_depth_profile(
            &provenance,
            overlap,
            profile,
            tolerance.ray_t_epsilon,
            &mut fragments,
        );
    }
    ContributionConservation {
        original,
        fragments,
    }
}

fn outside_support_slabs(
    provenance: &ContributionProvenance,
    original: ContributionDomain,
    retained: ContributionDomain,
) -> Vec<ContributionFragment> {
    let mut fragments = Vec::new();
    for interval in original
        .source_parameter
        .outside(retained.source_parameter)
        .into_iter()
        .flatten()
    {
        fragments.push(fragment(
            provenance,
            ContributionDomain {
                source_parameter: interval,
                ..original
            },
            ContributionDisposition::OutsideSourceSupport,
            "outside-source-parameter-support",
        ));
    }
    for interval in original
        .horizontal
        .outside(retained.horizontal)
        .into_iter()
        .flatten()
    {
        fragments.push(fragment(
            provenance,
            ContributionDomain {
                source_parameter: retained.source_parameter,
                horizontal: interval,
                vertical: original.vertical,
            },
            ContributionDisposition::OutsideSourceSupport,
            "outside-horizontal-support",
        ));
    }
    for interval in original
        .vertical
        .outside(retained.vertical)
        .into_iter()
        .flatten()
    {
        fragments.push(fragment(
            provenance,
            ContributionDomain {
                source_parameter: retained.source_parameter,
                horizontal: retained.horizontal,
                vertical: interval,
            },
            ContributionDisposition::OutsideSourceSupport,
            "outside-vertical-support",
        ));
    }
    fragments
}

fn split_depth_profile(
    provenance: &ContributionProvenance,
    overlap: ContributionDomain,
    profile: DepthProfileSegment,
    epsilon: f64,
    fragments: &mut Vec<ContributionFragment>,
) {
    let start_beyond = profile.candidate_minus_authority_start > epsilon;
    let end_beyond = profile.candidate_minus_authority_end > epsilon;
    if start_beyond == end_beyond {
        fragments.push(fragment(
            provenance,
            ContributionDomain {
                source_parameter: profile.source_parameter,
                ..overlap
            },
            if start_beyond {
                ContributionDisposition::RejectedBeyond
            } else {
                ContributionDisposition::RetainedNearer
            },
            if start_beyond {
                "candidate-beyond-authority"
            } else {
                "candidate-nearer-than-authority"
            },
        ));
        return;
    }

    let denominator =
        profile.candidate_minus_authority_end - profile.candidate_minus_authority_start;
    if denominator.abs() <= f64::EPSILON {
        fragments.push(fragment(
            provenance,
            ContributionDomain {
                source_parameter: profile.source_parameter,
                ..overlap
            },
            ContributionDisposition::UnresolvedFailOpen,
            "unstable-depth-crossing",
        ));
        return;
    }
    let fraction = (-profile.candidate_minus_authority_start / denominator).clamp(0.0, 1.0);
    let crossing = profile.source_parameter.start
        + fraction * (profile.source_parameter.end - profile.source_parameter.start);
    let Some(left) = FiniteInterval::new(profile.source_parameter.start, crossing) else {
        fragments.push(fragment(
            provenance,
            ContributionDomain {
                source_parameter: profile.source_parameter,
                ..overlap
            },
            ContributionDisposition::UnresolvedFailOpen,
            "crossing-at-source-range-boundary",
        ));
        return;
    };
    let Some(right) = FiniteInterval::new(crossing, profile.source_parameter.end) else {
        fragments.push(fragment(
            provenance,
            ContributionDomain {
                source_parameter: profile.source_parameter,
                ..overlap
            },
            ContributionDisposition::UnresolvedFailOpen,
            "crossing-at-source-range-boundary",
        ));
        return;
    };
    for (domain, beyond) in [(left, start_beyond), (right, end_beyond)] {
        fragments.push(fragment(
            provenance,
            ContributionDomain {
                source_parameter: domain,
                ..overlap
            },
            if beyond {
                ContributionDisposition::RejectedBeyond
            } else {
                ContributionDisposition::RetainedNearer
            },
            if beyond {
                "candidate-beyond-authority-after-split"
            } else {
                "candidate-nearer-than-authority-after-split"
            },
        ));
    }
}

fn fragment(
    provenance: &ContributionProvenance,
    domain: ContributionDomain,
    disposition: ContributionDisposition,
    reason: &'static str,
) -> ContributionFragment {
    ContributionFragment {
        provenance: provenance.clone(),
        uv_source_parameter: domain.source_parameter,
        domain,
        disposition,
        reason,
    }
}

/// Observes finite support before any depth comparison occurs.
pub fn observe_candidate_support(
    candidate: &CandidateSourceSupport,
    authority: &AuthorityOccurrence,
) -> SupportObservation {
    let Some((source_parameter, horizontal, vertical)) = candidate.finite_domains() else {
        let CandidateSourceSupport::Unresolved { reason } = candidate else {
            unreachable!("finite_domains and source support variant disagree")
        };
        return SupportObservation::UnresolvedSupport {
            reason: reason.clone(),
        };
    };

    let Some(horizontal_overlap) = horizontal.intersection(authority.view_horizontal) else {
        return SupportObservation::OutsideSourceSupport {
            candidate_source_parameter: source_parameter,
            reason: "outside-authorized-horizontal-interval",
        };
    };
    let Some(vertical_overlap) = vertical.intersection(authority.vertical) else {
        return SupportObservation::OutsideSourceSupport {
            candidate_source_parameter: source_parameter,
            reason: "outside-authorized-vertical-interval",
        };
    };
    let Some(source_parameter_overlap) = source_parameter.intersection(authority.source_parameter)
    else {
        return SupportObservation::OutsideSourceSupport {
            candidate_source_parameter: source_parameter,
            reason: "outside-authorized-source-parameter-interval",
        };
    };
    SupportObservation::Supported {
        candidate_source_parameter: source_parameter,
        source_parameter_overlap,
        horizontal_overlap,
        vertical_overlap,
        outside_source_parameter: source_parameter.outside(source_parameter_overlap),
        outside_horizontal: horizontal.outside(horizontal_overlap),
        outside_vertical: vertical.outside(vertical_overlap),
    }
}

/// Resolves authority from retained Doom order. Equal-order overlapping
/// occurrences are ambiguous and fail open rather than using proximity,
/// material, or incidental iteration order as a tie breaker.
pub fn resolve_ordered_authority(
    candidate: &CandidateSourceSupport,
    ledger: &[AuthorityOccurrence],
) -> OrderedAuthorityResolution {
    let Some((candidate_source_parameter, _, _)) = candidate.finite_domains() else {
        let CandidateSourceSupport::Unresolved { reason } = candidate else {
            unreachable!("finite_domains and source support variant disagree")
        };
        return OrderedAuthorityResolution::Unresolved {
            reason: format!("candidate-support-unresolved:{reason}"),
        };
    };

    let mut supported = ledger
        .iter()
        .filter_map(|authority| {
            let support = observe_candidate_support(candidate, authority);
            matches!(support, SupportObservation::Supported { .. }).then_some((authority, support))
        })
        .collect::<Vec<_>>();
    supported.sort_by(|(left, _), (right, _)| {
        left.order
            .cmp(&right.order)
            .then_with(|| left.identity.cmp(&right.identity))
    });

    let Some((first, support)) = supported.first() else {
        return OrderedAuthorityResolution::OutsideAllAuthority {
            candidate_source_parameter,
        };
    };
    if supported
        .get(1)
        .is_some_and(|(second, _)| second.order == first.order)
    {
        return OrderedAuthorityResolution::Unresolved {
            reason: format!("ambiguous-authority-order:{}", first.order),
        };
    }
    OrderedAuthorityResolution::Resolved {
        authority: (*first).clone(),
        support: support.clone(),
    }
}

/// Classifies comparable samples only inside the already-resolved finite
/// support. Smaller positive ray `t` is nearer. Equality within tolerance is
/// retained as nearer so numerical uncertainty cannot silently delete input.
pub fn classify_relational_depth(
    support: SupportObservation,
    _authority: &AuthorityOccurrence,
    samples: &[ComparisonSample],
    tolerance: RelationalTolerance,
    facing: CandidateFacingObservation,
) -> RelationalClassification {
    let mut nearer = false;
    let mut beyond = false;
    let mut conventions = samples.iter().map(|sample| sample.convention);
    let convention = conventions.next();
    if conventions.any(|other| Some(other) != convention) {
        return unresolved(
            support,
            "mixed-column-center-edge-conventions",
            tolerance,
            facing,
        );
    }

    let supported_observation = support.clone();
    let SupportObservation::Supported {
        source_parameter_overlap,
        horizontal_overlap,
        vertical_overlap,
        ..
    } = support
    else {
        return RelationalClassification {
            support,
            depth: None,
            facing,
            comparison_domain: "prepared-view-source-ray-t",
            tolerance,
        };
    };

    if samples.is_empty() {
        return unresolved(
            supported_observation.clone(),
            "missing-comparison-samples",
            tolerance,
            facing,
        );
    }

    for sample in samples {
        if !horizontal_overlap.contains(sample.horizontal)
            || !vertical_overlap.contains(sample.vertical)
            || !source_parameter_overlap.contains(sample.authority_source_parameter)
        {
            return unresolved(
                supported_observation.clone(),
                "sample-outside-finite-authority-support",
                tolerance,
                facing,
            );
        }
        let ComparisonSampleValue::Comparable {
            candidate_ray_t,
            authority_ray_t,
        } = sample.value
        else {
            return unresolved(
                supported_observation.clone(),
                match sample.value {
                    ComparisonSampleValue::Parallel => "parallel-ray-relation",
                    ComparisonSampleValue::BehindView => "behind-view-relation",
                    ComparisonSampleValue::NearPlane => "near-plane-relation",
                    ComparisonSampleValue::NonFinite => "non-finite-ray-relation",
                    ComparisonSampleValue::Comparable { .. } => unreachable!(),
                },
                tolerance,
                facing,
            );
        };
        if !candidate_ray_t.is_finite()
            || !authority_ray_t.is_finite()
            || candidate_ray_t < 0.0
            || authority_ray_t < 0.0
        {
            return unresolved(
                supported_observation.clone(),
                "invalid-ray-parameter",
                tolerance,
                facing,
            );
        }
        match (candidate_ray_t - authority_ray_t)
            .partial_cmp(&tolerance.ray_t_epsilon)
            .unwrap_or(Ordering::Equal)
        {
            Ordering::Greater => beyond = true,
            Ordering::Less | Ordering::Equal => nearer = true,
        }
    }

    let depth = match (nearer, beyond) {
        (true, false) => DepthObservation::Nearer,
        (false, true) => DepthObservation::Beyond,
        (true, true) => DepthObservation::Straddling,
        (false, false) => DepthObservation::Unresolved {
            reason: "no-comparable-result".to_owned(),
        },
    };
    RelationalClassification {
        support: supported_observation,
        depth: Some(depth),
        facing,
        comparison_domain: "prepared-view-source-ray-t",
        tolerance,
    }
}

fn unresolved(
    support: SupportObservation,
    reason: &str,
    tolerance: RelationalTolerance,
    facing: CandidateFacingObservation,
) -> RelationalClassification {
    RelationalClassification {
        support,
        depth: Some(DepthObservation::Unresolved {
            reason: reason.to_owned(),
        }),
        facing,
        comparison_domain: "prepared-view-source-ray-t",
        tolerance,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn interval(start: f64, end: f64) -> FiniteInterval {
        FiniteInterval::new(start, end).expect("valid interval")
    }

    fn wall(horizontal: FiniteInterval, vertical: FiniteInterval) -> CandidateSourceSupport {
        CandidateSourceSupport::WallSeg {
            source_seg: 7,
            source_parameter: interval(0.0, 1.0),
            view_horizontal: horizontal,
            vertical,
        }
    }

    fn authority(order: u32) -> AuthorityOccurrence {
        AuthorityOccurrence {
            identity: format!("sky-boundary-{order}"),
            order,
            source_parameter: interval(0.2, 0.8),
            view_horizontal: interval(0.25, 0.75),
            vertical: interval(0.2, 0.9),
        }
    }

    fn sample(horizontal: f64, candidate: f64, boundary: f64) -> ComparisonSample {
        ComparisonSample {
            horizontal,
            vertical: 0.5,
            authority_source_parameter: 0.5,
            convention: ComparisonConvention::ExplicitRay,
            value: ComparisonSampleValue::Comparable {
                candidate_ray_t: candidate,
                authority_ray_t: boundary,
            },
        }
    }

    fn classify(samples: &[ComparisonSample]) -> RelationalClassification {
        let candidate = wall(interval(0.3, 0.7), interval(0.3, 0.8));
        let boundary = authority(3);
        classify_relational_depth(
            observe_candidate_support(&candidate, &boundary),
            &boundary,
            samples,
            RelationalTolerance::new(0.001).unwrap(),
            CandidateFacingObservation {
                normal_dot_view: -1.0,
            },
        )
    }

    #[test]
    fn separates_nearer_beyond_and_straddling() {
        assert_eq!(
            classify(&[sample(0.4, 4.0, 8.0)]).depth,
            Some(DepthObservation::Nearer)
        );
        assert_eq!(
            classify(&[sample(0.4, 9.0, 8.0)]).depth,
            Some(DepthObservation::Beyond)
        );
        assert_eq!(
            classify(&[sample(0.4, 4.0, 8.0), sample(0.6, 9.0, 8.0)]).depth,
            Some(DepthObservation::Straddling)
        );
    }

    #[test]
    fn infinite_supporting_plane_cannot_classify_outside_finite_domain() {
        let candidate = wall(interval(0.8, 0.95), interval(0.3, 0.8));
        let boundary = authority(3);
        let support = observe_candidate_support(&candidate, &boundary);
        assert!(matches!(
            support,
            SupportObservation::OutsideSourceSupport { .. }
        ));
        let result = classify_relational_depth(
            support,
            &boundary,
            &[sample(0.9, 20.0, 2.0)],
            RelationalTolerance::new(0.001).unwrap(),
            CandidateFacingObservation {
                normal_dot_view: 1.0,
            },
        );
        assert_eq!(result.depth, None);
    }

    #[test]
    fn source_parameter_must_overlap_before_nearer_depth_can_be_observed() {
        let candidate = CandidateSourceSupport::WallSeg {
            source_seg: 7,
            source_parameter: interval(0.81, 0.95),
            view_horizontal: interval(0.3, 0.7),
            vertical: interval(0.3, 0.8),
        };
        let boundary = authority(3);
        let support = observe_candidate_support(&candidate, &boundary);
        assert_eq!(
            support,
            SupportObservation::OutsideSourceSupport {
                candidate_source_parameter: interval(0.81, 0.95),
                reason: "outside-authorized-source-parameter-interval",
            }
        );
        assert_eq!(
            classify_relational_depth(
                support,
                &boundary,
                &[sample(0.4, 1.0, 20.0)],
                RelationalTolerance::new(0.001).unwrap(),
                CandidateFacingObservation {
                    normal_dot_view: -1.0,
                },
            )
            .depth,
            None
        );
    }

    #[test]
    fn partial_support_retains_every_excluded_range_explicitly() {
        let candidate = CandidateSourceSupport::WallSeg {
            source_seg: 7,
            source_parameter: interval(0.0, 1.0),
            view_horizontal: interval(0.0, 1.0),
            vertical: interval(0.0, 1.0),
        };
        assert_eq!(
            observe_candidate_support(&candidate, &authority(3)),
            SupportObservation::Supported {
                candidate_source_parameter: interval(0.0, 1.0),
                source_parameter_overlap: interval(0.2, 0.8),
                horizontal_overlap: interval(0.25, 0.75),
                vertical_overlap: interval(0.2, 0.9),
                outside_source_parameter: [Some(interval(0.0, 0.2)), Some(interval(0.8, 1.0)),],
                outside_horizontal: [Some(interval(0.0, 0.25)), Some(interval(0.75, 1.0)),],
                outside_vertical: [Some(interval(0.0, 0.2)), Some(interval(0.9, 1.0)),],
            }
        );
    }

    #[test]
    fn plane_support_is_occurrence_local_not_sector_global() {
        let candidate = CandidateSourceSupport::PlaneOccurrence {
            source_subsector: 104,
            plane: PlaneKind::Ceiling,
            occurrence: 41,
            source_parameter: interval(0.0, 1.0),
            view_horizontal: interval(0.0, 0.2),
            vertical: interval(0.3, 0.8),
        };
        assert!(matches!(
            observe_candidate_support(&candidate, &authority(3)),
            SupportObservation::OutsideSourceSupport { .. }
        ));
    }

    #[test]
    fn ordered_ledger_not_proximity_selects_authority() {
        let candidate = wall(interval(0.3, 0.7), interval(0.3, 0.8));
        let later = authority(9);
        let earlier = authority(2);
        let resolved = resolve_ordered_authority(&candidate, &[later, earlier.clone()]);
        assert!(matches!(
            resolved,
            OrderedAuthorityResolution::Resolved { authority, .. } if authority == earlier
        ));
    }

    #[test]
    fn ambiguous_order_and_unresolved_support_fail_open() {
        let candidate = wall(interval(0.3, 0.7), interval(0.3, 0.8));
        let mut duplicate = authority(2);
        duplicate.identity = "different-source-same-order".to_owned();
        assert!(matches!(
            resolve_ordered_authority(&candidate, &[authority(2), duplicate]),
            OrderedAuthorityResolution::Unresolved { .. }
        ));
        assert!(matches!(
            resolve_ordered_authority(
                &CandidateSourceSupport::Unresolved {
                    reason: "missing-subsector-loop".to_owned()
                },
                &[authority(2)]
            ),
            OrderedAuthorityResolution::Unresolved { .. }
        ));
    }

    #[test]
    fn invalid_ray_relations_and_mixed_sample_conventions_fail_open() {
        for value in [
            ComparisonSampleValue::Parallel,
            ComparisonSampleValue::BehindView,
            ComparisonSampleValue::NearPlane,
            ComparisonSampleValue::NonFinite,
            ComparisonSampleValue::Comparable {
                candidate_ray_t: -1.0,
                authority_ray_t: 8.0,
            },
            ComparisonSampleValue::Comparable {
                candidate_ray_t: f64::NAN,
                authority_ray_t: 8.0,
            },
        ] {
            let mut invalid = sample(0.4, 4.0, 8.0);
            invalid.value = value;
            assert!(matches!(
                classify(&[invalid]).depth,
                Some(DepthObservation::Unresolved { .. })
            ));
        }

        assert!(matches!(
            classify(&[]).depth,
            Some(DepthObservation::Unresolved { .. })
        ));

        let mut edge = sample(0.6, 4.0, 8.0);
        edge.convention = ComparisonConvention::ColumnEdge;
        assert!(matches!(
            classify(&[sample(0.4, 4.0, 8.0), edge]).depth,
            Some(DepthObservation::Unresolved { .. })
        ));
    }

    #[test]
    fn facing_does_not_change_authoritative_depth_result() {
        let candidate = wall(interval(0.3, 0.7), interval(0.3, 0.8));
        let boundary = authority(3);
        let support = observe_candidate_support(&candidate, &boundary);
        let classify_facing = |normal_dot_view| {
            classify_relational_depth(
                support.clone(),
                &boundary,
                &[sample(0.4, 9.0, 8.0)],
                RelationalTolerance::new(0.001).unwrap(),
                CandidateFacingObservation { normal_dot_view },
            )
        };
        assert_eq!(classify_facing(-1.0).depth, classify_facing(1.0).depth);
    }

    fn provenance() -> ContributionProvenance {
        ContributionProvenance {
            source_identity: "synthetic-linedef-7-seg-9".to_owned(),
            sidedef_role: "upper".to_owned(),
            material_identity: "SYNTHETIC7".to_owned(),
        }
    }

    fn complete_domain() -> ContributionDomain {
        ContributionDomain {
            source_parameter: interval(0.0, 1.0),
            horizontal: interval(0.0, 1.0),
            vertical: interval(0.0, 1.0),
        }
    }

    fn fully_supported() -> SupportObservation {
        SupportObservation::Supported {
            candidate_source_parameter: interval(0.0, 1.0),
            source_parameter_overlap: interval(0.0, 1.0),
            horizontal_overlap: interval(0.0, 1.0),
            vertical_overlap: interval(0.0, 1.0),
            outside_source_parameter: [None, None],
            outside_horizontal: [None, None],
            outside_vertical: [None, None],
        }
    }

    fn profile(start: f64, end: f64, start_delta: f64, end_delta: f64) -> DepthProfileSegment {
        DepthProfileSegment {
            source_parameter: interval(start, end),
            candidate_minus_authority_start: start_delta,
            candidate_minus_authority_end: end_delta,
        }
    }

    #[test]
    fn whole_nearer_and_beyond_contributions_remain_distinct() {
        for (delta, expected) in [
            (-4.0, ContributionDisposition::RetainedNearer),
            (4.0, ContributionDisposition::RejectedBeyond),
        ] {
            let result = split_contribution(
                provenance(),
                complete_domain(),
                &fully_supported(),
                &[profile(0.0, 1.0, delta, delta)],
                RelationalTolerance::new(0.001).unwrap(),
            );
            assert!(result.is_conserved(1.0e-9));
            assert_eq!(result.fragments.len(), 1);
            assert_eq!(result.fragments[0].disposition, expected);
        }
    }

    #[test]
    fn straddling_depth_splits_deterministically_and_preserves_uv_progress() {
        let result = split_contribution(
            provenance(),
            complete_domain(),
            &fully_supported(),
            &[profile(0.0, 1.0, -2.0, 2.0)],
            RelationalTolerance::new(0.001).unwrap(),
        );
        assert!(result.is_conserved(1.0e-9));
        assert_eq!(result.fragments.len(), 2);
        assert_eq!(
            result.fragments[0].domain.source_parameter,
            interval(0.0, 0.5)
        );
        assert_eq!(
            result.fragments[1].domain.source_parameter,
            interval(0.5, 1.0)
        );
        assert_eq!(result.fragments[0].uv_source_parameter, interval(0.0, 0.5));
        assert_eq!(result.fragments[1].uv_source_parameter, interval(0.5, 1.0));
        assert_eq!(
            result.fragments[0].provenance,
            result.fragments[1].provenance
        );
        assert_eq!(
            result.fragments[0].disposition,
            ContributionDisposition::RetainedNearer
        );
        assert_eq!(
            result.fragments[1].disposition,
            ContributionDisposition::RejectedBeyond
        );
    }

    #[test]
    fn horizontal_and_vertical_authority_edges_produce_explicit_outside_slabs() {
        let support = SupportObservation::Supported {
            candidate_source_parameter: interval(0.0, 1.0),
            source_parameter_overlap: interval(0.0, 1.0),
            horizontal_overlap: interval(0.2, 0.8),
            vertical_overlap: interval(0.25, 0.75),
            outside_source_parameter: [None, None],
            outside_horizontal: [Some(interval(0.0, 0.2)), Some(interval(0.8, 1.0))],
            outside_vertical: [Some(interval(0.0, 0.25)), Some(interval(0.75, 1.0))],
        };
        let result = split_contribution(
            provenance(),
            complete_domain(),
            &support,
            &[profile(0.0, 1.0, -1.0, -1.0)],
            RelationalTolerance::new(0.001).unwrap(),
        );
        assert!(result.is_conserved(1.0e-9));
        assert_eq!(
            result
                .fragments
                .iter()
                .filter(|fragment| fragment.disposition
                    == ContributionDisposition::OutsideSourceSupport)
                .count(),
            4
        );
        assert_eq!(
            result
                .fragments
                .iter()
                .filter(|fragment| fragment.disposition == ContributionDisposition::RetainedNearer)
                .count(),
            1
        );
    }

    #[test]
    fn lazy_oversized_plane_cannot_borrow_nearer_classification_outside_occurrence() {
        let support = SupportObservation::Supported {
            candidate_source_parameter: interval(0.0, 1.0),
            source_parameter_overlap: interval(0.2, 0.7),
            horizontal_overlap: interval(0.1, 0.9),
            vertical_overlap: interval(0.0, 1.0),
            outside_source_parameter: [Some(interval(0.0, 0.2)), Some(interval(0.7, 1.0))],
            outside_horizontal: [Some(interval(0.0, 0.1)), Some(interval(0.9, 1.0))],
            outside_vertical: [None, None],
        };
        let result = split_contribution(
            ContributionProvenance {
                source_identity: "subsector-104-ceiling-occurrence-41".to_owned(),
                ..provenance()
            },
            complete_domain(),
            &support,
            &[profile(0.2, 0.7, -10.0, -10.0)],
            RelationalTolerance::new(0.001).unwrap(),
        );
        assert!(result.is_conserved(1.0e-9));
        assert!(result.fragments.iter().any(|fragment| {
            fragment.disposition == ContributionDisposition::RetainedNearer
                && fragment.domain.source_parameter == interval(0.2, 0.7)
        }));
        assert!(result.fragments.iter().any(|fragment| {
            fragment.disposition == ContributionDisposition::OutsideSourceSupport
        }));
    }

    #[test]
    fn supported_nearer_and_unsupported_beyond_regions_do_not_lend_authority() {
        let support = SupportObservation::Supported {
            candidate_source_parameter: interval(0.0, 1.0),
            source_parameter_overlap: interval(0.25, 0.75),
            horizontal_overlap: interval(0.0, 1.0),
            vertical_overlap: interval(0.0, 1.0),
            outside_source_parameter: [Some(interval(0.0, 0.25)), Some(interval(0.75, 1.0))],
            outside_horizontal: [None, None],
            outside_vertical: [None, None],
        };
        let result = split_contribution(
            provenance(),
            complete_domain(),
            &support,
            &[profile(0.25, 0.75, -1.0, 1.0)],
            RelationalTolerance::new(0.001).unwrap(),
        );
        assert!(result.is_conserved(1.0e-9));
        assert_eq!(
            result
                .fragments
                .iter()
                .filter(|fragment| fragment.disposition
                    == ContributionDisposition::OutsideSourceSupport)
                .count(),
            2
        );
        assert!(result
            .fragments
            .iter()
            .any(|fragment| fragment.disposition == ContributionDisposition::RetainedNearer));
        assert!(result
            .fragments
            .iter()
            .any(|fragment| fragment.disposition == ContributionDisposition::RejectedBeyond));
    }

    #[test]
    fn adjacent_plane_occurrences_do_not_share_support_by_sector_identity() {
        let first = CandidateSourceSupport::PlaneOccurrence {
            source_subsector: 104,
            plane: PlaneKind::Ceiling,
            occurrence: 41,
            source_parameter: interval(0.0, 0.4),
            view_horizontal: interval(0.3, 0.6),
            vertical: interval(0.3, 0.8),
        };
        let adjacent = CandidateSourceSupport::PlaneOccurrence {
            source_subsector: 105,
            plane: PlaneKind::Ceiling,
            occurrence: 42,
            source_parameter: interval(0.81, 1.0),
            view_horizontal: interval(0.3, 0.6),
            vertical: interval(0.3, 0.8),
        };
        assert!(matches!(
            observe_candidate_support(&first, &authority(3)),
            SupportObservation::Supported { .. }
        ));
        assert!(matches!(
            observe_candidate_support(&adjacent, &authority(3)),
            SupportObservation::OutsideSourceSupport { .. }
        ));
    }

    #[test]
    fn invalid_profile_fails_open_while_conserving_the_supported_domain() {
        let result = split_contribution(
            provenance(),
            complete_domain(),
            &fully_supported(),
            &[profile(0.0, 0.4, -1.0, -1.0), profile(0.5, 1.0, 1.0, 1.0)],
            RelationalTolerance::new(0.001).unwrap(),
        );
        assert!(result.is_conserved(1.0e-9));
        assert_eq!(result.fragments.len(), 1);
        assert_eq!(
            result.fragments[0].disposition,
            ContributionDisposition::UnresolvedFailOpen
        );
    }

    #[test]
    fn cutout_occurrence_never_becomes_solid_authority() {
        let candidate = wall(interval(0.3, 0.7), interval(0.3, 0.8));
        assert!(matches!(
            resolve_ordered_solid_authority(
                &candidate,
                &[(authority(1), AuthorityKind::CutoutNonSolid)]
            ),
            OrderedAuthorityResolution::OutsideAllAuthority { .. }
        ));
        assert!(matches!(
            resolve_ordered_solid_authority(
                &candidate,
                &[
                    (authority(1), AuthorityKind::CutoutNonSolid),
                    (authority(2), AuthorityKind::SolidCoverage),
                ]
            ),
            OrderedAuthorityResolution::Resolved { authority, .. } if authority.order == 2
        ));
    }

    #[test]
    fn ordered_disjoint_authorities_require_partitioned_composition() {
        let candidate = wall(interval(0.0, 1.0), interval(0.0, 1.0));
        let first = AuthorityOccurrence {
            identity: "first-authority".to_owned(),
            order: 1,
            source_parameter: interval(0.0, 0.5),
            view_horizontal: interval(0.0, 0.5),
            vertical: interval(0.0, 1.0),
        };
        let second = AuthorityOccurrence {
            identity: "second-authority".to_owned(),
            order: 2,
            source_parameter: interval(0.5, 1.0),
            view_horizontal: interval(0.5, 1.0),
            vertical: interval(0.0, 1.0),
        };
        let OrderedAuthorityResolution::Resolved { authority, support } =
            resolve_ordered_authority(&candidate, &[second.clone(), first.clone()])
        else {
            panic!("the first ordered authority should resolve");
        };
        assert_eq!(authority.identity, first.identity);

        let first_only = split_contribution(
            provenance(),
            complete_domain(),
            &support,
            &[profile(0.0, 0.5, -1.0, -1.0)],
            RelationalTolerance::new(0.001).unwrap(),
        );
        assert!(first_only.is_conserved(1.0e-9));
        assert!(first_only.fragments.iter().any(|fragment| {
            fragment.disposition == ContributionDisposition::OutsideSourceSupport
                && fragment.domain.source_parameter == interval(0.5, 1.0)
        }));
        assert!(matches!(
            observe_candidate_support(&candidate, &second),
            SupportObservation::Supported {
                source_parameter_overlap,
                ..
            } if source_parameter_overlap == interval(0.5, 1.0)
        ));

        // A single-authority classification conserves the candidate but cannot
        // distinguish genuinely unsupported space from space owned by a later
        // authority. Resolving that remainder requires ordered partitioned
        // composition, not another whole-candidate priority rule.
    }

    fn partition_authority(
        identity: &str,
        order: u32,
        source_start: f64,
        source_end: f64,
        delta_start: f64,
        delta_end: f64,
    ) -> OrderedPartitionAuthority {
        OrderedPartitionAuthority {
            authority: AuthorityOccurrence {
                identity: identity.to_owned(),
                order,
                source_parameter: interval(source_start, source_end),
                view_horizontal: interval(0.0, 1.0),
                vertical: interval(0.0, 1.0),
            },
            kind: AuthorityKind::SolidCoverage,
            depth_profiles: vec![profile(source_start, source_end, delta_start, delta_end)],
        }
    }

    #[test]
    fn ordered_partition_composition_resolves_later_owned_remainder() {
        let result = compose_ordered_partitions(
            provenance(),
            complete_domain(),
            &[
                partition_authority("later", 2, 0.5, 1.0, 1.0, 1.0),
                partition_authority("earlier", 1, 0.0, 0.5, -1.0, -1.0),
            ],
            RelationalTolerance::new(0.001).unwrap(),
        );

        assert!(result.is_conserved(1.0e-9));
        assert_eq!(result.steps.len(), 2);
        assert!(result.steps.iter().all(|step| step.conserved));
        assert_eq!(result.fragments.len(), 2);
        assert!(result.fragments.iter().any(|fragment| {
            fragment.domain.source_parameter == interval(0.0, 0.5)
                && fragment.disposition == ContributionDisposition::RetainedNearer
        }));
        assert!(result.fragments.iter().any(|fragment| {
            fragment.domain.source_parameter == interval(0.5, 1.0)
                && fragment.disposition == ContributionDisposition::RejectedBeyond
        }));
        assert!(!result.fragments.iter().any(|fragment| {
            fragment.disposition == ContributionDisposition::OutsideSourceSupport
        }));
    }

    #[test]
    fn overlapping_authority_order_is_semantically_observable_and_monotonic() {
        let earlier_near = partition_authority("near", 1, 0.2, 0.8, -1.0, -1.0);
        let later_far = partition_authority("far", 2, 0.5, 1.0, 1.0, 1.0);
        let near_first = compose_ordered_partitions(
            provenance(),
            complete_domain(),
            &[later_far.clone(), earlier_near.clone()],
            RelationalTolerance::new(0.001).unwrap(),
        );
        let far_first = compose_ordered_partitions(
            provenance(),
            complete_domain(),
            &[
                OrderedPartitionAuthority {
                    authority: AuthorityOccurrence {
                        order: 2,
                        ..earlier_near.authority
                    },
                    ..earlier_near
                },
                OrderedPartitionAuthority {
                    authority: AuthorityOccurrence {
                        order: 1,
                        ..later_far.authority
                    },
                    ..later_far
                },
            ],
            RelationalTolerance::new(0.001).unwrap(),
        );

        assert!(near_first.is_conserved(1.0e-9));
        assert!(far_first.is_conserved(1.0e-9));
        assert!(near_first.fragments.iter().any(|fragment| {
            fragment.domain.source_parameter == interval(0.2, 0.8)
                && fragment.disposition == ContributionDisposition::RetainedNearer
        }));
        assert!(far_first.fragments.iter().any(|fragment| {
            fragment.domain.source_parameter == interval(0.5, 1.0)
                && fragment.disposition == ContributionDisposition::RejectedBeyond
        }));
        assert!(!far_first.fragments.iter().any(|fragment| {
            fragment.disposition == ContributionDisposition::RetainedNearer
                && fragment.domain.source_parameter.start >= 0.5
        }));
    }

    #[test]
    fn unresolved_region_is_not_reopened_by_later_authority() {
        let invalid = OrderedPartitionAuthority {
            authority: AuthorityOccurrence {
                identity: "invalid-first".to_owned(),
                order: 1,
                source_parameter: interval(0.0, 0.5),
                view_horizontal: interval(0.0, 1.0),
                vertical: interval(0.0, 1.0),
            },
            kind: AuthorityKind::SolidCoverage,
            depth_profiles: vec![profile(0.0, 0.2, -1.0, -1.0)],
        };
        let result = compose_ordered_partitions(
            provenance(),
            complete_domain(),
            &[invalid, partition_authority("later", 2, 0.0, 1.0, 1.0, 1.0)],
            RelationalTolerance::new(0.001).unwrap(),
        );

        assert!(result.is_conserved(1.0e-9));
        assert!(result.fragments.iter().any(|fragment| {
            fragment.domain.source_parameter == interval(0.0, 0.5)
                && fragment.disposition == ContributionDisposition::UnresolvedFailOpen
        }));
        assert!(!result.fragments.iter().any(|fragment| {
            fragment.domain.source_parameter.start < 0.5
                && fragment.disposition == ContributionDisposition::RejectedBeyond
        }));
    }

    #[test]
    fn unsupported_lazy_mapper_domain_stays_unresolved() {
        let result = compose_ordered_partitions(
            provenance(),
            complete_domain(),
            &[partition_authority("bounded", 1, 0.25, 0.75, -1.0, -1.0)],
            RelationalTolerance::new(0.001).unwrap(),
        );
        assert!(result.is_conserved(1.0e-9));
        assert!(result.fragments.iter().any(|fragment| {
            fragment.disposition == ContributionDisposition::UnresolvedFailOpen
                && fragment.reason == "outside-all-ordered-authority"
        }));
    }

    #[test]
    fn cutout_authority_is_skipped_without_consuming_remaining_domain() {
        let mut cutout = partition_authority("cutout", 1, 0.0, 1.0, 1.0, 1.0);
        cutout.kind = AuthorityKind::CutoutNonSolid;
        let result = compose_ordered_partitions(
            provenance(),
            complete_domain(),
            &[
                cutout,
                partition_authority("solid", 2, 0.0, 1.0, -1.0, -1.0),
            ],
            RelationalTolerance::new(0.001).unwrap(),
        );
        assert!(result.is_conserved(1.0e-9));
        assert_eq!(result.steps[0].classified_now, 0.0);
        assert_eq!(result.fragments.len(), 1);
        assert_eq!(
            result.fragments[0].disposition,
            ContributionDisposition::RetainedNearer
        );
    }

    #[test]
    fn equal_order_overlapping_authorities_fail_open() {
        let result = compose_ordered_partitions(
            provenance(),
            complete_domain(),
            &[
                partition_authority("a", 1, 0.0, 0.75, -1.0, -1.0),
                partition_authority("b", 1, 0.5, 1.0, 1.0, 1.0),
            ],
            RelationalTolerance::new(0.001).unwrap(),
        );
        assert!(result.is_conserved(1.0e-9));
        assert_eq!(result.fragments.len(), 1);
        assert_eq!(
            result.fragments[0].reason,
            "ambiguous-overlapping-authority-order"
        );
    }

    #[test]
    fn equal_order_overlap_is_detected_across_disjoint_sibling() {
        let result = compose_ordered_partitions(
            provenance(),
            complete_domain(),
            &[
                partition_authority("a", 1, 0.0, 0.75, -1.0, -1.0),
                partition_authority("b", 1, 0.8, 1.0, -1.0, -1.0),
                partition_authority("c", 1, 0.5, 0.7, 1.0, 1.0),
            ],
            RelationalTolerance::new(0.001).unwrap(),
        );
        assert!(result.is_conserved(1.0e-9));
        assert_eq!(result.fragments.len(), 1);
        assert_eq!(
            result.fragments[0].reason,
            "ambiguous-overlapping-authority-order"
        );
    }
}
