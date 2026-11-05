//! Finds the canvas note a `canvas` fence names, for every place a note is
//! drawn with its embeds: the exports, the published pages and the phone.

use kanso_render::canvas::embed::EmbedNote;
use kanso_store::Store;

/// What an export of `body` should tell the person about its canvas embeds:
/// each notice once, in the order found (a whole-canvas embed that is large,
/// an embed whose group is gone). Blocks while the canvases are drawn.
pub fn notices(
    store: &Store,
    body: &str,
    palette: &kanso_render::Palette,
) -> Vec<kanso_render::canvas::embed::Notice> {
    use kanso_render::canvas::embed::{Shown, show};
    let notes = |id: &str| embed_note(store, id);
    let mut found = Vec::new();
    for embed in kanso_core::canvas::embed::find(body) {
        if let Shown::Drawing(svg) = show(&embed, &notes, palette) {
            for notice in svg.notices {
                if !found.contains(&notice) {
                    found.push(notice);
                }
            }
        }
    }
    found
}

/// What an embed of note `id` shows. `None` for a note that isn't there or
/// is in the trash. A locked note (itself, or by its notebook) is
/// [`EmbedNote::Locked`], found without reading its text, so a locked
/// canvas never reaches a drawing, a page or a cache.
pub fn embed_note(store: &Store, id: &str) -> Option<EmbedNote> {
    if store.note_is_locked(id).ok()? {
        return Some(EmbedNote::Locked);
    }
    let note = store.note(id).ok()??;
    if note.is_trashed() {
        return None;
    }
    if note.is_locked() {
        return Some(EmbedNote::Locked);
    }
    if note.doctype != "canvas" {
        return Some(EmbedNote::NotCanvas);
    }
    Some(EmbedNote::Open {
        title: note.title,
        body: note.body.into(),
    })
}
