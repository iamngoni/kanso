//! Saving the shared items in the inbox folder: what a drop becomes, the
//! order, what is removed and what is kept, and what a `drop.json` can't
//! make the library read.

mod common;

use std::fs;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::time::{Duration, SystemTime};

use common::*;
use kanso_mobile::capture::save_inbox;
use kanso_mobile::{DropsSaved, Library};
use kanso_store::Store;
use kanso_sync::MemoryServer;
use proptest::prelude::*;
use serde_json::{Value, json};

const PDF: &[u8] = b"%PDF-1.7\n%%EOF\n";

/// The inbox folder beside a phone's library.
fn inbox_of(phone: &Phone) -> PathBuf {
    phone.dir.path().join("inbox")
}

/// A finished drop: `inbox/<id>/` holding `drop.json` and the files.
fn write_drop(inbox: &Path, id: &str, manifest: &Value, files: &[(&str, &[u8])]) -> PathBuf {
    let folder = inbox.join(id);
    fs::create_dir_all(&folder).unwrap();
    for (name, bytes) in files {
        fs::write(folder.join(name), bytes).unwrap();
    }
    fs::write(folder.join("drop.json"), manifest.to_string()).unwrap();
    folder
}

fn id(n: u8) -> String {
    format!("{n:02x}").repeat(16)
}

fn text_drop(text: &str, created: i64) -> Value {
    json!({"version": 1, "createdAt": created, "text": text})
}

fn note_ids(library: &Library) -> Vec<String> {
    let mut ids: Vec<String> = library
        .list(Default::default())
        .unwrap()
        .rows
        .into_iter()
        .map(|row| row.id)
        .collect();
    ids.sort();
    ids
}

// no-prop-test: the contract's sequence for one drop, once.
#[test]
fn a_drop_becomes_a_note_in_the_inbox_and_its_folder_goes() {
    let phone = lone_phone();
    let inbox = inbox_of(&phone);
    let manifest = json!({
        "version": 1, "createdAt": 5, "subject": "Trip", "text": "Packing",
        "url": "https://t.io/x",
        "files": [
            {"path": "0.pdf", "name": "Tickets.pdf", "type": "application/pdf"},
            {"path": "1.txt", "name": "notes.txt", "type": "text/plain"},
        ],
    });
    let folder = write_drop(
        &inbox,
        &id(1),
        &manifest,
        &[("0.pdf", PDF), ("1.txt", b"hello")],
    );

    let saved = phone.library.save_drops(&inbox).unwrap();
    assert_eq!(saved.notes, vec![format!("note:cap-{}", id(1))]);
    assert_eq!(saved.kept, 0);
    assert_eq!(saved.problems.len(), 1, "{:?}", saved.problems);
    assert!(saved.problems[0].starts_with("notes.txt: Only images and PDFs"));
    assert!(!folder.exists());

    let note = phone.library.note(&saved.notes[0]).unwrap();
    assert_eq!(note.title, "Trip");
    assert_eq!(note.notebook_id, "book:inbox");
    let body = note.body.unwrap();
    assert!(
        body.starts_with("Packing\n\n[Trip](https://t.io/x)\n\n[Tickets.pdf](inkdrop://file:cap-"),
        "{body}"
    );

    // Nothing more to do the second time, and no error for a folder that isn't there.
    assert_eq!(
        phone.library.save_drops(&inbox).unwrap(),
        DropsSaved::default()
    );
    let nowhere = phone.dir.path().join("none");
    assert_eq!(
        phone.library.save_drops(&nowhere).unwrap(),
        DropsSaved::default()
    );
}

// no-prop-test: three fixed drops in an order that isn't their names'.
#[test]
fn drops_are_saved_oldest_first_by_time_then_by_name() {
    let phone = lone_phone();
    let inbox = inbox_of(&phone);
    write_drop(&inbox, &id(3), &text_drop("third", 300), &[]);
    write_drop(&inbox, &id(1), &text_drop("second b", 200), &[]);
    write_drop(&inbox, &id(2), &text_drop("second a", 200), &[]);
    write_drop(&inbox, &id(9), &text_drop("first", 100), &[]);
    let saved = phone.library.save_drops(&inbox).unwrap();
    let titles: Vec<String> = saved
        .notes
        .iter()
        .map(|note| phone.library.note(note).unwrap().title)
        .collect();
    assert_eq!(titles, ["first", "second b", "second a", "third"]);
    let expected: Vec<String> = [9, 1, 2, 3]
        .iter()
        .map(|n| format!("note:cap-{}", id(*n)))
        .collect();
    assert_eq!(saved.notes, expected);
}

// no-prop-test: a table of folders the reader leaves alone or removes.
#[test]
fn unfinished_folders_are_never_read_and_old_ones_are_removed() {
    let phone = lone_phone();
    let inbox = inbox_of(&phone);
    // Being written now: a complete-looking drop.json inside, still not read.
    let fresh = write_drop(
        &inbox,
        &format!("{}.partial", id(1)),
        &text_drop("early", 1),
        &[],
    );
    // Abandoned two days ago.
    let stale = write_drop(
        &inbox,
        &format!("{}.partial", id(2)),
        &text_drop("late", 1),
        &[],
    );
    let two_days = SystemTime::now() - Duration::from_secs(2 * 24 * 60 * 60);
    fs::File::open(&stale)
        .unwrap()
        .set_modified(two_days)
        .unwrap();
    // Not a drop's name: left alone, old or not.
    let other = inbox.join("somebody-elses");
    fs::create_dir_all(&other).unwrap();
    fs::File::open(&other)
        .unwrap()
        .set_modified(two_days)
        .unwrap();
    let other_partial = inbox.join("notes.partial");
    fs::create_dir_all(&other_partial).unwrap();
    fs::File::open(&other_partial)
        .unwrap()
        .set_modified(two_days)
        .unwrap();
    // A plain file in the inbox.
    fs::write(inbox.join("stray.txt"), "x").unwrap();

    let saved = phone.library.save_drops(&inbox).unwrap();
    assert_eq!(saved, DropsSaved::default());
    assert!(fresh.exists() && !stale.exists());
    assert!(other.exists() && other_partial.exists() && inbox.join("stray.txt").exists());
    assert!(note_ids(&phone.library).is_empty());
}

// no-prop-test: the refusals, one drop each, with the words they give.
#[test]
fn drops_that_can_never_be_saved_are_removed_with_a_reason() {
    let phone = lone_phone();
    let inbox = inbox_of(&phone);
    let broken = write_drop(&inbox, &id(1), &json!({}), &[]);
    fs::write(broken.join("drop.json"), "{not json").unwrap();
    let missing = write_drop(&inbox, &id(2), &json!({}), &[]);
    fs::remove_file(missing.join("drop.json")).unwrap();
    let wrong_version = write_drop(&inbox, &id(3), &json!({"version": 0, "text": "x"}), &[]);
    let empty = write_drop(&inbox, &id(4), &json!({"version": 1, "text": "  "}), &[]);
    let not_files = write_drop(
        &inbox,
        &id(5),
        &json!({"version": 1, "files": [{"path": "a.zip", "name": "a.zip"}]}),
        &[("a.zip", b"PK")],
    );

    let saved = phone.library.save_drops(&inbox).unwrap();
    assert!(saved.notes.is_empty() && saved.kept == 0);
    for folder in [&broken, &missing, &wrong_version, &empty, &not_files] {
        assert!(!folder.exists(), "{folder:?}");
    }
    assert_eq!(
        saved
            .problems
            .iter()
            .filter(|p| p.as_str() == "A shared item couldn't be read and was discarded.")
            .count(),
        3
    );
    assert!(
        saved
            .problems
            .iter()
            .any(|p| p.contains("a.zip: Only images and PDFs")
                && p.ends_with("had nothing Kanso could save and was discarded.")),
        "{:?}",
        saved.problems
    );
    assert!(
        saved
            .problems
            .iter()
            .any(|p| p == "A shared item had nothing Kanso could save and was discarded."),
        "{:?}",
        saved.problems
    );
}

// no-prop-test: a locked Inbox, once; the property below covers the tree.
#[test]
fn a_locked_inbox_takes_nothing_and_the_drop_is_discarded() {
    let server = Arc::new(MemoryServer::new());
    let phone = phone_with(&server, |store| {
        let inbox = store
            .role_book(kanso_core::roles::Role::Inbox, kanso_store::Origin::Ui)
            .unwrap();
        lock_book(store, &inbox);
    });
    let inbox = inbox_of(&phone);
    let folder = write_drop(&inbox, &id(1), &text_drop("Secret", 1), &[]);
    let saved = phone.library.save_drops(&inbox).unwrap();
    assert!(saved.notes.is_empty() && saved.kept == 0);
    assert_eq!(
        saved.problems,
        ["The Inbox is locked, so a shared item wasn't saved and was discarded."]
    );
    assert!(!folder.exists());
    assert!(note_ids(&phone.library).is_empty());
}

/// Locks notebook `id` the way a synced lock arrives.
fn lock_book(store: &Store, id: &str) {
    let lock = json!({"v": 1, "salt": "c2FsdA", "m": 64, "t": 1, "p": 1, "data": "notebook"});
    store
        .put(
            kanso_core::DocType::Book,
            json!({"_id": id, "kansoLock": lock}),
            kanso_store::Put::Merge,
            kanso_store::Origin::Sync,
        )
        .unwrap();
}

// no-prop-test: needs a file the process can't read, once.
#[cfg(unix)]
#[test]
fn a_failure_that_may_pass_keeps_the_drop_and_a_later_try_saves_it() {
    use std::os::unix::fs::PermissionsExt;
    let phone = lone_phone();
    let inbox = inbox_of(&phone);
    let folder = write_drop(
        &inbox,
        &id(1),
        &json!({"version": 1, "createdAt": 1, "text": "Kept", "files": [{"path": "0.pdf", "name": "a.pdf"}]}),
        &[("0.pdf", PDF)],
    );
    write_drop(&inbox, &id(2), &text_drop("Fine", 2), &[]);
    let file = folder.join("0.pdf");
    fs::set_permissions(&file, fs::Permissions::from_mode(0o000)).unwrap();
    // Run as root, a file with no permissions is still readable: nothing to test.
    if fs::read(&file).is_ok() {
        fs::set_permissions(&file, fs::Permissions::from_mode(0o644)).unwrap();
        return;
    }

    let saved = phone.library.save_drops(&inbox).unwrap();
    assert_eq!(saved.kept, 1);
    assert_eq!(saved.notes, vec![format!("note:cap-{}", id(2))]);
    assert!(folder.exists());
    assert_eq!(saved.problems.len(), 1);
    assert!(
        saved.problems[0].ends_with("It will be tried again."),
        "{:?}",
        saved.problems
    );

    fs::set_permissions(&file, fs::Permissions::from_mode(0o644)).unwrap();
    let saved = phone.library.save_drops(&inbox).unwrap();
    assert_eq!(saved.kept, 0);
    assert_eq!(saved.notes, vec![format!("note:cap-{}", id(1))]);
    assert!(!folder.exists());
}

// no-prop-test: a manifest from a newer Kanso, once.
#[test]
fn a_drop_from_a_newer_kanso_is_kept_whole() {
    let phone = lone_phone();
    let inbox = inbox_of(&phone);
    let folder = write_drop(
        &inbox,
        &id(1),
        &json!({"version": 2, "text": "from the future"}),
        &[],
    );
    let saved = phone.library.save_drops(&inbox).unwrap();
    assert_eq!((saved.notes.len(), saved.kept), (0, 1));
    assert!(
        saved.problems[0].contains("newer Kanso"),
        "{:?}",
        saved.problems
    );
    assert!(folder.exists());
}

// no-prop-test: the process's own open library is written to, once.
#[test]
fn the_library_the_app_has_open_is_the_one_written_to() {
    let server = Arc::new(MemoryServer::new());
    let phone = lone_phone();
    let seen = Arc::new(Mutex::new(Vec::<String>::new()));
    let sink = seen.clone();
    phone.library.changes(move |ids| {
        sink.lock().unwrap().extend(ids);
        true
    });
    let inbox = inbox_of(&phone);
    write_drop(&inbox, &id(1), &text_drop("Hello", 1), &[]);
    // A second copy of the secret store would show as a different library.
    let saved = save_inbox(&phone.path(), &inbox, phone.secrets.clone(), server).unwrap();
    assert_eq!(saved.notes, vec![format!("note:cap-{}", id(1))]);
    let wanted = saved.notes[0].clone();
    for _ in 0..100 {
        if seen.lock().unwrap().contains(&wanted) {
            return;
        }
        std::thread::sleep(Duration::from_millis(20));
    }
    panic!(
        "the open library never saw the note: {:?}",
        seen.lock().unwrap()
    );
}

// no-prop-test: a closed library is opened by the call, once.
#[test]
fn a_closed_library_is_opened_for_the_call() {
    let phone = lone_phone();
    let (path, secrets) = (phone.path(), phone.secrets.clone());
    let keep = phone.dir;
    drop(phone.library);
    let inbox = keep.path().join("inbox");
    write_drop(&inbox, &id(1), &text_drop("Alone", 1), &[]);
    let saved = save_inbox(
        &path,
        &inbox,
        secrets.clone(),
        Arc::new(MemoryServer::new()),
    )
    .unwrap();
    assert_eq!(saved.notes.len(), 1);
    let json: Value = serde_json::from_str(&saved.to_json()).unwrap();
    assert_eq!(
        json,
        json!({"notes": [format!("note:cap-{}", id(1))], "problems": [], "kept": 0})
    );
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(24))]

    /// Invariant: a file path in a `drop.json` that isn't a plain name inside
    /// the drop's folder is never read, however it is written, even when the
    /// file it points at exists.
    #[test]
    fn prop_paths_that_leave_the_folder_are_refused(
        prefix in prop::sample::select(vec!["../", "/", "..\\", "./", "a/", "..", "a/../", "../../", "\\", "C:"]),
        name in prop::sample::select(vec!["secret.pdf", "0.pdf", "drop.json", ".."]),
    ) {
        let phone = lone_phone();
        let inbox = inbox_of(&phone);
        fs::create_dir_all(&inbox).unwrap();
        // The secret is beside the drop's folder, in the inbox, and also at
        // an absolute path.
        fs::write(inbox.join("secret.pdf"), PDF).unwrap();
        let outside = phone.dir.path().join("secret.pdf");
        fs::write(&outside, PDF).unwrap();
        let path = if prefix == "/" {
            outside.to_str().unwrap().to_owned()
        } else {
            format!("{prefix}{name}")
        };
        let manifest = json!({"version": 1, "createdAt": 1, "text": "Hi",
            "files": [{"path": path, "name": "x.pdf", "type": "application/pdf"}]});
        write_drop(&inbox, &id(1), &manifest, &[("0.pdf", PDF)]);
        let saved = phone.library.save_drops(&inbox).unwrap();
        prop_assert_eq!(saved.notes.len(), 1);
        let body = phone.library.note(&saved.notes[0]).unwrap().body.unwrap();
        prop_assert!(!body.contains("inkdrop://file:"), "{path:?}: {body}");
        prop_assert!(saved.problems.iter().any(|p| p.starts_with("x.pdf: ")), "{path:?}: {:?}", saved.problems);
    }

    /// Invariant: drops come out in the order of their times then names,
    /// whatever order they were made in, and every folder is gone after.
    #[test]
    fn prop_drops_are_saved_oldest_first(times in prop::collection::vec(0i64..4, 1..6)) {
        let phone = lone_phone();
        let inbox = inbox_of(&phone);
        let mut expected: Vec<(i64, String)> = Vec::new();
        for (n, time) in times.iter().enumerate() {
            let drop_id = id(n as u8 + 1);
            write_drop(&inbox, &drop_id, &text_drop(&format!("note {n}"), *time), &[]);
            expected.push((*time, drop_id));
        }
        expected.sort();
        let saved = phone.library.save_drops(&inbox).unwrap();
        let expected: Vec<String> = expected.into_iter().map(|(_, id)| format!("note:cap-{id}")).collect();
        prop_assert_eq!(saved.notes, expected);
        prop_assert_eq!(fs::read_dir(&inbox).unwrap().count(), 0);
    }

    /// Invariant: over notebook trees with the Inbox anywhere in them and
    /// locks anywhere, a drop is saved exactly when the Inbox is not locked
    /// or inside a lock, and nothing is written into a locked notebook.
    #[test]
    fn prop_nothing_reaches_a_locked_inbox(
        locks in prop::collection::vec(any::<bool>(), 3),
        inbox_parent in prop::option::of(0usize..3),
    ) {
        let server = Arc::new(MemoryServer::new());
        let closed = std::cell::Cell::new(false);
        let phone = phone_with(&server, |store| {
            let ids: Vec<String> = (0..3).map(|n| book(store, &format!("B{n}"))).collect();
            let inbox = store.role_book(kanso_core::roles::Role::Inbox, kanso_store::Origin::Ui).unwrap();
            if let Some(parent) = inbox_parent {
                store.put(kanso_core::DocType::Book, json!({"_id": inbox, "parentBookId": ids[parent]}),
                    kanso_store::Put::Merge, kanso_store::Origin::Sync).unwrap();
            }
            for (n, locked) in locks.iter().enumerate() {
                if *locked {
                    lock_book(store, &ids[n]);
                }
            }
            closed.set(store.locked_books().unwrap().contains(&inbox));
        });
        let inbox = inbox_of(&phone);
        let folder = write_drop(&inbox, &id(1), &text_drop("Secret", 1), &[]);
        let saved = phone.library.save_drops(&inbox).unwrap();
        prop_assert_eq!(saved.notes.len(), usize::from(!closed.get()));
        prop_assert_eq!(saved.problems.len(), usize::from(closed.get()));
        prop_assert!(!folder.exists());
        prop_assert_eq!(note_ids(&phone.library).len(), usize::from(!closed.get()));
    }
}
