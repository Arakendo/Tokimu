use std::{fs, path::PathBuf, sync::Arc};

use milkdrop_tools::{
    MilkDropClassicFrameControls, MilkDropCustomShapeFrame, MilkDropCustomWaveFrame,
    MilkDropSelectedRuntime,
};
use tokimu::{
    run_window_with_app, BlendMode, Camera, CameraHandle, ClearCommand, Color, DrawMeshCommand,
    FrameOutcome, Instance2d, KeyCode, Material, MaterialDefinition, MaterialDefinitionId,
    MaterialHandle, MaterialParameterDeclaration, MaterialParameterKind, MaterialParameterValue,
    Mesh, MeshHandle, NativeWindow, Pipeline, PipelineHandle, PipelineRenderState,
    PlatformEventHandler, PlatformInputEvent, PlatformResult, RenderCommand, RenderFrameStats,
    Renderer, Rgba8TextureColorSpace, Rgba8TextureDescriptor, ShaderBindingDeclaration,
    ShaderBindingSource, ShaderModuleDefinition, ShaderModuleValidationError, ShaderVertexInput,
    ShaderVertexSemantic, TextureHandle, WgpuBackend, WindowConfig,
};
use tokimu_core::math::{Mat4, Vec3};
use visualizer_tools::{
    decode_pcm16_wav, encode_pcm16_wav_fixture, observe_pcm_analysis_timing,
    observe_pcm_analysis_working_set, write_cpu_preview, NativeVisualizerDefinition,
    NativeVisualizerKind, PcmAnalysisBacklog, PcmAnalysisConfig, PcmAnalyzer,
    PcmBacklogOverflowPolicy, PcmFixture, SyntheticAudioFixture, SyntheticVisualizerConfig,
    SyntheticVisualizerInput, VisualizerPassGraph, VisualizerRadialShape, VisualizerSpectrumBars,
    VisualizerViewport, VisualizerWaveform,
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
const MILKDROP_FROM_HISTORY_A: MaterialHandle = MaterialHandle(10);
const MILKDROP_FROM_HISTORY_B: MaterialHandle = MaterialHandle(11);
const MILKDROP_CUSTOM_WAVE_MESHES: [MeshHandle; 4] =
    [MeshHandle(2), MeshHandle(3), MeshHandle(4), MeshHandle(5)];
const MILKDROP_CUSTOM_WAVE_MATERIALS: [MaterialHandle; 4] = [
    MaterialHandle(12),
    MaterialHandle(13),
    MaterialHandle(14),
    MaterialHandle(15),
];
const MILKDROP_CUSTOM_SHAPE_MESHES: [MeshHandle; 4] =
    [MeshHandle(6), MeshHandle(7), MeshHandle(8), MeshHandle(9)];
const MILKDROP_CUSTOM_SHAPE_MATERIALS: [MaterialHandle; 4] = [
    MaterialHandle(16),
    MaterialHandle(17),
    MaterialHandle(18),
    MaterialHandle(19),
];
const MAX_NATIVE_CUSTOM_WAVES: usize = MILKDROP_CUSTOM_WAVE_MESHES.len();
const MAX_NATIVE_CUSTOM_SHAPES: usize = MILKDROP_CUSTOM_SHAPE_MESHES.len();
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
const MILKDROP_CLASSIC_WGSL: &str = include_str!("../assets/milkdrop_classic.wgsl");
const MILKDROP_SELECTED_FIXTURE: &str =
    include_str!("../../hello-milkdrop/assets/tokimu-selected-fixture.milk");

fn milkdrop_overlay_pipeline(
    additive: bool,
    alpha_pipeline: PipelineHandle,
    additive_pipeline: PipelineHandle,
) -> PipelineHandle {
    if additive {
        additive_pipeline
    } else {
        alpha_pipeline
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum VisualizerSelection {
    Native(NativeVisualizerKind),
    MilkDropClassic,
}

impl VisualizerSelection {
    fn id(self) -> &'static str {
        match self {
            Self::Native(kind) => kind.id(),
            Self::MilkDropClassic => "milkdrop-classic-selected",
        }
    }

    fn label(self) -> &'static str {
        match self {
            Self::Native(kind) => kind.label(),
            Self::MilkDropClassic => "MilkDrop Classic / Selected Subset",
        }
    }

    fn uses_feedback(self) -> bool {
        matches!(
            self,
            Self::Native(NativeVisualizerKind::FeedbackBloom) | Self::MilkDropClassic
        )
    }
}

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

/// Native-consumer lowering for the renderer-neutral custom-wave point
/// contract. `milkdrop-tools` intentionally stops at normalized points; this
/// example decides how thick lines and dots become triangle geometry.
fn custom_wave_mesh(frame: &MilkDropCustomWaveFrame, aspect: f32) -> Option<Mesh> {
    let map_point = |point: [f32; 2]| {
        [
            (point[0].clamp(0.0, 1.0) * 2.0 - 1.0) * aspect,
            1.0 - point[1].clamp(0.0, 1.0) * 2.0,
        ]
    };
    let half_width = if frame.wave.thick { 0.012 } else { 0.006 };
    let mut positions = Vec::new();

    if frame.wave.dots {
        for point in frame
            .points
            .iter()
            .copied()
            .filter(|point| point[0].is_finite() && point[1].is_finite())
        {
            let [x, y] = map_point(point);
            positions.extend_from_slice(&[
                [x - half_width, y + half_width, 0.0],
                [x - half_width, y - half_width, 0.0],
                [x + half_width, y - half_width, 0.0],
                [x - half_width, y + half_width, 0.0],
                [x + half_width, y - half_width, 0.0],
                [x + half_width, y + half_width, 0.0],
            ]);
        }
    } else {
        for pair in frame.points.windows(2) {
            if pair
                .iter()
                .any(|point| !point[0].is_finite() || !point[1].is_finite())
            {
                continue;
            }
            let [ax, ay] = map_point(pair[0]);
            let [bx, by] = map_point(pair[1]);
            let dx = bx - ax;
            let dy = by - ay;
            let length = (dx * dx + dy * dy).sqrt();
            if length <= f32::EPSILON {
                continue;
            }
            let offset_x = -dy / length * half_width;
            let offset_y = dx / length * half_width;
            let a_left = [ax + offset_x, ay + offset_y, 0.0];
            let a_right = [ax - offset_x, ay - offset_y, 0.0];
            let b_left = [bx + offset_x, by + offset_y, 0.0];
            let b_right = [bx - offset_x, by - offset_y, 0.0];
            positions.extend_from_slice(&[a_left, a_right, b_right, a_left, b_right, b_left]);
        }
    }

    (!positions.is_empty()).then(|| Mesh::uniform_normal(positions, [0.0, 0.0, 1.0]))
}

/// Native-consumer lowering for the selected literal convex custom-shape
/// contract. The provider owns normalized polygon points only; this consumer
/// chooses a simple triangle fan for its bounded convex subset.
fn custom_shape_mesh(frame: &MilkDropCustomShapeFrame, aspect: f32) -> Option<Mesh> {
    let points = frame
        .points
        .iter()
        .copied()
        .filter(|point| point[0].is_finite() && point[1].is_finite())
        .map(|point| {
            [
                (point[0].clamp(0.0, 1.0) * 2.0 - 1.0) * aspect,
                1.0 - point[1].clamp(0.0, 1.0) * 2.0,
                0.0,
            ]
        })
        .collect::<Vec<_>>();
    if points.len() < 3 {
        return None;
    }

    let mut positions = Vec::with_capacity((points.len() - 2) * 3);
    for index in 1..points.len() - 1 {
        positions.extend_from_slice(&[points[0], points[index], points[index + 1]]);
    }
    Some(Mesh::uniform_normal(positions, [0.0, 0.0, 1.0]))
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
    write_milkdrop_compatibility_artifacts(&output, viewport)?;
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
        let radial_shape = VisualizerRadialShape::from_frame(&frame, 24, 0.35, 0.4)?;
        fs::write(
            output.join(format!("{stem}.radial-shape.json")),
            format!("{}\n", radial_shape.to_structural_json()?),
        )?;
        write_cpu_preview(
            output.join(format!("{stem}.bmp")),
            output.join(format!("{stem}.preview.txt")),
            &frame,
        )?;
        println!(
            "wrote {stem} input, waveform, spectrum-bar, radial-shape, and CPU preview evidence"
        );
    }
    println!("wrote validated two-pass, three-pass, and feedback graph evidence");
    Ok(())
}

fn write_milkdrop_compatibility_artifacts(
    output: &std::path::Path,
    viewport: VisualizerViewport,
) -> PlatformResult<()> {
    let mut runtime = MilkDropSelectedRuntime::from_source(MILKDROP_SELECTED_FIXTURE)?;
    let source = SyntheticVisualizerInput::new(
        SyntheticAudioFixture::FrequencySweep,
        SyntheticVisualizerConfig::default(),
    )?;
    let observation = source.frame(90, viewport)?;
    let signal = observation.shader_signal();
    let controls = runtime.step_with_audio(
        90,
        observation.time_seconds,
        [signal[1], signal[2], signal[3]],
        &observation.waveform,
        &observation.spectrum,
    )?;

    fs::write(
        output.join("milkdrop-selected-fixture.milk"),
        MILKDROP_SELECTED_FIXTURE,
    )?;
    fs::write(
        output.join("milkdrop-selected-fixture.document.json"),
        format!("{}\n", runtime.document().to_structural_json()?),
    )?;
    fs::write(
        output.join("milkdrop-selected-fixture.parameters.json"),
        format!("{}\n", serde_json::to_string_pretty(runtime.parameters())?),
    )?;
    fs::write(
        output.join("milkdrop-selected-fixture.frame.json"),
        format!("{}\n", controls.to_structural_json()?),
    )?;
    println!(
        "wrote selected MilkDrop parse, parameter, equation, scalar-control, custom-wave-point, and custom-shape-point evidence"
    );
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

    let milkdrop_module = milkdrop_classic_shader_module()?;
    fs::write(
        output.join("milkdrop-classic-selected.wgsl"),
        &milkdrop_module.source,
    )?;
    fs::write(
        output.join("milkdrop-classic-selected.contract.txt"),
        format!(
            "schema=tokimu-milkdrop-shader-contract-v1\n\
shader_label={}\n\
source_file=milkdrop-classic-selected.wgsl\n\
source_fingerprint=fnv1a64:{:016x}\n\
control_slot=phase,audio-energy,decay,zoom\n\
execution_scope=selected-milkdrop-1-scalars-init-and-per-frame-equations\n\
deferred=per-pixel-equations,custom-waves,custom-shapes,textures,embedded-shaders\n\
projectm_dependency=none\n",
            milkdrop_module.label,
            fnv1a64(milkdrop_module.source.as_bytes()),
        ),
    )?;
    println!("wrote selected MilkDrop shader contract evidence");
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
    sampled_visualizer_shader_module("audio-visualizer-feedback", FEEDBACK_WGSL)
}

fn milkdrop_classic_shader_module() -> Result<ShaderModuleDefinition, ShaderModuleValidationError> {
    sampled_visualizer_shader_module("milkdrop-classic-selected", MILKDROP_CLASSIC_WGSL)
}

fn sampled_visualizer_shader_module(
    label: &str,
    source: &str,
) -> Result<ShaderModuleDefinition, ShaderModuleValidationError> {
    ShaderModuleDefinition::new(
        label,
        source,
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
    milkdrop_pipeline: PipelineHandle,
    signal_field_pipeline: PipelineHandle,
    composite_pipeline: PipelineHandle,
    present_pipeline: PipelineHandle,
    spectrum_bar_pipeline: PipelineHandle,
    milkdrop_additive_pipeline: PipelineHandle,
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
    selected_visualizer: VisualizerSelection,
    milkdrop_runtime: MilkDropSelectedRuntime,
    milkdrop_controls: MilkDropClassicFrameControls,
    milkdrop_custom_shapes_uploaded: bool,
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
        let initial_viewport = VisualizerViewport::new(1, 1).expect("unit viewport is valid");
        let initial_observation = source
            .frame(0, initial_viewport)
            .expect("initial synthetic visualizer frame is valid");
        let initial_signal = initial_observation.shader_signal();
        let mut milkdrop_runtime = MilkDropSelectedRuntime::from_source(MILKDROP_SELECTED_FIXTURE)
            .expect("Tokimu-authored MilkDrop fixture is valid");
        let milkdrop_controls = milkdrop_runtime
            .step_with_audio(
                0,
                initial_observation.time_seconds,
                [initial_signal[1], initial_signal[2], initial_signal[3]],
                &initial_observation.waveform,
                &initial_observation.spectrum,
            )
            .expect("initial MilkDrop controls are valid");
        Self {
            renderer: None,
            window: None,
            pipeline: PipelineHandle(0),
            milkdrop_pipeline: PipelineHandle(0),
            signal_field_pipeline: PipelineHandle(0),
            composite_pipeline: PipelineHandle(0),
            present_pipeline: PipelineHandle(0),
            spectrum_bar_pipeline: PipelineHandle(0),
            milkdrop_additive_pipeline: PipelineHandle(0),
            viewport: initial_viewport,
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
            selected_visualizer: VisualizerSelection::Native(NativeVisualizerKind::FeedbackBloom),
            milkdrop_runtime,
            milkdrop_controls,
            milkdrop_custom_shapes_uploaded: false,
        }
    }

    fn reset(&mut self) -> PlatformResult<()> {
        self.visualizer_frame = 0;
        self.step_accumulator = 0.0;
        self.history_a_is_previous = true;
        self.feedback_reset_pending = true;
        self.feedback_warm_observed = false;
        self.milkdrop_custom_shapes_uploaded = false;
        self.milkdrop_runtime.reset()?;
        self.update_milkdrop_controls()?;
        Ok(())
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

    fn upload_milkdrop_material_bindings(
        renderer: &mut WgpuBackend,
        controls: [f32; 4],
    ) -> PlatformResult<()> {
        for (material, target, label) in [
            (
                MILKDROP_FROM_HISTORY_A,
                HISTORY_A,
                "milkdrop-classic-from-history-a",
            ),
            (
                MILKDROP_FROM_HISTORY_B,
                HISTORY_B,
                "milkdrop-classic-from-history-b",
            ),
        ] {
            renderer.upload_material(
                material,
                &Material::new(
                    label,
                    Color::rgba(controls[0], controls[1], controls[2], controls[3]),
                )
                .with_texture(target),
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

    fn cycle_fixture(&mut self, direction: isize) -> PlatformResult<()> {
        let count = SyntheticAudioFixture::ALL.len() as isize;
        self.fixture_index = (self.fixture_index as isize + direction).rem_euclid(count) as usize;
        self.source
            .set_fixture(SyntheticAudioFixture::ALL[self.fixture_index]);
        self.reset()
    }

    fn select_visualizer(&mut self, selection: VisualizerSelection) {
        if self.selected_visualizer == selection {
            return;
        }
        self.selected_visualizer = selection;
        self.history_a_is_previous = true;
        self.feedback_reset_pending = true;
        self.feedback_warm_observed = false;
    }

    fn advance(&mut self, delta_seconds: f64) -> PlatformResult<()> {
        if self.paused {
            return Ok(());
        }
        self.step_accumulator += (delta_seconds as f32).clamp(0.0, 0.1) * self.time_scale;
        while self.step_accumulator >= FIXED_STEP_SECONDS {
            self.visualizer_frame = self.visualizer_frame.saturating_add(1);
            self.step_accumulator -= FIXED_STEP_SECONDS;
            self.update_milkdrop_controls()?;
        }
        Ok(())
    }

    fn update_milkdrop_controls(&mut self) -> PlatformResult<()> {
        let observation = self.source.frame(self.visualizer_frame, self.viewport)?;
        let signal = observation.shader_signal();
        self.milkdrop_controls = self.milkdrop_runtime.step_with_audio(
            self.visualizer_frame,
            observation.time_seconds,
            [signal[1], signal[2], signal[3]],
            &observation.waveform,
            &observation.spectrum,
        )?;
        Ok(())
    }

    fn render(&mut self) -> PlatformResult<FrameOutcome> {
        match self.selected_visualizer {
            VisualizerSelection::Native(NativeVisualizerKind::SignalField) => {
                self.render_signal_field()
            }
            VisualizerSelection::Native(NativeVisualizerKind::FeedbackBloom) => {
                self.render_feedback_bloom()
            }
            VisualizerSelection::Native(NativeVisualizerKind::SignalComposite) => {
                self.render_signal_composite()
            }
            VisualizerSelection::Native(NativeVisualizerKind::SpectrumBars) => {
                self.render_spectrum_bars()
            }
            VisualizerSelection::MilkDropClassic => self.render_milkdrop_classic(),
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
        self.render_feedback_pass(
            observation,
            signal,
            self.pipeline,
            FEEDBACK_FROM_HISTORY_A,
            FEEDBACK_FROM_HISTORY_B,
            &[],
        )
    }

    fn render_milkdrop_classic(&mut self) -> PlatformResult<FrameOutcome> {
        let observation = self.source.frame(self.visualizer_frame, self.viewport)?;
        let aspect = self.feedback_target_viewport.width as f32
            / self.feedback_target_viewport.height as f32;
        if !self.milkdrop_custom_shapes_uploaded {
            let Some(renderer) = self.renderer.as_mut() else {
                return Ok(FrameOutcome::Continue);
            };
            Self::upload_custom_shape_geometry(
                renderer,
                &self.milkdrop_controls.custom_shape_frames,
                aspect,
            )?;
            self.milkdrop_custom_shapes_uploaded = true;
        }
        let overlay_draws = {
            let Some(renderer) = self.renderer.as_mut() else {
                return Ok(FrameOutcome::Continue);
            };
            let mut draws = Vec::new();
            for (slot, frame) in self
                .milkdrop_controls
                .custom_wave_frames
                .iter()
                .take(MAX_NATIVE_CUSTOM_WAVES)
                .enumerate()
            {
                let Some(mesh) = custom_wave_mesh(frame, aspect) else {
                    continue;
                };
                renderer.upload_mesh(MILKDROP_CUSTOM_WAVE_MESHES[slot], &mesh);
                renderer.update_material_color(
                    MILKDROP_CUSTOM_WAVE_MATERIALS[slot],
                    Color::rgba(
                        frame.wave.color[0],
                        frame.wave.color[1],
                        frame.wave.color[2],
                        frame.wave.color[3],
                    ),
                )?;
                draws.push(DrawMeshCommand {
                    mesh: MILKDROP_CUSTOM_WAVE_MESHES[slot],
                    material: MILKDROP_CUSTOM_WAVE_MATERIALS[slot],
                    pipeline: milkdrop_overlay_pipeline(
                        frame.wave.additive,
                        self.spectrum_bar_pipeline,
                        self.milkdrop_additive_pipeline,
                    ),
                    instance: Instance2d::identity(),
                    camera: Some(FEEDBACK_CAMERA),
                    viewport: None,
                });
            }
            for (slot, frame) in self
                .milkdrop_controls
                .custom_shape_frames
                .iter()
                .take(MAX_NATIVE_CUSTOM_SHAPES)
                .enumerate()
            {
                if frame.points.len() < 3 {
                    continue;
                }
                // The selected shape subset is convex. Its mesh was uploaded
                // once per viewport because only presentation aspect changes
                // its native coordinates; source properties remain provider
                // data and do not imply per-frame mesh churn.
                draws.push(DrawMeshCommand {
                    mesh: MILKDROP_CUSTOM_SHAPE_MESHES[slot],
                    material: MILKDROP_CUSTOM_SHAPE_MATERIALS[slot],
                    pipeline: milkdrop_overlay_pipeline(
                        frame.shape.additive,
                        self.spectrum_bar_pipeline,
                        self.milkdrop_additive_pipeline,
                    ),
                    instance: Instance2d::identity(),
                    camera: Some(FEEDBACK_CAMERA),
                    viewport: None,
                });
            }
            draws
        };
        self.render_feedback_pass(
            observation,
            self.milkdrop_controls.shader_signal(),
            self.milkdrop_pipeline,
            MILKDROP_FROM_HISTORY_A,
            MILKDROP_FROM_HISTORY_B,
            &overlay_draws,
        )
    }

    fn upload_custom_shape_geometry(
        renderer: &mut WgpuBackend,
        frames: &[MilkDropCustomShapeFrame],
        aspect: f32,
    ) -> PlatformResult<()> {
        for (slot, frame) in frames.iter().take(MAX_NATIVE_CUSTOM_SHAPES).enumerate() {
            let Some(mesh) = custom_shape_mesh(frame, aspect) else {
                continue;
            };
            renderer.upload_mesh(MILKDROP_CUSTOM_SHAPE_MESHES[slot], &mesh);
            renderer.update_material_color(
                MILKDROP_CUSTOM_SHAPE_MATERIALS[slot],
                Color::rgba(
                    frame.shape.color[0],
                    frame.shape.color[1],
                    frame.shape.color[2],
                    frame.shape.color[3],
                ),
            )?;
        }
        Ok(())
    }

    fn render_feedback_pass(
        &mut self,
        observation: visualizer_tools::VisualizerFrameInput,
        signal: [f32; 4],
        pipeline: PipelineHandle,
        from_history_a: MaterialHandle,
        from_history_b: MaterialHandle,
        overlay_draws: &[DrawMeshCommand],
    ) -> PlatformResult<FrameOutcome> {
        let aspect = self.viewport.width as f32 / self.viewport.height as f32;
        let Some(renderer) = self.renderer.as_mut() else {
            return Ok(FrameOutcome::Continue);
        };

        renderer.begin_frame();
        let (current_target, feedback_material, present_material) = if self.history_a_is_previous {
            (HISTORY_B, from_history_a, PRESENT_HISTORY_B)
        } else {
            (HISTORY_A, from_history_b, PRESENT_HISTORY_A)
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
        let mut draws = Vec::with_capacity(1 + overlay_draws.len());
        draws.push(DrawMeshCommand {
            mesh: QUAD,
            material: feedback_material,
            pipeline,
            instance: Instance2d::identity().with_scale([feedback_aspect, 1.0]),
            camera: Some(FEEDBACK_CAMERA),
            viewport: None,
        });
        draws.extend_from_slice(overlay_draws);
        renderer.draw_meshes_to_render_target(
            current_target,
            Color::rgb(0.015, 0.025, 0.045),
            &draws,
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
        if self.selected_visualizer.uses_feedback()
            && !self.feedback_warm_observed
            && self.visualizer_frame >= FEEDBACK_WARMUP_FRAMES
        {
            let expected_dynamic_mesh_updates = if matches!(
                self.selected_visualizer,
                VisualizerSelection::MilkDropClassic
            ) {
                self.milkdrop_controls
                    .custom_wave_frames
                    .len()
                    .min(MAX_NATIVE_CUSTOM_WAVES) as u32
            } else {
                0
            };
            let expected_mesh_replacements = stats
                .frame
                .mesh_replacements
                .min(expected_dynamic_mesh_updates);
            let churn = steady_state_resource_churn(&stats.frame)
                .saturating_sub(expected_mesh_replacements);
            println!(
                "hello-audio-visualizer warm feedback observation: frame={}, unexpected_resource_churn={}, expected_dynamic_mesh_updates={}, mesh_replacements={}, binding_allocations={}, pipeline_creations={}, mesh_uploads={}, texture_allocations={}, render_targets={}, target_pixels={}, target_estimated_bytes={}",
                self.visualizer_frame,
                churn,
                expected_dynamic_mesh_updates,
                stats.frame.mesh_replacements,
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
                "Tokimu Audio Visualizer | {} | {} | frame={} | {:.2}x | {} | Q signal | W feedback | E composite | X bars | M MilkDrop | Left/Right fixture | Space pause | Up/Down speed | R reset",
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
        // A non-empty placeholder keeps every bounded custom-wave handle
        // valid until the first audio-driven mesh replacement.
        for mesh in MILKDROP_CUSTOM_WAVE_MESHES {
            renderer.upload_mesh(mesh, &Mesh::triangle());
        }
        // Literal selected custom shapes use the same bounded placeholder
        // policy. Their static convex geometry is uploaded before its first
        // MilkDrop presentation frame or after a viewport-aspect change.
        for mesh in MILKDROP_CUSTOM_SHAPE_MESHES {
            renderer.upload_mesh(mesh, &Mesh::triangle());
        }
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
        Self::upload_milkdrop_material_bindings(
            &mut renderer,
            self.milkdrop_controls.shader_signal(),
        )?;
        Self::upload_signal_field_material_bindings(&mut renderer, [0.0; 4])?;
        Self::upload_composite_material_bindings(&mut renderer)?;
        renderer.upload_material(
            SPECTRUM_BAR_MATERIAL,
            &Material::new("visualizer-spectrum-bars", Color::rgb(0.35, 0.95, 0.82)),
        )?;
        for (slot, material) in MILKDROP_CUSTOM_WAVE_MATERIALS.into_iter().enumerate() {
            renderer.upload_material(
                material,
                &Material::new(
                    format!("milkdrop-custom-wave-{slot}"),
                    Color::rgba(0.25, 0.85, 1.0, 0.8),
                ),
            )?;
        }
        for (slot, material) in MILKDROP_CUSTOM_SHAPE_MATERIALS.into_iter().enumerate() {
            renderer.upload_material(
                material,
                &Material::new(
                    format!("milkdrop-custom-shape-{slot}"),
                    Color::rgba(1.0, 0.75, 0.2, 0.65),
                ),
            )?;
        }
        let pipeline =
            Pipeline::custom_wgsl_module("audio-visualizer-feedback", visualizer_shader_module()?)?;
        pipeline.validate_draw_contract(&visualizer_material_definition()?, &Mesh::quad())?;
        self.pipeline = renderer.register_pipeline(&pipeline)?;
        let milkdrop_pipeline = Pipeline::custom_wgsl_module(
            "milkdrop-classic-selected",
            milkdrop_classic_shader_module()?,
        )?;
        milkdrop_pipeline
            .validate_draw_contract(&visualizer_material_definition()?, &Mesh::quad())?;
        self.milkdrop_pipeline = renderer.register_pipeline(&milkdrop_pipeline)?;
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
        let milkdrop_additive_pipeline = Pipeline::new(
            "milkdrop-additive-overlays",
            tokimu::PipelineKind::SolidColor2d,
        )
        .with_render_state(PipelineRenderState {
            blend: BlendMode::Additive,
            ..PipelineRenderState::painter_ordered_2d()
        })?;
        self.milkdrop_additive_pipeline =
            renderer.register_pipeline(&milkdrop_additive_pipeline)?;
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
                        self.milkdrop_custom_shapes_uploaded = false;
                        // Replacing target textures invalidates the views held
                        // by material bind groups. Rebind once at this explicit
                        // lifecycle boundary rather than on every frame.
                        Self::upload_feedback_material_bindings(renderer, [0.0; 4])?;
                        Self::upload_milkdrop_material_bindings(
                            renderer,
                            self.milkdrop_controls.shader_signal(),
                        )?;
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
                KeyCode::ArrowRight => self.cycle_fixture(1)?,
                KeyCode::ArrowLeft => self.cycle_fixture(-1)?,
                KeyCode::ArrowUp => self.time_scale = (self.time_scale * 2.0).min(4.0),
                KeyCode::ArrowDown => self.time_scale = (self.time_scale * 0.5).max(0.25),
                KeyCode::Space => self.paused = !self.paused,
                KeyCode::KeyR => self.reset()?,
                KeyCode::KeyQ => self.select_visualizer(VisualizerSelection::Native(
                    NativeVisualizerKind::SignalField,
                )),
                KeyCode::KeyW => self.select_visualizer(VisualizerSelection::Native(
                    NativeVisualizerKind::FeedbackBloom,
                )),
                KeyCode::KeyE => self.select_visualizer(VisualizerSelection::Native(
                    NativeVisualizerKind::SignalComposite,
                )),
                KeyCode::KeyX => self.select_visualizer(VisualizerSelection::Native(
                    NativeVisualizerKind::SpectrumBars,
                )),
                KeyCode::KeyM => self.select_visualizer(VisualizerSelection::MilkDropClassic),
                _ => {}
            },
            _ => {}
        }
        self.update_title();
        Ok(())
    }

    fn on_frame(&mut self, delta_seconds: f64) -> PlatformResult<FrameOutcome> {
        self.advance(delta_seconds)?;
        self.render()
    }
}

#[cfg(test)]
mod tests {
    use super::{
        composite_shader_module, custom_shape_mesh, custom_wave_mesh,
        milkdrop_classic_shader_module, milkdrop_overlay_pipeline, signal_field_shader_module,
        steady_state_resource_churn, visualizer_material_definition, visualizer_shader_module,
    };
    use milkdrop_tools::{MilkDropCustomShape, MilkDropCustomShapeFrame, MilkDropCustomWaveFrame};
    use tokimu::{
        Color, MaterialDefinition, MaterialDefinitionId, MaterialParameterDeclaration,
        MaterialParameterKind, MaterialParameterValue, Mesh, Pipeline, PipelineDrawContractError,
        PipelineHandle, RenderFrameStats, ShaderDiagnosticStage, ShaderMaterialCompatibilityError,
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
    fn milkdrop_classic_contract_reuses_the_bounded_feedback_material_shape() {
        let pipeline = Pipeline::custom_wgsl_module(
            "milkdrop-classic-selected",
            milkdrop_classic_shader_module().unwrap(),
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
    fn milkdrop_overlay_pipeline_preserves_selected_additive_intent() {
        let alpha = PipelineHandle(41);
        let additive = PipelineHandle(42);

        assert_eq!(milkdrop_overlay_pipeline(false, alpha, additive), alpha);
        assert_eq!(milkdrop_overlay_pipeline(true, alpha, additive), additive);
    }

    #[test]
    fn native_custom_wave_line_lowering_produces_finite_triangle_geometry() {
        let frame = MilkDropCustomWaveFrame {
            wave: milkdrop_tools::MilkDropCustomWave {
                index: 0,
                enabled: true,
                samples: 16,
                spectrum: false,
                dots: false,
                thick: true,
                additive: false,
                scaling: 1.0,
                color: [0.25, 0.85, 1.0, 0.8],
                center: [0.5, 0.5],
            },
            source: milkdrop_tools::MilkDropCustomWaveSampleSource::Waveform,
            points: vec![[0.0, 0.0], [0.5, 0.5], [1.0, 1.0]],
        };

        let mesh = custom_wave_mesh(&frame, 16.0 / 9.0).expect("line geometry");
        assert_eq!(mesh.vertex_count(), 12);
        assert!(mesh
            .positions
            .iter()
            .flatten()
            .all(|value| value.is_finite()));
    }

    #[test]
    fn native_custom_wave_dot_lowering_uses_one_quad_per_point() {
        let frame = MilkDropCustomWaveFrame {
            wave: milkdrop_tools::MilkDropCustomWave {
                index: 0,
                enabled: true,
                samples: 16,
                spectrum: false,
                dots: true,
                thick: false,
                additive: false,
                scaling: 1.0,
                color: [0.25, 0.85, 1.0, 0.8],
                center: [0.5, 0.5],
            },
            source: milkdrop_tools::MilkDropCustomWaveSampleSource::Waveform,
            points: vec![[0.25, 0.25], [0.75, 0.75]],
        };

        let mesh = custom_wave_mesh(&frame, 1.0).expect("dot geometry");
        assert_eq!(mesh.vertex_count(), 12);
    }

    #[test]
    fn native_custom_shape_lowering_uses_a_finite_convex_triangle_fan() {
        let frame = MilkDropCustomShapeFrame {
            shape: MilkDropCustomShape {
                index: 0,
                enabled: true,
                sides: 4,
                additive: false,
                thick_outline: true,
                textured: false,
                center: [0.5, 0.5],
                radius: 0.2,
                angle_radians: 0.0,
                color: [1.0, 0.75, 0.2, 0.65],
            },
            points: vec![[0.3, 0.5], [0.5, 0.3], [0.7, 0.5], [0.5, 0.7]],
        };

        let mesh = custom_shape_mesh(&frame, 16.0 / 9.0).expect("convex shape geometry");
        assert_eq!(mesh.vertex_count(), 6);
        assert!(mesh
            .positions
            .iter()
            .flatten()
            .all(|value| value.is_finite()));
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
