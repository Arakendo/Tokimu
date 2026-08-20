//! Versioned Rust/WASM boundary for the DOOM browser-intake corpus study.
//!
//! Browser code supplies only user-selected bytes and descriptive metadata.
//! This session owns limits, retained resource identity, and observations.

use archive_provider::{ArchiveFormat, ArchiveReadLimits, ZipArchiveProvider};
use doom_wad_package::{read_wad_package_member, InspectWadPackageRequest};
use doom_wad_provider::WadReadLimits;
use resource_space::{
    AddressCasePolicy, FolderId, InMemoryResourceSpace, ResourceMetadata, ResourceName,
    ResourceRootDescriptor, ResourceRootId, ResourceSpaceLimits, StoreId,
};
use resource_space_archive::InspectArchiveResourceRequest;
use serde::Serialize;
use wasm_bindgen::prelude::*;

#[cfg(target_arch = "wasm32")]
use doom_geometry_provider::{
    locate_doom_point_subsector, lower_doom_paired_sky_boundary_triangles,
    lower_doom_sector_bounded_subsector_surfaces, lower_doom_textured_wall_triangles,
    observe_doom_sky_surfaces, observe_doom_two_sided_middle_textures,
    resolve_doom_subsector_bsp_paths, resolve_doom_subsector_sector_ownership,
};
#[cfg(target_arch = "wasm32")]
use doom_map_provider::{decode_doom_map_core, resolve_doom_player_one_start};
#[cfg(target_arch = "wasm32")]
use doom_wad_package::select_doom_episode_map;
#[cfg(target_arch = "wasm32")]
use hello_doom_e1m1::{
    assemble_experimental_masked_middle_cutouts, assemble_static_opaque_flats,
    assemble_static_opaque_walls, build_experimental_cutout_draw_plan,
    build_experimental_cutout_texture_uploads, build_static_draw_plan,
    build_static_texture_uploads, classify_static_draw_frustum_rejection,
    lower_static_flat_triangle, observer_direction, observer_right, observer_yaw_from_forward,
    prepare_e1m1_flat_textures, prepare_e1m1_flats, prepare_e1m1_masked_middle_cutouts,
    prepare_e1m1_sky_diagnostic_flats, prepare_e1m1_static_sky_panorama_texture,
    prepare_e1m1_wall_texture_extents, prepare_e1m1_wall_textures, prepare_e1m1_walls,
    prepared_e1m1_masked_middle_texture_names, prepared_e1m1_wall_texture_names,
    reembed_comparative_mesh, DoomComparativeEmbedding, FlatExtent, PreparedE1m1Flats,
    PreparedE1m1MaskedMiddleCutouts, PreparedE1m1Walls, StaticDrawAabb, StaticDrawPlanEntry,
    StaticDrawSource, StaticTextureEligibility,
};
#[cfg(target_arch = "wasm32")]
use hello_render_resource_identity::correlate_scene_resource_inventories;
#[cfg(any(target_arch = "wasm32", test))]
use hello_render_resource_identity::CorpusSceneResourceInventory;
#[cfg(target_arch = "wasm32")]
use raster_image_corpus::{decode_png, prepare_renderer_texture, DecodeLimits, TextureUse};
#[cfg(target_arch = "wasm32")]
use tokimu::{
    BlendMode, Camera, CameraHandle, CategoricalCutout, ClearCommand, Color, ColorWriteMask,
    CullMode, CutoutComparison, CutoutThreshold, DepthTest, DrawMeshCommand,
    ExperimentalSceneResourceResetObservation, Instance2d, Material, MaterialHandle, Mesh,
    MeshHandle, Pipeline, PipelineKind, PipelineRenderState, RenderCommand, Renderer,
    Rgba8TextureColorSpace, Rgba8TextureDescriptor, StencilMode, TextureAddressMode, TextureFilter,
    TextureHandle, TextureSampler, WgpuBackend,
};
#[cfg(target_arch = "wasm32")]
use tokimu_core::math::Vec3;
#[cfg(target_arch = "wasm32")]
use web_sys::HtmlCanvasElement;

pub const SCHEMA_VERSION: u32 = 1;
pub const MAX_INPUT_BYTES: usize = 64 * 1024 * 1024;
#[cfg(any(target_arch = "wasm32", test))]
const MAX_BROWSER_WORKING_MESHES: u64 = 20_000;
#[cfg(any(target_arch = "wasm32", test))]
const MAX_BROWSER_WORKING_TEXTURES: u64 = 2_048;
#[cfg(any(target_arch = "wasm32", test))]
const MAX_BROWSER_WORKING_MATERIALS: u64 = 2_050;
#[cfg(any(target_arch = "wasm32", test))]
const MAX_BROWSER_WORKING_PIPELINES: u64 = 16;
#[cfg(any(target_arch = "wasm32", test))]
const MAX_BROWSER_WORKING_CAMERAS: u64 = 1;
#[cfg(any(target_arch = "wasm32", test))]
const MAX_BROWSER_WORKING_COMMANDS: u64 = 100_000;
#[cfg(any(target_arch = "wasm32", test))]
const MAX_BROWSER_WORKING_MESH_VERTEX_BYTES: u64 = 64 * 1024 * 1024;
#[cfg(any(target_arch = "wasm32", test))]
const MAX_BROWSER_WORKING_TEXTURE_PAYLOAD_BYTES: u64 = 128 * 1024 * 1024;

#[cfg(target_arch = "wasm32")]
const WAD_LIMITS: WadReadLimits =
    WadReadLimits::new(64 * 1024 * 1024, 8_192, 16 * 1024 * 1024, 64 * 1024 * 1024);
#[cfg(target_arch = "wasm32")]
const MAP_LIMITS: doom_map_provider::DoomMapDecodeLimits = doom_map_provider::DoomMapDecodeLimits {
    max_things: 100_000,
    max_vertices: 100_000,
    max_linedefs: 100_000,
    max_sidedefs: 100_000,
    max_sectors: 100_000,
    max_segs: 100_000,
    max_subsectors: 100_000,
    max_nodes: 100_000,
    max_reject_bytes: 64 * 1024 * 1024,
    max_blockmap_bytes: 64 * 1024 * 1024,
    max_blockmap_cells: 1_000_000,
    max_blockmap_linedef_refs: 10_000_000,
    max_total_record_bytes: 64 * 1024 * 1024,
};
#[cfg(target_arch = "wasm32")]
const RASTER_LIMITS: doom_raster_provider::DoomRasterDecodeLimits =
    doom_raster_provider::DoomRasterDecodeLimits {
        max_playpal_bytes: 64 * 1024 * 1024,
        max_palettes: 4096,
        max_colormap_bytes: 64 * 1024 * 1024,
        max_colormaps: 4096,
        max_total_decoded_bytes: 128 * 1024 * 1024,
    };
#[cfg(target_arch = "wasm32")]
const DIAGNOSTIC_SKY_TEXTURE: TextureHandle = TextureHandle(9_000_010);
#[cfg(target_arch = "wasm32")]
const DIAGNOSTIC_SKY_MATERIAL: MaterialHandle = MaterialHandle(9_000_010);
#[cfg(target_arch = "wasm32")]
const WORKING_SKY_TEXTURE: TextureHandle = TextureHandle(9_100_000);
#[cfg(target_arch = "wasm32")]
const WORKING_SKY_MATERIAL: MaterialHandle = MaterialHandle(9_100_000);
#[cfg(target_arch = "wasm32")]
const WORKING_SKY_BOUNDARY_MATERIAL: MaterialHandle = MaterialHandle(9_100_001);
#[cfg(target_arch = "wasm32")]
const WORKING_CAMERA: CameraHandle = CameraHandle(91);
#[cfg(target_arch = "wasm32")]
const FLAT_LIMITS: doom_raster_provider::DoomFlatDecodeLimits =
    doom_raster_provider::DoomFlatDecodeLimits {
        max_flat_bytes: 4096,
    };
#[cfg(target_arch = "wasm32")]
const TEXTURE_LIMITS: doom_raster_provider::DoomTextureDecodeLimits =
    doom_raster_provider::DoomTextureDecodeLimits {
        max_pnames_bytes: 64 * 1024 * 1024,
        max_texture_bytes: 64 * 1024 * 1024,
        max_patch_names: 1_000_000,
        max_textures: 1_000_000,
        max_patches_per_texture: 16_384,
        max_total_patch_references: 10_000_000,
    };
#[cfg(target_arch = "wasm32")]
const PATCH_LIMITS: doom_raster_provider::DoomPatchDecodeLimits =
    doom_raster_provider::DoomPatchDecodeLimits {
        max_patch_bytes: 64 * 1024 * 1024,
        max_width: 4096,
        max_height: 4096,
        max_pixels: 16 * 1024 * 1024,
        max_posts: 16 * 1024 * 1024,
    };
#[cfg(target_arch = "wasm32")]
const COMPOSE_LIMITS: doom_raster_provider::DoomTextureComposeLimits =
    doom_raster_provider::DoomTextureComposeLimits {
        max_width: 4096,
        max_height: 4096,
        max_pixels: 16 * 1024 * 1024,
    };

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct IntakeObservation {
    schema_version: u32,
    source_label: String,
    media_hint: String,
    byte_length: usize,
    fingerprint_blake3: String,
    retained_resources: usize,
    retained_bytes: usize,
    status: &'static str,
}

#[cfg(target_arch = "wasm32")]
struct BrowserWorkingModel {
    renderer: WgpuBackend,
    commands: Vec<RenderCommand>,
    logical_resources: WorkingLogicalResources,
    semantic_inventory: CorpusSceneResourceInventory,
    position: Vec3,
    yaw: f32,
    pitch: f32,
    width: u32,
    height: u32,
    far_plane: f32,
}

#[cfg(any(target_arch = "wasm32", test))]
#[derive(Clone, Copy, Debug, Default)]
struct WorkingLogicalResources {
    meshes: u64,
    textures: u64,
    materials: u64,
    pipelines: u64,
    cameras: u64,
    commands: u64,
    mesh_vertex_bytes: u64,
    source_texture_payload_bytes: u64,
}

#[cfg(any(target_arch = "wasm32", test))]
fn validate_browser_working_model_budget(resources: WorkingLogicalResources) -> Result<(), String> {
    let limits = [
        ("meshes", resources.meshes, MAX_BROWSER_WORKING_MESHES),
        ("textures", resources.textures, MAX_BROWSER_WORKING_TEXTURES),
        (
            "materials",
            resources.materials,
            MAX_BROWSER_WORKING_MATERIALS,
        ),
        (
            "pipelines",
            resources.pipelines,
            MAX_BROWSER_WORKING_PIPELINES,
        ),
        ("cameras", resources.cameras, MAX_BROWSER_WORKING_CAMERAS),
        ("commands", resources.commands, MAX_BROWSER_WORKING_COMMANDS),
        (
            "mesh-vertex-bytes",
            resources.mesh_vertex_bytes,
            MAX_BROWSER_WORKING_MESH_VERTEX_BYTES,
        ),
        (
            "source-texture-payload-bytes",
            resources.source_texture_payload_bytes,
            MAX_BROWSER_WORKING_TEXTURE_PAYLOAD_BYTES,
        ),
    ];
    for (resource, observed, limit) in limits {
        if observed > limit {
            return Err(format!(
                "browser working-model budget exceeded: resource={resource}; observed={observed}; limit={limit}"
            ));
        }
    }
    Ok(())
}

#[cfg(any(target_arch = "wasm32", test))]
fn working_semantic_inventory(
    source_label: String,
    resources: WorkingLogicalResources,
) -> CorpusSceneResourceInventory {
    CorpusSceneResourceInventory {
        source_label,
        meshes: resources.meshes,
        textures: resources.textures,
        materials: resources.materials,
        pipelines: resources.pipelines,
        cameras: resources.cameras,
        commands: resources.commands,
    }
}

#[cfg(target_arch = "wasm32")]
#[derive(Clone, Copy, Debug, Default)]
struct WorkingLifetimeObservation {
    replacement_attempts: u64,
    replacements_presented: u64,
    backend_creations: u64,
    device_creations: u64,
    surface_creations: u64,
    scene_resets: u64,
    retired_sets: u64,
    retired_resources: WorkingLogicalResources,
}

#[cfg(target_arch = "wasm32")]
impl WorkingLifetimeObservation {
    fn retire(&mut self, resources: WorkingLogicalResources) {
        self.retired_sets = self.retired_sets.saturating_add(1);
        self.retired_resources.meshes = self
            .retired_resources
            .meshes
            .saturating_add(resources.meshes);
        self.retired_resources.textures = self
            .retired_resources
            .textures
            .saturating_add(resources.textures);
        self.retired_resources.materials = self
            .retired_resources
            .materials
            .saturating_add(resources.materials);
        self.retired_resources.pipelines = self
            .retired_resources
            .pipelines
            .saturating_add(resources.pipelines);
        self.retired_resources.cameras = self
            .retired_resources
            .cameras
            .saturating_add(resources.cameras);
        self.retired_resources.commands = self
            .retired_resources
            .commands
            .saturating_add(resources.commands);
        self.retired_resources.mesh_vertex_bytes = self
            .retired_resources
            .mesh_vertex_bytes
            .saturating_add(resources.mesh_vertex_bytes);
        self.retired_resources.source_texture_payload_bytes = self
            .retired_resources
            .source_texture_payload_bytes
            .saturating_add(resources.source_texture_payload_bytes);
    }
}

/// One transient Rust-owned selection session. It exposes no browser path,
/// directory, fetch, storage, or Doom semantic API.
#[wasm_bindgen]
pub struct BrowserIntakeSession {
    space: InMemoryResourceSpace,
    folder: FolderId,
    #[cfg(target_arch = "wasm32")]
    working_model: Option<BrowserWorkingModel>,
    #[cfg(target_arch = "wasm32")]
    working_lifetime: WorkingLifetimeObservation,
}

#[wasm_bindgen]
impl BrowserIntakeSession {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Result<Self, JsValue> {
        Self::new_inner().map_err(js_error)
    }

    /// Replaces the current explicit selection atomically within this bounded
    /// session and returns a provider-neutral JSON observation.
    pub fn import_selected_package(
        &mut self,
        source_label: &str,
        media_hint: &str,
        bytes: &[u8],
    ) -> Result<String, JsValue> {
        self.import_selected_package_inner(source_label, media_hint, bytes)
            .map_err(js_error)
    }

    /// Releases all selected bytes by replacing the bounded session.
    pub fn dispose(&mut self) -> Result<(), JsValue> {
        *self = Self::new_inner().map_err(js_error)?;
        Ok(())
    }

    /// Inspects the canonical `DOOM1.WAD` ZIP member through Rust providers.
    /// The WAD is a transient derived read, never a second retained browser
    /// resource, and TypeScript receives only this compact observation.
    pub fn inspect_doom1_wad(&self) -> Result<String, JsValue> {
        self.inspect_doom1_wad_inner().map_err(js_error)
    }

    /// Presents one fixed-camera static E1M1 frame from the already retained
    /// package. This is a consumer-local WASM proof, not a browser renderer API.
    #[cfg(target_arch = "wasm32")]
    pub async fn render_static_e1m1(&self, canvas: HtmlCanvasElement) -> Result<String, JsValue> {
        self.render_static_e1m1_inner(canvas, false, false, false, false)
            .await
            .map_err(js_error)
    }

    /// Presents the same fixed E1M1 scene plus its corpus-local, source-
    /// selected masked-middle cutout candidates. TypeScript still supplies only
    /// a canvas; Rust owns all Doom parsing and policy selection.
    #[cfg(target_arch = "wasm32")]
    pub async fn render_static_e1m1_masked_cutouts(
        &self,
        canvas: HtmlCanvasElement,
    ) -> Result<String, JsValue> {
        self.render_static_e1m1_inner(canvas, true, false, false, false)
            .await
            .map_err(js_error)
    }

    /// Presents source-spawn E1M1 after corpus-local conservative AABB/frustum
    /// selection. This is AR-0025 target evidence, not a general renderer
    /// visibility contract or a TypeScript-owned scene operation.
    #[cfg(target_arch = "wasm32")]
    pub async fn render_static_e1m1_selected_cutouts(
        &self,
        canvas: HtmlCanvasElement,
    ) -> Result<String, JsValue> {
        self.render_static_e1m1_inner(canvas, true, true, false, false)
            .await
            .map_err(js_error)
    }

    /// Presents the canonical E1M1 EXITSIGN walls from their owning side.
    /// This fixed corpus camera exists only for AR-0028 browser orientation
    /// evidence; neither the renderer nor TypeScript interprets Doom sides.
    #[cfg(target_arch = "wasm32")]
    pub async fn render_e1m1_exitsign(&self, canvas: HtmlCanvasElement) -> Result<String, JsValue> {
        self.render_static_e1m1_inner(canvas, false, false, true, false)
            .await
            .map_err(js_error)
    }

    /// Presents the same overview plus an application-selected Purple PNG on
    /// retained sky omissions. This is AR-0027 browser evidence: the stand-in
    /// is opt-in, carries a bounded omission count, and is not renderer fallback.
    #[cfg(target_arch = "wasm32")]
    pub async fn render_e1m1_diagnostic_sky_omissions(
        &self,
        canvas: HtmlCanvasElement,
    ) -> Result<String, JsValue> {
        self.render_static_e1m1_inner(canvas, false, false, false, true)
            .await
            .map_err(js_error)
    }

    /// Presents one source-spawn frame for an explicitly selected shareware
    /// episode map using the current native working-model plane trim and
    /// grouped sky parity sequence. TypeScript transports only the requested
    /// marker and canvas; Rust validates the marker and owns all preparation.
    #[cfg(target_arch = "wasm32")]
    pub async fn render_working_map(
        &mut self,
        canvas: HtmlCanvasElement,
        map_name: &str,
    ) -> Result<String, JsValue> {
        self.render_working_map_inner(canvas, map_name, false)
            .await
            .map_err(js_error)
    }

    /// Presents the same corpus-private working model while replacing its
    /// logical resources inside one retained WGPU provider session. This is
    /// Alternative-B evidence, not an admitted browser renderer contract.
    #[cfg(target_arch = "wasm32")]
    pub async fn render_working_map_retained_session(
        &mut self,
        canvas: HtmlCanvasElement,
        map_name: &str,
    ) -> Result<String, JsValue> {
        self.render_working_map_inner(canvas, map_name, true)
            .await
            .map_err(js_error)
    }

    /// Advances and presents the retained working-model scene. This is an
    /// explicit browser noclip inspection camera, not Doom player simulation.
    #[cfg(target_arch = "wasm32")]
    pub fn step_working_model(
        &mut self,
        delta_seconds: f32,
        forward_axis: f32,
        strafe_axis: f32,
        vertical_axis: f32,
        yaw_delta: f32,
        pitch_delta: f32,
        running: bool,
    ) -> Result<(), JsValue> {
        self.step_working_model_inner(
            delta_seconds,
            forward_axis,
            strafe_axis,
            vertical_axis,
            yaw_delta,
            pitch_delta,
            running,
        )
        .map_err(js_error)
    }
}

impl BrowserIntakeSession {
    fn new_inner() -> Result<Self, String> {
        const STORE: StoreId = StoreId::from_u128(0xD001_0001);
        const ROOT: ResourceRootId = ResourceRootId::from_u128(0xD001_0002);
        const FOLDER: FolderId = FolderId::from_u128(0xD001_0003);
        let mut space = InMemoryResourceSpace::with_limits(
            STORE,
            AddressCasePolicy::Sensitive,
            ResourceSpaceLimits {
                max_entries: Some(1),
                max_total_bytes: Some(MAX_INPUT_BYTES),
                max_bytes_per_entry: Some(MAX_INPUT_BYTES),
            },
        );
        space
            .create_root(
                ResourceRootDescriptor::new(ROOT, "Browser-selected DOOM package"),
                FOLDER,
                ResourceMetadata::default(),
            )
            .map_err(|error| error.to_string())?;
        Ok(Self {
            space,
            folder: FOLDER,
            #[cfg(target_arch = "wasm32")]
            working_model: None,
            #[cfg(target_arch = "wasm32")]
            working_lifetime: WorkingLifetimeObservation::default(),
        })
    }

    fn import_selected_package_inner(
        &mut self,
        source_label: &str,
        media_hint: &str,
        bytes: &[u8],
    ) -> Result<String, String> {
        if source_label.is_empty() {
            return Err("selected package has an empty source label".into());
        }
        if bytes.is_empty() {
            return Err("selected package is empty".into());
        }
        if bytes.len() > MAX_INPUT_BYTES {
            return Err(format!(
                "selected package has {} bytes, exceeding the limit of {MAX_INPUT_BYTES}",
                bytes.len()
            ));
        }
        // A new selection gets a new bounded session: no TypeScript-held
        // identity decides replacement, and old bytes do not survive it.
        *self = Self::new_inner()?;
        let name = ResourceName::parse("selected-doom-package", AddressCasePolicy::Sensitive)
            .map_err(|error| error.to_string())?;
        let entry = self
            .space
            .insert_resource(
                self.folder,
                name,
                bytes.to_vec(),
                ResourceMetadata::default(),
            )
            .map_err(|error| error.to_string())?;
        let summary = self.space.summary();
        let observation = IntakeObservation {
            schema_version: SCHEMA_VERSION,
            source_label: source_label.to_owned(),
            media_hint: media_hint.to_owned(),
            byte_length: entry.byte_len(),
            fingerprint_blake3: entry
                .content_fingerprint()
                .digest()
                .iter()
                .map(|byte| format!("{byte:02x}"))
                .collect(),
            retained_resources: summary.resources(),
            retained_bytes: summary.retained_bytes(),
            status: "retained",
        };
        serde_json::to_string(&observation).map_err(|error| error.to_string())
    }

    fn inspect_doom1_wad_inner(&self) -> Result<String, String> {
        let name = ResourceName::parse("selected-doom-package", AddressCasePolicy::Sensitive)
            .map_err(|error| error.to_string())?;
        let read = read_wad_package_member(
            &self.space,
            InspectWadPackageRequest {
                archive: InspectArchiveResourceRequest {
                    source_folder: self.folder,
                    source_name: name,
                    format: ArchiveFormat::Zip,
                    limits: ArchiveReadLimits::new(
                        64 * 1024 * 1024,
                        2048,
                        16 * 1024 * 1024,
                        64 * 1024 * 1024,
                        4096,
                    ),
                },
                member_name: "DOOM1.WAD".into(),
                wad_source_label: "browser-selected:DOOM1.WAD".into(),
                wad_limits: WadReadLimits::new(
                    64 * 1024 * 1024,
                    8192,
                    16 * 1024 * 1024,
                    64 * 1024 * 1024,
                ),
            },
            &ZipArchiveProvider,
        )
        .map_err(|error| error.to_string())?;
        serde_json::to_string(&serde_json::json!({
            "schemaVersion": SCHEMA_VERSION, "status": "observed", "member": read.observation.member.normalized_name,
            "wadKind": format!("{:?}", read.observation.wad.kind), "wadBytes": read.bytes.len(),
            "lumpCount": read.observation.wad.lumps.len(), "retainedResources": self.space.summary().resources(),
        })).map_err(|error| error.to_string())
    }

    #[cfg(target_arch = "wasm32")]
    async fn render_working_map_inner(
        &mut self,
        canvas: HtmlCanvasElement,
        map_name: &str,
        retain_provider_session: bool,
    ) -> Result<String, String> {
        self.working_lifetime.replacement_attempts =
            self.working_lifetime.replacement_attempts.saturating_add(1);
        if !matches!(map_name.as_bytes(), [b'E', b'1', b'M', b'1'..=b'9']) {
            return Err(format!(
                "working-model browser map must be E1M1 through E1M9; got {map_name}"
            ));
        }
        let name = ResourceName::parse("selected-doom-package", AddressCasePolicy::Sensitive)
            .map_err(|error| error.to_string())?;
        let read = read_wad_package_member(
            &self.space,
            InspectWadPackageRequest {
                archive: InspectArchiveResourceRequest {
                    source_folder: self.folder,
                    source_name: name,
                    format: ArchiveFormat::Zip,
                    limits: ArchiveReadLimits::new(
                        64 * 1024 * 1024,
                        2048,
                        16 * 1024 * 1024,
                        64 * 1024 * 1024,
                        4096,
                    ),
                },
                member_name: "DOOM1.WAD".into(),
                wad_source_label: "browser-selected:DOOM1.WAD".into(),
                wad_limits: WAD_LIMITS,
            },
            &ZipArchiveProvider,
        )
        .map_err(|error| error.to_string())?;
        let selection = select_doom_episode_map(&read.observation.wad, map_name)
            .map_err(|error| error.to_string())?;
        let map = decode_doom_map_core(&read.bytes, &selection, MAP_LIMITS)
            .map_err(|error| error.to_string())?;
        let paths = resolve_doom_subsector_bsp_paths(&map).map_err(|error| error.to_string())?;
        let surface_bake = lower_doom_sector_bounded_subsector_surfaces(&map, &paths)
            .map_err(|error| error.to_string())?;
        let sky_surfaces =
            observe_doom_sky_surfaces(&map, &paths).map_err(|error| error.to_string())?;
        let flats = PreparedE1m1Flats {
            map_name: map.map_name.clone(),
            flat_assembly: assemble_static_opaque_flats(
                &surface_bake.surfaces,
                &sky_surfaces,
                FlatExtent::E1M1,
            )
            .map_err(|error| error.to_string())?,
        };
        let wall_extents =
            prepare_e1m1_wall_texture_extents(&read.bytes, &read.observation.wad, TEXTURE_LIMITS)
                .map_err(|error| error.to_string())?;
        let source_walls = lower_doom_textured_wall_triangles(&map, &wall_extents)
            .map_err(|error| error.to_string())?;
        let masked_middles =
            observe_doom_two_sided_middle_textures(&map).map_err(|error| error.to_string())?;
        let walls = PreparedE1m1Walls {
            map_name: map.map_name.clone(),
            wall_assembly: assemble_static_opaque_walls(
                &source_walls,
                &masked_middles,
                &wall_extents,
            )
            .map_err(|error| error.to_string())?,
        };
        let cutouts = PreparedE1m1MaskedMiddleCutouts {
            map_name: map.map_name.clone(),
            assembly: assemble_experimental_masked_middle_cutouts(
                &source_walls,
                &masked_middles,
                &wall_extents,
            )
            .map_err(|error| error.to_string())?,
        };
        let flat_textures = prepare_e1m1_flat_textures(
            &read.bytes,
            &read.observation.wad,
            &flats,
            RASTER_LIMITS,
            FLAT_LIMITS,
        )
        .map_err(|error| error.to_string())?;
        let wall_names = prepared_e1m1_wall_texture_names(&walls);
        let wall_textures = prepare_e1m1_wall_textures(
            &read.bytes,
            &read.observation.wad,
            &wall_names,
            RASTER_LIMITS,
            TEXTURE_LIMITS,
            PATCH_LIMITS,
            COMPOSE_LIMITS,
        )
        .map_err(|error| error.to_string())?;
        let uploads = build_static_texture_uploads(&flat_textures, &wall_textures);
        let mut draws =
            build_static_draw_plan(&flats, &walls, &uploads).map_err(|error| error.to_string())?;
        let masked_names = prepared_e1m1_masked_middle_texture_names(&walls);
        let masked_textures = prepare_e1m1_wall_textures(
            &read.bytes,
            &read.observation.wad,
            &masked_names,
            RASTER_LIMITS,
            TEXTURE_LIMITS,
            PATCH_LIMITS,
            COMPOSE_LIMITS,
        )
        .map_err(|error| error.to_string())?;
        let cutout_uploads =
            build_experimental_cutout_texture_uploads(&masked_textures, uploads.len() as u64 + 1);
        let mut cutout_draws = build_experimental_cutout_draw_plan(&cutouts, &cutout_uploads)
            .map_err(|error| error.to_string())?;

        let mut sky_plane_meshes = Vec::new();
        for surface in &surface_bake.surfaces {
            if !sky_surfaces.iter().any(|sky| {
                sky.source_subsector == surface.source_subsector
                    && sky.source_sector == surface.source_sector
                    && sky.plane == surface.plane
            }) {
                continue;
            }
            match lower_static_flat_triangle(surface, FlatExtent::E1M1) {
                Ok(flat) => sky_plane_meshes.push(flat.mesh),
                Err(hello_doom_e1m1::StaticFlatLoweringError::DegenerateTriangle) => {}
                Err(error) => return Err(error.to_string()),
            }
        }
        let mut skywall_meshes = lower_doom_paired_sky_boundary_triangles(&map)
            .map_err(|error| error.to_string())?
            .into_iter()
            .map(|triangle| {
                Mesh::uniform_normal(
                    triangle
                        .positions
                        .into_iter()
                        .map(|position| position.map(|component| component as f32))
                        .collect(),
                    [0.0, 1.0, 0.0],
                )
            })
            .collect::<Vec<_>>();

        let embedding = DoomComparativeEmbedding::PreserveNorth;
        reembed_browser_draws(&mut draws, embedding);
        reembed_browser_draws(&mut cutout_draws, embedding);
        for mesh in sky_plane_meshes.iter_mut().chain(skywall_meshes.iter_mut()) {
            reembed_comparative_mesh(mesh, embedding, false);
        }

        let start =
            resolve_doom_player_one_start(&map.things).map_err(|error| error.to_string())?;
        let location = locate_doom_point_subsector(start.position, &paths)
            .map_err(|error| error.to_string())?;
        let ownership =
            resolve_doom_subsector_sector_ownership(&map).map_err(|error| error.to_string())?;
        let owner = ownership
            .iter()
            .find(|entry| entry.source_subsector == location.source_subsector)
            .ok_or_else(|| "player-one start subsector has no sector ownership".to_owned())?;
        let sector = &map.sectors[usize::from(owner.sector_index)];
        let eye_height = (f32::from(sector.floor_height) + f32::from(sector.ceiling_height)) * 0.5;
        let observer = embedding.lift_direction(start.position.map(f32::from), eye_height);
        let forward = embedding.lift_heading_degrees(f32::from(start.angle));

        let (center, radius) = working_scene_bounds(&draws, &cutout_draws)?;
        let sky_mesh =
            build_working_sky_cylinder(center, radius).map_err(|error| error.to_string())?;
        let sky_texture = prepare_e1m1_static_sky_panorama_texture(
            &read.bytes,
            &read.observation.wad,
            RASTER_LIMITS,
            TEXTURE_LIMITS,
            PATCH_LIMITS,
            COMPOSE_LIMITS,
        )
        .map_err(|error| error.to_string())?;
        let StaticTextureEligibility::Opaque(sky_info) = &sky_texture.eligibility else {
            return Err("SKY1 working-model panorama was not opaque after bounded crop".into());
        };

        let width = canvas.width().max(1);
        let height = canvas.height().max(1);
        let expected_commands = 2_usize
            .saturating_add(draws.len().saturating_mul(2))
            .saturating_add(cutout_draws.len().saturating_mul(2))
            .saturating_add(skywall_meshes.len())
            .saturating_add(sky_plane_meshes.len());
        let logical_resources = WorkingLogicalResources {
            meshes: (1
                + draws.len()
                + cutout_draws.len()
                + skywall_meshes.len()
                + sky_plane_meshes.len()) as u64,
            textures: (uploads.len() + cutout_uploads.len() + 1) as u64,
            materials: (uploads.len() + cutout_uploads.len() + 2) as u64,
            pipelines: 7,
            cameras: 1,
            commands: expected_commands as u64,
            mesh_vertex_bytes: working_mesh_vertex_bytes(
                &draws,
                &cutout_draws,
                &skywall_meshes,
                &sky_plane_meshes,
                &sky_mesh,
            ),
            source_texture_payload_bytes: uploads
                .iter()
                .chain(&cutout_uploads)
                .map(|upload| upload.rgba8.len() as u64)
                .sum::<u64>()
                .saturating_add(sky_texture.rgba8.len() as u64),
        };
        validate_browser_working_model_budget(logical_resources)?;
        let current_semantic_inventory = working_semantic_inventory(
            format!("DOOM {} prepared inventory", map.map_name),
            logical_resources,
        );
        let previous_semantic_inventory = self
            .working_model
            .as_ref()
            .map(|previous| previous.semantic_inventory.clone());
        // CPU preparation above coexists with the current map. Alternative A
        // then replaces the whole backend; the private Alternative-B path
        // keeps its provider session but retires the old logical scene before
        // uploading the successor. The latter is intentionally not claimed to
        // preserve the last-known-good scene if GPU staging fails.
        let mut retained_renderer = None;
        let mut reset_observation: Option<ExperimentalSceneResourceResetObservation> = None;
        if let Some(previous) = self.working_model.take() {
            self.working_lifetime.retire(previous.logical_resources);
            if retain_provider_session && previous.width == width && previous.height == height {
                let BrowserWorkingModel {
                    mut renderer,
                    logical_resources: _,
                    semantic_inventory: _,
                    commands: _,
                    position: _,
                    yaw: _,
                    pitch: _,
                    width: _,
                    height: _,
                    far_plane: _,
                } = previous;
                let observation = renderer.experimental_reset_scene_resources();
                self.working_lifetime.scene_resets =
                    self.working_lifetime.scene_resets.saturating_add(1);
                reset_observation = Some(observation);
                retained_renderer = Some(renderer);
            }
        }
        let mut renderer = if let Some(renderer) = retained_renderer {
            renderer
        } else {
            let renderer = WgpuBackend::for_window(canvas, width, height)
                .await
                .map_err(|error| error.to_string())?;
            self.working_lifetime.backend_creations =
                self.working_lifetime.backend_creations.saturating_add(1);
            self.working_lifetime.device_creations =
                self.working_lifetime.device_creations.saturating_add(1);
            self.working_lifetime.surface_creations =
                self.working_lifetime.surface_creations.saturating_add(1);
            renderer
        };
        let adapter_name = renderer.adapter_name().to_owned();
        let backend_api = renderer.backend_api();
        let device_kind = renderer.device_kind();
        for upload in uploads.iter().chain(&cutout_uploads) {
            renderer
                .create_texture_rgba8(upload.texture, upload.descriptor, &upload.rgba8)
                .map_err(|error| error.to_string())?;
            renderer
                .upload_material(upload.material, &upload.material_value)
                .map_err(|error| error.to_string())?;
        }
        renderer
            .create_texture_rgba8(WORKING_SKY_TEXTURE, sky_info.descriptor, &sky_texture.rgba8)
            .map_err(|error| error.to_string())?;
        renderer
            .upload_material(
                WORKING_SKY_MATERIAL,
                &Material::new("doom-browser-working-sky", Color::rgb(1.0, 1.0, 1.0))
                    .with_texture(WORKING_SKY_TEXTURE)
                    .with_texture_sampler(sky_info.sampler),
            )
            .map_err(|error| error.to_string())?;
        renderer
            .upload_material(
                WORKING_SKY_BOUNDARY_MATERIAL,
                &Material::new(
                    "doom-browser-working-sky-boundary",
                    Color::rgb(0.0, 0.0, 0.0),
                ),
            )
            .map_err(|error| error.to_string())?;

        const SKY_MESH: MeshHandle = MeshHandle(9_100_000);
        const OPAQUE_MESH_BASE: u64 = 1;
        let cutout_mesh_base = draws.len() as u64 + 1;
        let skywall_mesh_base = cutout_mesh_base + cutout_draws.len() as u64;
        let sky_plane_mesh_base = skywall_mesh_base + skywall_meshes.len() as u64;
        renderer.upload_mesh(SKY_MESH, &sky_mesh);
        for (index, draw) in draws.iter().enumerate() {
            renderer.upload_mesh(MeshHandle(OPAQUE_MESH_BASE + index as u64), &draw.mesh);
        }
        for (index, draw) in cutout_draws.iter().enumerate() {
            renderer.upload_mesh(MeshHandle(cutout_mesh_base + index as u64), &draw.mesh);
        }
        for (index, mesh) in skywall_meshes.iter().enumerate() {
            renderer.upload_mesh(MeshHandle(skywall_mesh_base + index as u64), mesh);
        }
        for (index, mesh) in sky_plane_meshes.iter().enumerate() {
            renderer.upload_mesh(MeshHandle(sky_plane_mesh_base + index as u64), mesh);
        }

        let opaque_state = PipelineRenderState {
            blend: BlendMode::Opaque,
            depth_test: DepthTest::LessEqual,
            depth_write: true,
            cull_mode: CullMode::Back,
            color_write: ColorWriteMask::ALL,
        };
        let opaque_pipeline = renderer
            .register_pipeline(
                &Pipeline::new("doom-browser-working-opaque", PipelineKind::Textured3d)
                    .with_render_state(opaque_state)
                    .map_err(|error| error.to_string())?
                    .with_stencil_mode(StencilMode::RequireZero),
            )
            .map_err(|error| error.to_string())?;
        let opaque_depth_pipeline = renderer
            .register_pipeline(
                &Pipeline::new(
                    "doom-browser-working-opaque-depth",
                    PipelineKind::Textured3d,
                )
                .with_render_state(PipelineRenderState {
                    color_write: ColorWriteMask::NONE,
                    ..opaque_state
                })
                .map_err(|error| error.to_string())?,
            )
            .map_err(|error| error.to_string())?;
        let one_sided_depth_pipeline = renderer
            .register_pipeline(
                &Pipeline::new(
                    "doom-browser-working-one-sided-depth",
                    PipelineKind::Textured3d,
                )
                .with_render_state(PipelineRenderState {
                    cull_mode: CullMode::None,
                    color_write: ColorWriteMask::NONE,
                    ..opaque_state
                })
                .map_err(|error| error.to_string())?,
            )
            .map_err(|error| error.to_string())?;
        let sky_pipeline = renderer
            .register_pipeline(
                &Pipeline::new("doom-browser-working-panorama", PipelineKind::Textured3d)
                    .with_render_state(PipelineRenderState {
                        blend: BlendMode::Opaque,
                        depth_test: DepthTest::LessEqual,
                        depth_write: false,
                        cull_mode: CullMode::None,
                        color_write: ColorWriteMask::ALL,
                    })
                    .map_err(|error| error.to_string())?,
            )
            .map_err(|error| error.to_string())?;
        let boundary_pipeline = renderer
            .register_pipeline(
                &Pipeline::new(
                    "doom-browser-working-sky-boundary",
                    PipelineKind::LitColor3d,
                )
                .with_render_state(PipelineRenderState {
                    blend: BlendMode::Opaque,
                    depth_test: DepthTest::LessEqual,
                    depth_write: false,
                    cull_mode: CullMode::None,
                    color_write: ColorWriteMask::NONE,
                })
                .map_err(|error| error.to_string())?
                .with_stencil_mode(StencilMode::InvertOnDepthPass),
            )
            .map_err(|error| error.to_string())?;
        let cutout = CategoricalCutout::new(
            CutoutThreshold::new(0.0).map_err(|error| error.to_string())?,
            CutoutComparison::DiscardAtOrBelow,
        );
        let cutout_pipeline = renderer
            .register_pipeline(
                &Pipeline::textured_3d_cutout("doom-browser-working-cutout", cutout)
                    .with_stencil_mode(StencilMode::RequireZero),
            )
            .map_err(|error| error.to_string())?;
        let cutout_depth_pipeline = renderer
            .register_pipeline(
                &Pipeline::textured_3d_cutout("doom-browser-working-cutout-depth", cutout)
                    .with_render_state(PipelineRenderState::categorical_cutout_depth_prepass_3d())
                    .map_err(|error| error.to_string())?,
            )
            .map_err(|error| error.to_string())?;

        let far_plane = radius * 8.0;
        let yaw = observer_yaw_from_forward(forward);
        let pitch = 0.0;
        let mut camera = Camera::perspective_3d(width as f32, height as f32);
        camera.projection = tokimu_core::math::try_projection_perspective_rh_gl(
            60_f32.to_radians(),
            width as f32 / height as f32,
            0.1,
            far_plane,
        )
        .ok_or_else(|| "working-model camera projection is invalid".to_owned())?;
        camera.view = tokimu_core::math::try_view_look_at_rh(
            observer,
            observer + observer_direction(yaw, pitch) * 128.0,
            Vec3::Y,
        )
        .ok_or_else(|| "working-model source-spawn camera basis is invalid".to_owned())?;
        renderer.upload_camera(WORKING_CAMERA, camera);

        let mut commands = vec![
            RenderCommand::Clear(ClearCommand {
                color: Color::rgb(0.015, 0.02, 0.025),
            }),
            RenderCommand::DrawMesh(DrawMeshCommand {
                mesh: SKY_MESH,
                material: WORKING_SKY_MATERIAL,
                pipeline: sky_pipeline,
                instance: Instance2d::identity(),
                camera: Some(WORKING_CAMERA),
                viewport: None,
            }),
        ];
        for (index, draw) in draws.iter().enumerate() {
            commands.push(RenderCommand::DrawMesh(DrawMeshCommand {
                mesh: MeshHandle(OPAQUE_MESH_BASE + index as u64),
                material: draw.material,
                pipeline: if is_working_one_sided_wall(draw, &map) {
                    one_sided_depth_pipeline
                } else {
                    opaque_depth_pipeline
                },
                instance: Instance2d::identity(),
                camera: Some(WORKING_CAMERA),
                viewport: None,
            }));
        }
        for (index, draw) in cutout_draws.iter().enumerate() {
            commands.push(RenderCommand::DrawMesh(DrawMeshCommand {
                mesh: MeshHandle(cutout_mesh_base + index as u64),
                material: draw.material,
                pipeline: cutout_depth_pipeline,
                instance: Instance2d::identity(),
                camera: Some(WORKING_CAMERA),
                viewport: None,
            }));
        }
        for index in 0..skywall_meshes.len() {
            commands.push(RenderCommand::DrawMesh(DrawMeshCommand {
                mesh: MeshHandle(skywall_mesh_base + index as u64),
                material: WORKING_SKY_BOUNDARY_MATERIAL,
                pipeline: boundary_pipeline,
                instance: Instance2d::identity(),
                camera: Some(WORKING_CAMERA),
                viewport: None,
            }));
        }
        for index in 0..sky_plane_meshes.len() {
            commands.push(RenderCommand::DrawMesh(DrawMeshCommand {
                mesh: MeshHandle(sky_plane_mesh_base + index as u64),
                material: WORKING_SKY_BOUNDARY_MATERIAL,
                pipeline: boundary_pipeline,
                instance: Instance2d::identity(),
                camera: Some(WORKING_CAMERA),
                viewport: None,
            }));
        }
        for (index, draw) in draws.iter().enumerate() {
            commands.push(RenderCommand::DrawMesh(DrawMeshCommand {
                mesh: MeshHandle(OPAQUE_MESH_BASE + index as u64),
                material: draw.material,
                pipeline: opaque_pipeline,
                instance: Instance2d::identity(),
                camera: Some(WORKING_CAMERA),
                viewport: None,
            }));
        }
        for (index, draw) in cutout_draws.iter().enumerate() {
            commands.push(RenderCommand::DrawMesh(DrawMeshCommand {
                mesh: MeshHandle(cutout_mesh_base + index as u64),
                material: draw.material,
                pipeline: cutout_pipeline,
                instance: Instance2d::identity(),
                camera: Some(WORKING_CAMERA),
                viewport: None,
            }));
        }
        if commands.len() != expected_commands {
            return Err(format!(
                "browser working-model command accounting mismatch: expected={expected_commands}; actual={}",
                commands.len()
            ));
        }
        renderer.begin_frame();
        renderer.submit(&commands);
        renderer.present().map_err(|error| error.to_string())?;
        if let Some(record) = renderer.drain_diagnostics().into_iter().next() {
            return Err(format!(
                "working-model initial WebGPU diagnostic: category={:?}; source={}; message={}",
                record.kind, record.source, record.message
            ));
        }

        self.working_lifetime.replacements_presented = self
            .working_lifetime
            .replacements_presented
            .saturating_add(1);

        let semantic_correlation = previous_semantic_inventory
            .map(|previous| {
                correlate_scene_resource_inventories(previous, current_semantic_inventory.clone())
            })
            .transpose()
            .map_err(|error| format!("Alternative-C inventory correlation failed: {error:?}"))?;

        self.working_model = Some(BrowserWorkingModel {
            renderer,
            commands,
            logical_resources,
            semantic_inventory: current_semantic_inventory,
            position: observer,
            yaw,
            pitch,
            width,
            height,
            far_plane,
        });

        let lifetime_alternative = if retain_provider_session {
            "adapter-private-scene-reset"
        } else {
            "whole-backend-replacement"
        };
        Ok(format!(
            "browser working-model frame presented: map={}; strategy=global-full-plus-grouped-sky-parity; stages=sky-panorama>full-world-depth-prepass>paired-skywall-and-source-sky-plane-stencil-inversion>even-parity-world-color; sector-boundary-trim=true; opaque={}; cutouts={}; skywalls={}; sky-planes={}; surface-triangles={}; edge-conformance-insertions={}; camera=source-spawn; embedding=preserve-north; backend={backend_api}; device={device_kind}; adapter={adapter_name}; canvas={}x{}; lifetime-alternative={}; replacement-attempts={}; replacements-presented={}; backend-creations={}; device-creations={}; surface-creations={}; scene-resets={}; reset-observation={:?}; current-logical-resources=[meshes:{},textures:{},materials:{},pipelines:{},cameras:{},commands:{}]; current-logical-uploads=[meshes:{},textures:{},materials:{},pipelines:{},cameras:{}]; current-same-handle-replacements=[meshes:0,textures:0,materials:0,pipelines:0,cameras:0]; current-estimated-bytes=[mesh-vertices:{},source-texture-payloads:{}]; retired-logical-sets={}; retired-logical-resources=[meshes:{},textures:{},materials:{},pipelines:{},cameras:{},commands:{}]; retired-estimated-bytes=[mesh-vertices:{},source-texture-payloads:{}]; retained-provider-session={}; alternative-c-inventory-correlation={semantic_correlation:?}; alternative-c-authority=semantic-shadow-not-provider-lifetime; physical-gpu-reclamation=unobserved",
            map.map_name,
            draws.len(),
            cutout_draws.len(),
            skywall_meshes.len(),
            sky_plane_meshes.len(),
            surface_bake.audit.surface_triangles,
            surface_bake.audit.edge_conformance_insertions,
            width,
            height,
            lifetime_alternative,
            self.working_lifetime.replacement_attempts,
            self.working_lifetime.replacements_presented,
            self.working_lifetime.backend_creations,
            self.working_lifetime.device_creations,
            self.working_lifetime.surface_creations,
            self.working_lifetime.scene_resets,
            reset_observation,
            logical_resources.meshes,
            logical_resources.textures,
            logical_resources.materials,
            logical_resources.pipelines,
            logical_resources.cameras,
            logical_resources.commands,
            logical_resources.meshes,
            logical_resources.textures,
            logical_resources.materials,
            logical_resources.pipelines,
            logical_resources.cameras,
            logical_resources.mesh_vertex_bytes,
            logical_resources.source_texture_payload_bytes,
            self.working_lifetime.retired_sets,
            self.working_lifetime.retired_resources.meshes,
            self.working_lifetime.retired_resources.textures,
            self.working_lifetime.retired_resources.materials,
            self.working_lifetime.retired_resources.pipelines,
            self.working_lifetime.retired_resources.cameras,
            self.working_lifetime.retired_resources.commands,
            self.working_lifetime.retired_resources.mesh_vertex_bytes,
            self.working_lifetime
                .retired_resources
                .source_texture_payload_bytes,
            retain_provider_session,
        ))
    }

    #[cfg(target_arch = "wasm32")]
    #[allow(clippy::too_many_arguments)]
    fn step_working_model_inner(
        &mut self,
        delta_seconds: f32,
        forward_axis: f32,
        strafe_axis: f32,
        vertical_axis: f32,
        yaw_delta: f32,
        pitch_delta: f32,
        running: bool,
    ) -> Result<(), String> {
        let values = [
            delta_seconds,
            forward_axis,
            strafe_axis,
            vertical_axis,
            yaw_delta,
            pitch_delta,
        ];
        if values.iter().any(|value| !value.is_finite()) {
            return Err("working-model input contains a non-finite value".into());
        }
        let model = self
            .working_model
            .as_mut()
            .ok_or_else(|| "no browser working-model scene is retained".to_owned())?;
        if let Some(record) = model.renderer.drain_diagnostics().into_iter().next() {
            return Err(format!(
                "working-model retained WebGPU diagnostic: category={:?}; source={}; message={}",
                record.kind, record.source, record.message
            ));
        }
        let delta_seconds = delta_seconds.clamp(0.0, 0.25);
        model.yaw += yaw_delta.clamp(-0.5, 0.5);
        model.pitch = (model.pitch + pitch_delta.clamp(-0.5, 0.5)).clamp(-1.5, 1.5);

        let forward = observer_direction(model.yaw, 0.0);
        let right = observer_right(forward);
        let mut movement = forward * forward_axis.clamp(-1.0, 1.0)
            + right * strafe_axis.clamp(-1.0, 1.0)
            + Vec3::Y * vertical_axis.clamp(-1.0, 1.0);
        if movement.length_squared() > 1.0 {
            movement = movement.normalize();
        }
        let speed = if running { 480.0 } else { 240.0 };
        model.position += movement * speed * delta_seconds;

        let mut camera = Camera::perspective_3d(model.width as f32, model.height as f32);
        camera.projection = tokimu_core::math::try_projection_perspective_rh_gl(
            60_f32.to_radians(),
            model.width as f32 / model.height as f32,
            0.1,
            model.far_plane,
        )
        .ok_or_else(|| "working-model camera projection is invalid".to_owned())?;
        camera.view = tokimu_core::math::try_view_look_at_rh(
            model.position,
            model.position + observer_direction(model.yaw, model.pitch) * 128.0,
            Vec3::Y,
        )
        .ok_or_else(|| "working-model inspection camera basis is invalid".to_owned())?;
        model.renderer.upload_camera(WORKING_CAMERA, camera);
        model.renderer.begin_frame();
        model.renderer.submit(&model.commands);
        model
            .renderer
            .present()
            .map(|_| ())
            .map_err(|error| error.to_string())?;
        if let Some(record) = model.renderer.drain_diagnostics().into_iter().next() {
            return Err(format!(
                "working-model WebGPU diagnostic: category={:?}; source={}; message={}",
                record.kind, record.source, record.message
            ));
        }
        Ok(())
    }

    #[cfg(target_arch = "wasm32")]
    async fn render_static_e1m1_inner(
        &self,
        canvas: HtmlCanvasElement,
        include_masked_cutouts: bool,
        select_by_frustum: bool,
        focus_exitsign: bool,
        include_diagnostic_sky: bool,
    ) -> Result<String, String> {
        // Browser evidence uses the same explicit, orientation-preserving Doom
        // adapter as the native corpus default. This remains a Doom-consumer
        // convention; it does not establish Tokimu's global cardinal axes.
        let embedding = DoomComparativeEmbedding::PreserveNorth;
        let name = ResourceName::parse("selected-doom-package", AddressCasePolicy::Sensitive)
            .map_err(|error| error.to_string())?;
        let _selected_package = self
            .space
            .resource(self.folder, &name)
            .map_err(|error| error.to_string())?
            .ok_or_else(|| "no selected package is retained".to_owned())?;
        let read = read_wad_package_member(
            &self.space,
            InspectWadPackageRequest {
                archive: InspectArchiveResourceRequest {
                    source_folder: self.folder,
                    source_name: name,
                    format: ArchiveFormat::Zip,
                    limits: ArchiveReadLimits::new(
                        64 * 1024 * 1024,
                        2048,
                        16 * 1024 * 1024,
                        64 * 1024 * 1024,
                        4096,
                    ),
                },
                member_name: "DOOM1.WAD".into(),
                wad_source_label: "browser-selected:DOOM1.WAD".into(),
                wad_limits: WAD_LIMITS,
            },
            &ZipArchiveProvider,
        )
        .map_err(|error| error.to_string())?;
        let source_spawn = if select_by_frustum {
            let selection = select_doom_episode_map(&read.observation.wad, "E1M1")
                .map_err(|error| error.to_string())?;
            let map = decode_doom_map_core(&read.bytes, &selection, MAP_LIMITS)
                .map_err(|error| error.to_string())?;
            let start =
                resolve_doom_player_one_start(&map.things).map_err(|error| error.to_string())?;
            let paths =
                resolve_doom_subsector_bsp_paths(&map).map_err(|error| error.to_string())?;
            let location = locate_doom_point_subsector(start.position, &paths)
                .map_err(|error| error.to_string())?;
            let ownership =
                resolve_doom_subsector_sector_ownership(&map).map_err(|error| error.to_string())?;
            let sector = ownership
                .iter()
                .find(|entry| entry.source_subsector == location.source_subsector)
                .ok_or_else(|| "player-one start subsector has no sector ownership".to_owned())?;
            let vertical = &map.sectors[usize::from(sector.sector_index)];
            Some((
                embedding.lift_direction(
                    start.position.map(f32::from),
                    (f32::from(vertical.floor_height) + f32::from(vertical.ceiling_height)) * 0.5,
                ),
                embedding.lift_heading_degrees(f32::from(start.angle)),
            ))
        } else {
            None
        };
        let flats = prepare_e1m1_flats(&read.bytes, &read.observation.wad, MAP_LIMITS)
            .map_err(|error| error.to_string())?;
        let walls = prepare_e1m1_walls(
            &read.bytes,
            &read.observation.wad,
            MAP_LIMITS,
            TEXTURE_LIMITS,
        )
        .map_err(|error| error.to_string())?;
        let flat_textures = prepare_e1m1_flat_textures(
            &read.bytes,
            &read.observation.wad,
            &flats,
            RASTER_LIMITS,
            FLAT_LIMITS,
        )
        .map_err(|error| error.to_string())?;
        let names = hello_doom_e1m1::prepared_e1m1_wall_texture_names(&walls);
        let wall_textures = prepare_e1m1_wall_textures(
            &read.bytes,
            &read.observation.wad,
            &names,
            RASTER_LIMITS,
            TEXTURE_LIMITS,
            PATCH_LIMITS,
            COMPOSE_LIMITS,
        )
        .map_err(|error| error.to_string())?;
        let uploads = build_static_texture_uploads(&flat_textures, &wall_textures);
        let mut draws =
            build_static_draw_plan(&flats, &walls, &uploads).map_err(|error| error.to_string())?;
        reembed_browser_draws(&mut draws, embedding);
        let mut diagnostic_sky_draws = if include_diagnostic_sky {
            prepare_e1m1_sky_diagnostic_flats(&read.bytes, &read.observation.wad, MAP_LIMITS)
                .map_err(|error| error.to_string())?
        } else {
            Vec::new()
        };
        for flat in &mut diagnostic_sky_draws {
            reembed_comparative_mesh(&mut flat.mesh, embedding, false);
            for uv in &mut flat.mesh.texture_coordinates {
                uv[0] = -uv[0];
            }
        }
        let exitsign_view = focus_exitsign
            .then(|| exitsign_camera(&draws))
            .transpose()?;
        let (cutout_uploads, mut cutout_draws) = if include_masked_cutouts {
            let masked = prepare_e1m1_masked_middle_cutouts(
                &read.bytes,
                &read.observation.wad,
                MAP_LIMITS,
                TEXTURE_LIMITS,
            )
            .map_err(|error| error.to_string())?;
            let masked_names = prepared_e1m1_masked_middle_texture_names(&walls);
            let masked_textures = prepare_e1m1_wall_textures(
                &read.bytes,
                &read.observation.wad,
                &masked_names,
                RASTER_LIMITS,
                TEXTURE_LIMITS,
                PATCH_LIMITS,
                COMPOSE_LIMITS,
            )
            .map_err(|error| error.to_string())?;
            let uploads = build_experimental_cutout_texture_uploads(
                &masked_textures,
                uploads.len() as u64 + 1,
            );
            let draws = build_experimental_cutout_draw_plan(&masked, &uploads)
                .map_err(|error| error.to_string())?;
            (uploads, draws)
        } else {
            (Vec::new(), Vec::new())
        };
        reembed_browser_draws(&mut cutout_draws, embedding);
        let width = canvas.width().max(1);
        let height = canvas.height().max(1);
        let mut renderer = WgpuBackend::for_window(canvas, width, height)
            .await
            .map_err(|error| error.to_string())?;
        let adapter_name = renderer.adapter_name().to_owned();
        let backend_api = renderer.backend_api();
        let device_kind = renderer.device_kind();
        for upload in &uploads {
            renderer
                .create_texture_rgba8(upload.texture, upload.descriptor, &upload.rgba8)
                .map_err(|error| error.to_string())?;
            renderer
                .upload_material(upload.material, &upload.material_value)
                .map_err(|error| error.to_string())?;
        }
        if include_masked_cutouts {
            for upload in &cutout_uploads {
                renderer
                    .create_texture_rgba8(upload.texture, upload.descriptor, &upload.rgba8)
                    .map_err(|error| error.to_string())?;
                renderer
                    .upload_material(upload.material, &upload.material_value)
                    .map_err(|error| error.to_string())?;
            }
        }
        if include_diagnostic_sky {
            let decoded = decode_png(
                include_bytes!("../../../../assets/PNG/Purple/texture_01.png"),
                DecodeLimits::default(),
            )
            .map_err(|error| error.to_string())?;
            let prepared = prepare_renderer_texture(&decoded, TextureUse::ColorSrgb)
                .map_err(|error| error.to_string())?;
            renderer
                .create_texture_rgba8(
                    DIAGNOSTIC_SKY_TEXTURE,
                    Rgba8TextureDescriptor::new(
                        prepared.texture.width,
                        prepared.texture.height,
                        Rgba8TextureColorSpace::Srgb,
                    ),
                    &prepared.texture.rgba8,
                )
                .map_err(|error| error.to_string())?;
            renderer
                .upload_material(
                    DIAGNOSTIC_SKY_MATERIAL,
                    &Material::new(
                        "e1m1-browser-diagnostic-sky-omission",
                        Color::rgb(1.0, 1.0, 1.0),
                    )
                    .with_texture(DIAGNOSTIC_SKY_TEXTURE)
                    .with_texture_sampler(TextureSampler {
                        filter: TextureFilter::Point,
                        address_u: TextureAddressMode::Repeat,
                        address_v: TextureAddressMode::Repeat,
                    }),
                )
                .map_err(|error| error.to_string())?;
        }
        let pipeline = renderer
            .register_pipeline(
                &Pipeline::new("doom-e1m1-browser-opaque", PipelineKind::Textured3d)
                    .with_render_state(PipelineRenderState {
                        blend: BlendMode::Opaque,
                        depth_test: DepthTest::LessEqual,
                        depth_write: true,
                        cull_mode: CullMode::Back,
                        color_write: ColorWriteMask::ALL,
                    })
                    .map_err(|error| error.to_string())?,
            )
            .map_err(|error| error.to_string())?;
        let cutout_pipeline = if include_masked_cutouts {
            Some(
                renderer
                    .register_pipeline(&Pipeline::textured_3d_cutout(
                        "doom-e1m1-browser-masked-cutout",
                        CategoricalCutout::new(
                            CutoutThreshold::new(0.0).map_err(|error| error.to_string())?,
                            CutoutComparison::DiscardAtOrBelow,
                        ),
                    ))
                    .map_err(|error| error.to_string())?,
            )
        } else {
            None
        };
        let mut minimum = [f32::INFINITY; 3];
        let mut maximum = [f32::NEG_INFINITY; 3];
        for draw in draws.iter().chain(cutout_draws.iter()) {
            for position in &draw.mesh.positions {
                for axis in 0..3 {
                    minimum[axis] = minimum[axis].min(position[axis]);
                    maximum[axis] = maximum[axis].max(position[axis]);
                }
            }
        }
        for draw in &diagnostic_sky_draws {
            for position in &draw.mesh.positions {
                for axis in 0..3 {
                    minimum[axis] = minimum[axis].min(position[axis]);
                    maximum[axis] = maximum[axis].max(position[axis]);
                }
            }
        }
        let center = Vec3::new(
            (minimum[0] + maximum[0]) * 0.5,
            (minimum[1] + maximum[1]) * 0.5,
            (minimum[2] + maximum[2]) * 0.5,
        );
        let radius = (maximum[0] - minimum[0])
            .max(maximum[1] - minimum[1])
            .max(maximum[2] - minimum[2])
            .max(1.0);
        let mut camera = Camera::perspective_3d(width as f32, height as f32);
        camera.projection = tokimu_core::math::try_projection_perspective_rh_gl(
            60_f32.to_radians(),
            width as f32 / height as f32,
            (radius * 0.0001).max(0.1),
            radius * 4.0,
        )
        .expect("perspective parameters must be finite and ordered");
        camera.view = if let Some((position, target)) = exitsign_view {
            tokimu_core::math::try_view_look_at_rh(position, target, Vec3::Y)
                .expect("camera basis must be finite and non-degenerate")
        } else if let Some((position, source_forward)) = source_spawn {
            let yaw = observer_yaw_from_forward(source_forward) + std::f32::consts::FRAC_PI_2;
            tokimu_core::math::try_view_look_at_rh(
                position,
                position + observer_direction(yaw, 0.0) * 128.0,
                Vec3::Y,
            )
            .expect("camera basis must be finite and non-degenerate")
        } else {
            tokimu_core::math::try_view_look_at_rh(
                center + Vec3::new(radius, radius * 0.72, radius),
                center,
                Vec3::Y,
            )
            .expect("camera basis must be finite and non-degenerate")
        };
        renderer.upload_camera(CameraHandle(1), camera);
        renderer.begin_frame();
        let mut commands = vec![RenderCommand::Clear(ClearCommand {
            color: Color::rgb(0.015, 0.02, 0.025),
        })];
        let view_projection = camera.projection * camera.view;
        let opaque_bounds = draws
            .iter()
            .map(|draw| StaticDrawAabb::from_positions(&draw.mesh.positions))
            .collect::<Vec<_>>();
        let cutout_bounds = cutout_draws
            .iter()
            .map(|draw| StaticDrawAabb::from_positions(&draw.mesh.positions))
            .collect::<Vec<_>>();
        let mut opaque_rejected = 0_usize;
        let mut cutout_rejected = 0_usize;
        for (index, draw) in draws.iter().enumerate() {
            let mesh = MeshHandle(index as u64 + 1);
            renderer.upload_mesh(mesh, &draw.mesh);
            let selected = !select_by_frustum
                || opaque_bounds[index]
                    .and_then(|bounds| {
                        classify_static_draw_frustum_rejection(bounds, view_projection)
                    })
                    .is_none();
            if !selected {
                opaque_rejected += 1;
                continue;
            }
            commands.push(RenderCommand::DrawMesh(DrawMeshCommand {
                mesh,
                material: draw.material,
                pipeline,
                instance: Instance2d::identity(),
                camera: Some(CameraHandle(1)),
                viewport: None,
            }));
        }
        if include_masked_cutouts {
            let cutout_pipeline = cutout_pipeline
                .ok_or_else(|| "masked-cutout pipeline was not prepared".to_owned())?;
            for (offset, draw) in cutout_draws.iter().enumerate() {
                let mesh = MeshHandle(draws.len() as u64 + offset as u64 + 1);
                renderer.upload_mesh(mesh, &draw.mesh);
                let selected = !select_by_frustum
                    || cutout_bounds[offset]
                        .and_then(|bounds| {
                            classify_static_draw_frustum_rejection(bounds, view_projection)
                        })
                        .is_none();
                if !selected {
                    cutout_rejected += 1;
                    continue;
                }
                commands.push(RenderCommand::DrawMesh(DrawMeshCommand {
                    mesh,
                    material: draw.material,
                    pipeline: cutout_pipeline,
                    instance: Instance2d::identity(),
                    camera: Some(CameraHandle(1)),
                    viewport: None,
                }));
            }
        }
        if include_diagnostic_sky {
            for (offset, draw) in diagnostic_sky_draws.iter().enumerate() {
                let mesh =
                    MeshHandle(draws.len() as u64 + cutout_draws.len() as u64 + offset as u64 + 1);
                renderer.upload_mesh(mesh, &draw.mesh);
                commands.push(RenderCommand::DrawMesh(DrawMeshCommand {
                    mesh,
                    material: DIAGNOSTIC_SKY_MATERIAL,
                    pipeline,
                    instance: Instance2d::identity(),
                    camera: Some(CameraHandle(1)),
                    viewport: None,
                }));
            }
        }
        renderer.submit(&commands);
        renderer.present().map_err(|error| error.to_string())?;
        let opaque_submitted = draws.len() - opaque_rejected;
        let cutout_submitted = if include_masked_cutouts {
            cutout_draws.len() - cutout_rejected
        } else {
            0
        };
        let diagnostic_submitted = diagnostic_sky_draws.len();
        let draw_count = opaque_submitted + cutout_submitted + diagnostic_submitted;
        Ok(format!(
            "browser first frame presented: {draw_count} draws; candidates={}; rejected={}; opaque={opaque_submitted}/{}; cutouts={cutout_submitted}/{}; diagnostic_sky={diagnostic_submitted}/{}; diagnostic_asset={}; diagnostic_reason={}; frustum_aabb={select_by_frustum}; camera={}; embedding=preserve-north; backend={backend_api}; device={device_kind}; adapter={adapter_name}; canvas={}x{}",
            draws.len() + if include_masked_cutouts { cutout_draws.len() } else { 0 } + diagnostic_sky_draws.len(),
            opaque_rejected + cutout_rejected,
            draws.len(),
            if include_masked_cutouts { cutout_draws.len() } else { 0 },
            diagnostic_sky_draws.len(),
            if include_diagnostic_sky { "corpus/assets/PNG/Purple/texture_01.png" } else { "none" },
            if include_diagnostic_sky { "intentional-source-sky-omission" } else { "none" },
            if focus_exitsign { "canonical-exitsign" } else if select_by_frustum { "source-spawn-plus-90" } else { "overview" },
            width, height
        ))
    }
}

#[cfg(target_arch = "wasm32")]
fn reembed_browser_draws(draws: &mut [StaticDrawPlanEntry], embedding: DoomComparativeEmbedding) {
    for draw in draws {
        let is_wall = matches!(draw.source, StaticDrawSource::Wall { .. });
        reembed_comparative_mesh(&mut draw.mesh, embedding, is_wall);
        if matches!(draw.source, StaticDrawSource::Flat { .. }) {
            // Flat U is a continuous source-spatial field. Reverse it about
            // the source origin, matching the native corpus adapter.
            for uv in &mut draw.mesh.texture_coordinates {
                uv[0] = -uv[0];
            }
        }
    }
}

#[cfg(target_arch = "wasm32")]
fn is_working_one_sided_wall(
    draw: &StaticDrawPlanEntry,
    map: &doom_map_provider::DoomMapCore,
) -> bool {
    let StaticDrawSource::Wall { source_linedef, .. } = draw.source else {
        return false;
    };
    map.linedefs
        .get(source_linedef.record_index as usize)
        .filter(|linedef| linedef.source == source_linedef)
        .is_some_and(|linedef| linedef.right_sidedef.is_some() ^ linedef.left_sidedef.is_some())
}

#[cfg(target_arch = "wasm32")]
fn working_mesh_vertex_bytes(
    draws: &[StaticDrawPlanEntry],
    cutout_draws: &[StaticDrawPlanEntry],
    skywall_meshes: &[Mesh],
    sky_plane_meshes: &[Mesh],
    sky_mesh: &Mesh,
) -> u64 {
    // The current WGPU lowering stores position (3xf32), normal (3xf32), and
    // texture coordinate (2xf32) for every vertex. This deliberately excludes
    // buffer alignment, allocator/driver overhead, staging, and residency.
    const GPU_VERTEX_BYTES: u64 = 8 * std::mem::size_of::<f32>() as u64;
    draws
        .iter()
        .chain(cutout_draws)
        .map(|draw| draw.mesh.positions.len() as u64)
        .chain(
            skywall_meshes
                .iter()
                .chain(sky_plane_meshes)
                .map(|mesh| mesh.positions.len() as u64),
        )
        .chain(std::iter::once(sky_mesh.positions.len() as u64))
        .fold(0_u64, u64::saturating_add)
        .saturating_mul(GPU_VERTEX_BYTES)
}

#[cfg(target_arch = "wasm32")]
fn working_scene_bounds(
    opaque: &[StaticDrawPlanEntry],
    cutouts: &[StaticDrawPlanEntry],
) -> Result<(Vec3, f32), String> {
    let mut minimum = Vec3::splat(f32::INFINITY);
    let mut maximum = Vec3::splat(f32::NEG_INFINITY);
    for position in opaque
        .iter()
        .chain(cutouts)
        .flat_map(|draw| draw.mesh.positions.iter())
    {
        let point = Vec3::from_array(*position);
        minimum = minimum.min(point);
        maximum = maximum.max(point);
    }
    if !minimum.is_finite() || !maximum.is_finite() {
        return Err("working-model map has no finite ordinary geometry".into());
    }
    let center = (minimum + maximum) * 0.5;
    let radius = (maximum - minimum).max_element().max(1.0);
    Ok((center, radius))
}

#[cfg(target_arch = "wasm32")]
fn build_working_sky_cylinder(
    center: Vec3,
    scene_radius: f32,
) -> Result<Mesh, tokimu::MeshValidationError> {
    const SEGMENTS: usize = 64;
    let radius = scene_radius * 1.5;
    let bottom = center.y - scene_radius * 3.0;
    let top = center.y + scene_radius * 3.0;
    let mut positions = Vec::with_capacity(SEGMENTS * 6);
    let mut normals = Vec::with_capacity(SEGMENTS * 6);
    let mut texture_coordinates = Vec::with_capacity(SEGMENTS * 6);
    for segment in 0..SEGMENTS {
        let u0 = segment as f32 / SEGMENTS as f32;
        let u1 = (segment + 1) as f32 / SEGMENTS as f32;
        let angle0 = u0 * std::f32::consts::TAU;
        let angle1 = u1 * std::f32::consts::TAU;
        let radial0 = Vec3::new(angle0.cos(), 0.0, angle0.sin());
        let radial1 = Vec3::new(angle1.cos(), 0.0, angle1.sin());
        let p0_bottom = center + radial0 * radius + Vec3::Y * (bottom - center.y);
        let p0_top = center + radial0 * radius + Vec3::Y * (top - center.y);
        let p1_bottom = center + radial1 * radius + Vec3::Y * (bottom - center.y);
        let p1_top = center + radial1 * radius + Vec3::Y * (top - center.y);
        for (position, normal, uv) in [
            (p0_bottom, -radial0, [u0, 1.0]),
            (p1_top, -radial1, [u1, 0.0]),
            (p1_bottom, -radial1, [u1, 1.0]),
            (p0_bottom, -radial0, [u0, 1.0]),
            (p0_top, -radial0, [u0, 0.0]),
            (p1_top, -radial1, [u1, 0.0]),
        ] {
            positions.push(position.to_array());
            normals.push(normal.to_array());
            texture_coordinates.push(uv);
        }
    }
    Mesh::new(positions, normals).with_texture_coordinates(texture_coordinates)
}

#[cfg(target_arch = "wasm32")]
fn exitsign_camera(draws: &[hello_doom_e1m1::StaticDrawPlanEntry]) -> Result<(Vec3, Vec3), String> {
    let exitsign = draws
        .iter()
        .filter(|draw| {
            matches!(
                draw.source,
                StaticDrawSource::Wall {
                    source_linedef, ..
                }
                    if source_linedef.record_index == 342
            ) && draw.source_label.contains("EXITSIGN")
        })
        .collect::<Vec<_>>();
    if exitsign.is_empty() {
        return Err("canonical E1M1 EXITSIGN draws are absent".to_owned());
    }

    let mut position_sum = Vec3::ZERO;
    let mut normal_sum = Vec3::ZERO;
    let mut position_count = 0_usize;
    for draw in exitsign {
        for position in &draw.mesh.positions {
            position_sum += Vec3::from_array(*position);
            position_count += 1;
        }
        for normal in &draw.mesh.normals {
            normal_sum += Vec3::from_array(*normal);
        }
    }
    let target = position_sum / position_count as f32;
    let owning_side_normal = normal_sum.normalize_or_zero();
    if owning_side_normal == Vec3::ZERO {
        return Err(
            "canonical E1M1 EXITSIGN linedef 342 has no stable owning-side normal".to_owned(),
        );
    }
    Ok((target + owning_side_normal * 96.0, target))
}

fn js_error(message: String) -> JsValue {
    JsValue::from_str(&message)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn retains_one_bounded_selection_and_replaces_it() {
        let mut session = BrowserIntakeSession::new_inner().unwrap();
        let first = session
            .import_selected_package_inner("first.zip", "application/zip", b"first")
            .unwrap();
        assert!(first.contains("\"retainedBytes\":5"));
        let second = session
            .import_selected_package_inner("second.zip", "application/zip", b"next")
            .unwrap();
        assert!(second.contains("\"retainedResources\":1"));
        assert!(second.contains("\"retainedBytes\":4"));
    }
    #[test]
    fn rejects_empty_bytes_without_retaining_them() {
        let mut session = BrowserIntakeSession::new_inner().unwrap();
        assert_eq!(
            session.import_selected_package_inner("empty.zip", "application/zip", b""),
            Err("selected package is empty".into())
        );
        assert_eq!(session.space.summary().resources(), 0);
    }

    #[test]
    fn rejects_an_empty_label_and_disposal_releases_retained_bytes() {
        let mut session = BrowserIntakeSession::new_inner().unwrap();
        assert_eq!(
            session.import_selected_package_inner("", "application/zip", b"bytes"),
            Err("selected package has an empty source label".into())
        );
        session
            .import_selected_package_inner("selected.zip", "application/zip", b"bytes")
            .unwrap();
        session.dispose().unwrap();
        assert_eq!(session.space.summary().resources(), 0);
        assert_eq!(session.space.summary().retained_bytes(), 0);
    }

    #[test]
    fn rejects_an_oversized_selection_without_retaining_it() {
        let mut session = BrowserIntakeSession::new_inner().unwrap();
        let oversized = vec![0; MAX_INPUT_BYTES + 1];
        let result =
            session.import_selected_package_inner("large.zip", "application/zip", &oversized);
        assert!(result.unwrap_err().contains("exceeding the limit"));
        assert_eq!(session.space.summary().resources(), 0);
    }

    #[test]
    fn browser_working_model_budget_accepts_limits_and_names_each_rejection() {
        let at_limits = WorkingLogicalResources {
            meshes: MAX_BROWSER_WORKING_MESHES,
            textures: MAX_BROWSER_WORKING_TEXTURES,
            materials: MAX_BROWSER_WORKING_MATERIALS,
            pipelines: MAX_BROWSER_WORKING_PIPELINES,
            cameras: MAX_BROWSER_WORKING_CAMERAS,
            commands: MAX_BROWSER_WORKING_COMMANDS,
            mesh_vertex_bytes: MAX_BROWSER_WORKING_MESH_VERTEX_BYTES,
            source_texture_payload_bytes: MAX_BROWSER_WORKING_TEXTURE_PAYLOAD_BYTES,
        };
        validate_browser_working_model_budget(at_limits).unwrap();

        let overages = [
            (
                "meshes",
                WorkingLogicalResources {
                    meshes: MAX_BROWSER_WORKING_MESHES + 1,
                    ..WorkingLogicalResources::default()
                },
            ),
            (
                "textures",
                WorkingLogicalResources {
                    textures: MAX_BROWSER_WORKING_TEXTURES + 1,
                    ..WorkingLogicalResources::default()
                },
            ),
            (
                "materials",
                WorkingLogicalResources {
                    materials: MAX_BROWSER_WORKING_MATERIALS + 1,
                    ..WorkingLogicalResources::default()
                },
            ),
            (
                "pipelines",
                WorkingLogicalResources {
                    pipelines: MAX_BROWSER_WORKING_PIPELINES + 1,
                    ..WorkingLogicalResources::default()
                },
            ),
            (
                "cameras",
                WorkingLogicalResources {
                    cameras: MAX_BROWSER_WORKING_CAMERAS + 1,
                    ..WorkingLogicalResources::default()
                },
            ),
            (
                "commands",
                WorkingLogicalResources {
                    commands: MAX_BROWSER_WORKING_COMMANDS + 1,
                    ..WorkingLogicalResources::default()
                },
            ),
            (
                "mesh-vertex-bytes",
                WorkingLogicalResources {
                    mesh_vertex_bytes: MAX_BROWSER_WORKING_MESH_VERTEX_BYTES + 1,
                    ..WorkingLogicalResources::default()
                },
            ),
            (
                "source-texture-payload-bytes",
                WorkingLogicalResources {
                    source_texture_payload_bytes: MAX_BROWSER_WORKING_TEXTURE_PAYLOAD_BYTES + 1,
                    ..WorkingLogicalResources::default()
                },
            ),
        ];
        for (resource, overage) in overages {
            let error = validate_browser_working_model_budget(overage).unwrap_err();
            assert!(error.contains(&format!("resource={resource}")));
        }
    }

    #[test]
    fn semantic_inventory_preserves_every_measured_working_resource_family() {
        let resources = WorkingLogicalResources {
            meshes: 1_921,
            textures: 83,
            materials: 85,
            pipelines: 7,
            cameras: 1,
            commands: 3_844,
            mesh_vertex_bytes: 123_456,
            source_texture_payload_bytes: 654_321,
        };
        let inventory =
            working_semantic_inventory("DOOM E1M2 prepared inventory".into(), resources);
        assert_eq!(inventory.source_label, "DOOM E1M2 prepared inventory");
        assert_eq!(inventory.meshes, resources.meshes);
        assert_eq!(inventory.textures, resources.textures);
        assert_eq!(inventory.materials, resources.materials);
        assert_eq!(inventory.pipelines, resources.pipelines);
        assert_eq!(inventory.cameras, resources.cameras);
        assert_eq!(inventory.commands, resources.commands);
    }
}
