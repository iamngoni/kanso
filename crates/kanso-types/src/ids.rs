//! Prefixed, time-ordered entity identifiers.
//!
//! IDs are `"<prefix>:<uuid-v7>"`. v7 is time-ordered (matching the `uuid` v7
//! convention used across the other services), so IDs sort roughly by creation
//! time, which keeps SQLite primary-key inserts append-friendly.

use serde::{Deserialize, Serialize};
use uuid::Uuid;

macro_rules! prefixed_id {
    ($name:ident, $prefix:literal) => {
        #[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
        pub struct $name(pub String);

        impl $name {
            /// Mint a fresh id with a v7 UUID.
            pub fn new() -> Self {
                Self(format!("{}:{}", $prefix, Uuid::now_v7()))
            }

            /// Wrap an existing raw id string (e.g. read back from the database).
            pub fn from_raw(raw: impl Into<String>) -> Self {
                Self(raw.into())
            }

            pub fn as_str(&self) -> &str {
                &self.0
            }

            pub const PREFIX: &'static str = $prefix;
        }

        impl std::fmt::Display for $name {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                f.write_str(&self.0)
            }
        }
    };
}

prefixed_id!(NotebookId, "notebook");
prefixed_id!(NoteId, "note");
prefixed_id!(TagId, "tag");
prefixed_id!(AttachmentId, "attachment");
prefixed_id!(SketchId, "sketch");
prefixed_id!(RevisionId, "revision");
prefixed_id!(DeviceId, "device");
prefixed_id!(SkillId, "skill");
prefixed_id!(SkillRunId, "skillrun");
prefixed_id!(McpClientId, "mcpclient");
