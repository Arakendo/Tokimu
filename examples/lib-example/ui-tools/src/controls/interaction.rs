use super::UiButton;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum UiInteractionState {
    Idle,
    Hovered,
    Pressed,
    Focused,
    Selected,
    Disabled,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct UiButtonId(pub u8);

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct UiActionId(pub u16);

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum UiEvent {
    Activated(UiActionId),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum UiActivationKey {
    Enter,
    Space,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum UiFocusDirection {
    Forward,
    Backward,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct UiFocusState {
    focused: Option<UiButtonId>,
}

impl UiFocusState {
    pub const fn new() -> Self {
        Self { focused: None }
    }

    pub const fn focused(self) -> Option<UiButtonId> {
        self.focused
    }

    pub const fn set_focus(&mut self, focused: Option<UiButtonId>) {
        self.focused = focused;
    }

    pub fn clear(&mut self) {
        self.focused = None;
    }

    pub fn move_focus(&mut self, buttons: &[UiButton], direction: UiFocusDirection) {
        let focusable: Vec<UiButtonId> = buttons
            .iter()
            .filter(|button| button.enabled && button.action.is_some())
            .map(|button| button.id)
            .collect();
        if focusable.is_empty() {
            self.focused = None;
            return;
        }

        let current = self
            .focused
            .and_then(|focused| focusable.iter().position(|id| *id == focused));
        let next = match (current, direction) {
            (Some(index), UiFocusDirection::Forward) => (index + 1) % focusable.len(),
            (Some(index), UiFocusDirection::Backward) => {
                (index + focusable.len() - 1) % focusable.len()
            }
            (None, UiFocusDirection::Forward) => 0,
            (None, UiFocusDirection::Backward) => focusable.len() - 1,
        };
        self.focused = Some(focusable[next]);
    }

    pub fn activate(
        &self,
        buttons: &[UiButton],
        key: UiActivationKey,
        enabled: bool,
    ) -> Option<UiEvent> {
        self.focused.and_then(|focused| {
            buttons
                .iter()
                .find(|button| button.id == focused)
                .and_then(|button| button.focused_activation_event(true, key, enabled))
        })
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum UiDiagnosticSeverity {
    Warning,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum UiDiagnosticKind {
    TextClipped {
        control: UiButtonId,
        label: &'static str,
    },
    DuplicateControlId(UiButtonId),
    DuplicateActionId(UiActionId),
    ZeroSizeControl(UiButtonId),
    FocusableWithoutAction(UiButtonId),
    MissingControlLabel(UiButtonId),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct UiDiagnostic {
    pub severity: UiDiagnosticSeverity,
    pub kind: UiDiagnosticKind,
}
