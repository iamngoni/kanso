//! Typed views of the four document kinds, in the local-server API's JSON shape.
//!
//! Unknown fields are kept in `extra` so documents round-trip unchanged.

use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};

/// The pseudo-notebook that holds trashed notes.
pub const TRASH_BOOK_ID: &str = "trash";

/// The note or notebook field (in `extra`) holding a published note's or
/// notebook's link and key (`kanso_sync::share::KansoPublish`). Only the
/// app's Publish sheets set it: the local API and MCP keep a document's
/// value and ignore any they're sent.
pub const PUBLISH_FIELD: &str = "kansoPublish";

/// The file field (in `extra`) holding the text read from an image
/// attachment: `{"text", "md5", "engine"}`, where `md5` is the image's
/// `md5digest` it was read from. It syncs inside the sealed document, so a
/// device that can't read images still finds them by their words.
pub const OCR_FIELD: &str = "kansoOcr";

/// The notebook field (in `extra`) naming the icon it shows in the sidebar
/// and lists: one of the app's choices, by its Lucide name (`"rocket"`).
/// It syncs with the notebook; a name the app doesn't know shows the
/// default icon.
pub const BOOK_ICON_FIELD: &str = "kansoIcon";

/// The notebook field (in `extra`) holding the color its sidebar row is
/// washed in, so it's easy to find, as hex (`"#2185d0"`). It syncs with the
/// notebook; no field, or a value that isn't hex, shows the row uncolored.
pub const BOOK_COLOR_FIELD: &str = "kansoColor";

/// The note field (in `extra`) holding a locked note's sealed text
/// (`kanso_sync::lock::Envelope`). A locked note's `body` is always empty:
/// its text is only in here, so search, links, tasks, embeddings and the
/// local API never see it. Only the app sets it.
///
/// A notebook has it too when it is locked: an envelope sealing nothing
/// under the lock password, which tells the password and its salt. Every
/// note in a locked notebook, and in the notebooks inside it, is locked.
pub const LOCK_FIELD: &str = "kansoLock";

/// The note field (in `extra`) marking a shared note (docs/ACCOUNTS.md ›
/// Collaboration): `{"shareId", "role", "owner", "ownerName"}`. Its text
/// is the shared note's CRDT's, written back by `kanso-collab`; private sync
/// carries the note without its `title` and `body`. Only Kanso sets it: the
/// local API and MCP keep a note's value and ignore any they're sent.
pub const SHARE_FIELD: &str = "kansoShare";

/// Longest note title the API accepts.
pub const MAX_TITLE_LEN: usize = 128;
/// Longest note body the API accepts, in characters.
pub const MAX_BODY_LEN: usize = 1_048_576;
/// Longest notebook or tag name the API accepts.
pub const MAX_NAME_LEN: usize = 64;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum NoteStatus {
    #[default]
    None,
    Active,
    OnHold,
    Completed,
    Dropped,
}

impl NoteStatus {
    pub const ALL: [NoteStatus; 5] = [
        NoteStatus::None,
        NoteStatus::Active,
        NoteStatus::OnHold,
        NoteStatus::Completed,
        NoteStatus::Dropped,
    ];

    /// The value stored in documents and accepted by `status:` searches.
    pub fn as_str(self) -> &'static str {
        match self {
            NoteStatus::None => "none",
            NoteStatus::Active => "active",
            NoteStatus::OnHold => "onHold",
            NoteStatus::Completed => "completed",
            NoteStatus::Dropped => "dropped",
        }
    }

    /// The label shown in the UI.
    pub fn label(self) -> &'static str {
        match self {
            NoteStatus::None => "None",
            NoteStatus::Active => "Active",
            NoteStatus::OnHold => "On hold",
            NoteStatus::Completed => "Completed",
            NoteStatus::Dropped => "Dropped",
        }
    }

    pub fn parse(value: &str) -> Option<Self> {
        Self::ALL
            .into_iter()
            .find(|s| s.as_str().eq_ignore_ascii_case(value))
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum NoteShare {
    #[default]
    Private,
    Public,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum TagColor {
    #[default]
    Default,
    Red,
    Orange,
    Yellow,
    Olive,
    Green,
    Teal,
    Blue,
    Violet,
    Purple,
    Pink,
    Brown,
    Grey,
    Black,
}

impl TagColor {
    pub const ALL: [TagColor; 14] = [
        TagColor::Default,
        TagColor::Red,
        TagColor::Orange,
        TagColor::Yellow,
        TagColor::Olive,
        TagColor::Green,
        TagColor::Teal,
        TagColor::Blue,
        TagColor::Violet,
        TagColor::Purple,
        TagColor::Pink,
        TagColor::Brown,
        TagColor::Grey,
        TagColor::Black,
    ];

    /// RGB used to draw the tag in the UI.
    pub fn rgb(self) -> u32 {
        match self {
            TagColor::Default => 0x8a8f98,
            TagColor::Red => 0xdb2828,
            TagColor::Orange => 0xf2711c,
            TagColor::Yellow => 0xe5b800,
            TagColor::Olive => 0xb5cc18,
            TagColor::Green => 0x21ba45,
            TagColor::Teal => 0x00b5ad,
            TagColor::Blue => 0x2185d0,
            TagColor::Violet => 0x6435c9,
            TagColor::Purple => 0xa333c8,
            TagColor::Pink => 0xe03997,
            TagColor::Brown => 0xa5673f,
            TagColor::Grey => 0x767676,
            TagColor::Black => 0x1b1c1d,
        }
    }
}

fn markdown() -> String {
    "markdown".to_owned()
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Note {
    #[serde(rename = "_id")]
    pub id: String,
    #[serde(rename = "_rev", default, skip_serializing_if = "Option::is_none")]
    pub rev: Option<String>,
    #[serde(default)]
    pub title: String,
    #[serde(default)]
    pub body: String,
    pub book_id: String,
    #[serde(default)]
    pub tags: Vec<String>,
    #[serde(default)]
    pub status: NoteStatus,
    #[serde(default)]
    pub share: NoteShare,
    #[serde(default)]
    pub pinned: bool,
    #[serde(default = "markdown")]
    pub doctype: String,
    #[serde(default)]
    pub created_at: i64,
    #[serde(default)]
    pub updated_at: i64,
    #[serde(default)]
    pub timestamp: i64,
    #[serde(default)]
    pub num_of_tasks: u32,
    #[serde(default)]
    pub num_of_checked_tasks: u32,
    #[serde(flatten)]
    pub extra: Map<String, Value>,
}

impl Note {
    pub fn is_trashed(&self) -> bool {
        self.book_id == TRASH_BOOK_ID
    }

    /// Whether the note's text is sealed under the lock password.
    pub fn is_locked(&self) -> bool {
        self.extra.contains_key(LOCK_FIELD)
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Book {
    #[serde(rename = "_id")]
    pub id: String,
    #[serde(rename = "_rev", default, skip_serializing_if = "Option::is_none")]
    pub rev: Option<String>,
    pub name: String,
    #[serde(default)]
    pub parent_book_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub order: Option<i64>,
    /// Number of notes in the notebook. Derived; never stored.
    #[serde(default)]
    pub count: u32,
    #[serde(default)]
    pub created_at: i64,
    #[serde(default)]
    pub updated_at: i64,
    #[serde(flatten)]
    pub extra: Map<String, Value>,
}

impl Book {
    /// Whether the notebook itself is locked (a notebook inside it is
    /// locked too, without its own lock).
    pub fn is_locked(&self) -> bool {
        self.extra.contains_key(LOCK_FIELD)
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Tag {
    #[serde(rename = "_id")]
    pub id: String,
    #[serde(rename = "_rev", default, skip_serializing_if = "Option::is_none")]
    pub rev: Option<String>,
    pub name: String,
    #[serde(default)]
    pub color: TagColor,
    /// Number of notes with the tag. Derived; never stored.
    #[serde(default)]
    pub count: u32,
    #[serde(default)]
    pub created_at: i64,
    #[serde(default)]
    pub updated_at: i64,
    #[serde(flatten)]
    pub extra: Map<String, Value>,
}

/// An attachment's metadata. The bytes are stored separately.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FileMeta {
    #[serde(rename = "_id")]
    pub id: String,
    #[serde(rename = "_rev", default, skip_serializing_if = "Option::is_none")]
    pub rev: Option<String>,
    pub name: String,
    pub content_type: String,
    #[serde(default)]
    pub content_length: u64,
    #[serde(default)]
    pub public_in: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub md5digest: Option<String>,
    #[serde(default)]
    pub created_at: i64,
    #[serde(flatten)]
    pub extra: Map<String, Value>,
}

/// Image types the API accepts as attachments.
pub const SUPPORTED_IMAGE_TYPES: [&str; 7] = [
    "image/png",
    "image/jpeg",
    "image/jpg",
    "image/svg+xml",
    "image/gif",
    "image/heic",
    "image/heif",
];

/// Audio types the app attaches, so a recording can sit in a note beside
/// its transcript. The local API's MCP tools keep to images.
pub const SUPPORTED_AUDIO_TYPES: [&str; 7] = [
    "audio/mp4",
    "audio/mpeg",
    "audio/wav",
    "audio/aiff",
    "audio/x-caf",
    "audio/flac",
    "audio/ogg",
];

/// Document types the app attaches, so a PDF shared from another app can
/// sit in a note. The local API's MCP tools keep to images.
pub const SUPPORTED_DOCUMENT_TYPES: [&str; 1] = ["application/pdf"];

/// Largest attachment the API accepts, in bytes.
pub const MAX_ATTACHMENT_SIZE: usize = 10 * 1024 * 1024;

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn note_round_trips_unknown_fields() {
        let raw = json!({
            "_id": "note:abc", "_rev": "2-aa", "title": "T", "body": "B",
            "bookId": "book:x", "tags": ["tag:a"], "status": "onHold", "share": "public",
            "pinned": false, "doctype": "markdown", "createdAt": 1, "updatedAt": 2,
            "timestamp": 3, "numOfTasks": 1, "numOfCheckedTasks": 0,
            "sourceTemplateId": "note:tpl", "migratedBy": "x"
        });
        let note: Note = serde_json::from_value(raw.clone()).unwrap();
        assert_eq!(note.status, NoteStatus::OnHold);
        assert_eq!(note.extra["sourceTemplateId"], "note:tpl");
        assert_eq!(serde_json::to_value(&note).unwrap(), raw);
    }

    #[test]
    fn status_parsing_is_case_insensitive() {
        assert_eq!(NoteStatus::parse("onhold"), Some(NoteStatus::OnHold));
        assert_eq!(NoteStatus::parse("Completed"), Some(NoteStatus::Completed));
        assert_eq!(NoteStatus::parse("later"), None);
    }
}
