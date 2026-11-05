//! A canvas embedded in a note: a `canvas` fence names a canvas note (and
//! maybe one of its groups) and the preview shows it in place.
//!
//! The canvas note is read off the main thread and kept for a moment; the
//! drawing is a `kanso_render` job like any other. A locked canvas is never
//! read: the preview shows a lock and "Locked canvas", and nothing of the
//! canvas is drawn or kept. Hovering shows **Open canvas**, **Open at this
//! group** and **Copy as image**; clicking the picture opens the canvas.
//!
//! No view here reads its own entity while it renders: the hover state is
//! a keyed state of its own, as the drawing block's is.

use std::collections::{HashMap, HashSet};
use std::sync::{Arc, LazyLock, Mutex};
use std::time::{Duration, Instant};

use gpui_kit::assets::IconName as Lucide;
use gpui_kit::component::button::{Button, ButtonVariants as _};
use gpui_kit::component::notification::Notification;
use gpui_kit::component::{Icon, Sizable as _, WindowExt as _, h_flex, v_flex};
use gpui_kit::prelude::FluentBuilder as _;
use gpui_kit::*;
use kanso_core::canvas::Embed;
use kanso_render::Job;
use kanso_render::canvas::embed::{EmbedNote, Notice, Spec};

use crate::preview_rich::{
    BLOCK, Block, CANVAS_GROUP_ANCHOR, Rich, drawing, image, palette, placeholder,
};
use crate::theme::Colors;
use crate::zoom::px;

/// How long a canvas note read is trusted before it is read again in the
/// background; the preview keeps showing the old one meanwhile.
const FRESH: Duration = Duration::from_secs(1);

/// What was read of each canvas note, and when.
static NOTES: LazyLock<Mutex<HashMap<String, Read>>> = LazyLock::new(Default::default);

/// When a canvas note was read, and what it was (`None`: no such note).
type Read = (Instant, Option<EmbedNote>);
static READING: LazyLock<Mutex<HashSet<String>>> = LazyLock::new(Default::default);

/// What the canvas note `id` is: `Some(None)` for no such note, `None`
/// until the first read finishes. A note older than [`FRESH`] is read again
/// in the background, and the preview woken if it changed.
fn note(id: &str, rich: &Rich, cx: &mut App) -> Option<Option<EmbedNote>> {
    let known = NOTES.lock().ok()?.get(id).cloned();
    let stale = known.as_ref().is_none_or(|(at, _)| at.elapsed() > FRESH);
    let known = if stale {
        shown_while_stale(id, known, &rich.store)
    } else {
        known
    };
    if stale && READING.lock().ok()?.insert(id.to_owned()) {
        let (store, id, wake) = (rich.store.clone(), id.to_owned(), rich.waker());
        cx.background_executor()
            .spawn(async move {
                let found = kanso_engine::canvas_embed::embed_note(&store, &id);
                let changed = NOTES.lock().is_ok_and(|mut notes| {
                    let changed = notes.get(&id).is_none_or(|(_, old)| *old != found);
                    notes.insert(id.clone(), (Instant::now(), found));
                    changed
                });
                if let Ok(mut reading) = READING.lock() {
                    reading.remove(&id);
                }
                if changed {
                    wake();
                }
            })
            .detach();
    }
    known.map(|(_, note)| note)
}

/// What an old read of canvas `id` may still show while it is read again:
/// a drawing only while the canvas is still unlocked. A canvas locked since
/// (itself, or by its notebook) is never drawn from memory, not even for a
/// frame; asking the store that is one small query. Anything else old is
/// shown as it was until the new read lands.
fn shown_while_stale(id: &str, known: Option<Read>, store: &kanso_store::Store) -> Option<Read> {
    let (at, note) = known?;
    if matches!(note, Some(EmbedNote::Open { .. })) && store.note_is_locked(id).unwrap_or(true) {
        let locked = (at, Some(EmbedNote::Locked));
        if let Ok(mut notes) = NOTES.lock() {
            notes.insert(id.to_owned(), locked.clone());
        }
        return Some(locked);
    }
    Some((at, note))
}

/// Note `id` changed: what was read of it is forgotten, so no preview draws
/// it from before the change (a canvas locked or trashed since, above all).
/// A notebook or a tag can lock or move many notes at once, so a change to
/// one forgets every canvas. Whether `body` (a preview's text) embeds a
/// canvas the change may touch, to draw again.
pub(crate) fn forget(id: &str, body: &str) -> bool {
    if let Ok(mut notes) = NOTES.lock() {
        if touches_every_note(id) {
            notes.clear();
        } else {
            notes.remove(id);
        }
    }
    redraws(id, body)
}

/// A notebook or a tag: changing one can lock, move or hide many notes.
fn touches_every_note(id: &str) -> bool {
    id.starts_with("book:") || id.starts_with("tag:")
}

/// Whether a change to `id` touches what `body` embeds. Pure.
fn redraws(id: &str, body: &str) -> bool {
    body.contains("```canvas")
        && (touches_every_note(id)
            || kanso_core::canvas::embed::find(body)
                .iter()
                .any(|embed| embed.note == id))
}

#[cfg(test)]
mod tests {
    use kanso_core::DocType;
    use kanso_store::{Origin, Put, Store};
    use serde_json::json;

    use super::*;

    fn canvas(store: &Store) -> String {
        let book = store
            .put(DocType::Book, json!({"name": "B"}), Put::Upsert, Origin::Ui)
            .unwrap()
            .id;
        store
            .put(
                DocType::Note,
                json!({"title": "Board", "doctype": "canvas", "bookId": book,
                       "body": "```canvas\n{\"nodes\":[],\"edges\":[]}\n```"}),
                Put::Upsert,
                Origin::Ui,
            )
            .unwrap()
            .id
    }

    /// A canvas read while open and locked since is never drawn from what
    /// was read: the old read shows the lock, not the drawing.
    #[test]
    fn a_canvas_locked_since_it_was_read_is_not_drawn_from_memory() {
        let dir = tempfile::tempdir().unwrap();
        let store = Store::open(dir.path().join("k.db")).unwrap();
        let id = canvas(&store);
        let old = Instant::now() - Duration::from_secs(60);
        let open = EmbedNote::Open {
            title: "Board".into(),
            body: "secret".into(),
        };
        let shown = shown_while_stale(&id, Some((old, Some(open.clone()))), &store);
        assert_eq!(shown.map(|(_, n)| n), Some(Some(open.clone())));
        let rev = store.get(&id).unwrap().unwrap().rev;
        store
            .put(
                DocType::Note,
                json!({"_id": id, "_rev": rev, "body": "",
                       "kansoLock": {"v": 1, "salt": "c2FsdA", "m": 64, "t": 1, "p": 1, "data": "sealed"}}),
                Put::Merge,
                Origin::Sync,
            )
            .unwrap();
        let shown = shown_while_stale(&id, Some((old, Some(open))), &store);
        assert_eq!(shown.map(|(_, n)| n), Some(Some(EmbedNote::Locked)));
    }

    /// A change forgets what was read: of that note, or of every canvas for
    /// a notebook or tag; and a preview redraws only for an embed it shows.
    #[test]
    fn a_change_forgets_what_was_read() {
        let body = "Plan\n```canvas\nnote:abc\n```\n";
        seed("note:abc", EmbedNote::Locked);
        seed("note:other", EmbedNote::Locked);
        assert!(forget("note:abc", body));
        assert!(NOTES.lock().unwrap().get("note:abc").is_none());
        assert!(NOTES.lock().unwrap().get("note:other").is_some());
        assert!(!forget("note:other", body));
        assert!(!forget("note:abc", "no embeds here"));
        assert!(redraws("book:x", body) && redraws("tag:x", body));
        assert!(!redraws("book:x", "no embeds here"));
        assert!(!redraws("file:x", body));
    }
}
