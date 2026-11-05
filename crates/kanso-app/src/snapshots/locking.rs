//! Snapshot states for locking with attachments: the lock dialog's line
//! about pictures and files, the progress sheet, and the bar a lock that
//! stopped leaves on the note.

use gpui_kit::{AnyWindowHandle, Entity, HeadlessAppContext};
use kanso_engine::lock::files::{Interrupted, Phase, Progress, Summary};
use kanso_store::Store;

use crate::workspace::Workspace;

/// Twelve pictures and files, three of them also shown by other notes.
fn summary() -> Summary {
    Summary {
        files: 12,
        bytes: 84 * 1024 * 1024,
        shared: 3,
    }
}

/// The dialog Lock Note asks, with its line about the note's attachments and
/// the copies it makes.
pub(super) fn lock_dialog(
    cx: &mut HeadlessAppContext,
    window: AnyWindowHandle,
    _: &Entity<Workspace>,
    _: &Store,
) {
    cx.update_window(window, |_, window, cx| {
        crate::dialogs::confirm(
            "Lock this note?",
            crate::locked::lock_description("MCP support model", &summary()),
            "Lock",
            window,
            cx,
            |_, _| {},
        );
    })
    .unwrap();
}

/// The progress sheet, five of twelve files sealed.
pub(super) fn lock_progress(
    cx: &mut HeadlessAppContext,
    window: AnyWindowHandle,
    _: &Entity<Workspace>,
    _: &Store,
) {
    cx.update_window(window, |_, window, cx| {
        let summary = summary();
        let sheet = crate::lock_flow::open_sheet(
            "Locking note",
            crate::lock_flow::sheet_line(&summary),
            &summary,
            window,
            cx,
        );
        sheet.update(cx, |sheet, cx| {
            sheet.show(
                Progress {
                    phase: Phase::Sealing,
                    done: 5,
                    total: summary.files,
                    bytes_done: 31 * 1024 * 1024,
                    bytes_total: summary.bytes,
                },
                cx,
            )
        });
    })
    .unwrap();
}

/// The open note, plain, after a lock stopped with three of its five files
/// sealed.
pub(super) fn lock_interrupted(
    cx: &mut HeadlessAppContext,
    window: AnyWindowHandle,
    workspace: &Entity<Workspace>,
    _: &Store,
) {
    let editor = cx.update(|cx| workspace.read(cx).editor.clone());
    cx.update_window(window, |_, _, cx| {
        editor.update(cx, |editor, cx| {
            editor.show_interrupted_for_test(
                Interrupted {
                    files: 5,
                    sealed: 3,
                },
                cx,
            )
        });
    })
    .unwrap();
}
