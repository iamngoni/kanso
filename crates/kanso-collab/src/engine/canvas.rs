//! A shared canvas against a library (issue #45, section 4): the open
//! view's side of the engine.
//!
//! The view holds a [`Canvas`]; the CRDT ([`CanvasDoc`]) holds the truth. The
//! view hands the engine what it has ([`Engine::sync_canvas`]); the engine
//! writes the difference from what the view last got as Yrs updates, merges
//! what others wrote, and hands back the canvas as it is now. The library's
//! body is the projection of the CRDT: written at most every two seconds
//! while the canvas is being drawn on, and once more when it is let go.
//!
//! A canvas that is too big to be a note's body is never cut short: the
//! library keeps the last canvas that fit, the CRDT keeps everything, and
//! the person is told the canvas is full.

use kanso_core::canvas::{self, Canvas, Point};
use kanso_core::now_ms;
use kanso_store::{SharedCanvas, StoreError};

use super::{Engine, Loaded, STATE_LIMIT, UPDATE_PART, colour_of, lock};
use crate::api::{CollabError, Result};
use crate::canvas_doc::{Erased, Streamed, StrokeHead};
use crate::model::{CURSOR_COLOURS, CanvasMerge, CanvasPointer, RemotePointer};

/// The library's body is written at most this often while the canvas is
/// being drawn on.
pub const WRITE_EVERY_MS: i64 = 2_000;

/// What the engine says to the person about a full canvas.
pub const FULL: &str = "This shared canvas is full. Erase some ink or move cards to a new canvas: what you add now can't be saved in the note.";

impl Engine {
    /// The open canvas as the CRDT has it (tombstones included): what the
    /// view starts from, and what its later edits are measured from.
    pub fn canvas(&self, share_id: &str) -> Result<Canvas> {
        let loaded = self.loaded(share_id)?;
        let mut loaded = lock(&loaded);
        let Some(doc) = loaded.doc.canvas() else {
            return Err(not_a_canvas());
        };
        let view = doc.view();
        loaded.canvas_base = Some(view.clone());
        Ok(view)
    }

    /// The view's sync, once drawing pauses and whenever the room says edits
    /// are waiting: takes in what the view changed since it last synced,
    /// merges what arrived, and says what the canvas is now.
    pub fn sync_canvas(&self, share_id: &str, local: &Canvas) -> Result<CanvasMerge> {
        let loaded = self.loaded(share_id)?;
        let mut loaded = lock(&loaded);
        if loaded.doc.canvas().is_none() {
            return Err(not_a_canvas());
        }
        if loaded.ended || loaded.reloading {
            // Nothing to merge into yet: the view keeps what it has.
            return Ok(CanvasMerge {
                canvas: local.clone(),
                changed: false,
                full: loaded.canvas_full,
            });
        }
        let mut dirty = false;
        let base = loaded
            .canvas_base
            .clone()
            .unwrap_or_else(|| loaded.doc.canvas().map(|d| d.view()).unwrap_or_default());
        if loaded.role().may_edit() {
            dirty |= self.take_canvas_edit(&mut loaded, &base, local)?;
        }
        let delivered = self.merge_delivered(&mut loaded)?;
        dirty |= delivered.text;
        if !delivered.authors.is_empty() {
            self.store
                .add_shared_authors(&loaded.note.note_id, &delivered.authors)?;
        }
        let view = loaded
            .doc
            .canvas()
            .map(|d| d.view())
            .ok_or_else(not_a_canvas)?;
        loaded.canvas_base = Some(view.clone());
        if dirty {
            loaded.canvas_dirty = true;
        }
        self.settle_canvas(&mut loaded, false)?;
        Ok(CanvasMerge {
            changed: view != *local,
            canvas: view,
            full: loaded.canvas_full,
        })
    }

    /// Writes what the view changed since `base` to the CRDT, as updates in
    /// the outbox. Whether anything was written.
    fn take_canvas_edit(&self, loaded: &mut Loaded, base: &Canvas, local: &Canvas) -> Result<bool> {
        if local == base {
            return Ok(false);
        }
        // A canvas's whole state must fit one snapshot. The note's own limit
        // (1 MB) is far below it, so this is the last guard, not the one a
        // person meets.
        let growth = size_of(local).saturating_sub(size_of(base));
        if growth > 0 && loaded.doc.encode_state().len() + 2 * growth > STATE_LIMIT {
            self.add_problem(&loaded.note.share_id, FULL);
            return Ok(false);
        }
        let Some(doc) = loaded.doc.canvas() else {
            return Ok(false);
        };
        let updates = doc.edit(base, local, UPDATE_PART);
        let mut changed = false;
        for update in updates {
            self.store
                .push_outbox(&loaded.note.share_id, "u", &update)?;
            changed = true;
        }
        if changed {
            self.store.add_shared_authors(
                &loaded.note.note_id,
                std::slice::from_ref(&self.account.caller.username),
            )?;
        }
        Ok(changed)
    }

    /// Streams the points of the stroke being drawn (a chunk at a time,
    /// about every 150 ms), so others see the ink as it is made. `points`
    /// are all the stroke's points so far, relative to its ink node; only
    /// what hasn't gone is sent. [`Erased`] when someone erased the stroke:
    /// the pen's stroke stops there.
    pub fn stream_stroke(
        &self,
        share_id: &str,
        head: &StrokeHead,
        points: &[Point],
    ) -> Result<std::result::Result<Streamed, Erased>> {
        let loaded = self.loaded(share_id)?;
        let mut loaded = lock(&loaded);
        if !loaded.role().may_edit() || loaded.ended || loaded.reloading {
            return Ok(Ok(Streamed::default()));
        }
        let Some(doc) = loaded.doc.canvas() else {
            return Err(not_a_canvas());
        };
        let streamed = match doc.stream(head, points) {
            Ok(streamed) => streamed,
            Err(erased) => return Ok(Err(erased)),
        };
        if let Some(update) = &streamed.update {
            self.store.push_outbox(&loaded.note.share_id, "u", update)?;
            loaded.canvas_dirty = true;
        }
        Ok(Ok(streamed))
    }

    /// Takes back the stroke streamed under `id` that the pen won't finish
    /// ([`CanvasDoc::abandon`](crate::canvas_doc::CanvasDoc::abandon)): what
    /// the others already got of it goes.
    pub fn abandon_stroke(&self, share_id: &str, id: &str) -> Result<()> {
        let loaded = self.loaded(share_id)?;
        let mut loaded = lock(&loaded);
        if !loaded.role().may_edit() || loaded.ended || loaded.reloading {
            return Ok(());
        }
        let Some(doc) = loaded.doc.canvas() else {
            return Err(not_a_canvas());
        };
        if let Some(update) = doc.abandon(id) {
            self.store
                .push_outbox(&loaded.note.share_id, "u", &update)?;
            loaded.canvas_dirty = true;
        }
        Ok(())
    }

    /// Where this device's pointer is on the canvas (`None` when it left),
    /// for the presence others see. Nothing is sent for a pointer that
    /// hasn't moved.
    pub fn set_pointer(&self, share_id: &str, at: Option<(f64, f64, &str)>) {
        let Ok(loaded) = self.loaded(share_id) else {
            return;
        };
        let mut loaded = lock(&loaded);
        loaded.pointer = match (at, loaded.pointer.take()) {
            (Some((x, y, tool)), Some(old))
                if (old.x, old.y, old.tool.as_str()) == (x, y, tool) =>
            {
                Some(old)
            }
            (Some((x, y, tool)), old) => Some(CanvasPointer {
                x,
                y,
                tool: tool.to_owned(),
                // Later than any earlier move, this device's or the one
                // before it was restarted.
                seq: old
                    .map_or(0, |p| p.seq + 1)
                    .max(u64::try_from(now_ms()).unwrap_or(0)),
            }),
            (None, _) => None,
        };
    }

    /// Other people's pointers on the canvas, newest state of each.
    pub fn pointers(&self, share_id: &str) -> Vec<RemotePointer> {
        let Ok(loaded) = self.loaded(share_id) else {
            return Vec::new();
        };
        let mut loaded = lock(&loaded);
        let now = now_ms();
        loaded
            .cursors
            .retain(|_, (_, at)| now - *at < super::CURSOR_TTL_MS);
        let mut out: Vec<RemotePointer> = loaded
            .cursors
            .iter()
            .filter_map(|((account, device), (presence, at))| {
                let pointer = presence.canvas.as_ref()?;
                Some(RemotePointer {
                    account: account.clone(),
                    device: device.clone(),
                    name: presence.name.clone(),
                    x: pointer.x,
                    y: pointer.y,
                    tool: pointer.tool.clone(),
                    colour: colour_of(account) % CURSOR_COLOURS,
                    seen_at: *at,
                })
            })
            .collect();
        out.sort_by(|a, b| (&a.name, &a.device).cmp(&(&b.name, &b.device)));
        out
    }

    /// Writes the library's body when the canvas changed and the last write
    /// is two seconds old: the live room calls it every tick.
    pub(crate) fn flush_canvas(&self, share_id: &str) {
        let Ok(loaded) = self.loaded(share_id) else {
            return;
        };
        let mut loaded = lock(&loaded);
        if loaded.canvas_dirty && loaded.doc.canvas().is_some() {
            let _ = self.settle_canvas(&mut loaded, false);
        }
    }

    /// Makes the library's body the CRDT's projection, after taking in a
    /// title another writer changed. Unless `force`, only when the canvas
    /// changed and the last write is two seconds old.
    pub(crate) fn settle_canvas(&self, loaded: &mut Loaded, force: bool) -> Result<()> {
        if loaded.ended || loaded.reloading {
            return Ok(());
        }
        let note_id = loaded.note.note_id.clone();
        if self.store.note_is_locked(&note_id)? || self.store.note_in_locked_book(&note_id)? {
            self.add_problem(&loaded.note.share_id, kanso_store::LOCKED_SHARED);
            return self.persist(loaded);
        }
        self.take_canvas_title(loaded)?;
        let now = now_ms();
        if !loaded.canvas_dirty || (!force && now - loaded.canvas_written_at < WRITE_EVERY_MS) {
            return Ok(());
        }
        let Some(doc) = loaded.doc.canvas() else {
            return Ok(());
        };
        let projection = doc.to_canvas();
        let (title, _) = super::clamp_text(doc.title(), String::new());
        let inbox = self
            .store
            .role_book(kanso_core::roles::Role::Inbox, kanso_store::Origin::Collab)?;
        for _ in 0..5 {
            let rev = self.store.note(&note_id)?.and_then(|note| note.rev);
            let written = self.store.write_shared_canvas(&SharedCanvas {
                note_id: &note_id,
                title: &title,
                canvas: &projection,
                mark: Some(Engine::mark(&loaded.note)),
                new_in: &inbox,
                expected_rev: rev.as_deref(),
                authors: &[],
            });
            match written {
                Ok(_) => {
                    loaded.base.0 = title;
                    loaded.canvas_dirty = false;
                    loaded.canvas_full = false;
                    loaded.canvas_written_at = now;
                    return self.persist(loaded);
                }
                // Someone wrote the note meanwhile (a tag, a notebook).
                Err(StoreError::Conflict) => continue,
                // Too big to be a note's body, or not a canvas the store
                // takes: the library keeps the last body that fit, the CRDT
                // keeps all, and the person is told.
                Err(StoreError::Invalid(message)) => {
                    loaded.canvas_full = canvas::check_body(&canvas::to_body(&projection)).is_err();
                    let words = if loaded.canvas_full {
                        FULL.to_owned()
                    } else {
                        message
                    };
                    self.add_problem(&loaded.note.share_id, &words);
                    loaded.canvas_dirty = false;
                    loaded.canvas_written_at = now;
                    return self.persist(loaded);
                }
                Err(err) => return Err(err.into()),
            }
        }
        Err(CollabError::Refused(
            "The canvas kept changing while it was being saved.".into(),
        ))
    }

    /// A title another writer put in the library (the title field) becomes
    /// a change to the CRDT's title, made on whatever the title is now.
    fn take_canvas_title(&self, loaded: &mut Loaded) -> Result<()> {
        let Some(note) = self.store.note(&loaded.note.note_id)? else {
            return Ok(());
        };
        if note.title == loaded.base.0 {
            return Ok(());
        }
        let (base, local) = (loaded.base.0.clone(), note.title);
        loaded.base.0 = local.clone();
        if !loaded.role().may_edit() {
            return Ok(());
        }
        let Some(doc) = loaded.doc.canvas() else {
            return Ok(());
        };
        let now = doc.title();
        let target = super::rebase(&base, &local, &now);
        if let Some(update) = doc.set_title(&target) {
            self.store
                .push_outbox(&loaded.note.share_id, "u", &update)?;
            self.store.add_shared_authors(
                &loaded.note.note_id,
                std::slice::from_ref(&self.account.caller.username),
            )?;
            loaded.canvas_dirty = true;
        }
        Ok(())
    }
}

fn not_a_canvas() -> CollabError {
    CollabError::Refused("The note isn't a canvas.".into())
}

/// About how many bytes a canvas takes as JSON.
fn size_of(canvas: &Canvas) -> usize {
    serde_json::to_vec(canvas).map_or(0, |json| json.len())
}
