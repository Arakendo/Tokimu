use std::collections::BTreeSet;

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum MouseButton {
    Left,
    Middle,
    Right,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct MouseState {
    pub x: f32,
    pub y: f32,
    buttons: BTreeSet<MouseButton>,
}

impl MouseState {
    pub fn move_to(&mut self, x: f32, y: f32) {
        self.x = x;
        self.y = y;
    }

    pub fn press(&mut self, button: MouseButton) {
        self.buttons.insert(button);
    }

    pub fn release(&mut self, button: MouseButton) {
        self.buttons.remove(&button);
    }

    pub fn is_pressed(&self, button: MouseButton) -> bool {
        self.buttons.contains(&button)
    }
}
