//! Batches of operations on a canvas: what the local API's and MCP's
//! `kanso-edit-canvas` applies (issue #45, step 12), and the read views
//! `kanso-read-canvas` returns.
//!
//! A batch is atomic: [`apply`] works on a copy and hands it back only when
//! every operation held, so a refused batch changes nothing. Each operation
//! goes through the same stamped edits the canvas view uses (`put_node`,
//! `remove_stroke` and the rest), so the private merge, the shared
//! canvas's CRDT and history see an agent's edit like a person's. A
//! batch never rewrites the document as text.
//!
//! Versions: an operation may carry `ifVersion`, the `kansoVersion` of its
//! item as the agent read it. It is checked against the canvas as it was
//! before the batch, for every operation, and every stale item is reported
//! with its current version.

use std::collections::{HashMap, HashSet};

use serde::Deserialize;
use serde_json::Value;

use super::edit::{StrokeRef, unique_id};
use super::model::{Canvas, Edge, INK, Node, Point, Stroke, TOOLS};
use super::{MAX_BYTES, MAX_STROKE_POINTS, WARN_BYTES};

/// The most operations one batch may hold.
pub const MAX_OPS: usize = 500;

/// The largest coordinate or size an operation may set, in points.
const LIMIT: f64 = 1.0e7;

/// The longest text an operation may give a text card, in characters.
const MAX_TEXT: usize = 100_000;

/// The longest title, label or URL an operation may set, in bytes.
pub const MAX_FIELD_BYTES: usize = 4 * 1024;

/// The most strokes one batch may add, and the most points in all of them.
/// A canvas holds at most 1 MB, so more could never be saved; the limits
/// refuse such a batch before any work is done on it.
pub const MAX_BATCH_STROKES: usize = 10_000;
pub const MAX_BATCH_POINTS: usize = 200_000;

/// The most IDs one operation may list (strokes to remove or move, cards to
/// group).
pub const MAX_LISTED: usize = 20_000;

/// The largest batch, as JSON. Anything bigger could not fit the canvas.
pub const MAX_BATCH_BYTES: usize = 4 * MAX_BYTES;

/// Whether a shared canvas may take image (attachment) cards. Shared notes
/// do not carry attachments yet, so a card would be a placeholder for
/// everyone else: refused until "Sharing attachments of shared notes"
/// lands (open question 2 of issue #45). Flip this to allow them.
pub const SHARED_ATTACHMENT_CARDS: bool = false;

/// What the batch is refused with on a shared canvas that gets a picture.
pub const SHARED_ATTACHMENT_REFUSAL: &str = "Pictures can't be added to a shared canvas yet: shared canvases don't carry attachments, so everyone else would see an empty card.";

/// A batch: operations in order, and optionally the revision of the note
/// they were made against.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Batch {
    pub if_rev: Option<String>,
    pub ops: Vec<Op>,
}

/// One operation. `ifVersion` is the item's `kansoVersion` as the agent
/// read it.
#[derive(Clone, Debug, PartialEq, Deserialize)]
#[serde(
    tag = "op",
    rename_all = "camelCase",
    rename_all_fields = "camelCase",
    deny_unknown_fields
)]
pub enum Op {
    AddNode {
        #[serde(default)]
        id: Option<String>,
        #[serde(rename = "type")]
        kind: String,
        x: f64,
        y: f64,
        #[serde(default)]
        width: Option<f64>,
        #[serde(default)]
        height: Option<f64>,
        #[serde(default)]
        text: Option<String>,
        #[serde(default)]
        file: Option<String>,
        #[serde(default)]
        title: Option<String>,
        #[serde(default)]
        url: Option<String>,
        #[serde(default)]
        label: Option<String>,
        #[serde(default)]
        color: Option<String>,
    },
    UpdateNode {
        id: String,
        #[serde(default)]
        text: Option<String>,
        #[serde(default)]
        title: Option<String>,
        #[serde(default)]
        color: Option<String>,
        #[serde(default)]
        label: Option<String>,
        #[serde(default)]
        if_version: Option<u64>,
    },
    MoveNode {
        id: String,
        x: f64,
        y: f64,
        #[serde(default)]
        if_version: Option<u64>,
    },
    ResizeNode {
        id: String,
        width: f64,
        height: f64,
        #[serde(default)]
        if_version: Option<u64>,
    },
    RemoveNode {
        id: String,
        #[serde(default)]
        if_version: Option<u64>,
    },
    AddEdge {
        #[serde(default)]
        id: Option<String>,
        from_node: String,
        to_node: String,
        #[serde(default)]
        from_side: Option<String>,
        #[serde(default)]
        to_side: Option<String>,
        #[serde(default)]
        from_end: Option<String>,
        #[serde(default)]
        to_end: Option<String>,
        #[serde(default)]
        label: Option<String>,
        #[serde(default)]
        color: Option<String>,
    },
    UpdateEdge {
        id: String,
        #[serde(default)]
        from_node: Option<String>,
        #[serde(default)]
        to_node: Option<String>,
        #[serde(default)]
        from_side: Option<String>,
        #[serde(default)]
        to_side: Option<String>,
        #[serde(default)]
        from_end: Option<String>,
        #[serde(default)]
        to_end: Option<String>,
        #[serde(default)]
        label: Option<String>,
        #[serde(default)]
        color: Option<String>,
        #[serde(default)]
        if_version: Option<u64>,
    },
    RemoveEdge {
        id: String,
        #[serde(default)]
        if_version: Option<u64>,
    },
    AddStrokes {
        node: String,
        strokes: Vec<NewStroke>,
    },
    RemoveStrokes {
        node: String,
        strokes: Vec<StrokeId>,
    },
    MoveStrokes {
        node: String,
        strokes: Vec<StrokeId>,
        dx: f64,
        dy: f64,
    },
    SetGroup {
        nodes: Vec<String>,
        /// The group to put them in, or none to take them out of the group
        /// they are in.
        #[serde(default)]
        group: Option<String>,
    },
    RenameGroup {
        id: String,
        label: String,
    },
    Rename {
        title: String,
    },
}

/// A stroke to add. Points are `[x, y, pressure]` (or with a pencil's
/// tilt, five numbers) relative to the ink node, as they are stored.
#[derive(Clone, Debug, PartialEq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct NewStroke {
    #[serde(default)]
    pub tool: Option<String>,
    #[serde(default)]
    pub color: Option<String>,
    #[serde(default)]
    pub width: Option<f64>,
    pub points: Vec<Point>,
}

/// A stroke named by ID, with the version the agent read when it has one.
#[derive(Clone, Debug, PartialEq, Deserialize)]
#[serde(untagged)]
pub enum StrokeId {
    Plain(String),
    Versioned {
        id: String,
        #[serde(rename = "ifVersion")]
        if_version: u64,
    },
}

impl StrokeId {
    fn id(&self) -> &str {
        match self {
            StrokeId::Plain(id) | StrokeId::Versioned { id, .. } => id,
        }
    }

    fn version(&self) -> Option<u64> {
        match self {
            StrokeId::Plain(_) => None,
            StrokeId::Versioned { if_version, .. } => Some(*if_version),
        }
    }
}

impl Batch {
    /// Reads a batch from the request's JSON: `{ "ifRev": "…", "ops": […] }`.
    /// A bare array is a batch of those operations.
    pub fn from_json(value: &Value) -> Result<Batch, String> {
        let (if_rev, ops) = match value {
            Value::Array(ops) => (None, ops),
            Value::Object(fields) => {
                if let Some(extra) = fields
                    .keys()
                    .find(|k| !matches!(k.as_str(), "ifRev" | "ops"))
                {
                    return Err(format!(
                        "A batch has no “{extra}”: it has “ops” and “ifRev”."
                    ));
                }
                let if_rev = match fields.get("ifRev") {
                    None | Some(Value::Null) => None,
                    Some(Value::String(rev)) => Some(rev.clone()),
                    Some(_) => return Err("“ifRev” must be a revision string.".into()),
                };
                let Some(Value::Array(ops)) = fields.get("ops") else {
                    return Err("A batch needs an “ops” list.".into());
                };
                (if_rev, ops)
            }
            _ => return Err("A batch is an object with an “ops” list.".into()),
        };
        if ops.len() > MAX_OPS {
            return Err(format!("A batch holds at most {MAX_OPS} operations."));
        }
        let bytes: usize = ops
            .iter()
            .map(|op| serde_json::to_vec(op).map_or(usize::MAX, |v| v.len()))
            .fold(0, usize::saturating_add);
        if bytes > MAX_BATCH_BYTES {
            return Err(format!(
                "A batch is at most {MAX_BATCH_BYTES} bytes of JSON: a canvas holds at most {MAX_BYTES}. Split it into smaller batches."
            ));
        }
        let ops = ops
            .iter()
            .enumerate()
            .map(|(at, op)| {
                serde_json::from_value::<Op>(op.clone())
                    .map_err(|err| format!("Operation {} isn't valid: {err}", at + 1))
            })
            .collect::<Result<_, _>>()?;
        let batch = Batch { if_rev, ops };
        batch.check_size()?;
        Ok(batch)
    }

    /// Refuses a batch that adds more strokes or points, or lists more IDs,
    /// than a canvas could ever hold.
    pub fn check_size(&self) -> Result<(), String> {
        let (mut strokes, mut points) = (0usize, 0usize);
        for (at, op) in self.ops.iter().enumerate() {
            let listed = match op {
                Op::AddStrokes { strokes: new, .. } => {
                    strokes = strokes.saturating_add(new.len());
                    points = new
                        .iter()
                        .fold(points, |n, s| n.saturating_add(s.points.len()));
                    0
                }
                Op::RemoveStrokes { strokes: ids, .. } | Op::MoveStrokes { strokes: ids, .. } => {
                    ids.len()
                }
                Op::SetGroup { nodes, .. } => nodes.len(),
                _ => 0,
            };
            if listed > MAX_LISTED {
                return Err(format!(
                    "Operation {} lists more than {MAX_LISTED} items.",
                    at + 1
                ));
            }
        }
        if strokes > MAX_BATCH_STROKES || points > MAX_BATCH_POINTS {
            return Err(format!(
                "A batch adds at most {MAX_BATCH_STROKES} strokes and {MAX_BATCH_POINTS} points. Split it into smaller batches."
            ));
        }
        Ok(())
    }
}

/// What a batch needs to know about the library.
pub struct Context<'a> {
    /// Milliseconds since the epoch: the stamp of every change.
    pub now: i64,
    /// The canvas is shared: its edits go to the CRDT.
    pub shared: bool,
    /// Whether `note:…` is a note on this device.
    pub note_exists: &'a dyn Fn(&str) -> bool,
    /// Whether `file:…` is one of this note's attachments.
    pub attachment_ok: &'a dyn Fn(&str) -> bool,
}

/// What a batch made.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Applied {
    pub nodes: Vec<String>,
    pub edges: Vec<String>,
    pub strokes: Vec<String>,
    /// A new title for the note, when the batch renamed it.
    pub title: Option<String>,
}

/// An item whose version isn't the one the agent read.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Stale {
    pub id: String,
    /// The item's version now; 0 for an item that isn't there.
    pub version: u64,
}

/// Why a batch changed nothing.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Refusal {
    /// An operation asked for something the canvas can't hold.
    Invalid(String),
    /// An operation named an item that isn't there.
    Missing(String),
    /// Items changed since the agent read them.
    Stale(Vec<Stale>),
    /// The result would be over the canvas's 1 MB budget.
    Full(String),
}

impl Refusal {
    /// One plain sentence for a person or an agent.
    pub fn message(&self) -> String {
        match self {
            Refusal::Invalid(why) | Refusal::Missing(why) | Refusal::Full(why) => why.clone(),
            Refusal::Stale(items) => {
                let list = items
                    .iter()
                    .map(|s| format!("{} (now version {})", s.id, s.version))
                    .collect::<Vec<_>>()
                    .join(", ");
                format!(
                    "These changed since you read them: {list}. Read the canvas again and redo the edit."
                )
            }
        }
    }
}

/// Applies `batch` to `canvas`, all or none. Returns the new canvas and what
/// the batch made. Pure besides `ctx`'s lookups.
pub fn apply(canvas: &Canvas, batch: &Batch, ctx: &Context) -> Result<(Canvas, Applied), Refusal> {
    batch.check_size().map_err(Refusal::Full)?;
    stale_check(canvas, &batch.ops)?;
    let mut next = canvas.clone();
    let mut made = Applied::default();
    for (at, op) in batch.ops.iter().enumerate() {
        run(&mut next, op, ctx, &mut made).map_err(|refusal| match refusal {
            Refusal::Invalid(why) => Refusal::Invalid(format!("Operation {}: {why}", at + 1)),
            Refusal::Missing(why) => Refusal::Missing(format!("Operation {}: {why}", at + 1)),
            other => other,
        })?;
    }
    next.refit_ink();
    next.validate()
        .map_err(|why| Refusal::Invalid(format!("The result isn't a valid canvas: {why}")))?;
    let bytes = serde_json::to_vec(&next).map_or(0, |v| v.len());
    if bytes >= MAX_BYTES {
        return Err(Refusal::Full(format!(
            "The canvas would be {bytes} bytes; its budget is {MAX_BYTES} (it warns at {WARN_BYTES}). Move some of it to a new canvas."
        )));
    }
    Ok((next, made))
}

/// Refuses the batch when an item it carries `ifVersion` for has another
/// version now, naming all of them.
fn stale_check(canvas: &Canvas, ops: &[Op]) -> Result<(), Refusal> {
    let nodes: HashMap<&str, u64> = canvas
        .nodes
        .iter()
        .map(|n| (n.id.as_str(), n.kanso_version))
        .collect();
    let edges: HashMap<&str, u64> = canvas
        .edges
        .iter()
        .map(|e| (e.id.as_str(), e.kanso_version))
        .collect();
    let strokes: HashMap<&str, u64> = canvas
        .nodes
        .iter()
        .flat_map(|n| n.strokes.iter())
        .map(|s| (s.id.as_str(), s.kanso_version))
        .collect();
    let mut stale: Vec<Stale> = Vec::new();
    let mut check = |id: &str, want: Option<u64>, have: &HashMap<&str, u64>| {
        let Some(want) = want else { return };
        let now = have.get(id).copied().unwrap_or(0);
        if now != want && !stale.iter().any(|s| s.id == id) {
            stale.push(Stale {
                id: id.to_owned(),
                version: now,
            });
        }
    };
    for op in ops {
        match op {
            Op::UpdateNode { id, if_version, .. }
            | Op::MoveNode { id, if_version, .. }
            | Op::ResizeNode { id, if_version, .. }
            | Op::RemoveNode { id, if_version } => check(id, *if_version, &nodes),
            Op::UpdateEdge { id, if_version, .. } | Op::RemoveEdge { id, if_version } => {
                check(id, *if_version, &edges);
            }
            Op::RemoveStrokes { strokes: list, .. } | Op::MoveStrokes { strokes: list, .. } => {
                for s in list {
                    check(s.id(), s.version(), &strokes);
                }
            }
            _ => {}
        }
    }
    if stale.is_empty() {
        Ok(())
    } else {
        Err(Refusal::Stale(stale))
    }
}

fn run(c: &mut Canvas, op: &Op, ctx: &Context, made: &mut Applied) -> Result<(), Refusal> {
    let now = ctx.now;
    match op {
        Op::AddNode {
            id,
            kind,
            x,
            y,
            width,
            height,
            text,
            file,
            title,
            url,
            label,
            color,
        } => {
            let id = new_id(id.as_deref(), "card", c.nodes.iter().map(|n| n.id.as_str()))?;
            let (dw, dh) = default_size(kind);
            let mut node = Node {
                id: id.clone(),
                kind: kind.clone(),
                x: coordinate(*x, "x")?,
                y: coordinate(*y, "y")?,
                width: size(width.unwrap_or(dw), "width")?,
                height: size(height.unwrap_or(dh), "height")?,
                color: color_of(color.as_deref())?,
                ..Node::default()
            };
            match kind.as_str() {
                "text" => node.text = Some(words(text.as_deref().unwrap_or_default(), "text")?),
                "file" => {
                    let target = file
                        .as_deref()
                        .ok_or_else(|| invalid("A file card needs “file”: a note ID or one of this note's attachments."))?;
                    node.file = Some(card_target(target, ctx)?);
                    node.title = title.as_deref().map(|t| words(t, "title")).transpose()?;
                }
                "link" => {
                    let link = url
                        .as_deref()
                        .ok_or_else(|| invalid("A link card needs “url”."))?;
                    if !(link.starts_with("https://") || link.starts_with("http://")) {
                        return Err(invalid(
                            "A link card's “url” must start with http:// or https://.",
                        ));
                    }
                    node.url = Some(words(link, "url")?);
                }
                "group" => node.label = label.as_deref().map(|l| words(l, "label")).transpose()?,
                "ink" => {
                    node.width = node.width.max(1.0);
                    node.height = node.height.max(1.0);
                }
                other => {
                    return Err(invalid(format!(
                        "“{other}” isn't a card type: use text, file, link, group or ink."
                    )));
                }
            }
            if kind != "text" && text.is_some()
                || kind != "file" && (file.is_some() || title.is_some())
                || kind != "link" && url.is_some()
                || kind != "group" && label.is_some()
            {
                return Err(invalid(format!(
                    "That field doesn't belong on a {kind} card."
                )));
            }
            c.put_node(node, now);
            made.nodes.push(id);
        }
        Op::UpdateNode {
            id,
            text,
            title,
            color,
            label,
            ..
        } => {
            let mut node = live_node(c, id)?.clone();
            if node.kind == INK {
                return Err(invalid("An ink layer has no fields to update."));
            }
            if let Some(text) = text {
                if node.kind != "text" {
                    return Err(invalid("Only a text card has “text”."));
                }
                node.text = Some(words(text, "text")?);
            }
            if let Some(title) = title {
                if node.kind != "file" {
                    return Err(invalid("Only a file or note card has “title”."));
                }
                node.title = Some(words(title, "title")?);
            }
            if let Some(label) = label {
                if node.kind != "group" {
                    return Err(invalid("Only a group has “label”."));
                }
                node.label = Some(words(label, "label")?);
            }
            if color.is_some() {
                node.color = color_of(color.as_deref())?;
            }
            c.put_node(node, now);
        }
        Op::MoveNode { id, x, y, .. } => {
            let node = live_node(c, id)?;
            if node.kind == INK {
                return Err(invalid("An ink layer can't be moved: use moveStrokes."));
            }
            let (x, y) = (coordinate(*x, "x")?, coordinate(*y, "y")?);
            c.move_node(id, x, y, now);
        }
        Op::ResizeNode {
            id, width, height, ..
        } => {
            let mut node = live_node(c, id)?.clone();
            if node.kind == INK {
                return Err(invalid("An ink layer's size follows its strokes."));
            }
            node.width = size(*width, "width")?;
            node.height = size(*height, "height")?;
            c.put_node(node, now);
        }
        Op::RemoveNode { id, .. } => {
            if live_node(c, id)?.kind == INK {
                return Err(invalid("An ink layer can't be removed: use removeStrokes."));
            }
            c.remove_node(id, now);
            // Arrows to a removed card go with it.
            let touching: Vec<String> = c
                .edges
                .iter()
                .filter(|e| !e.deleted && (e.from_node == *id || e.to_node == *id))
                .map(|e| e.id.clone())
                .collect();
            for edge in touching {
                c.remove_edge(&edge, now);
            }
        }
        Op::AddEdge {
            id,
            from_node,
            to_node,
            from_side,
            to_side,
            from_end,
            to_end,
            label,
            color,
        } => {
            let id = new_id(id.as_deref(), "edge", c.edges.iter().map(|e| e.id.as_str()))?;
            for end in [from_node, to_node] {
                live_node(c, end)?;
            }
            let edge = Edge {
                id: id.clone(),
                from_node: from_node.clone(),
                to_node: to_node.clone(),
                from_side: side(from_side.as_deref())?,
                to_side: side(to_side.as_deref())?,
                from_end: end(from_end.as_deref())?,
                to_end: end(to_end.as_deref())?,
                label: label.as_deref().map(|l| words(l, "label")).transpose()?,
                color: color_of(color.as_deref())?,
                ..Edge::default()
            };
            c.put_edge(edge, now);
            made.edges.push(id);
        }
        Op::UpdateEdge {
            id,
            from_node,
            to_node,
            from_side,
            to_side,
            from_end,
            to_end,
            label,
            color,
            ..
        } => {
            let mut edge = c
                .edges
                .iter()
                .find(|e| e.id == *id && !e.deleted)
                .cloned()
                .ok_or_else(|| Refusal::Missing(format!("There is no arrow “{id}”.")))?;
            if let Some(n) = from_node {
                live_node(c, n)?;
                edge.from_node = n.clone();
            }
            if let Some(n) = to_node {
                live_node(c, n)?;
                edge.to_node = n.clone();
            }
            if from_side.is_some() {
                edge.from_side = side(from_side.as_deref())?;
            }
            if to_side.is_some() {
                edge.to_side = side(to_side.as_deref())?;
            }
            if from_end.is_some() {
                edge.from_end = end(from_end.as_deref())?;
            }
            if to_end.is_some() {
                edge.to_end = end(to_end.as_deref())?;
            }
            if let Some(label) = label {
                edge.label = Some(words(label, "label")?);
            }
            if color.is_some() {
                edge.color = color_of(color.as_deref())?;
            }
            c.put_edge(edge, now);
        }
        Op::RemoveEdge { id, .. } => {
            if !c.remove_edge(id, now) {
                return Err(Refusal::Missing(format!("There is no arrow “{id}”.")));
            }
        }
        Op::AddStrokes { node, strokes } => {
            live_ink(c, node)?;
            if strokes.is_empty() {
                return Err(invalid("addStrokes needs at least one stroke."));
            }
            let mut taken: HashSet<String> =
                c.stroke_ids().into_iter().map(str::to_owned).collect();
            for new in strokes {
                if new.points.is_empty() || new.points.len() > MAX_STROKE_POINTS {
                    return Err(invalid(format!(
                        "A stroke has 1 to {MAX_STROKE_POINTS} points."
                    )));
                }
                let tool = new.tool.clone().unwrap_or_else(|| "pen".into());
                if !TOOLS.contains(&tool.as_str()) {
                    return Err(invalid(format!(
                        "“{tool}” isn't a pen: use {}.",
                        TOOLS.join(", ")
                    )));
                }
                let width = new.width.unwrap_or(2.0);
                if !(width.is_finite() && width > 0.0 && width <= 500.0) {
                    return Err(invalid("A stroke's width is above 0 and at most 500."));
                }
                for p in &new.points {
                    if !(p.x.is_finite()
                        && p.y.is_finite()
                        && p.x.abs() <= LIMIT
                        && p.y.abs() <= LIMIT
                        && (0.0..=1.0).contains(&p.pressure))
                    {
                        return Err(invalid(
                            "A point is [x, y, pressure] with finite coordinates and pressure from 0 to 1.",
                        ));
                    }
                }
                let id = unique_id("s", &taken);
                taken.insert(id.clone());
                let stroke = Stroke {
                    id: id.clone(),
                    tool,
                    color: color_of(new.color.as_deref())?.unwrap_or_else(|| "#000000".into()),
                    width,
                    points: new.points.clone(),
                    ..Stroke::default()
                };
                if !c.add_stroke(node, stroke, now) {
                    return Err(invalid("The stroke couldn't be added."));
                }
                made.strokes.push(id);
            }
        }
        Op::RemoveStrokes { node, strokes } => {
            live_ink(c, node)?;
            for s in strokes {
                if !c.remove_stroke(node, s.id(), now) {
                    return Err(Refusal::Missing(format!(
                        "There is no stroke “{}”.",
                        s.id()
                    )));
                }
            }
        }
        Op::MoveStrokes {
            node,
            strokes,
            dx,
            dy,
        } => {
            live_ink(c, node)?;
            if !(dx.is_finite() && dy.is_finite() && dx.abs() <= LIMIT && dy.abs() <= LIMIT) {
                return Err(invalid("dx and dy must be finite."));
            }
            for s in strokes {
                let live = c
                    .nodes
                    .iter()
                    .filter(|n| n.id == *node)
                    .flat_map(|n| n.strokes.iter())
                    .any(|x| x.id == s.id() && !x.deleted);
                if !live {
                    return Err(Refusal::Missing(format!(
                        "There is no stroke “{}”.",
                        s.id()
                    )));
                }
            }
            let refs: Vec<StrokeRef> = strokes
                .iter()
                .map(|s| StrokeRef {
                    node: node.clone(),
                    stroke: s.id().to_owned(),
                })
                .collect();
            // A stroke moves as the canvas view moves it: the original is
            // tombstoned and a translated copy gets a fresh ID.
            if let Some(action) = c.move_strokes(&refs, *dx, *dy, now) {
                made.strokes
                    .extend(action.added.iter().map(|(_, s)| s.id.clone()));
            }
        }
        Op::SetGroup { nodes, group } => {
            let area = match group {
                Some(group) => Some(
                    c.group_bounds(group)
                        .ok_or_else(|| Refusal::Missing(format!("There is no group “{group}”.")))?,
                ),
                None => None,
            };
            for id in nodes {
                let node = live_node(c, id)?.clone();
                if node.kind == INK || node.kind == "group" {
                    return Err(invalid("Only cards can be put in or taken out of a group."));
                }
                let b = node.bounds();
                let (x, y) = match area {
                    // Into the group: the nearest place inside it.
                    Some(a) => (
                        clamp_into(b.x, b.width, a.x, a.width),
                        clamp_into(b.y, b.height, a.y, a.height),
                    ),
                    // Out of the group it is in: just to its right.
                    None => match c
                        .nodes
                        .iter()
                        .filter(|g| g.kind == "group" && !g.deleted && g.id != *id)
                        .map(Node::bounds)
                        .find(|g| g.contains(&b))
                    {
                        Some(g) => (g.right() + 20.0, b.y),
                        None => continue,
                    },
                };
                if (x, y) != (b.x, b.y) {
                    c.move_node(id, x, y, now);
                }
            }
        }
        Op::RenameGroup { id, label } => {
            let mut node = live_node(c, id)?.clone();
            if node.kind != "group" {
                return Err(invalid(format!("“{id}” isn't a group.")));
            }
            node.label = Some(words(label, "label")?);
            c.put_node(node, now);
        }
        Op::Rename { title } => {
            let title = title.trim();
            if title.is_empty() || title.chars().count() > 500 {
                return Err(invalid("A title has 1 to 500 characters."));
            }
            made.title = Some(title.to_owned());
        }
    }
    Ok(())
}

fn invalid(why: impl Into<String>) -> Refusal {
    Refusal::Invalid(why.into())
}

fn live_node<'a>(c: &'a Canvas, id: &str) -> Result<&'a Node, Refusal> {
    c.node(id)
        .ok_or_else(|| Refusal::Missing(format!("There is no card “{id}”.")))
}

fn live_ink(c: &Canvas, id: &str) -> Result<(), Refusal> {
    match c.node(id) {
        Some(n) if n.kind == INK => Ok(()),
        Some(_) => Err(invalid(format!("“{id}” isn't an ink layer."))),
        None => Err(Refusal::Missing(format!("There is no ink layer “{id}”."))),
    }
}

/// The ID asked for (which must be free, tombstones included), or a fresh one.
fn new_id<'a>(
    asked: Option<&str>,
    prefix: &str,
    taken: impl Iterator<Item = &'a str>,
) -> Result<String, Refusal> {
    let taken: HashSet<String> = taken.map(str::to_owned).collect();
    match asked {
        None => Ok(unique_id(&format!("{prefix}-"), &taken)),
        Some(id) => {
            let fine = !id.is_empty()
                && id.len() <= 64
                && id
                    .chars()
                    .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_');
            if !fine {
                Err(invalid(
                    "An ID is 1 to 64 letters, digits, dashes or underscores.",
                ))
            } else if taken.contains(id) {
                Err(invalid(format!("The ID “{id}” is already used.")))
            } else {
                Ok(id.to_owned())
            }
        }
    }
}

fn default_size(kind: &str) -> (f64, f64) {
    match kind {
        "group" => (400.0, 300.0),
        "file" => (400.0, 300.0),
        "ink" => (1.0, 1.0),
        _ => (250.0, 60.0),
    }
}

fn coordinate(v: f64, name: &str) -> Result<f64, Refusal> {
    if v.is_finite() && v.abs() <= LIMIT {
        Ok(v)
    } else {
        Err(invalid(format!("“{name}” must be a real number.")))
    }
}

fn size(v: f64, name: &str) -> Result<f64, Refusal> {
    if v.is_finite() && v > 0.0 && v <= LIMIT {
        Ok(v)
    } else {
        Err(invalid(format!("“{name}” must be above 0.")))
    }
}

/// A text card's text, or a title, label or URL, within its limit. A text
/// card's text goes to a shared canvas's room in pieces; every other field
/// goes whole, so it is kept far under the room's 256 KB per change.
fn words(s: &str, name: &str) -> Result<String, Refusal> {
    let too_long = if name == "text" {
        s.chars().count() > MAX_TEXT
    } else {
        s.len() > MAX_FIELD_BYTES
    };
    if !too_long {
        Ok(s.to_owned())
    } else if name == "text" {
        Err(invalid(format!(
            "“{name}” is longer than {MAX_TEXT} characters."
        )))
    } else {
        Err(invalid(format!(
            "“{name}” is longer than {MAX_FIELD_BYTES} bytes."
        )))
    }
}

/// A colour is a preset "1" to "6" or `#rgb`, `#rrggbb` or `#rrggbbaa`.
fn color_of(c: Option<&str>) -> Result<Option<String>, Refusal> {
    let Some(c) = c else { return Ok(None) };
    let preset = matches!(c, "1" | "2" | "3" | "4" | "5" | "6");
    let hex = c.strip_prefix('#').is_some_and(|h| {
        matches!(h.len(), 3 | 4 | 6 | 8) && h.chars().all(|d| d.is_ascii_hexdigit())
    });
    if preset || hex {
        Ok(Some(c.to_owned()))
    } else {
        Err(invalid(
            "A colour is a preset from \"1\" to \"6\" or a hex colour like \"#cc3300\".",
        ))
    }
}

fn side(s: Option<&str>) -> Result<Option<String>, Refusal> {
    match s {
        None => Ok(None),
        Some(s @ ("top" | "right" | "bottom" | "left")) => Ok(Some(s.to_owned())),
        Some(other) => Err(invalid(format!(
            "“{other}” isn't a side: use top, right, bottom or left."
        ))),
    }
}

fn end(s: Option<&str>) -> Result<Option<String>, Refusal> {
    match s {
        None => Ok(None),
        Some(s @ ("none" | "arrow")) => Ok(Some(s.to_owned())),
        Some(other) => Err(invalid(format!(
            "“{other}” isn't an arrow end: use none or arrow."
        ))),
    }
}

/// What a file card may point at: a note on this device, or (outside a
/// shared canvas) one of this canvas's attachments.
fn card_target(target: &str, ctx: &Context) -> Result<String, Refusal> {
    if target.starts_with(super::NOTE_PREFIX) {
        if (ctx.note_exists)(target) {
            Ok(target.to_owned())
        } else {
            Err(Refusal::Missing(format!("There is no note “{target}”.")))
        }
    } else if target.starts_with(super::ATTACHMENT_PREFIX) {
        if ctx.shared && !SHARED_ATTACHMENT_CARDS {
            Err(invalid(SHARED_ATTACHMENT_REFUSAL))
        } else if (ctx.attachment_ok)(target) {
            Ok(target.to_owned())
        } else {
            Err(invalid(format!(
                "“{target}” isn't an attachment of this canvas."
            )))
        }
    } else {
        Err(invalid(
            "A file card points at a note (note:…) or one of this canvas's attachments (file:…).",
        ))
    }
}

/// The start of a span of `len` that keeps it inside `lo..lo + room`, as near
/// to `at` as it can; a span wider than the room starts at `lo`. The last
/// start, `lo + room - len`, can round below `lo` for a span a hair
/// narrower than the room, and `f64::clamp` panics when its bounds cross,
/// so that span starts at `lo` too.
fn clamp_into(at: f64, len: f64, lo: f64, room: f64) -> f64 {
    let last = lo + room - len;
    if len >= room || last < lo {
        lo
    } else {
        at.clamp(lo, last)
    }
}

impl Canvas {
    /// The canvas without its tombstones: what an agent reads. Versions stay.
    /// Pure.
    pub fn live(&self) -> Canvas {
        let mut out = self.clone();
        out.nodes.retain(|n| !n.deleted);
        for node in &mut out.nodes {
            node.strokes.retain(|s| !s.deleted);
        }
        out.edges.retain(|e| !e.deleted);
        out
    }

    /// The live part of the canvas inside group `group`'s rectangle: the
    /// group, the cards wholly in it, the strokes wholly in it (their ink
    /// nodes cut down to those) and the arrows between what is left. `None`
    /// when there is no such group. Pure.
    pub fn group_view(&self, group: &str) -> Option<Canvas> {
        let area = self.group_bounds(group)?;
        let live = self.live();
        let mut keep: HashSet<String> = HashSet::new();
        let mut nodes = Vec::new();
        for mut node in live.nodes {
            if node.kind == INK {
                let (ox, oy) = (node.x, node.y);
                node.strokes.retain(|s| {
                    s.bounds().is_some_and(|b| {
                        area.contains(&super::Bounds {
                            x: b.x + ox,
                            y: b.y + oy,
                            ..b
                        })
                    })
                });
                if node.strokes.is_empty() {
                    continue;
                }
            } else if node.id != group && !area.contains(&node.bounds()) {
                continue;
            }
            keep.insert(node.id.clone());
            nodes.push(node);
        }
        let edges = live
            .edges
            .into_iter()
            .filter(|e| keep.contains(&e.from_node) && keep.contains(&e.to_node))
            .collect();
        Some(Canvas {
            nodes,
            edges,
            ..Canvas {
                kanso_sketch: live.kanso_sketch,
                kanso_sketch_id: live.kanso_sketch_id,
                kanso_ink_ocr: live.kanso_ink_ocr,
                extra: live.extra,
                ..Canvas::default()
            }
        })
    }
}
