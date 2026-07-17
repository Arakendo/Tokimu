use crate::{KeyCode, MouseButton};

#[derive(Clone, Debug, PartialEq)]
pub enum InputEvent {
    KeyboardInput { key: KeyCode, pressed: bool },
    TextInput(String),
    CursorMoved { x: f32, y: f32 },
    MouseInput { button: MouseButton, pressed: bool },
}
