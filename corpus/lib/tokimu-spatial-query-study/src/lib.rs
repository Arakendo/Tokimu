//! Portable, corpus-local conservative spatial-query mechanics.
//!
//! This is experimental evidence, not an admitted Tokimu capability. It owns
//! no visibility or presentation decisions; callers decide what candidate
//! query results mean.

use std::{cmp::Ordering, collections::BTreeSet, error::Error, fmt};

use tokimu_core::math::{Mat4, Vec3, Vec4};

const LEAF_MEMBERS: usize = 24;
const MAX_DEPTH: usize = 20;
const EPSILON: f32 = 1.0e-4;
const HASH_OFFSET: u64 = 0xcbf29ce484222325;

/// One exact finite triangle with caller-owned stable correlation.
#[derive(Clone, Debug, PartialEq)]
pub struct TriangleMember {
    pub identity: usize,
    pub correlation: String,
    pub vertices: [[f32; 3]; 3],
}

impl TriangleMember {
    pub fn new(
        identity: usize,
        correlation: impl Into<String>,
        vertices: [[f32; 3]; 3],
    ) -> Result<Self, BuildError> {
        if vertices.into_iter().flatten().all(f32::is_finite) {
            Ok(Self {
                identity,
                correlation: correlation.into(),
                vertices,
            })
        } else {
            Err(BuildError::NonFiniteMember { identity })
        }
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct BuildStats {
    pub nodes: usize,
    pub leaves: usize,
    pub maximum_depth: usize,
    pub depth_limited_leaves: usize,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct QueryStats {
    pub visited_nodes: usize,
    pub visited_leaves: usize,
    pub rejected_nodes: usize,
    pub tested_members: usize,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct Audit {
    pub nodes: usize,
    pub leaves: usize,
    pub represented_members: usize,
    pub missing_members: usize,
    pub duplicate_members: usize,
    pub containment_failures: usize,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct RayHit {
    pub identity: usize,
    pub distance: f32,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum BuildError {
    EmptyInventory,
    NonFiniteMember { identity: usize },
    DuplicateIdentity { identity: usize },
    IdentityOrderMismatch { expected: usize, actual: usize },
    MemberCorrelationMismatch { identity: usize },
}

impl fmt::Display for BuildError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EmptyInventory => formatter.write_str("triangle inventory is empty"),
            Self::NonFiniteMember { identity } => {
                write!(
                    formatter,
                    "member {identity} contains a non-finite coordinate"
                )
            }
            Self::DuplicateIdentity { identity } => {
                write!(
                    formatter,
                    "member identity {identity} occurs more than once"
                )
            }
            Self::IdentityOrderMismatch { expected, actual } => write!(
                formatter,
                "dense study identity order mismatch: expected {expected}, found {actual}"
            ),
            Self::MemberCorrelationMismatch { identity } => write!(
                formatter,
                "member {identity} does not retain its artifact correlation"
            ),
        }
    }
}

impl Error for BuildError {}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct RevisionMismatch {
    pub artifact_revision: u64,
    pub requested_revision: u64,
}

impl fmt::Display for RevisionMismatch {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "artifact revision {:016x} does not match requested revision {:016x}",
            self.artifact_revision, self.requested_revision
        )
    }
}

impl Error for RevisionMismatch {}

#[derive(Clone, Copy, Debug)]
struct Bounds {
    minimum: [f32; 3],
    maximum: [f32; 3],
}

impl Bounds {
    fn from_triangle(vertices: [[f32; 3]; 3]) -> Self {
        let mut bounds = Self {
            minimum: vertices[0],
            maximum: vertices[0],
        };
        for point in &vertices[1..] {
            bounds.include_point(*point);
        }
        bounds
    }

    fn from_members(members: &[TriangleMember], identities: &[usize]) -> Self {
        let mut bounds = Self::from_triangle(members[identities[0]].vertices);
        for identity in &identities[1..] {
            bounds.include(Self::from_triangle(members[*identity].vertices));
        }
        bounds
    }

    fn include_point(&mut self, point: [f32; 3]) {
        for (axis, coordinate) in point.into_iter().enumerate() {
            self.minimum[axis] = self.minimum[axis].min(coordinate);
            self.maximum[axis] = self.maximum[axis].max(coordinate);
        }
    }

    fn include(&mut self, other: Self) {
        self.include_point(other.minimum);
        self.include_point(other.maximum);
    }

    fn contains_point(self, point: [f32; 3]) -> bool {
        (0..3).all(|axis| {
            point[axis] >= self.minimum[axis] - EPSILON
                && point[axis] <= self.maximum[axis] + EPSILON
        })
    }

    fn contains(self, other: Self) -> bool {
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

    fn corners(self) -> [Vec3; 8] {
        let min = Vec3::from_array(self.minimum);
        let max = Vec3::from_array(self.maximum);
        [
            Vec3::new(min.x, min.y, min.z),
            Vec3::new(max.x, min.y, min.z),
            Vec3::new(min.x, max.y, min.z),
            Vec3::new(max.x, max.y, min.z),
            Vec3::new(min.x, min.y, max.z),
            Vec3::new(max.x, min.y, max.z),
            Vec3::new(min.x, max.y, max.z),
            Vec3::new(max.x, max.y, max.z),
        ]
    }
}

#[derive(Clone, Debug)]
enum Node {
    Leaf {
        bounds: Bounds,
        members: Vec<usize>,
    },
    Branch {
        bounds: Bounds,
        lower: Box<Node>,
        upper: Box<Node>,
    },
}

impl Node {
    fn bounds(&self) -> Bounds {
        match self {
            Self::Leaf { bounds, .. } | Self::Branch { bounds, .. } => *bounds,
        }
    }
}

/// Immutable study artifact. A query must present its expected revision.
#[derive(Clone, Debug)]
pub struct Artifact {
    members: Vec<TriangleMember>,
    root: Node,
    stats: BuildStats,
    structure_fingerprint: u64,
    revision: u64,
}

impl Artifact {
    pub fn build(members: Vec<TriangleMember>, geometry_revision: u64) -> Result<Self, BuildError> {
        validate_members(&members)?;
        let mut stats = BuildStats::default();
        let root = build_node(&members, (0..members.len()).collect(), 0, &mut stats);
        let structure_fingerprint = fingerprint_node(&root, &members);
        Ok(Self {
            members,
            root,
            stats,
            structure_fingerprint,
            revision: bind_geometry_revision(structure_fingerprint, geometry_revision),
        })
    }

    pub fn refit(
        &self,
        members: Vec<TriangleMember>,
        geometry_revision: u64,
    ) -> Result<Self, BuildError> {
        validate_members(&members)?;
        if self.members.len() != members.len() {
            return Err(BuildError::IdentityOrderMismatch {
                expected: self.members.len(),
                actual: members.len(),
            });
        }
        if let Some((left, _)) = self.members.iter().zip(&members).find(|(left, right)| {
            left.identity != right.identity || left.correlation != right.correlation
        }) {
            return Err(BuildError::MemberCorrelationMismatch {
                identity: left.identity,
            });
        }
        let root = refit_node(&self.root, &members);
        let structure_fingerprint = fingerprint_node(&root, &members);
        Ok(Self {
            members,
            root,
            stats: self.stats,
            structure_fingerprint,
            revision: bind_geometry_revision(structure_fingerprint, geometry_revision),
        })
    }

    pub const fn build_stats(&self) -> BuildStats {
        self.stats
    }
    pub const fn structure_fingerprint(&self) -> u64 {
        self.structure_fingerprint
    }
    pub const fn revision(&self) -> u64 {
        self.revision
    }
    pub fn member_count(&self) -> usize {
        self.members.len()
    }

    pub fn query_frustum(
        &self,
        expected_revision: u64,
        view_projection: Mat4,
    ) -> Result<(BTreeSet<usize>, QueryStats), RevisionMismatch> {
        self.check_revision(expected_revision)?;
        let mut identities = BTreeSet::new();
        let mut stats = QueryStats::default();
        query_frustum_node(
            &self.root,
            &self.members,
            view_projection,
            &mut identities,
            &mut stats,
        );
        Ok((identities, stats))
    }

    pub fn query_nearest_ray(
        &self,
        expected_revision: u64,
        origin: Vec3,
        direction: Vec3,
    ) -> Result<(Option<RayHit>, QueryStats), RevisionMismatch> {
        self.check_revision(expected_revision)?;
        let mut stats = QueryStats::default();
        let hit = query_ray_node(&self.root, &self.members, origin, direction, &mut stats);
        Ok((hit, stats))
    }

    pub fn audit(&self) -> Audit {
        let mut audit = Audit::default();
        let mut observed = vec![0usize; self.members.len()];
        audit_node(&self.root, &self.members, None, &mut observed, &mut audit);
        audit.missing_members = observed.iter().filter(|count| **count == 0).count();
        audit.duplicate_members = observed.iter().filter(|count| **count > 1).count();
        audit
    }

    fn check_revision(&self, expected_revision: u64) -> Result<(), RevisionMismatch> {
        if self.revision == expected_revision {
            Ok(())
        } else {
            Err(RevisionMismatch {
                artifact_revision: self.revision,
                requested_revision: expected_revision,
            })
        }
    }
}

pub fn bind_geometry_revision(structure_fingerprint: u64, geometry_revision: u64) -> u64 {
    let mut fingerprint = structure_fingerprint;
    hash_bytes(&mut fingerprint, &geometry_revision.to_le_bytes());
    fingerprint
}

fn validate_members(members: &[TriangleMember]) -> Result<(), BuildError> {
    if members.is_empty() {
        return Err(BuildError::EmptyInventory);
    }
    let mut identities = BTreeSet::new();
    for (expected, member) in members.iter().enumerate() {
        if member.identity != expected {
            return Err(BuildError::IdentityOrderMismatch {
                expected,
                actual: member.identity,
            });
        }
        if !identities.insert(member.identity) {
            return Err(BuildError::DuplicateIdentity {
                identity: member.identity,
            });
        }
        if !member.vertices.into_iter().flatten().all(f32::is_finite) {
            return Err(BuildError::NonFiniteMember {
                identity: member.identity,
            });
        }
    }
    Ok(())
}

fn build_node(
    members: &[TriangleMember],
    mut identities: Vec<usize>,
    depth: usize,
    stats: &mut BuildStats,
) -> Node {
    stats.nodes += 1;
    stats.maximum_depth = stats.maximum_depth.max(depth);
    let bounds = Bounds::from_members(members, &identities);
    if identities.len() <= LEAF_MEMBERS || depth >= MAX_DEPTH {
        stats.leaves += 1;
        stats.depth_limited_leaves += usize::from(depth >= MAX_DEPTH);
        return Node::Leaf {
            bounds,
            members: identities,
        };
    }
    let axis = bounds.longest_axis();
    identities.sort_by(|left, right| {
        centroid(members[*left].vertices)[axis]
            .total_cmp(&centroid(members[*right].vertices)[axis])
            .then_with(|| left.cmp(right))
    });
    let upper = identities.split_off(identities.len() / 2);
    Node::Branch {
        bounds,
        lower: Box::new(build_node(members, identities, depth + 1, stats)),
        upper: Box::new(build_node(members, upper, depth + 1, stats)),
    }
}

fn refit_node(node: &Node, members: &[TriangleMember]) -> Node {
    match node {
        Node::Leaf {
            members: identities,
            ..
        } => Node::Leaf {
            bounds: Bounds::from_members(members, identities),
            members: identities.clone(),
        },
        Node::Branch { lower, upper, .. } => {
            let lower = Box::new(refit_node(lower, members));
            let upper = Box::new(refit_node(upper, members));
            let mut bounds = lower.bounds();
            bounds.include(upper.bounds());
            Node::Branch {
                bounds,
                lower,
                upper,
            }
        }
    }
}

fn query_frustum_node(
    node: &Node,
    members: &[TriangleMember],
    view_projection: Mat4,
    identities: &mut BTreeSet<usize>,
    stats: &mut QueryStats,
) {
    stats.visited_nodes += 1;
    if bounds_outside_frustum(node.bounds(), view_projection) {
        stats.rejected_nodes += 1;
        return;
    }
    match node {
        Node::Leaf { members: leaf, .. } => {
            stats.visited_leaves += 1;
            for identity in leaf {
                stats.tested_members += 1;
                if !bounds_outside_frustum(
                    Bounds::from_triangle(members[*identity].vertices),
                    view_projection,
                ) {
                    identities.insert(*identity);
                }
            }
        }
        Node::Branch { lower, upper, .. } => {
            query_frustum_node(lower, members, view_projection, identities, stats);
            query_frustum_node(upper, members, view_projection, identities, stats);
        }
    }
}

fn bounds_outside_frustum(bounds: Bounds, view_projection: Mat4) -> bool {
    let clip = bounds
        .corners()
        .map(|point| view_projection * Vec4::new(point.x, point.y, point.z, 1.0));
    let tests: [fn(Vec4) -> bool; 6] = [
        |point| point.x < -point.w,
        |point| point.x > point.w,
        |point| point.y < -point.w,
        |point| point.y > point.w,
        |point| point.z < -point.w,
        |point| point.z > point.w,
    ];
    tests
        .into_iter()
        .any(|outside| clip.iter().copied().all(outside))
}

fn query_ray_node(
    node: &Node,
    members: &[TriangleMember],
    origin: Vec3,
    direction: Vec3,
    stats: &mut QueryStats,
) -> Option<RayHit> {
    stats.visited_nodes += 1;
    if ray_bounds_distance(origin, direction, node.bounds()).is_none() {
        stats.rejected_nodes += 1;
        return None;
    }
    match node {
        Node::Leaf { members: leaf, .. } => {
            stats.visited_leaves += 1;
            leaf.iter()
                .filter_map(|identity| {
                    stats.tested_members += 1;
                    ray_triangle_distance(origin, direction, members[*identity].vertices).map(
                        |distance| RayHit {
                            identity: *identity,
                            distance,
                        },
                    )
                })
                .min_by(compare_hits)
        }
        Node::Branch { lower, upper, .. } => [lower.as_ref(), upper.as_ref()]
            .into_iter()
            .filter_map(|child| query_ray_node(child, members, origin, direction, stats))
            .min_by(compare_hits),
    }
}

fn compare_hits(left: &RayHit, right: &RayHit) -> Ordering {
    left.distance
        .total_cmp(&right.distance)
        .then_with(|| left.identity.cmp(&right.identity))
}

fn ray_bounds_distance(origin: Vec3, direction: Vec3, bounds: Bounds) -> Option<f32> {
    let mut minimum_distance = 0.0_f32;
    let mut maximum_distance = f32::INFINITY;
    for (axis, direction_axis) in direction.to_array().into_iter().enumerate() {
        let origin_axis = origin[axis];
        if direction_axis.abs() <= f32::EPSILON {
            if origin_axis < bounds.minimum[axis] || origin_axis > bounds.maximum[axis] {
                return None;
            }
            continue;
        }
        let inverse = direction_axis.recip();
        let mut near = (bounds.minimum[axis] - origin_axis) * inverse;
        let mut far = (bounds.maximum[axis] - origin_axis) * inverse;
        if near > far {
            std::mem::swap(&mut near, &mut far);
        }
        minimum_distance = minimum_distance.max(near);
        maximum_distance = maximum_distance.min(far);
        if maximum_distance < minimum_distance {
            return None;
        }
    }
    (maximum_distance >= 0.0).then_some(minimum_distance.max(0.0))
}

fn ray_triangle_distance(origin: Vec3, direction: Vec3, vertices: [[f32; 3]; 3]) -> Option<f32> {
    let [a, b, c] = vertices.map(Vec3::from_array);
    let edge_a = b - a;
    let edge_b = c - a;
    let normal = direction.cross(edge_b);
    let determinant = edge_a.dot(normal);
    if determinant.abs() <= 1.0e-6 {
        return None;
    }
    let inverse = determinant.recip();
    let offset = origin - a;
    let u = offset.dot(normal) * inverse;
    if !(0.0..=1.0).contains(&u) {
        return None;
    }
    let q = offset.cross(edge_a);
    let v = direction.dot(q) * inverse;
    if v < 0.0 || u + v > 1.0 {
        return None;
    }
    let distance = edge_b.dot(q) * inverse;
    (distance >= 0.0).then_some(distance)
}

fn audit_node(
    node: &Node,
    members: &[TriangleMember],
    parent: Option<Bounds>,
    observed: &mut [usize],
    audit: &mut Audit,
) {
    audit.nodes += 1;
    audit.containment_failures +=
        usize::from(parent.is_some_and(|bounds| !bounds.contains(node.bounds())));
    match node {
        Node::Leaf {
            bounds,
            members: leaf,
        } => {
            audit.leaves += 1;
            for identity in leaf {
                audit.represented_members += 1;
                observed[*identity] += 1;
                audit.containment_failures += usize::from(
                    members[*identity]
                        .vertices
                        .iter()
                        .any(|point| !bounds.contains_point(*point)),
                );
            }
        }
        Node::Branch { lower, upper, .. } => {
            audit_node(lower, members, Some(node.bounds()), observed, audit);
            audit_node(upper, members, Some(node.bounds()), observed, audit);
        }
    }
}

fn fingerprint_node(node: &Node, members: &[TriangleMember]) -> u64 {
    fn visit(node: &Node, members: &[TriangleMember], hash: &mut u64) {
        match node {
            Node::Leaf { members: leaf, .. } => {
                hash_bytes(hash, &[0]);
                for identity in leaf {
                    hash_bytes(hash, &(*identity as u64).to_le_bytes());
                    hash_bytes(hash, members[*identity].correlation.as_bytes());
                    for value in members[*identity].vertices.into_iter().flatten() {
                        hash_bytes(hash, &value.to_bits().to_le_bytes());
                    }
                }
            }
            Node::Branch { lower, upper, .. } => {
                hash_bytes(hash, &[1]);
                visit(lower, members, hash);
                visit(upper, members, hash);
            }
        }
    }
    let mut hash = HASH_OFFSET;
    visit(node, members, &mut hash);
    hash
}

fn centroid(vertices: [[f32; 3]; 3]) -> [f32; 3] {
    [
        (vertices[0][0] + vertices[1][0] + vertices[2][0]) / 3.0,
        (vertices[0][1] + vertices[1][1] + vertices[2][1]) / 3.0,
        (vertices[0][2] + vertices[1][2] + vertices[2][2]) / 3.0,
    ]
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
    #[cfg(target_arch = "wasm32")]
    use wasm_bindgen_test::wasm_bindgen_test;

    fn fixture(revised: bool) -> Vec<TriangleMember> {
        let offset = if revised { 0.5 } else { 0.0 };
        vec![
            TriangleMember::new(
                0,
                "left",
                [
                    [-2.0 + offset, -1.0, 0.0],
                    [-0.5 + offset, -1.0, 0.0],
                    [-1.0 + offset, 1.0, 0.0],
                ],
            )
            .unwrap(),
            TriangleMember::new(
                1,
                "center",
                [[-0.4, -1.0, 0.0], [0.4, -1.0, 0.0], [0.0, 1.0, 0.0]],
            )
            .unwrap(),
            TriangleMember::new(
                2,
                "right",
                [[0.5, -1.0, 0.0], [2.0, -1.0, 0.0], [1.0, 1.0, 0.0]],
            )
            .unwrap(),
        ]
    }

    fn portable_fixture_assertions() {
        let baseline = Artifact::build(fixture(false), 7).unwrap();
        assert_eq!(baseline.structure_fingerprint(), 0xa7ab_8dff_a4f4_b487);
        assert_eq!(baseline.revision(), 0x0c2f_9ba4_8338_4480);
        let audit = baseline.audit();
        assert_eq!(audit.missing_members, 0);
        assert_eq!(audit.duplicate_members, 0);
        assert_eq!(audit.containment_failures, 0);
        let (candidates, _) = baseline
            .query_frustum(baseline.revision(), Mat4::IDENTITY)
            .unwrap();
        assert_eq!(candidates, BTreeSet::from([0, 1, 2]));
        let (hit, _) = baseline
            .query_nearest_ray(baseline.revision(), Vec3::new(0.0, 0.0, 2.0), Vec3::NEG_Z)
            .unwrap();
        assert_eq!(hit.unwrap().identity, 1);
        assert!(baseline
            .query_frustum(
                bind_geometry_revision(baseline.structure_fingerprint(), 8),
                Mat4::IDENTITY
            )
            .is_err());

        let revised = baseline.refit(fixture(true), 8).unwrap();
        assert_ne!(baseline.revision(), revised.revision());
        assert!(revised
            .query_frustum(baseline.revision(), Mat4::IDENTITY)
            .is_err());
        let (revised_candidates, _) = revised
            .query_frustum(revised.revision(), Mat4::IDENTITY)
            .unwrap();
        assert_eq!(revised_candidates, BTreeSet::from([0, 1, 2]));
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
    fn portable_native_and_wasm_fixture() {
        portable_fixture_assertions();
    }

    #[test]
    fn structure_fingerprint_is_deterministic_and_revision_is_bound() {
        let left = Artifact::build(fixture(false), 7).unwrap();
        let right = Artifact::build(fixture(false), 7).unwrap();
        assert_eq!(left.structure_fingerprint(), right.structure_fingerprint());
        assert_eq!(left.revision(), right.revision());
        assert_ne!(
            left.revision(),
            bind_geometry_revision(left.structure_fingerprint(), 8)
        );
    }
}
