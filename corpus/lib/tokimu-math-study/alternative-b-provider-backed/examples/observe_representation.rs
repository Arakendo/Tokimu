//! Corpus-only representation observations for Full B.
//!
//! These values describe this compiler, target, candidate, and selected private
//! provider. They are not a Tokimu ABI, SIMD, FFI, serialization, or GPU-buffer
//! contract.

use core::mem::{align_of, size_of};
use tokimu_math_study_provider_backed::{Mat4, Quat, Vec2, Vec3, Vec4};

fn observe_copy<T: Copy>(type_name: &str) {
    println!(
        "candidate=FullB,type={type_name},size={},align={},copy=true",
        size_of::<T>(),
        align_of::<T>()
    );
}

fn main() {
    observe_copy::<Vec2>("Vec2");
    observe_copy::<Vec3>("Vec3");
    observe_copy::<Vec4>("Vec4");
    observe_copy::<Quat>("Quat");
    observe_copy::<Mat4>("Mat4");

    let vector = Vec3::new(1.0, 2.0, 3.0);
    let matrix = Mat4::from_cols_array(&[
        1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0, 10.0, 11.0, 12.0, 13.0, 14.0, 15.0, 16.0,
    ]);
    println!(
        "candidate=FullB,access=scalar-array,vec3={:?},mat4={:?}",
        vector.to_array(),
        matrix.to_cols_array()
    );
}
