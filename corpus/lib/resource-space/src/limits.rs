/// Explicit retention limits for an in-memory resource provider.
///
/// `None` means the corresponding limit is not enforced by this provider.
/// Applications remain responsible for selecting values appropriate to their
/// trust and memory policies.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct ResourceSpaceLimits {
    pub max_entries: Option<usize>,
    pub max_total_bytes: Option<usize>,
    pub max_bytes_per_entry: Option<usize>,
}
