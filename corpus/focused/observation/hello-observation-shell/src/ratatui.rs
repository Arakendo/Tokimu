//! Standalone Ratatui host for the Observation Shell corpus.
//!
//! This is deliberately a terminal adapter, not a second shell. It forwards
//! literal input to the shared fixture and only owns terminal input, viewport,
//! and paint lifecycle.

mod session;

use std::io;

use crossterm::{
    event::{
        self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEventKind, MouseEventKind,
    },
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout},
    style::{Color, Style},
    text::{Line, Text},
    widgets::{Block, Borders, Paragraph, Wrap},
    Terminal,
};
use unicode_width::UnicodeWidthChar;

use session::ShellFixture;

fn main() -> io::Result<()> {
    enable_raw_mode()?;
    let _raw_mode_guard = RawModeGuard;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let _alternate_screen_guard = AlternateScreenGuard;
    execute!(stdout, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;
    let result = run(&mut terminal);

    terminal.show_cursor()?;
    result
}

/// Restores raw mode even when alternate-screen setup fails before the
/// terminal adapter is fully constructed.
struct RawModeGuard;

impl Drop for RawModeGuard {
    fn drop(&mut self) {
        let _ = disable_raw_mode();
    }
}

/// Restores the alternate screen and mouse capture after their acquisition.
/// Shell state remains independent of this terminal-host lifecycle.
struct AlternateScreenGuard;

impl Drop for AlternateScreenGuard {
    fn drop(&mut self) {
        let mut stdout = io::stdout();
        let _ = execute!(stdout, DisableMouseCapture, LeaveAlternateScreen);
    }
}

fn run(terminal: &mut Terminal<CrosstermBackend<io::Stdout>>) -> io::Result<()> {
    let mut session = TerminalSession::new();

    loop {
        terminal.draw(|frame| draw(frame, &mut session))?;

        if !event::poll(std::time::Duration::from_millis(100))? {
            continue;
        }
        match event::read()? {
            Event::Key(key) if key.kind == KeyEventKind::Press => {
                if session.handle_key(key.code) == TerminalAction::Exit {
                    return Ok(());
                }
            }
            Event::Mouse(mouse) => session.handle_mouse(mouse.kind),
            _ => {}
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum TerminalAction {
    Continue,
    Exit,
}

/// Ratatui-hosted terminal state. It deliberately owns only editor and
/// transcript mechanics; the `ShellFixture` still owns command meaning.
struct TerminalSession {
    fixture: ShellFixture,
    prompt: String,
    transcript: Vec<String>,
    history_cursor: Option<usize>,
    transcript_scroll: usize,
    transcript_viewport_rows: usize,
    transcript_viewport_columns: usize,
}

impl TerminalSession {
    fn new() -> Self {
        Self {
            fixture: ShellFixture::new(),
            prompt: String::new(),
            transcript: vec![
                "[system] standalone Ratatui host ready".to_owned(),
                "[hint] type HELP, INSPECT WORLD, LIST ENTITIES, or EXIT".to_owned(),
            ],
            history_cursor: None,
            transcript_scroll: 0,
            transcript_viewport_rows: 1,
            transcript_viewport_columns: 1,
        }
    }

    fn handle_key(&mut self, key: KeyCode) -> TerminalAction {
        match key {
            KeyCode::Esc => self.prompt.clear(),
            KeyCode::Backspace => {
                self.prompt.pop();
            }
            KeyCode::Char(character) => {
                self.prompt.push(character);
                self.history_cursor = None;
            }
            KeyCode::Up => self.recall_previous(),
            KeyCode::Down => self.recall_next(),
            KeyCode::PageUp => self.scroll_transcript_back(5),
            KeyCode::PageDown => self.scroll_transcript_forward(5),
            KeyCode::Home => self.transcript_scroll = 0,
            KeyCode::End => self.scroll_to_live_output(),
            KeyCode::Enter => return self.submit_prompt(),
            _ => {}
        }
        TerminalAction::Continue
    }

    fn handle_mouse(&mut self, kind: MouseEventKind) {
        match kind {
            MouseEventKind::ScrollUp => self.scroll_transcript_back(3),
            MouseEventKind::ScrollDown => self.scroll_transcript_forward(3),
            _ => {}
        }
    }

    fn submit_prompt(&mut self) -> TerminalAction {
        let command = self.prompt.trim().to_owned();
        self.prompt.clear();
        self.history_cursor = None;
        if matches!(command.to_ascii_lowercase().as_str(), "exit" | "quit") {
            return TerminalAction::Exit;
        }
        if command.is_empty() {
            return TerminalAction::Continue;
        }

        self.transcript.push(format!("> {command}"));
        if let Some(projection) = self.fixture.execute_line(&command) {
            self.transcript
                .extend(projection.lines().map(str::to_owned));
        }
        self.scroll_to_live_output();
        TerminalAction::Continue
    }

    fn recall_previous(&mut self) {
        let history = self.fixture.shell.history();
        if history.is_empty() {
            return;
        }
        let index = self
            .history_cursor
            .unwrap_or(history.len())
            .saturating_sub(1);
        self.prompt = history[index].input.clone();
        self.history_cursor = Some(index);
    }

    fn recall_next(&mut self) {
        let history = self.fixture.shell.history();
        let Some(index) = self.history_cursor else {
            return;
        };
        if index + 1 < history.len() {
            self.prompt = history[index + 1].input.clone();
            self.history_cursor = Some(index + 1);
        } else {
            self.prompt.clear();
            self.history_cursor = None;
        }
    }

    fn scroll_transcript_back(&mut self, rows: usize) {
        self.transcript_scroll = self.transcript_scroll.saturating_sub(rows);
    }

    fn scroll_transcript_forward(&mut self, rows: usize) {
        let maximum = self.maximum_transcript_scroll();
        self.transcript_scroll = (self.transcript_scroll + rows).min(maximum);
    }

    fn scroll_to_live_output(&mut self) {
        self.transcript_scroll = self.maximum_transcript_scroll();
    }

    fn maximum_transcript_scroll(&self) -> usize {
        self.transcript_visual_rows()
            .saturating_sub(self.transcript_viewport_rows)
    }

    fn transcript_visual_rows(&self) -> usize {
        self.transcript
            .iter()
            .map(|line| visual_rows(line, self.transcript_viewport_columns))
            .sum()
    }

    fn set_transcript_viewport(&mut self, rows: usize, columns: usize) {
        self.transcript_viewport_rows = rows.max(1);
        self.transcript_viewport_columns = columns.max(1);
        self.transcript_scroll = self.transcript_scroll.min(self.maximum_transcript_scroll());
    }
}

/// Counts the terminal rows consumed by a source line at the current viewport
/// width. This follows the same display-width model Ratatui uses for ordinary
/// text; ownership stays in the terminal adapter because source projections
/// remain unwrapped shell output.
fn visual_rows(line: &str, columns: usize) -> usize {
    let columns = columns.max(1);
    let mut rows = 1;
    let mut occupied = 0;

    for character in line.chars() {
        let width = UnicodeWidthChar::width(character).unwrap_or(0).min(columns);
        if width > 0 && occupied + width > columns {
            rows += 1;
            occupied = 0;
        }
        occupied += width;
    }

    rows
}

/// Locates the prompt cursor inside its bounded terminal cell area. Long input
/// remains adapter-local text editing state, so the native host visibly clips
/// the cursor at the final available cell rather than changing shell input.
fn prompt_cursor_offset(prompt: &str, columns: usize) -> u16 {
    let columns = columns.max(1);
    let prompt_width = 2 + prompt
        .chars()
        .map(|character| UnicodeWidthChar::width(character).unwrap_or(0))
        .sum::<usize>();
    prompt_width
        .min(columns.saturating_sub(1))
        .min(u16::MAX as usize) as u16
}

fn draw(frame: &mut ratatui::Frame, session: &mut TerminalSession) {
    let sections = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(3), Constraint::Length(3)])
        .split(frame.area());
    session.set_transcript_viewport(
        usize::from(sections[0].height.saturating_sub(2)),
        usize::from(sections[0].width.saturating_sub(2)),
    );
    let transcript = Text::from(
        session
            .transcript
            .iter()
            .map(|line| Line::styled(line, Style::default().fg(Color::Green)))
            .collect::<Vec<_>>(),
    );
    frame.render_widget(
        Paragraph::new(transcript)
            .block(
                Block::default()
                    .title(
                        " Tokimu Observation Shell / Ratatui | Wheel, PgUp/PgDn, Home/End scroll ",
                    )
                    .borders(Borders::ALL),
            )
            .scroll((session.transcript_scroll.min(u16::MAX as usize) as u16, 0))
            .wrap(Wrap { trim: false }),
        sections[0],
    );
    frame.render_widget(
        Paragraph::new(format!("> {}", session.prompt))
            .block(
                Block::default()
                    .title(" Prompt | Enter submits | Esc clears | Exit closes ")
                    .borders(Borders::ALL),
            )
            .style(Style::default().fg(Color::Cyan)),
        sections[1],
    );
    let prompt_columns = usize::from(sections[1].width.saturating_sub(2));
    frame.set_cursor_position((
        sections[1]
            .x
            .saturating_add(1)
            .saturating_add(prompt_cursor_offset(&session.prompt, prompt_columns)),
        sections[1].y.saturating_add(1),
    ));
}

#[cfg(test)]
mod tests {
    use super::{
        prompt_cursor_offset, visual_rows, KeyCode, MouseEventKind, TerminalAction, TerminalSession,
    };

    fn enter(session: &mut TerminalSession, line: &str) {
        for character in line.chars() {
            assert_eq!(
                session.handle_key(KeyCode::Char(character)),
                TerminalAction::Continue
            );
        }
        assert_eq!(session.handle_key(KeyCode::Enter), TerminalAction::Continue);
    }

    #[test]
    fn terminal_host_dispatches_through_the_shared_shell_fixture() {
        let mut terminal = TerminalSession::new();
        enter(&mut terminal, "help");
        enter(&mut terminal, "inspect world");

        assert_eq!(terminal.fixture.shell.history().len(), 2);
        assert!(terminal.transcript.iter().any(|line| line == "> help"));
        assert!(terminal
            .transcript
            .iter()
            .any(|line| line.contains("[shell] help")));
        assert!(terminal
            .transcript
            .iter()
            .any(|line| line.starts_with("revision: 0")));
    }

    #[test]
    fn terminal_host_history_and_exit_are_adapter_local() {
        let mut terminal = TerminalSession::new();
        enter(&mut terminal, "list entities");
        terminal.handle_key(KeyCode::Up);
        assert_eq!(terminal.prompt, "list entities");
        terminal.handle_key(KeyCode::Down);
        assert!(terminal.prompt.is_empty());

        for character in "exit".chars() {
            terminal.handle_key(KeyCode::Char(character));
        }
        assert_eq!(terminal.handle_key(KeyCode::Enter), TerminalAction::Exit);
        assert_eq!(terminal.fixture.shell.history().len(), 1);
    }

    #[test]
    fn terminal_host_keeps_transcript_navigation_adapter_local() {
        let mut terminal = TerminalSession::new();
        terminal.transcript = (0..24).map(|index| format!("line {index}")).collect();
        terminal.set_transcript_viewport(6, 80);

        terminal.handle_key(KeyCode::End);
        assert_eq!(terminal.transcript_scroll, 18);
        terminal.handle_key(KeyCode::PageUp);
        assert_eq!(terminal.transcript_scroll, 13);
        terminal.handle_key(KeyCode::Home);
        assert_eq!(terminal.transcript_scroll, 0);
        terminal.handle_key(KeyCode::PageDown);
        assert_eq!(terminal.transcript_scroll, 5);

        enter(&mut terminal, "help");
        assert_eq!(
            terminal.transcript_scroll,
            terminal.maximum_transcript_scroll()
        );
    }

    #[test]
    fn terminal_host_maps_wheel_direction_to_older_and_newer_output() {
        let mut terminal = TerminalSession::new();
        terminal.transcript = (0..24).map(|index| format!("line {index}")).collect();
        terminal.set_transcript_viewport(6, 80);
        terminal.handle_key(KeyCode::End);

        terminal.handle_mouse(MouseEventKind::ScrollUp);
        assert_eq!(terminal.transcript_scroll, 15);
        terminal.handle_mouse(MouseEventKind::ScrollDown);
        assert_eq!(terminal.transcript_scroll, 18);
        assert!(terminal.fixture.shell.history().is_empty());
    }

    #[test]
    fn terminal_host_clamps_the_viewport_after_a_resize() {
        let mut terminal = TerminalSession::new();
        terminal.transcript = (0..24).map(|index| format!("line {index}")).collect();
        terminal.set_transcript_viewport(6, 80);
        terminal.handle_key(KeyCode::End);
        assert_eq!(terminal.transcript_scroll, 18);

        terminal.set_transcript_viewport(12, 80);
        assert_eq!(terminal.transcript_scroll, 12);
        terminal.set_transcript_viewport(0, 0);
        assert_eq!(terminal.transcript_viewport_rows, 1);
        assert_eq!(terminal.transcript_viewport_columns, 1);
        assert_eq!(terminal.transcript_scroll, 12);
    }

    #[test]
    fn terminal_host_scrolls_visual_rows_after_wrapping() {
        let mut terminal = TerminalSession::new();
        terminal.transcript = vec!["abcdefgh".to_owned(), "ij".to_owned()];
        terminal.set_transcript_viewport(2, 4);

        assert_eq!(terminal.transcript_visual_rows(), 3);
        terminal.handle_key(KeyCode::End);
        assert_eq!(terminal.transcript_scroll, 1);
        terminal.handle_key(KeyCode::PageUp);
        assert_eq!(terminal.transcript_scroll, 0);
    }

    #[test]
    fn visual_row_accounting_uses_terminal_display_width() {
        assert_eq!(visual_rows("", 4), 1);
        assert_eq!(visual_rows("abcd", 4), 1);
        assert_eq!(visual_rows("abcde", 4), 2);
        assert_eq!(visual_rows("a界b", 3), 2);
    }

    #[test]
    fn prompt_cursor_stays_inside_the_bounded_prompt_area() {
        assert_eq!(prompt_cursor_offset("", 10), 2);
        assert_eq!(prompt_cursor_offset("abc", 10), 5);
        assert_eq!(prompt_cursor_offset("a界", 10), 5);
        assert_eq!(prompt_cursor_offset("abcdef", 4), 3);
        assert_eq!(prompt_cursor_offset("anything", 1), 0);
    }
}
