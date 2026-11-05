//! What opening a library gives back, and the Keychain item that holds its
//! key (issue #243).

use kanso_store::Store;
use serde::{Deserialize, Serialize};
use zeroize::Zeroize as _;

/// The value of the `library-key:<key_id>` item. The key's hex is wiped
/// when it is dropped.
#[derive(Serialize, Deserialize)]
pub(crate) struct KeyItem {
    pub v: u32,
    /// The 256-bit key as 64 hex digits.
    pub key: String,
}

impl Drop for KeyItem {
    fn drop(&mut self) {
        self.key.zeroize();
    }
}

/// Whether an opened library is encrypted.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Encryption {
    Encrypted,
    /// A library from before encryption, left as it was.
    Plain,
    /// A new library made plain because the secret store couldn't keep a key;
    /// holds why.
    PlainNoKeychain(String),
}

/// An opened library.
pub struct OpenedLibrary {
    pub store: Store,
    pub encryption: Encryption,
}
