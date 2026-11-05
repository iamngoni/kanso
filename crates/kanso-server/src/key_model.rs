//! The values of API keys (issue #243): what a key looks like, which one it
//! is, what is asked to make one, and the one time the key itself is shown.

use std::fmt;

use kanso_store::KeyAccess;

/// What every API key starts with. It sets keys apart from session tokens
/// and makes a leaked key easy to search for.
pub const KEY_PREFIX: &str = "kcl_";
/// Characters in a key: the prefix and 43 of base64url (256 bits).
pub const KEY_LEN: usize = 47;

/// Which key, without the key: what Settings lists and revokes.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct KeyId(pub String);

impl KeyId {
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// What the person chose on the New key sheet.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NewKey {
    pub name: String,
    /// Which tool it is for: `codex`, `claude-code`, … `other`.
    pub client: String,
    pub access: KeyAccess,
    /// The notebooks it may read; `None` is all of them.
    pub books: Option<Vec<String>>,
}

/// A key just made. The text is what the caller sends; it is shown once,
/// at minting, and never printed or logged.
pub struct MintedKey {
    pub id: KeyId,
    pub key: String,
}

impl fmt::Debug for MintedKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("MintedKey")
            .field("id", &self.id)
            .field("key", &"<redacted>")
            .finish()
    }
}

/// Whether `text` has the shape of a key: the prefix and 43 base64url
/// characters. Says nothing about whether any key matches.
pub(crate) fn is_key_shaped(text: &str) -> bool {
    text.len() == KEY_LEN
        && text.starts_with(KEY_PREFIX)
        && text[KEY_PREFIX.len()..]
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || b == b'-' || b == b'_')
}
