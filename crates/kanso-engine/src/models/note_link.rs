use sqlx::FromRow;

/// An outgoing reference from a note: `[[note]]`, `![[sketch:id]]`,
/// `![[attachment:name]]`.
#[derive(Debug, Clone, FromRow)]
pub struct NoteLink {
    pub source_note_id: String,
    pub target_ref: String,
    /// `note` | `sketch` | `attachment`.
    pub link_kind: String,
}
