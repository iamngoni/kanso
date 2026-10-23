//! Kanso's domain layer.
//!
//! Documents keep the shapes and IDs of the local-server API (`note:…`, `book:…`, `tag:…`,
//! `file:…`) so links inside note bodies, API clients and imported data keep
//! working unchanged.

pub mod canvas;
pub mod dates;
pub mod excerpt;
pub mod id;
pub mod links;
pub mod model;
pub mod outline;
pub mod query;
pub mod reminders;
pub mod replace;
pub mod rev;
pub mod roles;
pub mod tags;
pub mod tasks;
pub mod template;

pub use excerpt::excerpt;
pub use id::{DocType, new_id};
pub use model::{Book, FileMeta, Note, NoteShare, NoteStatus, TRASH_BOOK_ID, Tag, TagColor};
pub use query::{Clause, Field, NoteFacts, Query};
pub use rev::Rev;

/// Milliseconds since the Unix epoch, the timestamp unit every document uses.
pub fn now_ms() -> i64 {
    use std::time::{SystemTime, UNIX_EPOCH};
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis() as i64)
        .unwrap_or_default()
}
