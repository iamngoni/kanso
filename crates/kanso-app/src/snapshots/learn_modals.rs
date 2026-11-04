//! Snapshot states for Quiz Me and Make Flashcards as dialogs over the
//! note: a question waiting for its answer, the same question graded, the
//! suggested cards, and a card being edited.

use gpui_kit::{AnyWindowHandle, Entity, HeadlessAppContext};
use kanso_store::Store;

use crate::learn_model::Mode;
use crate::workspace::{AppState, Workspace};

fn open(
    cx: &mut HeadlessAppContext,
    window: AnyWindowHandle,
    workspace: &Entity<Workspace>,
    mode: Mode,
    second: bool,
) {
    cx.update(|cx| {
        cx.set_reduce_motion(true);
        cx.global_mut::<AppState>().config.ai.enabled = true;
    });
    cx.update_window(window, |_, window, cx| {
        let id = workspace.read(cx).library.read(cx).selected.clone();
        let view = workspace.update(cx, |workspace, cx| {
            workspace.show_learn_modal(mode, id.unwrap(), window, cx)
        });
        let view = view.expect("the dialog opens");
        view.update(cx, |view, cx| view.snapshot_state(second, window, cx));
    })
    .unwrap();
}

pub(super) fn quiz_question(
    cx: &mut HeadlessAppContext,
    window: AnyWindowHandle,
    workspace: &Entity<Workspace>,
    _: &Store,
) {
    open(cx, window, workspace, Mode::Quiz, false);
}

pub(super) fn quiz_graded(
    cx: &mut HeadlessAppContext,
    window: AnyWindowHandle,
    workspace: &Entity<Workspace>,
    _: &Store,
) {
    open(cx, window, workspace, Mode::Quiz, true);
}

pub(super) fn cards(
    cx: &mut HeadlessAppContext,
    window: AnyWindowHandle,
    workspace: &Entity<Workspace>,
    _: &Store,
) {
    open(cx, window, workspace, Mode::Make, false);
}

pub(super) fn cards_editing(
    cx: &mut HeadlessAppContext,
    window: AnyWindowHandle,
    workspace: &Entity<Workspace>,
    _: &Store,
) {
    open(cx, window, workspace, Mode::Make, true);
}
