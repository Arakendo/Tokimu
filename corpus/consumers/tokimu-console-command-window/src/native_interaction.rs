//! Consumer-owned interaction state for the native command-window proof.
//!
//! Command meaning and provider execution remain outside this module. It owns
//! only the state a native host needs to edit and navigate one transcript.

use std::ops::Range;

use ui_tools::{UiTextInputOperation, UiTextInputState};

#[derive(Clone, Debug)]
pub struct ConsoleInteractionState {
    input: UiTextInputState,
    focused: bool,
    transcript: Vec<String>,
    transcript_limit: usize,
    scroll_offset: usize,
    command_history: Vec<String>,
    history_cursor: Option<usize>,
}

impl ConsoleInteractionState {
    pub fn new(
        initial_transcript: impl IntoIterator<Item = String>,
        transcript_limit: usize,
    ) -> Self {
        let mut state = Self {
            input: UiTextInputState::new(""),
            focused: true,
            transcript: Vec::new(),
            transcript_limit: transcript_limit.max(1),
            scroll_offset: 0,
            command_history: Vec::new(),
            history_cursor: None,
        };
        for line in initial_transcript {
            state.append_line(line);
        }
        state
    }

    pub const fn focused(&self) -> bool {
        self.focused
    }

    pub fn set_focused(&mut self, focused: bool) {
        self.focused = focused;
    }

    pub fn input(&self) -> &UiTextInputState {
        &self.input
    }

    pub fn edit(&mut self, operation: UiTextInputOperation) {
        self.input.apply(operation);
    }

    pub fn insert_text(&mut self, text: &str) {
        self.history_cursor = None;
        for character in text.chars() {
            self.input.apply(UiTextInputOperation::Insert(character));
        }
    }

    pub fn clear_prompt(&mut self) {
        self.input = UiTextInputState::new("");
        self.history_cursor = None;
    }

    pub fn take_submission(&mut self) -> Option<String> {
        let command = self.input.value().trim().to_owned();
        if command.is_empty() {
            return None;
        }
        if self.command_history.last() != Some(&command) {
            self.command_history.push(command.clone());
        }
        self.history_cursor = None;
        self.append_line(format!("> {command}"));
        self.input = UiTextInputState::new("");
        Some(command)
    }

    pub fn append_line(&mut self, line: impl Into<String>) {
        self.transcript.push(line.into());
        if self.transcript.len() > self.transcript_limit {
            self.transcript
                .drain(0..self.transcript.len() - self.transcript_limit);
        }
        self.scroll_offset = 0;
    }

    pub fn replace_transcript(&mut self, line: impl Into<String>) {
        self.transcript.clear();
        self.scroll_offset = 0;
        self.append_line(line);
    }

    pub fn transcript(&self) -> &[String] {
        &self.transcript
    }

    pub const fn scroll_offset(&self) -> usize {
        self.scroll_offset
    }

    pub fn scroll(&mut self, direction: i32, maximum: usize) {
        let next = if direction.is_positive() {
            self.scroll_offset.saturating_add(direction as usize)
        } else {
            self.scroll_offset
                .saturating_sub(direction.unsigned_abs() as usize)
        };
        self.scroll_offset = next.min(maximum);
    }

    pub fn visible_range(&self, line_count: usize, capacity: usize) -> Range<usize> {
        let end = line_count.saturating_sub(self.scroll_offset);
        end.saturating_sub(capacity)..end
    }

    pub fn recall_previous_command(&mut self) {
        let Some(last) = self.command_history.len().checked_sub(1) else {
            return;
        };
        let index = self
            .history_cursor
            .map_or(last, |current| current.saturating_sub(1));
        self.history_cursor = Some(index);
        self.input = UiTextInputState::new(&self.command_history[index]);
    }

    pub fn recall_next_command(&mut self) {
        let Some(current) = self.history_cursor else {
            return;
        };
        let next = current + 1;
        if next < self.command_history.len() {
            self.history_cursor = Some(next);
            self.input = UiTextInputState::new(&self.command_history[next]);
        } else {
            self.clear_prompt();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn complete_command_input_survives_editing_and_submission() {
        let mut state = ConsoleInteractionState::new([], 15);
        state.insert_text("DESCRIBE demo/message 2048?!");

        assert_eq!(
            state.take_submission().as_deref(),
            Some("DESCRIBE demo/message 2048?!")
        );
        assert_eq!(state.transcript(), &["> DESCRIBE demo/message 2048?!"]);
        assert!(state.input().value().is_empty());
    }

    #[test]
    fn history_recall_is_bounded_and_returns_to_an_empty_live_prompt() {
        let mut state = ConsoleInteractionState::new([], 15);
        for command in ["STATUS", "CHECK"] {
            state.insert_text(command);
            state.take_submission();
        }

        state.recall_previous_command();
        assert_eq!(state.input().value(), "CHECK");
        state.recall_previous_command();
        state.recall_previous_command();
        assert_eq!(state.input().value(), "STATUS");
        state.recall_next_command();
        assert_eq!(state.input().value(), "CHECK");
        state.recall_next_command();
        assert!(state.input().value().is_empty());
    }

    #[test]
    fn transcript_navigation_clamps_without_mutating_content() {
        let mut state = ConsoleInteractionState::new((0..15).map(|n| format!("line {n}")), 15);
        let original = state.transcript().to_vec();

        state.scroll(100, 7);
        assert_eq!(state.scroll_offset(), 7);
        assert_eq!(state.visible_range(15, 8), 0..8);
        state.scroll(-3, 7);
        assert_eq!(state.scroll_offset(), 4);
        state.scroll(-100, 7);
        assert_eq!(state.scroll_offset(), 0);
        assert_eq!(state.transcript(), original);
    }

    #[test]
    fn focus_and_prompt_state_are_independent_from_transcript_state() {
        let mut state = ConsoleInteractionState::new(["retained".to_owned()], 15);
        state.set_focused(false);
        state.insert_text("draft");
        state.clear_prompt();

        assert!(!state.focused());
        assert_eq!(state.transcript(), &["retained"]);
    }
}
