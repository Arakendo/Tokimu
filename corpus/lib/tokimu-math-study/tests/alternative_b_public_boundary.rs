//! External-consumer evidence for Alternative B's public boundary.

use tokimu_math_study::{
    alternative_b::{Mat4, Vec2, Vec3, Vec4},
    migration_b::Camera,
};

#[test]
fn external_consumer_uses_candidate_math_without_a_provider_type() {
    let camera = Camera {
        eye: Vec3::new(0.0, 0.0, 5.0),
        center: Vec3::ZERO,
        up: Vec3::Y,
        vertical_fov_radians: 1.0,
        aspect_ratio: 16.0 / 9.0,
        near: 0.1,
        far: 100.0,
    };
    let view_projection = camera.view_projection();
    let transformed = view_projection.transform_point3(Vec3::new(1.0, 2.0, 3.0));

    assert!(transformed.to_array().into_iter().all(f32::is_finite));
    assert_eq!(
        Vec3::new(1.0, 2.0, 3.0).extend(1.0),
        Vec4::new(1.0, 2.0, 3.0, 1.0)
    );
    assert_eq!(Vec2::new(1.0, 2.0).to_array(), [1.0, 2.0]);
    assert_eq!(Mat4::IDENTITY.transform_vector3(Vec3::Y), Vec3::Y);
}

#[test]
fn long_lived_full_b_state_crosses_renderer_transport_as_scalars() {
    #[derive(Clone, Copy)]
    struct StoredCamera {
        view: Mat4,
        projection: Mat4,
    }

    let camera = Camera {
        eye: Vec3::new(2.5, 0.4, 1.0),
        center: Vec3::ZERO,
        up: Vec3::Y,
        vertical_fov_radians: 1.0,
        aspect_ratio: 16.0 / 9.0,
        near: 0.1,
        far: 100.0,
    };
    let stored = StoredCamera {
        view: Mat4::look_at_rh(camera.eye, camera.center, camera.up),
        projection: Mat4::perspective_rh_gl(
            camera.vertical_fov_radians,
            camera.aspect_ratio,
            camera.near,
            camera.far,
        ),
    };
    let upload_columns = (stored.projection * stored.view).to_cols_array();
    let restored = Mat4::from_cols_array(&upload_columns);

    assert_eq!(restored.to_cols_array(), upload_columns);
    assert!(upload_columns.into_iter().all(f32::is_finite));
}
