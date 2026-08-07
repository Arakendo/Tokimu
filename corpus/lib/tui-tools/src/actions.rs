//! Host-neutral actions and bounded focus mechanics.
//!
//! This module deliberately describes interaction intent rather than platform
//! key codes. Hosts translate keyboard, pointer, and wheel events before they
//! reach this boundary; applications retain ownership of command execution and
//! editable content.

/// An interaction intent normalized by a host adapter.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum TuiAction {
    FocusNext,
    FocusPrevious,
    MovePrevious,
    MoveNext,
    PagePrevious,
    PageNext,
    Home,
    End,
    Activate,
    Cancel,
    InsertText(String),
    Backspace,
}

/// The view-local outcome of applying one normalized action.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum TuiActionOutcome {
    Applied,
    Disabled,
    Unhandled,
}

/// Caller-qualified focus data for one interactive region.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TuiFocusItem {
    id: String,
    enabled: bool,
}

impl TuiFocusItem {
    pub fn new(id: impl Into<String>, enabled: bool) -> Self {
        Self {
            id: id.into(),
            enabled,
        }
    }

    pub fn id(&self) -> &str {
        &self.id
    }

    pub const fn enabled(&self) -> bool {
        self.enabled
    }
}

/// A bounded, deterministic focus path.
///
/// It owns no widget behavior and does not activate application commands. The
/// caller supplies the logical item identifiers and enabled state each time it
/// constructs or refreshes the path.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TuiFocusPath {
    items: Vec<TuiFocusItem>,
    selected: Option<usize>,
}

impl TuiFocusPath {
    pub fn new(items: impl IntoIterator<Item = TuiFocusItem>) -> Self {
        let items: Vec<_> = items.into_iter().collect();
        let selected = items.iter().position(TuiFocusItem::enabled);
        Self { items, selected }
    }

    pub fn items(&self) -> &[TuiFocusItem] {
        &self.items
    }

    pub fn selected(&self) -> Option<&TuiFocusItem> {
        self.selected.and_then(|index| self.items.get(index))
    }

    pub fn apply(&mut self, action: &TuiAction) -> TuiActionOutcome {
        match action {
            TuiAction::FocusNext => self.move_focus(1),
            TuiAction::FocusPrevious => self.move_focus(-1),
            // Activation and editable-content actions intentionally cross the
            // caller boundary unchanged. This path only owns focus mechanics.
            TuiAction::Activate
            | TuiAction::Cancel
            | TuiAction::InsertText(_)
            | TuiAction::Backspace => TuiActionOutcome::Unhandled,
            _ => TuiActionOutcome::Unhandled,
        }
    }

    fn move_focus(&mut self, direction: i8) -> TuiActionOutcome {
        if self.items.iter().all(|item| !item.enabled) {
            return TuiActionOutcome::Disabled;
        }

        let len = self.items.len();
        let Some(current) = self.selected else {
            self.selected = self.items.iter().position(TuiFocusItem::enabled);
            return TuiActionOutcome::Applied;
        };

        for step in 1..=len {
            let index = if direction.is_negative() {
                (current + len - (step % len)) % len
            } else {
                (current + step) % len
            };
            if self.items[index].enabled {
                if index == current {
                    return TuiActionOutcome::Disabled;
                }
                self.selected = Some(index);
                return TuiActionOutcome::Applied;
            }
        }
        TuiActionOutcome::Disabled
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn focus_traversal_skips_disabled_items_deterministically() {
        let mut path = TuiFocusPath::new([
            TuiFocusItem::new("open", true),
            TuiFocusItem::new("save", false),
            TuiFocusItem::new("close", true),
        ]);
        assert_eq!(path.selected().map(TuiFocusItem::id), Some("open"));

        assert_eq!(path.apply(&TuiAction::FocusNext), TuiActionOutcome::Applied);
        assert_eq!(path.selected().map(TuiFocusItem::id), Some("close"));
        assert_eq!(
            path.apply(&TuiAction::FocusPrevious),
            TuiActionOutcome::Applied
        );
        assert_eq!(path.selected().map(TuiFocusItem::id), Some("open"));
    }

    #[test]
    fn disabled_focus_actions_do_not_change_selection() {
        let mut path = TuiFocusPath::new([TuiFocusItem::new("only", true)]);
        assert_eq!(
            path.apply(&TuiAction::FocusNext),
            TuiActionOutcome::Disabled
        );
        assert_eq!(path.selected().map(TuiFocusItem::id), Some("only"));

        let mut empty = TuiFocusPath::new([TuiFocusItem::new("disabled", false)]);
        assert_eq!(
            empty.apply(&TuiAction::FocusNext),
            TuiActionOutcome::Disabled
        );
        assert_eq!(empty.selected(), None);
    }

    #[test]
    fn command_and_text_actions_remain_caller_owned() {
        let mut path = TuiFocusPath::new([TuiFocusItem::new("command", true)]);
        assert_eq!(
            path.apply(&TuiAction::Activate),
            TuiActionOutcome::Unhandled
        );
        assert_eq!(
            path.apply(&TuiAction::InsertText("status".to_owned())),
            TuiActionOutcome::Unhandled
        );
        assert_eq!(path.selected().map(TuiFocusItem::id), Some("command"));
    }
}
