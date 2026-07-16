use crate::{
    region::UiCardRole,
    text::{UiTextAlign, UiTextOverflow, UiTextRole},
    UiInteractionState, UiRadius, UiSpacing, UiSurfaceRole,
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum UiControlRole {
    Primary,
    Secondary,
    Quiet,
    Accent,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum UiElevation {
    Flat,
    Raised,
    Floating,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct UiSpacingScale {
    pub xs: UiSpacing,
    pub sm: UiSpacing,
    pub md: UiSpacing,
    pub lg: UiSpacing,
    pub xl: UiSpacing,
}

impl Default for UiSpacingScale {
    fn default() -> Self {
        Self {
            xs: UiSpacing::Xs,
            sm: UiSpacing::Small,
            md: UiSpacing::Medium,
            lg: UiSpacing::Large,
            xl: UiSpacing::Xl,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct UiRadiusScale {
    pub none: UiRadius,
    pub sm: UiRadius,
    pub md: UiRadius,
    pub lg: UiRadius,
}

impl Default for UiRadiusScale {
    fn default() -> Self {
        Self {
            none: UiRadius::Small,
            sm: UiRadius::Small,
            md: UiRadius::Medium,
            lg: UiRadius::Large,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct UiBorderScale {
    pub hairline: f32,
    pub thin: f32,
    pub medium: f32,
}

impl Default for UiBorderScale {
    fn default() -> Self {
        Self {
            hairline: 0.004,
            thin: 0.007,
            medium: 0.012,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct UiSurfaceStyle {
    pub role: UiSurfaceRole,
    pub border_role: Option<UiSurfaceRole>,
    pub border_width: f32,
    pub opacity: f32,
    pub elevation: UiElevation,
    pub radius: UiRadius,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct UiTextStyle {
    pub role: UiTextRole,
    pub align: UiTextAlign,
    pub overflow: UiTextOverflow,
    pub height: f32,
    pub opacity: f32,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct UiTheme {
    pub spacing: UiSpacingScale,
    pub radii: UiRadiusScale,
    pub borders: UiBorderScale,
}

impl Default for UiTheme {
    fn default() -> Self {
        Self {
            spacing: UiSpacingScale::default(),
            radii: UiRadiusScale::default(),
            borders: UiBorderScale::default(),
        }
    }
}

impl UiTheme {
    pub fn surface(&self, role: UiSurfaceRole) -> UiSurfaceStyle {
        match role {
            UiSurfaceRole::Background => UiSurfaceStyle {
                role,
                border_role: None,
                border_width: 0.0,
                opacity: 1.0,
                elevation: UiElevation::Flat,
                radius: self.radii.none,
            },
            UiSurfaceRole::Region => UiSurfaceStyle {
                role,
                border_role: None,
                border_width: 0.0,
                opacity: 0.90,
                elevation: UiElevation::Flat,
                radius: self.radii.sm,
            },
            UiSurfaceRole::Panel => UiSurfaceStyle {
                role,
                border_role: Some(UiSurfaceRole::Overlay),
                border_width: self.borders.hairline,
                opacity: 0.95,
                elevation: UiElevation::Raised,
                radius: self.radii.md,
            },
            UiSurfaceRole::Card => UiSurfaceStyle {
                role,
                border_role: Some(UiSurfaceRole::Accent),
                border_width: self.borders.thin,
                opacity: 0.98,
                elevation: UiElevation::Raised,
                radius: self.radii.md,
            },
            UiSurfaceRole::Toolbar => UiSurfaceStyle {
                role,
                border_role: Some(UiSurfaceRole::Overlay),
                border_width: self.borders.thin,
                opacity: 0.94,
                elevation: UiElevation::Floating,
                radius: self.radii.md,
            },
            UiSurfaceRole::Raised => UiSurfaceStyle {
                role,
                border_role: Some(UiSurfaceRole::Accent),
                border_width: self.borders.hairline,
                opacity: 0.97,
                elevation: UiElevation::Raised,
                radius: self.radii.md,
            },
            UiSurfaceRole::Selected => UiSurfaceStyle {
                role,
                border_role: Some(UiSurfaceRole::Accent),
                border_width: self.borders.hairline,
                opacity: 0.95,
                elevation: UiElevation::Raised,
                radius: self.radii.md,
            },
            UiSurfaceRole::Accent => UiSurfaceStyle {
                role,
                border_role: Some(UiSurfaceRole::Selected),
                border_width: self.borders.hairline,
                opacity: 0.98,
                elevation: UiElevation::Floating,
                radius: self.radii.md,
            },
            UiSurfaceRole::Overlay => UiSurfaceStyle {
                role,
                border_role: None,
                border_width: 0.0,
                opacity: 0.82,
                elevation: UiElevation::Floating,
                radius: self.radii.sm,
            },
        }
    }

    pub fn control(&self, role: UiControlRole, state: UiInteractionState) -> UiSurfaceStyle {
        let base_role = match role {
            UiControlRole::Primary => UiSurfaceRole::Panel,
            UiControlRole::Secondary => UiSurfaceRole::Panel,
            UiControlRole::Quiet => UiSurfaceRole::Region,
            UiControlRole::Accent => UiSurfaceRole::Selected,
        };

        let mut style = self.surface(base_role);
        if matches!(role, UiControlRole::Quiet)
            && matches!(state, UiInteractionState::Idle)
        {
            style.border_width = self.borders.hairline;
            style.border_role = Some(UiSurfaceRole::Overlay);
        }
        match state {
            UiInteractionState::Idle => {}
            UiInteractionState::Hovered => {
                style.border_width = self.borders.thin;
                style.border_role = Some(UiSurfaceRole::Accent);
                style.elevation = UiElevation::Raised;
            }
            UiInteractionState::Pressed => {
                style.role = UiSurfaceRole::Selected;
                style.border_role = None;
            }
            UiInteractionState::Focused => {
                style.border_width = self.borders.medium;
                style.border_role = Some(UiSurfaceRole::Accent);
            }
            UiInteractionState::Selected => {
                style.role = UiSurfaceRole::Selected;
                style.border_role = None;
            }
            UiInteractionState::Disabled => {
                style.opacity = 0.58;
                style.elevation = UiElevation::Flat;
            }
        }

        style
    }

    pub fn card(&self, role: UiCardRole) -> UiSurfaceStyle {
        match role {
            UiCardRole::Browser => self.surface(UiSurfaceRole::Card),
            UiCardRole::Editor => self.surface(UiSurfaceRole::Raised),
            UiCardRole::Preview => self.surface(UiSurfaceRole::Accent),
            UiCardRole::Selected => self.surface(UiSurfaceRole::Selected),
            UiCardRole::Inspector => self.surface(UiSurfaceRole::Panel),
            UiCardRole::Status => self.surface(UiSurfaceRole::Overlay),
        }
    }

    pub fn text(&self, role: UiTextRole) -> UiTextStyle {
        match role {
            UiTextRole::Title => UiTextStyle {
                role,
                align: UiTextAlign::Start,
                overflow: UiTextOverflow::Clip,
                height: 0.056,
                opacity: 1.0,
            },
            UiTextRole::Heading => UiTextStyle {
                role,
                align: UiTextAlign::Center,
                overflow: UiTextOverflow::Clip,
                height: 0.043,
                opacity: 1.0,
            },
            UiTextRole::Body => UiTextStyle {
                role,
                align: UiTextAlign::Start,
                overflow: UiTextOverflow::Clip,
                height: 0.034,
                opacity: 0.92,
            },
            UiTextRole::Caption => UiTextStyle {
                role,
                align: UiTextAlign::Center,
                overflow: UiTextOverflow::Clip,
                height: 0.029,
                opacity: 0.85,
            },
            UiTextRole::Button => UiTextStyle {
                role,
                align: UiTextAlign::Center,
                overflow: UiTextOverflow::Clip,
                height: 0.036,
                opacity: 1.0,
            },
            UiTextRole::Chip | UiTextRole::Status => UiTextStyle {
                role,
                align: UiTextAlign::Center,
                overflow: UiTextOverflow::Clip,
                height: 0.027,
                opacity: 0.88,
            },
        }
    }
}
