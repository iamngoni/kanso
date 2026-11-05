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
    /// A plain library that this open encrypted.
    Converted,
    /// A plain library, new or from before encryption, left plain because
    /// the secret store couldn't keep a key; holds why. The next open tries
    /// again.
    PlainNoKeychain(String),
    /// A plain library that could not be encrypted (the disk is too small,
    /// say), left as it was; holds why in words a person can act on. The next
    /// open tries again.
    PlainEncryptFailed(String),
}

/// An opened library.
pub struct OpenedLibrary {
    pub store: Store,
    pub encryption: Encryption,
}
