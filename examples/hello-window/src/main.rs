mod output;

use output::{Channel, OutputRouter};
use std::sync::Arc;
use tokimu::{run_window_with_app, App, KeyCode, NativeWindow, PlatformEventHandler, PlatformInputEvent, PlatformResult, WindowConfig};
use tokimu::FrameOutcome;

fn main() -> PlatformResult<()> {
    run_window_with_app(WindowConfig::default(), HelloWindowApp::new())
}

#[derive(Default)]
struct HelloWindowApp {
    app: App,
    output: OutputRouter,
    last_move_intent: (f32, f32),
    last_cursor_position: (f32, f32),
    last_mouse_button_count: usize,
    last_fixed_update_count: u32,
    last_status_print_elapsed: f64,
    window: Option<Arc<NativeWindow>>,
}

impl PlatformEventHandler for HelloWindowApp {
    fn on_native_window_created(&mut self, window: Arc<NativeWindow>) -> PlatformResult<()> {
        self.window = Some(window);
        self.output.emit_one_shot(Channel::Lifecycle, "hello-window native window created");
        self.update_window_title();
        Ok(())
    }

    fn on_platform_event(&mut self, event: PlatformInputEvent) -> PlatformResult<()> {
        if let PlatformInputEvent::CloseRequested = event {
            let diagnostics = self.app.run_loop_diagnostics();
            self.output.flush();
            self.output.emit_one_shot(Channel::Lifecycle, format!(
                "shutdown summary frames={} fixed_updates={} cap_hits={} elapsed={:.2}s",
                diagnostics.frame_count(),
                diagnostics.total_fixed_updates(),
                diagnostics.fixed_step_cap_hits(),
                self.app.elapsed_seconds(),
            ));
            return Ok(());
        }

        let previous_cursor = self.cursor_position();
        let previous_mouse_button_count = self.mouse_button_count();

        if let Some(input_event) = event.as_input_event() {
            self.app.apply_input_event(input_event);
        }

        let move_intent = movement_intent(&self.app);
        if move_intent != self.last_move_intent
            || self.cursor_position() != previous_cursor
            || self.mouse_button_count() != previous_mouse_button_count
        {
            self.output.emit_on_change(
                Channel::Input,
                format!(
                    "move intent = ({:.1}, {:.1}) cursor = ({:.0}, {:.0}) mouse_buttons = {}",
                    move_intent.0,
                    move_intent.1,
                    self.cursor_position().0,
                    self.cursor_position().1,
                    self.mouse_button_count()
                ),
            );
            self.last_move_intent = move_intent;
            self.last_cursor_position = self.cursor_position();
            self.last_mouse_button_count = self.mouse_button_count();
            self.update_window_title();
        }

        Ok(())
    }

    fn on_frame(&mut self, delta_seconds: f64) -> PlatformResult<FrameOutcome> {
        let outcome = self.app.run_frame(delta_seconds);
        let summary = self
            .app
            .run_loop_diagnostics()
            .last_summary()
            .expect("hello-window frame delta should always be present");
        let diagnostics = self.app.run_loop_diagnostics();
        let elapsed = self.app.elapsed_seconds();

        if summary.hit_fixed_step_cap {
            self.output.emit_event(
                Channel::Warn,
                format!(
                    "frame overrun dt={:.4}s requested_fixed_updates={} ran_fixed_updates={} leftover={:.4}s cap_hits={} elapsed={:.2}s",
                summary.frame_delta_seconds,
                summary.requested_fixed_updates,
                summary.fixed_updates,
                summary.accumulator_seconds,
                diagnostics.fixed_step_cap_hits(),
                self.app.elapsed_seconds()
                ),
            );
        }

        if summary.fixed_updates != self.last_fixed_update_count
            || elapsed - self.last_status_print_elapsed >= 1.0
        {
            self.output.emit_sampled(
                Channel::Frame,
                elapsed,
                format!(
                    "frame dt={:.4}s fixed_updates={} requested_fixed_updates={} elapsed={:.2}s total_frames={} cap_hits={}",
                    summary.frame_delta_seconds,
                    summary.fixed_updates,
                    summary.requested_fixed_updates,
                    elapsed,
                    diagnostics.frame_count(),
                    diagnostics.fixed_step_cap_hits()
                ),
            );
            self.last_fixed_update_count = summary.fixed_updates;
            self.last_status_print_elapsed = elapsed;
            self.update_window_title();
        }

        Ok(outcome)
    }
}

impl HelloWindowApp {
    fn new() -> Self {
        Self {
            output: OutputRouter::with_startup_policy(),
            ..Self::default()
        }
    }

    fn update_window_title(&self) {
        if let Some(window) = self.window.as_ref() {
            let (x, y) = self.last_move_intent;
            let (cursor_x, cursor_y) = self.last_cursor_position;
            window.set_title(&format!(
                "Tokimu | this page is intentionally left blank | move=({:.0}, {:.0}) | cursor=({:.0}, {:.0}) | mouse_buttons={} | fixed={} | elapsed={:.1}s",
                x,
                y,
                cursor_x,
                cursor_y,
                self.last_mouse_button_count,
                self.last_fixed_update_count,
                self.app.elapsed_seconds()
            ));
        }
    }

    fn cursor_position(&self) -> (f32, f32) {
        (self.app.input.mouse.x, self.app.input.mouse.y)
    }

    fn mouse_button_count(&self) -> usize {
        [
            tokimu::MouseButton::Left,
            tokimu::MouseButton::Middle,
            tokimu::MouseButton::Right,
        ]
        .into_iter()
        .filter(|button| self.app.input.mouse.is_pressed(*button))
        .count()
    }
}

fn movement_intent(app: &App) -> (f32, f32) {
    let x = axis(
        app.input.keyboard.is_pressed(KeyCode::KeyA),
        app.input.keyboard.is_pressed(KeyCode::KeyD),
    );
    let y = axis(
        app.input.keyboard.is_pressed(KeyCode::KeyS),
        app.input.keyboard.is_pressed(KeyCode::KeyW),
    );
    (x, y)
}

fn axis(negative: bool, positive: bool) -> f32 {
    match (negative, positive) {
        (true, false) => -1.0,
        (false, true) => 1.0,
        _ => 0.0,
    }
}
