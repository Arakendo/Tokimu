#[cfg(not(target_arch = "wasm32"))]
use std::sync::Arc;

#[cfg(not(target_arch = "wasm32"))]
use tokimu::{
    run_window_with_app, Camera, CameraHandle, ClearCommand, Color, DrawMeshCommand, FrameOutcome,
    Instance2d, Material, MaterialHandle, Mesh, MeshHandle, NativeWindow, Pipeline, PipelineHandle,
    PipelineKind, PlatformEventHandler, PlatformInputEvent, PlatformResult, RenderCommand,
    Renderer, Rgba8TextureColorSpace, Rgba8TextureDescriptor, TextureHandle, WgpuBackend,
    WindowConfig,
};

#[cfg(any(test, not(target_arch = "wasm32")))]
const IMAGE_WIDTH: u32 = 256;
#[cfg(any(test, not(target_arch = "wasm32")))]
const IMAGE_HEIGHT: u32 = 256;
#[cfg(not(target_arch = "wasm32"))]
const QUAD: MeshHandle = MeshHandle(1);
#[cfg(not(target_arch = "wasm32"))]
const CAMERA: CameraHandle = CameraHandle(1);
#[cfg(not(target_arch = "wasm32"))]
const LINEAR_TEXTURE: TextureHandle = TextureHandle(1);
#[cfg(not(target_arch = "wasm32"))]
const SRGB_TEXTURE: TextureHandle = TextureHandle(2);
#[cfg(not(target_arch = "wasm32"))]
const LINEAR_MATERIAL: MaterialHandle = MaterialHandle(1);
#[cfg(not(target_arch = "wasm32"))]
const SRGB_MATERIAL: MaterialHandle = MaterialHandle(2);

#[cfg(not(target_arch = "wasm32"))]
#[derive(Default)]
struct App {
    renderer: Option<WgpuBackend>,
    size: [f32; 2],
    pipeline: PipelineHandle,
}

#[cfg(not(target_arch = "wasm32"))]
fn main() -> PlatformResult<()> {
    run_window_with_app(
        WindowConfig {
            title: "Tokimu Hello Texture Color Space | linear / sRGB RGBA8".into(),
            width: 960,
            height: 540,
        },
        App::default(),
    )
}

#[cfg(target_arch = "wasm32")]
fn main() {
    // This corpus is a native visual comparison. Its portable descriptor and
    // byte-generation checks still compile for WASM; browser execution is a
    // separate evidence slice.
}

#[cfg(not(target_arch = "wasm32"))]
impl PlatformEventHandler for App {
    fn on_native_window_created(&mut self, window: Arc<NativeWindow>) -> PlatformResult<()> {
        let size = window.inner_size();
        self.size = [size.width.max(1) as f32, size.height.max(1) as f32];
        let mut renderer = WgpuBackend::for_window(window, size.width, size.height)?;
        renderer.upload_mesh(QUAD, &Mesh::quad());
        self.pipeline = renderer.register_pipeline(&Pipeline::new(
            "hello-texture-color-space",
            PipelineKind::Texture2d,
        ))?;

        let encoded_srgb = encoded_srgb_ramp(IMAGE_WIDTH, IMAGE_HEIGHT);
        renderer.create_texture_rgba8(
            LINEAR_TEXTURE,
            Rgba8TextureDescriptor::new(IMAGE_WIDTH, IMAGE_HEIGHT, Rgba8TextureColorSpace::Linear),
            &encoded_srgb,
        )?;
        renderer.create_texture_rgba8(
            SRGB_TEXTURE,
            Rgba8TextureDescriptor::new(IMAGE_WIDTH, IMAGE_HEIGHT, Rgba8TextureColorSpace::Srgb),
            &encoded_srgb,
        )?;
        renderer.upload_material(
            LINEAR_MATERIAL,
            &Material::new("encoded-srgb-as-linear", Color::rgb(1.0, 1.0, 1.0))
                .with_texture(LINEAR_TEXTURE),
        )?;
        renderer.upload_material(
            SRGB_MATERIAL,
            &Material::new("encoded-srgb-as-srgb", Color::rgb(1.0, 1.0, 1.0))
                .with_texture(SRGB_TEXTURE),
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
        renderer.upload_camera(CAMERA, Camera::orthographic_2d(self.size[0], self.size[1]));
        renderer.begin_frame();
        renderer.submit(&[
            RenderCommand::Clear(ClearCommand {
                color: Color::rgb(0.025, 0.035, 0.05),
            }),
            texture_draw(LINEAR_MATERIAL, self.pipeline, -0.46),
            texture_draw(SRGB_MATERIAL, self.pipeline, 0.46),
        ]);
        renderer.present()?;
        Ok(FrameOutcome::Continue)
    }
}

#[cfg(not(target_arch = "wasm32"))]
fn texture_draw(material: MaterialHandle, pipeline: PipelineHandle, x: f32) -> RenderCommand {
    RenderCommand::DrawMesh(DrawMeshCommand {
        mesh: QUAD,
        material,
        pipeline,
        instance: Instance2d::new([x, 0.0], [0.42, 0.74], 0.0),
        camera: Some(CAMERA),
        viewport: None,
    })
}

#[cfg(any(test, not(target_arch = "wasm32")))]
fn encoded_srgb_ramp(width: u32, height: u32) -> Vec<u8> {
    let mut rgba8 = vec![0; (width * height * 4) as usize];
    for y in 0..height {
        for x in 0..width {
            let offset = ((y * width + x) * 4) as usize;
            let value = ((x * 255) / width.saturating_sub(1).max(1)) as u8;
            rgba8[offset..offset + 4].copy_from_slice(&[value, value, value, 255]);
        }
    }
    rgba8
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn encoded_ramp_is_deterministic_and_opaque() {
        let first = encoded_srgb_ramp(4, 2);
        assert_eq!(first, encoded_srgb_ramp(4, 2));
        assert!(first.chunks_exact(4).all(|pixel| pixel[3] == 255));
        assert_eq!(first[0], 0);
        assert_eq!(first[12], 255);
    }

    #[test]
    fn comparison_descriptors_differ_only_by_color_interpretation() {
        let linear =
            Rgba8TextureDescriptor::new(IMAGE_WIDTH, IMAGE_HEIGHT, Rgba8TextureColorSpace::Linear);
        let srgb =
            Rgba8TextureDescriptor::new(IMAGE_WIDTH, IMAGE_HEIGHT, Rgba8TextureColorSpace::Srgb);

        assert_eq!(linear.width, srgb.width);
        assert_eq!(linear.height, srgb.height);
        assert_ne!(linear.color_space, srgb.color_space);
    }
}
