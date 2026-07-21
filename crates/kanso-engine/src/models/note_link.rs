/// An outgoing reference from a note: `[[note]]`, `![[sketch:id]]`,
/// `![[attachment:name]]`.
#[derive(Debug, Clone)]
pub struct NoteLink {
    pub source_note_id: String,
    pub target_ref: String,
    /// `note` | `sketch` | `attachment`.
    pub link_kind: String,
}

impl_sqlite_from_row!(NoteLink {
    source_note_id,
    target_ref,
    link_kind,
});
