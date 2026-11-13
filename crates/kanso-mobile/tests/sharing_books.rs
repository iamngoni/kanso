//! Shared notebooks between phones (#390 slice 39): two or three libraries
//! on one in-memory sync server and one in-memory collab server, each
//! signed in with a password and syncing for real, with real crypto. The
//! rank rules' properties are in `src/sharing/books.rs`; these walk the
//! calls the Share sheet, the Shared section and a note's page make.

mod common;

use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use kanso_collab::memory::MemoryCollab;
use kanso_mobile::{
    BookRole, BookStandingRow, KansoError, Library, ListPlace, ListQuery, MemorySecrets, NewNote,
    PersonLookup,
    SharedEvent, SharedNotes,
};
use kanso_sync::{DEFAULT_SERVER, DeviceIdentity, MemoryServer, Password};

const PASSWORD: &str = "correct horse battery";

struct World {
    server: Arc<MemoryServer>,
    collab: MemoryCollab,
}

struct Phone {
    _dir: tempfile::TempDir,
    library: Library,
}

fn world() -> World {
    World {
        server: Arc::new(MemoryServer::new()),
        collab: MemoryCollab::new(),
    }
}

impl World {
    fn phone(&self, name: &str) -> Phone {
        let device =
            DeviceIdentity::from_parts("l_ffffffffffffffffffffffffffffffff", "Setup", [9; 32]);
        let password = Password::new(PASSWORD, name).unwrap();
        let _ = kanso_sync::sign_up(
            self.server.as_ref(),
            &MemorySecrets::new(),
            DEFAULT_SERVER,
            name,
            &password,
            &device,
            &|_| {},
        );
        let dir = tempfile::tempdir().unwrap();
        let library = Library::open_for_test_with_collab(
            &dir.path().join("library.db"),
            Arc::new(MemorySecrets::new()),
            self.server.clone(),
            Arc::new(self.collab.clone()),
        )
        .unwrap();
        library
            .sync()
            .sign_in_password(
                DEFAULT_SERVER.to_owned(),
                name.to_owned(),
                PASSWORD.to_owned(),
                format!("{name}'s phone"),
            )
            .unwrap();
        let phone = Phone { _dir: dir, library };
        phone.shared().sync_shared().unwrap();
        phone
    }
}

impl Phone {
    fn shared(&self) -> SharedNotes {
        self.library.shared()
    }

    fn sync(&self) -> SharedEvent {
        self.shared().sync_shared().unwrap()
    }

    fn body(&self, note_id: &str) -> String {
        self.library.note(note_id).unwrap().body.unwrap_or_default()
    }

    fn edit(&self, note_id: &str, body: &str) {
        let rev = self.library.note(note_id).unwrap().rev;
        self.library.save(note_id, &rev, body.to_owned()).unwrap();
    }

    /// Looks `name` up until their keys are published, so they can be
    /// invited.
    fn find(&self, name: &str) {
        eventually(&format!("{name} can be found"), || {
            matches!(
                self.shared().lookup_person(name.into()),
                Ok(PersonLookup::Found(_))
            )
        });
    }

    fn notebook_with(&self, name: &str, notes: &[(&str, &str)]) -> String {
        let book = self
            .library
            .create_notebook(name.to_owned(), None)
            .unwrap()
            .id;
        for (title, body) in notes {
            self.library
                .create(NewNote {
                    notebook_id: Some(book.clone()),
                    title: (*title).to_owned(),
                    body: (*body).to_owned(),
                    ..NewNote::default()
                })
                .unwrap();
        }
        book
    }

    fn note_in(&self, notebook: &str, title: &str) -> String {
        self.library
            .list(ListQuery {
                place: ListPlace::Notebook {
                    id: notebook.to_owned(),
                },
                ..ListQuery::default()
            })
            .unwrap()
            .rows
            .into_iter()
            .find(|row| row.title == title)
            .unwrap_or_else(|| panic!("no note {title}"))
            .id
    }
}

fn eventually(what: &str, mut check: impl FnMut() -> bool) {
    let until = Instant::now() + Duration::from_secs(10);
    while Instant::now() < until {
        if check() {
            return;
        }
        std::thread::sleep(Duration::from_millis(25));
    }
    panic!("never happened: {what}");
}

/// Alice's "Studio" with two notes, shared with Bob as `role` and joined by
/// him. (world, alice, bob, alice's notebook, bob's notebook)
fn studio(role: BookRole) -> (World, Phone, Phone, String, String) {
    let world = world();
    let alice = world.phone("alice");
    let bob = world.phone("bob");
    alice.find("bob");
    let book = alice.notebook_with("Studio", &[("Brief", "Pixel review"), ("Shots", "B-roll")]);
    alice
        .shared()
        .share_book(book.clone(), vec![("bob".into(), role)])
        .unwrap();
    let invitation = bob.shared().book_invitations().unwrap().remove(0);
    let bobs = bob.shared().accept_book(invitation.share_id).unwrap();
    (world, alice, bob, book, bobs)
}

// no-prop-test: one walk through the Share sheet against a server; the
// rows' rank rules are properties in src/sharing/books.rs.
#[test]
fn a_notebook_is_shared_its_invitation_accepted_and_its_notes_read() {
    let world = world();
    let alice = world.phone("alice");
    let bob = world.phone("bob");
    alice.find("bob");
    let book = alice.notebook_with("Studio", &[("Brief", "Pixel review"), ("Shots", "B-roll")]);

    let check = alice.shared().can_share_book(book.clone()).unwrap();
    assert_eq!((check.name.as_str(), check.notes, check.locked), ("Studio", 2, 0));
    assert_eq!(check.locked_name, "Studio (locked)");
    assert!(alice.shared().book_info(book.clone()).unwrap().is_none());

    let progress = Arc::new(Mutex::new(Vec::new()));
    let heard = progress.clone();
    alice.shared().book_progress(move |p| {
        heard.lock().unwrap().push((p.done, p.total));
        true
    });
    let info = alice
        .shared()
        .share_book(book.clone(), vec![("bob".into(), BookRole::Editor)])
        .unwrap();
    assert_eq!(progress.lock().unwrap().last(), Some(&(2, 2)));
    assert_eq!(info.notebook_id, book);
    assert_eq!(info.name, "Studio");
    assert_eq!(info.my_role, BookRole::Owner);
    assert!(info.owned);
    assert_eq!(
        info.invite_roles,
        [BookRole::Manager, BookRole::Editor, BookRole::Commenter, BookRole::Viewer]
    );
    let who: Vec<_> = info
        .members
        .iter()
        .map(|m| (m.username.as_str(), m.role, m.me, m.standing))
        .collect();
    assert_eq!(
        who,
        [
            ("alice", BookRole::Owner, true, BookStandingRow::Member),
            ("bob", BookRole::Editor, false, BookStandingRow::Invited),
        ]
    );
    assert!(info.members[0].roles.is_empty() && !info.members[0].removable);
    assert!(info.members[1].removable);
    assert!(!info.members[1].fingerprint.is_empty());
    assert!(alice.shared().can_share_book(book.clone()).is_err());

    // Bob sees who, with what role, and no name until he joins.
    assert_eq!(bob.sync().book_invitations, 1);
    let invitations = bob.shared().book_invitations().unwrap();
    assert_eq!(invitations.len(), 1);
    assert_eq!(invitations[0].from, "alice");
    assert_eq!(invitations[0].role, BookRole::Editor);
    assert!(invitations[0].invited);
    let bobs = bob
        .shared()
        .accept_book(invitations[0].share_id.clone())
        .unwrap();
    assert!(bob.shared().book_invitations().unwrap().is_empty());
    assert_eq!(bob.body(&bob.note_in(&bobs, "Brief")), "Pixel review");

    // Both list it in their Shared section.
    let rows = bob.shared().shared_books().unwrap();
    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0].notebook_id, bobs);
    assert_eq!(rows[0].name, "Studio");
    assert_eq!(rows[0].my_role, BookRole::Editor);
    assert_eq!(rows[0].owner, "alice");
    assert_eq!(rows[0].members, ["alice", "bob"]);
    assert!(!rows[0].status.offline);
    assert_eq!(rows[0].status.waiting, 0);
    let mine = alice.shared().book_info(book).unwrap().unwrap();
    assert_eq!(mine.members[1].standing, BookStandingRow::Member);
    // Bob, an editor, may not invite or change anyone.
    let his = bob.shared().book_info(bobs).unwrap().unwrap();
    assert!(his.invite_roles.is_empty() && !his.owned);
    assert!(his.members.iter().all(|m| m.roles.is_empty() && !m.removable));
}

// no-prop-test: the people calls against a server.
#[test]
fn people_are_invited_given_roles_and_removed() {
    let (world, alice, bob, book, bobs) = studio(BookRole::Manager);
    let carol = world.phone("carol");
    alice.find("carol");
    bob.find("carol");
    bob.sync();

    // A manager invites below Manager.
    let as_manager = bob
        .shared()
        .book_info(bobs.clone())
        .unwrap()
        .unwrap();
    assert_eq!(
        as_manager.invite_roles,
        [BookRole::Editor, BookRole::Commenter, BookRole::Viewer]
    );
    let refused = bob
        .shared()
        .invite_to_book(bobs.clone(), "carol".into(), BookRole::Manager);
    assert!(matches!(refused, Err(KansoError::Invalid { .. })));
    let info = bob
        .shared()
        .invite_to_book(bobs.clone(), " carol ".into(), BookRole::Commenter)
        .unwrap();
    let carol_row = info.members.iter().find(|m| m.username == "carol").unwrap();
    assert_eq!(carol_row.standing, BookStandingRow::Invited);
    assert_eq!(
        carol_row.roles,
        [BookRole::Editor, BookRole::Commenter, BookRole::Viewer]
    );
    let share = carol.shared().book_invitations().unwrap().remove(0).share_id;
    let carols = carol.shared().accept_book(share).unwrap();
    assert_eq!(carol.body(&carol.note_in(&carols, "Shots")), "B-roll");

    // The owner raises her and lowers her again.
    alice.sync();
    let account = carol_row.account_id.clone();
    let info = alice
        .shared()
        .set_book_role(book.clone(), account.clone(), BookRole::Editor)
        .unwrap();
    let row = info.members.iter().find(|m| m.account_id == account).unwrap();
    assert_eq!(row.role, BookRole::Editor);
    let info = alice
        .shared()
        .set_book_role(book.clone(), account.clone(), BookRole::Viewer)
        .unwrap();
    assert_eq!(
        info.members.iter().find(|m| m.account_id == account).unwrap().role,
        BookRole::Viewer
    );

    // The words of the question, as the desktop's.
    let words = alice.shared().book_words("Studio".into(), "carol".into());
    assert_eq!(words.remove_title, "Remove carol from \"Studio\"?");
    assert!(words.remove_body.contains("changes the notebook's key"));
    assert_eq!(words.leave_title, "Leave \"Studio\"? You'll keep a private copy.");

    let info = alice
        .shared()
        .remove_from_book(book, account)
        .unwrap();
    assert!(info.members.iter().all(|m| m.username != "carol"));
    let event = carol.sync();
    assert!(
        event
            .book_notices
            .iter()
            .any(|m| m.contains("Your copy is in \"Studio (no longer shared)\""))
    );
    assert!(carol.shared().shared_books().unwrap().is_empty());
}

// no-prop-test: edits, the offline count and "Edited by" against a server.
#[test]
fn edited_by_names_who_changed_a_note_and_the_status_counts_whats_waiting() {
    let (_world, alice, bob, book, bobs) = studio(BookRole::Editor);
    let brief = bob.note_in(&bobs, "Brief");
    bob.edit(&brief, "Pixel review, cut 2");
    let waiting = bob.shared().book_status(bobs.clone()).unwrap().unwrap();
    assert_eq!(waiting.waiting, 1);
    let event = bob.sync();
    assert!(event.notebooks.contains(&bobs));
    assert_eq!(
        bob.shared().book_status(bobs.clone()).unwrap().unwrap().waiting,
        0
    );
    let mine = bob.shared().book_edits(bobs).unwrap();
    assert_eq!(mine.len(), 1);
    assert!(mine[0].me);

    alice.sync();
    let alices = alice.note_in(&book, "Brief");
    assert_eq!(alice.body(&alices), "Pixel review, cut 2");
    let edits = alice.shared().book_edits(book.clone()).unwrap();
    assert_eq!(edits.len(), 1);
    assert_eq!(edits[0].note_id, alices);
    assert_eq!(edits[0].name, "bob");
    assert!(!edits[0].me && edits[0].at_ms > 0);
    // A notebook that isn't shared has none, and no status.
    let other = alice.notebook_with("Private", &[]);
    assert!(alice.shared().book_edits(other.clone()).unwrap().is_empty());
    assert!(alice.shared().book_status(other).unwrap().is_none());
}

// no-prop-test: ending a notebook against a server.
#[test]
fn leaving_keeps_a_copy_and_stopping_tells_everyone() {
    let world = world();
    let (alice, bob, carol) = (world.phone("alice"), world.phone("bob"), world.phone("carol"));
    alice.find("bob");
    alice.find("carol");
    let book = alice.notebook_with("Studio", &[("Brief", "Pixel review")]);
    alice
        .shared()
        .share_book(
            book.clone(),
            vec![
                ("bob".into(), BookRole::Editor),
                ("carol".into(), BookRole::Viewer),
            ],
        )
        .unwrap();
    let bobs = bob
        .shared()
        .accept_book(bob.shared().book_invitations().unwrap().remove(0).share_id)
        .unwrap();
    let carols_invitation = carol.shared().book_invitations().unwrap().remove(0);
    let carols = carol
        .shared()
        .accept_book(carols_invitation.share_id)
        .unwrap();

    // The owner can't leave; a member can't stop.
    assert!(alice.shared().leave_book(book.clone()).is_err());
    assert!(bob.shared().stop_book(bobs.clone()).is_err());
    let left = bob.shared().leave_book(bobs).unwrap();
    assert_eq!(left.copy_name, "Studio (no longer shared)");
    assert!(left.message.starts_with("You left \"Studio\""));
    assert!(bob.shared().shared_books().unwrap().is_empty());
    assert_eq!(bob.body(&bob.note_in(&left.notebook_id, "Brief")), "Pixel review");

    let stopped = alice.shared().stop_book(book).unwrap();
    assert!(stopped.message.contains("isn't shared any more"));
    let event = carol.sync();
    assert!(
        event
            .book_notices
            .iter()
            .any(|m| m.contains("Your copy is in \"Studio (no longer shared)\""))
    );
    assert!(carol.shared().shared_books().unwrap().is_empty());
    assert!(carol.shared().book_info(carols).unwrap().is_none());
}

// no-prop-test: declining against a server.
#[test]
fn a_declined_invitation_goes_away() {
    let world = world();
    let alice = world.phone("alice");
    let bob = world.phone("bob");
    alice.find("bob");
    let book = alice.notebook_with("Studio", &[("Brief", "x")]);
    alice
        .shared()
        .share_book(book.clone(), vec![("bob".into(), BookRole::Viewer)])
        .unwrap();
    let share = bob.shared().book_invitations().unwrap().remove(0).share_id;
    bob.shared().decline_book(share).unwrap();
    assert!(bob.shared().book_invitations().unwrap().is_empty());
    let info = alice.shared().book_info(book).unwrap().unwrap();
    let row = info.members.iter().find(|m| m.username == "bob").unwrap();
    assert_eq!(row.standing, BookStandingRow::Left);
}

// no-prop-test: signed out, nothing reaches the server.
#[test]
fn signed_out_the_shared_section_is_empty_and_sharing_needs_an_account() {
    let dir = tempfile::tempdir().unwrap();
    let library = Library::open_for_test_with_collab(
        &dir.path().join("library.db"),
        Arc::new(MemorySecrets::new()),
        Arc::new(MemoryServer::new()),
        Arc::new(MemoryCollab::new()),
    )
    .unwrap();
    let shared = library.shared();
    assert!(shared.shared_books().unwrap().is_empty());
    assert!(shared.book_edits("book:x".into()).unwrap().is_empty());
    assert!(shared.book_status("book:x".into()).unwrap().is_none());
    assert_eq!(
        shared.can_share_book("book:x".into()),
        Err(KansoError::SignedOut)
    );
    assert_eq!(shared.book_invitations(), Err(KansoError::SignedOut));
    assert_eq!(
        shared.share_book("book:x".into(), vec![]),
        Err(KansoError::SignedOut)
    );
}

// no-prop-test: two open editors in a notebook note, waited for; where a
// caret lands is `visible_units`, with its property in editing.rs.
#[test]
fn a_notebook_note_shows_where_the_others_caret_is_by_their_list_name() {
    let (_world, alice, bob, book, bobs) = studio(BookRole::Editor);
    let alices = alice.note_in(&book, "Brief");
    let his = bob.note_in(&bobs, "Brief");
    alice.shared().open_live(alices.clone()).unwrap();
    bob.shared().open_live(his.clone()).unwrap();
    assert!(alice.shared().is_live(&alices));
    let alice_session = alice.library.edit(&alices).unwrap();
    let bob_session = bob.library.edit(&his).unwrap();
    assert!(bob_session.remote_cursors().unwrap().is_empty());
    let status = bob.shared().live_status(his.clone()).unwrap().unwrap();
    assert_eq!(status.role, kanso_mobile::ShareRole::Editor);

    alice_session
        .select(kanso_mobile::Range16 { start: 1, end: 4 }, false)
        .unwrap();
    eventually("Bob sees Alice's selection", || {
        bob_session
            .remote_cursors()
            .unwrap()
            .iter()
            .any(|c| c.name == "alice" && c.start == 1 && c.end == 4)
    });
    eventually("the room says it is live", || {
        bob.shared()
            .live_status(his.clone())
            .unwrap()
            .is_some_and(|s| s.connected)
    });
    assert!(bob.shared().book_status(bobs).unwrap().unwrap().live);

    // Closing the note leaves the room; the last one closes it.
    bob.shared().close_live(his.clone());
    assert!(!bob.shared().is_live(&his));
    assert!(bob_session.remote_cursors().unwrap().is_empty());
}

/// Types `text` at the end of the session's visible text.
fn type_at_end(session: &kanso_mobile::EditorSession, text: &str) {
    let snap = session.snapshot().unwrap();
    let end = snap.text.encode_utf16().count() as u32;
    let after = end + text.encode_utf16().count() as u32;
    session
        .apply(kanso_mobile::TextChange {
            base_version: snap.version,
            start: end,
            end,
            text: text.into(),
            selection: kanso_mobile::Range16 {
                start: after,
                end: after,
            },
            composing: None,
        })
        .unwrap();
}

// no-prop-test: a walk against a server; the merge's properties are in
// kanso-collab's `merge_apart`.
#[test]
fn typing_in_a_notebook_note_never_overwrites_a_collaborators_edit() {
    let (_world, alice, bob, book, bobs) = studio(BookRole::Editor);
    let alices = alice.note_in(&book, "Brief");
    let his = bob.note_in(&bobs, "Brief");
    let session = alice.library.edit(&alices).unwrap();
    // Alice types at the end; nothing saved yet.
    type_at_end(&session, " from Alice");
    // Bob changes the start, and it reaches Alice's library.
    bob.edit(&his, "Bob's pixel review");
    bob.sync();
    alice.sync();
    // Edits that don't overlap merge: no question, both kept.
    let snap = session.snapshot().unwrap();
    assert!(
        !matches!(snap.save, kanso_mobile::SaveState::Conflict { .. }),
        "{:?}",
        snap.save
    );
    assert_eq!(snap.text, "Bob's pixel review from Alice");
    // Leaving the note saves the merge.
    session.close().unwrap();
    assert_eq!(alice.body(&alices), "Bob's pixel review from Alice");
}

// no-prop-test: a walk against a server; the merge's properties are in
// kanso-collab's `merge_apart`.
#[test]
fn edits_that_meet_in_a_notebook_note_ask_which_to_keep() {
    let (_world, alice, bob, book, bobs) = studio(BookRole::Editor);
    let alices = alice.note_in(&book, "Brief");
    let his = bob.note_in(&bobs, "Brief");
    let session = alice.library.edit(&alices).unwrap();
    // Both type at the end of the same words.
    type_at_end(&session, " from Alice");
    bob.edit(&his, "Pixel review from Bob");
    bob.sync();
    alice.sync();
    let snap = session.snapshot().unwrap();
    assert_eq!(snap.save, kanso_mobile::SaveState::Conflict { deleted: false });
    assert_eq!(snap.text, "Pixel review from Alice");
    // Their version comes in when asked for.
    let theirs = session.use_theirs().unwrap();
    assert_eq!(theirs.text, "Pixel review from Bob");
}

// no-prop-test: a walk against a server; the orders are in kanso-collab's
// `book_conflict_tests`.
#[test]
fn a_note_deleted_here_and_edited_there_comes_back_saying_so() {
    let (_world, alice, bob, book, bobs) = studio(BookRole::Editor);
    let alices = alice.note_in(&book, "Brief");
    let his = bob.note_in(&bobs, "Brief");
    alice.library.trash(&alices).unwrap();
    bob.edit(&his, "Pixel review, cut 3");
    bob.sync();
    let event = alice.sync();
    assert_eq!(
        event.book_notices,
        ["\"Brief\" was edited after you deleted it, so it's back."]
    );
    assert_eq!(alice.body(&alice.note_in(&book, "Brief")), "Pixel review, cut 3");
    // Bob, who deleted nothing, is told nothing.
    assert!(bob.sync().book_notices.is_empty());
}
