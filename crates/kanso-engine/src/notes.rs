//! Note commands — the core write path.
//!
//! Every mutation runs in one transaction that keeps the note row, the FTS
//! index, derived tasks/links, revisions, and the sync outbox consistent. The
//! native apps never see this sequencing; they call one command.

use std::collections::HashMap;
use std::fs;

use base64::Engine as _;
use kanso_types::NoteId;
use kanso_types::payloads::{
    DeletePayload, NoteCreatedPayload, NoteMovedPayload, NoteUpdatedPayload,
};
use kanso_types::sync::{EntityType, Operation};
use sqlx::SqliteConnection;

use crate::db::{Engine, enqueue_outbox, insert_tombstone, now_ms};
use crate::error::{EngineError, Result};
use crate::markdown::{self, RefKind};
use crate::models::{Attachment, Note, TaskItem};

const NOTE_COLUMNS: &str =
    "id, notebook_id, title, body_markdown, created_at, updated_at, pinned, favorite, status";

impl Engine {
    pub(crate) async fn rebuild_note_indexes(&self) -> Result<()> {
        let notes = sqlx::query_as::<_, (String, String, String)>(
            "SELECT id, title, body_markdown FROM notes WHERE deleted_at IS NULL",
        )
        .fetch_all(&self.pool)
        .await?;

        let mut tx = self.pool.begin().await?;
        sqlx::query("DELETE FROM notes_fts")
            .execute(&mut *tx)
            .await?;
        sqlx::query("DELETE FROM note_links")
            .execute(&mut *tx)
            .await?;
        sqlx::query("DELETE FROM note_tasks")
            .execute(&mut *tx)
            .await?;

        for (id, title, body) in notes {
            reindex_note(&mut *tx, &id, &title, &body).await?;
        }

        tx.commit().await?;
        Ok(())
    }

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
        let (payload_body, body_cipher) = self.encrypt_body(body)?;
        let payload = NoteCreatedPayload {
            notebook_id: notebook_id.to_string(),
            title: title.to_string(),
            body_markdown: payload_body,
            body_cipher,
            created_at: now,
            updated_at: now,
        };
        enqueue_outbox(
            &mut tx,
            EntityType::Note,
            &id,
            Operation::NoteCreated,
            serde_json::to_value(&payload)?,
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
        let (payload_body, body_cipher) = self.encrypt_body(body)?;
        let payload = NoteUpdatedPayload {
            title: title.clone(),
            body_markdown: payload_body,
            body_cipher,
            updated_at: now,
        };
        enqueue_outbox(
            &mut tx,
            EntityType::Note,
            note_id,
            Operation::NoteUpdated,
            serde_json::to_value(&payload)?,
            now,
        )
        .await?;

        tx.commit().await?;
        Ok(())
    }

    /// Set the checked state for a derived Markdown task line.
    ///
    /// Tasks are indexed from `- [ ]` / `- [x]` Markdown lines, so the body stays
    /// the source of truth. The indexed task id carries the source line number;
    /// older random task ids fall back to matching the first task with the same
    /// text and checked state.
    pub async fn set_task_checked(&self, task_id: &str, checked: bool) -> Result<()> {
        let task: Option<TaskItem> =
            sqlx::query_as("SELECT id, note_id, text, checked FROM note_tasks WHERE id = ?")
                .bind(task_id)
                .fetch_optional(&self.pool)
                .await?;
        let Some(task) = task else {
            return Err(EngineError::NotFound(task_id.to_string()));
        };
        if (task.checked != 0) == checked {
            return Ok(());
        }

        let current: Option<(String,)> =
            sqlx::query_as("SELECT body_markdown FROM notes WHERE id = ? AND deleted_at IS NULL")
                .bind(&task.note_id)
                .fetch_optional(&self.pool)
                .await?;
        let Some((body,)) = current else {
            return Err(EngineError::NotFound(task.note_id));
        };

        let Some(updated) = rewrite_task_line(&body, &task, checked) else {
            return Err(EngineError::NotFound(task_id.to_string()));
        };
        if updated != body {
            self.update_note_body(&task.note_id, &updated).await?;
        }
        Ok(())
    }

    /// Pin/unpin a note. (Local metadata; not synced through the outbox yet.)
    pub async fn set_note_pinned(&self, note_id: &str, pinned: bool) -> Result<()> {
        self.set_note_flag("pinned", pinned as i64, note_id).await
    }

    /// Favorite/unfavorite a note.
    pub async fn set_note_favorite(&self, note_id: &str, favorite: bool) -> Result<()> {
        self.set_note_flag("favorite", favorite as i64, note_id)
            .await
    }

    /// Set a note's status (e.g. `active`, `archived`).
    pub async fn set_note_status(&self, note_id: &str, status: &str) -> Result<()> {
        let result = sqlx::query(
            "UPDATE notes SET status = ?, updated_at = ? WHERE id = ? AND deleted_at IS NULL",
        )
        .bind(status)
        .bind(now_ms())
        .bind(note_id)
        .execute(&self.pool)
        .await?;
        if result.rows_affected() == 0 {
            return Err(EngineError::NotFound(note_id.to_string()));
        }
        Ok(())
    }

    // Shared setter for the integer flag columns (`pinned`, `favorite`). The
    // column name is a fixed internal literal, never user input.
    async fn set_note_flag(&self, column: &str, value: i64, note_id: &str) -> Result<()> {
        let sql = format!(
            "UPDATE notes SET {column} = ?, updated_at = ? WHERE id = ? AND deleted_at IS NULL"
        );
        let result = sqlx::query(&sql)
            .bind(value)
            .bind(now_ms())
            .bind(note_id)
            .execute(&self.pool)
            .await?;
        if result.rows_affected() == 0 {
            return Err(EngineError::NotFound(note_id.to_string()));
        }
        Ok(())
    }

    /// Get-or-create today's daily note (titled `YYYY-MM-DD`) in a notebook.
    pub async fn create_daily_note(&self, notebook_id: &str) -> Result<Note> {
        let date = chrono::Utc::now().format("%Y-%m-%d").to_string();
        let query = format!(
            "SELECT {NOTE_COLUMNS} FROM notes \
             WHERE notebook_id = ? AND title = ? AND deleted_at IS NULL"
        );
        if let Some(existing) = sqlx::query_as::<_, Note>(&query)
            .bind(notebook_id)
            .bind(&date)
            .fetch_optional(&self.pool)
            .await?
        {
            return Ok(existing);
        }
        self.create_note(notebook_id, &date, "").await
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

    /// Permanently remove a soft-deleted note and all its derived rows. The
    /// tombstone is kept so sync won't resurrect it.
    pub async fn purge_note(&self, note_id: &str) -> Result<()> {
        let mut tx = self.pool.begin().await?;

        let present: Option<(i64,)> =
            sqlx::query_as("SELECT 1 FROM notes WHERE id = ? AND deleted_at IS NOT NULL")
                .bind(note_id)
                .fetch_optional(&mut *tx)
                .await?;
        if present.is_none() {
            return Err(EngineError::NotFound(note_id.to_string()));
        }

        for stmt in [
            "DELETE FROM notes_fts WHERE note_id = ?",
            "DELETE FROM note_tags WHERE note_id = ?",
            "DELETE FROM attachments WHERE note_id = ?",
            "DELETE FROM sketches WHERE note_id = ?",
            "DELETE FROM note_links WHERE source_note_id = ?",
            "DELETE FROM note_tasks WHERE note_id = ?",
            "DELETE FROM revisions WHERE note_id = ?",
            "DELETE FROM notes WHERE id = ?",
        ] {
            sqlx::query(stmt).bind(note_id).execute(&mut *tx).await?;
        }

        tx.commit().await?;
        Ok(())
    }

    /// Rename a note's title, re-indexing FTS and enqueuing a sync event.
    pub async fn rename_note(&self, note_id: &str, title: &str) -> Result<()> {
        let now = now_ms();
        let mut tx = self.pool.begin().await?;

        let current: Option<(String,)> =
            sqlx::query_as("SELECT body_markdown FROM notes WHERE id = ? AND deleted_at IS NULL")
                .bind(note_id)
                .fetch_optional(&mut *tx)
                .await?;
        let Some((body,)) = current else {
            return Err(EngineError::NotFound(note_id.to_string()));
        };

        sqlx::query("UPDATE notes SET title = ?, updated_at = ? WHERE id = ?")
            .bind(title)
            .bind(now)
            .bind(note_id)
            .execute(&mut *tx)
            .await?;

        reindex_note(&mut *tx, note_id, title, &body).await?;
        let (payload_body, body_cipher) = self.encrypt_body(&body)?;
        let payload = NoteUpdatedPayload {
            title: title.to_string(),
            body_markdown: payload_body,
            body_cipher,
            updated_at: now,
        };
        enqueue_outbox(
            &mut *tx,
            EntityType::Note,
            note_id,
            Operation::NoteUpdated,
            serde_json::to_value(&payload)?,
            now,
        )
        .await?;

        tx.commit().await?;
        Ok(())
    }

    /// Full-text search scoped to a single notebook.
    pub async fn search_notes_in(&self, notebook_id: &str, query: &str) -> Result<Vec<Note>> {
        let cols = NOTE_COLUMNS
            .split(", ")
            .map(|c| format!("n.{c}"))
            .collect::<Vec<_>>()
            .join(", ");
        let sql = format!(
            "SELECT {cols} FROM notes_fts f \
             JOIN notes n ON n.id = f.note_id \
             WHERE notes_fts MATCH ? AND n.notebook_id = ? AND n.deleted_at IS NULL \
             ORDER BY rank"
        );
        Ok(sqlx::query_as::<_, Note>(&sql)
            .bind(query)
            .bind(notebook_id)
            .fetch_all(&self.pool)
            .await?)
    }

    /// Fetch a single note by id, returning `None` if it does not exist or has
    /// been soft-deleted.
    pub async fn get_note(&self, note_id: &str) -> Result<Option<Note>> {
        let query = format!("SELECT {NOTE_COLUMNS} FROM notes WHERE id = ? AND deleted_at IS NULL");
        let note = sqlx::query_as::<_, Note>(&query)
            .bind(note_id)
            .fetch_optional(&self.pool)
            .await?;
        Ok(note)
    }

    /// Render a note body to preview HTML using the engine-owned Markdown and
    /// Kanso-reference rules.
    pub async fn render_note_html(&self, note_id: &str) -> Result<String> {
        let Some(note) = self.get_note(note_id).await? else {
            return Err(EngineError::NotFound(note_id.to_string()));
        };

        let mut sketch_blocks = HashMap::new();
        let mut attachment_blocks = HashMap::new();
        let attachments = self.list_attachments(note_id).await?;
        for reference in markdown::extract(&note.body_markdown).refs.into_iter() {
            match reference.kind {
                RefKind::Sketch => {
                    if sketch_blocks.contains_key(&reference.target) {
                        continue;
                    }
                    let sketch_id = if reference.target.starts_with("sketch:") {
                        reference.target.clone()
                    } else {
                        format!("sketch:{}", reference.target)
                    };
                    if let Ok(png) = self.render_sketch_preview(&sketch_id, 960, 420).await {
                        let encoded = base64::engine::general_purpose::STANDARD.encode(png);
                        sketch_blocks.insert(
                            reference.target.clone(),
                            format!(
                                "<a class=\"kanso-block-link\" href=\"kanso://sketch/{}\">\
                                 <figure class=\"kanso-block kanso-sketch\" data-kanso-kind=\"sketch\" data-kanso-target=\"{}\">\
                                 <div class=\"kanso-block-icon\">Sketch</div>\
                                 <img src=\"data:image/png;base64,{}\" alt=\"Sketch preview\" />\
                                 <figcaption>{}</figcaption>\
                                 </figure>\
                                 </a>",
                                markdown::percent_encode(&reference.target),
                                markdown::escape_attr(&reference.target),
                                encoded,
                                markdown::escape_html(&reference.target)
                            ),
                        );
                    }
                }
                RefKind::Attachment => {
                    if attachment_blocks.contains_key(&reference.target) {
                        continue;
                    }
                    if let Some(attachment) = attachments.iter().find(|attachment| {
                        attachment_matches_reference(attachment, &reference.target)
                    }) {
                        attachment_blocks.insert(
                            reference.target.clone(),
                            render_attachment_block(&reference.target, attachment),
                        );
                    }
                }
                RefKind::Note => {}
            }
        }

        Ok(markdown::render_html_with_reference_blocks(
            &note.body_markdown,
            |kind, target| match kind {
                RefKind::Sketch => sketch_blocks.get(target).cloned(),
                RefKind::Attachment => attachment_blocks.get(target).cloned(),
                RefKind::Note => None,
            },
        ))
    }

    /// Soft-delete a note, remove it from the FTS index, write a tombstone, and
    /// enqueue a `NoteDeleted` outbox event — all in one transaction.
    pub async fn delete_note(&self, note_id: &str) -> Result<()> {
        let now = now_ms();
        let mut tx = self.pool.begin().await?;

        let result = sqlx::query(
            "UPDATE notes SET deleted_at = ?, updated_at = ? WHERE id = ? AND deleted_at IS NULL",
        )
        .bind(now)
        .bind(now)
        .bind(note_id)
        .execute(&mut *tx)
        .await?;

        if result.rows_affected() == 0 {
            return Err(EngineError::NotFound(note_id.to_string()));
        }

        sqlx::query("DELETE FROM notes_fts WHERE note_id = ?")
            .bind(note_id)
            .execute(&mut *tx)
            .await?;

        insert_tombstone(&mut *tx, EntityType::Note, note_id, now).await?;

        let payload = DeletePayload { deleted_at: now };
        enqueue_outbox(
            &mut *tx,
            EntityType::Note,
            note_id,
            Operation::NoteDeleted,
            serde_json::to_value(&payload)?,
            now,
        )
        .await?;

        tx.commit().await?;
        Ok(())
    }

    /// Restore a previously soft-deleted note: clear `deleted_at`, remove the
    /// tombstone, re-add the FTS row, and enqueue a `NoteUpdated` event.
    pub async fn restore_note(&self, note_id: &str) -> Result<()> {
        let now = now_ms();
        let mut tx = self.pool.begin().await?;

        let current: Option<(String, String)> =
            sqlx::query_as("SELECT title, body_markdown FROM notes WHERE id = ?")
                .bind(note_id)
                .fetch_optional(&mut *tx)
                .await?;
        let Some((title, body)) = current else {
            return Err(EngineError::NotFound(note_id.to_string()));
        };

        sqlx::query("UPDATE notes SET deleted_at = NULL, updated_at = ? WHERE id = ?")
            .bind(now)
            .bind(note_id)
            .execute(&mut *tx)
            .await?;

        sqlx::query("DELETE FROM tombstones WHERE entity_type = 'note' AND entity_id = ?")
            .bind(note_id)
            .execute(&mut *tx)
            .await?;

        reindex_note(&mut *tx, note_id, &title, &body).await?;

        let (payload_body, body_cipher) = self.encrypt_body(&body)?;
        let payload = NoteUpdatedPayload {
            title: title.clone(),
            body_markdown: payload_body,
            body_cipher,
            updated_at: now,
        };
        enqueue_outbox(
            &mut *tx,
            EntityType::Note,
            note_id,
            Operation::NoteUpdated,
            serde_json::to_value(&payload)?,
            now,
        )
        .await?;

        tx.commit().await?;
        Ok(())
    }

    /// Move a note to a different notebook, updating `notebook_id` atomically
    /// and emitting a `NoteMoved` outbox event.
    pub async fn move_note(&self, note_id: &str, notebook_id: &str) -> Result<()> {
        let now = now_ms();
        let mut tx = self.pool.begin().await?;

        let result = sqlx::query(
            "UPDATE notes SET notebook_id = ?, updated_at = ? WHERE id = ? AND deleted_at IS NULL",
        )
        .bind(notebook_id)
        .bind(now)
        .bind(note_id)
        .execute(&mut *tx)
        .await?;

        if result.rows_affected() == 0 {
            return Err(EngineError::NotFound(note_id.to_string()));
        }

        let payload = NoteMovedPayload {
            notebook_id: notebook_id.to_string(),
            updated_at: now,
        };
        enqueue_outbox(
            &mut *tx,
            EntityType::Note,
            note_id,
            Operation::NoteMoved,
            serde_json::to_value(&payload)?,
            now,
        )
        .await?;

        tx.commit().await?;
        Ok(())
    }
}

/// Rebuild the FTS row and the derived tasks/links for a note. Runs inside the
/// caller's transaction so the index can never drift from the note.
pub(crate) async fn reindex_note(
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
            .bind(format!("task:{}:{}", task.line_index, note_id))
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

fn attachment_matches_reference(attachment: &Attachment, target: &str) -> bool {
    attachment.id == target
        || attachment.id == format!("attachment:{target}")
        || attachment.filename == target
        || attachment.content_hash == target
}

fn render_attachment_block(target: &str, attachment: &Attachment) -> String {
    let maybe_inline_image = attachment
        .local_path
        .as_deref()
        .filter(|_| attachment.mime_type.starts_with("image/"))
        .and_then(|path| fs::read(path).ok())
        .filter(|bytes| bytes.len() <= 20 * 1024 * 1024)
        .map(|bytes| base64::engine::general_purpose::STANDARD.encode(bytes));

    match maybe_inline_image {
        Some(encoded) => format!(
            "<a class=\"kanso-block-link\" href=\"kanso://attachment/{}\">\
             <figure class=\"kanso-block kanso-attachment kanso-attachment-image\" data-kanso-kind=\"attachment\" data-kanso-target=\"{}\">\
             <div class=\"kanso-block-icon\">Attachment</div>\
             <img src=\"data:{};base64,{}\" alt=\"{}\" />\
             <figcaption>{} · {}</figcaption>\
             </figure>\
             </a>",
            markdown::percent_encode(target),
            markdown::escape_attr(target),
            markdown::escape_attr(&attachment.mime_type),
            encoded,
            markdown::escape_attr(&attachment.filename),
            markdown::escape_html(&attachment.filename),
            format_size(attachment.size_bytes),
        ),
        None => format!(
            "<a class=\"kanso-block-link\" href=\"kanso://attachment/{}\">\
             <figure class=\"kanso-block kanso-attachment\" data-kanso-kind=\"attachment\" data-kanso-target=\"{}\">\
             <div class=\"kanso-block-icon\">Attachment</div>\
             <figcaption>{}</figcaption>\
             <div class=\"kanso-attachment-meta\">{} · {}</div>\
             </figure>\
             </a>",
            markdown::percent_encode(target),
            markdown::escape_attr(target),
            markdown::escape_html(&attachment.filename),
            markdown::escape_html(&attachment.mime_type),
            format_size(attachment.size_bytes),
        ),
    }
}

fn format_size(bytes: i64) -> String {
    let bytes = bytes.max(0) as f64;
    let units = ["B", "KB", "MB", "GB"];
    let mut value = bytes;
    let mut unit = units[0];
    for next in units.iter().skip(1) {
        if value < 1024.0 {
            break;
        }
        value /= 1024.0;
        unit = next;
    }

    if unit == "B" {
        format!("{} {unit}", value as i64)
    } else {
        format!("{value:.1} {unit}")
    }
}

fn rewrite_task_line(body: &str, task: &TaskItem, checked: bool) -> Option<String> {
    let preferred_line = task_line_index(&task.id);
    let mut out = String::with_capacity(body.len());
    let mut changed = false;

    for (line_index, segment) in body.split_inclusive('\n').enumerate() {
        let (line, ending) = split_line_ending(segment);
        let mut replacement: Option<String> = None;

        if !changed {
            let is_preferred_line = preferred_line == Some(line_index);
            let is_fallback_match =
                preferred_line.is_none() && task_line_matches(line, &task.text, task.checked != 0);
            if is_preferred_line || is_fallback_match {
                replacement = rewrite_task_marker(line, checked);
            }
        }

        if let Some(rewritten) = replacement {
            out.push_str(&rewritten);
            changed = true;
        } else {
            out.push_str(line);
        }
        out.push_str(ending);
    }

    changed.then_some(out)
}

fn task_line_index(task_id: &str) -> Option<usize> {
    let rest = task_id.strip_prefix("task:")?;
    let (line, _) = rest.split_once(':')?;
    line.parse().ok()
}

fn split_line_ending(segment: &str) -> (&str, &str) {
    if let Some(line) = segment.strip_suffix("\r\n") {
        (line, "\r\n")
    } else if let Some(line) = segment.strip_suffix('\n') {
        (line, "\n")
    } else {
        (segment, "")
    }
}

fn task_line_matches(line: &str, text: &str, checked: bool) -> bool {
    match task_line(line) {
        Some((line_checked, line_text)) => line_checked == checked && line_text == text,
        None => false,
    }
}

fn rewrite_task_marker(line: &str, checked: bool) -> Option<String> {
    let trimmed = line.trim_start();
    if task_line(line).is_none() {
        return None;
    }

    let leading_len = line.len() - trimmed.len();
    let mut updated = String::with_capacity(line.len());
    updated.push_str(&line[..leading_len]);
    updated.push_str(if checked { "- [x]" } else { "- [ ]" });
    updated.push_str(&trimmed[5..]);
    Some(updated)
}

fn task_line(line: &str) -> Option<(bool, &str)> {
    let trimmed = line.trim_start();
    let checked = if trimmed.starts_with("- [x]") || trimmed.starts_with("- [X]") {
        true
    } else if trimmed.starts_with("- [ ]") {
        false
    } else {
        return None;
    };
    Some((checked, trimmed[5..].trim()))
}
