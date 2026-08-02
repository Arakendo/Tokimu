use std::sync::Arc;

use wgpu::util::DeviceExt;

use crate::{Material, MaterialHandle};

use super::material_support::{create_derived_material, derived_material_key};
use super::texture_support::rgba8_point_sampler_descriptor;
use super::{GpuMaterial, WgpuBackend, WgpuBackendError};

impl WgpuBackend {
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
                        Arc::clone(&texture.view),
                        Arc::new(
                            self._device
                                .create_sampler(&rgba8_point_sampler_descriptor()),
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
                let view = Arc::new(texture.create_view(&wgpu::TextureViewDescriptor::default()));
                let sampler = Arc::new(
                    self._device
                        .create_sampler(&rgba8_point_sampler_descriptor()),
                );
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

    pub(super) fn prepare_derived_materials(&mut self) -> Result<(), WgpuBackendError> {
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
}
