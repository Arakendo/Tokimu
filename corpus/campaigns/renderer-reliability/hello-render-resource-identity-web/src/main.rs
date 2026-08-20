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
    provider_staging_current_scene: Option<u32>,
    provider_staging_commits: u32,
    provider_staging_failures: u32,
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
            provider_staging_current_scene: None,
            provider_staging_commits: 0,
            provider_staging_failures: 0,
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

    /// Exercises real WGPU allocation overlap without changing the public
    /// renderer lifetime contract. A remains live while a deliberately invalid
    /// B is populated, presents again after that failure, and is replaced only
    /// after a complete second B stage validates.
    pub async fn probe_provider_staging(
        &mut self,
        canvas: HtmlCanvasElement,
    ) -> Result<String, JsValue> {
        const RESOURCE_COUNT: u64 = 8;
        const TEXTURE_WIDTH: u32 = 16;
        const TEXTURE_HEIGHT: u32 = 16;

        let width = canvas.width().max(1);
        let height = canvas.height().max(1);
        let mut renderer = WgpuBackend::for_window(canvas, width, height)
            .await
            .map_err(js_debug)?;
        self.backend_creations = self.backend_creations.saturating_add(1);
        let backend = renderer.backend_api();
        let device = renderer.device_kind();
        let adapter = renderer.adapter_name().to_owned();

        let commands_a = upload_provider_stage_fixture_to_backend(
            &mut renderer,
            0,
            RESOURCE_COUNT,
            TEXTURE_WIDTH,
            TEXTURE_HEIGHT,
        )?;
        renderer.begin_frame();
        renderer.submit(&commands_a);
        let initial_a = renderer.present().map_err(js_debug)?;

        let mut failed_b = renderer
            .experimental_begin_scene_resource_stage()
            .map_err(js_debug)?;
        upload_provider_stage_fixture_to_stage(
            &mut failed_b,
            1,
            RESOURCE_COUNT,
            TEXTURE_WIDTH,
            TEXTURE_HEIGHT,
        )?;
        let staged_before_failure = RESOURCE_COUNT * 3 + 2;
        let forced_failure = failed_b
            .upload_material(
                MaterialHandle(RESOURCE_COUNT + 1),
                &Material::new("forced-late-stage-failure", Color::rgb(1.0, 1.0, 1.0))
                    .with_texture(TextureHandle(RESOURCE_COUNT + 1)),
            )
            .expect_err("missing staged texture must fail before commit");
        drop(failed_b);

        renderer.begin_frame();
        renderer.submit(&commands_a);
        let a_after_failure = renderer.present().map_err(js_debug)?;

        let mut committed_b = renderer
            .experimental_begin_scene_resource_stage()
            .map_err(js_debug)?;
        let commands_b = upload_provider_stage_fixture_to_stage(
            &mut committed_b,
            1,
            RESOURCE_COUNT,
            TEXTURE_WIDTH,
            TEXTURE_HEIGHT,
        )?;
        committed_b.begin_frame();
        committed_b.submit(&commands_b);
        let commit = renderer
            .experimental_commit_scene_resource_stage(committed_b)
            .map_err(js_debug)?;
        let committed_b_frame = renderer.present().map_err(js_debug)?;
        let diagnostic_count = renderer.drain_diagnostics().len();
        self.renderer = Some(renderer);

        Ok(format!(
            "status=complete; lifetime-alternative=C-corpus-private-real-provider-staging; sequence=present-A>stage-B-late-failure>present-A>stage-B-complete>atomic-commit-B>present-B; backend-creations=1; device-creations=1; surface-creations=1; retained-provider-session=true; staged-before-failure={staged_before_failure}; forced-stage-failure={forced_failure:?}; A-draws-initial={}; A-draws-after-failed-B={}; last-known-good-preserved={}; commit-observation={commit:?}; B-draws-after-commit={}; retired-A-predictable={}; provider-diagnostics={diagnostic_count}; overlap-physical-bytes=unmeasured; retired-physical-reclamation=unobserved; repeated-replacement-pressure=not-exercised; public-handle-contract=unchanged; backend={backend}; device={device}; adapter={adapter}; canvas={width}x{height}",
            initial_a.frame.draw_calls,
            a_after_failure.frame.draw_calls,
            initial_a.frame.draw_calls == a_after_failure.frame.draw_calls,
            committed_b_frame.frame.draw_calls,
            commit.retired_meshes == RESOURCE_COUNT as u32
                && commit.committed_meshes == RESOURCE_COUNT as u32,
        ))
    }

    /// Performs one yielded step of the fixed Alternative-C provider pressure
    /// workload. JavaScript owns pacing between calls so WGPU and the browser
    /// event loop receive an ordinary presentation boundary after every
    /// replacement.
    pub async fn replace_scene_staged(
        &mut self,
        canvas: HtmlCanvasElement,
        scene_index: u32,
        inject_late_failure: bool,
    ) -> Result<String, JsValue> {
        const RESOURCE_COUNT: u64 = 64;
        const TEXTURE_WIDTH: u32 = 16;
        const TEXTURE_HEIGHT: u32 = 16;

        let width = canvas.width().max(1);
        let height = canvas.height().max(1);
        if self.renderer.is_none() {
            let mut renderer = WgpuBackend::for_window(canvas, width, height)
                .await
                .map_err(js_debug)?;
            self.backend_creations = self.backend_creations.saturating_add(1);
            let commands = upload_provider_stage_fixture_to_backend(
                &mut renderer,
                0,
                RESOURCE_COUNT,
                TEXTURE_WIDTH,
                TEXTURE_HEIGHT,
            )?;
            renderer.begin_frame();
            renderer.submit(&commands);
            let initial = renderer.present().map_err(js_debug)?;
            if initial.frame.draw_calls != RESOURCE_COUNT as u32 {
                return Err(JsValue::from_str(
                    "initial provider-pressure scene did not present its complete draw set",
                ));
            }
            self.renderer = Some(renderer);
            self.provider_staging_current_scene = Some(0);
        }

        self.replacement_attempts = self.replacement_attempts.saturating_add(1);
        let renderer = self
            .renderer
            .as_mut()
            .expect("provider-pressure renderer initialized above");
        let previous_scene = self
            .provider_staging_current_scene
            .expect("provider-pressure scene initialized above");
        let backend = renderer.backend_api();
        let device = renderer.device_kind();
        let adapter = renderer.adapter_name().to_owned();
        let mut preserved_draws_after_failure = None;
        let mut forced_failure = None;

        if inject_late_failure {
            let mut failed_candidate = renderer
                .experimental_begin_scene_resource_stage()
                .map_err(js_debug)?;
            upload_provider_stage_fixture_to_stage(
                &mut failed_candidate,
                scene_index,
                RESOURCE_COUNT,
                TEXTURE_WIDTH,
                TEXTURE_HEIGHT,
            )?;
            let failure = failed_candidate
                .upload_material(
                    MaterialHandle(RESOURCE_COUNT + 1),
                    &Material::new(
                        format!("pressure-{scene_index}-late-failure"),
                        Color::rgb(1.0, 1.0, 1.0),
                    )
                    .with_texture(TextureHandle(RESOURCE_COUNT + 1)),
                )
                .expect_err("missing candidate texture must reject the pressure stage");
            drop(failed_candidate);
            self.provider_staging_failures = self.provider_staging_failures.saturating_add(1);

            let previous_commands =
                provider_stage_commands(RESOURCE_COUNT, tokimu::PipelineHandle(0), previous_scene);
            renderer.begin_frame();
            renderer.submit(&previous_commands);
            let preserved = renderer.present().map_err(js_debug)?;
            if preserved.frame.draw_calls != RESOURCE_COUNT as u32 {
                return Err(JsValue::from_str(
                    "late candidate failure did not preserve the complete current draw set",
                ));
            }
            preserved_draws_after_failure = Some(preserved.frame.draw_calls);
            forced_failure = Some(failure);
        }

        let mut candidate = renderer
            .experimental_begin_scene_resource_stage()
            .map_err(js_debug)?;
        let candidate_commands = upload_provider_stage_fixture_to_stage(
            &mut candidate,
            scene_index,
            RESOURCE_COUNT,
            TEXTURE_WIDTH,
            TEXTURE_HEIGHT,
        )?;
        candidate.begin_frame();
        candidate.submit(&candidate_commands);
        let commit = renderer
            .experimental_commit_scene_resource_stage(candidate)
            .map_err(js_debug)?;
        let presented = renderer.present().map_err(js_debug)?;
        let diagnostics = renderer.drain_diagnostics();
        if !diagnostics.is_empty() {
            return Err(JsValue::from_str(&format!(
                "provider staging pressure produced diagnostics: {diagnostics:?}"
            )));
        }
        if presented.frame.draw_calls != RESOURCE_COUNT as u32
            || commit.retired_meshes != RESOURCE_COUNT as u32
            || commit.committed_meshes != RESOURCE_COUNT as u32
            || commit.retired_textures != RESOURCE_COUNT as u32
            || commit.committed_textures != RESOURCE_COUNT as u32
            || commit.retired_materials != RESOURCE_COUNT as u32
            || commit.committed_materials != RESOURCE_COUNT as u32
            || commit.retired_queued_draws != RESOURCE_COUNT as u32
            || commit.committed_queued_draws != RESOURCE_COUNT as u32
        {
            return Err(JsValue::from_str(
                "provider staging pressure departed from its steady logical inventory",
            ));
        }

        self.provider_staging_commits = self.provider_staging_commits.saturating_add(1);
        self.replacements_presented = self.replacements_presented.saturating_add(1);
        self.retired_logical_sets = self.retired_logical_sets.saturating_add(1);
        self.provider_staging_current_scene = Some(scene_index);

        let live_estimated_bytes = provider_stage_estimated_source_bytes(
            scene_index,
            RESOURCE_COUNT,
            TEXTURE_WIDTH,
            TEXTURE_HEIGHT,
        );
        let overlap_estimated_bytes = live_estimated_bytes.saturating_mul(2);
        Ok(format!(
            "status=presented; lifetime-alternative=C-corpus-private-real-provider-staging-pressure; replacement-attempt={}; committed-replacements={}; target-scene={scene_index}; previous-scene={previous_scene}; injected-late-failure={inject_late_failure}; forced-stage-failure={forced_failure:?}; preserved-draws-after-failure={preserved_draws_after_failure:?}; total-injected-failures={}; draws={}; steady-logical-resources=[meshes:{RESOURCE_COUNT},textures:{RESOURCE_COUNT},materials:{RESOURCE_COUNT},pipelines:1,cameras:1,commands:{RESOURCE_COUNT}]; commit-observation={commit:?}; logical-overlap-sets-during-stage=2; estimated-source-bytes-live={live_estimated_bytes}; estimated-source-bytes-at-overlap={overlap_estimated_bytes}; post-commit-logical-sets=1; provider-object-drop-issued=true; physical-gpu-reclamation=unobserved; provider-diagnostics=0; backend-creations={}; device-creations={}; surface-creations={}; retained-provider-session=true; backend={backend}; device={device}; adapter={adapter}; canvas={width}x{height}",
            self.replacement_attempts,
            self.provider_staging_commits,
            self.provider_staging_failures,
            presented.frame.draw_calls,
            self.backend_creations,
            self.backend_creations,
            self.backend_creations,
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
fn upload_provider_stage_fixture_to_backend(
    renderer: &mut WgpuBackend,
    scene_index: u32,
    resource_count: u64,
    texture_width: u32,
    texture_height: u32,
) -> Result<Vec<RenderCommand>, JsValue> {
    let pipeline = renderer
        .register_pipeline(&Pipeline::new(
            format!("provider-stage-{scene_index}"),
            PipelineKind::LitColor3d,
        ))
        .map_err(js_debug)?;
    renderer.upload_camera(CameraHandle(1), Camera::default());
    let descriptor =
        Rgba8TextureDescriptor::new(texture_width, texture_height, Rgba8TextureColorSpace::Srgb);
    let mut commands = provider_stage_commands(resource_count, pipeline, scene_index);
    for resource_index in 0..resource_count {
        let rgba8 =
            provider_stage_texture(scene_index, resource_index, texture_width, texture_height);
        renderer
            .create_texture_rgba8(TextureHandle(resource_index + 1), descriptor, &rgba8)
            .map_err(js_debug)?;
        renderer
            .upload_material(
                MaterialHandle(resource_index + 1),
                &Material::new(
                    format!("provider-stage-{scene_index}-material-{resource_index}"),
                    Color::rgb(1.0, 1.0, 1.0),
                )
                .with_texture(TextureHandle(resource_index + 1)),
            )
            .map_err(js_debug)?;
        renderer.upload_mesh(
            MeshHandle(resource_index + 1),
            &provider_stage_mesh(scene_index, resource_index),
        );
    }
    commands.shrink_to_fit();
    Ok(commands)
}

#[cfg(target_arch = "wasm32")]
fn upload_provider_stage_fixture_to_stage(
    stage: &mut tokimu::ExperimentalSceneResourceStage,
    scene_index: u32,
    resource_count: u64,
    texture_width: u32,
    texture_height: u32,
) -> Result<Vec<RenderCommand>, JsValue> {
    let pipeline = stage
        .register_pipeline(&Pipeline::new(
            format!("provider-stage-{scene_index}"),
            PipelineKind::LitColor3d,
        ))
        .map_err(js_debug)?;
    stage.upload_camera(CameraHandle(1), Camera::default());
    let descriptor =
        Rgba8TextureDescriptor::new(texture_width, texture_height, Rgba8TextureColorSpace::Srgb);
    let mut commands = provider_stage_commands(resource_count, pipeline, scene_index);
    for resource_index in 0..resource_count {
        let rgba8 =
            provider_stage_texture(scene_index, resource_index, texture_width, texture_height);
        stage
            .create_texture_rgba8(TextureHandle(resource_index + 1), descriptor, &rgba8)
            .map_err(js_debug)?;
        stage
            .upload_material(
                MaterialHandle(resource_index + 1),
                &Material::new(
                    format!("provider-stage-{scene_index}-material-{resource_index}"),
                    Color::rgb(1.0, 1.0, 1.0),
                )
                .with_texture(TextureHandle(resource_index + 1)),
            )
            .map_err(js_debug)?;
        stage.upload_mesh(
            MeshHandle(resource_index + 1),
            &provider_stage_mesh(scene_index, resource_index),
        );
    }
    commands.shrink_to_fit();
    Ok(commands)
}

#[cfg(target_arch = "wasm32")]
fn provider_stage_commands(
    resource_count: u64,
    pipeline: tokimu::PipelineHandle,
    scene_index: u32,
) -> Vec<RenderCommand> {
    let mut commands = Vec::with_capacity(resource_count as usize + 1);
    commands.push(RenderCommand::Clear(ClearCommand {
        color: if scene_index == 0 {
            Color::rgb(0.02, 0.03, 0.08)
        } else {
            Color::rgb(0.08, 0.02, 0.03)
        },
    }));
    for resource_index in 0..resource_count {
        let column = (resource_index % 4) as f32;
        let row = (resource_index / 4) as f32;
        commands.push(RenderCommand::DrawMesh(DrawMeshCommand {
            mesh: MeshHandle(resource_index + 1),
            material: MaterialHandle(resource_index + 1),
            pipeline,
            instance: Instance2d::new([-0.75 + column * 0.5, -0.35 + row * 0.7], [0.16, 0.24], 0.0),
            camera: Some(CameraHandle(1)),
            viewport: None,
        }));
    }
    commands
}

#[cfg(target_arch = "wasm32")]
fn provider_stage_texture(
    scene_index: u32,
    resource_index: u64,
    width: u32,
    height: u32,
) -> Vec<u8> {
    let shade = scene_index
        .wrapping_mul(79)
        .wrapping_add(resource_index as u32 * 23) as u8;
    let mut rgba8 = vec![0; (width * height * 4) as usize];
    for pixel in rgba8.chunks_exact_mut(4) {
        pixel.copy_from_slice(&[shade, 255 - shade, shade.wrapping_add(61), 255]);
    }
    rgba8
}

#[cfg(target_arch = "wasm32")]
fn provider_stage_mesh(scene_index: u32, resource_index: u64) -> Mesh {
    if (u64::from(scene_index) + resource_index) % 2 == 0 {
        Mesh::triangle()
    } else {
        Mesh::diamond()
    }
}

#[cfg(target_arch = "wasm32")]
fn provider_stage_estimated_source_bytes(
    scene_index: u32,
    resource_count: u64,
    texture_width: u32,
    texture_height: u32,
) -> u64 {
    let texture_bytes = resource_count
        .saturating_mul(u64::from(texture_width))
        .saturating_mul(u64::from(texture_height))
        .saturating_mul(4);
    let mesh_vertex_bytes = (0..resource_count).fold(0_u64, |total, resource_index| {
        let mesh = provider_stage_mesh(scene_index, resource_index);
        total.saturating_add(mesh.positions.len() as u64 * 8 * 4)
    });
    texture_bytes.saturating_add(mesh_vertex_bytes)
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
