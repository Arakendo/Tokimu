use crate::{plugin::Plugin, tick_fixed_updates, RunLoopDiagnostics, RunLoopSummary, RuntimeConfig};
use tokimu_core::{Diagnostics, Schedule, World};
use tokimu_input::{InputEvent, InputState};

#[derive(Debug)]
pub struct App {
    pub world: World,
    pub schedule: Schedule,
    pub diagnostics: Diagnostics,
    pub run_loop_diagnostics: RunLoopDiagnostics,
    pub config: RuntimeConfig,
    pub input: InputState,
    accumulator_seconds: f64,
    elapsed_seconds: f64,
}

impl Default for App {
    fn default() -> Self {
        Self {
            world: World::default(),
            schedule: Schedule::default(),
            diagnostics: Diagnostics::default(),
            run_loop_diagnostics: RunLoopDiagnostics::default(),
            config: RuntimeConfig::default(),
            input: InputState::default(),
            accumulator_seconds: 0.0,
            elapsed_seconds: 0.0,
        }
    }
}

impl App {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add_plugin<P>(&mut self, plugin: &P) -> &mut Self
    where
        P: Plugin,
    {
        plugin.build(self);
        self
    }

    pub fn apply_input_event(&mut self, event: InputEvent) -> &mut Self {
        self.input.apply_event(event);
        self
    }

    pub fn tick(&mut self, frame_delta_seconds: f64) -> RunLoopSummary {
        self.elapsed_seconds += frame_delta_seconds;

        let summary = tick_fixed_updates(
            self.config,
            self.accumulator_seconds,
            frame_delta_seconds,
        );
        self.accumulator_seconds = summary.accumulator_seconds;
        self.run_loop_diagnostics.record_frame(summary);
        summary
    }

    pub fn elapsed_seconds(&self) -> f64 {
        self.elapsed_seconds
    }

    pub fn run_loop_diagnostics(&self) -> &RunLoopDiagnostics {
        &self.run_loop_diagnostics
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tick_updates_run_loop_diagnostics() {
        let mut app = App::default();
        app.config.fixed_time_step_seconds = 0.25;
        app.config.max_fixed_steps_per_frame = 2;

        let summary = app.tick(1.0);

        assert_eq!(summary.fixed_updates, 2);
        assert!(summary.hit_fixed_step_cap);

        let diagnostics = app.run_loop_diagnostics();
        assert_eq!(diagnostics.frame_count(), 1);
        assert_eq!(diagnostics.total_fixed_updates(), 2);
        assert_eq!(diagnostics.fixed_step_cap_hits(), 1);
        assert_eq!(diagnostics.last_frame_delta_seconds(), 1.0);
        assert_eq!(diagnostics.max_frame_delta_seconds(), 1.0);
        assert_eq!(diagnostics.last_summary(), Some(summary));
    }
}
