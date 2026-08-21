//! Doom-corpus composition of the existing Tokimu text raster seam.
//!
//! This is a bounded application debug surface, not an admitted Observation
//! Shell, command language, or renderer-owned inspection facility.

use ui_tools::provider::UiFontRasterizer;

const TRANSCRIPT_LIMIT: usize = 12;
const VISIBLE_TRANSCRIPT_LINES: usize = 8;
const TEXT_PIXELS: f32 = 18.0;
const LINE_HEIGHT: u32 = 24;
const PADDING: u32 = 12;
const TRUNCATION_MARKER: &str = "[console] earlier transcript lines truncated";

#[derive(Clone, Debug)]
pub struct DoomDebugConsole {
    open: bool,
    prompt: String,
    transcript: Vec<String>,
    dirty: bool,
}

impl Default for DoomDebugConsole {
    fn default() -> Self {
        Self {
            open: false,
            prompt: String::new(),
            transcript: vec![
                "[doom] debug console ready; HELP lists corpus-local commands".to_owned(),
                "[boundary] inspection only; no renderer or Ring 0 command ownership".to_owned(),
            ],
            dirty: true,
        }
    }
}

impl DoomDebugConsole {
    /// Reports the fixed presentation extent used by the current corpus
    /// console without requiring a font provider or performing raster work.
    /// This is useful when inventorying which resources change on an edit.
    pub const fn raster_dimensions(width: u32) -> [u32; 2] {
        [
            if width < 320 { 320 } else { width },
            PADDING * 2 + LINE_HEIGHT * (VISIBLE_TRANSCRIPT_LINES as u32 + 2),
        ]
    }

    pub const fn is_open(&self) -> bool {
        self.open
    }

    pub fn set_open(&mut self, open: bool) {
        if self.open != open {
            self.open = open;
            self.dirty = true;
        }
    }

    pub fn insert_text(&mut self, text: &str) {
        for character in text.chars() {
            if !character.is_control() && character != '`' && character != '~' {
                self.prompt.push(character);
            }
        }
        self.dirty = true;
    }

    pub fn backspace(&mut self) {
        self.prompt.pop();
        self.dirty = true;
    }

    pub fn take_submission(&mut self) -> Option<String> {
        let command = self.prompt.trim().to_owned();
        self.prompt.clear();
        self.dirty = true;
        if command.is_empty() {
            return None;
        }
        self.append(format!("> {command}"));
        Some(command)
    }

    pub fn append(&mut self, line: impl Into<String>) {
        self.transcript.push(line.into());
        if self.transcript.len() > TRANSCRIPT_LIMIT {
            let remove = self.transcript.len() - TRANSCRIPT_LIMIT + 1;
            self.transcript.drain(0..remove);
            self.transcript.insert(0, TRUNCATION_MARKER.to_owned());
        }
        self.dirty = true;
    }

    pub fn clear(&mut self) {
        self.transcript.clear();
        self.append("[doom] transcript cleared");
    }

    pub fn take_dirty(&mut self) -> bool {
        std::mem::take(&mut self.dirty)
    }

    pub fn invalidate(&mut self) {
        self.dirty = true;
    }

    pub fn rasterize(&self, font: &UiFontRasterizer, width: u32) -> DebugConsoleRaster {
        let [width, height] = Self::raster_dimensions(width);
        let mut rgba8 = vec![0_u8; width as usize * height as usize * 4];
        for pixel in rgba8.chunks_exact_mut(4) {
            pixel.copy_from_slice(&[3, 13, 16, 248]);
        }
        let mut surface = RasterSurface {
            rgba8: &mut rgba8,
            width,
            height,
        };

        let usable_width = width.saturating_sub(PADDING * 2);
        let wrapped_transcript = self
            .transcript
            .iter()
            .flat_map(|line| wrap_text(font, line, usable_width))
            .collect::<Vec<_>>();
        let visible_start = wrapped_transcript
            .len()
            .saturating_sub(VISIBLE_TRANSCRIPT_LINES);
        for (row, line) in wrapped_transcript[visible_start..].iter().enumerate() {
            surface.blit_line(
                font,
                line,
                PADDING,
                PADDING + row as u32 * LINE_HEIGHT,
                [166, 224, 211],
            );
        }
        let prompt_y = height - PADDING - LINE_HEIGHT;
        let prompt = wrap_text(font, &format!("> {}_", self.prompt), usable_width)
            .into_iter()
            .last()
            .unwrap_or_else(|| "> _".to_owned());
        surface.blit_line(font, &prompt, PADDING, prompt_y, [115, 230, 198]);
        DebugConsoleRaster {
            width,
            height,
            rgba8,
        }
    }
}

/// Wraps at word boundaries using the actual provider font metrics. Long
/// tokens are split only when they cannot fit on an otherwise empty row.
/// This stays at the raster boundary, so the transcript retains its full
/// corpus diagnostic text rather than acquiring a presentation-width contract.
fn wrap_text(font: &UiFontRasterizer, text: &str, maximum_width: u32) -> Vec<String> {
    if text.trim().is_empty() {
        return vec![String::new()];
    }
    let fits = |candidate: &str| font.rasterize_text(candidate, TEXT_PIXELS).width <= maximum_width;
    let mut rows = Vec::new();
    let mut current = String::new();
    for word in text.split_whitespace() {
        let candidate = if current.is_empty() {
            word.to_owned()
        } else {
            format!("{current} {word}")
        };
        if fits(&candidate) {
            current = candidate;
            continue;
        }
        if !current.is_empty() {
            rows.push(std::mem::take(&mut current));
        }
        if fits(word) {
            current = word.to_owned();
            continue;
        }
        for character in word.chars() {
            let candidate = format!("{current}{character}");
            if !current.is_empty() && !fits(&candidate) {
                rows.push(std::mem::take(&mut current));
            }
            current.push(character);
        }
    }
    if !current.is_empty() {
        rows.push(current);
    }
    rows
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DebugConsoleRaster {
    pub width: u32,
    pub height: u32,
    pub rgba8: Vec<u8>,
}

struct RasterSurface<'a> {
    rgba8: &'a mut [u8],
    width: u32,
    height: u32,
}

impl RasterSurface<'_> {
    fn blit_line(
        &mut self,
        font: &UiFontRasterizer,
        text: &str,
        origin_x: u32,
        origin_y: u32,
        tint: [u8; 3],
    ) {
        let bitmap = font.rasterize_text(text, TEXT_PIXELS);
        for y in 0..bitmap.height {
            let destination_y = origin_y + y;
            if destination_y >= self.height {
                break;
            }
            for x in 0..bitmap.width {
                let destination_x = origin_x + x;
                if destination_x >= self.width {
                    break;
                }
                let alpha = bitmap.alpha[(y * bitmap.width + x) as usize];
                if alpha == 0 {
                    continue;
                }
                let offset = ((destination_y * self.width + destination_x) * 4) as usize;
                self.rgba8[offset] = tint[0];
                self.rgba8[offset + 1] = tint[1];
                self.rgba8[offset + 2] = tint[2];
                self.rgba8[offset + 3] = 255;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn font() -> UiFontRasterizer {
        let source = ui_tools::provider::UiFontSource::from_native_default()
            .expect("checked-in default font");
        UiFontRasterizer::from_bytes(source.bytes).expect("valid default font")
    }

    #[test]
    fn prompt_filters_toggle_characters_and_retains_submission() {
        let mut console = DoomDebugConsole::default();
        console.insert_text("`camera~");
        assert_eq!(console.take_submission().as_deref(), Some("camera"));
    }

    #[test]
    fn raster_dimensions_are_fixed_except_for_bounded_width() {
        assert_eq!(DoomDebugConsole::raster_dimensions(200), [320, 264]);
        assert_eq!(DoomDebugConsole::raster_dimensions(960), [960, 264]);
    }

    #[test]
    fn transcript_is_bounded() {
        let mut console = DoomDebugConsole::default();
        for index in 0..32 {
            console.append(format!("line {index}"));
        }
        assert_eq!(console.transcript.len(), TRANSCRIPT_LIMIT);
        assert_eq!(console.transcript.first().unwrap(), TRUNCATION_MARKER);
        assert_eq!(
            console.transcript.last().map(String::as_str),
            Some("line 31")
        );
    }

    #[test]
    fn long_diagnostic_lines_wrap_to_the_measured_width() {
        let font = font();
        let rows = wrap_text(
            &font,
            "look: exact prepared-triangle hit distance=105.802 family=opaque material=41 label=wall:94:STARG3 source=wall linedef=94 sidedef=12 sector=3",
            240,
        );
        assert!(rows.len() > 1);
        assert!(rows
            .iter()
            .all(|row| font.rasterize_text(row, TEXT_PIXELS).width <= 240));
        assert_eq!(
            rows.join(" ").split_whitespace().collect::<Vec<_>>(),
            [
                "look:",
                "exact",
                "prepared-triangle",
                "hit",
                "distance=105.802",
                "family=opaque",
                "material=41",
                "label=wall:94:STARG3",
                "source=wall",
                "linedef=94",
                "sidedef=12",
                "sector=3",
            ]
        );
    }
}
