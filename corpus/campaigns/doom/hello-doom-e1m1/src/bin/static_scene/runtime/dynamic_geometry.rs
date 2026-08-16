//! Doom runtime-height projection and dynamic presentation identity helpers.
//!
//! The application owns activation and timing; these helpers apply explicit
//! current state to retained presentation inputs.

use super::super::*;

/// Corpus-local presentation lowering for one active manual-door ceiling flat.
/// Wall spans are re-lowered from retained source data rather than deformed in
/// place, preserving the distinction between height changes and texture-span
/// policy.
pub(crate) fn apply_door_ceiling_flat_height(
    draws: &mut [StaticDrawPlanEntry],
    target_sector: doom_map_provider::DoomSourceRecord,
    previous_height: i16,
    next_height: i16,
) -> Vec<usize> {
    apply_sector_flat_height(
        draws,
        target_sector,
        doom_geometry_provider::DoomSurfacePlane::Ceiling,
        previous_height,
        next_height,
    )
}

pub(crate) fn apply_sector_flat_height(
    draws: &mut [StaticDrawPlanEntry],
    target_sector: doom_map_provider::DoomSourceRecord,
    target_plane: doom_geometry_provider::DoomSurfacePlane,
    previous_height: i16,
    next_height: i16,
) -> Vec<usize> {
    let previous = f32::from(previous_height);
    let next = f32::from(next_height);
    let mut changed = Vec::new();
    for (index, draw) in draws.iter_mut().enumerate() {
        let is_target_flat = matches!(
            draw.source,
            StaticDrawSource::Flat {
                source_sector,
                plane,
                ..
            } if source_sector == target_sector && plane == target_plane
        );
        if !is_target_flat {
            continue;
        }
        let mut modified = false;
        for position in &mut draw.mesh.positions {
            if (position[1] - previous).abs() <= f32::EPSILON {
                position[1] = next;
                modified = true;
            }
        }
        if modified {
            changed.push(index);
        }
    }
    changed
}

pub(crate) fn carry_observer_with_floor(
    observer: Option<&mut SpawnObserver>,
    target_sector: doom_map_provider::DoomSourceRecord,
    previous_height: i16,
    next_height: i16,
) -> bool {
    let Some(observer) = observer else {
        return false;
    };
    if observer.sector != target_sector.record_index || observer.floor != previous_height {
        return false;
    }
    observer.position.y += f32::from(next_height - previous_height);
    observer.floor = next_height;
    true
}

pub(crate) fn dynamic_wall_triangle_key(
    source_linedef: doom_map_provider::DoomSourceRecord,
    source_sidedef: doom_map_provider::DoomSourceRecord,
    source_sector: doom_map_provider::DoomSourceRecord,
    role: doom_geometry_provider::DoomWallTextureRole,
    texture_name: &str,
) -> String {
    format!(
        "{}/{}/{}/{}/{}/{}/{:?}/{texture_name}",
        source_linedef.lump_index,
        source_linedef.record_index,
        source_sidedef.lump_index,
        source_sidedef.record_index,
        source_sector.lump_index,
        source_sector.record_index,
        role,
    )
}

pub(crate) fn static_wall_triangle_key(draw: &StaticDrawPlanEntry) -> Option<String> {
    let StaticDrawSource::Wall {
        source_linedef,
        source_sidedef,
        source_sector,
        role,
    } = draw.source
    else {
        return None;
    };
    let (_, texture_name) = draw.source_label.rsplit_once(':')?;
    Some(dynamic_wall_triangle_key(
        source_linedef,
        source_sidedef,
        source_sector,
        role,
        texture_name,
    ))
}

pub(crate) fn is_dynamic_mesh_for_target(
    draw: &StaticDrawPlanEntry,
    target_sector: doom_map_provider::DoomSourceRecord,
    target_plane: doom_geometry_provider::DoomSurfacePlane,
    boundary_linedefs: &[doom_map_provider::DoomSourceRecord],
) -> bool {
    match draw.source {
        StaticDrawSource::Flat {
            source_sector,
            plane,
            ..
        } => source_sector == target_sector && plane == target_plane,
        StaticDrawSource::Wall { source_sector, .. } if source_sector == target_sector => true,
        StaticDrawSource::Wall {
            source_linedef,
            role: doom_geometry_provider::DoomWallTextureRole::Upper,
            ..
        } if target_plane == doom_geometry_provider::DoomSurfacePlane::Ceiling => {
            boundary_linedefs.contains(&source_linedef)
        }
        StaticDrawSource::Wall {
            source_linedef,
            role: doom_geometry_provider::DoomWallTextureRole::Lower,
            ..
        } if target_plane == doom_geometry_provider::DoomSurfacePlane::Floor => {
            boundary_linedefs.contains(&source_linedef)
        }
        StaticDrawSource::Wall { .. } => false,
    }
}

pub(crate) fn is_door_mesh_for_target(
    draw: &StaticDrawPlanEntry,
    target_sector: doom_map_provider::DoomSourceRecord,
    boundary_linedefs: &[doom_map_provider::DoomSourceRecord],
) -> bool {
    is_dynamic_mesh_for_target(
        draw,
        target_sector,
        doom_geometry_provider::DoomSurfacePlane::Ceiling,
        boundary_linedefs,
    )
}

/// Returns the source linedefs which bound an active manual-door sector. The
/// result remains Doom-corpus evidence: the visual lowerer receives only these
/// retained identities, not a generalized portal or moving-wall contract.
pub(crate) fn manual_door_boundary_linedefs(
    source: &DoomLineActivationSource,
    target_sector: doom_map_provider::DoomSourceRecord,
) -> Vec<doom_map_provider::DoomSourceRecord> {
    source
        .linedefs
        .iter()
        .filter(|line| {
            [line.right_sidedef, line.left_sidedef]
                .into_iter()
                .flatten()
                .filter_map(|sidedef| source.sidedefs.get(usize::from(sidedef)))
                .filter_map(|sidedef| source.sectors.get(usize::from(sidedef.sector)))
                .any(|sector| sector.source == target_sector)
        })
        .map(|line| line.source)
        .collect()
}

/// Determines the source textures which become geometrically relevant at the
/// fully-open height of presently classified manual doors. This admits no
/// extra renderer behavior: it only makes their ordinary texture/material
/// inputs available before a runtime door can create the corresponding spans.
pub(crate) fn manual_door_dynamic_wall_texture_names(
    map: &DoomMapCore,
    source: &DoomLineActivationSource,
    extents: &[DoomTextureExtent],
) -> Result<Vec<String>, io::Error> {
    let mut open_map = map.clone();
    let mut targets = Vec::new();
    for line in &source.linedefs {
        let DoomLineActivationResolution::Accepted {
            intent: DoomLineActivationIntent::RaiseDoor { target_sector },
            ..
        } = resolve_doom_line_activation(
            source,
            DoomLineActivationRequest {
                source_linedef: line.source,
                activation: DoomLineActivation::Use,
            },
        )
        else {
            continue;
        };
        if targets.contains(&target_sector) {
            continue;
        }
        let door = DoomManualDoorRuntime::start(
            source,
            target_sector,
            DoomManualDoorPolicy::CLASSIC_NORMAL,
        )
        .map_err(|error| io::Error::other(format!("manual-door preparation failed: {error:?}")))?;
        let sector = open_map
            .sectors
            .iter_mut()
            .find(|sector| sector.source == target_sector)
            .ok_or_else(|| io::Error::other("manual-door target disappeared from decoded map"))?;
        sector.ceiling_height = door.open_ceiling_height;
        targets.push(target_sector);
    }

    let triangles = lower_doom_textured_wall_triangles(&open_map, extents).map_err(|error| {
        io::Error::other(format!("manual-door span preparation failed: {error}"))
    })?;
    let mut names = triangles
        .into_iter()
        .filter(|triangle| {
            targets.iter().any(|target_sector| {
                triangle.source_sector == *target_sector
                    || (triangle.role == doom_geometry_provider::DoomWallTextureRole::Upper
                        && manual_door_boundary_linedefs(source, *target_sector)
                            .contains(&triangle.source_linedef))
            })
        })
        .map(|triangle| triangle.texture_name)
        .collect::<Vec<_>>();
    names.sort();
    names.dedup();
    Ok(names)
}
