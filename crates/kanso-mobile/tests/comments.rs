//! Comments on shared notes between accounts: three libraries on one
//! in-memory sync server and one in-memory shared-note server, each signed
//! in with a password, with real crypto. A comments, B sees it and hears
//! of it, B resolves and replies by role, and the edit and delete rules hold.

mod common;

use std::sync::mpsc;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use common::{LOCK_PASSWORD, note};
use kanso_collab::memory::MemoryCollab;
use kanso_mobile::{
    CommentPlacement, CommentRow, KansoError, Library, MemorySecrets, ShareRole, SharedEvent,
    SharedNotes,
};
use kanso_sync::{DEFAULT_SERVER, DeviceIdentity, MemoryServer, Password};
use proptest::prelude::*;

const PASSWORD: &str = "correct horse battery";

/// The text the comments sit in: "beta" is units 6..10, the emoji 17..19.
const BODY: &str = "Alpha beta gamma 😀 delta";

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
        Phone { _dir: dir, library }
    }
}

impl Phone {
    fn shared(&self) -> SharedNotes {
        self.library.shared()
    }

    fn sync(&self) -> SharedEvent {
        self.shared().sync_shared().unwrap()
    }

    fn edit(&self, note_id: &str, body: &str) {
        let rev = self.library.note(note_id).unwrap().rev;
        self.library.save(note_id, &rev, body.to_owned()).unwrap();
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

/// Alice's note "Plan", shared with Bob as a commenter and Carol as a viewer,
/// both accepted. Returns (alice, bob, carol, their three note IDs).
fn trio() -> (Phone, Phone, Phone, String, String, String) {
    let world = world();
    let alice = world.phone("alice");
    let bob = world.phone("bob");
    let carol = world.phone("carol");
    bob.sync();
    carol.sync();
    let alices = note(&alice.library, "Plan", BODY);
    alice
        .shared()
        .share(
            alices.clone(),
            vec![
                ("bob".into(), ShareRole::Commenter),
                ("carol".into(), ShareRole::Viewer),
            ],
        )
        .unwrap();
    let accept = |phone: &Phone| {
        let invitation = phone.shared().invitations().unwrap().remove(0);
        phone.shared().accept(invitation.share_id).unwrap()
    };
    let (bobs, carols) = (accept(&bob), accept(&carol));
    (alice, bob, carol, alices, bobs, carols)
}

fn by_body<'a>(rows: &'a [CommentRow], body: &str) -> &'a CommentRow {
    rows.iter()
        .find(|row| row.body == body)
        .unwrap_or_else(|| panic!("no comment saying {body:?} in {rows:?}"))
}

// no-prop-test: one walk through two devices commenting on a note, with each
// role's rights; the offset conversion's properties are in the crate.
#[test]
fn a_comment_travels_between_devices_and_each_role_may_what_it_may() {
    let (alice, bob, carol, alices, bobs, carols) = trio();
    let heard = Arc::new(Mutex::new(Vec::<SharedEvent>::new()));
    let sink = heard.clone();
    bob.shared().events(move |event| {
        sink.lock().unwrap().push(event);
        true
    });

    // Nothing yet: no comments, an empty band, and zero open.
    assert!(alice.shared().comments(alices.clone()).unwrap().is_empty());
    assert_eq!(alice.shared().comments_band(alices.clone()).unwrap(), "");
    assert_eq!(alice.shared().open_comments(alices.clone()).unwrap(), 0);

    // A comments on "beta", naming Bob.
    let first = alice
        .shared()
        .add_comment(alices.clone(), 6, 10, "Check @bob this".into())
        .unwrap();
    let rows = alice.shared().comments(alices.clone()).unwrap();
    assert_eq!(rows.len(), 1);
    let row = &rows[0];
    assert_eq!(row.id, first);
    assert_eq!(row.quote, "beta");
    assert_eq!(
        row.placement,
        CommentPlacement::Anchored { start: 6, end: 10 }
    );
    assert!((row.mine && row.can_edit && row.can_delete && row.can_resolve) && !row.resolved);
    assert_eq!(row.author_name, "alice");
    assert!(row.pending, "not on the server yet");
    let spans: Vec<_> = row.mentions.iter().map(|m| (m.start, m.end)).collect();
    assert_eq!(spans, [(6, 10)]);

    // B hears of it with the next sync, and sees it as someone else's.
    alice.sync();
    let found = bob.sync();
    assert_eq!(found.comment_notes, std::slice::from_ref(&bobs));
    let told = heard
        .lock()
        .unwrap()
        .iter()
        .any(|e| e.comment_notes == [bobs.clone()]);
    assert!(told);
    let seen = bob.shared().comments(bobs.clone()).unwrap();
    assert_eq!(seen.len(), 1);
    let row = &seen[0];
    assert_eq!(
        (row.quote.as_str(), row.author_name.as_str()),
        ("beta", "alice")
    );
    assert_eq!(
        row.placement,
        CommentPlacement::Anchored { start: 6, end: 10 }
    );
    assert!(!row.mine && !row.pending);
    // A commenter resolves and reopens; the words are the author's.
    assert!(row.can_resolve && !row.can_edit && !row.can_delete);
    assert_eq!(bob.shared().open_comments(bobs.clone()).unwrap(), 1);
    assert_eq!(bob.shared().comments_band(bobs.clone()).unwrap(), "1 open");
    // Another sync with nothing new says nothing of comments.
    assert!(bob.sync().comment_notes.is_empty());

    // B resolves it; A sees it resolved.
    bob.shared()
        .resolve_comment(bobs.clone(), first.clone(), true)
        .unwrap();
    bob.sync();
    assert_eq!(alice.sync().comment_notes, std::slice::from_ref(&alices));
    let rows = alice.shared().comments(alices.clone()).unwrap();
    assert!(rows[0].resolved);
    assert_eq!(alice.shared().open_comments(alices.clone()).unwrap(), 0);
    assert_eq!(
        alice.shared().comments_band(alices.clone()).unwrap(),
        "1 resolved"
    );
    // And A opens it again.
    alice
        .shared()
        .resolve_comment(alices.clone(), first.clone(), false)
        .unwrap();
    assert_eq!(alice.shared().open_comments(alices.clone()).unwrap(), 1);

    // B replies on the emoji: units 17..19 are one character.
    let second = bob
        .shared()
        .add_comment(bobs.clone(), 17, 19, "Nice one".into())
        .unwrap();
    bob.sync();
    alice.sync();
    let rows = alice.shared().comments(alices.clone()).unwrap();
    assert_eq!(rows.len(), 2);
    let reply = by_body(&rows, "Nice one");
    assert_eq!(reply.id, second);
    assert_eq!(reply.quote, "😀");
    assert_eq!(
        reply.placement,
        CommentPlacement::Anchored { start: 17, end: 19 }
    );
    assert!(!reply.mine && !reply.can_edit && !reply.can_delete && reply.can_resolve);
    // In text order.
    assert_eq!(rows[0].id, first);

    // Nobody changes the words of another's comment, or deletes it.
    let theirs = KansoError::invalid("You can only change your own comments.");
    let (a, id) = (alice.shared(), alices.clone());
    assert_eq!(
        a.edit_comment(id.clone(), second.clone(), "Mine now".into()),
        Err(theirs.clone())
    );
    assert_eq!(a.delete_comment(id.clone(), second.clone()), Err(theirs));
    assert_eq!(
        a.edit_comment(id.clone(), "nope".into(), "x".into()),
        Err(KansoError::NotFound)
    );
    // An author edits and deletes her own.
    assert_eq!(
        a.edit_comment(id.clone(), first.clone(), "  ".into()),
        Err(KansoError::invalid("A comment needs some words."))
    );
    a.edit_comment(id, first.clone(), "Check this, @bob".into())
        .unwrap();
    alice.sync();
    bob.sync();
    let seen = bob.shared().comments(bobs.clone()).unwrap();
    assert!(by_body(&seen, "Check this, @bob").edited_ms.is_some());
    bob.shared()
        .delete_comment(bobs.clone(), second.clone())
        .unwrap();
    bob.sync();
    alice.sync();
    let rows = alice.shared().comments(alices.clone()).unwrap();
    let ids: Vec<_> = rows.iter().map(|r| r.id.as_str()).collect();
    assert_eq!(ids, [first.as_str()]);

    // A viewer reads the comments and may do nothing to them.
    carol.sync();
    let read = carol.shared().comments(carols.clone()).unwrap();
    assert_eq!(read.len(), 1);
    assert!(!read[0].can_edit && !read[0].can_delete && !read[0].can_resolve);
    let view_only = KansoError::invalid("You can view this note, but not comment on it.");
    let c = carol.shared();
    assert_eq!(
        c.add_comment(carols.clone(), 0, 5, "Hi".into()),
        Err(view_only.clone())
    );
    assert_eq!(
        c.resolve_comment(carols.clone(), first.clone(), true),
        Err(view_only)
    );
}

// no-prop-test: one walk with the room open; the engine's live link is the
// engine's.
#[test]
fn a_comment_reaches_an_open_note_live() {
    let (alice, bob, _carol, alices, bobs, _) = trio();
    let (tx, rx) = mpsc::channel();
    bob.shared().events(move |event| tx.send(event).is_ok());
    bob.shared().open_live(bobs.clone()).unwrap();
    alice
        .shared()
        .add_comment(alices.clone(), 0, 5, "Live one".into())
        .unwrap();
    // The comment goes out with the log; Bob's room hears it.
    let mut heard = false;
    eventually("Bob's open note hears the comment", || {
        alice.sync();
        while let Ok(event) = rx.try_recv() {
            heard |= event.comment_notes.contains(&bobs);
        }
        heard
    });
    let rows = bob.shared().comments(bobs.clone()).unwrap();
    assert_eq!(by_body(&rows, "Live one").quote, "Alpha");
    bob.shared().close_live(bobs);
}

// no-prop-test: every call that names a locked note, refused.
#[test]
fn a_locked_note_has_no_comments_to_read_or_write() {
    let world = world();
    let alice = world.phone("alice");
    let id = note(&alice.library, "Secret", "the plan");
    alice
        .library
        .locks()
        .lock_note(&id, LOCK_PASSWORD.into())
        .unwrap();
    let shared = alice.shared();
    let locked = Err(KansoError::Locked);
    assert_eq!(shared.comments(id.clone()).map(|_| ()), locked);
    assert_eq!(shared.open_comments(id.clone()).map(|_| ()), locked);
    assert_eq!(shared.comments_band(id.clone()).map(|_| ()), locked);
    let added = shared.add_comment(id.clone(), 0, 3, "x".into());
    assert_eq!(added.map(|_| ()), locked);
    let edited = shared.edit_comment(id.clone(), "c".into(), "x".into());
    assert_eq!(edited, locked);
    assert_eq!(shared.delete_comment(id.clone(), "c".into()), locked);
    assert_eq!(shared.resolve_comment(id, "c".into(), true), locked);
}

// no-prop-test: one walk through words that go away, a selection that
// selects nothing and a note that is not shared.
#[test]
fn comments_follow_their_words_and_orphan_when_deleted() {
    let (alice, bob, _carol, alices, bobs, _) = trio();
    let a = alice.shared();
    a.add_comment(alices.clone(), 6, 10, "On beta".into())
        .unwrap();
    a.add_comment(alices.clone(), 20, 25, "On delta".into())
        .unwrap();
    alice.sync();
    bob.sync();

    // Text goes in front, and the anchors move with the words, in UTF-16.
    alice.edit(&alices, &format!("Intro 😀 {BODY}"));
    alice.sync();
    bob.sync();
    let _ = &bobs;
    let rows = a.comments(alices.clone()).unwrap();
    let body = alice.library.note(&alices).unwrap().body.unwrap();
    let units: Vec<u16> = body.encode_utf16().collect();
    for row in &rows {
        let CommentPlacement::Anchored { start, end } = row.placement else {
            panic!("{row:?} lost its words");
        };
        let said = String::from_utf16(&units[start as usize..end as usize]).unwrap();
        assert_eq!(said, row.quote);
    }

    // The words of "On beta" are deleted: it is orphaned and listed last.
    alice.edit(&alices, &body.replace("beta ", ""));
    alice.sync();
    let rows = a.comments(alices.clone()).unwrap();
    assert_eq!(rows.len(), 2);
    assert!(matches!(
        rows[0].placement,
        CommentPlacement::Anchored { .. }
    ));
    assert_eq!(rows[0].body, "On delta");
    assert!(matches!(
        rows[1].placement,
        CommentPlacement::Orphaned { .. }
    ));
    assert_eq!(rows[1].body, "On beta");
    assert_eq!(rows[1].quote, "beta");
    assert_eq!(a.comments_band(alices.clone()).unwrap(), "2 open");

    // Selecting nothing, or past the text, is refused in the desktop words.
    let select = KansoError::invalid("Select the words to comment on first.");
    assert_eq!(
        a.add_comment(alices.clone(), 3, 3, "x".into()),
        Err(select.clone())
    );
    assert_eq!(
        a.add_comment(alices.clone(), 0, 9999, "x".into()),
        Err(select)
    );
    assert_eq!(
        a.add_comment(alices.clone(), 0, 3, "  ".into()),
        Err(KansoError::invalid("A comment needs some words."))
    );

    // A note that is not shared has no comments; nothing is open on it.
    let private = note(&alice.library, "Mine", "words");
    assert!(matches!(
        a.comments(private.clone()),
        Err(KansoError::Invalid { .. })
    ));
    assert_eq!(a.open_comments(private.clone()).unwrap(), 0);
    assert_eq!(a.comments_band(private).unwrap(), "");
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(6))]

    /// A comment on a random range of a random text (astral characters,
    /// combining marks, CRLF) is read back on the same words: the quote is
    /// the text at the placement, in UTF-16 units, whatever the range.
    #[test]
    fn prop_a_comment_is_read_back_on_the_words_it_was_made_on(
        parts in prop::collection::vec(
            prop::sample::select(vec!["a", "b", " ", "\r\n", "é", "e\u{301}", "😀", "👩\u{200d}💻", "中"]),
            4..24,
        ),
        a in 0usize..100,
        b in 0usize..100,
    ) {
        let world = world();
        let alice = world.phone("alice");
        let bob = world.phone("bob");
        bob.sync();
        let id = note(&alice.library, "Plan", &parts.concat());
        // The library keeps the text it is given, perhaps with its line
        // ends made plain: the offsets count in what it holds.
        let body = alice.library.note(&id).unwrap().body.unwrap();
        let invites = vec![("bob".into(), ShareRole::Commenter)];
        alice.shared().share(id.clone(), invites).unwrap();
        let units: Vec<u16> = body.encode_utf16().collect();
        let boundaries: Vec<usize> = std::iter::once(0)
            .chain(body.char_indices().map(|(at, c)| at + c.len_utf8()))
            .map(|byte| body[..byte].encode_utf16().count())
            .collect();
        let (x, y) = (boundaries[a % boundaries.len()], boundaries[b % boundaries.len()]);
        let (start, end) = (x.min(y), x.max(y));
        let made = alice.shared().add_comment(id.clone(), start as u32, end as u32, "Note".into());
        if start == end {
            prop_assert!(made.is_err());
        } else {
            prop_assert!(made.is_ok(), "{made:?}");
            let rows = alice.shared().comments(id).unwrap();
            prop_assert_eq!(rows.len(), 1);
            let CommentPlacement::Anchored { start: s, end: e } = rows[0].placement else {
                return Err(TestCaseError::fail("orphaned on its own words"));
            };
            prop_assert_eq!((s as usize, e as usize), (start, end));
            let said = String::from_utf16(&units[start..end]).unwrap();
            prop_assert_eq!(rows[0].quote.as_str(), said.trim());
        }
    }
}

/// The words of `range` (UTF-16 units) of `text`.
fn words(text: &str, range: &kanso_mobile::Range16) -> String {
    let units: Vec<u16> = text.encode_utf16().collect();
    String::from_utf16(&units[range.start as usize..range.end as usize]).unwrap()
}

// no-prop-test: one walk through the editor's three comment calls on a
// note with front matter; the property below holds them over random words.
#[test]
fn the_editor_comments_on_its_selection_tints_it_and_selects_it_again() {
    let (alice, _bob, _carol, alices, _, _) = trio();
    let rev = alice.library.note(&alices).unwrap().rev;
    alice
        .library
        .save(&alices, &rev, format!("---\nstatus: draft\n---\n{BODY}"))
        .unwrap();
    let session = alice.library.edit(&alices).unwrap();
    let text = session.snapshot().unwrap().text;
    assert_eq!(text, BODY, "the front matter is not in the editor's text");

    // Nothing selected: refused in the desktop's words.
    assert_eq!(
        session.comment_on_selection("Hm".into()),
        Err(KansoError::invalid("Select the words to comment on first."))
    );
    // "delta", after the emoji.
    let delta = kanso_mobile::Range16 { start: 20, end: 25 };
    session.select(delta, false).unwrap();
    let id = session.comment_on_selection("Why delta?".into()).unwrap();
    assert_eq!(session.comment_marks().unwrap(), vec![delta]);
    assert_eq!(words(&text, &delta), "delta");

    // Selecting it again from elsewhere in the text.
    session
        .select(kanso_mobile::Range16 { start: 0, end: 0 }, false)
        .unwrap();
    assert_eq!(session.select_comment(id.clone()).unwrap(), Some(delta));
    assert_eq!(session.snapshot().unwrap().selection, delta);
    assert_eq!(session.select_comment("no-such".into()).unwrap(), None);

    // Resolved, it is no longer tinted.
    alice
        .shared()
        .resolve_comment(alices.clone(), id, true)
        .unwrap();
    assert!(session.comment_marks().unwrap().is_empty());

    // A note that isn't shared has none, and can't be commented on.
    let mine = note(&alice.library, "Mine", "words here");
    let private = alice.library.edit(&mine).unwrap();
    assert!(private.comment_marks().unwrap().is_empty());
    private
        .select(kanso_mobile::Range16 { start: 0, end: 5 }, false)
        .unwrap();
    assert!(matches!(
        private.comment_on_selection("x".into()),
        Err(KansoError::Invalid { .. })
    ));
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(6))]

    /// Whatever words of a note with front matter are selected, the comment
    /// made on them is tinted on exactly those words.
    #[test]
    fn prop_a_comment_from_the_editor_is_tinted_on_its_words(
        a in 0usize..40,
        b in 0usize..40,
    ) {
        let world = world();
        let alice = world.phone("alice");
        let bob = world.phone("bob");
        bob.sync();
        let id = note(&alice.library, "Plan", &format!("---\nk: v\n---\n{BODY}"));
        let invites = vec![("bob".into(), ShareRole::Commenter)];
        alice.shared().share(id.clone(), invites).unwrap();
        let session = alice.library.edit(&id).unwrap();
        let text = session.snapshot().unwrap().text;
        let boundaries: Vec<u32> = std::iter::once(0)
            .chain(text.char_indices().map(|(at, c)| at + c.len_utf8()))
            .map(|byte| text[..byte].encode_utf16().count() as u32)
            .collect();
        let (x, y) = (boundaries[a % boundaries.len()], boundaries[b % boundaries.len()]);
        let range = kanso_mobile::Range16 { start: x.min(y), end: x.max(y) };
        session.select(range, false).unwrap();
        let made = session.comment_on_selection("Note".into());
        if range.start == range.end {
            prop_assert!(made.is_err());
        } else {
            prop_assert!(made.is_ok(), "{made:?}");
            prop_assert_eq!(session.comment_marks().unwrap(), vec![range]);
        }
    }
}
