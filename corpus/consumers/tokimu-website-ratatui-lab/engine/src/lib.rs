//! Browser-facing Ratatui template evidence.
//!
//! Template selection belongs to the website consumer. Ratatui owns terminal
//! composition, the retained Tokimu backend owns the bounded cell target, and
//! Tokimu's text provider produces the pixels returned to the browser.

mod backend;
mod raster;

use ratatui::{
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Gauge, List, ListItem, Paragraph, Wrap},
    Terminal,
};
use serde::Serialize;
use wasm_bindgen::prelude::*;

use backend::TokimuBackend;
use raster::{rasterize, CELL_PIXEL_HEIGHT, CELL_PIXEL_WIDTH};

const MIN_COLUMNS: u16 = 32;
const MIN_ROWS: u16 = 12;

#[wasm_bindgen]
pub fn template_snapshot(template: &str, columns: u16, rows: u16) -> Result<String, JsValue> {
    let template =
        Template::parse(template).ok_or_else(|| JsValue::from_str("unknown Ratatui template"))?;
    let snapshot = render_template(template, columns, rows)
        .map_err(|error| JsValue::from_str(&format!("render Ratatui template: {error}")))?;
    serde_json::to_string(&snapshot)
        .map_err(|error| JsValue::from_str(&format!("serialize Ratatui snapshot: {error}")))
}

#[wasm_bindgen]
pub fn template_frame_rgba(template: &str, columns: u16, rows: u16) -> Result<Vec<u8>, JsValue> {
    let template =
        Template::parse(template).ok_or_else(|| JsValue::from_str("unknown Ratatui template"))?;
    let buffer = compose_template(template, columns, rows)
        .map_err(|error| JsValue::from_str(&format!("compose Ratatui template: {error}")))?;
    rasterize(&buffer)
        .map(|frame| frame.rgba)
        .map_err(|error| JsValue::from_str(&format!("rasterize Tokimu frame: {error}")))
}

#[wasm_bindgen]
pub fn cell_pixel_width() -> u32 {
    CELL_PIXEL_WIDTH
}

#[wasm_bindgen]
pub fn cell_pixel_height() -> u32 {
    CELL_PIXEL_HEIGHT
}

#[wasm_bindgen]
pub fn template_catalog_json() -> Result<String, JsValue> {
    serde_json::to_string(&[
        TemplateSummary::new(Template::SystemMonitor),
        TemplateSummary::new(Template::AssetInspector),
        TemplateSummary::new(Template::CommandTranscript),
    ])
    .map_err(|error| JsValue::from_str(&format!("serialize Ratatui catalog: {error}")))
}

#[derive(Clone, Copy)]
enum Template {
    SystemMonitor,
    AssetInspector,
    CommandTranscript,
}

impl Template {
    fn parse(value: &str) -> Option<Self> {
        match value {
            "system-monitor" => Some(Self::SystemMonitor),
            "asset-inspector" => Some(Self::AssetInspector),
            "command-transcript" => Some(Self::CommandTranscript),
            _ => None,
        }
    }

    const fn id(self) -> &'static str {
        match self {
            Self::SystemMonitor => "system-monitor",
            Self::AssetInspector => "asset-inspector",
            Self::CommandTranscript => "command-transcript",
        }
    }

    const fn label(self) -> &'static str {
        match self {
            Self::SystemMonitor => "System monitor",
            Self::AssetInspector => "Asset inspector",
            Self::CommandTranscript => "Command transcript",
        }
    }

    const fn description(self) -> &'static str {
        match self {
            Self::SystemMonitor => "Synthetic scheduler, resource, and frame observations.",
            Self::AssetInspector => "A deterministic provider-neutral asset observation.",
            Self::CommandTranscript => "A bounded command-history and prompt layout.",
        }
    }
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct TemplateSummary {
    id: &'static str,
    label: &'static str,
    description: &'static str,
}

impl TemplateSummary {
    const fn new(template: Template) -> Self {
        Self {
            id: template.id(),
            label: template.label(),
            description: template.description(),
        }
    }
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct Snapshot {
    schema: u32,
    template: &'static str,
    label: &'static str,
    width: u16,
    height: u16,
    cells: Vec<Cell>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct Cell {
    x: u16,
    y: u16,
    symbol: String,
    foreground: &'static str,
    background: &'static str,
    modifiers: Vec<&'static str>,
}

fn render_template(template: Template, width: u16, height: u16) -> Result<Snapshot, String> {
    let buffer = compose_template(template, width, height)?;
    let cells = buffer
        .content()
        .iter()
        .enumerate()
        .map(|(index, cell)| Cell {
            x: (index % width as usize) as u16,
            y: (index / width as usize) as u16,
            symbol: cell.symbol().to_owned(),
            foreground: color_name(cell.fg),
            background: color_name(cell.bg),
            modifiers: modifier_names(cell.modifier),
        })
        .collect();

    Ok(Snapshot {
        schema: 2,
        template: template.id(),
        label: template.label(),
        width,
        height,
        cells,
    })
}

fn compose_template(
    template: Template,
    width: u16,
    height: u16,
) -> Result<ratatui::buffer::Buffer, String> {
    if width < MIN_COLUMNS || height < MIN_ROWS {
        return Err(format!(
            "Ratatui template requires at least {MIN_COLUMNS}x{MIN_ROWS} cells, received {width}x{height}"
        ));
    }

    let backend = TokimuBackend::new(width, height);
    let mut terminal = Terminal::new(backend).map_err(|error| error.to_string())?;
    terminal
        .draw(|frame| match template {
            Template::SystemMonitor => render_system_monitor(frame),
            Template::AssetInspector => render_asset_inspector(frame),
            Template::CommandTranscript => render_command_transcript(frame),
        })
        .map_err(|error| error.to_string())?;

    Ok(terminal.backend().buffer().clone())
}

fn render_system_monitor(frame: &mut ratatui::Frame) {
    let regions = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Min(7),
            Constraint::Length(3),
        ])
        .split(frame.area());
    frame.render_widget(
        Paragraph::new(Line::from(vec![
            Span::styled(
                " TOKIMU RUNTIME MONITOR ",
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw(" deterministic dummy observations"),
        ]))
        .block(Block::default().borders(Borders::ALL)),
        regions[0],
    );
    let columns = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(54), Constraint::Percentage(46)])
        .split(regions[1]);
    frame.render_widget(
        List::new(vec![
            ListItem::new(" scheduler: fixed-step / healthy"),
            ListItem::new(" entities: 128 / active: 17"),
            ListItem::new(" frame: 16.2 ms / budget: 16.7 ms"),
            ListItem::new(" diagnostics: 0 warnings"),
            ListItem::new(" transport: loopback / sequence: 42"),
        ])
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title("World observation"),
        ),
        columns[0],
    );
    frame.render_widget(
        Gauge::default()
            .block(Block::default().borders(Borders::ALL).title("Frame budget"))
            .gauge_style(Style::default().fg(Color::Green).bg(Color::Black))
            .ratio(0.72)
            .label("72% / 12.0 ms"),
        columns[1],
    );
    frame.render_widget(
        Paragraph::new("All values are deterministic fixture data. No host telemetry is read.")
            .style(Style::default().fg(Color::DarkGray))
            .block(Block::default().borders(Borders::ALL).title("Boundary")),
        regions[2],
    );
}

fn render_asset_inspector(frame: &mut ratatui::Frame) {
    let regions = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Min(7),
            Constraint::Length(3),
        ])
        .split(frame.area());
    frame.render_widget(
        Paragraph::new(" TOKIMU ASSET INSPECTOR / provider-neutral fixture")
            .style(
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            )
            .block(Block::default().borders(Borders::ALL)),
        regions[0],
    );
    let columns = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(44), Constraint::Percentage(56)])
        .split(regions[1]);
    frame.render_widget(
        List::new(vec![
            ListItem::new(" > Box.glb"),
            ListItem::new("   Box0.bin"),
            ListItem::new("   albedo.png"),
            ListItem::new("   notes/"),
        ])
        .style(Style::default().fg(Color::Cyan))
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title("Resource space"),
        ),
        columns[0],
    );
    frame.render_widget(
        Paragraph::new(vec![
            Line::from("kind: GLB / renderable preview"),
            Line::from("scenes: 1   meshes: 1   primitives: 1"),
            Line::from("triangles: 12   animations: 0"),
            Line::from("source: corpus fixture / no host path"),
            Line::from("diagnostics: no importer diagnostics"),
        ])
        .wrap(Wrap { trim: false })
        .block(Block::default().borders(Borders::ALL).title("Observation")),
        columns[1],
    );
    frame.render_widget(
        Paragraph::new(
            "Selection and inspect data are template fixtures; asset parsing is not invoked.",
        )
        .style(Style::default().fg(Color::DarkGray))
        .block(Block::default().borders(Borders::ALL).title("Boundary")),
        regions[2],
    );
}

fn render_command_transcript(frame: &mut ratatui::Frame) {
    let regions = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Min(7),
            Constraint::Length(3),
        ])
        .split(frame.area());
    frame.render_widget(
        Paragraph::new(" TOKIMU OBSERVATION SHELL / command transcript fixture")
            .style(
                Style::default()
                    .fg(Color::Green)
                    .add_modifier(Modifier::BOLD),
            )
            .block(Block::default().borders(Borders::ALL)),
        regions[0],
    );
    frame.render_widget(
        Paragraph::new(vec![
            Line::styled("> STATUS", Style::default().fg(Color::Cyan)),
            Line::from("world revision: 42 / selected entity: 7"),
            Line::styled("> INSPECT entity/7", Style::default().fg(Color::Cyan)),
            Line::from("components: Transform, Velocity, PresentationTarget"),
            Line::styled("> PLAY clip/idle", Style::default().fg(Color::Cyan)),
            Line::from("accepted: command outcome is application-owned evidence"),
        ])
        .wrap(Wrap { trim: false })
        .block(Block::default().borders(Borders::ALL).title("Transcript")),
        regions[1],
    );
    frame.render_widget(
        Paragraph::new("> _")
            .style(Style::default().fg(Color::Green))
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title("Prompt / static template"),
            ),
        regions[2],
    );
}

fn color_name(color: Color) -> &'static str {
    match color {
        Color::Reset => "reset",
        Color::Black => "black",
        Color::Red | Color::LightRed => "red",
        Color::Green | Color::LightGreen => "green",
        Color::Yellow | Color::LightYellow => "yellow",
        Color::Blue | Color::LightBlue => "blue",
        Color::Magenta | Color::LightMagenta => "magenta",
        Color::Cyan | Color::LightCyan => "cyan",
        Color::Gray | Color::DarkGray | Color::White => "white",
        Color::Rgb(_, _, _) | Color::Indexed(_) => "white",
    }
}

fn modifier_names(modifiers: Modifier) -> Vec<&'static str> {
    [
        (Modifier::BOLD, "bold"),
        (Modifier::DIM, "dim"),
        (Modifier::UNDERLINED, "underline"),
    ]
    .into_iter()
    .filter(|(flag, _)| modifiers.contains(*flag))
    .map(|(_, name)| name)
    .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_template_projects_a_bounded_complete_grid() {
        for template in [
            Template::SystemMonitor,
            Template::AssetInspector,
            Template::CommandTranscript,
        ] {
            let snapshot = render_template(template, 72, 24).expect("template snapshot");
            assert_eq!(snapshot.cells.len(), 72 * 24);
            assert_eq!(snapshot.template, template.id());
        }
    }

    #[test]
    fn undersized_grids_are_explicit_failures() {
        let error = render_template(Template::SystemMonitor, 31, 24).expect_err("small grid");
        assert!(error.contains("at least"));
    }

    #[test]
    fn tokimu_pixel_frames_are_bounded_and_repeatable() {
        let buffer = compose_template(Template::SystemMonitor, 72, 24).expect("composition");
        let first = rasterize(&buffer).expect("first frame");
        let second = rasterize(&buffer).expect("second frame");
        assert_eq!((first.width, first.height), (720, 432));
        assert_eq!(first.rgba.len(), 720 * 432 * 4);
        assert_eq!(first.fingerprint(), second.fingerprint());
    }
}
