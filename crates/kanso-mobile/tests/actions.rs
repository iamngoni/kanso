//! The note actions of `Library`: status, tags, restore, deleting for good,
//! the batch notice and Undo, emptying the trash, duplicating and copying a
//! link. Locked notes are in `tests/actions_locked.rs`.

mod common;

use std::collections::BTreeSet;

use common::*;
use kanso_mobile::{KansoError, Library, ListPlace, ListQuery, NewNote, NoteChange, NoteView};
use proptest::prelude::*;

/// What a note holds, apart from its revision and times.
type Print = (String, Option<String>, String, Vec<String>, String, bool);

fn print(view: &NoteView) -> Print {
    (
        view.title.clone(),
        view.body.clone(),
        view.notebook_id.clone(),
        view.tags.clone(),
        view.status.clone(),
        view.pinned,
    )
}

fn prints(library: &Library, ids: &[String]) -> Vec<Print> {
    ids.iter()
        .map(|id| print(&library.note(id).unwrap()))
        .collect()
}

fn in_book(library: &Library, book: &str, title: &str, body: &str) -> String {
    library
        .create(NewNote {
            notebook_id: Some(book.to_owned()),
            title: title.to_owned(),
            body: body.to_owned(),
            ..NewNote::default()
        })
        .unwrap()
        .id
}

fn place(library: &Library, place: ListPlace) -> Vec<String> {
    library
        .list(ListQuery {
            place,
            ..ListQuery::default()
        })
        .unwrap()
        .rows
        .into_iter()
        .map(|row| row.id)
        .collect()
}

/// A library with notebooks Alpha and Beta, and `count` plain notes in Alpha.
fn world(count: usize) -> (Phone, [String; 2], Vec<String>) {
    let phone = lone_phone();
    let alpha = phone
        .library
        .create_notebook("Alpha".into(), None)
        .unwrap()
        .id;
    let beta = phone
        .library
        .create_notebook("Beta".into(), None)
        .unwrap()
        .id;
    let ids = (0..count)
        .map(|i| in_book(&phone.library, &alpha, &format!("Note {i}"), "text"))
        .collect();
    (phone, [alpha, beta], ids)
}

fn set_status(status: &str) -> NoteChange {
    NoteChange::SetStatus {
        status: status.to_owned(),
    }
}

// no-prop-test: fixed calls and the exact notices they answer with; the
// idempotence is a property below.
#[test]
fn setting_a_status_writes_the_notes_and_words_the_notice() {
    let (phone, _, ids) = world(3);
    let library = &phone.library;
    let done = library
        .change(ids[..2].to_vec(), set_status("onHold"))
        .unwrap();
    assert_eq!(done.count, 2);
    assert_eq!(done.message, "Set 2 notes to On hold.");
    assert!(done.undo.is_some());
    assert_eq!(library.note(&ids[0]).unwrap().status, "onHold");
    assert_eq!(library.note(&ids[2]).unwrap().status, "none");
    let one = library
        .change(vec![ids[2].clone()], set_status("active"))
        .unwrap();
    assert_eq!(one.message, "Set 1 note to Active.");
    // Already so: nothing is written, said or undoable.
    let again = library
        .change(ids[..2].to_vec(), set_status("onHold"))
        .unwrap();
    assert_eq!(
        (again.count, again.message.as_str(), again.undo),
        (0, "", None)
    );
    assert!(matches!(
        library.change(ids.clone(), set_status("someday")),
        Err(KansoError::Invalid { .. })
    ));
    assert_eq!(
        library
            .change(vec!["note:nope".into()], set_status("active"))
            .map(|done| done.count),
        Err(KansoError::NotFound)
    );
    // Clearing a status is setting "none".
    assert_eq!(
        library
            .change(ids[..2].to_vec(), set_status("none"))
            .unwrap()
            .count,
        2
    );
}

// no-prop-test: fixed calls; add-then-remove is a property below.
#[test]
fn adding_and_removing_a_tag() {
    let (phone, _, ids) = world(2);
    let library = &phone.library;
    let tag = library.create_tag("work".into()).unwrap();
    let added = library
        .change(
            ids.clone(),
            NoteChange::AddTag {
                tag_id: tag.id.clone(),
            },
        )
        .unwrap();
    assert_eq!(
        (added.count, added.message.as_str()),
        (2, "Added #work to 2 notes.")
    );
    assert_eq!(library.note(&ids[0]).unwrap().tags, vec![tag.id.clone()]);
    assert_eq!(library.tags().unwrap()[0].count, 2);
    let removed = library
        .change(
            vec![ids[0].clone()],
            NoteChange::RemoveTag {
                tag_id: tag.id.clone(),
            },
        )
        .unwrap();
    assert_eq!(removed.message, "Took #work off 1 note.");
    assert!(library.note(&ids[0]).unwrap().tags.is_empty());
    // A note without the tag is not written by a removal.
    assert_eq!(
        library
            .change(
                vec![ids[0].clone()],
                NoteChange::RemoveTag {
                    tag_id: tag.id.clone()
                }
            )
            .unwrap()
            .count,
        0
    );
    for change in [
        NoteChange::AddTag {
            tag_id: "tag:missing".into(),
        },
        NoteChange::RemoveTag {
            tag_id: "tag:missing".into(),
        },
        NoteChange::AddTag {
            tag_id: ids[0].clone(),
        },
    ] {
        assert_eq!(
            library.change(ids.clone(), change).map(|done| done.count),
            Err(KansoError::NotFound)
        );
    }
}

// no-prop-test: the notices of every kind, as the desktop words them.
#[test]
fn the_notices_are_the_desktops_words() {
    let (phone, [_, beta], ids) = world(3);
    let library = &phone.library;
    let say = |ids: &[String], change| library.change(ids.to_vec(), change).unwrap().message;
    assert_eq!(say(&ids[..1], NoteChange::TogglePin), "Pinned 1 note.");
    assert_eq!(say(&ids[..1], NoteChange::TogglePin), "Unpinned 1 note.");
    assert_eq!(
        say(&ids, NoteChange::Move { notebook_id: beta }),
        "Moved 3 notes to Beta."
    );
    assert_eq!(
        say(&ids[..2], NoteChange::Trash),
        "Moved 2 notes to the trash."
    );
    assert_eq!(
        say(&ids[..2], NoteChange::Restore),
        "Moved 2 notes to Alpha."
    );
    assert_eq!(
        say(&ids[..1], NoteChange::Trash),
        "Moved 1 note to the trash."
    );
    assert_eq!(
        say(&ids[..1], NoteChange::DeleteForever),
        "Deleted 1 note permanently."
    );
}

// no-prop-test: the token's lifetime, step by step; the round trip is a
// property below.
#[test]
fn each_batch_undoes_its_own_write() {
    let (phone, [_, beta], ids) = world(2);
    let library = &phone.library;
    let before = prints(library, &ids);
    let first = library
        .change(ids.clone(), NoteChange::Move { notebook_id: beta })
        .unwrap();
    let second = library.change(ids.clone(), NoteChange::TogglePin).unwrap();
    // The pin comes off again; the move is still there.
    assert_eq!(library.undo(second.undo.unwrap()), Ok(2));
    assert!(!library.note(&ids[0]).unwrap().pinned);
    // A token works once.
    assert_eq!(library.undo(second.undo.unwrap()), Ok(0));
    assert_eq!(library.undo(9999), Ok(0));
    assert_ne!(prints(library, &ids), before, "the move was not undone");
    // The earlier change is still the earlier change's to take back.
    assert_eq!(library.undo(first.undo.unwrap()), Ok(2));
    assert_eq!(prints(library, &ids), before);
    // A change that wrote nothing has no token, and leaves the latest alone.
    let third = library.change(ids.clone(), NoteChange::Trash).unwrap();
    let nothing = library.change(ids.clone(), NoteChange::Trash).unwrap();
    assert_eq!(nothing.undo, None);
    assert_eq!(library.undo(third.undo.unwrap()), Ok(2));
}

// no-prop-test: two late-edit cases the engine's property covers in general.
#[test]
fn undo_skips_deleted_notes_and_keeps_later_changes() {
    let (phone, [alpha, beta], ids) = world(3);
    let library = &phone.library;
    let moved = library
        .change(
            ids.clone(),
            NoteChange::Move {
                notebook_id: beta.clone(),
            },
        )
        .unwrap();
    // One note went to the trash and was deleted; one moved again.
    library
        .change(vec![ids[0].clone()], NoteChange::Trash)
        .unwrap();
    library
        .change(vec![ids[0].clone()], NoteChange::DeleteForever)
        .unwrap();
    library
        .change(vec![ids[1].clone()], NoteChange::Trash)
        .unwrap();
    assert_eq!(library.undo(moved.undo.unwrap()), Ok(1));
    assert_eq!(library.note(&ids[2]).unwrap().notebook_id, alpha);
    assert_eq!(library.note(&ids[1]).unwrap().notebook_id, "trash");
    assert_eq!(library.note(&ids[0]), Err(KansoError::NotFound));
}

// no-prop-test: the notebook Restore picks, and its error paths.
#[test]
fn restore_goes_to_the_first_notebook_by_name() {
    let phone = lone_phone();
    let library = &phone.library;
    // Made first, but sorts last: the desktop's first is the first by name.
    let zed = library.create_notebook("Zed".into(), None).unwrap().id;
    let alpha = library.create_notebook("alpha".into(), None).unwrap().id;
    let a = in_book(library, &zed, "A", "a");
    let b = in_book(library, &zed, "B", "b");
    library.change(vec![a.clone()], NoteChange::Trash).unwrap();
    let restored = library
        .change(vec![a.clone(), b.clone()], NoteChange::Restore)
        .unwrap();
    // B was not in the trash: left alone.
    assert_eq!(
        (restored.count, restored.message.as_str()),
        (1, "Moved 1 note to alpha.")
    );
    assert_eq!(library.note(&a).unwrap().notebook_id, alpha);
    assert_eq!(library.note(&b).unwrap().notebook_id, zed);
    assert_eq!(library.undo(restored.undo.unwrap()), Ok(1));
    assert_eq!(library.note(&a).unwrap().notebook_id, "trash");
    // Nothing in the trash: nothing to do.
    assert_eq!(
        library.change(vec![b], NoteChange::Restore).unwrap().count,
        0
    );
}

// no-prop-test: the no-notebook error path needs exactly this library.
#[test]
fn restore_needs_a_notebook_to_restore_into() {
    let (phone, [alpha, beta], ids) = world(1);
    let library = &phone.library;
    library.change(ids.clone(), NoteChange::Trash).unwrap();
    library.delete_notebook(&alpha).unwrap();
    library.delete_notebook(&beta).unwrap();
    assert!(library.notebooks().unwrap().is_empty());
    assert_eq!(
        library
            .change(ids.clone(), NoteChange::Restore)
            .map(|done| done.count),
        Err(KansoError::invalid(
            "Create a notebook to restore notes into."
        ))
    );
    assert_eq!(library.note(&ids[0]).unwrap().notebook_id, "trash");
}

// no-prop-test: the all-or-nothing rule and the cannot-undo rule.
#[test]
fn deleting_for_good_takes_only_notes_in_the_trash() {
    let (phone, _, ids) = world(3);
    let library = &phone.library;
    library
        .change(ids[..2].to_vec(), NoteChange::Trash)
        .unwrap();
    assert!(matches!(
        library.change(ids.clone(), NoteChange::DeleteForever),
        Err(KansoError::Invalid { .. })
    ));
    assert_eq!(
        place(library, ListPlace::Trash).len(),
        2,
        "nothing was deleted"
    );
    let gone = library
        .change(ids[..2].to_vec(), NoteChange::DeleteForever)
        .unwrap();
    assert_eq!((gone.count, gone.undo), (2, None));
    assert_eq!(library.note(&ids[0]), Err(KansoError::NotFound));
    assert!(library.note(&ids[2]).is_ok());
    assert_eq!(
        library
            .change(vec!["note:nope".into()], NoteChange::DeleteForever)
            .map(|done| done.count),
        Err(KansoError::NotFound)
    );
    assert_eq!(
        library
            .change(vec![], NoteChange::DeleteForever)
            .unwrap()
            .count,
        0
    );
}

// no-prop-test: a fixed library; the invariant over any library is below.
#[test]
fn emptying_the_trash_leaves_the_rest() {
    let (phone, _, ids) = world(4);
    let library = &phone.library;
    library
        .change(ids[..3].to_vec(), NoteChange::Trash)
        .unwrap();
    assert_eq!(library.overview().unwrap().trash, 3);
    assert_eq!(library.empty_trash(), Ok(3));
    let overview = library.overview().unwrap();
    assert_eq!((overview.trash, overview.all), (0, 1));
    assert_eq!(library.empty_trash(), Ok(0));
    assert!(library.note(&ids[3]).is_ok());
}

// no-prop-test: a fixed note; preservation over any note is below.
#[test]
fn a_duplicate_is_a_copy_with_a_new_title() {
    let (phone, [_, beta], ids) = world(1);
    let library = &phone.library;
    let tag = library.create_tag("work".into()).unwrap();
    library
        .change(
            ids.clone(),
            NoteChange::AddTag {
                tag_id: tag.id.clone(),
            },
        )
        .unwrap();
    library.change(ids.clone(), set_status("active")).unwrap();
    library
        .change(
            ids.clone(),
            NoteChange::Move {
                notebook_id: beta.clone(),
            },
        )
        .unwrap();
    library.change(ids.clone(), NoteChange::TogglePin).unwrap();
    let copy = library.duplicate(&ids[0]).unwrap();
    assert_ne!(copy.id, ids[0]);
    let (original, copied) = (
        library.note(&ids[0]).unwrap(),
        library.note(&copy.id).unwrap(),
    );
    assert_eq!(copied.title, "Note 0 (copy)");
    assert_eq!(copied.body, original.body);
    assert_eq!(copied.notebook_id, beta);
    assert_eq!(copied.tags, vec![tag.id]);
    assert_eq!(copied.status, "active");
    // The copy is a fresh note, not pinned like the desktop's.
    assert!(!copied.pinned);
    assert_eq!(library.duplicate("note:nope"), Err(KansoError::NotFound));
    // A copy of a copy.
    let again = library.duplicate(&copy.id).unwrap();
    assert_eq!(
        library.note(&again.id).unwrap().title,
        "Note 0 (copy) (copy)"
    );
}

// no-prop-test: the strings the engine's link rules give, for each case.
#[test]
fn a_note_link_is_the_desktops_choice_and_message() {
    let (phone, _, _) = world(0);
    let library = &phone.library;
    let alpha = library.notebooks().unwrap()[0].id.clone();
    let plain = in_book(library, &alpha, "Garden plan", "x");
    let wiki = library.note_link(&plain, false).unwrap();
    assert_eq!(
        (wiki.text.as_str(), wiki.message.as_str()),
        ("[[Garden plan]]", "Link copied.")
    );
    let by_id = library.note_link(&plain, true).unwrap();
    assert!(by_id.text.starts_with("[Garden plan]("), "{}", by_id.text);
    assert!(by_id.text.contains(plain.trim_start_matches("note:")));
    assert_eq!(by_id.message, "Link copied.");
    // A title that can't be a wiki-link.
    let odd = in_book(library, &alpha, "Item #3", "x");
    let link = library.note_link(&odd, false).unwrap();
    assert!(link.text.contains(odd.trim_start_matches("note:")));
    assert_eq!(
        link.message,
        "Copied a Kanso link: the title can't be a [[wiki-link]]."
    );
    // An older note has the same title.
    let twin = in_book(library, &alpha, "Garden plan", "y");
    let link = library.note_link(&twin, false).unwrap();
    assert!(link.text.contains(twin.trim_start_matches("note:")));
    assert_eq!(
        link.message,
        "Copied a Kanso link: an older note has the same title."
    );
    assert_eq!(
        library.note_link("note:nope", false),
        Err(KansoError::NotFound)
    );
}

// no-prop-test: the choices are constants of the model; their order matters.
#[test]
fn the_statuses_and_tag_colors_come_in_the_desktops_order() {
    let statuses = Library::statuses();
    let names: Vec<&str> = statuses.iter().map(|s| s.status.as_str()).collect();
    assert_eq!(names, ["none", "active", "onHold", "completed", "dropped"]);
    assert_eq!(statuses[2].label, "On hold");
    let colors = Library::tag_colors();
    assert_eq!(colors.len(), 14);
    assert_eq!(
        (colors[0].name.as_str(), colors[1].name.as_str()),
        ("default", "red")
    );
    assert_eq!(colors[1].rgb, 0xdb2828);
}

// no-prop-test: the two new places on fixed notes, and their error path.
#[test]
fn status_and_tag_path_places_list_what_they_name() {
    let (phone, _, ids) = world(4);
    let library = &phone.library;
    let nested = library.create_tag("work/acme".into()).unwrap();
    let top = library.create_tag("work".into()).unwrap();
    library
        .change(ids[..2].to_vec(), set_status("active"))
        .unwrap();
    library
        .change(ids[2..3].to_vec(), set_status("dropped"))
        .unwrap();
    library
        .change(ids[..1].to_vec(), NoteChange::AddTag { tag_id: nested.id })
        .unwrap();
    library
        .change(ids[1..2].to_vec(), NoteChange::AddTag { tag_id: top.id })
        .unwrap();
    let sorted = |mut v: Vec<String>| {
        v.sort();
        v
    };
    let status = |s: &str| ListPlace::Status { status: s.into() };
    assert_eq!(
        sorted(place(library, status("active"))),
        sorted(ids[..2].to_vec())
    );
    assert_eq!(place(library, status("dropped")), vec![ids[2].clone()]);
    assert_eq!(
        sorted(place(
            library,
            ListPlace::TagPath {
                path: "work".into()
            }
        )),
        sorted(ids[..2].to_vec()),
        "a level lists the tags under it"
    );
    assert_eq!(
        place(
            library,
            ListPlace::TagPath {
                path: "work/acme".into()
            }
        ),
        vec![ids[0].clone()]
    );
    assert!(
        place(
            library,
            ListPlace::TagPath {
                path: "play".into()
            }
        )
        .is_empty()
    );
    assert!(matches!(
        library.list(ListQuery {
            place: status("someday"),
            ..ListQuery::default()
        }),
        Err(KansoError::Invalid { .. })
    ));
    let overview = library.overview().unwrap();
    let counts: Vec<(&str, u32)> = overview
        .by_status
        .iter()
        .map(|s| (s.status.as_str(), s.count))
        .collect();
    assert_eq!(
        counts,
        [
            ("active", 2),
            ("onHold", 0),
            ("completed", 0),
            ("dropped", 1)
        ]
    );
}

/// A note the property tests seed.
#[derive(Clone, Debug)]
struct Seed {
    book: usize,
    tags: BTreeSet<usize>,
    status: usize,
    pinned: bool,
}

fn seeds() -> impl Strategy<Value = Vec<Seed>> {
    proptest::collection::vec(
        (
            0usize..2,
            proptest::collection::btree_set(0usize..3, 0..3),
            0usize..5,
            any::<bool>(),
        )
            .prop_map(|(book, tags, status, pinned)| Seed {
                book,
                tags,
                status,
                pinned,
            }),
        1..8,
    )
}

const STATUSES: [&str; 5] = ["none", "active", "onHold", "completed", "dropped"];

/// A library holding `seeds`: notebooks Alpha and Beta, tags t0 to t2.
fn seeded(seeds: &[Seed]) -> (Phone, [String; 2], Vec<String>, Vec<String>) {
    let (phone, books, _) = world(0);
    let library = &phone.library;
    let tags: Vec<String> = (0..3)
        .map(|i| library.create_tag(format!("t{i}")).unwrap().id)
        .collect();
    let ids: Vec<String> = seeds
        .iter()
        .enumerate()
        .map(|(i, seed)| {
            let id = in_book(library, &books[seed.book], &format!("n{i}"), "body");
            for tag in &seed.tags {
                let tag_id = tags[*tag].clone();
                library
                    .change(vec![id.clone()], NoteChange::AddTag { tag_id })
                    .unwrap();
            }
            library
                .change(vec![id.clone()], set_status(STATUSES[seed.status]))
                .unwrap();
            if seed.pinned {
                library
                    .change(vec![id.clone()], NoteChange::TogglePin)
                    .unwrap();
            }
            id
        })
        .collect();
    (phone, books, ids, tags)
}

fn chosen(ids: &[String], pick: &[bool]) -> Vec<String> {
    ids.iter()
        .zip(pick.iter().cycle())
        .filter(|(_, p)| **p)
        .map(|(id, _)| id.clone())
        .collect()
}

fn a_change(kind: usize, books: &[String; 2], tags: &[String], status: usize) -> NoteChange {
    match kind {
        0 => NoteChange::TogglePin,
        1 => NoteChange::Trash,
        2 => NoteChange::Move {
            notebook_id: books[1].clone(),
        },
        3 => set_status(STATUSES[status]),
        4 => NoteChange::AddTag {
            tag_id: tags[0].clone(),
        },
        _ => NoteChange::RemoveTag {
            tag_id: tags[1].clone(),
        },
    }
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(24))]

    /// Round trip: a batch and then Undo puts every note back exactly as it
    /// was, and Undo writes as many notes as the batch did.
    #[test]
    fn prop_undo_restores_every_note(
        seeds in seeds(),
        pick in proptest::collection::vec(any::<bool>(), 1..8),
        kind in 0usize..6,
        status in 0usize..5,
    ) {
        let (phone, books, ids, tags) = seeded(&seeds);
        let library = &phone.library;
        let chosen = chosen(&ids, &pick);
        let before = prints(library, &ids);
        let done = library
            .change(chosen, a_change(kind, &books, &tags, status))
            .unwrap();
        prop_assert_eq!(done.undo.is_some(), done.count > 0);
        prop_assert_eq!(done.message.is_empty(), done.count == 0);
        if let Some(token) = done.undo {
            prop_assert_eq!(library.undo(token), Ok(done.count));
        }
        prop_assert_eq!(prints(library, &ids), before);
    }

    /// Round trip: adding a tag and taking it off leaves the notes that
    /// lacked it as they were, and the others without it and otherwise as
    /// they were.
    #[test]
    fn prop_add_then_remove_a_tag(
        seeds in seeds(),
        pick in proptest::collection::vec(any::<bool>(), 1..8),
    ) {
        let (phone, _, ids, tags) = seeded(&seeds);
        let library = &phone.library;
        let chosen = chosen(&ids, &pick);
        let before = prints(library, &ids);
        let tag_id = tags[0].clone();
        library.change(chosen.clone(), NoteChange::AddTag { tag_id: tag_id.clone() }).unwrap();
        for id in &chosen {
            prop_assert!(library.note(id).unwrap().tags.contains(&tag_id));
        }
        library.change(chosen.clone(), NoteChange::RemoveTag { tag_id: tag_id.clone() }).unwrap();
        for (id, old) in ids.iter().zip(&before) {
            let now = print(&library.note(id).unwrap());
            if chosen.contains(id) && old.3.contains(&tag_id) {
                let mut expected = old.clone();
                expected.3.retain(|t| *t != tag_id);
                prop_assert_eq!(&now, &expected);
            } else {
                prop_assert_eq!(&now, old);
            }
        }
    }

    /// Idempotence: asking for a status twice writes the second time none,
    /// and the first writes exactly the notes that had another status.
    #[test]
    fn prop_set_status_is_idempotent(
        seeds in seeds(),
        pick in proptest::collection::vec(any::<bool>(), 1..8),
        status in 0usize..5,
    ) {
        let (phone, _, ids, _) = seeded(&seeds);
        let library = &phone.library;
        let chosen = chosen(&ids, &pick);
        let differing = chosen
            .iter()
            .filter(|id| library.note(id).unwrap().status != STATUSES[status])
            .count();
        let change = set_status(STATUSES[status]);
        prop_assert_eq!(library.change(chosen.clone(), change.clone()).unwrap().count as usize, differing);
        prop_assert_eq!(library.change(chosen.clone(), change).unwrap().count, 0);
        for id in &chosen {
            prop_assert_eq!(library.note(id).unwrap().status, STATUSES[status]);
        }
    }

    /// Restore moves every trashed note out of the trash, into the first
    /// notebook by name, leaves the rest alone, and Undo puts them back.
    #[test]
    fn prop_restore_moves_them_all_and_undoes(
        seeds in seeds(),
        pick in proptest::collection::vec(any::<bool>(), 1..8),
    ) {
        let (phone, books, ids, _) = seeded(&seeds);
        let library = &phone.library;
        let trashed = chosen(&ids, &pick);
        library.change(trashed.clone(), NoteChange::Trash).unwrap();
        let before = prints(library, &ids);
        let done = library.change(ids.clone(), NoteChange::Restore).unwrap();
        prop_assert_eq!(done.count as usize, trashed.len());
        prop_assert!(place(library, ListPlace::Trash).is_empty());
        for (id, old) in ids.iter().zip(&before) {
            let now = library.note(id).unwrap();
            if trashed.contains(id) {
                prop_assert_eq!(&now.notebook_id, &books[0], "Alpha is first by name");
            } else {
                prop_assert_eq!(&print(&now), old);
            }
        }
        if let Some(token) = done.undo {
            prop_assert_eq!(library.undo(token), Ok(done.count));
            prop_assert_eq!(prints(library, &ids), before);
        } else {
            prop_assert!(trashed.is_empty());
        }
    }

    /// Emptying the trash leaves the trash empty and every other count as
    /// it was.
    #[test]
    fn prop_empty_trash_leaves_the_rest(
        seeds in seeds(),
        pick in proptest::collection::vec(any::<bool>(), 1..8),
    ) {
        let (phone, _, ids, _) = seeded(&seeds);
        let library = &phone.library;
        let trashed = chosen(&ids, &pick);
        library.change(trashed.clone(), NoteChange::Trash).unwrap();
        let before = library.overview().unwrap();
        prop_assert_eq!(before.trash as usize, trashed.len());
        prop_assert_eq!(library.empty_trash().unwrap(), before.trash);
        let after = library.overview().unwrap();
        prop_assert_eq!(after.trash, 0);
        prop_assert_eq!((after.all, after.pinned), (before.all, before.pinned));
        prop_assert_eq!(after.by_status, before.by_status);
        for id in &ids {
            prop_assert_eq!(library.note(id).is_ok(), !trashed.contains(id));
        }
    }

    /// A duplicate holds the original's text, notebook, tags and status,
    /// under the title with " (copy)"; the original is untouched.
    #[test]
    fn prop_duplicate_preserves(seeds in seeds(), body in "[a-z ]{0,30}", which in 0usize..8) {
        let (phone, _, ids, _) = seeded(&seeds);
        let library = &phone.library;
        let id = &ids[which % ids.len()];
        let rev = library.note(id).unwrap().rev;
        library.save(id, &rev, body.clone()).unwrap();
        let original = library.note(id).unwrap();
        let copy = library.note(&library.duplicate(id).unwrap().id).unwrap();
        prop_assert_eq!(copy.title, format!("{} (copy)", original.title));
        prop_assert_eq!(&copy.body, &original.body);
        prop_assert_eq!(&copy.notebook_id, &original.notebook_id);
        prop_assert_eq!(&copy.tags, &original.tags);
        prop_assert_eq!(&copy.status, &original.status);
        prop_assert_eq!(library.note(id).unwrap(), original);
    }
}
