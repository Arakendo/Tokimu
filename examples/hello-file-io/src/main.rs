use std::{fs, path::PathBuf, sync::Arc};

use tokimu::{
    run_window_with_app, Camera, CameraHandle, ClearCommand, Color, DrawMeshCommand, FrameOutcome,
    Instance2d, Material, MaterialHandle, Mesh, MeshHandle, NativeWindow, Pipeline, PipelineHandle,
    PipelineKind, PlatformEventHandler, PlatformInputEvent, PlatformResult, RenderCommand,
    Renderer, WgpuBackend, WindowConfig,
};
use ui_tools::{layout_bitmap_text, UiRect, UiTextAlign, UiTextRole, UiTextSpec};

const GLYPH_MESH: MeshHandle = MeshHandle(1);
const CAMERA: CameraHandle = CameraHandle(1);
const TITLE: MaterialHandle = MaterialHandle(1);
const BODY: MaterialHandle = MaterialHandle(2);
const STATUS: MaterialHandle = MaterialHandle(3);

fn round_trip() -> Result<(PathBuf, usize), String> {
    let path = PathBuf::from("target/hello-file-io/roundtrip.txt");
    let payload = "Tokimu file IO corpus v1\nread after write\n";
    fs::create_dir_all(path.parent().expect("fixture has parent"))
        .map_err(|error| format!("create directory: {error}"))?;
    fs::write(&path, payload).map_err(|error| format!("write {}: {error}", path.display()))?;
    let contents =
        fs::read_to_string(&path).map_err(|error| format!("read {}: {error}", path.display()))?;
    if contents != payload {
        return Err(format!("round-trip mismatch: read {contents:?}"));
    }
    Ok((path, contents.len()))
}

fn text_commands(
    spec: &UiTextSpec,
    height: f32,
    material: MaterialHandle,
    pipeline: PipelineHandle,
) -> Vec<RenderCommand> {
    layout_bitmap_text(spec, height)
        .into_iter()
        .map(|quad| {
            RenderCommand::DrawMesh(DrawMeshCommand {
                mesh: GLYPH_MESH,
                material,
                pipeline,
                instance: Instance2d::new(quad.center, quad.size, 0.0),
                camera: Some(CAMERA),
                viewport: None,
            })
        })
        .collect()
}

fn label(text: impl Into<String>, y: f32, role: UiTextRole) -> UiTextSpec {
    UiTextSpec::new(text, UiRect::new([0.0, y], [1.6, 0.10]), role)
        .with_alignment(UiTextAlign::Center, UiTextAlign::Center)
}

fn main() -> PlatformResult<()> {
    run_window_with_app(
        WindowConfig {
            title: "Tokimu Hello File IO | read/write round trip".into(),
            width: 900,
            height: 540,
        },
        HelloFileIo {
            result: round_trip(),
            ..Default::default()
        },
    )
}

struct HelloFileIo {
    renderer: Option<WgpuBackend>,
    size: [f32; 2],
    pipeline: PipelineHandle,
    result: Result<(PathBuf, usize), String>,
}

impl Default for HelloFileIo {
    fn default() -> Self {
        Self {
            renderer: None,
            size: [1.0, 1.0],
            pipeline: PipelineHandle(0),
            result: Err("file IO proof has not run".to_owned()),
        }
    }
}

impl PlatformEventHandler for HelloFileIo {
    fn on_native_window_created(&mut self, window: Arc<NativeWindow>) -> PlatformResult<()> {
        let size = window.inner_size();
        self.size = [size.width.max(1) as f32, size.height.max(1) as f32];
        let mut renderer = WgpuBackend::for_window(window, size.width, size.height)?;
        renderer.upload_mesh(GLYPH_MESH, &Mesh::quad());
        renderer.upload_material(
            TITLE,
            &Material::new("file-io-title", Color::rgb(0.92, 0.94, 0.98)),
        )?;
        renderer.upload_material(
            BODY,
            &Material::new("file-io-body", Color::rgb(0.70, 0.78, 0.88)),
        )?;
        renderer.upload_material(
            STATUS,
            &Material::new("file-io-status", Color::rgb(0.45, 0.68, 0.92)),
        )?;
        self.pipeline = renderer
            .register_pipeline(&Pipeline::new("hello-file-io", PipelineKind::SolidColor2d))?;
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
        let (status, details) = match &self.result {
            Ok((path, bytes)) => (
                "ROUND TRIP PASSED".to_owned(),
                format!("{bytes} bytes | {}", path.display()),
            ),
            Err(error) => ("ROUND TRIP FAILED".to_owned(), error.clone()),
        };
        let specs = [
            (label("HELLO FILE IO", 0.28, UiTextRole::Title), 0.07, TITLE),
            (
                label("WRITE / READ / VERIFY", 0.14, UiTextRole::Heading),
                0.045,
                BODY,
            ),
            (label(status, -0.02, UiTextRole::Status), 0.06, STATUS),
            (label(details, -0.14, UiTextRole::Body), 0.035, BODY),
            (
                label(
                    "APPLICATION-OWNED FILESYSTEM PROOF",
                    -0.30,
                    UiTextRole::Caption,
                ),
                0.03,
                BODY,
            ),
        ];
        for (spec, height, material) in specs {
            renderer.submit(&text_commands(&spec, height, material, self.pipeline));
        }
        let _ = renderer.present()?;
        Ok(FrameOutcome::Continue)
    }
}
