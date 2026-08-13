//! Independently compilable Alternative C source boundary.
//!
//! This crate deliberately has no dependencies. It reuses the experimental
//! owned candidate source rather than copying it, so this target checks that
//! the candidate's compilation boundary remains provider-free.

#[path = "../../src/alternative_c.rs"]
mod implementation;

pub use implementation::{Mat4, Vec3, Vec4};

/// Bounded plain-WASM execution probe for the owned candidate.
///
/// This is not a browser API or a stable FFI contract. It exists only so the
/// study can execute one checked C transform path in a WASM engine.
#[no_mangle]
pub extern "C" fn tokimu_math_study_wasm_probe() -> f32 {
    let transform = Mat4::from_translation(Vec3::new(4.0, -2.0, 1.0))
        * Mat4::from_rotation_y(0.7)
        * Mat4::from_scale(Vec3::new(1.5, 0.5, 2.0));
    let restored = transform
        .inverse()
        .transform_point3(transform.transform_point3(Vec3::new(3.0, -1.0, 2.0)));

    restored.x * 100.0 + restored.y * 10.0 + restored.z
}

/// Bounded checked-contract semantic probe for plain-WASM execution.
///
/// Each bit denotes one locally meaningful C0 behavior. This is retained
/// corpus machinery, not a stable FFI or diagnostics contract.
#[no_mangle]
pub extern "C" fn tokimu_math_study_wasm_checked_probe() -> u32 {
    let mut result = 0_u32;
    if Vec3::new(3.0, 4.0, 0.0)
        .try_normalize()
        .is_some_and(|value| (value.length() - 1.0).abs() <= 1.0e-6)
    {
        result |= 1 << 0;
    }
    if Vec3::ZERO.try_normalize().is_none()
        && Vec3::new(f32::NAN, 0.0, 0.0)
            .try_normalize_or_zero()
            .is_none()
    {
        result |= 1 << 1;
    }

    let transform = Mat4::from_translation(Vec3::new(4.0, -2.0, 1.0))
        * Mat4::from_rotation_y(0.7)
        * Mat4::from_scale(Vec3::new(1.5, 0.5, 2.0));
    if transform
        .try_inverse()
        .is_some_and(|inverse| {
            let restored = inverse.transform_point3(transform.transform_point3(Vec3::new(3.0, -1.0, 2.0)));
            (restored - Vec3::new(3.0, -1.0, 2.0)).length() <= 1.0e-3
        })
    {
        result |= 1 << 2;
    }
    if Mat4::from_scale(Vec3::new(1.0, 0.0, 1.0))
        .try_inverse()
        .is_none()
    {
        result |= 1 << 3;
    }

    let view = Mat4::try_look_at_rh(Vec3::new(2.0, 3.0, 5.0), Vec3::ZERO, Vec3::Y);
    let projection = Mat4::try_perspective_rh_gl(1.0, 16.0 / 9.0, 0.1, 100.0);
    if let (Some(view), Some(projection)) = (view, projection) {
        let matrix = projection * view;
        if matrix
            .try_project_point3(Vec3::new(0.25, -0.5, 0.75))
            .is_some()
            && projection.try_project_point3(Vec3::ZERO).is_none()
        {
            result |= 1 << 4;
        }
    }
    if Mat4::try_look_at_rh(Vec3::ZERO, Vec3::ZERO, Vec3::Y).is_none()
        && Mat4::try_perspective_rh_gl(0.0, 1.0, 0.1, 100.0).is_none()
    {
        result |= 1 << 5;
    }
    result
}

/// Corpus-only repeated stereo-camera math and column-boundary probe.
#[no_mangle]
pub extern "C" fn tokimu_math_study_wasm_stereo_camera_probe(iterations: u32) -> f32 {
    let mut checksum = 0.0;
    for frame in 0..iterations {
        let seconds = core::hint::black_box(frame as f32 * 0.016);
        let angle = seconds * 0.8;
        let center_eye = Vec3::new(
            angle.cos() * 3.0,
            0.35 + (seconds * 1.3).sin() * 0.15,
            angle.sin() * 3.0,
        );
        let right = (Vec3::ZERO - center_eye)
            .normalize()
            .cross(Vec3::Y)
            .normalize();
        let projection =
            Mat4::perspective_rh_gl(60.0_f32.to_radians(), 1280.0 / 720.0 * 0.5, 0.1, 100.0);
        let left = (projection * Mat4::look_at_rh(center_eye - right * 0.14, Vec3::ZERO, Vec3::Y))
            .to_cols_array();
        let right = (projection * Mat4::look_at_rh(center_eye + right * 0.14, Vec3::ZERO, Vec3::Y))
            .to_cols_array();
        checksum += core::hint::black_box(left[0] + right[5]);
    }
    checksum
}

/// Corpus-only WASM layout observation for the owned candidate.
#[no_mangle]
pub extern "C" fn tokimu_math_study_wasm_vec4_alignment() -> usize {
    core::mem::align_of::<Vec4>()
}

/// Corpus-only WASM size observation for the owned candidate.
#[no_mangle]
pub extern "C" fn tokimu_math_study_wasm_vec4_size() -> usize {
    core::mem::size_of::<Vec4>()
}

/// Corpus-only WASM layout observation for the owned candidate.
#[no_mangle]
pub extern "C" fn tokimu_math_study_wasm_mat4_alignment() -> usize {
    core::mem::align_of::<Mat4>()
}

/// Corpus-only WASM size observation for the owned candidate.
#[no_mangle]
pub extern "C" fn tokimu_math_study_wasm_mat4_size() -> usize {
    core::mem::size_of::<Mat4>()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn owned_subset_compiles_as_a_dependency_free_candidate() {
        let transform = Mat4::from_translation(Vec3::new(4.0, 5.0, 6.0));

        assert_eq!(
            transform
                .inverse()
                .transform_point3(Vec3::new(5.0, 7.0, 9.0))
                .to_array(),
            [1.0, 2.0, 3.0]
        );
    }

    #[test]
    fn checked_probe_retains_the_selected_six_semantic_outcomes() {
        assert_eq!(tokimu_math_study_wasm_checked_probe(), 0b11_1111);
    }

    #[test]
    fn native_representation_observation_remains_scalar_and_non_contractual() {
        assert_eq!(core::mem::size_of::<Vec4>(), 16);
        assert_eq!(core::mem::align_of::<Vec4>(), 4);
        assert_eq!(core::mem::size_of::<Mat4>(), 64);
        assert_eq!(core::mem::align_of::<Mat4>(), 4);
    }
}
