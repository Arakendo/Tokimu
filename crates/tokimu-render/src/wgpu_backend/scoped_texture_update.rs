//! WGPU realization of ADR-0019 texture-content replacement.
//!
//! The transaction is texture-only and fixed-descriptor. It stages a new
//! provider realization and every dependent material bind group without
//! mutating the authoritative set, then swaps them at one commit.

use std::collections::HashMap;
use std::sync::Arc;

use wgpu::util::DeviceExt;

use crate::{
    MaterialHandle, RenderResourceSetId, RenderTextureContentUpdateLifecycle,
    Rgba8TextureDescriptor, TextureContentUpdateCommitObservation, TextureHandle,
};

use super::material_resources::material_uniform_buffer_usage;
use super::texture_support::{
    rgba8_texture_format, validate_rgba8_texture_update, write_rgba8_texture,
};
use super::{
    GpuMaterial, GpuTexture, GpuTextureRole, RenderResourceSetAuthority, WgpuBackend,
    WgpuBackendError, WgpuResourceSetSession,
};

/// Isolated WGPU candidate for one fixed-descriptor texture-content update.
/// Dropping it leaves the current set and its material bindings unchanged.
pub struct WgpuTextureContentUpdateCandidate {
    authority: Arc<RenderResourceSetAuthority>,
    resource_set: RenderResourceSetId,
    handle: TextureHandle,
    descriptor: Rgba8TextureDescriptor,
    texture: GpuTexture,
    rebound_materials: HashMap<MaterialHandle, GpuMaterial>,
    source_bytes: u64,
}

impl WgpuTextureContentUpdateCandidate {
    pub const fn resource_set(&self) -> RenderResourceSetId {
        self.resource_set
    }

    pub const fn texture(&self) -> TextureHandle {
        self.handle
    }
}

impl RenderTextureContentUpdateLifecycle for WgpuResourceSetSession {
    type Candidate = WgpuTextureContentUpdateCandidate;
    type Error = WgpuBackendError;

    fn prepare_texture_content_update(
        &self,
        handle: TextureHandle,
        rgba8: &[u8],
    ) -> Result<Self::Candidate, Self::Error> {
        self.backend().prepare_texture_content_update(handle, rgba8)
    }

    fn commit_texture_content_update(
        &mut self,
        candidate: Self::Candidate,
    ) -> Result<TextureContentUpdateCommitObservation, Self::Error> {
        self.backend_mut().commit_texture_content_update(candidate)
    }
}

impl WgpuBackend {
    fn prepare_texture_content_update(
        &self,
        handle: TextureHandle,
        rgba8: &[u8],
    ) -> Result<WgpuTextureContentUpdateCandidate, WgpuBackendError> {
        let existing = self.textures.get(&handle);
        if matches!(
            existing.map(|texture| texture.role),
            Some(GpuTextureRole::RenderTarget)
        ) {
            return Err(WgpuBackendError::TextureIsRenderTarget(handle.0));
        }
        let expected = existing.map(|texture| texture.descriptor);
        let (width, height) = expected
            .map(|descriptor| (descriptor.width, descriptor.height))
            .unwrap_or_default();
        let descriptor = validate_rgba8_texture_update(handle, expected, width, height, rgba8)?;
        let material_bind_group_layout = self
            .material_bind_group_layout()
            .ok_or(WgpuBackendError::ResourceSetStageRequiresSurface)?;

        let texture = self._device.create_texture(&wgpu::TextureDescriptor {
            label: Some("tokimu-texture-content-update-candidate"),
            size: wgpu::Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: rgba8_texture_format(descriptor.color_space),
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        });
        write_rgba8_texture(&self._queue, &texture, width, height, rgba8);
        let view = Arc::new(texture.create_view(&wgpu::TextureViewDescriptor::default()));
        let texture = GpuTexture {
            texture,
            view: Arc::clone(&view),
            _depth_texture: None,
            depth_view: None,
            descriptor,
            role: GpuTextureRole::Source,
        };

        let mut rebound_materials = HashMap::new();
        for (material_handle, material) in self
            .materials
            .iter()
            .filter(|(_, material)| material.texture == Some(handle))
        {
            let uniform = [
                material.base_color.r,
                material.base_color.g,
                material.base_color.b,
                material.base_color.a,
            ];
            let uniform_buffer =
                self._device
                    .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                        label: Some("tokimu-texture-content-update-material-uniform"),
                        contents: bytemuck::cast_slice(&uniform),
                        usage: material_uniform_buffer_usage(),
                    });
            let bind_group = self._device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("tokimu-texture-content-update-material-binding"),
                layout: material_bind_group_layout,
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: uniform_buffer.as_entire_binding(),
                    },
                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: wgpu::BindingResource::TextureView(&view),
                    },
                    wgpu::BindGroupEntry {
                        binding: 2,
                        resource: wgpu::BindingResource::Sampler(&material.sampler),
                    },
                ],
            });
            rebound_materials.insert(
                *material_handle,
                GpuMaterial {
                    base_color: material.base_color,
                    texture: material.texture,
                    _uniform_buffer: uniform_buffer,
                    bind_group,
                    texture_view: Arc::clone(&view),
                    sampler: Arc::clone(&material.sampler),
                    _fallback_texture: None,
                    _fallback_view: None,
                    _fallback_sampler: None,
                },
            );
        }

        Ok(WgpuTextureContentUpdateCandidate {
            authority: Arc::clone(&self.resource_set_authority),
            resource_set: self.current_resource_set,
            handle,
            descriptor,
            texture,
            rebound_materials,
            source_bytes: rgba8.len() as u64,
        })
    }

    fn commit_texture_content_update(
        &mut self,
        mut candidate: WgpuTextureContentUpdateCandidate,
    ) -> Result<TextureContentUpdateCommitObservation, WgpuBackendError> {
        if !Arc::ptr_eq(&self.resource_set_authority, &candidate.authority) {
            return Err(WgpuBackendError::TextureContentUpdateWrongProviderSession);
        }
        if candidate.resource_set != self.current_resource_set {
            return Err(WgpuBackendError::TextureContentUpdateStaleResourceSet {
                requested: candidate.resource_set,
                current: self.current_resource_set,
            });
        }

        // Scope rejects above intentionally precede target lookup.
        let current = self
            .textures
            .get(&candidate.handle)
            .ok_or(WgpuBackendError::MissingTexture(candidate.handle.0))?;
        if current.role == GpuTextureRole::RenderTarget {
            return Err(WgpuBackendError::TextureIsRenderTarget(candidate.handle.0));
        }
        if current.descriptor != candidate.descriptor {
            return Err(WgpuBackendError::TextureDimensionsMismatch {
                handle: candidate.handle.0,
                expected_width: current.descriptor.width,
                expected_height: current.descriptor.height,
                actual_width: candidate.descriptor.width,
                actual_height: candidate.descriptor.height,
            });
        }

        let rebound_materials = candidate.rebound_materials.len() as u32;
        let dependent_handles = candidate
            .rebound_materials
            .keys()
            .copied()
            .collect::<Vec<_>>();
        self.derived_materials
            .retain(|key, _| !dependent_handles.contains(&key.source));

        self.textures.insert(candidate.handle, candidate.texture);
        for (handle, material) in candidate.rebound_materials.drain() {
            self.materials.insert(handle, material);
            self.stats.record_binding_allocation();
        }
        self.stats.record_texture_allocation(true);
        self.stats.record_texture_write();

        Ok(TextureContentUpdateCommitObservation {
            resource_set: self.current_resource_set,
            texture: candidate.handle,
            descriptor: candidate.descriptor,
            source_bytes: candidate.source_bytes,
            dependent_materials: rebound_materials,
        })
    }
}
