//! Corpus-local lowering from retained terminal cells to Tokimu-facing layout.
//!
//! Ratatui remains on the input side of this adapter. The output contains only
//! cell rectangles, glyph text, and presentation colors, so no Ratatui type is
//! exposed to Tokimu UI or renderer APIs.

use serde::Serialize;
use std::collections::BTreeSet;

use crate::ratatui_projection::RatatuiSnapshot;

#[derive(Debug, Serialize)]
pub struct TokimuCellLayout {
    pub schema_version: u64,
    pub columns: u16,
    pub rows: u16,
    pub cell_size: [f32; 2],
    pub row_baselines: Vec<f32>,
    pub cells: Vec<TokimuStyledCell>,
    pub cursor: TokimuCursor,
    pub diagnostics: Vec<CellProjectionDiagnostic>,
}

#[derive(Debug, Serialize)]
pub struct TokimuStyledCell {
    pub column: u16,
    pub row: u16,
    pub bounds: CellBounds,
    pub baseline_y: f32,
    pub glyph: String,
    pub foreground: [u8; 4],
    pub background: [u8; 4],
    pub draw_glyph: bool,
    pub draw_background: bool,
    pub selected: bool,
}

#[derive(Debug, Serialize)]
pub struct CellBounds {
    pub origin: [f32; 2],
    pub size: [f32; 2],
}

#[derive(Debug, Serialize)]
pub struct TokimuCursor {
    pub column: u16,
    pub row: u16,
    pub visible: bool,
    pub bounds: CellBounds,
}

#[derive(Clone, Copy, Debug, Default)]
pub struct CellSelection {
    pub start: [u16; 2],
    pub end: [u16; 2],
}

#[derive(Debug, Serialize)]
pub struct CellProjectionDiagnostic {
    pub code: &'static str,
    pub message: String,
    pub column: u16,
    pub row: u16,
}

pub struct CellLoweringOptions<'a> {
    pub selection: Option<CellSelection>,
    pub glyph_available: Option<&'a dyn Fn(char) -> bool>,
    pub baseline_offset: Option<f32>,
    pub caret_width: f32,
}

impl Default for CellLoweringOptions<'_> {
    fn default() -> Self {
        Self {
            selection: None,
            glyph_available: None,
            baseline_offset: None,
            caret_width: 1.0,
        }
    }
}

/// Lowers a retained terminal grid into renderer-neutral cell rectangles.
///
/// The color mapping is intentionally corpus-local and conservative. It
/// preserves the style intent exercised by this fixture without claiming that
/// terminal palette semantics are a Tokimu public contract.
pub fn lower_cells(
    snapshot: &RatatuiSnapshot,
    cell_size: [f32; 2],
) -> Result<TokimuCellLayout, String> {
    lower_cells_with_options(snapshot, cell_size, CellLoweringOptions::default())
}

pub fn lower_cells_with_options(
    snapshot: &RatatuiSnapshot,
    cell_size: [f32; 2],
    options: CellLoweringOptions<'_>,
) -> Result<TokimuCellLayout, String> {
    if !cell_size[0].is_finite()
        || !cell_size[1].is_finite()
        || cell_size[0] <= 0.0
        || cell_size[1] <= 0.0
    {
        return Err(format!(
            "Tokimu cell lowering requires finite positive cell dimensions, received {:?}",
            cell_size
        ));
    }
    validate_snapshot_grid(snapshot)?;

    let baseline_offset = options.baseline_offset.unwrap_or(cell_size[1] * 0.8);
    if !baseline_offset.is_finite() || !(0.0..=cell_size[1]).contains(&baseline_offset) {
        return Err(format!(
            "Tokimu cell lowering requires a finite baseline inside the cell height, received {baseline_offset} for height {}",
            cell_size[1]
        ));
    }
    if !options.caret_width.is_finite()
        || options.caret_width <= 0.0
        || options.caret_width > cell_size[0]
    {
        return Err(format!(
            "Tokimu cell lowering requires a finite positive caret width no larger than the cell width, received {} for width {}",
            options.caret_width, cell_size[0]
        ));
    }

    let bounds_for = |column: u16, row: u16| CellBounds {
        origin: [column as f32 * cell_size[0], row as f32 * cell_size[1]],
        size: cell_size,
    };
    let mut diagnostics = Vec::new();
    let mut cells = Vec::with_capacity(snapshot.cells.len());
    for cell in &snapshot.cells {
        for modifier in &cell.modifiers {
            diagnostics.push(CellProjectionDiagnostic {
                code: "unsupported-terminal-modifier",
                message: format!("terminal modifier `{modifier}` is retained as diagnostic evidence but is not lowered"),
                column: cell.x,
                row: cell.y,
            });
        }
        let visible_character = single_visible_character(&cell.symbol);
        if let (Some(character), Some(glyph_available)) =
            (visible_character, options.glyph_available)
        {
            if !glyph_available(character) {
                diagnostics.push(CellProjectionDiagnostic {
                    code: "missing-font-glyph",
                    message: format!(
                        "resolved font provider does not contain glyph U+{:04X}",
                        character as u32
                    ),
                    column: cell.x,
                    row: cell.y,
                });
            }
        }
        let background = terminal_color(&cell.background, [0, 0, 0, 0]);
        cells.push(TokimuStyledCell {
            column: cell.x,
            row: cell.y,
            bounds: bounds_for(cell.x, cell.y),
            baseline_y: cell.y as f32 * cell_size[1] + baseline_offset,
            glyph: cell.symbol.clone(),
            foreground: terminal_color(&cell.foreground, [190, 235, 222, 255]),
            background,
            draw_glyph: visible_character.is_some(),
            draw_background: background[3] > 0,
            selected: options
                .selection
                .is_some_and(|selection| selection.contains(cell.x, cell.y)),
        });
    }

    Ok(TokimuCellLayout {
        schema_version: 1,
        columns: snapshot.width,
        rows: snapshot.height,
        cell_size,
        row_baselines: (0..snapshot.height)
            .map(|row| row as f32 * cell_size[1] + baseline_offset)
            .collect(),
        cells,
        cursor: TokimuCursor {
            column: snapshot.cursor.x,
            row: snapshot.cursor.y,
            visible: snapshot.cursor.visible,
            bounds: CellBounds {
                origin: [
                    snapshot.cursor.x as f32 * cell_size[0],
                    snapshot.cursor.y as f32 * cell_size[1],
                ],
                size: [options.caret_width, cell_size[1]],
            },
        },
        diagnostics,
    })
}

impl CellSelection {
    fn contains(self, column: u16, row: u16) -> bool {
        let min_column = self.start[0].min(self.end[0]);
        let max_column = self.start[0].max(self.end[0]);
        let min_row = self.start[1].min(self.end[1]);
        let max_row = self.start[1].max(self.end[1]);
        (min_column..=max_column).contains(&column) && (min_row..=max_row).contains(&row)
    }
}

fn single_visible_character(symbol: &str) -> Option<char> {
    let mut characters = symbol.chars();
    let character = characters.next()?;
    (characters.next().is_none() && !character.is_whitespace() && !character.is_control())
        .then_some(character)
}

fn validate_snapshot_grid(snapshot: &RatatuiSnapshot) -> Result<(), String> {
    if snapshot.width == 0 || snapshot.height == 0 {
        return Err("Tokimu cell lowering requires a non-empty terminal grid".to_owned());
    }
    if snapshot.cursor.x >= snapshot.width || snapshot.cursor.y >= snapshot.height {
        return Err(format!(
            "Tokimu cell lowering received cursor ({}, {}) outside {}x{} grid",
            snapshot.cursor.x, snapshot.cursor.y, snapshot.width, snapshot.height
        ));
    }

    let expected_cells = snapshot.width as usize * snapshot.height as usize;
    if snapshot.cells.len() != expected_cells {
        return Err(format!(
            "Tokimu cell lowering requires a complete {}x{} grid ({expected_cells} cells), received {}",
            snapshot.width,
            snapshot.height,
            snapshot.cells.len()
        ));
    }

    let mut positions = BTreeSet::new();
    for cell in &snapshot.cells {
        if cell.x >= snapshot.width || cell.y >= snapshot.height {
            return Err(format!(
                "Tokimu cell lowering received cell ({}, {}) outside {}x{} grid",
                cell.x, cell.y, snapshot.width, snapshot.height
            ));
        }
        if !positions.insert((cell.x, cell.y)) {
            return Err(format!(
                "Tokimu cell lowering received duplicate cell ({}, {})",
                cell.x, cell.y
            ));
        }
    }
    Ok(())
}

fn terminal_color(color: &str, fallback: [u8; 4]) -> [u8; 4] {
    match color {
        "Cyan" | "LightCyan" => [115, 230, 198, 255],
        "Green" | "LightGreen" => [115, 230, 198, 255],
        "Black" => [0, 0, 0, 255],
        "Reset" => fallback,
        _ => fallback,
    }
}

#[cfg(test)]
mod tests {
    use crate::ratatui_projection::{CellEvidence, CursorEvidence, RatatuiSnapshot};

    use super::*;

    #[test]
    fn lowering_keeps_every_cell_and_cursor_inside_its_layout_grid() {
        let snapshot = RatatuiSnapshot {
            schema_version: 1,
            width: 2,
            height: 2,
            cursor: CursorEvidence {
                x: 1,
                y: 1,
                visible: true,
            },
            cells: vec![
                CellEvidence {
                    x: 0,
                    y: 0,
                    symbol: " ".into(),
                    foreground: "Reset".into(),
                    background: "Reset".into(),
                    modifiers: Vec::new(),
                },
                CellEvidence {
                    x: 1,
                    y: 0,
                    symbol: "T".into(),
                    foreground: "Cyan".into(),
                    background: "Reset".into(),
                    modifiers: Vec::new(),
                },
                CellEvidence {
                    x: 0,
                    y: 1,
                    symbol: " ".into(),
                    foreground: "Reset".into(),
                    background: "Reset".into(),
                    modifiers: Vec::new(),
                },
                CellEvidence {
                    x: 1,
                    y: 1,
                    symbol: " ".into(),
                    foreground: "Reset".into(),
                    background: "Reset".into(),
                    modifiers: Vec::new(),
                },
            ],
        };

        let layout = lower_cells(&snapshot, [8.0, 16.0]).expect("layout");
        assert_eq!(layout.cells.len(), 4);
        assert_eq!(layout.cells[1].bounds.origin, [8.0, 0.0]);
        assert_eq!(layout.cursor.bounds.origin, [8.0, 16.0]);
        assert_eq!(layout.cells[1].foreground, [115, 230, 198, 255]);
    }

    #[test]
    fn invalid_cell_dimensions_are_explicitly_rejected() {
        let snapshot = RatatuiSnapshot {
            schema_version: 1,
            width: 1,
            height: 1,
            cursor: CursorEvidence {
                x: 0,
                y: 0,
                visible: false,
            },
            cells: Vec::new(),
        };
        assert!(lower_cells(&snapshot, [0.0, 16.0]).is_err());
    }

    #[test]
    fn incomplete_or_duplicate_provider_grids_are_rejected() {
        let mut snapshot = RatatuiSnapshot {
            schema_version: 1,
            width: 2,
            height: 1,
            cursor: CursorEvidence {
                x: 0,
                y: 0,
                visible: true,
            },
            cells: vec![CellEvidence {
                x: 0,
                y: 0,
                symbol: "a".into(),
                foreground: "Reset".into(),
                background: "Reset".into(),
                modifiers: Vec::new(),
            }],
        };
        assert!(lower_cells(&snapshot, [8.0, 16.0])
            .expect_err("incomplete grids must be rejected")
            .contains("complete"));

        snapshot.cells.push(CellEvidence {
            x: 0,
            y: 0,
            symbol: "b".into(),
            foreground: "Reset".into(),
            background: "Reset".into(),
            modifiers: Vec::new(),
        });
        assert!(lower_cells(&snapshot, [8.0, 16.0])
            .expect_err("duplicate cells must be rejected")
            .contains("duplicate"));
    }

    #[test]
    fn draw_intent_keeps_empty_cells_geometry_free_and_marks_selection() {
        let snapshot = RatatuiSnapshot {
            schema_version: 1,
            width: 2,
            height: 1,
            cursor: CursorEvidence {
                x: 1,
                y: 0,
                visible: true,
            },
            cells: vec![
                CellEvidence {
                    x: 0,
                    y: 0,
                    symbol: " ".into(),
                    foreground: "Reset".into(),
                    background: "Reset".into(),
                    modifiers: Vec::new(),
                },
                CellEvidence {
                    x: 1,
                    y: 0,
                    symbol: "A".into(),
                    foreground: "Cyan".into(),
                    background: "Black".into(),
                    modifiers: Vec::new(),
                },
            ],
        };
        let layout = lower_cells_with_options(
            &snapshot,
            [8.0, 16.0],
            CellLoweringOptions {
                selection: Some(CellSelection {
                    start: [1, 0],
                    end: [1, 0],
                }),
                glyph_available: None,
                baseline_offset: Some(12.0),
                caret_width: 2.0,
            },
        )
        .expect("layout");

        assert!(!layout.cells[0].draw_glyph);
        assert!(!layout.cells[0].draw_background);
        assert!(!layout.cells[0].selected);
        assert!(layout.cells[1].draw_glyph);
        assert!(layout.cells[1].draw_background);
        assert!(layout.cells[1].selected);
        assert_eq!(layout.row_baselines, vec![12.0]);
        assert_eq!(layout.cells[1].baseline_y, 12.0);
        assert_eq!(layout.cursor.bounds.size, [2.0, 16.0]);
    }

    #[test]
    fn unsupported_modifiers_and_missing_glyphs_are_diagnostics() {
        let snapshot = RatatuiSnapshot {
            schema_version: 1,
            width: 1,
            height: 1,
            cursor: CursorEvidence {
                x: 0,
                y: 0,
                visible: false,
            },
            cells: vec![CellEvidence {
                x: 0,
                y: 0,
                symbol: "X".into(),
                foreground: "Reset".into(),
                background: "Reset".into(),
                modifiers: vec!["BOLD".into()],
            }],
        };
        let unavailable = |_character: char| false;
        let layout = lower_cells_with_options(
            &snapshot,
            [8.0, 16.0],
            CellLoweringOptions {
                selection: None,
                glyph_available: Some(&unavailable),
                ..CellLoweringOptions::default()
            },
        )
        .expect("layout");

        assert_eq!(layout.diagnostics.len(), 2);
        assert_eq!(layout.diagnostics[0].code, "unsupported-terminal-modifier");
        assert_eq!(layout.diagnostics[1].code, "missing-font-glyph");
    }

    #[test]
    fn cell_pixel_bounds_tile_the_declared_layout_without_overlap() {
        let snapshot = RatatuiSnapshot {
            schema_version: 1,
            width: 3,
            height: 2,
            cursor: CursorEvidence {
                x: 2,
                y: 1,
                visible: true,
            },
            cells: (0..2)
                .flat_map(|row| {
                    (0..3).map(move |column| CellEvidence {
                        x: column,
                        y: row,
                        symbol: " ".into(),
                        foreground: "Reset".into(),
                        background: "Reset".into(),
                        modifiers: Vec::new(),
                    })
                })
                .collect(),
        };
        let layout = lower_cells(&snapshot, [7.5, 13.25]).expect("layout");
        for cell in &layout.cells {
            assert_eq!(
                cell.bounds.origin,
                [cell.column as f32 * 7.5, cell.row as f32 * 13.25]
            );
            assert_eq!(cell.bounds.size, [7.5, 13.25]);
            assert!(cell.bounds.origin[0] + cell.bounds.size[0] <= 22.5);
            assert!(cell.bounds.origin[1] + cell.bounds.size[1] <= 26.5);
        }
        assert_eq!(layout.row_baselines, vec![10.6, 23.85]);
        assert_eq!(layout.cells[0].baseline_y, 10.6);
        assert_eq!(layout.cells[3].baseline_y, 23.85);
    }

    #[test]
    fn invalid_baseline_and_caret_metrics_are_rejected() {
        let snapshot = RatatuiSnapshot {
            schema_version: 1,
            width: 1,
            height: 1,
            cursor: CursorEvidence {
                x: 0,
                y: 0,
                visible: true,
            },
            cells: vec![CellEvidence {
                x: 0,
                y: 0,
                symbol: "A".into(),
                foreground: "Reset".into(),
                background: "Reset".into(),
                modifiers: Vec::new(),
            }],
        };

        assert!(lower_cells_with_options(
            &snapshot,
            [8.0, 16.0],
            CellLoweringOptions {
                baseline_offset: Some(17.0),
                ..CellLoweringOptions::default()
            }
        )
        .is_err());
        assert!(lower_cells_with_options(
            &snapshot,
            [8.0, 16.0],
            CellLoweringOptions {
                caret_width: 9.0,
                ..CellLoweringOptions::default()
            }
        )
        .is_err());
    }
}
