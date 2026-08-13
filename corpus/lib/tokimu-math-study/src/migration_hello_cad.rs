//! Corpus-local port of `hello-cad`'s cursor-to-world camera-ray path.
//!
//! This admits only the real caller pressure absent from the earlier fixtures:
//! homogeneous `Mat4 * Vec4`, perspective divide, and zero-safe ray rejection.

use crate::{
    alternative_b::{Mat4 as BMat4, Vec3 as BVec3, Vec4 as BVec4},
    alternative_c::{Mat4 as CMat4, Vec3 as CVec3, Vec4 as CVec4},
};
use tokimu_core::math::{
    try_projection_perspective_rh_gl, try_view_look_at_rh, Vec3 as AVec3, Vec4 as AVec4,
};

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct CameraRay {
    pub origin: [f32; 3],
    pub direction: [f32; 3],
}

#[must_use]
pub fn camera_ray_with_a(window_size: [f32; 2], cursor: [f32; 2]) -> Option<CameraRay> {
    let (ndc_x, ndc_y) = cursor_ndc(window_size, cursor);
    let view = try_view_look_at_rh(
        AVec3::new(4.0, 3.0, 4.0),
        AVec3::new(0.0, 0.25, 0.0),
        AVec3::Y,
    )?;
    let projection = try_projection_perspective_rh_gl(
        60.0_f32.to_radians(),
        aspect_ratio(window_size),
        0.1,
        100.0,
    )?;
    let inverse = (projection * view).inverse();
    camera_ray_a(
        inverse * AVec4::new(ndc_x, ndc_y, -1.0, 1.0),
        inverse * AVec4::new(ndc_x, ndc_y, 1.0, 1.0),
    )
}

#[must_use]
pub fn camera_ray_with_b(window_size: [f32; 2], cursor: [f32; 2]) -> Option<CameraRay> {
    let (ndc_x, ndc_y) = cursor_ndc(window_size, cursor);
    let view = BMat4::look_at_rh(
        BVec3::new(4.0, 3.0, 4.0),
        BVec3::new(0.0, 0.25, 0.0),
        BVec3::Y,
    );
    let projection =
        BMat4::perspective_rh_gl(60.0_f32.to_radians(), aspect_ratio(window_size), 0.1, 100.0);
    let inverse = (projection * view).inverse();
    camera_ray_b(
        inverse * BVec4::new(ndc_x, ndc_y, -1.0, 1.0),
        inverse * BVec4::new(ndc_x, ndc_y, 1.0, 1.0),
    )
}

#[must_use]
pub fn camera_ray_with_c(window_size: [f32; 2], cursor: [f32; 2]) -> Option<CameraRay> {
    let (ndc_x, ndc_y) = cursor_ndc(window_size, cursor);
    let view = CMat4::look_at_rh(
        CVec3::new(4.0, 3.0, 4.0),
        CVec3::new(0.0, 0.25, 0.0),
        CVec3::Y,
    );
    let projection =
        CMat4::perspective_rh_gl(60.0_f32.to_radians(), aspect_ratio(window_size), 0.1, 100.0);
    let inverse = (projection * view).inverse();
    camera_ray_c(
        inverse * CVec4::new(ndc_x, ndc_y, -1.0, 1.0),
        inverse * CVec4::new(ndc_x, ndc_y, 1.0, 1.0),
    )
}

fn aspect_ratio(window_size: [f32; 2]) -> f32 {
    window_size[0].max(1.0) / window_size[1].max(1.0)
}

fn cursor_ndc(window_size: [f32; 2], cursor: [f32; 2]) -> (f32, f32) {
    let width = window_size[0].max(1.0);
    let height = window_size[1].max(1.0);
    (
        (cursor[0] / width) * 2.0 - 1.0,
        1.0 - (cursor[1] / height) * 2.0,
    )
}

fn camera_ray_a(near: AVec4, far: AVec4) -> Option<CameraRay> {
    if near.w.abs() <= f32::EPSILON || far.w.abs() <= f32::EPSILON {
        return None;
    }
    let origin = near.truncate() / near.w;
    let direction = (far.truncate() / far.w - origin).normalize_or_zero();
    (direction.length_squared() > f32::EPSILON).then_some(CameraRay {
        origin: origin.to_array(),
        direction: direction.to_array(),
    })
}

fn camera_ray_b(near: BVec4, far: BVec4) -> Option<CameraRay> {
    if near.w().abs() <= f32::EPSILON || far.w().abs() <= f32::EPSILON {
        return None;
    }
    let origin = near.truncate() / near.w();
    let direction = (far.truncate() / far.w() - origin).normalize_or_zero();
    (direction.length_squared() > f32::EPSILON).then_some(CameraRay {
        origin: origin.to_array(),
        direction: direction.to_array(),
    })
}

fn camera_ray_c(near: CVec4, far: CVec4) -> Option<CameraRay> {
    if near.w.abs() <= f32::EPSILON || far.w.abs() <= f32::EPSILON {
        return None;
    }
    let origin = near.truncate() / near.w;
    let direction = (far.truncate() / far.w - origin).normalize_or_zero();
    (direction.length_squared() > f32::EPSILON).then_some(CameraRay {
        origin: origin.to_array(),
        direction: direction.to_array(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn assert_ray_near(actual: CameraRay, expected: CameraRay) {
        for (actual, expected) in actual
            .origin
            .into_iter()
            .chain(actual.direction)
            .zip(expected.origin.into_iter().chain(expected.direction))
        {
            assert!(
                (actual - expected).abs() <= 1.0e-5,
                "{actual} != {expected}"
            );
        }
    }

    #[test]
    fn candidates_match_the_hello_cad_cursor_ray_path() {
        let input = ([1280.0, 720.0], [407.5, 231.25]);
        let baseline = camera_ray_with_a(input.0, input.1).expect("baseline ray");
        assert_ray_near(
            camera_ray_with_b(input.0, input.1).expect("B ray"),
            baseline,
        );
        assert_ray_near(
            camera_ray_with_c(input.0, input.1).expect("C ray"),
            baseline,
        );
    }
}
