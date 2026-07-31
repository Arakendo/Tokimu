use crate::{
    BlendMode, Camera, CameraHandle, Color, ColorWriteMask, CullMode, DepthTest, Instance2d,
    Material, MaterialHandle, MaterialOverride, Mesh, MeshHandle, Pipeline, PipelineHandle,
    PipelineKind, PipelineRegistry, PipelineRenderState, PipelineValidationError, RenderCommand,
    RenderFrameCpuTimings, RenderStats, Renderable, RenderableHandle, Renderer, Texture,
    TextureHandle,
};
use bytemuck::{Pod, Zeroable};
use raw_window_handle::{HasDisplayHandle, HasWindowHandle};
#[cfg(target_arch = "wasm32")]
use raw_window_handle::{WebCanvasWindowHandle, WebDisplayHandle};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::Instant;
use thiserror::Error;
#[cfg(target_arch = "wasm32")]
use web_sys::HtmlCanvasElement;
use wgpu::util::DeviceExt;
#[cfg(target_arch = "wasm32")]
use wgpu::SurfaceTargetUnsafe;

#[derive(Debug, Error)]
pub enum WgpuBackendError {
    #[error("failed to request a compatible GPU adapter")]
    AdapterRequest,
    #[error("failed to request a GPU device: {0}")]
    DeviceRequest(String),
    #[error("failed to create a render surface: {0}")]
    SurfaceCreation(String),
    #[error("surface did not report any supported texture formats")]
    SurfaceFormatUnavailable,
    #[error("failed to acquire the current surface texture: {0}")]
    SurfaceAcquire(String),
    #[error("mesh handle {0} has not been uploaded")]
    MissingMesh(u64),
    #[error("material handle {0} has not been uploaded")]
    MissingMaterial(u64),
    #[error("pipeline handle {0} has not been uploaded")]
    MissingPipeline(u64),
    #[error("renderable handle {0} has not been uploaded")]
    MissingRenderable(u64),
    #[error("texture handle {0} has not been uploaded")]
    MissingTexture(u64),
    #[error("invalid pipeline declaration: {0}")]
    InvalidPipeline(#[from] PipelineValidationError),
}

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
    _texture: wgpu::Texture,
    _view: Arc<wgpu::TextureView>,
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

fn install_backend_diagnostic_sink(device: &wgpu::Device) -> Arc<Mutex<Vec<String>>> {
    let messages = Arc::new(Mutex::new(Vec::new()));
    let sink = Arc::clone(&messages);
    device.on_uncaptured_error(Box::new(move |error| {
        let mut messages = match sink.lock() {
            Ok(messages) => messages,
            Err(poisoned) => poisoned.into_inner(),
        };
        messages.push(format!("WebGPU backend validation failed: {error}"));
    }));
    messages
}

fn drain_backend_diagnostic_messages(
    messages: &Mutex<Vec<String>>,
) -> Vec<tokimu_core::DiagnosticRecord> {
    let mut messages = match messages.lock() {
        Ok(messages) => messages,
        Err(poisoned) => poisoned.into_inner(),
    };
    std::mem::take(&mut *messages)
        .into_iter()
        .map(|message| {
            tokimu_core::DiagnosticRecord::error(
                tokimu_core::DiagnosticKind::BackendError,
                "tokimu-render.wgpu",
                message,
            )
        })
        .collect()
}

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
    pub fn new() -> Result<Self, WgpuBackendError> {
        pollster::block_on(Self::create())
    }

    #[cfg(not(target_arch = "wasm32"))]
    pub fn for_window<W>(window: Arc<W>, width: u32, height: u32) -> Result<Self, WgpuBackendError>
    where
        W: HasDisplayHandle + HasWindowHandle + Send + Sync + 'static,
    {
        pollster::block_on(Self::create_for_window(window, width, height))
    }

    #[cfg(target_arch = "wasm32")]
    pub async fn for_window(
        canvas: HtmlCanvasElement,
        width: u32,
        height: u32,
    ) -> Result<Self, WgpuBackendError> {
        Self::create_for_canvas(canvas, width, height).await
    }

    async fn create() -> Result<Self, WgpuBackendError> {
        let instance = wgpu::Instance::default();
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions::default())
            .await
            .ok_or(WgpuBackendError::AdapterRequest)?;
        let adapter_info = adapter.get_info();
        let (device, queue) = adapter
            .request_device(&wgpu::DeviceDescriptor::default(), None)
            .await
            .map_err(|error| WgpuBackendError::DeviceRequest(error.to_string()))?;
        let backend_diagnostic_messages = install_backend_diagnostic_sink(&device);

        Ok(Self {
            stats: crate::renderer::RenderStatsTracker::default(),
            queued_draws: Vec::new(),
            instance_bindings: Vec::new(),
            camera_bindings: HashMap::new(),
            meshes: HashMap::new(),
            materials: HashMap::new(),
            derived_materials: HashMap::new(),
            pipelines: HashMap::new(),
            pipeline_registry: PipelineRegistry::new(),
            renderables: HashMap::new(),
            textures: HashMap::new(),
            cameras: HashMap::new(),
            active_camera: crate::resources::CameraHandle::default(),
            _instance: instance,
            _device: device,
            _queue: queue,
            adapter_info,
            surface_state: None,
            backend_diagnostic_messages,
        })
    }

    async fn create_for_window<W>(
        window: Arc<W>,
        width: u32,
        height: u32,
    ) -> Result<Self, WgpuBackendError>
    where
        W: HasDisplayHandle + HasWindowHandle + Send + Sync + 'static,
    {
        let instance = wgpu::Instance::default();
        let surface = instance
            .create_surface(window)
            .map_err(|error| WgpuBackendError::SurfaceCreation(error.to_string()))?;
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                compatible_surface: Some(&surface),
                ..Default::default()
            })
            .await
            .ok_or(WgpuBackendError::AdapterRequest)?;
        let adapter_info = adapter.get_info();
        let (device, queue) = adapter
            .request_device(&wgpu::DeviceDescriptor::default(), None)
            .await
            .map_err(|error| WgpuBackendError::DeviceRequest(error.to_string()))?;
        let backend_diagnostic_messages = install_backend_diagnostic_sink(&device);
        let capabilities = surface.get_capabilities(&adapter);
        let format = capabilities
            .formats
            .iter()
            .copied()
            .find(wgpu::TextureFormat::is_srgb)
            .or_else(|| capabilities.formats.first().copied())
            .ok_or(WgpuBackendError::SurfaceFormatUnavailable)?;
        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format,
            width: width.max(1),
            height: height.max(1),
            present_mode: capabilities.present_modes[0],
            alpha_mode: capabilities.alpha_modes[0],
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
        };
        let camera_bind_group_layout = create_camera_bind_group_layout(&device);
        let material_bind_group_layout = create_material_bind_group_layout(&device);
        let instance_bind_group_layout = create_instance_bind_group_layout(&device);
        let (depth_texture, depth_view) = create_depth_texture(&device, width, height);
        surface.configure(&device, &config);

        Ok(Self {
            stats: crate::renderer::RenderStatsTracker::default(),
            queued_draws: Vec::new(),
            instance_bindings: Vec::new(),
            camera_bindings: HashMap::new(),
            meshes: HashMap::new(),
            materials: HashMap::new(),
            derived_materials: HashMap::new(),
            pipelines: HashMap::new(),
            pipeline_registry: PipelineRegistry::new(),
            renderables: HashMap::new(),
            textures: HashMap::new(),
            cameras: HashMap::new(),
            active_camera: crate::resources::CameraHandle::default(),
            _instance: instance,
            _device: device,
            _queue: queue,
            adapter_info,
            surface_state: Some(SurfaceState {
                surface,
                config,
                clear_color: Color::BLACK,
                depth_texture,
                depth_view,
                camera_bind_group_layout,
                material_bind_group_layout,
                instance_bind_group_layout,
            }),
            backend_diagnostic_messages,
        })
    }

    #[cfg(target_arch = "wasm32")]
    async fn create_for_canvas(
        canvas: HtmlCanvasElement,
        width: u32,
        height: u32,
    ) -> Result<Self, WgpuBackendError> {
        let instance = wgpu::Instance::default();
        let value: &wasm_bindgen::JsValue = &canvas;
        let obj = std::ptr::NonNull::from(value).cast();
        let raw_window_handle = WebCanvasWindowHandle::new(obj).into();
        let raw_display_handle = WebDisplayHandle::new().into();
        let surface = unsafe {
            instance.create_surface_unsafe(SurfaceTargetUnsafe::RawHandle {
                raw_display_handle,
                raw_window_handle,
            })
        }
        .map_err(|error| WgpuBackendError::SurfaceCreation(error.to_string()))?;
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                compatible_surface: Some(&surface),
                ..Default::default()
            })
            .await
            .ok_or(WgpuBackendError::AdapterRequest)?;
        let adapter_info = adapter.get_info();
        let (device, queue) = adapter
            .request_device(&wgpu::DeviceDescriptor::default(), None)
            .await
            .map_err(|error| WgpuBackendError::DeviceRequest(error.to_string()))?;
        let backend_diagnostic_messages = install_backend_diagnostic_sink(&device);
        let capabilities = surface.get_capabilities(&adapter);
        let format = capabilities
            .formats
            .iter()
            .copied()
            .find(wgpu::TextureFormat::is_srgb)
            .or_else(|| capabilities.formats.first().copied())
            .ok_or(WgpuBackendError::SurfaceFormatUnavailable)?;
        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format,
            width: width.max(1),
            height: height.max(1),
            present_mode: capabilities.present_modes[0],
            alpha_mode: capabilities.alpha_modes[0],
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
        };
        let camera_bind_group_layout = create_camera_bind_group_layout(&device);
        let material_bind_group_layout = create_material_bind_group_layout(&device);
        let instance_bind_group_layout = create_instance_bind_group_layout(&device);
        let (depth_texture, depth_view) = create_depth_texture(&device, width, height);
        surface.configure(&device, &config);

        Ok(Self {
            stats: crate::renderer::RenderStatsTracker::default(),
            queued_draws: Vec::new(),
            instance_bindings: Vec::new(),
            camera_bindings: HashMap::new(),
            meshes: HashMap::new(),
            materials: HashMap::new(),
            derived_materials: HashMap::new(),
            pipelines: HashMap::new(),
            pipeline_registry: PipelineRegistry::new(),
            renderables: HashMap::new(),
            textures: HashMap::new(),
            cameras: HashMap::new(),
            active_camera: crate::resources::CameraHandle::default(),
            _instance: instance,
            _device: device,
            _queue: queue,
            adapter_info,
            surface_state: Some(SurfaceState {
                surface,
                config,
                clear_color: Color::BLACK,
                depth_texture,
                depth_view,
                camera_bind_group_layout,
                material_bind_group_layout,
                instance_bind_group_layout,
            }),
            backend_diagnostic_messages,
        })
    }

    pub fn adapter_name(&self) -> &str {
        &self.adapter_info.name
    }

    pub fn backend_api(&self) -> &'static str {
        match self.adapter_info.backend {
            wgpu::Backend::Vulkan => "vulkan",
            wgpu::Backend::Metal => "metal",
            wgpu::Backend::Dx12 => "dx12",
            wgpu::Backend::Gl => "gl",
            wgpu::Backend::BrowserWebGpu => "browser-webgpu",
            _ => "unknown",
        }
    }

    pub fn device_kind(&self) -> &'static str {
        match self.adapter_info.device_type {
            wgpu::DeviceType::Other => "other",
            wgpu::DeviceType::IntegratedGpu => "integrated-gpu",
            wgpu::DeviceType::DiscreteGpu => "discrete-gpu",
            wgpu::DeviceType::VirtualGpu => "virtual-gpu",
            wgpu::DeviceType::Cpu => "cpu",
        }
    }

    pub fn resize_surface(&mut self, width: u32, height: u32) {
        let Some(surface_state) = self.surface_state.as_mut() else {
            return;
        };

        if width == 0 || height == 0 {
            return;
        }

        surface_state.config.width = width;
        surface_state.config.height = height;
        let (depth_texture, depth_view) = create_depth_texture(&self._device, width, height);
        surface_state.depth_texture = depth_texture;
        surface_state.depth_view = depth_view;
        surface_state
            .surface
            .configure(&self._device, &surface_state.config);
    }

    pub fn upload_mesh(&mut self, handle: MeshHandle, mesh: &Mesh) {
        let replaced_existing = self.meshes.contains_key(&handle);
        let vertices: Vec<GpuVertex> = mesh
            .positions
            .iter()
            .copied()
            .zip(mesh.normals.iter().copied())
            .map(|(position, normal)| GpuVertex { position, normal })
            .collect();
        let vertex_buffer = self
            ._device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("tokimu-mesh-vertex-buffer"),
                contents: bytemuck::cast_slice(&vertices),
                usage: wgpu::BufferUsages::VERTEX,
            });

        self.meshes.insert(
            handle,
            GpuMesh {
                vertex_buffer,
                vertex_count: mesh.vertex_count(),
            },
        );
        self.stats.record_mesh_upload(replaced_existing);
    }

    /// Uploads an RGBA8 image for future texture-backed pipelines.
    pub fn upload_texture(&mut self, handle: TextureHandle, texture: &Texture) {
        let gpu_texture = self._device.create_texture(&wgpu::TextureDescriptor {
            label: Some("tokimu-texture"),
            size: wgpu::Extent3d {
                width: texture.width,
                height: texture.height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8UnormSrgb,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        });
        self._queue.write_texture(
            wgpu::ImageCopyTexture {
                texture: &gpu_texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            &texture.rgba8,
            wgpu::ImageDataLayout {
                offset: 0,
                bytes_per_row: Some(4 * texture.width),
                rows_per_image: Some(texture.height),
            },
            wgpu::Extent3d {
                width: texture.width,
                height: texture.height,
                depth_or_array_layers: 1,
            },
        );
        let view = Arc::new(gpu_texture.create_view(&wgpu::TextureViewDescriptor::default()));
        self.textures.insert(
            handle,
            GpuTexture {
                _texture: gpu_texture,
                _view: view,
            },
        );
    }

    pub fn upload_material(
        &mut self,
        handle: MaterialHandle,
        material: &Material,
    ) -> Result<(), WgpuBackendError> {
        let Some(surface_state) = self.surface_state.as_ref() else {
            return Ok(());
        };

        let uniform = [
            material.base_color.r,
            material.base_color.g,
            material.base_color.b,
            material.base_color.a,
        ];
        let uniform_buffer = self
            ._device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("tokimu-material-uniform-buffer"),
                contents: bytemuck::cast_slice(&uniform),
                usage: wgpu::BufferUsages::UNIFORM,
            });
        let (fallback_texture, fallback_view, fallback_sampler, texture_view, sampler) =
            if let Some(texture_handle) = material.texture {
                if let Some(texture) = self.textures.get(&texture_handle) {
                    (
                        None,
                        None,
                        None,
                        Arc::clone(&texture._view),
                        Arc::new(
                            self._device
                                .create_sampler(&wgpu::SamplerDescriptor::default()),
                        ),
                    )
                } else {
                    return Err(WgpuBackendError::MissingTexture(texture_handle.0));
                }
            } else {
                let texture = self._device.create_texture(&wgpu::TextureDescriptor {
                    label: Some("tokimu-material-fallback-texture"),
                    size: wgpu::Extent3d {
                        width: 1,
                        height: 1,
                        depth_or_array_layers: 1,
                    },
                    mip_level_count: 1,
                    sample_count: 1,
                    dimension: wgpu::TextureDimension::D2,
                    format: wgpu::TextureFormat::Rgba8UnormSrgb,
                    usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
                    view_formats: &[],
                });
                self._queue.write_texture(
                    wgpu::ImageCopyTexture {
                        texture: &texture,
                        mip_level: 0,
                        origin: wgpu::Origin3d::ZERO,
                        aspect: wgpu::TextureAspect::All,
                    },
                    &[255, 255, 255, 255],
                    wgpu::ImageDataLayout {
                        offset: 0,
                        bytes_per_row: Some(4),
                        rows_per_image: Some(1),
                    },
                    wgpu::Extent3d {
                        width: 1,
                        height: 1,
                        depth_or_array_layers: 1,
                    },
                );
                let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
                let sampler = self
                    ._device
                    .create_sampler(&wgpu::SamplerDescriptor::default());
                let view = Arc::new(view);
                let sampler = Arc::new(sampler);
                (
                    Some(texture),
                    Some(Arc::clone(&view)),
                    Some(Arc::clone(&sampler)),
                    view,
                    sampler,
                )
            };
        let bind_group = self._device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("tokimu-material-bind-group"),
            layout: &surface_state.material_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: uniform_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::TextureView(&texture_view),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: wgpu::BindingResource::Sampler(&sampler),
                },
            ],
        });

        self.materials.insert(
            handle,
            GpuMaterial {
                base_color: material.base_color,
                _uniform_buffer: uniform_buffer,
                bind_group,
                texture_view,
                sampler,
                _fallback_texture: fallback_texture,
                _fallback_view: fallback_view,
                _fallback_sampler: fallback_sampler,
            },
        );
        self.derived_materials.retain(|key, _| key.source != handle);

        Ok(())
    }

    fn prepare_derived_materials(&mut self) -> Result<(), WgpuBackendError> {
        let requests = self
            .queued_draws
            .iter()
            .filter_map(|draw| {
                draw.material_override
                    .map(|override_value| (draw.material, override_value))
            })
            .collect::<Vec<_>>();
        let material_bind_group_layout = &self
            .surface_state
            .as_ref()
            .expect("derived materials require an initialized surface")
            .material_bind_group_layout;
        let device = &self._device;
        let materials = &self.materials;
        let derived_materials = &mut self.derived_materials;
        let stats = &mut self.stats;

        for (source_handle, override_value) in requests {
            let key = derived_material_key(source_handle, override_value);
            if derived_materials.contains_key(&key) {
                stats.record_derived_material_cache_hit();
                continue;
            }

            let source = materials
                .get(&source_handle)
                .ok_or(WgpuBackendError::MissingMaterial(source_handle.0))?;
            let material =
                create_derived_material(device, material_bind_group_layout, source, override_value);
            derived_materials.insert(key, material);
            stats.record_binding_allocation();
            stats.record_derived_material_cache_miss();
        }

        Ok(())
    }

    pub fn upload_pipeline(
        &mut self,
        handle: PipelineHandle,
        pipeline: &Pipeline,
    ) -> Result<(), WgpuBackendError> {
        if let Err(error) = pipeline.validate() {
            self.record_backend_diagnostic(format!(
                "pipeline `{}` declaration was rejected before backend compilation: {error}",
                pipeline.label
            ));
            return Err(error.into());
        }
        let Some(surface_state) = self.surface_state.as_ref() else {
            return Ok(());
        };
        let shader_label = pipeline.backend_shader_label();

        let compiled = match pipeline.kind {
            PipelineKind::SolidColor2d => create_solid_color_pipeline(
                &self._device,
                surface_state.config.format,
                DEPTH_FORMAT,
                &surface_state.material_bind_group_layout,
                &surface_state.instance_bind_group_layout,
                &surface_state.camera_bind_group_layout,
                pipeline.render_state,
            ),
            PipelineKind::Texture2d => create_custom_pipeline(
                &self._device,
                surface_state.config.format,
                DEPTH_FORMAT,
                &surface_state.material_bind_group_layout,
                &surface_state.instance_bind_group_layout,
                &surface_state.camera_bind_group_layout,
                &pipeline.label,
                &shader_label,
                pipeline
                    .shader_source
                    .as_deref()
                    .or_else(|| pipeline.kind.default_shader_source())
                    .unwrap(),
                &pipeline.vertex_entry_point,
                &pipeline.fragment_entry_point,
                pipeline.render_state,
            ),
            PipelineKind::LitColor3d => create_custom_pipeline(
                &self._device,
                surface_state.config.format,
                DEPTH_FORMAT,
                &surface_state.material_bind_group_layout,
                &surface_state.instance_bind_group_layout,
                &surface_state.camera_bind_group_layout,
                &pipeline.label,
                &shader_label,
                pipeline
                    .shader_source
                    .as_deref()
                    .or_else(|| pipeline.kind.default_shader_source())
                    .unwrap_or(Pipeline::default_2d_shader_source()),
                &pipeline.vertex_entry_point,
                &pipeline.fragment_entry_point,
                pipeline.render_state,
            ),
            PipelineKind::CustomWgsl2d => create_custom_pipeline(
                &self._device,
                surface_state.config.format,
                DEPTH_FORMAT,
                &surface_state.material_bind_group_layout,
                &surface_state.instance_bind_group_layout,
                &surface_state.camera_bind_group_layout,
                &pipeline.label,
                &shader_label,
                pipeline
                    .shader_source
                    .as_deref()
                    .expect("validated custom WGSL pipelines always contain shader source"),
                &pipeline.vertex_entry_point,
                &pipeline.fragment_entry_point,
                pipeline.render_state,
            ),
        };

        let replaced_existing = self.pipelines.contains_key(&handle);
        self.pipeline_registry
            .register_with_handle(handle, pipeline);
        self.pipelines.insert(handle, compiled);
        self.stats.record_pipeline_creation(replaced_existing);
        Ok(())
    }

    pub fn register_pipeline(
        &mut self,
        pipeline: &Pipeline,
    ) -> Result<PipelineHandle, WgpuBackendError> {
        if let Err(error) = pipeline.validate() {
            self.record_backend_diagnostic(format!(
                "pipeline `{}` declaration was rejected before backend compilation: {error}",
                pipeline.label
            ));
            return Err(error.into());
        }
        let handle = self.pipeline_registry.register(pipeline);
        self.upload_pipeline(handle, pipeline)?;
        Ok(handle)
    }

    /// Drains renderer-adapter diagnostics without exposing backend-native error types.
    ///
    /// WebGPU shader and pipeline validation can be reported after a synchronous
    /// pipeline creation call returns. The backend records those messages in its
    /// error callback and presents them here as Tokimu diagnostics for callers to
    /// route alongside their own application diagnostics.
    pub fn drain_diagnostics(&self) -> Vec<tokimu_core::DiagnosticRecord> {
        drain_backend_diagnostic_messages(&self.backend_diagnostic_messages)
    }

    /// Flushes backend work and callbacks before diagnostics are inspected.
    ///
    /// Native WebGPU validation may be reported asynchronously after resource
    /// creation returns. Presentation diagnostics use this bounded adapter hook
    /// instead of exposing `wgpu::Device` to callers.
    pub fn poll_diagnostics(&self) {
        let _ = self._device.poll(wgpu::Maintain::Wait);
    }

    fn record_backend_diagnostic(&self, message: impl Into<String>) {
        let mut messages = match self.backend_diagnostic_messages.lock() {
            Ok(messages) => messages,
            Err(poisoned) => poisoned.into_inner(),
        };
        messages.push(message.into());
    }

    pub fn pipeline_handle(&self, label: &str) -> Option<PipelineHandle> {
        self.pipeline_registry.handle_for_label(label)
    }

    pub fn upload_renderable(&mut self, handle: RenderableHandle, renderable: Renderable) {
        self.renderables.insert(handle, renderable);
    }

    pub fn upload_camera(&mut self, handle: crate::resources::CameraHandle, camera: Camera) {
        self.cameras.insert(handle, camera);
    }

    pub fn set_active_camera(&mut self, handle: crate::resources::CameraHandle) {
        self.active_camera = handle;
    }

    pub fn present(&mut self) -> Result<RenderStats, WgpuBackendError> {
        if self.surface_state.is_none() {
            return Ok(self.end_frame());
        }
        self.prepare_derived_materials()?;
        let Some(surface_state) = self.surface_state.as_ref() else {
            return Ok(self.end_frame());
        };

        let surface_acquire_start = Instant::now();
        let frame = surface_state
            .surface
            .get_current_texture()
            .map_err(|error| WgpuBackendError::SurfaceAcquire(error.to_string()))?;
        let surface_acquire_call = surface_acquire_start.elapsed();

        let resource_preparation_start = Instant::now();
        let view = frame
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());
        let mut encoder = self
            ._device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("tokimu-clear-pass"),
            });
        for (index, draw) in self.queued_draws.iter().enumerate() {
            let (rotation_sin, rotation_cos) = draw.instance.rotation.sin_cos();
            let uniform = GpuInstanceUniform {
                translation: draw.instance.translation,
                scale: draw.instance.scale,
                rotation: [rotation_sin, rotation_cos],
                _padding: [0.0, 0.0],
            };

            if index == self.instance_bindings.len() {
                let buffer = self
                    ._device
                    .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                        label: Some("tokimu-instance-uniform-buffer"),
                        contents: bytemuck::bytes_of(&uniform),
                        usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
                    });
                let bind_group = self._device.create_bind_group(&wgpu::BindGroupDescriptor {
                    label: Some("tokimu-instance-bind-group"),
                    layout: &surface_state.instance_bind_group_layout,
                    entries: &[wgpu::BindGroupEntry {
                        binding: 0,
                        resource: buffer.as_entire_binding(),
                    }],
                });
                self.instance_bindings.push(GpuInstanceBinding {
                    uniform,
                    _buffer: buffer,
                    bind_group,
                });
                self.stats.record_binding_allocation();
            } else if self.instance_bindings[index].uniform != uniform {
                let binding = &mut self.instance_bindings[index];
                self._queue
                    .write_buffer(&binding._buffer, 0, bytemuck::bytes_of(&uniform));
                binding.uniform = uniform;
                self.stats.record_uniform_buffer_write();
            }
        }

        let camera_handles = self
            .queued_draws
            .iter()
            .map(|draw| draw.camera.unwrap_or(self.active_camera))
            .collect::<std::collections::HashSet<_>>();
        for camera_handle in camera_handles {
            let camera = self
                .cameras
                .get(&camera_handle)
                .copied()
                .unwrap_or_default();
            let uniform = GpuCameraUniform {
                view_projection: (camera.projection * camera.view).to_cols_array_2d(),
            };
            if let Some(binding) = self.camera_bindings.get_mut(&camera_handle) {
                if binding.uniform != uniform {
                    self._queue
                        .write_buffer(&binding._buffer, 0, bytemuck::bytes_of(&uniform));
                    binding.uniform = uniform;
                    self.stats.record_uniform_buffer_write();
                }
            } else {
                let buffer = self
                    ._device
                    .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                        label: Some("tokimu-camera-uniform-buffer"),
                        contents: bytemuck::bytes_of(&uniform),
                        usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
                    });
                let bind_group = self._device.create_bind_group(&wgpu::BindGroupDescriptor {
                    label: Some("tokimu-camera-bind-group"),
                    layout: &surface_state.camera_bind_group_layout,
                    entries: &[wgpu::BindGroupEntry {
                        binding: 0,
                        resource: buffer.as_entire_binding(),
                    }],
                });
                self.camera_bindings.insert(
                    camera_handle,
                    GpuCameraBinding {
                        uniform,
                        _buffer: buffer,
                        bind_group,
                    },
                );
                self.stats.record_binding_allocation();
            }
        }
        let resource_preparation = resource_preparation_start.elapsed();

        let command_encoding_start = Instant::now();
        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("tokimu-clear-pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: surface_state.clear_color.r as f64,
                            g: surface_state.clear_color.g as f64,
                            b: surface_state.clear_color.b as f64,
                            a: surface_state.clear_color.a as f64,
                        }),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                    view: &surface_state.depth_view,
                    depth_ops: Some(wgpu::Operations {
                        load: wgpu::LoadOp::Clear(1.0),
                        store: wgpu::StoreOp::Store,
                    }),
                    stencil_ops: None,
                }),
                occlusion_query_set: None,
                timestamp_writes: None,
            });

            if self.stats.has_frame_draws() {
                let mut active_pipeline = None;
                for (index, draw) in self.queued_draws.iter().enumerate() {
                    let gpu_mesh = self
                        .meshes
                        .get(&draw.mesh)
                        .ok_or(WgpuBackendError::MissingMesh(draw.mesh.0))?;
                    let gpu_material = match draw.material_override {
                        Some(override_value) => self
                            .derived_materials
                            .get(&derived_material_key(draw.material, override_value))
                            .expect("derived material binding prepared before render pass"),
                        None => self
                            .materials
                            .get(&draw.material)
                            .ok_or(WgpuBackendError::MissingMaterial(draw.material.0))?,
                    };
                    self.stats.record_material_resolution();
                    if gpu_material.base_color.a < 1.0 {
                        self.stats.record_transparent_draw();
                    }
                    let pipeline = self
                        .pipelines
                        .get(&draw.pipeline)
                        .ok_or(WgpuBackendError::MissingPipeline(draw.pipeline.0))?;
                    let camera_handle = draw.camera.unwrap_or(self.active_camera);
                    let camera_bind_group = &self
                        .camera_bindings
                        .get(&camera_handle)
                        .expect("camera binding prepared before render pass")
                        .bind_group;
                    if active_pipeline != Some(draw.pipeline) {
                        self.stats.record_pipeline_switch();
                        active_pipeline = Some(draw.pipeline);
                    }
                    if let Some(viewport) = draw.viewport {
                        render_pass.set_scissor_rect(
                            viewport.x.max(0.0) as u32,
                            viewport.y.max(0.0) as u32,
                            viewport.width.max(0.0) as u32,
                            viewport.height.max(0.0) as u32,
                        );
                    } else {
                        render_pass.set_scissor_rect(
                            0,
                            0,
                            surface_state.config.width,
                            surface_state.config.height,
                        );
                    }
                    render_pass.set_pipeline(pipeline);
                    render_pass.set_bind_group(2, camera_bind_group, &[]);
                    render_pass.set_bind_group(0, &gpu_material.bind_group, &[]);
                    render_pass.set_bind_group(1, &self.instance_bindings[index].bind_group, &[]);
                    render_pass.set_vertex_buffer(0, gpu_mesh.vertex_buffer.slice(..));
                    render_pass.draw(0..gpu_mesh.vertex_count, 0..1);
                }
            }
        }
        let command_buffer = encoder.finish();
        let command_encoding = command_encoding_start.elapsed();

        let queue_submit_start = Instant::now();
        self._queue.submit(std::iter::once(command_buffer));
        let queue_submit_call = queue_submit_start.elapsed();

        let surface_present_start = Instant::now();
        frame.present();
        let surface_present_call = surface_present_start.elapsed();

        self.stats.record_frame_cpu_timings(RenderFrameCpuTimings {
            surface_acquire_call: Some(surface_acquire_call),
            resource_preparation: Some(resource_preparation),
            command_encoding: Some(command_encoding),
            queue_submit_call: Some(queue_submit_call),
            surface_present_call: Some(surface_present_call),
        });
        Ok(self.end_frame())
    }
}

impl Renderer for WgpuBackend {
    fn name(&self) -> &'static str {
        "wgpu"
    }

    fn clear_color(&self) -> Color {
        Color::BLACK
    }

    fn begin_frame(&mut self) {
        self.stats.begin_frame();
        self.queued_draws.clear();
    }

    fn submit(&mut self, commands: &[RenderCommand]) {
        self.stats.record_submit_call();
        if let Some(clear_color) = commands.iter().find_map(|command| match command {
            RenderCommand::Clear(clear) => Some(clear.color),
            RenderCommand::DrawMesh(_) => None,
            RenderCommand::DrawMeshMaterialOverride(_) => None,
            RenderCommand::DrawRenderable(_) => None,
        }) {
            if let Some(surface_state) = self.surface_state.as_mut() {
                surface_state.clear_color = clear_color;
            }
        }

        self.queued_draws
            .extend(commands.iter().filter_map(|command| match command {
                RenderCommand::Clear(_) => None,
                RenderCommand::DrawMesh(draw) => Some(QueuedDraw {
                    mesh: draw.mesh,
                    material: draw.material,
                    pipeline: draw.pipeline,
                    instance: draw.instance,
                    camera: draw.camera,
                    viewport: draw.viewport,
                    material_override: None,
                }),
                RenderCommand::DrawMeshMaterialOverride(draw) => Some(QueuedDraw {
                    mesh: draw.draw.mesh,
                    material: draw.draw.material,
                    pipeline: draw.draw.pipeline,
                    instance: draw.draw.instance,
                    camera: draw.draw.camera,
                    viewport: draw.draw.viewport,
                    material_override: Some(draw.material_override),
                }),
                RenderCommand::DrawRenderable(draw) => {
                    let renderable = self.renderables.get(&draw.renderable)?;
                    Some(QueuedDraw {
                        mesh: renderable.mesh,
                        material: renderable.material,
                        pipeline: renderable.pipeline,
                        instance: draw.instance,
                        camera: draw.camera,
                        viewport: draw.viewport,
                        material_override: None,
                    })
                }
            }));

        self.stats.record_draw_calls(
            commands
                .iter()
                .filter(|command| {
                    matches!(
                        command,
                        RenderCommand::DrawMesh(_)
                            | RenderCommand::DrawMeshMaterialOverride(_)
                            | RenderCommand::DrawRenderable(_)
                    )
                })
                .count() as u32,
        );
    }

    fn end_frame(&mut self) -> RenderStats {
        self.stats.snapshot()
    }
}

fn derived_material_key(
    source: MaterialHandle,
    override_value: MaterialOverride,
) -> DerivedMaterialKey {
    DerivedMaterialKey {
        source,
        replacement_color: override_value.replacement_color().map(color_bits),
        opacity_multiplier: override_value.opacity_multiplier().to_bits(),
    }
}

fn color_bits(color: Color) -> [u32; 4] {
    [
        color.r.to_bits(),
        color.g.to_bits(),
        color.b.to_bits(),
        color.a.to_bits(),
    ]
}

fn create_derived_material(
    device: &wgpu::Device,
    material_bind_group_layout: &wgpu::BindGroupLayout,
    source: &GpuMaterial,
    override_value: MaterialOverride,
) -> GpuMaterial {
    let base_color = override_value.apply_to_color(source.base_color);
    let uniform = [base_color.r, base_color.g, base_color.b, base_color.a];
    let uniform_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("tokimu-derived-material-uniform-buffer"),
        contents: bytemuck::cast_slice(&uniform),
        usage: wgpu::BufferUsages::UNIFORM,
    });
    let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: Some("tokimu-derived-material-bind-group"),
        layout: material_bind_group_layout,
        entries: &[
            wgpu::BindGroupEntry {
                binding: 0,
                resource: uniform_buffer.as_entire_binding(),
            },
            wgpu::BindGroupEntry {
                binding: 1,
                resource: wgpu::BindingResource::TextureView(&source.texture_view),
            },
            wgpu::BindGroupEntry {
                binding: 2,
                resource: wgpu::BindingResource::Sampler(&source.sampler),
            },
        ],
    });

    GpuMaterial {
        base_color,
        _uniform_buffer: uniform_buffer,
        bind_group,
        texture_view: Arc::clone(&source.texture_view),
        sampler: Arc::clone(&source.sampler),
        _fallback_texture: None,
        _fallback_view: None,
        _fallback_sampler: None,
    }
}

fn create_solid_color_pipeline(
    device: &wgpu::Device,
    surface_format: wgpu::TextureFormat,
    depth_format: wgpu::TextureFormat,
    material_bind_group_layout: &wgpu::BindGroupLayout,
    instance_bind_group_layout: &wgpu::BindGroupLayout,
    camera_bind_group_layout: &wgpu::BindGroupLayout,
    render_state: PipelineRenderState,
) -> wgpu::RenderPipeline {
    create_custom_pipeline(
        device,
        surface_format,
        depth_format,
        material_bind_group_layout,
        instance_bind_group_layout,
        camera_bind_group_layout,
        "tokimu-solid-color-pipeline",
        "tokimu-solid-color-shader",
        PipelineKind::SolidColor2d.default_shader_source().unwrap(),
        "vs_main",
        "fs_main",
        render_state,
    )
}

#[allow(clippy::too_many_arguments)]
fn create_custom_pipeline(
    device: &wgpu::Device,
    surface_format: wgpu::TextureFormat,
    depth_format: wgpu::TextureFormat,
    material_bind_group_layout: &wgpu::BindGroupLayout,
    instance_bind_group_layout: &wgpu::BindGroupLayout,
    camera_bind_group_layout: &wgpu::BindGroupLayout,
    pipeline_label: &str,
    shader_label: &str,
    shader_source: &str,
    vertex_entry_point: &str,
    fragment_entry_point: &str,
    render_state: PipelineRenderState,
) -> wgpu::RenderPipeline {
    let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some(shader_label),
        source: wgpu::ShaderSource::Wgsl(shader_source.into()),
    });

    let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: Some(pipeline_label),
        bind_group_layouts: &[
            material_bind_group_layout,
            instance_bind_group_layout,
            camera_bind_group_layout,
        ],
        push_constant_ranges: &[],
    });

    device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: Some(pipeline_label),
        layout: Some(&pipeline_layout),
        vertex: wgpu::VertexState {
            module: &shader,
            entry_point: Some(vertex_entry_point),
            buffers: &[wgpu::VertexBufferLayout {
                array_stride: std::mem::size_of::<GpuVertex>() as u64,
                step_mode: wgpu::VertexStepMode::Vertex,
                attributes: &[
                    wgpu::VertexAttribute {
                        format: wgpu::VertexFormat::Float32x3,
                        offset: 0,
                        shader_location: 0,
                    },
                    wgpu::VertexAttribute {
                        format: wgpu::VertexFormat::Float32x3,
                        offset: std::mem::size_of::<[f32; 3]>() as u64,
                        shader_location: 1,
                    },
                ],
            }],
            compilation_options: wgpu::PipelineCompilationOptions::default(),
        },
        primitive: wgpu::PrimitiveState {
            cull_mode: match render_state.cull_mode {
                CullMode::None => None,
                CullMode::Front => Some(wgpu::Face::Front),
                CullMode::Back => Some(wgpu::Face::Back),
            },
            ..wgpu::PrimitiveState::default()
        },
        depth_stencil: depth_stencil_state(depth_format, render_state),
        multisample: wgpu::MultisampleState::default(),
        fragment: Some(wgpu::FragmentState {
            module: &shader,
            entry_point: Some(fragment_entry_point),
            compilation_options: wgpu::PipelineCompilationOptions::default(),
            targets: &[Some(wgpu::ColorTargetState {
                format: surface_format,
                blend: match render_state.blend {
                    BlendMode::Opaque => None,
                    BlendMode::AlphaBlend => Some(wgpu::BlendState::ALPHA_BLENDING),
                },
                write_mask: color_write_mask(render_state.color_write),
            })],
        }),
        multiview: None,
        cache: None,
    })
}

fn depth_stencil_state(
    depth_format: wgpu::TextureFormat,
    render_state: PipelineRenderState,
) -> Option<wgpu::DepthStencilState> {
    match render_state.depth_test {
        DepthTest::Disabled => None,
        DepthTest::LessEqual => Some(wgpu::DepthStencilState {
            format: depth_format,
            depth_write_enabled: render_state.depth_write,
            depth_compare: wgpu::CompareFunction::LessEqual,
            stencil: wgpu::StencilState::default(),
            bias: wgpu::DepthBiasState::default(),
        }),
    }
}

fn color_write_mask(mask: ColorWriteMask) -> wgpu::ColorWrites {
    let mut writes = wgpu::ColorWrites::empty();
    if mask.red {
        writes |= wgpu::ColorWrites::RED;
    }
    if mask.green {
        writes |= wgpu::ColorWrites::GREEN;
    }
    if mask.blue {
        writes |= wgpu::ColorWrites::BLUE;
    }
    if mask.alpha {
        writes |= wgpu::ColorWrites::ALPHA;
    }
    writes
}

fn create_depth_texture(
    device: &wgpu::Device,
    width: u32,
    height: u32,
) -> (wgpu::Texture, wgpu::TextureView) {
    let texture = device.create_texture(&wgpu::TextureDescriptor {
        label: Some("tokimu-depth-texture"),
        size: wgpu::Extent3d {
            width: width.max(1),
            height: height.max(1),
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: DEPTH_FORMAT,
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
        view_formats: &[],
    });
    let view = texture.create_view(&wgpu::TextureViewDescriptor::default());

    (texture, view)
}

fn create_instance_bind_group_layout(device: &wgpu::Device) -> wgpu::BindGroupLayout {
    device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        label: Some("tokimu-instance-bind-group-layout"),
        entries: &[wgpu::BindGroupLayoutEntry {
            binding: 0,
            visibility: wgpu::ShaderStages::VERTEX,
            ty: wgpu::BindingType::Buffer {
                ty: wgpu::BufferBindingType::Uniform,
                has_dynamic_offset: false,
                min_binding_size: None,
            },
            count: None,
        }],
    })
}

fn create_camera_bind_group_layout(device: &wgpu::Device) -> wgpu::BindGroupLayout {
    device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        label: Some("tokimu-camera-bind-group-layout"),
        entries: &[wgpu::BindGroupLayoutEntry {
            binding: 0,
            visibility: wgpu::ShaderStages::VERTEX,
            ty: wgpu::BindingType::Buffer {
                ty: wgpu::BufferBindingType::Uniform,
                has_dynamic_offset: false,
                min_binding_size: None,
            },
            count: None,
        }],
    })
}

fn create_material_bind_group_layout(device: &wgpu::Device) -> wgpu::BindGroupLayout {
    device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        label: Some("tokimu-material-bind-group-layout"),
        entries: &[
            wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            },
            wgpu::BindGroupLayoutEntry {
                binding: 1,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Texture {
                    sample_type: wgpu::TextureSampleType::Float { filterable: true },
                    view_dimension: wgpu::TextureViewDimension::D2,
                    multisampled: false,
                },
                count: None,
            },
            wgpu::BindGroupLayoutEntry {
                binding: 2,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                count: None,
            },
        ],
    })
}

#[cfg(test)]
mod tests {
    use super::{derived_material_key, drain_backend_diagnostic_messages};
    use crate::{Color, MaterialHandle, MaterialOverride};
    use std::sync::Mutex;

    #[test]
    fn backend_diagnostic_sink_drains_into_tokimu_records() {
        let messages = Mutex::new(vec!["shader validation failed".to_owned()]);

        let diagnostics = drain_backend_diagnostic_messages(&messages);

        assert_eq!(diagnostics.len(), 1);
        assert_eq!(
            diagnostics[0].kind,
            tokimu_core::DiagnosticKind::BackendError
        );
        assert_eq!(
            diagnostics[0].severity,
            tokimu_core::DiagnosticSeverity::Error
        );
        assert_eq!(diagnostics[0].source, "tokimu-render.wgpu");
        assert_eq!(diagnostics[0].message, "shader validation failed");
        assert!(drain_backend_diagnostic_messages(&messages).is_empty());
    }

    #[test]
    fn derived_material_keys_reuse_identical_overrides_and_split_distinct_ones() {
        let source = MaterialHandle(12);
        let selected = MaterialOverride::with_replacement_color(Color::rgb(1.0, 0.5, 0.0)).unwrap();
        let faded = MaterialOverride::default()
            .with_opacity_multiplier(0.5)
            .unwrap();

        assert_eq!(
            derived_material_key(source, selected),
            derived_material_key(source, selected)
        );
        assert_ne!(
            derived_material_key(source, selected),
            derived_material_key(source, faded)
        );
    }
}
