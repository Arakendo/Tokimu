use std::sync::Arc;

use wgpu::util::DeviceExt;

use crate::{CameraHandle, Color, DrawMeshCommand, TextureHandle};

use super::cpu_timer::CpuTimer;
use super::texture_support::rgba8_texture_format;
use super::{
    GpuCameraBinding, GpuCameraUniform, GpuInstanceUniform, GpuTextureRole, WgpuBackend,
    WgpuBackendError,
};

impl WgpuBackend {
    /// Draws a bounded command slice into an existing renderer-owned target.
    ///
    /// This is deliberately a WGPU backend proof, not a `Renderer` trait
    /// feature. It establishes the execution seam used by the audio visualizer
    /// corpus while feedback scheduling and provider-neutral pass APIs remain
    /// under evaluation.
    pub fn draw_meshes_to_render_target(
        &mut self,
        target: TextureHandle,
        clear_color: Color,
        draws: &[DrawMeshCommand],
    ) -> Result<(), WgpuBackendError> {
        let surface_format = self
            .surface_state
            .as_ref()
            .map(|surface| surface.config.format)
            .unwrap_or(wgpu::TextureFormat::Rgba8UnormSrgb);

        for draw in draws {
            self.prepare_render_target_camera(draw.camera.unwrap_or(self.active_camera))?;
        }

        let (target_view, depth_view, target_format) = {
            let texture = self
                .textures
                .get(&target)
                .ok_or(WgpuBackendError::MissingTexture(target.0))?;
            if texture.role != GpuTextureRole::RenderTarget {
                return Err(WgpuBackendError::TextureIsNotRenderTarget(target.0));
            }
            (
                Arc::clone(&texture.view),
                texture
                    .depth_view
                    .as_ref()
                    .expect("render targets allocate a matching depth view")
                    .clone(),
                rgba8_texture_format(texture.descriptor.color_space),
            )
        };
        if target_format != surface_format {
            return Err(WgpuBackendError::RenderTargetFormatMismatch {
                target: target.0,
                target_format: format!("{target_format:?}"),
                surface_format: format!("{surface_format:?}"),
            });
        }

        for draw in draws {
            let material = self
                .materials
                .get(&draw.material)
                .ok_or(WgpuBackendError::MissingMaterial(draw.material.0))?;
            if material.texture == Some(target) {
                return Err(WgpuBackendError::RenderTargetSelfSampling {
                    target: target.0,
                    material: draw.material.0,
                });
            }
            if !self.meshes.contains_key(&draw.mesh) {
                return Err(WgpuBackendError::MissingMesh(draw.mesh.0));
            }
            if !self.pipelines.contains_key(&draw.pipeline) {
                return Err(WgpuBackendError::MissingPipeline(draw.pipeline.0));
            }
        }

        if self.surface_state.is_none() {
            // Headless targets can be allocated, but this first execution proof
            // deliberately reuses the native window pipeline layouts.
            return Ok(());
        }
        let mut instance_binding_indices = Vec::with_capacity(draws.len());
        for draw in draws {
            let (rotation_sin, rotation_cos) = draw.instance.rotation.sin_cos();
            let uniform = GpuInstanceUniform {
                translation: draw.instance.translation,
                scale: draw.instance.scale,
                rotation: [rotation_sin, rotation_cos],
                _padding: [0.0, 0.0],
            };
            let binding_index = instance_binding_indices.len();
            self.prepare_render_target_instance_binding(binding_index, uniform);
            instance_binding_indices.push(binding_index);
        }

        let command_encoding_start = CpuTimer::start();
        let mut encoder = self
            ._device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("tokimu-render-target-pass"),
            });
        {
            let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("tokimu-render-target-pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &target_view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: clear_color.r as f64,
                            g: clear_color.g as f64,
                            b: clear_color.b as f64,
                            a: clear_color.a as f64,
                        }),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                    view: &depth_view,
                    depth_ops: Some(wgpu::Operations {
                        load: wgpu::LoadOp::Clear(1.0),
                        store: wgpu::StoreOp::Store,
                    }),
                    stencil_ops: None,
                }),
                occlusion_query_set: None,
                timestamp_writes: None,
            });
            for (index, draw) in draws.iter().enumerate() {
                let mesh = self
                    .meshes
                    .get(&draw.mesh)
                    .expect("mesh was validated before the pass");
                let material = self
                    .materials
                    .get(&draw.material)
                    .expect("material was validated before the pass");
                let pipeline = self
                    .pipelines
                    .get(&draw.pipeline)
                    .expect("pipeline was validated before the pass");
                let camera = self
                    .camera_bindings
                    .get(&draw.camera.unwrap_or(self.active_camera))
                    .expect("camera binding was prepared before the pass");
                pass.set_pipeline(pipeline);
                pass.set_bind_group(0, &material.bind_group, &[]);
                pass.set_bind_group(
                    1,
                    &self.instance_bindings[instance_binding_indices[index]].bind_group,
                    &[],
                );
                pass.set_bind_group(2, &camera.bind_group, &[]);
                pass.set_vertex_buffer(0, mesh.vertex_buffer.slice(..));
                pass.draw(0..mesh.vertex_count, 0..1);
            }
        }
        let command_buffer = encoder.finish();
        let command_encoding = command_encoding_start.map(CpuTimer::elapsed);
        let queue_submit_start = CpuTimer::start();
        self._queue.submit(std::iter::once(command_buffer));
        let queue_submit_call = queue_submit_start.map(CpuTimer::elapsed);
        self.stats.record_submit_call();
        self.stats.record_draw_calls(draws.len() as u32);
        self.stats
            .record_render_target_cpu_timings(command_encoding, queue_submit_call);
        Ok(())
    }

    fn prepare_render_target_camera(
        &mut self,
        handle: CameraHandle,
    ) -> Result<(), WgpuBackendError> {
        if self.camera_bindings.contains_key(&handle) {
            return Ok(());
        }
        let camera = self.cameras.get(&handle).copied().unwrap_or_default();
        let uniform = GpuCameraUniform {
            view_projection: (camera.projection * camera.view).to_cols_array_2d(),
        };
        let Some(surface_state) = self.surface_state.as_ref() else {
            return Ok(());
        };
        let buffer = self
            ._device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("tokimu-render-target-camera-uniform-buffer"),
                contents: bytemuck::bytes_of(&uniform),
                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            });
        let bind_group = self._device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("tokimu-render-target-camera-bind-group"),
            layout: &surface_state.camera_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: buffer.as_entire_binding(),
            }],
        });
        self.camera_bindings.insert(
            handle,
            GpuCameraBinding {
                uniform,
                _buffer: buffer,
                bind_group,
            },
        );
        self.stats.record_binding_allocation();
        Ok(())
    }

    fn prepare_render_target_instance_binding(
        &mut self,
        index: usize,
        uniform: GpuInstanceUniform,
    ) {
        if let Some(binding) = self.instance_bindings.get_mut(index) {
            if binding.uniform != uniform {
                self._queue
                    .write_buffer(&binding._buffer, 0, bytemuck::bytes_of(&uniform));
                binding.uniform = uniform;
                self.stats.record_uniform_buffer_write();
            }
            return;
        }
        if index != self.instance_bindings.len() {
            unreachable!("render-target instance bindings grow contiguously");
        }

        let surface_state = self
            .surface_state
            .as_ref()
            .expect("render-target draws require an initialized surface");
        let buffer = self
            ._device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("tokimu-render-target-instance-uniform-buffer"),
                contents: bytemuck::bytes_of(&uniform),
                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            });
        let bind_group = self._device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("tokimu-render-target-instance-bind-group"),
            layout: &surface_state.instance_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: buffer.as_entire_binding(),
            }],
        });
        self.instance_bindings.push(super::GpuInstanceBinding {
            uniform,
            _buffer: buffer,
            bind_group,
        });
        self.stats.record_binding_allocation();
    }
}
