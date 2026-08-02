use std::{fs, path::PathBuf, sync::Arc};

use tokimu::{
    run_window_with_app, Camera, CameraHandle, ClearCommand, Color, DrawMeshCommand, FrameOutcome,
    Instance2d, KeyCode, Material, MaterialDefinition, MaterialDefinitionId, MaterialHandle,
    MaterialParameterDeclaration, MaterialParameterKind, MaterialParameterValue, Mesh, MeshHandle,
    NativeWindow, Pipeline, PipelineHandle, PlatformEventHandler, PlatformInputEvent,
    PlatformResult, RenderCommand, Renderer, Rgba8TextureColorSpace, Rgba8TextureDescriptor,
    ShaderBindingDeclaration, ShaderBindingSource, ShaderModuleDefinition,
    ShaderModuleValidationError, ShaderVertexInput, ShaderVertexSemantic, TextureHandle,
    WgpuBackend, WindowConfig,
};
use tokimu_core::math::{Mat4, Vec3};
use visualizer_tools::{
    decode_pcm16_wav, encode_pcm16_wav_fixture, observe_pcm_analysis_timing, write_cpu_preview,
    PcmAnalysisBacklog, PcmAnalysisConfig, PcmAnalyzer, PcmBacklogOverflowPolicy, PcmFixture,
    SyntheticAudioFixture, SyntheticVisualizerConfig, SyntheticVisualizerInput,
    VisualizerPassGraph, VisualizerViewport,
};

const QUAD: MeshHandle = MeshHandle(1);
const MATERIAL: MaterialHandle = MaterialHandle(1);
const CAMERA: CameraHandle = CameraHandle(1);
const FIXED_STEP_SECONDS: f32 = 1.0 / 60.0;
const WGSL: &str = include_str!("../assets/visualizer.wgsl");

fn main() -> PlatformResult<()> {
    if std::env::args().any(|argument| argument == "--structural-fixture") {
        return run_structural_fixture();
    }
    if std::env::args().any(|argument| argument == "--write-artifacts") {
        return write_fixture_artifacts();
    }
    if std::env::args().any(|argument| argument == "--offscreen-probe") {
        return run_offscreen_probe();
    }
    if std::env::args().any(|argument| argument == "--pcm-fixtures") {
        return run_pcm_fixtures();
    }
    if std::env::args().any(|argument| argument == "--wav-fixtures") {
        return run_wav_fixtures();
    }
    if std::env::args().any(|argument| argument == "--pcm-backlog-fixture") {
        return run_pcm_backlog_fixture();
    }
    if std::env::args().any(|argument| argument == "--pcm-measure") {
        return run_pcm_measurement();
    }

    run_window_with_app(
        WindowConfig {
            title: "Tokimu Hello Audio Visualizer".into(),
            width: 1100,
            height: 720,
        },
        App::new(),
    )
}

fn write_fixture_artifacts() -> PlatformResult<()> {
    let viewport = VisualizerViewport::new(640, 360)?;
    let output = PathBuf::from("target/audio-visualizer");
    fs::create_dir_all(&output)?;
    let pass_graph = VisualizerPassGraph::two_pass_signal(viewport.width, viewport.height);
    fs::write(
        output.join("two-pass-signal.graph.json"),
        format!("{}\n", pass_graph.to_structural_json()?),
    )?;
    write_shader_contract_artifacts(&output)?;
    for fixture in PcmFixture::ALL {
        let window = fixture.window();
        let analysis = PcmAnalyzer::analyze(&window, PcmAnalysisConfig::default())?;
        let stem = fixture.label();
        fs::write(
            output.join(format!("{stem}.analysis.json")),
            format!("{}\n", analysis.to_structural_json()?),
        )?;
        let timing = observe_pcm_analysis_timing(&window, PcmAnalysisConfig::default(), 32)?;
        fs::write(
            output.join(format!("{stem}.timing.observation.json")),
            format!("{}\n", timing.to_observation_json()?),
        )?;
        let wav = encode_pcm16_wav_fixture(fixture);
        let decoded_window = decode_pcm16_wav(&wav)?;
        let decoded_analysis = PcmAnalyzer::analyze(&decoded_window, PcmAnalysisConfig::default())?;
        fs::write(output.join(format!("{stem}.pcm16.wav")), &wav)?;
        fs::write(
            output.join(format!("{stem}.pcm16.analysis.json")),
            format!("{}\n", decoded_analysis.to_structural_json()?),
        )?;
        fs::write(
            output.join(format!("{stem}.pcm16.source.txt")),
            format!(
                "schema=tokimu-audio-source-fixture-v1\n\
source_kind=generated-riff-wave-pcm16-little-endian\n\
fixture={stem}\n\
encoded_file={stem}.pcm16.wav\n\
decoded_analysis_file={stem}.pcm16.analysis.json\n\
source_bytes={}\n\
source_fingerprint=fnv1a64:{:016x}\n\
sample_rate_hz={}\n\
channels={}\n\
frames={}\n\
provider_scope=corpus-byte-source-adapter-not-playback-or-capture\n",
                wav.len(),
                fnv1a64(&wav),
                decoded_window.sample_rate_hz,
                decoded_window.channels,
                decoded_window.frame_count(),
            ),
        )?;
        println!("wrote {stem} PCM analysis evidence");
    }
    fs::write(
        output.join("pcm-backlog-drop-oldest.json"),
        format!("{}\n", pcm_backlog_fixture()?.to_structural_json()?),
    )?;
    println!("wrote bounded PCM backlog evidence");
    for fixture in SyntheticAudioFixture::ALL {
        let source = SyntheticVisualizerInput::new(fixture, SyntheticVisualizerConfig::default())?;
        let frame = source.frame(90, viewport)?;
        let stem = fixture.label();
        fs::write(
            output.join(format!("{stem}.input.json")),
            format!("{}\n", frame.to_structural_json()?),
        )?;
        write_cpu_preview(
            output.join(format!("{stem}.bmp")),
            output.join(format!("{stem}.preview.txt")),
            &frame,
        )?;
        println!("wrote {stem} input and CPU preview evidence");
    }
    println!("wrote validated two-pass render-target graph evidence");
    Ok(())
}

/// Emits source and semantic contract evidence without claiming that the
/// backend accepts arbitrary runtime material parameters yet.
fn write_shader_contract_artifacts(output: &std::path::Path) -> PlatformResult<()> {
    let module = visualizer_shader_module()?;
    let material = visualizer_material_definition()?;
    let source_fingerprint = fnv1a64(module.source.as_bytes());

    fs::write(
        output.join("audio-visualizer-single-pass.wgsl"),
        &module.source,
    )?;
    fs::write(
        output.join("audio-visualizer-single-pass.contract.txt"),
        format!(
            "schema=tokimu-visualizer-shader-contract-v1\n\
shader_label={}\n\
source_file=audio-visualizer-single-pass.wgsl\n\
source_fingerprint=fnv1a64:{source_fingerprint:016x}\n\
vertex_entry_point={}\n\
fragment_entry_point={}\n\
material_id={}\n\
material_parameter=visualizer_signal\n\
material_parameter_kind=vector4\n\
binding_0_0=material-parameter:visualizer_signal:vector4\n\
binding_1_0=instance-transform\n\
binding_2_0=camera\n\
vertex_input_0=position3\n\
execution_bridge=legacy-four-float-material-slot\n\
arbitrary-vector4_execution=not-claimed\n\
native_screenshot=manual-evidence-only\n",
            module.label,
            module.vertex_entry_point,
            module.fragment_entry_point,
            material.id.as_str(),
        ),
    )?;
    println!("wrote visualizer shader and binding contract evidence");
    Ok(())
}

fn fnv1a64(bytes: &[u8]) -> u64 {
    bytes.iter().fold(0xcbf2_9ce4_8422_2325_u64, |hash, byte| {
        (hash ^ u64::from(*byte)).wrapping_mul(0x0000_0100_0000_01b3)
    })
}

fn run_structural_fixture() -> PlatformResult<()> {
    let viewport = VisualizerViewport::new(640, 360)?;
    let pass_graph = VisualizerPassGraph::two_pass_signal(viewport.width, viewport.height);
    println!("--- two-pass-signal graph ---");
    println!("{}", pass_graph.to_structural_json()?);
    print_pcm_fixtures()?;
    for fixture in SyntheticAudioFixture::ALL {
        let source = SyntheticVisualizerInput::new(fixture, SyntheticVisualizerConfig::default())?;
        let frame = source.frame(90, viewport)?;
        println!("--- {} ---", fixture.label());
        println!("{}", frame.to_structural_json()?);
    }
    Ok(())
}

fn run_pcm_fixtures() -> PlatformResult<()> {
    print_pcm_fixtures()
}

fn run_wav_fixtures() -> PlatformResult<()> {
    for fixture in PcmFixture::ALL {
        let wav = encode_pcm16_wav_fixture(fixture);
        let window = decode_pcm16_wav(&wav)?;
        let analysis = PcmAnalyzer::analyze(&window, PcmAnalysisConfig::default())?;
        println!("--- {} / generated PCM16 WAVE ---", fixture.label());
        println!(
            "bytes={}, sample_rate_hz={}, channels={}, frames={}, spectrum_bins={}",
            wav.len(),
            window.sample_rate_hz,
            window.channels,
            window.frame_count(),
            analysis.spectrum.len(),
        );
    }
    Ok(())
}

fn run_pcm_backlog_fixture() -> PlatformResult<()> {
    println!("{}", pcm_backlog_fixture()?.to_structural_json()?);
    Ok(())
}

fn run_pcm_measurement() -> PlatformResult<()> {
    for fixture in PcmFixture::ALL {
        let observation =
            observe_pcm_analysis_timing(&fixture.window(), PcmAnalysisConfig::default(), 32)?;
        println!("--- {} ---", fixture.label());
        println!("{}", observation.to_observation_json()?);
    }
    Ok(())
}

fn pcm_backlog_fixture() -> PlatformResult<visualizer_tools::PcmBacklogSnapshot> {
    let mut backlog = PcmAnalysisBacklog::new(2, PcmBacklogOverflowPolicy::DropOldest)?;
    for fixture in [
        PcmFixture::Silence,
        PcmFixture::Impulse,
        PcmFixture::ToneAtBin8,
    ] {
        backlog.push(fixture.window())?;
    }
    Ok(backlog.snapshot())
}

fn print_pcm_fixtures() -> PlatformResult<()> {
    for fixture in PcmFixture::ALL {
        let analysis = PcmAnalyzer::analyze(&fixture.window(), PcmAnalysisConfig::default())?;
        println!("--- {} ---", fixture.label());
        println!("{}", analysis.to_structural_json()?);
    }
    Ok(())
}

// This proves native, headless allocation of a renderer-owned target. It does
// not claim that pass routing, feedback, or GPU framebuffer equivalence exists.
fn run_offscreen_probe() -> PlatformResult<()> {
    let viewport = VisualizerViewport::new(640, 360)?;
    let mut renderer = WgpuBackend::new()?;
    let descriptor = Rgba8TextureDescriptor::new(
        viewport.width,
        viewport.height,
        Rgba8TextureColorSpace::Srgb,
    );
    renderer.create_render_target_rgba8(TextureHandle(2), descriptor)?;
    println!(
        "hello-audio-visualizer: allocated headless sampleable render target {}x{} (sRGB)",
        descriptor.width, descriptor.height
    );
    Ok(())
}

fn visualizer_shader_module() -> Result<ShaderModuleDefinition, ShaderModuleValidationError> {
    ShaderModuleDefinition::new(
        "audio-visualizer-single-pass",
        WGSL,
        "vs_main",
        "fs_main",
        vec![
            ShaderBindingDeclaration::new(
                0,
                0,
                ShaderBindingSource::MaterialParameter {
                    parameter: "visualizer_signal".to_owned(),
                    kind: MaterialParameterKind::Vector4,
                },
            ),
            ShaderBindingDeclaration::new(1, 0, ShaderBindingSource::InstanceTransform),
            ShaderBindingDeclaration::new(2, 0, ShaderBindingSource::Camera),
        ],
        vec![ShaderVertexInput::new(0, ShaderVertexSemantic::Position3)],
    )
}

fn visualizer_material_definition() -> PlatformResult<MaterialDefinition> {
    MaterialDefinition::new(
        MaterialDefinitionId::new("audio-visualizer")?,
        [MaterialParameterDeclaration::new(
            "visualizer_signal",
            MaterialParameterKind::Vector4,
            MaterialParameterValue::Vector4([0.0; 4]),
        )?],
    )
    .map_err(Into::into)
}

struct App {
    renderer: Option<WgpuBackend>,
    window: Option<Arc<NativeWindow>>,
    pipeline: PipelineHandle,
    viewport: VisualizerViewport,
    source: SyntheticVisualizerInput,
    fixture_index: usize,
    visualizer_frame: u64,
    step_accumulator: f32,
    time_scale: f32,
    paused: bool,
}

impl App {
    fn new() -> Self {
        let source = SyntheticVisualizerInput::new(
            SyntheticAudioFixture::SteadyTone,
            SyntheticVisualizerConfig::default(),
        )
        .expect("default synthetic visualizer configuration is valid");
        Self {
            renderer: None,
            window: None,
            pipeline: PipelineHandle(0),
            viewport: VisualizerViewport::new(1, 1).expect("unit viewport is valid"),
            source,
            fixture_index: 2,
            visualizer_frame: 0,
            step_accumulator: 0.0,
            time_scale: 1.0,
            paused: false,
        }
    }

    fn reset(&mut self) {
        self.visualizer_frame = 0;
        self.step_accumulator = 0.0;
    }

    fn cycle_fixture(&mut self, direction: isize) {
        let count = SyntheticAudioFixture::ALL.len() as isize;
        self.fixture_index = (self.fixture_index as isize + direction).rem_euclid(count) as usize;
        self.source
            .set_fixture(SyntheticAudioFixture::ALL[self.fixture_index]);
        self.reset();
    }

    fn advance(&mut self, delta_seconds: f64) {
        if self.paused {
            return;
        }
        self.step_accumulator += (delta_seconds as f32).clamp(0.0, 0.1) * self.time_scale;
        while self.step_accumulator >= FIXED_STEP_SECONDS {
            self.visualizer_frame = self.visualizer_frame.saturating_add(1);
            self.step_accumulator -= FIXED_STEP_SECONDS;
        }
    }

    fn render(&mut self) -> PlatformResult<FrameOutcome> {
        let observation = self.source.frame(self.visualizer_frame, self.viewport)?;
        let signal = observation.shader_signal();
        let aspect = self.viewport.width as f32 / self.viewport.height as f32;
        let Some(renderer) = self.renderer.as_mut() else {
            return Ok(FrameOutcome::Continue);
        };

        renderer.begin_frame();
        // This replacement is deliberately visible corpus pressure. The semantic
        // model already supports Vector4 parameters, while the current execution
        // material exposes only its legacy vec4 color slot.
        renderer.upload_material(
            MATERIAL,
            &Material::new(
                "visualizer-signal-phase-low-mid-high",
                Color::rgba(signal[0], signal[1], signal[2], signal[3]),
            ),
        )?;

        let mut camera = Camera::orthographic_2d_with_height(
            self.viewport.width as f32,
            self.viewport.height as f32,
            2.0,
        );
        camera.view = Mat4::from_translation(Vec3::ZERO);
        renderer.upload_camera(CAMERA, camera);
        renderer.set_active_camera(CAMERA);
        renderer.submit(&[
            RenderCommand::Clear(ClearCommand {
                color: Color::rgb(0.015, 0.025, 0.045),
            }),
            RenderCommand::DrawMesh(DrawMeshCommand {
                mesh: QUAD,
                material: MATERIAL,
                pipeline: self.pipeline,
                instance: Instance2d::identity().with_scale([aspect, 1.0]),
                camera: Some(CAMERA),
                viewport: None,
            }),
        ]);
        let stats = renderer.present()?;
        if self.visualizer_frame.is_multiple_of(120) {
            println!(
                "hello-audio-visualizer frame {}: fixture={}, time={:.3}, bands=[{:.3},{:.3},{:.3}], beat={:.3}, draws={}, binding_allocations={}, uniform_writes={}",
                self.visualizer_frame,
                observation.fixture.label(),
                observation.time_seconds,
                signal[1],
                signal[2],
                signal[3],
                observation.beat.pulse,
                stats.frame.draw_calls,
                stats.frame.binding_allocations,
                stats.frame.uniform_buffer_writes,
            );
        }
        self.update_title();
        Ok(FrameOutcome::Continue)
    }

    fn update_title(&self) {
        if let Some(window) = self.window.as_ref() {
            window.set_title(&format!(
                "Tokimu Audio Visualizer | {} | frame={} | {:.2}x | {} | Left/Right fixture | Space pause | Up/Down speed | R reset",
                self.source.fixture().label(),
                self.visualizer_frame,
                self.time_scale,
                if self.paused { "paused" } else { "running" },
            ));
        }
    }
}

impl PlatformEventHandler for App {
    fn on_native_window_created(&mut self, window: Arc<NativeWindow>) -> PlatformResult<()> {
        let size = window.inner_size();
        self.viewport = VisualizerViewport::new(size.width.max(1), size.height.max(1))?;
        self.window = Some(window.clone());

        let mut renderer = WgpuBackend::for_window(window, size.width, size.height)?;
        renderer.upload_mesh(QUAD, &Mesh::quad());
        renderer.upload_material(MATERIAL, &Material::new("visualizer-signal", Color::BLACK))?;
        let pipeline = Pipeline::custom_wgsl_module(
            "audio-visualizer-single-pass",
            visualizer_shader_module()?,
        )?;
        pipeline.validate_draw_contract(&visualizer_material_definition()?, &Mesh::quad())?;
        self.pipeline = renderer.register_pipeline(&pipeline)?;
        self.renderer = Some(renderer);
        self.update_title();
        Ok(())
    }

    fn on_platform_event(&mut self, event: PlatformInputEvent) -> PlatformResult<()> {
        match event {
            PlatformInputEvent::Resized { width, height } => {
                self.viewport = VisualizerViewport::new(width.max(1), height.max(1))?;
                if let Some(renderer) = self.renderer.as_mut() {
                    renderer.resize_surface(width, height);
                }
            }
            PlatformInputEvent::KeyboardInput { key, pressed: true } => match key {
                KeyCode::ArrowRight => self.cycle_fixture(1),
                KeyCode::ArrowLeft => self.cycle_fixture(-1),
                KeyCode::ArrowUp => self.time_scale = (self.time_scale * 2.0).min(4.0),
                KeyCode::ArrowDown => self.time_scale = (self.time_scale * 0.5).max(0.25),
                KeyCode::Space => self.paused = !self.paused,
                KeyCode::KeyR => self.reset(),
                _ => {}
            },
            _ => {}
        }
        self.update_title();
        Ok(())
    }

    fn on_frame(&mut self, delta_seconds: f64) -> PlatformResult<FrameOutcome> {
        self.advance(delta_seconds);
        self.render()
    }
}

#[cfg(test)]
mod tests {
    use super::{visualizer_material_definition, visualizer_shader_module};
    use tokimu::{
        Color, MaterialDefinition, MaterialDefinitionId, MaterialParameterDeclaration,
        MaterialParameterKind, MaterialParameterValue, Mesh, Pipeline, PipelineDrawContractError,
        ShaderDiagnosticStage, ShaderMaterialCompatibilityError,
    };

    #[test]
    fn visualizer_contract_passes_before_backend_submission() {
        let pipeline = Pipeline::custom_wgsl_module(
            "audio-visualizer-single-pass",
            visualizer_shader_module().unwrap(),
        )
        .unwrap();

        assert_eq!(
            pipeline
                .validate_draw_contract(&visualizer_material_definition().unwrap(), &Mesh::quad()),
            Ok(())
        );
    }

    #[test]
    fn visualizer_binding_kind_failure_reports_draw_contract_stage() {
        let pipeline = Pipeline::custom_wgsl_module(
            "audio-visualizer-single-pass",
            visualizer_shader_module().unwrap(),
        )
        .unwrap();
        let incompatible_material = MaterialDefinition::new(
            MaterialDefinitionId::new("incompatible-visualizer").unwrap(),
            [MaterialParameterDeclaration::new(
                "visualizer_signal",
                MaterialParameterKind::Color,
                MaterialParameterValue::Color(Color::BLACK),
            )
            .unwrap()],
        )
        .unwrap();

        let error = pipeline
            .validate_draw_contract(&incompatible_material, &Mesh::quad())
            .unwrap_err();
        assert_eq!(error.stage(), ShaderDiagnosticStage::DrawContractValidation);
        assert!(matches!(
            error,
            PipelineDrawContractError::Material(
                ShaderMaterialCompatibilityError::MaterialParameterKindMismatch { .. }
            )
        ));
    }
}
