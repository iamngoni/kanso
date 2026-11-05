use std::collections::{HashMap, HashSet};

use proptest::prelude::*;
use proptest::test_runner::FileFailurePersistence;
use serde_json::{Map, Value, json};

use super::tests::arb_canvas;
use super::*;

const VAULT: &str = include_str!("fixtures/vault.canvas");

/// The field an export keeps a card's Kanso ID in.
const KANSO_ID: &str = "kansoId";

fn config(cases: u32) -> ProptestConfig {
    ProptestConfig {
        cases,
        failure_persistence: Some(Box::new(FileFailurePersistence::WithSource("regressions"))),
        ..ProptestConfig::default()
    }
}

fn import(text: &str) -> Imported {
    Canvas::from_obsidian(text).expect("the file imports")
}

fn file_card(id: &str, file: &str, title: &str) -> Node {
    Node {
        id: id.into(),
        kind: "file".into(),
        file: Some(file.into()),
        title: Some(title.into()),
        width: 100.0,
        height: 80.0,
        ..Node::default()
    }
}

#[test]
fn a_real_vault_canvas_comes_in_whole() {
    let Imported { canvas, warnings } = import(VAULT);
    assert!(warnings.is_empty(), "{warnings:?}");
    assert_eq!(canvas.nodes.len(), 7);
    assert_eq!(canvas.edges.len(), 4);
    canvas.validate().unwrap();
    assert!(is_canvas(&to_body(&canvas)));

    // The file's order is the stacking order, and the group is under the rest.
    let order: Vec<&str> = canvas.live_nodes().map(|n| n.kind.as_str()).collect();
    assert_eq!(
        order,
        ["group", "text", "file", "file", "link", "file", "file"]
    );
    let text = canvas.node("7f1c0a52b3d94e02").unwrap();
    assert_eq!(
        text.text.as_deref(),
        Some("# Launch plan\n\n- Ship the beta\n- Write the post\n- [[Welcome]] the first users")
    );
    assert_eq!(text.color.as_deref(), Some("2"));
    let plan = canvas.node("7f1c0a52b3d94e03").unwrap();
    assert_eq!(plan.file.as_deref(), Some("Projects/Plan.md"));
    assert_eq!(plan.subpath.as_deref(), Some("#Milestones"));
    assert_eq!(
        canvas.node("7f1c0a52b3d94e06").unwrap().color.as_deref(),
        Some("#3a86ff")
    );
    let labelled = canvas.edges.iter().find(|e| e.label.is_some()).unwrap();
    assert_eq!(labelled.label.as_deref(), Some("see the diagram"));
    assert_eq!(labelled.from_end.as_deref(), Some("none"));
    assert_eq!(labelled.to_side.as_deref(), Some("top"));
    // What Kanso has no use for stays.
    assert_eq!(
        canvas.extra["metadata"],
        json!({"version": "1.0-1.0", "frontmatter": {}})
    );
}

#[test]
fn a_vault_canvas_survives_import_then_export() {
    let canvas = import(VAULT).canvas;
    let text = canvas.to_obsidian_json(&ExportFiles::default());
    let again = import(&text).canvas;
    let view = |c: &Canvas| -> Vec<(String, String, Option<String>, Option<String>)> {
        c.live_nodes()
            .map(|n| (n.id.clone(), n.kind.clone(), n.file.clone(), n.text.clone()))
            .collect()
    };
    assert_eq!(view(&canvas), view(&again));
    assert_eq!(canvas.edges.len(), again.edges.len());
    assert_eq!(canvas.extra, again.extra);
}

#[test]
fn paths_become_note_and_attachment_ids() {
    let mut canvas = import(VAULT).canvas;
    let mut paths = HashMap::new();
    paths.insert(
        "Projects/Plan.md".to_owned(),
        ("note:plan".to_owned(), "Plan".to_owned()),
    );
    paths.insert(
        "Attachments/diagram.png".to_owned(),
        ("file:diagram".to_owned(), "diagram.png".to_owned()),
    );
    canvas.map_paths(&paths);
    let plan = canvas.node("7f1c0a52b3d94e03").unwrap();
    assert_eq!(Canvas::note_id(plan), Some("note:plan"));
    assert_eq!(plan.title.as_deref(), Some("Plan"));
    let diagram = canvas.node("7f1c0a52b3d94e04").unwrap();
    assert_eq!(Canvas::attachment_id(diagram), Some("file:diagram"));
    assert_eq!(canvas.attachment_ids(), ["file:diagram"]);
    // A path the import didn't make stays as it was written.
    assert_eq!(
        canvas.node("7f1c0a52b3d94e07").unwrap().file.as_deref(),
        Some("Missing/Gone.md")
    );
}

#[test]
fn an_export_names_the_files_its_cards_point_at() {
    let mut canvas = Canvas::default();
    canvas.put_node(file_card("p", "file:abc", "diagram.png"), 1);
    let mut files = ExportFiles::default();
    files
        .attachments
        .insert("file:abc".into(), "diagram.png".into());
    let out = canvas.to_obsidian_with(&files);
    assert_eq!(out.nodes[0].file.as_deref(), Some("diagram.png"));
    assert_eq!(out.nodes[0].extra[KANSO_ID], "file:abc");
    // And the card finds its attachment again.
    let mut back = import(&canvas.to_obsidian_json(&files)).canvas;
    assert_eq!(back.node("p").unwrap().file.as_deref(), Some("diagram.png"));
    back.restore_ids(&|id| id == "file:abc");
    assert_eq!(back.node("p").unwrap().file.as_deref(), Some("file:abc"));
}

#[test]
fn an_id_that_is_not_here_is_not_restored() {
    let mut canvas = Canvas::default();
    canvas.put_node(file_card("p", "note:gone", "Gone"), 1);
    let mut back = import(&canvas.to_obsidian_json(&ExportFiles::default())).canvas;
    back.restore_ids(&|_| false);
    assert_eq!(back.node("p").unwrap().file.as_deref(), Some("Gone.md"));
}

#[test]
fn what_cannot_be_a_canvas_is_left_out_and_said() {
    let text = json!({
        "nodes": [
            {"id": "a", "type": "text", "text": "kept", "x": 0, "y": 0, "width": 100, "height": 50},
            {"id": "ink1", "type": "ink", "x": 0, "y": 0, "width": 10, "height": 10},
            "not a card",
            {"id": "b", "type": "text", "x": "far", "y": 0, "width": 10, "height": 10}
        ],
        "edges": [
            {"id": "e1", "fromNode": "a", "toNode": "a"},
            {"id": "e2", "fromNode": "a", "toNode": "nowhere"}
        ]
    })
    .to_string();
    let Imported { canvas, warnings } = import(&text);
    assert_eq!(canvas.nodes.len(), 1);
    assert_eq!(canvas.edges.len(), 1);
    assert_eq!(warnings.len(), 4, "{warnings:?}");
    canvas.validate().unwrap();
}

#[test]
fn missing_sizes_ids_and_types_are_filled_in() {
    let text = json!({
        "nodes": [
            {"text": "no id or type"},
            {"id": "a", "type": "group", "x": 5, "y": 5, "width": 0, "height": -4},
            {"id": "a", "type": "link", "url": "https://example.com"},
            {"file": "x.png"}
        ],
        "edges": [{"fromNode": "a", "toNode": "a", "fromSide": "middle", "toEnd": "dot"}]
    })
    .to_string();
    let canvas = import(&text).canvas;
    canvas.validate().unwrap();
    let kinds: Vec<&str> = canvas.nodes.iter().map(|n| n.kind.as_str()).collect();
    assert_eq!(kinds, ["text", "group", "link", "file"]);
    let ids: HashSet<&str> = canvas.nodes.iter().map(|n| n.id.as_str()).collect();
    assert_eq!(ids.len(), 4);
    assert_eq!(canvas.nodes[1].width, 400.0);
    assert_eq!(canvas.nodes[1].height, 300.0);
    assert_eq!(canvas.edges[0].from_side, None);
    assert_eq!(canvas.edges[0].to_end, None);
}

#[test]
fn a_file_cannot_pose_as_a_kanso_note_or_attachment() {
    let text = json!({"nodes": [
        {"id": "a", "type": "file", "file": "note:abc", "width": 10, "height": 10},
        {"id": "b", "type": "file", "file": "file:abc", "width": 10, "height": 10},
        {"id": "c", "type": "file", "file": "notes/c.md", "kansoId": "note:abc", "width": 10, "height": 10}
    ]})
    .to_string();
    let mut canvas = import(&text).canvas;
    assert!(canvas.nodes.iter().all(|n| Canvas::note_id(n).is_none()));
    assert!(canvas.attachment_ids().is_empty());
    // Only an import that checked the ID restores one.
    canvas.restore_ids(&|id| id == "note:abc");
    assert_eq!(Canvas::note_id(canvas.node("c").unwrap()), Some("note:abc"));
    assert!(Canvas::note_id(canvas.node("a").unwrap()).is_none());
}

#[test]
fn files_that_are_not_canvases_are_refused() {
    assert!(Canvas::from_obsidian("[1, 2]").is_err());
    assert!(Canvas::from_obsidian("not json").is_err());
    assert!(Canvas::from_obsidian(&" ".repeat(MAX_FILE_BYTES + 1)).is_err());
    let nothing = Canvas::from_obsidian("").unwrap();
    assert!(nothing.canvas.nodes.is_empty());
    let nothing = Canvas::from_obsidian("\u{feff}{}").unwrap();
    assert!(nothing.canvas.nodes.is_empty());
}

#[test]
fn a_canvas_over_its_budget_is_refused() {
    let big = "x".repeat(2_000);
    let nodes: Vec<Value> = (0..600)
        .map(|i| json!({"id": format!("n{i}"), "type": "text", "text": big, "width": 10, "height": 10}))
        .collect();
    let err = Canvas::from_obsidian(&json!({"nodes": nodes}).to_string()).unwrap_err();
    assert!(err.contains("budget"), "{err}");
}

// ---------- properties ----------

/// What a card says on a canvas that went out and came back.
type CardView = (
    String,
    String,
    [i64; 4],
    [Option<String>; 6],
    Map<String, Value>,
);

fn cards(canvas: &Canvas) -> Vec<CardView> {
    canvas
        .live_nodes()
        .filter(|n| n.kind != INK)
        .map(|n| {
            (
                n.id.clone(),
                n.kind.clone(),
                [n.x, n.y, n.width, n.height].map(|v| v as i64),
                [
                    n.color.clone(),
                    n.text.clone(),
                    n.file.clone(),
                    n.url.clone(),
                    n.label.clone(),
                    n.title.clone(),
                ],
                n.extra.clone(),
            )
        })
        .collect()
}

type ArrowView = (
    String,
    String,
    String,
    [Option<String>; 4],
    Map<String, Value>,
);

fn arrows(canvas: &Canvas, ids: &HashSet<String>) -> Vec<ArrowView> {
    let mut edges: Vec<&Edge> = canvas
        .edges
        .iter()
        .filter(|e| !e.deleted && ids.contains(&e.from_node) && ids.contains(&e.to_node))
        .collect();
    edges.sort_by(|a, b| {
        a.kanso_z
            .total_cmp(&b.kanso_z)
            .then_with(|| a.id.cmp(&b.id))
    });
    edges
        .into_iter()
        .map(|e| {
            (
                e.id.clone(),
                e.from_node.clone(),
                e.to_node.clone(),
                [
                    e.from_side.clone(),
                    e.to_side.clone(),
                    e.from_end.clone(),
                    e.to_end.clone(),
                ],
                e.extra.clone(),
            )
        })
        .collect()
}

fn arb_json() -> impl Strategy<Value = Value> {
    let leaf = prop_oneof![
        Just(Value::Null),
        any::<bool>().prop_map(Value::Bool),
        any::<i64>().prop_map(Value::from),
        (-1.0e30f64..1.0e30).prop_map(Value::from),
        ".{0,8}".prop_map(Value::String),
    ];
    leaf.prop_recursive(3, 24, 4, |inner| {
        prop_oneof![
            prop::collection::vec(inner.clone(), 0..4).prop_map(Value::Array),
            prop::collection::btree_map("[a-z]{1,6}", inner, 0..4)
                .prop_map(|m| Value::Object(m.into_iter().collect())),
        ]
    })
}

const KEYS: &[&str] = &[
    "id",
    "type",
    "x",
    "y",
    "width",
    "height",
    "text",
    "file",
    "url",
    "label",
    "color",
    "subpath",
    "strokes",
    "deleted",
    "kansoId",
    "kansoVersion",
    "kansoZ",
    "fromNode",
    "toNode",
    "fromSide",
    "toSide",
    "fromEnd",
    "toEnd",
    "other",
];

fn field(key: &'static str) -> BoxedStrategy<Value> {
    let name = || prop::sample::select(vec!["a", "b", "c", "d", ""]);
    match key {
        "id" | "fromNode" | "toNode" => prop_oneof![
            3 => name().prop_map(Value::from),
            1 => arb_json(),
        ]
        .boxed(),
        "type" => prop_oneof![
            4 => prop::sample::select(vec!["text", "file", "link", "group", "ink", "weird", ""])
                .prop_map(Value::from),
            1 => arb_json(),
        ]
        .boxed(),
        "x" | "y" | "width" | "height" => prop_oneof![
            4 => (-500i64..500).prop_map(Value::from),
            1 => Just(Value::from(0)),
            1 => Just(json!(1e30)),
            1 => Just(json!(-1e300)),
            1 => arb_json(),
        ]
        .boxed(),
        "file" | "kansoId" => prop_oneof![
            3 => prop::sample::select(vec![
                "note:abc", "file:abc", "a.md", "dir/b.png", "./c", "note:", ""
            ])
            .prop_map(Value::from),
            1 => arb_json(),
        ]
        .boxed(),
        "fromSide" | "toSide" => prop_oneof![
            3 => prop::sample::select(vec!["top", "left", "middle", ""]).prop_map(Value::from),
            1 => arb_json(),
        ]
        .boxed(),
        "fromEnd" | "toEnd" => prop_oneof![
            3 => prop::sample::select(vec!["none", "arrow", "dot"]).prop_map(Value::from),
            1 => arb_json(),
        ]
        .boxed(),
        _ => arb_json().boxed(),
    }
}

fn arb_loose_object() -> impl Strategy<Value = Value> {
    prop::collection::vec(
        prop::sample::select(KEYS).prop_flat_map(|k| (Just(k), field(k))),
        0..9,
    )
    .prop_map(|pairs| {
        Value::Object(
            pairs
                .into_iter()
                .map(|(k, v)| (k.to_owned(), v))
                .collect::<Map<_, _>>(),
        )
    })
}

fn arb_loose_item() -> impl Strategy<Value = Value> {
    prop_oneof![8 => arb_loose_object(), 1 => arb_json()]
}

/// A document that only looks like a canvas file: fields of the wrong type,
/// sizes out of range, arrows to nowhere, repeated and missing IDs.
fn arb_loose_file() -> impl Strategy<Value = Value> {
    (
        prop::collection::vec(arb_loose_item(), 0..8),
        prop::collection::vec(arb_loose_item(), 0..8),
        prop::collection::btree_map(
            prop_oneof![
                Just("metadata".to_owned()),
                "[a-z]{1,5}",
                Just("kansoSketch".to_owned()),
            ],
            arb_json(),
            0..3,
        ),
    )
        .prop_map(|(nodes, edges, extra)| {
            let mut doc: Map<String, Value> = extra.into_iter().collect();
            doc.insert("nodes".into(), Value::Array(nodes));
            doc.insert("edges".into(), Value::Array(edges));
            Value::Object(doc)
        })
}

proptest! {
    #![proptest_config(config(256))]

    /// No file, however mangled, imports as a canvas Kanso can't open: it is
    /// refused, or it validates, fits the store's check, and is a canvas
    /// note's body that parses back.
    #[test]
    fn prop_no_import_is_an_invalid_canvas(doc in prop_oneof![arb_loose_file(), arb_json()]) {
        if let Ok(Imported { canvas, .. }) = Canvas::from_obsidian(&doc.to_string()) {
            prop_assert!(canvas.validate().is_ok());
            let body = to_body(&canvas);
            prop_assert!(check_body(&body).is_ok());
            prop_assert!(parse(&body).is_ok());
            // Nothing from outside is ink, a tombstone, a Kanso ID or a
            // Kanso field.
            for node in &canvas.nodes {
                prop_assert!(node.kind != INK && !node.deleted && node.strokes.is_empty());
                prop_assert_eq!(node.kanso_version, 0);
                let posing = node.file.as_deref().is_some_and(|f| {
                    f.starts_with(NOTE_PREFIX) || f.starts_with(ATTACHMENT_PREFIX)
                });
                prop_assert!(!posing);
            }
            prop_assert!(canvas.kanso_sketch.is_none() && canvas.kanso_ink_ocr.is_empty());
            // And it can go back out as a file that comes in again.
            let out = canvas.to_obsidian_json(&ExportFiles::default());
            prop_assert!(Canvas::from_obsidian(&out).is_ok());
        }
    }

    /// Cards, text and arrows survive export then import: the same cards in
    /// the same order with the same text, notes, links, colours and extra
    /// fields, and the same arrows between them. Ink is not exported.
    #[test]
    fn prop_export_then_import_keeps_cards_text_and_arrows(canvas in arb_canvas()) {
        let text = canvas.to_obsidian_json(&ExportFiles::default());
        let mut back = Canvas::from_obsidian(&text).unwrap().canvas;
        back.restore_ids(&|_| true);
        prop_assert_eq!(cards(&back), cards(&canvas));
        let kept: HashSet<String> = canvas
            .live_nodes()
            .filter(|n| n.kind != INK)
            .map(|n| n.id.clone())
            .collect();
        prop_assert_eq!(arrows(&back, &kept), arrows(&canvas, &kept));
        prop_assert_eq!(&back.extra, &canvas.extra);
        // Importing what we just wrote is stable.
        let again = back.to_obsidian_json(&ExportFiles::default());
        let mut twice = Canvas::from_obsidian(&again).unwrap().canvas;
        twice.restore_ids(&|_| true);
        prop_assert_eq!(cards(&twice), cards(&back));
    }

    /// The same path always maps to the same ID, however it was written
    /// (with or without `.md`, with `./`), and mapping twice changes nothing.
    #[test]
    fn prop_ids_map_consistently(
        picks in prop::collection::vec((0usize..3, 0usize..3), 1..12),
        known in prop::collection::vec(any::<bool>(), 3),
    ) {
        let stems = ["Projects/Plan", "Welcome", "People/Ada"];
        let spell = |stem: &str, how: usize| match how {
            0 => stem.to_owned(),
            1 => format!("{stem}.md"),
            _ => format!("./{stem}.md"),
        };
        let mut paths: HashMap<String, (String, String)> = HashMap::new();
        for (i, stem) in stems.iter().enumerate() {
            if known[i] {
                paths.insert(
                    format!("{stem}.md"),
                    (format!("note:id{i}"), format!("Title {i}")),
                );
            }
        }
        let nodes: Vec<Value> = picks
            .iter()
            .enumerate()
            .map(|(n, (s, how))| json!({
                "id": format!("n{n}"), "type": "file",
                "file": spell(stems[*s], *how), "width": 10, "height": 10
            }))
            .collect();
        let mut canvas = Canvas::from_obsidian(&json!({"nodes": nodes}).to_string())
            .unwrap()
            .canvas;
        canvas.map_paths(&paths);
        for (n, (s, _)) in picks.iter().enumerate() {
            let node = canvas.node(&format!("n{n}")).unwrap();
            let id = Canvas::note_id(node);
            if known[*s] {
                let want = format!("note:id{s}");
                prop_assert_eq!(id, Some(want.as_str()));
                prop_assert_eq!(node.title.clone(), Some(format!("Title {s}")));
            } else {
                prop_assert!(id.is_none());
            }
        }
        let before = canvas.clone();
        canvas.map_paths(&paths);
        prop_assert_eq!(canvas, before);
    }
}
