//! Emits semantic agreement evidence for the command-session projections.

use std::{fs, path::PathBuf};

use tokimu_console_command_window::{
    projection_conformance::compare, ratatui_projection::render_session,
    tokimu_cell_projection::lower_cells, tosumu_session::TosumuSession,
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
    // This evidence view is intentionally tall enough to retain the complete
    // bounded fixture transcript. Compact native views are allowed to clip.
    let snapshot = render_session(&evidence, 120, 80)?;
    let layout = lower_cells(&snapshot, [10.0, 20.0])?;
    let conformance = compare(&evidence, &snapshot, &layout)?;

    let directory = artifact_directory()?;
    fs::create_dir_all(&directory)
        .map_err(|error| format!("create artifact directory {}: {error}", directory.display()))?;
    let artifact = directory.join("projection-conformance.json");
    fs::write(
        &artifact,
        serde_json::to_vec_pretty(&conformance)
            .map_err(|error| format!("encode conformance artifact: {error}"))?,
    )
    .map_err(|error| format!("write {}: {error}", artifact.display()))?;

    println!(
        "projection-conformance-evidence: transcript_lines={}, cells={}, cursor_consistent={}, artifact={}",
        conformance.transcript_lines,
        conformance.tokimu_cells,
        conformance.cursor_consistent,
        artifact.display(),
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
        .join("projection-conformance-v1"))
}
