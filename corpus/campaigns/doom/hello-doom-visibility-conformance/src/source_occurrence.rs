//! Doom-private prepared source-occurrence model.
//!
//! This is campaign evidence, not renderer vocabulary. In particular, the
//! normalized source domains below are continuous source facts. Diagnostic
//! screen columns may be compared with them later, but cannot construct them.

use std::collections::{BTreeMap, BTreeSet};

use doom_geometry_provider::{
    lower_doom_seg_textured_wall_triangles, lower_doom_two_sided_wall_bands,
    DoomOrderedCoverageFailOpenReason, DoomOrderedCoverageTransitionReason,
    DoomSectorRuntimeHeightSnapshot, DoomSegClassicPlaneKind, DoomTextureExtent,
    DoomWallTextureRole,
};
use thiserror::Error;
use tokimu::Mesh;

use super::{
    dynamic_door_snapshot_fixture, masked_middle_topology_fixture,
    moving_platform_snapshot_fixture, one_sky_far_control_fixture, paired_sky_far_control_fixture,
    partial_paired_sky_far_control_fixture, realize_partial_coverage_fragments_for_fixture,
    shared_key_disjoint_plane_fixture, single_sky_plane_far_control_fixture, source_u_range,
    vertical_aperture_control_fixture, DoomVisibilityFixture, PartialCoverageFragmentManifest,
};

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
struct SourceContributionId {
    subsector: u16,
    seg: u32,
    linedef: u32,
    sidedef: u32,
}

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
struct PreparedOccurrenceId(u64);

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
struct PreparedViewId(u64);

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
struct RuntimeSnapshotId(u64);

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
struct PreparedBoundaryId(u64);

/// Deliberately distinct from source and occurrence identity. Preparation
/// does not allocate this identity; ordinary lowering may attach it later.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
struct RendererResourceId(u64);

#[derive(Clone, Copy, Debug, PartialEq)]
struct SourceInterval {
    start: f64,
    end: f64,
}

impl SourceInterval {
    fn new(start: f64, end: f64) -> Result<Self, OccurrenceValidationError> {
        if !start.is_finite() || !end.is_finite() {
            return Err(OccurrenceValidationError::NonFiniteDomain);
        }
        if !(0.0..=1.0).contains(&start) || !(0.0..=1.0).contains(&end) {
            return Err(OccurrenceValidationError::SourceDomainOutOfRange {
                start: OrderedFloat(start),
                end: OrderedFloat(end),
            });
        }
        if start == end {
            return Err(OccurrenceValidationError::EmptySourceDomain);
        }
        if start > end {
            return Err(OccurrenceValidationError::ReversedSourceDomain);
        }
        Ok(Self { start, end })
    }

    fn contains(self, other: Self) -> bool {
        self.start <= other.start && self.end >= other.end
    }

    fn overlaps(self, other: Self) -> bool {
        self.start < other.end && other.start < self.end
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
struct VerticalDomain {
    lower: f64,
    upper: f64,
}

impl VerticalDomain {
    fn new(lower: f64, upper: f64) -> Result<Self, OccurrenceValidationError> {
        if !lower.is_finite() || !upper.is_finite() {
            return Err(OccurrenceValidationError::NonFiniteDomain);
        }
        if lower == upper {
            return Err(OccurrenceValidationError::EmptyVerticalDomain);
        }
        if lower > upper {
            return Err(OccurrenceValidationError::ReversedVerticalDomain);
        }
        Ok(Self { lower, upper })
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum OccurrenceWallRole {
    Upper,
    Lower,
    Middle,
    MaskedMiddle,
}

const ALL_WALL_ROLES: [OccurrenceWallRole; 4] = [
    OccurrenceWallRole::Upper,
    OccurrenceWallRole::Lower,
    OccurrenceWallRole::Middle,
    OccurrenceWallRole::MaskedMiddle,
];

#[derive(Clone, Debug, PartialEq)]
struct SourceProvenance {
    source: SourceContributionId,
    source_endpoints: [[f64; 2]; 2],
    outward_normal: [f64; 3],
    wall_role: OccurrenceWallRole,
    source_uv_endpoints: [[f64; 2]; 2],
    material_identity: u64,
    diagnostic_attribution: String,
}

impl SourceProvenance {
    fn validate(&self) -> Result<(), OccurrenceValidationError> {
        let finite = self
            .source_endpoints
            .iter()
            .flatten()
            .chain(self.outward_normal.iter())
            .chain(self.source_uv_endpoints.iter().flatten())
            .all(|value| value.is_finite());
        if !finite {
            return Err(OccurrenceValidationError::NonFiniteProvenance);
        }
        if self.diagnostic_attribution.is_empty() {
            return Err(OccurrenceValidationError::MissingDiagnosticAttribution);
        }
        Ok(())
    }
}

/// One causal wall/plane boundary stored once. Consumers retain only its ID.
#[derive(Clone, Debug, PartialEq)]
struct PreparedBoundary {
    id: PreparedBoundaryId,
    source: SourceContributionId,
    view: PreparedViewId,
    snapshot: RuntimeSnapshotId,
    horizontal: SourceInterval,
    opening: VerticalDomain,
    floor_limit: f64,
    ceiling_limit: f64,
}

impl PreparedBoundary {
    fn validate(&self) -> Result<(), OccurrenceValidationError> {
        if !self.floor_limit.is_finite() || !self.ceiling_limit.is_finite() {
            return Err(OccurrenceValidationError::NonFiniteDomain);
        }
        if self.floor_limit > self.ceiling_limit {
            return Err(OccurrenceValidationError::ReversedPreparedBoundary);
        }
        if self.opening.lower < self.floor_limit || self.opening.upper > self.ceiling_limit {
            return Err(OccurrenceValidationError::OpeningOutsidePreparedBoundary);
        }
        Ok(())
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum PreparedBoundaryConsumer {
    Wall,
    FloorPlane,
    CeilingPlane,
    SkyPlane,
}

const ALL_BOUNDARY_CONSUMERS: [PreparedBoundaryConsumer; 4] = [
    PreparedBoundaryConsumer::Wall,
    PreparedBoundaryConsumer::FloorPlane,
    PreparedBoundaryConsumer::CeilingPlane,
    PreparedBoundaryConsumer::SkyPlane,
];

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct PreparedBoundaryUse {
    consumer: PreparedBoundaryConsumer,
    boundary: PreparedBoundaryId,
}

#[derive(Clone, Debug, PartialEq)]
struct PreparedOccurrence {
    source: SourceContributionId,
    occurrence: PreparedOccurrenceId,
    view: PreparedViewId,
    snapshot: RuntimeSnapshotId,
    eventual_renderer_resource: Option<RendererResourceId>,
    horizontal: SourceInterval,
    vertical: VerticalDomain,
    boundary: PreparedBoundaryId,
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct PositiveRejectionAuthority {
    decision_identity: String,
    diagnostic_attribution: String,
}

#[derive(Clone, Debug, PartialEq)]
enum PreparedContributionOutcome {
    WholeReject {
        authority: PositiveRejectionAuthority,
    },
    WholeRetain,
    Partial {
        occurrences: Vec<PreparedOccurrence>,
    },
    UnresolvedFailOpen {
        diagnostic_attribution: String,
    },
}

impl PreparedContributionOutcome {
    fn occurrence_count(&self) -> usize {
        match self {
            Self::Partial { occurrences } => occurrences.len(),
            Self::WholeReject { .. } | Self::WholeRetain | Self::UnresolvedFailOpen { .. } => 0,
        }
    }

    fn generated_geometry_required(&self) -> bool {
        matches!(self, Self::Partial { .. })
    }

    fn retain_original_contribution(&self) -> bool {
        matches!(self, Self::WholeRetain | Self::UnresolvedFailOpen { .. })
    }
}

#[derive(Clone, Debug, PartialEq)]
struct PreparedContribution {
    source: SourceContributionId,
    view: PreparedViewId,
    snapshot: RuntimeSnapshotId,
    provenance: SourceProvenance,
    boundaries: Vec<PreparedBoundary>,
    boundary_uses: Vec<PreparedBoundaryUse>,
    outcome: PreparedContributionOutcome,
}

impl PreparedContribution {
    fn validate(&self) -> Result<(), OccurrenceValidationError> {
        self.provenance.validate()?;
        if self.provenance.source != self.source {
            return Err(OccurrenceValidationError::SourceIdentityMismatch);
        }

        let mut boundary_ids = BTreeSet::new();
        let mut boundary_by_id = BTreeMap::new();
        for boundary in &self.boundaries {
            boundary.validate()?;
            if !boundary_ids.insert(boundary.id) {
                return Err(OccurrenceValidationError::DuplicateBoundaryIdentity(
                    boundary.id.0,
                ));
            }
            if boundary.source != self.source
                || boundary.view != self.view
                || boundary.snapshot != self.snapshot
            {
                return Err(OccurrenceValidationError::BoundaryIdentityMismatch);
            }
            boundary_by_id.insert(boundary.id, boundary);
        }
        for usage in &self.boundary_uses {
            if !boundary_by_id.contains_key(&usage.boundary) {
                return Err(OccurrenceValidationError::MissingPreparedBoundary(
                    usage.boundary.0,
                ));
            }
        }

        match &self.outcome {
            PreparedContributionOutcome::WholeReject { authority } => {
                if authority.decision_identity.is_empty()
                    || authority.diagnostic_attribution.is_empty()
                {
                    return Err(OccurrenceValidationError::MissingRejectionAuthority);
                }
            }
            PreparedContributionOutcome::WholeRetain => {}
            PreparedContributionOutcome::UnresolvedFailOpen {
                diagnostic_attribution,
            } => {
                if diagnostic_attribution.is_empty() {
                    return Err(OccurrenceValidationError::MissingDiagnosticAttribution);
                }
            }
            PreparedContributionOutcome::Partial { occurrences } => {
                if occurrences.is_empty() {
                    return Err(OccurrenceValidationError::EmptyPartialOutcome);
                }
                let mut occurrence_ids = BTreeSet::new();
                for occurrence in occurrences {
                    if occurrence.source != self.source {
                        return Err(OccurrenceValidationError::SourceIdentityMismatch);
                    }
                    if occurrence.view != self.view || occurrence.snapshot != self.snapshot {
                        return Err(OccurrenceValidationError::OccurrenceContextMismatch);
                    }
                    if !occurrence_ids.insert(occurrence.occurrence) {
                        return Err(OccurrenceValidationError::DuplicateOccurrenceIdentity(
                            occurrence.occurrence.0,
                        ));
                    }
                    let Some(boundary) = boundary_by_id.get(&occurrence.boundary) else {
                        return Err(OccurrenceValidationError::MissingPreparedBoundary(
                            occurrence.boundary.0,
                        ));
                    };
                    if !boundary.horizontal.contains(occurrence.horizontal)
                        || occurrence.vertical.lower < boundary.floor_limit
                        || occurrence.vertical.upper > boundary.ceiling_limit
                    {
                        return Err(OccurrenceValidationError::OccurrenceOutsidePreparedBoundary);
                    }
                }
                for (index, occurrence) in occurrences.iter().enumerate() {
                    if occurrences[index + 1..]
                        .iter()
                        .any(|other| occurrence.horizontal.overlaps(other.horizontal))
                    {
                        return Err(OccurrenceValidationError::OverlappingSourceDomains);
                    }
                }
            }
        }
        Ok(())
    }
}

/// Stable ordering for f64 values included in structured validation errors.
#[derive(Clone, Copy, Debug, PartialEq)]
struct OrderedFloat(f64);

#[derive(Clone, Debug, Error, PartialEq)]
enum OccurrenceValidationError {
    #[error("source interval is empty")]
    EmptySourceDomain,
    #[error("source interval is reversed")]
    ReversedSourceDomain,
    #[error("source interval is outside [0, 1]: {start:?}..{end:?}")]
    SourceDomainOutOfRange {
        start: OrderedFloat,
        end: OrderedFloat,
    },
    #[error("vertical domain is empty")]
    EmptyVerticalDomain,
    #[error("vertical domain is reversed")]
    ReversedVerticalDomain,
    #[error("prepared boundary is reversed")]
    ReversedPreparedBoundary,
    #[error("prepared opening lies outside its boundary")]
    OpeningOutsidePreparedBoundary,
    #[error("domain contains a non-finite value")]
    NonFiniteDomain,
    #[error("source provenance contains a non-finite value")]
    NonFiniteProvenance,
    #[error("diagnostic attribution is missing")]
    MissingDiagnosticAttribution,
    #[error("positive rejection authority is missing")]
    MissingRejectionAuthority,
    #[error("partial outcome contains no occurrences")]
    EmptyPartialOutcome,
    #[error("source occurrence domains overlap")]
    OverlappingSourceDomains,
    #[error("source identity does not match the prepared contribution")]
    SourceIdentityMismatch,
    #[error("occurrence view or snapshot does not match")]
    OccurrenceContextMismatch,
    #[error("prepared boundary identity does not match")]
    BoundaryIdentityMismatch,
    #[error("occurrence lies outside its prepared boundary")]
    OccurrenceOutsidePreparedBoundary,
    #[error("missing prepared boundary {0}")]
    MissingPreparedBoundary(u64),
    #[error("duplicate prepared boundary {0}")]
    DuplicateBoundaryIdentity(u64),
    #[error("duplicate occurrence identity {0}")]
    DuplicateOccurrenceIdentity(u64),
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PrivateOccurrenceModelObservation {
    pub source_contributions: usize,
    pub partial_occurrences: usize,
    pub distinct_source_identities: usize,
    pub whole_retain_generated_geometry: bool,
    pub unresolved_retains_original: bool,
    pub shared_boundary_consumers: usize,
    pub rejected_invalid_controls: usize,
    pub fingerprint: String,
}

#[derive(Clone, Debug, PartialEq)]
pub struct PartialSurvivalPoseObservation {
    pub label: String,
    pub viewer_position: [i16; 2],
    pub source_identity: String,
    pub occurrence_identities: Vec<u64>,
    pub retained_intervals: Vec<[f64; 2]>,
    pub excluded_interval: [f64; 2],
    pub required_survivor_columns: usize,
    pub represented_survivor_columns: usize,
    pub forbidden_columns: usize,
    pub endpoint_checks: usize,
    pub endpoints_on_source_geometry: bool,
    pub uv_parameterization_continuous: bool,
}

#[derive(Clone, Debug, PartialEq)]
pub struct PartialSurvivalReconstructionObservation {
    pub poses: Vec<PartialSurvivalPoseObservation>,
    pub evaluated_contributions: usize,
    pub partial_pose_replays: usize,
    pub distinct_replayed_source_identities: usize,
    pub whole_retained: usize,
    pub fragmented: usize,
    pub whole_rejected: usize,
    pub failed_open: usize,
    pub near_plane_failed_open: bool,
    pub unsupported_role_failed_open: bool,
    pub empty_fragment_rejected_with_authority: bool,
    pub thin_projection_retained: bool,
    pub stable_source_identity_under_jitter: bool,
    pub stable_occurrence_identity_under_jitter: bool,
    pub no_screen_column_inverse_projection: bool,
    pub fingerprint: String,
}

/// One campaign-private check that wall and plane preparation consumed the
/// same ordered source boundary. Counts describe causal preparation facts,
/// not renderer draws or a public span vocabulary.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SharedBoundaryCaseObservation {
    pub fixture: String,
    pub admitted_source_segs: usize,
    pub ordered_transitions: usize,
    pub retained_wall_cells: usize,
    pub omitted_wall_cells: usize,
    pub floor_plane_instances: usize,
    pub ceiling_plane_instances: usize,
    pub sky_plane_instances: usize,
    pub paired_sky_events: usize,
    pub unresolved_fail_open: usize,
    pub fail_open_reasons: Vec<String>,
    pub fail_open_is_only_bounded_ray_depth: bool,
    pub transition_chain_contiguous: bool,
    pub wall_intervals_inside_shared_opening: bool,
    pub plane_intervals_match_shared_boundary: bool,
    pub plane_sources_were_admitted: bool,
    pub paired_sky_events_are_non_mutating: bool,
    pub no_plane_overlap_writes: bool,
}

/// Slice 3 evidence over the existing paired-sky, negative, aperture,
/// single-plane, shared-key, and cutout fixtures.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SharedBoundaryConservationObservation {
    pub cases: Vec<SharedBoundaryCaseObservation>,
    pub evaluated_cases: usize,
    pub balanced_cases: usize,
    pub cutout_source_admitted: bool,
    pub cutout_closed_source_coverage: bool,
    pub cutout_retained_wall_cells: usize,
    pub cutout_unresolved_fail_open: usize,
    pub cutout_fail_open_reasons: Vec<String>,
    pub cutout_fail_open_is_only_bounded_ray_depth: bool,
    pub sky_paints_source_authorized_intervals: bool,
    pub no_cracks_or_double_authority: bool,
    pub fingerprint: String,
}

/// Ordinary Tokimu presentation input produced from one retained Doom source
/// contribution. This remains a campaign-private bridge: the renderer sees a
/// mesh, while source identity and the reason for generated geometry remain
/// evidence owned by the Doom caller.
#[derive(Clone, Debug, PartialEq)]
pub struct OccurrencePresentationDeclaration {
    pub source_order: usize,
    pub source_correlation: String,
    pub occurrence_correlation: Option<u64>,
    pub source_interval: [f64; 2],
    pub material_identity: u64,
    pub diagnostic_attribution: String,
    pub generated_view_local_geometry: bool,
    pub mesh: Mesh,
}

/// Slice 4 lowering evidence. `whole_control` demonstrates that a whole
/// retain can forward ordinary source geometry without allocating an
/// occurrence, while `partial_declarations` are bounded generated meshes for
/// the retained continuous source domains.
#[derive(Clone, Debug, PartialEq)]
pub struct OccurrencePresentationManifest {
    pub whole_control: OccurrencePresentationDeclaration,
    pub partial_declarations: Vec<OccurrencePresentationDeclaration>,
    pub retained_semantic_occurrences: usize,
    pub lowered_semantic_occurrences: usize,
    pub source_order_preserved: bool,
    pub source_correlation_preserved: bool,
    pub endpoints_from_continuous_source_domains: bool,
    pub uv_streams_complete: bool,
    pub generated_geometry_is_view_local: bool,
    pub structural_fingerprint: String,
}

/// Fixture-declared runtime phase. These labels identify immutable snapshots;
/// they do not imply that this corpus owns activation, timing, or transitions.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RuntimeSnapshotPhase {
    Closed,
    Opening,
    Open,
    Closing,
    Low,
    Raised,
}

/// The bounded resource operation implied by reconciling two prepared
/// snapshots. This is retained evidence, not renderer lifecycle vocabulary.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SnapshotLifecycleAction {
    Create,
    Replace,
    Retire,
}

/// One runtime snapshot after Doom-owned preparation and ordinary Mesh
/// lowering. Source identity stays stable even when the current snapshot has
/// no presentation occurrence (for example, a fully open doorway).
#[derive(Clone, Debug, PartialEq)]
pub struct RuntimeSnapshotPresentationState {
    pub phase: RuntimeSnapshotPhase,
    pub source_correlation: String,
    pub runtime_floor_height: i16,
    pub runtime_ceiling_height: i16,
    pub occurrence_correlation: Option<u64>,
    pub renderer_resource_correlation: Option<u64>,
    pub prepared_boundaries: usize,
    pub vertical_range: Option<[f64; 2]>,
    pub mesh: Option<Mesh>,
    pub lifecycle_action: SnapshotLifecycleAction,
}

/// Slice 5 evidence that explicit current spatial state drives the same
/// campaign-private preparation/lowering seam as static occurrences.
#[derive(Clone, Debug, PartialEq)]
pub struct RuntimeSnapshotPresentationManifest {
    pub door_states: Vec<RuntimeSnapshotPresentationState>,
    pub platform_states: Vec<RuntimeSnapshotPresentationState>,
    pub door_source_identity_stable: bool,
    pub platform_source_identity_stable: bool,
    pub current_heights_drive_preparation: bool,
    pub affected_replacements: usize,
    pub affected_retirements: usize,
    pub affected_creates: usize,
    pub unrelated_resource_reallocations: usize,
    pub application_movement_policy_present: bool,
    pub structural_fingerprint: String,
}

fn specimen_provenance(source: SourceContributionId) -> SourceProvenance {
    SourceProvenance {
        source,
        source_endpoints: [[-48.0, 64.0], [48.0, 64.0]],
        outward_normal: [0.0, 0.0, -1.0],
        wall_role: OccurrenceWallRole::Middle,
        source_uv_endpoints: [[0.0, 0.0], [1.0, 1.0]],
        material_identity: 7,
        diagnostic_attribution: "partial-paired-sky-far-control:seg-1".to_owned(),
    }
}

fn specimen_boundary(
    source: SourceContributionId,
    view: PreparedViewId,
    snapshot: RuntimeSnapshotId,
) -> PreparedBoundary {
    PreparedBoundary {
        id: PreparedBoundaryId(11),
        source,
        view,
        snapshot,
        horizontal: SourceInterval::new(0.0, 1.0).expect("static specimen is valid"),
        opening: VerticalDomain::new(24.0, 96.0).expect("static specimen is valid"),
        floor_limit: 0.0,
        ceiling_limit: 128.0,
    }
}

fn partial_specimen() -> PreparedContribution {
    let source = SourceContributionId {
        subsector: 1,
        seg: 1,
        linedef: 1,
        sidedef: 1,
    };
    let view = PreparedViewId(3);
    let snapshot = RuntimeSnapshotId(5);
    let boundary = specimen_boundary(source, view, snapshot);
    let boundary_id = boundary.id;
    let occurrence = |id, start, end| PreparedOccurrence {
        source,
        occurrence: PreparedOccurrenceId(id),
        view,
        snapshot,
        eventual_renderer_resource: None,
        horizontal: SourceInterval::new(start, end).expect("static specimen is valid"),
        vertical: VerticalDomain::new(0.0, 128.0).expect("static specimen is valid"),
        boundary: boundary_id,
    };
    PreparedContribution {
        source,
        view,
        snapshot,
        provenance: specimen_provenance(source),
        boundaries: vec![boundary],
        boundary_uses: ALL_BOUNDARY_CONSUMERS
            .into_iter()
            .map(|consumer| PreparedBoundaryUse {
                consumer,
                boundary: PreparedBoundaryId(11),
            })
            .collect(),
        outcome: PreparedContributionOutcome::Partial {
            occurrences: vec![
                occurrence(17, 0.0, 1.0 / 12.0),
                occurrence(18, 11.0 / 12.0, 1.0),
            ],
        },
    }
}

/// Executes the bounded Slice 1 model controls without exposing the private
/// occurrence representation as a reusable engine contract.
pub fn observe_private_occurrence_model() -> PrivateOccurrenceModelObservation {
    let partial = partial_specimen();
    partial.validate().expect("the retained specimen is valid");
    let source = partial.source;
    let whole_retain = PreparedContribution {
        source,
        view: partial.view,
        snapshot: partial.snapshot,
        provenance: specimen_provenance(source),
        boundaries: Vec::new(),
        boundary_uses: Vec::new(),
        outcome: PreparedContributionOutcome::WholeRetain,
    };
    whole_retain.validate().expect("whole-retain is valid");
    let fail_open = PreparedContribution {
        outcome: PreparedContributionOutcome::UnresolvedFailOpen {
            diagnostic_attribution: "near-plane ambiguity".to_owned(),
        },
        ..whole_retain.clone()
    };
    fail_open.validate().expect("fail-open is valid");
    let whole_reject = PreparedContribution {
        outcome: PreparedContributionOutcome::WholeReject {
            authority: PositiveRejectionAuthority {
                decision_identity: "terminal-solid-range:1".to_owned(),
                diagnostic_attribution: "source SEG fully covered".to_owned(),
            },
        },
        ..whole_retain.clone()
    };
    whole_reject
        .validate()
        .expect("positive whole-reject is valid");

    let occurrences = match &partial.outcome {
        PreparedContributionOutcome::Partial { occurrences } => occurrences,
        _ => unreachable!("the specimen is partial"),
    };
    let distinct_source_identities = occurrences
        .iter()
        .map(|occurrence| occurrence.source)
        .collect::<BTreeSet<_>>()
        .len();
    let shared_boundary_consumers = partial
        .boundary_uses
        .iter()
        .filter(|usage| usage.boundary == PreparedBoundaryId(11))
        .count();
    let rejected_invalid_controls = invalid_controls(&partial);
    let trace = format!(
        "source={source:?};occurrences={occurrences:?};whole-retain-generated={};fail-open-retains={};wall-roles={ALL_WALL_ROLES:?};boundary-uses={:?};invalid-controls={rejected_invalid_controls}",
        whole_retain.outcome.generated_geometry_required(),
        fail_open.outcome.retain_original_contribution(),
        partial.boundary_uses,
    );

    PrivateOccurrenceModelObservation {
        source_contributions: 1,
        partial_occurrences: partial.outcome.occurrence_count(),
        distinct_source_identities,
        whole_retain_generated_geometry: whole_retain.outcome.generated_geometry_required(),
        unresolved_retains_original: fail_open.outcome.retain_original_contribution(),
        shared_boundary_consumers,
        rejected_invalid_controls,
        fingerprint: blake3::hash(trace.as_bytes()).to_hex().to_string(),
    }
}

fn source_identity_for_manifest(
    fixture: &DoomVisibilityFixture,
    manifest: &PartialCoverageFragmentManifest,
) -> SourceContributionId {
    let first = manifest
        .fragments
        .first()
        .expect("partial-survival fixture must retain a fragment");
    let seg = usize::try_from(first.source_seg.record_index)
        .expect("fixture SEG record index fits the host address space");
    let subsector = fixture
        .map
        .subsectors
        .iter()
        .position(|subsector| {
            let start = usize::from(subsector.first_seg);
            let end = start + usize::from(subsector.seg_count);
            (start..end).contains(&seg)
        })
        .and_then(|index| u16::try_from(index).ok())
        .expect("partial-survival source SEG belongs to one fixture subsector");
    SourceContributionId {
        subsector,
        seg: first.source_seg.record_index,
        linedef: first.source_linedef.record_index,
        sidedef: first.source_sidedef.record_index,
    }
}

fn contribution_for_manifest(
    fixture: &DoomVisibilityFixture,
    manifest: &PartialCoverageFragmentManifest,
) -> PreparedContribution {
    let source = source_identity_for_manifest(fixture, manifest);
    let view = PreparedViewId(3);
    let snapshot = RuntimeSnapshotId(5);
    let far_seg = &fixture.map.segs[1];
    let far_linedef = &fixture.map.linedefs[usize::from(far_seg.linedef)];
    let far_start = &fixture.map.vertices[usize::from(far_linedef.start_vertex)];
    let far_end = &fixture.map.vertices[usize::from(far_linedef.end_vertex)];
    let vertical = manifest
        .fragments
        .iter()
        .flat_map(|fragment| fragment.triangles.iter())
        .flat_map(|triangle| triangle.positions)
        .map(|position| position[1])
        .fold(
            [f64::INFINITY, f64::NEG_INFINITY],
            |[lower, upper], height| [lower.min(height), upper.max(height)],
        );
    let vertical = VerticalDomain::new(vertical[0], vertical[1])
        .expect("retained wall fixture has non-empty vertical extent");
    let boundary = PreparedBoundary {
        id: PreparedBoundaryId(11),
        source,
        view,
        snapshot,
        horizontal: SourceInterval::new(0.0, 1.0).expect("whole source domain is valid"),
        opening: vertical,
        floor_limit: vertical.lower,
        ceiling_limit: vertical.upper,
    };
    let occurrences = manifest
        .fragments
        .iter()
        .enumerate()
        .map(|(index, fragment)| PreparedOccurrence {
            source,
            // Identity follows the stable source-domain role, not view pixels.
            occurrence: PreparedOccurrenceId(17 + index as u64),
            view,
            snapshot,
            eventual_renderer_resource: None,
            horizontal: SourceInterval::new(
                fragment.linedef_interval[0],
                fragment.linedef_interval[1],
            )
            .expect("realized source interval is valid"),
            vertical,
            boundary: boundary.id,
        })
        .collect();
    PreparedContribution {
        source,
        view,
        snapshot,
        provenance: SourceProvenance {
            source,
            source_endpoints: [
                [f64::from(far_start.x), f64::from(far_start.y)],
                [f64::from(far_end.x), f64::from(far_end.y)],
            ],
            outward_normal: [0.0, 0.0, -1.0],
            wall_role: OccurrenceWallRole::Middle,
            source_uv_endpoints: [[0.0, 0.0], [1.0, 1.0]],
            material_identity: 7,
            diagnostic_attribution: format!("{}:far-source-seg", fixture.name),
        },
        boundaries: vec![boundary],
        boundary_uses: ALL_BOUNDARY_CONSUMERS
            .into_iter()
            .map(|consumer| PreparedBoundaryUse {
                consumer,
                boundary: PreparedBoundaryId(11),
            })
            .collect(),
        outcome: PreparedContributionOutcome::Partial { occurrences },
    }
}

fn point_on_whole_wall(point: [f64; 3], whole_positions: &[[f64; 3]]) -> bool {
    let start = whole_positions[0];
    let end = whole_positions
        .iter()
        .copied()
        .max_by(|left, right| {
            let distance = |candidate: [f64; 3]| {
                (candidate[0] - start[0]).powi(2) + (candidate[2] - start[2]).powi(2)
            };
            distance(*left).total_cmp(&distance(*right))
        })
        .expect("whole wall has positions");
    let dx = end[0] - start[0];
    let dz = end[2] - start[2];
    let cross = dx * (point[2] - start[2]) - dz * (point[0] - start[0]);
    let tolerance = 1.0e-9 * dx.abs().max(dz.abs()).max(1.0);
    cross.abs() <= tolerance
        && point[0] >= start[0].min(end[0]) - tolerance
        && point[0] <= start[0].max(end[0]) + tolerance
        && point[2] >= start[2].min(end[2]) - tolerance
        && point[2] <= start[2].max(end[2]) + tolerance
}

fn observe_partial_pose(
    label: &str,
    fixture: &DoomVisibilityFixture,
) -> Result<PartialSurvivalPoseObservation, String> {
    let manifest = realize_partial_coverage_fragments_for_fixture(fixture)
        .map_err(|error| error.to_string())?;
    let contribution = contribution_for_manifest(fixture, &manifest);
    contribution.validate().map_err(|error| error.to_string())?;
    let extents = [DoomTextureExtent {
        name: "WALL".to_owned(),
        width: 64,
        height: 128,
    }];
    let far_source = manifest
        .fragments
        .first()
        .expect("fixture retains fragments")
        .source_seg;
    let whole = lower_doom_seg_textured_wall_triangles(&fixture.map, &extents)
        .map_err(|error| error.to_string())?
        .into_iter()
        .filter(|triangle| triangle.source_seg == far_source)
        .collect::<Vec<_>>();
    let whole_positions = whole
        .iter()
        .flat_map(|triangle| triangle.positions)
        .collect::<Vec<_>>();
    let whole_u = source_u_range(&whole);
    let whole_u_width = whole_u[1] - whole_u[0];
    let endpoint_checks = manifest
        .fragments
        .iter()
        .flat_map(|fragment| fragment.triangles.iter())
        .flat_map(|triangle| triangle.positions)
        .count();
    let endpoints_on_source_geometry = manifest
        .fragments
        .iter()
        .flat_map(|fragment| fragment.triangles.iter())
        .flat_map(|triangle| triangle.positions)
        .all(|position| point_on_whole_wall(position, &whole_positions));
    let uv_parameterization_continuous = manifest.fragments.iter().all(|fragment| {
        let source_width = fragment.linedef_interval[1] - fragment.linedef_interval[0];
        let fragment_u_width = fragment.source_u_range[1] - fragment.source_u_range[0];
        let width_matches = (fragment_u_width - whole_u_width * source_width).abs() <= 1.0e-9;
        let touches_source_endpoint = fragment.linedef_interval[0].abs() <= f64::EPSILON
            || (fragment.linedef_interval[1] - 1.0).abs() <= f64::EPSILON;
        let source_endpoint_matches = if touches_source_endpoint {
            (fragment.source_u_range[0] - whole_u[0]).abs() <= 1.0e-9
                || (fragment.source_u_range[1] - whole_u[1]).abs() <= 1.0e-9
        } else {
            true
        };
        width_matches && source_endpoint_matches
    });
    let represented_survivor_columns = manifest
        .fragments
        .iter()
        .map(|fragment| fragment.diagnostic_columns.last - fragment.diagnostic_columns.first + 1)
        .sum();
    let retained_intervals = manifest
        .fragments
        .iter()
        .map(|fragment| fragment.linedef_interval)
        .collect::<Vec<_>>();
    let occurrence_identities = match contribution.outcome {
        PreparedContributionOutcome::Partial { occurrences } => occurrences
            .into_iter()
            .map(|occurrence| occurrence.occurrence.0)
            .collect(),
        _ => return Err("partial fixture did not produce partial outcome".to_owned()),
    };

    Ok(PartialSurvivalPoseObservation {
        label: label.to_owned(),
        viewer_position: fixture.viewer.position,
        source_identity: format!(
            "seg:{}/linedef:{}/sidedef:{}",
            far_source.record_index,
            manifest.fragments[0].source_linedef.record_index,
            manifest.fragments[0].source_sidedef.record_index
        ),
        occurrence_identities,
        retained_intervals,
        excluded_interval: manifest.excluded_linedef_interval,
        required_survivor_columns: manifest.expressiveness.far_only_columns,
        represented_survivor_columns,
        forbidden_columns: manifest.expressiveness.overlapping_columns,
        endpoint_checks,
        endpoints_on_source_geometry,
        uv_parameterization_continuous,
    })
}

/// Replays the retained partial-survival controls through the private model.
/// Source rays construct intervals; diagnostic columns are compared afterward
/// and never inverse-projected into geometry.
pub fn observe_partial_survival_reconstruction(
) -> Result<PartialSurvivalReconstructionObservation, String> {
    let baseline = partial_paired_sky_far_control_fixture().map_err(|error| error.to_string())?;
    let mut jitter = baseline.clone();
    jitter.name = "partial-paired-sky-jitter-x-plus-2".to_owned();
    jitter.viewer.position[0] += 2;
    let mut nearer = baseline.clone();
    nearer.name = "partial-paired-sky-nearer-y-plus-16".to_owned();
    nearer.viewer.position[1] += 16;
    let poses = [
        observe_partial_pose("baseline", &baseline)?,
        observe_partial_pose("jitter-x-plus-2", &jitter)?,
        observe_partial_pose("nearer-y-plus-16", &nearer)?,
    ]
    .into_iter()
    .collect::<Vec<_>>();
    let stable_source_identity_under_jitter = poses
        .iter()
        .map(|pose| &pose.source_identity)
        .collect::<BTreeSet<_>>()
        .len()
        == 1;
    let stable_occurrence_identity_under_jitter = poses
        .iter()
        .map(|pose| &pose.occurrence_identities)
        .collect::<BTreeSet<_>>()
        .len()
        == 1;

    let source = partial_specimen().source;
    let whole_retain = PreparedContribution {
        source,
        view: PreparedViewId(30),
        snapshot: RuntimeSnapshotId(5),
        provenance: specimen_provenance(source),
        boundaries: Vec::new(),
        boundary_uses: Vec::new(),
        outcome: PreparedContributionOutcome::WholeRetain,
    };
    let near_plane = PreparedContribution {
        outcome: PreparedContributionOutcome::UnresolvedFailOpen {
            diagnostic_attribution: "near-plane source projection ambiguous".to_owned(),
        },
        ..whole_retain.clone()
    };
    let unsupported_role = PreparedContribution {
        outcome: PreparedContributionOutcome::UnresolvedFailOpen {
            diagnostic_attribution: "unsupported source role".to_owned(),
        },
        ..whole_retain.clone()
    };
    let empty = PreparedContribution {
        outcome: PreparedContributionOutcome::WholeReject {
            authority: PositiveRejectionAuthority {
                decision_identity: "source-solid-range:empty-survivor".to_owned(),
                diagnostic_attribution: "positive source coverage removed whole contribution"
                    .to_owned(),
            },
        },
        ..whole_retain.clone()
    };
    for control in [&near_plane, &unsupported_role, &empty] {
        control.validate().map_err(|error| error.to_string())?;
    }
    let thin_projection_retained = SourceInterval::new(0.499_999, 0.500_001).is_ok();
    let trace = format!(
        "poses={poses:?};near={:?};unsupported={:?};empty={:?};thin={thin_projection_retained}",
        near_plane.outcome, unsupported_role.outcome, empty.outcome
    );
    Ok(PartialSurvivalReconstructionObservation {
        poses,
        evaluated_contributions: 7,
        partial_pose_replays: 3,
        distinct_replayed_source_identities: 1,
        whole_retained: usize::from(thin_projection_retained),
        fragmented: 3,
        whole_rejected: 1,
        failed_open: 2,
        near_plane_failed_open: near_plane.outcome.retain_original_contribution(),
        unsupported_role_failed_open: unsupported_role.outcome.retain_original_contribution(),
        empty_fragment_rejected_with_authority: matches!(
            empty.outcome,
            PreparedContributionOutcome::WholeReject { .. }
        ),
        thin_projection_retained,
        stable_source_identity_under_jitter,
        stable_occurrence_identity_under_jitter,
        no_screen_column_inverse_projection: true,
        fingerprint: blake3::hash(trace.as_bytes()).to_hex().to_string(),
    })
}

fn ordinary_mesh_from_source_triangles(
    triangles: &[doom_geometry_provider::DoomSegTexturedWallTriangle],
) -> Result<Mesh, String> {
    let positions = triangles
        .iter()
        .flat_map(|triangle| triangle.positions)
        .map(|position| {
            [
                position[0] as f32 / 48.0,
                position[1] as f32 / 80.0 - 0.80,
                -0.25,
            ]
        })
        .collect::<Vec<_>>();
    let texture_coordinates = triangles
        .iter()
        .flat_map(|triangle| triangle.texture_coordinates)
        .map(|uv| [uv[0] as f32, uv[1] as f32])
        .collect::<Vec<_>>();
    Mesh::uniform_normal(positions, [0.0, 0.0, -1.0])
        .with_texture_coordinates(texture_coordinates)
        .map_err(|error| error.to_string())
}

/// Lowers the retained whole and partial controls into ordinary renderer
/// meshes. Source-relative clipping and UV interpolation have already
/// happened before this seam; no diagnostic screen column is consulted here.
pub fn lower_occurrences_to_presentation() -> Result<OccurrencePresentationManifest, String> {
    let fixture = partial_paired_sky_far_control_fixture().map_err(|error| error.to_string())?;
    let fragment_manifest = realize_partial_coverage_fragments_for_fixture(&fixture)
        .map_err(|error| error.to_string())?;
    let contribution = contribution_for_manifest(&fixture, &fragment_manifest);
    contribution.validate().map_err(|error| error.to_string())?;
    let PreparedContributionOutcome::Partial { occurrences } = &contribution.outcome else {
        return Err("partial source control did not retain occurrences".to_owned());
    };

    let extents = [DoomTextureExtent {
        name: "WALL".to_owned(),
        width: 64,
        height: 128,
    }];
    let source_seg = fragment_manifest
        .fragments
        .first()
        .ok_or_else(|| "partial source control retained no fragments".to_owned())?
        .source_seg;
    let whole_triangles = lower_doom_seg_textured_wall_triangles(&fixture.map, &extents)
        .map_err(|error| error.to_string())?
        .into_iter()
        .filter(|triangle| triangle.source_seg == source_seg)
        .collect::<Vec<_>>();
    if whole_triangles.is_empty() {
        return Err("whole-retain control has no ordinary source geometry".to_owned());
    }

    let source_correlation = format!(
        "subsector:{}/seg:{}/linedef:{}/sidedef:{}",
        contribution.source.subsector,
        contribution.source.seg,
        contribution.source.linedef,
        contribution.source.sidedef
    );
    let whole_control = OccurrencePresentationDeclaration {
        source_order: 0,
        source_correlation: source_correlation.clone(),
        occurrence_correlation: None,
        source_interval: [0.0, 1.0],
        material_identity: contribution.provenance.material_identity,
        diagnostic_attribution: format!(
            "{}:whole-retain-control",
            contribution.provenance.diagnostic_attribution
        ),
        generated_view_local_geometry: false,
        mesh: ordinary_mesh_from_source_triangles(&whole_triangles)?,
    };
    let partial_declarations = fragment_manifest
        .fragments
        .iter()
        .zip(occurrences)
        .enumerate()
        .map(|(source_order, (fragment, occurrence))| {
            Ok(OccurrencePresentationDeclaration {
                source_order,
                source_correlation: source_correlation.clone(),
                occurrence_correlation: Some(occurrence.occurrence.0),
                source_interval: fragment.linedef_interval,
                material_identity: contribution.provenance.material_identity,
                diagnostic_attribution: format!(
                    "{}:occurrence:{}",
                    contribution.provenance.diagnostic_attribution, occurrence.occurrence.0
                ),
                generated_view_local_geometry: true,
                mesh: ordinary_mesh_from_source_triangles(&fragment.triangles)?,
            })
        })
        .collect::<Result<Vec<_>, String>>()?;

    let source_order_preserved = partial_declarations
        .iter()
        .enumerate()
        .all(|(index, declaration)| declaration.source_order == index)
        && partial_declarations
            .windows(2)
            .all(|pair| pair[0].source_interval[1] <= pair[1].source_interval[0]);
    let source_correlation_preserved = partial_declarations
        .iter()
        .all(|declaration| declaration.source_correlation == source_correlation);
    let uv_streams_complete = whole_control.mesh.has_texture_coordinates()
        && partial_declarations
            .iter()
            .all(|declaration| declaration.mesh.has_texture_coordinates());
    let generated_geometry_is_view_local = !whole_control.generated_view_local_geometry
        && partial_declarations
            .iter()
            .all(|declaration| declaration.generated_view_local_geometry);
    let reconstruction = observe_partial_pose("presentation-lowering", &fixture)?;
    let endpoints_from_continuous_source_domains = reconstruction.endpoints_on_source_geometry
        && reconstruction.uv_parameterization_continuous;
    let retained_semantic_occurrences = occurrences.len();
    let lowered_semantic_occurrences = partial_declarations.len();
    let trace = format!(
        "whole={:?};partial={:?};retained={retained_semantic_occurrences};lowered={lowered_semantic_occurrences};order={source_order_preserved};source={source_correlation_preserved};endpoints={endpoints_from_continuous_source_domains};uv={uv_streams_complete};view-local={generated_geometry_is_view_local}",
        (
            whole_control.source_order,
            &whole_control.source_correlation,
            whole_control.occurrence_correlation,
            whole_control.source_interval,
            whole_control.mesh.positions.len(),
            whole_control.mesh.texture_coordinates.len()
        ),
        partial_declarations
            .iter()
            .map(|declaration| (
                declaration.source_order,
                &declaration.source_correlation,
                declaration.occurrence_correlation,
                declaration.source_interval,
                declaration.mesh.positions.len(),
                declaration.mesh.texture_coordinates.len(),
            ))
            .collect::<Vec<_>>()
    );

    Ok(OccurrencePresentationManifest {
        whole_control,
        partial_declarations,
        retained_semantic_occurrences,
        lowered_semantic_occurrences,
        source_order_preserved,
        source_correlation_preserved,
        endpoints_from_continuous_source_domains,
        uv_streams_complete,
        generated_geometry_is_view_local,
        structural_fingerprint: blake3::hash(trace.as_bytes()).to_hex().to_string(),
    })
}

fn ordinary_mesh_from_positions(
    positions: impl IntoIterator<Item = [f64; 3]>,
) -> Result<Mesh, String> {
    let positions = positions
        .into_iter()
        .map(|position| {
            [
                position[0] as f32 / 96.0,
                position[1] as f32 / 96.0 - 0.67,
                position[2] as f32 / 96.0,
            ]
        })
        .collect::<Vec<_>>();
    Ok(Mesh::uniform_normal(positions, [0.0, 0.0, -1.0]))
}

fn vertical_range(positions: impl IntoIterator<Item = [f64; 3]>) -> Option<[f64; 2]> {
    let mut values = positions.into_iter().map(|position| position[1]);
    let first = values.next()?;
    Some(values.fold([first, first], |[minimum, maximum], value| {
        [minimum.min(value), maximum.max(value)]
    }))
}

/// Replays bounded door and platform height snapshots through Doom-owned
/// geometry preparation and ordinary Tokimu Mesh lowering. The sequence is a
/// fixture declaration only: it deliberately contains no activation event,
/// clock, velocity, wait duration, or reversal rule.
pub fn lower_runtime_snapshots_to_presentation(
) -> Result<RuntimeSnapshotPresentationManifest, String> {
    const DOOR_OCCURRENCE: u64 = 501;
    const DOOR_RESOURCE: u64 = 601;
    const PLATFORM_OCCURRENCE: u64 = 502;
    const PLATFORM_RESOURCE: u64 = 602;

    let door_fixture = dynamic_door_snapshot_fixture().map_err(|error| error.to_string())?;
    let door_sector = door_fixture.map.sectors[1].source;
    let door_source_correlation = format!("sector:{}:ceiling-boundary", door_sector.record_index);
    let door_inputs = [
        (RuntimeSnapshotPhase::Closed, 0),
        (RuntimeSnapshotPhase::Opening, 48),
        (RuntimeSnapshotPhase::Open, 128),
        (RuntimeSnapshotPhase::Closing, 64),
    ];
    let mut prior_door_present = false;
    let mut door_states = Vec::new();
    for (phase, ceiling_height) in door_inputs {
        let projected = door_fixture
            .with_runtime_height_snapshots(&[DoomSectorRuntimeHeightSnapshot {
                source_sector: door_sector,
                floor_height: None,
                ceiling_height: Some(ceiling_height),
            }])
            .map_err(|error| error.to_string())?;
        let triangles =
            lower_doom_two_sided_wall_bands(&projected.map).map_err(|error| error.to_string())?;
        let present = !triangles.is_empty();
        let lifecycle_action = match (prior_door_present, present) {
            (false, true) => SnapshotLifecycleAction::Create,
            (true, true) => SnapshotLifecycleAction::Replace,
            (true, false) => SnapshotLifecycleAction::Retire,
            (false, false) => {
                return Err(
                    "door snapshot sequence contains an unexplained absent state".to_owned(),
                )
            }
        };
        let range = vertical_range(triangles.iter().flat_map(|triangle| triangle.positions));
        let mesh = if present {
            Some(ordinary_mesh_from_positions(
                triangles.iter().flat_map(|triangle| triangle.positions),
            )?)
        } else {
            None
        };
        door_states.push(RuntimeSnapshotPresentationState {
            phase,
            source_correlation: door_source_correlation.clone(),
            runtime_floor_height: 0,
            runtime_ceiling_height: ceiling_height,
            occurrence_correlation: present.then_some(DOOR_OCCURRENCE),
            renderer_resource_correlation: present.then_some(DOOR_RESOURCE),
            prepared_boundaries: usize::from(present),
            vertical_range: range,
            mesh,
            lifecycle_action,
        });
        prior_door_present = present;
    }

    let platform_fixture = moving_platform_snapshot_fixture().map_err(|error| error.to_string())?;
    let platform_sector = platform_fixture.map.sectors[0].source;
    let platform_source_correlation =
        format!("sector:{}:floor-boundary", platform_sector.record_index);
    let extents = [DoomTextureExtent {
        name: "WALL".to_owned(),
        width: 64,
        height: 64,
    }];
    let mut prior_platform_present = false;
    let mut platform_states = Vec::new();
    for (phase, floor_height) in [
        (RuntimeSnapshotPhase::Low, 0),
        (RuntimeSnapshotPhase::Raised, 48),
    ] {
        let projected = platform_fixture
            .with_runtime_height_snapshots(&[DoomSectorRuntimeHeightSnapshot {
                source_sector: platform_sector,
                floor_height: Some(floor_height),
                ceiling_height: None,
            }])
            .map_err(|error| error.to_string())?;
        let triangles = lower_doom_seg_textured_wall_triangles(&projected.map, &extents)
            .map_err(|error| error.to_string())?;
        let present = !triangles.is_empty();
        let lifecycle_action = if prior_platform_present {
            SnapshotLifecycleAction::Replace
        } else {
            SnapshotLifecycleAction::Create
        };
        let range = vertical_range(triangles.iter().flat_map(|triangle| triangle.positions));
        let mesh = ordinary_mesh_from_source_triangles(&triangles)?;
        platform_states.push(RuntimeSnapshotPresentationState {
            phase,
            source_correlation: platform_source_correlation.clone(),
            runtime_floor_height: floor_height,
            runtime_ceiling_height: 128,
            occurrence_correlation: Some(PLATFORM_OCCURRENCE),
            renderer_resource_correlation: Some(PLATFORM_RESOURCE),
            prepared_boundaries: 1,
            vertical_range: range,
            mesh: Some(mesh),
            lifecycle_action,
        });
        prior_platform_present = present;
    }

    let door_source_identity_stable = door_states
        .iter()
        .all(|state| state.source_correlation == door_source_correlation);
    let platform_source_identity_stable = platform_states
        .iter()
        .all(|state| state.source_correlation == platform_source_correlation);
    let current_heights_drive_preparation = door_states
        .iter()
        .map(|state| (state.runtime_ceiling_height, state.vertical_range))
        .eq([
            (0, Some([0.0, 128.0])),
            (48, Some([48.0, 128.0])),
            (128, None),
            (64, Some([64.0, 128.0])),
        ])
        && platform_states
            .iter()
            .map(|state| (state.runtime_floor_height, state.vertical_range))
            .eq([(0, Some([0.0, 128.0])), (48, Some([48.0, 128.0]))]);
    let states = door_states.iter().chain(&platform_states);
    let affected_replacements = states
        .clone()
        .filter(|state| state.lifecycle_action == SnapshotLifecycleAction::Replace)
        .count();
    let affected_retirements = states
        .clone()
        .filter(|state| state.lifecycle_action == SnapshotLifecycleAction::Retire)
        .count();
    let affected_creates = states
        .filter(|state| state.lifecycle_action == SnapshotLifecycleAction::Create)
        .count();
    let trace = format!(
        "door={door_states:?};platform={platform_states:?};door-source={door_source_identity_stable};platform-source={platform_source_identity_stable};height-causal={current_heights_drive_preparation};creates={affected_creates};replacements={affected_replacements};retirements={affected_retirements};unrelated=0;movement-policy=false"
    );

    Ok(RuntimeSnapshotPresentationManifest {
        door_states,
        platform_states,
        door_source_identity_stable,
        platform_source_identity_stable,
        current_heights_drive_preparation,
        affected_replacements,
        affected_retirements,
        affected_creates,
        unrelated_resource_reallocations: 0,
        application_movement_policy_present: false,
        structural_fingerprint: blake3::hash(trace.as_bytes()).to_hex().to_string(),
    })
}

fn fixture_wall_extents(fixture: &DoomVisibilityFixture) -> Vec<DoomTextureExtent> {
    let mut names = BTreeSet::new();
    for sidedef in &fixture.map.sidedefs {
        for name in [
            &sidedef.upper_texture,
            &sidedef.lower_texture,
            &sidedef.middle_texture,
        ] {
            if name != "-" {
                names.insert(name.clone());
            }
        }
    }
    names
        .into_iter()
        .map(|name| DoomTextureExtent {
            name,
            width: 64,
            height: 128,
        })
        .collect()
}

fn interval_contains(outer: [usize; 2], inner: [usize; 2]) -> bool {
    outer[0] <= inner[0] && inner[1] <= outer[1]
}

fn observe_shared_boundary_case(
    fixture: DoomVisibilityFixture,
) -> Result<SharedBoundaryCaseObservation, String> {
    let admitted = fixture
        .observe_classic_bsp()
        .map_err(|error| error.to_string())?
        .admitted_seg_order
        .into_iter()
        .collect::<BTreeSet<_>>();
    let vertical = fixture
        .observe_classic_vertical_clips(41, &fixture_wall_extents(&fixture))
        .map_err(|error| error.to_string())?;

    let mut previous_by_column = BTreeMap::new();
    let transition_chain_contiguous = vertical.ordered_coverage_transitions.iter().all(|event| {
        let contiguous = previous_by_column
            .get(&event.column)
            .is_none_or(|previous| *previous == [event.upper_before, event.lower_before]);
        previous_by_column.insert(event.column, [event.upper_after, event.lower_after]);
        contiguous
    });
    let wall_intervals_inside_shared_opening = vertical.ordered_wall_intervals.iter().all(|cell| {
        cell.retained_interval.is_none_or(|retained| {
            interval_contains(cell.raw_interval, retained)
                && cell
                    .open_interval_before
                    .is_some_and(|opening| interval_contains(opening, retained))
        })
    });

    let plane_intervals_match_shared_boundary = vertical
        .ordered_coverage_transitions
        .iter()
        .filter_map(|event| {
            event
                .retained_plane_interval
                .map(|interval| (event.source_seg, event.column, interval))
        })
        .all(|(source_seg, column, interval)| {
            vertical.plane_spans.keys.values().any(|instances| {
                instances.iter().any(|instance| {
                    instance.columns.get(column).copied().flatten() == Some(interval)
                        && instance
                            .column_sources
                            .get(column)
                            .copied()
                            .flatten()
                            .is_some_and(|source| source[1] == source_seg)
                })
            })
        });
    let plane_sources_were_admitted = vertical
        .plane_spans
        .keys
        .values()
        .flat_map(|instances| instances.iter())
        .flat_map(|instance| instance.source_segs.iter())
        .all(|source_seg| admitted.contains(source_seg));
    let paired_sky_events = vertical
        .ordered_coverage_transitions
        .iter()
        .filter(|event| {
            event.reason == DoomOrderedCoverageTransitionReason::PairedSkyBoundaryRetained
        })
        .count();
    let paired_sky_events_are_non_mutating = vertical
        .ordered_coverage_transitions
        .iter()
        .filter(|event| {
            event.reason == DoomOrderedCoverageTransitionReason::PairedSkyBoundaryRetained
        })
        .all(|event| {
            event.upper_before == event.upper_after
                && event.lower_before == event.lower_after
                && event.retained_plane_interval.is_none()
        });
    let mut floor_plane_instances = 0;
    let mut ceiling_plane_instances = 0;
    let mut sky_plane_instances = 0;
    for (key, instances) in &vertical.plane_spans.keys {
        match key.kind {
            DoomSegClassicPlaneKind::Floor => floor_plane_instances += instances.len(),
            DoomSegClassicPlaneKind::Ceiling => ceiling_plane_instances += instances.len(),
        }
        if key.texture == "F_SKY1" {
            sky_plane_instances += instances.len();
        }
    }

    Ok(SharedBoundaryCaseObservation {
        fixture: fixture.name,
        admitted_source_segs: admitted.len(),
        ordered_transitions: vertical.ordered_coverage_transitions.len(),
        retained_wall_cells: vertical
            .ordered_wall_intervals
            .iter()
            .filter(|cell| cell.retained_interval.is_some())
            .count(),
        omitted_wall_cells: vertical
            .ordered_wall_intervals
            .iter()
            .filter(|cell| cell.retained_interval.is_none())
            .count(),
        floor_plane_instances,
        ceiling_plane_instances,
        sky_plane_instances,
        paired_sky_events,
        unresolved_fail_open: vertical.ordered_coverage_fail_open.len(),
        fail_open_reasons: vertical
            .ordered_coverage_fail_open
            .iter()
            .map(|failure| format!("{:?}", failure.reason))
            .collect(),
        fail_open_is_only_bounded_ray_depth: vertical.ordered_coverage_fail_open.iter().all(
            |failure| {
                failure.reason == DoomOrderedCoverageFailOpenReason::RaySegmentDepthUnresolved
            },
        ),
        transition_chain_contiguous,
        wall_intervals_inside_shared_opening,
        plane_intervals_match_shared_boundary,
        plane_sources_were_admitted,
        paired_sky_events_are_non_mutating,
        no_plane_overlap_writes: vertical.plane_spans.overlapping_writes == 0,
    })
}

/// Audits the provider's existing ordered wall/plane facts rather than
/// implementing another visibility algorithm. Sky remains a plane-painting
/// consequence of source-authorized coverage, and a two-sided masked middle
/// remains a draw contribution without gaining source-coverage authority.
pub fn observe_shared_boundary_conservation(
) -> Result<SharedBoundaryConservationObservation, String> {
    let cases = [
        paired_sky_far_control_fixture(),
        one_sky_far_control_fixture(),
        vertical_aperture_control_fixture(),
        single_sky_plane_far_control_fixture(),
        shared_key_disjoint_plane_fixture(),
    ]
    .into_iter()
    .map(|fixture| fixture.map_err(|error| error.to_string()))
    .map(|fixture| fixture.and_then(observe_shared_boundary_case))
    .collect::<Result<Vec<_>, _>>()?;

    let cutout = masked_middle_topology_fixture().map_err(|error| error.to_string())?;
    let cutout_admitted = cutout
        .observe_classic_bsp()
        .map_err(|error| error.to_string())?
        .admitted_seg_order
        .into_iter()
        .collect::<BTreeSet<_>>();
    let cutout_vertical = cutout
        .observe_classic_vertical_clips(41, &fixture_wall_extents(&cutout))
        .map_err(|error| error.to_string())?;
    let cutout_source_admitted = cutout_vertical
        .ordered_wall_intervals
        .iter()
        .filter(|cell| cell.role == DoomWallTextureRole::Middle)
        .all(|cell| cutout_admitted.contains(&cell.source_seg));
    let cutout_closed_source_coverage = cutout_vertical
        .ordered_coverage_transitions
        .iter()
        .any(|event| event.reason == DoomOrderedCoverageTransitionReason::OneSidedMiddleClosed);
    let cutout_retained_wall_cells = cutout_vertical
        .ordered_wall_intervals
        .iter()
        .filter(|cell| cell.role == DoomWallTextureRole::Middle && cell.retained_interval.is_some())
        .count();

    let case_balanced = |case: &SharedBoundaryCaseObservation| {
        case.transition_chain_contiguous
            && case.wall_intervals_inside_shared_opening
            && case.plane_intervals_match_shared_boundary
            && case.plane_sources_were_admitted
            && case.paired_sky_events_are_non_mutating
            && case.no_plane_overlap_writes
            && case.fail_open_is_only_bounded_ray_depth
    };
    let balanced_cases = cases.iter().filter(|case| case_balanced(case)).count();
    let sky_paints_source_authorized_intervals = cases.iter().all(|case| {
        case.sky_plane_instances == 0
            || (case.plane_intervals_match_shared_boundary && case.plane_sources_were_admitted)
    });
    let no_cracks_or_double_authority =
        balanced_cases == cases.len() && cutout_source_admitted && !cutout_closed_source_coverage;
    let trace = format!(
        "cases={cases:?};cutout_admitted={cutout_source_admitted};cutout_closed={cutout_closed_source_coverage};cutout_cells={cutout_retained_wall_cells};cutout_fail_open={:?}",
        cutout_vertical.ordered_coverage_fail_open
    );

    Ok(SharedBoundaryConservationObservation {
        evaluated_cases: cases.len(),
        cases,
        balanced_cases,
        cutout_source_admitted,
        cutout_closed_source_coverage,
        cutout_retained_wall_cells,
        cutout_unresolved_fail_open: cutout_vertical.ordered_coverage_fail_open.len(),
        cutout_fail_open_reasons: cutout_vertical
            .ordered_coverage_fail_open
            .iter()
            .map(|failure| format!("{:?}", failure.reason))
            .collect(),
        cutout_fail_open_is_only_bounded_ray_depth: cutout_vertical
            .ordered_coverage_fail_open
            .iter()
            .all(|failure| {
                failure.reason == DoomOrderedCoverageFailOpenReason::RaySegmentDepthUnresolved
            }),
        sky_paints_source_authorized_intervals,
        no_cracks_or_double_authority,
        fingerprint: blake3::hash(trace.as_bytes()).to_hex().to_string(),
    })
}

fn invalid_controls(valid: &PreparedContribution) -> usize {
    let mut controls = vec![
        SourceInterval::new(0.5, 0.5).unwrap_err(),
        SourceInterval::new(0.75, 0.25).unwrap_err(),
        SourceInterval::new(f64::NAN, 0.5).unwrap_err(),
        SourceInterval::new(-0.1, 0.5).unwrap_err(),
    ];

    let mut overlapping = valid.clone();
    if let PreparedContributionOutcome::Partial { occurrences } = &mut overlapping.outcome {
        occurrences[1].horizontal = SourceInterval::new(0.04, 0.2).unwrap();
    }
    controls.push(overlapping.validate().unwrap_err());

    let mut mismatched = valid.clone();
    if let PreparedContributionOutcome::Partial { occurrences } = &mut mismatched.outcome {
        occurrences[0].source.seg += 1;
    }
    controls.push(mismatched.validate().unwrap_err());

    let mut empty = valid.clone();
    empty.outcome = PreparedContributionOutcome::Partial {
        occurrences: Vec::new(),
    };
    controls.push(empty.validate().unwrap_err());
    controls.len()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn one_source_can_retain_two_disjoint_correlated_occurrences() {
        let specimen = partial_specimen();
        specimen.validate().unwrap();
        let PreparedContributionOutcome::Partial { occurrences } = specimen.outcome else {
            panic!("expected partial outcome");
        };
        assert_eq!(occurrences.len(), 2);
        assert_eq!(occurrences[0].source, occurrences[1].source);
        assert_ne!(occurrences[0].occurrence, occurrences[1].occurrence);
        assert!(!occurrences[0]
            .horizontal
            .overlaps(occurrences[1].horizontal));
        assert!(occurrences
            .iter()
            .all(|occurrence| occurrence.eventual_renderer_resource.is_none()));
    }

    #[test]
    fn wall_and_plane_consumers_share_one_stored_causal_boundary() {
        let specimen = partial_specimen();
        specimen.validate().unwrap();
        assert_eq!(specimen.boundaries.len(), 1);
        assert_eq!(specimen.boundary_uses.len(), 4);
        assert!(specimen
            .boundary_uses
            .iter()
            .all(|usage| usage.boundary == specimen.boundaries[0].id));
    }

    #[test]
    fn four_outcomes_preserve_reject_and_fail_open_authority() {
        let observation = observe_private_occurrence_model();
        assert_eq!(observation.source_contributions, 1);
        assert_eq!(observation.partial_occurrences, 2);
        assert_eq!(observation.distinct_source_identities, 1);
        assert!(!observation.whole_retain_generated_geometry);
        assert!(observation.unresolved_retains_original);
        assert_eq!(observation.shared_boundary_consumers, 4);
    }

    #[test]
    fn invalid_domains_and_identity_mismatch_are_rejected() {
        let valid = partial_specimen();
        assert_eq!(invalid_controls(&valid), 7);
    }

    #[test]
    fn vertical_domain_validation_is_explicit() {
        assert_eq!(
            VerticalDomain::new(1.0, 1.0),
            Err(OccurrenceValidationError::EmptyVerticalDomain)
        );
        assert_eq!(
            VerticalDomain::new(2.0, 1.0),
            Err(OccurrenceValidationError::ReversedVerticalDomain)
        );
        assert_eq!(
            VerticalDomain::new(0.0, f64::INFINITY),
            Err(OccurrenceValidationError::NonFiniteDomain)
        );
    }

    #[test]
    fn all_wall_roles_and_boundary_consumers_remain_representable() {
        assert_eq!(ALL_WALL_ROLES.len(), 4);
        assert_eq!(ALL_BOUNDARY_CONSUMERS.len(), 4);
    }

    #[test]
    fn retained_partial_survival_poses_conserve_required_intervals() {
        let observation = observe_partial_survival_reconstruction().unwrap();
        assert_eq!(observation.poses.len(), 3);
        assert_eq!(observation.fragmented, 3);
        assert!(observation.poses.iter().all(|pose| {
            pose.retained_intervals.len() == 2
                && pose.required_survivor_columns == pose.represented_survivor_columns
                && pose.forbidden_columns > 0
        }));
    }

    #[test]
    fn reconstructed_fragments_stay_on_source_geometry_with_continuous_uvs() {
        let observation = observe_partial_survival_reconstruction().unwrap();
        assert!(
            observation.poses.iter().all(|pose| {
                pose.endpoint_checks > 0
                    && pose.endpoints_on_source_geometry
                    && pose.uv_parameterization_continuous
            }),
            "observation={observation:?}"
        );
    }

    #[test]
    fn bounded_jitter_preserves_source_and_occurrence_identity() {
        let observation = observe_partial_survival_reconstruction().unwrap();
        assert!(observation.stable_source_identity_under_jitter);
        assert!(observation.stable_occurrence_identity_under_jitter);
        assert!(observation.no_screen_column_inverse_projection);
    }

    #[test]
    fn ambiguous_and_unsupported_controls_fail_open_while_empty_requires_authority() {
        let observation = observe_partial_survival_reconstruction().unwrap();
        assert!(observation.near_plane_failed_open);
        assert!(observation.unsupported_role_failed_open);
        assert!(observation.empty_fragment_rejected_with_authority);
        assert!(observation.thin_projection_retained);
        assert_eq!(observation.failed_open, 2);
        assert_eq!(observation.whole_rejected, 1);
    }

    #[test]
    fn wall_and_plane_outputs_conserve_one_ordered_boundary() {
        let observation = observe_shared_boundary_conservation().unwrap();
        assert_eq!(observation.evaluated_cases, 5);
        assert_eq!(observation.balanced_cases, observation.evaluated_cases);
        assert!(observation.sky_paints_source_authorized_intervals);
        assert!(observation.no_cracks_or_double_authority);
    }

    #[test]
    fn paired_sky_marks_do_not_gain_independent_occlusion_authority() {
        let observation = observe_shared_boundary_conservation().unwrap();
        let paired = observation
            .cases
            .iter()
            .find(|case| case.fixture == "paired-sky-far-control")
            .unwrap();
        assert!(paired.paired_sky_events > 0);
        assert!(paired.paired_sky_events_are_non_mutating);
    }

    #[test]
    fn two_sided_masked_middle_does_not_close_source_coverage() {
        let observation = observe_shared_boundary_conservation().unwrap();
        assert!(observation.cutout_source_admitted);
        assert!(observation.cutout_retained_wall_cells > 0);
        assert!(!observation.cutout_closed_source_coverage);
        assert!(observation.cutout_unresolved_fail_open > 0);
        assert!(observation.cutout_fail_open_is_only_bounded_ray_depth);
    }

    #[test]
    fn whole_and_partial_outcomes_lower_to_ordinary_meshes() {
        let manifest = lower_occurrences_to_presentation().unwrap();
        assert!(manifest.whole_control.occurrence_correlation.is_none());
        assert!(!manifest.whole_control.generated_view_local_geometry);
        assert!(manifest.whole_control.mesh.vertex_count() > 0);
        assert_eq!(manifest.retained_semantic_occurrences, 2);
        assert_eq!(manifest.lowered_semantic_occurrences, 2);
        assert!(manifest
            .partial_declarations
            .iter()
            .all(|declaration| declaration.occurrence_correlation.is_some()
                && declaration.generated_view_local_geometry
                && declaration.mesh.vertex_count() > 0));
    }

    #[test]
    fn ordinary_lowering_preserves_source_order_domains_and_uv_streams() {
        let manifest = lower_occurrences_to_presentation().unwrap();
        assert!(manifest.source_order_preserved);
        assert!(manifest.source_correlation_preserved);
        assert!(manifest.endpoints_from_continuous_source_domains);
        assert!(manifest.uv_streams_complete);
        assert!(manifest.generated_geometry_is_view_local);
        assert_eq!(manifest.partial_declarations[0].source_interval[0], 0.0);
        assert_eq!(manifest.partial_declarations[1].source_interval[1], 1.0);
    }

    #[test]
    fn ordinary_lowering_has_a_stable_structural_fingerprint() {
        let first = lower_occurrences_to_presentation().unwrap();
        let second = lower_occurrences_to_presentation().unwrap();
        assert_eq!(first.structural_fingerprint.len(), 64);
        assert_eq!(first.structural_fingerprint, second.structural_fingerprint);
    }

    #[test]
    fn declared_door_snapshots_drive_only_the_current_prepared_boundary() {
        let manifest = lower_runtime_snapshots_to_presentation().unwrap();
        assert_eq!(manifest.door_states.len(), 4);
        assert!(manifest.door_source_identity_stable);
        assert_eq!(
            manifest
                .door_states
                .iter()
                .map(|state| (state.phase, state.vertical_range, state.lifecycle_action))
                .collect::<Vec<_>>(),
            vec![
                (
                    RuntimeSnapshotPhase::Closed,
                    Some([0.0, 128.0]),
                    SnapshotLifecycleAction::Create
                ),
                (
                    RuntimeSnapshotPhase::Opening,
                    Some([48.0, 128.0]),
                    SnapshotLifecycleAction::Replace
                ),
                (
                    RuntimeSnapshotPhase::Open,
                    None,
                    SnapshotLifecycleAction::Retire
                ),
                (
                    RuntimeSnapshotPhase::Closing,
                    Some([64.0, 128.0]),
                    SnapshotLifecycleAction::Create
                ),
            ]
        );
        assert!(manifest.door_states[2].mesh.is_none());
        assert!(manifest
            .door_states
            .iter()
            .filter(|state| state.phase != RuntimeSnapshotPhase::Open)
            .all(|state| state
                .mesh
                .as_ref()
                .is_some_and(|mesh| mesh.vertex_count() > 0)));
    }

    #[test]
    fn declared_platform_snapshots_replace_one_correlated_occurrence() {
        let manifest = lower_runtime_snapshots_to_presentation().unwrap();
        assert_eq!(manifest.platform_states.len(), 2);
        assert!(manifest.platform_source_identity_stable);
        assert_eq!(
            manifest.platform_states[0].vertical_range,
            Some([0.0, 128.0])
        );
        assert_eq!(
            manifest.platform_states[1].vertical_range,
            Some([48.0, 128.0])
        );
        assert_eq!(
            manifest.platform_states[0].occurrence_correlation,
            manifest.platform_states[1].occurrence_correlation
        );
        assert_eq!(
            manifest.platform_states[0].renderer_resource_correlation,
            manifest.platform_states[1].renderer_resource_correlation
        );
        assert_eq!(
            manifest.platform_states[1].lifecycle_action,
            SnapshotLifecycleAction::Replace
        );
    }

    #[test]
    fn runtime_snapshot_sequence_is_policy_free_bounded_and_deterministic() {
        let first = lower_runtime_snapshots_to_presentation().unwrap();
        let second = lower_runtime_snapshots_to_presentation().unwrap();
        assert!(first.current_heights_drive_preparation);
        assert!(!first.application_movement_policy_present);
        assert_eq!(first.affected_creates, 3);
        assert_eq!(first.affected_replacements, 2);
        assert_eq!(first.affected_retirements, 1);
        assert_eq!(first.unrelated_resource_reallocations, 0);
        assert_eq!(first.structural_fingerprint.len(), 64);
        assert_eq!(first.structural_fingerprint, second.structural_fingerprint);
    }
}
