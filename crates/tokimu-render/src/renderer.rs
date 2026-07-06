use crate::{Color, RenderCommand};

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct RenderStats {
    pub draw_calls: u32,
}

pub trait Renderer {
    fn name(&self) -> &'static str;
    fn clear_color(&self) -> Color;
    fn begin_frame(&mut self);
    fn submit(&mut self, commands: &[RenderCommand]);
    fn end_frame(&mut self) -> RenderStats;
}
