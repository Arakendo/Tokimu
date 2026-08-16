//! Viewer-relative Doom sky-span reconstruction and lowering.

use super::super::*;

/// Reconstructs only the current Doom-owned sky ceiling screen cells on their
/// source ceiling planes. Unlike the falsified source-sector control, these
/// cells do not grant an entire retained subsector flat depth authority merely
/// because one part of its sector contributed a visible sky span. The mesh is
/// replaced as the observer moves and remains corpus falsification machinery,
/// not an admitted renderer stencil, portal, or sky contract.
pub(crate) fn prepare_viewer_relative_source_sky_span_mesh(
    source: &DoomDynamicDoorGeometrySource,
    observer: SpawnObserver,
    look: ObserverLook,
    embedding: DoomComparativeEmbedding,
) -> PlatformResult<(Option<Mesh>, usize)> {
    let (source_position, source_angle) = observer_doom_source_pose(observer, look, embedding);
    let traversal =
        observe_doom_seg_classic_bsp(&source.map, source_position, source_angle, &BTreeSet::new())?;
    let lowerable_triangles =
        lower_doom_seg_textured_wall_triangles(&source.map, &source.wall_extents)?;
    let plane_marks = observe_doom_seg_plane_marks(&source.map, observer.position.y as i16)?;
    let vertical = observe_shared_doom_classic_vertical_clip_state(
        &source.map,
        &lowerable_triangles,
        &plane_marks,
        &traversal,
        source_position,
        source_angle,
        f64::from(observer.position.y),
    );
    let reconstruction = reconstruct_doom_seg_classic_sky_cells(
        &vertical.plane_spans,
        &source.map,
        source_position,
        source_angle,
        f64::from(observer.position.y),
    );
    let mut positions = Vec::with_capacity(reconstruction.cells.len() * 6);
    for cell in &reconstruction.cells {
        let corners = cell.source_corners.map(|[x, y]| {
            embedding.lift_direction([x as f32, y as f32], f32::from(cell.source_height))
        });
        let desired_normal = -Vec3::Y;
        let mut indices = [0usize, 1, 2, 0, 2, 3];
        let normal = (corners[indices[1]] - corners[indices[0]])
            .cross(corners[indices[2]] - corners[indices[0]]);
        if normal.dot(desired_normal) < 0.0 {
            indices = [0, 2, 1, 0, 3, 2];
        }
        positions.extend(indices.map(|index| corners[index].to_array()));
    }
    let triangles = positions.len() / 3;
    Ok((
        (!positions.is_empty()).then(|| Mesh::uniform_normal(positions, (-Vec3::Y).to_array())),
        triangles,
    ))
}
