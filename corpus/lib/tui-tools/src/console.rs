use crate::{
    ProjectionArtifact, StyleRole, Surface, TextAlignment, TranscriptLine, TuiDiagnostic,
    TuiExtent, TuiInsets, TuiRect, TuiViewport, TUI_TOOLS_ARTIFACT_SCHEMA,
};

/// Caller-owned prompt presentation for a bounded embedded console.
///
/// This is display state only. Command parsing, history, and command outcomes
/// remain outside `tui-tools`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ConsolePrompt {
    pub prefix: String,
    pub input: String,
    pub focused: bool,
    pub cursor_visible: bool,
}

impl ConsolePrompt {
    pub fn new(prefix: impl Into<String>, input: impl Into<String>) -> Self {
        Self {
            prefix: prefix.into(),
            input: input.into(),
            focused: false,
            cursor_visible: false,
        }
    }

    pub fn focused(mut self, cursor_visible: bool) -> Self {
        self.focused = true;
        self.cursor_visible = cursor_visible;
        self
    }

    fn display_text(&self) -> String {
        let cursor = if self.focused && self.cursor_visible {
            "_"
        } else {
            ""
        };
        format!("{} {}{}", self.prefix, self.input, cursor)
    }
}

/// Projects a console-sized transcript and prompt without becoming a shell or
/// terminal host. Callers retain ownership of every record and prompt value.
pub fn render_embedded_console(
    title: &str,
    lines: &[TranscriptLine],
    viewport: TuiViewport,
    prompt: &ConsolePrompt,
    extent: TuiExtent,
) -> ProjectionArtifact {
    let mut surface = Surface::new(extent);
    let frame = extent.rect();
    surface.draw_frame(frame, StyleRole::Frame);
    let content = frame.inset(TuiInsets::all(1));
    if content.height < 3 {
        surface.extend_diagnostics([TuiDiagnostic::Undersized {
            axis: "vertical",
            available: content.height,
            required: 3,
        }]);
        return ProjectionArtifact {
            schema: TUI_TOOLS_ARTIFACT_SCHEMA,
            producer: "tui-tools/embedded-console",
            extent,
            surface,
        };
    }

    surface.write_line(content, 0, title, TextAlignment::Start, StyleRole::Heading);
    let transcript = TuiRect::new(
        content.x,
        content.y.saturating_add(1),
        content.width,
        content.height.saturating_sub(3),
    );
    let (start, end) = viewport.visible_rows();
    for (row, line) in lines[start.min(lines.len())..end.min(lines.len())]
        .iter()
        .take(usize::from(transcript.height))
        .enumerate()
    {
        surface.write_line(
            transcript,
            row as u16,
            &line.text,
            TextAlignment::Start,
            line.role,
        );
    }

    let prompt_region = TuiRect::new(
        content.x,
        content.bottom().saturating_sub(1),
        content.width,
        1,
    );
    surface.write_line(
        prompt_region,
        0,
        &prompt.display_text(),
        TextAlignment::Start,
        if prompt.focused {
            StyleRole::Accent
        } else {
            StyleRole::Muted
        },
    );

    ProjectionArtifact {
        schema: TUI_TOOLS_ARTIFACT_SCHEMA,
        producer: "tui-tools/embedded-console",
        extent,
        surface,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn lines() -> Vec<TranscriptLine> {
        ["one", "two", "three", "four"]
            .into_iter()
            .map(|text| TranscriptLine::new(text, StyleRole::Value))
            .collect()
    }

    #[test]
    fn prompt_stays_pinned_while_history_is_reviewed() {
        let lines = lines();
        let (mut viewport, _) = TuiViewport::new(2, lines.len() as u16);
        viewport.scroll_by(-1);
        let artifact = render_embedded_console(
            "CONSOLE",
            &lines,
            viewport,
            &ConsolePrompt::new(">", "help").focused(true),
            TuiExtent::new(24, 7),
        );
        let rendered = artifact.surface.to_plain_text();
        assert!(rendered.contains("two"));
        assert!(!rendered.contains("four"));
        assert!(rendered.contains("> help_"));
    }

    #[test]
    fn undersized_console_reports_its_constraint() {
        let (viewport, _) = TuiViewport::new(1, 0);
        let artifact = render_embedded_console(
            "CONSOLE",
            &[],
            viewport,
            &ConsolePrompt::new(">", ""),
            TuiExtent::new(12, 3),
        );
        assert!(artifact.surface.diagnostics().iter().any(|diagnostic| {
            matches!(
                diagnostic,
                TuiDiagnostic::Undersized {
                    axis: "vertical",
                    ..
                }
            )
        }));
    }
}
