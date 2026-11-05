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
    // A key's name already beside a library that isn't there yet: a create
    // that stopped after naming the key (or a database file still empty,
    // its first pages in the WAL). Its key is reused, never replaced, so
    // nothing written under it becomes unreadable.
    if let Some(key_id) = read_key_id(db)
        && let Some(key) = load_key(secrets, &key_id).map_err(OpenLibraryError::Keychain)?
    {
        return Ok(OpenedLibrary {
            store: Store::open_with_key(db, Some(&key))?,
            encryption: Encryption::Encrypted,
        });
    }
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
    let named = write_key_id(db, &key_id);
    let opened = named
        .map_err(StoreError::from)
        .and_then(|()| Store::open_with_key(db, Some(&key)));
    match opened {
        Ok(store) => Ok(OpenedLibrary {
            store,
            encryption: Encryption::Encrypted,
        }),
        Err(err) => {
            // Undone only while nothing was written under the key: once the
            // file has pages, its key and name stay, so the next open
            // finishes the job instead of finding a library it can't read.
            if LibraryFile::of(db) == LibraryFile::New {
                let _ = secrets.delete(&library_key_item(&key_id));
                let _ = std::fs::remove_file(key_id_path(db));
            }
            Err(err.into())
        }
    }
}

/// Writes the key's name beside the library: to a temporary file, flushed,
/// then renamed over, so a crash leaves the old name or the new one.
fn write_key_id(db: &Path, key_id: &str) -> std::io::Result<()> {
    use std::io::Write as _;
    let path = key_id_path(db);
    let mut temp = path.clone().into_os_string();
    temp.push(".tmp");
    let temp = PathBuf::from(temp);
    {
        let mut file = std::fs::File::create(&temp)?;
        file.write_all(key_id.as_bytes())?;
        file.sync_all()?;
    }
    std::fs::rename(&temp, &path)
}

/// Copies the key of the library at `db` from `from` to `to`, read back, so
/// the library opens with either store: for a sign-in that moves the
/// library's secrets to another store. Nothing to do (`false`) for a plain
/// or missing library. A key `from` doesn't hold is
/// [`StoreError::WrongKey`]; `to` refusing it is
/// [`OpenLibraryError::Keychain`], and the library still opens from `from`.
pub fn copy_key(
    db: &Path,
    from: &dyn SecretStore,
    to: &dyn SecretStore,
) -> Result<bool, OpenLibraryError> {
    if LibraryFile::of(db) != LibraryFile::Encrypted {
        return Ok(false);
    }
    let key_id = read_key_id(db).ok_or(StoreError::WrongKey)?;
    let key = load_key(from, &key_id)
        .map_err(OpenLibraryError::Keychain)?
        .ok_or(StoreError::WrongKey)?;
    save_key(to, &key_id, &key).map_err(OpenLibraryError::Keychain)?;
    match load_key(to, &key_id).map_err(OpenLibraryError::Keychain)? {
        Some(read) if read == key => Ok(true),
        _ => Err(OpenLibraryError::Keychain(SecretError(
            "the key did not read back".into(),
        ))),
    }
}

/// Deletes the key of the library at `db` from `from`, once `kept` holds it
/// ([`copy_key`]). Never deletes the only copy: nothing happens unless
/// `kept` reads the key back, and if the two are one store the key is put
/// back. Nothing to do for a plain library.
pub fn forget_key(
    db: &Path,
    from: &dyn SecretStore,
    kept: &dyn SecretStore,
) -> Result<(), SecretError> {
    let Some(key_id) = read_key_id(db) else {
        return Ok(());
    };
    let Some(key) = load_key(kept, &key_id)? else {
        return Ok(());
    };
    from.delete(&library_key_item(&key_id))?;
    if load_key(kept, &key_id)?.as_ref() != Some(&key) {
        save_key(kept, &key_id, &key)?;
    }
    Ok(())
}
