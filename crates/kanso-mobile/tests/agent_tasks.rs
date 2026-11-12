//! Agent tasks on the phone never reach a locked, trashed or `#ai-exclude`
//! note, whatever the model asks for, and a task with nothing to apply fails
//! to review rather than guessing.

mod common;

use std::time::{Duration, Instant};

use common::model::*;
use common::*;
use kanso_core::DocType;
use kanso_core::model::TRASH_BOOK_ID;
use kanso_store::{Origin, Put, Store};
use proptest::prelude::*;
use serde_json::json;

/// How many notes of each kind a library has.
#[derive(Clone, Debug)]
struct Library {
    plain: usize,
    sealed: usize,
    in_locked_book: usize,
    in_nested_book: usize,
    trashed: usize,
    excluded: usize,
}

fn libraries() -> impl Strategy<Value = Library> {
    (
        1usize..4,
        0usize..3,
        0usize..3,
        0usize..3,
        0usize..3,
        0usize..3,
    )
        .prop_map(
            |(plain, sealed, in_locked_book, in_nested_book, trashed, excluded)| Library {
                plain,
                sealed,
                in_locked_book,
                in_nested_book,
                trashed,
                excluded,
            },
        )
}

/// A kept-out note's words, none of which appears in an ordinary note.
struct Hidden {
    id: String,
    title: String,
    body: String,
}

struct Seeded {
    plain: Vec<Hidden>,
    hidden: Vec<Hidden>,
}

fn kept_out(store: &Store, kind: &str, i: usize, fields: serde_json::Value) -> Hidden {
    let title = format!("Zqx{kind}{i}Title");
    let body = format!("Zqx{kind}{i}Body");
    let mut all = json!({"title": title, "body": format!("quokka {body} quokka")});
    all.as_object_mut()
        .unwrap()
        .extend(fields.as_object().unwrap().clone());
    // As a sync writes it: a locked notebook refuses writes from the UI.
    Hidden {
        id: store
            .put(DocType::Note, all, Put::Upsert, Origin::Sync)
            .unwrap()
            .id,
        title,
        body,
    }
}

fn seed(store: &Store, shape: &Library) -> Seeded {
    let notes = book(store, "Notes");
    let vault = store
        .put(
            DocType::Book,
            json!({"name": "Vault", "kansoLock": {"salt": "abc"}}),
            Put::Upsert,
            Origin::Ui,
        )
        .unwrap()
        .id;
    let inner = book(store, "Inner");
    store
        .put(
            DocType::Book,
            json!({"_id": inner, "parentBookId": vault}),
            Put::Merge,
            Origin::Ui,
        )
        .unwrap();
    let exclude = tag(store, "AI-Exclude");
    let mut plain = Vec::new();
    for i in 0..shape.plain {
        plain.push(Hidden {
            id: raw_note(
                store,
                json!({"title": format!("Plainly{i}"),
                    "body": format!("quokka facts Plainbody{i}"), "bookId": notes}),
            ),
            title: format!("Plainly{i}"),
            body: format!("Plainbody{i}"),
        });
    }
    let mut hidden = Vec::new();
    for i in 0..shape.sealed {
        let note = kept_out(store, "Sealed", i, json!({"bookId": notes}));
        // Locked on its own: the lock field is there, the body stays so that
        // any leak shows.
        store
            .put(
                DocType::Note,
                json!({"_id": note.id, "kansoLock": {"salt": "abc"}}),
                Put::Merge,
                Origin::Sync,
            )
            .unwrap();
        hidden.push(note);
    }
    for i in 0..shape.in_locked_book {
        hidden.push(kept_out(store, "Vault", i, json!({"bookId": vault})));
    }
    for i in 0..shape.in_nested_book {
        hidden.push(kept_out(store, "Nested", i, json!({"bookId": inner})));
    }
    for i in 0..shape.trashed {
        hidden.push(kept_out(
            store,
            "Trash",
            i,
            json!({"bookId": TRASH_BOOK_ID}),
        ));
    }
    for i in 0..shape.excluded {
        hidden.push(kept_out(
            store,
            "Excl",
            i,
            json!({"bookId": notes, "tags": [exclude]}),
        ));
    }
    Seeded { plain, hidden }
}

/// Starts a library with the lock session open (a note of its own is locked
/// with the password, which leaves the session unlocked), then seeds it.
fn library_for(shape: &Library) -> (Phone, Seeded, Hidden) {
    let phone = lone_phone();
    let anchor_id = note(&phone.library, "Zqxanchor", "quokka Zqxanchorbody");
    phone
        .library
        .locks()
        .lock_note(&anchor_id, LOCK_PASSWORD.into())
        .unwrap();
    assert!(phone.library.locks().is_unlocked().unwrap());
    let store = Store::open(phone.path()).unwrap();
    let seeded = seed(&store, shape);
    let anchor = Hidden {
        id: anchor_id,
        title: "Zqxanchor".into(),
        body: "Zqxanchorbody".into(),
    };
    (phone, seeded, anchor)
}

fn assert_clean(sent: &str, hidden: &[Hidden], ids_too: bool) -> Result<(), TestCaseError> {
    for note in hidden {
        prop_assert!(!sent.contains(&note.title), "title {} was sent", note.title);
        prop_assert!(!sent.contains(&note.body), "body {} was sent", note.body);
        if ids_too {
            prop_assert!(!sent.contains(&note.id), "id {} was sent", note.id);
        }
    }
    Ok(())
}

fn wait_for_end(tasks: &kanso_mobile::AgentTasks) -> Vec<kanso_mobile::TaskView> {
    let start = Instant::now();
    loop {
        let all = tasks.tasks();
        if !all.is_empty()
            && all
                .iter()
                .all(|t| !matches!(t.phase.as_str(), "running" | "waiting"))
        {
            return all;
        }
        assert!(
            start.elapsed() < Duration::from_secs(60),
            "the task never ended: {all:?}"
        );
        std::thread::sleep(Duration::from_millis(20));
    }
}

proptest! {
    #![proptest_config(ProptestConfig {
        cases: 8,
        failure_persistence: None,
        ..ProptestConfig::default()
    })]

    /// A task that searches and then reads every kept-out note by ID is sent
    /// none of them.
    #[test]
    fn prop_mobile_task_never_returns_a_locked_note(shape in libraries()) {
        let (phone, seeded, anchor) = library_for(&shape);
        let mut hidden: Vec<Hidden> = seeded.hidden;
        hidden.push(anchor);
        let mut calls = vec![("search_notes", json!({"keyword": "quokka"}).to_string())];
        for note in &hidden {
            calls.push(("read_note", json!({"noteId": note.id}).to_string()));
        }
        calls.push(("read_note", json!({"noteId": seeded.plain[0].id}).to_string()));
        let model = Scripted::with("Done.", calls, Ending::Complete);
        let ai = ai_on(&phone.library, &model);
        let tasks = ai.agent_tasks();
        tasks.start("Tidy the quokka notes", Vec::new(), "", true).unwrap();
        let ended = wait_for_end(&tasks);
        let sent = model.everything_sent();
        assert_clean(&sent, &hidden, false)?;
        prop_assert!(sent.contains("Plainbody0"), "the plain notes must be sent: {ended:?} {sent}");
    }
}

// no-prop-test: AI off is one state.
#[test]
fn a_task_does_not_start_while_ai_is_off() {
    let phone = lone_phone();
    let model = Scripted::new("never");
    let ai = phone.library.ai_with_provider(maker_of(model.clone()));
    assert!(
        ai.agent_tasks()
            .start("Tidy", Vec::new(), "", true)
            .is_err()
    );
    assert_eq!(model.request_count(), 0);
}

// no-prop-test: a task that proposes nothing has nothing to review.
#[test]
fn a_task_with_no_changes_has_nothing_to_review() {
    let model = Scripted::new("Nothing to change.");
    let (phone, ai) = ai_phone(&model);
    note(&phone.library, "Pricing", "quokka");
    let tasks = ai.agent_tasks();
    let id = tasks.start("Tidy", Vec::new(), "", true).unwrap();
    wait_for_end(&tasks);
    assert!(tasks.review(id).is_err() || tasks.review(id).unwrap().cards.is_empty());
}

// no-prop-test: one scope, two notes; the property over shapes is above.
#[test]
fn a_task_started_on_note_ids_can_read_them_but_not_a_locked_one() {
    let shape = Library {
        plain: 1,
        sealed: 1,
        in_locked_book: 0,
        in_nested_book: 0,
        trashed: 0,
        excluded: 0,
    };
    let (phone, seeded, anchor) = library_for(&shape);
    let plain = &seeded.plain[0];
    let sealed = &seeded.hidden[0];
    let calls = vec![
        ("read_note", json!({"noteId": plain.id}).to_string()),
        ("read_note", json!({"noteId": sealed.id}).to_string()),
    ];
    let model = Scripted::with("Done.", calls, Ending::Complete);
    let ai = ai_on(&phone.library, &model);
    let tasks = ai.agent_tasks();
    tasks
        .start(
            "Tidy",
            vec![plain.id.clone(), sealed.id.clone(), anchor.id.clone()],
            "",
            false,
        )
        .unwrap();
    wait_for_end(&tasks);
    let sent = model.everything_sent();
    assert!(
        sent.contains(&plain.body),
        "the named note is readable: {sent}"
    );
    assert!(!sent.contains(&sealed.body) && !sent.contains(&anchor.body));
}

// no-prop-test: one gate, one answer.
#[test]
fn reading_every_notebook_needs_the_confirmation_settings_ask_for() {
    let model = Scripted::new("Nothing to change.");
    let (phone, ai) = ai_phone(&model);
    note(&phone.library, "Pricing", "quokka");
    let tasks = ai.agent_tasks();
    let refused = tasks.start("Tidy", Vec::new(), "", false).unwrap_err();
    assert!(refused.to_string().contains("Confirm"), "{refused}");
    assert_eq!(model.request_count(), 0);
    // Chosen notes need no such question; and Settings can turn it off.
    let mut prefs = tasks.prefs();
    prefs.ask_before_reading_all = false;
    tasks.set_prefs(prefs).unwrap();
    tasks.start("Tidy", Vec::new(), "", false).unwrap();
    wait_for_end(&tasks);
}

// no-prop-test: the limits are a round trip through the file.
#[test]
fn the_time_limit_and_note_limit_are_kept() {
    let model = Scripted::new("x");
    let (_phone, ai) = ai_phone(&model);
    let tasks = ai.agent_tasks();
    let mut prefs = tasks.prefs();
    assert_eq!((prefs.time_limit_minutes, prefs.max_notes), (20, 200));
    prefs.time_limit_minutes = 5;
    prefs.max_notes = 50;
    tasks.set_prefs(prefs).unwrap();
    let saved = tasks.prefs();
    assert_eq!((saved.time_limit_minutes, saved.max_notes), (5, 50));
}

// no-prop-test: one scenario; the engine's property covers apply and undo.
#[test]
fn ticking_a_card_that_changed_since_applies_over_it() {
    let (phone, _) = (lone_phone(), ());
    let id = note(&phone.library, "Pricing", "quokka one");
    let store = Store::open(phone.path()).unwrap();
    let rev = store.get(&id).unwrap().unwrap().rev;
    let calls = vec![(
        "update_note",
        json!({"_id": id, "_rev": rev, "body": "quokka two"}).to_string(),
    )];
    let model = Scripted::with("Done.", calls, Ending::Complete);
    let ai = ai_on(&phone.library, &model);
    let tasks = ai.agent_tasks();
    let task = tasks.start("Edit", vec![id.clone()], "", false).unwrap();
    wait_for_end(&tasks);
    // The person edits the note meanwhile.
    store
        .put(
            DocType::Note,
            json!({"_id": id, "body": "mine"}),
            Put::Merge,
            Origin::Ui,
        )
        .unwrap();
    let review = tasks.review(task).unwrap();
    assert_eq!(review.cards.len(), 1);
    assert!(review.cards[0].changed_since);
    let said = tasks.apply(task, vec![0], Vec::new(), 100).unwrap();
    assert!(said.contains("Applied"), "{said}");
    assert_eq!(store.note(&id).unwrap().unwrap().body, "quokka two");
    // Undo reports from what it did.
    assert!(tasks.undo(task, 200).is_ok());
}

/// A task that proposed one edit, finished, with the phone it ran on.
fn finished_edit() -> (Phone, kanso_mobile::AgentTasks) {
    let phone = lone_phone();
    let id = note(&phone.library, "Pricing", "quokka one");
    let store = Store::open(phone.path()).unwrap();
    let rev = store.get(&id).unwrap().unwrap().rev;
    let calls = vec![(
        "update_note",
        json!({"_id": id, "_rev": rev, "body": "quokka two"}).to_string(),
    )];
    let model = Scripted::with("Done.", calls, Ending::Complete);
    let ai = ai_on(&phone.library, &model);
    let tasks = ai.agent_tasks();
    tasks
        .start("Edit the pricing note", vec![id], "", false)
        .unwrap();
    wait_for_end(&tasks);
    (phone, tasks)
}

// no-prop-test: one scripted run, read back through the session calls.
#[test]
fn a_finished_task_is_a_session_row_that_needs_the_person() {
    let (_phone, tasks) = finished_edit();
    let rows = tasks.sessions(i64::MAX / 4);
    assert_eq!(rows.len(), 1);
    let row = &rows[0];
    assert_eq!(row.title, "Edit the pricing note");
    assert_eq!(row.agent, "Kanso AI");
    assert_eq!(row.phase, "ready");
    assert_eq!(row.status, "1 change to review");
    assert_eq!(row.dot, "needs_you");
    assert!(row.can_review && !row.can_stop && !row.can_follow_up);
    assert!(row.unread, "nobody has opened it yet");
    assert!(!row.key.is_empty());
}

// no-prop-test: opening is seeing; the transcript starts with the goal.
#[test]
fn opening_a_session_reads_its_transcript_and_clears_the_unread_dot() {
    let (_phone, tasks) = finished_edit();
    let key = tasks.sessions(0)[0].key.clone();
    let opened = tasks.transcript(&key, 0).expect("the session is listed");
    let first = &opened.items[0];
    assert_eq!(
        (first.kind.as_str(), first.text.as_str()),
        ("user", "Edit the pricing note")
    );
    assert!(opened.items.iter().any(|item| item.kind == "agent"));
    let proposed = opened
        .items
        .iter()
        .find(|item| item.kind == "proposed")
        .expect("the round's proposal is in the transcript");
    assert_eq!((proposed.round, proposed.state.as_str()), (1, "to_review"));
    assert_eq!(proposed.counts, "1 edit");
    assert!(!tasks.sessions(0)[0].unread);
    assert_eq!(tasks.transcript(&key, 0).unwrap().version, opened.version);
}

// no-prop-test: a key nobody has is the same answer whatever it says.
#[test]
fn a_session_that_is_not_listed_has_no_transcript_and_refuses_calls() {
    let (_phone, tasks) = finished_edit();
    assert!(tasks.transcript("NOPE", 0).is_none());
    assert!(tasks.follow_up("NOPE", "hi").is_err());
    assert!(tasks.end_session("NOPE").is_err());
}

// no-prop-test: until step 4 only a question can be answered.
#[test]
fn a_message_to_a_session_that_is_not_asking_is_refused() {
    let (_phone, tasks) = finished_edit();
    let key = tasks.sessions(0)[0].key.clone();
    let before = tasks.transcript(&key, 0).unwrap().items.len();
    let refused = tasks.follow_up(&key, "and the rest?").unwrap_err();
    assert!(refused.to_string().contains("isn't waiting"), "{refused}");
    assert_eq!(tasks.transcript(&key, 0).unwrap().items.len(), before);
}

// no-prop-test: ending a finished session leaves what it proposed to review.
#[test]
fn ending_a_session_keeps_its_changes_for_review() {
    let (_phone, tasks) = finished_edit();
    let session = tasks.sessions(0).remove(0);
    tasks.end_session(&session.key).unwrap();
    assert!(tasks.review(session.id).is_ok());
}
