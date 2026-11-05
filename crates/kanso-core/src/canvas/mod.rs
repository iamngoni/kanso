//! A canvas note: a [JSON Canvas 1.0](https://jsoncanvas.org) document, plus
//! Kanso's ink, kept in a fenced `canvas` block as the note's whole body.
//! Nodes are laid out at `x`/`y` with a width and height; edges join them;
//! `ink` nodes hold pen strokes.
//!
//! A note card is a `file` node whose `file` is the Kanso note's ID, so the
//! card survives a rename. The note's title travels beside it in `title`,
//! for Obsidian exports.
//!
//! Fields Kanso doesn't know are kept: each node's, edge's and document's
//! extras take part in the round trip, so an Obsidian canvas brought in and
//! saved again keeps everything Kanso has no use for.
//!
//! The data is in [`model`]; the rules are split by job: [`bounds`] (where
//! things are), [`ink`] (the partial eraser's geometry), [`ops`] (versioned
//! edits), [`embed`] (the fence that shows a canvas in a note), [`merge`] (private sync), [`text`] (summary and search text) and
//! [`sketch`] (inline sketches in a Markdown note).

mod bounds;
mod edit;
#[cfg(test)]
mod edit_tests;
pub mod embed;
mod ink;
mod interop;
#[cfg(test)]
mod interop_tests;
mod merge;
mod model;
mod ops;
mod recognize;
#[cfg(test)]
mod recognize_tests;
mod sanitize;
pub mod sketch;
#[cfg(test)]
mod tests;
mod text;

use std::collections::{HashMap, HashSet};

use serde_json::Value;

pub use bounds::Bounds;
pub use edit::{
    Action, CardChange, MIN_SAMPLE_DISTANCE, Refused, StrokeRef, in_polygon, streamed_points, thin,
    thin_so_far, unique_id,
};
pub use ink::{Eraser, Split, covers, split_stroke};
pub use interop::{ATTACHMENT_PREFIX, EXTENSION, ExportFiles, Imported, MAX_FILE_BYTES};
pub use merge::{merge, merge_bodies};
pub use model::{Canvas, Edge, Embed, INK, InkOcr, Node, Point, Sketch, Stroke, TOOLS};
pub use recognize::{DEFAULT_ENGINE, InkToRead, ink_text_change};
pub use text::SUMMARY_CAP;

/// The fence's info string.
pub const FENCE: &str = "canvas";

/// The ID prefix a note card's `file` holds.
pub const NOTE_PREFIX: &str = "note:";

/// Past this size a canvas warns and offers "Move to a new canvas".
pub const WARN_BYTES: usize = 700_000;

/// Past this size new ink is refused.
pub const MAX_BYTES: usize = 1_000_000;

/// The most points one stroke may hold.
pub const MAX_STROKE_POINTS: usize = 20_000;

/// The most nodes (tombstones included) or edges one canvas may hold.
pub const MAX_ITEMS: usize = 20_000;

/// Where a canvas stands against its size budget.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Budget {
    Ok,
    /// 700 KB or more: warn, offer "Move to a new canvas".
    Warn,
    /// 1 MB or more: refuse new ink.
    Full,
}

/// How a body of `bytes` bytes stands against the budget. Pure.
pub fn budget(bytes: usize) -> Budget {
    if bytes >= MAX_BYTES {
        Budget::Full
    } else if bytes >= WARN_BYTES {
        Budget::Warn
    } else {
        Budget::Ok
    }
}

impl Canvas {
    /// Whether node `id` is on the canvas.
    pub fn has_node(&self, id: &str) -> bool {
        self.node(id).is_some()
    }

    /// The live node `id`, if it is on the canvas.
    pub fn node(&self, id: &str) -> Option<&Node> {
        self.nodes.iter().find(|n| n.id == id && !n.deleted)
    }

    /// The same node, to change.
    pub fn node_mut(&mut self, id: &str) -> Option<&mut Node> {
        self.nodes.iter_mut().find(|n| n.id == id && !n.deleted)
    }

    /// The note ID a note card points at.
    pub fn note_id(node: &Node) -> Option<&str> {
        node.file
            .as_deref()
            .filter(|file| file.starts_with(NOTE_PREFIX))
    }

    /// Checks what the editor relies on: every ID is there and unique (a
    /// tombstone holds its ID), live sizes are positive, every live arrow
    /// joins two cards (a deleted one hides it), and every live stroke is well formed.
    pub fn validate(&self) -> Result<(), String> {
        if self.nodes.len() > MAX_ITEMS || self.edges.len() > MAX_ITEMS {
            return Err("The canvas has too many items.".into());
        }
        let mut ids = HashSet::new();
        let mut stroke_ids = HashSet::new();
        for node in &self.nodes {
            if node.id.is_empty() {
                return Err("A card has no ID.".into());
            }
            if !ids.insert(node.id.as_str()) {
                return Err(format!("Two cards have the ID “{}”.", node.id));
            }
            for stroke in &node.strokes {
                if stroke.id.is_empty() || !stroke_ids.insert(stroke.id.as_str()) {
                    return Err(format!(
                        "A stroke in “{}” has a missing or repeated ID.",
                        node.id
                    ));
                }
            }
            if node.deleted {
                continue;
            }
            if node.kind.is_empty() {
                return Err(format!("Card “{}” has no type.", node.id));
            }
            if !(node.x.is_finite()
                && node.y.is_finite()
                && node.width.is_finite()
                && node.height.is_finite())
            {
                return Err(format!("Card “{}” is not at a real place.", node.id));
            }
            if !(node.width > 0. && node.height > 0.) {
                return Err(format!("Card “{}” has no size.", node.id));
            }
            if node.kind != INK && !node.strokes.is_empty() {
                return Err(format!("Card “{}” holds strokes but isn't ink.", node.id));
            }
            for stroke in node.strokes.iter().filter(|s| !s.deleted) {
                validate_stroke(stroke)?;
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
            if edge.deleted {
                continue;
            }
            // An end that is a deleted card is allowed: the arrow stays in the
            // data, hidden, so the merge doesn't depend on the order of edits.
            if !ids.contains(edge.from_node.as_str()) || !ids.contains(edge.to_node.as_str()) {
                return Err(format!(
                    "Arrow “{}” points at a card that isn't there.",
                    edge.id
                ));
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

    /// The canvas as an Obsidian `.canvas` document, cards only: tombstones
    /// are dropped, ink is left out, note cards name the note's title as a
    /// path and keep its ID beside it, the arrays follow stacking order and
    /// Kanso's bookkeeping fields are removed. Pure.
    pub fn to_obsidian(&self) -> Canvas {
        self.to_obsidian_with(&ExportFiles::default())
    }

    /// As [`to_obsidian`](Self::to_obsidian), but an ink node named in
    /// `files.ink` (node ID to the PNG's file name, rasterized by the
    /// exporter) becomes a `file` card at its ink bounds, and an attachment
    /// card named in `files.attachments` points at the file the exporter
    /// wrote. Pure.
    pub fn to_obsidian_with(&self, files: &ExportFiles) -> Canvas {
        let mut out = Canvas {
            extra: self.extra.clone(),
            ..Canvas::default()
        };
        let mut kept: HashSet<&str> = HashSet::new();
        for node in self.live_nodes() {
            let mut card = if node.kind == INK {
                let Some(file) = files.ink.get(&node.id) else {
                    continue;
                };
                let b = node.bounds();
                Node {
                    id: node.id.clone(),
                    kind: "file".into(),
                    x: b.x,
                    y: b.y,
                    width: b.width,
                    height: b.height,
                    file: Some(file.clone()),
                    ..Node::default()
                }
            } else {
                Node {
                    strokes: Vec::new(),
                    ..node.clone()
                }
            };
            card.kanso_version = 0;
            card.kanso_updated = 0;
            card.kanso_z = 0.0;
            if let Some(id) = Canvas::note_id(&card).map(str::to_owned) {
                let title = card.title.clone().unwrap_or_else(|| "Untitled".into());
                card.extra
                    .insert(interop::ID_FIELD.into(), Value::String(id));
                card.file = Some(export_path(&title));
            } else if let Some(id) = Canvas::attachment_id(&card).map(str::to_owned)
                && let Some(name) = files.attachments.get(&id)
            {
                card.extra
                    .insert(interop::ID_FIELD.into(), Value::String(id));
                card.file = Some(name.clone());
            }
            kept.insert(node.id.as_str());
            out.nodes.push(card);
        }
        let mut edges: Vec<&Edge> = self.edges.iter().filter(|e| !e.deleted).collect();
        edges.sort_by(|a, b| {
            a.kanso_z
                .total_cmp(&b.kanso_z)
                .then_with(|| a.id.cmp(&b.id))
        });
        for edge in edges {
            if kept.contains(edge.from_node.as_str()) && kept.contains(edge.to_node.as_str()) {
                out.edges.push(Edge {
                    kanso_version: 0,
                    kanso_updated: 0,
                    kanso_z: 0.0,
                    ..edge.clone()
                });
            }
        }
        out
    }

    /// An Obsidian `.canvas` file's text: [`to_obsidian_with`](Self::to_obsidian_with)
    /// with whole-number geometry, as Obsidian writes it. Pure.
    pub fn to_obsidian_json(&self, files: &ExportFiles) -> String {
        let canvas = self.to_obsidian_with(files);
        let mut value = serde_json::to_value(&canvas).unwrap_or(Value::Null);
        if let Some(nodes) = value.get_mut("nodes").and_then(Value::as_array_mut) {
            for node in nodes {
                for key in ["x", "y", "width", "height"] {
                    if let Some(n) = node.get(key).and_then(Value::as_f64) {
                        node[key] = Value::from(n.round() as i64);
                    }
                }
            }
        }
        serde_json::to_string_pretty(&value).unwrap_or_else(|_| "{}".into())
    }

    /// The canvas with fresh IDs where one is missing or repeated, so an
    /// edited copy can be written: what Kanso doesn't understand is kept.
    /// Arrows that join a card that isn't there are dropped. Pure.
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
            .retain(|e| e.deleted || (ids.contains(&e.from_node) && ids.contains(&e.to_node)));
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

fn validate_stroke(stroke: &Stroke) -> Result<(), String> {
    let id = &stroke.id;
    if !TOOLS.contains(&stroke.tool.as_str()) {
        return Err(format!("Stroke “{id}” uses no known pen."));
    }
    if !(stroke.width.is_finite() && stroke.width > 0. && stroke.width <= 500.) {
        return Err(format!("Stroke “{id}” has no real width."));
    }
    if stroke.points.is_empty() || stroke.points.len() > MAX_STROKE_POINTS {
        return Err(format!("Stroke “{id}” has too few or too many points."));
    }
    for p in &stroke.points {
        let tilt_ok = p.tilt.is_none_or(|(a, b)| a.is_finite() && b.is_finite());
        if !(p.x.is_finite() && p.y.is_finite() && (0.0..=1.0).contains(&p.pressure) && tilt_ok) {
            return Err(format!("Stroke “{id}” has a point that isn't real."));
        }
    }
    Ok(())
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
/// stores it, in compact JSON (the encoding is the size lever). Pure.
pub fn to_body(canvas: &Canvas) -> String {
    let json = serde_json::to_string(canvas).unwrap_or_else(|_| "{}".into());
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
    Some(rest[..close].trim())
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

/// What the store requires of a body that claims `doctype: "canvas"`: empty
/// (a new canvas, or one sealed by a lock) or a canvas that parses and fits
/// the hard size cap. Pure.
pub fn check_body(body: &str) -> Result<(), String> {
    if body.trim().is_empty() {
        return Ok(());
    }
    if body.len() > MAX_BYTES + 1024 {
        return Err("The canvas is over its 1 MB budget. Move some of it to a new canvas.".into());
    }
    parse(body).map(|_| ())
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
