use std::{fs, path::PathBuf, sync::Arc};

use tokimu::{
    run_window_with_app, Camera, CameraHandle, ClearCommand, Color, DrawMeshCommand, FrameOutcome,
    Instance2d, Material, MaterialHandle, Mesh, MeshHandle, NativeWindow, Pipeline, PipelineHandle,
    PipelineKind, PlatformEventHandler, PlatformInputEvent, PlatformResult, RenderCommand,
    Renderer, WgpuBackend, WindowConfig,
};

const MESH: MeshHandle = MeshHandle(1);
const CAMERA: CameraHandle = CameraHandle(1);
const BACKDROP: MaterialHandle = MaterialHandle(1);
const PANEL: MaterialHandle = MaterialHandle(2);
const ACCENT: MaterialHandle = MaterialHandle(3);

const SCENE_WIDTH: u32 = 640;
const SCENE_HEIGHT: u32 = 360;

fn scene_color(rgb: [u8; 3]) -> Color {
    Color::rgb(
        srgb_to_linear(rgb[0]),
        srgb_to_linear(rgb[1]),
        srgb_to_linear(rgb[2]),
    )
}

fn srgb_to_linear(value: u8) -> f32 {
    let value = value as f32 / 255.0;
    if value <= 0.04045 {
        value / 12.92
    } else {
        ((value + 0.055) / 1.055).powf(2.4)
    }
}

#[derive(Clone, Copy)]
struct Rect {
    x: u32,
    y: u32,
    width: u32,
    height: u32,
    color: [u8; 3],
}

const SCENE: [Rect; 3] = [
    Rect {
        x: 0,
        y: 0,
        width: SCENE_WIDTH,
        height: SCENE_HEIGHT,
        color: [13, 15, 20],
    },
    Rect {
        x: 80,
        y: 55,
        width: 480,
        height: 250,
        color: [78, 88, 108],
    },
    Rect {
        x: 120,
        y: 95,
        width: 400,
        height: 120,
        color: [142, 190, 232],
    },
];

fn scene_pixels() -> Vec<u8> {
    let mut pixels = vec![0; (SCENE_WIDTH * SCENE_HEIGHT * 3) as usize];
    for rect in SCENE {
        for y in rect.y..(rect.y + rect.height).min(SCENE_HEIGHT) {
            for x in rect.x..(rect.x + rect.width).min(SCENE_WIDTH) {
                let index = ((y * SCENE_WIDTH + x) * 3) as usize;
                pixels[index..index + 3].copy_from_slice(&rect.color);
            }
        }
    }
    pixels
}

fn write_bmp(path: &PathBuf, pixels: &[u8]) -> Result<(), String> {
    let row_size = SCENE_WIDTH * 3;
    let padding = (4 - row_size % 4) % 4;
    let image_size = (row_size + padding) * SCENE_HEIGHT;
    let file_size = 54 + image_size;
    let mut bytes = Vec::with_capacity(file_size as usize);
    bytes.extend_from_slice(b"BM");
    bytes.extend_from_slice(&file_size.to_le_bytes());
    bytes.extend_from_slice(&[0; 4]);
    bytes.extend_from_slice(&54_u32.to_le_bytes());
    bytes.extend_from_slice(&40_u32.to_le_bytes());
    bytes.extend_from_slice(&SCENE_WIDTH.to_le_bytes());
    bytes.extend_from_slice(&SCENE_HEIGHT.to_le_bytes());
    bytes.extend_from_slice(&1_u16.to_le_bytes());
    bytes.extend_from_slice(&24_u16.to_le_bytes());
    bytes.extend_from_slice(&[0; 4]);
    bytes.extend_from_slice(&image_size.to_le_bytes());
    bytes.extend_from_slice(&[0; 16]);
    for y in (0..SCENE_HEIGHT).rev() {
        for x in 0..SCENE_WIDTH {
            let index = ((y * SCENE_WIDTH + x) * 3) as usize;
            bytes.extend_from_slice(&[pixels[index + 2], pixels[index + 1], pixels[index]]);
        }
        bytes.extend(std::iter::repeat_n(0, padding as usize));
    }
    fs::write(path, bytes).map_err(|error| format!("write {}: {error}", path.display()))
}

fn save_scene() -> Result<PathBuf, String> {
    let path = PathBuf::from("target/hello-save-image/scene.bmp");
    fs::create_dir_all(path.parent().expect("scene has parent"))
        .map_err(|error| error.to_string())?;
    write_bmp(&path, &scene_pixels())?;
    Ok(path)
}

fn main() -> PlatformResult<()> {
    let path = save_scene().map_err(|error| -> Box<dyn std::error::Error> { error.into() })?;
    run_window_with_app(
        WindowConfig {
            title: format!("Tokimu Hello Save Image | {}", path.display()),
            width: SCENE_WIDTH,
            height: SCENE_HEIGHT,
        },
        App::default(),
    )
}

#[derive(Default)]
struct App {
    renderer: Option<WgpuBackend>,
    size: [f32; 2],
    pipeline: PipelineHandle,
}

impl PlatformEventHandler for App {
    fn on_native_window_created(&mut self, window: Arc<NativeWindow>) -> PlatformResult<()> {
        let size = window.inner_size();
        self.size = [size.width.max(1) as f32, size.height.max(1) as f32];
        let mut renderer = WgpuBackend::for_window(window, size.width, size.height)?;
        renderer.upload_mesh(MESH, &Mesh::quad());
        renderer.upload_material(
            BACKDROP,
            &Material::new("save-backdrop", scene_color(SCENE[0].color)),
        )?;
        renderer.upload_material(
            PANEL,
            &Material::new("save-panel", scene_color(SCENE[1].color)),
        )?;
        renderer.upload_material(
            ACCENT,
            &Material::new("save-accent", scene_color(SCENE[2].color)),
        )?;
        self.pipeline = renderer.register_pipeline(&Pipeline::new(
            "hello-save-image",
            PipelineKind::SolidColor2d,
        ))?;
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
        renderer.upload_camera(
            CAMERA,
            Camera::orthographic_2d(SCENE_WIDTH as f32, SCENE_HEIGHT as f32),
        );
        renderer.begin_frame();
        renderer.submit(&[RenderCommand::Clear(ClearCommand {
            color: scene_color(SCENE[0].color),
        })]);
        for (index, rect) in SCENE.iter().enumerate().skip(1) {
            let material = if index == 1 { PANEL } else { ACCENT };
            let center_x = rect.x as f32 + rect.width as f32 * 0.5;
            let center_y = rect.y as f32 + rect.height as f32 * 0.5;
            renderer.submit(&[RenderCommand::DrawMesh(DrawMeshCommand {
                mesh: MESH,
                material,
                pipeline: self.pipeline,
                instance: Instance2d::new(
                    [
                        (center_x - SCENE_WIDTH as f32 * 0.5) / (SCENE_HEIGHT as f32 * 0.5),
                        (SCENE_HEIGHT as f32 * 0.5 - center_y) / (SCENE_HEIGHT as f32 * 0.5),
                    ],
                    [
                        rect.width as f32 / (SCENE_HEIGHT as f32 * 0.5),
                        rect.height as f32 / (SCENE_HEIGHT as f32 * 0.5),
                    ],
                    0.0,
                ),
                camera: Some(CAMERA),
                viewport: None,
            })]);
        }
        let _ = renderer.present()?;
        Ok(FrameOutcome::Continue)
    }
}
