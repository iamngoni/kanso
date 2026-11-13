//! Quick Capture's Task mode on the phone: a typed task written into today's
//! daily note or the Inbox's task note, with the hint line under the field,
//! the same engine code as the desktop.

mod common;

use std::sync::Arc;

use common::*;
use kanso_core::DocType;
use kanso_core::roles::Role;
use kanso_mobile::{CaptureTarget, KansoError, Library, LocalTime};
use kanso_store::{Origin, Put, Store};
use kanso_sync::MemoryServer;
use proptest::prelude::*;
use serde_json::json;

/// Tuesday 20 October 2026, 10:00.
fn now() -> LocalTime {
    LocalTime {
        year: 2026,
        month: 10,
        day: 20,
        hour: 10,
        minute: 0,
        second: 0,
    }
}

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

fn body(library: &Library, id: &str) -> String {
    library.note(id).unwrap().body.unwrap()
}

fn notes(library: &Library) -> usize {
    library.list(Default::default()).unwrap().rows.len()
}

// no-prop-test: the contract's sequence for one task, once; the properties
// below cover the words.
#[test]
fn a_task_with_a_date_and_repeat_goes_to_the_daily_note() {
    let phone = lone_phone();
    let library = &phone.library;
    let saved = library
        .capture_task(
            "film Pixel review fri 3pm every week",
            CaptureTarget::DailyNote,
            now(),
            false,
        )
        .unwrap();
    assert_eq!(
        saved.line,
        "- [ ] film Pixel review @2026-10-23T15:00 every week"
    );
    assert_eq!(saved.note_id, library.today_note(now()).unwrap());
    assert_eq!(saved.note_id, "note:daily-2026-10-20");
    assert!(
        body(library, &saved.note_id)
            .ends_with("- [ ] film Pixel review @2026-10-23T15:00 every week\n"),
        "{}",
        body(library, &saved.note_id)
    );
    // The next one lands under it: one list.
    let again = library
        .capture_task("water plants", CaptureTarget::DailyNote, now(), false)
        .unwrap();
    assert_eq!(again.note_id, saved.note_id);
    assert!(
        body(library, &saved.note_id).ends_with(
            "- [ ] film Pixel review @2026-10-23T15:00 every week\n- [ ] water plants\n"
        ),
        "{}",
        body(library, &saved.note_id)
    );
}

// no-prop-test: the Inbox target's sequence, once.
#[test]
fn a_task_for_the_inbox_goes_to_its_task_note_and_collects_there() {
    let phone = lone_phone();
    let library = &phone.library;
    let first = library
        .capture_task(
            "todo: buy tape tomorrow",
            CaptureTarget::Inbox,
            now(),
            false,
        )
        .unwrap();
    let second = library
        .capture_task("[] call Sam", CaptureTarget::Inbox, now(), false)
        .unwrap();
    assert_eq!(first.note_id, "note:inbox-tasks");
    assert_eq!(second.note_id, first.note_id);
    let note = library.note(&first.note_id).unwrap();
    assert_eq!(note.title, "Inbox tasks");
    assert_eq!(note.notebook_id, "book:inbox");
    assert_eq!(
        note.body.unwrap(),
        "- [ ] buy tape @2026-10-21\n- [ ] call Sam\n"
    );
}

// no-prop-test: how a slash date is read, once each way.
#[test]
fn the_phone_says_which_way_a_slash_date_reads() {
    let day_first = Library::capture_task_hint("pay 14/11", now(), false).unwrap();
    let month_first = Library::capture_task_hint("pay 11/14", now(), true).unwrap();
    assert_eq!(day_first, month_first);
    assert!(day_first.unwrap().contains("14 Nov"));
}

// no-prop-test: the hint for one phrase, as the desktop words it.
#[test]
fn the_hint_reads_the_date_and_repeat_and_is_silent_otherwise() {
    let hint =
        Library::capture_task_hint("film Pixel review fri 3pm every week", now(), false).unwrap();
    assert_eq!(hint.as_deref(), Some("Fri 23 Oct, 15:00 \u{b7} every week"));
    assert_eq!(
        Library::capture_task_hint("film review", now(), false).unwrap(),
        None
    );
    assert_eq!(
        Library::capture_task_hint("fri", now(), false).unwrap(),
        None
    );
    assert_eq!(
        Library::capture_task_hint("   ", now(), false).unwrap(),
        None
    );
}

// no-prop-test: text with no task in it, and a time that does not exist.
#[test]
fn text_with_no_task_is_invalid_and_writes_nothing() {
    let phone = lone_phone();
    for text in ["", "   ", "todo", "[ ]", "- [ ]  "] {
        let refused = phone
            .library
            .capture_task(text, CaptureTarget::DailyNote, now(), false)
            .unwrap_err();
        assert!(
            matches!(refused, KansoError::Invalid { .. }),
            "{text:?}: {refused:?}"
        );
    }
    let bad_time = LocalTime { hour: 25, ..now() };
    let refused = phone
        .library
        .capture_task("x", CaptureTarget::Inbox, bad_time, false)
        .unwrap_err();
    assert!(matches!(refused, KansoError::Invalid { .. }), "{refused:?}");
    assert_eq!(notes(&phone.library), 0);
}

// no-prop-test: the two locked destinations, once each.
#[test]
fn a_locked_inbox_or_journal_takes_no_task() {
    let inbox = phone_with(&Arc::new(MemoryServer::new()), |store| {
        let id = store.role_book(Role::Inbox, Origin::Ui).unwrap();
        lock_book(store, &id);
    });
    let refused = inbox
        .library
        .capture_task("secret", CaptureTarget::Inbox, now(), false)
        .unwrap_err();
    assert_eq!(refused, KansoError::Locked);
    assert_eq!(notes(&inbox.library), 0);

    let journal = phone_with(&Arc::new(MemoryServer::new()), |store| {
        let id = store.role_book(Role::Journal, Origin::Ui).unwrap();
        lock_book(store, &id);
    });
    let refused = journal
        .library
        .capture_task("secret", CaptureTarget::DailyNote, now(), false)
        .unwrap_err();
    assert_eq!(refused, KansoError::Locked);
    assert_eq!(notes(&journal.library), 0);
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(24))]

    /// Plain words (no date in them) are written as one open task, each
    /// capture adds exactly one line at the end, and the lines keep the order
    /// they were captured in, in either place.
    #[test]
    fn prop_tasks_collect_in_order(
        words in prop::collection::vec("[a-z]{1,8}( [a-z]{1,8}){0,3}", 1..6),
        inbox in any::<bool>(),
    ) {
        let phone = lone_phone();
        let target = if inbox { CaptureTarget::Inbox } else { CaptureTarget::DailyNote };
        let mut id = String::new();
        for (n, text) in words.iter().enumerate() {
            // The words could end in a weekday or a month; leave those out.
            prop_assume!(Library::capture_task_hint(text, now(), false).unwrap().is_none());
            let saved = phone.library.capture_task(text, target, now(), false).unwrap();
            prop_assert_eq!(&saved.line, &format!("- [ ] {text}"));
            if n == 0 {
                id = saved.note_id.clone();
            }
            prop_assert_eq!(&saved.note_id, &id);
            let written = body(&phone.library, &id);
            let tail = format!("{}\n", saved.line);
            prop_assert!(written.ends_with(&tail), "{}", written);
            prop_assert_eq!(written.matches("- [ ] ").count(), n + 1);
        }
        let written = body(&phone.library, &id);
        let tasks: Vec<&str> = written.lines().filter(|l| l.starts_with("- [ ] ")).collect();
        let expected: Vec<String> = words.iter().map(|w| format!("- [ ] {w}")).collect();
        prop_assert_eq!(tasks, expected.iter().map(String::as_str).collect::<Vec<_>>());
    }

    /// The hint never changes with the destination, and a task that shows a
    /// hint is written with its date as the canonical token, never as typed.
    #[test]
    fn prop_the_hint_and_the_line_agree(
        words in "[a-z]{2,8}( [a-z]{2,8}){0,2}",
        phrase in prop::sample::select(vec![
            "tomorrow", "today", "fri 3pm", "in 3 days", "next mon", "tomorrow 9am every week",
            "every weekday", "14 nov", "tonight",
        ]),
        month_first in any::<bool>(),
    ) {
        let text = format!("{words} {phrase}");
        let hint = Library::capture_task_hint(&text, now(), month_first).unwrap();
        let phone = lone_phone();
        let saved = phone
            .library
            .capture_task(&text, CaptureTarget::Inbox, now(), month_first)
            .unwrap();
        prop_assert_eq!(hint.is_some(), saved.line.contains(" @20"), "{} -> {}", text, saved.line);
        prop_assert!(!saved.line.contains(phrase) || phrase.starts_with("every"));
        prop_assert!(saved.line.starts_with("- [ ] "));
        prop_assert_eq!(notes(&phone.library), 1);
    }
}
