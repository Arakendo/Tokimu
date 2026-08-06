//! Emits provider-neutral cell-layout evidence from a headless Ratatui frame.

use std::{fs, path::PathBuf};

use screenshot::{write_bmp, write_manifest, Rgba8Image};
use tokimu_console_command_window::{
    cell_grid_raster::{rasterize_cell_layout, CellGridRasterOptions},
    ratatui_projection::render_session,
    tokimu_cell_projection::{lower_cells_with_options, CellLoweringOptions, CellSelection},
    tosumu_session::TosumuSession,
};
use ui_tools::{UiFontRasterizer, UiFontSource};

const SCRIPT: [&str; 6] = [
    "STATUS",
    "CHECK",
    "DESCRIBE demo/message",
    "DESCRIBE missing/key",
    "WAL STATUS",
    "STATUS trailing",
];

fn main() -> Result<(), String> {
    let session = TosumuSession::open_fixture()?;
    let evidence = session.run_script(&SCRIPT);
    let snapshot = render_session(&evidence, 96, 28)?;
    let font = UiFontSource::from_native_default()
        .map_err(|error| format!("resolve Departure Mono for cell evidence: {error}"))?;
    let rasterizer = UiFontRasterizer::from_bytes(font.bytes)
        .map_err(|error| format!("parse Departure Mono for cell evidence: {error}"))?;
    let glyph_available = |character| rasterizer.outline(character).is_ok();
    let font_pixels = 16.0;
    let font_metrics = rasterizer.layout("M", font_pixels);
    let line_height = font_metrics.ascent - font_metrics.descent;
    let baseline_offset = ((20.0 - line_height) * 0.5 + font_metrics.ascent).clamp(0.0, 20.0);
    let layout = lower_cells_with_options(
        &snapshot,
        [10.0, 20.0],
        CellLoweringOptions {
            // Selection remains presentation evidence rather than shell state.
            selection: Some(CellSelection {
                start: [1, 1],
                end: [10, 1],
            }),
            glyph_available: Some(&glyph_available),
            baseline_offset: Some(baseline_offset),
            caret_width: 2.0,
        },
    )?;
    let bitmap = rasterize_cell_layout(
        &layout,
        &rasterizer,
        CellGridRasterOptions {
            font_pixels,
            ..CellGridRasterOptions::default()
        },
    )?;

    let artifact_root =
        PathBuf::from("target/artifacts/console-command-window/tokimu-cell-layout-v1");
    fs::create_dir_all(&artifact_root)
        .map_err(|error| format!("create cell-layout artifact directory: {error}"))?;
    let artifact = artifact_root.join("tokimu-layout.json");
    fs::write(
        &artifact,
        serde_json::to_string_pretty(&layout)
            .map_err(|error| format!("serialize cell layout: {error}"))?,
    )
    .map_err(|error| format!("write {}: {error}", artifact.display()))?;
    let image_artifact = artifact_root.join("tokimu-cell-grid.bmp");
    write_bmp(
        &image_artifact,
        Rgba8Image {
            width: bitmap.width,
            height: bitmap.height,
            pixels: &bitmap.rgba,
        },
    )?;
    let width = bitmap.width.to_string();
    let height = bitmap.height.to_string();
    let fingerprint = format!("{:016x}", bitmap.fingerprint());
    let baseline = format!("{baseline_offset:.3}");
    write_manifest(
        artifact_root.join("tokimu-cell-grid-manifest.txt"),
        &[
            ("schema", "tokimu-console-cell-grid-raster-v1"),
            ("source_stage", "provider-neutral-cell-layout"),
            ("terminal_provider", "ratatui-0.29"),
            ("font_provider", "departure-mono-native-default"),
            ("font_pixels", "16"),
            ("baseline_offset", &baseline),
            ("width", &width),
            ("height", &height),
            ("format", "rgba8-exported-as-bgra32-bmp"),
            ("pixel_fingerprint_algorithm", "fnv1a64"),
            ("pixel_fingerprint", &fingerprint),
            ("gpu_framebuffer_equivalent", "false"),
        ],
    )?;

    println!(
        "tokimu-cell-layout-evidence: {}x{} cells={} diagnostics={} cell_size={:?} image={} fingerprint={} artifact={}",
        layout.columns,
        layout.rows,
        layout.cells.len(),
        layout.diagnostics.len(),
        layout.cell_size,
        image_artifact.display(),
        fingerprint,
        artifact.display()
    );
    Ok(())
}
