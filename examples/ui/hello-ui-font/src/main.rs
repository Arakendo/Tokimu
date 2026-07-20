use std::{fs, path::PathBuf, sync::Arc};
use tokimu::{run_window_with_app, Camera, CameraHandle, ClearCommand, Color, DrawMeshCommand, FrameOutcome, Instance2d, Material, MaterialHandle, Mesh, MeshHandle, NativeWindow, Pipeline, PipelineHandle, PipelineKind, PlatformEventHandler, PlatformInputEvent, PlatformResult, RenderCommand, Renderer, Texture, TextureHandle, WgpuBackend, WindowConfig};
use ui_tools::UiFontRasterizer;

const QUAD: MeshHandle = MeshHandle(1);
const CAMERA: CameraHandle = CameraHandle(1);
const GLYPH_SIZE: f32 = 48.0;
const GLYPHS: &[char] = &[
    'A', 'B', 'C', 'D', 'E', 'F',
    'G', 'H', 'I', 'J', 'K', 'L',
    'M', 'N', 'O', 'P', 'Q', 'R',
    'S', 'T', 'U', 'V', 'W', 'X',
    'Y', 'Z', 'a', 'b', 'c', 'd',
    'e', 'f', 'g', 'h', 'i', 'j',
    'k', 'l', 'm', 'n', 'o', 'p',
    'q', 'r', 's', 't', 'u', 'v',
    'w', 'x', 'y', 'z', '0', '1',
    '2', '3', '4', '5', '6', '7',
    '8', '9',
];

fn main() -> PlatformResult<()> { run_window_with_app(WindowConfig { title: "Tokimu Hello UI Font | TTF / OTF Glyph Corpus".into(), width: 900, height: 600 }, App::default()) }

#[derive(Default)]
struct App { renderer: Option<WgpuBackend>, size: [f32; 2], pipeline: PipelineHandle, glyphs: Vec<GlyphDraw> }

struct GlyphDraw { material: MaterialHandle, center: [f32; 2], scale: [f32; 2] }

fn first_font(extension: &str) -> Option<Vec<u8>> {
    let provider = if extension == "otf" { "noto" } else { "inter" };
    let mut roots = vec![PathBuf::from(format!("target/glyph-corpus/fonts/{provider}"))];
    if let Ok(current_dir) = std::env::current_dir() {
            roots.extend(current_dir.ancestors().map(|path| path.join(format!("target/glyph-corpus/fonts/{provider}"))));
    }
    if let Ok(executable) = std::env::current_exe() {
        roots.extend(executable.ancestors().map(|path| path.join(format!("target/glyph-corpus/fonts/{provider}"))));
    }
    for root in roots {
        let mut pending = vec![root];
        while let Some(path) = pending.pop() {
            let Ok(entries) = fs::read_dir(path) else { continue; };
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_dir() { pending.push(path); }
                else if path.extension().is_some_and(|ext| ext == extension) {
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
        let ttf = first_font("ttf").ok_or_else(|| "run prepare-glyph-corpus.ps1 first (no TTF found)".to_string())?;
        let otf = first_font("otf").ok_or_else(|| "run prepare-glyph-corpus.ps1 first (no OTF found)".to_string())?;
        self.pipeline = renderer.register_pipeline(&Pipeline::new("font-texture", PipelineKind::Texture2d))?;
        let ttf = UiFontRasterizer::from_bytes(ttf).map_err(|error| error.to_string())?;
        let otf = UiFontRasterizer::from_bytes(otf).map_err(|error| error.to_string())?;
        fn rgba(glyph: &ui_tools::UiRasterGlyph) -> Vec<u8> { glyph.alpha.iter().flat_map(|alpha| [255, 255, 255, *alpha]).collect() }
        for (font_index, font) in [ttf, otf].into_iter().enumerate() {
            for (glyph_index, character) in GLYPHS.iter().copied().enumerate() {
                let glyph = font.rasterize(character, GLYPH_SIZE);
                let texture = TextureHandle(100 + (font_index * GLYPHS.len() + glyph_index) as u64);
                let material = MaterialHandle(100 + (font_index * GLYPHS.len() + glyph_index) as u64);
                if glyph.width == 0 || glyph.height == 0 { continue; }
                renderer.upload_texture(texture, &Texture::rgba8(glyph.width, glyph.height, rgba(&glyph)));
                let color = if font_index == 0 { Color::rgb(0.92, 0.94, 0.98) } else { Color::rgb(0.45, 0.68, 0.92) };
                renderer.upload_material(material, &Material::new("font-glyph", color).with_texture(texture))?;
                let column = glyph_index % 6;
                let row = glyph_index / 6;
                let x_origin = if font_index == 0 { -0.73 } else { 0.05 };
                let cell_left = x_origin + column as f32 * 0.115;
                let baseline_y = 0.75 - row as f32 * 0.115;
                let center = [
                    cell_left + (glyph.bearing_x + glyph.width as f32 * 0.5) * 2.0 / self.size[0],
                    baseline_y - (glyph.bearing_y + glyph.height as f32 * 0.5) * 2.0 / self.size[1],
                ];
                let scale = [glyph.width as f32 / self.size[0], glyph.height as f32 / self.size[1]];
                self.glyphs.push(GlyphDraw { material, center, scale });
            }
        }
        self.renderer = Some(renderer);
        Ok(())
    }
    fn on_platform_event(&mut self, event: PlatformInputEvent) -> PlatformResult<()> {
        if let PlatformInputEvent::Resized { width, height } = event { self.size = [width.max(1) as f32, height.max(1) as f32]; if let Some(renderer) = self.renderer.as_mut() { renderer.resize_surface(width, height); } }
        Ok(())
    }
    fn on_frame(&mut self, _delta_seconds: f64) -> PlatformResult<FrameOutcome> {
        let Some(renderer) = self.renderer.as_mut() else { return Ok(FrameOutcome::Continue); };
        renderer.upload_camera(CAMERA, Camera::orthographic_2d(self.size[0], self.size[1]));
        renderer.begin_frame();
        renderer.submit(&[
            RenderCommand::Clear(ClearCommand { color: Color::rgb(0.05, 0.06, 0.08) }),
            ]);
        let glyph_commands = self.glyphs.iter().map(|glyph| RenderCommand::DrawMesh(DrawMeshCommand {
            mesh: QUAD,
            material: glyph.material,
            pipeline: self.pipeline,
            instance: Instance2d::new(glyph.center, glyph.scale, 0.0),
            camera: Some(CAMERA),
            viewport: None,
        })).collect::<Vec<_>>();
        renderer.submit(&glyph_commands);
        let _ = renderer.present()?;
        Ok(FrameOutcome::Continue)
    }
}
