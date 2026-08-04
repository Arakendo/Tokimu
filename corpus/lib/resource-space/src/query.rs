use crate::VisibilityQuery;

/// A deliberately bounded, literal resource search.
///
/// Prefix and suffix filters apply to a resource's normalized direct name,
/// rather than to provider paths. They are literal string comparisons: this
/// contract does not define glob, regular-expression, or MIME wildcard
/// semantics. Search includes the selected folder and all of its descendants;
/// use direct folder navigation when recursion is not wanted.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResourceSearchQuery {
    visibility: VisibilityQuery,
    name_prefix: Option<String>,
    name_suffix: Option<String>,
    media_type: Option<String>,
    max_results: usize,
}

impl ResourceSearchQuery {
    /// Creates a visible-only recursive search with an explicit result cap.
    pub const fn new(max_results: usize) -> Self {
        Self {
            visibility: VisibilityQuery::VisibleOnly,
            name_prefix: None,
            name_suffix: None,
            media_type: None,
            max_results,
        }
    }

    pub const fn visibility(mut self, visibility: VisibilityQuery) -> Self {
        self.visibility = visibility;
        self
    }

    pub fn with_name_prefix(mut self, prefix: impl Into<String>) -> Self {
        self.name_prefix = Some(prefix.into());
        self
    }

    pub fn with_name_suffix(mut self, suffix: impl Into<String>) -> Self {
        self.name_suffix = Some(suffix.into());
        self
    }

    /// Requires an exact media-type metadata value. Parameter and wildcard
    /// matching remain a future consumer-driven concern.
    pub fn with_media_type(mut self, media_type: impl Into<String>) -> Self {
        self.media_type = Some(media_type.into());
        self
    }

    pub const fn visibility_query(&self) -> VisibilityQuery {
        self.visibility
    }

    pub fn name_prefix(&self) -> Option<&str> {
        self.name_prefix.as_deref()
    }

    pub fn name_suffix(&self) -> Option<&str> {
        self.name_suffix.as_deref()
    }

    pub fn media_type(&self) -> Option<&str> {
        self.media_type.as_deref()
    }

    pub const fn max_results(&self) -> usize {
        self.max_results
    }
}
