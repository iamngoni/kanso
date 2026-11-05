//! Snapshot states for the library key (issue #243, step 11): the sheet that
//! asks to save it, the same sheet from Show key…, and Restore from
//! backup… with and without the 24 words.
//!
//! Every picture uses a demo key (the bytes 0 to 31), never a real one.

use gpui_kit::{AnyWindowHandle, Entity, HeadlessAppContext};
use kanso_store::{KEY_BYTES, LibraryKey, Store};

use crate::library_key_sheet::Mode;
use crate::workspace::Workspace;

/// The shots: name (`dark-…` or `light-…`), how to set the window up, and
/// the width of the dialog it opens.
pub const SHOTS: &[(&str, super::Prepare, Option<f32>)] = &[
    ("dark-library-key-save", save, Some(WIDTH)),
    ("light-library-key-save", save, Some(WIDTH)),
    ("dark-library-key-show", show, Some(WIDTH)),
    ("light-library-key-show", show, Some(WIDTH)),
    ("dark-library-key-restore", restore, Some(WIDTH)),
    ("light-library-key-restore", restore, Some(WIDTH)),
    ("dark-library-key-restore-words", restore_words, Some(WIDTH)),
    (
        "light-library-key-restore-words",
        restore_words,
        Some(WIDTH),
    ),
];

const WIDTH: f32 = crate::library_key_sheet::SHEET_WIDTH;

/// The demo key: 0, 1, 2 … 31. Not a key anyone uses.
pub fn demo_key() -> LibraryKey {
    let mut bytes = [0u8; KEY_BYTES];
    for (ix, byte) in bytes.iter_mut().enumerate() {
        *byte = ix as u8;
    }
    LibraryKey::from_bytes(bytes)
}

/// The sheet Kanso opens after encrypting, with the key not yet saved.
fn save(cx: &mut HeadlessAppContext, window: AnyWindowHandle, _: &Entity<Workspace>, _: &Store) {
    cx.update(|cx| cx.set_reduce_motion(true));
    cx.update_window(window, |_, window, cx| {
        let words = demo_key().to_words();
        crate::library_key_sheet::open_sheet_for_test(Mode::Save, &words, false, window, cx);
    })
    .unwrap();
}

/// Show key…, for a key that was saved.
fn show(cx: &mut HeadlessAppContext, window: AnyWindowHandle, _: &Entity<Workspace>, _: &Store) {
    cx.update(|cx| cx.set_reduce_motion(true));
    cx.update_window(window, |_, window, cx| {
        let words = demo_key().to_words();
        crate::library_key_sheet::open_sheet_for_test(Mode::Show, &words, true, window, cx);
    })
    .unwrap();
}

/// A backup this computer's key opens: just the question.
fn restore(cx: &mut HeadlessAppContext, window: AnyWindowHandle, _: &Entity<Workspace>, _: &Store) {
    cx.update(|cx| cx.set_reduce_motion(true));
    cx.update_window(window, |_, window, cx| {
        crate::restore_sheet::open_for_test(&backup(), false, None, window, cx);
    })
    .unwrap();
}

/// A backup from another computer: the words are asked for, and the last
/// try failed.
fn restore_words(
    cx: &mut HeadlessAppContext,
    window: AnyWindowHandle,
    _: &Entity<Workspace>,
    _: &Store,
) {
    cx.update(|cx| cx.set_reduce_motion(true));
    cx.update_window(window, |_, window, cx| {
        crate::restore_sheet::open_for_test(
            &backup(),
            true,
            Some("Word 5 isn't one of the library key words. Check how it is spelled."),
            window,
            cx,
        );
    })
    .unwrap();
}

/// A backup made three days before the demo's "now".
fn backup() -> std::path::PathBuf {
    let made = super::DEMO_NOW_MS - 3 * 24 * 60 * super::MINUTE;
    std::path::PathBuf::from(format!("/Users/demo/Kanso/backups/kanso-manual-{made}.db"))
}
