//! Engine handle, connection setup, and shared transaction helpers.

use std::str::FromStr;

use kanso_types::sync::{EntityType, Operation};
use sqlx::SqliteConnection;
use sqlx::SqlitePool;
use sqlx::sqlite::{
    SqliteConnectOptions, SqliteJournalMode, SqlitePoolOptions, SqliteSynchronous,
};

use crate::error::Result;

/// The Kanso engine — the single owner of the local SQLite database and all
/// product mutations. Cheap to clone (wraps a connection pool).
#[derive(Clone)]
pub struct Engine {
    pub(crate) pool: SqlitePool,
}

impl Engine {
    /// Open (creating if needed) a file-backed database and run migrations.
    pub async fn open(path: &str) -> Result<Self> {
        let options = SqliteConnectOptions::from_str(path)?
            .create_if_missing(true)
            .journal_mode(SqliteJournalMode::Wal)
            .synchronous(SqliteSynchronous::Normal)
            .foreign_keys(true)
            .busy_timeout(std::time::Duration::from_secs(5));

        let pool = SqlitePoolOptions::new()
            .max_connections(5)
            .connect_with(options)
            .await?;

        let engine = Self { pool };
        engine.migrate().await?;
        Ok(engine)
    }

    /// Open an ephemeral in-memory database (single connection). For tests.
    pub async fn open_in_memory() -> Result<Self> {
        let options = SqliteConnectOptions::from_str("sqlite::memory:")?.foreign_keys(true);

        // A `:memory:` database is per-connection, so the pool must hold exactly
        // one connection or different queries would see different databases.
        let pool = SqlitePoolOptions::new()
            .max_connections(1)
            .connect_with(options)
            .await?;

        let engine = Self { pool };
        engine.migrate().await?;
        Ok(engine)
    }

    async fn migrate(&self) -> Result<()> {
        sqlx::migrate!("./migrations").run(&self.pool).await?;
        Ok(())
    }
}

/// Current wall-clock time as Unix milliseconds (stored as `INTEGER`).
pub(crate) fn now_ms() -> i64 {
    chrono::Utc::now().timestamp_millis()
}

/// Serialize a `#[serde(rename_all = "snake_case")]` unit enum to its string
/// form (e.g. `Operation::NoteCreated` -> `"note_created"`).
pub(crate) fn enum_str<T: serde::Serialize>(value: &T) -> String {
    match serde_json::to_value(value) {
        Ok(serde_json::Value::String(s)) => s,
        other => other.map(|v| v.to_string()).unwrap_or_default(),
    }
}

/// Reserve the next monotonic local sequence number, inside the caller's
/// transaction.
pub(crate) async fn next_local_sequence(conn: &mut SqliteConnection) -> Result<i64> {
    let (seq,): (i64,) =
        sqlx::query_as("UPDATE local_sequence SET value = value + 1 WHERE id = 1 RETURNING value")
            .fetch_one(&mut *conn)
            .await?;
    Ok(seq)
}

/// Append a mutation to the sync outbox, inside the caller's transaction.
///
/// This is the only place that writes the outbox, so every product mutation
/// goes through it and nothing reaches the backend without a durable record.
pub(crate) async fn enqueue_outbox(
    conn: &mut SqliteConnection,
    entity_type: EntityType,
    entity_id: &str,
    operation: Operation,
    payload: serde_json::Value,
    now: i64,
) -> Result<()> {
    let local_sequence = next_local_sequence(&mut *conn).await?;
    sqlx::query(
        "INSERT INTO sync_outbox \
         (id, entity_type, entity_id, operation, payload_json, local_sequence, created_at) \
         VALUES (?, ?, ?, ?, ?, ?, ?)",
    )
    .bind(uuid::Uuid::now_v7().to_string())
    .bind(enum_str(&entity_type))
    .bind(entity_id)
    .bind(enum_str(&operation))
    .bind(serde_json::to_string(&payload)?)
    .bind(local_sequence)
    .bind(now)
    .execute(&mut *conn)
    .await?;
    Ok(())
}
