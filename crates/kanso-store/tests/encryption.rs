//! Encrypted libraries (issue #243 step 9): a keyed library never opens without
//! its key, a backup round-trips with it, search works inside it, and no note
//! text reaches the disk in the clear.

use std::path::{Path, PathBuf};

use kanso_core::{DocType, Query};
use kanso_store::{
    KEY_BYTES, LibraryFile, LibraryKey, Origin, Put, SearchParams, Sort, Store, StoreError,
};
use proptest::prelude::*;
use serde_json::json;

fn key_from(seed: u8, salt: u8) -> LibraryKey {
    let mut bytes = [seed; KEY_BYTES];
    bytes[0] = salt;
    bytes[KEY_BYTES - 1] = seed.wrapping_add(salt);
    LibraryKey::from_bytes(bytes)
}

fn db_path(dir: &tempfile::TempDir) -> PathBuf {
    dir.path().join("kanso.db")
}

fn add_note(store: &Store, title: &str, body: &str) -> String {
    let book = match store.meta("test_book").unwrap() {
        Some(book) => book,
        None => {
            let book = store
                .put(
                    DocType::Book,
                    json!({"name": "Notes"}),
                    Put::Upsert,
                    Origin::Api,
                )
                .unwrap()
                .id;
            store.set_meta("test_book", &book).unwrap();
            book
        }
    };
    store
        .put(
            DocType::Note,
            json!({"bookId": book, "title": title, "body": body}),
            Put::Upsert,
            Origin::Api,
        )
        .unwrap()
        .id
}

fn found(store: &Store, term: &str) -> Vec<String> {
    let params = SearchParams {
        query: Query::parse(term),
        sort: Sort::Title,
        descending: false,
        ..SearchParams::default()
    };
    store
        .search(&params)
        .unwrap()
        .into_iter()
        .map(|doc| doc.id)
        .collect()
}

/// The library's three files, whichever exist, with their bytes.
fn library_bytes(db: &Path) -> Vec<(String, Vec<u8>)> {
    ["", "-wal", "-shm"]
        .iter()
        .filter_map(|suffix| {
            let mut name = db.as_os_str().to_owned();
            name.push(suffix);
            let path = PathBuf::from(name);
            std::fs::read(&path)
                .ok()
                .map(|bytes| (path.display().to_string(), bytes))
        })
        .collect()
}

fn contains(haystack: &[u8], needle: &[u8]) -> bool {
    haystack.windows(needle.len()).any(|w| w == needle)
}

#[test]
fn a_new_keyed_library_is_not_a_plain_sqlite_file() {
    let dir = tempfile::tempdir().unwrap();
    let path = db_path(&dir);
    assert_eq!(LibraryFile::of(&path), LibraryFile::New);
    let key = key_from(1, 2);
    let store = Store::open_with_key(&path, Some(&key)).unwrap();
    assert!(store.is_encrypted());
    add_note(&store, "Hello", "world");
    drop(store);
    assert_eq!(LibraryFile::of(&path), LibraryFile::Encrypted);
    let head = std::fs::read(&path).unwrap();
    assert!(!head.starts_with(b"SQLite format 3"));
}

#[test]
fn a_plain_library_opens_exactly_as_before() {
    let dir = tempfile::tempdir().unwrap();
    let path = db_path(&dir);
    let store = Store::open(&path).unwrap();
    assert!(!store.is_encrypted());
    let id = add_note(&store, "Old note", "from before encryption");
    drop(store);
    assert_eq!(LibraryFile::of(&path), LibraryFile::Plain);
    let again = Store::open(&path).unwrap();
    assert!(again.get(&id).unwrap().is_some());
    // A key does not unlock a plain file, and does not touch it.
    let err = Store::open_with_key(&path, Some(&key_from(3, 4))).err();
    assert!(matches!(err, Some(StoreError::WrongKey)), "{err:?}");
    assert_eq!(LibraryFile::of(&path), LibraryFile::Plain);
}

#[test]
fn the_key_in_hex_reads_back() {
    let key = key_from(9, 200);
    let hex = key.to_hex();
    assert_eq!(hex.len(), 64);
    assert_eq!(LibraryKey::from_hex(&hex), Some(key.clone()));
    assert_eq!(LibraryKey::from_hex(&hex.to_uppercase()), Some(key.clone()));
    assert_eq!(format!("{key:?}"), "LibraryKey(..)");
    for bad in [
        "",
        "zz",
        &hex[..63],
        &format!("{}0", hex.as_str()),
        &"g".repeat(64),
    ] {
        assert_eq!(LibraryKey::from_hex(bad), None, "{bad}");
    }
    assert_ne!(LibraryKey::generate(), LibraryKey::generate());
}

#[test]
fn search_works_inside_an_encrypted_library() {
    let dir = tempfile::tempdir().unwrap();
    let key = key_from(5, 6);
    let store = Store::open_with_key(db_path(&dir), Some(&key)).unwrap();
    let hit = add_note(&store, "Garden plans", "Plant the tomatoes near the fence");
    add_note(&store, "Taxes", "Receipts for the year");
    // The words, a fragment of a word (trigram) and a title all find it.
    assert_eq!(found(&store, "tomatoes"), vec![hit.clone()]);
    assert_eq!(found(&store, "omato"), vec![hit.clone()]);
    assert_eq!(found(&store, "garden"), vec![hit.clone()]);
    assert!(found(&store, "nothing here").is_empty());
    // And after a reopen, on a fresh pooled reader.
    drop(store);
    let again = Store::open_with_key(db_path(&dir), Some(&key)).unwrap();
    assert_eq!(found(&again, "tomatoes"), vec![hit]);
}

#[test]
fn a_backup_round_trips_with_its_key_and_only_with_it() {
    let dir = tempfile::tempdir().unwrap();
    let key = key_from(7, 8);
    let store = Store::open_with_key(db_path(&dir), Some(&key)).unwrap();
    let id = add_note(&store, "Kept", "backup-canary-body");
    let backup = dir.path().join("copy.db");
    store.backup_to(&backup).unwrap();

    assert_eq!(LibraryFile::of(&backup), LibraryFile::Encrypted);
    for (name, bytes) in library_bytes(&backup) {
        assert!(!contains(&bytes, b"backup-canary-body"), "{name}");
    }
    let restored = Store::open_with_key(&backup, Some(&key)).unwrap();
    assert!(restored.get(&id).unwrap().is_some());
    assert_eq!(found(&restored, "backup-canary-body"), vec![id]);
    assert!(matches!(Store::open(&backup), Err(StoreError::WrongKey)));
    assert!(matches!(
        Store::open_with_key(&backup, Some(&key_from(8, 7))),
        Err(StoreError::WrongKey)
    ));
}

#[test]
fn daily_and_manual_backups_are_keyed() {
    let dir = tempfile::tempdir().unwrap();
    let key = key_from(11, 12);
    let store = Store::open_with_key(db_path(&dir), Some(&key)).unwrap();
    add_note(&store, "x", "y");
    let backups = dir.path().join("backups");
    let daily = store
        .backup_if_due(&kanso_store::BackupPolicy::daily(&backups), 1_000)
        .unwrap()
        .unwrap();
    let manual = store.backup_now(&backups, 2_000).unwrap();
    for path in [daily, manual] {
        assert!(matches!(Store::open(&path), Err(StoreError::WrongKey)));
        Store::open_with_key(&path, Some(&key)).unwrap();
    }
}

#[test]
fn a_read_only_view_needs_the_key_too() {
    let dir = tempfile::tempdir().unwrap();
    let path = db_path(&dir);
    let key = key_from(13, 14);
    let store = Store::open_with_key(&path, Some(&key)).unwrap();
    store.set_meta("library_id", "l_1").unwrap();
    assert!(matches!(
        Store::open_read_only(&path),
        Err(StoreError::WrongKey)
    ));
    let view = Store::open_read_only_with_key(&path, Some(&key)).unwrap();
    assert_eq!(view.meta("library_id").unwrap().as_deref(), Some("l_1"));
}

#[cfg(not(any(target_os = "ios", target_os = "android")))]
#[test]
fn the_library_files_hold_no_note_text_and_the_scan_can_see_it() {
    const CANARY: &str = "zqxcanary-7f3a91-moonlit-harbour";
    let write_notes = |store: &Store| {
        let id = add_note(store, CANARY, &format!("body of {CANARY} and more words"));
        // An edit and a deletion leave older text behind in a plain library.
        let mut doc = store.get_api(&id, None, false).unwrap();
        doc["body"] = json!(format!("second {CANARY} draft"));
        store
            .put(DocType::Note, doc, Put::Upsert, Origin::Api)
            .unwrap();
        add_note(store, "other", &format!("gone {CANARY}"));
    };

    // Positive control: the same notes in a plain library show the canary.
    let plain_dir = tempfile::tempdir().unwrap();
    let plain = Store::open(db_path(&plain_dir)).unwrap();
    write_notes(&plain);
    let seen = |dir: &tempfile::TempDir| {
        library_bytes(&db_path(dir))
            .iter()
            .any(|(_, bytes)| contains(bytes, CANARY.as_bytes()))
    };
    assert!(
        seen(&plain_dir),
        "the scan can't see text in a plain library"
    );
    drop(plain);
    assert!(seen(&plain_dir), "the scan can't see text after a close");

    // The encrypted library, while open (the -wal and -shm exist) and closed.
    let dir = tempfile::tempdir().unwrap();
    let store = Store::open_with_key(db_path(&dir), Some(&key_from(21, 22))).unwrap();
    write_notes(&store);
    let files = library_bytes(&db_path(&dir));
    assert!(
        files.iter().any(|(name, _)| name.ends_with("-wal")),
        "no -wal to scan: {:?}",
        files.iter().map(|(n, _)| n).collect::<Vec<_>>()
    );
    assert!(!seen(&dir), "note text is in the clear while open");
    drop(store);
    assert!(!seen(&dir), "note text is in the clear after a close");
}

proptest! {
    #![proptest_config(ProptestConfig { cases: 24, ..ProptestConfig::default() })]

    /// Invariants: a keyed library opens with its own key; it never opens
    /// with no key, nor with any other key; a failed open writes nothing.
    #[test]
    fn prop_a_keyed_library_opens_only_with_its_key(
        right in proptest::array::uniform32(any::<u8>()),
        wrong in proptest::array::uniform32(any::<u8>()),
    ) {
        let right = LibraryKey::from_bytes(right);
        let wrong = LibraryKey::from_bytes(wrong);
        let dir = tempfile::tempdir().unwrap();
        let path = db_path(&dir);
        let store = Store::open_with_key(&path, Some(&right)).unwrap();
        let id = add_note(&store, "a note", "its body");
        drop(store);

        let again = Store::open_with_key(&path, Some(&right)).unwrap();
        prop_assert!(again.get(&id).unwrap().is_some());
        drop(again);

        prop_assert!(matches!(Store::open(&path), Err(StoreError::WrongKey)));
        if wrong != right {
            prop_assert!(matches!(
                Store::open_with_key(&path, Some(&wrong)),
                Err(StoreError::WrongKey)
            ));
        }
        // The reopen with the right key may checkpoint; the wrong opens did
        // not change the file any further.
        let after_right = std::fs::read(&path).unwrap();
        let _ = Store::open(&path);
        let _ = Store::open_with_key(&path, Some(&wrong));
        prop_assert_eq!(std::fs::read(&path).unwrap(), after_right);
    }

    /// Invariants: a backup of an encrypted library opens with the key, holds
    /// every note, and opens with no other key.
    #[test]
    fn prop_a_backup_round_trips_with_its_key(
        raw in proptest::array::uniform32(any::<u8>()),
        other in proptest::array::uniform32(any::<u8>()),
        bodies in proptest::collection::vec("[a-z]{4,12}", 1..6),
    ) {
        let key = LibraryKey::from_bytes(raw);
        let other = LibraryKey::from_bytes(other);
        let dir = tempfile::tempdir().unwrap();
        let store = Store::open_with_key(db_path(&dir), Some(&key)).unwrap();
        let ids: Vec<String> = bodies
            .iter()
            .enumerate()
            .map(|(i, body)| add_note(&store, &format!("note {i}"), body))
            .collect();
        let backup = dir.path().join("b.db");
        store.backup_to(&backup).unwrap();

        let restored = Store::open_with_key(&backup, Some(&key)).unwrap();
        for id in &ids {
            prop_assert!(restored.get(id).unwrap().is_some());
        }
        prop_assert!(matches!(Store::open(&backup), Err(StoreError::WrongKey)));
        if other != key {
            prop_assert!(matches!(
                Store::open_with_key(&backup, Some(&other)),
                Err(StoreError::WrongKey)
            ));
        }
    }

    /// Invariant: the hex form of any key reads back to the same key.
    #[test]
    fn prop_key_hex_round_trips(raw in proptest::array::uniform32(any::<u8>())) {
        let key = LibraryKey::from_bytes(raw);
        prop_assert_eq!(LibraryKey::from_hex(&key.to_hex()), Some(key));
    }
}
