/// A file attached to a note. Content-addressed by `content_hash` for dedupe
/// and sync. The engine owns the record; the native layer owns the bytes.
#[derive(Debug, Clone)]
pub struct Attachment {
    pub id: String,
    pub note_id: String,
    pub filename: String,
    pub mime_type: String,
    pub size_bytes: i64,
    pub content_hash: String,
    pub local_path: Option<String>,
    pub remote_key: Option<String>,
    pub created_at: i64,
    pub updated_at: i64,
}

impl_sqlite_from_row!(Attachment {
    id,
    note_id,
    filename,
    mime_type,
    size_bytes,
    content_hash,
    local_path,
    remote_key,
    created_at,
    updated_at,
});

/// Parameters for registering a new attachment. The native layer supplies the
/// file facts; the engine assigns the id and timestamps.
#[derive(Debug, Clone)]
pub struct NewAttachment {
    pub filename: String,
    pub mime_type: String,
    pub size_bytes: i64,
    pub content_hash: String,
    pub local_path: Option<String>,
}
