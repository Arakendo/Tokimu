use crate::{
    ExpandedName, XmlAttribute, XmlDiagnostic, XmlDiagnosticCategory, XmlDiagnosticCode, XmlEvent,
    XmlSourceId, XmlSpan,
};

/// Opaque identity supplied by the consumer that retains an XML document.
///
/// Node handles carry this value so traversal cannot accidentally use a handle
/// produced by another document.
#[derive(Clone, Copy, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct XmlDocumentId(u32);

impl XmlDocumentId {
    pub const fn new(value: u32) -> Self {
        Self(value)
    }

    pub const fn value(self) -> u32 {
        self.0
    }
}

/// A document-local immutable node handle.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct XmlNodeId {
    document: XmlDocumentId,
    index: usize,
}

/// The parser-neutral content stored by an immutable XML document node.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum XmlNodeKind {
    Element {
        name: ExpandedName,
        lexical_prefix: Option<String>,
        attributes: Vec<XmlAttribute>,
    },
    Text {
        text: String,
    },
    Comment {
        text: String,
    },
    ProcessingInstruction {
        target: String,
        data: Option<String>,
    },
}

#[derive(Clone, Debug)]
struct XmlNodeRecord {
    kind: XmlNodeKind,
    parent: Option<XmlNodeId>,
    children: Vec<XmlNodeId>,
    span: XmlSpan,
}

/// An immutable XML document built from bounded parser-neutral events.
///
/// This is intentionally not a mutable browser DOM. It retains source order,
/// parent/child relationships, attributes, expanded names, and source spans.
#[derive(Clone, Debug)]
pub struct XmlDocument {
    id: XmlDocumentId,
    source: XmlSourceId,
    nodes: Vec<XmlNodeRecord>,
    roots: Vec<XmlNodeId>,
}

impl XmlDocument {
    /// Builds a document from events produced by this crate's bounded parser.
    pub fn from_events(
        id: XmlDocumentId,
        source: XmlSourceId,
        events: &[XmlEvent],
    ) -> Result<Self, XmlDiagnostic> {
        let mut document = Self {
            id,
            source,
            nodes: Vec::with_capacity(events.len()),
            roots: Vec::new(),
        };
        let mut open_elements = Vec::with_capacity(16);

        for event in events {
            let span = event_span(event);
            if span.source != source {
                return Err(document_error(
                    source,
                    span,
                    "XML event source does not match the document source",
                ));
            }

            match event {
                XmlEvent::StartElement {
                    name,
                    lexical_prefix,
                    attributes,
                    ..
                } => {
                    let node = document.push_node(
                        XmlNodeKind::Element {
                            name: name.clone(),
                            lexical_prefix: lexical_prefix.clone(),
                            attributes: attributes.clone(),
                        },
                        span,
                        open_elements.last().copied(),
                    );
                    open_elements.push(node);
                }
                XmlEvent::EndElement { name, .. } => {
                    let Some(node) = open_elements.pop() else {
                        return Err(document_error(
                            source,
                            span,
                            format!("XML end element '{}' has no open element", name.local_name),
                        ));
                    };
                    let XmlNodeKind::Element {
                        name: open_name, ..
                    } = &document.nodes[node.index].kind
                    else {
                        unreachable!("open-element stack only contains element nodes");
                    };
                    if open_name != name {
                        let mut error = document_error(
                            source,
                            span,
                            format!(
                                "XML end element '{}' does not match open element '{}'",
                                name.local_name, open_name.local_name
                            ),
                        );
                        error.related_span = Some(document.nodes[node.index].span);
                        return Err(error);
                    }
                    document.nodes[node.index].span.end = span.end;
                }
                XmlEvent::Text { text, .. } => {
                    document.push_node(
                        XmlNodeKind::Text { text: text.clone() },
                        span,
                        open_elements.last().copied(),
                    );
                }
                XmlEvent::Comment { text, .. } => {
                    document.push_node(
                        XmlNodeKind::Comment { text: text.clone() },
                        span,
                        open_elements.last().copied(),
                    );
                }
                XmlEvent::ProcessingInstruction { target, data, .. } => {
                    document.push_node(
                        XmlNodeKind::ProcessingInstruction {
                            target: target.clone(),
                            data: data.clone(),
                        },
                        span,
                        open_elements.last().copied(),
                    );
                }
            }
        }

        if let Some(node) = open_elements.last().copied() {
            let XmlNodeKind::Element { name, .. } = &document.nodes[node.index].kind else {
                unreachable!("open-element stack only contains element nodes");
            };
            let mut error = document_error(
                source,
                document.nodes[node.index].span,
                format!("XML element '{}' was not closed", name.local_name),
            );
            error.related_span = Some(document.nodes[node.index].span);
            return Err(error);
        }

        Ok(document)
    }

    pub const fn id(&self) -> XmlDocumentId {
        self.id
    }

    pub const fn source(&self) -> XmlSourceId {
        self.source
    }

    pub fn roots(&self) -> &[XmlNodeId] {
        &self.roots
    }

    pub fn document_element(&self) -> Option<XmlNodeId> {
        self.roots
            .iter()
            .copied()
            .find(|node| matches!(self.node_kind(*node), Some(XmlNodeKind::Element { .. })))
    }

    pub fn node_kind(&self, node: XmlNodeId) -> Option<&XmlNodeKind> {
        self.node(node).map(|record| &record.kind)
    }

    pub fn node_span(&self, node: XmlNodeId) -> Option<XmlSpan> {
        self.node(node).map(|record| record.span)
    }

    pub fn parent(&self, node: XmlNodeId) -> Option<XmlNodeId> {
        self.node(node).and_then(|record| record.parent)
    }

    pub fn children(&self, node: XmlNodeId) -> Option<&[XmlNodeId]> {
        self.node(node).map(|record| record.children.as_slice())
    }

    fn push_node(
        &mut self,
        kind: XmlNodeKind,
        span: XmlSpan,
        parent: Option<XmlNodeId>,
    ) -> XmlNodeId {
        let node = XmlNodeId {
            document: self.id,
            index: self.nodes.len(),
        };
        self.nodes.push(XmlNodeRecord {
            kind,
            parent,
            children: Vec::new(),
            span,
        });
        if let Some(parent) = parent {
            self.nodes[parent.index].children.push(node);
        } else {
            self.roots.push(node);
        }
        node
    }

    fn node(&self, node: XmlNodeId) -> Option<&XmlNodeRecord> {
        (node.document == self.id)
            .then(|| self.nodes.get(node.index))
            .flatten()
    }
}

fn event_span(event: &XmlEvent) -> XmlSpan {
    match event {
        XmlEvent::StartElement { span, .. }
        | XmlEvent::EndElement { span, .. }
        | XmlEvent::Text { span, .. }
        | XmlEvent::Comment { span, .. }
        | XmlEvent::ProcessingInstruction { span, .. } => *span,
    }
}

fn document_error(source: XmlSourceId, span: XmlSpan, message: impl Into<String>) -> XmlDiagnostic {
    XmlDiagnostic::at(
        XmlDiagnosticCategory::WellFormedness,
        XmlDiagnosticCode::DocumentStructure,
        source,
        span,
        message,
    )
}
