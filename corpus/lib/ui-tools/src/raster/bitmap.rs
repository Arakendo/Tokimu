use super::{UiFontRasterizer, UiRasterTextBitmap, UiTextMetrics};

pub fn alpha_to_rgba8(alpha: &[u8], color: [u8; 3]) -> Vec<u8> {
    alpha
        .iter()
        .flat_map(|coverage| [color[0], color[1], color[2], *coverage])
        .collect()
}

impl UiFontRasterizer {
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
