//! Corpus-local Ratatui producer evidence.
//!
//! This module converts Ratatui's resolved `TestBackend` cells into the local
//! terminal-surface candidate. Ratatui remains an optional corpus dependency;
//! no Ratatui type crosses the candidate boundary.

use ratatui::{
    backend::TestBackend,
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, Paragraph, Wrap},
    Terminal,
};

use super::{
    CellContent, ChangedCells, CursorState, FullFrame, NamedSurfaceColor, ResolvedCell,
    ResolvedCellStyle, SurfaceColor, SurfaceEmphasis, SurfaceExtent,
};

pub(super) fn render_fixture(
    epoch: u64,
    extent: SurfaceExtent,
    message: &str,
) -> Result<FullFrame, String> {
    if extent.columns < 8 || extent.rows < 4 {
        return Err("Ratatui fixture requires at least an 8x4 terminal surface".to_owned());
    }

    let backend = TestBackend::new(extent.columns, extent.rows);
    let mut terminal = Terminal::new(backend).map_err(|error| error.to_string())?;
    terminal
        .draw(|frame| {
            frame.render_widget(
                Paragraph::new(message)
                    .block(Block::default().borders(Borders::ALL).title("Ratatui"))
                    .style(Style::default().fg(Color::Cyan))
                    .wrap(Wrap { trim: false }),
                frame.area(),
            );
        })
        .map_err(|error| error.to_string())?;

    let cells = terminal
        .backend()
        .buffer()
        .content
        .iter()
        .enumerate()
        .map(|(index, cell)| ResolvedCell {
            column: (index % usize::from(extent.columns)) as u16,
            row: (index / usize::from(extent.columns)) as u16,
            content: if cell.symbol() == " " {
                CellContent::Empty
            } else {
                CellContent::grapheme(cell.symbol())
            },
            style: resolved_cell_style(cell.fg, cell.bg, cell.modifier),
        })
        .collect();

    Ok(FullFrame {
        epoch,
        extent,
        cells,
        // TestBackend is headless; the fixture's prompt location is therefore
        // declared by this adapter rather than inferred from a terminal host.
        cursor: CursorState {
            column: 1,
            row: extent.rows - 1,
            visible: true,
        },
    })
}

fn resolved_cell_style(
    foreground: Color,
    background: Color,
    modifier: Modifier,
) -> ResolvedCellStyle {
    ResolvedCellStyle {
        foreground: surface_color(foreground),
        background: surface_color(background),
        emphasis: SurfaceEmphasis {
            bold: modifier.contains(Modifier::BOLD),
            dim: modifier.contains(Modifier::DIM),
            italic: modifier.contains(Modifier::ITALIC),
            underlined: modifier.contains(Modifier::UNDERLINED),
            reversed: modifier.contains(Modifier::REVERSED),
            hidden: modifier.contains(Modifier::HIDDEN),
            crossed_out: modifier.contains(Modifier::CROSSED_OUT),
        },
    }
}

fn surface_color(color: Color) -> SurfaceColor {
    match color {
        Color::Reset => SurfaceColor::Default,
        Color::Black => SurfaceColor::Named(NamedSurfaceColor::Black),
        Color::Red => SurfaceColor::Named(NamedSurfaceColor::Red),
        Color::Green => SurfaceColor::Named(NamedSurfaceColor::Green),
        Color::Yellow => SurfaceColor::Named(NamedSurfaceColor::Yellow),
        Color::Blue => SurfaceColor::Named(NamedSurfaceColor::Blue),
        Color::Magenta => SurfaceColor::Named(NamedSurfaceColor::Magenta),
        Color::Cyan => SurfaceColor::Named(NamedSurfaceColor::Cyan),
        Color::Gray => SurfaceColor::Named(NamedSurfaceColor::Gray),
        Color::DarkGray => SurfaceColor::Named(NamedSurfaceColor::DarkGray),
        Color::LightRed => SurfaceColor::Named(NamedSurfaceColor::LightRed),
        Color::LightGreen => SurfaceColor::Named(NamedSurfaceColor::LightGreen),
        Color::LightYellow => SurfaceColor::Named(NamedSurfaceColor::LightYellow),
        Color::LightBlue => SurfaceColor::Named(NamedSurfaceColor::LightBlue),
        Color::LightMagenta => SurfaceColor::Named(NamedSurfaceColor::LightMagenta),
        Color::LightCyan => SurfaceColor::Named(NamedSurfaceColor::LightCyan),
        Color::White => SurfaceColor::Named(NamedSurfaceColor::White),
        Color::Rgb(red, green, blue) => SurfaceColor::Rgb { red, green, blue },
        Color::Indexed(index) => SurfaceColor::Indexed(index),
    }
}

pub(super) fn changed_cells_between(
    previous: &FullFrame,
    current: &FullFrame,
) -> Result<ChangedCells, String> {
    if previous.epoch != current.epoch || previous.extent != current.extent {
        return Err("Ratatui delta requires a matching epoch and extent".to_owned());
    }
    if previous.cells.len() != current.cells.len() {
        return Err("Ratatui full snapshots must describe the same cell count".to_owned());
    }

    let cells = previous
        .cells
        .iter()
        .zip(&current.cells)
        .filter(|(before, after)| before != after)
        .map(|(_, after)| after.clone())
        .collect();

    Ok(ChangedCells {
        epoch: current.epoch,
        extent: current.extent,
        cells,
        cursor: current.cursor,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{SurfaceUpdate, TerminalSurfaceReplica};

    #[test]
    fn ratatui_full_and_delta_snapshots_share_the_local_lifecycle() {
        let extent = SurfaceExtent {
            columns: 24,
            rows: 6,
        };
        let first = render_fixture(3, extent, "READY").unwrap();
        let second = render_fixture(3, extent, "DONE").unwrap();
        let change = changed_cells_between(&first, &second).unwrap();
        assert!(!change.cells.is_empty());

        let mut replica = TerminalSurfaceReplica::default();
        replica.apply(SurfaceUpdate::Full(first)).unwrap();
        let surface = replica.apply(SurfaceUpdate::Delta(change)).unwrap();

        assert!(surface
            .cells
            .iter()
            .flatten()
            .any(|cell| cell.content == CellContent::grapheme("D")));
        assert!(surface
            .cells
            .iter()
            .flatten()
            .any(|cell| { cell.style.foreground == SurfaceColor::Named(NamedSurfaceColor::Cyan) }));
        assert_eq!(replica.damage_statistics().full_frames, 1);
        assert_eq!(replica.damage_statistics().delta_frames, 1);
    }

    #[test]
    fn ratatui_requires_a_bounded_surface() {
        let error = render_fixture(
            1,
            SurfaceExtent {
                columns: 7,
                rows: 4,
            },
            "too small",
        )
        .unwrap_err();
        assert!(error.contains("8x4"));
    }
}
