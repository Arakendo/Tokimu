//! Provider-neutral vector paths used by presentation geometry.
//!
//! This module defines the shared path contract and keeps fill, stroke, and
//! construction algorithms behind one provider-neutral capability boundary.

mod builder;
mod fill;
mod geometry;
mod stroke;
mod types;

pub use builder::PathBuilder;
pub use fill::{
    tessellate_convex_fill, tessellate_general_fill, tessellate_general_fill_with_rule,
    validate_convex_fill, VectorFillRule,
};
pub use stroke::{tessellate_path_strokes, tessellate_stroke};
pub use types::{VectorContour, VectorPath};

#[cfg(test)]
use fill::tessellate_simple_loop;
#[cfg(test)]
use geometry::{cross, subtract};

#[cfg(test)]
mod tests;
