use crate::UiRect;

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
            content_extent: content_extent.max(0.0),
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
        self.content_extent = content_extent.max(0.0);
        self.clamp_offset();
    }

    pub fn set_offset(&mut self, offset: f32) {
        self.offset = offset.clamp(0.0, self.max_offset());
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
        UiRect::new([rect.center[0], rect.center[1] + self.offset], rect.size)
    }

    pub fn visible_rect(&self, rect: UiRect) -> Option<UiRect> {
        self.content_rect(rect).intersection(self.viewport)
    }

    pub fn hit_test(&self, rect: UiRect, point: [f32; 2]) -> bool {
        self.visible_rect(rect)
            .is_some_and(|visible| visible.contains(point))
    }

    fn clamp_offset(&mut self) {
        self.offset = self.offset.clamp(0.0, self.max_offset());
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
}
