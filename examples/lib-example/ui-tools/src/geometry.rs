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
