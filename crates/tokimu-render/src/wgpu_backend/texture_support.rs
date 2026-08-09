use super::{GpuTextureRole, WgpuBackendError};
use crate::{
    Rgba8TextureColorSpace, Rgba8TextureDescriptor, TextureAddressMode, TextureFilter,
    TextureHandle, TextureSampler,
};

pub(super) fn rgba8_texture_format(color_space: Rgba8TextureColorSpace) -> wgpu::TextureFormat {
    match color_space {
        Rgba8TextureColorSpace::Linear => wgpu::TextureFormat::Rgba8Unorm,
        Rgba8TextureColorSpace::Srgb => wgpu::TextureFormat::Rgba8UnormSrgb,
    }
}

pub(super) fn rgba8_sampler_descriptor(
    sampler: TextureSampler,
) -> wgpu::SamplerDescriptor<'static> {
    wgpu::SamplerDescriptor {
        label: Some("tokimu-rgba8-material-sampler"),
        address_mode_u: address_mode(sampler.address_u),
        address_mode_v: address_mode(sampler.address_v),
        address_mode_w: wgpu::AddressMode::ClampToEdge,
        mag_filter: filter_mode(sampler.filter),
        min_filter: filter_mode(sampler.filter),
        mipmap_filter: wgpu::FilterMode::Nearest,
        ..wgpu::SamplerDescriptor::default()
    }
}

fn address_mode(mode: TextureAddressMode) -> wgpu::AddressMode {
    match mode {
        TextureAddressMode::Clamp => wgpu::AddressMode::ClampToEdge,
        TextureAddressMode::Repeat => wgpu::AddressMode::Repeat,
    }
}

fn filter_mode(filter: TextureFilter) -> wgpu::FilterMode {
    match filter {
        TextureFilter::Point => wgpu::FilterMode::Nearest,
        TextureFilter::Linear => wgpu::FilterMode::Linear,
    }
}

pub(super) fn write_rgba8_texture(
    queue: &wgpu::Queue,
    texture: &wgpu::Texture,
    width: u32,
    height: u32,
    rgba8: &[u8],
) {
    queue.write_texture(
        wgpu::ImageCopyTexture {
            texture,
            mip_level: 0,
            origin: wgpu::Origin3d::ZERO,
            aspect: wgpu::TextureAspect::All,
        },
        rgba8,
        wgpu::ImageDataLayout {
            offset: 0,
            bytes_per_row: Some(4 * width),
            rows_per_image: Some(height),
        },
        wgpu::Extent3d {
            width,
            height,
            depth_or_array_layers: 1,
        },
    );
}

// These checks are intentionally isolated from wgpu so failed requests can be
// tested as non-destructive contract behavior without allocating a device.
pub(super) fn validate_rgba8_texture_creation(
    handle: TextureHandle,
    handle_exists: bool,
    descriptor: Rgba8TextureDescriptor,
    rgba8: &[u8],
) -> Result<(), WgpuBackendError> {
    descriptor.validate_payload(rgba8)?;
    if handle_exists {
        return Err(WgpuBackendError::TextureAlreadyExists(handle.0));
    }
    Ok(())
}

// This check is kept free of WGPU allocation so corpus callers can prove that
// invalid render-target requests fail at the renderer contract boundary.
pub(super) fn validate_rgba8_render_target_creation(
    handle: TextureHandle,
    handle_exists: bool,
    descriptor: Rgba8TextureDescriptor,
) -> Result<(), WgpuBackendError> {
    descriptor.expected_payload_len()?;
    if handle_exists {
        return Err(WgpuBackendError::TextureAlreadyExists(handle.0));
    }
    Ok(())
}

pub(super) fn validate_rgba8_render_target_replacement(
    handle: TextureHandle,
    existing_role: Option<GpuTextureRole>,
    descriptor: Rgba8TextureDescriptor,
) -> Result<(), WgpuBackendError> {
    descriptor.expected_payload_len()?;
    match existing_role {
        Some(GpuTextureRole::RenderTarget) => Ok(()),
        Some(GpuTextureRole::Source) => Err(WgpuBackendError::TextureIsNotRenderTarget(handle.0)),
        None => Err(WgpuBackendError::MissingTexture(handle.0)),
    }
}

/// Keeps target release deterministic and non-destructive when a material still
/// retains the target's backend view through its bind group.
pub(super) fn validate_rgba8_render_target_release(
    handle: TextureHandle,
    existing_role: Option<GpuTextureRole>,
    material_references: u32,
) -> Result<(), WgpuBackendError> {
    match existing_role {
        Some(GpuTextureRole::RenderTarget) => {}
        Some(GpuTextureRole::Source) => {
            return Err(WgpuBackendError::TextureIsNotRenderTarget(handle.0))
        }
        None => return Err(WgpuBackendError::MissingTexture(handle.0)),
    }
    if material_references > 0 {
        return Err(WgpuBackendError::RenderTargetStillReferenced {
            target: handle.0,
            material_count: material_references,
        });
    }
    Ok(())
}

pub(super) fn validate_rgba8_texture_update(
    handle: TextureHandle,
    expected: Option<Rgba8TextureDescriptor>,
    width: u32,
    height: u32,
    rgba8: &[u8],
) -> Result<Rgba8TextureDescriptor, WgpuBackendError> {
    let expected = expected.ok_or(WgpuBackendError::MissingTexture(handle.0))?;
    if width != expected.width || height != expected.height {
        return Err(WgpuBackendError::TextureDimensionsMismatch {
            handle: handle.0,
            expected_width: expected.width,
            expected_height: expected.height,
            actual_width: width,
            actual_height: height,
        });
    }
    expected.validate_payload(rgba8)?;
    Ok(expected)
}

pub(super) fn validate_legacy_texture_replacement(
    handle: TextureHandle,
    existing_role: Option<GpuTextureRole>,
) -> Result<(), WgpuBackendError> {
    if existing_role == Some(GpuTextureRole::RenderTarget) {
        return Err(WgpuBackendError::TextureIsRenderTarget(handle.0));
    }
    Ok(())
}
