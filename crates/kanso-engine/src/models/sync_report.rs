/// Summary of one [`crate::Engine::sync`] cycle.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct SyncReport {
    /// Local outbox events the backend accepted.
    pub pushed: usize,
    /// Remote changes applied to local state.
    pub applied: usize,
    /// Remote changes that lost to a newer local version (preserved as conflict
    /// revisions, never discarded).
    pub conflicted: usize,
    /// Remote deletes applied locally.
    pub deleted: usize,
    /// Remote changes skipped (already applied or superseded).
    pub skipped: usize,
}
