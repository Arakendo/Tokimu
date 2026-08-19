use std::{
    collections::{BTreeMap, BTreeSet},
    io,
    time::Instant,
};

use doom_geometry_provider::{
    locate_doom_point_subsector, lower_doom_seg_textured_wall_triangles,
    observe_doom_seg_plane_marks, resolve_doom_linedef_subsector_membership,
    resolve_doom_subsector_bsp_paths, resolve_doom_subsector_regions, DoomSegClassicPlaneInstance,
    DoomSegClassicPlaneKey, DoomSegClassicPlaneKind, DoomSegClassicPlaneSpanObservation,
    DoomSubsectorBspPath, DoomSurfacePlane, DoomTextureExtent,
};
use doom_map_provider::DoomMapCore;
use hello_doom_e1m1::{
    observer_right, DoomComparativeEmbedding, StaticDrawPlanEntry, StaticDrawSource,
};
use tokimu::PlatformResult;
use tokimu_core::math::Vec3;

use crate::{
    bsp_diagnostic_hit, compact_draw_source, format_ordered_occurrence_domain_trace,
    nearest_mesh_ray_hit, observe_bsp_diagnostic_manifest_at_source, observe_doom_seg_classic_bsp,
    observe_shared_doom_classic_vertical_clip_state, BspDiagnosticDisposition, BspDiagnosticDraw,
    BspDiagnosticFamily, BspDiagnosticManifest, BspDiagnosticReason, DoomSkyBoundaryDepthDraw,
    OrderedOccurrenceTraceTarget, OrderedPlaneKind, SceneInput,
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct ClassicPlaneSpanSupport {
    instances: usize,
    populated_columns: usize,
    populated_cells: usize,
    source_segs: usize,
}

pub(crate) const DEFAULT_SCAN_COLUMNS: usize = 32;
pub(crate) const DEFAULT_SCAN_ROWS: usize = 20;
pub(crate) const MAX_SCAN_SAMPLES: usize = 4_096;
pub(crate) const MAX_SCAN_GROUPS_REPORTED: usize = 24;

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct SourceLookRay {
    origin: [f32; 3],
    direction: [f32; 3],
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct SourceViewportScan {
    origin: [f32; 3],
    center_direction: [f32; 3],
    size: [f32; 2],
    columns: usize,
    rows: usize,
}

#[derive(Clone, Debug)]
pub(crate) struct BspViewportScanGroup {
    pub(crate) diagnostic: BspDiagnosticDraw,
    pub(crate) source: String,
    pub(crate) samples: usize,
    pub(crate) minimum_pixel: [f32; 2],
    pub(crate) maximum_pixel: [f32; 2],
    pub(crate) representative_pixel: [f32; 2],
}

#[derive(Clone, Debug)]
pub(crate) struct BspViewportScanObservation {
    pub(crate) columns: usize,
    pub(crate) rows: usize,
    pub(crate) hits: usize,
    pub(crate) misses: usize,
    pub(crate) accepted: usize,
    pub(crate) rejected: usize,
    pub(crate) unresolved: usize,
    pub(crate) unavailable: usize,
    pub(crate) groups: Vec<BspViewportScanGroup>,
    pub(crate) elapsed_ms: f64,
}

impl BspViewportScanObservation {
    pub(crate) fn report(&self) -> String {
        let mut lines = vec![format!(
            "scan: frozen-view grid={}x{} samples={} hits={} misses={} accepted={} rejected={} unresolved={} unavailable={} suspicious-groups={} elapsed-ms={:.3} meaning=nearest-prepared-triangle-shadow-classification-not-rendered-pixel-parity",
            self.columns,
            self.rows,
            self.columns * self.rows,
            self.hits,
            self.misses,
            self.accepted,
            self.rejected,
            self.unresolved,
            self.unavailable,
            self.groups.len(),
            self.elapsed_ms,
        )];
        for group in self.groups.iter().take(MAX_SCAN_GROUPS_REPORTED) {
            lines.push(format!(
                "scan suspicious: family={} classification={} reason={} samples={} pixel-bounds=({:.1},{:.1})..({:.1},{:.1}) inspect=LOOK PIXEL {:.1} {:.1} source={}",
                group.diagnostic.family.label(),
                group.diagnostic.disposition.label(),
                group.diagnostic.reason.label(),
                group.samples,
                group.minimum_pixel[0],
                group.minimum_pixel[1],
                group.maximum_pixel[0],
                group.maximum_pixel[1],
                group.representative_pixel[0],
                group.representative_pixel[1],
                group.source,
            ));
        }
        if self.groups.len() > MAX_SCAN_GROUPS_REPORTED {
            lines.push(format!(
                "scan: omitted-suspicious-groups={} report-limit={MAX_SCAN_GROUPS_REPORTED}",
                self.groups.len() - MAX_SCAN_GROUPS_REPORTED
            ));
        }
        lines.join("\n")
    }
}

#[derive(Clone, Copy, Debug)]
pub(crate) struct PreparedRayHit<'a> {
    pub(crate) distance: f32,
    pub(crate) draw: &'a StaticDrawPlanEntry,
    pub(crate) family: &'static str,
}

#[derive(Clone, Copy, Debug)]
pub(crate) struct PreparedSkyBoundaryRayHit<'a> {
    distance: f32,
    draw: &'a DoomSkyBoundaryDepthDraw,
}

/// One retained intersection with an omitted source `F_SKY1` plane. This is
/// LOOK/headless evidence only: it distinguishes a ray leaving the source
/// world through a sky aperture from a ray that reaches ordinary geometry
/// without crossing sky. It does not make the diagnostic flat a renderer
/// mask or admit a general sky/portal contract.
#[derive(Clone, Copy, Debug)]
pub(crate) struct PreparedSourceSkyPlaneRayHit<'a> {
    distance: f32,
    draw: &'a StaticDrawPlanEntry,
}

pub(crate) fn nearest_prepared_ray_hit<'a>(
    origin: Vec3,
    direction: Vec3,
    opaque_draws: &'a [StaticDrawPlanEntry],
    cutout_draws: Option<&'a [StaticDrawPlanEntry]>,
) -> Option<PreparedRayHit<'a>> {
    let mut nearest = None;
    for (draw, family) in opaque_draws.iter().map(|draw| (draw, "opaque")).chain(
        cutout_draws
            .into_iter()
            .flat_map(|draws| draws.iter().map(|draw| (draw, "cutout"))),
    ) {
        for triangle in draw.mesh.positions.chunks_exact(3) {
            let Some(distance) = crate::ray_triangle_distance(
                origin,
                direction,
                Vec3::from_array(triangle[0]),
                Vec3::from_array(triangle[1]),
                Vec3::from_array(triangle[2]),
            ) else {
                continue;
            };
            if nearest.is_none_or(|hit: PreparedRayHit<'_>| distance < hit.distance) {
                nearest = Some(PreparedRayHit {
                    distance,
                    draw,
                    family,
                });
            }
        }
    }
    nearest
}

pub(crate) fn viewport_inspection_direction(
    center_direction: Vec3,
    size: [f32; 2],
    ndc: [f32; 2],
) -> Vec3 {
    let forward = center_direction.normalize_or_zero();
    let right = observer_right(forward);
    let up = right.cross(forward).normalize_or_zero();
    let vertical_tangent = (30.0_f32.to_radians()).tan();
    let aspect = size[0] / size[1].max(1.0);
    (forward + right * (ndc[0] * aspect * vertical_tangent) + up * (ndc[1] * vertical_tangent))
        .normalize_or_zero()
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn scan_bsp_viewport(
    origin: Vec3,
    center_direction: Vec3,
    size: [f32; 2],
    columns: usize,
    rows: usize,
    opaque_draws: &[StaticDrawPlanEntry],
    cutout_draws: &[StaticDrawPlanEntry],
    include_cutouts: bool,
    manifest: &BspDiagnosticManifest,
) -> BspViewportScanObservation {
    let started = Instant::now();
    let mut hits = 0;
    let mut misses = 0;
    let mut accepted = 0;
    let mut rejected = 0;
    let mut unresolved = 0;
    let mut unavailable = 0;
    let mut groups = BTreeMap::<
        (
            BspDiagnosticDisposition,
            BspDiagnosticFamily,
            BspDiagnosticReason,
            String,
        ),
        BspViewportScanGroup,
    >::new();
    for row in 0..rows {
        for column in 0..columns {
            let pixel = [
                (column as f32 + 0.5) * size[0] / columns as f32,
                (row as f32 + 0.5) * size[1] / rows as f32,
            ];
            let ndc = [
                2.0 * pixel[0] / size[0] - 1.0,
                1.0 - 2.0 * pixel[1] / size[1],
            ];
            let direction = viewport_inspection_direction(center_direction, size, ndc);
            let Some(hit) = nearest_prepared_ray_hit(
                origin,
                direction,
                opaque_draws,
                include_cutouts.then_some(cutout_draws),
            ) else {
                misses += 1;
                continue;
            };
            hits += 1;
            let Some(diagnostic) = bsp_diagnostic_hit(
                manifest,
                hit.draw,
                hit.family == "cutout",
                opaque_draws,
                cutout_draws,
            ) else {
                unavailable += 1;
                continue;
            };
            match diagnostic.disposition {
                BspDiagnosticDisposition::Accepted => {
                    accepted += 1;
                    continue;
                }
                BspDiagnosticDisposition::RejectedSolidRange
                | BspDiagnosticDisposition::RejectedOutsideFrustum => rejected += 1,
                BspDiagnosticDisposition::UnresolvedFailOpen => unresolved += 1,
            }
            let source = compact_draw_source(&hit.draw.source);
            let key = (
                diagnostic.disposition,
                diagnostic.family,
                diagnostic.reason,
                source.clone(),
            );
            let group = groups.entry(key).or_insert(BspViewportScanGroup {
                diagnostic,
                source,
                samples: 0,
                minimum_pixel: pixel,
                maximum_pixel: pixel,
                representative_pixel: pixel,
            });
            group.samples += 1;
            group.minimum_pixel[0] = group.minimum_pixel[0].min(pixel[0]);
            group.minimum_pixel[1] = group.minimum_pixel[1].min(pixel[1]);
            group.maximum_pixel[0] = group.maximum_pixel[0].max(pixel[0]);
            group.maximum_pixel[1] = group.maximum_pixel[1].max(pixel[1]);
        }
    }
    let mut groups = groups.into_values().collect::<Vec<_>>();
    groups.sort_by(|left, right| {
        let severity = |disposition| match disposition {
            BspDiagnosticDisposition::RejectedSolidRange
            | BspDiagnosticDisposition::RejectedOutsideFrustum => 0,
            BspDiagnosticDisposition::UnresolvedFailOpen => 1,
            BspDiagnosticDisposition::Accepted => 2,
        };
        severity(left.diagnostic.disposition)
            .cmp(&severity(right.diagnostic.disposition))
            .then_with(|| right.samples.cmp(&left.samples))
            .then_with(|| left.source.cmp(&right.source))
    });
    BspViewportScanObservation {
        columns,
        rows,
        hits,
        misses,
        accepted,
        rejected,
        unresolved,
        unavailable,
        groups,
        elapsed_ms: started.elapsed().as_secs_f64() * 1_000.0,
    }
}

pub(crate) fn format_look_ray_observation(
    world_origin: Vec3,
    world_direction: Vec3,
    embedding: DoomComparativeEmbedding,
    hit: Option<PreparedRayHit<'_>>,
    sky_boundary_hit: Option<PreparedSkyBoundaryRayHit<'_>>,
    source_sky_plane_hit: Option<PreparedSourceSkyPlaneRayHit<'_>>,
) -> String {
    let (source_xy, source_z) = embedding.lower_direction(world_origin);
    let (source_direction_xy, source_direction_z) = embedding.lower_direction(world_direction);
    let replay = format!(
        "source_xyz=({:.3},{:.3},{:.3}) source_direction=({:.6},{:.6},{:.6}) world_xyz=({:.3},{:.3},{:.3}) world_direction=({:.6},{:.6},{:.6}) replay=--look-ray-report={:.9},{:.9},{:.9},{:.9},{:.9},{:.9}",
        source_xy[0], source_xy[1], source_z,
        source_direction_xy[0], source_direction_xy[1], source_direction_z,
        world_origin.x, world_origin.y, world_origin.z,
        world_direction.x, world_direction.y, world_direction.z,
        source_xy[0], source_xy[1], source_z,
        source_direction_xy[0], source_direction_xy[1], source_direction_z,
    );
    let ordinary = hit.map_or_else(
        || format!("look: no prepared triangle intersects ray; {replay}"),
        |hit| {
            let world_hit = world_origin + world_direction * hit.distance;
            let (source_hit_xy, source_hit_z) = embedding.lower_direction(world_hit);
            format!(
                "look: exact prepared-triangle hit distance={:.3} family={} material={} label={} source={} hit_source_xyz=({:.3},{:.3},{:.3}); {replay}",
                hit.distance,
                hit.family,
                hit.draw.material.0,
                hit.draw.source_label,
                compact_draw_source(&hit.draw.source),
                source_hit_xy[0], source_hit_xy[1], source_hit_z,
            )
        },
    );
    let boundary = sky_boundary_hit.map_or_else(
        || "sky_boundary=none".to_owned(),
        |boundary| {
            let relation = hit.map_or("no-ordinary-hit", |hit| {
                if boundary.distance < hit.distance {
                    "before-ordinary-hit"
                } else {
                    "behind-ordinary-hit"
                }
            });
            format!(
                "sky_boundary=distance:{:.3},linedef:{},sidedef:{},sector:{},relation:{relation}",
                boundary.distance,
                boundary.draw.source_linedef.record_index,
                boundary.draw.source_sidedef.record_index,
                boundary.draw.source_sector.record_index,
            )
        },
    );
    let sky_plane = source_sky_plane_hit.map_or_else(
        || "source_sky_plane=none".to_owned(),
        |sky| {
            let relation = hit.map_or("no-ordinary-hit", |hit| {
                if sky.distance < hit.distance {
                    "before-ordinary-hit"
                } else {
                    "behind-ordinary-hit"
                }
            });
            format!(
                "source_sky_plane=distance:{:.3},source:{},relation:{relation}",
                sky.distance,
                compact_draw_source(&sky.draw.source),
            )
        },
    );
    format!("{ordinary} {boundary} {sky_plane}")
}

/// Describes the corpus-private containment behavior applied to an exact
/// one-sided wall hit by the grouped-sky experiment. The wall's back remains
/// color-culled, but both faces terminate the opaque depth prepass so a ray
/// cannot escape through the back and re-enter through a later sky boundary.
pub(crate) fn format_one_sided_wall_boundary_observation(
    map: &DoomMapCore,
    origin: Vec3,
    hit: Option<PreparedRayHit<'_>>,
    presentation_enabled: bool,
) -> String {
    let Some(hit) = hit else {
        return "one_sided_wall_boundary=not-applicable:no-ordinary-hit".to_owned();
    };
    let StaticDrawSource::Wall { source_linedef, .. } = hit.draw.source else {
        return "one_sided_wall_boundary=not-applicable:non-wall-hit".to_owned();
    };
    let Some(linedef) = map
        .linedefs
        .get(source_linedef.record_index as usize)
        .filter(|linedef| linedef.source == source_linedef)
    else {
        return format!(
            "one_sided_wall_boundary=unavailable:linedef:{}",
            source_linedef.record_index
        );
    };
    if linedef.right_sidedef.is_some() == linedef.left_sidedef.is_some() {
        return "one_sided_wall_boundary=not-applicable:two-sided-wall-hit".to_owned();
    }
    let front_facing = crate::mesh_owning_side_visible(&hit.draw.mesh, origin);
    let facing = if front_facing { "front" } else { "back" };
    let color = if front_facing { "present" } else { "culled" };
    let parity_depth = if presentation_enabled {
        "terminating"
    } else {
        "shadow-only"
    };
    format!(
        "one_sided_wall_boundary=linedef:{},facing:{facing},color:{color},parity-depth:{parity_depth},parity-toggle:none authority=source-one-sidedness+prepared-wall-facing",
        source_linedef.record_index
    )
}

pub(crate) fn nearest_sky_boundary_ray_hit<'a>(
    origin: Vec3,
    direction: Vec3,
    draws: &'a [DoomSkyBoundaryDepthDraw],
) -> Option<PreparedSkyBoundaryRayHit<'a>> {
    draws
        .iter()
        .filter_map(|draw| {
            nearest_mesh_ray_hit(origin, direction, &draw.mesh)
                .map(|distance| PreparedSkyBoundaryRayHit { distance, draw })
        })
        .min_by(|left, right| left.distance.total_cmp(&right.distance))
}

/// Reports the paired-skywall and source-sky-plane crossings used by the
/// corpus-private grouped stencil parity experiment. Triangle hits sharing
/// one source surface are collapsed so triangulation seams do not count
/// twice. This predicts the low stencil bit from CPU geometry; it is not a
/// rendered pixel or stencil-buffer readback.
pub(crate) fn format_grouped_sky_parity_observation(
    origin: Vec3,
    direction: Vec3,
    ordinary_distance: Option<f32>,
    skywall_draws: &[DoomSkyBoundaryDepthDraw],
    sky_plane_draws: &[StaticDrawPlanEntry],
    presentation_enabled: bool,
) -> String {
    const CROSSING_EPSILON: f32 = 1.0e-3;
    let mut crossings = BTreeMap::<String, (&'static str, f32, usize)>::new();
    for draw in skywall_draws {
        let identity = format!(
            "linedef:{},sidedef:{},sector:{}",
            draw.source_linedef.record_index,
            draw.source_sidedef.record_index,
            draw.source_sector.record_index,
        );
        for triangle in draw.mesh.positions.chunks_exact(3) {
            let Some(distance) = crate::ray_triangle_distance(
                origin,
                direction,
                Vec3::from_array(triangle[0]),
                Vec3::from_array(triangle[1]),
                Vec3::from_array(triangle[2]),
            ) else {
                continue;
            };
            crossings
                .entry(identity.clone())
                .and_modify(|(_, nearest, raw_hits)| {
                    *nearest = nearest.min(distance);
                    *raw_hits += 1;
                })
                .or_insert(("skywall", distance, 1));
        }
    }
    for draw in sky_plane_draws {
        let StaticDrawSource::Flat {
            source_subsector,
            source_sector,
            plane,
        } = draw.source
        else {
            continue;
        };
        let plane = match plane {
            DoomSurfacePlane::Floor => "Floor",
            DoomSurfacePlane::Ceiling => "Ceiling",
        };
        let identity = format!(
            "subsector:{},sector:{},plane:{plane}",
            source_subsector.record_index, source_sector.record_index,
        );
        for triangle in draw.mesh.positions.chunks_exact(3) {
            let Some(distance) = crate::ray_triangle_distance(
                origin,
                direction,
                Vec3::from_array(triangle[0]),
                Vec3::from_array(triangle[1]),
                Vec3::from_array(triangle[2]),
            ) else {
                continue;
            };
            crossings
                .entry(identity.clone())
                .and_modify(|(_, nearest, raw_hits)| {
                    *nearest = nearest.min(distance);
                    *raw_hits += 1;
                })
                .or_insert(("sky-plane", distance, 1));
        }
    }
    let mut crossings = crossings.into_iter().collect::<Vec<_>>();
    crossings.sort_by(|left, right| {
        let left_distance = (left.1).1;
        let right_distance = (right.1).1;
        left_distance
            .total_cmp(&right_distance)
            .then_with(|| left.0.cmp(&right.0))
    });
    let ordered = crossings
        .iter()
        .map(|(identity, (family, distance, raw_hits))| {
            let relation = ordinary_distance.map_or("no-ordinary-hit", |ordinary_distance| {
                if *distance <= ordinary_distance + CROSSING_EPSILON {
                    "before-ordinary-hit"
                } else {
                    "behind-ordinary-hit"
                }
            });
            format!(
                "distance:{distance:.3},family:{family},{identity},raw-triangle-hits:{raw_hits},relation:{relation}"
            )
        })
        .collect::<Vec<_>>()
        .join(";");
    let skywall_crossings = crossings
        .iter()
        .filter(|(_, (family, _, _))| *family == "skywall")
        .count();
    let sky_plane_crossings = crossings.len() - skywall_crossings;
    let presentation = if presentation_enabled {
        "enabled"
    } else {
        "shadow-only"
    };
    let Some(ordinary_distance) = ordinary_distance else {
        return format!(
            "grouped_sky_parity=families:paired-skywalls+source-sky-planes,presentation:{presentation},ray-crossings={},skywall-crossings:{skywall_crossings},sky-plane-crossings:{sky_plane_crossings},crossings-before-ordinary:not-applicable,parity:not-applicable,rule-world-color:no-ordinary-fragment,crossings:[{ordered}] authority=cpu-ray-prediction-not-rendered-stencil-readback",
            crossings.len(),
        );
    };
    let before = crossings
        .iter()
        .filter(|(_, (_, distance, _))| *distance <= ordinary_distance + CROSSING_EPSILON)
        .count();
    let (parity, world_color) = if before % 2 == 0 {
        ("even", "retained")
    } else {
        ("odd", "masked")
    };
    format!(
        "grouped_sky_parity=families:paired-skywalls+source-sky-planes,presentation:{presentation},ray-crossings={},skywall-crossings:{skywall_crossings},sky-plane-crossings:{sky_plane_crossings},crossings-before-ordinary:{before},parity:{parity},rule-world-color:{world_color},crossings:[{ordered}] authority=cpu-ray-prediction-not-rendered-stencil-readback",
        crossings.len(),
    )
}

pub(crate) fn nearest_source_sky_plane_ray_hit<'a>(
    origin: Vec3,
    direction: Vec3,
    draws: &'a [StaticDrawPlanEntry],
) -> Option<PreparedSourceSkyPlaneRayHit<'a>> {
    draws
        .iter()
        .filter_map(|draw| {
            nearest_mesh_ray_hit(origin, direction, &draw.mesh)
                .map(|distance| PreparedSourceSkyPlaneRayHit { distance, draw })
        })
        .min_by(|left, right| left.distance.total_cmp(&right.distance))
}

pub(crate) fn format_source_classic_ray_trace(
    map: &DoomMapCore,
    source_origin: [f32; 2],
    source_direction: [f32; 2],
    hit: Option<PreparedRayHit<'_>>,
) -> String {
    let rounded = source_origin.map(|value| value.round());
    if rounded
        .iter()
        .any(|value| *value < f32::from(i16::MIN) || *value > f32::from(i16::MAX))
    {
        return "classic_source_trace=unavailable:viewer-outside-i16-source-domain".to_owned();
    }
    let viewer = [rounded[0] as i16, rounded[1] as i16];
    let paths = match resolve_doom_subsector_bsp_paths(map) {
        Ok(paths) => paths,
        Err(error) => return format!("classic_source_trace=unavailable:paths:{error}"),
    };
    let viewer_subsector = locate_doom_point_subsector(viewer, &paths)
        .map(|location| location.source_subsector.record_index.to_string())
        .unwrap_or_else(|_| String::from("ambiguous"));
    let mut target_subsectors = BTreeSet::new();
    let mut target_linedef = None;
    if let Some(hit) = hit {
        match hit.draw.source {
            StaticDrawSource::Flat {
                source_subsector, ..
            } => {
                if let Ok(index) = u16::try_from(source_subsector.record_index) {
                    target_subsectors.insert(index);
                }
            }
            StaticDrawSource::Wall { source_linedef, .. } => {
                target_linedef = Some(source_linedef.record_index);
                if let Some(membership) = resolve_doom_linedef_subsector_membership(map)
                    .into_iter()
                    .find(|entry| entry.source_linedef == source_linedef)
                {
                    target_subsectors.extend(
                        membership
                            .source_subsectors
                            .into_iter()
                            .filter_map(|source| u16::try_from(source.record_index).ok()),
                    );
                }
            }
        }
    }
    if source_direction[0].abs() <= f32::EPSILON && source_direction[1].abs() <= f32::EPSILON {
        return format!("classic_source_trace=viewer-subsector:{viewer_subsector},target-subsectors:{target_subsectors:?},unavailable:vertical-source-ray");
    }
    let heading = f64::from(source_direction[1]).atan2(f64::from(source_direction[0]));
    let observation = match observe_doom_seg_classic_bsp(map, viewer, heading, &target_subsectors) {
        Ok(observation) => observation,
        Err(error) => return format!("classic_source_trace=viewer-subsector:{viewer_subsector},target-subsectors:{target_subsectors:?},unavailable:bsp:{error}"),
    };
    let reached = target_subsectors
        .intersection(&observation.visited_subsectors)
        .copied()
        .collect::<Vec<_>>();
    let target_seg_records = target_linedef.map_or_else(Vec::new, |linedef| {
        map.segs
            .iter()
            .filter(|seg| u32::from(seg.linedef) == linedef)
            .map(|seg| seg.source.record_index)
            .collect::<Vec<_>>()
    });
    let admitted_target_segs = target_seg_records
        .iter()
        .filter(|record| observation.admitted_seg_records.contains(record))
        .copied()
        .collect::<Vec<_>>();
    let elision_geometry = format_elision_geometry(
        map,
        viewer,
        heading,
        hit.map(|hit| {
            [
                f64::from(source_origin[0])
                    + f64::from(source_direction[0]) * f64::from(hit.distance),
                f64::from(source_origin[1])
                    + f64::from(source_direction[1]) * f64::from(hit.distance),
            ]
        }),
        &observation.watched_subsector_elisions,
    );
    let target_geometry = format_target_subsector_geometry(
        map,
        &paths,
        &target_subsectors,
        hit.map(|hit| {
            [
                f64::from(source_origin[0])
                    + f64::from(source_direction[0]) * f64::from(hit.distance),
                f64::from(source_origin[1])
                    + f64::from(source_direction[1]) * f64::from(hit.distance),
            ]
        }),
    );
    format!(
        "classic_source_trace=viewer-subsector:{viewer_subsector},heading-degrees:{:.3},target-subsectors:{target_subsectors:?},reached:{reached:?},target-segs:{target_seg_records:?},admitted-target-segs:{admitted_target_segs:?},target-geometry:[{}],elisions:{},elision-geometry:[{}] meaning=doom-bsp-horizontal-source-protocol-not-pixel-parity",
        heading.to_degrees(),
        target_geometry.join("|"),
        observation.watched_subsector_elisions.join("|"),
        elision_geometry.join("|"),
    )
}

/// Reports whether Classic's frozen-view plane-span reconstruction contains
/// the hit plane's exact source key and sector. This is deliberately weaker
/// than pixel or prepared-mesh visibility: the source reconstruction is a
/// fixed 320x200 diagnostic domain and cannot be mapped honestly onto an
/// arbitrary pitched Tokimu viewport sample.
pub(crate) fn format_source_classic_plane_span_support(
    map: &DoomMapCore,
    wall_extents: &[DoomTextureExtent],
    source_origin: [f32; 2],
    source_center_direction: [f32; 2],
    eye_height: f32,
    hit: Option<PreparedRayHit<'_>>,
) -> String {
    let Some(hit) = hit else {
        return "classic_plane_occurrence=not-applicable:no-ordinary-hit".to_owned();
    };
    let StaticDrawSource::Flat {
        source_sector,
        plane,
        ..
    } = hit.draw.source
    else {
        return "classic_plane_occurrence=not-applicable:non-plane-hit".to_owned();
    };
    let rounded = source_origin.map(|value| value.round());
    if rounded
        .iter()
        .any(|value| *value < f32::from(i16::MIN) || *value > f32::from(i16::MAX))
    {
        return "classic_plane_occurrence=unavailable:viewer-outside-i16-source-domain".to_owned();
    }
    if source_center_direction[0].abs() <= f32::EPSILON
        && source_center_direction[1].abs() <= f32::EPSILON
    {
        return "classic_plane_occurrence=unavailable:vertical-frozen-view".to_owned();
    }
    let Some(sector) = map
        .sectors
        .iter()
        .find(|sector| sector.source == source_sector)
    else {
        return format!(
            "classic_plane_occurrence=unavailable:source-sector-{}-missing",
            source_sector.record_index
        );
    };
    let (kind, height, texture) = match plane {
        DoomSurfacePlane::Floor => (
            DoomSegClassicPlaneKind::Floor,
            sector.floor_height,
            sector.floor_texture.clone(),
        ),
        DoomSurfacePlane::Ceiling => (
            DoomSegClassicPlaneKind::Ceiling,
            sector.ceiling_height,
            sector.ceiling_texture.clone(),
        ),
    };
    let key = DoomSegClassicPlaneKey {
        kind,
        height,
        texture,
        light: sector.light_level,
    };
    let viewer = [rounded[0] as i16, rounded[1] as i16];
    let heading =
        f64::from(source_center_direction[1]).atan2(f64::from(source_center_direction[0]));
    let traversal = match observe_doom_seg_classic_bsp(map, viewer, heading, &BTreeSet::new()) {
        Ok(value) => value,
        Err(error) => return format!("classic_plane_occurrence=unavailable:bsp:{error}"),
    };
    let triangles = match lower_doom_seg_textured_wall_triangles(map, wall_extents) {
        Ok(value) => value,
        Err(error) => return format!("classic_plane_occurrence=unavailable:walls:{error}"),
    };
    let marks = match observe_doom_seg_plane_marks(map, eye_height as i16) {
        Ok(value) => value,
        Err(error) => return format!("classic_plane_occurrence=unavailable:marks:{error}"),
    };
    let vertical = observe_shared_doom_classic_vertical_clip_state(
        map,
        &triangles,
        &marks,
        &traversal,
        viewer,
        heading,
        f64::from(eye_height),
    );
    let support = summarize_classic_plane_span_support(
        &vertical.plane_spans,
        &key,
        source_sector.record_index,
    );
    let key_label = format!(
        "kind:{:?},height:{},flat:{},light:{},sector:{}",
        key.kind, key.height, key.texture, key.light, source_sector.record_index
    );
    match support {
        Some(support) => format!(
            "classic_plane_occurrence=source-key-present,{key_label},instances:{},populated-columns:{},populated-cells:{},source-segs:{} authority=source-key-frozen-view-occurrence-not-prepared-mesh-or-pixel-proof",
            support.instances,
            support.populated_columns,
            support.populated_cells,
            support.source_segs,
        ),
        None => format!(
            "classic_plane_occurrence=source-key-absent,{key_label} authority=diagnostic-absence-not-plane-rejection-proof"
        ),
    }
}

fn summarize_classic_plane_span_support(
    spans: &DoomSegClassicPlaneSpanObservation,
    key: &DoomSegClassicPlaneKey,
    source_sector: u32,
) -> Option<ClassicPlaneSpanSupport> {
    let matching = spans
        .keys
        .get(key)?
        .iter()
        .filter(|instance| instance.source_sectors.contains(&source_sector))
        .collect::<Vec<&DoomSegClassicPlaneInstance>>();
    if matching.is_empty() {
        return None;
    }
    let populated_columns = matching
        .iter()
        .map(|instance| instance.columns.iter().flatten().count())
        .sum();
    let populated_cells = matching
        .iter()
        .flat_map(|instance| instance.columns.iter().flatten())
        .map(|[top, bottom]| bottom - top + 1)
        .sum();
    let source_segs = matching
        .iter()
        .flat_map(|instance| instance.source_segs.iter().copied())
        .collect::<BTreeSet<_>>()
        .len();
    Some(ClassicPlaneSpanSupport {
        instances: matching.len(),
        populated_columns,
        populated_cells,
        source_segs,
    })
}

fn format_target_subsector_geometry(
    map: &DoomMapCore,
    paths: &[DoomSubsectorBspPath],
    targets: &BTreeSet<u16>,
    hit: Option<[f64; 2]>,
) -> Vec<String> {
    let regions = resolve_doom_subsector_regions(map, paths).ok();
    targets
        .iter()
        .filter_map(|target| {
            let subsector = map.subsectors.get(usize::from(*target))?;
            let first = usize::from(subsector.first_seg);
            let end = first + usize::from(subsector.seg_count);
            let mut left = i16::MAX;
            let mut right = i16::MIN;
            let mut bottom = i16::MAX;
            let mut top = i16::MIN;
            for seg in map.segs.get(first..end)? {
                for vertex_index in [seg.start_vertex, seg.end_vertex] {
                    let vertex = map.vertices.get(usize::from(vertex_index))?;
                    left = left.min(vertex.x);
                    right = right.max(vertex.x);
                    bottom = bottom.min(vertex.y);
                    top = top.max(vertex.y);
                }
            }
            let hit_inside = hit.map(|point| {
                point[0] >= f64::from(left)
                    && point[0] <= f64::from(right)
                    && point[1] >= f64::from(bottom)
                    && point[1] <= f64::from(top)
            });
            let region = regions
                .as_ref()
                .and_then(|regions| regions.get(usize::from(*target)));
            let region_bbox = region.and_then(|region| polygon_bounds(&region.vertices));
            let hit_inside_region = hit.zip(region).map(|(point, region)| {
                point_inside_convex_polygon(point, &region.vertices)
            });
            let hit_outside_seg_bbox = hit.map(|point| {
                let dx = if point[0] < f64::from(left) {
                    f64::from(left) - point[0]
                } else if point[0] > f64::from(right) {
                    point[0] - f64::from(right)
                } else {
                    0.0
                };
                let dy = if point[1] < f64::from(bottom) {
                    f64::from(bottom) - point[1]
                } else if point[1] > f64::from(top) {
                    point[1] - f64::from(top)
                } else {
                    0.0
                };
                dx.hypot(dy)
            });
            Some(format!(
                "subsector={target}:seg-count={}:seg-endpoint-bbox=[top:{top},bottom:{bottom},left:{left},right:{right}]:bsp-path-steps={}:inferred-region-vertices={}:inferred-region-bbox={region_bbox:?}:hit={hit:?}:hit-inside-seg-bbox={hit_inside:?}:hit-outside-seg-bbox-distance={hit_outside_seg_bbox:?}:hit-inside-inferred-region={hit_inside_region:?}",
                subsector.seg_count,
                paths.get(usize::from(*target)).map_or(0, |path| path.steps.len()),
                region.map_or(0, |region| region.vertices.len()),
            ))
        })
        .collect()
}

fn polygon_bounds(vertices: &[[f64; 2]]) -> Option<[[f64; 2]; 2]> {
    let first = *vertices.first()?;
    let [minimum, maximum] = vertices.iter().copied().fold(
        [first, first],
        |[[minimum_x, minimum_y], [maximum_x, maximum_y]], [x, y]| {
            [
                [minimum_x.min(x), minimum_y.min(y)],
                [maximum_x.max(x), maximum_y.max(y)],
            ]
        },
    );
    Some([minimum, maximum])
}

fn point_inside_convex_polygon(point: [f64; 2], vertices: &[[f64; 2]]) -> bool {
    const EPSILON: f64 = 1.0e-7;
    let mut observed_sign = 0.0_f64;
    for (start, end) in vertices
        .iter()
        .copied()
        .zip(vertices.iter().copied().cycle().skip(1))
        .take(vertices.len())
    {
        let cross = (end[0] - start[0]) * (point[1] - start[1])
            - (end[1] - start[1]) * (point[0] - start[0]);
        if cross.abs() <= EPSILON {
            continue;
        }
        if observed_sign == 0.0 {
            observed_sign = cross.signum();
        } else if cross.signum() != observed_sign {
            return false;
        }
    }
    observed_sign != 0.0
}

fn format_elision_geometry(
    map: &DoomMapCore,
    viewer: [i16; 2],
    heading: f64,
    hit: Option<[f64; 2]>,
    elisions: &[String],
) -> Vec<String> {
    let relative_bearing = |point: [f64; 2]| {
        let absolute = (point[1] - f64::from(viewer[1])).atan2(point[0] - f64::from(viewer[0]));
        let mut relative = absolute - heading;
        while relative > std::f64::consts::PI {
            relative -= std::f64::consts::TAU;
        }
        while relative < -std::f64::consts::PI {
            relative += std::f64::consts::TAU;
        }
        relative.to_degrees()
    };
    elisions
        .iter()
        .filter_map(|elision| {
            let node_index = elision
                .split(':')
                .find_map(|part| part.strip_prefix("node="))?
                .parse::<usize>()
                .ok()?;
            let node = map.nodes.get(node_index)?;
            let side = i64::from(node.delta_x) * i64::from(viewer[1] - node.y)
                - i64::from(node.delta_y) * i64::from(viewer[0] - node.x);
            let bbox = if side < 0 {
                node.left_bbox
            } else {
                node.right_bbox
            };
            let [top, bottom, left, right] = bbox;
            let corners = [
                [f64::from(left), f64::from(top)],
                [f64::from(right), f64::from(top)],
                [f64::from(right), f64::from(bottom)],
                [f64::from(left), f64::from(bottom)],
            ];
            let bearings = corners.map(relative_bearing);
            let hit_inside = hit.map(|point| {
                point[0] >= f64::from(left)
                    && point[0] <= f64::from(right)
                    && point[1] >= f64::from(bottom)
                    && point[1] <= f64::from(top)
            });
            Some(format!(
                "node={node_index}:far-bbox={bbox:?}:corner-bearings-degrees={bearings:?}:hit={hit:?}:hit-inside={hit_inside:?}:hit-bearing-degrees={:?}",
                hit.map(relative_bearing),
            ))
        })
        .collect()
}

pub(crate) fn parse_source_look_ray(value: &str) -> PlatformResult<SourceLookRay> {
    let values = value
        .split(',')
        .map(str::trim)
        .map(str::parse::<f32>)
        .collect::<Result<Vec<_>, _>>()
        .map_err(|error| io::Error::other(format!("invalid --look-ray-report value: {error}")))?;
    let [x, y, z, dx, dy, dz] = values.as_slice() else {
        return Err(io::Error::other(
            "--look-ray-report expects source x,y,z,direction-x,direction-y,direction-z",
        )
        .into());
    };
    let direction = Vec3::new(*dx, *dz, *dy);
    if !values.iter().all(|value| value.is_finite()) || direction.length_squared() <= f32::EPSILON {
        return Err(io::Error::other(
            "--look-ray-report requires finite values and a nonzero direction",
        )
        .into());
    }
    Ok(SourceLookRay {
        origin: [*x, *y, *z],
        direction: [*dx, *dy, *dz],
    })
}

pub(crate) fn parse_source_viewport_scan(value: &str) -> PlatformResult<SourceViewportScan> {
    let fields = value.split(',').map(str::trim).collect::<Vec<_>>();
    if !matches!(fields.len(), 8 | 10) {
        return Err(io::Error::other(
            "--bsp-diagnostic-scan-report expects source x,y,z,center-dx,center-dy,center-dz,width,height[,columns,rows]",
        )
        .into());
    }
    let values = fields[..8]
        .iter()
        .map(|value| value.parse::<f32>())
        .collect::<Result<Vec<_>, _>>()
        .map_err(|error| io::Error::other(format!("invalid scan camera value: {error}")))?;
    let columns = fields
        .get(8)
        .map_or(Ok(DEFAULT_SCAN_COLUMNS), |value| value.parse::<usize>())
        .map_err(|error| io::Error::other(format!("invalid scan column count: {error}")))?;
    let rows = fields
        .get(9)
        .map_or(Ok(DEFAULT_SCAN_ROWS), |value| value.parse::<usize>())
        .map_err(|error| io::Error::other(format!("invalid scan row count: {error}")))?;
    let origin = [values[0], values[1], values[2]];
    let center_direction = [values[3], values[4], values[5]];
    let size = [values[6], values[7]];
    if !values.iter().all(|value| value.is_finite())
        || Vec3::new(
            center_direction[0],
            center_direction[2],
            center_direction[1],
        )
        .length_squared()
            <= f32::EPSILON
        || size[0] <= 0.0
        || size[1] <= 0.0
    {
        return Err(io::Error::other(
            "scan camera requires finite values, a nonzero center direction, and positive dimensions",
        )
        .into());
    }
    if columns < 4
        || rows < 4
        || columns > 128
        || rows > 128
        || columns.saturating_mul(rows) > MAX_SCAN_SAMPLES
    {
        return Err(io::Error::other(format!(
            "scan grid axes must be 4..128 and contain at most {MAX_SCAN_SAMPLES} samples"
        ))
        .into());
    }
    Ok(SourceViewportScan {
        origin,
        center_direction,
        size,
        columns,
        rows,
    })
}

pub(crate) fn report_source_viewport_scan(
    scene: &SceneInput,
    embedding: DoomComparativeEmbedding,
    scan: SourceViewportScan,
    include_cutouts: bool,
) -> PlatformResult<()> {
    let origin = embedding.lift_direction([scan.origin[0], scan.origin[1]], scan.origin[2]);
    let center_direction = embedding
        .lift_direction(
            [scan.center_direction[0], scan.center_direction[1]],
            scan.center_direction[2],
        )
        .normalize_or_zero();
    let viewer = [
        scan.origin[0]
            .round()
            .clamp(f32::from(i16::MIN), f32::from(i16::MAX)) as i16,
        scan.origin[1]
            .round()
            .clamp(f32::from(i16::MIN), f32::from(i16::MAX)) as i16,
    ];
    if scan.origin[0] < f32::from(i16::MIN)
        || scan.origin[0] > f32::from(i16::MAX)
        || scan.origin[1] < f32::from(i16::MIN)
        || scan.origin[1] > f32::from(i16::MAX)
    {
        return Err(io::Error::other("scan viewer lies outside the i16 source domain").into());
    }
    let center_heading =
        f64::from(scan.center_direction[1]).atan2(f64::from(scan.center_direction[0]));
    let bounds_draws = scene
        .opaque_draws
        .iter()
        .chain(scene.cutout_draws.iter())
        .cloned()
        .collect::<Vec<_>>();
    let (_, radius) = crate::scene_bounds(&bounds_draws);
    let view =
        tokimu_core::math::try_view_look_at_rh(origin, origin + center_direction * 128.0, Vec3::Y)
            .ok_or_else(|| io::Error::other("headless scan camera view is degenerate"))?;
    let projection = tokimu_core::math::try_projection_perspective_rh_gl(
        60.0_f32.to_radians(),
        scan.size[0] / scan.size[1],
        (radius * 0.000_1).max(0.1),
        radius * 4.0,
    )
    .ok_or_else(|| io::Error::other("headless scan camera projection is invalid"))?;
    let manifest = observe_bsp_diagnostic_manifest_at_source(
        &scene.door_geometry_source.map,
        &scene.opaque_draws,
        &scene.cutout_draws,
        viewer,
        center_heading,
        Some(projection * view),
    )?;
    let observation = scan_bsp_viewport(
        origin,
        center_direction,
        scan.size,
        scan.columns,
        scan.rows,
        &scene.opaque_draws,
        &scene.cutout_draws,
        include_cutouts,
        &manifest,
    );
    println!(
        "headless_scan_view=source_xyz:({:.9},{:.9},{:.9}),center_direction:({:.9},{:.9},{:.9}),center_heading_degrees:{:.3},client:({:.0},{:.0}),runtime_heights:static-scene-snapshot",
        scan.origin[0],
        scan.origin[1],
        scan.origin[2],
        scan.center_direction[0],
        scan.center_direction[1],
        scan.center_direction[2],
        center_heading.to_degrees(),
        scan.size[0],
        scan.size[1],
    );
    println!("{}", observation.report());
    for (index, group) in observation
        .groups
        .iter()
        .take(MAX_SCAN_GROUPS_REPORTED)
        .enumerate()
    {
        let pixel = group.representative_pixel;
        let ndc = [
            2.0 * pixel[0] / scan.size[0] - 1.0,
            1.0 - 2.0 * pixel[1] / scan.size[1],
        ];
        let direction = viewport_inspection_direction(center_direction, scan.size, ndc);
        let hit = nearest_prepared_ray_hit(
            origin,
            direction,
            &scene.opaque_draws,
            include_cutouts.then_some(scene.cutout_draws.as_slice()),
        );
        let (sample_source_direction, _) = embedding.lower_direction(direction);
        let sample_heading =
            f64::from(sample_source_direction[1]).atan2(f64::from(sample_source_direction[0]));
        let mut heading_offset = (sample_heading - center_heading).to_degrees();
        while heading_offset > 180.0 {
            heading_offset -= 360.0;
        }
        while heading_offset < -180.0 {
            heading_offset += 360.0;
        }
        println!(
            "headless_scan_look={index},pixel:({:.1},{:.1}),ndc:({:.6},{:.6}),bsp-view-heading-degrees:{:.3},sample-ray-heading-degrees:{:.3},sample-minus-view-heading-degrees:{heading_offset:.3}",
            pixel[0],
            pixel[1],
            ndc[0],
            ndc[1],
            center_heading.to_degrees(),
            sample_heading.to_degrees(),
        );
        println!(
            "{}",
            format_look_ray_observation(
                origin,
                direction,
                embedding,
                hit,
                nearest_sky_boundary_ray_hit(origin, direction, &scene.doom_sky_boundary_draws),
                nearest_source_sky_plane_ray_hit(origin, direction, &scene.diagnostic_sky_draws,),
            )
        );
        println!(
            "{}",
            format_grouped_sky_parity_observation(
                origin,
                direction,
                hit.map(|hit| hit.distance),
                &scene.doom_sky_boundary_draws,
                &scene.diagnostic_sky_draws,
                false,
            )
        );
        println!(
            "{}",
            format_source_classic_ray_trace(
                &scene.door_geometry_source.map,
                [scan.origin[0], scan.origin[1]],
                sample_source_direction,
                hit,
            )
        );
        println!(
            "{}",
            format_source_classic_plane_span_support(
                &scene.door_geometry_source.map,
                &scene.door_geometry_source.wall_extents,
                [scan.origin[0], scan.origin[1]],
                [scan.center_direction[0], scan.center_direction[1]],
                scan.origin[2],
                hit,
            )
        );
        let classification = hit.map_or_else(
            || "bsp_shadow_classification=no-ordinary-hit".to_owned(),
            |hit| {
                let classification = crate::describe_bsp_diagnostic_hit(
                    &manifest,
                    hit.draw,
                    hit.family == "cutout",
                    &scene.opaque_draws,
                    &scene.cutout_draws,
                );
                format!(
                    "{classification},source:{}",
                    compact_draw_source(&hit.draw.source)
                )
            },
        );
        println!("{classification}");
    }
    Ok(())
}

pub(crate) fn report_source_look_ray(
    scene: &SceneInput,
    embedding: DoomComparativeEmbedding,
    ray: SourceLookRay,
    include_cutouts: bool,
    bsp_diagnostic: bool,
    skywall_parity_enabled: bool,
) {
    let origin = embedding.lift_direction([ray.origin[0], ray.origin[1]], ray.origin[2]);
    let direction = embedding
        .lift_direction([ray.direction[0], ray.direction[1]], ray.direction[2])
        .normalize_or_zero();
    let hit = nearest_prepared_ray_hit(
        origin,
        direction,
        &scene.opaque_draws,
        include_cutouts.then_some(scene.cutout_draws.as_slice()),
    );
    let rounded = [ray.origin[0].round(), ray.origin[1].round()];
    let ordered_domains = if rounded
        .iter()
        .chain(std::iter::once(&ray.origin[2]))
        .any(|value| *value < f32::from(i16::MIN) || *value > f32::from(i16::MAX))
    {
        String::from(
            "ordered_occurrence_domains=unavailable:viewer-or-eye-outside-i16-source-domain",
        )
    } else if ray.direction[0].abs() <= f32::EPSILON && ray.direction[1].abs() <= f32::EPSILON {
        String::from("ordered_occurrence_domains=unavailable:vertical-source-ray")
    } else {
        let trace_target = |draw: &StaticDrawPlanEntry| match draw.source {
            StaticDrawSource::Wall { source_linedef, .. } => OrderedOccurrenceTraceTarget::Wall {
                source_linedef: source_linedef.record_index,
            },
            StaticDrawSource::Flat {
                source_subsector,
                plane,
                ..
            } => OrderedOccurrenceTraceTarget::Plane {
                source_subsector: source_subsector.record_index,
                kind: match plane {
                    DoomSurfacePlane::Floor => OrderedPlaneKind::Floor,
                    DoomSurfacePlane::Ceiling => OrderedPlaneKind::Ceiling,
                },
            },
        };
        let candidate = hit.map(|hit| trace_target(hit.draw));
        let boundary_authority =
            nearest_sky_boundary_ray_hit(origin, direction, &scene.doom_sky_boundary_draws).map(
                |authority| OrderedOccurrenceTraceTarget::Wall {
                    source_linedef: authority.draw.source_linedef.record_index,
                },
            );
        let plane_authority =
            nearest_source_sky_plane_ray_hit(origin, direction, &scene.diagnostic_sky_draws)
                .map(|authority| trace_target(authority.draw));
        let authority = match (boundary_authority, plane_authority) {
            (Some(boundary), Some(plane)) => {
                let boundary_distance =
                    nearest_sky_boundary_ray_hit(origin, direction, &scene.doom_sky_boundary_draws)
                        .map(|hit| hit.distance)
                        .unwrap_or(f32::INFINITY);
                let plane_distance = nearest_source_sky_plane_ray_hit(
                    origin,
                    direction,
                    &scene.diagnostic_sky_draws,
                )
                .map(|hit| hit.distance)
                .unwrap_or(f32::INFINITY);
                Some(if boundary_distance <= plane_distance {
                    boundary
                } else {
                    plane
                })
            }
            (Some(boundary), None) => Some(boundary),
            (None, Some(plane)) => Some(plane),
            (None, None) => None,
        };
        let eye_height = ray.origin[2].round() as i16;
        format_ordered_occurrence_domain_trace(
            &scene.door_geometry_source.map,
            [rounded[0] as i16, rounded[1] as i16],
            f64::from(ray.direction[1]).atan2(f64::from(ray.direction[0])),
            eye_height,
            candidate,
            authority,
        )
    };
    let bsp_classification = if !bsp_diagnostic {
        "bsp_shadow_classification=disabled".to_owned()
    } else if rounded
        .iter()
        .any(|value| *value < f32::from(i16::MIN) || *value > f32::from(i16::MAX))
    {
        "bsp_shadow_classification=unavailable:viewer-outside-i16-source-domain".to_owned()
    } else if ray.direction[0].abs() <= f32::EPSILON && ray.direction[1].abs() <= f32::EPSILON {
        "bsp_shadow_classification=unavailable:vertical-source-ray".to_owned()
    } else {
        match observe_bsp_diagnostic_manifest_at_source(
            &scene.door_geometry_source.map,
            &scene.opaque_draws,
            &scene.cutout_draws,
            [rounded[0] as i16, rounded[1] as i16],
            f64::from(ray.direction[1]).atan2(f64::from(ray.direction[0])),
            None,
        ) {
            Ok(manifest) => hit.map_or_else(
                || "bsp_shadow_classification=no-ordinary-hit".to_owned(),
                |hit| {
                    let diagnostic = if hit.family == "cutout" {
                        scene
                            .cutout_draws
                            .iter()
                            .position(|draw| std::ptr::eq(draw, hit.draw))
                            .and_then(|index| manifest.cutouts.get(index))
                    } else {
                        scene
                            .opaque_draws
                            .iter()
                            .position(|draw| std::ptr::eq(draw, hit.draw))
                            .and_then(|index| manifest.opaque.get(index))
                    };
                    diagnostic.map_or_else(
                        || "bsp_shadow_classification=unavailable:hit-index".to_owned(),
                        |diagnostic| {
                            format!(
                                "bsp_shadow_classification=family:{},classification:{},reason:{},source:{}",
                                diagnostic.family.label(),
                                diagnostic.disposition.label(),
                                diagnostic.reason.label(),
                                compact_draw_source(&hit.draw.source),
                            )
                        },
                    )
                },
            ),
            Err(error) => format!("bsp_shadow_classification=unavailable:{error}"),
        }
    };
    println!(
        "{}\n{}\n{}\n{}\n{}\n{}\n{}",
        format_look_ray_observation(
            origin,
            direction,
            embedding,
            hit,
            nearest_sky_boundary_ray_hit(origin, direction, &scene.doom_sky_boundary_draws),
            nearest_source_sky_plane_ray_hit(origin, direction, &scene.diagnostic_sky_draws),
        ),
        format_grouped_sky_parity_observation(
            origin,
            direction,
            hit.map(|hit| hit.distance),
            &scene.doom_sky_boundary_draws,
            &scene.diagnostic_sky_draws,
            skywall_parity_enabled,
        ),
        format_one_sided_wall_boundary_observation(
            &scene.door_geometry_source.map,
            origin,
            hit,
            skywall_parity_enabled,
        ),
        format_source_classic_ray_trace(
            &scene.door_geometry_source.map,
            [ray.origin[0], ray.origin[1]],
            [ray.direction[0], ray.direction[1]],
            hit,
        ),
        format_source_classic_plane_span_support(
            &scene.door_geometry_source.map,
            &scene.door_geometry_source.wall_extents,
            [ray.origin[0], ray.origin[1]],
            [ray.direction[0], ray.direction[1]],
            ray.origin[2],
            hit,
        ),
        ordered_domains,
        bsp_classification,
    );
}

#[cfg(test)]
mod tests {
    use super::{
        format_grouped_sky_parity_observation, format_look_ray_observation, parse_source_look_ray,
        parse_source_viewport_scan, summarize_classic_plane_span_support,
        viewport_inspection_direction, DEFAULT_SCAN_COLUMNS, DEFAULT_SCAN_ROWS,
    };
    use crate::{
        DoomSkyBoundaryDepthDraw, MaterialHandle, Mesh, StaticDrawPlanEntry, StaticDrawSource,
    };
    use doom_geometry_provider::{
        DoomSegClassicPlaneInstance, DoomSegClassicPlaneKey, DoomSegClassicPlaneKind,
        DoomSegClassicPlaneSpanObservation, DoomSurfacePlane,
    };
    use doom_map_provider::DoomSourceRecord;
    use hello_doom_e1m1::DoomComparativeEmbedding;
    use std::collections::{BTreeMap, BTreeSet};
    use tokimu_core::math::Vec3;

    fn skywall_triangle(identity: u32, positions: [[f32; 3]; 3]) -> DoomSkyBoundaryDepthDraw {
        let source = |lump_index| DoomSourceRecord {
            lump_index,
            record_index: identity,
        };
        DoomSkyBoundaryDepthDraw {
            source_linedef: source(8),
            source_sidedef: source(9),
            source_sector: source(14),
            mesh: Mesh::uniform_normal(positions.to_vec(), [0.0, 1.0, 0.0]),
        }
    }

    fn sky_plane_triangle(identity: u32, distance: f32) -> StaticDrawPlanEntry {
        let source = |lump_index| DoomSourceRecord {
            lump_index,
            record_index: identity,
        };
        StaticDrawPlanEntry {
            mesh: Mesh::uniform_normal(
                vec![
                    [-1.0, -1.0, distance],
                    [1.0, -1.0, distance],
                    [0.0, 1.0, distance],
                ],
                [0.0, 0.0, -1.0],
            ),
            material: MaterialHandle(1),
            source_label: format!("sky-plane:{identity}"),
            source: StaticDrawSource::Flat {
                source_subsector: source(13),
                source_sector: source(14),
                plane: DoomSurfacePlane::Ceiling,
            },
        }
    }

    #[test]
    fn source_look_ray_parser_rejects_incomplete_and_stationary_rays() {
        let ray = parse_source_look_ray("1056,-3616,36,0,1,0").expect("source ray");
        assert_eq!(ray.origin, [1056.0, -3616.0, 36.0]);
        assert_eq!(ray.direction, [0.0, 1.0, 0.0]);
        assert!(parse_source_look_ray("1056,-3616,36,0,0").is_err());
        assert!(parse_source_look_ray("1056,-3616,36,0,0,0").is_err());
    }

    #[test]
    fn classic_plane_support_requires_both_exact_key_and_source_sector() {
        let key = DoomSegClassicPlaneKey {
            kind: DoomSegClassicPlaneKind::Floor,
            height: 0,
            texture: "FLOOR4_8".to_owned(),
            light: 160,
        };
        let instance = DoomSegClassicPlaneInstance {
            columns: vec![None, Some([10, 12]), Some([20, 20])],
            column_sources: vec![None, Some([38, 4]), Some([38, 5])],
            minimum_column: 1,
            maximum_column: 2,
            source_sectors: BTreeSet::from([38]),
            source_segs: BTreeSet::from([4, 5]),
        };
        let spans = DoomSegClassicPlaneSpanObservation {
            keys: BTreeMap::from([(key.clone(), vec![instance])]),
            ..Default::default()
        };

        assert_eq!(
            summarize_classic_plane_span_support(&spans, &key, 38),
            Some(super::ClassicPlaneSpanSupport {
                instances: 1,
                populated_columns: 2,
                populated_cells: 4,
                source_segs: 2,
            })
        );
        assert_eq!(summarize_classic_plane_span_support(&spans, &key, 29), None);
    }

    #[test]
    fn look_observation_retains_replayable_source_ray() {
        let embedding = DoomComparativeEmbedding::PreserveNorth;
        let source_origin = [1056.0, -3616.0, 36.0];
        let source_direction = [0.0, 1.0, 0.0];
        let world_origin =
            embedding.lift_direction([source_origin[0], source_origin[1]], source_origin[2]);
        let world_direction = embedding.lift_direction(
            [source_direction[0], source_direction[1]],
            source_direction[2],
        );
        let observation =
            format_look_ray_observation(world_origin, world_direction, embedding, None, None, None);
        assert!(observation.contains("source_xyz=(1056.000,-3616.000,36.000)"));
        assert!(observation.contains("source_direction=(0.000000,1.000000,0.000000)"));
        assert!(observation.contains("replay=--look-ray-report=1056.000000000,-3616.000000000,36.000000000,0.000000000,1.000000000,0.000000000"));
    }

    #[test]
    fn skywall_parity_collapses_quad_seams_and_counts_before_world() {
        let draws = vec![
            skywall_triangle(10, [[-1.0, -1.0, 2.0], [1.0, -1.0, 2.0], [1.0, 1.0, 2.0]]),
            skywall_triangle(10, [[-1.0, -1.0, 2.0], [1.0, 1.0, 2.0], [-1.0, 1.0, 2.0]]),
            skywall_triangle(20, [[-1.0, -1.0, 4.0], [1.0, -1.0, 4.0], [0.0, 1.0, 4.0]]),
        ];

        let even = format_grouped_sky_parity_observation(
            Vec3::ZERO,
            Vec3::Z,
            Some(5.0),
            &draws,
            &[],
            true,
        );
        assert!(even.contains("presentation:enabled,ray-crossings=2"));
        assert!(even.contains("crossings-before-ordinary:2,parity:even"));
        assert!(even.contains("rule-world-color:retained"));
        assert!(even.contains("linedef:10,sidedef:10,sector:10,raw-triangle-hits:2"));

        let odd = format_grouped_sky_parity_observation(
            Vec3::ZERO,
            Vec3::Z,
            Some(3.0),
            &draws,
            &[],
            false,
        );
        assert!(odd.contains("presentation:shadow-only,ray-crossings=2"));
        assert!(odd.contains("crossings-before-ordinary:1,parity:odd"));
        assert!(odd.contains("rule-world-color:masked"));
        assert!(odd.contains(
            "linedef:20,sidedef:20,sector:20,raw-triangle-hits:1,relation:behind-ordinary-hit"
        ));
    }

    #[test]
    fn grouped_sky_parity_counts_planes_and_walls_together() {
        let walls = vec![skywall_triangle(
            253,
            [[-1.0, -1.0, 2.0], [1.0, -1.0, 2.0], [0.0, 1.0, 2.0]],
        )];
        let planes = vec![sky_plane_triangle(48, 1.0)];

        let observation = format_grouped_sky_parity_observation(
            Vec3::ZERO,
            Vec3::Z,
            Some(3.0),
            &walls,
            &planes,
            true,
        );

        assert!(observation.contains("skywall-crossings:1,sky-plane-crossings:1"));
        assert!(observation.contains("crossings-before-ordinary:2,parity:even"));
        assert!(observation.contains("rule-world-color:retained"));
        assert!(observation.contains("family:sky-plane,subsector:48,sector:48,plane:Ceiling"));
        assert!(observation.contains("family:skywall,linedef:253"));
    }

    #[test]
    fn headless_scan_parser_defaults_and_bounds_the_grid() {
        let scan =
            parse_source_viewport_scan("10,20,36,0,1,0,1280,800").expect("default viewport scan");
        assert_eq!(scan.origin, [10.0, 20.0, 36.0]);
        assert_eq!(scan.center_direction, [0.0, 1.0, 0.0]);
        assert_eq!(scan.size, [1280.0, 800.0]);
        assert_eq!(scan.columns, DEFAULT_SCAN_COLUMNS);
        assert_eq!(scan.rows, DEFAULT_SCAN_ROWS);
        assert!(parse_source_viewport_scan("10,20,36,0,1,0,1280,800,128,128").is_err());
    }

    #[test]
    fn viewport_center_ray_preserves_the_supplied_camera_direction() {
        let center = Vec3::new(0.3, -0.2, -0.9).normalize();
        let ray = viewport_inspection_direction(center, [1280.0, 800.0], [0.0, 0.0]);
        assert!(ray.dot(center) > 0.999_999);
    }
}
