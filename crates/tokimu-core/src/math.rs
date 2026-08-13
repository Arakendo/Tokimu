pub use glam::{Mat4, Quat, Vec2, Vec3, Vec4};

/// Constructs a finite right-handed, Y-up view matrix with `-Z` forward.
///
/// Degenerate or non-finite camera bases are rejected. This is Tokimu's
/// checked view-construction boundary; the selected numerical provider remains
/// an implementation detail.
#[must_use]
pub fn try_view_look_at_rh(eye: Vec3, center: Vec3, up: Vec3) -> Option<Mat4> {
    if !eye.is_finite() || !center.is_finite() || !up.is_finite() {
        return None;
    }

    let matrix = glam::camera::rh::view::look_at_mat4(eye, center, up);
    matrix.is_finite().then_some(matrix)
}

/// Constructs a finite right-handed perspective projection with Y-up and
/// GL-style `[-1, 1]` clip depth.
#[must_use]
pub fn try_projection_perspective_rh_gl(
    vertical_fov_radians: f32,
    aspect_ratio: f32,
    near: f32,
    far: f32,
) -> Option<Mat4> {
    let valid = vertical_fov_radians.is_finite()
        && aspect_ratio.is_finite()
        && near.is_finite()
        && far.is_finite()
        && 0.0 < vertical_fov_radians
        && vertical_fov_radians < core::f32::consts::PI
        && aspect_ratio > 0.0
        && 0.0 < near
        && near < far;
    if !valid {
        return None;
    }

    let matrix =
        glam::camera::rh::proj::opengl::perspective(vertical_fov_radians, aspect_ratio, near, far);
    matrix.is_finite().then_some(matrix)
}

/// Constructs a finite right-handed orthographic projection with Y-up and
/// GL-style `[-1, 1]` clip depth.
#[must_use]
pub fn try_projection_orthographic_rh_gl(
    left: f32,
    right: f32,
    bottom: f32,
    top: f32,
    near: f32,
    far: f32,
) -> Option<Mat4> {
    let valid = left.is_finite()
        && right.is_finite()
        && bottom.is_finite()
        && top.is_finite()
        && near.is_finite()
        && far.is_finite()
        && left < right
        && bottom < top
        && near < far;
    if !valid {
        return None;
    }

    let matrix = glam::camera::rh::proj::opengl::orthographic(left, right, bottom, top, near, far);
    matrix.is_finite().then_some(matrix)
}

#[cfg(test)]
mod tests {
    use super::*;

    const EPSILON: f32 = 1.0e-5;

    fn assert_near(actual: f32, expected: f32) {
        assert!(
            (actual - expected).abs() <= EPSILON,
            "expected {expected}, got {actual}"
        );
    }

    #[test]
    fn checked_view_uses_right_handed_y_up_basis() {
        let eye = Vec3::new(2.0, 3.0, 5.0);
        let center = Vec3::new(2.0, 3.0, 4.0);
        let view = try_view_look_at_rh(eye, center, Vec3::Y).expect("valid camera basis");
        assert!(view.transform_point3(eye).abs_diff_eq(Vec3::ZERO, EPSILON));
        assert!(view
            .transform_point3(center)
            .abs_diff_eq(Vec3::new(0.0, 0.0, -1.0), EPSILON));
    }

    #[test]
    fn checked_view_rejects_degenerate_and_non_finite_bases() {
        assert_eq!(try_view_look_at_rh(Vec3::ZERO, Vec3::ZERO, Vec3::Y), None);
        assert_eq!(try_view_look_at_rh(Vec3::ZERO, Vec3::Z, Vec3::Z), None);
        assert_eq!(
            try_view_look_at_rh(Vec3::new(f32::NAN, 0.0, 0.0), Vec3::Z, Vec3::Y),
            None
        );
    }

    #[test]
    fn perspective_retains_gl_clip_depth() {
        let near = 0.1;
        let far = 100.0;
        let projection = try_projection_perspective_rh_gl(1.0, 16.0 / 9.0, near, far)
            .expect("valid perspective");
        let near_clip = projection * Vec4::new(0.0, 0.0, -near, 1.0);
        let far_clip = projection * Vec4::new(0.0, 0.0, -far, 1.0);
        assert_near(near_clip.z / near_clip.w, -1.0);
        assert_near(far_clip.z / far_clip.w, 1.0);
    }

    #[test]
    fn perspective_rejects_invalid_parameters() {
        assert_eq!(try_projection_perspective_rh_gl(0.0, 1.0, 0.1, 10.0), None);
        assert_eq!(try_projection_perspective_rh_gl(1.0, 0.0, 0.1, 10.0), None);
        assert_eq!(try_projection_perspective_rh_gl(1.0, 1.0, 1.0, 1.0), None);
    }

    #[test]
    fn orthographic_retains_gl_clip_depth() {
        let projection = try_projection_orthographic_rh_gl(-2.0, 2.0, -1.0, 1.0, 0.1, 10.0)
            .expect("valid orthographic projection");
        let near_clip = projection * Vec4::new(0.0, 0.0, -0.1, 1.0);
        let far_clip = projection * Vec4::new(0.0, 0.0, -10.0, 1.0);
        assert_near(near_clip.z / near_clip.w, -1.0);
        assert_near(far_clip.z / far_clip.w, 1.0);
    }

    #[test]
    fn orthographic_rejects_invalid_parameters() {
        assert_eq!(
            try_projection_orthographic_rh_gl(1.0, -1.0, -1.0, 1.0, 0.1, 10.0),
            None
        );
        assert_eq!(
            try_projection_orthographic_rh_gl(-1.0, 1.0, -1.0, 1.0, 1.0, 1.0),
            None
        );
    }
}
