use std::{
    fs,
    path::{Path, PathBuf},
    sync::Arc,
};

use raster_image_corpus::{
    decode_bmp, decode_jpeg, decode_png, prepare_renderer_texture, DecodeLimits, DecodedImage,
    TextureUploadArtifact, TextureUse,
};
use serde::Serialize;
use tokimu::{
    run_window_with_app, BlendMode, Camera, CameraHandle, ClearCommand, Color, DrawMeshCommand,
    FrameOutcome, Instance2d, KeyCode, Material, MaterialHandle, Mesh, MeshHandle, NativeWindow,
    Pipeline, PipelineHandle, PipelineKind, PlatformEventHandler, PlatformInputEvent,
    PlatformResult, RenderCommand, Renderer, Rgba8TextureColorSpace, Rgba8TextureDescriptor,
    TextureHandle, WgpuBackend, WindowConfig,
};

const QUAD: MeshHandle = MeshHandle(1);
const CAMERA: CameraHandle = CameraHandle(1);
const BACKGROUND: Color = Color::rgb(0.045, 0.055, 0.075);

#[derive(Clone, Copy)]
enum SourceFormat {
    Png,
    Jpeg,
    Bmp,
}

impl SourceFormat {
    const fn label(self) -> &'static str {
        match self {
            Self::Png => "png",
            Self::Jpeg => "jpeg",
            Self::Bmp => "bmp",
        }
    }
}

struct FixtureSpec {
    label: &'static str,
    relative_path: &'static str,
    format: SourceFormat,
}

const FIXTURES: [FixtureSpec; 5] = [
    FixtureSpec {
        label: "PngSuite palette transparency",
        relative_path:
            "third-party/fixtures/w3c-svg-1.1-2nd-edition/upstream/images/PngSuite/tp1n3p08.png",
        format: SourceFormat::Png,
    },
    FixtureSpec {
        label: "libjpeg-turbo baseline YCbCr",
        relative_path:
            "third-party/fixtures/raster-images/upstream/libjpeg-turbo/testimages/testorig.jpg",
        format: SourceFormat::Jpeg,
    },
    FixtureSpec {
        label: "libjpeg-turbo baseline alternate",
        relative_path:
            "third-party/fixtures/raster-images/upstream/libjpeg-turbo/testimages/testimgint.jpg",
        format: SourceFormat::Jpeg,
    },
    FixtureSpec {
        label: "jpeg-decoder baseline grayscale",
        relative_path:
            "third-party/fixtures/raster-images/upstream/jpeg-decoder/grayscale_square.jpg",
        format: SourceFormat::Jpeg,
    },
    FixtureSpec {
        label: "libjpeg-turbo 24-bit BMP",
        relative_path:
            "third-party/fixtures/raster-images/upstream/libjpeg-turbo/testimages/shira_bird8.bmp",
        format: SourceFormat::Bmp,
    },
];

struct RasterDraw {
    label: &'static str,
    source_material: MaterialHandle,
    inspection_material: MaterialHandle,
    dimensions: [u32; 2],
}

#[derive(Clone, Copy, Default)]
enum PresentationMode {
    #[default]
    Source,
    Inspection,
}

impl PresentationMode {
    const fn label(self) -> &'static str {
        match self {
            Self::Source => "source color",
            Self::Inspection => "cyan tint + 45% opacity",
        }
    }

    const fn toggled(self) -> Self {
        match self {
            Self::Source => Self::Inspection,
            Self::Inspection => Self::Source,
        }
    }
}

#[derive(Serialize)]
struct FixtureTextureEvidence {
    label: &'static str,
    encoded_format: &'static str,
    preparation: TextureUploadArtifact,
}

#[derive(Serialize)]
struct RasterShaderContractArtifact {
    schema: u32,
    artifact_kind: &'static str,
    provider_neutral_input: &'static str,
    fixture_texture_evidence: Vec<FixtureTextureEvidence>,
    material_texture_slot: &'static str,
    pipeline_kind: &'static str,
    shader_operation: &'static str,
    sampler_policy: &'static str,
    blend_policy: &'static str,
    orientation_policy: &'static str,
    gpu_framebuffer_captured: bool,
}

#[derive(Default)]
struct App {
    renderer: Option<WgpuBackend>,
    window: Option<Arc<NativeWindow>>,
    size: [f32; 2],
    pipeline: PipelineHandle,
    images: Vec<RasterDraw>,
    selected: usize,
    presentation_mode: PresentationMode,
}

fn main() -> PlatformResult<()> {
    run_window_with_app(
        WindowConfig {
            title: "Tokimu Hello Raster Image | loading corpus fixtures".into(),
            width: 960,
            height: 640,
        },
        App::default(),
    )
}

impl App {
    fn update_window_title(&self) {
        let Some(window) = self.window.as_ref() else {
            return;
        };
        let Some(image) = self.images.get(self.selected) else {
            return;
        };
        window.set_title(&format!(
            "Tokimu Hello Raster Image | {} | {}x{} | shader={} | space: mode | left/right: fixture",
            image.label,
            image.dimensions[0],
            image.dimensions[1],
            self.presentation_mode.label(),
        ));
    }

    fn step_selection(&mut self, offset: isize) {
        if self.images.is_empty() {
            return;
        }
        let count = self.images.len() as isize;
        self.selected = (self.selected as isize + offset).rem_euclid(count) as usize;
        self.update_window_title();
    }
}

impl PlatformEventHandler for App {
    fn on_native_window_created(&mut self, window: Arc<NativeWindow>) -> PlatformResult<()> {
        self.window = Some(window.clone());
        let size = window.inner_size();
        self.size = [size.width.max(1) as f32, size.height.max(1) as f32];

        let mut renderer = WgpuBackend::for_window(window, size.width, size.height)?;
        renderer.upload_mesh(QUAD, &Mesh::quad());
        let pipeline = Pipeline::new("hello-raster-image-texture", PipelineKind::Texture2d);
        debug_assert_eq!(
            pipeline.render_state.blend,
            BlendMode::AlphaBlend,
            "the raster proof requires explicit source-alpha blending"
        );
        self.pipeline = renderer.register_pipeline(&pipeline)?;

        let mut fixture_texture_evidence = Vec::with_capacity(FIXTURES.len());
        for (index, fixture) in FIXTURES.iter().enumerate() {
            let source_path = find_fixture(fixture.relative_path)?;
            let image = decode_fixture(fixture, &source_path)?;
            let prepared = prepare_renderer_texture(&image, TextureUse::ColorSrgb)
                .map_err(|error| error.to_string())?;
            fixture_texture_evidence.push(FixtureTextureEvidence {
                label: fixture.label,
                encoded_format: fixture.format.label(),
                preparation: prepared.artifact(),
            });
            let texture = TextureHandle(100 + index as u64);
            let source_material = MaterialHandle(100 + index as u64);
            let inspection_material = MaterialHandle(200 + index as u64);
            renderer.create_texture_rgba8(
                texture,
                Rgba8TextureDescriptor::new(
                    prepared.texture.width,
                    prepared.texture.height,
                    Rgba8TextureColorSpace::Srgb,
                ),
                &prepared.texture.rgba8,
            )?;
            renderer.upload_material(
                source_material,
                &Material::new("raster-image-source", Color::rgb(1.0, 1.0, 1.0))
                    .with_texture(texture),
            )?;
            renderer.upload_material(
                inspection_material,
                &Material::new(
                    "raster-image-inspection",
                    Color::rgba(0.45, 0.95, 1.0, 0.45),
                )
                .with_texture(texture),
            )?;
            self.images.push(RasterDraw {
                label: fixture.label,
                source_material,
                inspection_material,
                dimensions: [image.width, image.height],
            });
        }
        write_shader_contract_artifact(fixture_texture_evidence)?;

        self.renderer = Some(renderer);
        self.update_window_title();
        Ok(())
    }

    fn on_platform_event(&mut self, event: PlatformInputEvent) -> PlatformResult<()> {
        match event {
            PlatformInputEvent::Resized { width, height } => {
                self.size = [width.max(1) as f32, height.max(1) as f32];
                if let Some(renderer) = self.renderer.as_mut() {
                    renderer.resize_surface(width, height);
                }
            }
            PlatformInputEvent::KeyboardInput {
                key: KeyCode::ArrowLeft,
                pressed: true,
            } => {
                self.step_selection(-1);
            }
            PlatformInputEvent::KeyboardInput {
                key: KeyCode::ArrowRight,
                pressed: true,
            } => {
                self.step_selection(1);
            }
            PlatformInputEvent::KeyboardInput {
                key: KeyCode::Space,
                pressed: true,
            } => {
                self.presentation_mode = self.presentation_mode.toggled();
                self.update_window_title();
            }
            _ => {}
        }
        Ok(())
    }

    fn on_frame(&mut self, _delta_seconds: f64) -> PlatformResult<FrameOutcome> {
        let Some(renderer) = self.renderer.as_mut() else {
            return Ok(FrameOutcome::Continue);
        };
        let Some(image) = self.images.get(self.selected) else {
            return Ok(FrameOutcome::Continue);
        };

        renderer.upload_camera(CAMERA, Camera::orthographic_2d(self.size[0], self.size[1]));
        renderer.begin_frame();
        renderer.submit(&[RenderCommand::Clear(ClearCommand { color: BACKGROUND })]);

        let width = image.dimensions[0] as f32;
        let height = image.dimensions[1] as f32;
        let available_width = self.size[0] * 0.88;
        let available_height = self.size[1] * 0.82;
        let fit = (available_width / width)
            .min(available_height / height)
            .max(0.0);
        renderer.submit(&[RenderCommand::DrawMesh(DrawMeshCommand {
            mesh: QUAD,
            material: match self.presentation_mode {
                PresentationMode::Source => image.source_material,
                PresentationMode::Inspection => image.inspection_material,
            },
            pipeline: self.pipeline,
            instance: Instance2d::new(
                [0.0, 0.0],
                [width * fit / self.size[0], height * fit / self.size[1]],
                0.0,
            ),
            camera: Some(CAMERA),
            viewport: None,
        })]);
        let _ = renderer.present()?;
        Ok(FrameOutcome::Continue)
    }
}

fn write_shader_contract_artifact(
    fixture_texture_evidence: Vec<FixtureTextureEvidence>,
) -> Result<(), String> {
    let shader_source = PipelineKind::Texture2d
        .default_shader_source()
        .ok_or_else(|| "Texture2d has no default shader source".to_owned())?;
    if !shader_source.contains("textureSample(material_texture, material_sampler, uv)") {
        return Err("Texture2d shader no longer samples the material texture slot".to_owned());
    }

    let artifact = RasterShaderContractArtifact {
        schema: 1,
        artifact_kind: "raster-material-shader-contract",
        provider_neutral_input: "DecodedImage",
        fixture_texture_evidence,
        material_texture_slot: "Material.texture -> TextureHandle",
        pipeline_kind: "Texture2d",
        shader_operation: "textureSample(material_texture, material_sampler, uv) * material_color",
        sampler_policy: "renderer-owned default sampler",
        blend_policy: "source-alpha",
        orientation_policy: "top-down before texture preparation",
        gpu_framebuffer_captured: false,
    };
    let output = PathBuf::from("target/hello-raster-image/raster-shader-contract.json");
    if let Some(parent) = output.parent() {
        fs::create_dir_all(parent).map_err(|error| error.to_string())?;
    }
    let json = serde_json::to_string_pretty(&artifact).map_err(|error| error.to_string())?;
    fs::write(output, format!("{json}\n")).map_err(|error| error.to_string())
}

fn decode_fixture(fixture: &FixtureSpec, source_path: &Path) -> Result<DecodedImage, String> {
    let bytes = fs::read(source_path).map_err(|error| error.to_string())?;
    let limits = DecodeLimits::default();
    match fixture.format {
        SourceFormat::Png => decode_png(&bytes, limits),
        SourceFormat::Jpeg => decode_jpeg(&bytes, limits),
        SourceFormat::Bmp => decode_bmp(&bytes, limits),
    }
    .map_err(|error| format!("{}: {error}", source_path.display()))
}

fn find_fixture(relative_path: &str) -> Result<PathBuf, String> {
    let mut candidates = vec![PathBuf::from(relative_path)];
    if let Ok(current_dir) = std::env::current_dir() {
        candidates.extend(
            current_dir
                .ancestors()
                .map(|ancestor| ancestor.join(relative_path)),
        );
    }
    if let Ok(executable) = std::env::current_exe() {
        candidates.extend(
            executable
                .ancestors()
                .map(|ancestor| ancestor.join(relative_path)),
        );
    }
    candidates
        .into_iter()
        .find(|candidate| candidate.is_file())
        .ok_or_else(|| format!("missing raster fixture `{relative_path}`"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn viewer_fixtures_decode_to_texture_ready_rgba8() {
        for fixture in &FIXTURES {
            let source_path = find_fixture(fixture.relative_path)
                .expect("the viewer fixture should remain available in the repository");
            let decoded = decode_fixture(fixture, &source_path)
                .expect("the viewer fixture should remain within the bounded decoder profile");
            let prepared = prepare_renderer_texture(&decoded, TextureUse::ColorSrgb)
                .expect("a decoded viewer fixture should prepare as a color texture");

            assert!(prepared.texture.width > 0, "{}", fixture.label);
            assert!(prepared.texture.height > 0, "{}", fixture.label);
            assert!(!prepared.texture.rgba8.is_empty(), "{}", fixture.label);
        }
    }

    #[test]
    fn textured_pipeline_samples_material_texture_with_alpha_blending() {
        let shader = PipelineKind::Texture2d
            .default_shader_source()
            .expect("Texture2d should retain a built-in shader");
        assert!(shader.contains("@binding(1) var material_texture: texture_2d<f32>"));
        assert!(shader.contains("@binding(2) var material_sampler: sampler"));
        assert!(shader.contains("textureSample(material_texture, material_sampler, uv)"));
        assert_eq!(
            PipelineKind::Texture2d.default_render_state().blend,
            BlendMode::AlphaBlend
        );
    }
}
