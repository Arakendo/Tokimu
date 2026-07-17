#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum UiTextInputOperation {
    Insert(char),
    MoveLeft,
    MoveRight,
    DeleteBackward,
    DeleteForward,
    SelectAll,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct UiTextInputState {
    value: String,
    caret: usize,
    selection_anchor: Option<usize>,
}

impl UiTextInputState {
    pub fn new(value: impl Into<String>) -> Self {
        let value = value.into();
        let caret = value.chars().count();
        Self {
            value,
            caret,
            selection_anchor: None,
        }
    }

    pub fn value(&self) -> &str {
        &self.value
    }

    pub const fn caret(&self) -> usize {
        self.caret
    }

    pub const fn selection_anchor(&self) -> Option<usize> {
        self.selection_anchor
    }

    pub fn apply(&mut self, operation: UiTextInputOperation) {
        match operation {
            UiTextInputOperation::Insert(character) => self.insert(character),
            UiTextInputOperation::MoveLeft => self.move_left(),
            UiTextInputOperation::MoveRight => self.move_right(),
            UiTextInputOperation::DeleteBackward => self.delete_backward(),
            UiTextInputOperation::DeleteForward => self.delete_forward(),
            UiTextInputOperation::SelectAll => {
                self.selection_anchor = Some(0);
                self.caret = self.value.chars().count();
            }
        }
    }

    fn insert(&mut self, character: char) {
        self.replace_selection_if_any();
        let byte = self.byte_offset(self.caret);
        self.value.insert(byte, character);
        self.caret += 1;
    }

    fn move_left(&mut self) {
        self.caret = self.caret.saturating_sub(1);
        self.selection_anchor = None;
    }

    fn move_right(&mut self) {
        self.caret = (self.caret + 1).min(self.value.chars().count());
        self.selection_anchor = None;
    }

    fn delete_backward(&mut self) {
        if self.replace_selection_if_any() || self.caret == 0 {
            return;
        }
        let start = self.byte_offset(self.caret - 1);
        let end = self.byte_offset(self.caret);
        self.value.replace_range(start..end, "");
        self.caret -= 1;
    }

    fn delete_forward(&mut self) {
        if self.replace_selection_if_any() {
            return;
        }
        let count = self.value.chars().count();
        if self.caret >= count {
            return;
        }
        let start = self.byte_offset(self.caret);
        let end = self.byte_offset(self.caret + 1);
        self.value.replace_range(start..end, "");
    }

    fn replace_selection_if_any(&mut self) -> bool {
        let Some(anchor) = self.selection_anchor.take() else {
            return false;
        };
        let start = anchor.min(self.caret);
        let end = anchor.max(self.caret);
        let start_byte = self.byte_offset(start);
        let end_byte = self.byte_offset(end);
        self.value.replace_range(start_byte..end_byte, "");
        self.caret = start;
        true
    }

    fn byte_offset(&self, character_offset: usize) -> usize {
        self.value
            .char_indices()
            .nth(character_offset)
            .map_or(self.value.len(), |(index, _)| index)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn editing_operations_keep_caret_in_character_space() {
        let mut input = UiTextInputState::new("AB");
        input.apply(UiTextInputOperation::MoveLeft);
        input.apply(UiTextInputOperation::Insert('X'));
        input.apply(UiTextInputOperation::DeleteBackward);

        assert_eq!(input.value(), "AB");
        assert_eq!(input.caret(), 1);

        input.apply(UiTextInputOperation::DeleteForward);
        assert_eq!(input.value(), "A");
        assert_eq!(input.caret(), 1);
    }

    #[test]
    fn select_all_is_replaced_by_the_next_insert() {
        let mut input = UiTextInputState::new("hello");
        input.apply(UiTextInputOperation::SelectAll);
        input.apply(UiTextInputOperation::Insert('X'));

        assert_eq!(input.value(), "X");
        assert_eq!(input.caret(), 1);
        assert_eq!(input.selection_anchor(), None);
    }
}
