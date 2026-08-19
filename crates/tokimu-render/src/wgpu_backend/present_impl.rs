use wgpu::util::DeviceExt;

use crate::{RenderFrameCpuTimings, RenderStats, Renderer};

use super::cpu_timer::CpuTimer;
use super::material_support::derived_material_key;
use super::{
    wgpu_camera_uniform, GpuCameraBinding, GpuInstanceBinding, GpuInstanceUniform, QueuedGeometry,
    WgpuBackend, WgpuBackendError,
};

impl WgpuBackend {
    pub fn present(&mut self) -> Result<RenderStats, WgpuBackendError> {
        if self.surface_state.is_none() {
            return Ok(self.end_frame());
        }
        self.prepare_derived_materials()?;
        let Some(surface_state) = self.surface_state.as_ref() else {
            return Ok(self.end_frame());
        };

        let surface_acquire_start = CpuTimer::start();
        let frame = surface_state
            .surface
            .get_current_texture()
            .map_err(|error| WgpuBackendError::SurfaceAcquire(error.to_string()))?;
        let surface_acquire_call = surface_acquire_start.map(CpuTimer::elapsed);

        let resource_preparation_start = CpuTimer::start();
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
            let uniform = wgpu_camera_uniform(camera);
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
        let resource_preparation = resource_preparation_start.map(CpuTimer::elapsed);

        let command_encoding_start = CpuTimer::start();
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
                    stencil_ops: Some(wgpu::Operations {
                        load: wgpu::LoadOp::Clear(0),
                        store: wgpu::StoreOp::Store,
                    }),
                }),
                occlusion_query_set: None,
                timestamp_writes: None,
            });

            if self.stats.has_frame_draws() {
                render_pass.set_stencil_reference(0);
                let mut active_pipeline = None;
                for (index, draw) in self.queued_draws.iter().enumerate() {
                    let gpu_mesh = match draw.geometry {
                        QueuedGeometry::Persistent(handle) => self
                            .meshes
                            .get(&handle)
                            .ok_or(WgpuBackendError::MissingMesh(handle.0))?,
                        #[cfg(feature = "experimental-submission-local-geometry")]
                        QueuedGeometry::SubmissionLocal(slot) => self
                            .submission_local_meshes
                            .get(slot)
                            .ok_or(WgpuBackendError::MissingSubmissionLocalGeometry(slot))?,
                    };
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
        let command_encoding = command_encoding_start.map(CpuTimer::elapsed);

        let queue_submit_start = CpuTimer::start();
        self._queue.submit(std::iter::once(command_buffer));
        let queue_submit_call = queue_submit_start.map(CpuTimer::elapsed);

        let surface_present_start = CpuTimer::start();
        frame.present();
        let surface_present_call = surface_present_start.map(CpuTimer::elapsed);

        self.stats.record_frame_cpu_timings(RenderFrameCpuTimings {
            surface_acquire_call,
            resource_preparation,
            command_encoding,
            queue_submit_call,
            surface_present_call,
            ..RenderFrameCpuTimings::default()
        });
        Ok(self.end_frame())
    }
}
