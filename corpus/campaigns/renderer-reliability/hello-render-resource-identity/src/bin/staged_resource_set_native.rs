use std::sync::Arc;

use tokimu_core::FrameOutcome;
use tokimu_platform::{
    run_window_with_app, NativeWindow, PlatformEventHandler, PlatformInputEvent, PlatformResult,
    WindowConfig,
};
use tokimu_render::{
    Camera, CameraHandle, ClearCommand, Color, DrawMeshCommand, Instance2d, Material,
    MaterialHandle, Mesh, MeshHandle, Pipeline, PipelineKind, RenderCommand, RenderCommandSetError,
    RenderResourceSetLifecycle, Renderer, Rgba8TextureColorSpace, Rgba8TextureDescriptor,
    TextureHandle, WgpuBackend, WgpuBackendError, WgpuResourceSetStage,
};

const MESH: MeshHandle = MeshHandle(1);
const MATERIAL: MaterialHandle = MaterialHandle(1);
const TEXTURE: TextureHandle = TextureHandle(1);
const CAMERA: CameraHandle = CameraHandle(1);

fn main() -> PlatformResult<()> {
    run_window_with_app(
        WindowConfig {
            title: "Tokimu ADR-0018 native conformance".into(),
            width: 640,
            height: 360,
        },
        NativeResourceSetProbe::default(),
    )
}

#[derive(Default)]
struct NativeResourceSetProbe {
    renderer: Option<WgpuBackend>,
    complete: bool,
}

impl NativeResourceSetProbe {
    fn run_probe(&mut self) -> PlatformResult<()> {
        let renderer = self.renderer.as_mut().expect("window initializes renderer");
        let commands_a = populate_backend(renderer, 0)?;
        let retained_a = renderer.scope_render_commands(&commands_a);
        let set_a = retained_a.resource_set();
        renderer.begin_frame();
        renderer.submit(&commands_a);
        let presented_a = renderer.present()?;

        let mut failed_b = renderer.begin_resource_set_stage()?;
        let failed_commands = populate_stage(&mut failed_b, 1)?;
        failed_b.begin_frame();
        failed_b.submit(&failed_commands);
        let forced_failure = failed_b
            .upload_material(
                MaterialHandle(2),
                &Material::new("forced-late-failure", Color::rgb(1.0, 1.0, 1.0))
                    .with_texture(TextureHandle(2)),
            )
            .expect_err("missing candidate texture must reject");
        drop(failed_b);

        renderer.begin_frame();
        renderer.submit(&commands_a);
        let preserved_a = renderer.present()?;

        let mut scoped_b = None;
        let commit = renderer.replace_resource_set(|candidate| {
            let commands_b = populate_stage(candidate, 1)?;
            scoped_b = Some(candidate.scope_render_commands(&commands_b));
            candidate.begin_frame();
            candidate.submit(&commands_b);
            Ok(())
        })?;
        let scoped_b = scoped_b.expect("successful stage scopes B commands");
        let set_b = scoped_b.resource_set();
        let stale_a = renderer
            .submit_render_command_set(&retained_a)
            .expect_err("retired A command must reject before handle resolution");
        assert!(matches!(
            stale_a,
            WgpuBackendError::RenderCommandSet(RenderCommandSetError::StaleResourceSet {
                requested,
                current,
            }) if requested == set_a && current == set_b
        ));
        let presented_b = renderer.present()?;
        renderer.begin_frame();
        renderer.submit(&commands_a);
        let unscoped_a_after_b = renderer.present()?;
        renderer.begin_frame();
        renderer.submit_render_command_set(&scoped_b)?;
        let presented_scoped_b = renderer.present()?;
        let diagnostics = renderer.drain_diagnostics();

        println!(
            "status=falsified; contract=ADR-0018-provider-neutral-lifecycle-candidate; target=native-wgpu; sequence=present-A>stage-B-all-families>late-failure>present-A>replace-B>reject-scoped-A>present-B>submit-unscoped-A>present-aliased-B>submit-scoped-B>present-B; A-draws={}; A-after-failure-draws={}; B-draws={}; unscoped-A-after-B-draws={}; scoped-B-draws={}; set-A={}; set-B={}; forced-failure={forced_failure:?}; commit={commit:?}; scoped-stale-A={stale_a:?}; unscoped-submit-bypass=true; provider-diagnostics={}; backend={}; device={}; adapter={}",
            presented_a.frame.draw_calls,
            preserved_a.frame.draw_calls,
            presented_b.frame.draw_calls,
            unscoped_a_after_b.frame.draw_calls,
            presented_scoped_b.frame.draw_calls,
            set_a.diagnostic_value(),
            set_b.diagnostic_value(),
            diagnostics.len(),
            renderer.backend_api(),
            renderer.device_kind(),
            renderer.adapter_name(),
        );
        Ok(())
    }
}

impl PlatformEventHandler for NativeResourceSetProbe {
    fn on_native_window_created(&mut self, window: Arc<NativeWindow>) -> PlatformResult<()> {
        let size = window.inner_size();
        self.renderer = Some(WgpuBackend::for_window(window, size.width, size.height)?);
        Ok(())
    }

    fn on_platform_event(&mut self, _event: PlatformInputEvent) -> PlatformResult<()> {
        Ok(())
    }

    fn on_frame(&mut self, _delta_seconds: f64) -> PlatformResult<FrameOutcome> {
        if !self.complete {
            self.run_probe()?;
            self.complete = true;
        }
        Ok(FrameOutcome::Exit)
    }
}

fn populate_backend(
    renderer: &mut WgpuBackend,
    scene: u8,
) -> Result<Vec<RenderCommand>, WgpuBackendError> {
    let pipeline = renderer.register_pipeline(&Pipeline::new(
        format!("native-resource-set-{scene}"),
        PipelineKind::LitColor3d,
    ))?;
    renderer.create_texture_rgba8(TEXTURE, texture_descriptor(), &texture_pixels(scene))?;
    renderer.upload_material(MATERIAL, &material(scene))?;
    renderer.upload_mesh(MESH, &Mesh::quad());
    renderer.upload_camera(CAMERA, Camera::default());
    renderer.set_active_camera(CAMERA);
    Ok(commands(scene, pipeline))
}

fn populate_stage(
    stage: &mut WgpuResourceSetStage,
    scene: u8,
) -> Result<Vec<RenderCommand>, WgpuBackendError> {
    let pipeline = stage.register_pipeline(&Pipeline::new(
        format!("native-resource-set-{scene}"),
        PipelineKind::LitColor3d,
    ))?;
    stage.create_texture_rgba8(TEXTURE, texture_descriptor(), &texture_pixels(scene))?;
    stage.upload_material(MATERIAL, &material(scene))?;
    stage.upload_mesh(MESH, &Mesh::quad());
    stage.upload_camera(CAMERA, Camera::default());
    stage.set_active_camera(CAMERA);
    Ok(commands(scene, pipeline))
}

fn material(scene: u8) -> Material {
    Material::new(
        format!("native-resource-set-material-{scene}"),
        Color::rgb(1.0, 1.0, 1.0),
    )
    .with_texture(TEXTURE)
}

fn texture_descriptor() -> Rgba8TextureDescriptor {
    Rgba8TextureDescriptor::new(2, 2, Rgba8TextureColorSpace::Srgb)
}

fn texture_pixels(scene: u8) -> [u8; 16] {
    let shade = if scene == 0 { 80 } else { 180 };
    [
        shade, 40, 200, 255, shade, 40, 200, 255, shade, 40, 200, 255, shade, 40, 200, 255,
    ]
}

fn commands(scene: u8, pipeline: tokimu_render::PipelineHandle) -> Vec<RenderCommand> {
    vec![
        RenderCommand::Clear(ClearCommand {
            color: if scene == 0 {
                Color::rgb(0.02, 0.03, 0.08)
            } else {
                Color::rgb(0.08, 0.02, 0.03)
            },
        }),
        RenderCommand::DrawMesh(DrawMeshCommand {
            mesh: MESH,
            material: MATERIAL,
            pipeline,
            instance: Instance2d::identity(),
            camera: Some(CAMERA),
            viewport: None,
        }),
    ]
}
