//! Doom-owned comparative selectors retained by AR-0025.
//!
//! These controls consume source membership or ordered SEG observations and
//! filter already-declared presentation work. They remain Doom corpus
//! evidence, not renderer-owned visibility or generic scene membership.

use std::collections::{BTreeMap, BTreeSet};

use doom_map_provider::DoomMapCore;
use hello_doom_e1m1::{
    classify_static_draw_frustum_rejection, DoomComparativeEmbedding, StaticDrawAabb,
    StaticDrawPlanEntry, StaticDrawSource,
};
use tokimu::PlatformResult;
use tokimu_core::math::Mat4;

use crate::observer::{doom_source_pose, ObserverLook, SpawnObserver};

use super::super::{observe_doom_seg_classic_bsp, observe_doom_seg_screen_grid};
use super::model::CandidateSelectionSummary;

#[derive(Clone, Debug)]
pub(crate) struct DoomMembershipSelectionInput {
    pub(crate) subsector_bounds: Vec<Option<StaticDrawAabb>>,
    pub(crate) linedef_subsectors: Vec<Vec<u32>>,
}

/// Retained lookup for the interactive Stage 3B control. Geometry remains
/// static; only the caller-owned submitted subset changes with the observer.
pub(crate) struct DoomSegDynamicSelectionInput {
    pub(crate) draw_indices_by_seg: BTreeMap<u32, Vec<usize>>,
    pub(crate) flat_indices_by_subsector: BTreeMap<u32, Vec<usize>>,
    pub(crate) unsupported_textures: BTreeSet<String>,
}

pub(crate) fn membership_draw_selected(
    draw: &StaticDrawPlanEntry,
    selected_subsectors: &[bool],
    linedef_subsectors: &[Vec<u32>],
) -> bool {
    match draw.source {
        StaticDrawSource::Flat {
            source_subsector, ..
        } => selected_subsectors
            .get(source_subsector.record_index as usize)
            .copied()
            .unwrap_or(true),
        StaticDrawSource::Wall { source_linedef, .. } => linedef_subsectors
            .get(source_linedef.record_index as usize)
            .map(|subsectors| {
                subsectors.iter().any(|subsector| {
                    selected_subsectors
                        .get(*subsector as usize)
                        .copied()
                        .unwrap_or(true)
                })
            })
            .unwrap_or(true),
    }
}

/// Updates only the caller-owned submission mask for the retained Stage 3B
/// SEG meshes. Flats stay fail-open. This is deliberately not a renderer
/// culling path or a claim that the source grid is historic Doom visibility.
#[allow(clippy::too_many_arguments)]
pub(crate) fn select_seg_per_column_candidates(
    draws: &[StaticDrawPlanEntry],
    input: &DoomSegDynamicSelectionInput,
    map: &DoomMapCore,
    observer: SpawnObserver,
    look: ObserverLook,
    embedding: DoomComparativeEmbedding,
    selected: &mut [bool],
    summary: &mut CandidateSelectionSummary,
    rejection_samples: &mut Vec<String>,
    capture_samples: bool,
) -> PlatformResult<()> {
    let (source_position, source_angle) = doom_source_pose(observer, look, embedding);
    let observation = observe_doom_seg_screen_grid(
        map,
        observer.position.y,
        true,
        source_position,
        source_angle,
    )?;
    selected.fill(true);
    for indices in input.draw_indices_by_seg.values() {
        for &index in indices {
            selected[index] = false;
        }
    }
    for source_seg in &observation.selected_seg_records {
        if let Some(indices) = input.draw_indices_by_seg.get(source_seg) {
            for &index in indices {
                selected[index] = true;
            }
        }
    }
    for (index, (draw, is_selected)) in draws.iter().zip(selected.iter()).enumerate() {
        summary.candidates += 1;
        if *is_selected {
            summary.submitted += 1;
        } else {
            summary.rejected += 1;
            if capture_samples && rejection_samples.len() < 12 {
                rejection_samples.push(format!(
                    "{}:doom-seg-per-column-source-filtered",
                    draw.source_label
                ));
            }
        }
        debug_assert!(index < selected.len());
    }
    Ok(())
}

/// Applies the Doom-owned Stage 3B BSP/solid-range protocol to stable,
/// already-uploaded SEG wall draws. Whole flat draws stay fail-open because
/// reached BSP leaves are not equivalent to presented plane coverage.
#[allow(clippy::too_many_arguments)]
pub(crate) fn select_seg_classic_bsp_candidates(
    draws: &[StaticDrawPlanEntry],
    input: &DoomSegDynamicSelectionInput,
    map: &DoomMapCore,
    observer: SpawnObserver,
    look: ObserverLook,
    embedding: DoomComparativeEmbedding,
    selected: &mut [bool],
    summary: &mut CandidateSelectionSummary,
    rejection_samples: &mut Vec<String>,
    capture_samples: bool,
) -> PlatformResult<()> {
    let (source_position, source_angle) = doom_source_pose(observer, look, embedding);
    let observation =
        observe_doom_seg_classic_bsp(map, source_position, source_angle, &BTreeSet::new())?;

    selected.fill(true);
    for indices in input.draw_indices_by_seg.values() {
        for &index in indices {
            selected[index] = false;
        }
    }
    for source_seg in &observation.admitted_seg_records {
        if let Some(indices) = input.draw_indices_by_seg.get(source_seg) {
            for &index in indices {
                selected[index] = true;
            }
        }
    }
    for (draw, is_selected) in draws.iter().zip(selected.iter()) {
        summary.candidates += 1;
        if *is_selected {
            summary.submitted += 1;
        } else {
            summary.rejected += 1;
            if capture_samples && rejection_samples.len() < 12 {
                rejection_samples.push(format!(
                    "{}:doom-classic-bsp-source-filtered",
                    draw.source_label
                ));
            }
        }
    }
    Ok(())
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn select_membership_candidates(
    draws: &[StaticDrawPlanEntry],
    view_projection: Mat4,
    input: &DoomMembershipSelectionInput,
    selected: &mut [bool],
    summary: &mut CandidateSelectionSummary,
    rejection_samples: &mut Vec<String>,
    capture_samples: bool,
) {
    let subsectors = input
        .subsector_bounds
        .iter()
        .map(|bounds| {
            bounds.is_none_or(|bounds| {
                classify_static_draw_frustum_rejection(bounds, view_projection).is_none()
            })
        })
        .collect::<Vec<_>>();
    for (selected, draw) in selected.iter_mut().zip(draws) {
        summary.candidates += 1;
        *selected = membership_draw_selected(draw, &subsectors, &input.linedef_subsectors);
        if *selected {
            summary.submitted += 1;
        } else {
            summary.rejected += 1;
            if capture_samples && rejection_samples.len() < 12 {
                rejection_samples.push(format!("{}:doom-membership-filtered", draw.source_label));
            }
        }
    }
}
