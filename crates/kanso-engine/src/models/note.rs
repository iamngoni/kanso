/// A Markdown document with metadata. The body stays plain and durable; derived
/// data (FTS, tasks, links) is maintained separately by the engine.
#[derive(Debug, Clone)]
pub struct Note {
    pub id: String,
    pub notebook_id: String,
    pub title: String,
    pub body_markdown: String,
    pub created_at: i64,
    pub updated_at: i64,
    pub pinned: i64,
    pub favorite: i64,
    pub status: String,
}

impl_sqlite_from_row!(Note {
    id,
    notebook_id,
    title,
    body_markdown,
    created_at,
    updated_at,
    pinned,
    favorite,
    status,
});
