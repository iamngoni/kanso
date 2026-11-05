//! Opening a library with its key (issue #243).

use std::io::Read;
use std::path::Path;

use rusqlite::{Connection, ErrorCode};

use crate::{LibraryFile, LibraryKey, Result, Store, StoreError};

/// The first 16 bytes of every plain SQLite file.
const PLAIN_HEADER: &[u8; 16] = b"SQLite format 3\0";

impl LibraryFile {
    /// What is at `path`. Only a missing (or empty) file is new: a file that
    /// is there but can't be read (its permissions, a lock, a directory in
    /// its place) reads as encrypted, so the open that follows says why and
    /// nothing is made over it or its key's name.
    pub fn of(path: &Path) -> LibraryFile {
        let mut file = match std::fs::File::open(path) {
            Ok(file) => file,
            Err(err) if err.kind() == std::io::ErrorKind::NotFound => return LibraryFile::New,
            Err(_) => return LibraryFile::Encrypted,
        };
        let mut head = [0u8; 16];
        let mut read = 0;
        while read < head.len() {
            match file.read(&mut head[read..]) {
                Ok(0) => break,
                Ok(n) => read += n,
                Err(_) => return LibraryFile::Encrypted,
            }
        }
        match read {
            0 => LibraryFile::New,
            16 if &head == PLAIN_HEADER => LibraryFile::Plain,
            _ => LibraryFile::Encrypted,
        }
    }
}

/// Unlocks `conn` with `key` (when there is one) and proves the file reads:
/// a wrong key, or no key for an encrypted file, is [`StoreError::WrongKey`];
/// a key on a build with no SQLCipher is [`StoreError::NoEncryption`].
pub(crate) fn unlock(conn: &Connection, key: Option<&LibraryKey>) -> Result<()> {
    if let Some(key) = key {
        let has_cipher = conn
            .query_row("PRAGMA cipher_version", [], |row| row.get::<_, String>(0))
            .is_ok();
        if !has_cipher {
            return Err(StoreError::NoEncryption);
        }
        conn.execute_batch(&key.pragma())?;
    }
    conn.query_row("SELECT count(*) FROM sqlite_master", [], |row| {
        row.get::<_, i64>(0)
    })
    .map_err(|err| match err.sqlite_error_code() {
        Some(ErrorCode::NotADatabase) => StoreError::WrongKey,
        _ => StoreError::Sqlite(err),
    })?;
    Ok(())
}

impl Store {
    /// Opens the library at `path` with `key`: `None` for a plain library
    /// (and for a new one that stays plain), `Some` to create an encrypted
    /// one or open one. A wrong key, or none for an encrypted file, is
    /// [`StoreError::WrongKey`] and nothing is written.
    pub fn open_with_key(path: impl AsRef<Path>, key: Option<&LibraryKey>) -> Result<Store> {
        Store::open_inner(path.as_ref(), key)
    }

    /// Whether this library is encrypted.
    pub fn is_encrypted(&self) -> bool {
        self.key.is_some()
    }

    /// The library's key, for a backup or a second connection.
    pub fn library_key(&self) -> Option<&LibraryKey> {
        self.key.as_ref()
    }
}
