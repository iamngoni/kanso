//! Inline sketches: a fixed-size drawing area stored as a `sketch` fence in a
//! Markdown note's body. The JSON is the canvas document with only ink nodes,
//! plus `kansoSketch` (the box) and `kansoSketchId` (its identity in the note,
//! because a note can hold several and the text around them moves).

use std::ops::Range;

use super::model::{Canvas, INK, Node, Sketch};

/// The fence's info string.
pub const FENCE: &str = "sketch";

/// One sketch's JSON may be this long, in bytes (about 10,000 points).
pub const MAX_SKETCH_BYTES: usize = 256 * 1024;

/// All of one note's sketches together.
pub const MAX_NOTE_SKETCH_BYTES: usize = 512 * 1024;

/// The sizes "Insert sketch" offers, in points.
pub const SIZES: &[(u32, u32)] = &[(320, 240), (480, 320), (640, 480)];

/// A `sketch` fence found in a note's body.
#[derive(Clone, Debug, PartialEq)]
pub struct Found {
    /// The whole fence, both fence lines included.
    pub range: Range<usize>,
    /// The JSON between them.
    pub json: Range<usize>,
}

/// An empty sketch `w` by `h` points with one ink layer, `id` in its note.
/// Pure.
pub fn new_sketch(id: &str, w: u32, h: u32) -> Canvas {
    Canvas {
        nodes: vec![Node {
            id: "i1".into(),
            kind: INK.into(),
            width: f64::from(w),
            height: f64::from(h),
            kanso_version: 1,
            ..Node::default()
        }],
        kanso_sketch: Some(Sketch { w, h, v: 1 }),
        kanso_sketch_id: Some(id.to_owned()),
        ..Canvas::default()
    }
}

impl Canvas {
    /// Checks what an inline sketch must be on top of a canvas: a box, ink
    /// nodes only, and no arrows.
    pub fn validate_sketch(&self) -> Result<(), String> {
        self.validate()?;
        let Some(sketch) = self.kanso_sketch else {
            return Err("A sketch needs its size.".into());
        };
        if !(32..=4096).contains(&sketch.w) || !(32..=4096).contains(&sketch.h) {
            return Err("A sketch's size is out of range.".into());
        }
        if self.kanso_sketch_id.as_deref().is_none_or(str::is_empty) {
            return Err("A sketch needs an ID.".into());
        }
        if self.nodes.iter().any(|n| !n.deleted && n.kind != INK) {
            return Err("A sketch holds ink only, no cards or groups.".into());
        }
        if self.edges.iter().any(|e| !e.deleted) {
            return Err("A sketch has no arrows.".into());
        }
        Ok(())
    }
}

/// The `sketch` fences in `body`, in order. A fence nested in another code
/// block isn't one. Pure.
pub fn find(body: &str) -> Vec<Found> {
    let mut found = Vec::new();
    let mut offset = 0;
    let mut open: Option<(usize, usize)> = None;
    let mut other_fence: Option<usize> = None;
    for line in body.split_inclusive('\n') {
        let start = offset;
        offset += line.len();
        let text = line.trim_end();
        match open {
            Some((from, json_start)) => {
                if text == "```" {
                    found.push(Found {
                        range: from..offset,
                        json: json_start..start,
                    });
                    open = None;
                }
            }
            None if other_fence.is_some() => {
                let ticks = text.len() - text.trim_start_matches('`').len();
                if ticks >= other_fence.unwrap_or(3)
                    && text.trim_start_matches('`').trim().is_empty()
                {
                    other_fence = None;
                }
            }
            None if text == "```sketch" => open = Some((start, offset)),
            None if text.starts_with("```") => {
                other_fence = Some(text.len() - text.trim_start_matches('`').len());
            }
            None => {}
        }
    }
    found
}

/// The sketch `id` in `body`: where it is and what it holds. Pure.
pub fn find_by_id(body: &str, id: &str) -> Option<(Found, Canvas)> {
    find(body).into_iter().find_map(|f| {
        let canvas: Canvas = serde_json::from_str(&body[f.json.clone()]).ok()?;
        (canvas.kanso_sketch_id.as_deref() == Some(id)).then_some((f, canvas))
    })
}

/// `body` with sketch `id` replaced by `sketch`, or `None` when no fence in
/// the note has that ID (the note changed underneath). Never touches another
/// sketch. Pure.
pub fn replace(body: &str, id: &str, sketch: &Canvas) -> Option<String> {
    let (found, _) = find_by_id(body, id)?;
    let json = serde_json::to_string(sketch).ok()?;
    let mut out = String::with_capacity(body.len() + json.len());
    out.push_str(&body[..found.range.start]);
    out.push_str(&format!("```{FENCE}\n{json}\n```\n"));
    out.push_str(&body[found.range.end..]);
    Some(out)
}

/// `body` with sketch `id` replaced by `replacement` (any text, for example a
/// `canvas` embed). The one edit "Open as canvas" makes. Pure.
pub fn replace_with_text(body: &str, id: &str, replacement: &str) -> Option<String> {
    let (found, _) = find_by_id(body, id)?;
    let mut out = String::with_capacity(body.len() + replacement.len());
    out.push_str(&body[..found.range.start]);
    out.push_str(replacement);
    if !replacement.ends_with('\n') {
        out.push('\n');
    }
    out.push_str(&body[found.range.end..]);
    Some(out)
}

/// The size caps: each sketch, and all of a note's together. Pure.
pub fn check_budget(body: &str) -> Result<(), String> {
    let mut total = 0;
    for found in find(body) {
        let len = found.json.len();
        if len > MAX_SKETCH_BYTES {
            return Err("This sketch is full. Open it as a canvas to keep drawing.".into());
        }
        total += len;
    }
    if total > MAX_NOTE_SKETCH_BYTES {
        return Err("This note's sketches are full. Open one as a canvas.".into());
    }
    Ok(())
}

/// A Markdown note's text for the search index: each sketch fence is replaced
/// by the handwriting recognized in it (or nothing), so a sketch's numbers
/// never pollute the index. Pure.
pub fn index_text(body: &str) -> String {
    let found = find(body);
    if found.is_empty() {
        return body.to_owned();
    }
    let mut out = String::with_capacity(body.len());
    let mut from = 0;
    for f in found {
        out.push_str(&body[from..f.range.start]);
        if let Ok(canvas) = serde_json::from_str::<Canvas>(&body[f.json]) {
            let words = canvas.search_text();
            if !words.is_empty() {
                out.push_str(&words);
                out.push('\n');
            }
        }
        from = f.range.end;
    }
    out.push_str(&body[from..]);
    out
}
