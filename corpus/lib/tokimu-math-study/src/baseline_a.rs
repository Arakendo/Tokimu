//! Alternative A: the currently shipped direct `glam` vocabulary.
//!
//! The values reported here are observations from the selected provider and
//! target. They are not a new Tokimu representation guarantee.

use std::mem::{align_of, size_of};

use tokimu_core::math::{Mat4, Quat, Vec2, Vec3, Vec4};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct TypeLayoutObservation {
    pub type_name: &'static str,
    pub size_bytes: usize,
    pub alignment_bytes: usize,
}

pub fn type_layouts() -> [TypeLayoutObservation; 5] {
    [
        TypeLayoutObservation {
            type_name: "Vec2",
            size_bytes: size_of::<Vec2>(),
            alignment_bytes: align_of::<Vec2>(),
        },
        TypeLayoutObservation {
            type_name: "Vec3",
            size_bytes: size_of::<Vec3>(),
            alignment_bytes: align_of::<Vec3>(),
        },
        TypeLayoutObservation {
            type_name: "Vec4",
            size_bytes: size_of::<Vec4>(),
            alignment_bytes: align_of::<Vec4>(),
        },
        TypeLayoutObservation {
            type_name: "Quat",
            size_bytes: size_of::<Quat>(),
            alignment_bytes: align_of::<Quat>(),
        },
        TypeLayoutObservation {
            type_name: "Mat4",
            size_bytes: size_of::<Mat4>(),
            alignment_bytes: align_of::<Mat4>(),
        },
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tokimu_types_are_the_current_glam_types() {
        fn accepts_glam_vec2(_: glam::Vec2) {}
        fn accepts_glam_vec3(_: glam::Vec3) {}
        fn accepts_glam_vec4(_: glam::Vec4) {}
        fn accepts_glam_quat(_: glam::Quat) {}
        fn accepts_glam_mat4(_: glam::Mat4) {}

        accepts_glam_vec2(Vec2::ZERO);
        accepts_glam_vec3(Vec3::ZERO);
        accepts_glam_vec4(Vec4::ZERO);
        accepts_glam_quat(Quat::IDENTITY);
        accepts_glam_mat4(Mat4::IDENTITY);
    }

    #[test]
    fn baseline_records_one_layout_per_public_type() {
        let layouts = type_layouts();

        assert_eq!(
            layouts.map(|layout| layout.type_name),
            ["Vec2", "Vec3", "Vec4", "Quat", "Mat4"]
        );
        assert!(layouts.iter().all(|layout| layout.size_bytes > 0));
        assert!(layouts.iter().all(|layout| layout.alignment_bytes > 0));
    }
}
