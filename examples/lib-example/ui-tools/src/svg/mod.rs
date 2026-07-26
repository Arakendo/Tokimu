//! SVG document lowering into provider-neutral presentation geometry.

mod document;
mod path;
mod primitives;
mod semantic;
mod transform;
mod types;

pub use document::{
    parse_svg_document_convex_fill_meshes, parse_svg_document_vector_paths,
    parse_svg_document_vector_records, parse_svg_document_vector_records_from_xml_events,
    parse_svg_document_vector_records_with_viewport,
    parse_svg_document_vector_records_with_xml_options,
};
pub use path::{parse_path, tokenize_path, SvgPathCommand, SvgToken};
pub use types::{
    SvgFillRule, SvgImportDiagnostic, SvgImportStage, SvgVectorRecord, SvgViewportSource,
};

#[cfg(test)]
use path::flatten_path;
#[cfg(test)]
use primitives::stroke_paths;

#[cfg(test)]
mod tests;
