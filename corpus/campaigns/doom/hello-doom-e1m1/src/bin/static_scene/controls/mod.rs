//! Native inspection-control interpretation for the E1M1 evidence app.
//!
//! This module converts retained input state into application movement intent.
//! It does not own collision, floor transitions, source specials, camera
//! mutation, or a general Tokimu input contract.

use hello_doom_e1m1::{observer_direction, observer_right};
use tokimu_core::math::Vec3;
use tokimu_input::{InputState, KeyCode};

const NAVIGATION_KEYS: [KeyCode; 8] = [
    KeyCode::KeyW,
    KeyCode::KeyA,
    KeyCode::KeyS,
    KeyCode::KeyD,
    KeyCode::Space,
    KeyCode::ControlLeft,
    KeyCode::ShiftLeft,
    KeyCode::ShiftRight,
];

/// Releases only the keys interpreted by the corpus inspection camera.
pub(super) fn release_navigation_keys(input: &mut InputState) {
    for key in NAVIGATION_KEYS {
        input.keyboard.release(key);
    }
}

/// Computes one frame's requested observer displacement.
///
/// A/D are expressed relative to the observer's screen-right basis. Vertical
/// movement is admitted only for the explicit noclip inspection mode. The
/// result is normalized before speed is applied so diagonals are not faster.
pub(super) fn inspection_movement_delta(
    input: &InputState,
    yaw: f32,
    noclip: bool,
    delta_seconds: f64,
    walk_speed: f32,
    run_speed_multiplier: f32,
) -> Option<Vec3> {
    let mut direction = Vec3::ZERO;
    let forward = observer_direction(yaw, 0.0);
    let right = observer_right(forward);

    if input.keyboard.is_pressed(KeyCode::KeyW) {
        direction += forward;
    }
    if input.keyboard.is_pressed(KeyCode::KeyS) {
        direction -= forward;
    }
    if input.keyboard.is_pressed(KeyCode::KeyD) {
        direction += right;
    }
    if input.keyboard.is_pressed(KeyCode::KeyA) {
        direction -= right;
    }
    if noclip && input.keyboard.is_pressed(KeyCode::Space) {
        direction += Vec3::Y;
    }
    if noclip && input.keyboard.is_pressed(KeyCode::ControlLeft) {
        direction -= Vec3::Y;
    }
    if direction.length_squared() == 0.0 {
        return None;
    }

    let running = input.keyboard.is_pressed(KeyCode::ShiftLeft)
        || input.keyboard.is_pressed(KeyCode::ShiftRight);
    let speed = if running {
        walk_speed * run_speed_multiplier
    } else {
        walk_speed
    };
    Some(direction.normalize() * (speed * delta_seconds as f32))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn delta(input: &InputState, noclip: bool) -> Option<Vec3> {
        inspection_movement_delta(input, 0.0, noclip, 0.5, 10.0, 2.0)
    }

    #[test]
    fn strafe_keys_follow_observer_right_and_left() {
        let forward = observer_direction(0.0, 0.0);
        let right = observer_right(forward);

        let mut input = InputState::default();
        input.keyboard.press(KeyCode::KeyD);
        assert_eq!(delta(&input, false), Some(right * 5.0));

        input.keyboard.release(KeyCode::KeyD);
        input.keyboard.press(KeyCode::KeyA);
        assert_eq!(delta(&input, false), Some(right * -5.0));
    }

    #[test]
    fn run_multiplier_applies_after_diagonal_normalization() {
        let mut input = InputState::default();
        input.keyboard.press(KeyCode::KeyW);
        input.keyboard.press(KeyCode::KeyD);
        input.keyboard.press(KeyCode::ShiftRight);

        let movement = delta(&input, false).expect("movement");
        assert!((movement.length() - 10.0).abs() < 1.0e-5);
    }

    #[test]
    fn vertical_controls_require_noclip() {
        let mut input = InputState::default();
        input.keyboard.press(KeyCode::Space);
        assert_eq!(delta(&input, false), None);
        assert_eq!(delta(&input, true), Some(Vec3::Y * 5.0));

        input.keyboard.release(KeyCode::Space);
        input.keyboard.press(KeyCode::ControlLeft);
        assert_eq!(delta(&input, true), Some(Vec3::Y * -5.0));
    }

    #[test]
    fn release_navigation_keys_clears_all_interpreted_controls() {
        let mut input = InputState::default();
        for key in NAVIGATION_KEYS {
            input.keyboard.press(key);
        }

        release_navigation_keys(&mut input);

        for key in NAVIGATION_KEYS {
            assert!(!input.keyboard.is_pressed(key));
        }
    }
}
