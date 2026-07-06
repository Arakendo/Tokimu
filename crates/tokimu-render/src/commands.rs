use crate::{Color, MeshHandle, TextureHandle};

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ClearCommand {
    pub color: Color,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct DrawMeshCommand {
    pub mesh: MeshHandle,
    pub texture: Option<TextureHandle>,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum RenderCommand {
    Clear(ClearCommand),
    DrawMesh(DrawMeshCommand),
}