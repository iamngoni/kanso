//! Snapshot states for New Notebook (issue #390, slice 29): the choice of a
//! blank notebook or a Video studio, with Video studio chosen and a name
//! typed, in both themes.

use gpui_kit::{AnyWindowHandle, Entity, HeadlessAppContext};
use kanso_engine::presets::NotebookKind;
use kanso_store::Store;

use crate::new_notebook_sheet::SHEET_WIDTH;
use crate::workspace::Workspace;

/// The shots: name (`dark-…` or `light-…`), how to set the window up, and
/// the width of the dialog it opens.
pub const SHOTS: &[(&str, super::Prepare, Option<f32>)] = &[
    ("dark-new-notebook", studio_chosen, Some(SHEET_WIDTH)),
    ("light-new-notebook", studio_chosen, Some(SHEET_WIDTH)),
];

/// Video studio chosen, "Channel" typed.
fn studio_chosen(
    cx: &mut HeadlessAppContext,
    window: AnyWindowHandle,
    workspace: &Entity<Workspace>,
    _: &Store,
) {
    // Without it the dialog is drawn mid slide-in.
    cx.update(|cx| cx.set_reduce_motion(true));
    let library = cx.update(|cx| workspace.read(cx).library.clone());
    cx.update_window(window, |_, window, cx| {
        let sheet = crate::new_notebook_sheet::open(library, None, window, cx);
        sheet.update(cx, |sheet, cx| {
            sheet.set_kind(NotebookKind::VideoStudio, cx);
            sheet.set_name("Channel", window, cx);
        });
    })
    .unwrap();
}
