use std::collections::{HashMap, HashSet};

use proptest::prelude::*;
use proptest::test_runner::FileFailurePersistence;
use serde_json::{Map, Value};

use super::*;

fn config(cases: u32) -> ProptestConfig {
    ProptestConfig {
        cases,
        failure_persistence: Some(Box::new(FileFailurePersistence::WithSource("regressions"))),
        ..ProptestConfig::default()
    }
}

fn side() -> impl Strategy<Value = Option<String>> {
    prop_oneof![
        Just(None),
        Just(Some("top".into())),
        Just(Some("right".into())),
        Just(Some("bottom".into())),
        Just(Some("left".into())),
    ]
}

fn extra() -> impl Strategy<Value = Map<String, Value>> {
    prop::collection::btree_map("zz[a-zA-Z0-9]{0,6}", 0i64..1000, 0..3)
        .prop_map(|map| map.into_iter().map(|(k, v)| (k, Value::from(v))).collect())
}

fn arb_point() -> impl Strategy<Value = Point> {
    (
        -5000i32..5000,
        -5000i32..5000,
        0u32..=1000,
        prop::option::of((0u32..=6283, 0u32..=1571)),
    )
        .prop_map(|(x, y, p, tilt)| Point {
            x: f64::from(x) / 10.0,
            y: f64::from(y) / 10.0,
            pressure: f64::from(p) / 1000.0,
            tilt: tilt.map(|(a, b)| (f64::from(a) / 1000.0, f64::from(b) / 1000.0)),
        })
}

fn arb_stroke(id: String) -> impl Strategy<Value = Stroke> {
    (
        prop::collection::vec(arb_point(), 1..30),
        0usize..4,
        1u32..80,
        "#[0-9a-f]{6}",
    )
        .prop_map(move |(points, tool, width, color)| Stroke {
            id: id.clone(),
            tool: TOOLS[tool].into(),
            color,
            width: f64::from(width) / 10.0,
            points,
            kanso_version: 1,
            ..Stroke::default()
        })
}

fn arb_node(ix: usize) -> impl Strategy<Value = Node> {
    (
        0u8..5,
        -2000i64..2000,
        -2000i64..2000,
        20i64..600,
        20i64..600,
        prop_oneof![
            Just(None),
            Just(Some("1".into())),
            Just(Some("#a1b2c3".into()))
        ],
        "(\n|[a-zA-Z0-9 ]){0,40}",
        "[a-zA-Z0-9 ]{0,20}",
        extra(),
        prop::collection::vec(0u8..1, 0..4),
        1u64..9,
    )
        .prop_flat_map(
            move |(kind, x, y, width, height, color, text, title, extra, strokes, version)| {
                let kind = ["text", "file", "link", "group", "ink"][kind as usize];
                let strokes: Vec<_> = (0..strokes.len())
                    .map(|s| arb_stroke(format!("n{ix}s{s}")))
                    .collect();
                strokes.prop_map(move |strokes| Node {
                    id: format!("n{ix}"),
                    kind: kind.into(),
                    x: x as f64,
                    y: y as f64,
                    width: width as f64,
                    height: height as f64,
                    color: color.clone(),
                    text: (kind == "text").then(|| text.clone()),
                    file: (kind == "file").then(|| format!("note:n{ix}")),
                    url: (kind == "link").then(|| "https://example.com".into()),
                    label: (kind == "group").then(|| title.clone()),
                    title: (kind == "file").then(|| title.clone()),
                    strokes: if kind == "ink" { strokes } else { Vec::new() },
                    kanso_version: version,
                    kanso_updated: 1_000 + version as i64,
                    kanso_z: ix as f64,
                    extra: extra.clone(),
                    ..Node::default()
                })
            },
        )
}

fn arb_canvas() -> impl Strategy<Value = Canvas> {
    (1usize..10).prop_flat_map(|n| {
        let nodes: Vec<_> = (0..n).map(arb_node).collect();
        (
            nodes,
            prop::collection::vec((0usize..12, 0usize..12, side(), side(), extra()), 0..16),
            extra(),
        )
            .prop_map(|(nodes, raw_edges, extra)| {
                let ids: Vec<String> = nodes.iter().map(|n| n.id.clone()).collect();
                let edges: Vec<Edge> = raw_edges
                    .into_iter()
                    .enumerate()
                    .map(|(ix, (from, to, from_side, to_side, extra))| Edge {
                        id: format!("e{ix}"),
                        from_node: ids[from % ids.len()].clone(),
                        to_node: ids[to % ids.len()].clone(),
                        from_side,
                        from_end: Some("arrow".into()),
                        to_side,
                        kanso_version: 1,
                        kanso_z: ix as f64,
                        extra,
                        ..Edge::default()
                    })
                    .collect();
                Canvas {
                    nodes,
                    edges,
                    extra,
                    ..Canvas::default()
                }
            })
    })
}

/// An edit one device makes.
#[derive(Clone, Debug)]
enum Op {
    Draw(usize, Vec<Point>),
    Undraw(usize, usize),
    Erase(usize, usize, Vec<(i32, i32)>, u32),
    Move(usize, i32, i32),
    Delete(usize),
    DeleteEdge,
}

fn arb_op() -> impl Strategy<Value = Op> {
    prop_oneof![
        (0usize..4, prop::collection::vec(arb_point(), 1..12)).prop_map(|(n, p)| Op::Draw(n, p)),
        (0usize..4, 0usize..8).prop_map(|(n, s)| Op::Undraw(n, s)),
        (
            0usize..4,
            0usize..8,
            prop::collection::vec((-500i32..500, -500i32..500), 1..4),
            1u32..120
        )
            .prop_map(|(n, s, p, r)| Op::Erase(n, s, p, r)),
        (0usize..4, -900i32..900, -900i32..900).prop_map(|(n, x, y)| Op::Move(n, x, y)),
        (0usize..4).prop_map(Op::Delete),
        Just(Op::DeleteEdge),
    ]
}

/// Two ink layers, two cards and an arrow, with a few strokes already on the
/// first layer: what two devices start from.
fn base() -> Canvas {
    let mut c = Canvas::default();
    let ink = |id: &str, x: f64| Node {
        id: id.into(),
        kind: INK.into(),
        x,
        y: 0.0,
        width: 100.0,
        height: 100.0,
        ..Node::default()
    };
    let card = |id: &str, x: f64| Node {
        id: id.into(),
        kind: "text".into(),
        text: Some(format!("card {id}")),
        x,
        y: 300.0,
        width: 100.0,
        height: 50.0,
        ..Node::default()
    };
    c.put_node(ink("i1", 0.0), 1);
    c.put_node(ink("i2", 500.0), 1);
    c.put_node(card("t1", 0.0), 1);
    c.put_node(card("t2", 200.0), 1);
    c.put_edge(
        Edge {
            id: "e1".into(),
            from_node: "t1".into(),
            to_node: "t2".into(),
            ..Edge::default()
        },
        1,
    );
    for s in 0..3 {
        let stroke = Stroke {
            id: format!("base{s}"),
            tool: "pen".into(),
            color: "#000000".into(),
            width: 2.0,
            points: (0..6)
                .map(|i| Point::new(f64::from(i) * 10.0, f64::from(s) * 20.0, 0.5))
                .collect(),
            ..Stroke::default()
        };
        assert!(c.add_stroke("i1", stroke, 1));
    }
    c
}

const NODES: [&str; 4] = ["i1", "i2", "t1", "t2"];

/// One device's edits, its strokes named after `who` so two devices never
/// make the same ID.
fn edit(mut c: Canvas, who: &str, ops: &[Op], clock: i64) -> Canvas {
    for (n, op) in ops.iter().enumerate() {
        let now = clock + n as i64;
        match op {
            Op::Draw(node, points) => {
                let stroke = Stroke {
                    id: format!("{who}{n}"),
                    tool: "pen".into(),
                    color: "#111111".into(),
                    width: 2.0,
                    points: points.clone(),
                    ..Stroke::default()
                };
                c.add_stroke(NODES[*node % 2], stroke, now);
            }
            Op::Undraw(node, s) => {
                if let Some(id) = pick_stroke(&c, NODES[*node % 2], *s) {
                    c.remove_stroke(NODES[*node % 2], &id, now);
                }
            }
            Op::Erase(node, s, path, r) => {
                if let Some(id) = pick_stroke(&c, NODES[*node % 2], *s) {
                    let eraser = Eraser {
                        path: path
                            .iter()
                            .map(|(x, y)| (f64::from(*x), f64::from(*y)))
                            .collect(),
                        radius: f64::from(*r),
                    };
                    c.erase_partial(NODES[*node % 2], &id, &eraser, now);
                }
            }
            Op::Move(node, x, y) => {
                c.move_node(NODES[*node], f64::from(*x), f64::from(*y), now);
            }
            Op::Delete(node) => {
                c.remove_node(NODES[*node], now);
            }
            Op::DeleteEdge => {
                c.remove_edge("e1", now);
            }
        }
    }
    c
}

fn pick_stroke(c: &Canvas, node: &str, ix: usize) -> Option<String> {
    let live: Vec<&Stroke> = c
        .node(node)?
        .strokes
        .iter()
        .filter(|s| !s.deleted)
        .collect();
    (!live.is_empty()).then(|| live[ix % live.len()].id.clone())
}

fn live_strokes(c: &Canvas) -> HashSet<(String, String)> {
    c.nodes
        .iter()
        .filter(|n| !n.deleted)
        .flat_map(|n| {
            n.strokes
                .iter()
                .filter(|s| !s.deleted)
                .map(|s| (n.id.clone(), s.id.clone()))
        })
        .collect()
}

proptest! {
    #![proptest_config(config(64))]

    /// Writing a canvas into a note and reading it back gives the same
    /// canvas, ink, unknown fields and all.
    #[test]
    fn prop_round_trip(canvas in arb_canvas()) {
        let body = to_body(&canvas);
        prop_assert!(parse(&body).is_ok(), "{:?}", parse(&body).err());
        prop_assert_eq!(parse(&body).unwrap(), canvas);
    }

    /// The stored JSON is compact: one line, no pretty-printing.
    #[test]
    fn prop_body_is_compact(canvas in arb_canvas()) {
        let body = to_body(&canvas);
        let json = fenced(&body).unwrap();
        prop_assert!(!json.contains('\n'));
        prop_assert!(!json.contains(": "));
    }

    /// A fence is found only when it is the note's own, opening and
    /// closing on lines of their own.
    #[test]
    fn prop_only_a_canvas_fence_is_found(text in "(\n|[a-zA-Z0-9`{}]){0,200}") {
        let bare = text.trim();
        let is_fence = bare.starts_with("```canvas\n") && bare.ends_with("```");
        prop_assert_eq!(fenced(&text).is_some(), is_fence);
    }

    /// The repaired canvas keeps every card and every arrow that still
    /// joins two of them, with one unique ID each.
    #[test]
    fn prop_repaired_keeps_cards(mut canvas in arb_canvas(), ids in prop::collection::vec("[a-z]{0,4}", 0..8)) {
        for (ix, id) in ids.iter().enumerate() {
            if let Some(node) = canvas.nodes.get_mut(ix) {
                node.id = id.clone();
            }
        }
        canvas = canvas.repaired();
        let node_ids: HashSet<&str> = canvas.nodes.iter().map(|n| n.id.as_str()).collect();
        prop_assert_eq!(node_ids.len(), canvas.nodes.len());
        for edge in &canvas.edges {
            prop_assert!(node_ids.contains(edge.from_node.as_str()));
            prop_assert!(node_ids.contains(edge.to_node.as_str()));
        }
    }

    /// A cards-only Obsidian export drops ink, keeps every other card in
    /// stacking order, never writes Kanso's bookkeeping, and says which
    /// note each card is.
    #[test]
    fn prop_obsidian_export_keeps_the_cards(canvas in arb_canvas()) {
        let out = canvas.to_obsidian();
        let cards: Vec<&Node> = canvas.live_nodes().filter(|n| n.kind != INK).collect();
        prop_assert_eq!(out.nodes.len(), cards.len());
        for (before, after) in cards.iter().zip(&out.nodes) {
            prop_assert_eq!(&before.id, &after.id);
            prop_assert!(after.strokes.is_empty());
            prop_assert_eq!(after.kanso_version, 0);
            if let Some(id) = Canvas::note_id(before) {
                prop_assert_eq!(after.extra.get("kansoId").and_then(Value::as_str), Some(id));
                prop_assert!(after.file.as_deref().is_some_and(|f| f.ends_with(".md")));
            } else {
                prop_assert_eq!(after.file.as_deref(), before.file.as_deref());
            }
        }
        let ids: HashSet<&str> = out.nodes.iter().map(|n| n.id.as_str()).collect();
        for edge in &out.edges {
            prop_assert!(ids.contains(edge.from_node.as_str()) && ids.contains(edge.to_node.as_str()));
        }
    }

    /// The exported file has whole-number geometry, ink or not.
    #[test]
    fn prop_obsidian_geometry_is_integer(canvas in arb_canvas()) {
        let files: HashMap<String, String> = canvas
            .nodes
            .iter()
            .filter(|n| n.kind == INK)
            .map(|n| (n.id.clone(), format!("{}.png", n.id)))
            .collect();
        let text = canvas.to_obsidian_json(&files);
        let value: Value = serde_json::from_str(&text).unwrap();
        for node in value["nodes"].as_array().unwrap() {
            for key in ["x", "y", "width", "height"] {
                prop_assert!(node[key].is_i64(), "{key} in {node}");
            }
        }
    }

    /// Import maps the paths it knows to note IDs and titles, and
    /// leaves the rest as they were.
    #[test]
    fn prop_paths_map_only_what_the_import_knows(
        paths in prop::collection::btree_map("[A-Za-z]{1,8}", ("note:[a-z0-9]{4}", "[A-Za-z ]{1,8}"), 0..6)
    ) {
        let paths: HashMap<String, (String, String)> = paths.into_iter().collect();
        let mut canvas = Canvas::default();
        for (ix, path) in paths.keys().enumerate() {
            canvas.nodes.push(Node {
                id: format!("n{ix}"),
                kind: "file".into(),
                width: 200.,
                height: 100.,
                file: Some(path.clone()),
                ..Node::default()
            });
        }
        canvas.nodes.push(Node {
            id: "gone".into(),
            kind: "file".into(),
            width: 1.,
            height: 1.,
            file: Some("Not There.md".into()),
            ..Node::default()
        });
        canvas.map_paths(&paths);
        for node in &canvas.nodes {
            match paths.get(node.file.as_deref().unwrap_or_default()) {
                Some((id, _)) => prop_assert_eq!(node.file.as_deref(), Some(id.as_str())),
                None => prop_assert!(
                    node.file.as_deref() == Some("Not There.md")
                        || node.file.as_deref().is_some_and(|f| f.starts_with("note:"))
                ),
            }
        }
    }

    /// Whatever sequence of edits the app or the API makes, the canvas is
    /// still valid, versions never go down, and a tombstone stays one.
    #[test]
    fn prop_edits_keep_a_valid_canvas(ops in prop::collection::vec(arb_op(), 0..24)) {
        let before = base();
        let after = edit(before.clone(), "a", &ops, 100);
        prop_assert!(after.validate().is_ok(), "{:?}", after.validate());
        prop_assert!(parse(&to_body(&after)).is_ok());
        for old in &before.nodes {
            let new = after.nodes.iter().find(|n| n.id == old.id).unwrap();
            prop_assert!(new.kanso_version >= old.kanso_version);
            prop_assert!(!old.deleted || new.deleted);
            for os in &old.strokes {
                let ns = new.strokes.iter().find(|s| s.id == os.id);
                if new.deleted {
                    continue;
                }
                let ns = ns.unwrap();
                prop_assert!(ns.kanso_version >= os.kanso_version);
                prop_assert!(!os.deleted || ns.deleted);
                // Points are never rewritten.
                prop_assert!(ns.deleted || ns.points == os.points);
            }
        }
    }

    /// Merging two devices' edits is commutative, and merging the result
    /// again, with itself or with either side, changes nothing.
    #[test]
    fn prop_merge_is_commutative_and_idempotent(
        common in prop::collection::vec(arb_op(), 0..6),
        a in prop::collection::vec(arb_op(), 0..12),
        b in prop::collection::vec(arb_op(), 0..12),
    ) {
        let start = edit(base(), "c", &common, 10);
        let left = edit(start.clone(), "a", &a, 100);
        let right = edit(start, "b", &b, 200);
        let m = merge(&left, &right);
        prop_assert_eq!(&m, &merge(&right, &left));
        prop_assert_eq!(&m, &merge(&m, &m));
        prop_assert_eq!(&m, &merge(&m, &left));
        prop_assert_eq!(&m, &merge(&right, &m));
        prop_assert!(m.validate().is_ok(), "{:?}", m.validate());
    }

    /// No stroke that is not tombstoned is lost: a stroke live on one side
    /// stays live unless the other side deleted that very stroke (or its
    /// layer), and a deleted stroke never comes back.
    #[test]
    fn prop_merge_keeps_every_live_stroke(
        a in prop::collection::vec(arb_op(), 0..12),
        b in prop::collection::vec(arb_op(), 0..12),
    ) {
        let left = edit(base(), "a", &a, 100);
        let right = edit(base(), "b", &b, 200);
        let m = merge(&left, &right);
        let merged = live_strokes(&m);
        let dead_in = |c: &Canvas, node: &str, stroke: &str| {
            c.nodes.iter().find(|n| n.id == node).is_none_or(|n| {
                n.deleted || n.strokes.iter().any(|s| s.id == stroke && s.deleted)
            })
        };
        for (side, other) in [(&left, &right), (&right, &left)] {
            for (node, stroke) in live_strokes(side) {
                let killed = dead_in(other, &node, &stroke) && other.nodes.iter().any(|n| n.id == node && (n.deleted || n.strokes.iter().any(|s| s.id == stroke)));
                if !killed {
                    prop_assert!(merged.contains(&(node.clone(), stroke.clone())), "{node}/{stroke} lost");
                }
            }
        }
        // Nothing is invented.
        let known: HashSet<(String, String)> = live_strokes(&left).union(&live_strokes(&right)).cloned().collect();
        prop_assert!(merged.is_subset(&known));
    }

    /// The partial eraser: nothing left under the eraser, every original
    /// point it did not reach is kept verbatim and in order, and the cut
    /// lands on the eraser's edge.
    #[test]
    fn prop_split_leaves_nothing_under_the_eraser(
        stroke in arb_stroke("s".into()),
        path in prop::collection::vec((-500i32..500, -500i32..500), 1..5),
        radius in 1u32..200,
    ) {
        let eraser = Eraser {
            path: path.iter().map(|(x, y)| (f64::from(*x), f64::from(*y))).collect(),
            radius: f64::from(radius),
        };
        let mut n = 0;
        let split = split_stroke(&stroke, &eraser, || { n += 1; format!("p{n}") });
        match split {
            Split::Untouched => {
                prop_assert!(stroke.points.iter().all(|p| !covers(&eraser, p.x, p.y)));
            }
            Split::Pieces(pieces) => {
                let mut kept: Vec<Point> = Vec::new();
                for piece in &pieces {
                    prop_assert!(piece.points.len() >= 2);
                    prop_assert_eq!(&piece.tool, &stroke.tool);
                    prop_assert_eq!(piece.width, stroke.width);
                    kept.extend(piece.points.iter().copied());
                }
                // Every original point outside the eraser survives verbatim.
                for p in stroke.points.iter().filter(|p| !covers(&eraser, p.x, p.y)) {
                    prop_assert!(kept.contains(p), "{p:?} lost");
                }
                // Pieces never invent a point inside the eraser's body.
                for p in &kept {
                    prop_assert!(!covers(&shrunk(&eraser, -0.05), p.x, p.y), "{p:?} under the eraser");
                }
            }
        }
    }

    /// An ink node's bounds hold all its live strokes, and the canvas's
    /// bounds hold every live node.
    #[test]
    fn prop_bounds_hold_the_content(canvas in arb_canvas()) {
        match canvas.content_bounds() {
            None => prop_assert!(canvas.nodes.iter().all(|n| n.deleted)),
            Some(all) => {
                for node in canvas.nodes.iter().filter(|n| !n.deleted) {
                    let b = node.bounds();
                    prop_assert!(b.x >= all.x - 1e-9 && b.right() <= all.right() + 1e-9);
                    prop_assert!(b.y >= all.y - 1e-9 && b.bottom() <= all.bottom() + 1e-9);
                    if node.kind == INK {
                        for s in node.strokes.iter().filter(|s| !s.deleted).filter_map(Stroke::bounds) {
                            prop_assert!(b.x - 1e-9 <= node.x + s.x && node.x + s.right() <= b.right() + 1e-9);
                        }
                    }
                }
            }
        }
    }

    /// The summary is capped and the search text never holds JSON keys.
    #[test]
    fn prop_text_is_capped_and_plain(canvas in arb_canvas()) {
        prop_assert!(canvas.summary().chars().count() <= SUMMARY_CAP);
        let words = canvas.search_text();
        prop_assert!(!words.contains("\"nodes\"") && !words.contains("kansoVersion"));
    }

    /// Replacing a sketch by its ID touches that fence and no other.
    #[test]
    fn prop_sketch_replace_touches_only_its_fence(which in 0usize..3, n in 1u8..9) {
        let sketches: Vec<Canvas> = (0..3).map(|i| sketch::new_sketch(&format!("sk-{i}"), 320, 240)).collect();
        let body: String = sketches
            .iter()
            .map(|s| format!("Text before\n\n```sketch\n{}\n```\n\n", serde_json::to_string(s).unwrap()))
            .collect();
        let mut edited = sketches[which].clone();
        edited.kanso_ink_ocr.push(InkOcr { node: "i1".into(), md5: "x".into(), engine: "vision".into(), text: format!("word{n}") });
        let out = sketch::replace(&body, &format!("sk-{which}"), &edited).unwrap();
        let found = sketch::find(&out);
        prop_assert_eq!(found.len(), 3);
        for (i, f) in found.iter().enumerate() {
            let c: Canvas = serde_json::from_str(&out[f.json.clone()]).unwrap();
            if i == which {
                prop_assert_eq!(c, edited.clone());
            } else {
                prop_assert_eq!(c, sketches[i].clone());
            }
        }
        prop_assert!(sketch::replace(&body, "sk-missing", &edited).is_none());
        let index = sketch::index_text(&out);
        let expected = format!("word{n}");
        prop_assert!(index.contains(&expected));
        prop_assert!(!index.contains("kansoSketch"));
    }
}

fn shrunk(e: &Eraser, by: f64) -> Eraser {
    Eraser {
        path: e.path.clone(),
        radius: e.radius + by,
    }
}

fn pen(id: &str, pts: &[(f64, f64, f64)]) -> Stroke {
    Stroke {
        id: id.into(),
        tool: "pen".into(),
        color: "#000".into(),
        width: 2.0,
        points: pts.iter().map(|&(x, y, p)| Point::new(x, y, p)).collect(),
        ..Stroke::default()
    }
}

#[test]
fn a_canvas_needs_its_fence() {
    assert!(fenced("Intro\n\n```canvas\n{\"nodes\":[]}\n```\n").is_none());
    assert_eq!(fenced("```canvas\n{}\n```"), Some("{}"));
    assert_eq!(fenced("  ```canvas  \n{}\n```  "), Some("{}"));
    assert!(fenced("```json\n{}\n```").is_none());
    assert!(fenced("```canvas\n{}\n```\ntrailing").is_none());
    assert!(parse("```canvas\nnot json\n```").is_err());
}

#[test]
fn a_canvas_with_a_missing_id_is_repaired() {
    let canvas: Canvas = serde_json::from_str(
        r#"{"nodes":[{"id":"a","type":"text","text":"hi","x":0,"y":0,"width":10,"height":10},
                      {"id":"a","type":"text","x":5,"y":5,"width":10,"height":10}],
            "edges":[{"id":"e","fromNode":"a","toNode":"a"},
                     {"id":"e","fromNode":"a","toNode":"gone"}]}"#,
    )
    .unwrap();
    let repaired = canvas.repaired();
    assert_eq!(repaired.nodes.len(), 2);
    assert_eq!(repaired.edges.len(), 1);
    assert!(repaired.validate().is_ok());
}

#[test]
fn export_paths_are_safe() {
    assert_eq!(export_path("A/B: C"), "A-B- C.md");
    assert_eq!(export_path("   "), "Untitled.md");
    assert_eq!(export_path("Note"), "Note.md");
}

#[test]
fn the_documented_example_reads() {
    let json = r##"{"nodes":[
      {"id":"n1","type":"text","x":0,"y":0,"width":320,"height":120,"text":"Launch plan","color":"4","kansoVersion":3},
      {"id":"i1","type":"ink","x":-40,"y":180,"width":900,"height":400,"kansoVersion":7,
       "strokes":[{"id":"s1","tool":"pen","color":"#1e1e1e","width":2.2,"kansoVersion":2,
                   "points":[[0,0,0.42],[4.5,1.2,0.51],[9.1,3.8,0.47]]},
                  {"id":"s2","tool":"pencil","color":"#000","width":1.5,
                   "points":[[0,0,0.4,1.2,0.8],[3,3,0.5,1.2,0.7]]},
                  {"id":"s3","deleted":true,"kansoVersion":4}]}],
      "edges":[]}"##;
    let body = format!("```canvas\n{json}\n```");
    let canvas = parse(&body).unwrap();
    let ink = canvas.node("i1").unwrap();
    assert_eq!(ink.strokes[0].points[1], Point::new(4.5, 1.2, 0.51));
    assert_eq!(ink.strokes[1].points[0].tilt, Some((1.2, 0.8)));
    assert!(ink.strokes[2].deleted);
    // Five-number points write back as five numbers.
    let again = to_body(&canvas);
    assert!(again.contains("[0.0,0.0,0.4,1.2,0.8]"), "{again}");
}

#[test]
fn a_bad_stroke_is_refused() {
    let with = |stroke: Stroke| {
        let mut c = Canvas::default();
        c.put_node(
            Node {
                id: "i".into(),
                kind: INK.into(),
                width: 10.,
                height: 10.,
                ..Node::default()
            },
            1,
        );
        c.add_stroke("i", stroke, 1);
        c
    };
    assert!(with(pen("a", &[(0., 0., 0.5)])).validate().is_ok());
    assert!(with(pen("a", &[(f64::NAN, 0., 0.5)])).validate().is_err());
    assert!(with(pen("a", &[(0., 0., 1.5)])).validate().is_err());
    assert!(with(pen("a", &[])).validate().is_err());
    let mut unknown = pen("a", &[(0., 0., 0.5)]);
    unknown.tool = "laser".into();
    assert!(with(unknown).validate().is_err());
    assert!(serde_json::from_str::<Point>("[1]").is_err());
    assert_eq!(
        serde_json::from_str::<Point>("[1,2]").unwrap().pressure,
        0.5
    );
}

#[test]
fn a_tombstone_wins_over_a_live_item_of_the_same_version() {
    let mut a = base();
    let mut b = a.clone();
    a.remove_stroke("i1", "base0", 50);
    // The other device edits a different stroke's layer concurrently.
    b.move_node("i1", 9.0, 9.0, 50);
    let m = merge(&a, &b);
    let ink = m.node("i1").unwrap();
    assert!(
        ink.strokes
            .iter()
            .find(|s| s.id == "base0")
            .unwrap()
            .deleted
    );
    assert_eq!(ink.strokes.iter().filter(|s| !s.deleted).count(), 2);
}

#[test]
fn ink_added_apart_is_united() {
    let mut a = base();
    let mut b = base();
    a.add_stroke("i1", pen("fromA", &[(0., 0., 0.5), (1., 1., 0.5)]), 10);
    b.add_stroke("i1", pen("fromB", &[(5., 5., 0.5), (6., 6., 0.5)]), 11);
    let ids: Vec<_> = merge(&a, &b)
        .node("i1")
        .unwrap()
        .strokes
        .iter()
        .map(|s| s.id.clone())
        .collect();
    for want in ["base0", "base1", "base2", "fromA", "fromB"] {
        assert!(ids.iter().any(|i| i == want), "{want} missing from {ids:?}");
    }
}

#[test]
fn an_arrow_to_a_deleted_card_is_deleted_by_the_merge() {
    let mut a = base();
    let b = base();
    // Remove the card but keep the arrow live, as a stale writer might.
    a.nodes.iter_mut().find(|n| n.id == "t2").unwrap().deleted = true;
    a.nodes
        .iter_mut()
        .find(|n| n.id == "t2")
        .unwrap()
        .kanso_version = 5;
    let m = merge(&a, &b);
    assert!(m.edges.iter().find(|e| e.id == "e1").unwrap().deleted);
    assert!(m.validate().is_ok());
}

#[test]
fn bodies_merge_only_when_both_are_canvases() {
    let (a, b) = (to_body(&base()), to_body(&base()));
    assert!(merge_bodies(&a, &b).is_some());
    assert!(merge_bodies(&a, "plain text").is_none());
    assert!(merge_bodies("```canvas\nnope\n```", &b).is_none());
}

#[test]
fn the_eraser_cuts_exactly_at_its_edge() {
    // A horizontal stroke 0..100; a disc of radius 10 at x=50 removes 40..60.
    let stroke = pen("s", &[(0., 0., 0.2), (100., 0., 0.8)]);
    let eraser = Eraser {
        path: vec![(50., 0.)],
        radius: 10.,
    };
    let mut n = 0;
    let Split::Pieces(pieces) = split_stroke(&stroke, &eraser, || {
        n += 1;
        format!("p{n}")
    }) else {
        panic!("the eraser missed");
    };
    assert_eq!(pieces.len(), 2);
    assert_eq!(pieces[0].points.last().unwrap().x, 40.0);
    assert_eq!(pieces[1].points[0].x, 60.0);
    // Pressure is interpolated between the neighbouring samples.
    assert_eq!(pieces[0].points.last().unwrap().pressure, 0.2 + 0.6 * 0.4);
    assert_eq!(pieces[0].points[0], stroke.points[0]);
    assert_eq!(pieces[1].points.last(), stroke.points.last());
    assert_eq!(pieces[0].id, "p1");
}

#[test]
fn the_eraser_can_take_a_whole_stroke_or_miss() {
    let stroke = pen("s", &[(0., 0., 0.5), (4., 0., 0.5)]);
    let big = Eraser {
        path: vec![(2., 0.)],
        radius: 50.,
    };
    assert_eq!(
        split_stroke(&stroke, &big, String::new),
        Split::Pieces(Vec::new())
    );
    let far = Eraser {
        path: vec![(2., 90.), (9., 90.)],
        radius: 5.,
    };
    assert_eq!(split_stroke(&stroke, &far, String::new), Split::Untouched);
    let dot = pen("d", &[(1., 1., 0.5)]);
    assert_eq!(
        split_stroke(&dot, &big, String::new),
        Split::Pieces(Vec::new())
    );
}

#[test]
fn erasing_in_a_canvas_is_one_edit() {
    let mut c = base();
    let eraser = Eraser {
        path: vec![(25., 0.)],
        radius: 8.,
    };
    let ids = c.erase_partial("i1", "base0", &eraser, 77).unwrap();
    assert_eq!(ids.len(), 2);
    let ink = c.node("i1").unwrap();
    assert!(
        ink.strokes
            .iter()
            .find(|s| s.id == "base0")
            .unwrap()
            .deleted
    );
    for id in &ids {
        let s = ink.strokes.iter().find(|s| &s.id == id).unwrap();
        assert_eq!((s.kanso_version, s.kanso_updated), (1, 77));
    }
    assert!(c.validate().is_ok());
    assert!(c.erase_partial("i1", "base0", &eraser, 78).is_none());
}

#[test]
fn bounds_follow_the_ink() {
    let mut c = Canvas::default();
    c.put_node(
        Node {
            id: "i".into(),
            kind: INK.into(),
            x: 100.,
            y: 200.,
            width: 5.,
            height: 5.,
            ..Node::default()
        },
        1,
    );
    assert_eq!(
        c.content_bounds().unwrap(),
        Bounds {
            x: 100.,
            y: 200.,
            width: 5.,
            height: 5.
        }
    );
    c.add_stroke("i", pen("s", &[(0., 0., 0.5), (50., 30., 0.5)]), 2);
    let b = c.content_bounds().unwrap();
    assert_eq!((b.x, b.y, b.width, b.height), (99., 199., 52., 32.));
    c.refit_ink();
    assert_eq!(
        (c.node("i").unwrap().width, c.node("i").unwrap().height),
        (51., 31.)
    );
    c.put_node(
        Node {
            id: "g".into(),
            kind: "group".into(),
            x: 0.,
            y: 0.,
            width: 400.,
            height: 400.,
            ..Node::default()
        },
        3,
    );
    assert_eq!(c.group_bounds("g").unwrap().width, 400.);
    assert!(c.group_bounds("i").is_none());
    let inside: Vec<_> = c.nodes_in_group("g").iter().map(|n| n.id.clone()).collect();
    assert_eq!(inside, vec!["i"]);
}

#[test]
fn summary_and_search_text_read_the_words() {
    let mut c = Canvas::default();
    c.put_node(
        Node {
            id: "t".into(),
            kind: "text".into(),
            text: Some("Launch\nplan".into()),
            width: 10.,
            height: 10.,
            ..Node::default()
        },
        1,
    );
    c.put_node(
        Node {
            id: "f".into(),
            kind: "file".into(),
            file: Some("note:abc".into()),
            title: Some("Q4 budget".into()),
            width: 10.,
            height: 10.,
            ..Node::default()
        },
        1,
    );
    c.put_node(
        Node {
            id: "i".into(),
            kind: INK.into(),
            width: 10.,
            height: 10.,
            ..Node::default()
        },
        1,
    );
    c.put_node(
        Node {
            id: "gone".into(),
            kind: "text".into(),
            text: Some("secret".into()),
            width: 10.,
            height: 10.,
            ..Node::default()
        },
        1,
    );
    c.remove_node("gone", 2);
    c.kanso_ink_ocr.push(InkOcr {
        node: "i".into(),
        md5: "m".into(),
        engine: "vision".into(),
        text: "buy milk".into(),
    });
    let summary = c.summary();
    assert!(summary.contains("- Launch plan") && summary.contains("- Card: Q4 budget"));
    assert!(summary.contains("- Handwriting: buy milk") && !summary.contains("secret"));
    assert_eq!(c.search_text(), "Launch\nplan\nQ4 budget\nbuy milk");
    let mut long = Canvas::default();
    long.put_node(
        Node {
            id: "t".into(),
            kind: "text".into(),
            text: Some("x".repeat(9000)),
            width: 1.,
            height: 1.,
            ..Node::default()
        },
        1,
    );
    assert_eq!(long.summary().chars().count(), SUMMARY_CAP);
}

#[test]
fn the_budget_has_two_lines() {
    assert_eq!(budget(0), Budget::Ok);
    assert_eq!(budget(WARN_BYTES - 1), Budget::Ok);
    assert_eq!(budget(WARN_BYTES), Budget::Warn);
    assert_eq!(budget(MAX_BYTES), Budget::Full);
}

#[test]
fn the_store_check_accepts_empty_and_refuses_junk() {
    assert!(check_body("").is_ok());
    assert!(check_body(&to_body(&base())).is_ok());
    assert!(check_body("# Not a canvas").is_err());
    assert!(check_body("```canvas\n{\"nodes\":[{\"id\":\"a\"}]}\n```").is_err());
    let huge = format!("```canvas\n{}\n```", " ".repeat(MAX_BYTES + 2000));
    assert!(check_body(&huge).is_err());
}

#[test]
fn a_sketch_holds_ink_only() {
    let s = sketch::new_sketch("sk-1", 480, 320);
    assert!(s.validate_sketch().is_ok());
    let mut card = s.clone();
    card.put_node(
        Node {
            id: "t".into(),
            kind: "text".into(),
            width: 1.,
            height: 1.,
            ..Node::default()
        },
        1,
    );
    assert!(card.validate_sketch().is_err());
    let mut plain = s;
    plain.kanso_sketch = None;
    assert!(plain.validate_sketch().is_err());
}

#[test]
fn sketch_fences_are_found_by_id_and_capped() {
    let a = serde_json::to_string(&sketch::new_sketch("sk-a", 320, 240)).unwrap();
    let body = format!("````\n```sketch\n{a}\n```\n````\n\nhello\n\n```sketch\n{a}\n```\n");
    // The first one is inside another fence and isn't a sketch.
    assert_eq!(sketch::find(&body).len(), 1);
    assert!(sketch::find_by_id(&body, "sk-a").is_some());
    let promoted =
        sketch::replace_with_text(&body, "sk-a", "```canvas\nnote:XYZ\nw:480\n```").unwrap();
    assert!(promoted.contains("note:XYZ") && sketch::find(&promoted).is_empty());
    assert!(sketch::replace_with_text(&promoted, "sk-a", "x").is_none());
    assert!(sketch::check_budget(&body).is_ok());
    let big = format!(
        "```sketch\n{}\n```\n",
        "a".repeat(sketch::MAX_SKETCH_BYTES + 1)
    );
    assert!(sketch::check_budget(&big).is_err());
    let many = big
        .replace(
            &"a".repeat(sketch::MAX_SKETCH_BYTES + 1),
            &"a".repeat(200 * 1024),
        )
        .repeat(3);
    assert!(sketch::check_budget(&many).is_err());
}
