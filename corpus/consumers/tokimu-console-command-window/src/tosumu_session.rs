//! Corpus-local adapter for Tosumu's provisional TQL JSON process boundary.
//!
//! This module deliberately treats command text and JSON as opaque contracts.
//! It does not link Tosumu crates or reproduce TQL parsing and dispatch.

use std::{
    env,
    path::{Path, PathBuf},
    process::{Command, Output},
    time::{SystemTime, UNIX_EPOCH},
};

use serde::Serialize;
use serde_json::Value;

const SCHEMA_VERSION: u64 = 1;
const MAX_PROJECTED_LINES: usize = 24;
const MAX_COMMAND_BYTES: usize = 4 * 1024;

pub struct TosumuSession {
    cli: PathBuf,
    database: String,
}

#[derive(Debug, Serialize)]
pub struct SessionEvidence {
    pub schema_version: u64,
    pub fixture: &'static str,
    pub commands: Vec<CommandEvidence>,
}

#[derive(Debug, Serialize)]
pub struct CommandEvidence {
    pub input: String,
    pub lines: Vec<String>,
    pub outcome: CommandOutcome,
    pub envelope: Option<Value>,
}

/// The corpus distinguishes a provider-declared command failure from a
/// process or JSON-contract failure at the CLI boundary.
#[derive(Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum CommandOutcome {
    Success,
    ProviderFailure,
    ProcessFailure,
    ContractFailure,
}

impl SessionEvidence {
    pub fn transcript(&self) -> Vec<String> {
        self.commands
            .iter()
            .flat_map(|command| {
                std::iter::once(format!("> {}", command.input)).chain(command.lines.iter().cloned())
            })
            .collect()
    }

    pub fn success_count(&self) -> usize {
        self.commands
            .iter()
            .filter(|command| command.outcome == CommandOutcome::Success)
            .count()
    }

    pub fn failure_count(&self) -> usize {
        self.commands.len() - self.success_count()
    }
}

impl TosumuSession {
    pub fn open_fixture() -> Result<Self, String> {
        let cli = find_cli()?;
        let database = fixture_path()?;
        run_success(&cli, ["init", database.as_str()])?;
        run_success(
            &cli,
            [
                "put",
                database.as_str(),
                "demo/message",
                "hello from tokimu",
            ],
        )?;
        Ok(Self { cli, database })
    }

    pub fn execute(&self, command: &str) -> Vec<String> {
        self.execute_evidence(command).lines
    }

    pub fn run_script(&self, commands: &[&str]) -> SessionEvidence {
        SessionEvidence {
            schema_version: SCHEMA_VERSION,
            fixture: "disposable-demo-message",
            commands: commands
                .iter()
                .map(|command| self.execute_evidence(command))
                .collect(),
        }
    }

    fn execute_evidence(&self, command: &str) -> CommandEvidence {
        if command.len() > MAX_COMMAND_BYTES {
            return CommandEvidence {
                input: format!("[input omitted: {} bytes]", command.len()),
                lines: vec![format!(
                    "[tokimu corpus input rejected] command exceeded the {MAX_COMMAND_BYTES}-byte adapter limit"
                )],
                outcome: CommandOutcome::ContractFailure,
                envelope: None,
            };
        }
        let output = match Command::new(&self.cli)
            .args(["tql", self.database.as_str(), command, "--json"])
            .output()
        {
            Ok(output) => output,
            Err(error) => {
                return CommandEvidence {
                    input: command.to_owned(),
                    lines: vec![format!("[tosumu process error] {error}")],
                    outcome: CommandOutcome::ProcessFailure,
                    envelope: None,
                };
            }
        };

        match decode_envelope(&output) {
            Ok(envelope) => CommandEvidence {
                input: command.to_owned(),
                lines: bounded_lines(envelope_lines(&envelope)),
                outcome: if output.status.success() {
                    CommandOutcome::Success
                } else {
                    CommandOutcome::ProviderFailure
                },
                envelope: Some(envelope),
            },
            Err(error) => CommandEvidence {
                input: command.to_owned(),
                lines: vec![format!("[tosumu contract error] {error}")],
                outcome: CommandOutcome::ContractFailure,
                envelope: None,
            },
        }
    }
}

fn find_cli() -> Result<PathBuf, String> {
    if let Some(path) = env::var_os("TOSUMU_CLI_BIN") {
        let path = PathBuf::from(path);
        return path
            .is_file()
            .then_some(path.clone())
            .ok_or_else(|| format!("TOSUMU_CLI_BIN does not name a file: {}", path.display()));
    }

    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .ancestors()
        .nth(3)
        .ok_or_else(|| "could not locate the Tokimu workspace root".to_owned())?;
    let executable = if cfg!(windows) {
        "tosumu.exe"
    } else {
        "tosumu"
    };
    let path = root
        .join("third-party")
        .join("tosumu")
        .join("target")
        .join("debug")
        .join(executable);
    path.is_file().then_some(path.clone()).ok_or_else(|| {
        format!(
            "Tosumu CLI was not built at {}. Build `tosumu-cli` or set TOSUMU_CLI_BIN.",
            path.display()
        )
    })
}

fn fixture_path() -> Result<String, String> {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .ancestors()
        .nth(3)
        .ok_or_else(|| "could not locate the Tokimu workspace root".to_owned())?;
    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|error| format!("read fixture clock: {error}"))?
        .as_nanos();
    let directory = root
        .join("target")
        .join("tokimu-console-command-window")
        .join(format!("session-{}-{nonce}", std::process::id()));
    std::fs::create_dir_all(&directory)
        .map_err(|error| format!("create session directory {}: {error}", directory.display()))?;
    Ok(directory.join("fixture.tsm").display().to_string())
}

fn run_success<const N: usize>(cli: &Path, arguments: [&str; N]) -> Result<(), String> {
    let output = Command::new(cli)
        .args(arguments)
        .output()
        .map_err(|error| format!("start Tosumu fixture command: {error}"))?;
    output
        .status
        .success()
        .then_some(())
        .ok_or_else(|| format_process_failure("fixture setup", &output))
}

fn decode_envelope(output: &Output) -> Result<Value, String> {
    let envelope: Value = serde_json::from_slice(&output.stdout).map_err(|error| {
        format!(
            "decode TQL JSON: {error}; {}",
            format_process_failure("TQL", output)
        )
    })?;
    let version = envelope["schema_version"]
        .as_u64()
        .ok_or_else(|| "TQL JSON omitted schema_version".to_owned())?;
    if version != SCHEMA_VERSION {
        return Err(format!(
            "TQL JSON used schema v{version}, expected v{SCHEMA_VERSION}"
        ));
    }
    Ok(envelope)
}

fn envelope_lines(envelope: &Value) -> Vec<String> {
    let mut lines = vec![format!(
        "[tosumu / tql v{}] {}",
        envelope["schema_version"].as_u64().unwrap_or_default(),
        envelope["command"].as_str().unwrap_or("UNKNOWN")
    )];
    let payload = envelope
        .get("outcome")
        .filter(|value| !value.is_null())
        .or_else(|| envelope.get("error"));
    if let Some(payload) = payload {
        if let Ok(pretty) = serde_json::to_string_pretty(payload) {
            lines.extend(pretty.lines().map(|line| format!("  {line}")));
        }
    }
    lines
}

fn bounded_lines(mut lines: Vec<String>) -> Vec<String> {
    if lines.len() <= MAX_PROJECTED_LINES {
        return lines;
    }

    let omitted = lines.len() - (MAX_PROJECTED_LINES - 1);
    lines.truncate(MAX_PROJECTED_LINES - 1);
    lines.push(format!("  [projection truncated {omitted} provider lines]"));
    lines
}

fn format_process_failure(command: &str, output: &Output) -> String {
    format!(
        "Tosumu command {command:?} exited {:?}; stderr={}",
        output.status.code(),
        String::from_utf8_lossy(&output.stderr).trim(),
    )
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;

    #[test]
    fn envelope_projection_keeps_provider_fields_visible() {
        let lines = envelope_lines(&json!({
            "schema_version": 1,
            "command": "DESCRIBE",
            "outcome": { "key": "demo/message", "state": "found", "value_bytes": 17 },
        }));
        assert!(lines.iter().any(|line| line.contains("DESCRIBE")));
        assert!(lines.iter().any(|line| line.contains("demo/message")));
        assert!(lines.iter().any(|line| line.contains("value_bytes")));
    }

    #[test]
    fn session_evidence_preserves_command_order_in_the_transcript() {
        let evidence = SessionEvidence {
            schema_version: 1,
            fixture: "test",
            commands: vec![
                CommandEvidence {
                    input: "STATUS".into(),
                    lines: vec!["[tosumu / tql v1] STATUS".into()],
                    outcome: CommandOutcome::Success,
                    envelope: None,
                },
                CommandEvidence {
                    input: "DESCRIBE missing/key".into(),
                    lines: vec!["[tosumu / tql v1] DESCRIBE".into()],
                    outcome: CommandOutcome::ProviderFailure,
                    envelope: None,
                },
            ],
        };
        assert_eq!(
            evidence.transcript(),
            vec![
                "> STATUS",
                "[tosumu / tql v1] STATUS",
                "> DESCRIBE missing/key",
                "[tosumu / tql v1] DESCRIBE",
            ]
        );
        assert_eq!(evidence.success_count(), 1);
        assert_eq!(evidence.failure_count(), 1);
    }

    #[test]
    fn verbose_provider_projection_is_explicitly_bounded() {
        let lines = bounded_lines(
            (0..MAX_PROJECTED_LINES + 4)
                .map(|index| index.to_string())
                .collect(),
        );
        assert_eq!(lines.len(), MAX_PROJECTED_LINES);
        assert!(lines
            .last()
            .is_some_and(|line| line.contains("truncated 5")));
    }

    #[test]
    fn oversized_input_is_rejected_without_becoming_a_process_request() {
        let session = TosumuSession {
            cli: PathBuf::from("intentionally-unused-cli"),
            database: "unused.tsm".into(),
        };
        let command = "x".repeat(MAX_COMMAND_BYTES + 1);

        let evidence = session.execute_evidence(&command);

        assert_eq!(evidence.outcome, CommandOutcome::ContractFailure);
        assert!(evidence.lines[0].contains("adapter limit"));
        assert!(evidence.envelope.is_none());
        assert!(evidence.input.contains("input omitted"));
    }

    #[test]
    fn unavailable_provider_process_is_an_explicit_boundary_failure() {
        let session = TosumuSession {
            cli: PathBuf::from("intentionally-missing-tosumu-cli"),
            database: "unused.tsm".into(),
        };

        let evidence = session.execute_evidence("STATUS");

        assert_eq!(evidence.outcome, CommandOutcome::ProcessFailure);
        assert!(evidence.lines[0].starts_with("[tosumu process error]"));
        assert!(evidence.envelope.is_none());
    }
}
