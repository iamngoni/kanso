//! The private-sync merge: two versions of one canvas, made on two devices
//! apart, become one without losing a stroke.
//!
//! Items are united by ID. The higher `kansoVersion` wins; an item only one
//! side has is kept; a tombstone beats a live item of the same version. An
//! ink node's strokes are united across both sides whichever node wins, so
//! two people adding ink to the same layer keep both. This is a per-item
//! last-writer merge, not a CRDT: it can lose one concurrent move of a node
//! when the other side adds a stroke to it, and the losing version is in
//! history. A shared canvas does not use it.

use std::cmp::Ordering;
use std::collections::{BTreeMap, HashSet};

use serde_json::{Map, Value};

use super::model::{Canvas, Edge, INK, InkOcr, Node, Stroke};

/// The union of `a` and `b`. Commutative, and merging its result with either
/// side (or itself) changes nothing. Pure.
pub fn merge(a: &Canvas, b: &Canvas) -> Canvas {
    let mut nodes = merge_by_id(
        &a.nodes,
        &b.nodes,
        |n| &n.id,
        node_order,
        |win, lose| {
            let mut win = win.clone();
            // A tombstone keeps the strokes of the layer it beat: a later
            // version of the layer from a third device brings them back, so
            // the order three devices meet in can't decide which ink lives.
            let inky = |n: &Node| n.kind == INK || n.deleted;
            if inky(&win) && inky(lose) {
                win.strokes = merge_strokes(&win.strokes, &lose.strokes);
                // Each side's layer already held its own ink, so the larger
                // of the two holds the union. Refitting here instead would
                // make the size depend on the order three devices meet in.
                win.width = win.width.max(lose.width);
                win.height = win.height.max(lose.height);
            }
            win
        },
    );
    nodes.sort_by(|p, q| {
        p.kanso_z
            .total_cmp(&q.kanso_z)
            .then_with(|| p.id.cmp(&q.id))
    });
    let live: HashSet<&str> = nodes
        .iter()
        .filter(|n| !n.deleted)
        .map(|n| n.id.as_str())
        .collect();
    let mut edges = merge_by_id(
        &a.edges,
        &b.edges,
        |e| &e.id,
        edge_order,
        |win, _| win.clone(),
    );
    // An arrow whose end card was deleted stays in the data and is hidden
    // when drawn, so the result doesn't depend on the order of the edits.
    edges.sort_by(|p, q| {
        p.kanso_z
            .total_cmp(&q.kanso_z)
            .then_with(|| p.id.cmp(&q.id))
    });

    let mut ocr: BTreeMap<&str, &InkOcr> = BTreeMap::new();
    for entry in a.kanso_ink_ocr.iter().chain(&b.kanso_ink_ocr) {
        if !live.contains(entry.node.as_str()) {
            continue;
        }
        let better = |old: &&InkOcr| {
            (&entry.text, &entry.md5, &entry.engine) > (&old.text, &old.md5, &old.engine)
        };
        match ocr.get(entry.node.as_str()) {
            Some(old) if !better(old) => {}
            _ => {
                ocr.insert(entry.node.as_str(), entry);
            }
        }
    }

    Canvas {
        nodes,
        edges,
        kanso_sketch: match (a.kanso_sketch, b.kanso_sketch) {
            (Some(p), Some(q)) => Some(if (p.v, p.w, p.h) >= (q.v, q.w, q.h) {
                p
            } else {
                q
            }),
            (p, q) => p.or(q),
        },
        kanso_sketch_id: match (&a.kanso_sketch_id, &b.kanso_sketch_id) {
            (Some(p), Some(q)) => Some(p.max(q).clone()),
            (p, q) => p.clone().or_else(|| q.clone()),
        },
        kanso_ink_ocr: ocr.into_values().cloned().collect(),
        extra: merge_extra(&a.extra, &b.extra),
    }
}

/// The merge of two note bodies that are both canvases, as a body. `None`
/// when either isn't one, so the caller falls back to last-writer-wins. Pure.
pub fn merge_bodies(a: &str, b: &str) -> Option<String> {
    let (a, b) = (super::parse(a).ok()?, super::parse(b).ok()?);
    Some(super::to_body(&merge(&a, &b)))
}

fn merge_strokes(a: &[Stroke], b: &[Stroke]) -> Vec<Stroke> {
    let mut out = merge_by_id(a, b, |s| &s.id, stroke_order, |win, _| win.clone());
    out.sort_by(|p, q| {
        p.kanso_z
            .total_cmp(&q.kanso_z)
            .then_with(|| p.id.cmp(&q.id))
    });
    out
}

/// Unites two lists by ID, keeping the item `order` ranks higher and letting
/// `combine` build the result from the winner and the loser.
fn merge_by_id<T: Clone>(
    a: &[T],
    b: &[T],
    id: impl Fn(&T) -> &String,
    order: impl Fn(&T, &T) -> Ordering,
    combine: impl Fn(&T, &T) -> T,
) -> Vec<T> {
    let mut by_id: BTreeMap<&String, (&T, Option<&T>)> = BTreeMap::new();
    for item in a.iter().chain(b) {
        match by_id.get_mut(id(item)) {
            None => {
                by_id.insert(id(item), (item, None));
            }
            Some(slot) => {
                let (win, lose) = if order(item, slot.0) == Ordering::Greater {
                    (item, slot.0)
                } else {
                    (slot.0, item)
                };
                *slot = (win, Some(lose));
            }
        }
    }
    by_id
        .into_values()
        .map(|(win, lose)| match lose {
            Some(lose) => combine(win, lose),
            None => win.clone(),
        })
        .collect()
}

fn node_order(a: &Node, b: &Node) -> Ordering {
    // Strokes are united whichever node wins, so they take no part in the
    // tie-break: a node whose strokes a merge has grown must rank as it did.
    let bare = |n: &Node| {
        json(&Node {
            strokes: Vec::new(),
            ..n.clone()
        })
    };
    (a.kanso_version, a.deleted, a.kanso_updated)
        .cmp(&(b.kanso_version, b.deleted, b.kanso_updated))
        .then_with(|| bare(a).cmp(&bare(b)))
}

fn edge_order(a: &Edge, b: &Edge) -> Ordering {
    (a.kanso_version, a.deleted, a.kanso_updated)
        .cmp(&(b.kanso_version, b.deleted, b.kanso_updated))
        .then_with(|| json(a).cmp(&json(b)))
}

/// A stroke being drawn grows, so among equal versions the longer one wins.
fn stroke_order(a: &Stroke, b: &Stroke) -> Ordering {
    (a.kanso_version, a.deleted, a.points.len(), a.kanso_updated)
        .cmp(&(b.kanso_version, b.deleted, b.points.len(), b.kanso_updated))
        .then_with(|| json(a).cmp(&json(b)))
}

fn json<T: serde::Serialize>(item: &T) -> String {
    serde_json::to_string(item).unwrap_or_default()
}

fn merge_extra(a: &Map<String, Value>, b: &Map<String, Value>) -> Map<String, Value> {
    let mut out = a.clone();
    for (key, value) in b {
        match out.get(key) {
            Some(mine) if json(mine) >= json(value) => {}
            _ => {
                out.insert(key.clone(), value.clone());
            }
        }
    }
    out
}
