//! Opening a library with the key the OS keeps for it (issue #243 step 9).
//!
//! A new library is made encrypted: its key is 256 random bits, kept as the
//! Keychain (or Credential Manager, or Secret Service) item
//! `library-key:<key_id>`, unlocked with the login session. A library made
//! before this stays plain until it is encrypted (a later step), and opens
//! exactly as it always did.
//!
//! The item can't be named by the library's ID, which lives inside the
//! encrypted file. `<db>.keyid` beside the library holds an opaque random
//! `key_id` instead; it is no secret and says nothing about the notes.

use std::path::{Path, PathBuf};

use kanso_store::{LibraryFile, LibraryKey, Store, StoreError};

use crate::keychain::{SecretError, SecretStore, library_key_item};
use crate::library_key_model::{Encryption, KeyItem, OpenedLibrary};

/// Why a library didn't open.
#[derive(Debug, thiserror::Error)]
pub enum OpenLibraryError {
    #[error(transparent)]
    Store(#[from] StoreError),
    /// The key's item could not be read at all (a locked or denied Keychain).
    /// Not "no key": the item may be fine once the Keychain is.
    #[error("Kanso can't read this library's key from the system keychain: {0}")]
    Keychain(SecretError),
}

/// Where the key's name is kept for the library at `db`.
pub fn key_id_path(db: &Path) -> PathBuf {
    let mut name = db.as_os_str().to_owned();
    name.push(".keyid");
    PathBuf::from(name)
}

fn read_key_id(db: &Path) -> Option<String> {
    let text = std::fs::read_to_string(key_id_path(db)).ok()?;
    let id = text.trim();
    is_key_id(id).then(|| id.to_owned())
}

/// Whether `id` is a key ID this module makes: 32 lowercase hex digits.
pub fn is_key_id(id: &str) -> bool {
    id.len() == 32 && id.bytes().all(|b| matches!(b, b'0'..=b'9' | b'a'..=b'f'))
}

fn new_key_id() -> String {
    let mut bytes = [0u8; 16];
    getrandom::fill(&mut bytes).expect("the system's random source");
    bytes.iter().map(|b| format!("{b:02x}")).collect()
}

/// The key saved under `key_id`: `None` when there is no such item, or it
/// isn't a key.
pub fn load_key(
    secrets: &dyn SecretStore,
    key_id: &str,
) -> Result<Option<LibraryKey>, SecretError> {
    let Some(value) = secrets.get(&library_key_item(key_id))? else {
        return Ok(None);
    };
    Ok(serde_json::from_str::<KeyItem>(&value)
        .ok()
        .filter(|item| item.v == 1)
        .and_then(|item| LibraryKey::from_hex(&item.key)))
}

/// Saves `key` under `key_id`.
pub fn save_key(
    secrets: &dyn SecretStore,
    key_id: &str,
    key: &LibraryKey,
) -> Result<(), SecretError> {
    let item = zeroize::Zeroizing::new(
        serde_json::to_string(&KeyItem {
            v: 1,
            key: key.to_hex().as_str().to_owned(),
        })
        .expect("a key item is JSON"),
    );
    secrets.set(&library_key_item(key_id), &item)
}

/// Opens the library at `db`.
///
/// - A plain file opens plain, as before.
/// - An encrypted file opens with the key the secrets hold for it. No key
///   (the item is gone, or this computer never had it) is
///   [`StoreError::WrongKey`]: "Kanso can't open this library with this
///   computer's key."
/// - A new library is made encrypted. If the secret store can't keep the key
///   (a Linux computer with no Secret Service, say), it is made plain
///   instead, never half-keyed: the answer says so, and a later step
///   encrypts it.
pub fn open_library(
    db: &Path,
    secrets: &dyn SecretStore,
) -> Result<OpenedLibrary, OpenLibraryError> {
    if let Some(dir) = db.parent() {
        std::fs::create_dir_all(dir).map_err(StoreError::from)?;
    }
    match LibraryFile::of(db) {
        LibraryFile::Plain => Ok(OpenedLibrary {
            store: Store::open(db)?,
            encryption: Encryption::Plain,
        }),
        LibraryFile::Encrypted => {
            let key = read_key_id(db)
                .map(|id| load_key(secrets, &id))
                .transpose()
                .map_err(OpenLibraryError::Keychain)?
                .flatten()
                .ok_or(StoreError::WrongKey)?;
            Ok(OpenedLibrary {
                store: Store::open_with_key(db, Some(&key))?,
                encryption: Encryption::Encrypted,
            })
        }
        LibraryFile::New => create_library(db, secrets),
    }
}

fn create_library(db: &Path, secrets: &dyn SecretStore) -> Result<OpenedLibrary, OpenLibraryError> {
    let key = LibraryKey::generate();
    let key_id = new_key_id();
    let kept = save_key(secrets, &key_id, &key).and_then(|()| match load_key(secrets, &key_id)? {
        Some(read) if read == key => Ok(()),
        _ => Err(SecretError("the key did not read back".into())),
    });
    if let Err(err) = kept {
        let _ = secrets.delete(&library_key_item(&key_id));
        return Ok(OpenedLibrary {
            store: Store::open(db)?,
            encryption: Encryption::PlainNoKeychain(err.0),
        });
    }
    // The key is safe before the library exists, so a crash between the two
    // leaves an unused item, never a library with no key.
    let named = std::fs::write(key_id_path(db), &key_id);
    let opened = named
        .map_err(StoreError::from)
        .and_then(|()| Store::open_with_key(db, Some(&key)));
    match opened {
        Ok(store) => Ok(OpenedLibrary {
            store,
            encryption: Encryption::Encrypted,
        }),
        Err(err) => {
            let _ = secrets.delete(&library_key_item(&key_id));
            let _ = std::fs::remove_file(key_id_path(db));
            Err(err.into())
        }
    }
}
