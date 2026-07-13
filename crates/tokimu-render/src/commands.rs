use crate::{Color, Instance2d, MaterialHandle, MeshHandle, PipelineHandle, RenderableHandle};

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ClearCommand {
    pub color: Color,
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct DrawMeshCommand {
    pub mesh: MeshHandle,
    pub material: MaterialHandle,
    pub pipeline: PipelineHandle,
    pub instance: Instance2d,
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct DrawRenderableCommand {
    pub renderable: RenderableHandle,
    pub instance: Instance2d,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum RenderCommand {
    Clear(ClearCommand),
    DrawMesh(DrawMeshCommand),
    DrawRenderable(DrawRenderableCommand),
}