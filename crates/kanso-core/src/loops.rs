//! Open loops: things a note says the writer will do, is waiting on, or
//! has to check, without being a checkbox ("I'll send Tendai the deck",
//! "waiting on Rudo for the quote", "TODO: renew passport").
//!
//! Detection is a small set of English phrase rules ([`RULES`]), run per
//! sentence on every save next to the task parser. The rules are
//! deliberately conservative: a missed loop costs less than a noisy list.
//! Open checkboxes are tasks and never loops.
//!
//! A loop is known by its key ([`loop_key`]): the note and the sentence,
//! not the line, so moving it around the note keeps it the same loop.
//! Closing or dismissing one writes `{key: at}` into the note's
//! `kansoLoops` field, which syncs with the note and merges by union
//! ([`merge_loops`]). Loops an AI pass proposed and the person approved
//! live in the same field, by their sentence.

use std::collections::BTreeMap;

use serde_json::{Map, Value, json};

use crate::tasks::{due_date, fence_marker, task_marker};

/// The note field holding closed, dismissed and approved AI loops.
pub const LOOPS_FIELD: &str = "kansoLoops";

/// The longest sentence kept, in characters.
pub const MAX_TEXT_CHARS: usize = 240;

/// What kind of promise a loop is.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum LoopKind {
    /// Something the writer will do.
    Mine,
    /// Something somebody else owes the writer.
    Waiting,
    /// Something to find out.
    Question,
}

impl LoopKind {
    pub fn as_str(self) -> &'static str {
        match self {
            LoopKind::Mine => "mine",
            LoopKind::Waiting => "waiting",
            LoopKind::Question => "question",
        }
    }

    pub fn parse(text: &str) -> Option<LoopKind> {
        match text {
            "mine" => Some(LoopKind::Mine),
            "waiting" => Some(LoopKind::Waiting),
            "question" => Some(LoopKind::Question),
            _ => None,
        }
    }
}

/// A loop found in a note body.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LoopItem {
    /// The line (0-based) it is on, as [`crate::tasks::TaskItem::line`].
    pub line: u32,
    pub kind: LoopKind,
    /// The sentence, trimmed, at most [`MAX_TEXT_CHARS`] characters.
    pub text: String,
    /// The matched phrase within `text` (byte range), for bold.
    pub span: (usize, usize),
    /// A due date in the sentence (`@2026-10-25`), as tasks read it.
    pub due: Option<String>,
}

/// How a rule recognises the start of a sentence.
#[derive(Clone, Copy, Debug)]
enum Shape {
    /// The sentence starts with one of the phrases (case-insensitive, on a
    /// word boundary).
    Phrases(&'static [&'static str]),
    /// As `Phrases`, but matched exactly as written (for `TODO`, which
    /// "todo list" must not trigger).
    Exact(&'static [&'static str]),
    /// `<Name> will|to send|get back|reply|confirm|respond`.
    NameThen,
    /// `ask <Name>`.
    Ask,
}

/// One detection rule: a kind and how its sentences start. The table is
/// data so other languages can be added with a corpus each.
#[derive(Clone, Copy, Debug)]
struct Rule {
    kind: LoopKind,
    shape: Shape,
}

/// The English rules, tried in order.
const RULES: &[Rule] = &[
    Rule {
        kind: LoopKind::Waiting,
        shape: Shape::Phrases(&[
            "i'm waiting on",
            "i am waiting on",
            "i'm waiting for",
            "i am waiting for",
            "still waiting on",
            "still waiting for",
            "waiting on",
            "waiting for",
            "pending from",
        ]),
    },
    Rule {
        kind: LoopKind::Waiting,
        shape: Shape::NameThen,
    },
    Rule {
        kind: LoopKind::Mine,
        shape: Shape::Phrases(&[
            "i'll",
            "i will",
            "i need to",
            "i have to",
            "i must",
            "i should",
            "need to",
            "have to",
            "must",
            "remember to",
            "don't forget to",
            "do not forget to",
            "follow up with",
            "follow up on",
            "to do:",
            "action:",
        ]),
    },
    Rule {
        kind: LoopKind::Mine,
        shape: Shape::Exact(&["TODO:", "TODO", "FIXME:", "FIXME", "todo:"]),
    },
    Rule {
        kind: LoopKind::Question,
        shape: Shape::Phrases(&["check whether", "check if", "find out"]),
    },
    Rule {
        kind: LoopKind::Question,
        shape: Shape::Ask,
    },
];

/// Words that follow a phrase and turn it into something that isn't a
/// promise ("I will not", "I should have", "must be").
const NEGATING: &[&str] = &["not", "never", "no", "have", "be", "been", "had"];

/// Capitalised words that start sentences without being names.
const NOT_NAMES: &[&str] = &[
    "I", "We", "You", "They", "He", "She", "It", "This", "That", "The", "There", "These", "Those",
    "What", "When", "Where", "Why", "How", "Who", "Which", "My", "Our", "Your", "Their", "His",
    "Her", "Its", "If", "But", "And", "Or", "So", "Then", "Also", "Maybe", "Today", "Tomorrow",
    "Nobody", "Everyone", "Someone", "Anyone", "Nothing", "Everything",
];

/// Verbs that make `<Name> will|to <verb>` a thing awaited.
const AWAITED: &[&str] = &["send", "get back", "reply", "confirm", "respond"];

/// Lowercases `text` and reads `’` as `'`, so one table covers both.
fn fold(text: &str) -> String {
    text.chars()
        .map(|c| if c == '’' { '\'' } else { c })
        .flat_map(char::to_lowercase)
        .collect()
}

/// The sentences of one line of prose as `(start byte, text)`, in order.
/// A sentence ends at `.`, `!` or `?` followed by a space, a closing mark
/// or the end of the line; the terminator stays with it. Sentences that
/// start inside quotes or inline code are left out.
fn sentences(line: &str) -> Vec<(usize, &str)> {
    let mut out = Vec::new();
    let mut quoted = false;
    let mut code = false;
    // Where the sentence in progress began and whether it began quoted.
    let mut begun: Option<(usize, bool)> = None;
    let mut chars = line.char_indices().peekable();
    while let Some((at, c)) = chars.next() {
        if begun.is_none() && !c.is_whitespace() {
            let opens = matches!(c, '"' | '“' | '`');
            begun = Some((at, quoted || code || opens));
        }
        match c {
            '`' => code = !code,
            '"' => quoted = !quoted,
            '“' => quoted = true,
            '”' => quoted = false,
            _ => {}
        }
        let ends = matches!(c, '.' | '!' | '?')
            && chars.peek().is_none_or(|&(_, next)| {
                next.is_whitespace() || matches!(next, '"' | '”' | ')' | '*' | '_')
            });
        if ends && let Some((start, hidden)) = begun.take() {
            if !hidden {
                out.push((start, &line[start..at + c.len_utf8()]));
            }
        }
    }
    if let Some((start, false)) = begun {
        out.push((start, &line[start..]));
    }
    out
}

/// The loop a sentence is, if any: its kind, where its words start (after
/// bullets and an "and"/"also" joiner) and the length of the matched
/// phrase from there. `line_len` is the whole line's length, for the
/// "short line" test of questions.
fn classify(sentence: &str, line_len: usize) -> Option<(LoopKind, usize, usize)> {
    let mut begin = 0;
    loop {
        let rest = &sentence[begin..];
        let trimmed = rest.trim_start_matches([' ', '\t', '-', '*', '_', '+']);
        begin += rest.len() - trimmed.len();
        let rest = &sentence[begin..];
        let joiner = ["and ", "also ", "then "]
            .iter()
            .find(|j| rest.len() >= j.len() && rest.is_char_boundary(j.len()) && fold(&rest[..j.len()]) == **j);
        match joiner {
            Some(j) => begin += j.len(),
            None => break,
        }
    }
    let words = &sentence[begin..];
    if words.is_empty() {
        return None;
    }
    let ended_with_question = words.trim_end().ends_with('?');
    for rule in RULES {
        let hit = match rule.shape {
            Shape::Phrases(phrases) => phrases.iter().find_map(|p| phrase_len(words, p, false)),
            Shape::Exact(phrases) => phrases.iter().find_map(|p| phrase_len(words, p, true)),
            Shape::NameThen => name_then(words),
            Shape::Ask => ask_name(words),
        };
        let Some(len) = hit else { continue };
        let rest = words[len..].trim_start_matches([' ', ':', '-', '\t']);
        if rest.chars().filter(|c| c.is_alphanumeric()).count() < 3 {
            continue;
        }
        if rule.kind == LoopKind::Mine && matches!(rule.shape, Shape::Phrases(_)) {
            let next_word: String = fold(rest)
                .chars()
                .take_while(|c| c.is_alphabetic() || *c == '\'')
                .collect();
            if NEGATING.contains(&next_word.as_str()) || next_word.ends_with("n't") {
                continue;
            }
        }
        if rule.kind == LoopKind::Question && !(ended_with_question || line_len < 80) {
            continue;
        }
        return Some((rule.kind, begin, len));
    }
    None
}

/// The byte length of `phrase` at the start of `words` when it is there on
/// a word boundary.
fn phrase_len(words: &str, phrase: &str, exact: bool) -> Option<usize> {
    let mut taken = 0;
    let mut chars = words.chars();
    for want in phrase.chars() {
        let got = chars.next()?;
        taken += got.len_utf8();
        let same = if exact {
            got == want
        } else {
            fold(&got.to_string()) == want.to_string()
        };
        if !same {
            return None;
        }
    }
    if phrase.ends_with(':') {
        return Some(taken);
    }
    match words[taken..].chars().next() {
        Some(c) if c.is_alphanumeric() => None,
        Some(c) if (c == '\'' || c == '’') && !phrase.ends_with('\'') => None,
        _ => Some(taken),
    }
}

/// `<Name> will|to <verb>`: the length through the verb.
fn name_then(words: &str) -> Option<usize> {
    let name_end = words
        .char_indices()
        .find(|(_, c)| !c.is_alphabetic())
        .map_or(words.len(), |(at, _)| at);
    let name = &words[..name_end];
    let first = name.chars().next()?;
    if !first.is_uppercase() || name.chars().count() < 2 || NOT_NAMES.contains(&name) {
        return None;
    }
    // A name written in capitals is an acronym or a shout.
    if name.chars().all(char::is_uppercase) {
        return None;
    }
    let after = words[name_end..].strip_prefix(' ')?;
    let lower = fold(after);
    let connector = ["will ", "to "].iter().find(|c| lower.starts_with(**c))?;
    let verbs = &lower[connector.len()..];
    let verb = AWAITED.iter().find(|v| {
        verbs.starts_with(**v)
            && !verbs[v.len()..]
                .chars()
                .next()
                .is_some_and(char::is_alphanumeric)
    })?;
    // `fold` can change byte lengths for odd letters; stay on the original.
    let through = name_end + 1 + connector.len() + verb.len();
    words.is_char_boundary(through).then_some(through)
}

/// `ask <Name>`: the length through the name.
fn ask_name(words: &str) -> Option<usize> {
    let head = words.get(..4).filter(|w| fold(w) == "ask ")?;
    let tail = &words[head.len()..];
    let name_end = tail
        .char_indices()
        .find(|(_, c)| !c.is_alphabetic())
        .map_or(tail.len(), |(at, _)| at);
    let name = &tail[..name_end];
    let first = name.chars().next()?;
    if !first.is_uppercase() || NOT_NAMES.contains(&name) {
        return None;
    }
    Some(head.len() + name_end)
}

/// Cuts `text` to [`MAX_TEXT_CHARS`] characters.
fn clip(text: &str) -> &str {
    match text.char_indices().nth(MAX_TEXT_CHARS) {
        Some((at, _)) => text[..at].trim_end(),
        None => text,
    }
}

/// The loops in a markdown body, outside code fences, front matter,
/// headings, block quotes and task items, in order.
pub fn loop_items(body: &str) -> Vec<LoopItem> {
    let mut items = Vec::new();
    let mut fence: Option<(u8, usize)> = None;
    let mut front_matter = false;
    for (ix, line) in body.lines().enumerate() {
        if ix == 0 && line.trim_end() == "---" {
            front_matter = true;
            continue;
        }
        if front_matter {
            if matches!(line.trim_end(), "---" | "...") {
                front_matter = false;
            }
            continue;
        }
        let trimmed = line.trim_start();
        if let Some(open) = fence_marker(trimmed) {
            fence = match fence {
                None => Some(open),
                Some((ch, len)) if open.0 == ch && open.1 >= len => None,
                other => other,
            };
            continue;
        }
        if fence.is_some()
            || trimmed.starts_with(['#', '>', '|'])
            || task_marker(line).is_some()
            || line.starts_with("    ")
        {
            continue;
        }
        let prose = list_content(trimmed);
        for (_, sentence) in sentences(prose) {
            let Some((kind, begin, len)) = classify(sentence, prose.len()) else {
                continue;
            };
            let text = clip(sentence[begin..].trim()).trim_end_matches(['*', '_']);
            let mut end = len.min(text.len());
            while !text.is_char_boundary(end) {
                end -= 1;
            }
            items.push(LoopItem {
                line: ix as u32,
                kind,
                due: due_date(text),
                span: (0, end),
                text: text.to_owned(),
            });
        }
    }
    items
}

/// A line without its list marker (`- `, `* `, `1. `).
fn list_content(trimmed: &str) -> &str {
    if let Some(rest) = trimmed.strip_prefix(['-', '*', '+'])
        && rest.starts_with([' ', '\t'])
    {
        return rest.trim_start();
    }
    let digits = trimmed.bytes().take_while(u8::is_ascii_digit).count();
    if (1..=9).contains(&digits)
        && let Some(rest) = trimmed[digits..].strip_prefix(['.', ')'])
        && rest.starts_with([' ', '\t'])
    {
        return rest.trim_start();
    }
    trimmed
}

/// A stable 64-bit FNV-1a hash of `text`.
fn fnv1a(text: &str) -> u64 {
    let mut hash: u64 = 0xcbf2_9ce4_8422_2325;
    for byte in text.bytes() {
        hash ^= u64::from(byte);
        hash = hash.wrapping_mul(0x0100_0000_01b3);
    }
    hash
}

/// A sentence with its whitespace collapsed and its case, curly quotes and
/// closing punctuation ignored: what a loop's key is made from.
pub fn normalize(text: &str) -> String {
    let folded = fold(text);
    let words: Vec<&str> = folded.split_whitespace().collect();
    words
        .join(" ")
        .trim_end_matches(['.', '!', ',', ';'])
        .to_owned()
}

/// The key a loop is known by: `<note id>~<hash of its sentence>`. The
/// same sentence on any device, at any line, gets the same key. (Like
/// [`crate::reminders::task_key`], without a due date.)
pub fn loop_key(note_id: &str, text: &str) -> String {
    format!("{note_id}~{:016x}", fnv1a(&normalize(text)))
}

/// A loop an AI pass proposed and the person approved.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AiLoop {
    pub key: String,
    pub text: String,
    pub kind: LoopKind,
    pub approved_at: i64,
}

/// What a note's `kansoLoops` field holds.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Loops {
    /// Loop key to when it was closed (ms since the epoch).
    pub closed: BTreeMap<String, i64>,
    /// Loop key to when it was dismissed ("Not a loop").
    pub dismissed: BTreeMap<String, i64>,
    /// Approved AI loops, by key.
    pub ai: BTreeMap<String, AiLoop>,
}

impl Loops {
    pub fn is_empty(&self) -> bool {
        self.closed.is_empty() && self.dismissed.is_empty() && self.ai.is_empty()
    }

    /// Whether the loop `key` was closed or dismissed.
    pub fn is_hidden(&self, key: &str) -> bool {
        self.closed.contains_key(key) || self.dismissed.contains_key(key)
    }
}

fn read_times(value: Option<&Value>) -> BTreeMap<String, i64> {
    value
        .and_then(Value::as_object)
        .map(|map| {
            map.iter()
                .filter_map(|(key, at)| Some((key.clone(), at.as_i64()?)))
                .collect()
        })
        .unwrap_or_default()
}

/// The loops a note's fields hold; anything malformed is left out.
pub fn read_loops(fields: &Map<String, Value>) -> Loops {
    let Some(field) = fields.get(LOOPS_FIELD).and_then(Value::as_object) else {
        return Loops::default();
    };
    let ai = field
        .get("ai")
        .and_then(Value::as_array)
        .map(|list| {
            list.iter()
                .filter_map(|item| {
                    let text = item.get("text")?.as_str()?.to_owned();
                    let key = item.get("key")?.as_str()?.to_owned();
                    let kind = LoopKind::parse(item.get("kind")?.as_str()?)?;
                    let approved_at = item.get("approvedAt").and_then(Value::as_i64).unwrap_or(0);
                    Some((
                        key.clone(),
                        AiLoop {
                            key,
                            text,
                            kind,
                            approved_at,
                        },
                    ))
                })
                .collect()
        })
        .unwrap_or_default();
    Loops {
        closed: read_times(field.get("closed")),
        dismissed: read_times(field.get("dismissed")),
        ai,
    }
}

/// The loops as the note field stores them.
pub fn loops_value(loops: &Loops) -> Value {
    let times = |map: &BTreeMap<String, i64>| {
        Value::Object(map.iter().map(|(k, v)| (k.clone(), json!(v))).collect())
    };
    json!({
        "closed": times(&loops.closed),
        "dismissed": times(&loops.dismissed),
        "ai": loops.ai.values().map(|a| json!({
            "key": a.key,
            "text": a.text,
            "kind": a.kind.as_str(),
            "approvedAt": a.approved_at,
        })).collect::<Vec<_>>(),
    })
}

fn union_times(a: &BTreeMap<String, i64>, b: &BTreeMap<String, i64>) -> BTreeMap<String, i64> {
    let mut out = a.clone();
    for (key, at) in b {
        out.entry(key.clone())
            .and_modify(|mine| *mine = (*mine).min(*at))
            .or_insert(*at);
    }
    out
}

/// Both sets of loops: every key either has, with the earlier time where
/// both do. An AI loop on both sides keeps the one approved earlier (ties
/// broken by text and kind), so the result doesn't depend on the order.
/// Commutative, associative and idempotent.
pub fn merge_loops(a: &Loops, b: &Loops) -> Loops {
    let mut ai = a.ai.clone();
    for (key, other) in &b.ai {
        ai.entry(key.clone())
            .and_modify(|mine| {
                let mine_first =
                    (mine.approved_at, &mine.text, mine.kind) <= (other.approved_at, &other.text, other.kind);
                if !mine_first {
                    *mine = other.clone();
                }
            })
            .or_insert_with(|| other.clone());
    }
    Loops {
        closed: union_times(&a.closed, &b.closed),
        dismissed: union_times(&a.dismissed, &b.dismissed),
        ai,
    }
}

/// The loops after hiding `key` (closed, or dismissed when `dismissed`)
/// at `at`, keeping only the entries whose keys are in `live` (the note's
/// loops) besides it, so the field doesn't grow for sentences long gone.
/// Approved AI loops stay as they are.
pub fn hide(loops: &Loops, key: &str, dismissed: bool, at: i64, live: &[String]) -> Loops {
    let prune = |map: &BTreeMap<String, i64>| -> BTreeMap<String, i64> {
        map.iter()
            .filter(|(k, _)| k.as_str() == key || live.contains(k))
            .map(|(k, v)| (k.clone(), *v))
            .collect()
    };
    let mut next = Loops {
        closed: prune(&loops.closed),
        dismissed: prune(&loops.dismissed),
        ai: loops.ai.clone(),
    };
    let target = if dismissed {
        &mut next.dismissed
    } else {
        &mut next.closed
    };
    target.entry(key.to_owned()).or_insert(at);
    next
}

/// The line (0-based) holding `text` (whitespace-normalised) in `body`,
/// preferring the one nearest `near`; `None` when no line has it. Code
/// fences and task items are skipped, as detection skips them.
pub fn locate(body: &str, text: &str, near: Option<u32>) -> Option<u32> {
    let wanted = normalize(text);
    if wanted.is_empty() {
        return None;
    }
    let mut fence: Option<(u8, usize)> = None;
    let mut best: Option<u32> = None;
    for (ix, line) in body.lines().enumerate() {
        if let Some(open) = fence_marker(line.trim_start()) {
            fence = match fence {
                None => Some(open),
                Some((ch, len)) if open.0 == ch && open.1 >= len => None,
                other => other,
            };
            continue;
        }
        if fence.is_some() || task_marker(line).is_some() {
            continue;
        }
        if fold_line(line).contains(&wanted) {
            let line = ix as u32;
            let better = match (best, near) {
                (None, _) => true,
                (Some(b), Some(n)) => line.abs_diff(n) < b.abs_diff(n),
                (Some(_), None) => false,
            };
            if better {
                best = Some(line);
            }
        }
    }
    best
}

/// The line (0-based) of `body` to edit for the loop with sentence `text`
/// that was seen on `line`: that line when it still holds the sentence,
/// else the nearest line that does.
fn line_holding(lines: &[&str], line: u32, text: &str) -> Option<usize> {
    if lines.get(line as usize).is_some_and(|l| l.contains(text)) {
        return Some(line as usize);
    }
    lines
        .iter()
        .enumerate()
        .filter(|(_, l)| l.contains(text))
        .map(|(ix, _)| ix)
        .min_by_key(|ix| ix.abs_diff(line as usize))
}

/// `body` with its lines replaced by `lines`, in the body's line ending
/// and with its final newline kept.
fn rebuilt(body: &str, lines: &[String]) -> String {
    let eol = if body.contains("\r\n") { "\r\n" } else { "\n" };
    let mut out = lines.join(eol);
    if body.ends_with('\n') {
        out.push_str(eol);
    }
    out
}

/// `body` with the sentence `text` (seen on `line`) made a task:
/// `- [ ] <sentence>` on a line of its own, with what else was on its line
/// kept beside it. `None` when the sentence isn't in the body any more.
pub fn make_task(body: &str, line: u32, text: &str) -> Option<String> {
    let lines: Vec<&str> = body.lines().collect();
    let at = line_holding(&lines, line, text)?;
    let original = lines[at];
    let content_start = original.len() - list_content(original.trim_start()).len();
    let indent = &original[..original.len() - original.trim_start().len()];
    let lead = &original[..content_start];
    let content = &original[content_start..];
    let pos = content.find(text)?;
    let before = content[..pos].trim();
    let after = content[pos + text.len()..].trim();
    let mut replaced: Vec<String> = Vec::new();
    if !before.is_empty() {
        replaced.push(format!("{lead}{before}"));
    }
    replaced.push(format!("{indent}- [ ] {text}"));
    if !after.is_empty() {
        replaced.push(format!("{indent}{after}"));
    }
    let mut out: Vec<String> = lines.iter().map(|l| (*l).to_owned()).collect();
    out.splice(at..=at, replaced);
    Some(rebuilt(body, &out))
}

/// The mark "Mark done loops in the note" adds after a closed sentence.
pub const DONE_MARK: &str = " \u{2713}";

/// `body` with the sentence `text` (seen on `line`) followed by
/// [`DONE_MARK`]. `None` when it isn't in the body, or is marked already.
pub fn mark_done(body: &str, line: u32, text: &str) -> Option<String> {
    let lines: Vec<&str> = body.lines().collect();
    let at = line_holding(&lines, line, text)?;
    let original = lines[at];
    let pos = original.find(text)? + text.len();
    if original[pos..].starts_with(DONE_MARK) {
        return None;
    }
    let mut out: Vec<String> = lines.iter().map(|l| (*l).to_owned()).collect();
    out[at] = format!("{}{DONE_MARK}{}", &original[..pos], &original[pos..]);
    Some(rebuilt(body, &out))
}

/// A line folded and whitespace-collapsed, without trimming its end (so a
/// sentence in the middle of it is still found).
fn fold_line(line: &str) -> String {
    fold(line).split_whitespace().collect::<Vec<_>>().join(" ")
}

/// The loop's age as a phrase ("today", "3 days ago", "2 weeks ago") for a
/// change at `then_ms`, seen at `now_ms`.
pub fn age_label(then_ms: i64, now_ms: i64) -> String {
    match age_days(then_ms, now_ms) {
        0 => "today".to_owned(),
        1 => "yesterday".to_owned(),
        d if d < 14 => format!("{d} days ago"),
        d if d < 60 => format!("{} weeks ago", d / 7),
        d => format!("{} months ago", d / 30),
    }
}

/// Whole days from `then_ms` to `now_ms` (0 when it is in the future).
pub fn age_days(then_ms: i64, now_ms: i64) -> i64 {
    (now_ms - then_ms).max(0) / 86_400_000
}

#[cfg(test)]
mod tests;
