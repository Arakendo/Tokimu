use std::collections::HashMap;
#[cfg(not(target_arch = "wasm32"))]
use std::sync::Arc;

#[cfg(not(target_arch = "wasm32"))]
use raw_window_handle::{HasDisplayHandle, HasWindowHandle};
#[cfg(target_arch = "wasm32")]
use raw_window_handle::{WebCanvasWindowHandle, WebDisplayHandle};
#[cfg(target_arch = "wasm32")]
use web_sys::HtmlCanvasElement;
#[cfg(target_arch = "wasm32")]
use wgpu::SurfaceTargetUnsafe;

use crate::{Color, PipelineRegistry};

use super::diagnostics::install_backend_diagnostic_sink;
use super::pipeline_support::{
    create_camera_bind_group_layout, create_depth_texture, create_instance_bind_group_layout,
    create_material_bind_group_layout,
};
use super::{ProviderShared, SurfaceState, WgpuBackend, WgpuBackendError};

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
            #[cfg(feature = "experimental-submission-local-geometry")]
            submission_local_meshes: Vec::new(),
            materials: HashMap::new(),
            derived_materials: HashMap::new(),
            pipelines: HashMap::new(),
            pipeline_registry: PipelineRegistry::new(),
            renderables: HashMap::new(),
            textures: HashMap::new(),
            cameras: HashMap::new(),
            active_camera: crate::resources::CameraHandle::default(),
            _instance: ProviderShared::new(instance),
            _device: ProviderShared::new(device),
            _queue: ProviderShared::new(queue),
            adapter_info,
            surface_state: None,
            backend_diagnostic_messages,
            #[cfg(feature = "experimental-scene-resource-staging")]
            experimental_stage_context: None,
            #[cfg(feature = "experimental-scene-resource-staging")]
            experimental_resource_set_authority:
                crate::experimental_render_resource_set::ExperimentalRenderResourceSetAuthority::new(),
            #[cfg(feature = "experimental-scene-resource-staging")]
            experimental_current_resource_set:
                crate::experimental_render_resource_set::ExperimentalRenderResourceSetAuthority::initial_id(),
        })
    }

    #[cfg(not(target_arch = "wasm32"))]
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
            #[cfg(feature = "experimental-submission-local-geometry")]
            submission_local_meshes: Vec::new(),
            materials: HashMap::new(),
            derived_materials: HashMap::new(),
            pipelines: HashMap::new(),
            pipeline_registry: PipelineRegistry::new(),
            renderables: HashMap::new(),
            textures: HashMap::new(),
            cameras: HashMap::new(),
            active_camera: crate::resources::CameraHandle::default(),
            _instance: ProviderShared::new(instance),
            _device: ProviderShared::new(device),
            _queue: ProviderShared::new(queue),
            adapter_info,
            surface_state: Some(SurfaceState {
                surface,
                config,
                clear_color: Color::BLACK,
                depth_texture,
                depth_view,
                camera_bind_group_layout: ProviderShared::new(camera_bind_group_layout),
                material_bind_group_layout: ProviderShared::new(material_bind_group_layout),
                instance_bind_group_layout: ProviderShared::new(instance_bind_group_layout),
            }),
            backend_diagnostic_messages,
            #[cfg(feature = "experimental-scene-resource-staging")]
            experimental_stage_context: None,
            #[cfg(feature = "experimental-scene-resource-staging")]
            experimental_resource_set_authority:
                crate::experimental_render_resource_set::ExperimentalRenderResourceSetAuthority::new(),
            #[cfg(feature = "experimental-scene-resource-staging")]
            experimental_current_resource_set:
                crate::experimental_render_resource_set::ExperimentalRenderResourceSetAuthority::initial_id(),
        })
    }

    #[cfg(target_arch = "wasm32")]
    async fn create_for_canvas(
        canvas: HtmlCanvasElement,
        width: u32,
        height: u32,
    ) -> Result<Self, WgpuBackendError> {
        // Browser WebGPU availability is asynchronous. WGPU's own helper probes
        // it before fixing the instance backend set, which avoids treating a
        // synchronous `Instance::default()` construction as browser readiness.
        let instance =
            wgpu::util::new_instance_with_webgpu_detection(wgpu::InstanceDescriptor::default())
                .await;
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
            #[cfg(feature = "experimental-submission-local-geometry")]
            submission_local_meshes: Vec::new(),
            materials: HashMap::new(),
            derived_materials: HashMap::new(),
            pipelines: HashMap::new(),
            pipeline_registry: PipelineRegistry::new(),
            renderables: HashMap::new(),
            textures: HashMap::new(),
            cameras: HashMap::new(),
            active_camera: crate::resources::CameraHandle::default(),
            _instance: ProviderShared::new(instance),
            _device: ProviderShared::new(device),
            _queue: ProviderShared::new(queue),
            adapter_info,
            surface_state: Some(SurfaceState {
                surface,
                config,
                clear_color: Color::BLACK,
                depth_texture,
                depth_view,
                camera_bind_group_layout: ProviderShared::new(camera_bind_group_layout),
                material_bind_group_layout: ProviderShared::new(material_bind_group_layout),
                instance_bind_group_layout: ProviderShared::new(instance_bind_group_layout),
            }),
            backend_diagnostic_messages,
            #[cfg(feature = "experimental-scene-resource-staging")]
            experimental_stage_context: None,
            #[cfg(feature = "experimental-scene-resource-staging")]
            experimental_resource_set_authority:
                crate::experimental_render_resource_set::ExperimentalRenderResourceSetAuthority::new(),
            #[cfg(feature = "experimental-scene-resource-staging")]
            experimental_current_resource_set:
                crate::experimental_render_resource_set::ExperimentalRenderResourceSetAuthority::initial_id(),
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
}
