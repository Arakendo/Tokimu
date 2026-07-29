use crate::{
    region::{UiCard, UiRegion},
    text::{UiTextAlign, UiTextOverflow, UiTextRole, UiTextSpec},
    UiButton, UiInteractionState, UiLabel, UiRect, UiStateChip,
};

use crate::theme::{UiControlRole, UiSurfaceStyle, UiTextStyle, UiTheme};
use crate::{PathBuilder, VectorPath};

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct UiSurfaceCommand {
    pub rect: UiRect,
    pub style: UiSurfaceStyle,
    /// Optional rectangular scissor region supplied by the layout layer.
    ///
    /// Vector lowering preserves the semantic geometry and carries this
    /// metadata forward. It does not synthesize rounded corners at a clip
    /// boundary; applying the scissor remains a renderer-adapter concern.
    pub clip: Option<UiRect>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum UiSurfaceVectorLayerKind {
    Shadow,
    Border,
    Fill,
}

#[derive(Clone, Debug, PartialEq)]
pub struct UiSurfaceVectorLayer {
    pub kind: UiSurfaceVectorLayerKind,
    pub path: VectorPath,
    pub role: crate::UiSurfaceRole,
    pub opacity: f32,
    pub clip: Option<UiRect>,
}

/// Lowers one semantic surface into ordered vector presentation layers.
///
/// The result deliberately contains no renderer or material handles. The
/// renderer adapter decides how each semantic role is painted later.
pub fn lower_surface_to_vector(command: &UiSurfaceCommand) -> Vec<UiSurfaceVectorLayer> {
    let rect = command.rect;
    let radius = radius_value(command.style.radius);
    let mut layers = Vec::with_capacity(3);

    if matches!(
        command.style.elevation,
        crate::UiElevation::Raised | crate::UiElevation::Floating
    ) {
        layers.push(UiSurfaceVectorLayer {
            kind: UiSurfaceVectorLayerKind::Shadow,
            path: rounded_rect_path(rect, radius, [0.01, -0.01]),
            role: crate::UiSurfaceRole::Overlay,
            opacity: command.style.opacity,
            clip: command.clip,
        });
    }

    if let Some(role) = command.style.border_role {
        let border_rect = UiRect::new(
            rect.center,
            [
                rect.size[0] + command.style.border_width * 2.0,
                rect.size[1] + command.style.border_width * 2.0,
            ],
        );
        layers.push(UiSurfaceVectorLayer {
            kind: UiSurfaceVectorLayerKind::Border,
            path: rounded_rect_path(border_rect, radius, [0.0, 0.0]),
            role,
            opacity: command.style.opacity,
            clip: command.clip,
        });
    }

    layers.push(UiSurfaceVectorLayer {
        kind: UiSurfaceVectorLayerKind::Fill,
        path: rounded_rect_path(rect, radius, [0.0, 0.0]),
        role: command.style.role,
        opacity: command.style.opacity,
        clip: command.clip,
    });
    layers
}

fn rounded_rect_path(rect: UiRect, radius: f32, offset: [f32; 2]) -> VectorPath {
    let min = [
        rect.center[0] - rect.size[0] * 0.5 + offset[0],
        rect.center[1] - rect.size[1] * 0.5 + offset[1],
    ];
    PathBuilder::new()
        .rounded_rect(min, rect.size, radius)
        .build()
}

fn radius_value(radius: crate::UiRadius) -> f32 {
    match radius {
        crate::UiRadius::None => 0.0,
        crate::UiRadius::Small => 0.01,
        crate::UiRadius::Medium => 0.02,
        crate::UiRadius::Large => 0.04,
    }
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
        if self.clipped_rect(region.rect).is_some() {
            self.surfaces.push(UiSurfaceCommand {
                rect: region.rect,
                style: self.theme.surface(region.role),
                clip: self.clip,
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
                rect: button.rect,
                style: self.theme.control(role, state),
                clip: self.clip,
            });
            let spec = UiTextSpec::new(button.label, rect, UiTextRole::Button);
            self.text.push(UiTextCommand {
                spec,
                style: self.theme.text(UiTextRole::Button),
            });
        }
    }

    pub fn card(&mut self, card: &UiCard) {
        if self.clipped_rect(card.region.rect).is_some() {
            self.surfaces.push(UiSurfaceCommand {
                rect: card.region.rect,
                style: self.theme.card(card.role),
                clip: self.clip,
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
