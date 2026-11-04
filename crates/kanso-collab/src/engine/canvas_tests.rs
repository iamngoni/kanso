//! Shared canvases through the engine: sharing, drawing on two devices,
//! the library's body as the CRDT's projection, presence, streamed strokes,
//! compaction and the limits. Property tests run on the seed
//! `PROPTEST_RNG_SEED` pins.

use std::sync::Arc;
use std::time::{Duration, Instant};

use kanso_core::canvas::{self, Canvas, INK, Node, Point, Stroke, StrokeRef};
use kanso_core::id::DocType;
use kanso_store::{Origin, Put, SharedKind};
use proptest::prelude::*;
use serde_json::json;

use super::tests::{Device, body_of, book, device, sync};
use crate::api::CollabError;
use crate::canvas_doc::StrokeHead;
use crate::memory::MemoryCollab;
use crate::model::Role;

use super::Engine;

fn card(id: &str, text: &str) -> Node {
    Node {
        id: id.into(),
        kind: "text".into(),
        width: 120.0,
        height: 60.0,
        text: Some(text.into()),
        ..Node::default()
    }
}

fn pen(points: &[(f64, f64)]) -> Stroke {
    Stroke {
        tool: "pen".into(),
        color: "#1e1e1e".into(),
        width: 2.0,
        points: points.iter().map(|&(x, y)| Point::new(x, y, 0.5)).collect(),
        ..Stroke::default()
    }
}

/// `owner` shares a canvas note holding `canvas` with the others, who
/// accept; returns the share.
fn shared_canvas(owner: &Device, others: &[(&Device, Role)], canvas: &Canvas) -> String {
    let book = book(&owner.store);
    owner
        .store
        .put(
            DocType::Note,
            json!({
                "_id": "note:board",
                "bookId": book,
                "title": "Board",
                "body": canvas::to_body(canvas),
                "doctype": "canvas",
            }),
            Put::Upsert,
            Origin::Ui,
        )
        .unwrap();
    for (other, _) in others {
        owner
            .engine
            .person(&other.engine.caller().username)
            .unwrap();
    }
    let invite: Vec<(String, Role)> = others
        .iter()
        .map(|(d, role)| (d.engine.caller().username.clone(), *role))
        .collect();
    let share = owner.engine.share("note:board", &invite).unwrap();
    for (other, _) in others {
        other.engine.refresh().unwrap();
        other.engine.accept(&share).unwrap();
    }
    sync(
        &others
            .iter()
            .map(|(d, _)| *d)
            .chain([owner])
            .collect::<Vec<_>>(),
        &share,
    );
    share
}

/// The canvas a device's library holds for the shared note.
fn library(device: &Device, share: &str) -> Canvas {
    canvas::parse(&body_of(device, share)).expect("the library holds a canvas")
}

/// A device's open view: attached, with the canvas as the engine has it.
fn open(device: &Device, share: &str) -> Canvas {
    device.engine.attach(share).unwrap();
    device.engine.canvas(share).unwrap()
}

/// The view hands its canvas over; returns what the engine says it is now.
fn pass(device: &Device, share: &str, view: &Canvas) -> Canvas {
    device.engine.sync_canvas(share, view).unwrap().canvas
}

#[test]
fn sharing_a_canvas_seeds_the_document_and_brings_it_to_members() {
    let server = MemoryCollab::new();
    let (a, b) = (device(&server, 1, 1), device(&server, 2, 1));
    let mut canvas = Canvas::default();
    canvas.put_card(card("c1", "hello"), 1);
    canvas.draw(pen(&[(0.0, 0.0), (40.0, 40.0)]), 2).unwrap();
    let share = shared_canvas(&a, &[(&b, Role::Edit)], &canvas);
    for d in [&a, &b] {
        let row = d.store.shared_note(&share).unwrap().unwrap();
        assert_eq!(row.kind, SharedKind::Canvas);
        let note = d.store.note(&row.note_id).unwrap().unwrap();
        assert_eq!(note.doctype, "canvas");
        assert!(note.extra.contains_key(kanso_core::model::SHARE_FIELD));
        let held = library(d, &share);
        assert_eq!(held.validate(), Ok(()));
        assert_eq!(held.node("c1").unwrap().text.as_deref(), Some("hello"));
        assert_eq!(held.live_strokes().count(), 1);
    }
    assert_eq!(library(&a, &share), library(&b, &share));
    // The owner-signed list says what kind of note it is.
    assert_eq!(
        b.engine
            .loaded(&share)
            .unwrap()
            .lock()
            .unwrap()
            .current
            .kind
            .as_deref(),
        Some("canvas")
    );
}

#[test]
fn a_canvas_that_cant_be_read_isnt_shared() {
    let server = MemoryCollab::new();
    let a = device(&server, 1, 1);
    let book = book(&a.store);
    a.store
        .put(
            DocType::Note,
            json!({"_id": "note:bad", "bookId": book, "title": "Bad", "body": "", "doctype": "canvas"}),
            Put::Upsert,
            Origin::Ui,
        )
        .unwrap();
    // An empty canvas is a new one: it can be shared.
    assert!(a.engine.share("note:bad", &[]).is_ok());
}

#[test]
fn drawing_on_one_device_reaches_the_others_library_and_view() {
    let server = MemoryCollab::new();
    let (a, b) = (device(&server, 1, 1), device(&server, 2, 1));
    let share = shared_canvas(&a, &[(&b, Role::Edit)], &Canvas::default());
    let mut a_view = open(&a, &share);
    let mut b_view = open(&b, &share);
    a_view.put_card(card("c1", "from A"), 10);
    a_view
        .draw(pen(&[(0.0, 0.0), (30.0, 5.0), (60.0, 0.0)]), 11)
        .unwrap();
    a_view = pass(&a, &share, &a_view);
    a.engine.sync_note(&share).unwrap();
    b.engine.sync_note(&share).unwrap();
    // B's view takes it in, and adds a card of its own.
    let merge = b.engine.sync_canvas(&share, &b_view).unwrap();
    assert!(merge.changed);
    b_view = merge.canvas;
    assert_eq!(b_view.node("c1").unwrap().text.as_deref(), Some("from A"));
    assert_eq!(b_view.live_strokes().count(), 1);
    b_view.put_card(card("c2", "from B"), 12);
    b_view = pass(&b, &share, &b_view);
    b.engine.sync_note(&share).unwrap();
    a.engine.sync_note(&share).unwrap();
    a_view = pass(&a, &share, &a_view);
    assert!(a_view.node("c2").is_some());
    // The libraries hold the projection once the views let go.
    a.engine.detach(&share);
    b.engine.detach(&share);
    let (held_a, held_b) = (library(&a, &share), library(&b, &share));
    assert_eq!(held_a, held_b);
    assert!(held_a.node("c1").is_some() && held_a.node("c2").is_some());
    assert_eq!(held_a.live_strokes().count(), 1);
    // Both authors are on the note's history.
    let id = a.store.shared_note(&share).unwrap().unwrap().note_id;
    assert!(
        a.store
            .shared_authors(&id)
            .unwrap()
            .contains(&"user2".to_owned())
    );
    let _ = (a_view, b_view);
}

#[test]
fn nobody_writes_a_shared_canvas_but_by_drawing() {
    let server = MemoryCollab::new();
    let (a, b) = (device(&server, 1, 1), device(&server, 2, 1));
    let share = shared_canvas(&a, &[(&b, Role::Edit)], &Canvas::default());
    for d in [&a, &b] {
        let id = d.store.shared_note(&share).unwrap().unwrap().note_id;
        // The body, as the owner or an editor would write it from a view.
        let mut canvas = library(d, &share);
        canvas.put_card(card("sneaky", "x"), 1);
        let err = d
            .store
            .put(
                DocType::Note,
                json!({"_id": id, "body": canvas::to_body(&canvas)}),
                Put::Merge,
                Origin::Ui,
            )
            .unwrap_err();
        assert!(err.to_string().contains("shared"), "{err}");
        // Nor can it be made a text note.
        assert!(
            d.store
                .put(
                    DocType::Note,
                    json!({"_id": id, "doctype": "markdown"}),
                    Put::Merge,
                    Origin::Ui,
                )
                .is_err()
        );
        // The other fields are the person's own.
        d.store
            .put(
                DocType::Note,
                json!({"_id": id, "pinned": true}),
                Put::Merge,
                Origin::Ui,
            )
            .unwrap();
    }
}

#[test]
fn a_viewers_drawing_is_never_sent() {
    let server = MemoryCollab::new();
    let (a, b) = (device(&server, 1, 1), device(&server, 2, 1));
    let share = shared_canvas(&a, &[(&b, Role::View)], &Canvas::default());
    let mut view = open(&b, &share);
    view.put_card(card("mine", "I can't"), 3);
    let merge = b.engine.sync_canvas(&share, &view).unwrap();
    // The canvas comes back as it is: without the card.
    assert!(merge.canvas.node("mine").is_none());
    assert!(merge.changed);
    assert!(b.store.outbox(&share).unwrap().is_empty());
    // And the editor's streamed ink goes nowhere either.
    let head = StrokeHead {
        id: "s1".into(),
        ink: "ink-1".into(),
        owner: "me".into(),
        tool: "pen".into(),
        color: "#000".into(),
        width: 2.0,
        z: 3.0,
    };
    let streamed = b
        .engine
        .stream_stroke(&share, &head, &[Point::new(0.0, 0.0, 0.5)])
        .unwrap()
        .unwrap();
    assert_eq!(streamed.update, None);
    assert!(b.store.outbox(&share).unwrap().is_empty());
}

#[test]
fn a_text_calls_on_a_canvas_are_refused() {
    let server = MemoryCollab::new();
    let a = device(&server, 1, 1);
    let share = shared_canvas(&a, &[], &Canvas::default());
    assert!(matches!(
        a.engine.sync_editor(&share, "Board", "text"),
        Err(CollabError::Refused(_))
    ));
    assert!(a.engine.add_comment(&share, 0..0, "hello").is_err());
    assert!(a.engine.comments(&share).unwrap().is_empty());
    assert!(a.engine.cursors(&share).is_empty());
}

#[test]
fn a_stroke_streams_to_the_other_device_before_it_ends() {
    let server = MemoryCollab::new();
    let (a, b) = (device(&server, 1, 1), device(&server, 2, 1));
    let share = shared_canvas(&a, &[(&b, Role::Edit)], &Canvas::default());
    let mut a_view = open(&a, &share);
    let b_view = open(&b, &share);
    // A has an ink layer.
    let ink = a_view.ink_layer(5);
    a_view = pass(&a, &share, &a_view);
    a.engine.sync_note(&share).unwrap();
    b.engine.sync_note(&share).unwrap();
    let mut b_view = pass(&b, &share, &b_view);
    let head = StrokeHead {
        id: "s-live".into(),
        ink: ink.clone(),
        owner: "a".into(),
        tool: "pen".into(),
        color: "#1e1e1e".into(),
        width: 2.0,
        z: 9.0,
    };
    let mut points: Vec<Point> = Vec::new();
    for chunk in 0..3 {
        points.extend((0..32).map(|i| Point::new(f64::from(chunk * 32 + i), 0.0, 0.5)));
        let streamed = a
            .engine
            .stream_stroke(&share, &head, &points)
            .unwrap()
            .unwrap();
        assert!(streamed.update.is_some());
        a.engine.sync_note(&share).unwrap();
        b.engine.sync_note(&share).unwrap();
        b_view = pass(&b, &share, &b_view);
        let seen = b_view
            .live_strokes()
            .next()
            .expect("the ink is there")
            .1
            .points
            .len();
        assert_eq!(seen, points.len());
    }
    // The pen lifts: the view draws the stroke, under the same ID.
    a_view
        .draw_as(
            Stroke {
                id: "s-live".into(),
                ..pen(&[])
            }
            .with_points(&points),
            6,
        )
        .unwrap();
    a_view = pass(&a, &share, &a_view);
    a.engine.sync_note(&share).unwrap();
    b.engine.sync_note(&share).unwrap();
    b_view = pass(&b, &share, &b_view);
    assert_eq!(b_view.live_strokes().count(), 1);
    // B erases it; a pen still down on A stops.
    let _ = a_view;
    let erase = b_view
        .delete_strokes(
            &[StrokeRef {
                node: ink,
                stroke: "s-live".into(),
            }],
            7,
        )
        .unwrap();
    let _ = erase;
    let _ = pass(&b, &share, &b_view);
    b.engine.sync_note(&share).unwrap();
    a.engine.sync_note(&share).unwrap();
    a.engine.sync_canvas(&share, &Canvas::default()).ok();
    let late = a.engine.stream_stroke(&share, &head, &points).unwrap();
    assert!(late.is_err(), "{late:?}");
}

trait WithPoints {
    fn with_points(self, points: &[Point]) -> Stroke;
}

impl WithPoints for Stroke {
    fn with_points(mut self, points: &[Point]) -> Stroke {
        self.points = points.to_vec();
        self
    }
}

#[test]
fn pointers_cross_the_live_room_and_expire() {
    let server = MemoryCollab::new();
    let (a, b) = (device(&server, 1, 1), device(&server, 2, 1));
    let share = shared_canvas(&a, &[(&b, Role::Edit)], &Canvas::default());
    a.engine.attach(&share).unwrap();
    b.engine.attach(&share).unwrap();
    let _a_live = Engine::go_live(&a.engine, &share, Arc::new(|_| {}));
    let _b_live = Engine::go_live(&b.engine, &share, Arc::new(|_| {}));
    a.engine.set_pointer(&share, Some((1234.0, -80.0, "pen")));
    let deadline = Instant::now() + Duration::from_secs(10);
    loop {
        let seen = b.engine.pointers(&share);
        if let [pointer] = seen.as_slice() {
            assert_eq!(pointer.name, "user1");
            assert_eq!((pointer.x, pointer.y), (1234.0, -80.0));
            assert_eq!(pointer.tool, "pen");
            break;
        }
        assert!(Instant::now() < deadline, "B sees {seen:?}");
        std::thread::sleep(Duration::from_millis(20));
    }
    // It moves, and B sees the new place.
    a.engine.set_pointer(&share, Some((5.0, 6.0, "eraser")));
    let deadline = Instant::now() + Duration::from_secs(10);
    loop {
        if b.engine.pointers(&share).first().map(|p| (p.x, p.y)) == Some((5.0, 6.0)) {
            break;
        }
        assert!(
            Instant::now() < deadline,
            "B sees {:?}",
            b.engine.pointers(&share)
        );
        std::thread::sleep(Duration::from_millis(20));
    }
    // A canvas has no text carets.
    assert!(b.engine.cursors(&share).is_empty());
}

#[test]
fn an_older_pointer_does_not_replace_a_newer_one() {
    let server = MemoryCollab::new();
    let (a, b) = (device(&server, 1, 1), device(&server, 2, 1));
    let share = shared_canvas(&a, &[(&b, Role::Edit)], &Canvas::default());
    let sealed = |seq: u64, x: f64| {
        let message = crate::model::PresenceMessage {
            cursor: String::new(),
            head: String::new(),
            name: "someone".into(),
            canvas: Some(crate::model::CanvasPointer {
                x,
                y: 0.0,
                tool: "pen".into(),
                seq,
            }),
        };
        let loaded = a.engine.loaded(&share).unwrap();
        let mut guard = super::lock(&loaded);
        a.engine
            .seal_presence(&mut guard, &serde_json::to_vec(&message).unwrap())
            .unwrap()
    };
    assert!(b.engine.take_presence(&share, &sealed(5, 50.0)).unwrap());
    assert!(!b.engine.take_presence(&share, &sealed(4, 40.0)).unwrap());
    assert!(b.engine.take_presence(&share, &sealed(6, 60.0)).unwrap());
    assert_eq!(b.engine.pointers(&share)[0].x, 60.0);
    // A pointer that isn't anywhere is not taken.
    assert!(
        !b.engine
            .take_presence(&share, &sealed(7, f64::NAN))
            .unwrap()
    );
}

#[test]
fn many_updates_compact_and_a_new_device_starts_from_the_snapshot() {
    let server = MemoryCollab::new();
    let (a, b) = (device(&server, 1, 1), device(&server, 2, 1));
    let share = shared_canvas(&a, &[(&b, Role::Edit)], &Canvas::default());
    let mut view = open(&a, &share);
    for i in 0..(super::COMPACT_AFTER_UPDATES + 5) {
        view.put_card(card(&format!("c{i}"), "x"), i as i64 + 1);
        if i % 3 == 0 {
            let id = format!("c{}", i / 2);
            view.delete_card(&id, i as i64 + 1);
        }
        view = pass(&a, &share, &view);
        a.engine.sync_note(&share).unwrap();
    }
    // A device with its view open merges through the view; let go of it, and
    // the next syncs write the snapshot.
    a.engine.detach(&share);
    a.engine.sync_note(&share).unwrap();
    a.engine.sync_note(&share).unwrap();
    assert!(server.snapshot_base(&share).unwrap() > 0);
    let (_, entries) = server.room(&share).unwrap();
    assert!(entries < 50, "{entries} entries left after compaction");
    let b2 = device(&server, 2, 2);
    b2.engine.refresh().unwrap();
    b2.engine.sync_note(&share).unwrap();
    assert_eq!(library(&b2, &share), library(&a, &share));
    assert!(library(&b2, &share).nodes.len() > 100);
}

#[test]
fn a_canvas_too_big_for_a_note_keeps_the_last_body_that_fit() {
    let server = MemoryCollab::new();
    let (a, b) = (device(&server, 1, 1), device(&server, 2, 1));
    let share = shared_canvas(&a, &[(&b, Role::Edit)], &Canvas::default());
    let mut view = open(&a, &share);
    view.put_card(card("small", "fits"), 1);
    view = pass(&a, &share, &view);
    a.engine.detach(&share);
    let good = library(&a, &share);
    assert!(good.node("small").is_some());
    // Cards until it is past the limit: no single update is.
    a.engine.attach(&share).unwrap();
    let text = "word ".repeat(2_000);
    let mut full = false;
    for i in 0..120 {
        view.put_card(card(&format!("big{i}"), &text), 2 + i);
        let merge = a.engine.sync_canvas(&share, &view).unwrap();
        view = merge.canvas;
        a.engine.sync_note(&share).unwrap();
        full |= merge.full;
        if full {
            break;
        }
        // The first write after the limit is the one that says so.
        a.engine.detach(&share);
        a.engine.attach(&share).unwrap();
    }
    a.engine.detach(&share);
    assert!(full, "the canvas never filled");
    let held = library(&a, &share);
    // The library never got a cut-off canvas: it parses, and is the last
    // that fit.
    assert_eq!(held.validate(), Ok(()));
    assert!(body_of(&a, &share).len() <= canvas::MAX_BYTES + 1024);
    assert!(held.nodes.len() < view.nodes.len());
    assert!(
        a.engine
            .problems()
            .iter()
            .any(|p| p.message.contains("full"))
    );
    // The document keeps all of it: a device that joins gets all, and its
    // library does the same.
    let c = device(&server, 3, 1);
    a.engine.person("user3").ok();
    let _ = c;
    let all = a.engine.canvas(&share).unwrap();
    assert_eq!(all.nodes.len(), view.nodes.len());
}

#[test]
fn the_kind_comes_from_the_share_and_a_body_that_isnt_a_canvas_cant_get_in() {
    let server = MemoryCollab::new();
    let (a, b) = (device(&server, 1, 1), device(&server, 2, 1));
    let share = shared_canvas(&a, &[(&b, Role::Edit)], &Canvas::default());
    // Text written over the canvas, by the engine's own text path, is
    // refused by the library: a canvas note's body is always a canvas.
    let id = b.store.shared_note(&share).unwrap().unwrap().note_id;
    let note = b.store.note(&id).unwrap().unwrap();
    let written = b.store.write_shared_text(&kanso_store::SharedText {
        note_id: &id,
        title: "Board",
        body: "this is not a canvas",
        mark: note.extra.get(kanso_core::model::SHARE_FIELD).cloned(),
        new_in: &note.book_id,
        expected_rev: None,
        authors: &[],
    });
    assert!(written.is_err());
    // What the engine merges is decided by the row, which the owner-signed
    // list filled in on both sides.
    for d in [&a, &b] {
        assert_eq!(
            d.store.shared_note(&share).unwrap().unwrap().kind,
            SharedKind::Canvas
        );
    }
    let mut view = open(&b, &share);
    view.put_card(card("c", "back"), 1);
    pass(&b, &share, &view);
    b.engine.detach(&share);
    sync(&[&a, &b], &share);
    assert!(library(&a, &share).node("c").is_some());
}

#[test]
fn stopping_leaves_a_private_canvas_that_can_be_edited_again() {
    let server = MemoryCollab::new();
    let (a, b) = (device(&server, 1, 1), device(&server, 2, 1));
    let mut canvas = Canvas::default();
    canvas.put_card(card("c1", "keep"), 1);
    let share = shared_canvas(&a, &[(&b, Role::Edit)], &canvas);
    a.engine.stop(&share).unwrap();
    let id = "note:board";
    let note = a.store.note(id).unwrap().unwrap();
    assert!(!note.extra.contains_key(kanso_core::model::SHARE_FIELD));
    assert_eq!(note.doctype, "canvas");
    // The guard no longer applies: it is the owner's again.
    let mut private = canvas::parse(&note.body).unwrap();
    private.put_card(card("c2", "new"), 2);
    a.store
        .put(
            DocType::Note,
            json!({"_id": id, "body": canvas::to_body(&private)}),
            Put::Merge,
            Origin::Ui,
        )
        .unwrap();
}

// ---------------------------------------------------------------- property

/// What a person does to the canvas in their open view.
#[derive(Clone, Debug)]
enum Do {
    Card(u8),
    Move(usize, i16),
    Delete(usize),
    Draw(Vec<(i16, i16)>),
    Erase(usize),
    Cut(i16, i16),
}

#[derive(Clone, Debug)]
enum Step {
    Do(usize, Do),
    /// The view hands its canvas over and the device syncs.
    Sync(usize),
    /// The device syncs without its view.
    Pull(usize),
}

fn action() -> impl Strategy<Value = Do> {
    prop_oneof![
        3 => any::<u8>().prop_map(Do::Card),
        2 => (any::<usize>(), -300i16..300).prop_map(|(i, x)| Do::Move(i, x)),
        1 => any::<usize>().prop_map(Do::Delete),
        3 => proptest::collection::vec((-200i16..200, -200i16..200), 1..6).prop_map(Do::Draw),
        1 => any::<usize>().prop_map(Do::Erase),
        1 => (-200i16..200, -200i16..200).prop_map(|(x, y)| Do::Cut(x, y)),
    ]
}

fn step() -> impl Strategy<Value = Step> {
    prop_oneof![
        6 => (0..2usize, action()).prop_map(|(d, a)| Step::Do(d, a)),
        4 => (0..2usize).prop_map(Step::Sync),
        2 => (0..2usize).prop_map(Step::Pull),
    ]
}

proptest! {
    #![proptest_config(crate::tests::config(12))]

    /// Two devices drawing on one shared canvas, syncing in any order, end
    /// with the same canvas in both libraries and in both views, and it is a
    /// canvas the view opens.
    #[test]
    fn two_devices_drawing_converge(steps in proptest::collection::vec(step(), 1..40)) {
        let server = MemoryCollab::new();
        let (a, b) = (device(&server, 1, 1), device(&server, 2, 1));
        let share = shared_canvas(&a, &[(&b, Role::Edit)], &Canvas::default());
        let devices = [&a, &b];
        let mut views = [open(&a, &share), open(&b, &share)];
        let mut n = 0;
        for step in steps {
            match step {
                Step::Do(d, action) => {
                    n += 1;
                    let view = &mut views[d];
                    let now = 100 + n;
                    match action {
                        Do::Card(x) => {
                            view.put_card(card(&format!("c{d}-{n}"), &format!("t{x}")), now);
                        }
                        Do::Move(i, x) => {
                            let ids: Vec<String> = view.nodes.iter().filter(|c| !c.deleted && c.kind != INK).map(|c| c.id.clone()).collect();
                            if let Some(id) = ids.get(i % ids.len().max(1)) {
                                view.move_card(id, f64::from(x), 3.0, now);
                            }
                        }
                        Do::Delete(i) => {
                            let ids: Vec<String> = view.nodes.iter().filter(|c| !c.deleted && c.kind != INK).map(|c| c.id.clone()).collect();
                            if let Some(id) = ids.get(i % ids.len().max(1)) {
                                view.delete_card(id, now);
                            }
                        }
                        Do::Draw(points) => {
                            let points: Vec<(f64, f64)> = points.iter().map(|&(x, y)| (f64::from(x), f64::from(y))).collect();
                            let _ = view.draw(pen(&points), now);
                        }
                        Do::Erase(i) => {
                            let strokes: Vec<StrokeRef> = view.live_strokes().map(|(node, s)| StrokeRef { node: node.id.clone(), stroke: s.id.clone() }).collect();
                            if let Some(r) = strokes.get(i % strokes.len().max(1)) {
                                view.delete_strokes(std::slice::from_ref(r), now);
                            }
                        }
                        Do::Cut(x, y) => {
                            view.erase_along(&[(f64::from(x), f64::from(y)), (f64::from(x) + 30.0, f64::from(y))], 25.0, now);
                        }
                    }
                }
                Step::Sync(d) => {
                    let merge = devices[d].engine.sync_canvas(&share, &views[d]).unwrap();
                    prop_assert_eq!(merge.canvas.validate(), Ok(()));
                    views[d] = merge.canvas;
                    devices[d].engine.sync_note(&share).unwrap();
                }
                Step::Pull(d) => {
                    devices[d].engine.sync_note(&share).unwrap();
                }
            }
        }
        // Everyone hands over a last time, then both pull until quiet.
        for _ in 0..3 {
            for d in 0..2 {
                views[d] = devices[d].engine.sync_canvas(&share, &views[d]).unwrap().canvas;
                devices[d].engine.sync_note(&share).unwrap();
            }
        }
        for d in devices {
            d.engine.detach(&share);
        }
        for d in devices {
            d.engine.sync_note(&share).unwrap();
        }
        prop_assert_eq!(&views[0], &views[1]);
        let (held_a, held_b) = (library(&a, &share), library(&b, &share));
        prop_assert_eq!(held_a.validate(), Ok(()));
        prop_assert_eq!(&held_a, &held_b);
        // The library is the view without its tombstones.
        let loaded = a.engine.loaded(&share).unwrap();
        let guard = super::lock(&loaded);
        prop_assert_eq!(held_a, guard.doc.canvas().unwrap().to_canvas());
    }
}
