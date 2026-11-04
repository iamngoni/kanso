//! Where things are: the rectangle of a node, a stroke or the whole canvas,
//! used for embeds, exports, fitting the view and culling.

use super::model::{Canvas, INK, Node, Stroke};

/// An axis-aligned rectangle.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Bounds {
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
}

impl Bounds {
    pub fn right(&self) -> f64 {
        self.x + self.width
    }

    pub fn bottom(&self) -> f64 {
        self.y + self.height
    }

    /// The smallest rectangle holding both.
    pub fn union(self, other: Bounds) -> Bounds {
        let x = self.x.min(other.x);
        let y = self.y.min(other.y);
        Bounds {
            x,
            y,
            width: self.right().max(other.right()) - x,
            height: self.bottom().max(other.bottom()) - y,
        }
    }

    /// Whether `other` lies wholly inside.
    pub fn contains(&self, other: &Bounds) -> bool {
        other.x >= self.x
            && other.y >= self.y
            && other.right() <= self.right()
            && other.bottom() <= self.bottom()
    }
}

impl Stroke {
    /// The rectangle the stroke covers, in its node's coordinates, with half
    /// its width around the centreline. `None` for a stroke with no points.
    pub fn bounds(&self) -> Option<Bounds> {
        let first = self.points.first()?;
        let (mut lo_x, mut lo_y, mut hi_x, mut hi_y) = (first.x, first.y, first.x, first.y);
        for p in &self.points {
            lo_x = lo_x.min(p.x);
            lo_y = lo_y.min(p.y);
            hi_x = hi_x.max(p.x);
            hi_y = hi_y.max(p.y);
        }
        let pad = (self.width / 2.0).max(0.0);
        Some(Bounds {
            x: lo_x - pad,
            y: lo_y - pad,
            width: hi_x - lo_x + 2.0 * pad,
            height: hi_y - lo_y + 2.0 * pad,
        })
    }
}

impl Node {
    /// The node's rectangle. An ink node's is recomputed from its live
    /// strokes (so it follows a stroke that grew past the stored size); an
    /// ink node with no live strokes keeps its stored one.
    pub fn bounds(&self) -> Bounds {
        let stored = Bounds {
            x: self.x,
            y: self.y,
            width: self.width,
            height: self.height,
        };
        if self.kind != INK {
            return stored;
        }
        self.strokes
            .iter()
            .filter(|s| !s.deleted)
            .filter_map(Stroke::bounds)
            .reduce(Bounds::union)
            .map(|b| Bounds {
                x: self.x + b.x,
                y: self.y + b.y,
                ..b
            })
            .unwrap_or(stored)
    }
}

impl Canvas {
    /// The union of every live node's rectangle: what an embed shows when it
    /// names no group. `None` for an empty canvas.
    pub fn content_bounds(&self) -> Option<Bounds> {
        self.nodes
            .iter()
            .filter(|n| !n.deleted)
            .map(Node::bounds)
            .reduce(Bounds::union)
    }

    /// The rectangle of group `id`. `None` when there is no such live group.
    pub fn group_bounds(&self, id: &str) -> Option<Bounds> {
        self.nodes
            .iter()
            .find(|n| n.id == id && n.kind == "group" && !n.deleted)
            .map(Node::bounds)
    }

    /// The live nodes that lie wholly inside group `id` (not the group).
    pub fn nodes_in_group(&self, id: &str) -> Vec<&Node> {
        let Some(area) = self.group_bounds(id) else {
            return Vec::new();
        };
        self.nodes
            .iter()
            .filter(|n| !n.deleted && n.id != id && area.contains(&n.bounds()))
            .collect()
    }

    /// Sets each live ink node's stored size to what its live strokes cover,
    /// measured from the node's own origin, so the stored rectangle never
    /// disagrees with the ink. A node with no live strokes keeps its size.
    pub fn refit_ink(&mut self) {
        for node in self
            .nodes
            .iter_mut()
            .filter(|n| n.kind == INK && !n.deleted)
        {
            let live = node
                .strokes
                .iter()
                .filter(|s| !s.deleted)
                .filter_map(Stroke::bounds)
                .reduce(Bounds::union);
            if let Some(b) = live {
                node.width = b.right().max(1.0).ceil();
                node.height = b.bottom().max(1.0).ceil();
            }
        }
    }
}
