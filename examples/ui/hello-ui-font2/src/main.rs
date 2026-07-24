use screenshot::write_manifest;
use std::{collections::HashMap, env, fs, sync::Arc, time::Instant};
use tokimu::{
    run_window_with_app, Camera, CameraHandle, ClearCommand, Color, DrawMeshCommand, FrameOutcome,
    Instance2d, Material, MaterialHandle, Mesh, MeshHandle, NativeWindow, Pipeline, PipelineHandle,
    PipelineKind, PlatformEventHandler, PlatformResult, RenderCommand, Renderer, Texture,
    TextureHandle, WgpuBackend, WindowConfig,
};
use ui_tools::{alpha_to_rgba8, UiFontFormat, UiFontRasterizer, UiFontSource, TEXT_CORPUS};
use ui_tools::{tessellate_general_fill, UiGlyphOutlineSegment, UiGlyphVectorOptions};

const QUAD: MeshHandle = MeshHandle(1);
const CAMERA: CameraHandle = CameraHandle(1);
const FONT_SIZE: f32 = 56.0;
const RASTER_COLUMN_X: f32 = -0.78;
const VECTOR_COLUMN_X: f32 = 0.78;
fn corpus_text(id: &str) -> &'static str {
    TEXT_CORPUS
        .iter()
        .find(|sample| sample.id == id)
        .unwrap_or_else(|| panic!("missing text corpus sample: {id}"))
        .text
}

fn report_vector_sizes(font: &UiFontRasterizer, sample_id: &str, sample: &str) {
    for pixels in [24.0_f32, 56.0, 96.0] {
        let mut points = 0usize;
        let mut triangles = 0usize;
        let mut lines = 0usize;
        let mut quadratics = 0usize;
        let mut cubics = 0usize;
        for character in sample
            .chars()
            .filter(|character| !character.is_whitespace())
        {
            let Ok(outline) = font.outline(character) else {
                continue;
            };
            for contour in &outline.contours {
                for segment in &contour.segments {
                    match segment {
                        UiGlyphOutlineSegment::LineTo(_) => lines += 1,
                        UiGlyphOutlineSegment::QuadTo { .. } => quadratics += 1,
                        UiGlyphOutlineSegment::CubicTo { .. } => cubics += 1,
                    }
                }
            }
            let Ok(path) =
                outline.to_vector_path(UiGlyphVectorOptions::new(pixels, [0.0, 0.0], 0.35))
            else {
                continue;
            };
            points += path
                .contours
                .iter()
                .map(|contour| contour.points.len())
                .sum::<usize>();
            if let Ok(fill) = tessellate_general_fill(&path) {
                triangles += fill.len() / 3;
            }
        }
        println!(
            "hello-ui-font2 size sample: id={sample_id}, pixels={pixels:.0}, points={points}, triangles={triangles}, native_segments=line:{lines},quad:{quadratics},cubic:{cubics}"
        );
    }
}

fn main() -> PlatformResult<()> {
    run_window_with_app(WindowConfig { title: "Tokimu Hello UI Font 2 | Inter-Regular.ttf (TTF) | JetBrainsMono-Regular.otf (OTF) | NotoSans-VF.ttf (TTF)".into(), width: 1440, height: 760 }, App::default())
}

#[derive(Default)]
struct App {
    renderer: Option<WgpuBackend>,
    size: [f32; 2],
    pipeline: PipelineHandle,
    vector_pipeline: PipelineHandle,
    glyphs: Vec<GlyphDraw>,
    vector_glyphs: Vec<VectorGlyphDraw>,
    reported_frames: u32,
}

struct GlyphDraw {
    material: MaterialHandle,
    center: [f32; 2],
    scale: [f32; 2],
}

struct VectorGlyphDraw {
    mesh: MeshHandle,
    material: MaterialHandle,
    center: [f32; 2],
}

impl PlatformEventHandler for App {
    fn on_native_window_created(&mut self, window: Arc<NativeWindow>) -> PlatformResult<()> {
        let size = window.inner_size();
        self.size = [size.width.max(1) as f32, size.height.max(1) as f32];
        let mut renderer = WgpuBackend::for_window(window, size.width, size.height)?;
        renderer.upload_mesh(QUAD, &Mesh::quad());
        self.pipeline =
            renderer.register_pipeline(&Pipeline::new("font2-texture", PipelineKind::Texture2d))?;
        let vector_pipeline = renderer
            .register_pipeline(&Pipeline::new("font2-vector", PipelineKind::SolidColor2d))?;
        self.vector_pipeline = vector_pipeline;
        let vector_build_start = Instant::now();
        let mut vector_triangle_count = 0usize;
        let mut vector_mesh_cache = HashMap::<(usize, char), MeshHandle>::new();
        // The orthographic camera's world height is two units, so this is the
        // example's single pixel-to-world convention for both text paths.
        let pixel_scale = 1.0 / self.size[1];
        println!(
            "hello-ui-font2 layout: viewport={}x{}, pixel_scale={:.6}, raster_x={:.2}, vector_x={:.2}",
            self.size[0],
            self.size[1],
            pixel_scale,
            RASTER_COLUMN_X,
            VECTOR_COLUMN_X
        );
        let artifact_dir = std::path::PathBuf::from("target/hello-ui-font2");
        fs::create_dir_all(&artifact_dir).map_err(|error| error.to_string())?;
        let viewport = format!("{}x{}", self.size[0], self.size[1]);
        let scale = format!("{pixel_scale:.6}");
        let raster_anchor = format!("{RASTER_COLUMN_X:.2}");
        let vector_anchor = format!("{VECTOR_COLUMN_X:.2}");
        let source_revision =
            env::var("TOKIMU_CORPUS_REVISION").unwrap_or_else(|_| "unknown".to_owned());
        let fixture_set = "inter:ttf,jetbrains-mono:otf,noto:ttf";
        write_manifest(
            artifact_dir.join("comparison.txt"),
            &[
                ("example", "hello-ui-font2"),
                ("viewport", &viewport),
                ("pixel_scale", &scale),
                ("raster_column_x", &raster_anchor),
                ("vector_column_x", &vector_anchor),
                ("source_revision", &source_revision),
                ("fixture_set", fixture_set),
                ("raster_strategy", "font bitmap texture"),
                ("vector_strategy", "glyph outline tessellation"),
                ("gpu_readback", "false"),
            ],
        )
        .map_err(|error| error.to_string())?;
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
            println!(
                "hello-ui-font2 fixture: provider={provider}, format={}, path={}, bytes={}",
                format.extension(),
                source.path.display(),
                source.bytes.len()
            );
            let font =
                UiFontRasterizer::from_bytes(source.bytes).map_err(|error| error.to_string())?;
            for sample_id in [
                "sphinx",
                "capital-i",
                "capital-w",
                "zeros",
                "punctuation",
                "av-pair",
            ] {
                report_vector_sizes(&font, sample_id, corpus_text(sample_id));
            }
            let vector_material = MaterialHandle(5000 + font_index as u64);
            renderer.upload_material(
                vector_material,
                &Material::new(
                    "font2-vector-glyph",
                    Color::rgb(color[0], color[1], color[2]),
                ),
            )?;
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
                        RASTER_COLUMN_X
                            + (heading.left + heading.width as f32 * 0.5) * pixel_scale * 2.0,
                        baseline_y
                            - (heading.baseline + heading.top + heading.height as f32 * 0.5) * 2.0
                                / self.size[1],
                    ],
                    scale: [
                        heading.width as f32 * pixel_scale,
                        heading.height as f32 * pixel_scale,
                    ],
                });
                material_id += 1;

                let heading_layout = font.layout(label, 30.0);
                // Match the raster quad's camera convention: one world unit
                // spans half the viewport height in this orthographic scene.
                let heading_scale = pixel_scale;
                let heading_origin = [
                    VECTOR_COLUMN_X - heading_layout.width * heading_scale * 0.5,
                    baseline_y - 0.04,
                ];
                for positioned in &heading_layout.glyphs {
                    if positioned.glyph.character.is_whitespace() {
                        continue;
                    }
                    let mut local = positioned.clone();
                    local.pen_x = 0.0;
                    let triangles = match font.tessellate_positioned_glyph(
                        &local,
                        30.0,
                        heading_scale,
                        [0.0, 0.0],
                        heading_scale * 0.35,
                    ) {
                        Ok(triangles) => triangles,
                        Err(error) => {
                            println!(
                                "hello-ui-font2 vector diagnostic: provider={provider}, label={label}, character={:?}, kind={:?}, message={}",
                                positioned.glyph.character,
                                error.kind, error.message
                            );
                            continue;
                        }
                    };
                    if triangles.is_empty() {
                        continue;
                    }
                    vector_triangle_count += triangles.len() / 3;
                    let mesh = MeshHandle(7000 + self.vector_glyphs.len() as u64);
                    let positions = triangles.into_iter().map(|[x, y]| [x, y, 0.0]).collect();
                    renderer.upload_mesh(mesh, &Mesh::uniform_normal(positions, [0.0, 0.0, 1.0]));
                    self.vector_glyphs.push(VectorGlyphDraw {
                        mesh,
                        material: vector_material,
                        center: [
                            heading_origin[0] + positioned.pen_x * heading_scale,
                            heading_origin[1],
                        ],
                    });
                }
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
                    RASTER_COLUMN_X,
                    baseline_y
                        - (bitmap.baseline + bitmap.top + bitmap.height as f32 * 0.5) * 2.0
                            / self.size[1],
                ];
                self.glyphs.push(GlyphDraw {
                    material,
                    center,
                    scale: [
                        bitmap.width as f32 * pixel_scale,
                        bitmap.height as f32 * pixel_scale,
                    ],
                });
                material_id += 1;

                // The vector path consumes the same provider layout and line
                // baseline. It is intentionally separate from texture draws
                // so the comparison cannot hide a metrics disagreement.
                let layout = font.layout(line, FONT_SIZE);
                println!(
                    "hello-ui-font2 parity: provider={provider}, line={line_index}, advance_width_px={:.2}, raster_ink_width_px={}",
                    layout.width, bitmap.width
                );
                // Keep vector glyphs in the same presentation scale as the
                // raster texture path above.
                let output_scale = pixel_scale;
                let vector_origin_x = VECTOR_COLUMN_X - layout.width * output_scale * 0.5;
                let vector_origin_y = baseline_y - 0.08;
                for positioned in &layout.glyphs {
                    if positioned.glyph.character.is_whitespace() {
                        continue;
                    }
                    let character = positioned.glyph.character;
                    let mesh = if let Some(mesh) = vector_mesh_cache.get(&(font_index, character)) {
                        *mesh
                    } else {
                        // Build the glyph at a local baseline origin. Placement
                        // remains an instance concern, so repeated letters can
                        // share one uploaded mesh.
                        let mut local = positioned.clone();
                        local.pen_x = 0.0;
                        let triangles = match font.tessellate_positioned_glyph(
                            &local,
                            FONT_SIZE,
                            output_scale,
                            [0.0, 0.0],
                            output_scale * 0.35,
                        ) {
                            Ok(triangles) => triangles,
                            Err(error) => {
                                println!(
                                "hello-ui-font2 vector diagnostic: provider={provider}, line={line_index}, character={character:?}, kind={:?}, message={}",
                                error.kind, error.message
                            );
                                continue;
                            }
                        };
                        if triangles.is_empty() {
                            continue;
                        }
                        vector_triangle_count += triangles.len() / 3;
                        let mesh = MeshHandle(6000 + vector_mesh_cache.len() as u64);
                        let positions = triangles.into_iter().map(|[x, y]| [x, y, 0.0]).collect();
                        renderer
                            .upload_mesh(mesh, &Mesh::uniform_normal(positions, [0.0, 0.0, 1.0]));
                        vector_mesh_cache.insert((font_index, character), mesh);
                        mesh
                    };
                    self.vector_glyphs.push(VectorGlyphDraw {
                        mesh,
                        material: vector_material,
                        center: [
                            vector_origin_x + positioned.pen_x * output_scale,
                            vector_origin_y,
                        ],
                    });
                }
            }
        }
        println!(
            "hello-ui-font2 vector proof: glyph_instances={}, cached_meshes={}, triangles={}, build_ms={:.3}",
            self.vector_glyphs.len(),
            vector_mesh_cache.len(),
            vector_triangle_count,
            vector_build_start.elapsed().as_secs_f64() * 1000.0
        );
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
        let vector_commands = self
            .vector_glyphs
            .iter()
            .map(|glyph| {
                RenderCommand::DrawMesh(DrawMeshCommand {
                    mesh: glyph.mesh,
                    material: glyph.material,
                    pipeline: self.vector_pipeline,
                    instance: Instance2d::new(glyph.center, [1.0, 1.0], 0.0),
                    camera: Some(CAMERA),
                    viewport: None,
                })
            })
            .collect::<Vec<_>>();
        renderer.submit(&vector_commands);
        let stats = renderer.present()?;
        if self.reported_frames < 2 {
            println!(
                "hello-ui-font2 render stats: frame={}, draw_calls={}, mesh_uploads={}, mesh_replacements={}",
                self.reported_frames + 1,
                stats.draw_calls,
                stats.mesh_uploads,
                stats.mesh_replacements
            );
            self.reported_frames += 1;
        }
        Ok(FrameOutcome::Continue)
    }
}
