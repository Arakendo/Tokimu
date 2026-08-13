//! Independently compilable Alternative B source boundary.
//!
//! The provider remains a direct private dependency by design. This target
//! makes that closure observable without pulling in the A control or C.

#[cfg(all(feature = "provider-029", feature = "provider-033"))]
compile_error!("select exactly one Full-B provider revision");
#[cfg(not(any(feature = "provider-029", feature = "provider-033")))]
compile_error!("select one Full-B provider revision");

#[cfg(feature = "provider-029")]
use glam_029 as selected_provider;
#[cfg(feature = "provider-033")]
use glam_033 as selected_provider;

pub(crate) mod alternative_b_provider {
    pub(crate) use super::selected_provider::{Mat4, Quat, Vec2, Vec3, Vec4};

    #[cfg(feature = "provider-029")]
    pub(crate) fn look_at_rh(eye: Vec3, target: Vec3, up: Vec3) -> Mat4 {
        Mat4::look_at_rh(eye, target, up)
    }

    #[cfg(feature = "provider-033")]
    pub(crate) fn look_at_rh(eye: Vec3, target: Vec3, up: Vec3) -> Mat4 {
        super::selected_provider::camera::rh::view::look_at_mat4(eye, target, up)
    }

    #[cfg(feature = "provider-029")]
    pub(crate) fn perspective_rh_gl(
        vertical_fov_radians: f32,
        aspect_ratio: f32,
        near: f32,
        far: f32,
    ) -> Mat4 {
        Mat4::perspective_rh_gl(vertical_fov_radians, aspect_ratio, near, far)
    }

    #[cfg(feature = "provider-033")]
    pub(crate) fn perspective_rh_gl(
        vertical_fov_radians: f32,
        aspect_ratio: f32,
        near: f32,
        far: f32,
    ) -> Mat4 {
        super::selected_provider::camera::rh::proj::opengl::perspective(
            vertical_fov_radians,
            aspect_ratio,
            near,
            far,
        )
    }

    #[cfg(feature = "provider-029")]
    pub(crate) fn orthographic_rh_gl(
        left: f32,
        right: f32,
        bottom: f32,
        top: f32,
        near: f32,
        far: f32,
    ) -> Mat4 {
        Mat4::orthographic_rh_gl(left, right, bottom, top, near, far)
    }

    #[cfg(feature = "provider-033")]
    pub(crate) fn orthographic_rh_gl(
        left: f32,
        right: f32,
        bottom: f32,
        top: f32,
        near: f32,
        far: f32,
    ) -> Mat4 {
        super::selected_provider::camera::rh::proj::opengl::orthographic(
            left, right, bottom, top, near, far,
        )
    }
}

#[path = "../../src/alternative_b.rs"]
mod implementation;

pub use implementation::{Mat4, MathError, MathFailure, MathOperation, Quat, Vec2, Vec3, Vec4};

/// Bounded plain-WASM execution probe for the provider-backed candidate.
///
/// This is not a browser API or a stable FFI contract. It exists only so the
/// study can execute one checked B transform path in a WASM engine.
#[no_mangle]
pub extern "C" fn tokimu_math_study_wasm_probe() -> f32 {
    let transform = Mat4::from_translation(Vec3::new(4.0, -2.0, 1.0))
        * Mat4::from_rotation_y(0.7)
        * Mat4::from_scale(Vec3::new(1.5, 0.5, 2.0));
    let restored = transform
        .inverse()
        .transform_point3(transform.transform_point3(Vec3::new(3.0, -1.0, 2.0)));

    restored.x() * 100.0 + restored.y() * 10.0 + restored.z()
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

/// Corpus-only WASM layout observation for the provider-backed candidate.
#[no_mangle]
pub extern "C" fn tokimu_math_study_wasm_vec4_alignment() -> usize {
    core::mem::align_of::<Vec4>()
}

/// Corpus-only WASM layout observation for the provider-backed candidate.
#[no_mangle]
pub extern "C" fn tokimu_math_study_wasm_mat4_alignment() -> usize {
    core::mem::align_of::<Mat4>()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn provider_backed_subset_compiles_with_a_private_provider() {
        let transform = Mat4::from_translation(Vec3::new(4.0, 5.0, 6.0));

        assert_eq!(
            transform
                .inverse()
                .transform_point3(Vec3::new(5.0, 7.0, 9.0))
                .to_array(),
            [1.0, 2.0, 3.0]
        );
    }
}
