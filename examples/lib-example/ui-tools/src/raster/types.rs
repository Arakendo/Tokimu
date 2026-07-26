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

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct UiTextMetrics {
    pub width: f32,
    pub ascent: f32,
    pub descent: f32,
    pub line_gap: f32,
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
