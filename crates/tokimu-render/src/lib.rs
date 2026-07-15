pub mod camera;
pub mod color;
pub mod commands;
pub mod instance;
pub mod material;
pub mod mesh;
pub mod pipeline;
pub mod renderable;
pub mod renderer;
pub mod resources;
pub mod texture;
pub mod wgpu_backend;

pub use camera::Camera;
pub use color::Color;
pub use commands::{
    ClearCommand, DrawMeshCommand, DrawRenderableCommand, RenderCommand, ViewportRect,
};
pub use instance::Instance2d;
pub use material::Material;
pub use mesh::Mesh;
pub use pipeline::{Pipeline, PipelineKind, PipelineRegistry};
pub use renderable::Renderable;
pub use renderer::{RenderStats, Renderer};
pub use resources::{
    CameraHandle, MaterialHandle, MeshHandle, PipelineHandle, RenderableHandle, TextureHandle,
};
pub use texture::Texture;
pub use wgpu_backend::{WgpuBackend, WgpuBackendError};
