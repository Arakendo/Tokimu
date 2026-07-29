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
pub(crate) use fill::tessellate_font_fill_with_rule;
pub use fill::{
    tessellate_convex_fill, tessellate_general_fill, tessellate_general_fill_with_rule,
    validate_convex_fill, VectorFillRule,
};
pub use geometry::{
    clip_path_to_axis_aligned_rect, clip_path_to_convex_polygon, is_convex_polygon_clip,
};
pub use stroke::{
    tessellate_path_strokes, tessellate_stroke, tessellate_stroke_with_style, VectorStrokeCap,
    VectorStrokeJoin, VectorStrokeStyle,
};
pub use types::{VectorContour, VectorPath};

#[cfg(test)]
use fill::tessellate_simple_loop;
#[cfg(test)]
use geometry::{cross, subtract};

#[cfg(test)]
mod tests;
