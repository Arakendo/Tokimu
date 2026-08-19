//! Source-record and non-mutating runtime evidence reports.
//!
//! These reports inspect retained Doom meaning without owning application
//! lifecycle, renderer state, or source mutation policy.

use super::super::*;
use super::ordered_causality::{
    ordered_six_ray_cases, OrderedSixRayExpectedTarget as ExpectedTarget,
};
use super::tokimu_spatial_bake::SpatialRayShadow;
use hello_doom_e1m1::ordered_occurrence::prepare_ordered_occurrence_declarations;
use hello_doom_e1m1::things::{
    classify_e1m1_thing_kind, resolve_doom_sprite_patch, select_doom_sprite_view_rotation,
};

pub(crate) fn report_doom_thing_classification(
    things: &[DoomThing],
    frames: &[DoomSpriteFrameRotation],
    viewer: [i16; 2],
) {
    let mut family_counts = BTreeMap::new();
    let mut kind_counts = BTreeMap::new();
    let mut unknown_counts = BTreeMap::new();
    for source in things {
        *kind_counts.entry(source.kind).or_insert(0_usize) += 1;
        match classify_e1m1_thing_kind(source.kind) {
            Some(classification) => {
                *family_counts
                    .entry(classification.family)
                    .or_insert(0_usize) += 1;
            }
            None => *unknown_counts.entry(source.kind).or_insert(0_usize) += 1,
        }
    }
    let families = family_counts
        .iter()
        .map(|(family, count)| format!("{family:?}={count}"))
        .collect::<Vec<_>>()
        .join(",");
    let kinds = kind_counts
        .iter()
        .map(|(kind, count)| {
            classify_e1m1_thing_kind(*kind).map_or_else(
                || format!("{kind}:unknown:{count}"),
                |classification| {
                    let sprite = match (classification.initial_sprite, classification.initial_frame)
                    {
                        (Some(root), Some(frame)) => format!("{root}{frame}"),
                        _ => "none".to_owned(),
                    };
                    format!(
                        "{kind}:{}:{:?}:sprite={sprite}:count={count}",
                        classification.name, classification.family,
                    )
                },
            )
        })
        .collect::<Vec<_>>()
        .join(" | ");
    let unknown = unknown_counts.values().sum::<usize>();
    let mut sprite_bearing = 0_usize;
    let mut sprite_resolved = 0_usize;
    let mut rotation_zero = 0_usize;
    let mut mirrored = 0_usize;
    let mut sprite_errors = Vec::new();
    for source in things {
        let Some(classification) = classify_e1m1_thing_kind(source.kind) else {
            continue;
        };
        let (Some(sprite), Some(frame)) =
            (classification.initial_sprite, classification.initial_frame)
        else {
            continue;
        };
        sprite_bearing += 1;
        let rotation = select_doom_sprite_view_rotation(
            [f64::from(viewer[0]), f64::from(viewer[1])],
            [f64::from(source.x), f64::from(source.y)],
            f64::from(source.angle),
        );
        match resolve_doom_sprite_patch(frames, sprite, frame, rotation) {
            Ok(selection) => {
                sprite_resolved += 1;
                rotation_zero += usize::from(selection.source_rotation == 0);
                mirrored += usize::from(selection.mirrored);
            }
            Err(error) => sprite_errors.push(format!(
                "thing={}:kind={}:error={error:?}",
                source.source.record_index, source.kind
            )),
        }
    }
    println!(
        "E1M1 Slice 9 thing-classification observation: source-things={}; classified={}; unknown={unknown}; families=[{families}]; sprite-bearing={sprite_bearing}; sprite-patches-resolved={sprite_resolved}; rotation-zero={rotation_zero}; view-rotated={}; mirrored-selections={mirrored}; sprite-errors=[{}]; billboard-policy=unrealized; map-projectiles=0; projectile-policy=runtime-created-not-map-authored; source-flags-retained-not-filtered; runtime-state-created=false; renderer-initialized=false; kinds={kinds}",
        things.len(),
        things.len() - unknown,
        sprite_resolved - rotation_zero,
        sprite_errors.join(" | "),
    );
}

/// Exercises the shared Slice 6B entry point as a composition-local refresh
/// sequence. This is headless structural evidence: it proves complete results
/// are prepared before replacing the active declaration set, not pixel output.
pub(crate) fn report_ordered_occurrence_live_refresh(scene: &SceneInput) -> PlatformResult<()> {
    let cutout_materials = scene
        .cutout_uploads
        .iter()
        .map(|upload| (upload.source_name.clone(), upload.material))
        .collect::<BTreeMap<_, _>>();
    let origin = scene.spawn_observer.source_position;
    let heading = f64::from(scene.spawn_observer.source_angle).to_radians();
    let eye_height = scene.spawn_observer.position.y as i16;
    let poses = [
        ("spawn", origin, heading),
        (
            "yaw-minus-2-degrees",
            origin,
            heading - 2.0_f64.to_radians(),
        ),
        ("yaw-plus-2-degrees", origin, heading + 2.0_f64.to_radians()),
        (
            "forward-16",
            [origin[0].saturating_add(16), origin[1]],
            heading,
        ),
        ("return-spawn", origin, heading),
    ];
    let mut active = None;
    let mut rows = Vec::new();
    for (ordinal, (label, viewer, pose_heading)) in poses.into_iter().enumerate() {
        let prepared = prepare_ordered_occurrence_declarations(
            &scene.door_geometry_source.map,
            viewer,
            pose_heading,
            eye_height,
            &scene.door_geometry_source.wall_extents,
            &scene.door_geometry_source.wall_materials,
            &cutout_materials,
            &scene.opaque_uploads,
        )
        .map_err(io::Error::other)?;
        let opaque = prepared.opaque_draws.len();
        let cutouts = prepared.cutout_draws.len();
        // The assignment follows successful preparation, so a failed future
        // refresh leaves `active` unchanged rather than exposing a partial set.
        active = Some(prepared);
        rows.push(format!(
            "{ordinal}:{label}:source=({},{}):heading={:.3}:opaque={opaque}:cutout={cutouts}",
            viewer[0],
            viewer[1],
            pose_heading.to_degrees(),
        ));
    }
    let active = active.expect("the nonempty refresh sequence installs a result");
    println!(
        "E1M1 Slice 6B live-refresh structural replay: poses={}; active-opaque={}; active-cutout={}; swap=prepare-then-replace; stale-or-partial-declarations=not-installed; generic-filter=none; conservation={}; rows=[{}]",
        poses.len(),
        active.opaque_draws.len(),
        active.cutout_draws.len(),
        active.conservation_report,
        rows.join(" | "),
    );
    Ok(())
}

/// Replays the six retained Slice 6B rays against the literal ordered-result
/// handoff. The expected result is deliberately expressed in source identity:
/// five rejected wall/plane contributions must have no final declaration,
/// while the reached ceiling may survive only through partial-plane output.
pub(crate) fn report_ordered_occurrence_six_ray_handoff(scene: &SceneInput) -> PlatformResult<()> {
    let cutout_materials = scene
        .cutout_uploads
        .iter()
        .map(|upload| (upload.source_name.clone(), upload.material))
        .collect::<BTreeMap<_, _>>();
    let spatial_shadow = SpatialRayShadow::build(scene)?;
    let mut reports = Vec::new();
    for case in ordered_six_ray_cases() {
        let spatial_hit = spatial_shadow
            // Headless reports run before the optional comparative re-embed;
            // the prepared inventory is still in its source-aligned frame.
            .query_source_ray(
                DoomComparativeEmbedding::CurrentReflected,
                case.origin,
                case.direction,
            )?
            .ok_or_else(|| {
                io::Error::other(format!(
                    "Slice 6B ray {} expected a global prepared-triangle BVH hit",
                    case.name,
                ))
            })?;
        if spatial_hit.source_label != case.expected_global_label {
            return Err(io::Error::other(format!(
                "Slice 6B ray {} expected global BVH source {} but hit {} at distance {:.3}",
                case.name,
                case.expected_global_label,
                spatial_hit.source_label,
                spatial_hit.distance,
            ))
            .into());
        }
        let viewer = [case.origin[0].round() as i16, case.origin[1].round() as i16];
        let heading = case.direction[1].atan2(case.direction[0]);
        let eye_height = case.origin[2].round() as i16;
        let observation = prepare_ordered_occurrence_submission(
            &scene.door_geometry_source.map,
            viewer,
            heading,
            eye_height,
            &scene.door_geometry_source.wall_extents,
            &scene.door_geometry_source.wall_materials,
            &cutout_materials,
            &scene.opaque_uploads,
        )
        .map_err(io::Error::other)?;
        observation
            .verify_conservation()
            .map_err(io::Error::other)?;

        let result = match case.expected {
            ExpectedTarget::RejectedWallSegs { source_segs, .. } => {
                let declaration_count = observation
                    .walls
                    .prepared_declarations
                    .iter()
                    .filter(|declaration| source_segs.contains(&declaration.occurrence.source_seg))
                    .count();
                let dispositions = observation
                    .source
                    .dispositions
                    .iter()
                    .filter(|disposition| source_segs.contains(&disposition.source_seg))
                    .collect::<Vec<_>>();
                let terminal = dispositions
                    .iter()
                    .filter(|disposition| {
                        disposition.kind == OrderedSourceDispositionKind::TerminalRejected
                    })
                    .count();
                if declaration_count != 0 || terminal != dispositions.len() {
                    return Err(io::Error::other(format!(
                        "Slice 6B ray {} expected terminal wall rejection: segs={source_segs:?} declarations={declaration_count} terminal={terminal}/{}",
                        case.name,
                        dispositions.len(),
                    ))
                    .into());
                }
                format!(
                    "{}=terminal-wall:segs={source_segs:?}:dispositions={terminal}:declarations=0",
                    case.name,
                )
            }
            ExpectedTarget::RejectedPlane { subsector, kind } => {
                let associations = observation
                    .planes
                    .associations
                    .iter()
                    .filter(|association| {
                        association.source_subsector == subsector && association.kind == kind
                    })
                    .count();
                let instance_ordinals = observation
                    .planes
                    .plane_destinations
                    .iter()
                    .filter(|destination| {
                        destination.source_subsector == subsector && destination.kind == kind
                    })
                    .map(|destination| destination.plane_instance_ordinal)
                    .collect::<BTreeSet<_>>();
                let declarations = observation
                    .plane_lowering
                    .prepared_declarations
                    .iter()
                    .filter(|declaration| {
                        declaration.source_subsector == subsector
                            && instance_ordinals.contains(&declaration.plane_instance_ordinal)
                    })
                    .count();
                let dispositions = observation
                    .plane_lowering
                    .source_dispositions
                    .iter()
                    .filter(|disposition| {
                        disposition.source_subsector == subsector
                            && instance_ordinals.contains(&disposition.plane_instance_ordinal)
                    })
                    .count();
                if associations != 0
                    || !instance_ordinals.is_empty()
                    || dispositions != 0
                    || declarations != 0
                {
                    return Err(io::Error::other(format!(
                        "Slice 6B ray {} expected source-protocol rejection for {:?} plane subsector {} but found associations={associations} destinations={} dispositions={dispositions} declarations={declarations}",
                        case.name,
                        kind,
                        subsector,
                        instance_ordinals.len(),
                    ))
                    .into());
                }
                format!(
                    "{}=source-protocol-rejected-plane:subsector={subsector}:kind={kind:?}:associations=0:destinations=0:dispositions=0:declarations=0",
                    case.name,
                )
            }
            ExpectedTarget::PartialPlane { subsector, kind } => {
                let destinations = observation
                    .planes
                    .plane_destinations
                    .iter()
                    .filter(|destination| {
                        destination.source_subsector == subsector && destination.kind == kind
                    })
                    .collect::<Vec<_>>();
                let instance_ordinals = destinations
                    .iter()
                    .map(|destination| destination.plane_instance_ordinal)
                    .collect::<BTreeSet<_>>();
                let declarations = observation
                    .plane_lowering
                    .prepared_declarations
                    .iter()
                    .filter(|declaration| {
                        declaration.source_subsector == subsector
                            && instance_ordinals.contains(&declaration.plane_instance_ordinal)
                    })
                    .count();
                let partial_sources = observation
                    .plane_lowering
                    .source_dispositions
                    .iter()
                    .filter(|disposition| {
                        disposition.source_subsector == subsector
                            && instance_ordinals.contains(&disposition.plane_instance_ordinal)
                            && disposition.kind == OrderedSourceDispositionKind::PartialPlane
                    })
                    .count();
                let intervals = destinations
                    .iter()
                    .flat_map(|destination| destination.view_intervals.iter().copied())
                    .collect::<Vec<_>>();
                if declarations == 0 || partial_sources == 0 || intervals.is_empty() {
                    return Err(io::Error::other(format!(
                        "Slice 6B ray {} expected partial {:?} plane subsector {}: destinations={} intervals={intervals:?} partial_sources={partial_sources} declarations={declarations}",
                        case.name,
                        kind,
                        subsector,
                        destinations.len(),
                    ))
                    .into());
                }
                let instance = destinations
                    .first()
                    .and_then(|destination| {
                        observation
                            .planes
                            .plane_instances
                            .get(destination.plane_instance_ordinal)
                    })
                    .ok_or_else(|| {
                        io::Error::other(format!(
                            "Slice 6B ray {} partial plane has no retained instance",
                            case.name,
                        ))
                    })?;
                let exact = prepare_doom_ordered_coverage(
                    &scene.door_geometry_source.map,
                    &scene.door_geometry_source.wall_extents,
                    viewer,
                    heading,
                    f64::from(eye_height),
                    true,
                )?;
                let reconstruction = reconstruct_doom_seg_classic_plane_cells(
                    &exact.vertical.plane_spans,
                    viewer,
                    heading,
                    f64::from(eye_height),
                );
                let exact_kind = match kind {
                    OrderedPlaneKind::Floor => DoomSegClassicPlaneKind::Floor,
                    OrderedPlaneKind::Ceiling => DoomSegClassicPlaneKind::Ceiling,
                };
                let exact_cells = reconstruction
                    .cells
                    .iter()
                    .filter(|cell| {
                        cell.source_sector == instance.source_sector
                            && cell.key.kind == exact_kind
                            && cell.key.height == instance.source_height
                            && cell.key.texture == instance.texture
                            && cell.key.light == instance.light_level
                    })
                    .collect::<Vec<_>>();
                let exact_source_segs = exact_cells
                    .iter()
                    .map(|cell| cell.source_seg)
                    .collect::<BTreeSet<_>>();
                let exact_subsectors = scene
                    .door_geometry_source
                    .map
                    .subsectors
                    .iter()
                    .filter(|subsector_record| {
                        let start = usize::from(subsector_record.first_seg);
                        let end = start + usize::from(subsector_record.seg_count);
                        scene.door_geometry_source.map.segs[start..end]
                            .iter()
                            .any(|seg| exact_source_segs.contains(&seg.source.record_index))
                    })
                    .map(|subsector_record| subsector_record.source.record_index)
                    .collect::<BTreeSet<_>>();
                if exact_cells.is_empty() {
                    return Err(io::Error::other(format!(
                        "Slice 6B ray {} partial plane has no exact source plane-domain cells",
                        case.name,
                    ))
                    .into());
                }
                format!(
                    "{}=partial-plane:subsector={subsector}:kind={kind:?}:destinations={}:intervals={intervals:?}:partial-sources={partial_sources}:declarations={declarations}:exact-plane-domain-cells={}:exact-plane-domain-source-segs={exact_source_segs:?}:exact-plane-domain-subsectors={exact_subsectors:?}",
                    case.name,
                    destinations.len(),
                    exact_cells.len(),
                )
            }
        };
        reports.push(format!(
            "{};bvh=geometrically-relevant:source={}:member={}:distance={:.3}:visited-nodes={}:tested-members={}",
            result,
            spatial_hit.source_label,
            spatial_hit.member_identity,
            spatial_hit.distance,
            spatial_hit.visited_nodes,
            spatial_hit.tested_members,
        ));
    }

    println!(
        "E1M1 Slice 6B six-ray BVH/source shadow replay: cases={}; conservation=balanced; bvh-role=geometric-relevance-only; source-role=participation-authority; submission-changes=none; results=[{}]",
        reports.len(),
        reports.join(" | "),
    );
    Ok(())
}

/// Correlates immutable E1M1 runtime-height snapshots with the same ordered
/// source-occurrence preparation seam used by the integrated candidate. This
/// deliberately supplies current spatial facts without recreating activation,
/// timing, waiting, or reversal policy.
pub(crate) fn report_ordered_occurrence_runtime_snapshots(
    scene: &SceneInput,
) -> PlatformResult<()> {
    let map = &scene.door_geometry_source.map;
    let cutout_materials = scene
        .cutout_uploads
        .iter()
        .map(|upload| (upload.source_name.clone(), upload.material))
        .collect::<BTreeMap<_, _>>();
    let prepare = |map: &DoomMapCore, viewer: [i16; 2], heading: f64, eye_height: i16| {
        prepare_ordered_occurrence_submission(
            map,
            viewer,
            heading,
            eye_height,
            &scene.door_geometry_source.wall_extents,
            &scene.door_geometry_source.wall_materials,
            &cutout_materials,
            &scene.opaque_uploads,
        )
    };
    let source_sector = |record_index: u32| {
        map.sectors
            .iter()
            .find(|sector| sector.source.record_index == record_index)
            .map(|sector| sector.source)
            .ok_or_else(|| format!("E1M1 sector record {record_index} is unavailable"))
    };

    let door_sector = source_sector(4).map_err(io::Error::other)?;
    let door_open_map = project_doom_sector_runtime_heights(
        map,
        &[DoomSectorRuntimeHeightSnapshot {
            source_sector: door_sector,
            floor_height: None,
            ceiling_height: Some(68),
        }],
    )?;
    let platform_sector = source_sector(70).map_err(io::Error::other)?;
    let platform_low_map = project_doom_sector_runtime_heights(
        map,
        &[DoomSectorRuntimeHeightSnapshot {
            source_sector: platform_sector,
            floor_height: Some(-48),
            ceiling_height: None,
        }],
    )?;
    let prepared_draws = |observation: &OrderedPreparedSubmissionObservation| {
        observation
            .walls
            .prepared_declarations
            .iter()
            .map(|declaration| declaration.draw.clone())
            .chain(
                observation
                    .plane_lowering
                    .prepared_declarations
                    .iter()
                    .map(|declaration| declaration.draw.clone()),
            )
            .collect::<Vec<_>>()
    };

    let lowering_unresolved = |observation: &OrderedPreparedSubmissionObservation| {
        observation.walls.unresolved_fail_open
            + observation.planes.unresolved_fail_open
            + observation.plane_lowering.unresolved_fail_open
    };
    let source_labels = |draws: &[StaticDrawPlanEntry]| {
        draws
            .iter()
            .map(|draw| draw.source_label.clone())
            .collect::<BTreeSet<_>>()
    };
    let target_linedefs = |target_sector: u32| {
        map.linedefs
            .iter()
            .filter(|linedef| {
                [linedef.right_sidedef, linedef.left_sidedef]
                    .into_iter()
                    .flatten()
                    .filter_map(|index| map.sidedefs.get(usize::from(index)))
                    .filter_map(|side| map.sectors.get(usize::from(side.sector)))
                    .any(|sector| sector.source.record_index == target_sector)
            })
            .map(|linedef| linedef.source.record_index)
            .collect::<BTreeSet<_>>()
    };
    let target_views = |target_sector: u32| {
        let mut views = Vec::new();
        for linedef in &map.linedefs {
            let side_sector = |side: Option<u16>| {
                side.and_then(|index| map.sidedefs.get(usize::from(index)))
                    .and_then(|side| map.sectors.get(usize::from(side.sector)))
            };
            let right = side_sector(linedef.right_sidedef);
            let left = side_sector(linedef.left_sidedef);
            if ![right, left]
                .into_iter()
                .flatten()
                .any(|sector| sector.source.record_index == target_sector)
            {
                continue;
            }
            let (Some(start), Some(end)) = (
                map.vertices.get(usize::from(linedef.start_vertex)),
                map.vertices.get(usize::from(linedef.end_vertex)),
            ) else {
                continue;
            };
            let midpoint = [
                (f64::from(start.x) + f64::from(end.x)) * 0.5,
                (f64::from(start.y) + f64::from(end.y)) * 0.5,
            ];
            let delta = [f64::from(end.x - start.x), f64::from(end.y - start.y)];
            let length = delta[0].hypot(delta[1]);
            if length <= f64::EPSILON {
                continue;
            }
            let right_normal = [delta[1] / length, -delta[0] / length];
            for (side_name, sign, sector) in [("right", 1.0, right), ("left", -1.0, left)] {
                let Some(sector) = sector else {
                    continue;
                };
                let viewer_f64 = [
                    midpoint[0] + right_normal[0] * 48.0 * sign,
                    midpoint[1] + right_normal[1] * 48.0 * sign,
                ];
                let viewer = [viewer_f64[0].round() as i16, viewer_f64[1].round() as i16];
                let heading = (midpoint[1] - viewer_f64[1]).atan2(midpoint[0] - viewer_f64[0]);
                views.push((
                    linedef.source.record_index,
                    side_name,
                    viewer,
                    heading,
                    sector.floor_height.saturating_add(36),
                ));
            }
        }
        views
    };
    let correlate = |name: &str,
                     target_sector: u32,
                     snapshot_map: &DoomMapCore|
     -> Result<_, io::Error> {
        let mut best = None;
        let target_linedefs = target_linedefs(target_sector);
        for (linedef, side, viewer, heading, eye_height) in target_views(target_sector) {
            let baseline = prepare(map, viewer, heading, eye_height).map_err(io::Error::other)?;
            let snapshot =
                prepare(snapshot_map, viewer, heading, eye_height).map_err(io::Error::other)?;
            let baseline_draws = prepared_draws(&baseline);
            let snapshot_draws = prepared_draws(&snapshot);
            let is_target_draw = |draw: &&StaticDrawPlanEntry| match draw.source {
                StaticDrawSource::Flat { source_sector, .. } => {
                    source_sector.record_index == target_sector
                }
                StaticDrawSource::Wall { source_linedef, .. } => {
                    target_linedefs.contains(&source_linedef.record_index)
                }
            };
            let baseline_target_draws = baseline_draws
                .iter()
                .filter(is_target_draw)
                .cloned()
                .collect::<Vec<_>>();
            let snapshot_target_draws = snapshot_draws
                .iter()
                .filter(is_target_draw)
                .cloned()
                .collect::<Vec<_>>();
            let target_changed = baseline_target_draws != snapshot_target_draws;
            let changed_target_declarations = baseline_target_draws
                .iter()
                .zip(&snapshot_target_draws)
                .filter(|(baseline, snapshot)| baseline != snapshot)
                .count()
                + baseline_target_draws
                    .len()
                    .abs_diff(snapshot_target_draws.len());
            let baseline_labels = source_labels(&baseline_target_draws);
            let snapshot_labels = source_labels(&snapshot_target_draws);
            let changed_labels = baseline_labels
                .symmetric_difference(&snapshot_labels)
                .take(12)
                .cloned()
                .collect::<Vec<_>>();
            let candidate = (
                usize::from(target_changed),
                changed_target_declarations,
                linedef,
                side,
                viewer,
                heading,
                eye_height,
                baseline,
                snapshot,
                baseline_draws.len(),
                snapshot_draws.len(),
                baseline_target_draws.len(),
                snapshot_target_draws.len(),
                changed_labels,
            );
            if best.as_ref().is_none_or(
                |current: &(usize, usize, u32, _, _, _, _, _, _, _, _, _, _, _)| {
                    (candidate.0, candidate.1) > (current.0, current.1)
                },
            ) {
                best = Some(candidate);
            }
        }
        best.ok_or_else(|| io::Error::other(format!("{name} has no source-boundary-local view")))
    };

    let door = correlate("door", 4, &door_open_map)?;
    let platform = correlate("platform", 70, &platform_low_map)?;
    let door_changed = door.0 == 1;
    let platform_changed = platform.0 == 1;
    let lowering_unresolved = lowering_unresolved(&door.7)
        + lowering_unresolved(&door.8)
        + lowering_unresolved(&platform.7)
        + lowering_unresolved(&platform.8);

    println!(
        "E1M1 ordered-occurrence runtime snapshot correlation: source-view=deterministic-source-boundary-local; snapshot-policy=immutable-current-heights-only; activation-timing-policy=absent; door=[sector:4,ceiling:0->68,target-changed:{door_changed},view:linedef-{}/{},viewer:{:?},heading:{:.6},eye:{},occurrences:{}->{},source-fail-open:{}->{},declarations:{}->{},target-declarations:{}->{},changed-target-declarations:{},changed-target-labels:{:?}]; platform=[sector:70,floor:104->-48,target-changed:{platform_changed},view:linedef-{}/{},viewer:{:?},heading:{:.6},eye:{},occurrences:{}->{},source-fail-open:{}->{},declarations:{}->{},target-declarations:{}->{},changed-target-declarations:{},changed-target-labels:{:?}]; lowering-unresolved={lowering_unresolved}; same-preparation-seam=true",
        door.2,
        door.3,
        door.4,
        door.5,
        door.6,
        door.7.source.occurrences.len(),
        door.8.source.occurrences.len(),
        door.7.source.unresolved_fail_open,
        door.8.source.unresolved_fail_open,
        door.9,
        door.10,
        door.11,
        door.12,
        door.1,
        door.13,
        platform.2,
        platform.3,
        platform.4,
        platform.5,
        platform.6,
        platform.7.source.occurrences.len(),
        platform.8.source.occurrences.len(),
        platform.7.source.unresolved_fail_open,
        platform.8.source.unresolved_fail_open,
        platform.9,
        platform.10,
        platform.11,
        platform.12,
        platform.1,
        platform.13,
    );
    println!(
        "E1M1 ordered-occurrence runtime snapshot controls: door-baseline=[{}]; door-open=[{}]; platform-raised=[{}]; platform-low=[{}]",
        door.7.source.report(),
        door.8.source.report(),
        platform.7.source.report(),
        platform.8.source.report(),
    );

    if lowering_unresolved != 0 {
        return Err(format!(
            "runtime snapshot preparation retained {lowering_unresolved} unresolved lowering observations"
        )
        .into());
    }
    if !door_changed || !platform_changed {
        return Err(format!(
            "runtime snapshot correlation incomplete: door_changed={door_changed}; platform_changed={platform_changed}"
        )
        .into());
    }
    Ok(())
}

/// Narrow source-to-lowered inspection for a single wall selected from a
/// native `LOOK` observation. It keeps Doom source meaning at the corpus edge
/// and is diagnostic only: no geometry classification changes here.
pub(crate) fn report_wall_source(scene: &SceneInput, record_index: u32) {
    let map = &scene.door_geometry_source.map;
    let Some(linedef) = map
        .linedefs
        .iter()
        .find(|linedef| linedef.source.record_index == record_index)
    else {
        println!("E1M1 wall source report: linedef={record_index} missing");
        return;
    };
    let side = |index: Option<u16>| -> String {
        let Some(index) = index else {
            return "none".to_owned();
        };
        let Some(sidedef) = map.sidedefs.get(usize::from(index)) else {
            return format!("sidedef={index}:missing");
        };
        let Some(sector) = map.sectors.get(usize::from(sidedef.sector)) else {
            return format!("sidedef={index}:sector={}:missing", sidedef.sector);
        };
        format!(
            "sidedef={} sector={} heights={}/{} flats={}/{} textures={}/{}/{} offsets={}/{}",
            sidedef.source.record_index,
            sidedef.sector,
            sector.floor_height,
            sector.ceiling_height,
            sector.floor_texture,
            sector.ceiling_texture,
            sidedef.upper_texture,
            sidedef.lower_texture,
            sidedef.middle_texture,
            sidedef.x_offset,
            sidedef.y_offset,
        )
    };
    println!(
        "E1M1 wall source report: linedef={} flags=0x{:04x} special={} tag={} right/front=[{}] left/back=[{}]",
        linedef.source.record_index,
        linedef.flags,
        linedef.special,
        linedef.tag,
        side(linedef.right_sidedef),
        side(linedef.left_sidedef),
    );
    let mut generated = 0usize;
    for draw in scene.opaque_draws.iter().chain(scene.cutout_draws.iter()) {
        let StaticDrawSource::Wall {
            source_linedef,
            source_sidedef,
            source_sector,
            role,
        } = draw.source
        else {
            continue;
        };
        if source_linedef.record_index != record_index {
            continue;
        }
        let bottom = draw
            .mesh
            .positions
            .iter()
            .map(|position| position[1])
            .fold(f32::INFINITY, f32::min);
        let top = draw
            .mesh
            .positions
            .iter()
            .map(|position| position[1])
            .fold(f32::NEG_INFINITY, f32::max);
        println!(
            "generated linedef={} sidedef={} sector={} role={role:?} bottom={bottom:.1} top={top:.1} label={}",
            source_linedef.record_index,
            source_sidedef.record_index,
            source_sector.record_index,
            draw.source_label,
        );
        generated += 1;
    }
    println!("E1M1 wall source report summary: generated_wall_draws={generated}");
}

pub(crate) fn report_doom_reject(report: &DoomRejectReport) {
    println!(
        "E1M1 AR-0025 Doom REJECT source observation: sectors={}; bytes={}; player_sector={}; monster_sectors_forbidden_to_sight_player={}; monster_sectors_not_forbidden={}; meaning=classic-monster-sight-prefilter-not-render-visibility",
        report.sector_count,
        report.byte_len,
        report.player_sector,
        report.forbidden_monster_sectors,
        report.sector_count - report.forbidden_monster_sectors,
    );
}

pub(crate) fn report_doom_topology(report: &DoomTopologyReport) {
    println!(
        "E1M1 AR-0025 Doom SEGS-to-SSECTORS source observation: linedefs={}; no_subsector_membership={}; one_subsector_membership={}; multiple_subsector_membership={}; maximum_subsector_membership={}; meaning=source-topology-not-render-membership",
        report.linedefs,
        report.no_subsector_membership,
        report.one_subsector_membership,
        report.multiple_subsector_membership,
        report.maximum_subsector_membership,
    );
}

/// Reports the deterministic `Use` resolution of every nonzero E1M1 line.
/// This is source/request evidence only: accepted door intent is deliberately
/// not treated as a moved sector, and crossing-only specials remain visible.
pub(crate) fn report_doom_use_activation(source: &DoomLineActivationSource) {
    let mut accepted = 0;
    let mut no_special = 0;
    let mut wrong_activation = 0;
    let mut unsupported = 0;
    let mut invalid_target = 0;
    let mut details = Vec::new();
    for linedef in source
        .linedefs
        .iter()
        .filter(|linedef| linedef.special != 0)
    {
        let resolution = resolve_doom_line_activation(
            source,
            DoomLineActivationRequest {
                source_linedef: linedef.source,
                activation: DoomLineActivation::Use,
            },
        );
        match resolution {
            DoomLineActivationResolution::Accepted { intent, .. } => {
                accepted += 1;
                details.push(format!(
                    "{}:special{}:tag{}:accepted:{}:target:{}",
                    linedef.source.record_index,
                    linedef.special,
                    linedef.tag,
                    compact_activation_intent(intent),
                    compact_activation_target(intent),
                ));
            }
            DoomLineActivationResolution::NoSpecial { .. } => no_special += 1,
            DoomLineActivationResolution::WrongActivation { required, .. } => {
                wrong_activation += 1;
                details.push(format!(
                    "{}:special{}:tag{}:requires:{required:?}",
                    linedef.source.record_index, linedef.special, linedef.tag
                ));
            }
            DoomLineActivationResolution::UnsupportedSpecial { .. } => {
                unsupported += 1;
                details.push(format!(
                    "{}:special{}:tag{}:unsupported",
                    linedef.source.record_index, linedef.special, linedef.tag
                ));
            }
            DoomLineActivationResolution::UnknownLinedef { .. } => unreachable!(
                "a request derived from the retained E1M1 lines must resolve to one of them"
            ),
            DoomLineActivationResolution::MissingManualDoorTarget {
                missing_left_sidedef,
                ..
            } => {
                invalid_target += 1;
                details.push(format!(
                    "{}:special{}:missing-opposite-sidedef:{missing_left_sidedef:?}",
                    linedef.source.record_index, linedef.special
                ));
            }
            DoomLineActivationResolution::InvalidManualDoorTarget {
                sidedef_index,
                sector_index,
                ..
            } => {
                invalid_target += 1;
                details.push(format!(
                    "{}:special{}:invalid-target:sidedef{}:sector{}",
                    linedef.source.record_index, linedef.special, sidedef_index, sector_index
                ));
            }
        }
    }
    println!(
        "E1M1 Slice 8 use-request observation: nonzero_linedefs={}; accepted={accepted}; no_special={no_special}; wrong_activation={wrong_activation}; unsupported={unsupported}; invalid_target={invalid_target}; accepted_effects_are_not_executed=true; details={}",
        details.len(),
        details.join(" | "),
    );
}

pub(crate) fn report_doom_switch_textures(
    source: &DoomLineActivationSource,
    wall_materials: &BTreeMap<String, MaterialHandle>,
) {
    let mut details = Vec::new();
    for linedef in source.linedefs.iter().filter(|line| line.special == 11) {
        let (resolution, change) = resolve_doom_shareware_switch_texture(source, linedef.source);
        let detail = change.map_or_else(
            || {
                format!(
                    "linedef={}:resolution={resolution:?}",
                    linedef.source.record_index
                )
            },
            |change| {
                format!(
                    "linedef={}:sidedef={}:slot={:?}:{}->{}:target-material-prepared={}",
                    change.source_linedef.record_index,
                    change.source_sidedef.record_index,
                    change.slot,
                    change.before_texture,
                    change.after_texture,
                    wall_materials.contains_key(&change.after_texture),
                )
            },
        );
        details.push(detail);
    }
    let scrolling = source
        .linedefs
        .iter()
        .filter(|line| line.special == 48)
        .map(|line| {
            let sidedef = line
                .right_sidedef
                .and_then(|index| source.sidedefs.get(usize::from(index)))
                .map_or_else(
                    || "front-sidedef-unavailable".to_owned(),
                    |sidedef| format!("front-sidedef={}", sidedef.source.record_index),
                );
            format!("linedef={}:{sidedef}", line.source.record_index)
        })
        .collect::<Vec<_>>();
    println!(
        "E1M1 Slice 8 texture-state observation: exit-switch-lines={}; scrolling-lines={}; scrolling-rule=front-sidedef-u-plus-one-source-texel-per-tic; source-map-mutated=false; renderer-doom-vocabulary=false; switch-details={}; scrolling-details={}",
        details.len(),
        scrolling.len(),
        details.join(" | "),
        scrolling.join(" | "),
    );
}

pub(crate) fn report_doom_progression_sources(source: &DoomLineActivationSource) {
    let secret_sectors = source
        .sectors
        .iter()
        .filter(|sector| sector.special == 9)
        .map(|sector| sector.source.record_index.to_string())
        .collect::<Vec<_>>();
    let exit_lines = source
        .linedefs
        .iter()
        .filter(|line| line.special == 11)
        .map(|line| line.source.record_index.to_string())
        .collect::<Vec<_>>();
    println!(
        "E1M1 Slice 8 progression-source observation: secret-sectors={}; secret-sector-indices=[{}]; exit-lines={}; exit-linedef-indices=[{}]; secret-policy=first-player-entry-per-source-sector; exit-policy=accepted-front-use-to-next-bounded-wad-map; source-map-mutated=false; renderer-doom-vocabulary=false",
        secret_sectors.len(),
        secret_sectors.join(","),
        exit_lines.len(),
        exit_lines.join(","),
    );
}

/// Runs each E1M1 manual-door intent through the corpus-local, deterministic
/// moving-sector state machine. It reports height transitions only: no mesh,
/// collision, input reach, or renderer state is changed by this evidence path.
pub(crate) fn report_doom_manual_door_runtime(source: &DoomLineActivationSource) {
    let mut started = 0;
    let mut rejected = 0;
    let mut details = Vec::new();
    for linedef in source.linedefs.iter().filter(|line| line.special == 1) {
        let DoomLineActivationResolution::Accepted {
            intent: DoomLineActivationIntent::RaiseDoor { target_sector },
            ..
        } = resolve_doom_line_activation(
            source,
            DoomLineActivationRequest {
                source_linedef: linedef.source,
                activation: DoomLineActivation::Use,
            },
        )
        else {
            unreachable!("classified E1M1 code-1 lines must resolve to manual-door intent");
        };
        let mut door = match DoomManualDoorRuntime::start(
            source,
            target_sector,
            DoomManualDoorPolicy::CLASSIC_NORMAL,
        ) {
            Ok(door) => door,
            Err(error) => {
                rejected += 1;
                details.push(format!(
                    "line{}:target-sector{}:start-rejected:{error:?}",
                    linedef.source.record_index, target_sector.record_index
                ));
                continue;
            }
        };
        started += 1;
        let mut ticks = 0_u32;
        let mut reached_waiting = false;
        while door.phase != DoomManualDoorPhase::Closed && ticks < 4_096 {
            let transition = door.advance_tick();
            reached_waiting |=
                matches!(transition.after_phase, DoomManualDoorPhase::Waiting { .. });
            ticks += 1;
        }
        details.push(format!(
            "line{}:target-sector{}:closed-height{}:open-height{}:ticks{}:waited{}:final={:?}",
            linedef.source.record_index,
            target_sector.record_index,
            door.closed_ceiling_height,
            door.open_ceiling_height,
            ticks,
            reached_waiting,
            door.phase,
        ));
    }
    println!(
        "E1M1 Slice 8 manual-door runtime observation: code1_lines={}; started={started}; start_rejected={rejected}; source_map_mutated=false; presentation_mutated=false; details={}",
        details.len(),
        details.join(" | "),
    );
}

/// Runs the two tagged E1M1 moving-floor effects through their distinct
/// released-source state machines. This path changes no imported sector,
/// collision world, mesh, or renderer resource.
pub(crate) fn report_doom_moving_floor_runtime(source: &DoomLineActivationSource) {
    let mut details = Vec::new();
    let mut started = 0_usize;
    let mut rejected = 0_usize;
    for linedef in source
        .linedefs
        .iter()
        .filter(|line| matches!(line.special, 36 | 88))
    {
        let resolution = resolve_doom_line_activation(
            source,
            DoomLineActivationRequest {
                source_linedef: linedef.source,
                activation: DoomLineActivation::Cross,
            },
        );
        match resolution {
            DoomLineActivationResolution::Accepted {
                intent: DoomLineActivationIntent::LowerFloorTurbo { tag },
                ..
            } => match DoomTurboLowerFloorRuntime::start_tagged(
                source,
                tag,
                DoomTurboLowerFloorPolicy::CLASSIC,
            ) {
                Ok(mut floors) => {
                    started += floors.len();
                    for floor in &mut floors {
                        let mut ticks = 0_u32;
                        while floor.phase != DoomTurboLowerFloorPhase::Complete && ticks < 4_096 {
                            floor.advance_tick();
                            ticks += 1;
                        }
                        details.push(format!(
                            "line{}:code36:tag{}:sector{}:{}->{}:ticks{}:final={:?}:one-shot=true",
                            linedef.source.record_index,
                            tag,
                            floor.target_sector.record_index,
                            floor.start_floor_height,
                            floor.destination_floor_height,
                            ticks,
                            floor.phase,
                        ));
                    }
                }
                Err(error) => {
                    rejected += 1;
                    details.push(format!(
                        "line{}:code36:tag{}:start-rejected:{error:?}",
                        linedef.source.record_index, tag
                    ));
                }
            },
            DoomLineActivationResolution::Accepted {
                intent: DoomLineActivationIntent::PlatformDownWaitUpStay { tag },
                ..
            } => match DoomDownWaitUpStayRuntime::start_tagged(
                source,
                tag,
                DoomDownWaitUpStayPolicy::CLASSIC,
            ) {
                Ok(mut platforms) => {
                    started += platforms.len();
                    for platform in &mut platforms {
                        let mut ticks = 0_u32;
                        while platform.phase != DoomDownWaitUpStayPhase::Complete && ticks < 4_096 {
                            platform.advance_tick();
                            ticks += 1;
                        }
                        details.push(format!(
                            "line{}:code88:tag{}:sector{}:high{}:low{}:ticks{}:final={:?}:retriggerable-after-complete=true",
                            linedef.source.record_index,
                            tag,
                            platform.target_sector.record_index,
                            platform.high_floor_height,
                            platform.low_floor_height,
                            ticks,
                            platform.phase,
                        ));
                    }
                }
                Err(error) => {
                    rejected += 1;
                    details.push(format!(
                        "line{}:code88:tag{}:start-rejected:{error:?}",
                        linedef.source.record_index, tag
                    ));
                }
            },
            other => {
                rejected += 1;
                details.push(format!(
                    "line{}:special{}:unexpected-resolution:{other:?}",
                    linedef.source.record_index, linedef.special
                ));
            }
        }
    }
    println!(
        "E1M1 Slice 8 moving-floor runtime observation: source_lines={}; started_sectors={started}; start_rejected={rejected}; source_map_mutated=false; presentation_mutated=false; details={}",
        details.len(),
        details.join(" | "),
    );
}

pub(crate) fn report_flat_normals(draws: &[StaticDrawPlanEntry]) {
    let mut floors_up = 0;
    let mut floors_down = 0;
    let mut ceilings_up = 0;
    let mut ceilings_down = 0;
    for draw in draws {
        let StaticDrawSource::Flat { plane, .. } = draw.source else {
            continue;
        };
        let normal_y = draw.mesh.normals.first().map_or(0.0, |normal| normal[1]);
        let is_floor = plane == doom_geometry_provider::DoomSurfacePlane::Floor;
        match (is_floor, normal_y.is_sign_positive()) {
            (true, true) => floors_up += 1,
            (true, false) => floors_down += 1,
            (false, true) => ceilings_up += 1,
            (false, false) => ceilings_down += 1,
        }
    }
    println!(
        "E1M1 flat-normal observation: floor_up={floors_up}; floor_down={floors_down}; ceiling_up={ceilings_up}; ceiling_down={ceilings_down}"
    );
}

pub(crate) fn report_doom_membership_union(
    scene: &SceneInput,
    center: Vec3,
    radius: f32,
    include_cutouts: bool,
) {
    let size = [1280.0, 800.0];
    let poses = [
        ("overview", scene_camera(size, center, radius, None, None)),
        (
            "spawn-yaw-plus-90",
            scene_camera(
                size,
                center,
                radius,
                Some(scene.spawn_observer),
                Some(ObserverLook {
                    yaw: observer_yaw_from_forward(scene.spawn_observer.forward)
                        + std::f32::consts::FRAC_PI_2,
                    pitch: 0.0,
                    last_cursor: None,
                }),
            ),
        ),
    ];
    for (name, camera) in poses {
        let view_projection = camera.projection * camera.view;
        let selection_started = Instant::now();
        let selected_subsectors = scene
            .membership_selection
            .subsector_bounds
            .iter()
            .map(|bounds| {
                bounds.is_none_or(|bounds| {
                    classify_static_draw_frustum_rejection(bounds, view_projection).is_none()
                })
            })
            .collect::<Vec<_>>();
        let draws = scene.opaque_draws.iter().chain(
            include_cutouts
                .then_some(&scene.cutout_draws)
                .into_iter()
                .flatten(),
        );
        let submitted = draws
            .filter(|draw| {
                membership_draw_selected(
                    draw,
                    &selected_subsectors,
                    &scene.membership_selection.linedef_subsectors,
                )
            })
            .count();
        let selection_cpu_us = selection_started.elapsed().as_micros();
        println!(
            "E1M1 AR-0025 membership-union control: pose={name}; source_subsectors={}/{}; submitted_draws={submitted}; candidates={}; selection_cpu_us={selection_cpu_us}; meaning=conservative-source-membership-not-bsp-visibility",
            selected_subsectors.iter().filter(|selected| **selected).count(),
            selected_subsectors.len(),
            scene.opaque_draws.len() + if include_cutouts { scene.cutout_draws.len() } else { 0 },
        );
    }
}
