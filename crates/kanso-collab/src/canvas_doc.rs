//! A shared canvas's CRDT (issue #45, section 4): the other module that knows
//! Yrs. A canvas is not text, so it is not a text CRDT: its items are cards,
//! arrows and pen strokes, each a map entry keyed by its ID.
//!
//! ```text
//! root
//! ├─ title:   Y.Text
//! ├─ meta:    Y.Map                    schema, top (the document's other fields)
//! ├─ nodes:   Y.Map<id, Y.Map>         the card's fields; `text` is a Y.Text
//! ├─ edges:   Y.Map<id, Y.Map>         the arrow's fields
//! └─ strokes: Y.Map<id, Y.Map>         ink (the ink node's ID), the pen's fields, done
//!      └─ points: Y.Array<Any>         appended `[x, y, p]` or `[x, y, p, az, alt]`
//! ```
//!
//! - **IDs, not arrays.** Two people adding a card at once make two keys; a
//!   move or a delete names its item.
//! - **Fields are last-writer values.** Each field of an item is one map
//!   key, so two people changing different fields of a card both win. Fields
//!   Kanso doesn't know are kept (they are map keys like any other).
//! - **Deletes are tombstones.** A deleted item keeps its key and gets
//!   `deleted: true`; nothing here ever writes `deleted: false` over a
//!   tombstone except a person's own undo ([`CanvasDoc::edit`]), so a device
//!   that was offline cannot bring an item back by merging.
//! - **Points are appended, by the stroke's author only**, a chunk at a time
//!   while the pen is down ([`CanvasDoc::stream`]); a stroke that is erased
//!   under the pen stops growing.
//! - **The projection is always a canvas the view can open.** What the
//!   document holds is untrusted (any member writes to it): [`CanvasDoc::to_canvas`]
//!   reads it and repairs what it must ([`Canvas::sanitized`]), so a forged or
//!   buggy update can hide an item but never break the canvas.
//!
//! Every local change reaches the CRDT as the difference between the canvas
//! the person's view last had ([`CanvasDoc::view`]) and the one it has now
//! ([`CanvasDoc::edit`]), written as updates under the server's cap.

use std::collections::HashMap;
use std::sync::Arc;

use serde::Serialize;
use serde_json::{Map, Value};
use yrs::updates::decoder::Decode;
use yrs::{
    Any, Array, ArrayPrelim, ArrayRef, Doc, GetString, Map as _, MapPrelim, MapRef, OffsetKind,
    Options, Out, ReadTxn, StateVector, Text, TextPrelim, TextRef, Transact, TransactionMut,
    Update,
};

use kanso_core::canvas::{Canvas, Edge, Node, Point, Stroke};

use crate::doc::{DocError, client_id, edits, units};
use crate::engine::rebase;

/// The layout this module writes.
const SCHEMA: i64 = 1;

/// The most points of one stroke that stream while the pen is down; the
/// rest reach everyone when the stroke ends.
pub const MAX_STREAM_POINTS: usize = 4_000;

/// The most bytes of a card's text written in one update.
const TEXT_PART: usize = 32 * 1024;
/// Points written to the array in one go.
const POINTS_PER_PUSH: usize = 512;
/// What a field adds to an update besides its value, at most.
const FIELD_COST: usize = 40;
/// What a point adds to an update, at most.
const POINT_COST: usize = 48;

/// Map keys the card's fields never hold: its ID is the key, its strokes are
/// top level, and its text is a text.
const NODE_RESERVED: &[&str] = &["id", "strokes", "text"];
const EDGE_RESERVED: &[&str] = &["id"];
/// A stroke's map also holds what isn't the stroke's own: its ink node, whether
/// it ended and who drew it.
const STROKE_RESERVED: &[&str] = &["id", "points", "ink", "done", "owner"];

/// What a tombstone keeps.
const KEEP: &[&str] = &["deleted", "kansoVersion", "kansoUpdated", "kansoZ", "ink"];

/// A shared canvas.
pub struct CanvasDoc {
    doc: Doc,
    title: TextRef,
    meta: MapRef,
    nodes: MapRef,
    edges: MapRef,
    strokes: MapRef,
}

impl std::fmt::Debug for CanvasDoc {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let txn = self.doc.transact();
        f.debug_struct("CanvasDoc")
            .field("nodes", &self.nodes.len(&txn))
            .field("edges", &self.edges.len(&txn))
            .field("strokes", &self.strokes.len(&txn))
            .finish()
    }
}

impl Default for CanvasDoc {
    fn default() -> CanvasDoc {
        CanvasDoc::new()
    }
}

/// The head of a stroke being streamed: what the stroke is, beside its
/// points.
#[derive(Clone, Debug, PartialEq)]
pub struct StrokeHead {
    pub id: String,
    /// The ink node it belongs to; its points are relative to that node.
    pub ink: String,
    /// Who is drawing it (an account ID): only the author appends.
    pub owner: String,
    pub tool: String,
    pub color: String,
    pub width: f64,
    pub z: f64,
}

/// What streaming points wrote.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Streamed {
    /// The update to send, if anything was written.
    pub update: Option<Vec<u8>>,
    /// The stroke holds as many points as stream ([`MAX_STREAM_POINTS`]):
    /// the rest of it goes when the stroke ends.
    pub full: bool,
}

/// The stroke was erased (by anyone) while it was being drawn: it stops
/// growing, the tombstone wins.
#[derive(Clone, Copy, Debug, PartialEq, Eq, thiserror::Error)]
#[error("the stroke was erased")]
pub struct Erased;

impl CanvasDoc {
    /// An empty canvas.
    pub fn new() -> CanvasDoc {
        let doc = Doc::with_options(Options {
            client_id: client_id(),
            offset_kind: OffsetKind::Utf16,
            ..Options::default()
        });
        let title = doc.get_or_insert_text("title");
        let meta = doc.get_or_insert_map("meta");
        let nodes = doc.get_or_insert_map("nodes");
        let edges = doc.get_or_insert_map("edges");
        let strokes = doc.get_or_insert_map("strokes");
        CanvasDoc {
            doc,
            title,
            meta,
            nodes,
            edges,
            strokes,
        }
    }

    /// A document seeded from a canvas (and the note's title): what sharing a
    /// canvas starts from. Tombstones come along, so a copy of the canvas
    /// that is still around cannot bring a deleted item back.
    pub fn from_canvas(title: &str, canvas: &Canvas) -> CanvasDoc {
        let doc = CanvasDoc::new();
        {
            let mut txn = doc.doc.transact_mut();
            doc.meta.insert(&mut txn, "schema", SCHEMA);
        }
        doc.set_title(title);
        // Parts of any size: the state is what is sent, not these.
        doc.edit(&Canvas::default(), canvas, usize::MAX);
        doc
    }

    /// A document from its encoded state ([`CanvasDoc::encode_state`]).
    pub fn load(state: &[u8]) -> Result<CanvasDoc, DocError> {
        let doc = CanvasDoc::new();
        doc.apply(state)?;
        Ok(doc)
    }

    /// The whole state, as an update from the empty state.
    pub fn encode_state(&self) -> Vec<u8> {
        self.doc
            .transact()
            .encode_state_as_update_v1(&StateVector::default())
    }

    /// Merges an update (from anyone, in any order, any number of times).
    /// Whether it changed anything this document had.
    pub fn apply(&self, update: &[u8]) -> Result<bool, DocError> {
        let update = Update::decode_v1(update).map_err(|_| DocError::Malformed)?;
        let before = self.doc.transact().snapshot();
        {
            let mut txn = self.doc.transact_mut();
            txn.apply_update(update).map_err(|_| DocError::Malformed)?;
        }
        Ok(self.doc.transact().snapshot() != before)
    }

    // ---------------------------------------------------------------- title

    pub fn title(&self) -> String {
        self.title.get_string(&self.doc.transact())
    }

    /// Makes the title read `new`. The update to send, or `None` when it
    /// already does.
    pub fn set_title(&self, new: &str) -> Option<Vec<u8>> {
        let old = self.title();
        if old == new {
            return None;
        }
        let mut txn = self.doc.transact_mut();
        rewrite(&mut txn, &self.title, &old, new);
        Some(txn.encode_update_v1())
    }

    // ----------------------------------------------------------- projection

    /// The canvas as the library stores it: what everyone sees. Deleted
    /// items are left out; the cards are in stacking order, each ink node
    /// holds the strokes drawn on it and has the size of its ink. Always a
    /// canvas that [`Canvas::validate`] accepts.
    pub fn to_canvas(&self) -> Canvas {
        self.read(false)
    }

    /// The canvas as a person's view holds it: [`to_canvas`](Self::to_canvas)
    /// with the tombstones, which the view needs to undo an erase and which
    /// [`edit`](Self::edit) needs to tell a delete from something never seen.
    pub fn view(&self) -> Canvas {
        self.read(true)
    }

    fn read(&self, tombstones: bool) -> Canvas {
        let txn = self.doc.transact();
        let mut canvas = Canvas::default();
        if let Some(Out::Any(Any::String(top))) = self.meta.get(&txn, "top")
            && let Ok(Value::Object(mut fields)) = serde_json::from_str::<Value>(&top)
        {
            fields.remove("nodes");
            fields.remove("edges");
            if let Ok(parsed) = serde_json::from_value::<Canvas>(Value::Object(fields)) {
                canvas = parsed;
            }
        }
        canvas.nodes = self
            .nodes
            .iter(&txn)
            .filter_map(|(id, out)| match out {
                Out::YMap(map) => read_node(&txn, id, &map),
                _ => None,
            })
            .collect();
        canvas.edges = self
            .edges
            .iter(&txn)
            .filter_map(|(id, out)| match out {
                Out::YMap(map) => read_edge(&txn, id, &map),
                _ => None,
            })
            .collect();
        let mut ink: HashMap<String, Vec<Stroke>> = HashMap::new();
        for (id, out) in self.strokes.iter(&txn) {
            if let Out::YMap(map) = out
                && let Some((node, stroke)) = read_stroke(&txn, id, &map)
            {
                ink.entry(node).or_default().push(stroke);
            }
        }
        for node in &mut canvas.nodes {
            if let Some(mut strokes) = ink.remove(&node.id) {
                strokes.sort_by(|a, b| {
                    a.kanso_z
                        .total_cmp(&b.kanso_z)
                        .then_with(|| a.id.cmp(&b.id))
                });
                node.strokes = strokes;
            }
        }
        canvas.nodes.sort_by(|a, b| {
            a.kanso_z
                .total_cmp(&b.kanso_z)
                .then_with(|| a.id.cmp(&b.id))
        });
        canvas.edges.sort_by(|a, b| {
            a.kanso_z
                .total_cmp(&b.kanso_z)
                .then_with(|| a.id.cmp(&b.id))
        });
        let mut canvas = canvas.sanitized();
        if !tombstones {
            // What repair turned into a tombstone is not shown either, nor
            // the arrows that joined it.
            canvas.nodes.retain(|n| !n.deleted);
            canvas.edges.retain(|e| !e.deleted);
            for node in &mut canvas.nodes {
                node.strokes.retain(|s| !s.deleted);
            }
            canvas = canvas.sanitized();
        }
        canvas.refit_ink();
        canvas
    }

    // --------------------------------------------------------------- writing

    /// Writes what changed between `base` (the canvas the person's view last
    /// had from [`view`](Self::view)) and `local` (what it has now), as
    /// updates of at most about `part` bytes each, in the order to send them.
    ///
    /// Only items `local` changed are written, and only their changed
    /// fields, so what others did meanwhile stays. An item missing from
    /// `local` is not a delete: a delete is the item with `deleted: true`.
    /// An item whose tombstone `local` brought back (the person's undo) is
    /// written live again, with its points.
    pub fn edit(&self, base: &Canvas, local: &Canvas, part: usize) -> Vec<Vec<u8>> {
        let mut batch = Batch::new(&self.doc, part);
        let (old_top, new_top) = (top_of(base), top_of(local));
        if !same(&old_top, &new_top) {
            let json = new_top.to_string();
            let cost = json.len() + FIELD_COST;
            self.meta.insert(batch.txn(), "top", json);
            batch.spend(cost);
        }
        let old_nodes: HashMap<&str, &Node> =
            base.nodes.iter().map(|n| (n.id.as_str(), n)).collect();
        for node in &local.nodes {
            let old = old_nodes.get(node.id.as_str()).copied();
            self.edit_node(&mut batch, old, node);
            let old_strokes: HashMap<&str, &Stroke> = old
                .map(|n| n.strokes.iter().map(|s| (s.id.as_str(), s)).collect())
                .unwrap_or_default();
            for stroke in &node.strokes {
                let old = old_strokes.get(stroke.id.as_str()).copied();
                self.edit_stroke(&mut batch, &node.id, old, stroke);
            }
        }
        let old_edges: HashMap<&str, &Edge> =
            base.edges.iter().map(|e| (e.id.as_str(), e)).collect();
        for edge in &local.edges {
            self.edit_edge(&mut batch, old_edges.get(edge.id.as_str()).copied(), edge);
        }
        batch.finish()
    }

    fn edit_node(&self, batch: &mut Batch<'_>, old: Option<&Node>, new: &Node) {
        let new_fields = fields_of(new, NODE_RESERVED);
        let old_fields = old.map(|o| fields_of(o, NODE_RESERVED));
        let (set, remove) = diff(old_fields.as_ref(), &new_fields);
        let old_text = old.and_then(|o| o.text.as_deref());
        let text_changed = match old {
            None => new.text.is_some(),
            Some(_) => old_text != new.text.as_deref(),
        };
        if old.is_some() && set.is_empty() && remove.is_empty() && !text_changed {
            return;
        }
        let cost = FIELD_COST * (set.len() + remove.len() + 1);
        let txn = batch.txn();
        let map = match self.nodes.get(&*txn, &new.id) {
            Some(Out::YMap(map)) => map,
            _ => self
                .nodes
                .insert(&mut *txn, new.id.as_str(), MapPrelim::default()),
        };
        write_fields(txn, &map, &set, &remove);
        let text_write = match (&new.text, map.get(&*txn, "text")) {
            (None, Some(_)) => {
                map.remove(txn, "text");
                None
            }
            (Some(text), Some(Out::YText(current))) if text_changed => {
                let now = current.get_string(&*txn);
                let base = old_text.unwrap_or("");
                // What was typed here since the view last had the text,
                // made again on the text as it is now.
                let target = rebase(base, text, &now);
                Some((current, now, target))
            }
            (Some(_), Some(Out::YText(_))) => None,
            (Some(text), _) => {
                let fresh = map.insert(txn, "text", TextPrelim::new(""));
                Some((fresh, String::new(), text.clone()))
            }
            (None, None) => None,
        };
        batch.spend(cost);
        // A long text (a paste) goes in parts: one update of it could be
        // past what the server takes, and would be refused for good.
        if let Some((text, now, target)) = text_write {
            rewrite_in_parts(batch, &text, &now, &target);
        }
    }

    fn edit_edge(&self, batch: &mut Batch<'_>, old: Option<&Edge>, new: &Edge) {
        let new_fields = fields_of(new, EDGE_RESERVED);
        let old_fields = old.map(|o| fields_of(o, EDGE_RESERVED));
        let (set, remove) = diff(old_fields.as_ref(), &new_fields);
        if old.is_some() && set.is_empty() && remove.is_empty() {
            return;
        }
        let txn = batch.txn();
        let map = match self.edges.get(&*txn, &new.id) {
            Some(Out::YMap(map)) => map,
            _ => self
                .edges
                .insert(&mut *txn, new.id.as_str(), MapPrelim::default()),
        };
        write_fields(txn, &map, &set, &remove);
        batch.spend(FIELD_COST * (set.len() + remove.len() + 1));
    }

    fn edit_stroke(&self, batch: &mut Batch<'_>, ink: &str, old: Option<&Stroke>, new: &Stroke) {
        if old.is_some_and(|o| o == new) {
            return;
        }
        let new_fields = fields_of(new, STROKE_RESERVED);
        let old_fields = old.map(|o| fields_of(o, STROKE_RESERVED));
        let (mut set, remove) = diff(old_fields.as_ref(), &new_fields);
        let mut cost = FIELD_COST * (set.len() + remove.len() + 3);
        let txn = batch.txn();
        let map = match self.strokes.get(&*txn, &new.id) {
            Some(Out::YMap(map)) => map,
            _ => self
                .strokes
                .insert(&mut *txn, new.id.as_str(), MapPrelim::default()),
        };
        // A stroke that was streamed is there already, with its head: the
        // fields below bring it to what the person's canvas says it ended as.
        if flag(&*txn, &map, "ink").as_deref() != Some(ink) {
            set.push(("ink".into(), Value::String(ink.to_owned())));
        }
        if !new.deleted {
            set.push(("done".into(), Value::Bool(true)));
        }
        write_fields(txn, &map, &set, &remove);
        if new.deleted || new.points.is_empty() {
            map.remove(txn, "points");
        } else {
            cost += sync_points(batch, &map, &new.points);
        }
        batch.spend(cost);
    }

    /// Streams the points of a stroke being drawn: makes the stroke exist
    /// (with `head`, not done) and holds the points drawn so far, appending
    /// only what it doesn't have yet. Only the author calls this for a
    /// stroke, so points always append in order. A stroke someone erased
    /// meanwhile answers [`Erased`]: the tombstone wins.
    pub fn stream(&self, head: &StrokeHead, points: &[Point]) -> Result<Streamed, Erased> {
        let mut txn = self.doc.transact_mut();
        let mut changed = false;
        let map = match self.strokes.get(&txn, &head.id) {
            Some(Out::YMap(map)) => {
                if flag_true(&txn, &map, "deleted") {
                    return Err(Erased);
                }
                if flag_true(&txn, &map, "done") {
                    return Ok(Streamed::default());
                }
                map
            }
            _ => {
                changed = true;
                let map = self
                    .strokes
                    .insert(&mut txn, head.id.as_str(), MapPrelim::default());
                map.insert(&mut txn, "ink", head.ink.as_str());
                map.insert(&mut txn, "owner", head.owner.as_str());
                map.insert(&mut txn, "tool", head.tool.as_str());
                map.insert(&mut txn, "color", head.color.as_str());
                map.insert(&mut txn, "width", head.width);
                map.insert(&mut txn, "kansoZ", head.z);
                map.insert(&mut txn, "kansoVersion", 1);
                map.insert(&mut txn, "done", false);
                map
            }
        };
        let array = match map.get(&txn, "points") {
            Some(Out::YArray(array)) => array,
            _ => {
                changed = true;
                map.insert(&mut txn, "points", ArrayPrelim::default())
            }
        };
        let have = array.len(&txn) as usize;
        let upto = points.len().min(MAX_STREAM_POINTS);
        if upto > have {
            changed = true;
            array.insert_range(
                &mut txn,
                have as u32,
                points[have..upto].iter().map(point_any),
            );
        }
        Ok(Streamed {
            update: changed.then(|| txn.encode_update_v1()),
            full: upto >= MAX_STREAM_POINTS,
        })
    }

    /// Takes back a stroke this device streamed but won't finish (the pen's
    /// stroke was refused when it ended, or the view went away under the
    /// pen): it becomes a tombstone, so nobody is left with half of it. A
    /// stroke that is done, deleted or not there is left alone. The update
    /// to send, or `None`.
    pub fn abandon(&self, id: &str) -> Option<Vec<u8>> {
        let mut txn = self.doc.transact_mut();
        let Some(Out::YMap(map)) = self.strokes.get(&txn, id) else {
            return None;
        };
        if flag_true(&txn, &map, "deleted") || flag_true(&txn, &map, "done") {
            return None;
        }
        let version = match map.get(&txn, "kansoVersion") {
            Some(Out::Any(Any::Number(n))) => n.as_i64().unwrap_or(0),
            _ => 0,
        };
        map.insert(&mut txn, "deleted", true);
        map.insert(&mut txn, "kansoVersion", version.max(0).saturating_add(1));
        map.remove(&mut txn, "points");
        Some(txn.encode_update_v1())
    }

    /// Takes the weight of deleted items off the document: a tombstone keeps
    /// its key, its stamps and its place (so no device can bring the item
    /// back) and drops the rest, and a deleted stroke its points. The
    /// projection is the same before and after. The update to send, or
    /// `None` when there was nothing to trim.
    pub fn trim_tombstones(&self) -> Option<Vec<u8>> {
        let mut txn = self.doc.transact_mut();
        let mut changed = false;
        for maps in [&self.nodes, &self.edges, &self.strokes] {
            let dead: Vec<MapRef> = maps
                .iter(&txn)
                .filter_map(|(_, out)| match out {
                    Out::YMap(map) if flag_true(&txn, &map, "deleted") => Some(map),
                    _ => None,
                })
                .collect();
            for map in dead {
                let extra: Vec<String> = map
                    .keys(&txn)
                    .filter(|key| !KEEP.contains(key))
                    .map(str::to_owned)
                    .collect();
                for key in extra {
                    map.remove(&mut txn, &key);
                    changed = true;
                }
            }
        }
        changed.then(|| txn.encode_update_v1())
    }

    #[cfg(test)]
    pub(crate) fn doc_for_test(&self) -> &Doc {
        &self.doc
    }

    /// How many tombstones the document holds.
    pub fn tombstones(&self) -> usize {
        let txn = self.doc.transact();
        [&self.nodes, &self.edges, &self.strokes]
            .into_iter()
            .map(|maps| {
                maps.iter(&txn)
                    .filter(|(_, out)| matches!(out, Out::YMap(m) if flag_true(&txn, m, "deleted")))
                    .count()
            })
            .sum()
    }
}

// ------------------------------------------------------------------ reading

fn scalars(txn: &impl ReadTxn, map: &MapRef) -> Map<String, Value> {
    map.iter(txn)
        .filter_map(|(key, out)| match out {
            Out::Any(any) => match serde_json::to_value(&any) {
                Ok(Value::Null) | Err(_) => None,
                Ok(value) => Some((key.to_owned(), value)),
            },
            _ => None,
        })
        .collect()
}

fn read_node(txn: &impl ReadTxn, id: &str, map: &MapRef) -> Option<Node> {
    let mut fields = scalars(txn, map);
    for key in NODE_RESERVED {
        fields.remove(*key);
    }
    fields.insert("id".into(), Value::String(id.to_owned()));
    if let Some(Out::YText(text)) = map.get(txn, "text") {
        fields.insert("text".into(), Value::String(text.get_string(txn)));
    }
    serde_json::from_value(Value::Object(fields)).ok()
}

fn read_edge(txn: &impl ReadTxn, id: &str, map: &MapRef) -> Option<Edge> {
    let mut fields = scalars(txn, map);
    for key in EDGE_RESERVED {
        fields.remove(*key);
    }
    fields.insert("id".into(), Value::String(id.to_owned()));
    serde_json::from_value(Value::Object(fields)).ok()
}

/// A stroke and the ink node it belongs to.
fn read_stroke(txn: &impl ReadTxn, id: &str, map: &MapRef) -> Option<(String, Stroke)> {
    let mut fields = scalars(txn, map);
    let ink = fields.get("ink")?.as_str()?.to_owned();
    for key in STROKE_RESERVED {
        fields.remove(*key);
    }
    fields.insert("id".into(), Value::String(id.to_owned()));
    let mut stroke: Stroke = serde_json::from_value(Value::Object(fields)).ok()?;
    if let Some(Out::YArray(array)) = map.get(txn, "points") {
        stroke.points = read_points(txn, &array);
    }
    Some((ink, stroke))
}

fn read_points(txn: &impl ReadTxn, array: &ArrayRef) -> Vec<Point> {
    array
        .iter(txn)
        .filter_map(|out| {
            let Out::Any(any) = out else { return None };
            let Ok(Value::Array(numbers)) = serde_json::to_value(&any) else {
                return None;
            };
            let numbers: Option<Vec<f64>> = numbers.iter().map(Value::as_f64).collect();
            Point::try_from(numbers?).ok()
        })
        .collect()
}

fn flag(txn: &impl ReadTxn, map: &MapRef, key: &str) -> Option<String> {
    match map.get(txn, key) {
        Some(Out::Any(Any::String(text))) => Some(text.to_string()),
        _ => None,
    }
}

fn flag_true(txn: &impl ReadTxn, map: &MapRef, key: &str) -> bool {
    matches!(map.get(txn, key), Some(Out::Any(Any::Bool(true))))
}

// ------------------------------------------------------------------ writing

/// Updates built up as items are written, cut under a size.
struct Batch<'a> {
    doc: &'a Doc,
    txn: Option<TransactionMut<'a>>,
    updates: Vec<Vec<u8>>,
    cost: usize,
    part: usize,
}

impl<'a> Batch<'a> {
    fn new(doc: &'a Doc, part: usize) -> Batch<'a> {
        Batch {
            doc,
            txn: None,
            updates: Vec::new(),
            cost: 0,
            part,
        }
    }

    fn txn(&mut self) -> &mut TransactionMut<'a> {
        let doc = self.doc;
        self.txn.get_or_insert_with(|| doc.transact_mut())
    }

    /// Counts `cost` bytes written; past a part's worth, the transaction is
    /// sent and the next write starts another.
    fn spend(&mut self, cost: usize) {
        self.cost += cost;
        if self.cost >= self.part {
            self.flush();
        }
    }

    fn flush(&mut self) {
        if let Some(txn) = self.txn.take() {
            self.updates.push(txn.encode_update_v1());
        }
        self.cost = 0;
    }

    fn finish(mut self) -> Vec<Vec<u8>> {
        self.flush();
        self.updates
    }
}

/// Sets and removes an item's fields.
fn write_fields(
    txn: &mut TransactionMut,
    map: &MapRef,
    set: &[(String, Value)],
    remove: &[String],
) {
    for (key, value) in set {
        map.insert(txn, key.as_str(), any_of(value));
    }
    for key in remove {
        map.remove(txn, key);
    }
}

/// Brings the stroke's points to `want`: appends what the array lacks when
/// it holds a start of `want`, else replaces it. Returns the bytes spent.
fn sync_points(batch: &mut Batch<'_>, map: &MapRef, want: &[Point]) -> usize {
    let txn = batch.txn();
    let (array, have) = match map.get(&*txn, "points") {
        Some(Out::YArray(array)) => {
            let held = read_points(&*txn, &array);
            if held.len() <= want.len() && held[..] == want[..held.len()] {
                let have = held.len();
                (array, have)
            } else {
                (map.insert(txn, "points", ArrayPrelim::default()), 0)
            }
        }
        _ => (map.insert(txn, "points", ArrayPrelim::default()), 0),
    };
    let mut spent = 0;
    let mut at = have;
    while at < want.len() {
        let end = (at + POINTS_PER_PUSH).min(want.len());
        let txn = batch.txn();
        array.insert_range(txn, at as u32, want[at..end].iter().map(point_any));
        spent += (end - at) * POINT_COST;
        at = end;
        if at < want.len() {
            batch.spend(spent);
            spent = 0;
        }
    }
    spent
}

/// Rewrites `text` from `old` (what it holds) to `new`.
fn rewrite(txn: &mut TransactionMut, text: &TextRef, old: &str, new: &str) {
    if old == new {
        return;
    }
    let edits = edits(old, new);
    let mut spans = Vec::with_capacity(edits.len());
    let (mut byte, mut unit) = (0, 0);
    for edit in &edits {
        unit += units(&old[byte..edit.at]);
        let delete = units(&old[edit.at..edit.at + edit.delete]);
        spans.push((unit, delete));
        unit += delete;
        byte = edit.at + edit.delete;
    }
    // From the end, so every offset still refers to the old text.
    for (edit, &(start, delete)) in edits.iter().zip(&spans).rev() {
        if edit.delete > 0 {
            text.remove_range(txn, start, delete);
        }
        if !edit.insert.is_empty() {
            text.insert(txn, start, &edit.insert);
        }
    }
}

/// [`rewrite`] through `batch`: what is inserted goes at most a part's
/// worth (and never more than [`TEXT_PART`] bytes) per update.
fn rewrite_in_parts(batch: &mut Batch<'_>, text: &TextRef, old: &str, new: &str) {
    if old == new {
        return;
    }
    let size = batch.part.clamp(1, TEXT_PART);
    let edits = edits(old, new);
    let mut spans = Vec::with_capacity(edits.len());
    let (mut byte, mut unit) = (0, 0);
    for edit in &edits {
        unit += units(&old[byte..edit.at]);
        let delete = units(&old[edit.at..edit.at + edit.delete]);
        spans.push((unit, delete));
        unit += delete;
        byte = edit.at + edit.delete;
    }
    // From the end, so every offset still refers to the old text.
    for (edit, &(start, delete)) in edits.iter().zip(&spans).rev() {
        if edit.delete > 0 {
            text.remove_range(batch.txn(), start, delete);
            batch.spend(FIELD_COST);
        }
        let (mut at, mut rest) = (start, edit.insert.as_str());
        while !rest.is_empty() {
            let mut cut = rest.len().min(size);
            while !rest.is_char_boundary(cut) {
                cut -= 1;
            }
            if cut == 0 {
                // One character wider than a part: it goes whole.
                cut = rest.chars().next().map_or(rest.len(), char::len_utf8);
            }
            let (piece, tail) = rest.split_at(cut);
            text.insert(batch.txn(), at, piece);
            at += units(piece);
            batch.spend(piece.len() + FIELD_COST);
            rest = tail;
        }
    }
}

fn point_any(point: &Point) -> Any {
    let numbers: Vec<f64> = (*point).into();
    Any::Array(Arc::from(
        numbers.into_iter().map(Any::from).collect::<Vec<_>>(),
    ))
}

fn any_of(value: &Value) -> Any {
    serde_json::from_value(value.clone()).unwrap_or(Any::Null)
}

/// An item's fields as the map holds them, without the reserved ones.
fn fields_of<T: Serialize>(item: &T, reserved: &[&str]) -> Map<String, Value> {
    match serde_json::to_value(item) {
        Ok(Value::Object(mut fields)) => {
            for key in reserved {
                fields.remove(*key);
            }
            fields.retain(|_, value| !value.is_null());
            fields
        }
        _ => Map::new(),
    }
}

/// What the document holds besides its cards and arrows.
fn top_of(canvas: &Canvas) -> Value {
    let rest = Canvas {
        nodes: Vec::new(),
        edges: Vec::new(),
        ..canvas.clone()
    };
    match serde_json::to_value(rest) {
        Ok(Value::Object(mut fields)) => {
            fields.remove("nodes");
            fields.remove("edges");
            Value::Object(fields)
        }
        _ => Value::Null,
    }
}

/// The fields to set and to remove to turn `old` into `new`; for an item
/// that isn't there yet (`old` is `None`) every field is set.
fn diff(
    old: Option<&Map<String, Value>>,
    new: &Map<String, Value>,
) -> (Vec<(String, Value)>, Vec<String>) {
    let empty = Map::new();
    let old = old.unwrap_or(&empty);
    let set = new
        .iter()
        .filter(|(key, value)| old.get(*key).is_none_or(|was| !same(was, value)))
        .map(|(key, value)| (key.clone(), value.clone()))
        .collect();
    let remove = old
        .keys()
        .filter(|key| !new.contains_key(*key))
        .cloned()
        .collect();
    (set, remove)
}

/// Whether two values are the same, numbers by value (`100` and `100.0`
/// are).
fn same(a: &Value, b: &Value) -> bool {
    match (a, b) {
        (Value::Number(x), Value::Number(y)) => x.as_f64() == y.as_f64(),
        (Value::Array(x), Value::Array(y)) => {
            x.len() == y.len() && x.iter().zip(y).all(|(a, b)| same(a, b))
        }
        (Value::Object(x), Value::Object(y)) => {
            x.len() == y.len()
                && x.iter()
                    .all(|(key, a)| y.get(key).is_some_and(|b| same(a, b)))
        }
        _ => a == b,
    }
}
