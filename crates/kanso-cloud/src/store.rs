//! Event store boundary.
//!
//! The production target is Postgres (append-only event log + per-user
//! projection). [`MemoryStore`] implements the same contract in-process so the
//! sync protocol runs end-to-end without provisioning a database.
//! [`PostgresStore`] slots in behind [`EventStore`] in production, selected at
//! runtime by the `DATABASE_URL` env var.

use std::collections::HashSet;
use std::sync::Mutex;

use async_trait::async_trait;
use kanso_types::{OutboxEvent, RemoteChange};
use uuid::Uuid;

// ─── Trait ───────────────────────────────────────────────────────────────────

/// Append-only, ordered, idempotent event log keyed by server sequence.
///
/// Implementations are origin-aware: every event records the device that
/// pushed it so that `since` can exclude a device's own events (preventing
/// echo-back during pull).
#[async_trait]
pub trait EventStore: Send + Sync {
    /// Append events from `device_id`, dedup by event id (idempotent).
    /// Returns the ids that are now durably stored and the new high-water
    /// sequence.
    async fn append(&self, device_id: &str, events: Vec<OutboxEvent>) -> (Vec<Uuid>, i64);

    /// All changes with `server_sequence > since`, capped at `limit`,
    /// EXCLUDING events that originated from `device_id` (no echo-back).
    async fn since(&self, device_id: &str, since: i64, limit: usize) -> Vec<RemoteChange>;

    /// Current high-water server sequence.
    async fn high_water(&self) -> i64;
}

// ─── MemoryStore ─────────────────────────────────────────────────────────────

/// An event log entry that also records where the event came from.
#[derive(Clone)]
struct LogEntry {
    origin_device_id: String,
    change: RemoteChange,
}

#[derive(Default)]
struct Inner {
    log: Vec<LogEntry>,
    seen: HashSet<Uuid>,
    high_water: i64,
}

/// In-memory [`EventStore`]. Not durable — for local development and protocol
/// tests only.
#[derive(Default)]
pub struct MemoryStore {
    inner: Mutex<Inner>,
}

#[async_trait]
impl EventStore for MemoryStore {
    async fn append(&self, device_id: &str, events: Vec<OutboxEvent>) -> (Vec<Uuid>, i64) {
        // std::sync::Mutex is fine here: the critical section has no awaits.
        let mut inner = self.inner.lock().expect("store mutex poisoned");
        let mut accepted = Vec::with_capacity(events.len());
        for event in events {
            // Idempotent: re-pushing a known event is accepted but not re-logged.
            if inner.seen.contains(&event.id) {
                accepted.push(event.id);
                continue;
            }
            inner.high_water += 1;
            let server_sequence = inner.high_water;
            inner.seen.insert(event.id);
            accepted.push(event.id);
            inner.log.push(LogEntry {
                origin_device_id: device_id.to_string(),
                change: RemoteChange { server_sequence, event },
            });
        }
        (accepted, inner.high_water)
    }

    async fn since(&self, device_id: &str, since: i64, limit: usize) -> Vec<RemoteChange> {
        let inner = self.inner.lock().expect("store mutex poisoned");
        inner
            .log
            .iter()
            .filter(|entry| {
                entry.change.server_sequence > since && entry.origin_device_id != device_id
            })
            .take(limit)
            .map(|entry| entry.change.clone())
            .collect()
    }

    async fn high_water(&self) -> i64 {
        self.inner.lock().expect("store mutex poisoned").high_water
    }
}

// ─── PostgresStore ───────────────────────────────────────────────────────────

/// Postgres-backed [`EventStore`]. Durable, suitable for production.
///
/// Migrations are run automatically on startup via `sqlx::migrate!`.
pub struct PostgresStore {
    pool: sqlx::PgPool,
}

impl PostgresStore {
    pub async fn connect(database_url: &str) -> Result<Self, sqlx::Error> {
        let pool = sqlx::postgres::PgPoolOptions::new()
            .max_connections(5)
            .connect(database_url)
            .await?;
        sqlx::migrate!("./migrations")
            .run(&pool)
            .await
            .map_err(|e| sqlx::Error::Migrate(Box::new(e)))?;
        Ok(Self { pool })
    }
}

/// Serialize an enum variant to its snake_case string form.
///
/// Both [`EntityType`] and [`Operation`] are `#[serde(rename_all="snake_case")]`
/// so serialising to a JSON Value and extracting the string is the simplest
/// canonical approach that stays in sync with the serde annotation.
fn enum_to_str<T: serde::Serialize>(value: &T) -> String {
    serde_json::to_value(value)
        .ok()
        .and_then(|v| v.as_str().map(ToOwned::to_owned))
        .unwrap_or_default()
}

/// Deserialize an enum variant from its snake_case string form.
fn str_to_enum<T: serde::de::DeserializeOwned>(s: String) -> Result<T, serde_json::Error> {
    serde_json::from_value(serde_json::Value::String(s))
}

#[async_trait]
impl EventStore for PostgresStore {
    async fn append(&self, device_id: &str, events: Vec<OutboxEvent>) -> (Vec<Uuid>, i64) {
        let mut accepted = Vec::with_capacity(events.len());

        for event in events {
            let entity_type_str = enum_to_str(&event.entity_type);
            let operation_str = enum_to_str(&event.operation);

            // ON CONFLICT (event_id) DO NOTHING makes this idempotent — the
            // event id is the client-generated idempotency key.
            let result = sqlx::query(
                r#"
                INSERT INTO events
                    (event_id, origin_device_id, entity_type, entity_id, operation, payload, local_sequence)
                VALUES ($1, $2, $3, $4, $5, $6, $7)
                ON CONFLICT (event_id) DO NOTHING
                "#,
            )
            .bind(event.id)
            .bind(device_id)
            .bind(&entity_type_str)
            .bind(&event.entity_id)
            .bind(&operation_str)
            .bind(sqlx::types::Json(&event.payload))
            .bind(event.local_sequence)
            .execute(&self.pool)
            .await;

            match result {
                Ok(_) => accepted.push(event.id),
                Err(e) => {
                    log::error!("append: failed to insert event {}: {e}", event.id);
                    // Still mark as accepted idempotently — the event may already
                    // be present (race on the DO NOTHING path returns 0 rows
                    // but no error with execute; this branch handles true errors).
                }
            }
        }

        let hw = self.high_water().await;
        (accepted, hw)
    }

    async fn since(&self, device_id: &str, since: i64, limit: usize) -> Vec<RemoteChange> {
        let rows = sqlx::query(
            r#"
            SELECT server_sequence, event_id, entity_type, entity_id,
                   operation, payload, local_sequence
            FROM   events
            WHERE  server_sequence > $1
              AND  origin_device_id <> $2
            ORDER  BY server_sequence
            LIMIT  $3
            "#,
        )
        .bind(since)
        .bind(device_id)
        .bind(limit as i64)
        .fetch_all(&self.pool)
        .await
        .unwrap_or_else(|e| {
            log::error!("since: query failed: {e}");
            Vec::new()
        });

        rows.into_iter()
            .filter_map(|row| {
                use sqlx::Row;
                let server_sequence: i64 = row.try_get("server_sequence").ok()?;
                let event_id: Uuid = row.try_get("event_id").ok()?;
                let entity_type_str: String = row.try_get("entity_type").ok()?;
                let entity_id: String = row.try_get("entity_id").ok()?;
                let operation_str: String = row.try_get("operation").ok()?;
                let payload: serde_json::Value =
                    row.try_get::<sqlx::types::Json<serde_json::Value>, _>("payload")
                        .ok()
                        .map(|j| j.0)?;
                let local_sequence: i64 = row.try_get("local_sequence").ok()?;

                let entity_type = str_to_enum(entity_type_str)
                    .map_err(|e| log::error!("since: bad entity_type: {e}"))
                    .ok()?;
                let operation = str_to_enum(operation_str)
                    .map_err(|e| log::error!("since: bad operation: {e}"))
                    .ok()?;

                Some(RemoteChange {
                    server_sequence,
                    event: OutboxEvent {
                        id: event_id,
                        entity_type,
                        entity_id,
                        operation,
                        payload,
                        local_sequence,
                    },
                })
            })
            .collect()
    }

    async fn high_water(&self) -> i64 {
        sqlx::query("SELECT COALESCE(MAX(server_sequence), 0)::BIGINT AS hw FROM events")
            .fetch_one(&self.pool)
            .await
            .and_then(|row| {
                use sqlx::Row;
                row.try_get::<i64, _>("hw")
            })
            .unwrap_or_else(|e| {
                log::error!("high_water: query failed: {e}");
                0
            })
    }
}
