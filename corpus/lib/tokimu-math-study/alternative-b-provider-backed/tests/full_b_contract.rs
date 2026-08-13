use tokimu_math_study_provider_backed::{Mat4, MathError, MathFailure, MathOperation, Vec3};
#[cfg(target_arch = "wasm32")]
use wasm_bindgen_test::wasm_bindgen_test;

const ABS_EPSILON: f32 = 1.0e-5;
const REL_EPSILON: f32 = 1.0e-5;

fn assert_close(actual: f32, expected: f32) {
    let tolerance = ABS_EPSILON + REL_EPSILON * expected.abs();
    assert!(
        (actual - expected).abs() <= tolerance,
        "actual={actual:?} expected={expected:?} tolerance={tolerance:?}"
    );
}

fn assert_vec3_close(actual: Vec3, expected: [f32; 3]) {
    for (actual, expected) in actual.to_array().into_iter().zip(expected) {
        assert_close(actual, expected);
    }
}

fn next_scalar(state: &mut u32) -> f32 {
    *state = state.wrapping_mul(1_664_525).wrapping_add(1_013_904_223);
    ((*state >> 8) as f32 / 16_777_215.0) * 20.0 - 10.0
}

#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
#[cfg_attr(not(target_arch = "wasm32"), test)]
fn checked_failures_are_bounded_and_provider_neutral() {
    assert_eq!(
        Vec3::ZERO.try_normalize(),
        Err(MathError {
            operation: MathOperation::Normalize,
            failure: MathFailure::ZeroLength,
        })
    );
    assert_eq!(
        Vec3::new(f32::NAN, 0.0, 0.0).try_normalize(),
        Err(MathError {
            operation: MathOperation::Normalize,
            failure: MathFailure::NonFiniteInput,
        })
    );
    assert_eq!(
        Vec3::splat(1.0e-30).try_normalize(),
        Err(MathError {
            operation: MathOperation::Normalize,
            failure: MathFailure::ZeroLength,
        })
    );
    assert_eq!(
        Vec3::splat(f32::MAX).try_normalize(),
        Err(MathError {
            operation: MathOperation::Normalize,
            failure: MathFailure::NonFiniteResult,
        })
    );
    assert_eq!(
        Mat4::from_scale(Vec3::new(1.0, 0.0, 1.0)).try_inverse(),
        Err(MathError {
            operation: MathOperation::Inverse,
            failure: MathFailure::Singular,
        })
    );
    assert_eq!(
        Mat4::from_cols_array(&[f32::NAN; 16]).try_inverse(),
        Err(MathError {
            operation: MathOperation::Inverse,
            failure: MathFailure::NonFiniteInput,
        })
    );
    assert_eq!(
        Mat4::try_look_at_rh(Vec3::ZERO, Vec3::ZERO, Vec3::Y),
        Err(MathError {
            operation: MathOperation::ViewLookAtRh,
            failure: MathFailure::DegenerateView,
        })
    );
    assert_eq!(
        Mat4::try_perspective_rh_gl(0.0, 1.0, 0.1, 100.0),
        Err(MathError {
            operation: MathOperation::PerspectiveRhGl,
            failure: MathFailure::InvalidFrustum,
        })
    );
    assert_eq!(
        Mat4::try_perspective_rh_gl(1.0, f32::from_bits(1), 0.1, 100.0),
        Err(MathError {
            operation: MathOperation::PerspectiveRhGl,
            failure: MathFailure::NonFiniteResult,
        })
    );
    assert_eq!(
        Mat4::try_look_at_rh(Vec3::splat(f32::MAX), Vec3::splat(-f32::MAX), Vec3::Y),
        Err(MathError {
            operation: MathOperation::ViewLookAtRh,
            failure: MathFailure::NonFiniteResult,
        })
    );
}

#[cfg(not(target_arch = "wasm32"))]
#[test]
fn rejected_inputs_return_without_unwinding() {
    let result = std::panic::catch_unwind(|| {
        let failures = [
            Vec3::ZERO.try_normalize().map(|_| ()),
            Vec3::splat(f32::MAX).try_normalize().map(|_| ()),
            Mat4::from_scale(Vec3::new(1.0, 0.0, 1.0))
                .try_inverse()
                .map(|_| ()),
            Mat4::try_look_at_rh(Vec3::ZERO, Vec3::ZERO, Vec3::Y).map(|_| ()),
            Mat4::try_perspective_rh_gl(0.0, 1.0, 0.1, 100.0).map(|_| ()),
        ];
        assert!(failures.into_iter().all(|failure| failure.is_err()));
    });
    assert!(result.is_ok(), "checked failure paths must not unwind");
}

#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
#[cfg_attr(not(target_arch = "wasm32"), test)]
fn checked_projection_matches_independent_homogeneous_arithmetic() {
    let projection =
        Mat4::try_perspective_rh_gl(1.1, 16.0 / 9.0, 0.1, 100.0).expect("valid perspective");
    let point = Vec3::new(0.25, -0.5, -2.0);
    let columns = projection.to_cols_array();
    let [x, y, z] = point.to_array();
    let clip_x = columns[0] * x + columns[4] * y + columns[8] * z + columns[12];
    let clip_y = columns[1] * x + columns[5] * y + columns[9] * z + columns[13];
    let clip_z = columns[2] * x + columns[6] * y + columns[10] * z + columns[14];
    let clip_w = columns[3] * x + columns[7] * y + columns[11] * z + columns[15];

    assert_vec3_close(
        projection.try_project_point3(point).expect("finite point"),
        [clip_x / clip_w, clip_y / clip_w, clip_z / clip_w],
    );
    assert_eq!(
        projection.try_project_point3(Vec3::ZERO),
        Err(MathError {
            operation: MathOperation::ProjectPoint,
            failure: MathFailure::ZeroHomogeneousW,
        })
    );
}

#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
#[cfg_attr(not(target_arch = "wasm32"), test)]
fn fixed_seed_normalization_and_inverse_properties_hold() {
    let mut state = 0x5eed_c0de;
    for _ in 0..256 {
        let vector = Vec3::new(
            next_scalar(&mut state),
            next_scalar(&mut state),
            next_scalar(&mut state),
        );
        let normalized = vector.try_normalize().expect("generated nonzero vector");
        assert_close(normalized.length(), 1.0);

        let translation = Vec3::new(
            next_scalar(&mut state),
            next_scalar(&mut state),
            next_scalar(&mut state),
        );
        let scale = Vec3::new(
            next_scalar(&mut state).abs() + 0.25,
            next_scalar(&mut state).abs() + 0.25,
            next_scalar(&mut state).abs() + 0.25,
        );
        let transform = Mat4::from_translation(translation)
            * Mat4::from_rotation_y(next_scalar(&mut state))
            * Mat4::from_scale(scale);
        let point = Vec3::new(
            next_scalar(&mut state),
            next_scalar(&mut state),
            next_scalar(&mut state),
        );
        let restored = transform
            .try_inverse()
            .expect("generated transform is invertible")
            .transform_point3(transform.transform_point3(point));
        assert_vec3_close(restored, point.to_array());
    }
}

#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
#[cfg_attr(not(target_arch = "wasm32"), test)]
fn checked_view_and_orthographic_constructors_have_independent_landmarks() {
    let view =
        Mat4::try_look_at_rh(Vec3::new(0.0, 0.0, 5.0), Vec3::ZERO, Vec3::Y).expect("valid view");
    assert_vec3_close(view.transform_point3(Vec3::ZERO), [0.0, 0.0, -5.0]);

    let orthographic = Mat4::try_orthographic_rh_gl(-2.0, 2.0, -1.0, 1.0, 1.0, 11.0)
        .expect("valid orthographic projection");
    assert_vec3_close(
        orthographic
            .try_project_point3(Vec3::new(-2.0, -1.0, -1.0))
            .expect("near lower-left point"),
        [-1.0, -1.0, -1.0],
    );
    assert_vec3_close(
        orthographic
            .try_project_point3(Vec3::new(2.0, 1.0, -11.0))
            .expect("far upper-right point"),
        [1.0, 1.0, 1.0],
    );
}

#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
#[cfg_attr(not(target_arch = "wasm32"), test)]
fn pressured_compatibility_surface_remains_bounded() {
    assert_eq!(Vec3::ONE.to_array(), [1.0, 1.0, 1.0]);
    assert_eq!(Vec3::Y.to_array(), [0.0, 1.0, 0.0]);
    let left = Vec3::new(1.0, 4.0, -2.0);
    let right = Vec3::splat(2.0);
    assert_eq!(Vec3::from_array(left.to_array()), left);
    assert_eq!(left.min(right).to_array(), [1.0, 2.0, -2.0]);
    assert_eq!(left.max(right).to_array(), [2.0, 4.0, 2.0]);
    assert_eq!(left.lerp(right, 0.5).to_array(), [1.5, 3.0, 0.0]);
    assert_close(left.distance(right), (21.0_f32).sqrt());
    assert_eq!(left.dot(right), 6.0);
    assert_eq!(left.cross(right).to_array(), [12.0, -6.0, -6.0]);
    assert_eq!((left + right - right).to_array(), left.to_array());
    assert_eq!((-left).to_array(), [-1.0, -4.0, 2.0]);
    assert_eq!((left * right / right).to_array(), left.to_array());
    assert_eq!((left * 2.0 / 2.0).to_array(), left.to_array());
    let mut accumulated = Vec3::ZERO;
    accumulated += left;
    assert_eq!(accumulated, left);
    assert!(left.is_finite());
    assert_close(left.normalize_or_zero().length(), 1.0);

    let quarter_turn = core::f32::consts::FRAC_PI_2;
    assert_vec3_close(
        Mat4::from_rotation_x(quarter_turn).transform_vector3(Vec3::Y),
        [0.0, 0.0, 1.0],
    );
    assert_vec3_close(
        Mat4::from_rotation_y(quarter_turn).transform_vector3(Vec3::new(1.0, 0.0, 0.0)),
        [0.0, 0.0, -1.0],
    );
    assert_vec3_close(
        Mat4::from_rotation_z(quarter_turn).transform_vector3(Vec3::new(1.0, 0.0, 0.0)),
        [0.0, 1.0, 0.0],
    );
    assert_vec3_close(
        Mat4::from_translation(Vec3::new(3.0, 4.0, 5.0)).transform_point3(Vec3::new(1.0, 2.0, 3.0)),
        [4.0, 6.0, 8.0],
    );
    assert_vec3_close(
        Mat4::from_scale(Vec3::new(2.0, 3.0, 4.0)).transform_vector3(Vec3::new(1.0, 2.0, 3.0)),
        [2.0, 6.0, 12.0],
    );

    let mut transform = Mat4::from_translation(Vec3::new(3.0, 4.0, 5.0))
        * Mat4::from_rotation_x(0.2)
        * Mat4::from_rotation_y(0.3)
        * Mat4::from_rotation_z(0.4)
        * Mat4::from_scale(Vec3::new(2.0, 3.0, 4.0));
    assert!(transform.is_finite());
    let columns = transform.to_cols_array();
    assert_eq!(Mat4::from_cols_array(&columns).to_cols_array(), columns);
    assert!(transform
        .transform_vector3(Vec3::new(1.0, 0.0, 0.0))
        .is_finite());
    assert!(transform.transpose().transpose().is_finite());
    let final_column = transform.w_axis();
    assert!(final_column.is_finite());
    assert_eq!(
        [
            final_column.x(),
            final_column.y(),
            final_column.z(),
            final_column.w(),
        ],
        final_column.to_array()
    );
    transform.set_w_axis(final_column);
    assert_eq!(transform.w_axis(), final_column);
    assert!(Mat4::IDENTITY.is_finite());
}
