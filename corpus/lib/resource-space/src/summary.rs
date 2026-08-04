/// Point-in-time provider-neutral resource-space counts.
///
/// These are current retained values, not lifetime counters and not a
/// performance profile.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ResourceSpaceSummary {
    roots: usize,
    folders: usize,
    resources: usize,
    retained_bytes: usize,
}

impl ResourceSpaceSummary {
    pub(crate) const fn new(
        roots: usize,
        folders: usize,
        resources: usize,
        retained_bytes: usize,
    ) -> Self {
        Self {
            roots,
            folders,
            resources,
            retained_bytes,
        }
    }

    pub const fn roots(self) -> usize {
        self.roots
    }

    pub const fn folders(self) -> usize {
        self.folders
    }

    pub const fn resources(self) -> usize {
        self.resources
    }

    pub const fn retained_bytes(self) -> usize {
        self.retained_bytes
    }
}
