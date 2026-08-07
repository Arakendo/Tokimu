use ui_tools::UiFontRasterizer;

use crate::{StyleRole, Surface, TuiExtent};

pub const CELL_PIXEL_WIDTH: u32 = 10;
pub const CELL_PIXEL_HEIGHT: u32 = 18;

const MAX_PIXELS: u64 = 16_777_216;
const DEFAULT_CANVAS_COLOR: [u8; 4] = [5, 11, 13, 255];

/// Caller-selected metrics for a bounded terminal raster surface.
///
/// The renderer owns only cell-to-pixel execution. Providers retain ownership
/// of their cell dimensions, font scale, baseline, and canvas color.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct TuiRasterOptions {
    pub cell_width: u32,
    pub cell_height: u32,
    pub font_pixels: f32,
    pub baseline_offset: f32,
    /// Alpha applied to a provider-resolved dim style.
    pub dim_alpha: u8,
    pub canvas: [u8; 4],
}

impl TuiRasterOptions {
    pub const DEFAULT: Self = Self {
        cell_width: CELL_PIXEL_WIDTH,
        cell_height: CELL_PIXEL_HEIGHT,
        font_pixels: 14.0,
        baseline_offset: 14.0,
        dim_alpha: 140,
        canvas: DEFAULT_CANVAS_COLOR,
    };
}

/// One normalized terminal cell ready for Tokimu's CPU text raster seam.
///
/// Terminal providers adapt their native styles into this shape. The raster
/// seam deliberately has no Ratatui dependency and does not select a font.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TuiRasterCell {
    pub symbol: char,
    /// This cell continues the preceding provider-resolved grapheme.
    ///
    /// Continuations retain their background but produce neither glyph ink nor
    /// cell decorations. The raster seam does not calculate Unicode width.
    pub continuation: bool,
    pub foreground: [u8; 4],
    pub background: Option<[u8; 4]>,
    pub bold: bool,
    pub dim: bool,
    pub underlined: bool,
    pub crossed_out: bool,
    pub hidden: bool,
}

impl TuiRasterCell {
    pub const fn plain(symbol: char, foreground: [u8; 4]) -> Self {
        Self {
            symbol,
            continuation: false,
            foreground,
            background: None,
            bold: false,
            dim: false,
            underlined: false,
            crossed_out: false,
            hidden: false,
        }
    }
}

/// Deterministic CPU RGBA output for an already-composed terminal surface.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TuiRasterFrame {
    pub width: u32,
    pub height: u32,
    pub rgba: Vec<u8>,
}

impl TuiRasterFrame {
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

    fn fill_cell(&mut self, column: u16, row: u16, color: [u8; 4], options: TuiRasterOptions) {
        let left = u32::from(column) * options.cell_width;
        let top = u32::from(row) * options.cell_height;
        for y in top..top + options.cell_height {
            for x in left..left + options.cell_width {
                self.blend_pixel(x as i32, y as i32, color);
            }
        }
    }
}

/// Rasterizes normalized cells through a concrete caller-selected font.
///
/// The output is deterministic CPU evidence. It is not a GPU framebuffer
/// capture and does not establish browser or backend pixel equivalence.
pub fn rasterize_cells(
    extent: TuiExtent,
    cells: &[TuiRasterCell],
    font: &UiFontRasterizer,
) -> Result<TuiRasterFrame, String> {
    rasterize_cells_with_options(extent, cells, font, TuiRasterOptions::DEFAULT)
}

/// Rasterizes normalized cells using provider-selected surface metrics.
///
/// This permits providers with different terminal-cell conventions to share
/// Tokimu's font raster path without teaching it terminal layout or styling
/// semantics.
pub fn rasterize_cells_with_options(
    extent: TuiExtent,
    cells: &[TuiRasterCell],
    font: &UiFontRasterizer,
    options: TuiRasterOptions,
) -> Result<TuiRasterFrame, String> {
    let expected = usize::from(extent.columns) * usize::from(extent.rows);
    if cells.len() != expected {
        return Err(format!(
            "terminal raster cell count mismatch: expected {expected}, received {}",
            cells.len()
        ));
    }
    if options.cell_width == 0 || options.cell_height == 0 || options.font_pixels <= 0.0 {
        return Err(
            "terminal raster options require non-zero cell metrics and font size".to_owned(),
        );
    }
    let width = u32::from(extent.columns) * options.cell_width;
    let height = u32::from(extent.rows) * options.cell_height;
    let pixels = u64::from(width) * u64::from(height);
    if pixels > MAX_PIXELS {
        return Err(format!(
            "terminal raster frame exceeds the {MAX_PIXELS}-pixel limit: {width}x{height}"
        ));
    }
    let mut frame = TuiRasterFrame {
        width,
        height,
        rgba: options.canvas.repeat(pixels as usize),
    };
    for (index, cell) in cells.iter().enumerate() {
        let column = (index % usize::from(extent.columns)) as u16;
        let row = (index / usize::from(extent.columns)) as u16;
        if let Some(background) = cell.background {
            frame.fill_cell(column, row, background, options);
        }
        if cell.hidden || cell.continuation || cell.symbol.is_whitespace() {
            continue;
        }
        let glyph = font.rasterize(cell.symbol, options.font_pixels);
        let cell_left = f32::from(column) * options.cell_width as f32;
        let cell_top = f32::from(row) * options.cell_height as f32;
        let pen_x = cell_left + (options.cell_width as f32 - glyph.advance) * 0.5;
        let glyph_left = (pen_x + glyph.bearing_x).round() as i32;
        let glyph_top = (cell_top + options.baseline_offset + glyph.bearing_y).round() as i32;
        let mut foreground = cell.foreground;
        if cell.dim {
            foreground[3] = (u16::from(foreground[3]) * u16::from(options.dim_alpha) / 255) as u8;
        }
        for y in 0..glyph.height {
            for x in 0..glyph.width {
                let coverage = glyph.alpha[(y * glyph.width + x) as usize];
                let alpha = ((u16::from(coverage) * u16::from(foreground[3]) + 127) / 255) as u8;
                let pixel = [foreground[0], foreground[1], foreground[2], alpha];
                frame.blend_pixel(glyph_left + x as i32, glyph_top + y as i32, pixel);
                if cell.bold {
                    frame.blend_pixel(glyph_left + x as i32 + 1, glyph_top + y as i32, pixel);
                }
            }
        }
        if cell.underlined {
            let underline_y = i32::from(row) * options.cell_height as i32
                + options.cell_height.saturating_sub(3) as i32;
            let underline_x = i32::from(column) * options.cell_width as i32;
            for x in 0..options.cell_width as i32 {
                frame.blend_pixel(underline_x + x, underline_y, foreground);
            }
        }
        if cell.crossed_out {
            let crossed_out_y =
                i32::from(row) * options.cell_height as i32 + (options.cell_height / 2) as i32;
            let crossed_out_x = i32::from(column) * options.cell_width as i32;
            for x in 0..options.cell_width as i32 {
                frame.blend_pixel(crossed_out_x + x, crossed_out_y, foreground);
            }
        }
    }
    Ok(frame)
}

/// Rasterizes a Tokimu-authored terminal surface using the default semantic
/// palette. Consumers retain ownership of the font provider they pass in.
pub fn rasterize_surface(
    surface: &Surface,
    font: &UiFontRasterizer,
) -> Result<TuiRasterFrame, String> {
    let cells = surface
        .cells()
        .iter()
        .map(|cell| TuiRasterCell::plain(cell.symbol, role_color(cell.role)))
        .collect::<Vec<_>>();
    rasterize_cells(surface.extent(), &cells, font)
}

fn role_color(role: StyleRole) -> [u8; 4] {
    match role {
        StyleRole::Plain | StyleRole::Value => [216, 235, 231, 255],
        StyleRole::Frame | StyleRole::Accent => [77, 201, 194, 255],
        StyleRole::Heading => [115, 229, 223, 255],
        StyleRole::Label | StyleRole::Muted => [155, 171, 168, 255],
        StyleRole::Warning => [231, 192, 109, 255],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_a_surface_with_the_wrong_cell_count() {
        let font = UiFontRasterizer::from_bytes(vec![0; 4]);
        assert!(font.is_err());
        let error = rasterize_cells(
            TuiExtent::new(2, 2),
            &[TuiRasterCell::plain('A', [255; 4])],
            // The count validation happens before the font is used. A valid
            // font is covered by the consumer seam test below.
            &UiFontRasterizer::from_bytes(
                include_bytes!(concat!(
                env!("CARGO_MANIFEST_DIR"),
                "/../../../third-party/fonts/departure-mono/public/assets/DepartureMono-Regular.otf"
            ))
                .to_vec(),
            )
            .expect("Departure Mono fixture"),
        )
        .expect_err("mismatched cells are rejected");
        assert!(error.contains("cell count mismatch"));
    }

    #[test]
    fn honors_explicit_metrics_and_terminal_cell_styles() {
        let font = UiFontRasterizer::from_bytes(
            include_bytes!(concat!(
                env!("CARGO_MANIFEST_DIR"),
                "/../../../third-party/fonts/departure-mono/public/assets/DepartureMono-Regular.otf"
            ))
            .to_vec(),
        )
        .expect("Departure Mono fixture");
        let options = TuiRasterOptions {
            cell_width: 12,
            cell_height: 20,
            font_pixels: 15.0,
            baseline_offset: 15.0,
            dim_alpha: 96,
            canvas: [3, 4, 5, 255],
        };
        let visible = TuiRasterCell {
            symbol: 'A',
            continuation: false,
            foreground: [220, 230, 240, 255],
            background: Some([10, 20, 30, 255]),
            bold: true,
            dim: true,
            underlined: true,
            crossed_out: true,
            hidden: false,
        };
        let hidden = TuiRasterCell {
            hidden: true,
            ..visible
        };

        let visible_frame =
            rasterize_cells_with_options(TuiExtent::new(1, 1), &[visible], &font, options)
                .expect("visible terminal cell");
        let hidden_frame =
            rasterize_cells_with_options(TuiExtent::new(1, 1), &[hidden], &font, options)
                .expect("hidden terminal cell");

        assert_eq!((visible_frame.width, visible_frame.height), (12, 20));
        assert_ne!(visible_frame.fingerprint(), hidden_frame.fingerprint());
        // Hidden text preserves the provider-selected cell background.
        assert_eq!(&hidden_frame.rgba[..4], &[10, 20, 30, 255]);
    }

    #[test]
    fn continuation_preserves_its_background_without_emitting_ink_or_decoration() {
        let font = UiFontRasterizer::from_bytes(
            include_bytes!(concat!(
                env!("CARGO_MANIFEST_DIR"),
                "/../../../third-party/fonts/departure-mono/public/assets/DepartureMono-Regular.otf"
            ))
            .to_vec(),
        )
        .expect("Departure Mono fixture");
        let options = TuiRasterOptions {
            cell_width: 12,
            cell_height: 20,
            font_pixels: 15.0,
            baseline_offset: 15.0,
            dim_alpha: 96,
            canvas: [3, 4, 5, 255],
        };
        let blank = TuiRasterCell {
            symbol: ' ',
            continuation: false,
            foreground: [220, 230, 240, 255],
            background: Some([10, 20, 30, 255]),
            bold: true,
            dim: false,
            underlined: true,
            crossed_out: true,
            hidden: false,
        };
        let continuation = TuiRasterCell {
            symbol: 'A',
            continuation: true,
            ..blank
        };

        let blank_frame =
            rasterize_cells_with_options(TuiExtent::new(1, 1), &[blank], &font, options)
                .expect("blank cell rasterizes");
        let continuation_frame =
            rasterize_cells_with_options(TuiExtent::new(1, 1), &[continuation], &font, options)
                .expect("continuation cell rasterizes");

        assert_eq!(continuation_frame, blank_frame);
        assert_eq!(&continuation_frame.rgba[..4], &[10, 20, 30, 255]);
    }
}
