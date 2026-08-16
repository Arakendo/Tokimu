//! Mechanically extracted AR-0025 evidence subject.

use super::super::super::*;
pub(crate) fn source_fov_column_interval(
    first_angle: f64,
    second_angle: f64,
    half_fov: f64,
    columns: usize,
) -> [usize; 2] {
    let column = |angle: f64| {
        let normalized = angle.clamp(-half_fov, half_fov).tan() / half_fov.tan();
        (((normalized + 1.0) * 0.5) * columns as f64) as usize
    };
    let first = column(first_angle).min(columns - 1);
    let second = column(second_angle).min(columns - 1);
    [first.min(second), first.max(second)]
}

/// A segment is horizontally outside only when both endpoint bearings lie on
/// the same exterior side. Opposite exterior bearings cross the view and must
/// not be rejected merely because each endpoint is individually outside.
pub(crate) fn source_segment_outside_horizontal_fov(
    first_angle: f64,
    second_angle: f64,
    half_fov: f64,
) -> bool {
    (first_angle > half_fov && second_angle > half_fov)
        || (first_angle < -half_fov && second_angle < -half_fov)
}

/// Source-only far-child bbox outcome for the Stage 3B `R_CheckBBox` control.
/// It distinguishes a definitely outside FOV from geometry whose projection is
/// ambiguous and must remain fail-open.
#[cfg(test)]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum SourceBBoxProjection {
    OutsideFov,
    Interval([usize; 2]),
    Uncertain,
}

/// Projects the two source bbox silhouette corners selected by the classic
/// `R_CheckBBox` position table. This is Doom-only protocol work, not a
/// generic bounding-box culler.
#[cfg(test)]
pub(crate) fn source_bbox_fov_column_interval(
    viewer: [i16; 2],
    heading: f64,
    bbox: [i16; 4],
    half_fov: f64,
    columns: usize,
) -> SourceBBoxProjection {
    let [top, bottom, left, right] = bbox;
    let box_x = if viewer[0] <= left {
        0
    } else if viewer[0] < right {
        1
    } else {
        2
    };
    let box_y = if viewer[1] >= top {
        0
    } else if viewer[1] > bottom {
        1
    } else {
        2
    };
    let box_position = box_y * 4 + box_x;
    if box_position == 5 {
        return SourceBBoxProjection::Uncertain;
    }
    // Matches `r_bsp.c`'s `checkcoord`: each value indexes Doom's decoded
    // bbox layout [top, bottom, left, right].
    const CHECK_COORD: [[usize; 4]; 12] = [
        [3, 0, 2, 1],
        [3, 0, 2, 0],
        [3, 1, 2, 0],
        [0, 0, 0, 0],
        [2, 0, 2, 1],
        [0, 0, 0, 0],
        [3, 1, 3, 0],
        [0, 0, 0, 0],
        [2, 0, 3, 1],
        [2, 1, 3, 1],
        [2, 1, 3, 0],
        [0, 0, 0, 0],
    ];
    let coordinates = CHECK_COORD[box_position];
    let source = [top, bottom, left, right];
    let points = [
        [source[coordinates[0]], source[coordinates[1]]],
        [source[coordinates[2]], source[coordinates[3]]],
    ];
    let forward = [heading.cos(), heading.sin()];
    let view_right = [-forward[1], forward[0]];
    let mut angles = Vec::with_capacity(2);
    for point in points {
        let relative = [
            f64::from(point[0] - viewer[0]),
            f64::from(point[1] - viewer[1]),
        ];
        let depth = relative[0] * forward[0] + relative[1] * forward[1];
        if depth <= 0.0 {
            return SourceBBoxProjection::Uncertain;
        }
        let lateral = relative[0] * view_right[0] + relative[1] * view_right[1];
        angles.push(lateral.atan2(depth));
    }
    let first_angle = angles[0];
    let second_angle = angles[1];
    let span = (first_angle - second_angle).abs();
    if span >= std::f64::consts::PI {
        return SourceBBoxProjection::Uncertain;
    }

    let minimum = first_angle.min(second_angle);
    let maximum = first_angle.max(second_angle);
    if maximum < -half_fov || minimum > half_fov {
        SourceBBoxProjection::OutsideFov
    } else {
        SourceBBoxProjection::Interval(source_fov_column_interval(
            minimum, maximum, half_fov, columns,
        ))
    }
}

/// Inserts one inclusive source screen interval into the current horizontal
/// solid-range union. Returns true when the interval was already fully closed.
/// This mirrors only the union property of Doom `solidsegs`, not its sentinel
/// representation, clipping details, or BSP bbox policy.
pub(crate) fn merge_solid_range(ranges: &mut Vec<[usize; 2]>, interval: [usize; 2]) -> bool {
    let fully_covered = ranges
        .iter()
        .any(|[first, last]| *first <= interval[0] && interval[1] <= *last);
    let mut merged = interval;
    let mut index = 0;
    while index < ranges.len() {
        let [first, last] = ranges[index];

        if last.saturating_add(1) < merged[0] || merged[1].saturating_add(1) < first {
            index += 1;
            continue;
        }
        merged[0] = merged[0].min(first);
        merged[1] = merged[1].max(last);
        ranges.remove(index);
    }
    ranges.insert(index, merged);
    fully_covered
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum SourceSegFacing {
    Front,
    Back,
    EdgeOn,
}

/// Classic Doom treats the directed SEG's right side as its visible front.
/// This is source-data interpretation, intentionally separate from Tokimu
/// mesh normals, camera controls, or renderer culling.
pub(crate) fn source_seg_facing(
    viewer: [i16; 2],
    start: [i16; 2],
    end: [i16; 2],
) -> SourceSegFacing {
    let segment = [i64::from(end[0] - start[0]), i64::from(end[1] - start[1])];
    let to_viewer = [
        i64::from(viewer[0] - start[0]),
        i64::from(viewer[1] - start[1]),
    ];
    let side = segment[0] * to_viewer[1] - segment[1] * to_viewer[0];
    if side < 0 {
        SourceSegFacing::Front
    } else if side > 0 {
        SourceSegFacing::Back
    } else {
        SourceSegFacing::EdgeOn
    }
}

pub(crate) fn observe_doom_seg_screen_grid(
    map: &DoomMapCore,
    eye_height: f32,
    per_column: bool,
    viewer: [i16; 2],
    angle: f64,
) -> PlatformResult<DoomSegScreenGridObservation> {
    observe_doom_seg_screen_grid_with_order(
        map,
        eye_height,
        per_column,
        viewer,
        angle,
        DoomSegScreenGridOrder::BspLeafThenSource,
    )
}

/// Runs the same bounded source grid with one declared diagnostic ordering.
/// The alternate nearest-segment order exists only to test whether coarse BSP
/// leaf order explains the retained depth inversions; it is not Doom parity.
pub(crate) fn observe_doom_seg_screen_grid_with_order(
    map: &DoomMapCore,
    eye_height: f32,
    per_column: bool,
    viewer: [i16; 2],
    angle: f64,
    ordering: DoomSegScreenGridOrder,
) -> PlatformResult<DoomSegScreenGridObservation> {
    const COLUMNS: usize = 320;
    const ROWS: usize = 200;
    const HALF_HORIZONTAL_FOV: f64 = std::f64::consts::FRAC_PI_4;
    let half_vertical_fov = ((ROWS as f64 / COLUMNS as f64) * HALF_HORIZONTAL_FOV.tan()).atan();
    let eye_height = f64::from(eye_height);
    let forward = [angle.cos(), angle.sin()];
    let right = [-forward[1], forward[0]];
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
            let subsector = map.subsectors.iter().position(|subsector| {
                let start = usize::from(subsector.first_seg);
                let end = start + usize::from(subsector.seg_count);
                (start..end).any(|index| map.segs[index].source == seg.source)
            })?;
            Some((
                order_by_source.get(&map.subsectors[subsector].source.record_index)?,
                seg,
            ))
        })
        .collect::<Vec<_>>();
    let candidates = resolve_doom_wall_candidates(map)?
        .into_iter()
        .map(|candidate| (candidate.source_linedef.record_index, candidate))
        .collect::<BTreeMap<_, _>>();
    let occluders = observe_doom_seg_occluders(map)?
        .into_iter()
        .map(|observation| (observation.source_seg.record_index, observation))
        .collect::<BTreeMap<_, _>>();
    let project = |point: [i16; 2]| {
        let relative = [
            f64::from(point[0] - viewer[0]),
            f64::from(point[1] - viewer[1]),
        ];
        let depth = relative[0] * forward[0] + relative[1] * forward[1];
        let lateral = relative[0] * right[0] + relative[1] * right[1];
        (depth, lateral.atan2(depth))
    };
    match ordering {
        DoomSegScreenGridOrder::BspLeafThenSource => {
            ordered_segs.sort_by_key(|(rank, seg)| (**rank, seg.source.record_index));
        }
        DoomSegScreenGridOrder::NearestSegmentToViewer => {
            ordered_segs.sort_by(|(_, left), (_, right)| {
                let left_start = &map.vertices[usize::from(left.start_vertex)];
                let left_end = &map.vertices[usize::from(left.end_vertex)];
                let right_start = &map.vertices[usize::from(right.start_vertex)];
                let right_end = &map.vertices[usize::from(right.end_vertex)];
                source_point_segment_distance_squared(
                    viewer,
                    [left_start.x, left_start.y],
                    [left_end.x, left_end.y],
                )
                .total_cmp(&source_point_segment_distance_squared(
                    viewer,
                    [right_start.x, right_start.y],
                    [right_end.x, right_end.y],
                ))
                .then_with(|| left.source.record_index.cmp(&right.source.record_index))
            });
        }
    }
    let column = |angle: f64| {
        ((angle.clamp(-HALF_HORIZONTAL_FOV, HALF_HORIZONTAL_FOV) + HALF_HORIZONTAL_FOV)
            / (2.0 * HALF_HORIZONTAL_FOV)
            * COLUMNS as f64) as usize
    };
    let row = |angle: f64| {
        ((half_vertical_fov - angle.clamp(-half_vertical_fov, half_vertical_fov))
            / (2.0 * half_vertical_fov)
            * ROWS as f64) as usize
    };

    let mut covered = vec![false; COLUMNS * ROWS];
    // This stays beside, rather than inside, the boolean coverage state so the
    // established falsified control retains its exact selection behavior.
    // It merely exposes cases where leaf/source order disagrees with local
    // ray depth for an attempted occluding write.
    let mut covering_depths = vec![None::<(f64, u32)>; COLUMNS * ROWS];
    let mut depth_order_inversions = 0usize;
    let mut depth_order_samples = Vec::new();
    let mut outside = 0usize;
    let mut fully_covered = 0usize;

    let mut partial = 0usize;
    let mut fully_visible = 0usize;
    let mut contributors = 0usize;
    let mut samples = Vec::new();
    let mut selected_seg_records = BTreeSet::new();
    for (rank, seg) in ordered_segs {
        let start = &map.vertices[usize::from(seg.start_vertex)];
        let end = &map.vertices[usize::from(seg.end_vertex)];
        let (start_depth, start_angle) = project([start.x, start.y]);
        let (end_depth, end_angle) = project([end.x, end.y]);
        if start_depth <= 0.0
            || end_depth <= 0.0
            || (start_angle.abs() > HALF_HORIZONTAL_FOV && end_angle.abs() > HALF_HORIZONTAL_FOV)
        {
            outside += 1;
            continue;
        }
        let candidate = candidates
            .get(&map.linedefs[usize::from(seg.linedef)].source.record_index)
            .expect("every SEG linedef has a resolved source wall");
        let (front, back) = match seg.direction {
            0 => (candidate.right.as_ref(), candidate.left.as_ref()),

            1 => (candidate.left.as_ref(), candidate.right.as_ref()),
            direction => {
                return Err(io::Error::other(format!(
                    "Stage 3B source SEG {} has unsupported direction {direction}",
                    seg.source.record_index
                ))
                .into())
            }
        };
        let front = front.expect("SEG direction names an existing owning side");
        let mut floor = map.sectors[usize::from(front.sector_index)].floor_height;
        let mut ceiling = map.sectors[usize::from(front.sector_index)].ceiling_height;
        if let Some(back) = back {
            let back_sector = &map.sectors[usize::from(back.sector_index)];
            floor = floor.min(back_sector.floor_height);
            ceiling = ceiling.max(back_sector.ceiling_height);
        }
        let left = column(start_angle).min(COLUMNS - 1);
        let right_column = column(end_angle).min(COLUMNS - 1);
        let (left, right_column) = (left.min(right_column), left.max(right_column));
        let rectangle_span = || {
            let vertical_angles = [
                (f64::from(floor) - eye_height).atan2(start_depth),
                (f64::from(ceiling) - eye_height).atan2(start_depth),
                (f64::from(floor) - eye_height).atan2(end_depth),
                (f64::from(ceiling) - eye_height).atan2(end_depth),
            ];
            let top = row(vertical_angles
                .iter()
                .copied()
                .fold(f64::NEG_INFINITY, f64::max))
            .min(ROWS - 1);
            let bottom = row(vertical_angles
                .iter()
                .copied()
                .fold(f64::INFINITY, f64::min))
            .min(ROWS - 1);
            [top.min(bottom), top.max(bottom)]
        };
        let vertical_spans = if per_column {
            (left..=right_column)
                .map(|x| {
                    let local_angle = -HALF_HORIZONTAL_FOV
                        + ((x as f64 + 0.5) / COLUMNS as f64) * (2.0 * HALF_HORIZONTAL_FOV);
                    let ray = [
                        forward[0] * local_angle.cos() + right[0] * local_angle.sin(),
                        forward[1] * local_angle.cos() + right[1] * local_angle.sin(),
                    ];
                    let depth =
                        source_ray_segment_depth(viewer, ray, [start.x, start.y], [end.x, end.y])
                            .unwrap_or_else(|| {
                                let fraction = if right_column == left {
                                    0.5
                                } else {
                                    (x - left) as f64 / (right_column - left) as f64
                                };
                                start_depth + (end_depth - start_depth) * fraction
                            });
                    let top = row((f64::from(ceiling) - eye_height).atan2(depth)).min(ROWS - 1);
                    let bottom = row((f64::from(floor) - eye_height).atan2(depth)).min(ROWS - 1);
                    ([top.min(bottom), top.max(bottom)], depth)
                })
                .collect::<Vec<_>>()
        } else {
            let depth = (start_depth + end_depth) * 0.5;
            vec![(rectangle_span(), depth); right_column - left + 1]
        };
        let mut cells = 0usize;
        let mut visible_cells = 0usize;
        for (offset, ([top, bottom], _depth)) in vertical_spans.iter().copied().enumerate() {
            let x = left + offset;
            for y in top..=bottom {
                cells += 1;
                visible_cells += usize::from(!covered[y * COLUMNS + x]);
            }
        }
        let result = if visible_cells == 0 {
            fully_covered += 1;
            "covered"
        } else if visible_cells == cells {
            fully_visible += 1;
            "visible"
        } else {
            partial += 1;
            "partial"
        };
        if visible_cells > 0 {
            selected_seg_records.insert(seg.source.record_index);
        }
        let authority = occluders
            .get(&seg.source.record_index)
            .expect("every source SEG is classified");
        let closes = authority.kind != doom_geometry_provider::DoomSegOccluderKind::Open;
        if closes {
            for (offset, ([top, bottom], depth)) in vertical_spans.iter().copied().enumerate() {
                let x = left + offset;
                for y in top..=bottom {
                    let cell = y * COLUMNS + x;
                    if let Some((prior_depth, prior_seg)) = covering_depths[cell] {
                        if depth + 0.01 < prior_depth {
                            depth_order_inversions += 1;
                            if depth_order_samples.len() < 8 {
                                depth_order_samples.push(format!(
                                    "cell=({x},{y}) prior-seg={prior_seg} prior-depth={prior_depth:.3} later-nearer-seg={} later-depth={depth:.3}",
                                    seg.source.record_index,
                                ));
                            }
                        }
                    }
                    // Retain the first closing SEG exactly as the existing
                    // boolean control does; do not let this audit repair the
                    // experiment while it is being measured.
                    covering_depths[cell].get_or_insert((depth, seg.source.record_index));
                    covered[cell] = true;
                }
            }
            contributors += 1;
        }
        if seg.linedef == 247 || samples.len() < 8 {
            let [top, bottom] = rectangle_span();
            samples.push(format!(
                "seg={} line={} rank={} horizontal=[{left}..{right_column}] enclosing-vertical=[{top}..{bottom}] mode={} visible={visible_cells}/{cells} authority={:?} result={result} contributor={closes}",
                seg.source.record_index,
                seg.linedef,
                rank,
                if per_column { "per-column" } else { "rectangle" },
                authority.kind
            ));
        }
    }
    Ok(DoomSegScreenGridObservation {
        selected_seg_records,
        outside,
        fully_covered,
        partial,
        fully_visible,
        contributors,
        covered_cells: covered.iter().filter(|covered| **covered).count(),
        depth_order_inversions,
        depth_order_samples,
        samples,
    })
}

/// Returns the positive depth at which a source-space camera ray meets one
/// source SEG, when that intersection lies on the finite SEG. This is retained
/// only for the Stage 3B per-column diagnostic grid; it does not define a
/// generic ray query or visibility capability.
pub(crate) fn source_ray_segment_depth(
    viewer: [i16; 2],
    ray: [f64; 2],
    start: [i16; 2],
    end: [i16; 2],
) -> Option<f64> {
    let offset = [
        f64::from(start[0] - viewer[0]),
        f64::from(start[1] - viewer[1]),
    ];
    let segment = [f64::from(end[0] - start[0]), f64::from(end[1] - start[1])];
    let cross = |left: [f64; 2], right: [f64; 2]| left[0] * right[1] - left[1] * right[0];
    let denominator = cross(ray, segment);
    if denominator.abs() <= f64::EPSILON {
        return None;
    }
    let depth = cross(offset, segment) / denominator;
    let progression = cross(offset, ray) / denominator;
    (depth > 0.0 && (0.0..=1.0).contains(&progression)).then_some(depth)
}

/// Squared source-space distance from a point to one finite SEG. This is only
/// a coarse ordering probe for Stage 3B; it does not claim camera-ray order,
/// source visibility, or generic spatial-query meaning.
pub(crate) fn source_point_segment_distance_squared(
    point: [i16; 2],
    start: [i16; 2],
    end: [i16; 2],
) -> f64 {
    let offset = [
        f64::from(point[0] - start[0]),
        f64::from(point[1] - start[1]),
    ];
    let segment = [f64::from(end[0] - start[0]), f64::from(end[1] - start[1])];

    let length_squared = segment[0] * segment[0] + segment[1] * segment[1];
    let progression = if length_squared <= f64::EPSILON {
        0.0
    } else {
        ((offset[0] * segment[0] + offset[1] * segment[1]) / length_squared).clamp(0.0, 1.0)
    };
    let nearest = [
        f64::from(start[0]) + progression * segment[0],
        f64::from(start[1]) + progression * segment[1],
    ];
    let delta = [
        f64::from(point[0]) - nearest[0],
        f64::from(point[1]) - nearest[1],
    ];
    delta[0] * delta[0] + delta[1] * delta[1]
}

/// Returns contiguous not-yet-covered runs as offsets within one projected SEG
/// interval. The caller owns all screen-column meaning and source conversion.
pub(crate) fn visible_column_runs(covered: &[bool]) -> Vec<[usize; 2]> {
    let mut runs = Vec::new();
    let mut start = None;
    for (index, is_covered) in covered.iter().copied().enumerate() {
        match (start, is_covered) {
            (None, false) => start = Some(index),
            (Some(first), true) => {
                runs.push([first, index]);
                start = None;
            }
            _ => {}
        }
    }
    if let Some(first) = start {
        runs.push([first, covered.len()]);
    }
    runs
}

/// Maps a source SEG's endpoints onto the owning linedef progression. This is
/// Doom-only retained source math for the Stage 3B diagnostic lowering path.
pub(crate) fn source_seg_linedef_interval(
    map: &DoomMapCore,
    seg: &doom_map_provider::DoomSeg,
) -> [f64; 2] {
    let line = &map.linedefs[usize::from(seg.linedef)];
    let line_start = &map.vertices[usize::from(line.start_vertex)];
    let line_end = &map.vertices[usize::from(line.end_vertex)];
    let delta = [
        f64::from(line_end.x - line_start.x),
        f64::from(line_end.y - line_start.y),
    ];
    let length_squared = delta[0].mul_add(delta[0], delta[1] * delta[1]);
    let progression = |vertex: u16| {
        let point = &map.vertices[usize::from(vertex)];
        ((f64::from(point.x - line_start.x) * delta[0])
            + (f64::from(point.y - line_start.y) * delta[1]))
            / length_squared
    };
    let start = progression(seg.start_vertex);
    let end = progression(seg.end_vertex);
    [start.min(end), start.max(end)]
}

#[cfg(test)]
pub(crate) fn source_sky_sectors(spans: &DoomSegClassicPlaneSpanObservation) -> BTreeSet<u32> {
    spans
        .keys
        .iter()
        .filter(|(key, _)| key.kind == DoomSegClassicPlaneKind::Ceiling && key.texture == "F_SKY1")
        .flat_map(|(_, instances)| instances.iter())
        .flat_map(|instance| instance.source_sectors.iter().copied())
        .collect()
}
