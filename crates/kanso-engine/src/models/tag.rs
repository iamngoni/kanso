use sqlx::FromRow;

/// A global label that can attach to notes.
#[derive(Debug, Clone, FromRow)]
pub struct Tag {
    pub id: String,
    pub name: String,
    pub color: Option<String>,
}
