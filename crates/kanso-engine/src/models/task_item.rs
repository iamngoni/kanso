use sqlx::FromRow;

/// A task extracted from a note body (`- [ ]` / `- [x]`).
#[derive(Debug, Clone, FromRow)]
pub struct TaskItem {
    pub id: String,
    pub note_id: String,
    pub text: String,
    pub checked: i64,
}
