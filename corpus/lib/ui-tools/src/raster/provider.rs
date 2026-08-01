use ab_glyph::{point, Font, FontArc, PxScale, ScaleFont};

use super::UiRasterGlyph;
use crate::{UiRect, UiTextDiagnostic, UiTextDiagnosticKind, UiTextMeasure, UiTextMetricsProvider};

pub struct UiFontRasterizer {
    pub(crate) font: FontArc,
    // Outline adapters need the original provider bytes so they can preserve
    // move/close commands that the rasterizer-oriented API intentionally hides.
    pub(crate) font_bytes: Vec<u8>,
}

impl UiFontRasterizer {
    pub fn from_bytes(bytes: Vec<u8>) -> Result<Self, ab_glyph::InvalidFont> {
        Ok(Self {
            font: FontArc::try_from_vec(bytes.clone())?,
            font_bytes: bytes,
        })
    }

    /// Adapts this concrete font provider to provider-neutral text metrics.
    pub fn metrics_provider(&self, pixels: f32) -> UiRasterTextMetricsProvider<'_> {
        UiRasterTextMetricsProvider {
            rasterizer: self,
            pixels,
        }
    }

    pub fn rasterize(&self, character: char, pixels: f32) -> UiRasterGlyph {
        let scaled = self.font.as_scaled(PxScale::from(pixels));
        let glyph = self
            .font
            .glyph_id(character)
            .with_scale_and_position(PxScale::from(pixels), point(0.0, 0.0));
        let advance = scaled.h_advance(glyph.id);
        let mut width = 0;
        let mut height = 0;
        let mut left = 0.0;
        let mut top = 0.0;
        let mut bottom = 0.0;
        let mut bearing_x = 0.0;
        let mut bearing_y = 0.0;
        let mut alpha = Vec::new();
        if let Some(outlined) = self.font.outline_glyph(glyph) {
            let bounds = outlined.px_bounds();
            // Keep these dimensions identical to ab_glyph's draw callback.
            // Its internal rasterizer uses truncation, so rounding up here
            // would give the callback and our texture different row strides.
            width = bounds.width().max(0.0) as u32;
            height = bounds.height().max(0.0) as u32;
            left = bounds.min.x;
            top = bounds.min.y;
            bottom = bounds.max.y;
            bearing_x = bounds.min.x;
            bearing_y = bounds.min.y;
            alpha = vec![0; (width * height) as usize];
            outlined.draw(|x, y, coverage| {
                let index = (y * width + x) as usize;
                if let Some(pixel) = alpha.get_mut(index) {
                    *pixel = (coverage * 255.0).round() as u8;
                }
            });
        }
        UiRasterGlyph {
            character,
            width,
            height,
            left,
            top,
            bottom,
            baseline: 0.0,
            bearing_x,
            bearing_y,
            ascent: scaled.ascent(),
            descent: scaled.descent(),
            advance,
            alpha,
        }
    }
}

/// Fixed-size provider-neutral metrics view over a TTF/OTF rasterizer.
pub struct UiRasterTextMetricsProvider<'a> {
    rasterizer: &'a UiFontRasterizer,
    pixels: f32,
}

impl UiRasterTextMetricsProvider<'_> {
    pub fn pixels(&self) -> f32 {
        self.pixels
    }
}

impl UiTextMetricsProvider for UiRasterTextMetricsProvider<'_> {
    fn measure(&self, text: &str) -> Result<UiTextMeasure, UiTextDiagnostic> {
        if !self.pixels.is_finite() || self.pixels <= 0.0 {
            return Err(UiTextDiagnostic {
                kind: UiTextDiagnosticKind::ProviderUnavailable,
            });
        }

        let lines = text.lines().collect::<Vec<_>>();
        let layouts = lines
            .iter()
            .map(|line| self.rasterizer.layout(line, self.pixels))
            .collect::<Vec<_>>();
        let line_count = layouts.len().max(1);
        let ascent = layouts.first().map(|line| line.ascent).unwrap_or(0.0);
        let descent = layouts.first().map(|line| line.descent).unwrap_or(0.0);
        let line_height = (ascent - descent).max(0.0);
        let mut bounds: Option<[f32; 4]> = None;
        for (line_index, layout) in layouts.iter().enumerate() {
            let baseline_offset = line_index as f32 * line_height;
            for glyph in &layout.glyphs {
                if glyph.glyph.width == 0 || glyph.glyph.height == 0 {
                    continue;
                }
                let left = glyph.pen_x + glyph.glyph.left;
                let right = left + glyph.glyph.width as f32;
                let top = glyph.glyph.top + baseline_offset;
                let bottom = glyph.glyph.bottom + baseline_offset;
                bounds = Some(match bounds {
                    Some([min_x, min_y, max_x, max_y]) => [
                        min_x.min(left),
                        min_y.min(top),
                        max_x.max(right),
                        max_y.max(bottom),
                    ],
                    None => [left, top, right, bottom],
                });
            }
        }
        let visible_bounds = bounds.map(|[left, top, right, bottom]| {
            UiRect::new(
                [(left + right) * 0.5, (top + bottom) * 0.5],
                [(right - left).max(0.0), (bottom - top).max(0.0)],
            )
        });

        Ok(UiTextMeasure {
            advance: layouts.iter().map(|line| line.width).fold(0.0, f32::max),
            ascent: ascent + line_count.saturating_sub(1) as f32 * line_height,
            descent,
            line_gap: 0.0,
            visible_bounds,
            diagnostics: Vec::new(),
        })
    }
}
