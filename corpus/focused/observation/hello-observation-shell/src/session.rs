//! Shared fixture and dispatch seam for Observation Shell host evidence.
//!
//! Host adapters retain their own input and terminal mechanics. They all call
//! this session so command parsing, history, and projection remain owned by
//! `ObservationShell`.

use observation_shell::{ObservationShell, ObservationSource};
use tokimu_core::{Diagnostics, World};

#[derive(Debug)]
struct Follows;

pub(crate) struct ShellFixture {
    world: World,
    diagnostics: Diagnostics,
    pub(crate) shell: ObservationShell,
}

impl ShellFixture {
    pub(crate) fn new() -> Self {
        let mut world = World::default();
        let observer = world.spawn();
        let target = world.spawn();
        world.add_relationship::<Follows>(observer, target);

        let mut diagnostics = Diagnostics::default();
        diagnostics.record("hello-observation-shell fixture initialized");

        Self {
            world,
            diagnostics,
            shell: ObservationShell::default(),
        }
    }

    pub(crate) fn execute_line(&mut self, input: &str) -> Option<String> {
        let input = input.trim();
        if input.is_empty() {
            return None;
        }

        let source = ObservationSource::from_world_and_diagnostics(&self.world, &self.diagnostics);
        Some(self.shell.execute(&source, input).projection)
    }
}

#[cfg(test)]
mod tests {
    use super::ShellFixture;

    #[test]
    fn independent_host_fixtures_preserve_the_same_command_trace() {
        let commands = ["help", "inspect world", "list entities"];
        let mut first = ShellFixture::new();
        let mut second = ShellFixture::new();

        let first_trace = commands
            .iter()
            .filter_map(|command| first.execute_line(command))
            .collect::<Vec<_>>();
        let second_trace = commands
            .iter()
            .filter_map(|command| second.execute_line(command))
            .collect::<Vec<_>>();

        assert_eq!(first_trace, second_trace);
        assert_eq!(first.shell.history().len(), commands.len());
    }
}
