use std::sync::Arc;

use render_orientation_conformance::{
    conformance_pipeline, cull_modes, directional_atlas_rgba8, fixture_cases, fixture_layout,
    DIRECTIONAL_ATLAS_HEIGHT, DIRECTIONAL_ATLAS_WIDTH,
};
use tokimu::{
    run_window_with_app, Camera, CameraHandle, ClearCommand, Color, DrawMeshCommand, FrameOutcome,
    Material, MaterialHandle, MeshHandle, NativeWindow, PipelineHandle, PlatformEventHandler,
    PlatformInputEvent, PlatformResult, RenderCommand, Renderer, Rgba8TextureColorSpace,
    Rgba8TextureDescriptor, TextureHandle, ViewportRect, WgpuBackend, WindowConfig,
};

const WIDTH: u32 = 1200;
const HEIGHT: u32 = 800;
const CAMERA: CameraHandle = CameraHandle(1);
const MATERIAL: MaterialHandle = MaterialHandle(1);
const DIRECTIONAL_ATLAS: TextureHandle = TextureHandle(1);
const FIRST_MESH: u64 = 1;

fn main() -> PlatformResult<()> {
    run_window_with_app(
        WindowConfig {
            title: "Tokimu Orientation | rows: identity, rotate, reflect, compensated | columns: none, back, front".into(),
            width: WIDTH,
            height: HEIGHT,
        },
        OrientationApp::default(),
    )
}

#[derive(Default)]
struct OrientationApp {
    renderer: Option<WgpuBackend>,
    size: [u32; 2],
    pipelines: Vec<PipelineHandle>,
}

impl PlatformEventHandler for OrientationApp {
    fn on_native_window_created(&mut self, window: Arc<NativeWindow>) -> PlatformResult<()> {
        let size = window.inner_size();
        self.size = [size.width.max(1), size.height.max(1)];
        let mut renderer =
            WgpuBackend::for_window(Arc::clone(&window), self.size[0], self.size[1])?;
        window.set_title(&format!(
            "Tokimu Orientation | adapter: {} | rows: identity, rotate, reflect, compensated | columns: none, back, front",
            renderer.adapter_name()
        ));

        for (index, case) in fixture_cases().into_iter().enumerate() {
            renderer.upload_mesh(MeshHandle(FIRST_MESH + index as u64), &case.mesh);
        }
        renderer.create_texture_rgba8(
            DIRECTIONAL_ATLAS,
            Rgba8TextureDescriptor::new(
                DIRECTIONAL_ATLAS_WIDTH,
                DIRECTIONAL_ATLAS_HEIGHT,
                Rgba8TextureColorSpace::Srgb,
            ),
            &directional_atlas_rgba8(),
        )?;
        renderer.upload_material(
            MATERIAL,
            &Material::new("orientation-fixture-material", Color::rgb(1.0, 1.0, 1.0))
                .with_texture(DIRECTIONAL_ATLAS),
        )?;
        renderer.upload_camera(CAMERA, Camera::default());
        self.pipelines = cull_modes()
            .into_iter()
            .map(|cull_mode| renderer.register_pipeline(&conformance_pipeline(cull_mode)))
            .collect::<Result<Vec<_>, _>>()?;
        self.renderer = Some(renderer);
        Ok(())
    }

    fn on_platform_event(&mut self, event: PlatformInputEvent) -> PlatformResult<()> {
        if let PlatformInputEvent::Resized { width, height } = event {
            self.size = [width.max(1), height.max(1)];
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

        renderer.begin_frame();
        let mut commands = vec![RenderCommand::Clear(ClearCommand {
            color: Color::rgb(0.025, 0.035, 0.045),
        })];
        for cell in fixture_layout(self.size[0], self.size[1]) {
            commands.push(RenderCommand::DrawMesh(DrawMeshCommand {
                mesh: MeshHandle(FIRST_MESH + cell.case_index as u64),
                material: MATERIAL,
                pipeline: self.pipelines[cell.cull_index],
                instance: cell.instance,
                camera: Some(CAMERA),
                viewport: Some(ViewportRect {
                    x: cell.viewport[0],
                    y: cell.viewport[1],
                    width: cell.viewport[2],
                    height: cell.viewport[3],
                }),
            }));
        }
        renderer.submit(&commands);
        let _ = renderer.present()?;
        Ok(FrameOutcome::Continue)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn native_consumer_uses_the_shared_twelve_cell_layout() {
        assert_eq!(fixture_layout(WIDTH, HEIGHT).len(), 12);
    }
}
