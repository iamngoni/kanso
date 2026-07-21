//! Persistent domain models.
//!
//! These are the data shapes the engine returns across its API. They live apart
//! from the command logic that reads and mutates them — one entity per file.

mod apply;
mod attachment;
mod io;
mod mcp_client;
mod note;
mod note_link;
mod notebook;
mod revision;
mod share;
mod sketch;
mod skill;
mod sync_report;
mod tag;
mod task_item;

pub use apply::ApplyOutcome;
pub use attachment::{Attachment, NewAttachment};
pub use io::{ExportFile, ImportFile};
pub use mcp_client::McpClient;
pub use note::Note;
pub use note_link::NoteLink;
pub use notebook::Notebook;
pub use revision::Revision;
pub use share::{Share, ShareMember};
pub use sketch::Sketch;
pub use skill::{Skill, SkillRun};
pub use sync_report::SyncReport;
pub use tag::Tag;
pub use task_item::TaskItem;
