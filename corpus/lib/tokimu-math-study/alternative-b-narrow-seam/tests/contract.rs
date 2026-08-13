use tokimu_math_study_narrow_b::{
    projection_orthographic_rh_gl, projection_perspective_rh_gl, view_look_at_rh,
    ConstructionError, ConstructionFailure, ConstructionOperation, Vec3,
};
#[cfg(target_arch = "wasm32")]
use wasm_bindgen_test::wasm_bindgen_test;

const ABSOLUTE_TOLERANCE: f32 = 1.0e-5;
const RELATIVE_TOLERANCE: f32 = 1.0e-5;

fn assert_near(actual: f32, expected: f32) {
    let tolerance = ABSOLUTE_TOLERANCE + RELATIVE_TOLERANCE * expected.abs();
    assert!(
        (actual - expected).abs() <= tolerance,
        "expected {expected}, got {actual}, tolerance {tolerance}"
    );
}

fn expected_error(
    operation: ConstructionOperation,
    failure: ConstructionFailure,
) -> ConstructionError {
    ConstructionError { operation, failure }
}

#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
#[cfg_attr(not(target_arch = "wasm32"), test)]
fn view_matches_independent_point_and_basis_controls() {
    let eye = Vec3::new(2.0, 3.0, 5.0);
    let target = Vec3::new(2.0, 3.0, 4.0);
    let view = view_look_at_rh(eye, target, Vec3::Y).expect("valid view");

    let eye_in_view = view.transform_point3(eye).to_array();
    let target_in_view = view.transform_point3(target).to_array();
    for component in eye_in_view {
        assert_near(component, 0.0);
    }
    assert_near(target_in_view[0], 0.0);
    assert_near(target_in_view[1], 0.0);
    assert_near(target_in_view[2], -1.0);

    let columns = view.to_cols_array();
    let x = [columns[0], columns[4], columns[8]];
    let y = [columns[1], columns[5], columns[9]];
    let z = [columns[2], columns[6], columns[10]];
    let dot = |a: [f32; 3], b: [f32; 3]| a[0] * b[0] + a[1] * b[1] + a[2] * b[2];
    assert_near(dot(x, x), 1.0);
    assert_near(dot(y, y), 1.0);
    assert_near(dot(z, z), 1.0);
    assert_near(dot(x, y), 0.0);
    assert_near(dot(y, z), 0.0);
    let determinant = x[0] * (y[1] * z[2] - y[2] * z[1]) - x[1] * (y[0] * z[2] - y[2] * z[0])
        + x[2] * (y[0] * z[1] - y[1] * z[0]);
    assert_near(determinant, 1.0);
}

#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
#[cfg_attr(not(target_arch = "wasm32"), test)]
fn view_rejections_are_bounded_and_provider_neutral() {
    assert_eq!(
        view_look_at_rh(Vec3::ZERO, Vec3::ZERO, Vec3::Y),
        Err(expected_error(
            ConstructionOperation::ViewLookAtRh,
            ConstructionFailure::DegenerateView,
        ))
    );
    assert_eq!(
        view_look_at_rh(Vec3::ZERO, -Vec3::Z, Vec3::ZERO),
        Err(expected_error(
            ConstructionOperation::ViewLookAtRh,
            ConstructionFailure::DegenerateView,
        ))
    );
    assert_eq!(
        view_look_at_rh(Vec3::ZERO, -Vec3::Z, Vec3::Z),
        Err(expected_error(
            ConstructionOperation::ViewLookAtRh,
            ConstructionFailure::DegenerateView,
        ))
    );
    assert_eq!(
        view_look_at_rh(Vec3::new(f32::NAN, 0.0, 0.0), -Vec3::Z, Vec3::Y),
        Err(expected_error(
            ConstructionOperation::ViewLookAtRh,
            ConstructionFailure::NonFiniteInput,
        ))
    );

    let near_collinear = view_look_at_rh(Vec3::ZERO, -Vec3::Z, Vec3::new(1.0e-8, 0.0, 1.0))
        .expect("finite near-collinear basis remains admitted");
    assert!(near_collinear
        .to_cols_array()
        .into_iter()
        .all(f32::is_finite));

    assert_eq!(
        view_look_at_rh(Vec3::splat(f32::MAX), Vec3::splat(-f32::MAX), Vec3::Y),
        Err(expected_error(
            ConstructionOperation::ViewLookAtRh,
            ConstructionFailure::NonFiniteResult,
        ))
    );

    assert_eq!(
        view_look_at_rh(Vec3::ZERO, Vec3::new(0.0, 0.0, -1.0e-30), Vec3::Y),
        Err(expected_error(
            ConstructionOperation::ViewLookAtRh,
            ConstructionFailure::DegenerateView,
        ))
    );
}

#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
#[cfg_attr(not(target_arch = "wasm32"), test)]
fn perspective_maps_gl_depth_and_rejects_invalid_frusta() {
    let near = 0.1;
    let far = 100.0;
    let projection =
        projection_perspective_rh_gl(1.0, 16.0 / 9.0, near, far).expect("valid perspective");
    let columns = projection.to_cols_array();
    let project_z = |view_z: f32| {
        let clip_z = columns[10] * view_z + columns[14];
        let clip_w = columns[11] * view_z + columns[15];
        clip_z / clip_w
    };
    assert_near(project_z(-near), -1.0);
    assert_near(project_z(-far), 1.0);

    for parameters in [
        [0.0, 1.0, 0.1, 10.0],
        [core::f32::consts::PI, 1.0, 0.1, 10.0],
        [1.0, 0.0, 0.1, 10.0],
        [1.0, 1.0, 0.0, 10.0],
        [1.0, 1.0, 1.0, 1.0],
    ] {
        assert_eq!(
            projection_perspective_rh_gl(
                parameters[0],
                parameters[1],
                parameters[2],
                parameters[3]
            ),
            Err(expected_error(
                ConstructionOperation::PerspectiveRhGl,
                ConstructionFailure::InvalidFrustum,
            ))
        );
    }
    assert_eq!(
        projection_perspective_rh_gl(f32::INFINITY, 1.0, 0.1, 10.0),
        Err(expected_error(
            ConstructionOperation::PerspectiveRhGl,
            ConstructionFailure::NonFiniteInput,
        ))
    );
    assert_eq!(
        projection_perspective_rh_gl(1.0, f32::from_bits(1), 0.1, 10.0),
        Err(expected_error(
            ConstructionOperation::PerspectiveRhGl,
            ConstructionFailure::NonFiniteResult,
        ))
    );
}

#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
#[cfg_attr(not(target_arch = "wasm32"), test)]
fn orthographic_maps_all_extents_and_rejects_invalid_frusta() {
    let projection = projection_orthographic_rh_gl(-2.0, 2.0, -1.0, 1.0, 0.1, 10.0)
        .expect("valid orthographic projection");
    for (point, expected) in [
        ([-2.0, 0.0, -0.1], [-1.0, 0.0, -1.0]),
        ([2.0, 0.0, -10.0], [1.0, 0.0, 1.0]),
        ([0.0, -1.0, -0.1], [0.0, -1.0, -1.0]),
        ([0.0, 1.0, -10.0], [0.0, 1.0, 1.0]),
    ] {
        let actual = projection
            .transform_point3(Vec3::from_array(point))
            .to_array();
        for index in 0..3 {
            assert_near(actual[index], expected[index]);
        }
    }

    assert_eq!(
        projection_orthographic_rh_gl(1.0, -1.0, -1.0, 1.0, 0.1, 10.0),
        Err(expected_error(
            ConstructionOperation::OrthographicRhGl,
            ConstructionFailure::InvalidFrustum,
        ))
    );
    assert_eq!(
        projection_orthographic_rh_gl(-1.0, 1.0, -1.0, 1.0, f32::NAN, 10.0),
        Err(expected_error(
            ConstructionOperation::OrthographicRhGl,
            ConstructionFailure::NonFiniteInput,
        ))
    );
}

#[cfg(not(target_arch = "wasm32"))]
#[test]
fn rejected_inputs_return_without_unwinding() {
    let result = std::panic::catch_unwind(|| {
        let failures = [
            view_look_at_rh(Vec3::ZERO, Vec3::ZERO, Vec3::Y),
            view_look_at_rh(Vec3::splat(f32::MAX), Vec3::splat(-f32::MAX), Vec3::Y),
            projection_perspective_rh_gl(0.0, 1.0, 0.1, 10.0),
            projection_orthographic_rh_gl(1.0, -1.0, -1.0, 1.0, 0.1, 10.0),
        ];
        assert!(failures.into_iter().all(|failure| failure.is_err()));
    });
    assert!(result.is_ok(), "checked failure paths must not unwind");
}
