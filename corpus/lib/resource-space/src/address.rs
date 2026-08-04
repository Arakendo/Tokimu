use std::fmt;

use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AddressCasePolicy {
    Sensitive,
    Insensitive,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct ResourceName(String);

impl ResourceName {
    pub fn parse(value: &str, policy: AddressCasePolicy) -> Result<Self, ResourceAddressError> {
        validate_segment(value, 0)?;
        Ok(Self(normalize_case(value, policy)))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for ResourceName {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.0)
    }
}

/// A relative logical address inside an explicitly selected store root.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct ResourceAddress {
    segments: Vec<ResourceName>,
}

impl ResourceAddress {
    pub fn parse(value: &str, policy: AddressCasePolicy) -> Result<Self, ResourceAddressError> {
        if value.is_empty() {
            return Err(ResourceAddressError::EmptyAddress);
        }
        if value.starts_with(['/', '\\']) {
            return Err(ResourceAddressError::AbsoluteAddress);
        }

        let mut segments = Vec::new();
        for (index, segment) in value.split(['/', '\\']).enumerate() {
            validate_segment(segment, index)?;
            segments.push(ResourceName(normalize_case(segment, policy)));
        }

        Ok(Self { segments })
    }

    pub fn from_segments(
        segments: impl IntoIterator<Item = ResourceName>,
    ) -> Result<Self, ResourceAddressError> {
        let segments = segments.into_iter().collect::<Vec<_>>();
        if segments.is_empty() {
            return Err(ResourceAddressError::EmptyAddress);
        }
        Ok(Self { segments })
    }

    pub fn segments(&self) -> &[ResourceName] {
        &self.segments
    }

    pub fn file_name(&self) -> &ResourceName {
        self.segments
            .last()
            .expect("resource addresses always contain one segment")
    }

    pub fn parent(&self) -> Option<Self> {
        (self.segments.len() > 1).then(|| Self {
            segments: self.segments[..self.segments.len() - 1].to_vec(),
        })
    }

    pub fn join(&self, child: ResourceName) -> Self {
        let mut segments = self.segments.clone();
        segments.push(child);
        Self { segments }
    }

    /// Returns whether this address begins with every normalized segment in
    /// `prefix`. This is segment-aware: `assets` does not match `assets-old`.
    pub fn has_prefix(&self, prefix: &Self) -> bool {
        self.segments.starts_with(&prefix.segments)
    }
}

impl fmt::Display for ResourceAddress {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut segments = self.segments.iter();
        if let Some(first) = segments.next() {
            write!(formatter, "{first}")?;
        }
        for segment in segments {
            write!(formatter, "/{segment}")?;
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum ResourceAddressError {
    #[error("resource address is empty")]
    EmptyAddress,
    #[error("resource address must be relative to an explicit root")]
    AbsoluteAddress,
    #[error("resource address segment {index} is empty")]
    EmptySegment { index: usize },
    #[error("resource address segment {index} is the current-directory marker")]
    CurrentSegment { index: usize },
    #[error("resource address segment {index} attempts parent traversal")]
    ParentTraversal { index: usize },
    #[error("resource address segment {index} contains a provider qualifier")]
    ProviderQualifier { index: usize },
    #[error("resource address segment {index} contains a control character")]
    ControlCharacter { index: usize },
}

fn validate_segment(value: &str, index: usize) -> Result<(), ResourceAddressError> {
    if value.is_empty() {
        return Err(ResourceAddressError::EmptySegment { index });
    }
    if value == "." {
        return Err(ResourceAddressError::CurrentSegment { index });
    }
    if value == ".." {
        return Err(ResourceAddressError::ParentTraversal { index });
    }
    if value.contains(':') {
        return Err(ResourceAddressError::ProviderQualifier { index });
    }
    if value.chars().any(char::is_control) {
        return Err(ResourceAddressError::ControlCharacter { index });
    }
    Ok(())
}

fn normalize_case(value: &str, policy: AddressCasePolicy) -> String {
    match policy {
        AddressCasePolicy::Sensitive => value.to_owned(),
        AddressCasePolicy::Insensitive => value.to_lowercase(),
    }
}
