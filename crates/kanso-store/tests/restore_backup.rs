//! Restoring a backup (issue #243 step 11): a backup opens with the key it
//! was made with, this computer's or the 24 words from another, and takes
//! the library's place at the next launch. Seeds are pinned.

use std::path::{Path, PathBuf};

use kanso_core::DocType;
use kanso_store::{
    KEY_BYTES, LibraryFile, LibraryKey, Origin, Put, Store, StoreError, apply_staged_restore,
    check_backup, restore_staged, stage_restore,
};
use proptest::prelude::*;
use proptest::test_runner::RngSeed;
use serde_json::json;

fn key_from(seed: u8) -> LibraryKey {
    let mut bytes = [seed; KEY_BYTES];
    bytes[0] = seed.wrapping_mul(7).wrapping_add(1);
    LibraryKey::from_bytes(bytes)
}

fn db_path(dir: &Path) -> PathBuf {
    dir.join("kanso.db")
}

fn add_note(store: &Store, title: &str) {
    let book = store
        .put(
            DocType::Book,
            json!({"name": "Notes"}),
            Put::Upsert,
            Origin::Api,
        )
        .unwrap()
        .id;
    store
        .put(
            DocType::Note,
            json!({"bookId": book, "title": title, "body": "text"}),
            Put::Upsert,
            Origin::Api,
        )
        .unwrap();
}

/// The library at `dir` with `notes`, encrypted with `key`.
fn library(dir: &Path, key: &LibraryKey, notes: &[&str]) {
    let store = Store::open_with_key(db_path(dir), Some(key)).unwrap();
    for note in notes {
        store.set_meta(&format!("note.{note}"), "yes").unwrap();
    }
    if let Some(first) = notes.first() {
        add_note(&store, first);
    }
}

fn marks(db: &Path, key: &LibraryKey) -> Vec<String> {
    let store = Store::open_with_key(db, Some(key)).unwrap();
    let mut found: Vec<String> = ["a", "b", "c", "d", "e", "f"]
        .iter()
        .filter(|name| store.meta(&format!("note.{name}")).unwrap().is_some())
        .map(|name| (*name).to_owned())
        .collect();
    found.sort();
    found
}

#[test]
fn a_backup_made_here_takes_the_librarys_place_and_the_old_library_is_kept() {
    let dir = tempfile::tempdir().unwrap();
    let key = key_from(1);
    let db = db_path(dir.path());
    library(dir.path(), &key, &["a", "b"]);
    let backups = dir.path().join("backups");
    let backup = Store::open_with_key(&db, Some(&key))
        .unwrap()
        .backup_now(&backups, 1_000)
        .unwrap();
    Store::open_with_key(&db, Some(&key))
        .unwrap()
        .set_meta("note.c", "yes")
        .unwrap();

    let before = std::fs::read(&backup).unwrap();
    check_backup(&backup, Some(&key)).unwrap();
    stage_restore(&db, &backup, Some(&key), &key).unwrap();
    // The backup is only read.
    assert_eq!(std::fs::read(&backup).unwrap(), before);
    assert!(restore_staged(&db));
    // Nothing has changed yet.
    assert_eq!(marks(&db, &key), ["a", "b", "c"]);

    assert!(apply_staged_restore(&db, &key, &backups, 2_000).unwrap());
    assert!(!restore_staged(&db));
    assert_eq!(marks(&db, &key), ["a", "b"]);
    // The library it replaced is a backup of its own, with the later note.
    let kept = backups.join("kanso-manual-2000.db");
    assert_eq!(marks(&kept, &key), ["a", "b", "c"]);
    // Nothing is left to apply twice.
    assert!(!apply_staged_restore(&db, &key, &backups, 3_000).unwrap());
}

#[test]
fn a_backup_from_another_computer_opens_with_its_words() {
    let there = tempfile::tempdir().unwrap();
    let here = tempfile::tempdir().unwrap();
    let theirs = key_from(2);
    let ours = key_from(3);
    library(there.path(), &theirs, &["a", "b", "d"]);
    let backup = Store::open_with_key(db_path(there.path()), Some(&theirs))
        .unwrap()
        .backup_now(&there.path().join("backups"), 5)
        .unwrap();
    library(here.path(), &ours, &["f"]);
    let db = db_path(here.path());

    // This computer's key doesn't open it; the words do.
    assert!(matches!(
        check_backup(&backup, Some(&ours)),
        Err(StoreError::WrongKey)
    ));
    let words = theirs.to_words();
    let from_words = LibraryKey::from_words(&words).unwrap();
    check_backup(&backup, Some(&from_words)).unwrap();
    stage_restore(&db, &backup, Some(&from_words), &ours).unwrap();
    apply_staged_restore(&db, &ours, &here.path().join("backups"), 9).unwrap();

    // The notes came; the library is encrypted with this computer's key.
    assert_eq!(marks(&db, &ours), ["a", "b", "d"]);
    assert!(matches!(
        Store::open_with_key(&db, Some(&theirs)),
        Err(StoreError::WrongKey)
    ));
}

#[test]
fn a_wrong_key_or_a_file_that_is_not_a_backup_stages_nothing() {
    let dir = tempfile::tempdir().unwrap();
    let key = key_from(4);
    let db = db_path(dir.path());
    library(dir.path(), &key, &["a"]);
    let backup = Store::open_with_key(&db, Some(&key))
        .unwrap()
        .backup_now(&dir.path().join("backups"), 1)
        .unwrap();

    let wrong = key_from(5);
    assert!(matches!(
        stage_restore(&db, &backup, Some(&wrong), &key),
        Err(StoreError::WrongKey)
    ));
    assert!(matches!(
        stage_restore(&db, &backup, None, &key),
        Err(StoreError::WrongKey)
    ));
    let junk = dir.path().join("junk.db");
    std::fs::write(&junk, b"not a database at all, but longer than a header").unwrap();
    assert!(check_backup(&junk, Some(&key)).is_err());
    assert!(check_backup(&junk, None).is_err());
    let empty = dir.path().join("empty.db");
    std::fs::write(&empty, b"").unwrap();
    assert!(matches!(
        check_backup(&empty, None),
        Err(StoreError::Invalid(_))
    ));
    assert!(!restore_staged(&db));
    assert!(
        std::fs::read_dir(dir.path())
            .unwrap()
            .flatten()
            .all(|entry| !entry.file_name().to_string_lossy().contains("restore"))
    );
    assert_eq!(marks(&db, &key), ["a"]);
}

#[test]
fn a_plain_backup_restores_into_an_encrypted_library() {
    let dir = tempfile::tempdir().unwrap();
    let key = key_from(6);
    let db = db_path(dir.path());
    library(dir.path(), &key, &["f"]);
    let plain = dir.path().join("old.db");
    {
        let old = Store::open(&plain).unwrap();
        old.set_meta("note.a", "yes").unwrap();
    }
    assert_eq!(LibraryFile::of(&plain), LibraryFile::Plain);
    check_backup(&plain, None).unwrap();
    stage_restore(&db, &plain, None, &key).unwrap();
    apply_staged_restore(&db, &key, &dir.path().join("backups"), 7).unwrap();
    assert_eq!(marks(&db, &key), ["a"]);
    assert_eq!(LibraryFile::of(&db), LibraryFile::Encrypted);
}

/// A crash between keeping the old library and renaming the copy leaves no
/// library and a staged copy; the next launch finishes the job.
#[test]
fn a_crash_between_the_two_renames_is_finished_at_the_next_launch() {
    let dir = tempfile::tempdir().unwrap();
    let key = key_from(7);
    let db = db_path(dir.path());
    library(dir.path(), &key, &["a"]);
    let backups = dir.path().join("backups");
    let backup = Store::open_with_key(&db, Some(&key))
        .unwrap()
        .backup_now(&backups, 1)
        .unwrap();
    stage_restore(&db, &backup, Some(&key), &key).unwrap();
    std::fs::remove_file(&db).unwrap();
    assert!(apply_staged_restore(&db, &key, &backups, 2).unwrap());
    assert_eq!(marks(&db, &key), ["a"]);
}

/// A staged copy that has gone bad is dropped, so Kanso still starts, and
/// the library is as it was.
#[test]
fn a_bad_staged_copy_is_dropped_and_the_library_is_left_alone() {
    let dir = tempfile::tempdir().unwrap();
    let key = key_from(8);
    let db = db_path(dir.path());
    library(dir.path(), &key, &["a", "b"]);
    std::fs::write(kanso_store::staged_restore_path(&db), vec![7u8; 8192]).unwrap();
    assert!(apply_staged_restore(&db, &key, &dir.path().join("backups"), 1).is_err());
    assert!(!restore_staged(&db));
    assert_eq!(marks(&db, &key), ["a", "b"]);
}

proptest! {
    #![proptest_config(ProptestConfig {
        cases: 12,
        rng_seed: RngSeed::Fixed(0x006b_616e_736f),
        failure_persistence: None,
        ..ProptestConfig::default()
    })]

    /// Invariants: whichever notes the backup and the library hold and
    /// whichever keys they have, restoring leaves exactly the backup's notes
    /// in a library only this computer's key opens, and keeps the old one.
    #[test]
    fn prop_a_restore_leaves_the_backups_notes_under_this_computers_key(
        theirs in 1u8..=100,
        ours in 101u8..=200,
        in_backup in proptest::sample::subsequence(vec!["a", "b", "c"], 0..=3),
        in_library in proptest::sample::subsequence(vec!["d", "e", "f"], 0..=3),
    ) {
        let there = tempfile::tempdir().unwrap();
        let here = tempfile::tempdir().unwrap();
        let theirs = key_from(theirs);
        let ours = key_from(ours);
        library(there.path(), &theirs, &in_backup);
        library(here.path(), &ours, &in_library);
        let backup = Store::open_with_key(db_path(there.path()), Some(&theirs))
            .unwrap()
            .backup_now(&there.path().join("backups"), 1)
            .unwrap();
        let words = LibraryKey::from_words(&theirs.to_words()).unwrap();
        let db = db_path(here.path());
        let keep = here.path().join("backups");
        stage_restore(&db, &backup, Some(&words), &ours).unwrap();
        apply_staged_restore(&db, &ours, &keep, 9).unwrap();
        prop_assert_eq!(marks(&db, &ours), in_backup.iter().map(|s| (*s).to_owned()).collect::<Vec<_>>());
        prop_assert!(Store::open_with_key(&db, Some(&theirs)).is_err());
        prop_assert_eq!(
            marks(&keep.join("kanso-manual-9.db"), &ours),
            in_library.iter().map(|s| (*s).to_owned()).collect::<Vec<_>>()
        );
    }
}
