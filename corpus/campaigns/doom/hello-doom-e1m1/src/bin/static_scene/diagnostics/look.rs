use std::{collections::BTreeSet, io};

use doom_geometry_provider::{
    locate_doom_point_subsector, resolve_doom_linedef_subsector_membership,
    resolve_doom_subsector_bsp_paths, DoomSurfacePlane,
};
use doom_map_provider::DoomMapCore;
use hello_doom_e1m1::{DoomComparativeEmbedding, StaticDrawPlanEntry, StaticDrawSource};
use tokimu::PlatformResult;
use tokimu_core::math::Vec3;

use crate::{
    compact_draw_source, format_ordered_occurrence_domain_trace, nearest_mesh_ray_hit,
    observe_doom_seg_classic_bsp, DoomSkyBoundaryDepthDraw, OrderedOccurrenceTraceTarget,
    OrderedPlaneKind, SceneInput,
};

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct SourceLookRay {
    origin: [f32; 3],
    direction: [f32; 3],
}

#[derive(Clone, Copy, Debug)]
pub(crate) struct PreparedRayHit<'a> {
    distance: f32,
    draw: &'a StaticDrawPlanEntry,
    family: &'static str,
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
    format!(
        "classic_source_trace=viewer-subsector:{viewer_subsector},heading-degrees:{:.3},target-subsectors:{target_subsectors:?},reached:{reached:?},target-segs:{target_seg_records:?},admitted-target-segs:{admitted_target_segs:?},elisions:{} meaning=doom-bsp-horizontal-source-protocol-not-pixel-parity",
        heading.to_degrees(), observation.watched_subsector_elisions.join("|")
    )
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

pub(crate) fn report_source_look_ray(
    scene: &SceneInput,
    embedding: DoomComparativeEmbedding,
    ray: SourceLookRay,
    include_cutouts: bool,
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
    println!(
        "{}\n{}\n{}",
        format_look_ray_observation(
            origin,
            direction,
            embedding,
            hit,
            nearest_sky_boundary_ray_hit(origin, direction, &scene.doom_sky_boundary_draws),
            nearest_source_sky_plane_ray_hit(origin, direction, &scene.diagnostic_sky_draws),
        ),
        format_source_classic_ray_trace(
            &scene.door_geometry_source.map,
            [ray.origin[0], ray.origin[1]],
            [ray.direction[0], ray.direction[1]],
            hit,
        ),
        ordered_domains,
    );
}

#[cfg(test)]
mod tests {
    use super::{format_look_ray_observation, parse_source_look_ray};
    use hello_doom_e1m1::DoomComparativeEmbedding;

    #[test]
    fn source_look_ray_parser_rejects_incomplete_and_stationary_rays() {
        let ray = parse_source_look_ray("1056,-3616,36,0,1,0").expect("source ray");
        assert_eq!(ray.origin, [1056.0, -3616.0, 36.0]);
        assert_eq!(ray.direction, [0.0, 1.0, 0.0]);
        assert!(parse_source_look_ray("1056,-3616,36,0,0").is_err());
        assert!(parse_source_look_ray("1056,-3616,36,0,0,0").is_err());
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
}
