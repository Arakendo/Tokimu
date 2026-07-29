use crate::parser_support::{append_text, flush_pending_text, push_event, structure_error};
use crate::{
    ExpandedName, XmlAttribute, XmlDiagnostic, XmlDiagnosticCode, XmlEvent, XmlOptions,
    XmlSourceId, XmlSpan,
};

/// Parser-neutral structural state retained while a private adapter translates
/// streaming parser events.
pub(crate) struct ParserState {
    source: XmlSourceId,
    input_len: usize,
    options: XmlOptions,
    events: Vec<XmlEvent>,
    depth: usize,
    open_elements: Vec<(ExpandedName, XmlSpan)>,
    decoded_text_bytes: usize,
    pending_text: Option<(String, XmlSpan)>,
    document_elements: usize,
}

impl ParserState {
    pub(crate) fn new(source: XmlSourceId, input_len: usize, options: XmlOptions) -> Self {
        // Keep eager allocation proportional to normal source size while
        // bounding it independently from attacker-controlled input length.
        let event_capacity = (input_len / 32)
            .max(8)
            .min(options.limits.max_nodes)
            .min(4096);
        Self {
            source,
            input_len,
            options,
            events: Vec::with_capacity(event_capacity),
            depth: 0,
            open_elements: Vec::with_capacity(options.limits.max_nesting_depth.min(32)),
            decoded_text_bytes: 0,
            pending_text: None,
            document_elements: 0,
        }
    }

    pub(crate) fn start_element(
        &mut self,
        name: ExpandedName,
        lexical_prefix: Option<String>,
        attributes: Vec<XmlAttribute>,
        span: XmlSpan,
    ) -> Result<(), XmlDiagnostic> {
        self.flush_pending_text()?;
        self.register_document_element(span)?;
        self.depth = self
            .depth
            .checked_add(1)
            .expect("XML depth cannot overflow usize");
        if self.depth > self.options.limits.max_nesting_depth {
            return Err(crate::parser_support::limit_error(
                self.source,
                span,
                XmlDiagnosticCode::NestingDepthExceeded,
                format!(
                    "XML nesting depth {} exceeds the configured {}-level limit",
                    self.depth, self.options.limits.max_nesting_depth
                ),
            ));
        }
        push_event(
            &mut self.events,
            XmlEvent::StartElement {
                name: name.clone(),
                lexical_prefix,
                attributes,
                span,
            },
            self.options.limits.max_nodes,
        )?;
        self.open_elements.push((name, span));
        Ok(())
    }

    pub(crate) fn empty_element(
        &mut self,
        name: ExpandedName,
        lexical_prefix: Option<String>,
        attributes: Vec<XmlAttribute>,
        span: XmlSpan,
    ) -> Result<(), XmlDiagnostic> {
        self.flush_pending_text()?;
        self.register_document_element(span)?;
        push_event(
            &mut self.events,
            XmlEvent::StartElement {
                name: name.clone(),
                lexical_prefix: lexical_prefix.clone(),
                attributes,
                span,
            },
            self.options.limits.max_nodes,
        )?;
        push_event(
            &mut self.events,
            XmlEvent::EndElement {
                name,
                lexical_prefix,
                span,
            },
            self.options.limits.max_nodes,
        )
    }

    pub(crate) fn end_element(
        &mut self,
        name: ExpandedName,
        lexical_prefix: Option<String>,
        span: XmlSpan,
    ) -> Result<(), XmlDiagnostic> {
        self.flush_pending_text()?;
        let Some((open_name, open_span)) = self.open_elements.pop() else {
            return Err(structure_error(
                self.source,
                span,
                None,
                format!("XML end element '{}' has no open element", name.local_name),
            ));
        };
        if open_name != name {
            return Err(structure_error(
                self.source,
                span,
                Some(open_span),
                format!(
                    "XML end element '{}' does not match open element '{}'",
                    name.local_name, open_name.local_name
                ),
            ));
        }
        push_event(
            &mut self.events,
            XmlEvent::EndElement {
                name,
                lexical_prefix,
                span,
            },
            self.options.limits.max_nodes,
        )?;
        self.depth = self.depth.saturating_sub(1);
        Ok(())
    }

    pub(crate) fn append_text(&mut self, text: String, span: XmlSpan) -> Result<(), XmlDiagnostic> {
        if self.depth == 0 && !text.trim().is_empty() {
            return Err(structure_error(
                self.source,
                span,
                None,
                "XML text outside the document element is not allowed",
            ));
        }
        append_text(
            &mut self.pending_text,
            text,
            span,
            &mut self.decoded_text_bytes,
            self.options.limits.max_decoded_text_bytes,
        )
    }

    pub(crate) fn comment(&mut self, text: String, span: XmlSpan) -> Result<(), XmlDiagnostic> {
        self.flush_pending_text()?;
        push_event(
            &mut self.events,
            XmlEvent::Comment { text, span },
            self.options.limits.max_nodes,
        )
    }

    pub(crate) fn processing_instruction(
        &mut self,
        target: String,
        data: Option<String>,
        span: XmlSpan,
    ) -> Result<(), XmlDiagnostic> {
        self.flush_pending_text()?;
        push_event(
            &mut self.events,
            XmlEvent::ProcessingInstruction { target, data, span },
            self.options.limits.max_nodes,
        )
    }

    pub(crate) fn finish(mut self) -> Result<Vec<XmlEvent>, XmlDiagnostic> {
        self.flush_pending_text()?;
        if let Some((name, open_span)) = self.open_elements.last() {
            return Err(structure_error(
                self.source,
                XmlSpan::new(self.source, self.input_len, self.input_len),
                Some(*open_span),
                format!(
                    "XML input ended before element '{}' was closed",
                    name.local_name
                ),
            ));
        }
        if self.document_elements == 0 {
            return Err(structure_error(
                self.source,
                XmlSpan::new(self.source, self.input_len, self.input_len),
                None,
                "XML source contains no document element",
            ));
        }
        Ok(self.events)
    }

    fn flush_pending_text(&mut self) -> Result<(), XmlDiagnostic> {
        flush_pending_text(
            &mut self.events,
            &mut self.pending_text,
            self.options.limits.max_nodes,
        )
    }

    fn register_document_element(&mut self, span: XmlSpan) -> Result<(), XmlDiagnostic> {
        if self.depth == 0 {
            self.document_elements = self.document_elements.saturating_add(1);
            if self.document_elements > 1 {
                return Err(structure_error(
                    self.source,
                    span,
                    None,
                    "XML source contains more than one document element",
                ));
            }
        }
        Ok(())
    }
}
