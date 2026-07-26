use crate::XmlSpan;

/// A namespace-resolved XML name. The lexical prefix remains presentation data;
/// XML consumers should make semantic decisions from this expanded identity.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct ExpandedName {
    pub namespace_uri: Option<String>,
    pub local_name: String,
}

/// An owned XML attribute in source order.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct XmlAttribute {
    pub name: ExpandedName,
    pub lexical_prefix: Option<String>,
    pub value: String,
    pub span: XmlSpan,
}

/// Parser-neutral XML events for the first bounded importer profile.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum XmlEvent {
    StartElement {
        name: ExpandedName,
        lexical_prefix: Option<String>,
        attributes: Vec<XmlAttribute>,
        span: XmlSpan,
    },
    EndElement {
        name: ExpandedName,
        lexical_prefix: Option<String>,
        span: XmlSpan,
    },
    Text {
        text: String,
        span: XmlSpan,
    },
    Comment {
        text: String,
        span: XmlSpan,
    },
    ProcessingInstruction {
        target: String,
        data: Option<String>,
        span: XmlSpan,
    },
}
