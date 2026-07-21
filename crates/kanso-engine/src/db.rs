//! Engine handle, connection setup, and shared transaction helpers.

use std::str::FromStr;
use std::time::Instant;

use kanso_types::sync::{EntityType, Operation};
use sqlx::SqliteConnection;
use sqlx::SqlitePool;
use sqlx::sqlite::{SqliteConnectOptions, SqliteJournalMode, SqlitePoolOptions, SqliteSynchronous};

use crate::error::Result;

/// The Kanso engine — the single owner of the local SQLite database and all
/// product mutations. Cheap to clone (wraps a connection pool).
#[derive(Clone)]
pub struct Engine {
    pub(crate) pool: SqlitePool,
    /// Optional client-side E2EE key. When set, note bodies are encrypted in
    /// outbound sync payloads and decrypted on apply; local storage stays
    /// plaintext so FTS keeps working.
    pub(crate) enc: Option<std::sync::Arc<[u8; 32]>>,
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

        let engine = Self { pool, enc: None };
        engine.migrate().await?;
        engine.rebuild_note_indexes().await?;
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

        let engine = Self { pool, enc: None };
        engine.migrate().await?;
        engine.rebuild_note_indexes().await?;
        Ok(engine)
    }

    async fn migrate(&self) -> Result<()> {
        sqlx::query(
            "CREATE TABLE IF NOT EXISTS _sqlx_migrations (
                version BIGINT PRIMARY KEY,
                description TEXT NOT NULL,
                installed_on TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
                success BOOLEAN NOT NULL,
                checksum BLOB NOT NULL,
                execution_time BIGINT NOT NULL
            )",
        )
        .execute(&self.pool)
        .await?;

        for migration in ENGINE_MIGRATIONS {
            let applied: Option<(i64,)> =
                sqlx::query_as("SELECT success FROM _sqlx_migrations WHERE version = ?")
                    .bind(migration.version)
                    .fetch_optional(&self.pool)
                    .await?;
            if applied.is_some_and(|(success,)| success != 0) {
                continue;
            }

            let started = Instant::now();
            let mut tx = self.pool.begin().await?;
            sqlx::raw_sql(migration.sql).execute(&mut *tx).await?;
            let elapsed_micros = i64::try_from(started.elapsed().as_micros()).unwrap_or(i64::MAX);
            sqlx::query(
                "INSERT OR REPLACE INTO _sqlx_migrations
                    (version, description, success, checksum, execution_time)
                 VALUES (?, ?, TRUE, ?, ?)",
            )
            .bind(migration.version)
            .bind(migration.description)
            .bind(migration.sql.as_bytes())
            .bind(elapsed_micros)
            .execute(&mut *tx)
            .await?;
            tx.commit().await?;
        }
        Ok(())
    }
}

struct EngineMigration {
    version: i64,
    description: &'static str,
    sql: &'static str,
}

const ENGINE_MIGRATIONS: &[EngineMigration] = &[
    EngineMigration {
        version: 1,
        description: "init",
        sql: include_str!("../migrations/0001_init.sql"),
    },
    EngineMigration {
        version: 2,
        description: "skills",
        sql: include_str!("../migrations/0002_skills.sql"),
    },
    EngineMigration {
        version: 3,
        description: "mcp",
        sql: include_str!("../migrations/0003_mcp.sql"),
    },
    EngineMigration {
        version: 4,
        description: "sharing",
        sql: include_str!("../migrations/0004_sharing.sql"),
    },
];

impl Engine {
    /// Enable end-to-end encryption with a 32-byte key (derive it via
    /// `kanso_crypto::derive_key`). Note bodies in sync payloads are encrypted;
    /// the local database stays plaintext.
    pub fn with_encryption_key(mut self, key: [u8; 32]) -> Self {
        self.enc = Some(std::sync::Arc::new(key));
        self
    }

    /// Build the `(body_markdown, body_cipher)` pair for a sync payload: with
    /// encryption on, the plaintext field is emptied and the ciphertext set.
    pub(crate) fn encrypt_body(&self, body: &str) -> Result<(String, Option<Vec<u8>>)> {
        match &self.enc {
            Some(key) => {
                let ciphertext = kanso_crypto::encrypt(key, body.as_bytes())
                    .map_err(|e| crate::error::EngineError::Decode(e.to_string()))?;
                Ok((String::new(), Some(ciphertext)))
            }
            None => Ok((body.to_string(), None)),
        }
    }

    /// Resolve a note body from a sync payload, decrypting if it carries
    /// ciphertext. Errors if encrypted but no key is configured.
    pub(crate) fn resolve_body(
        &self,
        body_markdown: &str,
        body_cipher: &Option<Vec<u8>>,
    ) -> Result<String> {
        match body_cipher {
            Some(ciphertext) => match &self.enc {
                Some(key) => {
                    let plaintext = kanso_crypto::decrypt(key, ciphertext)
                        .map_err(|e| crate::error::EngineError::Decode(e.to_string()))?;
                    Ok(String::from_utf8_lossy(&plaintext).into_owned())
                }
                None => Err(crate::error::EngineError::Decode(
                    "encrypted note body but no decryption key is set".to_string(),
                )),
            },
            None => Ok(body_markdown.to_string()),
        }
    }

    /// Byte-oriented counterpart of [`Engine::encrypt_body`], for binary blobs
    /// such as a sketch's CBOR document.
    pub(crate) fn encrypt_blob(&self, bytes: &[u8]) -> Result<(Vec<u8>, Option<Vec<u8>>)> {
        match &self.enc {
            Some(key) => {
                let ciphertext = kanso_crypto::encrypt(key, bytes)
                    .map_err(|e| crate::error::EngineError::Decode(e.to_string()))?;
                Ok((Vec::new(), Some(ciphertext)))
            }
            None => Ok((bytes.to_vec(), None)),
        }
    }

    /// Byte-oriented counterpart of [`Engine::resolve_body`].
    pub(crate) fn resolve_blob(
        &self,
        data_blob: &[u8],
        data_cipher: &Option<Vec<u8>>,
    ) -> Result<Vec<u8>> {
        match data_cipher {
            Some(ciphertext) => match &self.enc {
                Some(key) => kanso_crypto::decrypt(key, ciphertext)
                    .map_err(|e| crate::error::EngineError::Decode(e.to_string())),
                None => Err(crate::error::EngineError::Decode(
                    "encrypted sketch but no decryption key is set".to_string(),
                )),
            },
            None => Ok(data_blob.to_vec()),
        }
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

/// Record (or refresh) a tombstone so a deleted entity does not resurrect from
/// another device. Inside the caller's transaction.
pub(crate) async fn insert_tombstone(
    conn: &mut SqliteConnection,
    entity_type: EntityType,
    entity_id: &str,
    now: i64,
) -> Result<()> {
    sqlx::query(
        "INSERT INTO tombstones (entity_type, entity_id, deleted_at) VALUES (?, ?, ?) \
         ON CONFLICT (entity_type, entity_id) DO UPDATE SET deleted_at = excluded.deleted_at",
    )
    .bind(enum_str(&entity_type))
    .bind(entity_id)
    .bind(now)
    .execute(&mut *conn)
    .await?;
    Ok(())
}

/// True if a tombstone for the entity exists with `deleted_at` at or after the
/// given timestamp — i.e. a delete that should suppress an older upsert.
pub(crate) async fn tombstoned_after(
    conn: &mut SqliteConnection,
    entity_type: EntityType,
    entity_id: &str,
    at_or_after: i64,
) -> Result<bool> {
    let row: Option<(i64,)> =
        sqlx::query_as("SELECT deleted_at FROM tombstones WHERE entity_type = ? AND entity_id = ?")
            .bind(enum_str(&entity_type))
            .bind(entity_id)
            .fetch_optional(&mut *conn)
            .await?;
    Ok(matches!(row, Some((deleted_at,)) if deleted_at >= at_or_after))
}
