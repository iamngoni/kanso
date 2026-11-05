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
//! It is incremental: an entry carries a fingerprint of the strokes it was
//! read from, so only a layer whose strokes changed is read again, and words
//! read from strokes that changed meanwhile are refused. Which layer is next
//! and keeping the words are this module's; reading is the platform's. A
//! locked note, a note in a locked notebook, one tagged `#ai-exclude`, one in
//! the trash and a shared one are never offered, so their bodies are never
//! read, and the store refuses the write if a note locked in the meantime.

use std::collections::HashSet;

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

impl InkToRead {
    /// Names this layer at these strokes, to leave it out of a pass.
    pub fn key(&self) -> String {
        key(&self.note, self.sketch.as_deref(), &self.node, &self.md5)
    }
}

fn key(note: &str, sketch: Option<&str>, node: &str, md5: &str) -> String {
    format!(
        "{note}\u{1f}{}\u{1f}{node}\u{1f}{md5}",
        sketch.unwrap_or("")
    )
}

/// One canvas document inside a note: the canvas itself, or one sketch.
fn documents(note: &InkNote) -> Vec<(Option<String>, Canvas)> {
    if note.canvas {
        return canvas::parse(&note.body)
            .map(|c| vec![(None, c)])
            .unwrap_or_default();
    }
    canvas::sketch::find(&note.body)
        .into_iter()
        .filter_map(|f| {
            let c: Canvas = serde_json::from_str(&note.body[f.json]).ok()?;
            c.kanso_sketch?;
            Some((c.kanso_sketch_id.clone(), c))
        })
        .collect()
}

/// A layer of ink waiting to be read, and the document it is in.
struct Job {
    note: String,
    sketch: Option<String>,
    job: canvas::InkToRead,
    doc: Canvas,
}

impl Job {
    fn key(&self) -> String {
        key(
            &self.note,
            self.sketch.as_deref(),
            &self.job.node,
            &self.job.md5,
        )
    }
}

/// Every layer of ink waiting to be read, most recently edited note first.
fn waiting_jobs(store: &Store) -> kanso_store::Result<Vec<Job>> {
    let mut out = Vec::new();
    for note in store.ink_notes()? {
        for (sketch, doc) in documents(&note) {
            for job in doc.ink_to_read() {
                out.push(Job {
                    note: note.id.clone(),
                    sketch: sketch.clone(),
                    job,
                    doc: doc.clone(),
                });
            }
        }
    }
    Ok(out)
}

/// How many layers of ink are waiting to be read.
pub fn waiting(store: &Store) -> kanso_store::Result<usize> {
    Ok(waiting_jobs(store)?.len())
}

/// The next layer of ink to read, with its picture; `None` when none is
/// waiting. A layer that can't be drawn is kept as having no words, so it
/// is passed over. Layers named in `skip` are left out.
pub fn next_unread_except(
    store: &Store,
    skip: &HashSet<String>,
) -> kanso_store::Result<Option<InkToRead>> {
    let all: Vec<_> = waiting_jobs(store)?
        .into_iter()
        .filter(|job| !skip.contains(&job.key()))
        .collect();
    let count = all.len();
    for (
        ix,
        Job {
            note,
            sketch,
            job,
            doc,
        },
    ) in all.into_iter().enumerate()
    {
        let png = doc
            .live_nodes()
            .find(|n| n.id == job.node)
            .and_then(kanso_render::canvas::recognize::ink_png);
        if let Some(png) = png {
            return Ok(Some(InkToRead {
                note,
                sketch,
                node: job.node,
                md5: job.md5,
                png,
                waiting: count - ix,
            }));
        }
        keep(
            store,
            &note,
            sketch.as_deref(),
            &job.node,
            &job.md5,
            "",
            "none",
        )?;
    }
    Ok(None)
}

/// [`next_unread_except`] with nothing left out.
pub fn next_unread(store: &Store) -> kanso_store::Result<Option<InkToRead>> {
    next_unread_except(store, &HashSet::new())
}

/// Keeps `text` as what ink `node` of `note` (of its inline `sketch`, if it
/// is one) says, read from the strokes with fingerprint `md5` by `engine`.
/// Nothing is kept when the note changed, was locked or is gone, or the
/// strokes changed since: it is offered again. Whether it was kept. Ink that
/// couldn't be read is kept with no words, so it isn't tried again until it
/// changes.
pub fn keep(
    store: &Store,
    note: &str,
    sketch: Option<&str>,
    node: &str,
    md5: &str,
    text: &str,
    engine: &str,
) -> kanso_store::Result<bool> {
    let Some(found) = store.ink_notes()?.into_iter().find(|n| n.id == note) else {
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
    store.set_ink_note_body(note, &found.rev, &new_body)
}

/// Reads the next waiting layer of ink with `reader`, if there is one:
/// returns how many were waiting (this one included), 0 when none. A layer
/// whose words the store refuses to keep (a canvas at its size limit, say)
/// is added to `skip`, so the rest are still read.
pub fn read_next(
    store: &Store,
    reader: &dyn TextReader,
    skip: &mut HashSet<String>,
) -> kanso_store::Result<usize> {
    let Some(ink) = next_unread_except(store, skip)? else {
        return Ok(0);
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
    match keep(
        store,
        &ink.note,
        ink.sketch.as_deref(),
        &ink.node,
        &ink.md5,
        &text,
        engine,
    ) {
        Ok(_) => {}
        Err(StoreError::Invalid(why)) => {
            eprintln!(
                "kanso: couldn't keep the handwriting in {}: {why}",
                ink.note
            );
            skip.insert(ink.key());
        }
        Err(err) => return Err(err),
    }
    Ok(ink.waiting)
}

#[cfg(test)]
mod tests;
