//! What writing a note's property changed, and what it takes to put it back.

/// One property written to a note, kept so the write can be undone as one
/// step: the body and edit time the note had, and the body it has now.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PropertyEdit {
    pub note_id: String,
    /// The body before the write.
    pub before_body: String,
    /// The edit time before the write.
    pub before_updated_at: i64,
    /// The body the write made. Undo only puts the note back while this is
    /// still its body, so a later edit is never overwritten.
    pub after_body: String,
    /// Whether the write changed the body at all.
    pub changed: bool,
}
