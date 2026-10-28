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
use kanso_store::{NoteSummary, Origin};
use serde_json::Value;

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
        let notes: Vec<NoteSummary> = library
            .notes
            .iter()
            .filter(|n| ids.contains(&n.id))
            .cloned()
            .collect();
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
            Change::AddTag(tag) | Change::ToggleTag(tag) => {
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
                    crate::editor::peers::flush_everywhere(cx);
                    // A note deleted since is skipped, and so is what was
                    // changed again after the batch.
                    let undo: Vec<Value> = edits
                        .iter()
                        .filter_map(|edit| {
                            let note = store.note(&edit.id).ok().flatten()?;
                            let current = serde_json::to_value(note).ok()?;
                            batch::undo_now(&edit.change, &edit.undo, &current)
                        })
                        .collect();
                    if undo.is_empty() {
                        return;
                    }
                    if let Err(err) = store.merge_many(DocType::Note, undo, Origin::Ui) {
                        window.push_notification(
                            Notification::error(format!("Couldn't undo: {err}")),
                            cx,
                        );
                    }
                })
        });
    window.push_notification(notification, cx);
}
