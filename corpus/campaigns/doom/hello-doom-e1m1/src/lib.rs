//! Application-local lowering for the selected static E1M1 flat policy.
//!
//! Doom records stay at this corpus edge. The renderer receives only the
//! resulting ordinary mesh with supplied texture coordinates.

pub mod collision;
pub mod debug_console;
/// Doom-private ordered source-occurrence preparation shared by native and
/// browser corpus hosts.  It produces ordinary Tokimu declarations but does
/// not admit Doom semantics into the renderer boundary.
#[path = "bin/static_scene/presentation/ordered_occurrence.rs"]
pub mod ordered_occurrence;
pub mod specials;

use doom_geometry_provider::{
    DoomMiddleTextureObservation, DoomSegTexturedWallTriangle, DoomSkySurfaceObservation,
    DoomSurfacePlane, DoomSurfaceTriangle, DoomTextureExtent, DoomTexturedWallTriangle,
    DoomWallTextureRole,
};
use doom_map_provider::DoomSourceRecord;
use doom_raster_provider::{
    compose_doom_texture, decode_doom_flat, decode_doom_raster_globals,
    decode_doom_texture_catalog, indexed_image_from_doom_flat, lower_doom_indexed_image,
    DoomFlatDecodeLimits, DoomIndexedImage, DoomPatchDecodeLimits, DoomRasterDecodeLimits,
    DoomTextureComposeLimits, DoomTextureDecodeLimits,
};
use doom_wad_package::select_doom_episode_map;
use doom_wad_provider::WadManifest;
use thiserror::Error;
use tokimu::{
    Color, Material, MaterialHandle, Mesh, Rgba8TextureColorSpace, Rgba8TextureDescriptor,
    TextureAddressMode, TextureFilter, TextureHandle, TextureSampler,
};
use tokimu_core::math::{Mat4, Vec3, Vec4};

/// Source identity retained beside a submitted mesh rather than embedded in a
/// renderer type or material label.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct StaticFlatSource {
    pub subsector: DoomSourceRecord,
    pub sector: DoomSourceRecord,
    pub plane: DoomSurfacePlane,
    pub flat_name: String,
}

/// One opaque-candidate flat triangle under the Slice 5B static map-axis
/// policy. It does not mean that the flat has been selected for drawing.
#[derive(Clone, Debug, PartialEq)]
pub struct StaticFlatMesh {
    pub source: StaticFlatSource,
    pub mesh: Mesh,
}

/// The first static-scene flat assembly. Sky observations are retained rather
/// than inferred from a flat name, so a later sky policy can consume the
/// original source classification without changing the submitted mesh list.
#[derive(Clone, Debug, PartialEq)]
pub struct StaticFlatAssembly {
    pub opaque_flats: Vec<StaticFlatMesh>,
    pub omitted_sky: Vec<DoomSkySurfaceObservation>,
    /// Individually retained zero-area candidates; no normal is fabricated.
    pub omitted_degenerate: Vec<StaticFlatSource>,
}

/// One opaque-candidate wall mesh under the source-coordinate normalization
/// policy. Its texture name is evidence for a later application material
/// lookup, not a renderer material identity.
#[derive(Clone, Debug, PartialEq)]
pub struct StaticWallMesh {
    pub source_linedef: DoomSourceRecord,
    pub source_sidedef: DoomSourceRecord,
    pub source_sector: DoomSourceRecord,
    /// Retained Doom source orientation for texture-placement diagnostics. It
    /// is not a renderer culling or material property.
    pub side: doom_geometry_provider::DoomWallSideKind,
    pub role: DoomWallTextureRole,
    pub texture_name: String,
    pub mesh: Mesh,
}

/// One source-SEG-labelled wall mesh for the AR-0025 Stage 3B presentation
/// comparison. It is corpus-only: a SEG remains source topology rather than a
/// renderer mesh category.
#[derive(Clone, Debug, PartialEq)]
pub struct StaticSegWallMesh {
    pub source_seg: DoomSourceRecord,
    pub wall: StaticWallMesh,
}

#[derive(Clone, Debug, PartialEq)]
pub struct StaticWallAssembly {
    pub opaque_walls: Vec<StaticWallMesh>,
    pub omitted_masked_middles: Vec<DoomMiddleTextureObservation>,
    pub omitted_degenerate: Vec<DoomTexturedWallTriangle>,
}

/// Corpus-local cutout declaration for E1M1's source-classified masked
/// middles. The exact `0` cutoff is selected by this Doom consumer because
/// its raster provider lowers coverage to only RGBA8 alpha `0` or `255`.
/// It is neither a renderer API nor a claim that this value is suitable for
/// other sources.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ExperimentalCutoutIntent {
    pub discard_at_or_below_alpha: u8,
    pub depth_write: bool,
}

/// One source-traceable wall prepared for the AR-0023 cutout experiment.
/// The generic intent is deliberately declared here, at the Doom consumer
/// boundary; WAD/source terminology does not cross into renderer vocabulary.
#[derive(Clone, Debug, PartialEq)]
pub struct ExperimentalCutoutWall {
    pub source: DoomMiddleTextureObservation,
    pub wall: StaticWallMesh,
    pub intent: ExperimentalCutoutIntent,
}

/// Experimental masked-middle lowering results. Degenerate candidates remain
/// explicit evidence and no unrelated lowering failure is swallowed.
#[derive(Clone, Debug, PartialEq)]
pub struct ExperimentalCutoutWallAssembly {
    pub candidates: Vec<ExperimentalCutoutWall>,
    pub omitted_degenerate: Vec<DoomTexturedWallTriangle>,
}

/// Consumer-owned eligibility for the first opaque static scene. It neither
/// uploads pixels nor treats source coverage as a renderer alpha policy.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum StaticTextureEligibility {
    Opaque(StaticOpaqueTexture),
    DeferredAlpha {
        texture_name: String,
        uncovered_pixels: usize,
        /// Retained upload facts for a later caller-owned alpha experiment;
        /// their presence does not select an alpha policy or schedule upload.
        descriptor: Rgba8TextureDescriptor,
        sampler: TextureSampler,
        selected_palette: u16,
    },
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct StaticOpaqueTexture {
    pub texture_name: String,
    pub descriptor: Rgba8TextureDescriptor,
    pub sampler: TextureSampler,
    pub selected_palette: u16,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct FlatExtent {
    pub width: u16,
    pub height: u16,
}

impl FlatExtent {
    pub const E1M1: Self = Self {
        width: 64,
        height: 64,
    };
}

#[derive(Clone, Debug, Error, PartialEq)]
pub enum StaticFlatLoweringError {
    #[error("flat extent must be non-zero; got {width} by {height}")]
    ZeroExtent { width: u16, height: u16 },
    #[error("wall texture `{name}` has no supplied source extent")]
    MissingWallTextureExtent { name: String },
    #[error("surface position {vertex} has a non-finite {component} coordinate")]
    NonFinitePosition {
        vertex: usize,
        component: &'static str,
    },
    #[error("surface triangle is degenerate and has no stable facing normal")]
    DegenerateTriangle,
    #[error("renderer mesh validation failed: {0}")]
    Mesh(#[from] tokimu::MeshValidationError),
}

/// Source-traceable E1M1 flat preparation before texture upload or rendering.
#[derive(Clone, Debug, PartialEq)]
pub struct PreparedE1m1Flats {
    pub map_name: String,
    pub flat_assembly: StaticFlatAssembly,
}

#[derive(Clone, Debug, PartialEq)]
pub struct PreparedE1m1Walls {
    pub map_name: String,
    pub wall_assembly: StaticWallAssembly,
}

/// Source-traceable, non-submitted masked-middle candidates for the AR-0023
/// real-caller study.
#[derive(Clone, Debug, PartialEq)]
pub struct PreparedE1m1MaskedMiddleCutouts {
    pub map_name: String,
    pub assembly: ExperimentalCutoutWallAssembly,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PreparedStaticTexture {
    pub eligibility: StaticTextureEligibility,
    pub rgba8: Vec<u8>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum StaticTextureSourceKind {
    Flat,
    Wall,
}

#[derive(Clone, Debug, PartialEq)]
pub struct StaticTextureUpload {
    pub source_kind: StaticTextureSourceKind,
    pub source_name: String,
    pub texture: TextureHandle,
    pub material: MaterialHandle,
    pub descriptor: Rgba8TextureDescriptor,
    pub rgba8: Vec<u8>,
    pub material_value: Material,
}

/// Corpus-only source provenance kept beside a prepared draw while AR-0025
/// compares source topology with presentation candidate granularity. It never
/// crosses into renderer commands.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum StaticDrawSource {
    Flat {
        source_subsector: DoomSourceRecord,
        source_sector: DoomSourceRecord,
        plane: DoomSurfacePlane,
    },
    Wall {
        source_linedef: DoomSourceRecord,
        source_sidedef: DoomSourceRecord,
        source_sector: DoomSourceRecord,
        /// Retained only for corpus-local dynamic-span lowering. It does not
        /// become a renderer material or mesh property.
        role: DoomWallTextureRole,
    },
}

#[derive(Clone, Debug, PartialEq)]
pub struct StaticDrawPlanEntry {
    pub mesh: Mesh,
    pub material: MaterialHandle,
    pub source_label: String,
    pub source: StaticDrawSource,
}

/// Corpus-local conservative bounds for one prepared static-scene draw. The
/// bounds are evidence used by AR-0025 candidate selection; they are neither a
/// renderer resource nor a source identity.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct StaticDrawAabb {
    minimum: Vec3,
    maximum: Vec3,
}

/// Corpus-local enclosing sphere for one prepared static-scene draw. It exists
/// solely to compare conservative bound shapes in AR-0025; it is not renderer
/// or source identity vocabulary.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct StaticDrawSphere {
    center: Vec3,
    radius: f32,
}

/// One homogeneous clip plane that wholly excludes a prepared static draw.
/// This stays corpus-local while AR-0025 determines whether any general
/// candidate-selection vocabulary is earned.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum StaticDrawFrustumRejection {
    Left,
    Right,
    Bottom,
    Top,
    Near,
    Far,
}

type ClipPlaneTest = fn(Vec4) -> bool;

impl StaticDrawAabb {
    /// Builds finite ordered bounds for a corpus-only derived volume.
    pub fn from_minimum_maximum(minimum: Vec3, maximum: Vec3) -> Option<Self> {
        (minimum.is_finite()
            && maximum.is_finite()
            && minimum.x <= maximum.x
            && minimum.y <= maximum.y
            && minimum.z <= maximum.z)
            .then_some(Self { minimum, maximum })
    }

    /// Derives conservative bounds from supplied mesh positions. Empty or
    /// non-finite position streams produce no bounds so callers can fail open.
    pub fn from_positions(positions: &[[f32; 3]]) -> Option<Self> {
        let mut minimum = Vec3::splat(f32::INFINITY);
        let mut maximum = Vec3::splat(f32::NEG_INFINITY);
        for position in positions {
            let position = Vec3::new(position[0], position[1], position[2]);
            if !position.is_finite() {
                return None;
            }
            minimum = minimum.min(position);
            maximum = maximum.max(position);
        }
        (!positions.is_empty()).then_some(Self { minimum, maximum })
    }

    /// Returns the smallest AABB enclosing every supplied finite bound. An
    /// empty group has no declared bound so callers can retain it fail-open.
    pub fn enclosing(bounds: &[Self]) -> Option<Self> {
        Self::enclosing_iter(bounds.iter().copied())
    }

    /// Iterator form used by corpus grouping experiments to avoid making a
    /// group allocation part of the selection measurement.
    pub fn enclosing_iter(bounds: impl IntoIterator<Item = Self>) -> Option<Self> {
        let mut iter = bounds.into_iter();
        let first = iter.next()?;
        Some(iter.fold(first, |combined, bounds| Self {
            minimum: combined.minimum.min(bounds.minimum),
            maximum: combined.maximum.max(bounds.maximum),
        }))
    }

    /// Corpus-only derived-bound accessors. These expose no renderer resource
    /// or source identity and exist solely for AR-0025 index experiments.
    pub const fn minimum(self) -> Vec3 {
        self.minimum
    }

    /// See [`Self::minimum`].
    pub const fn maximum(self) -> Vec3 {
        self.maximum
    }

    fn corners(self) -> [Vec3; 8] {
        let min = self.minimum;
        let max = self.maximum;
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

impl StaticDrawSphere {
    /// Derives an enclosing sphere from supplied finite mesh positions. Empty
    /// or non-finite input produces no sphere so callers can fail open.
    pub fn from_positions(positions: &[[f32; 3]]) -> Option<Self> {
        let bounds = StaticDrawAabb::from_positions(positions)?;
        let center = (bounds.minimum + bounds.maximum) * 0.5;
        let mut radius_squared = 0.0_f32;
        for position in positions {
            let position = Vec3::new(position[0], position[1], position[2]);
            if !position.is_finite() {
                return None;
            }
            radius_squared = radius_squared.max(center.distance_squared(position));
        }
        Some(Self {
            center,
            radius: radius_squared.sqrt(),
        })
    }
}

/// Returns a rejection only when every AABB corner lies outside the same
/// homogeneous GL-style clip plane. It is therefore conservative: intersecting
/// or uncertain geometry remains a candidate.
pub fn classify_static_draw_frustum_rejection(
    bounds: StaticDrawAabb,
    view_projection: Mat4,
) -> Option<StaticDrawFrustumRejection> {
    let clip = bounds
        .corners()
        .map(|point| view_projection * Vec4::new(point.x, point.y, point.z, 1.0));
    let tests: [(StaticDrawFrustumRejection, ClipPlaneTest); 6] = [
        (StaticDrawFrustumRejection::Left, |point| point.x < -point.w),
        (StaticDrawFrustumRejection::Right, |point| point.x > point.w),
        (StaticDrawFrustumRejection::Bottom, |point| {
            point.y < -point.w
        }),
        (StaticDrawFrustumRejection::Top, |point| point.y > point.w),
        (StaticDrawFrustumRejection::Near, |point| point.z < -point.w),
        (StaticDrawFrustumRejection::Far, |point| point.z > point.w),
    ];
    tests
        .into_iter()
        .find_map(|(reason, outside)| clip.iter().copied().all(outside).then_some(reason))
}

/// Returns a rejection only when the complete enclosing sphere lies outside a
/// homogeneous GL-style clip plane. Sphere tests are deliberately compared
/// with AABBs as AR-0025 evidence; neither shape is an admitted contract.
pub fn classify_static_draw_sphere_frustum_rejection(
    sphere: StaticDrawSphere,
    view_projection: Mat4,
) -> Option<StaticDrawFrustumRejection> {
    let row = |index| {
        let columns = [
            view_projection.x_axis,
            view_projection.y_axis,
            view_projection.z_axis,
            view_projection.w_axis,
        ];
        Vec4::new(
            columns[0][index],
            columns[1][index],
            columns[2][index],
            columns[3][index],
        )
    };
    let left = row(0) + row(3);
    let right = row(3) - row(0);
    let bottom = row(1) + row(3);
    let top = row(3) - row(1);
    let near = row(2) + row(3);
    let far = row(3) - row(2);
    [
        (StaticDrawFrustumRejection::Left, left),
        (StaticDrawFrustumRejection::Right, right),
        (StaticDrawFrustumRejection::Bottom, bottom),
        (StaticDrawFrustumRejection::Top, top),
        (StaticDrawFrustumRejection::Near, near),
        (StaticDrawFrustumRejection::Far, far),
    ]
    .into_iter()
    .find_map(|(reason, plane)| {
        let distance = plane.truncate().dot(sphere.center) + plane.w;
        let support = plane.truncate().length() * sphere.radius;
        (distance + support < 0.0).then_some(reason)
    })
}

/// Maps classic Doom heading degrees onto this corpus's X/Z world convention:
/// angle zero points +X and 90 points +Z. It supplies no player policy.
pub fn doom_heading_forward(angle: u16) -> Vec3 {
    let radians = f32::from(angle).to_radians();
    Vec3::new(radians.cos(), 0.0, radians.sin())
}

/// Converts a corpus X/Z source heading into observer-camera yaw, where yaw
/// zero points +Z.
pub fn observer_yaw_from_forward(forward: Vec3) -> f32 {
    forward.x.atan2(forward.z)
}

/// Converts classic Doom heading degrees into the current observer yaw. Doom
/// zero points +X; observer yaw zero points +Z.
pub fn doom_heading_degrees_to_observer_yaw(degrees: f32) -> f32 {
    observer_yaw_from_forward(Vec3::new(
        degrees.to_radians().cos(),
        0.0,
        degrees.to_radians().sin(),
    ))
}

/// Inverse orientation conversion retained for AR-0028 round-trip evidence.
pub fn observer_yaw_to_doom_heading_degrees(yaw: f32) -> f32 {
    (90.0 - yaw.to_degrees()).rem_euclid(360.0)
}

/// Produces a right-handed first-person observer direction. Positive yaw turns
/// from +Z toward +X; positive pitch looks up. It is corpus evidence helper,
/// not an admitted runtime input or player policy.
pub fn observer_direction(yaw: f32, pitch: f32) -> Vec3 {
    Vec3::new(
        yaw.sin() * pitch.cos(),
        pitch.sin(),
        yaw.cos() * pitch.cos(),
    )
}

/// Derives this corpus observer's horizontal screen-right direction from a
/// world-space forward vector and its declared `+Y` up axis.
///
/// This helper retains the cross-product order that `look_at_rh` presents as
/// camera right. It is corpus evidence for AR-0028, not an admitted Tokimu
/// camera-basis contract.
pub fn observer_right(forward: Vec3) -> Vec3 {
    Vec3::new(forward.x, 0.0, forward.z)
        .normalize_or_zero()
        .cross(Vec3::Y)
        .normalize_or_zero()
}

/// Headless evidence for how the current Doom ground-plane lift relates to
/// this corpus observer's right-handed camera basis.
///
/// The observation deliberately separates an invertible coordinate mapping
/// from orientation preservation. A source conversion can round-trip every
/// number while still reversing the signed ground-plane orientation relative
/// to Tokimu world `+Y`.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct DoomGroundFrameObservation {
    pub embedding: DoomComparativeEmbedding,
    pub source_right: [f32; 2],
    pub source_forward: [f32; 2],
    pub source_signed_orientation: f32,
    pub lifted_right: Vec3,
    pub lifted_forward: Vec3,
    pub lifted_orientation_about_world_up: f32,
    pub camera_right: Vec3,
    pub source_right_camera_right_alignment: f32,
}

/// Corpus-only alternatives for AR-0028. These do not change the active Doom
/// provider conversion or establish a Tokimu world-axis contract.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DoomComparativeEmbedding {
    CurrentReflected,
    PreserveEast,
    PreserveNorth,
}

impl DoomComparativeEmbedding {
    pub const ALL: [Self; 3] = [
        Self::CurrentReflected,
        Self::PreserveEast,
        Self::PreserveNorth,
    ];

    /// Lifts unchanged decoded Doom direction facts into one experimental
    /// world frame.
    pub fn lift_direction(self, source_xy: [f32; 2], vertical: f32) -> Vec3 {
        match self {
            Self::CurrentReflected => Vec3::new(source_xy[0], vertical, source_xy[1]),
            Self::PreserveEast => Vec3::new(source_xy[0], vertical, -source_xy[1]),
            Self::PreserveNorth => Vec3::new(-source_xy[0], vertical, source_xy[1]),
        }
    }

    /// Exact corpus inverse retained separately from orientation behavior.
    pub fn lower_direction(self, world: Vec3) -> ([f32; 2], f32) {
        match self {
            Self::CurrentReflected => ([world.x, world.z], world.y),
            Self::PreserveEast => ([world.x, -world.z], world.y),
            Self::PreserveNorth => ([-world.x, world.z], world.y),
        }
    }

    /// Converts an unchanged decoded Doom heading through this experimental
    /// embedding. This remains corpus comparison machinery rather than an
    /// admitted camera or spatial-frame API.
    pub fn lift_heading_degrees(self, degrees: f32) -> Vec3 {
        let radians = degrees.to_radians();
        self.lift_direction([radians.cos(), radians.sin()], 0.0)
            .normalize_or_zero()
    }
}

/// Re-embeds one already-lowered corpus mesh for AR-0028 comparison.
///
/// Candidate embeddings reflect the current prepared positions, so the helper
/// rebuilds triangle winding and normals. `reverse_u` is deliberately explicit:
/// Doom walls currently compensate for the reflected lift, while flat UVs do
/// not share that wall-specific policy.
pub fn reembed_comparative_mesh(
    mesh: &mut Mesh,
    embedding: DoomComparativeEmbedding,
    reverse_u: bool,
) {
    if embedding == DoomComparativeEmbedding::CurrentReflected {
        return;
    }
    for triangle_index in 0..mesh.positions.len() / 3 {
        let base = triangle_index * 3;
        for index in base..base + 3 {
            let current = mesh.positions[index];
            mesh.positions[index] = embedding
                .lift_direction([current[0], current[2]], current[1])
                .to_array();
        }
        mesh.positions.swap(base + 1, base + 2);
        if mesh.has_texture_coordinates() {
            mesh.texture_coordinates.swap(base + 1, base + 2);
        }

        if reverse_u && mesh.has_texture_coordinates() {
            let minimum_u = mesh.texture_coordinates[base..base + 3]
                .iter()
                .map(|uv| uv[0])
                .fold(f32::INFINITY, f32::min);
            let maximum_u = mesh.texture_coordinates[base..base + 3]
                .iter()
                .map(|uv| uv[0])
                .fold(f32::NEG_INFINITY, f32::max);
            for uv in &mut mesh.texture_coordinates[base..base + 3] {
                uv[0] = minimum_u + maximum_u - uv[0];
            }
        }

        let a = Vec3::from_array(mesh.positions[base]);
        let b = Vec3::from_array(mesh.positions[base + 1]);
        let c = Vec3::from_array(mesh.positions[base + 2]);
        let normal = (b - a).cross(c - a).normalize_or_zero().to_array();
        mesh.normals[base..base + 3].fill(normal);
    }
}

/// Observes, but does not repair, the current Doom-to-world ground frame.
/// This remains corpus evidence for AR-0028 rather than a public spatial API.
pub fn observe_doom_ground_frame(
    source_right: [f32; 2],
    source_forward: [f32; 2],
) -> DoomGroundFrameObservation {
    observe_doom_ground_frame_with_embedding(
        DoomComparativeEmbedding::CurrentReflected,
        source_right,
        source_forward,
    )
}

/// Observes one corpus-only candidate without changing active provider code.
pub fn observe_doom_ground_frame_with_embedding(
    embedding: DoomComparativeEmbedding,
    source_right: [f32; 2],
    source_forward: [f32; 2],
) -> DoomGroundFrameObservation {
    let lifted_right = embedding.lift_direction(source_right, 0.0);
    let lifted_forward = embedding.lift_direction(source_forward, 0.0);
    let camera_right = observer_right(lifted_forward);

    DoomGroundFrameObservation {
        embedding,
        source_right,
        source_forward,
        source_signed_orientation: source_right[0] * source_forward[1]
            - source_right[1] * source_forward[0],
        lifted_right,
        lifted_forward,
        lifted_orientation_about_world_up: lifted_right.cross(lifted_forward).dot(Vec3::Y),
        camera_right,
        source_right_camera_right_alignment: lifted_right.dot(camera_right),
    }
}

/// Aggregate impact of the explicitly omitted zero-area candidates. This is
/// evidence for the Slice 5B escalation rule, not renderer input.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct StaticDegenerateOmissionImpact {
    pub flat_candidates: usize,
    pub flat_subsectors: usize,
    pub flat_sectors: usize,
    pub fully_omitted_flat_subsectors: usize,
    pub wall_candidates: usize,
    pub wall_linedefs: usize,
    pub wall_sidedefs: usize,
    pub wall_sectors: usize,
    pub fully_omitted_wall_linedefs: usize,
}

/// One source-level wall omission summary. It preserves the vertical span that
/// the lowerer attempted, making a collapsed authored span distinguishable
/// from a later conversion failure.
#[derive(Clone, Debug, PartialEq)]
pub struct StaticDegenerateWallOmission {
    pub linedef_index: u32,
    pub sidedef_index: u32,
    pub sector_index: u32,
    pub texture_name: String,
    pub role: DoomWallTextureRole,
    pub minimum_height: f64,
    pub maximum_height: f64,
}

#[derive(Clone, Debug, Error, Eq, PartialEq)]
pub enum StaticDrawPlanError {
    #[error("no opaque material was prepared for {source_kind:?} texture `{source_name}`")]
    MissingMaterial {
        source_kind: StaticTextureSourceKind,
        source_name: String,
    },
}

/// Creates deterministic opaque-only renderer inputs. Deferred-alpha sources
/// intentionally receive no handle or material.
pub fn build_static_texture_uploads(
    flat_textures: &[PreparedStaticTexture],
    wall_textures: &[PreparedStaticTexture],
) -> Vec<StaticTextureUpload> {
    let mut uploads = Vec::new();
    for (kind, textures) in [
        (StaticTextureSourceKind::Flat, flat_textures),
        (StaticTextureSourceKind::Wall, wall_textures),
    ] {
        let mut textures = textures.iter().collect::<Vec<_>>();
        textures.sort_by_key(|texture| match &texture.eligibility {
            StaticTextureEligibility::Opaque(texture) => texture.texture_name.as_str(),
            StaticTextureEligibility::DeferredAlpha { texture_name, .. } => texture_name.as_str(),
        });
        for texture in textures {
            let StaticTextureEligibility::Opaque(opaque) = &texture.eligibility else {
                continue;
            };
            let index = uploads.len() as u64 + 1;
            uploads.push(StaticTextureUpload {
                source_kind: kind,
                source_name: opaque.texture_name.clone(),
                texture: TextureHandle(index),
                material: MaterialHandle(index),
                descriptor: opaque.descriptor,
                rgba8: texture.rgba8.clone(),
                material_value: Material::new(
                    format!("doom-{kind:?}-{}", opaque.texture_name),
                    Color::rgb(1.0, 1.0, 1.0),
                )
                .with_texture(TextureHandle(index))
                .with_texture_sampler(opaque.sampler),
            });
        }
    }
    uploads
}

/// Builds corpus-only upload inputs for already-declared cutout candidates.
/// The caller supplies the first free handle so these resources cannot alias
/// the opaque static scene. This helper does not upload, schedule, or choose a
/// pipeline.
pub fn build_experimental_cutout_texture_uploads(
    textures: &[PreparedStaticTexture],
    first_handle: u64,
) -> Vec<StaticTextureUpload> {
    // Source classification, not the texture's coverage bytes, chose these
    // inputs. A Doom masked middle can be fully covered (for example,
    // `BROWNGRN`) and still be part of the caller's cutout experiment.
    let mut selected = textures
        .iter()
        .map(|texture| match &texture.eligibility {
            StaticTextureEligibility::Opaque(opaque) => (
                &opaque.texture_name,
                &opaque.descriptor,
                &opaque.sampler,
                &texture.rgba8,
            ),
            StaticTextureEligibility::DeferredAlpha {
                texture_name,
                descriptor,
                sampler,
                ..
            } => (texture_name, descriptor, sampler, &texture.rgba8),
        })
        .collect::<Vec<_>>();
    selected.sort_by_key(|(name, _, _, _)| *name);
    selected
        .into_iter()
        .enumerate()
        .map(|(offset, (name, descriptor, sampler, rgba8))| {
            let handle = first_handle + offset as u64;
            StaticTextureUpload {
                source_kind: StaticTextureSourceKind::Wall,
                source_name: name.clone(),
                texture: TextureHandle(handle),
                material: MaterialHandle(handle),
                descriptor: *descriptor,
                rgba8: rgba8.clone(),
                material_value: Material::new(
                    format!("doom-cutout-candidate-{name}"),
                    Color::rgb(1.0, 1.0, 1.0),
                )
                .with_texture(TextureHandle(handle))
                .with_texture_sampler(*sampler),
            }
        })
        .collect()
}

/// Corpus-local WGSL for the E1M1 masked-middle experiment. It is deliberately
/// not a public renderer shader contract: the Doom consumer has already
/// selected the candidate and supplied a binary-alpha declaration.
pub fn experimental_masked_cutout_wgsl() -> &'static str {
    r#"
@group(0) @binding(0) var<uniform> material_color: vec4<f32>;
@group(0) @binding(1) var material_texture: texture_2d<f32>;
@group(0) @binding(2) var material_sampler: sampler;
struct InstanceParams { translation: vec2<f32>, scale: vec2<f32>, rotation: vec2<f32>, padding: vec2<f32>, };
@group(1) @binding(0) var<uniform> _instance_params: InstanceParams;
@group(2) @binding(0) var<uniform> camera_params: mat4x4<f32>;
struct VertexOutput { @builtin(position) position: vec4<f32>, @location(0) uv: vec2<f32>, };
@vertex fn vs_main(@location(0) position: vec3<f32>, @location(1) _normal: vec3<f32>, @location(2) uv: vec2<f32>) -> VertexOutput {
    var output: VertexOutput;
    output.position = camera_params * vec4<f32>(position, 1.0);
    output.uv = uv;
    return output;
}
@fragment fn fs_main(@location(0) uv: vec2<f32>) -> @location(0) vec4<f32> {
    let sampled = textureSample(material_texture, material_sampler, uv) * material_color;
    if (sampled.a <= 0.0) { discard; }
    return sampled;
}
"#
    .trim()
}

/// Lowers prepared application evidence to ordinary mesh/material draw inputs.
/// The renderer receives neither Doom identifiers nor source classifications.
pub fn build_static_draw_plan(
    flats: &PreparedE1m1Flats,
    walls: &PreparedE1m1Walls,
    uploads: &[StaticTextureUpload],
) -> Result<Vec<StaticDrawPlanEntry>, StaticDrawPlanError> {
    let material_for = |kind, name: &str| {
        uploads
            .iter()
            .find(|upload| {
                upload.source_kind == kind && upload.source_name.eq_ignore_ascii_case(name)
            })
            .map(|upload| upload.material)
            .ok_or_else(|| StaticDrawPlanError::MissingMaterial {
                source_kind: kind,
                source_name: name.to_owned(),
            })
    };
    let mut draws = Vec::with_capacity(
        flats.flat_assembly.opaque_flats.len() + walls.wall_assembly.opaque_walls.len(),
    );
    for flat in &flats.flat_assembly.opaque_flats {
        draws.push(StaticDrawPlanEntry {
            mesh: flat.mesh.clone(),
            material: material_for(StaticTextureSourceKind::Flat, &flat.source.flat_name)?,
            source_label: format!(
                "flat:{}:{}",
                flat.source.sector.record_index, flat.source.flat_name
            ),
            source: StaticDrawSource::Flat {
                source_subsector: flat.source.subsector,
                source_sector: flat.source.sector,
                plane: flat.source.plane,
            },
        });
    }
    for wall in &walls.wall_assembly.opaque_walls {
        draws.push(StaticDrawPlanEntry {
            mesh: wall.mesh.clone(),
            material: material_for(StaticTextureSourceKind::Wall, &wall.texture_name)?,
            source_label: format!(
                "wall:{}:{}",
                wall.source_linedef.record_index, wall.texture_name
            ),
            source: StaticDrawSource::Wall {
                source_linedef: wall.source_linedef,
                source_sidedef: wall.source_sidedef,
                source_sector: wall.source_sector,
                role: wall.role,
            },
        });
    }
    Ok(draws)
}

/// Converts only the corpus-local experimental cutout candidates into ordinary
/// renderer inputs. Source classification and alpha declaration remain beside
/// this consumer path rather than inside `tokimu-render`.
pub fn build_experimental_cutout_draw_plan(
    prepared: &PreparedE1m1MaskedMiddleCutouts,
    uploads: &[StaticTextureUpload],
) -> Result<Vec<StaticDrawPlanEntry>, StaticDrawPlanError> {
    prepared
        .assembly
        .candidates
        .iter()
        .map(|candidate| {
            let material = uploads
                .iter()
                .find(|upload| {
                    upload
                        .source_name
                        .eq_ignore_ascii_case(&candidate.wall.texture_name)
                })
                .map(|upload| upload.material)
                .ok_or_else(|| StaticDrawPlanError::MissingMaterial {
                    source_kind: StaticTextureSourceKind::Wall,
                    source_name: candidate.wall.texture_name.clone(),
                })?;
            Ok(StaticDrawPlanEntry {
                mesh: candidate.wall.mesh.clone(),
                material,
                source_label: format!(
                    "cutout:{}:{}",
                    candidate.wall.source_linedef.record_index, candidate.wall.texture_name
                ),
                source: StaticDrawSource::Wall {
                    source_linedef: candidate.wall.source_linedef,
                    source_sidedef: candidate.wall.source_sidedef,
                    source_sector: candidate.wall.source_sector,
                    role: candidate.wall.role,
                },
            })
        })
        .collect()
}

/// Counts the independently identifiable source records affected by the
/// intentionally omitted degenerate candidates. A caller can use this compact
/// report to decide whether a local omission remains acceptable evidence or
/// has become systemic topology loss.
pub fn static_degenerate_omission_impact(
    flats: &PreparedE1m1Flats,
    walls: &PreparedE1m1Walls,
) -> StaticDegenerateOmissionImpact {
    let source_key = |source: DoomSourceRecord| (source.lump_index, source.record_index);
    let flat_subsectors = flats
        .flat_assembly
        .omitted_degenerate
        .iter()
        .map(|source| source_key(source.subsector))
        .collect::<std::collections::BTreeSet<_>>();
    let mut flat_subsector_totals = std::collections::BTreeMap::<(u32, u32), usize>::new();
    let mut flat_subsector_omissions = std::collections::BTreeMap::<(u32, u32), usize>::new();
    for flat in &flats.flat_assembly.opaque_flats {
        *flat_subsector_totals
            .entry(source_key(flat.source.subsector))
            .or_default() += 1;
    }
    for flat in &flats.flat_assembly.omitted_degenerate {
        let key = source_key(flat.subsector);
        *flat_subsector_totals.entry(key).or_default() += 1;
        *flat_subsector_omissions.entry(key).or_default() += 1;
    }
    let flat_sectors = flats
        .flat_assembly
        .omitted_degenerate
        .iter()
        .map(|source| source_key(source.sector))
        .collect::<std::collections::BTreeSet<_>>();
    let wall_linedefs = walls
        .wall_assembly
        .omitted_degenerate
        .iter()
        .map(|wall| source_key(wall.source_linedef))
        .collect::<std::collections::BTreeSet<_>>();
    let mut wall_linedef_totals = std::collections::BTreeMap::<(u32, u32), usize>::new();
    let mut wall_linedef_omissions = std::collections::BTreeMap::<(u32, u32), usize>::new();
    for wall in &walls.wall_assembly.opaque_walls {
        *wall_linedef_totals
            .entry(source_key(wall.source_linedef))
            .or_default() += 1;
    }
    for wall in &walls.wall_assembly.omitted_degenerate {
        let key = source_key(wall.source_linedef);
        *wall_linedef_totals.entry(key).or_default() += 1;
        *wall_linedef_omissions.entry(key).or_default() += 1;
    }
    let wall_sidedefs = walls
        .wall_assembly
        .omitted_degenerate
        .iter()
        .map(|wall| source_key(wall.source_sidedef))
        .collect::<std::collections::BTreeSet<_>>();
    let wall_sectors = walls
        .wall_assembly
        .omitted_degenerate
        .iter()
        .map(|wall| source_key(wall.source_sector))
        .collect::<std::collections::BTreeSet<_>>();
    StaticDegenerateOmissionImpact {
        flat_candidates: flats.flat_assembly.omitted_degenerate.len(),
        flat_subsectors: flat_subsectors.len(),
        flat_sectors: flat_sectors.len(),
        fully_omitted_flat_subsectors: flat_subsector_omissions
            .iter()
            .filter(|(key, omitted)| flat_subsector_totals.get(*key) == Some(omitted))
            .count(),
        wall_candidates: walls.wall_assembly.omitted_degenerate.len(),
        wall_linedefs: wall_linedefs.len(),
        wall_sidedefs: wall_sidedefs.len(),
        wall_sectors: wall_sectors.len(),
        fully_omitted_wall_linedefs: wall_linedef_omissions
            .iter()
            .filter(|(key, omitted)| wall_linedef_totals.get(*key) == Some(omitted))
            .count(),
    }
}

/// Stable source-linedef identities whose complete emitted wall candidate set
/// was omitted as degenerate. This is intentionally diagnostic-only so a
/// caller can inspect concrete topology without broadening render input.
pub fn fully_omitted_wall_linedef_indices(walls: &PreparedE1m1Walls) -> Vec<u32> {
    let source_key = |source: DoomSourceRecord| (source.lump_index, source.record_index);
    let mut totals = std::collections::BTreeMap::<(u32, u32), usize>::new();
    let mut omitted = std::collections::BTreeMap::<(u32, u32), usize>::new();
    for wall in &walls.wall_assembly.opaque_walls {
        *totals.entry(source_key(wall.source_linedef)).or_default() += 1;
    }
    for wall in &walls.wall_assembly.omitted_degenerate {
        let key = source_key(wall.source_linedef);
        *totals.entry(key).or_default() += 1;
        *omitted.entry(key).or_default() += 1;
    }
    omitted
        .into_iter()
        .filter_map(|(key @ (_, index), count)| (totals.get(&key) == Some(&count)).then_some(index))
        .collect()
}

/// Stable, one-per-linedef detail for the full wall-omission cases.
pub fn fully_omitted_wall_details(walls: &PreparedE1m1Walls) -> Vec<StaticDegenerateWallOmission> {
    let fully_omitted = fully_omitted_wall_linedef_indices(walls)
        .into_iter()
        .collect::<std::collections::BTreeSet<_>>();
    let mut details = walls
        .wall_assembly
        .omitted_degenerate
        .iter()
        .filter(|wall| fully_omitted.contains(&wall.source_linedef.record_index))
        .map(|wall| {
            let heights = wall.positions.map(|position| position[1]);
            StaticDegenerateWallOmission {
                linedef_index: wall.source_linedef.record_index,
                sidedef_index: wall.source_sidedef.record_index,
                sector_index: wall.source_sector.record_index,
                texture_name: wall.texture_name.clone(),
                role: wall.role,
                minimum_height: heights.into_iter().fold(f64::INFINITY, f64::min),
                maximum_height: heights.into_iter().fold(f64::NEG_INFINITY, f64::max),
            }
        })
        .collect::<Vec<_>>();
    details.sort_by_key(|detail| detail.linedef_index);
    details.dedup_by_key(|detail| detail.linedef_index);
    details
}

/// Stable, sorted source names for the texture-upload stage. This is source
/// vocabulary at the application boundary, not a renderer handle table.
pub fn opaque_texture_names(textures: &[PreparedStaticTexture]) -> Vec<String> {
    let mut names = textures
        .iter()
        .filter_map(|texture| match &texture.eligibility {
            StaticTextureEligibility::Opaque(texture) => Some(texture.texture_name.clone()),
            StaticTextureEligibility::DeferredAlpha { .. } => None,
        })
        .collect::<Vec<_>>();
    names.sort();
    names.dedup();
    names
}

pub fn prepared_e1m1_wall_texture_names(prepared: &PreparedE1m1Walls) -> Vec<String> {
    let mut names = prepared
        .wall_assembly
        .opaque_walls
        .iter()
        .map(|wall| wall.texture_name.clone())
        .collect::<Vec<_>>();
    names.sort();
    names.dedup();
    names
}

/// Sorted, deduplicated authored names for the source-classified two-sided
/// masked-middle observations. These remain Doom-consumer evidence only: the
/// helper does not choose coverage policy or allocate renderer resources.
pub fn prepared_e1m1_masked_middle_texture_names(prepared: &PreparedE1m1Walls) -> Vec<String> {
    let mut names = prepared
        .wall_assembly
        .omitted_masked_middles
        .iter()
        .map(|middle| middle.texture_name.clone())
        .collect::<Vec<_>>();
    names.sort();
    names.dedup();
    names
}

/// Compact deterministic preparation evidence emitted before renderer upload.
pub fn prepared_e1m1_report(
    prepared: &PreparedE1m1Flats,
    textures: &[PreparedStaticTexture],
) -> String {
    let opaque_textures = textures
        .iter()
        .filter(|texture| matches!(texture.eligibility, StaticTextureEligibility::Opaque(_)))
        .count();
    let deferred_alpha = textures.len() - opaque_textures;
    format!(
        "map={} opaque_flat_triangles={} omitted_sky_surfaces={} omitted_degenerate_flats={} prepared_textures={} opaque_textures={} deferred_alpha_textures={}",
        prepared.map_name,
        prepared.flat_assembly.opaque_flats.len(),
        prepared.flat_assembly.omitted_sky.len(),
        prepared.flat_assembly.omitted_degenerate.len(),
        textures.len(),
        opaque_textures,
        deferred_alpha,
    )
}

/// Deterministic cross-assembly report before any renderer resource allocation.
pub fn prepared_e1m1_scene_report(
    flats: &PreparedE1m1Flats,
    walls: &PreparedE1m1Walls,
    flat_textures: &[PreparedStaticTexture],
    wall_textures: &[PreparedStaticTexture],
) -> String {
    let wall_opaque = opaque_texture_names(wall_textures).len();
    let floors = flats
        .flat_assembly
        .opaque_flats
        .iter()
        .filter(|flat| flat.source.plane == DoomSurfacePlane::Floor)
        .count();
    let ceilings = flats.flat_assembly.opaque_flats.len() - floors;
    let wall_middle = walls
        .wall_assembly
        .opaque_walls
        .iter()
        .filter(|wall| wall.role == DoomWallTextureRole::Middle)
        .count();
    let wall_upper = walls
        .wall_assembly
        .opaque_walls
        .iter()
        .filter(|wall| wall.role == DoomWallTextureRole::Upper)
        .count();
    let wall_lower = walls
        .wall_assembly
        .opaque_walls
        .iter()
        .filter(|wall| wall.role == DoomWallTextureRole::Lower)
        .count();
    let degenerate = static_degenerate_omission_impact(flats, walls);
    let fully_omitted_linedefs = fully_omitted_wall_linedef_indices(walls);
    format!(
        "{} submitted_floor_triangles={floors} submitted_ceiling_triangles={ceilings} opaque_wall_triangles={} submitted_wall_middle_triangles={wall_middle} submitted_wall_upper_triangles={wall_upper} submitted_wall_lower_triangles={wall_lower} omitted_masked_middles={} omitted_degenerate_walls={} wall_texture_names={} wall_opaque_textures={} wall_deferred_alpha_textures={} degenerate_flat_subsectors={} degenerate_flat_sectors={} fully_omitted_flat_subsectors={} degenerate_wall_linedefs={} degenerate_wall_sidedefs={} degenerate_wall_sectors={} fully_omitted_wall_linedefs={} fully_omitted_wall_linedef_indices={fully_omitted_linedefs:?}",
        prepared_e1m1_report(flats, flat_textures),
        walls.wall_assembly.opaque_walls.len(),
        walls.wall_assembly.omitted_masked_middles.len(),
        walls.wall_assembly.omitted_degenerate.len(),
        prepared_e1m1_wall_texture_names(walls).len(),
        wall_opaque,
        wall_textures.len() - wall_opaque,
        degenerate.flat_subsectors,
        degenerate.flat_sectors,
        degenerate.fully_omitted_flat_subsectors,
        degenerate.wall_linedefs,
        degenerate.wall_sidedefs,
        degenerate.wall_sectors,
        degenerate.fully_omitted_wall_linedefs,
    )
}

#[derive(Debug, Error)]
pub enum E1m1PreparationError {
    #[error("E1M1 selection failed: {0}")]
    MapSelection(#[from] doom_wad_package::DoomMapSelectionError),
    #[error("E1M1 map decode failed: {0}")]
    MapDecode(#[from] doom_map_provider::DoomMapDecodeError),
    #[error("E1M1 geometry preparation failed: {0}")]
    Geometry(#[from] doom_geometry_provider::DoomGeometryError),
    #[error("E1M1 static flat lowering failed: {0}")]
    Flat(#[from] StaticFlatLoweringError),
    #[error("E1M1 raster preparation failed: {0}")]
    Raster(#[from] doom_raster_provider::DoomRasterDecodeError),
    #[error("E1M1 static sky preparation failed: {0}")]
    SkyPanorama(#[from] DoomSkyPanoramaError),
}

/// Why a composed Doom sky cannot be used as the bounded static panorama
/// experiment. This is deliberately stricter than ordinary texture handling:
/// it may crop a wholly empty outer row band, but never fills or ignores an
/// internal/partial coverage gap.
#[derive(Clone, Debug, Error, Eq, PartialEq)]
pub enum DoomSkyPanoramaError {
    #[error("sky raster has no fully covered rows")]
    NoCoveredRows,
    #[error("sky raster row {row} has partial coverage ({covered}/{width})")]
    PartialCoverage {
        row: usize,
        covered: usize,
        width: usize,
    },
    #[error("sky raster has an uncovered row {row} inside its covered band")]
    InternalUncoveredRow { row: usize },
}

struct E1m1WallInputs {
    map_name: String,
    walls: Vec<DoomTexturedWallTriangle>,
    masked_middles: Vec<DoomMiddleTextureObservation>,
    extents: Vec<DoomTextureExtent>,
}

fn decode_e1m1_wall_inputs(
    wad_bytes: &[u8],
    manifest: &WadManifest,
    map_limits: doom_map_provider::DoomMapDecodeLimits,
    texture_limits: DoomTextureDecodeLimits,
) -> Result<E1m1WallInputs, E1m1PreparationError> {
    let selection = select_doom_episode_map(manifest, "E1M1")?;
    let map = doom_map_provider::decode_doom_map_core(wad_bytes, &selection, map_limits)?;
    let extents = prepare_e1m1_wall_texture_extents(wad_bytes, manifest, texture_limits)?;
    let walls = doom_geometry_provider::lower_doom_textured_wall_triangles(&map, &extents)?;
    let masked_middles = doom_geometry_provider::observe_doom_two_sided_middle_textures(&map)?;
    Ok(E1m1WallInputs {
        map_name: selection.map_name,
        walls,
        masked_middles,
        extents,
    })
}

/// Retains the full source texture-extent catalog used to resolve Doom wall
/// spans. This is geometry metadata only: it neither decodes RGBA8 pixels nor
/// admits every catalog entry as a renderer texture.
pub fn prepare_e1m1_wall_texture_extents(
    wad_bytes: &[u8],
    manifest: &WadManifest,
    texture_limits: DoomTextureDecodeLimits,
) -> Result<Vec<DoomTextureExtent>, E1m1PreparationError> {
    Ok(
        decode_doom_texture_catalog(wad_bytes, manifest, texture_limits)?
            .textures
            .iter()
            .map(|texture| DoomTextureExtent {
                name: texture.name.clone(),
                width: texture.width,
                height: texture.height,
            })
            .collect(),
    )
}

/// Decodes caller-selected wall texture names through the retained composition
/// provider, retaining alpha deferrals rather than choosing a fallback draw.
pub fn prepare_e1m1_wall_textures(
    wad_bytes: &[u8],
    manifest: &WadManifest,
    names: &[String],
    raster_limits: DoomRasterDecodeLimits,
    texture_limits: DoomTextureDecodeLimits,
    patch_limits: DoomPatchDecodeLimits,
    compose_limits: DoomTextureComposeLimits,
) -> Result<Vec<PreparedStaticTexture>, E1m1PreparationError> {
    let globals = decode_doom_raster_globals(wad_bytes, manifest, raster_limits)?;
    let catalog = decode_doom_texture_catalog(wad_bytes, manifest, texture_limits)?;
    let mut names = names.to_vec();
    names.sort();
    names.dedup();
    names
        .iter()
        .map(|name| {
            let indexed = compose_doom_texture(
                wad_bytes,
                manifest,
                &catalog,
                name,
                patch_limits,
                compose_limits,
            )?;
            let lowered = lower_doom_indexed_image(&indexed, &globals.palettes[0])?;
            Ok(PreparedStaticTexture {
                eligibility: classify_static_texture(&indexed),
                rgba8: lowered.pixels,
            })
        })
        .collect()
}

/// Prepares the Doom-only static panorama source used by the E1M1 sky corpus
/// experiment. Unlike an ordinary wall texture, classic `SKY1` may contain a
/// wholly uncovered outer band. We retain only its contiguous full-width
/// covered band, never manufacture texels, and reject partial or internal
/// holes as source evidence requiring a different policy.
pub fn prepare_e1m1_static_sky_panorama_texture(
    wad_bytes: &[u8],
    manifest: &WadManifest,
    raster_limits: DoomRasterDecodeLimits,
    texture_limits: DoomTextureDecodeLimits,
    patch_limits: DoomPatchDecodeLimits,
    compose_limits: DoomTextureComposeLimits,
) -> Result<PreparedStaticTexture, E1m1PreparationError> {
    let globals = decode_doom_raster_globals(wad_bytes, manifest, raster_limits)?;
    let catalog = decode_doom_texture_catalog(wad_bytes, manifest, texture_limits)?;
    let indexed = compose_doom_texture(
        wad_bytes,
        manifest,
        &catalog,
        "SKY1",
        patch_limits,
        compose_limits,
    )?;
    let cropped = crop_static_sky_coverage_band(&indexed)?;
    let lowered = lower_doom_indexed_image(&cropped, &globals.palettes[0])?;
    Ok(PreparedStaticTexture {
        eligibility: StaticTextureEligibility::Opaque(StaticOpaqueTexture {
            texture_name: cropped.texture_name.clone(),
            descriptor: Rgba8TextureDescriptor::new(
                u32::from(cropped.width),
                u32::from(cropped.height),
                Rgba8TextureColorSpace::Srgb,
            ),
            sampler: TextureSampler {
                filter: TextureFilter::Point,
                address_u: TextureAddressMode::Repeat,
                address_v: TextureAddressMode::Clamp,
            },
            selected_palette: 0,
        }),
        rgba8: lowered.pixels,
    })
}

fn crop_static_sky_coverage_band(
    image: &DoomIndexedImage,
) -> Result<DoomIndexedImage, DoomSkyPanoramaError> {
    let width = usize::from(image.width);
    let rows = image.coverage.chunks_exact(width).collect::<Vec<_>>();
    let mut full_rows = Vec::with_capacity(rows.len());
    for (row, coverage) in rows.iter().enumerate() {
        let covered = coverage.iter().filter(|covered| **covered).count();
        if covered != 0 && covered != width {
            return Err(DoomSkyPanoramaError::PartialCoverage {
                row,
                covered,
                width,
            });
        }
        full_rows.push(covered == width);
    }
    let Some(first) = full_rows.iter().position(|full| *full) else {
        return Err(DoomSkyPanoramaError::NoCoveredRows);
    };
    let last = full_rows
        .iter()
        .rposition(|full| *full)
        .expect("first full row exists");
    for (row, full) in full_rows[first..=last].iter().enumerate() {
        if !full {
            return Err(DoomSkyPanoramaError::InternalUncoveredRow { row: first + row });
        }
    }
    let height = last - first + 1;
    let start = first * width;
    let end = (last + 1) * width;
    Ok(DoomIndexedImage {
        source_texture_lump_index: image.source_texture_lump_index,
        texture_name: image.texture_name.clone(),
        width: image.width,
        height: height as u16,
        color_indices: image.color_indices[start..end].to_vec(),
        coverage: vec![true; width * height],
        opaque_pixels: width * height,
    })
}

/// Decodes exactly the non-sky flat names selected by a prepared scene using
/// palette zero. The result is ready for a later caller-owned texture upload.
pub fn prepare_e1m1_flat_textures(
    wad_bytes: &[u8],
    manifest: &WadManifest,
    prepared: &PreparedE1m1Flats,
    raster_limits: DoomRasterDecodeLimits,
    flat_limits: DoomFlatDecodeLimits,
) -> Result<Vec<PreparedStaticTexture>, E1m1PreparationError> {
    let globals = decode_doom_raster_globals(wad_bytes, manifest, raster_limits)?;
    let mut names = prepared
        .flat_assembly
        .opaque_flats
        .iter()
        .map(|flat| flat.source.flat_name.as_str())
        .collect::<Vec<_>>();
    names.sort_unstable();
    names.dedup();
    names
        .into_iter()
        .map(|name| {
            let flat = decode_doom_flat(wad_bytes, manifest, name, flat_limits)?;
            let indexed = indexed_image_from_doom_flat(&flat);
            let lowered = lower_doom_indexed_image(&indexed, &globals.palettes[0])?;
            Ok(PreparedStaticTexture {
                eligibility: classify_static_texture(&indexed),
                rgba8: lowered.pixels,
            })
        })
        .collect()
}

/// Builds the first static-flat scene input from a caller-owned, already
/// inspected WAD. This does not acquire files, decode textures, allocate GPU
/// resources, or submit a frame.
pub fn prepare_e1m1_flats(
    wad_bytes: &[u8],
    manifest: &WadManifest,
    limits: doom_map_provider::DoomMapDecodeLimits,
) -> Result<PreparedE1m1Flats, E1m1PreparationError> {
    let selection = select_doom_episode_map(manifest, "E1M1")?;
    let map = doom_map_provider::decode_doom_map_core(wad_bytes, &selection, limits)?;
    let paths = doom_geometry_provider::resolve_doom_subsector_bsp_paths(&map)?;
    let surfaces = doom_geometry_provider::lower_doom_subsector_surfaces(&map, &paths)?;
    let sky = doom_geometry_provider::observe_doom_sky_surfaces(&map, &paths)?;
    Ok(PreparedE1m1Flats {
        map_name: selection.map_name,
        flat_assembly: assemble_static_opaque_flats(&surfaces, &sky, FlatExtent::E1M1)?,
    })
}

/// Re-lowers only the explicitly retained sky omissions for AR-0027's
/// opt-in diagnostic presentation experiment.  The returned meshes retain
/// their original flat identity; callers must still declare why they chose a
/// stand-in and supply its material.  Normal E1M1 preparation never calls
/// this function.
pub fn prepare_e1m1_sky_diagnostic_flats(
    wad_bytes: &[u8],
    manifest: &WadManifest,
    limits: doom_map_provider::DoomMapDecodeLimits,
) -> Result<Vec<StaticFlatMesh>, E1m1PreparationError> {
    let selection = select_doom_episode_map(manifest, "E1M1")?;
    let map = doom_map_provider::decode_doom_map_core(wad_bytes, &selection, limits)?;
    let paths = doom_geometry_provider::resolve_doom_subsector_bsp_paths(&map)?;
    let surfaces = doom_geometry_provider::lower_doom_subsector_surfaces(&map, &paths)?;
    let sky = doom_geometry_provider::observe_doom_sky_surfaces(&map, &paths)?;
    let mut lowered = Vec::new();
    for surface in &surfaces {
        let is_retained_sky = sky.iter().any(|observation| {
            observation.source_subsector == surface.source_subsector
                && observation.source_sector == surface.source_sector
                && observation.plane == surface.plane
        });
        if !is_retained_sky {
            continue;
        }
        match lower_static_flat_triangle(surface, FlatExtent::E1M1) {
            Ok(mesh) => lowered.push(mesh),
            // This diagnostic path does not invent a stand-in for geometry
            // with no stable face either.
            Err(StaticFlatLoweringError::DegenerateTriangle) => {}
            Err(error) => return Err(error.into()),
        }
    }
    Ok(lowered)
}

/// Builds source-textured wall candidates and source-classified masked-middle
/// omissions from a caller-owned, already-inspected E1M1 WAD.
pub fn prepare_e1m1_walls(
    wad_bytes: &[u8],
    manifest: &WadManifest,
    map_limits: doom_map_provider::DoomMapDecodeLimits,
    texture_limits: DoomTextureDecodeLimits,
) -> Result<PreparedE1m1Walls, E1m1PreparationError> {
    let inputs = decode_e1m1_wall_inputs(wad_bytes, manifest, map_limits, texture_limits)?;
    Ok(PreparedE1m1Walls {
        map_name: inputs.map_name,
        wall_assembly: assemble_static_opaque_walls(
            &inputs.walls,
            &inputs.masked_middles,
            &inputs.extents,
        )?,
    })
}

/// Replays E1M1's retained source classification into only the corpus-local
/// cutout candidate. It is intentionally distinct from `prepare_e1m1_walls`,
/// which prepares the opaque static scene.
pub fn prepare_e1m1_masked_middle_cutouts(
    wad_bytes: &[u8],
    manifest: &WadManifest,
    map_limits: doom_map_provider::DoomMapDecodeLimits,
    texture_limits: DoomTextureDecodeLimits,
) -> Result<PreparedE1m1MaskedMiddleCutouts, E1m1PreparationError> {
    let inputs = decode_e1m1_wall_inputs(wad_bytes, manifest, map_limits, texture_limits)?;
    Ok(PreparedE1m1MaskedMiddleCutouts {
        map_name: inputs.map_name,
        assembly: assemble_experimental_masked_middle_cutouts(
            &inputs.walls,
            &inputs.masked_middles,
            &inputs.extents,
        )?,
    })
}

/// Lowers a Doom floor or ceiling candidate into an ordinary supplied-UV mesh.
///
/// This is intentionally a declared modern static projection rather than a
/// claim to reproduce Doom's original view-dependent plane-span renderer.
pub fn lower_static_flat_triangle(
    triangle: &DoomSurfaceTriangle,
    extent: FlatExtent,
) -> Result<StaticFlatMesh, StaticFlatLoweringError> {
    if extent.width == 0 || extent.height == 0 {
        return Err(StaticFlatLoweringError::ZeroExtent {
            width: extent.width,
            height: extent.height,
        });
    }

    let positions = [
        lower_position(triangle.positions[0], 0)?,
        lower_position(triangle.positions[1], 1)?,
        lower_position(triangle.positions[2], 2)?,
    ];
    let normal = triangle_normal(positions)?;
    let coordinates = positions
        .iter()
        .map(|[x, _, z]| [*x / f32::from(extent.width), -*z / f32::from(extent.height)])
        .collect();
    let mesh =
        Mesh::uniform_normal(positions.to_vec(), normal).with_texture_coordinates(coordinates)?;

    Ok(StaticFlatMesh {
        source: StaticFlatSource {
            subsector: triangle.source_subsector,
            sector: triangle.source_sector,
            plane: triangle.plane,
            flat_name: triangle.texture_name.clone(),
        },
        mesh,
    })
}

/// Applies the selected first-scene sky omission before lowering the remaining
/// floor and ceiling candidates. This function deliberately has no material,
/// texture upload, pipeline, or renderer dependency.
pub fn assemble_static_opaque_flats(
    surfaces: &[DoomSurfaceTriangle],
    sky: &[DoomSkySurfaceObservation],
    extent: FlatExtent,
) -> Result<StaticFlatAssembly, StaticFlatLoweringError> {
    let mut opaque_flats = Vec::with_capacity(surfaces.len());
    let mut omitted_degenerate = Vec::new();
    for surface in surfaces {
        if sky.iter().any(|observation| {
            observation.source_subsector == surface.source_subsector
                && observation.source_sector == surface.source_sector
                && observation.plane == surface.plane
        }) {
            continue;
        }
        match lower_static_flat_triangle(surface, extent) {
            Ok(flat) => opaque_flats.push(flat),
            Err(StaticFlatLoweringError::DegenerateTriangle) => {
                omitted_degenerate.push(StaticFlatSource {
                    subsector: surface.source_subsector,
                    sector: surface.source_sector,
                    plane: surface.plane,
                    flat_name: surface.texture_name.clone(),
                });
            }
            Err(error) => return Err(error),
        }
    }
    Ok(StaticFlatAssembly {
        opaque_flats,
        omitted_sky: sky.to_vec(),
        omitted_degenerate,
    })
}

/// Lowers one source-textured wall triangle without repeating the Doom pegging
/// calculation. Coordinates are normalized from the retained source texels.
pub fn lower_static_wall_triangle(
    triangle: &DoomTexturedWallTriangle,
    extent: DoomTextureExtent,
) -> Result<StaticWallMesh, StaticFlatLoweringError> {
    if extent.width == 0 || extent.height == 0 {
        return Err(StaticFlatLoweringError::ZeroExtent {
            width: extent.width,
            height: extent.height,
        });
    }
    let positions = [
        lower_position(triangle.positions[0], 0)?,
        lower_position(triangle.positions[1], 1)?,
        lower_position(triangle.positions[2], 2)?,
    ];
    let normal = triangle_normal(positions)?;
    let coordinates = triangle.texture_coordinates.map(|[u, v]| {
        [
            u as f32 / f32::from(extent.width),
            v as f32 / f32::from(extent.height),
        ]
    });
    let mesh = Mesh::uniform_normal(positions.to_vec(), normal)
        .with_texture_coordinates(coordinates.to_vec())?;
    Ok(StaticWallMesh {
        source_linedef: triangle.source_linedef,
        source_sidedef: triangle.source_sidedef,
        source_sector: triangle.source_sector,
        side: triangle.side,
        role: triangle.role,
        texture_name: triangle.texture_name.clone(),
        mesh,
    })
}

/// Applies the established ordinary supplied-UV wall lowering to a bounded
/// SEG-derived source triangle. The source screen-span experiment supplies the
/// bounded triangle; this function deliberately contributes no visibility
/// rule, material policy, or renderer vocabulary.
pub fn lower_static_seg_wall_triangle(
    triangle: &DoomSegTexturedWallTriangle,
    extent: DoomTextureExtent,
) -> Result<StaticSegWallMesh, StaticFlatLoweringError> {
    let wall = lower_static_wall_triangle(
        &DoomTexturedWallTriangle {
            source_linedef: triangle.source_linedef,
            source_sidedef: triangle.source_sidedef,
            source_sector: triangle.source_sector,
            side: triangle.side,
            role: triangle.role,
            texture_name: triangle.texture_name.clone(),
            positions: triangle.positions,
            texture_coordinates: triangle.texture_coordinates,
        },
        extent,
    )?;
    Ok(StaticSegWallMesh {
        source_seg: triangle.source_seg,
        wall,
    })
}

/// Excludes only explicitly observed two-sided masked middles before lowering
/// the remaining source-textured wall candidates.
pub fn assemble_static_opaque_walls(
    walls: &[DoomTexturedWallTriangle],
    masked_middles: &[DoomMiddleTextureObservation],
    extents: &[DoomTextureExtent],
) -> Result<StaticWallAssembly, StaticFlatLoweringError> {
    let mut opaque_walls = Vec::with_capacity(walls.len());
    let mut omitted_degenerate = Vec::new();
    for wall in walls {
        let masked_middle = wall.role == DoomWallTextureRole::Middle
            && masked_middles.iter().any(|middle| {
                middle.source_linedef == wall.source_linedef
                    && middle.source_sidedef == wall.source_sidedef
                    && middle.side == wall.side
            });
        if masked_middle {
            continue;
        }
        let extent = extents
            .iter()
            .find(|extent| extent.name.eq_ignore_ascii_case(&wall.texture_name))
            .cloned()
            .ok_or_else(|| StaticFlatLoweringError::MissingWallTextureExtent {
                name: wall.texture_name.clone(),
            })?;
        match lower_static_wall_triangle(wall, extent) {
            Ok(lowered) => opaque_walls.push(lowered),
            Err(StaticFlatLoweringError::DegenerateTriangle) => {
                omitted_degenerate.push(wall.clone())
            }
            Err(error) => return Err(error),
        }
    }
    Ok(StaticWallAssembly {
        opaque_walls,
        omitted_masked_middles: masked_middles.to_vec(),
        omitted_degenerate,
    })
}

/// Lowers only explicitly source-classified two-sided masked middles into the
/// corpus-local AR-0023 cutout candidate. This remains separate from the
/// static opaque scene: it uploads nothing and establishes no renderer-facing
/// alpha contract.
pub fn assemble_experimental_masked_middle_cutouts(
    walls: &[DoomTexturedWallTriangle],
    masked_middles: &[DoomMiddleTextureObservation],
    extents: &[DoomTextureExtent],
) -> Result<ExperimentalCutoutWallAssembly, StaticFlatLoweringError> {
    let mut candidates = Vec::new();
    let mut omitted_degenerate = Vec::new();
    for wall in walls {
        let Some(source) = masked_middles.iter().find(|middle| {
            wall.role == DoomWallTextureRole::Middle
                && middle.source_linedef == wall.source_linedef
                && middle.source_sidedef == wall.source_sidedef
                && middle.side == wall.side
                && middle.texture_name == wall.texture_name
        }) else {
            continue;
        };
        let extent = extents
            .iter()
            .find(|extent| extent.name.eq_ignore_ascii_case(&wall.texture_name))
            .cloned()
            .ok_or_else(|| StaticFlatLoweringError::MissingWallTextureExtent {
                name: wall.texture_name.clone(),
            })?;
        match lower_static_wall_triangle(wall, extent) {
            Ok(lowered) => candidates.push(ExperimentalCutoutWall {
                source: source.clone(),
                wall: lowered,
                intent: ExperimentalCutoutIntent {
                    discard_at_or_below_alpha: 0,
                    depth_write: true,
                },
            }),
            Err(StaticFlatLoweringError::DegenerateTriangle) => {
                omitted_degenerate.push(wall.clone())
            }
            Err(error) => return Err(error),
        }
    }
    Ok(ExperimentalCutoutWallAssembly {
        candidates,
        omitted_degenerate,
    })
}

/// Chooses the bounded palette-zero/sRGB/point-repeat profile only when every
/// source pixel is covered. Any hole remains an explicit AR-0023 deferral.
pub fn classify_static_texture(image: &DoomIndexedImage) -> StaticTextureEligibility {
    let uncovered_pixels = image.coverage.iter().filter(|covered| !**covered).count();
    if uncovered_pixels > 0 {
        return StaticTextureEligibility::DeferredAlpha {
            texture_name: image.texture_name.clone(),
            uncovered_pixels,
            descriptor: Rgba8TextureDescriptor::new(
                u32::from(image.width),
                u32::from(image.height),
                Rgba8TextureColorSpace::Srgb,
            ),
            sampler: TextureSampler {
                filter: TextureFilter::Point,
                address_u: TextureAddressMode::Repeat,
                address_v: TextureAddressMode::Repeat,
            },
            selected_palette: 0,
        };
    }
    StaticTextureEligibility::Opaque(StaticOpaqueTexture {
        texture_name: image.texture_name.clone(),
        descriptor: Rgba8TextureDescriptor::new(
            u32::from(image.width),
            u32::from(image.height),
            Rgba8TextureColorSpace::Srgb,
        ),
        sampler: TextureSampler {
            filter: TextureFilter::Point,
            address_u: TextureAddressMode::Repeat,
            address_v: TextureAddressMode::Repeat,
        },
        selected_palette: 0,
    })
}

fn lower_position(position: [f64; 3], vertex: usize) -> Result<[f32; 3], StaticFlatLoweringError> {
    Ok([
        finite_f32(position[0], vertex, "x")?,
        finite_f32(position[1], vertex, "height")?,
        finite_f32(position[2], vertex, "z")?,
    ])
}

fn finite_f32(
    value: f64,
    vertex: usize,
    component: &'static str,
) -> Result<f32, StaticFlatLoweringError> {
    if !value.is_finite() || value < f64::from(f32::MIN) || value > f64::from(f32::MAX) {
        return Err(StaticFlatLoweringError::NonFinitePosition { vertex, component });
    }
    Ok(value as f32)
}

fn triangle_normal(positions: [[f32; 3]; 3]) -> Result<[f32; 3], StaticFlatLoweringError> {
    let ab = subtract(positions[1], positions[0]);
    let ac = subtract(positions[2], positions[0]);
    let cross = cross(ab, ac);
    let length_squared = dot(cross, cross);
    if !length_squared.is_finite() || length_squared <= f32::EPSILON {
        return Err(StaticFlatLoweringError::DegenerateTriangle);
    }
    let reciprocal_length = length_squared.sqrt().recip();
    Ok(cross.map(|component| component * reciprocal_length))
}

fn subtract(left: [f32; 3], right: [f32; 3]) -> [f32; 3] {
    [left[0] - right[0], left[1] - right[1], left[2] - right[2]]
}

fn cross(left: [f32; 3], right: [f32; 3]) -> [f32; 3] {
    [
        left[1] * right[2] - left[2] * right[1],
        left[2] * right[0] - left[0] * right[2],
        left[0] * right[1] - left[1] * right[0],
    ]
}

fn dot(left: [f32; 3], right: [f32; 3]) -> f32 {
    left[0] * right[0] + left[1] * right[1] + left[2] * right[2]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn current_doom_ground_lift_reverses_orientation_about_world_up() {
        let observation = observe_doom_ground_frame([1.0, 0.0], [0.0, 1.0]);

        assert_eq!(observation.source_signed_orientation, 1.0);
        assert_eq!(observation.lifted_right, Vec3::X);
        assert_eq!(observation.lifted_forward, Vec3::Z);
        assert_eq!(observation.lifted_orientation_about_world_up, -1.0);
    }

    #[test]
    fn current_lifted_source_right_opposes_observer_camera_right() {
        let observation = observe_doom_ground_frame([1.0, 0.0], [0.0, 1.0]);

        assert_eq!(observation.camera_right, Vec3::NEG_X);
        assert_eq!(observation.source_right_camera_right_alignment, -1.0);
    }

    #[test]
    fn both_orientation_preserving_candidates_align_source_and_camera_right() {
        for embedding in [
            DoomComparativeEmbedding::PreserveEast,
            DoomComparativeEmbedding::PreserveNorth,
        ] {
            let observation =
                observe_doom_ground_frame_with_embedding(embedding, [1.0, 0.0], [0.0, 1.0]);

            assert_eq!(observation.source_signed_orientation, 1.0);
            assert_eq!(observation.lifted_orientation_about_world_up, 1.0);
            assert_eq!(observation.source_right_camera_right_alignment, 1.0);
        }
    }

    #[test]
    fn comparative_embeddings_retain_distinct_east_and_north_choices() {
        let east = [1.0, 0.0];
        let north = [0.0, 1.0];

        assert_eq!(
            DoomComparativeEmbedding::PreserveEast.lift_direction(east, 0.0),
            Vec3::X
        );
        assert_eq!(
            DoomComparativeEmbedding::PreserveEast.lift_direction(north, 0.0),
            Vec3::NEG_Z
        );
        assert_eq!(
            DoomComparativeEmbedding::PreserveNorth.lift_direction(east, 0.0),
            Vec3::NEG_X
        );
        assert_eq!(
            DoomComparativeEmbedding::PreserveNorth.lift_direction(north, 0.0),
            Vec3::Z
        );
    }

    #[test]
    fn every_comparative_embedding_round_trips_without_proving_orientation() {
        let source = [23.5, -91.25];
        for embedding in DoomComparativeEmbedding::ALL {
            let world = embedding.lift_direction(source, 7.0);
            let (round_trip, vertical) = embedding.lower_direction(world);

            assert_eq!(round_trip, source);
            assert_eq!(vertical, 7.0);
        }
    }

    #[test]
    fn candidate_command_replay_preserves_source_forward_strafe_and_screen_right_yaw() {
        for embedding in [
            DoomComparativeEmbedding::PreserveEast,
            DoomComparativeEmbedding::PreserveNorth,
        ] {
            for source_heading in [0.0_f32, 45.0, 90.0, 180.0, 270.0] {
                let radians = source_heading.to_radians();
                let source_forward = [radians.cos(), radians.sin()];
                let source_right = [source_forward[1], -source_forward[0]];
                let lifted_forward = embedding.lift_heading_degrees(source_heading);
                let lifted_right = embedding
                    .lift_direction(source_right, 0.0)
                    .normalize_or_zero();
                let yaw = observer_yaw_from_forward(lifted_forward);

                // W and D remain source-forward and source-right after the
                // embedding, rather than acquiring compensating input signs.
                assert!(observer_direction(yaw, 0.0).dot(lifted_forward) > 0.999_9);
                assert!(observer_right(lifted_forward).dot(lifted_right) > 0.999_9);

                // The existing pointer-look policy subtracts yaw for a
                // screen-right turn. It must land on the same transformed
                // source-right direction for either candidate.
                let screen_right_yaw = yaw - std::f32::consts::FRAC_PI_2;
                assert!(
                    observer_direction(screen_right_yaw, 0.0).dot(lifted_right) > 0.999_9,
                    "{embedding:?} heading {source_heading} broke screen-right replay"
                );

                let (round_trip, vertical) = embedding.lower_direction(lifted_forward);
                assert!((round_trip[0] - source_forward[0]).abs() < 0.000_1);
                assert!((round_trip[1] - source_forward[1]).abs() < 0.000_1);
                assert_eq!(vertical, 0.0);
            }
        }
    }

    #[test]
    fn candidate_embeddings_require_wall_winding_to_be_rebuilt() {
        let source_start = [10.0_f32, 20.0_f32];
        let source_end = [30.0_f32, 40.0_f32];
        let source_delta = [
            source_end[0] - source_start[0],
            source_end[1] - source_start[1],
        ];
        let source_right_normal = [source_delta[1], -source_delta[0]];

        for embedding in [
            DoomComparativeEmbedding::PreserveEast,
            DoomComparativeEmbedding::PreserveNorth,
        ] {
            let start_bottom = embedding.lift_direction(source_start, 0.0);
            let end_bottom = embedding.lift_direction(source_end, 0.0);
            let end_top = embedding.lift_direction(source_end, 64.0);
            let expected_right = embedding.lift_direction(source_right_normal, 0.0);

            // The current right/front order is start-bottom, end-top,
            // end-bottom. A reflected embedding makes that order face away
            // from the transformed owning-side normal.
            let current_order_normal = (end_top - start_bottom).cross(end_bottom - start_bottom);
            assert!(current_order_normal.dot(expected_right) < 0.0);

            // Rebuilding the candidate winding, rather than patching culling,
            // restores the source-owned right/front relationship.
            let rebuilt_right_normal = (end_bottom - start_bottom).cross(end_top - start_bottom);
            assert!(rebuilt_right_normal.dot(expected_right) > 0.0);
            assert!(rebuilt_right_normal.dot(-expected_right) < 0.0);
        }
    }

    #[test]
    fn candidate_embedding_rebuilds_untextured_depth_mesh_without_uv_access() {
        let mut mesh = Mesh::uniform_normal(
            vec![[10.0, 0.0, 20.0], [30.0, 64.0, 40.0], [30.0, 0.0, 40.0]],
            [0.0, 0.0, 1.0],
        );

        reembed_comparative_mesh(&mut mesh, DoomComparativeEmbedding::PreserveNorth, false);

        assert!(mesh.texture_coordinates.is_empty());
        assert_eq!(mesh.positions.len(), 3);
        assert_eq!(mesh.normals.len(), 3);
        assert!(mesh
            .positions
            .iter()
            .flatten()
            .all(|component| component.is_finite()));
        assert!(mesh
            .normals
            .iter()
            .flatten()
            .all(|component| component.is_finite()));
    }

    #[test]
    fn canonical_e1m1_spawn_doorway_and_hut_landmarks_reverse_about_world_up() {
        // Reviewed DOOM1.WAD E1M1 identities:
        // - THINGS #0: player start (1056, -3616)
        // - LINEDEFS #0 midpoint: start doorway (1056, -3680)
        // - LINEDEFS #208 midpoint: interactively identified BROWN1 hut wall
        //   (2176, -3824)
        let spawn = [1056.0_f32, -3616.0_f32];
        let doorway = [1056.0_f32, -3680.0_f32];
        let hut = [2176.0_f32, -3824.0_f32];
        let doorway_relative = [doorway[0] - spawn[0], doorway[1] - spawn[1]];
        let hut_relative = [hut[0] - spawn[0], hut[1] - spawn[1]];
        let source_orientation =
            doorway_relative[0] * hut_relative[1] - doorway_relative[1] * hut_relative[0];
        let lifted_doorway =
            DoomComparativeEmbedding::CurrentReflected.lift_direction(doorway_relative, 0.0);
        let lifted_hut =
            DoomComparativeEmbedding::CurrentReflected.lift_direction(hut_relative, 0.0);
        let world_orientation = lifted_doorway.cross(lifted_hut).dot(Vec3::Y);
        let source_heading = [0.0_f32, 1.0_f32];
        let source_right = [source_heading[1], -source_heading[0]];
        let source_hut_side = hut_relative[0] * source_right[0] + hut_relative[1] * source_right[1];
        let camera_right = observer_right(Vec3::Z);
        let presented_hut_side = lifted_hut.dot(camera_right);

        assert_eq!(source_orientation, 71_680.0);
        assert_eq!(world_orientation, -71_680.0);
        assert_eq!(source_hut_side, 1_120.0);
        assert_eq!(presented_hut_side, -1_120.0);

        for embedding in [
            DoomComparativeEmbedding::PreserveEast,
            DoomComparativeEmbedding::PreserveNorth,
        ] {
            let candidate_hut = embedding.lift_direction(hut_relative, 0.0);
            let candidate_forward = embedding.lift_direction(source_heading, 0.0);
            let candidate_camera_right = observer_right(candidate_forward);
            assert_eq!(candidate_hut.dot(candidate_camera_right), 1_120.0);
        }
    }

    fn candidate() -> DoomSurfaceTriangle {
        DoomSurfaceTriangle {
            source_subsector: DoomSourceRecord {
                lump_index: 6,
                record_index: 12,
            },
            source_sector: DoomSourceRecord {
                lump_index: 8,
                record_index: 3,
            },
            plane: DoomSurfacePlane::Floor,
            texture_name: "FLOOR0_1".into(),
            positions: [[64.0, 0.0, 128.0], [128.0, 0.0, 128.0], [64.0, 0.0, 192.0]],
        }
    }

    fn wall_candidate() -> DoomTexturedWallTriangle {
        DoomTexturedWallTriangle {
            source_linedef: DoomSourceRecord {
                lump_index: 2,
                record_index: 7,
            },
            source_sidedef: DoomSourceRecord {
                lump_index: 3,
                record_index: 9,
            },
            source_sector: DoomSourceRecord {
                lump_index: 8,
                record_index: 3,
            },
            side: doom_geometry_provider::DoomWallSideKind::Right,
            role: DoomWallTextureRole::Middle,
            texture_name: "STARTAN3".into(),
            positions: [[0.0, 64.0, 0.0], [64.0, 64.0, 0.0], [0.0, 0.0, 0.0]],
            texture_coordinates: [[32.0, 16.0], [96.0, 16.0], [32.0, 80.0]],
        }
    }

    fn raster(coverage: Vec<bool>) -> DoomIndexedImage {
        raster_with_dimensions(2, 2, coverage)
    }

    fn raster_with_dimensions(width: u16, height: u16, coverage: Vec<bool>) -> DoomIndexedImage {
        DoomIndexedImage {
            source_texture_lump_index: 17,
            texture_name: "STARTAN3".into(),
            width,
            height,
            color_indices: (0..coverage.len()).map(|index| index as u8).collect(),
            opaque_pixels: coverage.iter().filter(|covered| **covered).count(),
            coverage,
        }
    }

    fn opaque_prepared(name: &str) -> PreparedStaticTexture {
        PreparedStaticTexture {
            eligibility: StaticTextureEligibility::Opaque(StaticOpaqueTexture {
                texture_name: name.into(),
                descriptor: Rgba8TextureDescriptor::new(2, 2, Rgba8TextureColorSpace::Srgb),
                sampler: TextureSampler {
                    filter: TextureFilter::Point,
                    address_u: TextureAddressMode::Repeat,
                    address_v: TextureAddressMode::Repeat,
                },
                selected_palette: 0,
            }),
            rgba8: vec![255; 16],
        }
    }

    #[test]
    fn lowers_map_axes_to_the_documented_supplied_uv_policy() {
        let lowered = lower_static_flat_triangle(&candidate(), FlatExtent::E1M1).unwrap();
        assert_eq!(
            lowered.mesh.texture_coordinates,
            vec![[1.0, -2.0], [2.0, -2.0], [1.0, -3.0]]
        );
        assert!(lowered.mesh.has_texture_coordinates());
        assert_eq!(lowered.source.flat_name, "FLOOR0_1");
        assert_eq!(lowered.source.subsector.record_index, 12);
    }

    #[test]
    fn rejects_degenerate_source_triangles() {
        let mut triangle = candidate();
        triangle.positions[2] = triangle.positions[1];
        assert_eq!(
            lower_static_flat_triangle(&triangle, FlatExtent::E1M1),
            Err(StaticFlatLoweringError::DegenerateTriangle)
        );
    }

    #[test]
    fn assembly_retains_collinear_candidate_as_an_omission_and_continues() {
        let valid = candidate();
        let mut collinear = candidate();
        collinear.positions = [[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [2.0, 0.0, 0.0]];
        let assembly =
            assemble_static_opaque_flats(&[valid, collinear], &[], FlatExtent::E1M1).unwrap();
        assert_eq!(assembly.opaque_flats.len(), 1);
        assert_eq!(assembly.omitted_degenerate.len(), 1);
        assert_eq!(assembly.omitted_degenerate[0].flat_name, "FLOOR0_1");
    }

    #[test]
    fn rejects_zero_extent_before_dividing_coordinates() {
        assert_eq!(
            lower_static_flat_triangle(
                &candidate(),
                FlatExtent {
                    width: 0,
                    height: 64
                }
            ),
            Err(StaticFlatLoweringError::ZeroExtent {
                width: 0,
                height: 64
            })
        );
    }

    #[test]
    fn assembly_uses_retained_sky_classification_not_a_texture_name_heuristic() {
        let floor = candidate();
        let mut ceiling = candidate();
        ceiling.plane = DoomSurfacePlane::Ceiling;
        ceiling.texture_name = "CEIL1_1".into();
        let sky = DoomSkySurfaceObservation {
            source_subsector: ceiling.source_subsector,
            source_sector: ceiling.source_sector,
            plane: DoomSurfacePlane::Ceiling,
            texture_name: "F_SKY1".into(),
        };

        let assembly = assemble_static_opaque_flats(
            &[floor, ceiling],
            std::slice::from_ref(&sky),
            FlatExtent::E1M1,
        )
        .unwrap();

        assert_eq!(assembly.opaque_flats.len(), 1);
        assert_eq!(
            assembly.opaque_flats[0].source.plane,
            DoomSurfacePlane::Floor
        );
        assert_eq!(assembly.omitted_sky, vec![sky]);
    }

    #[test]
    fn wall_lowering_normalizes_retained_source_texels_without_repegging() {
        let lowered = lower_static_wall_triangle(
            &wall_candidate(),
            DoomTextureExtent {
                name: "STARTAN3".into(),
                width: 128,
                height: 128,
            },
        )
        .unwrap();
        assert_eq!(
            lowered.mesh.texture_coordinates,
            vec![[0.25, 0.125], [0.75, 0.125], [0.25, 0.625]]
        );
        assert_eq!(lowered.source_linedef.record_index, 7);
    }

    #[test]
    fn wall_assembly_excludes_only_explicitly_classified_masked_middles() {
        let wall = wall_candidate();
        let masked = DoomMiddleTextureObservation {
            source_linedef: wall.source_linedef,
            source_sidedef: wall.source_sidedef,
            source_sector: wall.source_sector,
            side: wall.side,
            texture_name: wall.texture_name.clone(),
            opening_floor: 0,
            opening_ceiling: 64,
        };
        let assembly = assemble_static_opaque_walls(
            &[wall],
            std::slice::from_ref(&masked),
            &[DoomTextureExtent {
                name: "STARTAN3".into(),
                width: 128,
                height: 128,
            }],
        )
        .unwrap();
        assert!(assembly.opaque_walls.is_empty());
        assert_eq!(assembly.omitted_masked_middles, vec![masked]);
    }

    #[test]
    fn masked_middle_lowering_declares_binary_cutout_without_changing_opaque_assembly() {
        let wall = wall_candidate();
        let masked = DoomMiddleTextureObservation {
            source_linedef: wall.source_linedef,
            source_sidedef: wall.source_sidedef,
            source_sector: wall.source_sector,
            side: wall.side,
            texture_name: wall.texture_name.clone(),
            opening_floor: 0,
            opening_ceiling: 64,
        };
        let mut degenerate = wall.clone();
        degenerate.positions[2] = degenerate.positions[1];
        let assembly = assemble_experimental_masked_middle_cutouts(
            &[wall, degenerate],
            std::slice::from_ref(&masked),
            &[DoomTextureExtent {
                name: "STARTAN3".into(),
                width: 128,
                height: 128,
            }],
        )
        .unwrap();

        assert_eq!(assembly.candidates.len(), 1);
        assert_eq!(assembly.candidates[0].source, masked);
        assert_eq!(
            assembly.candidates[0].intent,
            ExperimentalCutoutIntent {
                discard_at_or_below_alpha: 0,
                depth_write: true,
            }
        );
        assert_eq!(assembly.omitted_degenerate.len(), 1);
    }

    #[test]
    fn masked_middle_name_inventory_is_sorted_and_separate_from_opaque_walls() {
        let masked = vec![
            DoomMiddleTextureObservation {
                source_linedef: DoomSourceRecord {
                    lump_index: 1,
                    record_index: 2,
                },
                source_sidedef: DoomSourceRecord {
                    lump_index: 3,
                    record_index: 4,
                },
                source_sector: DoomSourceRecord {
                    lump_index: 5,
                    record_index: 6,
                },
                side: doom_geometry_provider::DoomWallSideKind::Right,
                texture_name: "Z_MASK".into(),
                opening_floor: 0,
                opening_ceiling: 8,
            },
            DoomMiddleTextureObservation {
                source_linedef: DoomSourceRecord {
                    lump_index: 1,
                    record_index: 5,
                },
                source_sidedef: DoomSourceRecord {
                    lump_index: 3,
                    record_index: 6,
                },
                source_sector: DoomSourceRecord {
                    lump_index: 5,
                    record_index: 7,
                },
                side: doom_geometry_provider::DoomWallSideKind::Left,
                texture_name: "A_MASK".into(),
                opening_floor: 0,
                opening_ceiling: 8,
            },
            DoomMiddleTextureObservation {
                source_linedef: DoomSourceRecord {
                    lump_index: 1,
                    record_index: 7,
                },
                source_sidedef: DoomSourceRecord {
                    lump_index: 3,
                    record_index: 8,
                },
                source_sector: DoomSourceRecord {
                    lump_index: 5,
                    record_index: 8,
                },
                side: doom_geometry_provider::DoomWallSideKind::Right,
                texture_name: "A_MASK".into(),
                opening_floor: 0,
                opening_ceiling: 8,
            },
        ];
        let prepared = PreparedE1m1Walls {
            map_name: "E1M1".into(),
            wall_assembly: StaticWallAssembly {
                opaque_walls: Vec::new(),
                omitted_masked_middles: masked,
                omitted_degenerate: Vec::new(),
            },
        };
        assert_eq!(
            prepared_e1m1_masked_middle_texture_names(&prepared),
            ["A_MASK", "Z_MASK"]
        );
    }

    #[test]
    fn opaque_raster_selects_the_declared_static_material_profile() {
        assert_eq!(
            classify_static_texture(&raster(vec![true; 4])),
            StaticTextureEligibility::Opaque(StaticOpaqueTexture {
                texture_name: "STARTAN3".into(),
                descriptor: Rgba8TextureDescriptor::new(2, 2, Rgba8TextureColorSpace::Srgb),
                sampler: TextureSampler {
                    filter: TextureFilter::Point,
                    address_u: TextureAddressMode::Repeat,
                    address_v: TextureAddressMode::Repeat,
                },
                selected_palette: 0,
            })
        );
    }

    #[test]
    fn uncovered_raster_is_deferred_instead_of_selecting_alpha_behavior() {
        assert_eq!(
            classify_static_texture(&raster(vec![true, false, true, false])),
            StaticTextureEligibility::DeferredAlpha {
                texture_name: "STARTAN3".into(),
                uncovered_pixels: 2,
                descriptor: Rgba8TextureDescriptor::new(2, 2, Rgba8TextureColorSpace::Srgb),
                sampler: TextureSampler {
                    filter: TextureFilter::Point,
                    address_u: TextureAddressMode::Repeat,
                    address_v: TextureAddressMode::Repeat,
                },
                selected_palette: 0,
            }
        );
    }

    #[test]
    fn static_sky_crop_retains_only_a_wholly_covered_outer_band() {
        let cropped = crop_static_sky_coverage_band(&raster_with_dimensions(
            3,
            4,
            vec![
                true, true, true, true, true, true, true, true, true, false, false, false,
            ],
        ))
        .unwrap();
        assert_eq!(cropped.width, 3);
        assert_eq!(cropped.height, 3);
        assert_eq!(cropped.color_indices, (0..9).collect::<Vec<_>>());
        assert!(cropped.coverage.iter().all(|covered| *covered));
    }

    #[test]
    fn static_sky_crop_rejects_partial_or_internal_gaps() {
        assert_eq!(
            crop_static_sky_coverage_band(&raster_with_dimensions(
                3,
                2,
                vec![true, false, true, true, true, true],
            )),
            Err(DoomSkyPanoramaError::PartialCoverage {
                row: 0,
                covered: 2,
                width: 3,
            })
        );
        assert_eq!(
            crop_static_sky_coverage_band(&raster_with_dimensions(
                2,
                3,
                vec![true, true, false, false, true, true],
            )),
            Err(DoomSkyPanoramaError::InternalUncoveredRow { row: 1 })
        );
    }

    #[test]
    fn opaque_uploads_are_deterministic_and_deferred_alpha_receives_no_handle() {
        let uploads = build_static_texture_uploads(
            &[
                opaque_prepared("ZFLAT"),
                opaque_prepared("AFLAT"),
                PreparedStaticTexture {
                    eligibility: StaticTextureEligibility::DeferredAlpha {
                        texture_name: "MASKED".into(),
                        uncovered_pixels: 1,
                        descriptor: Rgba8TextureDescriptor::new(2, 2, Rgba8TextureColorSpace::Srgb),
                        sampler: TextureSampler {
                            filter: TextureFilter::Point,
                            address_u: TextureAddressMode::Repeat,
                            address_v: TextureAddressMode::Repeat,
                        },
                        selected_palette: 0,
                    },
                    rgba8: vec![0; 16],
                },
            ],
            &[opaque_prepared("WALL")],
        );
        assert_eq!(uploads.len(), 3);
        assert_eq!(uploads[0].source_kind, StaticTextureSourceKind::Flat);
        assert_eq!(uploads[0].source_name, "AFLAT");
        assert_eq!(uploads[0].texture, TextureHandle(1));
        assert_eq!(uploads[1].source_name, "ZFLAT");
        assert_eq!(uploads[2].source_kind, StaticTextureSourceKind::Wall);
        assert_eq!(uploads[2].material, MaterialHandle(3));
    }

    #[test]
    fn experimental_cutout_uploads_follow_selected_source_names_not_coverage() {
        let uploads = build_experimental_cutout_texture_uploads(
            &[
                opaque_prepared("BROWNGRN"),
                PreparedStaticTexture {
                    eligibility: StaticTextureEligibility::DeferredAlpha {
                        texture_name: "BRNBIGC".into(),
                        uncovered_pixels: 1,
                        descriptor: Rgba8TextureDescriptor::new(2, 2, Rgba8TextureColorSpace::Srgb),
                        sampler: TextureSampler {
                            filter: TextureFilter::Point,
                            address_u: TextureAddressMode::Repeat,
                            address_v: TextureAddressMode::Repeat,
                        },
                        selected_palette: 0,
                    },
                    rgba8: vec![0; 16],
                },
            ],
            41,
        );

        assert_eq!(uploads.len(), 2);
        assert_eq!(uploads[0].source_name, "BRNBIGC");
        assert_eq!(uploads[0].texture, TextureHandle(41));
        assert_eq!(uploads[1].source_name, "BROWNGRN");
        assert_eq!(uploads[1].texture, TextureHandle(42));
    }

    #[test]
    fn draw_plan_converts_only_ordinary_mesh_and_material_inputs() {
        let flats = PreparedE1m1Flats {
            map_name: "E1M1".into(),
            flat_assembly: assemble_static_opaque_flats(&[candidate()], &[], FlatExtent::E1M1)
                .unwrap(),
        };
        let wall = wall_candidate();
        let walls = PreparedE1m1Walls {
            map_name: "E1M1".into(),
            wall_assembly: assemble_static_opaque_walls(
                &[wall],
                &[],
                &[DoomTextureExtent {
                    name: "STARTAN3".into(),
                    width: 128,
                    height: 128,
                }],
            )
            .unwrap(),
        };
        let uploads = build_static_texture_uploads(
            &[opaque_prepared("FLOOR0_1")],
            &[opaque_prepared("STARTAN3")],
        );
        let draws = build_static_draw_plan(&flats, &walls, &uploads).unwrap();
        assert_eq!(draws.len(), 2);
        assert_eq!(draws[0].material, MaterialHandle(1));
        assert_eq!(draws[0].source_label, "flat:3:FLOOR0_1");
        assert_eq!(
            draws[0].source,
            StaticDrawSource::Flat {
                source_subsector: DoomSourceRecord {
                    lump_index: 6,
                    record_index: 12,
                },
                source_sector: DoomSourceRecord {
                    lump_index: 8,
                    record_index: 3,
                },
                plane: DoomSurfacePlane::Floor,
            }
        );
        assert_eq!(draws[1].material, MaterialHandle(2));
        assert_eq!(draws[1].source_label, "wall:7:STARTAN3");
        assert_eq!(
            draws[1].source,
            StaticDrawSource::Wall {
                source_linedef: DoomSourceRecord {
                    lump_index: 2,
                    record_index: 7,
                },
                source_sidedef: DoomSourceRecord {
                    lump_index: 3,
                    record_index: 9,
                },
                source_sector: DoomSourceRecord {
                    lump_index: 8,
                    record_index: 3,
                },
                role: DoomWallTextureRole::Middle,
            }
        );
    }
}
