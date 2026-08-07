//! Corpus-local resource-browser and filter-form evidence.
//!
//! This is deliberately not a generic `tui-tools` widget. The consumer owns
//! resource names, filter text, selection, and activation semantics; the
//! library contributes only the bounded surface, focus path, and normalized
//! actions it has already earned.

use tui_tools::{
    StyleRole, Surface, TextAlignment, TuiAction, TuiActionOutcome, TuiExtent, TuiFocusItem,
    TuiFocusPath, TuiInsets, TuiRect,
};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ResourceBrowser {
    resources: Vec<&'static str>,
    selected: usize,
    filter: String,
    focus: TuiFocusPath,
}

impl ResourceBrowser {
    pub fn fixture() -> Self {
        Self {
            resources: vec!["Box.glb", "POLYLN01.cgm", "notes/readme.txt"],
            selected: 0,
            filter: String::new(),
            focus: TuiFocusPath::new([
                TuiFocusItem::new("resource-filter", true),
                TuiFocusItem::new("resource-list", true),
                TuiFocusItem::new("inspect-selected", true),
            ]),
        }
    }

    pub fn selected_resource(&self) -> &str {
        self.resources[self.selected]
    }

    pub fn filter(&self) -> &str {
        &self.filter
    }

    pub fn focused_region(&self) -> Option<&str> {
        self.focus.selected().map(TuiFocusItem::id)
    }

    pub fn apply(&mut self, action: &TuiAction) -> TuiActionOutcome {
        match action {
            TuiAction::FocusNext | TuiAction::FocusPrevious => self.focus.apply(action),
            TuiAction::MovePrevious if self.focused_region() == Some("resource-list") => {
                if self.selected == 0 {
                    TuiActionOutcome::Disabled
                } else {
                    self.selected -= 1;
                    TuiActionOutcome::Applied
                }
            }
            TuiAction::MoveNext if self.focused_region() == Some("resource-list") => {
                if self.selected + 1 >= self.resources.len() {
                    TuiActionOutcome::Disabled
                } else {
                    self.selected += 1;
                    TuiActionOutcome::Applied
                }
            }
            TuiAction::InsertText(value) if self.focused_region() == Some("resource-filter") => {
                self.filter.push_str(value);
                TuiActionOutcome::Applied
            }
            TuiAction::Backspace if self.focused_region() == Some("resource-filter") => {
                if self.filter.pop().is_some() {
                    TuiActionOutcome::Applied
                } else {
                    TuiActionOutcome::Disabled
                }
            }
            // Activation remains a caller-owned request.
            TuiAction::Activate => TuiActionOutcome::Unhandled,
            _ => TuiActionOutcome::Unhandled,
        }
    }

    pub fn render(&self, extent: TuiExtent) -> Surface {
        let mut surface = Surface::new(extent);
        let frame = extent.rect();
        surface.draw_frame(frame, StyleRole::Frame);
        let content = frame.inset(TuiInsets::all(1));

        surface.write_line(
            content,
            0,
            "RESOURCE BROWSER",
            TextAlignment::Start,
            StyleRole::Heading,
        );
        surface.write_line(
            content,
            1,
            &format!("filter: {}", self.filter),
            TextAlignment::Start,
            if self.focused_region() == Some("resource-filter") {
                StyleRole::Accent
            } else {
                StyleRole::Muted
            },
        );

        let rows = content.height.saturating_sub(4);
        let list = TuiRect::new(content.x, content.y.saturating_add(3), content.width, rows);
        for (row, resource) in self
            .resources
            .iter()
            .take(usize::from(list.height))
            .enumerate()
        {
            let is_selected = row == self.selected;
            surface.write_line(
                list,
                row as u16,
                &format!("{} {}", if is_selected { '>' } else { ' ' }, resource),
                TextAlignment::Start,
                if is_selected && self.focused_region() == Some("resource-list") {
                    StyleRole::Warning
                } else if is_selected {
                    StyleRole::Value
                } else {
                    StyleRole::Plain
                },
            );
        }

        if content.height >= 1 {
            surface.write_line(
                content,
                content.height - 1,
                &format!("selected: {}", self.selected_resource()),
                TextAlignment::Start,
                if self.focused_region() == Some("inspect-selected") {
                    StyleRole::Accent
                } else {
                    StyleRole::Muted
                },
            );
        }

        surface
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn resource_browser_keeps_data_and_activation_with_its_consumer() {
        let mut browser = ResourceBrowser::fixture();
        assert_eq!(browser.focused_region(), Some("resource-filter"));
        assert_eq!(
            browser.apply(&TuiAction::InsertText("box".to_owned())),
            TuiActionOutcome::Applied
        );
        assert_eq!(browser.filter(), "box");

        assert_eq!(
            browser.apply(&TuiAction::FocusNext),
            TuiActionOutcome::Applied
        );
        assert_eq!(browser.focused_region(), Some("resource-list"));
        assert_eq!(
            browser.apply(&TuiAction::MoveNext),
            TuiActionOutcome::Applied
        );
        assert_eq!(browser.selected_resource(), "POLYLN01.cgm");

        assert_eq!(
            browser.apply(&TuiAction::FocusNext),
            TuiActionOutcome::Applied
        );
        assert_eq!(browser.focused_region(), Some("inspect-selected"));
        assert_eq!(
            browser.apply(&TuiAction::Activate),
            TuiActionOutcome::Unhandled
        );
        assert_eq!(browser.selected_resource(), "POLYLN01.cgm");
    }

    #[test]
    fn resource_browser_rendering_is_bounded_and_repeatable() {
        let mut browser = ResourceBrowser::fixture();
        browser.apply(&TuiAction::FocusNext);
        browser.apply(&TuiAction::MoveNext);

        let first = browser.render(TuiExtent::new(36, 9));
        let second = browser.render(TuiExtent::new(36, 9));
        assert_eq!(first, second);
        assert_eq!(first.cells().len(), 36 * 9);
        assert!(first.to_plain_text().contains("POLYLN01.cgm"));
    }
}
