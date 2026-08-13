//! Corpus-local port of the E1M1 source-observer camera preparation.
//!
//! The Doom source embedding and observer meaning remain caller-owned. This
//! port measures only the ordinary vector/matrix mechanics beneath the
//! currently selected PreserveNorth source-to-world convention.

use crate::{
    alternative_b::{Mat4 as BMat4, Vec3 as BVec3},
    alternative_c::{Mat4 as CMat4, Vec3 as CVec3},
};
use tokimu_core::math::{Mat4 as AMat4, Vec3 as AVec3};

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct DoomObserverCamera {
    pub position: [f32; 3],
    pub forward: [f32; 3],
    pub view_projection_columns: [f32; 16],
}

#[must_use]
pub fn observer_camera_with_a(source_xy: [f32; 2], yaw: f32, pitch: f32) -> DoomObserverCamera {
    let position = AVec3::new(-source_xy[0], 36.0, source_xy[1]);
    let forward = AVec3::new(
        yaw.cos() * pitch.cos(),
        pitch.sin(),
        yaw.sin() * pitch.cos(),
    )
    .normalize();
    let view = AMat4::look_at_rh(position, position + forward * 128.0, AVec3::Y);
    let projection = AMat4::perspective_rh_gl(75.0_f32.to_radians(), 16.0 / 9.0, 0.1, 4096.0);
    DoomObserverCamera {
        position: position.to_array(),
        forward: forward.to_array(),
        view_projection_columns: (projection * view).to_cols_array(),
    }
}

#[must_use]
pub fn observer_camera_with_b(source_xy: [f32; 2], yaw: f32, pitch: f32) -> DoomObserverCamera {
    let position = BVec3::new(-source_xy[0], 36.0, source_xy[1]);
    let forward = BVec3::new(
        yaw.cos() * pitch.cos(),
        pitch.sin(),
        yaw.sin() * pitch.cos(),
    )
    .normalize();
    let view = BMat4::look_at_rh(position, position + forward * 128.0, BVec3::Y);
    let projection = BMat4::perspective_rh_gl(75.0_f32.to_radians(), 16.0 / 9.0, 0.1, 4096.0);
    DoomObserverCamera {
        position: position.to_array(),
        forward: forward.to_array(),
        view_projection_columns: (projection * view).to_cols_array(),
    }
}

#[must_use]
pub fn observer_camera_with_c(source_xy: [f32; 2], yaw: f32, pitch: f32) -> DoomObserverCamera {
    let position = CVec3::new(-source_xy[0], 36.0, source_xy[1]);
    let forward = CVec3::new(
        yaw.cos() * pitch.cos(),
        pitch.sin(),
        yaw.sin() * pitch.cos(),
    )
    .normalize();
    let view = CMat4::look_at_rh(position, position + forward * 128.0, CVec3::Y);
    let projection = CMat4::perspective_rh_gl(75.0_f32.to_radians(), 16.0 / 9.0, 0.1, 4096.0);
    DoomObserverCamera {
        position: position.to_array(),
        forward: forward.to_array(),
        view_projection_columns: (projection * view).to_cols_array(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn assert_camera_near(actual: DoomObserverCamera, expected: DoomObserverCamera) {
        for (actual, expected) in actual
            .position
            .into_iter()
            .chain(actual.forward)
            .chain(actual.view_projection_columns)
            .zip(
                expected
                    .position
                    .into_iter()
                    .chain(expected.forward)
                    .chain(expected.view_projection_columns),
            )
        {
            assert!(
                (actual - expected).abs() <= 1.0e-4,
                "{actual} != {expected}"
            );
        }
    }

    #[test]
    fn candidates_match_the_e1m1_preserve_north_observer_port() {
        let input = ([1056.0, -3616.0], 0.0, 0.0);
        let baseline = observer_camera_with_a(input.0, input.1, input.2);
        assert_camera_near(observer_camera_with_b(input.0, input.1, input.2), baseline);
        assert_camera_near(observer_camera_with_c(input.0, input.1, input.2), baseline);
    }
}
