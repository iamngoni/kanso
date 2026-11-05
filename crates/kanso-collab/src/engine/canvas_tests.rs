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

#[test]
fn stopping_or_leaving_keeps_what_was_drawn_just_before() {
    let server = MemoryCollab::new();
    let (a, b) = (device(&server, 1, 1), device(&server, 2, 1));
    let share = shared_canvas(&a, &[(&b, Role::Edit)], &Canvas::default());
    let mut a_view = open(&a, &share);
    let mut b_view = open(&b, &share);
    // Two edits in a row: the library is written for the first, and the
    // second is inside the two seconds between writes.
    a_view.put_card(card("first", "1"), 1);
    a_view = pass(&a, &share, &a_view);
    a_view.put_card(card("second", "2"), 2);
    let _ = pass(&a, &share, &a_view);
    b_view.put_card(card("b-first", "1"), 3);
    b_view = pass(&b, &share, &b_view);
    b_view.put_card(card("b-second", "2"), 4);
    let _ = pass(&b, &share, &b_view);
    // B leaves and A stops, both at once: each keeps its canvas as drawn.
    let b_id = b.store.shared_note(&share).unwrap().unwrap().note_id;
    b.engine.leave(&share).unwrap();
    a.engine.stop(&share).unwrap();
    let held = canvas::parse(&a.store.note("note:board").unwrap().unwrap().body).unwrap();
    assert!(held.node("second").is_some(), "{held:?}");
    let held = canvas::parse(&b.store.note(&b_id).unwrap().unwrap().body).unwrap();
    assert!(held.node("b-second").is_some(), "{held:?}");
}

#[test]
fn a_card_with_a_long_text_goes_in_parts_under_the_cap() {
    let server = MemoryCollab::new();
    let (a, b) = (device(&server, 1, 1), device(&server, 2, 1));
    let share = shared_canvas(&a, &[(&b, Role::Edit)], &Canvas::default());
    let mut view = open(&a, &share);
    let long = "a long paste, ".repeat(25_000);
    assert!(long.len() > crate::envelope::MAX_UPDATE);
    view.put_card(card("long", &long), 1);
    let view = pass(&a, &share, &view);
    sync(&[&a, &b], &share);
    sync(&[&a, &b], &share);
    assert!(a.engine.problems().is_empty(), "{:?}", a.engine.problems());
    let b_view = open(&b, &share);
    assert_eq!(
        b_view.node("long").and_then(|n| n.text.as_deref()),
        Some(long.as_str())
    );
    let _ = view;
}

#[test]
fn an_update_too_large_to_send_is_not_sent_again_after_the_restart() {
    let server = MemoryCollab::new();
    let (a, b) = (device(&server, 1, 1), device(&server, 2, 1));
    let share = shared_canvas(&a, &[(&b, Role::Edit)], &Canvas::default());
    let mut view = open(&a, &share);
    // One field no update can carry.
    let mut big = card("big", "x");
    big.extra.insert("huge".into(), json!("z".repeat(300_000)));
    view.put_card(big, 1);
    view = pass(&a, &share, &view);
    let _ = a.engine.sync_note(&share);
    // The view keeps drawing on what it has, then syncs a few times.
    for i in 0..3 {
        view.put_card(card(&format!("after{i}"), "ok"), 2 + i);
        view = pass(&a, &share, &view);
        let _ = a.engine.sync_note(&share);
    }
    let book = a.store.note("note:board").unwrap().unwrap().book_id;
    let copies = a
        .store
        .notes_in_books(&[book])
        .unwrap()
        .into_iter()
        .filter(|n| n.title.contains("(not shared)"))
        .count();
    assert_eq!(copies, 1, "the private copy is made once");
    sync(&[&a, &b], &share);
    let seen = open(&b, &share);
    assert!(seen.node("after2").is_some());
    assert!(seen.node("big").is_none());
}

#[test]
fn a_shared_text_note_cant_become_a_canvas() {
    let server = MemoryCollab::new();
    let a = device(&server, 1, 1);
    let book = book(&a.store);
    a.store
        .put(
            DocType::Note,
            json!({"_id": "note:text", "bookId": book, "title": "T", "body": "words"}),
            Put::Upsert,
            Origin::Ui,
        )
        .unwrap();
    let _share = a.engine.share("note:text", &[]).unwrap();
    let body = canvas::to_body(&Canvas::default());
    assert!(
        a.store
            .put(
                DocType::Note,
                json!({"_id": "note:text", "doctype": "canvas", "body": body}),
                Put::Merge,
                Origin::Api,
            )
            .is_err()
    );
    assert!(
        a.store
            .put(
                DocType::Note,
                json!({"_id": "note:text", "doctype": "canvas"}),
                Put::Merge,
                Origin::Ui,
            )
            .is_err()
    );
}

#[test]
fn a_streamed_stroke_the_pen_wont_finish_leaves_nothing_behind() {
    let server = MemoryCollab::new();
    let (a, b) = (device(&server, 1, 1), device(&server, 2, 1));
    let share = shared_canvas(&a, &[(&b, Role::Edit)], &Canvas::default());
    let mut a_view = open(&a, &share);
    let ink = a_view.ink_layer(5);
    a_view = pass(&a, &share, &a_view);
    let head = StrokeHead {
        id: "s-cut".into(),
        ink,
        owner: a.engine.caller().account_id.clone(),
        tool: "pen".into(),
        color: "#1e1e1e".into(),
        width: 2.0,
        z: 9.0,
    };
    let points: Vec<Point> = (0..40)
        .map(|i| Point::new(f64::from(i), 0.0, 0.5))
        .collect();
    a.engine
        .stream_stroke(&share, &head, &points)
        .unwrap()
        .unwrap();
    sync(&[&a, &b], &share);
    let b_view = open(&b, &share);
    assert_eq!(b_view.live_strokes().count(), 1, "B sees the ink coming");
    // The pen's stroke is refused when it ends (or the view closes under
    // it): A takes it back.
    a.engine.abandon_stroke(&share, "s-cut").unwrap();
    let a_view = pass(&a, &share, &a_view);
    assert_eq!(a_view.live_strokes().count(), 0);
    sync(&[&a, &b], &share);
    let b_view = pass(&b, &share, &b_view);
    assert_eq!(b_view.live_strokes().count(), 0, "no ghost on B");
    a.engine.detach(&share);
    b.engine.detach(&share);
    assert_eq!(library(&a, &share), library(&b, &share));
    assert_eq!(library(&b, &share).live_strokes().count(), 0);
    // A finished stroke is never taken back this way.
    let mut a_view = open(&a, &share);
    a_view.draw(pen(&[(0.0, 0.0), (30.0, 30.0)]), 7).unwrap();
    let a_view = pass(&a, &share, &a_view);
    let done = a_view.live_strokes().next().unwrap().1.id.clone();
    a.engine.abandon_stroke(&share, &done).unwrap();
    assert_eq!(pass(&a, &share, &a_view).live_strokes().count(), 1);
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

// ---------------------------------------------------- the local API's edits

#[test]
fn an_api_edit_reaches_the_library_and_the_other_device_without_a_view() {
    let server = MemoryCollab::new();
    let (a, b) = (device(&server, 1, 1), device(&server, 2, 1));
    let mut canvas = Canvas::default();
    canvas.put_card(card("c1", "hello"), 1);
    let share = shared_canvas(&a, &[(&b, Role::Edit)], &canvas);
    let made = a
        .engine
        .edit_canvas(&share, |live| {
            let mut next = live.clone();
            next.move_node("c1", 77.0, 88.0, 20);
            next.put_card(card("api1", "from the API"), 21);
            Ok((next, None, 2))
        })
        .unwrap();
    assert_eq!(made, 2);
    // The library holds it at once: the edit settles the projection.
    let held = library(&a, &share);
    assert_eq!(held.node("c1").unwrap().x, 77.0);
    assert!(held.node("api1").is_some());
    // The other device gets it as it gets a collaborator's.
    a.engine.sync_note(&share).unwrap();
    b.engine.sync_note(&share).unwrap();
    let view = open(&b, &share);
    assert_eq!(view.node("c1").unwrap().x, 77.0);
    assert_eq!(
        view.node("api1").unwrap().text.as_deref(),
        Some("from the API")
    );
}

#[test]
fn an_api_edit_leaves_an_open_views_own_drawing_alone() {
    let server = MemoryCollab::new();
    let a = device(&server, 1, 1);
    let share = shared_canvas(&a, &[], &Canvas::default());
    let mut view = open(&a, &share);
    // The person draws, and has not synced yet.
    view.put_card(card("mine", "typed"), 5);
    a.engine
        .edit_canvas(&share, |live| {
            let mut next = live.clone();
            next.put_card(card("api1", "agent"), 6);
            Ok((next, None, ()))
        })
        .unwrap();
    // The view's next sync keeps both.
    let merged = pass(&a, &share, &view);
    assert!(merged.node("mine").is_some() && merged.node("api1").is_some());
}

#[test]
fn an_api_edit_is_refused_for_a_viewer_a_full_canvas_and_a_refusing_change() {
    let server = MemoryCollab::new();
    let (a, b) = (device(&server, 1, 1), device(&server, 2, 1));
    let share = shared_canvas(&a, &[(&b, Role::View)], &Canvas::default());
    let err = b
        .engine
        .edit_canvas(&share, |live| Ok((live.clone(), None, ())))
        .unwrap_err();
    assert!(matches!(err, CollabError::Refused(_)), "{err:?}");
    let err = a
        .engine
        .edit_canvas::<()>(&share, |_| Err("not today".into()))
        .unwrap_err();
    assert!(err.to_string().contains("not today"), "{err}");
    let err = a
        .engine
        .edit_canvas(&share, |live| {
            let mut next = live.clone();
            next.put_card(card("huge", &"x".repeat(canvas::MAX_BYTES + 10)), 3);
            Ok((next, None, ()))
        })
        .unwrap_err();
    assert!(err.to_string().contains("full"), "{err}");
    assert!(library(&a, &share).node("huge").is_none());
    // A note that is not shared is not a room at all.
    assert!(
        a.engine
            .edit_canvas("share-none", |l| Ok((l.clone(), None, ())))
            .is_err()
    );
}

#[test]
fn an_api_edit_sees_what_arrived_while_a_view_is_open() {
    let server = MemoryCollab::new();
    let (a, b) = (device(&server, 1, 1), device(&server, 2, 1));
    let mut canvas = Canvas::default();
    canvas.put_card(card("c1", "hello"), 1);
    let share = shared_canvas(&a, &[(&b, Role::Edit)], &canvas);
    // A has the canvas open, so what arrives waits for its view or the API.
    let _a_view = open(&a, &share);
    let mut b_view = open(&b, &share);
    b_view.move_node("c1", 300.0, 300.0, 10);
    let _ = pass(&b, &share, &b_view);
    sync(&[&b], &share);
    a.engine.sync_note(&share).unwrap();
    let seen = a
        .engine
        .edit_canvas(&share, |live| {
            let node = live.node("c1").unwrap();
            Ok((live.clone(), None, (node.x, node.kanso_version)))
        })
        .unwrap();
    // The batch is checked against B's move, so a stale `ifVersion` is caught.
    assert_eq!(seen.0, 300.0);
    assert!(seen.1 > 1, "{seen:?}");
}

/// What `device` has waiting to send for `share`.
fn outbox(device: &Device, share: &str) -> Vec<Vec<u8>> {
    device
        .store
        .outbox(share)
        .unwrap()
        .into_iter()
        .map(|o| o.payload)
        .collect()
}

#[test]
fn an_api_edit_goes_to_the_room_as_one_update_title_and_all() {
    let server = MemoryCollab::new();
    let (a, b) = (device(&server, 1, 1), device(&server, 2, 1));
    let share = shared_canvas(&a, &[(&b, Role::Edit)], &Canvas::default());
    sync(&[&a, &b], &share);
    let before = outbox(&a, &share).len();
    // Well over one part's worth: the view's path would cut it in pieces.
    a.engine
        .edit_canvas(&share, |live| {
            let mut next = live.clone();
            for i in 0..120 {
                next.put_card(card(&format!("api{i}"), &"words ".repeat(170)), 20);
            }
            Ok((next, Some("Renamed by the API".into()), ()))
        })
        .unwrap();
    let sent = outbox(&a, &share);
    assert_eq!(sent.len(), before + 1, "one batch is one update");
    assert!(sent.last().unwrap().len() > super::UPDATE_PART);
    assert!(sent.last().unwrap().len() <= crate::envelope::MAX_UPDATE);
    // The other device gets all of it, title included.
    sync(&[&a, &b], &share);
    let view = open(&b, &share);
    assert_eq!(view.nodes.iter().filter(|n| !n.deleted).count(), 120);
    let b_id = b.store.shared_note(&share).unwrap().unwrap().note_id;
    assert_eq!(
        b.store.note(&b_id).unwrap().unwrap().title,
        "Renamed by the API"
    );
}

#[test]
fn an_api_edit_too_big_for_one_update_is_refused_and_nothing_is_sent() {
    let server = MemoryCollab::new();
    let a = device(&server, 1, 1);
    let share = shared_canvas(&a, &[], &Canvas::default());
    sync(&[&a], &share);
    let before = outbox(&a, &share).len();
    let held = library(&a, &share);
    let err = a
        .engine
        .edit_canvas(&share, |live| {
            let mut next = live.clone();
            for i in 0..400 {
                next.put_card(card(&format!("api{i}"), &"words ".repeat(170)), 20);
            }
            Ok((next, None, ()))
        })
        .unwrap_err();
    assert!(err.to_string().contains("too big"), "{err}");
    assert_eq!(outbox(&a, &share).len(), before);
    assert_eq!(library(&a, &share), held);
    assert!(a.engine.canvas(&share).unwrap().nodes.is_empty());
}

/// An operation a hostile key might send: mostly ones that hold, with odd
/// IDs, huge or tiny numbers, long fields and long strokes among them.
fn hostile_op() -> impl Strategy<Value = serde_json::Value> {
    let id = prop_oneof![
        6 => Just("c1"),
        1 => prop::sample::select(vec!["ink", "ghost", "", "../x", "é"]),
    ];
    let num = prop_oneof![
        8 => -500.0f64..500.0,
        1 => Just(1e7),
        1 => Just(1e300),
        1 => Just(f64::MIN_POSITIVE),
    ];
    let long = prop_oneof![
        6 => Just(1usize),
        1 => prop::sample::select(vec![4_000usize, 5_000, 60_000, 300_000]),
    ];
    // Characters four bytes long, in a field the room gets whole.
    let wide = prop_oneof![3 => Just(1usize), 1 => Just(1_100usize), 1 => Just(90_000usize)];
    prop_oneof![
        (any::<u32>(), num.clone(), num.clone(), long.clone()).prop_map(|(n, x, y, len)| {
            json!({ "op": "addNode", "id": format!("n{n}"), "type": "text", "x": x, "y": y, "text": "t".repeat(len) })
        }),
        (num.clone(), long.clone()).prop_map(|(x, n)| {
            json!({ "op": "addNode", "type": "group", "x": x, "y": 0, "label": "l".repeat(n) })
        }),
        (num.clone(), wide).prop_map(|(x, n)| {
            json!({ "op": "addNode", "type": "link", "x": x, "y": 0, "url": format!("https://x/{}", "\u{1D11E}".repeat(n)) })
        }),
        (id.clone(), long.clone()).prop_map(|(id, n)| {
            json!({ "op": "updateNode", "id": id, "text": "u".repeat(n) })
        }),
        (id.clone(), num.clone(), num.clone())
            .prop_map(|(id, x, y)| json!({ "op": "moveNode", "id": id, "x": x, "y": y })),
        (id.clone(), long.clone()).prop_map(|(a, n)| {
            json!({ "op": "addEdge", "fromNode": a, "toNode": "c1", "label": "e".repeat(n.min(5_000)) })
        }),
        (num.clone(), 1usize..3_000).prop_map(|(x, n)| {
            let points: Vec<serde_json::Value> =
                (0..n).map(|i| json!([x + i as f64, i as f64, 0.5])).collect();
            json!({ "op": "addStrokes", "node": "ink", "strokes": [{ "points": points }] })
        }),
        (0u64..4).prop_map(|v| json!({ "op": "moveNode", "id": "c1", "x": 1, "y": 1, "ifVersion": v })),
        long.prop_map(|n| json!({ "op": "rename", "title": "r".repeat(n.min(600)) })),
        Just(json!({ "op": "removeNode", "id": "c1" })),
    ]
}

proptest! {
    #![proptest_config(ProptestConfig {
        cases: 24,
        rng_seed: proptest::test_runner::RngSeed::Fixed(20_261_017),
        failure_persistence: None,
        ..ProptestConfig::default()
    })]

    /// Hostile batches from the local API on a shared canvas, while someone
    /// draws on it: nothing panics, every update fits the room, a refused
    /// batch sends nothing, and both devices end with the same valid canvas.
    #[test]
    fn hostile_api_batches_never_break_the_room(
        batches in prop::collection::vec(prop::collection::vec(hostile_op(), 1..4), 1..5),
    ) {
        let server = MemoryCollab::new();
        let (a, b) = (device(&server, 1, 1), device(&server, 2, 1));
        let mut start = Canvas::default();
        start.put_card(card("c1", "hello"), 1);
        start.put_node(Node { id: "ink".into(), kind: INK.into(), width: 1.0, height: 1.0, ..Node::default() }, 1);
        let share = shared_canvas(&a, &[(&b, Role::Edit)], &start);
        let nothing = |_: &str| false;
        let mut b_view = open(&b, &share);
        for (round, ops) in batches.iter().enumerate() {
            b_view.put_card(card(&format!("b{round}"), "drawn"), 50 + round as i64);
            b_view = pass(&b, &share, &b_view);
            let before = outbox(&a, &share).len();
            let applied = canvas::Batch::from_json(&json!({ "ops": ops })).ok().map(|batch| {
                a.engine.edit_canvas(&share, |live| {
                    let ctx = canvas::BatchContext {
                        now: 100 + round as i64,
                        shared: true,
                        note_exists: &nothing,
                        attachment_ok: &nothing,
                    };
                    canvas::apply_batch(live, &batch, &ctx)
                        .map(|(next, made)| (next, made.title.clone(), ()))
                        .map_err(|why| why.message())
                })
            });
            let sent = outbox(&a, &share);
            match applied {
                Some(Ok(())) => prop_assert!(sent.len() <= before + 1),
                _ => prop_assert_eq!(sent.len(), before, "a refused batch sent something"),
            }
            for update in &sent {
                prop_assert!(update.len() <= crate::envelope::MAX_UPDATE, "{} bytes", update.len());
            }
            sync(&[&a, &b], &share);
            b_view = pass(&b, &share, &b_view);
        }
        sync(&[&a, &b], &share);
        prop_assert!(a.engine.problems().is_empty(), "{:?}", a.engine.problems());
        let held_a = a.engine.canvas(&share).unwrap();
        let held_b = b.engine.canvas(&share).unwrap();
        prop_assert_eq!(&held_a, &held_b);
        prop_assert_eq!(library(&a, &share).validate(), Ok(()));
        prop_assert_eq!(library(&b, &share).validate(), Ok(()));
    }
}
