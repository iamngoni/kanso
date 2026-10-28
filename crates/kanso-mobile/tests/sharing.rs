//! Shared notes between accounts: two or three libraries on one in-memory
//! sync server and one in-memory shared-note server, each signed in with a
//! password and syncing for real, with real crypto.

mod common;

use std::sync::mpsc;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use common::{LOCK_PASSWORD, note};
use kanso_collab::memory::MemoryCollab;
use kanso_mobile::{
    KansoError, Library, MemorySecrets, PersonLookup, ShareRole, SharedEvent, SharedNotes,
};
use kanso_sync::{DEFAULT_SERVER, DeviceIdentity, MemoryServer, Password};
use proptest::prelude::*;

const PASSWORD: &str = "correct horse battery";

/// The servers every phone of a test talks to.
struct World {
    server: Arc<MemoryServer>,
    collab: MemoryCollab,
}

/// A library on its own folder, signed in as `name`.
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
    /// A new phone signed in to `name`'s account, made on first use.
    fn phone(&self, name: &str) -> Phone {
        let device =
            DeviceIdentity::from_parts("l_ffffffffffffffffffffffffffffffff", "Setup", [9; 32]);
        let password = Password::new(PASSWORD, name).unwrap();
        // The account exists after the first phone; a second phone signs in.
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
        let status = library
            .sync()
            .sign_in_password(
                DEFAULT_SERVER.to_owned(),
                name.to_owned(),
                PASSWORD.to_owned(),
                format!("{name}'s phone"),
            )
            .unwrap();
        assert_eq!(status.username.as_deref(), Some(name));
        Phone { _dir: dir, library }
    }
}

impl Phone {
    fn shared(&self) -> SharedNotes {
        self.library.shared()
    }

    /// Publishes this account's identity, as its first sync of shared notes
    /// does, so others can look it up.
    fn arrive(&self) -> SharedEvent {
        self.shared().sync_shared().unwrap()
    }

    fn body(&self, note_id: &str) -> Option<String> {
        self.library.note(note_id).unwrap().body
    }

    fn edit(&self, note_id: &str, body: &str) {
        let rev = self.library.note(note_id).unwrap().rev;
        self.library.save(note_id, &rev, body.to_owned()).unwrap();
    }
}

/// Polls `check` until it holds, for at most ten seconds.
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

/// Alice's note "Plan", shared with Bob as `role`, accepted by Bob. Returns
/// (alice, bob, alice's note ID, bob's note ID).
fn shared_pair(role: ShareRole) -> (World, Phone, Phone, String, String) {
    let world = world();
    let alice = world.phone("alice");
    let bob = world.phone("bob");
    bob.arrive();
    let note_id = note(&alice.library, "Plan", "Alpha");
    alice
        .shared()
        .share(note_id.clone(), vec![("bob".into(), role)])
        .unwrap();
    let invitation = bob.shared().invitations().unwrap().remove(0);
    let bobs = bob.shared().accept(invitation.share_id).unwrap();
    (world, alice, bob, note_id, bobs)
}

// no-prop-test: one walk through sharing, an invitation, an edit each way and
// the sheet's rows; the properties of the rows are in `sharing.rs`.
#[test]
fn a_note_is_shared_accepted_edited_and_merged_back() {
    let world = world();
    let alice = world.phone("alice");
    let bob = world.phone("bob");
    let heard = Arc::new(Mutex::new(Vec::<SharedEvent>::new()));
    let sink = heard.clone();
    bob.shared().events(move |event| {
        sink.lock().unwrap().push(event);
        true
    });
    let note_id = note(&alice.library, "Plan", "Alpha");

    // Signing in published Bob's keys (off the sign-in's thread): he can be
    // found before he has synced anything.
    let mut found = None;
    eventually("bob can be found", || {
        match alice.shared().lookup_person("bob".into()) {
            Ok(PersonLookup::Found(person)) => {
                found = Some(person);
                true
            }
            Ok(PersonLookup::Changed { .. }) => false,
            Err(KansoError::Invalid { message })
                if message.starts_with("There's no Kanso account called bob") =>
            {
                false
            }
            Err(error) => panic!("looking up Bob failed: {error:?}"),
        }
    });
    let person = found.unwrap();
    assert_eq!(bob.arrive().invitations, 0);
    assert_eq!(person.username, "bob");
    assert!(!person.verified && !person.fingerprint.is_empty());
    alice.shared().verify_person(person.account_id).unwrap();

    alice.shared().can_share(note_id.clone()).unwrap();
    let info = alice
        .shared()
        .share(note_id.clone(), vec![("bob".into(), ShareRole::Editor)])
        .unwrap();
    assert_eq!(info.note_id, note_id);
    assert_eq!(info.my_role, ShareRole::Owner);
    assert_eq!(info.owner, "alice");
    let who: Vec<_> = info
        .members
        .iter()
        .map(|m| (m.username.as_str(), m.role, m.me, m.invited))
        .collect();
    assert_eq!(
        who,
        [
            ("alice", ShareRole::Owner, true, false),
            ("bob", ShareRole::Editor, false, true)
        ]
    );
    assert!(info.members.iter().all(|m| !m.key_changed));
    assert!(info.links.is_empty() && info.requests.is_empty());
    assert_eq!(
        alice.shared().can_share(note_id.clone()),
        Err(KansoError::Invalid {
            message: "The note is already shared.".into()
        })
    );

    // Bob hears of it, and sees who and what role, never a title.
    let invitations = bob.shared().invitations().unwrap();
    assert_eq!(invitations.len(), 1);
    assert_eq!(invitations[0].from, "alice");
    assert_eq!(invitations[0].role, ShareRole::Editor);
    assert_eq!(invitations[0].title, None);
    assert_eq!(invitations, bob.shared().invitations().unwrap());
    assert_eq!(bob.arrive().invitations, 1);
    assert!(heard.lock().unwrap().iter().any(|e| e.invitations == 1));

    let bobs = bob
        .shared()
        .accept(invitations[0].share_id.clone())
        .unwrap();
    assert_eq!(bob.body(&bobs).as_deref(), Some("Alpha"));
    assert!(bob.shared().invitations().unwrap().is_empty());
    let seen = bob.shared().info(bobs.clone()).unwrap().unwrap();
    assert_eq!(seen.my_role, ShareRole::Editor);
    assert!(seen.members.iter().any(|m| m.me && m.username == "bob"));
    assert!(seen.links.is_empty() && seen.requests.is_empty());

    // Bob edits; Alice's next sync of shared notes has it, and says so.
    bob.edit(&bobs, "Alpha and Bob");
    bob.shared().sync_shared().unwrap();
    let (tx, rx) = mpsc::channel();
    alice.shared().events(move |event| tx.send(event).is_ok());
    let found = alice.shared().sync_shared().unwrap();
    assert_eq!(found.note_ids, std::slice::from_ref(&note_id));
    assert_eq!(alice.body(&note_id).as_deref(), Some("Alpha and Bob"));
    assert_eq!(rx.try_recv().unwrap(), found);

    // And the other way round.
    alice.edit(&note_id, "Alpha and Bob and Alice");
    alice.shared().sync_shared().unwrap();
    bob.shared().sync_shared().unwrap();
    assert_eq!(bob.body(&bobs).as_deref(), Some("Alpha and Bob and Alice"));
    // A sync with nothing new says nothing.
    assert!(alice.shared().sync_shared().unwrap().note_ids.is_empty());
}

// no-prop-test: one walk through a link, a request, its approval, a role
// change and a removal.
#[test]
fn a_link_joins_a_third_account_and_the_owner_manages_the_people() {
    let (world, alice, bob, note_id, bobs) = shared_pair(ShareRole::Editor);
    let carol = world.phone("carol");
    carol.arrive();

    // Only the owner makes links; a link has a role, and not the owner's.
    assert!(matches!(
        bob.shared().create_link(bobs.clone(), ShareRole::Viewer, 7),
        Err(KansoError::Invalid { .. })
    ));
    assert!(matches!(
        alice
            .shared()
            .create_link(note_id.clone(), ShareRole::Owner, 7),
        Err(KansoError::Invalid { .. })
    ));
    let url = alice
        .shared()
        .create_link(note_id.clone(), ShareRole::Commenter, 7)
        .unwrap();
    let info = alice.shared().info(note_id.clone()).unwrap().unwrap();
    assert_eq!(info.links.len(), 1);
    assert_eq!(info.links[0].role, ShareRole::Commenter);
    assert!(url.contains(&format!("/j/{}#", info.links[0].id)));
    assert!(
        !format!("{info:?}").contains(url.rsplit('#').next().unwrap()),
        "the link's secret is in no row"
    );

    // Carol asks; Alice sees her request, and lets her in.
    assert!(matches!(
        carol.shared().join("https://example.com/not-a-link".into()),
        Err(KansoError::Invalid { .. })
    ));
    let asked = carol.shared().join(url).unwrap();
    assert_eq!(asked.owner, "alice");
    assert_eq!(asked.role, ShareRole::Commenter);
    let info = alice.shared().info(note_id.clone()).unwrap().unwrap();
    assert_eq!(info.requests.len(), 1);
    assert_eq!(info.requests[0].username, "carol");
    assert_eq!(info.requests[0].role, ShareRole::Commenter);
    let carols_id = info.requests[0].account_id.clone();
    let info = alice
        .shared()
        .approve_request(note_id.clone(), carols_id.clone())
        .unwrap();
    assert!(info.requests.is_empty());
    assert!(info.members.iter().any(|m| m.username == "carol"));

    // Approved through a link, she is a member at once: the next sync
    // brings the note in, with the link's role and Alice's text.
    assert!(carol.shared().invitations().unwrap().is_empty());
    let found = carol.shared().sync_shared().unwrap();
    assert_eq!(found.note_ids.len(), 1);
    let carols = found.note_ids[0].clone();
    assert_eq!(carol.body(&carols).as_deref(), Some("Alpha"));
    assert_eq!(
        carol
            .shared()
            .info(carols.clone())
            .unwrap()
            .unwrap()
            .my_role,
        ShareRole::Commenter
    );

    // A commenter can't change the text: the library refuses her save.
    let rev = carol.library.note(&carols).unwrap().rev;
    assert!(matches!(
        carol.library.save(&carols, &rev, "Carol rewrote it".into()),
        Err(KansoError::Invalid { message }) if message.contains("can't be changed")
    ));
    carol.shared().sync_shared().unwrap();
    alice.shared().sync_shared().unwrap();
    assert_eq!(alice.body(&note_id).as_deref(), Some("Alpha"));

    // The owner changes Bob's role, then removes Carol and her link.
    let bobs_id = info
        .members
        .iter()
        .find(|m| m.username == "bob")
        .unwrap()
        .account_id
        .clone();
    let info = alice
        .shared()
        .set_role(note_id.clone(), bobs_id.clone(), ShareRole::Viewer)
        .unwrap();
    assert_eq!(
        info.members
            .iter()
            .find(|m| m.username == "bob")
            .unwrap()
            .role,
        ShareRole::Viewer
    );
    assert!(matches!(
        alice
            .shared()
            .set_role(note_id.clone(), bobs_id.clone(), ShareRole::Owner),
        Err(KansoError::Invalid { .. })
    ));
    bob.shared().sync_shared().unwrap();
    assert_eq!(
        bob.shared().info(bobs.clone()).unwrap().unwrap().my_role,
        ShareRole::Viewer
    );
    let info = alice.shared().remove(note_id.clone(), carols_id).unwrap();
    assert!(info.members.iter().all(|m| m.username != "carol"));
    let link = info.links[0].id.clone();
    let info = alice.shared().cancel_link(note_id, link).unwrap();
    assert!(info.links.is_empty());
}

// no-prop-test: one walk through stopping, leaving and declining.
#[test]
fn stopping_and_leaving_keep_everyone_a_private_copy() {
    let (world, alice, bob, note_id, bobs) = shared_pair(ShareRole::Editor);

    // The right person for each: an editor can't stop, the owner can't leave.
    assert!(matches!(
        bob.shared().stop(bobs.clone()),
        Err(KansoError::Invalid { .. })
    ));
    assert!(matches!(
        alice.shared().leave(note_id.clone()),
        Err(KansoError::Invalid { .. })
    ));
    assert!(matches!(
        alice.shared().stop("note:nothing".into()),
        Err(KansoError::NotFound)
    ));
    let loose = note(&alice.library, "Loose", "not shared");
    assert_eq!(alice.shared().info(loose.clone()).unwrap(), None);
    assert!(matches!(
        alice.shared().stop(loose),
        Err(KansoError::Invalid { .. })
    ));

    // Bob edits, then leaves: he keeps the note as it is, Alice no longer
    // has him in it.
    bob.edit(&bobs, "Alpha, Bob was here");
    bob.shared().sync_shared().unwrap();
    bob.shared().leave(bobs.clone()).unwrap();
    assert_eq!(bob.shared().info(bobs.clone()).unwrap(), None);
    assert_eq!(bob.body(&bobs).as_deref(), Some("Alpha, Bob was here"));
    alice.shared().sync_shared().unwrap();
    assert_eq!(alice.body(&note_id).as_deref(), Some("Alpha, Bob was here"));
    let info = alice.shared().info(note_id.clone()).unwrap().unwrap();
    assert!(
        info.members.iter().all(|m| m.username != "bob" || m.left),
        "bob is out: {info:?}"
    );

    // Alice stops it: her note is plain again, and another account in it
    // would keep its copy.
    let carol = world.phone("carol");
    carol.arrive();
    alice
        .shared()
        .invite(note_id.clone(), "carol".into(), ShareRole::Viewer)
        .unwrap();
    let waiting = carol.shared().invitations().unwrap().remove(0);
    let carols = carol.shared().accept(waiting.share_id).unwrap();
    alice.shared().stop(note_id.clone()).unwrap();
    assert_eq!(alice.shared().info(note_id.clone()).unwrap(), None);
    assert_eq!(alice.body(&note_id).as_deref(), Some("Alpha, Bob was here"));
    carol.shared().sync_shared().unwrap();
    assert_eq!(carol.shared().info(carols.clone()).unwrap(), None);
    assert_eq!(carol.body(&carols).as_deref(), Some("Alpha, Bob was here"));

    // Declining an invitation leaves nothing behind.
    let again = note(&alice.library, "Again", "second");
    alice
        .shared()
        .share(again, vec![("bob".into(), ShareRole::Viewer)])
        .unwrap();
    let waiting = bob.shared().invitations().unwrap();
    assert_eq!(waiting.len(), 1);
    bob.shared().decline(waiting[0].share_id.clone()).unwrap();
    assert!(bob.shared().invitations().unwrap().is_empty());
}

// no-prop-test: every call that names a note, against one locked note.
#[test]
fn a_locked_note_is_refused_everywhere_and_never_shown() {
    let world = world();
    let alice = world.phone("alice");
    let bob = world.phone("bob");
    bob.arrive();
    let id = note(&alice.library, "Secret", "the plan");
    alice
        .library
        .locks()
        .lock_note(&id, LOCK_PASSWORD.into())
        .unwrap();

    let refused = Err(KansoError::Locked);
    let shared = alice.shared();
    assert_eq!(shared.can_share(id.clone()), refused);
    assert_eq!(
        shared
            .share(id.clone(), vec![("bob".into(), ShareRole::Viewer)])
            .map(|_| ()),
        refused
    );
    assert_eq!(shared.info(id.clone()).map(|_| ()), refused);
    assert_eq!(shared.open_live(id.clone()), refused);
    assert_eq!(
        shared
            .create_link(id.clone(), ShareRole::Viewer, 1)
            .map(|_| ()),
        refused
    );
    assert_eq!(shared.stop(id.clone()), refused);
    assert_eq!(shared.leave(id.clone()), refused);
    assert_eq!(
        shared
            .invite(id.clone(), "bob".into(), ShareRole::Viewer)
            .map(|_| ()),
        refused
    );
    assert!(bob.shared().invitations().unwrap().is_empty());
    assert!(!alice.shared().is_live(&id));

    // A shared note can't be locked at all: the store says to stop sharing
    // it first, so no shared note's text is ever sealed behind a lock.
    let open = note(&alice.library, "Plan", "Alpha");
    alice
        .shared()
        .share(open.clone(), vec![("bob".into(), ShareRole::Editor)])
        .unwrap();
    assert!(matches!(
        alice.library.locks().lock_note(&open, LOCK_PASSWORD.into()),
        Err(KansoError::Invalid { message }) if message.contains("stop sharing")
    ));
    assert!(alice.shared().info(open).unwrap().is_some());
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(8))]

    /// Whatever a locked note says, nothing the sharing calls return holds
    /// it, and every call that names the note is refused.
    #[test]
    fn prop_no_sharing_call_shows_a_locked_notes_text(
        title in "[A-Za-z][A-Za-z0-9 ]{3,20}",
        body in "[A-Za-z0-9 ]{8,60}",
    ) {
        let world = world();
        let alice = world.phone("alice");
        let bob = world.phone("bob");
        bob.arrive();
        let id = note(&alice.library, &title, &body);
        alice.library.locks().lock_note(&id, LOCK_PASSWORD.into()).unwrap();
        alice.library.locks().lock().unwrap();
        let shared = alice.shared();
        let calls = [
            format!("{:?}", shared.can_share(id.clone())),
            format!("{:?}", shared.info(id.clone())),
            format!("{:?}", shared.share(id.clone(), vec![("bob".into(), ShareRole::Editor)])),
            format!("{:?}", shared.open_live(id.clone())),
            format!("{:?}", shared.sync_shared()),
            format!("{:?}", shared.invitations()),
            format!("{:?}", bob.shared().invitations()),
            format!("{:?}", bob.shared().sync_shared()),
        ];
        for shown in calls {
            prop_assert!(!shown.contains(body.trim()), "the text is in {shown}");
            prop_assert!(!shown.contains(title.trim()), "the title is in {shown}");
        }
        prop_assert!(matches!(shared.can_share(id.clone()), Err(KansoError::Locked)));
        prop_assert!(bob.shared().invitations().unwrap().is_empty());
    }
}

// no-prop-test: two live rooms and edits across them, waiting for the
// merge; the engine's own properties are in kanso-collab.
#[test]
fn an_edit_in_a_live_note_reaches_the_other_library_without_a_sync() {
    let (_world, alice, bob, note_id, bobs) = shared_pair(ShareRole::Editor);
    alice.shared().open_live(note_id.clone()).unwrap();
    bob.shared().open_live(bobs.clone()).unwrap();
    // Opening it twice is one room.
    bob.shared().open_live(bobs.clone()).unwrap();
    assert!(alice.shared().is_live(&note_id) && bob.shared().is_live(&bobs));

    alice.edit(&note_id, "Alpha from Alice");
    eventually("Bob sees Alice's edit", || {
        bob.body(&bobs).as_deref() == Some("Alpha from Alice")
    });
    bob.edit(&bobs, "Alpha from Alice, Bob adds");
    eventually("Alice sees Bob's edit", || {
        alice.body(&note_id).as_deref() == Some("Alpha from Alice, Bob adds")
    });

    // The change stream reports what the room merged.
    let (tx, rx) = mpsc::channel();
    bob.library.changes(move |ids| tx.send(ids).is_ok());
    alice.edit(&note_id, "Alpha from Alice, Bob adds, then Alice again");
    let deadline = Instant::now() + Duration::from_secs(10);
    let mut reported = false;
    while Instant::now() < deadline && !reported {
        if let Ok(ids) = rx.recv_timeout(Duration::from_millis(100)) {
            reported = ids.contains(&bobs);
        }
    }
    assert!(reported, "the library's changes stream reported the merge");

    // Leaving the room: nothing more arrives, the note still syncs by hand.
    bob.shared().close_live(bobs.clone());
    assert!(!bob.shared().is_live(&bobs));
    alice.edit(&note_id, "Alpha, said after Bob left the room");
    std::thread::sleep(Duration::from_millis(700));
    assert_eq!(
        bob.body(&bobs).as_deref(),
        Some("Alpha from Alice, Bob adds, then Alice again")
    );
    bob.shared().sync_shared().unwrap();
    eventually("a manual sync still brings it", || {
        alice.shared().sync_shared().ok();
        bob.shared().sync_shared().ok();
        bob.body(&bobs).as_deref() == Some("Alpha, said after Bob left the room")
    });
}

// no-prop-test: the background closes rooms, and so does a sign-out.
#[test]
fn going_to_the_background_or_signing_out_closes_every_room() {
    let (_world, alice, bob, note_id, bobs) = shared_pair(ShareRole::Editor);
    alice.shared().open_live(note_id.clone()).unwrap();
    bob.shared().open_live(bobs.clone()).unwrap();
    alice.library.sync().set_foreground(false);
    assert!(!alice.shared().is_live(&note_id), "behind: rooms closed");
    assert!(bob.shared().is_live(&bobs), "another phone's stay open");

    // Signed out, sharing says so and the room is gone.
    bob.library.sync().sign_out().unwrap();
    assert_eq!(bob.shared().invitations(), Err(KansoError::SignedOut));
    assert!(!bob.shared().is_live(&bobs));
    assert_eq!(bob.shared().sync_shared(), Err(KansoError::SignedOut));
    assert_eq!(bob.shared().open_live(bobs), Err(KansoError::SignedOut));
}

// no-prop-test: signed out, each call, once.
#[test]
fn signed_out_every_call_says_so() {
    let world = world();
    let alice = world.phone("alice");
    let id = note(&alice.library, "Plan", "Alpha");
    alice.library.sync().sign_out().unwrap();
    let shared = alice.shared();
    assert_eq!(shared.can_share(id.clone()), Err(KansoError::SignedOut));
    assert_eq!(
        shared.share(id.clone(), vec![]).map(|_| ()),
        Err(KansoError::SignedOut)
    );
    assert_eq!(
        shared.info(id.clone()).map(|_| ()),
        Err(KansoError::SignedOut)
    );
    assert_eq!(shared.invitations().map(|_| ()), Err(KansoError::SignedOut));
    assert_eq!(
        shared.join("x".into()).map(|_| ()),
        Err(KansoError::SignedOut)
    );
    assert_eq!(shared.sync_shared().map(|_| ()), Err(KansoError::SignedOut));
    // A note that isn't there is not found, signed in or not.
    assert_eq!(
        shared.can_share("note:gone".into()),
        Err(KansoError::NotFound)
    );
}

// no-prop-test: the schedule's sync of shared notes, heard through events.
#[test]
fn an_account_sync_in_front_brings_the_shared_notes_with_it() {
    let world = world();
    let alice = world.phone("alice");
    let bob = world.phone("bob");
    bob.arrive();
    let (tx, rx) = mpsc::channel();
    bob.shared().events(move |event| tx.send(event).is_ok());
    let note_id = note(&alice.library, "Plan", "Alpha");
    alice
        .shared()
        .share(note_id, vec![("bob".into(), ShareRole::Editor)])
        .unwrap();

    // No call of ours: the app coming to the front syncs, and then syncs
    // the shared notes, which finds the invitation.
    let sync = bob.library.sync();
    sync.set_schedule_for_test(Duration::from_secs(3600), Duration::from_secs(3600));
    sync.set_foreground(true);
    let event = rx.recv_timeout(Duration::from_secs(10)).unwrap();
    assert_eq!(event.invitations, 1);
    sync.set_foreground(false);
}

/// Types `text` at the end of the session's visible text, leaving the caret
/// after it.
fn type_at_end(session: &kanso_mobile::EditorSession, text: &str) {
    let snapshot = session.snapshot().unwrap();
    let end = snapshot.text.encode_utf16().count() as u32;
    let after = end + text.encode_utf16().count() as u32;
    session
        .apply(kanso_mobile::TextChange {
            base_version: snapshot.version,
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

/// Ticks `session`'s autosave until `check` holds of its snapshot.
fn ticked_until(
    session: &kanso_mobile::EditorSession,
    what: &str,
    mut check: impl FnMut(&kanso_mobile::EditorSnapshot) -> bool,
) -> kanso_mobile::EditorSnapshot {
    let until = Instant::now() + Duration::from_secs(10);
    while Instant::now() < until {
        session.tick(Instant::now() + Duration::from_secs(5));
        let snapshot = session.snapshot().unwrap();
        if check(&snapshot) {
            return snapshot;
        }
        std::thread::sleep(Duration::from_millis(25));
    }
    panic!("never happened: {what}");
}

// no-prop-test: two people typing at once through an open editor, waiting
// for the room; the merge's own properties (rebase, remap) are in
// kanso-collab.
#[test]
fn typing_in_an_open_shared_note_merges_with_the_others_and_keeps_the_caret() {
    let (_world, alice, bob, note_id, bobs) = shared_pair(ShareRole::Editor);
    alice.shared().open_live(note_id.clone()).unwrap();
    bob.shared().open_live(bobs.clone()).unwrap();
    let session = bob.library.edit(&bobs).unwrap();
    let events = Arc::new(Mutex::new(Vec::new()));
    {
        let events = events.clone();
        session.events(move |event| {
            events.lock().unwrap().push(event);
            true
        });
    }

    // Bob types at the end; the room takes it when his typing is saved.
    type_at_end(&session, " Bob");
    ticked_until(&session, "Bob's typing is merged", |s| {
        matches!(s.save, kanso_mobile::SaveState::Saved { .. })
    });
    eventually("Alice sees Bob's typing", || {
        alice.body(&note_id).as_deref() == Some("Alpha Bob")
    });

    // Alice writes at the start while Bob has typed more and not saved.
    alice.edit(&note_id, "Hi. Alpha Bob");
    type_at_end(&session, "!");
    let merged = ticked_until(&session, "Alice's words reach Bob's editor", |s| {
        s.text == "Hi. Alpha Bob!"
    });
    // Bob's caret stays after his "!", four characters further on.
    let end = "Hi. Alpha Bob!".encode_utf16().count() as u32;
    assert_eq!(merged.selection, kanso_mobile::Range16 { start: end, end });
    // Nobody is asked to choose between versions.
    assert!(
        !events
            .lock()
            .unwrap()
            .iter()
            .any(|event| matches!(event, kanso_mobile::EditorEvent::Incoming { .. })),
        "a live merge is never a conflict"
    );
    eventually("Alice gets Bob's \"!\" too", || {
        alice.body(&note_id).as_deref() == Some("Hi. Alpha Bob!")
    });
    assert_eq!(bob.body(&bobs).as_deref(), Some("Hi. Alpha Bob!"));
}

// no-prop-test: a room's connection and end, waited for; the status's rows
// are read straight from the engine, whose properties are in kanso-collab.
#[test]
fn the_page_says_the_role_whether_the_room_is_live_and_when_it_ends() {
    let (_world, alice, bob, note_id, bobs) = shared_pair(ShareRole::Viewer);
    let loose = note(&bob.library, "Loose", "not shared");
    assert_eq!(bob.shared().live_status(loose).unwrap(), None);

    // Not open: the role, and nothing live.
    let status = bob.shared().live_status(bobs.clone()).unwrap().unwrap();
    assert_eq!(status.role, ShareRole::Viewer);
    assert!(!status.connected && !status.ended && !status.near_limit);
    assert!(status.problems.is_empty());

    // Open: the room connects, and says so.
    let (tx, rx) = mpsc::channel();
    bob.shared().events(move |event| tx.send(event).is_ok());
    bob.shared().open_live(bobs.clone()).unwrap();
    eventually("Bob's room connects", || {
        bob.shared()
            .live_status(bobs.clone())
            .unwrap()
            .is_some_and(|s| s.connected)
    });
    let told = rx.try_iter().any(|event| event.live_notes.contains(&bobs));
    assert!(told, "the connection was told as a live note");
    let owner = alice
        .shared()
        .live_status(note_id.clone())
        .unwrap()
        .unwrap();
    assert_eq!(owner.role, ShareRole::Owner);

    // Alice stops sharing: Bob's open note is a private copy now.
    alice.shared().stop(note_id).unwrap();
    eventually("Bob's room ends", || {
        bob.shared()
            .live_status(bobs.clone())
            .unwrap()
            .is_some_and(|s| s.ended && !s.connected)
    });
}

// no-prop-test: one note grown past its size limit through the room; the
// limit's arithmetic is `SizeUse::is_near`, tested in kanso-collab.
#[test]
fn a_note_near_its_limit_says_so_and_a_refused_edit_is_a_problem() {
    use kanso_collab::engine::STATE_LIMIT;

    let (_world, alice, _bob, note_id, _bobs) = shared_pair(ShareRole::Editor);
    alice.shared().open_live(note_id.clone()).unwrap();
    let status = || {
        alice
            .shared()
            .live_status(note_id.clone())
            .unwrap()
            .unwrap()
    };
    assert!(!status().near_limit);

    let near = "😀".repeat(1_030_000);
    assert!(near.len() * 5 >= STATE_LIMIT * 4 && near.len() < STATE_LIMIT);
    alice.edit(&note_id, &near);
    eventually("the note is near its limit", || status().near_limit);
    assert!(status().problems.is_empty());

    let past = format!("{near}{}", "😀".repeat(18_000));
    alice.edit(&note_id, &past);
    eventually("the refused edit is a problem", || {
        status().problems.iter().any(|p| p.contains("size limit"))
    });
    alice.shared().dismiss_problems(note_id.clone()).unwrap();
    assert!(status().problems.is_empty());
}

// no-prop-test: two open editors and the presence between them, waited
// for; where a caret lands is `visible_units`, with its property in
// editing.rs.
#[test]
fn each_editor_shows_where_the_others_caret_is() {
    let (_world, alice, bob, note_id, bobs) = shared_pair(ShareRole::Editor);
    alice.shared().open_live(note_id.clone()).unwrap();
    bob.shared().open_live(bobs.clone()).unwrap();
    let alices = alice.library.edit(&note_id).unwrap();
    let bobs_session = bob.library.edit(&bobs).unwrap();
    assert!(bobs_session.remote_cursors().unwrap().is_empty());

    // Alice selects "lph".
    alices
        .select(kanso_mobile::Range16 { start: 1, end: 4 }, false)
        .unwrap();
    eventually("Bob sees Alice's selection", || {
        bobs_session
            .remote_cursors()
            .unwrap()
            .iter()
            .any(|c| c.name == "alice" && c.start == 1 && c.end == 4 && c.caret == 4)
    });
    let mark = bobs_session.remote_cursors().unwrap().remove(0);
    assert_ne!(mark.light, mark.dark, "a colour for each page");

    // She moves her caret to the start.
    alices
        .select(kanso_mobile::Range16 { start: 0, end: 0 }, false)
        .unwrap();
    eventually("Bob sees her caret move", || {
        bobs_session
            .remote_cursors()
            .unwrap()
            .iter()
            .any(|c| c.name == "alice" && c.caret == 0 && c.start == c.end)
    });
}
