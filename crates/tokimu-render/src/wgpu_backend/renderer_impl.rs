use crate::{Color, RenderCommand, RenderStats, Renderer};

use super::{QueuedDraw, WgpuBackend};

impl Renderer for WgpuBackend {
    fn name(&self) -> &'static str {
        "wgpu"
    }

    fn clear_color(&self) -> Color {
        Color::BLACK
    }

    fn begin_frame(&mut self) {
        self.stats.begin_frame();
        self.queued_draws.clear();
    }

    fn submit(&mut self, commands: &[RenderCommand]) {
        self.stats.record_submit_call();
        if let Some(clear_color) = commands.iter().find_map(|command| match command {
            RenderCommand::Clear(clear) => Some(clear.color),
            RenderCommand::DrawMesh(_) => None,
            RenderCommand::DrawMeshMaterialOverride(_) => None,
            RenderCommand::DrawRenderable(_) => None,
        }) {
            if let Some(surface_state) = self.surface_state.as_mut() {
                surface_state.clear_color = clear_color;
            }
        }

        self.queued_draws
            .extend(commands.iter().filter_map(|command| match command {
                RenderCommand::Clear(_) => None,
                RenderCommand::DrawMesh(draw) => Some(QueuedDraw {
                    mesh: draw.mesh,
                    material: draw.material,
                    pipeline: draw.pipeline,
                    instance: draw.instance,
                    camera: draw.camera,
                    viewport: draw.viewport,
                    material_override: None,
                }),
                RenderCommand::DrawMeshMaterialOverride(draw) => Some(QueuedDraw {
                    mesh: draw.draw.mesh,
                    material: draw.draw.material,
                    pipeline: draw.draw.pipeline,
                    instance: draw.draw.instance,
                    camera: draw.draw.camera,
                    viewport: draw.draw.viewport,
                    material_override: Some(draw.material_override),
                }),
                RenderCommand::DrawRenderable(draw) => {
                    let renderable = self.renderables.get(&draw.renderable)?;
                    Some(QueuedDraw {
                        mesh: renderable.mesh,
                        material: renderable.material,
                        pipeline: renderable.pipeline,
                        instance: draw.instance,
                        camera: draw.camera,
                        viewport: draw.viewport,
                        material_override: None,
                    })
                }
            }));

        self.stats.record_draw_calls(
            commands
                .iter()
                .filter(|command| {
                    matches!(
                        command,
                        RenderCommand::DrawMesh(_)
                            | RenderCommand::DrawMeshMaterialOverride(_)
                            | RenderCommand::DrawRenderable(_)
                    )
                })
                .count() as u32,
        );
    }

    fn end_frame(&mut self) -> RenderStats {
        self.stats.snapshot()
    }
}
