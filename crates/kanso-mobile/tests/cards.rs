//! Writing from notes and learning from notes on a phone (issue #198):
//! the review loop, the log that survives a reopen, suggested cards, Quiz
//! Me and drafting with a scripted model. A note that is locked, in a
//! locked notebook, in the trash or tagged `#ai-exclude` is never sent to
//! the model and never becomes a card or a source.

mod common;

use common::model::*;
use common::*;
use kanso_core::DocType;
use kanso_core::model::TRASH_BOOK_ID;
use kanso_mobile::{
    AiRun, CardPair, CardScopeArg, CardScopeKind, DraftEvent, DraftLength, DraftShape, GradeEvent,
    QuizEvent, SuggestEvent,
};
use kanso_store::{Origin, Put, Store};
use serde_json::json;

fn put(store: &Store, doc_type: DocType, body: serde_json::Value, origin: Origin) -> String {
    store.put(doc_type, body, Put::Upsert, origin).unwrap().id
}

fn all() -> CardScopeArg {
    CardScopeArg {
        kind: CardScopeKind::All,
        id: String::new(),
    }
}

struct Library {
    book: String,
    cards: String,
    locked: String,
    vaulted: String,
    trashed: String,
    excluded: String,
}

/// A deck note, and one note of each kind that must stay out.
fn seed(store: &Store) -> Library {
    let book = put(store, DocType::Book, json!({"name": "Study"}), Origin::Ui);
    let vault = put(
        store,
        DocType::Book,
        json!({"name": "Vault", "kansoLock": {"salt": "abc"}}),
        Origin::Ui,
    );
    let exclude = put(
        store,
        DocType::Tag,
        json!({"name": "ai-exclude"}),
        Origin::Ui,
    );
    let cards = put(
        store,
        DocType::Note,
        json!({"title": "Deck", "bookId": book,
            "body": "# Week 3\n\nFSRS stands for Free Spaced Repetition Scheduler.\n\nFSRS? :: a scheduler\nZimbabwe capital :: Harare\n"}),
        Origin::Ui,
    );
    let secret = |kind: &str, fields: serde_json::Value| {
        let mut all = json!({
            "title": format!("Zqx{kind}"),
            "body": format!("Zqx{kind}Body :: Zqx{kind}Answer"),
        });
        all.as_object_mut()
            .unwrap()
            .extend(fields.as_object().unwrap().clone());
        put(store, DocType::Note, all, Origin::Sync)
    };
    Library {
        locked: secret(
            "Sealed",
            json!({"bookId": book, "kansoLock": {"salt": "abc"}}),
        ),
        vaulted: secret("Vault", json!({"bookId": vault})),
        trashed: secret("Trash", json!({"bookId": TRASH_BOOK_ID})),
        excluded: secret("Excl", json!({"bookId": book, "tags": [exclude]})),
        book,
        cards,
    }
}

fn phone_with_deck(model: &std::sync::Arc<Scripted>) -> (Phone, kanso_mobile::AiHandle, Library) {
    let (phone, ai) = ai_phone(model);
    let store = Store::open(phone.path()).unwrap();
    let lib = seed(&store);
    (phone, ai, lib)
}

#[test]
fn reviewing_schedules_cards_and_the_log_survives_a_reopen() {
    let model = Scripted::new("unused");
    let (phone, ai, lib) = phone_with_deck(&model);
    let cards = phone.library.cards_with_ai(ai);
    let due = cards.due(all(), 50, 0).unwrap();
    // The two written cards, and the explicit `::` cards of the kept-out
    // notes that may still be reviewed: only the excluded one (it never
    // leaves the device).
    let questions: Vec<&str> = due.iter().map(|c| c.question.as_str()).collect();
    assert!(questions.contains(&"FSRS?") && questions.contains(&"Zimbabwe capital"));
    assert!(
        !questions
            .iter()
            .any(|q| q.contains("ZqxSealed") || q.contains("ZqxVault") || q.contains("ZqxTrash")),
        "{questions:?}"
    );
    assert!(due.iter().all(|c| c.intervals.len() == 4 && c.is_new));
    let first = due.iter().find(|c| c.question == "FSRS?").unwrap();
    let counts = cards.counts(all(), 0).unwrap();
    let waiting = counts.waiting;
    let done = cards
        .answer(first.note_id.clone(), first.card_key.clone(), 4, 0)
        .unwrap();
    assert_eq!(done.waiting, waiting - 1);
    assert!(done.due_ms > kanso_core::now_ms());
    assert!(cards.answer("note:x".into(), "bad".into(), 9, 0).is_err());

    // The notes' cards, with the duplicate marked.
    let marks = cards.note_cards(lib.cards.clone()).unwrap();
    assert_eq!(marks.len(), 2);
    assert!(
        marks
            .iter()
            .find(|c| c.question == "FSRS?")
            .unwrap()
            .due_ms
            .is_some()
    );

    let stats = cards.stats(all(), 0).unwrap();
    assert_eq!(stats.per_day.iter().sum::<u32>(), 1);
    assert!((stats.retention.unwrap() - 1.0).abs() < 1e-9);

    // A reopened library still knows the answer.
    let reopened = kanso_mobile::Library::open_for_test(
        &phone.path(),
        phone.secrets.clone(),
        std::sync::Arc::new(kanso_sync::MemoryServer::new()),
    )
    .unwrap();
    let again = reopened.cards();
    assert_eq!(again.counts(all(), 0).unwrap().waiting, waiting - 1);

    // Suspending hides, and settings clamp.
    let other = due.iter().find(|c| c.question != "FSRS?").unwrap();
    again
        .suspend(other.note_id.clone(), other.card_key.clone(), true)
        .unwrap();
    assert_eq!(again.counts(all(), 0).unwrap().suspended, 1);
    let mut settings = again.settings().unwrap();
    settings.retention = 2.0;
    settings.reminder = Some("25:00".into());
    let saved = again.set_settings(settings).unwrap();
    assert!(saved.retention <= 0.97 && saved.reminder.is_none());
    // The excluded note's card still reviews (it never leaves the device);
    // the reminder says a count and no words of any card.
    let text = again.reminder_text(0).unwrap().unwrap();
    assert_eq!(text, "1 flashcard is due.");
    assert_eq!(again.reset_history().unwrap(), 1);
}

#[test]
fn suggested_cards_send_only_the_one_note_and_adding_is_one_edit() {
    let reply = r#"{"q":"What does FSRS stand for?","a":"Free Spaced Repetition Scheduler","source":"FSRS stands for Free Spaced"}
{"q":"Made up?","a":"yes","source":"not in the note"}"#;
    let model = Scripted::new(reply);
    let (phone, ai, lib) = phone_with_deck(&model);
    let cards = phone.library.cards_with_ai(ai);
    assert_eq!(cards.make_blocked(lib.cards.clone()).unwrap(), None);
    assert!(cards.make_blocked(lib.excluded.clone()).unwrap().is_some());
    assert!(cards.make_blocked(lib.locked.clone()).unwrap().is_some());

    let (sink, rx) = listen();
    cards
        .suggest(lib.cards.clone(), &AiRun::new(), sink)
        .unwrap();
    let heard = until(&rx, |e| !matches!(e, SuggestEvent::Status { .. }));
    let SuggestEvent::Done { cards: found } = heard.last().unwrap() else {
        panic!("{heard:?}")
    };
    assert_eq!(found.len(), 2);
    assert!(!found[0].source_missing && found[0].line == Some(2));
    assert!(found[1].source_missing);
    let sent = model.everything_sent();
    assert!(sent.contains("FSRS stands for"));
    for kept in ["ZqxSealed", "ZqxVault", "ZqxTrash", "ZqxExcl"] {
        assert!(!sent.contains(kept), "{kept} was sent");
    }

    let added = cards
        .add_cards(
            lib.cards.clone(),
            vec![CardPair {
                question: found[0].question.clone(),
                answer: found[0].answer.clone(),
            }],
        )
        .unwrap();
    assert_eq!(added, 1);
    let marks = cards.note_cards(lib.cards.clone()).unwrap();
    assert_eq!(marks.len(), 3);
    let body = Store::open(phone.path())
        .unwrap()
        .note(&lib.cards)
        .unwrap()
        .unwrap()
        .body;
    assert!(body.contains("## Flashcards") && body.contains("What does FSRS stand for? :: Free"));

    // A locked note is refused before anything is sent.
    let before = model.requests.load(std::sync::atomic::Ordering::SeqCst);
    let (sink, rx) = listen();
    assert!(
        cards
            .suggest(lib.locked.clone(), &AiRun::new(), sink)
            .is_ok()
    );
    let heard = until(&rx, |e| !matches!(e, SuggestEvent::Status { .. }));
    assert!(matches!(heard.last().unwrap(), SuggestEvent::Failed { .. }));
    assert_eq!(
        model.requests.load(std::sync::atomic::Ordering::SeqCst),
        before
    );
}

#[test]
fn quiz_me_asks_from_allowed_notes_and_grades_against_one() {
    let model = Scripted::new(r#"{"q":"What does FSRS schedule?","n":1,"source":"Free Spaced"}"#);
    let (phone, ai, lib) = phone_with_deck(&model);
    let cards = phone.library.cards_with_ai(ai);
    // Nothing can be asked of kept-out notes.
    let (sink, _rx) = listen();
    assert!(
        cards
            .quiz(
                vec![lib.locked.clone(), lib.excluded.clone()],
                &AiRun::new(),
                sink
            )
            .is_err()
    );
    let (sink, rx) = listen();
    cards
        .quiz(
            vec![lib.cards.clone(), lib.excluded.clone()],
            &AiRun::new(),
            sink,
        )
        .unwrap();
    let heard = until(&rx, |e| !matches!(e, QuizEvent::Status { .. }));
    let QuizEvent::Questions { items } = heard.last().unwrap() else {
        panic!("{heard:?}")
    };
    assert_eq!(items.len(), 1);
    assert_eq!(items[0].note_id, lib.cards);
    assert!(!model.everything_sent().contains("ZqxExcl"));

    let grader = Scripted::new("Partly: the note says \"Free Spaced\".");
    let (phone, ai, lib) = phone_with_deck(&grader);
    let cards = phone.library.cards_with_ai(ai);
    let (sink, rx) = listen();
    cards
        .grade(
            "What does FSRS schedule?".into(),
            lib.cards.clone(),
            "reviews".into(),
            &AiRun::new(),
            sink,
        )
        .unwrap();
    let heard = until(&rx, |_| true);
    assert!(matches!(&heard[0], GradeEvent::Graded { label, .. } if label == "Partly"));
    // Quiz answers are not stored: the schedule is untouched.
    assert_eq!(cards.counts(all(), 0).unwrap().due, 0);
}

#[test]
fn a_draft_loses_a_citation_to_a_note_it_was_not_given_and_saves_with_sources() {
    let model = Scripted::new("# Update\n\nThe deck covers FSRS [1]. A made-up claim [99].");
    let (phone, ai, lib) = phone_with_deck(&model);
    let drafting = phone.library.drafting_with_ai(ai);
    let ids = vec![
        lib.cards.clone(),
        lib.locked.clone(),
        lib.vaulted.clone(),
        lib.trashed.clone(),
        lib.excluded.clone(),
    ];
    let listed = drafting.sources(ids.clone()).unwrap();
    assert_eq!(listed.notes.len(), 1);
    assert_eq!(listed.left_out_excluded, 1);
    assert!(listed.left_out_locked >= 2);
    assert!(
        !listed.summary.contains("Zqx"),
        "left-out notes are counted, never named"
    );

    let (sink, rx) = listen();
    drafting
        .write(
            ids,
            DraftShape::Update,
            "a status update".into(),
            DraftLength::Short,
            &AiRun::new(),
            sink,
        )
        .unwrap();
    let heard = until(&rx, |e| {
        matches!(
            e,
            DraftEvent::Done { .. }
                | DraftEvent::Failed { .. }
                | DraftEvent::CutOff { .. }
                | DraftEvent::Stopped
        )
    });
    let DraftEvent::Done { draft, sources } = heard.last().unwrap() else {
        panic!("{heard:?}")
    };
    assert_eq!(draft.removed, 1);
    assert!(!draft.text.contains("[99]") && draft.text.contains("[1]"));
    assert!(draft.spans.iter().any(|s| s.unsourced));
    assert!(draft.spans.iter().any(|s| !s.cites.is_empty()));
    assert_eq!(sources.len(), 1);
    let sent = model.everything_sent();
    assert!(sent.contains("FSRS stands for"));
    for kept in ["ZqxSealed", "ZqxVault", "ZqxTrash", "ZqxExcl"] {
        assert!(!sent.contains(kept), "{kept} was sent");
    }

    // Nothing is a note until it is saved.
    let store = Store::open(phone.path()).unwrap();
    let count = |s: &Store| {
        s.counts()
            .unwrap()
            .get(&DocType::Note)
            .copied()
            .unwrap_or(0)
    };
    let before = count(&store);
    let preview = drafting
        .preview(draft.text.clone(), sources.clone())
        .unwrap();
    assert_eq!(preview.title, "Update");
    assert_eq!(preview.default_book_id, lib.book);
    assert!(preview.markdown.contains("## Sources") && preview.markdown.contains("1. [[Deck]]"));
    assert_eq!(count(&store), before);
    let id = drafting
        .save(preview.title, preview.markdown, preview.default_book_id, 1)
        .unwrap();
    assert_eq!(count(&store), before + 1);
    assert_eq!(store.backlinks(&lib.cards).unwrap().len(), 1);
    assert_eq!(
        store.revisions(&id).unwrap()[0].author.as_deref(),
        Some("Drafted by AI from 1 note")
    );
}

#[test]
fn drafting_refuses_when_no_source_may_be_read() {
    let model = Scripted::new("never");
    let (phone, ai, lib) = phone_with_deck(&model);
    let drafting = phone.library.drafting_with_ai(ai);
    let (sink, _rx) = listen();
    let err = drafting
        .write(
            vec![lib.locked, lib.excluded, lib.trashed],
            DraftShape::Report,
            "a report".into(),
            DraftLength::Medium,
            &AiRun::new(),
            sink,
        )
        .unwrap_err();
    assert!(err.to_string().contains("None of these notes can be sent"));
    assert_eq!(model.requests.load(std::sync::atomic::Ordering::SeqCst), 0);
}
