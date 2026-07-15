use std::sync::Arc;

use tokimu::{
    run_window_with_app, ClearCommand, Color, DrawMeshCommand, FrameOutcome, Instance2d,
    Material, MaterialHandle, Mesh, MeshHandle, NativeWindow, Pipeline, PipelineHandle,
    PipelineKind, PlatformEventHandler, PlatformInputEvent, PlatformResult, RenderCommand,
    Renderer, WgpuBackend, WindowConfig,
};

const STROKE_MESH: MeshHandle = MeshHandle(1);
const NODE_MESH: MeshHandle = MeshHandle(2);
const PEN_MESH: MeshHandle = MeshHandle(3);
const STROKE_MATERIAL: MaterialHandle = MaterialHandle(1);
const HIGHLIGHT_MATERIAL: MaterialHandle = MaterialHandle(2);
const NODE_MATERIAL: MaterialHandle = MaterialHandle(3);
const PEN_MATERIAL: MaterialHandle = MaterialHandle(4);

const VECTOR_POINTS: [[f32; 2]; 8] = [
    [-0.72, 0.48],
    [0.02, 0.48],
    [0.02, -0.10],
    [-0.72, -0.10],
    [-0.42, 0.18],
    [0.30, 0.18],
    [0.30, -0.40],
    [-0.42, -0.40],
];

const VECTOR_SEGMENTS: [(usize, usize); 12] = [
    (0, 1),
    (1, 2),
    (2, 3),
    (3, 0),
    (4, 5),
    (5, 6),
    (6, 7),
    (7, 4),
    (0, 4),
    (1, 5),
    (2, 6),
    (3, 7),
];

fn main() -> PlatformResult<()> {
    run_window_with_app(
        WindowConfig {
            title: "Tokimu Hello Vector Draw".into(),
            width: 1280,
            height: 720,
        },
        HelloVectorDrawApp::new(),
    )
}

struct HelloVectorDrawApp {
    renderer: Option<WgpuBackend>,
    window: Option<Arc<NativeWindow>>,
    window_size: [f32; 2],
    elapsed_seconds: f64,
    pipeline: PipelineHandle,
}

impl Default for HelloVectorDrawApp {
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

impl HelloVectorDrawApp {
    fn new() -> Self {
        Self::default()
    }

    fn update_window_title(&self, draw_index: usize, progress: f32) {
        if let Some(window) = self.window.as_ref() {
            window.set_title(&format!(
                "Tokimu Hello Vector Draw | segment={} | progress={:.2}",
                draw_index, progress
            ));
        }
    }

    fn render_scene(&mut self) -> PlatformResult<FrameOutcome> {
        let seconds = self.elapsed_seconds as f32;
        let Some(renderer) = self.renderer.as_mut() else {
            return Ok(FrameOutcome::Continue);
        };

        renderer.upload_mesh(STROKE_MESH, &Mesh::quad());
        renderer.upload_mesh(NODE_MESH, &Mesh::diamond());
        renderer.upload_mesh(PEN_MESH, &Mesh::triangle());

        let progress = draw_progress(seconds);
        let active_segment = ((progress * VECTOR_SEGMENTS.len() as f32).floor() as usize)
            .min(VECTOR_SEGMENTS.len() - 1);

        renderer.begin_frame();
        let mut commands = vec![RenderCommand::Clear(ClearCommand {
            color: Color::rgb(0.05, 0.07, 0.10),
        })];

        for (index, &(start_index, end_index)) in VECTOR_SEGMENTS.iter().enumerate() {
            let start = VECTOR_POINTS[start_index];
            let end = VECTOR_POINTS[end_index];
            let is_active = index == active_segment;
            let highlight = index <= active_segment;
            let stroke = stroke_instance(start, end, if is_active { 0.065 } else { 0.038 });
            commands.push(RenderCommand::DrawMesh(DrawMeshCommand {
                mesh: STROKE_MESH,
                material: if highlight {
                    HIGHLIGHT_MATERIAL
                } else {
                    STROKE_MATERIAL
                },
                pipeline: self.pipeline,
                instance: stroke,
                camera: None,
                viewport: None,
            }));
        }

        for (index, point) in VECTOR_POINTS.iter().enumerate() {
            let node = Instance2d::identity()
                .with_translation(*point)
                .with_scale([0.07, 0.07])
                .with_rotation(if index == active_segment { seconds * 0.7 } else { 0.0 });
            commands.push(RenderCommand::DrawMesh(DrawMeshCommand {
                mesh: NODE_MESH,
                material: if index == active_segment {
                    HIGHLIGHT_MATERIAL
                } else {
                    NODE_MATERIAL
                },
                pipeline: self.pipeline,
                instance: node,
                camera: None,
                viewport: None,
            }));
        }

        let (pen_start, pen_end) = VECTOR_SEGMENTS[active_segment];
        let pen = stroke_instance(
            VECTOR_POINTS[pen_start],
            VECTOR_POINTS[pen_end],
            0.10,
        )
        .with_scale([0.12, 0.12])
        .with_rotation(segment_angle(VECTOR_POINTS[pen_start], VECTOR_POINTS[pen_end])
            + std::f32::consts::FRAC_PI_2);
        commands.push(RenderCommand::DrawMesh(DrawMeshCommand {
            mesh: PEN_MESH,
            material: PEN_MATERIAL,
            pipeline: self.pipeline,
            instance: pen,
            camera: None,
            viewport: None,
        }));

        renderer.submit(&commands);
        let _ = renderer.present()?;
        self.update_window_title(active_segment, progress);
        Ok(FrameOutcome::Continue)
    }
}

impl PlatformEventHandler for HelloVectorDrawApp {
    fn on_native_window_created(&mut self, window: Arc<NativeWindow>) -> PlatformResult<()> {
        let size = window.inner_size();
        self.window_size = [size.width.max(1) as f32, size.height.max(1) as f32];
        self.window = Some(window.clone());

        let mut renderer = WgpuBackend::for_window(window, size.width, size.height)?;
        renderer.upload_material(
            STROKE_MATERIAL,
            &Material::new("vector-stroke", Color::rgb(0.44, 0.88, 0.99)),
        )?;
        renderer.upload_material(
            HIGHLIGHT_MATERIAL,
            &Material::new("vector-highlight", Color::rgb(0.98, 0.80, 0.34)),
        )?;
        renderer.upload_material(
            NODE_MATERIAL,
            &Material::new("vector-node", Color::rgb(0.75, 0.55, 0.96)),
        )?;
        renderer.upload_material(
            PEN_MATERIAL,
            &Material::new("vector-pen", Color::rgb(0.95, 0.42, 0.60)),
        )?;
        self.pipeline = renderer.register_pipeline(&Pipeline::new(
            "vector-draw-pipeline",
            PipelineKind::SolidColor2d,
        ))?;
        self.renderer = Some(renderer);
        self.update_window_title(0, 0.0);
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

fn draw_progress(seconds: f32) -> f32 {
    (seconds.rem_euclid(6.0) / 6.0).clamp(0.0, 1.0)
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

fn segment_angle(start: [f32; 2], end: [f32; 2]) -> f32 {
    (end[1] - start[1]).atan2(end[0] - start[0])
}
