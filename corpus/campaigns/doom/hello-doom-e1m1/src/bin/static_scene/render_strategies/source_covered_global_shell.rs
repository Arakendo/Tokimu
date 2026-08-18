//! Doom-private source-covered-domain walkabout experiment.
//!
//! This deliberately retains the complete reconstructed contribution for each
//! reached source owner. It does not attempt Classic plane clipping or expose
//! Doom traversal concepts to the renderer.

use std::collections::{BTreeMap, BTreeSet};

use doom_geometry_provider::{observe_doom_classic_bsp, resolve_doom_linedef_subsector_membership};
use doom_map_provider::DoomMapCore;
use tokimu::PlatformResult;

use super::{AppliedRenderStrategy, CandidateSelection, SceneInput};
use crate::StaticDrawSource;

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct SourceCoveredDomainObservation {
    pub(crate) visited_subsectors: BTreeSet<u16>,
    pub(crate) input_opaque: usize,
    pub(crate) input_cutouts: usize,
    pub(crate) retained_opaque: usize,
    pub(crate) retained_cutouts: usize,
    pub(crate) rejected_flat_draws: usize,
    pub(crate) rejected_wall_draws: usize,
    pub(crate) unresolved_fail_open: usize,
}

impl SourceCoveredDomainObservation {
    pub(crate) fn verify_conservation(&self) -> Result<(), String> {
        let retained = self.retained_opaque + self.retained_cutouts;
        let rejected = self.rejected_flat_draws + self.rejected_wall_draws;
        let input = self.input_opaque + self.input_cutouts;
        if retained + rejected != input {
            return Err(format!(
                "source-covered domain conservation failed: retained={retained}, rejected={rejected}, input={input}"
            ));
        }
        Ok(())
    }

    pub(crate) fn report(&self) -> String {
        format!(
            "visited-subsectors={}; input=[opaque:{},cutout:{}]; retained=[opaque:{},cutout:{}]; rejected=[flat:{},wall:{}]; unresolved-fail-open={}; conservation=balanced; plane-policy=whole-original-subsector-geometry-for-reached-owner; wall-policy=retain-if-any-owner-reached",
            self.visited_subsectors.len(),
            self.input_opaque,
            self.input_cutouts,
            self.retained_opaque,
            self.retained_cutouts,
            self.rejected_flat_draws,
            self.rejected_wall_draws,
            self.unresolved_fail_open,
        )
    }
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct SourceCoveredGlobalShellPreparation {
    pub(crate) opaque_draws: Vec<crate::StaticDrawPlanEntry>,
    pub(crate) cutout_draws: Vec<crate::StaticDrawPlanEntry>,
    pub(crate) observation: SourceCoveredDomainObservation,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum DomainDecision {
    Retain,
    RejectFlat,
    RejectWall,
    UnresolvedFailOpen,
}

pub(crate) fn prepare(
    source: &SceneInput,
    runtime_map: &DoomMapCore,
    viewer: [i16; 2],
    heading: f64,
) -> PlatformResult<SourceCoveredGlobalShellPreparation> {
    let classic = observe_doom_classic_bsp(runtime_map, viewer, heading, &BTreeSet::new())?;
    let memberships = resolve_doom_linedef_subsector_membership(runtime_map)
        .into_iter()
        .map(|membership| {
            (
                membership.source_linedef.record_index,
                membership
                    .source_subsectors
                    .into_iter()
                    .filter_map(|subsector| u16::try_from(subsector.record_index).ok())
                    .collect::<BTreeSet<_>>(),
            )
        })
        .collect::<BTreeMap<_, _>>();

    let mut rejected_flat_draws = 0usize;
    let mut rejected_wall_draws = 0usize;
    let mut unresolved_fail_open = 0usize;
    let classify = |draw: &crate::StaticDrawPlanEntry| match draw.source {
        StaticDrawSource::Flat {
            source_subsector, ..
        } => match u16::try_from(source_subsector.record_index) {
            Ok(subsector) if classic.visited_subsectors.contains(&subsector) => {
                DomainDecision::Retain
            }
            Ok(_) => DomainDecision::RejectFlat,
            Err(_) => DomainDecision::UnresolvedFailOpen,
        },
        StaticDrawSource::Wall { source_linedef, .. } => {
            match memberships.get(&source_linedef.record_index) {
                Some(owners) if owners.is_empty() => DomainDecision::UnresolvedFailOpen,
                Some(owners)
                    if owners
                        .iter()
                        .any(|owner| classic.visited_subsectors.contains(owner)) =>
                {
                    DomainDecision::Retain
                }
                Some(_) => DomainDecision::RejectWall,
                None => DomainDecision::UnresolvedFailOpen,
            }
        }
    };
    let mut filter = |draws: &[crate::StaticDrawPlanEntry]| {
        draws
            .iter()
            .filter_map(|draw| match classify(draw) {
                DomainDecision::Retain => Some(draw.clone()),
                DomainDecision::RejectFlat => {
                    rejected_flat_draws += 1;
                    None
                }
                DomainDecision::RejectWall => {
                    rejected_wall_draws += 1;
                    None
                }
                DomainDecision::UnresolvedFailOpen => {
                    unresolved_fail_open += 1;
                    Some(draw.clone())
                }
            })
            .collect::<Vec<_>>()
    };
    let opaque_draws = filter(&source.opaque_draws);
    let cutout_draws = filter(&source.cutout_draws);
    let observation = SourceCoveredDomainObservation {
        visited_subsectors: classic.visited_subsectors,
        input_opaque: source.opaque_draws.len(),
        input_cutouts: source.cutout_draws.len(),
        retained_opaque: opaque_draws.len(),
        retained_cutouts: cutout_draws.len(),
        rejected_flat_draws,
        rejected_wall_draws,
        unresolved_fail_open,
    };
    observation
        .verify_conservation()
        .map_err(|error| -> Box<dyn std::error::Error> { error.into() })?;
    Ok(SourceCoveredGlobalShellPreparation {
        opaque_draws,
        cutout_draws,
        observation,
    })
}

pub(super) fn apply(scene: &mut SceneInput) -> PlatformResult<AppliedRenderStrategy> {
    let prepared = prepare(
        scene,
        &scene.door_geometry_source.map,
        scene.spawn_observer.source_position,
        f64::from(scene.spawn_observer.source_angle).to_radians(),
    )?;
    eprintln!(
        "E1M1 source-covered global-shell initial preparation: {}",
        prepared.observation.report(),
    );
    scene.opaque_draws = prepared.opaque_draws;
    scene.cutout_draws = prepared.cutout_draws;
    Ok(AppliedRenderStrategy {
        candidate_selection: CandidateSelection::FullSubmission,
        ordered_coverage_prepared: false,
        source_covered_domain_filter: true,
        fixed_reconstruction_camera: false,
    })
}
