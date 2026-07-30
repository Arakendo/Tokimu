use std::fmt;

use serde::{Deserialize, Serialize};

use crate::PresentationControlError;

const MAX_TARGET_KEY_BYTES: usize = 256;

/// Semantic kind of an addressable presentation unit.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum PresentationTargetKind {
    VectorRecord,
    MeshPrimitive,
    ModelNode,
    UiRegion,
    TextRun,
    Renderable,
}

impl fmt::Display for PresentationTargetKind {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let value = match self {
            Self::VectorRecord => "vector-record",
            Self::MeshPrimitive => "mesh-primitive",
            Self::ModelNode => "model-node",
            Self::UiRegion => "ui-region",
            Self::TextRun => "text-run",
            Self::Renderable => "renderable",
        };
        formatter.write_str(value)
    }
}

/// Stable, provider-neutral identity for an addressable presentation target.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct PresentationTargetId {
    kind: PresentationTargetKind,
    key: String,
}

impl PresentationTargetId {
    pub fn new(
        kind: PresentationTargetKind,
        key: impl Into<String>,
    ) -> Result<Self, PresentationControlError> {
        let key = key.into();
        validate_target_text(&key)?;
        Ok(Self { kind, key })
    }

    pub fn kind(&self) -> PresentationTargetKind {
        self.kind
    }

    pub fn key(&self) -> &str {
        &self.key
    }
}

impl fmt::Display for PresentationTargetId {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{}:{}", self.kind, self.key)
    }
}

/// Provider-neutral target identity plus optional source-facing context.
///
/// `id` remains the only selection key. Source names are preserved solely for
/// inspection and diagnostics because importers may omit or duplicate them.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct PresentationTargetDescriptor {
    id: PresentationTargetId,
    source_name: Option<String>,
}

impl PresentationTargetDescriptor {
    pub fn new(id: PresentationTargetId) -> Self {
        Self {
            id,
            source_name: None,
        }
    }

    pub fn with_source_name(
        mut self,
        source_name: impl Into<String>,
    ) -> Result<Self, PresentationControlError> {
        let source_name = source_name.into();
        validate_target_text(&source_name)?;
        self.source_name = Some(source_name);
        Ok(self)
    }

    pub fn id(&self) -> &PresentationTargetId {
        &self.id
    }

    pub fn source_name(&self) -> Option<&str> {
        self.source_name.as_deref()
    }

    pub fn display_name(&self) -> &str {
        self.source_name().unwrap_or_else(|| self.id.key())
    }
}

fn validate_target_text(value: &str) -> Result<(), PresentationControlError> {
    if value.is_empty() {
        return Err(PresentationControlError::EmptyTargetKey);
    }
    if value.trim() != value {
        return Err(PresentationControlError::TargetKeyWhitespace);
    }
    if value.len() > MAX_TARGET_KEY_BYTES {
        return Err(PresentationControlError::TargetKeyTooLong {
            maximum: MAX_TARGET_KEY_BYTES,
        });
    }
    if value.chars().any(char::is_control) {
        return Err(PresentationControlError::TargetKeyControlCharacter);
    }
    Ok(())
}
