//! Summarise Recording on the phone (issue #38, step 12): the same flow as
//! the desktop's sheet behind the bridge. The shell asks
//! [`AiHandle::summary_offer`] to know whether to show the action and what
//! to say before sending, [`AiHandle::summarize_recording`] to ask the
//! person's own AI (nothing is written), and
//! [`AiHandle::apply_recording_summary`] once the person has ticked the
//! action items they want, which writes in one change with an Undo ticket.
//!
//! The rules are the engine's (`kanso_engine::recording_summary`). The
//! transcript, the note's title and its date are all that is sent: no audio,
//! no other note.

use std::collections::BTreeSet;
use std::sync::atomic::Ordering;

use kanso_ai::recording_summary::summarize_recording;
use kanso_engine::recording_summary::{
    SummaryDraft, SummaryError, SummaryItem, apply_summary, ground, send_line, summary_source,
};
use kanso_store::Origin;

use crate::ai::{AiHandle, AiRun};
use crate::boundary::{guard, held};
use crate::errors::KansoError;
use crate::recording_summary_model::{
    SummaryApplied, SummaryDraftView, SummaryItemView, SummaryOffer,
};

fn map(err: SummaryError) -> KansoError {
    match err {
        SummaryError::Missing => KansoError::NotFound,
        other => KansoError::invalid(other.to_string()),
    }
}

fn view(item: &SummaryItem) -> SummaryItemView {
    SummaryItemView {
        text: item.text.clone(),
        who: item.who.clone(),
        when: item.when.clone(),
        moment: item.moment,
    }
}

fn item(view: &SummaryItemView) -> SummaryItem {
    SummaryItem {
        text: view.text.clone(),
        who: view.who.clone(),
        when: view.when.clone(),
        moment: view.moment,
    }
}

fn draft_view(draft: &SummaryDraft) -> SummaryDraftView {
    SummaryDraftView {
        file_id: draft.file_id.clone(),
        overview: draft.overview.clone(),
        decisions: draft.decisions.iter().map(view).collect(),
        actions: draft.actions.iter().map(view).collect(),
        dropped: draft.dropped as u32,
        cut: draft.cut,
    }
}

fn draft_of(view: &SummaryDraftView) -> SummaryDraft {
    SummaryDraft {
        file_id: view.file_id.clone(),
        overview: view.overview.clone(),
        decisions: view.decisions.iter().map(item).collect(),
        actions: view.actions.iter().map(item).collect(),
        dropped: view.dropped as usize,
        cut: view.cut,
    }
}

impl AiHandle {
    /// Whether Summarise Recording is offered for note `id`: AI is on, the
    /// note holds a recording's transcript and may be sent. Reads the note,
    /// sends nothing. Never an error for a note that is simply not
    /// summarisable (locked, trashed, `#ai-exclude`, no transcript): it is
    /// not offered.
    pub fn summary_offer(&self, id: &str) -> Result<SummaryOffer, KansoError> {
        guard(|| {
            let none = SummaryOffer {
                available: false,
                words: 0,
                send_line: String::new(),
                replaces: false,
            };
            let Ok(settings) = self.enabled() else {
                return Ok(none);
            };
            let Ok(source) = summary_source(&self.inner.store, id) else {
                return Ok(none);
            };
            Ok(SummaryOffer {
                available: true,
                words: source.words as u32,
                send_line: send_line(source.words, settings.provider, &settings.base_url, None),
                replaces: source.replaces,
            })
        })
    }

    /// Asks the person's own AI to summarise note `id`'s recording. Blocks
    /// until it has answered; `run` stops it. Refuses a note kept out of AI
    /// before anything is sent. Nothing is written: the draft holds only
    /// the decisions and action items the transcript backs.
    pub fn summarize_recording(
        &self,
        id: &str,
        run: &AiRun,
    ) -> Result<SummaryDraftView, KansoError> {
        guard(|| {
            let settings = self.enabled()?;
            let source = summary_source(&self.inner.store, id).map_err(map)?;
            let provider = self.provider_for(&settings)?;
            self.inner.metrics().count(crate::metrics::Feature::Ai);
            let result = {
                let provider = held(&provider);
                summarize_recording(
                    provider.as_ref(),
                    &source.title,
                    source.date.as_deref(),
                    &source.prompt_text,
                    &run.cancel_flag(),
                )
            };
            if run.cancel_flag().load(Ordering::Relaxed) {
                return Err(KansoError::invalid("Stopped."));
            }
            let tidy = result.map_err(KansoError::invalid)?;
            let draft = ground(&source, &tidy);
            if draft.overview.is_empty() && draft.decisions.is_empty() && draft.actions.is_empty() {
                return Err(map(SummaryError::Empty));
            }
            Ok(draft_view(&draft))
        })
    }

    /// Writes `draft`, with the action items at the positions in `ticked`,
    /// above the transcript in note `id`, in one change. The note is read
    /// again first: it must still be fit for AI and still hold the
    /// transcript, and a note edited meanwhile is refused, not overwritten.
    pub fn apply_recording_summary(
        &self,
        id: &str,
        draft: SummaryDraftView,
        ticked: Vec<u32>,
    ) -> Result<SummaryApplied, KansoError> {
        guard(|| {
            let keep: BTreeSet<usize> = ticked.into_iter().map(|t| t as usize).collect();
            let applied =
                apply_summary(&self.inner.store, id, &draft_of(&draft), &keep, Origin::Ui)
                    .map_err(map)?;
            let undo =
                held(&self.inner.undo).keep(crate::undo::Undoable::Fields(vec![applied.edit]));
            Ok(SummaryApplied {
                replaced: applied.replaced,
                undo: Some(undo),
            })
        })
    }
}
