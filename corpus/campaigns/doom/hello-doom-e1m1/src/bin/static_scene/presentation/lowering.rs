//! Lowering from Doom-owned presentation observations into ordinary Tokimu
//! renderer declarations.
//!
//! The functions in this module consume already-prepared Doom meaning. They
//! do not select visibility, mutate runtime state, or teach `tokimu-render`
//! about Doom planes.

use std::{collections::BTreeMap, io};

use doom_geometry_provider::{
    DoomSegClassicPlaneKey, DoomSegClassicPlaneKind, DoomSurfacePlane, DoomWallTextureRole,
};
use doom_map_provider::{DoomMapCore, DoomSourceRecord};
use hello_doom_e1m1::{
    DoomComparativeEmbedding, StaticDrawPlanEntry, StaticDrawSource, StaticTextureSourceKind,
    StaticTextureUpload,
};
use tokimu::{Mesh, PlatformResult};
use tokimu_core::math::Vec3;

use super::model::{DoomSegClassicPlaneCellReconstruction, DoomSegClassicPlanePresentation};

pub(crate) fn doom_wall_role_key(role: DoomWallTextureRole) -> u8 {
    match role {
        DoomWallTextureRole::Upper => 0,
        DoomWallTextureRole::Lower => 1,
        DoomWallTextureRole::Middle => 2,
    }
}

/// Lowers retained Doom plane cells into ordinary renderer declarations.
pub(crate) fn lower_doom_seg_classic_plane_presentation(
    map: &DoomMapCore,
    opaque_uploads: &[StaticTextureUpload],
    embedding: DoomComparativeEmbedding,
    reconstruction: DoomSegClassicPlaneCellReconstruction,
) -> PlatformResult<DoomSegClassicPlanePresentation> {
    let mut subsector_by_seg = BTreeMap::new();
    for subsector in &map.subsectors {
        let start = usize::from(subsector.first_seg);
        let end = start + usize::from(subsector.seg_count);
        for seg in &map.segs[start..end] {
            subsector_by_seg.insert(seg.source.record_index, subsector.source);
        }
    }
    let flat_materials = opaque_uploads
        .iter()
        .filter(|upload| upload.source_kind == StaticTextureSourceKind::Flat)
        .map(|upload| (upload.source_name.as_str(), upload.material))
        .collect::<BTreeMap<_, _>>();
    let mut grouped =
        BTreeMap::<(DoomSegClassicPlaneKey, u32, u32), (DoomSourceRecord, Vec<[f32; 3]>)>::new();

    for cell in &reconstruction.cells {
        let source_subsector = *subsector_by_seg.get(&cell.source_seg).ok_or_else(|| {
            io::Error::other(format!(
                "classic plane cell SEG {} has no owning subsector",
                cell.source_seg
            ))
        })?;
        let positions = cell.source_corners.map(|[x, y]| {
            embedding.lift_direction([x as f32, y as f32], f32::from(cell.source_height))
        });
        let desired_normal = match cell.key.kind {
            DoomSegClassicPlaneKind::Floor => Vec3::Y,
            DoomSegClassicPlaneKind::Ceiling => -Vec3::Y,
        };
        let mut indices = [0usize, 1, 2, 0, 2, 3];
        let normal = (positions[indices[1]] - positions[indices[0]])
            .cross(positions[indices[2]] - positions[indices[0]]);
        if normal.dot(desired_normal) < 0.0 {
            indices = [0, 2, 1, 0, 3, 2];
        }
        let group = grouped
            .entry((
                cell.key.clone(),
                cell.source_sector,
                source_subsector.record_index,
            ))
            .or_insert_with(|| (source_subsector, Vec::new()));
        group
            .1
            .extend(indices.into_iter().map(|index| positions[index].to_array()));
    }

    let mut draws = Vec::with_capacity(grouped.len());
    let mut triangles = 0usize;
    for ((key, source_sector_index, _), (source_subsector, positions)) in grouped {
        let material = *flat_materials.get(key.texture.as_str()).ok_or_else(|| {
            io::Error::other(format!(
                "classic plane presentation flat `{}` has no material",
                key.texture
            ))
        })?;
        let source_sector = map
            .sectors
            .get(source_sector_index as usize)
            .ok_or_else(|| {
                io::Error::other(format!(
                    "classic plane presentation sector {source_sector_index} is absent"
                ))
            })?
            .source;
        let normal = match key.kind {
            DoomSegClassicPlaneKind::Floor => Vec3::Y,
            DoomSegClassicPlaneKind::Ceiling => -Vec3::Y,
        };
        let texture_coordinates = positions
            .iter()
            .map(|[x, _, z]| [*x / 64.0, -*z / 64.0])
            .collect::<Vec<_>>();
        triangles += positions.len() / 3;
        let mesh = Mesh::uniform_normal(positions, normal.to_array())
            .with_texture_coordinates(texture_coordinates)?;
        let plane = match key.kind {
            DoomSegClassicPlaneKind::Floor => DoomSurfacePlane::Floor,
            DoomSegClassicPlaneKind::Ceiling => DoomSurfacePlane::Ceiling,
        };
        draws.push(StaticDrawPlanEntry {
            source_label: format!(
                "classic-plane:{:?}:{}:{}:{}",
                key.kind, source_sector_index, source_subsector.record_index, key.texture
            ),
            source: StaticDrawSource::Flat {
                source_subsector,
                source_sector,
                plane,
            },
            mesh,
            material,
        });
    }

    Ok(DoomSegClassicPlanePresentation {
        source_cells: reconstruction.source_cells,
        grouped_meshes: draws.len(),
        triangles,
        draws,
    })
}
