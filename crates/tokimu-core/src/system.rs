use crate::{Phase, Schedule, World};
use std::cmp::Reverse;
use std::collections::{HashMap, HashSet};
use std::fmt::{Debug, Formatter};

pub struct RegisteredSystem {
    phase: Phase,
    priority: i32,
    type_name: &'static str,
    set_name: Option<String>,
    system: Box<dyn System>,
}

impl RegisteredSystem {
    pub fn new(system: Box<dyn System>) -> Self {
        let phase = system.phase();
        let priority = system.priority();
        let type_name = system.type_name();
        Self {
            phase,
            priority,
            type_name,
            set_name: None,
            system,
        }
    }

    pub fn new_in_set(system: Box<dyn System>, set_name: impl Into<String>) -> Self {
        let phase = system.phase();
        let priority = system.priority();
        let type_name = system.type_name();
        Self {
            phase,
            priority,
            type_name,
            set_name: Some(set_name.into()),
            system,
        }
    }

    pub fn phase(&self) -> Phase {
        self.phase
    }

    pub fn priority(&self) -> i32 {
        self.priority
    }

    pub fn type_name(&self) -> &'static str {
        self.type_name
    }

    pub fn set_name(&self) -> Option<&str> {
        self.set_name.as_deref()
    }

    fn run(&mut self, world: &mut World) {
        self.system.run(world);
    }
}

impl Debug for RegisteredSystem {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("RegisteredSystem")
            .field("phase", &self.phase)
            .field("priority", &self.priority)
            .field("type_name", &self.type_name)
            .field("set_name", &self.set_name)
            .finish()
    }
}

#[derive(Debug, Default)]
pub struct SystemRegistry {
    systems: Vec<RegisteredSystem>,
}

impl SystemRegistry {
    pub fn register(&mut self, system: impl System + 'static) {
        self.systems.push(RegisteredSystem::new(Box::new(system)));
    }

    pub fn register_in_set(&mut self, set_name: impl Into<String>, system: impl System + 'static) {
        self.systems.push(RegisteredSystem::new_in_set(Box::new(system), set_name));
    }

    pub fn is_empty(&self) -> bool {
        self.systems.is_empty()
    }

    pub fn len(&self) -> usize {
        self.systems.len()
    }

    pub fn count_in_phase(&self, phase: Phase) -> usize {
        self.systems
            .iter()
            .filter(|system| system.phase() == phase)
            .count()
    }

    pub fn count_in_set(&self, set_name: &str) -> usize {
        self.systems
            .iter()
            .filter(|system| system.set_name() == Some(set_name))
            .count()
    }

    pub fn remove_in_phase(&mut self, phase: Phase) -> usize {
        let before = self.systems.len();
        self.systems.retain(|system| system.phase() != phase);
        before - self.systems.len()
    }

    pub fn remove_in_set(&mut self, set_name: &str) -> usize {
        let before = self.systems.len();
        self.systems.retain(|system| system.set_name() != Some(set_name));
        before - self.systems.len()
    }

    pub fn run(&mut self, schedule: &Schedule, world: &mut World) {
        for phase in schedule.phases() {
            let system_indices = self
                .systems
                .iter()
                .enumerate()
                .filter(|(_, system)| system.phase() == *phase)
                .map(|(index, _)| index)
                .collect::<Vec<_>>();

            for index in self.ordered_system_indices(system_indices) {
                self.systems[index].run(world);
            }
        }
    }

    pub fn run_set(&mut self, set_name: &str, schedule: &Schedule, world: &mut World) {
        for phase in schedule.phases() {
            let system_indices = self
                .systems
                .iter()
                .enumerate()
                .filter(|(_, system)| {
                    system.phase() == *phase && system.set_name() == Some(set_name)
                })
                .map(|(index, _)| index)
                .collect::<Vec<_>>();

            for index in self.ordered_system_indices(system_indices) {
                self.systems[index].run(world);
            }
        }
    }

    fn ordered_system_indices(&self, system_indices: Vec<usize>) -> Vec<usize> {
        if system_indices.len() <= 1 {
            return system_indices;
        }

        let mut indices_by_type: HashMap<&'static str, Vec<usize>> = HashMap::new();
        for &index in &system_indices {
            indices_by_type
                .entry(self.systems[index].type_name())
                .or_default()
                .push(index);
        }

        let mut indegree: HashMap<usize, usize> = system_indices.iter().map(|&index| (index, 0)).collect();
        let mut dependents: HashMap<usize, Vec<usize>> = HashMap::new();

        for &index in &system_indices {
            let mut seen_dependencies = HashSet::new();

            for dependency in self.systems[index].system.depends_on() {
                if !seen_dependencies.insert(dependency) {
                    continue;
                }

                if let Some(dependency_indices) = indices_by_type.get(dependency) {
                    for &dependency_index in dependency_indices {
                        dependents.entry(dependency_index).or_default().push(index);
                        if let Some(indegree) = indegree.get_mut(&index) {
                            *indegree += 1;
                        }
                    }
                }
            }
        }

        let mut ready = system_indices
            .iter()
            .copied()
            .filter(|index| indegree.get(index).copied().unwrap_or_default() == 0)
            .collect::<Vec<_>>();
        let mut ordered = Vec::with_capacity(system_indices.len());
        let mut emitted = HashSet::new();

        while !ready.is_empty() {
            ready.sort_by_key(|index| (Reverse(self.systems[*index].priority()), *index));
            let index = ready.remove(0);

            if !emitted.insert(index) {
                continue;
            }

            ordered.push(index);

            if let Some(next_indices) = dependents.get(&index) {
                for &dependent_index in next_indices {
                    if let Some(entry) = indegree.get_mut(&dependent_index) {
                        *entry -= 1;

                        if *entry == 0 {
                            ready.push(dependent_index);
                        }
                    }
                }
            }
        }

        if ordered.len() < system_indices.len() {
            let mut remaining = system_indices
                .into_iter()
                .filter(|index| !emitted.contains(index))
                .collect::<Vec<_>>();
            remaining.sort_by_key(|index| (Reverse(self.systems[*index].priority()), *index));
            ordered.extend(remaining);
        }

        ordered
    }
}

pub trait System {
    fn phase(&self) -> Phase {
        Phase::Update
    }

    fn priority(&self) -> i32 {
        0
    }

    fn depends_on(&self) -> &'static [&'static str] {
        &[]
    }

    fn type_name(&self) -> &'static str {
        std::any::type_name::<Self>()
    }

    fn run(&mut self, world: &mut World);
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::RefCell;
    use std::rc::Rc;

    struct PrioritySystem {
        phase: Phase,
        priority: i32,
        label: &'static str,
        log: Rc<RefCell<Vec<&'static str>>>,
    }

    impl System for PrioritySystem {
        fn phase(&self) -> Phase {
            self.phase
        }

        fn priority(&self) -> i32 {
            self.priority
        }

        fn run(&mut self, _world: &mut World) {
            self.log.borrow_mut().push(self.label);
        }
    }

    struct DependencySystem {
        phase: Phase,
        priority: i32,
        type_name: &'static str,
        label: &'static str,
        log: Rc<RefCell<Vec<&'static str>>>,
        depends_on: &'static [&'static str],
    }

    impl System for DependencySystem {
        fn phase(&self) -> Phase {
            self.phase
        }

        fn priority(&self) -> i32 {
            self.priority
        }

        fn type_name(&self) -> &'static str {
            self.type_name
        }

        fn depends_on(&self) -> &'static [&'static str] {
            self.depends_on
        }

        fn run(&mut self, _world: &mut World) {
            self.log.borrow_mut().push(self.label);
        }
    }

    #[test]
    fn runs_higher_priority_systems_first_within_a_phase() {
        let mut registry = SystemRegistry::default();
        let mut world = World::default();
        let schedule = Schedule::default();
        let log = Rc::new(RefCell::new(Vec::new()));

        registry.register(PrioritySystem {
            phase: Phase::Update,
            priority: 1,
            label: "low",
            log: Rc::clone(&log),
        });
        registry.register(PrioritySystem {
            phase: Phase::Update,
            priority: 5,
            label: "high",
            log: Rc::clone(&log),
        });
        registry.register(PrioritySystem {
            phase: Phase::Startup,
            priority: 10,
            label: "startup",
            log: Rc::clone(&log),
        });

        registry.run(&schedule, &mut world);

        assert_eq!(log.borrow().as_slice(), ["startup", "high", "low"]);
    }

    #[test]
    fn runs_named_sets_in_priority_order() {
        let mut registry = SystemRegistry::default();
        let mut world = World::default();
        let schedule = Schedule::default();
        let log = Rc::new(RefCell::new(Vec::new()));

        registry.register_in_set(
            "simulation",
            PrioritySystem {
                phase: Phase::Update,
                priority: 1,
                label: "low",
                log: Rc::clone(&log),
            },
        );
        registry.register_in_set(
            "simulation",
            PrioritySystem {
                phase: Phase::Update,
                priority: 3,
                label: "high",
                log: Rc::clone(&log),
            },
        );
        registry.register_in_set(
            "rendering",
            PrioritySystem {
                phase: Phase::Update,
                priority: 9,
                label: "ignored",
                log: Rc::clone(&log),
            },
        );

        registry.run_set("simulation", &schedule, &mut world);

        assert_eq!(log.borrow().as_slice(), ["high", "low"]);
    }

    #[test]
    fn runs_systems_after_declared_dependencies_within_a_phase() {
        let mut registry = SystemRegistry::default();
        let mut world = World::default();
        let schedule = Schedule::default();
        let log = Rc::new(RefCell::new(Vec::new()));

        const DEPENDS_ON_PARSE: &[&str] = &["ParseSystem"];
        const DEPENDS_ON_PARSE_AND_SIMULATE: &[&str] = &["ParseSystem", "SimulateSystem"];

        registry.register(DependencySystem {
            phase: Phase::Update,
            priority: 1,
            type_name: "FlushSystem",
            label: "flush",
            log: Rc::clone(&log),
            depends_on: DEPENDS_ON_PARSE_AND_SIMULATE,
        });
        registry.register(DependencySystem {
            phase: Phase::Update,
            priority: 4,
            type_name: "SimulateSystem",
            label: "simulate",
            log: Rc::clone(&log),
            depends_on: DEPENDS_ON_PARSE,
        });
        registry.register(DependencySystem {
            phase: Phase::Update,
            priority: 10,
            type_name: "ParseSystem",
            label: "parse",
            log: Rc::clone(&log),
            depends_on: &[],
        });

        registry.run(&schedule, &mut world);

        assert_eq!(log.borrow().as_slice(), ["parse", "simulate", "flush"]);
    }
}