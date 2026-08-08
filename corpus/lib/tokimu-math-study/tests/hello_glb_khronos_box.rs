//! Imported-scene pressure from the pinned Khronos Box corpus asset.
//!
//! This is deliberately a decoded-input transform comparison, rather than a
//! second application copy. It keeps the loader and renderer outside this
//! math-vocabulary study while checking the real positions and normals that
//! the `hello-glb` corpus path receives.

use std::path::PathBuf;

use gltf_corpus::decode_glb_file;
use tokimu_math_study::{
    migration_hello_3d_mono::TransformedMesh,
    migration_hello_glb::{
        floor_with_a, floor_with_b, floor_with_c, model_with_a, model_with_b, model_with_c,
    },
};

fn khronos_box_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../..")
        .join(
        "third-party/fixtures/khronos-gltf-sample-assets/upstream/Models/Box/glTF-Binary/Box.glb",
    )
}

fn assert_mesh_near(left: &TransformedMesh, right: &TransformedMesh) {
    assert_eq!(left.positions.len(), right.positions.len());
    assert_eq!(left.normals.len(), right.normals.len());

    for (left, right) in left
        .positions
        .iter()
        .chain(&left.normals)
        .zip(right.positions.iter().chain(&right.normals))
    {
        for (left, right) in left.iter().zip(right) {
            assert!((left - right).abs() <= 1.0e-5, "{left} != {right}");
        }
    }
}

#[test]
fn candidates_match_the_baseline_for_pinned_khronos_box_geometry() {
    let model = decode_glb_file(khronos_box_path()).expect("the pinned Box.glb fixture decodes");
    let primitive = model
        .primitives
        .first()
        .expect("the pinned Box.glb fixture has one decoded primitive");
    assert!(!primitive.positions.is_empty());
    assert_eq!(primitive.positions.len(), primitive.normals.len());

    let seconds = 1.25;
    for (a, b, c) in [
        (
            model_with_a(seconds, &primitive.positions, &primitive.normals),
            model_with_b(seconds, &primitive.positions, &primitive.normals),
            model_with_c(seconds, &primitive.positions, &primitive.normals),
        ),
        (
            floor_with_a(seconds, &primitive.positions, &primitive.normals),
            floor_with_b(seconds, &primitive.positions, &primitive.normals),
            floor_with_c(seconds, &primitive.positions, &primitive.normals),
        ),
    ] {
        assert_mesh_near(&a, &b);
        assert_mesh_near(&a, &c);
    }
}
