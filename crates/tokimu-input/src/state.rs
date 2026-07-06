use crate::{KeyboardState, MouseState};

#[derive(Clone, Debug, Default, PartialEq)]
pub struct InputState {
    pub keyboard: KeyboardState,
    pub mouse: MouseState,
}
