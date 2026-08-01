use crate::UiRect;

/// Visibility of translated content within a scroll viewport.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum UiScrollVisibility {
    Hidden,
    Partial,
    Full,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct UiVerticalScroll {
    viewport: UiRect,
    content_extent: f32,
    offset: f32,
}

impl UiVerticalScroll {
    pub fn new(viewport: UiRect, content_extent: f32) -> Self {
        Self {
            viewport,
            content_extent: finite_non_negative(content_extent),
            offset: 0.0,
        }
    }

    pub fn viewport(&self) -> UiRect {
        self.viewport
    }

    pub fn content_extent(&self) -> f32 {
        self.content_extent
    }

    pub fn max_offset(&self) -> f32 {
        (self.content_extent - self.viewport.size[1]).max(0.0)
    }

    pub fn offset(&self) -> f32 {
        self.offset
    }

    pub fn set_viewport(&mut self, viewport: UiRect) {
        self.viewport = viewport;
        self.clamp_offset();
    }

    pub fn set_content_extent(&mut self, content_extent: f32) {
        self.content_extent = finite_non_negative(content_extent);
        self.clamp_offset();
    }

    pub fn set_offset(&mut self, offset: f32) {
        self.offset = finite_non_negative(offset).clamp(0.0, self.max_offset());
    }

    pub fn scroll_by(&mut self, delta: f32) {
        self.set_offset(self.offset + delta);
    }

    pub fn scroll_to_start(&mut self) {
        self.offset = 0.0;
    }

    pub fn scroll_to_end(&mut self) {
        self.offset = self.max_offset();
    }

    pub fn content_rect(&self, rect: UiRect) -> UiRect {
        rect.translated(self.content_translation())
    }

    /// Translation consumed by `UiNodeSpec::with_child_translation`.
    pub fn content_translation(&self) -> [f32; 2] {
        [0.0, self.offset]
    }

    pub fn visible_rect(&self, rect: UiRect) -> Option<UiRect> {
        self.content_rect(rect).intersection(self.viewport)
    }

    pub fn hit_test(&self, rect: UiRect, point: [f32; 2]) -> bool {
        self.visible_rect(rect)
            .is_some_and(|visible| visible.contains(point))
    }

    pub fn visibility(&self, rect: UiRect) -> UiScrollVisibility {
        let translated = self.content_rect(rect);
        match translated.intersection(self.viewport) {
            None => UiScrollVisibility::Hidden,
            Some(visible) if visible == translated => UiScrollVisibility::Full,
            Some(_) => UiScrollVisibility::Partial,
        }
    }

    /// Moves the nearest content edge into view and preserves a valid offset.
    ///
    /// Oversized content is aligned to the nearest viewport edge rather than
    /// claiming that the complete rectangle can become visible.
    pub fn ensure_visible(&mut self, rect: UiRect) {
        let translated = self.content_rect(rect);
        let viewport_top = self.viewport.center[1] + self.viewport.size[1] * 0.5;
        let viewport_bottom = self.viewport.center[1] - self.viewport.size[1] * 0.5;
        let content_top = translated.center[1] + translated.size[1] * 0.5;
        let content_bottom = translated.center[1] - translated.size[1] * 0.5;

        if content_top > viewport_top {
            self.set_offset(self.offset - (content_top - viewport_top));
        } else if content_bottom < viewport_bottom {
            self.set_offset(self.offset + (viewport_bottom - content_bottom));
        }
    }

    fn clamp_offset(&mut self) {
        self.offset = self.offset.clamp(0.0, self.max_offset());
    }
}

fn finite_non_negative(value: f32) -> f32 {
    if value.is_finite() {
        value.max(0.0)
    } else {
        0.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn scroll() -> UiVerticalScroll {
        UiVerticalScroll::new(UiRect::new([0.0, 0.0], [2.0, 2.0]), 6.0)
    }

    #[test]
    fn scroll_offset_is_clamped_to_content_bounds() {
        let mut scroll = scroll();

        scroll.scroll_by(3.0);
        assert_eq!(scroll.offset(), 3.0);
        scroll.scroll_by(10.0);
        assert_eq!(scroll.offset(), 4.0);
        scroll.scroll_by(-10.0);
        assert_eq!(scroll.offset(), 0.0);
    }

    #[test]
    fn content_rects_move_upward_on_screen_as_offset_increases() {
        let mut scroll = scroll();
        let content = UiRect::new([0.0, 0.5], [1.0, 0.5]);

        assert_eq!(scroll.content_rect(content), content);
        scroll.set_offset(1.0);
        assert_eq!(scroll.content_rect(content).center, [0.0, 1.5]);
    }

    #[test]
    fn visibility_and_hit_testing_use_the_viewport_clip() {
        let mut scroll = scroll();
        let content = UiRect::new([0.0, 1.25], [1.0, 1.0]);

        assert_eq!(
            scroll.visible_rect(content),
            Some(UiRect::new([0.0, 0.875], [1.0, 0.25]))
        );
        assert!(scroll.hit_test(content, [0.0, 0.9]));
        assert!(!scroll.hit_test(content, [0.0, -1.1]));

        scroll.set_offset(2.0);
        assert!(!scroll.hit_test(content, [0.0, 0.9]));
    }

    #[test]
    fn resizing_viewport_and_content_reclamps_offset() {
        let mut scroll = scroll();
        scroll.scroll_to_end();
        assert_eq!(scroll.offset(), 4.0);

        scroll.set_content_extent(3.0);
        assert_eq!(scroll.offset(), 1.0);
        scroll.set_viewport(UiRect::new([0.0, 0.0], [2.0, 4.0]));
        assert_eq!(scroll.offset(), 0.0);
    }

    #[test]
    fn visibility_distinguishes_hidden_partial_and_full_content() {
        let scroll = scroll();

        assert_eq!(
            scroll.visibility(UiRect::new([0.0, 0.0], [1.0, 1.0])),
            UiScrollVisibility::Full
        );
        assert_eq!(
            scroll.visibility(UiRect::new([0.0, 1.25], [1.0, 1.0])),
            UiScrollVisibility::Partial
        );
        assert_eq!(
            scroll.visibility(UiRect::new([0.0, 2.0], [1.0, 1.0])),
            UiScrollVisibility::Hidden
        );
    }

    #[test]
    fn ensure_visible_moves_the_nearest_edge_into_the_viewport() {
        let mut scroll = scroll();
        let below_viewport = UiRect::new([0.0, -2.0], [1.0, 0.5]);

        scroll.ensure_visible(below_viewport);

        assert_eq!(scroll.offset(), 1.25);
        assert_eq!(scroll.visibility(below_viewport), UiScrollVisibility::Full);
    }

    #[test]
    fn non_finite_scroll_inputs_are_bounded() {
        let mut scroll = scroll();

        scroll.set_offset(f32::NAN);
        scroll.set_content_extent(f32::INFINITY);

        assert_eq!(scroll.offset(), 0.0);
        assert_eq!(scroll.content_extent(), 0.0);
    }
}
