//! Tidying a voice note (issue #193): reading the note's transcript, and
//! writing the tidy version back safely.
//!
//! The model call itself is `kanso_ai::tidy::tidy`, run by the app on a
//! worker. This module holds the rest: which note may be tidied
//! ([`tidy_source`]: a voice capture, not locked, not trashed, not
//! `#ai-exclude`), how the result reads ([`render_tidy`], grounded in the
//! transcript), and the one write that puts it in ([`apply_tidy`]), which
//! never overwrites what the person changed meanwhile.
//!
//! Issue #38's meeting summary calls the same pair: `tidy` with
//! `TidyKind::Meeting`, then [`render_tidy`].

mod quote;
mod render;

use std::fmt;

use kanso_ai::tidy::{Tidy, TidyKind};
use kanso_core::DocType;
use kanso_core::capture_mark::{CAPTURE_FIELD, Engine, TidyState, read_duration_ms, with_tidy};
use kanso_store::{Origin, Put, Store, StoreError};
use serde_json::json;

use crate::assist::{KEPT_OUT, note_for_ai};
use crate::voice_capture::{audio_file_id, is_meeting, voice_title};

pub use quote::{MIN_QUOTE, locate_quote, valid_quotes};
pub use render::{
    DETAILS_CLOSE, DETAILS_OPEN, RenderMeta, Rendered, Split, details_block, render_tidy,
    split_body,
};

/// Why a note wasn't tidied.
#[derive(Debug)]
pub enum TidyError {
    /// The note isn't there.
    Missing,
    /// The note isn't a voice capture.
    NotVoice,
    /// Locked, trashed or `#ai-exclude`: [`KEPT_OUT`].
    KeptOut,
    /// The note was changed while the model worked and the write was
    /// refused. Try again.
    Changed,
    Store(StoreError),
}

impl fmt::Display for TidyError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TidyError::Missing => f.write_str("That note isn't there any more."),
            TidyError::NotVoice => f.write_str("Only voice notes can be tidied."),
            TidyError::KeptOut => f.write_str(KEPT_OUT),
            TidyError::Changed => f.write_str("The note changed while it was being tidied."),
            TidyError::Store(err) => err.fmt(f),
        }
    }
}

impl std::error::Error for TidyError {}

impl From<StoreError> for TidyError {
    fn from(err: StoreError) -> TidyError {
        TidyError::Store(err)
    }
}

/// What a tidy-up starts from, read once so the write can tell whether the
/// note moved on.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TidySource {
    pub id: String,
    pub title: String,
    /// The body as it was read; the write is compared to it.
    pub body: String,
    /// The words that were said (what the model reads).
    pub transcript: String,
    pub kind: TidyKind,
    pub duration_ms: u64,
    pub on_device: bool,
    pub started_ms: i64,
}

/// The audio link a capture's body may end with, if its file is `cap`'s.
fn audio_tail(note_id: &str, body: &str) -> Option<(String, String)> {
    let cap = note_id.strip_prefix("note:cap-")?;
    let link = format!("(inkdrop://{})", audio_file_id(cap));
    let at = body.rfind("\n\n")?;
    let last = body[at + 2..].trim();
    (last.starts_with('[') && last.ends_with(&link))
        .then(|| (body[..at].to_owned(), last.to_owned()))
}

/// Reads note `id` for tidying. Meetings are voice notes longer than three
/// minutes when `meetings` is on.
pub fn tidy_source(store: &Store, id: &str, meetings: bool) -> Result<TidySource, TidyError> {
    let note = store.note(id)?.ok_or(TidyError::Missing)?;
    let mark = note.extra.get(CAPTURE_FIELD).and_then(|m| m.as_object());
    let Some(mark) = mark.filter(|m| m.get("kind").and_then(|k| k.as_str()) == Some("voice"))
    else {
        return Err(TidyError::NotVoice);
    };
    let Some(candidate) = note_for_ai(store, id) else {
        return Err(TidyError::KeptOut);
    };
    let duration_ms = read_duration_ms(&note.extra).unwrap_or(0);
    let on_device = mark
        .get("engine")
        .and_then(|e| e.as_str())
        .and_then(Engine::from_word)
        .is_none_or(Engine::on_device);
    let transcript = match split_body(&candidate.body) {
        Some(split) => split.transcript,
        None => match audio_tail(&note.id, &candidate.body) {
            Some((text, _)) => text,
            None => candidate.body.clone(),
        },
    };
    Ok(TidySource {
        id: note.id,
        title: candidate.title,
        body: candidate.body,
        transcript,
        kind: if is_meeting(duration_ms, meetings) {
            TidyKind::Meeting
        } else {
            TidyKind::Note
        },
        duration_ms,
        on_device,
        started_ms: note.created_at,
    })
}

/// A day as a meeting heading says it: "2 Nov 2026", in local time.
pub fn date_label(ms: i64) -> Option<String> {
    use chrono::TimeZone as _;
    chrono::Local
        .timestamp_millis_opt(ms)
        .single()
        .map(|d| d.format("%-d %b %Y").to_string())
}

/// The renderer's settings for `source`.
pub fn meta_for(source: &TidySource) -> RenderMeta {
    RenderMeta {
        kind: source.kind,
        duration_ms: source.duration_ms,
        on_device: source.on_device,
        date: date_label(source.started_ms),
        fallback_title: voice_title(&source.transcript),
    }
}

/// What [`apply_tidy`] did.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Applied {
    pub id: String,
    /// The note had been edited since it was read, so the tidy part was
    /// put above the person's text instead of replacing it.
    pub inserted_above: bool,
    /// Decisions and tasks left out for want of a quote.
    pub dropped: usize,
}

/// Writes `tidy` into the note `source` was read from, in one `put`
/// (`Origin::Ui`), so Undo and the note's history bring the raw note back.
///
/// If the note is as it was read, its body becomes the tidy part and the
/// folded transcript (and its audio link, if any), and its title the tidy
/// title unless the person renamed it. If the person edited it meanwhile,
/// nothing of theirs is replaced: the tidy part goes above their text.
/// Refused ([`TidyError::KeptOut`]) if the note was locked, trashed or
/// excluded meanwhile.
pub fn apply_tidy(
    store: &Store,
    source: &TidySource,
    tidy: &Tidy,
    origin: Origin,
) -> Result<Applied, TidyError> {
    let candidate = note_for_ai(store, &source.id).ok_or(TidyError::KeptOut)?;
    let note = store.note(&source.id)?.ok_or(TidyError::Missing)?;
    let rendered = render_tidy(tidy, &source.transcript, &meta_for(source));
    let unchanged = candidate.body == source.body;
    let (body, title) = if unchanged {
        let mut body = rendered.body();
        if let Some((_, link)) = audio_tail(&source.id, &source.body) {
            body.push_str("\n\n");
            body.push_str(&link);
        }
        let title = if candidate.title == source.title {
            rendered.title.clone()
        } else {
            candidate.title.clone()
        };
        (body, title)
    } else {
        (
            format!("{}\n\n{}", rendered.above, candidate.body),
            candidate.title.clone(),
        )
    };
    let mark = note
        .extra
        .get(CAPTURE_FIELD)
        .map(|m| with_tidy(m, TidyState::Done))
        .unwrap_or_else(|| json!({"v": 1, "kind": "voice", "tidy": "done"}));
    let result = store.put(
        DocType::Note,
        json!({
            "_id": source.id,
            "_rev": note.rev,
            "title": title,
            "body": body,
            CAPTURE_FIELD: mark,
        }),
        Put::Merge,
        origin,
    );
    match result {
        Ok(_) => Ok(Applied {
            id: source.id.clone(),
            inserted_above: !unchanged,
            dropped: rendered.dropped,
        }),
        Err(StoreError::Conflict) => Err(TidyError::Changed),
        Err(err) => Err(err.into()),
    }
}

#[cfg(test)]
mod tests;
