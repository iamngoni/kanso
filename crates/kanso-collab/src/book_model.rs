//! The shapes shared notebooks' keys are made of (docs/ACCOUNTS.md ›
//! Collaboration; issue #390 › Shared notebooks: keys). Logic lives in
//! [`crate::book`].

use std::collections::BTreeMap;
use std::fmt;

use zeroize::Zeroizing;

/// A notebook's key for one epoch, `K_b[e]`. Every note's key and the
/// manifest's key derive from it. Zeroized on drop; `Debug` shows nothing.
#[derive(Clone)]
pub struct BookKey(pub(crate) Zeroizing<[u8; 32]>);

impl fmt::Debug for BookKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("BookKey(<redacted>)")
    }
}

impl PartialEq for BookKey {
    fn eq(&self, other: &BookKey) -> bool {
        self.0[..] == other.0[..]
    }
}

impl BookKey {
    pub fn from_bytes(bytes: [u8; 32]) -> BookKey {
        BookKey(Zeroizing::new(bytes))
    }

    pub fn bytes(&self) -> &[u8; 32] {
        &self.0
    }
}

/// The keys of a notebook's earlier epochs, as far as a member's history
/// links reach, by epoch number.
#[derive(Debug, Default, PartialEq)]
pub struct BookHistory {
    pub keys: BTreeMap<u32, BookKey>,
}
