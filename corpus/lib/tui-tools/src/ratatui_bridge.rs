//! Optional Ratatui buffer adapter for Tokimu's terminal raster seam.
//!
//! Ratatui owns terminal layout, widget composition, and its native style
//! model. This bridge converts an already-composed buffer into the normalized
//! cells consumed by the provider-neutral CPU raster path.

use ratatui::{
    buffer::Buffer,
    style::{Color, Modifier},
};
use ui_tools::UiFontRasterizer;

use crate::{rasterize_cells, TuiExtent, TuiRasterCell, TuiRasterFrame};

const CANVAS_COLOR: [u8; 4] = [5, 11, 13, 255];

/// Converts a completed Ratatui buffer into Tokimu raster cells.
///
/// This is intentionally post-composition: it does not expose Ratatui layout
/// types through the base TUI contract or make terminal composition a
/// renderer concern.
pub fn ratatui_buffer_cells(buffer: &Buffer) -> Vec<TuiRasterCell> {
    buffer.content().iter().map(to_raster_cell).collect()
}

/// Executes a completed Ratatui buffer through Tokimu's font raster seam.
///
/// The resulting frame is deterministic CPU evidence, not a terminal host or
/// backend framebuffer capture.
pub fn rasterize_ratatui_buffer(
    buffer: &Buffer,
    font: &UiFontRasterizer,
) -> Result<TuiRasterFrame, String> {
    rasterize_cells(
        TuiExtent::new(buffer.area.width, buffer.area.height),
        &ratatui_buffer_cells(buffer),
        font,
    )
}

fn to_raster_cell(cell: &ratatui::buffer::Cell) -> TuiRasterCell {
    TuiRasterCell {
        symbol: cell.symbol().chars().next().unwrap_or(' '),
        // Ratatui 0.29 resets the trailing cells of a multi-width grapheme to
        // blanks but does not expose a public continuation marker. Its `skip`
        // flag is for graphics diffing, not text width, so it must not be
        // reinterpreted here.
        continuation: false,
        foreground: ratatui_color_rgba(cell.fg, [216, 235, 231, 255]),
        background: (cell.bg != Color::Reset).then(|| ratatui_color_rgba(cell.bg, CANVAS_COLOR)),
        bold: cell.modifier.contains(Modifier::BOLD),
        dim: cell.modifier.contains(Modifier::DIM),
        underlined: cell.modifier.contains(Modifier::UNDERLINED),
        crossed_out: cell.modifier.contains(Modifier::CROSSED_OUT),
        hidden: cell.modifier.contains(Modifier::HIDDEN),
    }
}

fn ratatui_color_rgba(color: Color, reset: [u8; 4]) -> [u8; 4] {
    match color {
        Color::Reset => reset,
        Color::Black => [5, 11, 13, 255],
        Color::Red => [214, 91, 91, 255],
        Color::Green => [83, 193, 135, 255],
        Color::Yellow => [210, 168, 75, 255],
        Color::Blue => [88, 137, 218, 255],
        Color::Magenta => [184, 106, 202, 255],
        Color::Cyan => [77, 201, 194, 255],
        Color::Gray => [155, 171, 168, 255],
        Color::DarkGray => [91, 108, 106, 255],
        Color::LightRed => [241, 143, 143, 255],
        Color::LightGreen => [119, 227, 171, 255],
        Color::LightYellow => [231, 192, 109, 255],
        Color::LightBlue => [144, 185, 255, 255],
        Color::LightMagenta => [219, 157, 240, 255],
        Color::LightCyan => [115, 229, 223, 255],
        Color::White => [216, 235, 231, 255],
        Color::Rgb(red, green, blue) => [red, green, blue, 255],
        Color::Indexed(index) => indexed_color(index),
    }
}

fn indexed_color(index: u8) -> [u8; 4] {
    const ANSI: [[u8; 4]; 16] = [
        [5, 11, 13, 255],
        [214, 91, 91, 255],
        [83, 193, 135, 255],
        [210, 168, 75, 255],
        [88, 137, 218, 255],
        [184, 106, 202, 255],
        [77, 201, 194, 255],
        [155, 171, 168, 255],
        [91, 108, 106, 255],
        [241, 143, 143, 255],
        [119, 227, 171, 255],
        [231, 192, 109, 255],
        [144, 185, 255, 255],
        [219, 157, 240, 255],
        [115, 229, 223, 255],
        [216, 235, 231, 255],
    ];
    if index < 16 {
        return ANSI[index as usize];
    }
    if index >= 232 {
        let value = 8 + (index - 232) * 10;
        return [value, value, value, 255];
    }
    let cube = index - 16;
    let channel = |value: u8| if value == 0 { 0 } else { 55 + value * 40 };
    [
        channel(cube / 36),
        channel((cube / 6) % 6),
        channel(cube % 6),
        255,
    ]
}

#[cfg(test)]
mod tests {
    use ratatui::{
        buffer::Buffer,
        layout::Rect,
        style::{Color, Style},
    };
    use ui_tools::UiFontRasterizer;

    use super::{rasterize_ratatui_buffer, ratatui_buffer_cells};

    const DEPARTURE_MONO: &[u8] = include_bytes!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../../third-party/fonts/departure-mono/public/assets/DepartureMono-Regular.otf"
    ));

    #[test]
    fn preserves_ratatui_style_flags_through_the_normalized_cell_boundary() {
        let mut buffer = Buffer::empty(Rect::new(0, 0, 2, 1));
        buffer[(0, 0)].set_symbol("A").set_style(
            Style::default()
                .fg(Color::Cyan)
                .bg(Color::DarkGray)
                .add_modifier(
                    ratatui::style::Modifier::BOLD | ratatui::style::Modifier::CROSSED_OUT,
                ),
        );
        buffer[(1, 0)]
            .set_symbol("B")
            .set_style(Style::default().add_modifier(ratatui::style::Modifier::HIDDEN));

        let cells = ratatui_buffer_cells(&buffer);
        assert!(cells[0].bold);
        assert!(cells[0].crossed_out);
        assert_eq!(cells[0].background, Some([91, 108, 106, 255]));
        assert!(cells[1].hidden);

        let font = UiFontRasterizer::from_bytes(DEPARTURE_MONO.to_vec()).expect("font loads");
        let frame = rasterize_ratatui_buffer(&buffer, &font).expect("buffer rasterizes");
        assert_eq!((frame.width, frame.height), (20, 18));
    }

    #[test]
    fn does_not_mislabel_ratatui_wide_grapheme_padding_as_provider_continuation() {
        let mut buffer = Buffer::empty(Rect::new(0, 0, 3, 1));
        buffer.set_string(0, 0, "\u{30b3}", Style::default());

        let cells = ratatui_buffer_cells(&buffer);
        assert_eq!(cells[0].symbol, '\u{30b3}');
        assert_eq!(cells[1].symbol, ' ');
        assert!(!cells[0].continuation);
        assert!(!cells[1].continuation);
    }
}
