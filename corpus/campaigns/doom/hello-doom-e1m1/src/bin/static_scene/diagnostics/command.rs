//! Parsing for the bounded, corpus-local debug console vocabulary.
//!
//! Execution remains in the application composition because commands inspect
//! and mutate several independent corpus subjects. This module owns only the
//! normalization and classification needed to keep that dispatch explicit.

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum DebugCommand {
    Help,
    Clear,
    Camera,
    Status,
    Catalog,
    Inventory,
    Warnings,
    Timings,
    Collision,
    Look(String),
    Scan(String),
    Use(String),
    Noclip(NoclipAction),
    Unsupported(String),
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum NoclipAction {
    Toggle,
    On,
    Off,
}

pub(crate) fn parse_debug_command(command: &str) -> DebugCommand {
    let normalized = command.trim().to_ascii_lowercase();
    match normalized.as_str() {
        "help" => DebugCommand::Help,
        "clear" => DebugCommand::Clear,
        "camera" => DebugCommand::Camera,
        "status" => DebugCommand::Status,
        "catalog" => DebugCommand::Catalog,
        "inventory" => DebugCommand::Inventory,
        "warnings" => DebugCommand::Warnings,
        "timings" => DebugCommand::Timings,
        "collision" => DebugCommand::Collision,
        command
            if command == "look"
                || command == "inspect"
                || command.starts_with("look ")
                || command.starts_with("inspect ") =>
        {
            DebugCommand::Look(command.to_owned())
        }
        command if command == "scan" || command.starts_with("scan ") => {
            DebugCommand::Scan(command.to_owned())
        }
        command if command.starts_with("use") => DebugCommand::Use(command.to_owned()),
        "noclip" | "noclip toggle" => DebugCommand::Noclip(NoclipAction::Toggle),
        "noclip on" => DebugCommand::Noclip(NoclipAction::On),
        "noclip off" => DebugCommand::Noclip(NoclipAction::Off),
        _ => DebugCommand::Unsupported(normalized),
    }
}

#[cfg(test)]
mod tests {
    use super::{parse_debug_command, DebugCommand, NoclipAction};

    #[test]
    fn aliases_and_case_normalize_without_execution_policy() {
        assert_eq!(
            parse_debug_command(" INSPECT "),
            DebugCommand::Look("inspect".to_owned())
        );
        assert_eq!(
            parse_debug_command("LOOK PIXEL 900 600"),
            DebugCommand::Look("look pixel 900 600".to_owned())
        );
        assert_eq!(
            parse_debug_command("SCAN 48 30"),
            DebugCommand::Scan("scan 48 30".to_owned())
        );
        assert_eq!(
            parse_debug_command("NOCLIP On"),
            DebugCommand::Noclip(NoclipAction::On)
        );
        assert_eq!(
            parse_debug_command("USE 151"),
            DebugCommand::Use("use 151".to_owned())
        );
        assert_eq!(parse_debug_command("CATALOG"), DebugCommand::Catalog);
        assert_eq!(parse_debug_command("INVENTORY"), DebugCommand::Inventory);
        assert_eq!(parse_debug_command("WARNINGS"), DebugCommand::Warnings);
        assert_eq!(parse_debug_command("TIMINGS"), DebugCommand::Timings);
    }

    #[test]
    fn unknown_commands_retain_normalized_diagnostic_identity() {
        assert_eq!(
            parse_debug_command("  Teleport  "),
            DebugCommand::Unsupported("teleport".to_owned())
        );
    }
}
