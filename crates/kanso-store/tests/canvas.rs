//! Canvas notes in the store: a body that claims to be a canvas must be one,
//! search reads a canvas's words and not its JSON, two devices' strokes are
//! united by sync, and a canvas keeps a short history.

use kanso_core::canvas::{self, Canvas, Node, Point, Stroke};
use kanso_core::{DocType, Query};
use kanso_store::{ApplyOutcome, Origin, Put, SearchParams, Store, SyncDoc};
use proptest::prelude::*;
use proptest::test_runner::FileFailurePersistence;
use serde_json::{Map, Value, json};

fn config(cases: u32) -> ProptestConfig {
    ProptestConfig {
        cases,
        failure_persistence: Some(Box::new(FileFailurePersistence::WithSource("regressions"))),
        ..ProptestConfig::default()
    }
}

fn temp_store() -> (tempfile::TempDir, Store) {
    let dir = tempfile::tempdir().unwrap();
    let store = Store::open(dir.path().join("kanso.db")).unwrap();
    (dir, store)
}

fn book(store: &Store) -> String {
    store
        .put(
            DocType::Book,
            json!({"_id": "book:a", "name": "A"}),
            Put::Upsert,
            Origin::Ui,
        )
        .unwrap()
        .id
}

fn stroke(id: &str, y: f64) -> Stroke {
    Stroke {
        id: id.into(),
        tool: "pen".into(),
        color: "#000000".into(),
        width: 2.0,
        points: (0..5)
            .map(|i| Point::new(f64::from(i) * 4.0, y, 0.5))
            .collect(),
        ..Stroke::default()
    }
}

/// A canvas with a text card and one ink layer.
fn canvas_with(text: &str) -> Canvas {
    let mut c = Canvas::default();
    c.put_node(
        Node {
            id: "t1".into(),
            kind: "text".into(),
            text: Some(text.into()),
            width: 200.0,
            height: 80.0,
            ..Node::default()
        },
        1,
    );
    c.put_node(
        Node {
            id: "i1".into(),
            kind: "ink".into(),
            width: 100.0,
            height: 100.0,
            ..Node::default()
        },
        1,
    );
    c
}

fn fields(book: &str, canvas: &Canvas, updated: i64) -> Map<String, Value> {
    json!({
        "bookId": book,
        "title": "Board",
        "doctype": "canvas",
        "body": canvas::to_body(canvas),
        "updatedAt": updated,
    })
    .as_object()
    .unwrap()
    .clone()
}

fn put(store: &Store, id: &str, f: &Map<String, Value>, origin: Origin) -> Result<String, String> {
    let mut doc = Value::Object(f.clone());
    doc["_id"] = json!(id);
    if let Some(rev) = store.get(id).unwrap().map(|d| d.rev) {
        doc["_rev"] = json!(rev);
    }
    store
        .put(DocType::Note, doc, Put::Upsert, origin)
        .map(|r| r.rev)
        .map_err(|e| e.to_string())
}

fn stored(store: &Store, id: &str) -> Canvas {
    canvas::parse(&store.note(id).unwrap().unwrap().body).unwrap()
}

fn live_ids(c: &Canvas, node: &str) -> Vec<String> {
    let mut ids: Vec<String> = c
        .node(node)
        .unwrap()
        .strokes
        .iter()
        .filter(|s| !s.deleted)
        .map(|s| s.id.clone())
        .collect();
    ids.sort();
    ids
}

#[test]
fn a_canvas_body_must_be_a_canvas() {
    let (_dir, store) = temp_store();
    let b = book(&store);
    let ok = fields(&b, &canvas_with("hello"), 10);
    assert!(put(&store, "note:ok", &ok, Origin::Api).is_ok());

    // Empty is a new canvas (or a sealed one).
    let mut empty = ok.clone();
    empty.insert("body".into(), "".into());
    assert!(put(&store, "note:new", &empty, Origin::Ui).is_ok());

    for junk in [
        "just some text",
        "```canvas\nnot json\n```",
        "```canvas\n{\"nodes\":[{\"id\":\"a\"}]}\n```",
    ] {
        let mut bad = ok.clone();
        bad.insert("body".into(), junk.into());
        let err = put(&store, "note:bad", &bad, Origin::Api).unwrap_err();
        assert!(err.contains("canvas"), "{err}");
    }
    assert!(store.get("note:bad").unwrap().is_none());

    // Every origin: sync is checked the same way.
    let doc = SyncDoc {
        id: "note:remote".into(),
        rev: "1-aa".into(),
        deleted: false,
        fields: {
            let mut f = ok.clone();
            f.insert("body".into(), "nonsense".into());
            f
        },
        base_rev: None,
    };
    assert!(store.apply_remote(&doc, None, None).is_err());
}

#[test]
fn search_reads_the_words_not_the_json() {
    let (_dir, store) = temp_store();
    let b = book(&store);
    let mut c = canvas_with("quarterly launch plan");
    c.add_stroke("i1", stroke("s1", 5.0), 2);
    put(&store, "note:board", &fields(&b, &c, 10), Origin::Ui).unwrap();
    let find = |q: &str| {
        let params = SearchParams {
            query: Query::parse(q),
            ..SearchParams::default()
        };
        store.search_summaries(&params).unwrap().len()
    };
    assert_eq!(find("launch"), 1);
    // JSON keys and numbers are not in the index.
    assert_eq!(find("kansoVersion"), 0);
    assert_eq!(find("strokes"), 0);
    assert_eq!(find("pressure"), 0);
}

#[test]
fn a_card_that_looks_like_a_task_is_not_a_task() {
    let (_dir, store) = temp_store();
    let b = book(&store);
    let c = canvas_with("- [ ] buy milk\n- [x] pay rent");
    put(&store, "note:board", &fields(&b, &c, 10), Origin::Ui).unwrap();
    let doc = store.get("note:board").unwrap().unwrap();
    assert_eq!(doc.fields.get("numOfTasks"), Some(&json!(0)));
}

#[test]
fn a_sketch_in_a_note_is_searched_by_its_words_only() {
    let (_dir, store) = temp_store();
    let b = book(&store);
    let mut sk = canvas::sketch::new_sketch("sk-1", 320, 240);
    sk.kanso_ink_ocr.push(canvas::InkOcr {
        node: "i1".into(),
        md5: "m".into(),
        engine: "vision".into(),
        text: "groceries".into(),
    });
    let body = format!(
        "Notes\n\n```sketch\n{}\n```\n",
        serde_json::to_string(&sk).unwrap()
    );
    let f = json!({"bookId": b, "title": "N", "body": body, "updatedAt": 1})
        .as_object()
        .unwrap()
        .clone();
    put(&store, "note:n", &f, Origin::Ui).unwrap();
    let find = |q: &str| {
        let params = SearchParams {
            query: Query::parse(q),
            ..SearchParams::default()
        };
        store.search_summaries(&params).unwrap().len()
    };
    assert_eq!(find("groceries"), 1);
    assert_eq!(find("kansoSketch"), 0);
    // A sketch over its cap is refused before the write.
    let huge = format!(
        "```sketch\n{}{}\n```\n",
        serde_json::to_string(&sk).unwrap(),
        " ".repeat(canvas::sketch::MAX_SKETCH_BYTES + 10)
    );
    let mut big = f.clone();
    big.insert("body".into(), huge.into());
    assert!(put(&store, "note:big", &big, Origin::Ui).is_err());
    // A `sketch` code block that isn't Kanso's is the note's own text: no
    // cap, and searched as written.
    let arduino = format!(
        "```sketch\nvoid setup() {{ pinMode(13, OUTPUT); }}\n{}```\n",
        "// blink\n".repeat(40_000)
    );
    let mut code = f.clone();
    code.insert("body".into(), arduino.into());
    put(&store, "note:code", &code, Origin::Ui).unwrap();
    assert_eq!(find("pinMode"), 1);
}

/// Makes the store hold `local` as an unsynced edit of `base`, then applies a
/// remote edit of `base`.
fn conflict(
    local: &Canvas,
    remote: &Canvas,
    l_up: i64,
    r_up: i64,
    extra: &[(&str, Value)],
) -> (Store, tempfile::TempDir, ApplyOutcome, Map<String, Value>) {
    let (dir, store) = temp_store();
    let b = book(&store);
    let mut lf = fields(&b, local, l_up);
    let mut rf = fields(&b, remote, r_up);
    for (k, v) in extra {
        lf.insert((*k).into(), v.clone());
        rf.insert((*k).into(), v.clone());
    }
    put(&store, "note:board", &lf, Origin::Ui).unwrap();
    let doc = SyncDoc {
        id: "note:board".into(),
        rev: "5-bb".into(),
        deleted: false,
        fields: rf.clone(),
        base_rev: None,
    };
    let outcome = store.apply_remote(&doc, None, None).unwrap();
    (store, dir, outcome, rf)
}

#[test]
fn strokes_drawn_on_two_devices_are_both_kept() {
    let mut local = canvas_with("board");
    local.add_stroke("i1", stroke("mac", 1.0), 100);
    let mut remote = canvas_with("board");
    remote.add_stroke("i1", stroke("phone", 9.0), 200);
    // The remote save is newer: last-writer-wins would drop "mac".
    let (store, _dir, outcome, remote_fields) = conflict(&local, &remote, 1_000, 2_000, &[]);
    assert!(
        matches!(outcome, ApplyOutcome::LocalWon { .. }),
        "{outcome:?}"
    );
    let merged = stored(&store, "note:board");
    assert_eq!(live_ids(&merged, "i1"), vec!["mac", "phone"]);
    assert!(merged.validate().is_ok());
    // The remote version is in history, and the note moved to the newer time.
    let history = store.revisions("note:board").unwrap();
    assert!(
        history
            .iter()
            .any(|r| r.fields.get("body") == remote_fields.get("body"))
    );
    assert_eq!(store.note("note:board").unwrap().unwrap().updated_at, 2_000);
}

/// `n` long strokes named `who0`, `who1`… on layer i1.
fn heavy(who: &str, n: usize) -> Canvas {
    let mut c = canvas_with("board");
    for s in 0..n {
        let mut long = stroke(&format!("{who}{s}"), 0.0);
        long.points = (0..15_000)
            .map(|i| Point::new(f64::from(i % 997) + 0.25, f64::from(i / 997) + 0.75, 0.5))
            .collect();
        c.add_stroke("i1", long, 100);
    }
    c
}

/// Two halves within the budget whose union is past it are not united (no
/// other store would take the result): the newer side wins whole, and the
/// other is in history.
#[test]
fn a_union_past_the_budget_falls_back_to_the_newer_side() {
    let local = heavy("mac", 2);
    let remote = heavy("phone", 2);
    let (l, r) = (canvas::to_body(&local), canvas::to_body(&remote));
    assert!(canvas::check_body(&l).is_ok() && canvas::check_body(&r).is_ok());
    let union = canvas::merge_bodies(&l, &r).unwrap();
    assert!(canvas::check_body(&union).is_err(), "{} bytes", union.len());
    let (store, _dir, outcome, remote_fields) = conflict(&local, &remote, 1_000, 2_000, &[]);
    assert!(matches!(outcome, ApplyOutcome::RemoteWon), "{outcome:?}");
    let note = store.note("note:board").unwrap().unwrap();
    assert_eq!(
        Some(&Value::String(note.body.clone())),
        remote_fields.get("body")
    );
    assert!(canvas::check_body(&note.body).is_ok());
}

#[test]
fn a_remote_that_already_holds_everything_just_wins() {
    let local = canvas_with("board");
    let mut remote = local.clone();
    remote.add_stroke("i1", stroke("phone", 9.0), 200);
    let (store, _dir, outcome, _) = conflict(&local, &remote, 1_000, 2_000, &[]);
    assert!(
        matches!(
            outcome,
            ApplyOutcome::RemoteWon | ApplyOutcome::LocalWon { .. }
        ),
        "{outcome:?}"
    );
    assert_eq!(live_ids(&stored(&store, "note:board"), "i1"), vec!["phone"]);
}

#[test]
fn a_deleted_stroke_stays_deleted_through_the_merge() {
    let mut base = canvas_with("board");
    base.add_stroke("i1", stroke("old", 1.0), 10);
    let mut local = base.clone();
    local.remove_stroke("i1", "old", 100);
    let mut remote = base.clone();
    remote.add_stroke("i1", stroke("new", 2.0), 200);
    let (store, _dir, _, _) = conflict(&local, &remote, 1_000, 2_000, &[]);
    assert_eq!(live_ids(&stored(&store, "note:board"), "i1"), vec!["new"]);
}

#[test]
fn a_shared_canvas_is_left_to_its_crdt() {
    let mut local = canvas_with("board");
    local.add_stroke("i1", stroke("mac", 1.0), 100);
    let mut remote = canvas_with("board");
    remote.add_stroke("i1", stroke("phone", 9.0), 200);
    // Marked shared: the CRDT owns the text, so private sync neither merges nor replaces it.
    let share = [("kansoShare", json!({"shareId": "s", "role": "editor"}))];
    let (store, _dir, _, _) = conflict(&local, &remote, 1_000, 2_000, &share);
    assert_eq!(live_ids(&stored(&store, "note:board"), "i1"), vec!["mac"]);
}

#[test]
fn a_canvas_keeps_ten_snapshots_and_a_note_fifty() {
    let (_dir, store) = temp_store();
    let b = book(&store);
    let mut c = canvas_with("board");
    put(&store, "note:board", &fields(&b, &c, 1), Origin::Api).unwrap();
    let md = json!({"bookId": b, "title": "N", "body": "v0", "updatedAt": 1})
        .as_object()
        .unwrap()
        .clone();
    put(&store, "note:md", &md, Origin::Api).unwrap();
    for i in 0..60 {
        c.add_stroke(
            "i1",
            stroke(&format!("s{i}"), f64::from(i)),
            10 + i64::from(i),
        );
        put(
            &store,
            "note:board",
            &fields(&b, &c, 10 + i64::from(i)),
            Origin::Api,
        )
        .unwrap();
        let mut f = md.clone();
        f.insert("body".into(), format!("v{}", i + 1).into());
        put(&store, "note:md", &f, Origin::Api).unwrap();
    }
    assert_eq!(store.revisions("note:board").unwrap().len(), 10);
    assert_eq!(store.revisions("note:md").unwrap().len(), 50);
}

#[test]
fn a_canvas_is_snapshotted_only_when_idle() {
    let (_dir, store) = temp_store();
    let b = book(&store);
    let mut c = canvas_with("board");
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_millis() as i64;
    put(&store, "note:board", &fields(&b, &c, now), Origin::Ui).unwrap();
    // Saves from the canvas view while the pen is moving add no snapshot.
    for i in 0..5 {
        c.add_stroke(
            "i1",
            stroke(&format!("s{i}"), f64::from(i)),
            now + i64::from(i),
        );
        put(
            &store,
            "note:board",
            &fields(&b, &c, now + i64::from(i)),
            Origin::Ui,
        )
        .unwrap();
    }
    assert!(store.revisions("note:board").unwrap().is_empty());
    // A save after an idle spell keeps the version the pen left behind.
    let long_ago = now - 10 * 60 * 1000;
    let (_dir2, store2) = temp_store();
    let b2 = book(&store2);
    put(
        &store2,
        "note:board",
        &fields(&b2, &c, long_ago),
        Origin::Ui,
    )
    .unwrap();
    let mut c2 = c.clone();
    c2.add_stroke("i1", stroke("late", 99.0), now);
    put(&store2, "note:board", &fields(&b2, &c2, now), Origin::Ui).unwrap();
    assert_eq!(store2.revisions("note:board").unwrap().len(), 1);
    // A version stamped by a device whose clock runs an hour ahead is not
    // "still being drawn": the next save here keeps it.
    let (_dir3, store3) = temp_store();
    let b3 = book(&store3);
    put(
        &store3,
        "note:board",
        &fields(&b3, &c, now + 60 * 60 * 1000),
        Origin::Ui,
    )
    .unwrap();
    put(&store3, "note:board", &fields(&b3, &c2, now), Origin::Ui).unwrap();
    assert_eq!(store3.revisions("note:board").unwrap().len(), 1);
}

proptest! {
    #![proptest_config(config(24))]

    /// Whatever two devices drew, the synced canvas holds every stroke from
    /// both and is a valid canvas the store accepts.
    #[test]
    fn prop_sync_never_loses_a_stroke(
        mine in prop::collection::btree_set(0u8..20, 0..8),
        theirs in prop::collection::btree_set(20u8..40, 0..8),
        remote_newer in any::<bool>(),
    ) {
        let mut local = canvas_with("board");
        for (n, s) in mine.iter().enumerate() {
            local.add_stroke("i1", stroke(&format!("m{s}"), f64::from(*s)), 100 + n as i64);
        }
        let mut remote = canvas_with("board");
        for (n, s) in theirs.iter().enumerate() {
            remote.add_stroke("i1", stroke(&format!("r{s}"), f64::from(*s)), 100 + n as i64);
        }
        let (l_up, r_up) = if remote_newer { (1_000, 2_000) } else { (2_000, 1_000) };
        let (store, _dir, _, _) = conflict(&local, &remote, l_up, r_up, &[]);
        let merged = stored(&store, "note:board");
        let mut want: Vec<String> = mine.iter().map(|s| format!("m{s}")).chain(theirs.iter().map(|s| format!("r{s}"))).collect();
        want.sort();
        prop_assert_eq!(live_ids(&merged, "i1"), want);
        prop_assert!(merged.validate().is_ok());
    }
}

/// A plain Markdown note in the same book.
fn markdown(store: &Store, id: &str, b: &str, body: &str) {
    let f = json!({"bookId": b, "title": "Plain", "body": body, "updatedAt": 1})
        .as_object()
        .unwrap()
        .clone();
    put(store, id, &f, Origin::Ui).unwrap();
}

fn listed(store: &Store, doctype: Option<&str>, not: bool, q: &str) -> Vec<String> {
    let params = SearchParams {
        query: Query::parse(q),
        doctype: doctype.map(str::to_owned),
        not_doctype: not,
        ..SearchParams::default()
    };
    let mut ids: Vec<String> = store
        .search_summaries(&params)
        .unwrap()
        .into_iter()
        .map(|s| s.id)
        .collect();
    ids.sort();
    ids
}

#[test]
fn canvases_have_their_own_list_and_leave_all_notes() {
    let (_dir, store) = temp_store();
    let b = book(&store);
    put(
        &store,
        "note:board",
        &fields(&b, &canvas_with("shared words"), 10),
        Origin::Ui,
    )
    .unwrap();
    markdown(&store, "note:plain", &b, "shared words too");

    assert_eq!(listed(&store, Some("canvas"), false, ""), ["note:board"]);
    // All Notes with no search leaves canvases out ...
    assert_eq!(listed(&store, Some("canvas"), true, ""), ["note:plain"]);
    // ... and no filter (a search, a notebook, a tag) finds both.
    assert_eq!(
        listed(&store, None, false, "shared"),
        ["note:board", "note:plain"]
    );

    let summaries = store.search_summaries(&SearchParams::default()).unwrap();
    assert!(
        summaries
            .iter()
            .find(|s| s.id == "note:board")
            .unwrap()
            .canvas
    );
    assert!(
        !summaries
            .iter()
            .find(|s| s.id == "note:plain")
            .unwrap()
            .canvas
    );

    let counts = store.status_counts().unwrap();
    assert_eq!((counts.all, counts.canvases), (2, 1));
    assert_eq!(
        store.canvas_ids().unwrap().into_iter().collect::<Vec<_>>(),
        ["note:board"]
    );
}

#[test]
fn a_trashed_canvas_is_not_counted_or_listed_as_one() {
    let (_dir, store) = temp_store();
    let b = book(&store);
    put(
        &store,
        "note:board",
        &fields(&b, &canvas_with("x"), 10),
        Origin::Ui,
    )
    .unwrap();
    let mut f = fields(&b, &canvas_with("x"), 11);
    f.insert("bookId".into(), "trash".into());
    put(&store, "note:board", &f, Origin::Ui).unwrap();
    let counts = store.status_counts().unwrap();
    assert_eq!((counts.canvases, counts.trash), (0, 1));
    assert!(store.canvas_ids().unwrap().is_empty());
    assert!(listed(&store, Some("canvas"), false, "").is_empty());
}
