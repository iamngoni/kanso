//! Stored documents and their API representation.

use base64::Engine as _;
use kanso_core::DocType;
use serde_json::{Map, Value, json};

/// A stored document: its JSON fields plus `_id` and `_rev`.
#[derive(Clone, Debug, PartialEq)]
pub struct Doc {
    pub id: String,
    pub doc_type: DocType,
    pub rev: String,
    pub seq: i64,
    pub deleted: bool,
    /// Fields without `_id`, `_rev` and derived values.
    pub fields: Map<String, Value>,
    /// A note in a locked notebook (or one inside it): served as a locked
    /// note is, sealed or not.
    pub in_locked_notebook: bool,
}

/// An attachment's bytes as stored in the `blobs` table.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Attachment {
    pub md5: [u8; 16],
    pub data: Vec<u8>,
}

impl Attachment {
    pub fn new(data: Vec<u8>) -> Self {
        use md5::{Digest, Md5};
        Attachment {
            md5: Md5::digest(&data).into(),
            data,
        }
    }

    /// PouchDB's digest form: `md5-<base64 of the raw MD5>`.
    pub fn digest(&self) -> String {
        format!(
            "md5-{}",
            base64::engine::general_purpose::STANDARD.encode(self.md5)
        )
    }

    pub fn md5_hex(&self) -> String {
        self.md5.iter().map(|b| format!("{b:02x}")).collect()
    }
}

impl Doc {
    /// The document as the HTTP API returns it: `_id` and `_rev` first, then
    /// the stored fields, then `extra` (derived values such as `count`).
    pub fn to_api(&self, extra: Map<String, Value>) -> Value {
        let mut out = Map::new();
        out.insert("_id".into(), Value::String(self.id.clone()));
        out.insert("_rev".into(), Value::String(self.rev.clone()));
        out.extend(self.fields.clone());
        out.extend(extra);
        Value::Object(out)
    }

    /// [`Doc::to_api`] as the local API and MCP serve it to clients: a locked
    /// note goes out with an empty body and without its sealed text, so a
    /// client can neither read it nor guess the lock password offline. A
    /// note in a locked notebook goes out the same way, sealed or not, and a
    /// locked notebook without its lock.
    pub fn to_served(&self, extra: Map<String, Value>) -> Value {
        let mut out = self.to_api(extra);
        if let Some(fields) = out.as_object_mut() {
            if self.doc_type == DocType::Note {
                fields.remove(kanso_core::model::KANSO_FILES_FIELD);
            }
            let sealed = fields.remove(kanso_core::model::LOCK_FIELD).is_some();
            if self.doc_type == DocType::Note && (sealed || self.in_locked_notebook) {
                fields.insert("body".into(), Value::String(String::new()));
                fields.insert("numOfTasks".into(), 0.into());
                fields.insert("numOfCheckedTasks".into(), 0.into());
            }
            if self.doc_type == DocType::File {
                // Sealing and deliberate-unseal markers are app-only.
                fields.remove(kanso_core::model::FILE_LOCK_FIELD);
                fields.remove(kanso_core::model::UNSEALED_FIELD);
            }
        }
        out
    }

    /// The API form of a file document. With `data`, the attachment carries its
    /// base64 bytes; otherwise it is a stub, as in CouchDB.
    pub fn file_to_api(&self, content_type: &str, attachment: &Attachment, data: bool) -> Value {
        let locked = self.fields.contains_key(kanso_core::model::FILE_LOCK_FIELD);
        let mut index = json!({
            "content_type": if locked {
                kanso_core::model::SEALED_FILE_CONTENT_TYPE
            } else {
                content_type
            },
            "digest": attachment.digest(),
            "length": attachment.data.len(),
        });
        // The local API is lock-blind: a sealed file is always a stub, even
        // when a caller asks for its bytes and even while the app is open.
        if data && !locked {
            index["data"] =
                Value::String(base64::engine::general_purpose::STANDARD.encode(&attachment.data));
        } else {
            index["stub"] = Value::Bool(true);
        }
        let mut extra = Map::new();
        extra.insert("_attachments".into(), json!({ "index": index }));
        let mut out = self.to_api(extra);
        if let Some(fields) = out.as_object_mut() {
            if locked {
                fields.insert("name".into(), kanso_core::model::SEALED_FILE_NAME.into());
                fields.insert(
                    "contentType".into(),
                    kanso_core::model::SEALED_FILE_CONTENT_TYPE.into(),
                );
                fields.insert("publicIn".into(), Value::Array(Vec::new()));
                fields.insert("locked".into(), Value::Bool(true));
                // Sealed attachments never expose their OCR text, even if an
                // older writer left a copy in the document.
                fields.remove(kanso_core::model::OCR_FIELD);
            }
            // Both are app-only sync/lock bookkeeping, never API fields.
            fields.remove(kanso_core::model::FILE_LOCK_FIELD);
            fields.remove(kanso_core::model::UNSEALED_FIELD);
        }
        out
    }

    pub fn str_field(&self, key: &str) -> Option<&str> {
        self.fields.get(key).and_then(Value::as_str)
    }
}
