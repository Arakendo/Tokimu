#[derive(Clone, Copy, Debug, PartialEq)]
pub struct UiRect {
    pub center: [f32; 2],
    pub size: [f32; 2],
}

impl UiRect {
    pub fn new(center: [f32; 2], size: [f32; 2]) -> Self {
        Self { center, size }
    }

    pub fn contains(&self, point: [f32; 2]) -> bool {
        let half_width = self.size[0] * 0.5;
        let half_height = self.size[1] * 0.5;
        point[0] >= self.center[0] - half_width
            && point[0] <= self.center[0] + half_width
            && point[1] >= self.center[1] - half_height
            && point[1] <= self.center[1] + half_height
    }

    pub fn intersection(self, other: Self) -> Option<Self> {
        let left = (self.center[0] - self.size[0] * 0.5).max(other.center[0] - other.size[0] * 0.5);
        let right =
            (self.center[0] + self.size[0] * 0.5).min(other.center[0] + other.size[0] * 0.5);
        let bottom =
            (self.center[1] - self.size[1] * 0.5).max(other.center[1] - other.size[1] * 0.5);
        let top = (self.center[1] + self.size[1] * 0.5).min(other.center[1] + other.size[1] * 0.5);
        if left >= right || bottom >= top {
            return None;
        }

        Some(Self::new(
            [(left + right) * 0.5, (bottom + top) * 0.5],
            [right - left, top - bottom],
        ))
    }

    pub fn inset(self, amount: f32) -> Self {
        Self {
            center: self.center,
            size: [
                (self.size[0] - amount * 2.0).max(0.0),
                (self.size[1] - amount * 2.0).max(0.0),
            ],
        }
    }

    pub fn inset_by(self, insets: UiInsets) -> Self {
        let width = (self.size[0] - insets.left - insets.right).max(0.0);
        let height = (self.size[1] - insets.top - insets.bottom).max(0.0);
        let center_x = self.center[0] + (insets.right - insets.left) * 0.5;
        let center_y = self.center[1] + (insets.top - insets.bottom) * 0.5;

        Self {
            center: [center_x, center_y],
            size: [width, height],
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct UiHitRegion {
    pub rect: UiRect,
    pub clip: Option<UiRect>,
}

impl UiHitRegion {
    pub const fn new(rect: UiRect) -> Self {
        Self { rect, clip: None }
    }

    pub fn with_clip(mut self, clip: UiRect) -> Self {
        self.clip = Some(match self.clip {
            Some(existing) => match existing.intersection(clip) {
                Some(intersection) => intersection,
                None => UiRect::new([0.0, 0.0], [0.0, 0.0]),
            },
            None => clip,
        });
        self
    }

    pub fn visible_rect(&self) -> Option<UiRect> {
        self.clip
            .map_or(Some(self.rect), |clip| self.rect.intersection(clip))
    }

    pub fn contains(&self, point: [f32; 2]) -> bool {
        self.visible_rect()
            .is_some_and(|visible| visible.contains(point))
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct UiInsets {
    pub top: f32,
    pub right: f32,
    pub bottom: f32,
    pub left: f32,
}

impl UiInsets {
    pub const fn new(top: f32, right: f32, bottom: f32, left: f32) -> Self {
        Self {
            top,
            right,
            bottom,
            left,
        }
    }

    pub const fn uniform(value: f32) -> Self {
        Self::new(value, value, value, value)
    }

    pub const fn symmetric(horizontal: f32, vertical: f32) -> Self {
        Self::new(vertical, horizontal, vertical, horizontal)
    }
}

pub fn window_to_world(window_size: [f32; 2], cursor_position: [f32; 2]) -> [f32; 2] {
    let width = window_size[0].max(1.0);
    let height = window_size[1].max(1.0);
    let half_height = 1.0;
    let half_width = half_height * (width / height);
    let x = (cursor_position[0] / width) * (half_width * 2.0) - half_width;
    let y = half_height - (cursor_position[1] / height) * (half_height * 2.0);
    [x, y]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn asymmetric_insets_preserve_the_remaining_content_bounds() {
        let rect = UiRect::new([0.0, 0.0], [1.0, 0.8]);
        let inset = rect.inset_by(UiInsets::new(0.1, 0.2, 0.3, 0.4));

        assert!((inset.size[0] - 0.4).abs() < 0.00001);
        assert!((inset.size[1] - 0.4).abs() < 0.00001);
        assert!((inset.center[0] + 0.1).abs() < 0.00001);
        assert!((inset.center[1] + 0.1).abs() < 0.00001);
    }

    #[test]
    fn insets_clamp_overconstrained_rectangles_to_zero_size() {
        let rect = UiRect::new([0.0, 0.0], [0.2, 0.1]);

        assert_eq!(rect.inset_by(UiInsets::uniform(0.2)).size, [0.0, 0.0]);
    }

    #[test]
    fn window_corners_map_to_world_corners() {
        assert_eq!(window_to_world([100.0, 50.0], [0.0, 0.0]), [-2.0, 1.0]);
        assert_eq!(window_to_world([100.0, 50.0], [100.0, 50.0]), [2.0, -1.0]);
    }

    #[test]
    fn rectangle_intersection_clips_and_rejects_disjoint_bounds() {
        let rect = UiRect::new([0.0, 0.0], [2.0, 2.0]);
        let clip = UiRect::new([0.5, 0.0], [1.0, 0.8]);

        assert_eq!(
            rect.intersection(clip),
            Some(UiRect::new([0.5, 0.0], [1.0, 0.8]))
        );
        assert_eq!(rect.intersection(UiRect::new([3.0, 0.0], [1.0, 1.0])), None);
    }

    #[test]
    fn hit_regions_require_points_to_be_inside_the_clip() {
        let region = UiHitRegion::new(UiRect::new([0.0, 0.0], [2.0, 2.0]))
            .with_clip(UiRect::new([0.5, 0.0], [1.0, 0.8]));

        assert!(region.contains([0.5, 0.0]));
        assert!(!region.contains([-0.5, 0.0]));
        assert!(!region.contains([0.5, 0.5]));
    }
}
