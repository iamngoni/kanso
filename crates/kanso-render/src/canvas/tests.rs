use std::collections::HashMap;

use kanso_core::canvas::{Bounds, Canvas, Edge, Node, Point, Stroke};
use proptest::prelude::*;

use super::*;

fn stroke(y: f64, pressure: f64) -> Stroke {
    Stroke {
        tool: "pen".into(),
        color: "#e03131".into(),
        width: 3.0,
        points: (0..25)
            .map(|i| Point::new(i as f64 * 6.0, y + (i as f64 * 0.5).sin() * 6.0, pressure))
            .collect(),
        ..Stroke::default()
    }
}

fn sample() -> Canvas {
    let mut c = Canvas::default();
    c.put_node(
        Node {
            id: "n1".into(),
            kind: "text".into(),
            x: 0.0,
            y: 0.0,
            width: 240.0,
            height: 100.0,
            text: Some("Launch plan <b>&</b> a rather long second line that must wrap".into()),
            color: Some("4".into()),
            ..Node::default()
        },
        1,
    );
    c.put_node(
        Node {
            id: "n2".into(),
            kind: "file".into(),
            file: Some("note:AbCd1234".into()),
            title: Some("Old title".into()),
            x: 300.0,
            y: 0.0,
            width: 200.0,
            height: 80.0,
            ..Node::default()
        },
        2,
    );
    c.put_node(
        Node {
            id: "g1".into(),
            kind: "group".into(),
            label: Some("Sketch".into()),
            x: -20.0,
            y: 150.0,
            width: 600.0,
            height: 200.0,
            ..Node::default()
        },
        3,
    );
    c.put_edge(
        Edge {
            id: "e1".into(),
            from_node: "n1".into(),
            to_node: "n2".into(),
            label: Some("then".into()),
            ..Edge::default()
        },
        4,
    );
    c.draw(stroke(200.0, 0.8), 5).unwrap();
    c
}

fn parses(markup: &str) -> bool {
    usvg::Tree::from_str(markup, &usvg::Options::default()).is_ok()
}

#[test]
fn a_canvas_draws_every_kind_of_item() {
    let svg = render(&sample(), &Scene::default(), &Palette::LIGHT).unwrap();
    assert!(parses(&svg.markup), "{}", svg.markup);
    for part in [
        "Launch plan",
        "&lt;b&gt;",
        "Sketch",
        "then",
        "Old title",
        "fill-opacity",
        "viewBox",
    ] {
        assert!(svg.markup.contains(part), "missing {part}");
    }
    assert!(!svg.markup.contains("<b>"));
    assert!(svg.width > 0.0 && svg.height > 0.0);
}

#[test]
fn an_arrow_whose_end_card_was_deleted_is_kept_but_not_drawn() {
    let mut canvas = sample();
    let edge = canvas.edges.iter().find(|e| e.id == "e1").unwrap().clone();
    let end = canvas
        .nodes
        .iter_mut()
        .find(|n| n.id == edge.to_node)
        .unwrap();
    end.deleted = true;
    let svg = render(&canvas, &Scene::default(), &Palette::LIGHT).unwrap();
    assert!(parses(&svg.markup));
    assert!(
        !svg.markup.contains("then"),
        "the hidden arrow's label drew"
    );
    assert!(canvas.edges.iter().any(|e| e.id == "e1" && !e.deleted));
}

#[test]
fn a_note_card_shows_the_live_title() {
    let titles = HashMap::from([("AbCd1234".to_string(), "New title".to_string())]);
    let scene = Scene {
        titles: Some(&titles),
        ..Scene::default()
    };
    let svg = render(&sample(), &scene, &Palette::LIGHT).unwrap();
    assert!(svg.markup.contains("New title") && !svg.markup.contains("Old title"));
}

#[test]
fn the_area_picks_what_is_drawn() {
    let canvas = sample();
    let group = canvas.group_bounds("g1").unwrap();
    let scene = Scene {
        area: Some(group),
        ..Scene::default()
    };
    let svg = render(&canvas, &scene, &Palette::LIGHT).unwrap();
    assert!(parses(&svg.markup));
    assert!(svg.markup.contains(&format!(
        "viewBox=\"{} {} {} {}\"",
        n(group.x),
        n(group.y),
        n(group.width),
        n(group.height)
    )));
    // The text card lies outside the group, so it is culled.
    assert!(!svg.markup.contains("Launch plan"));
}

#[test]
fn an_empty_canvas_or_area_is_refused() {
    assert!(render(&Canvas::default(), &Scene::default(), &Palette::LIGHT).is_err());
    let scene = Scene {
        area: Some(Bounds {
            x: 0.0,
            y: 0.0,
            width: 0.0,
            height: 10.0,
        }),
        ..Scene::default()
    };
    assert!(render(&sample(), &scene, &Palette::LIGHT).is_err());
}

#[test]
fn a_huge_area_is_scaled_down() {
    let scene = Scene {
        area: Some(Bounds {
            x: 0.0,
            y: 0.0,
            width: 100_000.0,
            height: 50_000.0,
        }),
        ..Scene::default()
    };
    let svg = render(&sample(), &scene, &Palette::LIGHT).unwrap();
    assert!(svg.width as f64 <= MAX_SIDE + 1.0);
}

#[test]
fn dark_pages_keep_ink_readable() {
    let mut c = Canvas::default();
    let mut s = stroke(0.0, 0.5);
    s.color = "#1e1e1e".into();
    c.draw(s, 1).unwrap();
    let svg = render(&c, &Scene::default(), &Palette::DARK).unwrap();
    assert!(svg.markup.contains(&Palette::DARK.text.hex()));
    assert!(!svg.markup.contains("#1e1e1e"));
}

#[test]
fn highlighters_are_translucent() {
    let mut c = Canvas::default();
    let mut s = stroke(0.0, 0.5);
    s.tool = "highlighter".into();
    c.draw(s, 1).unwrap();
    let svg = render(&c, &Scene::default(), &Palette::LIGHT).unwrap();
    assert!(svg.markup.contains(r#"fill-opacity="0.35""#));
}

#[test]
fn text_wraps_and_truncates() {
    assert_eq!(
        wrap("one two three four", 9, 10),
        ["one two", "three", "four"]
    );
    let cut = wrap("a\nb\nc\nd", 10, 2);
    assert_eq!(cut.len(), 2);
    assert!(cut[1].ends_with('\u{2026}'));
    assert_eq!(wrap("abcdefghij", 4, 5), ["abcd", "efgh", "ij"]);
    assert!(wrap("", 5, 5).is_empty());
    assert_eq!(wrap("# Heading", 20, 2), ["Heading"]);
}

#[test]
fn excalidraw_export_has_pressures_and_reimports() {
    let canvas = sample();
    let file = excalidraw::export(&canvas, 99);
    let value: serde_json::Value = serde_json::from_str(&file).unwrap();
    assert_eq!(value["type"], "excalidraw");
    let elements = value["elements"].as_array().unwrap();
    let free = elements.iter().find(|e| e["type"] == "freedraw").unwrap();
    assert_eq!(free["points"].as_array().unwrap().len(), 25);
    assert_eq!(free["pressures"].as_array().unwrap().len(), 25);
    assert_eq!(free["simulatePressure"], false);
    assert!(elements.iter().any(|e| e["type"] == "arrow"));
    assert!(elements.iter().any(|e| e["type"] == "text"));
    // Excalidraw's own renderer in Kanso draws the file.
    assert!(crate::excalidraw::render(&file, &Palette::LIGHT).is_ok());

    let back = excalidraw::import(&file, 100).unwrap();
    assert!(back.validate().is_ok());
    let (_, s) = back
        .live_strokes()
        .find(|(_, s)| s.points.len() == 25)
        .unwrap();
    assert!((s.points[3].pressure - 0.8).abs() < 1e-6);
    assert!((s.width - 3.0).abs() < 0.01);
    assert!(back.nodes.iter().any(|n| n.kind == "text"));
}

#[test]
fn excalidraw_import_turns_shapes_into_ink() {
    let scene = r##"{"type":"excalidraw","elements":[
        {"type":"rectangle","x":10,"y":10,"width":50,"height":30,"strokeColor":"#1e1e1e","strokeWidth":2},
        {"type":"ellipse","x":0,"y":0,"width":40,"height":40},
        {"type":"freedraw","x":5,"y":5,"points":[[0,0],[10,5],[20,0]],"pressures":[0.2,0.9,0.4],"simulatePressure":false,"strokeColor":"#e03131","strokeWidth":1},
        {"type":"freedraw","x":5,"y":50,"points":[[0,0],[10,5],[20,0]],"simulatePressure":true},
        {"type":"text","x":0,"y":80,"width":100,"text":"hello"},
        {"type":"image","x":0,"y":0},
        {"type":"line","x":0,"y":0,"points":[[0,0]]}
    ]}"##;
    let c = excalidraw::import(scene, 1).unwrap();
    assert!(c.validate().is_ok());
    assert_eq!(c.live_strokes().count(), 4);
    let pressures: Vec<f64> = c
        .live_strokes()
        .find(|(_, s)| s.color == "#e03131")
        .unwrap()
        .1
        .points
        .iter()
        .map(|p| p.pressure)
        .collect();
    assert_eq!(pressures, [0.2, 0.9, 0.4]);
    assert!(excalidraw::import(r#"{"elements":[{"type":"image"}]}"#, 1).is_err());
    assert!(excalidraw::import("nope", 1).is_err());
}

fn arb_canvas() -> impl Strategy<Value = Canvas> {
    let node = (
        "[a-z]{1,3}",
        prop::sample::select(vec!["text", "file", "link", "group"]),
        -500f64..500.,
        -500f64..500.,
        10f64..400.,
        10f64..300.,
        "\\PC{0,60}",
    );
    let pts = prop::collection::vec((-300f64..300., -300f64..300., 0f64..=1.), 1..30);
    (
        prop::collection::vec(node, 0..8),
        prop::collection::vec(pts, 0..5),
    )
        .prop_map(|(nodes, strokes)| {
            let mut c = Canvas::default();
            for (i, (id, kind, x, y, w, h, text)) in nodes.into_iter().enumerate() {
                c.put_node(
                    Node {
                        id: format!("{id}{i}"),
                        kind: kind.into(),
                        x,
                        y,
                        width: w,
                        height: h,
                        text: Some(text.clone()),
                        title: Some(text.clone()),
                        label: Some(text),
                        file: (kind == "file").then(|| "note:Zz".to_string()),
                        ..Node::default()
                    },
                    1,
                );
            }
            for (i, pts) in strokes.into_iter().enumerate() {
                let s = Stroke {
                    tool: ["pen", "pencil", "marker", "highlighter"][i % 4].into(),
                    color: "#336699".into(),
                    width: 2.0,
                    points: pts
                        .into_iter()
                        .map(|(x, y, p)| Point::new(x, y, p))
                        .collect(),
                    ..Stroke::default()
                };
                let _ = c.draw(s, 2);
            }
            c
        })
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(64))]

    /// Invariants: any canvas draws as an SVG that parses, whatever text is
    /// on its cards (markup is escaped), or is refused as empty; never a
    /// panic.
    #[test]
    fn prop_any_canvas_draws_valid_svg(canvas in arb_canvas(), dark in any::<bool>()) {
        let palette = if dark { Palette::DARK } else { Palette::LIGHT };
        match render(&canvas, &Scene::default(), &palette) {
            Ok(svg) => {
                prop_assert!(parses(&svg.markup), "{}", svg.markup);
                prop_assert!(svg.width.is_finite() && svg.height.is_finite());
                prop_assert!(!svg.markup.contains("NaN") && !svg.markup.contains("inf"));
            }
            Err(_) => prop_assert!(canvas.content_bounds().is_none()),
        }
    }

    /// Invariant: exporting to Excalidraw and importing back keeps every
    /// live stroke's point count and pressures.
    #[test]
    fn prop_excalidraw_round_trip_keeps_ink(canvas in arb_canvas()) {
        let live: Vec<Vec<f64>> = canvas
            .live_strokes()
            .map(|(_, s)| s.points.iter().map(|p| p.pressure).collect())
            .filter(|p: &Vec<f64>| p.iter().any(|v| (*v - 0.5).abs() > 1e-9))
            .collect();
        let file = excalidraw::export(&canvas, 1);
        if let Ok(back) = excalidraw::import(&file, 2) {
            prop_assert!(back.validate().is_ok());
            let got: Vec<Vec<f64>> = back
                .live_strokes()
                .map(|(_, s)| s.points.iter().map(|p| p.pressure).collect())
                .collect();
            for want in live {
                prop_assert!(got.iter().any(|g| g.len() == want.len()
                    && g.iter().zip(&want).all(|(a, b)| (a - b).abs() < 1e-6)),
                    "lost {want:?} in {got:?}");
            }
        }
    }
}
