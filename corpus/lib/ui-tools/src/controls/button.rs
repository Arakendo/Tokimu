use crate::{
    text::{measure_bitmap_text_width, UiTextRole},
    UiControlRole, UiMeasurable, UiMeasureContext, UiRect, UiTheme,
};

use super::{
    UiActionId, UiActivationKey, UiButtonId, UiDiagnostic, UiDiagnosticKind, UiDiagnosticSeverity,
    UiEvent, UiInteractionState,
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct UiButtonSpec {
    pub id: UiButtonId,
    pub label: &'static str,
    pub action: Option<UiActionId>,
    pub enabled: bool,
}

impl UiButtonSpec {
    pub const fn new(id: UiButtonId, label: &'static str) -> Self {
        Self {
            id,
            label,
            action: None,
            enabled: true,
        }
    }

    pub const fn with_action(mut self, action: UiActionId) -> Self {
        self.action = Some(action);
        self
    }

    pub const fn with_enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct UiButton {
    pub id: UiButtonId,
    pub label: &'static str,
    pub rect: UiRect,
    pub action: Option<UiActionId>,
    pub enabled: bool,
}

impl UiButton {
    pub fn new(id: UiButtonId, label: &'static str, rect: UiRect) -> Self {
        Self {
            id,
            label,
            rect,
            action: None,
            enabled: true,
        }
    }

    pub fn with_action(mut self, action: UiActionId) -> Self {
        self.action = Some(action);
        self
    }

    pub fn with_enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }

    pub fn from_intrinsic(
        id: UiButtonId,
        label: &'static str,
        center: [f32; 2],
        theme: &UiTheme,
    ) -> Self {
        let size = Self::intrinsic_size_for(label, theme);
        Self::new(id, label, UiRect::new(center, size))
    }

    pub fn intrinsic_size(&self, theme: &UiTheme) -> [f32; 2] {
        Self::intrinsic_size_for(self.label, theme)
    }

    pub fn measure(&self, context: &UiMeasureContext<'_>) -> [f32; 2] {
        context
            .constraints
            .constrain(self.intrinsic_size(context.theme))
    }

    pub fn text_clips(&self, theme: &UiTheme) -> bool {
        let text = theme.text(UiTextRole::Button);
        let padding = theme.spacing.md.value() * 2.0;
        let border = theme
            .control(UiControlRole::Primary, UiInteractionState::Idle)
            .border_width
            * 2.0;
        let available_width = (self.rect.size[0] - padding - border).max(0.0);
        measure_bitmap_text_width(self.label, text.height) > available_width
    }

    pub fn diagnostics(&self, theme: &UiTheme) -> Option<UiDiagnostic> {
        self.text_clips(theme).then_some(UiDiagnostic {
            severity: UiDiagnosticSeverity::Warning,
            kind: UiDiagnosticKind::TextClipped {
                control: self.id,
                label: self.label,
            },
        })
    }

    fn intrinsic_size_for(label: &str, theme: &UiTheme) -> [f32; 2] {
        let text = theme.text(UiTextRole::Button);
        let padding = theme.spacing.md.value() * 2.0;
        let border = theme
            .control(UiControlRole::Primary, UiInteractionState::Idle)
            .border_width
            * 2.0;
        [
            measure_bitmap_text_width(label, text.height) + padding + border,
            text.height + padding + border,
        ]
    }

    pub fn contains(&self, point: [f32; 2]) -> bool {
        self.rect.contains(point)
    }

    pub fn activation_event(&self, point: [f32; 2], enabled: bool) -> Option<UiEvent> {
        (enabled && self.enabled)
            .then_some(self.action)
            .flatten()
            .filter(|_| self.contains(point))
            .map(UiEvent::Activated)
    }

    pub fn focused_activation_event(
        &self,
        focused: bool,
        key: UiActivationKey,
        enabled: bool,
    ) -> Option<UiEvent> {
        focused
            .then_some(key)
            .filter(|key| matches!(key, UiActivationKey::Enter | UiActivationKey::Space))
            .and_then(|_| (enabled && self.enabled).then_some(self.action))
            .flatten()
            .map(UiEvent::Activated)
    }

    pub fn label_anchor(&self) -> [f32; 2] {
        [
            self.rect.center[0],
            self.rect.center[1] - self.rect.size[1] * 0.02,
        ]
    }

    pub fn interaction_state(
        &self,
        hovered: bool,
        pressed: bool,
        selected: bool,
        disabled: bool,
    ) -> UiInteractionState {
        if disabled {
            UiInteractionState::Disabled
        } else if pressed {
            UiInteractionState::Pressed
        } else if selected {
            UiInteractionState::Selected
        } else if hovered {
            UiInteractionState::Hovered
        } else {
            UiInteractionState::Idle
        }
    }

    pub fn region(&self) -> crate::UiRegion {
        crate::UiRegion::panel(self.rect)
    }
}

impl UiMeasurable for UiButton {
    fn measure(&self, context: &UiMeasureContext<'_>) -> [f32; 2] {
        UiButton::measure(self, context)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn intrinsic_button_width_follows_label_measurement() {
        let theme = UiTheme::default();
        let short = UiButton::from_intrinsic(UiButtonId(0), "OK", [0.0, 0.0], &theme);
        let long = UiButton::from_intrinsic(UiButtonId(1), "COMPILE PROJECT", [0.0, 0.0], &theme);

        assert!(long.rect.size[0] > short.rect.size[0]);
        assert_eq!(short.rect.size[1], long.rect.size[1]);
    }
}
