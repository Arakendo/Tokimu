//! Study-local source-topology admission observations.
//!
//! These records classify source occurrences only. They do not own meshes,
//! renderer resources, scissors, or a reusable visibility contract.

use std::collections::BTreeSet;

use doom_geometry_provider::{
    lower_doom_two_sided_wall_bands, DoomGeometryError, DoomSectorRuntimeHeightSnapshot,
};

use crate::DoomVisibilityFixture;

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum TopologyContributionFamily {
    Floor,
    Ceiling,
    SkyPlane,
    WallUpper,
    WallLower,
    WallMiddle,
    CutoutMiddle,
    DynamicCeilingBoundary,
    DynamicFloorPlane,
}

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum TopologyAdmissionResult {
    Admitted,
    Rejected,
    UnresolvedFailOpen,
}

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum TopologyAdmissionReason {
    OrderedSourceSegAdmitted,
    VisitedSourcePlane,
    PositiveTerminalSolidRange,
    CurrentHeightBoundaryPresent,
    CurrentHeightOpening,
    CurrentFloorSnapshot,
    ProjectionOrTraversalAmbiguous,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TopologyAdmissionRecord {
    pub family: TopologyContributionFamily,
    pub result: TopologyAdmissionResult,
    pub reason: TopologyAdmissionReason,
    pub source_subsector: Option<u16>,
    pub source_seg: Option<u32>,
    pub source_linedef: Option<u32>,
    pub source_sidedef: Option<u32>,
    pub source_sector: Option<u32>,
    pub runtime_floor_height: Option<i16>,
    pub runtime_ceiling_height: Option<i16>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TopologyAdmissionManifest {
    pub fixture: String,
    pub records: Vec<TopologyAdmissionRecord>,
    pub admitted: usize,
    pub rejected: usize,
    pub unresolved_fail_open: usize,
    pub fingerprint: String,
    pub trace: String,
}

impl TopologyAdmissionManifest {
    fn from_records(fixture: String, mut records: Vec<TopologyAdmissionRecord>) -> Self {
        records.sort_by_key(|record| {
            (
                record.source_subsector,
                record.source_seg,
                record.source_linedef,
                record.family,
                record.result,
                record.reason,
            )
        });
        let admitted = records
            .iter()
            .filter(|record| record.result == TopologyAdmissionResult::Admitted)
            .count();
        let rejected = records
            .iter()
            .filter(|record| record.result == TopologyAdmissionResult::Rejected)
            .count();
        let unresolved_fail_open = records
            .iter()
            .filter(|record| record.result == TopologyAdmissionResult::UnresolvedFailOpen)
            .count();
        let trace = format!(
            "fixture={fixture};records={records:?};counts=admitted:{admitted},rejected:{rejected},unresolved-fail-open:{unresolved_fail_open}"
        );
        let fingerprint = blake3::hash(trace.as_bytes()).to_hex().to_string();
        Self {
            fixture,
            records,
            admitted,
            rejected,
            unresolved_fail_open,
            fingerprint,
            trace,
        }
    }
}

fn terminal_solid_subsectors(elisions: &[String]) -> BTreeSet<u16> {
    let mut result = BTreeSet::new();
    for elision in elisions {
        if !elision.contains("reason=solid-range") {
            continue;
        }
        let Some(start) = elision.find("subsectors=[") else {
            continue;
        };
        let values = &elision[start + "subsectors=[".len()..];
        let Some(end) = values.find(']') else {
            continue;
        };
        for value in values[..end].split(',').map(str::trim) {
            if let Ok(subsector) = value.parse() {
                result.insert(subsector);
            }
        }
    }
    result
}

fn selected_sidedef(fixture: &DoomVisibilityFixture, seg_index: usize) -> Option<u16> {
    let seg = fixture.map.segs.get(seg_index)?;
    let linedef = fixture.map.linedefs.get(usize::from(seg.linedef))?;
    match seg.direction {
        0 => linedef.right_sidedef,
        1 => linedef.left_sidedef,
        _ => None,
    }
}

fn wall_families(
    fixture: &DoomVisibilityFixture,
    seg_index: usize,
) -> Vec<TopologyContributionFamily> {
    let Some(sidedef_index) = selected_sidedef(fixture, seg_index) else {
        return vec![TopologyContributionFamily::WallMiddle];
    };
    let sidedef = &fixture.map.sidedefs[usize::from(sidedef_index)];
    let linedef = &fixture.map.linedefs[usize::from(fixture.map.segs[seg_index].linedef)];
    let two_sided = linedef.right_sidedef.is_some() && linedef.left_sidedef.is_some();
    let mut families = Vec::new();
    if sidedef.upper_texture != "-" {
        families.push(TopologyContributionFamily::WallUpper);
    }
    if sidedef.lower_texture != "-" {
        families.push(TopologyContributionFamily::WallLower);
    }
    if sidedef.middle_texture != "-" {
        families.push(if two_sided {
            TopologyContributionFamily::CutoutMiddle
        } else {
            TopologyContributionFamily::WallMiddle
        });
    }
    if families.is_empty() {
        // The source occurrence still participates in reachability even when
        // it has no authored ordinary wall tier.
        families.push(TopologyContributionFamily::WallMiddle);
    }
    families
}

fn source_sector_for_seg(fixture: &DoomVisibilityFixture, seg_index: usize) -> Option<u32> {
    let sidedef = selected_sidedef(fixture, seg_index)?;
    fixture
        .map
        .sectors
        .get(usize::from(
            fixture.map.sidedefs[usize::from(sidedef)].sector,
        ))
        .map(|sector| sector.source.record_index)
}

/// Classifies all decoded-style source occurrences using the shared ordered
/// BSP observation. Only a retained terminal solid-range fact may produce a
/// rejection; every other absence fails open.
pub fn observe_topology_admission(
    fixture: &DoomVisibilityFixture,
) -> Result<TopologyAdmissionManifest, DoomGeometryError> {
    let observation = fixture.observe_classic_bsp()?;
    let terminal_subsectors = terminal_solid_subsectors(&observation.watched_subsector_elisions);
    let mut records = Vec::new();

    for (subsector_index, subsector) in fixture.map.subsectors.iter().enumerate() {
        let source_subsector = u16::try_from(subsector_index).expect("fixture subsector fits u16");
        let terminal = terminal_subsectors.contains(&source_subsector);
        let visited = observation.visited_subsectors.contains(&source_subsector);
        let first_seg = usize::from(subsector.first_seg);
        let end_seg = first_seg + usize::from(subsector.seg_count);
        let source_sector =
            (first_seg..end_seg).find_map(|seg_index| source_sector_for_seg(fixture, seg_index));

        let (plane_result, plane_reason) = if terminal {
            (
                TopologyAdmissionResult::Rejected,
                TopologyAdmissionReason::PositiveTerminalSolidRange,
            )
        } else if visited {
            (
                TopologyAdmissionResult::Admitted,
                TopologyAdmissionReason::VisitedSourcePlane,
            )
        } else {
            (
                TopologyAdmissionResult::UnresolvedFailOpen,
                TopologyAdmissionReason::ProjectionOrTraversalAmbiguous,
            )
        };
        records.push(TopologyAdmissionRecord {
            family: TopologyContributionFamily::Floor,
            result: plane_result,
            reason: plane_reason,
            source_subsector: Some(source_subsector),
            source_seg: None,
            source_linedef: None,
            source_sidedef: None,
            source_sector,
            runtime_floor_height: None,
            runtime_ceiling_height: None,
        });
        let ceiling_family = source_sector
            .and_then(|record_index| {
                fixture
                    .map
                    .sectors
                    .iter()
                    .find(|sector| sector.source.record_index == record_index)
            })
            .map(|sector| {
                if sector.ceiling_texture == "F_SKY1" {
                    TopologyContributionFamily::SkyPlane
                } else {
                    TopologyContributionFamily::Ceiling
                }
            })
            .unwrap_or(TopologyContributionFamily::Ceiling);
        records.push(TopologyAdmissionRecord {
            family: ceiling_family,
            result: plane_result,
            reason: plane_reason,
            source_subsector: Some(source_subsector),
            source_seg: None,
            source_linedef: None,
            source_sidedef: None,
            source_sector,
            runtime_floor_height: None,
            runtime_ceiling_height: None,
        });

        for seg_index in first_seg..end_seg {
            let seg = &fixture.map.segs[seg_index];
            let admitted = observation
                .admitted_seg_records
                .contains(&seg.source.record_index);
            let (result, reason) = if terminal {
                (
                    TopologyAdmissionResult::Rejected,
                    TopologyAdmissionReason::PositiveTerminalSolidRange,
                )
            } else if observation.near_plane_fail_open > 0 {
                (
                    TopologyAdmissionResult::UnresolvedFailOpen,
                    TopologyAdmissionReason::ProjectionOrTraversalAmbiguous,
                )
            } else if admitted {
                (
                    TopologyAdmissionResult::Admitted,
                    TopologyAdmissionReason::OrderedSourceSegAdmitted,
                )
            } else {
                (
                    TopologyAdmissionResult::UnresolvedFailOpen,
                    TopologyAdmissionReason::ProjectionOrTraversalAmbiguous,
                )
            };
            let linedef = &fixture.map.linedefs[usize::from(seg.linedef)];
            let sidedef = selected_sidedef(fixture, seg_index)
                .map(|index| fixture.map.sidedefs[usize::from(index)].source.record_index);
            for family in wall_families(fixture, seg_index) {
                records.push(TopologyAdmissionRecord {
                    family,
                    result,
                    reason,
                    source_subsector: Some(source_subsector),
                    source_seg: Some(seg.source.record_index),
                    source_linedef: Some(linedef.source.record_index),
                    source_sidedef: sidedef,
                    source_sector,
                    runtime_floor_height: None,
                    runtime_ceiling_height: None,
                });
            }
        }
    }

    Ok(TopologyAdmissionManifest::from_records(
        fixture.name.clone(),
        records,
    ))
}

/// Applies an immutable current-height snapshot, then classifies the original
/// doorway boundary. This observes current spatial state; it does not model
/// activation, timing, waiting, reversal, or movement policy.
pub fn observe_dynamic_ceiling_admission(
    fixture: &DoomVisibilityFixture,
    snapshot: DoomSectorRuntimeHeightSnapshot,
) -> Result<TopologyAdmissionManifest, DoomGeometryError> {
    let projected = fixture.with_runtime_height_snapshots(&[snapshot])?;
    let boundary_present = !lower_doom_two_sided_wall_bands(&projected.map)?.is_empty();
    let result = if boundary_present {
        TopologyAdmissionResult::Admitted
    } else {
        TopologyAdmissionResult::Rejected
    };
    let reason = if boundary_present {
        TopologyAdmissionReason::CurrentHeightBoundaryPresent
    } else {
        TopologyAdmissionReason::CurrentHeightOpening
    };
    Ok(TopologyAdmissionManifest::from_records(
        projected.name.clone(),
        vec![TopologyAdmissionRecord {
            family: TopologyContributionFamily::DynamicCeilingBoundary,
            result,
            reason,
            source_subsector: None,
            source_seg: None,
            source_linedef: None,
            source_sidedef: None,
            source_sector: Some(snapshot.source_sector.record_index),
            runtime_floor_height: snapshot.floor_height,
            runtime_ceiling_height: snapshot.ceiling_height,
        }],
    ))
}

/// Retains the current platform plane with its explicit height identity. A
/// moving platform changes spatial state, not the source plane's reachability.
pub fn observe_dynamic_floor_admission(
    fixture: &DoomVisibilityFixture,
    snapshot: DoomSectorRuntimeHeightSnapshot,
) -> Result<TopologyAdmissionManifest, DoomGeometryError> {
    let projected = fixture.with_runtime_height_snapshots(&[snapshot])?;
    Ok(TopologyAdmissionManifest::from_records(
        projected.name.clone(),
        vec![TopologyAdmissionRecord {
            family: TopologyContributionFamily::DynamicFloorPlane,
            result: TopologyAdmissionResult::Admitted,
            reason: TopologyAdmissionReason::CurrentFloorSnapshot,
            source_subsector: Some(0),
            source_seg: None,
            source_linedef: None,
            source_sidedef: None,
            source_sector: Some(snapshot.source_sector.record_index),
            runtime_floor_height: snapshot.floor_height,
            runtime_ceiling_height: snapshot.ceiling_height,
        }],
    ))
}

/// One pose in the Slice 3 whole-contribution falsifier.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WholeContributionPoseObservation {
    pub label: String,
    pub viewer_position: [i16; 2],
    pub topology_admitted: usize,
    pub topology_rejected: usize,
    pub topology_unresolved_fail_open: usize,
    pub far_source_seg: u32,
    pub far_result: TopologyAdmissionResult,
    pub far_reason: TopologyAdmissionReason,
    pub unrelated_rejections: Vec<String>,
    pub overlapping_columns: usize,
    pub surviving_columns: usize,
    pub ordinary_depth_authority_in_overlap: bool,
    pub visible_source_invalid_columns_if_retained_whole: usize,
    pub requires_partial_survival: bool,
    pub topology_fingerprint: String,
}

/// Bounded Slice 3 result for whole-contribution admission.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WholeContributionFalsifierObservation {
    pub fixture: String,
    pub original_geometry_unchanged: bool,
    pub alternative_a_retains_whole_far_contribution: bool,
    pub poses: Vec<WholeContributionPoseObservation>,
    pub negative_control_requires_partial_survival: bool,
    pub fingerprint: String,
}

fn paired_sky_boundary_has_ordinary_depth_authority(fixture: &DoomVisibilityFixture) -> bool {
    let Some(sidedef_index) = selected_sidedef(fixture, 0) else {
        return false;
    };
    let sidedef = &fixture.map.sidedefs[usize::from(sidedef_index)];
    sidedef.upper_texture != "-" || sidedef.lower_texture != "-" || sidedef.middle_texture != "-"
}

fn observe_whole_contribution_pose(
    label: &str,
    fixture: &DoomVisibilityFixture,
) -> Result<WholeContributionPoseObservation, DoomGeometryError> {
    let topology = observe_topology_admission(fixture)?;
    let expressiveness = crate::observe_partial_coverage_expressiveness_for_fixture(fixture)?;
    let far_record = topology
        .records
        .iter()
        .find(|record| record.source_seg == Some(expressiveness.far_wall_source_seg))
        .expect("the partial fixture must retain its far source contribution");
    let ordinary_depth_authority_in_overlap =
        paired_sky_boundary_has_ordinary_depth_authority(fixture);
    let visible_source_invalid_columns_if_retained_whole = if far_record.result
        != TopologyAdmissionResult::Rejected
        && !ordinary_depth_authority_in_overlap
    {
        expressiveness.overlapping_columns
    } else {
        0
    };
    let unrelated_rejections = topology
        .records
        .iter()
        .filter(|record| {
            record.result == TopologyAdmissionResult::Rejected
                && record.source_seg != Some(expressiveness.far_wall_source_seg)
        })
        .map(|record| {
            format!(
                "subsector={:?},seg={:?},family={:?},reason={:?}",
                record.source_subsector, record.source_seg, record.family, record.reason
            )
        })
        .collect::<Vec<_>>();
    let requires_partial_survival = expressiveness.requires_source_fragments
        && visible_source_invalid_columns_if_retained_whole > 0;

    Ok(WholeContributionPoseObservation {
        label: label.to_owned(),
        viewer_position: fixture.viewer.position,
        topology_admitted: topology.admitted,
        topology_rejected: topology.rejected,
        topology_unresolved_fail_open: topology.unresolved_fail_open,
        far_source_seg: expressiveness.far_wall_source_seg,
        far_result: far_record.result,
        far_reason: far_record.reason,
        unrelated_rejections,
        overlapping_columns: expressiveness.overlapping_columns,
        surviving_columns: expressiveness.far_only_columns,
        ordinary_depth_authority_in_overlap,
        visible_source_invalid_columns_if_retained_whole,
        requires_partial_survival,
        topology_fingerprint: topology.fingerprint,
    })
}

/// Runs the same complete source geometry through alternatives A and B at the
/// baseline, a bounded lateral jitter, and a nearer viewer pose.
///
/// Alternative A's fact is explicit: it retains the far contribution whole.
/// Alternative B may only accept or reject that same contribution; this
/// observation never invokes source-fragment reconstruction as a repair.
pub fn observe_whole_contribution_falsifier(
) -> Result<WholeContributionFalsifierObservation, DoomGeometryError> {
    let baseline = crate::partial_paired_sky_far_control_fixture()
        .expect("the built-in partial fixture remains valid");
    let source_geometry_before = format!("{:?}", baseline.map);
    let mut jitter = baseline.clone();
    jitter.viewer.position[0] += 2;
    let mut near = baseline.clone();
    near.viewer.position[1] += 16;
    let poses = vec![
        observe_whole_contribution_pose("baseline", &baseline)?,
        observe_whole_contribution_pose("jitter-x-plus-2", &jitter)?,
        observe_whole_contribution_pose("near-plus-16", &near)?,
    ];
    let original_geometry_unchanged = format!("{:?}", baseline.map) == source_geometry_before
        && format!("{:?}", jitter.map) == source_geometry_before
        && format!("{:?}", near.map) == source_geometry_before;
    let negative_control_requires_partial_survival =
        poses.iter().all(|pose| pose.requires_partial_survival);
    let trace = format!(
        "fixture={};geometry-unchanged={original_geometry_unchanged};a-retains-whole=true;poses={poses:?};negative-control={negative_control_requires_partial_survival}",
        baseline.name
    );
    let fingerprint = blake3::hash(trace.as_bytes()).to_hex().to_string();

    Ok(WholeContributionFalsifierObservation {
        fixture: baseline.name,
        original_geometry_unchanged,
        alternative_a_retains_whole_far_contribution: true,
        poses,
        negative_control_requires_partial_survival,
        fingerprint,
    })
}

#[cfg(test)]
mod tests {
    use doom_map_provider::DoomSourceRecord;

    use super::*;
    use crate::{
        dynamic_door_snapshot_fixture, masked_middle_topology_fixture,
        moving_platform_snapshot_fixture, one_sky_identity_differential_fixture,
        paired_sky_far_control_fixture, projection_near_plane_crossing_fixture,
        source_terminal_boundary_fixture, vertical_aperture_control_fixture,
    };

    #[test]
    fn open_aperture_admits_both_source_leaves() {
        let manifest =
            observe_topology_admission(&source_terminal_boundary_fixture(true).unwrap()).unwrap();
        assert_eq!(manifest.rejected, 0, "{}", manifest.trace);
        assert!(manifest.records.iter().any(|record| {
            record.source_subsector == Some(1) && record.result == TopologyAdmissionResult::Admitted
        }));
    }

    #[test]
    fn terminal_solid_rejects_far_occurrences_with_positive_provenance() {
        let manifest =
            observe_topology_admission(&source_terminal_boundary_fixture(false).unwrap()).unwrap();
        assert!(manifest.rejected > 0, "{}", manifest.trace);
        assert!(manifest
            .records
            .iter()
            .filter(|record| { record.source_subsector == Some(1) })
            .all(|record| {
                record.result == TopologyAdmissionResult::Rejected
                    && record.reason == TopologyAdmissionReason::PositiveTerminalSolidRange
            }));
    }

    #[test]
    fn sky_identity_does_not_change_horizontal_reachability() {
        let paired =
            observe_topology_admission(&paired_sky_far_control_fixture().unwrap()).unwrap();
        let one =
            observe_topology_admission(&one_sky_identity_differential_fixture().unwrap()).unwrap();
        let paired_results = paired
            .records
            .iter()
            .map(|record| (record.source_subsector, record.source_seg, record.result))
            .collect::<Vec<_>>();
        let one_results = one
            .records
            .iter()
            .map(|record| (record.source_subsector, record.source_seg, record.result))
            .collect::<Vec<_>>();
        assert_eq!(paired_results, one_results);
    }

    #[test]
    fn vertical_aperture_admits_upper_and_lower_roles_independently() {
        let manifest =
            observe_topology_admission(&vertical_aperture_control_fixture().unwrap()).unwrap();
        for family in [
            TopologyContributionFamily::WallUpper,
            TopologyContributionFamily::WallLower,
        ] {
            assert!(manifest.records.iter().any(|record| {
                record.family == family && record.result == TopologyAdmissionResult::Admitted
            }));
        }
    }

    #[test]
    fn masked_middle_is_admitted_without_terminal_authority() {
        let manifest =
            observe_topology_admission(&masked_middle_topology_fixture().unwrap()).unwrap();
        let cutout = manifest
            .records
            .iter()
            .find(|record| record.family == TopologyContributionFamily::CutoutMiddle)
            .expect("fixture retains its authored masked middle");
        assert_eq!(cutout.result, TopologyAdmissionResult::Admitted);
        assert_eq!(manifest.rejected, 0);
    }

    #[test]
    fn declared_door_snapshots_change_only_the_boundary_outcome() {
        let fixture = dynamic_door_snapshot_fixture().unwrap();
        let source_sector = fixture.map.sectors[1].source;
        let closed = observe_dynamic_ceiling_admission(
            &fixture,
            DoomSectorRuntimeHeightSnapshot {
                source_sector,
                floor_height: None,
                ceiling_height: Some(0),
            },
        )
        .unwrap();
        let opened = observe_dynamic_ceiling_admission(
            &fixture,
            DoomSectorRuntimeHeightSnapshot {
                source_sector,
                floor_height: None,
                ceiling_height: Some(128),
            },
        )
        .unwrap();
        assert_eq!(closed.records[0].result, TopologyAdmissionResult::Admitted);
        assert_eq!(opened.records[0].result, TopologyAdmissionResult::Rejected);
        assert_eq!(
            closed.records[0].source_sector,
            opened.records[0].source_sector
        );
    }

    #[test]
    fn declared_platform_snapshots_retain_identity_and_current_height() {
        let fixture = moving_platform_snapshot_fixture().unwrap();
        let source_sector = fixture.map.sectors[0].source;
        let low = observe_dynamic_floor_admission(
            &fixture,
            DoomSectorRuntimeHeightSnapshot {
                source_sector,
                floor_height: Some(0),
                ceiling_height: None,
            },
        )
        .unwrap();
        let raised = observe_dynamic_floor_admission(
            &fixture,
            DoomSectorRuntimeHeightSnapshot {
                source_sector,
                floor_height: Some(48),
                ceiling_height: None,
            },
        )
        .unwrap();
        assert_eq!(low.records[0].result, TopologyAdmissionResult::Admitted);
        assert_eq!(raised.records[0].result, TopologyAdmissionResult::Admitted);
        assert_eq!(
            low.records[0].source_sector,
            raised.records[0].source_sector
        );
        assert_ne!(low.fingerprint, raised.fingerprint);
    }

    #[test]
    fn ambiguous_projection_fails_open() {
        let manifest = observe_topology_admission(
            &projection_near_plane_crossing_fixture().expect("fixture remains valid"),
        )
        .unwrap();
        assert_eq!(manifest.rejected, 0, "{}", manifest.trace);
        assert!(manifest.unresolved_fail_open > 0, "{}", manifest.trace);
        assert!(manifest.records.iter().any(|record| {
            record.result == TopologyAdmissionResult::UnresolvedFailOpen
                && record.reason == TopologyAdmissionReason::ProjectionOrTraversalAmbiguous
        }));
    }

    #[test]
    fn manifest_is_deterministic() {
        let fixture = source_terminal_boundary_fixture(false).unwrap();
        let first = observe_topology_admission(&fixture).unwrap();
        let second = observe_topology_admission(&fixture).unwrap();
        assert_eq!(first, second);
        assert_eq!(first.fingerprint.len(), 64);
    }

    #[test]
    fn admission_observation_does_not_mutate_source_geometry() {
        let fixture = source_terminal_boundary_fixture(false).unwrap();
        let before = fixture.structural_manifest();
        let _manifest = observe_topology_admission(&fixture).unwrap();
        assert_eq!(fixture.structural_manifest(), before);
    }

    #[test]
    fn unavailable_snapshot_remains_an_explicit_error() {
        let fixture = moving_platform_snapshot_fixture().unwrap();
        let error = observe_dynamic_floor_admission(
            &fixture,
            DoomSectorRuntimeHeightSnapshot {
                source_sector: DoomSourceRecord {
                    lump_index: 0,
                    record_index: 99,
                },
                floor_height: Some(48),
                ceiling_height: None,
            },
        )
        .unwrap_err();
        assert!(matches!(
            error,
            DoomGeometryError::RuntimeSnapshotSectorUnavailable { .. }
        ));
    }

    #[test]
    fn whole_contribution_admission_is_falsified_without_fragment_repair() {
        let observation = observe_whole_contribution_falsifier().unwrap();
        assert!(observation.original_geometry_unchanged);
        assert!(observation.alternative_a_retains_whole_far_contribution);
        assert!(observation.negative_control_requires_partial_survival);
        assert_eq!(observation.poses.len(), 3);
        for pose in &observation.poses {
            assert_ne!(pose.far_result, TopologyAdmissionResult::Rejected);
            assert!(pose.overlapping_columns > 0);
            assert!(pose.surviving_columns > 0);
            assert!(!pose.ordinary_depth_authority_in_overlap);
            assert_eq!(
                pose.visible_source_invalid_columns_if_retained_whole,
                pose.overlapping_columns
            );
            assert!(pose.requires_partial_survival);
        }
    }
}
