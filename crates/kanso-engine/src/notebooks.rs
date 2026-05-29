//! Notebook commands.

use kanso_types::NotebookId;
use kanso_types::payloads::{DeletePayload, NotebookPayload};
use kanso_types::sync::{EntityType, Operation};

use crate::db::{Engine, enqueue_outbox, insert_tombstone, now_ms};
use crate::error::{EngineError, Result};
use crate::models::Notebook;

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

        let payload = NotebookPayload {
            name: name.to_string(),
            parent_id: parent_id.map(str::to_string),
            created_at: Some(now),
            updated_at: now,
        };
        enqueue_outbox(
            &mut tx,
            EntityType::Notebook,
            &id,
            Operation::NotebookCreated,
            serde_json::to_value(&payload)?,
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
        let now = now_ms();
        let mut tx = self.pool.begin().await?;

        let row: Option<(Option<String>,)> =
            sqlx::query_as("SELECT parent_id FROM notebooks WHERE id = ? AND deleted_at IS NULL")
                .bind(id)
                .fetch_optional(&self.pool)
                .await?;
        let (parent_id,) = row.ok_or_else(|| EngineError::NotFound(id.to_string()))?;

        sqlx::query("UPDATE notebooks SET name = ?, updated_at = ? WHERE id = ?")
            .bind(name)
            .bind(now)
            .bind(id)
            .execute(&mut *tx)
            .await?;

        let payload = NotebookPayload {
            name: name.to_string(),
            parent_id,
            created_at: None,
            updated_at: now,
        };
        enqueue_outbox(
            &mut tx,
            EntityType::Notebook,
            id,
            Operation::NotebookUpdated,
            serde_json::to_value(&payload)?,
            now,
        )
        .await?;

        tx.commit().await?;
        Ok(())
    }

    pub async fn delete_notebook(&self, id: &str) -> Result<()> {
        let now = now_ms();
        let mut tx = self.pool.begin().await?;

        let result = sqlx::query(
            "UPDATE notebooks SET deleted_at = ?, updated_at = ? WHERE id = ? AND deleted_at IS NULL",
        )
        .bind(now)
        .bind(now)
        .bind(id)
        .execute(&mut *tx)
        .await?;

        if result.rows_affected() == 0 {
            return Err(EngineError::NotFound(id.to_string()));
        }

        insert_tombstone(&mut tx, EntityType::Notebook, id, now).await?;

        let payload = DeletePayload { deleted_at: now };
        enqueue_outbox(
            &mut tx,
            EntityType::Notebook,
            id,
            Operation::NotebookDeleted,
            serde_json::to_value(&payload)?,
            now,
        )
        .await?;

        tx.commit().await?;
        Ok(())
    }

    /// Reparent a notebook (pass `None` to move it to the root).
    pub async fn move_notebook(&self, id: &str, parent_id: Option<&str>) -> Result<()> {
        let now = now_ms();
        let mut tx = self.pool.begin().await?;

        let row: Option<(String,)> =
            sqlx::query_as("SELECT name FROM notebooks WHERE id = ? AND deleted_at IS NULL")
                .bind(id)
                .fetch_optional(&mut *tx)
                .await?;
        let (name,) = row.ok_or_else(|| EngineError::NotFound(id.to_string()))?;

        sqlx::query("UPDATE notebooks SET parent_id = ?, updated_at = ? WHERE id = ?")
            .bind(parent_id)
            .bind(now)
            .bind(id)
            .execute(&mut *tx)
            .await?;

        let payload = NotebookPayload {
            name,
            parent_id: parent_id.map(str::to_string),
            created_at: None,
            updated_at: now,
        };
        enqueue_outbox(
            &mut tx,
            EntityType::Notebook,
            id,
            Operation::NotebookUpdated,
            serde_json::to_value(&payload)?,
            now,
        )
        .await?;

        tx.commit().await?;
        Ok(())
    }

    pub async fn list_child_notebooks(&self, parent_id: &str) -> Result<Vec<Notebook>> {
        Ok(sqlx::query_as::<_, Notebook>(
            "SELECT id, name, parent_id, sort_order, created_at, updated_at, deleted_at \
             FROM notebooks WHERE parent_id = ? AND deleted_at IS NULL ORDER BY sort_order, name",
        )
        .bind(parent_id)
        .fetch_all(&self.pool)
        .await?)
    }

    pub async fn list_root_notebooks(&self) -> Result<Vec<Notebook>> {
        Ok(sqlx::query_as::<_, Notebook>(
            "SELECT id, name, parent_id, sort_order, created_at, updated_at, deleted_at \
             FROM notebooks WHERE parent_id IS NULL AND deleted_at IS NULL ORDER BY sort_order, name",
        )
        .fetch_all(&self.pool)
        .await?)
    }
}
