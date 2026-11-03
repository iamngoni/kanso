//! One change to several notes at once: the selected notes from the list's
//! menu or the palette, or the notes dropped on a notebook or tag in the
//! sidebar. Every note changes in one write, and a notice offers Undo,
//! which puts back what each note had, in one write too.

use gpui_kit::component::WindowExt as _;
use gpui_kit::component::button::Button;
use gpui_kit::component::notification::Notification;
use gpui_kit::*;
use kanso_core::DocType;
pub use kanso_engine::batch::Change;
use kanso_engine::batch::{self, Edit, summary, turned_on};
use kanso_store::Store;
use kanso_store::{NoteSummary, Origin};
use serde_json::Value;
use std::sync::Arc;
use std::time::{Duration, Instant};

use crate::library::Library;

/// Whether the notes must go one at a time, as [`batch::needs_one_by_one`]
/// says it, for the notebooks the library has locked.
fn must_go_one_by_one(notes: &[NoteSummary], change: &Change, library: &Library) -> bool {
    let locked = library
        .books
        .iter()
        .filter(|book| library.is_book_locked(&book.id))
        .map(|book| book.id.clone())
        .collect();
    batch::needs_one_by_one(notes, change, &locked)
}

/// Makes `change` to the listed notes among `ids`, in one write, and says
/// so with Undo. Typing waiting in any editor is saved first, so an open
/// note takes the change like any other.
pub fn apply(
    ids: &[String],
    change: Change,
    library: &Entity<Library>,
    window: &mut Window,
    cx: &mut App,
) {
    let (notes, place) = {
        let library = library.read(cx);
        let mut notes: Vec<NoteSummary> = library
            .notes
            .iter()
            .filter(|n| ids.contains(&n.id))
            .cloned()
            .collect();
        // A note open but not in the list now (found by search, say) still
        // takes the change.
        for id in ids {
            if !notes.iter().any(|n| &n.id == id)
                && let Ok(Some(note)) = library.store.note(id)
            {
                notes.push(summary_of(&note));
            }
        }
        if must_go_one_by_one(&notes, &change, library) {
            window.push_notification(
                Notification::info(
                    "Notes go into a locked notebook one at a time: open the note and choose Move to notebook….",
                ),
                cx,
            );
            return;
        }
        let place = match &change {
            Change::Move(book) => library.book(book).map(|b| b.name.clone()),
            // A tag just made isn't in the library until it reloads.
            Change::AddTag(tag) | Change::RemoveTag(tag) | Change::ToggleTag(tag) => {
                library.tag(tag).map(|t| t.name.clone()).or_else(|| {
                    let tags = library.store.tags().ok()?;
                    tags.into_iter().find(|t| &t.id == tag).map(|t| t.name)
                })
            }
            Change::Status(status) => Some(status.label().to_owned()),
            Change::TogglePin => None,
        };
        (notes, place.unwrap_or_default())
    };
    let edits = batch::plan(&notes, &change);
    if edits.is_empty() {
        return;
    }
    let on = turned_on(&change, &edits);
    let from: Vec<String> = edits
        .iter()
        .filter_map(|edit| notes.iter().find(|n| n.id == edit.id))
        .map(|n| n.book_id.clone())
        .collect();
    crate::editor::peers::flush_everywhere(cx);
    let store = library.read(cx).store.clone();
    let changes: Vec<Value> = edits.iter().map(|edit| edit.change.clone()).collect();
    let count = changes.len();
    if let Err(err) = store.merge_many(DocType::Note, changes, Origin::Ui) {
        window.push_notification(
            Notification::error(format!("Couldn't change the notes: {err}")),
            cx,
        );
        return;
    }
    // One count per note changed, recorded together.
    let mut tally: Vec<(crate::metrics::Feature, u32)> = Vec::new();
    for feature in crate::metrics_hooks::batch_features(&change, &from, on) {
        match tally.iter_mut().find(|(f, _)| *f == feature) {
            Some((_, n)) => *n += 1,
            None => tally.push((feature, 1)),
        }
    }
    for (feature, n) in tally {
        crate::metrics::record_n(cx, feature, n);
    }
    remember(&store, &edits, cx);
    let notification =
        // Each batch's notice takes the place of the one before, so its
        // Undo is always for the latest change.
        Notification::info(summary(&change, count, &place, on))
            .id::<Edit>()
            .action(move |_, _, _| {
                let (store, edits) = (store.clone(), edits.clone());
                Button::new("undo-notes")
                    .label("Undo")
                    .debug_selector(|| "undo-notes".into())
                    .on_click(move |_, window, cx| {
                        cx.set_global(Latest(None));
                        undo_edits(&store, &edits, window, cx);
                    })
            });
    window.push_notification(notification, cx);
}

/// How long a change's notice stays up, and so how long Cmd/Ctrl+Z outside
/// the note's text still takes it back.
pub const NOTICE_SECS: u64 = 5;

/// The latest batch change whose notice is up.
struct Latest(Option<(Instant, Arc<Store>, Vec<Edit>)>);

impl Global for Latest {}

fn remember(store: &Arc<Store>, edits: &[Edit], cx: &mut App) {
    cx.set_global(Latest(Some((
        Instant::now(),
        store.clone(),
        edits.to_vec(),
    ))));
}

/// Whether a change `since` ago still has its notice up. Pure.
pub fn notice_is_up(since: Duration) -> bool {
    since < Duration::from_secs(NOTICE_SECS)
}

/// Takes back the latest change (a trash, a move, any batch) while its
/// notice is up. Whether there was one.
pub fn undo_latest(window: &mut Window, cx: &mut App) -> bool {
    let latest = cx.try_global::<Latest>().and_then(|l| l.0.clone());
    let Some((at, store, edits)) = latest else {
        return false;
    };
    cx.set_global(Latest(None));
    if !notice_is_up(at.elapsed()) {
        return false;
    }
    undo_edits(&store, &edits, window, cx);
    true
}

/// Puts back what each note had, in one write. A note deleted since is
/// skipped, and so is what was changed again after the batch; every note
/// is stamped updatedAt = now.
fn undo_edits(store: &Arc<Store>, edits: &[Edit], window: &mut Window, cx: &mut App) {
    crate::editor::peers::flush_everywhere(cx);
    let now_ms = kanso_core::now_ms();
    let undo: Vec<Value> = edits
        .iter()
        .filter_map(|edit| {
            let note = store.note(&edit.id).ok().flatten()?;
            let current = serde_json::to_value(note).ok()?;
            batch::undo_stamped(&edit.change, &edit.undo, &current, now_ms)
        })
        .collect();
    if undo.is_empty() {
        return;
    }
    if let Err(err) = store.merge_many(DocType::Note, undo, Origin::Ui) {
        window.push_notification(Notification::error(format!("Couldn't undo: {err}")), cx);
    }
}

/// What the batch needs to know of a note.
fn summary_of(note: &kanso_core::Note) -> NoteSummary {
    NoteSummary {
        id: note.id.clone(),
        rev: note.rev.clone().unwrap_or_default(),
        title: note.title.clone(),
        snippet: String::new(),
        book_id: note.book_id.clone(),
        status: note.status,
        tags: note.tags.clone(),
        pinned: note.pinned,
        updated_at: note.updated_at,
        created_at: note.created_at,
        tasks_done: note.num_of_checked_tasks,
        tasks_total: note.num_of_tasks,
        locked: false,
    }
}

#[cfg(test)]
mod tests {
    use proptest::prelude::*;

    use super::{NOTICE_SECS, notice_is_up};
    use std::time::Duration;

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(256))]

        /// The notice is up for exactly `NOTICE_SECS`, whatever the time.
        #[test]
        fn notice_is_up_only_within_its_time(ms in 0u64..20_000) {
            prop_assert_eq!(
                notice_is_up(Duration::from_millis(ms)),
                ms < NOTICE_SECS * 1000
            );
        }
    }
}
