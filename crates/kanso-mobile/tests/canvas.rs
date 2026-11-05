//! Canvases on the phone (#45): opening and drawing, deferred saving, the
//! erasers and undo, the merge when a canvas changed elsewhere, the places
//! that list canvases, and the guarantee that a locked canvas shows nothing
//! while its lock is closed.

mod common;

use common::*;
use kanso_mobile::{
    CanvasBudget, CanvasPoint, CanvasSession, CanvasView, InkStyle, KansoError, Library, ListPlace,
    ListQuery, PenSample, SearchScope,
};
use proptest::prelude::*;

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

fn new_canvas(library: &Library, title: &str) -> String {
    library
        .create_canvas(None, None, title.to_owned())
        .unwrap()
        .id
}

/// A library with one notebook.
fn library() -> Phone {
    let phone = lone_phone();
    phone.library.create_notebook("Work".into(), None).unwrap();
    phone
}

fn live_ids(view: &CanvasView) -> Vec<String> {
    let mut ids: Vec<_> = view.strokes.iter().map(|s| s.id.clone()).collect();
    ids.sort();
    ids
}

#[test]
fn a_new_canvas_is_empty_and_opens() {
    let phone = library();
    let id = new_canvas(&phone.library, "Board");
    let view = phone.library.canvas(&id).unwrap().view(vec![]).unwrap();
    assert_eq!(view.title, "Board");
    assert!(view.strokes.is_empty() && view.bounds.is_none());
    assert_eq!(view.budget, CanvasBudget::Ok);
    assert!(!view.can_undo && !view.dirty);
    // A plain note is not a canvas.
    let plain = note(&phone.library, "Plain", "words");
    assert!(matches!(
        phone.library.canvas(&plain),
        Err(KansoError::Invalid { .. })
    ));
    assert!(matches!(
        phone.library.canvas("note:nope"),
        Err(KansoError::NotFound)
    ));
}

#[test]
fn drawing_is_saved_when_asked_and_survives_reopening() {
    let phone = library();
    let id = new_canvas(&phone.library, "Board");
    let session = phone.library.canvas(&id).unwrap();
    let view = session.draw(pen(), line(10.0), vec![]).unwrap();
    assert_eq!(view.strokes.len(), 1);
    assert!(view.dirty && view.can_undo);
    let stroke = &view.strokes[0];
    assert!(stroke.outline.len() > 20 && stroke.outline.len().is_multiple_of(2));
    assert!(!stroke.known && stroke.color == "#e03131");
    // Nothing is written until save.
    let before = phone.library.note(&id).unwrap().rev;
    let again = phone.library.canvas(&id).unwrap().view(vec![]).unwrap();
    assert!(again.strokes.is_empty());
    let saved = session.save().unwrap();
    assert_ne!(saved.rev, before);
    assert!(!saved.merged);
    let reopened = phone.library.canvas(&id).unwrap().view(vec![]).unwrap();
    assert_eq!(live_ids(&reopened), live_ids(&view));
    assert!(!reopened.dirty);
    // A save with nothing new writes nothing.
    assert_eq!(session.save().unwrap().rev, saved.rev);
}

#[test]
fn known_strokes_come_back_without_outlines() {
    let phone = library();
    let id = new_canvas(&phone.library, "Board");
    let session = phone.library.canvas(&id).unwrap();
    let first = session.draw(pen(), line(0.0), vec![]).unwrap();
    let known = vec![first.strokes[0].id.clone()];
    let second = session.draw(pen(), line(40.0), known).unwrap();
    assert_eq!(second.strokes.len(), 2);
    let old = second.strokes.iter().find(|s| s.known).unwrap();
    assert!(old.outline.is_empty());
    assert!(
        second
            .strokes
            .iter()
            .find(|s| !s.known)
            .is_some_and(|s| !s.outline.is_empty())
    );
}

#[test]
fn the_erasers_and_undo() {
    let phone = library();
    let id = new_canvas(&phone.library, "Board");
    let session = phone.library.canvas(&id).unwrap();
    session.draw(pen(), line(0.0), vec![]).unwrap();
    session.draw(pen(), line(100.0), vec![]).unwrap();
    // Stroke eraser takes a whole stroke.
    let v = session.erase_at(40.0, 1.0, 3.0, vec![]).unwrap();
    assert_eq!(v.strokes.len(), 1);
    // Undo brings it back; redo takes it away again.
    assert_eq!(session.undo(vec![]).unwrap().strokes.len(), 2);
    assert_eq!(session.redo(vec![]).unwrap().strokes.len(), 1);
    session.undo(vec![]).unwrap();
    // The stroke eraser dragged across both strokes is one step.
    let v = session
        .erase_strokes_along(
            vec![
                CanvasPoint { x: 40.0, y: -20.0 },
                CanvasPoint { x: 40.0, y: 120.0 },
            ],
            3.0,
            vec![],
        )
        .unwrap();
    assert!(v.strokes.is_empty());
    assert_eq!(session.undo(vec![]).unwrap().strokes.len(), 2);
    assert!(phone.library.is_canvas(&id));
    // Partial eraser splits a stroke in two.
    let v = session
        .erase_along(
            vec![
                CanvasPoint { x: 50.0, y: -5.0 },
                CanvasPoint { x: 50.0, y: 5.0 },
            ],
            4.0,
            vec![],
        )
        .unwrap();
    assert_eq!(v.strokes.len(), 3);
    assert_eq!(session.undo(vec![]).unwrap().strokes.len(), 2);
    // Lasso select, move, delete.
    let keys = session
        .select(vec![
            CanvasPoint { x: -10.0, y: -10.0 },
            CanvasPoint { x: 120.0, y: -10.0 },
            CanvasPoint { x: 120.0, y: 20.0 },
            CanvasPoint { x: -10.0, y: 20.0 },
        ])
        .unwrap();
    assert_eq!(keys.len(), 1);
    let moved = session
        .move_strokes(keys.clone(), 0.0, 300.0, vec![])
        .unwrap();
    assert_eq!(moved.strokes.len(), 2);
    assert!(moved.strokes.iter().any(|s| s.bounds.y > 250.0));
    let keys = session
        .select(vec![
            CanvasPoint { x: -10.0, y: 280.0 },
            CanvasPoint { x: 120.0, y: 280.0 },
            CanvasPoint { x: 120.0, y: 330.0 },
            CanvasPoint { x: -10.0, y: 330.0 },
        ])
        .unwrap();
    assert_eq!(
        session.delete_strokes(keys, vec![]).unwrap().strokes.len(),
        1
    );
    session.save().unwrap();
    let reopened = phone.library.canvas(&id).unwrap().view(vec![]).unwrap();
    assert_eq!(reopened.strokes.len(), 1);
}

#[test]
fn cards_arrows_and_live_titles() {
    let phone = library();
    let target = note(&phone.library, "Q4 budget", "numbers");
    let id = new_canvas(&phone.library, "Board");
    let session = phone.library.canvas(&id).unwrap();
    let v = session
        .add_text_card(0.0, 0.0, "Launch plan".into(), vec![])
        .unwrap();
    let text = v.cards[0].id.clone();
    let v = session
        .add_note_card(target.clone(), 400.0, 0.0, vec![])
        .unwrap();
    let card = v.cards.iter().find(|c| c.kind == "note").unwrap();
    assert_eq!(card.text, "Q4 budget");
    assert!(!card.missing && card.note_id.as_deref() == Some(target.as_str()));
    assert!(matches!(
        session.add_note_card("note:ghost".into(), 0.0, 0.0, vec![]),
        Err(KansoError::NotFound)
    ));
    let v = session
        .add_arrow(text.clone(), card.id.clone(), vec![])
        .unwrap();
    assert_eq!(v.arrows.len(), 1);
    assert!(v.arrows[0].to_arrow && v.arrows[0].from.x < v.arrows[0].to.x);
    // The title follows a rename.
    let r = phone.library.note(&target).unwrap().rev;
    phone
        .library
        .save(&target, &r, "numbers v2".into())
        .unwrap();
    let v = session
        .set_card_text(text.clone(), "Launch plan v2".into(), vec![])
        .unwrap();
    assert_eq!(
        v.cards.iter().find(|c| c.id == text).unwrap().text,
        "Launch plan v2"
    );
    // Moving a card moves the arrow with it.
    let rect = kanso_mobile::CanvasRect {
        x: 0.0,
        y: 300.0,
        width: 240.0,
        height: 120.0,
    };
    let v = session.set_card_rect(text.clone(), rect, vec![]).unwrap();
    assert!(v.arrows[0].from.y > 300.0 - 1.0);
    // Undo walks back through card edits; deleting a card removes its arrow.
    assert_eq!(session.undo(vec![]).unwrap().cards[0].rect.y, 0.0);
    let v = session.delete_card(text.clone(), vec![]).unwrap();
    assert!(v.arrows.is_empty() && v.cards.len() == 1);
    // The arrow was only hidden: undoing the delete shows it again, and
    // redoing it hides it again.
    let v = session.undo(vec![]).unwrap();
    assert_eq!((v.cards.len(), v.arrows.len()), (2, 1));
    let v = session.redo(vec![]).unwrap();
    assert!(v.arrows.is_empty() && v.cards.len() == 1);
    session.save().unwrap();
    assert_eq!(
        phone
            .library
            .canvas(&id)
            .unwrap()
            .view(vec![])
            .unwrap()
            .cards
            .len(),
        1
    );
}

#[test]
fn a_canvas_changed_elsewhere_is_merged_not_overwritten() {
    let phone = library();
    let id = new_canvas(&phone.library, "Board");
    let a = phone.library.canvas(&id).unwrap();
    let b = phone.library.canvas(&id).unwrap();
    a.draw(pen(), line(0.0), vec![]).unwrap();
    b.draw(pen(), line(200.0), vec![]).unwrap();
    assert!(!a.save().unwrap().merged);
    let saved = b.save().unwrap();
    assert!(saved.merged);
    let view = phone.library.canvas(&id).unwrap().view(vec![]).unwrap();
    assert_eq!(view.strokes.len(), 2);
    // The session that merged shows both too.
    assert_eq!(b.view(vec![]).unwrap().strokes.len(), 2);
}

#[test]
fn a_full_canvas_refuses_ink_and_warns_before() {
    let phone = library();
    let id = new_canvas(&phone.library, "Board");
    let session = phone.library.canvas(&id).unwrap();
    let mut last = CanvasBudget::Ok;
    let mut warned = false;
    for i in 0..400 {
        let samples: Vec<PenSample> = (0..2000)
            .map(|j| PenSample {
                x: f64::from(j) * 1.7 + f64::from(i % 7),
                y: f64::from(i) * 9.0 + (f64::from(j) * 0.37).sin() * 5.0,
                pressure: 0.3 + 0.2 * (f64::from(j) * 0.11).sin().abs(),
                ..PenSample::default()
            })
            .collect();
        match session.draw(pen(), samples, vec![]) {
            Ok(v) => {
                warned |= v.budget == CanvasBudget::Warn;
                last = v.budget;
                if last == CanvasBudget::Full {
                    break;
                }
            }
            Err(KansoError::Invalid { message }) => {
                assert!(message.contains("full"));
                assert!(warned, "the warning comes before the refusal");
                return;
            }
            Err(e) => panic!("{e}"),
        }
    }
    // At Full the next stroke is refused.
    assert_eq!(last, CanvasBudget::Full);
    assert!(matches!(
        session.draw(pen(), line(5.0), vec![]),
        Err(KansoError::Invalid { .. })
    ));
}

#[test]
fn ink_moves_to_a_new_canvas() {
    let phone = library();
    let id = new_canvas(&phone.library, "Board");
    let session = phone.library.canvas(&id).unwrap();
    session.draw(pen(), line(0.0), vec![]).unwrap();
    session.draw(pen(), line(100.0), vec![]).unwrap();
    let keys = session
        .select(vec![
            CanvasPoint { x: -10.0, y: 80.0 },
            CanvasPoint { x: 120.0, y: 80.0 },
            CanvasPoint { x: 120.0, y: 120.0 },
            CanvasPoint { x: -10.0, y: 120.0 },
        ])
        .unwrap();
    let (saved, view) = session
        .move_to_new_canvas(keys, "More".into(), vec![])
        .unwrap();
    assert_eq!(view.strokes.len(), 1);
    let other = phone
        .library
        .canvas(&saved.id)
        .unwrap()
        .view(vec![])
        .unwrap();
    assert_eq!(other.strokes.len(), 1);
    assert!(other.strokes[0].bounds.y > 90.0);
    assert!(matches!(
        session.move_to_new_canvas(vec![], "x".into(), vec![]),
        Err(KansoError::Invalid { .. })
    ));
}

#[test]
fn canvases_have_their_own_place() {
    let phone = library();
    let canvas = new_canvas(&phone.library, "Board plan");
    let plain = note(&phone.library, "Plain plan", "plan words");
    let ids = |place: ListPlace| -> Vec<String> {
        phone
            .library
            .list(ListQuery {
                place,
                ..ListQuery::default()
            })
            .unwrap()
            .rows
            .into_iter()
            .map(|r| r.id)
            .collect()
    };
    assert_eq!(ids(ListPlace::Canvases), std::slice::from_ref(&canvas));
    assert_eq!(ids(ListPlace::All), std::slice::from_ref(&plain));
    let overview = phone.library.overview().unwrap();
    assert_eq!((overview.all, overview.canvases), (1, 1));
    let rows = phone.library.list(ListQuery::default()).unwrap().rows;
    assert!(rows.iter().all(|r| !r.canvas));
    assert!(
        phone
            .library
            .list(ListQuery {
                place: ListPlace::Canvases,
                ..ListQuery::default()
            })
            .unwrap()
            .rows[0]
            .canvas
    );
    // A search finds both.
    let hits = phone
        .library
        .search("plan", SearchScope::default(), 0)
        .unwrap();
    assert_eq!(hits.len(), 2);
}

/// Handwriting a Mac read arrives in the canvas's body (`kansoInkOcr`); the
/// phone, which doesn't read ink itself, searches those words and nothing
/// of the JSON around them.
#[test]
fn handwriting_a_mac_read_is_searchable_on_the_phone() {
    use kanso_core::DocType;
    use kanso_core::canvas::{self, INK, InkOcr, Node};
    use kanso_store::{Origin, Put, Store};

    let phone = library();
    let id = new_canvas(&phone.library, "Board");
    let mut doc = canvas::Canvas::default();
    doc.put_node(
        Node {
            id: "i1".into(),
            kind: INK.into(),
            width: 100.0,
            height: 50.0,
            ..Node::default()
        },
        1,
    );
    doc.kanso_ink_ocr.push(InkOcr {
        node: "i1".into(),
        md5: "abc".into(),
        engine: "vision".into(),
        text: "groceries for friday".into(),
    });
    let store = Store::open(phone.path()).unwrap();
    let rev = store.get(&id).unwrap().unwrap().rev;
    store
        .put(
            DocType::Note,
            serde_json::json!({"_id": id, "_rev": rev, "body": canvas::to_body(&doc)}),
            Put::Merge,
            Origin::Sync,
        )
        .unwrap();
    let find = |query: &str| {
        phone
            .library
            .search(query, SearchScope::default(), 0)
            .unwrap()
            .len()
    };
    assert_eq!(find("groceries"), 1);
    assert_eq!(find("kansoInkOcr"), 0);
}

#[test]
fn exports_and_excalidraw_import() {
    let phone = library();
    let id = new_canvas(&phone.library, "Board");
    let session = phone.library.canvas(&id).unwrap();
    assert!(
        session.export_svg(false).is_err(),
        "an empty canvas has nothing to draw"
    );
    session
        .add_text_card(0.0, 0.0, "hello".into(), vec![])
        .unwrap();
    session.draw(pen(), line(200.0), vec![]).unwrap();
    let svg = session.export_svg(true).unwrap();
    assert!(svg.starts_with("<svg") && svg.contains("hello"));
    let scene = session.export_excalidraw().unwrap();
    assert!(scene.contains("freedraw") && scene.contains("pressures"));
    let imported = phone
        .library
        .import_excalidraw(scene, None, None, "From Excalidraw".into())
        .unwrap();
    let view = phone
        .library
        .canvas(&imported.id)
        .unwrap()
        .view(vec![])
        .unwrap();
    assert_eq!(view.title, "From Excalidraw");
    assert!(!view.strokes.is_empty());
    assert!(
        phone
            .library
            .import_excalidraw("nope".into(), None, None, "x".into())
            .is_err()
    );
}

const SECRET: &str = "CANVAS-SECRET-7731";

fn debug_has_secret<T: std::fmt::Debug>(value: &T) -> bool {
    format!("{value:?}").contains(SECRET)
}

#[test]
fn a_locked_canvas_shows_nothing_while_its_lock_is_closed() {
    let phone = library();
    let id = new_canvas(&phone.library, SECRET);
    let session = phone.library.canvas(&id).unwrap();
    session
        .add_text_card(0.0, 0.0, format!("{SECRET} body"), vec![])
        .unwrap();
    session.draw(pen(), line(0.0), vec![]).unwrap();
    session.save().unwrap();
    phone
        .library
        .locks()
        .lock_note(&id, "lock password".into())
        .unwrap();
    phone.library.locks().lock().unwrap();

    // Opening refuses, however it is asked.
    assert!(matches!(phone.library.canvas(&id), Err(KansoError::Locked)));
    // Every read path carries the title (as any locked note) and no content.
    let stored = phone.library.note(&id).unwrap();
    assert!(stored.body.is_none());
    let listed = phone
        .library
        .list(ListQuery {
            place: ListPlace::Canvases,
            ..ListQuery::default()
        })
        .unwrap();
    assert_eq!(listed.rows.len(), 1);
    assert!(listed.rows[0].snippet.is_empty() && listed.rows[0].locked);
    let word_hits = phone
        .library
        .search("body", SearchScope::default(), 0)
        .unwrap();
    assert!(word_hits.is_empty(), "{word_hits:?}");
    assert!(
        !debug_has_secret(&phone.library.overview().unwrap()),
        "the overview is counts"
    );

    // Opening the lock brings it back, and a save seals it again.
    assert!(
        phone
            .library
            .locks()
            .unlock_note(&id, "lock password".into())
            .unwrap()
    );
    let open = phone.library.canvas(&id).unwrap();
    let view = open.view(vec![]).unwrap();
    assert_eq!(view.cards.len(), 1);
    assert_eq!(view.strokes.len(), 1);
    open.draw(pen(), line(50.0), vec![]).unwrap();
    open.save().unwrap();
    phone.library.locks().lock().unwrap();
    assert!(matches!(phone.library.canvas(&id), Err(KansoError::Locked)));
    let raw = phone.library.note(&id).unwrap();
    assert!(raw.body.is_none());
}

fn assert_send_sync<T: Send + Sync>() {}

#[test]
fn a_live_stroke_has_an_outline_without_touching_the_library() {
    let outline = kanso_mobile::ink_preview(pen(), line(0.0));
    assert!(outline.len() > 20 && outline.len().is_multiple_of(2));
    assert!(outline.iter().all(|v| v.is_finite()));
    assert!(kanso_mobile::ink_preview(pen(), vec![]).is_empty());
    // One sample is a dot.
    assert!(!kanso_mobile::ink_preview(pen(), line(0.0)[..1].to_vec()).is_empty());
}

#[test]
fn a_session_can_cross_threads() {
    assert_send_sync::<CanvasSession>();
}

#[derive(Clone, Debug)]
enum Step {
    Draw(u8),
    Erase(u8),
    Partial(u8),
    Undo,
    Redo,
    Save,
}

fn steps() -> impl Strategy<Value = Vec<Step>> {
    prop::collection::vec(
        prop_oneof![
            (0u8..8).prop_map(Step::Draw),
            (0u8..8).prop_map(Step::Erase),
            (0u8..8).prop_map(Step::Partial),
            Just(Step::Undo),
            Just(Step::Redo),
            Just(Step::Save),
        ],
        1..16,
    )
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(12))]

    /// Whatever draws, erases, undoes and saves happen, a saved canvas
    /// reopens with exactly the strokes the session showed.
    #[test]
    fn prop_saved_canvas_reopens_as_shown(steps in steps()) {
        let phone = library();
        let id = new_canvas(&phone.library, "Board");
        let session = phone.library.canvas(&id).unwrap();
        for step in steps {
            match step {
                Step::Draw(row) => { session.draw(pen(), line(f64::from(row) * 30.0), vec![]).unwrap(); }
                Step::Erase(row) => { session.erase_at(40.0, f64::from(row) * 30.0, 4.0, vec![]).unwrap(); }
                Step::Partial(row) => {
                    let y = f64::from(row) * 30.0;
                    session.erase_along(vec![CanvasPoint { x: 50.0, y: y - 6.0 }, CanvasPoint { x: 50.0, y: y + 6.0 }], 4.0, vec![]).unwrap();
                }
                Step::Undo => { session.undo(vec![]).unwrap(); }
                Step::Redo => { session.redo(vec![]).unwrap(); }
                Step::Save => { session.save().unwrap(); }
            }
        }
        session.save().unwrap();
        let shown = session.view(vec![]).unwrap();
        let reopened = phone.library.canvas(&id).unwrap().view(vec![]).unwrap();
        prop_assert_eq!(live_ids(&shown), live_ids(&reopened));
        prop_assert!(!reopened.dirty);
    }
}

#[test]
fn an_erase_that_touches_nothing_does_not_write_the_document_out() {
    let phone = library();
    let id = new_canvas(&phone.library, "Board");
    let session = phone.library.canvas(&id).unwrap();
    session.draw(pen(), line(0.0), vec![]).unwrap();
    let opened = session.view(vec![]).unwrap();
    let before = session.size_measurements();
    // Views of an unchanged document, and erasers that miss, measure nothing.
    session.view(vec![]).unwrap();
    for _ in 0..5 {
        let v = session
            .erase_strokes_along(
                vec![
                    CanvasPoint { x: 900.0, y: 900.0 },
                    CanvasPoint { x: 950.0, y: 950.0 },
                ],
                4.0,
                vec![],
            )
            .unwrap();
        assert_eq!(v.bytes, opened.bytes);
        let v = session
            .erase_along(vec![CanvasPoint { x: 900.0, y: 900.0 }], 4.0, vec![])
            .unwrap();
        assert_eq!(v.bytes, opened.bytes);
    }
    assert_eq!(session.size_measurements(), before);
    // A whole drag that does erase is one measurement, however long.
    let path: Vec<CanvasPoint> = (0..40)
        .map(|i| CanvasPoint {
            x: f64::from(i) * 3.0,
            y: 1.0,
        })
        .collect();
    let v = session.erase_strokes_along(path, 4.0, vec![]).unwrap();
    assert!(v.strokes.is_empty());
    assert_eq!(session.size_measurements(), before + 1);
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(12))]

    /// The size a view reports is always that of the document, though it is
    /// measured at most once per call, and only when the document changed.
    #[test]
    fn prop_the_reported_size_is_the_documents(steps in steps()) {
        let phone = library();
        let id = new_canvas(&phone.library, "Board");
        let session = phone.library.canvas(&id).unwrap();
        for step in steps {
            let before = session.size_measurements();
            let view = match step {
                Step::Draw(row) => session.draw(pen(), line(f64::from(row) * 30.0), vec![]).unwrap(),
                Step::Erase(row) => session.erase_strokes_along(
                    vec![CanvasPoint { x: 40.0, y: f64::from(row) * 30.0 }], 4.0, vec![]).unwrap(),
                Step::Partial(row) => {
                    let y = f64::from(row) * 30.0;
                    session.erase_along(vec![CanvasPoint { x: 50.0, y: y - 6.0 }, CanvasPoint { x: 50.0, y: y + 6.0 }], 4.0, vec![]).unwrap()
                }
                Step::Undo => session.undo(vec![]).unwrap(),
                Step::Redo => session.redo(vec![]).unwrap(),
                Step::Save => session.view(vec![]).unwrap(),
            };
            prop_assert!(session.size_measurements() - before <= 1);
            let saved = session.save().unwrap();
            prop_assert_eq!(view.bytes, saved.bytes);
            // Asking again for an unchanged document measures nothing.
            let again = session.size_measurements();
            session.view(vec![]).unwrap();
            prop_assert_eq!(session.size_measurements(), again);
        }
    }
}
