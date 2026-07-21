/// A first-party sketch block. `data_blob` is the canonical CBOR document from
/// `kanso-ink`; the Markdown body references it via `![[sketch:id]]`.
#[derive(Debug, Clone)]
pub struct Sketch {
    pub id: String,
    pub note_id: String,
    pub title: Option<String>,
    pub format_version: i64,
    pub data_blob: Vec<u8>,
    pub preview_attachment_id: Option<String>,
    pub created_at: i64,
    pub updated_at: i64,
}

impl_sqlite_from_row!(Sketch {
    id,
    note_id,
    title,
    format_version,
    data_blob,
    preview_attachment_id,
    created_at,
    updated_at,
});
