//! A deliberately small producer with no terminal-library dependency.
//!
//! It gives the corpus a second producer so the replica contract is tested
//! independently from Ratatui's layout and widget behavior.

use super::{
    CellContent, ChangedCells, CursorState, FullFrame, ResolvedCell, ResolvedCellStyle,
    SurfaceColor, SurfaceEmphasis, SurfaceExtent,
};
use unicode_width::UnicodeWidthChar;

pub(super) fn render_fixture(
    epoch: u64,
    extent: SurfaceExtent,
    message: &str,
) -> Result<FullFrame, String> {
    if extent.columns < 8 || extent.rows < 3 {
        return Err(format!(
            "fixture producer requires at least 8x3 cells, received {}x{}",
            extent.columns, extent.rows
        ));
    }

    let mut cells = Vec::new();
    write_line(&mut cells, extent, 0, "fixture");
    write_line(&mut cells, extent, 1, message);

    let prompt_width = message.chars().count().min(usize::from(extent.columns - 1));
    Ok(FullFrame {
        epoch,
        extent,
        cells,
        cursor: CursorState {
            column: u16::try_from(prompt_width).expect("bounded by the surface extent"),
            row: 1,
            visible: true,
        },
    })
}

pub(super) fn changed_cells_between(
    previous: &FullFrame,
    current: &FullFrame,
) -> Result<ChangedCells, String> {
    if previous.epoch != current.epoch || previous.extent != current.extent {
        return Err("fixture frame comparison requires a matching epoch and extent".to_owned());
    }

    let width = usize::from(current.extent.columns);
    let mut previous_cells = vec![None; current.extent.cell_count()];
    let mut current_cells = vec![None; current.extent.cell_count()];
    for cell in &previous.cells {
        previous_cells[usize::from(cell.row) * width + usize::from(cell.column)] =
            Some(cell.clone());
    }
    for cell in &current.cells {
        current_cells[usize::from(cell.row) * width + usize::from(cell.column)] =
            Some(cell.clone());
    }

    let cells = current_cells
        .iter()
        .zip(previous_cells.iter())
        .enumerate()
        .filter(|(_, (current, previous))| current != previous)
        .map(|(index, (current, _))| ResolvedCell {
            column: u16::try_from(index % width).expect("bounded by surface width"),
            row: u16::try_from(index / width).expect("bounded by surface height"),
            content: current
                .as_ref()
                .map(|cell| cell.content.clone())
                .unwrap_or(CellContent::Empty),
            style: current.as_ref().map(|cell| cell.style).unwrap_or_default(),
        })
        .collect();

    Ok(ChangedCells {
        epoch: current.epoch,
        extent: current.extent,
        cells,
        cursor: current.cursor,
    })
}

fn write_line(cells: &mut Vec<ResolvedCell>, extent: SurfaceExtent, row: u16, text: &str) {
    let mut column = 0;
    for character in text.chars() {
        let width = UnicodeWidthChar::width(character).unwrap_or(0);
        if width == 0 {
            continue;
        }
        if column + width > usize::from(extent.columns) {
            break;
        }
        cells.push(ResolvedCell {
            column: u16::try_from(column).expect("bounded by surface width"),
            row,
            content: CellContent::grapheme(character.to_string()),
            style: fixture_style(row),
        });
        for continuation_column in (column + 1)..(column + width) {
            cells.push(ResolvedCell {
                column: u16::try_from(continuation_column).expect("bounded by surface width"),
                row,
                content: CellContent::Continuation,
                style: fixture_style(row),
            });
        }
        column += width;
    }
}

/// Keep the independent fixture small while exercising every color form the
/// surface seam accepts. Ratatui remains responsible for its own style model.
fn fixture_style(row: u16) -> ResolvedCellStyle {
    match row {
        0 => ResolvedCellStyle {
            foreground: SurfaceColor::Indexed(14),
            background: SurfaceColor::Rgb {
                red: 8,
                green: 24,
                blue: 28,
            },
            emphasis: SurfaceEmphasis {
                bold: true,
                ..SurfaceEmphasis::default()
            },
        },
        _ => ResolvedCellStyle {
            foreground: SurfaceColor::Rgb {
                red: 216,
                green: 235,
                blue: 231,
            },
            background: SurfaceColor::Default,
            emphasis: SurfaceEmphasis::default(),
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{SurfaceUpdate, TerminalSurfaceReplica};

    #[test]
    fn fixture_full_and_delta_snapshots_share_the_local_lifecycle() {
        let extent = SurfaceExtent {
            columns: 24,
            rows: 6,
        };
        let first = render_fixture(3, extent, "READY").unwrap();
        let second = render_fixture(3, extent, "DONE").unwrap();
        let delta = changed_cells_between(&first, &second).unwrap();
        assert!(!delta.cells.is_empty());

        let mut replica = TerminalSurfaceReplica::default();
        replica.apply(SurfaceUpdate::Full(first)).unwrap();
        replica.apply(SurfaceUpdate::Delta(delta)).unwrap();
        assert!(replica
            .current
            .as_ref()
            .expect("a full frame establishes a surface")
            .cells
            .iter()
            .any(|cell| matches!(cell, Some(cell) if cell.content == CellContent::grapheme("D"))));
    }

    #[test]
    fn fixture_marks_the_trailing_cell_of_a_wide_grapheme_as_continuation() {
        let extent = SurfaceExtent {
            columns: 8,
            rows: 3,
        };
        let frame = render_fixture(3, extent, "A界B").unwrap();

        assert!(frame.cells.iter().any(|cell| {
            cell.column == 2 && cell.row == 1 && cell.content == CellContent::Continuation
        }));
    }
}
