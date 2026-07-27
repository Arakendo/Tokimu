use std::path::{Path, PathBuf};

use gltf_corpus::{
    decode_glb_file, decode_gltf_file, inspect_glb_file, inspect_gltf_file, GlbChunkKind,
};

#[test]
fn triangle_json_and_external_buffer_are_structurally_valid() {
    let root = fixture_root();
    let path = root.join("upstream/Models/Triangle/glTF/Triangle.gltf");
    let inspection = inspect_gltf_file(path).expect("Triangle should inspect");

    assert_eq!(inspection.summary.asset_version, "2.0");
    assert_eq!(inspection.summary.scenes, 1);
    assert_eq!(inspection.summary.nodes, 1);
    assert_eq!(inspection.summary.meshes, 1);
    assert_eq!(inspection.summary.primitives, 1);
    assert_eq!(inspection.summary.accessors, 2);
    assert_eq!(inspection.summary.buffer_views, 2);
    assert_eq!(inspection.summary.buffers, 1);
    assert_eq!(inspection.resolved_buffers.len(), 1);
    assert_eq!(inspection.resolved_buffers[0].declared_byte_length, 44);
    assert_eq!(inspection.resolved_buffers[0].actual_byte_length, 44);
}

#[test]
fn box_glb_has_valid_json_and_binary_chunks() {
    let root = fixture_root();
    let path = root.join("upstream/Models/Box/glTF-Binary/Box.glb");
    let inspection = inspect_glb_file(path).expect("Box GLB should inspect");

    assert_eq!(inspection.version, 2);
    assert_eq!(inspection.declared_byte_length, 1_664);
    assert_eq!(inspection.chunks.len(), 2);
    assert_eq!(inspection.chunks[0].kind, GlbChunkKind::Json);
    assert_eq!(inspection.chunks[1].kind, GlbChunkKind::Binary);
    assert_eq!(inspection.chunks[1].byte_length, 648);
    assert_eq!(inspection.summary.scenes, 1);
    assert_eq!(inspection.summary.nodes, 2);
    assert_eq!(inspection.summary.meshes, 1);
    assert_eq!(inspection.summary.primitives, 1);
    assert_eq!(inspection.summary.accessors, 3);
    assert_eq!(inspection.summary.buffer_views, 2);
    assert_eq!(inspection.summary.materials, 1);
}

#[test]
fn triangle_positions_and_indices_decode() {
    let root = fixture_root();
    let path = root.join("upstream/Models/Triangle/glTF/Triangle.gltf");
    let model = decode_gltf_file(path).expect("Triangle should decode");

    assert_eq!(model.primitives.len(), 1);
    let primitive = &model.primitives[0];
    assert_eq!(primitive.positions.len(), 3);
    assert!(primitive.normals.is_empty());
    assert!(primitive.tex_coords_0.is_empty());
    assert_eq!(primitive.indices, [0, 1, 2]);
    assert_eq!(primitive.bounds.min, [0.0, 0.0, 0.0]);
    assert_eq!(primitive.bounds.max, [1.0, 1.0, 0.0]);
}

#[test]
fn box_positions_normals_and_indices_decode() {
    let root = fixture_root();
    let path = root.join("upstream/Models/Box/glTF-Binary/Box.glb");
    let model = decode_glb_file(path).expect("Box should decode");

    assert_eq!(model.primitives.len(), 1);
    let primitive = &model.primitives[0];
    assert_eq!(primitive.positions.len(), 24);
    assert_eq!(primitive.normals.len(), 24);
    assert!(primitive.tex_coords_0.is_empty());
    assert_eq!(primitive.indices.len(), 36);
    assert_eq!(primitive.bounds.min, [-0.5, -0.5, -0.5]);
    assert_eq!(primitive.bounds.max, [0.5, 0.5, 0.5]);
    assert!(primitive
        .normals
        .iter()
        .flatten()
        .all(|component| component.is_finite()));
}

#[test]
fn box_textured_positions_normals_indices_and_uvs_decode() {
    let root = fixture_root();
    let path = root.join("upstream/Models/BoxTextured/glTF/BoxTextured.gltf");
    let model = decode_gltf_file(path).expect("BoxTextured should decode");

    assert_eq!(model.primitives.len(), 1);
    let primitive = &model.primitives[0];
    assert_eq!(primitive.positions.len(), 24);
    assert_eq!(primitive.normals.len(), 24);
    assert_eq!(primitive.tex_coords_0.len(), 24);
    assert_eq!(primitive.indices.len(), 36);
    assert_eq!(primitive.bounds.min, [-0.5, -0.5, -0.5]);
    assert_eq!(primitive.bounds.max, [0.5, 0.5, 0.5]);
    assert!(primitive
        .tex_coords_0
        .iter()
        .flatten()
        .all(|value| value.is_finite()));
    assert_eq!(
        primitive
            .tex_coords_0
            .iter()
            .map(|uv| uv[0])
            .fold(f32::NEG_INFINITY, f32::max),
        6.0
    );
}

fn fixture_root() -> PathBuf {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../../third-party/fixtures/khronos-gltf-sample-assets");
    assert!(
        root.is_dir(),
        "missing Khronos fixtures at {}; run prepare-khronos-gltf-corpus.ps1",
        root.display()
    );
    root
}
