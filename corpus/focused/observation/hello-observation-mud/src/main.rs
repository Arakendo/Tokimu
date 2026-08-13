//! Headless text-first consumer proof for Observation Shell Slice 7.
//!
//! The scenario owns room state, inventory, movement rules, and debug access.
//! Observation Shell only discovers, parses, routes, and projects bounded
//! application outcomes.

use observation_shell::{
    ApplicationCommandDescription, ApplicationCommandInvocation, ApplicationCommandResult,
    ApplicationMutationReceipt, ApplicationQueryField, ApplicationQueryResult, ObservationShell,
    ObservationSource,
};
use tokimu_core::{Diagnostics, World};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum Room {
    Atrium,
    Archive,
}

impl Room {
    fn name(self) -> &'static str {
        match self {
            Self::Atrium => "Atrium",
            Self::Archive => "Archive",
        }
    }

    fn description(self) -> &'static str {
        match self {
            Self::Atrium => "A sunlit atrium. An archive doorway stands to the north.",
            Self::Archive => "A quiet archive. The atrium is south through a heavy door.",
        }
    }

    fn exits(self) -> &'static str {
        match self {
            Self::Atrium => "north",
            Self::Archive => "south",
        }
    }
}

#[derive(Debug)]
struct MudScenario {
    room: Room,
    inventory: Vec<&'static str>,
    revision: u64,
    debug_authorized: bool,
}

impl MudScenario {
    fn new(debug_authorized: bool) -> Self {
        Self {
            room: Room::Atrium,
            inventory: vec!["brass key", "field notebook"],
            revision: 0,
            debug_authorized,
        }
    }

    fn dispatch(&mut self, invocation: &ApplicationCommandInvocation) -> ApplicationCommandResult {
        match (invocation.owner.as_str(), invocation.command.as_str()) {
            ("mud", "look") => self.query(
                "Room observation",
                vec![
                    field("room", self.room.name()),
                    field("description", self.room.description()),
                    field("exits", self.room.exits()),
                ],
            ),
            ("mud", "status") => self.query(
                "Player status",
                vec![
                    field("location", self.room.name()),
                    field("health", "ready"),
                    field("revision", self.revision.to_string()),
                ],
            ),
            ("mud", "inventory") => self.query(
                "Player inventory",
                vec![
                    field("items", self.inventory.join(", ")),
                    field("count", self.inventory.len().to_string()),
                ],
            ),
            ("mud", "go") => self.go(&invocation.arguments),
            ("mud", "debug") => self.debug(),
            _ => ApplicationCommandResult::Mutation {
                receipt: receipt(
                    false,
                    self.revision,
                    "MUD scenario does not recognize this command.",
                ),
            },
        }
    }

    fn query(
        &self,
        summary: impl Into<String>,
        fields: Vec<ApplicationQueryField>,
    ) -> ApplicationCommandResult {
        ApplicationCommandResult::Query {
            result: ApplicationQueryResult {
                summary: summary.into(),
                fields,
            },
        }
    }

    fn go(&mut self, arguments: &[String]) -> ApplicationCommandResult {
        let direction = arguments.first().map(String::as_str);
        let destination = match (self.room, direction) {
            (Room::Atrium, Some("north")) => Some(Room::Archive),
            (Room::Archive, Some("south")) => Some(Room::Atrium),
            _ => None,
        };

        match destination {
            Some(room) => {
                self.room = room;
                self.revision += 1;
                ApplicationCommandResult::Mutation {
                    receipt: receipt(
                        true,
                        self.revision,
                        format!("Player moved to {}.", self.room.name()),
                    ),
                }
            }
            None => ApplicationCommandResult::Mutation {
                receipt: receipt(
                    false,
                    self.revision,
                    format!(
                        "Cannot move {} from {}.",
                        direction.unwrap_or("there"),
                        self.room.name()
                    ),
                ),
            },
        }
    }

    fn debug(&self) -> ApplicationCommandResult {
        if !self.debug_authorized {
            return self.query(
                "Scenario-owned debug observation denied",
                vec![field(
                    "authorization",
                    "MUD debug observation requires scenario-granted inspect authority.",
                )],
            );
        }

        self.query(
            "Scenario-owned debug observation",
            vec![
                field("room_variant", format!("{:?}", self.room)),
                field("inventory_slots", self.inventory.len().to_string()),
                field("scenario_revision", self.revision.to_string()),
            ],
        )
    }
}

fn field(name: impl Into<String>, value: impl Into<String>) -> ApplicationQueryField {
    ApplicationQueryField::visible(name, value)
}

fn receipt(
    accepted: bool,
    revision: u64,
    message: impl Into<String>,
) -> ApplicationMutationReceipt {
    ApplicationMutationReceipt {
        accepted,
        applied_tick: None,
        resulting_revision: Some(revision),
        message: message.into(),
    }
}

fn source() -> ObservationSource {
    let world = World::default();
    let mut diagnostics = Diagnostics::default();
    diagnostics.record("hello-observation-mud source initialized");
    ObservationSource::from_world_and_diagnostics(&world, &diagnostics)
}

fn shell() -> ObservationShell {
    let mut shell = ObservationShell::default();
    for command in [
        ApplicationCommandDescription::query(
            "mud",
            "look",
            "Describe the current room.",
            Vec::new(),
        ),
        ApplicationCommandDescription::query(
            "mud",
            "status",
            "Describe player status.",
            Vec::new(),
        ),
        ApplicationCommandDescription::query(
            "mud",
            "inventory",
            "List player inventory.",
            Vec::new(),
        ),
        ApplicationCommandDescription::mutation(
            "mud",
            "go",
            "Request a scenario-owned movement transition.",
            vec!["direction".to_owned()],
        ),
        ApplicationCommandDescription::query(
            "mud",
            "debug",
            "Request a separately scenario-authorized debug observation.",
            Vec::new(),
        ),
    ] {
        shell
            .register_application_command(command)
            .expect("MUD command identities must be unique");
    }
    shell
}

fn main() {
    let source = source();
    let mut shell = shell();
    let mut scenario = MudScenario::new(false);
    let script = [
        "help",
        "application mud look",
        "application mud inventory",
        "application mud go north",
        "application mud status",
        "application mud look",
        "application mud debug",
        "application mud go north",
        "format json",
        "application mud status",
    ];

    for input in script {
        let record = shell.execute_with_application_handler(&source, input, |invocation| {
            scenario.dispatch(invocation)
        });
        println!("> {input}\n{}\n", record.projection);
    }

    // A separately configured scenario grants its own debug capability. The
    // shell sees the same owner-qualified command and does not grant access.
    let mut privileged_scenario = MudScenario::new(true);
    let record =
        shell.execute_with_application_handler(&source, "application mud debug", |invocation| {
            privileged_scenario.dispatch(invocation)
        });
    println!(
        "> application mud debug (scenario-authorized)\n{}",
        record.projection
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn movement_and_debug_authority_remain_scenario_owned() {
        let source = source();
        let mut shell = shell();
        let mut scenario = MudScenario::new(false);

        let moved = shell.execute_with_application_handler(
            &source,
            "application mud go north",
            |invocation| scenario.dispatch(invocation),
        );
        assert!(moved.projection.contains("Player moved to Archive."));

        let denied = shell.execute_with_application_handler(
            &source,
            "application mud debug",
            |invocation| scenario.dispatch(invocation),
        );
        assert!(denied
            .projection
            .contains("requires scenario-granted inspect authority"));
        assert_eq!(scenario.room, Room::Archive);
    }

    #[test]
    fn query_projection_can_change_without_changing_scenario_truth() {
        let source = source();
        let mut shell = shell();
        let mut scenario = MudScenario::new(false);

        let text =
            shell.execute_with_application_handler(&source, "application mud look", |invocation| {
                scenario.dispatch(invocation)
            });
        shell.execute(&source, "format json");
        let json =
            shell.execute_with_application_handler(&source, "application mud look", |invocation| {
                scenario.dispatch(invocation)
            });

        assert!(text.projection.contains("Atrium"));
        assert!(json.projection.contains("Atrium"));
        assert_eq!(scenario.room, Room::Atrium);
    }
}
