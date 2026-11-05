//! Snapshot states for Summarise Recording (issue #38, step 12): the sheet
//! that says what is sent, the review of a summary with its action items
//! ticked, and a request that failed. The words are fixed, so the shots
//! don't depend on a model.

use gpui_kit::{AnyWindowHandle, Entity, HeadlessAppContext};
use kanso_engine::recording_summary::{SummaryDraft, SummaryItem, SummarySource};
use kanso_store::Store;

use crate::recording_summary::Phase;
use crate::workspace::Workspace;

/// The shots: name (`dark-…` or `light-…`), how to set the window up, and
/// the width of the dialog it opens.
pub const SHOTS: &[(&str, super::Prepare, Option<f32>)] = &[
    ("dark-recording-summary-send", send, Some(WIDTH)),
    ("light-recording-summary-send", send, Some(WIDTH)),
    ("dark-recording-summary-review", review, Some(WIDTH)),
    ("light-recording-summary-review", review, Some(WIDTH)),
    ("dark-recording-summary-failed", failed, Some(WIDTH)),
    ("light-recording-summary-failed", failed, Some(WIDTH)),
];

const WIDTH: f32 = crate::recording_summary::WIDTH;

fn source() -> SummarySource {
    SummarySource {
        id: "note:snapshot".into(),
        title: "Meeting — 23 Oct 2026, 14:05".into(),
        file_id: "file:abc123".into(),
        date: Some("23 Oct 2026".into()),
        segments: Vec::new(),
        said: String::new(),
        prompt_text: String::new(),
        words: 8_412,
        replaces: false,
    }
}

fn item(text: &str, who: Option<&str>, when: Option<&str>, moment: u32) -> SummaryItem {
    SummaryItem {
        text: text.into(),
        who: who.map(Into::into),
        when: when.map(Into::into),
        moment,
    }
}

fn draft() -> SummaryDraft {
    SummaryDraft {
        file_id: "file:abc123".into(),
        overview: vec![
            "The beta ships on the 6th.".into(),
            "Onboarding is the open risk.".into(),
            "Priya owns the beta checklist.".into(),
        ],
        decisions: vec![item("Ship the beta on the 6th", None, None, 22)],
        actions: vec![
            item(
                "Send the onboarding draft",
                Some("Priya"),
                Some("by the 30th"),
                22,
            ),
            item("Book the review with design", Some("Tendai"), None, 1_340),
            item("Update the roadmap page", None, Some("Friday"), 2_075),
        ],
        dropped: 1,
        cut: false,
    }
}

const SEND: &str = "Send this recording's transcript (8412 words), with the note's title and the date, to api.openai.com? The audio is not sent. The transcript is. api.openai.com handles it under its own retention rules.";

fn open(
    cx: &mut HeadlessAppContext,
    window: AnyWindowHandle,
    phase: Phase,
    draft: Option<SummaryDraft>,
) {
    // Without it the dialog is drawn mid slide-in.
    cx.update(|cx| cx.set_reduce_motion(true));
    cx.update_window(window, |_, window, cx| {
        crate::recording_summary::open_for_snapshot(
            phase,
            source(),
            draft,
            SEND.to_owned(),
            window,
            cx,
        );
    })
    .unwrap();
}

/// Before anything is sent.
fn send(cx: &mut HeadlessAppContext, window: AnyWindowHandle, _: &Entity<Workspace>, _: &Store) {
    open(cx, window, Phase::Confirm, None);
}

/// The model answered: the summary, a decision and three action items.
fn review(cx: &mut HeadlessAppContext, window: AnyWindowHandle, _: &Entity<Workspace>, _: &Store) {
    open(cx, window, Phase::Review, Some(draft()));
}

/// The request failed.
fn failed(cx: &mut HeadlessAppContext, window: AnyWindowHandle, _: &Entity<Workspace>, _: &Store) {
    open(
        cx,
        window,
        Phase::Failed("The reply wasn't understood.".into()),
        None,
    );
}
