//! Corpus-local command-session evidence helpers.
//!
//! This crate intentionally owns only adaptation to Tosumu's public CLI JSON
//! boundary. It neither links Tosumu nor reimplements TQL.

#[cfg(feature = "ratatui-evidence")]
pub mod cell_grid_raster;
pub mod native_interaction;
#[cfg(feature = "ratatui-evidence")]
pub mod projection_conformance;
#[cfg(feature = "ratatui-evidence")]
pub mod ratatui_projection;
#[cfg(feature = "ratatui-evidence")]
pub mod tokimu_cell_projection;
pub mod tosumu_session;
