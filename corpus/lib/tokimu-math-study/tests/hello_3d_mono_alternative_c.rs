use tokimu_math_study::migration_hello_3d_mono::spin_with_c;

#[test]
fn alternative_c_rotating_cube_case_is_independently_runnable() {
    let mesh = spin_with_c(1.25, &[[-1.0, -1.0, -1.0]], &[[0.0, 1.0, 0.0]]);

    assert_eq!(mesh.positions.len(), 1);
    assert_eq!(mesh.normals.len(), 1);
}
