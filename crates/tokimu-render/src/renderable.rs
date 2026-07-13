use crate::{MaterialHandle, MeshHandle, PipelineHandle};

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct Renderable {
    pub mesh: MeshHandle,
    pub material: MaterialHandle,
    pub pipeline: PipelineHandle,
}

impl Renderable {
    pub fn new(mesh: MeshHandle, material: MaterialHandle, pipeline: PipelineHandle) -> Self {
        Self {
            mesh,
            material,
            pipeline,
        }
    }
}
