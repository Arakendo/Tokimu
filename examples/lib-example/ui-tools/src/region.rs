use crate::{
    measure_bitmap_text_width, UiInsets, UiMeasurable, UiMeasureContext, UiRect, UiTextRole,
    UiTheme,
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
    Selected,
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
        Self::new(UiRegionKind::Sidebar, UiSurfaceRole::Panel, rect)
    }

    pub fn inspector(rect: UiRect) -> Self {
        Self::new(UiRegionKind::Inspector, UiSurfaceRole::Panel, rect)
    }

    pub fn status_bar(rect: UiRect) -> Self {
        Self::new(UiRegionKind::StatusBar, UiSurfaceRole::Overlay, rect)
    }

    pub fn tab_strip(rect: UiRect) -> Self {
        Self::new(UiRegionKind::TabStrip, UiSurfaceRole::Raised, rect)
    }

    pub fn card_grid(rect: UiRect) -> Self {
        Self::new(UiRegionKind::CardGrid, UiSurfaceRole::Card, rect)
    }

    pub fn canvas(rect: UiRect) -> Self {
        Self::new(UiRegionKind::Canvas, UiSurfaceRole::Panel, rect)
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
    pub fn from_intrinsic(
        role: UiCardRole,
        title: &'static str,
        body: &'static str,
        center: [f32; 2],
        theme: &UiTheme,
    ) -> Self {
        let size = Self::intrinsic_size_for(title, body, theme);
        Self::new(role, title, body, UiRegion::card(UiRect::new(center, size)))
    }

    pub fn intrinsic_size(&self, theme: &UiTheme) -> [f32; 2] {
        Self::intrinsic_size_for(self.title, self.body, theme)
    }

    pub fn measure(&self, context: &UiMeasureContext<'_>) -> [f32; 2] {
        context
            .constraints
            .constrain(self.intrinsic_size(context.theme))
    }

    fn intrinsic_size_for(title: &str, body: &str, theme: &UiTheme) -> [f32; 2] {
        let title_style = theme.text(UiTextRole::Heading);
        let body_style = theme.text(UiTextRole::Body);
        let padding = UiSpacing::Medium.value() * 2.0;
        let gap = theme.spacing.sm.value();
        let title_width = measure_bitmap_text_width(title, title_style.height);
        let body_width = measure_bitmap_text_width(body, body_style.height);
        [
            title_width.max(body_width) + padding,
            title_style.height + body_style.height + gap + padding,
        ]
    }

    pub fn new(
        role: UiCardRole,
        title: &'static str,
        body: &'static str,
        region: UiRegion,
    ) -> Self {
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
            // Keep the footer quieter than the header without making it read
            // as a separate dark overlay panel.
            UiSurfaceRole::Panel,
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

impl UiMeasurable for UiCard {
    fn measure(&self, context: &UiMeasureContext<'_>) -> [f32; 2] {
        UiCard::measure(self, context)
    }
}

pub type UiPanel = UiRegion;
pub type UiWorkspace = UiRegion;
pub type UiToolbar = UiRegion;
pub type UiSidebar = UiRegion;
pub type UiInspector = UiRegion;
pub type UiStatusBar = UiRegion;
pub type UiTabStrip = UiRegion;
