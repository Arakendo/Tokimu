use ui_tools::consumer::{
    UiButtonId, UiControlRole, UiFrameLayout, UiInsets, UiInteractionState, UiLayoutFit,
    UiModalDismissReason, UiNodeConstraints, UiNodeId, UiNodeInteraction, UiNodeKind, UiNodeLayout,
    UiNodeSpec, UiPresentationInputs, UiPresentationRevisionTracker, UiPresentationWorkEvidence,
    UiRect, UiRegionKind, UiResolvedFocus, UiSemanticRole, UiSurfaceRole, UiTextFit,
    UiTextInputEvent, UiTextInputOperation, UiTextInputRouter, UiTextRole, UiTextSpec, UiTheme,
    UiThemeProfile, UiTree, UiVerticalScroll,
};
use ui_tools::lowering::{
    lower_resolved_tree_to_draw_list, UiDrawCommand, UiDrawListBuilder, UiSurfaceCommand,
};
use ui_tools::provider::UiTextMetricsProvider;
use ui_tools::{UiTextDiagnostic, UiTextMeasure};

#[test]
fn ordinary_consumer_tier_supports_semantic_layout_and_text_intent() {
    let frame = UiFrameLayout::new(
        UiRect::new([0.0, 0.0], [1.6, 1.0]),
        UiInsets::uniform(0.05),
        0.12,
        0.10,
        0.02,
    );
    let label = UiTextSpec::new("runtime observation", frame.header, UiTextRole::Heading);

    assert!(frame.body.size[1] > 0.0);
    assert_eq!(label.rect, frame.header);
}

#[test]
fn ordinary_consumer_can_select_a_complete_high_contrast_theme() {
    let theme = UiTheme::high_contrast();
    assert_eq!(theme.profile, UiThemeProfile::HighContrast);
    assert!(theme.diagnostics().is_empty());

    for role in UiControlRole::ALL {
        for state in UiInteractionState::ALL {
            let style = theme.control(role, state);
            assert_eq!(style.control_role, Some(role));
            assert_eq!(style.interaction_state, Some(state));
        }
    }
}

#[test]
fn ordinary_consumer_can_observe_bounded_rebuild_evidence() {
    let mut tracker = UiPresentationRevisionTracker::default();
    let inputs = UiPresentationInputs::default();
    assert_eq!(tracker.observe(inputs).draw_list_rebuilds, 1);

    let stable = tracker.observe(inputs);
    assert!(stable.invalidation.none());

    let changed = tracker.observe(UiPresentationInputs {
        layout: 1,
        ..inputs
    });
    assert_eq!(changed.semantic_rebuilds, 0);
    assert_eq!(changed.measurement_rebuilds, 0);
    assert_eq!(changed.layout_rebuilds, 1);
    assert_eq!(changed.geometry_rebuilds, 1);
    assert_eq!(changed.draw_list_rebuilds, 1);

    let interaction = tracker.observe(
        UiPresentationInputs {
            layout: 1,
            ..inputs
        }
        .with_interaction_revision(1),
    );
    assert_eq!(interaction.semantic_rebuilds, 0);
    assert_eq!(interaction.measurement_rebuilds, 0);
    assert_eq!(interaction.layout_rebuilds, 0);
    assert_eq!(interaction.geometry_rebuilds, 0);
    assert_eq!(interaction.draw_list_rebuilds, 1);
}

#[test]
fn renderer_adapter_can_observe_stable_work_identity_and_batch_evidence() {
    let theme = UiTheme::default();
    let style = theme.surface(UiSurfaceRole::Panel);
    let mut first = UiDrawListBuilder::new(1);
    let mut second = UiDrawListBuilder::new(2);

    for (index, x) in [-0.3, 0.3].into_iter().enumerate() {
        let command = UiSurfaceCommand {
            rect: UiRect::new([x, 0.0], [0.2, 0.2]),
            style,
            clip: None,
        };
        first.surface(Some(UiNodeId(index as u64 + 10)), 0, command);
        second.surface(Some(UiNodeId(index as u64 + 20)), 0, command);
    }

    let first = first.finish().unwrap();
    let second = second.finish().unwrap();
    assert_eq!(first.cache_key(), second.cache_key());
    assert_eq!(first.statistics().surfaces, 2);
    assert_eq!(first.statistics().surface_batch_candidates, 1);
}

#[test]
fn application_can_capture_stage_timing_and_renderer_counts_without_gpu_ownership() {
    let evidence = UiPresentationWorkEvidence::default()
        .with_measurement_time(std::time::Duration::from_micros(20))
        .with_layout_time(std::time::Duration::from_micros(40))
        .with_renderer_counts(1, 2, 12);

    assert_eq!(evidence.measurement_micros, 20);
    assert_eq!(evidence.layout_micros, 40);
    assert_eq!(evidence.uploads, 1);
    assert_eq!(evidence.submits, 2);
    assert_eq!(evidence.draws, 12);
}

#[test]
fn ordinary_consumer_tier_resolves_a_headless_semantic_tree() {
    let root_id = UiNodeId(1);
    let child = UiNodeSpec::text(
        UiNodeId(2),
        &UiTextSpec::new(
            "headless text intent",
            UiRect::new([0.0, 0.0], [0.0, 0.0]),
            UiTextRole::Body,
        ),
    )
    .with_parent(root_id)
    .with_layout(UiNodeLayout::Inset(UiInsets::uniform(0.1)));
    let tree = UiTree::new(
        UiNodeSpec::new(
            root_id,
            UiNodeKind::Region(UiRegionKind::Workspace),
            UiSurfaceRole::Region,
            UiNodeLayout::Fill,
        )
        .with_child(child),
    );

    let resolved = tree.resolve(UiRect::new([0.0, 0.0], [2.0, 1.0])).unwrap();
    let size = resolved.root.children[0].bounds.size;
    assert!((size[0] - 1.8).abs() < 0.00001);
    assert!((size[1] - 0.8).abs() < 0.00001);
    assert_eq!(
        resolved.root.children[0].text.as_ref().unwrap().rect.size,
        size
    );
    assert!(resolved.diagnostics.is_empty());
}

#[test]
fn ordinary_consumer_can_anchor_domain_content_to_resolved_node_geometry() {
    let root_id = UiNodeId(1);
    let viewport_id = UiNodeId(2);
    let tree = UiTree::new(
        UiNodeSpec::new(
            root_id,
            UiNodeKind::Region(UiRegionKind::Panel),
            UiSurfaceRole::Panel,
            UiNodeLayout::Fill,
        )
        .with_child(
            UiNodeSpec::new(
                viewport_id,
                UiNodeKind::Region(UiRegionKind::Workspace),
                UiSurfaceRole::Background,
                UiNodeLayout::Inset(UiInsets::uniform(0.1)),
            )
            .with_parent(root_id)
            .with_semantic_label("domain viewport"),
        ),
    );

    let resolved = tree.resolve(UiRect::new([0.0, 0.0], [2.0, 1.0])).unwrap();
    let viewport = resolved.node(viewport_id).unwrap();

    assert_eq!(viewport.semantic_label.as_deref(), Some("domain viewport"));
    assert_eq!(viewport.bounds.center, [0.0, 0.0]);
    assert!((viewport.bounds.size[0] - 1.8).abs() < 1.0e-6);
    assert!((viewport.bounds.size[1] - 0.8).abs() < 1.0e-6);
    assert_eq!(resolved.node(UiNodeId(99)), None);
}

#[test]
fn ordinary_consumer_tier_reports_unfit_controls_and_uses_resolved_hit_bounds() {
    let root_id = UiNodeId(1);
    let button_id = UiNodeId(2);
    let button = UiNodeSpec::new(
        button_id,
        UiNodeKind::Button(UiButtonId(7)),
        UiSurfaceRole::Raised,
        UiNodeLayout::Fill,
    )
    .with_parent(root_id)
    .with_constraints(UiNodeConstraints::minimum([1.5, 1.0]));
    let tree = UiTree::new(
        UiNodeSpec::new(
            root_id,
            UiNodeKind::Region(UiRegionKind::Workspace),
            UiSurfaceRole::Region,
            UiNodeLayout::Fill,
        )
        .clips_children()
        .with_child(button),
    );

    let resolved = tree.resolve(UiRect::new([0.0, 0.0], [1.0, 0.5])).unwrap();
    assert_eq!(resolved.root.children[0].layout_fit, UiLayoutFit::Overflow);
    assert_eq!(
        resolved.hit_test([0.0, 0.0]).map(|node| node.id),
        Some(button_id)
    );
    assert_eq!(resolved.hit_test([1.0, 0.0]), None);
}

#[test]
fn resolved_consumer_tree_lowers_through_one_ordered_public_artifact() {
    let root_id = UiNodeId(1);
    let text_id = UiNodeId(2);
    let tree = UiTree::new(
        UiNodeSpec::new(
            root_id,
            UiNodeKind::Region(UiRegionKind::Panel),
            UiSurfaceRole::Panel,
            UiNodeLayout::Fill,
        )
        .with_child(
            UiNodeSpec::text(
                text_id,
                &UiTextSpec::new(
                    "public draw list",
                    UiRect::new([0.0, 0.0], [0.0, 0.0]),
                    UiTextRole::Body,
                ),
            )
            .with_parent(root_id)
            .with_layout(UiNodeLayout::Fill),
        ),
    );
    let resolved = tree.resolve(UiRect::new([0.0, 0.0], [1.0, 1.0])).unwrap();

    let draw_list = lower_resolved_tree_to_draw_list(&resolved, &UiTheme::default(), 3);

    assert_eq!(draw_list.revision, 3);
    assert_eq!(draw_list.entries().len(), 2);
    assert!(matches!(
        draw_list.entries()[0].command,
        UiDrawCommand::Surface(_)
    ));
    assert!(matches!(
        draw_list.entries()[1].command,
        UiDrawCommand::Text(_)
    ));
    assert_eq!(draw_list.entries()[1].source, Some(text_id));
}

#[test]
fn provider_metrics_enrich_consumer_tree_resolution_without_selecting_a_font() {
    struct Metrics;

    impl UiTextMetricsProvider for Metrics {
        fn measure(&self, _text: &str) -> Result<UiTextMeasure, UiTextDiagnostic> {
            Ok(UiTextMeasure {
                advance: 0.5,
                ascent: 0.04,
                descent: 0.01,
                line_gap: 0.0,
                visible_bounds: None,
                diagnostics: Vec::new(),
            })
        }
    }

    let root_id = UiNodeId(1);
    let tree = UiTree::new(
        UiNodeSpec::new(
            root_id,
            UiNodeKind::Region(UiRegionKind::Panel),
            UiSurfaceRole::Panel,
            UiNodeLayout::Fill,
        )
        .with_child(
            UiNodeSpec::text(
                UiNodeId(2),
                &UiTextSpec::new(
                    "metrics",
                    UiRect::new([0.0, 0.0], [0.0, 0.0]),
                    UiTextRole::Status,
                ),
            )
            .with_parent(root_id)
            .with_layout(UiNodeLayout::Fill),
        ),
    );

    let resolved = tree
        .resolve_with_text_metrics(UiRect::new([0.0, 0.0], [0.2, 0.1]), &Metrics)
        .unwrap();

    assert_eq!(
        resolved.root.children[0].text_fit,
        Some(UiTextFit {
            horizontal_overflow: true,
            vertical_overflow: false,
        })
    );
}

#[test]
fn ordinary_consumer_tier_routes_normalized_edits_without_owning_text_state() {
    let root_id = UiNodeId(1);
    let field_id = UiNodeId(2);
    let tree = UiTree::new(
        UiNodeSpec::new(
            root_id,
            UiNodeKind::Region(UiRegionKind::Panel),
            UiSurfaceRole::Panel,
            UiNodeLayout::Fill,
        )
        .with_child(
            UiNodeSpec::new(
                field_id,
                UiNodeKind::TextInput,
                UiSurfaceRole::Raised,
                UiNodeLayout::Fill,
            )
            .with_parent(root_id)
            .with_interaction(UiNodeInteraction::Editable),
        ),
    );
    let resolved = tree.resolve(UiRect::new([0.0, 0.0], [1.0, 0.4])).unwrap();
    let mut focus = UiResolvedFocus::default();

    focus.set_focus(&resolved, Some(field_id));
    let resolution = UiTextInputRouter.route(
        &resolved,
        &mut focus,
        UiTextInputEvent::new(UiTextInputOperation::Insert(' ')),
    );

    assert_eq!(resolution.target, Some(field_id));
    assert_eq!(resolution.operation, UiTextInputOperation::Insert(' '));
}

#[test]
fn resolved_semantics_preserve_application_meaning_and_focus() {
    let root_id = UiNodeId(1);
    let button_id = UiNodeId(2);
    let tree = UiTree::new(
        UiNodeSpec::new(
            root_id,
            UiNodeKind::Region(UiRegionKind::Panel),
            UiSurfaceRole::Panel,
            UiNodeLayout::Fill,
        )
        .with_semantic_label("asset actions")
        .with_child(
            UiNodeSpec::new(
                button_id,
                UiNodeKind::Button(UiButtonId(12)),
                UiSurfaceRole::Raised,
                UiNodeLayout::Fill,
            )
            .with_parent(root_id)
            .with_semantic_label("Save asset")
            .with_semantic_value("ready")
            .with_selected(true),
        ),
    );
    let resolved = tree.resolve(UiRect::new([0.0, 0.0], [1.0, 0.4])).unwrap();
    let mut focus = UiResolvedFocus::default();
    focus.set_focus(&resolved, Some(button_id));

    let semantics = resolved.semantic_nodes(&focus);

    assert_eq!(semantics.len(), 2);
    assert_eq!(semantics[0].role, UiSemanticRole::Region);
    assert_eq!(semantics[0].label.as_deref(), Some("asset actions"));
    assert_eq!(semantics[1].role, UiSemanticRole::Button);
    assert_eq!(semantics[1].label.as_deref(), Some("Save asset"));
    assert_eq!(semantics[1].value.as_deref(), Some("ready"));
    assert!(semantics[1].selected);
    assert!(semantics[1].focusable);
    assert!(semantics[1].focused);

    let draw_list = lower_resolved_tree_to_draw_list(&resolved, &UiTheme::default(), 5);
    assert!(draw_list
        .entries()
        .iter()
        .any(|entry| entry.source == Some(button_id)));
}

#[test]
fn scroll_resolution_keeps_draw_clip_and_hit_testing_on_one_geometry() {
    let viewport = UiRect::new([0.0, 0.0], [1.0, 1.0]);
    let mut scroll = UiVerticalScroll::new(viewport, 3.0);
    scroll.set_offset(1.0);
    let root_id = UiNodeId(1);
    let child_id = UiNodeId(2);
    let authored_child_bounds = UiRect::new([0.0, -1.3], [0.8, 0.8]);
    let tree = UiTree::new(
        UiNodeSpec::new(
            root_id,
            UiNodeKind::Region(UiRegionKind::Workspace),
            UiSurfaceRole::Region,
            UiNodeLayout::Fill,
        )
        .clips_children()
        .with_child_translation(scroll.content_translation())
        .with_child(
            UiNodeSpec::new(
                child_id,
                UiNodeKind::Button(UiButtonId(9)),
                UiSurfaceRole::Raised,
                UiNodeLayout::Explicit(authored_child_bounds),
            )
            .with_parent(root_id),
        ),
    );

    let resolved = tree.resolve(viewport).unwrap();
    let child = &resolved.root.children[0];
    let expected_bounds = scroll.content_rect(authored_child_bounds);
    let expected_clip = expected_bounds
        .intersection(viewport)
        .expect("translated child remains partially visible");

    assert_eq!(child.bounds, expected_bounds);
    assert_eq!(child.clip, Some(expected_clip));
    assert_eq!(
        resolved.hit_test([0.0, -0.4]).map(|node| node.id),
        Some(child_id)
    );
    assert_eq!(resolved.hit_test([0.0, -0.6]), None);

    let draw_list = lower_resolved_tree_to_draw_list(&resolved, &UiTheme::default(), 4);
    let child_surface = draw_list
        .entries()
        .iter()
        .find_map(|entry| match (&entry.source, &entry.command) {
            (Some(source), UiDrawCommand::Surface(surface)) if *source == child_id => Some(surface),
            _ => None,
        })
        .expect("translated child emits one sourced surface");

    assert_eq!(child_surface.rect, expected_bounds);
    assert_eq!(child_surface.clip, Some(expected_clip));
}

#[test]
fn modal_scope_excludes_translated_scroll_content_from_input_and_focus() {
    let viewport = UiRect::new([0.0, 0.0], [2.0, 2.0]);
    let scroll_bounds = UiRect::new([-0.5, 0.0], [0.8, 1.6]);
    let mut scroll = UiVerticalScroll::new(scroll_bounds, 3.0);
    scroll.set_offset(0.4);

    let root_id = UiNodeId(1);
    let scroll_id = UiNodeId(2);
    let background_button_id = UiNodeId(3);
    let modal_id = UiNodeId(4);
    let modal_button_id = UiNodeId(5);
    let tree = UiTree::new(
        UiNodeSpec::new(
            root_id,
            UiNodeKind::Region(UiRegionKind::Workspace),
            UiSurfaceRole::Region,
            UiNodeLayout::Fill,
        )
        .with_child(
            UiNodeSpec::new(
                scroll_id,
                UiNodeKind::Region(UiRegionKind::Panel),
                UiSurfaceRole::Panel,
                UiNodeLayout::Explicit(scroll_bounds),
            )
            .with_parent(root_id)
            .clips_children()
            .with_child_translation(scroll.content_translation())
            .with_child(
                UiNodeSpec::new(
                    background_button_id,
                    UiNodeKind::Button(UiButtonId(10)),
                    UiSurfaceRole::Raised,
                    UiNodeLayout::Explicit(UiRect::new([-0.5, -0.4], [0.5, 0.3])),
                )
                .with_parent(scroll_id),
            ),
        )
        .with_child(
            UiNodeSpec::new(
                modal_id,
                UiNodeKind::Region(UiRegionKind::Panel),
                UiSurfaceRole::Overlay,
                UiNodeLayout::Explicit(UiRect::new([0.45, 0.0], [0.7, 0.8])),
            )
            .with_parent(root_id)
            .as_modal(true)
            .with_child(
                UiNodeSpec::new(
                    modal_button_id,
                    UiNodeKind::Button(UiButtonId(11)),
                    UiSurfaceRole::Raised,
                    UiNodeLayout::Fill,
                )
                .with_parent(modal_id),
            ),
        ),
    );

    let resolved = tree.resolve(viewport).unwrap();

    assert_eq!(resolved.active_modal().map(|node| node.id), Some(modal_id));
    assert_eq!(resolved.hit_test([-0.5, 0.0]), None);
    assert_eq!(
        resolved.hit_test([0.45, 0.0]).map(|node| node.id),
        Some(modal_button_id)
    );
    assert_eq!(resolved.interactive_node_ids(), vec![modal_button_id]);
    let mut focus = UiResolvedFocus::default();
    focus.set_focus(&resolved, Some(modal_button_id));
    assert_eq!(
        resolved
            .semantic_nodes(&focus)
            .iter()
            .map(|node| node.id)
            .collect::<Vec<_>>(),
        vec![modal_id, modal_button_id]
    );
    assert_eq!(
        resolved
            .modal_dismissal(UiModalDismissReason::Backdrop)
            .map(|dismissal| dismissal.modal),
        Some(modal_id)
    );
}
