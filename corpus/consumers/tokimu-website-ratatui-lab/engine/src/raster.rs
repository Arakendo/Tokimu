use ratatui::{
    buffer::Buffer,
    style::{Color, Modifier},
};
use ui_tools::UiFontRasterizer;

pub(crate) const CELL_PIXEL_WIDTH: u32 = 10;
pub(crate) const CELL_PIXEL_HEIGHT: u32 = 18;
const FONT_PIXELS: f32 = 14.0;
const BASELINE_OFFSET: f32 = 14.0;
const MAX_PIXELS: u64 = 16_777_216;
const CANVAS_COLOR: [u8; 4] = [5, 11, 13, 255];

const DEPARTURE_MONO: &[u8] = include_bytes!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../../../third-party/fonts/departure-mono/public/assets/DepartureMono-Regular.otf"
));

#[derive(Debug, PartialEq, Eq)]
pub(crate) struct TokimuFrame {
    pub(crate) width: u32,
    pub(crate) height: u32,
    pub(crate) rgba: Vec<u8>,
}

impl TokimuFrame {
    #[cfg(test)]
    pub(crate) fn fingerprint(&self) -> u64 {
        self.rgba.iter().fold(0xcbf29ce484222325_u64, |hash, byte| {
            (hash ^ u64::from(*byte)).wrapping_mul(0x100000001b3)
        })
    }

    fn blend_pixel(&mut self, x: i32, y: i32, source: [u8; 4]) {
        if x < 0 || y < 0 || x >= self.width as i32 || y >= self.height as i32 {
            return;
        }
        let index = (y as usize * self.width as usize + x as usize) * 4;
        let alpha = u32::from(source[3]);
        let inverse = 255 - alpha;
        for (channel, source_value) in source.iter().copied().take(3).enumerate() {
            self.rgba[index + channel] = ((u32::from(source_value) * alpha
                + u32::from(self.rgba[index + channel]) * inverse
                + 127)
                / 255) as u8;
        }
        self.rgba[index + 3] = 255;
    }

    fn fill_cell(&mut self, column: u16, row: u16, color: [u8; 4]) {
        let left = u32::from(column) * CELL_PIXEL_WIDTH;
        let top = u32::from(row) * CELL_PIXEL_HEIGHT;
        for y in top..top + CELL_PIXEL_HEIGHT {
            for x in left..left + CELL_PIXEL_WIDTH {
                self.blend_pixel(x as i32, y as i32, color);
            }
        }
    }
}

pub(crate) fn rasterize(buffer: &Buffer) -> Result<TokimuFrame, String> {
    let width = u32::from(buffer.area.width) * CELL_PIXEL_WIDTH;
    let height = u32::from(buffer.area.height) * CELL_PIXEL_HEIGHT;
    let pixels = u64::from(width) * u64::from(height);
    if pixels > MAX_PIXELS {
        return Err(format!(
            "Tokimu Ratatui frame exceeds the {MAX_PIXELS}-pixel limit: {width}x{height}"
        ));
    }
    let font = UiFontRasterizer::from_bytes(DEPARTURE_MONO.to_vec())
        .map_err(|error| format!("load Departure Mono provider: {error}"))?;
    let mut frame = TokimuFrame {
        width,
        height,
        rgba: CANVAS_COLOR.repeat(pixels as usize),
    };

    for (index, cell) in buffer.content().iter().enumerate() {
        let column = (index % usize::from(buffer.area.width)) as u16;
        let row = (index / usize::from(buffer.area.width)) as u16;
        if cell.bg != Color::Reset {
            frame.fill_cell(column, row, color_rgba(cell.bg, CANVAS_COLOR));
        }

        let Some(character) = cell.symbol().chars().next() else {
            continue;
        };
        if character.is_whitespace() {
            continue;
        }
        let glyph = font.rasterize(character, FONT_PIXELS);
        let cell_left = f32::from(column) * CELL_PIXEL_WIDTH as f32;
        let cell_top = f32::from(row) * CELL_PIXEL_HEIGHT as f32;
        let pen_x = cell_left + (CELL_PIXEL_WIDTH as f32 - glyph.advance) * 0.5;
        let glyph_left = (pen_x + glyph.bearing_x).round() as i32;
        let glyph_top = (cell_top + BASELINE_OFFSET + glyph.bearing_y).round() as i32;
        let mut foreground = color_rgba(cell.fg, [216, 235, 231, 255]);
        if cell.modifier.contains(Modifier::DIM) {
            foreground[3] = 140;
        }
        for y in 0..glyph.height {
            for x in 0..glyph.width {
                let coverage = glyph.alpha[(y * glyph.width + x) as usize];
                let alpha = ((u16::from(coverage) * u16::from(foreground[3]) + 127) / 255) as u8;
                let pixel = [foreground[0], foreground[1], foreground[2], alpha];
                frame.blend_pixel(glyph_left + x as i32, glyph_top + y as i32, pixel);
                if cell.modifier.contains(Modifier::BOLD) {
                    frame.blend_pixel(glyph_left + x as i32 + 1, glyph_top + y as i32, pixel);
                }
            }
        }
        if cell.modifier.contains(Modifier::UNDERLINED) {
            let underline_y = i32::from(row) * CELL_PIXEL_HEIGHT as i32 + 16;
            let underline_x = i32::from(column) * CELL_PIXEL_WIDTH as i32;
            for x in 0..CELL_PIXEL_WIDTH as i32 {
                frame.blend_pixel(underline_x + x, underline_y, foreground);
            }
        }
    }
    Ok(frame)
}

fn color_rgba(color: Color, reset: [u8; 4]) -> [u8; 4] {
    match color {
        Color::Reset => reset,
        Color::Black => [5, 11, 13, 255],
        Color::Red => [214, 91, 91, 255],
        Color::Green => [83, 193, 135, 255],
        Color::Yellow => [210, 168, 75, 255],
        Color::Blue => [88, 137, 218, 255],
        Color::Magenta => [184, 106, 202, 255],
        Color::Cyan => [77, 201, 194, 255],
        Color::Gray => [155, 171, 168, 255],
        Color::DarkGray => [91, 108, 106, 255],
        Color::LightRed => [241, 143, 143, 255],
        Color::LightGreen => [119, 227, 171, 255],
        Color::LightYellow => [231, 192, 109, 255],
        Color::LightBlue => [144, 185, 255, 255],
        Color::LightMagenta => [219, 157, 240, 255],
        Color::LightCyan => [115, 229, 223, 255],
        Color::White => [216, 235, 231, 255],
        Color::Rgb(red, green, blue) => [red, green, blue, 255],
        Color::Indexed(index) => indexed_color(index),
    }
}

fn indexed_color(index: u8) -> [u8; 4] {
    const ANSI: [[u8; 4]; 16] = [
        [5, 11, 13, 255],
        [214, 91, 91, 255],
        [83, 193, 135, 255],
        [210, 168, 75, 255],
        [88, 137, 218, 255],
        [184, 106, 202, 255],
        [77, 201, 194, 255],
        [155, 171, 168, 255],
        [91, 108, 106, 255],
        [241, 143, 143, 255],
        [119, 227, 171, 255],
        [231, 192, 109, 255],
        [144, 185, 255, 255],
        [219, 157, 240, 255],
        [115, 229, 223, 255],
        [216, 235, 231, 255],
    ];
    if index < 16 {
        return ANSI[index as usize];
    }
    if index >= 232 {
        let value = 8 + (index - 232) * 10;
        return [value, value, value, 255];
    }
    let cube = index - 16;
    let channel = |value: u8| if value == 0 { 0 } else { 55 + value * 40 };
    [
        channel(cube / 36),
        channel((cube / 6) % 6),
        channel(cube % 6),
        255,
    ]
}
