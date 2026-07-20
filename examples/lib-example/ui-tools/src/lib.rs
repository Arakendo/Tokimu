mod controls;
mod draw;
mod geometry;
mod layout;
mod presets;
mod region;
mod raster;
mod scroll;
mod svg;
mod text;
mod text_input;
mod theme;

pub use controls::{
    UiActionId, UiActivationKey, UiButton, UiButtonId, UiButtonSpec, UiCardSpec, UiDiagnostic,
    UiDiagnosticKind, UiDiagnosticSeverity, UiEvent, UiFocusDirection, UiFocusState,
    UiInteractionState,
    UiLabel, UiLabelAnchor, UiLabelSpec, UiStateChip,
};
pub use draw::{UiDrawer, UiSurfaceCommand, UiTextCommand};
pub use geometry::{window_to_world, UiHitRegion, UiInsets, UiRect};
pub use layout::{
    UiConstraints, UiCrossAxisAlignment, UiHorizontalStack, UiLayoutResult, UiMainAxisAllocation,
    UiMeasurable, UiMeasureContext, UiSizePolicy, UiVerticalStack,
};
pub use presets::{UiToolbarLayout, UiWorkspaceLayout};
pub use region::UiCardRole;
pub use region::{
    UiCard, UiInspector, UiPanel, UiRadius, UiRegion, UiRegionKind, UiSidebar, UiSpacing,
    UiStatusBar, UiSurfaceRole, UiTabStrip, UiToolbar, UiWorkspace,
};
pub use scroll::UiVerticalScroll;
pub use raster::{UiFontRasterizer, UiRasterGlyph, UiRasterText, UiRasterTextBitmap, UiRasterTextGlyph};
pub use svg::{flatten_path, parse_path, stroke_paths, tokenize_path, SvgPathCommand, SvgToken};
pub use text::{
    bitmap_glyph_height, layout_bitmap_text, measure_bitmap_text_width, UiGlyphQuad, UiTextAlign,
    UiTextDirection, UiTextOverflow, UiTextRole, UiTextSpec,
};
pub use text_input::{UiTextInputOperation, UiTextInputState};
pub use theme::{
    UiBorderScale, UiControlRole, UiElevation, UiRadiusScale, UiSpacingScale, UiSurfaceStyle,
    UiTextStyle, UiTheme,
};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn toolbar_layout_can_hit_test_buttons() {
        let layout = UiToolbarLayout::new(
            [1280.0, 720.0],
            [
                UiButtonSpec::new(UiButtonId(0), "browse"),
                UiButtonSpec::new(UiButtonId(1), "edit"),
                UiButtonSpec::new(UiButtonId(2), "preview"),
            ],
            [
                UiCardSpec::new(UiCardRole::Browser, "browse", "shell"),
                UiCardSpec::new(UiCardRole::Editor, "edit", "select"),
                UiCardSpec::new(UiCardRole::Preview, "preview", "hover"),
            ],
        );
        let point = layout.buttons[1].rect.center;

        assert_eq!(layout.button_at(point), Some(UiButtonId(1)));
        assert_eq!(layout.title_chip.label, "WORKSPACE");
        assert_eq!(layout.workspace.kind, UiRegionKind::Workspace);
        assert_eq!(layout.sidebar.kind, UiRegionKind::Sidebar);
        assert!(layout
            .footer_chip
            .contains(layout.footer_chip.region().rect.center));
        assert_eq!(layout.cards[0].title, "browse");
    }

    #[test]
    fn label_and_card_metadata_are_usable() {
        let label = UiLabelSpec::new("hello", UiLabelAnchor::Start);
        let card = UiCardSpec::new(UiCardRole::Editor, "title", "body");
        let region = UiRegion::new(
            UiRegionKind::Card,
            UiSurfaceRole::Panel,
            UiRect::new([0.0, 0.0], [1.0, 1.0]),
        );
        let structured_card = UiCard::new(UiCardRole::Editor, "title", "body", region);

        assert_eq!(label.text, "hello");
        assert_eq!(card.body, "body");
        assert_eq!(structured_card.region.kind, UiRegionKind::Card);
        assert_eq!(structured_card.header.role, UiSurfaceRole::Raised);
    }

    #[test]
    fn intrinsic_card_width_follows_content_measurement() {
        let theme = UiTheme::default();
        let short = UiCard::from_intrinsic(
            UiCardRole::Editor,
            "Title",
            "Body",
            [0.0, 0.0],
            &theme,
        );
        let long = UiCard::from_intrinsic(
            UiCardRole::Editor,
            "Title",
            "A much longer body",
            [0.0, 0.0],
            &theme,
        );

        assert!(long.region.rect.size[0] > short.region.rect.size[0]);
        assert_eq!(long.region.rect.size[1], short.region.rect.size[1]);
    }

    #[test]
    fn measurement_applies_parent_constraints() {
        let theme = UiTheme::default();
        let button = UiButton::from_intrinsic(UiButtonId(0), "A LONG LABEL", [0.0, 0.0], &theme);
        let context = UiMeasureContext::new(&theme, [0.12, 0.08]).with_constraints(
            UiConstraints::new([0.06, 0.04], [0.12, 0.08]),
        );

        assert_eq!(button.measure(&context), [0.12, 0.08]);
    }

    #[test]
    fn malformed_constraints_are_normalized() {
        let constraints = UiConstraints::new([0.8, 0.6], [0.2, 0.1]);

        assert_eq!(constraints.min, [0.2, 0.1]);
        assert_eq!(constraints.max, [0.8, 0.6]);
        assert_eq!(constraints.constrain([0.0, 1.0]), [0.2, 0.6]);
    }

    #[test]
    fn size_policy_resolves_intrinsic_fill_fixed_min_and_max() {
        let constraints = UiConstraints::new([0.2, 0.1], [0.8, 0.6]);
        let intrinsic = [0.4, 0.3];

        assert_eq!(
            UiSizePolicy::Intrinsic.resolve(intrinsic, constraints),
            intrinsic
        );
        assert_eq!(
            UiSizePolicy::Fill.resolve(intrinsic, constraints),
            [0.8, 0.6]
        );
        assert_eq!(
            UiSizePolicy::Fixed([0.5, 0.2]).resolve(intrinsic, constraints),
            [0.5, 0.2]
        );
        assert_eq!(
            UiSizePolicy::Min([0.6, 0.5]).resolve(intrinsic, constraints),
            [0.6, 0.5]
        );
        assert_eq!(
            UiSizePolicy::Max([0.3, 0.2]).resolve(intrinsic, constraints),
            [0.3, 0.2]
        );
    }

    #[test]
    fn horizontal_stack_produces_ordered_nested_layout() {
        let theme = UiTheme::default();
        let first = UiButton::from_intrinsic(UiButtonId(0), "browse", [0.0, 0.0], &theme);
        let second = UiButton::from_intrinsic(UiButtonId(1), "edit", [0.0, 0.0], &theme);
        let stack = UiHorizontalStack::new(vec![first, second], 0.02);
        let context = UiMeasureContext::new(&theme, [0.8, 0.3]);
        let parent = UiRect::new([0.0, 0.0], [0.8, 0.3]);

        let measured = stack.measure(&context);
        let result = stack.layout(parent, &context);

        assert_eq!(result.rect, parent);
        assert_eq!(result.children.len(), 2);
        assert!(measured[0] > first.intrinsic_size(&theme)[0]);
        assert!(result.children[0].rect.center[0] < result.children[1].rect.center[0]);
        assert!(result
            .children
            .iter()
            .all(|child| child.rect.size[0] <= parent.size[0]));
    }

    #[test]
    fn horizontal_stack_clamps_oversized_gaps_to_parent_width() {
        let theme = UiTheme::default();
        let children = vec![
            UiButton::from_intrinsic(UiButtonId(0), "A", [0.0, 0.0], &theme),
            UiButton::from_intrinsic(UiButtonId(1), "B", [0.0, 0.0], &theme),
        ];
        let stack = UiHorizontalStack::new(children, 1.0);
        let context = UiMeasureContext::new(&theme, [0.1, 0.2]);
        let parent = UiRect::new([0.0, 0.0], [0.1, 0.2]);

        let result = stack.layout(parent, &context);
        let left = result.children[0].rect.center[0] - result.children[0].rect.size[0] * 0.5;
        let right = result.children[1].rect.center[0] + result.children[1].rect.size[0] * 0.5;

        assert!(left >= parent.center[0] - parent.size[0] * 0.5);
        assert!(right <= parent.center[0] + parent.size[0] * 0.5);
    }

    #[test]
    fn horizontal_stack_accepts_cards_as_measurable_children() {
        let theme = UiTheme::default();
        let cards = vec![
            UiCard::from_intrinsic(UiCardRole::Browser, "Files", "One", [0.0, 0.0], &theme),
            UiCard::from_intrinsic(UiCardRole::Preview, "Preview", "Two", [0.0, 0.0], &theme),
        ];
        let stack = UiHorizontalStack::new(cards, 0.02);
        let context = UiMeasureContext::new(&theme, [1.0, 0.5]);

        let result = stack.layout(UiRect::new([0.0, 0.0], [1.0, 0.5]), &context);

        assert_eq!(result.children.len(), 2);
        assert!(result.children[0].rect.size[0] > 0.0);
        assert!(result.children[1].rect.size[0] > 0.0);
    }

    #[test]
    fn vertical_stack_produces_top_to_bottom_nested_layout() {
        let theme = UiTheme::default();
        let children = vec![
            UiButton::from_intrinsic(UiButtonId(0), "TOP", [0.0, 0.0], &theme),
            UiButton::from_intrinsic(UiButtonId(1), "BOTTOM", [0.0, 0.0], &theme),
        ];
        let stack = UiVerticalStack::new(children, 0.02);
        let context = UiMeasureContext::new(&theme, [0.5, 0.6]);
        let parent = UiRect::new([0.0, 0.0], [0.5, 0.6]);

        let result = stack.layout(parent, &context);

        assert_eq!(result.rect, parent);
        assert_eq!(result.children.len(), 2);
        assert!(result.children[0].rect.center[1] > result.children[1].rect.center[1]);
        assert!(result
            .children
            .iter()
            .all(|child| child.rect.center[0] == parent.center[0]));
    }

    #[test]
    fn vertical_stack_clamps_oversized_gaps_to_parent_height() {
        let theme = UiTheme::default();
        let children = vec![
            UiButton::from_intrinsic(UiButtonId(0), "A", [0.0, 0.0], &theme),
            UiButton::from_intrinsic(UiButtonId(1), "B", [0.0, 0.0], &theme),
        ];
        let stack = UiVerticalStack::new(children, 1.0);
        let context = UiMeasureContext::new(&theme, [0.2, 0.1]);
        let parent = UiRect::new([0.0, 0.0], [0.2, 0.1]);

        let result = stack.layout(parent, &context);
        let top = result.children[0].rect.center[1] + result.children[0].rect.size[1] * 0.5;
        let bottom = result.children[1].rect.center[1] - result.children[1].rect.size[1] * 0.5;

        assert!(top <= parent.center[1] + parent.size[1] * 0.5);
        assert!(bottom >= parent.center[1] - parent.size[1] * 0.5);
    }

    #[test]
    fn stacks_apply_cross_axis_start_end_and_fill_alignment() {
        let theme = UiTheme::default();
        let button = UiButton::from_intrinsic(UiButtonId(0), "A", [0.0, 0.0], &theme);
        let context = UiMeasureContext::new(&theme, [0.6, 0.6]);
        let parent = UiRect::new([0.0, 0.0], [0.6, 0.6]);

        let horizontal_start = UiHorizontalStack::new(vec![button], 0.0)
            .with_cross_axis_alignment(UiCrossAxisAlignment::Start)
            .layout(parent, &context)
            .children[0]
            .rect;
        let horizontal_fill = UiHorizontalStack::new(vec![button], 0.0)
            .with_cross_axis_alignment(UiCrossAxisAlignment::Fill)
            .layout(parent, &context)
            .children[0]
            .rect;
        let vertical_end = UiVerticalStack::new(vec![button], 0.0)
            .with_cross_axis_alignment(UiCrossAxisAlignment::End)
            .layout(parent, &context)
            .children[0]
            .rect;

        assert!(horizontal_start.center[1] > 0.0);
        assert_eq!(horizontal_fill.size[1], parent.size[1]);
        assert!(vertical_end.center[0] > 0.0);
    }

    #[test]
    fn stacks_allocate_remaining_main_axis_space_in_fill_mode() {
        let theme = UiTheme::default();
        let buttons = vec![
            UiButton::from_intrinsic(UiButtonId(0), "A", [0.0, 0.0], &theme),
            UiButton::from_intrinsic(UiButtonId(1), "B", [0.0, 0.0], &theme),
        ];
        let context = UiMeasureContext::new(&theme, [0.8, 0.8]);
        let horizontal_parent = UiRect::new([0.0, 0.0], [0.8, 0.3]);
        let vertical_parent = UiRect::new([0.0, 0.0], [0.3, 0.8]);
        let gap = 0.02;

        let horizontal = UiHorizontalStack::new(buttons.clone(), gap)
            .with_main_axis_allocation(UiMainAxisAllocation::Fill)
            .layout(horizontal_parent, &context);
        let vertical = UiVerticalStack::new(buttons, gap)
            .with_main_axis_allocation(UiMainAxisAllocation::Fill)
            .layout(vertical_parent, &context);
        let horizontal_width = horizontal
            .children
            .iter()
            .map(|child| child.rect.size[0])
            .sum::<f32>()
            + gap;
        let vertical_height = vertical
            .children
            .iter()
            .map(|child| child.rect.size[1])
            .sum::<f32>()
            + gap;

        assert!((horizontal_width - horizontal_parent.size[0]).abs() < 0.00001);
        assert!((vertical_height - vertical_parent.size[1]).abs() < 0.00001);
    }

    #[test]
    fn button_specs_produce_stable_activation_events() {
        let theme = UiTheme::default();
        let action = UiActionId(42);
        let layout = UiWorkspaceLayout::new_with_theme(
            [1280.0, 720.0],
            [
                UiButtonSpec::new(UiButtonId(0), "PREV").with_action(action),
                UiButtonSpec::new(UiButtonId(1), "PIN").with_action(UiActionId(44)),
                UiButtonSpec::new(UiButtonId(2), "NEXT")
                    .with_action(UiActionId(43))
                    .with_enabled(false),
            ],
            [
                UiCardSpec::new(UiCardRole::Browser, "browse", "shell"),
                UiCardSpec::new(UiCardRole::Editor, "edit", "select"),
                UiCardSpec::new(UiCardRole::Preview, "preview", "hover"),
            ],
            &theme,
        );
        let point = layout.buttons[0].rect.center;

        assert_eq!(layout.event_at(point, true), Some(UiEvent::Activated(action)));
        assert_eq!(layout.buttons[0].activation_event(point, true), Some(UiEvent::Activated(action)));
        assert_eq!(
            layout.focused_event(UiButtonId(0), UiActivationKey::Enter, true),
            Some(UiEvent::Activated(action))
        );
        assert_eq!(
            layout.focused_event(UiButtonId(1), UiActivationKey::Space, true),
            Some(UiEvent::Activated(UiActionId(44)))
        );
        assert_eq!(layout.focused_event(UiButtonId(0), UiActivationKey::Enter, false), None);
        assert_eq!(layout.buttons[0].activation_event(point, false), None);
        assert_eq!(layout.buttons[1].activation_event(point, true), None);
        let disabled_point = layout.buttons[2].rect.center;
        assert_eq!(layout.event_at(disabled_point, true), None);
        assert_eq!(layout.focused_event(UiButtonId(2), UiActivationKey::Space, true), None);
        assert_eq!(layout.next_focus(None, UiFocusDirection::Forward), Some(UiButtonId(0)));
        assert_eq!(
            layout.next_focus(Some(UiButtonId(0)), UiFocusDirection::Forward),
            Some(UiButtonId(1))
        );
        assert_eq!(
            layout.next_focus(Some(UiButtonId(1)), UiFocusDirection::Forward),
            Some(UiButtonId(0))
        );
        assert_eq!(
            layout.next_focus(Some(UiButtonId(0)), UiFocusDirection::Backward),
            Some(UiButtonId(1))
        );
        assert_eq!(
            layout.next_focus(Some(UiButtonId(2)), UiFocusDirection::Forward),
            Some(UiButtonId(0))
        );
    }

    #[test]
    fn focus_state_wraps_and_activates_only_actionable_controls() {
        let buttons = vec![
            UiButton::new(UiButtonId(0), "ONE", UiRect::new([-0.2, 0.0], [0.2, 0.1]))
                .with_action(UiActionId(10)),
            UiButton::new(UiButtonId(1), "DISABLED", UiRect::new([0.0, 0.0], [0.2, 0.1]))
                .with_action(UiActionId(11))
                .with_enabled(false),
            UiButton::new(UiButtonId(2), "TWO", UiRect::new([0.2, 0.0], [0.2, 0.1]))
                .with_action(UiActionId(12)),
        ];
        let mut focus = UiFocusState::new();

        focus.move_focus(&buttons, UiFocusDirection::Forward);
        assert_eq!(focus.focused(), Some(UiButtonId(0)));
        focus.move_focus(&buttons, UiFocusDirection::Forward);
        assert_eq!(focus.focused(), Some(UiButtonId(2)));
        assert_eq!(
            focus.activate(&buttons, UiActivationKey::Enter, true),
            Some(UiEvent::Activated(UiActionId(12)))
        );
        focus.move_focus(&buttons, UiFocusDirection::Forward);
        assert_eq!(focus.focused(), Some(UiButtonId(0)));
    }

    #[test]
    fn clipped_button_text_produces_a_warning_diagnostic() {
        let theme = UiTheme::default();
        let fitting = UiButton::new(
            UiButtonId(0),
            "OK",
            UiRect::new([0.0, 0.0], [0.5, 0.2]),
        );
        let clipped = UiButton::new(
            UiButtonId(1),
            "COMPILE PROJECT",
            UiRect::new([0.0, 0.0], [0.2, 0.2]),
        );

        assert_eq!(fitting.diagnostics(&theme), None);
        assert_eq!(
            clipped.diagnostics(&theme),
            Some(UiDiagnostic {
                severity: UiDiagnosticSeverity::Warning,
                kind: UiDiagnosticKind::TextClipped {
                    control: UiButtonId(1),
                    label: "COMPILE PROJECT",
                },
            })
        );
    }

    #[test]
    fn duplicate_control_and_action_ids_produce_diagnostics() {
        let theme = UiTheme::default();
        let mut layout = UiWorkspaceLayout::new(
            [1280.0, 720.0],
            [
                UiButtonSpec::new(UiButtonId(0), "ONE").with_action(UiActionId(1)),
                UiButtonSpec::new(UiButtonId(1), "TWO").with_action(UiActionId(2)),
                UiButtonSpec::new(UiButtonId(2), "THREE").with_action(UiActionId(3)),
            ],
            [
                UiCardSpec::new(UiCardRole::Browser, "browse", "shell"),
                UiCardSpec::new(UiCardRole::Editor, "edit", "select"),
                UiCardSpec::new(UiCardRole::Preview, "preview", "hover"),
            ],
        );
        layout.buttons[1].id = UiButtonId(0);
        layout.buttons[1].action = Some(UiActionId(1));
        layout.buttons[2].rect.size = [0.0, 0.0];
        layout.buttons[2].action = None;
        layout.buttons[2].label = "";

        let diagnostics = layout.diagnostics(&theme);
        assert!(diagnostics.iter().any(|diagnostic| {
            diagnostic.kind == UiDiagnosticKind::DuplicateControlId(UiButtonId(0))
        }));
        assert!(diagnostics.iter().any(|diagnostic| {
            diagnostic.kind == UiDiagnosticKind::DuplicateActionId(UiActionId(1))
        }));
        assert!(diagnostics.iter().any(|diagnostic| {
            diagnostic.kind == UiDiagnosticKind::ZeroSizeControl(UiButtonId(2))
        }));
        assert!(diagnostics.iter().any(|diagnostic| {
            diagnostic.kind == UiDiagnosticKind::FocusableWithoutAction(UiButtonId(2))
        }));
        assert!(diagnostics.iter().any(|diagnostic| {
            diagnostic.kind == UiDiagnosticKind::MissingControlLabel(UiButtonId(2))
        }));
    }

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

        assert_eq!(surfaces[0].rect, UiRect::new([0.25, 0.0], [0.5, 0.4]));
        assert_eq!(text[0].spec.rect, surfaces[0].rect);

        {
            let mut drawer = UiDrawer::new(&mut surfaces, &mut text, &theme);
            drawer.set_clip(Some(UiRect::new([-2.0, 0.0], [1.0, 1.0])));
            drawer.button(&button, UiInteractionState::Hovered, UiControlRole::Primary);
        }
        assert_eq!(surfaces.len(), 1);
        assert_eq!(text.len(), 1);
    }
}
