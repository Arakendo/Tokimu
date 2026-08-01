//! Application-owned resource workbench semantics shared by native and web hosts.
//!
//! The model is deliberately independent of window, renderer, DOM, and WASM
//! mechanisms. Hosts translate interaction into these semantic operations and
//! present the resulting observations.

pub mod model;
pub mod ui;
