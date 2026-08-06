//! External-process evidence for Tosumu's provisional TQL JSON contract.
//!
//! The consumer intentionally treats TQL as opaque command text and JSON. It
//! must not link against Tosumu crates, parse the language, or inspect storage.

use std::{
    env,
    path::{Path, PathBuf},
    process::{Command, Output},
    time::{SystemTime, UNIX_EPOCH},
};

use serde_json::Value;

const SCHEMA_VERSION: u64 = 1;

fn main() -> Result<(), String> {
    let cli = find_tosumu_cli()?;
    let database = fixture_path()?;

    run_success(&cli, ["init", database.as_str()])?;
    run_success(
        &cli,
        ["put", database.as_str(), "asset/manifest", "schema-v1"],
    )?;

    let status = run_tql_json(&cli, &database, "STATUS", true)?;
    expect_success_command(&status, "STATUS")?;
    let check = run_tql_json(&cli, &database, "CHECK", true)?;
    expect_success_command(&check, "CHECK")?;
    let found = run_tql_json(&cli, &database, "DESCRIBE asset/manifest", true)?;
    expect_success_command(&found, "DESCRIBE")?;
    expect_field(&found, &["outcome", "state"], "found")?;
    let missing = run_tql_json(&cli, &database, "DESCRIBE missing/key", true)?;
    expect_success_command(&missing, "DESCRIBE")?;
    expect_field(&missing, &["outcome", "state"], "missing")?;
    let wal = run_tql_json(&cli, &database, "WAL STATUS", true)?;
    expect_success_command(&wal, "WAL STATUS")?;
    let invalid = run_tql_json(&cli, &database, "STATUS trailing", false)?;
    expect_field(&invalid, &["command"], "TQL")?;
    expect_field(&invalid, &["error", "code"], "TQL_UNEXPECTED_TOKEN")?;

    println!(
        "tosumu-tql-cli-consumer passed: schema=v{SCHEMA_VERSION}, commands=6, fixture={database}"
    );
    Ok(())
}

fn find_tosumu_cli() -> Result<PathBuf, String> {
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
            "Tosumu CLI was not built at {}. Run `cargo build --manifest-path .\\third-party\\tosumu\\Cargo.toml -p tosumu-cli` or set TOSUMU_CLI_BIN.",
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
        .join("tosumu-tql-cli-consumer")
        .join(format!("run-{}-{nonce}", std::process::id()));
    std::fs::create_dir_all(&directory)
        .map_err(|error| format!("create fixture directory {}: {error}", directory.display()))?;
    Ok(directory.join("fixture.tsm").display().to_string())
}

fn run_tql_json(
    cli: &Path,
    database: &str,
    command: &str,
    expect_success: bool,
) -> Result<Value, String> {
    let output = Command::new(cli)
        .args(["tql", database, command, "--json"])
        .output()
        .map_err(|error| format!("start TQL command {command:?}: {error}"))?;
    if output.status.success() != expect_success {
        return Err(format_process_failure(command, &output));
    }
    let value: Value = serde_json::from_slice(&output.stdout)
        .map_err(|error| format!("decode TQL JSON for {command:?}: {error}"))?;
    let version = value["schema_version"]
        .as_u64()
        .ok_or_else(|| format!("TQL JSON for {command:?} omitted schema_version"))?;
    if version != SCHEMA_VERSION {
        return Err(format!(
            "TQL JSON for {command:?} used schema v{version}, expected v{SCHEMA_VERSION}"
        ));
    }
    Ok(value)
}

fn run_success<const N: usize>(cli: &Path, arguments: [&str; N]) -> Result<(), String> {
    let output = Command::new(cli)
        .args(arguments)
        .output()
        .map_err(|error| format!("start Tosumu fixture command: {error}"))?;
    if output.status.success() {
        Ok(())
    } else {
        Err(format_process_failure("fixture setup", &output))
    }
}

fn expect_success_command(value: &Value, command: &str) -> Result<(), String> {
    expect_field(value, &["command"], command)?;
    value
        .get("outcome")
        .filter(|outcome| !outcome.is_null())
        .ok_or_else(|| format!("successful {command} response omitted an outcome"))?;
    value
        .get("error")
        .is_none_or(Value::is_null)
        .then_some(())
        .ok_or_else(|| format!("successful {command} response unexpectedly carried an error"))
}

fn expect_field(value: &Value, path: &[&str], expected: &str) -> Result<(), String> {
    let actual = path
        .iter()
        .try_fold(value, |current, segment| current.get(*segment))
        .ok_or_else(|| format!("TQL JSON omitted field {}", path.join(".")))?;
    (actual.as_str() == Some(expected))
        .then_some(())
        .ok_or_else(|| {
            format!(
                "TQL JSON field {} was {}, expected {expected:?}",
                path.join("."),
                actual
            )
        })
}

fn format_process_failure(command: &str, output: &Output) -> String {
    format!(
        "Tosumu command {command:?} exited {:?}; stdout={} stderr={}",
        output.status.code(),
        String::from_utf8_lossy(&output.stdout).trim(),
        String::from_utf8_lossy(&output.stderr).trim(),
    )
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;

    #[test]
    fn outcome_field_assertion_accepts_documented_values() {
        let value = json!({ "outcome": { "state": "found" } });
        expect_field(&value, &["outcome", "state"], "found").expect("documented state");
    }

    #[test]
    fn outcome_field_assertion_rejects_contract_drift() {
        let value = json!({ "outcome": { "state": "unknown" } });
        assert!(expect_field(&value, &["outcome", "state"], "found").is_err());
    }
}
