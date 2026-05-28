//! Tag commands.

use kanso_types::TagId;
use sqlx::FromRow;

use crate::db::{Engine, now_ms};
use crate::error::Result;

#[derive(Debug, Clone, FromRow)]
pub struct Tag {
    pub id: String,
    pub name: String,
    pub color: Option<String>,
}

impl Engine {
    pub async fn create_tag(&self, name: &str) -> Result<Tag> {
        let id = TagId::new().0;
        let now = now_ms();
        sqlx::query("INSERT INTO tags (id, name, created_at, updated_at) VALUES (?, ?, ?, ?)")
            .bind(&id)
            .bind(name)
            .bind(now)
            .bind(now)
            .execute(&self.pool)
            .await?;
        Ok(Tag { id, name: name.to_string(), color: None })
    }

    pub async fn tag_note(&self, note_id: &str, tag_id: &str) -> Result<()> {
        sqlx::query("INSERT OR IGNORE INTO note_tags (note_id, tag_id) VALUES (?, ?)")
            .bind(note_id)
            .bind(tag_id)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    pub async fn list_tags(&self) -> Result<Vec<Tag>> {
        let tags = sqlx::query_as::<_, Tag>("SELECT id, name, color FROM tags ORDER BY name")
            .fetch_all(&self.pool)
            .await?;
        Ok(tags)
    }
}
