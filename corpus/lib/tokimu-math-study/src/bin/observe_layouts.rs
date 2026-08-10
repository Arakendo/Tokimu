//! Native representation observations for the current corpus candidates.
//!
//! Output records compiler/target facts only. It does not turn a candidate's
//! present field layout into a stable Tokimu ABI, FFI, SIMD, or GPU contract.

use core::mem::{align_of, size_of};

use tokimu_core::math::{
    Mat4 as AMat4, Quat as AQuat, Vec2 as AVec2, Vec3 as AVec3, Vec4 as AVec4,
};
use tokimu_math_study::{
    alternative_b::{Mat4 as BMat4, Quat as BQuat, Vec2 as BVec2, Vec3 as BVec3, Vec4 as BVec4},
    alternative_c::{Mat4 as CMat4, Vec3 as CVec3, Vec4 as CVec4},
    alternative_d::Vec3 as DVec3,
};

fn observe<T>(candidate: &str, type_name: &str) {
    println!(
        "candidate={candidate},type={type_name},size={},align={}",
        size_of::<T>(),
        align_of::<T>()
    );
}

fn main() {
    observe::<AVec2>("A", "Vec2");
    observe::<AVec3>("A", "Vec3");
    observe::<AVec4>("A", "Vec4");
    observe::<AQuat>("A", "Quat");
    observe::<AMat4>("A", "Mat4");

    observe::<BVec2>("B", "Vec2");
    observe::<BVec3>("B", "Vec3");
    observe::<BVec4>("B", "Vec4");
    observe::<BQuat>("B", "Quat");
    observe::<BMat4>("B", "Mat4");

    observe::<CVec3>("C", "Vec3");
    observe::<CVec4>("C", "Vec4");
    observe::<CMat4>("C", "Mat4");

    observe::<DVec3>("D", "Vec3");
}
