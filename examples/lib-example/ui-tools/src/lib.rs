mod controls;
mod draw;
mod geometry;
mod layout;
mod text;
mod theme;

pub use controls::{
    UiButton, UiButtonId, UiButtonSpec, UiCardSpec, UiInteractionState, UiLabel, UiLabelAnchor,
    UiLabelSpec, UiStateChip,
};
pub use draw::{UiDrawer, UiSurfaceCommand, UiTextCommand};
pub use geometry::{window_to_world, UiInsets, UiRect};
pub use layout::UiCardRole;
pub use layout::{
    UiCard, UiInspector, UiPanel, UiRadius, UiRegion, UiRegionKind, UiSidebar, UiSpacing,
    UiStatusBar, UiSurfaceRole, UiTabStrip, UiToolbar, UiToolbarLayout, UiWorkspace,
    UiWorkspaceLayout,
};
pub use text::{UiTextAlign, UiTextDirection, UiTextOverflow, UiTextRole, UiTextSpec};
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
    fn drawer_emits_surface_and_text_commands() {
        let theme = UiTheme::default();
        let mut surfaces = Vec::new();
        let mut text = Vec::new();
        let mut drawer = UiDrawer::new(&mut surfaces, &mut text, &theme);
        let button = UiButton::new(UiButtonId(7), "edit", UiRect::new([0.0, 0.0], [0.4, 0.1]));

        drawer.button(&button, UiInteractionState::Hovered, UiControlRole::Primary);

        assert_eq!(surfaces.len(), 1);
        assert_eq!(text.len(), 1);
        assert_eq!(surfaces[0].style.role, UiSurfaceRole::Accent);
        assert_eq!(text[0].text, "edit");
    }
}
