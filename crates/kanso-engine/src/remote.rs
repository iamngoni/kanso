//! Inbound sync — applying remote changes from the backend.
//!
//! This is the counterpart to the outbox: the native sync adapter hands the
//! engine [`RemoteChange`]s and the engine reconciles them into local state.
//! Two rules are absolute here: applying a remote change never writes to the
//! outbox (no echo loop), and a losing note body is always preserved as a
//! revision rather than discarded.

use kanso_types::payloads::{
    AttachmentPayload, DeletePayload, NoteCreatedPayload, NoteMovedPayload, NoteTagPayload,
    NoteUpdatedPayload, NotebookPayload, SketchPayload, TagPayload,
};
use kanso_types::sync::{EntityType, Operation, RemoteChange};
use sqlx::SqliteConnection;

use crate::db::{Engine, insert_tombstone, tombstoned_after};
use crate::error::Result;
use crate::models::ApplyOutcome;
use crate::notes::reindex_note;

impl Engine {
    /// Apply a single remote change to local state, returning how it resolved.
    /// Runs in one transaction and never enqueues an outbox event.
    pub async fn apply_remote_change(&self, change: &RemoteChange) -> Result<ApplyOutcome> {
        let mut tx = self.pool.begin().await?;
        let event = &change.event;
        let id = event.entity_id.as_str();

        let outcome = match event.operation {
            Operation::NotebookCreated | Operation::NotebookUpdated => {
                let p: NotebookPayload = serde_json::from_value(event.payload.clone())?;
                apply_notebook_upsert(&mut tx, id, &p).await?
            }
            Operation::NotebookDeleted => {
                let p: DeletePayload = serde_json::from_value(event.payload.clone())?;
                apply_soft_delete(&mut tx, "notebooks", EntityType::Notebook, id, p.deleted_at).await?
            }
            Operation::NoteCreated => {
                let p: NoteCreatedPayload = serde_json::from_value(event.payload.clone())?;
                let body = self.resolve_body(&p.body_markdown, &p.body_cipher)?;
                apply_note_create(&mut tx, id, &p, &body).await?
            }
            Operation::NoteUpdated => {
                let p: NoteUpdatedPayload = serde_json::from_value(event.payload.clone())?;
                if tombstoned_after(&mut tx, EntityType::Note, id, p.updated_at).await? {
                    ApplyOutcome::Skipped
                } else {
                    let body = self.resolve_body(&p.body_markdown, &p.body_cipher)?;
                    apply_note_body(&mut tx, id, &p.title, &body, p.updated_at).await?
                }
            }
            Operation::NoteMoved => {
                let p: NoteMovedPayload = serde_json::from_value(event.payload.clone())?;
                apply_note_move(&mut tx, id, &p).await?
            }
            Operation::NoteDeleted => {
                let p: DeletePayload = serde_json::from_value(event.payload.clone())?;
                let outcome =
                    apply_soft_delete(&mut tx, "notes", EntityType::Note, id, p.deleted_at).await?;
                sqlx::query("DELETE FROM notes_fts WHERE note_id = ?")
                    .bind(id)
                    .execute(&mut *tx)
                    .await?;
                outcome
            }
            Operation::TagCreated | Operation::TagUpdated => {
                let p: TagPayload = serde_json::from_value(event.payload.clone())?;
                apply_tag_upsert(&mut tx, id, &p).await?
            }
            Operation::TagDeleted => {
                let p: DeletePayload = serde_json::from_value(event.payload.clone())?;
                sqlx::query("DELETE FROM note_tags WHERE tag_id = ?")
                    .bind(id)
                    .execute(&mut *tx)
                    .await?;
                sqlx::query("DELETE FROM tags WHERE id = ?")
                    .bind(id)
                    .execute(&mut *tx)
                    .await?;
                insert_tombstone(&mut tx, EntityType::Tag, id, p.deleted_at).await?;
                ApplyOutcome::Deleted
            }
            Operation::NoteTagged => {
                let p: NoteTagPayload = serde_json::from_value(event.payload.clone())?;
                sqlx::query("INSERT OR IGNORE INTO note_tags (note_id, tag_id) VALUES (?, ?)")
                    .bind(&p.note_id)
                    .bind(&p.tag_id)
                    .execute(&mut *tx)
                    .await?;
                ApplyOutcome::Applied
            }
            Operation::NoteUntagged => {
                let p: NoteTagPayload = serde_json::from_value(event.payload.clone())?;
                sqlx::query("DELETE FROM note_tags WHERE note_id = ? AND tag_id = ?")
                    .bind(&p.note_id)
                    .bind(&p.tag_id)
                    .execute(&mut *tx)
                    .await?;
                ApplyOutcome::Applied
            }
            Operation::AttachmentAdded => {
                let p: AttachmentPayload = serde_json::from_value(event.payload.clone())?;
                apply_attachment_upsert(&mut tx, id, &p).await?
            }
            Operation::AttachmentDeleted => {
                let p: DeletePayload = serde_json::from_value(event.payload.clone())?;
                sqlx::query("DELETE FROM attachments WHERE id = ?")
                    .bind(id)
                    .execute(&mut *tx)
                    .await?;
                insert_tombstone(&mut tx, EntityType::Attachment, id, p.deleted_at).await?;
                ApplyOutcome::Deleted
            }
            Operation::SketchCreated | Operation::SketchUpdated => {
                let p: SketchPayload = serde_json::from_value(event.payload.clone())?;
                let blob = self.resolve_blob(&p.data_blob, &p.data_cipher)?;
                apply_sketch_upsert(&mut tx, id, &p, &blob).await?
            }
            Operation::SketchDeleted => {
                let p: DeletePayload = serde_json::from_value(event.payload.clone())?;
                sqlx::query("DELETE FROM sketches WHERE id = ?")
                    .bind(id)
                    .execute(&mut *tx)
                    .await?;
                insert_tombstone(&mut tx, EntityType::Sketch, id, p.deleted_at).await?;
                ApplyOutcome::Deleted
            }
        };

        tx.commit().await?;
        Ok(outcome)
    }
}

async fn apply_notebook_upsert(
    conn: &mut SqliteConnection,
    id: &str,
    p: &NotebookPayload,
) -> Result<ApplyOutcome> {
    if tombstoned_after(&mut *conn, EntityType::Notebook, id, p.updated_at).await? {
        return Ok(ApplyOutcome::Skipped);
    }
    let existing: Option<(i64,)> = sqlx::query_as("SELECT updated_at FROM notebooks WHERE id = ?")
        .bind(id)
        .fetch_optional(&mut *conn)
        .await?;
    match existing {
        None => {
            let created = p.created_at.unwrap_or(p.updated_at);
            sqlx::query(
                "INSERT INTO notebooks (id, name, parent_id, sort_order, created_at, updated_at) \
                 VALUES (?, ?, ?, 0, ?, ?)",
            )
            .bind(id)
            .bind(&p.name)
            .bind(&p.parent_id)
            .bind(created)
            .bind(p.updated_at)
            .execute(&mut *conn)
            .await?;
            Ok(ApplyOutcome::Applied)
        }
        Some((local_updated,)) if p.updated_at >= local_updated => {
            sqlx::query(
                "UPDATE notebooks SET name = ?, parent_id = ?, updated_at = ?, deleted_at = NULL \
                 WHERE id = ?",
            )
            .bind(&p.name)
            .bind(&p.parent_id)
            .bind(p.updated_at)
            .bind(id)
            .execute(&mut *conn)
            .await?;
            Ok(ApplyOutcome::Applied)
        }
        Some(_) => Ok(ApplyOutcome::Skipped),
    }
}

/// Generic tombstone-aware soft delete by table name. `table` is always a fixed
/// internal literal, never user input.
async fn apply_soft_delete(
    conn: &mut SqliteConnection,
    table: &str,
    entity: EntityType,
    id: &str,
    deleted_at: i64,
) -> Result<ApplyOutcome> {
    let sql = format!("UPDATE {table} SET deleted_at = ? WHERE id = ?");
    sqlx::query(&sql).bind(deleted_at).bind(id).execute(&mut *conn).await?;
    insert_tombstone(&mut *conn, entity, id, deleted_at).await?;
    Ok(ApplyOutcome::Deleted)
}

async fn apply_note_create(
    conn: &mut SqliteConnection,
    id: &str,
    p: &NoteCreatedPayload,
    body: &str,
) -> Result<ApplyOutcome> {
    if tombstoned_after(&mut *conn, EntityType::Note, id, p.updated_at).await? {
        return Ok(ApplyOutcome::Skipped);
    }
    let existing: Option<(i64,)> = sqlx::query_as("SELECT updated_at FROM notes WHERE id = ?")
        .bind(id)
        .fetch_optional(&mut *conn)
        .await?;
    if existing.is_some() {
        // Already present — reconcile via the same last-write-wins body path.
        return apply_note_body(&mut *conn, id, &p.title, body, p.updated_at).await;
    }
    sqlx::query(
        "INSERT INTO notes (id, notebook_id, title, body_markdown, created_at, updated_at) \
         VALUES (?, ?, ?, ?, ?, ?)",
    )
    .bind(id)
    .bind(&p.notebook_id)
    .bind(&p.title)
    .bind(body)
    .bind(p.created_at)
    .bind(p.updated_at)
    .execute(&mut *conn)
    .await?;
    reindex_note(&mut *conn, id, &p.title, body).await?;
    Ok(ApplyOutcome::Applied)
}

/// Shared last-write-wins reconciliation for a note body, preserving the losing
/// version as a revision in both directions.
async fn apply_note_body(
    conn: &mut SqliteConnection,
    id: &str,
    title: &str,
    body: &str,
    remote_updated: i64,
) -> Result<ApplyOutcome> {
    let existing: Option<(String, i64)> =
        sqlx::query_as("SELECT body_markdown, updated_at FROM notes WHERE id = ?")
            .bind(id)
            .fetch_optional(&mut *conn)
            .await?;
    let Some((local_body, local_updated)) = existing else {
        // We don't have the note yet; an isolated update can't create it.
        return Ok(ApplyOutcome::Skipped);
    };

    if remote_updated >= local_updated {
        // Remote wins: snapshot the local body, then apply remote.
        let revision_id = kanso_types::RevisionId::new().0;
        sqlx::query(
            "INSERT INTO revisions (id, note_id, body_markdown, reason, source, created_at) \
             VALUES (?, ?, ?, 'superseded by remote', 'sync', ?)",
        )
        .bind(&revision_id)
        .bind(id)
        .bind(&local_body)
        .bind(remote_updated)
        .execute(&mut *conn)
        .await?;

        sqlx::query("UPDATE notes SET title = ?, body_markdown = ?, updated_at = ? WHERE id = ?")
            .bind(title)
            .bind(body)
            .bind(remote_updated)
            .bind(id)
            .execute(&mut *conn)
            .await?;
        reindex_note(&mut *conn, id, title, body).await?;
        Ok(ApplyOutcome::Applied)
    } else {
        // Local is newer: keep local, preserve the remote version as a conflict
        // revision. Never discard text.
        let revision_id = kanso_types::RevisionId::new().0;
        sqlx::query(
            "INSERT INTO revisions (id, note_id, body_markdown, reason, source, created_at) \
             VALUES (?, ?, ?, 'conflicting remote version', 'conflict', ?)",
        )
        .bind(&revision_id)
        .bind(id)
        .bind(body)
        .bind(remote_updated)
        .execute(&mut *conn)
        .await?;
        Ok(ApplyOutcome::Conflicted)
    }
}

async fn apply_note_move(
    conn: &mut SqliteConnection,
    id: &str,
    p: &NoteMovedPayload,
) -> Result<ApplyOutcome> {
    if tombstoned_after(&mut *conn, EntityType::Note, id, p.updated_at).await? {
        return Ok(ApplyOutcome::Skipped);
    }
    let existing: Option<(i64,)> = sqlx::query_as("SELECT updated_at FROM notes WHERE id = ?")
        .bind(id)
        .fetch_optional(&mut *conn)
        .await?;
    match existing {
        Some((local_updated,)) if p.updated_at >= local_updated => {
            sqlx::query("UPDATE notes SET notebook_id = ?, updated_at = ? WHERE id = ?")
                .bind(&p.notebook_id)
                .bind(p.updated_at)
                .bind(id)
                .execute(&mut *conn)
                .await?;
            Ok(ApplyOutcome::Applied)
        }
        _ => Ok(ApplyOutcome::Skipped),
    }
}

async fn apply_tag_upsert(
    conn: &mut SqliteConnection,
    id: &str,
    p: &TagPayload,
) -> Result<ApplyOutcome> {
    if tombstoned_after(&mut *conn, EntityType::Tag, id, p.updated_at).await? {
        return Ok(ApplyOutcome::Skipped);
    }
    sqlx::query(
        "INSERT INTO tags (id, name, color, created_at, updated_at) VALUES (?, ?, ?, ?, ?) \
         ON CONFLICT (id) DO UPDATE SET name = excluded.name, color = excluded.color, \
         updated_at = excluded.updated_at",
    )
    .bind(id)
    .bind(&p.name)
    .bind(&p.color)
    .bind(p.updated_at)
    .bind(p.updated_at)
    .execute(&mut *conn)
    .await?;
    Ok(ApplyOutcome::Applied)
}

async fn apply_attachment_upsert(
    conn: &mut SqliteConnection,
    id: &str,
    p: &AttachmentPayload,
) -> Result<ApplyOutcome> {
    if tombstoned_after(&mut *conn, EntityType::Attachment, id, p.updated_at).await? {
        return Ok(ApplyOutcome::Skipped);
    }
    sqlx::query(
        "INSERT INTO attachments \
         (id, note_id, filename, mime_type, size_bytes, content_hash, created_at, updated_at) \
         VALUES (?, ?, ?, ?, ?, ?, ?, ?) \
         ON CONFLICT (id) DO UPDATE SET filename = excluded.filename, \
         mime_type = excluded.mime_type, size_bytes = excluded.size_bytes, \
         content_hash = excluded.content_hash, updated_at = excluded.updated_at",
    )
    .bind(id)
    .bind(&p.note_id)
    .bind(&p.filename)
    .bind(&p.mime_type)
    .bind(p.size_bytes)
    .bind(&p.content_hash)
    .bind(p.updated_at)
    .bind(p.updated_at)
    .execute(&mut *conn)
    .await?;
    Ok(ApplyOutcome::Applied)
}

async fn apply_sketch_upsert(
    conn: &mut SqliteConnection,
    id: &str,
    p: &SketchPayload,
    blob: &[u8],
) -> Result<ApplyOutcome> {
    if tombstoned_after(&mut *conn, EntityType::Sketch, id, p.updated_at).await? {
        return Ok(ApplyOutcome::Skipped);
    }
    sqlx::query(
        "INSERT INTO sketches \
         (id, note_id, title, format_version, data_blob, created_at, updated_at) \
         VALUES (?, ?, ?, ?, ?, ?, ?) \
         ON CONFLICT (id) DO UPDATE SET title = excluded.title, \
         format_version = excluded.format_version, data_blob = excluded.data_blob, \
         updated_at = excluded.updated_at",
    )
    .bind(id)
    .bind(&p.note_id)
    .bind(&p.title)
    .bind(p.format_version)
    .bind(blob)
    .bind(p.updated_at)
    .bind(p.updated_at)
    .execute(&mut *conn)
    .await?;
    Ok(ApplyOutcome::Applied)
}
