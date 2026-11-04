//! Excalidraw in and out of a canvas. Excalidraw stays an interchange
//! format: a canvas exports as a scene (each stroke a `freedraw` element with
//! `points` and `pressures`, each card a box with its text, each arrow an
//! `arrow`), and a scene imports as a canvas (freehand strokes keep their
//! pressure; shapes and lines become ink; text becomes text cards).

use kanso_core::canvas::{Canvas, Node, Point, Stroke};
use serde_json::{Value, json};

use crate::Failure;

/// Excalidraw draws a freehand line `strokeWidth * WIDTH_PER_STROKE` wide,
/// as Kanso's own scene renderer does.
const WIDTH_PER_STROKE: f64 = 2.5;

/// The canvas as an `.excalidraw` file. Pure.
pub fn export(canvas: &Canvas, now: i64) -> String {
    let mut elements: Vec<Value> = Vec::new();
    let mut seed = 1_i64;
    let mut next = || {
        seed += 1;
        seed
    };
    for node in canvas.live_nodes() {
        if node.kind == "ink" {
            for s in node
                .strokes
                .iter()
                .filter(|s| !s.deleted && !s.points.is_empty())
            {
                elements.push(freedraw(node, s, next(), now));
            }
            continue;
        }
        let b = node.bounds();
        let dashed = node.kind == "group";
        elements.push(base(
            &format!("{}-box", node.id),
            "rectangle",
            b.x,
            b.y,
            json!({
                "width": b.width,
                "height": b.height,
                "strokeColor": super::card_color(node.color.as_deref()).unwrap_or_else(|| "#1e1e1e".into()),
                "strokeStyle": if dashed { "dashed" } else { "solid" },
                "roundness": {"type": 3},
            }),
            next(),
            now,
        ));
        let label = match node.kind.as_str() {
            "group" => node.label.clone(),
            "file" | "link" => node
                .title
                .clone()
                .or_else(|| node.url.clone())
                .or_else(|| node.file.clone()),
            _ => node.text.clone(),
        };
        if let Some(text) = label.filter(|t| !t.trim().is_empty()) {
            elements.push(base(
                &format!("{}-text", node.id),
                "text",
                b.x + 10.0,
                b.y + 10.0,
                json!({
                    "width": (b.width - 20.0).max(1.0),
                    "height": 20.0,
                    "text": text,
                    "originalText": text,
                    "fontSize": 16,
                    "fontFamily": 2,
                    "textAlign": "left",
                    "verticalAlign": "top",
                    "baseline": 14,
                    "containerId": null,
                    "lineHeight": 1.25,
                    "autoResize": true,
                }),
                next(),
                now,
            ));
        }
    }
    for edge in canvas.edges.iter().filter(|e| !e.deleted) {
        let (Some(a), Some(b)) = (canvas.node(&edge.from_node), canvas.node(&edge.to_node)) else {
            continue;
        };
        let (a, b) = (a.bounds(), b.bounds());
        let (ax, ay) = (a.x + a.width / 2.0, a.y + a.height / 2.0);
        let (bx, by) = (b.x + b.width / 2.0, b.y + b.height / 2.0);
        elements.push(base(
            &edge.id,
            "arrow",
            ax,
            ay,
            json!({
                "width": (bx - ax).abs(),
                "height": (by - ay).abs(),
                "points": [[0, 0], [bx - ax, by - ay]],
                "lastCommittedPoint": null,
                "startBinding": null,
                "endBinding": null,
                "startArrowhead": null,
                "endArrowhead": if edge.to_end.as_deref() == Some("none") { Value::Null } else { json!("arrow") },
            }),
            next(),
            now,
        ));
    }
    serde_json::to_string_pretty(&json!({
        "type": "excalidraw",
        "version": 2,
        "source": "https://kanso.app",
        "elements": elements,
        "appState": { "viewBackgroundColor": "#ffffff", "gridSize": null },
        "files": {},
    }))
    .unwrap_or_default()
}

fn base(id: &str, kind: &str, x: f64, y: f64, extra: Value, seed: i64, now: i64) -> Value {
    let mut v = json!({
        "id": id,
        "type": kind,
        "x": x,
        "y": y,
        "width": 0,
        "height": 0,
        "angle": 0,
        "strokeColor": "#1e1e1e",
        "backgroundColor": "transparent",
        "fillStyle": "solid",
        "strokeWidth": 1,
        "strokeStyle": "solid",
        "roughness": 0,
        "opacity": 100,
        "groupIds": [],
        "frameId": null,
        "roundness": null,
        "seed": seed,
        "version": 1,
        "versionNonce": seed,
        "isDeleted": false,
        "boundElements": null,
        "updated": now,
        "link": null,
        "locked": false,
    });
    if let (Some(into), Some(from)) = (v.as_object_mut(), extra.as_object()) {
        for (k, val) in from {
            into.insert(k.clone(), val.clone());
        }
    }
    v
}

fn freedraw(node: &Node, s: &Stroke, seed: i64, now: i64) -> Value {
    let first = s.points[0];
    let (ox, oy) = (node.x + first.x, node.y + first.y);
    let points: Vec<Value> = s
        .points
        .iter()
        .map(|p| json!([p.x - first.x, p.y - first.y]))
        .collect();
    let pressures: Vec<f64> = s.points.iter().map(|p| p.pressure).collect();
    let b = s.bounds();
    base(
        &s.id,
        "freedraw",
        ox,
        oy,
        json!({
            "width": b.map_or(0.0, |b| b.width - s.width),
            "height": b.map_or(0.0, |b| b.height - s.width),
            "strokeColor": s.color,
            "strokeWidth": (s.width / WIDTH_PER_STROKE).max(0.1),
            "opacity": (super::pen::opacity(&s.tool) * 100.0).round(),
            "points": points,
            "pressures": pressures,
            "simulatePressure": super::pen::simulated(s),
            "lastCommittedPoint": null,
        }),
        seed,
        now,
    )
}

/// A canvas from an Excalidraw scene. Freehand strokes keep their points and
/// pressures; lines, arrows and shapes become pen strokes along their
/// outline; text becomes text cards. Images and embeds are left out.
pub fn import(source: &str, now: i64) -> Result<Canvas, Failure> {
    let elements = crate::excalidraw::elements(source)?;
    let mut canvas = Canvas::default();
    let mut texts: Vec<(f64, f64, f64, String)> = Vec::new();
    let mut strokes: Vec<Stroke> = Vec::new();
    for e in &elements {
        let kind = e["type"].as_str().unwrap_or("");
        let (x, y) = (num(e, "x"), num(e, "y"));
        let (w, h) = (num(e, "width"), num(e, "height"));
        let color = e["strokeColor"].as_str().unwrap_or("#1e1e1e").to_string();
        let width = (num(e, "strokeWidth").max(0.1) * WIDTH_PER_STROKE).clamp(0.5, 60.0);
        let line = |pts: Vec<(f64, f64)>| Stroke {
            tool: "pen".into(),
            color: color.clone(),
            width,
            points: pts
                .into_iter()
                .map(|(px, py)| Point::new(px, py, super::pen::SIMULATED))
                .collect(),
            ..Stroke::default()
        };
        match kind {
            "freedraw" => {
                let pts = relative_points(e);
                let pressures: Vec<f64> = e["pressures"]
                    .as_array()
                    .map(|a| a.iter().filter_map(Value::as_f64).collect())
                    .unwrap_or_default();
                let real = !e["simulatePressure"].as_bool().unwrap_or(true)
                    && pressures.len() == pts.len();
                let points = pts
                    .iter()
                    .enumerate()
                    .map(|(i, &(px, py))| {
                        let p = if real {
                            pressures[i].clamp(0.0, 1.0)
                        } else {
                            super::pen::SIMULATED
                        };
                        Point::new(x + px, y + py, p)
                    })
                    .collect();
                strokes.push(Stroke {
                    tool: "pen".into(),
                    color,
                    width,
                    points,
                    ..Stroke::default()
                });
            }
            "line" | "arrow" => {
                let pts: Vec<_> = relative_points(e)
                    .into_iter()
                    .map(|(px, py)| (x + px, y + py))
                    .collect();
                if pts.len() >= 2 {
                    strokes.push(line(pts));
                }
            }
            "rectangle" | "frame" | "magicframe" => {
                strokes.push(line(vec![
                    (x, y),
                    (x + w, y),
                    (x + w, y + h),
                    (x, y + h),
                    (x, y),
                ]));
            }
            "diamond" => strokes.push(line(vec![
                (x + w / 2.0, y),
                (x + w, y + h / 2.0),
                (x + w / 2.0, y + h),
                (x, y + h / 2.0),
                (x + w / 2.0, y),
            ])),
            "ellipse" => {
                let (cx, cy, rx, ry) = (x + w / 2.0, y + h / 2.0, w / 2.0, h / 2.0);
                let pts = (0..=32)
                    .map(|i| {
                        let a = i as f64 / 32.0 * std::f64::consts::TAU;
                        (cx + rx * a.cos(), cy + ry * a.sin())
                    })
                    .collect();
                strokes.push(line(pts));
            }
            "text" => {
                let text = e["text"]
                    .as_str()
                    .or(e["originalText"].as_str())
                    .unwrap_or("");
                if !text.trim().is_empty() {
                    texts.push((x, y, w, text.to_string()));
                }
            }
            _ => {}
        }
    }
    for (i, (x, y, w, text)) in texts.into_iter().enumerate() {
        let lines = text.lines().count().max(1) as f64;
        canvas.put_node(
            Node {
                id: format!("t-{}", i + 1),
                kind: "text".into(),
                x,
                y,
                width: w.max(120.0),
                height: (lines * 24.0 + 24.0).max(48.0),
                text: Some(text),
                ..Node::default()
            },
            now,
        );
    }
    for s in strokes {
        // A stroke that can't be drawn (no points, or the canvas is full)
        // is left out; the rest of the scene still comes in.
        let _ = canvas.draw(s, now);
    }
    if canvas.nodes.is_empty() {
        return Err(Failure::Invalid(
            "The drawing has nothing Kanso can show.".into(),
        ));
    }
    Ok(canvas.repaired())
}

fn num(e: &Value, key: &str) -> f64 {
    e[key].as_f64().filter(|v| v.is_finite()).unwrap_or(0.0)
}

fn relative_points(e: &Value) -> Vec<(f64, f64)> {
    e["points"]
        .as_array()
        .map(|a| {
            a.iter()
                .filter_map(|p| Some((p.get(0)?.as_f64()?, p.get(1)?.as_f64()?)))
                .filter(|(x, y)| x.is_finite() && y.is_finite())
                .collect()
        })
        .unwrap_or_default()
}
