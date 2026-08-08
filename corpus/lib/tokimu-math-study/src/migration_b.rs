//! Representative Alternative B migration pressure.
//!
//! This is deliberately renderer-shaped, but remains corpus-local. It shows
//! where a provider conversion belongs when a candidate Tokimu math contract
//! meets a provider-specific rendering upload boundary.

use crate::alternative_b::{Mat4, Vec3};

/// Candidate-facing camera state with no foreign public type.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Camera {
    pub eye: Vec3,
    pub center: Vec3,
    pub up: Vec3,
    pub vertical_fov_radians: f32,
    pub aspect_ratio: f32,
    pub near: f32,
    pub far: f32,
}

impl Camera {
    #[must_use]
    pub fn view_projection(self) -> Mat4 {
        Mat4::perspective_rh_gl(
            self.vertical_fov_radians,
            self.aspect_ratio,
            self.near,
            self.far,
        ) * Mat4::look_at_rh(self.eye, self.center, self.up)
    }
}

/// The one counted conversion in this representative provider upload path.
///
/// This stays crate-private because `glam::Mat4` is evidence of a rendering
/// adapter boundary, not a candidate math contract.
#[must_use]
pub(crate) const fn provider_upload_matrix(transform: Mat4) -> glam::Mat4 {
    transform.into_provider()
}

/// Constructs the current public renderer camera at the explicit B boundary.
///
/// The facade presently exposes provider `Mat4` fields, so this handoff needs
/// two private conversions: one for view and one for projection. The candidate
/// camera itself remains provider-free.
#[must_use]
pub(crate) fn renderer_camera(camera: Camera) -> tokimu::Camera {
    let view = Mat4::look_at_rh(camera.eye, camera.center, camera.up);
    let projection = Mat4::perspective_rh_gl(
        camera.vertical_fov_radians,
        camera.aspect_ratio,
        camera.near,
        camera.far,
    );

    tokimu::Camera::new(
        provider_upload_matrix(view),
        provider_upload_matrix(projection),
    )
}

/// Repeats the representative upload conversion and returns a checksum.
#[must_use]
pub fn provider_upload_workload(iterations: u32) -> f32 {
    let transform = Camera {
        eye: Vec3::new(0.0, 0.0, 5.0),
        center: Vec3::ZERO,
        up: Vec3::Y,
        vertical_fov_radians: 1.0,
        aspect_ratio: 16.0 / 9.0,
        near: 0.1,
        far: 100.0,
    }
    .view_projection();
    let mut checksum = 0.0;

    for _ in 0..iterations {
        let uploaded = provider_upload_matrix(core::hint::black_box(transform));
        checksum += core::hint::black_box(uploaded.w_axis.x);
    }

    checksum
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn candidate_camera_needs_one_explicit_provider_upload_conversion() {
        let camera = Camera {
            eye: Vec3::new(0.0, 0.0, 5.0),
            center: Vec3::ZERO,
            up: Vec3::Y,
            vertical_fov_radians: 1.0,
            aspect_ratio: 16.0 / 9.0,
            near: 0.1,
            far: 100.0,
        };

        let candidate_matrix = camera.view_projection();
        let provider_matrix = provider_upload_matrix(candidate_matrix);

        assert_eq!(
            candidate_matrix.to_cols_array(),
            provider_matrix.to_cols_array()
        );
    }

    #[test]
    fn candidate_camera_handoff_to_the_current_renderer_is_private_and_explicit() {
        let camera = Camera {
            eye: Vec3::new(0.0, 0.0, 5.0),
            center: Vec3::ZERO,
            up: Vec3::Y,
            vertical_fov_radians: 1.0,
            aspect_ratio: 16.0 / 9.0,
            near: 0.1,
            far: 100.0,
        };

        let renderer_camera = renderer_camera(camera);

        assert_eq!(
            (renderer_camera.projection * renderer_camera.view).to_cols_array(),
            provider_upload_matrix(camera.view_projection()).to_cols_array()
        );
    }

    #[test]
    fn renderer_boundary_round_trips_a_candidate_matrix_without_public_leakage() {
        let candidate = Camera {
            eye: Vec3::new(2.0, 1.0, 5.0),
            center: Vec3::new(0.0, 0.25, 0.0),
            up: Vec3::Y,
            vertical_fov_radians: 1.0,
            aspect_ratio: 16.0 / 9.0,
            near: 0.1,
            far: 100.0,
        }
        .view_projection();

        let restored = Mat4::from_provider(provider_upload_matrix(candidate));

        assert_eq!(restored.to_cols_array(), candidate.to_cols_array());
    }
}
