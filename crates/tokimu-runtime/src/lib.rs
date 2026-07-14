pub mod app;
pub mod config;
pub mod plugin;
pub mod output;
pub mod run_loop;
pub mod run_loop_diagnostics;

pub use app::App;
pub use config::{startup_output_channels, startup_output_verbosity, OutputVerbosity, RuntimeConfig};
pub use plugin::Plugin;
pub use output::{OutputChannel, OutputRouter, RepeatCoalescer, RepeatCoalescerUpdate};
pub use run_loop::{tick_fixed_updates, RunLoopSummary};
pub use run_loop_diagnostics::RunLoopDiagnostics;
pub use tokimu_core::FrameOutcome;

#[derive(Clone, Debug, PartialEq)]
pub struct FieldSprintState {
	pub motion_phase: f32,
	pub player_position: [f32; 2],
	pub target_position: [f32; 2],
	pub target_index: usize,
	pub score: u32,
	pub collection_flash: f32,
	pub paused: bool,
	pub palette_mode: bool,
	pub reverse_motion: bool,
	pub accent_rgba: [f32; 4],
}

impl Default for FieldSprintState {
	fn default() -> Self {
		Self {
			motion_phase: 0.0,
			player_position: [0.0, -0.65],
			target_position: [0.0, 0.0],
			target_index: 0,
			score: 0,
			collection_flash: 0.0,
			paused: false,
			palette_mode: false,
			reverse_motion: false,
			accent_rgba: [0.08, 0.18, 0.34, 1.0],
		}
	}
}

pub const FIELD_SPRINT_TARGET_POINTS: [[f32; 2]; 6] = [
	[-0.55, 0.40],
	[0.55, 0.28],
	[0.40, -0.42],
	[-0.15, -0.48],
	[-0.70, 0.02],
	[0.10, 0.52],
];

pub fn advance_field_sprint(
	state: &mut FieldSprintState,
	input: &tokimu_input::InputState,
	mouse_hold_active: bool,
	delta_seconds: f32,
) {
	if state.paused {
		return;
	}

	let motion_delta = delta_seconds * std::f32::consts::TAU * 0.25;
	if state.reverse_motion {
		state.motion_phase = (state.motion_phase - motion_delta).rem_euclid(std::f32::consts::TAU);
	} else {
		state.motion_phase = (state.motion_phase + motion_delta).rem_euclid(std::f32::consts::TAU);
	}

	state.accent_rgba = if state.palette_mode {
		[
			0.20 + state.motion_phase.cos() * 0.04,
			0.10 + state.motion_phase.sin() * 0.03,
			0.28 + state.motion_phase.cos() * 0.05,
			1.0,
		]
	} else {
		[
			0.08 + state.motion_phase.sin() * 0.03,
			0.18 + state.motion_phase.cos() * 0.02,
			0.34 + state.motion_phase.sin() * 0.04,
			1.0,
		]
	};

	if mouse_hold_active {
		state.accent_rgba = [
			(state.accent_rgba[0] + 0.10).min(1.0),
			(state.accent_rgba[1] + 0.08).min(1.0),
			(state.accent_rgba[2] + 0.12).min(1.0),
			1.0,
		];
	}

	if state.collection_flash > 0.0 {
		state.collection_flash = (state.collection_flash - delta_seconds * 1.8).max(0.0);
	}

	let horizontal = axis(
		input.keyboard.is_pressed(tokimu_input::KeyCode::KeyA)
			|| input.keyboard.is_pressed(tokimu_input::KeyCode::ArrowLeft),
		input.keyboard.is_pressed(tokimu_input::KeyCode::KeyD)
			|| input.keyboard.is_pressed(tokimu_input::KeyCode::ArrowRight),
	);
	let vertical = axis(
		input.keyboard.is_pressed(tokimu_input::KeyCode::KeyS)
			|| input.keyboard.is_pressed(tokimu_input::KeyCode::ArrowDown),
		input.keyboard.is_pressed(tokimu_input::KeyCode::KeyW)
			|| input.keyboard.is_pressed(tokimu_input::KeyCode::ArrowUp),
	);
	let speed = 0.85;

	state.player_position[0] = (state.player_position[0] + horizontal * speed * delta_seconds)
		.clamp(-0.82, 0.82);
	state.player_position[1] = (state.player_position[1] + vertical * speed * delta_seconds)
		.clamp(-0.70, 0.70);

	let dx = state.player_position[0] - state.target_position[0];
	let dy = state.player_position[1] - state.target_position[1];
	let collected = dx * dx + dy * dy < 0.028;
	if collected {
		state.score += 1;
		state.target_index = (state.target_index + 1) % FIELD_SPRINT_TARGET_POINTS.len();
		state.target_position = FIELD_SPRINT_TARGET_POINTS[state.target_index];
		state.collection_flash = 1.0;
	}
}

impl FieldSprintState {
	pub fn accent_color(&self) -> [f32; 4] {
		self.accent_rgba
	}
}

pub fn axis(negative: bool, positive: bool) -> f32 {
	match (negative, positive) {
		(true, false) => -1.0,
		(false, true) => 1.0,
		_ => 0.0,
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use tokimu_input::{InputState, KeyCode};

	fn input_with_keys(left: bool, right: bool, up: bool, down: bool) -> InputState {
		let mut input = InputState::default();
		if left {
			input.keyboard.press(KeyCode::KeyA);
		}
		if right {
			input.keyboard.press(KeyCode::KeyD);
		}
		if up {
			input.keyboard.press(KeyCode::KeyW);
		}
		if down {
			input.keyboard.press(KeyCode::KeyS);
		}
		input
	}

	#[test]
	fn field_sprint_sequence_matches_for_native_and_browser_inputs() {
		let documented_sequence = [
			input_with_keys(false, true, false, false),
			input_with_keys(false, true, false, false),
			input_with_keys(false, false, true, false),
			input_with_keys(false, false, true, false),
		];

		let mut native_state = FieldSprintState::default();
		native_state.target_position = FIELD_SPRINT_TARGET_POINTS[0];
		let mut browser_state = native_state.clone();

		for input in documented_sequence {
			advance_field_sprint(&mut native_state, &input, false, 1.0 / 60.0);
			advance_field_sprint(&mut browser_state, &input, false, 1.0 / 60.0);
		}

		assert_eq!(native_state, browser_state);
		assert!(native_state.motion_phase > 0.0);
		assert_eq!(native_state.score, 0);
	}

	#[test]
	fn mouse_hold_toggles_accent_without_changing_the_shared_motion_logic() {
		let input = InputState::default();
		let mut held_state = FieldSprintState::default();
		held_state.target_position = FIELD_SPRINT_TARGET_POINTS[0];
		let mut free_state = held_state.clone();

		advance_field_sprint(&mut held_state, &input, true, 1.0 / 60.0);
		advance_field_sprint(&mut free_state, &input, false, 1.0 / 60.0);

		assert_ne!(held_state.accent_rgba, free_state.accent_rgba);
		assert_eq!(held_state.player_position, free_state.player_position);
	}
}
