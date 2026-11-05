//! What the in-progress sweep looks at and reports (issue #38, step 11):
//! plain data, with the logic in `sweep.rs`.

/// One session folder under the in-progress folder, as the sweep sees it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SweepEntry {
    /// The folder's name, the session ID.
    pub id: String,
    /// When anything in the folder last changed, in ms since the epoch;
    /// `None` when the folder can't say (it is then never swept).
    pub last_active_ms: Option<u64>,
}

/// What a sweep did.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct SweepReport {
    /// The session IDs whose folders were deleted.
    pub removed: Vec<String>,
    /// The ones that were due but couldn't be deleted; the next sweep tries
    /// again.
    pub failed: Vec<String>,
}
