//! Repairing a canvas that something other than the editor wrote. A shared
//! canvas is projected from a CRDT every member can write to, so the
//! projection cannot trust what it reads: whatever a member's device (or a
//! forged update) put there, the canvas that reaches the library must be one
//! the view can open.

use std::collections::HashSet;

use super::model::{Canvas, Edge, INK, Node, Point, Stroke, TOOLS};
use super::{MAX_ITEMS, MAX_STROKE_POINTS};

/// The widest pen [`Canvas::validate`] allows, in points.
const MAX_WIDTH: f64 = 500.0;

impl Canvas {
    /// The same canvas with whatever [`validate`](Self::validate) would
    /// refuse repaired or taken out, and nothing else changed:
    ///
    /// - a live card that has no type, no real place or no size becomes a
    ///   tombstone (an ink layer with a bad size is given one instead);
    /// - strokes with an unknown pen or no usable point are dropped, odd
    ///   numbers are made real (pressure into 0 to 1, width into range) and
    ///   a stroke is cut at the most points one may hold;
    /// - an ID that repeats keeps its first holder, a missing ID drops the
    ///   item;
    /// - an arrow that joins a card that isn't there is dropped, and a side
    ///   or end that isn't one of the four or the two is forgotten.
    ///
    /// Idempotent, and a canvas that is valid already comes back as it was.
    /// Pure.
    pub fn sanitized(mut self) -> Canvas {
        let mut ids = HashSet::new();
        let mut stroke_ids = HashSet::new();
        let mut nodes = Vec::with_capacity(self.nodes.len().min(MAX_ITEMS));
        for mut node in self.nodes {
            if nodes.len() >= MAX_ITEMS {
                break;
            }
            if node.id.is_empty() || !ids.insert(node.id.clone()) {
                continue;
            }
            node.strokes = clean_strokes(std::mem::take(&mut node.strokes), &mut stroke_ids);
            if node.deleted {
                // A tombstone is only its ID, its stamps and its place.
                let strokes = std::mem::take(&mut node.strokes);
                bury(&mut node);
                node.strokes = strokes;
            } else {
                repair_node(&mut node);
                node.kanso_z = finite(node.kanso_z);
            }
            nodes.push(node);
        }
        let mut edge_ids = HashSet::new();
        let mut edges = Vec::with_capacity(self.edges.len().min(MAX_ITEMS));
        for mut edge in self.edges {
            if edges.len() >= MAX_ITEMS {
                break;
            }
            if edge.id.is_empty() || !edge_ids.insert(edge.id.clone()) {
                continue;
            }
            if edge.deleted {
                edge = Edge {
                    id: edge.id,
                    deleted: true,
                    kanso_version: edge.kanso_version,
                    kanso_updated: edge.kanso_updated,
                    kanso_z: finite(edge.kanso_z),
                    ..Edge::default()
                };
            } else {
                if !ids.contains(&edge.from_node) || !ids.contains(&edge.to_node) {
                    continue;
                }
                repair_edge(&mut edge);
                edge.kanso_z = finite(edge.kanso_z);
            }
            edges.push(edge);
        }
        self.nodes = nodes;
        self.edges = edges;
        self
    }
}

fn repair_node(node: &mut Node) {
    let placed = node.x.is_finite() && node.y.is_finite();
    let sized =
        node.width.is_finite() && node.height.is_finite() && node.width > 0.0 && node.height > 0.0;
    if node.kind.is_empty() || !placed {
        bury(node);
        return;
    }
    if !sized {
        if node.kind == INK {
            node.width = 1.0;
            node.height = 1.0;
        } else {
            bury(node);
            return;
        }
    }
    if node.kind != INK {
        node.strokes.clear();
    }
}

/// Turns a card into a tombstone, keeping its place in the stacking order.
fn bury(node: &mut Node) {
    *node = Node {
        id: std::mem::take(&mut node.id),
        deleted: true,
        kanso_version: node.kanso_version,
        kanso_updated: node.kanso_updated,
        kanso_z: finite(node.kanso_z),
        ..Node::default()
    };
}

fn finite(z: f64) -> f64 {
    if z.is_finite() { z } else { 0.0 }
}

fn repair_edge(edge: &mut Edge) {
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
            .is_some_and(|s| !matches!(s, "none" | "arrow"))
        {
            *end = None;
        }
    }
}

fn clean_strokes(strokes: Vec<Stroke>, seen: &mut HashSet<String>) -> Vec<Stroke> {
    let mut out = Vec::with_capacity(strokes.len());
    for mut stroke in strokes {
        if stroke.id.is_empty() || !seen.insert(stroke.id.clone()) {
            continue;
        }
        if stroke.deleted {
            stroke = Stroke {
                id: stroke.id,
                deleted: true,
                kanso_version: stroke.kanso_version,
                kanso_updated: stroke.kanso_updated,
                kanso_z: finite(stroke.kanso_z),
                ..Stroke::default()
            };
        } else {
            if !TOOLS.contains(&stroke.tool.as_str())
                || !stroke.width.is_finite()
                || stroke.width <= 0.0
            {
                // Its ID stays taken: a second stroke can't borrow it.
                continue;
            }
            stroke.width = stroke.width.min(MAX_WIDTH);
            stroke.kanso_z = finite(stroke.kanso_z);
            stroke.points = stroke.points.into_iter().filter_map(clean_point).collect();
            stroke.points.truncate(MAX_STROKE_POINTS);
            if stroke.points.is_empty() {
                continue;
            }
        }
        out.push(stroke);
    }
    out
}

fn clean_point(p: Point) -> Option<Point> {
    if !(p.x.is_finite() && p.y.is_finite()) {
        return None;
    }
    Some(Point {
        pressure: if p.pressure.is_finite() {
            p.pressure.clamp(0.0, 1.0)
        } else {
            0.5
        },
        tilt: p.tilt.filter(|(a, b)| a.is_finite() && b.is_finite()),
        ..p
    })
}

#[cfg(test)]
mod tests {
    use proptest::prelude::*;

    use super::super::tests::config;
    use super::*;

    fn wild() -> impl Strategy<Value = f64> {
        prop_oneof![
            3 => -1000.0f64..1000.0,
            1 => Just(f64::NAN),
            1 => Just(f64::INFINITY),
            1 => Just(0.0),
            1 => Just(-5.0),
            1 => Just(900.0),
        ]
    }

    fn name() -> impl Strategy<Value = String> {
        prop_oneof!["a|b|c|d|e", Just(String::new())]
    }

    fn side() -> impl Strategy<Value = Option<String>> {
        prop::option::of(prop_oneof!["top|left|bottom|right", "[a-z]{1,5}"])
    }

    fn arb_stroke() -> impl Strategy<Value = Stroke> {
        (
            name(),
            prop_oneof!["pen|marker|pencil|highlighter", "[a-z]{1,4}"],
            wild(),
            any::<bool>(),
            prop::collection::vec((wild(), wild(), wild()), 0..6),
        )
            .prop_map(|(id, tool, width, deleted, points)| Stroke {
                id,
                tool,
                width,
                deleted,
                points: points
                    .into_iter()
                    .map(|(x, y, p)| Point::new(x, y, p))
                    .collect(),
                ..Stroke::default()
            })
    }

    fn arb_node() -> impl Strategy<Value = Node> {
        (
            name(),
            prop_oneof!["text|file|group|ink", Just(String::new())],
            (wild(), wild(), wild(), wild()),
            any::<bool>(),
            prop::collection::vec(arb_stroke(), 0..3),
        )
            .prop_map(|(id, kind, (x, y, width, height), deleted, strokes)| Node {
                id,
                kind,
                x,
                y,
                width,
                height,
                deleted,
                strokes,
                ..Node::default()
            })
    }

    fn arb_edge() -> impl Strategy<Value = Edge> {
        (name(), name(), name(), side(), side(), any::<bool>()).prop_map(
            |(id, from_node, to_node, from_side, to_end, deleted)| Edge {
                id,
                from_node,
                to_node,
                from_side,
                to_end,
                deleted,
                ..Edge::default()
            },
        )
    }

    fn arb_canvas() -> impl Strategy<Value = Canvas> {
        (
            prop::collection::vec(arb_node(), 0..6),
            prop::collection::vec(arb_edge(), 0..6),
        )
            .prop_map(|(nodes, edges)| Canvas {
                nodes,
                edges,
                ..Canvas::default()
            })
    }

    proptest! {
        #![proptest_config(config(256))]

        /// Whatever a canvas holds, its sanitized form is one the view opens,
        /// and sanitizing again changes nothing.
        #[test]
        fn a_sanitized_canvas_is_always_valid(canvas in arb_canvas()) {
            let clean = canvas.sanitized();
            prop_assert_eq!(clean.validate(), Ok(()));
            prop_assert_eq!(clean.clone().sanitized(), clean);
        }
    }

    #[test]
    fn a_valid_canvas_comes_back_as_it_was() {
        let canvas = Canvas {
            nodes: vec![
                Node {
                    id: "t".into(),
                    kind: "text".into(),
                    width: 10.0,
                    height: 10.0,
                    text: Some("hi".into()),
                    ..Node::default()
                },
                Node {
                    id: "ink".into(),
                    kind: INK.into(),
                    width: 4.0,
                    height: 4.0,
                    strokes: vec![Stroke {
                        id: "s1".into(),
                        tool: "pen".into(),
                        width: 2.0,
                        points: vec![Point::new(1.0, 1.0, 0.5)],
                        ..Stroke::default()
                    }],
                    ..Node::default()
                },
            ],
            edges: vec![Edge {
                id: "e".into(),
                from_node: "t".into(),
                to_node: "ink".into(),
                ..Edge::default()
            }],
            ..Canvas::default()
        };
        assert_eq!(canvas.validate(), Ok(()));
        assert_eq!(canvas.clone().sanitized(), canvas);
    }
}
