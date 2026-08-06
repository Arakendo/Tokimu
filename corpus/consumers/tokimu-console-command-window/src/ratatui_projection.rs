//! Corpus-local Ratatui projection of the provider-neutral command evidence.
//!
//! The projection consumes only the retained session transcript. It does not
//! inspect Tosumu storage or make terminal cells part of Tokimu's public API.

use ratatui::{
    backend::TestBackend,
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::Line,
    widgets::{Block, Borders, Paragraph, Wrap},
    Terminal,
};
use serde::Serialize;

use crate::tosumu_session::SessionEvidence;

#[derive(Debug, Serialize)]
pub struct RatatuiSnapshot {
    pub schema_version: u64,
    pub width: u16,
    pub height: u16,
    pub cursor: CursorEvidence,
    pub cells: Vec<CellEvidence>,
}

#[derive(Debug, Serialize)]
pub struct CursorEvidence {
    pub x: u16,
    pub y: u16,
    pub visible: bool,
}

#[derive(Debug, Serialize)]
pub struct CellEvidence {
    pub x: u16,
    pub y: u16,
    pub symbol: String,
    pub foreground: String,
    pub background: String,
    pub modifiers: Vec<String>,
}

pub fn render_session(
    evidence: &SessionEvidence,
    width: u16,
    height: u16,
) -> Result<RatatuiSnapshot, String> {
    if width < 8 || height < 5 {
        return Err(format!(
            "Ratatui console projection requires at least 8x5 cells, received {width}x{height}"
        ));
    }

    let backend = TestBackend::new(width, height);
    let mut terminal =
        Terminal::new(backend).map_err(|error| format!("create Ratatui backend: {error}"))?;
    let transcript = evidence.transcript();
    let prompt = "> _";

    terminal
        .draw(|frame| {
            let regions = Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Length(1),
                    Constraint::Min(3),
                    Constraint::Length(3),
                ])
                .split(frame.area());
            frame.render_widget(
                Paragraph::new("TOKIMU TERMINAL / TOSUMU TQL FIXTURE")
                    .style(Style::default().fg(Color::Cyan)),
                regions[0],
            );
            frame.render_widget(
                Paragraph::new(
                    transcript
                        .iter()
                        .map(|line| Line::from(line.as_str()))
                        .collect::<Vec<_>>(),
                )
                .block(Block::default().borders(Borders::ALL).title("Transcript"))
                .wrap(Wrap { trim: false }),
                regions[1],
            );
            frame.render_widget(
                Paragraph::new(prompt)
                    .block(Block::default().borders(Borders::ALL).title("Prompt"))
                    .style(Style::default().fg(Color::Green)),
                regions[2],
            );
        })
        .map_err(|error| format!("draw Ratatui snapshot: {error}"))?;

    let buffer = terminal.backend().buffer();
    // Retain the complete grid. Empty cells are meaningful terminal evidence:
    // they carry layout and background state even when they produce no glyph.
    let cells = buffer
        .content
        .iter()
        .enumerate()
        .map(|(index, cell)| CellEvidence {
            x: (index % width as usize) as u16,
            y: (index / width as usize) as u16,
            symbol: cell.symbol().to_owned(),
            foreground: format!("{:?}", cell.fg),
            background: format!("{:?}", cell.bg),
            modifiers: modifier_names(cell.modifier),
        })
        .collect();

    Ok(RatatuiSnapshot {
        schema_version: 1,
        width,
        height,
        cursor: CursorEvidence {
            x: 2,
            y: height.saturating_sub(2),
            visible: true,
        },
        cells,
    })
}

fn modifier_names(modifiers: Modifier) -> Vec<String> {
    [
        (Modifier::BOLD, "BOLD"),
        (Modifier::DIM, "DIM"),
        (Modifier::ITALIC, "ITALIC"),
        (Modifier::UNDERLINED, "UNDERLINED"),
        (Modifier::SLOW_BLINK, "SLOW_BLINK"),
        (Modifier::RAPID_BLINK, "RAPID_BLINK"),
        (Modifier::REVERSED, "REVERSED"),
        (Modifier::HIDDEN, "HIDDEN"),
        (Modifier::CROSSED_OUT, "CROSSED_OUT"),
    ]
    .into_iter()
    .filter(|(flag, _)| modifiers.contains(*flag))
    .map(|(_, name)| name.to_owned())
    .collect()
}

#[cfg(test)]
mod tests {
    use crate::tosumu_session::{CommandEvidence, CommandOutcome, SessionEvidence};

    use super::*;

    #[test]
    fn snapshot_contains_session_text_and_a_prompt_cursor() {
        let evidence = SessionEvidence {
            schema_version: 1,
            fixture: "test",
            commands: vec![CommandEvidence {
                input: "STATUS".into(),
                lines: vec!["[tosumu / tql v1] STATUS".into()],
                outcome: CommandOutcome::Success,
                envelope: None,
            }],
        };
        let snapshot = render_session(&evidence, 60, 16).expect("snapshot");
        assert!(snapshot.cells.iter().any(|cell| cell.symbol == "T"));
        assert!(snapshot.cells.iter().any(|cell| cell.symbol == ">"));
        assert!(snapshot.cursor.visible);
    }

    #[test]
    fn projection_stays_bounded_across_reviewed_terminal_sizes() {
        let evidence = SessionEvidence {
            schema_version: 1,
            fixture: "test",
            commands: vec![CommandEvidence {
                input: "DESCRIBE demo/message".into(),
                lines: vec![
                    "[tosumu / tql v1] DESCRIBE demo/message".into(),
                    "kind=resource name=demo/message bytes=23".into(),
                ],
                outcome: CommandOutcome::Success,
                envelope: None,
            }],
        };

        for (width, height) in [(32, 9), (64, 18), (96, 28)] {
            let snapshot = render_session(&evidence, width, height).expect("snapshot");
            assert_eq!((snapshot.width, snapshot.height), (width, height));
            assert_eq!(snapshot.cells.len(), width as usize * height as usize);
            assert!(snapshot
                .cells
                .iter()
                .all(|cell| cell.x < width && cell.y < height));
            assert!(snapshot.cursor.x < width);
            assert!(snapshot.cursor.y < height);
        }
    }

    #[test]
    fn undersized_terminal_is_an_explicit_projection_failure() {
        let evidence = SessionEvidence {
            schema_version: 1,
            fixture: "test",
            commands: Vec::new(),
        };

        let error = render_session(&evidence, 7, 5).expect_err("undersized terminal must fail");
        assert!(error.contains("at least 8x5 cells"));
    }
}
