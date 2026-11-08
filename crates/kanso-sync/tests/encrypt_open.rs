//! Encrypting an existing library through `open_library`, the way every
//! program opens it (issue #243 step 10): a crash at any step leaves no
//! plaintext once the next open is done, a lost key never strands the
//! library, and a state Kanso can't sort out says so instead of "open Kanso
//! to let it finish".
//!
//! Desktop only: the phones build stock SQLite without SQLCipher.
#![cfg(any(target_os = "macos", target_os = "windows", target_os = "linux"))]

use std::panic::{AssertUnwindSafe, catch_unwind};
use std::path::{Path, PathBuf};

use kanso_store::{EncryptOptions, EncryptStep, LibraryFile, Store, StoreError};
use kanso_sync::keychain::library_key_item;
use kanso_sync::{
    Encryption, MemorySecrets, OpenLibraryError, SecretStore as _, open_library, open_library_with,
};

const CANARY: &str = "zqxcanary-77e1-copper-meadow";

fn db(dir: &Path) -> PathBuf {
    dir.join("kanso.db")
}

fn beside(db: &Path, suffix: &str) -> PathBuf {
    let mut name = db.as_os_str().to_owned();
    name.push(suffix);
    PathBuf::from(name)
}

fn key_id(dir: &Path) -> String {
    std::fs::read_to_string(beside(&db(dir), ".keyid"))
        .unwrap()
        .trim()
        .to_owned()
}

/// A plain library holding the canary.
fn plain_library(dir: &Path) {
    let store = Store::open(db(dir)).unwrap();
    store.set_meta("secret", CANARY).unwrap();
    store.set_meta("a", "b").unwrap();
}

/// Opens with a hook that panics after `at`, as a kill would stop it.
fn crash_open_at(dir: &Path, secrets: &MemorySecrets, at: EncryptStep) {
    let hook = move |step: EncryptStep| {
        if step == at {
            panic!("killed after {step:?}");
        }
    };
    let opts = EncryptOptions {
        after_step: Some(&hook),
        ..EncryptOptions::default()
    };
    let crashed = catch_unwind(AssertUnwindSafe(|| {
        open_library_with(&db(dir), secrets, &opts)
    }));
    assert!(crashed.is_err(), "the open did not stop at {at:?}");
}

/// Every file in `dir` and below that holds the canary.
fn files_with_canary(dir: &Path) -> Vec<PathBuf> {
    let mut found = Vec::new();
    let mut stack = vec![dir.to_path_buf()];
    while let Some(next) = stack.pop() {
        for entry in std::fs::read_dir(next).unwrap().flatten() {
            let path = entry.path();
            if path.is_dir() {
                stack.push(path);
            } else if let Ok(bytes) = std::fs::read(&path)
                && bytes.windows(CANARY.len()).any(|w| w == CANARY.as_bytes())
            {
                found.push(path);
            }
        }
    }
    found
}

fn leftovers(dir: &Path) -> Vec<String> {
    std::fs::read_dir(dir)
        .unwrap()
        .flatten()
        .map(|e| e.file_name().to_string_lossy().into_owned())
        .filter(|name| {
            name.ends_with(".plain") || name.ends_with(".enc") || name.ends_with(".encrypting")
        })
        .collect()
}

/// Whatever step a crash stopped at, the next open finishes the job: the
/// library is encrypted, and no plaintext or marker is left anywhere in the
/// data folder. (A crash after the confirmation used to leave the plain
/// library's second name for good, since an encrypted library opened
/// without looking at the marker.)
#[test]
fn after_a_crash_at_any_step_the_next_open_leaves_no_plaintext() {
    for at in EncryptStep::ALL {
        let dir = tempfile::tempdir().unwrap();
        plain_library(dir.path());
        assert!(!files_with_canary(dir.path()).is_empty());
        let secrets = MemorySecrets::new();
        crash_open_at(dir.path(), &secrets, at);

        let opened = open_library(&db(dir.path()), &secrets).unwrap();
        assert_eq!(
            opened.store.meta("secret").unwrap().as_deref(),
            Some(CANARY),
            "{at:?}"
        );
        drop(opened);
        assert_eq!(
            LibraryFile::of(&db(dir.path())),
            LibraryFile::Encrypted,
            "{at:?}"
        );
        assert_eq!(leftovers(dir.path()), Vec::<String>::new(), "{at:?}");
        assert_eq!(
            files_with_canary(dir.path()),
            Vec::<PathBuf>::new(),
            "{at:?}"
        );
    }
}

/// The key gone after the encrypted library was confirmed, with the plain
/// one still beside it: the library opens (the plain one back, encrypted
/// again with a new key) instead of being stranded behind "can't open this
/// library with this computer's key".
#[test]
fn a_key_lost_after_the_confirmation_does_not_strand_the_library() {
    let dir = tempfile::tempdir().unwrap();
    plain_library(dir.path());
    let secrets = MemorySecrets::new();
    crash_open_at(dir.path(), &secrets, EncryptStep::Confirmed);
    let old = key_id(dir.path());
    secrets.delete(&library_key_item(&old)).unwrap();

    let opened = open_library(&db(dir.path()), &secrets).unwrap();
    assert_eq!(opened.encryption, Encryption::Converted);
    assert_eq!(
        opened.store.meta("secret").unwrap().as_deref(),
        Some(CANARY)
    );
    drop(opened);
    assert_ne!(key_id(dir.path()), old);
    assert_eq!(leftovers(dir.path()), Vec::<String>::new());
    // The old encrypted file is kept aside, and holds no plaintext.
    assert!(beside(&db(dir.path()), ".no-key").exists());
    assert_eq!(files_with_canary(dir.path()), Vec::<PathBuf>::new());
}

/// A stale marker beside an encrypted library never locks it out.
#[test]
fn a_stale_marker_beside_an_encrypted_library_does_not_lock_it_out() {
    let dir = tempfile::tempdir().unwrap();
    plain_library(dir.path());
    let secrets = MemorySecrets::new();
    drop(open_library(&db(dir.path()), &secrets).unwrap());
    std::fs::write(
        beside(&db(dir.path()), ".encrypting"),
        r#"{"v":1,"phase":"exporting"}"#,
    )
    .unwrap();
    let opened = open_library(&db(dir.path()), &secrets).unwrap();
    assert_eq!(opened.store.meta("a").unwrap().as_deref(), Some("b"));
    drop(opened);
    assert_eq!(leftovers(dir.path()), Vec::<String>::new());
}

/// A state Kanso can't sort out is said, with what to do, rather than the
/// "Kanso is encrypting this library… Open Kanso to let it finish" that
/// opening Kanso can never get past.
#[test]
fn a_state_kanso_cannot_sort_out_says_what_to_do() {
    let dir = tempfile::tempdir().unwrap();
    plain_library(dir.path());
    let secrets = MemorySecrets::new();
    drop(open_library(&db(dir.path()), &secrets).unwrap());
    // The library's name gone, and a marker from a conversion.
    std::fs::rename(db(dir.path()), beside(&db(dir.path()), ".enc")).unwrap();
    std::fs::write(
        beside(&db(dir.path()), ".encrypting"),
        r#"{"v":1,"phase":"exporting"}"#,
    )
    .unwrap();
    let err = match open_library(&db(dir.path()), &secrets) {
        Ok(_) => panic!("opened a library that isn't there"),
        Err(err) => err,
    };
    assert!(
        !matches!(err, OpenLibraryError::Store(StoreError::Converting)),
        "{err}"
    );
    let text = err.to_string();
    assert!(text.contains("kanso.db.encrypting"), "{text}");
    assert!(text.contains("remove"), "{text}");
    // Nothing was deleted.
    assert!(beside(&db(dir.path()), ".enc").exists());
}
