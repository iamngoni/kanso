//! Attachment commands.

use kanso_types::AttachmentId;
use kanso_types::payloads::{AttachmentPayload, DeletePayload};
use kanso_types::sync::{EntityType, Operation};

use crate::db::{Engine, enqueue_outbox, insert_tombstone, now_ms};
use crate::error::{EngineError, Result};
use crate::models::{Attachment, NewAttachment};

impl Engine {
    /// Persist a new attachment record for `note_id` and enqueue a sync event.
    pub async fn attach_file(&self, note_id: &str, input: NewAttachment) -> Result<Attachment> {
        let id = AttachmentId::new().0;
        let now = now_ms();

        let mut tx = self.pool.begin().await?;
        sqlx::query(
            "INSERT INTO attachments \
             (id, note_id, filename, mime_type, size_bytes, content_hash, local_path, created_at, updated_at) \
             VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)",
        )
        .bind(&id)
        .bind(note_id)
        .bind(&input.filename)
        .bind(&input.mime_type)
        .bind(input.size_bytes)
        .bind(&input.content_hash)
        .bind(&input.local_path)
        .bind(now)
        .bind(now)
        .execute(&mut *tx)
        .await?;

        let payload = AttachmentPayload {
            note_id: note_id.to_string(),
            filename: input.filename.clone(),
            mime_type: input.mime_type.clone(),
            size_bytes: input.size_bytes,
            content_hash: input.content_hash.clone(),
            updated_at: now,
        };
        enqueue_outbox(
            &mut *tx,
            EntityType::Attachment,
            &id,
            Operation::AttachmentAdded,
            serde_json::to_value(&payload)?,
            now,
        )
        .await?;

        tx.commit().await?;

        Ok(Attachment {
            id,
            note_id: note_id.to_string(),
            filename: input.filename,
            mime_type: input.mime_type,
            size_bytes: input.size_bytes,
            content_hash: input.content_hash,
            local_path: input.local_path,
            remote_key: None,
            created_at: now,
            updated_at: now,
        })
    }

    /// Return all attachments belonging to `note_id`, ordered by creation time.
    pub async fn list_attachments(&self, note_id: &str) -> Result<Vec<Attachment>> {
        let attachments = sqlx::query_as::<_, Attachment>(
            "SELECT id, note_id, filename, mime_type, size_bytes, content_hash, \
             local_path, remote_key, created_at, updated_at \
             FROM attachments WHERE note_id = ? ORDER BY created_at",
        )
        .bind(note_id)
        .fetch_all(&self.pool)
        .await?;
        Ok(attachments)
    }

    /// Hard-delete an attachment by `id`, record a tombstone, and enqueue a
    /// deletion event.  Returns [`EngineError::NotFound`] if no row is deleted.
    pub async fn delete_attachment(&self, id: &str) -> Result<()> {
        let now = now_ms();
        let mut tx = self.pool.begin().await?;

        let result = sqlx::query("DELETE FROM attachments WHERE id = ?")
            .bind(id)
            .execute(&mut *tx)
            .await?;

        if result.rows_affected() == 0 {
            return Err(EngineError::NotFound(id.to_string()));
        }

        insert_tombstone(&mut *tx, EntityType::Attachment, id, now).await?;

        let payload = DeletePayload { deleted_at: now };
        enqueue_outbox(
            &mut *tx,
            EntityType::Attachment,
            id,
            Operation::AttachmentDeleted,
            serde_json::to_value(&payload)?,
            now,
        )
        .await?;

        tx.commit().await?;
        Ok(())
    }
}
