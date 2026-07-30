use thiserror::Error;

use crate::PresentationTargetId;

/// Deterministic validation and target-resolution failures.
#[derive(Clone, Debug, Error, PartialEq, Eq)]
pub enum PresentationControlError {
    #[error("presentation target key must not be empty")]
    EmptyTargetKey,
    #[error("presentation target key must not contain leading or trailing whitespace")]
    TargetKeyWhitespace,
    #[error("presentation target key exceeds the maximum length of {maximum} bytes")]
    TargetKeyTooLong { maximum: usize },
    #[error("presentation target key contains a control character")]
    TargetKeyControlCharacter,
    #[error("presentation value `{field}` must be finite and within 0.0..=1.0")]
    InvalidUnitValue { field: &'static str },
    #[error("presentation target `{target}` is already registered")]
    DuplicateTarget { target: PresentationTargetId },
    #[error("presentation target `{target}` is not registered")]
    UnknownTarget { target: PresentationTargetId },
}
