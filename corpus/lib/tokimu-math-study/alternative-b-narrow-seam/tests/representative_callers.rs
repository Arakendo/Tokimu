use tokimu_math_study_narrow_b::{
    projection_orthographic_rh_gl, projection_perspective_rh_gl, view_look_at_rh, Mat4, Vec3,
};
#[cfg(target_arch = "wasm32")]
use wasm_bindgen_test::wasm_bindgen_test;

fn assert_near(actual: f32, expected: f32) {
    let tolerance = 1.0e-5 + 1.0e-5 * expected.abs();
    assert!((actual - expected).abs() <= tolerance);
}

#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
#[cfg_attr(not(target_arch = "wasm32"), test)]
fn stereo_and_doom_observer_callers_use_only_the_narrow_seam() {
    let center_eye = Vec3::new(2.5, 0.4, 1.0);
    let right = (Vec3::ZERO - center_eye)
        .normalize()
        .cross(Vec3::Y)
        .normalize();
    let projection =
        projection_perspective_rh_gl(60.0_f32.to_radians(), (1280.0 * 0.5) / 720.0, 0.1, 100.0)
            .expect("valid stereo projection");
    let left = projection
        * view_look_at_rh(center_eye - right * 0.14, Vec3::ZERO, Vec3::Y).expect("valid left view");
    let right = projection
        * view_look_at_rh(center_eye + right * 0.14, Vec3::ZERO, Vec3::Y)
            .expect("valid right view");
    assert_ne!(left.to_cols_array(), right.to_cols_array());

    let source_xy = [1056.0, -3616.0];
    let position = Vec3::new(-source_xy[0], 36.0, source_xy[1]);
    let forward = Vec3::new(0.0, 0.0, 1.0);
    let doom_view = view_look_at_rh(position, position + forward * 128.0, Vec3::Y)
        .expect("valid Doom observer view");
    let eye_in_view = doom_view.transform_point3(position).to_array();
    for component in eye_in_view {
        assert_near(component, 0.0);
    }
}

#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
#[cfg_attr(not(target_arch = "wasm32"), test)]
fn cad_picking_keeps_provider_values_but_owns_projection_construction() {
    let view = view_look_at_rh(Vec3::new(4.0, 3.0, 4.0), Vec3::new(0.0, 0.25, 0.0), Vec3::Y)
        .expect("valid CAD view");
    let projection =
        projection_perspective_rh_gl(60.0_f32.to_radians(), 1280.0 / 720.0, 0.1, 100.0)
            .expect("valid CAD projection");
    let inverse = (projection * view).inverse();
    let near = inverse.project_point3(Vec3::new(-0.25, 0.3, -1.0));
    let far = inverse.project_point3(Vec3::new(-0.25, 0.3, 1.0));
    let direction = (far - near).normalize();

    assert!(near.is_finite());
    assert_near(direction.length(), 1.0);
}

#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
#[cfg_attr(not(target_arch = "wasm32"), test)]
fn renderer_transport_remains_a_scalar_array_boundary() {
    let projection =
        projection_perspective_rh_gl(1.0, 16.0 / 9.0, 0.1, 100.0).expect("valid projection");
    let columns = projection.to_cols_array();
    let restored = Mat4::from_cols_array(&columns);

    assert_eq!(restored.to_cols_array(), columns);
}

#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
#[cfg_attr(not(target_arch = "wasm32"), test)]
fn asteroid_projection_and_long_lived_camera_state_remain_provider_values() {
    #[derive(Clone, Copy)]
    struct StoredCamera {
        view: Mat4,
        projection: Mat4,
    }

    let stored = StoredCamera {
        view: Mat4::IDENTITY,
        projection: projection_orthographic_rh_gl(-16.0, 16.0, -9.0, 9.0, -1.0, 1.0)
            .expect("valid asteroid projection"),
    };
    let clip = (stored.projection * stored.view).transform_point3(Vec3::new(8.0, 4.5, 0.0));

    assert_near(clip.x, 0.5);
    assert_near(clip.y, 0.5);
    assert_near(clip.z, 0.0);
}
