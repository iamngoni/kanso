//! The two tools a model with an API key may call while it answers, and the
//! record of everything it was shown.
//!
//! Both tools keep the exclusions retrieval keeps (locked notes, the trash,
//! `#ai-exclude`) and the question's scope, so a tool cannot leave either.
//! Their results are numbered lines continuing the numbering of the first
//! prompt's excerpts, and every result is registered in [`Shown`], so the
//! receipt checker can verify a citation of tool text exactly as it verifies
//! one of the first excerpts.

use std::sync::{Arc, Mutex};

use kanso_ai::prompt::{cut_line, escape_excerpt};
use kanso_ai::{Budget, Excerpt, JsonTool, Numbered, SentExcerpt, ToolRegistry};
use kanso_core::dates::Date;
use kanso_store::Store;
use serde::Deserialize;

use super::scope::ScopeFilter;
use super::{
    Candidate, candidate_for, content_words, question_terms, required_matches, search_documents,
    tag_names, term_hits,
};
use crate::ask::best_passages;

/// How much of one note a tool search result may carry.
const TOOL_EXCERPT_ROOM: usize = 900;
/// The most lines `read_note` returns in one call.
pub const READ_LINES: u32 = 200;
/// How many notes a tool's search may return.
const TOOL_RESULTS: usize = 5;

struct State {
    sent: Vec<SentExcerpt>,
    used: Vec<Excerpt>,
    next: usize,
}

/// What the model was shown in one question: the numbered lines of every
/// excerpt (the first prompt's and each tool result's), and the notes it
/// drew on for the "Used" and "Also read" lists. Shared between the tools
/// and the code that checks the answer.
pub struct Shown {
    state: Mutex<State>,
}

impl Shown {
    pub fn new() -> Arc<Shown> {
        Arc::new(Shown {
            state: Mutex::new(State {
                sent: Vec::new(),
                used: Vec::new(),
                next: 1,
            }),
        })
    }

    fn lock(&self) -> std::sync::MutexGuard<'_, State> {
        self.state
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
    }

    /// Records the first prompt: what it sent, and the notes it sent.
    pub fn start(&self, numbered: &Numbered) {
        let mut state = self.lock();
        state.sent = numbered.sent.clone();
        state.next = numbered.sent.iter().map(|s| s.number).max().unwrap_or(0) + 1;
        for sent in &numbered.sent {
            note_used(&mut state.used, &sent.id, &sent.title);
        }
    }

    /// Everything the model was shown, for checking its receipts.
    pub fn sent(&self) -> Vec<SentExcerpt> {
        self.lock().sent.clone()
    }

    /// The notes it drew on, one entry per note, in order of first use.
    pub fn used(&self) -> Vec<Excerpt> {
        self.lock().used.clone()
    }

    /// Notes a tool read, added to the used list.
    pub fn note_used(&self, id: &str, title: &str) {
        note_used(&mut self.lock().used, id, title);
    }

    /// Registers lines a tool returned as the next excerpt and returns its
    /// number.
    fn add(
        &self,
        id: &str,
        title: &str,
        updated: Option<String>,
        lines: Vec<(u32, String)>,
    ) -> usize {
        let mut state = self.lock();
        let number = state.next;
        state.next += 1;
        state.sent.push(SentExcerpt {
            number,
            id: id.to_owned(),
            title: title.to_owned(),
            updated,
            lines,
        });
        note_used(&mut state.used, id, title);
        number
    }

    /// Registers excerpts that some other reader (an agent's notes server)
    /// handed the model, under the numbers it gave them.
    pub fn register(&self, excerpt: SentExcerpt) {
        let mut state = self.lock();
        state.next = state.next.max(excerpt.number + 1);
        note_used(&mut state.used, &excerpt.id, &excerpt.title);
        state.sent.retain(|s| s.number != excerpt.number);
        state.sent.push(excerpt);
    }
}

fn note_used(used: &mut Vec<Excerpt>, id: &str, title: &str) {
    if !used.iter().any(|seen| seen.id == id) {
        used.push(Excerpt {
            id: id.to_owned(),
            title: title.to_owned(),
            ..Excerpt::default()
        });
    }
}

/// What a tool hands back for one note.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ToolNote {
    pub excerpt: usize,
    pub id: String,
    pub title: String,
    pub updated: Option<String>,
    pub lines: Vec<(u32, String)>,
}

impl ToolNote {
    /// The note as the tool's JSON: `lines` are `L12| text` rows, counted
    /// from 1 as the first prompt's are, and escaped the same way.
    fn json(&self) -> serde_json::Value {
        serde_json::json!({
            "excerpt": self.excerpt,
            "id": self.id,
            "title": self.title,
            "updated": self.updated,
            "lines": self.lines.iter()
                .map(|(n, text)| format!("L{}| {}", n + 1, escape_excerpt(text)))
                .collect::<Vec<_>>(),
        })
    }
}

fn updated_of(candidate: &Candidate) -> Option<String> {
    (candidate.updated_at > 0).then(|| Date::from_local_ms(candidate.updated_at).iso())
}

/// Trims lines to what is left of this question's tool room. A line that
/// doesn't fit is dropped whole; titles and IDs always stay so the model can
/// still name a note it saw.
fn clamp_lines(lines: Vec<(u32, String)>, room: &Mutex<usize>) -> Vec<(u32, String)> {
    let mut left = room.lock().unwrap_or_else(|poisoned| poisoned.into_inner());
    let mut kept = Vec::new();
    for (n, text) in lines {
        let size = text.chars().count() + 8;
        if size > *left {
            break;
        }
        *left -= size;
        kept.push((n, text));
    }
    kept
}

/// The numbered lines of a passage list.
fn lines_of(passages: &[kanso_core::passages::Passage]) -> Vec<(u32, String)> {
    let mut lines = Vec::new();
    for passage in passages {
        for (ix, line) in passage.text.split('\n').enumerate() {
            lines.push((passage.start + ix as u32, cut_line(line)));
        }
    }
    lines
}

/// The notes a tool search returns, with the passages that matched. Every
/// exclusion applies exactly as it does to the initial retrieval, and so
/// does the scope.
pub fn search_notes(
    store: &Store,
    query: &str,
    scope: &ScopeFilter,
    shown: &Shown,
    room: &Mutex<usize>,
) -> kanso_store::Result<Vec<ToolNote>> {
    let terms = question_terms(query);
    if terms.is_empty() || content_words(query).is_empty() {
        return Ok(Vec::new());
    }
    let names = tag_names(store)?;
    let (docs, failed, last_error) = search_documents(store, &terms, None)?;
    if failed == terms.len() {
        return Err(last_error.expect("a failed search has an error"));
    }
    let required = required_matches(terms.len()).min(1);
    let mut scored: Vec<(Candidate, usize, i64)> = Vec::new();
    for (id, doc) in &docs {
        if doc.doc_type != kanso_core::DocType::Note {
            continue;
        }
        let Some(candidate) = candidate_for(store, id, &names, None)? else {
            continue;
        };
        if !scope.allows(&candidate.id, &candidate.book_id, &candidate.tag_ids) {
            continue;
        }
        let (matched, in_title) = term_hits(&candidate.title, &candidate.body, &terms);
        if matched < required {
            continue;
        }
        let updated = candidate.updated_at;
        scored.push((candidate, matched * 10 + in_title * 8, updated));
    }
    scored.sort_by(|a, b| {
        b.1.cmp(&a.1)
            .then_with(|| b.2.cmp(&a.2))
            .then_with(|| a.0.id.cmp(&b.0.id))
    });
    let mut results = Vec::new();
    for (candidate, _, _) in scored.into_iter().take(TOOL_RESULTS) {
        let mut passages = best_passages(&candidate.body, &terms, TOOL_EXCERPT_ROOM);
        if passages.is_empty() {
            // Only the title matched: the note's opening is what it says.
            passages = kanso_core::passages::passages(&candidate.body)
                .into_iter()
                .take(1)
                .collect();
        }
        if passages.is_empty() {
            continue;
        }
        let lines = clamp_lines(lines_of(&passages), room);
        if lines.is_empty() {
            continue;
        }
        let updated = updated_of(&candidate);
        let excerpt = shown.add(
            &candidate.id,
            &candidate.title,
            updated.clone(),
            lines.clone(),
        );
        results.push(ToolNote {
            excerpt,
            id: candidate.id,
            title: candidate.title,
            updated,
            lines,
        });
    }
    Ok(results)
}

/// Lines `start..=end` (0-based) of one note, at most [`READ_LINES`], or
/// `None` when the note is missing, excluded or outside the scope (so an
/// excluded note is indistinguishable from a missing one). Blank lines are
/// numbered but not sent.
pub fn read_lines(
    store: &Store,
    id: &str,
    start: u32,
    end: u32,
    scope: &ScopeFilter,
    shown: &Shown,
    room: &Mutex<usize>,
) -> kanso_store::Result<Option<ToolNote>> {
    let id = if id.trim().starts_with("note:") {
        id.trim().to_owned()
    } else {
        format!("note:{}", id.trim())
    };
    let names = tag_names(store)?;
    let Some(candidate) = candidate_for(store, &id, &names, None)? else {
        return Ok(None);
    };
    if !scope.allows(&candidate.id, &candidate.book_id, &candidate.tag_ids) {
        return Ok(None);
    }
    let end = end.max(start).min(start.saturating_add(READ_LINES - 1));
    let lines: Vec<(u32, String)> = candidate
        .body
        .split('\n')
        .enumerate()
        .skip(start as usize)
        .take((end - start + 1) as usize)
        .filter(|(_, line)| !line.trim().is_empty())
        .map(|(n, line)| (n as u32, cut_line(line)))
        .collect();
    let lines = clamp_lines(lines, room);
    let updated = updated_of(&candidate);
    let excerpt = shown.add(
        &candidate.id,
        &candidate.title,
        updated.clone(),
        lines.clone(),
    );
    Ok(Some(ToolNote {
        excerpt,
        id: candidate.id,
        title: candidate.title,
        updated,
        lines,
    }))
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
    /// The first line to read, counted from 1 as the lines you were shown
    /// are. Defaults to the first line.
    #[serde(default)]
    start: Option<u32>,
    /// The last line to read; at most 200 lines are returned.
    #[serde(default)]
    end: Option<u32>,
}

/// The two tools a question may call. Both enforce the same exclusions as
/// retrieval and the question's scope, add what they read to `shown`, and
/// share one budget for everything they hand the model.
pub fn tool_registry(
    store: Arc<Store>,
    shown: Arc<Shown>,
    scope: ScopeFilter,
    budget: &Budget,
) -> ToolRegistry<()> {
    let mut registry = ToolRegistry::new();
    // One budget for everything the tools hand the model this question, so a
    // model that keeps calling search cannot grow the request without bound.
    let room = Arc::new(Mutex::new(budget.tool_chars));
    let scope = Arc::new(scope);
    let (search_store, search_shown, search_room, search_scope) =
        (store.clone(), shown.clone(), room.clone(), scope.clone());
    registry.register(JsonTool::new(
        "search_notes",
        "Search the user's notes. Returns matching notes, each with an excerpt number and numbered lines you can cite as [excerpt:Lline].",
        move |_ctx: (), args: SearchArgs| {
            let (store, shown, room, scope) = (
                search_store.clone(),
                search_shown.clone(),
                search_room.clone(),
                search_scope.clone(),
            );
            async move {
                match search_notes(&store, &args.query, &scope, &shown, &room) {
                    Ok(results) => Ok(serde_json::json!({
                        "results": results.iter().map(ToolNote::json).collect::<Vec<_>>(),
                    })),
                    Err(_) => Ok(serde_json::json!({
                        "error": "Couldn't search the notes.",
                    })),
                }
            }
        },
    ));
    registry.register(JsonTool::new(
        "read_note",
        "Read lines of one note by ID, as numbered lines you can cite as [excerpt:Lline]. A note that is locked, in the trash or tagged #ai-exclude is reported as not found.",
        move |_ctx: (), args: ReadArgs| {
            let (store, shown, room, scope) =
                (store.clone(), shown.clone(), room.clone(), scope.clone());
            async move {
                let start = args.start.unwrap_or(1).max(1) - 1;
                let end = args.end.unwrap_or(start + READ_LINES).max(1) - 1;
                match read_lines(&store, &args.id, start, end, &scope, &shown, &room) {
                    Ok(Some(note)) => {
                        let mut json = note.json();
                        json["found"] = serde_json::json!(true);
                        Ok(json)
                    }
                    Ok(None) | Err(_) => Ok(serde_json::json!({
                        "found": false,
                        "error": "not found",
                    })),
                }
            }
        },
    ));
    registry
}
