//! Shadow-only audit for the oriented sky-transition parity hypothesis.
//!
//! This module deliberately stops before parity when the source-derived sky
//! surfaces cannot prove a closed, oriented World/Sky transition system. It
//! never changes renderer submission.

use super::super::*;
use super::ordered_causality::{ordered_six_ray_cases, OrderedSixRayExpectedTarget};

const HIT_EPSILON: f32 = 1.0e-3;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum ExpectedState {
    World,
    Sky,
}

impl ExpectedState {
    const fn label(self) -> &'static str {
        match self {
            Self::World => "World",
            Self::Sky => "Sky",
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub(super) struct FrozenRay {
    pub(super) name: &'static str,
    pub(super) origin: [f64; 3],
    pub(super) direction: [f64; 3],
    pub(super) expected_label: &'static str,
    expected_state: ExpectedState,
}

const ORDINARY_HOLE_RAYS: [FrozenRay; 5] = [
    FrozenRay {
        name: "spawn-floor-103-hole",
        origin: [1056.0, -3616.0, 36.0],
        direction: [-0.063_943_826, 0.766_799_152, -0.638_694_227],
        expected_label: "flat:38:FLOOR4_8",
        expected_state: ExpectedState::World,
    },
    FrozenRay {
        name: "spawn-ceiling-103-hole",
        origin: [1056.0, -3616.0, 36.0],
        direction: [-0.058_917_262, 0.765_662_134, 0.640_539_050],
        expected_label: "flat:38:CEIL3_5",
        expected_state: ExpectedState::World,
    },
    FrozenRay {
        name: "sector-38-subsector-114-floor-hole",
        origin: [1011.078_369_141, -3023.246_826_172, 36.0],
        direction: [0.830_659_449, 0.459_405_005, -0.314_566_344],
        expected_label: "flat:38:FLOOR4_8",
        expected_state: ExpectedState::World,
    },
    FrozenRay {
        name: "sector-2-subsector-116-floor-hole",
        origin: [1286.139_648_438, -2552.103_515_625, 36.0],
        direction: [-0.152_104_899, -0.900_991_142, -0.406_299_144],
        expected_label: "flat:2:FLOOR4_8",
        expected_state: ExpectedState::World,
    },
    FrozenRay {
        name: "sector-12-subsector-29-floor-hole",
        origin: [1741.810_791_016, -2522.975_341_797, 36.0],
        direction: [0.860_694_408, -0.438_084_006, -0.259_398_639],
        expected_label: "flat:12:FLOOR4_8",
        expected_state: ExpectedState::World,
    },
];

#[derive(Clone, Debug)]
pub(super) struct CandidateTriangle {
    pub(super) identity: String,
    pub(super) family: &'static str,
    pub(super) vertices: [Vec3; 3],
}

#[derive(Clone, Debug)]
pub(super) struct RawHit {
    pub(super) identity: String,
    pub(super) family: &'static str,
    pub(super) distance: f32,
    pub(super) orientation: f32,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
struct ClosureAudit {
    triangles: usize,
    unique_edges: usize,
    manifold_edges: usize,
    open_edges: usize,
    non_manifold_edges: usize,
}

pub(crate) fn report_oriented_sky_transition_parity_shadow(
    scene: &SceneInput,
) -> PlatformResult<()> {
    let frozen = frozen_rays();
    let candidates = candidate_triangles(scene);
    let closure = audit_closure(&candidates);
    let paired_groups = scene
        .doom_sky_boundary_draws
        .iter()
        .map(|draw| draw.source_linedef.record_index)
        .collect::<BTreeSet<_>>();
    let plane_groups = scene
        .diagnostic_sky_draws
        .iter()
        .map(|draw| draw.source_label.as_str())
        .collect::<BTreeSet<_>>();
    let (paired_both_sky, paired_world_sky, paired_unresolved) =
        audit_paired_boundary_semantics(scene, &paired_groups);

    // Paired-sky height discontinuities have sky ceilings on both source
    // sides. Source-sky ceiling planes have a locally meaningful below/above
    // orientation, but their finite open surfaces do not establish a closed
    // sky domain or an Exit. The mandatory Slice 1 gate therefore fails.
    let proved_enters = 0usize;
    let proved_exits = 0usize;
    let transition_system_proved = proved_enters > 0
        && proved_exits > 0
        && closure.open_edges == 0
        && closure.non_manifold_edges == 0
        && paired_unresolved == 0;

    let mut rows = Vec::new();
    let mut world_counterfactual_matches = 0usize;
    let mut raw_presence_counterfactual_matches = 0usize;
    let mut raw_hits = 0usize;
    let mut semantic_groups = 0usize;
    let mut duplicate_collapses = 0usize;
    for ray in frozen {
        let (origin, direction) = source_ray_vectors(ray.origin, ray.direction);
        let ordinary = nearest_prepared_ray_hit(
            origin,
            direction,
            &scene.opaque_draws,
            Some(&scene.cutout_draws),
        )
        .ok_or_else(|| io::Error::other(format!("{} lost global-full target", ray.name)))?;
        if ordinary.draw.source_label != ray.expected_label {
            return Err(io::Error::other(format!(
                "{} expected global-full target {}, got {}",
                ray.name, ray.expected_label, ordinary.draw.source_label,
            ))
            .into());
        }
        let hits = candidate_hits_before(&candidates, origin, direction, ordinary.distance);
        let groups = collapse_hits(&hits);
        raw_hits += hits.len();
        semantic_groups += groups.len();
        duplicate_collapses += hits.len().saturating_sub(groups.len());
        // Slice 2 parity is not executed after the failed transition-system
        // gate. World is shown only as the mandated fail-open counterfactual.
        let fail_open_state = ExpectedState::World;
        world_counterfactual_matches += usize::from(fail_open_state == ray.expected_state);
        let raw_presence_state = if groups.is_empty() {
            ExpectedState::World
        } else {
            ExpectedState::Sky
        };
        raw_presence_counterfactual_matches +=
            usize::from(raw_presence_state == ray.expected_state);
        rows.push(format!(
            "case={}:target={}:distance={:.3}:expected={}:raw-hits={}:groups={}:observations=[{}]:proved-events=0:parity-executed=false:fail-open-state=World:counterfactual-match={}",
            ray.name,
            ordinary.draw.source_label,
            ordinary.distance,
            ray.expected_state.label(),
            hits.len(),
            groups.len(),
            format_groups(&groups),
            fail_open_state == ray.expected_state,
        ));
    }

    let conservation = candidates.len()
        == scene.doom_sky_boundary_draws.len()
            + scene
                .diagnostic_sky_draws
                .iter()
                .map(|draw| draw.mesh.positions.len() / 3)
                .sum::<usize>()
        && raw_hits == semantic_groups + duplicate_collapses;
    if !conservation {
        return Err(io::Error::other("sky-transition shadow conservation failed").into());
    }
    let fingerprint = fingerprint_report(
        &rows,
        &candidates,
        closure,
        paired_both_sky,
        paired_world_sky,
        paired_unresolved,
    );
    println!(
        "E1M1 oriented sky-transition parity shadow Slices 0-1: frozen-rays={}; expected=[Sky:5,World:5]; candidate-triangles={}; paired-triangles={}; paired-groups={}; paired-both-sky={paired_both_sky}; paired-world-sky={paired_world_sky}; paired-unresolved={paired_unresolved}; source-sky-plane-triangles={}; source-sky-plane-groups={}; closure=[unique-edges:{},manifold:{},open:{},non-manifold:{}]; proved-enter={proved_enters}; proved-exit={proved_exits}; transition-system-proved={transition_system_proved}; slice-2-parity-executed=false; fail-open-counterfactual={world_counterfactual_matches}/10; raw-any-hit-counterfactual={raw_presence_counterfactual_matches}/10:authority=correlation-only-not-semantic-transition; raw-ray-hits={raw_hits}; semantic-hit-groups={semantic_groups}; duplicate-collapses={duplicate_collapses}; conservation=balanced; renderer-mutation=false; fingerprint={fingerprint:016x}; disposition=park-real-parity-source-semantics-provide-open-one-way-sky-surfaces-not-closed-world-sky-volume; rows=[{}]",
        rows.len(),
        candidates.len(),
        scene.doom_sky_boundary_draws.len(),
        paired_groups.len(),
        scene
            .diagnostic_sky_draws
            .iter()
            .map(|draw| draw.mesh.positions.len() / 3)
            .sum::<usize>(),
        plane_groups.len(),
        closure.unique_edges,
        closure.manifold_edges,
        closure.open_edges,
        closure.non_manifold_edges,
        rows.join(" | "),
    );
    Ok(())
}

pub(super) fn frozen_rays() -> Vec<FrozenRay> {
    let mut rays = ordered_six_ray_cases()
        .iter()
        .filter(|case| {
            !matches!(
                case.expected,
                OrderedSixRayExpectedTarget::PartialPlane { .. }
            )
        })
        .map(|case| FrozenRay {
            name: case.name,
            origin: case.origin,
            direction: case.direction,
            expected_label: case.expected_global_label,
            expected_state: ExpectedState::Sky,
        })
        .collect::<Vec<_>>();
    rays.extend(ORDINARY_HOLE_RAYS);
    rays
}

pub(super) fn candidate_triangles(scene: &SceneInput) -> Vec<CandidateTriangle> {
    let mut triangles = scene
        .doom_sky_boundary_draws
        .iter()
        .map(|draw| CandidateTriangle {
            identity: format!("paired-linedef:{}", draw.source_linedef.record_index),
            family: "paired-sky-height-discontinuity",
            vertices: draw.mesh.positions[..3]
                .iter()
                .map(|position| Vec3::from_array(*position))
                .collect::<Vec<_>>()
                .try_into()
                .expect("paired sky boundary is one triangle"),
        })
        .collect::<Vec<_>>();
    for draw in &scene.diagnostic_sky_draws {
        for positions in draw.mesh.positions.chunks_exact(3) {
            triangles.push(CandidateTriangle {
                identity: draw.source_label.clone(),
                family: "source-sky-open-plane",
                vertices: positions
                    .iter()
                    .map(|position| Vec3::from_array(*position))
                    .collect::<Vec<_>>()
                    .try_into()
                    .expect("triangle chunk has three positions"),
            });
        }
    }
    triangles
}

fn audit_paired_boundary_semantics(
    scene: &SceneInput,
    groups: &BTreeSet<u32>,
) -> (usize, usize, usize) {
    let map = &scene.door_geometry_source.map;
    let mut both_sky = 0usize;
    let mut world_sky = 0usize;
    let mut unresolved = 0usize;
    for source_linedef in groups {
        let Some(linedef) = map
            .linedefs
            .iter()
            .find(|linedef| linedef.source.record_index == *source_linedef)
        else {
            unresolved += 1;
            continue;
        };
        let Some((right, left)) =
            linedef
                .right_sidedef
                .zip(linedef.left_sidedef)
                .and_then(|(right, left)| {
                    Some((
                        map.sidedefs.get(usize::from(right))?,
                        map.sidedefs.get(usize::from(left))?,
                    ))
                })
        else {
            unresolved += 1;
            continue;
        };
        let Some((right_sector, left_sector)) = map
            .sectors
            .get(usize::from(right.sector))
            .zip(map.sectors.get(usize::from(left.sector)))
        else {
            unresolved += 1;
            continue;
        };
        let right_sky = right_sector.ceiling_texture == "F_SKY1";
        let left_sky = left_sector.ceiling_texture == "F_SKY1";
        if right_sky && left_sky {
            both_sky += 1;
        } else if right_sky != left_sky {
            world_sky += 1;
        } else {
            unresolved += 1;
        }
    }
    (both_sky, world_sky, unresolved)
}

pub(super) fn candidate_hits_before(
    candidates: &[CandidateTriangle],
    origin: Vec3,
    direction: Vec3,
    ordinary_distance: f32,
) -> Vec<RawHit> {
    let mut hits = candidates
        .iter()
        .filter_map(|triangle| {
            let distance = crate::ray_triangle_distance(
                origin,
                direction,
                triangle.vertices[0],
                triangle.vertices[1],
                triangle.vertices[2],
            )?;
            if distance >= ordinary_distance {
                return None;
            }
            let normal = (triangle.vertices[1] - triangle.vertices[0])
                .cross(triangle.vertices[2] - triangle.vertices[0])
                .normalize_or_zero();
            Some(RawHit {
                identity: triangle.identity.clone(),
                family: triangle.family,
                distance,
                orientation: direction.dot(normal),
            })
        })
        .collect::<Vec<_>>();
    hits.sort_by(|left, right| {
        left.distance
            .total_cmp(&right.distance)
            .then_with(|| left.identity.cmp(&right.identity))
    });
    hits
}

pub(super) fn collapse_hits(hits: &[RawHit]) -> Vec<Vec<RawHit>> {
    let mut groups: Vec<Vec<RawHit>> = Vec::new();
    for hit in hits {
        if let Some(group) = groups.iter_mut().find(|group| {
            group[0].identity == hit.identity
                && (group[0].distance - hit.distance).abs() <= HIT_EPSILON
                && group[0].orientation.signum() == hit.orientation.signum()
        }) {
            group.push(hit.clone());
        } else {
            groups.push(vec![hit.clone()]);
        }
    }
    groups.sort_by(|left, right| left[0].distance.total_cmp(&right[0].distance));
    groups
}

fn format_groups(groups: &[Vec<RawHit>]) -> String {
    groups
        .iter()
        .map(|group| {
            let hit = &group[0];
            format!(
                "distance:{:.3},identity:{},family:{},orientation:{:.6},raw-members:{},role:{}",
                hit.distance,
                hit.identity,
                hit.family,
                hit.orientation,
                group.len(),
                if hit.family == "paired-sky-height-discontinuity" {
                    "NonTransition:both-adjacent-ceilings-sky"
                } else {
                    "Ambiguous:open-oriented-cap-without-closed-domain"
                },
            )
        })
        .collect::<Vec<_>>()
        .join(";")
}

fn audit_closure(candidates: &[CandidateTriangle]) -> ClosureAudit {
    let mut edges = BTreeMap::<([u32; 3], [u32; 3]), usize>::new();
    for triangle in candidates {
        let vertices = triangle
            .vertices
            .map(|vertex| vertex.to_array().map(f32::to_bits));
        for (mut left, mut right) in [
            (vertices[0], vertices[1]),
            (vertices[1], vertices[2]),
            (vertices[2], vertices[0]),
        ] {
            if left > right {
                std::mem::swap(&mut left, &mut right);
            }
            *edges.entry((left, right)).or_default() += 1;
        }
    }
    ClosureAudit {
        triangles: candidates.len(),
        unique_edges: edges.len(),
        manifold_edges: edges.values().filter(|count| **count == 2).count(),
        open_edges: edges.values().filter(|count| **count == 1).count(),
        non_manifold_edges: edges.values().filter(|count| **count > 2).count(),
    }
}

pub(super) fn source_ray_vectors(origin: [f64; 3], direction: [f64; 3]) -> (Vec3, Vec3) {
    let embedding = DoomComparativeEmbedding::CurrentReflected;
    let origin = embedding.lift_direction([origin[0] as f32, origin[1] as f32], origin[2] as f32);
    let direction = embedding
        .lift_direction(
            [direction[0] as f32, direction[1] as f32],
            direction[2] as f32,
        )
        .normalize_or_zero();
    (origin, direction)
}

fn fingerprint_report(
    rows: &[String],
    candidates: &[CandidateTriangle],
    closure: ClosureAudit,
    paired_both_sky: usize,
    paired_world_sky: usize,
    paired_unresolved: usize,
) -> u64 {
    let mut fingerprint = 0xcbf29ce484222325u64;
    for value in [
        closure.triangles,
        closure.unique_edges,
        closure.manifold_edges,
        closure.open_edges,
        closure.non_manifold_edges,
        paired_both_sky,
        paired_world_sky,
        paired_unresolved,
    ] {
        hash_bytes(&mut fingerprint, &value.to_le_bytes());
    }
    for candidate in candidates {
        hash_bytes(&mut fingerprint, candidate.identity.as_bytes());
        for vertex in candidate.vertices {
            for component in vertex.to_array() {
                hash_bytes(&mut fingerprint, &component.to_bits().to_le_bytes());
            }
        }
    }
    for row in rows {
        hash_bytes(&mut fingerprint, row.as_bytes());
    }
    fingerprint
}

fn hash_bytes(fingerprint: &mut u64, bytes: &[u8]) {
    for byte in bytes {
        *fingerprint ^= u64::from(*byte);
        *fingerprint = fingerprint.wrapping_mul(0x100000001b3);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn frozen_matrix_has_five_sky_and_five_world_controls() {
        let rays = frozen_rays();
        assert_eq!(rays.len(), 10);
        assert_eq!(
            rays.iter()
                .filter(|ray| ray.expected_state == ExpectedState::Sky)
                .count(),
            5
        );
        assert_eq!(
            rays.iter()
                .filter(|ray| ray.expected_state == ExpectedState::World)
                .count(),
            5
        );
        assert_eq!(
            rays.iter()
                .map(|ray| ray.name)
                .collect::<BTreeSet<_>>()
                .len(),
            rays.len()
        );
    }

    #[test]
    fn duplicate_collapse_requires_identity_distance_and_orientation() {
        let hit = |identity: &str, distance: f32, orientation: f32| RawHit {
            identity: identity.to_owned(),
            family: "test",
            distance,
            orientation,
        };
        let groups = collapse_hits(&[
            hit("a", 1.0, -1.0),
            hit("a", 1.0005, -0.5),
            hit("a", 1.0005, 0.5),
            hit("b", 1.0005, -0.5),
        ]);
        assert_eq!(groups.len(), 3);
        assert_eq!(groups.iter().map(Vec::len).sum::<usize>(), 4);
    }
}
