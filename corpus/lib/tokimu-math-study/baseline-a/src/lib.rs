//! Independently compilable Alternative A control boundary.
//!
//! It imports the current stable `tokimu_core::math` re-exports so the WASM
//! probe exercises the actual public vocabulary rather than a local alias.

use tokimu_core::math::{Mat4, Vec3};

/// Bounded plain-WASM execution probe for the stable direct-provider control.
///
/// This is not a browser API or a stable FFI contract. It exists only so the
/// study can execute the same transform path through A, B, and C.
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

/// Corpus-only repeated stereo-camera math and column-boundary probe.
///
/// This intentionally stays below the renderer: it observes A's candidate
/// math plus matrix-column representation work in a plain WASM engine. The
/// native study remains the evidence for construction of `tokimu::Camera`.
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

/// Corpus-only WASM layout observation for the stable control.
#[no_mangle]
pub extern "C" fn tokimu_math_study_wasm_vec4_alignment() -> usize {
    core::mem::align_of::<tokimu_core::math::Vec4>()
}

/// Corpus-only WASM layout observation for the stable control.
#[no_mangle]
pub extern "C" fn tokimu_math_study_wasm_mat4_alignment() -> usize {
    core::mem::align_of::<Mat4>()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn baseline_uses_the_stable_tokimu_math_reexport() {
        assert!((tokimu_math_study_wasm_probe() - 292.0).abs() < 1.0e-3);
    }
}
