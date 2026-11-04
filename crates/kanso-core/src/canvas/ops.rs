//! Edits to a canvas that stamp what the merge needs: every change bumps the
//! item's `kansoVersion` and sets `kansoUpdated`, exactly as an edit in the
//! canvas view or from the local API must. Deleting writes a tombstone.

use std::collections::HashSet;

use super::ink::{Eraser, Split, split_stroke};
use super::model::{Canvas, Edge, INK, Node, Stroke};

impl Canvas {
    /// Adds `node` on top, or replaces the live or deleted node with its ID.
    /// Pure besides `self`.
    pub fn put_node(&mut self, mut node: Node, now: i64) {
        let top = self.top_z();
        match self.nodes.iter_mut().find(|n| n.id == node.id) {
            Some(old) => {
                node.kanso_version = old.kanso_version + 1;
                node.kanso_z = if node.kanso_z == 0.0 {
                    old.kanso_z
                } else {
                    node.kanso_z
                };
                node.deleted = false;
                node.kanso_updated = now;
                *old = node;
            }
            None => {
                node.kanso_version = node.kanso_version.max(1);
                node.kanso_updated = now;
                if node.kanso_z == 0.0 {
                    node.kanso_z = top + 1.0;
                }
                self.nodes.push(node);
            }
        }
    }

    /// Adds `edge` on top, or replaces the one with its ID.
    pub fn put_edge(&mut self, mut edge: Edge, now: i64) {
        let top = self.top_z();
        match self.edges.iter_mut().find(|e| e.id == edge.id) {
            Some(old) => {
                edge.kanso_version = old.kanso_version + 1;
                edge.deleted = false;
                edge.kanso_updated = now;
                *old = edge;
            }
            None => {
                edge.kanso_version = edge.kanso_version.max(1);
                edge.kanso_updated = now;
                if edge.kanso_z == 0.0 {
                    edge.kanso_z = top + 1.0;
                }
                self.edges.push(edge);
            }
        }
    }

    /// Deletes node `id`: it becomes a tombstone. The arrows that end on it
    /// stay as they are, hidden while an end is deleted, so putting the card
    /// back (an undo, or a later version of it from another device) shows
    /// them again. `false` when there was no live node.
    pub fn remove_node(&mut self, id: &str, now: i64) -> bool {
        let Some(node) = self.nodes.iter_mut().find(|n| n.id == id && !n.deleted) else {
            return false;
        };
        *node = Node {
            id: node.id.clone(),
            deleted: true,
            kanso_version: node.kanso_version + 1,
            kanso_updated: now,
            kanso_z: node.kanso_z,
            ..Node::default()
        };
        true
    }

    /// Deletes arrow `id`. `false` when there was no live arrow.
    pub fn remove_edge(&mut self, id: &str, now: i64) -> bool {
        match self.edges.iter_mut().find(|e| e.id == id && !e.deleted) {
            Some(edge) => {
                *edge = tombstone_edge(edge, now);
                true
            }
            None => false,
        }
    }

    /// Adds a finished stroke to ink node `node`, on top. The caller owns the
    /// ID. `false` when `node` isn't a live ink node or the ID is taken.
    pub fn add_stroke(&mut self, node: &str, mut stroke: Stroke, now: i64) -> bool {
        if self.stroke_ids().contains(stroke.id.as_str()) {
            return false;
        }
        let top = self.top_z();
        let Some(ink) = self.live_ink_mut(node) else {
            return false;
        };
        stroke.deleted = false;
        stroke.kanso_version = 1;
        stroke.kanso_updated = now;
        if stroke.kanso_z == 0.0 {
            stroke.kanso_z = top + 1.0;
        }
        ink.strokes.push(stroke);
        touch_node(ink, now);
        true
    }

    /// Erases a whole stroke: a tombstone that keeps no points.
    pub fn remove_stroke(&mut self, node: &str, stroke: &str, now: i64) -> bool {
        let Some(ink) = self.live_ink_mut(node) else {
            return false;
        };
        let Some(old) = ink
            .strokes
            .iter_mut()
            .find(|s| s.id == stroke && !s.deleted)
        else {
            return false;
        };
        *old = tombstone_stroke(old, now);
        touch_node(ink, now);
        true
    }

    /// The partial eraser: tombstones stroke `stroke` and adds what is left
    /// of it as new strokes with fresh IDs, in one edit. `eraser` is in the
    /// node's coordinates. Returns the new strokes' IDs, or `None` when the
    /// eraser missed or the stroke isn't there.
    pub fn erase_partial(
        &mut self,
        node: &str,
        stroke: &str,
        eraser: &Eraser,
        now: i64,
    ) -> Option<Vec<String>> {
        let mut taken = self
            .stroke_ids()
            .into_iter()
            .map(str::to_owned)
            .collect::<HashSet<_>>();
        let top = self.top_z();
        let ink = self.live_ink_mut(node)?;
        let old = ink.strokes.iter().find(|s| s.id == stroke && !s.deleted)?;
        let mut n = 0;
        let split = split_stroke(old, eraser, || {
            loop {
                n += 1;
                let id = format!("{stroke}-{n}");
                if taken.insert(id.clone()) {
                    break id;
                }
            }
        });
        let Split::Pieces(pieces) = split else {
            return None;
        };
        let at = ink.strokes.iter().position(|s| s.id == stroke)?;
        ink.strokes[at] = tombstone_stroke(&ink.strokes[at], now);
        let mut ids = Vec::new();
        for (i, mut piece) in pieces.into_iter().enumerate() {
            piece.kanso_version = 1;
            piece.kanso_updated = now;
            piece.kanso_z = top + 1.0 + i as f64;
            ids.push(piece.id.clone());
            ink.strokes.push(piece);
        }
        touch_node(ink, now);
        Some(ids)
    }

    /// Moves a live node. The move is one scalar change: an ink node's
    /// points stay as they are.
    pub fn move_node(&mut self, id: &str, x: f64, y: f64, now: i64) -> bool {
        match self.nodes.iter_mut().find(|n| n.id == id && !n.deleted) {
            Some(node) => {
                node.x = x;
                node.y = y;
                touch_node(node, now);
                true
            }
            None => false,
        }
    }

    /// Every stroke ID in the document, tombstones included.
    pub fn stroke_ids(&self) -> HashSet<&str> {
        self.nodes
            .iter()
            .flat_map(|n| n.strokes.iter().map(|s| s.id.as_str()))
            .collect()
    }

    fn live_ink_mut(&mut self, id: &str) -> Option<&mut Node> {
        self.nodes
            .iter_mut()
            .find(|n| n.id == id && n.kind == INK && !n.deleted)
    }

    /// The highest `kansoZ` of any item.
    fn top_z(&self) -> f64 {
        let nodes = self
            .nodes
            .iter()
            .flat_map(|n| std::iter::once(n.kanso_z).chain(n.strokes.iter().map(|s| s.kanso_z)));
        nodes
            .chain(self.edges.iter().map(|e| e.kanso_z))
            .fold(0.0, f64::max)
    }
}

fn touch_node(node: &mut Node, now: i64) {
    node.kanso_version += 1;
    node.kanso_updated = now;
}

fn tombstone_edge(edge: &Edge, now: i64) -> Edge {
    Edge {
        id: edge.id.clone(),
        deleted: true,
        kanso_version: edge.kanso_version + 1,
        kanso_updated: now,
        kanso_z: edge.kanso_z,
        ..Edge::default()
    }
}

fn tombstone_stroke(stroke: &Stroke, now: i64) -> Stroke {
    Stroke {
        id: stroke.id.clone(),
        deleted: true,
        kanso_version: stroke.kanso_version + 1,
        kanso_updated: now,
        kanso_z: stroke.kanso_z,
        ..Stroke::default()
    }
}
