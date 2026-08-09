use std::{io, path::PathBuf, sync::Arc};

use gltf_corpus::{decode_glb_file, DecodedPrimitive};
use raster_image_corpus::{decode_png, prepare_renderer_texture, DecodeLimits, TextureUse};
use tokimu::{
    run_window_with_app, Camera, CameraHandle, ClearCommand, Color, CullMode, DrawMeshCommand,
    FrameOutcome, Instance2d, Material, MaterialHandle, Mesh, MeshHandle, NativeWindow, Pipeline,
    PipelineHandle, PipelineKind, PipelineRenderState, PlatformEventHandler, PlatformInputEvent,
    PlatformResult, RenderCommand, Renderer, Rgba8TextureColorSpace, Rgba8TextureDescriptor,
    TextureAddressMode, TextureFilter, TextureHandle, TextureSampler, WgpuBackend, WindowConfig,
};
use tokimu_core::math::{Mat4, Vec3};

const MESH: MeshHandle = MeshHandle(1);
const MATERIAL: MaterialHandle = MaterialHandle(1);
const TEXTURES: [TextureFixture; 3] = [
    TextureFixture {
        label: "grid",
        source: "corpus/assets/PNG/Dark/texture_01.png",
        handle: TextureHandle(1),
    },
    TextureFixture {
        label: "door-dark",
        source: "corpus/assets/PNG/Dark/texture_11.png",
        handle: TextureHandle(2),
    },
    TextureFixture {
        label: "door-green",
        source: "corpus/assets/PNG/Green/texture_11.png",
        handle: TextureHandle(3),
    },
];
const CAMERA: CameraHandle = CameraHandle(1);
const BOX_SOURCE: &str =
    "third-party/fixtures/khronos-gltf-sample-assets/upstream/Models/Box/glTF-Binary/Box.glb";
/// Intentionally exceeds the unit interval so sampler modes exercise
/// addressing rather than relying on an accidental visual difference.
const ADDRESSING_UV_SCALE: f32 = 3.25;

#[derive(Clone, Copy)]
struct TextureFixture {
    label: &'static str,
    source: &'static str,
    handle: TextureHandle,
}

#[derive(Clone, Copy, Default)]
enum SamplerMode {
    #[default]
    PointClamp,
    PointRepeat,
    LinearClamp,
    LinearRepeat,
}

#[derive(Clone, Copy, Default)]
enum UvMode {
    #[default]
    Identity,
    FlipU,
    SwapUv,
}

impl UvMode {
    fn next(self) -> Self {
        match self {
            Self::Identity => Self::FlipU,
            Self::FlipU => Self::SwapUv,
            Self::SwapUv => Self::Identity,
        }
    }

    fn apply(self, [u, v]: [f32; 2]) -> [f32; 2] {
        match self {
            Self::Identity => [u, v],
            Self::FlipU => [ADDRESSING_UV_SCALE - u, v],
            Self::SwapUv => [v, u],
        }
    }

    fn label(self) -> &'static str {
        match self {
            Self::Identity => "uv identity",
            Self::FlipU => "uv flip-u",
            Self::SwapUv => "uv swap",
        }
    }
}

impl SamplerMode {
    fn next(self) -> Self {
        match self {
            Self::PointClamp => Self::PointRepeat,
            Self::PointRepeat => Self::LinearClamp,
            Self::LinearClamp => Self::LinearRepeat,
            Self::LinearRepeat => Self::PointClamp,
        }
    }
    fn sampler(self) -> TextureSampler {
        match self {
            Self::PointClamp => TextureSampler::default(),
            Self::PointRepeat => TextureSampler {
                filter: TextureFilter::Point,
                address_u: TextureAddressMode::Repeat,
                address_v: TextureAddressMode::Repeat,
            },
            Self::LinearClamp => TextureSampler {
                filter: TextureFilter::Linear,
                address_u: TextureAddressMode::Clamp,
                address_v: TextureAddressMode::Clamp,
            },
            Self::LinearRepeat => TextureSampler {
                filter: TextureFilter::Linear,
                address_u: TextureAddressMode::Repeat,
                address_v: TextureAddressMode::Repeat,
            },
        }
    }
    fn label(self) -> &'static str {
        match self {
            Self::PointClamp => "point clamp",
            Self::PointRepeat => "point repeat",
            Self::LinearClamp => "linear clamp",
            Self::LinearRepeat => "linear repeat",
        }
    }
}

#[derive(Default)]
struct App {
    renderer: Option<WgpuBackend>,
    size: [f32; 2],
    pipeline: PipelineHandle,
    mesh: Mesh,
    window: Option<Arc<NativeWindow>>,
    texture_index: usize,
    sampler_mode: SamplerMode,
    uv_mode: UvMode,
}

fn main() -> PlatformResult<()> {
    run_window_with_app(
        WindowConfig {
            title: "Tokimu Textured Box | loading".into(),
            width: 960,
            height: 720,
        },
        App::default(),
    )
}

impl PlatformEventHandler for App {
    fn on_native_window_created(&mut self, window: Arc<NativeWindow>) -> PlatformResult<()> {
        let size = window.inner_size();
        self.size = [size.width.max(1) as f32, size.height.max(1) as f32];
        self.mesh = load_box_mesh_with_planar_uvs(self.uv_mode)?;

        let mut renderer = WgpuBackend::for_window(window.clone(), size.width, size.height)?;
        for fixture in TEXTURES {
            let png = std::fs::read(workspace_path(fixture.source))?;
            let decoded = decode_png(&png, DecodeLimits::default())?;
            let prepared = prepare_renderer_texture(&decoded, TextureUse::ColorSrgb)
                .map_err(|error| io::Error::other(error.to_string()))?;
            renderer.create_texture_rgba8(
                fixture.handle,
                Rgba8TextureDescriptor::new(
                    prepared.texture.width,
                    prepared.texture.height,
                    Rgba8TextureColorSpace::Srgb,
                ),
                &prepared.texture.rgba8,
            )?;
        }
        self.pipeline = renderer.register_pipeline(
            &Pipeline::new("textured-box-3d", PipelineKind::Textured3d).with_render_state(
                PipelineRenderState {
                    cull_mode: CullMode::Back,
                    ..PipelineRenderState::depth_writing_3d()
                },
            )?,
        )?;
        self.window = Some(window);
        self.renderer = Some(renderer);
        self.update_material()?;
        Ok(())
    }

    fn on_platform_event(&mut self, event: PlatformInputEvent) -> PlatformResult<()> {
        if let PlatformInputEvent::Resized { width, height } = event {
            self.size = [width.max(1) as f32, height.max(1) as f32];
            if let Some(renderer) = self.renderer.as_mut() {
                renderer.resize_surface(width, height);
            }
        }
        if let PlatformInputEvent::KeyboardInput { key, pressed: true } = event {
            match key {
                tokimu::KeyCode::KeyM => {
                    self.texture_index = (self.texture_index + 1) % TEXTURES.len();
                    self.update_material()?;
                }
                tokimu::KeyCode::KeyR => {
                    self.sampler_mode = self.sampler_mode.next();
                    self.update_material()?;
                }
                tokimu::KeyCode::KeyX => {
                    self.uv_mode = self.uv_mode.next();
                    self.mesh = load_box_mesh_with_planar_uvs(self.uv_mode)?;
                    self.update_material()?;
                }
                _ => {}
            }
        }
        Ok(())
    }

    fn on_frame(&mut self, _delta_seconds: f64) -> PlatformResult<FrameOutcome> {
        let mut camera = Camera::perspective_3d(self.size[0], self.size[1]);
        camera.view = Mat4::look_at_rh(Vec3::new(2.8, 1.8, 2.8), Vec3::ZERO, Vec3::Y);
        let renderer = self
            .renderer
            .as_mut()
            .ok_or_else(|| io::Error::other("renderer missing"))?;
        renderer.upload_mesh(MESH, &self.mesh);
        renderer.upload_camera(CAMERA, camera);
        renderer.begin_frame();
        renderer.submit(&[
            RenderCommand::Clear(ClearCommand {
                color: Color::rgb(0.03, 0.04, 0.06),
            }),
            RenderCommand::DrawMesh(DrawMeshCommand {
                mesh: MESH,
                material: MATERIAL,
                pipeline: self.pipeline,
                instance: Instance2d::identity(),
                camera: Some(CAMERA),
                viewport: None,
            }),
        ]);
        renderer.present()?;
        Ok(FrameOutcome::Continue)
    }
}

impl App {
    fn update_material(&mut self) -> PlatformResult<()> {
        let fixture = TEXTURES[self.texture_index];
        self.renderer
            .as_mut()
            .ok_or_else(|| io::Error::other("renderer missing"))?
            .upload_material(
                MATERIAL,
                &Material::new(
                    format!("textured-box-{}", fixture.label),
                    Color::rgb(1.0, 1.0, 1.0),
                )
                .with_texture(fixture.handle)
                .with_texture_sampler(self.sampler_mode.sampler()),
            )?;
        if let Some(window) = &self.window {
            window.set_title(&format!(
                "Tokimu Textured Box | {} | {} | {} | M texture; R sampler; X UV",
                fixture.label,
                self.sampler_mode.label(),
                self.uv_mode.label(),
            ));
        }
        Ok(())
    }
}

fn load_box_mesh_with_planar_uvs(uv_mode: UvMode) -> PlatformResult<Mesh> {
    let model = decode_glb_file(workspace_path(BOX_SOURCE))?;
    let primitive = model
        .primitives
        .first()
        .ok_or_else(|| io::Error::other("Box has no primitive"))?;
    let mut positions = Vec::with_capacity(primitive.indices.len());
    let mut normals = Vec::with_capacity(primitive.indices.len());
    let mut uvs = Vec::with_capacity(primitive.indices.len());
    for &index in &primitive.indices {
        let index = index as usize;
        let position = *primitive
            .positions
            .get(index)
            .ok_or_else(|| io::Error::other("Box index outside positions"))?;
        let normal = *primitive
            .normals
            .get(index)
            .ok_or_else(|| io::Error::other("Box index outside normals"))?;
        positions.push(position);
        normals.push(normal);
        uvs.push(uv_mode.apply(planar_uv(position, normal)));
    }
    Mesh::new(positions, normals)
        .with_texture_coordinates(uvs)
        .map_err(Into::into)
}

fn planar_uv(position: [f32; 3], normal: [f32; 3]) -> [f32; 2] {
    let [x, y, z] = position;
    let coordinates = if normal[2].abs() > 0.5 {
        [x + 0.5, 0.5 - y]
    } else if normal[0].abs() > 0.5 {
        [z + 0.5, 0.5 - y]
    } else {
        [x + 0.5, z + 0.5]
    };
    [
        coordinates[0] * ADDRESSING_UV_SCALE,
        coordinates[1] * ADDRESSING_UV_SCALE,
    ]
}

fn workspace_path(relative: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .join(relative)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn decoded_box_expands_to_supplied_uvs() {
        let mesh =
            load_box_mesh_with_planar_uvs(UvMode::Identity).expect("Box conversion should succeed");
        assert_eq!(mesh.positions.len(), 36);
        assert_eq!(mesh.positions.len(), mesh.texture_coordinates.len());
        assert!(mesh.has_texture_coordinates());
        assert!(mesh
            .texture_coordinates
            .iter()
            .any(|coordinates| coordinates[0] > 1.0 || coordinates[1] > 1.0));
    }

    #[test]
    fn uv_modes_transform_the_same_source_coordinates_deterministically() {
        let coordinates = [0.5, 1.25];
        assert_eq!(UvMode::Identity.apply(coordinates), coordinates);
        assert_eq!(UvMode::FlipU.apply(coordinates), [2.75, 1.25]);
        assert_eq!(UvMode::SwapUv.apply(coordinates), [1.25, 0.5]);
    }
}
