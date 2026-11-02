//! Snapshot states for the Ask panel: an answer with receipts, its sources
//! and the notes it also read, a sentence that failed its check, the banner
//! over an answer that could not be checked, and "You never wrote that
//! down" with the closest notes.

use gpui_kit::{AnyWindowHandle, Entity, HeadlessAppContext};
use kanso_ai::SentExcerpt;
use kanso_engine::ask::{AskScope, Closest, Retrieval, Shown, TimeNote, finalize};
use kanso_store::Store;

use crate::config::InspectorTab;
use crate::workspace::{AppState, Workspace};

fn shown() -> std::sync::Arc<Shown> {
    let shown = Shown::new();
    shown.register(SentExcerpt {
        number: 1,
        id: "note:pricing".into(),
        title: "Pricing decision".into(),
        updated: Some("2026-10-14".into()),
        lines: vec![
            (2, "We chose the yearly plan at 40 dollars.".into()),
            (3, "Sam argued the monthly plan churned too fast.".into()),
        ],
    });
    shown.register(SentExcerpt {
        number: 2,
        id: "note:call".into(),
        title: "Call with Priya".into(),
        updated: Some("2026-10-09".into()),
        lines: vec![(1, "Priya approved the launch price.".into())],
    });
    shown
}

fn open_ask(
    cx: &mut HeadlessAppContext,
    workspace: &Entity<Workspace>,
    exchanges: Vec<(&'static str, String, bool)>,
    scope: AskScope,
) {
    cx.update(|cx| {
        cx.global_mut::<AppState>().config.ai.enabled = true;
        workspace.update(cx, |workspace, cx| {
            workspace.show_inspector(Some(InspectorTab::Ask), cx);
            workspace.inspector.update(cx, |inspector, cx| {
                inspector.snapshot_ask(exchanges, scope, cx)
            });
        });
    });
}

/// Builds the answers the panel shows: the question, the model's words, and
/// whether the closest notes are attached.
pub(super) fn answer_with_receipts(
    cx: &mut HeadlessAppContext,
    _: AnyWindowHandle,
    workspace: &Entity<Workspace>,
    _: &Store,
) {
    open_ask(
        cx,
        workspace,
        vec![(
            "What did we decide about pricing?",
            "We chose the yearly plan at 40 dollars [1:L3]. Sam argued the monthly plan churned too fast [1:L4]. The price will double next year [1:L90].".to_owned(),
            false,
        )],
        AskScope::All,
    );
}

pub(super) fn never_wrote(
    cx: &mut HeadlessAppContext,
    _: AnyWindowHandle,
    workspace: &Entity<Workspace>,
    _: &Store,
) {
    open_ask(
        cx,
        workspace,
        vec![(
            "What is my passport number?",
            "NOT_IN_NOTES".to_owned(),
            true,
        )],
        AskScope::All,
    );
}

pub(super) fn no_sources_banner(
    cx: &mut HeadlessAppContext,
    _: AnyWindowHandle,
    workspace: &Entity<Workspace>,
    _: &Store,
) {
    open_ask(
        cx,
        workspace,
        vec![(
            "Who approved the price?",
            "Priya approved the launch price on 14 October.".to_owned(),
            false,
        )],
        AskScope::All,
    );
}

/// What the panel keeps for one shown exchange, built from the model's raw
/// words the way a finished question builds it.
pub(crate) fn finished_for(raw: &str, closest: bool) -> kanso_engine::ask::Finished {
    let retrieval = Retrieval {
        closest: if closest {
            vec![Closest {
                note_id: "note:pricing".into(),
                title: "Pricing decision".into(),
                start: 2,
                end: 3,
            }]
        } else {
            Vec::new()
        },
        time: Some(TimeNote {
            label: "October 2026".into(),
            intent: kanso_core::when::TimeIntent::Written(kanso_core::when::DateRange {
                start: kanso_core::dates::Date::new(2026, 10, 1),
                end: kanso_core::dates::Date::new(2026, 10, 31),
            }),
            relaxed: false,
        }),
        ..Retrieval::default()
    };
    finalize(raw, &shown(), true, &retrieval)
}
