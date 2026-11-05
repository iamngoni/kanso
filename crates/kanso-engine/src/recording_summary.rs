//! A recording's summary and action items (issue #38, step 12): reading the
//! transcript a recording left in its note, checking what the model says
//! against it, and writing the part the person approved above the
//! transcript.
//!
//! The model call itself is `kanso_ai::recording_summary`, run by the app
//! or the phone on a worker. This module is the rest, and it is pure over a
//! store:
//!
//! - [`summary_source`] reads the note: it must hold a recording's marked
//!   transcript and may be sent to AI (not locked, not trashed, not
//!   `#ai-exclude`). It gives the transcript back as the model reads it,
//!   with each paragraph's time in brackets.
//! - [`ground`] keeps a decision or an action item only if a quote it
//!   carries is really in the transcript, and links it to the moment the
//!   quote was said.
//! - [`render_summary`] writes the summary as Markdown between
//!   `kanso:summary` markers, with only the action items the person ticked.
//! - [`apply_summary`] puts that above the transcript in one write, so Undo
//!   brings the note back. A second summary replaces the first, markers to
//!   markers, and never touches anything outside them.

use std::collections::BTreeSet;
use std::ops::Range;

use kanso_ai::tidy::{MAX_CHARS, Tidy};
use kanso_core::DocType;
use kanso_store::{Origin, Put, Store, StoreError};
use serde_json::json;

use crate::ask::strip_remote_images;
use crate::assist::note_for_ai;
use crate::recording_note::{
    AUDIO_SCHEME, TRANSCRIPT_OPEN, moment_link, parse_moment, stamp, transcript_range,
};
use crate::voice_tidy::{date_label, valid_quotes};

pub use crate::recording_summary_model::{
    Applied, SummaryDraft, SummaryError, SummaryItem, SummarySegment, SummarySource,
};

/// Opens the summary.
pub const SUMMARY_OPEN: &str = "<!-- kanso:summary -->";
/// Closes the summary.
pub const SUMMARY_CLOSE: &str = "<!-- /kanso:summary -->";
/// The line that says the words are the model's.
pub const AI_LABEL: &str =
    "*Summarised by AI from the transcript below. Check it against the recording.*";
/// The longest an item or a point of the summary is, in characters.
pub const ITEM_MAX: usize = 400;
/// The most of each kind a summary keeps.
pub const ITEMS_MAX: usize = 50;

/// One paragraph of a transcript: its moment and its words, or `None` for a
/// line that doesn't start with a moment link.
fn parse_line(line: &str) -> Option<(u32, String)> {
    let line = line.trim();
    if !line.starts_with('[') {
        return None;
    }
    let after = &line[line.find("](")? + 2..];
    let end = after.find(')')?;
    let moment = parse_moment(&after[..end])?;
    Some((moment.seconds, after[end + 1..].trim().to_owned()))
}

/// The recording whose marked transcript comes first in `body`, and where
/// that block is. Pure.
pub fn find_recording_transcript(body: &str) -> Option<(String, Range<usize>)> {
    recording_transcripts(body).into_iter().next()
}

/// Every recording's marked transcript in `body`, in order, with where its
/// block is. Pure.
pub fn recording_transcripts(body: &str) -> Vec<(String, Range<usize>)> {
    let mut found = Vec::new();
    let mut from = 0;
    while let Some(open) = body[from..].find(TRANSCRIPT_OPEN).map(|at| at + from) {
        let inner = open + TRANSCRIPT_OPEN.len();
        let first = body[inner..]
            .lines()
            .map(str::trim)
            .find(|line| !line.is_empty())
            .filter(|line| line.contains(AUDIO_SCHEME))
            .and_then(|line| {
                let after = &line[line.find("](")? + 2..];
                parse_moment(&after[..after.find(')')?])
            });
        if let Some(moment) = first
            && let Some(range) = transcript_range(body, &moment.file_id)
            && range.start == open
        {
            from = range.end;
            found.push((moment.file_id, range));
            continue;
        }
        from = inner;
    }
    found
}

/// Where the summary block is in `body`: from its opening marker to the end
/// of its closing one. An opening marker with another opening one before
/// its close is skipped. Pure.
pub fn summary_range(body: &str) -> Option<Range<usize>> {
    let mut from = 0;
    while let Some(open) = body[from..].find(SUMMARY_OPEN).map(|at| at + from) {
        let inner = open + SUMMARY_OPEN.len();
        let close = body[inner..].find(SUMMARY_CLOSE)? + inner;
        if let Some(nested) = body[inner..close].find(SUMMARY_OPEN) {
            from = inner + nested;
            continue;
        }
        return Some(open..close + SUMMARY_CLOSE.len());
    }
    None
}

/// The paragraphs of `body`'s first recording transcript, in order, as the
/// source holds them. Pure.
fn segments_of(block: &str) -> (Vec<SummarySegment>, String) {
    let mut segments = Vec::new();
    let mut said = String::new();
    let mut chars = 0;
    for line in block.lines() {
        let Some((start_secs, text)) = parse_line(line) else {
            continue;
        };
        if text.is_empty() {
            continue;
        }
        if !said.is_empty() {
            said.push('\n');
            chars += 1;
        }
        segments.push(SummarySegment {
            start_secs,
            text: text.clone(),
            from: chars,
        });
        chars += text.chars().count();
        said.push_str(&text);
    }
    (segments, said)
}

/// What the model reads: each paragraph behind its time in brackets. Pure.
pub fn prompt_text(segments: &[SummarySegment]) -> String {
    segments
        .iter()
        .map(|s| format!("[{}] {}", stamp(s.start_secs), s.text))
        .collect::<Vec<_>>()
        .join("\n\n")
}

/// Whether `body` holds a transcript a summary can be made from. Pure.
pub fn has_transcript(body: &str) -> bool {
    find_recording_transcript(body).is_some_and(|(_, range)| {
        !segments_of(&body[range.start + TRANSCRIPT_OPEN.len()..range.end])
            .0
            .is_empty()
    })
}

/// The words of `prompt` that are sent, which stops at the cap: the times
/// in brackets are not counted. Pure.
fn words_sent(prompt: &str) -> usize {
    let sent: String = prompt.chars().take(MAX_CHARS).collect();
    sent.split_whitespace()
        .filter(|word| !(word.starts_with('[') && word.ends_with(']')))
        .count()
}

/// Reads note `id` for summarising: it holds a recording's transcript and
/// may be sent to AI. The gate comes first: nothing of a note that is
/// locked, trashed or tagged `#ai-exclude` is read.
pub fn summary_source(store: &Store, id: &str) -> Result<SummarySource, SummaryError> {
    let Some(candidate) = note_for_ai(store, id) else {
        // Only which refusal to give; nothing of the note is used.
        return Err(match store.note_is_locked(id) {
            Ok(false) if store.note(id)?.is_none() => SummaryError::Missing,
            _ => SummaryError::KeptOut,
        });
    };
    let recordings = recording_transcripts(&candidate.body);
    let (file_id, range) = recordings
        .first()
        .cloned()
        .ok_or(SummaryError::NoTranscript)?;
    let (segments, said) = segments_of(&candidate.body[range]);
    if segments.is_empty() {
        return Err(SummaryError::NoTranscript);
    }
    let prompt_text = prompt_text(&segments);
    Ok(SummarySource {
        id: candidate.id,
        title: candidate.title,
        file_id,
        date: date_label(candidate.created_at),
        words: words_sent(&prompt_text),
        cut: prompt_text.chars().count() > MAX_CHARS,
        recordings: recordings.len(),
        prompt_text,
        segments,
        said,
        replaces: summary_range(&candidate.body).is_some(),
    })
}

fn one_line(text: &str, max: usize) -> String {
    let line = text.split_whitespace().collect::<Vec<_>>().join(" ");
    let line = if line.chars().count() > max {
        let cut: String = line.chars().take(max).collect();
        format!("{}…", cut.trim_end())
    } else {
        line
    };
    // Never one of our own markers, or a comment that swallows the rest.
    line.replace("<!--", "< !--")
}

/// `text`, one line, as Markdown that only shows words: the model's words
/// are untrusted, so no image (a remote one would make the preview fetch,
/// and carry what was said out in its address) and no raw HTML, our
/// markers among it. Ordinary links stay links. Pure.
fn md_line(text: &str, max: usize) -> String {
    // A backslash of the model's own is escaped first, so it can never
    // undo the escapes that follow.
    strip_remote_images(&one_line(text, max))
        .replace('\\', "\\\\")
        .replace("![", "\\![")
        .replace('<', "\\<")
}

fn some_md(text: &Option<String>) -> Option<String> {
    text.as_deref()
        .map(|t| md_line(t, 80))
        .filter(|t| !t.is_empty())
}

fn some_line(text: &Option<String>) -> Option<String> {
    text.as_deref()
        .map(|t| one_line(t, 80))
        .filter(|t| !t.is_empty())
}

/// The second the first valid quote of `quotes` was said, or `None` when no
/// quote is in the transcript.
fn moment_of(source: &SummarySource, quotes: &[String]) -> Option<u32> {
    let start = valid_quotes(&source.said, quotes)
        .into_iter()
        .map(|(start, _)| start)
        .min()?;
    let at = source.segments.partition_point(|s| s.from <= start);
    source
        .segments
        .get(at.checked_sub(1)?)
        .map(|s| s.start_secs)
}

/// The model's reply, kept where the transcript backs it: a decision or an
/// action item survives only with a quote that is really there, and then
/// links to the moment that quote was said. The overview is the model's own
/// words and is kept as written, a few points at most. Pure.
pub fn ground(source: &SummarySource, tidy: &Tidy) -> SummaryDraft {
    let mut dropped = 0;
    let mut keep = |text: &str, who: &Option<String>, when: &Option<String>, quotes: &[String]| {
        let text = one_line(text, ITEM_MAX);
        match moment_of(source, quotes) {
            Some(moment) if !text.is_empty() => Some(SummaryItem {
                text,
                who: some_line(who),
                when: some_line(when),
                moment,
            }),
            _ => {
                dropped += 1;
                None
            }
        }
    };
    let decisions: Vec<SummaryItem> = tidy
        .decisions
        .iter()
        .filter_map(|d| keep(&d.text, &None, &None, &d.quotes))
        .take(ITEMS_MAX)
        .collect();
    let actions: Vec<SummaryItem> = tidy
        .actions
        .iter()
        .filter_map(|a| keep(&a.text, &a.who, &a.when, &a.quotes))
        .take(ITEMS_MAX)
        .collect();
    SummaryDraft {
        file_id: source.file_id.clone(),
        overview: tidy
            .summary
            .iter()
            .map(|s| one_line(s, ITEM_MAX))
            .filter(|s| !s.is_empty())
            .take(ITEMS_MAX)
            .collect(),
        decisions,
        actions,
        dropped,
        cut: tidy.cut,
    }
}

/// What the sheet before a summary says: where the transcript goes and what
/// does not. `agent` names the agent on this computer when one answers.
/// Every part of the sentence the design asks for is here: the provider, the
/// word count, "The audio is not sent. The transcript is.", and who keeps
/// what it receives. Pure.
pub fn send_line(
    words: usize,
    provider: kanso_ai::Provider,
    base_url: &str,
    agent: Option<&str>,
) -> String {
    let (to, after) = match agent {
        Some(agent) => (
            format!("{agent} on this computer"),
            format!("{agent} passes it on to its own provider under your account."),
        ),
        None if provider.needs_key() => {
            let host = provider.host(base_url);
            (
                host.clone(),
                format!("{host} handles it under its own retention rules."),
            )
        }
        None => (
            format!("the model at {}", provider.host(base_url)),
            "It goes only to that model.".to_owned(),
        ),
    };
    let count = if words == 1 {
        "1 word".to_owned()
    } else {
        format!("{words} words")
    };
    format!(
        "Send this recording's transcript ({count}), with the note's title and the date, to {to}? The audio is not sent. The transcript is. {after}"
    )
}

/// [`send_line`] for `source`, with what else the person should know
/// before Send: that a long transcript is cut, and that only the first of
/// several recordings is sent. Pure.
pub fn send_words(
    source: &SummarySource,
    provider: kanso_ai::Provider,
    base_url: &str,
    agent: Option<&str>,
) -> String {
    let mut words = send_line(source.words, provider, base_url, agent);
    if source.cut {
        words.push_str(" The transcript is long, so only its first part is sent.");
    }
    if source.recordings > 1 {
        words.push_str(&format!(
            " This note has {} recordings: only the first one's transcript is sent.",
            source.recordings
        ));
    }
    words
}

/// The line an action item is written as: `- [ ] Send the draft — Priya,
/// by the 30th ([00:22](kanso-audio://…))`.
fn action_line(item: &SummaryItem, file_id: &str) -> String {
    let mut line = format!("- [ ] {}", md_line(&item.text, ITEM_MAX));
    let (who, when) = (some_md(&item.who), some_md(&item.when));
    let who: Vec<&str> = [who.as_deref(), when.as_deref()]
        .into_iter()
        .flatten()
        .collect();
    if !who.is_empty() {
        line.push_str(" — ");
        line.push_str(&who.join(", "));
    }
    line.push_str(&format!(" ({})", moment_link(file_id, item.moment)));
    line
}

/// The summary as Markdown between its markers: the overview, the
/// decisions, and the action items whose positions are in `keep`. `None`
/// when there is nothing to write. Pure.
pub fn render_summary(draft: &SummaryDraft, keep: &BTreeSet<usize>) -> Option<String> {
    let mut sections: Vec<String> = Vec::new();
    if !draft.overview.is_empty() {
        let points: Vec<String> = draft
            .overview
            .iter()
            .map(|p| format!("- {}", md_line(p, ITEM_MAX)))
            .collect();
        sections.push(format!("## Summary\n\n{}", points.join("\n")));
    }
    if !draft.decisions.is_empty() {
        let lines: Vec<String> = draft
            .decisions
            .iter()
            .map(|d| {
                format!(
                    "- {} ({})",
                    md_line(&d.text, ITEM_MAX),
                    moment_link(&draft.file_id, d.moment)
                )
            })
            .collect();
        sections.push(format!("## Decisions\n\n{}", lines.join("\n")));
    }
    let actions: Vec<String> = draft
        .actions
        .iter()
        .enumerate()
        .filter(|(ix, _)| keep.contains(ix))
        .map(|(_, a)| action_line(a, &draft.file_id))
        .collect();
    if !actions.is_empty() {
        sections.push(format!("## Action items\n\n{}", actions.join("\n")));
    }
    if sections.is_empty() {
        return None;
    }
    Some(format!(
        "{SUMMARY_OPEN}\n{AI_LABEL}\n\n{}\n{SUMMARY_CLOSE}",
        sections.join("\n\n")
    ))
}

/// `body` with `block` put in: in place of the summary already there, else
/// just above the recording's transcript. Everything outside the markers is
/// untouched. `None` when there is neither a summary to replace nor a
/// transcript of `file_id` to go above. Pure.
pub fn put_summary(body: &str, file_id: &str, block: &str) -> Option<(String, bool)> {
    if let Some(range) = summary_range(body) {
        return Some((
            format!("{}{block}{}", &body[..range.start], &body[range.end..]),
            true,
        ));
    }
    let at = transcript_range(body, file_id)?.start;
    let before = &body[..at];
    let gap = if before.is_empty() || before.ends_with("\n\n") {
        ""
    } else if before.ends_with('\n') {
        "\n"
    } else {
        "\n\n"
    };
    Some((format!("{before}{gap}{block}\n\n{}", &body[at..]), false))
}

/// Writes the summary of `draft`, with the action items in `keep`, into
/// note `id` in one `put`, so Undo and the note's history bring the note
/// back. The note is read again first: it must still be fit for AI, still
/// hold the transcript, and not change under the write
/// ([`SummaryError::Changed`]). The write is made against the revision
/// that was read, so an edit, a sync or another device that gets in first
/// is never written over.
pub fn apply_summary(
    store: &Store,
    id: &str,
    draft: &SummaryDraft,
    keep: &BTreeSet<usize>,
    origin: Origin,
) -> Result<Applied, SummaryError> {
    let candidate = note_for_ai(store, id).ok_or(SummaryError::KeptOut)?;
    let note = store.note(id)?.ok_or(SummaryError::Missing)?;
    // The gate and the revision must be of one and the same note: a change
    // between the two reads (a lock seals the body) is refused.
    if note.body != candidate.body {
        return Err(SummaryError::Changed);
    }
    if store.note_is_locked(id).unwrap_or(true) {
        return Err(SummaryError::KeptOut);
    }
    // A recording id from outside is checked like one read from a link.
    if parse_moment(&format!("{AUDIO_SCHEME}{}", draft.file_id)).is_none() {
        return Err(SummaryError::NoTranscript);
    }
    let block = render_summary(draft, keep).ok_or(SummaryError::Empty)?;
    let (body, replaced) =
        put_summary(&candidate.body, &draft.file_id, &block).ok_or(SummaryError::NoTranscript)?;
    let result = store.put(
        DocType::Note,
        json!({"_id": id, "_rev": note.rev, "body": body}),
        Put::Merge,
        origin,
    );
    match result {
        Ok(_) => Ok(Applied {
            id: id.to_owned(),
            replaced,
            edit: crate::batch::Edit {
                id: id.to_owned(),
                change: json!({"_id": id, "body": body}),
                undo: json!({"_id": id, "body": note.body}),
            },
        }),
        Err(StoreError::Conflict) => Err(SummaryError::Changed),
        Err(err) => Err(err.into()),
    }
}

#[cfg(test)]
mod tests;
