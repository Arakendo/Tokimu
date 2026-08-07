use crate::{TuiAction, TuiActionOutcome, TuiDiagnostic};

/// View-local transcript state. It deliberately retains no application data.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TuiViewport {
    viewport_rows: u16,
    content_rows: u16,
    offset: u16,
    live_tail: bool,
}

impl TuiViewport {
    pub fn new(viewport_rows: u16, content_rows: u16) -> (Self, Vec<TuiDiagnostic>) {
        let mut viewport = Self {
            viewport_rows,
            content_rows,
            offset: 0,
            live_tail: true,
        };
        let diagnostics = viewport.follow_tail();
        (viewport, diagnostics)
    }

    pub const fn viewport_rows(self) -> u16 {
        self.viewport_rows
    }

    pub const fn content_rows(self) -> u16 {
        self.content_rows
    }

    pub const fn offset(self) -> u16 {
        self.offset
    }

    pub const fn live_tail(self) -> bool {
        self.live_tail
    }

    pub const fn max_offset(self) -> u16 {
        self.content_rows.saturating_sub(self.viewport_rows)
    }

    pub fn visible_rows(self) -> (usize, usize) {
        let start = usize::from(self.offset.min(self.content_rows));
        let end = start
            .saturating_add(usize::from(self.viewport_rows))
            .min(usize::from(self.content_rows));
        (start, end)
    }

    pub fn scroll_by(&mut self, delta: i16) -> Vec<TuiDiagnostic> {
        let requested = if delta.is_negative() {
            self.offset.saturating_sub(delta.unsigned_abs())
        } else {
            self.offset.saturating_add(delta as u16)
        };
        self.set_offset(requested)
    }

    /// Applies only navigation intents owned by this view-local viewport.
    ///
    /// Commands, activation, focus, and text editing remain outside viewport
    /// ownership and therefore return `Unhandled` without changing state.
    pub fn apply_action(&mut self, action: &TuiAction) -> TuiActionOutcome {
        let before = self.offset;
        match action {
            TuiAction::MovePrevious => {
                self.scroll_by(-1);
            }
            TuiAction::MoveNext => {
                self.scroll_by(1);
            }
            TuiAction::PagePrevious => {
                self.scroll_by(-(self.viewport_rows.min(i16::MAX as u16) as i16));
            }
            TuiAction::PageNext => {
                self.scroll_by(self.viewport_rows.min(i16::MAX as u16) as i16);
            }
            TuiAction::Home => {
                self.set_offset(0);
            }
            TuiAction::End => {
                self.set_offset(self.max_offset());
            }
            _ => return TuiActionOutcome::Unhandled,
        }
        if self.offset == before {
            TuiActionOutcome::Disabled
        } else {
            TuiActionOutcome::Applied
        }
    }

    pub fn set_offset(&mut self, requested_offset: u16) -> Vec<TuiDiagnostic> {
        let actual_offset = requested_offset.min(self.max_offset());
        self.offset = actual_offset;
        self.live_tail = self.offset == self.max_offset();
        if requested_offset == actual_offset {
            Vec::new()
        } else {
            vec![TuiDiagnostic::ViewportClamped {
                requested_offset,
                actual_offset,
            }]
        }
    }

    pub fn resize(&mut self, viewport_rows: u16, content_rows: u16) -> Vec<TuiDiagnostic> {
        self.viewport_rows = viewport_rows;
        self.content_rows = content_rows;
        let mut diagnostics = if viewport_rows == 0 {
            vec![TuiDiagnostic::EmptyViewport { viewport_rows }]
        } else {
            Vec::new()
        };
        if self.live_tail {
            diagnostics.extend(self.follow_tail());
        } else {
            diagnostics.extend(self.set_offset(self.offset));
        }
        diagnostics
    }

    pub fn append_rows(&mut self, rows: u16) -> Vec<TuiDiagnostic> {
        self.content_rows = self.content_rows.saturating_add(rows);
        if self.live_tail {
            self.follow_tail()
        } else {
            Vec::new()
        }
    }

    fn follow_tail(&mut self) -> Vec<TuiDiagnostic> {
        self.live_tail = true;
        self.set_offset(self.max_offset())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn live_tail_follows_appended_rows() {
        let (mut viewport, _) = TuiViewport::new(3, 5);
        assert_eq!(viewport.offset(), 2);
        viewport.append_rows(2);
        assert_eq!(viewport.offset(), 4);
        assert!(viewport.live_tail());
    }

    #[test]
    fn review_mode_does_not_jump_when_content_arrives() {
        let (mut viewport, _) = TuiViewport::new(3, 8);
        viewport.scroll_by(-2);
        assert_eq!(viewport.offset(), 3);
        assert!(!viewport.live_tail());
        viewport.append_rows(4);
        assert_eq!(viewport.offset(), 3);
        assert!(!viewport.live_tail());
    }

    #[test]
    fn resize_and_content_shrink_clamp_deterministically() {
        let (mut viewport, _) = TuiViewport::new(3, 10);
        viewport.scroll_by(-2);
        let diagnostics = viewport.resize(5, 6);
        assert_eq!(viewport.offset(), 1);
        assert!(matches!(
            diagnostics.last(),
            Some(TuiDiagnostic::ViewportClamped {
                requested_offset: 5,
                actual_offset: 1,
            })
        ));
    }

    #[test]
    fn scrolling_to_the_latest_row_restores_live_tail() {
        let (mut viewport, _) = TuiViewport::new(3, 8);
        viewport.scroll_by(-2);

        assert!(!viewport.live_tail());
        assert_eq!(viewport.offset(), 3);

        viewport.scroll_by(i16::MAX);

        assert!(viewport.live_tail());
        assert_eq!(viewport.offset(), viewport.max_offset());
    }

    #[test]
    fn normalized_navigation_changes_only_viewport_state() {
        let (mut viewport, _) = TuiViewport::new(3, 10);
        assert_eq!(viewport.offset(), 7);

        assert_eq!(
            viewport.apply_action(&TuiAction::MovePrevious),
            TuiActionOutcome::Applied
        );
        assert_eq!(viewport.offset(), 6);
        assert_eq!(
            viewport.apply_action(&TuiAction::Home),
            TuiActionOutcome::Applied
        );
        assert_eq!(viewport.offset(), 0);
        assert_eq!(
            viewport.apply_action(&TuiAction::MovePrevious),
            TuiActionOutcome::Disabled
        );
        assert_eq!(viewport.offset(), 0);
        assert_eq!(
            viewport.apply_action(&TuiAction::Activate),
            TuiActionOutcome::Unhandled
        );
        assert_eq!(viewport.offset(), 0);
    }
}
