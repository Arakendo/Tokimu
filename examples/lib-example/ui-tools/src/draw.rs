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

/// Backend-neutral text draw request.
///
/// This contains semantic text and theme style only. Font rasterizers, glyph
/// atlases, meshes, and GPU handles are intentionally resolved downstream.
#[derive(Clone, Debug, PartialEq)]
pub struct UiTextCommand {
    pub spec: UiTextSpec,
    pub style: UiTextStyle,
}

impl UiTextCommand {
    pub fn new(spec: UiTextSpec, style: UiTextStyle) -> Self {
        Self { spec, style }
    }
}

pub struct UiDrawer<'a> {
    pub surfaces: &'a mut Vec<UiSurfaceCommand>,
    pub text: &'a mut Vec<UiTextCommand>,
    pub theme: &'a UiTheme,
    clip: Option<UiRect>,
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
            clip: None,
        }
    }

    pub fn set_clip(&mut self, clip: Option<UiRect>) {
        self.clip = clip;
    }

    fn clipped_rect(&self, rect: UiRect) -> Option<UiRect> {
        self.clip.map_or(Some(rect), |clip| rect.intersection(clip))
    }

    pub fn surface(&mut self, region: &UiRegion) {
        if let Some(rect) = self.clipped_rect(region.rect) {
            self.surfaces.push(UiSurfaceCommand {
                rect,
                style: self.theme.surface(region.role),
            });
        }
    }

    pub fn label(&mut self, label: &UiLabel, role: UiTextRole) {
        let spec = UiTextSpec::new(
            label.text,
            UiRect::new([label.position[0], label.position[1]], [0.0, 0.0]),
            role,
        )
        .with_alignment(label.anchor.into(), UiTextAlign::Center)
        .with_overflow(UiTextOverflow::Clip);
        if let Some(rect) = self.clipped_rect(spec.rect) {
            self.text.push(UiTextCommand {
                spec: UiTextSpec { rect, ..spec },
                style: self.theme.text(role),
            });
        }
    }

    pub fn chip(&mut self, chip: &UiStateChip, role: UiTextRole) {
        self.surface(&chip.region());
        let spec = UiTextSpec::new(chip.label, chip.rect, role);
        if let Some(rect) = self.clipped_rect(spec.rect) {
            self.text.push(UiTextCommand {
                spec: UiTextSpec { rect, ..spec },
                style: self.theme.text(role),
            });
        }
    }

    pub fn button(&mut self, button: &UiButton, state: UiInteractionState, role: UiControlRole) {
        if let Some(rect) = self.clipped_rect(button.rect) {
            self.surfaces.push(UiSurfaceCommand {
                rect,
                style: self.theme.control(role, state),
            });
            let spec = UiTextSpec::new(button.label, rect, UiTextRole::Button);
            self.text.push(UiTextCommand {
                spec,
                style: self.theme.text(UiTextRole::Button),
            });
        }
    }

    pub fn card(&mut self, card: &UiCard) {
        if let Some(rect) = self.clipped_rect(card.region.rect) {
            self.surfaces.push(UiSurfaceCommand {
                rect,
                style: self.theme.card(card.role),
            });
        }
        self.surface(&card.header);
        self.surface(&card.body_region);
        self.surface(&card.footer);

        let title = UiTextSpec::new(
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
        );
        if let Some(rect) = self.clipped_rect(title.rect) {
            self.text.push(UiTextCommand {
                spec: UiTextSpec { rect, ..title },
                style: self.theme.text(UiTextRole::Heading),
            });
        }

        let body = UiTextSpec::new(
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
        .with_alignment(UiTextAlign::Start, UiTextAlign::Center);
        if let Some(rect) = self.clipped_rect(body.rect) {
            self.text.push(UiTextCommand {
                spec: UiTextSpec { rect, ..body },
                style: self.theme.text(UiTextRole::Body),
            });
        }
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
