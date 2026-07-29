//! Parser-neutral XML evidence shared by SVG corpus producers.

use xml_tools::{parse_xml_events, XmlDocument, XmlDocumentId, XmlEvent, XmlOptions, XmlSourceId};

#[derive(Clone, Copy, Debug)]
pub(crate) struct XmlStageEvidence {
    pub(crate) event_count: usize,
    pub(crate) start_elements: usize,
    pub(crate) end_elements: usize,
    pub(crate) text_nodes: usize,
    pub(crate) comments: usize,
    pub(crate) processing_instructions: usize,
    pub(crate) document_roots: usize,
    pub(crate) has_document_element: bool,
}

/// One parser-neutral XML event stream feeds both XML evidence and SVG lowering.
pub(crate) struct XmlStageInspection {
    pub(crate) evidence: XmlStageEvidence,
    pub(crate) events: Vec<XmlEvent>,
}

impl XmlStageEvidence {
    pub(crate) fn summary(self) -> String {
        format!(
            "events={} elements={}/{} text={} comments={} processing_instructions={} roots={} document_element={}",
            self.event_count,
            self.start_elements,
            self.end_elements,
            self.text_nodes,
            self.comments,
            self.processing_instructions,
            self.document_roots,
            self.has_document_element,
        )
    }
}

/// Parses and retains a document independently of SVG semantics so the corpus
/// can localize failures at the XML boundary before vector lowering begins.
pub(crate) fn inspect_xml_stage(
    source: &str,
    source_id: XmlSourceId,
) -> Result<XmlStageInspection, String> {
    let events = parse_xml_events(source_id, source, XmlOptions::default())
        .map_err(|error| format!("XML parse failed: {error}"))?;
    let document =
        XmlDocument::from_events(XmlDocumentId::new(source_id.value()), source_id, &events)
            .map_err(|error| format!("XML document construction failed: {error}"))?;

    let mut evidence = XmlStageEvidence {
        event_count: events.len(),
        start_elements: 0,
        end_elements: 0,
        text_nodes: 0,
        comments: 0,
        processing_instructions: 0,
        document_roots: document.roots().len(),
        has_document_element: document.document_element().is_some(),
    };
    for event in &events {
        match event {
            XmlEvent::StartElement { .. } => evidence.start_elements += 1,
            XmlEvent::EndElement { .. } => evidence.end_elements += 1,
            XmlEvent::Text { .. } => evidence.text_nodes += 1,
            XmlEvent::Comment { .. } => evidence.comments += 1,
            XmlEvent::ProcessingInstruction { .. } => evidence.processing_instructions += 1,
        }
    }
    Ok(XmlStageInspection { evidence, events })
}
