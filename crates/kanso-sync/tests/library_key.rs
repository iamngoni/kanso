//! Opening a library with the key the secret store keeps (issue #243 step 9).
//!
//! Desktop only: the phones build stock SQLite without SQLCipher.
#![cfg(any(target_os = "macos", target_os = "windows", target_os = "linux"))]

use kanso_store::{EncryptOptions, LibraryFile, Store, StoreError};
use kanso_sync::keychain::library_key_item;
use kanso_sync::library_key::{is_key_id, key_id_path, load_key};
use kanso_sync::{
    Encryption, MemorySecrets, OpenLibraryError, SecretStore, open_library, open_library_with,
};
use proptest::prelude::*;

fn db(dir: &tempfile::TempDir) -> std::path::PathBuf {
    dir.path().join("kanso.db")
}

fn key_id(dir: &tempfile::TempDir) -> String {
    std::fs::read_to_string(key_id_path(&db(dir))).unwrap()
}

#[test]
fn a_new_library_is_encrypted_and_reopens_with_the_saved_key() {
    let dir = tempfile::tempdir().unwrap();
    let secrets = MemorySecrets::new();
    let opened = open_library(&db(&dir), &secrets).unwrap();
    assert_eq!(opened.encryption, Encryption::Encrypted);
    opened.store.set_meta("hello", "world").unwrap();
    drop(opened);

    assert_eq!(LibraryFile::of(&db(&dir)), LibraryFile::Encrypted);
    let id = key_id(&dir);
    assert!(is_key_id(&id));
    assert_eq!(secrets.items(), vec![library_key_item(&id)]);
    assert!(load_key(&secrets, &id).unwrap().is_some());

    let again = open_library(&db(&dir), &secrets).unwrap();
    assert_eq!(again.encryption, Encryption::Encrypted);
    assert_eq!(again.store.meta("hello").unwrap().as_deref(), Some("world"));
}

#[test]
fn an_existing_plain_library_is_encrypted_at_the_next_open() {
    let dir = tempfile::tempdir().unwrap();
    Store::open(db(&dir)).unwrap().set_meta("a", "b").unwrap();
    let secrets = MemorySecrets::new();
    let opened = open_library(&db(&dir), &secrets).unwrap();
    assert_eq!(opened.encryption, Encryption::Converted);
    assert_eq!(opened.store.meta("a").unwrap().as_deref(), Some("b"));
    drop(opened);
    assert_eq!(LibraryFile::of(&db(&dir)), LibraryFile::Encrypted);
    let id = key_id(&dir);
    assert_eq!(secrets.items(), vec![library_key_item(&id)]);
    // And an ordinary open after.
    let again = open_library(&db(&dir), &secrets).unwrap();
    assert_eq!(again.encryption, Encryption::Encrypted);
    assert_eq!(again.store.meta("a").unwrap().as_deref(), Some("b"));
}

/// A library that stayed plain because no key could be kept is encrypted by
/// the first open that can keep one, not only on the "first launch after the
/// update".
#[test]
fn a_library_left_plain_is_encrypted_once_a_key_can_be_kept() {
    let dir = tempfile::tempdir().unwrap();
    Store::open(db(&dir)).unwrap().set_meta("a", "b").unwrap();
    let secrets = MemorySecrets::new();
    secrets.fail_writes_to(Some("library-key:"));
    for _ in 0..3 {
        let opened = open_library(&db(&dir), &secrets).unwrap();
        assert!(matches!(opened.encryption, Encryption::PlainNoKeychain(_)));
        assert_eq!(opened.store.meta("a").unwrap().as_deref(), Some("b"));
        drop(opened);
        assert_eq!(LibraryFile::of(&db(&dir)), LibraryFile::Plain);
        assert!(secrets.items().is_empty());
        assert!(!key_id_path(&db(&dir)).exists());
    }
    secrets.heal();
    let opened = open_library(&db(&dir), &secrets).unwrap();
    assert_eq!(opened.encryption, Encryption::Converted);
    assert_eq!(opened.store.meta("a").unwrap().as_deref(), Some("b"));
}

/// A disk too small is said plainly, the library opens plain with nothing
/// lost, and a later open with room encrypts it.
#[test]
fn a_disk_too_small_leaves_the_library_plain_and_is_tried_again() {
    let dir = tempfile::tempdir().unwrap();
    {
        let store = Store::open(db(&dir)).unwrap();
        for i in 0..300 {
            store
                .set_meta(&format!("k{i}"), &"filler text ".repeat(100))
                .unwrap();
        }
    }
    let secrets = MemorySecrets::new();
    let small = EncryptOptions {
        max_copy_bytes: Some(16 * 1024),
        ..EncryptOptions::default()
    };
    let opened = open_library_with(&db(&dir), &secrets, &small).unwrap();
    match &opened.encryption {
        Encryption::PlainEncryptFailed(why) => {
            assert!(why.contains("free space"), "{why}");
            assert!(why.contains("untouched"), "{why}");
        }
        other => panic!("{other:?}"),
    }
    assert!(opened.store.meta("k299").unwrap().is_some());
    drop(opened);
    assert_eq!(LibraryFile::of(&db(&dir)), LibraryFile::Plain);
    let roomy = open_library(&db(&dir), &secrets).unwrap();
    assert_eq!(roomy.encryption, Encryption::Converted);
    assert!(roomy.store.meta("k299").unwrap().is_some());
}

/// A conversion a crash stopped is finished by the next open, at every step.
#[test]
fn a_crashed_conversion_is_finished_by_the_next_open() {
    use std::panic::{AssertUnwindSafe, catch_unwind};
    for at in kanso_store::EncryptStep::ALL {
        let dir = tempfile::tempdir().unwrap();
        Store::open(db(&dir)).unwrap().set_meta("a", "b").unwrap();
        let secrets = MemorySecrets::new();
        let hook = move |step: kanso_store::EncryptStep| {
            if step == at {
                panic!("killed");
            }
        };
        let opts = EncryptOptions {
            after_step: Some(&hook),
            ..EncryptOptions::default()
        };
        assert!(
            catch_unwind(AssertUnwindSafe(|| open_library_with(
                &db(&dir),
                &secrets,
                &opts
            )))
            .is_err()
        );
        let opened = open_library(&db(&dir), &secrets).unwrap();
        assert!(
            matches!(
                opened.encryption,
                Encryption::Converted | Encryption::Encrypted
            ),
            "{at:?}: {:?}",
            opened.encryption
        );
        assert_eq!(
            opened.store.meta("a").unwrap().as_deref(),
            Some("b"),
            "{at:?}"
        );
        drop(opened);
        assert_eq!(LibraryFile::of(&db(&dir)), LibraryFile::Encrypted, "{at:?}");
        assert_eq!(secrets.items().len(), 1, "{at:?}");
    }
}

/// The key lost while a conversion was unfinished: the plain library comes
/// back and the next conversion makes a new key.
#[test]
fn a_crashed_conversion_whose_key_is_gone_is_undone_then_redone() {
    use std::panic::{AssertUnwindSafe, catch_unwind};
    let dir = tempfile::tempdir().unwrap();
    Store::open(db(&dir)).unwrap().set_meta("a", "b").unwrap();
    let secrets = MemorySecrets::new();
    let hook = |step: kanso_store::EncryptStep| {
        if step == kanso_store::EncryptStep::Swapped {
            panic!("killed");
        }
    };
    let opts = EncryptOptions {
        after_step: Some(&hook),
        ..EncryptOptions::default()
    };
    assert!(
        catch_unwind(AssertUnwindSafe(|| open_library_with(
            &db(&dir),
            &secrets,
            &opts
        )))
        .is_err()
    );
    let old = key_id(&dir);
    secrets.delete(&library_key_item(&old)).unwrap();
    let opened = open_library(&db(&dir), &secrets).unwrap();
    assert_eq!(opened.encryption, Encryption::Converted);
    assert_eq!(opened.store.meta("a").unwrap().as_deref(), Some("b"));
    drop(opened);
    assert_ne!(key_id(&dir), old);
}

#[test]
fn an_encrypted_library_without_its_key_does_not_open() {
    let dir = tempfile::tempdir().unwrap();
    let secrets = MemorySecrets::new();
    drop(open_library(&db(&dir), &secrets).unwrap());
    let id = key_id(&dir);

    // Another computer: the file, but not the key.
    let elsewhere = MemorySecrets::new();
    let err = open_library(&db(&dir), &elsewhere).err().unwrap();
    assert!(matches!(err, OpenLibraryError::Store(StoreError::WrongKey)));
    assert_eq!(
        err.to_string(),
        "Kanso can't open this library with this computer's key."
    );

    // The key item replaced by another library's key.
    let other = tempfile::tempdir().unwrap();
    drop(open_library(&db(&other), &elsewhere).unwrap());
    let stolen = load_key(&elsewhere, &key_id(&other)).unwrap().unwrap();
    kanso_sync::library_key::save_key(&elsewhere, &id, &stolen).unwrap();
    assert!(matches!(
        open_library(&db(&dir), &elsewhere).err().unwrap(),
        OpenLibraryError::Store(StoreError::WrongKey)
    ));

    // No key ID file.
    std::fs::remove_file(key_id_path(&db(&dir))).unwrap();
    assert!(matches!(
        open_library(&db(&dir), &secrets).err().unwrap(),
        OpenLibraryError::Store(StoreError::WrongKey)
    ));
}

#[test]
fn a_locked_keychain_is_not_taken_for_a_missing_key() {
    let dir = tempfile::tempdir().unwrap();
    let secrets = MemorySecrets::new();
    drop(open_library(&db(&dir), &secrets).unwrap());
    secrets.fail_reads(true);
    let err = open_library(&db(&dir), &secrets).err().unwrap();
    assert!(matches!(err, OpenLibraryError::Keychain(_)), "{err}");
    secrets.fail_reads(false);
    open_library(&db(&dir), &secrets).unwrap();
}

#[test]
fn a_secret_store_that_refuses_gives_a_plain_new_library_and_no_orphans() {
    let dir = tempfile::tempdir().unwrap();
    let secrets = MemorySecrets::new();
    secrets.fail_writes_to(Some("library-key:"));
    let opened = open_library(&db(&dir), &secrets).unwrap();
    assert!(matches!(opened.encryption, Encryption::PlainNoKeychain(_)));
    drop(opened);
    assert_eq!(LibraryFile::of(&db(&dir)), LibraryFile::Plain);
    assert!(secrets.items().is_empty());
    assert!(!key_id_path(&db(&dir)).exists());
}

/// The bug this guards: a library that was there but couldn't be opened
/// (its permissions) read as new, so a fresh key's name replaced its own and
/// was then deleted, leaving it unopenable once the permissions were fixed.
#[cfg(unix)]
#[test]
fn an_unreadable_library_keeps_its_key_name() {
    use std::os::unix::fs::PermissionsExt as _;
    let dir = tempfile::tempdir().unwrap();
    let secrets = MemorySecrets::new();
    let opened = open_library(&db(&dir), &secrets).unwrap();
    opened.store.set_meta("kept", "yes").unwrap();
    drop(opened);
    let id = key_id(&dir);
    let items = secrets.items();

    std::fs::set_permissions(db(&dir), std::fs::Permissions::from_mode(0o000)).unwrap();
    if std::fs::File::open(db(&dir)).is_ok() {
        // Running as root: permissions don't stop the open.
        return;
    }
    assert_eq!(LibraryFile::of(&db(&dir)), LibraryFile::Encrypted);
    assert!(open_library(&db(&dir), &secrets).is_err());
    assert_eq!(key_id(&dir), id);
    assert_eq!(secrets.items(), items);

    std::fs::set_permissions(db(&dir), std::fs::Permissions::from_mode(0o600)).unwrap();
    let again = open_library(&db(&dir), &secrets).unwrap();
    assert_eq!(again.store.meta("kept").unwrap().as_deref(), Some("yes"));
}

/// A create that stopped after naming its key (or whose database file is
/// still empty) reuses that key rather than replacing it.
#[test]
fn a_named_key_without_its_library_is_reused() {
    let dir = tempfile::tempdir().unwrap();
    let secrets = MemorySecrets::new();
    drop(open_library(&db(&dir), &secrets).unwrap());
    let id = key_id(&dir);
    std::fs::remove_file(db(&dir)).unwrap();
    let _ = std::fs::remove_file(dir.path().join("kanso.db-wal"));
    let _ = std::fs::remove_file(dir.path().join("kanso.db-shm"));
    std::fs::write(db(&dir), b"").unwrap();

    let opened = open_library(&db(&dir), &secrets).unwrap();
    assert_eq!(opened.encryption, Encryption::Encrypted);
    drop(opened);
    assert_eq!(key_id(&dir), id);
    assert_eq!(secrets.items(), vec![library_key_item(&id)]);
    assert!(open_library(&db(&dir), &secrets).is_ok());
}

/// A sign-in that moves the secrets to another store takes the library's
/// key along: the library then opens from the new store alone.
#[test]
fn the_key_copies_to_another_store_and_is_forgotten_in_the_old() {
    let dir = tempfile::tempdir().unwrap();
    let system = MemorySecrets::new();
    let file = MemorySecrets::new();
    drop(open_library(&db(&dir), &system).unwrap());
    assert!(matches!(
        open_library(&db(&dir), &file).err().unwrap(),
        OpenLibraryError::Store(StoreError::WrongKey)
    ));
    assert!(kanso_sync::library_key::copy_key(&db(&dir), &system, &file).unwrap());
    kanso_sync::library_key::forget_key(&db(&dir), &system, &file).unwrap();
    assert!(system.items().is_empty());
    // Never the only copy: one store as both, or a store without it.
    kanso_sync::library_key::forget_key(&db(&dir), &file, &file).unwrap();
    kanso_sync::library_key::forget_key(&db(&dir), &file, &system).unwrap();
    assert_eq!(file.items().len(), 1);
    assert_eq!(
        open_library(&db(&dir), &file).unwrap().encryption,
        Encryption::Encrypted
    );

    // A store that refuses the copy is an error, and the key stays put.
    let refusing = MemorySecrets::new();
    refusing.fail_writes_to(Some("library-key:"));
    assert!(matches!(
        kanso_sync::library_key::copy_key(&db(&dir), &file, &refusing),
        Err(OpenLibraryError::Keychain(_))
    ));
    assert!(open_library(&db(&dir), &file).is_ok());

    // A plain library has no key to copy.
    let plain = tempfile::tempdir().unwrap();
    drop(Store::open(db(&plain)).unwrap());
    assert!(!kanso_sync::library_key::copy_key(&db(&plain), &system, &file).unwrap());
}

proptest! {
    #![proptest_config(ProptestConfig { cases: 16, ..ProptestConfig::default() })]

    /// Invariants: each new library gets its own key and key ID; the keys of
    /// other libraries never open it.
    #[test]
    fn prop_one_libraries_key_does_not_open_another(n in 2usize..4) {
        let secrets = MemorySecrets::new();
        let dirs: Vec<_> = (0..n).map(|_| tempfile::tempdir().unwrap()).collect();
        for dir in &dirs {
            drop(open_library(&db(dir), &secrets).unwrap());
        }
        let ids: Vec<String> = dirs.iter().map(key_id).collect();
        let mut sorted = ids.clone();
        sorted.sort();
        sorted.dedup();
        prop_assert_eq!(sorted.len(), n);
        for (i, dir) in dirs.iter().enumerate() {
            let own = load_key(&secrets, &ids[i]).unwrap().unwrap();
            drop(Store::open_with_key(db(dir), Some(&own)).unwrap());
            for (j, id) in ids.iter().enumerate() {
                if i != j {
                    let foreign = load_key(&secrets, id).unwrap().unwrap();
                    prop_assert!(matches!(
                        Store::open_with_key(db(dir), Some(&foreign)),
                        Err(StoreError::WrongKey)
                    ));
                }
            }
        }
        // Reading the items back never needs more than the secret store.
        let _: &dyn SecretStore = &secrets;
    }
}
