//! Batches of operations (issue #45, step 12): atomic, validated, versioned.

use std::collections::HashMap;

use proptest::prelude::*;
use proptest::test_runner::FileFailurePersistence;
use serde_json::{Value, json};

use super::*;

fn config(cases: u32) -> ProptestConfig {
    ProptestConfig {
        cases,
        failure_persistence: Some(Box::new(FileFailurePersistence::WithSource("regressions"))),
        ..ProptestConfig::default()
    }
}

const NOW: i64 = 5_000;

fn notes(id: &str) -> bool {
    id == "note:known"
}

fn files(id: &str) -> bool {
    id == "file:mine"
}

fn ctx(shared: bool) -> BatchContext<'static> {
    BatchContext {
        now: NOW,
        shared,
        note_exists: &notes,
        attachment_ok: &files,
    }
}

fn batch(ops: Value) -> Batch {
    Batch::from_json(&json!({ "ops": ops })).unwrap()
}

/// A canvas with a text card, a group, an arrow and a layer of ink, all at
/// version 3.
fn seed() -> Canvas {
    let mut c = Canvas::default();
    let mut put = |node: Node| {
        let mut node = node;
        node.kanso_version = 3;
        node.kanso_updated = 1;
        node.kanso_z = c.nodes.len() as f64 + 1.0;
        c.nodes.push(node);
    };
    put(Node {
        id: "t1".into(),
        kind: "text".into(),
        x: 10.0,
        y: 10.0,
        width: 100.0,
        height: 40.0,
        text: Some("hello".into()),
        ..Node::default()
    });
    put(Node {
        id: "g1".into(),
        kind: "group".into(),
        x: 0.0,
        y: 0.0,
        width: 400.0,
        height: 300.0,
        label: Some("Plan".into()),
        ..Node::default()
    });
    put(Node {
        id: "far".into(),
        kind: "text".into(),
        x: 900.0,
        y: 900.0,
        width: 100.0,
        height: 40.0,
        text: Some("elsewhere".into()),
        ..Node::default()
    });
    put(Node {
        id: "i1".into(),
        kind: "ink".into(),
        width: 1.0,
        height: 1.0,
        strokes: vec![
            Stroke {
                id: "s1".into(),
                tool: "pen".into(),
                color: "#000000".into(),
                width: 2.0,
                points: vec![Point::new(20.0, 20.0, 0.5), Point::new(60.0, 60.0, 0.5)],
                kanso_version: 3,
                ..Stroke::default()
            },
            Stroke {
                id: "s2".into(),
                tool: "pen".into(),
                color: "#000000".into(),
                width: 2.0,
                points: vec![Point::new(800.0, 800.0, 0.5), Point::new(850.0, 850.0, 0.5)],
                kanso_version: 3,
                ..Stroke::default()
            },
        ],
        ..Node::default()
    });
    c.edges.push(Edge {
        id: "e1".into(),
        from_node: "t1".into(),
        to_node: "far".into(),
        kanso_version: 3,
        ..Edge::default()
    });
    c.refit_ink();
    c
}

fn run(c: &Canvas, ops: Value) -> Result<(Canvas, Applied), Refusal> {
    apply_batch(c, &batch(ops), &ctx(false))
}

#[test]
fn a_batch_adds_moves_and_removes_with_versions() {
    let (next, made) = run(
        &seed(),
        json!([
            { "op": "addNode", "type": "text", "x": 5, "y": 6, "text": "new" },
            { "op": "moveNode", "id": "t1", "x": 50, "y": 60 },
            { "op": "updateNode", "id": "t1", "text": "changed", "color": "2" },
            { "op": "removeNode", "id": "far" },
            { "op": "addEdge", "fromNode": "t1", "toNode": "g1", "label": "in" },
        ]),
    )
    .unwrap();
    assert_eq!(made.nodes.len(), 1);
    assert_eq!(made.edges.len(), 1);
    let t1 = next.node("t1").unwrap();
    assert_eq!((t1.x, t1.y), (50.0, 60.0));
    assert_eq!(t1.text.as_deref(), Some("changed"));
    // Two edits to one card: version 3 became 5, stamped now.
    assert_eq!(t1.kanso_version, 5);
    assert_eq!(t1.kanso_updated, NOW);
    assert!(next.node("far").is_none());
    // The arrow to the removed card went with it.
    assert!(next.edges.iter().find(|e| e.id == "e1").unwrap().deleted);
    assert!(next.node(&made.nodes[0]).is_some());
    assert_eq!(next.validate(), Ok(()));
}

#[test]
fn a_refused_operation_changes_nothing() {
    let before = seed();
    let refused = run(
        &before,
        json!([
            { "op": "moveNode", "id": "t1", "x": 1, "y": 1 },
            { "op": "moveNode", "id": "nope", "x": 1, "y": 1 },
        ]),
    );
    assert!(matches!(refused, Err(Refusal::Missing(_))), "{refused:?}");
    // The canvas the batch started from is untouched.
    assert_eq!(before, seed());
}

#[test]
fn an_error_names_the_operation() {
    let err = run(
        &seed(),
        json!([
            { "op": "addNode", "type": "text", "x": 0, "y": 0 },
            { "op": "addNode", "type": "text", "x": 0, "y": 0, "width": -4 },
        ]),
    )
    .unwrap_err();
    assert!(err.message().starts_with("Operation 2:"), "{err:?}");
}

#[test]
fn a_stale_version_is_refused_with_the_current_one() {
    let err = run(
        &seed(),
        json!([
            { "op": "moveNode", "id": "t1", "x": 1, "y": 1, "ifVersion": 2 },
            { "op": "removeNode", "id": "far", "ifVersion": 3 },
            { "op": "removeEdge", "id": "e1", "ifVersion": 9 },
            { "op": "removeStrokes", "node": "i1", "strokes": [{ "id": "s1", "ifVersion": 1 }] },
        ]),
    )
    .unwrap_err();
    let Refusal::Stale(items) = err else {
        panic!("not stale");
    };
    let mut ids: Vec<_> = items.iter().map(|s| (s.id.as_str(), s.version)).collect();
    ids.sort();
    assert_eq!(ids, [("e1", 3), ("s1", 3), ("t1", 3)]);
}

#[test]
fn the_right_version_passes() {
    let (next, _) = run(
        &seed(),
        json!([{ "op": "moveNode", "id": "t1", "x": 1, "y": 1, "ifVersion": 3 }]),
    )
    .unwrap();
    assert_eq!(next.node("t1").unwrap().kanso_version, 4);
}

#[test]
fn strokes_are_added_removed_and_moved_by_id() {
    let (next, made) = run(
        &seed(),
        json!([
            { "op": "addStrokes", "node": "i1", "strokes": [
                { "tool": "marker", "color": "#cc0000", "width": 6, "points": [[0, 0, 0.4], [10, 10, 0.6]] }
            ] },
            { "op": "removeStrokes", "node": "i1", "strokes": ["s2"] },
            { "op": "moveStrokes", "node": "i1", "strokes": ["s1"], "dx": 100, "dy": 0 },
        ]),
    )
    .unwrap();
    let live: Vec<_> = next.live_strokes().map(|(_, s)| s.id.clone()).collect();
    // s1 moved (a tombstone and a copy), s2 gone, one new stroke.
    assert_eq!(live.len(), 2, "{live:?}");
    assert!(!live.contains(&"s1".to_owned()) && !live.contains(&"s2".to_owned()));
    assert_eq!(made.strokes.len(), 2);
    let moved = next
        .live_strokes()
        .map(|(_, s)| s)
        .find(|s| s.points[0].x == 120.0)
        .unwrap();
    assert_eq!(moved.points[1].x, 160.0);
    assert!(next.validate().is_ok());
}

#[test]
fn moving_a_stroke_that_is_already_gone_is_refused() {
    let err = run(
        &seed(),
        json!([
            { "op": "removeStrokes", "node": "i1", "strokes": ["s1"] },
            { "op": "moveStrokes", "node": "i1", "strokes": ["s1"], "dx": 1, "dy": 1 },
        ]),
    )
    .unwrap_err();
    assert!(matches!(err, Refusal::Missing(_)), "{err:?}");
}

#[test]
fn cards_point_only_at_known_notes_and_own_attachments() {
    let add = |file: &str| {
        run(
            &seed(),
            json!([{ "op": "addNode", "type": "file", "x": 0, "y": 0, "file": file }]),
        )
    };
    assert!(add("note:known").is_ok());
    assert!(add("file:mine").is_ok());
    assert!(matches!(add("note:ghost"), Err(Refusal::Missing(_))));
    assert!(matches!(add("file:other"), Err(Refusal::Invalid(_))));
    assert!(matches!(add("/etc/passwd"), Err(Refusal::Invalid(_))));
}

#[test]
fn a_shared_canvas_takes_notes_but_no_pictures_yet() {
    let ops = |file: &str| {
        batch(json!([{ "op": "addNode", "type": "file", "x": 0, "y": 0, "file": file }]))
    };
    assert!(apply_batch(&seed(), &ops("note:known"), &ctx(true)).is_ok());
    let err = apply_batch(&seed(), &ops("file:mine"), &ctx(true)).unwrap_err();
    assert_eq!(
        err,
        Refusal::Invalid(format!("Operation 1: {SHARED_ATTACHMENT_REFUSAL}"))
    );
}

#[test]
fn bad_values_are_refused() {
    for op in [
        json!({ "op": "addNode", "type": "text", "x": 1e12, "y": 0 }),
        json!({ "op": "addNode", "type": "text", "x": 0, "y": 0, "width": 0 }),
        json!({ "op": "addNode", "type": "banana", "x": 0, "y": 0 }),
        json!({ "op": "addNode", "type": "text", "x": 0, "y": 0, "color": "red" }),
        json!({ "op": "addNode", "type": "link", "x": 0, "y": 0, "url": "javascript:alert(1)" }),
        json!({ "op": "addNode", "type": "text", "x": 0, "y": 0, "id": "t1" }),
        json!({ "op": "addNode", "type": "text", "x": 0, "y": 0, "label": "no" }),
        json!({ "op": "addEdge", "fromNode": "t1", "toNode": "ghost" }),
        json!({ "op": "addEdge", "fromNode": "t1", "toNode": "g1", "fromSide": "middle" }),
        json!({ "op": "moveNode", "id": "i1", "x": 1, "y": 1 }),
        json!({ "op": "removeNode", "id": "i1" }),
        json!({ "op": "updateNode", "id": "t1", "label": "x" }),
        json!({ "op": "addStrokes", "node": "t1", "strokes": [{ "points": [[0, 0, 1]] }] }),
        json!({ "op": "addStrokes", "node": "i1", "strokes": [{ "points": [[0, 0, 2]] }] }),
        json!({ "op": "addStrokes", "node": "i1", "strokes": [{ "tool": "laser", "points": [[0, 0, 1]] }] }),
        json!({ "op": "addStrokes", "node": "i1", "strokes": [{ "points": [] }] }),
        json!({ "op": "rename", "title": "  " }),
    ] {
        let result = run(&seed(), json!([op.clone()]));
        assert!(result.is_err(), "{op} was taken");
    }
    // Unknown operations and unknown fields are refused when read.
    assert!(Batch::from_json(&json!({ "ops": [{ "op": "explode" }] })).is_err());
    assert!(
        Batch::from_json(
            &json!({ "ops": [{ "op": "moveNode", "id": "a", "x": 1, "y": 1, "z": 3 }] })
        )
        .is_err()
    );
    assert!(Batch::from_json(&json!({ "ops": [], "extra": 1 })).is_err());
    assert!(Batch::from_json(&json!({})).is_err());
}

#[test]
fn a_batch_over_the_budget_is_refused_with_the_budget() {
    let mut c = seed();
    c.node_mut("t1")
        .unwrap()
        .extra
        .insert("pad".into(), "x".repeat(MAX_BYTES - 1_000).into());
    let err = run(
        &c,
        json!([{ "op": "addNode", "type": "text", "x": 0, "y": 0, "text": "y".repeat(5_000) }]),
    )
    .unwrap_err();
    assert!(
        matches!(&err, Refusal::Full(m) if m.contains("1000000")),
        "{err:?}"
    );
}

#[test]
fn groups_take_cards_in_and_out() {
    let (inside, _) = run(
        &seed(),
        json!([{ "op": "setGroup", "nodes": ["far"], "group": "g1" }]),
    )
    .unwrap();
    let area = inside.group_bounds("g1").unwrap();
    assert!(area.contains(&inside.node("far").unwrap().bounds()));
    let (outside, _) = run(&inside, json!([{ "op": "setGroup", "nodes": ["far"] }])).unwrap();
    assert!(
        !outside
            .group_bounds("g1")
            .unwrap()
            .contains(&outside.node("far").unwrap().bounds())
    );
    let (renamed, _) = run(
        &seed(),
        json!([{ "op": "renameGroup", "id": "g1", "label": "Ship it" }]),
    )
    .unwrap();
    assert_eq!(
        renamed.node("g1").unwrap().label.as_deref(),
        Some("Ship it")
    );
}

#[test]
fn a_group_view_has_only_what_is_inside() {
    let view = seed().group_view("g1").unwrap();
    let ids: Vec<_> = view.nodes.iter().map(|n| n.id.as_str()).collect();
    assert!(ids.contains(&"g1") && ids.contains(&"t1") && ids.contains(&"i1"));
    assert!(!ids.contains(&"far"));
    let ink = view.node("i1").unwrap();
    assert_eq!(ink.strokes.len(), 1);
    assert_eq!(ink.strokes[0].id, "s1");
    // The arrow leaves the group, so it isn't in the view.
    assert!(view.edges.is_empty());
    assert!(seed().group_view("t1").is_none());
}

#[test]
fn reading_leaves_out_tombstones_and_keeps_versions() {
    let (gone, _) = run(&seed(), json!([{ "op": "removeNode", "id": "far" }])).unwrap();
    let live = gone.live();
    assert!(live.nodes.iter().all(|n| !n.deleted));
    assert!(live.edges.is_empty());
    assert_eq!(live.node("t1").unwrap().kanso_version, 3);
}

// ---------------------------------------------------------------- properties

/// A random operation over the seed's IDs, some of them hostile.
fn arb_op() -> impl Strategy<Value = Value> {
    let id = prop::sample::select(vec![
        "t1", "g1", "far", "i1", "ghost", "e1", "new-1", "new-2",
    ]);
    let num = prop_oneof![
        -50.0f64..2000.0,
        Just(0.0),
        Just(-1.0),
        Just(1e12),
        Just(1e-9),
    ];
    let point = (
        num.clone(),
        num.clone(),
        prop_oneof![0.0f64..=1.0, Just(2.0)],
    )
        .prop_map(|(x, y, p)| json!([x, y, p]));
    let kind = prop::sample::select(vec!["text", "group", "link", "file", "ink", "bogus"]);
    prop_oneof![
        (kind, num.clone(), num.clone(), num.clone(), id.clone(), any::<bool>()).prop_map(
            |(kind, x, y, w, id, named)| {
                let mut op = json!({ "op": "addNode", "type": kind, "x": x, "y": y, "width": w });
                if named {
                    op["id"] = id.into();
                }
                match kind {
                    "link" => op["url"] = "https://example.com".into(),
                    "file" => op["file"] = "note:known".into(),
                    _ => {}
                }
                op
            }
        ),
        (id.clone(), num.clone(), num.clone())
            .prop_map(|(id, x, y)| json!({ "op": "moveNode", "id": id, "x": x, "y": y })),
        (id.clone(), num.clone(), num.clone())
            .prop_map(|(id, w, h)| json!({ "op": "resizeNode", "id": id, "width": w, "height": h })),
        id.clone().prop_map(|id| json!({ "op": "removeNode", "id": id })),
        id.clone().prop_map(|id| json!({ "op": "updateNode", "id": id, "text": "t", "label": "l" })),
        (id.clone(), id.clone())
            .prop_map(|(a, b)| json!({ "op": "addEdge", "fromNode": a, "toNode": b })),
        id.clone().prop_map(|id| json!({ "op": "removeEdge", "id": id })),
        (id.clone(), prop::collection::vec(point, 0..6))
            .prop_map(|(id, points)| json!({ "op": "addStrokes", "node": id, "strokes": [{ "points": points }] })),
        (id.clone(), prop::sample::select(vec!["s1", "s2", "zzz"]))
            .prop_map(|(id, s)| json!({ "op": "removeStrokes", "node": id, "strokes": [s] })),
        (prop::sample::select(vec!["s1", "s2"]), num.clone(), num.clone()).prop_map(
            |(s, dx, dy)| json!({ "op": "moveStrokes", "node": "i1", "strokes": [s], "dx": dx, "dy": dy })
        ),
        (id.clone(), prop::option::of(id.clone()))
            .prop_map(|(n, g)| json!({ "op": "setGroup", "nodes": [n], "group": g })),
        (id.clone(), 0u64..6)
            .prop_map(|(id, v)| json!({ "op": "moveNode", "id": id, "x": 1, "y": 1, "ifVersion": v })),
    ]
}

fn versions(c: &Canvas) -> HashMap<String, u64> {
    let mut out = HashMap::new();
    for n in &c.nodes {
        out.insert(n.id.clone(), n.kanso_version);
        for s in &n.strokes {
            out.insert(s.id.clone(), s.kanso_version);
        }
    }
    for e in &c.edges {
        out.insert(e.id.clone(), e.kanso_version);
    }
    out
}

proptest! {
    #![proptest_config(config(256))]

    /// No sequence of operations makes a canvas the view can't open, one over
    /// the budget, or one that doesn't survive being stored and read; and a
    /// refused batch is the same as none.
    #[test]
    fn no_batch_makes_an_invalid_canvas(ops in prop::collection::vec(arb_op(), 0..12)) {
        let before = seed();
        let batch = Batch::from_json(&json!({ "ops": ops })).unwrap();
        match apply_batch(&before, &batch, &ctx(false)) {
            Err(_) => prop_assert_eq!(&before, &seed()),
            Ok((next, made)) => {
                prop_assert_eq!(next.validate(), Ok(()));
                let body = to_body(&next);
                prop_assert!(body.len() < MAX_BYTES);
                // Read back, it is the same canvas. A coordinate may come
                // back a last bit off (serde_json reads floats fast, not
                // exactly), so compare what it stores again.
                let back = parse(&body).unwrap();
                prop_assert_eq!(back.validate(), Ok(()));
                prop_assert_eq!(back.nodes.len(), next.nodes.len());
                prop_assert_eq!(back.edges.len(), next.edges.len());
                let ids = |c: &Canvas| c.nodes.iter().map(|n| n.id.clone()).collect::<Vec<_>>();
                prop_assert_eq!(ids(&back), ids(&next));
                // What the batch made is there, and was not there before.
                let ids = versions(&before);
                for id in made.nodes.iter().chain(&made.edges).chain(&made.strokes) {
                    prop_assert!(!ids.contains_key(id), "{} was already there", id);
                }
                for id in &made.nodes {
                    prop_assert!(next.node(id).is_some());
                }
            }
        }
    }

    /// Versions only go up, and every item a batch touched is stamped with the
    /// batch's time: the private merge and history treat it as any edit.
    #[test]
    fn versions_never_go_down_and_changes_are_stamped(ops in prop::collection::vec(arb_op(), 1..10)) {
        let before = seed();
        let batch = Batch::from_json(&json!({ "ops": ops })).unwrap();
        if let Ok((next, _)) = apply_batch(&before, &batch, &ctx(false)) {
            let old = versions(&before);
            for (id, version) in versions(&next) {
                if let Some(was) = old.get(&id) {
                    prop_assert!(version >= *was, "{} went from {} to {}", id, was, version);
                }
            }
            for node in &next.nodes {
                let same = before.nodes.iter().find(|n| n.id == node.id) == Some(node);
                if !same {
                    prop_assert!(node.kanso_updated >= 1, "{}", node.id);
                    if node.kanso_version > 3 || !before.nodes.iter().any(|n| n.id == node.id) {
                        prop_assert_eq!(node.kanso_updated, NOW, "{}", node.id);
                    }
                }
            }
        }
    }

    /// A version that isn't the item's is always refused, naming it, and a
    /// matching one never is for that reason.
    #[test]
    fn a_wrong_version_is_always_refused(
        id in prop::sample::select(vec!["t1", "g1", "far"]),
        off in 1u64..5,
        down in any::<bool>(),
    ) {
        let want = if down { 3u64.saturating_sub(off) } else { 3 + off };
        let ops = json!([{ "op": "moveNode", "id": id, "x": 2, "y": 2, "ifVersion": want }]);
        match run(&seed(), ops) {
            Err(Refusal::Stale(items)) => {
                prop_assert_eq!(items.len(), 1);
                prop_assert_eq!(&items[0].id, id);
                prop_assert_eq!(items[0].version, 3);
            }
            other => prop_assert!(false, "{:?}", other),
        }
        let ok = json!([{ "op": "moveNode", "id": id, "x": 2, "y": 2, "ifVersion": 3 }]);
        prop_assert!(run(&seed(), ok).is_ok());
    }

    /// Two agents that read the same canvas and both move the same card: one
    /// wins, the other is told it is stale, never both applied. (The answer to
    /// open question 1: the version check, not a repair, settles a race.)
    #[test]
    fn racing_moves_of_one_item_are_never_both_taken(
        a in (0.0f64..500.0, 0.0f64..500.0),
        b in (0.0f64..500.0, 0.0f64..500.0),
    ) {
        let read = seed();
        let take = |c: &Canvas, p: (f64, f64)| run(c, json!([
            { "op": "moveNode", "id": "t1", "x": p.0, "y": p.1, "ifVersion": 3 }
        ]));
        let (after_a, _) = take(&read, a).unwrap();
        // The second agent works from what it read, but the canvas moved on.
        prop_assert!(matches!(take(&after_a, b), Err(Refusal::Stale(_))));
    }

    /// The same with a stroke: after one move, the other agent's stroke move
    /// is refused (the stroke it read is gone), never a second copy.
    #[test]
    fn racing_stroke_moves_never_duplicate(dx in -100.0f64..100.0, dy in -100.0f64..100.0) {
        let read = seed();
        let mv = |c: &Canvas| run(c, json!([
            { "op": "moveStrokes", "node": "i1", "strokes": [{ "id": "s1", "ifVersion": 3 }], "dx": dx, "dy": dy + 1.0 }
        ]));
        let (after, _) = mv(&read).unwrap();
        prop_assert!(mv(&after).is_err());
        prop_assert_eq!(after.live_strokes().count(), 2);
    }
}

// ------------------------------------------------------------ hostile input

use super::batch::{MAX_BATCH_POINTS, MAX_BATCH_STROKES, MAX_FIELD_BYTES, MAX_LISTED};

#[test]
fn a_title_label_or_url_is_kept_far_under_one_room_update() {
    // Four bytes a character: 100,000 of them would be 400 KB in one field.
    let wide = "\u{1D11E}".repeat(MAX_FIELD_BYTES / 4 + 1);
    for op in [
        json!({ "op": "addNode", "type": "group", "x": 0, "y": 0, "label": wide }),
        json!({ "op": "addNode", "type": "link", "x": 0, "y": 0, "url": format!("https://x/{wide}") }),
        json!({ "op": "addNode", "type": "file", "x": 0, "y": 0, "file": "note:known", "title": wide }),
        json!({ "op": "renameGroup", "id": "g1", "label": wide }),
        json!({ "op": "updateEdge", "id": "e1", "label": wide }),
    ] {
        assert!(
            matches!(run(&seed(), json!([op])), Err(Refusal::Invalid(_))),
            "{op}"
        );
    }
    // Text goes to the room in pieces, so a long text card is fine.
    let long = "t".repeat(90_000);
    assert!(
        run(
            &seed(),
            json!([{ "op": "updateNode", "id": "t1", "text": long }])
        )
        .is_ok()
    );
    let fits = "l".repeat(MAX_FIELD_BYTES);
    assert!(
        run(
            &seed(),
            json!([{ "op": "renameGroup", "id": "g1", "label": fits }])
        )
        .is_ok()
    );
}

#[test]
fn a_batch_heavier_than_any_canvas_is_refused_before_it_is_worked() {
    let stroke = json!({ "points": [[0, 0, 0.5]] });
    let many = vec![stroke; MAX_BATCH_STROKES + 1];
    let err = Batch::from_json(
        &json!({ "ops": [{ "op": "addStrokes", "node": "i1", "strokes": many }] }),
    )
    .unwrap_err();
    assert!(err.contains("at most"), "{err}");
    let points = vec![json!([0, 0, 0.5]); MAX_BATCH_POINTS / 10 + 1];
    let strokes = vec![json!({ "points": points }); 10];
    assert!(
        Batch::from_json(
            &json!({ "ops": [{ "op": "addStrokes", "node": "i1", "strokes": strokes }] })
        )
        .is_err()
    );
    let ids = vec![json!("s1"); MAX_LISTED + 1];
    assert!(
        Batch::from_json(
            &json!({ "ops": [{ "op": "removeStrokes", "node": "i1", "strokes": ids }] })
        )
        .is_err()
    );
    // A batch made in Rust is held to the same.
    let heavy = Batch {
        if_rev: None,
        ops: vec![Op::AddStrokes {
            node: "i1".into(),
            strokes: vec![
                NewStroke {
                    tool: None,
                    color: None,
                    width: None,
                    points: vec![Point::new(0.0, 0.0, 0.5)],
                };
                MAX_BATCH_STROKES + 1
            ],
        }],
    };
    assert!(matches!(
        apply_batch(&seed(), &heavy, &ctx(false)),
        Err(Refusal::Full(_))
    ));
}

/// A number a hostile caller might send, NaN and infinities included (a
/// batch made in Rust can carry them even though JSON can't).
fn wild() -> impl Strategy<Value = f64> {
    prop_oneof![
        6 => -2000.0f64..2000.0,
        1 => Just(f64::NAN),
        1 => Just(f64::INFINITY),
        1 => Just(f64::NEG_INFINITY),
        1 => Just(f64::MAX),
        1 => Just(-0.0),
        1 => Just(1e7),
    ]
}

fn wild_id() -> impl Strategy<Value = String> {
    prop_oneof![
        4 => prop::sample::select(vec!["t1", "g1", "far", "i1", "e1", "s1"]).prop_map(str::to_owned),
        1 => prop::sample::select(vec!["", "ghost", "../../x", "a b", "\u{0}", "é"]).prop_map(str::to_owned),
        1 => "[a-z0-9_-]{60,70}",
    ]
}

fn wild_op() -> impl Strategy<Value = Op> {
    let text = prop_oneof![
        4 => Just(String::from("x")),
        1 => Just("\u{1D11E}".repeat(MAX_FIELD_BYTES)),
        1 => Just("y".repeat(100_001)),
    ];
    let point = (wild(), wild(), wild(), prop::option::of((wild(), wild()))).prop_map(
        |(x, y, pressure, tilt)| Point {
            x,
            y,
            pressure,
            tilt,
        },
    );
    prop_oneof![
        (
            prop::option::of(wild_id()),
            prop::sample::select(vec!["text", "group", "link", "ink", "file", "?"]),
            wild(),
            wild(),
            prop::option::of(wild()),
            text.clone()
        )
            .prop_map(|(id, kind, x, y, width, words)| Op::AddNode {
                id,
                kind: kind.into(),
                x,
                y,
                width,
                height: width,
                text: (kind == "text").then(|| words.clone()),
                file: (kind == "file").then(|| "note:known".into()),
                title: None,
                url: (kind == "link").then(|| format!("https://x/{words}")),
                label: (kind == "group").then_some(words),
                color: None,
            }),
        (wild_id(), wild(), wild()).prop_map(|(id, x, y)| Op::MoveNode {
            id,
            x,
            y,
            if_version: None
        }),
        (wild_id(), wild(), wild()).prop_map(|(id, width, height)| Op::ResizeNode {
            id,
            width,
            height,
            if_version: None
        }),
        (wild_id(), text.clone()).prop_map(|(id, t)| Op::UpdateNode {
            id,
            text: Some(t),
            title: None,
            color: None,
            label: None,
            if_version: None
        }),
        wild_id().prop_map(|id| Op::RemoveNode {
            id,
            if_version: None
        }),
        (wild_id(), wild_id()).prop_map(|(from_node, to_node)| Op::AddEdge {
            id: None,
            from_node,
            to_node,
            from_side: None,
            to_side: None,
            from_end: None,
            to_end: None,
            label: None,
            color: None,
        }),
        (prop::collection::vec(point, 0..40), wild()).prop_map(|(points, width)| Op::AddStrokes {
            node: "i1".into(),
            strokes: vec![NewStroke {
                tool: None,
                color: None,
                width: Some(width),
                points
            }],
        }),
        (wild(), wild()).prop_map(|(dx, dy)| Op::MoveStrokes {
            node: "i1".into(),
            strokes: vec![StrokeId::Plain("s1".into()), StrokeId::Plain("s1".into())],
            dx,
            dy,
        }),
        (wild_id(), prop::option::of(wild_id())).prop_map(|(n, group)| Op::SetGroup {
            nodes: vec![n],
            group
        }),
        text.prop_map(|title| Op::Rename { title }),
    ]
}

/// Every number in a canvas is a real one.
fn all_finite(c: &Canvas) -> bool {
    c.nodes.iter().filter(|n| !n.deleted).all(|n| {
        [n.x, n.y, n.width, n.height].iter().all(|v| v.is_finite())
            && n.strokes.iter().filter(|s| !s.deleted).all(|s| {
                s.width.is_finite()
                    && s.points.iter().all(|p| {
                        p.x.is_finite()
                            && p.y.is_finite()
                            && p.pressure.is_finite()
                            && p.tilt.is_none_or(|(a, b)| a.is_finite() && b.is_finite())
                    })
            })
    })
}

proptest! {
    #![proptest_config(ProptestConfig {
        cases: 512,
        rng_seed: proptest::test_runner::RngSeed::Fixed(20_261_017),
        failure_persistence: None,
        ..ProptestConfig::default()
    })]

    /// Hostile batches, NaN and infinities among them, on a private canvas
    /// and a shared one: nothing panics; a refused batch changes nothing;
    /// what is taken is a valid canvas under the budget with only real
    /// numbers, and no title, label or URL long enough to choke the room.
    #[test]
    fn hostile_batches_never_panic_or_make_a_bad_canvas(
        ops in prop::collection::vec(wild_op(), 1..8),
        shared in any::<bool>(),
    ) {
        let before = seed();
        let batch = Batch { if_rev: None, ops };
        match apply_batch(&before, &batch, &ctx(shared)) {
            Err(_) => prop_assert_eq!(&before, &seed()),
            Ok((next, made)) => {
                prop_assert_eq!(next.validate(), Ok(()));
                prop_assert!(all_finite(&next));
                prop_assert!(to_body(&next).len() < MAX_BYTES);
                for node in next.nodes.iter().filter(|n| !n.deleted) {
                    for field in [&node.title, &node.label, &node.url].into_iter().flatten() {
                        prop_assert!(field.len() <= MAX_FIELD_BYTES + 16);
                    }
                }
                if let Some(title) = made.title {
                    prop_assert!(title.chars().count() <= 500);
                }
            }
        }
    }
}
