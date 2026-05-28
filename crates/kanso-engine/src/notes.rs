//! Note commands — the core write path.
//!
//! Every mutation runs in one transaction that keeps the note row, the FTS
//! index, derived tasks/links, revisions, and the sync outbox consistent. The
//! native apps never see this sequencing; they call one command.

use kanso_types::NoteId;
use kanso_types::sync::{EntityType, Operation};
use sqlx::{FromRow, SqliteConnection};

use crate::db::{Engine, enqueue_outbox, now_ms};
use crate::error::{EngineError, Result};
use crate::markdown::{self, RefKind};

#[derive(Debug, Clone, FromRow)]
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

const NOTE_COLUMNS: &str =
    "id, notebook_id, title, body_markdown, created_at, updated_at, pinned, favorite, status";

impl Engine {
    pub async fn create_note(&self, notebook_id: &str, title: &str, body: &str) -> Result<Note> {
        let id = NoteId::new().0;
        let now = now_ms();

        let mut tx = self.pool.begin().await?;

        sqlx::query(
            "INSERT INTO notes (id, notebook_id, title, body_markdown, created_at, updated_at) \
             VALUES (?, ?, ?, ?, ?, ?)",
        )
        .bind(&id)
        .bind(notebook_id)
        .bind(title)
        .bind(body)
        .bind(now)
        .bind(now)
        .execute(&mut *tx)
        .await?;

        reindex_note(&mut tx, &id, title, body).await?;
        enqueue_outbox(
            &mut tx,
            EntityType::Note,
            &id,
            Operation::NoteCreated,
            serde_json::json!({ "notebook_id": notebook_id, "title": title, "body_markdown": body }),
            now,
        )
        .await?;

        tx.commit().await?;

        Ok(Note {
            id,
            notebook_id: notebook_id.to_string(),
            title: title.to_string(),
            body_markdown: body.to_string(),
            created_at: now,
            updated_at: now,
            pinned: 0,
            favorite: 0,
            status: "active".to_string(),
        })
    }

    /// Update a note body, snapshotting the previous version into a revision and
    /// re-deriving the index — all atomically.
    pub async fn update_note_body(&self, note_id: &str, body: &str) -> Result<()> {
        let now = now_ms();
        let mut tx = self.pool.begin().await?;

        let current: Option<(String, String)> = sqlx::query_as(
            "SELECT title, body_markdown FROM notes WHERE id = ? AND deleted_at IS NULL",
        )
        .bind(note_id)
        .fetch_optional(&mut *tx)
        .await?;
        let Some((title, old_body)) = current else {
            return Err(EngineError::NotFound(note_id.to_string()));
        };

        // Snapshot the prior body before overwriting (source = user).
        let revision_id = kanso_types::RevisionId::new().0;
        sqlx::query(
            "INSERT INTO revisions (id, note_id, body_markdown, reason, source, created_at) \
             VALUES (?, ?, ?, ?, 'user', ?)",
        )
        .bind(&revision_id)
        .bind(note_id)
        .bind(&old_body)
        .bind("pre-edit snapshot")
        .bind(now)
        .execute(&mut *tx)
        .await?;

        sqlx::query(
            "UPDATE notes SET body_markdown = ?, updated_at = ?, current_revision_id = ? WHERE id = ?",
        )
        .bind(body)
        .bind(now)
        .bind(&revision_id)
        .bind(note_id)
        .execute(&mut *tx)
        .await?;

        reindex_note(&mut tx, note_id, &title, body).await?;
        enqueue_outbox(
            &mut tx,
            EntityType::Note,
            note_id,
            Operation::NoteUpdated,
            serde_json::json!({ "body_markdown": body }),
            now,
        )
        .await?;

        tx.commit().await?;
        Ok(())
    }

    pub async fn list_notes(&self, notebook_id: &str) -> Result<Vec<Note>> {
        let query = format!(
            "SELECT {NOTE_COLUMNS} FROM notes \
             WHERE notebook_id = ? AND deleted_at IS NULL ORDER BY updated_at DESC"
        );
        let notes = sqlx::query_as::<_, Note>(&query)
            .bind(notebook_id)
            .fetch_all(&self.pool)
            .await?;
        Ok(notes)
    }

    /// Full-text search across note titles and bodies, ranked by relevance.
    pub async fn search_notes(&self, query: &str) -> Result<Vec<Note>> {
        let sql = format!(
            "SELECT {} FROM notes_fts f \
             JOIN notes n ON n.id = f.note_id \
             WHERE notes_fts MATCH ? AND n.deleted_at IS NULL \
             ORDER BY rank",
            NOTE_COLUMNS
                .split(", ")
                .map(|c| format!("n.{c}"))
                .collect::<Vec<_>>()
                .join(", ")
        );
        let notes = sqlx::query_as::<_, Note>(&sql)
            .bind(query)
            .fetch_all(&self.pool)
            .await?;
        Ok(notes)
    }
}

/// Rebuild the FTS row and the derived tasks/links for a note. Runs inside the
/// caller's transaction so the index can never drift from the note.
async fn reindex_note(
    conn: &mut SqliteConnection,
    note_id: &str,
    title: &str,
    body: &str,
) -> Result<()> {
    sqlx::query("DELETE FROM notes_fts WHERE note_id = ?")
        .bind(note_id)
        .execute(&mut *conn)
        .await?;
    sqlx::query("INSERT INTO notes_fts (note_id, title, body) VALUES (?, ?, ?)")
        .bind(note_id)
        .bind(title)
        .bind(body)
        .execute(&mut *conn)
        .await?;

    let extracted = markdown::extract(body);

    sqlx::query("DELETE FROM note_tasks WHERE note_id = ?")
        .bind(note_id)
        .execute(&mut *conn)
        .await?;
    for task in &extracted.tasks {
        sqlx::query("INSERT INTO note_tasks (id, note_id, text, checked) VALUES (?, ?, ?, ?)")
            .bind(format!("task:{}", uuid::Uuid::now_v7()))
            .bind(note_id)
            .bind(&task.text)
            .bind(task.checked as i64)
            .execute(&mut *conn)
            .await?;
    }

    sqlx::query("DELETE FROM note_links WHERE source_note_id = ?")
        .bind(note_id)
        .execute(&mut *conn)
        .await?;
    for reference in &extracted.refs {
        let kind = match reference.kind {
            RefKind::Note => "note",
            RefKind::Sketch => "sketch",
            RefKind::Attachment => "attachment",
        };
        sqlx::query(
            "INSERT OR IGNORE INTO note_links (source_note_id, target_ref, link_kind) \
             VALUES (?, ?, ?)",
        )
        .bind(note_id)
        .bind(&reference.target)
        .bind(kind)
        .execute(&mut *conn)
        .await?;
    }

    Ok(())
}
