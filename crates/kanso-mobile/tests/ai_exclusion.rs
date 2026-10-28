//! The guarantee the AI features are built to keep: a note that is locked
//! (alone, in a locked notebook, or in one inside it), in the trash or tagged
//! `#ai-exclude` is never sent to a model, and never listed or cited, whatever
//! the model asks for, and even while the lock session is open.
//!
//! A random library of such notes and ordinary ones, each with its own title
//! and body words, is asked about through Ask (the model calls `search_notes`
//! and `read_note`), the inline assistant and the summary. Every byte the
//! model was sent is searched for the kept-out notes' titles, bodies and IDs,
//! and what comes back for the lists and links is checked too. The ordinary
//! notes must be sent, so the check isn't empty.

mod common;

use common::model::*;
use common::*;
use kanso_core::DocType;
use kanso_core::model::TRASH_BOOK_ID;
use kanso_mobile::{AiRun, AskEvent, AssistEvent};
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

proptest! {
    #![proptest_config(ProptestConfig {
        cases: 24,
        failure_persistence: None,
        ..ProptestConfig::default()
    })]

    /// Ask, with tool calls, sends and lists none of the kept-out notes.
    #[test]
    fn prop_ask_never_sends_or_lists_a_kept_out_note(shape in libraries()) {
        let (phone, seeded, anchor) = library_for(&shape);
        let mut hidden: Vec<Hidden> = seeded.hidden;
        hidden.push(anchor);
        let first_plain = seeded.plain[0].id.clone();

        for reads_hidden in [false, true] {
            // The model searches, then reads a plain note, or (the attack)
            // every kept-out note by ID.
            let mut calls = vec![("search_notes", json!({"query": "quokka"}).to_string())];
            if reads_hidden {
                for note in &hidden {
                    calls.push(("read_note", json!({"id": note.id}).to_string()));
                }
            } else {
                calls.push(("read_note", json!({"id": first_plain}).to_string()));
            }
            let cites: String = hidden
                .iter()
                .chain(seeded.plain.iter())
                .map(|note| format!("[[{}]] ", note.title))
                .collect();
            let model = Scripted::with(&format!("They are {cites}"), calls, Ending::Complete);
            let ai = ai_on(&phone.library, &model);
            let (sink, rx) = listen();
            ai.conversation().ask("Tell me about the quokka".into(), sink).unwrap();
            let heard = until(&rx, |e| {
                matches!(
                    e,
                    AskEvent::Done { .. } | AskEvent::Failed { .. } | AskEvent::NothingMatches { .. }
                        | AskEvent::CutOff { .. } | AskEvent::Stopped
                )
            });
            let sent = model.everything_sent();
            // The model's own echo of the IDs it asked for is not a leak.
            assert_clean(&sent, &hidden, !reads_hidden)?;
            prop_assert!(sent.contains("Plainbody0"), "the plain notes must be sent");

            let Some(AskEvent::Done { markdown, notes }) = heard.last() else {
                return Err(TestCaseError::fail(format!("no answer: {heard:?}")));
            };
            for event in &heard {
                let listed = match event {
                    AskEvent::Used { notes } | AskEvent::Done { notes, .. } => notes.as_slice(),
                    _ => &[],
                };
                for listed in listed {
                    prop_assert!(!hidden.iter().any(|h| h.id == listed.id), "{listed:?} listed");
                }
            }
            for note in &hidden {
                prop_assert!(
                    !markdown.contains(&format!("({}", kanso_mobile::CITATION_PREFIX))
                        || !markdown.contains(&note.id),
                    "a link to {} was made", note.id
                );
                prop_assert!(!notes.iter().any(|n| n.id == note.id));
            }
        }
    }

    /// The assistant and the summary refuse every kept-out note by ID and
    /// send nothing; for an ordinary note they send its text.
    #[test]
    fn prop_assist_and_summary_refuse_kept_out_notes(shape in libraries()) {
        let (phone, seeded, anchor) = library_for(&shape);
        let mut hidden: Vec<Hidden> = seeded.hidden;
        hidden.push(anchor);
        let model = Scripted::new("Done.");
        let ai = ai_on(&phone.library, &model);
        for note in &hidden {
            prop_assert!(!ai.note_allowed(&note.id).unwrap());
            let (sink, _rx) = listen();
            prop_assert!(ai
                .assist(&note.id, "improve", note.body.clone(), note.body.clone(), &AiRun::new(), sink)
                .is_err());
            let (sink, _rx) = listen();
            prop_assert!(ai.summarize(&note.id, &AiRun::new(), sink).is_err());
        }
        prop_assert_eq!(model.request_count(), 0);

        let plain = &seeded.plain[0];
        prop_assert!(ai.note_allowed(&plain.id).unwrap());
        let (sink, rx) = listen();
        ai.assist(&plain.id, "fix", String::new(), "Some text.".into(), &AiRun::new(), sink).unwrap();
        let end = until(&rx, |e| matches!(e, AssistEvent::Done { .. } | AssistEvent::Failed { .. }));
        prop_assert!(matches!(end.last(), Some(AssistEvent::Done { .. })), "{end:?}");
        let (sink, rx) = listen();
        ai.summarize(&plain.id, &AiRun::new(), sink).unwrap();
        until(&rx, |e| matches!(e, AssistEvent::Done { .. } | AssistEvent::Failed { .. }));
        let sent = model.everything_sent();
        prop_assert!(sent.contains(&plain.body), "the plain note was not sent");
        assert_clean(&sent, &hidden, true)?;
    }
}
