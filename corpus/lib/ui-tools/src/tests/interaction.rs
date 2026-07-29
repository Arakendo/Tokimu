use super::super::*;

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

    assert_eq!(
        layout.event_at(point, true),
        Some(UiEvent::Activated(action))
    );
    assert_eq!(
        layout.buttons[0].activation_event(point, true),
        Some(UiEvent::Activated(action))
    );
    assert_eq!(
        layout.focused_event(UiButtonId(0), UiActivationKey::Enter, true),
        Some(UiEvent::Activated(action))
    );
    assert_eq!(
        layout.focused_event(UiButtonId(1), UiActivationKey::Space, true),
        Some(UiEvent::Activated(UiActionId(44)))
    );
    assert_eq!(
        layout.focused_event(UiButtonId(0), UiActivationKey::Enter, false),
        None
    );
    assert_eq!(layout.buttons[0].activation_event(point, false), None);
    assert_eq!(layout.buttons[1].activation_event(point, true), None);
    let disabled_point = layout.buttons[2].rect.center;
    assert_eq!(layout.event_at(disabled_point, true), None);
    assert_eq!(
        layout.focused_event(UiButtonId(2), UiActivationKey::Space, true),
        None
    );
    assert_eq!(
        layout.next_focus(None, UiFocusDirection::Forward),
        Some(UiButtonId(0))
    );
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
        UiButton::new(
            UiButtonId(1),
            "DISABLED",
            UiRect::new([0.0, 0.0], [0.2, 0.1]),
        )
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
    let fitting = UiButton::new(UiButtonId(0), "OK", UiRect::new([0.0, 0.0], [0.5, 0.2]));
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
    assert!(diagnostics
        .iter()
        .any(|diagnostic| { diagnostic.kind == UiDiagnosticKind::ZeroSizeControl(UiButtonId(2)) }));
    assert!(diagnostics.iter().any(|diagnostic| {
        diagnostic.kind == UiDiagnosticKind::FocusableWithoutAction(UiButtonId(2))
    }));
    assert!(diagnostics.iter().any(|diagnostic| {
        diagnostic.kind == UiDiagnosticKind::MissingControlLabel(UiButtonId(2))
    }));
}
