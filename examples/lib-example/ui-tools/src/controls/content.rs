use crate::{region::UiCardRole, UiRect};

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct UiStateChip {
    pub label: &'static str,
    pub rect: UiRect,
}

impl UiStateChip {
    pub fn new(label: &'static str, rect: UiRect) -> Self {
        Self { label, rect }
    }

    pub fn contains(&self, point: [f32; 2]) -> bool {
        self.rect.contains(point)
    }

    pub fn region(&self) -> crate::UiRegion {
        crate::UiRegion::panel(self.rect)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum UiLabelAnchor {
    Start,
    Center,
    End,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct UiLabelSpec {
    pub text: &'static str,
    pub anchor: UiLabelAnchor,
}

impl UiLabelSpec {
    pub const fn new(text: &'static str, anchor: UiLabelAnchor) -> Self {
        Self { text, anchor }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct UiLabel {
    pub text: &'static str,
    pub anchor: UiLabelAnchor,
    pub position: [f32; 2],
}

impl UiLabel {
    pub fn new(text: &'static str, anchor: UiLabelAnchor, position: [f32; 2]) -> Self {
        Self {
            text,
            anchor,
            position,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct UiCardSpec {
    pub role: UiCardRole,
    pub title: &'static str,
    pub body: &'static str,
}

impl UiCardSpec {
    pub const fn new(role: UiCardRole, title: &'static str, body: &'static str) -> Self {
        Self { role, title, body }
    }
}
