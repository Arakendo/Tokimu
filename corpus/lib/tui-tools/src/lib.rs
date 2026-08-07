//! Corpus-local terminal composition candidates.
//!
//! This crate intentionally owns neither shell semantics nor terminal I/O. It
//! turns caller-owned view data into a deterministic, bounded cell surface.

mod actions;
mod console;
mod diagnostics;
mod geometry;
mod layout;
#[cfg(feature = "ratatui-oracle")]
mod oracle;
mod raster;
#[cfg(feature = "ratatui-bridge")]
mod ratatui_bridge;
mod surface;
mod viewport;
mod views;

pub use actions::{TuiAction, TuiActionOutcome, TuiFocusItem, TuiFocusPath};
pub use console::{render_embedded_console, ConsolePrompt};
pub use diagnostics::TuiDiagnostic;
pub use geometry::{TuiExtent, TuiInsets, TuiRect};
pub use layout::{split, Axis, LayoutConstraint, LayoutResult};
#[cfg(feature = "ratatui-oracle")]
pub use oracle::{
    compare_embedded_console, compare_status_dashboard, compare_status_dashboard_raster,
    ConsoleOracleFinding, ConsoleOracleReport, ExpectedDivergence, ExpectedDivergenceKind,
    RasterOracleFinding, RasterOracleReport, StatusOracleFinding, StatusOracleReport,
    TuiOracleProvider,
};
pub use raster::{
    rasterize_cells, rasterize_cells_with_options, rasterize_surface, TuiRasterCell,
    TuiRasterFrame, TuiRasterOptions, CELL_PIXEL_HEIGHT, CELL_PIXEL_WIDTH,
};
#[cfg(feature = "ratatui-bridge")]
pub use ratatui_bridge::{rasterize_ratatui_buffer, ratatui_buffer_cells};
pub use surface::{
    Cell, ProjectionArtifact, StyleRole, Surface, TextAlignment, WrapMode,
    TUI_TOOLS_ARTIFACT_SCHEMA,
};
pub use viewport::TuiViewport;
pub use views::{
    render_status_dashboard, render_transcript, write_label_value_row, StatusDashboard,
    StatusField, StatusSection, TranscriptLine,
};
