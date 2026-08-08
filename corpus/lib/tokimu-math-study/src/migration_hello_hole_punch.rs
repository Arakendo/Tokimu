//! Corpus-local port of `hello-hole-punch` node-transform resolution.
//!
//! The bounded path retains glTF column-array input, an animation translation
//! override, and parent-to-child composition. It deliberately excludes scene
//! traversal, animation scheduling, mesh lowering, and renderer integration.

use crate::{
    alternative_b::{Mat4 as BMat4, Vec3 as BVec3, Vec4 as BVec4},
    alternative_c::{Mat4 as CMat4, Vec3 as CVec3, Vec4 as CVec4},
};
use tokimu_core::math::{Mat4 as AMat4, Vec3 as AVec3, Vec4 as AVec4};

#[must_use]
pub fn resolve_two_node_world_with_a(
    parent_local: &[f32; 16],
    child_local: &[f32; 16],
    child_translation: Option<[f32; 3]>,
) -> [f32; 16] {
    let parent = AMat4::from_cols_array(parent_local);
    let mut child = AMat4::from_cols_array(child_local);
    if let Some([x, y, z]) = child_translation {
        child.w_axis = AVec4::new(x, y, z, 1.0);
    }
    (parent * child).to_cols_array()
}

#[must_use]
pub fn resolve_two_node_world_with_b(
    parent_local: &[f32; 16],
    child_local: &[f32; 16],
    child_translation: Option<[f32; 3]>,
) -> [f32; 16] {
    let parent = BMat4::from_cols_array(parent_local);
    let mut child = BMat4::from_cols_array(child_local);
    if let Some([x, y, z]) = child_translation {
        child.set_w_axis(BVec4::new(x, y, z, 1.0));
    }
    (parent * child).to_cols_array()
}

#[must_use]
pub fn resolve_two_node_world_with_c(
    parent_local: &[f32; 16],
    child_local: &[f32; 16],
    child_translation: Option<[f32; 3]>,
) -> [f32; 16] {
    let parent = CMat4::from_cols_array(parent_local);
    let mut child = CMat4::from_cols_array(child_local);
    if let Some([x, y, z]) = child_translation {
        child.set_w_axis(CVec4::new(x, y, z, 1.0));
    }
    (parent * child).to_cols_array()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn assert_matrix_near(actual: [f32; 16], expected: [f32; 16]) {
        for (actual, expected) in actual.into_iter().zip(expected) {
            assert!(
                (actual - expected).abs() <= 1.0e-5,
                "{actual} != {expected}"
            );
        }
    }

    #[test]
    fn candidates_match_the_two_node_hello_hole_punch_resolution_path() {
        let parent = AMat4::from_rotation_y(0.3).to_cols_array();
        let child = AMat4::from_translation(AVec3::new(8.0, 9.0, 10.0)).to_cols_array();

        for translation in [None, Some([1.25, -0.5, 3.75])] {
            let baseline = resolve_two_node_world_with_a(&parent, &child, translation);
            assert_matrix_near(
                resolve_two_node_world_with_b(&parent, &child, translation),
                baseline,
            );
            assert_matrix_near(
                resolve_two_node_world_with_c(&parent, &child, translation),
                baseline,
            );
        }
    }
}
