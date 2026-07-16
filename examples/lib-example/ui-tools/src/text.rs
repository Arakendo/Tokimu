use crate::{UiLabelAnchor, UiRect};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum UiTextDirection {
    Ltr,
    Rtl,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum UiTextRole {
    Title,
    Heading,
    Body,
    Caption,
    Button,
    Chip,
    Status,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum UiTextAlign {
    Start,
    Center,
    End,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum UiTextOverflow {
    Clip,
    Ellipsis,
    Wrap,
}

#[derive(Clone, Debug, PartialEq)]
pub struct UiTextSpec {
    pub text: String,
    pub rect: UiRect,
    pub role: UiTextRole,
    pub direction: UiTextDirection,
    pub align_x: UiTextAlign,
    pub align_y: UiTextAlign,
    pub overflow: UiTextOverflow,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct UiGlyphQuad {
    pub center: [f32; 2],
    pub size: [f32; 2],
}

impl UiTextSpec {
    pub fn new(text: impl Into<String>, rect: UiRect, role: UiTextRole) -> Self {
        Self {
            text: text.into(),
            rect,
            role,
            direction: UiTextDirection::Ltr,
            align_x: UiTextAlign::Center,
            align_y: UiTextAlign::Center,
            overflow: UiTextOverflow::Clip,
        }
    }

    pub fn with_direction(mut self, direction: UiTextDirection) -> Self {
        self.direction = direction;
        self
    }

    pub fn with_alignment(mut self, align_x: UiTextAlign, align_y: UiTextAlign) -> Self {
        self.align_x = align_x;
        self.align_y = align_y;
        self
    }

    pub fn with_overflow(mut self, overflow: UiTextOverflow) -> Self {
        self.overflow = overflow;
        self
    }

    pub fn centered_bounds(&self) -> [f32; 2] {
        self.rect.center
    }
}

pub fn layout_bitmap_text(spec: &UiTextSpec, height: f32) -> Vec<UiGlyphQuad> {
    let cell = bitmap_cell(height);
    let width = measure_bitmap_text_width(&spec.text, height);
    let ink_width = bitmap_ink_width(&spec.text, cell, width);
    let glyph_height = bitmap_glyph_height(height);
    let rect = spec.rect;
    let start_x = match spec.align_x {
        UiTextAlign::Start => rect.center[0] - rect.size[0] * 0.5 + cell * 0.5,
        UiTextAlign::Center => rect.center[0] - ink_width * 0.5 + cell * 0.5,
        UiTextAlign::End => rect.center[0] + rect.size[0] * 0.5 - ink_width + cell * 0.5,
    };
    let top_y = match spec.align_y {
        UiTextAlign::Start => rect.center[1] + rect.size[1] * 0.5 - cell * 0.5,
        UiTextAlign::Center => rect.center[1] + glyph_height * 0.5 - cell * 0.5,
        UiTextAlign::End => rect.center[1] - rect.size[1] * 0.5 + glyph_height - cell * 0.5,
    };
    let mut x_cursor = start_x;
    let mut quads = Vec::new();

    for ch in spec.text.chars() {
        if ch == ' ' {
            x_cursor += bitmap_space_advance(cell);
            continue;
        }

        for (row_index, row_bits) in bitmap_glyph_rows(ch).into_iter().enumerate() {
            for column in 0..5 {
                let mask = 1 << (4 - column);
                if row_bits & mask == 0 {
                    continue;
                }

                let center = [
                    x_cursor + column as f32 * cell,
                    top_y - row_index as f32 * cell,
                ];
                let quad = UiGlyphQuad {
                    center,
                    // Keep adjacent bitmap cells visually connected at this scale.
                    size: [cell, cell],
                };

                if should_emit_quad(spec, quad) {
                    quads.push(quad);
                }
            }
        }

        x_cursor += bitmap_glyph_advance(cell);
    }

    quads
}

pub fn measure_bitmap_text_width(text: &str, height: f32) -> f32 {
    let cell = bitmap_cell(height);
    text.chars().fold(0.0, |width, ch| {
        width
            + if ch == ' ' {
                bitmap_space_advance(cell)
            } else {
                bitmap_glyph_advance(cell)
            }
    })
}

fn bitmap_ink_width(text: &str, cell: f32, advance_width: f32) -> f32 {
    if text.is_empty() {
        0.0
    } else {
        // The final advance includes the half-cell tracking after the last
        // glyph. Alignment should use the visible ink, not that trailing gap.
        (advance_width - cell * 0.5).max(0.0)
    }
}

pub fn bitmap_glyph_height(height: f32) -> f32 {
    bitmap_cell(height) * 7.0
}

fn should_emit_quad(spec: &UiTextSpec, quad: UiGlyphQuad) -> bool {
    if spec.rect.size == [0.0, 0.0] {
        return true;
    }

    match spec.overflow {
        UiTextOverflow::Clip | UiTextOverflow::Ellipsis => spec.rect.contains(quad.center),
        UiTextOverflow::Wrap => true,
    }
}

fn bitmap_cell(height: f32) -> f32 {
    (height / 9.0).max(0.0025)
}

fn bitmap_glyph_advance(cell: f32) -> f32 {
    cell * 5.5
}

fn bitmap_space_advance(cell: f32) -> f32 {
    cell * 3.6
}

fn bitmap_glyph_rows(ch: char) -> [u8; 7] {
    match ch.to_ascii_uppercase() {
        'A' => [
            0b01110, 0b10001, 0b10001, 0b11111, 0b10001, 0b10001, 0b10001,
        ],
        'B' => [
            0b11110, 0b10001, 0b10001, 0b11110, 0b10001, 0b10001, 0b11110,
        ],
        'C' => [
            0b01111, 0b10000, 0b10000, 0b10000, 0b10000, 0b10000, 0b01111,
        ],
        'D' => [
            0b11110, 0b10001, 0b10001, 0b10001, 0b10001, 0b10001, 0b11110,
        ],
        'E' => [
            0b11111, 0b10000, 0b10000, 0b11110, 0b10000, 0b10000, 0b11111,
        ],
        'F' => [
            0b11111, 0b10000, 0b10000, 0b11110, 0b10000, 0b10000, 0b10000,
        ],
        'G' => [
            0b01111, 0b10000, 0b10000, 0b10011, 0b10001, 0b10001, 0b01111,
        ],
        'H' => [
            0b10001, 0b10001, 0b10001, 0b11111, 0b10001, 0b10001, 0b10001,
        ],
        'I' => [
            0b11111, 0b00100, 0b00100, 0b00100, 0b00100, 0b00100, 0b11111,
        ],
        'J' => [
            0b00001, 0b00001, 0b00001, 0b00001, 0b10001, 0b10001, 0b01110,
        ],
        'K' => [
            0b10001, 0b10010, 0b10100, 0b11000, 0b10100, 0b10010, 0b10001,
        ],
        'L' => [
            0b10000, 0b10000, 0b10000, 0b10000, 0b10000, 0b10000, 0b11111,
        ],
        'M' => [
            0b10001, 0b11011, 0b10101, 0b10101, 0b10001, 0b10001, 0b10001,
        ],
        'N' => [
            0b10001, 0b11001, 0b10101, 0b10011, 0b10001, 0b10001, 0b10001,
        ],
        'O' => [
            0b01110, 0b10001, 0b10001, 0b10001, 0b10001, 0b10001, 0b01110,
        ],
        'P' => [
            0b11110, 0b10001, 0b10001, 0b11110, 0b10000, 0b10000, 0b10000,
        ],
        'Q' => [
            0b01110, 0b10001, 0b10001, 0b10001, 0b10101, 0b10010, 0b01101,
        ],
        'R' => [
            0b11110, 0b10001, 0b10001, 0b11110, 0b10100, 0b10010, 0b10001,
        ],
        'S' => [
            0b01111, 0b10000, 0b10000, 0b01110, 0b00001, 0b00001, 0b11110,
        ],
        'T' => [
            0b11111, 0b00100, 0b00100, 0b00100, 0b00100, 0b00100, 0b00100,
        ],
        'U' => [
            0b10001, 0b10001, 0b10001, 0b10001, 0b10001, 0b10001, 0b01110,
        ],
        'V' => [
            0b10001, 0b10001, 0b10001, 0b10001, 0b01010, 0b01010, 0b00100,
        ],
        'W' => [
            0b10001, 0b10001, 0b10001, 0b10101, 0b10101, 0b11011, 0b10001,
        ],
        'X' => [
            0b10001, 0b01010, 0b00100, 0b00100, 0b00100, 0b01010, 0b10001,
        ],
        'Y' => [
            0b10001, 0b01010, 0b00100, 0b00100, 0b00100, 0b00100, 0b00100,
        ],
        'Z' => [
            0b11111, 0b00010, 0b00100, 0b00100, 0b01000, 0b10000, 0b11111,
        ],
        '+' => [
            0b00100, 0b00100, 0b00100, 0b11111, 0b00100, 0b00100, 0b00100,
        ],
        '?' => [
            0b01110, 0b10001, 0b00001, 0b00010, 0b00100, 0b00000, 0b00100,
        ],
        _ => [
            0b01110, 0b10001, 0b00001, 0b00010, 0b00100, 0b00000, 0b00100,
        ],
    }
}

impl From<UiLabelAnchor> for UiTextAlign {
    fn from(anchor: UiLabelAnchor) -> Self {
        match anchor {
            UiLabelAnchor::Start => Self::Start,
            UiLabelAnchor::Center => Self::Center,
            UiLabelAnchor::End => Self::End,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bitmap_layout_respects_center_alignment() {
        let spec = UiTextSpec::new("AA", UiRect::new([0.0, 0.0], [0.6, 0.2]), UiTextRole::Body)
            .with_alignment(UiTextAlign::Center, UiTextAlign::Center);
        let quads = layout_bitmap_text(&spec, 0.09);

        let min_x = quads
            .iter()
            .map(|quad| quad.center[0] - quad.size[0] * 0.5)
            .fold(f32::INFINITY, f32::min);
        let max_x = quads
            .iter()
            .map(|quad| quad.center[0] + quad.size[0] * 0.5)
            .fold(f32::NEG_INFINITY, f32::max);

        assert!(min_x < 0.0);
        assert!(max_x > 0.0);
    }

    #[test]
    fn bitmap_layout_uses_visible_ink_for_end_alignment() {
        let spec = UiTextSpec::new("AA", UiRect::new([0.0, 0.0], [0.4, 0.2]), UiTextRole::Body)
            .with_alignment(UiTextAlign::End, UiTextAlign::Center);
        let quads = layout_bitmap_text(&spec, 0.09);
        let max_x = quads
            .iter()
            .map(|quad| quad.center[0] + quad.size[0] * 0.5)
            .fold(f32::NEG_INFINITY, f32::max);
        let right = spec.rect.center[0] + spec.rect.size[0] * 0.5;

        assert!((max_x - right).abs() < 0.01);
    }

    #[test]
    fn bitmap_layout_clips_to_nonzero_rect() {
        let spec = UiTextSpec::new(
            "AAAAAA",
            UiRect::new([0.0, 0.0], [0.12, 0.08]),
            UiTextRole::Caption,
        )
        .with_alignment(UiTextAlign::Start, UiTextAlign::Center)
        .with_overflow(UiTextOverflow::Clip);
        let quads = layout_bitmap_text(&spec, 0.06);

        assert!(!quads.is_empty());
        assert!(quads.iter().all(|quad| spec.rect.contains(quad.center)));
    }

    #[test]
    fn bitmap_layout_keeps_start_aligned_glyphs_inside_bounds() {
        let spec = UiTextSpec::new("A", UiRect::new([0.0, 0.0], [0.12, 0.12]), UiTextRole::Body)
            .with_alignment(UiTextAlign::Start, UiTextAlign::Start)
            .with_overflow(UiTextOverflow::Clip);
        let quads = layout_bitmap_text(&spec, 0.06);

        assert!(!quads.is_empty());
        assert!(quads.iter().all(|quad| {
            let left = quad.center[0] - quad.size[0] * 0.5;
            let top = quad.center[1] + quad.size[1] * 0.5;
            left >= spec.rect.center[0] - spec.rect.size[0] * 0.5
                && top <= spec.rect.center[1] + spec.rect.size[1] * 0.5
        }));
    }
}
