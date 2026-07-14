use crate::{plugin::Plugin, tick_fixed_updates, RunLoopDiagnostics, RunLoopSummary, RuntimeConfig};
use tokimu_core::{Diagnostics, FrameOutcome, Schedule, System, SystemRegistry, World};
use tokimu_input::{InputEvent, InputState};

#[derive(Debug)]
pub struct App {
    pub world: World,
    pub schedule: Schedule,
    pub diagnostics: Diagnostics,
    pub run_loop_diagnostics: RunLoopDiagnostics,
    pub systems: SystemRegistry,
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
            systems: SystemRegistry::default(),
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

    pub fn register_system(&mut self, system: impl System + 'static) -> &mut Self {
        self.systems.register(system);
        self
    }

    pub fn register_system_in_set(
        &mut self,
        set_name: impl Into<String>,
        system: impl System + 'static,
    ) -> &mut Self {
        self.systems.register_in_set(set_name, system);
        self
    }

    pub fn registered_system_count_in_phase(&self, phase: tokimu_core::Phase) -> usize {
        self.systems.count_in_phase(phase)
    }

    pub fn registered_system_count_in_set(&self, set_name: &str) -> usize {
        self.systems.count_in_set(set_name)
    }

    pub fn remove_registered_systems_in_phase(&mut self, phase: tokimu_core::Phase) -> usize {
        self.systems.remove_in_phase(phase)
    }

    pub fn remove_registered_systems_in_set(&mut self, set_name: &str) -> usize {
        self.systems.remove_in_set(set_name)
    }

    pub fn tick_registered_systems_in_set(
        &mut self,
        frame_delta_seconds: f64,
        set_name: &str,
    ) -> RunLoopSummary {
        let summary = self.tick(frame_delta_seconds);
        self.systems.run_set(set_name, &self.schedule, &mut self.world);
        summary
    }

    pub fn tick(&mut self, frame_delta_seconds: f64) -> RunLoopSummary {
        self.tick_with_fixed_updates(frame_delta_seconds, |_| {})
    }

    pub fn run_frame(&mut self, frame_delta_seconds: f64) -> FrameOutcome {
        let _ = self.tick(frame_delta_seconds);
        FrameOutcome::Continue
    }

    pub fn tick_with_fixed_updates<F>(
        &mut self,
        frame_delta_seconds: f64,
        mut on_fixed_update: F,
    ) -> RunLoopSummary
    where
        F: FnMut(&mut World),
    {
        self.elapsed_seconds += frame_delta_seconds;

        let summary = tick_fixed_updates(
            self.config,
            self.accumulator_seconds,
            frame_delta_seconds,
        );
        self.accumulator_seconds = summary.accumulator_seconds;
        self.run_loop_diagnostics.record_frame(summary);

        for _ in 0..summary.fixed_updates {
            on_fixed_update(&mut self.world);
        }

        summary
    }

    pub fn tick_with_systems(
        &mut self,
        frame_delta_seconds: f64,
        systems: &mut [&mut dyn System],
    ) -> RunLoopSummary {
        let summary = self.tick(frame_delta_seconds);
        self.schedule.run_systems(&mut self.world, systems);
        summary
    }

    pub fn tick_registered_systems(&mut self, frame_delta_seconds: f64) -> RunLoopSummary {
        let summary = self.tick(frame_delta_seconds);
        self.systems.run(&self.schedule, &mut self.world);
        summary
    }

    pub fn run_frames<I, F>(
        &mut self,
        frame_deltas: I,
        on_fixed_update: F,
    ) -> Option<RunLoopSummary>
    where
        I: IntoIterator<Item = f64>,
        F: FnMut(&mut World),
    {
        let mut frame_deltas = frame_deltas.into_iter();

        self.run_until(|| frame_deltas.next(), on_fixed_update)
    }

    pub fn run_until<N, F>(
        &mut self,
        mut next_frame_delta_seconds: N,
        mut on_fixed_update: F,
    ) -> Option<RunLoopSummary>
    where
        N: FnMut() -> Option<f64>,
        F: FnMut(&mut World),
    {
        let mut last_summary = None;

        while let Some(frame_delta_seconds) = next_frame_delta_seconds() {
            last_summary = Some(self.tick_with_fixed_updates(
                frame_delta_seconds,
                &mut on_fixed_update,
            ));
        }

        last_summary
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

    #[test]
    fn tick_with_fixed_updates_drives_callback() {
        let mut app = App::default();
        app.config.fixed_time_step_seconds = 0.25;
        app.config.max_fixed_steps_per_frame = 2;

        let mut calls = 0;
        let summary = app.tick_with_fixed_updates(1.0, |_| {
            calls += 1;
        });

        assert_eq!(summary.fixed_updates, 2);
        assert_eq!(calls, 2);
        assert_eq!(app.run_loop_diagnostics().frame_count(), 1);
    }

    #[test]
    fn run_frames_drives_each_frame() {
        let mut app = App::default();
        app.config.fixed_time_step_seconds = 0.25;
        app.config.max_fixed_steps_per_frame = 4;

        let mut fixed_update_calls = 0;
        let summary = app.run_frames([0.25, 0.5], |_| {
            fixed_update_calls += 1;
        });

        assert_eq!(fixed_update_calls, 3);
        assert_eq!(summary, Some(RunLoopSummary {
            frame_delta_seconds: 0.5,
            fixed_updates: 2,
            requested_fixed_updates: 2,
            hit_fixed_step_cap: false,
            accumulator_seconds: 0.0,
        }));
        assert_eq!(app.run_loop_diagnostics().frame_count(), 2);
    }

    #[test]
    fn run_frame_advances_headless_state() {
        let mut app = App::default();
        app.config.fixed_time_step_seconds = 0.25;
        app.config.max_fixed_steps_per_frame = 2;

        let outcome = app.run_frame(0.5);

        assert_eq!(outcome, FrameOutcome::Continue);
        assert_eq!(app.run_loop_diagnostics().frame_count(), 1);
        assert_eq!(app.run_loop_diagnostics().total_fixed_updates(), 2);
    }

    #[test]
    fn run_frames_returns_none_for_empty_input() {
        let mut app = App::default();

        let summary = app.run_frames(std::iter::empty::<f64>(), |_| {});

        assert_eq!(summary, None);
        assert_eq!(app.run_loop_diagnostics().frame_count(), 0);
    }

    #[test]
    fn run_until_drives_frames_until_source_ends() {
        let mut app = App::default();
        app.config.fixed_time_step_seconds = 0.25;
        app.config.max_fixed_steps_per_frame = 4;

        let mut frame_deltas = [0.25, 0.25, 0.5].into_iter();
        let mut fixed_update_calls = 0;
        let summary = app.run_until(|| frame_deltas.next(), |_| {
            fixed_update_calls += 1;
        });

        assert_eq!(fixed_update_calls, 4);
        assert_eq!(summary, Some(RunLoopSummary {
            frame_delta_seconds: 0.5,
            fixed_updates: 2,
            requested_fixed_updates: 2,
            hit_fixed_step_cap: false,
            accumulator_seconds: 0.0,
        }));
        assert_eq!(app.run_loop_diagnostics().frame_count(), 3);
    }

    #[test]
    fn tick_with_systems_runs_scheduled_systems() {
        struct InsertMarkerSystem;

        impl tokimu_core::System for InsertMarkerSystem {
            fn phase(&self) -> tokimu_core::Phase {
                tokimu_core::Phase::Update
            }

            fn run(&mut self, world: &mut tokimu_core::World) {
                world.spawn();
            }
        }

        let mut app = App::default();
        app.config.fixed_time_step_seconds = 0.25;
        app.config.max_fixed_steps_per_frame = 4;
        let mut system = InsertMarkerSystem;
        let mut systems: [&mut dyn System; 1] = [&mut system];

        let summary = app.tick_with_systems(0.25, &mut systems);

        assert_eq!(summary.fixed_updates, 1);
        assert_eq!(app.world.spawn(), tokimu_core::EntityId(1));
    }

    #[test]
    fn register_systems_and_tick_registered_systems_runs_phase_order() {
        struct MarkerSystem {
            phase: tokimu_core::Phase,
            label: &'static str,
            log: std::rc::Rc<std::cell::RefCell<Vec<&'static str>>>,
        }

        impl tokimu_core::System for MarkerSystem {
            fn phase(&self) -> tokimu_core::Phase {
                self.phase
            }

            fn run(&mut self, _world: &mut tokimu_core::World) {
                self.log.borrow_mut().push(self.label);
            }
        }

        let mut app = App::default();
        app.config.fixed_time_step_seconds = 0.25;
        app.config.max_fixed_steps_per_frame = 4;

        let log = std::rc::Rc::new(std::cell::RefCell::new(Vec::new()));
        app.register_system(MarkerSystem {
            phase: tokimu_core::Phase::Render,
            label: "render",
            log: log.clone(),
        });
        app.register_system(MarkerSystem {
            phase: tokimu_core::Phase::Startup,
            label: "startup",
            log: log.clone(),
        });
        app.register_system(MarkerSystem {
            phase: tokimu_core::Phase::Update,
            label: "update",
            log: log.clone(),
        });

        let summary = app.tick_registered_systems(0.25);

        assert_eq!(summary.fixed_updates, 1);
        assert_eq!(log.borrow().as_slice(), ["startup", "update", "render"]);
        assert_eq!(app.systems.len(), 3);
        assert!(!app.systems.is_empty());
        assert_eq!(app.registered_system_count_in_phase(tokimu_core::Phase::Startup), 1);
        assert_eq!(app.registered_system_count_in_phase(tokimu_core::Phase::Update), 1);
        assert_eq!(app.registered_system_count_in_phase(tokimu_core::Phase::Render), 1);
        assert_eq!(app.remove_registered_systems_in_phase(tokimu_core::Phase::Update), 1);
        assert_eq!(app.registered_system_count_in_phase(tokimu_core::Phase::Update), 0);
        assert_eq!(app.systems.len(), 2);
    }

    #[test]
    fn register_systems_in_set_and_tick_registered_systems_in_set_runs_phase_order() {
        struct MarkerSystem {
            phase: tokimu_core::Phase,
            label: &'static str,
            log: std::rc::Rc<std::cell::RefCell<Vec<&'static str>>>,
        }

        impl tokimu_core::System for MarkerSystem {
            fn phase(&self) -> tokimu_core::Phase {
                self.phase
            }

            fn run(&mut self, _world: &mut tokimu_core::World) {
                self.log.borrow_mut().push(self.label);
            }
        }

        let mut app = App::default();
        app.config.fixed_time_step_seconds = 0.25;
        app.config.max_fixed_steps_per_frame = 4;

        let log = std::rc::Rc::new(std::cell::RefCell::new(Vec::new()));
        app.register_system_in_set(
            "simulation",
            MarkerSystem {
                phase: tokimu_core::Phase::Startup,
                label: "startup",
                log: log.clone(),
            },
        );
        app.register_system_in_set(
            "simulation",
            MarkerSystem {
                phase: tokimu_core::Phase::Update,
                label: "update",
                log: log.clone(),
            },
        );
        app.register_system_in_set(
            "rendering",
            MarkerSystem {
                phase: tokimu_core::Phase::Render,
                label: "render",
                log: log.clone(),
            },
        );

        let summary = app.tick_registered_systems_in_set(0.25, "simulation");

        assert_eq!(summary.fixed_updates, 1);
        assert_eq!(log.borrow().as_slice(), ["startup", "update"]);
        assert_eq!(app.registered_system_count_in_set("simulation"), 2);
        assert_eq!(app.registered_system_count_in_set("rendering"), 1);
        assert_eq!(app.remove_registered_systems_in_set("rendering"), 1);
        assert_eq!(app.registered_system_count_in_set("rendering"), 0);
    }
}
