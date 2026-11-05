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

/// What every program but the app says when a restore is staged: it leaves
/// the restore alone, because only the app can tell the person it happened.
pub const RESTORE_WAITING: &str = "A restore is waiting; open Kanso to apply it.";

/// Opens the library in `data_dir`, with its key from `secrets`. A staged
/// restore is left alone (and said so on stderr): only the app applies one
/// ([`open_store_applying_restore`]).
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
    if kanso_store::restore_staged(&paths::db_path(data_dir)) {
        eprintln!("{RESTORE_WAITING}");
    }
    open_after_restore(data_dir, secrets, progress)
}

/// [`open_store_reporting`] for the app: a restore staged by the last run
/// replaces the library first, and [`take_restore_outcome`] says what
/// happened, for the toast.
pub fn open_store_applying_restore(
    data_dir: &Path,
    secrets: &dyn SecretStore,
    progress: &(dyn Fn(EncryptProgress) + Sync),
) -> Result<(Store, Option<String>)> {
    if let Some(outcome) = apply_pending_restore(data_dir, secrets) {
        *RESTORE_OUTCOME
            .lock()
            .unwrap_or_else(|poison| poison.into_inner()) = Some(outcome);
    }
    open_after_restore(data_dir, secrets, progress)
}

fn open_after_restore(
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
    // Backups made before encryption are plain copies of the library. Any
    // that can't be encrypted now stay as they are and are tried at the
    // next open; the library opens either way.
    if let Some(key) = opened.store.library_key() {
        match kanso_store::encrypt_backups(&paths::backups_dir(data_dir), key, &opts) {
            Ok(backups) => {
                for (path, why) in &backups.failed {
                    eprintln!(
                        "kanso: couldn't encrypt the backup {}: {why}",
                        path.display()
                    );
                }
            }
            Err(err) => eprintln!("kanso: couldn't encrypt the backups: {err}"),
        }
    }
    Ok((opened.store, plain))
}

/// What a restore staged by Restore from backup… did at this launch: `Ok`
/// when it took the library's place, the reason when it couldn't. Taken once
/// by the app, which says so in its window.
static RESTORE_OUTCOME: std::sync::Mutex<Option<Result<(), RestoreFailure>>> =
    std::sync::Mutex::new(None);

/// Why a staged restore didn't apply, and whether it still waits for the
/// next launch.
pub type RestoreFailure = (String, bool);

pub fn take_restore_outcome() -> Option<Result<(), RestoreFailure>> {
    RESTORE_OUTCOME
        .lock()
        .unwrap_or_else(|poison| poison.into_inner())
        .take()
}

/// Before the library opens: a restore staged by the last run (issue #243
/// step 11) replaces it, and the library it replaces is kept as a backup. A
/// secret store that won't answer leaves the restore waiting for the next
/// launch; one with no key for this library drops it. A restore that can't
/// apply now (the library signed in to sync, in use, no room to keep it)
/// waits too.
fn apply_pending_restore(
    data_dir: &Path,
    secrets: &dyn SecretStore,
) -> Option<Result<(), RestoreFailure>> {
    let db = paths::db_path(data_dir);
    if !kanso_store::restore_staged(&db) || kanso_store::encryption_unfinished(&db) {
        return None;
    }
    let id = std::fs::read_to_string(key_id_path(&db)).unwrap_or_default();
    let outcome = match load_key(secrets, id.trim()) {
        Err(err) => {
            eprintln!(
                "kanso: a restore is waiting, but the system keychain wouldn't answer: {err}"
            );
            return None;
        }
        Ok(None) => {
            kanso_store::cancel_staged_restore(&db);
            Err("this computer has no key for the library".to_owned())
        }
        Ok(Some(key)) => match signed_in_now(&db, &key, secrets) {
            // The keychain wouldn't say: wait, as above.
            Err(err) => {
                eprintln!(
                    "kanso: a restore is waiting, but the system keychain wouldn't say whether the library is signed in: {err}"
                );
                return None;
            }
            Ok(true) => Err(crate::library_key_model::RESTORE_SIGNED_IN_AT_LAUNCH.to_owned()),
            Ok(false) => kanso_store::apply_staged_restore(
                &db,
                &key,
                &paths::backups_dir(data_dir),
                kanso_core::now_ms(),
            )
            .map(drop)
            .map_err(|err| err.to_string()),
        },
    };
    match &outcome {
        Ok(()) => eprintln!("kanso: restored the library from a backup"),
        Err(why) => eprintln!("kanso: couldn't restore the backup: {why}"),
    }
    Some(outcome.map_err(|why| (why, kanso_store::restore_staged(&db))))
}

/// Whether the library at `db` holds a sync session in `secrets`, signed in
/// (`sync-account`) or with a sign-in waiting on its key (`sync-pending`).
/// The restore sheet refuses to stage while signed in, but a sign-in can
/// still happen before the next launch (the Account page, `kanso account
/// sign-in`); applied then, the restored notes would sync before anyone was
/// asked. So the restore waits until the library is signed out.
fn signed_in_now(
    db: &Path,
    key: &kanso_store::LibraryKey,
    secrets: &dyn SecretStore,
) -> std::result::Result<bool, String> {
    if LibraryFile::of(db) == LibraryFile::New {
        return Ok(false);
    }
    let library_id = Store::open_read_only_with_key(db, Some(key))
        .and_then(|store| store.meta(kanso_sync::meta_keys::LIBRARY_ID))
        .map_err(|err| err.to_string())?;
    let Some(library_id) = library_id else {
        // Never had an account action: no item can name it.
        return Ok(false);
    };
    for item in [
        kanso_sync::keychain::account_item(&library_id),
        kanso_sync::keychain::pending_item(&library_id),
    ] {
        if secrets.get(&item).map_err(|err| err.to_string())?.is_some() {
            return Ok(true);
        }
    }
    Ok(false)
}

/// Opens the library in `data_dir`, with the key from the secret store the
/// config names. For `capture`, `serve`, `sticky` and the importers: a staged
/// restore is left for the app.
pub fn open_store(data_dir: &Path) -> Result<Store> {
    let reporter = crate::encrypt_progress::Reporter::stderr();
    Ok(
        open_with_config_secrets(data_dir, open_store_reporting, &|progress| {
            reporter.report(progress)
        })?
        .0,
    )
}

/// [`open_store_applying_restore`] with the secret store the config names,
/// telling `progress` how far an encryption has got. The app's own open,
/// and the only way to a staged restore: `restore_waiting_tests` checks that
/// nothing but the app's launch (`main.rs`) calls it.
pub fn open_store_progress(
    data_dir: &Path,
    progress: &(dyn Fn(EncryptProgress) + Sync),
) -> Result<(Store, Option<String>)> {
    open_with_config_secrets(data_dir, open_store_applying_restore, progress)
}

/// One of the opens above, given the secret store.
type Opener = fn(
    &Path,
    &dyn SecretStore,
    &(dyn Fn(EncryptProgress) + Sync),
) -> Result<(Store, Option<String>)>;

/// The one in-memory secret store the tests' opens share.
#[cfg(test)]
pub(crate) fn test_secrets() -> &'static kanso_sync::MemorySecrets {
    static SECRETS: std::sync::OnceLock<kanso_sync::MemorySecrets> = std::sync::OnceLock::new();
    SECRETS.get_or_init(kanso_sync::MemorySecrets::new)
}

fn open_with_config_secrets(
    data_dir: &Path,
    open: Opener,
    progress: &(dyn Fn(EncryptProgress) + Sync),
) -> Result<(Store, Option<String>)> {
    // Tests never touch the system's keychain: one in-memory store stands in.
    #[cfg(test)]
    {
        open(data_dir, test_secrets(), progress)
    }
    #[cfg(not(test))]
    {
        let secrets = secrets_for(data_dir);
        open(data_dir, secrets.as_ref(), progress)
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

    /// A restore staged by the last run replaces the library before it opens,
    /// keeps the library it replaces as a backup, and a restore with no key
    /// to apply it is dropped rather than tried at every launch.
    #[test]
    fn a_staged_restore_replaces_the_library_at_the_next_open() {
        let dir = tempfile::tempdir().unwrap();
        let db = paths::db_path(dir.path());
        let secrets = kanso_sync::MemorySecrets::new();
        let (store, _) = open_store_reporting(dir.path(), &secrets, &|_| {}).unwrap();
        store.set_meta("kept", "yes").unwrap();
        let backup = store
            .backup_now(&paths::backups_dir(dir.path()), 1_000)
            .unwrap();
        store.set_meta("later", "yes").unwrap();
        let key = store.library_key().unwrap().clone();
        drop(store);
        kanso_store::stage_restore(&db, &backup, Some(&key), &key).unwrap();

        assert_eq!(apply_pending_restore(dir.path(), &secrets), Some(Ok(())));
        let (store, _) = open_store_reporting(dir.path(), &secrets, &|_| {}).unwrap();
        assert_eq!(store.meta("kept").unwrap().as_deref(), Some("yes"));
        assert_eq!(store.meta("later").unwrap(), None);
        // Nothing is left to apply twice.
        assert_eq!(apply_pending_restore(dir.path(), &secrets), None);
        let kept = std::fs::read_dir(paths::backups_dir(dir.path()))
            .unwrap()
            .flatten()
            .filter(|entry| {
                entry
                    .file_name()
                    .to_string_lossy()
                    .starts_with("kanso-manual-")
            })
            .count();
        assert_eq!(kept, 2, "the backup and the library it replaced");
        drop(store);

        // A computer with no key for the library drops the staged copy.
        std::fs::write(kanso_store::staged_restore_path(&db), b"x").unwrap();
        let empty = kanso_sync::MemorySecrets::new();
        assert!(matches!(
            apply_pending_restore(dir.path(), &empty),
            Some(Err(_))
        ));
        assert!(!kanso_store::restore_staged(&db));
    }

    /// Regression: a restore that can't apply at this launch (here the
    /// backups folder is a file, so the library can't be kept) used to be
    /// dropped. It waits now, says it will be tried again, and the library
    /// opens as it was.
    #[test]
    fn a_restore_that_cannot_apply_now_waits_and_the_library_opens() {
        let dir = tempfile::tempdir().unwrap();
        let db = paths::db_path(dir.path());
        let secrets = kanso_sync::MemorySecrets::new();
        let (store, _) = open_store_reporting(dir.path(), &secrets, &|_| {}).unwrap();
        let elsewhere = tempfile::tempdir().unwrap();
        let backup = store.backup_now(elsewhere.path(), 1_000).unwrap();
        store.set_meta("later", "yes").unwrap();
        let key = store.library_key().unwrap().clone();
        drop(store);
        kanso_store::stage_restore(&db, &backup, Some(&key), &key).unwrap();
        std::fs::write(paths::backups_dir(dir.path()), b"not a folder").unwrap();

        assert!(matches!(
            apply_pending_restore(dir.path(), &secrets),
            Some(Err((_, true)))
        ));
        assert!(kanso_store::restore_staged(&db));
        let (store, _) = open_store_reporting(dir.path(), &secrets, &|_| {}).unwrap();
        assert_eq!(store.meta("later").unwrap().as_deref(), Some("yes"));
    }

    /// A backups folder that can't be read doesn't keep the library from
    /// opening (encrypting the backups used to be able to fail the open).
    #[cfg(unix)]
    #[test]
    fn an_unreadable_backups_folder_does_not_keep_the_library_shut() {
        use std::os::unix::fs::PermissionsExt as _;
        let dir = tempfile::tempdir().unwrap();
        let db = paths::db_path(dir.path());
        Store::open(&db).unwrap().set_meta("a", "b").unwrap();
        let backups = paths::backups_dir(dir.path());
        std::fs::create_dir_all(&backups).unwrap();
        std::fs::set_permissions(&backups, std::fs::Permissions::from_mode(0o000)).unwrap();
        let secrets = kanso_sync::MemorySecrets::new();
        let opened = open_store_reporting(dir.path(), &secrets, &|_| {});
        std::fs::set_permissions(&backups, std::fs::Permissions::from_mode(0o755)).unwrap();
        let (store, _) = opened.unwrap();
        assert!(store.is_encrypted());
        assert_eq!(store.meta("a").unwrap().as_deref(), Some("b"));
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
