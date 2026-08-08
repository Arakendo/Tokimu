//! Real glTF-node input for the bounded hole-punch transform comparison.

use std::path::PathBuf;

use gltf_corpus::decode_glb_file;
use tokimu_math_study::migration_hello_hole_punch::{
    resolve_two_node_world_with_a, resolve_two_node_world_with_b, resolve_two_node_world_with_c,
};

fn khronos_box_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../..")
        .join(
        "third-party/fixtures/khronos-gltf-sample-assets/upstream/Models/Box/glTF-Binary/Box.glb",
    )
}

fn assert_matrix_near(actual: [f32; 16], expected: [f32; 16]) {
    for (actual, expected) in actual.into_iter().zip(expected) {
        assert!(
            (actual - expected).abs() <= 1.0e-5,
            "{actual} != {expected}"
        );
    }
}

#[test]
fn candidates_match_the_baseline_for_a_pinned_gltf_node_and_translation_override() {
    let model = decode_glb_file(khronos_box_path()).expect("the pinned Box.glb fixture decodes");
    let node = model
        .nodes
        .first()
        .expect("the pinned Box.glb fixture has a node");
    let parent = [
        1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.5, -0.25, 1.0, 1.0,
    ];
    let translation = Some([0.75, 1.25, -0.5]);

    let baseline = resolve_two_node_world_with_a(&parent, &node.local_transform, translation);
    assert_matrix_near(
        resolve_two_node_world_with_b(&parent, &node.local_transform, translation),
        baseline,
    );
    assert_matrix_near(
        resolve_two_node_world_with_c(&parent, &node.local_transform, translation),
        baseline,
    );
}
