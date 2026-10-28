//! A note's saved versions through the facade: their order and words, what
//! changed since one, bringing one back, and what a locked note keeps from
//! all of it.

mod common;

use common::*;
use kanso_mobile::{DiffKind, KansoError, Library, LockState};
use kanso_store::{Origin, Put, SharedNote, SharedStatus, Store};
use proptest::prelude::*;
use serde_json::json;

/// What a locked note's versions say, which no result may carry while it is
/// closed.
const SECRET: &str = "SEALED-HISTORY-4410";

/// Writes `body` the way the API does, which keeps the version it replaces
/// in history whenever it is called (the app's own saves keep one every few
/// minutes).
fn api_write(store: &Store, id: &str, body: &str) {
    store
        .put(
            kanso_core::DocType::Note,
            json!({"_id": id, "body": body}),
            Put::Merge,
            Origin::Api,
        )
        .unwrap();
}

fn everything(value: &impl std::fmt::Debug) -> String {
    format!("{value:?}")
}

// no-prop-test: a fixed history with its words, order and authors.
#[test]
fn versions_come_newest_first_with_the_words_for_why_they_were_kept() {
    let phone = lone_phone();
    let library = &phone.library;
    let store = Store::open(phone.path()).unwrap();
    let id = note(library, "Plan", "one\n");
    let rev = library.note(&id).unwrap().rev;
    library.save(&id, &rev, "two\n".into()).unwrap();
    api_write(&store, &id, "three\n");
    std::thread::sleep(std::time::Duration::from_millis(3));
    api_write(&store, &id, "four\n");

    let history = library.history(&id).unwrap();
    assert_eq!(history.current_body, "four\n");
    assert_eq!(history.current_title, "Plan");
    assert_eq!(history.current_authors, None);
    let bodies: Vec<&str> = history.versions.iter().map(|v| v.body.as_str()).collect();
    assert_eq!(bodies, ["three\n", "two\n", "one\n"]);
    let origins: Vec<&str> = history.versions.iter().map(|v| v.origin.as_str()).collect();
    assert_eq!(origins, ["api", "api", "ui"]);
    let labels: Vec<&str> = history
        .versions
        .iter()
        .map(|v| v.origin_label.as_str())
        .collect();
    assert_eq!(
        labels,
        [
            "Replaced through the API",
            "Replaced through the API",
            "Edited in Kanso"
        ]
    );
    assert!(
        history
            .versions
            .windows(2)
            .all(|w| w[0].saved_at >= w[1].saved_at)
    );
    assert!(
        history
            .versions
            .iter()
            .all(|v| v.title == "Plan" && v.authors.is_none())
    );

    // Authors are kept against a shared note's row, so the note is shared
    // first (the row is all that matters here).
    store
        .put_shared_note(&SharedNote {
            share_id: "share-1".into(),
            note_id: id.clone(),
            owner_id: "owner".into(),
            owner_name: "Ada".into(),
            role: "owner".into(),
            status: SharedStatus::Active,
            epoch: 1,
            list: String::new(),
            list_sig: String::new(),
            state: Vec::new(),
            applied_seq: 0,
            since_snapshot: 0,
            bytes_since_snapshot: 0,
            counter: 0,
            needs_snapshot: false,
            updated_at: 0,
        })
        .unwrap();
    store
        .add_shared_authors(&id, &["user2".to_owned(), "user3,user2".to_owned()])
        .unwrap();
    assert_eq!(
        library.history(&id).unwrap().current_authors.as_deref(),
        Some("by user2, user3")
    );
    assert_eq!(library.history("note:nowhere"), Err(KansoError::NotFound));
    assert_eq!(library.history("book:x"), Err(KansoError::NotFound));
    assert_eq!(
        library.history_diff(&id, "9-nope"),
        Err(KansoError::NotFound)
    );
}

// no-prop-test: one change with its words marked.
#[test]
fn a_diff_has_the_lines_numbers_words_and_counts_a_screen_draws() {
    let phone = lone_phone();
    let library = &phone.library;
    let store = Store::open(phone.path()).unwrap();
    let id = note(library, "Plan", "# Plan\n\nBuy milk 🥛\nCall Sam\nWalk\n");
    api_write(
        &store,
        &id,
        "# Plan\n\nBuy oat milk 🥛\nCall Sam\nWalk\nRead\n",
    );
    let version = library.history(&id).unwrap().versions.remove(0);
    let hunks = library.history_diff(&id, &version.rev).unwrap();
    assert_eq!(hunks.len(), 1);
    let hunk = &hunks[0];
    assert_eq!((hunk.added, hunk.removed), (2, 1));
    assert_eq!((hunk.old_start, hunk.old_end), (Some(1), Some(5)));
    assert_eq!(hunk.old_text, "# Plan\n\nBuy milk 🥛\nCall Sam\nWalk\n");
    let removed = hunk
        .lines
        .iter()
        .find(|l| l.kind == DiffKind::Removed)
        .unwrap();
    assert_eq!(
        (removed.text.as_str(), removed.old_line, removed.new_line),
        ("Buy milk 🥛", Some(3), None)
    );
    let added = hunk
        .lines
        .iter()
        .find(|l| l.kind == DiffKind::Added)
        .unwrap();
    assert_eq!(
        (added.text.as_str(), added.old_line, added.new_line),
        ("Buy oat milk 🥛", None, Some(3))
    );
    // The changed words are in UTF-16 units of the line, past the emoji too.
    let units: Vec<u16> = added.text.encode_utf16().collect();
    assert!(added.changed.iter().any(|r| {
        String::from_utf16(&units[r.start as usize..r.end as usize])
            .unwrap()
            .contains("oat")
    }));
    assert!(
        hunk.lines
            .iter()
            .filter(|l| l.kind == DiffKind::Same)
            .all(|l| l.changed.is_empty())
    );
    // The same text has nothing to show. A write that changes only the
    // title still keeps the version it replaces, whose text is the note's.
    let current = library.note(&id).unwrap();
    store
        .put(
            kanso_core::DocType::Note,
            json!({"_id": id, "title": "Plan, again"}),
            Put::Merge,
            Origin::Api,
        )
        .unwrap();
    let newest = library.history(&id).unwrap().versions.remove(0);
    assert_eq!(newest.body, current.body.unwrap());
    assert!(library.history_diff(&id, &newest.rev).unwrap().is_empty());
}

// no-prop-test: the refusals of a stale revision and an unknown version.
#[test]
fn restoring_refuses_a_stale_revision_and_a_version_that_isnt_kept() {
    let phone = lone_phone();
    let library = &phone.library;
    let id = note(library, "Plan", "one");
    let rev = library.note(&id).unwrap().rev;
    library.save(&id, &rev, "two".into()).unwrap();
    let version = library.history(&id).unwrap().versions.remove(0);
    let now = library.note(&id).unwrap().rev;
    assert_eq!(
        library.restore_version(&id, &version.rev, &rev),
        Err(KansoError::Conflict {
            current_rev: now.clone()
        })
    );
    assert_eq!(
        library.restore_version(&id, "9-nope", &now),
        Err(KansoError::NotFound)
    );
    assert_eq!(library.note(&id).unwrap().body.as_deref(), Some("two"));
}

fn bodies() -> impl Strategy<Value = Vec<String>> {
    prop::collection::vec(
        prop::collection::vec(
            prop_oneof![
                3 => "[a-c ]{0,6}",
                1 => Just("\r\n".to_owned()),
                1 => Just("\n".to_owned()),
                1 => Just("é😀".to_owned()),
                1 => "\\PC{0,5}",
            ],
            0..5,
        )
        .prop_map(|parts| parts.concat()),
        2..6,
    )
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(24))]

    /// Restoring any version leaves the note with that version's title and
    /// text, and keeps the text it had in history, even right after another
    /// save. It adds a revision; it never rewinds.
    #[test]
    fn prop_a_restore_never_loses_the_text_it_replaces(
        bodies in bodies(),
        api_between in prop::collection::vec(any::<bool>(), 6),
        pick in any::<prop::sample::Index>(),
        again in any::<bool>(),
    ) {
        let phone = lone_phone();
        let library = &phone.library;
        let store = Store::open(phone.path()).unwrap();
        let id = note(library, "Title é", &bodies[0]);
        for (i, body) in bodies.iter().enumerate().skip(1) {
            if api_between[i] {
                api_write(&store, &id, body);
            } else {
                let rev = library.note(&id).unwrap().rev;
                library.save(&id, &rev, body.clone()).unwrap();
            }
        }
        let before = library.history(&id).unwrap();
        prop_assume!(!before.versions.is_empty());
        let version = pick.get(&before.versions).clone();
        let base = library.note(&id).unwrap().rev;
        let saved = library.restore_version(&id, &version.rev, &base).unwrap();

        let after = library.note(&id).unwrap();
        prop_assert_eq!(after.body.as_deref(), Some(version.body.as_str()));
        prop_assert_eq!(&after.title, &version.title);
        prop_assert_eq!(&after.rev, &saved.rev);
        let history = library.history(&id).unwrap();
        prop_assert!(
            history.versions.iter().any(|v| v.body == before.current_body && v.title == before.current_title),
            "the text before the restore is kept"
        );
        if again {
            // And right after it: restoring the version before this one.
            let rev = library.note(&id).unwrap().rev;
            let back = history.versions.iter().find(|v| v.body == before.current_body).unwrap().clone();
            library.restore_version(&id, &back.rev, &rev).unwrap();
            prop_assert_eq!(library.note(&id).unwrap().body, Some(before.current_body.clone()));
            prop_assert!(library
                .history(&id)
                .unwrap()
                .versions
                .iter()
                .any(|v| v.body == version.body));
        }
    }
}

/// A library with a sealed note whose history has two sealed versions, the
/// session open; the sealed text carries [`SECRET`].
fn sealed_with_history() -> (Phone, String) {
    let phone = lone_phone();
    let library = &phone.library;
    let id = note(library, "Vault plan", &format!("first {SECRET}\n"));
    library
        .locks()
        .lock_note(&id, LOCK_PASSWORD.into())
        .unwrap();
    for text in ["second", "third"] {
        let rev = library.note(&id).unwrap().rev;
        library
            .save(&id, &rev, format!("{text} {SECRET}\n"))
            .unwrap();
        // Saves within minutes share a version: make each one's snapshot.
        let store = Store::open(phone.path()).unwrap();
        let doc = store.get(&id).unwrap().unwrap();
        store
            .record_revision(&id, &doc.rev, &doc.fields, Origin::Ui)
            .unwrap();
    }
    (phone, id)
}

// no-prop-test: the fixed cases of a sealed note, open and then closed.
#[test]
fn a_sealed_notes_versions_are_opened_while_it_is_open_and_refused_when_it_isnt() {
    let (phone, id) = sealed_with_history();
    let library = &phone.library;
    let store = Store::open(phone.path()).unwrap();
    assert_eq!(library.note(&id).unwrap().lock, LockState::Open);

    let history = library.history(&id).unwrap();
    assert_eq!(history.current_body, format!("third {SECRET}\n"));
    assert!(!history.versions.is_empty());
    assert!(history.versions.iter().all(|v| v.body.contains(SECRET)));
    // A version no key opens is left out, never returned sealed.
    let doc = store.get(&id).unwrap().unwrap();
    let mut foreign = doc.fields.clone();
    foreign["kansoLock"] =
        json!({"v": 1, "salt": "b3RoZXI", "m": 64, "t": 1, "p": 1, "data": "CIPHERTEXT-FOREIGN"});
    store
        .record_revision(&id, "9-foreign", &foreign, Origin::Ui)
        .unwrap();
    let with_foreign = library.history(&id).unwrap();
    assert_eq!(with_foreign.versions.len(), history.versions.len());
    assert!(!everything(&with_foreign).contains("CIPHERTEXT-FOREIGN"));

    // Diff and restore, open.
    let version = history.versions.last().unwrap().clone();
    let hunks = library.history_diff(&id, &version.rev).unwrap();
    assert!(!hunks.is_empty());
    let base = library.note(&id).unwrap().rev;
    let saved = library.restore_version(&id, &version.rev, &base).unwrap();
    assert_eq!(library.note(&id).unwrap().body, Some(version.body.clone()));
    assert!(
        library
            .history(&id)
            .unwrap()
            .versions
            .iter()
            .any(|v| v.body == format!("third {SECRET}\n"))
    );
    // Stored sealed: the row holds none of the text.
    let stored = store.get(&id).unwrap().unwrap();
    assert_eq!(stored.rev, saved.rev);
    assert!(stored.fields.contains_key("kansoLock"));
    assert!(
        !serde_json::to_string(&stored.fields)
            .unwrap()
            .contains(SECRET)
    );

    // Closed: everything refuses and nothing changes.
    library.locks().lock().unwrap();
    let rows = |store: &Store| {
        let doc = store.get(&id).unwrap().unwrap();
        (doc.fields, doc.rev, store.revisions(&id).unwrap().len())
    };
    let before = rows(&store);
    let rev = before.1.clone();
    for result in [
        library.history(&id).map(|h| everything(&h)),
        library
            .history_diff(&id, &version.rev)
            .map(|h| everything(&h)),
        library
            .restore_version(&id, &version.rev, &rev)
            .map(|s| everything(&s)),
        library
            .restore_version(&id, &version.rev, "1-stale")
            .map(|s| everything(&s)),
    ] {
        assert_eq!(result, Err(KansoError::Locked));
    }
    assert_eq!(rows(&store), before);
}

// no-prop-test: a note in a locked notebook, closed and open.
#[test]
fn a_note_in_a_locked_notebook_keeps_its_history_from_a_closed_session() {
    let mut vaulted = String::new();
    let phone = phone_with(
        &std::sync::Arc::new(kanso_sync::MemoryServer::new()),
        |store| {
            let vault = book(store, "Vault");
            vaulted = raw_note(
                store,
                json!({"title": "Vaulted", "body": format!("vault {SECRET}"), "bookId": vault}),
            );
            let key = kanso_sync::lock::LockKey::create(
                &kanso_sync::lock::LockPassword::new(LOCK_PASSWORD).unwrap(),
                kanso_sync::KdfParams::TEST,
            )
            .unwrap();
            assert_eq!(
                kanso_engine::lock::seal_notebook(store, &vault, &key),
                Ok(1)
            );
        },
    );
    let library: &Library = &phone.library;
    let store = Store::open(phone.path()).unwrap();
    let before = store.get(&vaulted).unwrap().unwrap();
    assert_eq!(library.history(&vaulted), Err(KansoError::Locked));
    assert_eq!(
        library.history_diff(&vaulted, "1-x"),
        Err(KansoError::Locked)
    );
    assert_eq!(
        library.restore_version(&vaulted, "1-x", &before.rev),
        Err(KansoError::Locked)
    );
    assert_eq!(store.get(&vaulted).unwrap().unwrap(), before);
    assert_eq!(library.locks().unlock(LOCK_PASSWORD.into()), Ok(true));
    let history = library.history(&vaulted).unwrap();
    assert!(history.current_body.contains(SECRET));
    assert!(history.versions.iter().all(|v| v.title == "Vaulted"));
}
