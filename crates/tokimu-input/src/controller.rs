use std::collections::{BTreeMap, BTreeSet};

pub type ControllerId = u32;

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum ControllerButton {
    South,
    East,
    West,
    North,
    Start,
    Select,
    LeftShoulder,
    RightShoulder,
    LeftStick,
    RightStick,
    DPadUp,
    DPadDown,
    DPadLeft,
    DPadRight,
}

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum ControllerAxis {
    LeftStickX,
    LeftStickY,
    RightStickX,
    RightStickY,
    LeftTrigger,
    RightTrigger,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct ControllerState {
    pressed: BTreeSet<ControllerButton>,
    axes: BTreeMap<ControllerAxis, f32>,
}

impl ControllerState {
    pub fn press(&mut self, button: ControllerButton) {
        self.pressed.insert(button);
    }

    pub fn release(&mut self, button: ControllerButton) {
        self.pressed.remove(&button);
    }

    pub fn is_pressed(&self, button: ControllerButton) -> bool {
        self.pressed.contains(&button)
    }

    pub fn set_axis(&mut self, axis: ControllerAxis, value: f32) {
        self.axes.insert(axis, value);
    }

    pub fn axis(&self, axis: ControllerAxis) -> f32 {
        self.axes.get(&axis).copied().unwrap_or(0.0)
    }
}

#[cfg(test)]
mod tests {
    use super::{ControllerAxis, ControllerButton, ControllerState};

    #[test]
    fn controller_tracks_buttons_and_axes() {
        let mut controller = ControllerState::default();

        controller.press(ControllerButton::South);
        controller.set_axis(ControllerAxis::LeftStickX, 0.75);

        assert!(controller.is_pressed(ControllerButton::South));
        assert_eq!(controller.axis(ControllerAxis::LeftStickX), 0.75);
    }
}