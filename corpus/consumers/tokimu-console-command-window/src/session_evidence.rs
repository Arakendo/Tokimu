use std::{fs, path::PathBuf};

use tokimu_console_command_window::tosumu_session::TosumuSession;

const SCRIPT: [&str; 6] = [
    "STATUS",
    "CHECK",
    "DESCRIBE demo/message",
    "DESCRIBE missing/key",
    "WAL STATUS",
    "STATUS trailing",
];

fn main() -> Result<(), String> {
    let session = TosumuSession::open_fixture()?;
    let evidence = session.run_script(&SCRIPT);
    let directory = artifact_directory()?;
    fs::create_dir_all(&directory)
        .map_err(|error| format!("create {}: {error}", directory.display()))?;

    let session_json = serde_json::to_string_pretty(&evidence)
        .map_err(|error| format!("serialize session artifact: {error}"))?;
    fs::write(directory.join("session.json"), session_json)
        .map_err(|error| format!("write session artifact: {error}"))?;
    fs::write(
        directory.join("transcript.txt"),
        evidence.transcript().join("\n") + "\n",
    )
    .map_err(|error| format!("write transcript artifact: {error}"))?;

    println!(
        "console-session-evidence: commands={}, successes={}, failures={}, artifacts={}",
        evidence.commands.len(),
        evidence.success_count(),
        evidence.failure_count(),
        directory.display()
    );
    Ok(())
}

fn artifact_directory() -> Result<PathBuf, String> {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .ancestors()
        .nth(3)
        .ok_or_else(|| "could not locate the Tokimu workspace root".to_owned())?
        .to_path_buf();
    Ok(root
        .join("target")
        .join("artifacts")
        .join("console-command-window")
        .join("tql-session-v1"))
}
