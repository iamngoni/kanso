//! The edits a person makes by drawing: a finished stroke, the stroke
//! eraser, the partial eraser, delete and move for a lasso selection, and
//! undo. Every edit goes through [`Canvas`]'s stamped operations, so the
//! private-sync merge sees it like any other, and every edit returns an
//! [`Action`] that undoes it by writing new versions (an undo stamps
//! `kansoUpdated = now`; it never restores an old document).
//!
//! Coordinates in and out are the canvas's. Strokes live in ink nodes, whose
//! points are relative to the node's origin; the functions here translate.

use std::collections::HashSet;

use super::ink::Eraser;
use super::model::{Canvas, INK, Node, Point, Stroke, TOOLS};
use super::{MAX_BYTES, MAX_STROKE_POINTS, fresh_id};

/// An ID of `prefix` and eight random characters that `taken` doesn't hold.
/// Two devices drawing offline must not make the same ID for different
/// strokes, so a counter ([`fresh_id`]) is not enough for items drawn by hand.
pub fn unique_id(prefix: &str, taken: &HashSet<String>) -> String {
    loop {
        let tail: String = (0..8)
            .map(|_| char::from(b"abcdefghijklmnopqrstuvwxyz0123456789"[fastrand::usize(..36)]))
            .collect();
        let id = format!("{prefix}{tail}");
        if !taken.contains(&id) {
            return id;
        }
    }
}

/// Samples closer together than this, in points, are dropped on the way in.
pub const MIN_SAMPLE_DISTANCE: f64 = 0.4;

/// A stroke named by its ink node and its own ID.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct StrokeRef {
    pub node: String,
    pub stroke: String,
}

/// What one edit did, enough to undo and redo it.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Action {
    /// Strokes the edit took away, as they were.
    pub removed: Vec<(String, Stroke)>,
    /// Strokes the edit put in.
    pub added: Vec<(String, Stroke)>,
    /// Cards and groups the edit added, changed or deleted, one entry per
    /// node, as it was before the edit and as it is after.
    pub cards: Vec<CardChange>,
}

/// What an edit did to one card or group. `None` is "not there": a card an
/// edit added has no `before`, one it deleted has no `after`.
#[derive(Clone, Debug, PartialEq)]
pub struct CardChange {
    pub before: Option<Node>,
    pub after: Option<Node>,
}

impl CardChange {
    fn id(&self) -> &str {
        self.before
            .as_ref()
            .or(self.after.as_ref())
            .map_or("", |n| n.id.as_str())
    }
}

impl Action {
    pub fn is_empty(&self) -> bool {
        self.removed.is_empty() && self.added.is_empty() && self.cards.is_empty()
    }

    /// The one action that does what `self` did and then `next`: a drag of
    /// the eraser is many small edits and one undo step. A stroke `self`
    /// added that `next` removed again cancels out.
    pub fn then(mut self, next: Action) -> Action {
        for (node, stroke) in next.removed {
            match self.added.iter().position(|(_, s)| s.id == stroke.id) {
                Some(at) => {
                    self.added.remove(at);
                }
                None => self.removed.push((node, stroke)),
            }
        }
        self.added.extend(next.added);
        for change in next.cards {
            match self.cards.iter_mut().find(|c| c.id() == change.id()) {
                // The same card changed again: from where the first change
                // began to where the last one ended.
                Some(first) => first.after = change.after,
                None => self.cards.push(change),
            }
        }
        // A card added and deleted within one step, or changed and changed
        // back, leaves nothing to undo.
        self.cards.retain(|c| match (&c.before, &c.after) {
            (None, None) => false,
            (Some(a), Some(b)) => !same_card(a, b),
            _ => true,
        });
        self
    }
}

/// Why a stroke wasn't drawn.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Refused {
    /// The stroke has no usable points or an unknown pen.
    Empty,
    /// The canvas is at its 1 MB budget; something has to move first.
    Full,
    /// The stroke has more points than a stroke may.
    TooLong,
}

impl Canvas {
    /// The live ink node new strokes go in: the first one, or a new one at the
    /// origin. Creating it stamps it like any new node.
    pub fn ink_layer(&mut self, now: i64) -> String {
        if let Some(node) = self.nodes.iter().find(|n| n.kind == INK && !n.deleted) {
            return node.id.clone();
        }
        let taken = self
            .nodes
            .iter()
            .map(|n| n.id.clone())
            .collect::<HashSet<_>>();
        let id = fresh_id("ink", &taken);
        self.put_node(
            Node {
                id: id.clone(),
                kind: INK.into(),
                width: 1.0,
                height: 1.0,
                ..Node::default()
            },
            now,
        );
        id
    }

    /// Draws a finished stroke whose points are in canvas coordinates. It is
    /// thinned (samples nearer than [`MIN_SAMPLE_DISTANCE`] are dropped),
    /// rounded for a compact document, given a fresh ID and put on top of the
    /// ink layer. Refused when it has no points, too many, or the canvas is
    /// full.
    pub fn draw(&mut self, stroke: Stroke, now: i64) -> Result<(String, Action), Refused> {
        self.draw_with(stroke, now, false)
    }

    /// [`draw`](Self::draw) for a stroke that already has its ID: one whose
    /// points were streamed to a shared canvas while the pen was down keeps
    /// the ID they were streamed under. An ID that is empty or already taken
    /// gets a fresh one, as `draw` gives.
    pub fn draw_as(&mut self, stroke: Stroke, now: i64) -> Result<(String, Action), Refused> {
        self.draw_with(stroke, now, true)
    }

    fn draw_with(
        &mut self,
        mut stroke: Stroke,
        now: i64,
        keep_id: bool,
    ) -> Result<(String, Action), Refused> {
        if !TOOLS.contains(&stroke.tool.as_str()) {
            stroke.tool = "pen".into();
        }
        let points = thin(&stroke.points);
        if points.is_empty() {
            return Err(Refused::Empty);
        }
        if points.len() > MAX_STROKE_POINTS {
            return Err(Refused::TooLong);
        }
        if serde_json::to_vec(self).map_or(0, |v| v.len()) >= MAX_BYTES {
            return Err(Refused::Full);
        }
        let layer = self.ink_layer(now);
        let (ox, oy) = self.node(&layer).map_or((0.0, 0.0), |n| (n.x, n.y));
        let taken = self
            .stroke_ids()
            .into_iter()
            .map(str::to_owned)
            .collect::<HashSet<_>>();
        if !(keep_id && !stroke.id.is_empty() && !taken.contains(&stroke.id)) {
            stroke.id = unique_id("s", &taken);
        }
        stroke.points = points.iter().map(|p| stored(p, (ox, oy))).collect();
        stroke.width = if stroke.width.is_finite() && stroke.width > 0.0 {
            stroke.width.min(200.0)
        } else {
            2.0
        };
        stroke.deleted = false;
        stroke.kanso_z = 0.0;
        let id = stroke.id.clone();
        if !self.add_stroke(&layer, stroke, now) {
            return Err(Refused::Empty);
        }
        self.refit_ink();
        let added = self
            .stroke_in(&layer, &id)
            .cloned()
            .into_iter()
            .map(|s| (layer.clone(), s))
            .collect();
        Ok((
            id,
            Action {
                removed: Vec::new(),
                added,
                cards: Vec::new(),
            },
        ))
    }

    /// Every live stroke the eraser disc at `(x, y)` of `radius` touches.
    pub fn strokes_at(&self, x: f64, y: f64, radius: f64) -> Vec<StrokeRef> {
        self.live_strokes()
            .filter(|(node, s)| stroke_touches(node, s, &[(x, y)], radius))
            .map(|(node, s)| StrokeRef {
                node: node.id.clone(),
                stroke: s.id.clone(),
            })
            .collect()
    }

    /// The stroke eraser: tombstones every stroke the disc at `(x, y)`
    /// touches. `None` when it touched none.
    pub fn erase_strokes_at(&mut self, x: f64, y: f64, radius: f64, now: i64) -> Option<Action> {
        let hit = self.strokes_at(x, y, radius);
        self.delete_strokes(&hit, now)
    }

    /// Tombstones the named strokes in one edit.
    pub fn delete_strokes(&mut self, refs: &[StrokeRef], now: i64) -> Option<Action> {
        let mut action = Action::default();
        for r in refs {
            let Some(old) = self
                .stroke_in(&r.node, &r.stroke)
                .filter(|s| !s.deleted)
                .cloned()
            else {
                continue;
            };
            if self.remove_stroke(&r.node, &r.stroke, now) {
                action.removed.push((r.node.clone(), old));
            }
        }
        if action.is_empty() {
            return None;
        }
        self.refit_ink();
        Some(action)
    }

    /// The partial eraser: rubs out what the disc swept along `path`
    /// (canvas coordinates) touches, leaving the rest of each stroke as new
    /// strokes. One edit; `None` when nothing was touched.
    pub fn erase_along(&mut self, path: &[(f64, f64)], radius: f64, now: i64) -> Option<Action> {
        if path.is_empty() {
            return None;
        }
        let hit: Vec<StrokeRef> = self
            .live_strokes()
            .filter(|(node, s)| stroke_touches(node, s, path, radius))
            .map(|(node, s)| StrokeRef {
                node: node.id.clone(),
                stroke: s.id.clone(),
            })
            .collect();
        let mut action = Action::default();
        for r in hit {
            let Some(node) = self.node(&r.node) else {
                continue;
            };
            let eraser = Eraser {
                path: path
                    .iter()
                    .map(|&(x, y)| (x - node.x, y - node.y))
                    .collect(),
                radius,
            };
            let Some(old) = self.stroke_in(&r.node, &r.stroke).cloned() else {
                continue;
            };
            if let Some(ids) = self.erase_partial(&r.node, &r.stroke, &eraser, now) {
                action.removed.push((r.node.clone(), old));
                for id in ids {
                    if let Some(s) = self.stroke_in(&r.node, &id) {
                        action.added.push((r.node.clone(), s.clone()));
                    }
                }
            }
        }
        if action.is_empty() {
            return None;
        }
        self.refit_ink();
        Some(action)
    }

    /// The live strokes with a point inside `polygon` (canvas coordinates):
    /// what a lasso selects.
    pub fn strokes_in(&self, polygon: &[(f64, f64)]) -> Vec<StrokeRef> {
        if polygon.len() < 3 {
            return Vec::new();
        }
        self.live_strokes()
            .filter(|(node, s)| {
                s.points
                    .iter()
                    .any(|p| in_polygon((p.x + node.x, p.y + node.y), polygon))
            })
            .map(|(node, s)| StrokeRef {
                node: node.id.clone(),
                stroke: s.id.clone(),
            })
            .collect()
    }

    /// Moves the named strokes by `(dx, dy)`: each becomes a tombstone and a
    /// translated copy with a fresh ID, so the points of a stroke are never
    /// rewritten in place.
    pub fn move_strokes(
        &mut self,
        refs: &[StrokeRef],
        dx: f64,
        dy: f64,
        now: i64,
    ) -> Option<Action> {
        if !dx.is_finite() || !dy.is_finite() || (dx == 0.0 && dy == 0.0) {
            return None;
        }
        let mut taken = self
            .stroke_ids()
            .into_iter()
            .map(str::to_owned)
            .collect::<HashSet<_>>();
        let mut action = Action::default();
        for r in refs {
            let Some(old) = self
                .stroke_in(&r.node, &r.stroke)
                .filter(|s| !s.deleted)
                .cloned()
            else {
                continue;
            };
            let mut moved = old.clone();
            let new_id = unique_id("s", &taken);
            taken.insert(new_id.clone());
            moved.id = new_id.clone();
            for p in &mut moved.points {
                p.x = round(p.x + dx, 100.0);
                p.y = round(p.y + dy, 100.0);
            }
            if self.remove_stroke(&r.node, &r.stroke, now) {
                action.removed.push((r.node.clone(), old));
                if self.add_stroke(&r.node, moved, now)
                    && let Some(s) = self.stroke_in(&r.node, &new_id)
                {
                    action.added.push((r.node.clone(), s.clone()));
                }
            }
        }
        if action.is_empty() {
            return None;
        }
        self.refit_ink();
        Some(action)
    }

    /// Adds `card`, or replaces the card with its ID, as one undoable edit.
    pub fn put_card(&mut self, mut card: Node, now: i64) -> Action {
        let before = self.node(&card.id).cloned();
        // No stacking order given: it keeps its place, as `put_node` does.
        if let (Some(old), true) = (&before, card.kanso_z == 0.0) {
            card.kanso_z = old.kanso_z;
        }
        if before.as_ref().is_some_and(|b| same_card(b, &card)) {
            return Action::default();
        }
        let id = card.id.clone();
        self.put_node(card, now);
        let after = self.node(&id).cloned();
        card_action(before, after)
    }

    /// Moves a card or group as one undoable edit. `None` when it isn't
    /// there or is already at `(x, y)`.
    pub fn move_card(&mut self, id: &str, x: f64, y: f64, now: i64) -> Option<Action> {
        let before = self.node(id).cloned()?;
        if before.kind == INK || (before.x == x && before.y == y) {
            return None;
        }
        self.move_node(id, x, y, now);
        Some(card_action(Some(before), self.node(id).cloned()))
    }

    /// Deletes a card or group as one undoable edit; its arrows stay,
    /// hidden. `None` when it isn't there. Ink layers aren't cards.
    pub fn delete_card(&mut self, id: &str, now: i64) -> Option<Action> {
        let before = self.node(id).cloned()?;
        if before.kind == INK {
            return None;
        }
        self.remove_node(id, now);
        Some(card_action(Some(before), None))
    }

    /// Undoes `action` (or redoes it, with `forward`), by writing new
    /// versions. Returns whether anything changed.
    pub fn apply(&mut self, action: &Action, forward: bool, now: i64) -> bool {
        let (kill, live) = if forward {
            (&action.removed, &action.added)
        } else {
            (&action.added, &action.removed)
        };
        let mut changed = false;
        for change in &action.cards {
            let target = if forward {
                &change.after
            } else {
                &change.before
            };
            match target {
                Some(node) => {
                    self.put_node(node.clone(), now);
                    changed = true;
                }
                None => changed |= self.remove_node(change.id(), now),
            }
        }
        for (node, stroke) in kill {
            changed |= self.remove_stroke(node, &stroke.id, now);
        }
        for (node, stroke) in live {
            changed |= self.restore_stroke(node, stroke, now);
        }
        if changed {
            self.refit_ink();
        }
        changed
    }

    /// Puts `stroke` back in ink node `node` over its tombstone, one version
    /// higher, or adds it when the node never held it. `false` when it is
    /// already live or the node is gone.
    fn restore_stroke(&mut self, node: &str, stroke: &Stroke, now: i64) -> bool {
        let Some(ink) = self
            .nodes
            .iter_mut()
            .find(|n| n.id == node && n.kind == INK && !n.deleted)
        else {
            return false;
        };
        match ink.strokes.iter_mut().find(|s| s.id == stroke.id) {
            Some(slot) if !slot.deleted => false,
            Some(slot) => {
                let version = slot.kanso_version + 1;
                *slot = Stroke {
                    deleted: false,
                    kanso_version: version,
                    kanso_updated: now,
                    ..stroke.clone()
                };
                ink.kanso_version += 1;
                ink.kanso_updated = now;
                true
            }
            None => {
                ink.strokes.push(Stroke {
                    deleted: false,
                    kanso_version: stroke.kanso_version.max(1),
                    kanso_updated: now,
                    ..stroke.clone()
                });
                ink.kanso_version += 1;
                ink.kanso_updated = now;
                true
            }
        }
    }

    fn stroke_in(&self, node: &str, stroke: &str) -> Option<&Stroke> {
        self.nodes
            .iter()
            .find(|n| n.id == node && n.kind == INK)?
            .strokes
            .iter()
            .find(|s| s.id == stroke)
    }

    /// Every live stroke of every live ink node, with its node.
    pub fn live_strokes(&self) -> impl Iterator<Item = (&Node, &Stroke)> {
        self.nodes
            .iter()
            .filter(|n| n.kind == INK && !n.deleted)
            .flat_map(|n| n.strokes.iter().filter(|s| !s.deleted).map(move |s| (n, s)))
    }
}

fn card_action(before: Option<Node>, after: Option<Node>) -> Action {
    Action {
        cards: vec![CardChange { before, after }],
        ..Action::default()
    }
}

/// Whether two states of a card hold the same content (stamps aside).
fn same_card(a: &Node, b: &Node) -> bool {
    let stamp = |n: &Node| Node {
        kanso_version: 0,
        kanso_updated: 0,
        ..n.clone()
    };
    stamp(a) == stamp(b)
}

/// Whether the disc of `radius` swept along `path` touches `stroke`: its
/// bounds first, then the distance to its centreline against half its width
/// plus the radius.
fn stroke_touches(node: &Node, stroke: &Stroke, path: &[(f64, f64)], radius: f64) -> bool {
    let Some(b) = stroke.bounds() else {
        return false;
    };
    let reach = radius.max(0.0);
    let (lo_x, lo_y) = (b.x + node.x - reach, b.y + node.y - reach);
    let (hi_x, hi_y) = (b.right() + node.x + reach, b.bottom() + node.y + reach);
    let near = |p: (f64, f64)| p.0 >= lo_x && p.0 <= hi_x && p.1 >= lo_y && p.1 <= hi_y;
    let swept: Vec<(f64, f64)> = path.to_vec();
    if !swept.iter().any(|&p| near(p)) && !segments_cross_box(&swept, (lo_x, lo_y, hi_x, hi_y)) {
        return false;
    }
    let limit = stroke.width / 2.0 + reach;
    let limit2 = limit * limit;
    let pts: Vec<(f64, f64)> = stroke
        .points
        .iter()
        .map(|p| (p.x + node.x, p.y + node.y))
        .collect();
    let probes: Vec<((f64, f64), (f64, f64))> = match swept.as_slice() {
        [only] => vec![(*only, *only)],
        many => many.windows(2).map(|w| (w[0], w[1])).collect(),
    };
    for (a, b) in probes {
        if let [only] = pts.as_slice() {
            if dist2_to_segment(*only, a, b) <= limit2 {
                return true;
            }
            continue;
        }
        for w in pts.windows(2) {
            if segments_distance2(a, b, w[0], w[1]) <= limit2 {
                return true;
            }
        }
    }
    false
}

fn segments_cross_box(path: &[(f64, f64)], (lx, ly, hx, hy): (f64, f64, f64, f64)) -> bool {
    path.windows(2).any(|w| {
        let (a, b) = (w[0], w[1]);
        !(a.0.max(b.0) < lx || a.0.min(b.0) > hx || a.1.max(b.1) < ly || a.1.min(b.1) > hy)
    })
}

/// The squared distance between two segments.
fn segments_distance2(a: (f64, f64), b: (f64, f64), c: (f64, f64), d: (f64, f64)) -> f64 {
    if segments_intersect(a, b, c, d) {
        return 0.0;
    }
    dist2_to_segment(a, c, d)
        .min(dist2_to_segment(b, c, d))
        .min(dist2_to_segment(c, a, b))
        .min(dist2_to_segment(d, a, b))
}

fn segments_intersect(a: (f64, f64), b: (f64, f64), c: (f64, f64), d: (f64, f64)) -> bool {
    let cross = |o: (f64, f64), p: (f64, f64), q: (f64, f64)| {
        (p.0 - o.0) * (q.1 - o.1) - (p.1 - o.1) * (q.0 - o.0)
    };
    let (d1, d2, d3, d4) = (
        cross(c, d, a),
        cross(c, d, b),
        cross(a, b, c),
        cross(a, b, d),
    );
    (d1 > 0.0) != (d2 > 0.0)
        && (d3 > 0.0) != (d4 > 0.0)
        && d1 != 0.0
        && d2 != 0.0
        && d3 != 0.0
        && d4 != 0.0
}

fn dist2_to_segment(p: (f64, f64), a: (f64, f64), b: (f64, f64)) -> f64 {
    let (dx, dy) = (b.0 - a.0, b.1 - a.1);
    let len2 = dx * dx + dy * dy;
    let t = if len2 == 0.0 {
        0.0
    } else {
        (((p.0 - a.0) * dx + (p.1 - a.1) * dy) / len2).clamp(0.0, 1.0)
    };
    let (cx, cy) = (a.0 + t * dx, a.1 + t * dy);
    (p.0 - cx).powi(2) + (p.1 - cy).powi(2)
}

/// Even-odd point in polygon.
pub fn in_polygon(p: (f64, f64), polygon: &[(f64, f64)]) -> bool {
    let mut inside = false;
    let mut j = polygon.len().wrapping_sub(1);
    for i in 0..polygon.len() {
        let (a, b) = (polygon[i], polygon[j]);
        if (a.1 > p.1) != (b.1 > p.1) && p.0 < (b.0 - a.0) * (p.1 - a.1) / (b.1 - a.1) + a.0 {
            inside = !inside;
        }
        j = i;
    }
    inside
}

/// Drops samples nearer than [`MIN_SAMPLE_DISTANCE`] to the last one kept
/// (always keeping the first and the last) and any that aren't finite. Pure.
pub fn thin(points: &[Point]) -> Vec<Point> {
    let mut out = thin_so_far(points);
    let end = points
        .iter()
        .rev()
        .find(|p| p.x.is_finite() && p.y.is_finite());
    if let (Some(end), Some(kept)) = (end, out.last().copied())
        && out.len() > 1
        && (end.x, end.y) != (kept.x, kept.y)
    {
        out.push(*end);
    }
    out
}

/// [`thin`] without the last sample it would add: what a stroke still being
/// drawn has kept so far, which is always the start of what [`thin`] gives
/// when the stroke ends. Pure.
pub fn thin_so_far(points: &[Point]) -> Vec<Point> {
    let mut out: Vec<Point> = Vec::with_capacity(points.len());
    for p in points.iter().filter(|p| p.x.is_finite() && p.y.is_finite()) {
        match out.last() {
            Some(last) if (p.x - last.x).hypot(p.y - last.y) < MIN_SAMPLE_DISTANCE => {}
            _ => out.push(*p),
        }
    }
    out
}

/// A drawn sample as the document stores it: relative to its ink node's
/// origin, rounded for a compact document.
fn stored(p: &Point, (ox, oy): (f64, f64)) -> Point {
    Point {
        x: round(p.x - ox, 100.0),
        y: round(p.y - oy, 100.0),
        pressure: round(p.pressure.clamp(0.0, 1.0), 1000.0),
        tilt: p.tilt.map(|(a, b)| (round(a, 1000.0), round(b, 1000.0))),
    }
}

/// The points of a stroke still being drawn, as they will be stored once it
/// ends (canvas coordinates in, the ink node's at `origin`): a shared
/// canvas streams these, and the finished stroke starts with them. Pure.
pub fn streamed_points(points: &[Point], origin: (f64, f64)) -> Vec<Point> {
    thin_so_far(points)
        .iter()
        .take(MAX_STROKE_POINTS)
        .map(|p| stored(p, origin))
        .collect()
}

fn round(v: f64, scale: f64) -> f64 {
    (v * scale).round() / scale
}
