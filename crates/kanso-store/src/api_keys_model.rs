//! The rows of the API keys table (issue #243).

use std::fmt;

/// What a key may do.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum KeyAccess {
    /// Read notes. The default: the least a key can be given.
    #[default]
    Read,
    /// Read, and suggest edits the person reviews before they are saved.
    Suggest,
    /// Read and write; changes are saved right away.
    Write,
}

impl KeyAccess {
    pub const ALL: [KeyAccess; 3] = [KeyAccess::Read, KeyAccess::Suggest, KeyAccess::Write];

    pub fn as_str(self) -> &'static str {
        match self {
            KeyAccess::Read => "read",
            KeyAccess::Suggest => "suggest",
            KeyAccess::Write => "write",
        }
    }

    pub fn parse(text: &str) -> Option<KeyAccess> {
        KeyAccess::ALL.into_iter().find(|a| a.as_str() == text)
    }
}

/// A key to save. Carries the hash of the key, never the key.
#[derive(Clone, PartialEq, Eq)]
pub struct NewApiKey {
    /// A UUID v7.
    pub id: String,
    pub name: String,
    /// Which tool it is for: `codex`, `claude-code`, … `other`.
    pub client: String,
    /// SHA-256 of the key.
    pub token_hash: [u8; 32],
    pub access: KeyAccess,
    /// The notebooks it may read; `None` is all of them.
    pub books: Option<Vec<String>>,
    pub created_at: i64,
}

impl fmt::Debug for NewApiKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("NewApiKey")
            .field("id", &self.id)
            .field("name", &self.name)
            .field("access", &self.access)
            .field("token_hash", &"<redacted>")
            .finish_non_exhaustive()
    }
}

/// A saved key, as Settings lists it. The hash is not part of it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ApiKeyRow {
    pub id: String,
    pub name: String,
    pub client: String,
    pub access: KeyAccess,
    pub books: Option<Vec<String>>,
    pub created_at: i64,
    pub last_used_at: Option<i64>,
    pub revoked_at: Option<i64>,
}

/// A key that has not been revoked, with the hash to check a given key
/// against.
#[derive(Clone)]
pub struct LiveKey {
    pub row: ApiKeyRow,
    pub token_hash: [u8; 32],
}

impl fmt::Debug for LiveKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("LiveKey")
            .field("row", &self.row)
            .field("token_hash", &"<redacted>")
            .finish()
    }
}
