//! Pure reconstruction helpers for Doom-owned viewer-relative presentation.
//!
//! These functions consume retained Doom-provider observations and explicit
//! fixed-view inputs. They do not inspect renderer state or application
//! lifecycle policy.

use std::collections::BTreeSet;

use doom_geometry_provider::{
    lower_doom_seg_textured_wall_triangles, observe_doom_classic_bsp,
    observe_doom_classic_bsp_without_solid_range_pruning, observe_doom_classic_vertical_clip_state,
    observe_doom_seg_plane_marks, reconstruct_doom_ordered_wall_fragments, DoomSegClassicPlaneKey,
    DoomSegClassicPlaneKind, DoomSegClassicPlaneSpanObservation, DoomTextureExtent,
};
use doom_map_provider::DoomMapCore;
use tokimu::PlatformResult;

use super::model::{
    DoomOrderedCoveragePreparation, DoomSegClassicPlaneCell, DoomSegClassicPlaneCellReconstruction,
};
use super::viewport::{
    classic_presentation_half_vertical_fov, CLASSIC_PRESENTATION_COLUMNS,
    CLASSIC_PRESENTATION_HALF_HORIZONTAL_FOV, CLASSIC_PRESENTATION_ROWS,
};

pub(crate) fn reconstruct_doom_seg_classic_plane_cells(
    spans: &DoomSegClassicPlaneSpanObservation,
    viewer: [i16; 2],
    heading: f64,
    eye_height: f64,
) -> DoomSegClassicPlaneCellReconstruction {
    reconstruct_doom_seg_classic_plane_cells_with_height(
        spans,
        viewer,
        heading,
        eye_height,
        |key, _| {
            (!(key.kind == DoomSegClassicPlaneKind::Ceiling && key.texture == "F_SKY1"))
                .then_some(key.height)
        },
    )
}

pub(crate) fn reconstruct_doom_seg_classic_sky_cells(
    spans: &DoomSegClassicPlaneSpanObservation,
    map: &DoomMapCore,
    viewer: [i16; 2],
    heading: f64,
    eye_height: f64,
) -> DoomSegClassicPlaneCellReconstruction {
    reconstruct_doom_seg_classic_plane_cells_with_height(
        spans,
        viewer,
        heading,
        eye_height,
        |key, source_sector| {
            (key.kind == DoomSegClassicPlaneKind::Ceiling && key.texture == "F_SKY1")
                .then(|| {
                    map.sectors
                        .get(source_sector as usize)
                        .map(|sector| sector.ceiling_height)
                })
                .flatten()
        },
    )
}

fn reconstruct_doom_seg_classic_plane_cells_with_height(
    spans: &DoomSegClassicPlaneSpanObservation,
    viewer: [i16; 2],
    heading: f64,
    eye_height: f64,
    source_height: impl Fn(&DoomSegClassicPlaneKey, u32) -> Option<i16>,
) -> DoomSegClassicPlaneCellReconstruction {
    const MINIMUM_TANGENT: f64 = 1.0e-9;
    const MINIMUM_QUAD_AREA: f64 = 1.0e-9;

    let half_vertical_fov = classic_presentation_half_vertical_fov();
    let forward = [heading.cos(), heading.sin()];
    let right = [-forward[1], forward[0]];
    let viewer = [f64::from(viewer[0]), f64::from(viewer[1])];
    let horizontal_angle = |column_boundary: f64| {
        let normalized = -1.0 + (column_boundary / CLASSIC_PRESENTATION_COLUMNS as f64) * 2.0;
        (normalized * CLASSIC_PRESENTATION_HALF_HORIZONTAL_FOV.tan()).atan()
    };
    let vertical_angle = |row_boundary: f64| {
        let normalized = 1.0 - (row_boundary / CLASSIC_PRESENTATION_ROWS as f64) * 2.0;
        (normalized * half_vertical_fov.tan()).atan()
    };
    let mut result = DoomSegClassicPlaneCellReconstruction::default();

    for (key, instances) in &spans.keys {
        for (instance_index, instance) in instances.iter().enumerate() {
            for (column, rows) in instance.columns.iter().enumerate() {
                let Some([top, bottom]) = rows else {
                    continue;
                };
                let [source_sector, source_seg] = instance.column_sources[column]
                    .expect("each retained plane column preserves its source owner");
                let Some(source_height) = source_height(key, source_sector) else {
                    continue;
                };
                let plane_delta = f64::from(source_height) - eye_height;
                result.source_cells += bottom - top + 1;
                let corners = [
                    (column as f64, *top as f64),
                    ((column + 1) as f64, *top as f64),
                    ((column + 1) as f64, (*bottom + 1) as f64),
                    (column as f64, (*bottom + 1) as f64),
                ];
                let mut points = [[0.0; 2]; 4];
                let mut rejected = None;
                for (point, (column_boundary, row_boundary)) in points.iter_mut().zip(corners) {
                    let elevation = vertical_angle(row_boundary);
                    let tangent = elevation.tan();
                    if tangent.abs() <= MINIMUM_TANGENT {
                        rejected = Some("horizon");
                        break;
                    }
                    let forward_distance = plane_delta / tangent;
                    if !forward_distance.is_finite() || forward_distance <= 0.0 {
                        rejected = Some("behind");
                        break;
                    }
                    let angle = horizontal_angle(column_boundary);
                    let radial_distance = forward_distance / angle.cos();
                    let ray = [
                        forward[0] * angle.cos() + right[0] * angle.sin(),
                        forward[1] * angle.cos() + right[1] * angle.sin(),
                    ];
                    *point = [
                        viewer[0] + ray[0] * radial_distance,
                        viewer[1] + ray[1] * radial_distance,
                    ];
                    result.maximum_source_distance =
                        result.maximum_source_distance.max(radial_distance);
                }
                match rejected {
                    Some("horizon") => {
                        result.horizon_rejections += 1;
                        continue;
                    }
                    Some("behind") => {
                        result.behind_viewer_rejections += 1;
                        continue;
                    }
                    Some(_) => unreachable!("bounded plane-cell rejection reason"),
                    None => {}
                }
                let twice_area = |a: [f64; 2], b: [f64; 2], c: [f64; 2]| {
                    (b[0] - a[0]) * (c[1] - a[1]) - (b[1] - a[1]) * (c[0] - a[0])
                };
                let area = twice_area(points[0], points[1], points[2]).abs()
                    + twice_area(points[0], points[2], points[3]).abs();
                if !area.is_finite() || area <= MINIMUM_QUAD_AREA {
                    result.degenerate_rejections += 1;
                    continue;
                }
                result.reconstructed_quads += 1;
                result.reconstructed_triangles += 2;
                result.cells.push(DoomSegClassicPlaneCell {
                    key: key.clone(),
                    source_height,
                    source_sector,
                    source_seg,
                    source_corners: points,
                });
                if result.samples.len() < 12 {
                    result.samples.push(format!(
                        "kind={:?} flat={} instance={} column={} rows={}..{} source-corners=[{:.2},{:.2}..{:.2},{:.2}]",
                        key.kind,
                        key.texture,
                        instance_index,
                        column,
                        top,
                        bottom,
                        points.iter().map(|point| point[0]).fold(f64::INFINITY, f64::min),
                        points.iter().map(|point| point[1]).fold(f64::INFINITY, f64::min),
                        points.iter().map(|point| point[0]).fold(f64::NEG_INFINITY, f64::max),
                        points.iter().map(|point| point[1]).fold(f64::NEG_INFINITY, f64::max),
                    ));
                }
            }
        }
    }
    result
}

pub(crate) fn count_plane_intervals(
    spans: &DoomSegClassicPlaneSpanObservation,
    predicate: impl Fn(&DoomSegClassicPlaneKey) -> bool,
) -> usize {
    spans
        .keys
        .iter()
        .filter(|(key, _)| predicate(key))
        .flat_map(|(_, instances)| instances)
        .flat_map(|instance| &instance.columns)
        .filter(|rows| rows.is_some())
        .count()
}

/// Produces one coherent fixed-view Doom preparation from explicit source,
/// texture-extent, and observer inputs. Application-owned state machines are
/// responsible for projecting their current runtime state into `map` before
/// calling this function.
pub(crate) fn prepare_doom_ordered_coverage(
    map: &DoomMapCore,
    wall_extents: &[DoomTextureExtent],
    viewer: [i16; 2],
    heading: f64,
    eye_height: f64,
    solid_range_pruning: bool,
) -> PlatformResult<DoomOrderedCoveragePreparation> {
    let traversal = if solid_range_pruning {
        observe_doom_classic_bsp(map, viewer, heading, &BTreeSet::new())?
    } else {
        observe_doom_classic_bsp_without_solid_range_pruning(
            map,
            viewer,
            heading,
            &BTreeSet::new(),
        )?
    };
    let source_triangles = lower_doom_seg_textured_wall_triangles(map, wall_extents)?;
    let plane_marks = observe_doom_seg_plane_marks(map, eye_height as i16)?;
    let vertical = observe_doom_classic_vertical_clip_state(
        map,
        &source_triangles,
        &plane_marks,
        &traversal,
        viewer,
        heading,
        eye_height,
    );
    let walls = reconstruct_doom_ordered_wall_fragments(
        map,
        &source_triangles,
        &vertical,
        viewer,
        heading,
        eye_height,
    );
    let planes = reconstruct_doom_seg_classic_plane_cells(
        &vertical.plane_spans,
        viewer,
        heading,
        eye_height,
    );
    let ordinary_plane_intervals = count_plane_intervals(&vertical.plane_spans, |key| {
        !(key.kind == DoomSegClassicPlaneKind::Ceiling && key.texture == "F_SKY1")
    });
    let sky_plane_intervals = count_plane_intervals(&vertical.plane_spans, |key| {
        key.kind == DoomSegClassicPlaneKind::Ceiling && key.texture == "F_SKY1"
    });

    Ok(DoomOrderedCoveragePreparation {
        traversal,
        vertical,
        walls,
        planes,
        ordinary_plane_intervals,
        sky_plane_intervals,
    })
}

#[cfg(test)]
mod tests {
    use std::collections::{BTreeMap, BTreeSet};

    use doom_geometry_provider::{
        DoomSegClassicPlaneInstance, DoomSegClassicPlaneKey, DoomSegClassicPlaneKind,
        DoomSegClassicPlaneSpanObservation,
    };

    use super::{
        classic_presentation_half_vertical_fov, reconstruct_doom_seg_classic_plane_cells,
        CLASSIC_PRESENTATION_COLUMNS, CLASSIC_PRESENTATION_HALF_HORIZONTAL_FOV,
        CLASSIC_PRESENTATION_ROWS,
    };

    #[test]
    fn plane_cell_reconstruction_produces_one_quad_per_populated_column() {
        let mut floor_columns = vec![None; 320];
        floor_columns[160] = Some([120, 130]);
        let mut floor_sources = vec![None; 320];
        floor_sources[160] = Some([1, 2]);
        let mut ceiling_columns = vec![None; 320];
        ceiling_columns[161] = Some([70, 80]);
        let mut ceiling_sources = vec![None; 320];
        ceiling_sources[161] = Some([1, 3]);
        let spans = DoomSegClassicPlaneSpanObservation {
            keys: BTreeMap::from([
                (
                    DoomSegClassicPlaneKey {
                        kind: DoomSegClassicPlaneKind::Floor,
                        height: 0,
                        texture: String::from("FLOOR4_8"),
                        light: 160,
                    },
                    vec![DoomSegClassicPlaneInstance {
                        columns: floor_columns,
                        column_sources: floor_sources,
                        minimum_column: 160,
                        maximum_column: 160,
                        source_sectors: BTreeSet::from([1]),
                        source_segs: BTreeSet::from([2]),
                    }],
                ),
                (
                    DoomSegClassicPlaneKey {
                        kind: DoomSegClassicPlaneKind::Ceiling,
                        height: 72,
                        texture: String::from("CEIL3_5"),
                        light: 160,
                    },
                    vec![DoomSegClassicPlaneInstance {
                        columns: ceiling_columns,
                        column_sources: ceiling_sources,
                        minimum_column: 161,
                        maximum_column: 161,
                        source_sectors: BTreeSet::from([1]),
                        source_segs: BTreeSet::from([3]),
                    }],
                ),
            ]),
            ..Default::default()
        };

        let reconstructed = reconstruct_doom_seg_classic_plane_cells(&spans, [0, 0], 0.0, 36.0);

        assert_eq!(reconstructed.source_cells, 22);
        assert_eq!(reconstructed.reconstructed_quads, 2);
        assert_eq!(reconstructed.reconstructed_triangles, 4);
        assert_eq!(reconstructed.horizon_rejections, 0);
        assert_eq!(reconstructed.behind_viewer_rejections, 0);
        assert_eq!(reconstructed.degenerate_rejections, 0);
        assert_eq!(reconstructed.cells.len(), 2);
        assert_eq!(reconstructed.cells[0].source_sector, 1);
        assert!(matches!(reconstructed.cells[0].source_seg, 2 | 3));
        assert!(reconstructed.cells.iter().all(|cell| cell
            .source_corners
            .iter()
            .flatten()
            .all(|value| value.is_finite())));
        assert!(reconstructed.maximum_source_distance.is_finite());
        assert!(reconstructed.maximum_source_distance > 0.0);
    }

    #[test]
    fn plane_cell_reconstruction_rejects_a_horizon_crossing_explicitly() {
        let mut columns = vec![None; 320];
        columns[160] = Some([100, 100]);
        let mut column_sources = vec![None; 320];
        column_sources[160] = Some([1, 2]);
        let spans = DoomSegClassicPlaneSpanObservation {
            keys: BTreeMap::from([(
                DoomSegClassicPlaneKey {
                    kind: DoomSegClassicPlaneKind::Floor,
                    height: 0,
                    texture: String::from("FLOOR4_8"),
                    light: 160,
                },
                vec![DoomSegClassicPlaneInstance {
                    columns,
                    column_sources,
                    minimum_column: 160,
                    maximum_column: 160,
                    source_sectors: BTreeSet::from([1]),
                    source_segs: BTreeSet::from([2]),
                }],
            )]),
            ..Default::default()
        };

        let reconstructed = reconstruct_doom_seg_classic_plane_cells(&spans, [0, 0], 0.0, 36.0);

        assert_eq!(reconstructed.source_cells, 1);
        assert_eq!(reconstructed.reconstructed_quads, 0);
        assert_eq!(reconstructed.horizon_rejections, 1);
        assert!(reconstructed.cells.is_empty());
    }

    #[test]
    fn off_center_plane_cell_round_trips_through_rectilinear_projection() {
        let mut columns = vec![None; CLASSIC_PRESENTATION_COLUMNS];
        columns[40] = Some([120, 130]);
        let mut column_sources = vec![None; CLASSIC_PRESENTATION_COLUMNS];
        column_sources[40] = Some([1, 2]);
        let spans = DoomSegClassicPlaneSpanObservation {
            keys: BTreeMap::from([(
                DoomSegClassicPlaneKey {
                    kind: DoomSegClassicPlaneKind::Floor,
                    height: 0,
                    texture: String::from("FLOOR4_8"),
                    light: 160,
                },
                vec![DoomSegClassicPlaneInstance {
                    columns,
                    column_sources,
                    minimum_column: 40,
                    maximum_column: 40,
                    source_sectors: BTreeSet::from([1]),
                    source_segs: BTreeSet::from([2]),
                }],
            )]),
            ..Default::default()
        };

        let reconstructed = reconstruct_doom_seg_classic_plane_cells(&spans, [0, 0], 0.0, 36.0);
        let cell = &reconstructed.cells[0];
        let half_vertical_tangent = classic_presentation_half_vertical_fov().tan();
        let expected = [(40.0, 120.0), (41.0, 120.0), (41.0, 131.0), (40.0, 131.0)];

        for (point, (expected_column, expected_row)) in cell.source_corners.iter().zip(expected) {
            let forward_depth = point[0];
            let lateral = point[1];
            let projected_column =
                ((lateral / forward_depth / CLASSIC_PRESENTATION_HALF_HORIZONTAL_FOV.tan() + 1.0)
                    * 0.5)
                    * CLASSIC_PRESENTATION_COLUMNS as f64;
            let projected_row = (1.0
                - (f64::from(cell.source_height) - 36.0) / forward_depth / half_vertical_tangent)
                * 0.5
                * CLASSIC_PRESENTATION_ROWS as f64;

            assert!((projected_column - expected_column).abs() < 1.0e-9);
            assert!((projected_row - expected_row).abs() < 1.0e-9);
        }
    }
}
