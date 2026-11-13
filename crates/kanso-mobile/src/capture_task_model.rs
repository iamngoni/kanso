//! What a task typed into the phone's capture sheet carries to the bridge.

/// Where a captured task is written: the setting "Captured tasks go to"
/// on the desktop, and the same choice in the phone's Settings.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum CaptureTarget {
    /// The end of today's daily note, made when there is none.
    #[default]
    DailyNote,
    /// The end of the Inbox's task note.
    Inbox,
}

/// A task that was saved.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TaskCaptured {
    /// The note the task went into.
    pub note_id: String,
    /// The line written, as it is in the note:
    /// `- [ ] film Pixel review @2026-11-14T15:00 every week`.
    pub line: String,
}
