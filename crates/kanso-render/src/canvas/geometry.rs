//! Where an arrow runs: the same curve the SVG draws, for views that paint
//! it themselves (the phone's canvas).

use kanso_core::canvas::{Edge, Node};

/// A cubic curve from `from` to `to`, with the arrowhead each end carries.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Curve {
    pub from: (f64, f64),
    pub c1: (f64, f64),
    pub c2: (f64, f64),
    pub to: (f64, f64),
    /// Arrowheads at the start and the end. JSON Canvas's default is an
    /// arrow at the far end only.
    pub from_arrow: bool,
    pub to_arrow: bool,
}

/// The middle of the side of `node` an arrow uses (`side`, else the side
/// facing `toward`), and that side's outward normal.
fn anchor(node: &Node, side: Option<&str>, toward: (f64, f64)) -> ((f64, f64), (f64, f64)) {
    let b = node.bounds();
    let (cx, cy) = (b.x + b.width / 2.0, b.y + b.height / 2.0);
    let side = match side {
        Some(s @ ("top" | "bottom" | "left" | "right")) => s,
        _ => {
            let (dx, dy) = (toward.0 - cx, toward.1 - cy);
            if dx.abs() * b.height > dy.abs() * b.width {
                if dx > 0.0 { "right" } else { "left" }
            } else if dy > 0.0 {
                "bottom"
            } else {
                "top"
            }
        }
    };
    match side {
        "top" => ((cx, b.y), (0.0, -1.0)),
        "bottom" => ((cx, b.bottom()), (0.0, 1.0)),
        "left" => ((b.x, cy), (-1.0, 0.0)),
        _ => ((b.right(), cy), (1.0, 0.0)),
    }
}

/// The curve of `edge` between the cards `from` and `to`. Pure.
pub fn curve(edge: &Edge, from: &Node, to: &Node) -> Curve {
    let (fb, tb) = (from.bounds(), to.bounds());
    let fc = (fb.x + fb.width / 2.0, fb.y + fb.height / 2.0);
    let tc = (tb.x + tb.width / 2.0, tb.y + tb.height / 2.0);
    let (a, da) = anchor(from, edge.from_side.as_deref(), tc);
    let (b, db) = anchor(to, edge.to_side.as_deref(), fc);
    let reach = ((a.0 - b.0).hypot(a.1 - b.1) / 3.0).clamp(20.0, 160.0);
    Curve {
        from: a,
        c1: (a.0 + da.0 * reach, a.1 + da.1 * reach),
        c2: (b.0 + db.0 * reach, b.1 + db.1 * reach),
        to: b,
        from_arrow: edge.from_end.as_deref() == Some("arrow"),
        to_arrow: edge.to_end.as_deref().unwrap_or("arrow") == "arrow",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn card(id: &str, x: f64, y: f64) -> Node {
        Node {
            id: id.into(),
            kind: "text".into(),
            x,
            y,
            width: 100.0,
            height: 50.0,
            ..Node::default()
        }
    }

    #[test]
    fn an_arrow_leaves_the_side_facing_the_other_card() {
        let edge = Edge {
            id: "e".into(),
            from_node: "a".into(),
            to_node: "b".into(),
            ..Edge::default()
        };
        let c = curve(&edge, &card("a", 0.0, 0.0), &card("b", 300.0, 0.0));
        assert_eq!(c.from, (100.0, 25.0));
        assert_eq!(c.to, (300.0, 25.0));
        assert!(c.c1.0 > c.from.0 && c.c2.0 < c.to.0);
        assert!(c.to_arrow && !c.from_arrow);
        let down = curve(&edge, &card("a", 0.0, 0.0), &card("b", 0.0, 300.0));
        assert_eq!(down.from, (50.0, 50.0));
    }

    #[test]
    fn a_named_side_wins_and_a_bad_one_is_ignored() {
        let mut edge = Edge {
            id: "e".into(),
            from_side: Some("top".into()),
            to_side: Some("sideways".into()),
            to_end: Some("none".into()),
            ..Edge::default()
        };
        let c = curve(&edge, &card("a", 0.0, 0.0), &card("b", 300.0, 0.0));
        assert_eq!(c.from, (50.0, 0.0));
        assert_eq!(c.to, (300.0, 25.0));
        assert!(!c.to_arrow);
        edge.to_end = None;
        assert!(curve(&edge, &card("a", 0.0, 0.0), &card("b", 1.0, 1.0)).to_arrow);
    }
}
