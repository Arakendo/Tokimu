use std::sync::Arc;

use tokimu::{
    run_window_with_app, Camera, CameraHandle, ClearCommand, Color, DrawMeshCommand, FrameOutcome,
    Instance2d, Material, MaterialHandle, Mesh, MeshHandle, NativeWindow, Pipeline, PipelineHandle,
    PipelineKind, PlatformEventHandler, PlatformInputEvent, PlatformResult, RenderCommand,
    Renderer, ShaderBindingDeclaration, ShaderBindingSource, ShaderDiagnosticStage,
    ShaderModuleDefinition, ShaderVertexInput, ShaderVertexSemantic, WgpuBackend, WindowConfig,
};
use tokimu_assets::{AssetHandle, AssetStore};
use tokimu_core::math::{Mat4, Vec3};

const QUAD_MESH: MeshHandle = MeshHandle(1);
const TRIANGLE_MESH: MeshHandle = MeshHandle(2);
const DIAMOND_MESH: MeshHandle = MeshHandle(3);
const NEON_MATERIAL: MaterialHandle = MaterialHandle(1);
const INK_MATERIAL: MaterialHandle = MaterialHandle(2);
const HIGHLIGHT_MATERIAL: MaterialHandle = MaterialHandle(3);
const CAMERA_HANDLE: CameraHandle = CameraHandle(1);

const SHADER_VARIANTS: [ShaderVariantDefinition; 3] = [
    ShaderVariantDefinition {
        name: "neon",
        asset_label: "shaders/neon.wgsl",
        source: NEON_WGSL,
        pipeline_label: "shader-neon-pipeline",
        material: NEON_MATERIAL,
    },
    ShaderVariantDefinition {
        name: "ink",
        asset_label: "shaders/ink.wgsl",
        source: INK_WGSL,
        pipeline_label: "shader-ink-pipeline",
        material: INK_MATERIAL,
    },
    ShaderVariantDefinition {
        name: "ripple",
        asset_label: "shaders/ripple.wgsl",
        source: RIPPLE_WGSL,
        pipeline_label: "shader-ripple-pipeline",
        material: HIGHLIGHT_MATERIAL,
    },
];

#[derive(Clone, Copy, Debug)]
struct ShaderAsset;

#[derive(Clone, Copy)]
struct ShaderVariantDefinition {
    name: &'static str,
    asset_label: &'static str,
    source: &'static str,
    pipeline_label: &'static str,
    material: MaterialHandle,
}

struct ShaderVariantRuntime {
    definition: ShaderVariantDefinition,
    asset: AssetHandle<ShaderAsset>,
    pipeline: PipelineHandle,
}

const NEON_WGSL: &str = include_str!("../assets/neon.wgsl");
const INK_WGSL: &str = include_str!("../assets/ink.wgsl");
const RIPPLE_WGSL: &str = include_str!("../assets/ripple.wgsl");
const BACKEND_INVALID_WGSL: &str = include_str!("../assets/backend-invalid.wgsl");

const BACKEND_DIAGNOSTIC_FIXTURE_ARGUMENT: &str = "--backend-diagnostic-fixture";
const SEMANTIC_DIAGNOSTIC_FIXTURE_ARGUMENT: &str = "--semantic-diagnostic-fixture";
const PIPELINE_DIAGNOSTIC_FIXTURE_ARGUMENT: &str = "--pipeline-diagnostic-fixture";
const DRAW_CONTRACT_DIAGNOSTIC_FIXTURE_ARGUMENT: &str = "--draw-contract-diagnostic-fixture";
const STEADY_STATE_FIXTURE_ARGUMENT: &str = "--steady-state-fixture";

fn shader_module_for_variant(
    definition: ShaderVariantDefinition,
) -> Result<ShaderModuleDefinition, tokimu::ShaderModuleValidationError> {
    ShaderModuleDefinition::new(
        definition
            .asset_label
            .trim_end_matches(".wgsl")
            .replace('/', "-"),
        definition.source,
        "vs_main",
        "fs_main",
        vec![
            ShaderBindingDeclaration::new(
                0,
                0,
                ShaderBindingSource::MaterialParameter {
                    parameter: "base_color".to_owned(),
                    kind: tokimu::MaterialParameterKind::Color,
                },
            ),
            ShaderBindingDeclaration::new(1, 0, ShaderBindingSource::InstanceTransform),
            ShaderBindingDeclaration::new(2, 0, ShaderBindingSource::Camera),
        ],
        vec![
            ShaderVertexInput::new(0, ShaderVertexSemantic::Position3),
            ShaderVertexInput::new(1, ShaderVertexSemantic::Normal3),
        ],
    )
}

fn main() -> PlatformResult<()> {
    if std::env::args().any(|argument| argument == SEMANTIC_DIAGNOSTIC_FIXTURE_ARGUMENT) {
        return run_semantic_diagnostic_fixture();
    }
    if std::env::args().any(|argument| argument == PIPELINE_DIAGNOSTIC_FIXTURE_ARGUMENT) {
        return run_pipeline_diagnostic_fixture();
    }
    if std::env::args().any(|argument| argument == DRAW_CONTRACT_DIAGNOSTIC_FIXTURE_ARGUMENT) {
        return run_draw_contract_diagnostic_fixture();
    }

    let arguments = std::env::args().collect::<Vec<_>>();

    run_window_with_app(
        WindowConfig {
            title: "Tokimu Hello Shader".into(),
            width: 1280,
            height: 720,
        },
        HelloShaderApp::new(
            arguments
                .iter()
                .any(|argument| argument == BACKEND_DIAGNOSTIC_FIXTURE_ARGUMENT),
            arguments
                .iter()
                .any(|argument| argument == STEADY_STATE_FIXTURE_ARGUMENT),
        ),
    )
}

fn run_semantic_diagnostic_fixture() -> PlatformResult<()> {
    let error = ShaderModuleDefinition::new(
        "hello-shader-semantic-invalid",
        "@vertex fn vs_main() {}",
        "vs-invalid",
        "fs_main",
        vec![],
        vec![],
    )
    .expect_err("the fixture's malformed WGSL entry point must be rejected");

    if !matches!(
        &error,
        tokimu::ShaderModuleValidationError::InvalidIdentifier {
            kind: "shader entry point",
            ref value,
        } if value == "vs-invalid"
    ) {
        return Err(format!("unexpected semantic shader diagnostic: {error}").into());
    }
    if error.stage() != tokimu::ShaderDiagnosticStage::SemanticValidation {
        return Err(format!(
            "semantic shader diagnostic reported the wrong stage: {:?}",
            error.stage()
        )
        .into());
    }

    println!(
        "hello-shader semantic diagnostic fixture passed: stage=semantic-validation, entry-point=vs-invalid"
    );
    Ok(())
}

fn run_pipeline_diagnostic_fixture() -> PlatformResult<()> {
    let error = Pipeline::new("hello-shader-missing-source", PipelineKind::CustomWgsl2d)
        .validate()
        .expect_err("a custom pipeline without source must be rejected");
    if error.stage() != ShaderDiagnosticStage::PipelineValidation {
        return Err(format!(
            "pipeline diagnostic reported the wrong stage: {:?}",
            error.stage()
        )
        .into());
    }

    println!("hello-shader pipeline diagnostic fixture passed: stage=pipeline-validation");
    Ok(())
}

fn run_draw_contract_diagnostic_fixture() -> PlatformResult<()> {
    let pipeline = Pipeline::new("hello-shader-lit-contract", PipelineKind::LitColor3d);
    let material = tokimu::MaterialDefinition::solid_color(
        tokimu::MaterialDefinitionId::new("hello-shader-fixture")?,
        Color::rgb(1.0, 1.0, 1.0),
    );
    let mesh = Mesh::new(vec![[0.0, 0.0, 0.0]], vec![]);
    let error = pipeline
        .validate_draw_contract(&material, &mesh)
        .expect_err("a lit draw without normals must be rejected");
    if error.stage() != ShaderDiagnosticStage::DrawContractValidation {
        return Err(format!(
            "draw contract diagnostic reported the wrong stage: {:?}",
            error.stage()
        )
        .into());
    }

    println!(
        "hello-shader draw contract diagnostic fixture passed: stage=draw-contract-validation"
    );
    Ok(())
}

struct HelloShaderApp {
    renderer: Option<WgpuBackend>,
    window: Option<Arc<NativeWindow>>,
    window_size: [f32; 2],
    elapsed_seconds: f64,
    frame_index: u64,
    shader_variant: usize,
    shader_store: AssetStore,
    shader_variants: Vec<ShaderVariantRuntime>,
    include_backend_diagnostic_fixture: bool,
    verify_steady_state: bool,
}

impl Default for HelloShaderApp {
    fn default() -> Self {
        Self {
            renderer: None,
            window: None,
            window_size: [1.0, 1.0],
            elapsed_seconds: 0.0,
            frame_index: 0,
            shader_variant: 0,
            shader_store: AssetStore::default(),
            shader_variants: Vec::new(),
            include_backend_diagnostic_fixture: false,
            verify_steady_state: false,
        }
    }
}

impl HelloShaderApp {
    fn new(include_backend_diagnostic_fixture: bool, verify_steady_state: bool) -> Self {
        Self {
            include_backend_diagnostic_fixture,
            verify_steady_state,
            ..Self::default()
        }
    }

    fn register_backend_diagnostic_fixture(renderer: &mut WgpuBackend) -> PlatformResult<()> {
        let module = ShaderModuleDefinition::new(
            "hello-shader-intentional-invalid",
            BACKEND_INVALID_WGSL,
            "vs_fixture",
            "fs_fixture",
            vec![],
            vec![ShaderVertexInput::new(0, ShaderVertexSemantic::Position3)],
        )?;
        let pipeline = Pipeline::custom_wgsl_module("shader-backend-diagnostic-fixture", module)?;

        // The unresolved fragment symbol must be reported through the backend
        // diagnostic sink. This pipeline is never retained or submitted.
        renderer.register_pipeline(&pipeline)?;
        Ok(())
    }

    fn update_window_title(&self) {
        if let Some(window) = self.window.as_ref() {
            let variant = &self.shader_variants[self.shader_variant];
            let inventory = self.shader_store.inventory();
            let source_label = inventory
                .entries
                .get(self.shader_variant)
                .and_then(|entry| entry.source.as_deref())
                .unwrap_or(SHADER_VARIANTS[self.shader_variant].asset_label);
            window.set_title(&format!(
                "Tokimu Hello Shader | variant={} ({}) | asset=#{} {} | elapsed={:.1}s",
                self.shader_variant,
                variant.definition.name,
                variant.asset.id().0,
                source_label,
                self.elapsed_seconds
            ));
        }
    }

    fn current_pipeline(&self) -> PipelineHandle {
        self.shader_variants[self.shader_variant % self.shader_variants.len()].pipeline
    }

    fn current_material(&self) -> MaterialHandle {
        self.shader_variants[self.shader_variant % self.shader_variants.len()]
            .definition
            .material
    }

    fn cycle_shader_variant(&mut self, step: isize) {
        let count = self.shader_variants.len() as isize;
        let next = (self.shader_variant as isize + step).rem_euclid(count);
        self.shader_variant = next as usize;
    }

    /// Fixture windows must release their presentation backend before asking
    /// the native event loop to exit. This keeps short-lived diagnostics from
    /// relying on process teardown to destroy the surface and its window.
    fn finish_fixture(&mut self) -> FrameOutcome {
        self.renderer.take();
        self.window.take();
        FrameOutcome::Exit
    }

    fn render_scene(&mut self) -> PlatformResult<FrameOutcome> {
        let seconds = self.elapsed_seconds as f32;
        let active_pipeline = self.current_pipeline();
        let active_material = self.current_material();
        let Some(renderer) = self.renderer.as_mut() else {
            return Ok(FrameOutcome::Continue);
        };

        renderer.upload_mesh(QUAD_MESH, &Mesh::quad());
        renderer.upload_mesh(TRIANGLE_MESH, &Mesh::triangle());
        renderer.upload_mesh(DIAMOND_MESH, &Mesh::diamond());

        let mut camera =
            Camera::orthographic_2d_with_height(self.window_size[0], self.window_size[1], 4.0);
        camera.view = Mat4::from_translation(Vec3::new(0.0, 0.0, 0.0));
        renderer.upload_camera(CAMERA_HANDLE, camera);
        renderer.set_active_camera(CAMERA_HANDLE);

        let quad_instance = Instance2d::identity()
            .with_translation([-1.9, 0.0])
            .with_scale([1.4, 1.4])
            .with_rotation(seconds * 0.35);
        let triangle_instance = Instance2d::identity()
            .with_translation([0.15, 0.1])
            .with_scale([1.0, 1.0])
            .with_rotation(-seconds * 0.55);
        let diamond_instance = Instance2d::identity()
            .with_translation([2.1, -0.08])
            .with_scale([1.25, 1.25])
            .with_rotation(seconds * 0.8);
        let accent_instance = Instance2d::identity()
            .with_translation([0.0, -1.2 + seconds.sin() * 0.08])
            .with_scale([3.8, 0.22])
            .with_rotation(seconds * 0.12);

        renderer.begin_frame();
        renderer.submit(&[
            RenderCommand::Clear(ClearCommand {
                color: Color::rgb(0.05, 0.06, 0.09),
            }),
            RenderCommand::DrawMesh(DrawMeshCommand {
                mesh: QUAD_MESH,
                material: active_material,
                pipeline: active_pipeline,
                instance: quad_instance,
                camera: Some(CAMERA_HANDLE),
                viewport: None,
            }),
            RenderCommand::DrawMesh(DrawMeshCommand {
                mesh: TRIANGLE_MESH,
                material: active_material,
                pipeline: active_pipeline,
                instance: triangle_instance,
                camera: Some(CAMERA_HANDLE),
                viewport: None,
            }),
            RenderCommand::DrawMesh(DrawMeshCommand {
                mesh: DIAMOND_MESH,
                material: active_material,
                pipeline: active_pipeline,
                instance: diamond_instance,
                camera: Some(CAMERA_HANDLE),
                viewport: None,
            }),
            RenderCommand::DrawMesh(DrawMeshCommand {
                mesh: QUAD_MESH,
                material: HIGHLIGHT_MATERIAL,
                pipeline: active_pipeline,
                instance: accent_instance,
                camera: Some(CAMERA_HANDLE),
                viewport: None,
            }),
        ]);
        let stats = renderer.present()?;
        self.frame_index = self.frame_index.saturating_add(1);
        if self.frame_index.is_multiple_of(120) {
            println!(
                "hello-shader frame {}: draws={}, material_resolutions={}, pipeline_switches={}, pipeline_creations={}, pipeline_replacements={}, transparent_draws={}, derived_cache_hits={}, derived_cache_misses={}, binding_allocations={}, uniform_writes={}",
                self.frame_index,
                stats.frame.draw_calls,
                stats.frame.material_resolutions,
                stats.frame.pipeline_switches,
                stats.frame.pipeline_creations,
                stats.frame.pipeline_replacements,
                stats.frame.transparent_draws,
                stats.frame.derived_material_cache_hits,
                stats.frame.derived_material_cache_misses,
                stats.frame.binding_allocations,
                stats.frame.uniform_buffer_writes,
            );
        }
        if self.include_backend_diagnostic_fixture {
            renderer.poll_diagnostics();
        }
        // WGPU can report shader and pipeline validation after the synchronous
        // creation call. Keep that adapter evidence visible to this corpus.
        let diagnostics = renderer.drain_diagnostics();
        for diagnostic in &diagnostics {
            eprintln!("hello-shader backend diagnostic: {diagnostic}");
        }
        if self.include_backend_diagnostic_fixture {
            let diagnostic_text = diagnostics
                .iter()
                .map(|diagnostic| diagnostic.message.as_str())
                .collect::<Vec<_>>()
                .join("\n");
            for expected in [
                "hello-shader-intentional-invalid",
                "vs_fixture",
                "fs_fixture",
            ] {
                if !diagnostic_text.contains(expected) {
                    return Err(format!(
                        "backend diagnostic fixture did not retain expected semantic identity `{expected}`"
                    )
                    .into());
                }
            }
            println!(
                "hello-shader backend diagnostic fixture passed: module=hello-shader-intentional-invalid, vertex=vs_fixture, fragment=fs_fixture"
            );
            println!(
                "hello-shader backend diagnostic fixture continued with valid frame: draws={}, material_resolutions={}, pipeline_switches={}",
                stats.frame.draw_calls,
                stats.frame.material_resolutions,
                stats.frame.pipeline_switches,
            );
            return Ok(self.finish_fixture());
        }
        if self.verify_steady_state && self.frame_index >= 2 {
            if stats.frame.binding_allocations != 0 || stats.frame.pipeline_creations != 0 {
                return Err(format!(
                    "unchanged steady-state frame allocated {} material bindings and created {} pipelines",
                    stats.frame.binding_allocations, stats.frame.pipeline_creations
                )
                .into());
            }
            println!(
                "hello-shader steady-state fixture passed: frame={}, binding_allocations=0, pipeline_creations=0",
                self.frame_index,
            );
            return Ok(self.finish_fixture());
        }
        self.update_window_title();
        Ok(FrameOutcome::Continue)
    }
}

impl PlatformEventHandler for HelloShaderApp {
    fn on_native_window_created(&mut self, window: Arc<NativeWindow>) -> PlatformResult<()> {
        let size = window.inner_size();
        self.window_size = [size.width.max(1) as f32, size.height.max(1) as f32];
        self.window = Some(window.clone());

        let mut renderer = WgpuBackend::for_window(window, size.width, size.height)?;
        renderer.upload_mesh(QUAD_MESH, &Mesh::quad());
        renderer.upload_mesh(TRIANGLE_MESH, &Mesh::triangle());
        renderer.upload_mesh(DIAMOND_MESH, &Mesh::diamond());
        renderer.upload_material(
            NEON_MATERIAL,
            &Material::new("shader-neon", Color::rgb(0.90, 0.36, 0.92)),
        )?;
        renderer.upload_material(
            INK_MATERIAL,
            &Material::new("shader-ink", Color::rgb(0.30, 0.82, 0.96)),
        )?;
        renderer.upload_material(
            HIGHLIGHT_MATERIAL,
            &Material::new("shader-highlight", Color::rgb(0.97, 0.86, 0.44)),
        )?;
        if self.include_backend_diagnostic_fixture {
            Self::register_backend_diagnostic_fixture(&mut renderer)?;
        }
        for variant in SHADER_VARIANTS {
            let asset = self
                .shader_store
                .allocate_with_source::<ShaderAsset, _>(variant.asset_label);
            let pipeline_declaration = Pipeline::custom_wgsl_module(
                variant.pipeline_label,
                shader_module_for_variant(variant)?,
            )?;
            let material_definition = tokimu::MaterialDefinition::solid_color(
                tokimu::MaterialDefinitionId::new(variant.pipeline_label)?,
                Color::rgb(1.0, 1.0, 1.0),
            );
            for mesh in [Mesh::quad(), Mesh::triangle(), Mesh::diamond()] {
                pipeline_declaration.validate_draw_contract(&material_definition, &mesh)?;
            }
            let pipeline = renderer.register_pipeline(&pipeline_declaration)?;
            self.shader_variants.push(ShaderVariantRuntime {
                definition: variant,
                asset,
                pipeline,
            });
        }
        self.renderer = Some(renderer);
        self.update_window_title();
        Ok(())
    }

    fn on_platform_event(&mut self, event: PlatformInputEvent) -> PlatformResult<()> {
        if let PlatformInputEvent::CloseRequested = event {
            return Ok(());
        }

        if let PlatformInputEvent::KeyboardInput { key, pressed } = event {
            if pressed {
                match key {
                    tokimu::KeyCode::Space | tokimu::KeyCode::ArrowRight => {
                        self.cycle_shader_variant(1)
                    }
                    tokimu::KeyCode::ArrowLeft => self.cycle_shader_variant(-1),
                    _ => {}
                }
                self.update_window_title();
            }
        }

        if let PlatformInputEvent::Resized { width, height } = event {
            self.window_size = [width.max(1) as f32, height.max(1) as f32];
            if let Some(renderer) = self.renderer.as_mut() {
                renderer.resize_surface(width, height);
            }
        }

        Ok(())
    }

    fn on_frame(&mut self, delta_seconds: f64) -> PlatformResult<FrameOutcome> {
        self.elapsed_seconds += delta_seconds;
        self.render_scene()
    }
}
