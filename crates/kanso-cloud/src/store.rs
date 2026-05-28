//! Event store boundary.
//!
//! The production target is Postgres (append-only event log + per-user
//! projection). [`MemoryStore`] implements the same contract in-process so the
//! sync protocol runs end-to-end without provisioning a database; a
//! `PostgresStore` slots in behind [`EventStore`] later without touching the
//! HTTP layer.

use std::collections::HashSet;
use std::sync::Mutex;

use kanso_types::{OutboxEvent, RemoteChange};
use uuid::Uuid;

/// Append-only, ordered, idempotent event log keyed by server sequence.
pub trait EventStore: Send + Sync {
    /// Append events, skipping any already-seen ids (idempotent). Returns the
    /// ids that are now durably stored and the new high-water sequence.
    fn append(&self, events: Vec<OutboxEvent>) -> (Vec<Uuid>, i64);

    /// All changes with `server_sequence > since`, capped at `limit`.
    fn since(&self, since: i64, limit: usize) -> Vec<RemoteChange>;

    /// Current high-water server sequence.
    fn high_water(&self) -> i64;
}

#[derive(Default)]
struct Inner {
    log: Vec<RemoteChange>,
    seen: HashSet<Uuid>,
    high_water: i64,
}

/// In-memory [`EventStore`]. Not durable — for local development and protocol
/// tests only.
#[derive(Default)]
pub struct MemoryStore {
    inner: Mutex<Inner>,
}

impl EventStore for MemoryStore {
    fn append(&self, events: Vec<OutboxEvent>) -> (Vec<Uuid>, i64) {
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
            inner.log.push(RemoteChange { server_sequence, event });
        }
        (accepted, inner.high_water)
    }

    fn since(&self, since: i64, limit: usize) -> Vec<RemoteChange> {
        let inner = self.inner.lock().expect("store mutex poisoned");
        inner
            .log
            .iter()
            .filter(|change| change.server_sequence > since)
            .take(limit)
            .cloned()
            .collect()
    }

    fn high_water(&self) -> i64 {
        self.inner.lock().expect("store mutex poisoned").high_water
    }
}
