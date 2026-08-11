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
    locate_doom_point_subsector, resolve_doom_subsector_bsp_paths,
    resolve_doom_subsector_sector_ownership,
};
#[cfg(target_arch = "wasm32")]
use doom_map_provider::{decode_doom_map_core, resolve_doom_player_one_start};
#[cfg(target_arch = "wasm32")]
use doom_wad_package::select_doom_episode_map;
#[cfg(target_arch = "wasm32")]
use hello_doom_e1m1::{
    build_experimental_cutout_draw_plan, build_experimental_cutout_texture_uploads,
    build_static_draw_plan, build_static_texture_uploads, classify_static_draw_frustum_rejection,
    doom_heading_forward, observer_direction, observer_yaw_from_forward,
    prepare_e1m1_flat_textures, prepare_e1m1_flats, prepare_e1m1_masked_middle_cutouts,
    prepare_e1m1_wall_textures, prepare_e1m1_walls, prepared_e1m1_masked_middle_texture_names,
    StaticDrawAabb, StaticDrawSource,
};
#[cfg(target_arch = "wasm32")]
use tokimu::{
    BlendMode, Camera, CameraHandle, CategoricalCutout, ClearCommand, Color, ColorWriteMask,
    CullMode, CutoutComparison, CutoutThreshold, DepthTest, DrawMeshCommand, Instance2d,
    MeshHandle, Pipeline, PipelineKind, PipelineRenderState, RenderCommand, Renderer, WgpuBackend,
};
#[cfg(target_arch = "wasm32")]
use tokimu_core::math::{Mat4, Vec3};
#[cfg(target_arch = "wasm32")]
use web_sys::HtmlCanvasElement;

pub const SCHEMA_VERSION: u32 = 1;
pub const MAX_INPUT_BYTES: usize = 64 * 1024 * 1024;

const WAD_LIMITS: WadReadLimits =
    WadReadLimits::new(64 * 1024 * 1024, 8_192, 16 * 1024 * 1024, 64 * 1024 * 1024);
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
const RASTER_LIMITS: doom_raster_provider::DoomRasterDecodeLimits =
    doom_raster_provider::DoomRasterDecodeLimits {
        max_playpal_bytes: 64 * 1024 * 1024,
        max_palettes: 4096,
        max_colormap_bytes: 64 * 1024 * 1024,
        max_colormaps: 4096,
        max_total_decoded_bytes: 128 * 1024 * 1024,
    };
const FLAT_LIMITS: doom_raster_provider::DoomFlatDecodeLimits =
    doom_raster_provider::DoomFlatDecodeLimits {
        max_flat_bytes: 4096,
    };
const TEXTURE_LIMITS: doom_raster_provider::DoomTextureDecodeLimits =
    doom_raster_provider::DoomTextureDecodeLimits {
        max_pnames_bytes: 64 * 1024 * 1024,
        max_texture_bytes: 64 * 1024 * 1024,
        max_patch_names: 1_000_000,
        max_textures: 1_000_000,
        max_patches_per_texture: 16_384,
        max_total_patch_references: 10_000_000,
    };
const PATCH_LIMITS: doom_raster_provider::DoomPatchDecodeLimits =
    doom_raster_provider::DoomPatchDecodeLimits {
        max_patch_bytes: 64 * 1024 * 1024,
        max_width: 4096,
        max_height: 4096,
        max_pixels: 16 * 1024 * 1024,
        max_posts: 16 * 1024 * 1024,
    };
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

/// One transient Rust-owned selection session. It exposes no browser path,
/// directory, fetch, storage, or Doom semantic API.
#[wasm_bindgen]
pub struct BrowserIntakeSession {
    space: InMemoryResourceSpace,
    folder: FolderId,
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
        self.render_static_e1m1_inner(canvas, false, false, false)
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
        self.render_static_e1m1_inner(canvas, true, false, false)
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
        self.render_static_e1m1_inner(canvas, true, true, false)
            .await
            .map_err(js_error)
    }

    /// Presents the canonical E1M1 EXITSIGN walls from their owning side.
    /// This fixed corpus camera exists only for AR-0028 browser orientation
    /// evidence; neither the renderer nor TypeScript interprets Doom sides.
    #[cfg(target_arch = "wasm32")]
    pub async fn render_e1m1_exitsign(&self, canvas: HtmlCanvasElement) -> Result<String, JsValue> {
        self.render_static_e1m1_inner(canvas, false, false, true)
            .await
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
    async fn render_static_e1m1_inner(
        &self,
        canvas: HtmlCanvasElement,
        include_masked_cutouts: bool,
        select_by_frustum: bool,
        focus_exitsign: bool,
    ) -> Result<String, String> {
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
                Vec3::new(
                    f32::from(start.position[0]),
                    (f32::from(vertical.floor_height) + f32::from(vertical.ceiling_height)) * 0.5,
                    f32::from(start.position[1]),
                ),
                doom_heading_forward(start.angle),
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
        let draws =
            build_static_draw_plan(&flats, &walls, &uploads).map_err(|error| error.to_string())?;
        let exitsign_view = focus_exitsign
            .then(|| exitsign_camera(&draws))
            .transpose()?;
        let (cutout_uploads, cutout_draws) = if include_masked_cutouts {
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
        camera.projection = Mat4::perspective_rh_gl(
            60_f32.to_radians(),
            width as f32 / height as f32,
            (radius * 0.0001).max(0.1),
            radius * 4.0,
        );
        camera.view = if let Some((position, target)) = exitsign_view {
            Mat4::look_at_rh(position, target, Vec3::Y)
        } else if let Some((position, source_forward)) = source_spawn {
            let yaw = observer_yaw_from_forward(source_forward) + std::f32::consts::FRAC_PI_2;
            Mat4::look_at_rh(
                position,
                position + observer_direction(yaw, 0.0) * 128.0,
                Vec3::Y,
            )
        } else {
            Mat4::look_at_rh(
                center + Vec3::new(radius, radius * 0.72, radius),
                center,
                Vec3::Y,
            )
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
        renderer.submit(&commands);
        renderer.present().map_err(|error| error.to_string())?;
        let opaque_submitted = draws.len() - opaque_rejected;
        let cutout_submitted = if include_masked_cutouts {
            cutout_draws.len() - cutout_rejected
        } else {
            0
        };
        let draw_count = opaque_submitted + cutout_submitted;
        Ok(format!(
            "browser first frame presented: {draw_count} draws; candidates={}; rejected={}; opaque={opaque_submitted}/{}; cutouts={cutout_submitted}/{}; frustum_aabb={select_by_frustum}; camera={}; backend={backend_api}; device={device_kind}; adapter={adapter_name}; canvas={}x{}",
            draws.len() + if include_masked_cutouts { cutout_draws.len() } else { 0 },
            opaque_rejected + cutout_rejected,
            draws.len(),
            if include_masked_cutouts { cutout_draws.len() } else { 0 },
            if focus_exitsign { "canonical-exitsign" } else if select_by_frustum { "source-spawn-plus-90" } else { "overview" },
            width, height
        ))
    }
}

#[cfg(target_arch = "wasm32")]
fn exitsign_camera(draws: &[hello_doom_e1m1::StaticDrawPlanEntry]) -> Result<(Vec3, Vec3), String> {
    let exitsign = draws
        .iter()
        .filter(|draw| {
            matches!(
                draw.source,
                StaticDrawSource::Wall { source_linedef }
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
        session = BrowserIntakeSession::new_inner().unwrap();
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
}
