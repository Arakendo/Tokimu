//! Bounded Doom-private reference planner evidence.
//!
//! This module composes the existing source-backed BSP traversal, vertical
//! coverage transitions, wall tiers, plane instances, sky identities, and
//! deferred masked-middle facts into one deterministic manifest. It is an
//! executable oracle for the campaign, not a renderer API or a pixel-parity
//! reimplementation of the historical Doom renderer.

use std::collections::{BTreeMap, BTreeSet};

use doom_geometry_provider::{
    observe_doom_two_sided_middle_textures, DoomOrderedCoverageTransitionReason,
    DoomSectorRuntimeHeightSnapshot, DoomTextureExtent,
};

use crate::{
    dynamic_door_snapshot_fixture, masked_middle_topology_fixture,
    moving_platform_snapshot_fixture, one_sky_far_control_fixture, paired_sky_far_control_fixture,
    projection_close_forward_seg_fixture, projection_near_plane_crossing_fixture,
    projection_thin_forward_seg_fixture, shared_key_disjoint_plane_fixture,
    vertical_aperture_control_fixture, DoomVisibilityFixture,
};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct OrderedReferenceCaseManifest {
    pub case: String,
    pub runtime_snapshot: String,
    pub admitted_seg_order: Vec<u32>,
    pub solid_admitted: usize,
    pub pass_admitted: usize,
    pub covered_columns: usize,
    pub coverage_transitions: usize,
    pub wall_tier_cells: usize,
    pub retained_wall_tier_cells: usize,
    pub plane_marks: usize,
    pub plane_instances: usize,
    pub sky_plane_instances: usize,
    pub paired_sky_intervals: usize,
    pub deferred_masked_work: usize,
    pub fail_open: usize,
    pub transition_chain_contiguous: bool,
    pub retained_walls_inside_open_range: bool,
    pub plane_sources_admitted: bool,
    pub masked_work_did_not_close_coverage: bool,
    pub balanced: bool,
    pub trace: Vec<String>,
    pub structural_fingerprint: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct OrderedReferencePlannerManifest {
    pub cases: Vec<OrderedReferenceCaseManifest>,
    pub evaluated_cases: usize,
    pub balanced_cases: usize,
    pub has_solid_and_pass_ranges: bool,
    pub has_vertical_clip_mutations: bool,
    pub has_wall_tiers: bool,
    pub has_plane_instances: bool,
    pub has_sky_intervals: bool,
    pub has_deferred_masked_work: bool,
    pub fail_open_retained: bool,
    pub application_movement_policy_present: bool,
    pub structural_fingerprint: String,
}

fn wall_extents(fixture: &DoomVisibilityFixture) -> Vec<DoomTextureExtent> {
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

fn observe_case(
    case: &str,
    runtime_snapshot: &str,
    fixture: DoomVisibilityFixture,
) -> Result<OrderedReferenceCaseManifest, String> {
    let bsp = fixture
        .observe_classic_bsp()
        .map_err(|error| error.to_string())?;
    let vertical = fixture
        .observe_classic_vertical_clips(41, &wall_extents(&fixture))
        .map_err(|error| error.to_string())?;
    let deferred =
        observe_doom_two_sided_middle_textures(&fixture.map).map_err(|error| error.to_string())?;

    let mut prior_by_column = BTreeMap::new();
    let transition_chain_contiguous = vertical.ordered_coverage_transitions.iter().all(|event| {
        let contiguous = prior_by_column
            .get(&event.column)
            .is_none_or(|prior| *prior == [event.upper_before, event.lower_before]);
        prior_by_column.insert(event.column, [event.upper_after, event.lower_after]);
        contiguous
    });
    let retained_walls_inside_open_range = vertical.ordered_wall_intervals.iter().all(|cell| {
        cell.retained_interval.is_none_or(|retained| {
            interval_contains(cell.raw_interval, retained)
                && cell
                    .open_interval_before
                    .is_some_and(|open| interval_contains(open, retained))
        })
    });
    let admitted = bsp
        .admitted_seg_order
        .iter()
        .copied()
        .collect::<BTreeSet<_>>();
    let plane_sources_admitted = vertical
        .plane_spans
        .keys
        .values()
        .flatten()
        .all(|instance| {
            instance
                .source_segs
                .iter()
                .all(|seg| admitted.contains(seg))
        });
    let paired_sky_intervals = vertical
        .ordered_coverage_transitions
        .iter()
        .filter(|event| {
            event.reason == DoomOrderedCoverageTransitionReason::PairedSkyBoundaryRetained
        })
        .count();
    let sky_plane_instances = vertical
        .plane_spans
        .keys
        .iter()
        .filter(|(key, _)| key.texture == "F_SKY1")
        .map(|(_, instances)| instances.len())
        .sum();
    let masked_work_did_not_close_coverage = deferred.is_empty()
        || !vertical.ordered_coverage_transitions.iter().any(|event| {
            event.reason == DoomOrderedCoverageTransitionReason::OneSidedMiddleClosed
                && deferred
                    .iter()
                    .any(|middle| middle.source_linedef.record_index == event.source_linedef)
        });
    let plane_marks = vertical.floor_plane_marks + vertical.ceiling_plane_marks;
    let retained_wall_tier_cells = vertical
        .ordered_wall_intervals
        .iter()
        .filter(|cell| cell.retained_interval.is_some())
        .count();
    let balanced = transition_chain_contiguous
        && retained_walls_inside_open_range
        && plane_sources_admitted
        && masked_work_did_not_close_coverage
        && bsp.admitted_seg_order.len() == vertical.admitted_segs;

    let mut trace = vec![
        format!("case={case}"),
        format!("snapshot={runtime_snapshot}"),
        format!("order={:?}", bsp.admitted_seg_order),
        format!("solid={};pass={}", bsp.solid_admitted, bsp.pass_admitted),
        format!("covered={}", bsp.solid_range_covered_columns),
    ];
    trace.extend(vertical.ordered_coverage_transitions.iter().map(|event| {
        format!(
            "transition:{}:{}:{}:{}-{}>{}-{}:{:?}:{:?}",
            event.source_seg,
            event.source_linedef,
            event.column,
            event.upper_before,
            event.lower_before,
            event.upper_after,
            event.lower_after,
            event.reason,
            event.retained_plane_interval
        )
    }));
    trace.extend(vertical.ordered_wall_intervals.iter().map(|cell| {
        format!(
            "wall:{}:{}:{}:{:?}:{:?}:{:?}:{:?}",
            cell.source_seg,
            cell.source_linedef,
            cell.column,
            cell.role,
            cell.raw_interval,
            cell.open_interval_before,
            cell.retained_interval
        )
    }));
    for (key, instances) in &vertical.plane_spans.keys {
        for (ordinal, instance) in instances.iter().enumerate() {
            trace.push(format!(
                "plane:{:?}:{}:{}:{}:{}:{:?}:{:?}",
                key.kind,
                key.height,
                key.texture,
                key.light,
                ordinal,
                instance.source_sectors,
                instance.source_segs
            ));
        }
    }
    trace.extend(deferred.iter().map(|middle| {
        format!(
            "masked:{}:{}:{}:{:?}:{}:{}..{}",
            middle.source_linedef.record_index,
            middle.source_sidedef.record_index,
            middle.source_sector.record_index,
            middle.side,
            middle.texture_name,
            middle.opening_floor,
            middle.opening_ceiling
        )
    }));
    trace.extend(vertical.ordered_coverage_fail_open.iter().map(|failure| {
        format!(
            "fail-open:{}:{:?}:{:?}:{:?}",
            failure.source_seg, failure.source_linedef, failure.column, failure.reason
        )
    }));
    let structural_fingerprint = blake3::hash(trace.join("\n").as_bytes())
        .to_hex()
        .to_string();

    Ok(OrderedReferenceCaseManifest {
        case: case.to_owned(),
        runtime_snapshot: runtime_snapshot.to_owned(),
        admitted_seg_order: bsp.admitted_seg_order,
        solid_admitted: bsp.solid_admitted,
        pass_admitted: bsp.pass_admitted,
        covered_columns: bsp.solid_range_covered_columns,
        coverage_transitions: vertical.ordered_coverage_transitions.len(),
        wall_tier_cells: vertical.ordered_wall_intervals.len(),
        retained_wall_tier_cells,
        plane_marks,
        plane_instances: vertical.plane_spans.plane_instances,
        sky_plane_instances,
        paired_sky_intervals,
        deferred_masked_work: deferred.len(),
        fail_open: vertical.ordered_coverage_fail_open.len(),
        transition_chain_contiguous,
        retained_walls_inside_open_range,
        plane_sources_admitted,
        masked_work_did_not_close_coverage,
        balanced,
        trace,
        structural_fingerprint,
    })
}

pub fn observe_ordered_reference_planner() -> Result<OrderedReferencePlannerManifest, String> {
    let mut cases = vec![
        observe_case(
            "paired-sky",
            "static",
            paired_sky_far_control_fixture().map_err(|error| error.to_string())?,
        )?,
        observe_case(
            "one-sky-negative",
            "static",
            one_sky_far_control_fixture().map_err(|error| error.to_string())?,
        )?,
        observe_case(
            "vertical-aperture",
            "static",
            vertical_aperture_control_fixture().map_err(|error| error.to_string())?,
        )?,
        observe_case(
            "shared-plane-key",
            "static",
            shared_key_disjoint_plane_fixture().map_err(|error| error.to_string())?,
        )?,
    ];

    let door = dynamic_door_snapshot_fixture().map_err(|error| error.to_string())?;
    let door_sector = door.map.sectors[1].source;
    for (phase, height) in [
        ("closed", 0),
        ("opening", 48),
        ("open", 128),
        ("closing", 64),
    ] {
        let projected = door
            .with_runtime_height_snapshots(&[DoomSectorRuntimeHeightSnapshot {
                source_sector: door_sector,
                floor_height: None,
                ceiling_height: Some(height),
            }])
            .map_err(|error| error.to_string())?;
        cases.push(observe_case("dynamic-door", phase, projected)?);
    }

    let platform = moving_platform_snapshot_fixture().map_err(|error| error.to_string())?;
    let platform_sector = platform.map.sectors[0].source;
    for (phase, height) in [("low", 0), ("raised", 48)] {
        let projected = platform
            .with_runtime_height_snapshots(&[DoomSectorRuntimeHeightSnapshot {
                source_sector: platform_sector,
                floor_height: Some(height),
                ceiling_height: None,
            }])
            .map_err(|error| error.to_string())?;
        cases.push(observe_case("platform", phase, projected)?);
    }

    cases.push(observe_case(
        "projection-epsilon-near",
        "static",
        projection_near_plane_crossing_fixture().map_err(|error| error.to_string())?,
    )?);
    cases.push(observe_case(
        "projection-epsilon-thin",
        "static",
        projection_thin_forward_seg_fixture().map_err(|error| error.to_string())?,
    )?);
    cases.push(observe_case(
        "projection-epsilon-close",
        "static",
        projection_close_forward_seg_fixture().map_err(|error| error.to_string())?,
    )?);
    cases.push(observe_case(
        "cutout-non-occluder",
        "static",
        masked_middle_topology_fixture().map_err(|error| error.to_string())?,
    )?);

    let evaluated_cases = cases.len();
    let balanced_cases = cases.iter().filter(|case| case.balanced).count();
    let has_solid_and_pass_ranges = cases.iter().any(|case| case.solid_admitted > 0)
        && cases.iter().any(|case| case.pass_admitted > 0);
    let has_vertical_clip_mutations = cases.iter().any(|case| case.coverage_transitions > 0);
    let has_wall_tiers = cases.iter().any(|case| case.wall_tier_cells > 0);
    let has_plane_instances = cases.iter().any(|case| case.plane_instances > 0);
    let has_sky_intervals = cases
        .iter()
        .any(|case| case.sky_plane_instances > 0 || case.paired_sky_intervals > 0);
    let has_deferred_masked_work = cases.iter().any(|case| case.deferred_masked_work > 0);
    let fail_open_retained = cases.iter().any(|case| case.fail_open > 0);
    let trace = cases
        .iter()
        .map(|case| format!("{}:{}", case.case, case.structural_fingerprint))
        .collect::<Vec<_>>()
        .join("\n");

    Ok(OrderedReferencePlannerManifest {
        cases,
        evaluated_cases,
        balanced_cases,
        has_solid_and_pass_ranges,
        has_vertical_clip_mutations,
        has_wall_tiers,
        has_plane_instances,
        has_sky_intervals,
        has_deferred_masked_work,
        fail_open_retained,
        application_movement_policy_present: false,
        structural_fingerprint: blake3::hash(trace.as_bytes()).to_hex().to_string(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reference_planner_balances_every_required_control() {
        let manifest = observe_ordered_reference_planner().expect("planner manifest");
        assert_eq!(manifest.evaluated_cases, 14);
        assert_eq!(manifest.balanced_cases, manifest.evaluated_cases);
        assert!(manifest.has_solid_and_pass_ranges);
        assert!(manifest.has_vertical_clip_mutations);
        assert!(manifest.has_wall_tiers);
        assert!(manifest.has_plane_instances);
        assert!(manifest.has_sky_intervals);
        assert!(manifest.has_deferred_masked_work);
        assert!(manifest.fail_open_retained);
        assert!(!manifest.application_movement_policy_present);

        let paired_sky = manifest
            .cases
            .iter()
            .find(|case| case.case == "paired-sky")
            .expect("paired-sky case");
        let one_sky = manifest
            .cases
            .iter()
            .find(|case| case.case == "one-sky-negative")
            .expect("one-sky negative case");
        let aperture = manifest
            .cases
            .iter()
            .find(|case| case.case == "vertical-aperture")
            .expect("vertical-aperture case");
        let shared_plane = manifest
            .cases
            .iter()
            .find(|case| case.case == "shared-plane-key")
            .expect("shared-plane-key case");
        let projection_near = manifest
            .cases
            .iter()
            .find(|case| case.case == "projection-epsilon-near")
            .expect("projection near-plane case");

        assert!(paired_sky.paired_sky_intervals > 0);
        assert_eq!(one_sky.paired_sky_intervals, 0);
        assert!(aperture.wall_tier_cells > 0);
        assert!(aperture.plane_marks > 0);
        assert!(shared_plane.plane_instances >= 2);
        assert!(projection_near.fail_open > 0);
    }

    #[test]
    fn reference_planner_is_deterministic() {
        let first = observe_ordered_reference_planner().expect("first manifest");
        let second = observe_ordered_reference_planner().expect("second manifest");
        assert_eq!(first, second);
    }

    #[test]
    fn cutout_work_is_deferred_without_closing_coverage() {
        let manifest = observe_ordered_reference_planner().expect("planner manifest");
        let cutout = manifest
            .cases
            .iter()
            .find(|case| case.case == "cutout-non-occluder")
            .expect("cutout case");
        assert!(cutout.deferred_masked_work > 0);
        assert!(cutout.masked_work_did_not_close_coverage);
    }

    #[test]
    fn runtime_snapshots_change_planner_evidence_without_movement_policy() {
        let manifest = observe_ordered_reference_planner().expect("planner manifest");
        let door = manifest
            .cases
            .iter()
            .filter(|case| case.case == "dynamic-door")
            .collect::<Vec<_>>();
        assert_eq!(door.len(), 4);
        assert!(door
            .windows(2)
            .any(|pair| pair[0].structural_fingerprint != pair[1].structural_fingerprint));
        assert!(!manifest.application_movement_policy_present);

        let platform = manifest
            .cases
            .iter()
            .filter(|case| case.case == "platform")
            .collect::<Vec<_>>();
        assert_eq!(platform.len(), 2);
        assert_ne!(
            platform[0].structural_fingerprint,
            platform[1].structural_fingerprint
        );
    }
}
