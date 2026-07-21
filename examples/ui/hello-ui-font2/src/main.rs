use std::sync::Arc;
use tokimu::{
    run_window_with_app, Camera, CameraHandle, ClearCommand, Color, DrawMeshCommand, FrameOutcome,
    Instance2d, Material, MaterialHandle, Mesh, MeshHandle, NativeWindow, Pipeline, PipelineHandle,
    PipelineKind, PlatformEventHandler, PlatformResult, RenderCommand, Renderer, Texture,
    TextureHandle, WgpuBackend, WindowConfig,
};
use ui_tools::{alpha_to_rgba8, UiFontFormat, UiFontRasterizer, UiFontSource, TEXT_CORPUS};

const QUAD: MeshHandle = MeshHandle(1);
const CAMERA: CameraHandle = CameraHandle(1);
const FONT_SIZE: f32 = 56.0;
fn corpus_text(id: &str) -> &'static str {
    TEXT_CORPUS
        .iter()
        .find(|sample| sample.id == id)
        .unwrap_or_else(|| panic!("missing text corpus sample: {id}"))
        .text
}

fn main() -> PlatformResult<()> {
    run_window_with_app(WindowConfig { title: "Tokimu Hello UI Font 2 | Inter-Regular.ttf (TTF) | JetBrainsMono-Regular.otf (OTF) | NotoSans-VF.ttf (TTF)".into(), width: 1440, height: 760 }, App::default())
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
        self.pipeline =
            renderer.register_pipeline(&Pipeline::new("font2-texture", PipelineKind::Texture2d))?;
        let mut material_id = 10_u64;
        let fonts = [
            (
                "inter",
                UiFontFormat::Ttf,
                "INTER / TTF",
                [0.92, 0.94, 0.98],
            ),
            (
                "jetbrains-mono",
                UiFontFormat::Otf,
                "JETBRAINS MONO / OTF",
                [0.45, 0.68, 0.92],
            ),
            (
                "noto",
                UiFontFormat::Ttf,
                "NOTO SANS / TTF",
                [0.78, 0.86, 0.96],
            ),
        ];
        let lines = [
            corpus_text("sphinx"),
            corpus_text("body"),
            corpus_text("liquor-jugs"),
        ];
        for (font_index, (provider, format, label, color)) in fonts.into_iter().enumerate() {
            let source = UiFontSource::from_prepared_corpus(provider, format)
                .map_err(|error| error.to_string())?;
            let font =
                UiFontRasterizer::from_bytes(source.bytes).map_err(|error| error.to_string())?;
            let heading = font.rasterize_text(label, 30.0);
            if heading.width > 0 && heading.height > 0 {
                let texture = TextureHandle(material_id);
                let material = MaterialHandle(material_id);
                renderer.upload_texture(
                    texture,
                    &Texture::rgba8(
                        heading.width,
                        heading.height,
                        alpha_to_rgba8(&heading.alpha, [220, 228, 240]),
                    ),
                );
                renderer.upload_material(
                    material,
                    &Material::new("font2-heading", Color::rgb(color[0], color[1], color[2]))
                        .with_texture(texture),
                )?;
                let baseline_y = 0.91 - font_index as f32 * 0.64;
                self.glyphs.push(GlyphDraw {
                    material,
                    center: [
                        (heading.left + heading.width as f32 * 0.5) * 2.0 / self.size[0],
                        baseline_y
                            - (heading.baseline + heading.top + heading.height as f32 * 0.5) * 2.0
                                / self.size[1],
                    ],
                    scale: [
                        heading.width as f32 * 1.32 / self.size[0],
                        heading.height as f32 / self.size[1],
                    ],
                });
                material_id += 1;
            }
            for (line_index, line) in lines.iter().enumerate() {
                let bitmap = font.rasterize_text(line, FONT_SIZE);
                let baseline_y = 0.76 - font_index as f32 * 0.64 - line_index as f32 * 0.17;
                if bitmap.width == 0 || bitmap.height == 0 {
                    continue;
                }
                let texture = TextureHandle(material_id);
                let material = MaterialHandle(material_id);
                let rgba = alpha_to_rgba8(&bitmap.alpha, [0xff, 0xff, 0xff]);
                renderer
                    .upload_texture(texture, &Texture::rgba8(bitmap.width, bitmap.height, rgba));
                renderer.upload_material(
                    material,
                    &Material::new("font2-line", Color::rgb(color[0], color[1], color[2]))
                        .with_texture(texture),
                )?;
                let center = [
                    0.0,
                    baseline_y
                        - (bitmap.baseline + bitmap.top + bitmap.height as f32 * 0.5) * 2.0
                            / self.size[1],
                ];
                self.glyphs.push(GlyphDraw {
                    material,
                    center,
                    scale: [
                        bitmap.width as f32 * 1.32 / self.size[0],
                        bitmap.height as f32 / self.size[1],
                    ],
                });
                material_id += 1;
            }
        }
        self.renderer = Some(renderer);
        Ok(())
    }

    fn on_platform_event(&mut self, event: tokimu::PlatformInputEvent) -> PlatformResult<()> {
        if let tokimu::PlatformInputEvent::Resized { width, height } = event {
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
        renderer.submit(&commands);
        let _ = renderer.present()?;
        Ok(FrameOutcome::Continue)
    }
}
