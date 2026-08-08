//! Corpus-local port of `hello-asteroids`' orthographic camera construction.
//!
//! This exercises the renderer-owned 2D camera policy, including its explicit
//! zero-height aspect fallback. It does not migrate application state, input,
//! or the renderer API.

use crate::{alternative_b::Mat4 as BMat4, alternative_c::Mat4 as CMat4, migration_b, migration_c};
use tokimu_core::math::Mat4 as AMat4;

#[must_use]
pub fn camera_with_a(width: f32, height: f32, world_height: f32) -> tokimu::Camera {
    tokimu::Camera::orthographic_2d_with_height(width, height, world_height)
}

/// Forms the current renderer camera from B's candidate matrices.
#[must_use]
pub fn camera_with_b(width: f32, height: f32, world_height: f32) -> tokimu::Camera {
    let (left, right, bottom, top) = orthographic_bounds(width, height, world_height);
    tokimu::Camera::new(
        migration_b::provider_upload_matrix(BMat4::IDENTITY),
        migration_b::provider_upload_matrix(BMat4::orthographic_rh_gl(
            left, right, bottom, top, -1.0, 1.0,
        )),
    )
}

/// Forms the current renderer camera from C's candidate matrices.
#[must_use]
pub fn camera_with_c(width: f32, height: f32, world_height: f32) -> tokimu::Camera {
    let (left, right, bottom, top) = orthographic_bounds(width, height, world_height);
    tokimu::Camera::new(
        migration_c::provider_upload_matrix(CMat4::IDENTITY),
        migration_c::provider_upload_matrix(CMat4::orthographic_rh_gl(
            left, right, bottom, top, -1.0, 1.0,
        )),
    )
}

fn orthographic_bounds(width: f32, height: f32, world_height: f32) -> (f32, f32, f32, f32) {
    let aspect_ratio = if height > 0.0 { width / height } else { 1.0 };
    let half_height = world_height * 0.5;
    let half_width = half_height * aspect_ratio;
    (-half_width, half_width, -half_height, half_height)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn assert_camera_near(left: tokimu::Camera, right: tokimu::Camera) {
        for (left, right) in left
            .view
            .to_cols_array()
            .into_iter()
            .chain(left.projection.to_cols_array())
            .zip(
                right
                    .view
                    .to_cols_array()
                    .into_iter()
                    .chain(right.projection.to_cols_array()),
            )
        {
            assert!((left - right).abs() <= 1.0e-6, "{left} != {right}");
        }
    }

    #[test]
    fn candidates_match_the_hello_asteroids_orthographic_camera() {
        let a = camera_with_a(1280.0, 720.0, 24.0);
        assert_camera_near(a, camera_with_b(1280.0, 720.0, 24.0));
        assert_camera_near(a, camera_with_c(1280.0, 720.0, 24.0));
        assert_ne!(a.projection, AMat4::IDENTITY);
    }

    #[test]
    fn candidates_retain_the_current_zero_height_aspect_fallback() {
        let a = camera_with_a(1280.0, 0.0, 24.0);
        assert_camera_near(a, camera_with_b(1280.0, 0.0, 24.0));
        assert_camera_near(a, camera_with_c(1280.0, 0.0, 24.0));
    }
}
