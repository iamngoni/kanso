//! Encrypting a library that is already there (issue #243 step 10): the
//! contents survive unchanged, no note text is left in the clear, and a
//! crash at any step is recovered from at the next launch. Seeds are pinned.

use std::collections::BTreeMap;
use std::panic::{AssertUnwindSafe, catch_unwind};
use std::path::{Path, PathBuf};

use kanso_core::{DocType, Query};
use kanso_store::{
    EncryptOptions, EncryptStep, Encrypted, KEY_BYTES, LibraryFile, LibraryKey, Origin, Put,
    SearchParams, Sort, Store, StoreError, encrypt_backups, encrypt_library, encryption_unfinished,
};
use proptest::prelude::*;
use proptest::test_runner::RngSeed;
use rusqlite::Connection;
use rusqlite::types::ValueRef;
use serde_json::json;

const CANARY: &str = "zqxcanary-9d41e7-lantern-orchard";

fn key_from(seed: u8) -> LibraryKey {
    let mut bytes = [seed; KEY_BYTES];
    bytes[0] = seed.wrapping_mul(7).wrapping_add(1);
    LibraryKey::from_bytes(bytes)
}

fn db_path(dir: &Path) -> PathBuf {
    dir.join("kanso.db")
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

/// A plain library with `notes`, one of them carrying the canary and edited
/// once, so older text is left behind in the file.
fn plain_library(dir: &Path, notes: &[(String, String)]) {
    let store = Store::open(db_path(dir)).unwrap();
    for (title, body) in notes {
        add_note(&store, title, body);
    }
    let id = add_note(&store, CANARY, &format!("first draft of {CANARY}"));
    let mut doc = store.get_api(&id, None, false).unwrap();
    doc["body"] = json!(format!("second draft of {CANARY}"));
    store
        .put(DocType::Note, doc, Put::Upsert, Origin::Api)
        .unwrap();
}

/// Every row of every table, as text, for comparing before and after.
fn contents(db: &Path, key: Option<&LibraryKey>) -> BTreeMap<String, Vec<String>> {
    let conn = Connection::open(db).unwrap();
    if let Some(key) = key {
        conn.execute_batch(&format!("PRAGMA key = \"x'{}'\";", key.to_hex().as_str()))
            .unwrap();
    }
    let tables: Vec<String> = conn
        .prepare(
            "SELECT name FROM sqlite_master WHERE type = 'table' AND name NOT LIKE 'sqlite\\_%' ESCAPE '\\' AND sql NOT LIKE 'CREATE VIRTUAL%' ORDER BY name",
        )
        .unwrap()
        .query_map([], |row| row.get(0))
        .unwrap()
        .map(Result::unwrap)
        .collect();
    let mut out = BTreeMap::new();
    for table in tables {
        let mut stmt = conn
            .prepare(&format!("SELECT * FROM \"{table}\" NOT INDEXED"))
            .unwrap();
        let columns = stmt.column_count();
        let mut rows = stmt.query([]).unwrap();
        let mut lines = Vec::new();
        while let Some(row) = rows.next().unwrap() {
            let line: Vec<String> = (0..columns)
                .map(|i| match row.get_ref(i).unwrap() {
                    ValueRef::Null => "NULL".into(),
                    ValueRef::Integer(n) => n.to_string(),
                    ValueRef::Real(f) => f.to_string(),
                    ValueRef::Text(t) => String::from_utf8_lossy(t).into_owned(),
                    ValueRef::Blob(b) => format!("blob{}", b.len()),
                })
                .collect();
            lines.push(line.join("|"));
        }
        out.insert(table, lines);
    }
    out
}

/// Every file in `dir` and below, with its bytes.
fn all_bytes(dir: &Path) -> Vec<(PathBuf, Vec<u8>)> {
    let mut found = Vec::new();
    let mut stack = vec![dir.to_path_buf()];
    while let Some(next) = stack.pop() {
        for entry in std::fs::read_dir(next).unwrap().flatten() {
            let path = entry.path();
            if path.is_dir() {
                stack.push(path);
            } else if let Ok(bytes) = std::fs::read(&path) {
                found.push((path, bytes));
            }
        }
    }
    found
}

fn contains(haystack: &[u8], needle: &[u8]) -> bool {
    haystack.windows(needle.len()).any(|w| w == needle)
}

fn names(dir: &Path) -> Vec<String> {
    let mut names: Vec<String> = std::fs::read_dir(dir)
        .unwrap()
        .flatten()
        .map(|e| e.file_name().to_string_lossy().into_owned())
        .collect();
    names.sort();
    names
}

fn found(store: &Store, term: &str) -> usize {
    store
        .search(&SearchParams {
            query: Query::parse(term),
            sort: Sort::Title,
            descending: false,
            ..SearchParams::default()
        })
        .unwrap()
        .len()
}

/// What a finished conversion leaves: the library, and nothing of the attempt.
fn assert_finished(dir: &Path, key: &LibraryKey) {
    assert_eq!(LibraryFile::of(&db_path(dir)), LibraryFile::Encrypted);
    assert!(!encryption_unfinished(&db_path(dir)));
    for name in names(dir) {
        assert!(
            !name.ends_with(".enc")
                && !name.ends_with(".plain")
                && !name.ends_with(".encrypting")
                && !name.ends_with(".tmp"),
            "left behind: {name}"
        );
    }
    // The library opens with the key and no other way.
    let store = Store::open_with_key(db_path(dir), Some(key)).unwrap();
    assert!(found(&store, "orchard") >= 1);
    drop(store);
    assert!(matches!(
        Store::open(db_path(dir)),
        Err(StoreError::WrongKey)
    ));
}

fn assert_no_canary(dir: &Path) {
    for (path, bytes) in all_bytes(dir) {
        assert!(
            !contains(&bytes, CANARY.as_bytes()),
            "note text in the clear in {}",
            path.display()
        );
    }
}

#[test]
fn a_plain_library_is_encrypted_with_all_its_contents() {
    let dir = tempfile::tempdir().unwrap();
    let notes: Vec<(String, String)> = (0..40)
        .map(|i| {
            (
                format!("note {i}"),
                format!("body number {i} about harbours"),
            )
        })
        .collect();
    plain_library(dir.path(), &notes);
    // The positive control: the scan sees the canary in the plain library.
    assert!(
        all_bytes(dir.path())
            .iter()
            .any(|(_, bytes)| contains(bytes, CANARY.as_bytes())),
        "the scan can't see text in a plain library"
    );
    let before = contents(&db_path(dir.path()), None);
    assert!(before["docs"].len() > 40);

    let key = key_from(1);
    let done = encrypt_library(&db_path(dir.path()), &key, &EncryptOptions::default()).unwrap();
    assert_eq!(done, Encrypted::Converted);

    assert_eq!(contents(&db_path(dir.path()), Some(&key)), before);
    assert_finished(dir.path(), &key);
    assert_no_canary(dir.path());
    // Opened, written to and closed, still none.
    let store = Store::open_with_key(db_path(dir.path()), Some(&key)).unwrap();
    add_note(&store, "later", &format!("written after {CANARY}"));
    assert!(found(&store, "harbours") > 0);
    drop(store);
    assert_no_canary(dir.path());
    // And a second time there is nothing to do.
    assert_eq!(
        encrypt_library(&db_path(dir.path()), &key, &EncryptOptions::default()).unwrap(),
        Encrypted::AlreadyEncrypted
    );
}

#[test]
fn progress_only_goes_forward_and_ends_at_the_top() {
    let dir = tempfile::tempdir().unwrap();
    plain_library(dir.path(), &[("a".into(), "b".into())]);
    let seen = std::sync::Mutex::new(Vec::new());
    let progress = |p: kanso_store::EncryptProgress| seen.lock().unwrap().push(p.percent());
    let opts = EncryptOptions {
        progress: Some(&progress),
        ..EncryptOptions::default()
    };
    encrypt_library(&db_path(dir.path()), &key_from(2), &opts).unwrap();
    let seen = seen.into_inner().unwrap();
    assert_eq!(seen.last(), Some(&100));
    assert!(seen.windows(2).all(|w| w[0] <= w[1]), "{seen:?}");
}

/// Runs the conversion and panics after `at`, as a kill would stop it.
fn crash_at(dir: &Path, key: &LibraryKey, at: EncryptStep) {
    let hook = move |step: EncryptStep| {
        if step == at {
            panic!("killed after {step:?}");
        }
    };
    let opts = EncryptOptions {
        after_step: Some(&hook),
        ..EncryptOptions::default()
    };
    let result = catch_unwind(AssertUnwindSafe(|| {
        encrypt_library(&db_path(dir), key, &opts)
    }));
    assert!(result.is_err(), "the conversion did not stop at {at:?}");
}

/// The state a crash leaves: one library at the library's name, which opens
/// as what it is, and no program may open it while unfinished.
fn assert_one_live_library(dir: &Path) {
    let db = db_path(dir);
    assert_ne!(LibraryFile::of(&db), LibraryFile::New);
    if encryption_unfinished(&db) {
        assert!(matches!(Store::open(&db), Err(StoreError::Converting)));
        assert!(matches!(
            Store::open_read_only(&db),
            Err(StoreError::Converting)
        ));
    }
}

#[test]
fn a_kill_at_every_step_is_recovered_from_at_the_next_launch() {
    for at in EncryptStep::ALL {
        let dir = tempfile::tempdir().unwrap();
        let notes = vec![("one".to_owned(), "orchard apples".to_owned())];
        plain_library(dir.path(), &notes);
        let before = contents(&db_path(dir.path()), None);
        let key = key_from(3);
        crash_at(dir.path(), &key, at);
        assert_one_live_library(dir.path());

        // The next launch.
        let again = encrypt_library(&db_path(dir.path()), &key, &EncryptOptions::default());
        again.unwrap_or_else(|e| panic!("after {at:?}: {e}"));
        assert_eq!(
            contents(&db_path(dir.path()), Some(&key)),
            before,
            "after {at:?}"
        );
        assert_finished(dir.path(), &key);
        assert_no_canary(dir.path());
    }
}

#[test]
fn a_kill_before_the_swap_with_the_key_gone_starts_over_with_a_new_one() {
    for at in [
        EncryptStep::Exported,
        EncryptStep::ReadyMarked,
        EncryptStep::WalRemoved,
    ] {
        let dir = tempfile::tempdir().unwrap();
        plain_library(dir.path(), &[("one".into(), "two".into())]);
        let before = contents(&db_path(dir.path()), None);
        crash_at(dir.path(), &key_from(4), at);
        // The next launch has a different key: the old copy can't be used.
        let other = key_from(5);
        encrypt_library(&db_path(dir.path()), &other, &EncryptOptions::default()).unwrap();
        assert_eq!(contents(&db_path(dir.path()), Some(&other)), before);
        assert_finished(dir.path(), &other);
    }
}

#[test]
fn a_kill_after_the_swap_with_the_wrong_key_puts_the_plain_library_back() {
    for at in [EncryptStep::Swapped] {
        let dir = tempfile::tempdir().unwrap();
        plain_library(dir.path(), &[("one".into(), "two".into())]);
        let before = contents(&db_path(dir.path()), None);
        crash_at(dir.path(), &key_from(6), at);
        let err = encrypt_library(
            &db_path(dir.path()),
            &key_from(7),
            &EncryptOptions::default(),
        )
        .unwrap_err();
        assert!(matches!(err, StoreError::WrongKey), "{err:?}");
        // Back as it was: plain, whole, openable, nothing of the attempt.
        assert_eq!(LibraryFile::of(&db_path(dir.path())), LibraryFile::Plain);
        assert_eq!(contents(&db_path(dir.path()), None), before);
        assert!(!encryption_unfinished(&db_path(dir.path())));
        Store::open(db_path(dir.path())).unwrap();
        for name in names(dir.path()) {
            assert!(
                !name.ends_with(".plain") && !name.ends_with(".enc"),
                "{name}"
            );
        }
    }
}

#[test]
fn a_disk_too_small_fails_plainly_and_leaves_the_library_alone() {
    let dir = tempfile::tempdir().unwrap();
    let notes: Vec<(String, String)> = (0..200)
        .map(|i| {
            (
                format!("note {i}"),
                "a fairly long body of words ".repeat(40),
            )
        })
        .collect();
    plain_library(dir.path(), &notes);
    let before = contents(&db_path(dir.path()), None);
    let opts = EncryptOptions {
        max_copy_bytes: Some(64 * 1024),
        ..EncryptOptions::default()
    };
    let err = encrypt_library(&db_path(dir.path()), &key_from(8), &opts).unwrap_err();
    match &err {
        StoreError::DiskFull { needed_mb } => assert!(*needed_mb >= 1),
        other => panic!("{other:?}"),
    }
    assert!(err.to_string().contains("free space"), "{err}");
    assert_eq!(LibraryFile::of(&db_path(dir.path())), LibraryFile::Plain);
    assert!(!encryption_unfinished(&db_path(dir.path())));
    assert_eq!(contents(&db_path(dir.path()), None), before);
    assert_eq!(
        names(dir.path())
            .into_iter()
            .filter(|n| n.starts_with("kanso.db."))
            .count(),
        0,
        "{:?}",
        names(dir.path())
    );
    Store::open(db_path(dir.path())).unwrap();
    // Room again: it works.
    encrypt_library(
        &db_path(dir.path()),
        &key_from(8),
        &EncryptOptions::default(),
    )
    .unwrap();
    assert_finished(dir.path(), &key_from(8));
}

#[test]
fn plain_backups_are_encrypted_and_their_plaintext_deleted() {
    let dir = tempfile::tempdir().unwrap();
    plain_library(dir.path(), &[("one".into(), "two".into())]);
    let backups = dir.path().join("backups");
    {
        let store = Store::open(db_path(dir.path())).unwrap();
        store
            .backup_if_due(&kanso_store::BackupPolicy::daily(&backups), 1_000)
            .unwrap();
        store.backup_now(&backups, 2_000).unwrap();
    }
    let paths: Vec<PathBuf> = names(&backups).iter().map(|n| backups.join(n)).collect();
    assert_eq!(paths.len(), 2);
    let before: Vec<_> = paths.iter().map(|p| contents(p, None)).collect();
    let key = key_from(9);
    let report = encrypt_backups(&backups, &key, &EncryptOptions::default()).unwrap();
    assert_eq!(report.converted, 2);
    assert!(report.failed.is_empty(), "{report:?}");
    for (path, before) in paths.iter().zip(before) {
        assert_eq!(LibraryFile::of(path), LibraryFile::Encrypted);
        assert_eq!(contents(path, Some(&key)), before);
        Store::open_with_key(path, Some(&key)).unwrap();
    }
    assert_eq!(names(&backups).len(), 2, "{:?}", names(&backups));
    assert_no_canary(&backups);
    // Again: nothing left to do.
    let again = encrypt_backups(&backups, &key, &EncryptOptions::default()).unwrap();
    assert_eq!(again.converted, 0);
}

#[test]
fn a_backup_swap_a_crash_left_is_cleaned_without_losing_the_backup() {
    let dir = tempfile::tempdir().unwrap();
    plain_library(dir.path(), &[("one".into(), "two".into())]);
    let backups = dir.path().join("backups");
    Store::open(db_path(dir.path()))
        .unwrap()
        .backup_now(&backups, 5_000)
        .unwrap();
    let backup = backups.join("kanso-manual-5000.db");
    let before = contents(&backup, None);
    // Killed after linking the plaintext and before the swap: a partial
    // copy and a second name for the backup itself.
    std::fs::write(backups.join("kanso-manual-5000.db.enc"), b"partial").unwrap();
    std::fs::hard_link(&backup, backups.join("kanso-manual-5000.db.old")).unwrap();
    let key = key_from(10);
    let report = encrypt_backups(&backups, &key, &EncryptOptions::default()).unwrap();
    assert_eq!(report.converted, 1);
    assert_eq!(contents(&backup, Some(&key)), before);
    assert_eq!(names(&backups), vec!["kanso-manual-5000.db".to_owned()]);
    assert_no_canary(&backups);
}

#[test]
fn an_unfinished_conversion_keeps_every_other_program_out() {
    let dir = tempfile::tempdir().unwrap();
    plain_library(dir.path(), &[("one".into(), "two".into())]);
    crash_at(dir.path(), &key_from(11), EncryptStep::Exported);
    let db = db_path(dir.path());
    assert!(encryption_unfinished(&db));
    for open in [
        Store::open(&db).err(),
        Store::open_with_key(&db, Some(&key_from(11))).err(),
        Store::open_read_only(&db).err(),
    ] {
        assert!(matches!(open, Some(StoreError::Converting)), "{open:?}");
    }
}

// A real kill: this test binary runs itself and aborts partway.
const CHILD_DIR: &str = "KANSO_ENCRYPT_CHILD_DIR";
const CHILD_STEP: &str = "KANSO_ENCRYPT_CHILD_STEP";

#[test]
fn child_converts_and_aborts() {
    let (Ok(dir), Ok(step)) = (std::env::var(CHILD_DIR), std::env::var(CHILD_STEP)) else {
        return;
    };
    let at = EncryptStep::ALL[step.parse::<usize>().unwrap()];
    let hook = move |step: EncryptStep| {
        if step == at {
            std::process::abort();
        }
    };
    let opts = EncryptOptions {
        after_step: Some(&hook),
        ..EncryptOptions::default()
    };
    let _ = encrypt_library(&db_path(Path::new(&dir)), &key_from(12), &opts);
}

#[cfg(unix)]
#[test]
fn kill_9_at_each_step_then_the_next_launch_recovers() {
    for (index, at) in EncryptStep::ALL.into_iter().enumerate() {
        let dir = tempfile::tempdir().unwrap();
        plain_library(dir.path(), &[("one".into(), "orchard two".into())]);
        let before = contents(&db_path(dir.path()), None);
        let status = std::process::Command::new(std::env::current_exe().unwrap())
            .args(["--exact", "child_converts_and_aborts", "--nocapture"])
            .env(CHILD_DIR, dir.path())
            .env(CHILD_STEP, index.to_string())
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .status()
            .unwrap();
        assert!(!status.success(), "the child was not killed at {at:?}");
        assert_one_live_library(dir.path());
        let key = key_from(12);
        encrypt_library(&db_path(dir.path()), &key, &EncryptOptions::default())
            .unwrap_or_else(|e| panic!("after kill at {at:?}: {e}"));
        assert_eq!(
            contents(&db_path(dir.path()), Some(&key)),
            before,
            "after kill at {at:?}"
        );
        assert_finished(dir.path(), &key);
        assert_no_canary(dir.path());
    }
}

proptest! {
    #![proptest_config(ProptestConfig {
        cases: 24,
        rng_seed: RngSeed::Fixed(0x006b_616e_736f),
        failure_persistence: None,
        ..ProptestConfig::default()
    })]

    /// Invariants: whatever the library holds and wherever the conversion is
    /// killed, the next launch ends with an encrypted library whose contents
    /// are those of the plain one, no leftovers, and no note text in any
    /// file.
    #[test]
    fn prop_any_kill_then_a_relaunch_keeps_every_row(
        notes in proptest::collection::vec(("[a-z ]{1,20}", "[a-z \n]{0,200}"), 0..12),
        step in 0usize..EncryptStep::ALL.len(),
        seed in any::<u8>(),
    ) {
        let dir = tempfile::tempdir().unwrap();
        plain_library(dir.path(), &notes);
        let before = contents(&db_path(dir.path()), None);
        let key = key_from(seed);
        crash_at(dir.path(), &key, EncryptStep::ALL[step]);
        assert_one_live_library(dir.path());
        encrypt_library(&db_path(dir.path()), &key, &EncryptOptions::default()).unwrap();
        prop_assert_eq!(contents(&db_path(dir.path()), Some(&key)), before);
        assert_finished(dir.path(), &key);
        assert_no_canary(dir.path());
    }
}

/// A copy that doesn't match is never swapped in: the library stays plain
/// and whole.
#[test]
fn a_copy_that_does_not_match_is_refused() {
    let dir = tempfile::tempdir().unwrap();
    plain_library(dir.path(), &[("one".into(), "two".into())]);
    let before = contents(&db_path(dir.path()), None);
    let copy = {
        let mut name = db_path(dir.path()).into_os_string();
        name.push(".enc");
        PathBuf::from(name)
    };
    let hook = |step: EncryptStep| {
        if step == EncryptStep::Exported {
            // Damage the middle of the copy.
            let mut bytes = std::fs::read(&copy).unwrap();
            let mid = bytes.len() / 2;
            for byte in &mut bytes[mid..mid + 64] {
                *byte ^= 0xff;
            }
            std::fs::write(&copy, bytes).unwrap();
        }
    };
    let opts = EncryptOptions {
        after_step: Some(&hook),
        ..EncryptOptions::default()
    };
    assert!(encrypt_library(&db_path(dir.path()), &key_from(13), &opts).is_err());
    assert_eq!(LibraryFile::of(&db_path(dir.path())), LibraryFile::Plain);
    assert!(!encryption_unfinished(&db_path(dir.path())));
    assert_eq!(contents(&db_path(dir.path()), None), before);
    assert_eq!(
        names(dir.path())
            .into_iter()
            .filter(|n| n.starts_with("kanso.db."))
            .count(),
        0
    );
}

fn beside(db: &Path, suffix: &str) -> PathBuf {
    let mut name = db.as_os_str().to_owned();
    name.push(suffix);
    PathBuf::from(name)
}

fn write_marker(db: &Path, phase: &str) {
    std::fs::write(
        beside(db, ".encrypting"),
        format!("{{\"v\":1,\"phase\":\"{phase}\"}}"),
    )
    .unwrap();
}

/// Runs a conversion that changes the encrypted copy with `tamper` (given a
/// keyed connection to it) right after the export, and checks the copy is
/// refused and the library left plain and whole.
fn assert_tampered_copy_is_refused(
    dir: &Path,
    key: &LibraryKey,
    tamper: &(dyn Fn(&Connection) + Sync),
) {
    let db = db_path(dir);
    let before = contents(&db, None);
    let copy = beside(&db, ".enc");
    let hook = |step: EncryptStep| {
        if step == EncryptStep::Exported {
            let conn = Connection::open(&copy).unwrap();
            conn.execute_batch(&format!("PRAGMA key = \"x'{}'\";", key.to_hex().as_str()))
                .unwrap();
            tamper(&conn);
        }
    };
    let opts = EncryptOptions {
        after_step: Some(&hook),
        ..EncryptOptions::default()
    };
    let result = encrypt_library(&db, key, &opts);
    assert!(result.is_err(), "a changed copy was swapped in: {result:?}");
    assert_eq!(LibraryFile::of(&db), LibraryFile::Plain);
    assert!(!encryption_unfinished(&db));
    assert_eq!(contents(&db, None), before);
}

/// The compare covers rowids: the columns alone miss a row moved to another
/// rowid.
#[test]
fn a_copy_with_a_changed_rowid_is_refused() {
    let dir = tempfile::tempdir().unwrap();
    plain_library(dir.path(), &[("one".into(), "two".into())]);
    assert_tampered_copy_is_refused(dir.path(), &key_from(20), &|conn| {
        conn.execute_batch(
            "UPDATE docs SET rowid = rowid + 100000 WHERE rowid = (SELECT max(rowid) FROM docs)",
        )
        .unwrap();
    });
}

/// The compare covers `sqlite_sequence`, the counters of AUTOINCREMENT
/// tables: a counter that went back would hand out a used ID again.
#[test]
fn a_copy_with_a_changed_autoincrement_counter_is_refused() {
    let dir = tempfile::tempdir().unwrap();
    plain_library(dir.path(), &[("one".into(), "two".into())]);
    Connection::open(db_path(dir.path()))
        .unwrap()
        .execute_batch(
            "CREATE TABLE counted (id INTEGER PRIMARY KEY AUTOINCREMENT, v);
             INSERT INTO counted (v) VALUES (1), (2), (3);
             DELETE FROM counted;",
        )
        .unwrap();
    assert_tampered_copy_is_refused(dir.path(), &key_from(21), &|conn| {
        conn.execute_batch("UPDATE sqlite_sequence SET seq = 0 WHERE name = 'counted'")
            .unwrap();
    });
}

/// The compare covers each object's SQL, not only its name: an index of the
/// same name over other columns is a different schema.
#[test]
fn a_copy_with_a_changed_index_definition_is_refused() {
    let dir = tempfile::tempdir().unwrap();
    plain_library(dir.path(), &[("one".into(), "two".into())]);
    assert_tampered_copy_is_refused(dir.path(), &key_from(22), &|conn| {
        conn.execute_batch("DROP INDEX docs_book; CREATE INDEX docs_book ON docs (book_id)")
            .unwrap();
    });
}

/// The plain library's second name is a hard link: the same file. This
/// follows both names through the conversion and proves the overwrite goes
/// to the plain file only, after the encrypted one has taken the library's
/// name, and never to the library.
#[cfg(unix)]
#[test]
fn the_overwrite_reaches_the_plain_file_and_never_the_library() {
    use std::os::unix::fs::MetadataExt as _;
    let dir = tempfile::tempdir().unwrap();
    plain_library(dir.path(), &[("one".into(), "orchard two".into())]);
    let db = db_path(dir.path());
    let safety = beside(&db, ".plain");
    let copy = beside(&db, ".enc");
    let before = contents(&db, None);
    let plain_ino = std::fs::metadata(&db).unwrap().ino();
    // A handle on the plain file itself, kept through the conversion.
    let plain = std::fs::File::open(&db).unwrap();
    let ino = |path: &Path| std::fs::metadata(path).ok().map(|meta| meta.ino());
    let seen = std::sync::Mutex::new(Vec::new());
    let hook = |step: EncryptStep| {
        seen.lock()
            .unwrap()
            .push((step, ino(&db), ino(&safety), ino(&copy)));
    };
    let opts = EncryptOptions {
        after_step: Some(&hook),
        ..EncryptOptions::default()
    };
    let key = key_from(23);
    encrypt_library(&db, &key, &opts).unwrap();
    let seen = seen.into_inner().unwrap();
    let at = |step: EncryptStep| *seen.iter().find(|s| s.0 == step).unwrap();
    let copy_ino = at(EncryptStep::Verified).3.unwrap();
    assert_ne!(copy_ino, plain_ino);
    // Linked: two names, one plain file; the copy is apart.
    assert_eq!(
        at(EncryptStep::SafetyLinked),
        (
            EncryptStep::SafetyLinked,
            Some(plain_ino),
            Some(plain_ino),
            Some(copy_ino)
        )
    );
    // Swapped and confirmed: the library is the copy; the plain file is
    // only its second name.
    for step in [EncryptStep::Swapped, EncryptStep::Confirmed] {
        assert_eq!(at(step), (step, Some(copy_ino), Some(plain_ino), None));
    }
    // Shredded: the second name is gone and the library is still the copy.
    assert_eq!(
        at(EncryptStep::PlainShredded),
        (EncryptStep::PlainShredded, Some(copy_ino), None, None)
    );
    // The overwrite went to the plain file: through the handle it is empty.
    assert_eq!(plain.metadata().unwrap().len(), 0);
    drop(plain);
    assert_eq!(contents(&db, Some(&key)), before);
    assert_finished(dir.path(), &key);
    assert_no_canary(dir.path());
}

/// A second name that is the encrypted library itself (not the plain file)
/// must never be overwritten: that would destroy the only library.
#[cfg(unix)]
#[test]
fn a_second_name_for_the_encrypted_library_is_never_overwritten() {
    let dir = tempfile::tempdir().unwrap();
    plain_library(dir.path(), &[("one".into(), "orchard two".into())]);
    let db = db_path(dir.path());
    let key = key_from(24);
    encrypt_library(&db, &key, &EncryptOptions::default()).unwrap();
    let before = contents(&db, Some(&key));
    std::fs::hard_link(&db, beside(&db, ".plain")).unwrap();
    write_marker(&db, "cleanup");
    encrypt_library(&db, &key, &EncryptOptions::default()).unwrap();
    assert_eq!(contents(&db, Some(&key)), before);
    assert_finished(dir.path(), &key);
}

/// A plain copy from an earlier attempt that is not the library is never
/// put back over it: the conversion stops, says which file, and the library
/// stays as it is.
#[test]
fn a_stale_plain_copy_is_never_put_back_over_the_library() {
    let dir = tempfile::tempdir().unwrap();
    plain_library(dir.path(), &[("one".into(), "two".into())]);
    let db = db_path(dir.path());
    // An older library under the plain copy's name.
    std::fs::copy(&db, beside(&db, ".plain")).unwrap();
    let older = contents(&beside(&db, ".plain"), None);
    {
        let store = Store::open(&db).unwrap();
        add_note(&store, "newer", "written after the copy");
    }
    let before = contents(&db, None);
    assert_ne!(before, older);
    let err = encrypt_library(&db, &key_from(25), &EncryptOptions::default()).unwrap_err();
    assert_eq!(LibraryFile::of(&db), LibraryFile::Plain);
    assert!(
        contents(&db, None) == before,
        "the library was replaced: {err}"
    );
    assert!(err.to_string().contains("kanso.db.plain"), "{err}");
    // The other file is left for the person to look at.
    assert_eq!(contents(&beside(&db, ".plain"), None), older);
    assert!(!encryption_unfinished(&db));
}

/// On a disk without hard links the plain library's second name is a copy.
/// A crash partway through that copy leaves a partial file under the name;
/// the next launch must make it again, never compare with it or put it
/// back over the library.
#[test]
fn a_partial_plain_copy_from_a_crash_is_never_put_back() {
    let dir = tempfile::tempdir().unwrap();
    let notes: Vec<(String, String)> = (0..30)
        .map(|i| (format!("note {i}"), "orchard words ".repeat(30)))
        .collect();
    plain_library(dir.path(), &notes);
    let db = db_path(dir.path());
    let before = contents(&db, None);
    let key = key_from(31);
    crash_at(dir.path(), &key, EncryptStep::WalRemoved);
    // The copy stopped halfway.
    let bytes = std::fs::read(&db).unwrap();
    std::fs::write(beside(&db, ".plain"), &bytes[..bytes.len() / 2]).unwrap();
    encrypt_library(&db, &key, &EncryptOptions::default()).unwrap();
    assert_eq!(contents(&db, Some(&key)), before);
    assert_finished(dir.path(), &key);
    assert_no_canary(dir.path());
}

/// A marker that says the copy was still being made, beside a library that
/// is already encrypted, is stale: the library opens as it is.
#[test]
fn a_stale_exporting_marker_beside_an_encrypted_library_is_cleared() {
    let dir = tempfile::tempdir().unwrap();
    plain_library(dir.path(), &[("one".into(), "orchard two".into())]);
    let db = db_path(dir.path());
    let key = key_from(26);
    encrypt_library(&db, &key, &EncryptOptions::default()).unwrap();
    let before = contents(&db, Some(&key));
    write_marker(&db, "exporting");
    std::fs::write(beside(&db, ".enc"), b"partial").unwrap();
    assert!(encryption_unfinished(&db));
    assert_eq!(
        encrypt_library(&db, &key, &EncryptOptions::default()).unwrap(),
        Encrypted::AlreadyEncrypted
    );
    assert_eq!(contents(&db, Some(&key)), before);
    assert_finished(dir.path(), &key);
}

/// The library's name gone with its plain copy kept: the copy comes back.
#[test]
fn a_missing_library_with_its_plain_copy_kept_is_put_back() {
    let dir = tempfile::tempdir().unwrap();
    plain_library(dir.path(), &[("one".into(), "orchard two".into())]);
    let db = db_path(dir.path());
    let before = contents(&db, None);
    std::fs::rename(&db, beside(&db, ".plain")).unwrap();
    write_marker(&db, "ready");
    let key = key_from(27);
    encrypt_library(&db, &key, &EncryptOptions::default()).unwrap();
    assert_eq!(contents(&db, Some(&key)), before);
    assert_finished(dir.path(), &key);
}

/// A state Kanso can't sort out says what to do, naming the marker, and
/// deletes nothing.
#[test]
fn an_unexpected_state_says_how_to_get_out() {
    let dir = tempfile::tempdir().unwrap();
    let db = db_path(dir.path());
    std::fs::write(beside(&db, ".enc"), b"partial").unwrap();
    write_marker(&db, "exporting");
    let err = encrypt_library(&db, &key_from(28), &EncryptOptions::default()).unwrap_err();
    let text = err.to_string();
    assert!(text.contains("kanso.db.encrypting"), "{text}");
    assert!(text.contains("remove"), "{text}");
    assert!(beside(&db, ".enc").exists());
}

/// A crash after a backup's swap, before its plaintext was overwritten:
/// the next run overwrites it and keeps the encrypted backup.
#[test]
fn a_backup_swapped_but_not_shredded_is_finished() {
    let dir = tempfile::tempdir().unwrap();
    plain_library(dir.path(), &[("one".into(), "orchard two".into())]);
    let backups = dir.path().join("backups");
    Store::open(db_path(dir.path()))
        .unwrap()
        .backup_now(&backups, 6_000)
        .unwrap();
    let backup = backups.join("kanso-manual-6000.db");
    let before = contents(&backup, None);
    let key = key_from(29);
    // The encrypted backup, made apart, then swapped in over a backup whose
    // plaintext keeps a second name, as a crash before the overwrite leaves.
    let apart = dir.path().join("apart");
    std::fs::create_dir(&apart).unwrap();
    std::fs::copy(&backup, apart.join("kanso-manual-6000.db")).unwrap();
    encrypt_backups(&apart, &key, &EncryptOptions::default()).unwrap();
    std::fs::hard_link(&backup, backups.join("kanso-manual-6000.db.old")).unwrap();
    std::fs::rename(apart.join("kanso-manual-6000.db"), &backup).unwrap();
    std::fs::remove_dir_all(&apart).unwrap();

    let report = encrypt_backups(&backups, &key, &EncryptOptions::default()).unwrap();
    assert_eq!(report.converted, 0);
    assert_eq!(names(&backups), vec!["kanso-manual-6000.db".to_owned()]);
    assert_eq!(contents(&backup, Some(&key)), before);
    assert_no_canary(&backups);
}

/// A backup whose only name left is its plaintext's second one is put back
/// and encrypted, never deleted.
#[test]
fn a_backup_known_only_by_its_second_name_is_kept() {
    let dir = tempfile::tempdir().unwrap();
    plain_library(dir.path(), &[("one".into(), "orchard two".into())]);
    let backups = dir.path().join("backups");
    Store::open(db_path(dir.path()))
        .unwrap()
        .backup_now(&backups, 7_000)
        .unwrap();
    let backup = backups.join("kanso-manual-7000.db");
    let before = contents(&backup, None);
    std::fs::rename(&backup, backups.join("kanso-manual-7000.db.old")).unwrap();
    let key = key_from(30);
    let report = encrypt_backups(&backups, &key, &EncryptOptions::default()).unwrap();
    assert_eq!(report.converted, 1);
    assert_eq!(names(&backups), vec!["kanso-manual-7000.db".to_owned()]);
    assert_eq!(contents(&backup, Some(&key)), before);
    assert_no_canary(&backups);
}
