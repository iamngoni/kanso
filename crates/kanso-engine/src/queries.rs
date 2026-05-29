//! Read-only graph/derived queries over data the engine already indexes
//! (links and tasks).

use crate::db::Engine;
use crate::error::Result;
use crate::models::{Note, NoteLink, TaskItem};

const NOTE_COLUMNS_N: &str =
    "n.id, n.notebook_id, n.title, n.body_markdown, n.created_at, n.updated_at, n.pinned, n.favorite, n.status";

impl Engine {
    /// Notes that link to `note_id` by its title (`[[Title]]`).
    pub async fn backlinks(&self, note_id: &str) -> Result<Vec<Note>> {
        let sql = format!(
            "SELECT {NOTE_COLUMNS_N} FROM note_links l \
             JOIN notes target ON target.id = ? \
             JOIN notes n ON n.id = l.source_note_id \
             WHERE l.link_kind = 'note' AND l.target_ref = target.title \
             AND n.deleted_at IS NULL"
        );
        Ok(sqlx::query_as::<_, Note>(&sql).bind(note_id).fetch_all(&self.pool).await?)
    }

    /// All outgoing references from a note.
    pub async fn outgoing_links(&self, note_id: &str) -> Result<Vec<NoteLink>> {
        Ok(sqlx::query_as::<_, NoteLink>(
            "SELECT source_note_id, target_ref, link_kind FROM note_links \
             WHERE source_note_id = ?",
        )
        .bind(note_id)
        .fetch_all(&self.pool)
        .await?)
    }

    /// Soft-deleted notes (the trash), most-recently-deleted first.
    pub async fn list_trash(&self) -> Result<Vec<Note>> {
        let sql = format!(
            "SELECT {NOTE_COLUMNS_N} FROM notes n \
             WHERE n.deleted_at IS NOT NULL ORDER BY n.deleted_at DESC"
        );
        Ok(sqlx::query_as::<_, Note>(&sql).fetch_all(&self.pool).await?)
    }

    /// All pinned (non-deleted) notes across every notebook, most-recent first.
    pub async fn list_pinned(&self) -> Result<Vec<Note>> {
        let sql = format!(
            "SELECT {NOTE_COLUMNS_N} FROM notes n \
             WHERE n.pinned = 1 AND n.deleted_at IS NULL ORDER BY n.updated_at DESC"
        );
        Ok(sqlx::query_as::<_, Note>(&sql).fetch_all(&self.pool).await?)
    }

    /// All (non-deleted) notes carrying a given tag.
    pub async fn notes_with_tag(&self, tag_id: &str) -> Result<Vec<Note>> {
        let sql = format!(
            "SELECT {NOTE_COLUMNS_N} FROM note_tags nt \
             JOIN notes n ON n.id = nt.note_id \
             WHERE nt.tag_id = ? AND n.deleted_at IS NULL \
             ORDER BY n.updated_at DESC"
        );
        Ok(sqlx::query_as::<_, Note>(&sql).bind(tag_id).fetch_all(&self.pool).await?)
    }

    /// All tasks across a notebook's (non-deleted) notes.
    pub async fn list_tasks(&self, notebook_id: &str) -> Result<Vec<TaskItem>> {
        Ok(sqlx::query_as::<_, TaskItem>(
            "SELECT t.id, t.note_id, t.text, t.checked FROM note_tasks t \
             JOIN notes n ON n.id = t.note_id \
             WHERE n.notebook_id = ? AND n.deleted_at IS NULL \
             ORDER BY t.note_id",
        )
        .bind(notebook_id)
        .fetch_all(&self.pool)
        .await?)
    }

    /// Unchecked tasks across a notebook.
    pub async fn list_open_tasks(&self, notebook_id: &str) -> Result<Vec<TaskItem>> {
        Ok(sqlx::query_as::<_, TaskItem>(
            "SELECT t.id, t.note_id, t.text, t.checked FROM note_tasks t \
             JOIN notes n ON n.id = t.note_id \
             WHERE n.notebook_id = ? AND n.deleted_at IS NULL AND t.checked = 0 \
             ORDER BY t.note_id",
        )
        .bind(notebook_id)
        .fetch_all(&self.pool)
        .await?)
    }
}
