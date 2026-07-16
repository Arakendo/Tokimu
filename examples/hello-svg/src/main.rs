use std::sync::Arc;

use tokimu::{
    run_window_with_app, ClearCommand, Color, DrawMeshCommand, FrameOutcome, Instance2d, Material,
    MaterialHandle, Mesh, MeshHandle, NativeWindow, Pipeline, PipelineHandle, PipelineKind,
    PlatformEventHandler, PlatformInputEvent, PlatformResult, RenderCommand, Renderer, WgpuBackend,
    WindowConfig,
};

const FRAME_MESH: MeshHandle = MeshHandle(1);
const STROKE_MESH: MeshHandle = MeshHandle(2);
const ACCENT_MESH: MeshHandle = MeshHandle(3);
const FILL_MESH: MeshHandle = MeshHandle(4);
const FRAME_MATERIAL: MaterialHandle = MaterialHandle(1);
const STROKE_MATERIAL: MaterialHandle = MaterialHandle(2);
const ACCENT_MATERIAL: MaterialHandle = MaterialHandle(3);
const FILL_MATERIAL: MaterialHandle = MaterialHandle(4);

const PATH_POINTS: [[f32; 2]; 7] = [
    [-0.65, -0.25],
    [-0.45, 0.42],
    [-0.12, 0.58],
    [0.34, 0.48],
    [0.58, 0.10],
    [0.28, -0.34],
    [-0.12, -0.48],
];

const PATH_SEGMENTS: [(usize, usize); 6] = [(0, 1), (1, 2), (2, 3), (3, 4), (4, 5), (5, 6)];

fn main() -> PlatformResult<()> {
    run_window_with_app(
        WindowConfig {
            title: "Tokimu Hello SVG".into(),
            width: 1280,
            height: 720,
        },
        HelloSvgApp::new(),
    )
}

struct HelloSvgApp {
    renderer: Option<WgpuBackend>,
    window: Option<Arc<NativeWindow>>,
    window_size: [f32; 2],
    elapsed_seconds: f64,
    pipeline: PipelineHandle,
}

impl Default for HelloSvgApp {
    fn default() -> Self {
        Self {
            renderer: None,
            window: None,
            window_size: [1.0, 1.0],
            elapsed_seconds: 0.0,
            pipeline: PipelineHandle(0),
        }
    }
}

impl HelloSvgApp {
    fn new() -> Self {
        Self::default()
    }

    fn update_window_title(&self, progress: f32) {
        if let Some(window) = self.window.as_ref() {
            window.set_title(&format!("Tokimu Hello SVG | path progress={:.2}", progress));
        }
    }

    fn render_scene(&mut self) -> PlatformResult<FrameOutcome> {
        let seconds = self.elapsed_seconds as f32;
        let Some(renderer) = self.renderer.as_mut() else {
            return Ok(FrameOutcome::Continue);
        };

        renderer.upload_mesh(FRAME_MESH, &frame_mesh());
        renderer.upload_mesh(STROKE_MESH, &Mesh::quad());
        renderer.upload_mesh(ACCENT_MESH, &Mesh::diamond());
        renderer.upload_mesh(FILL_MESH, &Mesh::triangle());

        let progress = svg_progress(seconds);
        let active_segment =
            ((progress * PATH_SEGMENTS.len() as f32).floor() as usize).min(PATH_SEGMENTS.len() - 1);

        renderer.begin_frame();
        let mut commands = vec![RenderCommand::Clear(ClearCommand {
            color: Color::rgb(0.05, 0.06, 0.09),
        })];

        commands.push(RenderCommand::DrawMesh(DrawMeshCommand {
            mesh: FRAME_MESH,
            material: FRAME_MATERIAL,
            pipeline: self.pipeline,
            instance: Instance2d::identity()
                .with_scale([1.0, 1.0])
                .with_rotation(seconds * 0.05),
            camera: None,
            viewport: None,
        }));

        for (index, &(start_index, end_index)) in PATH_SEGMENTS.iter().enumerate() {
            let start = PATH_POINTS[start_index];
            let end = PATH_POINTS[end_index];
            let stroke = stroke_instance(
                start,
                end,
                if index == active_segment {
                    0.075
                } else {
                    0.044
                },
            );
            commands.push(RenderCommand::DrawMesh(DrawMeshCommand {
                mesh: STROKE_MESH,
                material: if index <= active_segment {
                    ACCENT_MATERIAL
                } else {
                    STROKE_MATERIAL
                },
                pipeline: self.pipeline,
                instance: stroke,
                camera: None,
                viewport: None,
            }));
        }

        for (index, point) in PATH_POINTS.iter().enumerate() {
            let pulse = if index == active_segment { 1.0 } else { 0.0 };
            commands.push(RenderMeshPoint::into_draw(
                ACCENT_MESH,
                if index == active_segment {
                    ACCENT_MATERIAL
                } else {
                    STROKE_MATERIAL
                },
                self.pipeline,
                point_instance(*point, 0.06 + pulse * 0.02, seconds + index as f32 * 0.15),
            ));
        }

        let fill = Instance2d::identity()
            .with_translation([0.02, 0.06])
            .with_scale([0.24 + progress * 0.05, 0.24 + progress * 0.05])
            .with_rotation(seconds * 0.2);
        commands.push(RenderCommand::DrawMesh(DrawMeshCommand {
            mesh: FILL_MESH,
            material: FILL_MATERIAL,
            pipeline: self.pipeline,
            instance: fill,
            camera: None,
            viewport: None,
        }));

        let accent = Instance2d::identity()
            .with_translation([0.42 + seconds.sin() * 0.03, -0.05 + seconds.cos() * 0.02])
            .with_scale([0.11, 0.11])
            .with_rotation(-seconds * 0.8);
        commands.push(RenderCommand::DrawMesh(DrawMeshCommand {
            mesh: ACCENT_MESH,
            material: ACCENT_MATERIAL,
            pipeline: self.pipeline,
            instance: accent,
            camera: None,
            viewport: None,
        }));

        renderer.submit(&commands);
        let _ = renderer.present()?;
        self.update_window_title(progress);
        Ok(FrameOutcome::Continue)
    }
}

impl PlatformEventHandler for HelloSvgApp {
    fn on_native_window_created(&mut self, window: Arc<NativeWindow>) -> PlatformResult<()> {
        let size = window.inner_size();
        self.window_size = [size.width.max(1) as f32, size.height.max(1) as f32];
        self.window = Some(window.clone());

        let mut renderer = WgpuBackend::for_window(window, size.width, size.height)?;
        renderer.upload_material(
            FRAME_MATERIAL,
            &Material::new("svg-frame", Color::rgb(0.18, 0.22, 0.30)),
        )?;
        renderer.upload_material(
            STROKE_MATERIAL,
            &Material::new("svg-stroke", Color::rgb(0.39, 0.84, 0.96)),
        )?;
        renderer.upload_material(
            ACCENT_MATERIAL,
            &Material::new("svg-accent", Color::rgb(0.98, 0.81, 0.33)),
        )?;
        renderer.upload_material(
            FILL_MATERIAL,
            &Material::new("svg-fill", Color::rgb(0.79, 0.52, 0.95)),
        )?;
        self.pipeline = renderer.register_pipeline(&Pipeline::new(
            "svg-draw-pipeline",
            PipelineKind::SolidColor2d,
        ))?;
        self.renderer = Some(renderer);
        self.update_window_title(0.0);
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

        Ok(())
    }

    fn on_frame(&mut self, delta_seconds: f64) -> PlatformResult<FrameOutcome> {
        self.elapsed_seconds += delta_seconds;
        self.render_scene()
    }
}

fn svg_progress(seconds: f32) -> f32 {
    (seconds.rem_euclid(5.0) / 5.0).clamp(0.0, 1.0)
}

fn frame_mesh() -> Mesh {
    Mesh::quad()
}

fn stroke_instance(start: [f32; 2], end: [f32; 2], thickness: f32) -> Instance2d {
    let dx = end[0] - start[0];
    let dy = end[1] - start[1];
    let length = (dx * dx + dy * dy).sqrt().max(0.001);
    let angle = dy.atan2(dx);
    Instance2d::identity()
        .with_translation([(start[0] + end[0]) * 0.5, (start[1] + end[1]) * 0.5])
        .with_scale([length, thickness])
        .with_rotation(angle)
}

fn point_instance(point: [f32; 2], size: f32, rotation: f32) -> Instance2d {
    Instance2d::identity()
        .with_translation(point)
        .with_scale([size, size])
        .with_rotation(rotation)
}

struct RenderMeshPoint;

impl RenderMeshPoint {
    fn into_draw(
        mesh: MeshHandle,
        material: MaterialHandle,
        pipeline: PipelineHandle,
        instance: Instance2d,
    ) -> RenderCommand {
        RenderCommand::DrawMesh(DrawMeshCommand {
            mesh,
            material,
            pipeline,
            instance,
            camera: None,
            viewport: None,
        })
    }
}
