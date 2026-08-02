use std::sync::Arc;

use wgpu::util::DeviceExt;

use crate::{Color, MaterialHandle, MaterialOverride};

use super::{DerivedMaterialKey, GpuMaterial};

pub(super) fn derived_material_key(
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

pub(super) fn create_derived_material(
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
        texture: source.texture,
        _uniform_buffer: uniform_buffer,
        bind_group,
        texture_view: Arc::clone(&source.texture_view),
        sampler: Arc::clone(&source.sampler),
        _fallback_texture: None,
        _fallback_view: None,
        _fallback_sampler: None,
    }
}
