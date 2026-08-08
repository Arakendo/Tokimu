//! Corpus-local port of `hello-fps-web` camera motion and hit-distance math.
//!
//! This preserves directional construction, zero-safe normalization, in-place
//! movement, fixed eye-height restoration, and target distance observation.
//! It excludes app state, input, renderer, and browser lifecycle behavior.

use crate::{alternative_b::Vec3 as BVec3, alternative_c::Vec3 as CVec3};
use tokimu_core::math::Vec3 as AVec3;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct FpsMotionSnapshot {
    pub camera_position: [f32; 3],
    pub forward: [f32; 3],
    pub right: [f32; 3],
    pub target_distance: f32,
}

#[must_use]
pub fn step_with_a(
    yaw: f32,
    pitch: f32,
    position: [f32; 3],
    target: [f32; 3],
) -> FpsMotionSnapshot {
    let forward = AVec3::new(
        yaw.sin() * pitch.cos(),
        pitch.sin(),
        yaw.cos() * pitch.cos(),
    )
    .normalize_or_zero();
    let flat_forward = AVec3::new(forward.x, 0.0, forward.z).normalize_or_zero();
    let right = flat_forward.cross(AVec3::Y).normalize_or_zero();
    let mut camera = AVec3::from_array(position);
    camera += flat_forward * 5.5 * 0.016 + right * -2.0 * 0.016;
    camera.y = 1.6;
    snapshot_a(camera, forward, right, AVec3::from_array(target))
}

#[must_use]
pub fn step_with_b(
    yaw: f32,
    pitch: f32,
    position: [f32; 3],
    target: [f32; 3],
) -> FpsMotionSnapshot {
    let forward = BVec3::new(
        yaw.sin() * pitch.cos(),
        pitch.sin(),
        yaw.cos() * pitch.cos(),
    )
    .normalize_or_zero();
    let flat_forward = BVec3::new(forward.x(), 0.0, forward.z()).normalize_or_zero();
    let right = flat_forward.cross(BVec3::Y).normalize_or_zero();
    let mut camera = BVec3::from_array(position);
    camera += flat_forward * 5.5 * 0.016 + right * -2.0 * 0.016;
    // `glam`'s mutable public field becomes a visible wrapper reconstruction.
    camera = BVec3::new(camera.x(), 1.6, camera.z());
    snapshot_b(camera, forward, right, BVec3::from_array(target))
}

#[must_use]
pub fn step_with_c(
    yaw: f32,
    pitch: f32,
    position: [f32; 3],
    target: [f32; 3],
) -> FpsMotionSnapshot {
    let forward = CVec3::new(
        yaw.sin() * pitch.cos(),
        pitch.sin(),
        yaw.cos() * pitch.cos(),
    )
    .normalize_or_zero();
    let flat_forward = CVec3::new(forward.x, 0.0, forward.z).normalize_or_zero();
    let right = flat_forward.cross(CVec3::Y).normalize_or_zero();
    let mut camera = CVec3::from_array(position);
    camera += flat_forward * 5.5 * 0.016 + right * -2.0 * 0.016;
    camera.y = 1.6;
    snapshot_c(camera, forward, right, CVec3::from_array(target))
}

fn snapshot_a(camera: AVec3, forward: AVec3, right: AVec3, target: AVec3) -> FpsMotionSnapshot {
    FpsMotionSnapshot {
        camera_position: camera.to_array(),
        forward: forward.to_array(),
        right: right.to_array(),
        target_distance: camera.distance(target),
    }
}

fn snapshot_b(camera: BVec3, forward: BVec3, right: BVec3, target: BVec3) -> FpsMotionSnapshot {
    FpsMotionSnapshot {
        camera_position: camera.to_array(),
        forward: forward.to_array(),
        right: right.to_array(),
        target_distance: camera.distance(target),
    }
}

fn snapshot_c(camera: CVec3, forward: CVec3, right: CVec3, target: CVec3) -> FpsMotionSnapshot {
    FpsMotionSnapshot {
        camera_position: camera.to_array(),
        forward: forward.to_array(),
        right: right.to_array(),
        target_distance: camera.distance(target),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn assert_near(actual: FpsMotionSnapshot, expected: FpsMotionSnapshot) {
        for (actual, expected) in actual
            .camera_position
            .into_iter()
            .chain(actual.forward)
            .chain(actual.right)
            .chain([actual.target_distance])
            .zip(
                expected
                    .camera_position
                    .into_iter()
                    .chain(expected.forward)
                    .chain(expected.right)
                    .chain([expected.target_distance]),
            )
        {
            assert!(
                (actual - expected).abs() <= 1.0e-5,
                "{actual} != {expected}"
            );
        }
    }

    #[test]
    fn candidates_match_the_hello_fps_motion_path() {
        let input = (0.4, -0.2, [0.0, 1.6, -4.0], [2.0, 0.85, 7.0]);
        let baseline = step_with_a(input.0, input.1, input.2, input.3);
        assert_near(step_with_b(input.0, input.1, input.2, input.3), baseline);
        assert_near(step_with_c(input.0, input.1, input.2, input.3), baseline);
    }
}
