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

/// Makes the data folder owner-only (`0700`) when others can reach into it:
/// with the file store, `secrets.json` (and the database) sit there, and the
/// folder was made with the process's umask, usually `0755`. Returns whether
/// it changed anything.
#[cfg(unix)]
pub fn protect_data_dir(data_dir: &Path) -> std::io::Result<bool> {
    use std::os::unix::fs::PermissionsExt as _;
    let mode = std::fs::metadata(data_dir)?.permissions().mode();
    if mode & 0o077 == 0 {
        return Ok(false);
    }
    std::fs::set_permissions(data_dir, std::fs::Permissions::from_mode(0o700))?;
    Ok(true)
}

/// Windows: the folder's ACL is the user's already.
#[cfg(not(unix))]
pub fn protect_data_dir(_: &Path) -> std::io::Result<bool> {
    Ok(false)
}

/// The key file `KANSO_SECRETS_KEY_FILE` names, if it is set: a path, never
/// key bytes, so the environment never holds the key.
pub fn key_file() -> Option<PathBuf> {
    std::env::var_os(SECRETS_KEY_FILE_ENV)
        .filter(|path| !path.is_empty())
        .map(PathBuf::from)
}

/// The warning for a library whose key sits in the clear: with the file
/// secret store and no `KANSO_SECRETS_KEY_FILE`, `secrets.json` holds the
/// library's key in plain text beside `kanso.db`, so encryption stops only a
/// copy of `kanso.db` on its own, not anyone who can read the folder. `None`
/// when the system's store or a key file protects it. Pure.
pub fn file_store_warning(
    kind: SecretStoreKind,
    key_file: Option<&Path>,
    data_dir: &Path,
) -> Option<String> {
    if kind != SecretStoreKind::File || key_file.is_some() {
        return None;
    }
    Some(format!(
        "kanso: the library's key is in {} in plain text, next to the library, because {SECRETS_KEY_FILE_ENV} is not set. Anyone who can read this folder can read your notes. Set {SECRETS_KEY_FILE_ENV} to a key file kept elsewhere to encrypt that file.",
        secrets_path(data_dir).display()
    ))
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
    let read = wrote
        && store
            .get(&item)
            .ok()
            .flatten()
            .as_deref()
            .map(String::as_str)
            == Some("probe");
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

    proptest! {
        #![proptest_config(ProptestConfig {
            rng_seed: proptest::test_runner::RngSeed::Fixed(0x006b_616e_736f),
            ..ProptestConfig::with_cases(64)
        })]

        /// Invariant: the plain-file warning shows exactly when the file
        /// store has no key file, and then names the file and the variable.
        #[test]
        fn prop_the_file_store_warns_only_without_a_key_file(
            file in any::<bool>(),
            has_key_file in any::<bool>(),
            dir in "[a-z]{1,8}",
        ) {
            let kind = if file { SecretStoreKind::File } else { SecretStoreKind::System };
            let key_file = has_key_file.then(|| Path::new("/keys/k"));
            let warning = file_store_warning(kind, key_file, Path::new(&dir));
            prop_assert_eq!(warning.is_some(), file && !has_key_file);
            if let Some(warning) = warning {
                prop_assert!(warning.contains(SECRETS_FILE));
                prop_assert!(warning.contains(SECRETS_KEY_FILE_ENV));
            }
        }
    }

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

    #[cfg(unix)]
    #[test]
    fn the_data_folder_is_made_owner_only() {
        use std::os::unix::fs::PermissionsExt as _;
        let dir = tempfile::tempdir().unwrap();
        let mode = || std::fs::metadata(dir.path()).unwrap().permissions().mode() & 0o777;
        std::fs::set_permissions(dir.path(), std::fs::Permissions::from_mode(0o755)).unwrap();
        assert!(protect_data_dir(dir.path()).unwrap());
        assert_eq!(mode(), 0o700);
        assert!(!protect_data_dir(dir.path()).unwrap());
        assert_eq!(mode(), 0o700);
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
