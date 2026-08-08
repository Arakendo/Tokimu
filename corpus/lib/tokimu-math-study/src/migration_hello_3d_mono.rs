//! Corpus-local ports of `hello-3d-mono`'s rotating-cube math path.
//!
//! The original application remains unchanged. This fixture retains only the
//! position/normal transform behavior needed to compare candidate math APIs.

use crate::{
    alternative_b::{Mat4 as BMat4, Vec3 as BVec3},
    alternative_c::{Mat4 as CMat4, Vec3 as CVec3},
};
use tokimu_core::math::{Mat4 as AMat4, Vec3 as AVec3};

#[derive(Clone, Debug, PartialEq)]
pub struct TransformedMesh {
    pub positions: Vec<[f32; 3]>,
    pub normals: Vec<[f32; 3]>,
}

#[must_use]
pub fn spin_with_a(seconds: f32, positions: &[[f32; 3]], normals: &[[f32; 3]]) -> TransformedMesh {
    let yaw = seconds * 0.7;
    let pitch = seconds * 0.45;
    let roll = seconds * 0.25;
    let transform =
        AMat4::from_rotation_y(yaw) * AMat4::from_rotation_x(pitch) * AMat4::from_rotation_z(roll);

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
                transform
                    .transform_vector3(AVec3::from_array(*normal))
                    .normalize()
                    .to_array()
            })
            .collect(),
    }
}

#[must_use]
pub fn spin_with_b(seconds: f32, positions: &[[f32; 3]], normals: &[[f32; 3]]) -> TransformedMesh {
    let yaw = seconds * 0.7;
    let pitch = seconds * 0.45;
    let roll = seconds * 0.25;
    let transform =
        BMat4::from_rotation_y(yaw) * BMat4::from_rotation_x(pitch) * BMat4::from_rotation_z(roll);

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
                transform
                    .transform_vector3(BVec3::from_array(*normal))
                    .normalize()
                    .to_array()
            })
            .collect(),
    }
}

#[must_use]
pub fn spin_with_c(seconds: f32, positions: &[[f32; 3]], normals: &[[f32; 3]]) -> TransformedMesh {
    let yaw = seconds * 0.7;
    let pitch = seconds * 0.45;
    let roll = seconds * 0.25;
    let transform =
        CMat4::from_rotation_y(yaw) * CMat4::from_rotation_x(pitch) * CMat4::from_rotation_z(roll);

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
                transform
                    .transform_vector3(CVec3::from_array(*normal))
                    .normalize()
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
            for (left, right) in left.into_iter().zip(right) {
                assert!((left - right).abs() <= 1.0e-6, "{left} != {right}");
            }
        }
    }

    #[test]
    fn candidates_match_the_hello_3d_mono_baseline_transform_path() {
        let positions = [[-1.0, -1.0, -1.0], [1.0, 1.0, 1.0]];
        let normals = [[0.0, 1.0, 0.0], [0.0, 0.0, 1.0]];

        let a = spin_with_a(1.25, &positions, &normals);
        let b = spin_with_b(1.25, &positions, &normals);
        let c = spin_with_c(1.25, &positions, &normals);

        assert_mesh_near(&a, &b);
        assert_mesh_near(&a, &c);
    }
}
