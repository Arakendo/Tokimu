//! Fixed-column diagnostic report for the first Stage 3B screen-coverage control.
//!
//! This remains observation-only and does not prepare renderer submissions.

use super::super::*;

/// First bounded screen-coverage control for Stage 3B. It uses a fixed
/// source-space 90-degree horizontal view and 320 diagnostic columns; it is
/// neither the native renderer's projection nor a generic occlusion service.
pub(crate) fn report_doom_seg_screen_clip(
    scene: &SceneInput,
    hut_pose: bool,
) -> PlatformResult<()> {
    const COLUMNS: usize = 320;
    const HALF_FOV: f64 = std::f64::consts::FRAC_PI_4;
    let map = &scene.door_geometry_source.map;
    let viewer = scene.spawn_observer.source_position;
    let angle = if hut_pose {
        (-208.0_f64).atan2(1120.0)
    } else {
        f64::from(scene.spawn_observer.source_angle).to_radians()
    };
    let forward = [angle.cos(), angle.sin()];
    let right = [-forward[1], forward[0]];
    let order = resolve_doom_viewer_subsector_order(map, viewer)?;
    let order_by_source = order
        .iter()
        .enumerate()
        .map(|(rank, source)| (source.record_index, rank))
        .collect::<BTreeMap<_, _>>();
    let occluders = observe_doom_seg_occluders(map)?
        .into_iter()
        .map(|observation| (observation.source_seg.record_index, observation))
        .collect::<BTreeMap<_, _>>();
    let seg_triangles =
        lower_doom_seg_textured_wall_triangles(map, &scene.door_geometry_source.wall_extents)?;
    let mut triangles_by_seg = BTreeMap::<u32, Vec<_>>::new();
    for triangle in &seg_triangles {
        triangles_by_seg
            .entry(triangle.source_seg.record_index)
            .or_default()
            .push(triangle);
    }
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
    ordered_segs.sort_by_key(|(rank, seg)| (**rank, seg.source.record_index));

    let mut covered = vec![false; COLUMNS];
    let mut outside = 0usize;
    let mut fully_covered = 0usize;
    let mut partial = 0usize;
    let mut fully_visible = 0usize;
    let mut contributors = 0usize;
    let mut samples = Vec::new();
    let mut lowered_visible_triangles = 0usize;
    let mut lowered_visible_meshes = 0usize;
    let mut retained_visible_intervals = 0usize;
    let mut lowered_identity_samples = Vec::new();
    for (rank, seg) in ordered_segs {
        let start = &map.vertices[usize::from(seg.start_vertex)];
        let end = &map.vertices[usize::from(seg.end_vertex)];
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
        if start_depth <= 0.0
            || end_depth <= 0.0
            || (start_angle.abs() > HALF_FOV && end_angle.abs() > HALF_FOV)
        {
            outside += 1;
            if seg.linedef == 247 {
                samples.push(format!(
                    "seg={} line={} rank={} result=outside depth=({start_depth:.2},{end_depth:.2}) angles=({start_angle:.3},{end_angle:.3})",
                    seg.source.record_index, seg.linedef, rank
                ));
            }
            continue;
        }
        let column = |angle: f64| {
            ((angle.clamp(-HALF_FOV, HALF_FOV) + HALF_FOV) / (2.0 * HALF_FOV) * COLUMNS as f64)
                as usize
        };
        let start_column = column(start_angle).min(COLUMNS - 1);
        let end_column = column(end_angle).min(COLUMNS - 1);
        let (left, right_column) = (start_column.min(end_column), start_column.max(end_column));
        let covered_before = covered.iter().filter(|covered| **covered).count();
        let visible = (left..=right_column)
            .filter(|column| !covered[*column])
            .count();
        let span = right_column - left + 1;
        let result = if visible == 0 {
            fully_covered += 1;
            "covered"
        } else if visible == span {
            fully_visible += 1;
            "visible"
        } else {
            partial += 1;
            "partial"
        };
        let authority = occluders
            .get(&seg.source.record_index)
            .expect("every source SEG is classified");
        let closes = authority.kind != doom_geometry_provider::DoomSegOccluderKind::Open;
        // This interval extraction intentionally uses the fixed diagnostic
        // columns, not renderer pixels or historic Doom projection. It is
        // sufficient only to lower source-labelled comparison geometry; the
        // next checkpoint must still upload and visually compare that separate
        // representation.
        let visible_runs = visible_column_runs(&covered[left..=right_column]);
        let line_interval = source_seg_linedef_interval(map, seg);
        for [run_start, run_end] in visible_runs {
            let start_fraction = run_start as f64 / span as f64;
            let end_fraction = run_end as f64 / span as f64;
            let interval = [
                line_interval[0] + (line_interval[1] - line_interval[0]) * start_fraction,
                line_interval[0] + (line_interval[1] - line_interval[0]) * end_fraction,
            ];
            retained_visible_intervals += 1;
            for triangle in triangles_by_seg
                .get(&seg.source.record_index)
                .into_iter()
                .flatten()
            {
                let extent = scene
                    .door_geometry_source
                    .wall_extents
                    .iter()
                    .find(|extent| extent.name == triangle.texture_name)
                    .cloned()
                    .ok_or_else(|| {
                        io::Error::other(format!(
                            "Stage 3B visible SEG `{}` has no texture extent",
                            triangle.texture_name
                        ))
                    })?;
                for clipped in clip_doom_seg_textured_wall_triangle_to_linedef_interval(
                    map, triangle, interval,
                )? {
                    lowered_visible_triangles += 1;
                    let lowered = lower_static_seg_wall_triangle(&clipped, extent.clone())?;
                    lowered_visible_meshes += 1;
                    if lowered_identity_samples.len() < 8 {
                        lowered_identity_samples.push(format!(
                            "seg={} line={} side={:?} role={:?} texture={} interval=[{:.3},{:.3}] vertices={}",
                            lowered.source_seg.record_index,
                            lowered.wall.source_linedef.record_index,
                            lowered.wall.side,
                            lowered.wall.role,
                            lowered.wall.texture_name,
                            interval[0],
                            interval[1],
                            lowered.wall.mesh.positions.len(),
                        ));
                    }
                }
            }
        }
        if closes {
            covered[left..=right_column].fill(true);
            contributors += 1;
        }
        let covered_after = covered.iter().filter(|covered| **covered).count();
        if seg.linedef == 247 || samples.len() < 8 {
            samples.push(format!(
                "seg={} line={} rank={} interval=[{left},{right_column}] visible={visible}/{span} authority={:?} result={result} coverage={covered_before}->{covered_after} contributor={closes}",
                seg.source.record_index, seg.linedef, rank, authority.kind
            ));
        }
    }
    println!("E1M1 AR-0025 Stage 3B screen-span control: columns={COLUMNS}; outside={outside}; fully_covered={fully_covered}; partial={partial}; fully_visible={fully_visible}; coverage_contributors={contributors}; covered_columns={}; meaning=fixed-source-space-experiment-not-renderer-visibility", covered.iter().filter(|covered| **covered).count());
    println!(
        "E1M1 AR-0025 Stage 3B screen-span samples: {}",
        samples.join(" | ")
    );
    println!(
        "E1M1 AR-0025 Stage 3B visible-SEG lowering: visible_intervals={retained_visible_intervals}; source_triangles={lowered_visible_triangles}; lowered_meshes={lowered_visible_meshes}; meaning=separate-source-labelled-comparison-representation-not-uploaded-or-renderer-visibility"
    );
    println!(
        "E1M1 AR-0025 Stage 3B visible-SEG lowering samples: {}",
        lowered_identity_samples.join(" | ")
    );
    Ok(())
}
