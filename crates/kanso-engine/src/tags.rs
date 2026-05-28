//! Tag commands.

use kanso_types::TagId;
use kanso_types::payloads::{NoteTagPayload, TagPayload};
use kanso_types::sync::{EntityType, Operation};

use crate::db::{Engine, enqueue_outbox, now_ms};
use crate::error::Result;
use crate::models::Tag;

impl Engine {
    pub async fn create_tag(&self, name: &str) -> Result<Tag> {
        let id = TagId::new().0;
        let now = now_ms();
        let mut tx = self.pool.begin().await?;

        sqlx::query("INSERT INTO tags (id, name, created_at, updated_at) VALUES (?, ?, ?, ?)")
            .bind(&id)
            .bind(name)
            .bind(now)
            .bind(now)
            .execute(&mut *tx)
            .await?;

        let payload = TagPayload { name: name.to_string(), color: None, updated_at: now };
        enqueue_outbox(
            &mut tx,
            EntityType::Tag,
            &id,
            Operation::TagCreated,
            serde_json::to_value(&payload)?,
            now,
        )
        .await?;

        tx.commit().await?;
        Ok(Tag { id, name: name.to_string(), color: None })
    }

    pub async fn tag_note(&self, note_id: &str, tag_id: &str) -> Result<()> {
        let now = now_ms();
        let mut tx = self.pool.begin().await?;

        sqlx::query("INSERT OR IGNORE INTO note_tags (note_id, tag_id) VALUES (?, ?)")
            .bind(note_id)
            .bind(tag_id)
            .execute(&mut *tx)
            .await?;

        let payload = NoteTagPayload {
            note_id: note_id.to_string(),
            tag_id: tag_id.to_string(),
        };
        enqueue_outbox(
            &mut tx,
            EntityType::NoteTag,
            note_id,
            Operation::NoteTagged,
            serde_json::to_value(&payload)?,
            now,
        )
        .await?;

        tx.commit().await?;
        Ok(())
    }

    pub async fn untag_note(&self, note_id: &str, tag_id: &str) -> Result<()> {
        let now = now_ms();
        let mut tx = self.pool.begin().await?;
        sqlx::query("DELETE FROM note_tags WHERE note_id = ? AND tag_id = ?")
            .bind(note_id)
            .bind(tag_id)
            .execute(&mut *tx)
            .await?;
        let payload = NoteTagPayload { note_id: note_id.to_string(), tag_id: tag_id.to_string() };
        enqueue_outbox(
            &mut *tx,
            EntityType::NoteTag,
            note_id,
            Operation::NoteUntagged,
            serde_json::to_value(&payload)?,
            now,
        )
        .await?;
        tx.commit().await?;
        Ok(())
    }

    pub async fn list_tags(&self) -> Result<Vec<Tag>> {
        let tags = sqlx::query_as::<_, Tag>("SELECT id, name, color FROM tags ORDER BY name")
            .fetch_all(&self.pool)
            .await?;
        Ok(tags)
    }
}
