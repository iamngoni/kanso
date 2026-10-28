//! What the comments of a shared note hand the shell: one row per comment
//! as the desktop's Comments tab shows it. Plain data, with text offsets in
//! UTF-16 code units of the note's body, the unit Flutter counts in.

/// Where a comment sits in the note's body now.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CommentPlacement {
    /// Its words are still in the text: `[start, end)`.
    Anchored { start: u32, end: u32 },
    /// Its words were deleted: the nearest place left.
    Orphaned { at: u32 },
}

/// A half-open range of UTF-16 code units of a comment's body.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct MentionSpan {
    pub start: u32,
    pub end: u32,
}

/// One comment on a shared note.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CommentRow {
    pub id: String,
    /// The author's account ID.
    pub author: String,
    pub author_name: String,
    pub body: String,
    pub created_ms: i64,
    /// When it was last edited, if it was.
    pub edited_ms: Option<i64>,
    pub resolved: bool,
    /// Written by this account.
    pub mine: bool,
    /// Written on this phone and not taken by the server yet.
    pub pending: bool,
    /// This account may change the words: its own comment, and it may
    /// comment on the note.
    pub can_edit: bool,
    pub can_delete: bool,
    /// This account may resolve it or open it again: anyone who may
    /// comment.
    pub can_resolve: bool,
    /// The words it is on, as they were when it was written.
    pub quote: String,
    pub placement: CommentPlacement,
    /// The `@name`s in `body` that are people in the note.
    pub mentions: Vec<MentionSpan>,
}
