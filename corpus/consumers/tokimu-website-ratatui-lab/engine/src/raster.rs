//! Website-local font selection over the shared Ratatui bridge.
//!
//! The website owns its Ratatui template composition. `tui-tools` owns the
//! optional provider translation and Tokimu's normalized cell-to-pixel seam.

use ratatui::buffer::Buffer;
use tui_tools::{rasterize_ratatui_buffer, TuiRasterFrame};
use ui_tools::UiFontRasterizer;

pub(crate) const CELL_PIXEL_WIDTH: u32 = tui_tools::CELL_PIXEL_WIDTH;
pub(crate) const CELL_PIXEL_HEIGHT: u32 = tui_tools::CELL_PIXEL_HEIGHT;

const DEPARTURE_MONO: &[u8] = include_bytes!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../../../third-party/fonts/departure-mono/public/assets/DepartureMono-Regular.otf"
));

pub(crate) type TokimuFrame = TuiRasterFrame;

pub(crate) fn rasterize(buffer: &Buffer) -> Result<TokimuFrame, String> {
    let font = UiFontRasterizer::from_bytes(DEPARTURE_MONO.to_vec())
        .map_err(|error| format!("load Departure Mono provider: {error}"))?;
    rasterize_ratatui_buffer(buffer, &font)
}

#[cfg(test)]
mod tests {
    use super::*;
    use ratatui::{buffer::Buffer, layout::Rect};

    #[test]
    fn ratatui_cells_use_the_shared_tokimu_raster_seam_deterministically() {
        let mut buffer = Buffer::empty(Rect::new(0, 0, 8, 3));
        buffer[(0, 0)].set_symbol("A");

        let first = rasterize(&buffer).expect("website adapter rasterizes a Ratatui buffer");
        let second = rasterize(&buffer).expect("website adapter rasterizes a Ratatui buffer");

        assert_eq!((first.width, first.height), (80, 54));
        assert_eq!(first.rgba.len(), 80 * 54 * 4);
        assert_eq!(first.fingerprint(), second.fingerprint());
    }
}
