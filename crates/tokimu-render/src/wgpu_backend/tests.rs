use super::{
    diagnostics::drain_backend_diagnostic_messages,
    material_support::derived_material_key,
    texture_support::{
        rgba8_point_sampler_descriptor, rgba8_texture_format, validate_legacy_texture_replacement,
        validate_rgba8_render_target_creation, validate_rgba8_texture_creation,
        validate_rgba8_texture_update,
    },
    GpuTextureRole, WgpuBackendError,
};
use crate::{
    Color, MaterialHandle, MaterialOverride, Rgba8TextureColorSpace, Rgba8TextureDescriptor,
    TextureHandle, TextureValidationError,
};
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

#[test]
fn rgba8_color_space_maps_to_explicit_backend_formats() {
    assert_eq!(
        rgba8_texture_format(Rgba8TextureColorSpace::Linear),
        wgpu::TextureFormat::Rgba8Unorm
    );
    assert_eq!(
        rgba8_texture_format(Rgba8TextureColorSpace::Srgb),
        wgpu::TextureFormat::Rgba8UnormSrgb
    );
}

#[test]
fn rgba8_profile_preserves_point_filtering_and_clamp_addressing() {
    let descriptor = rgba8_point_sampler_descriptor();

    assert_eq!(descriptor.address_mode_u, wgpu::AddressMode::ClampToEdge);
    assert_eq!(descriptor.address_mode_v, wgpu::AddressMode::ClampToEdge);
    assert_eq!(descriptor.address_mode_w, wgpu::AddressMode::ClampToEdge);
    assert_eq!(descriptor.mag_filter, wgpu::FilterMode::Nearest);
    assert_eq!(descriptor.min_filter, wgpu::FilterMode::Nearest);
    assert_eq!(descriptor.mipmap_filter, wgpu::FilterMode::Nearest);
}

#[test]
fn texture_creation_validation_rejects_duplicates_and_invalid_payloads_before_allocation() {
    let descriptor = Rgba8TextureDescriptor::new(2, 2, Rgba8TextureColorSpace::Srgb);

    assert!(matches!(
        validate_rgba8_texture_creation(TextureHandle(7), true, descriptor, &[0; 16]),
        Err(WgpuBackendError::TextureAlreadyExists(7))
    ));
    assert!(matches!(
        validate_rgba8_texture_creation(TextureHandle(7), false, descriptor, &[0; 15]),
        Err(WgpuBackendError::InvalidTexture(
            TextureValidationError::PayloadLengthMismatch { .. }
        ))
    ));
}

#[test]
fn render_target_creation_validation_rejects_duplicates_and_empty_dimensions() {
    let descriptor = Rgba8TextureDescriptor::new(1280, 720, Rgba8TextureColorSpace::Srgb);
    assert!(validate_rgba8_render_target_creation(TextureHandle(11), false, descriptor).is_ok());
    assert!(matches!(
        validate_rgba8_render_target_creation(TextureHandle(11), true, descriptor),
        Err(WgpuBackendError::TextureAlreadyExists(11))
    ));
    assert!(matches!(
        validate_rgba8_render_target_creation(
            TextureHandle(12),
            false,
            Rgba8TextureDescriptor::new(0, 720, Rgba8TextureColorSpace::Srgb)
        ),
        Err(WgpuBackendError::InvalidTexture(
            TextureValidationError::InvalidDimensions { .. }
        ))
    ));
}

#[test]
fn legacy_texture_replacement_rejects_renderer_owned_targets() {
    assert_eq!(
        validate_legacy_texture_replacement(TextureHandle(8), Some(GpuTextureRole::RenderTarget))
            .unwrap_err()
            .to_string(),
        "texture handle 8 is a renderer-owned render target and cannot receive source-pixel updates"
    );
    assert!(
        validate_legacy_texture_replacement(TextureHandle(8), Some(GpuTextureRole::Source)).is_ok()
    );
}

#[test]
fn texture_update_validation_rejects_missing_resized_and_invalid_payload_requests() {
    let handle = TextureHandle(9);
    let descriptor = Rgba8TextureDescriptor::new(2, 2, Rgba8TextureColorSpace::Linear);

    assert!(matches!(
        validate_rgba8_texture_update(handle, None, 2, 2, &[0; 16]),
        Err(WgpuBackendError::MissingTexture(9))
    ));
    assert!(matches!(
        validate_rgba8_texture_update(handle, Some(descriptor), 1, 2, &[0; 8]),
        Err(WgpuBackendError::TextureDimensionsMismatch {
            expected_width: 2,
            expected_height: 2,
            actual_width: 1,
            actual_height: 2,
            ..
        })
    ));
    assert!(matches!(
        validate_rgba8_texture_update(handle, Some(descriptor), 2, 2, &[0; 15]),
        Err(WgpuBackendError::InvalidTexture(
            TextureValidationError::PayloadLengthMismatch { .. }
        ))
    ));
    assert_eq!(
        validate_rgba8_texture_update(handle, Some(descriptor), 2, 2, &[0; 16]).unwrap(),
        descriptor
    );
}
