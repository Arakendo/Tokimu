use super::{
    UiGlyphQuad, UiTextAlign, UiTextAlignmentBasis, UiTextDirection, UiTextOverflow, UiTextSpec,
};

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

pub(super) fn physical_alignment(align: UiTextAlign, direction: UiTextDirection) -> UiTextAlign {
    match (align, direction) {
        (UiTextAlign::Start, UiTextDirection::Rtl) => UiTextAlign::End,
        (UiTextAlign::End, UiTextDirection::Rtl) => UiTextAlign::Start,
        _ => align,
    }
}

pub(super) fn text_lines(spec: &UiTextSpec, height: f32) -> Vec<String> {
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

pub(super) fn bitmap_cell(height: f32) -> f32 {
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
