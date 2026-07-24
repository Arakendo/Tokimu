use crate::{Color, RenderCommand};

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct RenderStats {
    pub draw_calls: u32,
    /// Number of mesh uploads performed since the renderer was created.
    pub mesh_uploads: u32,
    /// Number of uploads that replaced an existing mesh handle.
    pub mesh_replacements: u32,
}

pub trait Renderer {
    fn name(&self) -> &'static str;
    fn clear_color(&self) -> Color;
    fn begin_frame(&mut self);
    fn submit(&mut self, commands: &[RenderCommand]);
    fn end_frame(&mut self) -> RenderStats;
}
