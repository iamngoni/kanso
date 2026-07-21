//! Sync outbox read API — the boundary the native sync adapter calls.
//!
//! The engine never opens a socket. It hands pending [`OutboxEvent`]s to the
//! native layer and marks them acknowledged once the backend confirms.

use kanso_types::payloads::{NoteCreatedPayload, NoteUpdatedPayload, SketchPayload};
use kanso_types::sync::{EntityType, Operation, OutboxEvent};
use uuid::Uuid;

use crate::db::Engine;
use crate::error::Result;

struct OutboxRow {
    id: String,
    entity_type: String,
    entity_id: String,
    operation: String,
    payload_json: String,
    local_sequence: i64,
}
impl_sqlite_from_row!(OutboxRow {
    id,
    entity_type,
    entity_id,
    operation,
    payload_json,
    local_sequence,
});

impl Engine {
    /// Count outbox events not yet acknowledged by the backend.
    pub async fn pending_outbox_count(&self) -> Result<i64> {
        let (count,): (i64,) =
            sqlx::query_as("SELECT COUNT(*) FROM sync_outbox WHERE acknowledged_at IS NULL")
                .fetch_one(&self.pool)
                .await?;
        Ok(count)
    }

    /// Oldest pending events, ready to ship to the backend.
    pub async fn get_pending_sync_ops(&self, limit: i64) -> Result<Vec<OutboxEvent>> {
        let rows = sqlx::query_as::<_, OutboxRow>(
            "SELECT id, entity_type, entity_id, operation, payload_json, local_sequence \
             FROM sync_outbox WHERE acknowledged_at IS NULL ORDER BY local_sequence LIMIT ?",
        )
        .bind(limit)
        .fetch_all(&self.pool)
        .await?;

        rows.into_iter()
            .map(|row| {
                let mut event = row.into_event();
                event.payload = self.encrypt_pending_payload(event.operation, event.payload)?;
                Ok(event)
            })
            .collect()
    }

    /// Mark events acknowledged after the backend accepts them.
    pub async fn mark_sync_ops_acknowledged(&self, ids: &[String]) -> Result<()> {
        let now = crate::db::now_ms();
        let mut tx = self.pool.begin().await?;
        for id in ids {
            sqlx::query("UPDATE sync_outbox SET acknowledged_at = ? WHERE id = ?")
                .bind(now)
                .bind(id)
                .execute(&mut *tx)
                .await?;
        }
        tx.commit().await?;
        Ok(())
    }
}

impl Engine {
    fn encrypt_pending_payload(
        &self,
        operation: Operation,
        payload: serde_json::Value,
    ) -> Result<serde_json::Value> {
        match operation {
            Operation::NoteCreated => {
                let mut payload: NoteCreatedPayload = serde_json::from_value(payload)?;
                if self.enc.is_some() && payload.body_cipher.is_none() {
                    let (body_markdown, body_cipher) = self.encrypt_body(&payload.body_markdown)?;
                    payload.body_markdown = body_markdown;
                    payload.body_cipher = body_cipher;
                }
                Ok(serde_json::to_value(payload)?)
            }
            Operation::NoteUpdated => {
                let mut payload: NoteUpdatedPayload = serde_json::from_value(payload)?;
                if self.enc.is_some() && payload.body_cipher.is_none() {
                    let (body_markdown, body_cipher) = self.encrypt_body(&payload.body_markdown)?;
                    payload.body_markdown = body_markdown;
                    payload.body_cipher = body_cipher;
                }
                Ok(serde_json::to_value(payload)?)
            }
            Operation::SketchCreated | Operation::SketchUpdated => {
                let mut payload: SketchPayload = serde_json::from_value(payload)?;
                if self.enc.is_some() && payload.data_cipher.is_none() {
                    let (data_blob, data_cipher) = self.encrypt_blob(&payload.data_blob)?;
                    payload.data_blob = data_blob;
                    payload.data_cipher = data_cipher;
                }
                Ok(serde_json::to_value(payload)?)
            }
            _ => Ok(payload),
        }
    }
}

impl OutboxRow {
    fn into_event(self) -> OutboxEvent {
        // Stored strings are the serde snake_case forms; round-trip them back.
        let entity_type = serde_json::from_value(serde_json::Value::String(self.entity_type))
            .unwrap_or(EntityType::Note);
        let operation = serde_json::from_value(serde_json::Value::String(self.operation))
            .unwrap_or(Operation::NoteUpdated);
        OutboxEvent {
            id: Uuid::parse_str(&self.id).unwrap_or_else(|_| Uuid::nil()),
            entity_type,
            entity_id: self.entity_id,
            operation,
            payload: serde_json::from_str(&self.payload_json).unwrap_or(serde_json::Value::Null),
            local_sequence: self.local_sequence,
        }
    }
}
