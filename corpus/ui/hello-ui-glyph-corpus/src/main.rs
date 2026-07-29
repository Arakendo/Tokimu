use std::{
    fs,
    path::{Path, PathBuf},
    sync::Arc,
};

use tokimu::{
    run_window_with_app, Camera, CameraHandle, ClearCommand, Color, DrawMeshCommand, FrameOutcome,
    Instance2d, Material, MaterialHandle, Mesh, MeshHandle, NativeWindow, Pipeline, PipelineHandle,
    PipelineKind, PlatformEventHandler, PlatformInputEvent, PlatformResult, RenderCommand,
    Renderer, Texture, TextureHandle, WgpuBackend, WindowConfig,
};
use ui_tools::{
    alpha_to_rgba8, layout_bitmap_text, text_corpus_samples, UiFontFormat, UiFontRasterizer,
    UiFontSource, UiRect, UiTextAlign, UiTextCorpusGroup, UiTextRole, UiTextSpec,
    TEXT_CORPUS_VERSION,
};

const GLYPH_MESH: MeshHandle = MeshHandle(1);
const CAMERA: CameraHandle = CameraHandle(1);
const BACKDROP: MaterialHandle = MaterialHandle(1);
const TITLE: MaterialHandle = MaterialHandle(2);
const BODY: MaterialHandle = MaterialHandle(3);
const MUTED: MaterialHandle = MaterialHandle(4);
const ACCENT: MaterialHandle = MaterialHandle(5);

const PAGES: [&str; 4] = ["FONT SAMPLES", "ICON BATCH", "UNICODE", "METRIC TORTURE"];
fn unicode_rows() -> Vec<String> {
    text_corpus_samples(UiTextCorpusGroup::Coverage)
        .chain(text_corpus_samples(UiTextCorpusGroup::CoverageAndFallback))
        .map(|sample| sample.text.to_owned())
        .collect()
}

fn metric_rows() -> Vec<String> {
    text_corpus_samples(UiTextCorpusGroup::MetricTorture)
        .map(|sample| sample.text.to_owned())
        .collect()
}

fn corpus_files(root: &Path, extensions: &[&str], limit: usize) -> Vec<String> {
    fn visit(
        path: &Path,
        extensions: &[&str],
        limit: usize,
        result: &mut Vec<String>,
        root: &Path,
    ) {
        if result.len() >= limit {
            return;
        }
        let Ok(entries) = fs::read_dir(path) else {
            return;
        };
        for entry in entries.flatten() {
            if result.len() >= limit {
                return;
            }
            let path = entry.path();
            if path.is_dir() {
                visit(&path, extensions, limit, result, root);
            } else if path
                .extension()
                .and_then(|value| value.to_str())
                .is_some_and(|extension| extensions.contains(&extension))
            {
                if let Ok(relative) = path.strip_prefix(root) {
                    result.push(relative.to_string_lossy().replace('\\', "/"));
                }
            }
        }
    }
    let mut result = Vec::new();
    visit(root, extensions, limit, &mut result, root);
    result.sort();
    result
}

fn prepared_corpus() -> (Vec<String>, Vec<String>) {
    let mut candidates = vec![PathBuf::from("target/glyph-corpus")];
    if let Ok(current_dir) = std::env::current_dir() {
        for ancestor in current_dir.ancestors() {
            candidates.push(ancestor.join("target/glyph-corpus"));
        }
    }
    if let Ok(executable) = std::env::current_exe() {
        for ancestor in executable.ancestors() {
            candidates.push(ancestor.join("target/glyph-corpus"));
        }
    }
    let root = candidates
        .into_iter()
        .find(|path| path.join("manifest.json").is_file())
        .unwrap_or_else(|| PathBuf::from("target/glyph-corpus"));
    (
        corpus_files(&root.join("fonts"), &["ttf", "otf", "woff2"], 8),
        corpus_files(&root.join("icons"), &["svg"], 8),
    )
}

fn main() -> PlatformResult<()> {
    run_window_with_app(
        WindowConfig {
            title: "Tokimu Hello UI Glyph Corpus".into(),
            width: 1260,
            height: 760,
        },
        HelloUiGlyphCorpus::default(),
    )
}

struct HelloUiGlyphCorpus {
    renderer: Option<WgpuBackend>,
    window_size: [f32; 2],
    pipeline: PipelineHandle,
    font_pipeline: PipelineHandle,
    page: usize,
    row: usize,
    font_rows: Vec<String>,
    icon_rows: Vec<String>,
    font_draws: Vec<FontDraw>,
}

struct FontDraw {
    material: MaterialHandle,
    center: [f32; 2],
    scale: [f32; 2],
}

impl Default for HelloUiGlyphCorpus {
    fn default() -> Self {
        let (font_rows, icon_rows) = prepared_corpus();
        Self {
            renderer: None,
            window_size: [1.0, 1.0],
            pipeline: PipelineHandle(0),
            font_pipeline: PipelineHandle(0),
            page: 0,
            row: 0,
            font_rows,
            icon_rows,
            font_draws: Vec::new(),
        }
    }
}

impl HelloUiGlyphCorpus {
    fn rows(&self) -> Vec<String> {
        match self.page {
            0 => self.font_rows.iter().take(8).cloned().collect(),
            1 => self.icon_rows.iter().take(8).cloned().collect(),
            2 => unicode_rows(),
            _ => metric_rows(),
        }
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
}

impl PlatformEventHandler for HelloUiGlyphCorpus {
    fn on_native_window_created(&mut self, window: Arc<NativeWindow>) -> PlatformResult<()> {
        let size = window.inner_size();
        self.window_size = [size.width.max(1) as f32, size.height.max(1) as f32];
        let mut renderer = WgpuBackend::for_window(window, size.width, size.height)?;
        renderer.upload_mesh(GLYPH_MESH, &Mesh::quad());
        renderer.upload_material(
            BACKDROP,
            &Material::new("glyph-backdrop", Color::rgb(0.05, 0.06, 0.08)),
        )?;
        renderer.upload_material(
            TITLE,
            &Material::new("glyph-title", Color::rgb(0.92, 0.94, 0.98)),
        )?;
        renderer.upload_material(
            BODY,
            &Material::new("glyph-body", Color::rgb(0.74, 0.79, 0.87)),
        )?;
        renderer.upload_material(
            MUTED,
            &Material::new("glyph-muted", Color::rgb(0.48, 0.54, 0.63)),
        )?;
        renderer.upload_material(
            ACCENT,
            &Material::new("glyph-accent", Color::rgb(0.45, 0.68, 0.92)),
        )?;
        self.pipeline = renderer.register_pipeline(&Pipeline::new(
            "hello-ui-glyph-corpus",
            PipelineKind::SolidColor2d,
        ))?;
        self.font_pipeline = renderer.register_pipeline(&Pipeline::new(
            "hello-ui-glyph-corpus-font",
            PipelineKind::Texture2d,
        ))?;
        let source = UiFontSource::from_prepared_corpus("inter", UiFontFormat::Ttf)
            .map_err(|error| error.to_string())?;
        let font = UiFontRasterizer::from_bytes(source.bytes).map_err(|error| error.to_string())?;
        for (index, row) in self.font_rows.iter().take(8).enumerate() {
            let bitmap = font.rasterize_text(row, 24.0);
            if bitmap.width == 0 || bitmap.height == 0 {
                continue;
            }
            let texture = TextureHandle(100 + index as u64);
            let material = MaterialHandle(100 + index as u64);
            renderer.upload_texture(
                texture,
                &Texture::rgba8(
                    bitmap.width,
                    bitmap.height,
                    alpha_to_rgba8(&bitmap.alpha, [220, 228, 240]),
                ),
            );
            renderer.upload_material(
                material,
                &Material::new("glyph-font", Color::rgb(0.86, 0.90, 0.96)).with_texture(texture),
            )?;
            let baseline_y = 0.16 - index as f32 * 0.075;
            let center = [
                (bitmap.left + bitmap.width as f32 * 0.5) * 2.0 / self.window_size[0],
                baseline_y
                    - (bitmap.baseline + bitmap.top + bitmap.height as f32 * 0.5) * 2.0
                        / self.window_size[1],
            ];
            self.font_draws.push(FontDraw {
                material,
                center,
                scale: [
                    bitmap.width as f32 * 2.0 / self.window_size[0],
                    bitmap.height as f32 * 2.0 / self.window_size[1],
                ],
            });
        }
        self.renderer = Some(renderer);
        Ok(())
    }

    fn on_platform_event(&mut self, event: PlatformInputEvent) -> PlatformResult<()> {
        match event {
            PlatformInputEvent::KeyboardInput { key, pressed: true } => match key {
                tokimu::KeyCode::ArrowLeft => {
                    self.page = (self.page + PAGES.len() - 1) % PAGES.len();
                    self.row = 0;
                }
                tokimu::KeyCode::ArrowRight => {
                    self.page = (self.page + 1) % PAGES.len();
                    self.row = 0;
                }
                tokimu::KeyCode::ArrowUp => self.row = self.row.saturating_sub(1),
                tokimu::KeyCode::ArrowDown => {
                    self.row = (self.row + 1).min(self.rows().len().saturating_sub(1))
                }
                _ => {}
            },
            PlatformInputEvent::Resized { width, height } => {
                self.window_size = [width.max(1) as f32, height.max(1) as f32];
                if let Some(renderer) = self.renderer.as_mut() {
                    renderer.resize_surface(width, height);
                }
            }
            _ => {}
        }
        Ok(())
    }

    fn on_frame(&mut self, _delta_seconds: f64) -> PlatformResult<FrameOutcome> {
        let page = self.page;
        let selected_row = self.row;
        let rows = self.rows();
        let pipeline = self.pipeline;
        let Some(renderer) = self.renderer.as_mut() else {
            return Ok(FrameOutcome::Continue);
        };
        renderer.upload_camera(
            CAMERA,
            Camera::orthographic_2d(self.window_size[0], self.window_size[1]),
        );
        renderer.begin_frame();
        renderer.submit(&[RenderCommand::Clear(ClearCommand {
            color: Color::rgb(0.05, 0.06, 0.08),
        })]);

        if page == 0 {
            let commands = self
                .font_draws
                .iter()
                .map(|draw| {
                    RenderCommand::DrawMesh(DrawMeshCommand {
                        mesh: GLYPH_MESH,
                        material: draw.material,
                        pipeline: self.font_pipeline,
                        instance: Instance2d::new(draw.center, draw.scale, 0.0),
                        camera: Some(CAMERA),
                        viewport: None,
                    })
                })
                .collect::<Vec<_>>();
            renderer.submit(&commands);
        }

        let title = UiTextSpec::new(
            format!("GLYPH CORPUS {TEXT_CORPUS_VERSION}"),
            UiRect::new([0.0, 0.38], [1.0, 0.09]),
            UiTextRole::Title,
        )
        .with_alignment(UiTextAlign::Center, UiTextAlign::Center);
        let page_spec = UiTextSpec::new(
            PAGES[page],
            UiRect::new([0.0, 0.28], [1.0, 0.07]),
            UiTextRole::Heading,
        )
        .with_alignment(UiTextAlign::Center, UiTextAlign::Center);
        let hint = UiTextSpec::new(
            "LEFT RIGHT PAGE / UP DOWN ROW",
            UiRect::new([0.0, -0.40], [1.0, 0.06]),
            UiTextRole::Caption,
        )
        .with_alignment(UiTextAlign::Center, UiTextAlign::Center);
        let corpus_status = if self.font_rows.is_empty() || self.icon_rows.is_empty() {
            "CORPUS MISSING / RUN PREP SCRIPT"
        } else {
            "CORPUS READY / FONT AND SVG DATA LOADED"
        };
        let status = UiTextSpec::new(
            corpus_status,
            UiRect::new([0.0, -0.32], [1.0, 0.06]),
            UiTextRole::Status,
        )
        .with_alignment(UiTextAlign::Center, UiTextAlign::Center);
        let mut specs = vec![
            (title, 0.07, TITLE),
            (page_spec, 0.045, ACCENT),
            (status, 0.03, MUTED),
            (hint, 0.03, MUTED),
        ];
        if page != 0 {
            for (index, row) in rows.iter().enumerate() {
                let y = 0.16 - index as f32 * 0.075;
                let spec = UiTextSpec::new(
                    row.as_str(),
                    UiRect::new([0.0, y], [1.05, 0.06]),
                    UiTextRole::Body,
                )
                .with_alignment(UiTextAlign::Center, UiTextAlign::Center);
                specs.push((
                    spec,
                    0.035,
                    if index == selected_row { ACCENT } else { BODY },
                ));
            }
        }
        for (spec, height, material) in specs {
            renderer.submit(&Self::text_commands(&spec, height, material, pipeline));
        }
        let _ = renderer.present()?;
        Ok(FrameOutcome::Continue)
    }
}
