mod host_input;
mod resource_browser;

use host_input::{map_host_input, HostInput, HostKey};
use resource_browser::ResourceBrowser;
use tui_tools::{
    rasterize_surface, render_embedded_console, render_status_dashboard, render_transcript,
    ConsolePrompt, StatusDashboard, StatusField, StatusSection, StyleRole, TranscriptLine,
    TuiAction, TuiActionOutcome, TuiExtent, TuiViewport,
};
use ui_tools::UiFontRasterizer;

const DEPARTURE_MONO: &[u8] = include_bytes!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../../../third-party/fonts/departure-mono/public/assets/DepartureMono-Regular.otf"
));

fn main() {
    let dashboard = StatusDashboard {
        title: "TOKIMU OPERATIONS BOARD".to_owned(),
        subtitle: "consumer-owned data / tui-tools-owned projection".to_owned(),
        sections: vec![
            StatusSection {
                title: "RUNTIME".to_owned(),
                fields: vec![
                    StatusField::new("frame", "1842"),
                    StatusField::new("fixed step", "16.67 ms"),
                    StatusField::new("entities", "128"),
                    StatusField::new("revision", "77"),
                ],
            },
            StatusSection {
                title: "PRESENTATION".to_owned(),
                fields: vec![
                    StatusField::new("draws", "42"),
                    StatusField::new("submits", "3"),
                    StatusField::new("surface", "72 x 24"),
                    StatusField::new("budget", "stable"),
                ],
            },
            StatusSection {
                title: "ASSET PIPELINE".to_owned(),
                fields: vec![
                    StatusField::new("queued", "2"),
                    StatusField::new("decoded", "19"),
                    StatusField::new("deferred", "1").emphasized(),
                ],
            },
            StatusSection {
                title: "DIAGNOSTICS".to_owned(),
                fields: vec![
                    StatusField::new("info", "12"),
                    StatusField::new("warning", "1").emphasized(),
                    StatusField::new("error", "0"),
                ],
            },
        ],
        footer: "Q quit | R reset | arrows inspect | provider: tui-tools".to_owned(),
    };

    let normal = render_status_dashboard(&dashboard, TuiExtent::new(72, 24));
    println!("{}", normal.surface.to_plain_text());
    println!(
        "\nartifact schema={} producer={} cells={} diagnostics={}",
        normal.schema,
        normal.producer,
        normal.surface.cells().len(),
        normal.surface.diagnostics().len()
    );

    let constrained = render_status_dashboard(&dashboard, TuiExtent::new(24, 6));
    println!(
        "undersized evidence: extent={}x{}, diagnostics={:?}",
        constrained.extent.columns,
        constrained.extent.rows,
        constrained.surface.diagnostics()
    );

    let mut lines = vec![
        TranscriptLine::new("[system] runtime attached", StyleRole::Accent),
        TranscriptLine::new("[asset] waiting for source", StyleRole::Muted),
        TranscriptLine::new("[world] revision advanced to 77", StyleRole::Value),
        TranscriptLine::new("[warning] one deferred resource", StyleRole::Warning),
        TranscriptLine::new("[system] diagnostics remain explicit", StyleRole::Value),
        TranscriptLine::new(
            "[shell] review mode preserves this history",
            StyleRole::Label,
        ),
    ];
    let (mut viewport, _) = TuiViewport::new(4, lines.len() as u16);
    let host_previous = map_host_input(HostInput::Key(HostKey::ArrowUp))
        .expect("the corpus host maps ArrowUp to a normalized action");
    assert_eq!(
        viewport.apply_action(&host_previous),
        TuiActionOutcome::Applied
    );
    assert_eq!(
        viewport.apply_action(&TuiAction::PagePrevious),
        TuiActionOutcome::Applied
    );
    lines.push(TranscriptLine::new(
        "[asset] source decode completed while history review remained active",
        StyleRole::Value,
    ));
    viewport.append_rows(1);
    let review_transcript = render_transcript(
        "TRANSCRIPT / REVIEW MODE",
        &lines,
        viewport,
        TuiExtent::new(52, 7),
    );
    println!("\n{}", review_transcript.surface.to_plain_text());
    println!(
        "viewport evidence: offset={} tail={} visible={:?}; host ArrowUp -> {:?}",
        viewport.offset(),
        viewport.live_tail(),
        viewport.visible_rows(),
        host_previous
    );

    assert_eq!(
        viewport.apply_action(&TuiAction::End),
        TuiActionOutcome::Applied
    );
    let live_transcript = render_transcript(
        "TRANSCRIPT / LIVE TAIL",
        &lines,
        viewport,
        TuiExtent::new(52, 7),
    );
    println!("\n{}", live_transcript.surface.to_plain_text());
    println!(
        "live-tail evidence: offset={} tail={} visible={:?}",
        viewport.offset(),
        viewport.live_tail(),
        viewport.visible_rows()
    );

    let (mut console_viewport, _) = TuiViewport::new(4, lines.len() as u16);
    assert_eq!(
        console_viewport.apply_action(&TuiAction::PagePrevious),
        TuiActionOutcome::Applied
    );
    let console = render_embedded_console(
        "EMBEDDED CONSOLE / CALLER-OWNED VIEW DATA",
        &lines,
        console_viewport,
        &ConsolePrompt::new(">", "status").focused(true),
        TuiExtent::new(60, 10),
    );
    println!("\n{}", console.surface.to_plain_text());
    println!(
        "embedded-console evidence: offset={} tail={} prompt=caller-owned",
        console_viewport.offset(),
        console_viewport.live_tail()
    );
    let font = UiFontRasterizer::from_bytes(DEPARTURE_MONO.to_vec())
        .expect("load Departure Mono corpus provider");
    let frame = rasterize_surface(&console.surface, &font)
        .expect("rasterize console through the shared Tokimu seam");
    println!(
        "shared-raster evidence: {}x{} rgba={} fingerprint={:016x}",
        frame.width,
        frame.height,
        frame.rgba.len(),
        frame.fingerprint()
    );

    let mut browser = ResourceBrowser::fixture();
    assert_eq!(
        browser.apply(&TuiAction::InsertText("box".to_owned())),
        TuiActionOutcome::Applied
    );
    assert_eq!(
        browser.apply(&TuiAction::FocusNext),
        TuiActionOutcome::Applied
    );
    assert_eq!(
        browser.apply(&TuiAction::MoveNext),
        TuiActionOutcome::Applied
    );
    assert_eq!(
        browser.apply(&TuiAction::FocusNext),
        TuiActionOutcome::Applied
    );
    println!(
        "\n{}",
        browser.render(TuiExtent::new(40, 10)).to_plain_text()
    );
    println!(
        "resource-browser evidence: filter={:?} selected={} focus={} activation=caller-owned",
        browser.filter(),
        browser.selected_resource(),
        browser.focused_region().unwrap_or("none")
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn embedded_console_uses_the_shared_tokimu_text_raster_seam() {
        let lines = [TranscriptLine::new(
            "[system] raster seam",
            StyleRole::Value,
        )];
        let (viewport, _) = TuiViewport::new(1, 1);
        let console = render_embedded_console(
            "CONSOLE",
            &lines,
            viewport,
            &ConsolePrompt::new(">", "status"),
            TuiExtent::new(32, 8),
        );
        let font = UiFontRasterizer::from_bytes(DEPARTURE_MONO.to_vec())
            .expect("load Departure Mono corpus provider");
        let first = rasterize_surface(&console.surface, &font).expect("first raster");
        let second = rasterize_surface(&console.surface, &font).expect("second raster");
        assert_eq!((first.width, first.height), (320, 144));
        assert_eq!(first.rgba.len(), 320 * 144 * 4);
        assert_eq!(first.fingerprint(), second.fingerprint());
    }

    #[test]
    fn normalized_navigation_keeps_prompt_and_application_actions_caller_owned() {
        let lines = [
            TranscriptLine::new("one", StyleRole::Value),
            TranscriptLine::new("two", StyleRole::Value),
            TranscriptLine::new("three", StyleRole::Value),
        ];
        let (mut viewport, _) = TuiViewport::new(1, lines.len() as u16);

        assert_eq!(
            viewport.apply_action(&TuiAction::MovePrevious),
            TuiActionOutcome::Applied
        );
        assert_eq!(viewport.offset(), 1);
        assert_eq!(
            viewport.apply_action(&TuiAction::Activate),
            TuiActionOutcome::Unhandled
        );
        assert_eq!(viewport.offset(), 1);
    }

    #[test]
    fn resource_browser_fixture_exercises_form_and_selection_without_a_widget_api() {
        let mut browser = ResourceBrowser::fixture();
        assert_eq!(
            browser.apply(&TuiAction::InsertText("poly".to_owned())),
            TuiActionOutcome::Applied
        );
        assert_eq!(
            browser.apply(&TuiAction::FocusNext),
            TuiActionOutcome::Applied
        );
        assert_eq!(
            browser.apply(&TuiAction::MoveNext),
            TuiActionOutcome::Applied
        );
        assert_eq!(browser.selected_resource(), "POLYLN01.cgm");
        assert!(browser
            .render(TuiExtent::new(40, 10))
            .to_plain_text()
            .contains("POLYLN01.cgm"));
    }
}
