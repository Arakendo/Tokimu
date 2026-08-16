//! Mechanically extracted AR-0025 evidence subject.

use super::super::super::*;
#[cfg(test)]
use super::comparison_preparation::{
    doom_seg_classic_plane_key, retain_doom_seg_classic_plane_range,
};
use super::screen_projection::{
    merge_solid_range, source_fov_column_interval, source_segment_outside_horizontal_fov,
};
#[cfg(test)]
pub(crate) fn finalize_doom_seg_classic_plane_spans(
    observation: &mut DoomSegClassicPlaneSpanObservation,
) {
    observation.horizontal_spans = 0;
    observation.plane_instances = 0;
    observation.populated_columns = 0;
    observation.populated_cells = 0;
    observation.samples.clear();
    for (key, instances) in &observation.keys {
        let mut key_spans = 0usize;
        let mut key_columns = 0usize;
        let mut key_cells = 0usize;
        for instance in instances {
            observation.plane_instances += 1;
            let mut in_span = false;
            for column in &instance.columns {
                match column {
                    Some([top, bottom]) => {
                        if !in_span {
                            key_spans += 1;
                            in_span = true;
                        }
                        key_columns += 1;
                        key_cells += bottom - top + 1;
                    }
                    None => in_span = false,
                }
            }
        }

        observation.horizontal_spans += key_spans;
        observation.populated_columns += key_columns;
        observation.populated_cells += key_cells;
        if observation.samples.len() < 12 {
            observation.samples.push(format!(
                "kind={:?} height={} flat={} light={} instances={} spans={} columns={} cells={}",
                key.kind,
                key.height,
                key.texture,
                key.light,
                instances.len(),
                key_spans,
                key_columns,
                key_cells,
            ));
        }
    }
}

/// Bounded source-local observation of the clip boundaries that wall tiers
/// evolve after recursive BSP admission. The arrays are diagnostic only: no
/// renderer scissor, candidate selector, flat draw, or visplane consumes them.
#[cfg(test)]
#[allow(dead_code, clippy::too_many_arguments)]
fn legacy_observe_doom_seg_classic_vertical_clip_state(
    map: &DoomMapCore,
    triangles: &[DoomSegTexturedWallTriangle],
    plane_marks: &[DoomSegPlaneMarkObservation],
    traversal: &DoomSegClassicBspObservation,
    viewer: [i16; 2],
    heading: f64,
    eye_height: f64,
) -> DoomSegClassicVerticalClipObservation {
    let half_vertical_fov = classic_presentation_half_vertical_fov();
    let mut result = DoomSegClassicVerticalClipObservation {
        admitted_segs: traversal.admitted_seg_order.len(),
        ..Default::default()
    };
    let mut ceiling_clip = vec![0usize; CLASSIC_PRESENTATION_COLUMNS];
    let mut floor_clip = vec![CLASSIC_PRESENTATION_ROWS; CLASSIC_PRESENTATION_COLUMNS];
    let marks_by_seg = plane_marks
        .iter()
        .map(|mark| (mark.source_seg.record_index, mark))
        .collect::<BTreeMap<_, _>>();
    let segs_by_record = map
        .segs
        .iter()
        .map(|seg| (seg.source.record_index, seg))
        .collect::<BTreeMap<_, _>>();
    let sectors_by_record = map
        .sectors
        .iter()
        .map(|sector| (sector.source.record_index, sector))
        .collect::<BTreeMap<_, _>>();
    let mut tier_heights = BTreeMap::<(u32, u8), (DoomWallTextureRole, f64, f64)>::new();
    for triangle in triangles {
        if !traversal
            .admitted_seg_records
            .contains(&triangle.source_seg.record_index)
        {
            continue;
        }
        let role_key = match triangle.role {
            DoomWallTextureRole::Upper => 0,
            DoomWallTextureRole::Lower => 1,
            DoomWallTextureRole::Middle => 2,
        };
        let minimum = triangle
            .positions
            .iter()
            .map(|position| position[1])
            .fold(f64::INFINITY, f64::min);
        let maximum = triangle
            .positions
            .iter()
            .map(|position| position[1])
            .fold(f64::NEG_INFINITY, f64::max);
        tier_heights
            .entry((triangle.source_seg.record_index, role_key))
            .and_modify(|(_, stored_minimum, stored_maximum)| {
                *stored_minimum = stored_minimum.min(minimum);
                *stored_maximum = stored_maximum.max(maximum);
            })
            .or_insert((triangle.role, minimum, maximum));
    }
    let forward = [heading.cos(), heading.sin()];
    let right = [-forward[1], forward[0]];
    let project = |point: [i16; 2]| {
        let relative = [
            f64::from(point[0] - viewer[0]),
            f64::from(point[1] - viewer[1]),
        ];
        let depth = relative[0] * forward[0] + relative[1] * forward[1];
        let lateral = relative[0] * right[0] + relative[1] * right[1];
        (depth, lateral.atan2(depth))
    };
    let row = |angle: f64| {
        let normalized = (angle.tan() / half_vertical_fov.tan()).clamp(-1.0, 1.0);
        (((1.0 - normalized) * 0.5) * CLASSIC_PRESENTATION_ROWS as f64) as usize
    };
    for source_seg in &traversal.admitted_seg_order {
        let (Some(mark), Some(seg)) =
            (marks_by_seg.get(source_seg), segs_by_record.get(source_seg))
        else {
            continue;
        };
        let front_sector = sectors_by_record
            .get(&mark.front_sector.record_index)
            .expect("validated plane mark names an existing front sector");
        result.floor_plane_marks += usize::from(mark.floor_marked);
        result.ceiling_plane_marks += usize::from(mark.ceiling_marked);
        result.paired_sky_adjustments += usize::from(mark.paired_sky_ceiling_adjustment);
        let start = &map.vertices[usize::from(seg.start_vertex)];
        let end = &map.vertices[usize::from(seg.end_vertex)];
        let (start_depth, start_angle) = project([start.x, start.y]);
        let (end_depth, end_angle) = project([end.x, end.y]);
        if start_depth <= 0.0
            || end_depth <= 0.0
            || (start_angle.abs() > CLASSIC_PRESENTATION_HALF_HORIZONTAL_FOV
                && end_angle.abs() > CLASSIC_PRESENTATION_HALF_HORIZONTAL_FOV)
        {
            continue;
        }
        let [left, right_column] = source_fov_column_interval(
            start_angle,
            end_angle,
            CLASSIC_PRESENTATION_HALF_HORIZONTAL_FOV,
            CLASSIC_PRESENTATION_COLUMNS,
        );
        let has_upper = tier_heights.contains_key(&(*source_seg, 0));
        let has_lower = tier_heights.contains_key(&(*source_seg, 1));
        let has_middle = tier_heights.contains_key(&(*source_seg, 2));

        // Classic plane marking consumes the clip state that exists before
        // this wall range mutates it. Retain only bounded source-keyed cells;
        // later presentation lowering must remain a separate experiment.
        let mut ceiling_plane_writes = Vec::new();
        let mut floor_plane_writes = Vec::new();
        for x in left..=right_column {
            let normalized = -1.0 + ((x as f64 + 0.5) / CLASSIC_PRESENTATION_COLUMNS as f64) * 2.0;
            let local_angle = (normalized * CLASSIC_PRESENTATION_HALF_HORIZONTAL_FOV.tan()).atan();
            let ray = [
                forward[0] * local_angle.cos() + right[0] * local_angle.sin(),
                forward[1] * local_angle.cos() + right[1] * local_angle.sin(),
            ];
            let depth = source_ray_segment_depth(viewer, ray, [start.x, start.y], [end.x, end.y])
                .unwrap_or((start_depth + end_depth) * 0.5);
            let ceiling = row((f64::from(front_sector.ceiling_height) - eye_height).atan2(depth))
                .min(CLASSIC_PRESENTATION_ROWS - 1);
            let floor = row((f64::from(front_sector.floor_height) - eye_height).atan2(depth))
                .min(CLASSIC_PRESENTATION_ROWS - 1);

            let (ceiling, floor) = (ceiling.min(floor), ceiling.max(floor));
            if mark.ceiling_marked {
                let top = ceiling_clip[x].saturating_add(1);
                let bottom = ceiling.saturating_sub(1);
                ceiling_plane_writes.push((x, top, bottom));
            }
            if mark.floor_marked {
                let top = floor.saturating_add(1);
                let bottom = floor_clip[x].saturating_sub(1);
                floor_plane_writes.push((x, top, bottom));
            }
        }
        if !ceiling_plane_writes.is_empty() {
            retain_doom_seg_classic_plane_range(
                &mut result.plane_spans,
                doom_seg_classic_plane_key(DoomSegClassicPlaneKind::Ceiling, front_sector),
                mark.front_sector.record_index,
                *source_seg,
                &ceiling_plane_writes,
                CLASSIC_PRESENTATION_COLUMNS,
            );
        }
        if !floor_plane_writes.is_empty() {
            retain_doom_seg_classic_plane_range(
                &mut result.plane_spans,
                doom_seg_classic_plane_key(DoomSegClassicPlaneKind::Floor, front_sector),
                mark.front_sector.record_index,
                *source_seg,
                &floor_plane_writes,
                CLASSIC_PRESENTATION_COLUMNS,
            );
        }
        for role_key in 0..=2 {
            let Some((role, minimum, maximum)) = tier_heights.get(&(*source_seg, role_key)) else {
                continue;
            };
            match role {
                DoomWallTextureRole::Upper => result.upper_tier_spans += 1,
                DoomWallTextureRole::Lower => result.lower_tier_spans += 1,
                DoomWallTextureRole::Middle => result.middle_tier_spans += 1,
            }
            let mut center_trace = None;
            for x in left..=right_column {
                let normalized =
                    -1.0 + ((x as f64 + 0.5) / CLASSIC_PRESENTATION_COLUMNS as f64) * 2.0;
                let local_angle =
                    (normalized * CLASSIC_PRESENTATION_HALF_HORIZONTAL_FOV.tan()).atan();
                let ray = [
                    forward[0] * local_angle.cos() + right[0] * local_angle.sin(),
                    forward[1] * local_angle.cos() + right[1] * local_angle.sin(),
                ];
                let depth =
                    source_ray_segment_depth(viewer, ray, [start.x, start.y], [end.x, end.y])
                        .unwrap_or((start_depth + end_depth) * 0.5);
                let top =
                    row((maximum - eye_height).atan2(depth)).min(CLASSIC_PRESENTATION_ROWS - 1);
                let bottom =
                    row((minimum - eye_height).atan2(depth)).min(CLASSIC_PRESENTATION_ROWS - 1);
                let (top, bottom) = (top.min(bottom), top.max(bottom));
                let prior = [ceiling_clip[x], floor_clip[x]];
                match role {
                    DoomWallTextureRole::Upper => {
                        let next = ceiling_clip[x].max(bottom.saturating_add(1));
                        result.ceiling_clip_updates += usize::from(next != ceiling_clip[x]);
                        ceiling_clip[x] = next;
                    }
                    DoomWallTextureRole::Lower => {
                        let next = floor_clip[x].min(top);
                        result.floor_clip_updates += usize::from(next != floor_clip[x]);
                        floor_clip[x] = next;
                    }
                    DoomWallTextureRole::Middle => {}
                }
                if x == CLASSIC_PRESENTATION_COLUMNS / 2 {
                    center_trace = Some(format!(
                        "seg={source_seg} line={} tier={role:?} rows={top}..{bottom} clip-before={}..{} clip-after={}..{}",
                        seg.linedef, prior[0], prior[1], ceiling_clip[x], floor_clip[x],
                    ));
                }
            }
            if let Some(sample) = center_trace {
                if result.samples.len() < 12 {
                    result.samples.push(sample);
                }
            }
        }
        // The original wall loop also moves a clip boundary for a marked plane
        // when there is no corresponding upper/lower texture tier. A one-sided
        // middle is terminal, while a two-sided masked middle remains open.
        for x in left..=right_column {
            let normalized = -1.0 + ((x as f64 + 0.5) / CLASSIC_PRESENTATION_COLUMNS as f64) * 2.0;
            let local_angle = (normalized * CLASSIC_PRESENTATION_HALF_HORIZONTAL_FOV.tan()).atan();
            let ray = [
                forward[0] * local_angle.cos() + right[0] * local_angle.sin(),
                forward[1] * local_angle.cos() + right[1] * local_angle.sin(),
            ];
            let depth = source_ray_segment_depth(viewer, ray, [start.x, start.y], [end.x, end.y])
                .unwrap_or((start_depth + end_depth) * 0.5);
            let ceiling = row((f64::from(front_sector.ceiling_height) - eye_height).atan2(depth))
                .min(CLASSIC_PRESENTATION_ROWS - 1);
            let floor = row((f64::from(front_sector.floor_height) - eye_height).atan2(depth))
                .min(CLASSIC_PRESENTATION_ROWS - 1);
            let (ceiling, floor) = (ceiling.min(floor), ceiling.max(floor));
            if has_middle && mark.back_sector.is_none() {
                result.ceiling_clip_updates +=
                    usize::from(ceiling_clip[x] != CLASSIC_PRESENTATION_ROWS);
                result.floor_clip_updates += usize::from(floor_clip[x] != 0);
                ceiling_clip[x] = CLASSIC_PRESENTATION_ROWS;
                floor_clip[x] = 0;
            } else {
                if !has_upper && mark.ceiling_marked {
                    let next = ceiling_clip[x].max(ceiling.saturating_sub(1));
                    result.ceiling_clip_updates += usize::from(next != ceiling_clip[x]);
                    ceiling_clip[x] = next;
                }
                if !has_lower && mark.floor_marked {
                    let next = floor_clip[x].min(floor.saturating_add(1));
                    result.floor_clip_updates += usize::from(next != floor_clip[x]);
                    floor_clip[x] = next;
                }
            }
        }
    }
    finalize_doom_seg_classic_plane_spans(&mut result.plane_spans);
    result
}

/// Inventories the already lowerable wall tiers selected by the headless
/// source protocol. The roles remain Doom provider evidence; they are not a
/// renderer material taxonomy or a claim that all source wall tiers have been
/// classically clipped.
pub(crate) fn summarize_classic_bsp_wall_triangle_roles(
    triangles: &[DoomSegTexturedWallTriangle],
    admitted_seg_records: &BTreeSet<u32>,
) -> (usize, usize, usize) {
    triangles
        .iter()
        .fold((0, 0, 0), |(upper, lower, middle), triangle| {
            if !admitted_seg_records.contains(&triangle.source_seg.record_index) {
                return (upper, lower, middle);
            }
            match triangle.role {
                DoomWallTextureRole::Upper => (upper + 1, lower, middle),
                DoomWallTextureRole::Lower => (upper, lower + 1, middle),
                DoomWallTextureRole::Middle => (upper, lower, middle + 1),
            }
        })
}

/// Counts the source `R_StoreWallRange` plane-mark facts for admitted SEG
/// records. A mark is not a projected visplane span or a selected flat draw.
pub(crate) fn summarize_classic_bsp_plane_marks(
    plane_marks: &[DoomSegPlaneMarkObservation],
    admitted_seg_records: &BTreeSet<u32>,
) -> (usize, usize, usize) {
    plane_marks
        .iter()
        .fold((0, 0, 0), |(floors, ceilings, paired_sky), observation| {
            if !admitted_seg_records.contains(&observation.source_seg.record_index) {
                return (floors, ceilings, paired_sky);
            }
            (
                floors + usize::from(observation.floor_marked),
                ceilings + usize::from(observation.ceiling_marked),
                paired_sky + usize::from(observation.paired_sky_ceiling_adjustment),
            )
        })
}

/// Counts existing source-labelled static flat draws whose owning subsector was
/// reached by the headless Doom BSP protocol. These are not classic-Doom plane
/// spans and must not be submitted as a visibility result; the count merely
/// makes the currently unmodeled plane portion explicit.
pub(crate) fn count_classic_bsp_static_flat_draws(
    scene: &SceneInput,
    observation: &DoomSegClassicBspObservation,
) -> (usize, usize) {
    scene
        .opaque_draws
        .iter()
        .fold((0, 0), |(floors, ceilings), draw| match draw.source {
            StaticDrawSource::Flat {
                source_subsector,
                plane: doom_geometry_provider::DoomSurfacePlane::Floor,
                ..
            } if observation
                .visited_subsectors
                .contains(&(source_subsector.record_index as u16)) =>
            {
                (floors + 1, ceilings)
            }
            StaticDrawSource::Flat {
                source_subsector,
                plane: doom_geometry_provider::DoomSurfacePlane::Ceiling,
                ..
            } if observation
                .visited_subsectors
                .contains(&(source_subsector.record_index as u16)) =>
            {
                (floors, ceilings + 1)
            }
            _ => (floors, ceilings),
        })
}

pub(crate) fn observe_doom_seg_classic_bsp(
    map: &DoomMapCore,
    viewer: [i16; 2],
    heading: f64,
    watched_subsectors: &BTreeSet<u16>,
) -> PlatformResult<DoomSegClassicBspObservation> {
    Ok(observe_doom_classic_bsp(
        map,
        viewer,
        heading,
        watched_subsectors,
    )?)
}

#[cfg(test)]
#[allow(dead_code)]
#[allow(clippy::too_many_arguments)]
fn visit_doom_seg_classic_bsp_child(
    map: &DoomMapCore,
    child: DoomBspChild,
    viewer: [i16; 2],
    heading: f64,
    occluders: &BTreeMap<u32, doom_geometry_provider::DoomSegOccluderObservation>,
    solid_ranges: &mut Vec<[usize; 2]>,
    ancestors: &mut Vec<u16>,
    watched_subsectors: &BTreeSet<u16>,
    observation: &mut DoomSegClassicBspObservation,
) -> PlatformResult<()> {
    match child {
        DoomBspChild::Subsector(index) => {
            let subsector = map.subsectors.get(usize::from(index)).ok_or_else(|| {
                io::Error::other(format!(
                    "Stage 3B classic BSP subsector {index} is unavailable"
                ))
            })?;
            observation.leaves_visited += 1;
            observation.visited_subsectors.insert(index);
            let first = usize::from(subsector.first_seg);
            let end = first + usize::from(subsector.seg_count);
            for seg in &map.segs[first..end] {
                admit_doom_seg_classic(
                    map,
                    seg,
                    viewer,
                    heading,
                    occluders,
                    solid_ranges,
                    observation,
                );
            }
            Ok(())
        }
        DoomBspChild::Node(index) => {
            if ancestors.contains(&index) {
                return Err(io::Error::other(format!(
                    "Stage 3B classic BSP cycle at node {index}"
                ))
                .into());
            }
            let node = map.nodes.get(usize::from(index)).ok_or_else(|| {
                io::Error::other(format!("Stage 3B classic BSP node {index} is unavailable"))
            })?;
            ancestors.push(index);
            let side = i64::from(node.delta_x) * i64::from(viewer[1] - node.y)
                - i64::from(node.delta_y) * i64::from(viewer[0] - node.x);
            let (near, far, far_bbox) = if side < 0 {
                (node.right_child, node.left_child, node.left_bbox)
            } else {
                (node.left_child, node.right_child, node.right_bbox)
            };
            visit_doom_seg_classic_bsp_child(
                map,
                near,
                viewer,
                heading,
                occluders,
                solid_ranges,
                ancestors,
                watched_subsectors,
                observation,
            )?;
            let watched_far = watched_subsectors
                .iter()
                .filter_map(|target| {
                    doom_bsp_child_contains_subsector(map, far, *target).then_some(*target)
                })
                .collect::<Vec<_>>();
            let far_projection = source_bbox_fov_column_interval(
                viewer,
                heading,
                far_bbox,
                std::f64::consts::FRAC_PI_4,
                320,
            );
            match far_projection {
                SourceBBoxProjection::OutsideFov => {
                    observation.far_children_outside_fov += 1;
                    record_watched_subsector_elision(
                        observation,
                        index,
                        "outside-fov",
                        &watched_far,
                        None,
                        None,
                    );
                }
                SourceBBoxProjection::Interval(interval) => {
                    if let Some(covering_range) = solid_ranges
                        .iter()
                        .find(|[first, last]| *first <= interval[0] && interval[1] <= *last)
                    {
                        observation.far_children_pruned += 1;
                        record_watched_subsector_elision(
                            observation,
                            index,
                            "solid-range",
                            &watched_far,
                            Some(interval),
                            Some(*covering_range),
                        );
                    } else {
                        visit_doom_seg_classic_bsp_child(
                            map,
                            far,
                            viewer,
                            heading,
                            occluders,
                            solid_ranges,
                            ancestors,
                            watched_subsectors,
                            observation,
                        )?;
                    }
                }
                SourceBBoxProjection::Uncertain => {
                    if matches!(far_projection, SourceBBoxProjection::Uncertain) {
                        observation.far_children_fail_open += 1;
                    }
                    visit_doom_seg_classic_bsp_child(
                        map,
                        far,
                        viewer,
                        heading,
                        occluders,
                        solid_ranges,
                        ancestors,
                        watched_subsectors,
                        observation,
                    )?;
                }
            }
            ancestors.pop();
            Ok(())
        }
    }
}

#[cfg(test)]
fn record_watched_subsector_elision(
    observation: &mut DoomSegClassicBspObservation,
    node: u16,
    reason: &str,
    subsectors: &[u16],
    interval: Option<[usize; 2]>,
    covering_range: Option<[usize; 2]>,
) {
    if !subsectors.is_empty() {
        observation.watched_subsector_elisions.push(format!(
            "node={node}:reason={reason}:subsectors={subsectors:?}:interval={interval:?}:covering-range={covering_range:?}"
        ));
    }
}

#[cfg(test)]
fn doom_bsp_child_contains_subsector(map: &DoomMapCore, child: DoomBspChild, target: u16) -> bool {
    let mut visited_nodes = HashSet::new();
    doom_bsp_child_contains_subsector_inner(map, child, target, &mut visited_nodes)
}

#[cfg(test)]
fn doom_bsp_child_contains_subsector_inner(
    map: &DoomMapCore,
    child: DoomBspChild,
    target: u16,
    visited_nodes: &mut HashSet<u16>,
) -> bool {
    match child {
        DoomBspChild::Subsector(index) => index == target,
        DoomBspChild::Node(index) => {
            if !visited_nodes.insert(index) {
                return false;
            }
            let contains = map.nodes.get(usize::from(index)).is_some_and(|node| {
                doom_bsp_child_contains_subsector_inner(
                    map,
                    node.right_child,
                    target,
                    visited_nodes,
                ) || doom_bsp_child_contains_subsector_inner(
                    map,
                    node.left_child,
                    target,
                    visited_nodes,
                )
            });
            visited_nodes.remove(&index);
            contains
        }
    }
}

#[cfg(test)]
fn admit_doom_seg_classic(
    map: &DoomMapCore,
    seg: &doom_map_provider::DoomSeg,
    viewer: [i16; 2],
    heading: f64,
    occluders: &BTreeMap<u32, doom_geometry_provider::DoomSegOccluderObservation>,
    solid_ranges: &mut Vec<[usize; 2]>,
    observation: &mut DoomSegClassicBspObservation,
) {
    const HALF_FOV: f64 = std::f64::consts::FRAC_PI_4;
    observation.source_segs_visited += 1;
    if seg.linedef == 247 {
        observation.hut_linedef_segs_visited += 1;
    }
    let start = &map.vertices[usize::from(seg.start_vertex)];
    let end = &map.vertices[usize::from(seg.end_vertex)];
    match source_seg_facing(viewer, [start.x, start.y], [end.x, end.y]) {
        SourceSegFacing::Back => {
            observation.backface_rejected += 1;
            return;
        }
        SourceSegFacing::EdgeOn => {
            observation.edge_on += 1;
            return;
        }
        SourceSegFacing::Front => {}
    }
    let forward = [heading.cos(), heading.sin()];
    let right = [-forward[1], forward[0]];
    let project = |point: [i16; 2]| {
        let relative = [
            f64::from(point[0] - viewer[0]),
            f64::from(point[1] - viewer[1]),
        ];
        let depth = relative[0] * forward[0] + relative[1] * forward[1];
        let lateral = relative[0] * right[0] + relative[1] * right[1];
        (depth, lateral.atan2(depth))
    };
    let (start_depth, start_angle) = project([start.x, start.y]);
    let (end_depth, end_angle) = project([end.x, end.y]);
    if (start_depth <= 0.0 && end_depth <= 0.0)
        || source_segment_outside_horizontal_fov(start_angle, end_angle, HALF_FOV)
    {
        observation.outside_fov_rejected += 1;
        return;
    }
    let authority = occluders
        .get(&seg.source.record_index)
        .expect("every source SEG is classified");
    let solid = authority.kind != doom_geometry_provider::DoomSegOccluderKind::Open;
    observation
        .admitted_seg_records
        .insert(seg.source.record_index);
    observation.admitted_seg_order.push(seg.source.record_index);
    if seg.linedef == 247 {
        observation.hut_linedef_segs_admitted += 1;
    }
    if solid && start_depth > 0.0 && end_depth > 0.0 {
        observation.solid_admitted += 1;
        let interval = source_fov_column_interval(start_angle, end_angle, HALF_FOV, 320);
        if merge_solid_range(solid_ranges, interval) {
            observation.solid_range_fully_covered += 1;
        } else {
            observation.solid_range_contributors += 1;
        }
    } else if solid {
        // A wall crossing the viewer plane must remain present, but its
        // unclipped behind-view endpoint cannot safely close a screen range.
        observation.near_plane_fail_open += 1;
    } else {
        observation.pass_admitted += 1;
    }
    if observation.samples.len() < 8 {
        observation.samples.push(format!(
            "seg={} line={} kind={:?} admission={}",
            seg.source.record_index,
            seg.linedef,
            authority.kind,
            if solid { "solid" } else { "pass" },
        ));
    }
}

pub(crate) fn observe_doom_seg_classic_admission(
    map: &DoomMapCore,
    viewer: [i16; 2],
    heading: f64,
) -> PlatformResult<DoomSegClassicAdmissionObservation> {
    const HALF_FOV: f64 = std::f64::consts::FRAC_PI_4;
    let forward = [heading.cos(), heading.sin()];
    let right = [-forward[1], forward[0]];
    let occluders = observe_doom_seg_occluders(map)?
        .into_iter()
        .map(|observation| (observation.source_seg.record_index, observation))
        .collect::<BTreeMap<_, _>>();
    let mut result = DoomSegClassicAdmissionObservation {
        source_segs: map.segs.len(),
        ..Default::default()
    };
    let order = resolve_doom_viewer_subsector_order(map, viewer)?;
    let order_by_source = order
        .iter()
        .enumerate()
        .map(|(rank, source)| (source.record_index, rank))
        .collect::<BTreeMap<_, _>>();
    let mut ordered_segs = map
        .segs
        .iter()
        .filter_map(|seg| {
            let subsector = map.subsectors.iter().find(|subsector| {
                let first = usize::from(subsector.first_seg);
                let end = first + usize::from(subsector.seg_count);
                (first..end).any(|index| map.segs[index].source == seg.source)
            })?;
            Some((*order_by_source.get(&subsector.source.record_index)?, seg))
        })
        .collect::<Vec<_>>();
    ordered_segs.sort_by_key(|(rank, seg)| (*rank, seg.source.record_index));
    let mut solid_ranges = Vec::<[usize; 2]>::new();
    for (rank, seg) in ordered_segs {
        let start = &map.vertices[usize::from(seg.start_vertex)];

        let end = &map.vertices[usize::from(seg.end_vertex)];
        let facing = source_seg_facing(viewer, [start.x, start.y], [end.x, end.y]);
        match facing {
            SourceSegFacing::Back => {
                result.backface_rejected += 1;
                continue;
            }
            SourceSegFacing::EdgeOn => {
                result.edge_on += 1;
                continue;
            }
            SourceSegFacing::Front => {}
        }
        let project_angle = |point: [i16; 2]| {
            let relative = [
                f64::from(point[0] - viewer[0]),
                f64::from(point[1] - viewer[1]),
            ];
            let depth = relative[0] * forward[0] + relative[1] * forward[1];
            let lateral = relative[0] * right[0] + relative[1] * right[1];
            (depth, lateral.atan2(depth))
        };
        let (start_depth, start_angle) = project_angle([start.x, start.y]);
        let (end_depth, end_angle) = project_angle([end.x, end.y]);

        if (start_depth <= 0.0 && end_depth <= 0.0)
            || source_segment_outside_horizontal_fov(start_angle, end_angle, HALF_FOV)
        {
            result.outside_fov_rejected += 1;
            continue;
        }
        let authority = occluders
            .get(&seg.source.record_index)
            .expect("every source SEG is classified");
        let solid = authority.kind != doom_geometry_provider::DoomSegOccluderKind::Open;
        if solid && start_depth > 0.0 && end_depth > 0.0 {
            result.solid_admitted += 1;
            let interval = source_fov_column_interval(start_angle, end_angle, HALF_FOV, 320);
            if merge_solid_range(&mut solid_ranges, interval) {
                result.solid_range_fully_covered += 1;
            } else {
                result.solid_range_contributors += 1;
            }
        } else if solid {
            result.near_plane_fail_open += 1;
        } else {
            result.pass_admitted += 1;
        }
        if result.samples.len() < 8 {
            result.samples.push(format!(
                "seg={} line={} rank={rank} facing=front start-depth={start_depth:.1} end-depth={end_depth:.1} kind={:?} admission={}",
                seg.source.record_index,
                seg.linedef,
                authority.kind,
                if solid { "solid" } else { "pass" },
            ));
        }
    }

    result.solid_range_covered_columns = solid_ranges
        .iter()
        .map(|[first, last]| last - first + 1)
        .sum();
    Ok(result)
}
