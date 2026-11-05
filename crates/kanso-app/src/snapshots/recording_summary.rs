//! Snapshot states for Summarise Recording (issue #38, step 12): the sheet
//! that says what is sent, the review of a summary with its action items
//! ticked, and a request that failed. The words are fixed, so the shots
//! don't depend on a model.

use gpui_kit::{AnyWindowHandle, Entity, HeadlessAppContext};
use kanso_engine::recording_summary::{SummaryDraft, SummaryItem, SummarySource, send_words};
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
    ("dark-recording-summary-send-agent", send_agent, Some(WIDTH)),
    (
        "light-recording-summary-send-agent",
        send_agent,
        Some(WIDTH),
    ),
    ("dark-recording-summary-send-local", send_local, Some(WIDTH)),
    (
        "light-recording-summary-send-local",
        send_local,
        Some(WIDTH),
    ),
    ("dark-recording-summary-review-cut", review_cut, Some(WIDTH)),
    (
        "light-recording-summary-review-cut",
        review_cut,
        Some(WIDTH),
    ),
    ("dark-recording-summary-no-key", no_key, Some(WIDTH)),
    ("light-recording-summary-no-key", no_key, Some(WIDTH)),
    ("dark-recording-summary-no-agent", no_agent, Some(WIDTH)),
    ("light-recording-summary-no-agent", no_agent, Some(WIDTH)),
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
        cut: false,
        recordings: 1,
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

/// What the Confirm phase says on an API key, as the engine words it.
fn send_keyed(source: &SummarySource) -> String {
    send_words(
        source,
        kanso_ai::Provider::OpenAi,
        "https://api.openai.com/v1",
        None,
    )
}

fn open(
    cx: &mut HeadlessAppContext,
    window: AnyWindowHandle,
    phase: Phase,
    draft: Option<SummaryDraft>,
) {
    open_with(cx, window, phase, source(), draft, send_keyed(&source()));
}

fn open_with(
    cx: &mut HeadlessAppContext,
    window: AnyWindowHandle,
    phase: Phase,
    source: SummarySource,
    draft: Option<SummaryDraft>,
    words: String,
) {
    // Without it the dialog is drawn mid slide-in.
    cx.update(|cx| cx.set_reduce_motion(true));
    cx.update_window(window, |_, window, cx| {
        crate::recording_summary::open_for_snapshot(phase, source, draft, words, window, cx);
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

/// A long meeting with a second recording, on an agent: the sheet says the
/// transcript is cut, which recording goes, and that the agent passes it on.
fn send_agent(
    cx: &mut HeadlessAppContext,
    window: AnyWindowHandle,
    _: &Entity<Workspace>,
    _: &Store,
) {
    let source = SummarySource {
        cut: true,
        recordings: 2,
        words: 31_870,
        ..source()
    };
    let words = send_words(&source, kanso_ai::Provider::OpenAi, "", Some("Claude Code"));
    open_with(cx, window, Phase::Confirm, source, None, words);
}

/// A model on the person's own network: it goes only to that model.
fn send_local(
    cx: &mut HeadlessAppContext,
    window: AnyWindowHandle,
    _: &Entity<Workspace>,
    _: &Store,
) {
    let words = send_words(
        &source(),
        kanso_ai::Provider::LocalModel,
        "http://127.0.0.1:11434/v1",
        None,
    );
    open_with(cx, window, Phase::Confirm, source(), None, words);
}

/// The review of a transcript that was cut, which says so.
fn review_cut(
    cx: &mut HeadlessAppContext,
    window: AnyWindowHandle,
    _: &Entity<Workspace>,
    _: &Store,
) {
    let draft = SummaryDraft {
        cut: true,
        ..draft()
    };
    open(cx, window, Phase::Review, Some(draft));
}

/// An API key that isn't saved, in the words the sheet gives.
fn no_key(cx: &mut HeadlessAppContext, window: AnyWindowHandle, _: &Entity<Workspace>, _: &Store) {
    let reason = crate::capture_voice::trouble_words(&kanso_ai::AiError::MissingKey.to_string());
    open(cx, window, Phase::Failed(reason), None);
}

/// An agent that was uninstalled, in the words the sheet gives.
fn no_agent(
    cx: &mut HeadlessAppContext,
    window: AnyWindowHandle,
    _: &Entity<Workspace>,
    _: &Store,
) {
    let reason = crate::capture_voice::trouble_words(
        "Kanso can't find claude-code-acp on this computer. Settings › AI has the agent's command.",
    );
    open(cx, window, Phase::Failed(reason), None);
}
