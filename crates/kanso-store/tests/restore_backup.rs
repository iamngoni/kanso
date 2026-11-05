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

/// The files in `dir` and below, with their bytes, for "nothing changed".
fn snapshot(dir: &Path) -> Vec<(PathBuf, Vec<u8>)> {
    let mut found = Vec::new();
    let mut stack = vec![dir.to_path_buf()];
    while let Some(at) = stack.pop() {
        for entry in std::fs::read_dir(&at).unwrap().flatten() {
            let path = entry.path();
            if path.is_dir() {
                stack.push(path);
            } else {
                found.push((path.clone(), std::fs::read(&path).unwrap()));
            }
        }
    }
    found.sort();
    found
}

/// A library and a backup of it in `dir`, the library with a later note.
fn library_with_backup(dir: &Path, key: &LibraryKey) -> PathBuf {
    let db = db_path(dir);
    library(dir, key, &["a", "b"]);
    let backup = Store::open_with_key(&db, Some(key))
        .unwrap()
        .backup_now(&dir.join("backups"), 1_000)
        .unwrap();
    Store::open_with_key(&db, Some(key))
        .unwrap()
        .set_meta("note.c", "yes")
        .unwrap();
    backup
}

/// Wrong words, a plain backup taken for an encrypted one, and a file that
/// isn't a backup leave every file in the data folder as it was, the backup
/// included: no staged copy, no private copy, no side files.
#[test]
fn a_failed_restore_leaves_every_file_as_it_was() {
    let dir = tempfile::tempdir().unwrap();
    let key = key_from(21);
    let db = db_path(dir.path());
    let backup = library_with_backup(dir.path(), &key);
    let junk = dir.path().join("junk.db");
    std::fs::write(&junk, vec![3u8; 8192]).unwrap();
    let before = snapshot(dir.path());
    let typo = LibraryKey::from_words(&key_from(22).to_words()).unwrap();
    assert!(matches!(
        stage_restore(&db, &backup, Some(&typo), &key),
        Err(StoreError::WrongKey)
    ));
    assert!(stage_restore(&db, &junk, None, &key).is_err());
    assert!(stage_restore(&db, &junk, Some(&key), &key).is_err());
    assert_eq!(snapshot(dir.path()), before);
}

/// A backup on a read-only disk (or in a folder that can't be written) can
/// be restored: only a private copy of it is ever opened for writing.
#[cfg(unix)]
#[test]
fn a_read_only_backup_can_be_restored() {
    use std::os::unix::fs::PermissionsExt as _;
    let dir = tempfile::tempdir().unwrap();
    let key = key_from(23);
    let db = db_path(dir.path());
    let backup = library_with_backup(dir.path(), &key);
    let shelf = tempfile::tempdir().unwrap();
    let on_shelf = shelf.path().join("kanso-manual-1000.db");
    std::fs::copy(&backup, &on_shelf).unwrap();
    std::fs::set_permissions(&on_shelf, std::fs::Permissions::from_mode(0o444)).unwrap();
    std::fs::set_permissions(shelf.path(), std::fs::Permissions::from_mode(0o555)).unwrap();
    let staged = stage_restore(&db, &on_shelf, Some(&key), &key);
    std::fs::set_permissions(shelf.path(), std::fs::Permissions::from_mode(0o755)).unwrap();
    staged.unwrap();
    assert!(apply_staged_restore(&db, &key, &dir.path().join("backups"), 2_000).unwrap());
    assert_eq!(marks(&db, &key), ["a", "b"]);
}

/// Regression: a restore that can't keep the library it replaces (here the
/// backups folder is a file) leaves the library and the staged copy as they
/// are, for the next launch. It used to delete the staged copy, and a
/// failure after the old library was moved aside left no library at all.
#[test]
fn a_restore_that_cannot_keep_the_library_waits_for_the_next_launch() {
    let dir = tempfile::tempdir().unwrap();
    let key = key_from(24);
    let db = db_path(dir.path());
    let backup = library_with_backup(dir.path(), &key);
    stage_restore(&db, &backup, Some(&key), &key).unwrap();
    let blocked = dir.path().join("not-a-folder");
    std::fs::write(&blocked, b"x").unwrap();
    assert!(apply_staged_restore(&db, &key, &blocked, 2_000).is_err());
    assert!(restore_staged(&db), "the restore waits for the next launch");
    assert_eq!(marks(&db, &key), ["a", "b", "c"]);
    // The next launch, with somewhere to keep it, finishes.
    let backups = dir.path().join("backups");
    assert!(apply_staged_restore(&db, &key, &backups, 3_000).unwrap());
    assert_eq!(marks(&db, &key), ["a", "b"]);
    assert_eq!(
        marks(&backups.join("kanso-manual-3000.db"), &key),
        ["a", "b", "c"]
    );
}

/// Regression: a library another program has open (a reader holding the
/// log) isn't swapped, and the restore waits rather than being dropped.
#[test]
fn a_library_in_use_keeps_the_restore_waiting() {
    let dir = tempfile::tempdir().unwrap();
    let key = key_from(25);
    let db = db_path(dir.path());
    let backup = library_with_backup(dir.path(), &key);
    stage_restore(&db, &backup, Some(&key), &key).unwrap();
    let writer = Store::open_with_key(&db, Some(&key)).unwrap();
    writer.set_meta("note.d", "yes").unwrap();
    let reader = rusqlite::Connection::open(&db).unwrap();
    reader
        .execute_batch(&format!(
            "PRAGMA key = \"x'{}'\"; BEGIN; SELECT count(*) FROM meta;",
            key.to_hex().as_str()
        ))
        .unwrap();
    assert!(apply_staged_restore(&db, &key, &dir.path().join("backups"), 2_000).is_err());
    assert!(restore_staged(&db));
    drop(reader);
    drop(writer);
    assert_eq!(marks(&db, &key), ["a", "b", "c", "d"]);
    assert!(apply_staged_restore(&db, &key, &dir.path().join("backups"), 3_000).unwrap());
    assert_eq!(marks(&db, &key), ["a", "b"]);
}

/// The library is kept under a name no backup has: one made in the same
/// millisecond is left alone.
#[test]
fn the_kept_library_never_replaces_a_backup() {
    let dir = tempfile::tempdir().unwrap();
    let key = key_from(26);
    let db = db_path(dir.path());
    let backup = library_with_backup(dir.path(), &key);
    let backups = dir.path().join("backups");
    let taken = backups.join("kanso-manual-2000.db");
    std::fs::copy(&backup, &taken).unwrap();
    let before = std::fs::read(&taken).unwrap();
    stage_restore(&db, &backup, Some(&key), &key).unwrap();
    assert!(apply_staged_restore(&db, &key, &backups, 2_000).unwrap());
    assert_eq!(std::fs::read(&taken).unwrap(), before);
    assert_eq!(
        marks(&backups.join("kanso-manual-2001.db"), &key),
        ["a", "b", "c"]
    );
}

/// A crash after the library was kept and before the swap: the library is
/// still in place, and the next launch keeps it again and swaps. Nothing is
/// lost, and the library was never missing.
#[test]
fn a_crash_after_keeping_the_library_is_finished_at_the_next_launch() {
    let dir = tempfile::tempdir().unwrap();
    let key = key_from(27);
    let db = db_path(dir.path());
    let backup = library_with_backup(dir.path(), &key);
    let backups = dir.path().join("backups");
    stage_restore(&db, &backup, Some(&key), &key).unwrap();
    // What the first try left: the library kept under a second name.
    std::fs::hard_link(&db, backups.join("kanso-manual-2000.db")).unwrap();
    assert_eq!(marks(&db, &key), ["a", "b", "c"]);
    assert!(apply_staged_restore(&db, &key, &backups, 2_000).unwrap());
    assert_eq!(marks(&db, &key), ["a", "b"]);
    assert_eq!(
        marks(&backups.join("kanso-manual-2000.db"), &key),
        ["a", "b", "c"]
    );
}

fn api_key(store: &Store, id: &str, hash: u8) {
    store
        .api_key_insert(&kanso_store::NewApiKey {
            id: id.to_owned(),
            name: id.to_owned(),
            client: "other".to_owned(),
            token_hash: [hash; 32],
            access: kanso_store::KeyAccess::Write,
            books: None,
            created_at: 1,
        })
        .unwrap();
}

fn live_keys(db: &Path, key: &LibraryKey) -> Vec<String> {
    let mut ids: Vec<String> = Store::open_with_key(db, Some(key))
        .unwrap()
        .api_keys_live()
        .unwrap()
        .into_iter()
        .map(|live| live.row.id)
        .collect();
    ids.sort();
    ids
}

/// Regression: restoring a backup brought back the API keys it held, so a
/// key revoked after the backup was made worked again. Keys never travel
/// with a restore: the library keeps its own, revoked ones revoked and new
/// ones live.
#[test]
fn a_revoked_api_key_stays_revoked_after_a_restore() {
    let dir = tempfile::tempdir().unwrap();
    let key = key_from(28);
    let db = db_path(dir.path());
    library(dir.path(), &key, &["a"]);
    let backups = dir.path().join("backups");
    let backup = {
        let store = Store::open_with_key(&db, Some(&key)).unwrap();
        api_key(&store, "leaked", 1);
        let backup = store.backup_now(&backups, 1_000).unwrap();
        assert!(store.api_key_revoke("leaked", 2_000).unwrap());
        api_key(&store, "made-later", 2);
        backup
    };
    stage_restore(&db, &backup, Some(&key), &key).unwrap();
    assert!(apply_staged_restore(&db, &key, &backups, 3_000).unwrap());
    assert_eq!(marks(&db, &key), ["a"]);
    assert_eq!(live_keys(&db, &key), ["made-later"]);
}

/// A backup from another computer brings none of that computer's API keys.
#[test]
fn a_backup_from_another_computer_brings_no_api_keys() {
    let there = tempfile::tempdir().unwrap();
    let here = tempfile::tempdir().unwrap();
    let theirs = key_from(29);
    let ours = key_from(30);
    library(there.path(), &theirs, &["a"]);
    let backup = {
        let store = Store::open_with_key(db_path(there.path()), Some(&theirs)).unwrap();
        api_key(&store, "theirs", 3);
        store.backup_now(&there.path().join("backups"), 1).unwrap()
    };
    library(here.path(), &ours, &["f"]);
    let db = db_path(here.path());
    api_key(&Store::open_with_key(&db, Some(&ours)).unwrap(), "ours", 4);
    stage_restore(&db, &backup, Some(&theirs), &ours).unwrap();
    assert!(apply_staged_restore(&db, &ours, &here.path().join("backups"), 2).unwrap());
    assert_eq!(marks(&db, &ours), ["a"]);
    assert_eq!(live_keys(&db, &ours), ["ours"]);
}

/// The sync keys of a library: every one of these `meta` entries it holds.
const SYNC_NAMES: [&str; 12] = [
    "sync.v2.origin",
    "sync.v2.account_id",
    "sync.v2.username",
    "sync.v2.server",
    "sync.v2.library_id",
    "sync.v2.device_secret",
    "sync.v2.hardware",
    "sync.v2.last_seq",
    "sync.v2.epoch",
    "sync.v2.refetch_after",
    "sync.v2.quota.docs",
    "sync.last_seq",
];

/// The device identity, the only sync keys a restore keeps (docs/ACCOUNTS.md
/// › Device identity). Spelled out, not taken from the store's list, so a
/// change to that list fails here.
const DEVICE_IDENTITY: [&str; 3] = [
    "sync.v2.library_id",
    "sync.v2.device_secret",
    "sync.v2.hardware",
];

fn sync_meta(db: &Path, key: &LibraryKey) -> Vec<(String, String)> {
    let store = Store::open_with_key(db, Some(key)).unwrap();
    SYNC_NAMES
        .iter()
        .filter_map(|name| {
            store
                .meta(name)
                .unwrap()
                .map(|value| ((*name).to_owned(), value))
        })
        .collect()
}

/// A binding as a library signed in as `who` would hold it, cursor and all.
fn bind(db: &Path, key: &LibraryKey, who: &str) {
    let store = Store::open_with_key(db, Some(key)).unwrap();
    for name in SYNC_NAMES {
        store.set_meta(name, &format!("{name}@{who}")).unwrap();
    }
}

/// Marks every note of the library as on the server.
fn mark_all_synced(db: &Path, key: &LibraryKey) {
    let store = Store::open_with_key(db, Some(key)).unwrap();
    for state in store.sync_states().unwrap() {
        store.mark_synced(&state.id, &state.rev, None).unwrap();
    }
}

/// Regression: a backup made on another computer carried that library's
/// account, server and device identity, and the restore adopted them, so the
/// library claimed to be signed in as someone else. A restore keeps this
/// library's binding: here, none.
#[test]
fn a_restore_never_adopts_the_backups_sync_binding() {
    let there = tempfile::tempdir().unwrap();
    let here = tempfile::tempdir().unwrap();
    let theirs = key_from(40);
    let ours = key_from(41);
    library(there.path(), &theirs, &["a"]);
    bind(&db_path(there.path()), &theirs, "them");
    let backup = Store::open_with_key(db_path(there.path()), Some(&theirs))
        .unwrap()
        .backup_now(&there.path().join("backups"), 1)
        .unwrap();
    library(here.path(), &ours, &["f"]);
    let db = db_path(here.path());
    stage_restore(&db, &backup, Some(&theirs), &ours).unwrap();
    assert!(apply_staged_restore(&db, &ours, &here.path().join("backups"), 2).unwrap());
    assert_eq!(marks(&db, &ours), ["a"]);
    assert_eq!(sync_meta(&db, &ours), []);
}

/// The library's own device identity survives a restore; its account
/// binding doesn't, and neither does any cursor or epoch. Regression (PR
/// #301 review): the binding was kept, so the next sign-in resumed with no
/// question and merged at once, bringing back every note deleted since the
/// backup (`kanso-sync/tests/restore_sync.rs`).
#[test]
fn a_restore_keeps_only_this_librarys_device_identity() {
    let there = tempfile::tempdir().unwrap();
    let here = tempfile::tempdir().unwrap();
    let theirs = key_from(42);
    let ours = key_from(43);
    library(there.path(), &theirs, &["a"]);
    bind(&db_path(there.path()), &theirs, "them");
    let backup = Store::open_with_key(db_path(there.path()), Some(&theirs))
        .unwrap()
        .backup_now(&there.path().join("backups"), 1)
        .unwrap();
    library(here.path(), &ours, &["f"]);
    let db = db_path(here.path());
    bind(&db, &ours, "me");
    stage_restore(&db, &backup, Some(&theirs), &ours).unwrap();
    assert!(apply_staged_restore(&db, &ours, &here.path().join("backups"), 2).unwrap());
    let found = sync_meta(&db, &ours);
    let names: Vec<&str> = found.iter().map(|(name, _)| name.as_str()).collect();
    assert_eq!(names, DEVICE_IDENTITY);
    assert!(found.iter().all(|(_, value)| value.ends_with("@me")));
}

/// A backup's notes come back unsynced: none claims to be on a server.
#[test]
fn restored_notes_forget_what_they_synced() {
    let there = tempfile::tempdir().unwrap();
    let here = tempfile::tempdir().unwrap();
    let theirs = key_from(44);
    let ours = key_from(45);
    library(there.path(), &theirs, &["a"]);
    mark_all_synced(&db_path(there.path()), &theirs);
    let backup = Store::open_with_key(db_path(there.path()), Some(&theirs))
        .unwrap()
        .backup_now(&there.path().join("backups"), 1)
        .unwrap();
    {
        let store = Store::open_with_key(db_path(there.path()), Some(&theirs)).unwrap();
        assert!(
            store
                .sync_states()
                .unwrap()
                .iter()
                .any(|state| state.synced_rev.is_some()),
            "the fixture really synced something"
        );
    }
    library(here.path(), &ours, &["f"]);
    let db = db_path(here.path());
    stage_restore(&db, &backup, Some(&theirs), &ours).unwrap();
    apply_staged_restore(&db, &ours, &here.path().join("backups"), 2).unwrap();
    let store = Store::open_with_key(&db, Some(&ours)).unwrap();
    let states = store.sync_states().unwrap();
    assert!(!states.is_empty());
    assert!(
        states
            .iter()
            .filter(|state| !state.deleted)
            .all(|state| state.synced_rev.is_none() && state.synced_gen == 0)
    );
}

/// A library that is missing when a restore is applied (deleted by hand,
/// say) takes the staged copy.
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

proptest! {
    #![proptest_config(ProptestConfig {
        cases: 8,
        rng_seed: RngSeed::Fixed(0x006b_616e_736f),
        failure_persistence: None,
        ..ProptestConfig::default()
    })]

    /// Invariants: whichever bindings the backup and the library hold (or
    /// none), a restore leaves exactly the library's own device identity,
    /// no account binding and no cursor, and never a trace of the backup's.
    #[test]
    fn prop_a_restore_keeps_only_the_librarys_own_device_identity(
        theirs_bound in any::<bool>(),
        ours_bound in any::<bool>(),
        theirs in 1u8..=100,
        ours in 101u8..=200,
    ) {
        let there = tempfile::tempdir().unwrap();
        let here = tempfile::tempdir().unwrap();
        let theirs = key_from(theirs);
        let ours = key_from(ours);
        library(there.path(), &theirs, &["a"]);
        if theirs_bound {
            bind(&db_path(there.path()), &theirs, "them");
        }
        let backup = Store::open_with_key(db_path(there.path()), Some(&theirs))
            .unwrap()
            .backup_now(&there.path().join("backups"), 1)
            .unwrap();
        library(here.path(), &ours, &["f"]);
        let db = db_path(here.path());
        if ours_bound {
            bind(&db, &ours, "me");
        }
        stage_restore(&db, &backup, Some(&theirs), &ours).unwrap();
        apply_staged_restore(&db, &ours, &here.path().join("backups"), 2).unwrap();
        let found = sync_meta(&db, &ours);
        let own = found.iter().all(|(name, value)| {
            DEVICE_IDENTITY.contains(&name.as_str()) && value.ends_with("@me")
        });
        prop_assert!(own);
        prop_assert_eq!(found.len(), if ours_bound { DEVICE_IDENTITY.len() } else { 0 });
    }
}
