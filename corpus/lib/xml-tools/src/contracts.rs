use crate::XmlDiagnostic;

/// Opaque identity for a source supplied to an XML parser adapter.
#[derive(Clone, Copy, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct XmlSourceId(u32);

impl XmlSourceId {
    pub const fn new(value: u32) -> Self {
        Self(value)
    }

    pub const fn value(self) -> u32 {
        self.0
    }
}

/// A half-open byte span into the original UTF-8 source.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct XmlSpan {
    pub source: XmlSourceId,
    pub start: usize,
    pub end: usize,
}

impl XmlSpan {
    pub const fn new(source: XmlSourceId, start: usize, end: usize) -> Self {
        Self { source, start, end }
    }

    pub const fn is_valid(self) -> bool {
        self.start <= self.end
    }
}

/// Resource bounds applied before and during XML ingestion.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct XmlLimits {
    pub max_input_bytes: usize,
    pub max_nesting_depth: usize,
    pub max_nodes: usize,
    pub max_attributes_per_element: usize,
    pub max_name_bytes: usize,
    pub max_attribute_value_bytes: usize,
    pub max_decoded_text_bytes: usize,
    pub max_diagnostics: usize,
}

impl XmlLimits {
    /// Conservative defaults for untrusted UTF-8 XML supplied to an importer.
    pub const fn safe_defaults() -> Self {
        Self {
            max_input_bytes: 16 * 1024 * 1024,
            max_nesting_depth: 128,
            max_nodes: 100_000,
            max_attributes_per_element: 256,
            max_name_bytes: 1024,
            max_attribute_value_bytes: 64 * 1024,
            max_decoded_text_bytes: 4 * 1024 * 1024,
            max_diagnostics: 128,
        }
    }

    /// Rejects options that disable a required bound.
    pub fn validate(self) -> Result<(), XmlDiagnostic> {
        for (name, value) in [
            ("max_input_bytes", self.max_input_bytes),
            ("max_nesting_depth", self.max_nesting_depth),
            ("max_nodes", self.max_nodes),
            (
                "max_attributes_per_element",
                self.max_attributes_per_element,
            ),
            ("max_name_bytes", self.max_name_bytes),
            ("max_attribute_value_bytes", self.max_attribute_value_bytes),
            ("max_decoded_text_bytes", self.max_decoded_text_bytes),
            ("max_diagnostics", self.max_diagnostics),
        ] {
            if value == 0 {
                return Err(XmlDiagnostic::invalid_options(format!(
                    "XML limit '{name}' must be greater than zero"
                )));
            }
        }
        Ok(())
    }
}

impl Default for XmlLimits {
    fn default() -> Self {
        Self::safe_defaults()
    }
}

/// Options owned by the parser-neutral XML boundary.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct XmlOptions {
    pub limits: XmlLimits,
}

impl XmlOptions {
    pub fn validate(self) -> Result<(), XmlDiagnostic> {
        self.limits.validate()
    }
}

/// Validates options and source-buffer size before parser-specific ingestion.
pub fn validate_xml_input(
    source: XmlSourceId,
    input: &[u8],
    options: XmlOptions,
) -> Result<(), XmlDiagnostic> {
    options.validate()?;
    if input.len() > options.limits.max_input_bytes {
        return Err(XmlDiagnostic::input_too_large(
            source,
            input.len(),
            options.limits.max_input_bytes,
        ));
    }
    Ok(())
}
