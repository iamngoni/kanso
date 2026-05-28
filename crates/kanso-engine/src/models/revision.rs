use sqlx::FromRow;

/// A point-in-time snapshot of a note body. Sources: `user` (pre-edit),
/// `sync` (superseded by remote), `conflict` (a losing remote version preserved
/// rather than discarded), `import`, `agent`.
#[derive(Debug, Clone, FromRow)]
pub struct Revision {
    pub id: String,
    pub note_id: String,
    pub body_markdown: String,
    pub reason: Option<String>,
    pub source: String,
    pub created_at: i64,
}
