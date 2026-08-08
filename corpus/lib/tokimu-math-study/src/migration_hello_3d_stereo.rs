//! Corpus-local port of `hello-3d-stereo`'s two-camera construction path.
//!
//! This retains the distinct public-surface shape: two independently formed
//! views and projections cross into the current provider-valued renderer
//! camera. It intentionally adds no math operation beyond the reviewed
//! `Vec3`/`Mat4` inventory.

use crate::{
    alternative_b::{Mat4 as BMat4, Vec3 as BVec3},
    alternative_c::{Mat4 as CMat4, Vec3 as CVec3},
    migration_b, migration_c,
};
use tokimu_core::math::{Mat4 as AMat4, Vec3 as AVec3};

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct StereoViewProjections {
    pub left: [f32; 16],
    pub right: [f32; 16],
}

#[must_use]
pub fn stereo_with_a(seconds: f32, width: f32, height: f32) -> StereoViewProjections {
    let (left_eye, right_eye) = eyes_with_a(seconds);
    let mut left_camera = tokimu::Camera::perspective_3d(width * 0.5, height);
    left_camera.view = AMat4::look_at_rh(left_eye, AVec3::ZERO, AVec3::Y);
    let mut right_camera = tokimu::Camera::perspective_3d(width * 0.5, height);
    right_camera.view = AMat4::look_at_rh(right_eye, AVec3::ZERO, AVec3::Y);

    StereoViewProjections {
        left: (left_camera.projection * left_camera.view).to_cols_array(),
        right: (right_camera.projection * right_camera.view).to_cols_array(),
    }
}

#[must_use]
pub fn stereo_with_b(seconds: f32, width: f32, height: f32) -> StereoViewProjections {
    let (left_eye, right_eye) = eyes_with_b(seconds);
    let left_camera = migration_b::renderer_camera(migration_b::Camera {
        eye: left_eye,
        center: BVec3::ZERO,
        up: BVec3::Y,
        vertical_fov_radians: 60.0_f32.to_radians(),
        aspect_ratio: (width * 0.5) / height,
        near: 0.1,
        far: 100.0,
    });
    let right_camera = migration_b::renderer_camera(migration_b::Camera {
        eye: right_eye,
        center: BVec3::ZERO,
        up: BVec3::Y,
        vertical_fov_radians: 60.0_f32.to_radians(),
        aspect_ratio: (width * 0.5) / height,
        near: 0.1,
        far: 100.0,
    });

    StereoViewProjections {
        left: (left_camera.projection * left_camera.view).to_cols_array(),
        right: (right_camera.projection * right_camera.view).to_cols_array(),
    }
}

#[must_use]
pub fn stereo_with_c(seconds: f32, width: f32, height: f32) -> StereoViewProjections {
    let (left_eye, right_eye) = eyes_with_c(seconds);
    let left_camera = migration_c::renderer_camera(migration_c::Camera {
        eye: left_eye,
        center: CVec3::ZERO,
        up: CVec3::Y,
        vertical_fov_radians: 60.0_f32.to_radians(),
        aspect_ratio: (width * 0.5) / height,
        near: 0.1,
        far: 100.0,
    });
    let right_camera = migration_c::renderer_camera(migration_c::Camera {
        eye: right_eye,
        center: CVec3::ZERO,
        up: CVec3::Y,
        vertical_fov_radians: 60.0_f32.to_radians(),
        aspect_ratio: (width * 0.5) / height,
        near: 0.1,
        far: 100.0,
    });

    StereoViewProjections {
        left: (left_camera.projection * left_camera.view).to_cols_array(),
        right: (right_camera.projection * right_camera.view).to_cols_array(),
    }
}

fn eyes_with_a(seconds: f32) -> (AVec3, AVec3) {
    let center_eye = center_eye_with_a(seconds);
    let right = (AVec3::ZERO - center_eye)
        .normalize()
        .cross(AVec3::Y)
        .normalize();
    (center_eye - right * 0.14, center_eye + right * 0.14)
}

fn eyes_with_b(seconds: f32) -> (BVec3, BVec3) {
    let center_eye = center_eye_with_b(seconds);
    let right = (BVec3::ZERO - center_eye)
        .normalize()
        .cross(BVec3::Y)
        .normalize();
    (center_eye - right * 0.14, center_eye + right * 0.14)
}

fn eyes_with_c(seconds: f32) -> (CVec3, CVec3) {
    let center_eye = center_eye_with_c(seconds);
    let right = (CVec3::ZERO - center_eye)
        .normalize()
        .cross(CVec3::Y)
        .normalize();
    (center_eye - right * 0.14, center_eye + right * 0.14)
}

fn center_eye_with_a(seconds: f32) -> AVec3 {
    let angle = seconds * 0.8;
    AVec3::new(
        angle.cos() * 3.0,
        0.35 + (seconds * 1.3).sin() * 0.15,
        angle.sin() * 3.0,
    )
}

fn center_eye_with_b(seconds: f32) -> BVec3 {
    let angle = seconds * 0.8;
    BVec3::new(
        angle.cos() * 3.0,
        0.35 + (seconds * 1.3).sin() * 0.15,
        angle.sin() * 3.0,
    )
}

fn center_eye_with_c(seconds: f32) -> CVec3 {
    let angle = seconds * 0.8;
    CVec3::new(
        angle.cos() * 3.0,
        0.35 + (seconds * 1.3).sin() * 0.15,
        angle.sin() * 3.0,
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    fn assert_stereo_near(left: StereoViewProjections, right: StereoViewProjections) {
        for (left, right) in left
            .left
            .into_iter()
            .chain(left.right)
            .zip(right.left.into_iter().chain(right.right).into_iter())
        {
            assert!((left - right).abs() <= 1.0e-6, "{left} != {right}");
        }
    }

    #[test]
    fn candidates_match_the_hello_3d_stereo_two_camera_path() {
        let a = stereo_with_a(1.25, 1280.0, 720.0);
        assert_stereo_near(a, stereo_with_b(1.25, 1280.0, 720.0));
        assert_stereo_near(a, stereo_with_c(1.25, 1280.0, 720.0));
        assert_ne!(a.left, a.right);
    }
}
