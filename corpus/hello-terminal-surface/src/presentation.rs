use tui_tools::{rasterize_cells_with_options, TuiExtent, TuiRasterCell, TuiRasterOptions};
use ui_tools::UiFontRasterizer;

use crate::{
    CellContent, NamedSurfaceColor, ResolvedCellStyle, SurfaceColor, TerminalSurfaceObservation,
};

pub(crate) const CELL_PIXEL_WIDTH: u32 = 12;
pub(crate) const CELL_PIXEL_HEIGHT: u32 = 20;
const CANVAS: [u8; 4] = [5, 11, 13, 255];

const RASTER_OPTIONS: TuiRasterOptions = TuiRasterOptions {
    cell_width: CELL_PIXEL_WIDTH,
    cell_height: CELL_PIXEL_HEIGHT,
    font_pixels: 15.0,
    baseline_offset: 15.0,
    dim_alpha: 96,
    canvas: CANVAS,
};

const DEPARTURE_MONO: &[u8] = include_bytes!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../third-party/fonts/departure-mono/public/assets/DepartureMono-Regular.otf"
));

/// Deterministic CPU evidence for a bounded resolved cell surface.
///
/// `tui-tools` owns the cell-to-RGBA execution seam. This fixture retains
/// ownership of resolved terminal styles and provider-specific cell meaning.
pub(crate) type TerminalSurfaceRaster = tui_tools::TuiRasterFrame;

/// Corpus-local CPU raster reuse evidence for one resolved terminal surface.
///
/// This is intentionally not a `tui-tools` cache. The fixture owns its font,
/// invalidation policy, and the decision to retain a complete CPU frame. A
/// changed resolved surface currently invalidates the whole frame; future
/// partial-update work must establish its own evidence rather than inheriting
/// an accidental policy from this corpus application.
pub(crate) struct TerminalSurfaceRasterCache {
    font: UiFontRasterizer,
    cached_surface: Option<TerminalSurfaceObservation>,
    cached_raster: Option<TerminalSurfaceRaster>,
    observations: TerminalSurfaceRasterCacheObservations,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) struct TerminalSurfaceRasterCacheObservations {
    pub(crate) font_provider_loads: u64,
    pub(crate) rasterizations: u64,
    pub(crate) cache_hits: u64,
    pub(crate) full_invalidations: u64,
}

impl TerminalSurfaceRasterCache {
    pub(crate) fn new() -> Result<Self, String> {
        let font = UiFontRasterizer::from_bytes(DEPARTURE_MONO.to_vec())
            .map_err(|error| format!("Departure Mono rasterizer failed: {error}"))?;
        Ok(Self {
            font,
            cached_surface: None,
            cached_raster: None,
            observations: TerminalSurfaceRasterCacheObservations {
                font_provider_loads: 1,
                ..TerminalSurfaceRasterCacheObservations::default()
            },
        })
    }

    pub(crate) fn rasterize(
        &mut self,
        surface: &TerminalSurfaceObservation,
    ) -> Result<&TerminalSurfaceRaster, String> {
        if self.cached_surface.as_ref() == Some(surface) {
            self.observations.cache_hits += 1;
            return Ok(self
                .cached_raster
                .as_ref()
                .expect("matching cached surface retains its raster"));
        }

        if self.cached_surface.is_some() {
            self.observations.full_invalidations += 1;
        }
        let raster = rasterize_with_font(surface, &self.font)?;
        self.observations.rasterizations += 1;
        self.cached_surface = Some(surface.clone());
        self.cached_raster = Some(raster);
        Ok(self
            .cached_raster
            .as_ref()
            .expect("new cached surface retains its raster"))
    }

    pub(crate) const fn observations(&self) -> TerminalSurfaceRasterCacheObservations {
        self.observations
    }
}

pub(crate) fn rasterize(
    surface: &TerminalSurfaceObservation,
) -> Result<TerminalSurfaceRaster, String> {
    let font = UiFontRasterizer::from_bytes(DEPARTURE_MONO.to_vec())
        .map_err(|error| format!("Departure Mono rasterizer failed: {error}"))?;
    rasterize_with_font(surface, &font)
}

fn rasterize_with_font(
    surface: &TerminalSurfaceObservation,
    font: &UiFontRasterizer,
) -> Result<TerminalSurfaceRaster, String> {
    let cells = surface
        .cells
        .iter()
        .map(|cell| match cell {
            Some(cell) => {
                let (foreground, background) = resolved_colors(cell.style);
                let symbol = match &cell.content {
                    CellContent::Grapheme(grapheme) => grapheme.chars().next().unwrap_or(' '),
                    CellContent::Continuation | CellContent::Empty => ' ',
                };
                TuiRasterCell {
                    symbol,
                    continuation: matches!(&cell.content, CellContent::Continuation),
                    foreground,
                    background: (background != CANVAS).then_some(background),
                    bold: cell.style.emphasis.bold,
                    dim: cell.style.emphasis.dim,
                    underlined: cell.style.emphasis.underlined,
                    crossed_out: cell.style.emphasis.crossed_out,
                    hidden: cell.style.emphasis.hidden,
                }
            }
            None => TuiRasterCell::plain(' ', [216, 235, 231, 255]),
        })
        .collect::<Vec<_>>();
    rasterize_cells_with_options(
        TuiExtent::new(surface.extent.columns, surface.extent.rows),
        &cells,
        font,
        RASTER_OPTIONS,
    )
}

fn resolved_colors(style: ResolvedCellStyle) -> ([u8; 4], [u8; 4]) {
    let foreground = color(style.foreground, [216, 235, 231, 255]);
    let background = color(style.background, CANVAS);
    if style.emphasis.reversed {
        (background, foreground)
    } else {
        (foreground, background)
    }
}

fn color(color: SurfaceColor, fallback: [u8; 4]) -> [u8; 4] {
    match color {
        SurfaceColor::Default => fallback,
        SurfaceColor::Rgb { red, green, blue } => [red, green, blue, 255],
        SurfaceColor::Indexed(index) => indexed_color(index),
        SurfaceColor::Named(named) => match named {
            NamedSurfaceColor::Black => [0, 0, 0, 255],
            NamedSurfaceColor::Red => [205, 49, 49, 255],
            NamedSurfaceColor::Green => [13, 188, 121, 255],
            NamedSurfaceColor::Yellow => [229, 229, 16, 255],
            NamedSurfaceColor::Blue => [36, 114, 200, 255],
            NamedSurfaceColor::Magenta => [188, 63, 188, 255],
            NamedSurfaceColor::Cyan => [17, 168, 205, 255],
            NamedSurfaceColor::Gray => [229, 229, 229, 255],
            NamedSurfaceColor::DarkGray => [102, 102, 102, 255],
            NamedSurfaceColor::LightRed => [241, 76, 76, 255],
            NamedSurfaceColor::LightGreen => [35, 209, 139, 255],
            NamedSurfaceColor::LightYellow => [245, 245, 67, 255],
            NamedSurfaceColor::LightBlue => [59, 142, 234, 255],
            NamedSurfaceColor::LightMagenta => [214, 112, 214, 255],
            NamedSurfaceColor::LightCyan => [41, 184, 219, 255],
            NamedSurfaceColor::White => [255, 255, 255, 255],
        },
    }
}

fn indexed_color(index: u8) -> [u8; 4] {
    let level = [0, 95, 135, 175, 215, 255];
    match index {
        0 => color(SurfaceColor::Named(NamedSurfaceColor::Black), CANVAS),
        1 => color(SurfaceColor::Named(NamedSurfaceColor::Red), CANVAS),
        2 => color(SurfaceColor::Named(NamedSurfaceColor::Green), CANVAS),
        3 => color(SurfaceColor::Named(NamedSurfaceColor::Yellow), CANVAS),
        4 => color(SurfaceColor::Named(NamedSurfaceColor::Blue), CANVAS),
        5 => color(SurfaceColor::Named(NamedSurfaceColor::Magenta), CANVAS),
        6 => color(SurfaceColor::Named(NamedSurfaceColor::Cyan), CANVAS),
        7 => color(SurfaceColor::Named(NamedSurfaceColor::Gray), CANVAS),
        8 => color(SurfaceColor::Named(NamedSurfaceColor::DarkGray), CANVAS),
        9 => color(SurfaceColor::Named(NamedSurfaceColor::LightRed), CANVAS),
        10 => color(SurfaceColor::Named(NamedSurfaceColor::LightGreen), CANVAS),
        11 => color(SurfaceColor::Named(NamedSurfaceColor::LightYellow), CANVAS),
        12 => color(SurfaceColor::Named(NamedSurfaceColor::LightBlue), CANVAS),
        13 => color(SurfaceColor::Named(NamedSurfaceColor::LightMagenta), CANVAS),
        14 => color(SurfaceColor::Named(NamedSurfaceColor::LightCyan), CANVAS),
        15 => color(SurfaceColor::Named(NamedSurfaceColor::White), CANVAS),
        16..=231 => {
            let value = index - 16;
            [
                level[(value / 36) as usize],
                level[((value / 6) % 6) as usize],
                level[(value % 6) as usize],
                255,
            ]
        }
        232..=255 => {
            let shade = 8 + (index - 232) * 10;
            [shade, shade, shade, 255]
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        fixture_producer, ChangedCells, CursorState, ResolvedCell, SurfaceEmphasis, SurfaceExtent,
        SurfaceUpdate, TerminalSurfaceReplica,
    };

    #[test]
    fn rasterizes_a_bounded_fixture_surface() {
        let frame = fixture_producer::render_fixture(
            1,
            SurfaceExtent {
                columns: 12,
                rows: 3,
            },
            "READY",
        )
        .expect("fixture should render");
        let surface = TerminalSurfaceObservation::from_full(frame).expect("frame should apply");
        let raster = rasterize(&surface).expect("surface should rasterize");
        assert_eq!(raster.width, 12 * CELL_PIXEL_WIDTH);
        assert_eq!(raster.height, 3 * CELL_PIXEL_HEIGHT);
        assert_ne!(
            raster.fingerprint(),
            TerminalSurfaceRaster {
                width: raster.width,
                height: raster.height,
                rgba: CANVAS.repeat((raster.width * raster.height) as usize)
            }
            .fingerprint()
        );
    }

    #[test]
    fn clips_ink_that_reaches_the_surface_edge() {
        let frame = crate::FullFrame {
            epoch: 1,
            extent: SurfaceExtent {
                columns: 1,
                rows: 1,
            },
            cells: vec![ResolvedCell {
                column: 0,
                row: 0,
                content: CellContent::grapheme("W"),
                style: ResolvedCellStyle {
                    foreground: SurfaceColor::Named(NamedSurfaceColor::LightCyan),
                    background: SurfaceColor::Rgb {
                        red: 1,
                        green: 2,
                        blue: 3,
                    },
                    emphasis: SurfaceEmphasis {
                        bold: true,
                        underlined: true,
                        ..SurfaceEmphasis::default()
                    },
                },
            }],
            cursor: CursorState {
                column: 0,
                row: 0,
                visible: true,
            },
        };
        let surface = TerminalSurfaceObservation::from_full(frame).expect("frame should apply");
        let raster = rasterize(&surface).expect("edge glyph must clip instead of escaping");
        assert_eq!(
            raster.rgba.len(),
            (CELL_PIXEL_WIDTH * CELL_PIXEL_HEIGHT * 4) as usize
        );
        assert_eq!(&raster.rgba[..4], &[1, 2, 3, 255]);
    }

    #[test]
    fn style_only_delta_changes_cpu_evidence_without_relaying_text() {
        let full = crate::FullFrame {
            epoch: 3,
            extent: SurfaceExtent {
                columns: 2,
                rows: 1,
            },
            cells: vec![ResolvedCell {
                column: 0,
                row: 0,
                content: CellContent::grapheme("A"),
                style: ResolvedCellStyle::default(),
            }],
            cursor: CursorState {
                column: 0,
                row: 0,
                visible: false,
            },
        };
        let mut replica = TerminalSurfaceReplica::default();
        let before = replica
            .apply(SurfaceUpdate::Full(full))
            .expect("full applies");
        let before_fingerprint = rasterize(before).expect("full rasterizes").fingerprint();
        let changed = ChangedCells {
            epoch: 3,
            extent: SurfaceExtent {
                columns: 2,
                rows: 1,
            },
            cells: vec![ResolvedCell {
                column: 0,
                row: 0,
                content: CellContent::grapheme("A"),
                style: ResolvedCellStyle {
                    foreground: SurfaceColor::Named(NamedSurfaceColor::Black),
                    background: SurfaceColor::Indexed(24),
                    emphasis: SurfaceEmphasis {
                        reversed: true,
                        ..SurfaceEmphasis::default()
                    },
                },
            }],
            cursor: CursorState {
                column: 0,
                row: 0,
                visible: false,
            },
        };
        let after = replica
            .apply(SurfaceUpdate::Delta(changed))
            .expect("style delta applies");
        let after_fingerprint = rasterize(after).expect("delta rasterizes").fingerprint();
        assert_ne!(before_fingerprint, after_fingerprint);
    }

    #[test]
    fn cache_reuses_the_font_and_records_explicit_full_invalidations() {
        let full = crate::FullFrame {
            epoch: 3,
            extent: SurfaceExtent {
                columns: 2,
                rows: 1,
            },
            cells: vec![ResolvedCell {
                column: 0,
                row: 0,
                content: CellContent::grapheme("A"),
                style: ResolvedCellStyle::default(),
            }],
            cursor: CursorState {
                column: 0,
                row: 0,
                visible: false,
            },
        };
        let mut replica = TerminalSurfaceReplica::default();
        let before = replica
            .apply(SurfaceUpdate::Full(full))
            .expect("full applies")
            .clone();
        let mut cache = TerminalSurfaceRasterCache::new().expect("font provider loads once");
        let before_fingerprint = cache
            .rasterize(&before)
            .expect("surface rasterizes")
            .fingerprint();
        let repeated_fingerprint = cache
            .rasterize(&before)
            .expect("unchanged surface reuses raster")
            .fingerprint();
        assert_eq!(before_fingerprint, repeated_fingerprint);
        assert_eq!(
            cache.observations(),
            TerminalSurfaceRasterCacheObservations {
                font_provider_loads: 1,
                rasterizations: 1,
                cache_hits: 1,
                full_invalidations: 0,
            }
        );

        let changed = ChangedCells {
            epoch: 3,
            extent: SurfaceExtent {
                columns: 2,
                rows: 1,
            },
            cells: vec![ResolvedCell {
                column: 0,
                row: 0,
                content: CellContent::grapheme("A"),
                style: ResolvedCellStyle {
                    foreground: SurfaceColor::Named(NamedSurfaceColor::Black),
                    ..ResolvedCellStyle::default()
                },
            }],
            cursor: CursorState {
                column: 0,
                row: 0,
                visible: false,
            },
        };
        let after = replica
            .apply(SurfaceUpdate::Delta(changed))
            .expect("style delta applies")
            .clone();
        let after_fingerprint = cache
            .rasterize(&after)
            .expect("changed surface rasterizes")
            .fingerprint();
        assert_ne!(before_fingerprint, after_fingerprint);
        assert_eq!(
            cache.observations(),
            TerminalSurfaceRasterCacheObservations {
                font_provider_loads: 1,
                rasterizations: 2,
                cache_hits: 1,
                full_invalidations: 1,
            }
        );
    }

    #[cfg(feature = "ratatui-producer")]
    #[test]
    fn ratatui_resolved_cells_reach_the_same_bounded_cpu_surface() {
        let frame = crate::ratatui_producer::render_fixture(
            9,
            SurfaceExtent {
                columns: 24,
                rows: 6,
            },
            "READY",
        )
        .expect("Ratatui should produce the bounded fixture");
        let surface = TerminalSurfaceObservation::from_full(frame).expect("frame should apply");
        let raster = rasterize(&surface).expect("resolved Ratatui cells should rasterize");

        assert_eq!(raster.width, 24 * CELL_PIXEL_WIDTH);
        assert_eq!(raster.height, 6 * CELL_PIXEL_HEIGHT);
        assert!(raster
            .rgba
            .chunks_exact(4)
            .any(|pixel| pixel == [17, 168, 205, 255]));
    }
}
