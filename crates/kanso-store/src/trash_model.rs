//! The plain numbers Empty Trash reports (issue #44). No logic: `trash.rs`
//! fills them.

/// What is in the trash, as the Empty Trash confirmation counts it.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct TrashCounts {
    /// Every note in the trash.
    pub notes: u32,
    /// How many of those are locked (sealed), and so own sealed attachments.
    pub locked: u32,
}

/// What Empty Trash deleted.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Emptied {
    /// Notes deleted for good.
    pub notes: u32,
    /// How many of those were locked.
    pub locked: u32,
    /// Attachments deleted because only trashed notes owned them.
    pub files: u32,
}
