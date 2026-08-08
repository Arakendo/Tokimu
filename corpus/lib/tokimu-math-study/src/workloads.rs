//! Deterministic workloads shared by the case-study alternatives.
//!
//! The functions return a checksum so an executable measurement run also
//! verifies that the compared candidates performed equivalent visible work.
//! Timings are deliberately reported by the caller, alongside its target,
//! profile, toolchain, and host metadata; this module makes no universal
//! performance claim.

use crate::alternative_b::{Mat4 as CandidateMat4, Vec3 as CandidateVec3};
use crate::alternative_c::{Mat4 as OwnedMat4, Vec3 as OwnedVec3};
use crate::migration_hello_3d_stereo::{stereo_with_a, stereo_with_b, stereo_with_c};
use tokimu_core::math::{Mat4 as BaselineMat4, Vec3 as BaselineVec3};

/// Runs a transform-heavy workload through the current direct-provider API.
#[must_use]
pub fn baseline_transform_workload(iterations: u32) -> [f32; 3] {
    let transform = BaselineMat4::from_rotation_y(0.01)
        * BaselineMat4::from_translation(BaselineVec3::new(0.001, -0.002, 0.003));
    let mut value = BaselineVec3::ONE;
    let mut checksum = BaselineVec3::ZERO;

    for _ in 0..iterations {
        value = transform.transform_point3(value);
        checksum = checksum + value;
    }

    checksum.to_array()
}

/// Runs the identical transform-heavy workload through Alternative B.
#[must_use]
pub fn provider_backed_transform_workload(iterations: u32) -> [f32; 3] {
    let transform = CandidateMat4::from_rotation_y(0.01)
        * CandidateMat4::from_translation(CandidateVec3::new(0.001, -0.002, 0.003));
    let mut value = CandidateVec3::ONE;
    let mut checksum = CandidateVec3::ZERO;

    for _ in 0..iterations {
        value = transform.transform_point3(value);
        checksum = checksum + value;
    }

    checksum.to_array()
}

/// Runs the identical transform-heavy workload through Alternative C.
#[must_use]
pub fn owned_transform_workload(iterations: u32) -> [f32; 3] {
    let transform = OwnedMat4::from_rotation_y(0.01)
        * OwnedMat4::from_translation(OwnedVec3::new(0.001, -0.002, 0.003));
    let mut value = OwnedVec3::ONE;
    let mut checksum = OwnedVec3::ZERO;

    for _ in 0..iterations {
        value = transform.transform_point3(value);
        checksum = checksum + value;
    }

    checksum.to_array()
}

/// Builds the full current A stereo-camera pair repeatedly.
#[must_use]
pub fn baseline_stereo_camera_workload(iterations: u32) -> f32 {
    stereo_camera_workload(iterations, stereo_with_a)
}

/// Builds the B stereo-camera pair and crosses it into the current renderer.
#[must_use]
pub fn provider_backed_stereo_camera_workload(iterations: u32) -> f32 {
    stereo_camera_workload(iterations, stereo_with_b)
}

/// Builds the C stereo-camera pair and crosses it into the current renderer.
#[must_use]
pub fn owned_stereo_camera_workload(iterations: u32) -> f32 {
    stereo_camera_workload(iterations, stereo_with_c)
}

fn stereo_camera_workload(
    iterations: u32,
    build: fn(f32, f32, f32) -> crate::migration_hello_3d_stereo::StereoViewProjections,
) -> f32 {
    let mut checksum = 0.0;
    for frame in 0..iterations {
        let cameras = build(core::hint::black_box(frame as f32 * 0.016), 1280.0, 720.0);
        checksum += core::hint::black_box(cameras.left[0] + cameras.right[5]);
    }
    checksum
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn alternatives_produce_the_same_transform_workload_checksum() {
        assert_eq!(
            baseline_transform_workload(4_096),
            provider_backed_transform_workload(4_096)
        );
        assert_eq!(
            baseline_transform_workload(4_096),
            owned_transform_workload(4_096)
        );
    }

    #[test]
    fn alternatives_produce_near_stereo_camera_workload_checksums() {
        let baseline = baseline_stereo_camera_workload(4_096);
        let provider_backed = provider_backed_stereo_camera_workload(4_096);
        let owned = owned_stereo_camera_workload(4_096);

        assert!((baseline - provider_backed).abs() <= 1.0e-3);
        assert!((baseline - owned).abs() <= 1.0e-3);
    }
}
