use std::collections::BTreeMap;

use crate::{ControllerId, ControllerState, InputEvent, KeyboardState, MouseState};

#[derive(Clone, Debug, Default, PartialEq)]
pub struct InputState {
    pub keyboard: KeyboardState,
    pub mouse: MouseState,
    controllers: BTreeMap<ControllerId, ControllerState>,
}

impl InputState {
    pub fn apply_event(&mut self, event: InputEvent) {
        match event {
            InputEvent::KeyboardInput { key, pressed } => {
                if pressed {
                    self.keyboard.press(key);
                } else {
                    self.keyboard.release(key);
                }
            }
            InputEvent::CursorMoved { x, y } => self.mouse.move_to(x, y),
            InputEvent::MouseInput { button, pressed } => {
                if pressed {
                    self.mouse.press(button);
                } else {
                    self.mouse.release(button);
                }
            }
        }
    }

    pub fn controller(&self, controller: ControllerId) -> Option<&ControllerState> {
        self.controllers.get(&controller)
    }

    pub fn controller_mut(&mut self, controller: ControllerId) -> &mut ControllerState {
        self.controllers.entry(controller).or_default()
    }
}

#[cfg(test)]
mod tests {
    use super::InputState;
    use crate::{InputEvent, KeyCode, MouseButton};

    #[test]
    fn apply_event_updates_keyboard_and_mouse_state() {
        let mut input = InputState::default();

        input.apply_event(InputEvent::KeyboardInput {
            key: KeyCode::KeyW,
            pressed: true,
        });
        input.apply_event(InputEvent::CursorMoved { x: 12.0, y: 24.0 });
        input.apply_event(InputEvent::MouseInput {
            button: MouseButton::Left,
            pressed: true,
        });

        assert!(input.keyboard.is_pressed(KeyCode::KeyW));
        assert_eq!((input.mouse.x, input.mouse.y), (12.0, 24.0));
        assert!(input.mouse.is_pressed(MouseButton::Left));
    }
}
