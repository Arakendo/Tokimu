use std::{fs, path::PathBuf};

use tokimu_console_command_window::{
    ratatui_projection::render_session, tosumu_session::TosumuSession,
};

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
    let snapshot = render_session(&evidence, 96, 28)?;
    let directory = artifact_directory()?;
    fs::create_dir_all(&directory)
        .map_err(|error| format!("create artifact directory {}: {error}", directory.display()))?;
    fs::write(
        directory.join("ratatui-cells.json"),
        serde_json::to_vec_pretty(&snapshot)
            .map_err(|error| format!("encode cell artifact: {error}"))?,
    )
    .map_err(|error| format!("write cell artifact: {error}"))?;
    println!(
        "ratatui-session-evidence: cells={}, dimensions={}x{}, artifacts={}",
        snapshot.cells.len(),
        snapshot.width,
        snapshot.height,
        directory.display(),
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
        .join("ratatui-session-v1"))
}
