//! Notebook commands.

use kanso_types::NotebookId;
use kanso_types::sync::{EntityType, Operation};
use sqlx::FromRow;

use crate::db::{Engine, enqueue_outbox, now_ms};
use crate::error::{EngineError, Result};

#[derive(Debug, Clone, FromRow)]
pub struct Notebook {
    pub id: String,
    pub name: String,
    pub parent_id: Option<String>,
    pub sort_order: i64,
    pub created_at: i64,
    pub updated_at: i64,
    pub deleted_at: Option<i64>,
}

impl Engine {
    pub async fn create_notebook(&self, name: &str, parent_id: Option<&str>) -> Result<Notebook> {
        let id = NotebookId::new().0;
        let now = now_ms();

        let mut tx = self.pool.begin().await?;
        sqlx::query(
            "INSERT INTO notebooks (id, name, parent_id, sort_order, created_at, updated_at) \
             VALUES (?, ?, ?, 0, ?, ?)",
        )
        .bind(&id)
        .bind(name)
        .bind(parent_id)
        .bind(now)
        .bind(now)
        .execute(&mut *tx)
        .await?;

        enqueue_outbox(
            &mut tx,
            EntityType::Notebook,
            &id,
            Operation::NotebookCreated,
            serde_json::json!({ "name": name, "parent_id": parent_id }),
            now,
        )
        .await?;

        tx.commit().await?;

        Ok(Notebook {
            id,
            name: name.to_string(),
            parent_id: parent_id.map(str::to_string),
            sort_order: 0,
            created_at: now,
            updated_at: now,
            deleted_at: None,
        })
    }

    pub async fn list_notebooks(&self) -> Result<Vec<Notebook>> {
        let notebooks = sqlx::query_as::<_, Notebook>(
            "SELECT id, name, parent_id, sort_order, created_at, updated_at, deleted_at \
             FROM notebooks WHERE deleted_at IS NULL ORDER BY sort_order, name",
        )
        .fetch_all(&self.pool)
        .await?;
        Ok(notebooks)
    }

    pub async fn rename_notebook(&self, id: &str, name: &str) -> Result<()> {
        let result = sqlx::query("UPDATE notebooks SET name = ?, updated_at = ? WHERE id = ?")
            .bind(name)
            .bind(now_ms())
            .bind(id)
            .execute(&self.pool)
            .await?;
        if result.rows_affected() == 0 {
            return Err(EngineError::NotFound(id.to_string()));
        }
        Ok(())
    }
}
