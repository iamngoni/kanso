//! Which secret store a library's session lives in (docs/ACCOUNTS.md ›
//! Keychain items): the system's (the Keychain, Credential Manager or the
//! Secret Service), or `secrets.json` in the data folder for a computer that
//! has none, such as a headless Linux server running `kanso serve`.

use std::path::{Path, PathBuf};
use std::sync::Arc;

use anyhow::Result;
use kanso_sync::keychain::{FileSecrets, SECRETS_FILE, SECRETS_KEY_FILE_ENV};
use kanso_sync::{SecretError, SecretStore};
use zeroize::Zeroizing;

use crate::config::{AppConfig, SecretStoreKind};

/// Where `secrets.json` is for the library in `data_dir`.
pub fn secrets_path(data_dir: &Path) -> PathBuf {
    data_dir.join(SECRETS_FILE)
}

/// The key file `KANSO_SECRETS_KEY_FILE` names, if it is set: a path, never
/// key bytes, so the environment never holds the key.
pub fn key_file() -> Option<PathBuf> {
    std::env::var_os(SECRETS_KEY_FILE_ENV)
        .filter(|path| !path.is_empty())
        .map(PathBuf::from)
}

/// The secret store of `kind` for this flavour and data folder.
pub fn open_kind(kind: SecretStoreKind, data_dir: &Path) -> Result<Arc<dyn SecretStore>> {
    Ok(match kind {
        SecretStoreKind::System => Arc::new(crate::flavour::CURRENT.keychain()),
        SecretStoreKind::File => Arc::new(FileSecrets::open(
            secrets_path(data_dir),
            key_file().as_deref(),
        )?),
    })
}

/// The secret store the config names, for this flavour.
pub fn open_secrets(config: &AppConfig, data_dir: &Path) -> Result<Arc<dyn SecretStore>> {
    open_kind(config.sync.secret_store, data_dir)
}

/// The store the config names, or one that refuses every read and write
/// when it can't be opened (its key file is missing, say): the account then
/// shows a Keychain problem instead of reading as signed out, which would
/// let a sign-in overwrite the session it couldn't read.
pub fn open_secrets_or_refuse(config: &AppConfig, data_dir: &Path) -> Arc<dyn SecretStore> {
    match open_secrets(config, data_dir) {
        Ok(secrets) => secrets,
        Err(err) => {
            eprintln!("kanso: the secret store: {err:#}");
            Arc::new(Unavailable(format!("{err:#}")))
        }
    }
}

/// A secret store that couldn't be opened: every call fails with why.
pub struct Unavailable(pub String);

// no-prop-test: every method returns the same error.
impl SecretStore for Unavailable {
    fn get(&self, _: &str) -> Result<Option<Zeroizing<String>>, SecretError> {
        Err(SecretError(self.0.clone()))
    }

    fn set(&self, _: &str, _: &str) -> Result<(), SecretError> {
        Err(SecretError(self.0.clone()))
    }

    fn delete(&self, _: &str) -> Result<(), SecretError> {
        Err(SecretError(self.0.clone()))
    }
}

/// Whether `store` answers: writes, reads back and deletes a probe item
/// `kanso-probe:<random>`; false on any error. `kanso account sign-in` uses
/// it to pick the default store on Linux, where a server has no Secret
/// Service.
pub fn store_works(store: &dyn SecretStore) -> bool {
    let item = format!("kanso-probe:{:016x}", fastrand::u64(..));
    let wrote = store.set(&item, "probe").is_ok();
    let read = wrote && store.get(&item).ok().flatten().as_deref().map(String::as_str) == Some("probe");
    let deleted = store.delete(&item).is_ok();
    wrote && read && deleted
}

/// The store a new sign-in uses when none is asked for: the one the config
/// already names when it names the file; else the system's where it works,
/// and the file where it doesn't (Linux without a Secret Service). macOS and
/// Windows always have theirs.
pub fn default_kind(config: &AppConfig, system_works: impl FnOnce() -> bool) -> SecretStoreKind {
    if config.sync.secret_store == SecretStoreKind::File {
        return SecretStoreKind::File;
    }
    if cfg!(target_os = "linux") && !system_works() {
        SecretStoreKind::File
    } else {
        SecretStoreKind::System
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use kanso_sync::MemorySecrets;
    use proptest::prelude::*;

    #[test]
    fn a_working_store_passes_the_probe_and_keeps_nothing() {
        let secrets = MemorySecrets::new();
        assert!(store_works(&secrets));
        assert!(secrets.items().is_empty());
        secrets.fail_writes_to(Some("kanso-probe:"));
        assert!(!store_works(&secrets));
        secrets.heal();
        secrets.fail_reads(true);
        assert!(!store_works(&secrets));
        assert!(!store_works(&Unavailable("no".into())));
    }

    #[test]
    fn the_file_store_opens_in_the_data_folder() {
        let dir = tempfile::tempdir().unwrap();
        let secrets = open_kind(SecretStoreKind::File, dir.path()).unwrap();
        assert!(store_works(secrets.as_ref()));
        secrets.set("sync-account:l_1", "x").unwrap();
        assert!(secrets_path(dir.path()).exists());
    }

    proptest! {
        /// A config that names the file keeps it; otherwise the system's
        /// store is the default wherever it works, and the file only on
        /// Linux where it doesn't.
        #[test]
        fn prop_default_kind(file: bool, works: bool) {
            let mut config = AppConfig::default();
            if file {
                config.sync.secret_store = SecretStoreKind::File;
            }
            let kind = default_kind(&config, || works);
            let expected = if file || (cfg!(target_os = "linux") && !works) {
                SecretStoreKind::File
            } else {
                SecretStoreKind::System
            };
            prop_assert_eq!(kind, expected);
        }
    }
}
