//! Corpus-local Ratatui projection for the semantic Observation Shell.
//!
//! The shell owns commands and transcript meaning. Ratatui owns terminal
//! layout and style. This backend retains Ratatui's changed cells so Tokimu
//! can rasterize them without asking the browser to recreate terminal layout.

use std::io;

use observation_shell::ShellRecord;
use ratatui::{
    backend::{Backend, WindowSize},
    buffer::{Buffer, Cell},
    layout::{Constraint, Layout, Position, Rect, Size},
    style::{Color, Modifier, Style},
    text::{Line, Text},
    widgets::{Block, Borders, Paragraph, Wrap},
    Terminal,
};
use ui_tools::UiFontRasterizer;
use unicode_width::UnicodeWidthStr;

const CELL_WIDTH: u32 = 10;
const CELL_HEIGHT: u32 = 18;
const FONT_PIXELS: f32 = 14.0;
const BASELINE_OFFSET: f32 = 14.0;
const MAX_PIXELS: u64 = 16_777_216;
const BACKGROUND: [u8; 4] = [5, 11, 13, 255];
const DEPARTURE_MONO: &[u8] = include_bytes!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../../../third-party/fonts/departure-mono/public/assets/DepartureMono-Regular.otf"
));

pub(crate) struct TokimuFrame {
    pub(crate) width: u32,
    pub(crate) height: u32,
    pub(crate) rgba: Vec<u8>,
}

struct TokimuBackend {
    buffer: Buffer,
    cursor: Position,
    cursor_visible: bool,
}

impl TokimuBackend {
    fn new(width: u16, height: u16) -> Self {
        Self {
            buffer: Buffer::empty(Rect::new(0, 0, width, height)),
            cursor: Position::new(0, 0),
            cursor_visible: false,
        }
    }
}

impl Backend for TokimuBackend {
    fn draw<'a, I>(&mut self, content: I) -> io::Result<()>
    where
        I: Iterator<Item = (u16, u16, &'a Cell)>,
    {
        for (x, y, cell) in content {
            let target = self.buffer.cell_mut(Position::new(x, y)).ok_or_else(|| {
                io::Error::new(
                    io::ErrorKind::InvalidInput,
                    "Ratatui cell lies outside Tokimu grid",
                )
            })?;
            *target = cell.clone();
        }
        Ok(())
    }

    fn hide_cursor(&mut self) -> io::Result<()> {
        self.cursor_visible = false;
        Ok(())
    }
    fn show_cursor(&mut self) -> io::Result<()> {
        self.cursor_visible = true;
        Ok(())
    }
    fn get_cursor_position(&mut self) -> io::Result<Position> {
        Ok(self.cursor)
    }
    fn set_cursor_position<P: Into<Position>>(&mut self, position: P) -> io::Result<()> {
        self.cursor = position.into();
        Ok(())
    }
    fn clear(&mut self) -> io::Result<()> {
        self.buffer = Buffer::empty(self.buffer.area);
        Ok(())
    }
    fn size(&self) -> io::Result<Size> {
        Ok(Size::new(self.buffer.area.width, self.buffer.area.height))
    }
    fn window_size(&mut self) -> io::Result<WindowSize> {
        Ok(WindowSize {
            columns_rows: self.size()?,
            pixels: Size::new(0, 0),
        })
    }
    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

pub(crate) fn render_shell(
    history: &[ShellRecord],
    prompt: &str,
    scroll: usize,
    runtime_status: &str,
    requested_width: u32,
    requested_height: u32,
) -> Result<TokimuFrame, String> {
    let columns = (requested_width / CELL_WIDTH).clamp(48, 120) as u16;
    let rows = (requested_height / CELL_HEIGHT).clamp(18, 48) as u16;
    let backend = TokimuBackend::new(columns, rows);
    let mut terminal = Terminal::new(backend)
        .map_err(|error| format!("create Tokimu Ratatui terminal: {error}"))?;
    terminal
        .draw(|frame| {
            let [header, transcript_area, prompt_area, footer] = Layout::vertical([
                // The border consumes the first and last rows, leaving three
                // content rows for title, boundary description, and live
                // shared-runtime status.
                Constraint::Length(5),
                Constraint::Min(5),
                Constraint::Length(3),
                Constraint::Length(1),
            ])
            .areas(frame.area());
            let chrome = Style::default().fg(Color::LightCyan);
            frame.render_widget(
                Paragraph::new(Text::from(vec![
                    Line::styled(" TOKIMU OBSERVATION SHELL / RATATUI", chrome.add_modifier(Modifier::BOLD)),
                    Line::styled(" semantic commands in Rust; terminal composition in Ratatui", Style::default().fg(Color::DarkGray)),
                    Line::styled(runtime_status.to_owned(), Style::default().fg(Color::Gray)),
                ]))
                .block(Block::default().borders(Borders::ALL).border_style(chrome)),
                header,
            );
            let transcript = transcript_lines(history);
            let max_scroll = transcript_line_count(history, transcript_area.width.saturating_sub(2))
                .saturating_sub(usize::from(transcript_area.height.saturating_sub(2)));
            frame.render_widget(
                Paragraph::new(transcript)
                    .block(Block::default().borders(Borders::ALL).title(" TRANSCRIPT ").border_style(chrome))
                    .scroll((scroll.min(max_scroll) as u16, 0))
                    .wrap(Wrap { trim: false }),
                transcript_area,
            );
            frame.render_widget(
                Paragraph::new(format!("> {prompt}_"))
                    .style(Style::default().fg(Color::LightGreen))
                    .block(Block::default().borders(Borders::ALL).title(" PROMPT ").border_style(chrome)),
                prompt_area,
            );
            frame.render_widget(
                Paragraph::new(" Enter submits | Up/Down history | Wheel reviews transcript | Esc clears prompt")
                    .style(Style::default().fg(Color::DarkGray)),
                footer,
            );
        })
        .map_err(|error| format!("render Ratatui Observation Shell: {error}"))?;
    rasterize(&terminal.backend().buffer)
}

fn transcript_line_count(history: &[ShellRecord], content_width: u16) -> usize {
    // Ratatui remains authoritative for the actual line break. This matching
    // display-width estimate prevents the browser host from treating a long
    // semantic record as one scroll row without recreating terminal layout.
    let width = usize::from(content_width.max(1));
    let system_rows = [
        "[system] semantic Observation Shell ready",
        "[hint] try: help, application runtime world-summary, or application runtime playback",
        "",
    ]
    .into_iter()
    .map(|line| wrapped_row_count(line, width))
    .sum::<usize>();

    system_rows
        + history
            .iter()
            .map(|record| {
                wrapped_row_count(&format!("> {}", record.input), width)
                    + record
                        .projection
                        .lines()
                        .map(|line| wrapped_row_count(line, width))
                        .sum::<usize>()
                    + 1
            })
            .sum::<usize>()
}

fn wrapped_row_count(line: &str, width: usize) -> usize {
    UnicodeWidthStr::width(line).div_ceil(width).max(1)
}

fn transcript_lines(history: &[ShellRecord]) -> Text<'static> {
    let mut lines = vec![
        Line::styled(
            "[system] semantic Observation Shell ready",
            Style::default().fg(Color::Cyan),
        ),
        Line::styled(
            "[hint] try: help, application runtime world-summary, or application runtime playback",
            Style::default().fg(Color::Gray),
        ),
        Line::raw(""),
    ];
    for record in history {
        lines.push(Line::styled(
            format!("> {}", record.input),
            Style::default().fg(Color::LightGreen),
        ));
        for line in record.projection.lines() {
            lines.push(Line::raw(line.to_owned()));
        }
        lines.push(Line::raw(""));
    }
    Text::from(lines)
}

fn rasterize(buffer: &Buffer) -> Result<TokimuFrame, String> {
    let width = u32::from(buffer.area.width) * CELL_WIDTH;
    let height = u32::from(buffer.area.height) * CELL_HEIGHT;
    let pixels = u64::from(width) * u64::from(height);
    if pixels > MAX_PIXELS {
        return Err(format!(
            "Ratatui frame exceeds {MAX_PIXELS} pixels: {width}x{height}"
        ));
    }
    let font = UiFontRasterizer::from_bytes(DEPARTURE_MONO.to_vec())
        .map_err(|error| format!("load Departure Mono: {error}"))?;
    let mut frame = TokimuFrame {
        width,
        height,
        rgba: BACKGROUND.repeat(pixels as usize),
    };
    for (index, cell) in buffer.content().iter().enumerate() {
        let column = (index % usize::from(buffer.area.width)) as u16;
        let row = (index / usize::from(buffer.area.width)) as u16;
        if cell.bg != Color::Reset {
            fill_cell(&mut frame, column, row, color(cell.bg, BACKGROUND));
        }
        let Some(character) = cell.symbol().chars().next() else {
            continue;
        };
        if character.is_whitespace() {
            continue;
        }
        let glyph = font.rasterize(character, FONT_PIXELS);
        let left = (f32::from(column) * CELL_WIDTH as f32
            + (CELL_WIDTH as f32 - glyph.advance) * 0.5
            + glyph.bearing_x)
            .round() as i32;
        let top = (f32::from(row) * CELL_HEIGHT as f32 + BASELINE_OFFSET + glyph.bearing_y).round()
            as i32;
        let mut foreground = color(cell.fg, [216, 235, 231, 255]);
        if cell.modifier.contains(Modifier::DIM) {
            foreground[3] = 140;
        }
        for y in 0..glyph.height {
            for x in 0..glyph.width {
                let alpha = ((u16::from(glyph.alpha[(y * glyph.width + x) as usize])
                    * u16::from(foreground[3])
                    + 127)
                    / 255) as u8;
                blend(
                    &mut frame,
                    left + x as i32,
                    top + y as i32,
                    [foreground[0], foreground[1], foreground[2], alpha],
                );
            }
        }
    }
    Ok(frame)
}

fn fill_cell(frame: &mut TokimuFrame, column: u16, row: u16, value: [u8; 4]) {
    for y in u32::from(row) * CELL_HEIGHT..u32::from(row) * CELL_HEIGHT + CELL_HEIGHT {
        for x in u32::from(column) * CELL_WIDTH..u32::from(column) * CELL_WIDTH + CELL_WIDTH {
            blend(frame, x as i32, y as i32, value);
        }
    }
}

fn blend(frame: &mut TokimuFrame, x: i32, y: i32, source: [u8; 4]) {
    if x < 0 || y < 0 || x >= frame.width as i32 || y >= frame.height as i32 {
        return;
    }
    let index = (y as usize * frame.width as usize + x as usize) * 4;
    let alpha = u32::from(source[3]);
    let inverse = 255 - alpha;
    for (channel, value) in source.iter().copied().take(3).enumerate() {
        frame.rgba[index + channel] =
            ((u32::from(value) * alpha + u32::from(frame.rgba[index + channel]) * inverse + 127)
                / 255) as u8;
    }
    frame.rgba[index + 3] = 255;
}

fn color(value: Color, reset: [u8; 4]) -> [u8; 4] {
    match value {
        Color::Reset => reset,
        Color::Black => BACKGROUND,
        Color::Green | Color::LightGreen => [119, 227, 171, 255],
        Color::Cyan | Color::LightCyan => [115, 229, 223, 255],
        Color::Yellow | Color::LightYellow => [231, 192, 109, 255],
        Color::DarkGray => [91, 108, 106, 255],
        Color::Gray => [155, 171, 168, 255],
        Color::White => [216, 235, 231, 255],
        Color::Rgb(red, green, blue) => [red, green, blue, 255],
        _ => [216, 235, 231, 255],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn renders_a_bounded_tokimu_frame_without_a_terminal_provider() {
        let frame = render_shell(
            &[],
            "help",
            0,
            "runtime tick=0 | presentation=unselected",
            720,
            432,
        )
        .expect("Ratatui shell should render");

        assert_eq!(frame.width, 720);
        assert_eq!(frame.height, 432);
        assert_eq!(frame.rgba.len(), 720 * 432 * 4);
        assert!(frame.rgba.chunks_exact(4).any(|pixel| pixel != BACKGROUND));
    }

    #[test]
    fn clamps_an_oversized_browser_host_to_the_terminal_budget() {
        let frame = render_shell(
            &[],
            "",
            0,
            "runtime tick=0 | presentation=unselected",
            20_000,
            20_000,
        )
        .expect("Ratatui shell should keep its frame bounded");

        assert_eq!(frame.width, 120 * CELL_WIDTH);
        assert_eq!(frame.height, 48 * CELL_HEIGHT);
        assert!(u64::from(frame.width) * u64::from(frame.height) <= MAX_PIXELS);
    }

    #[test]
    fn display_width_estimate_accounts_for_wrapped_and_wide_text() {
        assert_eq!(wrapped_row_count("abcdefgh", 4), 2);
        assert_eq!(wrapped_row_count("界界界", 4), 2);
        assert_eq!(wrapped_row_count("", 4), 1);
    }
}
