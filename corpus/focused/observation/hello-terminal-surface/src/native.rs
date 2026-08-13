use std::{
    sync::Arc,
    time::{Duration, Instant},
};

use tokimu::{
    run_window_with_app, Camera, CameraHandle, ClearCommand, Color, DrawMeshCommand, FrameOutcome,
    Instance2d, Material, MaterialHandle, Mesh, MeshHandle, NativeWindow, Pipeline, PipelineHandle,
    PipelineKind, PlatformEventHandler, PlatformInputEvent, PlatformResult, RenderCommand,
    Renderer, Rgba8TextureColorSpace, Rgba8TextureDescriptor, TextureHandle, WgpuBackend,
    WindowConfig,
};

use crate::{presentation::TerminalSurfaceRaster, FixtureProducer};

const QUAD: MeshHandle = MeshHandle(1);
const CAMERA: CameraHandle = CameraHandle(1);
const TEXTURE: TextureHandle = TextureHandle(1);
const MATERIAL: MaterialHandle = MaterialHandle(1);
const BACKGROUND: Color = Color::rgb(0.018, 0.035, 0.04);

/// Submit the already-authoritative CPU raster through the native renderer.
///
/// This viewer intentionally does not re-run terminal layout, cell resolution,
/// Ratatui composition, or font rasterization. It is execution evidence for a
/// texture upload and draw only; it makes no framebuffer-readback claim.
pub(crate) fn run(producer: FixtureProducer) -> Result<(), String> {
    let raster = producer.raster()?;
    let title = format!(
        "Tokimu Hello Terminal Surface | {} raster {}x{}",
        producer.name(),
        raster.width,
        raster.height
    );
    run_window_with_app(
        WindowConfig {
            title,
            width: 960,
            height: 640,
        },
        App::new(raster),
    )
    .map_err(|error| error.to_string())
}

struct App {
    renderer: Option<WgpuBackend>,
    raster: Option<TerminalSurfaceRaster>,
    size: [f32; 2],
    pipeline: PipelineHandle,
    startup_started: Instant,
    frame_index: u64,
    last_performance_report: Instant,
}

impl App {
    fn new(raster: TerminalSurfaceRaster) -> Self {
        let now = Instant::now();
        Self {
            renderer: None,
            raster: Some(raster),
            size: [0.0, 0.0],
            pipeline: PipelineHandle::default(),
            startup_started: now,
            frame_index: 0,
            last_performance_report: now,
        }
    }
}

impl PlatformEventHandler for App {
    fn on_native_window_created(&mut self, window: Arc<NativeWindow>) -> PlatformResult<()> {
        let size = window.inner_size();
        self.size = [size.width.max(1) as f32, size.height.max(1) as f32];

        let mut renderer = WgpuBackend::for_window(window, size.width, size.height)?;
        renderer.upload_mesh(QUAD, &Mesh::quad());
        self.pipeline = renderer.register_pipeline(&Pipeline::new(
            "hello-terminal-surface-native-raster",
            PipelineKind::Texture2d,
        ))?;

        let raster = self
            .raster
            .as_ref()
            .expect("the native terminal viewer receives a CPU raster before startup");
        renderer.create_texture_rgba8(
            TEXTURE,
            Rgba8TextureDescriptor::new(raster.width, raster.height, Rgba8TextureColorSpace::Srgb),
            &raster.rgba,
        )?;
        renderer.upload_material(
            MATERIAL,
            &Material::new("terminal-surface-cpu-raster", Color::rgb(1.0, 1.0, 1.0))
                .with_texture(TEXTURE),
        )?;
        println!(
            "hello-terminal-surface native startup: window_ready_cpu_ms={:.3}, cpu_fingerprint={:016x}, raster={}x{}, texture_upload=1, framebuffer_readback=false",
            self.startup_started.elapsed().as_secs_f64() * 1000.0,
            raster.fingerprint(),
            raster.width,
            raster.height,
        );
        self.renderer = Some(renderer);
        Ok(())
    }

    fn on_platform_event(&mut self, event: PlatformInputEvent) -> PlatformResult<()> {
        if let PlatformInputEvent::Resized { width, height } = event {
            self.size = [width.max(1) as f32, height.max(1) as f32];
            if let Some(renderer) = self.renderer.as_mut() {
                renderer.resize_surface(width, height);
            }
        }
        Ok(())
    }

    fn on_frame(&mut self, delta_seconds: f64) -> PlatformResult<FrameOutcome> {
        let Some(renderer) = self.renderer.as_mut() else {
            return Ok(FrameOutcome::Continue);
        };
        let Some(raster) = self.raster.as_ref() else {
            return Ok(FrameOutcome::Continue);
        };

        renderer.upload_camera(CAMERA, Camera::orthographic_2d(self.size[0], self.size[1]));
        renderer.begin_frame();
        renderer.submit(&[RenderCommand::Clear(ClearCommand { color: BACKGROUND })]);

        let width = raster.width as f32;
        let height = raster.height as f32;
        let fit = ((self.size[0] * 0.86) / width)
            .min((self.size[1] * 0.74) / height)
            .max(0.0);
        renderer.submit(&[RenderCommand::DrawMesh(DrawMeshCommand {
            mesh: QUAD,
            material: MATERIAL,
            pipeline: self.pipeline,
            instance: Instance2d::new(
                [0.0, 0.0],
                [width * fit / self.size[0], height * fit / self.size[1]],
                0.0,
            ),
            camera: Some(CAMERA),
            viewport: None,
        })]);
        let present_started = Instant::now();
        let stats = renderer.present()?;
        let present_elapsed = present_started.elapsed();
        if self.frame_index < 3 || self.last_performance_report.elapsed() >= Duration::from_secs(2)
        {
            let timings = stats.frame.cpu_timings;
            println!(
                "hello-terminal-surface native frame {}: platform_frame_interval_ms={:.3}, renderer_present_call_cpu_ms={:.3}, surface_acquire_call_cpu_ms={:.3}, resource_preparation_cpu_ms={:.3}, command_encoding_cpu_ms={:.3}, queue_submit_call_cpu_ms={:.3}, surface_present_call_cpu_ms={:.3}, draws={}, submits={}, binding_allocations={}, uniform_writes={}, mesh_uploads={}, texture_allocations={}",
                self.frame_index,
                delta_seconds * 1000.0,
                present_elapsed.as_secs_f64() * 1000.0,
                timings.surface_acquire_call.unwrap_or_default().as_secs_f64() * 1000.0,
                timings.resource_preparation.unwrap_or_default().as_secs_f64() * 1000.0,
                timings.command_encoding.unwrap_or_default().as_secs_f64() * 1000.0,
                timings.queue_submit_call.unwrap_or_default().as_secs_f64() * 1000.0,
                timings.surface_present_call.unwrap_or_default().as_secs_f64() * 1000.0,
                stats.frame.draw_calls,
                stats.frame.submit_calls,
                stats.frame.binding_allocations,
                stats.frame.uniform_buffer_writes,
                stats.frame.mesh_uploads,
                stats.frame.texture_allocations,
            );
            self.last_performance_report = Instant::now();
        }
        self.frame_index += 1;
        Ok(FrameOutcome::Continue)
    }
}
