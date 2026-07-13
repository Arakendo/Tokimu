use crate::{System, World};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Phase {
    Startup,
    PreUpdate,
    FixedUpdate,
    Update,
    PostUpdate,
    RenderPrepare,
    Render,
    PostRender,
    Shutdown,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Schedule {
    phases: Vec<Phase>,
}

impl Default for Schedule {
    fn default() -> Self {
        Self {
            phases: vec![
                Phase::Startup,
                Phase::PreUpdate,
                Phase::FixedUpdate,
                Phase::Update,
                Phase::PostUpdate,
                Phase::RenderPrepare,
                Phase::Render,
                Phase::PostRender,
                Phase::Shutdown,
            ],
        }
    }
}

impl Schedule {
    pub fn with_phases(phases: Vec<Phase>) -> Self {
        Self { phases }
    }

    pub fn phases(&self) -> &[Phase] {
        &self.phases
    }

    pub fn set_phases(&mut self, phases: Vec<Phase>) {
        self.phases = phases;
    }

    pub fn push_phase(&mut self, phase: Phase) {
        self.phases.push(phase);
    }

    pub fn run_systems(&self, world: &mut World, systems: &mut [&mut dyn System]) {
        for phase in &self.phases {
            for system in systems.iter_mut() {
                if system.phase() == *phase {
                    system.run(world);
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::World;

    struct RecordingSystem {
        phase: Phase,
        label: &'static str,
        log: Vec<&'static str>,
    }

    impl RecordingSystem {
        fn new(phase: Phase, label: &'static str) -> Self {
            Self {
                phase,
                label,
                log: Vec::new(),
            }
        }
    }

    impl System for RecordingSystem {
        fn phase(&self) -> Phase {
            self.phase
        }

        fn run(&mut self, _world: &mut World) {
            self.log.push(self.label);
        }
    }

    #[test]
    fn runs_systems_in_phase_order() {
        let schedule = Schedule::default();
        let mut world = World::default();

        let mut startup = RecordingSystem::new(Phase::Startup, "startup");
        let mut update = RecordingSystem::new(Phase::Update, "update");
        let mut render = RecordingSystem::new(Phase::Render, "render");

        let mut systems: [&mut dyn System; 3] = [&mut render, &mut update, &mut startup];
        schedule.run_systems(&mut world, &mut systems);

        assert_eq!(startup.log, vec!["startup"]);
        assert_eq!(update.log, vec!["update"]);
        assert_eq!(render.log, vec!["render"]);
    }

    #[test]
    fn ignores_systems_with_unmatched_phases() {
        let schedule = Schedule::default();
        let mut world = World::default();

        let mut render = RecordingSystem::new(Phase::Render, "render");
        let mut systems: [&mut dyn System; 1] = [&mut render];

        schedule.run_systems(&mut world, &mut systems);

        assert_eq!(render.log, vec!["render"]);
    }

    #[test]
    fn counts_systems_by_phase() {
        struct StartupSystem;
        struct UpdateSystem;

        impl System for StartupSystem {
            fn phase(&self) -> Phase {
                Phase::Startup
            }

            fn run(&mut self, _world: &mut World) {}
        }

        impl System for UpdateSystem {
            fn phase(&self) -> Phase {
                Phase::Update
            }

            fn run(&mut self, _world: &mut World) {}
        }

        let mut registry = crate::SystemRegistry::default();
        registry.register(StartupSystem);
        registry.register(UpdateSystem);
        registry.register(UpdateSystem);

        assert_eq!(registry.count_in_phase(Phase::Startup), 1);
        assert_eq!(registry.count_in_phase(Phase::Update), 2);
        assert_eq!(registry.count_in_phase(Phase::Render), 0);
    }

    #[test]
    fn removes_systems_in_phase() {
        struct StartupSystem;
        struct UpdateSystem;

        impl System for StartupSystem {
            fn phase(&self) -> Phase {
                Phase::Startup
            }

            fn run(&mut self, _world: &mut World) {}
        }

        impl System for UpdateSystem {
            fn phase(&self) -> Phase {
                Phase::Update
            }

            fn run(&mut self, _world: &mut World) {}
        }

        let mut registry = crate::SystemRegistry::default();
        registry.register(StartupSystem);
        registry.register(UpdateSystem);
        registry.register(UpdateSystem);

        assert_eq!(registry.remove_in_phase(Phase::Update), 2);
        assert_eq!(registry.count_in_phase(Phase::Startup), 1);
        assert_eq!(registry.count_in_phase(Phase::Update), 0);
        assert_eq!(registry.len(), 1);
    }

    #[test]
    fn counts_systems_in_set() {
        struct StartupSystem;
        struct UpdateSystem;

        impl System for StartupSystem {
            fn phase(&self) -> Phase {
                Phase::Startup
            }

            fn run(&mut self, _world: &mut World) {}
        }

        impl System for UpdateSystem {
            fn phase(&self) -> Phase {
                Phase::Update
            }

            fn run(&mut self, _world: &mut World) {}
        }

        let mut registry = crate::SystemRegistry::default();
        registry.register_in_set("simulation", StartupSystem);
        registry.register_in_set("simulation", UpdateSystem);
        registry.register(UpdateSystem);

        assert_eq!(registry.count_in_set("simulation"), 2);
        assert_eq!(registry.count_in_set("render"), 0);
    }

    #[test]
    fn removes_systems_in_set() {
        struct StartupSystem;
        struct UpdateSystem;

        impl System for StartupSystem {
            fn phase(&self) -> Phase {
                Phase::Startup
            }

            fn run(&mut self, _world: &mut World) {}
        }

        impl System for UpdateSystem {
            fn phase(&self) -> Phase {
                Phase::Update
            }

            fn run(&mut self, _world: &mut World) {}
        }

        let mut registry = crate::SystemRegistry::default();
        registry.register_in_set("simulation", StartupSystem);
        registry.register_in_set("simulation", UpdateSystem);
        registry.register(UpdateSystem);

        assert_eq!(registry.remove_in_set("simulation"), 2);
        assert_eq!(registry.count_in_set("simulation"), 0);
        assert_eq!(registry.len(), 1);
    }

    #[test]
    fn runs_systems_in_set_and_phase_order() {
        let schedule = Schedule::default();
        let mut world = World::default();

        struct SetRecordingSystem {
            phase: Phase,
            label: &'static str,
            log: std::rc::Rc<std::cell::RefCell<Vec<&'static str>>>,
        }

        impl System for SetRecordingSystem {
            fn phase(&self) -> Phase {
                self.phase
            }

            fn run(&mut self, _world: &mut World) {
                self.log.borrow_mut().push(self.label);
            }
        }

        let log = std::rc::Rc::new(std::cell::RefCell::new(Vec::new()));

        let mut registry = crate::SystemRegistry::default();
        registry.register_in_set(
            "simulation",
            SetRecordingSystem {
                phase: Phase::Startup,
                label: "startup",
                log: log.clone(),
            },
        );
        registry.register_in_set(
            "simulation",
            SetRecordingSystem {
                phase: Phase::Update,
                label: "update",
                log: log.clone(),
            },
        );
        registry.register_in_set(
            "rendering",
            SetRecordingSystem {
                phase: Phase::Render,
                label: "render",
                log: log.clone(),
            },
        );

        registry.run_set("simulation", &schedule, &mut world);

        assert_eq!(log.borrow().as_slice(), ["startup", "update"]);
    }

    #[test]
    fn runs_systems_in_custom_phase_order() {
        let schedule = Schedule::with_phases(vec![Phase::Update, Phase::Startup]);
        let mut world = World::default();

        let mut startup = RecordingSystem::new(Phase::Startup, "startup");
        let mut update = RecordingSystem::new(Phase::Update, "update");

        let mut systems: [&mut dyn System; 2] = [&mut startup, &mut update];
        schedule.run_systems(&mut world, &mut systems);

        assert_eq!(startup.log, vec!["startup"]);
        assert_eq!(update.log, vec!["update"]);
    }

    #[test]
    fn mutates_phase_order() {
        let mut schedule = Schedule::default();
        schedule.set_phases(vec![Phase::Render, Phase::Update]);
        schedule.push_phase(Phase::Shutdown);

        assert_eq!(
            schedule.phases(),
            &[Phase::Render, Phase::Update, Phase::Shutdown]
        );
    }

}
