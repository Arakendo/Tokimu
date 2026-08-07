#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct TuiExtent {
    pub columns: u16,
    pub rows: u16,
}

impl TuiExtent {
    pub const fn new(columns: u16, rows: u16) -> Self {
        Self { columns, rows }
    }

    pub const fn rect(self) -> TuiRect {
        TuiRect::new(0, 0, self.columns, self.rows)
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct TuiRect {
    pub x: u16,
    pub y: u16,
    pub width: u16,
    pub height: u16,
}

impl TuiRect {
    pub const fn new(x: u16, y: u16, width: u16, height: u16) -> Self {
        Self {
            x,
            y,
            width,
            height,
        }
    }

    pub const fn is_empty(self) -> bool {
        self.width == 0 || self.height == 0
    }

    pub const fn right(self) -> u16 {
        self.x.saturating_add(self.width)
    }

    pub const fn bottom(self) -> u16 {
        self.y.saturating_add(self.height)
    }

    pub fn contains(self, x: u16, y: u16) -> bool {
        x >= self.x && x < self.right() && y >= self.y && y < self.bottom()
    }

    pub fn inset(self, insets: TuiInsets) -> Self {
        let horizontal = insets.left.saturating_add(insets.right);
        let vertical = insets.top.saturating_add(insets.bottom);
        Self::new(
            self.x.saturating_add(insets.left.min(self.width)),
            self.y.saturating_add(insets.top.min(self.height)),
            self.width.saturating_sub(horizontal),
            self.height.saturating_sub(vertical),
        )
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct TuiInsets {
    pub top: u16,
    pub right: u16,
    pub bottom: u16,
    pub left: u16,
}

impl TuiInsets {
    pub const fn all(value: u16) -> Self {
        Self {
            top: value,
            right: value,
            bottom: value,
            left: value,
        }
    }

    pub const fn symmetric(horizontal: u16, vertical: u16) -> Self {
        Self {
            top: vertical,
            right: horizontal,
            bottom: vertical,
            left: horizontal,
        }
    }
}
