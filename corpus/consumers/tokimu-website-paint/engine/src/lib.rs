//! Headless semantic engine for the Tokimu Paint consumer corpus.
//!
//! This crate is intentionally application-owned evidence. It does not admit
//! editable raster documents into Tokimu's public engine contracts.

mod command;
mod document;
mod export;
mod history;
mod replay;
mod session;
mod wasm;

pub use command::{
    apply_command, sample_color, CanvasPoint, CommandError, EditObservation, PaintCommand,
    PixelBounds, PixelPoint, MAX_STROKE_DIAMETER,
};
pub use document::{
    DocumentConfig, DocumentError, DocumentObservation, EditableRasterDocument, Rgba8,
};
pub use export::{export_png, ExportConfig, ExportError, ExportObservation, LosslessExport};
pub use history::{HistoryConfig, HistoryError, HistoryObservation, PaintWorkspace};
pub use replay::{replay_commands, BlankReplayDocument, PaintReplay, PaintReplayObservation};
pub use session::{
    DocumentPreview, PaintSession, PaintSessionConfig, SessionError, SessionObservation,
};
pub use wasm::WasmPaintSession;
