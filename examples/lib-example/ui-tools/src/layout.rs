use crate::{
    UiButton, UiButtonId, UiButtonSpec, UiCardSpec, UiInsets, UiLabel, UiLabelAnchor, UiRect,
    UiStateChip,
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum UiSurfaceRole {
    Background,
    Region,
    Panel,
    Card,
    Toolbar,
    Raised,
    Selected,
    Accent,
    Overlay,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum UiSpacing {
    Xs,
    Small,
    Medium,
    Large,
    Xl,
}

impl UiSpacing {
    pub const fn value(self) -> f32 {
        match self {
            Self::Xs => 0.01,
            Self::Small => 0.02,
            Self::Medium => 0.03,
            Self::Large => 0.05,
            Self::Xl => 0.08,
        }
    }

    pub const fn insets(self) -> UiInsets {
        UiInsets::uniform(self.value())
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum UiRadius {
    Small,
    Medium,
    Large,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum UiRegionKind {
    Workspace,
    Header,
    Toolbar,
    Sidebar,
    Inspector,
    StatusBar,
    TabStrip,
    CardGrid,
    Card,
    Canvas,
    Panel,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum UiCardRole {
    Browser,
    Editor,
    Preview,
    Inspector,
    Status,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct UiRegion {
    pub kind: UiRegionKind,
    pub role: UiSurfaceRole,
    pub rect: UiRect,
}

impl UiRegion {
    pub fn new(kind: UiRegionKind, role: UiSurfaceRole, rect: UiRect) -> Self {
        Self { kind, role, rect }
    }

    pub fn contains(&self, point: [f32; 2]) -> bool {
        self.rect.contains(point)
    }

    pub fn inset(self, amount: f32) -> Self {
        Self {
            rect: self.rect.inset(amount),
            ..self
        }
    }

    pub fn workspace(rect: UiRect) -> Self {
        Self::new(UiRegionKind::Workspace, UiSurfaceRole::Region, rect)
    }

    pub fn header(rect: UiRect) -> Self {
        Self::new(UiRegionKind::Header, UiSurfaceRole::Raised, rect)
    }

    pub fn toolbar(rect: UiRect) -> Self {
        Self::new(UiRegionKind::Toolbar, UiSurfaceRole::Toolbar, rect)
    }

    pub fn sidebar(rect: UiRect) -> Self {
        Self::new(UiRegionKind::Sidebar, UiSurfaceRole::Region, rect)
    }

    pub fn inspector(rect: UiRect) -> Self {
        Self::new(UiRegionKind::Inspector, UiSurfaceRole::Region, rect)
    }

    pub fn status_bar(rect: UiRect) -> Self {
        Self::new(UiRegionKind::StatusBar, UiSurfaceRole::Overlay, rect)
    }

    pub fn tab_strip(rect: UiRect) -> Self {
        Self::new(UiRegionKind::TabStrip, UiSurfaceRole::Raised, rect)
    }

    pub fn card_grid(rect: UiRect) -> Self {
        Self::new(UiRegionKind::CardGrid, UiSurfaceRole::Region, rect)
    }

    pub fn canvas(rect: UiRect) -> Self {
        Self::new(UiRegionKind::Canvas, UiSurfaceRole::Region, rect)
    }

    pub fn panel(rect: UiRect) -> Self {
        Self::new(UiRegionKind::Panel, UiSurfaceRole::Panel, rect)
    }

    pub fn card(rect: UiRect) -> Self {
        Self::new(UiRegionKind::Card, UiSurfaceRole::Card, rect)
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct UiCard {
    pub role: UiCardRole,
    pub title: &'static str,
    pub body: &'static str,
    pub region: UiRegion,
    pub header: UiRegion,
    pub body_region: UiRegion,
    pub footer: UiRegion,
    pub padding: UiSpacing,
    pub surface_role: UiSurfaceRole,
}

impl UiCard {
    pub fn new(role: UiCardRole, title: &'static str, body: &'static str, region: UiRegion) -> Self {
        let padding = UiSpacing::Medium;
        let inner = region.rect.inset_by(padding.insets());
        let header_height = (inner.size[1] * 0.24).max(0.01);
        let body_height = (inner.size[1] * 0.36).max(0.01);
        let footer_height = (inner.size[1] * 0.16).max(0.01);
        let header = UiRegion::new(
            UiRegionKind::Panel,
            UiSurfaceRole::Raised,
            UiRect::new(
                [inner.center[0], inner.center[1] + inner.size[1] * 0.24],
                [inner.size[0], header_height],
            ),
        );
        let body_region = UiRegion::new(
            UiRegionKind::Panel,
            UiSurfaceRole::Panel,
            UiRect::new(
                [inner.center[0], inner.center[1] - inner.size[1] * 0.02],
                [inner.size[0], body_height],
            ),
        );
        let footer = UiRegion::new(
            UiRegionKind::Panel,
            UiSurfaceRole::Overlay,
            UiRect::new(
                [inner.center[0], inner.center[1] - inner.size[1] * 0.26],
                [inner.size[0], footer_height],
            ),
        );

        Self {
            role,
            title,
            body,
            region,
            header,
            body_region,
            footer,
            padding,
            surface_role: UiSurfaceRole::Card,
        }
    }
}

pub type UiPanel = UiRegion;
pub type UiWorkspace = UiRegion;
pub type UiToolbar = UiRegion;
pub type UiSidebar = UiRegion;
pub type UiInspector = UiRegion;
pub type UiStatusBar = UiRegion;
pub type UiTabStrip = UiRegion;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct UiWorkspaceLayout {
    pub workspace: UiRegion,
    pub header: UiRegion,
    pub toolbar: UiToolbar,
    pub sidebar: UiSidebar,
    pub inspector: UiInspector,
    pub canvas: UiRegion,
    pub status_bar: UiStatusBar,
    pub card_grid: UiRegion,
    pub title_chip: UiStateChip,
    pub subtitle_chip: UiStateChip,
    pub footer_chip: UiStateChip,
    pub title_label: UiLabel,
    pub subtitle_label: UiLabel,
    pub footer_label: UiLabel,
    pub cards: [UiCard; 3],
    pub buttons: [UiButton; 3],
}

pub type UiToolbarLayout = UiWorkspaceLayout;

impl UiWorkspaceLayout {
    pub fn new(window_size: [f32; 2], button_specs: [UiButtonSpec; 3], card_specs: [UiCardSpec; 3]) -> Self {
        let width = window_size[0].max(1.0);
        let height = window_size[1].max(1.0);
        let half_height = 1.0;
        let half_width = half_height * (width / height);

        let workspace = UiRegion::workspace(UiRect::new([0.0, 0.0], [half_width * 1.92, half_height * 1.48]));
        let header = UiRegion::header(UiRect::new([0.0, half_height - 0.12], [half_width * 1.50, 0.12]));
        let toolbar = UiRegion::toolbar(UiRect::new([-half_width + 0.94, half_height - 0.39], [1.46, 0.15]));
        let sidebar = UiRegion::sidebar(UiRect::new([-half_width + 0.22, -0.03], [0.28, 0.82]));
        let inspector = UiRegion::inspector(UiRect::new([half_width - 0.22, -0.03], [0.28, 0.82]));
        let canvas = UiRegion::canvas(UiRect::new([0.0, -0.01], [half_width * 1.04, 0.92]));
        let status_bar = UiRegion::status_bar(UiRect::new([0.0, -half_height + 0.13], [1.00, 0.08]));
        let card_grid = UiRegion::card_grid(UiRect::new([0.0, -half_height + 0.42], [half_width * 1.56, 0.18]));
        let title_chip = UiStateChip::new("WORKSPACE", UiRect::new([-half_width + 0.18, half_height - 0.12], [0.62, 0.08]));
        let subtitle_chip = UiStateChip::new("REGIONS + SURFACES + STATES", UiRect::new([-half_width + 1.00, half_height - 0.12], [1.16, 0.08]));
        let footer_chip = UiStateChip::new("SHARED SEMANTICS", UiRect::new([0.0, -half_height + 0.13], [0.84, 0.08]));
        let title_label = UiLabel::new("UI FRAMEWORK", UiLabelAnchor::Start, [-half_width + 0.18, half_height - 0.14]);
        let subtitle_label = UiLabel::new("REGIONS + SURFACES + STATES", UiLabelAnchor::Start, [-half_width + 1.02, half_height - 0.14]);
        let footer_label = UiLabel::new("SHARED SEMANTICS", UiLabelAnchor::Center, [0.0, -half_height + 0.14]);
        let card_width = ((half_width * 1.72 - 0.48) / 3.0).max(0.34);
        let card_y = -half_height + 0.42;
        let cards = [
            UiCard::new(card_specs[0].role, card_specs[0].title, card_specs[0].body, UiRegion::card(UiRect::new([-card_width - 0.22, card_y], [card_width, 0.18]))),
            UiCard::new(card_specs[1].role, card_specs[1].title, card_specs[1].body, UiRegion::card(UiRect::new([0.0, card_y], [card_width, 0.18]))),
            UiCard::new(card_specs[2].role, card_specs[2].title, card_specs[2].body, UiRegion::card(UiRect::new([card_width + 0.22, card_y], [card_width, 0.18]))),
        ];

        let button_size = [0.38, 0.10];
        let first_x = toolbar.rect.center[0] - toolbar.rect.size[0] * 0.28;
        let second_x = status_bar.rect.center[0];
        let third_x = toolbar.rect.center[0] + toolbar.rect.size[0] * 0.28;
        let button_y = status_bar.rect.center[1];
        let buttons = [
            UiButton::new(button_specs[0].id, button_specs[0].label, UiRect::new([first_x, button_y], button_size)),
            UiButton::new(button_specs[1].id, button_specs[1].label, UiRect::new([second_x, button_y], button_size)),
            UiButton::new(button_specs[2].id, button_specs[2].label, UiRect::new([third_x, button_y], button_size)),
        ];

        Self { workspace, header, toolbar, sidebar, inspector, canvas, status_bar, card_grid, title_chip, subtitle_chip, footer_chip, title_label, subtitle_label, footer_label, cards, buttons }
    }

    pub fn button_at(&self, point: [f32; 2]) -> Option<UiButtonId> {
        self.buttons.iter().find(|button| button.contains(point)).map(|button| button.id)
    }

    pub fn button_label(&self, id: UiButtonId) -> &'static str {
        self.buttons.iter().find(|button| button.id == id).map(|button| button.label).unwrap_or("unknown")
    }
}
