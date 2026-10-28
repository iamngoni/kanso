//! Undo after a change to several notes (#48 step 7): the notice's ticket
//! puts back what each note had, as far as it still holds.

mod common;

use common::*;
use kanso_core::model::LOCK_FIELD;
use kanso_mobile::{KansoError, Library, NoteChange};
use proptest::prelude::*;
use serde_json::json;

/// A note the properties seed.
#[derive(Clone, Debug)]
struct Seed {
    pinned: bool,
    book: usize,
}

fn seeds() -> impl Strategy<Value = Vec<Seed>> {
    proptest::collection::vec(
        (any::<bool>(), 0usize..2).prop_map(|(pinned, book)| Seed { pinned, book }),
        1..8,
    )
}

fn seeded(seeds: &[Seed]) -> (Phone, Vec<String>, Vec<String>) {
    let mut ids = Vec::new();
    let mut books = Vec::new();
    let phone = phone_with(
        &std::sync::Arc::new(kanso_sync::MemoryServer::new()),
        |store| {
            books = vec![book(store, "One"), book(store, "Two"), book(store, "Three")];
            for (i, seed) in seeds.iter().enumerate() {
                ids.push(raw_note(
                    store,
                    json!({
                        "title": format!("Note {i:02}"),
                        "body": format!("body {i}"),
                        "bookId": books[seed.book],
                        "pinned": seed.pinned,
                    }),
                ));
            }
        },
    );
    (phone, ids, books)
}

/// What a change can touch, of every note.
fn fields(library: &Library, ids: &[String]) -> Vec<(bool, String)> {
    ids.iter()
        .map(|id| {
            let note = library.note(id).unwrap();
            (note.pinned, note.notebook_id)
        })
        .collect()
}

fn change_for(kind: usize, books: &[String]) -> NoteChange {
    match kind {
        0 => NoteChange::TogglePin,
        1 => NoteChange::Trash,
        _ => NoteChange::Move {
            notebook_id: books[2].clone(),
        },
    }
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(24))]

    /// Undone right after, a change leaves every note's fields exactly as
    /// they were, and says how many notes it wrote back.
    #[test]
    fn prop_undo_right_after_restores_every_note(seeds in seeds(), kind in 0usize..3) {
        let (phone, ids, books) = seeded(&seeds);
        let library = &phone.library;
        let before = fields(library, &ids);
        let changed = library.change(ids.clone(), change_for(kind, &books)).unwrap();
        match changed.undo {
            None => {
                prop_assert_eq!(changed.count, 0);
                prop_assert!(changed.message.is_empty());
                prop_assert_eq!(fields(library, &ids), before);
            }
            Some(ticket) => {
                prop_assert!(changed.count > 0 && !changed.message.is_empty());
                prop_assert_eq!(library.undo(ticket).unwrap(), changed.count);
                prop_assert_eq!(fields(library, &ids), before);
            }
        }
    }

    /// A ticket is used once: undoing again writes nothing.
    #[test]
    fn prop_a_used_ticket_does_nothing(seeds in seeds(), kind in 0usize..3) {
        let (phone, ids, books) = seeded(&seeds);
        let library = &phone.library;
        let changed = library.change(ids.clone(), change_for(kind, &books)).unwrap();
        let ticket = changed.undo.unwrap_or(0);
        library.undo(ticket).unwrap();
        let after = fields(library, &ids);
        prop_assert_eq!(library.undo(ticket).unwrap(), 0);
        prop_assert_eq!(fields(library, &ids), after);
    }

    /// A field changed again after the change keeps its newer value; the
    /// notes it didn't touch go back.
    #[test]
    fn prop_a_field_changed_again_keeps_its_newer_value(seeds in seeds()) {
        let (phone, ids, books) = seeded(&seeds);
        let library = &phone.library;
        let before = fields(library, &ids);
        let changed = library
            .change(ids.clone(), NoteChange::Move { notebook_id: books[2].clone() })
            .unwrap();
        // The first note moves again, to another notebook.
        library
            .change(vec![ids[0].clone()], NoteChange::Move { notebook_id: books[1].clone() })
            .unwrap();
        library.undo(changed.undo.unwrap()).unwrap();
        let after = fields(library, &ids);
        prop_assert_eq!(after[0].1.clone(), books[1].clone());
        prop_assert_eq!(&after[1..], &before[1..]);
    }
}

// no-prop-test: the sentences for three fixed changes and an empty one; the
// wording itself is `kanso_engine::batch::summary`, tested there.
#[test]
fn the_notice_says_what_was_done() {
    let mut ids = Vec::new();
    let mut work = String::new();
    let phone = phone_with(
        &std::sync::Arc::new(kanso_sync::MemoryServer::new()),
        |store| {
            work = book(store, "Work");
            for title in ["A", "B"] {
                ids.push(raw_note(store, json!({"title": title, "bookId": work})));
            }
        },
    );
    let library = &phone.library;
    let pinned = library.change(ids.clone(), NoteChange::TogglePin).unwrap();
    assert_eq!(pinned.message, "Pinned 2 notes.");
    let unpinned = library.change(ids.clone(), NoteChange::TogglePin).unwrap();
    assert_eq!(unpinned.message, "Unpinned 2 notes.");
    let trashed = library
        .change(vec![ids[0].clone()], NoteChange::Trash)
        .unwrap();
    assert_eq!(trashed.message, "Moved 1 note to the trash.");
    let moved = library
        .change(
            vec![ids[0].clone()],
            NoteChange::Move {
                notebook_id: work.clone(),
            },
        )
        .unwrap();
    assert_eq!(moved.message, "Moved 1 note to Work.");
    let nothing = library
        .change(vec![ids[0].clone()], NoteChange::Move { notebook_id: work })
        .unwrap();
    assert_eq!((nothing.count, nothing.undo), (0, None));
    assert_eq!(nothing.message, "");
}

// no-prop-test: the locked-notebook rules are fixed cases; the properties
// above cover the contents.
#[test]
fn locked_notes_keep_their_rules_and_their_text() {
    let (mut sealed_book, mut plain, mut sealed, mut open) =
        (String::new(), String::new(), String::new(), String::new());
    let phone = phone_with(
        &std::sync::Arc::new(kanso_sync::MemoryServer::new()),
        |store| {
            plain = book(store, "Plain");
            sealed_book = store
                .put(
                    kanso_core::DocType::Book,
                    json!({"name": "Sealed", LOCK_FIELD: {}}),
                    kanso_store::Put::Upsert,
                    kanso_store::Origin::Ui,
                )
                .unwrap()
                .id;
            sealed = raw_note(store, json!({"title": "S", "bookId": sealed_book}));
            open = raw_note(store, json!({"title": "O", "bookId": plain}));
        },
    );
    let library = &phone.library;
    // Still refused: into a locked notebook, and out of one.
    let into = NoteChange::Move {
        notebook_id: sealed_book.clone(),
    };
    assert_eq!(
        library.change(vec![open.clone()], into).unwrap_err(),
        KansoError::Locked
    );
    let out = NoteChange::Move {
        notebook_id: plain.clone(),
    };
    assert_eq!(
        library.change(vec![sealed.clone()], out).unwrap_err(),
        KansoError::Locked
    );
    // Allowed: pin and trash a locked note, and take it back.
    let ids = vec![sealed.clone(), open.clone()];
    let before = fields(library, &ids);
    let trashed = library.change(ids.clone(), NoteChange::Trash).unwrap();
    assert_eq!(trashed.count, 2);
    assert_eq!(library.undo(trashed.undo.unwrap()).unwrap(), 2);
    assert_eq!(fields(library, &ids), before);
    // The notice and the undo never carry the closed note's text.
    assert!(!trashed.message.contains("body"));
    assert_eq!(library.note(&sealed).unwrap().body, None);
}

// no-prop-test: a ticket nobody gave out.
#[test]
fn a_ticket_nobody_gave_out_does_nothing() {
    let phone = lone_phone();
    assert_eq!(phone.library.undo(12_345).unwrap(), 0);
}
