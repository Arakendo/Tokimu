use std::collections::BTreeSet;

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum KeyCode {
    Escape,
    Space,
    KeyA,
    KeyD,
    KeyS,
    KeyW,
    ArrowLeft,
    ArrowRight,
    ArrowUp,
    ArrowDown,
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct KeyboardState {
    pressed: BTreeSet<KeyCode>,
}

impl KeyboardState {
    pub fn press(&mut self, key: KeyCode) {
        self.pressed.insert(key);
    }

    pub fn release(&mut self, key: KeyCode) {
        self.pressed.remove(&key);
    }

    pub fn is_pressed(&self, key: KeyCode) -> bool {
        self.pressed.contains(&key)
    }
}
