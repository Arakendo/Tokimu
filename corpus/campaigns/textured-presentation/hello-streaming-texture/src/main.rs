#[cfg(not(target_arch = "wasm32"))]
use std::sync::Arc;
#[cfg(any(test, not(target_arch = "wasm32")))]
use std::{fs, path::Path};

#[cfg(any(test, not(target_arch = "wasm32")))]
use screenshot::{write_bmp, write_manifest, Rgba8Image};
#[cfg(not(target_arch = "wasm32"))]
use tokimu::{
    run_window_with_app, Camera, CameraHandle, ClearCommand, Color, DrawMeshCommand, FrameOutcome,
    Instance2d, Material, MaterialHandle, Mesh, MeshHandle, NativeWindow, Pipeline, PipelineHandle,
    PipelineKind, PlatformEventHandler, PlatformInputEvent, PlatformResult, RenderCommand,
    Renderer, Rgba8TextureColorSpace, Rgba8TextureDescriptor, TextureHandle, WgpuBackend,
    WindowConfig,
};

#[cfg(any(test, not(target_arch = "wasm32")))]
const DEFAULT_WIDTH: u32 = 320;
#[cfg(any(test, not(target_arch = "wasm32")))]
const DEFAULT_HEIGHT: u32 = 180;
#[cfg(any(test, not(target_arch = "wasm32")))]
const STRESS_WIDTH: u32 = 1920;
#[cfg(any(test, not(target_arch = "wasm32")))]
const STRESS_HEIGHT: u32 = 1080;
#[cfg(any(test, not(target_arch = "wasm32")))]
const VALIDATION_FRAME: u64 = 300;
#[cfg(not(target_arch = "wasm32"))]
const QUAD: MeshHandle = MeshHandle(1);
#[cfg(not(target_arch = "wasm32"))]
const MATERIAL: MaterialHandle = MaterialHandle(1);
#[cfg(not(target_arch = "wasm32"))]
const TEXTURE: TextureHandle = TextureHandle(1);
#[cfg(not(target_arch = "wasm32"))]
const CAMERA: CameraHandle = CameraHandle(1);

#[cfg(any(test, not(target_arch = "wasm32")))]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct TextureProfile {
    name: &'static str,
    width: u32,
    height: u32,
}

#[cfg(any(test, not(target_arch = "wasm32")))]
impl TextureProfile {
    const DEFAULT: Self = Self {
        name: "default",
        width: DEFAULT_WIDTH,
        height: DEFAULT_HEIGHT,
    };
    const STRESS_1080P: Self = Self {
        name: "stress-1080p",
        width: STRESS_WIDTH,
        height: STRESS_HEIGHT,
    };

    fn from_argument(argument: Option<&str>) -> Self {
        match argument {
            Some("--stress-1080p") => Self::STRESS_1080P,
            _ => Self::DEFAULT,
        }
    }
}

#[cfg(any(test, not(target_arch = "wasm32")))]
impl Default for TextureProfile {
    fn default() -> Self {
        Self::DEFAULT
    }
}

#[cfg(any(test, not(target_arch = "wasm32")))]
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
struct RunOptions {
    texture_profile: TextureProfile,
    exit_after_validation: bool,
}

#[cfg(any(test, not(target_arch = "wasm32")))]
impl RunOptions {
    fn from_arguments(arguments: impl IntoIterator<Item = String>) -> Self {
        let arguments = arguments.into_iter().collect::<Vec<_>>();
        Self {
            texture_profile: TextureProfile::from_argument(
                arguments
                    .iter()
                    .find(|argument| argument.as_str() == "--stress-1080p")
                    .map(String::as_str),
            ),
            exit_after_validation: arguments
                .iter()
                .any(|argument| argument == "--exit-after-validation"),
        }
    }

    #[cfg(not(target_arch = "wasm32"))]
    fn from_command_line() -> Self {
        Self::from_arguments(std::env::args().skip(1))
    }
}

#[cfg(not(target_arch = "wasm32"))]
#[derive(Default)]
struct App {
    renderer: Option<WgpuBackend>,
    window: Option<Arc<NativeWindow>>,
    pipeline: PipelineHandle,
    pixels: Vec<u8>,
    texture_profile: TextureProfile,
    exit_after_validation: bool,
    frame: u64,
    size: [f32; 2],
    validated: bool,
}

#[cfg(not(target_arch = "wasm32"))]
fn main() -> PlatformResult<()> {
    let options = RunOptions::from_command_line();
    run_window_with_app(
        WindowConfig {
            title: "Tokimu Hello Streaming Texture | warming up".into(),
            width: 960,
            height: 600,
        },
        App {
            texture_profile: options.texture_profile,
            exit_after_validation: options.exit_after_validation,
            ..App::default()
        },
    )
}

#[cfg(target_arch = "wasm32")]
fn main() {
    // Native-window motion is manual evidence. The frame-generation contract
    // and renderer API remain portable; browser visual proof is a later slice.
}

#[cfg(not(target_arch = "wasm32"))]
impl PlatformEventHandler for App {
    fn on_native_window_created(&mut self, window: Arc<NativeWindow>) -> PlatformResult<()> {
        let size = window.inner_size();
        self.size = [size.width.max(1) as f32, size.height.max(1) as f32];
        self.window = Some(window.clone());

        let mut renderer = WgpuBackend::for_window(window, size.width, size.height)?;
        renderer.upload_mesh(QUAD, &Mesh::quad());
        self.pipeline = renderer.register_pipeline(&Pipeline::new(
            "hello-streaming-texture",
            PipelineKind::Texture2d,
        ))?;

        self.pixels =
            vec![0; (self.texture_profile.width * self.texture_profile.height * 4) as usize];
        generate_frame(
            &mut self.pixels,
            self.texture_profile.width,
            self.texture_profile.height,
            0,
        );
        renderer.create_texture_rgba8(
            TEXTURE,
            Rgba8TextureDescriptor::new(
                self.texture_profile.width,
                self.texture_profile.height,
                Rgba8TextureColorSpace::Srgb,
            ),
            &self.pixels,
        )?;
        renderer.upload_material(
            MATERIAL,
            &Material::new("streaming-frame", Color::rgb(1.0, 1.0, 1.0)).with_texture(TEXTURE),
        )?;
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

        self.frame = self.frame.saturating_add(1);
        generate_frame(
            &mut self.pixels,
            self.texture_profile.width,
            self.texture_profile.height,
            self.frame,
        );
        renderer.upload_camera(CAMERA, Camera::orthographic_2d(self.size[0], self.size[1]));
        renderer.begin_frame();
        renderer.update_texture_rgba8(
            TEXTURE,
            self.texture_profile.width,
            self.texture_profile.height,
            &self.pixels,
        )?;
        renderer.submit(&[
            RenderCommand::Clear(ClearCommand {
                color: Color::rgb(0.025, 0.035, 0.05),
            }),
            RenderCommand::DrawMesh(DrawMeshCommand {
                mesh: QUAD,
                material: MATERIAL,
                pipeline: self.pipeline,
                instance: Instance2d::new([0.0, 0.0], [0.9, 0.9], 0.0),
                camera: Some(CAMERA),
                viewport: None,
            }),
        ]);
        let stats = renderer.present()?;

        if !self.validated && self.frame >= VALIDATION_FRAME {
            validate_steady_state(self.frame, stats)?;
            write_source_frame_evidence(self.frame, self.texture_profile)
                .map_err(|error| -> Box<dyn std::error::Error> { error.into() })?;
            self.validated = true;
            if let Some(window) = self.window.as_ref() {
                window.set_title(&format!(
                    "Tokimu Hello Streaming Texture | {} | validated {} writes | allocations=1 replacements=0",
                    self.texture_profile.name, self.frame
                ));
            }
            println!(
                "hello-streaming-texture validated: frames={}, allocations={}, replacements={}, writes={}",
                self.frame,
                stats.lifetime.texture_allocations,
                stats.lifetime.texture_replacements,
                stats.lifetime.texture_writes
            );

            if self.exit_after_validation {
                println!(
                    "hello-streaming-texture completed bounded native run after {} frames",
                    self.frame
                );
                return Ok(FrameOutcome::Exit);
            }
        }

        Ok(FrameOutcome::Continue)
    }
}

/// Writes deterministic source-frame artifacts. These prove the application
/// generated distinct payloads; they are not GPU surface captures.
#[cfg(any(test, not(target_arch = "wasm32")))]
fn write_source_frame_evidence(
    validated_frame: u64,
    profile: TextureProfile,
) -> Result<(), String> {
    let output = Path::new("target/hello-streaming-texture");
    fs::create_dir_all(output).map_err(|error| error.to_string())?;

    write_source_frame_artifact(output, "source-frame-000.bmp", 0, profile)?;
    write_source_frame_artifact(
        output,
        "source-frame-validated.bmp",
        validated_frame,
        profile,
    )
}

#[cfg(any(test, not(target_arch = "wasm32")))]
fn write_source_frame_artifact(
    output: &Path,
    name: &str,
    frame: u64,
    profile: TextureProfile,
) -> Result<(), String> {
    let mut pixels = vec![0; (profile.width * profile.height * 4) as usize];
    generate_frame(&mut pixels, profile.width, profile.height, frame);
    let image_path = output.join(name);
    write_bmp(
        &image_path,
        Rgba8Image {
            width: profile.width,
            height: profile.height,
            pixels: &pixels,
        },
    )?;

    let frame = frame.to_string();
    let width = profile.width.to_string();
    let height = profile.height.to_string();
    write_manifest(
        image_path.with_extension("txt"),
        &[
            ("example", "hello-streaming-texture"),
            ("capture_kind", "deterministic-cpu-source-frame"),
            ("gpu_framebuffer_capture", "false"),
            ("frame", &frame),
            ("profile", profile.name),
            ("width", &width),
            ("height", &height),
            ("color_space", "srgb-rgba8"),
        ],
    )
}

#[cfg(any(test, not(target_arch = "wasm32")))]
fn generate_frame(rgba8: &mut [u8], width: u32, height: u32, frame: u64) {
    let phase = frame as u32;
    for y in 0..height {
        for x in 0..width {
            let offset = ((y * width + x) * 4) as usize;
            let checker = (((x + phase) / 16) + ((y + phase / 2) / 16)) & 1;
            let wave = ((x + phase * 2) % width) as u8;
            rgba8[offset] = if checker == 0 { wave } else { 24 };
            rgba8[offset + 1] = if checker == 0 { 210 } else { wave };
            rgba8[offset + 2] = 235_u8.saturating_sub(wave / 2);
            rgba8[offset + 3] = 255;
        }
    }
}

#[cfg(any(test, not(target_arch = "wasm32")))]
fn validate_steady_state(frame: u64, stats: tokimu::RenderStats) -> Result<(), String> {
    if stats.frame.texture_allocations != 0
        || stats.frame.texture_replacements != 0
        || stats.frame.texture_writes != 1
        || stats.frame.binding_allocations != 0
        || stats.frame.pipeline_creations != 0
        || stats.lifetime.texture_allocations != 1
        || stats.lifetime.texture_replacements != 0
        || stats.lifetime.texture_writes != frame + 1
    {
        return Err(format!(
            "streaming texture lifecycle invariant failed at frame {frame}: {stats:?}"
        ));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn generated_frames_are_deterministic_and_change_over_time() {
        let mut first = vec![0; (DEFAULT_WIDTH * DEFAULT_HEIGHT * 4) as usize];
        let mut repeated = first.clone();
        let mut later = first.clone();
        generate_frame(&mut first, DEFAULT_WIDTH, DEFAULT_HEIGHT, 12);
        generate_frame(&mut repeated, DEFAULT_WIDTH, DEFAULT_HEIGHT, 12);
        generate_frame(&mut later, DEFAULT_WIDTH, DEFAULT_HEIGHT, 13);

        assert_eq!(first, repeated);
        assert_ne!(first, later);
        assert!(first.chunks_exact(4).all(|pixel| pixel[3] == 255));
    }

    #[test]
    fn steady_state_validation_accepts_one_write_without_resource_churn() {
        let frame = VALIDATION_FRAME;
        let mut frame_stats = tokimu::RenderFrameStats::EMPTY;
        frame_stats.texture_writes = 1;
        let mut lifetime = tokimu::RenderLifetimeStats::EMPTY;
        lifetime.texture_allocations = 1;
        lifetime.texture_writes = frame + 1;

        validate_steady_state(
            frame,
            tokimu::RenderStats {
                frame: frame_stats,
                lifetime,
            },
        )
        .expect("one stable write should satisfy the streaming contract");
    }

    #[test]
    fn steady_state_validation_rejects_hidden_binding_churn() {
        let frame = VALIDATION_FRAME;
        let mut frame_stats = tokimu::RenderFrameStats::EMPTY;
        frame_stats.texture_writes = 1;
        frame_stats.binding_allocations = 1;
        let mut lifetime = tokimu::RenderLifetimeStats::EMPTY;
        lifetime.texture_allocations = 1;
        lifetime.texture_writes = frame + 1;

        assert!(validate_steady_state(
            frame,
            tokimu::RenderStats {
                frame: frame_stats,
                lifetime
            }
        )
        .is_err());
    }

    #[test]
    fn source_frame_artifacts_are_explicitly_labeled_non_gpu_evidence() {
        let directory = std::env::temp_dir().join("tokimu-streaming-texture-evidence-test");
        let _ = fs::remove_dir_all(&directory);
        fs::create_dir_all(&directory).expect("temporary evidence directory should be created");

        write_source_frame_artifact(&directory, "frame.bmp", 7, TextureProfile::DEFAULT)
            .expect("source artifact should be written");
        let manifest = fs::read_to_string(directory.join("frame.txt"))
            .expect("source artifact manifest should be readable");
        assert!(manifest.contains("capture_kind=deterministic-cpu-source-frame"));
        assert!(manifest.contains("gpu_framebuffer_capture=false"));

        let _ = fs::remove_dir_all(directory);
    }

    #[test]
    fn stress_profile_is_explicit_and_does_not_change_the_default_case() {
        assert_eq!(TextureProfile::from_argument(None), TextureProfile::DEFAULT);
        assert_eq!(
            TextureProfile::from_argument(Some("--stress-1080p")),
            TextureProfile::STRESS_1080P
        );
        assert_eq!(TextureProfile::STRESS_1080P.width, 1920);
        assert_eq!(TextureProfile::STRESS_1080P.height, 1080);
    }

    #[test]
    fn bounded_run_option_is_opt_in() {
        assert_eq!(RunOptions::default(), RunOptions::from_arguments([]));
        assert_eq!(
            RunOptions {
                texture_profile: TextureProfile::STRESS_1080P,
                exit_after_validation: true,
            },
            RunOptions::from_arguments([
                "--stress-1080p".to_owned(),
                "--exit-after-validation".to_owned(),
            ])
        );
    }
}
