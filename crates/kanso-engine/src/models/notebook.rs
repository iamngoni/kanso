use sqlx::FromRow;

/// A user-facing container for notes.
#[derive(Debug, Clone, FromRow)]
pub struct Notebook {
    pub id: String,
    pub name: String,
    pub parent_id: Option<String>,
    pub sort_order: i64,
    pub created_at: i64,
    pub updated_at: i64,
    pub deleted_at: Option<i64>,
}
