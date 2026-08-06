//! Headless agreement checks across the command-session projections.
//!
//! The comparison asserts only shared semantics: retained transcript text,
//! complete grid ownership, and cursor identity. Wrapping, palette choices,
//! borders, and other terminal-provider details remain intentionally outside
//! this contract.

use serde::Serialize;

use crate::{
    ratatui_projection::RatatuiSnapshot, tokimu_cell_projection::TokimuCellLayout,
    tosumu_session::SessionEvidence,
};

#[derive(Debug, Serialize)]
pub struct ProjectionConformance {
    pub schema_version: u64,
    pub transcript_lines: usize,
    pub terminal_cells: usize,
    pub tokimu_cells: usize,
    pub cursor_consistent: bool,
}

/// Confirms that each retained session line survived terminal projection and
/// that Tokimu's lowered grid still describes the exact same cells.
pub fn compare(
    session: &SessionEvidence,
    terminal: &RatatuiSnapshot,
    tokimu: &TokimuCellLayout,
) -> Result<ProjectionConformance, String> {
    if terminal.width != tokimu.columns || terminal.height != tokimu.rows {
        return Err(format!(
            "projection dimensions diverged: terminal={}x{}, tokimu={}x{}",
            terminal.width, terminal.height, tokimu.columns, tokimu.rows
        ));
    }
    if terminal.cells.len() != tokimu.cells.len() {
        return Err(format!(
            "projection cell counts diverged: terminal={}, tokimu={}",
            terminal.cells.len(),
            tokimu.cells.len()
        ));
    }
    if terminal.cursor.x != tokimu.cursor.column
        || terminal.cursor.y != tokimu.cursor.row
        || terminal.cursor.visible != tokimu.cursor.visible
    {
        return Err("projection cursor identity diverged".to_owned());
    }

    let terminal_text = terminal
        .cells
        .iter()
        .map(|cell| cell.symbol.as_str())
        .collect::<String>();
    let normalized_terminal = normalize(&terminal_text);
    for line in session.transcript() {
        let normalized_line = normalize(&line);
        if !normalized_line.is_empty() && !normalized_terminal.contains(&normalized_line) {
            return Err(format!(
                "terminal projection omitted retained transcript line: {line:?}"
            ));
        }
    }

    for (terminal_cell, tokimu_cell) in terminal.cells.iter().zip(&tokimu.cells) {
        if terminal_cell.x != tokimu_cell.column
            || terminal_cell.y != tokimu_cell.row
            || terminal_cell.symbol != tokimu_cell.glyph
        {
            return Err(format!(
                "cell identity diverged at terminal ({}, {})",
                terminal_cell.x, terminal_cell.y
            ));
        }
    }

    Ok(ProjectionConformance {
        schema_version: 1,
        transcript_lines: session.transcript().len(),
        terminal_cells: terminal.cells.len(),
        tokimu_cells: tokimu.cells.len(),
        cursor_consistent: true,
    })
}

fn normalize(value: &str) -> String {
    value
        .chars()
        .filter(|character| !character.is_whitespace())
        .collect()
}

#[cfg(test)]
mod tests {
    use crate::{
        ratatui_projection::render_session,
        tokimu_cell_projection::lower_cells,
        tosumu_session::{CommandEvidence, CommandOutcome},
    };

    use super::*;

    fn fixture() -> SessionEvidence {
        SessionEvidence {
            schema_version: 1,
            fixture: "test",
            commands: vec![CommandEvidence {
                input: "STATUS".into(),
                lines: vec!["[tosumu / tql v1] STATUS".into()],
                outcome: CommandOutcome::Success,
                envelope: None,
            }],
        }
    }

    #[test]
    fn shared_transcript_and_grid_identity_conform() {
        let session = fixture();
        let terminal = render_session(&session, 64, 18).expect("terminal projection");
        let tokimu = lower_cells(&terminal, [8.0, 16.0]).expect("Tokimu lowering");

        let conformance = compare(&session, &terminal, &tokimu).expect("conformance");
        assert_eq!(conformance.transcript_lines, 2);
        assert_eq!(conformance.terminal_cells, 64 * 18);
        assert!(conformance.cursor_consistent);
    }

    #[test]
    fn missing_transcript_content_is_an_explicit_divergence() {
        let session = fixture();
        let mut terminal = render_session(&session, 64, 18).expect("terminal projection");
        for cell in &mut terminal.cells {
            if cell.symbol == "S" {
                cell.symbol = " ".into();
            }
        }
        let tokimu = lower_cells(&terminal, [8.0, 16.0]).expect("Tokimu lowering");

        assert!(compare(&session, &terminal, &tokimu)
            .expect_err("missing transcript content must fail")
            .contains("omitted retained transcript"));
    }
}
