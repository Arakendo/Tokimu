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
