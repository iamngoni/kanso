/// Result of applying one inbound remote change.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ApplyOutcome {
    /// The change was applied to local state.
    Applied,
    /// Local state was newer; the remote version was preserved as a conflict
    /// copy/revision and local was kept as primary. No text was discarded.
    Conflicted,
    /// The entity was soft-deleted (or already tombstoned).
    Deleted,
    /// Nothing to do — already applied, or superseded by a later local change.
    Skipped,
}
