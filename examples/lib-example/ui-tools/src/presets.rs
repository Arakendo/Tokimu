use crate::{
    region::{UiCard, UiInspector, UiRegion, UiSidebar, UiStatusBar, UiToolbar},
    UiButton, UiButtonId, UiButtonSpec, UiCardSpec, UiHorizontalStack, UiLabel, UiLabelAnchor,
    UiActivationKey, UiDiagnostic, UiDiagnosticKind, UiDiagnosticSeverity, UiEvent,
    UiFocusDirection, UiMeasureContext, UiRect, UiStateChip, UiTheme,
};

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
    pub fn new(
        window_size: [f32; 2],
        button_specs: [UiButtonSpec; 3],
        card_specs: [UiCardSpec; 3],
    ) -> Self {
        Self::new_with_theme(window_size, button_specs, card_specs, &UiTheme::default())
    }

    pub fn new_with_theme(
        window_size: [f32; 2],
        button_specs: [UiButtonSpec; 3],
        card_specs: [UiCardSpec; 3],
        theme: &UiTheme,
    ) -> Self {
        let width = window_size[0].max(1.0);
        let height = window_size[1].max(1.0);
        let half_height = 1.0;
        let half_width = half_height * (width / height);

        let workspace = UiRegion::workspace(UiRect::new(
            [0.0, 0.0],
            [half_width * 1.92, half_height * 1.48],
        ));
        let header = UiRegion::header(UiRect::new(
            [0.0, half_height - 0.12],
            [half_width * 1.50, 0.12],
        ));
        let toolbar = UiRegion::toolbar(UiRect::new(
            [-half_width + 0.94, half_height - 0.39],
            [1.46, 0.15],
        ));
        let sidebar = UiRegion::sidebar(UiRect::new([-half_width + 0.49, -0.01], [0.36, 0.92]));
        let inspector = UiRegion::inspector(UiRect::new([half_width - 0.49, -0.01], [0.36, 0.92]));
        let canvas = UiRegion::canvas(UiRect::new([0.0, -0.01], [half_width * 1.10, 0.92]));
        let status_bar =
            UiRegion::status_bar(UiRect::new([0.0, -half_height + 0.13], [1.00, 0.08]));
        let card_grid = UiRegion::card_grid(UiRect::new(
            [0.0, -half_height + 0.37],
            [half_width * 1.56, 0.26],
        ));
        let title_chip = UiStateChip::new(
            "WORKSPACE",
            UiRect::new([-half_width + 0.18, half_height - 0.12], [0.62, 0.08]),
        );
        let subtitle_chip = UiStateChip::new(
            "REGIONS + SURFACES + STATES",
            UiRect::new([-half_width + 1.00, half_height - 0.12], [1.16, 0.08]),
        );
        let footer_chip = UiStateChip::new(
            "SHARED SEMANTICS",
            UiRect::new([0.0, -half_height + 0.13], [0.84, 0.08]),
        );
        let title_label = UiLabel::new(
            "UI FRAMEWORK",
            UiLabelAnchor::Start,
            [-half_width + 0.18, half_height - 0.14],
        );
        let subtitle_label = UiLabel::new(
            "REGIONS + SURFACES + STATES",
            UiLabelAnchor::Start,
            [-half_width + 1.02, half_height - 0.14],
        );
        let footer_label = UiLabel::new(
            "SHARED SEMANTICS",
            UiLabelAnchor::Center,
            [0.0, -half_height + 0.14],
        );
        let card_width = ((half_width * 1.72 - 0.48) / 3.0).max(0.34);
        let card_y = -half_height + 0.37;
        let cards = [
            UiCard::new(
                card_specs[0].role,
                card_specs[0].title,
                card_specs[0].body,
                UiRegion::card(UiRect::new(
                    [-card_width - 0.22, card_y],
                    [card_width, 0.22],
                )),
            ),
            UiCard::new(
                card_specs[1].role,
                card_specs[1].title,
                card_specs[1].body,
                UiRegion::card(UiRect::new([0.0, card_y], [card_width, 0.22])),
            ),
            UiCard::new(
                card_specs[2].role,
                card_specs[2].title,
                card_specs[2].body,
                UiRegion::card(UiRect::new([card_width + 0.22, card_y], [card_width, 0.22])),
            ),
        ];

        let button_y = toolbar.rect.center[1];
        let mut buttons = [
            UiButton::new(
                button_specs[0].id,
                button_specs[0].label,
                UiRect::new([0.0, button_y], [0.0, 0.0]),
            ),
            UiButton::new(
                button_specs[1].id,
                button_specs[1].label,
                UiRect::new([0.0, button_y], [0.0, 0.0]),
            ),
            UiButton::new(
                button_specs[2].id,
                button_specs[2].label,
                UiRect::new([0.0, button_y], [0.0, 0.0]),
            ),
        ];
        for (button, spec) in buttons.iter_mut().zip(button_specs) {
            button.action = spec.action;
            button.enabled = spec.enabled;
        }
        let gap = theme.spacing.sm.value();
        let stack = UiHorizontalStack::new(buttons.to_vec(), gap);
        let context = UiMeasureContext::new(theme, toolbar.rect.size);
        let layout = stack.layout(toolbar.rect, &context);
        for (button, child) in buttons.iter_mut().zip(layout.children) {
            button.rect = child.rect;
        }

        Self {
            workspace,
            header,
            toolbar,
            sidebar,
            inspector,
            canvas,
            status_bar,
            card_grid,
            title_chip,
            subtitle_chip,
            footer_chip,
            title_label,
            subtitle_label,
            footer_label,
            cards,
            buttons,
        }
    }

    pub fn button_at(&self, point: [f32; 2]) -> Option<UiButtonId> {
        self.buttons
            .iter()
            .find(|button| button.contains(point))
            .map(|button| button.id)
    }

    pub fn event_at(&self, point: [f32; 2], enabled: bool) -> Option<UiEvent> {
        self.buttons
            .iter()
            .find_map(|button| button.activation_event(point, enabled))
    }

    pub fn focused_event(
        &self,
        focused: UiButtonId,
        key: UiActivationKey,
        enabled: bool,
    ) -> Option<UiEvent> {
        self.buttons
            .iter()
            .find(|button| button.id == focused)
            .and_then(|button| button.focused_activation_event(true, key, enabled))
    }

    pub fn next_focus(
        &self,
        current: Option<UiButtonId>,
        direction: UiFocusDirection,
    ) -> Option<UiButtonId> {
        let focusable: Vec<UiButtonId> = self
            .buttons
            .iter()
            .filter(|button| button.enabled && button.action.is_some())
            .map(|button| button.id)
            .collect();
        if focusable.is_empty() {
            return None;
        }

        let current_index = current.and_then(|id| focusable.iter().position(|candidate| *candidate == id));
        let next_index = match (current_index, direction) {
            (Some(index), UiFocusDirection::Forward) => (index + 1) % focusable.len(),
            (Some(index), UiFocusDirection::Backward) => {
                (index + focusable.len() - 1) % focusable.len()
            }
            (None, UiFocusDirection::Forward) => 0,
            (None, UiFocusDirection::Backward) => focusable.len() - 1,
        };
        Some(focusable[next_index])
    }

    pub fn diagnostics(&self, theme: &UiTheme) -> Vec<UiDiagnostic> {
        let mut diagnostics = self
            .buttons
            .iter()
            .filter_map(|button| button.diagnostics(theme))
            .collect::<Vec<_>>();
        let mut control_ids = Vec::new();
        let mut action_ids = Vec::new();
        for button in &self.buttons {
            if button.rect.size[0] <= 0.0 || button.rect.size[1] <= 0.0 {
                diagnostics.push(UiDiagnostic {
                    severity: UiDiagnosticSeverity::Warning,
                    kind: UiDiagnosticKind::ZeroSizeControl(button.id),
                });
            }

            if button.enabled && button.action.is_none() {
                diagnostics.push(UiDiagnostic {
                    severity: UiDiagnosticSeverity::Warning,
                    kind: UiDiagnosticKind::FocusableWithoutAction(button.id),
                });
            }

            if button.label.trim().is_empty() {
                diagnostics.push(UiDiagnostic {
                    severity: UiDiagnosticSeverity::Warning,
                    kind: UiDiagnosticKind::MissingControlLabel(button.id),
                });
            }

            if control_ids.contains(&button.id) {
                diagnostics.push(UiDiagnostic {
                    severity: UiDiagnosticSeverity::Warning,
                    kind: UiDiagnosticKind::DuplicateControlId(button.id),
                });
            } else {
                control_ids.push(button.id);
            }

            if let Some(action) = button.action {
                if action_ids.contains(&action) {
                    diagnostics.push(UiDiagnostic {
                        severity: UiDiagnosticSeverity::Warning,
                        kind: UiDiagnosticKind::DuplicateActionId(action),
                    });
                } else {
                    action_ids.push(action);
                }
            }
        }
        diagnostics
    }

    pub fn button_label(&self, id: UiButtonId) -> &'static str {
        self.buttons
            .iter()
            .find(|button| button.id == id)
            .map(|button| button.label)
            .unwrap_or("unknown")
    }
}
