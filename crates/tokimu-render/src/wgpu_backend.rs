use crate::{
    Camera, CameraHandle, Color, Instance2d, Material, MaterialHandle, Mesh, MeshHandle, Pipeline,
    PipelineHandle, PipelineKind, PipelineRegistry, RenderCommand, RenderStats, Renderable,
    RenderableHandle, Renderer,
};
use bytemuck::{Pod, Zeroable};
use raw_window_handle::{HasDisplayHandle, HasWindowHandle};
use std::collections::HashMap;
use std::sync::Arc;
use thiserror::Error;
use wgpu::util::DeviceExt;

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
    _uniform_buffer: wgpu::Buffer,
    bind_group: wgpu::BindGroup,
}

#[derive(Clone, Copy)]
struct QueuedDraw {
    mesh: MeshHandle,
    material: MaterialHandle,
    pipeline: PipelineHandle,
    instance: Instance2d,
    camera: Option<CameraHandle>,
    viewport: Option<crate::commands::ViewportRect>,
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
#[derive(Clone, Copy, Debug, Pod, Zeroable)]
struct GpuInstanceUniform {
    translation: [f32; 2],
    scale: [f32; 2],
    rotation: [f32; 2],
    _padding: [f32; 2],
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Pod, Zeroable)]
struct GpuCameraUniform {
    view_projection: [[f32; 4]; 4],
}

const DEPTH_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Depth32Float;

pub struct WgpuBackend {
    draw_calls: u32,
    queued_draws: Vec<QueuedDraw>,
    meshes: HashMap<MeshHandle, GpuMesh>,
    materials: HashMap<MaterialHandle, GpuMaterial>,
    pipelines: HashMap<PipelineHandle, wgpu::RenderPipeline>,
    pipeline_registry: PipelineRegistry,
    renderables: HashMap<RenderableHandle, Renderable>,
    cameras: HashMap<crate::resources::CameraHandle, Camera>,
    active_camera: crate::resources::CameraHandle,
    _instance: wgpu::Instance,
    _device: wgpu::Device,
    _queue: wgpu::Queue,
    adapter_info: wgpu::AdapterInfo,
    surface_state: Option<SurfaceState>,
}

impl WgpuBackend {
    pub fn new() -> Result<Self, WgpuBackendError> {
        pollster::block_on(Self::create())
    }

    pub fn for_window<W>(window: Arc<W>, width: u32, height: u32) -> Result<Self, WgpuBackendError>
    where
        W: HasDisplayHandle + HasWindowHandle + Send + Sync + 'static,
    {
        pollster::block_on(Self::create_for_window(window, width, height))
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

        Ok(Self {
            draw_calls: 0,
            queued_draws: Vec::new(),
            meshes: HashMap::new(),
            materials: HashMap::new(),
            pipelines: HashMap::new(),
            pipeline_registry: PipelineRegistry::new(),
            renderables: HashMap::new(),
            cameras: HashMap::new(),
            active_camera: crate::resources::CameraHandle::default(),
            _instance: instance,
            _device: device,
            _queue: queue,
            adapter_info,
            surface_state: None,
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
            draw_calls: 0,
            queued_draws: Vec::new(),
            meshes: HashMap::new(),
            materials: HashMap::new(),
            pipelines: HashMap::new(),
            pipeline_registry: PipelineRegistry::new(),
            renderables: HashMap::new(),
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
        let bind_group = self._device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("tokimu-material-bind-group"),
            layout: &surface_state.material_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: uniform_buffer.as_entire_binding(),
            }],
        });

        self.materials.insert(
            handle,
            GpuMaterial {
                _uniform_buffer: uniform_buffer,
                bind_group,
            },
        );

        Ok(())
    }

    pub fn upload_pipeline(
        &mut self,
        handle: PipelineHandle,
        pipeline: &Pipeline,
    ) -> Result<(), WgpuBackendError> {
        let Some(surface_state) = self.surface_state.as_ref() else {
            return Ok(());
        };

        let compiled = match pipeline.kind {
            PipelineKind::SolidColor2d => create_solid_color_pipeline(
                &self._device,
                surface_state.config.format,
                DEPTH_FORMAT,
                &surface_state.material_bind_group_layout,
                &surface_state.instance_bind_group_layout,
                &surface_state.camera_bind_group_layout,
            ),
            PipelineKind::LitColor3d => create_custom_pipeline(
                &self._device,
                surface_state.config.format,
                DEPTH_FORMAT,
                &surface_state.material_bind_group_layout,
                &surface_state.instance_bind_group_layout,
                &surface_state.camera_bind_group_layout,
                &pipeline.label,
                pipeline
                    .shader_source
                    .as_deref()
                    .or_else(|| pipeline.kind.default_shader_source())
                    .unwrap_or(Pipeline::default_2d_shader_source()),
                &pipeline.vertex_entry_point,
                &pipeline.fragment_entry_point,
            ),
            PipelineKind::CustomWgsl2d => create_custom_pipeline(
                &self._device,
                surface_state.config.format,
                DEPTH_FORMAT,
                &surface_state.material_bind_group_layout,
                &surface_state.instance_bind_group_layout,
                &surface_state.camera_bind_group_layout,
                &pipeline.label,
                pipeline
                    .shader_source
                    .as_deref()
                    .or_else(|| pipeline.kind.default_shader_source())
                    .unwrap_or(Pipeline::default_2d_shader_source()),
                &pipeline.vertex_entry_point,
                &pipeline.fragment_entry_point,
            ),
        };

        self.pipeline_registry
            .register_with_handle(handle, pipeline);
        self.pipeline_registry
            .register_with_handle(handle, pipeline);
        self.pipeline_registry
            .register_with_handle(handle, pipeline);
        self.pipeline_registry
            .register_with_handle(handle, pipeline);
        self.pipeline_registry
            .register_with_handle(handle, pipeline);
        self.pipeline_registry
            .register_with_handle(handle, pipeline);
        self.pipeline_registry
            .register_with_handle(handle, pipeline);
        self.pipeline_registry
            .register_with_handle(handle, pipeline);
        self.pipeline_registry
            .register_with_handle(handle, pipeline);
        self.pipeline_registry
            .register_with_handle(handle, pipeline);
        self.pipeline_registry
            .register_with_handle(handle, pipeline);
        self.pipelines.insert(handle, compiled);
        Ok(())
    }

    pub fn register_pipeline(
        &mut self,
        pipeline: &Pipeline,
    ) -> Result<PipelineHandle, WgpuBackendError> {
        let handle = self.pipeline_registry.register(pipeline);
        self.upload_pipeline(handle, pipeline)?;
        Ok(handle)
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
        let stats = self.end_frame();
        let Some(surface_state) = self.surface_state.as_ref() else {
            return Ok(stats);
        };

        let frame = surface_state
            .surface
            .get_current_texture()
            .map_err(|error| WgpuBackendError::SurfaceAcquire(error.to_string()))?;
        let view = frame
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());
        let mut encoder = self
            ._device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("tokimu-clear-pass"),
            });
        let mut instance_buffers = Vec::with_capacity(self.queued_draws.len());
        let mut instance_bind_groups = Vec::with_capacity(self.queued_draws.len());

        for draw in &self.queued_draws {
            let (rotation_sin, rotation_cos) = draw.instance.rotation.sin_cos();
            let uniform = GpuInstanceUniform {
                translation: draw.instance.translation,
                scale: draw.instance.scale,
                rotation: [rotation_sin, rotation_cos],
                _padding: [0.0, 0.0],
            };
            let buffer = self
                ._device
                .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: Some("tokimu-instance-uniform-buffer"),
                    contents: bytemuck::bytes_of(&uniform),
                    usage: wgpu::BufferUsages::UNIFORM,
                });
            let bind_group = self._device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("tokimu-instance-bind-group"),
                layout: &surface_state.instance_bind_group_layout,
                entries: &[wgpu::BindGroupEntry {
                    binding: 0,
                    resource: buffer.as_entire_binding(),
                }],
            });
            instance_buffers.push(buffer);
            instance_bind_groups.push(bind_group);
        }

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

            if stats.draw_calls > 0 {
                for (index, draw) in self.queued_draws.iter().enumerate() {
                    let gpu_mesh = self
                        .meshes
                        .get(&draw.mesh)
                        .ok_or(WgpuBackendError::MissingMesh(draw.mesh.0))?;
                    let gpu_material = self
                        .materials
                        .get(&draw.material)
                        .ok_or(WgpuBackendError::MissingMaterial(draw.material.0))?;
                    let pipeline = self
                        .pipelines
                        .get(&draw.pipeline)
                        .ok_or(WgpuBackendError::MissingPipeline(draw.pipeline.0))?;
                    let camera_handle = draw.camera.unwrap_or(self.active_camera);
                    let camera = self
                        .cameras
                        .get(&camera_handle)
                        .copied()
                        .unwrap_or_default();
                    let camera_uniform = GpuCameraUniform {
                        view_projection: (camera.projection * camera.view).to_cols_array_2d(),
                    };
                    let camera_buffer =
                        self._device
                            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                                label: Some("tokimu-camera-uniform-buffer"),
                                contents: bytemuck::bytes_of(&camera_uniform),
                                usage: wgpu::BufferUsages::UNIFORM,
                            });
                    let camera_bind_group =
                        self._device.create_bind_group(&wgpu::BindGroupDescriptor {
                            label: Some("tokimu-camera-bind-group"),
                            layout: &surface_state.camera_bind_group_layout,
                            entries: &[wgpu::BindGroupEntry {
                                binding: 0,
                                resource: camera_buffer.as_entire_binding(),
                            }],
                        });
                    if let Some(viewport) = draw.viewport {
                        render_pass.set_viewport(
                            viewport.x,
                            viewport.y,
                            viewport.width,
                            viewport.height,
                            0.0,
                            1.0,
                        );
                    }
                    render_pass.set_pipeline(pipeline);
                    render_pass.set_bind_group(2, &camera_bind_group, &[]);
                    render_pass.set_bind_group(0, &gpu_material.bind_group, &[]);
                    render_pass.set_bind_group(1, &instance_bind_groups[index], &[]);
                    render_pass.set_vertex_buffer(0, gpu_mesh.vertex_buffer.slice(..));
                    render_pass.draw(0..gpu_mesh.vertex_count, 0..1);
                }
            }
        }

        drop(instance_buffers);

        self._queue.submit(std::iter::once(encoder.finish()));
        frame.present();
        Ok(stats)
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
        self.draw_calls = 0;
        self.queued_draws.clear();
    }

    fn submit(&mut self, commands: &[RenderCommand]) {
        if let Some(clear_color) = commands.iter().find_map(|command| match command {
            RenderCommand::Clear(clear) => Some(clear.color),
            RenderCommand::DrawMesh(_) => None,
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
                    })
                }
            }));

        self.draw_calls += commands
            .iter()
            .filter(|command| {
                matches!(
                    command,
                    RenderCommand::DrawMesh(_) | RenderCommand::DrawRenderable(_)
                )
            })
            .count() as u32;
    }

    fn end_frame(&mut self) -> RenderStats {
        RenderStats {
            draw_calls: self.draw_calls,
        }
    }
}

fn create_solid_color_pipeline(
    device: &wgpu::Device,
    surface_format: wgpu::TextureFormat,
    depth_format: wgpu::TextureFormat,
    material_bind_group_layout: &wgpu::BindGroupLayout,
    instance_bind_group_layout: &wgpu::BindGroupLayout,
    camera_bind_group_layout: &wgpu::BindGroupLayout,
) -> wgpu::RenderPipeline {
    create_custom_pipeline(
        device,
        surface_format,
        depth_format,
        material_bind_group_layout,
        instance_bind_group_layout,
        camera_bind_group_layout,
        "tokimu-solid-color-pipeline",
        PipelineKind::SolidColor2d.default_shader_source().unwrap(),
        "vs_main",
        "fs_main",
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
    shader_source: &str,
    vertex_entry_point: &str,
    fragment_entry_point: &str,
) -> wgpu::RenderPipeline {
    let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some(pipeline_label),
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
        primitive: wgpu::PrimitiveState::default(),
        depth_stencil: Some(wgpu::DepthStencilState {
            format: depth_format,
            depth_write_enabled: true,
            depth_compare: wgpu::CompareFunction::LessEqual,
            stencil: wgpu::StencilState::default(),
            bias: wgpu::DepthBiasState::default(),
        }),
        multisample: wgpu::MultisampleState::default(),
        fragment: Some(wgpu::FragmentState {
            module: &shader,
            entry_point: Some(fragment_entry_point),
            compilation_options: wgpu::PipelineCompilationOptions::default(),
            targets: &[Some(wgpu::ColorTargetState {
                format: surface_format,
                blend: Some(wgpu::BlendState::REPLACE),
                write_mask: wgpu::ColorWrites::ALL,
            })],
        }),
        multiview: None,
        cache: None,
    })
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
        entries: &[wgpu::BindGroupLayoutEntry {
            binding: 0,
            visibility: wgpu::ShaderStages::FRAGMENT,
            ty: wgpu::BindingType::Buffer {
                ty: wgpu::BufferBindingType::Uniform,
                has_dynamic_offset: false,
                min_binding_size: None,
            },
            count: None,
        }],
    })
}
