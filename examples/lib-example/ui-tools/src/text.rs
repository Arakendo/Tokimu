use crate::{UiLabelAnchor, UiRect};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum UiTextDirection {
    Ltr,
    Rtl,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum UiTextRole {
    Title,
    Heading,
    Body,
    Caption,
    Button,
    Chip,
    Status,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum UiTextAlign {
    Start,
    Center,
    End,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum UiTextOverflow {
    Clip,
    Ellipsis,
    Wrap,
}

#[derive(Clone, Debug, PartialEq)]
pub struct UiTextSpec {
    pub text: String,
    pub rect: UiRect,
    pub role: UiTextRole,
    pub direction: UiTextDirection,
    pub align_x: UiTextAlign,
    pub align_y: UiTextAlign,
    pub overflow: UiTextOverflow,
}

impl UiTextSpec {
    pub fn new(text: impl Into<String>, rect: UiRect, role: UiTextRole) -> Self {
        Self {
            text: text.into(),
            rect,
            role,
            direction: UiTextDirection::Ltr,
            align_x: UiTextAlign::Center,
            align_y: UiTextAlign::Center,
            overflow: UiTextOverflow::Clip,
        }
    }

    pub fn with_direction(mut self, direction: UiTextDirection) -> Self {
        self.direction = direction;
        self
    }

    pub fn with_alignment(mut self, align_x: UiTextAlign, align_y: UiTextAlign) -> Self {
        self.align_x = align_x;
        self.align_y = align_y;
        self
    }

    pub fn with_overflow(mut self, overflow: UiTextOverflow) -> Self {
        self.overflow = overflow;
        self
    }

    pub fn centered_bounds(&self) -> [f32; 2] {
        self.rect.center
    }
}

impl From<UiLabelAnchor> for UiTextAlign {
    fn from(anchor: UiLabelAnchor) -> Self {
        match anchor {
            UiLabelAnchor::Start => Self::Start,
            UiLabelAnchor::Center => Self::Center,
            UiLabelAnchor::End => Self::End,
        }
    }
}
