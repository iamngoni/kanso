//! Tidy for a voice note in the library (issue #193): the "Tidy voice note"
//! command, and the strip at the top of a voice note that offers it.
//!
//! Tidying reads the note's transcript (`kanso_engine::voice_tidy`), asks
//! the person's own AI on a worker, and writes the result in one change, so
//! Undo and the note's history bring the raw note back. A note that can't be
//! sent to AI (locked, in the trash, `#ai-exclude`) is refused before
//! anything is sent. A failed tidy changes nothing and says why.

use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use std::sync::atomic::AtomicBool;

use gpui_kit::component::WindowExt as _;
use gpui_kit::component::notification::Notification;
use gpui_kit::*;
use kanso_core::Note;
use kanso_core::capture_mark::{TidyState, read_tidy};
use kanso_engine::voice_tidy::{TidyError, apply_tidy, tidy_source};
use kanso_store::Origin;

use crate::ai_feature;
use crate::config::TidyMode;
use crate::workspace::AppState;

/// The notes being tidied now, and why the last try on a note failed.
#[derive(Default)]
pub struct TidyRuns {
    working: HashSet<String>,
    failed: HashMap<String, String>,
}

impl Global for TidyRuns {}

/// What the strip at the top of a note says.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Strip {
    /// Offer the Tidy button.
    Offer,
    Working,
    /// The last try failed; offer it again, saying why.
    Failed(String),
}

/// Whether, and how, to offer tidying `note`: it's a voice capture that
/// hasn't been tidied, AI is set up, Settings doesn't have tidying off, and
/// the note may be sent. Pure over its inputs.
pub fn strip_for(
    note: &Note,
    ai_on: bool,
    mode: TidyMode,
    in_trash: bool,
    working: bool,
    failed: Option<&str>,
) -> Option<Strip> {
    let state = read_tidy(&note.extra)?;
    if state == TidyState::Done || !ai_on || mode == TidyMode::Off || in_trash || note.is_locked() {
        return None;
    }
    if working {
        return Some(Strip::Working);
    }
    Some(match failed {
        Some(reason) => Strip::Failed(reason.to_owned()),
        None => Strip::Offer,
    })
}

/// The strip for note `note` right now, if any.
pub fn strip(note: &Note, cx: &App) -> Option<Strip> {
    let config = &cx.try_global::<AppState>()?.config;
    let runs = cx.try_global::<TidyRuns>();
    strip_for(
        note,
        config.ai.enabled,
        config.voice.tidy,
        note.book_id == kanso_core::model::TRASH_BOOK_ID,
        runs.is_some_and(|r| r.working.contains(&note.id)),
        runs.and_then(|r| r.failed.get(&note.id))
            .map(String::as_str),
    )
}

/// Tidies note `id`, in `window`'s notifications.
// no-prop-test: glue over `tidy_source`, `kanso_ai::tidy::tidy` and `apply_tidy`,
// which have the properties.
pub fn tidy_note(id: String, window: &mut Window, cx: &mut App) {
    let store = cx.global::<AppState>().store.clone();
    let meetings = cx.global::<AppState>().config.voice.meetings;
    let source = match tidy_source(&store, &id, meetings) {
        Ok(source) => source,
        Err(err) => {
            window.push_notification(Notification::error(err.to_string()), cx);
            return;
        }
    };
    if !cx.has_global::<TidyRuns>() {
        cx.set_global(TidyRuns::default());
    }
    {
        let runs = cx.global_mut::<TidyRuns>();
        if !runs.working.insert(id.clone()) {
            return;
        }
        runs.failed.remove(&id);
    }
    let maker = ai_feature::maker(cx);
    crate::metrics::record(cx, crate::metrics::Feature::Ai);
    let (tx, rx) = async_channel::bounded(1);
    let for_worker = source.clone();
    let spawned = std::thread::Builder::new()
        .name("kanso-tidy".into())
        .spawn(move || {
            let cancel = Arc::new(AtomicBool::new(false));
            tx.try_send(ai_feature::tidy(
                &maker,
                for_worker.kind,
                &for_worker.title,
                &for_worker.transcript,
                &cancel,
            ))
            .ok();
        });
    if spawned.is_err() {
        finish(&id, Err("Couldn't start.".to_owned()), window, cx);
        return;
    }
    let handle = window.window_handle();
    cx.spawn(async move |cx| {
        let result = rx
            .recv()
            .await
            .unwrap_or_else(|_| Err("Stopped.".to_owned()));
        cx.update(|cx| {
            let outcome = result.and_then(|tidy| {
                apply_tidy(&store, &source, &tidy, Origin::Ui).map_err(|err| match err {
                    TidyError::Changed => {
                        "The note changed while it was being tidied. Try again.".to_owned()
                    }
                    other => other.to_string(),
                })
            });
            let _ = handle.update(cx, |_, window, cx| finish(&id, outcome, window, cx));
        });
    })
    .detach();
    cx.refresh_windows();
}

fn finish(
    id: &str,
    outcome: Result<kanso_engine::voice_tidy::Applied, String>,
    window: &mut Window,
    cx: &mut App,
) {
    let runs = cx.global_mut::<TidyRuns>();
    runs.working.remove(id);
    match outcome {
        Ok(applied) => {
            runs.failed.remove(id);
            let mut words =
                String::from("Tidied. Undo, or the note's history, brings the original back.");
            if applied.inserted_above {
                words = "You'd edited the note, so the tidy version went above your text.".into();
            }
            if applied.dropped > 0 {
                words.push_str(&format!(
                    " {} left out for want of a quote from the recording.",
                    if applied.dropped == 1 {
                        "One item was".to_owned()
                    } else {
                        format!("{} items were", applied.dropped)
                    }
                ));
            }
            window.push_notification(Notification::info(words), cx);
        }
        Err(reason) => {
            runs.failed.insert(id.to_owned(), reason.clone());
            window.push_notification(
                Notification::error(format!("Couldn't tidy this note: {reason}")),
                cx,
            );
        }
    }
    cx.refresh_windows();
}

#[cfg(test)]
mod tests {
    use super::{Strip, strip_for};
    use crate::config::TidyMode;
    use kanso_core::Note;
    use proptest::prelude::*;
    use serde_json::json;

    fn note(tidy: &str, locked: bool) -> Note {
        let mut value = json!({"_id": "note:x", "title": "t", "body": "b", "bookId": "book:inbox",
            "kansoCapture": {"v": 1, "kind": "voice", "tidy": tidy}});
        if locked {
            value["kansoLock"] =
                json!({"v": 1, "salt": "c2FsdA", "m": 64, "t": 1, "p": 1, "data": "x"});
        }
        serde_json::from_value(value).unwrap()
    }

    fn plain_note() -> Note {
        serde_json::from_value(
            json!({"_id": "note:p", "title": "t", "body": "b", "bookId": "book:inbox"}),
        )
        .unwrap()
    }

    // no-prop-test: fixed notes with known answers; the property is below.
    #[test]
    fn the_strip_follows_the_note_and_the_settings() {
        let plain = plain_note();
        let cases: Vec<(
            Note,
            bool,
            TidyMode,
            bool,
            bool,
            Option<&str>,
            Option<Strip>,
        )> = vec![
            (
                note("pending", false),
                true,
                TidyMode::Ask,
                false,
                false,
                None,
                Some(Strip::Offer),
            ),
            (
                note("none", false),
                true,
                TidyMode::Always,
                false,
                false,
                None,
                Some(Strip::Offer),
            ),
            (
                note("done", false),
                true,
                TidyMode::Ask,
                false,
                false,
                None,
                None,
            ),
            (
                note("none", false),
                true,
                TidyMode::Off,
                false,
                false,
                None,
                None,
            ),
            (
                note("none", true),
                true,
                TidyMode::Ask,
                false,
                false,
                None,
                None,
            ),
            (
                note("none", false),
                false,
                TidyMode::Ask,
                false,
                false,
                None,
                None,
            ),
            (
                note("none", false),
                true,
                TidyMode::Ask,
                true,
                false,
                None,
                None,
            ),
            (
                note("none", false),
                true,
                TidyMode::Ask,
                false,
                true,
                Some("x"),
                Some(Strip::Working),
            ),
            (
                note("none", false),
                true,
                TidyMode::Ask,
                false,
                false,
                Some("no key"),
                Some(Strip::Failed("no key".into())),
            ),
            (plain, true, TidyMode::Ask, false, false, None, None),
        ];
        for (n, ai, mode, trash, working, failed, want) in cases {
            assert_eq!(strip_for(&n, ai, mode, trash, working, failed), want);
        }
    }

    proptest! {
        /// A note that is tidied, locked or in the trash, or with AI off,
        /// never gets a strip, whatever else is true.
        #[test]
        fn prop_nothing_offered_where_ai_may_not_go(
            (ai, trash, locked, working, state) in (
                any::<bool>(),
                any::<bool>(),
                any::<bool>(),
                any::<bool>(),
                prop::sample::select(vec!["none", "pending", "done"]),
            ),
        ) {
            let got = strip_for(&note(state, locked), ai, TidyMode::Ask, trash, working, None);
            if got.is_some() {
                prop_assert!(ai && !trash && !locked && state != "done");
            }
        }
    }
}
