//! Corpus-local ports of `hello-glb`'s model and floor transform paths.
//!
//! This preserves the caller's composed rotation, non-uniform scale,
//! translation, inverse-transpose normal handling, and zero-safe normal
//! normalization without changing the original GLB application.

use crate::{
    alternative_b::{Mat4 as BMat4, Vec3 as BVec3},
    alternative_c::{Mat4 as CMat4, Vec3 as CVec3},
    migration_hello_3d_mono::TransformedMesh,
};
use tokimu_core::math::{Mat4 as AMat4, Vec3 as AVec3};

#[must_use]
pub fn model_with_a(seconds: f32, positions: &[[f32; 3]], normals: &[[f32; 3]]) -> TransformedMesh {
    let wobble = (seconds * 1.4).sin() * 0.06;
    let transform = AMat4::from_rotation_y(seconds * 0.18)
        * AMat4::from_rotation_x((seconds * 0.7).sin() * 0.15)
        * AMat4::from_scale(AVec3::new(
            1.0 + wobble * 0.5,
            1.0 + wobble,
            1.0 + wobble * 0.25,
        ))
        * AMat4::from_translation(AVec3::new(0.0, 0.35, 0.0));
    transform_mesh_a(transform, positions, normals)
}

#[must_use]
pub fn model_with_b(seconds: f32, positions: &[[f32; 3]], normals: &[[f32; 3]]) -> TransformedMesh {
    let wobble = (seconds * 1.4).sin() * 0.06;
    let transform = BMat4::from_rotation_y(seconds * 0.18)
        * BMat4::from_rotation_x((seconds * 0.7).sin() * 0.15)
        * BMat4::from_scale(BVec3::new(
            1.0 + wobble * 0.5,
            1.0 + wobble,
            1.0 + wobble * 0.25,
        ))
        * BMat4::from_translation(BVec3::new(0.0, 0.35, 0.0));
    transform_mesh_b(transform, positions, normals)
}

#[must_use]
pub fn model_with_c(seconds: f32, positions: &[[f32; 3]], normals: &[[f32; 3]]) -> TransformedMesh {
    let wobble = (seconds * 1.4).sin() * 0.06;
    let transform = CMat4::from_rotation_y(seconds * 0.18)
        * CMat4::from_rotation_x((seconds * 0.7).sin() * 0.15)
        * CMat4::from_scale(CVec3::new(
            1.0 + wobble * 0.5,
            1.0 + wobble,
            1.0 + wobble * 0.25,
        ))
        * CMat4::from_translation(CVec3::new(0.0, 0.35, 0.0));
    transform_mesh_c(transform, positions, normals)
}

#[must_use]
pub fn floor_with_a(seconds: f32, positions: &[[f32; 3]], normals: &[[f32; 3]]) -> TransformedMesh {
    let pulse = 0.02 + seconds.sin().abs() * 0.01;
    transform_mesh_a(
        AMat4::from_translation(AVec3::new(0.0, -0.8, 0.0))
            * AMat4::from_scale(AVec3::new(8.0, pulse, 8.0)),
        positions,
        normals,
    )
}

#[must_use]
pub fn floor_with_b(seconds: f32, positions: &[[f32; 3]], normals: &[[f32; 3]]) -> TransformedMesh {
    let pulse = 0.02 + seconds.sin().abs() * 0.01;
    transform_mesh_b(
        BMat4::from_translation(BVec3::new(0.0, -0.8, 0.0))
            * BMat4::from_scale(BVec3::new(8.0, pulse, 8.0)),
        positions,
        normals,
    )
}

#[must_use]
pub fn floor_with_c(seconds: f32, positions: &[[f32; 3]], normals: &[[f32; 3]]) -> TransformedMesh {
    let pulse = 0.02 + seconds.sin().abs() * 0.01;
    transform_mesh_c(
        CMat4::from_translation(CVec3::new(0.0, -0.8, 0.0))
            * CMat4::from_scale(CVec3::new(8.0, pulse, 8.0)),
        positions,
        normals,
    )
}

fn transform_mesh_a(
    transform: AMat4,
    positions: &[[f32; 3]],
    normals: &[[f32; 3]],
) -> TransformedMesh {
    let normal_transform = transform.inverse().transpose();
    TransformedMesh {
        positions: positions
            .iter()
            .map(|position| {
                transform
                    .transform_point3(AVec3::from_array(*position))
                    .to_array()
            })
            .collect(),
        normals: normals
            .iter()
            .map(|normal| {
                normal_transform
                    .transform_vector3(AVec3::from_array(*normal))
                    .normalize_or_zero()
                    .to_array()
            })
            .collect(),
    }
}

fn transform_mesh_b(
    transform: BMat4,
    positions: &[[f32; 3]],
    normals: &[[f32; 3]],
) -> TransformedMesh {
    let normal_transform = transform.inverse().transpose();
    TransformedMesh {
        positions: positions
            .iter()
            .map(|position| {
                transform
                    .transform_point3(BVec3::from_array(*position))
                    .to_array()
            })
            .collect(),
        normals: normals
            .iter()
            .map(|normal| {
                normal_transform
                    .transform_vector3(BVec3::from_array(*normal))
                    .normalize_or_zero()
                    .to_array()
            })
            .collect(),
    }
}

fn transform_mesh_c(
    transform: CMat4,
    positions: &[[f32; 3]],
    normals: &[[f32; 3]],
) -> TransformedMesh {
    let normal_transform = transform.inverse().transpose();
    TransformedMesh {
        positions: positions
            .iter()
            .map(|position| {
                transform
                    .transform_point3(CVec3::from_array(*position))
                    .to_array()
            })
            .collect(),
        normals: normals
            .iter()
            .map(|normal| {
                normal_transform
                    .transform_vector3(CVec3::from_array(*normal))
                    .normalize_or_zero()
                    .to_array()
            })
            .collect(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn assert_mesh_near(left: &TransformedMesh, right: &TransformedMesh) {
        for (left, right) in left
            .positions
            .iter()
            .chain(&left.normals)
            .zip(right.positions.iter().chain(&right.normals))
        {
            for (left, right) in left.iter().zip(right) {
                assert!((left - right).abs() <= 1.0e-5, "{left} != {right}");
            }
        }
    }

    #[test]
    fn candidates_match_hello_glb_model_and_floor_transform_paths() {
        let positions = [[-1.0, -1.0, -1.0], [1.0, 1.0, 1.0]];
        let normals = [[0.0, 1.0, 0.0], [0.0, 0.0, 1.0]];

        for (a, b, c) in [
            (
                model_with_a(1.25, &positions, &normals),
                model_with_b(1.25, &positions, &normals),
                model_with_c(1.25, &positions, &normals),
            ),
            (
                floor_with_a(1.25, &positions, &normals),
                floor_with_b(1.25, &positions, &normals),
                floor_with_c(1.25, &positions, &normals),
            ),
        ] {
            assert_mesh_near(&a, &b);
            assert_mesh_near(&a, &c);
        }
    }
}
