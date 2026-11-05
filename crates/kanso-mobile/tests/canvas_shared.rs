//! Shared canvases on the phone (#45 step 7): opening one in its room,
//! drawing on it with another phone, streaming a stroke and taking it back,
//! pointers, viewers, and what stopping or leaving keeps. Two libraries on
//! one in-memory sync server and one in-memory shared-note server, with real
//! crypto, as `sharing.rs` has them.

mod common;

use std::sync::Arc;
use std::time::{Duration, Instant};

use kanso_collab::memory::MemoryCollab;
use kanso_mobile::{
    CanvasPulse, CanvasSession, CanvasView, InkStyle, KansoError, Library, MemorySecrets,
    PenSample, ShareRole, SharedNotes,
};
use kanso_sync::{DEFAULT_SERVER, DeviceIdentity, MemoryServer, Password};
use proptest::prelude::*;

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
        library.create_notebook("Work".into(), None).unwrap();
        Phone { _dir: dir, library }
    }
}

impl Phone {
    fn shared(&self) -> SharedNotes {
        self.library.shared()
    }
}

fn eventually(what: &str, mut check: impl FnMut() -> bool) {
    let until = Instant::now() + Duration::from_secs(15);
    while Instant::now() < until {
        if check() {
            return;
        }
        std::thread::sleep(Duration::from_millis(25));
    }
    panic!("never happened: {what}");
}

fn pen() -> InkStyle {
    InkStyle {
        tool: "pen".into(),
        color: "#e03131".into(),
        width: 3.0,
    }
}

/// A horizontal stroke at height `y`.
fn line(y: f64) -> Vec<PenSample> {
    (0..20)
        .map(|i| PenSample {
            x: f64::from(i) * 5.0,
            y,
            pressure: 0.6,
            ..PenSample::default()
        })
        .collect()
}

fn ids(view: &CanvasView) -> Vec<String> {
    let mut ids: Vec<_> = view.strokes.iter().map(|s| s.id.clone()).collect();
    ids.sort();
    ids
}

/// Alice's canvas "Board", shared with Bob as `role` and accepted. Returns
/// (world, alice, bob, alice's note ID, bob's note ID).
fn shared_canvas(role: ShareRole) -> (World, Phone, Phone, String, String) {
    let world = world();
    let alice = world.phone("alice");
    let bob = world.phone("bob");
    bob.shared().sync_shared().unwrap();
    let id = alice
        .library
        .create_canvas(None, None, "Board".into())
        .unwrap()
        .id;
    alice
        .shared()
        .share(id.clone(), vec![("bob".into(), role)])
        .unwrap();
    let invitation = bob.shared().invitations().unwrap().remove(0);
    let bobs = bob.shared().accept(invitation.share_id).unwrap();
    (world, alice, bob, id, bobs)
}

/// Takes the room's news until `check` holds on the canvas the session has.
fn pulse_until(
    session: &CanvasSession,
    what: &str,
    mut check: impl FnMut(&CanvasView) -> bool,
) -> CanvasView {
    let mut last = session.view(vec![]).unwrap();
    eventually(what, || {
        if let Some(view) = session.pulse(vec![]).unwrap().view {
            last = view;
        }
        check(&last)
    });
    last
}

// no-prop-test: a walk through opening a shared canvas on two phones; the
// engine's convergence properties are in kanso-collab, and the phones' own in
// `prop_two_phones_drawing_converge` below.
#[test]
fn a_shared_canvas_opens_in_its_room_and_strokes_cross_both_ways() {
    let (_world, alice, bob, id, bobs) = shared_canvas(ShareRole::Editor);
    let a = alice.library.canvas(&id).unwrap();
    let b = bob.library.canvas(&bobs).unwrap();
    let view = a.view(vec![]).unwrap();
    assert!(view.shared && !view.read_only);
    assert!(alice.shared().is_live(&id), "opening it opens the room");

    let drawn = a.draw(pen(), line(10.0), vec![]).unwrap();
    // Alice's save hands the room what she drew; Bob's view does not change
    // until his canvas takes the room's news.
    let saved = a.save().unwrap();
    assert!(!saved.merged);
    assert!(b.view(vec![]).unwrap().strokes.is_empty());
    let seen = pulse_until(&b, "Bob sees Alice's stroke", |v| v.strokes.len() == 1);
    assert_eq!(ids(&seen), ids(&drawn));
    assert!(seen.shared && !seen.read_only);

    let theirs = b.draw(pen(), line(40.0), vec![]).unwrap();
    b.save().unwrap();
    let both = pulse_until(&a, "Alice sees Bob's stroke", |v| v.strokes.len() == 2);
    assert!(ids(&both).contains(&theirs.strokes[0].id));

    // The library's body follows the room: a note that lists, searches and
    // exports reads the canvas.
    eventually("the library's body has both strokes", || {
        alice
            .library
            .note(&id)
            .unwrap()
            .body
            .is_some_and(|b| b.matches("\"id\":\"s").count() >= 2)
    });
}

// no-prop-test: the roles of one share; the role table has its own tests.
#[test]
fn a_viewer_sees_the_canvas_and_cannot_draw_on_it() {
    let (_world, alice, bob, id, bobs) = shared_canvas(ShareRole::Viewer);
    let a = alice.library.canvas(&id).unwrap();
    let b = bob.library.canvas(&bobs).unwrap();
    let view = b.view(vec![]).unwrap();
    assert!(view.shared && view.read_only);

    a.draw(pen(), line(0.0), vec![]).unwrap();
    a.save().unwrap();
    pulse_until(&b, "the viewer sees the stroke", |v| v.strokes.len() == 1);

    let refused = |r: Result<CanvasView, KansoError>| {
        assert!(matches!(r, Err(KansoError::Invalid { .. })), "{r:?}")
    };
    refused(b.draw(pen(), line(5.0), vec![]));
    refused(b.erase_at(0.0, 0.0, 30.0, vec![]));
    refused(b.add_text_card(0.0, 0.0, "x".into(), vec![]));
    refused(b.undo(vec![]));
    assert!(b.view(vec![]).unwrap().strokes.len() == 1);
    // Streaming sends nothing for a viewer, and saving writes nothing.
    assert!(matches!(
        b.stream_stroke(pen(), line(9.0)),
        Err(KansoError::Invalid { .. })
    ));
    assert!(!b.save().unwrap().merged);
    // Alice's canvas has only her stroke, whatever Bob tried.
    std::thread::sleep(Duration::from_millis(400));
    a.save().unwrap();
    assert_eq!(a.view(vec![]).unwrap().strokes.len(), 1);
}

// no-prop-test: presence over a live room, waiting for it to arrive.
#[test]
fn the_others_pointers_are_seen_with_a_name_and_a_colour_and_go_when_they_leave() {
    let (_world, alice, bob, id, bobs) = shared_canvas(ShareRole::Editor);
    let a = alice.library.canvas(&id).unwrap();
    let b = bob.library.canvas(&bobs).unwrap();
    assert!(a.pointers().is_empty() && b.pointers().is_empty());
    a.set_pointer(120.0, -40.0, "pen".into());
    eventually("Bob sees Alice's pointer", || {
        b.pulse(vec![]).unwrap().pointers.len() == 1
    });
    let pointer = b.pointers().remove(0);
    assert_eq!(pointer.name, "alice");
    assert_eq!((pointer.x, pointer.y), (120.0, -40.0));
    assert_eq!(pointer.tool, "pen");
    assert!(pointer.colour < 8);
    assert!(pointer.colour_light.starts_with('#') && pointer.colour_dark.starts_with('#'));
    assert_ne!(pointer.colour_light, pointer.colour_dark);
    // Her own pointer is not among hers.
    assert!(a.pointers().is_empty());

    a.set_pointer(f64::NAN, 0.0, "pen".into());
    a.clear_pointer();
    eventually("her pointer goes", || b.pointers().is_empty());
}

// no-prop-test: a pen's stroke is streamed, seen growing, and taken back.
#[test]
fn a_streamed_stroke_is_seen_growing_and_a_refused_one_is_taken_back() {
    let (_world, alice, bob, id, bobs) = shared_canvas(ShareRole::Editor);
    let a = alice.library.canvas(&id).unwrap();
    let b = bob.library.canvas(&bobs).unwrap();

    // A dot streams nothing.
    assert!(!a.stream_stroke(pen(), line(0.0)[..1].to_vec()).unwrap());
    // Half a stroke streams: Bob sees ink before the pen is up.
    assert!(!a.stream_stroke(pen(), line(0.0)[..8].to_vec()).unwrap());
    let half = pulse_until(&b, "Bob sees the stroke being drawn", |v| {
        v.strokes.len() == 1
    });
    let streamed = half.strokes[0].id.clone();
    // The finished stroke keeps the ID it was streamed under.
    assert!(!a.stream_stroke(pen(), line(0.0)).unwrap());
    let done = a.draw(pen(), line(0.0), vec![]).unwrap();
    assert_eq!(ids(&done), vec![streamed.clone()]);
    a.save().unwrap();
    pulse_until(&b, "still one stroke after pen-up", |v| {
        ids(v) == vec![streamed.clone()]
    });

    // A stroke the canvas refuses at pen-up (nothing usable in it) goes
    // from the others: no ghost.
    assert!(!a.stream_stroke(pen(), line(30.0)).unwrap());
    pulse_until(&b, "Bob sees the ghost to be", |v| v.strokes.len() == 2);
    let nothing = vec![PenSample {
        x: f64::NAN,
        y: f64::NAN,
        pressure: 0.5,
        ..PenSample::default()
    }];
    let after = a.draw(pen(), nothing, vec![]).unwrap();
    assert_eq!(after.strokes.len(), 1);
    a.save().unwrap();
    pulse_until(&b, "the refused stroke is withdrawn", |v| {
        ids(v) == vec![streamed.clone()]
    });

    // A pen the system took is taken back as well.
    assert!(!a.stream_stroke(pen(), line(60.0)).unwrap());
    pulse_until(&b, "Bob sees it", |v| v.strokes.len() == 2);
    a.cancel_stroke().unwrap();
    pulse_until(&b, "and then it is gone", |v| v.strokes.len() == 1);
}

// no-prop-test: someone erasing the stroke under the pen, once.
#[test]
fn a_stroke_erased_under_the_pen_stops_there() {
    let (_world, alice, bob, id, bobs) = shared_canvas(ShareRole::Editor);
    let a = alice.library.canvas(&id).unwrap();
    let b = bob.library.canvas(&bobs).unwrap();
    assert!(!a.stream_stroke(pen(), line(0.0)[..8].to_vec()).unwrap());
    pulse_until(&b, "Bob has the stroke", |v| v.strokes.len() == 1);
    b.erase_at(10.0, 0.0, 30.0, vec![]).unwrap();
    b.save().unwrap();
    // The erase comes in when the shell asks (after the gesture), and the
    // stroke is gone from her canvas ...
    let view = pulse_until(&a, "the erase reaches Alice", |v| v.strokes.is_empty());
    assert!(view.strokes.is_empty());
    // ... and the next chunk of the pen's stroke finds it erased and says
    // so: it stops there.
    assert!(a.stream_stroke(pen(), line(0.0)).unwrap());
}

// no-prop-test: a long text goes in pieces, one walk.
#[test]
fn a_cards_long_text_goes_out_in_pieces_and_arrives_whole() {
    let (_world, alice, bob, id, bobs) = shared_canvas(ShareRole::Editor);
    let a = alice.library.canvas(&id).unwrap();
    let b = bob.library.canvas(&bobs).unwrap();
    // Well past one update (64 KB parts, 256 KB at most) and under a note.
    let text: String = "word ".repeat(100_000);
    assert!(text.len() > 400_000);
    a.add_text_card(0.0, 0.0, text.clone(), vec![]).unwrap();
    a.save().unwrap();
    let seen = pulse_until(&b, "Bob has the card", |v| v.cards.len() == 1);
    assert_eq!(seen.cards[0].text.len(), text.len());
    assert_eq!(seen.cards[0].text, text);
}

// no-prop-test: stopping and leaving, once each.
#[test]
fn stopping_or_leaving_keeps_the_canvas_as_it_is_now_and_makes_it_private() {
    let (_world, alice, bob, id, bobs) = shared_canvas(ShareRole::Editor);
    let a = alice.library.canvas(&id).unwrap();
    let b = bob.library.canvas(&bobs).unwrap();
    let drawn = a.draw(pen(), line(0.0), vec![]).unwrap();
    a.save().unwrap();
    pulse_until(&b, "Bob has it", |v| v.strokes.len() == 1);

    // Bob draws a second stroke and leaves at once: the library's body
    // trails the room by up to two seconds, yet his copy has both.
    let second = b.draw(pen(), line(30.0), vec![]).unwrap();
    b.save().unwrap();
    pulse_until(&a, "Alice has Bob's stroke", |v| v.strokes.len() >= 2);
    bob.shared().leave(bobs.clone()).unwrap();
    let body = bob.library.note(&bobs).unwrap().body.unwrap();
    for stroke in &second.strokes {
        assert!(body.contains(&stroke.id), "his own stroke is kept");
    }
    assert!(body.contains(&drawn.strokes[0].id), "and Alice's");
    // His open canvas is a private one now, and takes drawing.
    let view = b.pulse(vec![]).unwrap().view.expect("it changed");
    assert!(!view.shared && !view.read_only);
    b.draw(pen(), line(60.0), vec![]).unwrap();
    b.save().unwrap();
    let kept = bob.library.canvas(&bobs).unwrap().view(vec![]).unwrap();
    assert_eq!(kept.strokes.len(), 3);
    assert!(!kept.shared);

    // Alice draws and stops sharing before the library has written it.
    a.draw(pen(), line(90.0), vec![]).unwrap();
    a.save().unwrap();
    alice.shared().stop(id.clone()).unwrap();
    let stopped = alice.library.canvas(&id).unwrap().view(vec![]).unwrap();
    assert!(!stopped.shared);
    assert!(stopped.strokes.len() >= 3, "everything drawn is there");
    // Her open session goes private too, with what it held.
    let view = a.pulse(vec![]).unwrap().view.expect("it changed");
    assert!(!view.shared && view.strokes.len() >= 3);
}

// no-prop-test: the store's guard against a plain write, once; its
// properties are in kanso-store.
#[test]
fn a_shared_canvas_refuses_a_plain_text_write() {
    let (_world, alice, _bob, id, _bobs) = shared_canvas(ShareRole::Editor);
    let a = alice.library.canvas(&id).unwrap();
    a.draw(pen(), line(0.0), vec![]).unwrap();
    a.save().unwrap();
    let rev = alice.library.note(&id).unwrap().rev;
    let refused = alice.library.save(&id, &rev, "not a canvas".into());
    assert!(refused.is_err(), "{refused:?}");
}

// no-prop-test: signed out, once.
#[test]
fn signed_out_a_shared_canvas_opens_to_look_at() {
    let (_world, alice, _bob, id, _bobs) = shared_canvas(ShareRole::Editor);
    let a = alice.library.canvas(&id).unwrap();
    a.draw(pen(), line(0.0), vec![]).unwrap();
    a.save().unwrap();
    drop(a);
    alice.library.sync().sign_out().unwrap();
    let view = alice.library.canvas(&id).unwrap();
    let shown = view.view(vec![]).unwrap();
    assert!(shown.shared && shown.read_only);
    assert!(matches!(
        view.draw(pen(), line(9.0), vec![]),
        Err(KansoError::Invalid { .. })
    ));
    assert_eq!(view.pulse(vec![]).unwrap(), CanvasPulse::default());
}

// no-prop-test: a private canvas has no room, once.
#[test]
fn a_private_canvas_has_no_room_and_nothing_to_pulse() {
    let world = world();
    let alice = world.phone("alice");
    let id = alice
        .library
        .create_canvas(None, None, "Mine".into())
        .unwrap()
        .id;
    let session = alice.library.canvas(&id).unwrap();
    let view = session.view(vec![]).unwrap();
    assert!(!view.shared && !view.read_only);
    assert_eq!(session.pulse(vec![]).unwrap(), CanvasPulse::default());
    assert!(!session.stream_stroke(pen(), line(0.0)).unwrap());
    session.set_pointer(1.0, 2.0, "pen".into());
    session.clear_pointer();
    session.cancel_stroke().unwrap();
    session.reconnect().unwrap();
    session.close().unwrap();
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(4))]

    /// Whatever the two phones draw, and in whatever order they save, both
    /// end up with the same strokes, every one of them drawn.
    #[test]
    fn prop_two_phones_drawing_converge(
        marks in proptest::collection::vec((any::<bool>(), 0u8..40), 1..7),
    ) {
        let (_world, alice, bob, id, bobs) = shared_canvas(ShareRole::Editor);
        let a = alice.library.canvas(&id).unwrap();
        let b = bob.library.canvas(&bobs).unwrap();
        let mut want = std::collections::BTreeSet::new();
        for (n, (by_alice, row)) in marks.iter().enumerate() {
            let (who, other) = if *by_alice { (&a, &b) } else { (&b, &a) };
            let y = f64::from(*row) * 20.0 + n as f64;
            let view = who.draw(pen(), line(y), vec![]).unwrap();
            who.save().unwrap();
            for s in &view.strokes {
                want.insert(s.id.clone());
            }
            // Sometimes the other side takes it in between.
            if n % 2 == 0 {
                other.pulse(vec![]).unwrap();
            }
        }
        let all: Vec<String> = want.into_iter().collect();
        for session in [&a, &b] {
            let view = pulse_until(session, "both have every stroke", |v| ids(v) == all);
            prop_assert_eq!(ids(&view), all.clone());
        }
    }
}
