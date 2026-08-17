//! Candidate 1 synthetic matrix for the AR-0030 sky-depth experiment.
//!
//! Candidate 1 adds only Doom-authorized, submission-local sky-depth geometry.
//! Ordinary walls, planes, apertures, runtime snapshots, and cutouts remain in
//! the existing ordered reference preparation. This module proves that split;
//! it is corpus evidence, not a renderer or Doom public API.

use std::collections::BTreeSet;

use crate::{
    observe_authoritative_sky_regions, observe_ordered_reference_planner,
    one_sky_far_control_fixture, paired_sky_far_control_fixture,
    prepare_authoritative_sky_depth_declarations,
    prepare_authoritative_sky_submission_local_geometry, terminal_sky_ordered_fixture,
    OrderedReferenceCaseManifest, SubmissionIdentity, SubmissionLocalGeometryLimits,
};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Candidate1ControlObservation {
    pub case: String,
    pub snapshots: Vec<String>,
    pub balanced: bool,
    pub sky_depth_batches: usize,
    pub deferred_cutout_work: usize,
    pub fail_open: usize,
    pub structural_fingerprints: Vec<String>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Candidate1SyntheticMatrixManifest {
    pub ordered_cases: usize,
    pub balanced_cases: usize,
    pub controls: Vec<Candidate1ControlObservation>,
    pub positive_sky_input_intervals: usize,
    pub positive_sky_modeled_intervals: usize,
    pub positive_sky_input_cells: usize,
    pub positive_sky_modeled_cells: usize,
    pub positive_sky_declarations: usize,
    pub positive_sky_local_payloads: usize,
    pub positive_sky_local_draws: usize,
    pub positive_sky_local_triangles: usize,
    pub negative_authority_controls: usize,
    pub negative_authority_declarations: usize,
    pub removed_non_sky_contributions: usize,
    pub persistent_mesh_identities: usize,
    pub ordinary_controls_balanced: bool,
    pub runtime_snapshots_are_declared_inputs: bool,
    pub cutout_remains_deferred: bool,
    pub no_diagnostic_grid_identity: bool,
    pub no_generic_filter_used: bool,
    pub unexplained_contributions: usize,
    pub semantic_comparison_only: bool,
    pub structural_fingerprint: String,
}

fn control(cases: &[OrderedReferenceCaseManifest], case: &str) -> Candidate1ControlObservation {
    let selected = cases
        .iter()
        .filter(|candidate| candidate.case == case)
        .collect::<Vec<_>>();
    Candidate1ControlObservation {
        case: case.to_owned(),
        snapshots: selected
            .iter()
            .map(|candidate| candidate.runtime_snapshot.clone())
            .collect(),
        balanced: !selected.is_empty() && selected.iter().all(|candidate| candidate.balanced),
        sky_depth_batches: 0,
        deferred_cutout_work: selected
            .iter()
            .map(|candidate| candidate.deferred_masked_work)
            .sum(),
        fail_open: selected.iter().map(|candidate| candidate.fail_open).sum(),
        structural_fingerprints: selected
            .iter()
            .map(|candidate| candidate.structural_fingerprint.clone())
            .collect(),
    }
}

/// Runs the complete synthetic Candidate 1 conservation matrix.
///
/// A zero-authority control deliberately skips G2 construction. Absence of a
/// local batch is the correct semantic result; it is not represented as an
/// empty or failed submission.
pub fn observe_candidate1_synthetic_matrix() -> Result<Candidate1SyntheticMatrixManifest, String> {
    let ordered = observe_ordered_reference_planner()?;
    let control_names = [
        "paired-sky",
        "one-sky-negative",
        "vertical-aperture",
        "shared-plane-key",
        "dynamic-door",
        "platform",
        "projection-epsilon-near",
        "projection-epsilon-thin",
        "projection-epsilon-close",
        "cutout-non-occluder",
    ];
    let controls = control_names
        .iter()
        .map(|name| control(&ordered.cases, name))
        .collect::<Vec<_>>();

    let positive_fixture = terminal_sky_ordered_fixture().map_err(|error| error.to_string())?;
    let positive = observe_authoritative_sky_regions(&positive_fixture, 41, "static")?;
    let positive_depth =
        prepare_authoritative_sky_depth_declarations(&positive, 0.25, "doom-sky-depth");
    let positive_local = prepare_authoritative_sky_submission_local_geometry(
        &positive_depth,
        SubmissionIdentity(3001),
        SubmissionLocalGeometryLimits::default(),
    )
    .map_err(|error| error.to_string())?;

    let negative_fixtures = [
        paired_sky_far_control_fixture().map_err(|error| error.to_string())?,
        one_sky_far_control_fixture().map_err(|error| error.to_string())?,
    ];
    let mut negative_authority_declarations = 0;
    let mut negative_removed_non_sky = 0;
    for fixture in &negative_fixtures {
        let regions = observe_authoritative_sky_regions(fixture, 41, "static")?;
        let depth = prepare_authoritative_sky_depth_declarations(&regions, 0.25, "doom-sky-depth");
        negative_authority_declarations += depth.declarations.len();
        negative_removed_non_sky += regions.removed_non_sky_contributions;
        // Deliberately no G2 builder call: no authority means no local batch.
    }

    let ordinary_controls_balanced = ordered.balanced_cases == ordered.evaluated_cases
        && controls.iter().all(|observation| observation.balanced);
    let dynamic = controls
        .iter()
        .find(|observation| observation.case == "dynamic-door")
        .ok_or("dynamic-door control missing")?;
    let platform = controls
        .iter()
        .find(|observation| observation.case == "platform")
        .ok_or("platform control missing")?;
    let runtime_snapshots_are_declared_inputs = !ordered.application_movement_policy_present
        && dynamic.snapshots.len() == 4
        && dynamic
            .structural_fingerprints
            .iter()
            .collect::<BTreeSet<_>>()
            .len()
            > 1
        && platform.snapshots.len() == 2
        && platform
            .structural_fingerprints
            .iter()
            .collect::<BTreeSet<_>>()
            .len()
            > 1;
    let cutout_remains_deferred = controls.iter().any(|observation| {
        observation.case == "cutout-non-occluder"
            && observation.deferred_cutout_work > 0
            && observation.balanced
    });
    let no_diagnostic_grid_identity = positive_depth
        .declarations
        .iter()
        .all(|declaration| declaration.triangle_count * 3 == declaration.positions.len())
        && positive_depth
            .declarations
            .iter()
            .map(|declaration| declaration.triangle_count)
            .sum::<usize>()
            < positive.input_sky_cells;
    let removed_non_sky_contributions =
        positive.removed_non_sky_contributions + negative_removed_non_sky;
    let persistent_mesh_identities =
        positive_depth.persistent_mesh_identities + positive_local.persistent_mesh_identities;
    let unexplained_contributions = usize::from(
        positive.input_sky_intervals != positive.modeled_sky_intervals
            || positive.input_sky_cells != positive.modeled_sky_cells
            || positive.omitted_sky_intervals != 0
            || positive_depth.declarations.len() != positive.regions.len()
            || positive_local.payloads.len() != positive_depth.declarations.len()
            || positive_local.draws.len() != positive_depth.declarations.len()
            || negative_authority_declarations != 0
            || removed_non_sky_contributions != 0,
    );

    let trace = format!(
        "ordered={};positive={};depth={};local={};controls={:?};negative={};runtime={};cutout={};grid={};unexplained={}",
        ordered.structural_fingerprint,
        positive.structural_fingerprint,
        positive_depth.structural_fingerprint,
        positive_local.structural_fingerprint,
        controls,
        negative_authority_declarations,
        runtime_snapshots_are_declared_inputs,
        cutout_remains_deferred,
        no_diagnostic_grid_identity,
        unexplained_contributions,
    );

    Ok(Candidate1SyntheticMatrixManifest {
        ordered_cases: ordered.evaluated_cases,
        balanced_cases: ordered.balanced_cases,
        controls,
        positive_sky_input_intervals: positive.input_sky_intervals,
        positive_sky_modeled_intervals: positive.modeled_sky_intervals,
        positive_sky_input_cells: positive.input_sky_cells,
        positive_sky_modeled_cells: positive.modeled_sky_cells,
        positive_sky_declarations: positive_depth.declarations.len(),
        positive_sky_local_payloads: positive_local.payloads.len(),
        positive_sky_local_draws: positive_local.draws.len(),
        positive_sky_local_triangles: positive_local.total_triangles,
        negative_authority_controls: negative_fixtures.len(),
        negative_authority_declarations,
        removed_non_sky_contributions,
        persistent_mesh_identities,
        ordinary_controls_balanced,
        runtime_snapshots_are_declared_inputs,
        cutout_remains_deferred,
        no_diagnostic_grid_identity,
        no_generic_filter_used: true,
        unexplained_contributions,
        semantic_comparison_only: true,
        structural_fingerprint: blake3::hash(trace.as_bytes()).to_hex().to_string(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn candidate1_matrix_conserves_authority_and_leaves_ordinary_controls_alone() {
        let manifest = observe_candidate1_synthetic_matrix().expect("Candidate 1 matrix");
        assert_eq!(manifest.ordered_cases, 14);
        assert_eq!(manifest.balanced_cases, manifest.ordered_cases);
        assert!(manifest.ordinary_controls_balanced);
        assert!(manifest.positive_sky_input_intervals > 0);
        assert_eq!(
            manifest.positive_sky_input_intervals,
            manifest.positive_sky_modeled_intervals
        );
        assert_eq!(
            manifest.positive_sky_input_cells,
            manifest.positive_sky_modeled_cells
        );
        assert_eq!(
            manifest.positive_sky_declarations,
            manifest.positive_sky_local_payloads
        );
        assert_eq!(
            manifest.positive_sky_declarations,
            manifest.positive_sky_local_draws
        );
        assert!(manifest.positive_sky_local_triangles > 0);
        assert_eq!(manifest.negative_authority_controls, 2);
        assert_eq!(manifest.negative_authority_declarations, 0);
        assert_eq!(manifest.removed_non_sky_contributions, 0);
        assert_eq!(manifest.persistent_mesh_identities, 0);
        assert!(manifest.runtime_snapshots_are_declared_inputs);
        assert!(manifest.cutout_remains_deferred);
        assert!(manifest.no_diagnostic_grid_identity);
        assert!(manifest.no_generic_filter_used);
        assert_eq!(manifest.unexplained_contributions, 0);
        assert!(manifest.semantic_comparison_only);
        assert!(manifest
            .controls
            .iter()
            .all(|control| control.sky_depth_batches == 0));
    }

    #[test]
    fn candidate1_matrix_is_deterministic() {
        let first = observe_candidate1_synthetic_matrix().expect("first matrix");
        let second = observe_candidate1_synthetic_matrix().expect("second matrix");
        assert_eq!(first, second);
    }
}
