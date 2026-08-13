use std::{fs, io, path::PathBuf, sync::Arc, time::Instant};

use gltf_corpus::{decode_glb_file, DecodedPrimitive};
use presentation_control::{
    PresentationColor, PresentationControl, PresentationEmphasis, PresentationLayer,
    PresentationOverride, PresentationTargetDescriptor, PresentationTargetId,
    PresentationTargetKind, PresentationTint, ResolvedPresentation, SourcePresentation,
};
use tokimu::{
    run_window_with_app, BlendMode, Camera, CameraHandle, ClearCommand, Color, ColorWriteMask,
    CullMode, DepthTest, Diagnostics, DrawMeshCommand, DrawMeshMaterialOverrideCommand,
    FrameOutcome, Instance2d, KeyCode, Material, MaterialDefinition, MaterialDefinitionId,
    MaterialHandle, MaterialInstance, MaterialOverride, Mesh, MeshHandle, NativeWindow,
    PerformanceBudget, PerformanceMonitor, PerformanceUnit, Pipeline, PipelineHandle, PipelineKind,
    PipelineRenderState, PlatformEventHandler, PlatformInputEvent, PlatformResult, RenderCommand,
    Renderer, WgpuBackend, WindowConfig,
};
use tokimu_assets::{AssetLifecycleObservation, AssetStore};
use tokimu_core::math::{Mat4, Vec3};

const MODEL_MESH: MeshHandle = MeshHandle(1);
const FLOOR_MESH: MeshHandle = MeshHandle(2);
const MODEL_MATERIAL: MaterialHandle = MaterialHandle(1);
const FLOOR_MATERIAL: MaterialHandle = MaterialHandle(2);
const CAMERA_HANDLE: CameraHandle = CameraHandle(1);
const KHRONOS_BOX_SOURCE: &str =
    "third-party/fixtures/khronos-gltf-sample-assets/upstream/Models/Box/glTF-Binary/Box.glb";
const MODEL_TARGET_KEY: &str = "khronos-box/node/0/mesh/0/primitive/0";

fn main() -> PlatformResult<()> {
    let arguments = std::env::args().skip(1).collect::<Vec<_>>();
    if arguments
        .iter()
        .any(|argument| argument != "--transparent" && argument != "--measure-two-frames")
    {
        return Err("usage: hello-glb [--transparent] [--measure-two-frames]".into());
    }
    let mut app = HelloGlbApp::new();
    if arguments.iter().any(|argument| argument == "--transparent") {
        app.activate_transparent_inspection()?;
    }
    app.exit_after_two_frames = arguments
        .iter()
        .any(|argument| argument == "--measure-two-frames");
    run_window_with_app(
        WindowConfig {
            title: "Tokimu Hello GLB".into(),
            width: 1280,
            height: 720,
        },
        app,
    )
}

struct HelloGlbApp {
    renderer: Option<WgpuBackend>,
    window: Option<Arc<NativeWindow>>,
    window_size: [f32; 2],
    elapsed_seconds: f64,
    pipeline: PipelineHandle,
    transparent_pipeline: PipelineHandle,
    assets: AssetStore,
    asset_lifecycle: Vec<AssetLifecycleObservation>,
    model_mesh: Mesh,
    presentation: PresentationControl,
    model_target: PresentationTargetId,
    model_material_definition: MaterialDefinition,
    model_material_instance: MaterialInstance,
    model_material_override: Option<MaterialOverride>,
    presentation_step: usize,
    fixed_capture: bool,
    presentation_frames_since_change: u32,
    model_visible: bool,
    frame_index: u64,
    /// Corpus-only bounded native measurement mode. It does not affect normal
    /// interactive presentation or define a renderer lifecycle policy.
    exit_after_two_frames: bool,
    diagnostics: Diagnostics,
    frame_time_monitor: PerformanceMonitor,
    present_time_monitor: PerformanceMonitor,
}

impl Default for HelloGlbApp {
    fn default() -> Self {
        let model_target =
            PresentationTargetId::new(PresentationTargetKind::MeshPrimitive, MODEL_TARGET_KEY)
                .expect("static GLB presentation target should be valid");
        let mut presentation = PresentationControl::default();
        presentation
            .register_target_with_descriptor(
                PresentationTargetDescriptor::new(model_target.clone())
                    .with_source_name("Box")
                    .expect("static GLB source name should be valid"),
                SourcePresentation::new(presentation_color(0.86, 0.79, 0.72), 1.0, true)
                    .expect("static GLB source presentation should be valid"),
            )
            .expect("GLB presentation target should register once");
        let model_material_definition = MaterialDefinition::solid_color(
            MaterialDefinitionId::new("hello-glb-model")
                .expect("static GLB material definition should be valid"),
            Color::rgb(0.86, 0.79, 0.72),
        );
        let model_material_instance = MaterialInstance::from_definition(&model_material_definition);

        Self {
            renderer: None,
            window: None,
            window_size: [1.0, 1.0],
            elapsed_seconds: 0.0,
            pipeline: PipelineHandle(0),
            transparent_pipeline: PipelineHandle(0),
            assets: AssetStore::default(),
            asset_lifecycle: Vec::new(),
            model_mesh: Mesh::default(),
            presentation,
            model_target,
            model_material_definition,
            model_material_instance,
            model_material_override: None,
            presentation_step: 0,
            fixed_capture: false,
            presentation_frames_since_change: 0,
            model_visible: true,
            frame_index: 0,
            exit_after_two_frames: false,
            diagnostics: Diagnostics::default(),
            frame_time_monitor: PerformanceMonitor::new(
                PerformanceBudget::new(
                    "hello-glb",
                    "platform-reported frame interval",
                    25.0,
                    PerformanceUnit::Milliseconds,
                )
                .with_required_consecutive_violations(3),
            ),
            present_time_monitor: PerformanceMonitor::new(
                PerformanceBudget::new(
                    "hello-glb.renderer",
                    "renderer present call CPU wall duration",
                    16.0,
                    PerformanceUnit::Milliseconds,
                )
                .with_required_consecutive_violations(3),
            ),
        }
    }
}

impl HelloGlbApp {
    fn new() -> Self {
        Self::default()
    }

    /// Corpus-only fixed entry point for AR-0023 visual evidence. Interactive
    /// use still cycles through the same application-owned presentation state.
    fn activate_transparent_inspection(&mut self) -> PlatformResult<()> {
        self.presentation_step = 2;
        self.fixed_capture = true;
        self.cycle_model_presentation()
    }

    fn update_window_title(&self) {
        if let Some(window) = self.window.as_ref() {
            let inventory = self.assets.inventory();
            let source = inventory
                .entries
                .first()
                .and_then(|entry| entry.source.as_deref())
                .unwrap_or("models/cube.glb");
            window.set_title(&format!(
                "Tokimu Hello GLB | source={} | opaque-cull=back | presentation={} | E cycles | elapsed={:.1}s",
                source,
                presentation_step_name(self.presentation_step),
                self.elapsed_seconds
            ));
        }
    }

    fn cycle_model_presentation(&mut self) -> PlatformResult<()> {
        self.presentation_step = (self.presentation_step + 1) % 5;
        self.presentation_frames_since_change = 0;
        self.presentation
            .clear_target_overrides(&self.model_target)?;

        let override_value = match self.presentation_step {
            0 => None,
            1 => Some(
                PresentationOverride::default()
                    .with_tint(PresentationTint::replace(presentation_color(
                        0.38, 0.68, 0.96,
                    )))
                    .with_emphasis(PresentationEmphasis::Selected),
            ),
            2 => Some(
                PresentationOverride::default()
                    .with_tint(PresentationTint::replace(presentation_color(
                        1.0, 0.35, 0.10,
                    )))
                    .with_emphasis(PresentationEmphasis::Hotspot),
            ),
            3 => Some(
                PresentationOverride::default()
                    .with_tint(PresentationTint::multiply(presentation_color(
                        0.65, 0.90, 1.0,
                    )))
                    .with_opacity_multiplier(0.35)?,
            ),
            4 => Some(PresentationOverride::default().with_visibility(false)),
            _ => unreachable!("presentation step is reduced modulo five"),
        };
        if let Some(override_value) = override_value {
            self.presentation.set_override(
                &self.model_target,
                PresentationLayer::Application,
                override_value,
            )?;
        }

        self.refresh_model_presentation()?;
        self.update_window_title();
        Ok(())
    }

    fn refresh_model_presentation(&mut self) -> PlatformResult<()> {
        let resolved = self.presentation.resolve(&self.model_target)?;
        self.model_visible = resolved.visible;
        self.model_material_override = (self.presentation_step != 0)
            .then(|| resolved_to_material_override(resolved))
            .transpose()?;
        self.write_presentation_artifact(resolved)?;
        Ok(())
    }

    /// Writes a structural corpus artifact for review without claiming that it
    /// is a GPU framebuffer capture or a renderer-owned cache record.
    fn write_presentation_artifact(&self, resolved: ResolvedPresentation) -> PlatformResult<()> {
        let artifact = self.presentation_artifact_json(resolved)?;
        let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../../..")
            .join("target/hello-glb");
        fs::create_dir_all(&root)?;
        fs::write(root.join("presentation-state.json"), artifact)?;
        Ok(())
    }

    fn presentation_artifact_json(&self, resolved: ResolvedPresentation) -> PlatformResult<String> {
        let source = self.source_model_material()?;
        let selected_pipeline = if resolved.opacity < 1.0 {
            "glb-transparent-pipeline"
        } else {
            "glb-pipeline"
        };
        let artifact = format!(
            concat!(
                "{{\n",
                "  \"schema\": 1,\n",
                "  \"example\": \"hello-glb\",\n",
                "  \"source\": \"{}\",\n",
                "  \"target\": {{\"kind\": \"meshPrimitive\", \"key\": \"{}\"}},\n",
                "  \"state\": \"{}\",\n",
                "  \"sourceMaterial\": {{\"label\": \"{}\", \"baseColor\": [{:.6}, {:.6}, {:.6}, {:.6}]}},\n",
                "  \"resolvedPresentation\": {{\"color\": [{:.6}, {:.6}, {:.6}], \"opacity\": {:.6}, \"visible\": {}, \"emphasis\": {}}},\n",
                "  \"pipeline\": \"{}\",\n",
                "  \"transparency\": {{\"blend\": \"alphaBlend\", \"depthTest\": \"lessEqual\", \"depthWrite\": {}, \"ordering\": \"submissionOrder; intersecting transparent geometry is not guaranteed\"}}\n",
                "}}\n"
            ),
            KHRONOS_BOX_SOURCE,
            MODEL_TARGET_KEY,
            presentation_step_name(self.presentation_step),
            source.label,
            source.base_color.r,
            source.base_color.g,
            source.base_color.b,
            source.base_color.a,
            resolved.color.red,
            resolved.color.green,
            resolved.color.blue,
            resolved.opacity,
            resolved.visible,
            resolved
                .emphasis
                .map(|value| format!("\"{value:?}\""))
                .unwrap_or_else(|| "null".to_owned()),
            selected_pipeline,
            resolved.opacity >= 1.0,
        );
        Ok(artifact)
    }

    fn source_model_material(&self) -> PlatformResult<Material> {
        self.model_material_definition
            .lower_to_legacy_material(&self.model_material_instance, "glb-model")
            .map_err(Into::into)
    }

    fn render_scene(&mut self, delta_seconds: f64) -> PlatformResult<FrameOutcome> {
        let seconds = if self.fixed_capture {
            0.0
        } else {
            self.elapsed_seconds as f32
        };
        let mut camera = Camera::perspective_3d(self.window_size[0], self.window_size[1]);
        let orbit = seconds * 0.28;
        let eye = Vec3::new(
            orbit.cos() * 4.75,
            1.8 + orbit.sin() * 0.15,
            orbit.sin() * 4.75,
        );
        camera.view =
            tokimu_core::math::try_view_look_at_rh(eye, Vec3::new(0.0, 0.35, 0.0), Vec3::Y)
                .expect("camera basis must be finite and non-degenerate");
        let mut commands = vec![
            RenderCommand::Clear(ClearCommand {
                color: Color::rgb(0.05, 0.07, 0.11),
            }),
            RenderCommand::DrawMesh(DrawMeshCommand {
                mesh: FLOOR_MESH,
                material: FLOOR_MATERIAL,
                pipeline: self.pipeline,
                instance: Instance2d::identity(),
                camera: Some(CAMERA_HANDLE),
                viewport: None,
            }),
        ];
        let mut selected_pipeline = self.pipeline;
        if self.model_visible {
            let draw = DrawMeshCommand {
                mesh: MODEL_MESH,
                material: MODEL_MATERIAL,
                pipeline: if self
                    .model_material_override
                    .is_some_and(|override_value| override_value.opacity_multiplier() < 1.0)
                {
                    self.transparent_pipeline
                } else {
                    self.pipeline
                },
                instance: Instance2d::identity(),
                camera: Some(CAMERA_HANDLE),
                viewport: None,
            };
            selected_pipeline = draw.pipeline;
            commands.push(match self.model_material_override {
                Some(material_override) => {
                    RenderCommand::DrawMeshMaterialOverride(DrawMeshMaterialOverrideCommand {
                        draw,
                        material_override,
                    })
                }
                None => RenderCommand::DrawMesh(draw),
            });
        }

        let diagnostic_count = self.diagnostics.records().len();
        let present_start = Instant::now();
        let stats = {
            let Some(renderer) = self.renderer.as_mut() else {
                return Ok(FrameOutcome::Continue);
            };
            // Frame counters intentionally cover resource work as well as
            // submission/presentation. Starting the frame before replacement
            // uploads makes this corpus's retained telemetry match the work it
            // performs; it does not change renderer lifetime semantics.
            renderer.begin_frame();
            renderer.upload_mesh(MODEL_MESH, &transform_model_mesh(&self.model_mesh, seconds));
            renderer.upload_mesh(FLOOR_MESH, &build_floor_mesh(seconds));
            renderer.upload_camera(CAMERA_HANDLE, camera);
            renderer.submit(&commands);
            renderer.present()?
        };
        let present_time = present_start.elapsed();
        self.frame_index = self.frame_index.saturating_add(1);
        self.frame_time_monitor
            .observe(delta_seconds * 1000.0, &mut self.diagnostics);
        self.present_time_monitor
            .observe(present_time.as_secs_f64() * 1000.0, &mut self.diagnostics);
        for diagnostic in &self.diagnostics.records()[diagnostic_count..] {
            eprintln!("{diagnostic}");
        }
        if self.presentation_frames_since_change >= 1
            && self.model_material_override.is_some()
            && stats.frame.binding_allocations != 0
        {
            println!(
                "warning [hello-glb.presentation]: unchanged presentation state allocated {} derived material binding(s)",
                stats.frame.binding_allocations
            );
        }
        self.presentation_frames_since_change =
            self.presentation_frames_since_change.saturating_add(1);
        if self.frame_index <= 3 || self.frame_index.is_multiple_of(120) {
            println!(
                "hello-glb frame {}: presentation={}, visible={}, pipeline={}, platform_frame_interval_ms={:.3}, renderer_present_call_cpu_ms={:.3}, draws={}, submits={}, binding_allocations={}, uniform_writes={}, mesh_uploads={}, mesh_replacements={}, lifetime_binding_allocations={}",
                self.frame_index,
                presentation_step_name(self.presentation_step),
                self.model_visible,
                selected_pipeline.0,
                delta_seconds * 1000.0,
                present_time.as_secs_f64() * 1000.0,
                stats.frame.draw_calls,
                stats.frame.submit_calls,
                stats.frame.binding_allocations,
                stats.frame.uniform_buffer_writes,
                stats.frame.mesh_uploads,
                stats.frame.mesh_replacements,
                stats.lifetime.binding_allocations,
            );
        }
        self.update_window_title();
        Ok(if self.exit_after_two_frames && self.frame_index >= 2 {
            FrameOutcome::Exit
        } else {
            FrameOutcome::Continue
        })
    }
}

impl PlatformEventHandler for HelloGlbApp {
    fn on_native_window_created(&mut self, window: Arc<NativeWindow>) -> PlatformResult<()> {
        let size = window.inner_size();
        self.window_size = [size.width.max(1) as f32, size.height.max(1) as f32];
        self.window = Some(window.clone());

        self.model_mesh = load_khronos_box_mesh()?;
        let (model_asset, allocated) = self
            .assets
            .allocate_with_source_observed::<Mesh, _>(KHRONOS_BOX_SOURCE);
        let prepared = self.assets.mark_prepared(model_asset)?;
        self.asset_lifecycle.extend([allocated, prepared]);
        for observation in &self.asset_lifecycle {
            println!(
                "hello-glb asset lifecycle {}: asset={} generation={} kind={:?} source={}",
                observation.sequence,
                observation.asset_id.0,
                observation.generation,
                observation.kind,
                observation.source.as_deref().unwrap_or("<unknown>"),
            );
        }

        let mut renderer = WgpuBackend::for_window(window, size.width, size.height)?;
        let model_material = self.source_model_material()?;
        renderer.upload_material(MODEL_MATERIAL, &model_material)?;
        renderer.upload_material(
            FLOOR_MATERIAL,
            &Material::new("glb-floor", Color::rgb(0.08, 0.10, 0.13)),
        )?;
        self.pipeline = renderer.register_pipeline(
            &Pipeline::new("glb-pipeline", PipelineKind::LitColor3d)
                .with_render_state(opaque_model_render_state())?,
        )?;
        self.transparent_pipeline = renderer.register_pipeline(
            &Pipeline::new("glb-transparent-pipeline", PipelineKind::LitColor3d)
                .with_render_state(transparent_render_state())?,
        )?;
        println!(
            "hello-glb renderer initialized: backend={}; device={}; adapter={}; transparent_pipeline={}",
            renderer.backend_api(),
            renderer.device_kind(),
            renderer.adapter_name(),
            self.transparent_pipeline.0,
        );
        self.renderer = Some(renderer);
        self.refresh_model_presentation()?;
        self.update_window_title();
        Ok(())
    }

    fn on_platform_event(&mut self, event: PlatformInputEvent) -> PlatformResult<()> {
        if let PlatformInputEvent::CloseRequested = event {
            return Ok(());
        }

        if let PlatformInputEvent::Resized { width, height } = event {
            self.window_size = [width.max(1) as f32, height.max(1) as f32];
            if let Some(renderer) = self.renderer.as_mut() {
                renderer.resize_surface(width, height);
            }
        }
        if let PlatformInputEvent::KeyboardInput {
            key: KeyCode::KeyE,
            pressed: true,
        } = event
        {
            self.cycle_model_presentation()?;
        }

        Ok(())
    }

    fn on_frame(&mut self, delta_seconds: f64) -> PlatformResult<FrameOutcome> {
        self.elapsed_seconds += delta_seconds;
        self.render_scene(delta_seconds)
    }
}

fn presentation_color(red: f32, green: f32, blue: f32) -> PresentationColor {
    PresentationColor::new(red, green, blue)
        .expect("hard-coded corpus presentation color should be valid")
}

fn resolved_to_material_override(
    resolved: ResolvedPresentation,
) -> Result<MaterialOverride, tokimu::MaterialModelError> {
    MaterialOverride::with_replacement_color(Color::rgb(
        resolved.color.red,
        resolved.color.green,
        resolved.color.blue,
    ))?
    .with_opacity_multiplier(resolved.opacity)
}

fn presentation_step_name(step: usize) -> &'static str {
    match step {
        0 => "source",
        1 => "selected",
        2 => "hotspot",
        3 => "transparent",
        4 => "hidden",
        _ => "unknown",
    }
}

/// The opaque Box path is the native culling proof: its source mesh retains
/// Khronos winding and back-face culling is deliberately enabled.
const fn opaque_model_render_state() -> PipelineRenderState {
    PipelineRenderState {
        blend: BlendMode::Opaque,
        depth_test: DepthTest::LessEqual,
        depth_write: true,
        cull_mode: CullMode::Back,
        color_write: ColorWriteMask::ALL,
    }
}

/// First-proof transparency policy: alpha blend against previously submitted
/// draws, with depth testing but no depth writes. Culling is intentionally
/// disabled only for this diagnostic presentation state: its opacity does not
/// establish the opaque model's front-face policy.
const fn transparent_render_state() -> PipelineRenderState {
    PipelineRenderState {
        blend: BlendMode::AlphaBlend,
        depth_test: DepthTest::LessEqual,
        depth_write: false,
        cull_mode: CullMode::None,
        color_write: ColorWriteMask::ALL,
    }
}

fn load_khronos_box_mesh() -> PlatformResult<Mesh> {
    let path = khronos_box_path();
    let model = decode_glb_file(&path)?;
    let primitive = model.primitives.first().ok_or_else(|| {
        io::Error::other("Khronos Box GLB decoded without a renderable primitive")
    })?;
    indexed_primitive_to_mesh(primitive)
}

fn khronos_box_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../../..")
        .join(KHRONOS_BOX_SOURCE)
}

fn indexed_primitive_to_mesh(primitive: &DecodedPrimitive) -> PlatformResult<Mesh> {
    if primitive.normals.len() != primitive.positions.len() {
        return Err(io::Error::other(format!(
            "primitive has {} positions but {} normals",
            primitive.positions.len(),
            primitive.normals.len()
        ))
        .into());
    }

    let mut positions = Vec::with_capacity(primitive.indices.len());
    let mut normals = Vec::with_capacity(primitive.indices.len());
    for index in &primitive.indices {
        let index = *index as usize;
        positions.push(*primitive.positions.get(index).ok_or_else(|| {
            io::Error::other(format!(
                "primitive index {index} is outside decoded positions"
            ))
        })?);
        normals.push(*primitive.normals.get(index).ok_or_else(|| {
            io::Error::other(format!(
                "primitive index {index} is outside decoded normals"
            ))
        })?);
    }

    Ok(Mesh::new(positions, normals))
}

fn transform_model_mesh(model: &Mesh, seconds: f32) -> Mesh {
    let wobble = (seconds * 1.4).sin() * 0.06;
    let twist = seconds * 0.18;
    let transform = Mat4::from_rotation_y(twist)
        * Mat4::from_rotation_x((seconds * 0.7).sin() * 0.15)
        * Mat4::from_scale(Vec3::new(
            1.0 + wobble * 0.5,
            1.0 + wobble,
            1.0 + wobble * 0.25,
        ))
        * Mat4::from_translation(Vec3::new(0.0, 0.35, 0.0));
    let normal_transform = transform.inverse().transpose();
    Mesh::new(
        model
            .positions
            .iter()
            .copied()
            .map(|position| {
                transform
                    .transform_point3(Vec3::from_array(position))
                    .to_array()
            })
            .collect(),
        model
            .normals
            .iter()
            .copied()
            .map(|normal| {
                normal_transform
                    .transform_vector3(Vec3::from_array(normal))
                    .normalize_or_zero()
                    .to_array()
            })
            .collect(),
    )
}

fn build_floor_mesh(seconds: f32) -> Mesh {
    let pulse = 0.02 + seconds.sin().abs() * 0.01;
    let transform = Mat4::from_translation(Vec3::new(0.0, -0.8, 0.0))
        * Mat4::from_scale(Vec3::new(8.0, pulse, 8.0));
    let normal_transform = transform.inverse().transpose();
    let base = Mesh::cube();

    Mesh::new(
        base.positions
            .into_iter()
            .map(|position| {
                transform
                    .transform_point3(Vec3::from_array(position))
                    .to_array()
            })
            .collect(),
        base.normals
            .into_iter()
            .map(|normal| {
                normal_transform
                    .transform_vector3(Vec3::from_array(normal))
                    .normalize_or_zero()
                    .to_array()
            })
            .collect(),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn presentation_cycle_preserves_source_material_and_restores_it() {
        let mut app = HelloGlbApp::default();
        let source_material = app.source_model_material().unwrap();

        app.cycle_model_presentation().unwrap();
        assert!(app.model_visible);
        assert_eq!(app.presentation_step, 1);
        assert_eq!(
            app.model_material_override
                .expect("selected state should use an override")
                .opacity_multiplier(),
            1.0
        );
        assert_eq!(app.source_model_material().unwrap(), source_material);

        app.cycle_model_presentation().unwrap();
        assert!(app.model_visible);
        assert_eq!(app.presentation_step, 2);
        assert_eq!(app.source_model_material().unwrap(), source_material);

        app.cycle_model_presentation().unwrap();
        assert!(app.model_visible);
        assert_eq!(app.presentation_step, 3);
        assert_eq!(
            app.model_material_override
                .expect("transparent state should use an override")
                .opacity_multiplier(),
            0.35
        );
        assert_eq!(app.source_model_material().unwrap(), source_material);

        app.cycle_model_presentation().unwrap();
        assert!(!app.model_visible);
        assert_eq!(app.presentation_step, 4);
        assert_eq!(app.source_model_material().unwrap(), source_material);

        app.cycle_model_presentation().unwrap();
        assert!(app.model_visible);
        assert_eq!(app.presentation_step, 0);
        assert!(app.model_material_override.is_none());
        assert_eq!(app.source_model_material().unwrap(), source_material);
    }

    #[test]
    fn transparent_capture_entry_selects_fixed_continuous_alpha_state() {
        let mut app = HelloGlbApp::default();
        app.activate_transparent_inspection().unwrap();

        assert!(app.fixed_capture);
        assert_eq!(app.presentation_step, 3);
        assert_eq!(
            app.model_material_override
                .expect("transparent capture should resolve an opacity override")
                .opacity_multiplier(),
            0.35
        );
    }

    #[test]
    fn presentation_artifact_records_the_resolved_state_and_pipeline_policy() {
        let mut app = HelloGlbApp {
            presentation_step: 3,
            ..HelloGlbApp::default()
        };
        app.presentation
            .set_override(
                &app.model_target,
                PresentationLayer::Application,
                PresentationOverride::default()
                    .with_opacity_multiplier(0.35)
                    .unwrap(),
            )
            .unwrap();
        let resolved = app.presentation.resolve(&app.model_target).unwrap();
        let artifact = app.presentation_artifact_json(resolved).unwrap();

        assert!(artifact.contains("\"target\": {\"kind\": \"meshPrimitive\""));
        assert!(artifact.contains("\"pipeline\": \"glb-transparent-pipeline\""));
        assert!(artifact.contains("intersecting transparent geometry is not guaranteed"));
    }

    #[test]
    fn transparent_pipeline_policy_is_explicit_and_avoids_depth_writes() {
        assert_eq!(
            transparent_render_state(),
            PipelineRenderState {
                blend: BlendMode::AlphaBlend,
                depth_test: DepthTest::LessEqual,
                depth_write: false,
                cull_mode: CullMode::None,
                color_write: ColorWriteMask::ALL,
            }
        );
    }

    #[test]
    fn opaque_box_pipeline_explicitly_culls_back_faces() {
        assert_eq!(
            opaque_model_render_state(),
            PipelineRenderState {
                blend: BlendMode::Opaque,
                depth_test: DepthTest::LessEqual,
                depth_write: true,
                cull_mode: CullMode::Back,
                color_write: ColorWriteMask::ALL,
            }
        );
    }

    #[test]
    fn corpus_boundaries_fail_at_their_owning_stage() {
        let app = HelloGlbApp::default();
        let unknown_target = PresentationTargetId::new(
            PresentationTargetKind::MeshPrimitive,
            "khronos-box/node/99/mesh/99/primitive/99",
        )
        .unwrap();
        assert!(matches!(
            app.presentation.resolve(&unknown_target),
            Err(presentation_control::PresentationControlError::UnknownTarget { .. })
        ));

        let other_definition = MaterialDefinition::solid_color(
            MaterialDefinitionId::new("hello-glb-other").unwrap(),
            Color::rgb(1.0, 1.0, 1.0),
        );
        let other_instance = MaterialInstance::from_definition(&other_definition);
        assert!(matches!(
            app.model_material_definition
                .lower_to_legacy_material(&other_instance, "invalid-material"),
            Err(tokimu::MaterialModelError::DefinitionMismatch { .. })
        ));

        let invalid_pipeline =
            Pipeline::custom_wgsl_with_entry_points("missing-source", "", "vs_main", "fs_main");
        assert!(matches!(
            invalid_pipeline.validate(),
            Err(tokimu::PipelineValidationError::MissingCustomShaderSource { .. })
        ));

        let resolved = app.presentation.resolve(&app.model_target).unwrap();
        let output = app.presentation_artifact_json(resolved).unwrap();
        assert!(output.contains("\"resolvedPresentation\""));
    }
}
