use std::path::{Path, PathBuf};

use gltf_corpus::{
    decode_glb_file, decode_gltf_file, inspect_glb_file, inspect_gltf_file, CorpusError,
    GlbChunkKind,
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
    let material_path = root.join("upstream/Models/Box/glTF/Box.gltf");
    let material_inspection = inspect_gltf_file(material_path).expect("Box glTF should inspect");
    assert_eq!(material_inspection.materials.len(), 1);
    assert_eq!(
        material_inspection.materials[0].name.as_deref(),
        Some("Red")
    );
    assert_eq!(
        material_inspection.materials[0].base_color_factor,
        Some([0.8, 0.0, 0.0, 1.0])
    );
    assert_eq!(material_inspection.materials[0].base_color_texture, None);
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
    for indices in primitive.indices.chunks_exact(3) {
        let a = primitive.positions[indices[0] as usize];
        let b = primitive.positions[indices[1] as usize];
        let c = primitive.positions[indices[2] as usize];
        let geometric_normal = cross(subtract(b, a), subtract(c, a));
        for index in indices {
            let authored_normal = primitive.normals[*index as usize];
            assert!(
                dot(geometric_normal, authored_normal) > 0.0,
                "Box triangle {indices:?} winding disagrees with normal at vertex {index}"
            );
        }
    }
}

#[test]
fn box_textured_positions_normals_indices_and_uvs_decode() {
    let root = fixture_root();
    let path = root.join("upstream/Models/BoxTextured/glTF/BoxTextured.gltf");
    let model = decode_gltf_file(&path).expect("BoxTextured should decode");

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

    let inspection = inspect_gltf_file(&path).expect("BoxTextured should inspect");
    assert_eq!(inspection.materials.len(), 1);
    assert_eq!(inspection.materials[0].name.as_deref(), Some("Texture"));
    assert_eq!(inspection.materials[0].base_color_texture, Some(0));
    assert_eq!(inspection.textures.len(), 1);
    assert_eq!(inspection.textures[0].source, Some(0));
    assert_eq!(inspection.images.len(), 1);
    assert_eq!(
        inspection.images[0].uri.as_deref(),
        Some("CesiumLogoFlat.png")
    );

    assert_eq!(model.nodes.len(), 2);
    assert_eq!(model.nodes[0].children, [1]);
    assert_eq!(model.nodes[1].mesh, Some(0));
    assert_eq!(model.scenes.len(), 1);
    assert_eq!(model.scenes[0].roots, [0]);
    assert_eq!(
        model.scenes[0]
            .traversal
            .iter()
            .map(|entry| entry.node)
            .collect::<Vec<_>>(),
        [0, 1]
    );
    assert_eq!(
        model.scenes[0].traversal[1].world_transform,
        model.nodes[0].local_transform
    );
}

#[test]
fn multiple_scenes_decode_independent_meshes_buffers_and_transforms() {
    let root = fixture_root();
    let path = root.join("upstream/Models/MultipleScenes/glTF/MultipleScenes.gltf");
    let model = decode_gltf_file(path).expect("MultipleScenes should decode");

    assert_eq!(model.primitives.len(), 2);
    assert_eq!(model.primitives[0].location.mesh, 0);
    assert_eq!(model.primitives[0].positions.len(), 3);
    assert_eq!(model.primitives[0].indices.len(), 3);
    assert_eq!(model.primitives[1].location.mesh, 1);
    assert_eq!(model.primitives[1].positions.len(), 4);
    assert_eq!(model.primitives[1].indices.len(), 6);

    assert_eq!(model.scenes.len(), 2);
    assert_eq!(model.scenes[0].roots, [0]);
    assert_eq!(model.scenes[1].roots, [1]);
    assert_eq!(model.scenes[0].traversal[0].world_transform[12], 0.0);
    assert_eq!(model.scenes[1].traversal[0].world_transform[12], 0.0);
}

#[test]
fn simple_meshes_decodes_shared_mesh_instances_and_trs_translation() {
    let root = fixture_root();
    let path = root.join("upstream/Models/SimpleMeshes/glTF/SimpleMeshes.gltf");
    let model = decode_gltf_file(path).expect("SimpleMeshes should decode");

    assert_eq!(model.primitives.len(), 1);
    assert_eq!(model.nodes.len(), 2);
    assert_eq!(model.nodes[0].mesh, Some(0));
    assert_eq!(model.nodes[1].mesh, Some(0));
    assert_eq!(model.scenes.len(), 1);
    assert_eq!(model.scenes[0].roots, [0, 1]);
    assert_eq!(model.scenes[0].traversal.len(), 2);
    assert_eq!(model.scenes[0].traversal[0].world_transform[12], 0.0);
    assert_eq!(model.scenes[0].traversal[1].world_transform[12], 1.0);
}

#[test]
fn mesh_primitive_modes_preserve_source_topology_and_reject_unsupported_lowering() {
    let root = fixture_root();
    let path = root.join("upstream/Models/MeshPrimitiveModes/glTF/MeshPrimitiveModes.gltf");
    let inspection = inspect_gltf_file(&path).expect("MeshPrimitiveModes should inspect");

    assert_eq!(inspection.summary.meshes, 7);
    assert_eq!(inspection.summary.primitives, 7);
    assert_eq!(inspection.primitive_topologies.len(), 7);
    assert_eq!(
        inspection
            .primitive_topologies
            .iter()
            .map(|topology| topology.mode)
            .collect::<Vec<_>>(),
        [0, 1, 2, 3, 4, 5, 6]
    );
    assert_eq!(
        inspection
            .primitive_topologies
            .iter()
            .map(|topology| (topology.mesh, topology.primitive))
            .collect::<Vec<_>>(),
        [(0, 0), (1, 0), (2, 0), (3, 0), (4, 0), (5, 0), (6, 0)]
    );

    let error = decode_gltf_file(path).expect_err("POINTS must not silently lower as triangles");
    assert!(matches!(error, CorpusError::UnsupportedAccessor(_)));
    assert!(error.to_string().contains("mode 0"));
    assert!(error.to_string().contains("only TRIANGLES is admitted"));
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

fn subtract(left: [f32; 3], right: [f32; 3]) -> [f32; 3] {
    [left[0] - right[0], left[1] - right[1], left[2] - right[2]]
}

fn cross(left: [f32; 3], right: [f32; 3]) -> [f32; 3] {
    [
        left[1] * right[2] - left[2] * right[1],
        left[2] * right[0] - left[0] * right[2],
        left[0] * right[1] - left[1] * right[0],
    ]
}

fn dot(left: [f32; 3], right: [f32; 3]) -> f32 {
    left[0] * right[0] + left[1] * right[1] + left[2] * right[2]
}
