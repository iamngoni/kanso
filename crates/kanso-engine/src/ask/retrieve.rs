//! Retrieval: what a question may send. Everything here is local; nothing
//! calls a model.
//!
//! A question is understood first (the time in it, the earlier subject of a
//! follow-up), then candidate notes come from full-text search per term,
//! from meaning search over notes and over the passages of long notes, from
//! the links of the best notes, from the time range, and from what the
//! previous answer cited. The lists are fused by reciprocal rank, every
//! candidate goes through [`super::candidate_for`] (locks, the trash,
//! `#ai-exclude`) and the scope, and the best *passages* of what is left are
//! chosen within the budget. Passages carry their lines, so what is sent can
//! be cited by line.

use std::collections::HashMap;
use std::ops::Range;
use std::rc::Rc;

use kanso_ai::Excerpt;
use kanso_core::dates::Date;
use kanso_core::passages::{Passage, passages};
use kanso_core::when::{self, DateRange, TimeIntent};
use kanso_store::{MEANING_SIMILARITY, MeaningSearch, SearchParams, Store, TaskQuery};

use super::model::Retrieval;
use super::model::{AskRequest, Closest, ScopeRefusal, TimeNote};
use super::scope::{ScopeFilter, book_paths};
use super::{
    Bounds, Candidate, MAX_NOTES, MAX_PER_NOTE, MEANING_ONLY_ROOM, MEANING_ONLY_SIMILARITY,
    MEANING_RESULTS, MIN_PER_NOTE, candidate_for, content_words, eligible, followup, fuse,
    question_terms, required_matches, search_documents, tag_names, term_hits, word_count,
};

/// The most passages one question sends.
pub const MAX_PASSAGES: usize = 24;
/// How many passages of long notes meaning search looks for.
const PASSAGE_MEANING_RESULTS: usize = 40;
/// How many of the best notes have their links followed.
const LINK_SEEDS: usize = 4;
/// How many neighbours a linked note may add.
const NEIGHBOURS: usize = 8;
/// How many notes a question about a time alone starts from.
const TIME_ONLY_NOTES: usize = 40;
/// What a passage costs on top of its text, per line (`L123| `).
const LINE_COST: usize = 7;

/// Why retrieval could not run.
#[derive(Debug)]
pub enum AskError {
    Store(kanso_store::StoreError),
    Scope(ScopeRefusal),
}

impl std::fmt::Display for AskError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AskError::Store(err) => write!(f, "{err}"),
            AskError::Scope(refusal) => f.write_str(refusal.message()),
        }
    }
}

impl std::error::Error for AskError {}

impl From<kanso_store::StoreError> for AskError {
    fn from(err: kanso_store::StoreError) -> Self {
        AskError::Store(err)
    }
}

impl From<ScopeRefusal> for AskError {
    fn from(refusal: ScopeRefusal) -> Self {
        AskError::Scope(refusal)
    }
}

/// Notes loaded once per question, after every exclusion and the scope.
struct Pool<'a> {
    store: &'a Store,
    names: HashMap<String, String>,
    scope: ScopeFilter,
    cache: HashMap<String, Option<Rc<Candidate>>>,
}

impl<'a> Pool<'a> {
    fn new(store: &'a Store, scope: ScopeFilter) -> kanso_store::Result<Self> {
        Ok(Pool {
            store,
            names: tag_names(store)?,
            scope,
            cache: HashMap::new(),
        })
    }

    /// Note `id` if it may be sent and is inside the scope. Every candidate
    /// comes through `candidate_for`, which re-checks the note (including a
    /// lock check that fails closed).
    fn get(&mut self, id: &str) -> kanso_store::Result<Option<Rc<Candidate>>> {
        if let Some(found) = self.cache.get(id) {
            return Ok(found.clone());
        }
        let found = candidate_for(self.store, id, &self.names, None)?
            .filter(|c| self.scope.allows(&c.id, &c.book_id, &c.tag_ids))
            .map(Rc::new);
        self.cache.insert(id.to_owned(), found.clone());
        Ok(found)
    }
}

/// A candidate with the lines something points at.
struct Entry<'a> {
    candidate: &'a Candidate,
    /// `(first, last)` lines (0-based) that a link, a task, the previous
    /// answer or a passage match point at.
    seeds: &'a [(u32, u32)],
}

/// The passages chosen from one note.
struct Pick {
    index: usize,
    passages: Vec<Passage>,
    /// The score of its best passage.
    best: f64,
    best_range: (u32, u32),
}

/// What `choose` does with a note that has no passage matching a word.
#[derive(Clone, Copy, PartialEq, Eq)]
enum Mode {
    /// A question with words: notes must match them (or be a strong match
    /// by meaning).
    Words,
    /// A question about a time alone: any note in the range may send its
    /// opening.
    TimeOnly,
}

fn cost(passage: &Passage) -> usize {
    passage.text.chars().count() + LINE_COST * passage.lines() as usize
}

/// `passage` cut to its first lines that fit `room`, or `None` when not even
/// one does.
fn trim_to(passage: &Passage, room: usize) -> Option<Passage> {
    if cost(passage) <= room {
        return Some(passage.clone());
    }
    let mut used = 0;
    let mut kept: Vec<&str> = Vec::new();
    for line in passage.text.split('\n') {
        let line_cost = line.chars().count().min(kanso_ai::prompt::LINE_MAX) + LINE_COST;
        if used + line_cost > room {
            break;
        }
        used += line_cost;
        kept.push(line);
    }
    if kept.is_empty() {
        // A single long line is still one line: it is cut when sent.
        if room < LINE_COST {
            return None;
        }
        kept.push(passage.text.split('\n').next().unwrap_or(""));
    }
    Some(Passage {
        start: passage.start,
        end: passage.start + kept.len() as u32 - 1,
        heading: passage.heading.clone(),
        text: kept.join("\n"),
    })
}

fn overlaps(start: u32, end: u32, seed: &(u32, u32)) -> bool {
    start <= seed.1 && seed.0 <= end
}

/// How well a passage answers `terms`.
fn score_passage(passage: &Passage, terms: &[String], seeds: &[(u32, u32)]) -> f64 {
    let mut distinct = 0;
    let mut occurrences = 0;
    for term in terms {
        let n = word_count(&passage.text, term);
        if n > 0 {
            distinct += 1;
            occurrences += n;
        }
    }
    let in_heading = passage.heading.as_deref().map_or(0, |heading| {
        terms.iter().filter(|t| word_count(heading, t) > 0).count()
    });
    let mut score = distinct as f64 * 3. + occurrences.min(8) as f64 * 0.5 + in_heading as f64 * 2.;
    if seeds
        .iter()
        .any(|seed| overlaps(passage.start, passage.end, seed))
    {
        score += 5.;
    }
    score
}

/// The passages to send from `entries`, best note first, within `budget`
/// characters: the best passage of each note first, then the best of what is
/// left, at most [`MAX_PASSAGES`] from at most `limit` notes. The exclusions
/// are applied again here, before anything is read out.
fn choose(
    terms: &[String],
    entries: &[Entry],
    limit: usize,
    budget: usize,
    mode: Mode,
) -> Vec<Pick> {
    let allowed: Vec<(usize, &Entry)> = entries
        .iter()
        .enumerate()
        .filter(|(_, e)| eligible(e.candidate))
        .take(limit)
        .collect();
    if allowed.is_empty() || (terms.is_empty() && mode == Mode::Words) {
        return Vec::new();
    }
    let required = required_matches(terms.len());
    let per_note = if allowed.len() == 1 {
        budget
    } else {
        (budget / allowed.len()).clamp(MIN_PER_NOTE, MAX_PER_NOTE)
    };

    // (entry order, passage, score)
    let mut firsts: Vec<(usize, Passage, f64)> = Vec::new();
    let mut rest: Vec<(usize, Passage, f64)> = Vec::new();
    for (order, (_, entry)) in allowed.iter().enumerate() {
        let c = entry.candidate;
        let (matched, in_title) = term_hits(&c.title, &c.body, terms);
        let from_meaning = c.meaning.is_some_and(|score| score >= MEANING_SIMILARITY);
        let seeded = !entry.seeds.is_empty();
        if mode == Mode::Words && matched < required && !from_meaning && !seeded {
            continue;
        }
        let mut scored: Vec<(Passage, f64)> = passages(&c.body)
            .into_iter()
            .map(|p| {
                let score = score_passage(&p, terms, entry.seeds);
                (p, score)
            })
            .filter(|(_, score)| *score > 0.)
            .collect();
        scored.sort_by(|a, b| b.1.total_cmp(&a.1).then_with(|| a.0.start.cmp(&b.0.start)));
        if scored.is_empty() {
            // Nothing in the text matches. A time range alone, a seed, or a
            // title or meaning match sends the note's opening; a weak
            // meaning-only match sends nothing.
            let strong = c
                .meaning
                .is_some_and(|score| score >= MEANING_ONLY_SIMILARITY);
            let room = if mode == Mode::TimeOnly || seeded {
                per_note
            } else if matched == 0 {
                if !strong {
                    continue;
                }
                per_note.min(MEANING_ONLY_ROOM)
            } else if !from_meaning && in_title == 0 {
                continue;
            } else {
                per_note
            };
            let mut opening: Vec<Passage> = passages(&c.body);
            opening.truncate(MAX_PASSAGES);
            let mut used = 0;
            for p in opening {
                let fit = trim_to(&p, room.saturating_sub(used));
                let Some(fit) = fit else { break };
                used += cost(&fit);
                scored.push((fit, 0.1));
                if used >= room {
                    break;
                }
            }
            if scored.is_empty() {
                continue;
            }
        }
        let mut iter = scored.into_iter();
        if let Some((p, score)) = iter.next() {
            firsts.push((order, p, score));
        }
        rest.extend(iter.map(|(p, score)| (order, p, score)));
    }

    let mut used = 0;
    let mut per: HashMap<usize, usize> = HashMap::new();
    let mut chosen: HashMap<usize, Vec<(Passage, f64)>> = HashMap::new();
    let mut count = 0;
    for (order, passage, score) in firsts {
        if count >= MAX_PASSAGES {
            break;
        }
        let room = per_note.min(budget.saturating_sub(used));
        let Some(fit) = trim_to(&passage, room) else {
            continue;
        };
        let size = cost(&fit);
        used += size;
        *per.entry(order).or_default() += size;
        chosen.entry(order).or_default().push((fit, score));
        count += 1;
    }
    rest.sort_by(|a, b| {
        b.2.total_cmp(&a.2)
            .then_with(|| a.0.cmp(&b.0))
            .then_with(|| a.1.start.cmp(&b.1.start))
    });
    for (order, passage, score) in rest {
        if count >= MAX_PASSAGES {
            break;
        }
        if !chosen.contains_key(&order) {
            continue;
        }
        let size = cost(&passage);
        if per.get(&order).copied().unwrap_or(0) + size > per_note || used + size > budget {
            continue;
        }
        used += size;
        *per.entry(order).or_default() += size;
        chosen.entry(order).or_default().push((passage, score));
        count += 1;
    }

    let mut picks = Vec::new();
    for (order, (index, _)) in allowed.iter().enumerate() {
        let Some(mut list) = chosen.remove(&order) else {
            continue;
        };
        let (best, best_range) = list
            .iter()
            .max_by(|a, b| a.1.total_cmp(&b.1))
            .map(|(p, s)| (*s, (p.start, p.end)))
            .unwrap_or((0., (0, 0)));
        list.sort_by_key(|(p, _)| p.start);
        picks.push(Pick {
            index: *index,
            passages: list.into_iter().map(|(p, _)| p).collect(),
            best,
            best_range,
        });
    }
    picks
}

/// An excerpt from the passages chosen from a note.
fn excerpt_of(
    candidate: &Candidate,
    passages: &[Passage],
    utc_offset_ms: i64,
    books: &HashMap<String, String>,
) -> Excerpt {
    // Lines are kept as they will be sent: cut to the longest line allowed.
    let mut lines = Vec::new();
    let mut blocks = Vec::new();
    for passage in passages {
        let mut block = Vec::new();
        for (ix, line) in passage.text.split('\n').enumerate() {
            let cut = kanso_ai::prompt::cut_line(line);
            block.push(cut.clone());
            lines.push((passage.start + ix as u32, cut));
        }
        blocks.push(block.join("\n"));
    }
    Excerpt {
        id: candidate.id.clone(),
        title: candidate.title.clone(),
        text: blocks.join("\n\n"),
        lines,
        updated: (candidate.updated_at > 0)
            .then(|| Date::from_local_ms(candidate.updated_at + utc_offset_ms).iso()),
        notebook: books.get(&candidate.book_id).cloned(),
    }
}

/// The notes a question may send, best first, as excerpts with numbered
/// lines. The exclusions are applied here before anything is read out.
pub fn select(
    question: &str,
    candidates: &[Candidate],
    limit: usize,
    budget: usize,
) -> Vec<Excerpt> {
    if question.trim().is_empty() {
        return Vec::new();
    }
    let terms = question_terms(question);
    let entries: Vec<Entry> = candidates
        .iter()
        .map(|candidate| Entry {
            candidate,
            seeds: &[],
        })
        .collect();
    choose(&terms, &entries, limit, budget, Mode::Words)
        .into_iter()
        .map(|pick| excerpt_of(&candidates[pick.index], &pick.passages, 0, &HashMap::new()))
        .collect()
}

/// The passages of `body` that match `terms`, best first, within `room`
/// characters. A note with no matching passage gives nothing.
pub fn best_passages(body: &str, terms: &[String], room: usize) -> Vec<Passage> {
    if body.trim().is_empty() || room == 0 || terms.is_empty() {
        return Vec::new();
    }
    let mut scored: Vec<(Passage, f64)> = passages(body)
        .into_iter()
        .map(|p| {
            let score = score_passage(&p, terms, &[]);
            (p, score)
        })
        .filter(|(_, score)| *score > 0.)
        .collect();
    scored.sort_by(|a, b| b.1.total_cmp(&a.1).then_with(|| a.0.start.cmp(&b.0.start)));
    let mut out = Vec::new();
    let mut used = 0;
    for (p, _) in scored {
        let Some(fit) = trim_to(&p, room.saturating_sub(used)) else {
            break;
        };
        used += cost(&fit);
        out.push(fit);
        if used >= room {
            break;
        }
    }
    out.sort_by_key(|p| p.start);
    out
}

/// The passages of `body` that match `terms`, joined, within `room`
/// characters, as plain text.
pub fn best_text(body: &str, terms: &[String], room: usize) -> String {
    let found = best_passages(body, terms, room + LINE_COST * 8);
    let text = found
        .iter()
        .map(|p| p.text.as_str())
        .collect::<Vec<_>>()
        .join("\n\n");
    kanso_ai::prompt::truncate(&text, room)
}

/// Searches `store` for `question` and returns the excerpts to send, or
/// nothing when the notes don't cover it. For a question with a scope, a
/// budget or a time, use [`retrieve_with`].
pub fn retrieve(store: &Store, question: &str) -> kanso_store::Result<Vec<Excerpt>> {
    let today = Date::from_local_ms(kanso_core::now_ms());
    match retrieve_with(store, &AskRequest::new(question, today)) {
        Ok(found) => Ok(found.excerpts),
        Err(AskError::Store(err)) => Err(err),
        Err(AskError::Scope(_)) => Ok(Vec::new()),
    }
}

/// The date of a daily note, from its ID (`note:daily-2026-10-14`).
fn daily_date(id: &str) -> Option<Date> {
    let day = id.strip_prefix("note:daily-")?;
    Date::parse_iso(day.get(..10)?)
}

fn day_of(ms: i64, offset: &dyn Fn(i64) -> i64) -> Date {
    Date::from_local_ms(ms + offset(ms))
}

/// Whether the note was written in `range`: made or changed in it, or a
/// daily note whose date is in it.
fn written_in(c: &Candidate, range: &DateRange, offset: &dyn Fn(i64) -> i64) -> bool {
    range.contains(day_of(c.updated_at, offset))
        || range.contains(day_of(c.created_at, offset))
        || daily_date(&c.id).is_some_and(|d| range.contains(d))
}

/// Whether the note's own words hold a date in `range`.
fn about_in(c: &Candidate, range: &DateRange) -> bool {
    when::dates_in(&c.body).iter().any(|d| range.contains(*d))
}

fn matches_time(c: &Candidate, intent: &TimeIntent, offset: &dyn Fn(i64) -> i64) -> bool {
    match intent {
        TimeIntent::Written(range) => written_in(c, range, offset),
        TimeIntent::About(range) => about_in(c, range),
        TimeIntent::Either(range) => written_in(c, range, offset) || about_in(c, range),
    }
}

/// `text` without the bytes in `span`, white space tidied.
fn without(text: &str, span: &Range<usize>) -> String {
    let mut out = String::with_capacity(text.len());
    out.push_str(&text[..span.start]);
    out.push(' ');
    out.push_str(&text[span.end..]);
    out.split_whitespace().collect::<Vec<_>>().join(" ")
}

/// Everything a question is searched for, once it is understood.
struct Understood {
    /// The words searched: the question without its time, made stand-alone.
    query: String,
    terms: Vec<String>,
    time: Option<TimeIntent>,
}

fn understand(request: &AskRequest) -> Understood {
    let question = request.question.trim();
    let found = if request.ignore_time {
        None
    } else {
        when::time_in(question, request.today, request.week_start)
    };
    let (text, time) = match found {
        Some((intent, span)) => (without(question, &span), Some(intent)),
        None => (question.to_owned(), None),
    };
    let query = match &request.rewrite {
        Some(rewritten) if !rewritten.trim().is_empty() => rewritten.trim().to_owned(),
        _ => followup::standalone(&text, request.previous.as_ref()),
    };
    // A question that is nothing but a time ("what did I write last week")
    // has no word to search for; the time itself is the search.
    let terms = if time.is_some() && content_words(&query).is_empty() {
        Vec::new()
    } else {
        question_terms(&query)
    };
    Understood { query, terms, time }
}

/// Searches `store` for `request` and chooses what to send.
pub fn retrieve_with(store: &Store, request: &AskRequest) -> Result<Retrieval, AskError> {
    if request.question.trim().is_empty() {
        return Ok(Retrieval::default());
    }
    let scope = ScopeFilter::resolve(store, &request.scope)?;
    let understood = understand(request);
    let Understood { query, terms, time } = &understood;
    if terms.is_empty() && time.is_none() {
        return Ok(Retrieval::default());
    }
    let offset = |ms: i64| request.offset_at(ms);
    let mut pool = Pool::new(store, scope)?;
    let mut seeds: HashMap<String, Vec<(u32, u32)>> = HashMap::new();
    let mut meaning_scores: HashMap<String, f32> = HashMap::new();

    // What the previous answer cited can be cited again.
    let mut previous_ids: Vec<String> = Vec::new();
    if let Some(previous) = &request.previous {
        for (id, first, last) in &previous.cited {
            if !previous_ids.contains(id) {
                previous_ids.push(id.clone());
            }
            seeds.entry(id.clone()).or_default().push((*first, *last));
        }
    }

    let strict_bounds = match time {
        Some(TimeIntent::Written(range)) => {
            let (after, before) = request.bounds_ms(range);
            Some(Bounds { after, before })
        }
        _ => None,
    };

    // Full text, then (if a written range left too little) again without it.
    let mut text_ids: Vec<String> = Vec::new();
    let mut relaxed = false;
    let mut attempt_bounds = strict_bounds.as_ref();
    loop {
        text_ids.clear();
        let mut ranked: Vec<(String, f64, i64)> = Vec::new();
        if !terms.is_empty() {
            let (docs, failed, last_error) = search_documents(store, terms, attempt_bounds)?;
            // Every full-text search failing is a real search error; one
            // failing term is not.
            if failed == terms.len() {
                return Err(AskError::Store(
                    last_error.expect("a failed search has an error"),
                ));
            }
            let required = required_matches(terms.len());
            for (id, doc) in &docs {
                if doc.doc_type != kanso_core::DocType::Note {
                    continue;
                }
                let Some(candidate) = pool.get(id)? else {
                    continue;
                };
                let (matched, in_title) = term_hits(&candidate.title, &candidate.body, terms);
                if matched < required {
                    continue;
                }
                let occurrences: usize = terms
                    .iter()
                    .map(|term| word_count(&candidate.body, term))
                    .sum();
                let score = matched as f64 * 10.
                    + in_title as f64 * 8.
                    + (occurrences.min(12) as f64) * 0.5;
                ranked.push((id.clone(), score, candidate.updated_at));
            }
            ranked.sort_by(|a, b| {
                b.1.total_cmp(&a.1)
                    .then_with(|| b.2.cmp(&a.2))
                    .then_with(|| a.0.cmp(&b.0))
            });
        }
        text_ids = ranked.into_iter().map(|(id, _, _)| id).collect();
        // Does the time range leave enough?
        let Some(intent) = time else { break };
        if matches!(intent, TimeIntent::Either(_)) || terms.is_empty() {
            break;
        }
        let mut kept = 0;
        for id in &text_ids {
            if let Some(c) = pool.get(id)?
                && matches_time(&c, intent, &offset)
            {
                kept += 1;
            }
        }
        if kept >= 2 || (attempt_bounds.is_none() && matches!(intent, TimeIntent::About(_))) {
            if kept < 2 {
                relaxed = true;
            }
            break;
        }
        if attempt_bounds.is_some() {
            // Search again without the range and let it rank instead.
            attempt_bounds = None;
            relaxed = true;
            continue;
        }
        relaxed = true;
        break;
    }

    // A time alone searches the range itself.
    let mut time_ids: Vec<String> = Vec::new();
    if let Some(intent) = time {
        let range = intent.range();
        let mut found: Vec<String> = Vec::new();
        if !matches!(intent, TimeIntent::About(_)) && terms.is_empty() {
            let (after, before) = request.bounds_ms(&range);
            let params = SearchParams {
                updated_after: after,
                updated_before: before,
                limit: Some(TIME_ONLY_NOTES),
                ..SearchParams::default()
            };
            for doc in store.search(&params)? {
                found.push(doc.id);
            }
        }
        if !matches!(intent, TimeIntent::Written(_)) {
            // Tasks due in the range point at their own lines.
            let after = range.start.map(|d| d.add_days(-1).iso());
            let before = range.end.map(|d| d.add_days(1).iso());
            let tasks = store.tasks(&TaskQuery {
                include_done: true,
                due_after: after,
                due_before: before,
                limit: Some(200),
                ..TaskQuery::default()
            })?;
            for task in tasks {
                seeds
                    .entry(task.note_id.clone())
                    .or_default()
                    .push((task.line, task.line));
                if !found.contains(&task.note_id) {
                    found.push(task.note_id);
                }
            }
        }
        for id in found {
            if pool.get(&id)?.is_some() {
                time_ids.push(id);
            }
        }
    }

    // Meaning, best effort: no model, or an error, leaves full text alone.
    let mut meaning_ids = Vec::new();
    if let Ok(MeaningSearch::Found(found)) = store.search_by_meaning(query, MEANING_RESULTS) {
        for note in found {
            if note.score >= MEANING_SIMILARITY && pool.get(&note.id)?.is_some() {
                meaning_scores.insert(note.id.clone(), note.score);
                meaning_ids.push(note.id);
            }
        }
    }
    // The passages of long notes, which a note's own vector cannot see.
    let mut passage_ids = Vec::new();
    if let Ok(hits) = store.search_passages_by_meaning(query, PASSAGE_MEANING_RESULTS) {
        for hit in hits {
            if hit.score < MEANING_SIMILARITY || pool.get(&hit.note_id)?.is_none() {
                continue;
            }
            seeds
                .entry(hit.note_id.clone())
                .or_default()
                .push((hit.start, hit.end));
            let best = meaning_scores
                .entry(hit.note_id.clone())
                .or_insert(hit.score);
            if hit.score > *best {
                *best = hit.score;
            }
            if !passage_ids.contains(&hit.note_id) {
                passage_ids.push(hit.note_id);
            }
        }
    }

    // Links: the neighbours of the best notes, if they match at all.
    let mut lists = vec![text_ids.clone()];
    if !meaning_ids.is_empty() {
        lists.push(meaning_ids);
    }
    if !passage_ids.is_empty() {
        lists.push(passage_ids);
    }
    if !previous_ids.is_empty() {
        lists.push(previous_ids);
    }
    if !time_ids.is_empty() {
        lists.push(time_ids.clone());
    }
    if request.use_links {
        let best = fuse(&lists, LINK_SEEDS);
        let mut linked: Vec<(String, usize)> = Vec::new();
        for id in &best {
            let mut neighbours: Vec<(String, Option<u32>)> = Vec::new();
            for back in store.backlinks(id)? {
                neighbours.push((back.id, Some(back.line)));
            }
            let mut forward: Vec<String> = store.links_from(id)?.into_iter().collect();
            forward.sort();
            for target in forward {
                neighbours.push((target, None));
            }
            for (neighbour, line) in neighbours.into_iter().take(NEIGHBOURS * 2) {
                if best.contains(&neighbour) {
                    continue;
                }
                let Some(candidate) = pool.get(&neighbour)? else {
                    continue;
                };
                let matched = if terms.is_empty() {
                    0
                } else {
                    term_hits(&candidate.title, &candidate.body, terms).0
                };
                let by_meaning = meaning_scores
                    .get(&neighbour)
                    .is_some_and(|s| *s >= MEANING_SIMILARITY);
                if matched == 0 && !by_meaning {
                    continue;
                }
                if let Some(line) = line {
                    seeds
                        .entry(neighbour.clone())
                        .or_default()
                        .push((line, line));
                }
                match linked.iter_mut().find(|(n, _)| *n == neighbour) {
                    Some((_, hits)) => *hits += matched.max(1),
                    None => linked.push((neighbour, matched.max(1))),
                }
            }
        }
        linked.sort_by(|a, b| b.1.cmp(&a.1).then_with(|| a.0.cmp(&b.0)));
        let ids: Vec<String> = linked
            .into_iter()
            .take(NEIGHBOURS)
            .map(|(id, _)| id)
            .collect();
        if !ids.is_empty() {
            lists.push(ids);
        }
    }

    // A written or about range filters when it leaves enough, and only
    // ranks (as one more list) when it does not or when the kind is open.
    let fused = fuse(&lists, MAX_NOTES * 4);
    let mut candidates: Vec<Rc<Candidate>> = Vec::new();
    for id in &fused {
        if let Some(candidate) = pool.get(id)? {
            candidates.push(candidate);
        }
    }
    if let Some(intent) = time {
        let matching: Vec<Rc<Candidate>> = candidates
            .iter()
            .filter(|c| matches_time(c, intent, &offset))
            .cloned()
            .collect();
        match intent {
            TimeIntent::Either(_) => {
                // Either kind of time ranks the notes that have it first.
                let (mut first, later): (Vec<_>, Vec<_>) = candidates
                    .into_iter()
                    .partition(|c| matches_time(c, intent, &offset));
                first.extend(later);
                candidates = first;
            }
            _ if matching.len() >= 2 || terms.is_empty() => {
                if matching.len() < 2 && !terms.is_empty() {
                    relaxed = true;
                }
                candidates = matching;
            }
            _ => {
                relaxed = true;
                let (mut first, later): (Vec<_>, Vec<_>) = candidates
                    .into_iter()
                    .partition(|c| matches_time(c, intent, &offset));
                first.extend(later);
                candidates = first;
            }
        }
        if matches!(intent, TimeIntent::Written(_)) && terms.is_empty() && candidates.len() < 2 {
            relaxed = false;
        }
    }

    // Every note carries the meaning score it earned.
    let owned: Vec<Candidate> = candidates
        .iter()
        .map(|c| {
            let mut c = (**c).clone();
            c.meaning = meaning_scores.get(&c.id).copied();
            c
        })
        .collect();
    let no_seeds: Vec<(u32, u32)> = Vec::new();
    let entries: Vec<Entry> = owned
        .iter()
        .map(|candidate| Entry {
            candidate,
            seeds: seeds.get(&candidate.id).unwrap_or(&no_seeds),
        })
        .collect();
    let mode = if terms.is_empty() {
        Mode::TimeOnly
    } else {
        Mode::Words
    };
    let budget = request.budget.excerpt_chars;
    let picks = choose(terms, &entries, MAX_NOTES, budget, mode);

    let books = book_paths(store);
    let excerpts: Vec<Excerpt> = picks
        .iter()
        .map(|pick| {
            excerpt_of(
                &owned[pick.index],
                &pick.passages,
                request.utc_offset_ms,
                &books,
            )
        })
        .collect();
    let mut ranked: Vec<&Pick> = picks.iter().collect();
    ranked.sort_by(|a, b| b.best.total_cmp(&a.best));
    let closest = ranked
        .into_iter()
        .take(3)
        .map(|pick| Closest {
            note_id: owned[pick.index].id.clone(),
            title: owned[pick.index].title.clone(),
            start: pick.best_range.0,
            end: pick.best_range.1,
        })
        .collect();
    let time_note = time.map(|intent| TimeNote {
        label: intent.range().label(),
        intent,
        relaxed: relaxed && !excerpts.is_empty(),
    });
    Ok(Retrieval {
        excerpts,
        time: time_note,
        query: query.clone(),
        closest,
    })
}

#[cfg(test)]
mod tests;
