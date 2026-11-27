//! Snapshot states for New Notebook (issues #390 slice 29 and #514): the
//! name and the starters, with Product development chosen and a name typed,
//! in both themes.

use gpui_kit::{AnyWindowHandle, Entity, HeadlessAppContext};
use kanso_engine::presets::NotebookKind;
use kanso_store::Store;

use crate::new_notebook_sheet::SHEET_WIDTH;
use crate::workspace::Workspace;

/// The shots: name (`dark-…` or `light-…`), how to set the window up, and
/// the width of the dialog it opens.
pub const SHOTS: &[(&str, super::Prepare, Option<f32>)] = &[
    ("dark-new-notebook", product_chosen, Some(SHEET_WIDTH)),
    ("light-new-notebook", product_chosen, Some(SHEET_WIDTH)),
];

/// Product development chosen, "Acme" typed.
fn product_chosen(
    cx: &mut HeadlessAppContext,
    window: AnyWindowHandle,
    workspace: &Entity<Workspace>,
    _: &Store,
) {
    // Without it the dialog is drawn mid slide-in.
    cx.update(|cx| cx.set_reduce_motion(true));
    let (library, sidebar) = cx.update(|cx| {
        let workspace = workspace.read(cx);
        (workspace.library.clone(), workspace.sidebar.clone())
    });
    cx.update_window(window, |_, window, cx| {
        let sheet = crate::new_notebook_sheet::open(library, sidebar, None, window, cx);
        sheet.update(cx, |sheet, cx| {
            sheet.set_kind(NotebookKind::Product, cx);
            sheet.set_name("Acme", window, cx);
        });
    })
    .unwrap();
}
