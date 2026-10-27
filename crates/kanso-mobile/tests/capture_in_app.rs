//! Saving what is shared from inside the app (photos, the camera, a PDF) as
//! a note in the Inbox, and opening today's daily note: what they make, and
//! what they never put into a locked notebook.

mod common;

use std::sync::Arc;

use common::*;
use kanso_core::DocType;
use kanso_core::roles::Role;
use kanso_mobile::{KansoError, Library, LocalTime, Share, ShareFile};
use kanso_store::{Origin, Put, Store};
use kanso_sync::MemoryServer;
use proptest::prelude::*;
use serde_json::json;

const PDF: &[u8] = b"%PDF-1.7\n%%EOF\n";
const CANARY: &str = "CANARY-TEXT-OF-A-LOCKED-TEMPLATE";

fn file(name: &str, bytes: &[u8]) -> ShareFile {
    ShareFile {
        name: name.to_owned(),
        content_type: None,
        bytes: bytes.to_vec(),
    }
}

fn text(text: &str) -> Share {
    Share {
        text: Some(text.to_owned()),
        ..Share::default()
    }
}

fn day(year: i32, month: u32, day: u32) -> LocalTime {
    LocalTime {
        year,
        month,
        day,
        hour: 9,
        minute: 30,
        second: 0,
    }
}

/// Locks notebook `id` the way a synced lock arrives.
fn lock_book(store: &Store, id: &str) {
    let lock = json!({"v": 1, "salt": "c2FsdA", "m": 64, "t": 1, "p": 1, "data": "notebook"});
    store
        .put(
            DocType::Book,
            json!({"_id": id, "kansoLock": lock}),
            Put::Merge,
            Origin::Sync,
        )
        .unwrap();
}

fn locked_inbox_phone() -> Phone {
    phone_with(&Arc::new(MemoryServer::new()), |store| {
        let inbox = store.role_book(Role::Inbox, Origin::Ui).unwrap();
        lock_book(store, &inbox);
    })
}

fn all_notes(library: &Library) -> usize {
    library.list(Default::default()).unwrap().rows.len()
}

// no-prop-test: the contract's sequence for one share, once.
#[test]
fn a_share_becomes_a_note_in_the_inbox_with_its_files() {
    let phone = lone_phone();
    let outcome = phone
        .library
        .capture(Share {
            subject: Some("Receipt".into()),
            text: Some("Lunch".into()),
            url: None,
            files: vec![file("Receipt.pdf", PDF), file("notes.txt", b"hello")],
        })
        .unwrap();
    assert!(outcome.note_id.starts_with("note:cap-"), "{outcome:?}");
    assert_eq!(outcome.problems.len(), 1, "{:?}", outcome.problems);
    assert!(outcome.problems[0].starts_with("notes.txt: Only images and PDFs"));

    let note = phone.library.note(&outcome.note_id).unwrap();
    assert_eq!(note.title, "Receipt");
    assert_eq!(note.notebook_id, "book:inbox");
    let body = note.body.unwrap();
    assert!(body.contains("[Receipt.pdf](inkdrop://file:cap-"), "{body}");

    // Each share is a note of its own, even when it is the same.
    let again = phone.library.capture(text("Same")).unwrap();
    let twice = phone.library.capture(text("Same")).unwrap();
    assert_ne!(again.note_id, twice.note_id);
    assert_eq!(all_notes(&phone.library), 3);
}

// no-prop-test: the refusals, once each.
#[test]
fn a_share_with_nothing_to_save_is_invalid_and_says_why() {
    let phone = lone_phone();
    let nothing = phone.library.capture(Share::default()).unwrap_err();
    assert!(matches!(nothing, KansoError::Invalid { .. }), "{nothing:?}");
    let zip = phone
        .library
        .capture(Share {
            files: vec![file("a.zip", b"PK")],
            ..Share::default()
        })
        .unwrap_err();
    match zip {
        KansoError::Invalid { message } => assert!(message.contains("a.zip"), "{message}"),
        other => panic!("{other:?}"),
    }
    assert_eq!(all_notes(&phone.library), 0);
}

// no-prop-test: a locked Inbox, once; the property below covers the shares.
#[test]
fn a_locked_inbox_refuses_a_share() {
    let phone = locked_inbox_phone();
    let refused = phone.library.capture(text("Secret")).unwrap_err();
    assert_eq!(refused, KansoError::Locked);
    assert_eq!(all_notes(&phone.library), 0);
}

// no-prop-test: calendar days and clock times that don't exist, once each.
#[test]
fn today_is_made_once_and_a_day_that_does_not_exist_is_refused() {
    let phone = lone_phone();
    let first = phone.library.today_note(day(2026, 10, 26)).unwrap();
    assert_eq!(first, "note:daily-2026-10-26");
    let later = LocalTime {
        hour: 23,
        minute: 59,
        second: 59,
        ..day(2026, 10, 26)
    };
    assert_eq!(phone.library.today_note(later).unwrap(), first);
    assert_eq!(all_notes(&phone.library), 1);
    let note = phone.library.note(&first).unwrap();
    assert_eq!(note.title, "2026-10-26");
    assert_eq!(note.notebook_id, "book:journal");

    let next = phone.library.today_note(day(2026, 10, 27)).unwrap();
    assert_ne!(next, first);

    for bad in [
        day(2026, 2, 30),
        day(2026, 13, 1),
        day(2026, 0, 1),
        LocalTime {
            hour: 24,
            ..day(2026, 1, 1)
        },
        LocalTime {
            minute: 60,
            ..day(2026, 1, 1)
        },
    ] {
        assert!(
            matches!(
                phone.library.today_note(bad),
                Err(KansoError::Invalid { .. })
            ),
            "{bad:?}"
        );
    }
    assert_eq!(all_notes(&phone.library), 2);
}

// no-prop-test: the locked-template and locked-Journal cases, once each.
#[test]
fn today_in_a_locked_journal_leaks_no_text_from_a_locked_template() {
    let phone = phone_with(&Arc::new(MemoryServer::new()), |store| {
        let vault = book(store, "Vault");
        let template = store
            .put(
                DocType::Note,
                json!({"title": "Daily", "body": CANARY, "bookId": vault}),
                Put::Upsert,
                Origin::Ui,
            )
            .unwrap()
            .id;
        lock_book(store, &vault);
        store
            .set_daily_template(Some(&template), Origin::Api)
            .unwrap();
        let journal = store.role_book(Role::Journal, Origin::Ui).unwrap();
        lock_book(store, &journal);
    });

    let id = phone.library.today_note(day(2026, 10, 26)).unwrap();
    assert_eq!(id, "note:daily-2026-10-26");
    let store = Store::open(phone.path()).unwrap();
    let stored = store.note(&id).unwrap().unwrap();
    assert!(!stored.body.contains(CANARY), "{}", stored.body);
    let view = phone.library.note(&id).unwrap();
    assert!(view.body.is_none(), "{:?}", view.body);
    // Asking again finds the same note.
    assert_eq!(phone.library.today_note(day(2026, 10, 26)).unwrap(), id);
}

/// One file a share can carry.
fn any_file() -> impl Strategy<Value = ShareFile> {
    prop::sample::select(vec![
        ("a.pdf", PDF.to_vec()),
        ("b.png", b"not really a png".to_vec()),
        ("c.txt", b"plain".to_vec()),
        ("d.zip", b"PK".to_vec()),
        ("", Vec::new()),
    ])
    .prop_map(|(name, bytes)| ShareFile {
        name: name.to_owned(),
        content_type: None,
        bytes,
    })
}

fn any_share() -> impl Strategy<Value = Share> {
    (
        prop::option::of("[ -~]{0,20}"),
        prop::option::of("[ -~]{0,40}"),
        prop::option::of("https://[a-z]{1,8}\\.io/[a-z]{0,6}"),
        prop::collection::vec(any_file(), 0..4),
    )
        .prop_map(|(subject, text, url, files)| Share {
            subject,
            text,
            url,
            files,
        })
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(24))]

    /// Invariant: a share is never written into a locked Inbox (the answer
    /// is `Locked`, and the library holds no note), and into an open one it
    /// is either refused as nothing to save or returns a note that exists.
    #[test]
    fn prop_capture_never_writes_into_a_locked_inbox_and_always_returns_a_real_note(
        shares in prop::collection::vec(any_share(), 1..4),
        locked in any::<bool>(),
    ) {
        let phone = if locked { locked_inbox_phone() } else { lone_phone() };
        let mut made = 0;
        for share in shares {
            match phone.library.capture(share) {
                Ok(outcome) => {
                    prop_assert!(!locked, "{outcome:?}");
                    made += 1;
                    let note = phone.library.note(&outcome.note_id).unwrap();
                    prop_assert_eq!(note.notebook_id.as_str(), "book:inbox");
                }
                Err(KansoError::Locked) => prop_assert!(locked),
                Err(KansoError::Invalid { .. }) => {}
                Err(other) => prop_assert!(false, "{other:?}"),
            }
        }
        prop_assert_eq!(all_notes(&phone.library), made);
    }

    /// Invariant: today's note for any real day is made once, under that
    /// day's ID, and asking again, at any time of that day, finds it.
    #[test]
    fn prop_today_is_one_note_per_day(
        year in 1990i32..2100,
        month in 1u32..=12,
        day_of_month in 1u32..=28,
        hour in 0u32..24,
        minute in 0u32..60,
    ) {
        let phone = lone_phone();
        let morning = LocalTime { year, month, day: day_of_month, hour: 0, minute: 0, second: 0 };
        let moment = LocalTime { hour, minute, ..morning };
        let first = phone.library.today_note(morning).unwrap();
        prop_assert_eq!(
            first.as_str(),
            format!("note:daily-{year:04}-{month:02}-{day_of_month:02}")
        );
        prop_assert_eq!(phone.library.today_note(moment).unwrap(), first);
        prop_assert_eq!(all_notes(&phone.library), 1);
    }
}
