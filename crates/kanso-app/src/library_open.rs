//! Opening the library in a data folder with its key (issue #243 steps 9
//! and 10).
//!
//! A new library is encrypted, and its key goes in the secret store the
//! config names. A plain library is encrypted at the first open that can keep
//! a key (`kanso_sync::open_library`), with its backups. Every program that
//! opens the library holds the data-folder lock first, so none can open it
//! while it is converted.

use std::path::Path;

use anyhow::{Context as _, Result};
use kanso_store::{EncryptOptions, EncryptProgress, LibraryFile, ReadOnlyStore, Store};
use kanso_sync::keychain::SecretStore;
use kanso_sync::library_key::{key_id_path, load_key};
use kanso_sync::{Encryption, open_library_with};

#[cfg(not(test))]
use crate::config::AppConfig;
use crate::paths;

/// Whether opening the library will encrypt it: it is plain, or a
/// conversion is unfinished. The app shows a progress window then.
pub fn will_encrypt(data_dir: &Path) -> bool {
    let db = paths::db_path(data_dir);
    LibraryFile::of(&db) == LibraryFile::Plain || kanso_store::encryption_unfinished(&db)
}

/// Opens the library in `data_dir`, with its key from `secrets`.
pub fn open_store_with(data_dir: &Path, secrets: &dyn SecretStore) -> Result<Store> {
    let reporter = crate::encrypt_progress::Reporter::stderr();
    Ok(open_store_reporting(data_dir, secrets, &|progress| reporter.report(progress))?.0)
}

/// [`open_store_with`], telling `progress` how far an encryption has got,
/// and giving back why the library is plain when it is.
pub fn open_store_reporting(
    data_dir: &Path,
    secrets: &dyn SecretStore,
    progress: &(dyn Fn(EncryptProgress) + Sync),
) -> Result<(Store, Option<String>)> {
    let db = paths::db_path(data_dir);
    let opts = EncryptOptions {
        progress: Some(progress),
        ..EncryptOptions::default()
    };
    let opened = open_library_with(&db, secrets, &opts)
        .with_context(|| format!("opening {}", db.display()))?;
    let plain = match &opened.encryption {
        Encryption::PlainNoKeychain(why) => Some(format!(
            "The system keychain wouldn't keep the library's key ({why}). Kanso tries again each time it opens."
        )),
        Encryption::PlainEncryptFailed(why) => Some(why.clone()),
        Encryption::Converted => {
            eprintln!("kanso: your library is now encrypted");
            None
        }
        Encryption::Encrypted => None,
    };
    if let Some(why) = &plain {
        eprintln!("kanso: this library is not encrypted. {why}");
    }
    // Backups made before encryption are plain copies of the library.
    if let Some(key) = opened.store.library_key() {
        let backups = kanso_store::encrypt_backups(&paths::backups_dir(data_dir), key, &opts)?;
        for (path, why) in &backups.failed {
            eprintln!(
                "kanso: couldn't encrypt the backup {}: {why}",
                path.display()
            );
        }
    }
    Ok((opened.store, plain))
}

/// Opens the library in `data_dir`, with the key from the secret store the
/// config names.
pub fn open_store(data_dir: &Path) -> Result<Store> {
    let reporter = crate::encrypt_progress::Reporter::stderr();
    Ok(open_store_progress(data_dir, &|progress| reporter.report(progress))?.0)
}

/// [`open_store`], telling `progress` how far an encryption has got, and
/// giving back why the library is plain when it is.
pub fn open_store_progress(
    data_dir: &Path,
    progress: &(dyn Fn(EncryptProgress) + Sync),
) -> Result<(Store, Option<String>)> {
    // Tests never touch the system's keychain: one in-memory store stands in.
    #[cfg(test)]
    {
        static SECRETS: std::sync::OnceLock<kanso_sync::MemorySecrets> = std::sync::OnceLock::new();
        open_store_reporting(
            data_dir,
            SECRETS.get_or_init(kanso_sync::MemorySecrets::new),
            progress,
        )
    }
    #[cfg(not(test))]
    {
        let secrets = secrets_for(data_dir);
        open_store_reporting(data_dir, secrets.as_ref(), progress)
    }
}

/// [`Store::open_read_only`] for a library that may be encrypted, for
/// reading `meta` while another process owns it.
pub fn open_read_only(data_dir: &Path, secrets: &dyn SecretStore) -> Result<ReadOnlyStore> {
    let db = paths::db_path(data_dir);
    let key = match LibraryFile::of(&db) {
        LibraryFile::Encrypted => {
            let id = std::fs::read_to_string(key_id_path(&db)).unwrap_or_default();
            let key = load_key(secrets, id.trim())
                .map_err(|err| anyhow::anyhow!("{err}"))?
                .ok_or(kanso_store::StoreError::WrongKey)?;
            Some(key)
        }
        _ => None,
    };
    Ok(Store::open_read_only_with_key(&db, key.as_ref())?)
}

#[cfg(not(test))]
fn secrets_for(data_dir: &Path) -> std::sync::Arc<dyn SecretStore> {
    let config = AppConfig::load(&paths::config_path(data_dir)).unwrap_or_default();
    crate::secrets::open_secrets_or_refuse(&config, data_dir)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A plain library from before encryption opens encrypted, with its
    /// notes and a plain backup also encrypted, and the reason it is plain
    /// is cleared.
    #[test]
    fn opening_a_plain_library_encrypts_it_and_its_backups() {
        let dir = tempfile::tempdir().unwrap();
        let db = paths::db_path(dir.path());
        {
            let store = Store::open(&db).unwrap();
            store.set_meta("a", "b").unwrap();
            store
                .backup_now(&paths::backups_dir(dir.path()), 1_000)
                .unwrap();
        }
        let secrets = kanso_sync::MemorySecrets::new();
        let (store, plain) = open_store_reporting(dir.path(), &secrets, &|_| {}).unwrap();
        assert!(store.is_encrypted());
        assert_eq!(store.meta("a").unwrap().as_deref(), Some("b"));
        assert_eq!(plain, None);
        drop(store);
        assert_eq!(LibraryFile::of(&db), LibraryFile::Encrypted);
        for entry in std::fs::read_dir(paths::backups_dir(dir.path())).unwrap() {
            assert_eq!(
                LibraryFile::of(&entry.unwrap().path()),
                LibraryFile::Encrypted
            );
        }
    }

    /// A key store that refuses leaves the library plain, says why, and the
    /// next open (with a working store) encrypts it.
    #[test]
    fn a_refusing_keychain_is_remembered_then_cured() {
        let dir = tempfile::tempdir().unwrap();
        let db = paths::db_path(dir.path());
        Store::open(&db).unwrap().set_meta("a", "b").unwrap();
        let secrets = kanso_sync::MemorySecrets::new();
        secrets.fail_writes_to(Some("library-key:"));
        let (store, plain) = open_store_reporting(dir.path(), &secrets, &|_| {}).unwrap();
        assert!(!store.is_encrypted());
        assert!(plain.is_some_and(|why| why.contains("keychain")));
        drop(store);
        secrets.heal();
        let (store, plain) = open_store_reporting(dir.path(), &secrets, &|_| {}).unwrap();
        assert!(store.is_encrypted());
        assert_eq!(plain, None);
    }
}
