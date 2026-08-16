//! Mechanically extracted AR-0025 evidence subject.

use super::super::super::*;

/// Builds the deliberately bounded Stage 3B presentation control from the
/// same fixed source-space screen-span experiment reported above. This is not
/// a Doom renderer reconstruction: it only gives the corpus a visually
/// inspectable representation of the retained SEG subintervals.
pub(crate) fn prepare_doom_seg_clip_presentation(
    scene: &SceneInput,
    hut_pose: bool,
) -> PlatformResult<DoomSegClipPresentation> {
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

    let project = |point: [i16; 2]| {
        let relative = [
            f64::from(point[0] - viewer[0]),
            f64::from(point[1] - viewer[1]),
        ];
        let depth = relative[0] * forward[0] + relative[1] * forward[1];
        let lateral = relative[0] * right[0] + relative[1] * right[1];
        (depth, lateral.atan2(depth))
    };
    let column = |angle: f64| {
        ((angle.clamp(-HALF_FOV, HALF_FOV) + HALF_FOV) / (2.0 * HALF_FOV) * COLUMNS as f64) as usize
    };

    let mut covered = vec![false; COLUMNS];
    let mut draws = Vec::new();
    let mut visible_intervals = 0usize;
    let mut source_triangles = 0usize;
    for (_, seg) in ordered_segs {
        let start = &map.vertices[usize::from(seg.start_vertex)];
        let end = &map.vertices[usize::from(seg.end_vertex)];
        let (start_depth, start_angle) = project([start.x, start.y]);
        let (end_depth, end_angle) = project([end.x, end.y]);
        if start_depth <= 0.0
            || end_depth <= 0.0
            || (start_angle.abs() > HALF_FOV && end_angle.abs() > HALF_FOV)
        {
            continue;
        }
        let start_column = column(start_angle).min(COLUMNS - 1);
        let end_column = column(end_angle).min(COLUMNS - 1);
        let (left, right_column) = (start_column.min(end_column), start_column.max(end_column));
        let span = right_column - left + 1;
        let line_interval = source_seg_linedef_interval(map, seg);
        for [run_start, run_end] in visible_column_runs(&covered[left..=right_column]) {
            let start_fraction = run_start as f64 / span as f64;
            let end_fraction = run_end as f64 / span as f64;
            let interval = [
                line_interval[0] + (line_interval[1] - line_interval[0]) * start_fraction,
                line_interval[0] + (line_interval[1] - line_interval[0]) * end_fraction,
            ];
            visible_intervals += 1;
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
                    let lowered = lower_static_seg_wall_triangle(&clipped, extent.clone())?;
                    let material = scene
                        .door_geometry_source
                        .wall_materials
                        .get(&lowered.wall.texture_name)
                        .copied()
                        .ok_or_else(|| {
                            io::Error::other(format!(
                                "Stage 3B visible SEG `{}` has no wall material",
                                lowered.wall.texture_name
                            ))
                        })?;
                    source_triangles += 1;
                    draws.push(StaticDrawPlanEntry {
                        source_label: format!(
                            "seg-clip:{}:{}:{:?}:{:?}:{}:{:.3}-{:.3}",
                            lowered.source_seg.record_index,
                            lowered.wall.source_linedef.record_index,
                            lowered.wall.side,
                            lowered.wall.role,
                            lowered.wall.texture_name,
                            interval[0],
                            interval[1],
                        ),
                        source: StaticDrawSource::Wall {
                            source_linedef: lowered.wall.source_linedef,
                            source_sidedef: lowered.wall.source_sidedef,
                            source_sector: lowered.wall.source_sector,

                            role: lowered.wall.role,
                        },
                        mesh: lowered.wall.mesh,
                        material,
                    });
                }
            }
        }
        let authority = occluders
            .get(&seg.source.record_index)
            .expect("every source SEG is classified");
        if authority.kind != doom_geometry_provider::DoomSegOccluderKind::Open {
            covered[left..=right_column].fill(true);
        }
    }

    Ok(DoomSegClipPresentation {
        draws,
        visible_intervals,
        source_triangles,
    })
}

/// Adapts the retained per-column source-grid observation into ordinary
/// source-labelled wall draws for a manual comparison. A SEG survives as a
/// whole piece when at least one of its bounded grid cells remains uncovered;
/// this deliberately fails open rather than claiming pixel-exact clipping.
pub(crate) fn prepare_doom_seg_per_column_presentation(
    scene: &SceneInput,
) -> PlatformResult<DoomSegPerColumnPresentation> {
    let observation = observe_doom_seg_screen_grid(
        &scene.door_geometry_source.map,
        scene.spawn_observer.position.y,
        true,
        scene.spawn_observer.source_position,
        f64::from(scene.spawn_observer.source_angle).to_radians(),
    )?;
    let map = &scene.door_geometry_source.map;
    let triangles =
        lower_doom_seg_textured_wall_triangles(map, &scene.door_geometry_source.wall_extents)?;
    let mut wall_draws = Vec::new();
    for triangle in triangles.iter().filter(|triangle| {
        observation
            .selected_seg_records
            .contains(&triangle.source_seg.record_index)
    }) {
        let extent = scene
            .door_geometry_source
            .wall_extents
            .iter()
            .find(|extent| extent.name == triangle.texture_name)
            .cloned()
            .ok_or_else(|| {
                io::Error::other(format!(
                    "Stage 3B selected SEG `{}` has no texture extent",
                    triangle.texture_name
                ))
            })?;
        let lowered = match lower_static_seg_wall_triangle(triangle, extent) {
            Ok(lowered) => lowered,
            Err(StaticFlatLoweringError::DegenerateTriangle) => continue,
            Err(error) => return Err(error.into()),
        };
        let material = scene
            .door_geometry_source
            .wall_materials
            .get(&lowered.wall.texture_name)
            .copied()
            .ok_or_else(|| {
                io::Error::other(format!(
                    "Stage 3B selected SEG `{}` has no wall material",
                    lowered.wall.texture_name
                ))
            })?;
        wall_draws.push(StaticDrawPlanEntry {
            source_label: format!(
                "seg-grid:{}:{}:{:?}:{:?}:{}",
                lowered.source_seg.record_index,
                lowered.wall.source_linedef.record_index,
                lowered.wall.side,
                lowered.wall.role,
                lowered.wall.texture_name,
            ),
            source: StaticDrawSource::Wall {
                source_linedef: lowered.wall.source_linedef,
                source_sidedef: lowered.wall.source_sidedef,
                source_sector: lowered.wall.source_sector,
                role: lowered.wall.role,
            },
            mesh: lowered.wall.mesh,
            material,
        });
    }
    Ok(DoomSegPerColumnPresentation {
        wall_draws,
        selected_segs: observation.selected_seg_records.len(),
    })
}

/// Prepares every SEG-derived wall once, retaining the original flat/cutout
/// draws. The runtime control later filters this stable set by source SEG
/// identity, so observer movement cannot cause mesh uploads or replacements.
pub(crate) fn prepare_doom_seg_per_column_dynamic_scene(
    scene: &mut SceneInput,
) -> PlatformResult<DoomSegDynamicSelectionInput> {
    let map = &scene.door_geometry_source.map;
    let triangles =
        lower_doom_seg_textured_wall_triangles(map, &scene.door_geometry_source.wall_extents)?;
    let unsupported_linedefs = triangles
        .iter()
        .filter(|triangle| {
            !scene
                .door_geometry_source
                .wall_materials
                .contains_key(&triangle.texture_name)
        })
        .map(|triangle| triangle.source_linedef.record_index)
        .collect::<BTreeSet<_>>();
    scene.opaque_draws.retain(|draw| match draw.source {
        StaticDrawSource::Wall { source_linedef, .. } => {
            unsupported_linedefs.contains(&source_linedef.record_index)
        }
        _ => true,
    });
    let mut unsupported_textures = BTreeSet::new();
    for triangle in triangles {
        let extent = scene
            .door_geometry_source
            .wall_extents
            .iter()
            .find(|extent| extent.name == triangle.texture_name)
            .cloned()
            .ok_or_else(|| {
                io::Error::other(format!(
                    "Stage 3B dynamic SEG `{}` has no texture extent",
                    triangle.texture_name
                ))
            })?;
        let lowered = match lower_static_seg_wall_triangle(&triangle, extent) {
            Ok(lowered) => lowered,
            // Preserve the established E1M1 rule: confirmed zero-area source
            // candidates are retained omissions, never fabricated normals.
            Err(StaticFlatLoweringError::DegenerateTriangle) => continue,
            Err(error) => return Err(error.into()),
        };
        let Some(material) = scene
            .door_geometry_source
            .wall_materials
            .get(&lowered.wall.texture_name)
            .copied()
        else {
            unsupported_textures.insert(lowered.wall.texture_name);
            continue;
        };
        scene.opaque_draws.push(StaticDrawPlanEntry {
            source_label: format!(
                "seg-dynamic:{}:{}:{:?}:{:?}:{}",
                lowered.source_seg.record_index,
                lowered.wall.source_linedef.record_index,
                lowered.wall.side,
                lowered.wall.role,
                lowered.wall.texture_name,
            ),
            source: StaticDrawSource::Wall {
                source_linedef: lowered.wall.source_linedef,

                source_sidedef: lowered.wall.source_sidedef,
                source_sector: lowered.wall.source_sector,
                role: lowered.wall.role,
            },
            mesh: lowered.wall.mesh,
            material,
        });
    }
    let mut draw_indices_by_seg = BTreeMap::<u32, Vec<usize>>::new();
    let mut flat_indices_by_subsector = BTreeMap::<u32, Vec<usize>>::new();
    for (index, draw) in scene.opaque_draws.iter().enumerate() {
        if let Some(seg) = draw
            .source_label
            .strip_prefix("seg-dynamic:")
            .and_then(|label| label.split(':').next())
            .and_then(|record| record.parse::<u32>().ok())
        {
            draw_indices_by_seg.entry(seg).or_default().push(index);
        }
        if let StaticDrawSource::Flat {
            source_subsector, ..
        } = draw.source
        {
            flat_indices_by_subsector
                .entry(source_subsector.record_index)
                .or_default()
                .push(index);
        }
    }
    Ok(DoomSegDynamicSelectionInput {
        draw_indices_by_seg,
        flat_indices_by_subsector,
        unsupported_textures,
    })
}

pub(crate) fn observe_doom_seg_classic_plane_identities(
    map: &DoomMapCore,
    plane_marks: &[DoomSegPlaneMarkObservation],
    traversal: &DoomSegClassicBspObservation,
) -> DoomSegClassicPlaneIdentityObservation {
    let sectors_by_record = map
        .sectors
        .iter()
        .map(|sector| (sector.source.record_index, sector))
        .collect::<BTreeMap<_, _>>();
    let marks_by_seg = plane_marks
        .iter()
        .map(|mark| (mark.source_seg.record_index, mark))
        .collect::<BTreeMap<_, _>>();
    let mut result = DoomSegClassicPlaneIdentityObservation::default();
    let mut floor_keys = BTreeSet::new();
    let mut ceiling_keys = BTreeSet::new();

    for source_seg in &traversal.admitted_seg_order {
        let Some(mark) = marks_by_seg.get(source_seg) else {
            continue;
        };
        let sector = sectors_by_record
            .get(&mark.front_sector.record_index)
            .expect("validated plane mark names an existing front sector");
        if mark.floor_marked {
            result.floor_mark_contributors += 1;
            let key = (
                sector.floor_height,
                sector.floor_texture.clone(),
                sector.light_level,
            );
            if floor_keys.insert(key.clone()) && result.samples.len() < 12 {
                result.samples.push(format!(
                    "floor-sector={} height={} flat={} light={}",
                    mark.front_sector.record_index, key.0, key.1, key.2,
                ));
            }
        }
        if mark.ceiling_marked {
            result.ceiling_mark_contributors += 1;
            let sky = sector.ceiling_texture == "F_SKY1";
            result.sky_ceiling_contributors += usize::from(sky);
            let key = if sky {
                (0, String::from("F_SKY1"), 0)
            } else {
                (
                    sector.ceiling_height,
                    sector.ceiling_texture.clone(),
                    sector.light_level,
                )
            };
            if ceiling_keys.insert(key.clone()) && result.samples.len() < 12 {
                result.samples.push(format!(
                    "ceiling-sector={} height={} flat={} light={} sky={sky}",
                    mark.front_sector.record_index, key.0, key.1, key.2,
                ));
            }
        }
    }
    result.unique_floor_keys = floor_keys.len();
    result.unique_ceiling_keys = ceiling_keys.len();
    result
}

#[cfg(test)]
pub(crate) fn doom_seg_classic_plane_key(
    kind: DoomSegClassicPlaneKind,
    sector: &doom_map_provider::DoomSector,
) -> DoomSegClassicPlaneKey {
    if kind == DoomSegClassicPlaneKind::Ceiling && sector.ceiling_texture == "F_SKY1" {
        DoomSegClassicPlaneKey {
            kind,
            height: 0,
            texture: String::from("F_SKY1"),
            light: 0,
        }
    } else {
        DoomSegClassicPlaneKey {
            kind,
            height: match kind {
                DoomSegClassicPlaneKind::Floor => sector.floor_height,
                DoomSegClassicPlaneKind::Ceiling => sector.ceiling_height,
            },
            texture: match kind {
                DoomSegClassicPlaneKind::Floor => sector.floor_texture.clone(),
                DoomSegClassicPlaneKind::Ceiling => sector.ceiling_texture.clone(),
            },
            light: sector.light_level,
        }
    }
}

#[cfg(test)]
pub(crate) fn retain_doom_seg_classic_plane_range(
    observation: &mut DoomSegClassicPlaneSpanObservation,
    key: DoomSegClassicPlaneKey,
    source_sector: u32,
    source_seg: u32,
    writes: &[(usize, usize, usize)],
    columns: usize,
) {
    let valid = writes
        .iter()
        .filter_map(|&(column, top, bottom)| {
            if top > bottom {
                observation.empty_after_clip += 1;
                None
            } else {
                Some((column, top, bottom))
            }
        })
        .collect::<Vec<_>>();
    let Some(minimum_column) = valid.iter().map(|(column, _, _)| *column).min() else {
        return;
    };
    let maximum_column = valid
        .iter()
        .map(|(column, _, _)| *column)
        .max()
        .expect("a minimum column proves at least one valid plane write");
    let instances = observation.keys.entry(key).or_default();
    let compatible = instances.iter().position(|instance| {
        let intersection_start = minimum_column.max(instance.minimum_column);
        let intersection_end = maximum_column.min(instance.maximum_column);
        intersection_start > intersection_end
            || instance.columns[intersection_start..=intersection_end]
                .iter()
                .all(Option::is_none)
    });
    let instance_index = compatible.unwrap_or_else(|| {
        if !instances.is_empty() {
            observation.collision_splits += 1;
        }
        instances.push(DoomSegClassicPlaneInstance {
            columns: vec![None; columns],
            column_sources: vec![None; columns],
            minimum_column,
            maximum_column,
            source_sectors: BTreeSet::new(),
            source_segs: BTreeSet::new(),
        });
        instances.len() - 1
    });
    let instance = &mut instances[instance_index];
    instance.source_sectors.insert(source_sector);
    instance.source_segs.insert(source_seg);
    instance.minimum_column = instance.minimum_column.min(minimum_column);
    instance.maximum_column = instance.maximum_column.max(maximum_column);
    for (column, top, bottom) in valid {
        let slot = &mut instance.columns[column];
        if slot.is_some() {
            observation.overlapping_writes += 1;
        } else {
            *slot = Some([top, bottom]);
            instance.column_sources[column] = Some([source_sector, source_seg]);
        }
    }
}

pub(crate) fn resolve_doom_seg_classic_plane_flats(
    scene: &SceneInput,
    spans: &DoomSegClassicPlaneSpanObservation,
) -> DoomSegClassicPlaneFlatResolution {
    let mut result = DoomSegClassicPlaneFlatResolution::default();
    for (key, instances) in &spans.keys {
        for (instance_index, instance) in instances.iter().enumerate() {
            if key.kind == DoomSegClassicPlaneKind::Ceiling && key.texture == "F_SKY1" {
                result.sky_instances += 1;
                if result.samples.len() < 12 {
                    result.samples.push(format!(
                        "kind={:?} flat={} instance={} sectors={:?} result=sky-presentation",
                        key.kind, key.texture, instance_index, instance.source_sectors,
                    ));
                }
                continue;
            }

            let expected_plane = match key.kind {
                DoomSegClassicPlaneKind::Floor => DoomSurfacePlane::Floor,
                DoomSegClassicPlaneKind::Ceiling => DoomSurfacePlane::Ceiling,
            };
            let candidates = scene
                .opaque_draws
                .iter()
                .filter(|draw| {
                    matches!(
                        draw.source,
                        StaticDrawSource::Flat {
                            source_sector,
                            plane,
                            ..
                        } if plane == expected_plane
                            && instance.source_sectors.contains(&source_sector.record_index)
                    )
                })
                .collect::<Vec<_>>();
            let triangles = candidates
                .iter()
                .map(|draw| draw.mesh.positions.len() / 3)
                .sum::<usize>();
            result.candidate_draws += candidates.len();
            result.candidate_triangles += triangles;
            if candidates.is_empty() {
                result.unresolved_instances += 1;
            } else {
                result.resolved_instances += 1;
            }
            if result.samples.len() < 12 {
                result.samples.push(format!(
                    "kind={:?} flat={} instance={} sectors={:?} segs={} candidate-draws={} candidate-triangles={}",
                    key.kind,
                    key.texture,
                    instance_index,
                    instance.source_sectors,
                    instance.source_segs.len(),
                    candidates.len(),
                    triangles,
                ));
            }
        }
    }
    result
}

fn observe_fixed_source_ordered_coverage(
    scene: &SceneInput,
) -> PlatformResult<(DoomSegClassicVerticalClipObservation, [i16; 2], f64, f64)> {
    let viewer = scene.spawn_observer.source_position;
    let heading = f64::from(scene.spawn_observer.source_angle).to_radians();
    let eye_height = scene.spawn_observer.position.y as f64;
    let traversal = observe_doom_seg_classic_bsp(
        &scene.door_geometry_source.map,
        viewer,
        heading,
        &BTreeSet::new(),
    )?;
    let lowerable_triangles = lower_doom_seg_textured_wall_triangles(
        &scene.door_geometry_source.map,
        &scene.door_geometry_source.wall_extents,
    )?;
    let plane_marks =
        observe_doom_seg_plane_marks(&scene.door_geometry_source.map, eye_height as i16)?;
    let vertical = observe_shared_doom_classic_vertical_clip_state(
        &scene.door_geometry_source.map,
        &lowerable_triangles,
        &plane_marks,
        &traversal,
        viewer,
        heading,
        eye_height,
    );
    Ok((vertical, viewer, heading, eye_height))
}

fn prepare_doom_ordered_coverage_observation(
    scene: &SceneInput,
    view: DoomOrderedCoverageView,
) -> PlatformResult<DoomOrderedCoveragePreparation> {
    prepare_doom_ordered_coverage(
        &scene.door_geometry_source.map,
        &scene.door_geometry_source.wall_extents,
        view.source_position,
        view.source_heading_radians,
        view.eye_height,
        true,
    )
}

pub(crate) fn prepare_doom_seg_classic_plane_presentation(
    scene: &SceneInput,
) -> PlatformResult<DoomSegClassicPlanePresentation> {
    let (vertical, viewer, heading, eye_height) = observe_fixed_source_ordered_coverage(scene)?;
    let reconstruction = reconstruct_doom_seg_classic_plane_cells(
        &vertical.plane_spans,
        viewer,
        heading,
        eye_height,
    );
    lower_doom_seg_classic_plane_presentation(
        &scene.door_geometry_source.map,
        &scene.opaque_uploads,
        DoomComparativeEmbedding::CurrentReflected,
        reconstruction,
    )
}

/// Reconstructs the provider's retained per-column wall cells into grouped
/// ordinary meshes while preserving source and material identity. This is the
/// explicit Slice 7 falsification candidate: it may prove that source-derived
/// partial fragments are sufficient, or expose that the retained intervals are
/// still incomplete. It is not the default E1M1 preparation path.
pub(crate) fn prepare_doom_seg_ordered_coverage_presentation(
    scene: &SceneInput,
) -> PlatformResult<DoomSegOrderedCoveragePresentation> {
    prepare_doom_seg_ordered_coverage_presentation_for_view(
        scene,
        DoomOrderedCoverageView {
            source_position: scene.spawn_observer.source_position,
            source_heading_radians: f64::from(scene.spawn_observer.source_angle).to_radians(),
            eye_height: scene.spawn_observer.position.y as f64,
        },
    )
}

/// Rebuilds the Doom-owned ordered preparation for one explicit live view.
/// The result remains ordinary render declarations; no traversal or coverage
/// state crosses into `tokimu-render`.
pub(crate) fn prepare_doom_seg_ordered_coverage_presentation_for_view(
    scene: &SceneInput,
    view: DoomOrderedCoverageView,
) -> PlatformResult<DoomSegOrderedCoveragePresentation> {
    struct WallGroup {
        source_seg: doom_map_provider::DoomSourceRecord,
        source_linedef: doom_map_provider::DoomSourceRecord,
        source_sidedef: doom_map_provider::DoomSourceRecord,
        source_sector: doom_map_provider::DoomSourceRecord,
        role: DoomWallTextureRole,
        texture_name: String,
        material: MaterialHandle,
        cutout: bool,
        positions: Vec<[f32; 3]>,
        normals: Vec<[f32; 3]>,
        texture_coordinates: Vec<[f32; 2]>,
    }

    let preparation = prepare_doom_ordered_coverage_observation(scene, view)?;
    let DoomOrderedCoveragePreparation {
        traversal,
        vertical,
        walls: reconstruction,
        planes: plane_reconstruction,
        ordinary_plane_intervals,
        sky_plane_intervals,
    } = preparation;
    let rejected_plane_intervals = plane_reconstruction.horizon_rejections
        + plane_reconstruction.behind_viewer_rejections
        + plane_reconstruction.degenerate_rejections;
    if ordinary_plane_intervals
        != plane_reconstruction.reconstructed_quads + rejected_plane_intervals
    {
        return Err(io::Error::other(format!(
            "ordered coverage plane conservation failed: retained ordinary intervals={ordinary_plane_intervals}, reconstructed={}, rejected={rejected_plane_intervals}",
            plane_reconstruction.reconstructed_quads,
        ))
        .into());
    }
    let reconstructed_plane_quads = plane_reconstruction.reconstructed_quads;
    let planes = lower_doom_seg_classic_plane_presentation(
        &scene.door_geometry_source.map,
        &scene.opaque_uploads,
        DoomComparativeEmbedding::CurrentReflected,
        plane_reconstruction,
    )?;
    let lowered_plane_quads = planes.triangles / 2;
    if lowered_plane_quads != reconstructed_plane_quads {
        return Err(io::Error::other(format!(
            "ordered coverage plane lowering lost contributions: reconstructed quads={reconstructed_plane_quads}, lowered quads={lowered_plane_quads}",
        ))
        .into());
    }

    let cutout_materials = scene
        .cutout_draws
        .iter()
        .filter_map(|draw| match draw.source {
            StaticDrawSource::Wall {
                source_linedef,
                source_sidedef,
                role,
                ..
            } => Some((
                (
                    source_linedef.record_index,
                    source_sidedef.record_index,
                    doom_wall_role_key(role),
                ),
                draw.material,
            )),
            StaticDrawSource::Flat { .. } => None,
        })
        .collect::<BTreeMap<_, _>>();
    let retained_middle_segs = vertical
        .ordered_wall_intervals
        .iter()
        .filter(|interval| {
            interval.role == DoomWallTextureRole::Middle && interval.retained_interval.is_some()
        })
        .map(|interval| interval.source_seg)
        .collect::<BTreeSet<_>>();
    let source_cutout_keys = retained_middle_segs
        .iter()
        .filter_map(|source_seg| {
            let seg = scene
                .door_geometry_source
                .map
                .segs
                .iter()
                .find(|seg| seg.source.record_index == *source_seg)?;
            let linedef = &scene.door_geometry_source.map.linedefs[usize::from(seg.linedef)];
            let sidedef_index = match seg.direction {
                0 => linedef.right_sidedef,
                1 => linedef.left_sidedef,
                _ => None,
            }?;
            let sidedef = &scene.door_geometry_source.map.sidedefs[usize::from(sidedef_index)];
            let key = (
                linedef.source.record_index,
                sidedef.source.record_index,
                doom_wall_role_key(DoomWallTextureRole::Middle),
            );
            cutout_materials.contains_key(&key).then_some(key)
        })
        .collect::<BTreeSet<_>>();
    let extents = scene
        .door_geometry_source
        .wall_extents
        .iter()
        .map(|extent| (extent.name.as_str(), extent.clone()))
        .collect::<BTreeMap<_, _>>();
    let mut groups = BTreeMap::<(u32, u32, u8, String, bool), WallGroup>::new();
    let source_degenerate_cells = reconstruction.degenerate_cells;
    let source_unresolved_cells = reconstruction.unresolved_cells;
    let reconstructed_triangles = reconstruction.reconstructed_triangles.len();
    if reconstructed_triangles % 2 != 0
        || reconstruction.retained_cells
            != reconstructed_triangles / 2 + source_degenerate_cells + source_unresolved_cells
    {
        return Err(io::Error::other(format!(
            "ordered coverage wall reconstruction conservation failed: retained cells={}, reconstructed triangles={reconstructed_triangles}, source-degenerate cells={source_degenerate_cells}, source-unresolved cells={source_unresolved_cells}",
            reconstruction.retained_cells,
        ))
        .into());
    }
    let mut lowering_unresolved_triangles = 0;
    let mut lowering_degenerate_triangles = 0;
    let mut samples = reconstruction.samples.clone();

    for triangle in &reconstruction.reconstructed_triangles {
        let role_key = doom_wall_role_key(triangle.role);
        let cutout_key = (
            triangle.source_linedef.record_index,
            triangle.source_sidedef.record_index,
            role_key,
        );
        let (material, cutout) = if let Some(material) = cutout_materials.get(&cutout_key) {
            (*material, true)
        } else if let Some(material) = scene
            .door_geometry_source
            .wall_materials
            .get(&triangle.texture_name)
        {
            (*material, false)
        } else {
            lowering_unresolved_triangles += 1;
            if samples.len() < 12 {
                samples.push(format!(
                    "seg={}:linedef={}:texture={}:reason=material-unresolved",
                    triangle.source_seg.record_index,
                    triangle.source_linedef.record_index,
                    triangle.texture_name,
                ));
            }
            continue;
        };
        let Some(extent) = extents.get(triangle.texture_name.as_str()).cloned() else {
            lowering_unresolved_triangles += 1;
            if samples.len() < 12 {
                samples.push(format!(
                    "seg={}:linedef={}:texture={}:reason=extent-unresolved",
                    triangle.source_seg.record_index,
                    triangle.source_linedef.record_index,
                    triangle.texture_name,
                ));
            }
            continue;
        };

        let lowered = match lower_static_seg_wall_triangle(triangle, extent) {
            Ok(lowered) => lowered,
            Err(StaticFlatLoweringError::DegenerateTriangle) => {
                lowering_degenerate_triangles += 1;
                if samples.len() < 12 {
                    samples.push(format!(
                        "seg={}:linedef={}:texture={}:omitted=degenerate-reconstructed-fragment",
                        triangle.source_seg.record_index,
                        triangle.source_linedef.record_index,
                        triangle.texture_name,
                    ));
                }
                continue;
            }
            Err(error) => return Err(error.into()),
        };
        let key = (
            triangle.source_seg.record_index,
            triangle.source_sidedef.record_index,
            role_key,
            triangle.texture_name.clone(),
            cutout,
        );
        let group = groups.entry(key).or_insert_with(|| WallGroup {
            source_seg: triangle.source_seg,
            source_linedef: triangle.source_linedef,
            source_sidedef: triangle.source_sidedef,
            source_sector: triangle.source_sector,
            role: triangle.role,
            texture_name: triangle.texture_name.clone(),
            material,
            cutout,
            positions: Vec::new(),
            normals: Vec::new(),
            texture_coordinates: Vec::new(),
        });
        group.positions.extend(lowered.wall.mesh.positions);
        group.normals.extend(lowered.wall.mesh.normals);
        group
            .texture_coordinates
            .extend(lowered.wall.mesh.texture_coordinates);
    }

    let grouped_wall_meshes = groups.len();
    let lowered_wall_triangles = groups
        .values()
        .map(|group| group.positions.len() / 3)
        .sum::<usize>();
    if reconstructed_triangles
        != lowered_wall_triangles + lowering_degenerate_triangles + lowering_unresolved_triangles
    {
        return Err(io::Error::other(format!(
            "ordered coverage wall lowering conservation failed: reconstructed triangles={reconstructed_triangles}, lowered triangles={lowered_wall_triangles}, lowering-degenerate triangles={lowering_degenerate_triangles}, lowering-unresolved triangles={lowering_unresolved_triangles}",
        ))
        .into());
    }
    let mut opaque_draws = planes.draws;
    let mut cutout_draws = Vec::new();
    for (_, group) in groups {
        let mesh = Mesh::new(group.positions, group.normals)
            .with_texture_coordinates(group.texture_coordinates)?;
        let draw = StaticDrawPlanEntry {
            source_label: format!(
                "ordered-coverage-wall:{}:{}:{:?}:{}",
                group.source_seg.record_index,
                group.source_linedef.record_index,
                group.role,
                group.texture_name,
            ),
            source: StaticDrawSource::Wall {
                source_linedef: group.source_linedef,
                source_sidedef: group.source_sidedef,
                source_sector: group.source_sector,
                role: group.role,
            },
            mesh,

            material: group.material,
        };
        if group.cutout {
            cutout_draws.push(draw);
        } else {
            opaque_draws.push(draw);
        }
    }
    let lowered_cutout_keys = cutout_draws
        .iter()
        .filter_map(|draw| match draw.source {
            StaticDrawSource::Wall {
                source_linedef,
                source_sidedef,
                role,
                ..
            } => Some((
                source_linedef.record_index,
                source_sidedef.record_index,
                doom_wall_role_key(role),
            )),
            StaticDrawSource::Flat { .. } => None,
        })
        .collect::<BTreeSet<_>>();
    if lowered_cutout_keys != source_cutout_keys {
        let missing = source_cutout_keys
            .difference(&lowered_cutout_keys)
            .copied()
            .collect::<Vec<_>>();
        let fabricated = lowered_cutout_keys
            .difference(&source_cutout_keys)
            .copied()
            .collect::<Vec<_>>();
        return Err(io::Error::other(format!(
            "ordered coverage cutout conservation failed: missing={missing:?}, fabricated={fabricated:?}",
        ))
        .into());
    }

    Ok(DoomSegOrderedCoveragePresentation {
        opaque_draws,
        cutout_draws,
        retained_cells: reconstruction.retained_cells,
        reconstructed_triangles,
        lowered_wall_triangles,
        source_degenerate_cells,
        source_unresolved_cells,
        lowering_degenerate_triangles,
        lowering_unresolved_triangles,
        grouped_wall_meshes,
        ordinary_plane_intervals,
        sky_plane_intervals,
        reconstructed_plane_quads,
        rejected_plane_intervals,
        lowered_plane_quads,
        source_cutout_keys: source_cutout_keys.len(),
        lowered_cutout_keys: lowered_cutout_keys.len(),
        coverage_transitions: vertical.ordered_coverage_transitions.len(),
        coverage_fail_open: vertical.ordered_coverage_fail_open.len(),
        coverage_fail_open_reasons: DoomCoverageFailOpenSummary::default(),
        bsp_leaves_visited: traversal.leaves_visited,
        bsp_far_children_pruned: traversal.far_children_pruned,
        bsp_admitted_segs: traversal.admitted_seg_records.len(),
        bsp_solid_range_pruning: true,
        degenerate_omissions: source_degenerate_cells + lowering_degenerate_triangles,
        unresolved_cells: source_unresolved_cells + lowering_unresolved_triangles,
        samples,
    })
}

/// Adds the source-spawn BSP-admitted, already lowerable opaque wall tiers to
/// the reconstructed planes so a maintainer can judge plane gaps in context.
/// Wall tiers remain whole SEG fragments here; exact projected tier clipping
/// is deliberately not claimed by this intermediate visual control.
pub(crate) fn prepare_doom_seg_classic_context_presentation(
    scene: &SceneInput,
) -> PlatformResult<DoomSegClassicContextPresentation> {
    let planes = prepare_doom_seg_classic_plane_presentation(scene)?;
    let viewer = scene.spawn_observer.source_position;
    let heading = f64::from(scene.spawn_observer.source_angle).to_radians();
    let traversal = observe_doom_seg_classic_bsp(
        &scene.door_geometry_source.map,
        viewer,
        heading,
        &BTreeSet::new(),
    )?;
    let triangles = lower_doom_seg_textured_wall_triangles(
        &scene.door_geometry_source.map,
        &scene.door_geometry_source.wall_extents,
    )?;
    let mut draws = planes.draws;
    let mut wall_meshes = 0usize;
    let mut omitted_wall_triangles = 0usize;
    for triangle in triangles.iter().filter(|triangle| {
        traversal
            .admitted_seg_records
            .contains(&triangle.source_seg.record_index)
    }) {
        let Some(material) = scene
            .door_geometry_source
            .wall_materials
            .get(&triangle.texture_name)
            .copied()
        else {
            omitted_wall_triangles += 1;
            continue;
        };
        let extent = scene
            .door_geometry_source
            .wall_extents
            .iter()
            .find(|extent| extent.name == triangle.texture_name)
            .cloned()
            .ok_or_else(|| {
                io::Error::other(format!(
                    "classic context wall `{}` has no texture extent",
                    triangle.texture_name
                ))
            })?;
        let lowered = match lower_static_seg_wall_triangle(triangle, extent) {
            Ok(lowered) => lowered,
            Err(StaticFlatLoweringError::DegenerateTriangle) => {
                omitted_wall_triangles += 1;
                continue;
            }
            Err(error) => return Err(error.into()),
        };
        wall_meshes += 1;
        draws.push(StaticDrawPlanEntry {
            source_label: format!(
                "classic-context-wall:{}:{}:{}",
                lowered.source_seg.record_index,
                lowered.wall.source_linedef.record_index,
                lowered.wall.texture_name,
            ),
            source: StaticDrawSource::Wall {
                source_linedef: lowered.wall.source_linedef,
                source_sidedef: lowered.wall.source_sidedef,
                source_sector: lowered.wall.source_sector,
                role: lowered.wall.role,
            },
            mesh: lowered.wall.mesh,
            material,
        });
    }

    Ok(DoomSegClassicContextPresentation {
        plane_meshes: planes.grouped_meshes,
        plane_triangles: planes.triangles,
        wall_meshes,
        omitted_wall_triangles,
        draws,
    })
}
