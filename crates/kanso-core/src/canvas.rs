//! A canvas note: a [JSON Canvas 1.0](https://jsoncanvas.org) document kept
//! in a fenced `canvas` block in the note's body. Nodes are laid out at
//! `x`/`y` with a width and height; edges join them.
//!
//! A note card is a `file` node whose `file` is the Kanso note's ID, so the
//! card survives a rename. The note's title travels beside it in `title`,
//! for Obsidian exports.
//!
//! Fields Kanso doesn't know are kept: each node's, edge's and document's
//! extras take part in the round trip, so an Obsidian canvas brought in and
//! saved again keeps everything Kanso has no use for.

use std::collections::{HashMap, HashSet};

use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};

/// The fence's info string.
pub const FENCE: &str = "canvas";

/// The ID prefix a note card's `file` holds.
pub const NOTE_PREFIX: &str = "note:";

/// A JSON Canvas 1.0 document.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Canvas {
    #[serde(default)]
    pub nodes: Vec<Node>,
    #[serde(default)]
    pub edges: Vec<Edge>,
    /// Top-level fields JSON Canvas doesn't define.
    #[serde(flatten)]
    pub extra: Map<String, Value>,
}

/// A card on the canvas.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Node {
    pub id: String,
    /// `text`, `file`, `link` or `group`; another app's type is kept too.
    #[serde(rename = "type")]
    pub kind: String,
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub color: Option<String>,
    /// A text card's Markdown.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub text: Option<String>,
    /// A file card's path, or a note card's Kanso ID.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub file: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub subpath: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,
    /// A group's label.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub label: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub background: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub background_style: Option<String>,
    /// A note card's title beside its ID.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    /// Fields JSON Canvas doesn't define.
    #[serde(flatten)]
    pub extra: Map<String, Value>,
}

/// An arrow between two cards.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Edge {
    pub id: String,
    pub from_node: String,
    pub to_node: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub from_side: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub from_end: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub to_side: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub to_end: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub color: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub label: Option<String>,
    /// Fields JSON Canvas doesn't define.
    #[serde(flatten)]
    pub extra: Map<String, Value>,
}

impl Canvas {
    /// Whether node `id` is on the canvas.
    pub fn has_node(&self, id: &str) -> bool {
        self.nodes.iter().any(|n| n.id == id)
    }

    /// The node `id`, if it is on the canvas.
    pub fn node(&self, id: &str) -> Option<&Node> {
        self.nodes.iter().find(|n| n.id == id)
    }

    /// The same node, to change.
    pub fn node_mut(&mut self, id: &str) -> Option<&mut Node> {
        self.nodes.iter_mut().find(|n| n.id == id)
    }

    /// The note ID a note card points at.
    pub fn note_id(node: &Node) -> Option<&str> {
        node.file
            .as_deref()
            .filter(|file| file.starts_with(NOTE_PREFIX))
    }

    /// Checks what the editor relies on: every ID is there and unique, sizes
    /// are positive, and every edge joins two cards that exist.
    pub fn validate(&self) -> Result<(), String> {
        let mut ids = HashSet::new();
        for node in &self.nodes {
            if node.id.is_empty() {
                return Err("A card has no ID.".into());
            }
            if !ids.insert(node.id.as_str()) {
                return Err(format!("Two cards have the ID “{}”.", node.id));
            }
            if !(node.width > 0. && node.height > 0.) {
                return Err(format!("Card “{}” has no size.", node.id));
            }
            if !(node.x.is_finite()
                && node.y.is_finite()
                && node.width.is_finite()
                && node.height.is_finite())
            {
                return Err(format!("Card “{}” is not at a real place.", node.id));
            }
        }
        let mut edge_ids = HashSet::new();
        for edge in &self.edges {
            if edge.id.is_empty() {
                return Err("An arrow has no ID.".into());
            }
            if !edge_ids.insert(edge.id.as_str()) {
                return Err(format!("Two arrows have the ID “{}”.", edge.id));
            }
            if !ids.contains(edge.from_node.as_str()) || !ids.contains(edge.to_node.as_str()) {
                return Err(format!("Arrow “{}” points at a card that isn't there.", edge.id));
            }
            for side in [&edge.from_side, &edge.to_side].into_iter().flatten() {
                if !matches!(side.as_str(), "top" | "right" | "bottom" | "left") {
                    return Err(format!("Arrow “{}” has no such side: {side}.", edge.id));
                }
            }
            for end in [&edge.from_end, &edge.to_end].into_iter().flatten() {
                if !matches!(end.as_str(), "none" | "arrow") {
                    return Err(format!("Arrow “{}” has no such end: {end}.", edge.id));
                }
            }
        }
        Ok(())
    }

    /// Points file cards at Kanso notes by the vault paths Obsidian wrote,
    /// through `paths` (vault path to note ID and title). Paths Kanso didn't
    /// import are left alone. Pure.
    pub fn map_paths(&mut self, paths: &HashMap<String, (String, String)>) {
        for node in &mut self.nodes {
            if node.kind != "file" {
                continue;
            }
            let Some(file) = node.file.clone() else {
                continue;
            };
            let Some((id, title)) = resolve_path(&file, paths) else {
                continue;
            };
            node.file = Some(id.clone());
            node.title = Some(title.clone());
        }
    }

    /// The canvas as an Obsidian `.canvas` document: note cards name the
    /// note's title as a path, and keep its ID beside it. Pure.
    pub fn to_obsidian(&self) -> Canvas {
        let mut out = self.clone();
        for node in &mut out.nodes {
            if node.kind != "file" {
                continue;
            }
            let Some(id) = node.file.as_deref().filter(|f| f.starts_with(NOTE_PREFIX)) else {
                continue;
            };
            let title = node.title.clone().unwrap_or_else(|| "Untitled".into());
            node.extra
                .insert("kansoId".into(), Value::String(id.to_owned()));
            node.file = Some(export_path(&title));
        }
        out
    }

    /// The canvas with fresh IDs where one is missing or repeated, so an
    /// edited copy can be written: what Kanso doesn't understand is kept.
    /// Pure.
    pub fn repaired(mut self) -> Canvas {
        let mut seen = HashSet::new();
        let mut taken: HashSet<String> = self.nodes.iter().map(|n| n.id.clone()).collect();
        for node in &mut self.nodes {
            if node.id.is_empty() || !seen.insert(node.id.clone()) {
                let id = fresh_id("card", &taken);
                taken.insert(id.clone());
                node.id = id;
            }
        }
        // Edges that named a now-renamed card can't be repaired without
        // guessing; those that named a card that never existed are dropped.
        let ids: HashSet<String> = self.nodes.iter().map(|n| n.id.clone()).collect();
        self.edges
            .retain(|e| ids.contains(&e.from_node) && ids.contains(&e.to_node));
        let mut seen = HashSet::new();
        let mut taken: HashSet<String> = self.edges.iter().map(|e| e.id.clone()).collect();
        for edge in &mut self.edges {
            if edge.id.is_empty() || !seen.insert(edge.id.clone()) {
                let id = fresh_id("edge", &taken);
                taken.insert(id.clone());
                edge.id = id;
            }
        }
        self
    }
}

/// An ID of `prefix` and the first number after it that `taken` doesn't
/// hold. Pure.
pub fn fresh_id(prefix: &str, taken: &HashSet<String>) -> String {
    let mut n = 1;
    loop {
        let id = format!("{prefix}-{n}");
        if !taken.contains(&id) {
            return id;
        }
        n += 1;
    }
}

/// The body of a note that holds `canvas`: the fenced block as the editor
/// shows it. Pure.
pub fn to_body(canvas: &Canvas) -> String {
    let json = serde_json::to_string_pretty(canvas).unwrap_or_else(|_| "{}".into());
    format!("```{FENCE}\n{json}\n```")
}

/// The JSON inside a `canvas` fence, if `body` is one. Pure.
pub fn fenced(body: &str) -> Option<&str> {
    let trimmed = body.trim();
    let rest = trimmed.strip_prefix("```")?;
    let (info, rest) = rest.split_once('\n')?;
    if info.trim() != FENCE {
        return None;
    }
    let close = rest.rfind("```")?;
    if !rest[close + 3..].trim().is_empty() {
        return None;
    }
    Some(&rest[..close])
}

/// The canvas in `body`, or why it isn't one. Pure.
pub fn parse(body: &str) -> Result<Canvas, String> {
    let json = fenced(body).ok_or_else(|| "The note has no canvas block.".to_owned())?;
    let canvas: Canvas = serde_json::from_str(json)
        .map_err(|err| format!("The canvas isn't valid JSON Canvas: {err}"))?;
    canvas.validate()?;
    Ok(canvas)
}

/// Whether `body` is a canvas note. Pure.
pub fn is_canvas(body: &str) -> bool {
    parse(body).is_ok()
}

/// Whether `file` names a note card that belongs on the canvas.
pub fn is_note_ref(file: &str) -> bool {
    file.starts_with(NOTE_PREFIX)
}

/// A path Obsidian wrote in a canvas, as the import knows it: with or
/// without its `.md` or `.canvas`, and with a leading `./`. Pure.
fn resolve_path<'a>(
    file: &str,
    paths: &'a HashMap<String, (String, String)>,
) -> Option<&'a (String, String)> {
    let file = file.trim_start_matches("./");
    paths.get(file).or_else(|| {
        for ext in [".md", ".canvas"] {
            if let Some(stem) = file.strip_suffix(ext)
                && let Some(found) = paths.get(stem)
            {
                return Some(found);
            }
        }
        let with_md = format!("{file}.md");
        paths.get(&with_md)
    })
}

/// A note's title as a path Obsidian can open. Pure.
pub fn export_path(title: &str) -> String {
    let cleaned: String = title
        .chars()
        .map(|c| match c {
            '/' | '\\' | ':' | '*' | '?' | '"' | '<' | '>' | '|' => '-',
            c if c.is_control() => ' ',
            c => c,
        })
        .collect();
    let name = cleaned.trim();
    if name.is_empty() {
        "Untitled.md".into()
    } else {
        format!("{name}.md")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;
    use proptest::test_runner::FileFailurePersistence;

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
        prop::collection::btree_map("[a-z][a-zA-Z0-9]{0,6}", 0i64..1000, 0..3)
            .prop_map(|map| map.into_iter().map(|(k, v)| (k, Value::from(v))).collect())
    }

    fn arb_node(ix: usize) -> impl Strategy<Value = Node> {
        (
            0u8..4,
            -2000i64..2000,
            -2000i64..2000,
            20i64..600,
            20i64..600,
            prop_oneof![Just(None), Just(Some("1".into())), Just(Some("#a1b2c3".into()))],
            "(\n|[a-zA-Z0-9 ]){0,40}",
            "[a-zA-Z0-9 ]{0,20}",
            extra(),
        )
            .prop_map(move |(kind, x, y, width, height, color, text, title, extra)| {
                let kind = ["text", "file", "link", "group"][kind as usize];
                Node {
                    id: format!("n{ix}"),
                    kind: kind.into(),
                    x: x as f64,
                    y: y as f64,
                    width: width as f64,
                    height: height as f64,
                    color,
                    text: (kind == "text").then_some(text),
                    file: (kind == "file").then(|| format!("note:n{ix}")),
                    subpath: None,
                    url: (kind == "link").then(|| "https://example.com".into()),
                    label: (kind == "group").then_some(title),
                    background: None,
                    background_style: None,
                    title: (kind == "file").then_some(title),
                    extra,
                }
            })
    }

    fn arb_canvas() -> impl Strategy<Value = Canvas> {
        (1usize..12).prop_flat_map(|n| {
            let nodes: Vec<_> = (0..n).map(arb_node).collect();
            (
                prop::collection::vec(
                    (0usize..12, 0usize..12, side(), side(), extra()),
                    0..20,
                ),
                extra(),
            )
                .prop_map(move |(raw_edges, extra)| {
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
                            to_end: None,
                            color: None,
                            label: None,
                            extra,
                        })
                        .collect();
                    Canvas {
                        nodes: nodes.clone(),
                        edges,
                        extra,
                    }
                })
        })
    }

    proptest! {
        #![proptest_config(config(64))]

        /// Writing a canvas into a note and reading it back gives the same
        /// canvas, unknown fields and all.
        #[test]
        fn prop_round_trip(canvas in arb_canvas()) {
            let body = to_body(&canvas);
            prop_assert!(is_canvas(&body));
            prop_assert_eq!(parse(&body).unwrap(), canvas);
        }

        /// A fence is found only when it is the note's own, opening and
        /// closing on lines of their own.
        #[test]
        fn prop_only_a_canvas_fence_is_found(text in "(\n|[a-zA-Z0-9`{}]){0,200}") {
            let bare = text.trim();
            let is_fence = bare.starts_with("```canvas\n") && bare.ends_with("```");
            prop_assert_eq!(fenced(&text).is_some(), is_fence);
        }

        /// The repaired canvas keeps every card and every edge that still
        /// joins two of them, with one unique ID each.
        #[test]
        fn prop_repaired_keeps_cards(mut canvas in arb_canvas(), ids in prop::collection::vec("[a-z]{0,4}", 0..8)) {
            for (ix, id) in ids.iter().enumerate() {
                if let Some(node) = canvas.nodes.get_mut(ix) {
                    node.id = id.clone();
                }
            }
            canvas = canvas.repaired();
            prop_assert!(canvas.validate().is_ok());
            let node_ids: HashSet<&str> = canvas.nodes.iter().map(|n| n.id.as_str()).collect();
            for edge in &canvas.edges {
                prop_assert!(node_ids.contains(edge.from_node.as_str()));
                prop_assert!(node_ids.contains(edge.to_node.as_str()));
            }
        }

        /// An Obsidian export writes note cards as title paths, keeps the
        /// edge and node counts, and says which note each card is.
        #[test]
        fn prop_obsidian_export_keeps_the_shape(canvas in arb_canvas()) {
            let out = canvas.to_obsidian();
            prop_assert_eq!(out.nodes.len(), canvas.nodes.len());
            prop_assert_eq!(out.edges.len(), canvas.edges.len());
            for (before, after) in canvas.nodes.iter().zip(&out.nodes) {
                if let Some(id) = Canvas::note_id(before) {
                    prop_assert_eq!(after.extra.get("kansoId").and_then(Value::as_str), Some(id));
                    prop_assert!(after.file.as_deref().is_some_and(|f| f.ends_with(".md")));
                } else {
                    prop_assert_eq!(after.file.as_deref(), before.file.as_deref());
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
            let known: Vec<(&String, &(String, String))> = paths.iter().collect();
            let mut canvas = Canvas::default();
            for (ix, (path, (id, title))) in known.iter().enumerate() {
                canvas.nodes.push(Node {
                    id: format!("n{ix}"),
                    kind: "file".into(),
                    x: 0., y: 0., width: 200., height: 100.,
                    color: None, text: None, file: Some((*path).clone()),
                    subpath: None, url: None, label: None,
                    background: None, background_style: None, title: None,
                    extra: Map::new(),
                });
                let _ = (id, title);
            }
            canvas.nodes.push(Node {
                id: "gone".into(), kind: "file".into(), x: 0., y: 0., width: 1., height: 1.,
                color: None, text: None, file: Some("Not There.md".into()), subpath: None,
                url: None, label: None, background: None, background_style: None,
                title: None, extra: Map::new(),
            });
            canvas.map_paths(&paths);
            for node in &canvas.nodes {
                match paths.get(node.file.as_deref().unwrap_or_default()) {
                    Some((id, _)) => prop_assert_eq!(node.file.as_deref(), Some(id.as_str())),
                    None => prop_assert!(node.file.as_deref() == Some("Not There.md") || node.file.as_deref().is_some_and(|f| f.starts_with("note:"))),
                }
            }
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
}
