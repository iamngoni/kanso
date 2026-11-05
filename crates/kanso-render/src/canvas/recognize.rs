//! One layer of ink as a picture a text recognizer can read: black strokes
//! on the light page, at twice the size, with room around the writing.

use kanso_core::canvas::{Bounds, Canvas, INK, Node};

use super::{Scene, render};
use crate::{Palette, svg};

/// Pixels to the point.
const SCALE: f32 = 2.0;

/// Room around the writing, in points: recognizers read edge-to-edge text
/// badly.
const MARGIN: f64 = 16.0;

/// Ink node `node`'s handwriting as a PNG for a text recognizer: its
/// handwriting strokes only (a highlighter is left out), every one black,
/// on the light page. `None` when it isn't ink, has nothing to read or
/// can't be drawn. Pure.
// no-prop-test: hands the drawing to resvg; the tests below and the engine's draw through it.
pub fn ink_png(node: &Node) -> Option<Vec<u8>> {
    if node.kind != INK {
        return None;
    }
    let strokes: Vec<_> = node
        .handwriting()
        .map(|s| {
            let mut s = s.clone();
            s.color = "#000000".into();
            s.tool = "pen".into();
            s
        })
        .collect();
    if strokes.is_empty() {
        return None;
    }
    let layer = Node {
        strokes,
        ..node.clone()
    };
    let b = layer.bounds();
    let canvas = Canvas {
        nodes: vec![layer],
        ..Canvas::default()
    };
    let scene = Scene {
        area: Some(Bounds {
            x: b.x - MARGIN,
            y: b.y - MARGIN,
            width: b.width + 2.0 * MARGIN,
            height: b.height + 2.0 * MARGIN,
        }),
        ..Scene::default()
    };
    let drawing = render(&canvas, &scene, &Palette::LIGHT).ok()?;
    svg::rasterize(&drawing.markup, SCALE, super::obsidian::PNG_MAX_SIDE)
}

#[cfg(test)]
mod tests {
    use super::*;
    use kanso_core::canvas::{Point, Stroke};

    fn writing(tool: &str) -> Node {
        Node {
            id: "i1".into(),
            kind: INK.into(),
            strokes: vec![Stroke {
                id: "s1".into(),
                tool: tool.into(),
                color: "#e03131".into(),
                width: 3.0,
                points: (0..20)
                    .map(|i| Point::new(f64::from(i) * 5.0, f64::from(i % 5) * 4.0, 0.5))
                    .collect(),
                ..Stroke::default()
            }],
            ..Node::default()
        }
    }

    #[test]
    fn writing_becomes_a_png_and_a_wash_does_not() {
        let png = ink_png(&writing("pen")).unwrap();
        assert!(png.starts_with(b"\x89PNG"));
        assert!(ink_png(&writing("highlighter")).is_none());
        assert!(ink_png(&Node::default()).is_none());
    }
}
