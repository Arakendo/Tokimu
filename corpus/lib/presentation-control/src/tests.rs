use crate::{
    PresentationColor, PresentationControl, PresentationControlError, PresentationEmphasis,
    PresentationLayer, PresentationOverride, PresentationTargetDescriptor, PresentationTargetId,
    PresentationTargetKind, PresentationTint, SourcePresentation,
};

fn color(red: f32, green: f32, blue: f32) -> PresentationColor {
    PresentationColor::new(red, green, blue).expect("test color should be valid")
}

fn source(red: f32, green: f32, blue: f32) -> SourcePresentation {
    SourcePresentation::new(color(red, green, blue), 1.0, true)
        .expect("test source should be valid")
}

#[test]
fn vector_and_mesh_targets_use_the_same_override_vocabulary() {
    let vector =
        PresentationTargetId::new(PresentationTargetKind::VectorRecord, "picture/0/record/7")
            .unwrap();
    let mesh =
        PresentationTargetId::new(PresentationTargetKind::MeshPrimitive, "node/2/primitive/0")
            .unwrap();
    let mut control = PresentationControl::default();
    control
        .register_target(vector.clone(), source(0.4, 0.8, 0.6))
        .unwrap();
    control
        .register_target(mesh.clone(), source(0.8, 0.7, 0.5))
        .unwrap();

    let hotspot = PresentationOverride::default()
        .with_tint(PresentationTint::replace(color(1.0, 0.35, 0.1)))
        .with_opacity_multiplier(0.45)
        .unwrap()
        .with_emphasis(PresentationEmphasis::Hotspot);
    control
        .set_override(&vector, PresentationLayer::Hotspot, hotspot)
        .unwrap();
    control
        .set_override(&mesh, PresentationLayer::Hotspot, hotspot)
        .unwrap();

    assert_eq!(control.resolve(&vector), control.resolve(&mesh));
}

#[test]
fn override_layers_resolve_in_declared_order() {
    let target =
        PresentationTargetId::new(PresentationTargetKind::Renderable, "inspection-target").unwrap();
    let mut control = PresentationControl::default();
    control
        .register_target(target.clone(), source(0.8, 0.5, 0.25))
        .unwrap();
    control
        .set_override(
            &target,
            PresentationLayer::Theme,
            PresentationOverride::default()
                .with_tint(PresentationTint::multiply(color(0.5, 1.0, 1.0))),
        )
        .unwrap();
    control
        .set_override(
            &target,
            PresentationLayer::Selection,
            PresentationOverride::default()
                .with_tint(PresentationTint::replace(color(0.2, 0.4, 0.9)))
                .with_emphasis(PresentationEmphasis::Selected),
        )
        .unwrap();
    control
        .set_override(
            &target,
            PresentationLayer::Hotspot,
            PresentationOverride::default()
                .with_tint(PresentationTint::multiply(color(1.0, 0.5, 0.5)))
                .with_opacity_multiplier(0.5)
                .unwrap()
                .with_emphasis(PresentationEmphasis::Hotspot),
        )
        .unwrap();

    let resolved = control.resolve(&target).unwrap();
    assert_eq!(resolved.color, color(0.2, 0.2, 0.45));
    assert_eq!(resolved.opacity, 0.5);
    assert_eq!(resolved.emphasis, Some(PresentationEmphasis::Hotspot));
}

#[test]
fn clearing_one_target_restores_only_its_source_presentation() {
    let first = PresentationTargetId::new(PresentationTargetKind::MeshPrimitive, "mesh/0").unwrap();
    let second =
        PresentationTargetId::new(PresentationTargetKind::MeshPrimitive, "mesh/1").unwrap();
    let mut control = PresentationControl::default();
    control
        .register_target(first.clone(), source(0.2, 0.3, 0.4))
        .unwrap();
    control
        .register_target(second.clone(), source(0.7, 0.6, 0.5))
        .unwrap();
    let hidden = PresentationOverride::default().with_visibility(false);
    control
        .set_override(&first, PresentationLayer::Application, hidden)
        .unwrap();
    control
        .set_override(&second, PresentationLayer::Application, hidden)
        .unwrap();

    control.clear_target_overrides(&first).unwrap();

    assert!(control.resolve(&first).unwrap().visible);
    assert!(!control.resolve(&second).unwrap().visible);
    assert_eq!(control.resolve(&first).unwrap().color, color(0.2, 0.3, 0.4));
}

#[test]
fn invalid_values_and_unknown_targets_are_diagnosed() {
    assert_eq!(
        PresentationColor::new(f32::NAN, 0.0, 0.0),
        Err(PresentationControlError::InvalidUnitValue { field: "red" })
    );
    assert_eq!(
        PresentationTargetId::new(PresentationTargetKind::Renderable, " target"),
        Err(PresentationControlError::TargetKeyWhitespace)
    );

    let target = PresentationTargetId::new(PresentationTargetKind::Renderable, "missing").unwrap();
    let error = PresentationControl::default().resolve(&target).unwrap_err();
    assert_eq!(error, PresentationControlError::UnknownTarget { target });
}

#[test]
fn semantic_state_round_trips_without_renderer_data() {
    let target =
        PresentationTargetId::new(PresentationTargetKind::VectorRecord, "picture/0/record/1")
            .unwrap();
    let mut control = PresentationControl::default();
    control
        .register_target(target.clone(), source(0.25, 0.5, 0.75))
        .unwrap();
    control
        .set_override(
            &target,
            PresentationLayer::Warning,
            PresentationOverride::default()
                .with_tint(PresentationTint::multiply(color(1.0, 0.4, 0.2)))
                .with_opacity_multiplier(0.8)
                .unwrap()
                .with_emphasis(PresentationEmphasis::Warning),
        )
        .unwrap();

    let encoded = serde_json::to_string(&control).unwrap();
    let decoded: PresentationControl = serde_json::from_str(&encoded).unwrap();

    assert_eq!(decoded, control);
    assert_eq!(decoded.resolve(&target), control.resolve(&target));
    assert!(!encoded.contains("wgpu"));
    assert!(!encoded.contains("MaterialHandle"));
}

#[test]
fn stable_target_ids_distinguish_duplicate_source_names() {
    let first =
        PresentationTargetId::new(PresentationTargetKind::MeshPrimitive, "node/0/mesh/0").unwrap();
    let second =
        PresentationTargetId::new(PresentationTargetKind::MeshPrimitive, "node/1/mesh/0").unwrap();
    let first_descriptor = PresentationTargetDescriptor::new(first.clone())
        .with_source_name("Housing")
        .unwrap();
    let second_descriptor = PresentationTargetDescriptor::new(second.clone())
        .with_source_name("Housing")
        .unwrap();
    let mut control = PresentationControl::default();

    control
        .register_target_with_descriptor(first_descriptor, source(0.2, 0.4, 0.6))
        .unwrap();
    control
        .register_target_with_descriptor(second_descriptor, source(0.6, 0.4, 0.2))
        .unwrap();

    assert_ne!(first, second);
    assert_eq!(
        control
            .target_state(&first)
            .unwrap()
            .descriptor()
            .display_name(),
        "Housing"
    );
    assert_eq!(
        control
            .target_state(&second)
            .unwrap()
            .descriptor()
            .display_name(),
        "Housing"
    );
    assert_eq!(
        PresentationTargetDescriptor::new(first.clone()).display_name(),
        first.key()
    );
}
