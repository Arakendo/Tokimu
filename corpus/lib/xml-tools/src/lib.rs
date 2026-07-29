//! Parser-neutral, bounded XML ingestion contracts for incubating importers.
//!
//! This crate deliberately exposes no parser implementation types. Parser
//! adapters will translate their private errors and events into these stable
//! source, limit, and diagnostic contracts.

mod contracts;
mod diagnostic;
mod document;
mod model;
mod parser;
mod parser_adapters;
mod parser_names;
mod parser_state;
mod parser_support;

pub use contracts::{validate_xml_input, XmlLimits, XmlOptions, XmlSourceId, XmlSpan};
pub use diagnostic::{
    XmlDiagnostic, XmlDiagnosticCategory, XmlDiagnosticCode, XmlDiagnosticSeverity,
};
pub use document::{XmlDocument, XmlDocumentId, XmlNodeId, XmlNodeKind};
pub use model::{ExpandedName, XmlAttribute, XmlEvent};
pub use parser::parse_xml_events;
pub use parser_adapters::{parse_xml_bytes, parse_xml_document, parse_xml_document_bytes};

#[cfg(test)]
mod tests;
