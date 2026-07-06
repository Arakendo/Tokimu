use crate::{Color, RenderCommand, RenderStats, Renderer};

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct WgpuBackend {
    draw_calls: u32,
}

impl Renderer for WgpuBackend {
    fn name(&self) -> &'static str {
        "wgpu-placeholder"
    }

    fn clear_color(&self) -> Color {
        Color::BLACK
    }

    fn begin_frame(&mut self) {
        self.draw_calls = 0;
    }

    fn submit(&mut self, commands: &[RenderCommand]) {
        self.draw_calls += commands
            .iter()
            .filter(|command| matches!(command, RenderCommand::DrawMesh(_)))
            .count() as u32;
    }

    fn end_frame(&mut self) -> RenderStats {
        RenderStats {
            draw_calls: self.draw_calls,
        }
    }
}
