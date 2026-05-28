//! Device-side sync loop.
//!
//! The engine never opens a socket. The native layer implements [`SyncTransport`]
//! (HTTP to Kanso Cloud); the engine drives the push → pull → apply cycle and
//! tracks per-device cursors in `sync_state`. The transport is expected not to
//! echo a device's own events back to it (the backend knows each event's origin).

use kanso_types::sync::{OutboxEvent, RemoteChange};

use crate::db::{Engine, now_ms};
use crate::error::{EngineError, Result};
use crate::models::{ApplyOutcome, SyncReport};

/// The network boundary, implemented by the native app or a test harness.
/// Errors are surfaced as strings and wrapped into [`EngineError::Transport`].
#[async_trait::async_trait]
pub trait SyncTransport: Send + Sync {
    /// Push local events; return the ids the backend durably accepted plus its
    /// new high-water sequence.
    async fn push(
        &self,
        device_id: &str,
        last_known_server_seq: i64,
        events: Vec<OutboxEvent>,
    ) -> std::result::Result<(Vec<uuid::Uuid>, i64), String>;

    /// Pull changes with `server_sequence > since_server_seq`, capped at `limit`.
    async fn pull(
        &self,
        device_id: &str,
        since_server_seq: i64,
        limit: i64,
    ) -> std::result::Result<Vec<RemoteChange>, String>;
}

const SYNC_BATCH: i64 = 500;

impl Engine {
    /// Run one full sync cycle for `device_id` against `transport`.
    pub async fn sync(&self, device_id: &str, transport: &dyn SyncTransport) -> Result<SyncReport> {
        self.ensure_sync_state(device_id).await?;
        let mut report = SyncReport::default();

        // --- Push phase: ship pending outbox events, ack the accepted ones. ---
        let pending = self.get_pending_sync_ops(SYNC_BATCH).await?;
        if !pending.is_empty() {
            let last_seq = self.last_pulled_server_seq(device_id).await?;
            let (accepted, _high_water) = transport
                .push(device_id, last_seq, pending)
                .await
                .map_err(EngineError::Transport)?;
            let accepted_ids: Vec<String> = accepted.iter().map(uuid::Uuid::to_string).collect();
            self.mark_sync_ops_acknowledged(&accepted_ids).await?;
            report.pushed = accepted_ids.len();
        }

        // --- Pull phase: apply remote changes, advance the cursor. ---
        let since = self.last_pulled_server_seq(device_id).await?;
        let changes = transport
            .pull(device_id, since, SYNC_BATCH)
            .await
            .map_err(EngineError::Transport)?;

        let mut max_seq = since;
        for change in &changes {
            match self.apply_remote_change(change).await? {
                ApplyOutcome::Applied => report.applied += 1,
                ApplyOutcome::Conflicted => report.conflicted += 1,
                ApplyOutcome::Deleted => report.deleted += 1,
                ApplyOutcome::Skipped => report.skipped += 1,
            }
            max_seq = max_seq.max(change.server_sequence);
        }
        if max_seq > since {
            self.set_last_pulled_server_seq(device_id, max_seq).await?;
        }

        Ok(report)
    }

    async fn ensure_sync_state(&self, device_id: &str) -> Result<()> {
        sqlx::query("INSERT OR IGNORE INTO sync_state (device_id, updated_at) VALUES (?, ?)")
            .bind(device_id)
            .bind(now_ms())
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    async fn last_pulled_server_seq(&self, device_id: &str) -> Result<i64> {
        let row: Option<(i64,)> =
            sqlx::query_as("SELECT last_pulled_server_sequence FROM sync_state WHERE device_id = ?")
                .bind(device_id)
                .fetch_optional(&self.pool)
                .await?;
        Ok(row.map(|(seq,)| seq).unwrap_or(0))
    }

    async fn set_last_pulled_server_seq(&self, device_id: &str, seq: i64) -> Result<()> {
        sqlx::query(
            "UPDATE sync_state SET last_pulled_server_sequence = ?, updated_at = ? WHERE device_id = ?",
        )
        .bind(seq)
        .bind(now_ms())
        .bind(device_id)
        .execute(&self.pool)
        .await?;
        Ok(())
    }
}
