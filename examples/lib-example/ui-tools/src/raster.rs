use ab_glyph::{point, Font, FontArc, PxScale, ScaleFont};

#[derive(Clone, Debug, PartialEq)]
pub struct UiRasterGlyph {
    pub character: char,
    pub width: u32,
    pub height: u32,
    pub left: f32,
    pub top: f32,
    pub bottom: f32,
    /// Pixel-space baseline used by the font's scaled coordinate system.
    pub baseline: f32,
    /// Horizontal bitmap bearing from the glyph pen position.
    pub bearing_x: f32,
    /// Vertical bitmap bearing from the baseline to the bitmap's top edge.
    pub bearing_y: f32,
    pub ascent: f32,
    pub descent: f32,
    pub advance: f32,
    pub alpha: Vec<u8>,
}

pub struct UiFontRasterizer {
    pub(crate) font: FontArc,
    // Outline adapters need the original provider bytes so they can preserve
    // move/close commands that the rasterizer-oriented API intentionally hides.
    pub(crate) font_bytes: Vec<u8>,
}

pub fn alpha_to_rgba8(alpha: &[u8], color: [u8; 3]) -> Vec<u8> {
    alpha
        .iter()
        .flat_map(|coverage| [color[0], color[1], color[2], *coverage])
        .collect()
}

impl UiFontRasterizer {
    pub fn from_bytes(bytes: Vec<u8>) -> Result<Self, ab_glyph::InvalidFont> {
        Ok(Self {
            font: FontArc::try_from_vec(bytes.clone())?,
            font_bytes: bytes,
        })
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

    /// Rasterizes a complete line into one baseline-aligned bitmap.
    pub fn rasterize_text(&self, text: &str, pixels: f32) -> UiRasterTextBitmap {
        let layout = self.layout(text, pixels);
        let mut min_x = 0.0_f32;
        let mut max_x = layout.width;
        let mut min_y = 0.0_f32;
        let mut max_y = 0.0_f32;
        for positioned in &layout.glyphs {
            let glyph = &positioned.glyph;
            min_x = min_x.min(positioned.pen_x + glyph.bearing_x);
            max_x = max_x.max(positioned.pen_x + glyph.bearing_x + glyph.width as f32);
            min_y = min_y.min(glyph.bearing_y);
            max_y = max_y.max(glyph.bearing_y + glyph.height as f32);
        }
        let origin_x = min_x.floor();
        let origin_y = min_y.floor();
        let width = (max_x.ceil() - origin_x).max(0.0) as u32;
        let height = (max_y.ceil() - origin_y).max(0.0) as u32;
        let mut alpha = vec![0; (width * height) as usize];
        for positioned in layout.glyphs {
            let glyph = positioned.glyph;
            let x0 = (positioned.pen_x + glyph.bearing_x - origin_x).round() as i32;
            let y0 = (glyph.bearing_y - origin_y).round() as i32;
            for y in 0..glyph.height as i32 {
                for x in 0..glyph.width as i32 {
                    let source = (y * glyph.width as i32 + x) as usize;
                    let target_x = x0 + x;
                    let target_y = y0 + y;
                    if target_x < 0
                        || target_y < 0
                        || target_x >= width as i32
                        || target_y >= height as i32
                    {
                        continue;
                    }
                    let target = (target_y as u32 * width + target_x as u32) as usize;
                    alpha[target] = alpha[target].max(glyph.alpha[source]);
                }
            }
        }
        UiRasterTextBitmap {
            width,
            height,
            left: origin_x,
            top: origin_y,
            baseline: 0.0,
            ascent: layout.ascent,
            descent: layout.descent,
            alpha,
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct UiRasterTextGlyph {
    pub glyph: UiRasterGlyph,
    pub pen_x: f32,
}

#[derive(Clone, Debug, PartialEq)]
pub struct UiRasterText {
    pub glyphs: Vec<UiRasterTextGlyph>,
    pub width: f32,
    pub ascent: f32,
    pub descent: f32,
}

#[derive(Clone, Debug, PartialEq)]
pub struct UiRasterTextBlock {
    pub lines: Vec<UiRasterText>,
    pub baselines: Vec<f32>,
    pub line_gap: f32,
    pub width: f32,
}

impl UiFontRasterizer {
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

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct UiTextMetrics {
    pub width: f32,
    pub ascent: f32,
    pub descent: f32,
    pub line_gap: f32,
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

impl UiRasterTextBitmap {
    pub fn metrics(&self) -> UiTextMetrics {
        UiTextMetrics {
            width: self.width as f32,
            ascent: self.ascent,
            descent: self.descent,
            line_gap: 0.0,
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct UiRasterTextBitmap {
    pub width: u32,
    pub height: u32,
    /// Pixel-space visible-ink origin relative to the baseline.
    pub left: f32,
    pub top: f32,
    pub ascent: f32,
    pub descent: f32,
    pub baseline: f32,
    pub alpha: Vec<u8>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rasterized_glyph_has_coverage_and_advance() {
        let bytes =
            std::fs::read("../../../../target/glyph-corpus/fonts/inter/Inter[opsz,wght].ttf")
                .or_else(|_| {
                    std::fs::read("../../../../target/glyph-corpus/fonts/inter/Inter-Regular.ttf")
                });
        let Ok(bytes) = bytes else {
            return;
        };
        let rasterizer = UiFontRasterizer::from_bytes(bytes).unwrap();
        let glyph = rasterizer.rasterize('A', 32.0);
        assert!(glyph.advance > 0.0);
        assert!(glyph.alpha.iter().any(|coverage| *coverage > 0));
        assert!(glyph.alpha.iter().any(|coverage| *coverage == 0));
    }

    #[test]
    fn rasterized_text_reports_visible_ink_metrics() {
        let bytes =
            std::fs::read("../../../../target/glyph-corpus/fonts/inter/Inter[opsz,wght].ttf")
                .or_else(|_| {
                    std::fs::read("../../../../target/glyph-corpus/fonts/inter/Inter-Regular.ttf")
                });
        let Ok(bytes) = bytes else {
            return;
        };
        let rasterizer = UiFontRasterizer::from_bytes(bytes).unwrap();
        let text = rasterizer.rasterize_text("Ag", 32.0);

        assert!(text.width > 0);
        assert!(text.height > 0);
        assert!(text.ascent > 0.0);
        assert!(text.descent < 0.0);
        assert!(text.left <= 0.0);
        assert!(text.top < text.ascent);
        assert_eq!(text.baseline, 0.0);

        let metrics = text.metrics();
        assert_eq!(metrics.width, text.width as f32);
        assert_eq!(metrics.line_gap, 0.0);
    }

    #[test]
    fn tracking_changes_placement_without_changing_glyph_metrics() {
        let bytes =
            std::fs::read("../../../../target/glyph-corpus/fonts/inter/Inter[opsz,wght].ttf")
                .or_else(|_| {
                    std::fs::read("../../../../target/glyph-corpus/fonts/inter/Inter-Regular.ttf")
                });
        let Ok(bytes) = bytes else {
            return;
        };
        let rasterizer = UiFontRasterizer::from_bytes(bytes).unwrap();
        let normal = rasterizer.layout("AA", 32.0);
        let tracked = rasterizer.layout_with_tracking("AA", 32.0, 3.0);

        assert!(tracked.width > normal.width);
        assert_eq!(
            tracked.glyphs[0].glyph.advance,
            normal.glyphs[0].glyph.advance
        );
        assert_eq!(
            tracked.glyphs[1].glyph.advance,
            normal.glyphs[1].glyph.advance
        );
        assert_eq!(tracked.glyphs[0].pen_x, normal.glyphs[0].pen_x);
        assert_eq!(tracked.glyphs[1].pen_x, normal.glyphs[1].pen_x + 3.0);
    }

    #[test]
    fn multiline_layout_preserves_explicit_leading() {
        let bytes =
            std::fs::read("../../../../target/glyph-corpus/fonts/inter/Inter[opsz,wght].ttf")
                .or_else(|_| {
                    std::fs::read("../../../../target/glyph-corpus/fonts/inter/Inter-Regular.ttf")
                });
        let Ok(bytes) = bytes else {
            return;
        };
        let rasterizer = UiFontRasterizer::from_bytes(bytes).unwrap();
        let block = rasterizer.layout_lines(&["A", "g"], 32.0, 4.0);

        assert_eq!(block.lines.len(), 2);
        assert_eq!(block.line_gap, 4.0);
        assert!(block.baselines[0] > block.baselines[1]);
        assert!(block.width > 0.0);
    }

    #[test]
    fn checked_in_noto_fixture_is_loadable_without_prepared_corpus() {
        let bytes = include_bytes!("../fixtures/NotoSans-Regular.otf").to_vec();
        let rasterizer = UiFontRasterizer::from_bytes(bytes).expect("checked-in OTF fixture");
        let bitmap = rasterizer.rasterize_text("Noto 0123", 24.0);

        assert!(bitmap.width > 0);
        assert!(bitmap.height > 0);
        assert!(!bitmap.alpha.is_empty());
    }
}
