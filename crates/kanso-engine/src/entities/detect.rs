//! Finding people and projects that notes keep mentioning, without AI.
//!
//! Pure: a [`Detector`] is fed notes one at a time and then says which names
//! look like a person or a project. Nothing here writes a page; the result
//! is a list of suggestions the person may accept.
//!
//! Strong signals are `@name` with no page, a wiki-link that names no note,
//! the front matter keys `people`, `attendees`, `owner`, `with` and
//! `project`, and lines such as `Attendees: A, B, C`. Weak ones are pairs of
//! capitalised words in the middle of a sentence that recur in several notes.
//! Single capitalised words count only when a strong signal names them too.

use std::collections::{BTreeMap, HashMap, HashSet};

use kanso_core::entities::{EntityKind, at_mentions, normalize_name};
use kanso_core::links::{code_ranges, wiki_links};
use kanso_core::properties::{self, Value};
use kanso_store::CandidateRow;

/// Notes a weak name must appear in.
pub const WEAK_NOTES: usize = 3;
/// Notes a name found in a role (an attendee list, a property) needs.
pub const ROLE_NOTES: usize = 2;
/// The most suggestions kept.
pub const MAX_CANDIDATES: usize = 50;
/// A name this long, in words, is not a name.
const MAX_WORDS: usize = 3;
/// A single word that opens another candidate and shares this share of
/// its notes is proposed as an alias.
pub const ALIAS_SHARE: f64 = 0.5;
/// "Not now" hides a name until it has this many more notes.
pub const SNOOZE_EXTRA: u32 = 5;

/// Words that start sentences and titles, and say nothing about who or
/// what a note is about.
const STOP: &[&str] = &[
    "the",
    "a",
    "an",
    "this",
    "that",
    "these",
    "those",
    "i",
    "we",
    "you",
    "he",
    "she",
    "it",
    "they",
    "my",
    "our",
    "your",
    "his",
    "her",
    "their",
    "and",
    "but",
    "or",
    "if",
    "when",
    "then",
    "also",
    "here",
    "there",
    "what",
    "why",
    "how",
    "who",
    "where",
    "which",
    "should",
    "could",
    "would",
    "can",
    "will",
    "do",
    "does",
    "did",
    "not",
    "no",
    "yes",
    "ok",
    "okay",
    "please",
    "thanks",
    "thank",
    "hi",
    "hello",
    "dear",
    "mr",
    "mrs",
    "ms",
    "dr",
    "for",
    "from",
    "with",
    "about",
    "after",
    "before",
    "during",
    "just",
    "maybe",
    "check",
    "send",
    "call",
    "email",
    "read",
    "write",
    "buy",
    "fix",
    "add",
    "make",
    "get",
    "set",
    "see",
    "try",
    "ask",
    "need",
    "want",
    "remember",
    "look",
    "talk",
    "todo",
    "note",
    "notes",
    "meeting",
    "meetings",
    "action",
    "items",
    "item",
    "next",
    "steps",
    "step",
    "summary",
    "agenda",
    "review",
    "weekly",
    "daily",
    "monthly",
    "plan",
    "plans",
    "idea",
    "ideas",
    "draft",
    "update",
    "updates",
    "reminder",
    "question",
    "questions",
    "follow",
    "up",
    "team",
    "today",
    "tomorrow",
    "yesterday",
    "tonight",
    "monday",
    "tuesday",
    "wednesday",
    "thursday",
    "friday",
    "saturday",
    "sunday",
    "january",
    "february",
    "march",
    "april",
    "may",
    "june",
    "july",
    "august",
    "september",
    "october",
    "november",
    "december",
    "mon",
    "tue",
    "wed",
    "thu",
    "fri",
    "sat",
    "sun",
    "jan",
    "feb",
    "mar",
    "apr",
    "jun",
    "jul",
    "aug",
    "sep",
    "sept",
    "oct",
    "nov",
    "dec",
    "all",
    "some",
    "any",
    "each",
    "every",
    "both",
    "one",
    "two",
    "three",
    "first",
    "second",
    "third",
    "last",
    "new",
    "old",
    "good",
    "great",
    "so",
    "as",
    "at",
    "by",
    "in",
    "of",
    "on",
    "to",
    "is",
    "are",
    "was",
    "were",
    "be",
    "been",
    "have",
    "has",
    "had",
    "let",
    "like",
    "now",
    "still",
    "again",
    "done",
    "key",
    "main",
    "open",
    "close",
    "start",
    "stop",
    "end",
    "goal",
    "goals",
    "risk",
    "risks",
    "decision",
    "decisions",
    "status",
    "context",
    "background",
    "details",
    "overview",
    "attendees",
    "present",
    "people",
    "participants",
    "project",
    "projects",
    "owner",
    "tasks",
    "task",
    "list",
    "lists",
    "title",
    "tags",
    "tag",
    "date",
    "time",
    "link",
    "links",
    "more",
    "less",
    "other",
    "another",
    "something",
    "nothing",
    "everything",
    "anything",
    "someone",
    "everyone",
    "anyone",
    "none",
    "our",
    "off",
    "out",
    "over",
    "under",
    "until",
    "while",
    "since",
    "because",
    "though",
    "although",
    "unless",
    "whether",
    "either",
    "neither",
];

/// Words that follow a person's name when they speak or act.
const SPEECH: &[&str] = &[
    "said",
    "says",
    "thinks",
    "wants",
    "promised",
    "asked",
    "told",
    "suggested",
    "mentioned",
    "agreed",
    "replied",
    "emailed",
    "called",
    "wrote",
    "needs",
    "will",
    "prefers",
    "likes",
    "owes",
    "sent",
];

/// Words that, before a name, say a person is meant.
const PERSON_BEFORE: &[&str] = &[
    "ask", "tell", "email", "call", "ping", "cc", "thanks", "ring",
];

/// Words on a line that say a project is meant.
const PROJECT_WORDS: &[&str] = &[
    "launch",
    "milestone",
    "release",
    "roadmap",
    "sprint",
    "deadline",
];

/// What detection knows besides the notes.
#[derive(Clone, Debug, Default)]
pub struct Context {
    /// Normalized titles of every note, entity pages included: a wiki-link
    /// to one is not unresolved.
    pub known_titles: HashSet<String>,
    /// Every name and alias of a page, normalized, to the page's note ID.
    pub entity_names: HashMap<String, String>,
    /// Names never to suggest: tag names, notebook names, and what the
    /// person said is not a person or project.
    pub excluded: HashSet<String>,
    /// Names set aside with "Not now", to the notes they had then.
    pub snoozed: HashMap<String, u32>,
}

/// A note as detection reads it.
#[derive(Clone, Copy, Debug)]
pub struct Input<'a> {
    pub title: &'a str,
    pub body: &'a str,
    pub created_at: i64,
    pub tags: &'a [String],
}

#[derive(Default)]
struct Acc {
    forms: BTreeMap<String, u32>,
    notes: HashSet<u32>,
    strong_one: bool,
    strong_two: bool,
    person: u32,
    project: u32,
    words: usize,
    last_seen: i64,
}

impl Acc {
    fn note(&mut self, ordinal: u32, form: &str, seen: i64) {
        self.notes.insert(ordinal);
        *self.forms.entry(form.to_owned()).or_insert(0) += 1;
        self.last_seen = self.last_seen.max(seen);
    }
}

/// Collects signals across notes.
pub struct Detector {
    ctx: Context,
    accs: HashMap<String, Acc>,
    next: u32,
    /// For each page with a name of two or more words, the notes holding it.
    tracked: HashMap<String, (String, HashSet<u32>)>,
}

fn is_stop(word: &str) -> bool {
    STOP.contains(&word.to_lowercase().as_str())
}

/// A word that could be part of a name: an initial capital, letters only
/// (hyphens and apostrophes inside), and not shouting.
fn capitalised(word: &str) -> bool {
    let mut chars = word.chars();
    let Some(first) = chars.next() else {
        return false;
    };
    first.is_uppercase()
        && word.chars().count() >= 2
        && word
            .chars()
            .all(|c| c.is_alphabetic() || matches!(c, '\'' | '-' | '\u{2019}'))
        && word.chars().any(char::is_lowercase)
        && !word.ends_with(['-', '\''])
}

/// The words of a line with whether each opens a sentence and whether it
/// is followed by punctuation that ends a name.
struct Token<'a> {
    text: &'a str,
    sentence_start: bool,
    /// Text between this word and the next is a single space.
    joined: bool,
    after: Option<&'a str>,
}

fn tokens(line: &str) -> Vec<Token<'_>> {
    let mut out: Vec<Token<'_>> = Vec::new();
    let mut at = 0;
    let bytes = line.as_bytes();
    let mut start_sentence = true;
    while at < line.len() {
        let rest = &line[at..];
        let c = rest.chars().next().unwrap_or(' ');
        if c.is_alphabetic() {
            let end = rest
                .char_indices()
                .find(|(_, ch)| !(ch.is_alphabetic() || matches!(ch, '\'' | '-' | '\u{2019}')))
                .map_or(rest.len(), |(i, _)| i);
            let word = &rest[..end];
            let next_at = at + end;
            let gap_end = line[next_at..]
                .find(|ch: char| ch.is_alphabetic())
                .map_or(line.len(), |i| next_at + i);
            let gap = &line[next_at..gap_end];
            out.push(Token {
                text: word,
                sentence_start: start_sentence,
                joined: gap == " ",
                after: None,
            });
            start_sentence = gap.contains(['.', '!', '?', ':']);
            at = next_at;
        } else {
            at += c.len_utf8();
            if matches!(c, '.' | '!' | '?' | ':') && bytes.get(at - 1).is_some() {
                start_sentence = true;
            }
        }
    }
    // The word after each, for "Sam said".
    let texts: Vec<&str> = out.iter().map(|t| t.text).collect();
    for (ix, token) in out.iter_mut().enumerate() {
        token.after = texts.get(ix + 1).copied();
    }
    out
}

/// A line's text with list, quote and emphasis markers off.
fn line_body(line: &str) -> &str {
    let mut text = line.trim();
    loop {
        let before = text;
        text = text.trim_start_matches(['-', '*', '+', '>', ' ']);
        if let Some(rest) = text
            .strip_prefix("[ ] ")
            .or_else(|| text.strip_prefix("[x] "))
        {
            text = rest;
        }
        text = text.trim_start_matches("**").trim_start_matches("__");
        if text == before {
            return text;
        }
    }
}

fn title_case(text: &str) -> String {
    text.split_whitespace()
        .map(|w| {
            let mut c = w.chars();
            match c.next() {
                Some(f) => f.to_uppercase().chain(c).collect::<String>(),
                None => String::new(),
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}

/// Whether `text`, written by a person, could be a name: letters, spaces,
/// hyphens, apostrophes and dots, up to four words, not a date or number.
fn name_like(text: &str) -> bool {
    let text = text.trim();
    let words = text.split_whitespace().count();
    (1..=4).contains(&words)
        && text.chars().any(char::is_alphabetic)
        && text.chars().all(|c| {
            c.is_alphabetic() || matches!(c, ' ' | '-' | '\'' | '.' | '\u{2019}' | '@' | '_')
        })
        && !is_stop(text.trim_start_matches('@'))
}

impl Detector {
    pub fn new(ctx: Context) -> Detector {
        let mut tracked = HashMap::new();
        for (name, id) in &ctx.entity_names {
            if name.split(' ').count() >= 2 && name.chars().count() >= 5 {
                tracked.insert(name.clone(), (id.clone(), HashSet::new()));
            }
        }
        Detector {
            ctx,
            accs: HashMap::new(),
            next: 0,
            tracked,
        }
    }

    fn excluded(&self, key: &str) -> bool {
        key.is_empty()
            || self.ctx.excluded.contains(key)
            || self.ctx.entity_names.contains_key(key)
            || key.split(' ').all(is_stop)
    }

    fn add(&mut self, form: &str, ordinal: u32, seen: i64, edit: impl FnOnce(&mut Acc)) {
        let key = normalize_name(form);
        if self.excluded(&key) {
            return;
        }
        let words = key.split(' ').count();
        let acc = self.accs.entry(key).or_default();
        acc.words = words;
        acc.note(ordinal, form.trim(), seen);
        edit(acc);
    }

    /// Reads one note.
    pub fn feed(&mut self, note: &Input) {
        let ordinal = self.next;
        self.next += 1;
        let seen = note.created_at;
        let body = note.body;
        if body.is_empty() {
            return;
        }
        let lower = body.to_lowercase();
        for (name, (_, notes)) in self.tracked.iter_mut() {
            if lower.contains(name.as_str()) {
                notes.insert(ordinal);
            }
        }
        let code = code_ranges(body);
        // Front matter.
        let props = properties::parse(body);
        for (key, kind) in [
            ("people", EntityKind::Person),
            ("attendees", EntityKind::Person),
            ("owner", EntityKind::Person),
            ("with", EntityKind::Person),
            ("project", EntityKind::Project),
        ] {
            let Some(value) = properties::get(&props, key) else {
                continue;
            };
            let names: Vec<String> = match value {
                Value::List(items) => items.iter().map(Value::text).collect(),
                other => other.text().split([',', ';']).map(str::to_owned).collect(),
            };
            for name in names {
                let name = name.trim().trim_matches(['"', '\'']).to_owned();
                if name_like(&name) {
                    self.role(&name, kind, ordinal, seen);
                }
            }
        }
        // Tags such as `project-atlas`.
        for tag in note.tags {
            let lowered = tag.to_lowercase();
            if let Some(rest) = lowered
                .strip_prefix("project-")
                .or_else(|| lowered.strip_prefix("project/"))
                .or_else(|| lowered.strip_prefix("project_"))
            {
                let form = title_case(&rest.replace(['-', '_', '/'], " "));
                if name_like(&form) {
                    self.add(&form, ordinal, seen, |acc| {
                        acc.strong_one = true;
                        acc.project += 3;
                    });
                }
            }
        }
        // `@name` with no page.
        for (_, name) in at_mentions(body) {
            let form = format!("@{name}");
            if name_like(&name) || name.contains('.') {
                self.add(&form, ordinal, seen, |acc| {
                    acc.strong_one = true;
                    acc.person += 3;
                });
            }
        }
        // Wiki-links to titles no note has.
        for link in wiki_links(body) {
            let key = normalize_name(&link.title);
            let dateish =
                kanso_core::dates::Date::parse_iso(link.title.trim().get(..10).unwrap_or(""))
                    .is_some()
                    || link.title.chars().any(|c| c.is_ascii_digit());
            if key.is_empty()
                || dateish
                || self.ctx.known_titles.contains(&key)
                || self.ctx.entity_names.contains_key(&key)
                || code.iter().any(|r| r.contains(&link.range.start))
                || !name_like(&link.title)
            {
                continue;
            }
            let person = capitalised_count(&link.title) >= 2;
            self.add(&link.title, ordinal, seen, |acc| {
                acc.strong_one = true;
                if person {
                    acc.person += 1;
                } else {
                    acc.project += 1;
                }
            });
        }
        // Lines.
        let mut offset = 0usize;
        let mut in_fence = false;
        for raw in body.split('\n') {
            let line_start = offset;
            offset += raw.len() + 1;
            if raw.trim_start().starts_with("```") || raw.trim_start().starts_with("~~~") {
                in_fence = !in_fence;
                continue;
            }
            if in_fence
                || code
                    .iter()
                    .any(|r| r.start <= line_start && line_start < r.end)
            {
                continue;
            }
            let text = line_body(raw);
            if text.is_empty() || text.starts_with('#') || text.starts_with("---") {
                continue;
            }
            self.line(text, ordinal, seen);
        }
    }

    fn role(&mut self, name: &str, kind: EntityKind, ordinal: u32, seen: i64) {
        let form = name.trim().to_owned();
        self.add(&form, ordinal, seen, |acc| {
            acc.strong_two = true;
            match kind {
                EntityKind::Person => acc.person += 2,
                EntityKind::Project => acc.project += 2,
            }
        });
    }

    fn line(&mut self, text: &str, ordinal: u32, seen: i64) {
        let lower = text.to_lowercase();
        // `Attendees: A, B and C`.
        for label in [
            "attendees:",
            "present:",
            "with:",
            "people:",
            "participants:",
            "owner:",
        ] {
            if let Some(list) = lower.strip_prefix(label) {
                let skip = text.len() - list.len();
                let list = &text[skip..];
                for name in list.split([',', ';']).flat_map(|p| p.split(" and ")) {
                    let name = name.trim().trim_end_matches('.');
                    if name_like(name) {
                        self.role(name, EntityKind::Person, ordinal, seen);
                    }
                }
                return;
            }
        }
        if let Some(rest) = lower.strip_prefix("project:") {
            let skip = text.len() - rest.len();
            let name = text[skip..].trim().trim_end_matches('.');
            if name_like(name) {
                self.role(name, EntityKind::Project, ordinal, seen);
            }
            return;
        }
        let projecty = PROJECT_WORDS.iter().any(|w| lower.contains(w));
        // "project Atlas migration".
        let mut search = 0;
        while let Some(at) = lower[search..].find("project ") {
            let begin = search + at;
            search = begin + "project ".len();
            let before_ok = begin == 0
                || !lower[..begin]
                    .chars()
                    .next_back()
                    .is_some_and(char::is_alphanumeric);
            if !before_ok {
                continue;
            }
            let words: Vec<&str> = text[search..]
                .split_whitespace()
                .take(3)
                .map(|w| w.trim_matches(|c: char| !c.is_alphanumeric()))
                .take_while(|w| !w.is_empty() && !is_stop(w) && w.chars().all(char::is_alphabetic))
                .collect();
            if let Some(first) = words.first()
                && first.chars().next().is_some_and(char::is_uppercase)
            {
                let name = words.join(" ");
                self.add(&name, ordinal, seen, |acc| {
                    acc.project += 2;
                    acc.strong_two = true;
                });
            }
        }
        // Capitalised words in the middle of sentences.
        let toks = tokens(text);
        let mut ix = 0;
        while ix < toks.len() {
            if !capitalised(toks[ix].text) {
                ix += 1;
                continue;
            }
            let mut end = ix + 1;
            while end < toks.len()
                && toks[end - 1].joined
                && capitalised(toks[end].text)
                && !toks[end].sentence_start
            {
                end += 1;
            }
            let mut run: Vec<&Token> = toks[ix..end].iter().collect();
            let first_start = run[0].sentence_start;
            if first_start && is_stop(run[0].text) {
                run.remove(0);
            }
            let before = if ix > 0 {
                Some(toks[ix - 1].text.to_lowercase())
            } else {
                None
            };
            ix = end;
            if run.is_empty() || run.len() > MAX_WORDS || run.iter().any(|t| is_stop(t.text)) {
                continue;
            }
            let form = run.iter().map(|t| t.text).collect::<Vec<_>>().join(" ");
            let after = run.last().and_then(|t| t.after).map(str::to_lowercase);
            let speaks = after.as_deref().is_some_and(|w| SPEECH.contains(&w));
            let asked = before
                .as_deref()
                .is_some_and(|w| PERSON_BEFORE.contains(&w));
            let single = run.len() == 1;
            // A lone word that opens a sentence is just a capital.
            if single && first_start {
                continue;
            }
            self.add(&form, ordinal, seen, |acc| {
                if speaks {
                    acc.person += 2;
                }
                if asked {
                    acc.person += 1;
                }
                if projecty {
                    acc.project += 1;
                }
                if single {
                    // Counts only if a strong signal names it too.
                    acc.words = 1;
                }
            });
        }
    }

    /// The suggestions, best first, at most [`MAX_CANDIDATES`].
    pub fn finish(self) -> Vec<CandidateRow> {
        let Detector {
            ctx, accs, tracked, ..
        } = self;
        let newest = accs.values().map(|a| a.last_seen).max().unwrap_or(0);
        let mut rows: Vec<CandidateRow> = Vec::new();
        for (key, acc) in &accs {
            let notes = acc.notes.len();
            let strong = acc.strong_one || acc.strong_two;
            let single = key.split(' ').count() == 1;
            let enough = (acc.strong_one && notes >= 1)
                || (acc.strong_two && notes >= ROLE_NOTES)
                || (!single && notes >= WEAK_NOTES);
            if !enough || (single && !strong) {
                continue;
            }
            if let Some(&then) = ctx.snoozed.get(key)
                && (notes as u32) < then + SNOOZE_EXTRA
            {
                continue;
            }
            let kind = if acc.person > acc.project {
                EntityKind::Person
            } else if acc.project > acc.person {
                EntityKind::Project
            } else if key.split(' ').count() >= 2 {
                EntityKind::Person
            } else {
                EntityKind::Project
            };
            let recent = (newest - acc.last_seen) < 30 * 86_400_000;
            let score = notes as f64
                + if acc.strong_one { 3.0 } else { 0.0 }
                + if acc.strong_two { 1.5 } else { 0.0 }
                + 0.2 * f64::from(acc.person.max(acc.project))
                + if recent { 0.5 } else { 0.0 };
            let mut forms: Vec<(&String, &u32)> = acc.forms.iter().collect();
            forms.sort_by(|a, b| b.1.cmp(a.1).then(a.0.cmp(b.0)));
            rows.push(CandidateRow {
                name: key.clone(),
                kind,
                notes: notes as u32,
                last_seen: acc.last_seen,
                forms: forms.into_iter().take(3).map(|(f, _)| f.clone()).collect(),
                score,
                alias_of: None,
            });
        }
        // A single word that goes with a longer name is an alias of it, not
        // a name of its own: of a page that exists, or of another candidate.
        let long: Vec<(String, HashSet<u32>)> = rows
            .iter()
            .filter(|r| r.name.contains(' '))
            .map(|r| (r.name.clone(), accs[&r.name].notes.clone()))
            .collect();
        let mut hide: HashSet<String> = HashSet::new();
        for row in rows.iter_mut().filter(|r| !r.name.contains(' ')) {
            let mine = &accs[&row.name].notes;
            let leading = |name: &str| name.split(' ').next() == Some(row.name.as_str());
            for (name, (id, notes)) in &tracked {
                if leading(name) && !notes.is_empty() {
                    let shared = mine.intersection(notes).count();
                    if shared as f64 / notes.len() as f64 >= ALIAS_SHARE {
                        row.alias_of = Some(id.clone());
                    }
                }
            }
            if row.alias_of.is_none() {
                for (name, notes) in &long {
                    if leading(name) && !notes.is_empty() {
                        let shared = mine.intersection(notes).count();
                        if shared as f64 / notes.len() as f64 >= ALIAS_SHARE {
                            hide.insert(row.name.clone());
                        }
                    }
                }
            }
        }
        rows.retain(|r| !hide.contains(&r.name));
        rows.sort_by(|a, b| b.score.total_cmp(&a.score).then(a.name.cmp(&b.name)));
        rows.truncate(MAX_CANDIDATES);
        rows
    }
}

fn capitalised_count(text: &str) -> usize {
    text.split_whitespace().filter(|w| capitalised(w)).count()
}

/// Suggestions from a set of notes: the whole pass in one call.
pub fn detect<'a>(ctx: Context, notes: impl IntoIterator<Item = Input<'a>>) -> Vec<CandidateRow> {
    let mut detector = Detector::new(ctx);
    for note in notes {
        detector.feed(&note);
    }
    detector.finish()
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;

    fn input<'a>(body: &'a str) -> Input<'a> {
        Input {
            title: "t",
            body,
            created_at: 1_700_000_000_000,
            tags: &[],
        }
    }

    fn run(bodies: &[&str], ctx: Context) -> Vec<CandidateRow> {
        detect(ctx, bodies.iter().map(|b| input(b)))
    }

    #[test]
    fn strong_signals_need_one_note_and_weak_ones_three() {
        let found = run(&["Chat with @sam today"], Context::default());
        assert_eq!(found.len(), 1);
        assert_eq!(found[0].name, "sam");
        assert_eq!(found[0].kind, EntityKind::Person);
        // A pair of capitalised words in two notes is not enough, in three it is.
        let two = [
            "Lunch with Tendai Chirwa went well",
            "We asked Tendai Chirwa about it",
        ];
        assert!(run(&two, Context::default()).is_empty());
        let three = [
            "Lunch with Tendai Chirwa went well",
            "We asked Tendai Chirwa about it",
            "Tendai Chirwa said no",
        ];
        let found = run(&three, Context::default());
        assert_eq!(found.len(), 1);
        assert_eq!(found[0].name, "tendai chirwa");
        assert_eq!(found[0].notes, 3);
    }

    #[test]
    fn single_capitalised_words_and_sentence_starts_do_not_count() {
        let bodies = [
            "Monday we met Tendai",
            "Tuesday we met Tendai",
            "Today we met Tendai",
            "Next Steps are clear",
            "Action Items follow",
            "Next Steps again",
            "Next Steps once more",
        ];
        assert!(run(&bodies, Context::default()).is_empty());
    }

    #[test]
    fn roles_attendees_and_projects_are_found() {
        let bodies = [
            "---\nattendees: [Ana Pereira, Kofi Mensah]\n---\nnotes",
            "Attendees: Ana Pereira, Kofi Mensah and Lina Haddad\n",
            "Working on project Atlas migration this week\n",
            "Another note about project Atlas migration\n",
        ];
        let names: Vec<String> = run(&bodies, Context::default())
            .into_iter()
            .map(|c| c.name)
            .collect();
        assert!(names.contains(&"ana pereira".to_owned()));
        assert!(names.contains(&"kofi mensah".to_owned()));
        assert!(names.contains(&"atlas migration".to_owned()), "{names:?}");
        assert!(
            !names.contains(&"lina haddad".to_owned()),
            "one note is not enough in a list"
        );
    }

    #[test]
    fn unresolved_wiki_links_suggest_and_resolved_ones_do_not() {
        let mut ctx = Context::default();
        ctx.known_titles.insert("pricing".into());
        let found = run(
            &["See [[Sam Moyo]] and [[Pricing]] and [[2026-11-04]]"],
            ctx,
        );
        assert_eq!(
            found.iter().map(|c| c.name.as_str()).collect::<Vec<_>>(),
            vec!["sam moyo"]
        );
    }

    #[test]
    fn dismissed_snoozed_tag_and_existing_names_are_left_out() {
        let bodies = ["@sam and @rudo and @ana and @kofi"];
        let mut ctx = Context::default();
        ctx.excluded.insert("sam".into());
        ctx.entity_names.insert("rudo".into(), "note:r".into());
        ctx.snoozed.insert("ana".into(), 1);
        let names: Vec<String> = run(&bodies, ctx).into_iter().map(|c| c.name).collect();
        assert_eq!(names, vec!["kofi".to_owned()]);
    }

    #[test]
    fn a_first_name_that_goes_with_a_page_is_proposed_as_its_alias() {
        let mut ctx = Context::default();
        ctx.entity_names
            .insert("sam moyo".into(), "note:sam".into());
        ctx.entity_names.insert("samuel".into(), "note:sam".into());
        let bodies = [
            "@sam asked Sam Moyo about it",
            "Sam Moyo and @sam again",
            "Sam Moyo alone",
        ];
        let found = run(&bodies, ctx);
        assert_eq!(found.len(), 1);
        assert_eq!(found[0].name, "sam");
        assert_eq!(found[0].alias_of.as_deref(), Some("note:sam"));
    }

    #[test]
    fn code_and_headings_are_ignored() {
        let bodies = [
            "# Tendai Chirwa Notes\n```\nTendai Chirwa Tendai Chirwa\n```\n`Tendai Chirwa`",
            "# Tendai Chirwa",
            "# Tendai Chirwa",
        ];
        assert!(run(&bodies, Context::default()).is_empty());
    }

    /// A labelled library of 200 notes: ten real names among invented noise.
    /// Precision at the default threshold is recorded here and held at 0.8.
    #[test]
    fn precision_on_a_labelled_fixture_of_two_hundred_notes() {
        let people = [
            "Sam Moyo",
            "Tendai Chirwa",
            "Ana Pereira",
            "Rudo Dube",
            "Kofi Mensah",
            "Lina Haddad",
        ];
        let projects = [
            "Atlas migration",
            "Lusaka office",
            "Mobile revamp",
            "Billing rewrite",
        ];
        let noise = ["Google Docs", "New York", "Black Friday"];
        let filler = [
            "Remember to water the plants",
            "Weekly Review pending",
            "Read chapter four",
            "Draft the proposal",
            "Next Steps: decide",
            "Quarterly Review soon",
            "Buy milk and eggs",
            "Thoughts on the budget",
        ];
        let mut bodies: Vec<String> = Vec::new();
        for ix in 0..200 {
            let mut lines = vec![filler[ix % filler.len()].to_owned()];
            let person = people[ix % people.len()];
            let project = projects[ix % projects.len()];
            match ix % 5 {
                0 => lines.push(format!("Lunch with {person} about the plan")),
                1 => lines.push(format!("{person} said the numbers look fine")),
                2 => lines.push(format!(
                    "Attendees: {person}, {}",
                    people[(ix + 1) % people.len()]
                )),
                3 => lines.push(format!("Update on project {project} this week")),
                _ => lines.push(format!("We asked {person} for help")),
            }
            if ix % 7 == 0 {
                lines.push(format!("Notes in {} again", noise[ix % noise.len()]));
            }
            if ix % 11 == 0 {
                lines.push(format!("Stored in {} too", noise[(ix / 11) % noise.len()]));
            }
            bodies.push(lines.join("\n"));
        }
        let found = run(
            &bodies.iter().map(String::as_str).collect::<Vec<_>>(),
            Context::default(),
        );
        let truth: HashSet<String> = people
            .iter()
            .chain(projects.iter())
            .map(|n| normalize_name(n))
            .collect();
        let right = found.iter().filter(|c| truth.contains(&c.name)).count();
        let precision = right as f64 / found.len().max(1) as f64;
        eprintln!(
            "detection precision {precision:.2} ({right} of {})",
            found.len()
        );
        assert!(
            precision >= 0.8,
            "precision {precision} on {:?}",
            found.iter().map(|c| &c.name).collect::<Vec<_>>()
        );
        // And it finds what is there.
        assert!(right >= 8, "found only {right}");
    }

    proptest! {
        #![proptest_config(ProptestConfig { rng_seed: proptest::test_runner::RngSeed::Fixed(20261017), ..ProptestConfig::default() })]

        /// A name in the excluded set never comes back, whatever the notes say.
        #[test]
        fn prop_excluded_names_never_come_back(
            lines in proptest::collection::vec("[A-Za-z@\\[\\] ,.:\n-]{0,60}", 1..6),
        ) {
            let mut ctx = Context::default();
            for name in ["sam", "sam moyo", "atlas", "ana pereira"] {
                ctx.excluded.insert(name.into());
            }
            let mut bodies: Vec<String> = lines;
            bodies.push("@sam [[Sam Moyo]] project Atlas Attendees: Ana Pereira".into());
            let found = detect(ctx, bodies.iter().map(|b| input(b)));
            for row in found {
                prop_assert!(!["sam", "sam moyo", "atlas", "ana pereira"].contains(&row.name.as_str()));
            }
        }

        /// Whatever the text, detection never returns more than it keeps
        /// and every row has at least one note.
        #[test]
        fn prop_detection_is_total_and_bounded(body in "\\PC{0,200}") {
            let found = detect(Context::default(), [input(&body)]);
            prop_assert!(found.len() <= MAX_CANDIDATES);
            for row in found {
                prop_assert!(row.notes >= 1);
                prop_assert!(!row.forms.is_empty());
            }
        }
    }
}
