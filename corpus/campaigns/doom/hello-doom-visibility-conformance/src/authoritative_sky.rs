//! Doom-private authoritative sky-region evidence.
//!
//! The corrected ordered protocol retains diagnostic columns as its oracle.
//! This module deliberately does not forward those cells as presentation
//! primitives. It resolves maximal source-owned runs and describes their
//! boundaries in normalized view coordinates while retaining one explained
//! outcome for every input sky interval.

use std::collections::{BTreeMap, BTreeSet};

use doom_geometry_provider::{
    DoomSegClassicPlaneKey, DoomSegClassicPlaneKind, DoomSegClassicVerticalClipObservation,
    DoomTextureExtent,
};
use doom_map_provider::DoomMapCore;

use crate::DoomVisibilityFixture;

const REFERENCE_HEIGHT: usize = 200;

#[derive(Clone, Debug, PartialEq)]
pub struct AuthoritativeSkyViewIdentity {
    pub fixture: String,
    pub source_position: [i16; 2],
    pub heading_radians: f64,
    pub source_eye_height: i16,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct AuthoritativeSkyBoundaryKnot {
    /// Normalized device X in `[-1, 1]`.
    pub x: f64,
    /// Normalized device Y in `[-1, 1]`.
    pub y: f64,
}

#[derive(Clone, Debug, PartialEq)]
pub struct AuthoritativeSkyRegion {
    pub source_plane: DoomSegClassicPlaneKey,
    pub source_plane_instance: usize,
    pub source_sector: u32,
    pub source_seg: u32,
    pub source_order: usize,
    pub source_sectors: BTreeSet<u32>,
    pub source_segs: BTreeSet<u32>,
    pub paired_sky_boundary_source_segs: BTreeSet<u32>,
    pub prepared_view: AuthoritativeSkyViewIdentity,
    pub runtime_snapshot: String,
    /// Continuous normalized horizontal extent. This is deliberately not a
    /// renderer scissor or diagnostic-column interval.
    pub horizontal_ndc: [f64; 2],
    /// Piecewise-linear top boundary of the source-authorized region.
    pub upper_boundary: Vec<AuthoritativeSkyBoundaryKnot>,
    /// Piecewise-linear bottom boundary of the source-authorized region.
    pub lower_boundary: Vec<AuthoritativeSkyBoundaryKnot>,
    /// Oracle-only provenance used to prove conservation. Realization must not
    /// use this identity as a persistent mesh or public renderer primitive.
    pub oracle_columns: [usize; 2],
    pub oracle_intervals: usize,
    pub oracle_cells: usize,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AuthoritativeSkyOmissionReason {
    MissingSourceAuthority,
    SourceSegNotAdmitted,
    InvalidInterval,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum AuthoritativeSkyLedgerOutcomeKind {
    Modeled { region: usize },
    Omitted(AuthoritativeSkyOmissionReason),
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AuthoritativeSkyLedgerOutcome {
    pub source_plane_instance: usize,
    pub column: usize,
    pub interval: [usize; 2],
    pub source_authority: Option<[u32; 2]>,
    pub outcome: AuthoritativeSkyLedgerOutcomeKind,
}

#[derive(Clone, Debug, PartialEq)]
pub struct AuthoritativeSkyRegionManifest {
    pub prepared_view: AuthoritativeSkyViewIdentity,
    pub runtime_snapshot: String,
    pub regions: Vec<AuthoritativeSkyRegion>,
    pub ledger_outcomes: Vec<AuthoritativeSkyLedgerOutcome>,
    pub input_sky_intervals: usize,
    pub input_sky_cells: usize,
    pub modeled_sky_intervals: usize,
    pub modeled_sky_cells: usize,
    pub omitted_sky_intervals: usize,
    pub omitted_sky_cells: usize,
    /// This model reads only `F_SKY1` ceiling instances. Ordinary contribution
    /// removal is therefore structurally impossible at this seam.
    pub removed_non_sky_contributions: usize,
    /// Paired-sky boundaries are ordered Doom protocol evidence, but they do
    /// not become authoritative coverage unless a retained sky-plane region
    /// actually overlaps them.
    pub paired_boundary_columns_observed: usize,
    pub paired_boundary_columns_claimed: usize,
    pub fail_open: bool,
    pub structural_fingerprint: String,
}

/// Corpus-private, ephemeral realization request for one authoritative sky
/// region. It deliberately has no `MeshHandle`: assigning persistent renderer
/// identity to this prepared-view payload would misdescribe its lifetime.
#[derive(Clone, Debug, PartialEq)]
pub struct AuthoritativeSkyDepthDeclaration {
    pub source_plane: DoomSegClassicPlaneKey,
    pub source_plane_instance: usize,
    pub prepared_view: AuthoritativeSkyViewIdentity,
    pub runtime_snapshot: String,
    pub persistent_material_key: String,
    pub clip_depth: f32,
    pub positions: Vec<[f32; 3]>,
    pub triangle_count: usize,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AuthoritativeSkyDepthRejection {
    EmptyRegion,
    InvalidDepth,
    NearPlaneDepth,
    InvalidBoundary,
    MissingSourceSeg,
    MissingSourceVertex,
    SourceRayMiss,
    OutsideProjectionDepth,
}

#[derive(Clone, Debug, PartialEq)]
pub struct AuthoritativeSkyDepthOutcome {
    pub source_plane_instance: usize,
    pub declaration: Option<usize>,
    pub rejection: Option<AuthoritativeSkyDepthRejection>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct AuthoritativeSkyDepthManifest {
    pub declarations: Vec<AuthoritativeSkyDepthDeclaration>,
    pub outcomes: Vec<AuthoritativeSkyDepthOutcome>,
    pub persistent_material_key: String,
    pub persistent_mesh_identities: usize,
    pub structural_fingerprint: String,
}

/// Corpus-only comparison between the ordered Doom oracle and Candidate 1's
/// continuous triangle realization. This is diagnostic evidence, not renderer
/// vocabulary and not a pixel-identity contract.
#[derive(Clone, Debug, PartialEq)]
pub struct AuthoritativeSkyDepthApproximationObservation {
    pub oracle_samples: usize,
    pub coverage_mismatches: usize,
    pub coverage_extra_cells: usize,
    pub coverage_missing_cells: usize,
    pub depth_samples: usize,
    pub unresolved_depth_samples: usize,
    pub maximum_absolute_clip_depth_error: f64,
    pub mean_absolute_clip_depth_error: f64,
}

fn interpolate_boundary(knots: &[AuthoritativeSkyBoundaryKnot], x: f64) -> Option<f64> {
    knots.windows(2).find_map(|pair| {
        let [left, right] = pair else {
            return None;
        };
        if x < left.x - 1.0e-12 || x > right.x + 1.0e-12 {
            return None;
        }
        let width = right.x - left.x;
        if width.abs() <= 1.0e-12 {
            return Some(left.y);
        }
        let t = (x - left.x) / width;
        Some((right.y - left.y).mul_add(t, left.y))
    })
}

fn depth_positions(
    region: &AuthoritativeSkyRegion,
    clip_depth: f32,
) -> Result<Vec<[f32; 3]>, AuthoritativeSkyDepthRejection> {
    if region.upper_boundary.len() < 2 || region.lower_boundary.len() < 2 {
        return Err(AuthoritativeSkyDepthRejection::EmptyRegion);
    }
    let mut xs = region
        .upper_boundary
        .iter()
        .chain(&region.lower_boundary)
        .map(|knot| knot.x)
        .collect::<Vec<_>>();
    xs.sort_by(f64::total_cmp);
    xs.dedup_by(|left, right| (*left - *right).abs() <= 1.0e-12);
    let mut positions = Vec::new();
    for pair in xs.windows(2) {
        let left = pair[0];
        let right = pair[1];
        if right - left <= 1.0e-12 {
            continue;
        }
        let Some(upper_left) = interpolate_boundary(&region.upper_boundary, left) else {
            continue;
        };
        let Some(upper_right) = interpolate_boundary(&region.upper_boundary, right) else {
            continue;
        };
        let Some(lower_left) = interpolate_boundary(&region.lower_boundary, left) else {
            continue;
        };
        let Some(lower_right) = interpolate_boundary(&region.lower_boundary, right) else {
            continue;
        };
        if ![
            left,
            right,
            upper_left,
            upper_right,
            lower_left,
            lower_right,
        ]
        .into_iter()
        .all(f64::is_finite)
            || upper_left <= lower_left
            || upper_right <= lower_right
        {
            return Err(AuthoritativeSkyDepthRejection::InvalidBoundary);
        }
        let l = left as f32;
        let r = right as f32;
        let ul = upper_left as f32;
        let ur = upper_right as f32;
        let ll = lower_left as f32;
        let lr = lower_right as f32;
        positions.extend_from_slice(&[
            [l, ll, clip_depth],
            [r, lr, clip_depth],
            [r, ur, clip_depth],
            [l, ll, clip_depth],
            [r, ur, clip_depth],
            [l, ul, clip_depth],
        ]);
    }
    if positions.is_empty() {
        Err(AuthoritativeSkyDepthRejection::EmptyRegion)
    } else {
        Ok(positions)
    }
}

fn source_ray_supporting_line_depth(
    viewer: [i16; 2],
    ray: [f64; 2],
    start: [i16; 2],
    end: [i16; 2],
) -> Option<f64> {
    let offset = [
        f64::from(start[0]) - f64::from(viewer[0]),
        f64::from(start[1]) - f64::from(viewer[1]),
    ];
    let segment = [
        f64::from(end[0]) - f64::from(start[0]),
        f64::from(end[1]) - f64::from(start[1]),
    ];
    let cross = |left: [f64; 2], right: [f64; 2]| left[0] * right[1] - left[1] * right[0];
    let denominator = cross(ray, segment);
    if denominator.abs() <= f64::EPSILON {
        return None;
    }
    let depth = cross(offset, segment) / denominator;
    (depth > 0.0).then_some(depth)
}

fn gl_clip_depth(forward_depth: f64, near: f64, far: f64) -> Option<f32> {
    if !forward_depth.is_finite()
        || !near.is_finite()
        || !far.is_finite()
        || near <= 0.0
        || far <= near
        || forward_depth <= near
        || forward_depth > far
    {
        return None;
    }
    let depth = (far + near) / (far - near) - (2.0 * far * near) / ((far - near) * forward_depth);
    depth.is_finite().then_some(depth as f32)
}

fn source_depth_at_x(
    region: &AuthoritativeSkyRegion,
    map: &DoomMapCore,
    x: f64,
    near: f64,
    far: f64,
) -> Result<f32, AuthoritativeSkyDepthRejection> {
    let seg = map
        .segs
        .get(region.source_seg as usize)
        .ok_or(AuthoritativeSkyDepthRejection::MissingSourceSeg)?;
    let start = map
        .vertices
        .get(usize::from(seg.start_vertex))
        .ok_or(AuthoritativeSkyDepthRejection::MissingSourceVertex)?;
    let end = map
        .vertices
        .get(usize::from(seg.end_vertex))
        .ok_or(AuthoritativeSkyDepthRejection::MissingSourceVertex)?;
    // The reference protocol uses a 90-degree horizontal field of view, so
    // NDC x is tan(local angle). Keep that Doom-private projection fact here.
    let local_angle = x.atan();
    let (heading_sin, heading_cos) = region.prepared_view.heading_radians.sin_cos();
    let (local_sin, local_cos) = local_angle.sin_cos();
    let forward = [heading_cos, heading_sin];
    let right = [-forward[1], forward[0]];
    let ray = [
        forward[0].mul_add(local_cos, right[0] * local_sin),
        forward[1].mul_add(local_cos, right[1] * local_sin),
    ];
    // The ordered source protocol assigns authority to inclusive raster
    // columns. A continuous declaration uses those columns' outer edges, so
    // an edge ray may lie fractionally beyond the finite SEG endpoint even
    // though the complete source column was authoritatively assigned to that
    // SEG. Once that authority is established, interpolate depth on the SEG's
    // supporting line rather than re-litigating finite-segment membership at
    // the continuous edge. Parallel and behind-viewer intersections still
    // fail open.
    let radial_depth = source_ray_supporting_line_depth(
        region.prepared_view.source_position,
        ray,
        [start.x, start.y],
        [end.x, end.y],
    )
    .ok_or(AuthoritativeSkyDepthRejection::SourceRayMiss)?;
    let forward_depth = radial_depth * local_cos;
    gl_clip_depth(forward_depth, near, far)
        .ok_or(AuthoritativeSkyDepthRejection::OutsideProjectionDepth)
}

fn source_depth_positions(
    region: &AuthoritativeSkyRegion,
    map: &DoomMapCore,
    near: f64,
    far: f64,
) -> Result<Vec<[f32; 3]>, AuthoritativeSkyDepthRejection> {
    if region.upper_boundary.len() < 2 || region.lower_boundary.len() < 2 {
        return Err(AuthoritativeSkyDepthRejection::EmptyRegion);
    }
    let mut xs = region
        .upper_boundary
        .iter()
        .chain(&region.lower_boundary)
        .map(|knot| knot.x)
        .collect::<Vec<_>>();
    xs.sort_by(f64::total_cmp);
    xs.dedup_by(|left, right| (*left - *right).abs() <= 1.0e-12);
    let mut positions = Vec::new();
    for pair in xs.windows(2) {
        let [left, right] = [pair[0], pair[1]];
        if right - left <= 1.0e-12 {
            continue;
        }
        let upper_left = interpolate_boundary(&region.upper_boundary, left)
            .ok_or(AuthoritativeSkyDepthRejection::InvalidBoundary)?;
        let upper_right = interpolate_boundary(&region.upper_boundary, right)
            .ok_or(AuthoritativeSkyDepthRejection::InvalidBoundary)?;
        let lower_left = interpolate_boundary(&region.lower_boundary, left)
            .ok_or(AuthoritativeSkyDepthRejection::InvalidBoundary)?;
        let lower_right = interpolate_boundary(&region.lower_boundary, right)
            .ok_or(AuthoritativeSkyDepthRejection::InvalidBoundary)?;
        if upper_left <= lower_left || upper_right <= lower_right {
            return Err(AuthoritativeSkyDepthRejection::InvalidBoundary);
        }
        let left_depth = source_depth_at_x(region, map, left, near, far)?;
        let right_depth = source_depth_at_x(region, map, right, near, far)?;
        positions.extend_from_slice(&[
            [left as f32, lower_left as f32, left_depth],
            [right as f32, lower_right as f32, right_depth],
            [right as f32, upper_right as f32, right_depth],
            [left as f32, lower_left as f32, left_depth],
            [right as f32, upper_right as f32, right_depth],
            [left as f32, upper_left as f32, left_depth],
        ]);
    }
    if positions.is_empty() {
        Err(AuthoritativeSkyDepthRejection::EmptyRegion)
    } else {
        Ok(positions)
    }
}

fn declaration_depth_at_x(declaration: &AuthoritativeSkyDepthDeclaration, x: f64) -> Option<f64> {
    declaration.positions.chunks_exact(6).find_map(|positions| {
        let left_x = f64::from(positions[0][0]);
        let right_x = f64::from(positions[1][0]);
        if x < left_x - 1.0e-9 || x > right_x + 1.0e-9 {
            return None;
        }
        let width = right_x - left_x;
        if width.abs() <= 1.0e-12 {
            return Some(f64::from(positions[0][2]));
        }
        let t = (x - left_x) / width;
        Some(
            (f64::from(positions[1][2]) - f64::from(positions[0][2]))
                .mul_add(t, f64::from(positions[0][2])),
        )
    })
}

fn boundary_row(y: f64) -> Option<usize> {
    let row = (1.0 - y) * (REFERENCE_HEIGHT as f64 * 0.5);
    if !row.is_finite() || row < -1.0e-7 || row > REFERENCE_HEIGHT as f64 + 1.0e-7 {
        return None;
    }
    Some(row.round().clamp(0.0, REFERENCE_HEIGHT as f64) as usize)
}

/// Compares Candidate 1 at the centers of the exact source columns that
/// granted sky authority. Coverage mismatch identifies region-construction
/// loss/addition. Depth error identifies the separate approximation introduced
/// when exact source-ray depths are represented by linearly interpolated clip
/// triangles.
pub fn observe_authoritative_sky_source_depth_approximation(
    regions: &AuthoritativeSkyRegionManifest,
    depth: &AuthoritativeSkyDepthManifest,
    map: &DoomMapCore,
    near: f64,
    far: f64,
) -> AuthoritativeSkyDepthApproximationObservation {
    let mut observation = AuthoritativeSkyDepthApproximationObservation {
        oracle_samples: 0,
        coverage_mismatches: 0,
        coverage_extra_cells: 0,
        coverage_missing_cells: 0,
        depth_samples: 0,
        unresolved_depth_samples: 0,
        maximum_absolute_clip_depth_error: 0.0,
        mean_absolute_clip_depth_error: 0.0,
    };
    let mut absolute_depth_error_sum = 0.0;

    for outcome in &regions.ledger_outcomes {
        let AuthoritativeSkyLedgerOutcomeKind::Modeled { region } = outcome.outcome else {
            continue;
        };
        observation.oracle_samples += 1;
        let Some(source_region) = regions.regions.get(region) else {
            observation.coverage_mismatches += 1;
            observation.unresolved_depth_samples += 1;
            continue;
        };
        let x = ndc_x(outcome.column, 320) + 1.0 / 320.0;
        let realized_interval = interpolate_boundary(&source_region.upper_boundary, x)
            .and_then(boundary_row)
            .zip(interpolate_boundary(&source_region.lower_boundary, x).and_then(boundary_row))
            .and_then(|(upper, lower_edge)| lower_edge.checked_sub(1).map(|lower| [upper, lower]));
        match realized_interval {
            Some(realized) if realized == outcome.interval => {}
            Some(realized) => {
                observation.coverage_mismatches += 1;
                let source_cells =
                    (outcome.interval[0]..=outcome.interval[1]).collect::<BTreeSet<_>>();
                let realized_cells = if realized[0] <= realized[1] {
                    (realized[0]..=realized[1]).collect::<BTreeSet<_>>()
                } else {
                    BTreeSet::new()
                };
                observation.coverage_extra_cells +=
                    realized_cells.difference(&source_cells).count();
                observation.coverage_missing_cells +=
                    source_cells.difference(&realized_cells).count();
            }
            None => {
                observation.coverage_mismatches += 1;
                observation.coverage_missing_cells +=
                    outcome.interval[1].saturating_sub(outcome.interval[0]) + 1;
            }
        }

        let declaration = depth
            .outcomes
            .get(region)
            .and_then(|depth_outcome| depth_outcome.declaration)
            .and_then(|index| depth.declarations.get(index));
        let exact = source_depth_at_x(source_region, map, x, near, far);
        let approximate = declaration.and_then(|item| declaration_depth_at_x(item, x));
        match (exact, approximate) {
            (Ok(exact), Some(approximate)) => {
                let error = (f64::from(exact) - approximate).abs();
                observation.depth_samples += 1;
                absolute_depth_error_sum += error;
                observation.maximum_absolute_clip_depth_error =
                    observation.maximum_absolute_clip_depth_error.max(error);
            }
            _ => observation.unresolved_depth_samples += 1,
        }
    }
    if observation.depth_samples > 0 {
        observation.mean_absolute_clip_depth_error =
            absolute_depth_error_sum / observation.depth_samples as f64;
    }
    observation
}

/// Realizes each Doom-authorized sky region at the depth of the source SEG
/// which established that authority. Any unresolved region remains an
/// explicit rejection so the E1M1 caller can fail open as one whole batch.
pub fn prepare_authoritative_sky_source_depth_declarations(
    regions: &AuthoritativeSkyRegionManifest,
    map: &DoomMapCore,
    near: f64,
    far: f64,
    persistent_material_key: &str,
) -> AuthoritativeSkyDepthManifest {
    let mut declarations = Vec::new();
    let mut outcomes = Vec::new();
    for region in &regions.regions {
        match source_depth_positions(region, map, near, far) {
            Ok(positions) => {
                let declaration = declarations.len();
                let triangle_count = positions.len() / 3;
                let clip_depth = positions.iter().map(|position| position[2]).sum::<f32>()
                    / positions.len() as f32;
                declarations.push(AuthoritativeSkyDepthDeclaration {
                    source_plane: region.source_plane.clone(),
                    source_plane_instance: region.source_plane_instance,
                    prepared_view: region.prepared_view.clone(),
                    runtime_snapshot: region.runtime_snapshot.clone(),
                    persistent_material_key: persistent_material_key.to_owned(),
                    clip_depth,
                    positions,
                    triangle_count,
                });
                outcomes.push(AuthoritativeSkyDepthOutcome {
                    source_plane_instance: region.source_plane_instance,
                    declaration: Some(declaration),
                    rejection: None,
                });
            }
            Err(rejection) => outcomes.push(AuthoritativeSkyDepthOutcome {
                source_plane_instance: region.source_plane_instance,
                declaration: None,
                rejection: Some(rejection),
            }),
        }
    }
    let trace = format!(
        "source-depth-v1;view={:?};snapshot={};material={persistent_material_key};near={near:?};far={far:?};declarations={declarations:?};outcomes={outcomes:?};persistent-mesh-identities=0",
        regions.prepared_view, regions.runtime_snapshot
    );
    AuthoritativeSkyDepthManifest {
        declarations,
        outcomes,
        persistent_material_key: persistent_material_key.to_owned(),
        persistent_mesh_identities: 0,
        structural_fingerprint: blake3::hash(trace.as_bytes()).to_hex().to_string(),
    }
}

/// Lowers authoritative regions into provider-neutral clip-local triangles,
/// while retaining their explicitly ephemeral lifetime. This does not upload
/// them: the current stable renderer has no transient geometry submission.
pub fn prepare_authoritative_sky_depth_declarations(
    regions: &AuthoritativeSkyRegionManifest,
    clip_depth: f32,
    persistent_material_key: &str,
) -> AuthoritativeSkyDepthManifest {
    let depth_rejection = if !clip_depth.is_finite() || !(-1.0..=1.0).contains(&clip_depth) {
        Some(AuthoritativeSkyDepthRejection::InvalidDepth)
    } else if clip_depth <= -1.0 + 1.0e-6 {
        Some(AuthoritativeSkyDepthRejection::NearPlaneDepth)
    } else {
        None
    };
    let mut declarations = Vec::new();
    let mut outcomes = Vec::new();
    for region in &regions.regions {
        let positions = depth_rejection.map_or_else(|| depth_positions(region, clip_depth), Err);
        match positions {
            Ok(positions) => {
                let declaration = declarations.len();
                let triangle_count = positions.len() / 3;
                declarations.push(AuthoritativeSkyDepthDeclaration {
                    source_plane: region.source_plane.clone(),
                    source_plane_instance: region.source_plane_instance,
                    prepared_view: region.prepared_view.clone(),
                    runtime_snapshot: region.runtime_snapshot.clone(),
                    persistent_material_key: persistent_material_key.to_owned(),
                    clip_depth,
                    positions,
                    triangle_count,
                });
                outcomes.push(AuthoritativeSkyDepthOutcome {
                    source_plane_instance: region.source_plane_instance,
                    declaration: Some(declaration),
                    rejection: None,
                });
            }
            Err(rejection) => outcomes.push(AuthoritativeSkyDepthOutcome {
                source_plane_instance: region.source_plane_instance,
                declaration: None,
                rejection: Some(rejection),
            }),
        }
    }
    let trace = format!(
        "view={:?};snapshot={};material={persistent_material_key};depth={clip_depth:?};declarations={declarations:?};outcomes={outcomes:?};persistent-mesh-identities=0",
        regions.prepared_view, regions.runtime_snapshot
    );
    AuthoritativeSkyDepthManifest {
        declarations,
        outcomes,
        persistent_material_key: persistent_material_key.to_owned(),
        persistent_mesh_identities: 0,
        structural_fingerprint: blake3::hash(trace.as_bytes()).to_hex().to_string(),
    }
}

fn texture_extents(fixture: &DoomVisibilityFixture) -> Vec<DoomTextureExtent> {
    let mut names = BTreeSet::new();
    for sidedef in &fixture.map.sidedefs {
        for name in [
            &sidedef.upper_texture,
            &sidedef.lower_texture,
            &sidedef.middle_texture,
        ] {
            if name != "-" {
                names.insert(name.clone());
            }
        }
    }
    names
        .into_iter()
        .map(|name| DoomTextureExtent {
            name,
            width: 64,
            height: 128,
        })
        .collect()
}

fn ndc_x(edge: usize, width: usize) -> f64 {
    (edge as f64 / width as f64).mul_add(2.0, -1.0)
}

fn ndc_y(edge: usize) -> f64 {
    1.0 - (edge as f64 / REFERENCE_HEIGHT as f64) * 2.0
}

fn push_knot(knots: &mut Vec<AuthoritativeSkyBoundaryKnot>, knot: AuthoritativeSkyBoundaryKnot) {
    if knots.len() >= 2 {
        let a = knots[knots.len() - 2];
        let b = knots[knots.len() - 1];
        let cross = (b.x - a.x) * (knot.y - b.y) - (b.y - a.y) * (knot.x - b.x);
        if cross.abs() <= 1.0e-12 {
            knots.pop();
        }
    }
    knots.push(knot);
}

fn boundary_knots(
    columns: &[(usize, [usize; 2])],
    width: usize,
    upper: bool,
) -> Vec<AuthoritativeSkyBoundaryKnot> {
    let mut knots = Vec::new();
    for &(column, interval) in columns {
        let row = if upper { interval[0] } else { interval[1] + 1 };
        push_knot(
            &mut knots,
            AuthoritativeSkyBoundaryKnot {
                x: ndc_x(column, width),
                y: ndc_y(row),
            },
        );
        push_knot(
            &mut knots,
            AuthoritativeSkyBoundaryKnot {
                x: ndc_x(column + 1, width),
                y: ndc_y(row),
            },
        );
    }
    knots
}

struct ModeledRegionContext<'a> {
    source_sectors: &'a BTreeSet<u32>,
    source_segs: &'a BTreeSet<u32>,
    paired_by_column: &'a BTreeMap<usize, BTreeSet<u32>>,
    prepared_view: &'a AuthoritativeSkyViewIdentity,
    runtime_snapshot: &'a str,
    width: usize,
}

fn modeled_region(
    key: &DoomSegClassicPlaneKey,
    plane_instance: usize,
    columns: &[(usize, [usize; 2])],
    source_authority: [u32; 2],
    source_order: usize,
    context: &ModeledRegionContext<'_>,
) -> AuthoritativeSkyRegion {
    let first = columns.first().expect("a modeled run is non-empty").0;
    let last = columns.last().expect("a modeled run is non-empty").0;
    let oracle_cells = columns
        .iter()
        .map(|(_, interval)| interval[1] - interval[0] + 1)
        .sum();
    let paired_sky_boundary_source_segs = columns
        .iter()
        .filter_map(|(column, _)| context.paired_by_column.get(column))
        .flatten()
        .copied()
        .collect();
    AuthoritativeSkyRegion {
        source_plane: key.clone(),
        source_plane_instance: plane_instance,
        source_sector: source_authority[0],
        source_seg: source_authority[1],
        source_order,
        source_sectors: context.source_sectors.clone(),
        source_segs: context.source_segs.clone(),
        paired_sky_boundary_source_segs,
        prepared_view: context.prepared_view.clone(),
        runtime_snapshot: context.runtime_snapshot.to_owned(),
        horizontal_ndc: [ndc_x(first, context.width), ndc_x(last + 1, context.width)],
        upper_boundary: boundary_knots(columns, context.width, true),
        lower_boundary: boundary_knots(columns, context.width, false),
        oracle_columns: [first, last],
        oracle_intervals: columns.len(),
        oracle_cells,
    }
}

pub fn model_authoritative_sky_regions(
    vertical: &DoomSegClassicVerticalClipObservation,
    admitted_seg_order: &[u32],
    prepared_view: AuthoritativeSkyViewIdentity,
    runtime_snapshot: &str,
) -> AuthoritativeSkyRegionManifest {
    let admitted = admitted_seg_order
        .iter()
        .enumerate()
        .map(|(order, seg)| (*seg, order))
        .collect::<BTreeMap<_, _>>();
    let paired_by_column = vertical
        .column_traces
        .iter()
        .map(|trace| (trace.column, trace.paired_sky_boundary_source_segs.clone()))
        .collect::<BTreeMap<_, _>>();
    let paired_boundary_columns_observed = paired_by_column
        .values()
        .filter(|sources| !sources.is_empty())
        .count();
    let width = vertical
        .plane_spans
        .keys
        .values()
        .flatten()
        .next()
        .map_or(320, |instance| instance.columns.len());

    let mut regions = Vec::new();
    let mut ledger_outcomes = Vec::new();
    let mut input_sky_intervals = 0;
    let mut input_sky_cells = 0;
    let mut modeled_sky_intervals = 0;
    let mut modeled_sky_cells = 0;
    let mut omitted_sky_intervals = 0;
    let mut omitted_sky_cells = 0;

    for (key, instances) in &vertical.plane_spans.keys {
        if key.kind != DoomSegClassicPlaneKind::Ceiling || key.texture != "F_SKY1" {
            continue;
        }
        for (plane_instance, instance) in instances.iter().enumerate() {
            let mut column = instance.minimum_column;
            while column <= instance.maximum_column {
                let Some(interval) = instance.columns[column] else {
                    column += 1;
                    continue;
                };
                input_sky_intervals += 1;
                let cells = interval[1].saturating_sub(interval[0]) + 1;
                input_sky_cells += cells;
                let authority = instance.column_sources[column];
                let reason = if interval[0] > interval[1] {
                    Some(AuthoritativeSkyOmissionReason::InvalidInterval)
                } else if authority.is_none() {
                    Some(AuthoritativeSkyOmissionReason::MissingSourceAuthority)
                } else if !admitted.contains_key(&authority.expect("checked above")[1]) {
                    Some(AuthoritativeSkyOmissionReason::SourceSegNotAdmitted)
                } else {
                    None
                };
                if let Some(reason) = reason {
                    ledger_outcomes.push(AuthoritativeSkyLedgerOutcome {
                        source_plane_instance: plane_instance,
                        column,
                        interval,
                        source_authority: authority,
                        outcome: AuthoritativeSkyLedgerOutcomeKind::Omitted(reason),
                    });
                    omitted_sky_intervals += 1;
                    omitted_sky_cells += cells;
                    column += 1;
                    continue;
                }

                let authority = authority.expect("valid authority was established");
                let source_order = admitted[&authority[1]];
                let mut run = vec![(column, interval)];
                let mut next = column + 1;
                while next <= instance.maximum_column {
                    let Some(next_interval) = instance.columns[next] else {
                        break;
                    };
                    if instance.column_sources[next] != Some(authority) {
                        break;
                    }
                    run.push((next, next_interval));
                    next += 1;
                }
                let region_index = regions.len();
                regions.push(modeled_region(
                    key,
                    plane_instance,
                    &run,
                    authority,
                    source_order,
                    &ModeledRegionContext {
                        source_sectors: &instance.source_sectors,
                        source_segs: &instance.source_segs,
                        paired_by_column: &paired_by_column,
                        prepared_view: &prepared_view,
                        runtime_snapshot,
                        width,
                    },
                ));
                for (run_column, run_interval) in &run {
                    let run_cells = run_interval[1] - run_interval[0] + 1;
                    if *run_column != column {
                        input_sky_intervals += 1;
                        input_sky_cells += run_cells;
                    }
                    modeled_sky_intervals += 1;
                    modeled_sky_cells += run_cells;
                    ledger_outcomes.push(AuthoritativeSkyLedgerOutcome {
                        source_plane_instance: plane_instance,
                        column: *run_column,
                        interval: *run_interval,
                        source_authority: Some(authority),
                        outcome: AuthoritativeSkyLedgerOutcomeKind::Modeled {
                            region: region_index,
                        },
                    });
                }
                column = next;
            }
        }
    }

    let trace = std::iter::once(format!(
        "view={prepared_view:?};snapshot={runtime_snapshot};input-intervals={input_sky_intervals};input-cells={input_sky_cells}"
    ))
        .chain(regions.iter()
        .enumerate()
        .map(|(index, region)| {
            format!(
                "region={index};plane={:?}:{}:{}:{};instance={};authority={}:{};order={};ndc={:.9}..{:.9};columns={}..{};intervals={};cells={};upper={:?};lower={:?};paired={:?};view={:?};snapshot={}",
                region.source_plane.kind,
                region.source_plane.height,
                region.source_plane.texture,
                region.source_plane.light,
                region.source_plane_instance,
                region.source_sector,
                region.source_seg,
                region.source_order,
                region.horizontal_ndc[0],
                region.horizontal_ndc[1],
                region.oracle_columns[0],
                region.oracle_columns[1],
                region.oracle_intervals,
                region.oracle_cells,
                region.upper_boundary,
                region.lower_boundary,
                region.paired_sky_boundary_source_segs,
                region.prepared_view,
                region.runtime_snapshot,
            )
        }))
        .chain(ledger_outcomes.iter().filter_map(|outcome| {
            if matches!(outcome.outcome, AuthoritativeSkyLedgerOutcomeKind::Modeled { .. }) {
                None
            } else {
                Some(format!("omission={outcome:?}"))
            }
        }))
        .collect::<Vec<_>>();
    let structural_fingerprint = blake3::hash(trace.join("\n").as_bytes())
        .to_hex()
        .to_string();
    let paired_boundary_columns_claimed = regions
        .iter()
        .map(|region| {
            (region.oracle_columns[0]..=region.oracle_columns[1])
                .filter(|column| {
                    paired_by_column
                        .get(column)
                        .is_some_and(|sources| !sources.is_empty())
                })
                .count()
        })
        .sum();
    AuthoritativeSkyRegionManifest {
        prepared_view,
        runtime_snapshot: runtime_snapshot.to_owned(),
        regions,
        ledger_outcomes,
        input_sky_intervals,
        input_sky_cells,
        modeled_sky_intervals,
        modeled_sky_cells,
        omitted_sky_intervals,
        omitted_sky_cells,
        removed_non_sky_contributions: 0,
        paired_boundary_columns_observed,
        paired_boundary_columns_claimed,
        fail_open: omitted_sky_intervals > 0,
        structural_fingerprint,
    }
}

pub fn observe_authoritative_sky_regions(
    fixture: &DoomVisibilityFixture,
    source_eye_height: i16,
    runtime_snapshot: &str,
) -> Result<AuthoritativeSkyRegionManifest, String> {
    let traversal = fixture
        .observe_classic_bsp()
        .map_err(|error| error.to_string())?;
    let vertical = fixture
        .observe_classic_vertical_clips(source_eye_height, &texture_extents(fixture))
        .map_err(|error| error.to_string())?;
    Ok(model_authoritative_sky_regions(
        &vertical,
        &traversal.admitted_seg_order,
        AuthoritativeSkyViewIdentity {
            fixture: fixture.name.clone(),
            source_position: fixture.viewer.position,
            heading_radians: fixture.viewer.heading_radians,
            source_eye_height,
        },
        runtime_snapshot,
    ))
}

#[cfg(test)]
mod tests {
    use doom_geometry_provider::DoomSegClassicPlaneInstance;

    use super::*;
    use crate::{
        one_sky_far_control_fixture, paired_sky_far_control_fixture, terminal_sky_ordered_fixture,
        vertical_aperture_control_fixture,
    };

    #[test]
    fn authorized_column_edge_uses_the_source_seg_supporting_line() {
        // An inclusive projected column may extend slightly beyond either
        // finite endpoint. The finite SEG established authority earlier; its
        // supporting line supplies continuous depth at the column edge.
        assert_eq!(
            source_ray_supporting_line_depth([0, 0], [1.0, 0.2], [10, -1], [10, 1]),
            Some(10.0)
        );
        assert_eq!(
            source_ray_supporting_line_depth([0, 0], [1.0, -0.2], [10, -1], [10, 1]),
            Some(10.0)
        );
    }

    #[test]
    fn unresolved_supporting_line_depth_still_fails_open() {
        assert_eq!(
            source_ray_supporting_line_depth([0, 0], [1.0, 0.0], [10, 0], [20, 0]),
            None
        );
        assert_eq!(
            source_ray_supporting_line_depth([0, 0], [1.0, 0.0], [-10, -1], [-10, 1]),
            None
        );
    }

    #[test]
    fn paired_sky_boundary_without_plane_interval_does_not_fabricate_coverage() {
        let fixture = paired_sky_far_control_fixture().unwrap();
        let manifest = observe_authoritative_sky_regions(&fixture, 41, "static").unwrap();

        assert_eq!(manifest.input_sky_intervals, 0);
        assert_eq!(manifest.modeled_sky_intervals, 0);
        assert!(manifest.regions.is_empty());
        assert_eq!(manifest.omitted_sky_intervals, 0);
        assert_eq!(manifest.removed_non_sky_contributions, 0);
        assert!(!manifest.fail_open);
        assert!(manifest.paired_boundary_columns_observed > 0);
        assert_eq!(manifest.paired_boundary_columns_claimed, 0);
    }

    #[test]
    fn retained_sky_plane_intervals_become_conserved_normalized_regions() {
        let fixture = terminal_sky_ordered_fixture().unwrap();
        let manifest = observe_authoritative_sky_regions(&fixture, 41, "static").unwrap();

        assert!(manifest.input_sky_intervals > 0);
        assert_eq!(manifest.input_sky_intervals, manifest.modeled_sky_intervals);
        assert_eq!(manifest.input_sky_cells, manifest.modeled_sky_cells);
        assert_eq!(manifest.omitted_sky_intervals, 0);
        assert_eq!(manifest.removed_non_sky_contributions, 0);
        assert!(!manifest.fail_open);
        assert!(manifest.regions.iter().all(|region| {
            region.upper_boundary.len() < region.oracle_intervals * 2
                && region.lower_boundary.len() < region.oracle_intervals * 2
        }));
    }

    #[test]
    fn one_sky_boundary_does_not_gain_paired_sky_authority() {
        let fixture = one_sky_far_control_fixture().unwrap();
        let manifest = observe_authoritative_sky_regions(&fixture, 41, "static").unwrap();

        assert_eq!(manifest.input_sky_intervals, 0);
        assert_eq!(manifest.input_sky_intervals, manifest.modeled_sky_intervals);
        assert!(manifest.regions.is_empty());
        assert_eq!(manifest.paired_boundary_columns_observed, 0);
        assert_eq!(manifest.paired_boundary_columns_claimed, 0);
    }

    #[test]
    fn nearby_ordinary_geometry_is_outside_the_model_by_construction() {
        let fixture = vertical_aperture_control_fixture().unwrap();
        let manifest = observe_authoritative_sky_regions(&fixture, 41, "static").unwrap();

        assert_eq!(manifest.input_sky_intervals, 0);
        assert!(manifest.regions.is_empty());
        assert_eq!(manifest.removed_non_sky_contributions, 0);
    }

    #[test]
    fn missing_authority_fails_open_with_an_explained_outcome() {
        let mut vertical = DoomSegClassicVerticalClipObservation::default();
        let key = DoomSegClassicPlaneKey {
            kind: DoomSegClassicPlaneKind::Ceiling,
            height: 0,
            texture: "F_SKY1".to_owned(),
            light: 0,
        };
        let mut columns = vec![None; 320];
        columns[160] = Some([20, 40]);
        vertical.plane_spans.keys.insert(
            key,
            vec![DoomSegClassicPlaneInstance {
                columns,
                column_sources: vec![None; 320],
                minimum_column: 160,
                maximum_column: 160,
                source_sectors: BTreeSet::from([7]),
                source_segs: BTreeSet::from([11]),
            }],
        );
        let manifest = model_authoritative_sky_regions(
            &vertical,
            &[11],
            AuthoritativeSkyViewIdentity {
                fixture: "ambiguous-control".to_owned(),
                source_position: [0, 0],
                heading_radians: 0.0,
                source_eye_height: 41,
            },
            "declared",
        );

        assert!(manifest.fail_open);
        assert!(manifest.regions.is_empty());
        assert_eq!(manifest.input_sky_intervals, 1);
        assert_eq!(manifest.modeled_sky_intervals, 0);
        assert_eq!(manifest.omitted_sky_intervals, 1);
        assert_eq!(manifest.input_sky_cells, manifest.omitted_sky_cells);
        assert_eq!(
            manifest.ledger_outcomes[0].outcome,
            AuthoritativeSkyLedgerOutcomeKind::Omitted(
                AuthoritativeSkyOmissionReason::MissingSourceAuthority
            )
        );
    }

    #[test]
    fn unadmitted_source_seg_fails_open_with_an_explained_outcome() {
        let mut vertical = DoomSegClassicVerticalClipObservation::default();
        let key = DoomSegClassicPlaneKey {
            kind: DoomSegClassicPlaneKind::Ceiling,
            height: 0,
            texture: "F_SKY1".to_owned(),
            light: 0,
        };
        let mut columns = vec![None; 320];
        columns[160] = Some([20, 40]);
        let mut column_sources = vec![None; 320];
        column_sources[160] = Some([7, 12]);
        vertical.plane_spans.keys.insert(
            key,
            vec![DoomSegClassicPlaneInstance {
                columns,
                column_sources,
                minimum_column: 160,
                maximum_column: 160,
                source_sectors: BTreeSet::from([7]),
                source_segs: BTreeSet::from([12]),
            }],
        );
        let manifest = model_authoritative_sky_regions(
            &vertical,
            &[11],
            AuthoritativeSkyViewIdentity {
                fixture: "unadmitted-control".to_owned(),
                source_position: [0, 0],
                heading_radians: 0.0,
                source_eye_height: 41,
            },
            "declared",
        );

        assert!(manifest.fail_open);
        assert_eq!(manifest.omitted_sky_intervals, 1);
        assert_eq!(
            manifest.ledger_outcomes[0].outcome,
            AuthoritativeSkyLedgerOutcomeKind::Omitted(
                AuthoritativeSkyOmissionReason::SourceSegNotAdmitted
            )
        );
    }

    #[test]
    fn invalid_interval_fails_open_with_an_explained_outcome() {
        let mut vertical = DoomSegClassicVerticalClipObservation::default();
        let key = DoomSegClassicPlaneKey {
            kind: DoomSegClassicPlaneKind::Ceiling,
            height: 0,
            texture: "F_SKY1".to_owned(),
            light: 0,
        };
        let mut columns = vec![None; 320];
        columns[160] = Some([40, 20]);
        let mut column_sources = vec![None; 320];
        column_sources[160] = Some([7, 11]);
        vertical.plane_spans.keys.insert(
            key,
            vec![DoomSegClassicPlaneInstance {
                columns,
                column_sources,
                minimum_column: 160,
                maximum_column: 160,
                source_sectors: BTreeSet::from([7]),
                source_segs: BTreeSet::from([11]),
            }],
        );
        let manifest = model_authoritative_sky_regions(
            &vertical,
            &[11],
            AuthoritativeSkyViewIdentity {
                fixture: "invalid-interval-control".to_owned(),
                source_position: [0, 0],
                heading_radians: 0.0,
                source_eye_height: 41,
            },
            "declared",
        );

        assert!(manifest.fail_open);
        assert_eq!(manifest.omitted_sky_intervals, 1);
        assert_eq!(
            manifest.ledger_outcomes[0].outcome,
            AuthoritativeSkyLedgerOutcomeKind::Omitted(
                AuthoritativeSkyOmissionReason::InvalidInterval
            )
        );
    }

    #[test]
    fn continuous_depth_declarations_do_not_reconstruct_oracle_cells() {
        let fixture = terminal_sky_ordered_fixture().unwrap();
        let regions = observe_authoritative_sky_regions(&fixture, 41, "static").unwrap();
        let depth = prepare_authoritative_sky_depth_declarations(&regions, 0.25, "sky-1");

        assert_eq!(depth.declarations.len(), regions.regions.len());
        assert_eq!(depth.outcomes.len(), regions.regions.len());
        assert_eq!(depth.persistent_material_key, "sky-1");
        assert_eq!(depth.persistent_mesh_identities, 0);
        assert!(depth
            .declarations
            .iter()
            .all(|declaration| declaration.positions.len() % 3 == 0));
        assert!(depth.declarations.iter().all(|declaration| {
            declaration.triangle_count * 3 == declaration.positions.len()
                && declaration.triangle_count < regions.input_sky_cells
        }));
    }

    #[test]
    fn empty_authority_produces_no_declaration_or_persistent_identity() {
        let fixture = one_sky_far_control_fixture().unwrap();
        let regions = observe_authoritative_sky_regions(&fixture, 41, "static").unwrap();
        let depth = prepare_authoritative_sky_depth_declarations(&regions, 0.25, "sky-1");

        assert!(depth.declarations.is_empty());
        assert!(depth.outcomes.is_empty());
        assert_eq!(depth.persistent_mesh_identities, 0);
    }

    #[test]
    fn invalid_and_near_plane_depths_are_bounded_rejections() {
        let fixture = terminal_sky_ordered_fixture().unwrap();
        let regions = observe_authoritative_sky_regions(&fixture, 41, "static").unwrap();
        for (depth, expected) in [
            (f32::NAN, AuthoritativeSkyDepthRejection::InvalidDepth),
            (1.1, AuthoritativeSkyDepthRejection::InvalidDepth),
            (-1.0, AuthoritativeSkyDepthRejection::NearPlaneDepth),
        ] {
            let declarations =
                prepare_authoritative_sky_depth_declarations(&regions, depth, "sky-1");
            assert!(declarations.declarations.is_empty());
            assert_eq!(declarations.persistent_mesh_identities, 0);
            assert!(declarations
                .outcomes
                .iter()
                .all(|outcome| outcome.rejection == Some(expected)));
        }
    }

    #[test]
    fn prepared_view_change_changes_ephemeral_declaration_identity() {
        let fixture = terminal_sky_ordered_fixture().unwrap();
        let first_regions = observe_authoritative_sky_regions(&fixture, 41, "static").unwrap();
        let first = prepare_authoritative_sky_depth_declarations(&first_regions, 0.25, "sky-1");

        let mut moved_fixture = fixture;
        moved_fixture.viewer.position[0] += 1;
        let moved_regions =
            observe_authoritative_sky_regions(&moved_fixture, 41, "static").unwrap();
        let moved = prepare_authoritative_sky_depth_declarations(&moved_regions, 0.25, "sky-1");

        assert_ne!(first.structural_fingerprint, moved.structural_fingerprint);
        assert_eq!(first.persistent_material_key, moved.persistent_material_key);
        assert_eq!(first.persistent_mesh_identities, 0);
        assert_eq!(moved.persistent_mesh_identities, 0);
    }
}
