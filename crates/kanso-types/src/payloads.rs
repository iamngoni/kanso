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
    pub body_markdown: String,
    pub created_at: i64,
    pub updated_at: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NoteUpdatedPayload {
    pub title: String,
    pub body_markdown: String,
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
    /// Canonical CBOR blob. (Serialized as a JSON byte array for now; a real
    /// backend would base64/binary-frame this.)
    #[serde(default)]
    pub data_blob: Vec<u8>,
    pub updated_at: i64,
}
