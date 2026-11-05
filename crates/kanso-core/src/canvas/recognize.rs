//! Handwriting search, the document side: which ink nodes still need to be
//! read, and keeping what was read in `kansoInkOcr`.
//!
//! Recognition itself belongs to the platform (Vision on a Mac). Here is the
//! rule that makes it incremental: each entry carries a fingerprint of the
//! strokes it was read from, so only a node whose strokes changed since is
//! offered again, and words read from strokes that changed meanwhile are
//! refused.

use std::collections::HashSet;

use md5::{Digest, Md5};

use super::model::{Canvas, INK, InkOcr, Node, Stroke};

/// Names the reader in an entry when a caller has none.
pub const DEFAULT_ENGINE: &str = "vision";

/// An ink node waiting to be read: its ID and the fingerprint of the strokes
/// to read, to be given back with the words.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct InkToRead {
    pub node: String,
    pub md5: String,
}

impl Node {
    /// The strokes recognition reads: live pen, pencil and marker strokes.
    /// A highlighter is a wash of colour, not writing.
    pub fn handwriting(&self) -> impl Iterator<Item = &Stroke> {
        self.strokes
            .iter()
            .filter(|s| !s.deleted && s.tool != "highlighter" && !s.points.is_empty())
    }

    /// A fingerprint of what recognition would read from this ink node: the
    /// geometry of its handwriting strokes. Moving the node, recolouring a
    /// stroke or re-stamping its version does not change it; drawing,
    /// erasing or splitting a stroke does. `None` for a node that is not ink
    /// or has nothing to read. Pure.
    pub fn ink_fingerprint(&self) -> Option<String> {
        if self.kind != INK {
            return None;
        }
        let mut strokes: Vec<&Stroke> = self.handwriting().collect();
        if strokes.is_empty() {
            return None;
        }
        strokes.sort_by(|a, b| a.id.cmp(&b.id));
        let mut hasher = Md5::new();
        for stroke in strokes {
            hasher.update((stroke.id.len() as u64).to_le_bytes());
            hasher.update(stroke.id.as_bytes());
            hasher.update(stroke.width.to_bits().to_le_bytes());
            hasher.update((stroke.points.len() as u64).to_le_bytes());
            for p in &stroke.points {
                hasher.update(p.x.to_bits().to_le_bytes());
                hasher.update(p.y.to_bits().to_le_bytes());
            }
        }
        Some(format!("{:x}", hasher.finalize()))
    }
}

impl Canvas {
    /// The live ink nodes whose handwriting has not been read from its
    /// current strokes, in stacking order. Pure.
    pub fn ink_to_read(&self) -> Vec<InkToRead> {
        self.live_nodes()
            .filter_map(|node| {
                let md5 = node.ink_fingerprint()?;
                let current = self
                    .kanso_ink_ocr
                    .iter()
                    .any(|o| o.node == node.id && o.md5 == md5);
                (!current).then(|| InkToRead {
                    node: node.id.clone(),
                    md5,
                })
            })
            .collect()
    }

    /// Keeps `text` as what ink node `node` says, read from the strokes with
    /// fingerprint `md5` by `engine`. Nothing is kept, and `false` returned,
    /// when the node is gone or its strokes changed since: it is offered
    /// again. Empty `text` is kept too, so ink with no words is not read
    /// again until it changes. Entries of nodes that are gone or have
    /// nothing to read are dropped along the way; no other node's entry is
    /// touched.
    pub fn keep_ink_text(&mut self, node: &str, md5: &str, engine: &str, text: &str) -> bool {
        let current = self
            .live_nodes()
            .find(|n| n.id == node)
            .and_then(Node::ink_fingerprint);
        if current.as_deref() != Some(md5) {
            return false;
        }
        let entry = InkOcr {
            node: node.to_owned(),
            md5: md5.to_owned(),
            engine: engine.to_owned(),
            text: text.trim().to_owned(),
        };
        match self.kanso_ink_ocr.iter_mut().find(|o| o.node == node) {
            Some(old) => *old = entry,
            None => self.kanso_ink_ocr.push(entry),
        }
        self.prune_ink_text();
        true
    }

    /// Drops recognized text of ink nodes that are gone or have no
    /// handwriting left. Whether anything was dropped. Pure.
    pub fn prune_ink_text(&mut self) -> bool {
        let live: HashSet<String> = self
            .live_nodes()
            .filter(|n| n.ink_fingerprint().is_some())
            .map(|n| n.id.clone())
            .collect();
        let before = self.kanso_ink_ocr.len();
        self.kanso_ink_ocr.retain(|o| live.contains(&o.node));
        self.kanso_ink_ocr.len() != before
    }
}
