//! Shared notebooks, slice 36b (issue #390): a note another member deletes
//! lands in everyone's trash and can be restored, two clients see each other
//! live in a notebook's note (text, caret, comments), a person finds their
//! notebook invitations without a share id, and the notebook's property list
//! merges one property at a time. Against the in-memory server.

use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use kanso_core::id::DocType;
use kanso_core::schema::{PropertyDef, PropertyType};
use kanso_store::{Origin, Put};
use serde_json::json;

use super::Engine;
use super::book_tests::{body, invite, note_of, notebook, shared, sync, texts, write};
use super::tests::{Device, device};
use crate::memory::MemoryCollab;
use crate::model::{BookLiveNotice, Role};

fn wait_for(what: &str, mut check: impl FnMut() -> bool) {
    let deadline = Instant::now() + Duration::from_secs(15);
    while !check() {
        assert!(Instant::now() < deadline, "timed out waiting for {what}");
        std::thread::sleep(Duration::from_millis(20));
    }
}

fn book_of(d: &Device, share: &str) -> String {
    d.store.shared_book(share).unwrap().unwrap().book_id
}

fn stream_of(d: &Device, note: &str) -> String {
    d.store
        .book_stream_of_note(note)
        .unwrap()
        .expect("a notebook note")
        .stream
}

fn move_note(d: &Device, id: &str, to: &str) {
    d.store
        .put(
            DocType::Note,
            json!({"_id": id, "bookId": to}),
            Put::Merge,
            Origin::Ui,
        )
        .unwrap();
}

fn where_is(d: &Device, id: &str) -> String {
    d.store.note(id).unwrap().expect("the note").book_id
}

fn text_def(key: &str) -> PropertyDef {
    PropertyDef {
        key: key.to_owned(),
        kind: PropertyType::Text,
        options: Vec::new(),
    }
}

fn keys_of(d: &Device, book: &str) -> Vec<String> {
    let mut keys: Vec<String> = d
        .store
        .notebook_properties(book)
        .unwrap()
        .into_iter()
        .map(|def| def.key)
        .collect();
    keys.sort();
    keys
}

// ------------------------------------------------------------------ trash

#[test]
fn a_note_another_member_deletes_lands_in_every_members_trash_and_can_be_restored() {
    let server = MemoryCollab::new();
    let (a, b, c) = (
        device(&server, 1, 1),
        device(&server, 2, 1),
        device(&server, 3, 1),
    );
    let (book, _) = notebook(
        &a.store,
        "Studio",
        &[
            ("Keep".to_owned(), "keep".to_owned()),
            ("Drop".to_owned(), "drop".to_owned()),
        ],
    );
    let share = shared(&a, &[(&b, Role::Edit), (&c, Role::Edit)], &book);
    let (b_book, c_book) = (book_of(&b, &share), book_of(&c, &share));
    let (a_id, b_id, c_id) = (
        note_of(&a, &book, "Drop"),
        note_of(&b, &b_book, "Drop"),
        note_of(&c, &c_book, "Drop"),
    );
    // B deletes it (the app moves it to the trash).
    move_note(&b, &b_id, "trash");
    sync(&[&b, &a, &c], &share);
    for (d, id) in [(&a, &a_id), (&b, &b_id), (&c, &c_id)] {
        assert_eq!(where_is(d, id), "trash");
        // Trashed, not gone: the text is still there to restore.
        assert_eq!(body(d, id), "drop");
    }
    assert!(!texts(&a, &book).contains_key("Drop"));
    assert!(texts(&c, &c_book).contains_key("Keep"));
    // C restores it from its trash: it comes back for everyone.
    move_note(&c, &c_id, &c_book);
    sync(&[&c, &a, &b], &share);
    for (d, id, book) in [
        (&a, &a_id, &book),
        (&b, &b_id, &b_book),
        (&c, &c_id, &c_book),
    ] {
        assert_eq!(&where_is(d, id), book);
    }
    // And it stays: a further sync doesn't delete it again.
    sync(&[&a, &b, &c], &share);
    assert_eq!(where_is(&b, &b_id), b_book);
    assert_eq!(
        texts(&a, &book).get("Drop").map(String::as_str),
        Some("drop")
    );
}

#[test]
fn a_deletion_the_owner_makes_by_removing_the_note_also_reaches_the_others_trash() {
    let server = MemoryCollab::new();
    let (a, b) = (device(&server, 1, 1), device(&server, 2, 1));
    let (book, _) = notebook(&a.store, "Studio", &[("Gone".to_owned(), "x".to_owned())]);
    let share = shared(&a, &[(&b, Role::Edit)], &book);
    let b_book = book_of(&b, &share);
    let (a_id, b_id) = (note_of(&a, &book, "Gone"), note_of(&b, &b_book, "Gone"));
    a.store.delete(&a_id, Origin::Ui).unwrap();
    let synced = {
        a.engine.sync_book(&share).unwrap();
        b.engine.sync_book(&share).unwrap()
    };
    assert_eq!(synced.notes_trashed, 1);
    assert_eq!(where_is(&b, &b_id), "trash");
}

#[test]
fn an_edit_made_before_the_deletion_arrives_wins_and_brings_the_note_back() {
    let server = MemoryCollab::new();
    let (a, b) = (device(&server, 1, 1), device(&server, 2, 1));
    let (book, _) = notebook(&a.store, "Studio", &[("Plan".to_owned(), "one".to_owned())]);
    let share = shared(&a, &[(&b, Role::Edit)], &book);
    let b_book = book_of(&b, &share);
    let (a_id, b_id) = (note_of(&a, &book, "Plan"), note_of(&b, &b_book, "Plan"));
    move_note(&a, &a_id, "trash");
    // B, not having heard, edits the note.
    write(&b, &b_id, "one, edited");
    sync(&[&a, &b], &share);
    sync(&[&b, &a], &share);
    assert_eq!(where_is(&b, &b_id), b_book);
    assert_eq!(where_is(&a, &a_id), book);
    assert_eq!(body(&a, &a_id), "one, edited");
}

// -------------------------------------------------------------- invitations

#[test]
fn invitations_are_listed_without_a_share_id() {
    let server = MemoryCollab::new();
    let (a, b, c) = (
        device(&server, 1, 1),
        device(&server, 2, 1),
        device(&server, 3, 1),
    );
    let (one, _) = notebook(&a.store, "One", &[("n".to_owned(), "x".to_owned())]);
    let two = a
        .store
        .put(
            DocType::Book,
            json!({"name": "Two"}),
            Put::Upsert,
            Origin::Ui,
        )
        .unwrap()
        .id;
    a.store
        .put(
            DocType::Note,
            json!({"_id": "note:m", "bookId": two, "title": "m", "body": "y"}),
            Put::Upsert,
            Origin::Ui,
        )
        .unwrap();
    for who in [&b, &c] {
        a.engine.person(&who.engine.caller().username).unwrap();
    }
    let first = a
        .engine
        .share_book(&one, &invite(&[(&b, Role::Edit)]), false, &mut |_| {})
        .unwrap();
    let second = a
        .engine
        .share_book(
            &two,
            &invite(&[(&b, Role::View), (&c, Role::Edit)]),
            false,
            &mut |_| {},
        )
        .unwrap();
    // B asks with nothing but their own session.
    let mut mine = b.engine.book_invitations().unwrap();
    mine.sort_by(|x, y| x.share_id.cmp(&y.share_id));
    let mut want = vec![first.share_id.clone(), second.share_id.clone()];
    want.sort();
    assert_eq!(
        mine.iter().map(|i| i.share_id.clone()).collect::<Vec<_>>(),
        want
    );
    let for_two = mine.iter().find(|i| i.share_id == second.share_id).unwrap();
    assert_eq!(for_two.role, Role::View);
    assert_eq!(for_two.status, "invited");
    assert_eq!(for_two.owner_username, "user1");
    // C is only in the second; a stranger sees none; the owner is in both,
    // not invited to either.
    assert_eq!(
        c.engine
            .book_invitations()
            .unwrap()
            .iter()
            .map(|i| i.share_id.clone())
            .collect::<Vec<_>>(),
        vec![second.share_id.clone()]
    );
    assert!(
        device(&server, 4, 1)
            .engine
            .book_invitations()
            .unwrap()
            .is_empty()
    );
    assert!(a.engine.book_invitations().unwrap().is_empty());
    // Accepting one takes it off the list; it is still there to join on a
    // device that hasn't, until this one joins.
    b.engine.join_book(&first.share_id).unwrap();
    let left = b.engine.book_invitations().unwrap();
    assert_eq!(left.len(), 1);
    assert_eq!(left[0].share_id, second.share_id);
    assert_eq!(b.engine.joinable_books().unwrap().len(), 1);
    // Declining one removes it for good.
    b.engine
        .api
        .leave_book(b.engine.caller(), &second.share_id)
        .unwrap();
    assert!(b.engine.book_invitations().unwrap().is_empty());
}

// --------------------------------------------------------------- properties

#[test]
fn a_property_one_member_adds_appears_for_all_and_changes_merge_one_property_at_a_time() {
    let server = MemoryCollab::new();
    let (a, b, c) = (
        device(&server, 1, 1),
        device(&server, 2, 1),
        device(&server, 3, 1),
    );
    let (book, _) = notebook(&a.store, "Studio", &[("n".to_owned(), "x".to_owned())]);
    let share = shared(&a, &[(&b, Role::Edit), (&c, Role::Edit)], &book);
    let (b_book, c_book) = (book_of(&b, &share), book_of(&c, &share));
    // A adds one: B and C have it.
    a.store
        .set_notebook_properties(&book, &[text_def("stage")], Origin::Ui)
        .unwrap();
    sync(&[&a, &b, &c], &share);
    assert_eq!(keys_of(&b, &b_book), ["stage"]);
    assert_eq!(keys_of(&c, &c_book), ["stage"]);
    // B and C each add another before hearing of the other: all three end
    // with all three.
    b.store
        .set_notebook_properties(&b_book, &[text_def("stage"), text_def("due")], Origin::Ui)
        .unwrap();
    c.store
        .set_notebook_properties(
            &c_book,
            &[text_def("stage"), text_def("sponsor")],
            Origin::Ui,
        )
        .unwrap();
    sync(&[&b, &c, &a], &share);
    sync(&[&a, &b, &c], &share);
    for (d, book) in [(&a, &book), (&b, &b_book), (&c, &c_book)] {
        assert_eq!(keys_of(d, book), ["due", "sponsor", "stage"]);
    }
    // A removes one and B retypes another: both stand.
    a.store
        .set_notebook_properties(&book, &[text_def("due"), text_def("sponsor")], Origin::Ui)
        .unwrap();
    b.store
        .set_notebook_properties(
            &b_book,
            &[
                PropertyDef {
                    key: "due".into(),
                    kind: PropertyType::Date,
                    options: Vec::new(),
                },
                text_def("sponsor"),
                text_def("stage"),
            ],
            Origin::Ui,
        )
        .unwrap();
    sync(&[&a, &b, &c], &share);
    sync(&[&b, &c, &a], &share);
    for (d, book) in [(&a, &book), (&b, &b_book), (&c, &c_book)] {
        let defs = d.store.notebook_properties(book).unwrap();
        assert_eq!(
            defs.iter()
                .map(|p| p.key.as_str())
                .collect::<Vec<_>>()
                .len(),
            2,
            "{defs:?}"
        );
        assert!(
            defs.iter()
                .any(|p| p.key == "due" && p.kind == PropertyType::Date)
        );
        assert!(defs.iter().any(|p| p.key == "sponsor"));
    }
}

// --------------------------------------------------------------------- live

fn collect(notices: &Arc<Mutex<Vec<BookLiveNotice>>>) -> super::BookNotify {
    let notices = notices.clone();
    Arc::new(move |n| notices.lock().unwrap().push(n))
}

#[test]
fn two_clients_see_each_other_live_in_a_notebook_note() {
    let server = MemoryCollab::new();
    let (a, b) = (device(&server, 1, 1), device(&server, 2, 1));
    let (book, _) = notebook(
        &a.store,
        "Studio",
        &[("Script".to_owned(), "Hello".to_owned())],
    );
    let share = shared(&a, &[(&b, Role::Edit)], &book);
    let b_book = book_of(&b, &share);
    let (a_id, b_id) = (note_of(&a, &book, "Script"), note_of(&b, &b_book, "Script"));
    let (a_stream, b_stream) = (stream_of(&a, &a_id), stream_of(&b, &b_id));
    assert_eq!(a_stream, b_stream);
    let heard: Arc<Mutex<Vec<BookLiveNotice>>> = Default::default();
    a.engine.book_open_note(&share, &a_stream);
    b.engine.book_open_note(&share, &b_stream);
    let _a_live = Engine::go_live_book(&a.engine, &share, collect(&Default::default()));
    let _b_live = Engine::go_live_book(&b.engine, &share, collect(&heard));
    // Nobody calls sync: A types in the library, B's library gets it.
    wait_for("both rooms to connect", || {
        heard.lock().unwrap().contains(&BookLiveNotice::Connection {
            share_id: share.clone(),
            live: true,
        })
    });
    write(&a, &a_id, "Hello there");
    a.engine.set_book_caret(&share, &a_stream, 11, 11);
    wait_for("A's words in B's library", || {
        body(&b, &b_id) == "Hello there"
    });
    // B's caret is A's to see, and A's is B's.
    wait_for("A's caret in B's note", || {
        let cursors = b.engine.book_cursors(&share, &b_stream);
        cursors.len() == 1 && cursors[0].name == "user1" && cursors[0].head == 11
    });
    b.engine.set_book_caret(&share, &b_stream, 0, 5);
    wait_for("B's selection in A's note", || {
        let cursors = a.engine.book_cursors(&share, &a_stream);
        cursors.len() == 1
            && cursors[0].name == "user2"
            && (cursors[0].anchor, cursors[0].head) == (0, 5)
    });
    // And back: B edits, A's library follows.
    write(&b, &b_id, "Well, Hello there");
    wait_for("B's words in A's library", || {
        body(&a, &a_id) == "Well, Hello there"
    });
    // Both at once: the words of each survive.
    write(&a, &a_id, "Well, Hello there, all");
    write(&b, &b_id, "Oh well, Hello there");
    wait_for("both edits merged", || {
        let (ta, tb) = (body(&a, &a_id), body(&b, &b_id));
        ta == tb && ta.contains("all") && ta.starts_with("Oh well")
    });
    // The notices said so.
    assert!(heard.lock().unwrap().iter().any(|n| matches!(
        n,
        BookLiveNotice::Text { stream, .. } if *stream == b_stream
    )));
    assert!(
        heard
            .lock()
            .unwrap()
            .iter()
            .any(|n| matches!(n, BookLiveNotice::Presence { .. }))
    );
}

#[test]
fn comments_in_a_notebook_note_reach_the_other_member_live_and_over_http() {
    let server = MemoryCollab::new();
    let (a, b) = (device(&server, 1, 1), device(&server, 2, 1));
    let (book, _) = notebook(
        &a.store,
        "Studio",
        &[("Script".to_owned(), "Open on the hero shot".to_owned())],
    );
    let share = shared(&a, &[(&b, Role::Comment)], &book);
    let b_book = book_of(&b, &share);
    let (a_id, b_id) = (note_of(&a, &book, "Script"), note_of(&b, &b_book, "Script"));
    let stream = stream_of(&a, &a_id);
    a.engine.book_open_note(&share, &stream);
    b.engine.book_open_note(&share, &stream);
    // A commenter may comment, and not edit.
    let id = b
        .engine
        .add_book_comment(&share, &stream, 8..16, "Is it the hero shot?")
        .unwrap();
    let mine = b.engine.book_comments(&share, &stream).unwrap();
    assert_eq!(mine.len(), 1);
    assert!(mine[0].pending && mine[0].mine);
    // A's engine has no room open: the HTTP sync carries it.
    sync(&[&b, &a], &share);
    let seen = a.engine.book_comments(&share, &stream).unwrap();
    assert_eq!(seen.len(), 1);
    assert_eq!(seen[0].comment.id, id);
    assert_eq!(seen[0].comment.author_name, "user2");
    assert_eq!(seen[0].comment.body, "Is it the hero shot?");
    assert!(!seen[0].pending && !seen[0].mine);
    // A answers over the live rooms, resolving it.
    let heard: Arc<Mutex<Vec<BookLiveNotice>>> = Default::default();
    let _a_live = Engine::go_live_book(&a.engine, &share, collect(&Default::default()));
    let _b_live = Engine::go_live_book(&b.engine, &share, collect(&heard));
    wait_for("B's room", || {
        heard.lock().unwrap().contains(&BookLiveNotice::Connection {
            share_id: share.clone(),
            live: true,
        })
    });
    a.engine
        .resolve_book_comment(&share, &stream, &id, true)
        .unwrap();
    wait_for("the resolution on B", || {
        b.engine
            .book_comments(&share, &stream)
            .unwrap()
            .first()
            .is_some_and(|c| c.comment.resolved && !c.pending)
    });
    // A commenter's words reach the note's text: not at all (it was never
    // theirs to change).
    assert_eq!(body(&b, &b_id), "Open on the hero shot");
}

#[test]
fn a_member_who_is_removed_is_told_the_room_is_over() {
    let server = MemoryCollab::new();
    let (a, b) = (device(&server, 1, 1), device(&server, 2, 1));
    let (book, _) = notebook(&a.store, "Studio", &[("n".to_owned(), "x".to_owned())]);
    let share = shared(&a, &[(&b, Role::Edit)], &book);
    let heard: Arc<Mutex<Vec<BookLiveNotice>>> = Default::default();
    let _b_live = Engine::go_live_book(&b.engine, &share, collect(&heard));
    wait_for("B's room", || {
        heard.lock().unwrap().contains(&BookLiveNotice::Connection {
            share_id: share.clone(),
            live: true,
        })
    });
    a.engine
        .remove_book_member(&share, &b.engine.caller().account_id)
        .unwrap();
    wait_for("the end", || {
        heard.lock().unwrap().contains(&BookLiveNotice::Ended {
            share_id: share.clone(),
        })
    });
}
