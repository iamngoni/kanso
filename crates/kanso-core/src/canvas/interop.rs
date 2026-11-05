//! Canvases to and from the JSON Canvas files other apps write: reading one
//! in so that what comes out is always a canvas Kanso can open, and the names
//! an export gives the files its cards point at.
//!
//! A file from outside is not trusted. A card can't name a Kanso note or
//! attachment by accident, ink Kanso didn't draw is not taken, sizes and
//! places are brought into range, and arrows that join nothing are dropped.

use std::collections::{HashMap, HashSet};

use serde_json::{Map, Value};

use super::model::{Canvas, Edge, INK, Node};
use super::{MAX_BYTES, MAX_ITEMS, NOTE_PREFIX, to_body};

/// The ID prefix an attachment card's `file` holds: the attachment's
/// document ID, as a note card holds the note's.
pub const ATTACHMENT_PREFIX: &str = "file:";

/// The field an export keeps a card's Kanso ID in, beside the path it
/// writes.
pub(super) const ID_FIELD: &str = "kansoId";

/// The extension of a canvas file.
pub const EXTENSION: &str = "canvas";

/// The most a `.canvas` file may be, in bytes.
pub const MAX_FILE_BYTES: usize = 8 * 1024 * 1024;

/// How far from the origin a card may sit, in points.
const MAX_COORD: f64 = 1.0e7;

/// The largest a card may be, per side, in points.
const MAX_SIDE: f64 = 1.0e6;

/// What a card gets when its file gave it no size, by type.
fn default_size(kind: &str) -> (f64, f64) {
    match kind {
        "group" => (400.0, 300.0),
        _ => (250.0, 60.0),
    }
}

/// A canvas read from a file, with what had to be left out.
#[derive(Clone, Debug, PartialEq)]
pub struct Imported {
    pub canvas: Canvas,
    /// One plain sentence per thing left out or changed.
    pub warnings: Vec<String>,
}

/// The file names an export gives what its cards point at.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct ExportFiles {
    /// An ink node's ID to the name of its PNG.
    pub ink: HashMap<String, String>,
    /// An attachment's ID to the name of its file.
    pub attachments: HashMap<String, String>,
}

impl Canvas {
    /// Reads the text of a JSON Canvas file written by another app. Cards
    /// and arrows come in as they are, in the order the file stacks them;
    /// fields Kanso doesn't know are kept. What can't be a canvas is left
    /// out and said in the warnings: cards that aren't objects, ink, arrows
    /// that join a card that isn't there. A file that isn't JSON, is too
    /// big, or holds too many things is refused. Pure.
    pub fn from_obsidian(text: &str) -> Result<Imported, String> {
        if text.len() > MAX_FILE_BYTES {
            return Err("The canvas file is too large to import.".into());
        }
        let text = text.trim_start_matches('\u{feff}').trim();
        let mut warnings = Vec::new();
        let root: Map<String, Value> = if text.is_empty() {
            Map::new()
        } else {
            match serde_json::from_str::<Value>(text) {
                Ok(Value::Object(map)) => map,
                Ok(_) => return Err("The file isn't a JSON Canvas document.".into()),
                Err(err) => return Err(format!("The file isn't valid JSON: {err}")),
            }
        };
        let mut canvas = Canvas::default();
        for (key, value) in &root {
            match key.as_str() {
                "nodes" | "edges" => {}
                // Kanso's own top-level fields mean nothing from outside.
                "kansoSketch" | "kansoSketchId" | "kansoInkOcr" => {}
                _ => {
                    canvas.extra.insert(key.clone(), value.clone());
                }
            }
        }
        let raw_nodes = array_of(root.get("nodes"), "nodes", &mut warnings)?;
        let raw_edges = array_of(root.get("edges"), "edges", &mut warnings)?;
        if raw_nodes.len() > MAX_ITEMS || raw_edges.len() > MAX_ITEMS {
            return Err("The canvas has too many items to import.".into());
        }
        for raw in raw_nodes {
            match node_from(raw) {
                Ok(mut node) => {
                    node.kanso_z = (canvas.nodes.len() + 1) as f64;
                    canvas.nodes.push(node);
                }
                Err(why) => warnings.push(why),
            }
        }
        for raw in raw_edges {
            match edge_from(raw) {
                Ok(mut edge) => {
                    edge.kanso_z = (canvas.edges.len() + 1) as f64;
                    canvas.edges.push(edge);
                }
                Err(why) => warnings.push(why),
            }
        }
        let edges_before = canvas.edges.len();
        let canvas = canvas.repaired();
        if canvas.edges.len() < edges_before {
            warnings.push(format!(
                "{} arrow(s) joined a card that isn't on the canvas and were left out.",
                edges_before - canvas.edges.len()
            ));
        }
        canvas.validate()?;
        if to_body(&canvas).len() > MAX_BYTES {
            return Err("The canvas is over its 1 MB budget.".into());
        }
        Ok(Imported { canvas, warnings })
    }

    /// Points cards at the Kanso notes and attachments an earlier export
    /// named in their `kansoId`, where `exists` says the ID is here. A card
    /// whose path an import already resolved is left alone. Pure.
    pub fn restore_ids(&mut self, exists: &dyn Fn(&str) -> bool) {
        for node in &mut self.nodes {
            if node.kind != "file" {
                continue;
            }
            if node
                .file
                .as_deref()
                .is_some_and(|f| f.starts_with(NOTE_PREFIX) || f.starts_with(ATTACHMENT_PREFIX))
            {
                continue;
            }
            let Some(id) = node
                .extra
                .get(ID_FIELD)
                .and_then(Value::as_str)
                .map(str::to_owned)
            else {
                continue;
            };
            if !(id.starts_with(NOTE_PREFIX) || id.starts_with(ATTACHMENT_PREFIX)) || !exists(&id) {
                continue;
            }
            if node.title.is_none() {
                node.title = node.file.as_deref().map(title_of_path);
            }
            node.extra.remove(ID_FIELD);
            node.file = Some(id);
        }
    }

    /// The attachment an attachment card points at.
    pub fn attachment_id(node: &Node) -> Option<&str> {
        node.file
            .as_deref()
            .filter(|file| file.starts_with(ATTACHMENT_PREFIX))
    }

    /// The IDs of the attachments the live cards point at, in stacking
    /// order, each once. Pure.
    pub fn attachment_ids(&self) -> Vec<String> {
        let mut seen = HashSet::new();
        self.live_nodes()
            .filter_map(Canvas::attachment_id)
            .filter(|id| seen.insert(*id))
            .map(str::to_owned)
            .collect()
    }
}

/// A file name or path as a card title: the last part, without `.md`.
pub(super) fn title_of_path(path: &str) -> String {
    let name = path.rsplit('/').next().unwrap_or(path);
    name.strip_suffix(".md").unwrap_or(name).to_owned()
}

fn array_of<'a>(
    value: Option<&'a Value>,
    what: &str,
    warnings: &mut Vec<String>,
) -> Result<&'a [Value], String> {
    match value {
        None | Some(Value::Null) => Ok(&[]),
        Some(Value::Array(items)) => Ok(items),
        Some(_) => {
            warnings.push(format!("The file's {what} aren't a list; none were read."));
            Ok(&[])
        }
    }
}

/// A card from the file, in range, or why it is left out.
fn node_from(raw: &Value) -> Result<Node, String> {
    let mut node: Node = serde_json::from_value(with_id(raw))
        .map_err(|_| "A card in the file couldn't be read and was left out.".to_owned())?;
    if node.kind == INK || !node.strokes.is_empty() {
        return Err("A layer of ink from another app was left out.".into());
    }
    if node.kind.is_empty() {
        node.kind = if node.file.is_some() {
            "file"
        } else if node.url.is_some() {
            "link"
        } else if node.label.is_some() && node.text.is_none() {
            "group"
        } else {
            "text"
        }
        .into();
    }
    node.deleted = false;
    node.kanso_version = 0;
    node.kanso_updated = 0;
    node.kanso_z = 0.0;
    node.strokes = Vec::new();
    // A path the file wrote must not be taken for a note or attachment ID.
    if let Some(file) = node.file.as_mut()
        && (file.starts_with(NOTE_PREFIX) || file.starts_with(ATTACHMENT_PREFIX))
    {
        *file = format!("./{file}");
    }
    let (default_w, default_h) = default_size(&node.kind);
    node.x = place(node.x);
    node.y = place(node.y);
    node.width = size(node.width, default_w);
    node.height = size(node.height, default_h);
    Ok(node)
}

/// `raw` with an empty `id` when it has none; `repaired` gives it one.
fn with_id(raw: &Value) -> Value {
    let mut raw = raw.clone();
    if let Value::Object(map) = &mut raw {
        map.entry("id")
            .or_insert_with(|| Value::String(String::new()));
    }
    raw
}

fn place(v: f64) -> f64 {
    if v.is_finite() {
        v.clamp(-MAX_COORD, MAX_COORD)
    } else {
        0.0
    }
}

fn size(v: f64, default: f64) -> f64 {
    if v.is_finite() && v > 0.0 {
        v.min(MAX_SIDE)
    } else {
        default
    }
}

/// An arrow from the file, with sides and ends Kanso knows, or why it is
/// left out.
fn edge_from(raw: &Value) -> Result<Edge, String> {
    let mut edge: Edge = serde_json::from_value(with_id(raw))
        .map_err(|_| "An arrow in the file couldn't be read and was left out.".to_owned())?;
    edge.deleted = false;
    edge.kanso_version = 0;
    edge.kanso_updated = 0;
    edge.kanso_z = 0.0;
    for side in [&mut edge.from_side, &mut edge.to_side] {
        if side
            .as_deref()
            .is_some_and(|s| !matches!(s, "top" | "right" | "bottom" | "left"))
        {
            *side = None;
        }
    }
    for end in [&mut edge.from_end, &mut edge.to_end] {
        if end
            .as_deref()
            .is_some_and(|e| !matches!(e, "none" | "arrow"))
        {
            *end = None;
        }
    }
    Ok(edge)
}
