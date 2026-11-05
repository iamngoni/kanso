//! The shapes of Remove Lock's file step (`Store::unlock_note_unsealing`).

use kanso_core::model::FileMeta;
use serde_json::Value;

use crate::doc::Attachment;

/// What taking a locked note's lock off does to its files, all written in
/// the one transaction that writes the plain note.
#[derive(Debug, Default)]
pub struct Unsealing {
    /// Sealed files decrypted in place: the file as it is stored (its seal
    /// marker still on it) with the name and type to restore, and the plain
    /// bytes.
    pub files: Vec<(FileMeta, Attachment)>,
    /// Plain copies for files another locked note still shows: each a file
    /// body with its `_id` and `_attachments`, as `Store::put` takes one.
    pub copies: Vec<Value>,
}

impl Unsealing {
    /// Whether there is nothing to write but the note.
    pub fn is_empty(&self) -> bool {
        self.files.is_empty() && self.copies.is_empty()
    }
}
