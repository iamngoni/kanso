//! Inline sketches: a fixed-size drawing area stored as a `sketch` fence in a
//! Markdown note's body. The JSON is the canvas document with only ink nodes,
//! plus `kansoSketch` (the box) and `kansoSketchId` (its identity in the note,
//! because a note can hold several and the text around them moves).

use std::collections::{HashMap, HashSet};
use std::ops::Range;

use super::edit::unique_id;
use super::model::{Canvas, INK, Node, Sketch};
use super::{Budget, FENCE as CANVAS_FENCE, NOTE_PREFIX};

/// The fence's info string.
pub const FENCE: &str = "sketch";

/// One sketch's JSON may be this long, in bytes (about 10,000 points).
pub const MAX_SKETCH_BYTES: usize = 256 * 1024;

/// All of one note's sketches together.
pub const MAX_NOTE_SKETCH_BYTES: usize = 512 * 1024;

/// A sketch this long (75% of its cap) warns that it is nearly full.
pub const WARN_SKETCH_BYTES: usize = MAX_SKETCH_BYTES / 4 * 3;

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

/// The sketch in a fence's JSON, when it is one of Kanso's (it has its box).
fn read(json: &str) -> Option<Canvas> {
    serde_json::from_str::<Canvas>(json)
        .ok()
        .filter(|c| c.kanso_sketch.is_some())
}

/// The size caps: each sketch, and all of a note's together. Pure.
pub fn check_budget(body: &str) -> Result<(), String> {
    let mut total = 0;
    for found in find(body) {
        // A `sketch` code block that isn't Kanso's (an Arduino sketch, say)
        // is the note's own text, with no cap of its own.
        if read(&body[found.json.clone()]).is_none() {
            continue;
        }
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
        match read(&body[f.json.clone()]) {
            Some(canvas) => {
                let words = canvas.search_text();
                if !words.is_empty() {
                    out.push_str(&words);
                    out.push('\n');
                }
            }
            // Someone else's `sketch` block is searched like any code.
            None => out.push_str(&body[f.range.clone()]),
        }
        from = f.range.end;
    }
    out.push_str(&body[from..]);
    out
}

/// Why a sketch edit wasn't made.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Refusal {
    /// No fence in the note has that ID: the note changed underneath.
    Missing,
    /// The edit would pass a size cap; the words to show.
    Full(String),
}

impl Refusal {
    /// The words to show.
    pub fn words(&self) -> &str {
        match self {
            Refusal::Missing => "This sketch is gone from the note, so nothing was changed.",
            Refusal::Full(words) => words,
        }
    }
}

/// What "this sketch is full" says.
pub const SKETCH_FULL: &str = "This sketch is full. Open it as a canvas to keep drawing.";

/// What a note at its sketch cap says to "Insert sketch".
pub const NOTE_FULL: &str = "This note's sketches are full. Open one as a canvas.";

/// The sketch's JSON as the fence holds it. Pure.
pub fn to_json(sketch: &Canvas) -> String {
    serde_json::to_string(sketch).unwrap_or_else(|_| "{}".into())
}

/// The whole fence for `sketch`. Pure.
pub fn to_fence(sketch: &Canvas) -> String {
    format!("```{FENCE}\n{}\n```\n", to_json(sketch))
}

/// Whether `sketch` is within its own cap. Pure.
pub fn fits(sketch: &Canvas) -> bool {
    to_json(sketch).len() <= MAX_SKETCH_BYTES
}

/// How a sketch whose JSON is `bytes` long stands against its cap: a warning
/// from 75%, full at the cap. Pure.
pub fn budget(bytes: usize) -> Budget {
    if bytes >= MAX_SKETCH_BYTES {
        Budget::Full
    } else if bytes >= WARN_SKETCH_BYTES {
        Budget::Warn
    } else {
        Budget::Ok
    }
}

/// What a sketch past 75% of its cap says.
pub const SKETCH_NEARLY_FULL: &str =
    "This sketch is nearly full. Open it as a canvas to keep drawing without a limit.";

/// Whether `after` holds a Kanso sketch that `before` didn't: a new ID, or
/// one more fence with an ID it had (a pasted copy). Moving or editing a
/// sketch adds none. A shared note refuses an edit that adds one. Pure.
pub fn adds_sketch(before: &str, after: &str) -> bool {
    let count = |body: &str| {
        let mut ids: HashMap<String, usize> = HashMap::new();
        for (_, canvas) in sketches(body) {
            *ids.entry(canvas.kanso_sketch_id.unwrap_or_default())
                .or_default() += 1;
        }
        ids
    };
    let (was, now) = (count(before), count(after));
    now.iter()
        .any(|(id, n)| *n > was.get(id).copied().unwrap_or(0))
}

/// [`replace_checked`] for `mine`, a sketch drawn from `base`, keeping what
/// another writer did to the same sketch meanwhile (another device through
/// sync, or the handwriting reader): when the note's fence is no longer
/// `base`, the two are merged, every stroke and erasure of both kept, and the
/// merge is written. Comes back with the note's text and the sketch written.
/// Pure.
pub fn replace_merged(
    body: &str,
    id: &str,
    mine: &Canvas,
    base: &Canvas,
) -> Result<(String, Canvas), Refusal> {
    let (_, current) = find_by_id(body, id).ok_or(Refusal::Missing)?;
    let next = if current == *base || current == *mine {
        mine.clone()
    } else {
        let mut merged = super::merge(mine, &current);
        // The box and the ID are the sketch's own.
        merged.kanso_sketch = mine.kanso_sketch;
        merged.kanso_sketch_id = mine.kanso_sketch_id.clone();
        merged
    };
    let out = replace_checked(body, id, &next)?;
    Ok((out, next))
}

impl Sketch {
    /// Whether the point is inside the box. Anything outside is ignored,
    /// which also keeps a stray pen from growing the note. Pure.
    pub fn contains(&self, x: f64, y: f64) -> bool {
        x.is_finite()
            && y.is_finite()
            && x >= 0.0
            && y >= 0.0
            && x <= f64::from(self.w)
            && y <= f64::from(self.h)
    }
}

/// An ID for a new sketch that no sketch in `body` has. Random.
pub fn fresh_id(body: &str) -> String {
    let taken: HashSet<String> = find(body)
        .iter()
        .filter_map(|f| serde_json::from_str::<Canvas>(&body[f.json.clone()]).ok())
        .filter_map(|c| c.kanso_sketch_id)
        .collect();
    unique_id("sk-", &taken)
}

/// The bytes of Kanso sketch JSON `body` holds, all its sketches together.
/// Pure.
pub fn used_bytes(body: &str) -> usize {
    find(body)
        .into_iter()
        .filter(|f| read(&body[f.json.clone()]).is_some())
        .map(|f| f.json.len())
        .sum()
}

/// `body` with a new empty `w` by `h` sketch inserted at byte `at` on lines
/// of its own, and its ID; or why not. A note at its sketch cap refuses (the
/// app offers to open a sketch as a canvas). Random in the ID.
pub fn insert(body: &str, at: usize, w: u32, h: u32) -> Result<(String, String), Refusal> {
    let id = fresh_id(body);
    let fence = to_fence(&new_sketch(&id, w, h));
    if used_bytes(body) + fence.len() > MAX_NOTE_SKETCH_BYTES {
        return Err(Refusal::Full(NOTE_FULL.into()));
    }
    let mut at = at.min(body.len());
    while !body.is_char_boundary(at) {
        at -= 1;
    }
    let (before, after) = body.split_at(at);
    let mut out = String::with_capacity(body.len() + fence.len() + 4);
    out.push_str(before);
    if !before.is_empty() && !before.ends_with("\n\n") {
        out.push_str(if before.ends_with('\n') { "\n" } else { "\n\n" });
    }
    out.push_str(&fence);
    if !after.is_empty() && !after.starts_with('\n') {
        out.push('\n');
    }
    out.push_str(after);
    Ok((id, out))
}

/// [`replace`], refusing before the write when the sketch or the note would
/// pass its cap, so a sketch can never push the note past its own hard cap.
/// Pure.
pub fn replace_checked(body: &str, id: &str, sketch: &Canvas) -> Result<String, Refusal> {
    if !fits(sketch) {
        return Err(Refusal::Full(SKETCH_FULL.into()));
    }
    let out = replace(body, id, sketch).ok_or(Refusal::Missing)?;
    // Only a note that grows can be refused: shrinking is always allowed.
    let after = used_bytes(&out);
    if after > MAX_NOTE_SKETCH_BYTES && after > used_bytes(body) {
        return Err(Refusal::Full(NOTE_FULL.into()));
    }
    Ok(out)
}

/// The `canvas` embed that takes a promoted sketch's place. Pure.
pub fn embed(note_id: &str, width: u32) -> String {
    format!("```{CANVAS_FENCE}\n{NOTE_PREFIX}{note_id}\nw:{width}\n```\n")
}

impl Canvas {
    /// The canvas document a sketch becomes when opened as a canvas: the same
    /// ink, the same recognized handwriting, no sketch box or ID, now free to
    /// grow and take cards. Pure.
    pub fn promoted(&self) -> Canvas {
        let mut canvas = self.clone();
        canvas.kanso_sketch = None;
        canvas.kanso_sketch_id = None;
        canvas
    }
}

/// What promoting sketch `id` yields: the new canvas's document and the
/// note's body with the fence replaced by an embed of `note_id`, one edit. A
/// missing ID is `None`, and nothing is promoted. Pure.
pub fn promote(body: &str, id: &str, note_id: &str) -> Option<(Canvas, String)> {
    let (_, sketch) = find_by_id(body, id)?;
    let width = sketch.kanso_sketch.map_or(480, |s| s.w);
    let next = replace_with_text(body, id, &embed(note_id, width))?;
    Some((sketch.promoted(), next))
}

/// The sketches in `body` that are Kanso's, with where each is. Pure.
pub fn sketches(body: &str) -> Vec<(Found, Canvas)> {
    find(body)
        .into_iter()
        .filter_map(|f| read(&body[f.json.clone()]).map(|c| (f, c)))
        .collect()
}
