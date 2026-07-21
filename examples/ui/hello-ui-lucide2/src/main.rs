use std::{fs, path::PathBuf, sync::Arc};

use tokimu::{
    run_window_with_app, Camera, CameraHandle, ClearCommand, Color, DrawMeshCommand,
    FrameOutcome, Instance2d, Material, MaterialHandle, Mesh, MeshHandle, NativeWindow, Pipeline,
    MouseButton, PipelineHandle, PipelineKind, PlatformEventHandler, PlatformInputEvent, PlatformResult,
    RenderCommand, Renderer, WgpuBackend, WindowConfig,
};
use ui_tools::{parse_svg_document_paths, stroke_paths as tessellate_stroke_paths, window_to_world};

const CAMERA: CameraHandle = CameraHandle(1);
const MATERIAL: MaterialHandle = MaterialHandle(1);
const LUCIDE_STROKE_HALF_WIDTH: f32 = 1.0 / 32.0;

fn main() -> PlatformResult<()> {
    run_window_with_app(
        WindowConfig { title: "Tokimu Hello UI Lucide 2".into(), width: 900, height: 600 },
        App::default(),
    )
}

#[derive(Default)]
struct App {
    renderer: Option<WgpuBackend>,
    window: Option<Arc<NativeWindow>>,
    size: [f32; 2],
    pipeline: PipelineHandle,
    meshes: Vec<Option<(MeshHandle, [f32; 2], [f32; 2], f32)>>,
    icon_names: Vec<String>,
    cursor_position: [f32; 2],
    selected: Option<usize>,
}

impl App {
    fn update_window_title(&self) {
        let Some(window) = self.window.as_ref() else { return; };
        let title = self.selected
            .and_then(|index| self.icon_names.get(index))
            .map(|name| format!("Tokimu Hello UI Lucide 2 | {name} | click canvas to return"))
            .unwrap_or_else(|| "Tokimu Hello UI Lucide 2 | 100-icon grid".to_owned());
        window.set_title(&title);
    }
}

fn sample_manifest() -> Result<Vec<String>, String> {
    let relative = "target/lucide-corpus-100/manifest.txt";
    let mut candidates = vec![PathBuf::from(relative)];
    if let Ok(dir) = std::env::current_dir() { candidates.extend(dir.ancestors().map(|path| path.join(relative))); }
    let path = candidates.into_iter().find(|path| path.is_file()).ok_or_else(|| "run prepare-lucide-sample.ps1 first".to_string())?;
    Ok(fs::read_to_string(path).map_err(|error| error.to_string())?.lines().map(str::trim).filter(|line| !line.is_empty()).map(str::to_owned).collect())
}

fn sample_svg_paths(name: &str) -> Result<Vec<Vec<[f32; 2]>>, String> {
    let relative = format!("target/lucide-corpus-100/{name}");
    let mut candidates = vec![PathBuf::from(&relative)];
    if let Ok(dir) = std::env::current_dir() { candidates.extend(dir.ancestors().map(|path| path.join(&relative))); }
    let path = candidates.into_iter().find(|path| path.is_file()).ok_or_else(|| format!("missing sampled icon {name}"))?;
    let svg = fs::read_to_string(path).map_err(|error| error.to_string())?;
    parse_svg_document_paths(&svg, 32, [0.0, 0.0, 24.0, 24.0])
}

impl PlatformEventHandler for App {
    fn on_native_window_created(&mut self, window: Arc<NativeWindow>) -> PlatformResult<()> {
        self.window = Some(window.clone());
        let size = window.inner_size();
        self.size = [size.width.max(1) as f32, size.height.max(1) as f32];
        let mut renderer = WgpuBackend::for_window(window, size.width, size.height)?;
        renderer.upload_material(MATERIAL, &Material::new("lucide2", Color::rgb(0.45, 0.68, 0.92)))?;
        self.meshes.clear();
        let sampled = sample_manifest().map_err(|error| error.to_string())?.into_iter().take(100).collect::<Vec<_>>();
        self.icon_names = sampled.clone();
        for (index, name) in sampled.into_iter().take(100).enumerate() {
            let paths = sample_svg_paths(&name).map_err(|error| error.to_string())?;
            let row = index / 10;
            let column = index % 10;
            let cell = [-0.86 + column as f32 * 0.19, 0.84 - row as f32 * 0.19];
            if paths.is_empty() {
                self.meshes.push(None);
                continue;
            }
            let handle = MeshHandle(1000 + index as u64);
            renderer.upload_mesh(handle, &Mesh::uniform_normal(tessellate_stroke_paths(&paths, LUCIDE_STROKE_HALF_WIDTH), [0.0, 0.0, 1.0]));
            self.meshes.push(Some((handle, cell, [0.24, 0.24], 0.0)));
        }
        self.pipeline = renderer.register_pipeline(&Pipeline::new("lucide2", PipelineKind::SolidColor2d))?;
        self.renderer = Some(renderer);
        self.update_window_title();
        Ok(())
    }

    fn on_platform_event(&mut self, event: PlatformInputEvent) -> PlatformResult<()> {
        if let PlatformInputEvent::CursorMoved { x, y } = event {
            self.cursor_position = [x, y];
        }
        if let PlatformInputEvent::MouseInput { button: MouseButton::Left, pressed: true } = event {
            if self.selected.is_some() {
                self.selected = None;
                self.update_window_title();
                return Ok(());
            }
            let [world_x, world_y] = window_to_world(self.size, self.cursor_position);
            let column = ((world_x + 0.86 + 0.095) / 0.19).floor() as i32;
            let row = ((0.84 - world_y + 0.095) / 0.19).floor() as i32;
            self.selected = if (0..10).contains(&column) && (0..10).contains(&row)
                && world_x >= -0.86 + column as f32 * 0.19 - 0.095
                && world_x <= -0.86 + column as f32 * 0.19 + 0.095
                && world_y <= 0.84 - row as f32 * 0.19 + 0.095
                && world_y >= 0.84 - row as f32 * 0.19 - 0.095
            {
                Some(row as usize * 10 + column as usize)
            } else {
                None
            };
            self.update_window_title();
        }
        if let PlatformInputEvent::Resized { width, height } = event {
            self.size = [width.max(1) as f32, height.max(1) as f32];
            if let Some(renderer) = self.renderer.as_mut() { renderer.resize_surface(width, height); }
        }
        Ok(())
    }

    fn on_frame(&mut self, _delta_seconds: f64) -> PlatformResult<FrameOutcome> {
        let Some(renderer) = self.renderer.as_mut() else { return Ok(FrameOutcome::Continue); };
        renderer.upload_camera(CAMERA, Camera::orthographic_2d(self.size[0], self.size[1]));
        renderer.begin_frame();
        renderer.submit(&[RenderCommand::Clear(ClearCommand { color: Color::rgb(0.05, 0.06, 0.08) })]);
        let commands = self.meshes.iter().enumerate().filter_map(|(index, cell)| {
            if let Some(selected) = self.selected {
                if selected != index { return None; }
            }
            let (mesh, center, scale, rotation) = cell.as_ref()?;
            let draw_scale = if self.selected == Some(index) { [0.8, 0.8] } else { *scale };
            Some(RenderCommand::DrawMesh(DrawMeshCommand {
            mesh: *mesh, material: MATERIAL, pipeline: self.pipeline,
            instance: Instance2d::new(if self.selected == Some(index) { [0.0, 0.0] } else { *center }, draw_scale, *rotation), camera: Some(CAMERA), viewport: None,
            }))
        }).collect::<Vec<_>>();
        renderer.submit(&commands);
        let _ = renderer.present()?;
        Ok(FrameOutcome::Continue)
    }
}
