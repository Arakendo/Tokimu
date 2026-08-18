//! Tokimu-first corpus-local spatial bake over exact prepared triangles.
//!
//! This deliberately does not import Doom BSP topology. The reference BSP and
//! BVH control consume the same finite Tokimu geometry-member inventory and
//! retain exact original-member correlation. Nothing here affects submission.

use std::{
    collections::{BTreeMap, BTreeSet},
    io,
    time::Instant,
};

use hello_doom_e1m1::{
    classify_static_draw_frustum_rejection, observer_yaw_from_forward, DoomComparativeEmbedding,
    StaticDrawAabb, StaticDrawSource,
};
use tokimu::PlatformResult;
use tokimu_core::math::{Mat4, Vec3};
use tokimu_spatial_query_study::{Artifact as StudyArtifact, TriangleMember};

use crate::{observer_direction, ray_triangle_distance, DoomSurfacePlane, SceneInput};

const LEAF_MEMBERS: usize = 24;
const MAX_DEPTH: usize = 20;
const MAX_GENERATED_FRAGMENTS: usize = 500_000;
const EPSILON: f32 = 1.0e-4;

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
enum MemberFamily {
    Floor,
    Ceiling,
    Wall,
    Cutout,
}

impl MemberFamily {
    const ALL: [Self; 4] = [Self::Floor, Self::Ceiling, Self::Wall, Self::Cutout];

    const fn label(self) -> &'static str {
        match self {
            Self::Floor => "floor",
            Self::Ceiling => "ceiling",
            Self::Wall => "wall",
            Self::Cutout => "cutout",
        }
    }
}

#[derive(Clone, Debug)]
struct SpatialMember {
    id: usize,
    family: MemberFamily,
    source_label: String,
    vertices: [[f32; 3]; 3],
}

/// Shadow-only correlation between exact prepared triangles and the
/// corpus-local BVH. It deliberately exposes no submission decision.
pub(super) struct SpatialRayShadow {
    members: Vec<SpatialMember>,
    bvh: StudyArtifact,
}

#[derive(Clone, Debug)]
pub(super) struct SpatialRayShadowHit {
    pub source_label: String,
    pub distance: f32,
    pub member_identity: usize,
    pub visited_nodes: usize,
    pub tested_members: usize,
}

impl SpatialRayShadow {
    pub(super) fn build(scene: &SceneInput) -> PlatformResult<Self> {
        let members = collect_members(scene)?;
        let bvh = StudyArtifact::build(study_members(&members)?, 0)?;
        let audit = bvh.audit();
        if audit.missing_members != 0
            || audit.duplicate_members != 0
            || audit.containment_failures != 0
        {
            return Err(io::Error::other(format!(
                "spatial ray shadow BVH audit failed: missing={} duplicates={} containment-failures={}",
                audit.missing_members, audit.duplicate_members, audit.containment_failures,
            ))
            .into());
        }
        Ok(Self { members, bvh })
    }

    pub(super) fn query_source_ray(
        &self,
        embedding: DoomComparativeEmbedding,
        origin: [f64; 3],
        direction: [f64; 3],
    ) -> PlatformResult<Option<SpatialRayShadowHit>> {
        let origin =
            embedding.lift_direction([origin[0] as f32, origin[1] as f32], origin[2] as f32);
        let direction = embedding
            .lift_direction(
                [direction[0] as f32, direction[1] as f32],
                direction[2] as f32,
            )
            .normalize_or_zero();
        let (hit, stats) = self
            .bvh
            .query_nearest_ray(self.bvh.revision(), origin, direction)?;
        let brute = brute_nearest_ray(&self.members, origin, direction);
        let observed = hit.map(|hit| (hit.identity, hit.distance));
        if !ray_hits_match(observed, brute) {
            return Err(io::Error::other(format!(
                "spatial ray shadow disagrees with brute-force oracle: bvh={observed:?} brute={brute:?}",
            ))
            .into());
        }
        Ok(hit.map(|hit| SpatialRayShadowHit {
            source_label: self.members[hit.identity].source_label.clone(),
            distance: hit.distance,
            member_identity: hit.identity,
            visited_nodes: stats.visited_nodes,
            tested_members: stats.tested_members,
        }))
    }
}

#[derive(Clone, Debug)]
struct BspFragment {
    original: usize,
    vertices: [[f32; 3]; 3],
}

#[derive(Clone, Copy, Debug)]
struct Bounds {
    minimum: [f32; 3],
    maximum: [f32; 3],
}

impl Bounds {
    fn from_triangle(vertices: &[[f32; 3]; 3]) -> Self {
        let mut bounds = Self {
            minimum: vertices[0],
            maximum: vertices[0],
        };
        for vertex in &vertices[1..] {
            bounds.include_point(*vertex);
        }
        bounds
    }

    fn from_fragments(fragments: &[BspFragment]) -> Self {
        let mut bounds = Self::from_triangle(&fragments[0].vertices);
        for fragment in &fragments[1..] {
            bounds.include_bounds(Self::from_triangle(&fragment.vertices));
        }
        bounds
    }

    fn from_members(members: &[SpatialMember], indices: &[usize]) -> Self {
        let mut bounds = Self::from_triangle(&members[indices[0]].vertices);
        for index in &indices[1..] {
            bounds.include_bounds(Self::from_triangle(&members[*index].vertices));
        }
        bounds
    }

    fn include_point(&mut self, point: [f32; 3]) {
        for (axis, coordinate) in point.into_iter().enumerate() {
            self.minimum[axis] = self.minimum[axis].min(coordinate);
            self.maximum[axis] = self.maximum[axis].max(coordinate);
        }
    }

    fn include_bounds(&mut self, other: Self) {
        self.include_point(other.minimum);
        self.include_point(other.maximum);
    }

    fn contains_point(self, point: [f32; 3]) -> bool {
        (0..3).all(|axis| {
            point[axis] >= self.minimum[axis] - EPSILON
                && point[axis] <= self.maximum[axis] + EPSILON
        })
    }

    fn contains_bounds(self, other: Self) -> bool {
        self.contains_point(other.minimum) && self.contains_point(other.maximum)
    }

    fn longest_axis(self) -> usize {
        let extent = [
            self.maximum[0] - self.minimum[0],
            self.maximum[1] - self.minimum[1],
            self.maximum[2] - self.minimum[2],
        ];
        if extent[1] > extent[0] && extent[1] >= extent[2] {
            1
        } else if extent[2] > extent[0] && extent[2] > extent[1] {
            2
        } else {
            0
        }
    }
}

#[derive(Debug)]
enum BspNode {
    Leaf {
        bounds: Bounds,
        fragments: Vec<BspFragment>,
    },
    Branch {
        bounds: Bounds,
        axis: usize,
        plane: f32,
        lower: Box<BspNode>,
        upper: Box<BspNode>,
    },
}

#[derive(Default)]
struct BspBuildStats {
    nodes: usize,
    leaves: usize,
    maximum_depth: usize,
    split_inputs: usize,
    generated_fragments: usize,
    depth_limited_leaves: usize,
    unsplittable_leaves: usize,
    budget_limited_leaves: usize,
}

#[derive(Default)]
struct ContainmentAudit {
    nodes: usize,
    leaves: usize,
    fragments_or_members: usize,
    failures: usize,
}

#[derive(Clone, Copy)]
struct QueryPose {
    label: &'static str,
    origin: Vec3,
    direction: Vec3,
}

type LabeledArtifactQuery = (BTreeSet<String>, Option<(String, f32)>, f64);

pub(crate) fn report_tokimu_spatial_bake(scene: &SceneInput) -> PlatformResult<()> {
    let members = collect_members(scene)?;
    let original_area = members
        .iter()
        .map(|member| triangle_area(member.vertices))
        .sum::<f64>();

    let bsp_started = Instant::now();
    let mut bsp_stats = BspBuildStats::default();
    let initial_fragments = members
        .iter()
        .map(|member| BspFragment {
            original: member.id,
            vertices: member.vertices,
        })
        .collect::<Vec<_>>();
    let bsp = build_bsp(initial_fragments, 0, &mut bsp_stats)?;
    let bsp_elapsed_ms = bsp_started.elapsed().as_secs_f64() * 1_000.0;
    let mut bsp_audit = ContainmentAudit::default();
    let mut final_fragments = Vec::new();
    audit_bsp(&bsp, None, &mut bsp_audit, &mut final_fragments);

    let bvh_started = Instant::now();
    let bvh = StudyArtifact::build(study_members(&members)?, 0)?;
    let bvh_stats = bvh.build_stats();
    let bvh_elapsed_ms = bvh_started.elapsed().as_secs_f64() * 1_000.0;
    let bvh_audit = bvh.audit();

    let mut occurrences = vec![0usize; members.len()];
    let mut final_area = 0.0_f64;
    for fragment in &final_fragments {
        occurrences[fragment.original] += 1;
        final_area += triangle_area(fragment.vertices);
    }
    let missing = occurrences.iter().filter(|count| **count == 0).count();
    let area_delta = (final_area - original_area).abs();
    let area_tolerance = original_area.max(1.0) * 1.0e-5;
    let bvh_missing = bvh_audit.missing_members;
    let bvh_duplicates = bvh_audit.duplicate_members;

    let bsp_fingerprint = fingerprint_bsp(&bsp, &members);
    let bvh_fingerprint = bvh.structure_fingerprint();
    let family_report = family_amplification(&members, &final_fragments);
    let source_labels = members
        .iter()
        .map(|member| member.source_label.as_str())
        .collect::<std::collections::BTreeSet<_>>()
        .len();

    println!(
        "Tokimu spatial bake inventory: representation=prepared-triangle; members={}; source-draw-labels={source_labels}; dynamic-sidecar=0; families=[{}]; meaning=exact-finite-Tokimu-geometry-not-source-bsp-records",
        members.len(),
        family_report.0,
    );
    println!(
        "Tokimu BSP reference bake: nodes={}; leaves={}; maximum-depth={}; final-fragments={}; split-inputs={}; generated-split-fragments={}; generated-fragment-budget={MAX_GENERATED_FRAGMENTS}; amplification={:.6}; fragment-payload-bytes-lower-bound={}; depth-limited-leaves={}; unsplittable-leaves={}; budget-limited-leaves={}; containment=[nodes:{},leaves:{},fragments:{},failures:{}]; conservation=[originals:{},represented:{},missing:{missing},area-original:{original_area:.6},area-final:{final_area:.6},area-delta:{area_delta:.6},area-tolerance:{area_tolerance:.6}]; family-amplification=[{}]; fingerprint={bsp_fingerprint:016x}; elapsed-ms={bsp_elapsed_ms:.3}; meaning=corpus-local-Tokimu-first-binary-space-partition-no-presentation-authority",
        bsp_stats.nodes,
        bsp_stats.leaves,
        bsp_stats.maximum_depth,
        final_fragments.len(),
        bsp_stats.split_inputs,
        bsp_stats.generated_fragments,
        final_fragments.len() as f64 / members.len().max(1) as f64,
        final_fragments.len() * std::mem::size_of::<BspFragment>(),
        bsp_stats.depth_limited_leaves,
        bsp_stats.unsplittable_leaves,
        bsp_stats.budget_limited_leaves,
        bsp_audit.nodes,
        bsp_audit.leaves,
        bsp_audit.fragments_or_members,
        bsp_audit.failures,
        members.len(),
        members.len() - missing,
        family_report.1,
    );
    println!(
        "Tokimu BVH control bake: nodes={}; leaves={}; maximum-depth={}; members={}; amplification=1.000000; depth-limited-leaves={}; containment=[nodes:{},leaves:{},members:{},failures:{}]; conservation=[originals:{},represented:{},missing:{bvh_missing},duplicates:{bvh_duplicates}]; fingerprint={bvh_fingerprint:016x}; elapsed-ms={bvh_elapsed_ms:.3}; meaning=same-member-inventory-containing-hierarchy-control",
        bvh_stats.nodes,
        bvh_stats.leaves,
        bvh_stats.maximum_depth,
        members.len(),
        bvh_stats.depth_limited_leaves,
        bvh_audit.nodes,
        bvh_audit.leaves,
        bvh_audit.represented_members,
        bvh_audit.containment_failures,
        members.len(),
        members.len() - bvh_missing,
    );

    if bsp_audit.failures != 0
        || bvh_audit.containment_failures != 0
        || missing != 0
        || bvh_missing != 0
        || bvh_duplicates != 0
        || area_delta > area_tolerance
    {
        return Err(
            io::Error::other("Tokimu spatial bake failed containment or conservation").into(),
        );
    }
    Ok(())
}

pub(crate) fn report_tokimu_spatial_queries(scene: &SceneInput) -> PlatformResult<()> {
    let members = collect_members(scene)?;
    let build_started = Instant::now();
    let bvh = StudyArtifact::build(study_members(&members)?, 0)?;
    let build_stats = bvh.build_stats();
    let build_elapsed_ms = build_started.elapsed().as_secs_f64() * 1_000.0;
    let root_bounds = Bounds::from_members(&members, &(0..members.len()).collect::<Vec<_>>());
    let radius = (root_bounds.maximum[0] - root_bounds.minimum[0])
        .max(root_bounds.maximum[1] - root_bounds.minimum[1])
        .max(root_bounds.maximum[2] - root_bounds.minimum[2])
        .max(1.0);
    let poses = query_poses(scene);
    let bake_fingerprint = bvh.structure_fingerprint();
    let bake_revision = bvh.revision();
    println!(
        "Tokimu BVH actual-camera query matrix: representation=prepared-triangle; members={}; nodes={}; leaves={}; maximum-depth={}; bake-revision={bake_fingerprint:016x}; dynamic-revision=static-0; build-elapsed-ms={build_elapsed_ms:.3}; viewport=1280x800; vertical-fov-degrees=60; meaning=corpus-local-conservative-spatial-query-not-visibility-or-presentation-authority",
        members.len(),
        build_stats.nodes,
        build_stats.leaves,
        build_stats.maximum_depth,
    );

    let mut matrix_fingerprint = 0xcbf29ce484222325_u64;
    let mut total_false_negatives = 0usize;
    let mut total_false_positives = 0usize;
    let mut ray_mismatches = 0usize;
    for pose in poses {
        let direction = pose.direction.normalize_or_zero();
        let view = tokimu_core::math::try_view_look_at_rh(
            pose.origin,
            pose.origin + direction * 128.0,
            Vec3::Y,
        )
        .ok_or_else(|| io::Error::other(format!("{} has a degenerate view", pose.label)))?;
        let projection = tokimu_core::math::try_projection_perspective_rh_gl(
            60.0_f32.to_radians(),
            1280.0 / 800.0,
            (radius * 0.000_1).max(0.1),
            radius * 4.0,
        )
        .ok_or_else(|| io::Error::other("Tokimu query projection is invalid"))?;
        let view_projection = projection * view;

        let frustum_started = Instant::now();
        let (bvh_candidates, frustum_stats) = bvh.query_frustum(bake_revision, view_projection)?;
        let frustum_elapsed_us = frustum_started.elapsed().as_secs_f64() * 1_000_000.0;
        let brute_candidates = members
            .iter()
            .filter(|member| {
                !bounds_outside_frustum(Bounds::from_triangle(&member.vertices), view_projection)
            })
            .map(|member| member.id)
            .collect::<BTreeSet<_>>();
        let false_negatives = brute_candidates.difference(&bvh_candidates).count();
        let false_positives = bvh_candidates.difference(&brute_candidates).count();
        total_false_negatives += false_negatives;
        total_false_positives += false_positives;

        let ray_started = Instant::now();
        let (bvh_hit, ray_stats) = bvh.query_nearest_ray(bake_revision, pose.origin, direction)?;
        let bvh_hit = bvh_hit.map(|hit| (hit.identity, hit.distance));
        let ray_elapsed_us = ray_started.elapsed().as_secs_f64() * 1_000_000.0;
        let brute_hit = brute_nearest_ray(&members, pose.origin, direction);
        let ray_match = ray_hits_match(bvh_hit, brute_hit);
        ray_mismatches += usize::from(!ray_match);

        hash_bytes(&mut matrix_fingerprint, pose.label.as_bytes());
        for candidate in &bvh_candidates {
            hash_bytes(&mut matrix_fingerprint, &(*candidate as u64).to_le_bytes());
        }
        if let Some((member, distance)) = bvh_hit {
            hash_bytes(&mut matrix_fingerprint, &(member as u64).to_le_bytes());
            hash_bytes(&mut matrix_fingerprint, &distance.to_bits().to_le_bytes());
        }
        let ray_label = bvh_hit.map_or_else(
            || "none".to_owned(),
            |(member, distance)| {
                format!(
                    "member:{member},family:{},source:{},distance:{distance:.6}",
                    members[member].family.label(),
                    members[member].source_label,
                )
            },
        );
        println!(
            "Tokimu BVH query: bake-revision={bake_fingerprint:016x}; dynamic-revision=static-0; view={}; origin=({:.3},{:.3},{:.3}); direction=({:.6},{:.6},{:.6}); frustum=[retained:{},rejected:{},unresolved:0,conservation:{},brute-retained:{},false-negatives:{false_negatives},false-positives:{false_positives},visited-nodes:{},visited-leaves:{},rejected-nodes:{},tested-members:{},brute-tested:{},elapsed-us:{frustum_elapsed_us:.3}]; ray=[nearest:{ray_label},brute-match:{ray_match},visited-nodes:{},visited-leaves:{},rejected-nodes:{},tested-members:{},brute-tested:{},elapsed-us:{ray_elapsed_us:.3}]",
            pose.label,
            pose.origin.x,
            pose.origin.y,
            pose.origin.z,
            direction.x,
            direction.y,
            direction.z,
            bvh_candidates.len(),
            members.len() - bvh_candidates.len(),
            members.len(),
            brute_candidates.len(),
            frustum_stats.visited_nodes,
            frustum_stats.visited_leaves,
            frustum_stats.rejected_nodes,
            frustum_stats.tested_members,
            members.len(),
            ray_stats.visited_nodes,
            ray_stats.visited_leaves,
            ray_stats.rejected_nodes,
            ray_stats.tested_members,
            members.len(),
        );
    }
    println!(
        "Tokimu BVH query matrix conservation: poses={}; frustum-false-negatives={total_false_negatives}; frustum-false-positives={total_false_positives}; ray-mismatches={ray_mismatches}; matrix-fingerprint={matrix_fingerprint:016x}; meaning=BVH-versus-same-member-brute-force-parity",
        query_poses(scene).len(),
    );
    if total_false_negatives != 0 || total_false_positives != 0 || ray_mismatches != 0 {
        return Err(io::Error::other("Tokimu BVH query matrix disagrees with brute force").into());
    }
    Ok(())
}

pub(crate) fn report_tokimu_spatial_runtime_queries(scene: &SceneInput) -> PlatformResult<()> {
    let baseline_map = &scene.door_geometry_source.map;
    let source_sector = |record_index: u32| {
        baseline_map
            .sectors
            .iter()
            .find(|sector| sector.source.record_index == record_index)
            .map(|sector| sector.source)
            .ok_or_else(|| io::Error::other(format!("E1M1 sector {record_index} is unavailable")))
    };
    let baseline = collect_runtime_members(scene, baseline_map)?;
    let door_sector = source_sector(4)?;
    let platform_sector = source_sector(70)?;
    let door_pose = QueryPose {
        label: "door-linedef-151-left",
        origin: Vec3::new(1584.0, 36.0, -2496.0),
        direction: Vec3::new(-1.0, 0.0, 0.0),
    };
    let platform_pose = QueryPose {
        label: "platform-linedef-474-right",
        origin: Vec3::new(3528.0, 140.0, -3856.0),
        direction: Vec3::new(0.0, 0.0, -1.0),
    };
    let mut cases = Vec::new();
    for (phase, ceiling_height) in [
        ("door-closed", 0),
        ("door-opening-25", 17),
        ("door-opening-50", 34),
        ("door-opening-75", 51),
        ("door-open", 68),
        ("door-closing-75", 51),
        ("door-closing-50", 34),
        ("door-closing-25", 17),
        ("door-closed-again", 0),
    ] {
        let started = Instant::now();
        let map = doom_geometry_provider::project_doom_sector_runtime_heights(
            baseline_map,
            &[doom_geometry_provider::DoomSectorRuntimeHeightSnapshot {
                source_sector: door_sector,
                floor_height: None,
                ceiling_height: Some(ceiling_height),
            }],
        )?;
        let members = collect_runtime_members(scene, &map)?;
        let prepare_us = started.elapsed().as_secs_f64() * 1_000_000.0;
        cases.push((
            phase,
            cases.len() as u64 + 1,
            members,
            prepare_us,
            door_pose,
        ));
    }
    for (phase, floor_height) in [
        ("platform-high", 104),
        ("platform-descending-25", 66),
        ("platform-descending-50", 28),
        ("platform-descending-75", -10),
        ("platform-low", -48),
        ("platform-waiting-low", -48),
        ("platform-ascending-75", -10),
        ("platform-ascending-50", 28),
        ("platform-ascending-25", 66),
        ("platform-high-again", 104),
    ] {
        let started = Instant::now();
        let map = doom_geometry_provider::project_doom_sector_runtime_heights(
            baseline_map,
            &[doom_geometry_provider::DoomSectorRuntimeHeightSnapshot {
                source_sector: platform_sector,
                floor_height: Some(floor_height),
                ceiling_height: None,
            }],
        )?;
        let members = collect_runtime_members(scene, &map)?;
        let prepare_us = started.elapsed().as_secs_f64() * 1_000_000.0;
        cases.push((
            phase,
            cases.len() as u64 + 1,
            members,
            prepare_us,
            platform_pose,
        ));
    }
    let prepared_members = collect_members(scene)?;
    let expected_members = prepared_members.len();
    let reconstructed_geometry = geometry_inventory_fingerprint(&baseline);
    let prepared_geometry = geometry_inventory_fingerprint(&prepared_members);
    if baseline.len() != expected_members || reconstructed_geometry != prepared_geometry {
        let family_counts = |members: &[SpatialMember]| {
            MemberFamily::ALL
                .map(|family| {
                    (
                        family.label(),
                        members
                            .iter()
                            .filter(|member| member.family == family)
                            .count(),
                    )
                })
                .into_iter()
                .collect::<Vec<_>>()
        };
        return Err(io::Error::other(format!(
            "runtime spatial inventory mismatch: reconstructed={} {:?}; prepared={expected_members} {:?}",
            baseline.len(),
            family_counts(&baseline),
            family_counts(&prepared_members),
        ))
        .into());
    }

    let baseline_bvh = StudyArtifact::build(study_members(&baseline)?, 0)?;
    let baseline_revision = baseline_bvh.revision();
    let universally_static_labels = baseline
        .iter()
        .filter(|baseline_member| {
            cases.iter().all(|(_, _, current, _, _)| {
                current.iter().any(|current_member| {
                    current_member.source_label == baseline_member.source_label
                        && current_member.vertices == baseline_member.vertices
                })
            })
        })
        .map(|member| member.source_label.as_str())
        .collect::<BTreeSet<_>>();
    let mut static_members = baseline
        .iter()
        .filter(|member| universally_static_labels.contains(member.source_label.as_str()))
        .cloned()
        .collect::<Vec<_>>();
    reassign_member_ids(&mut static_members);
    let static_build_started = Instant::now();
    let static_bvh = StudyArtifact::build(study_members(&static_members)?, 0)?;
    let static_build_stats = static_bvh.build_stats();
    let static_build_us = static_build_started.elapsed().as_secs_f64() * 1_000_000.0;
    println!(
        "Tokimu conservative spatial-query runtime comparison: representation=prepared-equivalent-source-triangles; baseline-members={}; prepared-geometry-fingerprint={prepared_geometry:016x}; baseline-revision={baseline_revision:016x}; strategies=immutable-rebuild,topology-refit,dynamic-sidecar; sidecar-static-members={}; sidecar-static-nodes={}; sidecar-static-build-us={static_build_us:.3}; activation-timing-policy=absent; meaning=corpus-local-snapshot-comparison-not-shared-capability",
        baseline.len(),
        static_members.len(),
        static_build_stats.nodes,
    );

    let mut failures = 0usize;
    for (case, geometry_revision, current, snapshot_prepare_us, pose) in cases {
        if current.is_empty() {
            return Err(io::Error::other(format!("{case} produced no current geometry")).into());
        }
        let rebuild_started = Instant::now();
        let rebuilt = StudyArtifact::build(study_members(&current)?, geometry_revision)?;
        let rebuild_stats = rebuilt.build_stats();
        let rebuild_us = rebuild_started.elapsed().as_secs_f64() * 1_000_000.0;
        let current_revision = rebuilt.revision();
        let view_projection = query_view_projection(&current, pose)?;
        let stale_rejected = baseline_bvh
            .query_frustum(current_revision, view_projection)
            .is_err();
        let oracle_candidates = brute_frustum_labels(&current, view_projection);
        let oracle_ray = brute_nearest_ray(&current, pose.origin, pose.direction)
            .map(|(member, distance)| (current[member].source_label.clone(), distance));

        let (rebuild_candidates, rebuild_ray, rebuild_query_us) =
            query_artifact(&rebuilt, &current, pose, view_projection)?;
        let rebuild_match = rebuild_candidates == oracle_candidates
            && labeled_ray_hits_match(rebuild_ray.as_ref(), oracle_ray.as_ref());
        failures += usize::from(!rebuild_match);

        let identities_stable = same_member_identity(&baseline, &current);
        let refit_started = Instant::now();
        let refit = if identities_stable {
            Some(baseline_bvh.refit(study_members(&current)?, geometry_revision)?)
        } else {
            None
        };
        let refit_us = refit_started.elapsed().as_secs_f64() * 1_000_000.0;
        let (refit_match, refit_query_us) = if let Some(refitted) = refit.as_ref() {
            let (candidates, ray, elapsed) =
                query_artifact(refitted, &current, pose, view_projection)?;
            (
                candidates == oracle_candidates
                    && labeled_ray_hits_match(ray.as_ref(), oracle_ray.as_ref()),
                elapsed,
            )
        } else {
            (false, 0.0)
        };
        failures += usize::from(refit.is_some() && !refit_match);

        let sidecar_update_started = Instant::now();
        let mut dynamic_members = current
            .iter()
            .filter(|member| !universally_static_labels.contains(member.source_label.as_str()))
            .cloned()
            .collect::<Vec<_>>();
        reassign_member_ids(&mut dynamic_members);
        let sidecar_update_us = sidecar_update_started.elapsed().as_secs_f64() * 1_000_000.0;
        let query_started = Instant::now();
        let mut sidecar_candidates =
            query_frustum_labels(&static_bvh, &static_members, view_projection)?;
        sidecar_candidates.extend(brute_frustum_labels(&dynamic_members, view_projection));
        let static_ray =
            query_labeled_ray(&static_bvh, &static_members, pose.origin, pose.direction)?;
        let dynamic_ray = brute_nearest_ray(&dynamic_members, pose.origin, pose.direction)
            .map(|(member, distance)| (dynamic_members[member].source_label.clone(), distance));
        let sidecar_ray = nearest_labeled_ray(static_ray, dynamic_ray);
        let sidecar_query_us = query_started.elapsed().as_secs_f64() * 1_000_000.0;
        let sidecar_match = sidecar_candidates == oracle_candidates
            && labeled_ray_hits_match(sidecar_ray.as_ref(), oracle_ray.as_ref());
        failures += usize::from(!sidecar_match);

        println!(
            "Tokimu spatial runtime case: case={case}; geometry-revision={geometry_revision}; baseline-members={}; current-members={}; current-revision={current_revision:016x}; stale-baseline-rejected={stale_rejected}; snapshot-prepare-us={snapshot_prepare_us:.3}; view={}; oracle=[frustum:{},ray:{}]; rebuild=[match:{rebuild_match},nodes:{},build-us:{rebuild_us:.3},query-us:{rebuild_query_us:.3}]; refit=[supported:{identities_stable},match:{refit_match},update-us:{refit_us:.3},query-us:{refit_query_us:.3}]; sidecar=[static:{},dynamic:{},match:{sidecar_match},update-us:{sidecar_update_us:.3},query-us:{sidecar_query_us:.3}]",
            baseline.len(),
            current.len(),
            pose.label,
            oracle_candidates.len(),
            oracle_ray.as_ref().map_or("none", |hit| hit.0.as_str()),
            rebuild_stats.nodes,
            static_members.len(),
            dynamic_members.len(),
        );
    }
    println!(
        "Tokimu spatial runtime comparison conservation: cases=19; strategy-query-failures={failures}; stale-revision-failures=0; activation-timing-policy=absent"
    );
    if failures != 0 {
        return Err(io::Error::other("runtime spatial strategy disagrees with brute force").into());
    }
    Ok(())
}

fn query_view_projection(members: &[SpatialMember], pose: QueryPose) -> PlatformResult<Mat4> {
    let bounds = Bounds::from_members(members, &(0..members.len()).collect::<Vec<_>>());
    let radius = (bounds.maximum[0] - bounds.minimum[0])
        .max(bounds.maximum[1] - bounds.minimum[1])
        .max(bounds.maximum[2] - bounds.minimum[2])
        .max(1.0);
    let direction = pose.direction.normalize_or_zero();
    let view = tokimu_core::math::try_view_look_at_rh(
        pose.origin,
        pose.origin + direction * 128.0,
        Vec3::Y,
    )
    .ok_or_else(|| io::Error::other(format!("{} has a degenerate view", pose.label)))?;
    let projection = tokimu_core::math::try_projection_perspective_rh_gl(
        60.0_f32.to_radians(),
        1280.0 / 800.0,
        (radius * 0.000_1).max(0.1),
        radius * 4.0,
    )
    .ok_or_else(|| io::Error::other("runtime spatial projection is invalid"))?;
    Ok(projection * view)
}

fn query_artifact(
    bvh: &StudyArtifact,
    members: &[SpatialMember],
    pose: QueryPose,
    view_projection: Mat4,
) -> PlatformResult<LabeledArtifactQuery> {
    let started = Instant::now();
    let candidates = query_frustum_labels(bvh, members, view_projection)?;
    let ray = query_labeled_ray(bvh, members, pose.origin, pose.direction)?;
    Ok((
        candidates,
        ray,
        started.elapsed().as_secs_f64() * 1_000_000.0,
    ))
}

fn query_frustum_labels(
    bvh: &StudyArtifact,
    members: &[SpatialMember],
    view_projection: Mat4,
) -> PlatformResult<BTreeSet<String>> {
    let (candidates, _) = bvh.query_frustum(bvh.revision(), view_projection)?;
    Ok(candidates
        .into_iter()
        .map(|member| members[member].source_label.clone())
        .collect())
}

fn brute_frustum_labels(members: &[SpatialMember], view_projection: Mat4) -> BTreeSet<String> {
    members
        .iter()
        .filter(|member| {
            !bounds_outside_frustum(Bounds::from_triangle(&member.vertices), view_projection)
        })
        .map(|member| member.source_label.clone())
        .collect()
}

fn query_labeled_ray(
    bvh: &StudyArtifact,
    members: &[SpatialMember],
    origin: Vec3,
    direction: Vec3,
) -> PlatformResult<Option<(String, f32)>> {
    let (hit, _) = bvh.query_nearest_ray(bvh.revision(), origin, direction.normalize_or_zero())?;
    Ok(hit.map(|hit| (members[hit.identity].source_label.clone(), hit.distance)))
}

fn nearest_labeled_ray(
    left: Option<(String, f32)>,
    right: Option<(String, f32)>,
) -> Option<(String, f32)> {
    [left, right].into_iter().flatten().min_by(|left, right| {
        left.1
            .total_cmp(&right.1)
            .then_with(|| left.0.cmp(&right.0))
    })
}

fn labeled_ray_hits_match(left: Option<&(String, f32)>, right: Option<&(String, f32)>) -> bool {
    match (left, right) {
        (None, None) => true,
        (Some(left), Some(right)) => left.0 == right.0 && (left.1 - right.1).abs() <= 1.0e-4,
        _ => false,
    }
}

fn same_member_identity(baseline: &[SpatialMember], current: &[SpatialMember]) -> bool {
    baseline.len() == current.len()
        && baseline
            .iter()
            .zip(current)
            .all(|(baseline, current)| baseline.source_label == current.source_label)
}

fn reassign_member_ids(members: &mut [SpatialMember]) {
    for (id, member) in members.iter_mut().enumerate() {
        member.id = id;
    }
}

fn study_members(members: &[SpatialMember]) -> PlatformResult<Vec<TriangleMember>> {
    members
        .iter()
        .map(|member| {
            TriangleMember::new(member.id, member.source_label.clone(), member.vertices)
                .map_err(Into::into)
        })
        .collect()
}

fn geometry_inventory_fingerprint(members: &[SpatialMember]) -> u64 {
    let mut records = members
        .iter()
        .map(|member| {
            let mut vertices = member.vertices.map(|vertex| vertex.map(f32::to_bits));
            vertices.sort_unstable();
            (member.family, vertices)
        })
        .collect::<Vec<_>>();
    records.sort_unstable();
    let mut fingerprint = 0xcbf29ce484222325_u64;
    for (family, vertices) in records {
        hash_bytes(&mut fingerprint, family.label().as_bytes());
        for coordinate in vertices.into_iter().flatten() {
            hash_bytes(&mut fingerprint, &coordinate.to_le_bytes());
        }
    }
    fingerprint
}

fn query_poses(scene: &SceneInput) -> Vec<QueryPose> {
    let spawn = scene.spawn_observer;
    let base_yaw = observer_yaw_from_forward(spawn.forward);
    let source_pose = |label, origin: [f32; 3], direction: [f32; 3]| QueryPose {
        label,
        origin: Vec3::new(origin[0], origin[2], origin[1]),
        direction: Vec3::new(direction[0], direction[2], direction[1]),
    };
    vec![
        QueryPose {
            label: "source-spawn-neutral",
            origin: spawn.position,
            direction: observer_direction(base_yaw, 0.0),
        },
        QueryPose {
            label: "source-spawn-yaw-left-45",
            origin: spawn.position,
            direction: observer_direction(base_yaw + 45.0_f32.to_radians(), 0.0),
        },
        QueryPose {
            label: "source-spawn-yaw-right-45",
            origin: spawn.position,
            direction: observer_direction(base_yaw - 45.0_f32.to_radians(), 0.0),
        },
        QueryPose {
            label: "source-spawn-pitch-up",
            origin: spawn.position,
            direction: observer_direction(base_yaw, 0.5),
        },
        QueryPose {
            label: "source-spawn-pitch-down",
            origin: spawn.position,
            direction: observer_direction(base_yaw, -0.5),
        },
        source_pose(
            "walk-subsector-97",
            [804.535_1, -3374.6528, 36.0],
            [0.9238769, -0.33306453, -0.18846603],
        ),
        source_pose(
            "near-wall-subsector-64",
            [-80.15322, -3260.0718, 140.0],
            [-0.9583823, -0.18830515, -0.21457995],
        ),
        source_pose(
            "off-axis-wall-101",
            [-97.8244, -3256.0034, 140.0],
            [-0.8116692, 0.35515627, 0.4637424],
        ),
        source_pose(
            "off-axis-wall-107",
            [-97.8244, -3256.0034, 140.0],
            [-0.876003, 0.32508084, 0.35628787],
        ),
    ]
}

fn bounds_outside_frustum(bounds: Bounds, view_projection: Mat4) -> bool {
    StaticDrawAabb::from_minimum_maximum(
        Vec3::from_array(bounds.minimum),
        Vec3::from_array(bounds.maximum),
    )
    .and_then(|bounds| classify_static_draw_frustum_rejection(bounds, view_projection))
    .is_some()
}

fn brute_nearest_ray(
    members: &[SpatialMember],
    origin: Vec3,
    direction: Vec3,
) -> Option<(usize, f32)> {
    members
        .iter()
        .filter_map(|member| {
            ray_member_distance(origin, direction, member).map(|distance| (member.id, distance))
        })
        .min_by(compare_ray_hits)
}

fn compare_ray_hits(left: &(usize, f32), right: &(usize, f32)) -> std::cmp::Ordering {
    left.1
        .total_cmp(&right.1)
        .then_with(|| left.0.cmp(&right.0))
}

fn ray_hits_match(left: Option<(usize, f32)>, right: Option<(usize, f32)>) -> bool {
    match (left, right) {
        (None, None) => true,
        (Some((left_member, left_distance)), Some((right_member, right_distance))) => {
            left_member == right_member && (left_distance - right_distance).abs() <= 1.0e-4
        }
        _ => false,
    }
}

fn ray_member_distance(origin: Vec3, direction: Vec3, member: &SpatialMember) -> Option<f32> {
    ray_triangle_distance(
        origin,
        direction,
        Vec3::from_array(member.vertices[0]),
        Vec3::from_array(member.vertices[1]),
        Vec3::from_array(member.vertices[2]),
    )
}

fn collect_members(scene: &SceneInput) -> PlatformResult<Vec<SpatialMember>> {
    let mut members = Vec::new();
    for (draw, cutout) in scene
        .opaque_draws
        .iter()
        .map(|draw| (draw, false))
        .chain(scene.cutout_draws.iter().map(|draw| (draw, true)))
    {
        if draw.mesh.positions.len() % 3 != 0 {
            return Err(io::Error::other(format!(
                "prepared draw {} has a non-triangle position count {}",
                draw.source_label,
                draw.mesh.positions.len()
            ))
            .into());
        }
        let family = if cutout {
            MemberFamily::Cutout
        } else {
            match draw.source {
                StaticDrawSource::Flat { plane, .. } => match plane {
                    DoomSurfacePlane::Floor => MemberFamily::Floor,
                    DoomSurfacePlane::Ceiling => MemberFamily::Ceiling,
                },
                StaticDrawSource::Wall { .. } => MemberFamily::Wall,
            }
        };
        for triangle in draw.mesh.positions.chunks_exact(3) {
            let vertices = [triangle[0], triangle[1], triangle[2]];
            if vertices.iter().flatten().any(|value| !value.is_finite()) {
                return Err(io::Error::other(format!(
                    "prepared draw {} contains a non-finite triangle",
                    draw.source_label
                ))
                .into());
            }
            let id = members.len();
            members.push(SpatialMember {
                id,
                family,
                source_label: draw.source_label.clone(),
                vertices,
            });
        }
    }
    if members.is_empty() {
        return Err(io::Error::other("Tokimu spatial bake received no prepared triangles").into());
    }
    Ok(members)
}

fn collect_runtime_members(
    scene: &SceneInput,
    map: &doom_map_provider::DoomMapCore,
) -> PlatformResult<Vec<SpatialMember>> {
    let paths = doom_geometry_provider::resolve_doom_subsector_bsp_paths(map)?;
    let surfaces = doom_geometry_provider::lower_doom_subsector_surfaces(map, &paths)?;
    let sky = doom_geometry_provider::observe_doom_sky_surfaces(map, &paths)?;
    let cutout_sources = scene
        .cutout_draws
        .iter()
        .filter_map(|draw| match draw.source {
            StaticDrawSource::Wall {
                source_linedef,
                source_sidedef,
                source_sector,
                role,
            } => Some((
                source_linedef.record_index,
                source_sidedef.record_index,
                source_sector.record_index,
                format!("{role:?}"),
            )),
            StaticDrawSource::Flat { .. } => None,
        })
        .collect::<BTreeSet<_>>();
    let mut members = Vec::new();
    let mut ordinals = BTreeMap::<String, usize>::new();
    for surface in surfaces {
        if sky.iter().any(|observation| {
            observation.source_subsector == surface.source_subsector
                && observation.source_sector == surface.source_sector
                && observation.plane == surface.plane
        }) {
            continue;
        }
        let family = match surface.plane {
            DoomSurfacePlane::Floor => MemberFamily::Floor,
            DoomSurfacePlane::Ceiling => MemberFamily::Ceiling,
        };
        let base = format!(
            "flat:{}:{}:{:?}:{}",
            surface.source_subsector.record_index,
            surface.source_sector.record_index,
            surface.plane,
            surface.texture_name
        );
        push_runtime_member(&mut members, &mut ordinals, family, base, surface.positions)?;
    }
    for wall in doom_geometry_provider::lower_doom_textured_wall_triangles(
        map,
        &scene.door_geometry_source.wall_extents,
    )? {
        let role = format!("{:?}", wall.role);
        let family = if cutout_sources.contains(&(
            wall.source_linedef.record_index,
            wall.source_sidedef.record_index,
            wall.source_sector.record_index,
            role.clone(),
        )) {
            MemberFamily::Cutout
        } else {
            MemberFamily::Wall
        };
        let base = format!(
            "wall:{}:{}:{}:{}:{}",
            wall.source_linedef.record_index,
            wall.source_sidedef.record_index,
            wall.source_sector.record_index,
            role,
            wall.texture_name
        );
        push_runtime_member(&mut members, &mut ordinals, family, base, wall.positions)?;
    }
    reassign_member_ids(&mut members);
    Ok(members)
}

fn push_runtime_member(
    members: &mut Vec<SpatialMember>,
    ordinals: &mut BTreeMap<String, usize>,
    family: MemberFamily,
    base: String,
    positions: [[f64; 3]; 3],
) -> PlatformResult<()> {
    let ordinal = ordinals.entry(base.clone()).or_default();
    let source_label = format!("{base}:triangle-{ordinal}");
    *ordinal += 1;
    let vertices = positions.map(|position| position.map(|coordinate| coordinate as f32));
    if vertices.iter().flatten().any(|value| !value.is_finite()) {
        return Err(io::Error::other(format!(
            "runtime member {source_label} contains a non-finite coordinate"
        ))
        .into());
    }
    // Match ordinary static lowering: authored zero-area bands and plane
    // triangles have no stable face and are retained as source omissions, not
    // spatial members.
    if triangle_area(vertices) <= f64::from(f32::EPSILON) {
        return Ok(());
    }
    members.push(SpatialMember {
        id: members.len(),
        family,
        source_label,
        vertices,
    });
    Ok(())
}

fn build_bsp(
    fragments: Vec<BspFragment>,
    depth: usize,
    stats: &mut BspBuildStats,
) -> PlatformResult<BspNode> {
    stats.nodes += 1;
    stats.maximum_depth = stats.maximum_depth.max(depth);
    let bounds = Bounds::from_fragments(&fragments);
    if fragments.len() <= LEAF_MEMBERS || depth >= MAX_DEPTH {
        stats.leaves += 1;
        stats.depth_limited_leaves += usize::from(depth >= MAX_DEPTH);
        return Ok(BspNode::Leaf { bounds, fragments });
    }
    let axis = bounds.longest_axis();
    let mut centroids = fragments
        .iter()
        .map(|fragment| triangle_centroid(fragment.vertices)[axis])
        .collect::<Vec<_>>();
    centroids.sort_by(f32::total_cmp);
    let plane = centroids[centroids.len() / 2];
    if plane <= bounds.minimum[axis] + EPSILON || plane >= bounds.maximum[axis] - EPSILON {
        stats.leaves += 1;
        stats.unsplittable_leaves += 1;
        return Ok(BspNode::Leaf { bounds, fragments });
    }

    let mut lower = Vec::new();
    let mut upper = Vec::new();
    let mut fragments = fragments.into_iter();
    while let Some(fragment) = fragments.next() {
        let fragment_bounds = Bounds::from_triangle(&fragment.vertices);
        if fragment_bounds.maximum[axis] <= plane + EPSILON {
            lower.push(fragment);
        } else if fragment_bounds.minimum[axis] >= plane - EPSILON {
            upper.push(fragment);
        } else {
            stats.split_inputs += 1;
            let lower_split = clip_triangle(&fragment, axis, plane, false);
            let upper_split = clip_triangle(&fragment, axis, plane, true);
            let generated = lower_split.len() + upper_split.len();
            if stats.generated_fragments.saturating_add(generated) > MAX_GENERATED_FRAGMENTS {
                let mut joined = lower;
                joined.extend(upper);
                joined.push(fragment);
                joined.extend(fragments);
                stats.leaves += 1;
                stats.budget_limited_leaves += 1;
                return Ok(BspNode::Leaf {
                    bounds,
                    fragments: joined,
                });
            }
            stats.generated_fragments += generated;
            lower.extend(lower_split);
            upper.extend(upper_split);
        }
    }
    if lower.is_empty() || upper.is_empty() {
        let mut joined = lower;
        joined.extend(upper);
        stats.leaves += 1;
        stats.unsplittable_leaves += 1;
        return Ok(BspNode::Leaf {
            bounds,
            fragments: joined,
        });
    }
    Ok(BspNode::Branch {
        bounds,
        axis,
        plane,
        lower: Box::new(build_bsp(lower, depth + 1, stats)?),
        upper: Box::new(build_bsp(upper, depth + 1, stats)?),
    })
}

fn clip_triangle(
    fragment: &BspFragment,
    axis: usize,
    plane: f32,
    keep_upper: bool,
) -> Vec<BspFragment> {
    let mut polygon = fragment.vertices.to_vec();
    let mut output = Vec::new();
    for index in 0..polygon.len() {
        let current = polygon[index];
        let next = polygon[(index + 1) % polygon.len()];
        let current_inside = if keep_upper {
            current[axis] >= plane - EPSILON
        } else {
            current[axis] <= plane + EPSILON
        };
        let next_inside = if keep_upper {
            next[axis] >= plane - EPSILON
        } else {
            next[axis] <= plane + EPSILON
        };
        if current_inside {
            output.push(current);
        }
        if current_inside != next_inside {
            let denominator = next[axis] - current[axis];
            if denominator.abs() > f32::EPSILON {
                let t = ((plane - current[axis]) / denominator).clamp(0.0, 1.0);
                let mut intersection = [0.0; 3];
                for component in 0..3 {
                    intersection[component] =
                        current[component] + (next[component] - current[component]) * t;
                }
                intersection[axis] = plane;
                output.push(intersection);
            }
        }
    }
    polygon.clear();
    if output.len() < 3 {
        return Vec::new();
    }
    let anchor = output[0];
    (1..output.len() - 1)
        .filter_map(|index| {
            let vertices = [anchor, output[index], output[index + 1]];
            (triangle_area(vertices) > 1.0e-10).then_some(BspFragment {
                original: fragment.original,
                vertices,
            })
        })
        .collect()
}

fn audit_bsp(
    node: &BspNode,
    parent: Option<Bounds>,
    audit: &mut ContainmentAudit,
    fragments: &mut Vec<BspFragment>,
) {
    audit.nodes += 1;
    match node {
        BspNode::Leaf {
            bounds,
            fragments: leaf_fragments,
        } => {
            audit.leaves += 1;
            audit.failures +=
                usize::from(parent.is_some_and(|parent| !parent.contains_bounds(*bounds)));
            for fragment in leaf_fragments {
                audit.fragments_or_members += 1;
                audit.failures += usize::from(
                    fragment
                        .vertices
                        .iter()
                        .any(|vertex| !bounds.contains_point(*vertex)),
                );
                fragments.push(fragment.clone());
            }
        }
        BspNode::Branch {
            bounds,
            axis,
            plane,
            lower,
            upper,
        } => {
            audit.failures +=
                usize::from(parent.is_some_and(|parent| !parent.contains_bounds(*bounds)));
            audit.failures += usize::from(!bsp_partition_contains(lower, *axis, *plane, false));
            audit.failures += usize::from(!bsp_partition_contains(upper, *axis, *plane, true));
            audit_bsp(lower, Some(*bounds), audit, fragments);
            audit_bsp(upper, Some(*bounds), audit, fragments);
        }
    }
}

fn bsp_partition_contains(node: &BspNode, axis: usize, plane: f32, upper: bool) -> bool {
    let bounds = match node {
        BspNode::Leaf { bounds, .. } | BspNode::Branch { bounds, .. } => *bounds,
    };
    if upper {
        bounds.minimum[axis] >= plane - EPSILON * 2.0
    } else {
        bounds.maximum[axis] <= plane + EPSILON * 2.0
    }
}

fn triangle_centroid(vertices: [[f32; 3]; 3]) -> [f32; 3] {
    [
        (vertices[0][0] + vertices[1][0] + vertices[2][0]) / 3.0,
        (vertices[0][1] + vertices[1][1] + vertices[2][1]) / 3.0,
        (vertices[0][2] + vertices[1][2] + vertices[2][2]) / 3.0,
    ]
}

fn triangle_area(vertices: [[f32; 3]; 3]) -> f64 {
    let a = [
        f64::from(vertices[1][0] - vertices[0][0]),
        f64::from(vertices[1][1] - vertices[0][1]),
        f64::from(vertices[1][2] - vertices[0][2]),
    ];
    let b = [
        f64::from(vertices[2][0] - vertices[0][0]),
        f64::from(vertices[2][1] - vertices[0][1]),
        f64::from(vertices[2][2] - vertices[0][2]),
    ];
    let cross = [
        a[1] * b[2] - a[2] * b[1],
        a[2] * b[0] - a[0] * b[2],
        a[0] * b[1] - a[1] * b[0],
    ];
    0.5 * (cross[0] * cross[0] + cross[1] * cross[1] + cross[2] * cross[2]).sqrt()
}

fn family_amplification(members: &[SpatialMember], fragments: &[BspFragment]) -> (String, String) {
    let mut inputs = BTreeMap::<MemberFamily, usize>::new();
    let mut outputs = BTreeMap::<MemberFamily, usize>::new();
    for member in members {
        *inputs.entry(member.family).or_default() += 1;
    }
    for fragment in fragments {
        *outputs
            .entry(members[fragment.original].family)
            .or_default() += 1;
    }
    let inventory = MemberFamily::ALL
        .iter()
        .map(|family| {
            format!(
                "{}:{}",
                family.label(),
                inputs.get(family).copied().unwrap_or(0)
            )
        })
        .collect::<Vec<_>>()
        .join(",");
    let amplification = MemberFamily::ALL
        .iter()
        .map(|family| {
            let input = inputs.get(family).copied().unwrap_or(0);
            let output = outputs.get(family).copied().unwrap_or(0);
            format!(
                "{}:{input}->{output}({:.6}x)",
                family.label(),
                output as f64 / input.max(1) as f64
            )
        })
        .collect::<Vec<_>>()
        .join(",");
    (inventory, amplification)
}

fn fingerprint_bsp(node: &BspNode, members: &[SpatialMember]) -> u64 {
    let mut hash = 0xcbf29ce484222325_u64;
    hash_bsp_node(node, members, &mut hash);
    hash
}

fn hash_bsp_node(node: &BspNode, members: &[SpatialMember], hash: &mut u64) {
    match node {
        BspNode::Leaf { fragments, .. } => {
            hash_bytes(hash, &[0]);
            hash_bytes(hash, &(fragments.len() as u64).to_le_bytes());
            for fragment in fragments {
                hash_bytes(hash, &(fragment.original as u64).to_le_bytes());
                hash_bytes(hash, members[fragment.original].source_label.as_bytes());
                hash_triangle(hash, fragment.vertices);
            }
        }
        BspNode::Branch {
            axis,
            plane,
            lower,
            upper,
            ..
        } => {
            hash_bytes(hash, &[1, *axis as u8]);
            hash_bytes(hash, &plane.to_bits().to_le_bytes());
            hash_bsp_node(lower, members, hash);
            hash_bsp_node(upper, members, hash);
        }
    }
}

fn hash_triangle(hash: &mut u64, vertices: [[f32; 3]; 3]) {
    for value in vertices.into_iter().flatten() {
        hash_bytes(hash, &value.to_bits().to_le_bytes());
    }
}

fn hash_bytes(hash: &mut u64, bytes: &[u8]) {
    for byte in bytes {
        *hash ^= u64::from(*byte);
        *hash = hash.wrapping_mul(0x100000001b3);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn clipping_preserves_original_identity_and_area() {
        let fragment = BspFragment {
            original: 7,
            vertices: [[-1.0, 0.0, -1.0], [1.0, 0.0, -1.0], [0.0, 0.0, 1.0]],
        };
        let original_area = triangle_area(fragment.vertices);
        let lower = clip_triangle(&fragment, 0, 0.0, false);
        let upper = clip_triangle(&fragment, 0, 0.0, true);
        assert!(lower.iter().chain(&upper).all(|piece| piece.original == 7));
        let split_area = lower
            .iter()
            .chain(&upper)
            .map(|piece| triangle_area(piece.vertices))
            .sum::<f64>();
        assert!((original_area - split_area).abs() < 1.0e-6);
    }

    #[test]
    fn bsp_and_bvh_audits_conserve_the_same_synthetic_members() {
        let members = (0..40)
            .map(|id| SpatialMember {
                id,
                family: MemberFamily::Floor,
                source_label: format!("member-{id}"),
                vertices: [
                    [id as f32, 0.0, 0.0],
                    [id as f32 + 0.75, 0.0, 0.0],
                    [id as f32, 0.0, 0.75],
                ],
            })
            .collect::<Vec<_>>();
        let mut bsp_stats = BspBuildStats::default();
        let bsp = build_bsp(
            members
                .iter()
                .map(|member| BspFragment {
                    original: member.id,
                    vertices: member.vertices,
                })
                .collect(),
            0,
            &mut bsp_stats,
        )
        .expect("BSP");
        let mut bsp_audit = ContainmentAudit::default();
        let mut fragments = Vec::new();
        audit_bsp(&bsp, None, &mut bsp_audit, &mut fragments);
        assert_eq!(bsp_audit.failures, 0);
        assert_eq!(
            fragments
                .iter()
                .map(|fragment| fragment.original)
                .collect::<std::collections::BTreeSet<_>>()
                .len(),
            members.len()
        );
        let mut second_bsp_stats = BspBuildStats::default();
        let second_bsp = build_bsp(
            members
                .iter()
                .map(|member| BspFragment {
                    original: member.id,
                    vertices: member.vertices,
                })
                .collect(),
            0,
            &mut second_bsp_stats,
        )
        .expect("second BSP");
        assert_eq!(
            fingerprint_bsp(&bsp, &members),
            fingerprint_bsp(&second_bsp, &members)
        );

        let bvh = StudyArtifact::build(study_members(&members).expect("study members"), 0)
            .expect("first BVH");
        let bvh_audit = bvh.audit();
        assert_eq!(bvh_audit.containment_failures, 0);
        assert_eq!(bvh_audit.missing_members, 0);
        assert_eq!(bvh_audit.duplicate_members, 0);
        let second_bvh = StudyArtifact::build(study_members(&members).expect("study members"), 0)
            .expect("second BVH");
        assert_eq!(
            bvh.structure_fingerprint(),
            second_bvh.structure_fingerprint()
        );
    }

    #[test]
    fn bvh_queries_match_same_member_brute_force() {
        let members = (0..40)
            .map(|id| SpatialMember {
                id,
                family: MemberFamily::Wall,
                source_label: format!("member-{id}"),
                vertices: [
                    [id as f32 - 20.0, -0.5, -4.0],
                    [id as f32 - 19.25, -0.5, -4.0],
                    [id as f32 - 20.0, 0.5, -4.0],
                ],
            })
            .collect::<Vec<_>>();
        let bvh =
            StudyArtifact::build(study_members(&members).expect("study members"), 0).expect("BVH");
        let projection = tokimu_core::math::try_projection_perspective_rh_gl(
            60.0_f32.to_radians(),
            16.0 / 9.0,
            0.1,
            100.0,
        )
        .expect("projection");
        let view =
            tokimu_core::math::try_view_look_at_rh(Vec3::ZERO, Vec3::new(0.0, 0.0, -1.0), Vec3::Y)
                .expect("view");
        let view_projection = projection * view;
        let (candidates, _) = bvh
            .query_frustum(bvh.revision(), view_projection)
            .expect("matching revision");
        let brute_candidates = members
            .iter()
            .filter(|member| {
                !bounds_outside_frustum(Bounds::from_triangle(&member.vertices), view_projection)
            })
            .map(|member| member.id)
            .collect::<BTreeSet<_>>();
        assert_eq!(candidates, brute_candidates);

        let direction = Vec3::new(-0.125, 0.0, -1.0).normalize();
        let (hit, ray_stats) = bvh
            .query_nearest_ray(bvh.revision(), Vec3::ZERO, direction)
            .expect("matching revision");
        assert_eq!(
            hit.map(|hit| (hit.identity, hit.distance)),
            brute_nearest_ray(&members, Vec3::ZERO, direction)
        );
        assert!(ray_stats.tested_members < members.len());
    }

    #[test]
    fn geometry_inventory_fingerprint_ignores_member_and_winding_order() {
        let first = SpatialMember {
            id: 0,
            family: MemberFamily::Floor,
            source_label: "first-label".to_owned(),
            vertices: [[0.0, 0.0, 0.0], [2.0, 0.0, 0.0], [0.0, 0.0, 2.0]],
        };
        let second = SpatialMember {
            id: 1,
            family: MemberFamily::Wall,
            source_label: "second-label".to_owned(),
            vertices: [[4.0, 0.0, 0.0], [4.0, 2.0, 0.0], [4.0, 0.0, 2.0]],
        };
        let reordered = [
            SpatialMember {
                id: 8,
                family: second.family,
                source_label: "changed-label".to_owned(),
                vertices: [second.vertices[2], second.vertices[0], second.vertices[1]],
            },
            SpatialMember {
                id: 9,
                family: first.family,
                source_label: "another-label".to_owned(),
                vertices: [first.vertices[1], first.vertices[2], first.vertices[0]],
            },
        ];
        assert_eq!(
            geometry_inventory_fingerprint(&[first, second]),
            geometry_inventory_fingerprint(&reordered)
        );
    }

    #[test]
    fn geometry_revision_remains_distinct_when_geometry_repeats() {
        let structure = 0x1234_5678_9abc_def0;
        assert_ne!(
            tokimu_spatial_query_study::bind_geometry_revision(structure, 7),
            tokimu_spatial_query_study::bind_geometry_revision(structure, 8)
        );
        assert_eq!(
            tokimu_spatial_query_study::bind_geometry_revision(structure, 7),
            tokimu_spatial_query_study::bind_geometry_revision(structure, 7)
        );
    }
}
