use kanso_core::dates::{Date, DateTime};
use kanso_core::tasks::task_items;
use proptest::prelude::*;
use proptest::test_runner::{Config, RngAlgorithm, RngSeed};
use serde_json::json;

use super::*;

/// Thursday 12 November 2026, 10:30.
fn now() -> DateTime {
    DateTime {
        date: Date::new(2026, 11, 12).unwrap(),
        hour: 10,
        minute: 30,
        second: 0,
    }
}

fn fresh() -> (tempfile::TempDir, Store) {
    let dir = tempfile::tempdir().unwrap();
    let store = Store::open(dir.path().join("kanso.db")).unwrap();
    (dir, store)
}

fn line(text: &str) -> Option<String> {
    task_line(text, now(), DateOrder::DayMonth)
}

fn lock(store: &Store, book: &str) {
    store
        .put(
            DocType::Book,
            json!({"_id": book, "kansoLock": {"v": 1, "salt": "c2FsdA", "m": 64, "t": 1, "p": 1, "data": "notebook"}}),
            Put::Merge,
            Origin::Sync,
        )
        .unwrap();
}

#[test]
fn task_lines() {
    assert_eq!(line("  \n "), None);
    assert_eq!(line("todo"), None);
    assert_eq!(line("[ ]"), None);
    assert_eq!(line("buy milk").unwrap(), "- [ ] buy milk");
    assert_eq!(
        line("film Pixel review fri 3pm every week").unwrap(),
        "- [ ] film Pixel review @2026-11-13T15:00 every week"
    );
    assert_eq!(
        line("film Pixel review fri 3pm").unwrap(),
        "- [ ] film Pixel review @2026-11-13T15:00"
    );
    // Markers say "task" and are not repeated.
    for typed in [
        "[] call Sam",
        "- [ ] call Sam",
        "TODO call Sam",
        "todo: call Sam",
    ] {
        assert_eq!(line(typed).unwrap(), "- [ ] call Sam", "{typed}");
    }
    // A word that only starts like the marker is a word.
    assert_eq!(
        line("todos for the team").unwrap(),
        "- [ ] todos for the team"
    );
    // The whole text is one task.
    assert_eq!(
        line("call Sam\nabout the lease  tomorrow").unwrap(),
        "- [ ] call Sam about the lease @2026-11-13"
    );
    // A phrase alone, with nothing to do, stays words.
    assert_eq!(line("fri").unwrap(), "- [ ] fri");
}

#[test]
fn hints() {
    let hint = |t: &str| task_hint(t, now(), DateOrder::DayMonth);
    assert_eq!(
        hint("film Pixel review fri 3pm every week").unwrap(),
        "Fri 13 Nov, 15:00 \u{b7} every week"
    );
    assert_eq!(hint("buy milk"), None);
    assert_eq!(hint("fri"), None);
    assert_eq!(hint(""), None);
}

#[test]
fn appending() {
    assert_eq!(append_task("", "- [ ] a"), "- [ ] a\n");
    assert_eq!(append_task("# Day\n", "- [ ] a"), "# Day\n\n- [ ] a\n");
    assert_eq!(
        append_task("- [ ] a\n- [x] b\n\n", "- [ ] c"),
        "- [ ] a\n- [x] b\n- [ ] c\n"
    );
}

/// The default target: a captured task lands in today's daily note, made
/// with its title, and the next one joins it.
#[test]
fn a_captured_task_lands_in_todays_daily_note() {
    let (_dir, store) = fresh();
    let id = save_task(
        &store,
        "film Pixel review fri 3pm every week",
        TaskTarget::default(),
        now(),
        DateOrder::DayMonth,
        Origin::Ui,
    )
    .unwrap()
    .unwrap();
    let note = store.note(&id).unwrap().unwrap();
    assert_eq!(note.title, "2026-11-12");
    assert_eq!(
        note.body,
        "- [ ] film Pixel review @2026-11-13T15:00 every week\n"
    );
    assert_eq!(
        store.daily_dates().unwrap(),
        vec![Date::new(2026, 11, 12).unwrap()]
    );
    let again = save_task(
        &store,
        "water plants",
        TaskTarget::DailyNote,
        now(),
        DateOrder::DayMonth,
        Origin::Ui,
    )
    .unwrap()
    .unwrap();
    assert_eq!(again, id);
    let items = task_items(&store.note(&id).unwrap().unwrap().body);
    assert_eq!(items.len(), 2);
    assert_eq!(items[0].due.as_deref(), Some("2026-11-13T15:00"));
    assert_eq!(items[1].text, "water plants");
    // Nothing to add: nothing written.
    assert_eq!(
        save_task(
            &store,
            " todo ",
            TaskTarget::DailyNote,
            now(),
            DateOrder::DayMonth,
            Origin::Ui
        )
        .unwrap(),
        None
    );
}

/// With the Inbox as the target, no daily note is made; the tasks gather
/// in one note in the Inbox.
#[test]
fn the_inbox_setting_is_honoured() {
    let (_dir, store) = fresh();
    let target = TaskTarget::Inbox;
    let first = save_task(
        &store,
        "call Sam tomorrow",
        target,
        now(),
        DateOrder::DayMonth,
        Origin::Ui,
    )
    .unwrap()
    .unwrap();
    let second = save_task(
        &store,
        "buy milk",
        target,
        now(),
        DateOrder::DayMonth,
        Origin::Ui,
    )
    .unwrap()
    .unwrap();
    assert_eq!(first, INBOX_TASKS_ID);
    assert_eq!(second, first);
    let note = store.note(&first).unwrap().unwrap();
    assert_eq!(note.book_id, "book:inbox");
    assert_eq!(note.title, INBOX_TASKS_TITLE);
    assert_eq!(note.body, "- [ ] call Sam @2026-11-13\n- [ ] buy milk\n");
    assert!(store.daily_dates().unwrap().is_empty());
}

/// An Inbox task note that was trashed comes back for the next task.
#[test]
fn a_trashed_inbox_task_note_comes_back() {
    let (_dir, store) = fresh();
    let target = TaskTarget::Inbox;
    save_task(
        &store,
        "one",
        target,
        now(),
        DateOrder::DayMonth,
        Origin::Ui,
    )
    .unwrap();
    store
        .put(
            DocType::Note,
            json!({"_id": INBOX_TASKS_ID, "bookId": TRASH_BOOK_ID}),
            Put::Merge,
            Origin::Ui,
        )
        .unwrap();
    save_task(
        &store,
        "two",
        target,
        now(),
        DateOrder::DayMonth,
        Origin::Ui,
    )
    .unwrap();
    let note = store.note(INBOX_TASKS_ID).unwrap().unwrap();
    assert_eq!(note.book_id, "book:inbox");
    assert_eq!(note.body, "- [ ] one\n- [ ] two\n");
}

/// A locked Inbox or a locked Journal takes no task.
#[test]
fn locked_notebooks_refuse_tasks() {
    let (_dir, store) = fresh();
    store.role_book(Role::Inbox, Origin::Ui).unwrap();
    lock(&store, "book:inbox");
    let err = save_task(
        &store,
        "x",
        TaskTarget::Inbox,
        now(),
        DateOrder::DayMonth,
        Origin::Ui,
    )
    .unwrap_err();
    assert!(matches!(err, CaptureError::Locked), "{err}");

    let (_dir, store) = fresh();
    let journal = store.role_book(Role::Journal, Origin::Ui).unwrap();
    lock(&store, &journal);
    let err = save_task(
        &store,
        "x",
        TaskTarget::DailyNote,
        now(),
        DateOrder::DayMonth,
        Origin::Ui,
    )
    .unwrap_err();
    assert!(matches!(err, CaptureError::Locked), "{err}");
    assert!(store.daily_dates().unwrap().is_empty());
}

fn arb_body() -> impl Strategy<Value = String> {
    "[a-z#\\[\\]x*+ \n-]{0,120}"
}

proptest! {
    #![proptest_config(Config {
        cases: 512,
        rng_seed: RngSeed::Fixed(20_261_017),
        rng_algorithm: RngAlgorithm::ChaCha,
        failure_persistence: None,
        ..Config::default()
    })]

    /// Invariants of the line: one line, an open task item that the task
    /// reader finds, and no task for text with no words.
    #[test]
    fn prop_task_line_is_one_open_task(text in "[a-z0-9\\[\\] :*+\n-]{0,80}", day in 1u32..28) {
        let at = DateTime { date: Date::new(2026, 11, day).unwrap(), ..now() };
        match task_line(&text, at, DateOrder::DayMonth) {
            None => {
                let rest = without_task_marker(&text);
                prop_assert!(rest.trim().is_empty());
            }
            Some(task) => {
                prop_assert!(!task.contains('\n'));
                prop_assert!(task.starts_with("- [ ] "));
                let items = task_items(&task);
                prop_assert_eq!(items.len(), 1);
                prop_assert!(!items[0].checked);
                prop_assert!(!items[0].text.is_empty());
            }
        }
    }

    /// The hint is there exactly when the line carries a due date.
    #[test]
    fn prop_hint_matches_the_line(text in "[a-z \n]{0,30}", phrase in prop::sample::select(vec!["", " fri 3pm", " tomorrow", " fri 3pm every week"])) {
        let typed = format!("{text}{phrase}");
        let task = task_line(&typed, now(), DateOrder::DayMonth);
        let hint = task_hint(&typed, now(), DateOrder::DayMonth);
        let dated = task.as_deref().is_some_and(|t| task_items(t).first().is_some_and(|i| i.due.is_some()));
        prop_assert_eq!(hint.is_some(), dated);
    }

    /// A line made once is a fixed point: typing it back, marker and all,
    /// writes the same line.
    #[test]
    fn prop_task_line_is_stable(text in "[a-z \n]{0,40}", phrase in prop::sample::select(vec!["", " fri 3pm", " tomorrow", " 14 nov", " fri 3pm every week", " in 2 days"])) {
        let typed = format!("{text}{phrase}");
        if let Some(task) = task_line(&typed, now(), DateOrder::DayMonth) {
            prop_assert_eq!(task_line(&task, now(), DateOrder::DayMonth), Some(task));
        }
    }

    /// Appending keeps everything that was there, adds exactly one task at
    /// the end, and leaves a body that ends in one newline.
    #[test]
    fn prop_append_keeps_the_body(body in arb_body(), text in "[a-z]{1,10}( [a-z]{1,10}){0,4}") {
        let added = format!("- [ ] {text}");
        let out = append_task(&body, &added);
        prop_assert!(out.starts_with(body.trim_end()));
        let tail = format!("{added}\n");
        prop_assert!(out.ends_with(&tail));
        prop_assert!(!out.ends_with("\n\n"));
        prop_assert_eq!(out.lines().filter(|l| *l == added).count(), 1 + body.lines().filter(|l| *l == added).count());
    }

    /// Tasks already in a body are all still there, in order, after one is
    /// appended; the new one is last.
    #[test]
    fn prop_append_keeps_the_tasks(
        existing in prop::collection::vec("[a-z]{1,8}", 0..5),
        new in "[a-z]{1,8}",
    ) {
        let mut body = String::new();
        for word in &existing {
            body = append_task(&body, &format!("- [ ] {word}"));
        }
        let out = append_task(&body, &format!("- [ ] {new}"));
        let texts: Vec<String> = task_items(&out).into_iter().map(|t| t.text).collect();
        let mut want = existing.clone();
        want.push(new);
        prop_assert_eq!(texts, want);
    }
}

#[test]
fn previews_confirm_like_the_spec() {
    let preview = |t: &str| task_preview(t, now(), DateOrder::DayMonth);
    let dated = preview("film Pixel review fri 3pm every week").unwrap();
    assert_eq!(
        dated.line,
        "- [ ] film Pixel review @2026-11-13T15:00 every week"
    );
    assert_eq!(dated.words, "film Pixel review");
    assert_eq!(
        dated.confirmation(),
        "Added: film Pixel review \u{b7} Fri 13 Nov 15:00 \u{b7} every week"
    );
    let plain = preview("buy milk").unwrap();
    assert_eq!(plain.when, None);
    assert_eq!(plain.confirmation(), "Added: buy milk");
    let day_only = preview("call Sam tomorrow").unwrap();
    assert_eq!(day_only.confirmation(), "Added: call Sam \u{b7} Fri 13 Nov");
    assert_eq!(preview("  "), None);
}

#[test]
fn checked_lines() {
    assert_eq!(checked_task_line(" - [ ] a \n"), Some("- [ ] a"));
    assert_eq!(checked_task_line("- [ ] "), None);
    assert_eq!(checked_task_line("- [x] a"), None);
    assert_eq!(checked_task_line("# a"), None);
    assert_eq!(checked_task_line("- [ ] a\n- [ ] b"), None);
    assert_eq!(checked_task_line("- [ ] a\rb"), None);
    let long = format!("- [ ] {}", "a".repeat(MAX_TASK_LINE));
    assert_eq!(checked_task_line(&long), None);
}

/// A line written from a drop lands where the typed task would, and a line
/// that isn't one task is refused with nothing written.
#[test]
fn a_task_line_from_a_drop() {
    let (_dir, store) = fresh();
    let id = save_task_line(
        &store,
        "- [ ] film Pixel review @2026-11-13T15:00 every week",
        TaskTarget::Inbox,
        now(),
        Origin::Ui,
    )
    .unwrap();
    assert_eq!(id, INBOX_TASKS_ID);
    assert_eq!(
        store.note(&id).unwrap().unwrap().body,
        "- [ ] film Pixel review @2026-11-13T15:00 every week\n"
    );
    let err = save_task_line(
        &store,
        "# not a task",
        TaskTarget::DailyNote,
        now(),
        Origin::Ui,
    )
    .unwrap_err();
    assert!(matches!(err, CaptureError::Store(StoreError::Invalid(_))));
    assert!(store.daily_dates().unwrap().is_empty());
}

proptest! {
    #![proptest_config(Config {
        cases: 256,
        rng_seed: RngSeed::Fixed(20_261_017),
        rng_algorithm: RngAlgorithm::ChaCha,
        failure_persistence: None,
        ..Config::default()
    })]

    /// The preview's line is the line that is written, its confirmation
    /// names the words, and it carries a date exactly when the hint does.
    #[test]
    fn prop_preview_agrees_with_the_line(
        text in "[a-z \n]{0,30}",
        phrase in prop::sample::select(vec!["", " fri 3pm", " tomorrow", " 14 nov", " fri 3pm every week"]),
    ) {
        let typed = format!("{text}{phrase}");
        let preview = task_preview(&typed, now(), DateOrder::DayMonth);
        prop_assert_eq!(preview.as_ref().map(|p| p.line.clone()), task_line(&typed, now(), DateOrder::DayMonth));
        prop_assert_eq!(
            preview.as_ref().and_then(|p| p.when.clone()).is_some(),
            task_hint(&typed, now(), DateOrder::DayMonth).is_some()
        );
        if let Some(preview) = preview {
            let start = format!("Added: {}", preview.words);
            prop_assert!(preview.confirmation().starts_with(&start));
            prop_assert!(!preview.confirmation().contains(", "));
        }
    }

    /// Every line the parser writes passes the check unchanged.
    #[test]
    fn prop_every_task_line_is_checked_through(text in "[a-z0-9\\[\\] :*+\n-]{0,80}") {
        if let Some(task) = task_line(&text, now(), DateOrder::DayMonth) {
            prop_assert_eq!(checked_task_line(&task), Some(task.as_str()));
        }
    }

    /// Whatever a drop says, what passes the check is one open task line
    /// with words.
    #[test]
    fn prop_checked_lines_are_one_open_task(line in "[- \\[\\]xa-z\n\r]{0,40}") {
        if let Some(checked) = checked_task_line(&line) {
            prop_assert!(!checked.contains(['\n', '\r']));
            let items = task_items(checked);
            prop_assert_eq!(items.len(), 1);
            prop_assert!(!items[0].checked);
        }
    }

    /// Writing the parsed line gives the same note as saving the typed text.
    #[test]
    fn prop_saving_the_line_is_saving_the_text(
        text in "[a-z]{1,8}( [a-z]{1,8}){0,3}",
        phrase in prop::sample::select(vec!["", " fri 3pm", " tomorrow every week"]),
        inbox in any::<bool>(),
    ) {
        let typed = format!("{text}{phrase}");
        let target = if inbox { TaskTarget::Inbox } else { TaskTarget::DailyNote };
        let (_a, typed_store) = fresh();
        let (_b, line_store) = fresh();
        let typed_id = save_task(&typed_store, &typed, target, now(), DateOrder::DayMonth, Origin::Ui)
            .unwrap()
            .unwrap();
        let line = task_line(&typed, now(), DateOrder::DayMonth).unwrap();
        let line_id = save_task_line(&line_store, &line, target, now(), Origin::Ui).unwrap();
        prop_assert_eq!(&typed_id, &line_id);
        prop_assert_eq!(
            typed_store.note(&typed_id).unwrap().unwrap().body,
            line_store.note(&line_id).unwrap().unwrap().body
        );
    }
}
