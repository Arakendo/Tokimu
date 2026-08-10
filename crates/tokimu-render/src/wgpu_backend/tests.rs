use super::{
    diagnostics::drain_backend_diagnostic_messages,
    material_resources::{material_uniform_buffer_usage, validate_material_color},
    material_support::derived_material_key,
    texture_resources::render_target_resource_observation,
    texture_support::{
        rgba8_sampler_descriptor, rgba8_texture_format, validate_legacy_texture_replacement,
        validate_rgba8_render_target_creation, validate_rgba8_render_target_release,
        validate_rgba8_render_target_replacement, validate_rgba8_texture_creation,
        validate_rgba8_texture_update,
    },
    wgpu_camera_uniform, GpuTextureRole, WgpuBackendError,
};
use crate::{
    Camera, Color, MaterialHandle, MaterialOverride, Rgba8TextureColorSpace,
    Rgba8TextureDescriptor, TextureAddressMode, TextureFilter, TextureHandle, TextureSampler,
    TextureValidationError,
};
use std::sync::Mutex;
use tokimu_core::math::{Mat4, Vec3};

#[test]
fn wgpu_camera_upload_converts_tokimu_clip_depth_without_changing_camera() {
    let camera = Camera::new(Mat4::IDENTITY, Mat4::IDENTITY);
    let original = camera;
    let uploaded = Mat4::from_cols_array_2d(&wgpu_camera_uniform(camera).view_projection);

    for (tokimu_depth, expected_wgpu_depth) in [(-1.0, 0.0), (0.0, 0.5), (1.0, 1.0)] {
        let uploaded_depth = uploaded.project_point3(Vec3::new(0.0, 0.0, tokimu_depth)).z;
        assert!((uploaded_depth - expected_wgpu_depth).abs() <= f32::EPSILON);
    }
    assert_eq!(camera, original);
}

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
fn material_color_updates_reject_non_finite_uniform_data_before_queue_write() {
    assert!(validate_material_color(Color::rgba(0.1, 0.2, 0.3, 1.0)).is_ok());
    assert!(matches!(
        validate_material_color(Color::rgba(f32::NAN, 0.2, 0.3, 1.0)),
        Err(WgpuBackendError::InvalidMaterialColor)
    ));
}

#[test]
fn material_color_uniforms_support_runtime_queue_updates() {
    assert!(material_uniform_buffer_usage().contains(wgpu::BufferUsages::UNIFORM));
    assert!(material_uniform_buffer_usage().contains(wgpu::BufferUsages::COPY_DST));
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
    let descriptor = rgba8_sampler_descriptor(TextureSampler::default());

    assert_eq!(descriptor.address_mode_u, wgpu::AddressMode::ClampToEdge);
    assert_eq!(descriptor.address_mode_v, wgpu::AddressMode::ClampToEdge);
    assert_eq!(descriptor.address_mode_w, wgpu::AddressMode::ClampToEdge);
    assert_eq!(descriptor.mag_filter, wgpu::FilterMode::Nearest);
    assert_eq!(descriptor.min_filter, wgpu::FilterMode::Nearest);
    assert_eq!(descriptor.mipmap_filter, wgpu::FilterMode::Nearest);
}

#[test]
fn rgba8_sampler_maps_declared_linear_repeat_policy() {
    let descriptor = rgba8_sampler_descriptor(TextureSampler {
        filter: TextureFilter::Linear,
        address_u: TextureAddressMode::Repeat,
        address_v: TextureAddressMode::Repeat,
    });

    assert_eq!(descriptor.address_mode_u, wgpu::AddressMode::Repeat);
    assert_eq!(descriptor.address_mode_v, wgpu::AddressMode::Repeat);
    assert_eq!(descriptor.mag_filter, wgpu::FilterMode::Linear);
    assert_eq!(descriptor.min_filter, wgpu::FilterMode::Linear);
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
fn render_target_observation_counts_color_and_depth_images_without_claiming_gpu_residency() {
    let observation = render_target_resource_observation([
        Rgba8TextureDescriptor::new(640, 360, Rgba8TextureColorSpace::Srgb),
        Rgba8TextureDescriptor::new(320, 180, Rgba8TextureColorSpace::Linear),
    ]);

    assert_eq!(observation.target_count, 2);
    assert_eq!(observation.color_pixels, 288_000);
    assert_eq!(observation.estimated_color_bytes, 1_152_000);
    assert_eq!(observation.estimated_depth_bytes, 1_152_000);
    assert_eq!(observation.estimated_total_bytes, 2_304_000);
}

#[test]
fn render_target_replacement_requires_an_existing_renderer_owned_target() {
    let descriptor = Rgba8TextureDescriptor::new(640, 360, Rgba8TextureColorSpace::Srgb);
    assert!(validate_rgba8_render_target_replacement(
        TextureHandle(13),
        Some(GpuTextureRole::RenderTarget),
        descriptor,
    )
    .is_ok());
    assert!(matches!(
        validate_rgba8_render_target_replacement(TextureHandle(13), None, descriptor),
        Err(WgpuBackendError::MissingTexture(13))
    ));
    assert!(matches!(
        validate_rgba8_render_target_replacement(
            TextureHandle(13),
            Some(GpuTextureRole::Source),
            descriptor,
        ),
        Err(WgpuBackendError::TextureIsNotRenderTarget(13))
    ));
}

#[test]
fn render_target_release_requires_detached_materials() {
    assert!(validate_rgba8_render_target_release(
        TextureHandle(14),
        Some(GpuTextureRole::RenderTarget),
        0,
    )
    .is_ok());
    assert!(matches!(
        validate_rgba8_render_target_release(
            TextureHandle(14),
            Some(GpuTextureRole::RenderTarget),
            2,
        ),
        Err(WgpuBackendError::RenderTargetStillReferenced {
            target: 14,
            material_count: 2,
        })
    ));
    assert!(matches!(
        validate_rgba8_render_target_release(TextureHandle(14), Some(GpuTextureRole::Source), 0,),
        Err(WgpuBackendError::TextureIsNotRenderTarget(14))
    ));
    assert!(matches!(
        validate_rgba8_render_target_release(TextureHandle(14), None, 0),
        Err(WgpuBackendError::MissingTexture(14))
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
