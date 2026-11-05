//! The key an encrypted library opens with, and what a library file is
//! (issue #243).

use std::fmt;

use zeroize::{Zeroize, Zeroizing};

/// Bytes in a library key: SQLCipher's raw 256-bit key.
pub const KEY_BYTES: usize = 32;

/// The 256-bit key of an encrypted library. It is passed to SQLCipher as a raw
/// key (`x'<64 hex>'`), so opening costs no key stretching. Wiped from memory
/// when dropped; its `Debug` shows nothing.
#[derive(Clone)]
pub struct LibraryKey(Zeroizing<[u8; KEY_BYTES]>);

impl LibraryKey {
    /// A new key from the system's random source.
    pub fn generate() -> LibraryKey {
        let mut bytes = [0u8; KEY_BYTES];
        getrandom::fill(&mut bytes).expect("the system's random source");
        let key = LibraryKey::from_bytes(bytes);
        bytes.zeroize();
        key
    }

    pub fn from_bytes(bytes: [u8; KEY_BYTES]) -> LibraryKey {
        LibraryKey(Zeroizing::new(bytes))
    }

    pub fn as_bytes(&self) -> &[u8; KEY_BYTES] {
        &self.0
    }

    /// The key as 64 lowercase hex digits.
    pub fn to_hex(&self) -> Zeroizing<String> {
        let mut hex = Zeroizing::new(String::with_capacity(KEY_BYTES * 2));
        for byte in self.0.iter() {
            hex.push(char::from_digit(u32::from(byte >> 4), 16).expect("a nibble"));
            hex.push(char::from_digit(u32::from(byte & 15), 16).expect("a nibble"));
        }
        hex
    }

    /// The key written as exactly 64 hex digits (either case); `None` for
    /// anything else.
    pub fn from_hex(hex: &str) -> Option<LibraryKey> {
        if hex.len() != KEY_BYTES * 2 || !hex.is_ascii() {
            return None;
        }
        let mut bytes = [0u8; KEY_BYTES];
        for (byte, pair) in bytes.iter_mut().zip(hex.as_bytes().chunks(2)) {
            let high = char::from(pair[0]).to_digit(16)?;
            let low = char::from(pair[1]).to_digit(16)?;
            *byte = u8::try_from(high << 4 | low).ok()?;
        }
        let key = LibraryKey::from_bytes(bytes);
        bytes.zeroize();
        Some(key)
    }

    /// `PRAGMA key = "x'<hex>'"`: the statement that unlocks a connection.
    pub(crate) fn pragma(&self) -> Zeroizing<String> {
        Zeroizing::new(format!("PRAGMA key = \"x'{}'\";", self.to_hex().as_str()))
    }
}

impl PartialEq for LibraryKey {
    fn eq(&self, other: &LibraryKey) -> bool {
        // Without an early exit.
        self.0
            .iter()
            .zip(other.0.iter())
            .fold(0u8, |diff, (a, b)| diff | (a ^ b))
            == 0
    }
}

impl Eq for LibraryKey {}

impl fmt::Debug for LibraryKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("LibraryKey(..)")
    }
}

/// What is at a library's path.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LibraryFile {
    /// Nothing, or an empty file: a library about to be made.
    New,
    /// A plain SQLite file, from before libraries were encrypted.
    Plain,
    /// Anything else: an encrypted library, which looks like noise.
    Encrypted,
}
