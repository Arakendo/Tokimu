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
pub mod shader;
pub mod texture;
pub mod wgpu_backend;

pub use camera::Camera;
pub use color::Color;
pub use commands::{
    ClearCommand, DrawMeshCommand, DrawMeshMaterialOverrideCommand, DrawRenderableCommand,
    RenderCommand, ViewportRect,
};
pub use instance::Instance2d;
pub use material::{
    Material, MaterialDefinition, MaterialDefinitionId, MaterialFloatRange, MaterialInstance,
    MaterialModelError, MaterialOverride, MaterialParameterDeclaration, MaterialParameterKind,
    MaterialParameterValue, TextureAddressMode, TextureFilter, TextureSampler,
    MAX_MATERIAL_PARAMETERS,
};
pub use mesh::{Mesh, MeshValidationError};
pub use pipeline::{
    BlendMode, ColorWriteMask, CullMode, DepthTest, Pipeline, PipelineDrawContractError,
    PipelineKind, PipelineRegistry, PipelineRenderState, PipelineRenderStateError,
    PipelineValidationError,
};
pub use renderable::Renderable;
pub use renderer::{
    RenderFrameCpuTimings, RenderFrameStats, RenderLifetimeStats, RenderStats, Renderer,
};
pub use resources::{
    CameraHandle, MaterialHandle, MeshHandle, PipelineHandle, RenderableHandle, TextureHandle,
};
pub use shader::{
    ShaderBindingDeclaration, ShaderBindingSource, ShaderDiagnosticStage,
    ShaderMaterialCompatibilityError, ShaderMeshCompatibilityError, ShaderModuleDefinition,
    ShaderModuleValidationError, ShaderVertexInput, ShaderVertexSemantic, MAX_SHADER_BINDINGS,
    MAX_SHADER_SOURCE_BYTES, MAX_SHADER_VERTEX_INPUTS,
};
pub use texture::{
    Rgba8TextureColorSpace, Rgba8TextureDescriptor, Texture, TextureValidationError,
};
pub use wgpu_backend::{
    RenderTargetReplacement, RenderTargetResourceObservation, WgpuBackend, WgpuBackendError,
};
