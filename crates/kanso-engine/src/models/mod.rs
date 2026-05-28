//! Persistent domain models.
//!
//! These are the data shapes the engine returns across its API. They live apart
//! from the command logic that reads and mutates them — one entity per file.

mod apply;
mod attachment;
mod note;
mod notebook;
mod sketch;
mod sync_report;
mod tag;

pub use apply::ApplyOutcome;
pub use attachment::{Attachment, NewAttachment};
pub use note::Note;
pub use notebook::Notebook;
pub use sketch::Sketch;
pub use sync_report::SyncReport;
pub use tag::Tag;
