use std::{fs, path::PathBuf, sync::Arc};

use tokimu::{
    run_window_with_app, Camera, CameraHandle, ClearCommand, Color, DrawMeshCommand, FrameOutcome,
    Instance2d, Material, MaterialHandle, Mesh, MeshHandle, NativeWindow, Pipeline, PipelineHandle,
    PipelineKind, PlatformEventHandler, PlatformInputEvent, PlatformResult, RenderCommand,
    Renderer, WgpuBackend, WindowConfig,
};
use ui_tools::{parse_svg_document_vector_paths, tessellate_path_strokes, VectorPath};

const CAMERA: CameraHandle = CameraHandle(1);
const MATERIAL: MaterialHandle = MaterialHandle(1);
const ICONS: [&str; 5] = ["minus", "plus", "x", "check", "arrow-right"];

fn main() -> PlatformResult<()> {
    run_window_with_app(
        WindowConfig {
            title: "Tokimu Hello UI Lucide".into(),
            width: 900,
            height: 600,
        },
        App::default(),
    )
}

#[derive(Default)]
struct App {
    renderer: Option<WgpuBackend>,
    size: [f32; 2],
    pipeline: PipelineHandle,
    strokes: Vec<(MeshHandle, [f32; 2])>,
}

fn icon_paths(name: &str) -> Result<Vec<VectorPath>, String> {
    let relative = format!("target/glyph-corpus/icons/icons/{name}.svg");
    let mut paths = vec![PathBuf::from(&relative)];
    if let Ok(dir) = std::env::current_dir() {
        paths.extend(dir.ancestors().map(|path| path.join(&relative)));
    }
    if let Ok(exe) = std::env::current_exe() {
        paths.extend(exe.ancestors().map(|path| path.join(&relative)));
    }
    let path = paths
        .into_iter()
        .find(|path| path.is_file())
        .ok_or_else(|| "run prepare-glyph-corpus.ps1 first".to_string())?;
    let svg = fs::read_to_string(path).map_err(|error| error.to_string())?;
    parse_svg_document_vector_paths(&svg, 32, [0.0, 0.0, 24.0, 24.0])
}

impl PlatformEventHandler for App {
    fn on_native_window_created(&mut self, window: Arc<NativeWindow>) -> PlatformResult<()> {
        let size = window.inner_size();
        self.size = [size.width.max(1) as f32, size.height.max(1) as f32];
        let mut renderer = WgpuBackend::for_window(window, size.width, size.height)?;
        renderer.upload_material(
            MATERIAL,
            &Material::new("lucide", Color::rgb(0.45, 0.68, 0.92)),
        )?;

        for (icon_index, name) in ICONS.iter().enumerate() {
            let center_x = (icon_index as f32 - 2.0) * 0.62;
            let paths = icon_paths(name)?;
            let handle = MeshHandle(10 + icon_index as u64);
            let mesh =
                Mesh::uniform_normal(tessellate_path_strokes(&paths, 0.025), [0.0, 0.0, 1.0]);
            renderer.upload_mesh(handle, &mesh);
            self.strokes.push((handle, [center_x, 0.0]));
        }
        self.pipeline =
            renderer.register_pipeline(&Pipeline::new("lucide", PipelineKind::SolidColor2d))?;
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

    fn on_frame(&mut self, _delta_seconds: f64) -> PlatformResult<FrameOutcome> {
        let Some(renderer) = self.renderer.as_mut() else {
            return Ok(FrameOutcome::Continue);
        };
        renderer.upload_camera(CAMERA, Camera::orthographic_2d(self.size[0], self.size[1]));
        renderer.begin_frame();
        renderer.submit(&[RenderCommand::Clear(ClearCommand {
            color: Color::rgb(0.05, 0.06, 0.08),
        })]);
        let commands = self
            .strokes
            .iter()
            .map(|(mesh, center)| {
                RenderCommand::DrawMesh(DrawMeshCommand {
                    mesh: *mesh,
                    material: MATERIAL,
                    pipeline: self.pipeline,
                    instance: Instance2d::new(*center, [0.55, 0.55], 0.0),
                    camera: Some(CAMERA),
                    viewport: None,
                })
            })
            .collect::<Vec<_>>();
        renderer.submit(&commands);
        let _ = renderer.present()?;
        Ok(FrameOutcome::Continue)
    }
}
