//! Canvases on the phone (#45): a canvas note opened for drawing.
//!
//! A [`CanvasSession`] holds the document in Rust. The shell paints the
//! [`CanvasView`] it returns (ink as ready-made outlines from the shared
//! `kanso_render` pen, so a stroke looks as it does on the desktop) and
//! sends back pen samples and edits; every edit is a stamped operation of
//! `kanso_core::canvas`, so the private-sync merge treats it like any other.
//!
//! Saving is deferred: edits mark the session dirty and nothing is written
//! until the shell calls [`CanvasSession::save`] (when a stroke ends and
//! after two seconds without input), never per point.
//!
//! A locked canvas is never returned while its lock is closed
//! ([`KansoError::Locked`]); an open one is sealed again on every save. A
//! canvas's text is JSON, so none of the note-text rules run on it (no
//! inline `#tags`).

use std::collections::{HashMap, HashSet};
use std::sync::{Arc, Mutex};
use std::time::Instant;

use kanso_core::canvas::{
    self, Action, Bounds, Budget, Canvas, Edge, INK, Node, Point, Refused, Stroke, StrokeRef,
};
use kanso_core::{DocType, Note, new_id, now_ms};
use kanso_render::Palette;
use kanso_render::canvas::{self as draw, geometry, pen};
use kanso_store::{Origin, Put};
use serde_json::json;

use crate::boundary::{guard, held};
use crate::canvas_model::{
    CanvasArrow, CanvasBudget, CanvasCard, CanvasGroup, CanvasPoint, CanvasRect, CanvasSaved,
    CanvasStroke, CanvasView, InkStyle, PenSample, StrokeKey,
};
use crate::errors::KansoError;
use crate::library::Library;
use crate::model::{NewNote, Saved};

/// The most undo steps kept.
const UNDO_LIMIT: usize = 60;
/// A new text card's size.
const CARD_W: f64 = 240.0;
const CARD_H: f64 = 120.0;

/// One undoable step.
#[derive(Clone, Debug)]
enum Entry {
    Strokes(Action),
    Node {
        before: Option<Node>,
        after: Option<Node>,
    },
    Edge {
        before: Option<Edge>,
        after: Option<Edge>,
    },
}

struct State {
    id: String,
    rev: String,
    title: String,
    canvas: Canvas,
    undo: Vec<Entry>,
    redo: Vec<Entry>,
    dirty: bool,
}

/// A canvas open for drawing. Cheap to clone; clones share the session.
#[derive(Clone)]
pub struct CanvasSession {
    lib: Library,
    state: Arc<Mutex<State>>,
}

/// The outline of a stroke still being drawn, as `x0, y0, x1, y1, ...`: what
/// the shell paints under the pen each frame. Nothing is stored. Pure.
pub fn ink_preview(style: InkStyle, samples: Vec<PenSample>) -> Vec<f64> {
    let stroke = Stroke {
        tool: style.tool,
        color: style.color,
        width: style.width,
        points: samples.iter().map(point).collect(),
        ..Stroke::default()
    };
    pen::outline(&stroke, false)
        .into_iter()
        .flat_map(|p| [p[0], p[1]])
        .collect()
}

impl Library {
    /// Opens canvas note `id` for drawing. [`KansoError::Locked`] while it is
    /// locked and closed, [`KansoError::Invalid`] for a note that isn't a
    /// canvas.
    pub fn canvas(&self, id: &str) -> Result<CanvasSession, KansoError> {
        guard(|| {
            let (note, canvas) = self.read_canvas(id)?;
            Ok(CanvasSession {
                lib: self.clone(),
                state: Arc::new(Mutex::new(State {
                    id: note.id,
                    rev: note.rev.unwrap_or_default(),
                    title: note.title,
                    canvas,
                    undo: Vec::new(),
                    redo: Vec::new(),
                    dirty: false,
                })),
            })
        })
    }

    /// Whether note `id` is a canvas. Reads nothing of a locked one.
    pub fn is_canvas(&self, id: &str) -> bool {
        self.stored(id).is_ok_and(|note| note.doctype == "canvas")
    }

    /// Makes an empty canvas note in `notebook_id` (else the preferred
    /// notebook, else the first). A locked notebook refuses
    /// ([`KansoError::Locked`]).
    pub fn create_canvas(
        &self,
        notebook_id: Option<String>,
        preferred_notebook_id: Option<String>,
        title: String,
    ) -> Result<Saved, KansoError> {
        guard(|| {
            self.write_new_canvas(
                notebook_id,
                preferred_notebook_id,
                title,
                &Canvas::default(),
                Vec::new(),
            )
        })
    }

    /// Makes a canvas note from an Excalidraw scene (a file or the
    /// clipboard): freehand strokes keep their pressure.
    pub fn import_excalidraw(
        &self,
        source: String,
        notebook_id: Option<String>,
        preferred_notebook_id: Option<String>,
        title: String,
    ) -> Result<Saved, KansoError> {
        guard(|| {
            let canvas = draw::excalidraw::import(&source, now_ms())
                .map_err(|failure| KansoError::invalid(failure.badge()))?;
            self.write_new_canvas(
                notebook_id,
                preferred_notebook_id,
                title,
                &canvas,
                Vec::new(),
            )
        })
    }

    /// The canvas note `id` and its document. A closed lock refuses.
    fn read_canvas(&self, id: &str) -> Result<(Note, Canvas), KansoError> {
        let mut note = self.stored(id)?;
        if note.doctype != "canvas" {
            return Err(KansoError::invalid("That note isn't a canvas."));
        }
        let state = self.reveal(&mut note);
        if state.is_closed() {
            return Err(KansoError::Locked);
        }
        let canvas = canvas::parse(&note.body).map_err(KansoError::invalid)?;
        Ok((note, canvas))
    }

    fn write_new_canvas(
        &self,
        notebook_id: Option<String>,
        preferred_notebook_id: Option<String>,
        title: String,
        canvas: &Canvas,
        tags: Vec<String>,
    ) -> Result<Saved, KansoError> {
        let (book_id, _, _) = self.destination(&NewNote {
            notebook_id,
            title: title.clone(),
            body: String::new(),
            place: None,
            preferred_notebook_id,
            tags: Vec::new(),
            pinned: false,
        })?;
        if self.inner.store.locked_books()?.contains(&book_id) {
            return Err(KansoError::Locked);
        }
        let now = now_ms();
        let note: Note = serde_json::from_value(json!({
            "_id": new_id(DocType::Note),
            "title": title,
            "body": canvas::to_body(canvas),
            "doctype": "canvas",
            "bookId": book_id,
            "tags": tags,
            "pinned": false,
            "createdAt": now,
            "updatedAt": now,
        }))
        .map_err(|err| KansoError::invalid(err.to_string()))?;
        let value =
            serde_json::to_value(&note).map_err(|err| KansoError::storage(err.to_string()))?;
        let written = self
            .inner
            .store
            .put(DocType::Note, value, Put::Upsert, Origin::Ui)?;
        self.inner
            .metrics()
            .count(crate::metrics::Feature::NoteCreated);
        Ok(Saved {
            id: written.id,
            rev: written.rev,
        })
    }

    /// Writes `canvas` into note `id` if it is still at `base_rev`. A locked
    /// note that is open is sealed again.
    fn write_canvas(
        &self,
        id: &str,
        base_rev: &str,
        canvas: &Canvas,
    ) -> Result<(String, usize), KansoError> {
        let mut note = self.stored(id)?;
        let mut shown = note.clone();
        let state = self.reveal(&mut shown);
        if shown.rev != note.rev {
            note = self.stored(id)?;
        }
        if note.rev.as_deref() != Some(base_rev) {
            return Err(KansoError::Conflict {
                current_rev: note.rev.unwrap_or_default(),
            });
        }
        if state.is_closed() {
            return Err(KansoError::Locked);
        }
        note.body = canvas::to_body(canvas);
        let bytes = note.body.len();
        note.updated_at = now_ms();
        if state == kanso_engine::lock::LockView::Open {
            held(&self.inner.keys)
                .seal(&mut note, Instant::now())
                .map_err(|_| KansoError::Locked)?;
        }
        let value =
            serde_json::to_value(&note).map_err(|err| KansoError::storage(err.to_string()))?;
        match self
            .inner
            .store
            .put(DocType::Note, value, Put::Upsert, Origin::Ui)
        {
            Ok(written) => Ok((written.rev, bytes)),
            Err(kanso_store::StoreError::Conflict) => Err(KansoError::Conflict {
                current_rev: self.stored(id).ok().and_then(|n| n.rev).unwrap_or_default(),
            }),
            Err(err) => Err(err.into()),
        }
    }
}

impl CanvasSession {
    fn lock(&self) -> std::sync::MutexGuard<'_, State> {
        held(&self.state)
    }

    /// The canvas as the shell paints it. Outlines are built only for strokes
    /// whose ID is not in `known`; the others come back with `known` set and
    /// no outline, for the shell to reuse what it holds.
    pub fn view(&self, known: Vec<String>) -> Result<CanvasView, KansoError> {
        guard(|| {
            let titles = self.titles();
            Ok(view(&self.lock(), &titles, &known.into_iter().collect()))
        })
    }

    /// Draws a stroke from pen samples (canvas coordinates). A stroke with
    /// no usable sample draws nothing; a full canvas refuses
    /// ([`KansoError::Invalid`], with the words to show).
    pub fn draw(
        &self,
        style: InkStyle,
        samples: Vec<PenSample>,
        known: Vec<String>,
    ) -> Result<CanvasView, KansoError> {
        guard(|| {
            let mut state = self.lock();
            let stroke = Stroke {
                tool: style.tool,
                color: style.color,
                width: style.width,
                points: samples.iter().map(point).collect(),
                ..Stroke::default()
            };
            match state.canvas.draw(stroke, now_ms()) {
                Ok((_, action)) => state.push(Entry::Strokes(action)),
                Err(Refused::Empty) => {}
                Err(Refused::Full) => {
                    return Err(KansoError::invalid(
                        "This canvas is full. Move some ink to a new canvas to keep drawing.",
                    ));
                }
                Err(Refused::TooLong) => {
                    return Err(KansoError::invalid("That stroke is too long."));
                }
            }
            Ok(view(&state, &self.titles(), &known.into_iter().collect()))
        })
    }

    /// The stroke eraser: removes every whole stroke under the disc.
    pub fn erase_at(
        &self,
        x: f64,
        y: f64,
        radius: f64,
        known: Vec<String>,
    ) -> Result<CanvasView, KansoError> {
        self.edit(known, |c| {
            c.erase_strokes_at(x, y, radius, now_ms())
                .map(Entry::Strokes)
        })
    }

    /// The stroke eraser dragged along `path`: removes every whole stroke the
    /// disc touches on the way, as one step for undo.
    pub fn erase_strokes_along(
        &self,
        path: Vec<CanvasPoint>,
        radius: f64,
        known: Vec<String>,
    ) -> Result<CanvasView, KansoError> {
        self.edit(known, |c| {
            let mut hit: Vec<StrokeRef> = Vec::new();
            let step = (radius / 2.0).max(1.0);
            let mut probe = |x: f64, y: f64| {
                for r in c.strokes_at(x, y, radius) {
                    if !hit.contains(&r) {
                        hit.push(r);
                    }
                }
            };
            for pair in path.windows(2) {
                let (a, b) = (pair[0], pair[1]);
                let steps = ((b.x - a.x).hypot(b.y - a.y) / step).ceil().max(1.0) as u32;
                for i in 0..=steps {
                    let t = f64::from(i) / f64::from(steps);
                    probe(a.x + (b.x - a.x) * t, a.y + (b.y - a.y) * t);
                }
            }
            if let [only] = path.as_slice() {
                probe(only.x, only.y);
            }
            c.delete_strokes(&hit, now_ms()).map(Entry::Strokes)
        })
    }

    /// The partial eraser: rubs out what the disc swept along `path` touches,
    /// leaving the rest of each stroke as new strokes.
    pub fn erase_along(
        &self,
        path: Vec<CanvasPoint>,
        radius: f64,
        known: Vec<String>,
    ) -> Result<CanvasView, KansoError> {
        let path: Vec<(f64, f64)> = path.iter().map(|p| (p.x, p.y)).collect();
        self.edit(known, |c| {
            c.erase_along(&path, radius, now_ms()).map(Entry::Strokes)
        })
    }

    /// The strokes a lasso polygon encloses.
    pub fn select(&self, polygon: Vec<CanvasPoint>) -> Result<Vec<StrokeKey>, KansoError> {
        guard(|| {
            let polygon: Vec<(f64, f64)> = polygon.iter().map(|p| (p.x, p.y)).collect();
            Ok(self
                .lock()
                .canvas
                .strokes_in(&polygon)
                .into_iter()
                .map(|r| StrokeKey {
                    node_id: r.node,
                    stroke_id: r.stroke,
                })
                .collect())
        })
    }

    /// Deletes the selected strokes.
    pub fn delete_strokes(
        &self,
        keys: Vec<StrokeKey>,
        known: Vec<String>,
    ) -> Result<CanvasView, KansoError> {
        self.edit(known, |c| {
            c.delete_strokes(&refs(&keys), now_ms()).map(Entry::Strokes)
        })
    }

    /// Moves the selected strokes by `(dx, dy)`.
    pub fn move_strokes(
        &self,
        keys: Vec<StrokeKey>,
        dx: f64,
        dy: f64,
        known: Vec<String>,
    ) -> Result<CanvasView, KansoError> {
        self.edit(known, |c| {
            c.move_strokes(&refs(&keys), dx, dy, now_ms())
                .map(Entry::Strokes)
        })
    }

    /// Adds a text card at `(x, y)`.
    pub fn add_text_card(
        &self,
        x: f64,
        y: f64,
        text: String,
        known: Vec<String>,
    ) -> Result<CanvasView, KansoError> {
        self.edit(known, |c| {
            let id = canvas::unique_id("t", &node_ids(c));
            c.put_node(
                Node {
                    id: id.clone(),
                    kind: "text".into(),
                    x,
                    y,
                    width: CARD_W,
                    height: CARD_H,
                    text: Some(text.clone()),
                    ..Node::default()
                },
                now_ms(),
            );
            Some(Entry::Node {
                before: None,
                after: c.node(&id).cloned(),
            })
        })
    }

    /// Adds a card for note `note_id` at `(x, y)`; its title stays live.
    /// [`KansoError::NotFound`] for a note that isn't there.
    pub fn add_note_card(
        &self,
        note_id: String,
        x: f64,
        y: f64,
        known: Vec<String>,
    ) -> Result<CanvasView, KansoError> {
        let title = self
            .lib
            .stored(&note_id)
            .map(|n| n.title)
            .map_err(|_| KansoError::NotFound)?;
        self.edit(known, |c| {
            let id = canvas::unique_id("n", &node_ids(c));
            c.put_node(
                Node {
                    id: id.clone(),
                    kind: "file".into(),
                    x,
                    y,
                    width: 260.0,
                    height: 80.0,
                    file: Some(note_id.clone()),
                    title: Some(title.clone()),
                    ..Node::default()
                },
                now_ms(),
            );
            Some(Entry::Node {
                before: None,
                after: c.node(&id).cloned(),
            })
        })
    }

    /// Changes a text card's text.
    pub fn set_card_text(
        &self,
        id: String,
        text: String,
        known: Vec<String>,
    ) -> Result<CanvasView, KansoError> {
        self.change_node(id, known, |n| {
            if n.kind != "text" {
                return false;
            }
            n.text = Some(text.clone());
            true
        })
    }

    /// Moves and resizes a card.
    pub fn set_card_rect(
        &self,
        id: String,
        rect: CanvasRect,
        known: Vec<String>,
    ) -> Result<CanvasView, KansoError> {
        self.change_node(id, known, |n| {
            if n.kind == INK || !rect.width.is_finite() || !rect.height.is_finite() {
                return false;
            }
            n.x = rect.x;
            n.y = rect.y;
            n.width = rect.width.max(40.0);
            n.height = rect.height.max(24.0);
            true
        })
    }

    /// Deletes a card (and the arrows that end on it).
    pub fn delete_card(&self, id: String, known: Vec<String>) -> Result<CanvasView, KansoError> {
        self.edit(known, |c| {
            let before = c.node(&id).filter(|n| n.kind != INK).cloned()?;
            c.remove_node(&id, now_ms()).then_some(Entry::Node {
                before: Some(before),
                after: None,
            })
        })
    }

    /// Draws an arrow from card `from` to card `to`.
    pub fn add_arrow(
        &self,
        from: String,
        to: String,
        known: Vec<String>,
    ) -> Result<CanvasView, KansoError> {
        self.edit(known, |c| {
            let ok = |id: &str| c.node(id).is_some_and(|n| n.kind != INK);
            if from == to || !ok(&from) || !ok(&to) {
                return None;
            }
            let taken: HashSet<String> = c.edges.iter().map(|e| e.id.clone()).collect();
            let id = canvas::unique_id("e", &taken);
            c.put_edge(
                Edge {
                    id: id.clone(),
                    from_node: from.clone(),
                    to_node: to.clone(),
                    to_end: Some("arrow".into()),
                    ..Edge::default()
                },
                now_ms(),
            );
            Some(Entry::Edge {
                before: None,
                after: c.edges.iter().find(|e| e.id == id).cloned(),
            })
        })
    }

    /// Takes back the last edit. Strokes come back as new versions, never as
    /// an older document.
    pub fn undo(&self, known: Vec<String>) -> Result<CanvasView, KansoError> {
        self.step(known, true)
    }

    /// Redoes the last undone edit.
    pub fn redo(&self, known: Vec<String>) -> Result<CanvasView, KansoError> {
        self.step(known, false)
    }

    /// Writes the canvas if it changed. A canvas that changed elsewhere
    /// meanwhile (a sync) is merged with this one first, so neither side's
    /// strokes are lost.
    pub fn save(&self) -> Result<CanvasSaved, KansoError> {
        guard(|| {
            let mut state = self.lock();
            if !state.dirty {
                return Ok(CanvasSaved {
                    rev: state.rev.clone(),
                    bytes: canvas::to_body(&state.canvas).len() as u32,
                    merged: false,
                });
            }
            let mut merged = false;
            for _ in 0..3 {
                match self.lib.write_canvas(&state.id, &state.rev, &state.canvas) {
                    Ok((rev, bytes)) => {
                        state.rev = rev.clone();
                        state.dirty = false;
                        return Ok(CanvasSaved {
                            rev,
                            bytes: bytes as u32,
                            merged,
                        });
                    }
                    Err(KansoError::Conflict { .. }) => {
                        let (note, theirs) = self.lib.read_canvas(&state.id)?;
                        state.canvas = canvas::merge(&state.canvas, &theirs);
                        state.rev = note.rev.unwrap_or_default();
                        merged = true;
                    }
                    Err(err) => return Err(err),
                }
            }
            Err(KansoError::Conflict {
                current_rev: state.rev.clone(),
            })
        })
    }

    /// Moves the selected strokes to a new canvas in the same notebook, for
    /// a canvas near its size limit. Both notes are saved.
    pub fn move_to_new_canvas(
        &self,
        keys: Vec<StrokeKey>,
        title: String,
        known: Vec<String>,
    ) -> Result<(Saved, CanvasView), KansoError> {
        guard(|| {
            let (mut fresh, notebook) = {
                let state = self.lock();
                let mut fresh = Canvas::default();
                let wanted: HashSet<&StrokeKey> = keys.iter().collect();
                let now = now_ms();
                for (node, s) in state.canvas.live_strokes() {
                    let key = StrokeKey {
                        node_id: node.id.clone(),
                        stroke_id: s.id.clone(),
                    };
                    if !wanted.contains(&key) {
                        continue;
                    }
                    let mut copy = s.clone();
                    for p in &mut copy.points {
                        p.x += node.x;
                        p.y += node.y;
                    }
                    let _ = fresh.draw(copy, now);
                }
                let notebook = self.lib.stored(&state.id)?.book_id;
                (fresh, notebook)
            };
            if fresh.live_strokes().next().is_none() {
                return Err(KansoError::invalid("Select some ink to move first."));
            }
            fresh.refit_ink();
            let saved =
                self.lib
                    .write_new_canvas(Some(notebook), None, title, &fresh, Vec::new())?;
            let view = self.edit(known, |c| {
                c.delete_strokes(&refs(&keys), now_ms()).map(Entry::Strokes)
            })?;
            self.save()?;
            Ok((saved, view))
        })
    }

    /// The canvas as an SVG (the whole content), on a light or dark page.
    pub fn export_svg(&self, dark: bool) -> Result<String, KansoError> {
        guard(|| {
            let titles = self.titles();
            let palette = if dark { Palette::DARK } else { Palette::LIGHT };
            let scene = draw::Scene {
                titles: Some(&titles),
                ..draw::Scene::default()
            };
            draw::render(&self.lock().canvas, &scene, &palette)
                .map(|svg| svg.markup)
                .map_err(|failure| KansoError::invalid(failure.badge()))
        })
    }

    /// The canvas as an `.excalidraw` file.
    pub fn export_excalidraw(&self) -> Result<String, KansoError> {
        guard(|| Ok(draw::excalidraw::export(&self.lock().canvas, now_ms())))
    }

    fn titles(&self) -> HashMap<String, String> {
        self.lib
            .inner
            .store
            .note_titles()
            .map(|all| all.into_iter().collect())
            .unwrap_or_default()
    }

    /// Runs `edit` on the document; a step it returns is kept for undo.
    fn edit(
        &self,
        known: Vec<String>,
        edit: impl FnOnce(&mut Canvas) -> Option<Entry>,
    ) -> Result<CanvasView, KansoError> {
        guard(|| {
            let mut state = self.lock();
            if let Some(step) = edit(&mut state.canvas) {
                state.push(step);
            }
            Ok(view(&state, &self.titles(), &known.into_iter().collect()))
        })
    }

    fn change_node(
        &self,
        id: String,
        known: Vec<String>,
        change: impl FnOnce(&mut Node) -> bool,
    ) -> Result<CanvasView, KansoError> {
        self.edit(known, |c| {
            let before = c.node(&id).cloned()?;
            let mut next = before.clone();
            if !change(&mut next) {
                return None;
            }
            c.put_node(next, now_ms());
            Some(Entry::Node {
                before: Some(before),
                after: c.node(&id).cloned(),
            })
        })
    }

    fn step(&self, known: Vec<String>, undo: bool) -> Result<CanvasView, KansoError> {
        guard(|| {
            let mut state = self.lock();
            let entry = if undo {
                state.undo.pop()
            } else {
                state.redo.pop()
            };
            if let Some(entry) = entry {
                let now = now_ms();
                match &entry {
                    Entry::Strokes(action) => {
                        state.canvas.apply(action, !undo, now);
                    }
                    Entry::Node { before, after } => {
                        let target = if undo { before } else { after };
                        match target {
                            Some(node) => state.canvas.put_node(node.clone(), now),
                            None => {
                                let id = if undo { &after } else { &before };
                                if let Some(id) = id.as_ref().map(|n| n.id.clone()) {
                                    state.canvas.remove_node(&id, now);
                                }
                            }
                        }
                    }
                    Entry::Edge { before, after } => {
                        let target = if undo { before } else { after };
                        match target {
                            Some(edge) => state.canvas.put_edge(edge.clone(), now),
                            None => {
                                let other = if undo { after } else { before };
                                if let Some(id) = other.as_ref().map(|e| e.id.clone()) {
                                    state.canvas.remove_edge(&id, now);
                                }
                            }
                        }
                    }
                }
                state.dirty = true;
                if undo {
                    state.redo.push(entry);
                } else {
                    state.undo.push(entry);
                }
            }
            Ok(view(&state, &self.titles(), &known.into_iter().collect()))
        })
    }
}

impl State {
    fn push(&mut self, entry: Entry) {
        self.undo.push(entry);
        if self.undo.len() > UNDO_LIMIT {
            self.undo.remove(0);
        }
        self.redo.clear();
        self.dirty = true;
    }
}

fn point(s: &PenSample) -> Point {
    Point {
        x: s.x,
        y: s.y,
        pressure: if s.pressure.is_finite() {
            s.pressure.clamp(0.0, 1.0)
        } else {
            0.5
        },
        tilt: s.azimuth.zip(s.altitude),
    }
}

fn refs(keys: &[StrokeKey]) -> Vec<StrokeRef> {
    keys.iter()
        .map(|k| StrokeRef {
            node: k.node_id.clone(),
            stroke: k.stroke_id.clone(),
        })
        .collect()
}

fn node_ids(c: &Canvas) -> HashSet<String> {
    c.nodes.iter().map(|n| n.id.clone()).collect()
}

fn rect(b: &Bounds) -> CanvasRect {
    CanvasRect {
        x: b.x,
        y: b.y,
        width: b.width,
        height: b.height,
    }
}

fn view(state: &State, titles: &HashMap<String, String>, known: &HashSet<String>) -> CanvasView {
    let canvas = &state.canvas;
    let mut strokes: Vec<(f64, CanvasStroke)> = Vec::new();
    let mut cards = Vec::new();
    let mut groups = Vec::new();
    let by_id: HashMap<&str, &Node> = canvas.live_nodes().map(|n| (n.id.as_str(), n)).collect();
    for node in canvas.live_nodes() {
        match node.kind.as_str() {
            INK => {
                for s in node.strokes.iter().filter(|s| !s.deleted) {
                    let is_known = known.contains(&s.id);
                    let outline = if is_known {
                        Vec::new()
                    } else {
                        pen::outline(s, true)
                            .into_iter()
                            .flat_map(|p| [p[0] + node.x, p[1] + node.y])
                            .collect()
                    };
                    let mut bounds = s.bounds().unwrap_or(Bounds {
                        x: 0.0,
                        y: 0.0,
                        width: 0.0,
                        height: 0.0,
                    });
                    bounds.x += node.x;
                    bounds.y += node.y;
                    strokes.push((
                        s.kanso_z,
                        CanvasStroke {
                            id: s.id.clone(),
                            node_id: node.id.clone(),
                            tool: s.tool.clone(),
                            color: s.color.clone(),
                            opacity: pen::opacity(&s.tool),
                            width: s.width,
                            bounds: rect(&bounds),
                            outline,
                            known: is_known,
                        },
                    ));
                }
            }
            "group" => groups.push(CanvasGroup {
                id: node.id.clone(),
                rect: rect(&node.bounds()),
                label: node.label.clone().unwrap_or_default(),
                color: draw::card_color(node.color.as_deref()),
            }),
            _ => cards.push(card(node, titles)),
        }
    }
    strokes.sort_by(|a, b| a.0.total_cmp(&b.0).then_with(|| a.1.id.cmp(&b.1.id)));
    let arrows = canvas
        .edges
        .iter()
        .filter(|e| !e.deleted)
        .filter_map(|e| {
            let (a, b) = (
                by_id.get(e.from_node.as_str())?,
                by_id.get(e.to_node.as_str())?,
            );
            let c = geometry::curve(e, a, b);
            let p = |v: (f64, f64)| CanvasPoint { x: v.0, y: v.1 };
            Some(CanvasArrow {
                id: e.id.clone(),
                from: p(c.from),
                c1: p(c.c1),
                c2: p(c.c2),
                to: p(c.to),
                from_arrow: c.from_arrow,
                to_arrow: c.to_arrow,
                color: draw::card_color(e.color.as_deref()),
                label: e.label.clone().unwrap_or_default(),
            })
        })
        .collect();
    let bytes = canvas::to_body(canvas).len();
    CanvasView {
        note_id: state.id.clone(),
        rev: state.rev.clone(),
        title: state.title.clone(),
        strokes: strokes.into_iter().map(|(_, s)| s).collect(),
        cards,
        groups,
        arrows,
        bounds: canvas.content_bounds().as_ref().map(rect),
        bytes: bytes as u32,
        budget: match canvas::budget(bytes) {
            Budget::Ok => CanvasBudget::Ok,
            Budget::Warn => CanvasBudget::Warn,
            Budget::Full => CanvasBudget::Full,
        },
        can_undo: !state.undo.is_empty(),
        can_redo: !state.redo.is_empty(),
        dirty: state.dirty,
    }
}

fn card(node: &Node, titles: &HashMap<String, String>) -> CanvasCard {
    let note_id = Canvas::note_id(node).map(str::to_owned);
    let live = note_id.as_ref().and_then(|id| titles.get(id));
    let (kind, text) = match node.kind.as_str() {
        "text" => ("text", node.text.clone().unwrap_or_default()),
        _ if note_id.is_some() => (
            "note",
            live.cloned()
                .or_else(|| node.title.clone())
                .unwrap_or_default(),
        ),
        "link" => ("link", node.url.clone().unwrap_or_default()),
        _ => (
            "file",
            node.title
                .clone()
                .or_else(|| node.file.clone())
                .unwrap_or_default(),
        ),
    };
    CanvasCard {
        id: node.id.clone(),
        kind: kind.into(),
        rect: rect(&node.bounds()),
        text,
        color: draw::card_color(node.color.as_deref()),
        missing: note_id.is_some() && live.is_none(),
        note_id,
    }
}
