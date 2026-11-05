//! Handwriting search: the words in ink, read on this device so search finds
//! a canvas, or a note with a sketch, by what was written on it.
//!
//! Each layer of ink is drawn as a picture of black strokes on a light page
//! (`kanso_render::canvas::recognize`) and read by the platform's reader, the
//! same [`TextReader`] that reads images (Vision on a Mac). The words go into
//! the body, beside the ink, as `kansoInkOcr` (`kanso_core::canvas::InkOcr`):
//! inside the note's seal and history, so a locked note takes them with it,
//! and synced with it, so a device that can't read ink still searches them.
//!
//! It is incremental twice over. An entry carries a fingerprint of the
//! strokes it was read from, so only a layer whose strokes changed is read
//! again, and words read from strokes that changed meanwhile are refused.
//! And a [`Pass`] remembers what it found: each look at the store reads only
//! the notes written since the last one (`Store::ink_scan`), so a library of
//! big canvases is not parsed whole for every layer or every save. A note
//! is read only once it has been still for the pass's settle time, so ink
//! is never read while it is being drawn. A layer whose words couldn't be
//! kept is not tried again until its note changes.
//!
//! Which layer is next and keeping the words are this module's; reading is
//! the platform's. A locked note, a note in a locked notebook, one tagged
//! `#ai-exclude`, one in the trash and a shared one are never offered, so
//! their bodies are never read, and the store refuses the write if a note
//! was locked in the meantime.

use std::collections::{HashMap, HashSet};
use std::time::{Duration, Instant};

use kanso_core::canvas::{self, Canvas, DEFAULT_ENGINE};
use kanso_store::{InkNote, Store, StoreError};

use super::ocr::TextReader;

/// A layer of ink waiting to be read, with its picture.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct InkToRead {
    /// The note that holds the ink.
    pub note: String,
    /// The inline sketch inside that note, when it isn't a canvas.
    pub sketch: Option<String>,
    /// The ink node.
    pub node: String,
    /// Fingerprint of the strokes drawn; give it back with the words.
    pub md5: String,
    /// The PNG to read.
    pub png: Vec<u8>,
    /// How many layers are waiting, this one included.
    pub waiting: usize,
}

/// What was read from one layer, waiting to be kept ([`keep_read`]).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Read {
    pub ink: InkToRead,
    pub text: String,
    pub engine: &'static str,
}

/// What [`read`] found to do.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Next {
    /// No layer is waiting.
    Done,
    /// Layers wait for their notes to be still: look again after this long.
    Settling(Duration),
    /// One layer was read; keep its words with [`keep_read`].
    Read(Read),
}

/// One layer waiting in a note.
#[derive(Clone, Debug, PartialEq, Eq)]
struct Layer {
    sketch: Option<String>,
    node: String,
    md5: String,
}

/// A note with ink, as the pass last saw it.
#[derive(Clone, Debug)]
struct Seen {
    rev: String,
    updated_at: i64,
    /// When it last changed by anyone but this pass; `None` when it hasn't
    /// since the pass began, so it may be read at once.
    changed: Option<Instant>,
    /// The revision this pass's own write gave it: seeing that one isn't an
    /// edit to wait out.
    ours: Option<String>,
    layers: Vec<Layer>,
}

/// What one reader remembers from layer to layer: how far into the store's
/// changes it has looked, and the layers each note still waits to have read.
/// Keep one for as long as the reader runs.
#[derive(Debug)]
pub struct Pass {
    seq: i64,
    looked: bool,
    settle: Duration,
    notes: HashMap<String, Seen>,
}

impl Pass {
    /// A pass that reads a note only once it has been still for `settle`.
    /// What is waiting when it begins may be read at once.
    pub fn new(settle: Duration) -> Pass {
        Pass {
            seq: 0,
            looked: false,
            settle,
            notes: HashMap::new(),
        }
    }

    /// Takes in what changed in the store since the last look.
    pub fn refresh(&mut self, store: &Store) -> kanso_store::Result<()> {
        let mut scan = store.ink_scan(self.seq)?;
        if scan.rules_changed {
            // A notebook or tag changed: which notes may be read is looked
            // at again for all of them.
            scan = store.ink_scan(0)?;
            let readable: HashSet<&str> = scan.notes.iter().map(|n| n.id.as_str()).collect();
            self.notes.retain(|id, _| readable.contains(id.as_str()));
        }
        for id in &scan.dropped {
            self.notes.remove(id);
        }
        let now = Instant::now();
        for note in &scan.notes {
            self.see(note, now);
        }
        self.seq = scan.seq;
        self.looked = true;
        Ok(())
    }

    /// Takes in `note` as it is now.
    fn see(&mut self, note: &InkNote, now: Instant) {
        let old = self.notes.get(&note.id);
        if old.is_some_and(|seen| seen.rev == note.rev) {
            return;
        }
        let changed = match old {
            Some(seen) if seen.ours.as_deref() == Some(note.rev.as_str()) => seen.changed,
            _ if !self.looked => None,
            _ => Some(now),
        };
        let layers = layers(note);
        self.notes.insert(
            note.id.clone(),
            Seen {
                rev: note.rev.clone(),
                updated_at: note.updated_at,
                changed,
                ours: None,
                layers,
            },
        );
    }

    /// How many layers are waiting to be read, settling or not.
    pub fn waiting(&self) -> usize {
        self.notes.values().map(|seen| seen.layers.len()).sum()
    }

    /// Note `id` is being edited: its layers wait for it to be still again.
    pub fn postpone(&mut self, id: &str) {
        if let Some(seen) = self.notes.get_mut(id) {
            seen.changed = Some(Instant::now());
        }
    }

    /// Takes layer `node` at `md5` off note `note`'s list: it was read, or
    /// can't be, at this revision.
    fn done(&mut self, note: &str, sketch: Option<&str>, node: &str, md5: &str) {
        if let Some(seen) = self.notes.get_mut(note) {
            seen.layers
                .retain(|l| !(l.sketch.as_deref() == sketch && l.node == node && l.md5 == md5));
        }
    }

    /// The next note whose layers may be read now (the most recently edited
    /// first), or how long until one may.
    fn next(&self, now: Instant) -> Result<&str, Option<Duration>> {
        let mut soonest: Option<Duration> = None;
        let mut best: Option<(&str, i64)> = None;
        for (id, seen) in &self.notes {
            if seen.layers.is_empty() {
                continue;
            }
            let still = seen
                .changed
                .map_or(self.settle, |at| now.duration_since(at));
            if still < self.settle {
                let left = self.settle - still;
                soonest = Some(soonest.map_or(left, |s| s.min(left)));
                continue;
            }
            let newer = best.is_none_or(|(best_id, best_at)| {
                (seen.updated_at, std::cmp::Reverse(id.as_str()))
                    > (best_at, std::cmp::Reverse(best_id))
            });
            if newer {
                best = Some((id.as_str(), seen.updated_at));
            }
        }
        best.map(|(id, _)| id).ok_or(soonest)
    }
}

/// The canvas documents inside a note: the canvas itself, or each inline
/// sketch. A sketch without an ID can't be written back, and only the first
/// of two sketches with one ID (a pasted copy) is the one a write finds, so
/// the others are left out.
fn documents(note: &InkNote) -> Vec<(Option<String>, Canvas)> {
    if note.canvas {
        return canvas::parse(&note.body)
            .map(|c| vec![(None, c)])
            .unwrap_or_default();
    }
    let mut ids = HashSet::new();
    canvas::sketch::find(&note.body)
        .into_iter()
        .filter_map(|f| {
            let c: Canvas = serde_json::from_str(&note.body[f.json]).ok()?;
            c.kanso_sketch?;
            let id = c.kanso_sketch_id.clone().filter(|id| !id.is_empty())?;
            ids.insert(id.clone()).then_some((Some(id), c))
        })
        .collect()
}

/// Every layer in `note` waiting to be read.
fn layers(note: &InkNote) -> Vec<Layer> {
    documents(note)
        .into_iter()
        .flat_map(|(sketch, doc)| {
            doc.ink_to_read().into_iter().map(move |job| Layer {
                sketch: sketch.clone(),
                node: job.node,
                md5: job.md5,
            })
        })
        .collect()
}

/// The document `sketch` (or the canvas) in `note`.
fn document(note: &InkNote, sketch: Option<&str>) -> Option<Canvas> {
    documents(note)
        .into_iter()
        .find(|(id, _)| id.as_deref() == sketch)
        .map(|(_, doc)| doc)
}

/// How many layers of ink are waiting to be read, after a look at the store.
pub fn waiting(store: &Store, pass: &mut Pass) -> kanso_store::Result<usize> {
    pass.refresh(store)?;
    Ok(pass.waiting())
}

/// The next layer of ink to read, with its picture: `Ok` with `None` when
/// none is waiting, `Err` with how long until one may be read when they all
/// wait for their notes to be still. A layer that can't be drawn is kept as
/// having no words, so it is passed over.
pub fn next_unread(
    store: &Store,
    pass: &mut Pass,
) -> kanso_store::Result<Result<Option<InkToRead>, Duration>> {
    pass.refresh(store)?;
    loop {
        let id = match pass.next(Instant::now()) {
            Ok(id) => id.to_owned(),
            Err(None) => return Ok(Ok(None)),
            Err(Some(left)) => return Ok(Err(left)),
        };
        let Some(note) = store.ink_note(&id)? else {
            pass.notes.remove(&id);
            continue;
        };
        if pass.notes.get(&id).is_some_and(|seen| seen.rev != note.rev) {
            // Written since the look a moment ago: take it in, and wait.
            pass.see(&note, Instant::now());
            continue;
        }
        let Some(layer) = pass.notes.get(&id).and_then(|s| s.layers.first().cloned()) else {
            continue;
        };
        let png = document(&note, layer.sketch.as_deref()).and_then(|doc| {
            doc.live_nodes()
                .find(|n| n.id == layer.node)
                .and_then(kanso_render::canvas::recognize::ink_png)
        });
        if let Some(png) = png {
            return Ok(Ok(Some(InkToRead {
                note: id,
                sketch: layer.sketch,
                node: layer.node,
                md5: layer.md5,
                png,
                waiting: pass.waiting(),
            })));
        }
        keep(
            store,
            pass,
            &id,
            layer.sketch.as_deref(),
            &layer.node,
            &layer.md5,
            "",
            "none",
        )?;
    }
}

/// Keeps `text` as what ink `node` of `note` (of its inline `sketch`, if it
/// is one) says, read from the strokes with fingerprint `md5` by `engine`.
/// Nothing is kept when the note changed, was locked or is gone, or the
/// strokes changed since: what changed is offered again once it is still.
/// Whether it was kept. Ink that couldn't be read is kept with no words, so
/// it isn't tried again until it changes. Either way the layer is off this
/// pass's list until its note changes.
#[allow(clippy::too_many_arguments)]
pub fn keep(
    store: &Store,
    pass: &mut Pass,
    note: &str,
    sketch: Option<&str>,
    node: &str,
    md5: &str,
    text: &str,
    engine: &str,
) -> kanso_store::Result<bool> {
    pass.done(note, sketch, node, md5);
    let Some(found) = store.ink_note(note)? else {
        return Ok(false);
    };
    let new_body = if found.canvas {
        let Ok(mut doc) = canvas::parse(&found.body) else {
            return Ok(false);
        };
        if sketch.is_some() || !doc.keep_ink_text(node, md5, engine, text) {
            return Ok(false);
        }
        canvas::to_body(&doc)
    } else {
        let Some(id) = sketch else {
            return Ok(false);
        };
        let Some((_, mut doc)) = canvas::sketch::find_by_id(&found.body, id) else {
            return Ok(false);
        };
        if !doc.keep_ink_text(node, md5, engine, text) {
            return Ok(false);
        }
        let Some(body) = canvas::sketch::replace(&found.body, id, &doc) else {
            return Ok(false);
        };
        body
    };
    if new_body == found.body {
        return Ok(true);
    }
    let Some(rev) = store.set_ink_note_body(note, &found.rev, &new_body)? else {
        return Ok(false);
    };
    if let Some(seen) = pass.notes.get_mut(note) {
        seen.ours = Some(rev);
    }
    Ok(true)
}

/// Reads the next waiting layer of ink with `reader`, without keeping the
/// words: the caller keeps them with [`keep_read`] (on the app's own thread,
/// where it can tell whether the note is being edited).
pub fn read(store: &Store, reader: &dyn TextReader, pass: &mut Pass) -> kanso_store::Result<Next> {
    let ink = match next_unread(store, pass)? {
        Ok(Some(ink)) => ink,
        Ok(None) => return Ok(Next::Done),
        Err(left) => return Ok(Next::Settling(left)),
    };
    let text = reader.read(&ink.png).unwrap_or_else(|err| {
        eprintln!(
            "kanso: couldn't read the handwriting in {}: {err}",
            ink.note
        );
        String::new()
    });
    let engine = match reader.engine() {
        "" => DEFAULT_ENGINE,
        engine => engine,
    };
    Ok(Next::Read(Read { ink, text, engine }))
}

/// Keeps what [`read`] read. A layer whose words the store refuses (a canvas
/// at its size limit, say) is passed over until its note changes, so the
/// rest are still read.
pub fn keep_read(store: &Store, pass: &mut Pass, read: &Read) -> kanso_store::Result<bool> {
    let ink = &read.ink;
    match keep(
        store,
        pass,
        &ink.note,
        ink.sketch.as_deref(),
        &ink.node,
        &ink.md5,
        &read.text,
        read.engine,
    ) {
        Ok(kept) => Ok(kept),
        Err(StoreError::Invalid(why)) => {
            eprintln!(
                "kanso: couldn't keep the handwriting in {}: {why}",
                ink.note
            );
            Ok(false)
        }
        Err(err) => Err(err),
    }
}

/// Reads the next waiting layer of ink with `reader` and keeps its words, if
/// there is one ready: returns how many were waiting (this one included), 0
/// when none is ready.
pub fn read_next(
    store: &Store,
    reader: &dyn TextReader,
    pass: &mut Pass,
) -> kanso_store::Result<usize> {
    match read(store, reader, pass)? {
        Next::Read(read) => {
            keep_read(store, pass, &read)?;
            Ok(read.ink.waiting)
        }
        Next::Done | Next::Settling(_) => Ok(0),
    }
}

#[cfg(test)]
mod tests;
