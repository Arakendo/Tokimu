use ab_glyph::{point, Font, FontArc, PxScale, ScaleFont};

use super::UiRasterGlyph;

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
