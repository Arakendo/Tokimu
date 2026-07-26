use ab_glyph::{Font, PxScale, ScaleFont};

use super::{UiFontRasterizer, UiRasterText, UiRasterTextBlock, UiRasterTextGlyph, UiTextMetrics};

impl UiFontRasterizer {
    /// Layout glyphs on one shared baseline using font advances.
    pub fn layout(&self, text: &str, pixels: f32) -> UiRasterText {
        self.layout_with_tracking(text, pixels, 0.0)
    }

    /// Layout glyphs with explicit tracking added between adjacent glyphs.
    /// Tracking is presentation policy; it never changes provider advances.
    pub fn layout_with_tracking(&self, text: &str, pixels: f32, tracking: f32) -> UiRasterText {
        let scaled = self.font.as_scaled(PxScale::from(pixels));
        let mut pen_x = 0.0;
        let mut glyphs = Vec::new();
        let characters = text.chars().collect::<Vec<_>>();
        for (index, character) in characters.iter().copied().enumerate() {
            let glyph = self.rasterize(character, pixels);
            glyphs.push(UiRasterTextGlyph { glyph, pen_x });
            pen_x += scaled.h_advance(self.font.glyph_id(character));
            if index + 1 < characters.len() {
                pen_x += tracking;
            }
        }
        UiRasterText {
            glyphs,
            width: pen_x,
            ascent: scaled.ascent(),
            descent: scaled.descent(),
        }
    }

    /// Layouts independent lines with explicit leading between baselines.
    pub fn layout_lines(&self, lines: &[&str], pixels: f32, line_gap: f32) -> UiRasterTextBlock {
        let layouts = lines
            .iter()
            .map(|line| self.layout(line, pixels))
            .collect::<Vec<_>>();
        let line_height = layouts
            .first()
            .map(|line| line.ascent - line.descent + line_gap)
            .unwrap_or(line_gap);
        let baselines = (0..layouts.len())
            .map(|index| -(index as f32 * line_height))
            .collect::<Vec<_>>();
        let width = layouts.iter().map(|line| line.width).fold(0.0, f32::max);

        UiRasterTextBlock {
            lines: layouts,
            baselines,
            line_gap,
            width,
        }
    }
}

impl UiRasterText {
    pub fn metrics(&self) -> UiTextMetrics {
        UiTextMetrics {
            width: self.width,
            ascent: self.ascent,
            descent: self.descent,
            // Single-line rasterization has no provider-independent leading yet.
            line_gap: 0.0,
        }
    }
}
