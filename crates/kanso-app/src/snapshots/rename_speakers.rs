//! Snapshot states for telling speakers apart (issue #38, step 13): the
//! Rename Speakers sheet, and a transcription on its second stage, working
//! out who said the words. The names are fixed, so the shots don't depend
//! on a model.

use gpui_kit::{AnyWindowHandle, Entity, HeadlessAppContext};
use kanso_engine::recording_speakers::SpeakerName;
use kanso_store::Store;

use crate::workspace::Workspace;

/// The shots: name (`dark-…` or `light-…`), how to set the window up, and
/// the width of the dialog it opens.
pub const SHOTS: &[(&str, super::Prepare, Option<f32>)] = &[
    ("dark-rename-speakers", sheet, Some(WIDTH)),
    ("light-rename-speakers", sheet, Some(WIDTH)),
];

const WIDTH: f32 = 440.;

fn name(name: &str, paragraphs: u32) -> SpeakerName {
    SpeakerName {
        name: name.into(),
        paragraphs,
    }
}

/// Three names: the microphone, and two voices.
fn sheet(
    cx: &mut HeadlessAppContext,
    window: AnyWindowHandle,
    workspace: &Entity<Workspace>,
    _: &Store,
) {
    // Without it the dialog is drawn mid slide-in.
    cx.update(|cx| cx.set_reduce_motion(true));
    let editor = cx.update(|cx| workspace.read(cx).editor.clone());
    cx.update_window(window, |_, window, cx| {
        crate::rename_speakers::open_for_snapshot(
            editor,
            vec![name("You", 12), name("Speaker 1", 31), name("Speaker 2", 1)],
            window,
            cx,
        );
    })
    .unwrap();
}

/// A recording whose words are in, now being listened to for who said them.
pub(super) fn editor_telling(
    cx: &mut HeadlessAppContext,
    window: AnyWindowHandle,
    workspace: &Entity<Workspace>,
    _: &Store,
) {
    cx.update(|cx| {
        let editor = workspace.read(cx).editor.clone();
        crate::transcribe::show_telling_for_snapshot(
            &editor,
            window,
            "Standup 21 Oct.m4a",
            0.62,
            cx,
        );
    });
}
