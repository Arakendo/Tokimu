pub mod camera;
pub mod commands;
pub mod color;
pub mod instance;
pub mod material;
pub mod mesh;
pub mod pipeline;
pub mod renderable;
pub mod resources;
pub mod renderer;
pub mod texture;
pub mod wgpu_backend;

pub use camera::Camera;
pub use commands::{
	ClearCommand, DrawMeshCommand, DrawRenderableCommand, RenderCommand, ViewportRect,
};
pub use color::Color;
pub use instance::Instance2d;
pub use material::Material;
pub use mesh::Mesh;
pub use pipeline::{Pipeline, PipelineKind, PipelineRegistry};
pub use renderable::Renderable;
pub use resources::{CameraHandle, MaterialHandle, MeshHandle, PipelineHandle, RenderableHandle, TextureHandle};
pub use renderer::{RenderStats, Renderer};
pub use texture::Texture;
pub use wgpu_backend::{WgpuBackend, WgpuBackendError};
