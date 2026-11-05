//! A canvas drawn as one SVG: cards (text, notes, links, files), groups,
//! arrows and ink. The preview, an embed, an export and the phone's
//! thumbnails all draw from this, and the desktop view and the phone draw
//! the same ink outlines from [`pen`].
//!
//! The drawing covers an area of the canvas: a group's rectangle or the
//! bounds of the content, never an unbounded surface.

pub mod embed;
mod embed_model;
pub mod excalidraw;
pub mod geometry;
pub mod pen;

use std::collections::HashMap;
use std::fmt::Write as _;

use kanso_core::canvas::{Bounds, Canvas, Edge, INK, Node, Stroke};

use crate::{Failure, Outcome, Palette, Svg, escape, freehand, svg};

/// Room around the content, in points.
const PAD: f64 = 24.0;
/// The longest side of a drawing, in points; a larger area is scaled down.
pub const MAX_SIDE: f64 = 4096.0;
/// Text on a card.
const FONT: f64 = 16.0;
/// The most lines a card shows.
const MAX_LINES: usize = 400;

/// The six colors JSON Canvas names.
pub fn preset(name: &str) -> Option<&'static str> {
    Some(match name {
        "1" => "#fb464c",
        "2" => "#e9973f",
        "3" => "#e0de71",
        "4" => "#44cf6e",
        "5" => "#53dfdd",
        "6" => "#a882ff",
        _ => return None,
    })
}

/// A card or arrow colour as hex: a preset, a hex colour, or `None`.
pub fn card_color(color: Option<&str>) -> Option<String> {
    let color = color?;
    if let Some(p) = preset(color) {
        return Some(p.into());
    }
    pen::parse_hex(color).map(|(r, g, b)| format!("#{r:02x}{g:02x}{b:02x}"))
}

/// What a canvas is drawn from.
#[derive(Clone, Debug, Default)]
pub struct Scene<'a> {
    /// The area to show, in canvas coordinates. `None` is the content's
    /// bounds.
    pub area: Option<Bounds>,
    /// Live titles of note cards, by note ID, for cards whose note was
    /// renamed. A card without an entry shows the title stored with it.
    pub titles: Option<&'a HashMap<String, String>>,
    /// Whether to leave the page colour out, for an overlay.
    pub transparent: bool,
    /// The width to show the drawing at, in points; its height follows.
    /// `None` is the area's own size, scaled down to [`MAX_SIDE`].
    pub width: Option<f64>,
}

/// The area `canvas` is drawn over: `area` when given, else the content's
/// bounds, padded. `None` for an empty canvas with no area.
pub fn drawn_area(canvas: &Canvas, area: Option<Bounds>) -> Option<Bounds> {
    match area {
        Some(a) => Some(a),
        None => canvas.content_bounds().map(|b| Bounds {
            x: b.x - PAD,
            y: b.y - PAD,
            width: b.width + 2.0 * PAD,
            height: b.height + 2.0 * PAD,
        }),
    }
}

/// `canvas` as an SVG. Fails for an empty canvas or an area with no size.
pub fn render(canvas: &Canvas, scene: &Scene, palette: &Palette) -> Outcome {
    let Some(area) = drawn_area(canvas, scene.area) else {
        return Err(Failure::Invalid("The canvas is empty.".into()));
    };
    if !(area.width.is_finite() && area.height.is_finite() && area.width > 0.0 && area.height > 0.0)
    {
        return Err(Failure::Invalid("The canvas has no area to show.".into()));
    }
    let scale = match scene.width.filter(|w| w.is_finite() && *w > 0.0) {
        Some(width) => width / area.width,
        None => (MAX_SIDE / area.width.max(area.height)).min(1.0),
    };
    let (w, h) = ((area.width * scale) as f32, (area.height * scale) as f32);
    let mut body = String::new();
    if !scene.transparent {
        let _ = write!(
            body,
            r#"<rect x="{}" y="{}" width="{}" height="{}" fill="{}"/>"#,
            n(area.x),
            n(area.y),
            n(area.width),
            n(area.height),
            palette.background.hex()
        );
    }
    let nodes: Vec<&Node> = canvas.live_nodes().collect();
    let by_id: HashMap<&str, &Node> = nodes.iter().map(|n| (n.id.as_str(), *n)).collect();
    // Groups sit under everything, then arrows, cards and ink in stacking order.
    for node in nodes.iter().filter(|n| n.kind == "group") {
        if overlaps(&node.bounds(), &area) {
            group(node, palette, &mut body);
        }
    }
    let mut edges: Vec<&Edge> = canvas.edges.iter().filter(|e| !e.deleted).collect();
    edges.sort_by(|a, b| {
        a.kanso_z
            .total_cmp(&b.kanso_z)
            .then_with(|| a.id.cmp(&b.id))
    });
    for edge in edges {
        if let (Some(a), Some(b)) = (
            by_id.get(edge.from_node.as_str()),
            by_id.get(edge.to_node.as_str()),
        ) {
            arrow(edge, a, b, palette, &mut body);
        }
    }
    for node in nodes.iter().filter(|n| n.kind != "group") {
        if !overlaps(&node.bounds(), &area) {
            continue;
        }
        if node.kind == INK {
            ink(node, palette, &mut body);
        } else {
            card(node, scene.titles, palette, &mut body);
        }
    }
    let markup = format!(
        r#"<svg xmlns="http://www.w3.org/2000/svg" viewBox="{} {} {} {}" width="{}" height="{}">{body}</svg>"#,
        n(area.x),
        n(area.y),
        n(area.width),
        n(area.height),
        svg::number(w),
        svg::number(h),
    );
    Ok(Svg {
        markup,
        width: w,
        height: h,
        baseline: None,
        notices: Vec::new(),
    })
}

fn n(v: f64) -> String {
    svg::number(v as f32)
}

fn overlaps(a: &Bounds, b: &Bounds) -> bool {
    a.x <= b.right() && a.right() >= b.x && a.y <= b.bottom() && a.bottom() >= b.y
}

fn group(node: &Node, palette: &Palette, out: &mut String) {
    let color = card_color(node.color.as_deref()).unwrap_or_else(|| palette.muted.hex());
    let _ = write!(
        out,
        r#"<rect x="{}" y="{}" width="{}" height="{}" rx="8" fill="{color}" fill-opacity="0.07" stroke="{color}" stroke-opacity="0.7" stroke-width="1.5"/>"#,
        n(node.x),
        n(node.y),
        n(node.width),
        n(node.height)
    );
    if let Some(label) = node.label.as_deref().filter(|l| !l.is_empty()) {
        let _ = write!(
            out,
            r#"<text x="{}" y="{}" font-size="13" fill="{}">{}</text>"#,
            n(node.x + 10.0),
            n(node.y - 6.0),
            palette.muted.hex(),
            escape(label)
        );
    }
}

fn card(
    node: &Node,
    titles: Option<&HashMap<String, String>>,
    palette: &Palette,
    out: &mut String,
) {
    let color = card_color(node.color.as_deref());
    let border = color.clone().unwrap_or_else(|| palette.border.hex());
    let fill = match &color {
        Some(c) => crate::Rgb(u32::from_str_radix(&c[1..], 16).unwrap_or(0))
            .over(palette.background, 0.14)
            .hex(),
        None => palette.surface.hex(),
    };
    let _ = write!(
        out,
        r#"<rect x="{}" y="{}" width="{}" height="{}" rx="8" fill="{fill}" stroke="{border}" stroke-width="1.5"/>"#,
        n(node.x),
        n(node.y),
        n(node.width),
        n(node.height)
    );
    let (heading, body) = match node.kind.as_str() {
        "file" | "link" => {
            let live = Canvas::note_id(node)
                .and_then(|id| {
                    titles.and_then(|t| {
                        // By the whole ID, or without its `note:` prefix.
                        t.get(id)
                            .or_else(|| t.get(id.strip_prefix(kanso_core::canvas::NOTE_PREFIX)?))
                    })
                })
                .map(String::as_str);
            let title = live
                .or(node.title.as_deref())
                .or(node.url.as_deref())
                .or(node.file.as_deref())
                .unwrap_or("");
            (Some(title.to_string()), String::new())
        }
        _ => (None, node.text.clone().unwrap_or_default()),
    };
    let max_chars = (((node.width - 24.0) / (FONT * 0.55)).floor() as usize).max(4);
    let max_lines = (((node.height - 16.0) / (FONT * 1.35)).floor() as usize).clamp(1, MAX_LINES);
    let text = heading.clone().unwrap_or(body);
    let lines = wrap(&text, max_chars, max_lines);
    if lines.is_empty() {
        return;
    }
    let weight = if heading.is_some() { "600" } else { "400" };
    let _ = write!(
        out,
        r#"<text font-size="{FONT}" font-weight="{weight}" fill="{}" xml:space="preserve">"#,
        palette.text.hex()
    );
    for (i, line) in lines.iter().enumerate() {
        let _ = write!(
            out,
            r#"<tspan x="{}" y="{}">{}</tspan>"#,
            n(node.x + 12.0),
            n(node.y + 12.0 + FONT + i as f64 * FONT * 1.35 - 3.0),
            escape(line)
        );
    }
    out.push_str("</text>");
}

/// Wraps `text` to `width` characters and at most `lines` lines, ending a cut
/// text with an ellipsis. Pure.
pub fn wrap(text: &str, width: usize, lines: usize) -> Vec<String> {
    let mut out: Vec<String> = Vec::new();
    for raw in text.lines() {
        let raw = raw.trim_end();
        let raw = raw.trim_start_matches('#').trim_start();
        if raw.is_empty() {
            out.push(String::new());
            continue;
        }
        let mut line = String::new();
        for word in raw.split_whitespace() {
            let len = line.chars().count();
            let wlen = word.chars().count();
            if len > 0 && len + 1 + wlen > width {
                out.push(std::mem::take(&mut line));
            }
            if wlen > width {
                // A word longer than the line is broken where it falls.
                let chars: Vec<char> = word.chars().collect();
                for chunk in chars.chunks(width) {
                    if !line.is_empty() {
                        out.push(std::mem::take(&mut line));
                    }
                    line = chunk.iter().collect();
                }
                continue;
            }
            if !line.is_empty() {
                line.push(' ');
            }
            line.push_str(word);
        }
        out.push(line);
        if out.len() > lines {
            break;
        }
    }
    if out.len() > lines {
        out.truncate(lines);
        if let Some(last) = out.last_mut() {
            let keep = width.saturating_sub(1);
            *last = last.chars().take(keep).collect::<String>() + "\u{2026}";
        }
    }
    while out.last().is_some_and(String::is_empty) {
        out.pop();
    }
    out
}

/// Where an arrow leaves or reaches `node`: the middle of the side asked
/// for, else the side facing `toward`.
fn anchor(node: &Node, side: Option<&str>, toward: (f64, f64)) -> ((f64, f64), (f64, f64)) {
    let b = node.bounds();
    let (cx, cy) = (b.x + b.width / 2.0, b.y + b.height / 2.0);
    let side = side.map(str::to_owned).unwrap_or_else(|| {
        let (dx, dy) = (toward.0 - cx, toward.1 - cy);
        if dx.abs() * b.height > dy.abs() * b.width {
            if dx > 0.0 { "right" } else { "left" }
        } else if dy > 0.0 {
            "bottom"
        } else {
            "top"
        }
        .to_string()
    });
    match side.as_str() {
        "top" => ((cx, b.y), (0.0, -1.0)),
        "bottom" => ((cx, b.bottom()), (0.0, 1.0)),
        "left" => ((b.x, cy), (-1.0, 0.0)),
        _ => ((b.right(), cy), (1.0, 0.0)),
    }
}

fn arrow(edge: &Edge, from: &Node, to: &Node, palette: &Palette, out: &mut String) {
    let (fb, tb) = (from.bounds(), to.bounds());
    let (fc, tc) = (
        (fb.x + fb.width / 2.0, fb.y + fb.height / 2.0),
        (tb.x + tb.width / 2.0, tb.y + tb.height / 2.0),
    );
    let (a, da) = anchor(from, edge.from_side.as_deref(), tc);
    let (b, db) = anchor(to, edge.to_side.as_deref(), fc);
    let reach = ((a.0 - b.0).hypot(a.1 - b.1) / 3.0).clamp(20.0, 160.0);
    let (c1, c2) = (
        (a.0 + da.0 * reach, a.1 + da.1 * reach),
        (b.0 + db.0 * reach, b.1 + db.1 * reach),
    );
    let color = card_color(edge.color.as_deref()).unwrap_or_else(|| palette.muted.hex());
    let _ = write!(
        out,
        r#"<path d="M{} {}C{} {} {} {} {} {}" fill="none" stroke="{color}" stroke-width="2" stroke-linecap="round"/>"#,
        n(a.0),
        n(a.1),
        n(c1.0),
        n(c1.1),
        n(c2.0),
        n(c2.1),
        n(b.0),
        n(b.1)
    );
    // Arrowheads: the JSON Canvas default is an arrow at the far end. The
    // head's base sits out from the card along the side's normal.
    let heads = [
        (edge.from_end.as_deref() == Some("arrow"), a, da),
        (edge.to_end.as_deref().unwrap_or("arrow") == "arrow", b, db),
    ];
    for (on, tip, normal) in heads {
        if on {
            let (px, py) = (-normal.1, normal.0);
            let (bx, by) = (tip.0 + normal.0 * 12.0, tip.1 + normal.1 * 12.0);
            let _ = write!(
                out,
                r#"<path d="M{} {}L{} {}L{} {}Z" fill="{color}"/>"#,
                n(tip.0),
                n(tip.1),
                n(bx + px * 6.0),
                n(by + py * 6.0),
                n(bx - px * 6.0),
                n(by - py * 6.0),
            );
        }
    }
    if let Some(label) = edge.label.as_deref().filter(|l| !l.is_empty()) {
        let mid = ((a.0 + b.0) / 2.0, (a.1 + b.1) / 2.0);
        let _ = write!(
            out,
            r#"<text x="{}" y="{}" font-size="13" text-anchor="middle" fill="{}">{}</text>"#,
            n(mid.0),
            n(mid.1 - 6.0),
            palette.muted.hex(),
            escape(label)
        );
    }
}

fn ink(node: &Node, palette: &Palette, out: &mut String) {
    let mut strokes: Vec<&Stroke> = node.strokes.iter().filter(|s| !s.deleted).collect();
    strokes.sort_by(|a, b| {
        a.kanso_z
            .total_cmp(&b.kanso_z)
            .then_with(|| a.id.cmp(&b.id))
    });
    if strokes.is_empty() {
        return;
    }
    let _ = write!(
        out,
        r#"<g transform="translate({} {})">"#,
        n(node.x),
        n(node.y)
    );
    for s in strokes {
        let d = freehand::path_data(&pen::outline(s, true));
        if d.is_empty() {
            continue;
        }
        let _ = write!(
            out,
            r#"<path d="{d}" fill="{}" fill-opacity="{}"/>"#,
            pen::ink_color(&s.color, palette),
            pen::opacity(&s.tool)
        );
    }
    out.push_str("</g>");
}

#[cfg(test)]
mod tests;
