use std::{
    collections::BTreeMap,
    path::{Path, PathBuf},
};

use fbx_corpus::{
    animation_json, compare_static_observations, decode_ascii_fbx_file, decode_binary_fbx_file,
    lower_static_geometry, morph_json, resolve_animations, resolve_material_slots,
    resolve_materials, resolve_morphs, resolve_skeletons, resolve_skins, resolve_source_scene,
    resolve_transforms, skeleton_json, skinning_json, source_records_json, FbxLimits, FbxProperty,
    FbxRecord, STATIC_OBSERVATION_COMPARISON_ALGORITHM, STATIC_OBSERVATION_COMPARISON_SCHEMA,
};

#[test]
fn decodes_selected_legacy_binary_cube_deterministically() {
    let path = fixture("maya_cube_6100_binary.fbx");
    let first = decode_binary_fbx_file(&path, FbxLimits::default()).unwrap();
    let second = decode_binary_fbx_file(&path, FbxLimits::default()).unwrap();

    assert_eq!(first, second);
    assert_eq!(first.version, 6100);
    assert!(!first.records.is_empty());
    assert!(first.records.iter().any(|record| record.name == "Objects"));
    assert_eq!(
        source_records_json(&first).unwrap(),
        source_records_json(&second).unwrap()
    );
}

#[test]
fn decodes_selected_modern_binary_cube() {
    let document =
        decode_binary_fbx_file(fixture("maya_cube_7500_binary.fbx"), FbxLimits::default()).unwrap();

    assert_eq!(document.version, 7500);
    assert!(document
        .records
        .iter()
        .any(|record| record.name == "Objects"));
    assert!(all_records(&document.records).any(|record| {
        record.properties.iter().any(|property| {
            matches!(
                property,
                FbxProperty::F32Array(_)
                    | FbxProperty::F64Array(_)
                    | FbxProperty::I32Array(_)
                    | FbxProperty::I64Array(_)
            )
        })
    }));
    assert!(document.footer_offset < document.source_bytes);
}

#[test]
fn resolves_selected_maya_ascii_cube_source_graph() {
    let ascii_document =
        decode_ascii_fbx_file(fixture("maya_cube_7500_ascii.fbx"), FbxLimits::default()).unwrap();
    let repeated_ascii_document =
        decode_ascii_fbx_file(fixture("maya_cube_7500_ascii.fbx"), FbxLimits::default()).unwrap();
    let binary_document =
        decode_binary_fbx_file(fixture("maya_cube_7500_binary.fbx"), FbxLimits::default()).unwrap();
    let ascii_scene = resolve_source_scene(&ascii_document).unwrap();
    let binary_scene = resolve_source_scene(&binary_document).unwrap();
    let ascii_geometry = lower_static_geometry(&ascii_document, &ascii_scene).unwrap();
    let binary_geometry = lower_static_geometry(&binary_document, &binary_scene).unwrap();
    let ascii_transforms = resolve_transforms(&ascii_document, &ascii_scene).unwrap();
    let binary_transforms = resolve_transforms(&binary_document, &binary_scene).unwrap();
    let ascii_materials = resolve_materials(&ascii_document, &ascii_scene).unwrap();
    let binary_materials = resolve_materials(&binary_document, &binary_scene).unwrap();
    let comparison = compare_static_observations(
        &ascii_scene,
        &ascii_geometry,
        &ascii_transforms,
        &binary_scene,
        &binary_geometry,
        &binary_transforms,
    );

    assert_eq!(ascii_document.version, 7500);
    assert!(comparison.equivalent, "{comparison:#?}");
    assert_eq!(comparison.schema, STATIC_OBSERVATION_COMPARISON_SCHEMA);
    assert_eq!(
        comparison.algorithm,
        STATIC_OBSERVATION_COMPARISON_ALGORITHM
    );
    assert_eq!(
        source_records_json(&ascii_document).unwrap(),
        source_records_json(&repeated_ascii_document).unwrap()
    );
    assert_eq!(
        ascii_scene
            .objects
            .iter()
            .map(|object| (&object.kind, &object.class))
            .collect::<Vec<_>>(),
        binary_scene
            .objects
            .iter()
            .map(|object| (&object.kind, &object.class))
            .collect::<Vec<_>>()
    );
    assert_eq!(
        ascii_scene
            .connections
            .iter()
            .map(|connection| { (&connection.relation, connection.property.as_deref(),) })
            .collect::<Vec<_>>(),
        binary_scene
            .connections
            .iter()
            .map(|connection| { (&connection.relation, connection.property.as_deref(),) })
            .collect::<Vec<_>>()
    );
    assert_eq!(ascii_scene.nodes.len(), binary_scene.nodes.len());
    assert_eq!(ascii_geometry.meshes.len(), binary_geometry.meshes.len());
    for (ascii_mesh, binary_mesh) in ascii_geometry.meshes.iter().zip(&binary_geometry.meshes) {
        // Source IDs, labels, and offsets are encoding-local evidence. The mesh
        // contract is the topology and attribute data that an importer exposes.
        assert_eq!(ascii_mesh.control_points, binary_mesh.control_points);
        assert_eq!(ascii_mesh.polygons, binary_mesh.polygons);
        assert_eq!(ascii_mesh.triangles, binary_mesh.triangles);
        assert_eq!(ascii_mesh.normal_layer, binary_mesh.normal_layer);
        assert_eq!(ascii_mesh.uv_layer, binary_mesh.uv_layer);
        assert_eq!(ascii_mesh.material_layer, binary_mesh.material_layer);
        assert_eq!(ascii_mesh.bounds, binary_mesh.bounds);
    }
    assert_eq!(ascii_transforms.axes, binary_transforms.axes);
    assert_eq!(ascii_transforms.nodes.len(), binary_transforms.nodes.len());
    for (ascii_node, binary_node) in ascii_transforms.nodes.iter().zip(&binary_transforms.nodes) {
        // FBX object IDs and record offsets remain local to an encoding; the
        // ordered transform values and matrices are the shared observation.
        assert_eq!(ascii_node.local_translation, binary_node.local_translation);
        assert_eq!(
            ascii_node.local_rotation_degrees_xyz,
            binary_node.local_rotation_degrees_xyz
        );
        assert_eq!(ascii_node.local_scale, binary_node.local_scale);
        assert_eq!(ascii_node.local_matrix, binary_node.local_matrix);
        assert_eq!(ascii_node.world_matrix, binary_node.world_matrix);
    }
    assert_eq!(
        ascii_materials.materials.len(),
        binary_materials.materials.len()
    );
    assert_eq!(
        ascii_materials.textures.len(),
        binary_materials.textures.len()
    );
    assert_eq!(
        ascii_materials
            .bindings
            .iter()
            .map(|binding| (&binding.relation, binding.property.as_deref()))
            .collect::<Vec<_>>(),
        binary_materials
            .bindings
            .iter()
            .map(|binding| (&binding.relation, binding.property.as_deref()))
            .collect::<Vec<_>>(),
    );

    let mut divergent_geometry = ascii_geometry.clone();
    divergent_geometry.meshes[0].control_points[0][0] += 1.0;
    let divergence = compare_static_observations(
        &ascii_scene,
        &divergent_geometry,
        &ascii_transforms,
        &binary_scene,
        &binary_geometry,
        &binary_transforms,
    );
    assert!(!divergence.equivalent);
    assert!(matches!(
        divergence.first_difference.as_ref(),
        Some(difference)
            if difference.stage == "geometry" && difference.observation == "mesh 0 control points"
    ));
}

#[test]
fn decodes_selected_maya_6100_ascii_records_without_claiming_legacy_name_graph() {
    let document =
        decode_ascii_fbx_file(fixture("maya_cube_6100_ascii.fbx"), FbxLimits::default()).unwrap();

    let objects = document
        .records
        .iter()
        .find(|record| record.name == "Objects")
        .expect("legacy fixture retains an Objects record");
    assert!(objects.children.iter().any(|record| {
        record.name == "Model"
            && matches!(
                record.properties.as_slice(),
                [FbxProperty::String(name), FbxProperty::String(class)]
                    if name == "Model::pCube1" && class == "Mesh"
            )
            && record.children.iter().any(|child| {
                child.name == "Vertices"
                    && matches!(
                        child.properties.as_slice(),
                        [FbxProperty::F64Array(values)] if values.len() == 24
                    )
            })
    }));

    let error = resolve_source_scene(&document)
        .expect_err("legacy named Connect relationships must not be coerced into numeric IDs");
    assert!(matches!(
        error,
        fbx_corpus::FbxError::SourceGraph { reason, .. }
            if reason.contains("unsupported connection record `Connect`")
    ));
}

#[test]
fn rejects_selected_malformed_ascii_source_inputs() {
    for fixture_name in [
        "synthetic_truncated_quot_fail_7500_ascii.fbx",
        "synthetic_bad_inf_nan_fail_7700_ascii.fbx",
    ] {
        let error = decode_ascii_fbx_file(fixture(fixture_name), FbxLimits::default())
            .expect_err("selected malformed ASCII fixture should not produce source records");
        assert!(matches!(error, fbx_corpus::FbxError::AsciiSyntax { .. }));
    }
}

#[test]
fn lowers_selected_big_endian_array_cube_into_finite_geometry_evidence() {
    let document = decode_binary_fbx_file(
        fixture("maya_cube_big_endian_7500_binary.fbx"),
        FbxLimits::default(),
    )
    .unwrap();
    let scene = resolve_source_scene(&document).unwrap();
    let geometry = lower_static_geometry(&document, &scene).unwrap();
    let little_endian_document =
        decode_binary_fbx_file(fixture("maya_cube_7500_binary.fbx"), FbxLimits::default()).unwrap();
    let little_endian_scene = resolve_source_scene(&little_endian_document).unwrap();
    let little_endian_geometry =
        lower_static_geometry(&little_endian_document, &little_endian_scene).unwrap();

    assert_eq!(document.version, 7500);
    assert_eq!(document.byte_order, fbx_corpus::FbxByteOrder::BigEndian);
    assert!(!geometry.meshes.is_empty());
    assert!(geometry.meshes.iter().all(|mesh| {
        mesh.control_points
            .iter()
            .flatten()
            .all(|value| value.is_finite())
            && !mesh.polygons.is_empty()
            && !mesh.triangles.is_empty()
    }));
    assert_eq!(geometry.meshes.len(), little_endian_geometry.meshes.len());
    assert_eq!(
        geometry.meshes[0].control_points.len(),
        little_endian_geometry.meshes[0].control_points.len()
    );
    assert_eq!(
        geometry.meshes[0].bounds,
        little_endian_geometry.meshes[0].bounds
    );
}

#[test]
fn resolves_selected_cube_source_graph_deterministically() {
    let document =
        decode_binary_fbx_file(fixture("maya_cube_7500_binary.fbx"), FbxLimits::default()).unwrap();
    let first = resolve_source_scene(&document).unwrap();
    let second = resolve_source_scene(&document).unwrap();

    assert_eq!(first, second);
    assert!(first.objects.iter().any(|object| object.kind == "Model"));
    assert!(first.objects.iter().any(|object| object.kind == "Geometry"));
    assert!(!first.connections.is_empty());
    assert!(first.nodes.iter().any(|node| !node.geometry_ids.is_empty()));
}

#[test]
fn lowers_selected_cube_into_static_geometry_evidence() {
    let document =
        decode_binary_fbx_file(fixture("maya_cube_7500_binary.fbx"), FbxLimits::default()).unwrap();
    let scene = resolve_source_scene(&document).unwrap();
    let evidence = lower_static_geometry(&document, &scene).unwrap();

    assert!(!evidence.meshes.is_empty());
    assert!(evidence
        .meshes
        .iter()
        .all(|mesh| !mesh.control_points.is_empty()));
    assert!(evidence.meshes.iter().all(|mesh| !mesh.polygons.is_empty()));
    assert!(evidence
        .meshes
        .iter()
        .all(|mesh| !mesh.triangles.is_empty()));
    assert!(evidence.meshes.iter().all(|mesh| mesh
        .control_points
        .iter()
        .flatten()
        .all(|value| value.is_finite())));
}

#[test]
fn preserves_selected_blender_uv_layer_as_source_metadata() {
    let document = decode_binary_fbx_file(
        fixture("blender_279_uv_sets_7400_binary.fbx"),
        FbxLimits::default(),
    )
    .unwrap();
    let scene = resolve_source_scene(&document).unwrap();
    let geometry = lower_static_geometry(&document, &scene).unwrap();

    let layers = geometry
        .meshes
        .iter()
        .filter_map(|mesh| mesh.uv_layer.as_ref())
        .collect::<Vec<_>>();
    assert!(!layers.is_empty());
    assert!(layers.iter().all(|layer| {
        !layer.mapping.is_empty()
            && !layer.reference.is_empty()
            && !layer.values.is_empty()
            && layer.values.iter().flatten().all(|value| value.is_finite())
            && layer.indices.as_ref().is_none_or(|indices| {
                indices
                    .iter()
                    .all(|index| (*index as usize) < layer.values.len())
            })
    }));
}

#[test]
fn decodes_selected_blender_ascii_uv_records_without_claiming_legacy_name_graph() {
    let document = decode_ascii_fbx_file(
        fixture("blender_279_uv_sets_6100_ascii.fbx"),
        FbxLimits::default(),
    )
    .unwrap();
    let layer = all_records(&document.records)
        .find(|record| record.name == "LayerElementUV")
        .expect("legacy ASCII fixture retains a UV layer record");
    let uv_values = layer
        .children
        .iter()
        .find(|record| record.name == "UV")
        .and_then(|record| record.properties.first())
        .expect("UV layer retains its direct coordinate array");
    let uv_indices = layer
        .children
        .iter()
        .find(|record| record.name == "UVIndex")
        .and_then(|record| record.properties.first())
        .expect("UV layer retains its index array");

    assert!(matches!(uv_values, FbxProperty::F64Array(values) if !values.is_empty()));
    assert!(matches!(uv_indices, FbxProperty::I32Array(values) if !values.is_empty()));
    assert!(matches!(
        resolve_source_scene(&document),
        Err(fbx_corpus::FbxError::SourceGraph { reason, .. })
            if reason.contains("unsupported connection record `Connect`")
    ));
}

#[test]
fn resolves_selected_cube_transforms_deterministically() {
    let document =
        decode_binary_fbx_file(fixture("maya_cube_7500_binary.fbx"), FbxLimits::default()).unwrap();
    let scene = resolve_source_scene(&document).unwrap();
    let first = resolve_transforms(&document, &scene).unwrap();
    let second = resolve_transforms(&document, &scene).unwrap();

    assert_eq!(first, second);
    assert_eq!(first.nodes.len(), scene.nodes.len());
    assert!(first
        .nodes
        .iter()
        .all(|node| node.local_matrix.iter().all(|value| value.is_finite())));
    assert!(first
        .nodes
        .iter()
        .all(|node| node.world_matrix.iter().all(|value| value.is_finite())));
}

#[test]
fn preserves_shared_geometry_across_blender_instance_nodes() {
    let document = decode_binary_fbx_file(
        fixture("blender_293_instancing_7400_binary.fbx"),
        FbxLimits::default(),
    )
    .unwrap();
    let scene = resolve_source_scene(&document).unwrap();
    let transforms = resolve_transforms(&document, &scene).unwrap();

    let shared_geometry = scene
        .nodes
        .iter()
        .flat_map(|node| node.geometry_ids.iter().copied())
        .fold(
            std::collections::BTreeMap::<i64, usize>::new(),
            |mut counts, id| {
                *counts.entry(id).or_default() += 1;
                counts
            },
        );
    assert!(shared_geometry.values().any(|count| *count > 1));
    assert_eq!(transforms.nodes.len(), scene.nodes.len());
    assert!(transforms
        .nodes
        .iter()
        .all(|node| node.world_matrix.iter().all(|value| value.is_finite())));
}

#[test]
fn preserves_selected_max_unicode_source_identity() {
    let binary_document =
        decode_binary_fbx_file(fixture("max_unicode_7500_binary.fbx"), FbxLimits::default())
            .unwrap();
    let ascii_document =
        decode_ascii_fbx_file(fixture("max_unicode_7500_ascii.fbx"), FbxLimits::default()).unwrap();
    let first = resolve_source_scene(&binary_document).unwrap();
    let second = resolve_source_scene(&binary_document).unwrap();
    let ascii_scene = resolve_source_scene(&ascii_document).unwrap();

    assert_eq!(first, second);
    assert!(first.objects.iter().any(|object| !object.name.is_ascii()));
    assert!(ascii_scene
        .objects
        .iter()
        .any(|object| object.name.contains("aβカ😂")));
}

#[test]
fn records_distinct_blender_y_up_and_z_up_axis_metadata() {
    let y_up = transform_fixture("blender_340_y_up_7400_binary.fbx");
    let z_up = transform_fixture("blender_340_z_up_7400_binary.fbx");

    assert!(y_up.axes.up_axis.is_some());
    assert!(z_up.axes.up_axis.is_some());
    for axes in [&y_up.axes, &z_up.axes] {
        assert!(axes.up_axis_sign.is_some());
        assert!(axes.front_axis.is_some());
        assert!(axes.front_axis_sign.is_some());
        assert!(axes.coord_axis.is_some());
        assert!(axes.coord_axis_sign.is_some());
        assert!(axes
            .unit_scale_factor
            .is_some_and(|value| value.is_finite() && value > 0.0));
    }
    assert_ne!(y_up.axes, z_up.axes);
}

#[test]
fn preserves_selected_max_materials_and_connection_evidence() {
    let document = decode_binary_fbx_file(
        fixture("max_gltf_material_7700_binary.fbx"),
        FbxLimits::default(),
    )
    .unwrap();
    let scene = resolve_source_scene(&document).unwrap();
    let first = resolve_materials(&document, &scene).unwrap();
    let second = resolve_materials(&document, &scene).unwrap();

    assert_eq!(first, second);
    assert!(!first.materials.is_empty());
    assert!(first.materials.iter().all(|material| material
        .properties
        .iter()
        .all(|property| !property.name.is_empty())));
    assert!(!first.textures.is_empty());
    assert!(first.textures.iter().all(|texture| {
        texture
            .file_name
            .as_deref()
            .or(texture.relative_file_name.as_deref())
            .is_some_and(|path| !path.is_empty())
    }));
    assert!(first.textures.iter().any(|texture| {
        texture
            .file_name
            .as_deref()
            .or(texture.relative_file_name.as_deref())
            .is_some_and(|path| path.contains("checkerboard"))
    }));
    assert!(!first.bindings.is_empty());
    assert!(first.bindings.iter().any(|binding| binding
        .property
        .as_deref()
        .is_some_and(|name| !name.is_empty())));
}

#[test]
fn resolves_selected_ascii_material_source_evidence_against_binary_peer() {
    let ascii_document = decode_ascii_fbx_file(
        fixture("max_gltf_material_7700_ascii.fbx"),
        FbxLimits::default(),
    )
    .unwrap();
    let binary_document = decode_binary_fbx_file(
        fixture("max_gltf_material_7700_binary.fbx"),
        FbxLimits::default(),
    )
    .unwrap();
    let ascii_scene = resolve_source_scene(&ascii_document).unwrap();
    let binary_scene = resolve_source_scene(&binary_document).unwrap();
    let ascii = resolve_materials(&ascii_document, &ascii_scene).unwrap();
    let binary = resolve_materials(&binary_document, &binary_scene).unwrap();

    assert_eq!(ascii.materials.len(), binary.materials.len());
    assert_eq!(ascii.textures.len(), binary.textures.len());
    assert_eq!(ascii.bindings.len(), binary.bindings.len());
    assert!(!ascii.materials.is_empty());
    assert!(ascii.textures.iter().any(|texture| {
        texture
            .file_name
            .as_deref()
            .or(texture.relative_file_name.as_deref())
            .is_some_and(|path| path.contains("checkerboard"))
    }));
}

#[test]
fn resolves_selected_max_polygon_material_slots() {
    let document = decode_binary_fbx_file(
        fixture("max_gltf_material_7700_binary.fbx"),
        FbxLimits::default(),
    )
    .unwrap();
    let scene = resolve_source_scene(&document).unwrap();
    let geometry = lower_static_geometry(&document, &scene).unwrap();
    let slots = resolve_material_slots(&scene, &geometry).unwrap();

    assert!(!slots.is_empty());
    assert!(slots.iter().all(|slot| !slot.material_ids.is_empty()));
    assert!(slots.iter().all(|slot| slot.polygon_material_slots.len()
        == geometry
            .meshes
            .iter()
            .find(|mesh| mesh.source_id == slot.geometry_id)
            .expect("slot geometry is present")
            .polygons
            .len()));
}

#[test]
fn resolves_modern_blender_animation_curve_evidence() {
    let document = decode_binary_fbx_file(
        fixture("blender440_shape_weight_anim_7400_binary.fbx"),
        FbxLimits::default(),
    )
    .unwrap();
    let scene = resolve_source_scene(&document).unwrap();
    let first = resolve_animations(&document, &scene).unwrap();
    let second = resolve_animations(&document, &scene).unwrap();

    assert_eq!(first, second);
    assert_eq!(
        animation_json(&first).unwrap(),
        animation_json(&second).unwrap()
    );
    assert!(!first.stacks.is_empty());
    assert!(!first.layers.is_empty());
    assert!(!first.channels.is_empty());
    assert!(first.channels.iter().any(|channel| {
        channel.target_id.is_some()
            && channel
                .target_property
                .as_deref()
                .is_some_and(|property| !property.is_empty())
    }));
    assert!(first.channels.iter().all(|channel| {
        channel.key_times.len() == channel.key_values.len()
            && channel.key_times.windows(2).all(|pair| pair[0] < pair[1])
            && channel.key_values.iter().all(|value| value.is_finite())
            && channel
                .key_attr_flags
                .as_ref()
                .is_none_or(|flags| !flags.is_empty())
    }));
    assert!(first
        .channels
        .iter()
        .any(|channel| channel.key_attr_flags.is_some()));
}

#[test]
fn reports_unsupported_legacy_animation_header_before_semantic_lowering() {
    let error = decode_binary_fbx_file(
        fixture("max2009_cube_anim_5800_binary.fbx"),
        FbxLimits::default(),
    )
    .expect_err("the bounded binary decoder should reject the fixture's legacy header");

    assert!(matches!(
        error,
        fbx_corpus::FbxError::UnsupportedVersion { version: 3000 }
    ));
}

#[test]
fn decodes_legacy_ascii_animation_records_without_claiming_modern_source_graph() {
    let document = decode_ascii_fbx_file(
        fixture("max2009_cube_anim_5800_ascii.fbx"),
        FbxLimits::default(),
    )
    .unwrap();

    let takes = document
        .records
        .iter()
        .find(|record| record.name == "Takes")
        .expect("legacy animation source retains its Takes record");
    assert!(all_records(&takes.children).any(|record| {
        record.name == "Key"
            && record.properties.iter().any(|property| {
                matches!(property, FbxProperty::String(value) if matches!(value.as_str(), "U" | "s" | "a" | "r" | "n"))
            })
    }));

    let error = resolve_source_scene(&document).expect_err(
        "legacy top-level model records must not be coerced into the modern source graph",
    );
    assert!(matches!(
        error,
        fbx_corpus::FbxError::SourceGraph { reason, .. }
            if reason.contains("missing required top-level `Objects` record")
    ));
}

#[test]
fn resolves_selected_max_transformed_skin_evidence() {
    let document = decode_binary_fbx_file(
        fixture("max_transformed_skin_7500_binary.fbx"),
        FbxLimits::default(),
    )
    .unwrap();
    let scene = resolve_source_scene(&document).unwrap();
    let geometry = lower_static_geometry(&document, &scene).unwrap();
    let first = resolve_skins(&document, &scene, &geometry).unwrap();
    let second = resolve_skins(&document, &scene, &geometry).unwrap();
    let skeletons = resolve_skeletons(&scene, &first).unwrap();
    let skeletons_second = resolve_skeletons(&scene, &second).unwrap();

    assert_eq!(first, second);
    assert_eq!(
        skinning_json(&first).unwrap(),
        skinning_json(&second).unwrap()
    );
    assert!(!first.skins.is_empty());
    assert!(first.clusters.len() > 1);
    assert!(first.clusters.iter().all(|cluster| {
        cluster.control_point_indices.len() == cluster.weights.len()
            && !cluster.control_point_indices.is_empty()
            && cluster
                .weights
                .iter()
                .all(|weight| weight.is_finite() && *weight >= 0.0)
            && cluster.transform.iter().all(|value| value.is_finite())
            && cluster.transform_link.iter().all(|value| value.is_finite())
    }));
    assert!(first.skins.iter().all(|skin| {
        skin.weight_summary.influenced_control_point_count > 0
            && skin
                .weight_summary
                .minimum_influenced_sum
                .is_some_and(|sum| sum.is_finite() && sum > 0.0)
            && skin.weight_summary.maximum_influenced_sum.is_finite()
            && skin.weight_summary.maximum_influenced_sum <= 1.0 + 1.0e-5
    }));
    assert!(skeletons.joints.len() > 1);
    assert_eq!(skeletons, skeletons_second);
    assert_eq!(
        skeleton_json(&skeletons).unwrap(),
        skeleton_json(&skeletons_second).unwrap()
    );
    assert!(skeletons.joints.iter().all(|joint| {
        joint.parent_joint_source_id.is_none_or(|parent| {
            skeletons
                .joints
                .iter()
                .any(|candidate| candidate.source_id == parent)
        })
    }));
    assert!(skeletons
        .joints
        .iter()
        .any(|joint| !joint.cluster_ids.is_empty()));
}

#[test]
fn resolves_selected_ascii_skin_evidence_against_its_binary_peer() {
    let ascii_document = decode_ascii_fbx_file(
        fixture("max_transformed_skin_7500_ascii.fbx"),
        FbxLimits::default(),
    )
    .unwrap();
    let binary_document = decode_binary_fbx_file(
        fixture("max_transformed_skin_7500_binary.fbx"),
        FbxLimits::default(),
    )
    .unwrap();
    let ascii_scene = resolve_source_scene(&ascii_document).unwrap();
    let binary_scene = resolve_source_scene(&binary_document).unwrap();
    let ascii_geometry = lower_static_geometry(&ascii_document, &ascii_scene).unwrap();
    let binary_geometry = lower_static_geometry(&binary_document, &binary_scene).unwrap();
    let ascii_skins = resolve_skins(&ascii_document, &ascii_scene, &ascii_geometry).unwrap();
    let binary_skins = resolve_skins(&binary_document, &binary_scene, &binary_geometry).unwrap();
    let ascii_skeletons = resolve_skeletons(&ascii_scene, &ascii_skins).unwrap();
    let binary_skeletons = resolve_skeletons(&binary_scene, &binary_skins).unwrap();

    assert_eq!(ascii_skins.skins.len(), binary_skins.skins.len());
    assert_eq!(ascii_skins.clusters.len(), binary_skins.clusters.len());
    assert_eq!(
        ascii_skins
            .clusters
            .iter()
            .map(|cluster| &cluster.link_mode)
            .collect::<Vec<_>>(),
        binary_skins
            .clusters
            .iter()
            .map(|cluster| &cluster.link_mode)
            .collect::<Vec<_>>(),
        "paired skin fixtures must agree on provider-local Link_Mode presence and value"
    );
    assert_skin_cluster_observations_match(
        &ascii_scene,
        &ascii_skins,
        &binary_scene,
        &binary_skins,
    );
    assert_eq!(ascii_skeletons.joints.len(), binary_skeletons.joints.len());
    assert_eq!(
        skeleton_observations(&ascii_skeletons),
        skeleton_observations(&binary_skeletons)
    );
}

#[test]
fn separates_valid_ascii_skin_syntax_from_broken_cluster_semantics() {
    let document = decode_ascii_fbx_file(
        fixture("synthetic_broken_cluster_7500_ascii.fbx"),
        FbxLimits::default(),
    )
    .expect("the deliberately broken cluster fixture is valid bounded ASCII FBX");
    let scene = resolve_source_scene(&document)
        .expect("all connection endpoints exist even though one cluster has no joint model");
    let geometry = lower_static_geometry(&document, &scene)
        .expect("the static mesh remains independently valid");
    let error = resolve_skins(&document, &scene, &geometry)
        .expect_err("a cluster without a model connection is invalid skinning evidence");

    assert!(matches!(
        error,
        fbx_corpus::FbxError::Skinning { reason, .. }
            if reason.contains("has no model connection")
    ));
}

#[test]
fn resolves_selected_blender_static_blend_shape_evidence() {
    let document = decode_binary_fbx_file(
        fixture("blender_331_static_blend_shape_7400_binary.fbx"),
        FbxLimits::default(),
    )
    .unwrap();
    let scene = resolve_source_scene(&document).unwrap();
    let geometry = lower_static_geometry(&document, &scene).unwrap();
    let first = resolve_morphs(&document, &scene, &geometry).unwrap();
    let second = resolve_morphs(&document, &scene, &geometry).unwrap();

    assert_eq!(first, second);
    assert_eq!(morph_json(&first).unwrap(), morph_json(&second).unwrap());
    assert!(!first.blend_shapes.is_empty());
    assert!(!first.channels.is_empty());
    assert!(!first.targets.is_empty());
    assert!(first.targets.iter().all(|target| {
        target.control_point_indices.len() == target.position_values.len()
            && !target.control_point_indices.is_empty()
            && target
                .position_values
                .iter()
                .flatten()
                .all(|value| value.is_finite())
    }));
}

#[test]
fn preserves_animated_blend_shape_as_separate_morph_and_animation_evidence() {
    let document = decode_binary_fbx_file(
        fixture("blender440_shape_weight_anim_7400_binary.fbx"),
        FbxLimits::default(),
    )
    .unwrap();
    let scene = resolve_source_scene(&document).unwrap();
    let geometry = lower_static_geometry(&document, &scene).unwrap();
    let animations = resolve_animations(&document, &scene).unwrap();
    let morphs = resolve_morphs(&document, &scene, &geometry).unwrap();

    assert!(!animations.channels.is_empty());
    assert!(!morphs.blend_shapes.is_empty());
    assert!(!morphs.channels.is_empty());
    assert!(!morphs.targets.is_empty());
    assert!(animations
        .channels
        .iter()
        .any(|channel| channel.target_id.is_some()));
    assert!(morphs
        .targets
        .iter()
        .all(|target| { target.control_point_indices.len() == target.position_values.len() }));
}

fn transform_fixture(name: &str) -> fbx_corpus::FbxTransformEvidence {
    let document = decode_binary_fbx_file(fixture(name), FbxLimits::default()).unwrap();
    let scene = resolve_source_scene(&document).unwrap();
    resolve_transforms(&document, &scene).unwrap()
}

fn fixture(name: &str) -> PathBuf {
    workspace_root()
        .join("third-party/fixtures/fbx-corpus/upstream/data")
        .join(name)
}

fn assert_skin_cluster_observations_match(
    left_scene: &fbx_corpus::FbxSourceScene,
    left_skins: &fbx_corpus::FbxSkinEvidence,
    right_scene: &fbx_corpus::FbxSourceScene,
    right_skins: &fbx_corpus::FbxSkinEvidence,
) {
    let left = skin_cluster_observations(left_scene, left_skins);
    let right = skin_cluster_observations(right_scene, right_skins);
    assert_eq!(
        left.keys().collect::<Vec<_>>(),
        right.keys().collect::<Vec<_>>()
    );

    for (joint_name, left_influences) in left {
        let right_influences = &right[&joint_name];
        assert_eq!(
            left_influences.len(),
            right_influences.len(),
            "joint `{joint_name}` has a different influence count"
        );
        for ((left_index, left_weight), (right_index, right_weight)) in
            left_influences.iter().zip(right_influences)
        {
            assert_eq!(
                left_index, right_index,
                "joint `{joint_name}` references different control points"
            );
            // Binary FBX may store a source weight as f32 while ASCII writes
            // a decimal expansion. This compares the source observation at a
            // precision that does not mistake that representation detail for
            // a changed skin influence.
            assert!(
                (left_weight - right_weight).abs() <= 1.0e-6,
                "joint `{joint_name}` weight differs: {left_weight} vs {right_weight}"
            );
        }
    }
}

fn skin_cluster_observations(
    scene: &fbx_corpus::FbxSourceScene,
    skins: &fbx_corpus::FbxSkinEvidence,
) -> BTreeMap<String, Vec<(u32, f64)>> {
    let names = scene
        .objects
        .iter()
        .map(|object| (object.source_id, object.name.clone()))
        .collect::<BTreeMap<_, _>>();
    let mut observations = BTreeMap::new();
    for cluster in &skins.clusters {
        let mut influences = cluster
            .control_point_indices
            .iter()
            .copied()
            .zip(cluster.weights.iter().copied())
            .collect::<Vec<_>>();
        influences.sort_by_key(|(index, _)| *index);
        let joint_name = names
            .get(&cluster.joint_model_id)
            .expect("resolved skin cluster joint is present in source evidence")
            .as_str();
        let joint_name = logical_source_name(joint_name);
        assert!(
            observations
                .insert(joint_name.clone(), influences)
                .is_none(),
            "fixture has more than one selected cluster for joint `{joint_name}`"
        );
    }
    observations
}

fn logical_source_name(name: &str) -> String {
    // ASCII FBX typically prefixes an object label with `Kind::`, while
    // binary FBX commonly appends a NUL-delimited class tag. Those are source
    // container spellings, not a change to the logical object label used by
    // this paired-encoding observation.
    let name = name.split('\0').next().unwrap_or(name);
    name.rsplit_once("::")
        .map_or_else(|| name.to_owned(), |(_, label)| label.to_owned())
}

fn skeleton_observations(
    skeletons: &fbx_corpus::FbxSkeletonEvidence,
) -> Vec<(String, bool, usize)> {
    let mut observations = skeletons
        .joints
        .iter()
        .map(|joint| {
            (
                logical_source_name(&joint.name),
                joint.parent_joint_source_id.is_some(),
                joint.cluster_ids.len(),
            )
        })
        .collect::<Vec<_>>();
    observations.sort_unstable();
    observations
}

fn workspace_root() -> &'static Path {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .ancestors()
        .nth(3)
        .expect("fbx-corpus is nested three levels below the workspace root")
}

fn all_records(records: &[FbxRecord]) -> Box<dyn Iterator<Item = &FbxRecord> + '_> {
    Box::new(
        records
            .iter()
            .flat_map(|record| std::iter::once(record).chain(all_records(&record.children))),
    )
}
