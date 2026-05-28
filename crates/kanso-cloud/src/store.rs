//! Event store boundary — per-user, ordered, idempotent, origin-aware.
//!
//! [`MemoryStore`] is the in-process implementation (dev/tests). [`PostgresStore`]
//! is the durable production store, selected at runtime by `DATABASE_URL`. Both
//! scope every operation to a `user_id` so accounts never see each other's data,
//! and exclude a device's own events on pull (no echo-back).

use std::collections::{HashMap, HashSet};
use std::sync::Mutex;

use async_trait::async_trait;
use kanso_types::{OutboxEvent, RemoteChange};
use uuid::Uuid;

#[async_trait]
pub trait EventStore: Send + Sync {
    /// Append `events` from `device_id` under `user_id` (idempotent by event id).
    /// Returns accepted ids and the user's new high-water sequence.
    async fn append(
        &self,
        user_id: &str,
        device_id: &str,
        events: Vec<OutboxEvent>,
    ) -> (Vec<Uuid>, i64);

    /// The user's changes with `server_sequence > since`, capped at `limit`,
    /// EXCLUDING events that originated from `device_id`.
    async fn since(
        &self,
        user_id: &str,
        device_id: &str,
        since: i64,
        limit: usize,
    ) -> Vec<RemoteChange>;

    /// The user's current high-water server sequence.
    async fn high_water(&self, user_id: &str) -> i64;
}

// ─── MemoryStore ─────────────────────────────────────────────────────────────

#[derive(Clone)]
struct LogEntry {
    origin_device_id: String,
    change: RemoteChange,
}

#[derive(Default)]
struct UserLog {
    log: Vec<LogEntry>,
    seen: HashSet<Uuid>,
    high_water: i64,
}

/// In-memory [`EventStore`], keyed per user. Not durable.
#[derive(Default)]
pub struct MemoryStore {
    users: Mutex<HashMap<String, UserLog>>,
}

#[async_trait]
impl EventStore for MemoryStore {
    async fn append(
        &self,
        user_id: &str,
        device_id: &str,
        events: Vec<OutboxEvent>,
    ) -> (Vec<Uuid>, i64) {
        let mut users = self.users.lock().expect("store mutex poisoned");
        let user = users.entry(user_id.to_string()).or_default();

        let mut accepted = Vec::with_capacity(events.len());
        for event in events {
            if user.seen.contains(&event.id) {
                accepted.push(event.id); // idempotent
                continue;
            }
            user.high_water += 1;
            let server_sequence = user.high_water;
            user.seen.insert(event.id);
            accepted.push(event.id);
            user.log.push(LogEntry {
                origin_device_id: device_id.to_string(),
                change: RemoteChange { server_sequence, event },
            });
        }
        (accepted, user.high_water)
    }

    async fn since(
        &self,
        user_id: &str,
        device_id: &str,
        since: i64,
        limit: usize,
    ) -> Vec<RemoteChange> {
        let users = self.users.lock().expect("store mutex poisoned");
        let Some(user) = users.get(user_id) else {
            return Vec::new();
        };
        user.log
            .iter()
            .filter(|e| e.change.server_sequence > since && e.origin_device_id != device_id)
            .take(limit)
            .map(|e| e.change.clone())
            .collect()
    }

    async fn high_water(&self, user_id: &str) -> i64 {
        self.users
            .lock()
            .expect("store mutex poisoned")
            .get(user_id)
            .map(|u| u.high_water)
            .unwrap_or(0)
    }
}

// ─── PostgresStore ───────────────────────────────────────────────────────────

/// Postgres-backed [`EventStore`]. Durable; migrations run on startup.
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

/// Serialize a `#[serde(rename_all="snake_case")]` enum to its string form.
fn enum_to_str<T: serde::Serialize>(value: &T) -> String {
    serde_json::to_value(value)
        .ok()
        .and_then(|v| v.as_str().map(ToOwned::to_owned))
        .unwrap_or_default()
}

fn str_to_enum<T: serde::de::DeserializeOwned>(s: String) -> Result<T, serde_json::Error> {
    serde_json::from_value(serde_json::Value::String(s))
}

#[async_trait]
impl EventStore for PostgresStore {
    async fn append(
        &self,
        user_id: &str,
        device_id: &str,
        events: Vec<OutboxEvent>,
    ) -> (Vec<Uuid>, i64) {
        let mut accepted = Vec::with_capacity(events.len());
        for event in events {
            let entity_type = enum_to_str(&event.entity_type);
            let operation = enum_to_str(&event.operation);
            let result = sqlx::query(
                "INSERT INTO events \
                 (user_id, event_id, origin_device_id, entity_type, entity_id, operation, payload, local_sequence) \
                 VALUES ($1, $2, $3, $4, $5, $6, $7, $8) \
                 ON CONFLICT (event_id) DO NOTHING",
            )
            .bind(user_id)
            .bind(event.id)
            .bind(device_id)
            .bind(&entity_type)
            .bind(&event.entity_id)
            .bind(&operation)
            .bind(sqlx::types::Json(&event.payload))
            .bind(event.local_sequence)
            .execute(&self.pool)
            .await;
            match result {
                Ok(_) => accepted.push(event.id),
                Err(e) => log::error!("append: insert event {} failed: {e}", event.id),
            }
        }
        let hw = self.high_water(user_id).await;
        (accepted, hw)
    }

    async fn since(
        &self,
        user_id: &str,
        device_id: &str,
        since: i64,
        limit: usize,
    ) -> Vec<RemoteChange> {
        let rows = sqlx::query(
            "SELECT server_sequence, event_id, entity_type, entity_id, operation, payload, local_sequence \
             FROM events \
             WHERE user_id = $1 AND server_sequence > $2 AND origin_device_id <> $3 \
             ORDER BY server_sequence LIMIT $4",
        )
        .bind(user_id)
        .bind(since)
        .bind(device_id)
        .bind(limit as i64)
        .fetch_all(&self.pool)
        .await
        .unwrap_or_else(|e| {
            log::error!("since: query failed: {e}");
            Vec::new()
        });

        use sqlx::Row;
        rows.into_iter()
            .filter_map(|row| {
                let server_sequence: i64 = row.try_get("server_sequence").ok()?;
                let event = OutboxEvent {
                    id: row.try_get("event_id").ok()?,
                    entity_type: str_to_enum(row.try_get("entity_type").ok()?).ok()?,
                    entity_id: row.try_get("entity_id").ok()?,
                    operation: str_to_enum(row.try_get("operation").ok()?).ok()?,
                    payload: row
                        .try_get::<sqlx::types::Json<serde_json::Value>, _>("payload")
                        .ok()?
                        .0,
                    local_sequence: row.try_get("local_sequence").ok()?,
                };
                Some(RemoteChange { server_sequence, event })
            })
            .collect()
    }

    async fn high_water(&self, user_id: &str) -> i64 {
        use sqlx::Row;
        sqlx::query("SELECT COALESCE(MAX(server_sequence), 0) AS hw FROM events WHERE user_id = $1")
            .bind(user_id)
            .fetch_one(&self.pool)
            .await
            .and_then(|row| row.try_get::<i64, _>("hw"))
            .unwrap_or(0)
    }
}
