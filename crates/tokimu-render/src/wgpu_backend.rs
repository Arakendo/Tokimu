mod backend_init;
mod diagnostics;
mod error;
mod material_resources;
mod material_support;
mod mesh_resources;
mod pipeline_registry_impl;
mod pipeline_support;
mod present_impl;
mod renderer_impl;
mod texture_resources;
mod texture_support;

use crate::{
    Camera, CameraHandle, Color, Instance2d, MaterialHandle, MaterialOverride, MeshHandle,
    PipelineHandle, PipelineRegistry, Renderable, RenderableHandle, Rgba8TextureDescriptor,
    TextureHandle,
};
use bytemuck::{Pod, Zeroable};
pub use error::WgpuBackendError;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

#[repr(C)]
#[derive(Clone, Copy, Debug, Pod, Zeroable)]
struct GpuVertex {
    position: [f32; 3],
    normal: [f32; 3],
}

struct GpuMesh {
    vertex_buffer: wgpu::Buffer,
    vertex_count: u32,
}

struct GpuMaterial {
    base_color: Color,
    _uniform_buffer: wgpu::Buffer,
    bind_group: wgpu::BindGroup,
    texture_view: Arc<wgpu::TextureView>,
    sampler: Arc<wgpu::Sampler>,
    _fallback_texture: Option<wgpu::Texture>,
    _fallback_view: Option<Arc<wgpu::TextureView>>,
    _fallback_sampler: Option<Arc<wgpu::Sampler>>,
}

struct GpuTexture {
    texture: wgpu::Texture,
    view: Arc<wgpu::TextureView>,
    descriptor: Rgba8TextureDescriptor,
    role: GpuTextureRole,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum GpuTextureRole {
    Source,
    RenderTarget,
}

#[derive(Clone, Copy)]
struct QueuedDraw {
    mesh: MeshHandle,
    material: MaterialHandle,
    pipeline: PipelineHandle,
    instance: Instance2d,
    camera: Option<CameraHandle>,
    viewport: Option<crate::commands::ViewportRect>,
    material_override: Option<MaterialOverride>,
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
struct DerivedMaterialKey {
    source: MaterialHandle,
    replacement_color: Option<[u32; 4]>,
    opacity_multiplier: u32,
}

struct SurfaceState {
    surface: wgpu::Surface<'static>,
    config: wgpu::SurfaceConfiguration,
    clear_color: Color,
    depth_texture: wgpu::Texture,
    depth_view: wgpu::TextureView,
    camera_bind_group_layout: wgpu::BindGroupLayout,
    material_bind_group_layout: wgpu::BindGroupLayout,
    instance_bind_group_layout: wgpu::BindGroupLayout,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Pod, Zeroable)]
struct GpuInstanceUniform {
    translation: [f32; 2],
    scale: [f32; 2],
    rotation: [f32; 2],
    _padding: [f32; 2],
}

#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Pod, Zeroable)]
struct GpuCameraUniform {
    view_projection: [[f32; 4]; 4],
}

struct GpuInstanceBinding {
    uniform: GpuInstanceUniform,
    _buffer: wgpu::Buffer,
    bind_group: wgpu::BindGroup,
}

struct GpuCameraBinding {
    uniform: GpuCameraUniform,
    _buffer: wgpu::Buffer,
    bind_group: wgpu::BindGroup,
}

const DEPTH_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Depth32Float;

pub struct WgpuBackend {
    stats: crate::renderer::RenderStatsTracker,
    queued_draws: Vec<QueuedDraw>,
    instance_bindings: Vec<GpuInstanceBinding>,
    camera_bindings: HashMap<CameraHandle, GpuCameraBinding>,
    meshes: HashMap<MeshHandle, GpuMesh>,
    materials: HashMap<MaterialHandle, GpuMaterial>,
    derived_materials: HashMap<DerivedMaterialKey, GpuMaterial>,
    pipelines: HashMap<PipelineHandle, wgpu::RenderPipeline>,
    pipeline_registry: PipelineRegistry,
    renderables: HashMap<RenderableHandle, Renderable>,
    textures: HashMap<TextureHandle, GpuTexture>,
    cameras: HashMap<crate::resources::CameraHandle, Camera>,
    active_camera: crate::resources::CameraHandle,
    _instance: wgpu::Instance,
    _device: wgpu::Device,
    _queue: wgpu::Queue,
    adapter_info: wgpu::AdapterInfo,
    surface_state: Option<SurfaceState>,
    backend_diagnostic_messages: Arc<Mutex<Vec<String>>>,
}

impl WgpuBackend {
    pub fn upload_renderable(&mut self, handle: RenderableHandle, renderable: Renderable) {
        self.renderables.insert(handle, renderable);
    }

    pub fn upload_camera(&mut self, handle: crate::resources::CameraHandle, camera: Camera) {
        self.cameras.insert(handle, camera);
    }

    pub fn set_active_camera(&mut self, handle: crate::resources::CameraHandle) {
        self.active_camera = handle;
    }
}

#[cfg(test)]
mod tests;
