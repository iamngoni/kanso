use std::sync::Mutex;

use kanso_core::canvas::{self, Canvas, INK, Node, Point, Stroke};
use kanso_core::{DocType, Query};
use kanso_store::{Origin, Put, SearchParams, Store};
use proptest::prelude::*;
use serde_json::{Value, json};

use super::*;

/// Reads a PNG as the words "ink" and the number of the call, and keeps what
/// it was given.
#[derive(Default)]
struct Fake(Mutex<Vec<Vec<u8>>>);

impl TextReader for Fake {
    fn engine(&self) -> &'static str {
        "fake"
    }
    fn read(&self, image: &[u8]) -> Result<String, String> {
        let mut seen = self.0.lock().unwrap();
        seen.push(image.to_vec());
        Ok(format!("handwriting{}", seen.len()))
    }
}

impl Fake {
    fn calls(&self) -> usize {
        self.0.lock().unwrap().len()
    }
}

fn store() -> (tempfile::TempDir, Store) {
    let dir = tempfile::tempdir().unwrap();
    let store = Store::open(dir.path().join("k.db")).unwrap();
    (dir, store)
}

fn put(store: &Store, kind: DocType, fields: Value) -> String {
    store.put(kind, fields, Put::Upsert, Origin::Ui).unwrap().id
}

fn book(store: &Store) -> String {
    put(store, DocType::Book, json!({"name": "B"}))
}

fn stroke(id: &str, seed: u32) -> Stroke {
    Stroke {
        id: id.into(),
        tool: "pen".into(),
        color: "#1e1e1e".into(),
        width: 3.0,
        points: (0..12)
            .map(|i| Point::new(f64::from(i * 6 + seed), f64::from((i % 4) * 5 + seed), 0.5))
            .collect(),
        ..Stroke::default()
    }
}

/// A canvas of `n` ink layers `i0..`, each with a stroke.
fn board(n: usize) -> Canvas {
    let mut c = Canvas::default();
    for i in 0..n {
        let id = format!("i{i}");
        c.put_node(
            Node {
                id: id.clone(),
                kind: INK.into(),
                x: i as f64 * 300.0,
                width: 200.0,
                height: 100.0,
                ..Node::default()
            },
            1,
        );
        c.add_stroke(&id, stroke(&format!("s{i}"), i as u32), 2);
    }
    c
}

fn canvas_note(store: &Store, book: &str, canvas: &Canvas) -> String {
    put(
        store,
        DocType::Note,
        json!({
            "title": "Board", "doctype": "canvas", "bookId": book,
            "body": canvas::to_body(canvas), "updatedAt": 5,
        }),
    )
}

fn body_of(store: &Store, id: &str) -> String {
    store.note(id).unwrap().unwrap().body
}

fn canvas_of(store: &Store, id: &str) -> Canvas {
    canvas::parse(&body_of(store, id)).unwrap()
}

/// Draws one more stroke on layer `node`, as an edit would.
fn draw(store: &Store, id: &str, node: &str, stroke_id: &str) {
    let mut c = canvas_of(store, id);
    assert!(c.add_stroke(node, stroke(stroke_id, 40), 9));
    let rev = store.get(id).unwrap().unwrap().rev;
    store
        .put(
            DocType::Note,
            json!({"_id": id, "_rev": rev, "body": canvas::to_body(&c)}),
            Put::Merge,
            Origin::Ui,
        )
        .unwrap();
}

fn drain(store: &Store, reader: &Fake) -> usize {
    let mut skip = HashSet::new();
    let mut passes = 0;
    while read_next(store, reader, &mut skip).unwrap() > 0 {
        passes += 1;
        assert!(passes < 100, "never ends");
    }
    passes
}

fn found(store: &Store, word: &str) -> usize {
    let params = SearchParams {
        query: Query::parse(word),
        ..SearchParams::default()
    };
    store.search_summaries(&params).unwrap().len()
}

#[test]
fn recognized_handwriting_is_found_by_search() {
    let (_dir, store) = store();
    let b = book(&store);
    let id = canvas_note(&store, &b, &board(2));
    assert_eq!(found(&store, "handwriting1"), 0);
    let reader = Fake::default();
    assert_eq!(drain(&store, &reader), 2);
    assert!(
        reader
            .0
            .lock()
            .unwrap()
            .iter()
            .all(|p| p.starts_with(b"\x89PNG"))
    );
    let c = canvas_of(&store, &id);
    assert_eq!(c.kanso_ink_ocr.len(), 2);
    assert!(c.kanso_ink_ocr.iter().all(|o| o.engine == "fake"));
    assert_eq!(found(&store, "handwriting1"), 1);
    assert_eq!(found(&store, "handwriting2"), 1);
    // The note's date stays: reading is not an edit.
    assert_eq!(
        store.get(&id).unwrap().unwrap().fields.get("updatedAt"),
        Some(&json!(5))
    );
    // Nothing is left, and nothing is read twice.
    assert_eq!(waiting(&store).unwrap(), 0);
    drain(&store, &reader);
    assert_eq!(reader.calls(), 2);
}

#[test]
fn words_read_from_strokes_that_changed_are_not_kept() {
    let (_dir, store) = store();
    let b = book(&store);
    let id = canvas_note(&store, &b, &board(1));
    let ink = next_unread(&store).unwrap().unwrap();
    draw(&store, &id, "i0", "late");
    assert!(!keep(&store, &id, None, "i0", &ink.md5, "stale", "fake").unwrap());
    assert!(canvas_of(&store, &id).kanso_ink_ocr.is_empty());
    // The new strokes are offered.
    let again = next_unread(&store).unwrap().unwrap();
    assert_ne!(again.md5, ink.md5);
}

#[test]
fn an_inline_sketch_is_read_and_the_note_keeps_its_text() {
    let (_dir, store) = store();
    let b = book(&store);
    let mut sk = canvas::sketch::new_sketch("sk-1", 320, 240);
    sk.add_stroke("i1", stroke("a", 3), 2);
    let body = format!(
        "Shopping\n\n```sketch\n{}\n```\n\nafter",
        serde_json::to_string(&sk).unwrap()
    );
    let id = put(
        &store,
        DocType::Note,
        json!({"title": "N", "bookId": b, "body": body}),
    );
    let reader = Fake::default();
    assert_eq!(drain(&store, &reader), 1);
    let after = body_of(&store, &id);
    assert!(after.starts_with("Shopping\n\n```sketch\n") && after.ends_with("```\n\nafter"));
    let (_, doc) = canvas::sketch::find_by_id(&after, "sk-1").unwrap();
    assert_eq!(doc.kanso_ink_ocr[0].text, "handwriting1");
    assert_eq!(found(&store, "handwriting1"), 1);
    assert_eq!(found(&store, "kansoSketch"), 0);
}

#[test]
fn a_note_without_ink_is_never_read() {
    let (_dir, store) = store();
    let b = book(&store);
    put(
        &store,
        DocType::Note,
        json!({"title": "N", "bookId": b, "body": "plain words"}),
    );
    canvas_note(&store, &b, &canvas::Canvas::default());
    let reader = Fake::default();
    assert_eq!(drain(&store, &reader), 0);
    assert_eq!(reader.calls(), 0);
}

/// How a canvas ends up locked.
#[derive(Clone, Copy, Debug)]
enum Hidden {
    /// Sealed itself, its body empty on disk.
    Sealed,
    /// Sealed, but the plain body is still in the row (a sync mid-way).
    SealedWithBody,
    /// In a locked notebook.
    LockedBook,
    /// Tagged `#ai-exclude`.
    Excluded,
    /// In the trash.
    Trashed,
    /// Shared (its text is merged live).
    Shared,
}

fn lock() -> Value {
    json!({"v": 1, "salt": "c2FsdA", "m": 64, "t": 1, "p": 1, "data": "sealed"})
}

fn hidden() -> impl Strategy<Value = Hidden> {
    prop_oneof![
        Just(Hidden::Sealed),
        Just(Hidden::SealedWithBody),
        Just(Hidden::LockedBook),
        Just(Hidden::Excluded),
        Just(Hidden::Trashed),
        Just(Hidden::Shared),
    ]
}

/// A canvas of ink made unreadable in `how`.
fn hide(store: &Store, how: Hidden) -> (String, String) {
    let open = book(store);
    let id = canvas_note(store, &open, &board(2));
    let before = body_of(store, &id);
    let rev = store.get(&id).unwrap().unwrap().rev;
    match how {
        Hidden::Sealed => {
            store
                .put(
                    DocType::Note,
                    json!({"_id": id, "_rev": rev, "body": "", "kansoLock": lock()}),
                    Put::Merge,
                    Origin::Sync,
                )
                .unwrap();
        }
        Hidden::SealedWithBody => {
            store
                .put(
                    DocType::Note,
                    json!({"_id": id, "_rev": rev, "kansoLock": lock()}),
                    Put::Merge,
                    Origin::Sync,
                )
                .unwrap();
        }
        Hidden::LockedBook => {
            let private = put(store, DocType::Book, json!({"name": "Private"}));
            let mut note = store.get_api(&id, None, false).unwrap();
            note.as_object_mut().unwrap().remove("count");
            note["bookId"] = json!(private);
            store
                .put(DocType::Note, note, Put::Upsert, Origin::Ui)
                .unwrap();
            let mut locked = store.get_api(&private, None, false).unwrap();
            locked.as_object_mut().unwrap().remove("count");
            locked["kansoLock"] = lock();
            store
                .put(DocType::Book, locked, Put::Upsert, Origin::Sync)
                .unwrap();
        }
        Hidden::Excluded => {
            let tag = put(store, DocType::Tag, json!({"name": "ai-exclude"}));
            store
                .put(
                    DocType::Note,
                    json!({"_id": id, "_rev": rev, "tags": [tag]}),
                    Put::Merge,
                    Origin::Ui,
                )
                .unwrap();
        }
        Hidden::Trashed => {
            store
                .put(
                    DocType::Note,
                    json!({"_id": id, "_rev": rev, "bookId": "trash"}),
                    Put::Merge,
                    Origin::Ui,
                )
                .unwrap();
        }
        Hidden::Shared => {
            store
                .put(
                    DocType::Note,
                    json!({"_id": id, "_rev": rev, "kansoShare": {"id": "s"}}),
                    Put::Merge,
                    Origin::Ui,
                )
                .unwrap();
        }
    }
    (id, before)
}

proptest! {
    #![proptest_config(ProptestConfig { cases: 24, ..ProptestConfig::default() })]

    /// A locked canvas is never sent to recognition, however it is locked,
    /// and a canvas beside it still is.
    #[test]
    fn prop_locked_canvases_are_never_sent_to_recognition(how in hidden()) {
        let (_dir, store) = store();
        let (id, _) = hide(&store, how);
        let open = book(&store);
        let plain = canvas_note(&store, &open, &board(1));
        let reader = Fake::default();
        prop_assert_eq!(waiting(&store).unwrap(), 1);
        drain(&store, &reader);
        prop_assert_eq!(reader.calls(), 1);
        prop_assert_eq!(canvas_of(&store, &plain).kanso_ink_ocr.len(), 1);
        // The hidden one was neither read nor written to.
        let doc = store.get(&id).unwrap().unwrap();
        let body = doc.fields.get("body").and_then(Value::as_str).unwrap_or("");
        prop_assert!(!body.contains("kansoInkOcr"));
        // And a write aimed at it is refused.
        prop_assert!(!keep(&store, &id, None, "i0", "x", "secret", "fake").unwrap());
    }

    /// After a full read, editing some layers makes the next pass read
    /// exactly those layers, and the others' recognized text is untouched.
    #[test]
    fn prop_incremental_reread_touches_only_changed_nodes(
        changed in proptest::collection::vec(any::<bool>(), 1..6),
    ) {
        let (_dir, store) = store();
        let b = book(&store);
        let id = canvas_note(&store, &b, &board(changed.len()));
        let reader = Fake::default();
        drain(&store, &reader);
        prop_assert_eq!(reader.calls(), changed.len());
        let before = canvas_of(&store, &id).kanso_ink_ocr;
        let mut edited = Vec::new();
        for (i, &edit) in changed.iter().enumerate() {
            if edit {
                draw(&store, &id, &format!("i{i}"), &format!("late{i}"));
                edited.push(format!("i{i}"));
            }
        }
        prop_assert_eq!(waiting(&store).unwrap(), edited.len());
        drain(&store, &reader);
        prop_assert_eq!(reader.calls(), changed.len() + edited.len());
        let after = canvas_of(&store, &id).kanso_ink_ocr;
        prop_assert_eq!(after.len(), changed.len());
        for old in &before {
            let now = after.iter().find(|o| o.node == old.node).unwrap();
            if edited.contains(&old.node) {
                prop_assert_ne!(&now.md5, &old.md5);
                prop_assert_ne!(&now.text, &old.text);
            } else {
                prop_assert_eq!(now, old);
            }
        }
    }
}
