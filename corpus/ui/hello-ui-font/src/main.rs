use std::sync::Arc;
use tokimu::{
    run_window_with_app, Camera, CameraHandle, ClearCommand, Color, DrawMeshCommand, FrameOutcome,
    Instance2d, Material, MaterialHandle, Mesh, MeshHandle, NativeWindow, Pipeline, PipelineHandle,
    PipelineKind, PlatformEventHandler, PlatformInputEvent, PlatformResult, RenderCommand,
    Renderer, Texture, TextureHandle, WgpuBackend, WindowConfig,
};
use ui_tools::{alpha_to_rgba8, UiFontFormat, UiFontRasterizer, UiFontSource, TEXT_CORPUS};

const QUAD: MeshHandle = MeshHandle(1);
const CAMERA: CameraHandle = CameraHandle(1);
const GLYPH_SIZE: f32 = 48.0;
fn main() -> PlatformResult<()> {
    run_window_with_app(
        WindowConfig {
            title: "Tokimu Hello UI Font | TTF / OTF Glyph Corpus".into(),
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
    glyphs: Vec<GlyphDraw>,
}

struct GlyphDraw {
    material: MaterialHandle,
    center: [f32; 2],
    scale: [f32; 2],
}

impl PlatformEventHandler for App {
    fn on_native_window_created(&mut self, window: Arc<NativeWindow>) -> PlatformResult<()> {
        let size = window.inner_size();
        self.size = [size.width.max(1) as f32, size.height.max(1) as f32];
        let mut renderer = WgpuBackend::for_window(window, size.width, size.height)?;
        renderer.upload_mesh(QUAD, &Mesh::quad());
        let ttf = UiFontSource::from_prepared_corpus("inter", UiFontFormat::Ttf)
            .map_err(|error| error.to_string())?;
        let otf = UiFontSource::from_prepared_corpus("noto", UiFontFormat::Otf)
            .map_err(|error| error.to_string())?;
        self.pipeline =
            renderer.register_pipeline(&Pipeline::new("font-texture", PipelineKind::Texture2d))?;
        let ttf = UiFontRasterizer::from_bytes(ttf.bytes).map_err(|error| error.to_string())?;
        let otf = UiFontRasterizer::from_bytes(otf.bytes).map_err(|error| error.to_string())?;
        let glyphs = TEXT_CORPUS
            .iter()
            .find(|sample| sample.id == "uppercase")
            .map(|sample| sample.text)
            .unwrap_or("")
            .chars()
            .chain(
                TEXT_CORPUS
                    .iter()
                    .find(|sample| sample.id == "lowercase")
                    .map(|sample| sample.text)
                    .unwrap_or("")
                    .chars(),
            )
            .chain(
                TEXT_CORPUS
                    .iter()
                    .find(|sample| sample.id == "digits")
                    .map(|sample| sample.text)
                    .unwrap_or("")
                    .chars(),
            )
            .collect::<Vec<_>>();
        for (font_index, font) in [ttf, otf].into_iter().enumerate() {
            for (glyph_index, character) in glyphs.iter().copied().enumerate() {
                let glyph = font.rasterize(character, GLYPH_SIZE);
                let texture = TextureHandle(100 + (font_index * glyphs.len() + glyph_index) as u64);
                let material =
                    MaterialHandle(100 + (font_index * glyphs.len() + glyph_index) as u64);
                if glyph.width == 0 || glyph.height == 0 {
                    continue;
                }
                renderer.upload_texture(
                    texture,
                    &Texture::rgba8(
                        glyph.width,
                        glyph.height,
                        alpha_to_rgba8(&glyph.alpha, [255, 255, 255]),
                    ),
                );
                let color = if font_index == 0 {
                    Color::rgb(0.92, 0.94, 0.98)
                } else {
                    Color::rgb(0.45, 0.68, 0.92)
                };
                renderer.upload_material(
                    material,
                    &Material::new("font-glyph", color).with_texture(texture),
                )?;
                let column = glyph_index % 6;
                let row = glyph_index / 6;
                let x_origin = if font_index == 0 { -0.73 } else { 0.05 };
                let cell_left = x_origin + column as f32 * 0.115;
                let baseline_y = 0.75 - row as f32 * 0.115;
                let center = [
                    cell_left + (glyph.bearing_x + glyph.width as f32 * 0.5) * 2.0 / self.size[0],
                    baseline_y - (glyph.bearing_y + glyph.height as f32 * 0.5) * 2.0 / self.size[1],
                ];
                let scale = [
                    glyph.width as f32 / self.size[0],
                    glyph.height as f32 / self.size[1],
                ];
                self.glyphs.push(GlyphDraw {
                    material,
                    center,
                    scale,
                });
            }
        }
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
        let glyph_commands = self
            .glyphs
            .iter()
            .map(|glyph| {
                RenderCommand::DrawMesh(DrawMeshCommand {
                    mesh: QUAD,
                    material: glyph.material,
                    pipeline: self.pipeline,
                    instance: Instance2d::new(glyph.center, glyph.scale, 0.0),
                    camera: Some(CAMERA),
                    viewport: None,
                })
            })
            .collect::<Vec<_>>();
        renderer.submit(&glyph_commands);
        let _ = renderer.present()?;
        Ok(FrameOutcome::Continue)
    }
}
