//! Typed sync payloads.
//!
//! Each [`crate::sync::OutboxEvent`] carries an operation-specific payload as
//! JSON. These structs are that payload's shape — produced by the engine when
//! it enqueues a mutation, and consumed when a remote change is applied. Sharing
//! them here means the producer and consumer can never disagree on the wire
//! format. Timestamps are Unix millis and drive last-write-wins resolution.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotebookPayload {
    pub name: String,
    pub parent_id: Option<String>,
    pub created_at: Option<i64>,
    pub updated_at: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NoteCreatedPayload {
    pub notebook_id: String,
    pub title: String,
    /// Plaintext body, or empty when E2EE is on (see `body_cipher`).
    pub body_markdown: String,
    /// Ciphertext body when E2EE is on (`nonce || ciphertext+tag`). The server
    /// only ever sees this; the plaintext stays on-device.
    #[serde(default)]
    pub body_cipher: Option<Vec<u8>>,
    pub created_at: i64,
    pub updated_at: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NoteUpdatedPayload {
    pub title: String,
    /// Plaintext body, or empty when E2EE is on (see `body_cipher`).
    pub body_markdown: String,
    /// Ciphertext body when E2EE is on (`nonce || ciphertext+tag`).
    #[serde(default)]
    pub body_cipher: Option<Vec<u8>>,
    pub updated_at: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NoteMovedPayload {
    pub notebook_id: String,
    pub updated_at: i64,
}

/// Used for every soft delete (note, notebook, attachment, sketch, tag).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeletePayload {
    pub deleted_at: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TagPayload {
    pub name: String,
    pub color: Option<String>,
    pub updated_at: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NoteTagPayload {
    pub note_id: String,
    pub tag_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AttachmentPayload {
    pub note_id: String,
    pub filename: String,
    pub mime_type: String,
    pub size_bytes: i64,
    pub content_hash: String,
    pub updated_at: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SketchPayload {
    pub note_id: String,
    pub title: Option<String>,
    pub format_version: i64,
    /// Canonical CBOR blob, or empty when E2EE is on (see `data_cipher`).
    #[serde(default)]
    pub data_blob: Vec<u8>,
    /// Encrypted CBOR blob when E2EE is on (`nonce || ciphertext+tag`).
    #[serde(default)]
    pub data_cipher: Option<Vec<u8>>,
    pub updated_at: i64,
}
