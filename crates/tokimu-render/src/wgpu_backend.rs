mod backend_init;
mod cpu_timer;
mod diagnostics;
mod error;
mod material_resources;
mod material_support;
mod mesh_resources;
mod pipeline_registry_impl;
mod pipeline_support;
mod present_impl;
mod render_target_passes;
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
use tokimu_core::math::{Mat4, Vec4};

/// Reports the renderer-private consequences of replacing a render target.
///
/// Materials counted by `materials_requiring_rebind` still reference the old
/// texture view until their caller uploads them again. This makes target
/// replacement explicit rather than silently retaining stale bind groups.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct RenderTargetReplacement {
    pub width: u32,
    pub height: u32,
    pub materials_requiring_rebind: u32,
    pub invalidated_derived_materials: u32,
}

/// A backend-local snapshot of renderer-owned offscreen target storage.
///
/// The estimates count the RGBA8 color image and matching `Depth32Float`
/// image allocated for each live render target. They deliberately exclude
/// driver allocation overhead, surface buffers, samplers, views, mip levels,
/// caches, and GPU residency. This is diagnostic evidence, not a portable
/// memory budget or resource-management policy.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct RenderTargetResourceObservation {
    /// Number of live renderer-owned offscreen targets.
    pub target_count: u32,
    /// Total color-image pixels across those targets.
    pub color_pixels: u64,
    /// Estimated bytes occupied by RGBA8 color images.
    pub estimated_color_bytes: u64,
    /// Estimated bytes occupied by matching `Depth32Float` images.
    pub estimated_depth_bytes: u64,
    /// Sum of the color and depth image estimates.
    pub estimated_total_bytes: u64,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Pod, Zeroable)]
struct GpuVertex {
    position: [f32; 3],
    normal: [f32; 3],
    texture_coordinates: [f32; 2],
}

struct GpuMesh {
    vertex_buffer: wgpu::Buffer,
    vertex_count: u32,
}

struct GpuMaterial {
    base_color: Color,
    texture: Option<TextureHandle>,
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
    _depth_texture: Option<wgpu::Texture>,
    depth_view: Option<Arc<wgpu::TextureView>>,
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

/// Converts Tokimu's GL-style `[-1, 1]` clip depth to WGPU's `[0, 1]` depth.
///
/// Camera projection matrices remain Tokimu-owned values. The WGPU provider
/// adapts those values only while constructing its private GPU uniform.
fn wgpu_camera_uniform(camera: Camera) -> GpuCameraUniform {
    let tokimu_to_wgpu_clip = Mat4::from_cols(
        Vec4::X,
        Vec4::Y,
        Vec4::new(0.0, 0.0, 0.5, 0.0),
        Vec4::new(0.0, 0.0, 0.5, 1.0),
    );
    GpuCameraUniform {
        view_projection: (tokimu_to_wgpu_clip * camera.projection * camera.view).to_cols_array_2d(),
    }
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
