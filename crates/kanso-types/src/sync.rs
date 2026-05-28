//! Sync wire types — the contract between the on-device outbox and Kanso Cloud.
//!
//! The client records mutations as [`OutboxEvent`]s with a per-device monotonic
//! `local_sequence`. The server appends them to a per-user log, assigns an
//! authoritative `server_sequence`, and serves them back as [`RemoteChange`]s.

use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// The kind of entity a sync event refers to.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EntityType {
    Notebook,
    Note,
    Tag,
    NoteTag,
    Attachment,
    Sketch,
}

/// A mutation operation in the change log.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Operation {
    NotebookCreated,
    NotebookUpdated,
    NotebookDeleted,
    NoteCreated,
    NoteUpdated,
    NoteDeleted,
    NoteMoved,
    TagCreated,
    TagUpdated,
    TagDeleted,
    NoteTagged,
    NoteUntagged,
    AttachmentAdded,
    AttachmentDeleted,
    SketchCreated,
    SketchUpdated,
    SketchDeleted,
}

/// A single local change awaiting (or completed) sync.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OutboxEvent {
    /// Stable client-generated id — the idempotency key. The server dedupes on
    /// this, so retries are safe.
    pub id: Uuid,
    pub entity_type: EntityType,
    pub entity_id: String,
    pub operation: Operation,
    pub payload: serde_json::Value,
    /// Monotonic per-device sequence.
    pub local_sequence: i64,
}

/// A change as served by the backend, carrying the authoritative ordering.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RemoteChange {
    pub server_sequence: i64,
    #[serde(flatten)]
    pub event: OutboxEvent,
}

/// `POST /v1/sync/push` request body.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PushRequest {
    pub device_id: String,
    pub last_known_server_seq: i64,
    pub events: Vec<OutboxEvent>,
}

/// `POST /v1/sync/push` response body.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PushResponse {
    pub accepted_ids: Vec<Uuid>,
    pub server_high_water: i64,
}

/// `GET /v1/sync/pull` response body.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PullResponse {
    pub changes: Vec<RemoteChange>,
    pub server_high_water: i64,
}
