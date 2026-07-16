use crate::{
    region::{UiCard, UiInspector, UiRegion, UiSidebar, UiStatusBar, UiToolbar},
    UiButton, UiButtonId, UiButtonSpec, UiCardSpec, UiLabel, UiLabelAnchor, UiRect, UiStateChip,
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

        let button_size = [0.38, 0.10];
        let first_x = toolbar.rect.center[0] - toolbar.rect.size[0] * 0.28;
        let second_x = toolbar.rect.center[0];
        let third_x = toolbar.rect.center[0] + toolbar.rect.size[0] * 0.28;
        let button_y = toolbar.rect.center[1];
        let buttons = [
            UiButton::new(
                button_specs[0].id,
                button_specs[0].label,
                UiRect::new([first_x, button_y], button_size),
            ),
            UiButton::new(
                button_specs[1].id,
                button_specs[1].label,
                UiRect::new([second_x, button_y], button_size),
            ),
            UiButton::new(
                button_specs[2].id,
                button_specs[2].label,
                UiRect::new([third_x, button_y], button_size),
            ),
        ];

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

    pub fn button_label(&self, id: UiButtonId) -> &'static str {
        self.buttons
            .iter()
            .find(|button| button.id == id)
            .map(|button| button.label)
            .unwrap_or("unknown")
    }
}
