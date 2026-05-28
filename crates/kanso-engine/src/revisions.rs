//! Revision history and conflict-copy access.
//!
//! The write path records revisions (pre-edit, remote-supersede, and conflict
//! copies); this is how callers read and restore them. Reading conflict copies
//! is what makes the "never discard text" guarantee actionable in the UI.

use crate::db::Engine;
use crate::error::{EngineError, Result};
use crate::models::Revision;

const REVISION_COLUMNS: &str = "id, note_id, body_markdown, reason, source, created_at";

impl Engine {
    /// All revisions for a note, newest first.
    pub async fn list_revisions(&self, note_id: &str) -> Result<Vec<Revision>> {
        // v7 ids tiebreak same-millisecond timestamps by creation order.
        let sql = format!(
            "SELECT {REVISION_COLUMNS} FROM revisions WHERE note_id = ? \
             ORDER BY created_at DESC, id DESC"
        );
        Ok(sqlx::query_as::<_, Revision>(&sql).bind(note_id).fetch_all(&self.pool).await?)
    }

    /// Conflict copies for a note — remote versions preserved when local was
    /// newer. The UI surfaces these so the user can reconcile.
    pub async fn list_conflicts(&self, note_id: &str) -> Result<Vec<Revision>> {
        let sql = format!(
            "SELECT {REVISION_COLUMNS} FROM revisions WHERE note_id = ? AND source = 'conflict' \
             ORDER BY created_at DESC, id DESC"
        );
        Ok(sqlx::query_as::<_, Revision>(&sql).bind(note_id).fetch_all(&self.pool).await?)
    }

    /// Restore a note's body to a prior revision (which itself snapshots the
    /// current body first, via the normal update path).
    pub async fn restore_revision(&self, note_id: &str, revision_id: &str) -> Result<()> {
        let row: Option<(String,)> =
            sqlx::query_as("SELECT body_markdown FROM revisions WHERE id = ? AND note_id = ?")
                .bind(revision_id)
                .bind(note_id)
                .fetch_optional(&self.pool)
                .await?;
        let Some((body,)) = row else {
            return Err(EngineError::NotFound(revision_id.to_string()));
        };
        self.update_note_body(note_id, &body).await
    }
}
