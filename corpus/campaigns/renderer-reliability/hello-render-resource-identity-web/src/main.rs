#[cfg(target_arch = "wasm32")]
use hello_render_resource_identity::{
    correlate_scene_resource_inventories, observe_e1m1_e1m2_generation_replacement,
    observe_failure_boundary_fixture, ApplicationMeshRegistry, CorpusSceneResourceInventory,
    ExplicitLifecycleLedger, FailureObservationCategory, GenerationalMeshRegistry, LogicalMesh,
};
#[cfg(target_arch = "wasm32")]
use tokimu::{
    Camera, CameraHandle, ClearCommand, Color, DrawMeshCommand, Instance2d, Material,
    MaterialHandle, Mesh, MeshHandle, Pipeline, PipelineKind, RenderCommand, Renderer,
    Rgba8TextureColorSpace, Rgba8TextureDescriptor, TextureHandle, WgpuBackend,
};
#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;
#[cfg(target_arch = "wasm32")]
use web_sys::HtmlCanvasElement;

#[cfg(not(target_arch = "wasm32"))]
fn main() {
    println!("hello-render-resource-identity-web is a browser/WASM corpus consumer");
}

#[cfg(target_arch = "wasm32")]
fn main() {}

/// Runs the pure-Rust Alternative-C semantic experiment in browser WASM. It
/// does not stage WGPU resources or admit generation vocabulary to Tokimu.
#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
pub fn run_scene_generation_prototype() -> String {
    let evidence = observe_e1m1_e1m2_generation_replacement();
    let independent = correlate_scene_resource_inventories(
        independent_semantic_inventory(0, 65),
        independent_semantic_inventory(1, 65),
    )
    .expect("the fixed independent inventories must correlate");
    format!(
        "status=complete; lifetime-alternative=C-corpus-private-generation; sequence=commit-E1M1-A>reject-E1M2-B>retain-E1M1-A>commit-E1M2-B>reject-stale-E1M1-A; generation-a={}; failed-generation-b={:?}; map-after-failed-stage={:?}; generation-a-after-failed-stage={:?}; generation-b={}; retired-map={:?}; map-after-commit={:?}; committed-draws={}; generation-a-after-commit={:?}; generation-b-after-commit={:?}; independent-resource-rich-correlation={independent:?}; renderer-resources=not-exercised; provider-session=not-exercised; physical-gpu-reclamation=not-applicable; admission=none",
        evidence.generation_a,
        evidence.failed_generation_b,
        evidence.map_after_failed_stage,
        evidence.generation_a_after_failed_stage,
        evidence.generation_b,
        evidence.retired_map,
        evidence.map_after_commit,
        evidence.committed_draw_count,
        evidence.generation_a_after_commit,
        evidence.generation_b_after_commit,
    )
}

#[cfg(target_arch = "wasm32")]
fn independent_semantic_inventory(scene_index: u32, commands: u64) -> CorpusSceneResourceInventory {
    CorpusSceneResourceInventory {
        source_label: format!("independent pressure scene {scene_index}"),
        meshes: 64,
        textures: 64,
        materials: 64,
        pipelines: 1,
        cameras: 1,
        commands,
    }
}

/// Independent whole-backend replacement baseline for the renderer lifetime
/// study. It deliberately retains application-owned handles and exposes no
/// reset/arena/release contract.
#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
pub struct BrowserReplacementPressure {
    renderer: Option<WgpuBackend>,
    replacement_attempts: u32,
    replacements_presented: u32,
    backend_creations: u32,
    retired_logical_sets: u32,
    previous_semantic_inventory: Option<CorpusSceneResourceInventory>,
}

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
impl BrowserReplacementPressure {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        Self {
            renderer: None,
            replacement_attempts: 0,
            replacements_presented: 0,
            backend_creations: 0,
            retired_logical_sets: 0,
            previous_semantic_inventory: None,
        }
    }

    /// Replaces one small but resource-rich scene on the same canvas. Every
    /// call intentionally creates a fresh backend/device/surface so this path
    /// remains the non-Doom Alternative-A control.
    pub async fn replace_scene(
        &mut self,
        canvas: HtmlCanvasElement,
        scene_index: u32,
    ) -> Result<String, JsValue> {
        const RESOURCE_COUNT: u64 = 64;
        const TEXTURE_WIDTH: u32 = 16;
        const TEXTURE_HEIGHT: u32 = 16;
        self.replacement_attempts = self.replacement_attempts.saturating_add(1);
        if let Some(previous) = self.renderer.take() {
            self.retired_logical_sets = self.retired_logical_sets.saturating_add(1);
            drop(previous);
        }

        let width = canvas.width().max(1);
        let height = canvas.height().max(1);
        let mut renderer = WgpuBackend::for_window(canvas, width, height)
            .await
            .map_err(js_debug)?;
        self.backend_creations = self.backend_creations.saturating_add(1);
        let backend = renderer.backend_api();
        let device = renderer.device_kind();
        let adapter = renderer.adapter_name().to_owned();
        let pipeline = renderer
            .register_pipeline(&Pipeline::new(
                "resource-lifetime-browser-pressure",
                PipelineKind::LitColor3d,
            ))
            .map_err(js_debug)?;
        renderer.upload_camera(CameraHandle(1), Camera::default());

        let descriptor = Rgba8TextureDescriptor::new(
            TEXTURE_WIDTH,
            TEXTURE_HEIGHT,
            Rgba8TextureColorSpace::Srgb,
        );
        let mut commands = Vec::with_capacity(RESOURCE_COUNT as usize + 1);
        let mut mesh_vertex_bytes = 0_u64;
        commands.push(RenderCommand::Clear(ClearCommand {
            color: Color::rgb(0.01, 0.015, 0.02),
        }));
        for resource_index in 0..RESOURCE_COUNT {
            let mesh = MeshHandle(resource_index + 1);
            let texture = TextureHandle(resource_index + 1);
            let material = MaterialHandle(resource_index + 1);
            let shade = scene_index
                .wrapping_mul(31)
                .wrapping_add(resource_index as u32 * 17) as u8;
            let mut rgba8 = vec![0_u8; (TEXTURE_WIDTH * TEXTURE_HEIGHT * 4) as usize];
            for pixel in rgba8.chunks_exact_mut(4) {
                pixel.copy_from_slice(&[shade, shade.wrapping_add(73), 255 - shade, 255]);
            }
            renderer
                .create_texture_rgba8(texture, descriptor, &rgba8)
                .map_err(js_debug)?;
            renderer
                .upload_material(
                    material,
                    &Material::new(
                        format!("replacement-{scene_index}-material-{resource_index}"),
                        Color::rgb(1.0, 1.0, 1.0),
                    )
                    .with_texture(texture),
                )
                .map_err(js_debug)?;
            let mesh_value = if (scene_index + resource_index as u32) % 2 == 0 {
                Mesh::triangle()
            } else {
                Mesh::diamond()
            };
            mesh_vertex_bytes =
                mesh_vertex_bytes.saturating_add(mesh_value.positions.len() as u64 * 8 * 4);
            renderer.upload_mesh(mesh, &mesh_value);
            let column = (resource_index % 8) as f32;
            let row = (resource_index / 8) as f32;
            commands.push(RenderCommand::DrawMesh(DrawMeshCommand {
                mesh,
                material,
                pipeline,
                instance: Instance2d::new(
                    [-0.875 + column * 0.25, -0.875 + row * 0.25],
                    [0.1, 0.1],
                    0.0,
                ),
                camera: Some(CameraHandle(1)),
                viewport: None,
            }));
        }
        // Preserve the existing meaning of same-handle replacement inside one
        // live set. This is distinct from reusing the same numeric handle in a
        // successor set after reset.
        renderer.upload_mesh(MeshHandle(1), &Mesh::diamond());
        renderer.begin_frame();
        renderer.submit(&commands);
        let stats = renderer.present().map_err(js_debug)?;
        if let Some(record) = renderer.drain_diagnostics().into_iter().next() {
            return Err(JsValue::from_str(&format!(
                "replacement-pressure WebGPU diagnostic: category={:?}; source={}; message={}",
                record.kind, record.source, record.message
            )));
        }
        self.replacements_presented = self.replacements_presented.saturating_add(1);
        self.renderer = Some(renderer);

        let current_semantic_inventory =
            independent_semantic_inventory(scene_index, commands.len() as u64);
        let semantic_correlation = self
            .previous_semantic_inventory
            .as_ref()
            .map(|previous| {
                correlate_scene_resource_inventories(
                    previous.clone(),
                    current_semantic_inventory.clone(),
                )
            })
            .transpose()
            .map_err(js_debug)?;
        self.previous_semantic_inventory = Some(current_semantic_inventory);

        Ok(format!(
            "status=presented; caller=non-doom-resource-lifetime-pressure; lifetime-baseline=whole-backend-replacement; scene-index={scene_index}; replacement-attempts={}; replacements-presented={}; backend-creations={}; device-creations={}; surface-creations={}; current-logical-resources=[meshes:{RESOURCE_COUNT},textures:{RESOURCE_COUNT},materials:{RESOURCE_COUNT},pipelines:1,cameras:1,commands:{}]; current-logical-uploads=[meshes:{RESOURCE_COUNT},textures:{RESOURCE_COUNT},materials:{RESOURCE_COUNT},pipelines:1,cameras:1]; current-same-handle-replacements=[meshes:0,textures:0,materials:0,pipelines:0,cameras:0]; current-estimated-bytes=[mesh-vertices:{},source-texture-payloads:{}]; retired-logical-sets={}; alternative-c-inventory-correlation={semantic_correlation:?}; alternative-c-authority=semantic-shadow-not-provider-lifetime; physical-gpu-reclamation=unobserved; draws={}; backend={backend}; device={device}; adapter={adapter}; canvas={}x{}",
            self.replacement_attempts,
            self.replacements_presented,
            self.backend_creations,
            self.backend_creations,
            self.backend_creations,
            commands.len(),
            mesh_vertex_bytes,
            RESOURCE_COUNT * u64::from(TEXTURE_WIDTH) * u64::from(TEXTURE_HEIGHT) * 4,
            self.retired_logical_sets,
            stats.frame.draw_calls,
            width,
            height,
        ))
    }

    /// Replaces one scene's logical resource set while retaining the WGPU
    /// adapter/device/queue/surface session. This is the private Alternative-B
    /// prototype, not a stable renderer lifecycle contract.
    pub async fn replace_scene_retained(
        &mut self,
        canvas: HtmlCanvasElement,
        scene_index: u32,
    ) -> Result<String, JsValue> {
        const RESOURCE_COUNT: u64 = 64;
        const TEXTURE_WIDTH: u32 = 16;
        const TEXTURE_HEIGHT: u32 = 16;
        self.replacement_attempts = self.replacement_attempts.saturating_add(1);

        let width = canvas.width().max(1);
        let height = canvas.height().max(1);
        let (mut renderer, reset) = if let Some(mut retained) = self.renderer.take() {
            self.retired_logical_sets = self.retired_logical_sets.saturating_add(1);
            let reset = retained.experimental_reset_scene_resources();
            (retained, Some(reset))
        } else {
            let renderer = WgpuBackend::for_window(canvas, width, height)
                .await
                .map_err(js_debug)?;
            self.backend_creations = self.backend_creations.saturating_add(1);
            (renderer, None)
        };
        let backend = renderer.backend_api();
        let device = renderer.device_kind();
        let adapter = renderer.adapter_name().to_owned();
        let pipeline = renderer
            .register_pipeline(&Pipeline::new(
                "resource-lifetime-browser-pressure",
                PipelineKind::LitColor3d,
            ))
            .map_err(js_debug)?;
        renderer.upload_camera(CameraHandle(1), Camera::default());

        let descriptor = Rgba8TextureDescriptor::new(
            TEXTURE_WIDTH,
            TEXTURE_HEIGHT,
            Rgba8TextureColorSpace::Srgb,
        );
        let mut commands = Vec::with_capacity(RESOURCE_COUNT as usize + 1);
        let mut mesh_vertex_bytes = 0_u64;
        commands.push(RenderCommand::Clear(ClearCommand {
            color: Color::rgb(0.01, 0.015, 0.02),
        }));
        for resource_index in 0..RESOURCE_COUNT {
            let mesh = MeshHandle(resource_index + 1);
            let texture = TextureHandle(resource_index + 1);
            let material = MaterialHandle(resource_index + 1);
            let shade = scene_index
                .wrapping_mul(31)
                .wrapping_add(resource_index as u32 * 17) as u8;
            let mut rgba8 = vec![0_u8; (TEXTURE_WIDTH * TEXTURE_HEIGHT * 4) as usize];
            for pixel in rgba8.chunks_exact_mut(4) {
                pixel.copy_from_slice(&[shade, shade.wrapping_add(73), 255 - shade, 255]);
            }
            renderer
                .create_texture_rgba8(texture, descriptor, &rgba8)
                .map_err(js_debug)?;
            renderer
                .upload_material(
                    material,
                    &Material::new(
                        format!("replacement-{scene_index}-material-{resource_index}"),
                        Color::rgb(1.0, 1.0, 1.0),
                    )
                    .with_texture(texture),
                )
                .map_err(js_debug)?;
            let mesh_value = if (scene_index + resource_index as u32) % 2 == 0 {
                Mesh::triangle()
            } else {
                Mesh::diamond()
            };
            mesh_vertex_bytes =
                mesh_vertex_bytes.saturating_add(mesh_value.positions.len() as u64 * 8 * 4);
            renderer.upload_mesh(mesh, &mesh_value);
            let column = (resource_index % 8) as f32;
            let row = (resource_index / 8) as f32;
            commands.push(RenderCommand::DrawMesh(DrawMeshCommand {
                mesh,
                material,
                pipeline,
                instance: Instance2d::new(
                    [-0.875 + column * 0.25, -0.875 + row * 0.25],
                    [0.1, 0.1],
                    0.0,
                ),
                camera: Some(CameraHandle(1)),
                viewport: None,
            }));
        }
        renderer.begin_frame();
        renderer.submit(&commands);
        let stats = renderer.present().map_err(js_debug)?;
        if let Some(record) = renderer.drain_diagnostics().into_iter().next() {
            return Err(JsValue::from_str(&format!(
                "replacement-pressure WebGPU diagnostic: category={:?}; source={}; message={}",
                record.kind, record.source, record.message
            )));
        }
        self.replacements_presented = self.replacements_presented.saturating_add(1);
        self.renderer = Some(renderer);

        Ok(format!(
            "status=presented; caller=non-doom-resource-lifetime-pressure; lifetime-alternative=adapter-private-scene-reset; scene-index={scene_index}; replacement-attempts={}; replacements-presented={}; backend-creations={}; device-creations={}; surface-creations={}; current-logical-resources=[meshes:{RESOURCE_COUNT},textures:{RESOURCE_COUNT},materials:{RESOURCE_COUNT},pipelines:1,cameras:1,commands:{}]; retired-logical-sets={}; reset-observation={reset:?}; retained-provider-session=true; retained-instance-bindings={}; in-set-same-handle-replacements={}; physical-gpu-reclamation=unobserved; current-estimated-bytes=[mesh-vertices:{},source-texture-payloads:{}]; draws={}; backend={backend}; device={device}; adapter={adapter}; canvas={}x{}",
            self.replacement_attempts,
            self.replacements_presented,
            self.backend_creations,
            self.backend_creations,
            self.backend_creations,
            commands.len(),
            self.retired_logical_sets,
            reset.map_or(0, |value| value.retained_instance_bindings),
            stats.lifetime.mesh_replacements,
            mesh_vertex_bytes,
            RESOURCE_COUNT * u64::from(TEXTURE_WIDTH) * u64::from(TEXTURE_HEIGHT) * 4,
            stats.frame.draw_calls,
            width,
            height,
        ))
    }

    /// Submits a command whose bare handles are indistinguishable from an old
    /// scene's handles after those numeric values have been reused by the
    /// current scene. Successful presentation is the B stale-identity
    /// falsifier: the backend cannot reject the old command as old.
    pub fn probe_retained_cross_set_aliasing(&mut self) -> Result<String, JsValue> {
        if self.retired_logical_sets == 0 {
            return Err(JsValue::from_str(
                "run at least two retained-session replacements before probing aliasing",
            ));
        }
        let renderer = self
            .renderer
            .as_mut()
            .ok_or_else(|| JsValue::from_str("no retained scene is available"))?;
        renderer.begin_frame();
        renderer.submit(&[
            RenderCommand::Clear(ClearCommand {
                color: Color::rgb(0.01, 0.015, 0.02),
            }),
            RenderCommand::DrawMesh(DrawMeshCommand {
                mesh: MeshHandle(1),
                material: MaterialHandle(1),
                pipeline: tokimu::PipelineHandle(0),
                instance: Instance2d::identity(),
                camera: Some(CameraHandle(1)),
                viewport: None,
            }),
        ]);
        let stats = renderer.present().map_err(js_debug)?;

        Ok(format!(
            "status=falsified; lifetime-alternative=adapter-private-scene-reset; requirement=deterministic-cross-set-stale-handle-rejection; old-command-handles=[mesh:1,material:1,pipeline:0,camera:1]; successor-handles=[mesh:1,material:1,pipeline:0,camera:1]; result=old-command-resolved-successor-resources; draws={}; cross-set-aliasing=true; generation-evidence=absent",
            stats.frame.draw_calls,
        ))
    }

    /// Demonstrates Alternative B's atomicity limit by forcing staging to fail
    /// after the old logical set has already been retired, then attempting to
    /// resolve an old-scene draw.
    pub fn probe_retained_reset_atomicity(&mut self) -> Result<String, JsValue> {
        let mut renderer = self
            .renderer
            .take()
            .ok_or_else(|| JsValue::from_str("render a retained scene before probing atomicity"))?;
        let reset = renderer.experimental_reset_scene_resources();
        let descriptor = Rgba8TextureDescriptor::new(1, 1, Rgba8TextureColorSpace::Srgb);
        renderer
            .create_texture_rgba8(TextureHandle(1), descriptor, &[255, 0, 255, 255])
            .map_err(js_debug)?;
        let staging_failure = renderer
            .create_texture_rgba8(TextureHandle(1), descriptor, &[0, 0, 0, 255])
            .expect_err("duplicate texture must force successor staging failure");

        renderer.begin_frame();
        renderer.submit(&[
            RenderCommand::Clear(ClearCommand {
                color: Color::rgb(0.01, 0.015, 0.02),
            }),
            RenderCommand::DrawMesh(DrawMeshCommand {
                mesh: MeshHandle(1),
                material: MaterialHandle(1),
                pipeline: tokimu::PipelineHandle(0),
                instance: Instance2d::identity(),
                camera: Some(CameraHandle(1)),
                viewport: None,
            }),
        ]);
        let old_scene_resolution = renderer
            .present()
            .expect_err("retired old-scene handles must not resolve");
        self.renderer = Some(renderer);

        Ok(format!(
            "status=falsified; lifetime-alternative=adapter-private-scene-reset; requirement=atomic-last-known-good-replacement; reset-observation={reset:?}; forced-successor-staging-failure={staging_failure:?}; old-scene-after-failure={old_scene_resolution:?}; stale-handle-rejection=deterministic; last-known-good-preserved=false; physical-gpu-reclamation=unobserved"
        ))
    }
}

#[cfg(target_arch = "wasm32")]
impl Default for BrowserReplacementPressure {
    fn default() -> Self {
        Self::new()
    }
}

/// Runs the same B/D/E identity pressure in browser WASM, then proves that the
/// browser WGPU provider retains the existing same-handle replacement mechanic.
/// The returned record is fixture evidence, not a public renderer contract.
#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
pub async fn run_fixture(canvas: HtmlCanvasElement) -> Result<String, JsValue> {
    console_error_panic_hook::set_once();

    let mut application = ApplicationMeshRegistry::default();
    let application_resource = LogicalMesh::Dynamic(7);
    let application_handle = application.create(application_resource);
    application
        .replace(application_handle, application_resource)
        .map_err(js_debug)?;
    let application_mismatch = application
        .replace(application_handle, LogicalMesh::Dynamic(8))
        .expect_err("different logical identity must be rejected");

    let mut generational = GenerationalMeshRegistry::default();
    let old_generation = generational.create(LogicalMesh::Dynamic(11));
    generational.retire(old_generation).map_err(js_debug)?;
    let new_generation = generational.create(LogicalMesh::Dynamic(12));
    let stale_generation = generational
        .resolve(old_generation)
        .expect_err("retired generation must remain stale after slot reuse");

    let mut explicit = ExplicitLifecycleLedger::default();
    let explicit_handle = MeshHandle(44);
    explicit
        .create(explicit_handle, LogicalMesh::Dynamic(44))
        .map_err(js_debug)?;
    let duplicate_create = explicit
        .create(explicit_handle, LogicalMesh::Dynamic(45))
        .expect_err("duplicate live create must be rejected");
    explicit
        .replace(explicit_handle, LogicalMesh::Dynamic(44))
        .map_err(js_debug)?;
    explicit.retire(explicit_handle).map_err(js_debug)?;
    let missing_after_retire = explicit
        .resolve(explicit_handle)
        .expect_err("retired explicit identity must not resolve");

    let unresolved = observe_failure_boundary_fixture()
        .retained()
        .find(|record| record.category == FailureObservationCategory::ResourceUnresolved)
        .ok_or_else(|| JsValue::from_str("resource-unresolved observation is absent"))?;

    let width = canvas.width().max(1);
    let height = canvas.height().max(1);
    let mut renderer = WgpuBackend::for_window(canvas, width, height)
        .await
        .map_err(js_debug)?;
    let backend = renderer.backend_api();
    let device = renderer.device_kind();
    let adapter = renderer.adapter_name().to_owned();
    let mesh = MeshHandle(1);
    renderer.upload_mesh(mesh, &Mesh::triangle());
    renderer.upload_mesh(mesh, &Mesh::diamond());
    renderer
        .upload_material(
            MaterialHandle(1),
            &Material::new("resource-identity-replacement", Color::rgb(0.1, 0.9, 0.7)),
        )
        .map_err(js_debug)?;
    renderer.upload_camera(CameraHandle(1), Camera::default());
    let pipeline = renderer
        .register_pipeline(&Pipeline::new(
            "resource-identity-browser",
            PipelineKind::LitColor3d,
        ))
        .map_err(js_debug)?;
    renderer.begin_frame();
    renderer.submit(&[
        RenderCommand::Clear(ClearCommand {
            color: Color::rgb(0.01, 0.015, 0.02),
        }),
        RenderCommand::DrawMesh(DrawMeshCommand {
            mesh,
            material: MaterialHandle(1),
            pipeline,
            instance: Instance2d::identity(),
            camera: Some(CameraHandle(1)),
            viewport: None,
        }),
    ]);
    let stats = renderer.present().map_err(js_debug)?;

    Ok(format!(
        "status=presented; alternatives=B,D,E; application_handle={}; application_mismatch={application_mismatch:?}; application_counts={:?}; generational_old={}:{}; generational_new={}:{}; stale={stale_generation:?}; generational_counts={:?}; explicit_handle={}; duplicate={duplicate_create:?}; missing_after_retire={missing_after_retire:?}; explicit_counts={:?}; observation=category:{:?},resource:{:?},caller:{}; provider_same_handle_uploads={}; provider_same_handle_replacements={}; draws={}; backend={backend}; device={device}; adapter={adapter}; canvas={}x{}; host=DOM-after-provider-return",
        application_handle.0,
        application.counts(),
        old_generation.slot,
        old_generation.generation,
        new_generation.slot,
        new_generation.generation,
        generational.counts(),
        explicit_handle.0,
        explicit.counts(),
        unresolved.category,
        unresolved.resource,
        unresolved.caller,
        stats.lifetime.mesh_uploads,
        stats.lifetime.mesh_replacements,
        stats.frame.draw_calls,
        width,
        height,
    ))
}

#[cfg(target_arch = "wasm32")]
fn js_debug(error: impl std::fmt::Debug) -> JsValue {
    JsValue::from_str(&format!("{error:?}"))
}
