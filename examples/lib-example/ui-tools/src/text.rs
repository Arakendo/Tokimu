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

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum UiTextAlignmentBasis {
    /// Align using the logical advance box, including trailing spacing.
    Advance,
    /// Align using the visible bitmap ink, excluding trailing spacing.
    VisibleInk,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum UiTextOverflow {
    Clip,
    Ellipsis,
    Wrap,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum UiTextDiagnosticKind {
    MissingGlyph { character: char },
    ProviderUnavailable,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct UiTextDiagnostic {
    pub kind: UiTextDiagnosticKind,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum UiMissingGlyphPolicy {
    Replace(char),
    Skip,
    Report,
}

/// Provider-neutral measurements for one laid-out text request.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct UiTextMeasure {
    pub advance: f32,
    pub ascent: f32,
    pub descent: f32,
    pub line_gap: f32,
    pub visible_bounds: Option<UiRect>,
    pub diagnostics: Vec<UiTextDiagnostic>,
}

/// A provider can expose metrics without loading a rasterizer or renderer.
pub trait UiTextMetricsProvider {
    fn measure(&self, text: &str) -> Result<UiTextMeasure, UiTextDiagnostic>;
}

/// The placement contract for one logical line.
#[derive(Clone, Debug, PartialEq)]
pub struct UiTextLineLayout {
    pub text: String,
    pub origin: [f32; 2],
    pub advance: f32,
    pub baseline: f32,
}

/// Provider-neutral result consumed by native, headless, or diagnostic clients.
#[derive(Clone, Debug, PartialEq)]
pub struct UiTextLayout {
    pub measure: UiTextMeasure,
    pub lines: Vec<UiTextLineLayout>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct UiTextSpec {
    pub text: String,
    pub rect: UiRect,
    pub role: UiTextRole,
    pub direction: UiTextDirection,
    pub align_x: UiTextAlign,
    pub align_y: UiTextAlign,
    pub alignment_basis: UiTextAlignmentBasis,
    pub overflow: UiTextOverflow,
    pub missing_glyph: UiMissingGlyphPolicy,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct UiGlyphQuad {
    pub center: [f32; 2],
    pub size: [f32; 2],
}

#[derive(Clone, Debug, PartialEq)]
pub struct UiTextLayoutReport {
    pub text: String,
    pub line_count: usize,
    pub glyph_count: usize,
    pub visible_bounds: Option<UiRect>,
}

impl UiTextLayout {
    pub fn line_count(&self) -> usize {
        self.lines.len()
    }
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
            alignment_basis: UiTextAlignmentBasis::VisibleInk,
            overflow: UiTextOverflow::Clip,
            missing_glyph: UiMissingGlyphPolicy::Replace('?'),
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

    pub fn with_alignment_basis(mut self, basis: UiTextAlignmentBasis) -> Self {
        self.alignment_basis = basis;
        self
    }

    pub fn with_overflow(mut self, overflow: UiTextOverflow) -> Self {
        self.overflow = overflow;
        self
    }

    pub fn with_missing_glyph_policy(mut self, policy: UiMissingGlyphPolicy) -> Self {
        self.missing_glyph = policy;
        self
    }

    pub fn centered_bounds(&self) -> [f32; 2] {
        self.rect.center
    }

    pub fn headless_report(&self, height: f32) -> UiTextLayoutReport {
        let quads = layout_bitmap_text(self, height);
        let visible_bounds = quads.first().map(|_| {
            let (min_x, max_x, min_y, max_y) = quads.iter().fold(
                (
                    f32::INFINITY,
                    f32::NEG_INFINITY,
                    f32::INFINITY,
                    f32::NEG_INFINITY,
                ),
                |(min_x, max_x, min_y, max_y), quad| {
                    let half_width = quad.size[0] * 0.5;
                    let half_height = quad.size[1] * 0.5;
                    (
                        min_x.min(quad.center[0] - half_width),
                        max_x.max(quad.center[0] + half_width),
                        min_y.min(quad.center[1] - half_height),
                        max_y.max(quad.center[1] + half_height),
                    )
                },
            );
            UiRect::new(
                [(min_x + max_x) * 0.5, (min_y + max_y) * 0.5],
                [max_x - min_x, max_y - min_y],
            )
        });

        UiTextLayoutReport {
            text: self.text.clone(),
            line_count: self.text.lines().count(),
            glyph_count: quads.len(),
            visible_bounds,
        }
    }

    /// Produces the provider-neutral line contract for the bitmap proof path.
    pub fn bitmap_layout(&self, height: f32) -> UiTextLayout {
        let lines = text_lines(self, height);
        let line_height = bitmap_cell(height) * 9.0;
        let measure = UiTextMeasure {
            advance: lines
                .iter()
                .map(|line| measure_bitmap_text_width(line, height))
                .fold(0.0, f32::max),
            ascent: bitmap_glyph_height(height),
            descent: 0.0,
            line_gap: line_height - bitmap_glyph_height(height),
            visible_bounds: None,
            diagnostics: Vec::new(),
        };
        let block_height = measure.ascent + line_height * lines.len().saturating_sub(1) as f32;
        let first_baseline = match self.align_y {
            UiTextAlign::Start => self.rect.center[1] + self.rect.size[1] * 0.5 - measure.ascent,
            UiTextAlign::Center => self.rect.center[1] + block_height * 0.5 - measure.ascent,
            UiTextAlign::End => {
                self.rect.center[1] - self.rect.size[1] * 0.5 + block_height - measure.ascent
            }
        };
        let align = physical_alignment(self.align_x, self.direction);
        let layouts = lines
            .into_iter()
            .enumerate()
            .map(|(index, text)| {
                let advance = measure_bitmap_text_width(&text, height);
                let origin_x = match align {
                    UiTextAlign::Start => self.rect.center[0] - self.rect.size[0] * 0.5,
                    UiTextAlign::Center => self.rect.center[0] - advance * 0.5,
                    UiTextAlign::End => self.rect.center[0] + self.rect.size[0] * 0.5 - advance,
                };
                UiTextLineLayout {
                    text,
                    origin: [origin_x, first_baseline - index as f32 * line_height],
                    advance,
                    baseline: first_baseline - index as f32 * line_height,
                }
            })
            .collect();

        UiTextLayout {
            measure,
            lines: layouts,
        }
    }
}

pub fn layout_bitmap_text(spec: &UiTextSpec, height: f32) -> Vec<UiGlyphQuad> {
    let cell = bitmap_cell(height);
    let glyph_height = bitmap_glyph_height(height);
    let rect = spec.rect;
    let mut quads = Vec::new();
    let lines = text_lines(spec, height);
    let line_height = cell * 9.0;
    let block_height = glyph_height + line_height * lines.len().saturating_sub(1) as f32;
    let first_line_top = match spec.align_y {
        UiTextAlign::Start => rect.center[1] + rect.size[1] * 0.5,
        UiTextAlign::Center => rect.center[1] + block_height * 0.5,
        UiTextAlign::End => rect.center[1] - rect.size[1] * 0.5 + block_height,
    };

    for (line_index, line) in lines.iter().enumerate() {
        let width = measure_bitmap_text_width(line, height);
        let alignment_width = match spec.alignment_basis {
            UiTextAlignmentBasis::Advance => width,
            UiTextAlignmentBasis::VisibleInk => bitmap_ink_width(line, cell, width),
        };
        let align_x = physical_alignment(spec.align_x, spec.direction);
        let start_x = match align_x {
            UiTextAlign::Start => rect.center[0] - rect.size[0] * 0.5 + cell * 0.5,
            UiTextAlign::Center => rect.center[0] - alignment_width * 0.5 + cell * 0.5,
            UiTextAlign::End => rect.center[0] + rect.size[0] * 0.5 - alignment_width + cell * 0.5,
        };
        let top_y = first_line_top - line_index as f32 * line_height - cell * 0.5;
        let mut x_cursor = start_x;

        let characters: Box<dyn Iterator<Item = char>> = match spec.direction {
            UiTextDirection::Ltr => Box::new(line.chars()),
            UiTextDirection::Rtl => Box::new(line.chars().rev()),
        };
        for ch in characters {
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
    }

    quads
}

fn physical_alignment(align: UiTextAlign, direction: UiTextDirection) -> UiTextAlign {
    match (align, direction) {
        (UiTextAlign::Start, UiTextDirection::Rtl) => UiTextAlign::End,
        (UiTextAlign::End, UiTextDirection::Rtl) => UiTextAlign::Start,
        _ => align,
    }
}

fn text_lines(spec: &UiTextSpec, height: f32) -> Vec<String> {
    if spec.overflow == UiTextOverflow::Ellipsis && spec.rect.size[0] > 0.0 {
        return spec
            .text
            .lines()
            .map(|line| truncate_with_ellipsis(line, height, spec.rect.size[0]))
            .collect();
    }

    if spec.overflow != UiTextOverflow::Wrap || spec.rect.size[0] <= 0.0 {
        return spec.text.lines().map(str::to_owned).collect();
    }

    let max_width = spec.rect.size[0];
    let mut lines = Vec::new();
    for paragraph in spec.text.lines() {
        let mut current = String::new();
        for word in paragraph.split_whitespace() {
            let candidate = if current.is_empty() {
                word.to_owned()
            } else {
                format!("{current} {word}")
            };
            if !current.is_empty() && measure_bitmap_text_width(&candidate, height) > max_width {
                lines.push(std::mem::take(&mut current));
            }

            if measure_bitmap_text_width(word, height) <= max_width {
                if current.is_empty() {
                    current = word.to_owned();
                } else {
                    current.push(' ');
                    current.push_str(word);
                }
            } else {
                for ch in word.chars() {
                    let character = ch.to_string();
                    if !current.is_empty()
                        && measure_bitmap_text_width(&format!("{current}{character}"), height)
                            > max_width
                    {
                        lines.push(std::mem::take(&mut current));
                    }
                    current.push(ch);
                }
            }
        }
        if !current.is_empty() || paragraph.is_empty() {
            lines.push(current);
        }
    }
    if lines.is_empty() {
        lines.push(String::new());
    }
    lines
}

fn truncate_with_ellipsis(text: &str, height: f32, max_width: f32) -> String {
    if measure_bitmap_text_width(text, height) <= max_width {
        return text.to_owned();
    }
    let marker = "...";
    if measure_bitmap_text_width(marker, height) > max_width {
        return String::new();
    }
    let mut result = String::new();
    for ch in text.chars() {
        let candidate = format!("{result}{ch}{marker}");
        if measure_bitmap_text_width(&candidate, height) > max_width {
            break;
        }
        result.push(ch);
    }
    format!("{result}{marker}")
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
        '0' => [
            0b01110, 0b10001, 0b10011, 0b10101, 0b11001, 0b10001, 0b01110,
        ],
        '1' => [
            0b00100, 0b01100, 0b00100, 0b00100, 0b00100, 0b00100, 0b01110,
        ],
        '2' => [
            0b01110, 0b10001, 0b00001, 0b00010, 0b00100, 0b01000, 0b11111,
        ],
        '3' => [
            0b11110, 0b00001, 0b00001, 0b01110, 0b00001, 0b00001, 0b11110,
        ],
        '4' => [
            0b00010, 0b00110, 0b01010, 0b10010, 0b11111, 0b00010, 0b00010,
        ],
        '5' => [
            0b11111, 0b10000, 0b10000, 0b11110, 0b00001, 0b00001, 0b11110,
        ],
        '6' => [
            0b00110, 0b01000, 0b10000, 0b11110, 0b10001, 0b10001, 0b01110,
        ],
        '7' => [
            0b11111, 0b00001, 0b00010, 0b00100, 0b01000, 0b01000, 0b01000,
        ],
        '8' => [
            0b01110, 0b10001, 0b10001, 0b01110, 0b10001, 0b10001, 0b01110,
        ],
        '9' => [
            0b01110, 0b10001, 0b10001, 0b01111, 0b00001, 0b00010, 0b11100,
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
    fn bitmap_layout_can_align_end_using_advance_width() {
        let spec = UiTextSpec::new("AA", UiRect::new([0.0, 0.0], [0.4, 0.2]), UiTextRole::Body)
            .with_alignment(UiTextAlign::End, UiTextAlign::Center)
            .with_alignment_basis(UiTextAlignmentBasis::Advance);
        let quads = layout_bitmap_text(&spec, 0.09);
        let max_x = quads
            .iter()
            .map(|quad| quad.center[0] + quad.size[0] * 0.5)
            .fold(f32::NEG_INFINITY, f32::max);
        let right = spec.rect.center[0] + spec.rect.size[0] * 0.5;
        let cell = bitmap_cell(0.09);

        assert!((right - max_x - cell * 0.5).abs() < 0.01);
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
    fn ellipsis_overflow_truncates_to_the_available_width() {
        let spec = UiTextSpec::new(
            "A LONG LABEL",
            UiRect::new([0.0, 0.0], [0.12, 0.08]),
            UiTextRole::Button,
        )
        .with_overflow(UiTextOverflow::Ellipsis);

        let glyphs = layout_bitmap_text(&spec, 0.03);
        assert!(!glyphs.is_empty());
        assert!(glyphs.iter().all(|glyph| spec.rect.contains(glyph.center)));
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

    #[test]
    fn bitmap_layout_wraps_words_into_multiple_lines() {
        let spec = UiTextSpec::new(
            "BUILD SETTINGS",
            UiRect::new([0.0, 0.0], [0.16, 0.4]),
            UiTextRole::Body,
        )
        .with_alignment(UiTextAlign::Start, UiTextAlign::Start)
        .with_overflow(UiTextOverflow::Wrap);
        let quads = layout_bitmap_text(&spec, 0.06);
        let distinct_rows = quads
            .iter()
            .map(|quad| (quad.center[1] * 1000.0).round() as i32)
            .collect::<std::collections::BTreeSet<_>>();

        assert!(distinct_rows.len() > 7);
        assert!(quads.iter().all(|quad| {
            quad.center[0] >= spec.rect.center[0] - spec.rect.size[0] * 0.5
                && quad.center[0] <= spec.rect.center[0] + spec.rect.size[0] * 0.5
        }));
    }

    #[test]
    fn bitmap_layout_honors_explicit_newlines() {
        let spec = UiTextSpec::new(
            "A\nB",
            UiRect::new([0.0, 0.0], [0.2, 0.4]),
            UiTextRole::Body,
        )
        .with_overflow(UiTextOverflow::Wrap);
        let quads = layout_bitmap_text(&spec, 0.06);
        let rows = quads
            .iter()
            .map(|quad| (quad.center[1] * 1000.0).round() as i32)
            .collect::<std::collections::BTreeSet<_>>();

        assert!(rows.len() > 7);
    }

    #[test]
    fn bitmap_layout_resolves_start_alignment_by_text_direction() {
        let ltr = UiTextSpec::new("AB", UiRect::new([0.0, 0.0], [0.4, 0.2]), UiTextRole::Body)
            .with_alignment(UiTextAlign::Start, UiTextAlign::Center)
            .with_direction(UiTextDirection::Ltr);
        let rtl = ltr.clone().with_direction(UiTextDirection::Rtl);
        let ltr_quads = layout_bitmap_text(&ltr, 0.06);
        let rtl_quads = layout_bitmap_text(&rtl, 0.06);
        let ltr_min = ltr_quads
            .iter()
            .map(|quad| quad.center[0] - quad.size[0] * 0.5)
            .fold(f32::INFINITY, f32::min);
        let rtl_max = rtl_quads
            .iter()
            .map(|quad| quad.center[0] + quad.size[0] * 0.5)
            .fold(f32::NEG_INFINITY, f32::max);

        assert!(ltr_min < -0.15);
        assert!(rtl_max > 0.15);
    }

    #[test]
    fn headless_layout_produces_stable_bounds_without_renderer_state() {
        let spec = UiTextSpec::new(
            "HEADLESS TEXT\nSECOND LINE",
            UiRect::new([0.0, 0.0], [0.7, 0.4]),
            UiTextRole::Body,
        )
        .with_alignment(UiTextAlign::Start, UiTextAlign::Start)
        .with_overflow(UiTextOverflow::Wrap);

        let first = layout_bitmap_text(&spec, 0.05);
        let second = layout_bitmap_text(&spec, 0.05);

        assert!(!first.is_empty());
        assert_eq!(first, second);
        assert!(first.iter().all(|quad| spec.rect.contains(quad.center)));
    }

    #[test]
    fn headless_report_describes_the_same_layout_consumed_by_rendering() {
        let spec = UiTextSpec::new(
            "REPORT\nREADY",
            UiRect::new([0.0, 0.0], [0.5, 0.3]),
            UiTextRole::Status,
        )
        .with_alignment(UiTextAlign::Center, UiTextAlign::Center);
        let report = spec.headless_report(0.05);
        let rendered = layout_bitmap_text(&spec, 0.05);

        assert_eq!(report.text, "REPORT\nREADY");
        assert_eq!(report.line_count, 2);
        assert_eq!(report.glyph_count, rendered.len());
        assert!(report.visible_bounds.is_some());
    }

    #[test]
    fn bitmap_layout_handles_empty_spaces_and_punctuation_deterministically() {
        let bounds = UiRect::new([0.0, 0.0], [0.5, 0.2]);
        let empty = layout_bitmap_text(&UiTextSpec::new("", bounds, UiTextRole::Body), 0.05);
        let spaces = layout_bitmap_text(&UiTextSpec::new("   ", bounds, UiTextRole::Body), 0.05);
        let punctuation =
            layout_bitmap_text(&UiTextSpec::new("!?.,", bounds, UiTextRole::Body), 0.05);

        assert!(empty.is_empty());
        assert!(spaces.is_empty());
        assert!(!punctuation.is_empty());
        assert_eq!(
            empty,
            layout_bitmap_text(&UiTextSpec::new("", bounds, UiTextRole::Body), 0.05)
        );
    }

    #[test]
    fn bitmap_layout_keeps_zero_ink_unknown_text_on_a_stable_policy() {
        let spec = UiTextSpec::new(
            "\u{1f600}",
            UiRect::new([0.0, 0.0], [0.4, 0.2]),
            UiTextRole::Body,
        )
        .with_missing_glyph_policy(UiMissingGlyphPolicy::Report);

        let first = layout_bitmap_text(&spec, 0.05);
        let second = layout_bitmap_text(&spec, 0.05);

        assert_eq!(first, second);
    }

    #[test]
    fn bitmap_layout_exposes_provider_neutral_lines_and_baselines() {
        let spec = UiTextSpec::new(
            "FIRST\nSECOND",
            UiRect::new([0.0, 0.0], [0.6, 0.4]),
            UiTextRole::Body,
        )
        .with_alignment(UiTextAlign::Start, UiTextAlign::Center);
        let layout = spec.bitmap_layout(0.05);

        assert_eq!(layout.line_count(), 2);
        assert!(layout.lines[0].advance > 0.0);
        assert!(layout.lines[0].baseline > layout.lines[1].baseline);
        assert_eq!(layout.lines[0].origin[0], layout.lines[1].origin[0]);
    }
}
