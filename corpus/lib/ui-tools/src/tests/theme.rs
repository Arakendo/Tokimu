use super::super::*;

#[test]
fn admitted_themes_cover_every_surface_text_control_and_state() {
    for theme in [UiTheme::default(), UiTheme::high_contrast()] {
        assert_eq!(theme.diagnostics(), Vec::<UiThemeDiagnostic>::new());
    }
}

#[test]
fn high_contrast_theme_strengthens_structural_output() {
    let standard = UiTheme::default();
    let high_contrast = UiTheme::high_contrast();

    for role in UiControlRole::ALL {
        for state in UiInteractionState::ALL {
            let normal = standard.control(role, state);
            let contrast = high_contrast.control(role, state);
            assert_eq!(contrast.control_role, Some(role));
            assert!(contrast.border_role.is_some());
            assert!(contrast.border_width >= normal.border_width);
            assert!(contrast.opacity >= normal.opacity);
        }
    }

    for role in UiTextRole::ALL {
        assert_eq!(high_contrast.text(role).opacity, 1.0);
    }
}

#[test]
fn danger_intent_survives_each_interaction_state() {
    let theme = UiTheme::high_contrast();
    for state in UiInteractionState::ALL {
        let danger = theme.control(UiControlRole::Danger, state);
        let primary = theme.control(UiControlRole::Primary, state);
        assert_eq!(danger.control_role, Some(UiControlRole::Danger));
        assert_ne!(danger, primary);
    }
}

#[test]
fn malformed_theme_tokens_are_diagnosed() {
    let mut theme = UiTheme::default();
    theme.borders.thin = f32::NAN;
    theme.borders.medium = -1.0;

    let diagnostics = theme.diagnostics();
    assert!(diagnostics.contains(&UiThemeDiagnostic::InvalidBorderScale));
    assert!(diagnostics
        .iter()
        .any(|diagnostic| matches!(diagnostic, UiThemeDiagnostic::InvalidSurfaceStyle(_))));
}
