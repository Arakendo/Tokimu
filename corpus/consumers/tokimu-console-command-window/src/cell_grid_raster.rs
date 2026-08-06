//! Deterministic CPU raster evidence for provider-neutral terminal cells.
//!
//! This is intentionally a corpus-local presentation adapter. It proves that
//! the retained cell contract can become one bounded image without teaching
//! Tokimu's renderer about Ratatui, terminals, selections, or carets.

use ui_tools::UiFontRasterizer;

use crate::tokimu_cell_projection::{CellBounds, TokimuCellLayout};

const MAX_PIXELS: u64 = 16_777_216;

#[derive(Clone, Copy, Debug)]
pub struct CellGridRasterOptions {
    pub font_pixels: f32,
    pub canvas_color: [u8; 4],
    pub selection_color: [u8; 4],
    pub caret_color: [u8; 4],
}

impl Default for CellGridRasterOptions {
    fn default() -> Self {
        Self {
            font_pixels: 16.0,
            canvas_color: [8, 18, 19, 255],
            selection_color: [28, 104, 105, 176],
            caret_color: [115, 230, 198, 255],
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct CellGridBitmap {
    pub width: u32,
    pub height: u32,
    pub rgba: Vec<u8>,
}

impl CellGridBitmap {
    pub fn fingerprint(&self) -> u64 {
        self.rgba.iter().fold(0xcbf29ce484222325_u64, |hash, byte| {
            (hash ^ u64::from(*byte)).wrapping_mul(0x100000001b3)
        })
    }

    fn blend_pixel(&mut self, x: i32, y: i32, source: [u8; 4]) {
        if x < 0 || y < 0 || x >= self.width as i32 || y >= self.height as i32 {
            return;
        }
        let index = (y as usize * self.width as usize + x as usize) * 4;
        let alpha = u32::from(source[3]);
        let inverse = 255 - alpha;
        for (channel, source_value) in source.iter().copied().take(3).enumerate() {
            self.rgba[index + channel] = ((u32::from(source_value) * alpha
                + u32::from(self.rgba[index + channel]) * inverse
                + 127)
                / 255) as u8;
        }
        self.rgba[index + 3] = 255;
    }

    fn fill_bounds(&mut self, bounds: &CellBounds, color: [u8; 4]) {
        let left = bounds.origin[0].floor() as i32;
        let top = bounds.origin[1].floor() as i32;
        let right = (bounds.origin[0] + bounds.size[0]).ceil() as i32;
        let bottom = (bounds.origin[1] + bounds.size[1]).ceil() as i32;
        for y in top..bottom {
            for x in left..right {
                self.blend_pixel(x, y, color);
            }
        }
    }
}

pub fn rasterize_cell_layout(
    layout: &TokimuCellLayout,
    font: &UiFontRasterizer,
    options: CellGridRasterOptions,
) -> Result<CellGridBitmap, String> {
    if !options.font_pixels.is_finite() || options.font_pixels <= 0.0 {
        return Err(format!(
            "cell-grid raster requires finite positive font pixels, received {}",
            options.font_pixels
        ));
    }
    let width = raster_dimension(layout.columns, layout.cell_size[0], "width")?;
    let height = raster_dimension(layout.rows, layout.cell_size[1], "height")?;
    let pixels = u64::from(width) * u64::from(height);
    if pixels > MAX_PIXELS {
        return Err(format!(
            "cell-grid raster exceeds the {MAX_PIXELS}-pixel corpus limit: {width}x{height}"
        ));
    }
    let mut bitmap = CellGridBitmap {
        width,
        height,
        rgba: options.canvas_color.repeat(pixels as usize),
    };

    for cell in &layout.cells {
        if cell.draw_background {
            bitmap.fill_bounds(&cell.bounds, cell.background);
        }
        if cell.selected {
            bitmap.fill_bounds(&cell.bounds, options.selection_color);
        }
        if !cell.draw_glyph {
            continue;
        }
        let mut characters = cell.glyph.chars();
        let character = characters.next().ok_or_else(|| {
            format!(
                "cell ({}, {}) requests an empty glyph draw",
                cell.column, cell.row
            )
        })?;
        if characters.next().is_some() {
            return Err(format!(
                "cell ({}, {}) requests a multi-character glyph draw",
                cell.column, cell.row
            ));
        }

        let glyph = font.rasterize(character, options.font_pixels);
        let pen_x = cell.bounds.origin[0] + (cell.bounds.size[0] - glyph.advance) * 0.5;
        let glyph_left = (pen_x + glyph.bearing_x).round() as i32;
        let glyph_top = (cell.baseline_y + glyph.bearing_y).round() as i32;
        for y in 0..glyph.height {
            for x in 0..glyph.width {
                let coverage = glyph.alpha[(y * glyph.width + x) as usize];
                let alpha =
                    ((u16::from(coverage) * u16::from(cell.foreground[3]) + 127) / 255) as u8;
                bitmap.blend_pixel(
                    glyph_left + x as i32,
                    glyph_top + y as i32,
                    [
                        cell.foreground[0],
                        cell.foreground[1],
                        cell.foreground[2],
                        alpha,
                    ],
                );
            }
        }
    }

    if layout.cursor.visible {
        bitmap.fill_bounds(&layout.cursor.bounds, options.caret_color);
    }
    Ok(bitmap)
}

fn raster_dimension(cells: u16, cell_pixels: f32, axis: &str) -> Result<u32, String> {
    if cells == 0 || !cell_pixels.is_finite() || cell_pixels <= 0.0 {
        return Err(format!(
            "cell-grid raster requires a non-empty finite positive {axis}"
        ));
    }
    let extent = f32::from(cells) * cell_pixels;
    if extent > u32::MAX as f32 || (extent.round() - extent).abs() > 0.001 {
        return Err(format!(
            "cell-grid raster {axis} must resolve to whole pixels, received {extent}"
        ));
    }
    Ok(extent.round() as u32)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        ratatui_projection::{CellEvidence, CursorEvidence, RatatuiSnapshot},
        tokimu_cell_projection::{lower_cells_with_options, CellLoweringOptions, CellSelection},
    };
    use ui_tools::UiFontSource;

    fn font() -> UiFontRasterizer {
        let source = UiFontSource::from_native_default().expect("checked-in default font");
        UiFontRasterizer::from_bytes(source.bytes).expect("valid default font")
    }

    fn snapshot(symbols: &[&str], cursor_visible: bool) -> RatatuiSnapshot {
        RatatuiSnapshot {
            schema_version: 1,
            width: symbols.len() as u16,
            height: 1,
            cursor: CursorEvidence {
                x: 0,
                y: 0,
                visible: cursor_visible,
            },
            cells: symbols
                .iter()
                .enumerate()
                .map(|(x, symbol)| CellEvidence {
                    x: x as u16,
                    y: 0,
                    symbol: (*symbol).into(),
                    foreground: "Cyan".into(),
                    background: "Reset".into(),
                    modifiers: Vec::new(),
                })
                .collect(),
        }
    }

    #[test]
    fn empty_cells_do_not_change_the_canvas() {
        let layout = lower_cells_with_options(
            &snapshot(&[" "], false),
            [10.0, 20.0],
            CellLoweringOptions {
                baseline_offset: Some(16.0),
                ..CellLoweringOptions::default()
            },
        )
        .expect("layout");
        let options = CellGridRasterOptions::default();
        let bitmap = rasterize_cell_layout(&layout, &font(), options).expect("raster");
        assert_eq!((bitmap.width, bitmap.height), (10, 20));
        assert!(bitmap
            .rgba
            .chunks_exact(4)
            .all(|pixel| pixel == options.canvas_color));
    }

    #[test]
    fn glyph_selection_and_caret_are_all_visible() {
        let layout = lower_cells_with_options(
            &snapshot(&["A", " "], true),
            [10.0, 20.0],
            CellLoweringOptions {
                selection: Some(CellSelection {
                    start: [0, 0],
                    end: [0, 0],
                }),
                baseline_offset: Some(16.0),
                caret_width: 2.0,
                ..CellLoweringOptions::default()
            },
        )
        .expect("layout");
        let options = CellGridRasterOptions::default();
        let bitmap = rasterize_cell_layout(&layout, &font(), options).expect("raster");

        assert!(bitmap
            .rgba
            .chunks_exact(4)
            .any(|pixel| pixel == options.caret_color));
        assert!(bitmap.rgba.chunks_exact(4).any(|pixel| {
            pixel[0] > options.selection_color[0]
                && pixel[1] > options.selection_color[1]
                && pixel[2] > options.selection_color[2]
        }));
        assert_eq!(
            bitmap
                .rgba
                .chunks_exact(4)
                .filter(|pixel| *pixel == options.caret_color)
                .count(),
            40
        );
    }

    #[test]
    fn raster_is_repeatable() {
        let layout = lower_cells_with_options(
            &snapshot(&["T", "O", "K", "I", "M", "U"], false),
            [10.0, 20.0],
            CellLoweringOptions {
                baseline_offset: Some(16.0),
                ..CellLoweringOptions::default()
            },
        )
        .expect("layout");
        let options = CellGridRasterOptions::default();
        let first = rasterize_cell_layout(&layout, &font(), options).expect("first");
        let second = rasterize_cell_layout(&layout, &font(), options).expect("second");
        assert_eq!(first, second);
        assert_eq!(first.fingerprint(), second.fingerprint());
    }

    #[test]
    fn fractional_pixel_extents_are_rejected() {
        let layout = lower_cells_with_options(
            &snapshot(&["A"], false),
            [10.25, 20.0],
            CellLoweringOptions {
                baseline_offset: Some(16.0),
                ..CellLoweringOptions::default()
            },
        )
        .expect("layout");
        assert!(
            rasterize_cell_layout(&layout, &font(), CellGridRasterOptions::default())
                .expect_err("fractional dimensions must be explicit")
                .contains("whole pixels")
        );
    }
}
