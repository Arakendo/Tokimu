//! Feature-gated Ratatui comparison evidence.
//!
//! Ratatui provides a composition oracle only. The public report below uses
//! provider-neutral fixture inputs and findings, so Ratatui types never escape
//! the optional comparison boundary.

use ratatui::{
    backend::TestBackend,
    layout::Rect,
    style::{Color, Style},
    widgets::{Block, Borders, Paragraph, Wrap},
    Terminal,
};
use ui_tools::UiFontRasterizer;

use crate::{
    rasterize_ratatui_buffer, rasterize_surface, render_embedded_console, render_status_dashboard,
    ConsolePrompt, StatusDashboard, TranscriptLine, TuiExtent, TuiRasterFrame, TuiRasterOptions,
    TuiViewport,
};

const STATUS_DASHBOARD_MIN_COLUMNS: u16 = 8;
const STATUS_DASHBOARD_MIN_ROWS: u16 = 6;
const DEPARTURE_MONO: &[u8] = include_bytes!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../../third-party/fonts/departure-mono/public/assets/DepartureMono-Regular.otf"
));

/// The bounded composition path which produced an oracle observation.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TuiOracleProvider {
    /// Tokimu's corpus-local resolved-cell projection.
    Tokimu,
    /// Ratatui's feature-gated headless composition oracle.
    Ratatui,
}

/// A narrow, provider-local difference that a paired oracle records without
/// treating it as a shared terminal-semantic failure.
///
/// This is deliberately not a catch-all suppression mechanism. New kinds need
/// a code review, and every record must state why the difference is outside the
/// caller-owned contract being compared.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ExpectedDivergenceKind {
    /// Providers select different border glyphs or border placement details.
    BorderComposition,
    /// Providers select different colors, modifiers, or style encodings.
    StyleComposition,
}

/// A reasoned record for one expected provider-local difference.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ExpectedDivergence {
    pub kind: ExpectedDivergenceKind,
    pub reason: String,
}

impl ExpectedDivergence {
    /// Creates a divergence record only when its reason is explicit.
    pub fn new(kind: ExpectedDivergenceKind, reason: impl Into<String>) -> Result<Self, String> {
        let reason = reason.into();
        if reason.trim().is_empty() {
            return Err("expected divergence records require a non-empty reason".to_owned());
        }
        Ok(Self { kind, reason })
    }
}

/// A semantic invariant that one bounded terminal projection did not satisfy.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ConsoleOracleFinding {
    /// A transcript line selected by the shared viewport was not visible.
    MissingVisibleLine {
        provider: TuiOracleProvider,
        line: String,
    },
    /// A provider did not render the caller-owned prompt text.
    MissingPrompt { provider: TuiOracleProvider },
    /// The prompt did not remain on the expected terminal row.
    PromptNotPinned {
        provider: TuiOracleProvider,
        expected_row: u16,
        observed_row: Option<u16>,
    },
}

/// A status-dashboard fact was lost by one bounded composition path.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum StatusOracleFinding {
    /// Caller-owned dashboard text was not present in a provider projection.
    MissingText {
        provider: TuiOracleProvider,
        text: String,
    },
}

/// A renderer-seam invariant that one provider path did not satisfy.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum RasterOracleFinding {
    /// The shared raster output did not match the requested terminal extent.
    UnexpectedDimensions {
        provider: TuiOracleProvider,
        width: u32,
        height: u32,
    },
    /// The rendered frame did not contain any content beyond the canvas color.
    EmptyFrame { provider: TuiOracleProvider },
    /// Repeating the same CPU raster work changed its fingerprint.
    NondeterministicFrame { provider: TuiOracleProvider },
}

/// Provider-neutral result of a paired embedded-console comparison.
///
/// Borders, terminal symbols, and style encodings are intentionally excluded:
/// they are provider implementation details. The report compares only the
/// shared caller-owned transcript, viewport, and prompt semantics.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ConsoleOracleReport {
    pub extent: TuiExtent,
    pub visible_line_count: usize,
    pub expected_prompt_row: u16,
    pub findings: Vec<ConsoleOracleFinding>,
}

impl ConsoleOracleReport {
    /// Returns true when both providers satisfied all shared invariants.
    pub fn matches(&self) -> bool {
        self.findings.is_empty()
    }
}

/// Provider-neutral result of a paired status-dashboard comparison.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct StatusOracleReport {
    pub extent: TuiExtent,
    pub expected_text_count: usize,
    /// Named provider-local behavior intentionally excluded from semantic comparison.
    pub expected_divergences: Vec<ExpectedDivergence>,
    pub findings: Vec<StatusOracleFinding>,
}

impl StatusOracleReport {
    /// Returns true when both providers retained every shared dashboard fact.
    pub fn matches(&self) -> bool {
        self.findings.is_empty()
    }
}

/// Provider-neutral evidence that both composition paths reach the same CPU
/// cell-to-RGBA seam with stable, bounded output.
///
/// The two fingerprints are intentionally reported independently. Ratatui and
/// the corpus-local composition path own different border and style choices,
/// so matching fingerprints would be an invalid visual-equality contract.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RasterOracleReport {
    pub extent: TuiExtent,
    pub expected_width: u32,
    pub expected_height: u32,
    pub expected_rgba_bytes: usize,
    pub tokimu_fingerprint: u64,
    pub ratatui_fingerprint: u64,
    /// Named provider-local behavior intentionally excluded from pixel comparison.
    pub expected_divergences: Vec<ExpectedDivergence>,
    pub findings: Vec<RasterOracleFinding>,
}

impl RasterOracleReport {
    /// Returns true when both paths reached the shared raster seam cleanly.
    pub fn matches(&self) -> bool {
        self.findings.is_empty()
    }
}

/// Renders caller-owned console data through the Tokimu and Ratatui paths,
/// then compares only their shared semantic invariants.
pub fn compare_embedded_console(
    title: &str,
    lines: &[TranscriptLine],
    viewport: TuiViewport,
    prompt: &ConsolePrompt,
    extent: TuiExtent,
) -> Result<ConsoleOracleReport, String> {
    let tokimu = render_embedded_console(title, lines, viewport, prompt, extent)
        .surface
        .to_plain_text();
    let ratatui = render_console(title, lines, viewport, prompt, extent)?;
    let (start, end) = viewport.visible_rows();
    let visible = &lines[start.min(lines.len())..end.min(lines.len())];
    let expected_prompt_row = extent.rows.saturating_sub(2);
    let prompt = prompt_display(prompt);
    let mut findings = Vec::new();

    for line in visible {
        record_missing_line(
            &mut findings,
            TuiOracleProvider::Tokimu,
            &tokimu,
            &line.text,
        );
        record_missing_line(
            &mut findings,
            TuiOracleProvider::Ratatui,
            &ratatui,
            &line.text,
        );
    }
    record_prompt(
        &mut findings,
        TuiOracleProvider::Tokimu,
        &tokimu,
        &prompt,
        expected_prompt_row,
    );
    record_prompt(
        &mut findings,
        TuiOracleProvider::Ratatui,
        &ratatui,
        &prompt,
        expected_prompt_row,
    );

    Ok(ConsoleOracleReport {
        extent,
        visible_line_count: visible.len(),
        expected_prompt_row,
        findings,
    })
}

/// Renders a caller-owned status dashboard through Tokimu and Ratatui, then
/// checks that titles, fields, and footer facts survive both bounded layouts.
///
/// This does not compare border glyphs, padding, or terminal styles. Those
/// are provider details rather than dashboard semantics.
pub fn compare_status_dashboard(
    dashboard: &StatusDashboard,
    extent: TuiExtent,
) -> Result<StatusOracleReport, String> {
    require_status_dashboard_extent(extent)?;

    let tokimu = render_status_dashboard(dashboard, extent)
        .surface
        .to_plain_text();
    let ratatui = render_status_dashboard_ratatui(dashboard, extent)?;
    let expected = dashboard_text(dashboard);
    let mut findings = Vec::new();

    for text in &expected {
        record_missing_status_text(&mut findings, TuiOracleProvider::Tokimu, &tokimu, text);
        record_missing_status_text(&mut findings, TuiOracleProvider::Ratatui, &ratatui, text);
    }

    Ok(StatusOracleReport {
        extent,
        expected_text_count: expected.len(),
        expected_divergences: dashboard_expected_divergences(),
        findings,
    })
}

/// Renders the same status dashboard through both composition paths and then
/// through the shared CPU raster seam.
///
/// This is renderer-seam evidence, not visual parity evidence. It verifies
/// that provider-local cell composition produces bounded, repeatable RGBA
/// artifacts through the same Tokimu text-raster execution path.
pub fn compare_status_dashboard_raster(
    dashboard: &StatusDashboard,
    extent: TuiExtent,
) -> Result<RasterOracleReport, String> {
    require_status_dashboard_extent(extent)?;

    let font = UiFontRasterizer::from_bytes(DEPARTURE_MONO.to_vec())
        .map_err(|error| format!("load Departure Mono raster provider: {error}"))?;
    let local_surface = render_status_dashboard(dashboard, extent).surface;
    let tokimu = rasterize_surface(&local_surface, &font)?;
    let tokimu_repeat = rasterize_surface(&local_surface, &font)?;
    let ratatui = rasterize_status_dashboard_ratatui(dashboard, extent, &font)?;
    let ratatui_repeat = rasterize_status_dashboard_ratatui(dashboard, extent, &font)?;

    let expected_width = u32::from(extent.columns) * crate::CELL_PIXEL_WIDTH;
    let expected_height = u32::from(extent.rows) * crate::CELL_PIXEL_HEIGHT;
    let expected_rgba_bytes = expected_width as usize * expected_height as usize * 4;
    let mut findings = Vec::new();
    record_raster_findings(
        &mut findings,
        TuiOracleProvider::Tokimu,
        &tokimu,
        &tokimu_repeat,
        expected_width,
        expected_height,
        expected_rgba_bytes,
    );
    record_raster_findings(
        &mut findings,
        TuiOracleProvider::Ratatui,
        &ratatui,
        &ratatui_repeat,
        expected_width,
        expected_height,
        expected_rgba_bytes,
    );

    Ok(RasterOracleReport {
        extent,
        expected_width,
        expected_height,
        expected_rgba_bytes,
        tokimu_fingerprint: tokimu.fingerprint(),
        ratatui_fingerprint: ratatui.fingerprint(),
        expected_divergences: dashboard_expected_divergences(),
        findings,
    })
}

/// Publishes the smallest terminal grid that the paired dashboard oracle can
/// compare as a complete fixture. This is an oracle admission rule, not a
/// general Tokimu layout constraint: the corpus-local dashboard may still
/// emit diagnostic degraded output below this size.
fn require_status_dashboard_extent(extent: TuiExtent) -> Result<(), String> {
    if extent.columns < STATUS_DASHBOARD_MIN_COLUMNS || extent.rows < STATUS_DASHBOARD_MIN_ROWS {
        return Err(format!(
            "Ratatui status dashboard requires at least {}x{} cells; received {}x{}",
            STATUS_DASHBOARD_MIN_COLUMNS, STATUS_DASHBOARD_MIN_ROWS, extent.columns, extent.rows,
        ));
    }
    Ok(())
}

fn dashboard_expected_divergences() -> Vec<ExpectedDivergence> {
    vec![
        ExpectedDivergence::new(
            ExpectedDivergenceKind::BorderComposition,
            "Ratatui and the corpus-local dashboard own their border glyphs and placement; dashboard facts do not require matching frames.",
        )
        .expect("static expected-divergence reason must be valid"),
        ExpectedDivergence::new(
            ExpectedDivergenceKind::StyleComposition,
            "Ratatui styles and corpus-local style roles are provider-local; shared evidence requires readable caller-owned text, not matching pixels.",
        )
        .expect("static expected-divergence reason must be valid"),
    ]
}

fn record_raster_findings(
    findings: &mut Vec<RasterOracleFinding>,
    provider: TuiOracleProvider,
    frame: &TuiRasterFrame,
    repeated: &TuiRasterFrame,
    expected_width: u32,
    expected_height: u32,
    expected_rgba_bytes: usize,
) {
    if frame.width != expected_width
        || frame.height != expected_height
        || frame.rgba.len() != expected_rgba_bytes
    {
        findings.push(RasterOracleFinding::UnexpectedDimensions {
            provider,
            width: frame.width,
            height: frame.height,
        });
    }
    if !frame
        .rgba
        .chunks_exact(4)
        .any(|pixel| pixel != TuiRasterOptions::DEFAULT.canvas)
    {
        findings.push(RasterOracleFinding::EmptyFrame { provider });
    }
    if frame.fingerprint() != repeated.fingerprint() {
        findings.push(RasterOracleFinding::NondeterministicFrame { provider });
    }
}

fn record_missing_line(
    findings: &mut Vec<ConsoleOracleFinding>,
    provider: TuiOracleProvider,
    rendered: &str,
    line: &str,
) {
    if !rendered.lines().any(|row| row.contains(line)) {
        findings.push(ConsoleOracleFinding::MissingVisibleLine {
            provider,
            line: line.to_owned(),
        });
    }
}

fn record_prompt(
    findings: &mut Vec<ConsoleOracleFinding>,
    provider: TuiOracleProvider,
    rendered: &str,
    prompt: &str,
    expected_row: u16,
) {
    let observed_row = rendered
        .lines()
        .position(|row| row.contains(prompt))
        .map(|row| row as u16);
    match observed_row {
        None => findings.push(ConsoleOracleFinding::MissingPrompt { provider }),
        Some(row) if row != expected_row => findings.push(ConsoleOracleFinding::PromptNotPinned {
            provider,
            expected_row,
            observed_row: Some(row),
        }),
        Some(_) => {}
    }
}

fn record_missing_status_text(
    findings: &mut Vec<StatusOracleFinding>,
    provider: TuiOracleProvider,
    rendered: &str,
    text: &str,
) {
    if !rendered.lines().any(|row| row.contains(text)) {
        findings.push(StatusOracleFinding::MissingText {
            provider,
            text: text.to_owned(),
        });
    }
}

/// Renders the same caller-owned console inputs through Ratatui's headless
/// backend. It returns text only for oracle assertions.
fn render_console(
    title: &str,
    lines: &[TranscriptLine],
    viewport: TuiViewport,
    prompt: &ConsolePrompt,
    extent: TuiExtent,
) -> Result<String, String> {
    if extent.columns < 8 || extent.rows < 5 {
        return Err("Ratatui console oracle requires at least an 8x5 surface".to_owned());
    }

    let backend = TestBackend::new(extent.columns, extent.rows);
    let mut terminal = Terminal::new(backend).map_err(|error| error.to_string())?;
    terminal
        .draw(|frame| {
            let area = frame.area();
            frame.render_widget(
                Block::default()
                    .borders(Borders::ALL)
                    .title(title)
                    .style(Style::default().fg(Color::Cyan)),
                area,
            );

            let inner = Rect::new(
                1,
                1,
                area.width.saturating_sub(2),
                area.height.saturating_sub(2),
            );
            let transcript_height = inner.height.saturating_sub(1);
            let transcript = Rect::new(inner.x, inner.y, inner.width, transcript_height);
            let (start, end) = viewport.visible_rows();
            let visible = lines[start.min(lines.len())..end.min(lines.len())]
                .iter()
                .map(|line| line.text.as_str())
                .collect::<Vec<_>>()
                .join("\n");
            frame.render_widget(
                Paragraph::new(visible).wrap(Wrap { trim: false }),
                transcript,
            );
            frame.render_widget(
                Paragraph::new(prompt_display(prompt)),
                Rect::new(inner.x, inner.bottom().saturating_sub(1), inner.width, 1),
            );
        })
        .map_err(|error| error.to_string())?;

    let columns = usize::from(extent.columns);
    Ok(terminal
        .backend()
        .buffer()
        .content
        .chunks(columns)
        .map(|row| {
            row.iter()
                .map(|cell| cell.symbol())
                .collect::<Vec<_>>()
                .join("")
        })
        .collect::<Vec<_>>()
        .join("\n"))
}

/// Uses Ratatui only as an internal layout oracle for the same dashboard
/// input. The returned plain text is intentionally the sole comparison data.
fn render_status_dashboard_ratatui(
    dashboard: &StatusDashboard,
    extent: TuiExtent,
) -> Result<String, String> {
    let backend = TestBackend::new(extent.columns, extent.rows);
    let mut terminal = Terminal::new(backend).map_err(|error| error.to_string())?;
    terminal
        .draw(|frame| render_status_dashboard_widget(frame, dashboard))
        .map_err(|error| error.to_string())?;

    let columns = usize::from(extent.columns);
    Ok(terminal
        .backend()
        .buffer()
        .content
        .chunks(columns)
        .map(|row| {
            row.iter()
                .map(|cell| cell.symbol())
                .collect::<Vec<_>>()
                .join("")
        })
        .collect::<Vec<_>>()
        .join("\n"))
}

/// Composes the dashboard once so text and raster oracle paths observe the
/// identical Ratatui buffer layout.
fn render_status_dashboard_widget(frame: &mut ratatui::Frame, dashboard: &StatusDashboard) {
    let area = frame.area();
    frame.render_widget(
        Block::default()
            .borders(Borders::ALL)
            .style(Style::default().fg(Color::Cyan)),
        area,
    );
    let content = Rect::new(
        1,
        1,
        area.width.saturating_sub(2),
        area.height.saturating_sub(2),
    );
    let header = Rect::new(content.x, content.y, content.width, 2.min(content.height));
    frame.render_widget(Paragraph::new(dashboard.title.as_str()), header);
    if header.height > 1 {
        frame.render_widget(
            Paragraph::new(dashboard.subtitle.as_str()),
            Rect::new(header.x, header.y.saturating_add(1), header.width, 1),
        );
    }

    let footer = Rect::new(
        content.x,
        content.bottom().saturating_sub(1),
        content.width,
        1,
    );
    frame.render_widget(Paragraph::new(dashboard.footer.as_str()), footer);

    let body = Rect::new(
        content.x,
        content.y.saturating_add(2),
        content.width,
        content.height.saturating_sub(3),
    );
    let left_width = body.width / 2;
    let columns = [
        Rect::new(body.x, body.y, left_width, body.height),
        Rect::new(
            body.x.saturating_add(left_width),
            body.y,
            body.width.saturating_sub(left_width),
            body.height,
        ),
    ];
    for (section_index, section) in dashboard.sections.iter().enumerate() {
        let column = columns[section_index % columns.len()];
        let row_offset = (section_index / columns.len()) as u16 * 7;
        if row_offset >= column.height {
            continue;
        }
        let section_area = Rect::new(
            column.x.saturating_add(1),
            column.y.saturating_add(row_offset),
            column.width.saturating_sub(2),
            column.height.saturating_sub(row_offset).min(7),
        );
        frame.render_widget(Paragraph::new(section.title.as_str()), section_area);
        for (row, field) in section.fields.iter().take(5).enumerate() {
            frame.render_widget(
                Paragraph::new(format!("{:<12} {}", field.label, field.value)),
                Rect::new(
                    section_area.x,
                    section_area.y.saturating_add(row as u16 + 1),
                    section_area.width,
                    1,
                ),
            );
        }
    }
}

fn rasterize_status_dashboard_ratatui(
    dashboard: &StatusDashboard,
    extent: TuiExtent,
    font: &UiFontRasterizer,
) -> Result<TuiRasterFrame, String> {
    let backend = TestBackend::new(extent.columns, extent.rows);
    let mut terminal = Terminal::new(backend).map_err(|error| error.to_string())?;
    terminal
        .draw(|frame| render_status_dashboard_widget(frame, dashboard))
        .map_err(|error| error.to_string())?;
    rasterize_ratatui_buffer(terminal.backend().buffer(), font)
}

fn dashboard_text(dashboard: &StatusDashboard) -> Vec<String> {
    let mut text = vec![
        dashboard.title.clone(),
        dashboard.subtitle.clone(),
        dashboard.footer.clone(),
    ];
    for section in &dashboard.sections {
        text.push(section.title.clone());
        for field in section.fields.iter().take(5) {
            text.push(field.label.clone());
            text.push(field.value.clone());
        }
    }
    text
}

fn prompt_display(prompt: &ConsolePrompt) -> String {
    let cursor = if prompt.focused && prompt.cursor_visible {
        "_"
    } else {
        ""
    };
    format!("{} {}{}", prompt.prefix, prompt.input, cursor)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::StyleRole;

    fn transcript() -> Vec<TranscriptLine> {
        ["one", "two", "three", "four"]
            .into_iter()
            .map(|text| TranscriptLine::new(text, StyleRole::Value))
            .collect()
    }

    fn dashboard() -> StatusDashboard {
        StatusDashboard {
            title: "SYSTEM MONITOR".to_owned(),
            subtitle: "bounded synthetic evidence".to_owned(),
            sections: vec![
                crate::StatusSection {
                    title: "RUNTIME".to_owned(),
                    fields: vec![
                        crate::StatusField::new("frame", "42"),
                        crate::StatusField::new("status", "ready"),
                    ],
                },
                crate::StatusSection {
                    title: "RENDER".to_owned(),
                    fields: vec![crate::StatusField::new("draws", "3")],
                },
            ],
            footer: "Q quit | R reset".to_owned(),
        }
    }

    fn asset_inspector_dashboard() -> StatusDashboard {
        StatusDashboard {
            title: "ASSET INSPECTOR".to_owned(),
            subtitle: "provider-neutral asset observation".to_owned(),
            sections: vec![
                crate::StatusSection {
                    title: "ASSET".to_owned(),
                    fields: vec![
                        crate::StatusField::new("name", "Box.glb"),
                        crate::StatusField::new("kind", "glb"),
                    ],
                },
                crate::StatusSection {
                    title: "STRUCTURE".to_owned(),
                    fields: vec![
                        crate::StatusField::new("meshes", "1"),
                        crate::StatusField::new("primitives", "1"),
                    ],
                },
            ],
            footer: "Enter inspect | Esc return".to_owned(),
        }
    }

    #[test]
    fn paired_console_oracle_agrees_on_transcript_and_pinned_prompt() {
        let lines = transcript();
        let (mut viewport, _) = TuiViewport::new(2, lines.len() as u16);
        viewport.scroll_by(-1);

        let report = compare_embedded_console(
            "CONSOLE",
            &lines,
            viewport,
            &ConsolePrompt::new(">", "help").focused(true),
            TuiExtent::new(28, 7),
        )
        .unwrap();

        assert_eq!(report.visible_line_count, 2);
        assert_eq!(report.expected_prompt_row, 5);
        assert!(
            report.matches(),
            "unexpected findings: {:?}",
            report.findings
        );
    }

    #[test]
    fn paired_console_oracle_agrees_after_normalized_viewport_navigation() {
        let lines = transcript();
        let (mut viewport, _) = TuiViewport::new(2, lines.len() as u16);
        let prompt = ConsolePrompt::new(">", "status").focused(true);
        let extent = TuiExtent::new(28, 7);

        for (action, expected_offset, expected_live_tail) in [
            (crate::TuiAction::MovePrevious, 1, false),
            (crate::TuiAction::PagePrevious, 0, false),
            (crate::TuiAction::End, 2, true),
        ] {
            assert_eq!(
                viewport.apply_action(&action),
                crate::TuiActionOutcome::Applied
            );
            assert_eq!(viewport.offset(), expected_offset);
            assert_eq!(viewport.live_tail(), expected_live_tail);

            let report =
                compare_embedded_console("CONSOLE", &lines, viewport, &prompt, extent).unwrap();
            assert!(
                report.matches(),
                "unexpected findings after {action:?}: {:?}",
                report.findings
            );
        }
    }

    #[test]
    fn oracle_requires_a_bounded_surface() {
        let (viewport, _) = TuiViewport::new(1, 0);
        let error = compare_embedded_console(
            "CONSOLE",
            &[],
            viewport,
            &ConsolePrompt::new(">", ""),
            TuiExtent::new(7, 4),
        )
        .unwrap_err();

        assert!(error.contains("8x5"));
    }

    #[test]
    fn paired_dashboard_oracle_preserves_facts_across_resize() {
        for extent in [TuiExtent::new(48, 14), TuiExtent::new(64, 18)] {
            let report = compare_status_dashboard(&dashboard(), extent).unwrap();
            assert!(
                report.matches(),
                "unexpected findings at {:?}: {:?}",
                extent,
                report.findings
            );
            assert_eq!(report.expected_text_count, 11);
            assert_eq!(report.expected_divergences.len(), 2);
            assert!(report
                .expected_divergences
                .iter()
                .all(|record| !record.reason.trim().is_empty()));
        }
    }

    #[test]
    fn paired_dashboard_raster_oracle_reaches_the_shared_cpu_seam() {
        for extent in [TuiExtent::new(48, 14), TuiExtent::new(64, 18)] {
            let report = compare_status_dashboard_raster(&dashboard(), extent).unwrap();
            assert!(
                report.matches(),
                "unexpected raster findings at {:?}: {:?}",
                extent,
                report.findings
            );
            assert_eq!(report.expected_width, u32::from(extent.columns) * 10);
            assert_eq!(report.expected_height, u32::from(extent.rows) * 18);
            assert_eq!(
                report.expected_rgba_bytes,
                report.expected_width as usize * report.expected_height as usize * 4
            );
            assert_ne!(report.tokimu_fingerprint, 0);
            assert_ne!(report.ratatui_fingerprint, 0);
            assert_eq!(report.expected_divergences.len(), 2);
        }
    }

    #[test]
    fn paired_asset_inspector_oracle_preserves_provider_neutral_facts() {
        for extent in [TuiExtent::new(48, 14), TuiExtent::new(64, 18)] {
            let dashboard = asset_inspector_dashboard();
            let report = compare_status_dashboard(&dashboard, extent).unwrap();
            assert!(
                report.matches(),
                "unexpected asset-inspector findings at {:?}: {:?}",
                extent,
                report.findings
            );
            assert_eq!(report.expected_text_count, 13);

            let raster = compare_status_dashboard_raster(&dashboard, extent).unwrap();
            assert!(
                raster.matches(),
                "unexpected asset-inspector raster findings at {:?}: {:?}",
                extent,
                raster.findings
            );
            assert_ne!(raster.tokimu_fingerprint, 0);
            assert_ne!(raster.ratatui_fingerprint, 0);
        }
    }

    #[test]
    fn paired_dashboard_oracles_publish_one_undersized_extent_boundary() {
        let dashboard = dashboard();
        for extent in [
            TuiExtent::new(7, 6),
            TuiExtent::new(8, 5),
            TuiExtent::new(1, 1),
        ] {
            let layout_error = compare_status_dashboard(&dashboard, extent).unwrap_err();
            let raster_error = compare_status_dashboard_raster(&dashboard, extent).unwrap_err();

            assert_eq!(layout_error, raster_error);
            assert!(layout_error.contains("at least 8x6 cells"));
            assert!(layout_error.contains(&format!("{}x{}", extent.columns, extent.rows)));
        }
    }

    #[test]
    fn paired_dashboard_oracles_accept_their_published_minimum_extent() {
        let dashboard = dashboard();
        assert!(compare_status_dashboard(&dashboard, TuiExtent::new(8, 6)).is_ok());
        assert!(compare_status_dashboard_raster(&dashboard, TuiExtent::new(8, 6)).is_ok());
    }

    #[test]
    fn expected_divergence_requires_a_reason() {
        let error =
            ExpectedDivergence::new(ExpectedDivergenceKind::BorderComposition, "  ").unwrap_err();

        assert!(error.contains("non-empty reason"));
    }
}
