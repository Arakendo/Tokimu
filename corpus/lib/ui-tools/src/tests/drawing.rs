use super::super::*;

#[test]
fn drawer_emits_surface_and_text_commands() {
    let theme = UiTheme::default();
    let mut surfaces = Vec::new();
    let mut text = Vec::new();
    let mut drawer = UiDrawer::new(&mut surfaces, &mut text, &theme);
    let button = UiButton::new(UiButtonId(7), "edit", UiRect::new([0.0, 0.0], [0.4, 0.1]));

    drawer.button(&button, UiInteractionState::Hovered, UiControlRole::Primary);

    assert_eq!(surfaces.len(), 1);
    assert_eq!(text.len(), 1);
    assert_eq!(surfaces[0].style.role, UiSurfaceRole::Panel);
    assert_eq!(surfaces[0].style.border_role, Some(UiSurfaceRole::Accent));
    assert_eq!(text[0].spec.text, "edit");
}

#[test]
fn drawer_clips_commands_to_the_active_clip() {
    let theme = UiTheme::default();
    let mut surfaces = Vec::new();
    let mut text = Vec::new();
    let button = UiButton::new(UiButtonId(7), "edit", UiRect::new([0.5, 0.0], [1.0, 0.4]));

    {
        let mut drawer = UiDrawer::new(&mut surfaces, &mut text, &theme);
        drawer.set_clip(Some(UiRect::new([0.0, 0.0], [1.0, 1.0])));
        drawer.button(&button, UiInteractionState::Hovered, UiControlRole::Primary);
    }

    assert_eq!(surfaces[0].rect, button.rect);
    assert_eq!(surfaces[0].clip, Some(UiRect::new([0.0, 0.0], [1.0, 1.0])));
    assert_eq!(text[0].spec.rect, UiRect::new([0.25, 0.0], [0.5, 0.4]));

    {
        let mut drawer = UiDrawer::new(&mut surfaces, &mut text, &theme);
        drawer.set_clip(Some(UiRect::new([-2.0, 0.0], [1.0, 1.0])));
        drawer.button(&button, UiInteractionState::Hovered, UiControlRole::Primary);
    }
    assert_eq!(surfaces.len(), 1);
    assert_eq!(text.len(), 1);
}

#[test]
fn surface_lowering_preserves_shadow_border_fill_order() {
    let theme = UiTheme::default();
    let command = UiSurfaceCommand {
        rect: UiRect::new([0.0, 0.0], [0.4, 0.2]),
        style: theme.surface(UiSurfaceRole::Panel),
        clip: None,
    };

    let layers = lower_surface_to_vector(&command);

    assert_eq!(layers.len(), 3);
    assert_eq!(layers[0].kind, UiSurfaceVectorLayerKind::Shadow);
    assert_eq!(layers[1].kind, UiSurfaceVectorLayerKind::Border);
    assert_eq!(layers[2].kind, UiSurfaceVectorLayerKind::Fill);
    assert_eq!(layers[1].role, UiSurfaceRole::Overlay);
    assert_eq!(layers[2].role, UiSurfaceRole::Panel);
    assert!(layers.iter().all(|layer| layer.clip.is_none()));
    assert!(layers.iter().all(|layer| layer.path.is_finite()));
}

#[test]
fn vector_lowering_preserves_rectangular_clip_metadata() {
    let command = UiSurfaceCommand {
        rect: UiRect::new([0.0, 0.0], [0.4, 0.2]),
        style: UiTheme::default().surface(UiSurfaceRole::Panel),
        clip: Some(UiRect::new([0.0, 0.0], [0.2, 0.2])),
    };

    let layers = lower_surface_to_vector(&command);

    assert!(layers.iter().all(|layer| layer.clip == command.clip));
    assert!(layers.iter().all(|layer| layer.path.bounds().is_some()));
}

#[test]
fn vector_lowering_keeps_border_geometry_uniformly_outside_fill() {
    let mut style = UiTheme::default().surface(UiSurfaceRole::Panel);
    style.border_width = 0.01;
    let command = UiSurfaceCommand {
        rect: UiRect::new([0.0, 0.0], [0.4, 0.2]),
        style,
        clip: None,
    };

    let layers = lower_surface_to_vector(&command);
    let border_bounds = layers[1].path.bounds().unwrap();
    let fill_bounds = layers[2].path.bounds().unwrap();

    assert!((border_bounds.0[0] - fill_bounds.0[0] + 0.01).abs() < 1e-5);
    assert!((border_bounds.1[0] - fill_bounds.1[0] - 0.01).abs() < 1e-5);
    assert!((border_bounds.0[1] - fill_bounds.0[1] + 0.01).abs() < 1e-5);
    assert!((border_bounds.1[1] - fill_bounds.1[1] - 0.01).abs() < 1e-5);
}

#[test]
fn vector_lowering_applies_shadow_offset_before_fill() {
    let command = UiSurfaceCommand {
        rect: UiRect::new([0.0, 0.0], [0.4, 0.2]),
        style: UiTheme::default().surface(UiSurfaceRole::Panel),
        clip: None,
    };

    let layers = lower_surface_to_vector(&command);
    let shadow_bounds = layers[0].path.bounds().unwrap();
    let fill_bounds = layers[2].path.bounds().unwrap();

    assert!((shadow_bounds.0[0] - fill_bounds.0[0] - 0.01).abs() < 1e-5);
    assert!((shadow_bounds.0[1] - fill_bounds.0[1] + 0.01).abs() < 1e-5);
    assert!((shadow_bounds.1[0] - fill_bounds.1[0] - 0.01).abs() < 1e-5);
    assert!((shadow_bounds.1[1] - fill_bounds.1[1] + 0.01).abs() < 1e-5);
}

#[test]
fn vector_lowering_keeps_geometry_stable_when_roles_change() {
    let first_style = UiTheme::default().surface(UiSurfaceRole::Panel);
    let mut second_style = first_style;
    second_style.role = UiSurfaceRole::Accent;
    second_style.border_role = Some(UiSurfaceRole::Selected);

    let first = lower_surface_to_vector(&UiSurfaceCommand {
        rect: UiRect::new([0.0, 0.0], [0.4, 0.2]),
        style: first_style,
        clip: None,
    });
    let second = lower_surface_to_vector(&UiSurfaceCommand {
        rect: UiRect::new([0.0, 0.0], [0.4, 0.2]),
        style: second_style,
        clip: None,
    });

    assert_eq!(
        first.iter().map(|layer| &layer.path).collect::<Vec<_>>(),
        second.iter().map(|layer| &layer.path).collect::<Vec<_>>()
    );
}

#[test]
fn vector_lowering_changes_corner_geometry_without_changing_bounds() {
    let small_style = UiTheme::default().surface(UiSurfaceRole::Panel);
    let mut large_style = small_style;
    large_style.radius = UiRadius::Large;

    let rect = UiRect::new([0.0, 0.0], [0.4, 0.2]);
    let small = lower_surface_to_vector(&UiSurfaceCommand {
        rect,
        style: small_style,
        clip: None,
    });
    let large = lower_surface_to_vector(&UiSurfaceCommand {
        rect,
        style: large_style,
        clip: None,
    });

    let small_bounds = small[2].path.bounds().unwrap();
    let large_bounds = large[2].path.bounds().unwrap();
    for (small_bound, large_bound) in small_bounds.0.into_iter().zip(large_bounds.0) {
        assert!((small_bound - large_bound).abs() < 1e-5);
    }
    for (small_bound, large_bound) in small_bounds.1.into_iter().zip(large_bounds.1) {
        assert!((small_bound - large_bound).abs() < 1e-5);
    }
    assert_ne!(small[2].path, large[2].path);
}

#[test]
fn default_theme_distinguishes_square_from_small_radius() {
    let theme = UiTheme::default();

    assert_eq!(theme.radii.none, UiRadius::None);
    assert_ne!(theme.radii.none, theme.radii.sm);
}

#[test]
fn flat_surface_lowering_has_no_shadow_or_border() {
    let command = UiSurfaceCommand {
        rect: UiRect::new([0.0, 0.0], [0.4, 0.2]),
        style: UiTheme::default().surface(UiSurfaceRole::Background),
        clip: None,
    };

    let layers = lower_surface_to_vector(&command);

    assert_eq!(layers.len(), 1);
    assert_eq!(layers[0].kind, UiSurfaceVectorLayerKind::Fill);
}

#[test]
fn owned_draw_list_preserves_explicit_order_and_source_identity() {
    let theme = UiTheme::default();
    let source = UiNodeId(42);
    let surface = UiSurfaceCommand {
        rect: UiRect::new([0.0, 0.0], [0.4, 0.2]),
        style: theme.surface(UiSurfaceRole::Panel),
        clip: None,
    };
    let text = UiTextCommand::new(
        UiTextSpec::new("panel", surface.rect, UiTextRole::Body),
        theme.text(UiTextRole::Body),
    );

    let mut builder = UiDrawListBuilder::new(7);
    builder.push_clip(Some(source), 0, UiRect::new([0.0, 0.0], [0.8, 0.8]));
    builder.surface(Some(source), 1, surface);
    builder.text(Some(source), 1, text);
    builder.pop_clip(Some(source), 2);
    let list = builder.finish().unwrap();

    assert_eq!(list.revision, 7);
    assert_eq!(list.entries().len(), 4);
    assert_eq!(list.entries()[1].source, Some(source));
    assert_eq!(list.entries()[1].order, 1);
    assert_eq!(list.entries()[2].order, 2);
    assert!(matches!(
        list.entries()[0].command,
        UiDrawCommand::PushClip(_)
    ));
    assert!(matches!(list.entries()[3].command, UiDrawCommand::PopClip));
}

#[test]
fn owned_draw_list_rejects_invalid_clip_and_layer_sequences() {
    let mut underflow = UiDrawListBuilder::new(1);
    underflow.pop_clip(None, 0);
    assert_eq!(
        underflow.finish(),
        Err(UiDrawListError::ClipUnderflow { order: 0 })
    );

    let mut unclosed = UiDrawListBuilder::new(1);
    unclosed.push_clip(None, 0, UiRect::new([0.0, 0.0], [1.0, 1.0]));
    assert_eq!(
        unclosed.finish(),
        Err(UiDrawListError::UnclosedClips { remaining: 1 })
    );

    let mut descending_layer = UiDrawListBuilder::new(1);
    descending_layer.push_clip(None, 2, UiRect::new([0.0, 0.0], [1.0, 1.0]));
    descending_layer.pop_clip(None, 1);
    assert_eq!(
        descending_layer.finish(),
        Err(UiDrawListError::LayerOrder {
            previous: 2,
            next: 1,
        })
    );
}

#[test]
fn legacy_drawer_commands_adapt_to_a_single_ordered_draw_list() {
    let theme = UiTheme::default();
    let mut surfaces = Vec::new();
    let mut text = Vec::new();
    let button = UiButton::new(UiButtonId(7), "edit", UiRect::new([0.0, 0.0], [0.4, 0.1]));
    UiDrawer::new(&mut surfaces, &mut text, &theme).button(
        &button,
        UiInteractionState::Hovered,
        UiControlRole::Primary,
    );

    let list = UiDrawList::from_legacy_commands(9, &surfaces, &text);

    assert_eq!(list.revision, 9);
    assert_eq!(list.entries().len(), 2);
    assert!(matches!(
        list.entries()[0].command,
        UiDrawCommand::Surface(_)
    ));
    assert!(matches!(list.entries()[1].command, UiDrawCommand::Text(_)));
    assert_eq!(list.entries()[0].layer, 0);
    assert_eq!(list.entries()[1].layer, 1);
    assert_eq!(
        list.diagnostics,
        vec![UiDrawListDiagnostic {
            source: None,
            kind: UiDrawListDiagnosticKind::LegacyParallelCommandsAdapted,
        }]
    );
}

#[test]
fn resolved_tree_lowering_uses_one_ordered_geometry_source() {
    let root_id = UiNodeId(1);
    let text = UiTextSpec::new(
        "status ready",
        UiRect::new([99.0, 99.0], [0.0, 0.0]),
        UiTextRole::Status,
    );
    let tree = UiTree::new(
        UiNodeSpec::new(
            root_id,
            UiNodeKind::Region(UiRegionKind::Panel),
            UiSurfaceRole::Panel,
            UiNodeLayout::Fill,
        )
        .clips_children()
        .with_child(
            UiNodeSpec::text(UiNodeId(2), &text)
                .with_parent(root_id)
                .with_layout(UiNodeLayout::Inset(UiInsets::uniform(0.1))),
        ),
    );
    let resolved = tree.resolve(UiRect::new([0.0, 0.0], [1.0, 1.0])).unwrap();

    let list = lower_resolved_tree_to_draw_list(&resolved, &UiTheme::default(), 11);

    assert_eq!(list.revision, 11);
    assert!(list.diagnostics.is_empty());
    assert_eq!(list.entries().len(), 4);
    assert!(matches!(
        list.entries()[0].command,
        UiDrawCommand::Surface(_)
    ));
    assert!(matches!(
        list.entries()[1].command,
        UiDrawCommand::PushClip(_)
    ));
    assert!(matches!(list.entries()[2].command, UiDrawCommand::Text(_)));
    assert!(matches!(list.entries()[3].command, UiDrawCommand::PopClip));
    let UiDrawCommand::Text(text) = &list.entries()[2].command else {
        unreachable!("the third command is asserted as text above");
    };
    assert_eq!(text.spec.rect, resolved.root.children[0].bounds);
}

#[test]
fn resolved_geometry_is_shared_by_lowering_and_pointer_routing() {
    let root_id = UiNodeId(1);
    let control_id = UiNodeId(2);
    let tree = UiTree::new(
        UiNodeSpec::new(
            root_id,
            UiNodeKind::Region(UiRegionKind::Panel),
            UiSurfaceRole::Panel,
            UiNodeLayout::Fill,
        )
        .clips_children()
        .with_child(
            UiNodeSpec::new(
                control_id,
                UiNodeKind::Region(UiRegionKind::Card),
                UiSurfaceRole::Raised,
                UiNodeLayout::Inset(UiInsets::uniform(0.2)),
            )
            .with_parent(root_id)
            .with_interaction(UiNodeInteraction::Activatable),
        ),
    );
    let resolved = tree.resolve(UiRect::new([0.0, 0.0], [1.0, 1.0])).unwrap();
    let list = lower_resolved_tree_to_draw_list(&resolved, &UiTheme::default(), 13);
    let control = &resolved.root.children[0];
    let mut pointer = UiPointerRouter::default();

    let target = pointer.route(
        &resolved,
        UiPointerEvent::new(control.bounds.center, UiPointerPhase::Press),
    );
    let rendered_surface = list
        .entries()
        .iter()
        .find_map(|entry| match (&entry.source, &entry.command) {
            (Some(source), UiDrawCommand::Surface(surface)) if *source == control_id => {
                Some(surface)
            }
            _ => None,
        })
        .expect("the resolved control lowers to one source-provenance surface");

    assert_eq!(target.target, Some(control_id));
    assert_eq!(rendered_surface.rect, control.bounds);
    assert_eq!(rendered_surface.clip, control.clip);
}

#[test]
fn measured_text_overflow_reaches_the_renderer_neutral_draw_diagnostics() {
    #[derive(Clone)]
    struct Metrics;

    impl UiTextMetricsProvider for Metrics {
        fn measure(&self, _text: &str) -> Result<UiTextMeasure, UiTextDiagnostic> {
            Ok(UiTextMeasure {
                advance: 1.0,
                ascent: 0.05,
                descent: 0.0,
                line_gap: 0.0,
                visible_bounds: None,
                diagnostics: Vec::new(),
            })
        }
    }

    let root_id = UiNodeId(1);
    let text = UiTextSpec::new(
        "TOO WIDE",
        UiRect::new([0.0, 0.0], [0.2, 0.1]),
        UiTextRole::Status,
    );
    let tree = UiTree::new(
        UiNodeSpec::new(
            root_id,
            UiNodeKind::Region(UiRegionKind::Workspace),
            UiSurfaceRole::Region,
            UiNodeLayout::Fill,
        )
        .with_child(UiNodeSpec::text(UiNodeId(2), &text).with_parent(root_id)),
    );
    let resolved = tree
        .resolve_with_text_metrics(UiRect::new([0.0, 0.0], [0.2, 0.1]), &Metrics)
        .unwrap();

    let list = lower_resolved_tree_to_draw_list(&resolved, &UiTheme::default(), 14);

    assert!(list.diagnostics.contains(&UiDrawListDiagnostic {
        source: Some(UiNodeId(2)),
        kind: UiDrawListDiagnosticKind::TextOverflow,
    }));
}

#[test]
fn nested_clip_scopes_keep_execution_layers_monotonic() {
    let leaf = UiNodeSpec::new(
        UiNodeId(4),
        UiNodeKind::Region(UiRegionKind::Workspace),
        UiSurfaceRole::Panel,
        UiNodeLayout::Inset(UiInsets::uniform(0.05)),
    )
    .with_parent(UiNodeId(3))
    .clips_children();
    let middle = UiNodeSpec::new(
        UiNodeId(3),
        UiNodeKind::Region(UiRegionKind::Panel),
        UiSurfaceRole::Panel,
        UiNodeLayout::Inset(UiInsets::uniform(0.05)),
    )
    .with_parent(UiNodeId(2))
    .clips_children()
    .with_child(leaf);
    let child = UiNodeSpec::new(
        UiNodeId(2),
        UiNodeKind::Region(UiRegionKind::Panel),
        UiSurfaceRole::Panel,
        UiNodeLayout::Inset(UiInsets::uniform(0.05)),
    )
    .with_parent(UiNodeId(1))
    .clips_children()
    .with_child(middle);
    let tree = UiTree::new(
        UiNodeSpec::new(
            UiNodeId(1),
            UiNodeKind::Region(UiRegionKind::Workspace),
            UiSurfaceRole::Region,
            UiNodeLayout::Fill,
        )
        .clips_children()
        .with_child(child),
    );
    let resolved = tree.resolve(UiRect::new([0.0, 0.0], [1.0, 1.0])).unwrap();

    let list = lower_resolved_tree_to_draw_list(&resolved, &UiTheme::default(), 12);

    assert!(list
        .entries()
        .windows(2)
        .all(|entries| entries[0].layer < entries[1].layer));
    assert_eq!(
        list.entries()
            .iter()
            .filter(|entry| matches!(entry.command, UiDrawCommand::PushClip(_)))
            .count(),
        4
    );
    assert_eq!(
        list.entries()
            .iter()
            .filter(|entry| matches!(entry.command, UiDrawCommand::PopClip))
            .count(),
        4
    );
}

#[test]
fn equivalent_resolved_trees_have_the_same_draw_list_fingerprint() {
    let root = UiNodeSpec::new(
        UiNodeId(1),
        UiNodeKind::Region(UiRegionKind::Panel),
        UiSurfaceRole::Panel,
        UiNodeLayout::Fill,
    )
    .with_child(
        UiNodeSpec::text(
            UiNodeId(2),
            &UiTextSpec::new(
                "consistent evidence",
                UiRect::new([0.0, 0.0], [0.0, 0.0]),
                UiTextRole::Body,
            ),
        )
        .with_parent(UiNodeId(1))
        .with_layout(UiNodeLayout::Inset(UiInsets::uniform(0.1))),
    );
    let bounds = UiRect::new([0.0, 0.0], [1.0, 1.0]);
    let first = UiTree::new(root.clone()).resolve(bounds).unwrap();
    let second = UiTree::new(root).resolve(bounds).unwrap();
    let theme = UiTheme::default();

    let first_list = lower_resolved_tree_to_draw_list(&first, &theme, 1);
    let second_list = lower_resolved_tree_to_draw_list(&second, &theme, 2);

    assert_ne!(first_list.revision, second_list.revision);
    assert_eq!(
        first_list.structural_fingerprint(),
        second_list.structural_fingerprint()
    );
}

#[test]
fn draw_cache_key_excludes_revision_and_semantic_provenance() {
    let theme = UiTheme::default();
    let surface = UiSurfaceCommand {
        rect: UiRect::new([0.0, 0.0], [0.4, 0.2]),
        style: theme.surface(UiSurfaceRole::Panel),
        clip: None,
    };
    let mut first = UiDrawListBuilder::new(1);
    first.surface(Some(UiNodeId(10)), 0, surface);
    let first = first.finish().unwrap();

    let mut second = UiDrawListBuilder::new(99);
    second.surface(Some(UiNodeId(20)), 0, surface);
    let second = second.finish().unwrap();

    assert_ne!(
        first.structural_fingerprint(),
        second.structural_fingerprint()
    );
    assert_eq!(first.cache_key(), second.cache_key());
    assert_eq!(first.cache_key().value(), second.cache_key().value());
}

#[test]
fn draw_statistics_report_conservative_contiguous_batch_candidates() {
    let theme = UiTheme::default();
    let style = theme.surface(UiSurfaceRole::Panel);
    let mut builder = UiDrawListBuilder::new(1);
    for x in [-0.4, 0.0, 0.4] {
        builder.surface(
            None,
            0,
            UiSurfaceCommand {
                rect: UiRect::new([x, 0.0], [0.2, 0.2]),
                style,
                clip: None,
            },
        );
    }
    builder.text(
        None,
        1,
        UiTextCommand::new(
            UiTextSpec::new(
                "first",
                UiRect::new([-0.2, -0.3], [0.3, 0.1]),
                UiTextRole::Body,
            ),
            theme.text(UiTextRole::Body),
        ),
    );
    builder.text(
        None,
        1,
        UiTextCommand::new(
            UiTextSpec::new(
                "second",
                UiRect::new([0.2, -0.3], [0.3, 0.1]),
                UiTextRole::Body,
            ),
            theme.text(UiTextRole::Body),
        ),
    );
    let statistics = builder.finish().unwrap().statistics();

    assert_eq!(statistics.entries, 5);
    assert_eq!(statistics.surfaces, 3);
    assert_eq!(statistics.text, 2);
    assert_eq!(statistics.surface_batch_candidates, 1);
    assert_eq!(statistics.text_batch_candidates, 1);
}

#[test]
fn clip_boundaries_split_draw_batch_candidates() {
    let theme = UiTheme::default();
    let surface = UiSurfaceCommand {
        rect: UiRect::new([0.0, 0.0], [0.2, 0.2]),
        style: theme.surface(UiSurfaceRole::Panel),
        clip: None,
    };
    let mut builder = UiDrawListBuilder::new(1);
    builder.surface(None, 0, surface);
    builder.push_clip(None, 1, UiRect::new([0.0, 0.0], [0.5, 0.5]));
    builder.surface(None, 1, surface);
    builder.pop_clip(None, 2);
    builder.surface(None, 2, surface);
    let statistics = builder.finish().unwrap().statistics();

    assert_eq!(statistics.clip_pushes, 1);
    assert_eq!(statistics.clip_pops, 1);
    assert_eq!(statistics.surface_batch_candidates, 3);
}
