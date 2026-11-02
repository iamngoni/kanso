//! One question from start to finish, for the desktop panel and the phones
//! alike: understand and retrieve, build the numbered prompt, ask the model
//! (with tools), check the receipts, and repair an answer that ignored the
//! format once.
//!
//! The steps are public ([`prepare`], [`finalize`]) because an ACP agent is
//! asked through its own conversation and only needs the first and the last.

use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

use kanso_ai::{Budget, ChatProvider, Event, Excerpt, Numbered, SharedProvider, ToolRegistry};
use kanso_store::Store;

use super::model::{
    AskRequest, CHECKING, Checked, Closest, MOSTLY_UNCHECKED, NO_SOURCES, Receipt, Retrieval,
    SEARCHING, TimeNote, WRITING, reading_line,
};
use super::receipts::check;
use super::retrieve::{AskError, retrieve_with};
use super::scope::ScopeFilter;
use super::tools::{Shown, tool_registry};
use super::{sanitize_answer, stream_events};

/// The shortest answer worth a repair when it has no receipt at all.
const REPAIR_FROM: usize = 200;
/// The most a rewritten search query may be, in characters.
const REWRITE_MAX: usize = 200;
/// The output limit of the rewrite request.
const REWRITE_TOKENS: u32 = 300;

/// A line over an answer that could not be trusted as a whole.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Banner {
    /// More than half of its claims failed their check.
    MostlyUnchecked,
    /// It has no sources at all; the model did not follow the format.
    NoSources,
}

impl Banner {
    pub fn text(self) -> &'static str {
        match self {
            Banner::MostlyUnchecked => MOSTLY_UNCHECKED,
            Banner::NoSources => NO_SOURCES,
        }
    }
}

/// A finished answer, ready to show.
#[derive(Clone, Debug, Default)]
pub struct Finished {
    /// The answer as Markdown: safe to render, with receipts as
    /// `[1](kanso-cite:1)` links, the sentences that failed their check as
    /// `[…](kanso-unverified:K)` links followed by a label, and notes cited
    /// by title linked.
    pub markdown: String,
    /// The receipts that survived, in order: the Sources list.
    pub receipts: Vec<Receipt>,
    /// Notes the model was shown but did not cite.
    pub also_read: Vec<Excerpt>,
    pub banner: Option<Banner>,
    /// The notes do not answer: show "You never wrote that down."
    pub not_in_notes: bool,
    /// The best passages found, shown when the notes do not answer.
    pub closest: Vec<Closest>,
    pub time: Option<TimeNote>,
    /// How many sentences are marked as not found.
    pub unverified: usize,
    /// The model's words as they came, markers included.
    pub raw: String,
    /// Receipts were not checked (the setting is off).
    pub unchecked: bool,
}

impl Finished {
    /// The answer for the conversation's history: its words, without the
    /// receipts' numbers, which mean nothing to a later question.
    pub fn plain_answer(&self) -> String {
        plain(&self.raw)
    }
}

/// `text` without `[n:Lx]` markers and receipt links.
pub fn plain(text: &str) -> String {
    let checked = check(text, &[]);
    let mut out = checked.markdown;
    // Valid links can't exist without shown lines; strip any a model wrote.
    while let Some(at) = out.find("](kanso-cite:") {
        let Some(open) = out[..at].rfind('[') else {
            break;
        };
        let Some(close) = out[at..].find(')') else {
            break;
        };
        out.replace_range(open..at + close + 1, "");
    }
    out.split_whitespace().collect::<Vec<_>>().join(" ")
}

/// Everything known before the model is asked.
pub struct Prepared {
    pub retrieval: Retrieval,
    pub numbered: Numbered,
    pub shown: Arc<Shown>,
    pub scope: ScopeFilter,
}

/// Understands and retrieves for `request`, and builds the prompt. An empty
/// `retrieval` means no model should be asked.
pub fn prepare(store: &Store, request: &AskRequest) -> Result<Prepared, AskError> {
    let scope = ScopeFilter::resolve(store, &request.scope)?;
    let retrieval = retrieve_with(store, request)?;
    let numbered = kanso_ai::build_numbered(
        &request.question,
        &retrieval.excerpts,
        request.budget.excerpt_chars,
    );
    let shown = Shown::new();
    shown.start(&numbered);
    Ok(Prepared {
        retrieval,
        numbered,
        shown,
        scope,
    })
}

/// Checks `raw` against what `shown` says was sent and makes the answer to
/// display. With `check_sources` off the answer is shown as the model wrote
/// it.
pub fn finalize(raw: &str, shown: &Shown, check_sources: bool, retrieval: &Retrieval) -> Finished {
    let used = shown.used();
    if !check_sources {
        return Finished {
            markdown: sanitize_answer(&escape_markers(raw), &used),
            also_read: used,
            not_in_notes: super::receipts::check(raw, &[]).not_in_notes,
            closest: retrieval.closest.clone(),
            time: retrieval.time.clone(),
            raw: raw.to_owned(),
            unchecked: true,
            ..Finished::default()
        };
    }
    let checked = check(raw, &shown.sent());
    let banner = if checked.has_no_sources() {
        Some(Banner::NoSources)
    } else if checked.mostly_unchecked() {
        Some(Banner::MostlyUnchecked)
    } else {
        None
    };
    let cited: Vec<&str> = checked
        .receipts
        .iter()
        .map(|r| r.note_id.as_str())
        .collect();
    let also_read = used
        .iter()
        .filter(|u| !cited.contains(&u.id.as_str()))
        .cloned()
        .collect();
    let display = checked.display_markdown();
    Finished {
        markdown: sanitize_answer(&display, &used),
        receipts: checked.receipts.clone(),
        also_read,
        banner,
        not_in_notes: checked.not_in_notes,
        closest: retrieval.closest.clone(),
        time: retrieval.time.clone(),
        unverified: checked.unsupported.len(),
        raw: raw.to_owned(),
        unchecked: false,
    }
}

/// Markers shown as the model wrote them, kept from reading as links.
fn escape_markers(text: &str) -> String {
    text.replace('[', "\\[")
}

/// What a running question says to whoever shows it.
#[derive(Clone, Debug)]
pub enum TurnEvent {
    /// A line about what is happening: searching, reading, writing,
    /// checking, or an agent's tool.
    Status(String),
    /// The notes the answer has drawn on so far.
    Used(Vec<Excerpt>),
    /// The time Kanso understood in the question.
    Time(TimeNote),
    /// A piece of the model's words, markers included.
    Text(String),
    /// The finished answer.
    Done(Box<Finished>),
    /// Nothing in the notes matches: no model was asked.
    NothingMatches {
        /// The scope was narrower than all notes, so searching all may help.
        narrow: bool,
    },
    /// It failed, in plain words.
    Failed(String),
    /// The stream stopped without its end marker.
    CutOff,
    /// It was stopped.
    Stopped,
}

/// One question to run.
pub struct Turn {
    pub store: Arc<Store>,
    pub provider: SharedProvider,
    pub request: AskRequest,
    pub history: Vec<kanso_ai::Turn>,
    pub check_sources: bool,
    pub cancel: Arc<AtomicBool>,
}

/// Runs the question, telling `on_event` as it goes. The last event is
/// always one of `Done`, `NothingMatches`, `Failed`, `CutOff` or `Stopped`.
pub fn run(turn: Turn, mut on_event: impl FnMut(TurnEvent)) {
    let Turn {
        store,
        provider,
        mut request,
        history,
        check_sources,
        cancel,
    } = turn;
    let stopped = || cancel.load(Ordering::Relaxed);
    on_event(TurnEvent::Status(SEARCHING.to_owned()));
    let mut prepared = match prepare(&store, &request) {
        Ok(prepared) => prepared,
        Err(AskError::Scope(refusal)) => {
            on_event(TurnEvent::Failed(refusal.message().to_owned()));
            return;
        }
        Err(AskError::Store(err)) => {
            on_event(TurnEvent::Failed(format!(
                "Couldn't search your notes: {err}"
            )));
            return;
        }
    };
    if stopped() {
        on_event(TurnEvent::Stopped);
        return;
    }
    // A follow-up that found nothing is rewritten by the model, once, from
    // the earlier questions and answers alone.
    if prepared.retrieval.is_empty()
        && request.previous.is_some()
        && !history.is_empty()
        && let Some(query) = model_rewrite(&provider, &request, &history, &cancel)
    {
        request.rewrite = Some(query);
        prepared = match prepare(&store, &request) {
            Ok(prepared) => prepared,
            Err(err) => {
                on_event(TurnEvent::Failed(err.to_string()));
                return;
            }
        };
    }
    if prepared.retrieval.is_empty() {
        on_event(TurnEvent::NothingMatches {
            narrow: request.scope.is_narrow(),
        });
        return;
    }
    if let Some(time) = &prepared.retrieval.time {
        on_event(TurnEvent::Time(time.clone()));
    }
    let Prepared {
        retrieval,
        numbered,
        shown,
        scope,
    } = prepared;
    on_event(TurnEvent::Status(reading_line(numbered.sent.len())));
    on_event(TurnEvent::Used(shown.used()));
    let dirty = Arc::new(AtomicBool::new(false));
    let tools = tool_registry(store.clone(), shown.clone(), scope, &request.budget);
    let events = {
        let provider = lock(&provider);
        let dirty = dirty.clone();
        provider.ask(
            &numbered.prompt,
            &history,
            tools,
            Box::new(move || dirty.store(true, Ordering::Relaxed)),
            cancel.clone(),
            &request.budget,
        )
    };
    let mut raw = String::new();
    let mut ended = false;
    let mut failed = false;
    let mut wrote = false;
    stream_events(events, &cancel, |event| match event {
        Event::Text(piece) => {
            if !wrote {
                wrote = true;
                on_event(TurnEvent::Status(WRITING.to_owned()));
            }
            raw.push_str(&piece);
            on_event(TurnEvent::Text(piece));
            if dirty.swap(false, Ordering::Relaxed) {
                on_event(TurnEvent::Used(shown.used()));
            }
        }
        Event::Status(line) => {
            on_event(TurnEvent::Status(line));
            if dirty.swap(false, Ordering::Relaxed) {
                on_event(TurnEvent::Used(shown.used()));
            }
        }
        Event::Done => ended = true,
        Event::Error(message) if message == kanso_ai::CUT_OFF => {}
        Event::Error(message) => {
            failed = true;
            on_event(TurnEvent::Failed(message));
        }
    });
    if failed {
        return;
    }
    if stopped() {
        on_event(TurnEvent::Stopped);
        return;
    }
    if !ended {
        on_event(TurnEvent::CutOff);
        return;
    }
    if raw.trim().is_empty() {
        on_event(TurnEvent::Failed("Nothing came back.".to_owned()));
        return;
    }
    if dirty.swap(false, Ordering::Relaxed) {
        on_event(TurnEvent::Used(shown.used()));
    }
    if check_sources {
        on_event(TurnEvent::Status(CHECKING.to_owned()));
        if needs_repair(&raw, &shown) {
            if let Some(better) = repair(&provider, &numbered, &raw, &request.budget, &cancel) {
                raw = better;
            }
            if stopped() {
                on_event(TurnEvent::Stopped);
                return;
            }
        }
    }
    on_event(TurnEvent::Done(Box::new(finalize(
        &raw,
        &shown,
        check_sources,
        &retrieval,
    ))));
}

fn lock(provider: &SharedProvider) -> std::sync::MutexGuard<'_, Box<dyn ChatProvider>> {
    provider
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
}

/// Whether an answer ignored the receipt format: long enough to matter, no
/// marker the checker could read, and not the sentinel.
fn needs_repair(raw: &str, shown: &Shown) -> bool {
    if raw.trim().chars().count() <= REPAIR_FROM {
        return false;
    }
    let checked: Checked = check(raw, &shown.sent());
    checked.has_no_sources()
}

/// Asks the model once to add citations to `raw`. The repaired answer, or
/// `None` when the request failed or came back empty.
fn repair(
    provider: &SharedProvider,
    numbered: &Numbered,
    raw: &str,
    budget: &Budget,
    cancel: &Arc<AtomicBool>,
) -> Option<String> {
    let prompt = kanso_ai::prompt::repair(&numbered.prompt, raw);
    let events = {
        let provider = lock(provider);
        provider.ask(
            &prompt,
            &[],
            ToolRegistry::new(),
            Box::new(|| {}),
            cancel.clone(),
            budget,
        )
    };
    let mut text = String::new();
    let mut ok = false;
    stream_events(events, cancel, |event| match event {
        Event::Text(piece) => text.push_str(&piece),
        Event::Done => ok = true,
        _ => {}
    });
    (ok && !text.trim().is_empty()).then_some(text)
}

/// The follow-up as a stand-alone search query, written by the model from
/// earlier questions and answers (capped at 2,000 characters). No note text
/// is sent. `None` when it fails or says nothing.
fn model_rewrite(
    provider: &SharedProvider,
    request: &AskRequest,
    history: &[kanso_ai::Turn],
    cancel: &Arc<AtomicBool>,
) -> Option<String> {
    let pairs: Vec<(String, String)> = history
        .iter()
        .map(|t| (t.question.clone(), t.answer.clone()))
        .collect();
    let prompt = kanso_ai::prompt::rewrite_request(&request.question, &pairs, 2_000);
    let budget = Budget {
        output_tokens: REWRITE_TOKENS,
        ..request.budget
    };
    let events = {
        let provider = lock(provider);
        provider.ask(
            &prompt,
            &[],
            ToolRegistry::new(),
            Box::new(|| {}),
            cancel.clone(),
            &budget,
        )
    };
    let mut text = String::new();
    stream_events(events, cancel, |event| {
        if let Event::Text(piece) = event {
            text.push_str(&piece);
        }
    });
    let line = text
        .lines()
        .map(str::trim)
        .find(|l| !l.is_empty())?
        .trim_matches(|c: char| c == '"' || c == '\'' || c == '`')
        .trim();
    (!line.is_empty()).then(|| kanso_ai::prompt::truncate(line, REWRITE_MAX))
}
