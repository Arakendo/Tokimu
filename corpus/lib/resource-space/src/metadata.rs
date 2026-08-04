use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum ResourceVisibility {
    #[default]
    Visible,
    Hidden,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum VisibilityQuery {
    VisibleOnly,
    HiddenOnly,
    All,
}

impl VisibilityQuery {
    pub const fn includes(self, visibility: ResourceVisibility) -> bool {
        matches!(
            (self, visibility),
            (Self::VisibleOnly, ResourceVisibility::Visible)
                | (Self::HiddenOnly, ResourceVisibility::Hidden)
                | (Self::All, _)
        )
    }
}

/// Milliseconds since a caller-selected epoch. The base contract deliberately
/// avoids filesystem-specific timestamp types.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct ResourceTimestamp(u64);

impl ResourceTimestamp {
    pub const fn from_millis(value: u64) -> Self {
        Self(value)
    }

    pub const fn as_millis(self) -> u64 {
        self.0
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct ResourceMetadata {
    pub visibility: ResourceVisibility,
    pub media_type: Option<String>,
    pub created_at: Option<ResourceTimestamp>,
    pub modified_at: Option<ResourceTimestamp>,
}
