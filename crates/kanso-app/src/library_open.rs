//! Opening the library in a data folder with its key (issue #243 step 9).
//!
//! A new library is encrypted and its key goes in the secret store the config
//! names; a library from before stays plain until it is encrypted. See
//! `kanso_sync::open_library`.

use std::path::Path;

use anyhow::{Context as _, Result};
use kanso_store::{LibraryFile, ReadOnlyStore, Store};
use kanso_sync::Encryption;
use kanso_sync::keychain::SecretStore;
use kanso_sync::library_key::{key_id_path, load_key};

#[cfg(not(test))]
use crate::config::AppConfig;
use crate::paths;

/// Opens the library in `data_dir`, with its key from `secrets`.
pub fn open_store_with(data_dir: &Path, secrets: &dyn SecretStore) -> Result<Store> {
    let db = paths::db_path(data_dir);
    let opened = kanso_sync::open_library(&db, secrets)
        .with_context(|| format!("opening {}", db.display()))?;
    if let Encryption::PlainNoKeychain(why) = &opened.encryption {
        eprintln!(
            "kanso: this new library is not encrypted: the system keychain refused its key ({why})"
        );
    }
    Ok(opened.store)
}

/// Opens the library in `data_dir`, with the key from the secret store the
/// config names.
pub fn open_store(data_dir: &Path) -> Result<Store> {
    // Tests never touch the system's keychain: one in-memory store stands in.
    #[cfg(test)]
    {
        static SECRETS: std::sync::OnceLock<kanso_sync::MemorySecrets> = std::sync::OnceLock::new();
        open_store_with(
            data_dir,
            SECRETS.get_or_init(kanso_sync::MemorySecrets::new),
        )
    }
    #[cfg(not(test))]
    {
        let secrets = secrets_for(data_dir);
        open_store_with(data_dir, secrets.as_ref())
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
