//! Shadow-only adversarial search for one-way sky-hit correlation.
//!
//! A preceding Doom-related sky surface is recorded independently from final
//! ordered source participation. It never removes renderer declarations.

use super::super::*;
use super::sky_transition_parity::{
    candidate_hits_before, candidate_triangles, collapse_hits, frozen_rays, source_ray_vectors,
    CandidateTriangle,
};

const SOURCE_COLUMNS: usize = 320;
const SOURCE_ROWS: usize = 200;
const GRID_COLUMNS: usize = 32;
const GRID_ROWS: usize = 20;
const REPRESENTATIVE_LIMIT: usize = 32;

#[derive(Clone, Copy, Debug)]
struct SearchPose {
    name: &'static str,
    viewer: [i16; 2],
    eye_height: i16,
    heading_degrees: f64,
}

const SEARCH_POSES: [SearchPose; 8] = [
    SearchPose {
        name: "source-spawn",
        viewer: [1056, -3616],
        eye_height: 36,
        heading_degrees: 90.0,
    },
    SearchPose {
        name: "spawn-near-wall",
        viewer: [1202, -3502],
        eye_height: 36,
        heading_degrees: -24.0,
    },
    SearchPose {
        name: "stairs-high-platform",
        viewer: [1286, -2552],
        eye_height: 36,
        heading_degrees: -100.0,
    },
    SearchPose {
        name: "courtyard",
        viewer: [1514, -2481],
        eye_height: 36,
        heading_degrees: -29.2,
    },
    SearchPose {
        name: "hut-west",
        viewer: [2042, -2976],
        eye_height: -20,
        heading_degrees: -52.0,
    },
    SearchPose {
        name: "hut-east",
        viewer: [2076, -3560],
        eye_height: 36,
        heading_degrees: -25.0,
    },
    SearchPose {
        name: "hut-north",
        viewer: [2249, -3361],
        eye_height: -20,
        heading_degrees: -159.0,
    },
    SearchPose {
        name: "far-left-structure",
        viewer: [2902, -3207],
        eye_height: 9,
        heading_degrees: -163.0,
    },
];

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
enum SourceResult {
    ExactPresent,
    SourcePartial,
    Absent,
}

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
enum GroupParity {
    Even,
    Odd,
}

impl GroupParity {
    const fn from_count(count: usize) -> Self {
        if count.is_multiple_of(2) {
            Self::Even
        } else {
            Self::Odd
        }
    }

    const fn label(self) -> &'static str {
        match self {
            Self::Even => "even-world-candidate",
            Self::Odd => "odd-sky-candidate",
        }
    }
}

impl SourceResult {
    const fn label(self) -> &'static str {
        match self {
            Self::ExactPresent => "exact-present",
            Self::SourcePartial => "source-partial",
            Self::Absent => "absent",
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
enum Quadrant {
    NoSkyPresent,
    NoSkyNotExact,
    SkyNotExact,
    SkyExactPresent,
}

impl Quadrant {
    const fn label(self) -> &'static str {
        match self {
            Self::NoSkyPresent => "no-sky+exact-present",
            Self::NoSkyNotExact => "no-sky+partial-or-absent",
            Self::SkyNotExact => "sky-before+partial-or-absent",
            Self::SkyExactPresent => "sky-before+exact-present-critical-falsifier",
        }
    }
}

#[derive(Clone, Debug)]
struct RayObservation {
    report: String,
    quadrant: Quadrant,
}

pub(crate) fn report_one_way_sky_occlusion_correlation(scene: &SceneInput) -> PlatformResult<()> {
    let candidates = candidate_triangles(scene);
    let cutout_materials = scene
        .cutout_uploads
        .iter()
        .map(|upload| (upload.source_name.clone(), upload.material))
        .collect::<BTreeMap<_, _>>();
    let mut observations = Vec::new();
    let mut sampled_rays = 0usize;
    let mut no_ordinary_target = 0usize;
    let mut raw_sky_hits = 0usize;
    let mut semantic_sky_groups = 0usize;
    let mut duplicate_collapses = 0usize;

    for pose in SEARCH_POSES {
        let heading = pose.heading_degrees.to_radians();
        let ordered = prepare_ordered_occurrence_submission(
            &scene.door_geometry_source.map,
            pose.viewer,
            heading,
            pose.eye_height,
            &scene.door_geometry_source.wall_extents,
            &scene.door_geometry_source.wall_materials,
            &cutout_materials,
            &scene.opaque_uploads,
        )
        .map_err(io::Error::other)?;
        ordered.verify_conservation().map_err(io::Error::other)?;
        let ordered_draws = ordered_draws(&ordered);

        for grid_row in 0..GRID_ROWS {
            let row = grid_row * SOURCE_ROWS / GRID_ROWS + SOURCE_ROWS / GRID_ROWS / 2;
            for grid_column in 0..GRID_COLUMNS {
                let column =
                    grid_column * SOURCE_COLUMNS / GRID_COLUMNS + SOURCE_COLUMNS / GRID_COLUMNS / 2;
                sampled_rays += 1;
                let source_direction = source_cell_center_direction(column, row, heading);
                let source_origin = [
                    f64::from(pose.viewer[0]),
                    f64::from(pose.viewer[1]),
                    f64::from(pose.eye_height),
                ];
                let Some(observation) = observe_ray(
                    scene,
                    &candidates,
                    &ordered_draws,
                    pose.name,
                    source_origin,
                    source_direction,
                    Some([column, row]),
                )?
                else {
                    no_ordinary_target += 1;
                    continue;
                };
                raw_sky_hits += observation.1;
                semantic_sky_groups += observation.2;
                duplicate_collapses += observation.1.saturating_sub(observation.2);
                observations.push(observation.0);
            }
        }
    }

    let mut frozen_rows = Vec::new();
    for ray in frozen_rays() {
        let viewer = [ray.origin[0].round() as i16, ray.origin[1].round() as i16];
        let eye_height = ray.origin[2].round() as i16;
        let heading = ray.direction[1].atan2(ray.direction[0]);
        let ordered = prepare_ordered_occurrence_submission(
            &scene.door_geometry_source.map,
            viewer,
            heading,
            eye_height,
            &scene.door_geometry_source.wall_extents,
            &scene.door_geometry_source.wall_materials,
            &cutout_materials,
            &scene.opaque_uploads,
        )
        .map_err(io::Error::other)?;
        ordered.verify_conservation().map_err(io::Error::other)?;
        let observation = observe_ray(
            scene,
            &candidates,
            &ordered_draws(&ordered),
            ray.name,
            ray.origin,
            ray.direction,
            None,
        )?
        .ok_or_else(|| io::Error::other(format!("{} lost frozen ordinary target", ray.name)))?;
        frozen_rows.push(observation.0.report);
    }

    let quadrant_counts = observations.iter().fold(
        BTreeMap::<Quadrant, usize>::new(),
        |mut counts, observation| {
            *counts.entry(observation.quadrant).or_default() += 1;
            counts
        },
    );
    let critical_falsifiers = quadrant_counts
        .get(&Quadrant::SkyExactPresent)
        .copied()
        .unwrap_or_default();
    let ordinary_targets = observations.len();
    let sky_before = quadrant_counts
        .get(&Quadrant::SkyNotExact)
        .copied()
        .unwrap_or_default()
        + critical_falsifiers;
    let no_sky_before = ordinary_targets - sky_before;
    if sampled_rays != no_ordinary_target + ordinary_targets
        || ordinary_targets != sky_before + no_sky_before
        || raw_sky_hits != semantic_sky_groups + duplicate_collapses
    {
        return Err(io::Error::other("sky-occlusion correlation conservation failed").into());
    }

    let representatives = observations
        .iter()
        .filter(|observation| observation.quadrant == Quadrant::SkyExactPresent)
        .take(REPRESENTATIVE_LIMIT)
        .map(|observation| observation.report.as_str())
        .collect::<Vec<_>>();
    let supporting = observations
        .iter()
        .filter(|observation| observation.quadrant == Quadrant::SkyNotExact)
        .take(8)
        .map(|observation| observation.report.as_str())
        .collect::<Vec<_>>();
    let mut fingerprint = 0xcbf29ce484222325u64;
    for observation in &observations {
        hash_text(&mut fingerprint, &observation.report);
    }
    for row in &frozen_rows {
        hash_text(&mut fingerprint, row);
    }
    let disposition = if critical_falsifiers == 0 {
        "static-first-gate-survives-continue-adversarial-search"
    } else {
        "blanket-one-way-mask-falsified-valid-exact-source-geometry-exists-behind-sky-hit"
    };
    println!(
        "E1M1 one-way sky-occlusion correlation shadow Slice 0-1: poses={}; grid={}x{}; sampled-rays={sampled_rays}; ordinary-targets={ordinary_targets}; no-ordinary-target={no_ordinary_target}; sky-before={sky_before}; no-sky-before={no_sky_before}; quadrants=[no-sky+exact-present:{},no-sky+partial-or-absent:{},sky-before+partial-or-absent:{},sky-before+exact-present-critical-falsifier:{critical_falsifiers}]; raw-sky-hits={raw_sky_hits}; semantic-sky-groups={semantic_sky_groups}; duplicate-collapses={duplicate_collapses}; frozen-controls={}; historical-cause=ordered-solid-coverage-separate-from-sky-correlation; renderer-mutation=false; conservation=balanced; fingerprint={fingerprint:016x}; disposition={disposition}; critical-representatives=[{}]; supporting-representatives=[{}]; frozen-rows=[{}]",
        SEARCH_POSES.len(),
        GRID_COLUMNS,
        GRID_ROWS,
        quadrant_counts
            .get(&Quadrant::NoSkyPresent)
            .copied()
            .unwrap_or_default(),
        quadrant_counts
            .get(&Quadrant::NoSkyNotExact)
            .copied()
            .unwrap_or_default(),
        quadrant_counts
            .get(&Quadrant::SkyNotExact)
            .copied()
            .unwrap_or_default(),
        frozen_rows.len(),
        representatives.join(" | "),
        supporting.join(" | "),
        frozen_rows.join(" | "),
    );
    Ok(())
}

/// Reclassifies the exact deterministic 36-ray sky-hit corpus retained by the
/// one-way study. This is a correlation shadow, not proof that the open Doom
/// sky surfaces form a globally closed volume or authority to omit geometry.
pub(crate) fn report_grouped_sky_crossing_parity(scene: &SceneInput) -> PlatformResult<()> {
    let candidates = candidate_triangles(scene);
    let cutout_materials = scene
        .cutout_uploads
        .iter()
        .map(|upload| (upload.source_name.clone(), upload.material))
        .collect::<BTreeMap<_, _>>();
    let mut rows = Vec::new();
    let mut matrix = BTreeMap::<(GroupParity, SourceResult), usize>::new();
    let mut family_sequences = BTreeMap::<String, usize>::new();
    let mut raw_hits = 0usize;
    let mut grouped_hits = 0usize;
    let mut paired_groups = 0usize;
    let mut source_plane_groups = 0usize;
    let mut semantically_presenting = 0usize;
    let mut semantically_backside = 0usize;
    let mut semantically_unresolved = 0usize;

    for pose in SEARCH_POSES {
        let heading = pose.heading_degrees.to_radians();
        let ordered = prepare_ordered_occurrence_submission(
            &scene.door_geometry_source.map,
            pose.viewer,
            heading,
            pose.eye_height,
            &scene.door_geometry_source.wall_extents,
            &scene.door_geometry_source.wall_materials,
            &cutout_materials,
            &scene.opaque_uploads,
        )
        .map_err(io::Error::other)?;
        ordered.verify_conservation().map_err(io::Error::other)?;
        let ordered_draws = ordered_draws(&ordered);

        for grid_row in 0..GRID_ROWS {
            let row = grid_row * SOURCE_ROWS / GRID_ROWS + SOURCE_ROWS / GRID_ROWS / 2;
            for grid_column in 0..GRID_COLUMNS {
                let column =
                    grid_column * SOURCE_COLUMNS / GRID_COLUMNS + SOURCE_COLUMNS / GRID_COLUMNS / 2;
                let source_origin = [
                    f64::from(pose.viewer[0]),
                    f64::from(pose.viewer[1]),
                    f64::from(pose.eye_height),
                ];
                let source_direction = source_cell_center_direction(column, row, heading);
                let (origin, direction) = source_ray_vectors(source_origin, source_direction);
                let Some(ordinary) = nearest_prepared_ray_hit(
                    origin,
                    direction,
                    &scene.opaque_draws,
                    Some(&scene.cutout_draws),
                ) else {
                    continue;
                };
                let hits = candidate_hits_before(&candidates, origin, direction, ordinary.distance);
                let groups = collapse_hits(&hits);
                if groups.is_empty() {
                    continue;
                }

                let matching_draws = ordered_draws
                    .iter()
                    .filter(|draw| draw.source == ordinary.draw.source)
                    .cloned()
                    .collect::<Vec<_>>();
                let exact_hit = nearest_prepared_ray_hit(origin, direction, &matching_draws, None);
                let source_result = if exact_hit.is_some() {
                    SourceResult::ExactPresent
                } else if matching_draws.is_empty() {
                    SourceResult::Absent
                } else {
                    SourceResult::SourcePartial
                };
                let parity = GroupParity::from_count(groups.len());
                *matrix.entry((parity, source_result)).or_default() += 1;
                raw_hits += hits.len();
                grouped_hits += groups.len();

                let sequence = groups
                    .iter()
                    .map(|group| group[0].family)
                    .collect::<Vec<_>>()
                    .join(">");
                *family_sequences.entry(sequence.clone()).or_default() += 1;
                let observations = groups
                    .iter()
                    .map(|group| {
                        let hit = &group[0];
                        let semantic_side = if hit.family == "source-sky-open-plane" {
                            source_plane_groups += 1;
                            if source_direction[2] > f64::EPSILON {
                                semantically_presenting += 1;
                                "presenting:source-ceiling-underside"
                            } else if source_direction[2] < -f64::EPSILON {
                                semantically_backside += 1;
                                "backside:source-ceiling-topside"
                            } else {
                                semantically_unresolved += 1;
                                "unresolved:tangent-source-ceiling"
                            }
                        } else {
                            paired_groups += 1;
                            semantically_unresolved += 1;
                            "unresolved:paired-sky-both-adjacent-ceilings-sky"
                        };
                        let orientations = group
                            .iter()
                            .map(|member| format!("{:.6}", member.orientation))
                            .collect::<Vec<_>>()
                            .join(",");
                        format!(
                            "distance:{:.3},identity:{},family:{},raw-members:{},raw-orientations:[{}],semantic-side:{}",
                            hit.distance,
                            hit.identity,
                            hit.family,
                            group.len(),
                            orientations,
                            semantic_side,
                        )
                    })
                    .collect::<Vec<_>>()
                    .join(";");
                rows.push(format!(
                    "case={}:cell=({},{}):origin=({:.3},{:.3},{:.3}):direction=({:.9},{:.9},{:.9}):target={}:target-distance={:.3}:source-result={}:matching-source-declarations={}:exact-source-distance={}:group-count={}:parity={}:family-sequence={}:groups=[{}]",
                    pose.name,
                    column,
                    row,
                    source_origin[0],
                    source_origin[1],
                    source_origin[2],
                    source_direction[0],
                    source_direction[1],
                    source_direction[2],
                    ordinary.draw.source_label,
                    ordinary.distance,
                    source_result.label(),
                    matching_draws.len(),
                    exact_hit
                        .map(|hit| format!("{:.3}", hit.distance))
                        .unwrap_or_else(|| "none".to_owned()),
                    groups.len(),
                    parity.label(),
                    sequence,
                    observations,
                ));
            }
        }
    }

    let exact_present = matrix
        .iter()
        .filter(|((_, result), _)| *result == SourceResult::ExactPresent)
        .map(|(_, count)| count)
        .sum::<usize>();
    let non_exact = rows.len() - exact_present;
    if rows.len() != 36 || exact_present != 8 || non_exact != 28 {
        return Err(io::Error::other(format!(
            "retained sky-hit corpus drifted: expected 36/8/28 rays, got {}/{}/{}",
            rows.len(),
            exact_present,
            non_exact
        ))
        .into());
    }
    if raw_hits < grouped_hits
        || grouped_hits != paired_groups + source_plane_groups
        || grouped_hits != semantically_presenting + semantically_backside + semantically_unresolved
    {
        return Err(io::Error::other("grouped sky-crossing conservation failed").into());
    }

    let odd_exact = matrix
        .get(&(GroupParity::Odd, SourceResult::ExactPresent))
        .copied()
        .unwrap_or_default();
    let even_absent = matrix
        .get(&(GroupParity::Even, SourceResult::Absent))
        .copied()
        .unwrap_or_default();
    let mut fingerprint = 0xcbf29ce484222325u64;
    for row in &rows {
        hash_text(&mut fingerprint, row);
    }
    let mut matrix_cells = Vec::new();
    for parity in [GroupParity::Even, GroupParity::Odd] {
        for result in [
            SourceResult::ExactPresent,
            SourceResult::SourcePartial,
            SourceResult::Absent,
        ] {
            matrix_cells.push(format!(
                "{}+{}:{}",
                parity.label(),
                result.label(),
                matrix.get(&(parity, result)).copied().unwrap_or_default()
            ));
        }
    }
    let matrix_text = matrix_cells.join(",");
    let family_text = family_sequences
        .iter()
        .map(|(sequence, count)| format!("{}:{}", sequence, count))
        .collect::<Vec<_>>()
        .join(",");
    let disposition = if odd_exact == 0 && even_absent == 0 {
        "existing-36-correlation-gate-survives-broader-shadow-authorized"
    } else {
        "existing-36-correlation-gate-has-counterexamples-inspect-before-broadening"
    };
    println!(
        "E1M1 grouped sky-crossing parity reclassification Slice 1: retained-rays={}; exact-present={exact_present}; partial-or-absent={non_exact}; raw-hits={raw_hits}; grouped-hits={grouped_hits}; duplicate-collapses={}; paired-groups={paired_groups}; source-plane-groups={source_plane_groups}; semantic-sides=[presenting:{semantically_presenting},backside:{semantically_backside},unresolved:{semantically_unresolved}]; matrix=[{matrix_text}]; family-sequences=[{family_text}]; parity-errors=[odd+exact-present:{odd_exact},even+absent:{even_absent}]; source-result-authority=ordered-frozen-view-correlation-not-free-look-pixel-or-canonical-world-proof; raw-winding-authority=diagnostic-only; renderer-mutation=false; conservation=balanced; fingerprint={fingerprint:016x}; disposition={disposition}; rows=[{}]",
        rows.len(),
        raw_hits - grouped_hits,
        rows.join(" | "),
    );
    Ok(())
}

fn observe_ray(
    scene: &SceneInput,
    candidates: &[CandidateTriangle],
    ordered_draws: &[StaticDrawPlanEntry],
    name: &str,
    source_origin: [f64; 3],
    source_direction: [f64; 3],
    cell: Option<[usize; 2]>,
) -> PlatformResult<Option<(RayObservation, usize, usize)>> {
    let (origin, direction) = source_ray_vectors(source_origin, source_direction);
    let Some(ordinary) = nearest_prepared_ray_hit(
        origin,
        direction,
        &scene.opaque_draws,
        Some(&scene.cutout_draws),
    ) else {
        return Ok(None);
    };
    let raw_hits = candidate_hits_before(candidates, origin, direction, ordinary.distance);
    let groups = collapse_hits(&raw_hits);
    let matching_draws = ordered_draws
        .iter()
        .filter(|draw| draw.source == ordinary.draw.source)
        .cloned()
        .collect::<Vec<_>>();
    let exact_hit = nearest_prepared_ray_hit(origin, direction, &matching_draws, None);
    let source_result = if exact_hit.is_some() {
        SourceResult::ExactPresent
    } else if matching_draws.is_empty() {
        SourceResult::Absent
    } else {
        SourceResult::SourcePartial
    };
    let sky_before = !groups.is_empty();
    let quadrant = match (sky_before, source_result) {
        (false, SourceResult::ExactPresent) => Quadrant::NoSkyPresent,
        (false, SourceResult::SourcePartial | SourceResult::Absent) => Quadrant::NoSkyNotExact,
        (true, SourceResult::ExactPresent) => Quadrant::SkyExactPresent,
        (true, SourceResult::SourcePartial | SourceResult::Absent) => Quadrant::SkyNotExact,
    };
    let nearest_sky = groups.first().map(|group| &group[0]);
    let cell = cell
        .map(|[column, row]| format!("({column},{row})"))
        .unwrap_or_else(|| "exact-replay".to_owned());
    let report = format!(
        "case={name}:cell={cell}:origin=({:.3},{:.3},{:.3}):direction=({:.9},{:.9},{:.9}):target={}:target-distance={:.3}:sky-before={sky_before}:sky-groups={}:nearest-sky={}:matching-source-declarations={}:exact-source-distance={}:source-result={}:quadrant={}",
        source_origin[0],
        source_origin[1],
        source_origin[2],
        source_direction[0],
        source_direction[1],
        source_direction[2],
        ordinary.draw.source_label,
        ordinary.distance,
        groups.len(),
        nearest_sky
            .map(|hit| format!(
                "{}@{:.3}:family={}:orientation={:.6}",
                hit.identity, hit.distance, hit.family, hit.orientation
            ))
            .unwrap_or_else(|| "none".to_owned()),
        matching_draws.len(),
        exact_hit
            .map(|hit| format!("{:.3}", hit.distance))
            .unwrap_or_else(|| "none".to_owned()),
        source_result.label(),
        quadrant.label(),
    );
    Ok(Some((
        RayObservation { report, quadrant },
        raw_hits.len(),
        groups.len(),
    )))
}

fn ordered_draws(ordered: &OrderedPreparedSubmissionObservation) -> Vec<StaticDrawPlanEntry> {
    ordered
        .walls
        .prepared_declarations
        .iter()
        .map(|declaration| declaration.draw.clone())
        .chain(
            ordered
                .plane_lowering
                .prepared_declarations
                .iter()
                .map(|declaration| declaration.draw.clone()),
        )
        .collect()
}

fn source_cell_center_direction(column: usize, row: usize, heading: f64) -> [f64; 3] {
    let horizontal_normalized = ((column as f64 + 0.5) / SOURCE_COLUMNS as f64) * 2.0 - 1.0;
    let vertical_normalized = 1.0 - ((row as f64 + 0.5) / SOURCE_ROWS as f64) * 2.0;
    let half_vertical_fov =
        (std::f64::consts::FRAC_PI_4.tan() / (SOURCE_COLUMNS as f64 / SOURCE_ROWS as f64)).atan();
    let forward = [heading.cos(), heading.sin()];
    let right = [-forward[1], forward[0]];
    let lateral = horizontal_normalized * std::f64::consts::FRAC_PI_4.tan();
    let vertical = vertical_normalized * half_vertical_fov.tan();
    let direction = [
        forward[0] + right[0] * lateral,
        forward[1] + right[1] * lateral,
        vertical,
    ];
    let length = direction[0].hypot(direction[1]).hypot(direction[2]);
    [
        direction[0] / length,
        direction[1] / length,
        direction[2] / length,
    ]
}

fn hash_text(fingerprint: &mut u64, text: &str) {
    for byte in text.bytes() {
        *fingerprint ^= u64::from(byte);
        *fingerprint = fingerprint.wrapping_mul(0x100000001b3);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn adversarial_pose_names_are_unique() {
        assert_eq!(
            SEARCH_POSES
                .iter()
                .map(|pose| pose.name)
                .collect::<BTreeSet<_>>()
                .len(),
            SEARCH_POSES.len()
        );
    }

    #[test]
    fn critical_quadrant_is_sky_before_exact_present() {
        assert_eq!(
            Quadrant::SkyExactPresent.label(),
            "sky-before+exact-present-critical-falsifier"
        );
    }

    #[test]
    fn grouped_crossing_parity_is_even_world_odd_sky() {
        assert_eq!(GroupParity::from_count(0), GroupParity::Even);
        assert_eq!(GroupParity::from_count(2), GroupParity::Even);
        assert_eq!(GroupParity::from_count(1), GroupParity::Odd);
        assert_eq!(GroupParity::from_count(3), GroupParity::Odd);
    }
}
