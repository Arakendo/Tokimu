use std::{fs, path::PathBuf, sync::Arc};
use tokimu::{run_window_with_app, Camera, CameraHandle, ClearCommand, Color, DrawMeshCommand, FrameOutcome, Instance2d, Material, MaterialHandle, Mesh, MeshHandle, NativeWindow, Pipeline, PipelineHandle, PipelineKind, PlatformEventHandler, PlatformResult, RenderCommand, Renderer, Texture, TextureHandle, WgpuBackend, WindowConfig};
use ui_tools::UiFontRasterizer;

const QUAD: MeshHandle = MeshHandle(1);
const CAMERA: CameraHandle = CameraHandle(1);
const FONT_SIZE: f32 = 46.0;
const LINES: &[&str] = &[
    "Sphinx of black quartz, judge my vow.",
    "The quick brown fox jumps over the lazy dog.",
    "Pack my box with five dozen liquor jugs. 0123456789",
];

fn main() -> PlatformResult<()> {
    run_window_with_app(WindowConfig { title: "Tokimu Hello UI Font 2 | Public-Domain Text Corpus".into(), width: 1100, height: 620 }, App::default())
}

#[derive(Default)]
struct App { renderer: Option<WgpuBackend>, size: [f32; 2], pipeline: PipelineHandle, glyphs: Vec<GlyphDraw> }

struct GlyphDraw { material: MaterialHandle, center: [f32; 2], scale: [f32; 2] }

fn first_otf() -> Option<Vec<u8>> {
    let relative = PathBuf::from("target/glyph-corpus/fonts/noto");
    let mut roots = vec![relative];
    if let Ok(current) = std::env::current_dir() { roots.extend(current.ancestors().map(|path| path.join("target/glyph-corpus/fonts/noto"))); }
    for root in roots {
        let mut pending = vec![root];
        while let Some(path) = pending.pop() {
            let Ok(entries) = fs::read_dir(path) else { continue; };
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_dir() { pending.push(path); }
                else if path.extension().is_some_and(|extension| extension == "otf") {
                    if let Ok(bytes) = fs::read(path) { return Some(bytes); }
                }
            }
        }
    }
    None
}

impl PlatformEventHandler for App {
    fn on_native_window_created(&mut self, window: Arc<NativeWindow>) -> PlatformResult<()> {
        let size = window.inner_size();
        self.size = [size.width.max(1) as f32, size.height.max(1) as f32];
        let mut renderer = WgpuBackend::for_window(window, size.width, size.height)?;
        renderer.upload_mesh(QUAD, &Mesh::quad());
        let bytes = first_otf().ok_or_else(|| "run prepare-glyph-corpus.ps1 first (no OTF found)".to_string())?;
        let font = UiFontRasterizer::from_bytes(bytes).map_err(|error| error.to_string())?;
        self.pipeline = renderer.register_pipeline(&Pipeline::new("font2-texture", PipelineKind::Texture2d))?;
        let mut material_id = 10_u64;
        for (line_index, line) in LINES.iter().enumerate() {
            let bitmap = font.rasterize_text(line, FONT_SIZE);
            let baseline_y = 0.42 - line_index as f32 * 0.28;
            if bitmap.width == 0 || bitmap.height == 0 {
                continue;
            }
            let texture = TextureHandle(material_id);
            let material = MaterialHandle(material_id);
            let rgba = bitmap.alpha.iter().flat_map(|alpha| [0xff, 0xff, 0xff, *alpha]).collect::<Vec<_>>();
            renderer.upload_texture(texture, &Texture::rgba8(bitmap.width, bitmap.height, rgba));
            renderer.upload_material(material, &Material::new("font2-line", Color::rgb(0.78, 0.86, 0.96)).with_texture(texture))?;
            let center = [
                -0.88 + (bitmap.width as f32 * 0.5) * 2.0 / self.size[0],
                baseline_y - (bitmap.top + bitmap.height as f32 * 0.5) * 2.0 / self.size[1],
            ];
            self.glyphs.push(GlyphDraw { material, center, scale: [bitmap.width as f32 / self.size[0], bitmap.height as f32 / self.size[1]] });
            material_id += 1;
        }
        self.renderer = Some(renderer);
        Ok(())
    }

    fn on_platform_event(&mut self, event: tokimu::PlatformInputEvent) -> PlatformResult<()> {
        if let tokimu::PlatformInputEvent::Resized { width, height } = event {
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
        let commands = self.glyphs.iter().map(|glyph| RenderCommand::DrawMesh(DrawMeshCommand {
            mesh: QUAD, material: glyph.material, pipeline: self.pipeline,
            instance: Instance2d::new(glyph.center, glyph.scale, 0.0), camera: Some(CAMERA), viewport: None,
        })).collect::<Vec<_>>();
        renderer.submit(&commands);
        let _ = renderer.present()?;
        Ok(FrameOutcome::Continue)
    }
}
