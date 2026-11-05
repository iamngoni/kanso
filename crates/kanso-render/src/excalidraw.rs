//! Excalidraw scenes drawn natively: rectangles, ellipses, diamonds, lines,
//! arrows, freehand strokes, text and embedded images, in Excalidraw's
//! hand-drawn style ([`crate::rough`]) and its colors. On a dark page the
//! colors are turned as Excalidraw's own dark mode turns them.
//!
//! A scene is the JSON of an `.excalidraw` file, of Excalidraw's clipboard,
//! or a bare list of elements.

use std::fmt::Write as _;

use serde_json::Value;

use crate::rough::{Op, Options, Rough, path_data};
use crate::{Failure, Outcome, Palette, Svg, escape, svg};

/// The largest scene drawn, in bytes of JSON and in elements.
const MAX_SOURCE: usize = 8 * 1024 * 1024;
const MAX_ELEMENTS: usize = 10_000;
/// Room around a scene, in points.
const PAD: f32 = 12.;

/// A scene's elements, or why it isn't one. Pure.
pub fn elements(source: &str) -> Result<Vec<Value>, Failure> {
    if source.len() > MAX_SOURCE {
        return Err(Failure::Invalid("The drawing is too large to show.".into()));
    }
    let value: Value = serde_json::from_str(source)
        .map_err(|err| Failure::Invalid(format!("The drawing isn't valid JSON: {err}")))?;
    let elements = match &value {
        Value::Array(elements) => elements.clone(),
        Value::Object(scene) => match scene.get("elements") {
            Some(Value::Array(elements)) => elements.clone(),
            _ => return Err(Failure::Invalid("The drawing has no elements.".into())),
        },
        _ => return Err(Failure::Invalid("That isn't an Excalidraw drawing.".into())),
    };
    if elements.len() > MAX_ELEMENTS {
        return Err(Failure::Invalid(
            "The drawing has too many elements to show.".into(),
        ));
    }
    Ok(elements
        .into_iter()
        .filter(|e| e.is_object() && !e["isDeleted"].as_bool().unwrap_or(false))
        .collect())
}

/// Whether `source` reads as an Excalidraw scene: for a file or the
/// clipboard brought back in. Pure.
pub fn is_scene(source: &str) -> bool {
    let Ok(value) = serde_json::from_str::<Value>(source) else {
        return false;
    };
    match value["type"].as_str() {
        Some("excalidraw" | "excalidraw/clipboard") => value["elements"].is_array(),
        _ => false,
    }
}

/// A scene as an `.excalidraw` file's JSON: what Excalidraw opens.
pub fn to_file(source: &str) -> Result<String, Failure> {
    let value: Value = serde_json::from_str(source)
        .map_err(|err| Failure::Invalid(format!("The drawing isn't valid JSON: {err}")))?;
    let (elements, files) = match value {
        Value::Array(elements) => (elements, Value::Object(Default::default())),
        Value::Object(mut scene) => (
            match scene.remove("elements") {
                Some(Value::Array(elements)) => elements,
                _ => Vec::new(),
            },
            scene
                .remove("files")
                .unwrap_or_else(|| Value::Object(Default::default())),
        ),
        _ => return Err(Failure::Invalid("That isn't an Excalidraw drawing.".into())),
    };
    let file = serde_json::json!({
        "type": "excalidraw",
        "version": 2,
        "source": "https://kanso.app",
        "elements": elements,
        "appState": { "viewBackgroundColor": "#ffffff", "gridSize": null },
        "files": files,
    });
    Ok(serde_json::to_string_pretty(&file).unwrap_or_default())
}

/// A scene as Excalidraw's clipboard holds it, to paste into excalidraw.com.
pub fn to_clipboard(source: &str) -> Result<String, Failure> {
    let file: Value = serde_json::from_str(&to_file(source)?).unwrap_or_default();
    let clip = serde_json::json!({
        "type": "excalidraw/clipboard",
        "elements": file["elements"],
        "files": file["files"],
    });
    Ok(serde_json::to_string(&clip).unwrap_or_default())
}

/// A scene from a file or the clipboard, as a note keeps it: the elements
/// (and any images they show), compact. Pure.
pub fn to_note(source: &str) -> Result<String, Failure> {
    let file: Value = serde_json::from_str(&to_file(source)?).unwrap_or_default();
    let mut scene = serde_json::Map::new();
    scene.insert("type".into(), "excalidraw".into());
    scene.insert("version".into(), 2.into());
    scene.insert("elements".into(), file["elements"].clone());
    if file["files"].as_object().is_some_and(|f| !f.is_empty()) {
        scene.insert("files".into(), file["files"].clone());
    }
    Ok(serde_json::to_string(&Value::Object(scene)).unwrap_or_default())
}

/// Draws a scene.
pub(crate) fn render(source: &str, palette: &Palette) -> Outcome {
    let elements = elements(source)?;
    if elements.is_empty() {
        return Err(Failure::Invalid("The drawing is empty.".into()));
    }
    let files: Value = serde_json::from_str::<Value>(source)
        .ok()
        .map(|v| v["files"].clone())
        .unwrap_or(Value::Null);
    let mut body = String::new();
    for element in &elements {
        draw(element, &files, palette.dark, &mut body);
    }
    let markup = format!(
        r#"<svg xmlns="http://www.w3.org/2000/svg" xmlns:xlink="http://www.w3.org/1999/xlink">{body}</svg>"#
    );
    let flat = svg::flatten(&markup, PAD)
        .ok_or_else(|| Failure::Invalid("The drawing has nothing to show.".into()))?;
    Ok(Svg {
        markup: flat.markup,
        width: flat.width,
        height: flat.height,
        baseline: None,
        notices: Vec::new(),
    })
}

fn num(e: &Value, key: &str, default: f32) -> f32 {
    e[key]
        .as_f64()
        .map(|v| v as f32)
        .filter(|v| v.is_finite())
        .unwrap_or(default)
}

/// A color from the scene, safe in an attribute, turned for a dark page.
/// `None` for transparent.
fn color(raw: Option<&str>, dark: bool) -> Option<String> {
    let raw = raw?.trim();
    if raw.is_empty() || raw.eq_ignore_ascii_case("transparent") || raw.len() > 40 {
        return None;
    }
    let rgb = parse_color(raw);
    match (rgb, dark) {
        (Some(rgb), true) => Some(hex(dark_mode(rgb))),
        (Some(rgb), false) => Some(hex(rgb)),
        // A name or function usvg will read: allowed characters only.
        (None, _)
            if raw
                .chars()
                .all(|c| c.is_ascii_alphanumeric() || "#(),.% ".contains(c)) =>
        {
            Some(raw.to_owned())
        }
        (None, _) => Some(hex(if dark {
            [0xE0, 0xE0, 0xE0]
        } else {
            [0x1E, 0x1E, 0x1E]
        })),
    }
}

fn hex([r, g, b]: [u8; 3]) -> String {
    format!("#{r:02x}{g:02x}{b:02x}")
}

/// `#rgb`, `#rrggbb` or `#rrggbbaa` (alpha dropped), or black and white. Pure.
fn parse_color(raw: &str) -> Option<[u8; 3]> {
    match raw.to_ascii_lowercase().as_str() {
        "black" => return Some([0, 0, 0]),
        "white" => return Some([255, 255, 255]),
        _ => {}
    }
    let digits = raw.strip_prefix('#')?;
    let value = |s: &str| u8::from_str_radix(s, 16).ok();
    match digits.len() {
        3 | 4 => {
            let c: Vec<u8> = digits
                .chars()
                .take(3)
                .map(|c| value(&c.to_string()).map(|v| v * 17))
                .collect::<Option<_>>()?;
            Some([c[0], c[1], c[2]])
        }
        6 | 8 => Some([
            value(&digits[0..2])?,
            value(&digits[2..4])?,
            value(&digits[4..6])?,
        ]),
        _ => None,
    }
}

/// Excalidraw's dark mode: `invert(93%) hue-rotate(180deg)` over the
/// drawing. Pure.
pub(crate) fn dark_mode(rgb: [u8; 3]) -> [u8; 3] {
    let inverted = rgb.map(|c| {
        let c = f32::from(c) / 255.;
        0.93 + c * (1. - 2. * 0.93)
    });
    const HUE_180: [[f32; 3]; 3] = [
        [-0.574, 1.43, 0.144],
        [0.426, 0.43, 0.144],
        [0.426, 1.43, -0.856],
    ];
    HUE_180.map(|row| {
        let v = row[0] * inverted[0] + row[1] * inverted[1] + row[2] * inverted[2];
        (v.clamp(0., 1.) * 255.).round() as u8
    })
}

/// Excalidraw lowers the roughness of small shapes.
fn roughness(e: &Value, width: f32, height: f32, linear: bool) -> f32 {
    let roughness = num(e, "roughness", 1.).clamp(0., 4.);
    let (max, min) = (width.abs().max(height.abs()), width.abs().min(height.abs()));
    if (min >= 20. && max >= 50.) || min >= 15. || (linear && max >= 30.) {
        return roughness;
    }
    (roughness / if max < 10. { 3. } else { 2. }).min(2.5)
}

/// The corner radius Excalidraw gives a rounded shape `size` across.
fn corner_radius(e: &Value, size: f32) -> f32 {
    match e["roundness"]["type"].as_i64() {
        Some(3) => {
            if size <= 128. {
                size * 0.25
            } else {
                32.
            }
        }
        Some(_) => size * 0.25,
        None => 0.,
    }
}

/// Draws one element into `out`.
fn draw(e: &Value, files: &Value, dark: bool, out: &mut String) {
    let kind = e["type"].as_str().unwrap_or("");
    let (x, y) = (num(e, "x", 0.), num(e, "y", 0.));
    let (w, h) = (num(e, "width", 0.), num(e, "height", 0.));
    let stroke_width = num(e, "strokeWidth", 2.).clamp(0.1, 40.);
    let stroke = color(e["strokeColor"].as_str().or(Some("#1e1e1e")), dark);
    let fill = color(e["backgroundColor"].as_str(), dark);
    let fill_style = e["fillStyle"].as_str().unwrap_or("hachure");
    let stroke_style = e["strokeStyle"].as_str().unwrap_or("solid");
    let seed = e["seed"].as_i64().unwrap_or(1);
    let opacity = num(e, "opacity", 100.).clamp(0., 100.) / 100.;
    let angle = num(e, "angle", 0.);
    let linear = matches!(kind, "line" | "arrow" | "freedraw");
    let points = points(e);

    let mut options = Options::new(roughness(e, w, h, linear), stroke_width);
    options.single_stroke = stroke_style != "solid";
    // As Excalidraw: a line's points stay put; a shape's corners wander.
    options.preserve_vertices = linear && options.roughness < 2.;
    let mut rough = Rough::new(options, seed);
    let dash = match stroke_style {
        "dashed" => format!(
            r#" stroke-dasharray="8 {}""#,
            svg::number(8. + stroke_width)
        ),
        "dotted" => format!(
            r#" stroke-dasharray="1.5 {}""#,
            svg::number(6. + stroke_width)
        ),
        _ => String::new(),
    };

    let mut shape = String::new();
    let stroke_path = |ops: &[Op], shape: &mut String| {
        if let Some(stroke) = &stroke {
            let _ = write!(
                shape,
                r#"<path d="{}" fill="none" stroke="{stroke}" stroke-width="{}" stroke-linecap="round" stroke-linejoin="round"{dash}/>"#,
                path_data(ops),
                svg::number(stroke_width)
            );
        }
    };
    let fill_area = |rough: &mut Rough, polygon: &[(f32, f32)], shape: &mut String| {
        let Some(fill) = &fill else { return };
        match fill_style {
            "solid" => {
                let ops = rough.solid_fill(polygon);
                let _ = write!(
                    shape,
                    r#"<path d="{}Z" fill="{fill}" stroke="none"/>"#,
                    path_data(&ops)
                );
            }
            style => {
                let ops = rough.hachure(polygon, style == "cross-hatch");
                let _ = write!(
                    shape,
                    r#"<path d="{}" fill="none" stroke="{fill}" stroke-width="{}" stroke-linecap="round"/>"#,
                    path_data(&ops),
                    svg::number(stroke_width / 2.)
                );
            }
        }
    };

    let (cx, cy) = match kind {
        "line" | "arrow" | "freedraw" => {
            let (lo, hi) = bounds(&points);
            (x + (lo.0 + hi.0) / 2., y + (lo.1 + hi.1) / 2.)
        }
        _ => (x + w / 2., y + h / 2.),
    };

    match kind {
        "rectangle" | "diamond" => {
            let corners: Vec<(f32, f32)> = if kind == "rectangle" {
                vec![(x, y), (x + w, y), (x + w, y + h), (x, y + h)]
            } else {
                vec![
                    (x + w / 2., y),
                    (x + w, y + h / 2.),
                    (x + w / 2., y + h),
                    (x, y + h / 2.),
                ]
            };
            let size = if kind == "rectangle" {
                w.min(h)
            } else {
                w.min(h) / 2.
            };
            let radius = corner_radius(e, size.abs());
            let outline = rounded(&corners, radius);
            fill_area(&mut rough, &outline, &mut shape);
            let ops = if radius > 0. {
                rounded_ops(&mut rough, &corners, radius)
            } else {
                rough.linear_path(&corners, true)
            };
            stroke_path(&ops, &mut shape);
        }
        "ellipse" => {
            let (ops, _) = rough.ellipse(cx, cy, w, h);
            // Filled along a smooth outline: rough.js's few outline points
            // would show as corners in a solid fill.
            let outline: Vec<(f32, f32)> = (0..64)
                .map(|i| {
                    let a = i as f32 / 64. * std::f32::consts::TAU;
                    (cx + w / 2. * a.cos(), cy + h / 2. * a.sin())
                })
                .collect();
            fill_area(&mut rough, &outline, &mut shape);
            stroke_path(&ops, &mut shape);
        }
        "line" | "arrow" => {
            let absolute: Vec<(f32, f32)> =
                points.iter().map(|(px, py)| (x + px, y + py)).collect();
            if absolute.len() < 2 {
                return;
            }
            let closed = kind == "line"
                && absolute.len() > 2
                && near(absolute[0], absolute[absolute.len() - 1]);
            if closed {
                fill_area(&mut rough, &absolute, &mut shape);
            }
            let curved = !e["roundness"].is_null()
                && absolute.len() > 2
                && !e["elbowed"].as_bool().unwrap_or(false);
            let ops = if curved {
                rough.curve(&absolute)
            } else {
                rough.linear_path(&absolute, false)
            };
            stroke_path(&ops, &mut shape);
            if kind == "arrow" {
                let n = absolute.len();
                for (head, tip, from) in [
                    (e["endArrowhead"].as_str(), absolute[n - 1], absolute[n - 2]),
                    (e["startArrowhead"].as_str(), absolute[0], absolute[1]),
                ] {
                    if let Some(head) = head {
                        arrowhead(
                            &mut rough,
                            head,
                            tip,
                            from,
                            stroke_width,
                            &stroke,
                            &mut shape,
                        );
                    }
                }
            }
        }
        "freedraw" => {
            let absolute: Vec<(f32, f32)> =
                points.iter().map(|(px, py)| (x + px, y + py)).collect();
            if let (Some(stroke), Some(first)) = (&stroke, absolute.first()) {
                let mut d = format!("M{} {}", svg::number(first.0), svg::number(first.1));
                if absolute.len() == 1 {
                    let _ = write!(
                        d,
                        "L{} {}",
                        svg::number(first.0 + 0.01),
                        svg::number(first.1)
                    );
                }
                for pair in absolute.windows(2) {
                    let mid = ((pair[0].0 + pair[1].0) / 2., (pair[0].1 + pair[1].1) / 2.);
                    let _ = write!(
                        d,
                        "Q{} {} {} {}",
                        svg::number(pair[0].0),
                        svg::number(pair[0].1),
                        svg::number(mid.0),
                        svg::number(mid.1)
                    );
                }
                if let Some(last) = absolute.last() {
                    let _ = write!(d, "L{} {}", svg::number(last.0), svg::number(last.1));
                }
                let _ = write!(
                    shape,
                    r#"<path d="{d}" fill="none" stroke="{stroke}" stroke-width="{}" stroke-linecap="round" stroke-linejoin="round"/>"#,
                    svg::number(stroke_width * 2.5)
                );
            }
        }
        "text" => text(e, x, y, w, &stroke, &mut shape),
        "image" => {
            let id = e["fileId"].as_str().unwrap_or("");
            let url = files[id]["dataURL"].as_str().unwrap_or("");
            let safe = [
                "data:image/png;",
                "data:image/jpeg;",
                "data:image/gif;",
                "data:image/webp;",
            ]
            .iter()
            .any(|p| url.starts_with(p));
            if safe && w > 0. && h > 0. {
                let _ = write!(
                    shape,
                    r#"<image x="{}" y="{}" width="{}" height="{}" preserveAspectRatio="none" xlink:href="{}"/>"#,
                    svg::number(x),
                    svg::number(y),
                    svg::number(w),
                    svg::number(h),
                    escape(url)
                );
            } else if w > 0. && h > 0. {
                let ops =
                    rough.linear_path(&[(x, y), (x + w, y), (x + w, y + h), (x, y + h)], true);
                stroke_path(&ops, &mut shape);
            }
        }
        // Frames, embeds and kinds Excalidraw adds later: not drawn.
        _ => return,
    }
    if shape.is_empty() {
        return;
    }
    let mut attributes = String::new();
    if angle.abs() > 1e-6 {
        let _ = write!(
            attributes,
            r#" transform="rotate({} {} {})""#,
            svg::number(angle.to_degrees()),
            svg::number(cx),
            svg::number(cy)
        );
    }
    if opacity < 1. {
        let _ = write!(attributes, r#" opacity="{}""#, svg::number(opacity));
    }
    let _ = write!(out, "<g{attributes}>{shape}</g>");
}

/// A linear element's points, relative to its `x` and `y`.
fn points(e: &Value) -> Vec<(f32, f32)> {
    e["points"]
        .as_array()
        .map(|points| {
            points
                .iter()
                .filter_map(|p| {
                    let x = p.get(0)?.as_f64()? as f32;
                    let y = p.get(1)?.as_f64()? as f32;
                    (x.is_finite() && y.is_finite()).then_some((x, y))
                })
                .take(50_000)
                .collect()
        })
        .unwrap_or_default()
}

fn bounds(points: &[(f32, f32)]) -> ((f32, f32), (f32, f32)) {
    if points.is_empty() {
        return ((0., 0.), (0., 0.));
    }
    points.iter().fold(
        ((f32::MAX, f32::MAX), (f32::MIN, f32::MIN)),
        |((x0, y0), (x1, y1)), &(x, y)| ((x0.min(x), y0.min(y)), (x1.max(x), y1.max(y))),
    )
}

fn near(a: (f32, f32), b: (f32, f32)) -> bool {
    (a.0 - b.0).hypot(a.1 - b.1) < 8.
}

/// The outline of a polygon with its corners rounded by `radius`, as points.
fn rounded(corners: &[(f32, f32)], radius: f32) -> Vec<(f32, f32)> {
    if radius <= 0. {
        return corners.to_vec();
    }
    let n = corners.len();
    let mut out = Vec::new();
    for i in 0..n {
        let (prev, c, next) = (corners[(i + n - 1) % n], corners[i], corners[(i + 1) % n]);
        let (a, b) = (toward(c, prev, radius), toward(c, next, radius));
        for step in 0..=4 {
            let t = step as f32 / 4.;
            // A quadratic from a to b around the corner.
            let u = 1. - t;
            out.push((
                u * u * a.0 + 2. * u * t * c.0 + t * t * b.0,
                u * u * a.1 + 2. * u * t * c.1 + t * t * b.1,
            ));
        }
    }
    out
}

/// The point `radius` from `from` toward `to`, but no further than halfway.
fn toward(from: (f32, f32), to: (f32, f32), radius: f32) -> (f32, f32) {
    let length = (to.0 - from.0).hypot(to.1 - from.1);
    if length < 1e-6 {
        return from;
    }
    let r = radius.min(length / 2.);
    (
        from.0 + (to.0 - from.0) / length * r,
        from.1 + (to.1 - from.1) / length * r,
    )
}

/// A rounded polygon's rough outline: rough straight sides, and each corner
/// a rough curve.
fn rounded_ops(rough: &mut Rough, corners: &[(f32, f32)], radius: f32) -> Vec<Op> {
    let n = corners.len();
    let mut ops = Vec::new();
    for i in 0..n {
        let (c, next, after) = (corners[i], corners[(i + 1) % n], corners[(i + 2) % n]);
        let start = toward(c, next, radius);
        let end = toward(next, c, radius);
        ops.extend(rough.line(start, end));
        let leave = toward(next, after, radius);
        // The corner at `next`, as a cubic from the quadratic.
        let (q0, q1, q2) = (end, next, leave);
        let c1 = (
            q0.0 + 2. / 3. * (q1.0 - q0.0),
            q0.1 + 2. / 3. * (q1.1 - q0.1),
        );
        let c2 = (
            q2.0 + 2. / 3. * (q1.0 - q2.0),
            q2.1 + 2. / 3. * (q1.1 - q2.1),
        );
        ops.push(Op::Move(q0.0, q0.1));
        ops.push(Op::Curve(c1.0, c1.1, c2.0, c2.1, q2.0, q2.1));
    }
    ops
}

/// An arrowhead at `tip`, the line arriving from `from`.
fn arrowhead(
    rough: &mut Rough,
    head: &str,
    tip: (f32, f32),
    from: (f32, f32),
    stroke_width: f32,
    stroke: &Option<String>,
    out: &mut String,
) {
    let Some(stroke) = stroke else { return };
    let length = (tip.0 - from.0).hypot(tip.1 - from.1);
    if length < 1e-3 {
        return;
    }
    let (nx, ny) = ((tip.0 - from.0) / length, (tip.1 - from.1) / length);
    let size: f32 = match head {
        "arrow" => 25.,
        "diamond" | "diamond_outline" => 12.,
        "bar" => 15.,
        _ => 15.,
    };
    let multiplier = if head.starts_with("diamond") {
        0.25
    } else {
        0.5
    };
    let size = size.min(length * multiplier).max(stroke_width * 2.);
    let base = (tip.0 - nx * size, tip.1 - ny * size);
    let spread = match head {
        "bar" => 90f32,
        "arrow" => 20.,
        _ => 25.,
    }
    .to_radians();
    let rotate = |p: (f32, f32), a: f32| {
        let (s, c) = a.sin_cos();
        let (dx, dy) = (p.0 - tip.0, p.1 - tip.1);
        (tip.0 + dx * c - dy * s, tip.1 + dx * s + dy * c)
    };
    let (left, right) = (rotate(base, spread), rotate(base, -spread));
    let width = svg::number(stroke_width);
    let line = |ops: &[Op], out: &mut String| {
        let _ = write!(
            out,
            r#"<path d="{}" fill="none" stroke="{stroke}" stroke-width="{width}" stroke-linecap="round" stroke-linejoin="round"/>"#,
            path_data(ops)
        );
    };
    match head {
        "dot" | "circle" | "circle_outline" => {
            let r = (size / 2.).max(stroke_width);
            let fill = if head == "circle_outline" {
                "none"
            } else {
                stroke.as_str()
            };
            let center = (tip.0 - nx * r, tip.1 - ny * r);
            let _ = write!(
                out,
                r#"<circle cx="{}" cy="{}" r="{}" fill="{fill}" stroke="{stroke}" stroke-width="{width}"/>"#,
                svg::number(center.0),
                svg::number(center.1),
                svg::number(r)
            );
        }
        "triangle" | "triangle_outline" => {
            let fill = if head == "triangle" {
                stroke.as_str()
            } else {
                "none"
            };
            let ops = rough.linear_path(&[tip, left, right], true);
            let _ = write!(
                out,
                r#"<path d="M{} {}L{} {}L{} {}Z" fill="{fill}" stroke="none"/>"#,
                svg::number(tip.0),
                svg::number(tip.1),
                svg::number(left.0),
                svg::number(left.1),
                svg::number(right.0),
                svg::number(right.1)
            );
            line(&ops, out);
        }
        "diamond" | "diamond_outline" => {
            let back = (tip.0 - nx * size * 2., tip.1 - ny * size * 2.);
            let side = |sign: f32| {
                (
                    base.0 - ny * size * 0.5 * sign,
                    base.1 + nx * size * 0.5 * sign,
                )
            };
            let corners = [tip, side(1.), back, side(-1.)];
            if head == "diamond" {
                let ops = rough.solid_fill(&corners);
                let _ = write!(
                    out,
                    r#"<path d="{}Z" fill="{stroke}" stroke="none"/>"#,
                    path_data(&ops)
                );
            }
            line(&rough.linear_path(&corners, true), out);
        }
        "bar" => {
            let half = size / 2.;
            let a = (tip.0 - ny * half, tip.1 + nx * half);
            let b = (tip.0 + ny * half, tip.1 - nx * half);
            line(&rough.line(a, b), out);
        }
        _ => {
            let mut ops = rough.line(left, tip);
            ops.extend(rough.line(right, tip));
            line(&ops, out);
        }
    }
}

/// A text element: its lines, in its font and alignment.
fn text(e: &Value, x: f32, y: f32, w: f32, stroke: &Option<String>, out: &mut String) {
    let Some(stroke) = stroke else { return };
    let content = e["text"]
        .as_str()
        .or(e["originalText"].as_str())
        .unwrap_or("");
    if content.trim().is_empty() {
        return;
    }
    let size = num(e, "fontSize", 20.).clamp(1., 400.);
    let line_height = num(e, "lineHeight", 1.25).clamp(0.5, 4.) * size;
    let family = match e["fontFamily"].as_i64().unwrap_or(1) {
        2 | 6 | 9 => "'Helvetica Neue', 'Segoe UI', Inter, 'Noto Sans', Arial, sans-serif",
        3 => "'Cascadia Code', 'SF Mono', Menlo, Consolas, 'DejaVu Sans Mono', monospace",
        // Virgil and Excalifont, Excalidraw's own hands, or the nearest
        // informal face this machine has.
        _ => {
            "Virgil, Excalifont, 'Comic Shanns', 'Chalkboard SE', 'Segoe Print', 'Comic Neue', 'Comic Sans MS', 'Bradley Hand', cursive, sans-serif"
        }
    };
    let (anchor, tx) = match e["textAlign"].as_str() {
        Some("center") => ("middle", x + w / 2.),
        Some("right") => ("end", x + w),
        _ => ("start", x),
    };
    let _ = write!(
        out,
        r#"<text font-size="{}" font-family="{}" fill="{stroke}" text-anchor="{anchor}" xml:space="preserve">"#,
        svg::number(size),
        escape(family)
    );
    for (i, line) in content.split('\n').enumerate() {
        // The baseline sits a little below the middle of its line.
        let baseline = y + i as f32 * line_height + line_height / 2. + size * 0.33;
        let _ = write!(
            out,
            r#"<tspan x="{}" y="{}">{}</tspan>"#,
            svg::number(tx),
            svg::number(baseline),
            escape(line)
        );
    }
    out.push_str("</text>");
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;

    pub(crate) const SCENE: &str = r##"{
        "type": "excalidraw", "version": 2,
        "elements": [
            {"type": "rectangle", "x": 0, "y": 0, "width": 200, "height": 100, "strokeColor": "#1e1e1e", "backgroundColor": "#a5d8ff", "fillStyle": "hachure", "strokeWidth": 2, "roughness": 1, "seed": 7, "roundness": {"type": 3}},
            {"type": "ellipse", "x": 260, "y": 0, "width": 120, "height": 90, "strokeColor": "#e03131", "backgroundColor": "#ffc9c9", "fillStyle": "solid", "strokeWidth": 2, "roughness": 1, "seed": 9},
            {"type": "diamond", "x": 420, "y": 0, "width": 100, "height": 100, "strokeColor": "#2f9e44", "backgroundColor": "transparent", "strokeWidth": 2, "roughness": 2, "seed": 11, "strokeStyle": "dashed"},
            {"type": "arrow", "x": 200, "y": 50, "width": 60, "height": 0, "points": [[0, 0], [60, 0]], "strokeColor": "#1e1e1e", "endArrowhead": "arrow", "strokeWidth": 2, "roughness": 1, "seed": 13},
            {"type": "line", "x": 0, "y": 140, "width": 300, "height": 40, "points": [[0, 0], [150, 40], [300, 0]], "roundness": {"type": 2}, "strokeColor": "#1971c2", "strokeWidth": 1, "roughness": 1, "seed": 15},
            {"type": "freedraw", "x": 340, "y": 140, "width": 80, "height": 40, "points": [[0, 0], [20, 30], [40, 10], [80, 40]], "strokeColor": "#1e1e1e", "strokeWidth": 1, "seed": 17},
            {"type": "text", "x": 40, "y": 35, "width": 120, "height": 25, "text": "Hello <Kanso>", "fontSize": 20, "fontFamily": 1, "textAlign": "center", "strokeColor": "#1e1e1e", "seed": 19},
            {"type": "rectangle", "x": 0, "y": 0, "width": 10, "height": 10, "isDeleted": true}
        ],
        "appState": {}, "files": {}
    }"##;

    #[test]
    fn a_scene_draws() {
        for palette in [Palette::LIGHT, Palette::DARK] {
            let svg = render(SCENE, &palette).unwrap();
            assert!(svg.width > 500. && svg.height > 150., "{svg:?}");
            let tree = usvg::Tree::from_str(&svg.markup, &usvg::Options::default()).unwrap();
            assert!((tree.size().width() - svg.width).abs() < 0.01);
        }
        assert_eq!(elements(SCENE).unwrap().len(), 7);
    }

    /// Writes the sample scene, light and dark, to `$KANSO_RENDER_DUMP`,
    /// to look at.
    #[test]
    #[ignore]
    fn dump_sample() {
        let Some(dir) = std::env::var_os("KANSO_RENDER_DUMP") else {
            return;
        };
        for (name, palette) in [("light", Palette::LIGHT), ("dark", Palette::DARK)] {
            let svg = render(SCENE, &palette).unwrap();
            std::fs::write(
                std::path::Path::new(&dir).join(format!("scene-{name}.svg")),
                svg.markup,
            )
            .unwrap();
        }
    }

    #[test]
    fn scenes_come_in_three_shapes() {
        let file = SCENE;
        let clip = to_clipboard(file).unwrap();
        assert!(is_scene(file) && is_scene(&clip));
        let bare = serde_json::to_string(&serde_json::from_str::<Value>(file).unwrap()["elements"])
            .unwrap();
        assert!(!is_scene(&bare));
        for source in [file, clip.as_str(), bare.as_str()] {
            assert_eq!(elements(source).unwrap().len(), 7);
        }
        let note = to_note(&clip).unwrap();
        assert!(!note.contains('\n'));
        assert_eq!(elements(&note).unwrap().len(), 7);
        assert!(matches!(
            render("{}", &Palette::LIGHT),
            Err(Failure::Invalid(_))
        ));
        assert!(matches!(
            render("[]", &Palette::LIGHT),
            Err(Failure::Invalid(_))
        ));
        assert!(matches!(
            render("not json", &Palette::LIGHT),
            Err(Failure::Invalid(_))
        ));
    }

    #[test]
    fn dark_mode_turns_colors_as_excalidraw_does() {
        // Excalidraw's near-black stroke comes out near-white, white near
        // its dark canvas.
        let light = dark_mode([0x1e, 0x1e, 0x1e]);
        assert!(light.iter().all(|c| *c > 0xC0), "{light:?}");
        let dark = dark_mode([0xff, 0xff, 0xff]);
        assert!(dark.iter().all(|c| *c < 0x20), "{dark:?}");
        assert_eq!(parse_color("#abc"), Some([0xaa, 0xbb, 0xcc]));
        assert_eq!(parse_color("#a5d8ffcc"), Some([0xa5, 0xd8, 0xff]));
        assert_eq!(color(Some("transparent"), false), None);
        assert_eq!(color(Some("\"><script>"), false), Some("#1e1e1e".into()));
    }

    fn arb_element() -> impl Strategy<Value = Value> {
        (
            prop::sample::select(vec!["rectangle", "ellipse", "diamond", "line", "arrow", "freedraw", "text", "frame"]),
            (-500f64..500., -500f64..500., -50f64..400., -50f64..400.),
            prop::collection::vec((-200f64..200., -200f64..200.), 0..6),
            (0u8..3, any::<i32>(), -7f64..7.),
            prop::sample::select(vec!["hachure", "cross-hatch", "solid", "zigzag"]),
            prop::sample::select(vec!["solid", "dashed", "dotted"]),
        )
            .prop_map(|(kind, (x, y, w, h), points, (roughness, seed, angle), fill, style)| {
                serde_json::json!({
                    "type": kind, "x": x, "y": y, "width": w, "height": h,
                    "points": points.iter().map(|(a, b)| vec![*a, *b]).collect::<Vec<_>>(),
                    "roughness": roughness, "seed": seed, "angle": angle,
                    "strokeColor": "#1e1e1e", "backgroundColor": "#ffec99",
                    "fillStyle": fill, "strokeStyle": style, "text": "Hi",
                    "endArrowhead": "triangle", "startArrowhead": "dot",
                    "roundness": if seed % 2 == 0 { serde_json::json!({"type": 3}) } else { Value::Null },
                })
            })
    }

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(64))]

        /// Invariants: any scene draws without panicking; a drawing parses
        /// and covers every element; and drawing is deterministic.
        #[test]
        fn prop_scenes_draw_deterministically(elements in prop::collection::vec(arb_element(), 1..8), dark in any::<bool>()) {
            let scene = serde_json::json!({"type": "excalidraw", "elements": elements}).to_string();
            let palette = if dark { Palette::DARK } else { Palette::LIGHT };
            let first = render(&scene, &palette);
            prop_assert_eq!(&first, &render(&scene, &palette));
            if let Ok(svg) = first {
                prop_assert!(svg.width > 0. && svg.height > 0.);
                prop_assert!(usvg::Tree::from_str(&svg.markup, &usvg::Options::default()).is_ok());
            }
        }

        /// Round trip: a scene saved as a file and brought back to a note
        /// keeps its elements.
        #[test]
        fn prop_file_round_trip(elements in prop::collection::vec(arb_element(), 0..6)) {
            let scene = serde_json::json!({"type": "excalidraw", "elements": elements}).to_string();
            let file = to_file(&scene).unwrap();
            prop_assert!(is_scene(&file));
            let back = to_note(&file).unwrap();
            let a: Value = serde_json::from_str(&scene).unwrap();
            let b: Value = serde_json::from_str(&back).unwrap();
            prop_assert_eq!(&a["elements"], &b["elements"]);
        }

        /// Invariant: dark mode keeps a color a color, and darkens light
        /// colors and lightens dark ones.
        #[test]
        fn prop_dark_mode_flips_lightness(r in any::<u8>(), g in any::<u8>(), b in any::<u8>()) {
            let luma = |c: [u8; 3]| 0.2126 * f32::from(c[0]) + 0.7152 * f32::from(c[1]) + 0.0722 * f32::from(c[2]);
            let before = luma([r, g, b]);
            let after = luma(dark_mode([r, g, b]));
            if before > 200. { prop_assert!(after < 128.); }
            if before < 40. { prop_assert!(after > 128.); }
        }
    }
}
