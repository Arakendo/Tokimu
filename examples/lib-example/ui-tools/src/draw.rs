use crate::{
    region::{UiCard, UiRegion},
    text::{UiTextAlign, UiTextOverflow, UiTextRole, UiTextSpec},
    UiButton, UiInteractionState, UiLabel, UiRect, UiStateChip,
};

use crate::theme::{UiControlRole, UiSurfaceStyle, UiTextStyle, UiTheme};

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct UiSurfaceCommand {
    pub rect: UiRect,
    pub style: UiSurfaceStyle,
}

#[derive(Clone, Debug, PartialEq)]
pub struct UiTextCommand {
    pub spec: UiTextSpec,
    pub style: UiTextStyle,
}

pub struct UiDrawer<'a> {
    pub surfaces: &'a mut Vec<UiSurfaceCommand>,
    pub text: &'a mut Vec<UiTextCommand>,
    pub theme: &'a UiTheme,
}

impl<'a> UiDrawer<'a> {
    pub fn new(
        surfaces: &'a mut Vec<UiSurfaceCommand>,
        text: &'a mut Vec<UiTextCommand>,
        theme: &'a UiTheme,
    ) -> Self {
        Self {
            surfaces,
            text,
            theme,
        }
    }

    pub fn surface(&mut self, region: &UiRegion) {
        self.surfaces.push(UiSurfaceCommand {
            rect: region.rect,
            style: self.theme.surface(region.role),
        });
    }

    pub fn label(&mut self, label: &UiLabel, role: UiTextRole) {
        self.text.push(UiTextCommand {
            spec: UiTextSpec::new(
                label.text,
                UiRect::new([label.position[0], label.position[1]], [0.0, 0.0]),
                role,
            )
            .with_alignment(label.anchor.into(), UiTextAlign::Center)
            .with_overflow(UiTextOverflow::Clip),
            style: self.theme.text(role),
        });
    }

    pub fn chip(&mut self, chip: &UiStateChip, role: UiTextRole) {
        self.surface(&chip.region());
        self.text.push(UiTextCommand {
            spec: UiTextSpec::new(chip.label, chip.rect, role),
            style: self.theme.text(role),
        });
    }

    pub fn button(&mut self, button: &UiButton, state: UiInteractionState, role: UiControlRole) {
        self.surfaces.push(UiSurfaceCommand {
            rect: button.rect,
            style: self.theme.control(role, state),
        });
        self.text.push(UiTextCommand {
            spec: UiTextSpec::new(button.label, button.rect, UiTextRole::Button),
            style: self.theme.text(UiTextRole::Button),
        });
    }

    pub fn card(&mut self, card: &UiCard) {
        self.surfaces.push(UiSurfaceCommand {
            rect: card.region.rect,
            style: self.theme.card(card.role),
        });
        self.surface(&card.header);
        self.surface(&card.body_region);
        self.surface(&card.footer);

        self.text.push(UiTextCommand {
            spec: UiTextSpec::new(
                card.title,
                // Keep the visual header band narrow, but give glyphs enough
                // vertical bounds to avoid clipping bitmap rows.
                UiRect::new(
                    card.header.rect.center,
                    [
                        card.header.rect.size[0] - card.padding.value() * 2.0,
                        card.region.rect.size[1] * 0.34,
                    ],
                ),
                UiTextRole::Heading,
            ),
            style: self.theme.text(UiTextRole::Heading),
        });
        self.text.push(UiTextCommand {
            spec: UiTextSpec::new(
                card.body,
                UiRect::new(
                    card.body_region.rect.center,
                    [
                        card.body_region.rect.size[0] - card.padding.value() * 2.0,
                        card.region.rect.size[1] * 0.34,
                    ],
                ),
                UiTextRole::Body,
            )
                .with_alignment(UiTextAlign::Start, UiTextAlign::Center),
            style: self.theme.text(UiTextRole::Body),
        });
    }

    pub fn workspace(&mut self, region: &UiRegion) {
        self.surface(region);
    }

    pub fn toolbar(&mut self, region: &UiRegion) {
        self.surface(region);
    }

    pub fn divider(&mut self, region: &UiRegion) {
        self.surface(region);
    }

    pub fn button_strip(
        &mut self,
        button: &UiButton,
        state: UiInteractionState,
        role: UiControlRole,
    ) {
        self.button(button, state, role);
    }
}
