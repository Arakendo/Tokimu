use std::{fs, path::PathBuf, sync::Arc};

use tokimu::{
    run_window_with_app, Camera, CameraHandle, ClearCommand, Color, DrawMeshCommand, FrameOutcome,
    Instance2d, KeyCode, Material, MaterialDefinition, MaterialDefinitionId, MaterialHandle,
    MaterialParameterDeclaration, MaterialParameterKind, MaterialParameterValue, Mesh, MeshHandle,
    NativeWindow, Pipeline, PipelineHandle, PlatformEventHandler, PlatformInputEvent,
    PlatformResult, RenderCommand, RenderFrameStats, Renderer, Rgba8TextureColorSpace,
    Rgba8TextureDescriptor, ShaderBindingDeclaration, ShaderBindingSource, ShaderModuleDefinition,
    ShaderModuleValidationError, ShaderVertexInput, ShaderVertexSemantic, TextureHandle,
    WgpuBackend, WindowConfig,
};
use tokimu_core::math::{Mat4, Vec3};
use visualizer_tools::{
    decode_pcm16_wav, encode_pcm16_wav_fixture, observe_pcm_analysis_timing,
    observe_pcm_analysis_working_set, write_cpu_preview, NativeVisualizerDefinition,
    NativeVisualizerKind, PcmAnalysisBacklog, PcmAnalysisConfig, PcmAnalyzer,
    PcmBacklogOverflowPolicy, PcmFixture, SyntheticAudioFixture, SyntheticVisualizerConfig,
    SyntheticVisualizerInput, VisualizerPassGraph, VisualizerSpectrumBars, VisualizerViewport,
    VisualizerWaveform,
};

const QUAD: MeshHandle = MeshHandle(1);
const FEEDBACK_FROM_HISTORY_A: MaterialHandle = MaterialHandle(1);
const FEEDBACK_FROM_HISTORY_B: MaterialHandle = MaterialHandle(2);
const PRESENT_HISTORY_A: MaterialHandle = MaterialHandle(3);
const PRESENT_HISTORY_B: MaterialHandle = MaterialHandle(4);
const SIGNAL_FIELD_MATERIAL: MaterialHandle = MaterialHandle(5);
const PRESENT_SIGNAL_FIELD: MaterialHandle = MaterialHandle(6);
const COMPOSITE_FROM_SIGNAL: MaterialHandle = MaterialHandle(7);
const PRESENT_COMPOSITE: MaterialHandle = MaterialHandle(8);
const SPECTRUM_BAR_MATERIAL: MaterialHandle = MaterialHandle(9);
const FEEDBACK_CAMERA: CameraHandle = CameraHandle(1);
const PRESENT_CAMERA: CameraHandle = CameraHandle(2);
const HISTORY_A: TextureHandle = TextureHandle(2);
const HISTORY_B: TextureHandle = TextureHandle(3);
const SIGNAL_FIELD_TARGET: TextureHandle = TextureHandle(4);
const COMPOSITE_TARGET: TextureHandle = TextureHandle(5);
const FIXED_STEP_SECONDS: f32 = 1.0 / 60.0;
const FEEDBACK_WARMUP_FRAMES: u64 = 120;
const FEEDBACK_WGSL: &str = include_str!("../assets/feedback.wgsl");
const SIGNAL_FIELD_WGSL: &str = include_str!("../assets/signal_field.wgsl");
const COMPOSITE_WGSL: &str = include_str!("../assets/composite.wgsl");

// These counters identify resource churn that should not occur after the
// feedback targets and their material bindings have been initialized.
fn steady_state_resource_churn(stats: &RenderFrameStats) -> u32 {
    stats
        .binding_allocations
        .saturating_add(stats.pipeline_creations)
        .saturating_add(stats.pipeline_replacements)
        .saturating_add(stats.mesh_uploads)
        .saturating_add(stats.mesh_replacements)
        .saturating_add(stats.texture_allocations)
        .saturating_add(stats.texture_replacements)
}

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
    let three_pass_graph = VisualizerPassGraph::three_pass_signal(viewport.width, viewport.height);
    fs::write(
        output.join("three-pass-signal.graph.json"),
        format!("{}\n", three_pass_graph.to_structural_json()?),
    )?;
    fs::write(
        output.join("three-pass-signal.summary.json"),
        format!(
            "{}\n",
            serde_json::to_string_pretty(&three_pass_graph.validate_with_summary()?)?
        ),
    )?;
    let feedback_graph = VisualizerPassGraph::three_pass_feedback(viewport.width, viewport.height);
    fs::write(
        output.join("three-pass-feedback.graph.json"),
        format!("{}\n", feedback_graph.to_structural_json()?),
    )?;
    for definition in NativeVisualizerDefinition::all(viewport)? {
        fs::write(
            output.join(format!("native-visualizer-{}.json", definition.id)),
            format!("{}\n", definition.to_structural_json()?),
        )?;
    }
    fs::write(
        output.join("three-pass-feedback.summary.json"),
        format!(
            "{}\n",
            serde_json::to_string_pretty(&feedback_graph.validate_with_summary()?)?
        ),
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
        let working_set = observe_pcm_analysis_working_set(&window, PcmAnalysisConfig::default())?;
        fs::write(
            output.join(format!("{stem}.working-set.observation.json")),
            format!("{}\n", working_set.to_observation_json()?),
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
        let waveform = VisualizerWaveform::from_frame(&frame, 1.0)?;
        fs::write(
            output.join(format!("{stem}.waveform.json")),
            format!("{}\n", waveform.to_structural_json()?),
        )?;
        let spectrum_bars = VisualizerSpectrumBars::from_frame(&frame, 16, 1.0)?;
        fs::write(
            output.join(format!("{stem}.spectrum-bars.json")),
            format!("{}\n", spectrum_bars.to_structural_json()?),
        )?;
        write_cpu_preview(
            output.join(format!("{stem}.bmp")),
            output.join(format!("{stem}.preview.txt")),
            &frame,
        )?;
        println!("wrote {stem} input, waveform, spectrum-bar, and CPU preview evidence");
    }
    println!("wrote validated two-pass, three-pass, and feedback graph evidence");
    Ok(())
}

/// Emits source and semantic contract evidence without claiming that the
/// backend accepts arbitrary runtime material parameters yet.
fn write_shader_contract_artifacts(output: &std::path::Path) -> PlatformResult<()> {
    let module = visualizer_shader_module()?;
    let material = visualizer_material_definition()?;
    let source_fingerprint = fnv1a64(module.source.as_bytes());

    fs::write(
        output.join("audio-visualizer-feedback.wgsl"),
        &module.source,
    )?;
    fs::write(
        output.join("audio-visualizer-feedback.contract.txt"),
        format!(
            "schema=tokimu-visualizer-shader-contract-v2\n\
 shader_label={}\n\
 source_file=audio-visualizer-feedback.wgsl\n\
source_fingerprint=fnv1a64:{source_fingerprint:016x}\n\
vertex_entry_point={}\n\
fragment_entry_point={}\n\
material_id={}\n\
material_parameter=visualizer_signal\n\
 material_parameter_kind=vector4\n\
 binding_0_0=material-parameter:visualizer_signal:vector4\n\
 binding_0_1=material-texture:previous_frame\n\
 binding_0_2=material-sampler:previous_frame\n\
binding_1_0=instance-transform\n\
binding_2_0=camera\n\
vertex_input_0=position3\n\
 execution_bridge=legacy-four-float-material-slot-plus-renderer-owned-previous-frame-target\n\
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
    let three_pass_graph = VisualizerPassGraph::three_pass_signal(viewport.width, viewport.height);
    println!("--- three-pass-signal graph ---");
    println!("{}", three_pass_graph.to_structural_json()?);
    println!(
        "--- three-pass-signal summary ---\n{}",
        serde_json::to_string_pretty(&three_pass_graph.validate_with_summary()?)?
    );
    let feedback_graph = VisualizerPassGraph::three_pass_feedback(viewport.width, viewport.height);
    println!("--- three-pass-feedback graph ---");
    println!("{}", feedback_graph.to_structural_json()?);
    println!(
        "--- three-pass-feedback summary ---\n{}",
        serde_json::to_string_pretty(&feedback_graph.validate_with_summary()?)?
    );
    for definition in NativeVisualizerDefinition::all(viewport)? {
        println!("--- native visualizer: {} ---", definition.id);
        println!("{}", definition.to_structural_json()?);
    }
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
        let window = fixture.window();
        let observation = observe_pcm_analysis_timing(&window, PcmAnalysisConfig::default(), 32)?;
        let working_set = observe_pcm_analysis_working_set(&window, PcmAnalysisConfig::default())?;
        println!("--- {} ---", fixture.label());
        println!("{}", observation.to_observation_json()?);
        println!("{}", working_set.to_observation_json()?);
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
        "audio-visualizer-feedback",
        FEEDBACK_WGSL,
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
            ShaderBindingDeclaration::new(
                0,
                1,
                ShaderBindingSource::MaterialSampler {
                    texture_parameter: "previous_frame".to_owned(),
                },
            ),
            ShaderBindingDeclaration::new(
                0,
                2,
                ShaderBindingSource::MaterialSampler {
                    texture_parameter: "previous_frame".to_owned(),
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
        [
            MaterialParameterDeclaration::new(
                "visualizer_signal",
                MaterialParameterKind::Vector4,
                MaterialParameterValue::Vector4([0.0; 4]),
            )?,
            MaterialParameterDeclaration::new(
                "previous_frame",
                MaterialParameterKind::Texture,
                MaterialParameterValue::Texture(None),
            )?,
        ],
    )
    .map_err(Into::into)
}

fn signal_field_shader_module() -> Result<ShaderModuleDefinition, ShaderModuleValidationError> {
    ShaderModuleDefinition::new(
        "audio-visualizer-signal-field",
        SIGNAL_FIELD_WGSL,
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
            // The current WGPU material layout always exposes a texture and
            // sampler. This shader deliberately ignores them while proving a
            // provider-neutral signal field can use the same bounded contract.
            ShaderBindingDeclaration::new(
                0,
                1,
                ShaderBindingSource::MaterialSampler {
                    texture_parameter: "previous_frame".to_owned(),
                },
            ),
            ShaderBindingDeclaration::new(
                0,
                2,
                ShaderBindingSource::MaterialSampler {
                    texture_parameter: "previous_frame".to_owned(),
                },
            ),
            ShaderBindingDeclaration::new(1, 0, ShaderBindingSource::InstanceTransform),
            ShaderBindingDeclaration::new(2, 0, ShaderBindingSource::Camera),
        ],
        vec![ShaderVertexInput::new(0, ShaderVertexSemantic::Position3)],
    )
}

fn composite_shader_module() -> Result<ShaderModuleDefinition, ShaderModuleValidationError> {
    ShaderModuleDefinition::new(
        "audio-visualizer-signal-composite",
        COMPOSITE_WGSL,
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
            ShaderBindingDeclaration::new(
                0,
                1,
                ShaderBindingSource::MaterialSampler {
                    texture_parameter: "previous_frame".to_owned(),
                },
            ),
            ShaderBindingDeclaration::new(
                0,
                2,
                ShaderBindingSource::MaterialSampler {
                    texture_parameter: "previous_frame".to_owned(),
                },
            ),
            ShaderBindingDeclaration::new(1, 0, ShaderBindingSource::InstanceTransform),
            ShaderBindingDeclaration::new(2, 0, ShaderBindingSource::Camera),
        ],
        vec![ShaderVertexInput::new(0, ShaderVertexSemantic::Position3)],
    )
}

struct App {
    renderer: Option<WgpuBackend>,
    window: Option<Arc<NativeWindow>>,
    pipeline: PipelineHandle,
    signal_field_pipeline: PipelineHandle,
    composite_pipeline: PipelineHandle,
    present_pipeline: PipelineHandle,
    spectrum_bar_pipeline: PipelineHandle,
    viewport: VisualizerViewport,
    feedback_target_viewport: VisualizerViewport,
    history_a_is_previous: bool,
    feedback_reset_pending: bool,
    source: SyntheticVisualizerInput,
    fixture_index: usize,
    visualizer_frame: u64,
    step_accumulator: f32,
    time_scale: f32,
    paused: bool,
    feedback_warm_observed: bool,
    selected_visualizer: NativeVisualizerKind,
}

impl App {
    fn new() -> Self {
        let source = SyntheticVisualizerInput::new(
            // Start on a fixture whose band distribution changes over time so
            // the first native run visibly demonstrates signal progression.
            SyntheticAudioFixture::FrequencySweep,
            SyntheticVisualizerConfig::default(),
        )
        .expect("default synthetic visualizer configuration is valid");
        Self {
            renderer: None,
            window: None,
            pipeline: PipelineHandle(0),
            signal_field_pipeline: PipelineHandle(0),
            composite_pipeline: PipelineHandle(0),
            present_pipeline: PipelineHandle(0),
            spectrum_bar_pipeline: PipelineHandle(0),
            viewport: VisualizerViewport::new(1, 1).expect("unit viewport is valid"),
            feedback_target_viewport: VisualizerViewport::new(1, 1)
                .expect("unit viewport is valid"),
            history_a_is_previous: true,
            feedback_reset_pending: true,
            source,
            fixture_index: 3,
            visualizer_frame: 0,
            step_accumulator: 0.0,
            time_scale: 1.0,
            paused: false,
            feedback_warm_observed: false,
            selected_visualizer: NativeVisualizerKind::FeedbackBloom,
        }
    }

    fn reset(&mut self) {
        self.visualizer_frame = 0;
        self.step_accumulator = 0.0;
        self.history_a_is_previous = true;
        self.feedback_reset_pending = true;
        self.feedback_warm_observed = false;
    }

    fn upload_feedback_material_bindings(
        renderer: &mut WgpuBackend,
        signal: [f32; 4],
    ) -> PlatformResult<()> {
        for (material, target, label) in [
            (
                FEEDBACK_FROM_HISTORY_A,
                HISTORY_A,
                "visualizer-feedback-from-history-a",
            ),
            (
                FEEDBACK_FROM_HISTORY_B,
                HISTORY_B,
                "visualizer-feedback-from-history-b",
            ),
        ] {
            renderer.upload_material(
                material,
                &Material::new(
                    label,
                    Color::rgba(signal[0], signal[1], signal[2], signal[3]),
                )
                .with_texture(target),
            )?;
        }
        for (material, target, label) in [
            (PRESENT_HISTORY_A, HISTORY_A, "visualizer-present-history-a"),
            (PRESENT_HISTORY_B, HISTORY_B, "visualizer-present-history-b"),
        ] {
            renderer.upload_material(
                material,
                &Material::new(label, Color::rgb(1.0, 1.0, 1.0)).with_texture(target),
            )?;
        }
        Ok(())
    }

    fn upload_signal_field_material_bindings(
        renderer: &mut WgpuBackend,
        signal: [f32; 4],
    ) -> PlatformResult<()> {
        renderer.upload_material(
            SIGNAL_FIELD_MATERIAL,
            &Material::new(
                "visualizer-signal-field",
                Color::rgba(signal[0], signal[1], signal[2], signal[3]),
            ),
        )?;
        renderer.upload_material(
            PRESENT_SIGNAL_FIELD,
            &Material::new("visualizer-present-signal-field", Color::rgb(1.0, 1.0, 1.0))
                .with_texture(SIGNAL_FIELD_TARGET),
        )?;
        Ok(())
    }

    fn upload_composite_material_bindings(renderer: &mut WgpuBackend) -> PlatformResult<()> {
        renderer.upload_material(
            COMPOSITE_FROM_SIGNAL,
            &Material::new(
                "visualizer-composite-from-signal",
                Color::rgb(1.0, 1.0, 1.0),
            )
            .with_texture(SIGNAL_FIELD_TARGET),
        )?;
        renderer.upload_material(
            PRESENT_COMPOSITE,
            &Material::new("visualizer-present-composite", Color::rgb(1.0, 1.0, 1.0))
                .with_texture(COMPOSITE_TARGET),
        )?;
        Ok(())
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
        match self.selected_visualizer {
            NativeVisualizerKind::SignalField => self.render_signal_field(),
            NativeVisualizerKind::FeedbackBloom => self.render_feedback_bloom(),
            NativeVisualizerKind::SignalComposite => self.render_signal_composite(),
            NativeVisualizerKind::SpectrumBars => self.render_spectrum_bars(),
        }
    }

    fn render_spectrum_bars(&mut self) -> PlatformResult<FrameOutcome> {
        let observation = self.source.frame(self.visualizer_frame, self.viewport)?;
        let bars = VisualizerSpectrumBars::from_frame(&observation, 32, 1.5)?;
        let Some(renderer) = self.renderer.as_mut() else {
            return Ok(FrameOutcome::Continue);
        };

        renderer.begin_frame();
        let mut commands = Vec::with_capacity(bars.bars.len() + 1);
        commands.push(RenderCommand::Clear(ClearCommand {
            color: Color::rgb(0.015, 0.025, 0.045),
        }));
        commands.extend(bars.bars.iter().map(|bar| {
            let width = bar.maximum[0] - bar.minimum[0];
            let height = bar.maximum[1] - bar.minimum[1];
            RenderCommand::DrawMesh(DrawMeshCommand {
                mesh: QUAD,
                material: SPECTRUM_BAR_MATERIAL,
                pipeline: self.spectrum_bar_pipeline,
                instance: Instance2d::identity()
                    .with_translation([
                        (bar.minimum[0] + bar.maximum[0]) * 0.5,
                        (bar.minimum[1] + bar.maximum[1]) * 0.5,
                    ])
                    .with_scale([width, height]),
                camera: None,
                viewport: None,
            })
        }));
        renderer.submit(&commands);
        let stats = renderer.present()?;
        let target_resources = renderer.render_target_resource_observation();
        self.observe_frame(&stats, &observation, target_resources);
        self.update_title();
        Ok(FrameOutcome::Continue)
    }

    fn render_signal_field(&mut self) -> PlatformResult<FrameOutcome> {
        let observation = self.source.frame(self.visualizer_frame, self.viewport)?;
        let signal = observation.shader_signal();
        let aspect = self.viewport.width as f32 / self.viewport.height as f32;
        let Some(renderer) = self.renderer.as_mut() else {
            return Ok(FrameOutcome::Continue);
        };
        renderer.begin_frame();
        renderer.update_material_color(
            SIGNAL_FIELD_MATERIAL,
            Color::rgba(signal[0], signal[1], signal[2], signal[3]),
        )?;
        let mut target_camera = Camera::orthographic_2d_with_height(
            self.viewport.width as f32,
            self.viewport.height as f32,
            2.0,
        );
        target_camera.view = Mat4::from_translation(Vec3::ZERO);
        renderer.upload_camera(FEEDBACK_CAMERA, target_camera);
        renderer.draw_meshes_to_render_target(
            SIGNAL_FIELD_TARGET,
            Color::rgb(0.015, 0.025, 0.045),
            &[DrawMeshCommand {
                mesh: QUAD,
                material: SIGNAL_FIELD_MATERIAL,
                pipeline: self.signal_field_pipeline,
                instance: Instance2d::identity().with_scale([aspect, 1.0]),
                camera: Some(FEEDBACK_CAMERA),
                viewport: None,
            }],
        )?;
        let stats = Self::present_target(
            renderer,
            self.viewport,
            self.present_pipeline,
            PRESENT_SIGNAL_FIELD,
            aspect,
        )?;
        let target_resources = renderer.render_target_resource_observation();
        self.observe_frame(&stats, &observation, target_resources);
        self.update_title();
        Ok(FrameOutcome::Continue)
    }

    fn render_signal_composite(&mut self) -> PlatformResult<FrameOutcome> {
        let observation = self.source.frame(self.visualizer_frame, self.viewport)?;
        let signal = observation.shader_signal();
        let aspect = self.viewport.width as f32 / self.viewport.height as f32;
        let Some(renderer) = self.renderer.as_mut() else {
            return Ok(FrameOutcome::Continue);
        };
        renderer.begin_frame();
        renderer.update_material_color(
            SIGNAL_FIELD_MATERIAL,
            Color::rgba(signal[0], signal[1], signal[2], signal[3]),
        )?;
        renderer.update_material_color(
            COMPOSITE_FROM_SIGNAL,
            Color::rgba(signal[0], signal[1], signal[2], signal[3]),
        )?;
        let mut target_camera = Camera::orthographic_2d_with_height(
            self.viewport.width as f32,
            self.viewport.height as f32,
            2.0,
        );
        target_camera.view = Mat4::from_translation(Vec3::ZERO);
        renderer.upload_camera(FEEDBACK_CAMERA, target_camera);
        renderer.draw_meshes_to_render_target(
            SIGNAL_FIELD_TARGET,
            Color::rgb(0.015, 0.025, 0.045),
            &[DrawMeshCommand {
                mesh: QUAD,
                material: SIGNAL_FIELD_MATERIAL,
                pipeline: self.signal_field_pipeline,
                instance: Instance2d::identity().with_scale([aspect, 1.0]),
                camera: Some(FEEDBACK_CAMERA),
                viewport: None,
            }],
        )?;
        renderer.draw_meshes_to_render_target(
            COMPOSITE_TARGET,
            Color::rgb(0.004, 0.008, 0.016),
            &[DrawMeshCommand {
                mesh: QUAD,
                material: COMPOSITE_FROM_SIGNAL,
                pipeline: self.composite_pipeline,
                instance: Instance2d::identity().with_scale([aspect, 1.0]),
                camera: Some(FEEDBACK_CAMERA),
                viewport: None,
            }],
        )?;
        let stats = Self::present_target(
            renderer,
            self.viewport,
            self.present_pipeline,
            PRESENT_COMPOSITE,
            aspect,
        )?;
        let target_resources = renderer.render_target_resource_observation();
        self.observe_frame(&stats, &observation, target_resources);
        self.update_title();
        Ok(FrameOutcome::Continue)
    }

    fn render_feedback_bloom(&mut self) -> PlatformResult<FrameOutcome> {
        let observation = self.source.frame(self.visualizer_frame, self.viewport)?;
        let signal = observation.shader_signal();
        let aspect = self.viewport.width as f32 / self.viewport.height as f32;
        let Some(renderer) = self.renderer.as_mut() else {
            return Ok(FrameOutcome::Continue);
        };

        renderer.begin_frame();
        let (current_target, feedback_material, present_material) = if self.history_a_is_previous {
            (HISTORY_B, FEEDBACK_FROM_HISTORY_A, PRESENT_HISTORY_B)
        } else {
            (HISTORY_A, FEEDBACK_FROM_HISTORY_B, PRESENT_HISTORY_A)
        };
        // Texture identities are stable across ordinary frames, so only the
        // active feedback material color uniform changes with the audio signal.
        renderer.update_material_color(
            feedback_material,
            Color::rgba(signal[0], signal[1], signal[2], signal[3]),
        )?;
        if self.feedback_reset_pending {
            for target in [HISTORY_A, HISTORY_B] {
                renderer.draw_meshes_to_render_target(
                    target,
                    Color::rgb(0.015, 0.025, 0.045),
                    &[],
                )?;
            }
            self.feedback_reset_pending = false;
        }

        let mut feedback_camera = Camera::orthographic_2d_with_height(
            self.feedback_target_viewport.width as f32,
            self.feedback_target_viewport.height as f32,
            2.0,
        );
        feedback_camera.view = Mat4::from_translation(Vec3::ZERO);
        renderer.upload_camera(FEEDBACK_CAMERA, feedback_camera);
        let feedback_aspect = self.feedback_target_viewport.width as f32
            / self.feedback_target_viewport.height as f32;
        renderer.draw_meshes_to_render_target(
            current_target,
            Color::rgb(0.015, 0.025, 0.045),
            &[DrawMeshCommand {
                mesh: QUAD,
                material: feedback_material,
                pipeline: self.pipeline,
                instance: Instance2d::identity().with_scale([feedback_aspect, 1.0]),
                camera: Some(FEEDBACK_CAMERA),
                viewport: None,
            }],
        )?;

        let stats = Self::present_target(
            renderer,
            self.viewport,
            self.present_pipeline,
            present_material,
            aspect,
        )?;
        self.history_a_is_previous = !self.history_a_is_previous;
        let target_resources = renderer.render_target_resource_observation();
        self.observe_frame(&stats, &observation, target_resources);
        self.update_title();
        Ok(FrameOutcome::Continue)
    }

    fn present_target(
        renderer: &mut WgpuBackend,
        viewport: VisualizerViewport,
        pipeline: PipelineHandle,
        material: MaterialHandle,
        aspect: f32,
    ) -> PlatformResult<tokimu::RenderStats> {
        let mut present_camera =
            Camera::orthographic_2d_with_height(viewport.width as f32, viewport.height as f32, 2.0);
        present_camera.view = Mat4::from_translation(Vec3::ZERO);
        renderer.upload_camera(PRESENT_CAMERA, present_camera);
        renderer.set_active_camera(PRESENT_CAMERA);
        renderer.submit(&[
            RenderCommand::Clear(ClearCommand {
                color: Color::rgb(0.015, 0.025, 0.045),
            }),
            RenderCommand::DrawMesh(DrawMeshCommand {
                mesh: QUAD,
                material,
                pipeline,
                instance: Instance2d::identity().with_scale([aspect, 1.0]),
                camera: Some(PRESENT_CAMERA),
                viewport: None,
            }),
        ]);
        Ok(renderer.present()?)
    }

    fn observe_frame(
        &mut self,
        stats: &tokimu::RenderStats,
        observation: &visualizer_tools::VisualizerFrameInput,
        target_resources: tokimu::RenderTargetResourceObservation,
    ) {
        if self.selected_visualizer == NativeVisualizerKind::FeedbackBloom
            && !self.feedback_warm_observed
            && self.visualizer_frame >= FEEDBACK_WARMUP_FRAMES
        {
            let churn = steady_state_resource_churn(&stats.frame);
            println!(
                "hello-audio-visualizer warm feedback observation: frame={}, resource_churn={}, binding_allocations={}, pipeline_creations={}, mesh_uploads={}, texture_allocations={}, render_targets={}, target_pixels={}, target_estimated_bytes={}",
                self.visualizer_frame,
                churn,
                stats.frame.binding_allocations,
                stats.frame.pipeline_creations,
                stats.frame.mesh_uploads,
                stats.frame.texture_allocations,
                target_resources.target_count,
                target_resources.color_pixels,
                target_resources.estimated_total_bytes,
            );
            self.feedback_warm_observed = true;
        }
        if self.visualizer_frame.is_multiple_of(120) {
            println!(
                "hello-audio-visualizer frame {}: visualizer={}, fixture={}, time={:.3}, phase={:.3}, bands=[{:.3},{:.3},{:.3}], draws={}, binding_allocations={}, uniform_writes={}, render_targets={}, target_estimated_bytes={}, target_encode_ms={:.3}, target_submit_ms={:.3}",
                self.visualizer_frame,
                self.selected_visualizer.id(),
                observation.fixture.label(),
                observation.time_seconds,
                observation.shader_signal()[0],
                observation.shader_signal()[1],
                observation.shader_signal()[2],
                observation.shader_signal()[3],
                stats.frame.draw_calls,
                stats.frame.binding_allocations,
                stats.frame.uniform_buffer_writes,
                target_resources.target_count,
                target_resources.estimated_total_bytes,
                stats
                    .frame
                    .cpu_timings
                    .render_target_command_encoding
                    .map_or(0.0, |duration| duration.as_secs_f64() * 1_000.0),
                stats
                    .frame
                    .cpu_timings
                    .render_target_queue_submit_call
                    .map_or(0.0, |duration| duration.as_secs_f64() * 1_000.0),
            );
        }
    }

    fn update_title(&self) {
        if let Some(window) = self.window.as_ref() {
            window.set_title(&format!(
                "Tokimu Audio Visualizer | {} | {} | frame={} | {:.2}x | {} | Q signal | W feedback | E composite | X bars | Left/Right fixture | Space pause | Up/Down speed | R reset",
                self.selected_visualizer.label(),
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
        self.feedback_target_viewport = self.viewport;
        self.window = Some(window.clone());

        let mut renderer = WgpuBackend::for_window(window, size.width, size.height)?;
        renderer.upload_mesh(QUAD, &Mesh::quad());
        for target in [HISTORY_A, HISTORY_B, SIGNAL_FIELD_TARGET, COMPOSITE_TARGET] {
            renderer.create_render_target_rgba8(
                target,
                Rgba8TextureDescriptor::new(
                    self.feedback_target_viewport.width,
                    self.feedback_target_viewport.height,
                    Rgba8TextureColorSpace::Srgb,
                ),
            )?;
        }
        Self::upload_feedback_material_bindings(&mut renderer, [0.0; 4])?;
        Self::upload_signal_field_material_bindings(&mut renderer, [0.0; 4])?;
        Self::upload_composite_material_bindings(&mut renderer)?;
        renderer.upload_material(
            SPECTRUM_BAR_MATERIAL,
            &Material::new("visualizer-spectrum-bars", Color::rgb(0.35, 0.95, 0.82)),
        )?;
        let pipeline =
            Pipeline::custom_wgsl_module("audio-visualizer-feedback", visualizer_shader_module()?)?;
        pipeline.validate_draw_contract(&visualizer_material_definition()?, &Mesh::quad())?;
        self.pipeline = renderer.register_pipeline(&pipeline)?;
        let signal_field_pipeline = Pipeline::custom_wgsl_module(
            "audio-visualizer-signal-field",
            signal_field_shader_module()?,
        )?;
        signal_field_pipeline
            .validate_draw_contract(&visualizer_material_definition()?, &Mesh::quad())?;
        self.signal_field_pipeline = renderer.register_pipeline(&signal_field_pipeline)?;
        let composite_pipeline = Pipeline::custom_wgsl_module(
            "audio-visualizer-signal-composite",
            composite_shader_module()?,
        )?;
        composite_pipeline
            .validate_draw_contract(&visualizer_material_definition()?, &Mesh::quad())?;
        self.composite_pipeline = renderer.register_pipeline(&composite_pipeline)?;
        self.present_pipeline = renderer.register_pipeline(&Pipeline::new(
            "audio-visualizer-target-present",
            tokimu::PipelineKind::Texture2d,
        ))?;
        self.spectrum_bar_pipeline = renderer.register_pipeline(&Pipeline::new(
            "audio-visualizer-spectrum-bars",
            tokimu::PipelineKind::SolidColor2d,
        ))?;
        self.renderer = Some(renderer);
        self.update_title();
        Ok(())
    }

    fn on_platform_event(&mut self, event: PlatformInputEvent) -> PlatformResult<()> {
        match event {
            PlatformInputEvent::Resized { width, height } => {
                let viewport = VisualizerViewport::new(width.max(1), height.max(1))?;
                let replace_feedback_targets = self.feedback_target_viewport != viewport;
                self.viewport = viewport;
                if let Some(renderer) = self.renderer.as_mut() {
                    renderer.resize_surface(viewport.width, viewport.height);
                    if replace_feedback_targets {
                        let mut rebinds = 0;
                        let mut invalidated_derived_materials = 0;
                        for target in [HISTORY_A, HISTORY_B, SIGNAL_FIELD_TARGET, COMPOSITE_TARGET]
                        {
                            let replacement = renderer.replace_render_target_rgba8(
                                target,
                                Rgba8TextureDescriptor::new(
                                    viewport.width,
                                    viewport.height,
                                    Rgba8TextureColorSpace::Srgb,
                                ),
                            )?;
                            rebinds += replacement.materials_requiring_rebind;
                            invalidated_derived_materials +=
                                replacement.invalidated_derived_materials;
                        }
                        self.feedback_target_viewport = viewport;
                        // Replacing target textures invalidates the views held
                        // by material bind groups. Rebind once at this explicit
                        // lifecycle boundary rather than on every frame.
                        Self::upload_feedback_material_bindings(renderer, [0.0; 4])?;
                        Self::upload_signal_field_material_bindings(renderer, [0.0; 4])?;
                        Self::upload_composite_material_bindings(renderer)?;
                        self.feedback_reset_pending = true;
                        println!(
                            "hello-audio-visualizer resized feedback targets to {}x{}: rebinds={}, invalidated_derived_materials={}, render_targets={}, target_estimated_bytes={}",
                            viewport.width,
                            viewport.height,
                            rebinds,
                            invalidated_derived_materials,
                            renderer.render_target_resource_observation().target_count,
                            renderer
                                .render_target_resource_observation()
                                .estimated_total_bytes,
                        );
                    }
                }
            }
            PlatformInputEvent::KeyboardInput { key, pressed: true } => match key {
                KeyCode::ArrowRight => self.cycle_fixture(1),
                KeyCode::ArrowLeft => self.cycle_fixture(-1),
                KeyCode::ArrowUp => self.time_scale = (self.time_scale * 2.0).min(4.0),
                KeyCode::ArrowDown => self.time_scale = (self.time_scale * 0.5).max(0.25),
                KeyCode::Space => self.paused = !self.paused,
                KeyCode::KeyR => self.reset(),
                KeyCode::KeyQ => self.selected_visualizer = NativeVisualizerKind::SignalField,
                KeyCode::KeyW => self.selected_visualizer = NativeVisualizerKind::FeedbackBloom,
                KeyCode::KeyE => self.selected_visualizer = NativeVisualizerKind::SignalComposite,
                KeyCode::KeyX => self.selected_visualizer = NativeVisualizerKind::SpectrumBars,
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
    use super::{
        composite_shader_module, signal_field_shader_module, steady_state_resource_churn,
        visualizer_material_definition, visualizer_shader_module,
    };
    use tokimu::{
        Color, MaterialDefinition, MaterialDefinitionId, MaterialParameterDeclaration,
        MaterialParameterKind, MaterialParameterValue, Mesh, Pipeline, PipelineDrawContractError,
        RenderFrameStats, ShaderDiagnosticStage, ShaderMaterialCompatibilityError,
    };

    #[test]
    fn visualizer_contract_passes_before_backend_submission() {
        let pipeline = Pipeline::custom_wgsl_module(
            "audio-visualizer-feedback",
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
    fn signal_field_contract_reuses_the_bounded_visualizer_material() {
        let pipeline = Pipeline::custom_wgsl_module(
            "audio-visualizer-signal-field",
            signal_field_shader_module().unwrap(),
        )
        .unwrap();

        assert_eq!(
            pipeline
                .validate_draw_contract(&visualizer_material_definition().unwrap(), &Mesh::quad()),
            Ok(())
        );
    }

    #[test]
    fn composite_contract_samples_the_signal_target_through_the_same_material_shape() {
        let pipeline = Pipeline::custom_wgsl_module(
            "audio-visualizer-signal-composite",
            composite_shader_module().unwrap(),
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
            "audio-visualizer-feedback",
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

    #[test]
    fn steady_state_observation_counts_only_resource_churn() {
        let stable = RenderFrameStats {
            draw_calls: 2,
            uniform_buffer_writes: 3,
            material_resolutions: 2,
            ..RenderFrameStats::EMPTY
        };
        assert_eq!(steady_state_resource_churn(&stable), 0);

        let churned = RenderFrameStats {
            binding_allocations: 1,
            pipeline_creations: 2,
            pipeline_replacements: 3,
            mesh_uploads: 4,
            mesh_replacements: 5,
            texture_allocations: 6,
            texture_replacements: 7,
            ..RenderFrameStats::EMPTY
        };
        assert_eq!(steady_state_resource_churn(&churned), 28);
    }
}
