use crate::{KeyCode, MouseButton};

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum InputEvent {
    KeyboardInput { key: KeyCode, pressed: bool },
    CursorMoved { x: f32, y: f32 },
    MouseInput { button: MouseButton, pressed: bool },
}